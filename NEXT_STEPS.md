# 下一步工作计划

## ✅ 已完成

1. ✅ 实现 Qwen2.5-Coder-0.5B-Instruct 模型加载
2. ✅ 实现模型推理循环
3. ✅ 集成到 gcm-ai 命令
4. ✅ 代码编译通过（包括 release 模式）
5. ✅ 更新文档

## 📋 待完成（按优先级）

### 1. 测试功能（高优先级）

**目标**：验证整个流程是否正常工作

**步骤**：
1. 下载模型文件到 `~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct/`
2. 运行 `cyber-zen gcm-ai` 测试生成 commit message
3. 检查生成的 message 质量
4. 测试 `--rewrite` 功能

**命令**：
```bash
# 下载模型
huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct \
  --local-dir ~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct

# 测试
cd /path/to/test/repo
cyber-zen gcm-ai
```

### 2. 优化推理性能（中优先级）

**当前问题**：
- 每次生成都重新处理整个序列（没有使用 KV cache）
- 使用简单的 argmax 采样，可能生成质量不够好

**优化方向**：
1. 实现 KV cache 管理（提高推理速度）
2. 添加 temperature/top-k/top-p 采样策略
3. 优化 token 生成循环

**参考**：
- Candle 的 Qwen 示例代码中的 KV cache 实现
- 采样策略可以参考其他 LLM 推理库

### 3. 错误处理和用户体验（中优先级）

**改进点**：
1. 添加更详细的错误信息
2. 显示推理进度（已生成 token 数量）
3. 添加超时处理
4. 优化加载模型的提示信息

### 4. 代码优化（低优先级）

**改进点**：
1. 清理未使用的代码和警告
2. 添加代码注释
3. 优化代码结构
4. 添加单元测试

### 5. 功能扩展（低优先级）

**可能的扩展**：
1. 支持 GPU 加速（CUDA）
2. 支持量化模型（减少内存占用）
3. 支持批量处理多个 commit
4. 添加模型性能指标（生成速度、token 数量等）

## 🚀 立即可以做的

1. **下载并测试模型**（最重要）
   ```bash
   huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct \
     --local-dir ~/.cyber-zen/models/default_Qwen2.5-Coder-0.5B-Instruct
   ```

2. **运行测试**
   ```bash
   cargo build --release
   ./target/release/cyber-zen-tools gcm-ai
   ```

3. **检查生成的 commit message 质量**

## 📝 注意事项

- 当前实现使用 CPU 推理，速度可能较慢
- 首次加载模型需要时间
- 如果遇到错误，检查模型文件是否完整（config.json, tokenizer.json, model.safetensors）


