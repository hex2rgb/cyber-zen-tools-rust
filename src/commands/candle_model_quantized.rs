//! Candle 量化模型加载和推理模块（GGUF 格式）
//! 支持 Qwen2.5/Qwen2 量化模型
//! 当前配置：Qwen2.5-0.5B-Instruct-Q4_K_M.gguf (Instruct 模型，适合总结任务)

use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file;
use candle_transformers::models::quantized_qwen2::ModelWeights as Qwen2Quantized;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::utils;
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use tokenizers::Tokenizer;
use num_cpus;

/// 量化模型包装器（GGUF 格式）
pub struct CandleModelQuantized {
    device: Device,
    tokenizer: Tokenizer,
    model: Qwen2Quantized,
    model_path: PathBuf,
}

impl CandleModelQuantized {
    /// 从本地文件加载量化模型（GGUF 格式）
    /// model_path 应该是 .gguf 文件路径
    pub fn load_from_path(model_path: &PathBuf) -> Result<Self> {
        // 确保使用多线程（必须！单线程会导致极慢）
        let cpu_count = num_cpus::get();
        if std::env::var("OMP_NUM_THREADS").is_err() {
            std::env::set_var("OMP_NUM_THREADS", &cpu_count.to_string());
        }
        if std::env::var("RAYON_NUM_THREADS").is_err() {
            std::env::set_var("RAYON_NUM_THREADS", &cpu_count.to_string());
        }
        println!("  使用 {} 个 CPU 核心进行推理", cpu_count);
        
        let device = Device::Cpu;
        
        // 获取模型目录
        let model_dir = model_path.parent()
            .context("无法获取模型目录")?
            .to_path_buf();
        
        println!("  模型目录: {}", model_dir.display());
        
        // 1. 加载 GGUF 模型（自包含 tokenizer 信息）
        println!("  正在加载 GGUF 量化模型...");
        let mut file = File::open(model_path)
            .with_context(|| format!("无法打开模型文件: {}", model_path.display()))?;
        
        let start = std::time::Instant::now();
        let model_content = gguf_file::Content::read(&mut file)
            .map_err(|e| anyhow::anyhow!("无法读取 GGUF 文件: {}", e))?;
        
        // 检查 metadata 中是否包含 tokenizer 信息
        println!("  GGUF metadata keys: {:?}", model_content.metadata.keys().take(10).collect::<Vec<_>>());
        
        // 从 GGUF metadata 中提取 tokenizer 信息并构建 tokenizer
        // GGUF 包含 tokenizer 的词汇表和配置信息
        let tokenizer = build_tokenizer_from_gguf_metadata(&model_content)
            .context("无法从 GGUF metadata 构建 tokenizer")?;
        
        println!("  ✓ 从 GGUF 文件加载 tokenizer 成功");
        
        // 计算模型大小
        let mut total_size_in_bytes = 0;
        for (_, tensor) in model_content.tensor_infos.iter() {
            let elem_count = tensor.shape.elem_count();
            total_size_in_bytes +=
                elem_count * tensor.ggml_dtype.type_size() / tensor.ggml_dtype.block_size();
        }
        
        println!(
            "  加载了 {} 个张量 ({:.2} GB) 用时 {:.2}s",
            model_content.tensor_infos.len(),
            total_size_in_bytes as f64 / 1e9,
            start.elapsed().as_secs_f32(),
        );
        
        let model = Qwen2Quantized::from_gguf(model_content, &mut file, &device)?;
        
        println!("✓ GGUF 量化模型加载成功");
        
        Ok(Self {
            device,
            tokenizer,
            model,
            model_path: model_path.clone(),
        })
    }
    
    /// 生成文本
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        // Qwen2.5-Coder-Instruct 使用特定的指令格式
        let formatted_prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that generates commit messages based on code changes. Follow Conventional Commits format.<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            prompt
        );
        
        // Tokenize 输入
        let tokens = self.tokenizer
            .encode(formatted_prompt.as_str(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization 失败: {}", e))?
            .get_ids()
            .to_vec();
        
        println!("  初始序列长度: {}", tokens.len());
        
        // 创建采样器
        let mut logits_processor = LogitsProcessor::from_sampling(
            299792458,  // seed
            Sampling::All { temperature: 0.8 }
        );
        
        // 第一次前向传播（处理整个 prompt）
        let prompt_len = tokens.len();
        let input = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;
        let logits = self.model.forward(&input, 0)?;
        let logits = logits.squeeze(0)?;
        let mut next_token = logits_processor.sample(&logits)?;
        
        let mut all_tokens = tokens;
        all_tokens.push(next_token);
        
        // 获取结束 token
        let eos_token = *self.tokenizer.get_vocab(true)
            .get("<|im_end|>")
            .ok_or_else(|| anyhow::anyhow!("未找到 <|im_end|> token"))?;
        
        let mut generated_tokens = Vec::new();
        generated_tokens.push(next_token);
        
        // 生成循环（添加进度提示）
        for index in 0..max_tokens {
            // 每10个token打印一次进度
            if index > 0 && index % 10 == 0 {
                print!(".");
                std::io::stdout().flush().ok();
            }
            
            let input = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, prompt_len + index)?;
            let logits = logits.squeeze(0)?;
            
            // 应用 repeat penalty
            let logits = if generated_tokens.len() >= 64 {
                let start_at = generated_tokens.len().saturating_sub(64);
                utils::apply_repeat_penalty(
                    &logits,
                    1.1,
                    &generated_tokens[start_at..],
                )?
            } else {
                logits
            };
            
            next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);
            generated_tokens.push(next_token);
            
            if next_token == eos_token {
                println!(" [完成]");
                break;
            }
        }
        if generated_tokens.len() >= max_tokens {
            println!(" [达到最大长度]");
        }
        
        // 解码生成的 tokens
        let output_text = self.tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| anyhow::anyhow!("Token 解码失败: {}", e))?;
        
        // 清理输出
        let cleaned_output = output_text
            .trim()
            .replace("<|im_end|>", "")
            .replace("<|im_start|>", "")
            .trim()
            .to_string();
        
        Ok(cleaned_output)
    }
}

/// 从 GGUF metadata 构建 tokenizer
/// GGUF 文件包含 tokenizer 的词汇表和配置信息
fn build_tokenizer_from_gguf_metadata(content: &gguf_file::Content) -> Result<Tokenizer> {
    use gguf_file::Value;
    
    println!("  从 GGUF metadata 构建 tokenizer...");
    
    // 提取 tokens（词汇表）
    let tokens = if let Some(Value::Array(token_array)) = content.metadata.get("tokenizer.ggml.tokens") {
        let mut vocab = Vec::new();
        for token_val in token_array {
            if let Value::String(token) = token_val {
                vocab.push(token.clone());
            }
            // 注意：如果 token 不是 String，可能需要其他处理方式
        }
        vocab
    } else {
        anyhow::bail!("未找到 tokenizer.ggml.tokens");
    };
    
    println!("  提取了 {} 个 tokens", tokens.len());
    
    // 提取 token 类型（通常是 u32 数组）
    let _token_types = if let Some(Value::Array(type_array)) = content.metadata.get("tokenizer.ggml.token_type") {
        let mut types = Vec::new();
        for type_val in type_array {
            if let Value::U32(t) = type_val {
                types.push(*t);
            } else if let Value::U64(t) = type_val {
                types.push(*t as u32);
            }
        }
        Some(types)
    } else {
        None
    };
    
    // 提取特殊 token IDs
    let eos_token_id = content.metadata.get("tokenizer.ggml.eos_token_id")
        .and_then(|v| match v {
            Value::U32(id) => Some(*id as u64),
            Value::U64(id) => Some(*id),
            _ => None,
        })
        .unwrap_or(151643); // Qwen 默认 EOS token ID
    
    let bos_token_id = content.metadata.get("tokenizer.ggml.bos_token_id")
        .and_then(|v| match v {
            Value::U32(id) => Some(*id as u64),
            Value::U64(id) => Some(*id),
            _ => None,
        })
        .unwrap_or(151643); // 通常与 EOS 相同
    
    let pad_token_id = content.metadata.get("tokenizer.ggml.padding_token_id")
        .and_then(|v| match v {
            Value::U32(id) => Some(*id as u64),
            Value::U64(id) => Some(*id),
            _ => None,
        })
        .unwrap_or(151643); // 通常与 EOS 相同
    
    println!("  EOS: {}, BOS: {}, PAD: {}", eos_token_id, bos_token_id, pad_token_id);
    
    // 构建词汇表 HashMap
    let mut vocab_map = std::collections::HashMap::new();
    for (idx, token) in tokens.iter().enumerate() {
        vocab_map.insert(token.clone(), idx as u32);
    }
    
    // 构建 tokenizers::Tokenizer 配置
    // 注意：tokenizers crate 需要完整的配置，但我们可以创建一个简化版本
    // 对于 Qwen，通常是 BPE tokenizer
    
    // 创建一个基本的 tokenizer 配置 JSON
    let tokenizer_config = serde_json::json!({
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [
            {"id": bos_token_id, "content": "<|im_start|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true},
            {"id": eos_token_id, "content": "<|im_end|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true}
        ],
        "normalizer": null,
        "pre_tokenizer": null,
        "post_processor": null,
        "decoder": null,
        "model": {
            "type": "BPE",
            "dropout": null,
            "unk_token": null,
            "continuing_subword_prefix": null,
            "end_of_word_suffix": null,
            "fuse_unk": false,
            "vocab": vocab_map,
            "merges": []
        }
    });
    
    // 从配置构建 tokenizer
    // 注意：Tokenizer::from_str 不存在，需要使用其他方法
    // 我们可以创建一个临时文件或者使用内存中的 JSON 数据
    
    // 方案：将配置写入临时 JSON 字符串，然后使用 Tokenizer::from_file
    // 但 from_file 需要文件路径，所以我们使用另一种方法
    
    // 实际上，tokenizers crate 可能不支持直接从内存中的 JSON 构建
    // 我们需要将 JSON 写入临时文件
    use std::io::Write;
    let temp_file = std::env::temp_dir().join(format!("tokenizer_{}.json", std::process::id()));
    std::fs::write(&temp_file, tokenizer_config.to_string())
        .context("无法写入临时 tokenizer 配置文件")?;
    
    let tokenizer = Tokenizer::from_file(&temp_file)
        .map_err(|e| anyhow::anyhow!("无法从配置构建 tokenizer: {}", e))?;
    
    // 清理临时文件（可选）
    let _ = std::fs::remove_file(&temp_file);
    
    println!("  ✓ Tokenizer 构建成功");
    Ok(tokenizer)
}

