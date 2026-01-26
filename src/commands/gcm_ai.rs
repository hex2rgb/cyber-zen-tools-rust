use colored::*;
use crate::config::{FileTypeManager, get_model_folder_path};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::commands::candle_model::CandleModel;
use crate::commands::candle_model_quantized::CandleModelQuantized;

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
) -> Result<(), Box<dyn std::error::Error>> {
    // 获取模型文件路径（使用配置的模型文件夹）
    let model_path = find_model_file()?;
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

            if dry_run {
                // 预览模式：只显示生成的 commit message，不执行 Git 操作
                println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow());
                println!("{}", "📋 预览模式（Dry Run）".yellow().bold());
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow());
                println!("{} {}", "生成的 Commit Message:".cyan().bold(), msg);
                println!("\n{}", "以下是将要执行的 Git 操作：".yellow());
                println!("  1. git add .");
                println!("  2. git commit -m \"{}\" --no-verify", msg);
                println!("  3. git push");
                println!("\n{}", "（预览模式，不会实际执行）".yellow());
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow());
            } else {
                // 实际执行 Git 操作
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
        }

    Ok(())
}

/// 查找模型文件（支持 safetensors 和 gguf 格式）
/// 使用配置的模型文件夹路径
fn find_model_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let model_folder = get_model_folder_path();
    
    // 检查模型文件夹是否存在
    if !model_folder.exists() {
        return Err(format!(
            "模型文件夹不存在: {}",
            model_folder.display()
        ).into());
    }
    
    // 优先检查 GGUF 文件（量化模型）
    if let Ok(entries) = model_folder.read_dir() {
        let mut gguf_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "gguf")
                    .unwrap_or(false)
            })
            .collect();
        
        if !gguf_files.is_empty() {
            gguf_files.sort();
            if let Some(gguf_file) = gguf_files.first() {
                println!("{} {}", "找到 GGUF 量化模型:".cyan(), gguf_file.display());
                return Ok(gguf_file.clone());
            }
        }
    }
    
    // 如果没有 GGUF 文件，检查 safetensors 文件（普通模型）
    let index_file = model_folder.join("model.safetensors.index.json");
    let single_file = model_folder.join("model.safetensors");
    
    // 检查是否有分片文件
    let mut has_shards = false;
    if let Ok(entries) = model_folder.read_dir() {
        has_shards = entries.filter_map(|e| e.ok())
            .any(|e| {
                e.path().file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("model-") && n.ends_with(".safetensors"))
                    .unwrap_or(false)
            });
    }
    
    // 返回第一个分片文件、单文件或 index 文件路径
    if has_shards {
        let mut shard_files: Vec<_> = model_folder.read_dir()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("model-") && n.ends_with(".safetensors"))
                    .unwrap_or(false)
            })
            .collect();
        shard_files.sort();
        if let Some(first_shard) = shard_files.first() {
            return Ok(first_shard.clone());
        }
    } else if single_file.exists() {
        return Ok(single_file);
    } else if index_file.exists() {
        return Ok(index_file);
    }
    
    Err(format!(
        "未找到模型文件，请检查: {}/model*.safetensors 或 *.gguf",
        model_folder.display()
    ).into())
}

fn generate_ai_commit_message(model_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    check_git_repo()?;

    let file_type_manager = FileTypeManager::new()?;
    let changes = analyze_git_changes(&file_type_manager)?;

    if changes.is_empty() {
        return Err("没有检测到 Git 变更".into());
    }

    display_changes(&changes);
    
    // 获取代码 diff
    println!("{}", "正在获取代码变更内容...".yellow());
    let diff = get_git_diff()?;
    
    if diff.trim().is_empty() {
        println!("{}", "警告: 未获取到代码变更内容，将仅使用文件名信息".yellow());
    } else {
        let diff_lines = diff.lines().count();
        println!("{} {} {}", "✓ 获取到代码变更:".green(), diff_lines, "行");
    }
    
    // 构建 prompt（包含代码 diff）
    let prompt = build_commit_prompt(&changes, &diff, &file_type_manager);
    
    // 调用本地模型生成消息
    let ai_message = call_local_model(model_path, &prompt)?;
    
    println!("\n{}", " 生成的 AI Commit Message:".cyan());
    println!("{}", ai_message);

    if !confirm_with_user("是否使用此消息? [Y/n] ") {
        return Err("用户取消操作".into());
    }

    Ok(ai_message)
}

/// 获取 Git diff（代码变更内容）
fn get_git_diff() -> Result<String, Box<dyn std::error::Error>> {
    // 首先尝试获取暂存区的 diff
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--no-color")
        .output()?;
    
    if output.status.success() {
        let diff = String::from_utf8_lossy(&output.stdout).to_string();
        if !diff.trim().is_empty() {
            return Ok(diff);
        }
    }
    
    // 如果没有暂存区变更，获取工作区的 diff
    let output = Command::new("git")
        .arg("diff")
        .arg("--no-color")
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::new())
    }
}

fn build_commit_prompt(
    changes: &[ChangeInfo], 
    diff: &str,
    _file_type_manager: &FileTypeManager
) -> String {
    // 简洁的 prompt（小模型友好，符合 next.md 规范）
    let mut prompt = String::from(
        "你是 Git 提交信息生成助手。\n\n根据下面的代码变更，总结一个简短的提交信息。\n\n"
    );
    
    // 代码 Diff（严格限制长度）
    if !diff.trim().is_empty() {
        prompt.push_str("代码变更：\n");
        prompt.push_str("```diff\n");
        
        // 限制 diff 长度（严格限制为 20 行，确保性能）
        const MAX_DIFF_LINES: usize = 20;
        let diff_lines: Vec<&str> = diff.lines().take(MAX_DIFF_LINES).collect();
        prompt.push_str(&diff_lines.join("\n"));
        
        let total_lines = diff.lines().count();
        if total_lines > MAX_DIFF_LINES {
            prompt.push_str(&format!("\n... (还有 {} 行变更，已截断)", total_lines - MAX_DIFF_LINES));
        }
        
        prompt.push_str("\n```\n");
    }
    
    // 简洁的要求
    prompt.push_str("\n要求：\n");
    prompt.push_str("- 使用中文\n");
    prompt.push_str("- 格式：<type>: <description>\n");
    prompt.push_str("- description 不超过 20 个字\n");
    prompt.push_str("- 只输出 commit message\n");
    
    // 检查 prompt 长度，如果超过 2000 字符则截断
    const MAX_PROMPT_LENGTH: usize = 2000;
    if prompt.len() > MAX_PROMPT_LENGTH {
        prompt.truncate(MAX_PROMPT_LENGTH);
        prompt.push_str("... (已截断)");
    }
    
    prompt
}

/// 调用本地模型生成文本（使用 Candle）
/// 支持 safetensors 和 gguf 两种格式
fn call_local_model(model_path: &PathBuf, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("{} {}", "正在加载本地模型:".yellow(), model_path.display());
    println!("{} {}", "模型文件存在:".cyan(), model_path.exists());
    
    // 检查文件扩展名，判断使用哪种模型加载方式
    let is_gguf = model_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "gguf")
        .unwrap_or(false);
    
    if is_gguf {
        // 使用量化模型（GGUF 格式）
        println!("{}", "检测到 GGUF 格式，使用量化模型加载".cyan());
        let mut model = match CandleModelQuantized::load_from_path(model_path) {
            Ok(model) => model,
            Err(e) => {
                eprintln!("{}", "量化模型加载详细错误:".red());
                eprintln!("{}", format!("{:?}", e));
                return Err(format!("量化模型加载失败: {}", e).into());
            }
        };
        
        println!("{}", "✓ 量化模型加载成功".green());
        println!("{}", "正在生成文本...".yellow());
        println!("{} {}", "输入 prompt 长度:".cyan(), prompt.len());
        
        // 生成文本（最大 32 tokens，commit message 永远不需要更多）
        let output = model.generate(prompt, 32)
            .map_err(|e| format!("文本生成失败: {}", e))?;
        
        println!("{} {}", "生成文本长度:".cyan(), output.len());
        println!("{} {}", "生成内容:".cyan(), &output);
        
        // 验证输出格式
        if !output.contains(':') && !output.is_empty() {
            if output.starts_with("feat") || output.starts_with("fix") || output.starts_with("refactor") {
                // 已经是正确的格式
            } else {
                return Ok(format!("feat: {}", output));
            }
        }
        
        Ok(output)
    } else {
        // 使用普通模型（safetensors 格式）
        println!("{}", "检测到 safetensors 格式，使用普通模型加载".cyan());
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
        
        // 生成文本（最大 32 tokens，commit message 永远不需要更多）
        let output = model.generate(prompt, 32)
            .map_err(|e| format!("文本生成失败: {}", e))?;
        
        println!("{} {}", "生成文本长度:".cyan(), output.len());
        println!("{} {}", "生成内容:".cyan(), &output);
        
        // 验证输出格式
        if !output.contains(':') && !output.is_empty() {
            if output.starts_with("feat") || output.starts_with("fix") || output.starts_with("refactor") {
                // 已经是正确的格式
            } else {
                return Ok(format!("feat: {}", output));
            }
        }
        
        Ok(output)
    }
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

