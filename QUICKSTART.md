# 快速开始指南

## 5 分钟快速上手

### 1. 安装 Rust（如果还没有）

```bash
# macOS 和 Linux（使用官方安装脚本）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 进入项目目录

```bash
cd /Users/robert/SelfMine/MyProject/cyber-zen-tools/cyber-zen-tools-rust
```

### 3. 运行迁移脚本（推荐）

```bash
# 给脚本添加执行权限
chmod +x migrate.sh

# 运行迁移脚本
./migrate.sh
```

迁移脚本会自动：
- ✅ 检查 Rust 环境
- ✅ 构建项目
- ✅ 测试功能
- ✅ 安装到系统

### 4. 或手动构建

```bash
# 构建项目
cargo build --release

# 安装到系统
sudo cp target/release/cyber-zen-tools /usr/local/bin/cyber-zen
sudo chmod +x /usr/local/bin/cyber-zen
```

### 5. 验证安装

```bash
cyber-zen --version
cyber-zen status
```

## 常用命令

```bash
# Git 提交（自动生成 commit message）
cyber-zen gcm

# 压缩图片
cyber-zen compress --src "photo.jpg" --rate 0.8

# 启动静态服务器
cyber-zen server --port 3000

# 查看帮助
cyber-zen --help
```

## 下一步

- 📖 阅读 [README.md](./README.md) 了解完整功能
- 🔄 查看 [MIGRATION.md](./MIGRATION.md) 了解迁移详情
- 🛠️ 开始使用工具提升开发效率！

