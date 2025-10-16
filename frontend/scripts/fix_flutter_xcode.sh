#!/bin/bash
set -e

# 完全修复 Xcode 配置以支持 Flutter 运行和 Universal Binary 构建

cd "$(dirname "$0")/../appflowy_flutter/macos"

echo "🔧 完全修复 Xcode 配置..."

# 备份原始文件
if [ ! -f "Runner.xcodeproj/project.pbxproj.backup-original" ]; then
    cp Runner.xcodeproj/project.pbxproj Runner.xcodeproj/project.pbxproj.backup-original
    echo "✓ 已备份原始配置"
fi

# 使用 Python 进行精确修复
python3 << 'PYTHON_SCRIPT'
import re

# 读取文件
with open('Runner.xcodeproj/project.pbxproj', 'r') as f:
    content = f.read()

# 1. 注释掉 EXCLUDED_ARCHS 设置（这会阻止架构构建）
content = re.sub(
    r'EXCLUDED_ARCHS = "\$\(ARCHS_STANDARD\)";',
    '/* EXCLUDED_ARCHS = "$(ARCHS_STANDARD)"; */',
    content
)

# 2. 确保 ARCHS 设置为标准架构
content = re.sub(
    r'ARCHS = [^;]+;',
    'ARCHS = "$(ARCHS_STANDARD)";',
    content
)

# 3. 在 Debug 配置中设置 ONLY_ACTIVE_ARCH = YES（用于开发）
# 在 Release 配置中设置 ONLY_ACTIVE_ARCH = NO（用于 Universal Binary）

# 查找 Debug 配置并设置 ONLY_ACTIVE_ARCH = YES
debug_pattern = r'(/\* Debug \*/.*?buildSettings = \{.*?)ONLY_ACTIVE_ARCH = [^;]+;'
content = re.sub(debug_pattern, r'\1ONLY_ACTIVE_ARCH = YES;', content, flags=re.DOTALL)

# 查找 Release 配置并设置 ONLY_ACTIVE_ARCH = NO
release_pattern = r'(/\* Release \*/.*?buildSettings = \{.*?)ONLY_ACTIVE_ARCH = [^;]+;'
content = re.sub(release_pattern, r'\1ONLY_ACTIVE_ARCH = NO;', content, flags=re.DOTALL)

# 4. 确保 VALID_ARCHS 包含所需架构
if 'VALID_ARCHS' not in content:
    # 在 buildSettings 中添加 VALID_ARCHS
    content = re.sub(
        r'(buildSettings = \{)',
        r'\1\n\t\t\t\tVALID_ARCHS = "arm64 x86_64";',
        content
    )

# 写回文件
with open('Runner.xcodeproj/project.pbxproj', 'w') as f:
    f.write(content)

print("✓ 已修复 Xcode 配置")
PYTHON_SCRIPT

echo ""
echo "✅ Xcode 配置已完全修复"
echo ""
echo "修复内容："
echo "  ✓ 注释掉 EXCLUDED_ARCHS 设置"
echo "  ✓ 设置 ARCHS = \$(ARCHS_STANDARD)"
echo "  ✓ Debug: ONLY_ACTIVE_ARCH = YES (快速开发)"
echo "  ✓ Release: ONLY_ACTIVE_ARCH = NO (Universal Binary)"
echo "  ✓ 添加 VALID_ARCHS = arm64 x86_64"
echo ""
echo "现在可以："
echo "  1. 运行 Flutter: flutter run -d macos"
echo "  2. 构建 Universal Binary: ./scripts/build_and_package_universal.sh 0.9.9"
echo ""

