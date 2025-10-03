# MCP 工具缓存问题调查报告

## 问题描述

**报告时间**：2025-10-01 13:25  
**问题**：用户进入 MCP 设置时，只看到 MCP 服务器列表，但没有看到工具标签，即使之前已经检查并加载过工具。

**原始日志**：
```log
INFO flowy_ai::mcp::event_handler: Found 1 MCP server configurations
INFO flowy_ai::mcp::event_handler: Successfully retrieved 1 MCP servers
```

**关键观察**：
- 服务器配置成功加载（1个服务器）
- 但日志中**没有**关于缓存工具的信息
- 前端没有显示工具标签

## 已实现的功能

### 后端 Rust 代码

1. **数据结构**（`entities.rs`）
   ```rust
   pub struct MCPServerConfig {
       // ... 其他字段 ...
       #[serde(skip_serializing_if = "Option::is_none")]
       pub cached_tools: Option<Vec<MCPTool>>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub last_tools_check_at: Option<SystemTime>,
   }
   ```

2. **配置管理**（`config.rs`）
   - `save_tools_cache()` - 保存工具缓存
   - `get_cached_tools()` - 获取缓存工具
   - `get_server()` - 加载服务器配置（应包含缓存）

3. **连接管理**（`manager.rs`）
   - 连接成功后自动调用 `save_tools_cache()`

4. **事件处理**（`event_handler.rs`）
   - 将 Rust `MCPServerConfig` 转换为 Protobuf `MCPServerConfigPB`
   - 包含 `cached_tools` 和 `last_tools_check_at` 字段

### 前端 Dart 代码

1. **Protobuf 定义**
   - ✅ 已生成 `cachedTools` 字段
   - ✅ 已生成 `lastToolsCheckAt` 字段
   - ✅ 已生成 `hasCachedTools()` 方法
   - ✅ 已生成 `hasLastToolsCheckAt()` 方法

2. **UI 组件**（`workspace_mcp_settings_v2.dart`）
   ```dart
   final realTimeTools = state.serverTools[server.id];
   final cachedTools = server.hasCachedTools() 
       ? server.cachedTools.tools 
       : <MCPToolPB>[];
   final tools = realTimeTools ?? cachedTools;
   ```

## 问题诊断

### 可能的原因

#### 1. 数据未保存到数据库

**可能性**：中等

**原因**：
- `save_tools_cache()` 未被调用
- 保存过程中出错
- `SystemTime` 序列化失败

**验证方法**：
- 检查"一键检查"后的日志
- 应该看到 "tools cache successfully saved" 消息

#### 2. 数据保存了但加载失败

**可能性**：高

**原因**：
- `SystemTime` 反序列化失败
- `serde` 配置问题
- `store_preferences.get_object()` 失败
- 数据库损坏

**验证方法**：
- 检查加载时的日志
- 应该看到 "Server ... has X cached tools" 消息

#### 3. 数据加载成功但未转换到 Protobuf

**可能性**：低

**原因**：
- `event_handler.rs` 转换逻辑错误
- Protobuf 字段映射问题

**验证方法**：
- 检查 "Found X cached tools for server" 消息

#### 4. Protobuf 数据传递到前端但未显示

**可能性**：低

**原因**：
- `hasCachedTools()` 返回 false
- UI 组件条件渲染逻辑错误
- State management 问题

**验证方法**：
- 在前端添加调试日志
- 打印 `server.hasCachedTools()` 和 `server.cachedTools`

## 已添加的调试功能

### V2 调试增强（2025-10-01）

#### 1. 配置保存时
```rust
// 打印即将保存的完整 JSON
debug!("Saving server config JSON:\n{}", json_str);
```

#### 2. 配置加载时
```rust
// 打印加载的完整 JSON
debug!("Loaded server config JSON:\n{}", json_str);

// 详细的缓存工具信息
info!("✅ Server {} has {} cached tools", cfg.name, tools.len());
debug!("  Tool 1: {}", tool.name);
debug!("  Tool 2: {}", tool.name);
...
```

#### 3. 保存流程追踪
```rust
info!("Saving {} tools to cache for server: {}", tools.len(), server_id);
info!("Saving server config with {} cached tools", tools.len());
info!("✅ MCP server {} tools cache successfully saved with {} tools", server_id, tools.len());
```

#### 4. 状态标记
- ✅ 表示成功/有数据
- ⚠️ 表示警告/无数据
- ❌ 表示失败/错误

## 测试计划

### 步骤 1：添加服务器并检查工具

**操作**：
1. 打开 MCP 设置
2. 添加 MCP 服务器
3. 点击"一键检查"

**预期日志**：
```log
INFO: Discovered X tools for server: ...
INFO: Saving X tools to cache for server: ...
DEBUG: Saving server config JSON:
{
  ...
  "cached_tools": [ ... ],
  "last_tools_check_at": { ... }
}
INFO: ✅ MCP server ... tools cache successfully saved with X tools
```

### 步骤 2：重新加载（不关闭应用）

**操作**：
1. 关闭 MCP 设置页面
2. 重新打开 MCP 设置

**预期日志**：
```log
DEBUG: Loaded server config JSON:
{
  ...
  "cached_tools": [ ... ],  // ← 应该有数据
  "last_tools_check_at": { ... }  // ← 应该有时间戳
}
INFO: ✅ Server ... has X cached tools
INFO: ✅ Server ... last check time: ...
INFO: Found X cached tools for server: ...
```

### 步骤 3：完全重启应用

**操作**：
1. 完全关闭应用
2. 重新启动
3. 打开 MCP 设置

**预期**：与步骤 2 相同的日志

## 关键日志标记

请在新的日志输出中查找：

### 保存阶段
- [ ] `Saving N tools to cache for server`
- [ ] `Saving server config JSON:` 后面有完整的 JSON
- [ ] JSON 中 `"cached_tools"` 不是 `null`
- [ ] JSON 中 `"last_tools_check_at"` 不是 `null`
- [ ] `✅ tools cache successfully saved`

### 加载阶段
- [ ] `Loaded server config JSON:` 后面有完整的 JSON
- [ ] JSON 中 `"cached_tools"` 不是 `null`
- [ ] JSON 中 `"last_tools_check_at"` 不是 `null`
- [ ] `✅ Server ... has N cached tools`
- [ ] `✅ Server ... last check time: ...`

### 转换阶段
- [ ] `Found N cached tools for server: ...`
- [ ] `Found last check time for server ...`

### 前端显示
- [ ] 工具标签显示在服务器卡片上
- [ ] 显示"最后检查: XX分钟前"

## SystemTime 序列化问题

### 正确的格式

`SystemTime` 应该被序列化为：
```json
{
  "secs_since_epoch": 1727773200,
  "nanos_since_epoch": 123456789
}
```

### 检查方法

查看 `Saving server config JSON` 和 `Loaded server config JSON` 输出，确认：
1. `last_tools_check_at` 字段存在
2. 格式正确（包含 `secs_since_epoch` 和 `nanos_since_epoch`）
3. 保存和加载时的值相同

### 如果格式不正确

可能需要：
1. 添加自定义的 serde 序列化器
2. 使用 `chrono` 代替 `SystemTime`
3. 使用时间戳（i64）代替 `SystemTime`

## 前端调试

如果后端日志全部正常，但前端仍不显示，添加以下调试代码：

```dart
// 在 _buildServerList 方法中
...state.servers.map((server) {
  print('🔍 Debug Server: ${server.name}');
  print('  hasCachedTools: ${server.hasCachedTools()}');
  
  if (server.hasCachedTools()) {
    print('  cachedTools.tools.length: ${server.cachedTools.tools.length}');
    for (var tool in server.cachedTools.tools.take(3)) {
      print('    - ${tool.name}');
    }
  }
  
  print('  hasLastToolsCheckAt: ${server.hasLastToolsCheckAt()}');
  if (server.hasLastToolsCheckAt()) {
    print('  lastToolsCheckAt: ${server.lastToolsCheckAt}');
  }
  
  final realTimeTools = state.serverTools[server.id];
  final cachedTools = server.hasCachedTools() 
      ? server.cachedTools.tools 
      : <MCPToolPB>[];
  final tools = realTimeTools ?? cachedTools;
  
  print('  realTimeTools: ${realTimeTools?.length ?? 0}');
  print('  cachedTools: ${cachedTools.length}');
  print('  final tools: ${tools.length}');
  ...
});
```

## 下一步行动

1. **重新运行应用**
2. **执行完整测试流程**（步骤 1-3）
3. **收集所有日志**
4. **报告结果**，包括：
   - 是否看到 ✅ 标记
   - 是否看到 ⚠️ 或 ❌ 标记
   - JSON 输出的内容
   - 前端是否显示工具标签

## 相关文件

- `rust-lib/flowy-ai/src/mcp/config.rs` - 配置管理 + 详细日志
- `rust-lib/flowy-ai/src/mcp/event_handler.rs` - Protobuf 转换 + 日志
- `rust-lib/flowy-ai/src/mcp/manager.rs` - 连接管理
- `rust-lib/flowy-ai/src/mcp/entities.rs` - 数据结构
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/workspace_mcp_settings_v2.dart` - 前端 UI

## 调试文档

- [MCP_CACHE_DEBUG_GUIDE.md](./MCP_CACHE_DEBUG_GUIDE.md) - 基础调试指南
- [MCP_CACHE_DEBUG_V2.md](./MCP_CACHE_DEBUG_V2.md) - 详细调试指南（推荐）

---

**调查时间**：2025-10-01  
**调试版本**：V2 with full JSON logging  
**状态**：等待测试反馈



