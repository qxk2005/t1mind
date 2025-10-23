# 转换日志查看功能

## 概述

转换日志查看功能为AppFlowy的文档导入系统提供了详细的日志记录和查看能力，帮助用户诊断转换问题并监控转换过程。

## 功能特性

### 1. 日志显示
- **实时更新**: 日志会实时反映转换任务的状态变化
- **多级别日志**: 支持调试、信息、警告、错误、成功等不同级别的日志
- **详细信息**: 每个日志条目包含时间戳、任务ID、消息和详细信息

### 2. 过滤功能
- **级别过滤**: 可以按日志级别（调试、信息、警告、错误、成功）过滤
- **任务过滤**: 可以按特定任务过滤日志
- **关键词搜索**: 支持在日志消息和详情中搜索关键词

### 3. 导出功能
- **文本导出**: 将过滤后的日志导出为文本文件
- **时间戳命名**: 导出的文件包含时间戳，便于管理
- **跨平台支持**: 支持桌面和移动平台的导出

### 4. 性能优化
- **分页加载**: 大量日志采用分页加载，避免内存问题
- **虚拟滚动**: 使用ListView.builder实现高效的列表渲染
- **缓存优化**: 添加适当的缓存范围提高滚动性能

## 使用方法

### 1. 从转换进度对话框访问

```dart
// 在转换进度对话框中点击"查看日志"按钮
showConversionProgressDialog(
  context,
  tasks: conversionTasks,
  onCancel: () => cancelConversion(),
  onRetry: () => retryConversion(),
  onClose: () => closeDialog(),
);
```

### 2. 直接显示日志对话框

```dart
// 直接显示日志查看对话框
showConversionLogsDialog(
  context,
  tasks: conversionTasks,
  onClose: () => closeDialog(),
);
```

### 3. 创建转换任务

```dart
final task = ConversionTask(
  fileName: 'document.docx',
  filePath: '/path/to/document.docx',
  importType: ImportType.word,
  status: ConversionTaskStatus.processing,
  progress: 0.5,
  startTime: DateTime.now(),
);
```

## 文件结构

```
appflowy_flutter/lib/workspace/presentation/home/menu/sidebar/import/
├── conversion_logs_dialog.dart          # 日志查看对话框主文件
├── conversion_logs_example.dart          # 使用示例
├── conversion_progress_dialog.dart       # 转换进度对话框（已集成日志功能）
└── README.md                            # 本文档
```

## 核心组件

### ConversionLogsDialog
主要的日志查看对话框组件，提供完整的日志查看、过滤和导出功能。

### ConversionLogItem
单个日志条目的显示组件，支持不同级别的样式和详细信息展示。

### LogLevel
日志级别枚举，包含调试、信息、警告、错误、成功五个级别。

### ConversionLogEntry
日志条目的数据模型，包含ID、任务ID、级别、消息、时间戳和详细信息。

## 性能考虑

### 大日志量处理
- 使用分页机制，每页加载50条日志
- 实现"加载更多"按钮，按需加载
- 添加适当的缓存范围优化滚动性能

### 内存管理
- 使用ValueKey为列表项提供稳定的键值
- 及时释放定时器和控制器资源
- 避免在dispose后访问context

### 搜索优化
- 使用防抖机制，避免频繁的搜索操作
- 搜索查询延迟300毫秒执行
- 支持大小写不敏感的搜索

## 集成说明

日志查看功能已完全集成到现有的转换进度对话框中：

1. **进度对话框增强**: 在转换进度对话框中添加了"查看日志"按钮
2. **无缝切换**: 用户可以在进度和日志视图之间无缝切换
3. **状态同步**: 日志会实时反映转换任务的状态变化

## 示例代码

查看 `conversion_logs_example.dart` 文件获取完整的使用示例，包括：
- 如何创建示例转换任务
- 如何显示进度和日志对话框
- 如何测试大日志量的性能

## 注意事项

1. **平台兼容性**: 导出功能在不同平台上的行为可能略有不同
2. **文件权限**: 确保应用有足够的文件系统权限进行日志导出
3. **内存使用**: 对于极大的日志量，建议定期清理或使用更高级的日志管理策略

## 未来改进

1. **日志持久化**: 将日志保存到数据库，支持历史查看
2. **高级过滤**: 添加日期范围、文件类型等更多过滤选项
3. **日志分析**: 提供日志统计和分析功能
4. **实时推送**: 支持WebSocket实时推送日志更新