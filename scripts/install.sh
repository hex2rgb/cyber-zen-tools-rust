#!/bin/bash

# Cyber Zen Tools 安装脚本
# 从 GitHub Releases 下载预编译的二进制文件

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 变量定义
BINARY_NAME="cyber-zen"
INSTALL_DIR="/usr/local/bin"
REPO_URL="hex2rgb/cyber-zen-tools-rust"
VERSION=""

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

# 检测系统架构
detect_arch() {
    # 在 macOS 上，优先检测实际硬件架构（避免 Rosetta 2 干扰）
    if [ "$(uname -s)" = "Darwin" ]; then
        # 使用 arch 命令检测实际架构（在 Rosetta 2 下会返回实际架构）
        local actual_arch=$(arch)
        case "$actual_arch" in
            arm64) echo "aarch64" ;;
            i386) 
                # 如果是 i386，可能是 Rosetta 2，检查是否支持 ARM64
                if sysctl -n hw.optional.arm64 2>/dev/null | grep -q "1"; then
                    echo "aarch64"
                else
                    echo "x86_64"
                fi
                ;;
            x86_64) echo "x86_64" ;;
            *) echo "x86_64" ;;
        esac
    else
        # 非 macOS 系统，使用 uname -m
        case "$(uname -m)" in
            x86_64) echo "x86_64" ;;
            arm64|aarch64) echo "aarch64" ;;
            *) echo "x86_64" ;;
        esac
    fi
}

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Darwin*) echo "apple-darwin" ;;
        Linux*)
            # 检测 Linux 发行版
            if [ -f /etc/os-release ]; then
                local os_id=$(grep -E '^ID=' /etc/os-release | cut -d'=' -f2 | tr -d '"' | tr '[:upper:]' '[:lower:]')
                case "$os_id" in
                    debian) echo "debian-linux-gnu" ;;
                    ubuntu) echo "ubuntu-linux-gnu" ;;
                    *) echo "ubuntu-linux-gnu" ;;  # 默认使用 ubuntu
                esac
            else
                echo "ubuntu-linux-gnu"  # 默认使用 ubuntu
            fi
            ;;
        *) echo "ubuntu-linux-gnu" ;;
    esac
}

# 将架构映射到 release.yml 使用的格式
map_arch_for_release() {
    case "$1" in
        x86_64) echo "amd64" ;;
        aarch64) echo "arm64" ;;
        *) echo "amd64" ;;
    esac
}

# 将操作系统映射到 release.yml 使用的格式
map_os_for_release() {
    case "$1" in
        apple-darwin) echo "darwin" ;;
        debian-linux-gnu) echo "debian" ;;
        ubuntu-linux-gnu) echo "ubuntu" ;;
        *) echo "ubuntu" ;;  # 默认使用 ubuntu
    esac
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --help|-h)
                echo "用法: $0 [选项]"
                echo ""
                echo "选项:"
                echo "  --version VERSION    指定版本号 (例如: v1.0.1)"
                echo "  --help, -h           显示此帮助信息"
                echo ""
                echo "示例:"
                echo "  $0                   下载并安装最新版本"
                echo "  $0 --version v1.0.1  下载并安装指定版本"
                exit 0
                ;;
            *)
                print_error "未知参数: $1"
                exit 1
                ;;
        esac
    done
}

# 获取最新版本号
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO_URL}/releases/latest"
    local version=$(curl -s "$api_url" | grep '"tag_name"' | cut -d'"' -f4)
    if [ -z "$version" ]; then
        print_error "无法获取最新版本号"
        exit 1
    fi
    echo "$version"
}

# 从 GitHub 下载预编译的二进制文件
download_and_install() {
    local version="$1"
    local os=$(detect_os)
    local arch=$(detect_arch)
    
    print_info "检测到系统: $arch-$os"
    print_info "下载版本: $version"
    
    # 映射到 release.yml 使用的格式
    local os_name=$(map_os_for_release "$os")
    local arch_name=$(map_arch_for_release "$arch")
    
    # 构建下载 URL（匹配 release.yml 的文件命名格式）
    local download_url="https://github.com/${REPO_URL}/releases/download/${version}/cyber-zen-${os_name}-${arch_name}.tar.gz"
    
    print_info "下载地址: $download_url"
    
    # 创建临时目录
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # 下载程序
    print_info "正在下载..."
    if ! curl -L -o cyber-zen-tools.tar.gz "$download_url"; then
        print_error "下载失败，请检查版本号是否正确"
        cd - > /dev/null
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # 解压程序
    print_info "正在解压..."
    if ! tar -xzf cyber-zen-tools.tar.gz; then
        print_error "解压失败"
        cd - > /dev/null
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # 查找二进制文件（匹配 release.yml 的文件命名格式）
    local binary_file=""
    local expected_name="cyber-zen-${os_name}-${arch_name}"
    
    # 检查解压后是否创建了目录
    if [ -d "$expected_name" ]; then
        # 在目录中查找二进制文件
        if [ -f "$expected_name/$expected_name" ]; then
            binary_file="$expected_name/$expected_name"
        elif [ -f "$expected_name/cyber-zen-tools" ]; then
            binary_file="$expected_name/cyber-zen-tools"
        else
            # 在目录中查找第一个可执行文件（macOS 兼容的方式）
            for file in "$expected_name"/cyber-zen*; do
                if [ -f "$file" ] && [ -x "$file" ]; then
                    binary_file="$file"
                    break
                fi
            done
        fi
    elif [ -f "$expected_name" ]; then
        # 文件直接在当前目录
        binary_file="$expected_name"
    elif [ -f "cyber-zen-tools" ]; then
        binary_file="cyber-zen-tools"
    else
        # 在当前目录查找第一个可执行文件（macOS 兼容的方式）
        for file in ./cyber-zen*; do
            if [ -f "$file" ] && [ -x "$file" ]; then
                binary_file="$file"
                break
            fi
        done
    fi
    
    if [ -z "$binary_file" ] || [ ! -f "$binary_file" ]; then
        print_error "解压后未找到二进制文件"
        print_info "当前目录文件列表:"
        ls -la
        cd - > /dev/null
        rm -rf "$temp_dir"
        exit 1
    fi
    
    print_info "找到二进制文件: $binary_file"
    
    # 安装程序
    print_info "正在安装到 $INSTALL_DIR..."
    
    # 确保目标目录存在
    if [ ! -d "$INSTALL_DIR" ]; then
        print_warning "目标目录不存在，正在创建: $INSTALL_DIR"
        if ! sudo mkdir -p "$INSTALL_DIR"; then
            print_error "无法创建目标目录: $INSTALL_DIR"
            cd - > /dev/null
            rm -rf "$temp_dir"
            exit 1
        fi
    fi
    
    if [ ! -w "$INSTALL_DIR" ]; then
        print_warning "需要 sudo 权限安装到 $INSTALL_DIR"
        if ! sudo cp "$binary_file" "$INSTALL_DIR/$BINARY_NAME"; then
            print_error "安装失败: 无法复制文件到 $INSTALL_DIR/$BINARY_NAME"
            print_info "源文件: $binary_file"
            print_info "目标文件: $INSTALL_DIR/$BINARY_NAME"
            cd - > /dev/null
            rm -rf "$temp_dir"
            exit 1
        fi
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        if ! cp "$binary_file" "$INSTALL_DIR/$BINARY_NAME"; then
            print_error "安装失败: 无法复制文件到 $INSTALL_DIR/$BINARY_NAME"
            print_info "源文件: $binary_file"
            print_info "目标文件: $INSTALL_DIR/$BINARY_NAME"
            cd - > /dev/null
            rm -rf "$temp_dir"
            exit 1
        fi
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    # 在清理临时文件前，尝试从解压包中复制配置文件
    local configs_found=false
    if [ -d "$expected_name/configs" ]; then
        print_info "在解压包中找到配置文件目录"
        install_configs_from_dir "$expected_name/configs"
        configs_found=true
    elif [ -d "configs" ]; then
        print_info "在解压包根目录找到配置文件目录"
        install_configs_from_dir "configs"
        configs_found=true
    fi
    
    # 清理临时文件
    cd - > /dev/null
    rm -rf "$temp_dir"
    
    # 如果解压包中没有配置文件，尝试其他方法
    if [ "$configs_found" = false ]; then
        install_configs
    fi
    
    print_success "下载安装完成: $INSTALL_DIR/$BINARY_NAME"
}

# 验证安装
verify_installation() {
    print_info "验证安装..."
    
    if command -v "$BINARY_NAME" &> /dev/null; then
        print_success "✓ $BINARY_NAME 已安装"
        "$BINARY_NAME" --version
    else
        print_error "✗ $BINARY_NAME 未找到"
        exit 1
    fi
}

# 从指定目录安装配置文件
install_configs_from_dir() {
    local source_dir="$1"
    local user_config_dir="$HOME/.cyber-zen/configs"
    mkdir -p "$user_config_dir"
    
    local config_files=("file-types.toml" "categories.toml" "commit-templates.toml")
    local success_count=0
    
    print_info "从 $source_dir 复制配置文件..."
    for config_file in "${config_files[@]}"; do
        local source_file="$source_dir/$config_file"
        local target_file="$user_config_dir/$config_file"
        
        if [ -f "$source_file" ]; then
            if cp "$source_file" "$target_file"; then
                print_success "✓ $config_file 复制成功"
                ((success_count++))
            else
                print_warning "⚠ $config_file 复制失败"
            fi
        else
            print_warning "⚠ $config_file 不存在于 $source_dir"
        fi
    done
    
    if [ $success_count -eq ${#config_files[@]} ]; then
        print_success "✓ 所有配置文件安装完成: $user_config_dir"
    fi
}

# 安装配置文件
install_configs() {
    print_info "安装配置文件..."
    
    # 创建用户配置目录
    local user_config_dir="$HOME/.cyber-zen/configs"
    mkdir -p "$user_config_dir"
    
    # 尝试找到项目配置目录
    local project_config_dir=""
    
    # 方法1: 从脚本所在目录查找（如果脚本在项目目录中）
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ -d "$script_dir/../configs" ]; then
        project_config_dir="$(cd "$script_dir/../configs" && pwd)"
    # 方法2: 从当前工作目录查找
    elif [ -d "./configs" ]; then
        project_config_dir="$(cd "./configs" && pwd)"
    # 方法3: 从可执行文件目录查找（如果已安装）
    elif [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
        local exe_dir="$(dirname "$(readlink -f "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || echo "$INSTALL_DIR/$BINARY_NAME")")"
        if [ -d "$exe_dir/configs" ]; then
            project_config_dir="$exe_dir/configs"
        fi
    fi
    
    # 如果找到本地配置目录，从本地复制
    if [ -n "$project_config_dir" ] && [ -d "$project_config_dir" ]; then
        install_configs_from_dir "$project_config_dir"
    else
        # 如果本地没有，尝试从远程下载
        print_info "未找到本地配置文件，尝试从远程下载..."
        local base_url="https://raw.githubusercontent.com/hex2rgb/cyber-zen-tools-rust/main/configs"
        local config_files=("file-types.toml" "categories.toml" "commit-templates.toml")
        local success_count=0
        
        for config_file in "${config_files[@]}"; do
            local download_url="$base_url/$config_file"
            local target_file="$user_config_dir/$config_file"
            
            print_info "下载: $config_file"
            if curl -fsSL "$download_url" -o "$target_file"; then
                print_success "✓ $config_file 下载成功"
                ((success_count++))
            else
                print_warning "⚠ $config_file 下载失败"
            fi
        done
        
        if [ $success_count -eq ${#config_files[@]} ]; then
            print_success "✓ 所有配置文件安装完成: $user_config_dir"
        else
            print_warning "⚠ 部分配置文件安装失败，但程序仍可正常使用"
            print_info "已安装: $success_count/${#config_files[@]} 个配置文件"
            print_info "提示: 可以手动运行 scripts/install-configs.sh 安装配置文件"
        fi
    fi
}

# 主函数
main() {
    # 解析命令行参数
    parse_args "$@"
    
    print_info "开始安装 Cyber Zen Tools..."
    
    # 获取版本号
    if [ -z "$VERSION" ]; then
        VERSION=$(get_latest_version)
    fi
    
    # 下载并安装
    download_and_install "$VERSION"
    
    # 验证安装
    verify_installation
    
    # 安装配置文件
    install_configs
    
    print_success "🎉 安装完成！"
}

# 运行主函数
main "$@"

