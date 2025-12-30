use colored::*;
use std::path::Path;
use std::process::Command;

pub fn run_uninstall() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "开始卸载 Cyber Zen Tools...".yellow());
    
    let install_path = Path::new("/usr/local/bin/cyber-zen");
    
    if !install_path.exists() {
        println!("{} {}", "程序未安装:".yellow(), install_path.display());
        return Ok(());
    }
    
    println!("{} {}", "删除安装文件:".yellow(), install_path.display());
    
    let output = Command::new("sudo")
        .arg("rm")
        .arg("-f")
        .arg(install_path)
        .output()?;
    
    if !output.status.success() {
        return Err(format!("删除安装文件失败: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    println!("{}", "清理构建目录...".yellow());
    let build_dir = Path::new("build");
    if build_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(build_dir) {
            println!("{} {}", "清理构建目录失败:".yellow(), e);
        }
    }
    
    println!("{}", "✓ 卸载完成！".green());
    Ok(())
}

