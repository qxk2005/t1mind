# MCP 工具缓存调试指南

## 问题描述

用户报告在进入 MCP 设置时，MCP 服务器列表中没有显示工具标签，即使之前已经检查并加载过工具。

## 已添加的调试日志

为了追踪问题，我在以下位置添加了详细的日志：

### 1. 配置加载时（`config.rs`）

**`get_server()` 方法**：
```rust
pub fn get_server(&self, server_id: &str) -> Option<MCPServerConfig> {
    let config = self.store_preferences.get_object::<MCPServerConfig>(&key);
    
    if let Some(ref cfg) = config {
        debug!("Loaded server config: {} (id: {})", cfg.name, cfg.id);
        
        // 检查缓存工具
        if let Some(ref tools) = cfg.cached_tools {
            info!("Server {} has {} cached tools", cfg.name, tools.len());
        } else {
            debug!("Server {} has no cached tools", cfg.name);
        }
        
        // 检查最后检查时间
        if let Some(check_time) = cfg.last_tools_check_at {
            info!("Server {} last check time: {} seconds since epoch", cfg.name, ...);
        } else {
            debug!("Server {} has no last check time", cfg.name);
        }
    }
    
    config
}
```

**`save_tools_cache()` 方法**：
```rust
pub fn save_tools_cache(&self, server_id: &str, tools: Vec<MCPTool>) -> FlowyResult<()> {
    info!("Saving {} tools to cache for server: {}", tools.len(), server_id);
    // ...
    info!("Saving server config with {} cached tools", tools.len());
    self.save_server(config)?;
    info!("✅ MCP server {} tools cache successfully saved with {} tools", server_id, tools.len());
    Ok(())
}
```

### 2. 事件处理器（`event_handler.rs`）

**`get_mcp_server_list_handler()` 方法**：
```rust
// 转换缓存的工具列表
let cached_tools = config.cached_tools.map(|tools| {
    info!("Found {} cached tools for server: {}", tools.len(), config.name);
    // ...
});

if cached_tools.is_none() {
    debug!("No cached tools found for server: {}", config.name);
}

// 转换最后检查时间
let last_tools_check_at = config.last_tools_check_at.and_then(|time| {
    let timestamp = ...;
    if let Some(ts) = timestamp {
        info!("Found last check time for server {}: {} (timestamp: {})", ...);
    }
    timestamp
});

if last_tools_check_at.is_none() {
    debug!("No last check time found for server: {}", config.name);
}
```

## 测试步骤

### 步骤 1：清理测试环境（可选）

如果想从头开始测试，可以删除现有的 MCP 服务器配置。

### 步骤 2：添加并连接 MCP 服务器

1. 打开应用，进入 MCP 设置
2. 添加一个 MCP 服务器（例如 Excel MCP）
3. 点击"一键检查"按钮
4. 观察日志输出

**预期日志**：
```
INFO flowy_ai::mcp::manager: Discovered X tools for server: [服务器名]
INFO flowy_ai::mcp::config: Saving X tools to cache for server: [服务器ID]
INFO flowy_ai::mcp::config: Saving server config with X cached tools
INFO flowy_ai::mcp::config: ✅ MCP server [服务器ID] tools cache successfully saved with X tools
```

### 步骤 3：重新加载服务器列表

1. 关闭 MCP 设置页面
2. 重新打开 MCP 设置
3. **观察日志输出**

**预期日志 A（如果缓存成功）**：
```
INFO flowy_ai::mcp::config: Server [服务器名] has X cached tools
INFO flowy_ai::mcp::config: Server [服务器名] last check time: XXXXXX seconds since epoch
INFO flowy_ai::mcp::event_handler: Found X cached tools for server: [服务器名]
INFO flowy_ai::mcp::event_handler: Found last check time for server [服务器名]: ...
```

**预期日志 B（如果缓存失败）**：
```
DEBUG flowy_ai::mcp::config: Server [服务器名] has no cached tools
DEBUG flowy_ai::mcp::config: Server [服务器名] has no last check time
DEBUG flowy_ai::mcp::event_handler: No cached tools found for server: [服务器名]
DEBUG flowy_ai::mcp::event_handler: No last check time found for server: [服务器名]
```

### 步骤 4：完全重启应用

1. 完全关闭应用
2. 重新启动应用
3. 打开 MCP 设置
4. **观察日志输出**（应该与步骤 3 相同）

## 可能的问题诊断

### 情况 1：保存时没有日志

**症状**：
- 点击"一键检查"后没有看到 "Saving X tools to cache" 日志

**原因**：
- 工具发现失败
- `save_tools_cache` 没有被调用

**解决**：
- 检查工具发现的日志
- 检查是否有错误信息

### 情况 2：保存成功但加载时没有缓存

**症状**：
- 看到 "✅ MCP server ... tools cache successfully saved" 日志
- 但重新加载时看到 "Server ... has no cached tools" 日志

**原因**：
- 序列化/反序列化问题
- `serde` 配置问题
- `skip_serializing_if` 导致字段未保存

**解决**：
- 检查 `MCPServerConfig` 的 `serde` 属性
- 确认 `cached_tools` 和 `last_tools_check_at` 没有被跳过

### 情况 3：加载成功但前端未显示

**症状**：
- 看到 "Found X cached tools for server" 日志
- 但前端 UI 没有显示工具标签

**原因**：
- Protobuf 转换问题
- Dart 代码未正确读取 `oneof` 字段
- UI 组件逻辑问题

**解决**：
- 检查 Dart 端是否调用了 `hasCachedTools()`
- 检查 UI 组件的条件渲染逻辑

## 检查点清单

在重新测试之前，确认以下事项：

- [ ] ✅ Rust 代码已重新编译（使用 `cargo build --features dart`）
- [ ] ✅ Dart Protobuf 代码已重新生成
- [ ] ✅ 应用已重新启动（完全关闭后重新打开）
- [ ] ✅ 日志级别设置为 `INFO` 或 `DEBUG`

## 数据库检查（高级）

如果需要直接检查数据库中的数据，可以：

1. 找到 SQLite 数据库文件位置
2. 使用 SQLite 工具查看存储的配置
3. 检查 JSON 字段是否包含 `cached_tools` 和 `last_tools_check_at`

## 预期结果

完成调试后，应该能看到：

1. **保存阶段**：
   ```
   ✅ MCP server xxx tools cache successfully saved with N tools
   ```

2. **加载阶段**：
   ```
   Server xxx has N cached tools
   Server xxx last check time: XXXXXX seconds since epoch
   Found N cached tools for server: xxx
   ```

3. **前端显示**：
   - 工具标签显示在服务器卡片上
   - 显示"最后检查: XX分钟前"

## 下一步行动

请按照上述步骤重新测试，并提供以下信息：

1. **完整的日志输出**（从打开 MCP 设置到看到服务器列表）
2. **是否看到缓存保存的日志**
3. **是否看到缓存加载的日志**
4. **前端是否显示工具标签**

---

**编译日期**：2025-10-01  
**调试版本**：v1.0 with detailed logging


