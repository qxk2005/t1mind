# 引用显示组件 (CitationDisplay)

## 概述

`CitationDisplay` 是一个专门用于在AI聊天消息中显示网络搜索结果引用的Flutter组件。它提供了美观的引用列表显示，支持可点击链接、适当的格式化和安全的URL处理。

## 功能特性

- ✅ **安全的链接处理**: 自动验证URL格式，防止恶意链接
- ✅ **可展开/折叠显示**: 支持大量引用的分页显示
- ✅ **美观的UI设计**: 与AppFlowy设计系统保持一致
- ✅ **相关性评分显示**: 可选显示搜索结果的相关性评分
- ✅ **响应式设计**: 适配不同屏幕尺寸
- ✅ **国际化支持**: 支持多语言显示

## 基本用法

```dart
import 'package:appflowy/plugins/ai_chat/widgets/citation_display.dart';

// 创建引用信息列表
final citations = [
  CitationInfo(
    title: 'Flutter官方文档 - 状态管理',
    url: 'https://docs.flutter.dev/development/data-and-backend/state-mgmt',
    domain: 'docs.flutter.dev',
    snippet: '了解Flutter中的状态管理最佳实践...',
    relevanceScore: 0.95,
    index: 1,
  ),
  // 更多引用...
];

// 在Widget中使用
CitationDisplay(
  citations: citations,
  maxVisibleCitations: 3,
  showExpandButton: true,
)
```

## 组件参数

### CitationDisplay

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `citations` | `List<CitationInfo>` | 必需 | 引用信息列表 |
| `maxVisibleCitations` | `int` | 5 | 默认显示的最大引用数量 |
| `showExpandButton` | `bool` | true | 是否显示展开/折叠按钮 |

### CitationInfo

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `title` | `String` | 必需 | 引用标题 |
| `url` | `String` | 必需 | 引用URL |
| `domain` | `String` | 必需 | 引用域名 |
| `snippet` | `String` | 必需 | 引用摘要 |
| `relevanceScore` | `double` | 0.0 | 相关性评分 (0.0-1.0) |
| `index` | `int` | 0 | 引用索引 |

## 集成示例

### 在AI消息中集成

```dart
// 在AI消息组件中添加引用显示
Column(
  crossAxisAlignment: CrossAxisAlignment.start,
  children: [
    // 消息内容
    Padding(
      padding: const EdgeInsetsDirectional.only(start: 4.0),
      child: AIMarkdownText(markdown: messageText),
    ),
    
    // 网络搜索结果引用
    if (webSearchCitations.isNotEmpty)
      Padding(
        padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
        child: CitationDisplay(
          citations: webSearchCitations,
          maxVisibleCitations: 3,
          showExpandButton: true,
        ),
      ),
    
    // 现有文档来源元数据
    if (documentSources.isNotEmpty)
      AIMessageMetadata(
        sources: documentSources,
        onSelectedMetadata: onSelectedMetadata,
      ),
  ],
)
```

### 从ChatMessageRefSource转换

```dart
// 将现有的ChatMessageRefSource转换为CitationInfo
List<CitationInfo> convertRefSourcesToCitations(
  List<ChatMessageRefSource> refSources,
) {
  return refSources
      .where((ref) => ref.source == 'web')
      .map((ref) => CitationInfo.fromRefSource(ref))
      .toList();
}
```

### 从网络搜索结果数据创建

```dart
// 从Rust后端返回的搜索结果数据创建CitationInfo
List<CitationInfo> createCitationsFromWebSearchResults(
  List<Map<String, dynamic>> searchResults,
) {
  return searchResults
      .map((result) => CitationInfo.fromMap(result))
      .toList();
}
```

## 安全特性

### URL验证和处理

组件内置了多层安全机制：

1. **URL格式验证**: 使用 `string_validator` 包验证URL格式
2. **协议检查**: 确保URL包含有效的协议（http/https）
3. **本地主机处理**: 特殊处理localhost地址，添加http协议
4. **外部启动**: 使用 `url_launcher` 在外部浏览器中打开链接
5. **错误处理**: 捕获并记录URL启动失败的情况

### 示例安全处理

```dart
// 组件会自动处理以下情况：
// ✅ https://example.com - 正常URL
// ✅ http://localhost:3000 - 本地开发服务器
// ❌ javascript:alert('xss') - 恶意脚本
// ❌ file:///etc/passwd - 本地文件访问
```

## 样式定制

### 主题适配

组件自动适配AppFlowy主题系统：

- 使用 `Theme.of(context).hintColor` 作为次要文本颜色
- 使用 `Theme.of(context).colorScheme.primary` 作为链接颜色
- 使用 `Theme.of(context).dividerColor` 作为边框颜色

### 自定义样式

```dart
// 可以通过包装组件来自定义样式
Container(
  decoration: BoxDecoration(
    borderRadius: BorderRadius.circular(12.0),
    color: Theme.of(context).colorScheme.surface,
  ),
  child: CitationDisplay(
    citations: citations,
    // ... 其他参数
  ),
)
```

## 性能优化

### 大量引用处理

对于大量引用（>10个），建议：

1. 使用 `maxVisibleCitations` 限制默认显示数量
2. 启用 `showExpandButton` 提供展开功能
3. 考虑在服务端进行分页处理

### 内存管理

```dart
// 对于大量引用，考虑使用懒加载
ListView.builder(
  itemCount: citations.length,
  itemBuilder: (context, index) {
    return CitationDisplay(
      citations: [citations[index]],
      showExpandButton: false,
    );
  },
)
```

## 测试

### 运行测试页面

```dart
// 在开发环境中运行测试页面
void main() {
  runApp(const CitationDisplayTestApp());
}
```

### 测试场景

测试页面包含以下测试场景：

1. **基本引用显示**: 3个引用的基本显示
2. **可展开引用显示**: 5个引用，默认显示3个
3. **大量引用显示**: 10个引用的性能测试
4. **空引用列表**: 边界情况测试
5. **单个引用**: 最小用例测试

## 最佳实践

### 1. 数据准备

```dart
// 确保引用数据质量
final citations = searchResults
    .where((result) => result.url.isNotEmpty && isURL(result.url))
    .map((result) => CitationInfo.fromMap(result))
    .toList();
```

### 2. 错误处理

```dart
// 处理空引用列表
if (citations.isEmpty) {
  return const SizedBox.shrink();
}

// 处理无效引用
final validCitations = citations
    .where((citation) => citation.url.isNotEmpty)
    .toList();
```

### 3. 用户体验

```dart
// 提供加载状态
if (isLoading) {
  return const CircularProgressIndicator();
}

// 提供错误状态
if (hasError) {
  return const Text('加载引用失败');
}
```

## 故障排除

### 常见问题

1. **链接无法打开**
   - 检查URL格式是否正确
   - 确认设备上安装了默认浏览器
   - 查看控制台错误日志

2. **引用不显示**
   - 检查 `citations` 列表是否为空
   - 确认 `CitationInfo` 数据格式正确
   - 验证组件是否正确集成到Widget树中

3. **样式问题**
   - 确认主题配置正确
   - 检查父容器的约束条件
   - 验证FlowySvg图标是否正确导入

### 调试技巧

```dart
// 添加调试信息
debugPrint('Citations count: ${citations.length}');
debugPrint('First citation: ${citations.first}');

// 使用测试数据
final testCitations = CitationDisplayExampleData.generateSampleWebSearchCitations();
```

## 更新日志

### v1.0.0 (2025-01-11)
- ✅ 初始版本发布
- ✅ 基本引用显示功能
- ✅ 可展开/折叠功能
- ✅ 安全URL处理
- ✅ 相关性评分显示
- ✅ 测试页面和文档

## 贡献指南

1. Fork 项目仓库
2. 创建功能分支 (`git checkout -b feature/new-feature`)
3. 提交更改 (`git commit -am 'Add new feature'`)
4. 推送到分支 (`git push origin feature/new-feature`)
5. 创建 Pull Request

## 许可证

本项目遵循 AppFlowy 项目的许可证条款。
