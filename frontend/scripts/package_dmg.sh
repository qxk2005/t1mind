#!/bin/bash
set -e

# 快速打包 DMG（假设应用已经构建）
# 使用方法: ./package_dmg.sh [版本号] [构建类型]

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

APP_NAME="AppFlowy"
APP_VERSION="${1:-0.9.9}"
BUILD_TYPE="${2:-Release}"
DMG_NAME="${APP_NAME}-${APP_VERSION}-macos"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FLUTTER_DIR="${PROJECT_ROOT}/appflowy_flutter"
BUILD_DIR="${FLUTTER_DIR}/build/macos/Build/Products/${BUILD_TYPE}"
OUTPUT_DIR="${PROJECT_ROOT}/dist"

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}   快速打包 DMG${NC}"
echo -e "${BLUE}================================================${NC}"

# 检查应用是否存在
if [ ! -d "${BUILD_DIR}/${APP_NAME}.app" ]; then
    echo -e "${RED}错误: 未找到应用 ${BUILD_DIR}/${APP_NAME}.app${NC}"
    echo -e "${YELLOW}请先构建应用或使用 create_dmg.sh 脚本${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 找到应用: ${BUILD_DIR}/${APP_NAME}.app${NC}"

# 创建输出目录
mkdir -p "${OUTPUT_DIR}"

# 删除旧的 DMG
if [ -f "${OUTPUT_DIR}/${DMG_NAME}.dmg" ]; then
    echo -e "${YELLOW}删除旧的 DMG 文件...${NC}"
    rm -f "${OUTPUT_DIR}/${DMG_NAME}.dmg"
fi

echo -e "${BLUE}创建 DMG...${NC}"

# 使用 create-dmg 工具（如果已安装）
if command -v create-dmg &> /dev/null; then
    echo -e "${GREEN}使用 create-dmg 工具...${NC}"
    
    create-dmg \
        --volname "${APP_NAME}" \
        --volicon "${BUILD_DIR}/${APP_NAME}.app/Contents/Resources/AppIcon.icns" \
        --window-pos 200 120 \
        --window-size 520 340 \
        --icon-size 100 \
        --icon "${APP_NAME}.app" 130 180 \
        --hide-extension "${APP_NAME}.app" \
        --app-drop-link 390 180 \
        --background "${SCRIPT_DIR}/dmg_assets/AppFlowyInstallerBackground.jpg" \
        "${OUTPUT_DIR}/${DMG_NAME}.dmg" \
        "${BUILD_DIR}/${APP_NAME}.app" || {
            echo -e "${YELLOW}create-dmg 失败，使用基本方法...${NC}"
            # 基本方法作为后备
            hdiutil create -volname "${APP_NAME}" \
                -srcfolder "${BUILD_DIR}/${APP_NAME}.app" \
                -ov -format UDZO \
                -imagekey zlib-level=9 \
                "${OUTPUT_DIR}/${DMG_NAME}.dmg"
        }
else
    echo -e "${YELLOW}未找到 create-dmg 工具，使用基本方法...${NC}"
    echo -e "${BLUE}提示: 可以通过 'brew install create-dmg' 安装以获得更好的效果${NC}"
    
    # 创建临时目录
    TEMP_DIR="${OUTPUT_DIR}/temp_dmg"
    mkdir -p "${TEMP_DIR}"
    
    # 复制应用
    cp -R "${BUILD_DIR}/${APP_NAME}.app" "${TEMP_DIR}/"
    
    # 创建 Applications 链接
    ln -sf /Applications "${TEMP_DIR}/Applications"
    
    # 创建 DMG
    hdiutil create -volname "${APP_NAME}" \
        -srcfolder "${TEMP_DIR}" \
        -ov -format UDZO \
        -imagekey zlib-level=9 \
        "${OUTPUT_DIR}/${DMG_NAME}.dmg"
    
    # 清理
    rm -rf "${TEMP_DIR}"
fi

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   ✓ DMG 创建成功！${NC}"
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}文件位置: ${OUTPUT_DIR}/${DMG_NAME}.dmg${NC}"
echo -e "${GREEN}文件大小: $(du -h "${OUTPUT_DIR}/${DMG_NAME}.dmg" | cut -f1)${NC}"
echo ""

