#!/bin/bash
set -e

# 快速打包现有的 AppFlowy.app 为 DMG
# 使用方法: ./package_existing_app.sh [版本号]

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

APP_NAME="AppFlowy"
APP_VERSION="${1:-0.9.9}"
DMG_NAME="${APP_NAME}-${APP_VERSION}-macos"

# 源应用路径（Flutter 构建目录）
SOURCE_APP="/Users/niuzhidao/Documents/Program/t1mind/frontend/appflowy_flutter/build/macos/Build/Products/Release/${APP_NAME}.app"

# 输出目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_ROOT}/dist"
TEMP_DIR="${OUTPUT_DIR}/temp_dmg"
DMG_ASSETS_DIR="${SCRIPT_DIR}/dmg_assets"

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}   打包 ${APP_NAME} DMG 安装包${NC}"
echo -e "${BLUE}================================================${NC}"
echo -e "${GREEN}版本: ${APP_VERSION}${NC}"
echo ""

# 检查应用是否存在
if [ ! -d "${SOURCE_APP}" ]; then
    echo -e "${RED}错误: 未找到应用 ${SOURCE_APP}${NC}"
    echo -e "${YELLOW}请确认应用路径是否正确${NC}"
    exit 1
fi

echo -e "${GREEN}✓ 找到应用: ${SOURCE_APP}${NC}"

# 获取应用大小
APP_SIZE=$(du -sh "${SOURCE_APP}" | cut -f1)
echo -e "${BLUE}应用大小: ${APP_SIZE}${NC}"

# 创建输出目录
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TEMP_DIR}"

echo -e "${BLUE}准备 DMG 内容...${NC}"

# 复制应用到临时目录
echo -e "${BLUE}复制应用文件...${NC}"
cp -R "${SOURCE_APP}" "${TEMP_DIR}/"

# 创建 Applications 快捷方式
ln -sf /Applications "${TEMP_DIR}/Applications"

# 如果存在自定义背景，复制它
if [ -f "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" ]; then
    mkdir -p "${TEMP_DIR}/.background"
    cp "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" "${TEMP_DIR}/.background/"
    echo -e "${GREEN}✓ 已添加自定义背景${NC}"
fi

# 删除旧的 DMG（如果存在）
if [ -f "${OUTPUT_DIR}/${DMG_NAME}.dmg" ]; then
    echo -e "${YELLOW}删除旧的 DMG 文件...${NC}"
    rm -f "${OUTPUT_DIR}/${DMG_NAME}.dmg"
fi

echo -e "${BLUE}创建 DMG 镜像...${NC}"

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

echo -e "${BLUE}配置 DMG 外观...${NC}"

# 挂载临时 DMG 以设置视图选项
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "${TEMP_DMG}" | egrep '^/dev/' | sed 1q | awk '{print $3}')

# 等待挂载完成
sleep 2

# 使用 AppleScript 设置窗口位置和图标位置
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
' | osascript 2>/dev/null || echo -e "${YELLOW}警告: 无法设置 DMG 外观，但这不影响功能${NC}"

# 确保更改已写入
sync

# 卸载（尝试多次）
echo -e "${BLUE}卸载临时镜像...${NC}"
for i in {1..5}; do
    if hdiutil detach "${MOUNT_DIR}" -force 2>/dev/null; then
        echo -e "${GREEN}✓ 成功卸载${NC}"
        break
    fi
    echo -e "${YELLOW}重试卸载 ($i/5)...${NC}"
    sleep 2
done

# 等待系统释放资源
sleep 3

# 确保镜像已经卸载
if mount | grep -q "${MOUNT_DIR}"; then
    echo -e "${RED}警告: 镜像仍然挂载，强制卸载...${NC}"
    diskutil unmount force "${MOUNT_DIR}" 2>/dev/null || true
    sleep 2
fi

# 转换为压缩的只读 DMG
echo -e "${BLUE}压缩 DMG...${NC}"
hdiutil convert "${TEMP_DMG}" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "${OUTPUT_DIR}/${DMG_NAME}.dmg" 2>&1 || {
        echo -e "${RED}转换失败，尝试替代方案...${NC}"
        # 如果转换失败，直接使用临时 DMG
        mv "${TEMP_DMG}" "${OUTPUT_DIR}/${DMG_NAME}.dmg"
    }

# 清理临时文件
echo -e "${BLUE}清理临时文件...${NC}"
rm -rf "${TEMP_DIR}"
rm -f "${TEMP_DMG}"

# 获取最终 DMG 大小
DMG_SIZE=$(du -sh "${OUTPUT_DIR}/${DMG_NAME}.dmg" | cut -f1)

# 显示结果
echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}   ✓ DMG 创建成功！${NC}"
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}文件位置: ${OUTPUT_DIR}/${DMG_NAME}.dmg${NC}"
echo -e "${GREEN}原始应用: ${APP_SIZE}${NC}"
echo -e "${GREEN}DMG 大小: ${DMG_SIZE}${NC}"
echo ""
echo -e "${BLUE}安装说明:${NC}"
echo -e "1. 双击 ${DMG_NAME}.dmg 打开"
echo -e "2. 将 ${APP_NAME}.app 拖到 Applications 文件夹"
echo -e "3. 从 Applications 启动 ${APP_NAME}"
echo ""

