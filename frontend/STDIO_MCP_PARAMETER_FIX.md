# STDIO MCP 参数配置问题修复

## 🔍 问题分析

### 报错日志
```
Starting STDIO MCP process: /usr/local/bin/npx ["-y @readwise/readwise-mcp"]
Failed to initialize MCP server: Process closed stdout
```

### 问题原因

从日志可以看出参数传递错误：
- **错误**: `["-y @readwise/readwise-mcp"]` - 这是**一个参数**
- **正确**: `["-y", "@readwise/readwise-mcp"]` - 应该是**两个参数**

当用户在UI中把 `-y @readwise/readwise-mcp` 输入到一个参数框中时，整个字符串被当作一个参数传递给命令，导致npx无法识别，进程立即退出。

## ✅ 解决方案

### 1. 后端改进 - 添加stderr日志捕获

**文件**: `rust-lib/flowy-ai/src/mcp/client.rs`

**改进内容**:

#### a) 捕获stderr输出
```rust
// 在后台任务中捕获stderr输出
let server_name = self.config.name.clone();
tokio::spawn(async move {
    use tokio::io::AsyncBufReadExt;
    let mut stderr_reader = BufReader::new(stderr);
    let mut line = String::new();
    while let Ok(n) = stderr_reader.read_line(&mut line).await {
        if n == 0 { break; }
        if !line.trim().is_empty() {
            tracing::warn!("[STDIO stderr] {}: {}", server_name, line.trim());
        }
        line.clear();
    }
});
```

**好处**:
- 可以看到子进程的错误输出
- 帮助诊断参数错误、路径问题等
- 在日志中显示为 `[STDIO stderr] 服务器名: 错误信息`

#### b) 添加启动延迟和错误提示
```rust
// 给进程一点时间启动，检查是否立即退出
tokio::time::sleep(std::time::Duration::from_millis(100)).await;

// ... initialize 请求 ...

Err(e) => {
    let error_msg = format!("Failed to initialize MCP server: {}", e);
    tracing::error!("{} - Please check stderr logs above for details", error_msg);
    // ...
}
```

### 2. 前端改进 - 增强UI提示

**文件**: `workspace_mcp_settings_v2.dart`

#### a) 添加标题说明和图标提示
```dart
Row(
  children: [
    FlowyText.regular("命令参数", fontSize: 14),
    const SizedBox(width: 8),
    Tooltip(
      message: '每个参数单独添加，例如：第一个参数填"-y"，第二个参数填"@readwise/readwise-mcp"',
      child: Icon(
        Icons.info_outline,
        size: 16,
        color: Theme.of(context).hintColor,
      ),
    ),
  ],
),
```

#### b) 添加醒目的提示文本
```dart
FlowyText.regular(
  "提示：每个参数需要单独添加，不要在一个框中输入多个参数",
  fontSize: 11,
  color: Theme.of(context).hintColor,
),
```

#### c) 改进参数输入框的提示
```dart
TextField(
  decoration: InputDecoration(
    hintText: '单个参数，例如：-y 或 @readwise/readwise-mcp',
    helperText: '参数 ${index + 1}',
    helperStyle: TextStyle(fontSize: 10),
    // ...
  ),
)
```

## 📝 正确配置示例

### 示例1: Readwise MCP 服务器

**配置界面**:
```
服务器名称: readwise-mcp
命令路径: npx  (或 /usr/local/bin/npx)

命令参数:
  参数 1: -y
  参数 2: @readwise/readwise-mcp

环境变量:
  READWISE_API_KEY: your_api_key_here
```

**等价的命令行**:
```bash
npx -y @readwise/readwise-mcp
```

### 示例2: 文件系统 MCP 服务器

**配置界面**:
```
服务器名称: filesystem
命令路径: npx

命令参数:
  参数 1: -y
  参数 2: @modelcontextprotocol/server-filesystem
  参数 3: /path/to/directory
```

**等价的命令行**:
```bash
npx -y @modelcontextprotocol/server-filesystem /path/to/directory
```

### 示例3: Excel MCP 服务器

**配置界面**:
```
服务器名称: excel-mcp
命令路径: node

命令参数:
  参数 1: /path/to/excel-mcp/index.js
```

**等价的命令行**:
```bash
node /path/to/excel-mcp/index.js
```

## 🔧 如何修复现有配置

如果您已经配置了STDIO服务器但参数配置错误：

### 步骤1: 打开编辑对话框
1. 在MCP设置页面找到出错的服务器
2. 点击编辑按钮（铅笔图标）

### 步骤2: 删除错误的参数
- 如果您有一个参数写着 `-y @readwise/readwise-mcp`
- 点击该参数旁边的删除按钮（红色圆圈）

### 步骤3: 分别添加正确的参数
1. 点击"添加参数"按钮
2. 在"参数 1"中输入: `-y`
3. 再次点击"添加参数"按钮
4. 在"参数 2"中输入: `@readwise/readwise-mcp`

### 步骤4: 保存并测试
1. 点击"保存"按钮
2. 等待服务器自动重新连接
3. 查看连接状态是否变为"已连接"

## 🐛 调试技巧

### 查看stderr日志
现在当STDIO进程出现错误时，您可以在日志中看到：

```
[STDIO stderr] readwise-mcp: npm ERR! code ENOENT
[STDIO stderr] readwise-mcp: npm ERR! syscall spawn
[STDIO stderr] readwise-mcp: npm ERR! path /path/to/package.json
```

这些日志可以帮助您：
- 诊断命令路径问题
- 发现缺少的依赖
- 了解参数格式错误
- 识别权限问题

### 常见错误模式

| 错误信息 | 可能原因 | 解决方案 |
|---------|---------|---------|
| Process closed stdout | 参数错误或命令不存在 | 检查参数是否正确分开 |
| Failed to start STDIO process | 命令路径错误 | 验证命令完整路径 |
| npm ERR! code ENOENT | npm包未安装 | 手动运行命令测试 |
| Permission denied | 文件权限问题 | 检查命令可执行权限 |

### 手动测试命令
在修复之前，您可以在终端手动测试命令：

```bash
# 测试npx命令
which npx
# 输出: /usr/local/bin/npx

# 测试完整命令
npx -y @readwise/readwise-mcp
# 应该看到MCP服务器启动信息
```

## 📊 技术细节

### 参数传递流程

1. **前端UI** → 用户在每个参数框中输入单个参数
2. **保存逻辑** → `_arguments.map((a) => a['value'])` 收集所有参数值
3. **Protobuf** → `MCPStdioConfigPB.args` 数组包含参数列表
4. **后端启动** → `cmd.args(&stdio_config.args)` 传递给进程

### 为什么需要分开参数？

在命令行中，参数由空格分隔，但空格也可能是参数值的一部分。例如：

```bash
# 错误: 整个字符串作为一个参数
command "-y @readwise/readwise-mcp"  # 命令会收到 1 个参数

# 正确: 两个独立的参数
command "-y" "@readwise/readwise-mcp"  # 命令会收到 2 个参数
```

Rust的`Command::args()`方法接受字符串数组，每个元素是一个完整的参数，不会再按空格分割。

## 📋 文件修改清单

### 后端
- ✅ `rust-lib/flowy-ai/src/mcp/client.rs`
  - 添加stderr捕获（第176-196行）
  - 添加100ms启动延迟（第198-199行）
  - 改进错误日志（第230-231行）

### 前端
- ✅ `workspace_mcp_settings_v2.dart`
  - 添加参数标题提示和图标（第650-663行）
  - 添加醒目的提示文本（第665-669行）
  - 改进参数输入框提示（第681-690行）

## 🎯 测试检查清单

配置完成后，检查以下内容：

- [ ] 每个参数都在单独的输入框中
- [ ] 参数没有多余的空格
- [ ] 点击"保存"后服务器自动连接
- [ ] 连接状态显示为"已连接"（绿色）
- [ ] 工具数量显示正确（蓝色徽章）
- [ ] 没有错误日志输出

## 💡 最佳实践

### 1. 参数配置
- ✅ 每个参数单独一行
- ✅ 不要包含引号（除非引号是参数的一部分）
- ✅ 参数顺序很重要
- ❌ 不要在一个框中输入多个参数
- ❌ 不要用逗号分隔参数

### 2. 命令路径
- ✅ 使用完整路径: `/usr/local/bin/npx`
- ✅ 或使用系统命令: `npx`（如果在PATH中）
- ❌ 避免使用shell别名
- ❌ 避免使用相对路径

### 3. 环境变量
- ✅ API密钥等敏感信息应使用环境变量
- ✅ 变量名使用大写字母和下划线
- ✅ 确保值不包含多余的空格

## 🚀 下一步优化建议

### 短期
- [ ] 添加参数验证（检查是否包含空格）
- [ ] 提供参数自动分割功能（可选）
- [ ] 添加常用MCP服务器的配置模板

### 中期
- [ ] 支持从命令行粘贴并自动解析参数
- [ ] 添加配置导入导出功能
- [ ] 提供配置验证器

### 长期
- [ ] 集成MCP服务器市场
- [ ] 一键安装和配置热门MCP服务器
- [ ] 提供交互式配置向导

---

**修复日期**: 2025年10月3日
**状态**: ✅ 已完成
**影响范围**: STDIO类型MCP服务器配置

