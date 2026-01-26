# Qwen2.5-Coder-0.5B-Instruct GGUF 模型设置

## 模型信息

- **模型名称**：Qwen2.5-Coder-0.5B-Instruct
- **量化格式**：GGUF Q4_K_M
- **模型大小**：约 350-400MB（量化后）
- **内存占用**：约 1-2GB（运行时）
- **优势**：超轻量级，适合低配置电脑

## 下载模型

### 1. 创建模型目录

```bash
mkdir -p ~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf
```

### 2. 下载 GGUF 模型文件

```bash
# 使用 huggingface-cli 下载
huggingface-cli download lmstudio-community/Qwen2.5-Coder-0.5B-Instruct-GGUF \
  qwen2.5-coder-0.5b-instruct-q4_k_m.gguf \
  --local-dir ~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf \
  --local-dir-use-symlinks False
```

或者手动下载：
- Hugging Face: https://huggingface.co/lmstudio-community/Qwen2.5-Coder-0.5B-Instruct-GGUF
- 文件：`qwen2.5-coder-0.5b-instruct-q4_k_m.gguf`

### 3. 下载 Tokenizer

```bash
# 从原始模型下载 tokenizer
huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct \
  tokenizer.json \
  --local-dir ~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf \
  --local-dir-use-symlinks False
```

或者手动下载：
- Hugging Face: https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct
- 文件：`tokenizer.json`

### 4. 验证文件

下载完成后，检查文件是否存在：

```bash
ls -lh ~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf/
```

应该看到：
- `qwen2.5-coder-0.5b-instruct-q4_k_m.gguf` (约 350-400MB)
- `tokenizer.json` (约 11MB)

## 目录结构

```
~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf/
├── qwen2.5-coder-0.5b-instruct-q4_k_m.gguf  # GGUF 量化模型
└── tokenizer.json                            # Tokenizer 文件
```

## 使用

模型下载完成后，代码会自动检测并使用 GGUF 格式：

```bash
# 构建项目
cargo build

# 在 Git 仓库中测试
./target/debug/cyber-zen-tools gcm-ai --dry-run
```

## 优势

使用 0.5B 模型相比 7B 模型：

- ✅ **内存占用更少**：1-2GB vs 4-6GB（减少 70%）
- ✅ **加载速度更快**：文件更小，加载更快
- ✅ **适合低配置电脑**：即使在 8GB 内存的电脑上也能运行
- ✅ **代码理解能力**：虽然比 7B 稍弱，但对于 commit message 生成足够好

## 性能对比

| 特性 | 0.5B Q4_K_M | 7B Q4_K_M |
|------|------------|-----------|
| 模型大小 | ~350MB | ~4-5GB |
| 内存占用 | 1-2GB | 4-6GB |
| 加载时间 | ~2-3秒 | ~5-8秒 |
| 推理速度 | 较快 | 中等 |
| 代码理解 | 良好 | 优秀 |

## 故障排除

### 问题 1：找不到模型文件

**错误信息**：
```
模型文件夹不存在: ~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf
```

**解决方案**：
1. 确认模型目录路径正确
2. 检查目录名称是否匹配配置文件中的 `MODEL_FOLDER_NAME`

### 问题 2：找不到 tokenizer.json

**错误信息**：
```
未找到 tokenizer.json: ...
```

**解决方案**：
1. 确保 tokenizer.json 在模型目录中
2. 从 Qwen/Qwen2.5-Coder-0.5B-Instruct 下载 tokenizer.json

### 问题 3：GGUF 文件格式错误

**错误信息**：
```
无法读取 GGUF 文件: ...
```

**解决方案**：
1. 确保下载的 GGUF 文件完整（未损坏）
2. 重新下载模型文件
3. 检查文件大小是否正确（约 350-400MB）

## 参考链接

- Qwen2.5-Coder 模型：https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct
- GGUF 模型：https://huggingface.co/lmstudio-community/Qwen2.5-Coder-0.5B-Instruct-GGUF
- Candle 文档：https://github.com/huggingface/candle

