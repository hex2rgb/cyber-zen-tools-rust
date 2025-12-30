use colored::*;
use crate::config::FileTypeManager;
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub struct ChangeInfo {
    file: String,
    status: String,
    category: String,
    file_type: String,
}

pub fn run_gcm(message: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let msg = if let Some(m) = message {
        println!("{} {}", "使用用户提供的提交信息:".cyan(), m);
        m
    } else {
        println!("{}", "未提供提交信息，正在自动分析变更...".yellow());
        match generate_commit_message() {
            Ok(m) => {
                println!("{}", "自动生成成功！".green());
                m
            }
            Err(e) => {
                println!("{} {}", "自动生成失败:".red(), e);
                println!("{}", "使用默认提交信息: update".yellow());
                "update".to_string()
            }
        }
    };

    println!("{}", "开始执行 Git 操作...".green());
    println!("{} {}", "提交信息:".cyan(), msg);

    check_git_repo()?;

    println!("{}", "执行: git add .".yellow());
    exec_git_command(&["add", "."])?;
    println!("{}", "✓ git add . 完成".green());

    println!("{} {}", "执行: git commit -m \"{}\" --no-verify".yellow(), msg);
    exec_git_command(&["commit", "-m", &msg, "--no-verify"])?;
    println!("{}", "✓ git commit 完成".green());

    println!("{}", "执行: git push".yellow());
    exec_git_command(&["push"])?;
    println!("{}", "✓ git push 完成".green());

    println!("{}", "🎉 Git 操作完成！".green());
    Ok(())
}

fn generate_commit_message() -> Result<String, Box<dyn std::error::Error>> {
    check_git_repo()?;

    let file_type_manager = FileTypeManager::new()?;
    let changes = analyze_git_changes(&file_type_manager)?;

    display_changes(&changes);
    let message = generate_message_from_changes(&changes, &file_type_manager);

    println!("\n{}", " 生成的 Commit Message:".cyan());
    println!("{}", message);

    if !confirm_with_user("是否使用此消息? [Y/n] ") {
        return Err("用户取消操作".into());
    }

    Ok(message)
}

fn analyze_git_changes(file_type_manager: &FileTypeManager) -> Result<Vec<ChangeInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.trim().lines().collect();

    let mut changes = Vec::new();

    for line in lines {
        if line.is_empty() || line.len() < 3 {
            continue;
        }

        let status = line.chars().next().unwrap().to_string();
        let file = line[3..].trim().to_string();

        let change = ChangeInfo {
            file: file.clone(),
            status: status.clone(),
            category: file_type_manager.get_file_category(&file),
            file_type: file_type_manager.get_file_type(&file),
        };

        changes.push(change);
    }

    Ok(changes)
}

fn display_changes(changes: &[ChangeInfo]) {
    println!("{}", " 检测到 Git 变更...\n".cyan());
    println!("{}", "📁 文件变更状态:".yellow());

    for change in changes {
        match change.status.as_str() {
            "A" => println!("  {} {}", "✨ 新增:".green(), change.file),
            "M" => println!("  {} {}", "🔧 修改:".blue(), change.file),
            "D" => println!("  {} {}", "🗑️  删除:".red(), change.file),
            "R" => println!("  {} {}", "🔄 重命名:".yellow(), change.file),
            _ => println!("  {} {}: {}", "❓".cyan(), change.status, change.file),
        }
    }

    display_change_stats(changes);
}

fn display_change_stats(changes: &[ChangeInfo]) {
    let mut added = 0;
    let mut modified = 0;
    let mut deleted = 0;

    for change in changes {
        match change.status.as_str() {
            "A" => added += 1,
            "M" => modified += 1,
            "D" => deleted += 1,
            _ => {}
        }
    }

    println!();
    println!("{}", " 变更统计:".cyan());
    println!("  新增文件: {} 个", added);
    println!("  修改文件: {} 个", modified);
    println!("  删除文件: {} 个", deleted);
    println!("  总变更: {} 个文件", changes.len());
}

fn generate_message_from_changes(changes: &[ChangeInfo], file_type_manager: &FileTypeManager) -> String {
    if changes.is_empty() {
        return "update".to_string();
    }

    let mut added = 0;
    let mut modified = 0;
    let mut deleted = 0;
    let mut categories: HashMap<String, i32> = HashMap::new();

    for change in changes {
        match change.status.as_str() {
            "A" => added += 1,
            "M" => modified += 1,
            "D" => deleted += 1,
            _ => {}
        }
        *categories.entry(change.category.clone()).or_insert(0) += 1;
    }

    let commit_type = file_type_manager.get_commit_type(added, modified, deleted);
    let summary = generate_summary(changes, &categories);
    let details = generate_details(changes, file_type_manager);

    format!("{}: {}\n\n{}", commit_type, summary, details)
}

fn generate_summary(changes: &[ChangeInfo], categories: &HashMap<String, i32>) -> String {
    if changes.len() == 1 {
        let change = &changes[0];
        return match change.status.as_str() {
            "A" => format!("新增{}", change.category),
            "M" => format!("优化{}", change.category),
            "D" => format!("清理{}", change.category),
            _ => "更新项目文件".to_string(),
        };
    }

    if categories.len() == 1 {
        if let Some(category) = categories.keys().next() {
            return format!("更新{}", category);
        }
    }

    let main_categories: Vec<String> = categories
        .iter()
        .filter(|(_, &count)| count > 1)
        .map(|(category, _)| category.clone())
        .collect();

    if !main_categories.is_empty() {
        return format!("更新{}", main_categories.join("、"));
    }

    "更新项目文件".to_string()
}

fn generate_details(changes: &[ChangeInfo], file_type_manager: &FileTypeManager) -> String {
    let mut details = Vec::new();

    for change in changes {
        let action = match change.status.as_str() {
            "A" => file_type_manager.get_action_description("added"),
            "M" => file_type_manager.get_action_description("modified"),
            "D" => file_type_manager.get_action_description("deleted"),
            "R" => file_type_manager.get_action_description("renamed"),
            _ => change.status.clone(),
        };

        details.push(format!("- {} {}", action, change.file));
    }

    details.join("\n")
}

fn confirm_with_user(prompt: &str) -> bool {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();

    let response = response.trim().to_lowercase();
    response.is_empty() || response == "y" || response == "yes"
}

fn check_git_repo() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .output()?;

    if !output.status.success() {
        return Err("当前目录不是 Git 仓库".into());
    }

    Ok(())
}

fn exec_git_command(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.stdin(Stdio::inherit());

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("git {} 失败", args.join(" ")).into());
    }

    Ok(())
}

