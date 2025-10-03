# 工具发现缓存查找修复

## 🐛 问题描述

工具调用被正确检测和解析,但在执行时找不到工具,即使工具确实存在于 MCP 服务器的缓存中。

### 错误日志

```
🔧 [TOOL EXEC] Executing tool...
🔧 [TOOL EXEC] No source specified, auto-detecting...
🔍 [TOOL AUTO] Auto-detecting tool: read_data_from_excel
🔍 [TOOL AUTO] Tool 'read_data_from_excel' not found in any MCP server
❌ Tool call FAILED
Error: Native tool 'read_data_from_excel' not yet implemented
```

### 症状

- ✅ 工具详情成功加载到系统提示(25 个工具)
- ✅ AI 正确生成工具调用
- ✅ 工具调用格式正确(经过 markdown 转换)
- ✅ 工具调用被成功解析
- ❌ **`find_tool_by_name` 找不到工具**

## 🔍 根本原因

### 问题 1: 工具注册表为空

`ToolDiscoveryManager` 维护一个 `tool_registry`,只包含**已连接**服务器的工具:

```rust
// rust-lib/flowy-ai/src/mcp/tool_discovery.rs
pub async fn discover_all_tools(&self) -> HashMap<String, Vec<MCPTool>> {
    let clients_info = self.client_pool.list_clients().await;
    
    for client_info in clients_info {
        if client_info.status == MCPConnectionStatus::Connected {  // ❌ 只查找已连接的
            match self.discover_tools(&client_info.server_id).await {
                Ok(tools) => {
                    all_tools.insert(client_info.server_id.clone(), tools);
                }
            }
        }
    }
}
```

**问题**: 如果 MCP 服务器**未连接**,其工具就不会在 `tool_registry` 中。

### 问题 2: `find_tool_by_name` 只查注册表

```rust
// rust-lib/flowy-ai/src/mcp/tool_discovery.rs (修复前)
pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
    let registry = self.tool_registry.read().await;
    
    for (server_id, tools) in registry.iter() {  // ❌ 只查注册表
        for tool in tools {
            if tool.name == tool_name {
                return Some((server_id.clone(), tool.clone()));
            }
        }
    }
    
    None
}
```

**问题**: 没有检查 `MCPServerConfig.cached_tools`。

### 为什么缓存的工具没有用?

系统有两个工具数据源:
1. **工具注册表** (`tool_registry`): 从已连接的 MCP 服务器实时获取
2. **缓存的工具** (`MCPServerConfig.cached_tools`): 持久化在配置中

之前的实现只查找第 1 个,忽略了第 2 个!

## ✅ 解决方案

### 修改 `find_tool_by_name` 实现双重查找

**文件**: `rust-lib/flowy-ai/src/mcp/manager.rs`

```rust
pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
    // 🔍 优先从工具注册表中查找(已连接的服务器)
    if let Some(result) = self.tool_discovery.find_tool_by_name(tool_name).await {
        tracing::info!("🔍 [FIND TOOL] Found '{}' in connected server '{}'", tool_name, result.0);
        return Some(result);
    }
    
    // 🔍 如果注册表中没有,从配置的缓存中查找
    tracing::info!("🔍 [FIND TOOL] Tool '{}' not in registry, searching cached tools...", tool_name);
    
    let all_servers = self.config_manager.get_all_servers();
    for server in all_servers {
        if let Some(cached_tools) = &server.cached_tools {
            for tool in cached_tools {
                if tool.name == tool_name {
                    tracing::info!("🔍 [FIND TOOL] Found '{}' in cached tools of server '{}'", 
                                 tool_name, server.id);
                    return Some((server.id.clone(), tool.clone()));
                }
            }
        }
    }
    
    tracing::warn!("🔍 [FIND TOOL] Tool '{}' not found in any server (registry or cache)", tool_name);
    None
}
```

### 查找逻辑

```
1. 先查工具注册表 (已连接的服务器)
   ↓ 找到 → 返回
   ↓ 未找到
   
2. 再查配置缓存 (所有已配置的服务器)
   ↓ 找到 → 返回
   ↓ 未找到
   
3. 返回 None
```

### 优势

1. **性能优先**: 优先使用已连接服务器的实时数据
2. **兜底保障**: 即使服务器未连接,也能从缓存中找到工具
3. **自动连接**: 找到后会在 `execute_mcp_tool` 中自动连接服务器
4. **详细日志**: 清楚地显示从哪里找到的工具

## 🧪 测试验证

### 测试场景 1: 服务器已连接

```
用户: "查看 excel 文件 myfile.xlsx 的内容"
```

**预期日志**:
```
🔍 [FIND TOOL] Found 'read_data_from_excel' in connected server 'excel-mcp'
✓ [MCP TOOL] Server 'excel-mcp' already connected
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel
✅ [TOOL EXEC] Tool executed successfully
```

### 测试场景 2: 服务器未连接但有缓存

```
用户: "查看 excel 文件 myfile.xlsx 的内容"
```

**预期日志**:
```
🔍 [FIND TOOL] Tool 'read_data_from_excel' not in registry, searching cached tools...
🔍 [FIND TOOL] Found 'read_data_from_excel' in cached tools of server 'excel-mcp'
🔌 [MCP AUTO-CONNECT] Server 'excel-mcp' is not connected, attempting to connect...
✅ [MCP AUTO-CONNECT] Successfully connected to server 'excel-mcp'
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel
✅ [TOOL EXEC] Tool executed successfully
```

### 测试场景 3: 工具真的不存在

```
用户: "使用不存在的工具"
AI: <tool_call>{"tool_name": "non_existent_tool"}</tool_call>
```

**预期日志**:
```
🔍 [FIND TOOL] Tool 'non_existent_tool' not in registry, searching cached tools...
⚠️  [FIND TOOL] Tool 'non_existent_tool' not found in any server (registry or cache)
❌ Tool call FAILED
Error: Native tool 'non_existent_tool' not yet implemented
```

## 📊 完整的工具调用流程

```
1. AI 生成工具调用
   ↓
2. 检测并解析 <tool_call> 标签
   ↓ (如果是 ```tool_call,自动转换)
   
3. execute_auto_detected_tool
   ↓
4. find_tool_by_name
   ↓ 先查注册表 → 未找到 → 再查缓存
   ↓ 找到工具信息
   
5. execute_mcp_tool
   ↓ 检查连接状态
   ↓ 如果未连接 → 自动连接
   
6. 调用 MCP 工具
   ↓
7. 返回结果给 UI
```

## 🎯 修复前后对比

### 修复前

```
[TOOL AUTO] Auto-detecting tool: read_data_from_excel
[TOOL AUTO] Tool 'read_data_from_excel' not found in any MCP server
❌ Tool call FAILED
Error: Native tool 'read_data_from_excel' not yet implemented
```

❌ 即使工具在缓存中,也找不到

### 修复后

```
[FIND TOOL] Tool 'read_data_from_excel' not in registry, searching cached tools...
[FIND TOOL] Found 'read_data_from_excel' in cached tools of server 'excel-mcp'
[MCP AUTO-CONNECT] Server 'excel-mcp' is not connected, attempting to connect...
✅ [MCP AUTO-CONNECT] Successfully connected to server 'excel-mcp'
[MCP TOOL] Calling MCP tool: read_data_from_excel
✅ Tool execution completed successfully
```

✅ 从缓存中找到工具,自动连接,成功执行

## 🔗 相关功能

### 1. 工具缓存机制

工具缓存在连接 MCP 服务器时自动更新:

```rust
// rust-lib/flowy-ai/src/mcp/manager.rs
pub async fn connect_server(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
    // ... 连接服务器 ...
    
    // 发现并缓存工具
    match self.tool_discovery.discover_tools(&config.id).await {
        Ok(tools) => {
            if let Err(e) = self.config_manager.save_tools_cache(&config.id, tools) {
                tracing::error!("Failed to save tools cache: {}", e);
            }
        }
    }
}
```

### 2. 自动连接机制

在 `execute_mcp_tool` 中自动连接未连接的服务器:

```rust
// rust-lib/flowy-ai/src/agent/tool_call_handler.rs
async fn execute_mcp_tool(&self, server_id: &str, request: &ToolCallRequest) -> FlowyResult<String> {
    // 自动连接检查
    if !self.mcp_manager.is_server_connected(server_id) {
        info!("🔌 [MCP AUTO-CONNECT] Server '{}' is not connected, attempting to connect...", server_id);
        
        match self.mcp_manager.connect_server_from_config(server_id).await {
            Ok(()) => {
                info!("✅ [MCP AUTO-CONNECT] Successfully connected to server '{}'", server_id);
            }
            Err(e) => {
                error!("❌ [MCP AUTO-CONNECT] Failed to connect to server '{}': {}", server_id, e);
                return Err(e);
            }
        }
    }
    
    // 调用工具...
}
```

## 📝 总结

通过修复 `find_tool_by_name` 方法,我们实现了:

1. ✅ **双重查找**: 注册表 + 缓存,确保能找到工具
2. ✅ **自动连接**: 找到工具后自动连接服务器
3. ✅ **详细日志**: 清晰展示查找和连接过程
4. ✅ **健壮性**: 即使服务器未连接,也能通过缓存找到并执行工具

现在整个工具调用流程应该能够顺利运行! 🎉

