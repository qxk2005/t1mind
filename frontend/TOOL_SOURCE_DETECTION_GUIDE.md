# 工具来源检测机制说明

## AI 如何判断工具是原生还是 MCP？

### 原始设计

AI 通过在 `<tool_call>` JSON 中指定 `source` 字段来标识工具来源：

```json
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "arguments": {...},
  "source": "appflowy"     // AI 指定的来源
}
```

**可能的 `source` 值**：
- `"appflowy"` 或 `"native"` - 原生工具
- MCP 服务器 ID（如 `"excel-mcp"`）- MCP 工具
- `null` 或不指定 - 系统自动检测

### 问题：系统提示词硬编码了 `source`

之前的系统提示词中：

```rust
// ❌ 旧版本
prompt.push_str("    \"source\": \"appflowy\"\n");
```

这导致 AI **总是**生成 `"source": "appflowy"`，即使工具实际上是 MCP 工具。

### 解决方案：移除硬编码 + 智能路由

#### 1. 修改系统提示词

**新版本**：
```rust
// ✅ 新版本 - 不指定 source
{
  "id": "unique_call_id",
  "tool_name": "tool_name_here",
  "arguments": {
    "param1": "value1"
  }
  // 不包含 source 字段
}
```

并添加说明：
```
**Note:** Do not specify 'source' field - the system will automatically detect 
whether the tool is native or external.
```

#### 2. 智能路由逻辑

系统在 `tool_call_handler.rs` 中实现了三层路由逻辑：

```rust
if let Some(source) = &request.source {
    // 情况 1: AI 指定了 source
    if source == "native" || source == "appflowy" {
        // 即使 AI 说是原生工具，也先检查 MCP
        match find_tool_in_mcp(&request.tool_name) {
            Some(server_id) => execute_mcp_tool(server_id),  // 找到了，用 MCP
            None => execute_native_tool()                    // 没找到，用原生
        }
    } else {
        // source 是具体的 MCP server ID
        execute_mcp_tool(source)
    }
} else {
    // 情况 2: AI 没有指定 source（推荐）
    auto_detect_and_execute()
}
```

## 工具检测的详细流程

### 第 1 步：解析工具调用请求

```rust
pub struct ToolCallRequest {
    pub id: String,
    pub tool_name: String,
    pub arguments: Value,
    pub source: Option<String>,  // 可选字段
}
```

### 第 2 步：路由决策

代码位置：`rust-lib/flowy-ai/src/agent/tool_call_handler.rs:299-325`

```rust
// 路由逻辑
let result = if let Some(source) = &request.source {
    // A. AI 指定了 source
    if source == "native" || source == "appflowy" {
        // A1. 先尝试 MCP（容错机制）
        info!("🔧 Source specified as '{}', checking MCP first...", source);
        
        match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
            Some((server_id, _)) => {
                info!("✅ Tool '{}' found in MCP server '{}'", 
                      request.tool_name, server_id);
                self.execute_mcp_tool(&server_id, request).await
            }
            None => {
                info!("🔧 Tool not found in MCP, trying native");
                self.execute_native_tool(request).await
            }
        }
    } else {
        // A2. source 是具体的 MCP server ID
        info!("🔧 Calling MCP tool on server: {}", source);
        self.execute_mcp_tool(source, request).await
    }
} else {
    // B. AI 没有指定 source - 自动检测
    info!("🔧 No source specified, auto-detecting...");
    self.execute_auto_detected_tool(request).await
};
```

### 第 3 步：自动检测逻辑

代码位置：`rust-lib/flowy-ai/src/agent/tool_call_handler.rs:408-425`

```rust
async fn execute_auto_detected_tool(&self, request: &ToolCallRequest) -> FlowyResult<String> {
    info!("🔍 Auto-detecting tool source for: {}", request.tool_name);
    
    // 1. 先尝试从所有 MCP 服务器中查找
    match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
        Some((server_id, _tool)) => {
            info!("✅ Tool '{}' found on MCP server: {}", request.tool_name, server_id);
            return self.execute_mcp_tool(&server_id, request).await;
        }
        None => {
            warn!("⚠️ Tool '{}' not found in any MCP server", request.tool_name);
        }
    }
    
    // 2. MCP 中没找到，尝试原生工具
    info!("🔍 Trying native tools for: {}", request.tool_name);
    self.execute_native_tool(request).await
}
```

### 第 4 步：MCP 工具查找

MCP 管理器的 `find_tool_by_name` 方法会：

1. 遍历所有已连接的 MCP 服务器
2. 在每个服务器的工具注册表中查找工具名称
3. 返回第一个匹配的 `(server_id, tool)` 对

```rust
// rust-lib/flowy-ai/src/mcp/tool_discovery.rs
pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
    let registry = self.tool_registry.read().await;
    
    for (server_id, tools) in registry.iter() {
        for tool in tools {
            if tool.name == tool_name {
                return Some((server_id.clone(), tool.clone()));
            }
        }
    }
    
    None
}
```

## 日志追踪

### 情况 1：AI 指定了 source（旧行为）

```
🔧 [TOOL EXEC] Source: Some("appflowy")
🔧 [TOOL EXEC] Source specified as 'appflowy', checking MCP first...
✅ [TOOL EXEC] Tool 'read_data_from_excel' found in MCP server 'excel-mcp', using MCP instead
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel on server: excel-mcp
✅ Tool call SUCCEEDED
```

### 情况 2：AI 没有指定 source（新行为 - 推荐）

```
🔧 [TOOL EXEC] Source: None
🔧 [TOOL EXEC] No source specified, auto-detecting...
🔍 [TOOL DETECT] Auto-detecting tool source for: read_data_from_excel
✅ [TOOL DETECT] Tool 'read_data_from_excel' found on MCP server: excel-mcp
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel on server: excel-mcp
✅ Tool call SUCCEEDED
```

### 情况 3：AI 指定了具体的 MCP server ID

```
🔧 [TOOL EXEC] Source: Some("excel-mcp")
🔧 [TOOL EXEC] Calling MCP tool on server: excel-mcp
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel on server: excel-mcp
✅ Tool call SUCCEEDED
```

## 优先级和策略

### 当前策略（从高到低）

1. **MCP 工具优先**：
   - 即使 AI 说是原生工具，也先检查 MCP
   - 这避免了 AI 判断错误导致的问题

2. **原生工具作为后备**：
   - 只有在 MCP 中找不到时才尝试原生工具
   - 原生工具目前大多未实现

3. **自动检测是最佳实践**：
   - AI 不需要知道工具来源
   - 系统自动选择正确的执行路径

### 为什么这样设计？

1. **容错性**：AI 可能不知道哪些工具是 MCP 的
2. **灵活性**：添加新的 MCP 工具不需要更新 AI 提示词
3. **简化 AI 逻辑**：AI 只需要知道工具名称和参数
4. **向后兼容**：即使 AI 生成了 `source` 字段，系统也能正确处理

## 测试验证

### 测试 1：验证自动检测

```
User: 查看 excel 文件 myfile.xlsx 的内容

AI generates:
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "arguments": {"file_path": "myfile.xlsx"}
  // 没有 source 字段
}

Expected logs:
🔧 [TOOL EXEC] No source specified, auto-detecting...
✅ [TOOL DETECT] Tool found on MCP server: excel-mcp
```

### 测试 2：验证容错（AI 错误地指定了 appflowy）

```
AI generates:
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "source": "appflowy"  // ❌ 错误
}

Expected logs:
🔧 [TOOL EXEC] Source specified as 'appflowy', checking MCP first...
✅ [TOOL EXEC] Tool found in MCP server 'excel-mcp', using MCP instead
```

## 总结

**工具来源判断的责任分配**：

| 角色 | 责任 | 实现位置 |
|------|------|----------|
| **AI 模型** | 识别需要使用的工具名称和参数 | 通过系统提示词指导 |
| **系统路由** | 决定工具的实际来源（MCP vs Native） | `tool_call_handler.rs` |
| **MCP 管理器** | 维护工具注册表，查找工具 | `mcp/tool_discovery.rs` |

**推荐配置**：
- ✅ AI 不指定 `source`（已修改系统提示词）
- ✅ 系统自动检测（已实现）
- ✅ MCP 优先策略（已实现）
- ✅ 容错机制（已实现）

现在测试应该能成功了，因为：
1. 新创建的智能体不会在提示词中看到硬编码的 `source: "appflowy"`
2. 即使 AI 生成了 `source: "appflowy"`，系统也会自动纠正并使用 MCP 工具

