#!/bin/bash
set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认值
APP_NAME="AppFlowy"
APP_VERSION="${1:-0.9.9}"
DMG_NAME="${APP_NAME}-${APP_VERSION}-macos"
BUILD_TYPE="${2:-Release}"  # Debug 或 Release

# 目录设置
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FLUTTER_DIR="${PROJECT_ROOT}/appflowy_flutter"
BUILD_DIR="${FLUTTER_DIR}/build/macos/Build/Products/${BUILD_TYPE}"
DMG_ASSETS_DIR="${SCRIPT_DIR}/dmg_assets"
OUTPUT_DIR="${PROJECT_ROOT}/dist"
TEMP_DIR="${OUTPUT_DIR}/temp_dmg"

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}   创建 ${APP_NAME} DMG 安装包${NC}"
echo -e "${BLUE}================================================${NC}"
echo -e "${GREEN}版本: ${APP_VERSION}${NC}"
echo -e "${GREEN}构建类型: ${BUILD_TYPE}${NC}"
echo ""

# 检查是否已经构建
if [ ! -d "${BUILD_DIR}/${APP_NAME}.app" ]; then
    echo -e "${YELLOW}未找到编译好的应用，开始构建...${NC}"
    
    # 切换到 Flutter 目录
    cd "${FLUTTER_DIR}"
    
    # 清理并获取依赖
    echo -e "${BLUE}获取 Flutter 依赖...${NC}"
    flutter clean
    flutter pub get
    
    # 构建应用
    echo -e "${BLUE}构建 macOS 应用...${NC}"
    if [ "${BUILD_TYPE}" == "Release" ]; then
        flutter build macos --release --build-name="${APP_VERSION}"
    else
        flutter build macos --debug --build-name="${APP_VERSION}"
    fi
    
    # 检查构建是否成功
    if [ ! -d "${BUILD_DIR}/${APP_NAME}.app" ]; then
        echo -e "${RED}错误: 构建失败，未找到 ${APP_NAME}.app${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ 应用构建成功${NC}"
else
    echo -e "${GREEN}✓ 找到已编译的应用${NC}"
fi

# 创建输出目录
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TEMP_DIR}"

echo -e "${BLUE}准备 DMG 内容...${NC}"

# 复制应用到临时目录
cp -R "${BUILD_DIR}/${APP_NAME}.app" "${TEMP_DIR}/"

# 创建 Applications 快捷方式
ln -sf /Applications "${TEMP_DIR}/Applications"

# 如果存在自定义背景，复制它
if [ -f "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" ]; then
    mkdir -p "${TEMP_DIR}/.background"
    cp "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" "${TEMP_DIR}/.background/"
fi

echo -e "${BLUE}创建 DMG 镜像...${NC}"

# 删除旧的 DMG（如果存在）
if [ -f "${OUTPUT_DIR}/${DMG_NAME}.dmg" ]; then
    rm -f "${OUTPUT_DIR}/${DMG_NAME}.dmg"
fi

# 创建临时 DMG
TEMP_DMG="${OUTPUT_DIR}/temp_${DMG_NAME}.dmg"
if [ -f "${TEMP_DMG}" ]; then
    rm -f "${TEMP_DMG}"
fi

# 使用 hdiutil 创建 DMG
hdiutil create -volname "${APP_NAME}" \
    -srcfolder "${TEMP_DIR}" \
    -ov -format UDRW \
    "${TEMP_DMG}"

# 挂载临时 DMG 以设置视图选项
echo -e "${BLUE}配置 DMG 外观...${NC}"
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "${TEMP_DMG}" | egrep '^/dev/' | sed 1q | awk '{print $3}')

# 等待挂载完成
sleep 2

# 设置窗口位置和图标位置
echo '
   tell application "Finder"
     tell disk "'${APP_NAME}'"
           open
           set current view of container window to icon view
           set toolbar visible of container window to false
           set statusbar visible of container window to false
           set the bounds of container window to {400, 100, 920, 440}
           set viewOptions to the icon view options of container window
           set arrangement of viewOptions to not arranged
           set icon size of viewOptions to 72
           set position of item "'${APP_NAME}'.app" of container window to {130, 180}
           set position of item "Applications" of container window to {390, 180}
           if exists file ".background:AppFlowyInstallerBackground.jpg" then
               set background picture of viewOptions to file ".background:AppFlowyInstallerBackground.jpg"
           end if
           close
           open
           update without registering applications
           delay 2
     end tell
   end tell
' | osascript || true

# 确保更改已写入
sync

# 卸载
hdiutil detach "${MOUNT_DIR}" -force || true
sleep 2

# 转换为压缩的只读 DMG
echo -e "${BLUE}压缩 DMG...${NC}"
hdiutil convert "${TEMP_DMG}" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "${OUTPUT_DIR}/${DMG_NAME}.dmg"

# 清理临时文件
echo -e "${BLUE}清理临时文件...${NC}"
rm -rf "${TEMP_DIR}"
rm -f "${TEMP_DMG}"

# 显示结果
echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   ✓ DMG 创建成功！${NC}"
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}文件位置: ${OUTPUT_DIR}/${DMG_NAME}.dmg${NC}"
echo -e "${GREEN}文件大小: $(du -h "${OUTPUT_DIR}/${DMG_NAME}.dmg" | cut -f1)${NC}"
echo ""

