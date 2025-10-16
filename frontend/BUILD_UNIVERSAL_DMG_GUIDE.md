# 构建 Universal Binary DMG 完整指南

## 📋 问题分析

在 Apple Silicon (M 系列)芯片的 Mac 上构建 Universal Binary (同时支持 Intel x86_64 和 Apple Silicon ARM64) 时，遇到了 Xcode 架构配置问题：

```
xcodebuild: error: Unable to find a destination matching the provided destination specifier:
	{ platform:macOS }

My Mac doesn't support any of AppFlowy.app's architectures.
```

这是因为 Xcode 默认只构建当前机器的架构（ARM64），而要构建 Universal Binary 需要特殊配置。

---

## ✅ 解决方案

### 方案 1: 修改 Xcode 项目设置（推荐）

编辑 `appflowy_flutter/macos/Runner.xcodeproj/project.pbxproj` 文件，确保包含以下架构设置：

1. 打开 Xcode 项目：
   ```bash
   open appflowy_flutter/macos/Runner.xcworkspace
   ```

2. 在 Xcode 中：
   - 选择 "Runner" 项目
   - 选择 "Runner" target
   - 进入 "Build Settings"
   - 搜索 "Architectures"
   - 设置 "Architectures" 为 "Standard Architectures (Apple Silicon, Intel)" 或直接设置为 `arm64 x86_64`
   - 确保 "Build Active Architecture Only" 在 Release 配置下设置为 `NO`

3. 保存并关闭 Xcode

---

### 方案 2: 使用命令行修改（自动化）

创建并运行以下脚本来自动修改 Xcode 配置：

```bash
#!/bin/bash
# 文件: scripts/fix_universal_build.sh

cd appflowy_flutter/macos

# 备份原始文件
cp Runner.xcodeproj/project.pbxproj Runner.xcodeproj/project.pbxproj.backup

# 修改架构设置
sed -i '' 's/ARCHS = arm64;/ARCHS = "arm64 x86_64";/g' Runner.xcodeproj/project.pbxproj
sed -i '' 's/ONLY_ACTIVE_ARCH = YES;/ONLY_ACTIVE_ARCH = NO;/g' Runner.xcodeproj/project.pbxproj

echo "✓ Xcode 配置已更新为支持 Universal Binary"
```

运行：
```bash
chmod +x scripts/fix_universal_build.sh
./scripts/fix_universal_build.sh
```

---

### 方案 3: 分别构建然后合并（当前可行方案）

由于直接构建 Universal Binary 遇到问题，可以采用以下策略：

#### 步骤 1: 在 Apple Silicon Mac 上构建 ARM64 版本

```bash
# 清理
cd appflowy_flutter
flutter clean

# 构建 ARM64
flutter build macos --release --build-name=0.9.9
```

应用位置：`appflowy_flutter/build/macos/Build/Products/Release/AppFlowy.app`

#### 步骤 2: 在 Intel Mac 上构建 x86_64 版本（或使用 Rosetta）

如果你有 Intel Mac，在上面构建 x86_64 版本。

**或者**使用 Rosetta 模式构建（在 Apple Silicon Mac 上）：

```bash
# 使用 Rosetta 运行 Flutter 构建
arch -x86_64 /bin/zsh -c "cd appflowy_flutter && flutter build macos --release --build-name=0.9.9"
```

#### 步骤 3: 合并两个架构

```bash
# 解包两个 .app 文件
# 假设你有 AppFlowy-arm64.app 和 AppFlowy-x86_64.app

# 使用 lipo 合并二进制文件
lipo -create \
    AppFlowy-arm64.app/Contents/MacOS/AppFlowy \
    AppFlowy-x86_64.app/Contents/MacOS/AppFlowy \
    -output AppFlowy-universal.app/Contents/MacOS/AppFlowy

# 验证
lipo -archs AppFlowy-universal.app/Contents/MacOS/AppFlowy
# 应该输出: x86_64 arm64
```

---

## 🚀 完整构建流程（使用官方脚本）

官方的 `build_universal_package_for_macos.sh` 脚本应该能够构建 Universal Binary，但需要先修复 Xcode 配置：

```bash
# 1. 修复 Xcode 配置（方案 2 的脚本）
./scripts/fix_universal_build.sh

# 2. 运行官方构建脚本
./scripts/flutter_release_build/build_universal_package_for_macos.sh 0.9.9

# 3. 打包 DMG
./scripts/build_and_package_universal.sh 0.9.9
```

---

## 🔍 验证 Universal Binary

### 检查 Rust 库

```bash
lipo -archs appflowy_flutter/packages/appflowy_backend/macos/libdart_ffi.a
# 应该输出: x86_64 arm64
```

### 检查 Flutter 应用

```bash
lipo -archs appflowy_flutter/build/macos/Build/Products/Release/AppFlowy.app/Contents/MacOS/AppFlowy
# 应该输出: x86_64 arm64
```

### 挂载 DMG 并检查

```bash
hdiutil attach dist/AppFlowy-0.9.9-macos-universal.dmg
lipo -archs /Volumes/AppFlowy/AppFlowy.app/Contents/MacOS/AppFlowy
# 应该输出: x86_64 arm64
hdiutil detach /Volumes/AppFlowy
```

---

## ⚠️ 常见问题

### Q1: 为什么在 M 系列芯片上构建失败？

**A:** Xcode 默认只构建当前机器的架构。需要明确指定构建多个架构。

### Q2: 能否只发布 ARM64 版本？

**A:** 可以，但 Intel Mac 用户需要通过 Rosetta 2 运行，性能会有损失。Universal Binary 提供更好的用户体验。

### Q3: Rust 库已经是 Universal Binary，为什么 Flutter 构建还是失败？

**A:** Flutter 和 Rust 是分开构建的。Rust 库可以是 Universal Binary，但 Flutter/Xcode 构建阶段也需要正确配置。

### Q4: 如何只打包 ARM64 版本？

**A:** 使用以下命令：

```bash
cd appflowy_flutter
flutter build macos --release --build-name=0.9.9

# 打包
cd ..
./scripts/package_existing_app.sh 0.9.9-arm64
```

在脚本中修改 `SOURCE_APP` 路径为实际构建的路径。

---

## 📦 当前可用的打包选项

### 选项 1: ARM64 版本（M 系列芯片专用）

```bash
# 使用现有的应用
./scripts/package_existing_app.sh 0.9.9-arm64
```

**优点：**
- 构建简单，不需要特殊配置
- 在 Apple Silicon Mac 上性能最优

**缺点：**
- Intel Mac 用户需要 Rosetta 2
- 兼容性较差

### 选项 2: Universal Binary（推荐）

需要先修复 Xcode 配置，然后：

```bash
./scripts/build_and_package_universal.sh 0.9.9
```

**优点：**
- 同时支持 Intel 和 Apple Silicon
- 用户体验最佳
- 兼容性最好

**缺点：**
- 需要修改 Xcode 配置
- 文件体积稍大（约 2 倍）
- 构建时间更长

---

## 🛠️ 下一步行动

1. **立即可用：** 打包 ARM64 版本供测试

```bash
./scripts/package_existing_app.sh 0.9.9-arm64
```

2. **长期方案：** 修复 Xcode 配置以支持 Universal Binary

```bash
# 创建修复脚本
cat > scripts/fix_universal_build.sh << 'EOF'
#!/bin/bash
set -e
cd appflowy_flutter/macos
cp Runner.xcodeproj/project.pbxproj Runner.xcodeproj/project.pbxproj.backup
# 只修改 Release 配置
awk '
/ONLY_ACTIVE_ARCH = YES;/ && in_release {
    print "				ONLY_ACTIVE_ARCH = NO;"
    next
}
/buildSettings = {/ {
    if (prev ~ /Release/) in_release=1
    else in_release=0
}
{print; prev=$0}
' Runner.xcodeproj/project.pbxproj.backup > Runner.xcodeproj/project.pbxproj
echo "✓ 已修复 Xcode 配置"
EOF

chmod +x scripts/fix_universal_build.sh
./scripts/fix_universal_build.sh

# 然后重新构建
./scripts/build_and_package_universal.sh 0.9.9
```

---

## 📚 参考资料

- [Apple 文档：构建 Universal macOS Binaries](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary)
- [Flutter macOS 构建指南](https://docs.flutter.dev/deployment/macos)
- [lipo 命令使用](https://ss64.com/osx/lipo.html)

---

## 💡 提示

- 构建 Universal Binary 需要约 30-60 分钟
- 确保有足够的磁盘空间（至少 10GB）
- 建议在构建前运行 `flutter clean` 和 `cargo clean`

