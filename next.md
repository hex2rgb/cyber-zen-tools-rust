# Local AI Commit Message Generator（Candle + GGUF 优化说明）

> 本项目使用 **Candle + GGUF 本地模型** 生成 Git commit message。  
> **该方案已针对小模型（0.5B）+ CPU 场景做过性能与稳定性优化**。

---

## 一、重要结论（必须先看）

### ❗ 不需要 `tokenizer.json`

- 本项目 **使用 GGUF 模型**
- GGUF **内嵌 tokenizer**
- Candle **直接从 GGUF 读取 tokenizer 信息**

👉 **任何要求额外 `tokenizer.json` 的行为都是错误的**

---

## 二、模型选择（已验证）

### ✅ 推荐模型（默认）

```text
qwen2.5-0.5b-instruct-q4_k_m.gguf
```

#### 选择原因

- CPU 推理速度可接受
- 中文指令遵循能力稳定
- 内存 / 性能 / 质量三者平衡
- 适合短文本生成（commit message）

---

### ❌ 不推荐模型

| 模型 | 原因 |
|----|----|
| fp16.gguf | CPU 推理极慢 |
| q8_0 | 性能灾难 |
| q6_k | 明显拖慢 |
| q2_k | 输出不稳定，易胡说 |
| Coder 系列 | 不适合总结 / 归纳任务 |

> **Commit message = 总结任务**  
> 必须使用 **Instruct 模型**

---

## 三、核心性能优化策略（非常重要）

### 1️⃣ 严格限制 prompt 大小

#### ❌ 错误做法

- 把完整 `git diff` 喂给模型
- 假设小模型能“自己理解重点”

#### ✅ 正确做法

```text
最大 diff 行数：20
最大 prompt 长度：≤ 2000 chars
```

示例：

```rust
const MAX_DIFF_LINES: usize = 20;

if prompt.len() > 4000 {
    return Err("Prompt 过长，拒绝推理".into());
}
```

---

### 2️⃣ 明确小模型能力边界

**0.5B 模型只能做：**

- 简短总结
- 模板化输出
- 单一任务指令

**不能指望它：**

- 阅读大量代码
- 推理复杂业务逻辑
- 像 GPT-4 一样理解 diff

---

### 3️⃣ Token 生成上限（防卡死）

```text
max_tokens: 32 ~ 50
```

Commit message **永远不需要更多 token**。

---

### 4️⃣ 多线程（必须）

确保 Candle 使用 CPU 全核心：

```text
n_threads = CPU 核心数
```

单线程推理会导致 **“看起来卡死”**。

---

## 四、推荐推理配置（标准答案）

```text
模型：qwen2.5-0.5b-instruct-q4_k_m.gguf
diff 行数：≤ 20
prompt 长度：≤ 2k chars
max_tokens：32
用途：commit message / 简短总结
```

---

## 五、推荐 Prompt 设计（小模型友好）

### ❌ 不要这样

> “根据以下代码 diff，分析业务逻辑变化并生成高质量提交信息”

### ✅ 正确示例

```text
你是 Git 提交信息生成助手。

根据下面的代码变更，总结一个简短的提交信息。

要求：
- 使用中文
- 格式：<type>: <description>
- description 不超过 20 个字
- 只输出 commit message
```
---

## 六、为什么之前会“卡 10 分钟”

不是模型慢，而是：

- prompt 过大
- 无上下文限制
- 无 token 上限
- CPU 单线程
- 用 Coder 模型干总结任务

👉 **这是工程问题，不是 AI 问题**

---

## 七、工程原则（给 AI / Cursor 看）

> **本地小模型 ≠ 云端大模型**  
> **必须用工程手段约束模型，而不是指望模型“自己想明白”**

---

## 八、最终结论（一句话版）

> 本项目使用 GGUF + Candle，本地推理 **不需要 tokenizer.json**。  
> 模型选用 `qwen2.5-0.5b-instruct-q4_k_m.gguf`，  /Users/robert/.cyber-zen/models/qwen2.5-0.5b-instruct
> 并通过 **限制 prompt、token、diff 行数** 保证性能与稳定性。
