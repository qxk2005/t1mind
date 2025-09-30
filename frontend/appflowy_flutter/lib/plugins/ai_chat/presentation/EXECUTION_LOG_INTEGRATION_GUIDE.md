# 执行日志查看器集成指南

## 概述

执行日志查看器是一个完整的Flutter组件，用于展示智能体执行过程的详细日志。它支持实时更新、搜索过滤、高性能列表展示等功能。

## 🎯 主要功能

- ✅ **实时日志更新**: 支持自动刷新和实时显示新日志
- ✅ **搜索功能**: 支持关键词搜索并高亮显示匹配内容
- ✅ **过滤功能**: 按执行阶段和状态过滤日志
- ✅ **高性能展示**: 支持大量日志数据的虚拟滚动
- ✅ **详细信息**: 显示输入、输出、错误信息和执行时间
- ✅ **响应式设计**: 适配不同屏幕尺寸

## 📁 文件结构

```
lib/plugins/ai_chat/
├── application/
│   └── execution_log_bloc.dart              # 状态管理和事件调度
├── presentation/
│   ├── execution_log_viewer.dart            # 主查看器组件
│   ├── execution_log_integration_example.dart # 基础集成示例
│   ├── message/
│   │   └── ai_message_with_execution_logs.dart # 高级集成示例
│   └── EXECUTION_LOG_INTEGRATION_GUIDE.md   # 本文档
└── rust-lib/flowy-ai/src/
    ├── event_map.rs                         # 后端事件映射
    ├── agent/event_handler.rs               # 事件处理器
    ├── ai_manager.rs                        # AI管理器实现
    └── entities.rs                          # 数据实体定义
```

## 🚀 快速开始

### 1. 基础使用

最简单的使用方式是直接在页面中添加执行日志查看器：

```dart
import 'package:appflowy/plugins/ai_chat/presentation/execution_log_viewer.dart';

// 在你的Widget中使用
ExecutionLogViewer(
  sessionId: 'your_session_id',
  messageId: 'optional_message_id', // 可选，用于过滤特定消息的日志
  height: 400,
  showHeader: true,
)
```

### 2. 集成到聊天界面

#### 方式一：消息下方展开式

```dart
import 'package:appflowy/plugins/ai_chat/presentation/message/ai_message_with_execution_logs.dart';

// 在AI消息下方显示可展开的执行日志
AIMessageWithExecutionLogs(
  message: aiMessage,
  sessionId: chatSessionId,
  enableAnimation: true,
)
```

#### 方式二：消息气泡图标

```dart
// 在消息气泡右上角添加执行日志图标
SmartMessageBubble(
  message: aiMessage,
  sessionId: chatSessionId,
  child: YourMessageWidget(),
)
```

#### 方式三：底部面板

```dart
// 在聊天界面底部添加可切换的执行日志面板
ChatExecutionLogPanel(
  sessionId: chatSessionId,
  isVisible: showExecutionLogs,
  onToggle: () => setState(() => showExecutionLogs = !showExecutionLogs),
)
```

### 3. 完整页面集成

```dart
import 'package:appflowy/plugins/ai_chat/presentation/message/ai_message_with_execution_logs.dart';

// 完整的聊天页面示例
ChatPageWithExecutionLogs(
  sessionId: 'your_session_id',
)
```

## 🔧 API 参考

### ExecutionLogViewer

主要的执行日志查看器组件。

#### 参数

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `sessionId` | `String` | ✅ | - | 会话ID |
| `messageId` | `String?` | ❌ | `null` | 消息ID，用于过滤特定消息的日志 |
| `height` | `double` | ❌ | `400` | 查看器高度 |
| `showHeader` | `bool` | ❌ | `true` | 是否显示头部 |

#### 功能特性

- **搜索**: 在搜索框中输入关键词，自动高亮匹配内容
- **过滤**: 按执行阶段（规划、执行、工具调用、反思、完成）和状态（进行中、成功、失败、已取消）过滤
- **自动滚动**: 开启后自动显示最新日志并定时刷新
- **分页加载**: 支持大量日志的分页加载

### ExecutionLogBloc

状态管理组件，处理日志的加载、搜索、过滤等逻辑。

#### 事件

- `ExecutionLogEvent.loadLogs()`: 加载日志
- `ExecutionLogEvent.loadMoreLogs()`: 加载更多日志
- `ExecutionLogEvent.refreshLogs()`: 刷新日志
- `ExecutionLogEvent.searchLogs(String query)`: 搜索日志
- `ExecutionLogEvent.filterByPhase(ExecutionPhasePB? phase)`: 按阶段过滤
- `ExecutionLogEvent.filterByStatus(ExecutionStatusPB? status)`: 按状态过滤
- `ExecutionLogEvent.toggleAutoScroll(bool enabled)`: 切换自动滚动
- `ExecutionLogEvent.addLog(AgentExecutionLogPB log)`: 添加新日志

## 🔌 后端集成

### Rust后端实现

执行日志查看器的后端实现包括：

1. **事件映射** (`event_map.rs`):
   - `GetExecutionLogs`: 获取执行日志列表
   - `AddExecutionLog`: 添加执行日志
   - `ClearExecutionLogs`: 清空执行日志

2. **事件处理器** (`agent/event_handler.rs`):
   - `get_execution_logs_handler`: 处理日志获取请求
   - `add_execution_log_handler`: 处理日志添加请求
   - `clear_execution_logs_handler`: 处理日志清空请求

3. **AI管理器** (`ai_manager.rs`):
   - 内存中存储执行日志
   - 支持分页、过滤和搜索
   - 提供日志管理API

### 数据结构

```protobuf
// 执行日志项
message AgentExecutionLogPB {
  string id = 1;
  string session_id = 2;
  string message_id = 3;
  ExecutionPhasePB phase = 4;
  string step = 5;
  string input = 6;
  string output = 7;
  ExecutionStatusPB status = 8;
  int64 started_at = 9;
  int64 completed_at = 10;
  int64 duration_ms = 11;
  string error_message = 12;
}

// 执行阶段
enum ExecutionPhasePB {
  ExecPlanning = 0;
  ExecExecution = 1;
  ExecToolCall = 2;
  ExecReflection = 3;
  ExecCompletion = 4;
}

// 执行状态
enum ExecutionStatusPB {
  ExecRunning = 0;
  ExecSuccess = 1;
  ExecFailed = 2;
  ExecCancelled = 3;
}
```

## 🎨 自定义样式

执行日志查看器使用Flutter的主题系统，可以通过修改主题来自定义样式：

```dart
// 自定义颜色
Theme(
  data: Theme.of(context).copyWith(
    colorScheme: Theme.of(context).colorScheme.copyWith(
      surface: Colors.white,
      surfaceContainerLow: Colors.grey[50],
      surfaceContainerHighest: Colors.grey[100],
    ),
  ),
  child: ExecutionLogViewer(
    sessionId: sessionId,
  ),
)
```

## 🔍 入口位置

### 主要入口点

1. **直接使用**: `ExecutionLogViewer` 组件
   ```dart
   import 'package:appflowy/plugins/ai_chat/presentation/execution_log_viewer.dart';
   ```

2. **聊天集成**: `AIMessageWithExecutionLogs` 组件
   ```dart
   import 'package:appflowy/plugins/ai_chat/presentation/message/ai_message_with_execution_logs.dart';
   ```

3. **状态管理**: `ExecutionLogBloc`
   ```dart
   import 'package:appflowy/plugins/ai_chat/application/execution_log_bloc.dart';
   ```

### 集成到现有聊天界面

要将执行日志查看器集成到现有的聊天界面中，可以：

1. **修改AI消息组件**: 在现有的AI消息组件中添加执行日志按钮
2. **添加底部面板**: 在聊天界面底部添加执行日志面板
3. **使用弹出层**: 通过弹出层显示执行日志

### 示例集成代码

```dart
// 在现有的聊天页面中添加
class YourChatPage extends StatefulWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 现有的聊天消息列表
        Expanded(child: YourMessageList()),
        
        // 添加执行日志面板
        ChatExecutionLogPanel(
          sessionId: widget.sessionId,
          isVisible: _showLogs,
          onToggle: () => setState(() => _showLogs = !_showLogs),
        ),
        
        // 现有的输入框
        YourMessageInput(),
      ],
    );
  }
}
```

## 🐛 故障排除

### 常见问题

1. **日志不显示**: 检查sessionId是否正确，确保后端有相应的日志数据
2. **搜索不工作**: 确保搜索查询不为空，检查日志内容是否包含搜索关键词
3. **性能问题**: 对于大量日志，确保启用了分页加载
4. **样式问题**: 检查主题配置，确保颜色和字体设置正确

### 调试技巧

1. 启用BLoC的调试日志
2. 检查网络请求和响应
3. 使用Flutter Inspector查看组件树
4. 检查控制台错误信息

## 📚 更多资源

- [Flutter BLoC文档](https://bloclibrary.dev/)
- [AppFlowy开发文档](https://docs.appflowy.io/)
- [执行日志查看器源码](./execution_log_viewer.dart)
- [集成示例源码](./message/ai_message_with_execution_logs.dart)
