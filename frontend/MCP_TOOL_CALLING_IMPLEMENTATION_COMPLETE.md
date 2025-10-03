# MCP 工具调用完整实现

## 问题背景

之前AI调用MCP工具时报错:
```
🔧 [MCP TOOL] ❌ Tool call failed: read_data_from_excel
Error: code:Not support yet, message:Tool calling not implemented yet
```

虽然工具能被发现,连接检查也通过,但实际的工具调用功能还没有实现。

## 实现内容

### 1. MCP 工具调用自动连接机制

在 `tool_call_handler.rs` 中添加了自动连接检查:

```rust
// rust-lib/flowy-ai/src/agent/tool_call_handler.rs:375-391
async fn execute_mcp_tool(...) -> FlowyResult<String> {
    info!("🔧 [MCP TOOL] Calling MCP tool: {} on server: {}", ...);
    
    // 🔌 自动连接检查:如果服务器未连接,先尝试连接
    if !self.mcp_manager.is_server_connected(server_id) {
        info!("🔌 [MCP AUTO-CONNECT] Server '{}' is not connected, attempting to connect...", server_id);
        
        match self.mcp_manager.connect_server_from_config(server_id).await {
            Ok(()) => {
                info!("✅ [MCP AUTO-CONNECT] Successfully connected to server '{}'", server_id);
            }
            Err(e) => {
                error!("❌ [MCP AUTO-CONNECT] Failed to connect to server '{}': {}", server_id, e);
                return Err(...);
            }
        }
    } else {
        info!("✓ [MCP TOOL] Server '{}' already connected", server_id);
    }
    
    // 然后执行工具调用
    let response = self.mcp_manager.call_tool(...).await?;
    ...
}
```

### 2. HTTP MCP 客户端工具调用实现

为 `HttpMCPClient` 实现了完整的 MCP 协议工具调用:

```rust
// rust-lib/flowy-ai/src/mcp/client.rs:485-582
async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
    if !self.is_connected() {
        return Err(FlowyError::invalid_data().with_context("Client not connected"));
    }
    
    // 1. 构建 MCP 协议的 tools/call 请求
    let mcp_request = ProtocolCallToolRequest {
        name: request.name.clone(),
        arguments: Some(request.arguments.clone()),
    };
    
    let message = MCPMessage::request(
        serde_json::json!(timestamp),
        "tools/call".to_string(),
        Some(serde_json::to_value(&mcp_request)?),
    );
    
    // 2. 发送 HTTP POST 请求
    let mut http_request = self.client
        .post(&http_config.url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json");
    
    // 添加自定义头信息
    for (key, value) in &http_config.headers {
        http_request = http_request.header(key, value);
    }
    
    let json_body = serde_json::to_string(&message)?;
    let response = http_request.body(json_body).send().await?;
    
    // 3. 解析响应
    let response_text = response.text().await?;
    let response_message: MCPMessage = serde_json::from_str(&response_text)?;
    
    // 4. 检查错误
    if let Some(error) = response_message.error {
        return Err(...);
    }
    
    // 5. 提取工具调用结果
    let result = response_message.result.ok_or(...)?;
    let tool_response: ProtocolCallToolResponse = serde_json::from_value(result)?;
    
    // 6. 转换为我们的响应格式
    let content = tool_response.content.into_iter().map(|c| {
        ToolCallContent {
            r#type: c.r#type,
            text: c.text,
            data: None,
        }
    }).collect();
    
    Ok(ToolCallResponse {
        content,
        is_error: tool_response.is_error.unwrap_or(false),
    })
}
```

## 实现特点

### 1. 完整的 MCP 协议支持

- ✅ 遵循 MCP 协议规范
- ✅ 使用正确的请求格式 (`tools/call` 方法)
- ✅ 解析标准的 MCP 响应
- ✅ 处理错误情况

### 2. 自动连接机制

- ✅ 调用工具前自动检查连接状态
- ✅ 未连接时自动尝试连接
- ✅ 连接成功后继续执行工具调用
- ✅ 连接失败时返回清晰的错误信息

### 3. 详细的日志记录

```
🔌 [MCP AUTO-CONNECT] Server 'xxx' is not connected, attempting to connect...
✅ [MCP AUTO-CONNECT] Successfully connected to server 'xxx'
✓ [MCP TOOL] Server 'xxx' already connected
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel on server: xxx
🔧 [MCP TOOL] ✅ Tool call succeeded in 123ms
```

## 工作流程

### 完整的工具调用流程

```
1. AI 生成工具调用请求
   ↓
2. ToolCallHandler 解析请求
   ↓
3. 自动检测工具类型 (MCP/Native)
   ↓
4. 调用 execute_mcp_tool
   ↓
5. 检查服务器连接状态
   ├─ 未连接 → 自动连接 → 继续
   └─ 已连接 → 继续
   ↓
6. 构建 MCP 协议请求
   ↓
7. 发送 HTTP POST 到 MCP 服务器
   ↓
8. 接收并解析 MCP 响应
   ↓
9. 提取工具执行结果
   ↓
10. 返回结果给 AI
```

## 支持的传输类型

| 类型 | 工具调用状态 | 说明 |
|------|------------|------|
| HTTP | ✅ 已实现 | 完整的 MCP 协议实现 |
| SSE | ⚠️ 待实现 | 需要类似实现 |
| STDIO | ⚠️ 待实现 | 需要通过进程通信 |

当前优先实现了 HTTP 类型,因为 Excel MCP 服务器使用 HTTP 传输。

## MCP 协议细节

### 请求格式

```json
{
  "jsonrpc": "2.0",
  "id": 1733165526000,
  "method": "tools/call",
  "params": {
    "name": "read_data_from_excel",
    "arguments": {
      "filepath": "myfile.xlsx",
      "sheet_name": "Sheet1"
    }
  }
}
```

### 响应格式

```json
{
  "jsonrpc": "2.0",
  "id": 1733165526000,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"data\": [...]}"
      }
    ],
    "isError": false
  }
}
```

## 错误处理

### 1. 连接失败
```
❌ [MCP AUTO-CONNECT] Failed to connect to server 'xxx': Connection refused
Error: Auto-connect failed for server 'xxx': Connection refused
```

### 2. HTTP 请求失败
```
Error: HTTP request failed with status: 500
```

### 3. MCP 协议错误
```
Error: MCP error: Tool not found (code: -32601)
```

### 4. 响应解析失败
```
Error: Failed to parse MCP response: unexpected EOF
```

## 测试建议

### 1. 单元测试

```rust
#[tokio::test]
async fn test_http_mcp_tool_call() {
    let mut client = HttpMCPClient::new(config).unwrap();
    client.initialize().await.unwrap();
    
    let request = ToolCallRequest {
        name: "read_data_from_excel".to_string(),
        arguments: json!({
            "filepath": "test.xlsx",
            "sheet_name": "Sheet1"
        }),
    };
    
    let response = client.call_tool(request).await.unwrap();
    assert!(!response.is_error);
    assert!(!response.content.is_empty());
}
```

### 2. 集成测试

1. 启动 Excel MCP 服务器
2. 配置 MCP 服务器连接
3. 创建启用工具调用的智能体
4. 向 AI 提问触发工具调用
5. 验证工具调用成功并返回结果

### 3. 日志验证

查看日志确认:
- ✅ 工具自动发现成功
- ✅ 自动连接成功(如果需要)
- ✅ 工具调用请求发送
- ✅ 响应接收和解析
- ✅ 结果返回给 AI

## 下一步改进

### 1. 实现 SSE 客户端工具调用

SSE 类型需要处理事件流:

```rust
async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
    // 1. 发送 SSE 请求
    // 2. 监听事件流
    // 3. 收集响应数据
    // 4. 返回结果
}
```

### 2. 实现 STDIO 客户端工具调用

STDIO 类型需要通过标准输入输出与子进程通信:

```rust
async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
    // 1. 构建 JSON-RPC 请求
    // 2. 写入进程的 stdin
    // 3. 从进程的 stdout 读取响应
    // 4. 解析响应
}
```

### 3. 添加重试机制

对于网络错误,可以添加自动重试:

```rust
async fn call_tool_with_retry(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
    let max_retries = 3;
    let mut attempts = 0;
    
    loop {
        match self.call_tool(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 4. 添加超时控制

```rust
async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
    tokio::time::timeout(
        Duration::from_secs(30),
        self.call_tool_internal(request)
    ).await?
}
```

## 总结

### ✅ 已完成

1. **自动连接机制**: 工具调用前自动检查并连接服务器
2. **HTTP 工具调用**: 完整实现 HTTP MCP 客户端的工具调用
3. **MCP 协议支持**: 遵循标准 MCP 协议规范
4. **错误处理**: 完善的错误检查和日志记录
5. **自动路由**: 智能检测工具类型并路由到正确的执行器

### 🎯 现在可以

- ✅ AI 自动发现 MCP 工具
- ✅ AI 调用时自动连接服务器
- ✅ AI 成功执行 MCP 工具并获取结果
- ✅ 用户看到清晰的调用日志和错误信息

### 🚀 使用流程

1. 配置 MCP 服务器 (Excel MCP, HTTP 类型)
2. 创建启用工具调用的智能体
3. 向 AI 提问: "帮我读取 myfile.xlsx 的内容"
4. AI 自动:
   - 发现 `read_data_from_excel` 工具
   - 检查服务器连接状态
   - 如果未连接,自动连接
   - 调用工具获取数据
   - 将结果整合到回答中

这样用户就能体验到完整的 AI + MCP 工具集成了!

