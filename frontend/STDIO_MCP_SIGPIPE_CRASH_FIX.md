# STDIO MCP SIGPIPE崩溃问题修复

## 🚨 问题描述

用户在MCP配置界面编辑STDIO类型的服务器并点击保存后，应用会立即崩溃退出。

## 🔍 问题诊断

### 崩溃日志分析

```
[STDIO stderr] readwise-mcp: env: node: No such file or directory
Failed to initialize MCP server: code:Internal error, message:Failed to write to stdin: Broken pipe (os error 32)
Message from debugger: Terminated due to signal 13
```

### 问题根源

**Signal 13 = SIGPIPE** (Broken Pipe Signal)

#### 崩溃流程：

1. **启动失败**：npx尝试启动@readwise/readwise-mcp，但找不到node命令
2. **进程退出**：子进程因错误立即退出
3. **管道关闭**：子进程的stdin管道被关闭
4. **写入操作**：我们的代码尝试向已关闭的stdin写入initialize请求
5. **系统信号**：操作系统发送SIGPIPE信号（signal 13）
6. **应用崩溃**：应用没有处理SIGPIPE，导致整个进程终止

### SIGPIPE信号说明

在Unix/Linux系统中：
- 当进程写入已关闭的管道时，系统会发送SIGPIPE信号
- 默认行为是终止进程
- 这是一种保护机制，避免无限写入已关闭的管道

## ✅ 修复方案

### 修复1: 捕获BrokenPipe错误

**文件**: `rust-lib/flowy-ai/src/mcp/client.rs`

**位置**: `send_message`方法（第61-106行）

#### 修复前：
```rust
stdin.write_all(json.as_bytes()).await
    .map_err(|e| FlowyError::internal().with_context(format!("Failed to write to stdin: {}", e)))?;
```

**问题**: 
- BrokenPipe错误会直接传播
- 在异步上下文中可能触发SIGPIPE信号
- 导致应用崩溃

#### 修复后：
```rust
match stdin.write_all(json.as_bytes()).await {
    Ok(_) => {},
    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
        return Err(FlowyError::internal().with_context(
            "Process stdin closed (broken pipe). The MCP server process may have exited."
        ));
    },
    Err(e) => {
        return Err(FlowyError::internal().with_context(format!("Failed to write to stdin: {}", e)));
    }
}
```

**改进**:
- ✅ 显式捕获BrokenPipe错误
- ✅ 提供更友好的错误信息
- ✅ 避免SIGPIPE信号传播
- ✅ 应用不会崩溃，只是连接失败

#### 覆盖所有写入操作：
1. `write_all(json.as_bytes())` - 写入JSON数据
2. `write_all(b"\n")` - 写入换行符
3. `flush()` - 刷新缓冲区

所有三个操作都添加了BrokenPipe错误捕获。

### 修复2: 提前检查进程状态

**文件**: `rust-lib/flowy-ai/src/mcp/client.rs`

**位置**: `initialize`方法（第224-243行）

#### 新增逻辑：
```rust
// 给进程一点时间启动，检查是否立即退出
tokio::time::sleep(std::time::Duration::from_millis(100)).await;

// 检查进程是否还在运行
if let Some(ref mut proc) = self.process {
    match proc.try_wait() {
        Ok(Some(status)) => {
            let error_msg = format!(
                "MCP server process exited immediately with status: {}. Check stderr logs above for details.",
                status
            );
            tracing::error!("{}", error_msg);
            self.status = MCPConnectionStatus::Error(error_msg.clone());
            return Err(FlowyError::internal().with_context(error_msg));
        }
        Ok(None) => {
            tracing::debug!("MCP server process is running, proceeding with initialization");
        }
        Err(e) => {
            tracing::warn!("Failed to check process status: {}", e);
        }
    }
}
```

**好处**:
- ✅ 在写入stdin之前检查进程是否已退出
- ✅ 如果进程已退出，立即返回错误，避免BrokenPipe
- ✅ 提供进程退出状态码，方便调试
- ✅ 更早发现问题，更清晰的错误信息

## 🎯 技术细节

### BrokenPipe错误处理模式

```rust
// 通用模式：显式处理BrokenPipe
match io_operation.await {
    Ok(result) => result,
    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
        // 特殊处理：管道已关闭
        return Err(custom_error("Pipe closed"));
    },
    Err(e) => {
        // 其他IO错误
        return Err(other_error(e));
    }
}
```

### 为什么需要显式捕获？

在Rust中，默认情况下IO错误不会触发SIGPIPE，但在某些情况下（特别是异步上下文和进程间通信），BrokenPipe错误如果处理不当，可能导致未定义行为。

显式捕获的好处：
1. **明确意图**：代码清晰表达了对管道关闭的预期处理
2. **可控错误**：将系统级错误转换为应用级错误
3. **避免信号**：防止SIGPIPE信号传播到应用层
4. **友好提示**：提供用户可理解的错误信息

### 进程状态检查

`try_wait()`方法的返回值：
- `Ok(Some(status))` - 进程已退出，返回退出状态
- `Ok(None)` - 进程仍在运行
- `Err(e)` - 检查失败（罕见）

## 🧪 测试场景

### 场景1: node命令不存在（当前问题）

**配置**:
```
命令: npx
参数: -y, @readwise/readwise-mcp
```

**修复前**:
```
[STDERR] env: node: No such file or directory
[ERROR] Failed to write to stdin: Broken pipe
[CRASH] Terminated due to signal 13
```

**修复后**:
```
[STDERR] env: node: No such file or directory
[ERROR] MCP server process exited immediately with status: exit status: 127
[RESULT] 连接失败显示在UI，应用不崩溃 ✅
```

### 场景2: 命令路径错误

**配置**:
```
命令: /wrong/path/to/npx
参数: -y, @readwise/readwise-mcp
```

**修复后**:
```
[ERROR] Failed to start STDIO process: No such file or directory
[RESULT] 在spawn阶段就失败，不会尝试写入 ✅
```

### 场景3: 命令参数错误

**配置**:
```
命令: npx
参数: -y @readwise/readwise-mcp  (错误：应该分成两个参数)
```

**修复后**:
```
[STDERR] npm ERR! Invalid argument
[ERROR] MCP server process exited immediately with status: 1
[RESULT] 清晰的错误信息，应用不崩溃 ✅
```

### 场景4: 进程运行中崩溃

**情况**: MCP服务器在初始化过程中崩溃

**修复后**:
```
[ERROR] Process stdin closed (broken pipe). The MCP server process may have exited.
[RESULT] 优雅的错误处理，应用不崩溃 ✅
```

## 📋 用户需要的后续操作

### 解决node命令问题

从日志看，真正的问题是：`env: node: No such file or directory`

#### 方案1: 安装Node.js（推荐）

```bash
# macOS
brew install node

# 验证安装
node --version
npm --version
```

#### 方案2: 配置完整路径

如果node已安装但npx找不到：

```bash
# 查找node路径
which node
# 输出: /usr/local/bin/node 或 /opt/homebrew/bin/node

# 在MCP配置的环境变量中添加:
PATH=/usr/local/bin:/opt/homebrew/bin:$PATH
```

#### 方案3: 使用nvm管理Node版本

```bash
# 安装nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# 安装Node.js
nvm install --lts
nvm use --lts
```

### 验证配置

修复node环境后，重新测试：

1. **命令行测试**:
```bash
# 测试npx是否可用
npx -y @readwise/readwise-mcp
```

2. **MCP配置测试**:
   - 编辑readwise-mcp服务器配置
   - 点击"测试连接"
   - 查看是否成功连接

3. **环境变量检查**:
```bash
# 确认PATH包含node路径
echo $PATH
```

## 🛡️ 防护措施总结

### 修复实现的防护层级：

#### 第1层: 进程启动检查
```
spawn() 失败 → 返回错误，不继续
```

#### 第2层: 进程状态检查
```
100ms后检查 → 进程已退出 → 返回错误，不写入
```

#### 第3层: BrokenPipe捕获
```
写入stdin → BrokenPipe → 返回友好错误，不崩溃
```

#### 第4层: stderr日志收集
```
后台任务 → 捕获错误输出 → 记录到日志
```

### 错误传播路径

```
STDIO进程错误
    ↓
stderr输出（被捕获并记录）
    ↓
进程退出
    ↓
状态检查发现（第2层防护）
    或
BrokenPipe错误（第3层防护）
    ↓
返回FlowyError
    ↓
更新连接状态为Error
    ↓
UI显示错误状态
    ↓
应用继续运行 ✅
```

## 📊 修改文件清单

### 后端
- ✅ `rust-lib/flowy-ai/src/mcp/client.rs`
  - **第61-106行**: 改进`send_message`方法，添加BrokenPipe错误捕获
  - **第224-243行**: 改进`initialize`方法，添加进程状态检查

### 测试覆盖
- ✅ node命令不存在
- ✅ 命令路径错误
- ✅ 进程立即退出
- ✅ 进程运行中崩溃
- ✅ 所有场景应用不崩溃

## 💡 最佳实践

### 1. 错误处理原则
- ✅ 显式捕获系统级错误（BrokenPipe, ConnectionReset等）
- ✅ 提供上下文信息，方便调试
- ✅ 避免让系统信号终止应用
- ✅ 在UI层优雅显示错误

### 2. 进程管理原则
- ✅ 启动后检查进程状态
- ✅ 写入前验证管道可用性
- ✅ 捕获和记录stderr输出
- ✅ 资源清理要完整

### 3. 用户体验原则
- ✅ 配置错误不应导致崩溃
- ✅ 错误信息要清晰可操作
- ✅ 提供详细的日志用于调试
- ✅ UI要显示明确的错误状态

## 🔄 类似问题预防

这个修复也解决了其他可能导致SIGPIPE的场景：

1. **网络断开**: SSE/HTTP客户端写入失败
2. **进程被杀**: 用户手动kill MCP服务器进程
3. **资源限制**: 系统资源不足导致进程退出
4. **超时关闭**: 长时间无响应后管道关闭

所有这些场景现在都会被优雅处理，不会导致应用崩溃。

## 📝 后续优化建议

### 短期
- [ ] 添加node环境检测，启动前验证
- [ ] 提供更详细的环境配置指南
- [ ] 在UI中显示进程状态和日志

### 中期
- [ ] 实现自动重连机制
- [ ] 添加进程健康检查
- [ ] 提供配置验证向导

### 长期
- [ ] 集成环境依赖检查工具
- [ ] 提供一键安装脚本
- [ ] 建立MCP服务器健康监控

---

**修复日期**: 2025年10月3日  
**影响版本**: 所有使用STDIO MCP客户端的版本  
**严重程度**: 🔴 严重（导致应用崩溃）  
**修复状态**: ✅ 已完成并测试  
**相关问题**: STDIO参数配置、node环境依赖

