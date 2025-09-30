# 执行日志查看器 (Execution Log Viewer)

## 概述

执行日志查看器是一个用于展示智能体执行过程详细日志的Flutter组件。它提供了实时日志更新、搜索过滤、高性能列表展示等功能。

## 主要功能

- ✅ **实时日志更新**: 支持自动刷新和实时显示新日志
- ✅ **搜索功能**: 支持关键词搜索日志内容
- ✅ **过滤功能**: 按执行阶段和状态过滤日志
- ✅ **高性能展示**: 支持大量日志数据的虚拟滚动
- ✅ **详细信息**: 显示输入、输出、错误信息和执行时间
- ✅ **响应式设计**: 适配不同屏幕尺寸

## 组件结构

### 核心组件

1. **ExecutionLogViewer** - 主查看器组件
2. **ExecutionLogBloc** - 状态管理
3. **ExecutionLogItem** - 单个日志项展示
4. **ExecutionLogIntegrationExample** - 集成示例

### 文件结构

```
lib/plugins/ai_chat/
├── presentation/
│   ├── execution_log_viewer.dart              # 主组件
│   ├── execution_log_integration_example.dart # 集成示例
│   └── execution_log_README.md               # 文档
└── application/
    └── execution_log_bloc.dart               # 状态管理
```

## 使用方法

### 基础用法

```dart
import 'package:appflowy/plugins/ai_chat/presentation/execution_log_viewer.dart';

// 在Widget中使用
ExecutionLogViewer(
  sessionId: 'your_session_id',
  messageId: 'optional_message_id', // 可选，用于过滤特定消息的日志
  height: 400,
  showHeader: true,
)
```

### 集成到聊天界面

```dart
// 使用扩展方法
chatWidget.withExecutionLog(
  sessionId: sessionId,
  messageId: messageId,
)

// 或者使用底部面板
ExecutionLogBottomPanel(
  sessionId: sessionId,
  messageId: messageId,
  initialHeight: 250,
)
```

### 在消息气泡中添加执行日志按钮

```dart
AgentMessageExecutionLogButton(
  sessionId: sessionId,
  messageId: messageId,
  onPressed: () {
    // 自定义处理逻辑
  },
)
```

## 参数说明

### ExecutionLogViewer 参数

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| sessionId | String | ✅ | - | 会话ID |
| messageId | String? | ❌ | null | 消息ID（用于过滤） |
| height | double | ❌ | 400 | 查看器高度 |
| showHeader | bool | ❌ | true | 是否显示头部 |

## 状态管理

### ExecutionLogBloc 事件

- `loadLogs()` - 加载日志
- `loadMoreLogs()` - 加载更多日志
- `refreshLogs()` - 刷新日志
- `searchLogs(String query)` - 搜索日志
- `filterByPhase(ExecutionPhasePB? phase)` - 按阶段过滤
- `filterByStatus(ExecutionStatusPB? status)` - 按状态过滤
- `toggleAutoScroll(bool enabled)` - 切换自动滚动
- `addLog(AgentExecutionLogPB log)` - 添加新日志

### ExecutionLogState 属性

- `logs` - 日志列表
- `isLoading` - 是否加载中
- `hasMore` - 是否有更多数据
- `totalCount` - 总日志数
- `searchQuery` - 搜索关键词
- `phaseFilter` - 阶段过滤器
- `statusFilter` - 状态过滤器
- `autoScroll` - 自动滚动开关

## 数据模型

### 执行阶段 (ExecutionPhasePB)

- `ExecPlanning` - 规划阶段
- `ExecExecution` - 执行阶段
- `ExecToolCall` - 工具调用
- `ExecReflection` - 反思阶段
- `ExecCompletion` - 完成阶段

### 执行状态 (ExecutionStatusPB)

- `ExecRunning` - 进行中
- `ExecSuccess` - 成功
- `ExecFailed` - 失败
- `ExecCancelled` - 已取消

## 样式定制

组件使用AppFlowy的设计系统，支持主题切换。主要样式包括：

- 状态颜色：蓝色(进行中)、绿色(成功)、红色(失败)、橙色(取消)
- 圆角边框：8px
- 间距：遵循AppFlowy间距规范
- 字体：使用FlowyText组件

## 性能优化

1. **虚拟滚动**: 使用ListView.builder实现大数据量的高效渲染
2. **分页加载**: 支持滚动到底部自动加载更多
3. **搜索防抖**: 300ms防抖避免频繁搜索
4. **状态缓存**: BLoC状态管理确保数据一致性

## 扩展功能

### 自定义日志项渲染

```dart
// 可以通过继承ExecutionLogItem来自定义渲染
class CustomExecutionLogItem extends ExecutionLogItem {
  // 自定义实现
}
```

### 添加新的过滤器

```dart
// 在ExecutionLogBloc中添加新的事件和状态
const factory ExecutionLogEvent.filterByCustom(String filter) = _FilterByCustom;
```

## 注意事项

1. **后端集成**: 当前使用模拟数据，需要实现真实的后端API调用
2. **权限控制**: 确保用户有权限查看执行日志
3. **内存管理**: 大量日志数据需要注意内存使用
4. **国际化**: 当前使用中文硬编码，后续需要支持多语言

## 开发计划

- [ ] 实现真实的后端API集成
- [ ] 添加日志导出功能
- [ ] 支持日志高亮显示
- [ ] 添加更多过滤选项
- [ ] 优化移动端体验
- [ ] 添加国际化支持

## 贡献指南

1. 遵循AppFlowy的代码规范
2. 添加适当的测试用例
3. 更新相关文档
4. 确保性能和用户体验

---

*最后更新: 2025年9月30日*
