//! Candle 模型加载和推理模块
//! **支持 Qwen3 模型架构**
//! 当前配置：Qwen3-1.7B-Instruct
//! 参考: https://github.com/huggingface/candle/tree/main/candle-examples/examples/qwen

use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen3::{Config as Qwen3Config, ModelForCausalLM as Qwen3Model};
use candle_transformers::utils;
use candle_transformers::generation::LogitsProcessor;
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::fs;
use tokenizers::Tokenizer;
use serde_json::Value;

/// Candle 模型包装器（仅支持 Qwen3）
pub struct CandleModel {
    device: Device,
    tokenizer: Tokenizer,
    model: Qwen3Model,
    config: Qwen3Config,
    model_path: PathBuf,
}

impl CandleModel {
    /// 从本地文件加载模型（仅支持 Qwen3）
    /// model_path 应该是模型目录中的 model.safetensors 文件路径（或分片文件）
    pub fn load_from_path(model_path: &PathBuf) -> Result<Self> {
        let device = Device::Cpu;
        
        // 获取模型目录
        let model_dir = model_path.parent()
            .context("无法获取模型目录")?
            .to_path_buf();
        
        println!("  模型目录: {}", model_dir.display());
        
        // 1. 加载 config.json
        let config_path = model_dir.join("config.json");
        if !config_path.exists() {
            anyhow::bail!("未找到 config.json: {}", config_path.display());
        }
        
        let config_content = fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取 config.json: {}", config_path.display()))?;
        
        let config: Qwen3Config = serde_json::from_str(&config_content)
            .context("无法解析 config.json，请确保是有效的 Qwen3 模型配置")?;
        
        println!("  ✓ config.json 加载成功");
        
        // 2. tokenizer.json
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            anyhow::bail!("未找到 tokenizer.json: {}", tokenizer_path.display());
        }
        
        let tokenizer = Tokenizer::from_file(tokenizer_path.clone())
            .map_err(|e| anyhow::anyhow!("无法加载 tokenizer.json ({}): {}", tokenizer_path.display(), e))?;
        
        println!("  ✓ tokenizer.json 加载成功");
        
        // 3. 查找 safetensors 文件（支持分片）
        let safetensors_files = Self::find_safetensors_files(&model_dir)?;
        println!("  ✓ 找到 {} 个 safetensors 文件", safetensors_files.len());
        
        // 4. 加载模型权重
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&safetensors_files, DType::F32, &device)
                .with_context(|| format!("无法加载模型权重文件"))?
        };
        
        // 5. 创建 Qwen3 模型实例
        let model = Qwen3Model::new(&config, vb)
            .context("无法创建 Qwen3 模型实例，请检查配置是否正确")?;
        
        println!("✓ Qwen3 模型加载成功");
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
    
    /// 查找 safetensors 文件（支持分片）
    fn find_safetensors_files(model_dir: &PathBuf) -> Result<Vec<PathBuf>> {
        // 优先检查单文件模型：检查 model.safetensors
        let single_file = model_dir.join("model.safetensors");
        if single_file.exists() {
            return Ok(vec![single_file]);
        }
        
        // 如果没有单文件，检查是否有 model.safetensors.index.json（分片模型）
        let index_path = model_dir.join("model.safetensors.index.json");
        if index_path.exists() {
            // 分片模型：读取 index 文件，找到所有分片
            let index_content = fs::read_to_string(&index_path)?;
            let index: Value = serde_json::from_str(&index_content)?;
            
            if let Some(weight_map) = index.get("weight_map").and_then(|v| v.as_object()) {
                let files: std::collections::HashSet<String> = weight_map.values()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                let file_count = files.len();
                let mut safetensors_files = Vec::new();
                for file_name in &files {
                    let file_path = model_dir.join(file_name);
                    if file_path.exists() {
                        safetensors_files.push(file_path);
                    } else {
                        // 如果分片文件不存在，记录警告但继续
                        eprintln!("警告: index.json 中引用的分片文件不存在: {}", file_path.display());
                    }
                }
                
                // 如果所有分片文件都存在，使用分片模式
                if !safetensors_files.is_empty() && safetensors_files.len() == file_count {
                    // 按文件名排序（确保顺序正确）
                    safetensors_files.sort();
                    return Ok(safetensors_files);
                }
                // 否则继续尝试其他方式
            }
        }
        
        // 如果都不存在，尝试查找 model-*.safetensors 模式
        let mut files = Vec::new();
        for entry in fs::read_dir(model_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("model-") && name.ends_with(".safetensors") {
                    files.push(path);
                }
            }
        }
        
        if !files.is_empty() {
            files.sort();
            return Ok(files);
        }
        
        anyhow::bail!("未找到 model.safetensors 文件或分片文件")
    }
    
    /// 生成文本（完全按照 Candle 官方 Qwen 示例实现）
    /// 参考: https://github.com/huggingface/candle/blob/main/candle-examples/examples/qwen/main.rs
    /// 支持模型：Qwen3-1.7B-Instruct 及其他 Qwen3 系列模型
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        // Qwen3-Instruct 使用特定的指令格式
        let formatted_prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that generates commit messages based on code changes. Follow Conventional Commits format.<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            prompt
        );
        
        // Tokenize 输入 - 直接使用官方示例的方式
        let mut tokens = self.tokenizer
            .encode(formatted_prompt.as_str(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization 失败: {}", e))?
            .get_ids()
            .to_vec();
        
        println!("  初始序列长度: {}", tokens.len());
        println!("  Prompt 前50字符: '{}'", formatted_prompt.chars().take(50).collect::<String>());
        
        // 获取结束 token - 官方示例的方式
        let eos_token = 151643u32; // <|im_end|>
        let eos_token2 = 151645u32; // <|endoftext|>
        
        let mut generated_tokens = Vec::new();
        println!("  开始生成文本（最大 {} tokens）...", max_tokens);
        
        // 创建 LogitsProcessor（在循环外创建，循环内重复使用）
        // 温度 0.8 可以在确定性和多样性之间取得平衡
        let mut logits_processor = LogitsProcessor::new(299792458, Some(0.8), None);
        
        // 完全按照官方示例的生成循环
        for index in 0..max_tokens {
            // 官方示例的关键逻辑：
            // - 第一次 (index == 0): context_size = tokens.len(), start_pos = 0
            // - 后续: context_size = 1, start_pos = tokens.len() - 1
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            
            // 创建输入 tensor
            let input = Tensor::new(ctxt, &self.device)?
                .unsqueeze(0)?;
            
            // 模型前向传播（Qwen3 的 forward 只需要两个参数）
            let logits = self.model.forward(&input, start_pos)?;
            
            // 官方示例的 logits 处理方式：
            // forward 返回的 logits 形状可能是 [1, seq_len, vocab_size] 或 [1, vocab_size]
            // 需要正确处理形状
            let logits_shape = logits.shape();
            let mut logits = logits.squeeze(0)?; // 移除 batch 维度
            
            // 如果还有序列维度，取最后一个位置的 logits
            if logits_shape.rank() == 3 {
                // [1, seq_len, vocab_size] -> [seq_len, vocab_size]
                let seq_len = logits.dim(0)?;
                logits = logits.narrow(0, seq_len - 1, 1)?.squeeze(0)?; // 取最后一个位置 -> [vocab_size]
            }
            // 如果已经是 [vocab_size]，直接使用
            
            let mut logits = logits.to_dtype(DType::F32)?;
            
            // 应用 repeat penalty（避免重复生成）- 官方示例的方式
            let repeat_penalty = 1.1f32;
            let repeat_last_n = 64;
            if repeat_penalty != 1.0 {
                let start_at = tokens.len().saturating_sub(repeat_last_n);
                logits = utils::apply_repeat_penalty(
                    &logits,
                    repeat_penalty,
                    &tokens[start_at..],
                )?;
            }
            
            // 使用 LogitsProcessor 进行温度采样（增加多样性，避免总是选择同一个 token）
            let next_token = logits_processor.sample(&logits)?;
            
            // 检查结束条件
            if next_token == eos_token || next_token == eos_token2 {
                println!("  遇到结束 token，停止生成");
                break;
            }
            
            // 添加到 tokens 和 generated_tokens
            tokens.push(next_token);
            generated_tokens.push(next_token);
            
            // 检测重复模式（如果最近 10 个 token 重复出现，提前停止）
            if generated_tokens.len() >= 20 {
                let recent = &generated_tokens[generated_tokens.len() - 10..];
                let prev = &generated_tokens[generated_tokens.len() - 20..generated_tokens.len() - 10];
                if recent == prev {
                    println!("  检测到重复模式，提前停止生成");
                    break;
                }
            }
            
            // 打印进度（包含调试信息）
            if index % 5 == 0 || index == 0 {
                let partial_text = self.tokenizer.decode(&generated_tokens, true)
                    .unwrap_or_else(|_| "...".to_string());
                // 显示原始 token ID 和文本内容
                let last_5_tokens: Vec<u32> = generated_tokens.iter().rev().take(5).copied().collect();
                println!("  已生成 {} tokens (最后5个token ID: {:?}): '{}'", 
                    generated_tokens.len(),
                    last_5_tokens,
                    partial_text.chars().take(100).collect::<String>()
                );
            }
        }
        
        println!("  ✓ 生成完成，共 {} tokens", generated_tokens.len());
        
        if generated_tokens.is_empty() {
            anyhow::bail!("生成的 tokens 为空，模型可能没有正确生成内容");
        }
        
        // 解码生成的 tokens
        let output_text = self.tokenizer.decode(&generated_tokens, true)
            .map_err(|e| anyhow::anyhow!("Token 解码失败: {}", e))?;
        
        println!("  原始解码文本 (长度 {}): '{}'", 
            output_text.len(),
            output_text.chars().take(200).collect::<String>()
        );
        
        // 清理输出（保留更多内容）
        let mut cleaned_output = output_text
            .trim()
            .replace("<|im_end|>", "")
            .replace("<|im_start|>", "")
            .trim()
            .to_string();
        
        // 如果清理后为空，返回原始文本（可能包含特殊字符）
        if cleaned_output.is_empty() || cleaned_output.trim().is_empty() {
            println!("  警告: 清理后输出为空，使用原始文本");
            cleaned_output = output_text.trim().to_string();
        }
        
        // 如果还是为空，尝试只移除特定标记
        if cleaned_output.is_empty() {
            cleaned_output = output_text
                .replace("<|im_end|>", "")
                .replace("<|im_start|>", "")
                .to_string();
        }
        
        Ok(cleaned_output)
    }
}


