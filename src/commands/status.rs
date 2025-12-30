use colored::*;
use crate::config::get_install_dir;
use std::process::Command;

pub fn run_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=== Cyben Zen Tools 状态 ===".green());
    
    let install_dir = get_install_dir();
    println!("{} {}", "安装目录:".cyan(), install_dir.display());
    
    println!("{} {}", "版本:".cyan(), "1.0.0");
    println!("{} {}/{}", "平台:".cyan(), std::env::consts::OS, std::env::consts::ARCH);
    
    // 检查 Git 是否可用
    if Command::new("git").arg("--version").output().is_ok() {
        println!("{} {}", "✓".green(), "Git 可用".green());
    } else {
        println!("{} {}", "✗".red(), "Git 不可用".red());
    }
    
    // 检查 bash 是否可用
    if Command::new("bash").arg("--version").output().is_ok() {
        println!("{} {}", "✓".green(), "Bash 可用".green());
    } else {
        println!("{} {}", "✗".red(), "Bash 不可用".red());
    }
    
    Ok(())
}

