# 如何查找 Qwen3-Instruct 模型

## 🔍 主要途径

### 1. Hugging Face（推荐）

**官方网站**：https://huggingface.co/models

#### 方法 1：直接搜索

1. 访问 https://huggingface.co/models
2. 在搜索框输入：`Qwen3-Instruct`
3. 或者在搜索框输入：`Qwen3` 然后筛选 `Instruct`

#### 方法 2：访问官方组织页面

1. 访问 Qwen 官方组织：https://huggingface.co/Qwen
2. 查找所有以 `Qwen3-` 开头，以 `-Instruct` 结尾的模型

#### 方法 3：直接访问特定模型

**推荐的 Qwen3-Instruct 模型链接**：

- **Qwen3-8B-Instruct**（推荐）：
  - https://huggingface.co/Qwen/Qwen3-8B-Instruct
  - 或者搜索：`Qwen/Qwen3-8B-Instruct`

- **Qwen3-4B-Instruct**：
  - https://huggingface.co/Qwen/Qwen3-4B-Instruct
  - 或者搜索：`Qwen/Qwen3-4B-Instruct`

- **Qwen3-1.7B-Instruct**：
  - https://huggingface.co/Qwen/Qwen3-1.7B-Instruct
  - 或者搜索：`Qwen/Qwen3-1.7B-Instruct`

- **Qwen3-0.6B-Instruct**：
  - https://huggingface.co/Qwen/Qwen3-0.6B-Instruct
  - 或者搜索：`Qwen/Qwen3-0.6B-Instruct`

---

### 2. ModelScope（中国用户推荐）

**官方网站**：https://modelscope.cn/models

#### 查找方法：

1. 访问 https://modelscope.cn/models
2. 搜索：`Qwen3-Instruct`
3. 或者访问官方组织：https://modelscope.cn/organization/qwen

**优势**：
- ✅ 国内访问速度快
- ✅ 无需翻墙
- ✅ 完整的模型仓库

---

### 3. 官方文档页面

**Qwen 官方下载页面**：
- https://qwen-3.com/zh/download
- 包含所有可用模型的列表和下载链接

---

## 📋 识别 Instruct 模型的关键特征

### 模型命名规则

Instruct 模型通常有以下特征：

1. **命名包含 `-Instruct`**：
   - ✅ `Qwen3-8B-Instruct`
   - ✅ `Qwen3-4B-Instruct`
   - ❌ `Qwen3-8B`（这是 Base 模型，不是 Instruct）

2. **模型类型标注**：
   - 在 Hugging Face 页面，会显示 "Text Generation" 或 "Instruct"

3. **模型卡片说明**：
   - Instruct 模型会有 "chat"、"instruction following" 等描述

### Base vs Instruct 区别

| 特征 | Base 模型 | Instruct 模型 |
|------|----------|--------------|
| 命名 | `Qwen3-8B` | `Qwen3-8B-Instruct` |
| 用途 | 继续训练、预训练 | 对话、指令跟随 |
| 训练 | 预训练 | 预训练 + 指令微调 |
| 适用场景 | 不推荐 | ✅ 推荐用于我们的任务 |

---

## 🔧 如何下载模型

### 方法 1：使用 huggingface-cli（推荐）

```bash
# 安装 huggingface-cli（如果还没安装）
pip install huggingface-hub

# 下载模型
huggingface-cli download Qwen/Qwen3-8B-Instruct \
  --local-dir ~/.cyber-zen/models/qwen3-8b \
  --local-dir-use-symlinks False
```

### 方法 2：使用 Git LFS

```bash
# 安装 Git LFS
git lfs install

# 克隆模型仓库
cd ~/.cyber-zen/models
git clone https://huggingface.co/Qwen/Qwen3-8B-Instruct qwen3-8b
```

### 方法 3：使用 Python 脚本

```python
from huggingface_hub import snapshot_download

snapshot_download(
    repo_id="Qwen/Qwen3-8B-Instruct",
    local_dir="~/.cyber-zen/models/qwen3-8b",
    local_dir_use_symlinks=False
)
```

---

## 📦 下载后需要哪些文件

### 必需文件

1. **config.json**：模型配置文件
   - 必须，用于加载模型架构

2. **tokenizer.json**：Tokenizer 文件
   - 必须，用于文本编码/解码

3. **model.safetensors** 或 **model-*.safetensors**：模型权重文件
   - 必须，包含模型的权重数据
   - 可能是单个文件或分片文件

### 可选文件

- `tokenizer_config.json`：Tokenizer 配置
- `generation_config.json`：生成配置
- `README.md`：模型说明文档

### 验证文件完整性

下载后，检查文件是否存在：

```bash
ls -lh ~/.cyber-zen/models/qwen3-8b/
# 应该看到：
# - config.json (几百字节到几KB)
# - tokenizer.json (几MB)
# - model.safetensors 或 model-*.safetensors (几GB到几十GB)
```

---

## 🎯 快速查找指南

### 步骤 1：访问 Hugging Face

打开浏览器，访问：https://huggingface.co/models

### 步骤 2：搜索模型

在搜索框输入以下任一关键词：
- `Qwen3-Instruct`
- `Qwen3-8B-Instruct`
- `Qwen3-4B-Instruct`

### 步骤 3：查看模型详情

点击模型卡片，进入模型详情页，查看：
- ✅ 模型大小
- ✅ 支持的格式（确认有 `safetensors`）
- ✅ 模型卡片（Model Card）了解使用方法
- ✅ Files 标签页查看所有文件

### 步骤 4：确认是 Instruct 版本

检查：
- ✅ 模型名称包含 `-Instruct`
- ✅ 模型卡片提到 "chat" 或 "instruction following"
- ✅ 有示例对话格式说明

---

## 🔗 直接访问链接汇总

### Hugging Face 官方链接

#### Qwen 组织主页
- https://huggingface.co/Qwen

#### 推荐的 Qwen3-Instruct 模型

1. **Qwen3-8B-Instruct**（最推荐）
   - https://huggingface.co/Qwen/Qwen3-8B-Instruct
   - 大小：约 16GB（FP32）或 8GB（FP16）

2. **Qwen3-4B-Instruct**（速度优先）
   - https://huggingface.co/Qwen/Qwen3-4B-Instruct
   - 大小：约 8GB（FP32）或 4GB（FP16）

3. **Qwen3-1.7B-Instruct**（轻量级）
   - https://huggingface.co/Qwen/Qwen3-1.7B-Instruct
   - 大小：约 3.4GB（FP32）或 1.7GB（FP16）

4. **Qwen3-0.6B-Instruct**（超轻量）
   - https://huggingface.co/Qwen/Qwen3-0.6B-Instruct
   - 大小：约 1.2GB（FP32）或 0.6GB（FP16）

### ModelScope 官方链接（中国）

#### Qwen 组织主页
- https://modelscope.cn/organization/qwen

#### 搜索页面
- https://modelscope.cn/models?search=Qwen3-Instruct

---

## ⚠️ 注意事项

### 1. 确认模型格式

确保模型有 **Safetensors** 格式的权重文件，因为 Candle 使用 safetensors 格式。

### 2. 检查模型大小

- 确认磁盘空间足够
- 8B 模型需要约 16GB（FP32）或 8GB（FP16/BF16）
- 4B 模型需要约 8GB（FP32）或 4GB（FP16/BF16）

### 3. 网络问题

- 如果 Hugging Face 访问慢，可以使用 ModelScope（中国用户）
- 或者使用镜像站

### 4. 许可证

检查模型许可证，确保符合使用要求。Qwen3 系列通常是 Apache 2.0 许可证。

---

## 🚀 快速开始示例

### 示例：下载 Qwen3-8B-Instruct

```bash
# 1. 创建模型目录
mkdir -p ~/.cyber-zen/models/qwen3-8b

# 2. 下载模型（使用 huggingface-cli）
huggingface-cli download Qwen/Qwen3-8B-Instruct \
  --local-dir ~/.cyber-zen/models/qwen3-8b \
  --local-dir-use-symlinks False

# 3. 验证文件
ls -lh ~/.cyber-zen/models/qwen3-8b/
```

### 示例：使用 ModelScope（中国）

```bash
# 使用 modelscope 下载
pip install modelscope
python -c "from modelscope import snapshot_download; snapshot_download('Qwen/Qwen3-8B-Instruct', cache_dir='~/.cyber-zen/models/qwen3-8b')"
```

---

## 📚 相关资源

- **Hugging Face 模型库**：https://huggingface.co/models
- **Qwen 官方文档**：https://qwen-3.com
- **ModelScope**：https://modelscope.cn
- **Candle 文档**：https://github.com/huggingface/candle

---

## 💡 提示

1. **优先使用 Instruct 版本**：对于我们的任务（生成 commit message），Instruct 版本更适合
2. **检查模型卡片**：模型详情页的 Model Card 通常包含使用示例和格式说明
3. **查看 Files**：在模型页面点击 "Files" 标签，可以看到所有可用文件
4. **关注更新**：模型可能会更新，建议定期检查是否有新版本

