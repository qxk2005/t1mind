# AppFlowy Windows 安装程序创建指南

本目录包含了创建 AppFlowy Windows 安装程序的所有必要文件和脚本。

## 文件说明

- `appflowy.iss` - Inno Setup 安装程序脚本（支持动态版本）
- `flowy_logo.ico` - AppFlowy 应用程序图标
- `create_installer.bat` - 批处理脚本（创建 Inno Setup 安装程序）
- `create_installer.ps1` - PowerShell 脚本（创建 Inno Setup 安装程序）
- `create_installer_dynamic.bat` - 动态版本批处理脚本（推荐）
- `create_installer_dynamic.ps1` - 动态版本PowerShell脚本（推荐）
- `create_selfextracting.bat` - 创建自解压安装程序的脚本
- `create_simple_installer.ps1` - 创建简单ZIP安装程序的脚本

## 使用方法

### 方法1：使用动态版本脚本（推荐）

动态版本脚本会自动从 `pubspec.yaml` 文件中读取版本号，无需手动修改脚本。

1. **安装 Inno Setup**
   - 下载地址：https://jrsoftware.org/isinfo.php
   - 安装后确保 `iscc.exe` 在系统 PATH 中

2. **运行动态版本安装程序创建脚本**
   ```cmd
   # 使用批处理脚本（推荐）
   scripts\windows_installer\create_installer_dynamic.bat
   
   # 或使用 PowerShell 脚本（推荐）
   scripts\windows_installer\create_installer_dynamic.ps1
   ```

### 方法2：使用传统脚本

传统脚本需要手动修改版本号。

1. **安装 Inno Setup**
   - 下载地址：https://jrsoftware.org/isinfo.php
   - 安装后确保 `iscc.exe` 在系统 PATH 中

2. **运行安装程序创建脚本**
   ```cmd
   # 使用批处理脚本
   scripts\windows_installer\create_installer.bat
   
   # 或使用 PowerShell 脚本
   scripts\windows_installer\create_installer.ps1
   ```

3. **手动编译（可选）**
   ```cmd
   iscc scripts\windows_installer\appflowy.iss
   ```

### 方法3：使用自解压安装程序

1. **安装 7-Zip**
   - 下载地址：https://www.7-zip.org/
   - 确保 `7z.exe` 在系统 PATH 中

2. **运行自解压安装程序创建脚本**
   ```cmd
   scripts\windows_installer\create_selfextracting.bat
   ```

### 方法4：使用简单ZIP安装程序

使用PowerShell内置功能创建ZIP安装包：

```cmd
scripts\windows_installer\create_simple_installer.ps1
```

## 输出文件

安装程序将创建在以下位置：
```
appflowy_flutter\product\{版本号}\windows\
├── AppFlowy-Setup-{版本号}.exe (Inno Setup 安装程序)
└── AppFlowy-Portable-{版本号}.zip (便携版安装程序)
```

例如，如果版本是 `0.9.20251022`，则文件位于：
```
appflowy_flutter\product\0.9.20251022\windows\
├── AppFlowy-Setup-0.9.20251022.exe
└── AppFlowy-Portable-0.9.20251022.zip
```

## 安装程序功能

### Inno Setup 安装程序特性：
- ✅ 标准的 Windows 安装界面
- ✅ 支持中文和英文界面
- ✅ 创建桌面快捷方式（可选）
- ✅ 创建开始菜单项
- ✅ 支持卸载功能
- ✅ 注册 URL 协议处理
- ✅ 64位架构支持
- ✅ **运行库自动检测和安装引导**
- ✅ **系统兼容性检查**

### 自解压安装程序特性：
- ✅ 无需额外安装程序
- ✅ 自动解压到用户目录
- ✅ 创建桌面快捷方式
- ✅ 简单的安装过程

## 运行库要求

### 必需运行库
AppFlowy Windows 版本需要以下运行库：

1. **Microsoft Visual C++ Redistributable (x64)**
   - 版本：2015-2022
   - 下载地址：https://aka.ms/vs/17/release/vc_redist.x64.exe
   - 大小：约 25 MB

### 系统要求
- Windows 10 或更高版本
- 64位系统架构
- 至少 4GB RAM
- 至少 500MB 可用磁盘空间

### 自动检测和安装功能
安装程序包含智能运行库检测和自动安装：
- ✅ 自动检测 Visual C++ Redistributable 是否已安装
- ✅ **运行库文件已包含在安装包中（约14.4MB）**
- ✅ **如果缺少运行库，自动静默安装**
- ✅ 用户可选择跳过运行库安装（不推荐）
- ✅ 安装完成后自动清理临时文件
- ✅ 详细的故障排除指导

更多信息请参考：[运行库要求详细说明](RUNTIME_REQUIREMENTS.md)

## 动态版本功能

### 自动版本检测
动态版本脚本（`create_installer_dynamic.*`）会自动：
1. 从 `appflowy_flutter/pubspec.yaml` 文件中读取 `version` 字段
2. 设置 `APP_VERSION` 环境变量
3. 使用该版本号创建安装程序

### 版本格式
支持的版本格式：
- `0.9.20251022` (当前格式)
- `1.0.0`
- `2.1.3-beta`
- 任何符合语义版本规范的格式

### 手动设置版本
如果需要手动设置版本，可以：
1. 修改 `pubspec.yaml` 中的 `version` 字段
2. 或设置环境变量：`set APP_VERSION=1.0.0`

## 自定义配置

### 修改应用程序信息
编辑 `appflowy.iss` 文件顶部的定义：
```ini
#define AppName "AppFlowy"
#define AppVersion "0.9.20251022"
#define AppPublisher "AppFlowy-IO"
#define AppURL "https://appflowy.io"
```

### 修改安装路径
在 `appflowy.iss` 中修改：
```ini
DefaultDirName={autopf}\{#AppName}
```

### 添加许可证文件
1. 将许可证文件放在 `scripts/windows_installer/` 目录
2. 在 `appflowy.iss` 中添加：
```ini
LicenseFile=license.txt
```

## 故障排除

### 常见问题

1. **"iscc 不是内部或外部命令"**
   - 确保 Inno Setup 已正确安装
   - 将 Inno Setup 安装目录添加到系统 PATH

2. **"7z 不是内部或外部命令"**
   - 确保 7-Zip 已正确安装
   - 将 7-Zip 安装目录添加到系统 PATH

3. **构建失败**
   - 确保 AppFlowy 应用程序已成功构建
   - 检查 `appflowy_flutter\build\windows\x64\runner\Release\` 目录是否存在

### 手动检查构建
```cmd
cd appflowy_flutter
flutter build windows --release
```

## 分发说明

创建安装程序后，您可以：
1. 将安装程序上传到应用商店
2. 通过网站提供下载
3. 通过邮件或其他方式分发给用户

建议在分发前测试安装程序的功能。
