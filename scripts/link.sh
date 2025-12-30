#!/bin/bash

# Cyben Zen Tools 软链接创建脚本
# 简化版本，专注于创建软链接

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 变量定义
BINARY_NAME="cyber-zen-tools"
BUILD_DIR="target/release"
LINK_DIR="/usr/local/bin"
INSTALL_NAME="cyber-zen"

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

# 显示帮助信息
show_help() {
    echo "Cyben Zen Tools 软链接创建脚本"
    echo ""
    echo "用法: $0"
    echo ""
    echo "功能:"
    echo "  创建软链接到 $LINK_DIR"
    echo ""
    echo "注意:"
    echo "  - 需要先运行 'make release' 构建程序"
    echo "  - 需要 sudo 权限创建软链接"
}

# 检查构建文件
check_build_file() {
    if [ ! -f "$BUILD_DIR/$BINARY_NAME" ]; then
        print_error "构建文件不存在: $BUILD_DIR/$BINARY_NAME"
        print_info "请先运行 'make release' 构建程序"
        exit 1
    fi
}

# 创建软链接
create_link() {
    print_info "创建软链接到 $LINK_DIR..."
    
    # 检查构建文件
    check_build_file
    
    # 检查权限
    if [ ! -w "$LINK_DIR" ]; then
        print_warning "需要 sudo 权限创建软链接到 $LINK_DIR"
        print_info "请手动执行以下命令:"
        echo ""
        echo "  # 删除现有软链接（如果存在）"
        echo "  sudo rm -f $LINK_DIR/$INSTALL_NAME"
        echo ""
        echo "  # 创建新的软链接"
        echo "  sudo ln -sf $(pwd)/$BUILD_DIR/$BINARY_NAME $LINK_DIR/$INSTALL_NAME"
        echo ""
        return 1
    fi
    
    # 删除现有软链接
    if [ -L "$LINK_DIR/$INSTALL_NAME" ]; then
        print_info "删除现有软链接..."
        rm "$LINK_DIR/$INSTALL_NAME"
    fi
    
    # 创建新的软链接
    ln -sf "$(pwd)/$BUILD_DIR/$BINARY_NAME" "$LINK_DIR/$INSTALL_NAME"
    
    print_success "软链接创建完成: $LINK_DIR/$INSTALL_NAME"
    print_info "目标文件: $(pwd)/$BUILD_DIR/$BINARY_NAME"
}

# 主函数
main() {
    case "${1:-create}" in
        "create"|"")
            create_link
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "未知命令: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"

