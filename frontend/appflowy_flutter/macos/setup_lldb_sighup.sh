#!/bin/bash
# 配置 LLDB 忽略 SIGHUP 信号的脚本

echo "正在配置 LLDB 忽略 SIGHUP 信号..."

# 检查 ~/.lldbinit 是否存在
if [ -f ~/.lldbinit ]; then
    # 检查是否已经配置过
    if grep -q "process handle SIGHUP" ~/.lldbinit; then
        echo "✓ LLDB 配置已存在，跳过添加"
    else
        echo "" >> ~/.lldbinit
        echo "# 忽略 SIGHUP 信号，避免在 Xcode 调试时中断" >> ~/.lldbinit
        echo "process handle SIGHUP -n false -p false -s false" >> ~/.lldbinit
        echo "✓ 已添加到 ~/.lldbinit"
    fi
else
    # 创建新的 .lldbinit 文件
    cat > ~/.lldbinit << 'EOF'
# LLDB 初始化配置
# 忽略 SIGHUP 信号，避免在 Xcode 调试时中断
process handle SIGHUP -n false -p false -s false
EOF
    echo "✓ 已创建 ~/.lldbinit"
fi

echo ""
echo "配置完成！请重启 Xcode 以使配置生效。"
echo ""
echo "验证配置："
echo "  在 Xcode 的 LLDB 控制台输入: process handle SIGHUP"
echo "  应该看到: SIGHUP false false false"

