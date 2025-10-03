# MCP Bloc Fold Await 修复

## 问题描述

删除MCP服务器时出现异常：
```
emit was called after an event handler completed normally.
This is usually due to an unawaited future in an event handler.
```

## 根本原因

在 `MCPSettingsBloc` 中，多个事件处理器错误地使用了 `await result.fold(...)`。

**问题代码模式：**
```dart
final result = await AIEventRemoveMCPServer(request).send();
await result.fold(  // ❌ 错误：fold是同步方法，不会等待async回调
  (success) async {
    await _loadServerListAndEmit(emit);  // 这个await不会被等待
  },
  (error) async {
    emit(...);
  },
);
```

**问题分析：**
- `fold` 是一个**同步方法**，即使在回调中使用 `async/await`，`fold` 本身也不会等待这些异步操作完成
- 导致事件处理器在异步操作完成之前就返回了
- 当异步操作稍后尝试调用 `emit` 时，事件处理器已经完成，触发断言错误

## 修复方案

将 `fold` 的结果先存储在变量中，然后根据结果执行后续的异步操作：

**正确的代码模式：**
```dart
final result = await AIEventRemoveMCPServer(request).send();

// 使用临时变量保存fold结果，然后执行异步操作
final isSuccess = result.fold(
  (success) {
    Log.info('操作成功');
    return true;
  },
  (error) {
    Log.error('操作失败: $error');
    return false;
  },
);

// 根据结果执行后续异步操作
if (isSuccess) {
  await _loadServerListAndEmit(emit);
} else {
  final errorMsg = result.fold(
    (success) => '',
    (error) => error.msg,
  );
  if (!emit.isDone) {  // 检查emit是否还可用
    emit(state.copyWith(
      isOperating: false,
      error: '操作失败: $errorMsg',
    ));
  }
}
```

## 修复的方法

以下方法已被修复：

1. ✅ `_handleAddServer` - 添加服务器
2. ✅ `_handleUpdateServer` - 更新服务器  
3. ✅ `_handleRemoveServer` - 删除服务器（原始问题）
4. ✅ `_handleDisconnectServer` - 断开服务器连接
5. ✅ `_handleStarted` - 初始化（第二次修复）
6. ✅ `_handleLoadServerList` - 加载服务器列表（第二次修复）

## 关键改进

1. **正确处理 fold 返回值**：先将结果存储在变量中
2. **安全的 emit 调用**：在调用 emit 前检查 `emit.isDone`
3. **清晰的异步流程**：异步操作在 fold 之后，而不是在其回调中

## 第二次修复（2025-10-01）

在删除服务器时又发现了相同的问题，但这次是在 `_handleLoadServerList` 和 `_handleStarted` 方法中。

### 问题代码
```dart
// ❌ 错误：调用 _loadServerList() 会添加新事件
Future<void> _handleLoadServerList(Emitter<MCPSettingsState> emit) async {
  emit(state.copyWith(isLoading: true, error: null));
  await _loadServerList();  // 这个方法内部会调用 add()
}

// _loadServerList 的问题实现
Future<void> _loadServerList() async {
  final result = await AIEventGetMCPServerList().send();
  result.fold(
    (servers) {
      if (!isClosed) {
        add(MCPSettingsEvent.didReceiveServerList(servers));  // ❌ 添加新事件
      }
    },
    ...
  );
}
```

### 修复方案
```dart
// ✅ 正确：使用 _loadServerListAndEmit 直接emit
Future<void> _handleLoadServerList(Emitter<MCPSettingsState> emit) async {
  emit(state.copyWith(isLoading: true, error: null));
  await _loadServerListAndEmit(emit);  // 直接emit，不添加新事件
}

Future<void> _handleStarted(Emitter<MCPSettingsState> emit) async {
  emit(state.copyWith(isLoading: true, error: null));
  await _loadServerListAndEmit(emit);  // 直接emit，不添加新事件
}

// 删除不再使用的 _loadServerList 方法
```

## 测试建议

测试以下场景确保修复有效：
- ✅ 删除MCP服务器
- ✅ 添加MCP服务器
- ✅ 更新MCP服务器
- ✅ 断开MCP服务器连接
- ✅ 页面初始化加载
- ✅ 手动刷新服务器列表

所有操作应该能正常完成，不再出现 "emit was called after an event handler completed" 错误。

## 文件位置

修复文件：`appflowy_flutter/lib/plugins/ai_chat/application/mcp_settings_bloc.dart`

## 日期

2025-10-01
