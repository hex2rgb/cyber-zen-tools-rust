use std::path::PathBuf;

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
    // 直接使用用户主目录的配置
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cyber-zen").join("configs")
    } else {
        PathBuf::from("/tmp/.cyber-zen/configs")
    }
}

pub fn get_model_dir() -> PathBuf {
    // 模型文件目录
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cyber-zen").join("models")
    } else {
        PathBuf::from("/tmp/.cyber-zen/models")
    }
}

/// 模型文件夹名称（固定值）
/// 对应模型：Qwen2.5-0.5B-Instruct (GGUF 量化版本 Q4_K_M)
/// 注意：使用 Instruct 模型，更适合总结/归纳任务（commit message 生成）
/// 文件夹名称应该与下载的模型文件夹名称一致
pub const MODEL_FOLDER_NAME: &str = "qwen2.5-0.5b-instruct";

/// 获取模型文件夹路径
pub fn get_model_folder_path() -> PathBuf {
    get_model_dir().join(MODEL_FOLDER_NAME)
}

