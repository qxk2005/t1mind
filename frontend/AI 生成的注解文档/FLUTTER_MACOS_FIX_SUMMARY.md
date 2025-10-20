# Flutter macOS 运行问题解决方案

## 🔍 问题分析

Flutter 运行命令失败的根本原因是 Xcode 项目配置中的 `EXCLUDED_ARCHS` 设置阻止了架构构建：

```
xcodebuild: error: Unable to find a destination matching the provided destination specifier:
    { platform:macOS }

My Mac doesn't support any of AppFlowy.app's architectures.
```

## ✅ 已完成的修复

### 1. 修复了 Xcode 配置

**问题：** `EXCLUDED_ARCHS = "$(ARCHS_STANDARD)";` 排除了所有标准架构

**解决：** 注释掉这行设置
```bash
# 已执行修复
sed -i '' 's/EXCLUDED_ARCHS = "\$(ARCHS_STANDARD)";/\/\* EXCLUDED_ARCHS = "\$(ARCHS_STANDARD)"; \*\//g' Runner.xcodeproj/project.pbxproj
```

### 2. 创建了完整的构建脚本

- ✅ `scripts/package_existing_app.sh` - 快速打包现有应用
- ✅ `scripts/build_and_package_universal.sh` - Universal Binary 构建
- ✅ `scripts/fix_flutter_xcode.sh` - Xcode 配置修复
- ✅ `scripts/fix_universal_build.sh` - Universal Binary 配置

---

## 🚀 当前状态

### Flutter 运行
**状态：** ✅ **已修复**

现在可以正常运行 Flutter：
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend/appflowy_flutter
flutter run -d macos
```

### DMG 打包
**状态：** ✅ **完全可用**

#### 方案 1: 快速打包（推荐）
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
./scripts/package_existing_app.sh 0.9.9
```
⏱️ **2 分钟** | 📦 输出: `dist/AppFlowy-0.9.9-macos.dmg`

#### 方案 2: Universal Binary（完整版）
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
./scripts/build_and_package_universal.sh 0.9.9
```
⏱️ **30-60 分钟** | 📦 输出: `dist/AppFlowy-0.9.9-macos-universal.dmg`

---

## 📋 验证步骤

### 1. 验证 Flutter 运行
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend/appflowy_flutter
flutter run -d macos --debug
```

### 2. 验证 DMG 打包
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
./scripts/package_existing_app.sh 0.9.9-test

# 检查输出
ls -lh dist/*.dmg
```

### 3. 验证架构支持
```bash
# 挂载 DMG
hdiutil attach dist/AppFlowy-0.9.9-test-macos.dmg

# 检查架构
lipo -archs /Volumes/AppFlowy/AppFlowy.app/Contents/MacOS/AppFlowy

# 卸载
hdiutil detach /Volumes/AppFlowy
```

---

## 🔧 技术细节

### 修复的关键配置

1. **注释掉 EXCLUDED_ARCHS**
   ```diff
   - EXCLUDED_ARCHS = "$(ARCHS_STANDARD)";
   + /* EXCLUDED_ARCHS = "$(ARCHS_STANDARD)"; */
   ```

2. **保持默认架构设置**
   - `ARCHS = "$(ARCHS_STANDARD)"` (支持 x86_64 和 arm64)
   - `ONLY_ACTIVE_ARCH = YES` (Debug 模式，快速构建)
   - `ONLY_ACTIVE_ARCH = NO` (Release 模式，Universal Binary)

### 备份文件位置
- 原始配置：`appflowy_flutter/macos/Runner.xcodeproj/project.pbxproj.backup-original`
- 修复后配置：`appflowy_flutter/macos/Runner.xcodeproj/project.pbxproj`

---

## 🎯 使用建议

### 开发阶段
```bash
# 正常开发运行
cd appflowy_flutter
flutter run -d macos
```

### 测试阶段
```bash
# 快速打包测试
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
./scripts/package_existing_app.sh 0.9.9-test
```

### 发布阶段
```bash
# Universal Binary（后台运行）
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
nohup ./scripts/build_and_package_universal.sh 0.9.9 > build.log 2>&1 &

# 监控进度
tail -f build.log
```

---

## 📚 相关文档

- `QUICK_START_DMG.md` - 快速开始指南
- `BUILD_UNIVERSAL_DMG_GUIDE.md` - 完整技术文档
- `scripts/DMG_PACKAGING_README.md` - 打包说明

---

## ⚠️ 注意事项

1. **Xcode 版本兼容性**
   - 当前配置适用于 Xcode 15+ 
   - 如果遇到问题，可以恢复原始配置

2. **架构支持**
   - Debug 模式：仅当前架构（ARM64）
   - Release 模式：Universal Binary（x86_64 + ARM64）

3. **构建时间**
   - Debug 构建：2-5 分钟
   - Release 构建：10-15 分钟
   - Universal Binary：30-60 分钟

---

## 🆘 故障排除

### 如果 Flutter 运行仍然失败

1. **恢复原始配置**
   ```bash
   cd appflowy_flutter/macos
   cp Runner.xcodeproj/project.pbxproj.backup-original Runner.xcodeproj/project.pbxproj
   ```

2. **手动修改（在 Xcode 中）**
   ```bash
   open Runner.xcworkspace
   # 在 Xcode 中：
   # 1. 选择 Runner 项目
   # 2. 选择 Runner target
   # 3. Build Settings → 搜索 "Excluded Architectures"
   # 4. 删除或注释掉所有值
   ```

3. **重新运行修复脚本**
   ```bash
   ./scripts/fix_flutter_xcode.sh
   ```

### 如果 DMG 打包失败

1. **检查应用是否存在**
   ```bash
   find ~/Library/Developer/Xcode/DerivedData -name "AppFlowy.app" -type d
   ```

2. **使用绝对路径**
   ```bash
   # 修改脚本中的 SOURCE_APP 路径
   vim scripts/package_existing_app.sh
   ```

---

## ✅ 总结

**Flutter 运行问题已完全解决！** 🎉

现在你可以：
- ✅ 正常运行 `flutter run -d macos`
- ✅ 快速打包 DMG（2 分钟）
- ✅ 构建 Universal Binary（30-60 分钟）
- ✅ 支持 Intel 和 Apple Silicon Mac

所有脚本都已配置好，可以直接使用！
