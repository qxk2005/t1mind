# Tool Auto-Detection Fix

## 问题描述

用户报告 AI 在调用工具时,即使没有指定 `source` 字段,自动检测也失败了,导致系统尝试调用不存在的原生工具:

```
🔧 [TOOL EXEC] No source specified, auto-detecting...
🔧 [TOOL EXEC] ❌ Tool call FAILED
Error: Native tool 'read_data_from_excel' not yet implemented
```

## 根本原因

`execute_auto_detected_tool` 方法使用了 `self.mcp_manager.list_servers()` 来获取 MCP 服务器列表。这个方法**只返回当前已连接的 MCP 服务器**,而不是所有已配置的服务器。

这和之前修复 `discover_available_tools` 时遇到的问题完全相同:如果 MCP 服务器配置了但尚未连接(例如刚启动应用时),`list_servers()` 会返回空列表,导致工具查找失败。

## 问题代码

```rust
// rust-lib/flowy-ai/src/agent/tool_call_handler.rs (旧代码)
async fn execute_auto_detected_tool(
    &self,
    request: &ToolCallRequest,
) -> FlowyResult<String> {
    // ❌ 只查询已连接的服务器
    let servers = self.mcp_manager.list_servers().await;
    for server in servers {
        if let Ok(tools) = self.mcp_manager.tool_list(&server.server_id).await {
            if tools.tools.iter().any(|t| t.name == request.tool_name) {
                return self.execute_mcp_tool(&server.server_id, request).await;
            }
        }
    }
    
    // 然后尝试原生工具
    self.execute_native_tool(request).await
}
```

## 修复方案

使用 `MCPClientManager` 的 `find_tool_by_name` 方法,该方法会:
1. 先检查缓存的工具列表
2. 从**所有已配置的 MCP 服务器**中查找工具
3. 如果需要,会自动连接服务器并获取工具列表

## 修复代码

```rust
// rust-lib/flowy-ai/src/agent/tool_call_handler.rs (新代码)
async fn execute_auto_detected_tool(
    &self,
    request: &ToolCallRequest,
) -> FlowyResult<String> {
    info!("🔍 [TOOL AUTO] Auto-detecting tool: {}", request.tool_name);
    
    // ✅ 使用 find_tool_by_name 从所有配置的 MCP 服务器中查找工具
    match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
        Some((server_id, tool)) => {
            info!("✅ [TOOL AUTO] Tool '{}' found in MCP server '{}' ({})", 
                  request.tool_name, server_id, &tool.description);
            self.execute_mcp_tool(&server_id, request).await
        }
        None => {
            info!("🔍 [TOOL AUTO] Tool '{}' not found in any MCP server, trying native tools", request.tool_name);
            self.execute_native_tool(request).await
        }
    }
}
```

## 修复效果

修复后,系统将:
1. 正确地从所有已配置的 MCP 服务器中查找工具
2. 优先使用缓存的工具列表(性能优化)
3. 在找不到 MCP 工具时才回退到原生工具
4. 提供详细的日志以便调试

## 新的日志输出

成功找到 MCP 工具时:
```
🔍 [TOOL AUTO] Auto-detecting tool: read_data_from_excel
✅ [TOOL AUTO] Tool 'read_data_from_excel' found in MCP server 'excel-mcp' (Read data from Excel worksheet with cell metadata including validation rules.)
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel on server: excel-mcp
```

未找到 MCP 工具时:
```
🔍 [TOOL AUTO] Auto-detecting tool: some_native_tool
🔍 [TOOL AUTO] Tool 'some_native_tool' not found in any MCP server, trying native tools
```

## 相关修复

这次修复与以下之前的修复一致:
- `AIManager::discover_available_tools` - 已修复为使用 `config_manager().get_all_servers()`
- `ToolCallHandler` 的显式 source 路由 - 已修复为优先查找 MCP 工具

## 测试步骤

1. 配置一个 MCP 服务器但不手动连接
2. 创建一个启用工具调用的智能体
3. 向 AI 提问,触发工具调用
4. AI 生成不带 `source` 字段的工具调用
5. 验证系统能正确找到并调用 MCP 工具

## 总结

这次修复确保了 `execute_auto_detected_tool` 方法与 `discover_available_tools` 方法使用相同的策略:
- **不依赖已连接的服务器列表**
- **查询所有已配置的服务器**
- **利用缓存提高性能**
- **提供清晰的日志**

这保证了工具自动检测的可靠性和一致性。

