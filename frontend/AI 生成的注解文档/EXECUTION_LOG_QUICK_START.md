# 执行日志功能快速开始指南 🚀

## 概述

本指南将帮助你快速将执行日志查看器集成到聊天界面中。

## 前置条件 ✅

所有必需的组件都已经实现：
- ✅ 后端 API (`GetExecutionLogs`, `AddExecutionLog`, `ClearExecutionLogs`)
- ✅ 前端 BLoC (`ExecutionLogBloc`)
- ✅ UI 组件 (`ExecutionLogViewer`)
- ✅ 数据模型 (`AgentExecutionLogPB`)

## 快速集成步骤

### 方式 1：对话框形式

在聊天消息旁边添加一个"查看日志"按钮，点击后弹出对话框：

```dart
// 在消息气泡的操作栏中添加
IconButton(
  icon: const Icon(Icons.history),
  tooltip: '查看执行日志',
  onPressed: () {
    showDialog(
      context: context,
      builder: (context) => Dialog(
        child: Container(
          width: 800,
          height: 600,
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text(
                    '执行日志',
                    style: TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
              const Divider(),
              Expanded(
                child: BlocProvider(
                  create: (context) => ExecutionLogBloc(
                    sessionId: chatId,  // 聊天ID
                    messageId: messageId.toString(),  // 消息ID
                  )..add(const ExecutionLogEvent.loadLogs()),
                  child: ExecutionLogViewer(
                    sessionId: chatId,
                    messageId: messageId.toString(),
                    height: 500,
                    showHeader: false,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  },
)
```

### 方式 2：侧边栏形式

在聊天界面右侧添加一个可折叠的日志面板：

```dart
class ChatWithLogsPage extends StatefulWidget {
  final String chatId;
  
  const ChatWithLogsPage({Key? key, required this.chatId}) : super(key: key);
  
  @override
  State<ChatWithLogsPage> createState() => _ChatWithLogsPageState();
}

class _ChatWithLogsPageState extends State<ChatWithLogsPage> {
  bool _showLogs = false;
  String? _selectedMessageId;
  
  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        // 聊天主界面
        Expanded(
          flex: _showLogs ? 2 : 1,
          child: ChatMessagesWidget(
            chatId: widget.chatId,
            onMessageSelected: (messageId) {
              setState(() {
                _selectedMessageId = messageId;
                _showLogs = true;
              });
            },
          ),
        ),
        
        // 日志侧边栏
        if (_showLogs && _selectedMessageId != null)
          Expanded(
            flex: 1,
            child: Container(
              decoration: BoxDecoration(
                border: Border(
                  left: BorderSide(
                    color: Theme.of(context).dividerColor,
                    width: 1,
                  ),
                ),
              ),
              child: Column(
                children: [
                  // 头部
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.surfaceContainerHighest,
                      border: Border(
                        bottom: BorderSide(
                          color: Theme.of(context).dividerColor,
                        ),
                      ),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.timeline),
                        const SizedBox(width: 8),
                        const Text(
                          '执行日志',
                          style: TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        const Spacer(),
                        IconButton(
                          icon: const Icon(Icons.close),
                          onPressed: () {
                            setState(() {
                              _showLogs = false;
                            });
                          },
                        ),
                      ],
                    ),
                  ),
                  
                  // 日志查看器
                  Expanded(
                    child: BlocProvider(
                      create: (context) => ExecutionLogBloc(
                        sessionId: widget.chatId,
                        messageId: _selectedMessageId,
                      )..add(const ExecutionLogEvent.loadLogs()),
                      child: ExecutionLogViewer(
                        sessionId: widget.chatId,
                        messageId: _selectedMessageId,
                        showHeader: false,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}
```

### 方式 3：底部抽屉形式

在聊天界面底部添加一个可向上滑动的抽屉：

```dart
class ChatWithBottomLogs extends StatefulWidget {
  final String chatId;
  
  const ChatWithBottomLogs({Key? key, required this.chatId}) : super(key: key);
  
  @override
  State<ChatWithBottomLogs> createState() => _ChatWithBottomLogsState();
}

class _ChatWithBottomLogsPageState extends State<ChatWithBottomLogs> {
  final DraggableScrollableController _controller = DraggableScrollableController();
  String? _selectedMessageId;
  
  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        // 聊天主界面
        ChatMessagesWidget(
          chatId: widget.chatId,
          onMessageSelected: (messageId) {
            setState(() {
              _selectedMessageId = messageId;
            });
            // 展开抽屉
            _controller.animateTo(
              0.7,
              duration: const Duration(milliseconds: 300),
              curve: Curves.easeOut,
            );
          },
        ),
        
        // 可拖动的日志抽屉
        DraggableScrollableSheet(
          controller: _controller,
          initialChildSize: 0.1,
          minChildSize: 0.1,
          maxChildSize: 0.9,
          snap: true,
          snapSizes: const [0.1, 0.5, 0.9],
          builder: (context, scrollController) {
            return Container(
              decoration: BoxDecoration(
                color: Theme.of(context).scaffoldBackgroundColor,
                borderRadius: const BorderRadius.vertical(
                  top: Radius.circular(16),
                ),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withOpacity(0.1),
                    blurRadius: 8,
                    offset: const Offset(0, -2),
                  ),
                ],
              ),
              child: Column(
                children: [
                  // 拖动手柄
                  Container(
                    margin: const EdgeInsets.symmetric(vertical: 8),
                    width: 40,
                    height: 4,
                    decoration: BoxDecoration(
                      color: Colors.grey[300],
                      borderRadius: BorderRadius.circular(2),
                    ),
                  ),
                  
                  // 日志查看器
                  if (_selectedMessageId != null)
                    Expanded(
                      child: BlocProvider(
                        create: (context) => ExecutionLogBloc(
                          sessionId: widget.chatId,
                          messageId: _selectedMessageId,
                        )..add(const ExecutionLogEvent.loadLogs()),
                        child: ExecutionLogViewer(
                          sessionId: widget.chatId,
                          messageId: _selectedMessageId,
                        ),
                      ),
                    )
                  else
                    const Expanded(
                      child: Center(
                        child: Text('选择一条消息以查看执行日志'),
                      ),
                    ),
                ],
              ),
            );
          },
        ),
      ],
    );
  }
  
  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
```

## 消息气泡集成示例

在现有的 AI 消息气泡中添加日志按钮：

```dart
// 在 AITextMessage widget 中
Widget _buildMessageActions(BuildContext context) {
  return Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      // 现有的操作按钮（复制、重新生成等）
      // ...
      
      // 新增：查看执行日志按钮
      if (message.hasAgentId()) // 只对智能体消息显示
        Tooltip(
          message: '查看执行日志',
          child: InkWell(
            onTap: () => _showExecutionLogs(context),
            borderRadius: BorderRadius.circular(4),
            child: Padding(
              padding: const EdgeInsets.all(4),
              child: Icon(
                Icons.history,
                size: 16,
                color: Theme.of(context).hintColor,
              ),
            ),
          ),
        ),
    ],
  );
}

void _showExecutionLogs(BuildContext context) {
  showDialog(
    context: context,
    builder: (context) => Dialog(
      child: Container(
        width: 800,
        height: 600,
        child: BlocProvider(
          create: (context) => ExecutionLogBloc(
            sessionId: chatId,
            messageId: message.messageId.toString(),
          )..add(const ExecutionLogEvent.loadLogs()),
          child: ExecutionLogViewer(
            sessionId: chatId,
            messageId: message.messageId.toString(),
          ),
        ),
      ),
    ),
  );
}
```

## 使用提示

### 1. 过滤日志

```dart
// 按执行阶段过滤
bloc.add(ExecutionLogEvent.filterByPhase(ExecutionPhasePB.ExecToolCall));

// 按执行状态过滤
bloc.add(ExecutionLogEvent.filterByStatus(ExecutionStatusPB.ExecSuccess));

// 清除过滤
bloc.add(const ExecutionLogEvent.filterByPhase(null));
```

### 2. 搜索日志

```dart
// 搜索包含特定关键词的日志
bloc.add(ExecutionLogEvent.searchLogs('工具调用'));

// 清除搜索
bloc.add(const ExecutionLogEvent.searchLogs(''));
```

### 3. 自动刷新

```dart
// 启用自动刷新（每2秒）
bloc.add(const ExecutionLogEvent.toggleAutoScroll(true));

// 禁用自动刷新
bloc.add(const ExecutionLogEvent.toggleAutoScroll(false));
```

### 4. 手动刷新

```dart
// 手动刷新日志列表
bloc.add(const ExecutionLogEvent.refreshLogs());
```

### 5. 加载更多

```dart
// 加载更多日志（自动触发，也可手动调用）
bloc.add(const ExecutionLogEvent.loadMoreLogs());
```

## 完整示例

这是一个完整的可运行示例：

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_bloc.dart';
import 'package:appflowy/plugins/ai_chat/presentation/execution_log_viewer.dart';

class ExecutionLogExamplePage extends StatelessWidget {
  final String chatId;
  final String? messageId;
  
  const ExecutionLogExamplePage({
    Key? key,
    required this.chatId,
    this.messageId,
  }) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('执行日志'),
        actions: [
          // 过滤按钮
          PopupMenuButton<ExecutionPhasePB?>(
            icon: const Icon(Icons.filter_list),
            tooltip: '过滤阶段',
            onSelected: (phase) {
              context.read<ExecutionLogBloc>().add(
                ExecutionLogEvent.filterByPhase(phase),
              );
            },
            itemBuilder: (context) => [
              const PopupMenuItem(
                value: null,
                child: Text('所有阶段'),
              ),
              ...ExecutionPhasePB.values.map(
                (phase) => PopupMenuItem(
                  value: phase,
                  child: Text(_getPhaseDisplayName(phase)),
                ),
              ),
            ],
          ),
          
          // 刷新按钮
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '刷新',
            onPressed: () {
              context.read<ExecutionLogBloc>().add(
                const ExecutionLogEvent.refreshLogs(),
              );
            },
          ),
        ],
      ),
      body: BlocProvider(
        create: (context) => ExecutionLogBloc(
          sessionId: chatId,
          messageId: messageId,
        )..add(const ExecutionLogEvent.loadLogs()),
        child: ExecutionLogViewer(
          sessionId: chatId,
          messageId: messageId,
          showHeader: false,
        ),
      ),
    );
  }
  
  String _getPhaseDisplayName(ExecutionPhasePB phase) {
    switch (phase) {
      case ExecutionPhasePB.ExecPlanning:
        return '规划阶段';
      case ExecutionPhasePB.ExecExecution:
        return '执行阶段';
      case ExecutionPhasePB.ExecToolCall:
        return '工具调用';
      case ExecutionPhasePB.ExecReflection:
        return '反思阶段';
      case ExecutionPhasePB.ExecCompletion:
        return '完成阶段';
      default:
        return '未知阶段';
    }
  }
}
```

## 下一步

1. **选择集成方式**：根据你的UI设计选择上述任一集成方式
2. **测试功能**：创建智能体并发送消息，测试日志记录
3. **自定义样式**：根据应用主题调整日志查看器的样式
4. **添加更多功能**：如导出日志、日志统计等

## 注意事项

⚠️ **重要**：当前日志记录功能已经搭建完毕，但实际的日志记录代码还需要在 `chat.rs` 的关键执行点添加。请参考 `EXECUTION_LOG_IMPLEMENTATION.md` 中的"待完成功能"部分。

✅ **已就绪**：
- 后端API完全可用
- 前端UI组件完全就绪
- 状态管理已连接真实API
- 可以立即使用查看器展示日志

## 支持

如有问题或需要帮助，请参考：
- `EXECUTION_LOG_IMPLEMENTATION.md` - 完整实现文档
- `execution_log_viewer.dart` - UI组件源码
- `execution_log_bloc.dart` - 状态管理源码

---

**更新日期**：2025-10-03  
**版本**：v1.0


