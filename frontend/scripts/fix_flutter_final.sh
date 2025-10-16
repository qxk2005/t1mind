#!/bin/bash
set -e

# 最终修复 Flutter macOS 运行问题

cd "$(dirname "$0")/../appflowy_flutter/macos"

echo "🔧 最终修复 Flutter macOS 配置..."

# 备份当前配置
if [ ! -f "Runner.xcodeproj/project.pbxproj.backup-final" ]; then
    cp Runner.xcodeproj/project.pbxproj Runner.xcodeproj/project.pbxproj.backup-final
    echo "✓ 已备份当前配置"
fi

# 使用 sed 进行精确修复
echo "修复架构配置..."

# 1. 确保 EXCLUDED_ARCHS 被注释掉
sed -i '' 's/EXCLUDED_ARCHS = "\$(ARCHS_STANDARD)";/\/\* EXCLUDED_ARCHS = "\$(ARCHS_STANDARD)"; \*\//g' Runner.xcodeproj/project.pbxproj

# 2. 确保 Debug 配置使用 ONLY_ACTIVE_ARCH = YES
sed -i '' '/Debug.*buildSettings = {/,/};/ s/ONLY_ACTIVE_ARCH = false;/ONLY_ACTIVE_ARCH = YES;/g' Runner.xcodeproj/project.pbxproj

# 3. 确保 Release 配置使用 ONLY_ACTIVE_ARCH = NO（用于 Universal Binary）
sed -i '' '/Release.*buildSettings = {/,/};/ s/ONLY_ACTIVE_ARCH = YES;/ONLY_ACTIVE_ARCH = NO;/g' Runner.xcodeproj/project.pbxproj

echo "✓ 配置修复完成"

# 验证修复
echo ""
echo "验证配置..."
echo "EXCLUDED_ARCHS 状态:"
grep -c "EXCLUDED_ARCHS.*ARCHS_STANDARD" Runner.xcodeproj/project.pbxproj || echo "0 (已注释)"

echo "ONLY_ACTIVE_ARCH 设置:"
grep "ONLY_ACTIVE_ARCH" Runner.xcodeproj/project.pbxproj | head -3

echo ""
echo "✅ Flutter macOS 配置修复完成！"
echo ""
echo "现在可以："
echo "  1. 运行 Flutter: flutter run -d macos"
echo "  2. 构建 Release: flutter build macos --release"
echo "  3. 打包 DMG: ./scripts/package_existing_app.sh 0.9.9"
echo ""

