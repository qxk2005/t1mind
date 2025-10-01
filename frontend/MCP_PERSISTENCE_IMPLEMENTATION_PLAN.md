# MCP数据持久化实现计划

## 当前状态

### ✅ 已完成（Rust后端）
1. **Protobuf定义** (`rust-lib/flowy-ai/src/entities.rs`)
   - `MCPServerConfigPB` - 服务器配置
   - `MCPTransportTypePB` - 传输类型枚举
   - `MCPStdioConfigPB` - STDIO配置
   - `MCPHttpConfigPB` - HTTP/SSE配置
   - `MCPServerListPB` - 服务器列表
   - `MCPServerStatusPB` - 服务器状态

2. **事件定义** (`rust-lib/flowy-ai/src/event_map.rs`)
   - `GetMCPServerList` = 37
   - `AddMCPServer` = 38
   - `UpdateMCPServer` = 39
   - `RemoveMCPServer` = 40
   - `ConnectMCPServer` = 41
   - `DisconnectMCPServer` = 42
   - `GetMCPServerStatus` = 43
   - `GetMCPToolList` = 44
   - `CallMCPTool` = 45

3. **事件处理器** (`rust-lib/flowy-ai/src/mcp/event_handler.rs`)
   - `get_mcp_server_list_handler` - ✅ 完整实现
   - `add_mcp_server_handler` - ✅ 完整实现
   - `update_mcp_server_handler` - ✅ 完整实现
   - `remove_mcp_server_handler` - ✅ 完整实现
   - `connect_mcp_server_handler` - ✅ 完整实现
   - `disconnect_mcp_server_handler` - ✅ 完整实现
   - `get_mcp_server_status_handler` - ✅ 完整实现

4. **配置管理器** (`rust-lib/flowy-ai/src/mcp/config.rs`)
   - `MCPConfigManager` - ✅ 完整实现
   - 使用`KVStorePreferences`进行SQLite存储
   - 支持CRUD操作、导出导入、验证等

### ✅ 已完成（Flutter前端）
1. **Protobuf绑定** 
   - 自动生成：`appflowy_flutter/packages/appflowy_backend/lib/protobuf/flowy-ai/entities.pb.dart`
   - 包含所有MCP相关的protobuf类

2. **BLoC实现** (`appflowy_flutter/lib/plugins/ai_chat/application/mcp_settings_bloc.dart`)
   - `MCPSettingsBloc` - ✅ 完整实现
   - 所有事件处理：添加、更新、删除、连接、断开连接
   - 状态管理：服务器列表、服务器状态、加载状态、错误处理

3. **事件发送器**
   - `AIEventGetMCPServerList`
   - `AIEventAddMCPServer`
   - `AIEventUpdateMCPServer`
   - `AIEventRemoveMCPServer`
   - `AIEventConnectMCPServer`
   - `AIEventDisconnectMCPServer`

### ⚠️ 需要修改的部分

#### `workspace_mcp_settings.dart`
**当前问题**：
- 使用了已删除的`MCPServerManager`
- 数据结构使用`Map<String, dynamic>`而不是`MCPServerConfigPB`
- 没有使用BLoC模式
- 没有真正调用后端API

**需要的改动**：
1. 移除所有对`MCPServerManager`的引用
2. 将`_WorkspaceMCPServerList`改为使用`BlocProvider<MCPSettingsBloc>`
3. 将所有数据类型从`Map<String, dynamic>`改为`MCPServerConfigPB`
4. 更新`_SimpleMCPDialog`以构建和保存`MCPServerConfigPB`对象
5. 使用`context.read<MCPSettingsBloc>()`来发送事件

## 实施步骤

### 步骤1：重构数据模型
```dart
// 从这个：
Map<String, dynamic> serverConfig = {
  'name': _nameController.text,
  'transport_type': _transportType,
  ...
};

// 改为这个：
MCPServerConfigPB config = MCPServerConfigPB()
  ..id = ''  // 后端会生成
  ..name = _nameController.text
  ..transportType = _convertTransportType(_transportType)
  ..isActive = true
  ..description = _descriptionController.text;

if (_transportType == 'STDIO') {
  config.stdioConfig = MCPStdioConfigPB()
    ..command = _commandController.text
    ..args.addAll(_arguments.map((a) => a['value'] as String))
    ..envVars.addAll(Map.fromEntries(
      _environmentVariables.map((e) => MapEntry(
        e['key'] as String,
        e['value'] as String,
      )),
    ));
} else {
  config.httpConfig = MCPHttpConfigPB()
    ..url = _urlController.text
    ..headers.addAll({});  // 可以添加headers
}
```

### 步骤2：集成BLoC
```dart
// 在 _WorkspaceMCPServerList 中：
@override
Widget build(BuildContext context) {
  return BlocProvider(
    create: (_) => MCPSettingsBloc()..add(const MCPSettingsEvent.started()),
    child: BlocBuilder<MCPSettingsBloc, MCPSettingsState>(
      builder: (context, state) {
        if (state.isLoading && state.servers.isEmpty) {
          return const Center(child: CircularProgressIndicator());
        }
        
        if (state.servers.isEmpty) {
          return _buildEmptyState(context);
        }
        
        return _buildServerList(context, state.servers);
      },
    ),
  );
}
```

### 步骤3：修改添加服务器逻辑
```dart
// 在 _SimpleMCPDialog 中：
void _saveServer() {
  final config = _buildServerConfig();
  context.read<MCPSettingsBloc>().add(
    MCPSettingsEvent.addServer(config),
  );
  Navigator.of(context).pop();
}
```

### 步骤4：修改删除服务器逻辑
```dart
// 在服务器卡片中：
onPressed: () {
  context.read<MCPSettingsBloc>().add(
    MCPSettingsEvent.removeServer(server.id),
  );
}
```

### 步骤5：添加错误处理
```dart
return BlocListener<MCPSettingsBloc, MCPSettingsState>(
  listener: (context, state) {
    if (state.error != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(state.error!),
          backgroundColor: Colors.red,
        ),
      );
    }
  },
  child: BlocBuilder<MCPSettingsBloc, MCPSettingsState>(...),
);
```

## 数据流

```
用户操作 (UI)
   ↓
MCPSettingsEvent (Flutter BLoC)
   ↓
AIEventXXX.send() (Dispatch Layer)
   ↓
Rust Event Handler
   ↓
MCPConfigManager (Persistence)
   ↓
KVStorePreferences (SQLite)
```

## 测试计划

1. **单元测试**
   - ✅ Rust配置管理器测试（已有）
   - ⚠️ Flutter BLoC测试（需添加）

2. **集成测试**
   - 添加服务器 → 重启应用 → 验证服务器仍存在
   - 更新服务器 → 验证更新成功
   - 删除服务器 → 验证服务器已删除
   - 导出配置 → 清空 → 导入 → 验证恢复

3. **UI测试**
   - 添加STDIO服务器
   - 添加HTTP服务器
   - 添加SSE服务器
   - 编辑服务器配置
   - 删除服务器
   - 连接/断开连接
   - 测试连接功能

## 辅助函数

```dart
// 将UI中的transport_type字符串转换为protobuf枚举
MCPTransportTypePB _convertTransportType(String type) {
  switch (type.toUpperCase()) {
    case 'STDIO':
      return MCPTransportTypePB.Stdio;
    case 'SSE':
      return MCPTransportTypePB.SSE;
    case 'HTTP':
      return MCPTransportTypePB.HTTP;
    default:
      return MCPTransportTypePB.Stdio;
  }
}

// 将protobuf枚举转换回字符串
String _transportTypeToString(MCPTransportTypePB type) {
  switch (type) {
    case MCPTransportTypePB.Stdio:
      return 'STDIO';
    case MCPTransportTypePB.SSE:
      return 'SSE';
    case MCPTransportTypePB.HTTP:
      return 'HTTP';
  }
}

// 从MCPServerConfigPB构建UI显示的Map
Map<String, dynamic> _serverConfigToMap(MCPServerConfigPB config) {
  return {
    'id': config.id,
    'name': config.name,
    'icon': config.icon,
    'transport_type': _transportTypeToString(config.transportType),
    'is_active': config.isActive,
    'description': config.description,
    if (config.hasStdioConfig()) ...{
      'command': config.stdioConfig.command,
      'args': config.stdioConfig.args,
      'env_vars': config.stdioConfig.envVars,
    },
    if (config.hasHttpConfig()) ...{
      'url': config.httpConfig.url,
      'headers': config.httpConfig.headers,
    },
  };
}
```

## 迁移策略

由于`workspace_mcp_settings.dart`文件太大（2161行），建议：

1. **保留现有UI结构** - 只改变数据层
2. **渐进式迁移**：
   - 第一步：添加BLoC provider
   - 第二步：修改数据模型转换
   - 第三步：替换CRUD操作
   - 第四步：测试验证
3. **兼容性**：保持UI不变，用户体验一致

## 预期结果

✅ 数据真正持久化到SQLite数据库
✅ 应用重启后配置保持
✅ 支持多工作区隔离（通过workspaceId）
✅ 完整的错误处理和状态反馈
✅ 类型安全（使用protobuf而不是Map）
✅ 符合AppFlowy架构标准


