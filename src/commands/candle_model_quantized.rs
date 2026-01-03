//! Candle 量化模型加载和推理模块（GGUF 格式）
//! 支持 Qwen2.5/Qwen2 量化模型
//! 当前配置：Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf

use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file;
use candle_transformers::models::quantized_qwen2::ModelWeights as Qwen2Quantized;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::utils;
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::fs::File;
use tokenizers::Tokenizer;

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
        let device = Device::Cpu;
        
        // 获取模型目录
        let model_dir = model_path.parent()
            .context("无法获取模型目录")?
            .to_path_buf();
        
        println!("  模型目录: {}", model_dir.display());
        
        // 1. 加载 tokenizer（与模型在同一目录或父目录）
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            anyhow::bail!("未找到 tokenizer.json: {}", tokenizer_path.display());
        }
        
        let tokenizer = Tokenizer::from_file(tokenizer_path.clone())
            .map_err(|e| anyhow::anyhow!("无法加载 tokenizer.json: {}", e))?;
        
        println!("  ✓ tokenizer.json 加载成功");
        
        // 2. 加载 GGUF 模型
        println!("  正在加载 GGUF 量化模型...");
        let mut file = File::open(model_path)
            .with_context(|| format!("无法打开模型文件: {}", model_path.display()))?;
        
        let start = std::time::Instant::now();
        let model_content = gguf_file::Content::read(&mut file)
            .map_err(|e| anyhow::anyhow!("无法读取 GGUF 文件: {}", e))?;
        
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
        
        // 生成循环
        for index in 0..max_tokens {
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
                break;
            }
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

