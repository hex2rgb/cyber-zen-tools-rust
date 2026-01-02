use colored::*;
use crate::config::{FileTypeManager, get_model_dir};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::commands::candle_model::CandleModel;

pub struct ChangeInfo {
    file: String,
    status: String,
    category: String,
    file_type: String,
}

pub fn run_gcm_ai(
    message: Option<String>,
    rewrite: bool,
    max_commits: Option<usize>,
    dry_run: bool,
    model: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 查找模型文件
    let model_path = find_model_file(model)?;
    println!("{} {}", "✓ 找到模型文件:".green(), model_path.display());
    
    if rewrite {
        // 重写历史提交
        rewrite_commit_history(max_commits, dry_run, &model_path)?;
    } else {
        // 生成新提交消息
        let msg = if let Some(m) = message {
            println!("{} {}", "使用用户提供的提交信息:".cyan(), m);
            m
        } else {
            println!("{}", "正在使用 AI 生成提交信息...".yellow());
            match generate_ai_commit_message(&model_path) {
                Ok(m) => {
                    println!("{}", "AI 生成成功！".green());
                    m
                }
                Err(e) => {
                    eprintln!("{} {}", "AI 生成失败:".red(), e);
                    eprintln!("{}", "详细错误信息:".yellow());
                    eprintln!("{}", format!("{:?}", e));
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
    }

    Ok(())
}

/// 查找模型文件（支持 Safetensors 格式）
/// 优先级：1. 命令行指定 2. default_* 目录中的 model.safetensors 3. default_*.safetensors 文件
fn find_model_file(model_name: Option<String>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let model_dir = get_model_dir();
    
    // 确保模型目录存在
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir)?;
    }
    
    // 1. 如果指定了 --model 参数
    if let Some(name) = model_name {
        // 先尝试在子目录中查找
        let dir_path = model_dir.join(&name);
        let file_in_dir = dir_path.join("model.safetensors");
        if file_in_dir.exists() {
            return Ok(file_in_dir);
        }
        
        // 尝试直接文件
        let path = model_dir.join(format!("{}.safetensors", name));
        if path.exists() {
            return Ok(path);
        }
        
        // 兼容旧格式 .gguf（如果存在）
        let path_gguf = model_dir.join(format!("{}.gguf", name));
        if path_gguf.exists() {
            eprintln!("⚠️  警告: 找到 .gguf 格式文件，建议使用 .safetensors 格式");
            return Ok(path_gguf);
        }
        
        return Err(format!(
            "模型文件不存在: {}，请检查以下位置：\n  1. {}/model.safetensors\n  2. {}.safetensors",
            name,
            dir_path.display(),
            model_dir.join(&name).display()
        ).into());
    }
    
    // 2. 查找 default_* 目录中的 model.safetensors
    let mut default_models = Vec::new();
    if model_dir.exists() {
        for entry in std::fs::read_dir(&model_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // 如果是目录且以 default_ 开头
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with("default_") {
                        let model_file = path.join("model.safetensors");
                        if model_file.exists() {
                            default_models.push(model_file);
                        }
                    }
                }
            }
            
            // 如果是文件且以 default_ 开头，以 .safetensors 结尾
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".safetensors") && file_name.starts_with("default_") {
                        default_models.push(path);
                    }
                }
            }
        }
    }
    
    // 如果没找到 .safetensors，尝试兼容 .gguf（旧格式）
    if default_models.is_empty() {
        for entry in std::fs::read_dir(&model_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // 目录中的 .gguf
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with("default_") {
                        let model_file = path.join("model.gguf");
                        if model_file.exists() {
                            default_models.push(model_file);
                        }
                    }
                }
            }
            
            // 直接文件 .gguf
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".gguf") && file_name.starts_with("default_") {
                        default_models.push(path);
                    }
                }
            }
        }
        if !default_models.is_empty() {
            eprintln!("⚠️  警告: 找到 .gguf 格式文件，建议使用 .safetensors 格式");
        }
    }
    
    if default_models.is_empty() {
        return Err(format!(
            "未找到默认模型文件，请检查以下位置：\n  1. {}/default_*/model.safetensors（目录形式）\n  2. {}/default_*.safetensors（文件形式）",
            model_dir.display(),
            model_dir.display()
        ).into());
    }
    
    if default_models.len() > 1 {
        eprintln!("⚠️  警告: 找到多个默认模型文件，使用第一个: {}", 
                 default_models[0].display());
    }
    
    Ok(default_models[0].clone())
}

fn generate_ai_commit_message(model_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    check_git_repo()?;

    let file_type_manager = FileTypeManager::new()?;
    let changes = analyze_git_changes(&file_type_manager)?;

    if changes.is_empty() {
        return Err("没有检测到 Git 变更".into());
    }

    display_changes(&changes);
    
    // 构建 prompt
    let prompt = build_commit_prompt(&changes, &file_type_manager);
    
    // 调用本地模型生成消息
    let ai_message = call_local_model(model_path, &prompt)?;
    
    println!("\n{}", " 生成的 AI Commit Message:".cyan());
    println!("{}", ai_message);

    if !confirm_with_user("是否使用此消息? [Y/n] ") {
        return Err("用户取消操作".into());
    }

    Ok(ai_message)
}

fn build_commit_prompt(changes: &[ChangeInfo], _file_type_manager: &FileTypeManager) -> String {
    let mut prompt = String::from("根据以下 Git 变更，生成一个符合 Conventional Commits 规范的 commit message。\n\n");
    prompt.push_str("变更内容：\n");
    
    for change in changes {
        let action = match change.status.as_str() {
            "A" => "新增",
            "M" => "修改",
            "D" => "删除",
            "R" => "重命名",
            _ => "变更",
        };
        prompt.push_str(&format!("- {} {} ({})\n", action, change.file, change.category));
    }
    
    prompt.push_str("\n要求：\n");
    prompt.push_str("1. 使用中文\n");
    prompt.push_str("2. 格式：<type>: <description>\n");
    prompt.push_str("3. type 可以是：feat, fix, refactor, style, docs, test, chore, perf, cleanup\n");
    prompt.push_str("4. description 要简洁明了，描述主要变更\n");
    prompt.push_str("5. 只返回 commit message，不要其他说明\n");
    
    prompt
}

/// 调用本地模型生成文本（使用 Candle）
fn call_local_model(model_path: &PathBuf, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("{} {}", "正在加载本地模型:".yellow(), model_path.display());
    println!("{} {}", "模型文件存在:".cyan(), model_path.exists());
    
    // 加载模型
    let mut model = match CandleModel::load_from_path(model_path) {
        Ok(model) => model,
        Err(e) => {
            eprintln!("{}", "模型加载详细错误:".red());
            eprintln!("{}", format!("{:?}", e));
            eprintln!("{}", format!("错误链: {}", e.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(" -> ")));
            return Err(format!("模型加载失败: {}", e).into());
        }
    };
    
    println!("{}", "✓ 模型加载成功".green());
    println!("{}", "正在生成文本...".yellow());
    println!("{} {}", "输入 prompt 长度:".cyan(), prompt.len());
    
    // 生成文本（最大 200 tokens）
    let output = model.generate(prompt, 200)
        .map_err(|e| format!("文本生成失败: {}", e))?;
    
    println!("{} {}", "生成文本长度:".cyan(), output.len());
    println!("{} {}", "生成内容:".cyan(), &output);
    
    // 验证输出格式
    if !output.contains(':') && !output.is_empty() {
        // 如果没有冒号，尝试添加默认类型
        if output.starts_with("feat") || output.starts_with("fix") || output.starts_with("refactor") {
            // 已经是正确的格式
        } else {
            // 尝试添加 feat: 前缀
            return Ok(format!("feat: {}", output));
        }
    }
    
    Ok(output)
}

fn rewrite_commit_history(
    max_commits: Option<usize>,
    dry_run: bool,
    model_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    check_git_repo()?;
    
    let limit = max_commits.unwrap_or(10);
    
    println!("{} {} {}", "准备重写最近".yellow(), limit, "个提交的 commit message...");
    
    if dry_run {
        println!("{}", "（预览模式，不会实际修改）".yellow());
    }
    
    // 获取最近的提交
    let commits = get_recent_commits(limit)?;
    
    if commits.is_empty() {
        return Err("没有找到提交记录".into());
    }
    
    println!("\n{} {} {}", "找到".cyan(), commits.len(), "个提交：");
    for (i, commit) in commits.iter().enumerate() {
        println!("  {} {}", format!("{}.", i + 1).yellow(), commit.message);
    }
    
    if !confirm_with_user("\n是否继续重写这些提交? [Y/n] ") {
        return Err("用户取消操作".into());
    }
    
    // 对每个提交重写消息
    let mut new_messages = Vec::new();
    for commit in &commits {
        println!("\n{} {}", "处理提交:".cyan(), commit.hash);
        println!("  原始消息: {}", commit.message);
        
        let prompt = build_rewrite_prompt(&commit.message, &commit.diff);
        match call_local_model(model_path, &prompt) {
            Ok(new_msg) => {
                println!("  新消息: {}", new_msg);
                new_messages.push((commit.hash.clone(), new_msg));
            }
            Err(e) => {
                println!("  {} {}", "重写失败:".red(), e);
                new_messages.push((commit.hash.clone(), commit.message.clone()));
            }
        }
    }
    
    if dry_run {
        println!("\n{}", "预览模式：以下是将会应用的更改".yellow());
        for (hash, new_msg) in &new_messages {
            println!("  {} -> {}", hash, new_msg);
        }
        return Ok(());
    }
    
    // 实际重写（使用 git rebase）
    println!("\n{}", "开始重写提交历史...".yellow());
    rewrite_with_rebase(&new_messages)?;
    
    println!("{}", "✓ 提交历史重写完成！".green());
    println!("{}", "提示: 如果已推送到远程，需要使用 'git push --force-with-lease' 更新".yellow());
    
    Ok(())
}

struct CommitInfo {
    hash: String,
    message: String,
    diff: String,
}

fn get_recent_commits(limit: usize) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("log")
        .arg(format!("-{}", limit))
        .arg("--pretty=format:%H|%s")
        .arg("--no-patch")
        .output()?;

    if !output.status.success() {
        return Err("获取提交列表失败".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            let message = parts[1..].join("|");
            
            // 获取这个提交的 diff
            let diff = get_commit_diff(&hash)?;
            
            commits.push(CommitInfo {
                hash,
                message,
                diff,
            });
        }
    }

    Ok(commits)
}

fn get_commit_diff(hash: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("show")
        .arg("--stat")
        .arg(hash)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::new())
    }
}

fn build_rewrite_prompt(original_message: &str, diff: &str) -> String {
    format!(
        "重写以下 commit message，使其更清晰、更符合 Conventional Commits 规范。\n\n\
        原始 commit message: {}\n\n\
        变更内容：\n{}\n\n\
        要求：\n\
        1. 使用中文\n\
        2. 格式：<type>: <description>\n\
        3. type 可以是：feat, fix, refactor, style, docs, test, chore, perf, cleanup\n\
        4. description 要简洁明了，描述主要变更\n\
        5. 只返回新的 commit message，不要其他说明",
        original_message,
        diff.lines().take(20).collect::<Vec<_>>().join("\n") // 限制 diff 长度
    )
}

fn rewrite_with_rebase(new_messages: &[(String, String)]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "使用 git filter-branch 重写提交历史...".yellow());
    println!("{}", "注意: 这是一个复杂操作，建议先备份分支".yellow());
    
    // 创建备份分支
    let backup_branch = format!("backup-before-rewrite-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    println!("{} {}", "创建备份分支:".yellow(), backup_branch);
    
    // 创建备份
    exec_git_command(&["branch", &backup_branch])?;
    println!("{} {}", "✓ 备份分支已创建:".green(), backup_branch);
    
    // 构建统一的消息过滤脚本（一次性处理所有提交）
    // 参考 git-rewrite-commits 的实现方式
    let mut script = String::from("case \"$GIT_COMMIT\" in\n");
    for (hash, new_msg) in new_messages {
        // 转义特殊字符
        let escaped_msg = new_msg
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`");
        script.push_str(&format!("    {})\n        echo \"{}\"\n        ;;\n", hash, escaped_msg));
    }
    script.push_str("    *)\n        cat\n        ;;\nesac\n");
    
    // 创建临时脚本文件
    use std::fs;
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("git-rewrite-msg-{}.sh", std::process::id()));
    fs::write(&script_path, &script)?;
    
    // 设置执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }
    
    // 获取要重写的提交范围（从最旧的到最新的）
    let oldest_hash = new_messages.last().map(|(h, _)| h.as_str()).unwrap_or("HEAD");
    let newest_hash = new_messages.first().map(|(h, _)| h.as_str()).unwrap_or("HEAD");
    
    println!("{} {}..{}", "重写范围:".cyan(), oldest_hash, newest_hash);
    
    // 使用 git filter-branch 一次性重写所有提交
    let output = Command::new("git")
        .arg("filter-branch")
        .arg("-f")
        .arg("--msg-filter")
        .arg(format!("bash {}", script_path.to_string_lossy()))
        .arg(format!("{}^..{}", oldest_hash, newest_hash))
        .output()?;
    
    // 清理临时脚本
    let _ = fs::remove_file(&script_path);
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git filter-branch 失败: {}", error).into());
    }
    
    println!("\n{}", "✓ 提交历史重写完成！".green());
    println!("{} {}", "备份分支:".cyan(), backup_branch);
    println!("{}", "如需恢复，运行: git reset --hard backup-branch".yellow());
    println!("{}", "如需删除备份，运行: git branch -D backup-branch".yellow());
    println!("{}", "提示: 如果已推送到远程，需要使用 'git push --force-with-lease' 更新".yellow());
    
    Ok(())
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
        // git status --porcelain 格式: "XY filename"
        // 前两个字符是状态，第三个是空格，从第四个字符开始是文件名
        let file = if line.len() > 3 {
            line.chars().skip(3).collect::<String>().trim().to_string()
        } else {
            continue;
        };

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

