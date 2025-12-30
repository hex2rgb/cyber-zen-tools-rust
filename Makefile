# Cyben Zen Tools Makefile
# 简化版本，专注于核心功能

# 变量定义
BINARY_NAME := cyber-zen-tools
INSTALL_NAME := cyber-zen
BUILD_DIR := target
RELEASE_DIR := $(BUILD_DIR)/release
INSTALL_DIR := /usr/local/bin
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev-$(shell date +%Y%m%d-%H%M%S)")
COMMIT_HASH := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_TIME := $(shell date -u '+%Y-%m-%d_%H:%M:%S_UTC')

# 默认目标
.PHONY: help
help: ## 显示帮助信息
	@echo "Cyben Zen Tools 构建系统"
	@echo ""
	@echo "可用目标:"
	@echo "  build              - 构建程序（开发模式）"
	@echo "  release            - 构建程序（发布模式，优化）"
	@echo "  install            - 构建并安装"
	@echo "  dev                - 完整开发流程"
	@echo "  clean              - 清理构建目录"
	@echo "  test               - 运行测试"
	@echo "  uninstall          - 卸载程序"
	@echo "  install-configs    - 安装配置文件到用户目录"
	@echo "  link               - 创建软链接（推荐）"
	@echo ""
	@echo "变量:"
	@echo "  VERSION     - 版本号 (默认: git tag 或 dev-时间戳)"
	@echo "  BUILD_DIR   - 构建目录 (默认: target)"
	@echo "  INSTALL_DIR - 安装目录 (默认: /usr/local/bin)"

# 构建程序（开发模式）
.PHONY: build
build: ## 构建程序（开发模式）
	@echo "构建 $(BINARY_NAME) (开发模式)..."
	cargo build
	@echo "构建完成: $(BUILD_DIR)/debug/$(BINARY_NAME)"

# 构建程序（发布模式）
.PHONY: release
release: ## 构建程序（发布模式，优化）
	@echo "构建 $(BINARY_NAME) (发布模式)..."
	cargo build --release
	@echo "构建完成: $(RELEASE_DIR)/$(BINARY_NAME)"

# 安装
.PHONY: install
install: release ## 构建并安装
	@echo "安装到 $(INSTALL_DIR)..."
	@if [ ! -w $(INSTALL_DIR) ]; then \
		echo "需要 sudo 权限安装到 $(INSTALL_DIR)"; \
		sudo cp $(RELEASE_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_NAME); \
		sudo chmod +x $(INSTALL_DIR)/$(INSTALL_NAME); \
	else \
		cp $(RELEASE_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_NAME); \
		chmod +x $(INSTALL_DIR)/$(INSTALL_NAME); \
	fi
	@echo "安装完成: $(INSTALL_DIR)/$(INSTALL_NAME)"

# 卸载
.PHONY: uninstall
uninstall: ## 卸载程序
	@echo "从 $(INSTALL_DIR) 卸载..."
	@if [ -f $(INSTALL_DIR)/$(INSTALL_NAME) ]; then \
		if [ ! -w $(INSTALL_DIR) ]; then \
			sudo rm $(INSTALL_DIR)/$(INSTALL_NAME); \
		else \
			rm $(INSTALL_DIR)/$(INSTALL_NAME); \
		fi; \
		echo "卸载完成"; \
	else \
		echo "文件不存在: $(INSTALL_DIR)/$(INSTALL_NAME)"; \
	fi

# 清理
.PHONY: clean
clean: ## 清理构建目录
	@echo "清理构建目录..."
	cargo clean
	@echo "清理完成"

# 依赖管理
.PHONY: deps
deps: ## 更新依赖
	@echo "更新依赖..."
	cargo update
	@echo "依赖更新完成"

# 运行测试
.PHONY: test
test: ## 运行测试
	@echo "运行测试..."
	cargo test --verbose
	@echo "测试完成"

# 验证安装
.PHONY: verify
verify: ## 验证安装
	@echo "验证安装..."
	@if command -v $(INSTALL_NAME) >/dev/null 2>&1; then \
		echo "✓ $(INSTALL_NAME) 已安装"; \
		$(INSTALL_NAME) --version; \
	else \
		echo "✗ $(INSTALL_NAME) 未找到"; \
		exit 1; \
	fi
	@echo "验证完成"

# 安装配置文件
.PHONY: install-configs
install-configs: ## 安装配置文件到用户目录
	@echo "安装配置文件..."
	@if [ -f scripts/install-configs.sh ]; then \
		./scripts/install-configs.sh --user; \
	else \
		echo "错误: scripts/install-configs.sh 不存在"; \
		exit 1; \
	fi
	@echo "配置文件安装完成"

# 创建软链接
.PHONY: link
link: release ## 创建软链接（推荐，便于更新）
	@echo "创建软链接到 $(INSTALL_DIR)..."
	@if [ ! -f $(RELEASE_DIR)/$(BINARY_NAME) ]; then \
		echo "错误: 发布版本不存在，请先运行 'make release'"; \
		exit 1; \
	fi
	@if [ ! -w $(INSTALL_DIR) ]; then \
		echo "需要 sudo 权限创建软链接到 $(INSTALL_DIR)"; \
		sudo rm -f $(INSTALL_DIR)/$(INSTALL_NAME); \
		sudo ln -sf $(shell pwd)/$(RELEASE_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_NAME); \
	else \
		rm -f $(INSTALL_DIR)/$(INSTALL_NAME); \
		ln -sf $(shell pwd)/$(RELEASE_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/$(INSTALL_NAME); \
	fi
	@echo "软链接创建完成: $(INSTALL_DIR)/$(INSTALL_NAME)"
	@echo "目标文件: $(shell pwd)/$(RELEASE_DIR)/$(BINARY_NAME)"

# 完整流程
.PHONY: dev
dev: clean deps release install verify ## 完整开发流程
	@echo "构建完成！"

# 跨平台构建
.PHONY: build-all
build-all: ## 跨平台构建
	@echo "跨平台构建..."
	@mkdir -p $(BUILD_DIR)/dist
	@for target in x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-apple-darwin aarch64-unknown-linux-gnu; do \
		echo "构建 $$target..."; \
		cargo build --release --target $$target || echo "警告: $$target 构建失败（可能需要安装交叉编译工具链）"; \
	done
	@echo "跨平台构建完成"

