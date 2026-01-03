# GGUF 量化模型使用指南

## 什么是 GGUF 模型？

GGUF (GPT-Generated Unified Format) 是一种量化模型格式，可以大幅减少模型大小和内存占用。

### 优势

- ✅ **模型大小减少 50-75%**：3.2GB → 1.0-1.5GB（Q4_K_M）
- ✅ **内存占用减少 60-70%**：6-8GB → 2-4GB
- ✅ **加载速度更快**：文件更小，加载更快
- ✅ **质量损失小**：Q8_0 几乎无损，Q4_K_M 略有损失但可接受

### 量化级别

| 量化级别 | 模型大小 | 内存占用 | 质量 | 速度 |
|---------|---------|---------|------|------|
| **Q4_K_M**（推荐） | 1.0-1.5GB | 2-3GB | 良好 | 快 |
| **Q8_0** | 1.8-2.0GB | 3-4GB | 优秀（几乎无损） | 中等 |
| **Q5_K_M** | 1.3-1.7GB | 2.5-3.5GB | 很好 | 快 |

---

## 下载 GGUF 模型

### 方法 1：使用 huggingface-cli（推荐）

```bash
# 创建目录
mkdir -p ~/.cyber-zen/models/qwen3-1.7b-gguf

# 下载 Q4_K_M 量化模型（推荐，平衡质量和大小）
huggingface-cli download unsloth/Qwen3-1.7B-GGUF \
  Qwen3-1.7B-Q4_K_M.gguf \
  --local-dir ~/.cyber-zen/models/qwen3-1.7b-gguf \
  --local-dir-use-symlinks False

# 下载 tokenizer（同样需要）
huggingface-cli download Qwen/Qwen3-1.7B-Instruct \
  tokenizer.json \
  --local-dir ~/.cyber-zen/models/qwen3-1.7b-gguf \
  --local-dir-use-symlinks False
```

### 方法 2：使用 Python 脚本

```python
from huggingface_hub import hf_hub_download
import os

# 创建目录
os.makedirs("~/.cyber-zen/models/qwen3-1.7b-gguf", exist_ok=True)

# 下载模型
model_path = hf_hub_download(
    repo_id="unsloth/Qwen3-1.7B-GGUF",
    filename="Qwen3-1.7B-Q4_K_M.gguf",
    local_dir="~/.cyber-zen/models/qwen3-1.7b-gguf"
)

# 下载 tokenizer
tokenizer_path = hf_hub_download(
    repo_id="Qwen/Qwen3-1.7B-Instruct",
    filename="tokenizer.json",
    local_dir="~/.cyber-zen/models/qwen3-1.7b-gguf"
)
```

---

## 代码修改方案

### 方案 1：创建新的量化模型加载模块（推荐）

创建新文件 `src/commands/candle_model_quantized.rs`：

```rust
//! Candle 量化模型加载和推理模块（GGUF 格式）
//! 支持 Qwen3 量化模型

use candle_core::{Device, Tensor, DType};
use candle::quantized::gguf_file;
use candle_transformers::models::quantized_qwen3::ModelWeights as Qwen3Quantized;
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
    model: Qwen3Quantized,
    model_path: PathBuf,
}

impl CandleModelQuantized {
    /// 从本地文件加载量化模型（GGUF 格式）
    pub fn load_from_path(model_path: &PathBuf) -> Result<Self> {
        let device = Device::Cpu;
        
        // 获取模型目录
        let model_dir = model_path.parent()
            .context("无法获取模型目录")?
            .to_path_buf();
        
        println!("  模型目录: {}", model_dir.display());
        
        // 1. 加载 tokenizer
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            anyhow::bail!("未找到 tokenizer.json: {}", tokenizer_path.display());
        }
        
        let tokenizer = Tokenizer::from_file(tokenizer_path.clone())
            .map_err(|e| anyhow::anyhow!("无法加载 tokenizer.json: {}", e))?;
        
        println!("  ✓ tokenizer.json 加载成功");
        
        // 2. 加载 GGUF 模型
        println!("  正在加载 GGUF 模型...");
        let mut file = File::open(model_path)
            .with_context(|| format!("无法打开模型文件: {}", model_path.display()))?;
        
        let model_content = gguf_file::Content::read(&mut file)
            .map_err(|e| anyhow::anyhow!("无法读取 GGUF 文件: {}", e))?;
        
        let model = Qwen3Quantized::from_gguf(model_content, &mut file, &device)?;
        
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
        // Qwen3-Instruct 使用特定的指令格式
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
        let input = Tensor::new(&tokens, &self.device)?.unsqueeze(0)?;
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
            let logits = self.model.forward(&input, tokens.len() + index)?;
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
```

### 方案 2：修改现有代码支持两种格式

在 `src/commands/gcm_ai.rs` 中：

```rust
// 检测模型格式并选择加载方式
fn load_model(model_path: &PathBuf) -> Result<Box<dyn ModelTrait>> {
    if model_path.extension().and_then(|s| s.to_str()) == Some("gguf") {
        // 加载量化模型
        let model = CandleModelQuantized::load_from_path(model_path)?;
        Ok(Box::new(model))
    } else {
        // 加载普通模型
        let model = CandleModel::load_from_path(model_path)?;
        Ok(Box::new(model))
    }
}
```

---

## 使用示例

### 基本使用

```rust
use crate::commands::candle_model_quantized::CandleModelQuantized;

// 加载量化模型
let model_path = PathBuf::from("~/.cyber-zen/models/qwen3-1.7b-gguf/Qwen3-1.7B-Q4_K_M.gguf");
let mut model = CandleModelQuantized::load_from_path(&model_path)?;

// 生成文本
let prompt = "根据代码变更生成 commit message...";
let output = model.generate(prompt, 200)?;
```

---

## 配置更新

### 更新模型路径配置

在 `src/config/config.rs` 中：

```rust
/// 量化模型文件夹名称（可选）
pub const MODEL_FOLDER_NAME_QUANTIZED: &str = "qwen3-1.7b-gguf";

/// 量化模型文件名
pub const MODEL_FILE_NAME_GGUF: &str = "Qwen3-1.7B-Q4_K_M.gguf";
```

---

## 性能对比

### FP32 模型（当前）

- 模型大小：3.2GB
- 内存占用：6-8GB
- 加载时间：~10-15 秒
- 推理速度：较慢

### Q4_K_M 量化模型

- 模型大小：1.0-1.5GB（减少 50-60%）
- 内存占用：2-3GB（减少 60-70%）
- 加载时间：~5-8 秒（更快）
- 推理速度：稍慢但可接受

### Q8_0 量化模型

- 模型大小：1.8-2.0GB（减少 40%）
- 内存占用：3-4GB（减少 50%）
- 加载时间：~7-10 秒
- 推理速度：接近 FP32
- 质量：几乎无损

---

## 推荐配置

### 对于 16GB 内存的系统

推荐使用 **Q4_K_M** 量化模型：
- 内存占用：2-3GB（安全）
- 质量：良好（对于 commit message 生成足够好）
- 速度：可接受

### 对于 8GB 内存的系统

推荐使用 **Qwen3-0.6B-Q4_K_M**：
- 模型大小：~0.5GB
- 内存占用：~1.5GB
- 速度：快

---

## 下载链接

### Qwen3-1.7B GGUF 模型

- **Q4_K_M**（推荐）：
  - https://huggingface.co/unsloth/Qwen3-1.7B-GGUF/tree/main
  - 文件：`Qwen3-1.7B-Q4_K_M.gguf`

- **Q8_0**（高质量）：
  - https://huggingface.co/unsloth/Qwen3-1.7B-GGUF/tree/main
  - 文件：`Qwen3-1.7B-Q8_0.gguf`

### Tokenizer

从原始模型下载：
- https://huggingface.co/Qwen/Qwen3-1.7B-Instruct
- 文件：`tokenizer.json`

---

## 故障排除

### 问题 1：找不到 quantized_qwen3 模块

**解决方案**：确保 Cargo.toml 中使用了正确版本的 candle-transformers：

```toml
candle-transformers = "0.9.2-alpha.2"
```

### 问题 2：GGUF 文件格式错误

**解决方案**：确保下载的是完整的 GGUF 文件，可以使用以下命令验证：

```bash
file ~/.cyber-zen/models/qwen3-1.7b-gguf/Qwen3-1.7B-Q4_K_M.gguf
# 应该显示：GGUF file format
```

### 问题 3：内存仍然不足

**解决方案**：
1. 使用更小的量化级别（Q4_K_M 而不是 Q8_0）
2. 使用更小的模型（Qwen3-0.6B）
3. 进一步减少输入长度

---

## 总结

使用 GGUF 量化模型可以：
- ✅ 大幅减少内存占用（60-70%）
- ✅ 加快模型加载速度
- ✅ 保持可接受的质量（对于 commit message 生成）

**推荐步骤**：
1. 下载 Q4_K_M 量化模型
2. 实现量化模型加载代码
3. 测试内存占用和生成质量
4. 根据需要调整量化级别或模型大小

