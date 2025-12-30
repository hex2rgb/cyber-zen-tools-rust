#!/bin/bash

# Cyber Zen Tools - Go 到 Rust 迁移脚本
# 此脚本帮助用户从 Go 版本迁移到 Rust 版本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "\n${CYAN}=== $1 ===${NC}\n"
}

# 检查命令是否存在
check_command() {
    if command -v "$1" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# 检查 Rust 是否安装
check_rust() {
    print_step "检查 Rust 环境"
    
    if check_command rustc && check_command cargo; then
        RUST_VERSION=$(rustc --version)
        CARGO_VERSION=$(cargo --version)
        print_success "Rust 已安装: $RUST_VERSION"
        print_success "Cargo 已安装: $CARGO_VERSION"
        return 0
    else
        print_error "Rust 未安装"
        print_info "请先安装 Rust（使用官方安装脚本）:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "  或访问: https://www.rust-lang.org/tools/install"
        return 1
    fi
}

# 构建项目
build_project() {
    print_step "构建 Rust 项目"
    
    if [ ! -f "Cargo.toml" ]; then
        print_error "未找到 Cargo.toml，请确保在项目根目录运行此脚本"
        exit 1
    fi
    
    print_info "开始构建项目（这可能需要几分钟）..."
    
    if cargo build --release; then
        print_success "项目构建成功！"
        print_info "二进制文件位置: target/release/cyber-zen-tools"
        return 0
    else
        print_error "项目构建失败"
        return 1
    fi
}

# 测试功能
test_commands() {
    print_step "测试命令功能"
    
    BINARY="target/release/cyber-zen-tools"
    
    if [ ! -f "$BINARY" ]; then
        print_error "未找到二进制文件，请先构建项目"
        return 1
    fi
    
    print_info "测试 status 命令..."
    if $BINARY status; then
        print_success "status 命令测试通过"
    else
        print_warning "status 命令测试失败（可能正常）"
    fi
    
    print_info "测试 help 命令..."
    if $BINARY --help &> /dev/null; then
        print_success "help 命令测试通过"
    else
        print_error "help 命令测试失败"
        return 1
    fi
    
    return 0
}

# 安装到系统
install_binary() {
    print_step "安装到系统"
    
    BINARY="target/release/cyber-zen-tools"
    INSTALL_PATH="/usr/local/bin/cyber-zen"
    
    if [ ! -f "$BINARY" ]; then
        print_error "未找到二进制文件，请先构建项目"
        return 1
    fi
    
    print_info "安装路径: $INSTALL_PATH"
    
    # 检查是否已安装
    if [ -f "$INSTALL_PATH" ]; then
        print_warning "检测到已安装的版本"
        read -p "是否覆盖? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "跳过安装"
            return 0
        fi
    fi
    
    # 安装
    print_info "正在安装..."
    if sudo cp "$BINARY" "$INSTALL_PATH" && sudo chmod +x "$INSTALL_PATH"; then
        print_success "安装成功！"
        print_info "现在可以使用: cyber-zen --help"
        return 0
    else
        print_error "安装失败"
        return 1
    fi
}

# 验证配置文件
check_configs() {
    print_step "检查配置文件"
    
    CONFIG_DIRS=(
        "./configs"
        "$HOME/.cyber-zen/configs"
    )
    
    CONFIG_FILES=(
        "file-types.yaml"
        "categories.yaml"
        "commit-templates.yaml"
    )
    
    found_configs=0
    
    for config_dir in "${CONFIG_DIRS[@]}"; do
        if [ -d "$config_dir" ]; then
            print_info "找到配置目录: $config_dir"
            for config_file in "${CONFIG_FILES[@]}"; do
                if [ -f "$config_dir/$config_file" ]; then
                    print_success "  ✓ $config_file"
                    found_configs=$((found_configs + 1))
                fi
            done
        fi
    done
    
    if [ $found_configs -eq 0 ]; then
        print_warning "未找到配置文件，将使用默认配置"
        print_info "可以从 Go 版本复制配置文件，或创建新的配置文件"
    else
        print_success "找到 $found_configs 个配置文件"
    fi
}

# 主函数
main() {
    echo -e "${CYAN}"
    echo "╔════════════════════════════════════════════╗"
    echo "║  Cyber Zen Tools - Go 到 Rust 迁移脚本    ║"
    echo "╚════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    # 检查是否在项目目录
    if [ ! -f "Cargo.toml" ]; then
        print_error "请在项目根目录运行此脚本"
        print_info "项目根目录应包含 Cargo.toml 文件"
        exit 1
    fi
    
    # 执行迁移步骤
    if ! check_rust; then
        print_error "Rust 环境检查失败，请先安装 Rust"
        exit 1
    fi
    
    check_configs
    
    if ! build_project; then
        print_error "构建失败，请检查错误信息"
        exit 1
    fi
    
    if ! test_commands; then
        print_warning "部分测试失败，但可以继续安装"
    fi
    
    echo
    read -p "是否安装到系统? [Y/n] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        install_binary
    else
        print_info "跳过安装"
        print_info "可以手动安装: sudo cp target/release/cyber-zen-tools /usr/local/bin/cyber-zen"
    fi
    
    echo
    print_step "迁移完成"
    print_success "🎉 Rust 版本已准备就绪！"
    echo
    print_info "下一步："
    echo "  1. 测试命令: cyber-zen status"
    echo "  2. 查看帮助: cyber-zen --help"
    echo "  3. 阅读文档: cat README.md"
    echo "  4. 查看迁移指南: cat MIGRATION.md"
    echo
}

# 运行主函数
main "$@"

