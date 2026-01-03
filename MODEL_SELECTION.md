# Git Commit Message 生成 - 模型选择分析

## 当前实现分析

### 问题
当前实现只使用**文件名和状态**，没有实际的代码内容：
```rust
// 当前 build_commit_prompt 只包含：
- 修改 src/commands/gcm_ai.rs (后端)
- 新增 src/main.rs (后端)
```

### 改进方向
需要获取**实际的代码 diff**，让模型理解代码变更的具体内容。

---

## 模型选择建议

### 任务需求分析

**任务特点**：
1. **代码理解**：需要理解代码变更的语义
2. **文本生成**：生成符合 Conventional Commits 格式的 commit message
3. **中文支持**：需要支持中文输出
4. **本地推理**：使用 Rust Candle 框架

### ⚠️ 重要：当前代码已支持 Qwen3！

从代码分析来看，`candle_model.rs` **已经实现了 Qwen3 支持**，所以推荐优先考虑 Qwen3 系列模型。

### 推荐模型（按优先级）

#### 🥇 首选：Qwen3-8B-Instruct

**优势**：
- ✅ **最新架构**：2025年发布的最新版本，性能更强
- ✅ **中文支持优秀**：原生支持中英文，支持 119 种语言
- ✅ **Candle 已支持**：代码中已实现 Qwen3 架构支持
- ✅ **指令跟随好**：Instruct 版本适合任务导向
- ✅ **混合思考模式**：自动在"快思考"和"慢思考"之间切换
- ✅ **8B 规模**：在代码理解能力和推理速度之间平衡

**模型信息**：
- Hugging Face: `Qwen/Qwen3-8B-Instruct`
- 大小：约 16GB（FP32）或 8GB（FP16/BF16）
- 架构：Qwen3（与当前代码完全兼容）
- 支持格式：Instruct（对话格式）

**适用场景**：
- 需要理解复杂代码变更
- 需要生成高质量的 commit message
- 对代码语义理解要求高

---

#### 🥈 备选：Qwen3-4B-Instruct

**优势**：
- ✅ **中等规模**：推理速度更快，内存占用更少
- ✅ **中文支持优秀**：原生支持中英文
- ✅ **Candle 已支持**：代码中已实现
- ✅ **性能平衡**：在速度和能力之间取得平衡

**模型信息**：
- Hugging Face: `Qwen/Qwen3-4B-Instruct`
- 大小：约 8GB（FP32）或 4GB（FP16/BF16）
- 架构：Qwen3

**适用场景**：
- 代码变更相对简单
- 需要更快的推理速度
- 硬件资源有限

---

#### 🥉 备选：Qwen3-1.7B-Instruct

**优势**：
- ✅ **轻量级**：推理速度最快，内存占用最小
- ✅ **中文支持优秀**
- ✅ **Candle 已支持**
- ✅ **快速反馈**：适合快速生成 commit message

**模型信息**：
- Hugging Face: `Qwen/Qwen3-1.7B-Instruct`
- 大小：约 3.4GB（FP32）或 1.7GB（FP16/BF16）
- 架构：Qwen3

**适用场景**：
- 简单代码变更
- 需要极快的推理速度
- 硬件资源非常有限

---

#### 其他 Qwen3 选项

**Qwen3-0.6B-Instruct**：
- ✅ 超轻量级（约 1.2GB）
- ⚠️ 能力有限，适合非常简单的变更

**Qwen3-30B-A3B (MoE)**：
- ✅ 混合专家架构，激活参数少
- ⚠️ 总参数大，需要更多资源

---

#### 传统选项（如果 Qwen3 不可用）

**Qwen2.5-Coder-7B-Instruct**：
- ✅ **代码理解能力强**：专门为代码任务训练
- ✅ **中文支持优秀**
- ⚠️ **需要修改代码**：当前代码只支持 Qwen3，需要添加 Qwen2.5 支持

**Qwen2.5-7B-Instruct**：
- ✅ **通用能力强**
- ⚠️ **需要修改代码**：需要添加 Qwen2.5 支持

**DeepSeek-Coder-6.7B-Instruct**：
- ✅ **代码理解能力强**
- ⚠️ **需要添加架构支持**：Candle 可能不支持

---

## 模型对比表

### Qwen3 系列（推荐，代码已支持）

| 模型 | 代码理解 | 中文支持 | Candle支持 | 大小 | 推荐度 |
|------|---------|---------|-----------|------|--------|
| **Qwen3-8B-Instruct** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 8-16GB | ⭐⭐⭐⭐⭐ |
| **Qwen3-4B-Instruct** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 4-8GB | ⭐⭐⭐⭐ |
| **Qwen3-1.7B-Instruct** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 1.7-3.4GB | ⭐⭐⭐ |
| Qwen3-0.6B-Instruct | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 0.6-1.2GB | ⭐⭐ |
| Qwen3-30B-A3B (MoE) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 较大 | ⭐⭐⭐ |

### 其他选项（需要修改代码）

| 模型 | 代码理解 | 中文支持 | Candle支持 | 大小 | 推荐度 |
|------|---------|---------|-----------|------|--------|
| Qwen2.5-Coder-7B-Instruct | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⚠️需添加支持 | 7-14GB | ⭐⭐⭐ |
| Qwen2.5-7B-Instruct | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⚠️需添加支持 | 7-14GB | ⭐⭐ |
| DeepSeek-Coder-6.7B | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⚠️需添加支持 | 6.7GB | ⭐⭐ |

---

## 实现建议

### 1. 获取代码 Diff

```rust
fn get_git_diff() -> Result<String, Box<dyn std::error::Error>> {
    // 获取暂存区的 diff
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")  // 暂存区的变更
        .arg("--no-color")  // 去除颜色代码
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // 如果没有暂存区变更，获取工作区变更
        let output = Command::new("git")
            .arg("diff")
            .arg("--no-color")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

### 2. 改进 Prompt 构建

```rust
fn build_commit_prompt_with_diff(
    changes: &[ChangeInfo], 
    diff: &str,
    file_type_manager: &FileTypeManager
) -> String {
    let mut prompt = String::from(
        "你是一个专业的 Git 提交信息生成助手。根据以下代码变更，生成一个符合 Conventional Commits 规范的 commit message。\n\n"
    );
    
    // 文件变更列表
    prompt.push_str("变更文件：\n");
    for change in changes {
        let action = match change.status.as_str() {
            "A" => "新增",
            "M" => "修改",
            "D" => "删除",
            "R" => "重命名",
            _ => "变更",
        };
        prompt.push_str(&format!("- {} {} ({})\n", action, change.file, change.category));
    }
    
    // 代码 Diff（限制长度避免超出上下文）
    prompt.push_str("\n代码变更内容：\n");
    prompt.push_str("```diff\n");
    
    // 限制 diff 长度（例如前 2000 行）
    let diff_lines: Vec<&str> = diff.lines().take(2000).collect();
    prompt.push_str(&diff_lines.join("\n"));
    
    if diff.lines().count() > 2000 {
        prompt.push_str("\n... (还有更多变更，已截断)");
    }
    
    prompt.push_str("\n```\n");
    
    // 要求
    prompt.push_str("\n要求：\n");
    prompt.push_str("1. 使用中文\n");
    prompt.push_str("2. 格式：<type>: <description>\n");
    prompt.push_str("3. type 可以是：feat, fix, refactor, style, docs, test, chore, perf, cleanup\n");
    prompt.push_str("4. description 要简洁明了，准确描述代码变更的主要目的\n");
    prompt.push_str("5. 只返回 commit message，不要其他说明\n");
    prompt.push_str("6. 根据代码变更的具体内容，而不是文件名，来判断变更类型\n");
    
    prompt
}
```

### 3. 处理大 Diff

如果 diff 太大，可以：
- **截断**：只取前 N 行
- **摘要**：对每个文件分别处理，然后合并
- **智能选择**：优先包含关键变更（如函数签名、类定义等）

---

## 推荐方案

### 方案 1：Qwen3-8B-Instruct（强烈推荐）

**理由**：
1. **代码已支持**：`candle_model.rs` 已实现 Qwen3 架构，无需修改代码
2. **最新版本**：2025年发布，性能最强
3. **代码理解能力强**：适合代码任务
4. **中文支持优秀**：原生支持中英文
5. **8B 规模**：在能力和速度之间平衡良好

**实现步骤**：
1. 下载模型：`Qwen/Qwen3-8B-Instruct`
   ```bash
   huggingface-cli download Qwen/Qwen3-8B-Instruct \
     --local-dir ~/.cyber-zen/models/qwen3-8b \
     --local-dir-use-symlinks False
   ```
2. 修改 `build_commit_prompt` 包含代码 diff（见下方实现建议）
3. 测试生成质量

### 方案 2：Qwen3-4B-Instruct（推荐，速度优先）

**理由**：
1. **代码已支持**：无需修改代码
2. **推理更快**：4B 规模，速度快约 2 倍
3. **内存占用更少**：适合资源有限的机器
4. **性能仍然优秀**：对于大多数代码变更足够好

**适用场景**：
- 需要快速反馈
- 硬件资源有限
- 代码变更相对简单

### 方案 3：Qwen3-1.7B-Instruct（轻量级）

**理由**：
1. **代码已支持**：无需修改代码
2. **极快速度**：1.7B 规模，速度最快
3. **最小内存**：适合低配置机器
4. **快速迭代**：适合频繁使用

**适用场景**：
- 简单代码变更
- 需要极快的响应时间
- 硬件资源非常有限

---

## 性能考虑

### 模型大小 vs 速度

| 模型大小 | 推理速度（CPU） | 内存占用 | 推荐场景 |
|---------|---------------|---------|---------|
| 0.5B | 快 | 低（~2GB） | 简单变更，快速反馈 |
| 1.5B-3B | 中等 | 中（~4-8GB） | 平衡选择 |
| 7B | 慢 | 高（~14GB） | 复杂变更，高质量输出 |
| 14B+ | 很慢 | 很高（~28GB+） | 不推荐本地推理 |

### 优化建议

1. **使用量化**：FP16/BF16 可以减少一半内存
2. **GPU 加速**：如果有 GPU，速度可提升 10-100 倍
3. **上下文截断**：限制 diff 长度，避免超出模型上下文窗口

---

## 总结

### 🎯 强烈推荐：Qwen3-8B-Instruct

**原因**：
- ✅ **代码已完全支持**：无需任何架构修改，直接可用
- ✅ **最新最强性能**：2025年最新版本
- ✅ **中文支持优秀**：原生支持 119 种语言
- ✅ **代码理解能力强**：适合代码任务
- ✅ **8B 规模**：在质量和速度之间平衡

### 🚀 快速开始推荐：Qwen3-4B-Instruct

**原因**：
- ✅ **代码已完全支持**：无需任何架构修改
- ✅ **速度快**：推理速度约是 8B 的 2 倍
- ✅ **性能优秀**：对于大多数场景足够好
- ✅ **资源友好**：内存占用更少

### 📝 下一步行动

1. **选择模型**：
   - 优先推荐：`Qwen3-8B-Instruct`（质量优先）
   - 或选择：`Qwen3-4B-Instruct`（速度优先）

2. **下载模型**：
   ```bash
   # 创建模型目录
   mkdir -p ~/.cyber-zen/models/qwen3-8b
   
   # 下载模型（使用 huggingface-cli）
   huggingface-cli download Qwen/Qwen3-8B-Instruct \
     --local-dir ~/.cyber-zen/models/qwen3-8b \
     --local-dir-use-symlinks False
   ```

3. **实现代码 diff 获取功能**（见下方实现建议）

4. **改进 prompt 构建**，包含实际代码变更

5. **测试不同模型的效果**，根据实际情况调整

