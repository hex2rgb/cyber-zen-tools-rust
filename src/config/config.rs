use std::path::{Path, PathBuf};

pub fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    // 配置初始化逻辑（如果需要）
    Ok(())
}

pub fn get_install_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cyben-zen-tools")
    } else {
        PathBuf::from("/tmp/.cyben-zen-tools")
    }
}

pub fn get_config_dir() -> PathBuf {
    // 优先使用当前目录的配置
    let current_dir = Path::new("configs");
    if current_dir.exists() {
        return current_dir.to_path_buf();
    }
    
    // 使用可执行文件同目录的配置
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let config_path = exe_dir.join("configs");
            if config_path.exists() {
                return config_path;
            }
        }
    }
    
    // 使用用户主目录的配置
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cyber-zen").join("configs")
    } else {
        PathBuf::from("configs")
    }
}

