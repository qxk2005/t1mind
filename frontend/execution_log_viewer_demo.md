# 执行日志查看器组件 (ExecutionLogViewer)

## 概述

`ExecutionLogViewer` 是一个功能完整的执行日志查看器组件，专为AI聊天MCP工具编排功能设计。它提供了完整的日志浏览、搜索、过滤和导出功能，支持大量日志数据的高效显示。

## 主要功能

### 1. 日志浏览
- 分页加载，支持大量日志数据
- 实时状态显示（运行中、完成、失败等）
- 执行时间和步骤进度显示
- 智能体和工具信息展示

### 2. 搜索功能
- 实时搜索用户查询内容
- 搜索错误信息
- 搜索使用的MCP工具名称
- 支持清除搜索条件

### 3. 过滤功能
- 按执行状态过滤（完成、失败、运行中等）
- 按错误类型过滤
- 按智能体过滤
- 按MCP工具过滤
- 按标签过滤
- 按执行时间范围过滤
- 仅显示有错误的日志
- 仅显示有引用的日志

### 4. 排序功能
- 按创建时间排序
- 按结束时间排序
- 按执行时长排序
- 按状态排序
- 按步骤数排序
- 按错误类型排序
- 支持升序/降序

### 5. 多选功能
- 支持多选模式
- 全选/取消全选
- 批量操作选中项
- 长按进入多选模式

### 6. 导出功能
- JSON格式导出
- CSV格式导出
- HTML格式导出
- 纯文本格式导出
- 可配置导出内容（步骤、引用、元数据等）
- 支持分享导出文件

## 使用方法

### 基本用法

```dart
ExecutionLogViewer(
  sessionId: 'session_123', // 可选：按会话过滤
  onLogSelected: (log) {
    // 处理日志选择
    print('选中日志: ${log.id}');
  },
)
```

### 高级用法

```dart
ExecutionLogViewer(
  sessionId: 'session_123',
  agentId: 'agent_456',
  maxHeight: 800,
  showExportButton: true,
  showFilterButton: true,
  initialFilter: ExecutionLogFilter(
    statuses: [ExecutionLogStatus.failed],
    hasErrors: true,
  ),
  onLogSelected: (log) {
    // 显示日志详情
    showLogDetailDialog(context, log);
  },
)
```

## 组件参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| sessionId | String? | null | 会话ID过滤 |
| agentId | String? | null | 智能体ID过滤 |
| onLogSelected | ValueChanged<ExecutionLog>? | null | 日志选择回调 |
| maxHeight | double | 600 | 最大高度 |
| showExportButton | bool | true | 是否显示导出按钮 |
| showFilterButton | bool | true | 是否显示过滤按钮 |
| initialFilter | ExecutionLogFilter? | null | 初始过滤器 |

## 数据模型

### ExecutionLog
执行日志主数据模型，包含：
- 基本信息：ID、会话ID、用户查询
- 时间信息：开始时间、结束时间
- 状态信息：执行状态、错误信息、错误类型
- 执行信息：智能体ID、步骤统计、使用的工具
- 扩展信息：标签、上下文、重试信息

### ExecutionLogFilter
过滤器配置，支持：
- 状态过滤：按执行状态筛选
- 错误过滤：按错误类型筛选
- 工具过滤：按MCP工具筛选
- 时间过滤：按执行时长筛选
- 特殊过滤：仅错误日志、仅有引用日志

### ExecutionLogSortOptions
排序选项配置：
- sortBy：排序字段
- direction：排序方向（升序/降序）

### ExecutionLogExportOptions
导出选项配置：
- format：导出格式
- includeSteps：是否包含步骤
- includeReferences：是否包含引用
- includeMetadata：是否包含元数据
- includeErrorDetails：是否包含错误详情

## 性能优化

1. **分页加载**：支持大量数据的分页显示，避免一次性加载过多数据
2. **虚拟滚动**：使用ListView.separated实现高效的列表渲染
3. **延迟搜索**：搜索输入防抖，避免频繁查询
4. **状态缓存**：缓存过滤和排序状态，提升用户体验
5. **异步操作**：导出等耗时操作使用异步处理

## 可访问性

1. **键盘导航**：支持键盘操作
2. **屏幕阅读器**：提供语义化标签
3. **高对比度**：支持系统主题
4. **触摸友好**：适配移动端触摸操作

## 扩展性

1. **插件化过滤器**：可扩展自定义过滤器
2. **自定义导出格式**：支持添加新的导出格式
3. **主题定制**：支持自定义样式主题
4. **国际化**：支持多语言本地化

## 注意事项

1. 组件依赖于执行日志数据模型，需要确保数据结构正确
2. 导出功能需要文件系统权限
3. 大量数据时建议启用分页加载
4. 移动端使用时注意屏幕尺寸适配

## 未来改进

1. 支持实时日志流更新
2. 添加日志详情预览面板
3. 支持日志数据可视化图表
4. 添加日志分析和统计功能
5. 支持自定义列显示配置
