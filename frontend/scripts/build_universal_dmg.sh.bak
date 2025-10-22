#!/bin/bash
set -e

# 构建 Universal Binary (Intel + ARM) 并打包 DMG
# 使用方法: ./build_universal_dmg.sh [版本号]

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

# 检查必要的工具
echo -e "${BLUE}检查构建工具...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}错误: 未找到 cargo，请先安装 Rust${NC}"
    exit 1
fi

if ! command -v flutter &> /dev/null; then
    echo -e "${RED}错误: 未找到 flutter，请先安装 Flutter${NC}"
    exit 1
fi

if ! command -v cargo-make &> /dev/null; then
    echo -e "${YELLOW}未找到 cargo-make，正在安装...${NC}"
    cargo install cargo-make
fi

echo -e "${GREEN}✓ 所有构建工具已就绪${NC}"
echo ""

# 切换到项目根目录
cd "${PROJECT_ROOT}"

# 设置环境变量
export APP_VERSION="${APP_VERSION}"

# ====================================
# 步骤 1: 构建 x86_64 架构的 Rust 库
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 1/5: 构建 x86_64 Rust 库${NC}"
echo -e "${CYAN}======================================================${NC}"

echo -e "${BLUE}编译 x86_64-apple-darwin 目标...${NC}"
cargo make --profile production-mac-x86_64 appflowy-core-release

if [ ! -f "rust-lib/target/x86_64-apple-darwin/release/libdart_ffi.a" ]; then
    echo -e "${RED}错误: x86_64 构建失败${NC}"
    exit 1
fi
echo -e "${GREEN}✓ x86_64 架构构建完成${NC}"
echo ""

# ====================================
# 步骤 2: 构建 ARM64 架构的 Rust 库
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 2/5: 构建 ARM64 Rust 库${NC}"
echo -e "${CYAN}======================================================${NC}"

echo -e "${BLUE}编译 aarch64-apple-darwin 目标...${NC}"
cargo make --profile production-mac-arm64 appflowy-core-release

if [ ! -f "rust-lib/target/aarch64-apple-darwin/release/libdart_ffi.a" ]; then
    echo -e "${RED}错误: ARM64 构建失败${NC}"
    exit 1
fi
echo -e "${GREEN}✓ ARM64 架构构建完成${NC}"
echo ""

# ====================================
# 步骤 3: 创建 Universal Binary
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 3/5: 创建 Universal Binary${NC}"
echo -e "${CYAN}======================================================${NC}"

echo -e "${BLUE}合并 x86_64 和 ARM64 库...${NC}"
lipo -create \
    rust-lib/target/x86_64-apple-darwin/release/libdart_ffi.a \
    rust-lib/target/aarch64-apple-darwin/release/libdart_ffi.a \
    -output rust-lib/target/libdart_ffi.a

# 验证 Universal Binary
echo -e "${BLUE}验证 Universal Binary 架构...${NC}"
ARCHS=$(lipo -archs rust-lib/target/libdart_ffi.a)
echo -e "${GREEN}支持的架构: ${ARCHS}${NC}"

if [[ ! "$ARCHS" =~ "x86_64" ]] || [[ ! "$ARCHS" =~ "arm64" ]]; then
    echo -e "${RED}错误: Universal Binary 创建失败${NC}"
    exit 1
fi

# 复制到 Flutter 后端包
echo -e "${BLUE}复制到 AppFlowy Backend 包...${NC}"
cp -f rust-lib/target/libdart_ffi.a \
    appflowy_flutter/packages/appflowy_backend/macos/libdart_ffi.a

echo -e "${GREEN}✓ Universal Binary 创建完成${NC}"
echo ""

# ====================================
# 步骤 4: 构建 Flutter 应用
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 4/5: 构建 Flutter 应用${NC}"
echo -e "${CYAN}======================================================${NC}"

cd appflowy_flutter

echo -e "${BLUE}清理并获取依赖...${NC}"
flutter clean
flutter pub get

echo -e "${BLUE}运行代码生成...${NC}"
cd ..
./scripts/code_generation/generate.sh

cd appflowy_flutter

echo -e "${BLUE}构建 macOS 应用 (Universal Binary)...${NC}"
# 使用 --target-platform 指定构建 Universal Binary
flutter build macos --release --build-name="${APP_VERSION}" --dart-define=FLUTTER_BUILD_MODE=release

BUILD_DIR="build/macos/Build/Products/Release"
if [ ! -d "${BUILD_DIR}/${APP_NAME}.app" ]; then
    echo -e "${RED}错误: Flutter 应用构建失败${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Flutter 应用构建完成${NC}"

# 验证应用架构
echo -e "${BLUE}验证应用架构...${NC}"
APP_BINARY="${BUILD_DIR}/${APP_NAME}.app/Contents/MacOS/${APP_NAME}"
if [ -f "${APP_BINARY}" ]; then
    APP_ARCHS=$(lipo -archs "${APP_BINARY}" 2>/dev/null || echo "无法检测")
    echo -e "${GREEN}应用支持的架构: ${APP_ARCHS}${NC}"
fi

cd ..
echo ""

# ====================================
# 步骤 5: 创建 DMG 安装包
# ====================================
echo -e "${CYAN}======================================================${NC}"
echo -e "${CYAN}   步骤 5/5: 创建 DMG 安装包${NC}"
echo -e "${CYAN}======================================================${NC}"

OUTPUT_DIR="${PROJECT_ROOT}/dist"
TEMP_DIR="${OUTPUT_DIR}/temp_dmg"
DMG_ASSETS_DIR="${SCRIPT_DIR}/dmg_assets"
SOURCE_APP="${PROJECT_ROOT}/appflowy_flutter/build/macos/Build/Products/Release/${APP_NAME}.app"

# 创建输出目录
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TEMP_DIR}"

echo -e "${BLUE}准备 DMG 内容...${NC}"

# 复制应用
cp -R "${SOURCE_APP}" "${TEMP_DIR}/"

# 创建 Applications 快捷方式
ln -sf /Applications "${TEMP_DIR}/Applications"

# 复制背景图片
if [ -f "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" ]; then
    mkdir -p "${TEMP_DIR}/.background"
    cp "${DMG_ASSETS_DIR}/AppFlowyInstallerBackground.jpg" "${TEMP_DIR}/.background/"
fi

# 删除旧的 DMG
if [ -f "${OUTPUT_DIR}/${DMG_NAME}.dmg" ]; then
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
        break
    fi
    sleep 2
done

sleep 3

# 转换为压缩的只读 DMG
echo -e "${BLUE}压缩 DMG...${NC}"
hdiutil convert "${TEMP_DMG}" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "${OUTPUT_DIR}/${DMG_NAME}.dmg" 2>&1 || {
        echo -e "${YELLOW}使用未压缩版本...${NC}"
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
echo -e "${CYAN}验证命令:${NC}"
echo -e "lipo -archs ${OUTPUT_DIR}/${DMG_NAME}.dmg"
echo ""

