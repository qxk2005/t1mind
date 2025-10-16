# 快速开始：打包 macOS DMG

## 📦 已完成的准备工作

✅ 创建了多个打包脚本
✅ 修复了 Xcode Universal Binary 配置
✅ 创建了完整的构建指南

---

## 🚀 快速打包（3 种方案）

### 方案 1: 打包现有 ARM64 应用（最快 - 2 分钟）

如果你已经有编译好的应用，直接打包：

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend

# 找到最新的应用
APP=$(find ~/Library/Developer/Xcode/DerivedData -name "AppFlowy.app" -type d 2>/dev/null | head -1)
echo "找到应用: $APP"

# 快速打包
./scripts/package_existing_app.sh 0.9.9
```

**输出：** `dist/AppFlowy-0.9.9-macos.dmg`

---

### 方案 2: 构建并打包 Universal Binary（完整 - 30-60 分钟）

构建同时支持 Intel 和 ARM 的版本：

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend

# 一键构建和打包（需要较长时间）
./scripts/build_and_package_universal.sh 0.9.9
```

**这个命令会：**
1. 构建 x86_64 Rust 库（约 15 分钟）
2. 构建 ARM64 Rust 库（约 15 分钟）
3. 合并成 Universal Binary（1 分钟）
4. 构建 Flutter 应用（约 5 分钟）
5. 打包成 DMG（1 分钟）

**输出：** `dist/AppFlowy-0.9.9-macos-universal.dmg`

---

### 方案 3: 仅构建 ARM64 版本（中等 - 10-15 分钟）

只为 Apple Silicon Mac 构建：

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend

# 1. 构建 ARM64 Rust 库
cargo make --profile production-mac-arm64 appflowy-core-release

# 2. 构建 Flutter 应用
cd appflowy_flutter
flutter build macos --release --build-name=0.9.9
cd ..

# 3. 打包 DMG
./scripts/package_existing_app.sh 0.9.9-arm64
```

**输出：** `dist/AppFlowy-0.9.9-arm64-macos.dmg`

---

## 📍 关键文件位置

### 脚本文件

| 脚本 | 功能 | 预计时间 |
|------|------|----------|
| `scripts/package_existing_app.sh` | 打包现有应用 | 2 分钟 |
| `scripts/build_and_package_universal.sh` | 完整 Universal Binary | 30-60 分钟 |
| `scripts/fix_universal_build.sh` | 修复 Xcode 配置 | 1 秒 ✅已完成 |

### 输出文件

- DMG 文件：`dist/*.dmg`
- 应用文件：`appflowy_flutter/build/macos/Build/Products/Release/AppFlowy.app`

---

## 🔍 验证 DMG

```bash
# 挂载 DMG
hdiutil attach dist/AppFlowy-0.9.9-macos.dmg

# 检查架构
lipo -archs /Volumes/AppFlowy/AppFlowy.app/Contents/MacOS/AppFlowy

# 卸载
hdiutil detach /Volumes/AppFlowy
```

**期望输出：**
- ARM64 版本：`arm64`
- Universal 版本：`x86_64 arm64`

---

## ⏱️ 时间估算

| 操作 | 时间 | 说明 |
|------|------|------|
| 打包现有应用 | 2 分钟 | 最快，适合测试 |
| 构建 ARM64 版本 | 10-15 分钟 | 适合 Apple Silicon Mac |
| 构建 Universal Binary | 30-60 分钟 | 完整版本，兼容性最好 |

---

## 💡 推荐工作流程

### 开发阶段（快速迭代）

```bash
# 使用现有的 Debug 构建打包测试
./scripts/package_existing_app.sh 0.9.9-dev
```

### 测试阶段（单架构）

```bash
# 构建 Release 版本（ARM64）
cd appflowy_flutter
flutter build macos --release --build-name=0.9.9
cd ..
./scripts/package_existing_app.sh 0.9.9-arm64
```

### 发布阶段（Universal Binary）

```bash
# 完整构建
./scripts/build_and_package_universal.sh 0.9.9

# 或者后台运行
nohup ./scripts/build_and_package_universal.sh 0.9.9 > build.log 2>&1 &

# 查看进度
tail -f build.log
```

---

## ❓ 常见问题

### Q: 构建太慢了怎么办？

A: 可以先打包现有的应用进行测试：
```bash
./scripts/package_existing_app.sh 0.9.9-test
```

### Q: 如何只发布 ARM64 版本？

A: 使用方案 3，或者：
```bash
cd appflowy_flutter
flutter clean
flutter build macos --release --build-name=0.9.9
cd ..
./scripts/package_existing_app.sh 0.9.9
```

### Q: Xcode 配置修复失败怎么办？

A: 手动修改：
```bash
open appflowy_flutter/macos/Runner.xcworkspace
# 在 Xcode 中设置 Architectures 为 $(ARCHS_STANDARD)
# 设置 Build Active Architecture Only 为 NO（Release 配置）
```

### Q: 如何恢复原始 Xcode 配置？

A: 备份文件在这里：
```bash
cp appflowy_flutter/macos/Runner.xcodeproj/project.pbxproj.backup \
   appflowy_flutter/macos/Runner.xcodeproj/project.pbxproj
```

---

## 📚 详细文档

- `BUILD_UNIVERSAL_DMG_GUIDE.md` - 完整构建指南
- `scripts/DMG_PACKAGING_README.md` - DMG 打包说明

---

## 🎯 下一步

选择你需要的方案：

**立即测试：**
```bash
./scripts/package_existing_app.sh 0.9.9
```

**正式发布（后台运行）：**
```bash
nohup ./scripts/build_and_package_universal.sh 0.9.9 > build.log 2>&1 &
```

**监控进度：**
```bash
tail -f build.log
```

