# GGUF vs Safetensors 格式说明

## 主要区别

### GGUF 格式（当前使用）

**特点**：
- ✅ **单一文件**：模型权重、架构和元数据都在一个 `.gguf` 文件中
- ✅ **量化支持**：内置多种量化级别（Q4_K_M, Q8_0 等）
- ✅ **高效加载**：支持内存映射（mmap），加载速度快
- ✅ **文件结构简单**：只需要 `.gguf` 文件 + `tokenizer.json`

**文件结构**：
```
~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf/
├── qwen2.5-coder-0.5b-instruct-q4_k_m.gguf  # 单一模型文件（约 350-400MB）
└── tokenizer.json                            # Tokenizer 文件（约 11MB）
```

**代码处理方式**：
- 通过文件扩展名 `.gguf` 识别
- 使用 `CandleModelQuantized::load_from_path()` 加载
- 使用 `candle::quantized::gguf_file` 解析

### Safetensors 格式

**特点**：
- ✅ **安全**：不包含可执行代码，只有张量数据
- ✅ **高效加载**：支持延迟加载和部分加载
- ✅ **多文件结构**：需要多个文件（config.json, model.safetensors 等）

**文件结构**：
```
~/.cyber-zen/models/Qwen3-1.7B-Instruct/
├── config.json                    # 模型配置文件
├── model.safetensors              # 或 model-00001-of-00002.safetensors（分片）
├── model-00002-of-00002.safetensors
├── tokenizer.json
├── tokenizer_config.json
└── ...
```

**代码处理方式**：
- 通过文件扩展名 `.safetensors` 识别
- 使用 `CandleModel::load_from_path()` 加载
- 使用 `safetensors` crate 解析

## 代码中的区分逻辑

### 1. 文件查找 (`find_model_file()`)

```rust
// 优先检查 GGUF 文件（量化模型）
if let Ok(entries) = model_folder.read_dir() {
    let mut gguf_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "gguf")  // 通过扩展名识别
                .unwrap_or(false)
        })
        .collect();
    
    if !gguf_files.is_empty() {
        return Ok(gguf_file.clone());  // 返回 GGUF 文件路径
    }
}

// 如果没有 GGUF 文件，检查 safetensors 文件
// ...
```

### 2. 模型加载 (`call_local_model()`)

```rust
// 检查文件扩展名，判断使用哪种模型加载方式
let is_gguf = model_path.extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| ext == "gguf")
    .unwrap_or(false);

if is_gguf {
    // 使用量化模型（GGUF 格式）
    let mut model = CandleModelQuantized::load_from_path(model_path)?;
    // ...
} else {
    // 使用普通模型（safetensors 格式）
    let mut model = CandleModel::load_from_path(model_path)?;
    // ...
}
```

## 为什么选择 GGUF？

对于 0.5B 模型使用 GGUF 格式的优势：

1. **文件更小**：Q4_K_M 量化后约 350-400MB，比 safetensors 小 75%
2. **内存占用低**：运行时约 1-2GB，适合低配置电脑
3. **加载更快**：单一文件，无需解析多个文件
4. **结构简单**：只需要 2 个文件（.gguf + tokenizer.json）

## 当前配置

- **模型格式**：GGUF Q4_K_M 量化
- **模型文件**：`qwen2.5-coder-0.5b-instruct-q4_k_m.gguf`
- **模型目录**：`~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf/`
- **必需文件**：
  - `qwen2.5-coder-0.5b-instruct-q4_k_m.gguf`
  - `tokenizer.json`

## 注意事项

1. **GGUF 文件是自包含的**：所有模型权重都在一个文件中，不需要 config.json
2. **Tokenizer 需要单独下载**：从原始模型仓库下载 `tokenizer.json`
3. **文件位置**：GGUF 文件和 tokenizer.json 必须在同一目录中
4. **代码自动识别**：代码会根据文件扩展名自动选择正确的加载方式

