#!/bin/bash

# 定义要检查的变量列表
VARS=("PATH" "GOPATH" "GOROOT" "HOMEBREW_API_DOMAIN" "HOMEBREW_BOTTLE_DOMAIN" "HOMEBREW_PIP_INDEX_URL" "LANG" "RUSTUP_DIST_SERVER" "RUSTUP_UPDATE_ROOT")

# 定义配置文件列表（按照加载顺序）
CONFIG_FILES=(
    "/etc/zshenv"
    "$HOME/.zshenv"
    "/etc/zprofile"
    "$HOME/.zprofile"
    "/etc/zshrc"
    "$HOME/.zshrc"
    "/etc/zlogin"
    "$HOME/.zlogin"
)

# 遍历每个配置文件
for config_file in "${CONFIG_FILES[@]}"; do
    if [ -f "$config_file" ]; then
        echo "检查文件: $config_file"
        for var in "${VARS[@]}"; do
            # 搜索变量设置（包括export和设置等）
            grep -n "export.*$var\|$var=" "$config_file" | while read line; do
                echo "  $var: $line"
            done
        done
    fi
done
