#!/bin/bash
set -e

# 一键构建和打包 Universal Binary DMG
# 使用方法: ./build_and_package_universal.sh [版本号]

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

APP_NAME="AppFlowy"
APP_VERSION="${1:-0.9.9}"
DMG_NAME="${APP_NAME}-${APP_VERSION}-macos-universal"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   构建 Universal Binary DMG (Intel + ARM)${NC}"
echo -e "${CYAN}======================================================${NC}"
echo -e "${GREEN}应用: ${APP_NAME}${NC}"
echo -e "${GREEN}版本: ${APP_VERSION}${NC}"
echo -e "${GREEN}架构: x86_64 + arm64${NC}"
echo ""

# 切换到项目根目录
cd "${PROJECT_ROOT}"

# ====================================
# 步骤 1: 使用官方构建脚本构建应用
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 1/2: 构建 Universal Binary 应用${NC}"
echo -e "${CYAN}======================================================${NC}"

if [ ! -f "scripts/flutter_release_build/build_universal_package_for_macos.sh" ]; then
    echo -e "${RED}错误: 找不到构建脚本${NC}"
    exit 1
fi

echo -e "${BLUE}运行 Universal Binary 构建脚本...${NC}"
bash scripts/flutter_release_build/build_universal_package_for_macos.sh "${APP_VERSION}"

# 检查构建是否成功
BUILD_DIR="appflowy_flutter/product/${APP_VERSION}/macos/Release"
if [ ! -d "${BUILD_DIR}/${APP_NAME}.app" ]; then
    echo -e "${RED}错误: 构建失败，未找到应用${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Universal Binary 构建完成${NC}"

# 验证应用架构
APP_BINARY="${BUILD_DIR}/${APP_NAME}.app/Contents/MacOS/${APP_NAME}"
if [ -f "${APP_BINARY}" ]; then
    echo -e "${BLUE}验证应用架构...${NC}"
    APP_ARCHS=$(lipo -archs "${APP_BINARY}" 2>/dev/null || echo "无法检测")
    echo -e "${GREEN}应用支持的架构: ${APP_ARCHS}${NC}"
    
    if [[ ! "$APP_ARCHS" =~ "x86_64" ]] || [[ ! "$APP_ARCHS" =~ "arm64" ]]; then
        echo -e "${YELLOW}警告: 应用可能不是 Universal Binary${NC}"
    fi
fi

echo ""

# ====================================
# 步骤 2: 打包成 DMG
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 2/2: 创建 DMG 安装包${NC}"
echo -e "${CYAN}======================================================${NC}"

OUTPUT_DIR="${PROJECT_ROOT}/dist"
TEMP_DIR="${OUTPUT_DIR}/temp_dmg"
DMG_ASSETS_DIR="${SCRIPT_DIR}/dmg_assets"
SOURCE_APP="${BUILD_DIR}/${APP_NAME}.app"

# 创建输出目录
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TEMP_DIR}"

echo -e "${BLUE}准备 DMG 内容...${NC}"

# 复制应用
echo -e "${BLUE}复制应用文件...${NC}"
cp -R "${SOURCE_APP}" "${TEMP_DIR}/"

# 创建 Applications 快捷方式
ln -sf /Applications "${TEMP_DIR}/Applications"

# 复制背景图片
if [ -f "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" ]; then
    mkdir -p "${TEMP_DIR}/.background"
    cp "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" "${TEMP_DIR}/.background/"
    echo -e "${GREEN}✓ 已添加自定义背景${NC}"
fi

# 删除旧的 DMG
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

hdiutil create -volname "${APP_NAME}" \
    -srcfolder "${TEMP_DIR}" \
    -ov -format UDRW \
    "${TEMP_DMG}"

echo -e "${BLUE}配置 DMG 外观...${NC}"

# 挂载临时 DMG
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "${TEMP_DMG}" | egrep '^/dev/' | sed 1q | awk '{print $3}')
sleep 2

# 设置窗口布局
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
' | osascript 2>/dev/null || echo -e "${YELLOW}警告: 无法设置 DMG 外观${NC}"

sync

# 卸载（多次尝试）
echo -e "${BLUE}卸载临时镜像...${NC}"
for i in {1..5}; do
    if hdiutil detach "${MOUNT_DIR}" -force 2>/dev/null; then
        echo -e "${GREEN}✓ 成功卸载${NC}"
        break
    fi
    echo -e "${YELLOW}重试卸载 ($i/5)...${NC}"
    sleep 2
done

sleep 3

# 确保镜像已经卸载
if mount | grep -q "${MOUNT_DIR}" 2>/dev/null; then
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
        echo -e "${YELLOW}转换失败，使用未压缩版本...${NC}"
        mv "${TEMP_DMG}" "${OUTPUT_DIR}/${DMG_NAME}.dmg"
    }

# 清理临时文件
echo -e "${BLUE}清理临时文件...${NC}"
rm -rf "${TEMP_DIR}"
rm -f "${TEMP_DMG}"

# 获取文件大小
APP_SIZE=$(du -sh "${SOURCE_APP}" | cut -f1)
DMG_SIZE=$(du -sh "${OUTPUT_DIR}/${DMG_NAME}.dmg" | cut -f1)

# 显示最终结果
echo ""
echo -e "${GREEN}======================================================${NC}"
echo -e "${GREEN}   ✓ Universal Binary DMG 创建成功！${NC}"
echo -e "${GREEN}======================================================${NC}"
echo -e "${GREEN}文件位置: ${OUTPUT_DIR}/${DMG_NAME}.dmg${NC}"
echo -e "${GREEN}应用大小: ${APP_SIZE}${NC}"
echo -e "${GREEN}DMG 大小: ${DMG_SIZE}${NC}"
echo -e "${GREEN}支持架构: Intel (x86_64) + Apple Silicon (ARM64)${NC}"
echo ""
echo -e "${CYAN}安装说明:${NC}"
echo -e "1. 双击 ${DMG_NAME}.dmg 打开"
echo -e "2. 将 ${APP_NAME}.app 拖到 Applications 文件夹"
echo -e "3. 应用可在 Intel 和 Apple Silicon Mac 上运行"
echo ""

# 验证命令
echo -e "${CYAN}验证架构命令:${NC}"
echo -e "hdiutil attach ${OUTPUT_DIR}/${DMG_NAME}.dmg"
echo -e "lipo -archs /Volumes/${APP_NAME}/${APP_NAME}.app/Contents/MacOS/${APP_NAME}"
echo ""

