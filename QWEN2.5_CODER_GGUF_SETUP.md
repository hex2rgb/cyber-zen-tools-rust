# Qwen2.5-Coder-7B-Instruct GGUF 模型设置指南

## 已完成的修改

✅ 代码已更新，支持 GGUF 量化模型（Qwen2.5-Coder-7B-Instruct）
✅ 自动检测模型格式（GGUF 或 safetensors）
✅ 配置已更新为使用量化模型

## 下载模型

### 1. 创建模型目录

```bash
mkdir -p ~/.cyber-zen/models/qwen2.5-coder-7b-gguf
```

### 2. 下载 GGUF 量化模型

```bash
# 使用 huggingface-cli 下载
huggingface-cli download lmstudio-community/Qwen2.5-Coder-7B-Instruct-GGUF \
  qwen2.5-coder-7b-instruct-q4_k_m.gguf \
  --local-dir ~/.cyber-zen/models/qwen2.5-coder-7b-gguf \
  --local-dir-use-symlinks False
```

或者从以下地址手动下载：
- Hugging Face: https://huggingface.co/lmstudio-community/Qwen2.5-Coder-7B-Instruct-GGUF
- 文件：`qwen2.5-coder-7b-instruct-q4_k_m.gguf`

### 3. 下载 Tokenizer

```bash
# 从原始模型下载 tokenizer
huggingface-cli download Qwen/Qwen2.5-Coder-7B-Instruct \
  tokenizer.json \
  --local-dir ~/.cyber-zen/models/qwen2.5-coder-7b-gguf \
  --local-dir-use-symlinks False
```

或者从以下地址手动下载：
- Hugging Face: https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct
- 文件：`tokenizer.json`

### 4. 验证文件

下载完成后，检查文件是否存在：

```bash
ls -lh ~/.cyber-zen/models/qwen2.5-coder-7b-gguf/
```

应该看到：
- `qwen2.5-coder-7b-instruct-q4_k_m.gguf` (约 4-5GB)
- `tokenizer.json` (约 11MB)

## 目录结构

```
~/.cyber-zen/models/qwen2.5-coder-7b-gguf/
├── qwen2.5-coder-7b-instruct-q4_k_m.gguf  # GGUF 量化模型
└── tokenizer.json                          # Tokenizer 文件
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

使用 Qwen2.5-Coder-7B-Instruct Q4_K_M 量化模型：

- ✅ **代码理解能力强**：专门为代码任务训练
- ✅ **内存占用更少**：约 4-5GB（相比 FP32 的 14GB）
- ✅ **加载速度更快**：量化模型加载更快
- ✅ **质量损失小**：Q4_K_M 在质量和大小之间平衡良好

## 模型信息

- **模型名称**：Qwen2.5-Coder-7B-Instruct
- **量化格式**：GGUF Q4_K_M
- **模型大小**：约 4-5GB（量化后）
- **原始大小**：约 14GB（FP32）
- **内存占用**：约 4-6GB（运行时）
- **推荐场景**：代码理解、commit message 生成

## 故障排除

### 问题 1：找不到模型文件

**错误信息**：
```
模型文件夹不存在: ~/.cyber-zen/models/qwen2.5-coder-7b-gguf
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
2. 从 Qwen/Qwen2.5-Coder-7B-Instruct 下载 tokenizer.json

### 问题 3：GGUF 文件格式错误

**错误信息**：
```
无法读取 GGUF 文件: ...
```

**解决方案**：
1. 确保下载的 GGUF 文件完整（未损坏）
2. 重新下载模型文件
3. 检查文件大小是否正确（约 4-5GB）

### 问题 4：内存仍然不足

如果使用量化模型后仍然内存不足：

1. **使用更小的量化级别**：
   - 考虑 Q3_K_M 或 Q2_K（质量会降低）

2. **使用更小的模型**：
   - Qwen2.5-Coder-1.5B-Instruct
   - Qwen2.5-Coder-0.5B-Instruct

3. **进一步减少输入长度**：
   - 将 `MAX_DIFF_LINES` 从 500 减少到 300

## 其他量化级别

如果需要不同的量化级别，可以从以下仓库下载：

- **Q8_0**（高质量，较大）：
  ```bash
  huggingface-cli download lmstudio-community/Qwen2.5-Coder-7B-Instruct-GGUF \
    qwen2.5-coder-7b-instruct-q8_0.gguf \
    --local-dir ~/.cyber-zen/models/qwen2.5-coder-7b-gguf
  ```

- **Q5_K_M**（平衡）：
  ```bash
  huggingface-cli download lmstudio-community/Qwen2.5-Coder-7B-Instruct-GGUF \
    qwen2.5-coder-7b-instruct-q5_k_m.gguf \
    --local-dir ~/.cyber-zen/models/qwen2.5-coder-7b-gguf
  ```

## 参考链接

- Qwen2.5-Coder 模型：https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct
- GGUF 模型：https://huggingface.co/lmstudio-community/Qwen2.5-Coder-7B-Instruct-GGUF
- Candle 文档：https://github.com/huggingface/candle

