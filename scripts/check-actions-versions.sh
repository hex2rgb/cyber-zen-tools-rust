#!/bin/bash

# 检查 GitHub Actions 版本脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# 检查文件中的版本
check_versions() {
    local file="$1"
    local errors=0
    
    print_info "检查文件: $file"
    
    # 检查 upload-artifact 版本
    if grep -q "actions/upload-artifact@v3" "$file"; then
        print_error "  ✗ 发现 actions/upload-artifact@v3 (需要更新到 v4)"
        errors=$((errors + 1))
    elif grep -q "actions/upload-artifact@v4" "$file"; then
        print_success "  ✓ actions/upload-artifact 版本正确 (v4)"
    fi
    
    # 检查 download-artifact 版本
    if grep -q "actions/download-artifact@v3" "$file"; then
        print_error "  ✗ 发现 actions/download-artifact@v3 (需要更新到 v4)"
        errors=$((errors + 1))
    elif grep -q "actions/download-artifact@v4" "$file"; then
        print_success "  ✓ actions/download-artifact 版本正确 (v4)"
    fi
    
    # 检查 cache 版本
    if grep -q "actions/cache@v3" "$file"; then
        print_error "  ✗ 发现 actions/cache@v3 (需要更新到 v4)"
        errors=$((errors + 1))
    elif grep -q "actions/cache@v4" "$file"; then
        print_success "  ✓ actions/cache 版本正确 (v4)"
    fi
    
    return $errors
}

# 主函数
main() {
    print_info "检查 GitHub Actions 版本..."
    echo
    
    local total_errors=0
    local workflow_files=(
        ".github/workflows/release.yml"
    )
    
    for file in "${workflow_files[@]}"; do
        if [ -f "$file" ]; then
            if ! check_versions "$file"; then
                total_errors=$((total_errors + $?))
            fi
        else
            print_warning "文件不存在: $file"
        fi
        echo
    done
    
    # 总结
    if [ $total_errors -eq 0 ]; then
        print_success "🎉 所有 GitHub Actions 版本都是最新的！"
        echo
        print_info "已更新的版本:"
        echo "  - actions/upload-artifact: v3 → v4"
        echo "  - actions/download-artifact: v3 → v4"
        echo "  - actions/cache: v3 → v4"
    else
        print_error "❌ 发现 $total_errors 个版本问题需要修复"
        exit 1
    fi
}

# 运行主函数
main "$@"

