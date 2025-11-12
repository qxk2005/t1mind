# Marker 状态检查脚本使用说明

## 概述

`check_marker_status.sh` 是一个用于检查 marker 和 marker-pdf 组件信息以及模型下载情况的 bash 脚本。它可以帮助您在不同 macOS 机器上比对配置是否正确，诊断模型重复下载的问题。

## 功能

脚本会检查以下内容：

1. **Marker 脚本**
   - 查找 marker 脚本位置
   - 检查文件权限和可执行性
   - 显示文件大小和哈希值

2. **marker-pdf 安装**
   - 检查 pipx 是否安装
   - 检查 marker-pdf 是否通过 pipx 安装
   - 显示 Python 版本信息

3. **Python 环境**
   - 检查系统 Python3
   - 检查 pipx Python 环境

4. **模型缓存目录**
   - Hugging Face 模型缓存目录 (`$HOME/Library/Caches/huggingface`)
   - Surya OCR 模型缓存目录 (`$HOME/Library/Caches/datalab/models`)
   - 列出已下载的模型文件和大小

5. **环境变量**
   - 检查 marker 脚本中设置的环境变量
   - 检查当前环境变量设置

6. **系统依赖**
   - 检查 Homebrew 是否安装
   - 检查 marker-pdf 需要的依赖库（jpeg, libpng, freetype, openjpeg, libtiff, webp）

7. **执行测试**
   - 测试 marker 脚本是否可以正常执行

## 使用方法

### 基本用法

```bash
# 在项目根目录执行
./scripts/check_marker_status.sh

# 或者指定输出文件
./scripts/check_marker_status.sh my_report.txt
```

### 在不同机器上比对

1. **在正常工作的机器上运行：**
   ```bash
   ./scripts/check_marker_status.sh working_machine_report.txt
   ```

2. **在有问题的机器上运行：**
   ```bash
   ./scripts/check_marker_status.sh problem_machine_report.txt
   ```

3. **比对两个报告：**
   ```bash
   diff working_machine_report.txt problem_machine_report.txt
   ```

## 输出内容

脚本会生成一个详细的文本报告，包含：

- 系统信息（主机名、用户、系统版本）
- 每个检查项的详细结果
- 文件路径、大小、哈希值
- 模型文件列表和大小
- 环境变量设置
- 检查摘要和关键问题提示

## 常见问题诊断

### 问题：模型重复下载

**可能原因：**
1. 环境变量未正确设置
2. 模型缓存目录路径不一致
3. 模型文件损坏或不完整

**检查方法：**
1. 比对两个机器上的环境变量设置
2. 检查模型缓存目录路径是否一致
3. 检查模型文件大小和数量是否相同

### 问题：marker-pdf 未找到

**解决方案：**
```bash
# 安装依赖
brew install jpeg libpng freetype openjpeg libtiff webp

# 安装 pipx
brew install pipx

# 安装 marker-pdf
pipx install marker-pdf
```

### 问题：模型缓存目录为空

**说明：**
- 首次运行 marker 时会自动下载模型（约 2-3GB）
- 确保网络连接稳定
- 下载可能需要 15-30 分钟

## 报告示例

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Marker 和 marker-pdf 组件检查报告
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
生成时间: 2025-01-XX XX:XX:XX
主机名: MacBook-Pro.local
用户: username
系统: Darwin XX.X.X Darwin Kernel Version XX.X.X

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. Marker 脚本检查
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Marker 脚本找到: /path/to/marker
  绝对路径: /absolute/path/to/marker
  文件大小: 4.5K
  文件哈希 (SHA256): abc123...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
2. marker-pdf 安装检查
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ pipx 已安装: 1.2.3
  marker_single 路径: /Users/xxx/.local/pipx/venvs/marker-pdf/bin/marker_single
  Python 版本: Python 3.11.5

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
4. 模型缓存目录检查
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Hugging Face 缓存目录存在: /Users/xxx/Library/Caches/huggingface
  目录大小: 1.2G
  文件数量: 45

✓ Surya OCR 模型缓存目录存在: /Users/xxx/Library/Caches/datalab/models
  目录大小: 2.1G
  文件数量: 12
```

## 注意事项

1. **权限要求：** 脚本需要读取权限来检查文件和目录
2. **执行时间：** 完整检查可能需要几秒钟到几分钟，取决于模型文件数量
3. **网络要求：** 脚本本身不需要网络，但检查 marker 脚本执行时可能会尝试连接（如果模型未下载）
4. **输出文件：** 默认输出文件名包含时间戳，避免覆盖之前的报告

## 故障排除

如果脚本执行失败：

1. 检查脚本是否有执行权限：`chmod +x scripts/check_marker_status.sh`
2. 检查 bash 版本：`bash --version`（需要 bash 4.0+）
3. 检查系统工具：`which timeout`（macOS 可能需要安装 coreutils）

## 相关文件

- `resources/marker/marker` - Marker 包装脚本
- `rust-lib/flowy-document/src/import/marker_tool_manager.rs` - Marker 工具管理器
- `rust-lib/flowy-document/src/import/marker_pdf_converter.rs` - Marker PDF 转换器


