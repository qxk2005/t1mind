# AppFlowy Windows 运行库检测和安装指南

## 运行库要求

AppFlowy Windows 版本需要以下运行库：

### 必需运行库
1. **Microsoft Visual C++ Redistributable (x64)**
   - 版本：2015-2022
   - 下载地址：https://aka.ms/vs/17/release/vc_redist.x64.exe
   - 大小：约 25 MB

### 系统要求
- Windows 10 或更高版本
- 64位系统架构
- 至少 4GB RAM
- 至少 500MB 可用磁盘空间

## 自动检测和安装功能

安装程序现在包含以下功能：

### ✅ 运行库检测
- 自动检测系统中是否已安装 Visual C++ Redistributable
- 检查多个版本的注册表项（VS 2015-2022）

### ✅ 自动安装
- **运行库文件已包含在安装包中（约14.4MB）**
- 如果检测到缺少运行库，自动静默安装
- 使用 `/quiet /norestart` 参数进行静默安装
- 安装完成后自动清理临时文件

### ✅ 用户选择
- 用户可以选择跳过运行库安装（不推荐）
- 如果自动安装失败，提供手动下载链接

## 手动安装运行库

如果自动检测失败，可以手动安装：

1. **下载运行库**
   ```
   https://aka.ms/vs/17/release/vc_redist.x64.exe
   ```

2. **运行安装程序**
   - 双击下载的 `vc_redist.x64.exe`
   - 按照向导完成安装
   - 可能需要管理员权限

3. **验证安装**
   - 重新运行 AppFlowy 安装程序
   - 运行库检测应该通过

## 故障排除

### 问题：AppFlowy 启动时出现错误
**解决方案：**
1. 确保已安装 Visual C++ Redistributable
2. 重启计算机
3. 以管理员身份运行 AppFlowy

### 问题：安装程序检测不到运行库
**解决方案：**
1. 手动下载并安装运行库
2. 检查系统是否为 64位
3. 确保 Windows 版本支持

### 问题：下载失败
**解决方案：**
1. 检查网络连接
2. 尝试使用不同的浏览器
3. 手动从 Microsoft 官网下载

## 技术细节

### 检测方法
安装程序通过检查以下注册表项来检测运行库：
```
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\VisualStudio\15.0\VC\Runtimes\x64
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\VisualStudio\16.0\VC\Runtimes\x64
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\VisualStudio\17.0\VC\Runtimes\x64
```

### 支持的版本
- Visual Studio 2015 (v14.0)
- Visual Studio 2017 (v15.0)
- Visual Studio 2019 (v16.0)
- Visual Studio 2022 (v17.0)

## 开发者信息

如果您是开发者，可以在自己的项目中集成类似的检测功能：

```pascal
// Inno Setup 代码示例
function IsVCRedistInstalled: Boolean;
var
  Version: String;
begin
  Result := RegQueryStringValue(HKEY_LOCAL_MACHINE, 
    'SOFTWARE\Microsoft\VisualStudio\17.0\VC\Runtimes\x64', 
    'Version', Version);
end;
```
