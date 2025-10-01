# SSE MCP 服务器会话管理修复

## 问题描述

SSE 传输的 MCP 服务器（如 Excel MCP）无法获取工具列表，错误信息：

```
Bad Request: Missing session ID
```

## 根本原因

Excel MCP 服务器使用**会话管理机制**，要求客户端在初始化后发送 `notifications/initialized` 通知来建立会话。

当前的 SSE 客户端实现只发送了 `initialize` 请求，但**缺少发送 `notifications/initialized` 通知**的步骤，导致服务器认为会话未建立。

## 正确的 MCP 握手流程

参考 `excel_mcp_test.rs` (第 75-116 行)，完整的握手流程应该是：

```rust
// 1. 发送 initialize 请求
let init_request = InitializeRequest { ... };
let message = MCPMessage::request(
    json!(1),
    "initialize".to_string(),
    Some(serde_json::to_value(&init_request)?),
);
let response = send_mcp_message(message).await?;

// 2. 解析 initialize 响应
let init_response: InitializeResponse = serde_json::from_value(result)?;

// 3. ✅ 发送 initialized 通知（关键步骤！）
let initialized_notification = MCPMessage::notification(
    "notifications/initialized".to_string(),
    None,
);
send_mcp_message(initialized_notification).await?;

// 4. 现在可以调用其他 MCP 方法了
let tools = list_tools().await?;
```

## 修复内容

### 修改文件
- `rust-lib/flowy-ai/src/mcp/client.rs`

### 具体修改

在 `SSEMCPClient::initialize()` 方法中，在成功解析 `initialize` 响应后，立即发送 `notifications/initialized` 通知：

```rust
// 解析响应以验证初始化成功
if let Ok(_json) = self.parse_mcp_response(&response_text) {
    // ✅ 发送initialized通知（参考excel_mcp_test.rs第104-108行）
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    
    let mut notify_request = self.client
        .post(&http_config.url)
        .header("Content-Type", "application/json")
        .json(&initialized_notification);
    
    for (key, value) in &http_config.headers {
        notify_request = notify_request.header(key, value);
    }
    
    // 发送通知（忽略响应，通知不需要响应）
    if let Err(e) = notify_request.send().await {
        tracing::warn!("Failed to send initialized notification: {}", e);
    } else {
        tracing::debug!("Sent notifications/initialized to {}", self.config.name);
    }
    
    self.status = MCPConnectionStatus::Connected;
    return Ok(());
}
```

## MCP 协议要点

### 1. 会话初始化流程

```
客户端                                服务器
  |                                      |
  |--- initialize request ------------->|
  |                                      |
  |<-- initialize response (session) ---|
  |                                      |
  |--- notifications/initialized ------>| ✅ 建立会话
  |                                      |
  |--- tools/list request -------------->|
  |                                      |
  |<-- tools/list response -------------|
  |                                      |
```

### 2. 关键区别

- **Request（请求）**：需要 `id` 字段，期待响应
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }
  ```

- **Notification（通知）**：没有 `id` 字段，不期待响应
  ```json
  {
    "jsonrpc": "2.0",
    "method": "notifications/initialized",
    "params": {}
  }
  ```

### 3. 为什么需要 initialized 通知？

1. **会话建立**：服务器需要明确确认客户端已准备好接收通知和事件
2. **状态同步**：某些服务器（如 Excel MCP）会在收到此通知后才激活完整功能
3. **协议兼容性**：符合 MCP 2024-11-05 协议规范

## 测试验证

### 预期行为

1. ✅ 初始化请求成功（200 OK）
2. ✅ 发送 `notifications/initialized` 通知
3. ✅ 工具列表请求成功，返回工具数量 > 0
4. ✅ 工具调用正常工作

### 预期日志

```
SSE MCP client initialized for: excel-mcp (status: 200 OK)
Sent notifications/initialized to excel-mcp
SSE MCP client found X tools for: excel-mcp
```

### 不应该出现的错误

❌ `Bad Request: Missing session ID`
❌ `Invalid tools/list response format`

## 参考资料

- `excel_mcp_test.rs` - Excel MCP 客户端测试实现
- MCP Protocol Specification 2024-11-05
- JSON-RPC 2.0 Specification

## 修复时间

2025-10-01

## 相关文档

- [MCP_EMIT_FINAL_FIX.md](./MCP_EMIT_FINAL_FIX.md) - Bloc emit 错误修复
- [MCP_EVENT_HANDLER_IMPROVEMENTS.md](./MCP_EVENT_HANDLER_IMPROVEMENTS.md) - 事件处理改进

