# MCP Bloc Emit 错误最终修复

## 问题描述

在用户简化代码时，移除了所有 `emit.isDone` 检查，导致在 `await` 之后调用 `emit()` 时出现以下错误：

```
emit was called after an event handler completed normally.
```

## 根本原因

**关键规则**：在 Bloc 事件处理器中，任何 `await` 异步操作之后调用 `emit()`，都**必须**先检查 `emit.isDone`。

这是因为：
1. `await` 之后，事件处理器可能已经被取消或完成
2. `result.fold()` 虽然是同步执行，但它在 `await` 之后
3. 在 `fold` 回调中调用 `emit()` 时，必须确保 emit 仍然有效

## 修复方案

### 统一修复模式

对所有包含 `await` 的异步方法，应用以下模式：

```dart
Future<void> _handleSomeEvent(Emitter<MCPSettingsState> emit) async {
  // 1. await 之前的 emit 不需要检查
  emit(state.copyWith(...));
  
  try {
    // 2. 执行异步操作
    final result = await someAsyncOperation();
    
    // 3. await 之后，在 fold 之前检查 emit.isDone
    if (emit.isDone) return;
    
    // 4. 在 fold 回调中可以安全使用 emit 或 add
    result.fold(
      (success) {
        // 使用 add 触发新事件（推荐）
        add(SomeEvent(success));
        // 或者使用 emit（已经检查过 isDone）
        emit(state.copyWith(...));
      },
      (error) {
        // 直接 emit（已经检查过 isDone）
        emit(state.copyWith(error: error.msg));
      },
    );
  } catch (e) {
    // 5. catch 块中的 emit 必须检查 isDone
    if (!emit.isDone) {
      emit(state.copyWith(error: '$e'));
    }
  }
}
```

### 修复的方法列表

已修复以下所有方法：

1. ✅ `_loadServerListAndEmit` - 加载服务器列表
2. ✅ `_handleConnectServer` - 连接服务器
3. ✅ `_handleTestConnection` - 测试连接
4. ✅ `_handleLoadToolList` - 加载工具列表
5. ✅ `_handleCallTool` - 调用工具

## 关键要点

### ✅ 正确做法

```dart
// await 之后立即检查
final result = await operation();
if (emit.isDone) return;

result.fold(
  (success) => emit(...),  // 安全
  (error) => emit(...),    // 安全
);
```

### ❌ 错误做法

```dart
// 没有检查就直接 fold
final result = await operation();
result.fold(
  (success) => emit(...),  // 可能失败！
  (error) => emit(...),    // 可能失败！
);
```

### 特殊情况：使用 add() vs emit()

- **使用 `add()`**：触发新事件，不需要检查 `emit.isDone`
- **使用 `emit()`**：直接更新状态，必须检查 `emit.isDone`

```dart
result.fold(
  (success) {
    add(NewEvent(success));  // ✅ 不需要检查
  },
  (error) {
    emit(state.copyWith(...));  // ✅ 已检查 isDone
  },
);
```

## 测试建议

1. 打开 MCP 设置页面
2. 测试以下操作：
   - ✅ 加载服务器列表
   - ✅ 添加新服务器
   - ✅ 连接服务器
   - ✅ 删除服务器
   - ✅ 一键检查所有服务器
   - ✅ 加载工具列表

3. 观察控制台，确保没有 emit 相关错误

## 修复时间

2025-10-01

## 相关文档

- [MCP_EVENT_HANDLER_IMPROVEMENTS.md](./MCP_EVENT_HANDLER_IMPROVEMENTS.md) - 之前的修复
- [MCP_DELETE_FIX.md](./MCP_DELETE_FIX.md) - 删除功能修复
- [MCP_FOLD_AWAIT_FIX.md](./MCP_FOLD_AWAIT_FIX.md) - Fold 异步修复

