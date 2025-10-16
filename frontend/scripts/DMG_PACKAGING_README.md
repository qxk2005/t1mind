# macOS DMG 打包说明

本目录包含了用于将 AppFlowy 应用打包成 macOS DMG 安装包的脚本。

## 📦 打包脚本

### 1. `package_existing_app.sh` - 快速打包现有应用（推荐）

这是最简单的方式，适用于已经通过 Xcode 或 Flutter 编译好的应用。

**使用方法：**

```bash
# 使用默认版本号 0.9.9
./scripts/package_existing_app.sh

# 指定版本号
./scripts/package_existing_app.sh 1.0.0
```

**应用路径：**
- 默认从 Xcode DerivedData 目录读取：  
  `/Users/niuzhidao/Library/Developer/Xcode/DerivedData/Runner-*/Build/Products/Debug/AppFlowy.app`

**输出位置：**
- DMG 文件会生成在：`dist/AppFlowy-{版本号}-macos.dmg`

**特性：**
- ✅ 自动添加 Applications 快捷方式
- ✅ 自动设置美观的窗口布局
- ✅ 支持自定义背景图片
- ✅ 高压缩率（600M → 163M）

---

### 2. `create_dmg.sh` - 完整构建和打包

从源码开始构建并打包应用。

**使用方法：**

```bash
# Release 版本（默认）
./scripts/create_dmg.sh 1.0.0 Release

# Debug 版本
./scripts/create_dmg.sh 1.0.0 Debug
```

**功能：**
1. 检查是否已构建应用
2. 如果没有，自动执行 Flutter 构建
3. 创建 DMG 安装包

---

### 3. `package_dmg.sh` - 简单打包

基本的打包工具，支持使用 `create-dmg` 工具。

**使用方法：**

```bash
# 从 Flutter 构建目录打包
./scripts/package_dmg.sh 1.0.0 Release
```

**可选依赖：**
```bash
# 安装 create-dmg 获得更好的效果
brew install create-dmg
```

---

## 🎨 自定义背景

DMG 安装器的背景图片位于：
```
scripts/dmg_assets/AppFlowyInstallerBackground.jpg
```

要自定义背景：
1. 替换这个文件（建议尺寸：520x340 像素）
2. 重新运行打包脚本

---

## 📝 完整构建流程

如果你想从头开始构建整个应用：

### 使用 cargo-make（推荐）

```bash
# 构建 Universal Binary（支持 Intel + Apple Silicon）
./scripts/flutter_release_build/build_universal_package_for_macos.sh 1.0.0

# 然后打包
./scripts/package_existing_app.sh 1.0.0
```

### 单独构建架构

```bash
# 构建 ARM64 版本
cargo make --profile production-mac-arm64 appflowy-macos

# 构建 x86_64 版本
cargo make --profile production-mac-x86_64 appflowy-macos

# 然后打包
./scripts/package_existing_app.sh 1.0.0
```

---

## 🚀 快速开始

**最简单的方式（应用已编译）：**

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
./scripts/package_existing_app.sh 1.0.0
```

打包完成后，DMG 文件位于：  
`dist/AppFlowy-1.0.0-macos.dmg`

---

## 📦 安装和分发

### 安装步骤：

1. 双击 `AppFlowy-{版本号}-macos.dmg`
2. 将 AppFlowy.app 拖到 Applications 文件夹
3. 从 Applications 或 Spotlight 启动 AppFlowy

### 分发说明：

生成的 DMG 文件可以直接分发给用户，无需额外依赖。

**注意事项：**
- 首次运行时，macOS 可能会显示安全提示
- 用户需要在"系统偏好设置" → "安全性与隐私"中允许应用运行
- 建议对应用进行代码签名和公证以避免安全提示

---

## 🔧 问题排查

### 问题：找不到应用文件

**解决方案：**
检查 Xcode DerivedData 路径，或修改脚本中的 `SOURCE_APP` 变量。

```bash
# 查找所有 .app 文件
find ~/Library/Developer/Xcode/DerivedData -name "*.app" -type d
```

### 问题：DMG 创建失败

**解决方案：**
1. 确保没有挂载的镜像：
   ```bash
   hdiutil detach /Volumes/AppFlowy -force
   ```

2. 清理临时文件：
   ```bash
   rm -rf dist/temp_dmg dist/temp_*.dmg
   ```

3. 重新运行脚本

### 问题：权限不足

**解决方案：**
```bash
chmod +x scripts/*.sh
```

---

## 📊 构建统计

**最近一次打包结果：**
- 原始应用大小：600 MB
- DMG 文件大小：163 MB
- 压缩率：72.8%
- 打包时间：~10 秒

---

## 🎯 版本管理

建议的版本号格式：`主版本.次版本.修订号`

例如：
- `1.0.0` - 正式发布版
- `1.0.1` - Bug 修复版
- `1.1.0` - 新功能版
- `0.9.9` - 测试版

---

## 📞 支持

如有问题，请检查：
1. Flutter 是否正确安装
2. Xcode Command Line Tools 是否安装
3. 应用是否正确编译

```bash
# 检查 Flutter
flutter doctor

# 检查 Xcode 工具
xcode-select --install
```

