# 本地模型推理的替代方案

## 现实情况

本地运行大语言模型（即使是量化版本）对硬件要求很高：
- **内存需求**：即使是 Q4_K_M 量化模型，也需要 4-8GB 内存
- **计算能力**：CPU 推理速度很慢，可能几分钟甚至更久
- **实际体验**：可能卡顿、超时或被系统 kill
- **资源占用**：影响其他应用程序的使用

对于日常使用，本地推理确实不现实。

---

## 推荐方案

### 方案 1：使用现有的规则生成（推荐）⭐⭐⭐

你已经有了 `gcm` 命令，使用规则和配置文件生成 commit message。

#### 使用方法

```bash
# 使用规则生成（不依赖 AI 模型）
cyber-zen gcm
```

#### 优点
- ✅ **无需任何模型**：不占用内存和磁盘
- ✅ **速度快**：即时生成
- ✅ **资源占用低**：几乎为零
- ✅ **可配置**：通过配置文件自定义
- ✅ **稳定可靠**：不会因为资源问题失败

#### 如何改进

可以优化配置文件来提高生成质量：
- 完善 `commit-templates.toml` - 添加更多模板
- 优化 `categories.toml` - 改进分类规则
- 改进 `file-types.toml` - 更准确的类型识别

---

### 方案 2：未来考虑 API 服务

如果将来需要 AI 功能，可以考虑使用在线 API：

#### 可选的 API 服务

1. **OpenRouter** - 提供多种模型，价格合理
2. **Together AI** - 开源模型 API，有免费层
3. **Groq** - 速度快，免费层额度
4. **Hugging Face Inference API** - 官方服务

#### 实现方式

```rust
// 使用 HTTP 请求调用 API
let response = reqwest::Client::new()
    .post("https://api.openrouter.ai/api/v1/chat/completions")
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&json!({
        "model": "qwen/qwen2.5-coder-7b-instruct",
        "messages": [
            {"role": "system", "content": "..."},
            {"role": "user", "content": prompt}
        ]
    }))
    .send()
    .await?;
```

---

### 方案 3：禁用或移除 AI 功能

如果确定不需要 AI 功能，可以：

1. **移除 AI 相关代码和依赖**
   - 删除 `candle_model.rs` 和 `candle_model_quantized.rs`
   - 移除 `gcm-ai` 命令
   - 减少项目体积和编译时间

2. **仅保留规则生成**
   - 保留 `gcm` 命令
   - 专注于优化规则生成

---

## 当前建议

### 短期方案（立即实施）

**使用 `gcm` 命令（规则生成）**：
- 这是最可靠的方案
- 不依赖任何外部资源
- 可以满足大多数使用场景

### 改进规则生成

可以通过以下方式提高规则生成的质量：

1. **完善配置文件**：
   ```toml
   # configs/commit-templates.toml
   # 添加更多模板和规则
   ```

2. **优化分类规则**：
   ```toml
   # configs/categories.toml
   # 改进文件分类逻辑
   ```

3. **添加更多文件类型**：
   ```toml
   # configs/file-types.toml
   # 支持更多文件类型识别
   ```

---

## 代码处理建议

### 选项 1：改进错误处理（推荐）

在 `gcm-ai` 命令失败时，自动回退到规则生成：

```rust
match generate_ai_commit_message(&model_path) {
    Ok(m) => m,
    Err(e) => {
        eprintln!("AI 生成失败: {}", e);
        println!("回退到规则生成...");
        // 调用规则生成函数
        commands::gcm::generate_commit_message()?
    }
}
```

### 选项 2：添加功能开关

在配置文件中添加开关，让用户选择是否启用 AI 功能。

### 选项 3：移除 AI 功能

如果确定不需要，可以完全移除相关代码。

---

## 总结

对于配置有限的电脑：

1. **推荐使用规则生成**（`gcm` 命令）
   - 可靠、快速、无需额外资源
   - 可以满足大多数场景

2. **如果未来需要 AI**
   - 考虑使用在线 API
   - 或者在有 GPU 的服务器上运行

3. **本地模型推理**
   - 需要大量资源
   - 对于日常使用不实用
   - 建议仅在特殊场景使用

---

## 下一步行动

1. **继续使用 `gcm` 命令**（规则生成）
2. **优化配置文件**以提高生成质量
3. **移除或禁用 `gcm-ai` 功能**（如果不需要）
4. **或者改进 `gcm-ai` 错误处理**，失败时自动回退到规则生成
