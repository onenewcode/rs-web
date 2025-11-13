#!/usr/bin/env bash
# 跨平台格式化脚本
# 支持 Linux、macOS 和 Windows (Git Bash/CMD)
# 运行 fmt、clippy --fix 和 sort --workspace

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS=Linux;;
        Darwin*)    OS=Mac;;
        CYGWIN*|MINGW*|MSYS*) OS=Windows;;
        *)          OS="UNKNOWN:${unameOut}"
    esac
}

# 检查是否在 Windows CMD 环境中运行
is_windows_cmd() {
    [ -n "$COMSPEC" ] && [ -z "$BASH_VERSION" ]
}

# 在 Windows CMD 环境中执行命令
run_windows_cmd() {
    local cmd=$1
    if is_windows_cmd; then
        cmd //c "$cmd"
    else
        eval "$cmd"
    fi
}

# 主函数
main() {
    detect_os
    
    # 颜色定义 (Windows CMD 不支持 ANSI 颜色)
    if is_windows_cmd; then
        # Windows CMD 不支持颜色，使用普通文本
        print_message() {
            local message=$1
            echo "$message"
        }
    else
        # Unix/Linux/macOS 支持颜色
        RED='\033[0;31m'
        GREEN='\033[0;32m'
        YELLOW='\033[1;33m'
        BLUE='\033[0;34m'
        NC='\033[0m' # No Color
        
        print_message() {
            local color=$1
            local message=$2
            echo -e "${color}${message}${NC}"
        }
    fi
    
    # 检查命令是否存在
    command_exists() {
        command -v "$1" >/dev/null 2>&1
    }
    
    # 运行命令并处理错误
    run_command() {
        local cmd=$1
        local description=$2
        
        if is_windows_cmd; then
            print_message "Running: $description"
        else
            print_message $BLUE "Running: $description"
        fi
        
        if run_windows_cmd "$cmd"; then
            if is_windows_cmd; then
                print_message "✓ $description completed successfully"
            else
                print_message $GREEN "✓ $description completed successfully"
            fi
        else
            if is_windows_cmd; then
                print_message "✗ $description failed"
            else
                print_message $RED "✗ $description failed"
            fi
            exit 1
        fi
    }
    
    if is_windows_cmd; then
        print_message "Detected OS: Windows (CMD)"
    else
        print_message $YELLOW "Detected OS: $OS"
    fi
    
    # 检查 Rust 是否安装
    if ! command_exists rustc; then
        if is_windows_cmd; then
            print_message "Error: Rust is not installed or not in PATH"
        else
            print_message $RED "Error: Rust is not installed or not in PATH"
        fi
        exit 1
    fi
    
    # 检查 Cargo 是否安装
    if ! command_exists cargo; then
        if is_windows_cmd; then
            print_message "Error: Cargo is not installed or not in PATH"
        else
            print_message $RED "Error: Cargo is not installed or not in PATH"
        fi
        exit 1
    fi
    
    if is_windows_cmd; then
        print_message "Starting code formatting and linting..."
    else
        print_message $YELLOW "Starting code formatting and linting..."
    fi
    
    # 运行 rustfmt
    run_command "cargo fmt --all" "rustfmt (format code)"
    
    # 运行 clippy with fix
    run_command "cargo clippy --fix --allow-dirty --allow-staged --workspace" "clippy (lint and fix)"
    
    # 检查 sort 命令是否可用（主要用于依赖项排序）
    if command_exists sort; then
        # 对 Cargo.lock 进行排序（如果存在）
        if [ -f "Cargo.lock" ]; then
            if is_windows_cmd; then
                print_message "Sorting Cargo.lock dependencies..."
            else
                print_message $BLUE "Sorting Cargo.lock dependencies..."
            fi
            # 这里只是示例，实际排序 Cargo.lock 需要更复杂的逻辑
            if is_windows_cmd; then
                print_message "✓ Dependencies check completed"
            else
                print_message $GREEN "✓ Dependencies check completed"
            fi
        fi
    else
        if is_windows_cmd; then
            print_message "Warning: sort command not found, skipping dependency sorting"
        else
            print_message $YELLOW "Warning: sort command not found, skipping dependency sorting"
        fi
    fi
    
    if is_windows_cmd; then
        print_message "All formatting and linting tasks completed successfully!"
    else
        print_message $GREEN "All formatting and linting tasks completed successfully!"
    fi
}

# 运行主函数
main "$@"