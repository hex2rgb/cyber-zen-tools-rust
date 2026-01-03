# 内存优化方案

## 问题分析

进程被系统 kill (`zsh: killed`) 说明内存不足（OOM - Out of Memory）。

### 当前资源消耗

1. **模型文件大小**：3.2GB (FP32)
2. **加载到内存后**：约 6-8GB（模型权重 + 推理缓冲区）
3. **输入序列长度**：21,610 tokens（非常长！）
4. **系统内存**：16GB（总共）

### 问题根源

1. **模型精度**：使用 FP32（32位浮点数），内存占用大
2. **输入序列过长**：21,610 tokens 的输入会占用大量内存进行注意力计算
3. **CPU 推理**：CPU 推理需要将整个模型加载到内存，无法像 GPU 那样高效

---

## 优化方案

### 方案 1：限制输入长度（最简单，立即生效）⭐推荐

**问题**：当前 diff 有 1880 行，编码后变成 21,610 tokens，这太长了。

**解决方案**：大幅减少 diff 长度

```rust
// 将 MAX_DIFF_LINES 从 3000 减少到 500-800
const MAX_DIFF_LINES: usize = 800;  // 对于 commit message 生成，这个长度通常足够
```

**效果**：
- 减少输入序列长度约 70-80%
- 内存占用降低约 50-60%
- 推理速度提升 2-3 倍

---

### 方案 2：使用量化模型（最佳方案）⭐⭐⭐

**问题**：FP32 模型占用内存太大

**解决方案**：使用 GGUF 量化模型（Q4_K_M 或 Q8_0）

**优势**：
- 模型大小：3.2GB → 1.0-1.5GB（Q4_K_M）或 1.8-2.0GB（Q8_0）
- 内存占用：6-8GB → 2-4GB
- 速度：可能稍慢，但内存占用大幅降低
- 质量：Q8_0 几乎无损，Q4_K_M 略有损失但可接受

**下载量化模型**：
```bash
# Q4_K_M（推荐，平衡）
huggingface-cli download unsloth/Qwen3-1.7B-GGUF Qwen3-1.7B-Q4_K_M.gguf --local-dir ~/.cyber-zen/models/

# Q8_0（质量更好，但稍大）
huggingface-cli download unsloth/Qwen3-1.7B-GGUF Qwen3-1.7B-Q8_0.gguf --local-dir ~/.cyber-zen/models/
```

**需要修改代码**：使用 `candle-transformers::models::quantized_qwen3` 而不是普通模型

---

### 方案 3：智能截断 Diff（平衡方案）⭐⭐

**思路**：不是简单截断，而是选择重要的部分

**策略**：
1. 优先保留函数签名、类定义等关键变更
2. 对于大文件，只保留前 N 行和后 N 行（上下文）
3. 对于删除的文件，只保留文件名
4. 使用 diff 统计信息（git diff --stat）来识别主要变更

**实现示例**：
```rust
fn smart_truncate_diff(diff: &str, max_lines: usize) -> String {
    // 1. 按文件分组
    // 2. 对每个文件，保留关键部分
    // 3. 总长度不超过 max_lines
}
```

---

### 方案 4：减小模型规模

**选项**：
- Qwen3-0.6B-Instruct（超轻量，约 1.2GB，内存占用 2-3GB）
- 如果效果不够好，可以考虑 Qwen3-1.7B 的量化版本

---

## 推荐实施顺序

### 第一步：立即优化（方案 1）✅

**修改 `build_commit_prompt` 函数**：

```rust
// 将 MAX_DIFF_LINES 从 3000 减少到 500
const MAX_DIFF_LINES: usize = 500;
```

**效果**：立即减少内存占用 50-70%

### 第二步：考虑量化模型（方案 2）✅

如果方案 1 还不够，考虑使用量化模型。

---

## 代码修改示例

### 修改 1：限制 Diff 长度

在 `src/commands/gcm_ai.rs` 中：

```rust
fn build_commit_prompt(
    changes: &[ChangeInfo], 
    diff: &str,
    _file_type_manager: &FileTypeManager
) -> String {
    // ... 现有代码 ...
    
    // 代码 Diff（限制长度避免超出上下文）
    if !diff.trim().is_empty() {
        prompt.push_str("\n代码变更内容：\n");
        prompt.push_str("```diff\n");
        
        // 限制 diff 长度（减少到 500 行，避免内存溢出）
        const MAX_DIFF_LINES: usize = 500;  // 从 3000 减少到 500
        let diff_lines: Vec<&str> = diff.lines().take(MAX_DIFF_LINES).collect();
        prompt.push_str(&diff_lines.join("\n"));
        
        let total_lines = diff.lines().count();
        if total_lines > MAX_DIFF_LINES {
            prompt.push_str(&format!("\n... (还有 {} 行变更，已截断。主要变更已包含)", total_lines - MAX_DIFF_LINES));
        }
        
        prompt.push_str("\n```\n");
    }
    
    // ... 现有代码 ...
}
```

### 修改 2：添加内存使用提示

在生成前提示用户：

```rust
println!("{}", "注意: 模型推理可能需要 4-8GB 内存".yellow());
println!("{}", "如果内存不足，建议限制 diff 大小或使用量化模型".yellow());
```

---

## 内存使用估算

### FP32 模型（当前）

| 组件 | 内存占用 |
|------|---------|
| 模型权重（FP32） | 3.2GB × 2 = 6.4GB（加载后） |
| 推理缓冲区 | 1-2GB |
| 输入序列（21k tokens） | 1-2GB |
| **总计** | **8-10GB** |

### Q4_K_M 量化模型

| 组件 | 内存占用 |
|------|---------|
| 模型权重（Q4_K_M） | 1.0GB × 2 = 2GB |
| 推理缓冲区 | 0.5-1GB |
| 输入序列（5k tokens） | 0.5-1GB |
| **总计** | **3-4GB** |

---

## 总结

**立即行动**：
1. ✅ 将 `MAX_DIFF_LINES` 从 3000 减少到 500
2. ✅ 测试是否能正常运行

**如果还不够**：
3. ✅ 考虑使用量化模型（Q4_K_M 或 Q8_0）
4. ✅ 或使用更小的模型（Qwen3-0.6B）

