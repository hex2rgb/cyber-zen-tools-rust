#!/bin/bash

# GitHub Actions 配置验证脚本

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

# 检查文件是否存在
check_file() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ]; then
        print_success "✓ $description: $file"
        return 0
    else
        print_error "✗ $description: $file (文件不存在)"
        return 1
    fi
}

# 检查 YAML 语法
check_yaml() {
    local file="$1"
    local description="$2"
    
    if command -v python3 &> /dev/null; then
        if python3 -c "import yaml; yaml.safe_load(open('$file'))" 2>/dev/null; then
            print_success "✓ $description: YAML 语法正确"
            return 0
        else
            print_error "✗ $description: YAML 语法错误"
            return 1
        fi
    else
        print_warning "⚠ $description: 无法验证 YAML 语法 (python3 未安装)"
        return 0
    fi
}

# 检查 Rust 项目
check_rust_project() {
    if [ -f "Cargo.toml" ]; then
        print_success "✓ Rust 项目文件存在"
        
        # 检查项目名称
        local project_name=$(grep "^name" Cargo.toml | cut -d'"' -f2)
        print_info "项目名称: $project_name"
        
        # 检查 Rust 版本
        local rust_version=$(grep "^edition" Cargo.toml | cut -d'"' -f2)
        print_info "Rust Edition: $rust_version"
        
        return 0
    else
        print_error "✗ Rust 项目文件不存在"
        return 1
    fi
}

# 检查 Makefile
check_makefile() {
    if [ -f "Makefile" ]; then
        print_success "✓ Makefile 存在"
        
        # 检查关键目标
        local targets=("build" "release" "test" "clean")
        for target in "${targets[@]}"; do
            if grep -q "^$target:" Makefile; then
                print_success "  ✓ 目标 '$target' 存在"
            else
                print_warning "  ⚠ 目标 '$target' 不存在"
            fi
        done
        
        return 0
    else
        print_warning "⚠ Makefile 不存在"
        return 0
    fi
}

# 主函数
main() {
    print_info "开始验证 GitHub Actions 配置..."
    echo
    
    local errors=0
    
    # 检查 GitHub Actions 目录
    if [ ! -d ".github/workflows" ]; then
        print_error "✗ .github/workflows 目录不存在"
        errors=$((errors + 1))
    else
        print_success "✓ .github/workflows 目录存在"
    fi
    
    # 检查工作流文件
    local workflow_files=(
        ".github/workflows/release.yml"
    )
    
    for file in "${workflow_files[@]}"; do
        if check_file "$file" "工作流文件"; then
            # 检查版本
            if grep -q "actions/upload-artifact@v3\|actions/download-artifact@v3\|actions/cache@v3" "$file"; then
                print_error "  ✗ 发现过时的 actions 版本 (需要更新到 v4)"
                errors=$((errors + 1))
            else
                print_success "  ✓ actions 版本正确"
            fi
        else
            errors=$((errors + 1))
        fi
    done
    
    echo
    
    # 检查项目文件
    print_info "检查项目文件..."
    
    if ! check_rust_project; then
        errors=$((errors + 1))
    fi
    
    check_makefile
    
    if ! check_file "scripts/install.sh" "安装脚本"; then
        errors=$((errors + 1))
    fi
    
    if ! check_file "src/main.rs" "主程序入口"; then
        errors=$((errors + 1))
    fi
    
    echo
    
    # 检查文档
    print_info "检查文档..."
    
    local docs=(
        "QUICKSTART.md"
        "MIGRATION.md"
        "README.md"
    )
    
    for doc in "${docs[@]}"; do
        check_file "$doc" "文档文件"
    done
    
    echo
    
    # 总结
    if [ $errors -eq 0 ]; then
        print_success "🎉 所有检查通过！GitHub Actions 配置正确。"
        echo
        print_info "下一步："
        echo "1. 推送代码到 GitHub: git push origin main"
        echo "2. 查看 Actions 页面确认构建成功"
        echo "3. 创建标签发布版本: git tag v1.0.0 && git push origin v1.0.0"
    else
        print_error "❌ 发现 $errors 个错误，请修复后重试。"
        exit 1
    fi
}

# 运行主函数
main "$@"

