# Qwen2.5-Coder-0.5B-Instruct 设置指南

## 当前状态

✅ **代码已完成并可以使用！** Candle 0.9 版本已包含 Qwen2 支持，无需使用 git 版本。

## 功能状态

- ✅ 模型文件查找逻辑（支持 `default_*` 目录）
- ✅ Config.json 加载和解析
- ✅ Tokenizer 加载
- ✅ 模型权重文件加载（支持分片模型）
- ✅ Qwen2 模型实例化
- ✅ 实际推理循环实现
- ✅ Qwen 指令格式适配

### 4. 模型文件要求

模型文件应放在：
```
~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct/
├── model.safetensors    # 模型权重文件（或分片文件 model-*.safetensors）
├── config.json          # 模型配置文件
└── tokenizer.json       # Tokenizer 文件
```

### 5. 下载模型

从 Hugging Face 下载 Qwen2.5-Coder-0.5B-Instruct：
```bash
huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct \
  --local-dir ~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct
```

## 使用方法

### 1. 下载模型

从 Hugging Face 下载 Qwen2.5-Coder-0.5B-Instruct：

```bash
# 使用 huggingface-cli（推荐）
huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct \
  --local-dir ~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct

# 或者手动下载以下文件到 ~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct/：
# - model.safetensors（或分片文件 model-*.safetensors）
# - config.json
# - tokenizer.json
```

### 2. 使用 gcm-ai 命令

```bash
# 生成新的 commit message
cyber-zen gcm-ai

# 使用指定的模型
cyber-zen gcm-ai --model Qwen2.5-Coder-0.5B-Instruct

# 重写历史提交
cyber-zen gcm-ai --rewrite --max-commits 10
```

## 注意事项

1. 首次运行会加载模型，可能需要一些时间
2. 模型文件较大（约 1GB），确保有足够的磁盘空间
3. 当前使用 CPU 推理，速度可能较慢。如需 GPU 加速，需要配置 CUDA 支持
4. 推理循环使用简化的采样策略（argmax），未来可以优化为 temperature/top-k/top-p 采样

