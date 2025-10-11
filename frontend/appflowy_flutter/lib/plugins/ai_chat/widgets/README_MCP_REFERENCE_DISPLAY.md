# MCP引用显示组件 (MCPReferenceDisplay)

## 概述

`MCPReferenceDisplay` 是一个专门用于在AI聊天消息中显示MCP（Model Context Protocol）工具调用引用的Flutter组件。它能够清晰地展示哪些MCP工具被AI调用，提高了AI回答的透明度和可追溯性。

## 功能特性

- ✅ **服务器.工具名称格式**: 以清晰的格式显示"服务器名称.工具名称"
- ✅ **服务器名称高亮**: 使用主题色高亮显示服务器名称
- ✅ **展开/折叠功能**: 支持多个工具引用的分页显示
- ✅ **点击交互**: 可选的点击回调，支持未来功能扩展
- ✅ **状态指示**: 显示工具调用成功状态
- ✅ **主题适配**: 自动适配明暗主题
- ✅ **响应式设计**: 适配不同屏幕尺寸

## 快速开始

### 基本用法

```dart
import 'package:appflowy/plugins/ai_chat/widgets/mcp_reference_display.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';

// 创建MCP引用列表
final references = [
  ChatMessageRefSource(
    id: 'call_001',
    name: 'read_data_from_excel',
    source: 'mcp:excel',  // 格式: "mcp:服务器ID"
  ),
  ChatMessageRefSource(
    id: 'call_002',
    name: 'create_chart',
    source: 'mcp:excel',
  ),
];

// 在Widget中使用
MCPReferenceDisplay(
  references: references,
  showExpandButton: true,
  onReferenceSelected: (ref) {
    // 处理引用点击事件（可选）
    print('点击了工具: ${ref.name}');
  },
)
```

## 组件参数

### MCPReferenceDisplay

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| `references` | `List<ChatMessageRefSource>` | ✓ | - | MCP工具引用列表 |
| `showExpandButton` | `bool` | ✗ | `true` | 是否显示展开/折叠按钮 |
| `onReferenceSelected` | `Function(ChatMessageRefSource)?` | ✗ | `null` | 引用被点击时的回调 |

### ChatMessageRefSource (用于MCP)

| 字段 | 类型 | 示例 | 描述 |
|------|------|------|------|
| `id` | `String` | `"call_001"` | 工具调用ID |
| `name` | `String` | `"read_data_from_excel"` | 工具名称 |
| `source` | `String` | `"mcp:excel"` | 引用源，格式为 `mcp:服务器ID` |

## 数据格式

### Source字段格式

MCP引用的`source`字段使用特殊格式来标识：

```
mcp:服务器ID
```

例如：
- `mcp:excel` - Excel MCP服务器的工具
- `mcp:filesystem` - 文件系统MCP服务器的工具
- `mcp:readwise` - Readwise MCP服务器的工具
- `mcp:unknown` - 未知服务器（当无法确定服务器时）

### 显示格式

组件会自动解析`source`字段并以以下格式显示：

```
[序号] 服务器名称.工具名称 ✓
       ↑           ↑      ↑
    主色高亮    正常文本  成功图标
```

如果服务器ID为`unknown`或无法解析，则只显示工具名称。

## 使用场景

### 场景1：在AI消息中显示工具调用

这是最常见的使用场景，在`ai_text_message.dart`中集成：

```dart
// 在AI消息内容之后显示MCP工具引用
if (_hasMCPReferences(state.sources))
  Padding(
    padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
    child: MCPReferenceDisplay(
      references: _extractMCPReferences(state.sources),
      showExpandButton: true,
      onReferenceSelected: onSelectedMetadata,
    ),
  ),
```

### 场景2：显示单个工具调用

当只有一个工具调用时，可以隐藏展开按钮：

```dart
MCPReferenceDisplay(
  references: [singleReference],
  showExpandButton: false,
  onReferenceSelected: (ref) {
    // 处理点击
  },
)
```

### 场景3：只读显示（无交互）

如果不需要点击交互，省略`onReferenceSelected`参数：

```dart
MCPReferenceDisplay(
  references: references,
  showExpandButton: true,
  // 不提供onReferenceSelected，引用项不可点击
)
```

## 集成到现有聊天系统

### 步骤1：在metadata解析中识别MCP工具

在`chat_message_service.dart`中：

```dart
else if (map.containsKey("tool_call")) {
  final toolCallData = map["tool_call"] as Map<String, dynamic>?;
  if (toolCallData != null) {
    final toolName = toolCallData["tool_name"] as String?;
    final status = toolCallData["status"] as String?;
    
    if (status == "success" && toolName != null && toolName != "web_search") {
      // 创建MCP引用
      String? serverId = ...; // 从tool_name或metadata中提取
      
      metadata.add(ChatMessageRefSource(
        id: toolCallData["id"] ?? toolName,
        name: toolName,
        source: 'mcp:${serverId ?? "unknown"}',
      ));
    }
  }
}
```

### 步骤2：在消息组件中添加检测和提取方法

在`ai_text_message.dart`中：

```dart
/// 检查是否有MCP工具引用
bool _hasMCPReferences(List<ChatMessageRefSource> sources) {
  return sources.any((source) => source.source.startsWith('mcp'));
}

/// 提取MCP工具引用
List<ChatMessageRefSource> _extractMCPReferences(List<ChatMessageRefSource> sources) {
  return sources
      .where((source) => source.source.startsWith('mcp'))
      .toList();
}
```

### 步骤3：在UI中显示组件

```dart
// 在消息内容之后添加
if (_hasMCPReferences(state.sources))
  Padding(
    padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
    child: MCPReferenceDisplay(
      references: _extractMCPReferences(state.sources),
      showExpandButton: true,
      onReferenceSelected: onSelectedMetadata,
    ),
  ),
```

### 步骤4：处理点击事件

在`text_message_widget.dart`的`_onSelectMetadata`方法中：

```dart
// MCP工具引用点击处理
if (metadata.source.startsWith("mcp")) {
  // 显示工具详情、执行日志等
  Log.info("MCP tool reference clicked: ${metadata.name}");
  // TODO: 实现详情显示功能
  return;
}
```

## 样式定制

组件使用以下主题元素，会自动适配应用主题：

- **主色**: `theme.colorScheme.primary` - 用于服务器名称、图标
- **背景色**: 根据主题明暗度自动调整
- **边框色**: 根据主题明暗度自动调整
- **文本色**: 根据主题明暗度自动调整

## 状态管理

组件内部使用`StatefulWidget`来管理展开/折叠状态：

```dart
class _MCPReferenceDisplayState extends State<MCPReferenceDisplay> {
  bool _isExpanded = false;
  
  @override
  Widget build(BuildContext context) {
    // 默认显示前3个引用
    final maxVisible = _isExpanded ? widget.references.length : 3;
    final visibleReferences = widget.references.take(maxVisible).toList();
    final hasMore = widget.references.length > 3;
    
    // ...
  }
}
```

## 性能优化

- **延迟加载**: 默认只显示前3个引用，点击"查看全部"后才显示完整列表
- **轻量级渲染**: 使用基础的Flutter组件，避免复杂的布局计算
- **智能更新**: 只在必要时重新构建UI

## 未来扩展

以下是计划中的功能扩展：

1. **工具执行详情**
   - 点击引用显示详细的执行信息
   - 显示工具参数和返回值
   - 显示执行耗时

2. **工具结果预览**
   - 在引用卡片中显示简短的结果摘要
   - 支持展开查看完整结果

3. **工具分组**
   - 按服务器分组显示工具
   - 显示每个服务器的工具统计

4. **交互增强**
   - 长按复制工具信息
   - 支持拖拽排序
   - 添加收藏功能

## 常见问题

### Q: 为什么有些引用显示为"unknown.tool_name"？

A: 这通常是因为后端没有提供服务器信息。确保在metadata中包含服务器ID，或在tool_name中使用"server.tool"格式。

### Q: 如何自定义引用项的点击行为？

A: 通过`onReferenceSelected`回调参数：

```dart
MCPReferenceDisplay(
  references: references,
  onReferenceSelected: (ref) {
    // 自定义处理逻辑
    showDialog(...);
  },
)
```

### Q: 能否修改默认显示的引用数量？

A: 目前默认显示3个引用。要修改，需要在组件内部调整`maxVisible`的计算逻辑。

### Q: 如何区分MCP引用和其他引用源？

A: 通过检查`source`字段是否以"mcp"开头：

```dart
bool isMCP = source.source.startsWith('mcp');
```

## 测试

组件包含完整的测试示例，位于 `mcp_reference_display_example.dart`:

```dart
// 运行测试示例
Navigator.of(context).push(
  MaterialPageRoute(
    builder: (_) => const MCPReferenceDisplayExample(),
  ),
);

// 快速测试
Navigator.of(context).push(
  MaterialPageRoute(
    builder: (_) => const QuickMCPReferenceTest(),
  ),
);
```

## 相关文件

- **组件实现**: `mcp_reference_display.dart`
- **使用示例**: `mcp_reference_display_example.dart`
- **集成指南**: `AI聊天引用系统集成指南.md`
- **消息服务**: `chat_message_service.dart`
- **AI消息显示**: `ai_text_message.dart`

## 版本历史

- **v1.0.0** (2025-01-11)
  - 初始版本
  - 支持基本的MCP工具引用显示
  - 实现展开/折叠功能
  - 添加点击回调支持
  - 完成主题适配

## 贡献指南

如果要为此组件添加新功能或修复bug，请遵循以下步骤：

1. 确保修改符合整体设计风格
2. 添加相应的测试用例
3. 更新文档说明
4. 提交前运行linter检查

## 许可证

该组件是AppFlowy项目的一部分，遵循项目的开源许可证。

