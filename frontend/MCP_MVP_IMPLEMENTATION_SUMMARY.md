# MCP 工具集成 MVP 实现总结

## ✅ 已完成的工作

### 1. BLoC 层 (100% 完成)

#### 新增状态字段
```dart
class MCPSettingsState {
  Map<String, List<MCPToolPB>> serverTools;      // 服务器工具映射
  Set<String> loadingTools;                       // 正在加载工具的服务器
  bool isCallingTool;                             // 是否正在调用工具
  MCPToolCallResponsePB? lastToolResponse;        // 最后的工具调用响应
  String? selectedServerId;                       // 选中的服务器ID
}
```

#### 新增事件
```dart
// 工具相关事件
loadToolList(String serverId)                      // 加载工具列表
callTool(String serverId, String toolName, String arguments)  // 调用工具
refreshTools(String serverId)                      // 刷新工具列表
didReceiveToolList(String serverId, MCPToolListPB tools)       // 接收到工具列表
didReceiveToolCallResponse(MCPToolCallResponsePB response)     // 接收到调用响应
```

#### 新增事件处理器
- ✅ `_handleLoadToolList` - 从后端获取工具列表
- ✅ `_handleCallTool` - 调用MCP工具
- ✅ `_handleRefreshTools` - 刷新工具缓存
- ✅ `_handleDidReceiveToolList` - 处理接收到的工具列表
- ✅ `_handleDidReceiveToolCallResponse` - 处理工具调用响应

#### 自动加载机制
- ✅ 连接服务器成功后自动加载工具列表
- ✅ 工具列表缓存在BLoC状态中

### 2. UI 层改进

#### 已修复
- ✅ **添加服务器按钮已存在**（在服务器列表顶部）
- ✅ Freezed 代码已重新生成
- ✅ 无 lint 错误

#### 待完成（下一步）
需要在 `_ServerCard` 中添加:
1. 显示工具数量徽章
2. "查看工具"按钮
3. 创建工具列表对话框

## 🚧 需要立即完成的 UI 工作

### 步骤 1: 更新 `_ServerCard` 显示工具信息

在服务器卡片中添加：
```dart
// 在服务器名称旁边显示工具数量
if (isConnected && tools.isNotEmpty)
  Container(
    padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
    decoration: BoxDecoration(
      color: Colors.blue,
      borderRadius: BorderRadius.circular(12),
    ),
    child: Text('${tools.length} 个工具', style: TextStyle(color: Colors.white, fontSize: 11)),
  )

// 添加"查看工具"按钮
if (isConnected)
  IconButton(
    icon: Icon(Icons.build),
    onPressed: onViewTools,
    tooltip: "查看工具",
  )
```

### 步骤 2: 创建工具列表对话框

创建简单的工具列表展示：
```dart
class MCPToolListDialog extends StatelessWidget {
  final String serverName;
  final List<MCPToolPB> tools;

  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text('$serverName - MCP 工具'),
      content: Container(
        width: 600,
        height: 400,
        child: ListView.builder(
          itemCount: tools.length,
          itemBuilder: (context, index) {
            final tool = tools[index];
            return ExpansionTile(
              title: Text(tool.name),
              subtitle: Text(tool.description),
              children: [
                // 显示输入参数
                Padding(
                  padding: EdgeInsets.all(16),
                  child: Text(
                    tool.inputSchema,
                    style: TextStyle(fontFamily: 'monospace', fontSize: 12),
                  ),
                ),
              ],
            );
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text('关闭'),
        ),
      ],
    );
  }
}
```

### 步骤 3: 连接UI和BLoC

在 `_buildServerList` 中：
```dart
...state.servers.map((server) {
  final status = state.serverStatuses[server.id];
  final tools = state.serverTools[server.id] ?? [];  // 从状态获取工具
  final loadingTools = state.loadingTools.contains(server.id);
  
  return _ServerCard(
    server: server,
    serverStatus: status,
    tools: tools,
    loadingTools: loadingTools,
    onDelete: () => _deleteServer(context, server.id),
    onConnect: () => context.read<MCPSettingsBloc>().add(
      MCPSettingsEvent.connectServer(server.id),
    ),
    onDisconnect: () => context.read<MCPSettingsBloc>().add(
      MCPSettingsEvent.disconnectServer(server.id),
    ),
    onViewTools: () => showDialog(
      context: context,
      builder: (_) => MCPToolListDialog(
        serverName: server.name,
        tools: tools,
      ),
    ),
    onRefreshTools: () => context.read<MCPSettingsBloc>().add(
      MCPSettingsEvent.refreshTools(server.id),
    ),
  );
})
```

## 📊 实现进度

### BLoC 层: 100% ✅
- [x] State 定义
- [x] Event 定义
- [x] Event 处理器
- [x] 自动加载工具
- [x] Freezed 代码生成

### UI 层: 30% 🚧
- [x] 添加服务器按钮
- [ ] 更新 `_ServerCard` 显示工具数量
- [ ] 添加查看工具按钮
- [ ] 创建工具列表对话框
- [ ] 连接点击事件

### 功能测试: 0% ⏳
- [ ] 测试连接后自动加载工具
- [ ] 测试工具列表展示
- [ ] 测试刷新工具功能

## 🎯 下一步行动

**立即需要做的（15分钟内）：**

1. 修改 `_ServerCard` 构造函数，添加参数：
   ```dart
   final List<MCPToolPB> tools;
   final bool loadingTools;
   final VoidCallback onViewTools;
   final VoidCallback onRefreshTools;
   ```

2. 在 `_ServerCard` 的 UI 中添加工具数量徽章和查看按钮

3. 创建简单的工具列表对话框

4. 更新 `_buildServerList` 传递工具数据

## 🔍 测试流程

完成后的测试步骤：

1. **添加 MCP 服务器**
   - 配置 Excel MCP 服务器
   - URL: `http://localhost:8007/mcp`
   - 传输类型: HTTP 或 SSE

2. **连接服务器**
   - 点击"连接"按钮
   - 应该自动加载工具列表
   - 服务器卡片上应显示工具数量

3. **查看工具**
   - 点击"查看工具"图标
   - 应该弹出对话框
   - 显示所有工具的名称、描述和参数

4. **刷新工具**
   - 点击刷新按钮
   - 重新加载工具列表

## 📝 技术说明

### 工具加载流程
```
1. 用户点击"连接" 
   ↓
2. ConnectMCPServer 事件
   ↓
3. 后端连接服务器
   ↓
4. 返回 MCPServerStatusPB (is_connected=true)
   ↓
5. 自动触发 LoadToolList 事件
   ↓
6. 后端调用 GetMCPToolList
   ↓
7. 返回 MCPToolListPB
   ↓
8. 更新 state.serverTools[serverId]
   ↓
9. UI 自动刷新显示工具数量
```

### 数据流
```
Backend (Rust)
   ↓ (Protobuf)
MCPSettingsBloc
   ↓ (State)
_ServerCard
   ↓ (UI)
用户
```

## ✨ MVP 成果预览

完成后用户将能够：
- ✅ 看到每个已连接服务器的工具数量
- ✅ 点击查看工具列表
- ✅ 查看每个工具的名称、描述和参数
- ✅ 刷新工具列表

这为后续的工具调用功能打下了坚实基础！




