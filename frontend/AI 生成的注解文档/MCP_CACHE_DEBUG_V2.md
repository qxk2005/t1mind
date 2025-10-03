# MCP 工具缓存调试指南 V2

## 问题现状

用户在进入 MCP 设置时，只看到 MCP 服务器列表，但没有看到工具标签，尽管之前已经检查并加载过工具。

## 最新调试日志（V2）

已添加非常详细的日志，包括：

### 1. 保存配置时的完整 JSON
```rust
debug!("Saving server config JSON:\n{}", json_str);
```

### 2. 加载配置时的完整 JSON
```rust
debug!("Loaded server config JSON:\n{}", json_str);
```

### 3. 工具缓存详情
```rust
info!("✅ Server {} has {} cached tools", cfg.name, tools.len());
debug!("  Tool 1: {}", tool.name);
debug!("  Tool 2: {}", tool.name);
// ...
```

### 4. 保存流程追踪
```rust
info!("Saving {} tools to cache for server: {}", tools.len(), server_id);
info!("Saving server config with {} cached tools", tools.len());
info!("✅ MCP server {} tools cache successfully saved with {} tools", server_id, tools.len());
```

## 完整测试流程

### 步骤 1：清除现有缓存（可选）
如果想从头开始测试，删除现有的 MCP 服务器。

### 步骤 2：添加服务器并检查工具

1. 打开应用，进入 MCP 设置
2. 添加一个 MCP 服务器（例如 Excel MCP）
3. 点击"一键检查"按钮
4. 观察控制台输出

**预期日志输出**：

```log
# 1. 工具发现
INFO flowy_ai::mcp::manager: Discovered 15 tools for server: Excel MCP

# 2. 保存缓存
INFO flowy_ai::mcp::config: Saving 15 tools to cache for server: mcp_1234567890
DEBUG flowy_ai::mcp::config: Loaded server config: Excel MCP (id: mcp_1234567890)
DEBUG flowy_ai::mcp::config: Loaded server config JSON:
{
  "id": "mcp_1234567890",
  "name": "Excel MCP",
  ...
  "cached_tools": null,  // ← 注意这里是 null（保存前）
  "last_tools_check_at": null
}

# 3. 更新配置
INFO flowy_ai::mcp::config: Saving server config with 15 cached tools
DEBUG flowy_ai::mcp::config: Saving server config JSON:
{
  "id": "mcp_1234567890",
  "name": "Excel MCP",
  ...
  "cached_tools": [        // ← 这里应该有数据
    {
      "name": "read_data_from_excel",
      "description": "...",
      ...
    },
    ...
  ],
  "last_tools_check_at": {   // ← 这里应该有时间戳
    "secs_since_epoch": 1727773200,
    "nanos_since_epoch": 0
  }
}

# 4. 保存成功
INFO flowy_ai::mcp::config: MCP server config saved: Excel MCP (mcp_1234567890)
INFO flowy_ai::mcp::config: ✅ MCP server mcp_1234567890 tools cache successfully saved with 15 tools
```

### 步骤 3：重新加载（不关闭应用）

1. 关闭 MCP 设置页面
2. 重新打开 MCP 设置
3. **仔细观察日志**

**预期日志输出**：

```log
# 1. 加载配置
DEBUG flowy_ai::mcp::config: Loaded server config: Excel MCP (id: mcp_1234567890)

# 2. JSON 内容
DEBUG flowy_ai::mcp::config: Loaded server config JSON:
{
  "id": "mcp_1234567890",
  "name": "Excel MCP",
  ...
  "cached_tools": [        // ← ⭐ 关键：这里应该有数据
    {
      "name": "read_data_from_excel",
      ...
    },
    ...
  ],
  "last_tools_check_at": {   // ← ⭐ 关键：这里应该有时间戳
    "secs_since_epoch": 1727773200,
    "nanos_since_epoch": 0
  }
}

# 3. 缓存确认
INFO flowy_ai::mcp::config: ✅ Server Excel MCP has 15 cached tools
DEBUG flowy_ai::mcp::config:   Tool 1: read_data_from_excel
DEBUG flowy_ai::mcp::config:   Tool 2: write_data_to_excel
DEBUG flowy_ai::mcp::config:   Tool 3: apply_formula
DEBUG flowy_ai::mcp::config:   ... and 12 more tools
INFO flowy_ai::mcp::config: ✅ Server Excel MCP last check time: 1727773200 seconds since epoch

# 4. 事件处理器转换
INFO flowy_ai::mcp::event_handler: Found 15 cached tools for server: Excel MCP
INFO flowy_ai::mcp::event_handler: Found last check time for server Excel MCP: ... (timestamp: 1727773200)
```

### 步骤 4：完全重启应用

1. 完全关闭应用
2. 重新启动应用
3. 打开 MCP 设置
4. **观察日志**（应该与步骤 3 相同）

## 问题诊断表

### 问题 A：保存时 JSON 中没有 `cached_tools`

**症状**：
```json
"cached_tools": null  // ← 即使在"Saving server config with N cached tools"之后
```

**可能原因**：
1. `config.cached_tools` 在保存前被设置为 `None`
2. 代码逻辑错误

**排查**：
- 检查 `save_tools_cache` 方法中的 `config.cached_tools = Some(tools.clone())` 是否执行
- 在该行后添加 `assert!` 验证

### 问题 B：保存时 JSON 有数据，但加载时为 null

**症状**：
```json
// 保存时
"cached_tools": [ ... ]

// 加载时
"cached_tools": null
```

**可能原因**：
1. 序列化/反序列化问题
2. `SystemTime` 序列化失败导致整个配置加载失败
3. `skip_serializing_if` 导致字段被跳过（但这不应该发生，因为是 `Some`）

**排查**：
- 检查 `set_object` 的实现
- 检查 `get_object` 的实现
- 尝试直接查看数据库中的原始数据

### 问题 C：加载时 JSON 有数据，但前端未显示

**症状**：
```log
INFO: Server Excel MCP has 15 cached tools
INFO: Found 15 cached tools for server: Excel MCP
```
但前端 UI 没有显示工具标签。

**可能原因**：
1. Dart Protobuf 转换问题
2. `hasCachedTools()` 返回 false
3. UI 组件逻辑问题
4. State management 问题

**排查**：
- 检查 Dart 端日志
- 在前端添加日志打印 `server.hasCachedTools()`
- 检查 UI 组件的条件渲染

### 问题 D：SystemTime 序列化格式问题

**正确的 SystemTime JSON 格式**：
```json
"last_tools_check_at": {
  "secs_since_epoch": 1727773200,
  "nanos_since_epoch": 123456789
}
```

如果看到其他格式（如字符串或数字），说明 SystemTime 的序列化有问题。

## 重要检查点

请在日志中查找以下关键标记：

1. ✅ `Saving 15 tools to cache` - 开始保存
2. ✅ `Saving server config with 15 cached tools` - 准备保存配置
3. ✅ `"cached_tools": [ ... ]` - JSON 中包含工具数组
4. ✅ `"last_tools_check_at": { ... }` - JSON 中包含时间戳
5. ✅ `tools cache successfully saved` - 保存成功
6. ✅ `Server ... has 15 cached tools` - 加载成功
7. ✅ `Found 15 cached tools for server` - 转换成功

如果任何一步失败，请提供：
- 失败步骤的完整日志
- 上一步的日志（以便追踪状态变化）

## 如果所有后端日志都正常

如果所有后端日志都显示：
- ✅ 保存成功
- ✅ 加载成功
- ✅ 转换成功

但前端仍然没有显示工具标签，那么问题在前端 Dart 代码中。

请检查：

### 1. Dart 端是否收到缓存数据

在 `workspace_mcp_settings_v2.dart` 的 `_buildServerList` 方法中添加：

```dart
...state.servers.map((server) {
  print('🔍 Server: ${server.name}');
  print('  hasCachedTools: ${server.hasCachedTools()}');
  if (server.hasCachedTools()) {
    print('  cached tools count: ${server.cachedTools.tools.length}');
  }
  print('  hasLastToolsCheckAt: ${server.hasLastToolsCheckAt()}');
  if (server.hasLastToolsCheckAt()) {
    print('  last check at: ${server.lastToolsCheckAt}');
  }
  
  final realTimeTools = state.serverTools[server.id];
  final cachedTools = server.hasCachedTools() ? server.cachedTools.tools : <MCPToolPB>[];
  final tools = realTimeTools ?? cachedTools;
  print('  final tools count: ${tools.length}');
  ...
});
```

### 2. 检查 State Management

```dart
BlocConsumer<MCPSettingsBloc, MCPSettingsState>(
  builder: (context, state) {
    print('🔍 MCPSettingsState:');
    print('  servers count: ${state.servers.length}');
    print('  serverTools: ${state.serverTools}');
    ...
  },
)
```

## 下一步行动

1. **重新运行应用**
2. **执行"一键检查"**
3. **复制完整的日志输出**（从"开始加载MCP服务器列表"到看到服务器卡片）
4. **特别关注**：
   - 保存时的 JSON
   - 加载时的 JSON
   - ✅ 和 ⚠️ 标记

---

**调试版本**：V2 - 2025-10-01  
**特点**：完整 JSON 输出 + 详细状态追踪



