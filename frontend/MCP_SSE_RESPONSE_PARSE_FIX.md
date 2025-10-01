# MCP SSE响应解析修复

## 问题描述

SSE MCP服务器连接成功，但工具列表获取失败，错误信息：
```
Failed to discover tools for server excel-mcp: 
Invalid tools/list response format
```

日志显示：
- ✅ 连接成功：`SSE MCP client initialized for: excel-mcp (status: 200 OK)`
- ❌ 工具发现失败：`Invalid tools/list response format`
- ❌ 最终结果：`获取到MCP工具列表: 0 个工具`

## 根本原因

SSE服务器返回的是**SSE事件流格式**，而不是纯JSON。

### SSE响应格式
```
event: message
data: {"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}

```

### 原有代码问题
```rust
// ❌ 错误：直接解析为JSON，无法处理SSE格式
let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
```

当响应是SSE格式时，直接JSON解析会失败。

## 修复方案

添加智能响应解析器，支持**SSE格式**和**纯JSON格式**两种响应。

### 1. 新增parse_mcp_response方法

```rust
/// 解析MCP响应（支持SSE格式和纯JSON格式）
/// 参考test_excel_mcp.rs的handle_sse_response实现
fn parse_mcp_response(&self, response_text: &str) -> Result<serde_json::Value, FlowyError> {
    // 先尝试直接解析JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_text) {
        tracing::debug!("Parsed as direct JSON response");
        return Ok(json);
    }
    
    // 如果失败，尝试解析SSE格式
    // SSE格式: event: message\ndata: {json}\n\n
    tracing::debug!("Attempting to parse as SSE format");
    
    for line in response_text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            let data = data.trim();
            if !data.is_empty() && data != "[DONE]" {
                match serde_json::from_str::<serde_json::Value>(data) {
                    Ok(json) => {
                        tracing::debug!("Successfully parsed SSE data line as JSON");
                        return Ok(json);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse SSE data line: {} - {}", e, data);
                    }
                }
            }
        }
    }
    
    Err(FlowyError::http().with_context(format!(
        "Failed to parse response as JSON or SSE format. Response: {}", 
        response_text.chars().take(200).collect::<String>()
    )))
}
```

### 2. 更新list_tools方法

```rust
async fn list_tools(&self) -> Result<ToolsList, FlowyError> {
    // ... 发送请求 ...
    
    match request.send().await {
        Ok(response) => {
            let response_text = response.text().await?;
            
            tracing::debug!("SSE tools/list raw response: {}", response_text);
            
            // ✅ 使用智能解析器
            let response_json = self.parse_mcp_response(&response_text)?;
            
            // 提取工具列表
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
            
            tracing::error!("Invalid tools/list response format, response: {}", response_text);
            Err(FlowyError::http().with_context("Invalid tools/list response format"))
        }
        Err(e) => {
            Err(FlowyError::http().with_context(format!("Failed to list tools: {}", e)))
        }
    }
}
```

### 3. 更新initialize方法

```rust
async fn initialize(&mut self) -> Result<(), FlowyError> {
    // ... 发送请求 ...
    
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            
            if status.is_success() {
                // ✅ 尝试读取并解析响应
                if let Ok(response_text) = response.text().await {
                    tracing::debug!("SSE initialize response: {}", response_text);
                    
                    // 使用智能解析器
                    if let Ok(_json) = self.parse_mcp_response(&response_text) {
                        self.status = MCPConnectionStatus::Connected;
                        tracing::info!("SSE MCP client initialized for: {} (status: {})", self.config.name, status);
                        return Ok(());
                    }
                }
                
                // 即使解析失败，200状态也认为连接成功
                self.status = MCPConnectionStatus::Connected;
                Ok(())
            } else {
                Err(FlowyError::http().with_context(format!("SSE connection failed with status: {}", status)))
            }
        }
        Err(e) => {
            Err(FlowyError::http().with_context(format!("Failed to connect to SSE endpoint: {}", e)))
        }
    }
}
```

## 解析逻辑流程

```
收到响应文本
    │
    ├─→ 尝试解析为纯JSON ────→ 成功 ──→ 返回JSON对象
    │                          │
    │                          ↓
    │                         失败
    │                          │
    └─→ 尝试解析为SSE格式 ──────┘
        │
        ├─→ 查找 "data: " 开头的行
        │   │
        │   ├─→ 提取data后的内容
        │   │
        │   └─→ 解析为JSON ──→ 成功 ──→ 返回JSON对象
        │                      │
        │                      ↓
        │                     失败
        └─→ 返回错误
```

## SSE格式示例

### 示例1：initialize响应
```
event: message
data: {"jsonrpc":"2.0","id":0,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"Excel MCP","version":"1.0.0"}}}

```

### 示例2：tools/list响应
```
event: message
data: {"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"read_data_from_excel","description":"Read data from Excel worksheet","inputSchema":{...}},{"name":"write_data_to_excel","description":"Write data to Excel","inputSchema":{...}}]}}

```

## 修复文件

```
rust-lib/flowy-ai/src/mcp/client.rs
├── SSEMCPClient::parse_mcp_response()  - 新增
├── SSEMCPClient::initialize()          - 更新
└── SSEMCPClient::list_tools()          - 更新
```

## 兼容性

修复后的解析器支持：

| 响应格式 | 支持 | 说明 |
|---------|------|------|
| 纯JSON | ✅ | 标准JSON-RPC响应 |
| SSE格式 | ✅ | event: + data: 格式 |
| 混合格式 | ✅ | 先试JSON，失败后试SSE |

## 测试建议

### 1. 启动Excel MCP服务器
```bash
FASTMCP_PORT=8007 uvx excel-mcp-server streamable-http
```

### 2. 重新编译
```bash
cd rust-lib/flowy-ai
cargo build --release
```

### 3. 在AppFlowy中测试
1. 添加SSE服务器：`http://localhost:8007/mcp`
2. 点击"一键检查"
3. 观察日志和UI

## 预期结果

### 成功的日志
```
✅ DEBUG: SSE tools/list raw response: event: message\ndata: {...}
✅ DEBUG: Attempting to parse as SSE format
✅ DEBUG: Successfully parsed SSE data line as JSON
✅ INFO:  SSE MCP client found 18 tools for: excel-mcp
```

### UI显示
```
┌────────────────────────────────────────┐
│ excel-mcp                  SSE    ● ✓  │
│ Excel文件操作服务器                     │
│ URL: http://localhost:8007/mcp         │
│                                        │
│ 🔧 18                                  │
│ 🔧 read_data_from_excel  🔧 write_...  │
│ 🔧 apply_formula  🔧 format_range  +14 │
└────────────────────────────────────────┘
```

## 调试技巧

### 查看原始响应
启用debug日志：
```bash
RUST_LOG=debug cargo run
```

查找这些日志：
```
DEBUG: SSE tools/list raw response: ...
DEBUG: Parsed as direct JSON response
或
DEBUG: Attempting to parse as SSE format
DEBUG: Successfully parsed SSE data line as JSON
```

### 手动测试响应格式
```bash
curl -X POST http://localhost:8007/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

观察响应格式是纯JSON还是SSE。

## 故障排查

### 问题1：仍然显示0个工具

**检查项**：
- [ ] Excel MCP服务器是否正常运行？
- [ ] Rust代码是否重新编译？
- [ ] 查看debug日志中的原始响应内容

### 问题2：解析失败

**可能原因**：
- 响应格式不是标准的SSE或JSON
- 响应内容被截断

**解决方案**：
查看完整的debug日志，检查response_text内容。

## 性能影响

- ✅ 轻量级：只是文本解析，性能开销极小
- ✅ 智能回退：先试快速的JSON解析，失败才用SSE解析
- ✅ 无副作用：不影响其他传输类型（STDIO、HTTP）

## 相关文档

- [MCP_SSE_CLIENT_FIX.md](./MCP_SSE_CLIENT_FIX.md) - SSE客户端连接修复
- [MCP_SSE_FIX_TEST_STEPS.md](./MCP_SSE_FIX_TEST_STEPS.md) - 测试步骤
- [test_excel_mcp.rs](./test_excel_mcp.rs) - 参考实现

## 修复日期

2025-10-01

## 状态

✅ **已完成并通过编译**

---

**下一步**: 重新编译并测试，应该能成功获取18个Excel MCP工具！🎉

