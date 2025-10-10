# MCP 传输协议 UI 映射修复说明

## 问题描述

在全局设置中的 MCP 设置配置时，UI 中的传输协议选择与实际使用的传输实现之间存在映射错误：

- 当 UI 中选择传输协议为 **HTTP** 时，实际使用的传输方法实现是 **SSE 协议**
- 当 UI 中选择传输协议为 **SSE** 时，实际使用的传输方法实现是 **streamable-http 协议**

## 修复方案

通过交换 Rust 后端代码中 HTTP 和 SSE 的客户端映射关系来修正这个问题。

## 修改文件

### 1. `/rust-lib/flowy-ai/src/mcp/client_pool.rs`

**修改位置：** 第 45-58 行

**修改前：**
```rust
let mut client: Box<dyn MCPClient> = match config.transport_type {
    MCPTransportType::Stdio => {
        Box::new(StdioMCPClient::new(config.clone())?)
    }
    MCPTransportType::SSE => {
        Box::new(SSEMCPClient::new(config.clone())?)
    }
    MCPTransportType::HTTP => {
        Box::new(HttpMCPClient::new(config.clone())?)
    }
};
```

**修改后：**
```rust
let mut client: Box<dyn MCPClient> = match config.transport_type {
    MCPTransportType::Stdio => {
        Box::new(StdioMCPClient::new(config.clone())?)
    }
    // UI 中的 SSE 对应纯 JSON HTTP 实现
    MCPTransportType::SSE => {
        Box::new(HttpMCPClient::new(config.clone())?)
    }
    // UI 中的 HTTP 对应 streamable-http (SSE) 实现
    MCPTransportType::HTTP => {
        Box::new(SSEMCPClient::new(config.clone())?)
    }
};
```

### 2. `/rust-lib/flowy-ai/src/mcp/client.rs`

#### 修改 SSEMCPClient 类型检查

**修改位置：** 第 419-424 行

**修改前：**
```rust
impl SSEMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        if config.transport_type != MCPTransportType::SSE {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for SSE client"));
        }
```

**修改后：**
```rust
impl SSEMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        // UI中的HTTP对应streamable-http(SSE)实现
        if config.transport_type != MCPTransportType::HTTP {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for SSE client (expects HTTP)"));
        }
```

#### 修改 HttpMCPClient 类型检查

**修改位置：** 第 844-849 行

**修改前：**
```rust
impl HttpMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        if config.transport_type != MCPTransportType::HTTP {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for HTTP client"));
        }
```

**修改后：**
```rust
impl HttpMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        // UI中的SSE对应纯JSON HTTP实现
        if config.transport_type != MCPTransportType::SSE {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for HTTP client (expects SSE)"));
        }
```

## 修复后的映射关系

修复后，UI 中的传输协议选择与实际传输实现的对应关系如下：

| UI 选择 | 枚举类型 | 实际客户端 | 传输实现 |
|---------|----------|-----------|----------|
| **HTTP** | `MCPTransportType::HTTP` | `SSEMCPClient` | streamable-http (支持 SSE 流式响应，使用 `text/event-stream`) |
| **SSE** | `MCPTransportType::SSE` | `HttpMCPClient` | 纯 JSON HTTP (标准 HTTP 请求/响应) |
| **STDIO** | `MCPTransportType::Stdio` | `StdioMCPClient` | 标准输入输出 |

## 客户端实现说明

### SSEMCPClient (streamable-http)
- 支持 SSE 格式响应解析
- Accept 头：`application/json, text/event-stream`
- 支持会话管理（通过 `mcp-session-id` 头）
- 可以处理流式和非流式响应

### HttpMCPClient (纯 JSON HTTP)
- 仅支持标准 JSON 请求/响应
- Accept 头：`application/json`
- 无会话管理
- 标准的请求-响应模式

## 验证

修改后的代码已通过 Rust 编译检查：
```bash
cargo check --package flowy-ai
```

编译结果：✅ 成功（仅有一些预先存在的警告，与本次修改无关）

## 注意事项

1. 此修改仅涉及后端 Rust 代码的映射逻辑，**不需要修改 UI 代码**
2. 用户在 UI 中的选择保持不变，但底层实现已正确映射
3. 如果有现有的 MCP 服务器配置，建议用户重新测试连接以确保正常工作

## 测试建议

修复后，建议测试以下场景：

1. **测试 HTTP 传输（streamable-http）**：
   - 在 UI 中创建一个新的 MCP 服务器
   - 选择传输协议为 "HTTP"
   - 验证是否能正常连接支持 SSE 流式响应的服务器

2. **测试 SSE 传输（纯 JSON HTTP）**：
   - 在 UI 中创建一个新的 MCP 服务器
   - 选择传输协议为 "SSE"
   - 验证是否能正常连接标准 HTTP JSON 服务器

3. **测试 STDIO 传输**：
   - 确保 STDIO 传输方式不受影响

## 完成时间

2025-10-09

