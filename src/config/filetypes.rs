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
        for category in self.file_types.file_types.values() {
            for type_item in category.values() {
                for ext in &type_item.extensions {
                    if filename.ends_with(ext) {
                        return type_item.description.clone();
                    }
                }
            }
        }
        "其他文件".to_string()
    }
    
    pub fn get_file_category(&self, filepath: &str) -> String {
        for pattern in self.categories.directory_patterns.values() {
            for pat in &pattern.patterns {
                if filepath.contains(pat) {
                    return pattern.description.clone();
                }
            }
        }
        self.categories.default.clone()
    }
    
    pub fn get_commit_type(&self, added: i32, modified: i32, deleted: i32) -> String {
        if added > 0 && modified == 0 && deleted == 0 {
            "feat".to_string()
        } else if modified > 0 && added == 0 && deleted == 0 {
            "fix".to_string()
        } else if deleted > 0 && added == 0 && modified == 0 {
            "cleanup".to_string()
        } else if modified > 0 && added > 0 && deleted == 0 {
            "refactor".to_string()
        } else {
            "feat".to_string()
        }
    }
    
    #[allow(dead_code)]
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

    if !config_path.exists() {
        // 返回默认配置
        return Ok(FileTypeConfig {
            file_types: HashMap::new(),
        });
    }

    let content = fs::read_to_string(&config_path)?;
    let config: FileTypeConfig = toml::from_str(&content)?;
    Ok(config)
}

fn load_category_config(config_dir: &Path) -> Result<CategoryConfig, Box<dyn std::error::Error>> {
    let config_path = config_dir.join("categories.toml");

    if !config_path.exists() {
        // 返回默认配置
        return Ok(CategoryConfig {
            directory_patterns: HashMap::new(),
            default: "项目文件".to_string(),
        });
    }

    let content = fs::read_to_string(&config_path)?;
    let wrapper: CategoryConfigWrapper = toml::from_str(&content)?;
    Ok(CategoryConfig {
        directory_patterns: wrapper.categories.directory_patterns,
        default: wrapper.categories.default,
    })
}

fn load_commit_template_config(config_dir: &Path) -> Result<CommitTemplateConfig, Box<dyn std::error::Error>> {
    let config_path = config_dir.join("commit-templates.toml");

    if !config_path.exists() {
        // 返回默认配置
        let mut prefixes = HashMap::new();
        prefixes.insert("feat".to_string(), "feat".to_string());
        prefixes.insert("fix".to_string(), "fix".to_string());
        prefixes.insert("docs".to_string(), "docs".to_string());
        prefixes.insert("style".to_string(), "style".to_string());
        prefixes.insert("refactor".to_string(), "refactor".to_string());
        prefixes.insert("test".to_string(), "test".to_string());
        prefixes.insert("chore".to_string(), "chore".to_string());
        prefixes.insert("cleanup".to_string(), "cleanup".to_string());

        let mut descriptions = HashMap::new();
        descriptions.insert("feat".to_string(), "新功能".to_string());
        descriptions.insert("fix".to_string(), "修复".to_string());
        descriptions.insert("docs".to_string(), "文档".to_string());
        descriptions.insert("style".to_string(), "样式".to_string());
        descriptions.insert("refactor".to_string(), "重构".to_string());
        descriptions.insert("test".to_string(), "测试".to_string());
        descriptions.insert("chore".to_string(), "构建".to_string());
        descriptions.insert("cleanup".to_string(), "清理".to_string());

        let mut actions = HashMap::new();
        actions.insert("added".to_string(), "新增".to_string());
        actions.insert("modified".to_string(), "修改".to_string());
        actions.insert("deleted".to_string(), "删除".to_string());
        actions.insert("renamed".to_string(), "重命名".to_string());

        return Ok(CommitTemplateConfig {
            prefixes,
            descriptions,
            actions,
        });
    }

    let content = fs::read_to_string(&config_path)?;
    let wrapper: CommitTemplateConfigWrapper = toml::from_str(&content)?;
    Ok(wrapper.commit_templates)
}

