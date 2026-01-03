use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::config::get_config_dir;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileTypeItem {
    pub extensions: Vec<String>,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileTypeConfig {
    pub file_types: HashMap<String, HashMap<String, FileTypeItem>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryPattern {
    pub patterns: Vec<String>,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryConfigInner {
    #[serde(rename = "directory_patterns")]
    pub directory_patterns: HashMap<String, CategoryPattern>,
    pub default: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryConfigWrapper {
    pub categories: CategoryConfigInner,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryConfig {
    pub directory_patterns: HashMap<String, CategoryPattern>,
    pub default: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommitTemplateConfig {
    pub prefixes: HashMap<String, String>,
    pub descriptions: HashMap<String, String>,
    pub actions: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommitTemplateConfigWrapper {
    #[serde(rename = "commit_templates")]
    pub commit_templates: CommitTemplateConfig,
}

pub struct FileTypeManager {
    file_types: FileTypeConfig,
    categories: CategoryConfig,
    commit_templates: CommitTemplateConfig,
}

impl FileTypeManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = get_config_dir();
        
        let file_types = load_file_type_config(&config_dir)?;
        let categories = load_category_config(&config_dir)?;
        let commit_templates = load_commit_template_config(&config_dir)?;
        
        Ok(FileTypeManager {
            file_types,
            categories,
            commit_templates,
        })
    }
    
    pub fn get_file_type(&self, filename: &str) -> String {
        let filename_lower = filename.to_lowercase();
        
        // 首先检查完整文件名匹配（用于配置文件等）
        for category in self.file_types.file_types.values() {
            for type_item in category.values() {
                for ext in &type_item.extensions {
                    // 检查完整文件名匹配（用于 package.json, Dockerfile 等）
                    if filename == ext || filename_lower == ext.to_lowercase() {
                        return type_item.description.clone();
                    }
                    // 检查扩展名匹配
                    if filename_lower.ends_with(&ext.to_lowercase()) {
                        return type_item.description.clone();
                    }
                }
            }
        }
        
        // 如果没有匹配，尝试从文件名推断
        let filename_lower = filename_lower.as_str();
        if filename_lower.contains("test") || filename_lower.contains("spec") {
            return "测试文件".to_string();
        }
        if filename_lower.contains("readme") || filename_lower.contains("changelog") {
            return "文档文件".to_string();
        }
        if filename_lower.contains("config") || filename_lower.contains("setting") {
            return "配置文件".to_string();
        }
        
        "其他文件".to_string()
    }
    
    pub fn get_file_category(&self, filepath: &str) -> String {
        // 标准化路径（统一使用 / 作为分隔符）
        let normalized_path = filepath.replace('\\', "/").to_lowercase();
        
        // 按优先级匹配（更具体的模式优先）
        // 首先检查完整目录名匹配（更精确）
        for pattern in self.categories.directory_patterns.values() {
            for pat in &pattern.patterns {
                let pat_lower = pat.to_lowercase();
                // 完整目录名匹配（包含路径分隔符）
                if normalized_path.contains(&format!("/{}/", pat_lower)) || 
                   normalized_path.starts_with(&format!("{}/", pat_lower)) ||
                   normalized_path.ends_with(&format!("/{}", pat_lower)) ||
                   normalized_path == pat_lower {
                    return pattern.description.clone();
                }
            }
        }
        
        // 如果完整匹配失败，使用部分匹配作为备选
        for pattern in self.categories.directory_patterns.values() {
            for pat in &pattern.patterns {
                let pat_lower = pat.to_lowercase();
                if normalized_path.contains(&pat_lower) {
                    return pattern.description.clone();
                }
            }
        }
        
        self.categories.default.clone()
    }
    
    pub fn get_commit_type(&self, added: i32, modified: i32, deleted: i32) -> String {
        let total = added + modified + deleted;
        
        if total == 0 {
            return "chore".to_string();
        }
        
        // 单一操作类型的情况
        if added > 0 && modified == 0 && deleted == 0 {
            "feat".to_string()
        } else if modified > 0 && added == 0 && deleted == 0 {
            "fix".to_string()
        } else if deleted > 0 && added == 0 && modified == 0 {
            "cleanup".to_string()
        } 
        // 混合操作类型的情况
        else if added > 0 && modified > 0 && deleted == 0 {
            // 新增 + 修改 = 重构或新功能
            if added > modified {
                "feat".to_string()  // 新增更多，视为新功能
            } else {
                "refactor".to_string()  // 修改更多，视为重构
            }
        } else if modified > 0 && deleted > 0 {
            "refactor".to_string()  // 修改 + 删除 = 重构
        } else if added > 0 && deleted > 0 {
            "refactor".to_string()  // 新增 + 删除 = 重构（替换）
        } else {
            // 其他复杂情况，使用 refactor
            "refactor".to_string()
        }
    }
    
    pub fn get_commit_description(&self, commit_type: &str) -> String {
        self.commit_templates.descriptions
            .get(commit_type)
            .cloned()
            .unwrap_or_else(|| "更新项目".to_string())
    }
    
    pub fn get_action_description(&self, action: &str) -> String {
        self.commit_templates.actions
            .get(action)
            .cloned()
            .unwrap_or_else(|| action.to_string())
    }
}

fn load_file_type_config(config_dir: &Path) -> Result<FileTypeConfig, Box<dyn std::error::Error>> {
    let config_path = config_dir.join("file-types.toml");
    let content = fs::read_to_string(&config_path)?;
    let config: FileTypeConfig = toml::from_str(&content)?;
    Ok(config)
}

fn load_category_config(config_dir: &Path) -> Result<CategoryConfig, Box<dyn std::error::Error>> {
    let config_path = config_dir.join("categories.toml");
    let content = fs::read_to_string(&config_path)?;
    let wrapper: CategoryConfigWrapper = toml::from_str(&content)?;
    Ok(CategoryConfig {
        directory_patterns: wrapper.categories.directory_patterns,
        default: wrapper.categories.default,
    })
}

fn load_commit_template_config(config_dir: &Path) -> Result<CommitTemplateConfig, Box<dyn std::error::Error>> {
    let config_path = config_dir.join("commit-templates.toml");
    let content = fs::read_to_string(&config_path)?;
    let wrapper: CommitTemplateConfigWrapper = toml::from_str(&content)?;
    Ok(wrapper.commit_templates)
}

