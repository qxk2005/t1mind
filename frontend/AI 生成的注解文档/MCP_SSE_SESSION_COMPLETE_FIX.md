# SSE MCP 服务器会话管理完整修复

## 问题描述

Excel MCP 服务器（SSE 传输）返回 `"Bad Request: Missing session ID"` 错误，无法获取工具列表。

## 根本原因

Excel MCP 服务器使用 **HTTP 头传递会话 ID** 的方式进行会话管理，而不是使用 Cookie。我们之前的实现：

1. ❌ 没有从响应头提取 `mcp-session-id`
2. ❌ `notifications/initialized` 的 `params` 使用了空对象 `{}` 而不是省略
3. ❌ 后续请求（如 `tools/list`）没有携带 `mcp-session-id` 头

## 完整修复方案

### 参考实现

完全按照 `test_excel_mcp.rs` 的成功实现重写了 SSE 客户端：

**test_excel_mcp.rs 的关键点：**

1. **会话 ID 管理**（第 117, 360-372 行）
   ```rust
   session_id: Option<String>,
   
   // 从响应头提取会话ID
   if let Some(session_id) = response.headers().get("mcp-session-id") {
       if let Ok(session_id_str) = session_id.to_str() {
           self.session_id = Some(session_id_str.to_string());
       }
   }
   ```

2. **notifications/initialized 格式**（第 233-236 行）
   ```rust
   MCPMessage::notification(
       "notifications/initialized".to_string(),
       None,  // ✅ 注意：params 是 None，不是空对象！
   )
   ```

3. **携带会话 ID**（第 359-362 行）
   ```rust
   if let Some(session_id) = &self.session_id {
       request = request.header("mcp-session-id", session_id);
   }
   ```

### 修改内容

#### 1. 添加会话 ID 字段

```rust
pub struct SSEMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    tools: Vec<MCPTool>,
    client: reqwest::Client,
    session_id: Option<String>,  // ✅ 新增：会话ID
}
```

#### 2. 重写 initialize 方法

完整流程（参考 test_excel_mcp.rs）：

```rust
async fn initialize(&mut self) -> Result<(), FlowyError> {
    // 1. 发送 initialize 请求
    let init_message = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { ... }
    });
    
    let response = self.client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_message)
        .send()
        .await?;
    
    // 2. 提取会话ID
    if let Some(session_id) = response.headers().get("mcp-session-id") {
        self.session_id = Some(session_id.to_str()?.to_string());
    }
    
    // 3. 发送 notifications/initialized 通知
    // ✅ 注意：不包含 "params" 字段！
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    let mut notify_request = self.client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");
    
    // 4. 添加会话ID头
    if let Some(ref session_id) = self.session_id {
        notify_request = notify_request.header("mcp-session-id", session_id);
    }
    
    let notify_response = notify_request
        .json(&initialized_notification)
        .send()
        .await?;
    
    // 5. 再次提取会话ID（可能更新）
    if let Some(session_id) = notify_response.headers().get("mcp-session-id") {
        self.session_id = Some(session_id.to_str()?.to_string());
    }
    
    Ok(())
}
```

#### 3. 更新 list_tools 方法

在所有后续请求中携带会话 ID：

```rust
async fn list_tools(&self) -> Result<ToolsList, FlowyError> {
    let list_tools_message = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    
    let mut request = self.client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");
    
    // ✅ 添加会话ID头
    if let Some(ref session_id) = self.session_id {
        request = request.header("mcp-session-id", session_id);
    }
    
    let response = request
        .json(&list_tools_message)
        .send()
        .await?;
    
    // 解析工具列表...
}
```

## 关键差异对照

### ❌ 之前的错误实现

```rust
// 1. 没有 session_id 字段
pub struct SSEMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    // ❌ 缺少 session_id
}

// 2. notifications/initialized 格式错误
let initialized_notification = json!({
    "jsonrpc": "2.0",
    "method": "notifications/initialized",
    "params": {}  // ❌ 不应该有 params 字段！
});

// 3. 没有从响应头提取会话ID
let response = request.send().await?;
// ❌ 没有提取 mcp-session-id 头

// 4. 后续请求不携带会话ID
let request = self.client.post(&url)
    .header("Content-Type", "application/json")
    .json(&list_tools_message);
// ❌ 缺少 mcp-session-id 头
```

### ✅ 正确的实现

```rust
// 1. 添加 session_id 字段
pub struct SSEMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    session_id: Option<String>,  // ✅
}

// 2. notifications/initialized 格式正确
let initialized_notification = json!({
    "jsonrpc": "2.0",
    "method": "notifications/initialized"
    // ✅ 不包含 params 字段
});

// 3. 提取会话ID
if let Some(session_id) = response.headers().get("mcp-session-id") {
    self.session_id = Some(session_id.to_str()?.to_string());
}  // ✅

// 4. 携带会话ID
if let Some(ref session_id) = self.session_id {
    request = request.header("mcp-session-id", session_id);
}  // ✅
```

## Excel MCP 服务器的会话机制

1. **初始化流程**：
   ```
   客户端 -> initialize 请求
   服务器 -> initialize 响应 + Set header: mcp-session-id
   客户端 -> 保存 session_id
   客户端 -> notifications/initialized + header: mcp-session-id
   服务器 -> 确认会话建立
   ```

2. **后续请求**：
   ```
   客户端 -> tools/list + header: mcp-session-id
   服务器 -> 验证session_id -> 返回工具列表
   ```

3. **如果缺少 session_id**：
   ```
   客户端 -> tools/list (无 session_id 头)
   服务器 -> 400 Bad Request: Missing session ID
   ```

## 预期结果

重新运行应用后，日志应该显示：

```
✅ Got session ID from initialize: {session_id}
✅ Initialized notification sent (status: 200, empty response)
✅ SSE MCP client initialized for: excel-mcp with session_id: Some({id})
✅ Adding session ID to tools/list request: {session_id}
✅ SSE MCP client found X tools for: excel-mcp
```

**不应该再看到**：
- ❌ `Bad Request: Missing session ID`
- ❌ `Invalid tools/list response format`

## 修复时间

2025-10-01

## 相关文档

- `test_excel_mcp.rs` - Excel MCP 客户端参考实现
- [MCP_SSE_SESSION_FIX.md](./MCP_SSE_SESSION_FIX.md) - 之前的修复尝试
- [MCP_EMIT_FINAL_FIX.md](./MCP_EMIT_FINAL_FIX.md) - Bloc emit 错误修复

