# 工具调用和任务规划 UI 集成指南

**日期**: 2025-10-02  
**组件**: `ToolCallDisplay`, `TaskPlanDisplay`

## 概述

本指南说明如何在 AI 聊天消息中集成工具调用和任务规划的显示组件。

## 新增组件

### 1. ToolCallDisplay - 工具调用显示

**文件**: `tool_call_display.dart`

**功能**:
- ✅ 显示工具调用列表
- ✅ 显示工具状态（pending, running, success, failed）
- ✅ 可展开/折叠查看详情
- ✅ 显示工具参数和结果
- ✅ 动画效果

**使用示例**:
```dart
ToolCallDisplay(
  toolCalls: [
    ToolCallInfo(
      id: 'call_001',
      toolName: 'search_documents',
      status: ToolCallStatus.success,
      arguments: {'query': '搜索词', 'limit': 10},
      description: '搜索文档',
      result: '找到 5 个相关文档',
      startTime: DateTime.now().subtract(Duration(seconds: 2)),
      endTime: DateTime.now(),
    ),
  ],
)
```

### 2. TaskPlanDisplay - 任务规划显示

**文件**: `task_plan_display.dart`

**功能**:
- ✅ 显示任务计划目标
- ✅ 显示步骤列表和状态
- ✅ 显示每个步骤使用的工具
- ✅ 显示整体进度
- ✅ 渐变背景和精美设计

**使用示例**:
```dart
TaskPlanDisplay(
  plan: TaskPlanInfo(
    id: 'plan_001',
    goal: '创建一个完整的项目文档',
    status: TaskPlanStatus.running,
    steps: [
      TaskStepInfo(
        id: 'step_1',
        description: '分析项目需求',
        status: TaskStepStatus.completed,
        tools: ['document_analyzer'],
      ),
      TaskStepInfo(
        id: 'step_2',
        description: '生成文档大纲',
        status: TaskStepStatus.running,
        tools: ['outline_generator'],
      ),
      TaskStepInfo(
        id: 'step_3',
        description: '填充内容细节',
        status: TaskStepStatus.pending,
        tools: ['content_writer'],
      ),
    ],
  ),
)
```

## 集成到 ChatAIMessageState

### 第1步：扩展 State 定义

**文件**: `appflowy_flutter/lib/plugins/ai_chat/application/chat_ai_message_bloc.dart`

在 `ChatAIMessageState` 中添加新字段：

```dart
@freezed
class ChatAIMessageState with _$ChatAIMessageState {
  const factory ChatAIMessageState({
    @Default("") String text,
    @Default(LoadingState.loading()) LoadingState messageState,
    @Default([]) List<ChatMessageRefSource> sources,
    String? reasoningText,
    @Default(false) bool isReasoningComplete,
    // 🔧 新增字段
    @Default([]) List<ToolCallInfo> toolCalls,
    TaskPlanInfo? taskPlan,
  }) = _ChatAIMessageState;
}
```

### 第2步：更新 Bloc 事件处理

在 `ChatAIMessageBloc` 中解析工具调用和任务计划：

```dart
class ChatAIMessageBloc extends Bloc<ChatAIMessageEvent, ChatAIMessageState> {
  // ... 现有代码 ...
  
  Future<void> _handleMetadata(Map<String, dynamic> metadata) async {
    // 解析工具调用
    if (metadata.containsKey('tool_call')) {
      final toolCallData = metadata['tool_call'];
      _handleToolCall(toolCallData);
    }
    
    // 解析任务计划
    if (metadata.containsKey('task_plan')) {
      final planData = metadata['task_plan'];
      _handleTaskPlan(planData);
    }
  }
  
  void _handleToolCall(Map<String, dynamic> data) {
    // 创建或更新工具调用
    final toolCall = ToolCallInfo(
      id: data['id'],
      toolName: data['tool_name'],
      status: _parseToolCallStatus(data['status']),
      arguments: data['arguments'] ?? {},
      description: data['description'],
      result: data['result'],
      error: data['error'],
    );
    
    emit(state.copyWith(
      toolCalls: [...state.toolCalls, toolCall],
    ));
  }
  
  void _handleTaskPlan(Map<String, dynamic> data) {
    // 解析任务计划
    final plan = TaskPlanInfo(
      id: data['id'],
      goal: data['goal'],
      status: _parseTaskPlanStatus(data['status']),
      steps: (data['steps'] as List).map((s) => TaskStepInfo(
        id: s['id'],
        description: s['description'],
        status: _parseTaskStepStatus(s['status']),
        tools: List<String>.from(s['tools'] ?? []),
        error: s['error'],
      )).toList(),
    );
    
    emit(state.copyWith(taskPlan: plan));
  }
  
  ToolCallStatus _parseToolCallStatus(String status) {
    switch (status) {
      case 'pending': return ToolCallStatus.pending;
      case 'running': return ToolCallStatus.running;
      case 'success': return ToolCallStatus.success;
      case 'failed': return ToolCallStatus.failed;
      default: return ToolCallStatus.pending;
    }
  }
  
  TaskPlanStatus _parseTaskPlanStatus(String status) {
    switch (status) {
      case 'pending': return TaskPlanStatus.pending;
      case 'running': return TaskPlanStatus.running;
      case 'completed': return TaskPlanStatus.completed;
      case 'failed': return TaskPlanStatus.failed;
      case 'cancelled': return TaskPlanStatus.cancelled;
      default: return TaskPlanStatus.pending;
    }
  }
  
  TaskStepStatus _parseTaskStepStatus(String status) {
    switch (status) {
      case 'pending': return TaskStepStatus.pending;
      case 'running': return TaskStepStatus.running;
      case 'completed': return TaskStepStatus.completed;
      case 'failed': return TaskStepStatus.failed;
      default: return TaskStepStatus.pending;
    }
  }
}
```

### 第3步：更新 UI 显示

**文件**: `appflowy_flutter/lib/plugins/ai_chat/presentation/message/ai_text_message.dart`

在 `_buildMessageContent` 方法中添加组件：

```dart
Widget _buildMessageContent(ChatAIMessageState state, bool isLastMessage, bool isStreaming) {
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      // 推理过程显示
      if (state.reasoningText != null && state.reasoningText!.isNotEmpty)
        Padding(
          padding: EdgeInsetsDirectional.only(start: 4.0, bottom: 8.0),
          child: _AIReasoningDisplay(
            reasoningText: state.reasoningText!,
            isReasoningComplete: state.isReasoningComplete,
            isStreaming: isStreaming,
          ),
        ),
      
      // 🔧 任务计划显示（在消息内容之前）
      if (state.taskPlan != null)
        Padding(
          padding: EdgeInsetsDirectional.only(start: 4.0, bottom: 8.0),
          child: TaskPlanDisplay(plan: state.taskPlan!),
        ),
      
      // 🔧 工具调用显示（在消息内容之前）
      if (state.toolCalls.isNotEmpty)
        Padding(
          padding: EdgeInsetsDirectional.only(start: 4.0, bottom: 8.0),
          child: ToolCallDisplay(toolCalls: state.toolCalls),
        ),
      
      // 消息内容
      Padding(
        padding: EdgeInsetsDirectional.only(start: 4.0),
        child: AIMarkdownText(
          markdown: state.text,
          withAnimation: enableAnimation && stream != null,
        ),
      ),
      
      // 元数据（来源等）
      if (state.sources.isNotEmpty)
        SelectionContainer.disabled(
          child: AIMessageMetadata(
            sources: state.sources,
            onSelectedMetadata: onSelectedMetadata,
          ),
        ),
      if (state.sources.isNotEmpty && !isLastMessage) const VSpace(8.0),
    ],
  );
}
```

## 数据流

```
Rust Backend (StreamToolWrapper)
  ↓ 检测工具调用
  ↓ 执行工具
  ↓ 生成 Metadata
  
Flutter Frontend (ChatAIMessageBloc)
  ↓ 接收 Metadata
  ↓ 解析 tool_call / task_plan
  ↓ 更新 State
  
Flutter UI (ChatAIMessageWidget)
  ↓ BlocBuilder 触发重建
  ↓ 显示 ToolCallDisplay / TaskPlanDisplay
  ↓ 用户看到工具调用和计划进度
```

## 测试步骤

### 1. 工具调用测试

```dart
// 在开发环境中模拟工具调用
void testToolCalls() {
  final bloc = ChatAIMessageBloc(
    message: "测试消息",
    refSourceJsonString: null,
    chatId: "test_chat",
    questionId: 1,
  );
  
  // 模拟工具调用元数据
  bloc.add(ChatAIMessageEvent.receiveMetadata({
    'tool_call': {
      'id': 'call_001',
      'tool_name': 'search_documents',
      'status': 'running',
      'arguments': {'query': '测试查询', 'limit': 5},
    },
  }));
  
  // 等待一段时间后更新为成功
  Future.delayed(Duration(seconds: 2), () {
    bloc.add(ChatAIMessageEvent.receiveMetadata({
      'tool_call': {
        'id': 'call_001',
        'status': 'success',
        'result': '找到 3 个相关文档',
      },
    }));
  });
}
```

### 2. 任务计划测试

```dart
void testTaskPlan() {
  final bloc = ChatAIMessageBloc(
    message: "创建项目文档",
    refSourceJsonString: null,
    chatId: "test_chat",
    questionId: 1,
  );
  
  // 模拟任务计划创建
  bloc.add(ChatAIMessageEvent.receiveMetadata({
    'task_plan': {
      'id': 'plan_001',
      'goal': '创建完整的项目文档',
      'status': 'running',
      'steps': [
        {
          'id': 'step_1',
          'description': '分析项目需求',
          'status': 'completed',
          'tools': ['analyzer'],
        },
        {
          'id': 'step_2',
          'description': '生成文档大纲',
          'status': 'running',
          'tools': ['generator'],
        },
      ],
    },
  }));
}
```

## 样式定制

### 主题颜色

组件使用 `AFThemeExtension` 获取主题颜色，自动适配亮/暗模式：

```dart
AFThemeExtension.of(context).textColor
Theme.of(context).colorScheme.onSurface
Theme.of(context).colorScheme.surface
```

### 自定义颜色

可以通过修改组件内的颜色常量来定制：

```dart
// tool_call_display.dart
Color _getStatusColor(BuildContext context) {
  switch (widget.toolCall.status) {
    case ToolCallStatus.success:
      return Colors.green; // 可以改为自定义颜色
    // ...
  }
}

// task_plan_display.dart
decoration: BoxDecoration(
  gradient: LinearGradient(
    colors: [
      Colors.purple.withOpacity(0.05), // 可以改为自定义渐变
      Colors.blue.withOpacity(0.05),
    ],
  ),
)
```

## 性能优化

### 1. 使用 const 构造函数

所有可能的地方都使用了 `const` 构造函数以减少重建。

### 2. 动画控制

使用 `SingleTickerProviderStateMixin` 和 `AnimationController` 优化动画性能。

### 3. 条件渲染

使用条件语句避免渲染空组件：

```dart
if (toolCalls.isEmpty) {
  return const SizedBox.shrink();
}
```

## 未来增强

### 待实现功能

1. **工具调用重试** - 添加重试按钮
2. **计划编辑** - 允许用户修改计划步骤
3. **执行控制** - 暂停/继续/取消计划执行
4. **详细日志** - 查看每个步骤的详细执行日志
5. **导出功能** - 导出计划和结果为文档

### 建议的改进

1. **国际化** - 添加多语言支持
2. **无障碍性** - 添加语义标签和屏幕阅读器支持
3. **触觉反馈** - 在交互时提供触觉反馈
4. **声音提示** - 工具执行完成时播放提示音

## 常见问题

### Q: 如何更新工具调用状态？

A: 通过发送新的 Metadata 事件：

```dart
bloc.add(ChatAIMessageEvent.receiveMetadata({
  'tool_call': {
    'id': 'existing_call_id',
    'status': 'success',
    'result': '执行结果',
  },
}));
```

### Q: 如何显示多个工具调用？

A: `ToolCallDisplay` 自动支持多个工具调用，只需添加到列表：

```dart
toolCalls: [toolCall1, toolCall2, toolCall3]
```

### Q: 任务计划可以动态更新吗？

A: 是的，发送新的 Metadata 会替换整个计划状态，包括步骤的更新。

## 总结

- ✅ 创建了两个精美的 UI 组件
- ✅ 提供了完整的集成指南
- ✅ 包含测试和定制说明
- ✅ 遵循 Flutter 最佳实践
- ✅ 支持主题适配

---

**实施状态**: UI 组件完成，等待 Bloc 集成  
**文件数**: 3 个（2 个组件 + 1 个指南）  
**代码行数**: ~900 行  
**最后更新**: 2025-10-02


