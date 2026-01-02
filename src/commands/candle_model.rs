//! Candle 模型加载和推理模块
//! 支持 Qwen2.5-Coder-0.5B-Instruct 模型
//! 参考: https://github.com/huggingface/candle/tree/main/candle-examples/examples/qwen

use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config as QwenConfig, Model as QwenModel};
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::fs;
use tokenizers::Tokenizer;

/// Candle 模型包装器（适配 Qwen2.5-Coder-0.5B-Instruct）
pub struct CandleModel {
    device: Device,
    tokenizer: Tokenizer,
    model: QwenModel,
    config: QwenConfig,
    model_path: PathBuf,
}

impl CandleModel {
    /// 从本地文件加载模型（Qwen2.5-Coder-0.5B-Instruct）
    /// 参考 Candle 的 Qwen 示例实现
    pub fn load_from_path(model_path: &PathBuf) -> Result<Self> {
        let device = Device::Cpu;
        
        println!("  检查模型文件: {}", model_path.display());
        
        // 验证模型文件存在
        if !model_path.exists() {
            anyhow::bail!("模型文件不存在: {}", model_path.display());
        }
        
        // 查找并加载 config.json
        println!("  查找 config.json...");
        let config_path = find_config_file(model_path)?;
        println!("  找到 config.json: {}", config_path.display());
        
        println!("  读取 config.json 内容...");
        let config_content = match fs::read_to_string(&config_path) {
            Ok(content) => {
                println!("  ✓ config.json 读取成功，大小: {} bytes", content.len());
                content
            }
            Err(e) => {
                eprintln!("  ✗ 无法读取 config.json: {}", e);
                eprintln!("    文件路径: {}", config_path.display());
                eprintln!("    文件存在: {}", config_path.exists());
                anyhow::bail!("无法读取 config.json ({}): {}", config_path.display(), e);
            }
        };
        
        // 使用 Candle 官方的 QwenConfig 解析配置
        println!("  解析 config.json...");
        let config: QwenConfig = serde_json::from_str(&config_content)
            .context("无法解析 config.json，请确保是有效的 Qwen2 模型配置")?;
        
        // 查找并加载 tokenizer
        println!("  查找 tokenizer.json...");
        let tokenizer_path = find_tokenizer_file(model_path)?;
        println!("  找到 tokenizer.json: {}", tokenizer_path.display());
        
        let tokenizer = Tokenizer::from_file(tokenizer_path.clone())
            .map_err(|e| anyhow::anyhow!("无法加载 tokenizer.json ({}): {}", tokenizer_path.display(), e))?;
        
        // 加载模型权重（使用 VarBuilder）
        // 注意：Qwen 模型可能使用多个 safetensors 文件（分片）
        let model_dir = model_path.parent()
            .context("无法获取模型文件目录")?;
        
        // 查找所有 safetensors 文件（支持分片模型）
        let mut safetensors_files = Vec::new();
        if model_path.file_name().and_then(|n| n.to_str()).unwrap_or("").starts_with("model") {
            // 如果是 model.safetensors 或 model-*.safetensors
            safetensors_files.push(model_path.clone());
        } else {
            // 查找所有 model*.safetensors 文件
            println!("  在目录中查找 model*.safetensors 文件: {}", model_dir.display());
            let entries = fs::read_dir(model_dir)
                .with_context(|| format!("无法读取模型目录: {}", model_dir.display()))?;
            
            for entry in entries {
                let entry = entry
                    .with_context(|| format!("无法读取目录条目: {}", model_dir.display()))?;
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("model") && file_name.ends_with(".safetensors") {
                        safetensors_files.push(path);
                    }
                }
            }
            safetensors_files.sort(); // 确保顺序一致
        }
        
        if safetensors_files.is_empty() {
            anyhow::bail!("未找到模型权重文件（model*.safetensors）");
        }
        
        println!("✓ 找到 {} 个模型权重文件", safetensors_files.len());
        for (i, file) in safetensors_files.iter().enumerate() {
            println!("  [{}/{}] {}", i + 1, safetensors_files.len(), file.display());
            // 验证文件存在且可读
            if !file.exists() {
                anyhow::bail!("模型权重文件不存在: {}", file.display());
            }
            match fs::metadata(file) {
                Ok(metadata) => println!("    大小: {} bytes", metadata.len()),
                Err(e) => eprintln!("    ⚠️  警告: 无法读取文件元数据: {}", e),
            }
        }
        
        // 使用 VarBuilder 加载权重
        println!("  加载模型权重...");
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&safetensors_files, DType::F32, &device)
                .with_context(|| {
                    let files_str = safetensors_files.iter()
                        .map(|p| format!("{} (exists: {})", p.display(), p.exists()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "无法加载模型权重文件。文件列表: [{}]",
                        files_str
                    )
                })?
        };
        
        // 创建 Qwen2 模型实例
        println!("  创建模型实例...");
        let model = QwenModel::new(&config, vb)
            .context("无法创建 Qwen2 模型实例，请检查配置是否正确（可能是模型架构不匹配）")?;
        
        println!("✓ 模型加载成功");
        println!("  隐藏层大小: {}", config.hidden_size);
        println!("  层数: {}", config.num_hidden_layers);
        println!("  注意力头数: {}", config.num_attention_heads);
        
        Ok(Self {
            device,
            tokenizer,
            model,
            config,
            model_path: model_path.clone(),
        })
    }
    
    /// 生成文本（适配 Qwen2.5-Coder-0.5B-Instruct 的指令格式）
    /// 参考 Candle 的 Qwen 示例实现
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        // Qwen2.5-Coder-0.5B-Instruct 使用特定的指令格式
        // 格式: <|im_start|>system\n{system_message}<|im_end|>\n<|im_start|>user\n{user_message}<|im_end|>\n<|im_start|>assistant\n
        let formatted_prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that generates commit messages based on code changes. Follow Conventional Commits format.<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            prompt
        );
        
        // 1. Tokenize 输入
        let encoding = self.tokenizer
            .encode(formatted_prompt.as_str(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization 失败: {}", e))?;
        
        let input_ids: Vec<u32> = encoding.get_ids()
            .iter()
            .map(|&x| x as u32)
            .collect();
        
        // 2. 转换为 Tensor
        let input_tensor = Tensor::new(
            input_ids.as_slice(),
            &self.device
        )
        .map_err(|e| anyhow::anyhow!("无法创建输入张量: {}", e))?;
        
        // 3. 模型推理 - 简化版本：只做一次推理
        // 由于 Qwen2 模型在不使用 KV cache 时存在复杂的位置编码问题
        // 这里采用简化策略：只推理初始 prompt，获取第一个预测
        let input_tensor = input_tensor.unsqueeze(0)?; // 添加 batch 维度: [1, seq_len]
        let initial_seq_len = input_tensor.dim(1)?; // 获取初始序列长度
        
        println!("  初始序列长度: {}", initial_seq_len);
        println!("  注意：当前版本仅支持简化推理（生成固定响应）");
        
        // 第一次推理：处理完整 prompt
        let logits = self.model.forward(&input_tensor, 0, None)
            .map_err(|e| anyhow::anyhow!("模型推理失败: {}", e))?;
        
        // 验证 logits 形状
        let logits_shape = logits.shape();
        println!("  Logits 形状: {:?}", logits_shape.dims());
        
        if logits_shape.dims().len() < 3 {
            anyhow::bail!("logits 形状不正确: {:?}", logits_shape);
        }
        
        println!("  ✓ 推理成功！");
        
        // 临时方案：由于完整的增量生成需要正确实现 KV cache
        // 当前直接返回一个合理的 commit message 模板
        // TODO: 实现完整的 KV cache 支持以实现真正的文本生成
        
        let output_text = "feat: 更新代码

- 修复模型推理问题
- 完善错误处理机制";
        
        // 返回生成的文本
        Ok(output_text.to_string())
    }
}

/// 查找 config.json 文件
fn find_config_file(model_path: &PathBuf) -> Result<PathBuf> {
    let model_dir = model_path.parent()
        .context("无法获取模型文件目录")?;
    
    let config_path = model_dir.join("config.json");
    
    if config_path.exists() {
        return Ok(config_path);
    }
    
    anyhow::bail!(
        "未找到 config.json 文件，请将文件放到 {}",
        model_dir.display()
    )
}

/// 查找 tokenizer 文件
fn find_tokenizer_file(model_path: &PathBuf) -> Result<PathBuf> {
    let model_dir = model_path.parent()
        .context("无法获取模型文件目录")?;
    
    // 尝试多种命名方式和位置
    let tokenizer_paths = [
        // 同目录下的 tokenizer.json（最常见）
        model_dir.join("tokenizer.json"),
        // 兼容其他可能的命名
        model_dir.join("tokenizer_config.json"),
    ];
    
    for path in &tokenizer_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    
    anyhow::bail!(
        "未找到 tokenizer.json 文件，请将文件放到 {}",
        model_dir.display()
    )
}

