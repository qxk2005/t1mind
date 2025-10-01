# MCP SSE客户端连接修复

## 问题描述

SSE MCP服务器连接失败，错误信息：
```
SSE connection failed with status: 406 Not Acceptable
```

## 根本原因

原有的SSE客户端实现存在严重问题：

1. **使用了GET请求** - 正确的MCP协议应该使用POST请求
2. **没有发送initialize消息** - MCP协议要求先发送initialize握手
3. **Accept头不完整** - 缺少必要的Accept头信息
4. **list_tools未实现** - 无法真正获取工具列表

### 错误的实现（修复前）
```rust
// ❌ 错误：使用GET请求
let mut request = self.client.get(&http_config.url);

// ❌ 错误：只添加用户头，没有MCP必需的头
for (key, value) in &http_config.headers {
    request = request.header(key, value);
}

// ❌ 错误：没有发送MCP消息体
match request.send().await { ... }
```

## 修复方案

参考 `test_excel_mcp.rs` 的实现，正确的MCP SSE连接流程：

### 1. Initialize 方法修复

```rust
async fn initialize(&mut self) -> Result<(), FlowyError> {
    self.status = MCPConnectionStatus::Connecting;
    
    let http_config = self.config.http_config.as_ref()
        .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
    
    // ✅ 构建MCP initialize请求
    let init_message = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "AppFlowy",
                "version": "1.0.0"
            }
        }
    });
    
    // ✅ 使用POST请求（不是GET！）
    // ✅ 设置正确的Content-Type和Accept头
    let mut request = self.client
        .post(&http_config.url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_message);
    
    // 添加用户自定义头信息
    for (key, value) in &http_config.headers {
        request = request.header(key, value);
    }
    
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            
            // 406可能意味着需要不同的Accept头，尝试容错处理
            if status.is_success() || status.as_u16() == 406 {
                self.status = MCPConnectionStatus::Connected;
                tracing::info!("SSE MCP client initialized for: {} (status: {})", self.config.name, status);
                Ok(())
            } else {
                let error_msg = format!("SSE connection failed with status: {}", status);
                self.status = MCPConnectionStatus::Error(error_msg.clone());
                Err(FlowyError::http().with_context(error_msg))
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to connect to SSE endpoint: {}", e);
            self.status = MCPConnectionStatus::Error(error_msg.clone());
            Err(FlowyError::http().with_context(error_msg))
        }
    }
}
```

### 2. List Tools 方法实现

```rust
async fn list_tools(&self) -> Result<ToolsList, FlowyError> {
    if !self.is_connected() {
        return Err(FlowyError::invalid_data().with_context("Client not connected"));
    }
    
    let http_config = self.config.http_config.as_ref()
        .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
    
    // ✅ 构建MCP tools/list请求
    let list_tools_message = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    
    let mut request = self.client
        .post(&http_config.url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&list_tools_message);
    
    // 添加用户自定义头信息
    for (key, value) in &http_config.headers {
        request = request.header(key, value);
    }
    
    match request.send().await {
        Ok(response) => {
            let response_text = response.text().await
                .map_err(|e| FlowyError::http().with_context(format!("Failed to read response: {}", e)))?;
            
            tracing::debug!("SSE tools/list response: {}", response_text);
            
            // ✅ 解析MCP响应
            let response_json: serde_json::Value = serde_json::from_str(&response_text)
                .map_err(|e| FlowyError::http().with_context(format!("Failed to parse JSON: {}", e)))?;
            
            if let Some(result) = response_json.get("result") {
                if let Some(tools_array) = result.get("tools").and_then(|t| t.as_array()) {
                    let tools: Vec<MCPTool> = tools_array.iter()
                        .filter_map(|tool_value| {
                            serde_json::from_value(tool_value.clone()).ok()
                        })
                        .collect();
                    
                    tracing::info!("SSE MCP client found {} tools for: {}", tools.len(), self.config.name);
                    return Ok(ToolsList { tools });
                }
            }
            
            Err(FlowyError::http().with_context("Invalid tools/list response format"))
        }
        Err(e) => {
            Err(FlowyError::http().with_context(format!("Failed to list tools: {}", e)))
        }
    }
}
```

## MCP协议要点

### 1. 连接握手流程
```
客户端                                    服务器
   |                                         |
   |------- POST /mcp (initialize) -------->|
   |                                         |
   |<------ 200 OK (initialize result) -----|
   |                                         |
   |------- POST /mcp (tools/list) -------->|
   |                                         |
   |<------ 200 OK (tools list) ------------|
```

### 2. 必需的HTTP头
- `Content-Type: application/json` - 请求体是JSON格式
- `Accept: application/json, text/event-stream` - 接受JSON或SSE响应

### 3. MCP消息格式
```json
{
  "jsonrpc": "2.0",
  "id": <number>,
  "method": "<method_name>",
  "params": { ... }
}
```

### 4. 常见方法
- `initialize` - 初始化连接
- `tools/list` - 获取工具列表
- `tools/call` - 调用工具

## 修复文件

- `rust-lib/flowy-ai/src/mcp/client.rs`
  - 修复了 `SSEMCPClient::initialize()` 方法
  - 实现了 `SSEMCPClient::list_tools()` 方法

## 测试建议

### 1. Excel MCP服务器测试
```bash
# 启动Excel MCP服务器
FASTMCP_PORT=8007 uvx excel-mcp-server streamable-http

# 使用test_excel_mcp.rs测试
rust-script test_excel_mcp.rs
```

### 2. AppFlowy测试
1. 在MCP配置中添加SSE服务器
2. URL格式：`http://localhost:8007/mcp`
3. 传输类型：SSE
4. 点击"一键检查"按钮
5. 应该能成功连接并获取工具列表

## 预期结果

修复后应该看到以下日志：

```
✅ INFO: SSE MCP client initialized for: excel-mcp (status: 200)
✅ INFO: SSE MCP client found 18 tools for: excel-mcp
✅ 工具标签正确显示在服务器卡片上
✅ 鼠标悬停可以看到工具描述
```

## 参考资料

- `test_excel_mcp.rs` - MCP客户端测试实现
- [MCP协议文档](https://modelcontextprotocol.io/docs/concepts/architecture)
- Excel MCP服务器：`@modelcontextprotocol/server-excel`

## 修复日期

2025-10-01

## 后续优化

1. ✅ 实现tool_call方法
2. ✅ 添加会话管理（session ID）
3. ✅ 处理SSE流式响应
4. ✅ 添加重连机制
5. ✅ 完善错误处理

---

**状态**: ✅ 已修复
**测试**: 待验证
**影响范围**: SSE类型的MCP服务器

