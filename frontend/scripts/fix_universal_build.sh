#!/bin/bash
set -e

# 修复 Xcode 项目以支持 Universal Binary 构建

cd "$(dirname "$0")/../appflowy_flutter/macos"

echo "🔧 修复 Xcode 配置以支持 Universal Binary..."

# 备份原始文件
if [ ! -f "Runner.xcodeproj/project.pbxproj.backup" ]; then
    cp Runner.xcodeproj/project.pbxproj Runner.xcodeproj/project.pbxproj.backup
    echo "✓ 已备份原始配置"
fi

# 使用 Python 脚本来精确修改 plist（更可靠）
python3 << 'PYTHON_SCRIPT'
import re

# 读取文件
with open('Runner.xcodeproj/project.pbxproj', 'r') as f:
    content = f.read()

# 在 Release 配置中设置架构
# 查找 Release 构建配置块并修改

# 修改 ONLY_ACTIVE_ARCH 为 NO（在 Release 配置中）
content = re.sub(
    r'(/\* Release \*/.*?buildSettings = \{.*?)ONLY_ACTIVE_ARCH = YES;',
    r'\1ONLY_ACTIVE_ARCH = NO;',
    content,
    flags=re.DOTALL
)

# 确保 ARCHS 包含两个架构
# 如果已经有 ARCHS 设置，替换它
content = re.sub(
    r'ARCHS = [^;]+;',
    'ARCHS = "$(ARCHS_STANDARD)";',
    content
)

# 写回文件
with open('Runner.xcodeproj/project.pbxproj', 'w') as f:
    f.write(content)

print("✓ 已修改 Xcode 配置")
PYTHON_SCRIPT

echo ""
echo "✅ Xcode 配置已更新"
echo ""
echo "修改内容："
echo "  - ONLY_ACTIVE_ARCH = NO (Release 配置)"
echo "  - ARCHS = \$(ARCHS_STANDARD) (支持 x86_64 和 arm64)"
echo ""
echo "现在可以构建 Universal Binary："
echo "  cd $(dirname "$0")/.."
echo "  ./scripts/build_and_package_universal.sh 0.9.9"
echo ""

