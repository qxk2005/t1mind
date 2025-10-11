# AI聊天引用系统集成指南

## 概述

AI聊天系统现已支持多种引用源的统一显示，包括：
- **网络搜索引用** - 来自web_search工具的搜索结果
- **MCP工具引用** - 来自MCP服务器的工具调用
- **文档检索引用** - 来自AppFlowy文档的检索结果

每种引用源都有专门的显示组件，提供一致的用户体验。

## 架构设计

### 数据流

```
Rust后端 (tool_call metadata)
    ↓
Flutter前端 (parseMetadata)
    ↓
ChatMessageRefSource 列表
    ↓
按source类型分类显示
    ├─ web + URL → CitationDisplay (网络搜索)
    ├─ mcp:*    → MCPReferenceDisplay (MCP工具)
    └─ appflowy → AIMessageMetadata (文档检索)
```

### 引用源标识

每个`ChatMessageRefSource`都有一个`source`字段来标识其来源：

- `source: "web"` + `id`是URL → 网络搜索结果
- `source: "mcp:server_id"` → MCP工具调用（server_id是MCP服务器的ID）
- `source: "appflowy"` → AppFlowy文档检索

## 实现细节

### 1. 元数据解析 (chat_message_service.dart)

`parseMetadata`函数负责解析从后端发送的metadata，并提取引用信息：

#### 文档检索引用解析（RAG）

```dart
else if (map.containsKey("SOURCE_ID") && map["SOURCE_ID"] != null) {
  // 处理文档检索引用（RAG）
  // 格式: { "SOURCE_ID": "uuid", "SOURCE": "appflowy", "SOURCE_NAME": "document" }
  final sourceId = map["SOURCE_ID"].toString();
  final source = map["SOURCE"]?.toString() ?? "appflowy";
  
  Log.info("📄 [DOC_RETRIEVAL] Found document reference: id=$sourceId, source=$source");
  
  metadata.add(ChatMessageRefSource(
    id: sourceId,
    name: "", // 空name，让AIMessageMetadata组件异步加载真实文档名称
    source: source,
  ));
}
```

#### 工具调用引用解析

```dart
else if (map.containsKey("tool_call")) {
  final toolCallData = map["tool_call"] as Map<String, dynamic>?;
  if (toolCallData != null) {
    final toolName = toolCallData["tool_name"] as String?;
    final result = toolCallData["result"] as String?;
    final status = toolCallData["status"] as String?;
    final toolCallId = toolCallData["id"] as String?;
    
    if (status == "success" && result != null && toolName != null) {
      // 网络搜索：从result字符串中提取URL引用
      if (toolName == "web_search") {
        final citations = _extractCitationsFromSearchResult(result);
        metadata.addAll(citations);
      } 
      // MCP工具：创建MCP引用
      else {
        // 提取server信息（如果tool_name包含server前缀）
        String displayName = toolName;
        String? serverId;
        
        if (toolName.contains('.')) {
          final parts = toolName.split('.');
          if (parts.length >= 2) {
            serverId = parts[0];
            displayName = parts.sublist(1).join('.');
          }
        }
        
        final mcpReference = ChatMessageRefSource(
          id: toolCallId ?? toolName,
          name: displayName,
          source: 'mcp:${serverId ?? "unknown"}',
        );
        metadata.add(mcpReference);
      }
    }
  }
}
```

### 2. 网络搜索引用显示 (CitationDisplay)

用于显示网络搜索结果，具有以下特点：
- 可点击的链接，安全地打开外部浏览器
- 显示标题、域名、摘要
- 支持展开/折叠
- 显示相关性评分（可选）

**位置**: `appflowy_flutter/lib/plugins/ai_chat/widgets/citation_display.dart`

**使用示例**:
```dart
if (_hasWebSearchCitations(state.sources))
  Padding(
    padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
    child: CitationDisplay(
      citations: _extractWebSearchCitations(state.sources),
      maxVisibleCitations: 3,
      showExpandButton: true,
    ),
  ),
```

### 3. MCP工具引用显示 (MCPReferenceDisplay)

用于显示MCP工具调用的引用，具有以下特点：
- 显示格式：**服务器名称.工具名称**
- 服务器名称以主色高亮显示
- 显示工具调用成功状态
- 支持展开/折叠多个工具

**位置**: `appflowy_flutter/lib/plugins/ai_chat/widgets/mcp_reference_display.dart`

**显示格式**:
```
[1] excel.read_data_from_excel ✓
    ↑       ↑
  服务器   工具名称
```

**使用示例**:
```dart
if (_hasMCPReferences(state.sources))
  Padding(
    padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
    child: MCPReferenceDisplay(
      references: _extractMCPReferences(state.sources),
      showExpandButton: true,
    ),
  ),
```

### 4. 文档检索引用显示 (AIMessageMetadata)

用于显示AppFlowy文档的检索结果，具有以下特点：
- 显示文档名称
- 可点击打开对应文档
- 支持展开/折叠
- 使用文档UUID进行识别

**位置**: `appflowy_flutter/lib/plugins/ai_chat/presentation/message/ai_metadata.dart`

**使用示例**:
```dart
if (_hasDocumentSources(state.sources))
  SelectionContainer.disabled(
    child: AIMessageMetadata(
      sources: _extractDocumentSources(state.sources),
      onSelectedMetadata: onSelectedMetadata,
    ),
  ),
```

## 集成在AI消息中

在`ai_text_message.dart`中，引用显示按以下顺序排列：

1. 消息内容
2. 网络搜索引用 (CitationDisplay)
3. MCP工具引用 (MCPReferenceDisplay)
4. 文档检索引用 (AIMessageMetadata)

```dart
Column(
  crossAxisAlignment: CrossAxisAlignment.start,
  children: [
    // 消息内容
    AIMarkdownText(
      markdown: state.text,
      withAnimation: enableAnimation && stream != null,
    ),
    
    // 网络搜索结果引用显示
    if (_hasWebSearchCitations(state.sources))
      CitationDisplay(...),
    
    // MCP工具引用显示
    if (_hasMCPReferences(state.sources))
      MCPReferenceDisplay(...),
    
    // 文档来源元数据显示
    if (_hasDocumentSources(state.sources))
      AIMessageMetadata(...),
  ],
)
```

## 辅助函数

### 引用源检测

```dart
/// 检查是否有网络搜索引用
bool _hasWebSearchCitations(List<ChatMessageRefSource> sources) {
  return sources.any((source) => source.source == 'web' && isURL(source.id));
}

/// 检查是否有MCP工具引用
bool _hasMCPReferences(List<ChatMessageRefSource> sources) {
  return sources.any((source) => source.source.startsWith('mcp'));
}

/// 检查是否有文档来源
bool _hasDocumentSources(List<ChatMessageRefSource> sources) {
  return sources.any((source) => source.source == 'appflowy' || 
      (source.source == 'web' && !isURL(source.id)));
}
```

### 引用源提取

```dart
/// 提取网络搜索引用
List<CitationInfo> _extractWebSearchCitations(List<ChatMessageRefSource> sources) {
  return sources
      .where((source) => source.source == 'web' && isURL(source.id))
      .map((source) => CitationInfo.fromRefSource(source))
      .toList();
}

/// 提取MCP工具引用
List<ChatMessageRefSource> _extractMCPReferences(List<ChatMessageRefSource> sources) {
  return sources
      .where((source) => source.source.startsWith('mcp'))
      .toList();
}

/// 提取文档来源
List<ChatMessageRefSource> _extractDocumentSources(List<ChatMessageRefSource> sources) {
  return sources
      .where((source) => source.source == 'appflowy' || 
          (source.source == 'web' && !isURL(source.id)))
      .toList();
}
```

## 后端集成要点

### MCP工具调用metadata格式

当前后端发送的metadata格式：
```json
{
  "tool_call": {
    "id": "call_xxx",
    "tool_name": "tool_name",
    "status": "success",
    "result": "..."
  }
}
```

### 文档检索引用metadata格式

当RAG功能检索到文档时，后端发送的metadata格式：

```json
{
  "SOURCE_ID": "document-uuid",
  "SOURCE": "appflowy",
  "SOURCE_NAME": "document"
}
```

**后端实现位置**:
- `rust-lib/flowy-ai/src/local_ai/chat/chains/conversation_chain.rs` - stream结束后发送sources
- `rust-lib/flowy-ai/src/embeddings/document_indexer.rs` - 索引时添加metadata

前端会自动：
1. 识别`SOURCE_ID`字段
2. 使用`ViewBackendService.getView`异步查询真实文档名称
3. 在AIMessageMetadata组件中显示

### 建议改进：添加服务器信息

为了更好地显示"服务器名称.工具名称"格式，建议后端在metadata中添加服务器信息：

```json
{
  "tool_call": {
    "id": "call_xxx",
    "tool_name": "tool_name",
    "server_id": "excel",          // 新增
    "server_name": "Excel MCP",    // 新增（可选）
    "status": "success",
    "result": "..."
  }
}
```

或者，在tool_name中直接使用"server_id.tool_name"格式：
```json
{
  "tool_call": {
    "id": "call_xxx",
    "tool_name": "excel.read_data_from_excel",  // 使用点号分隔
    "status": "success",
    "result": "..."
  }
}
```

前端代码已支持这两种格式的解析。

## 样式设计

所有引用显示组件都遵循统一的设计原则：

1. **容器样式**
   - 圆角：8px (标题容器) / 6px (项目容器)
   - 边框：1px，根据主题自适应颜色
   - 背景：半透明，深色/浅色主题自适应

2. **颜色方案**
   - 主色：`theme.colorScheme.primary`
   - 文本：根据主题亮度自适应
   - 状态指示：绿色（成功）

3. **字体大小**
   - 标题：13px
   - 正文：12px
   - 辅助信息：10-11px

## 测试建议

### 测试场景

1. **网络搜索引用**
   - 使用web_search工具进行搜索
   - 验证引用是否正确显示
   - 点击链接测试是否能打开浏览器

2. **MCP工具引用**
   - 调用各种MCP工具（Excel、文件系统等）
   - 验证服务器名称和工具名称是否正确显示
   - 测试多个工具调用的显示

3. **文档检索引用**
   - 启用文档检索功能
   - 验证文档来源是否正确显示
   - 点击文档链接测试是否能打开对应文档

4. **混合引用**
   - 在单条消息中同时使用多种引用源
   - 验证所有引用都能正确显示且互不干扰

## 未来改进方向

1. **MCP工具详情**
   - 显示工具执行时间
   - 显示工具参数（可选）
   - 提供工具结果预览

2. **引用统计**
   - 统计各类引用的使用频率
   - 提供引用质量评分

3. **引用交互**
   - 支持引用的复制和分享
   - 提供引用的详细信息弹窗

4. **性能优化**
   - 大量引用的虚拟滚动
   - 引用数据的缓存机制

## 相关文件

- `appflowy_flutter/lib/plugins/ai_chat/application/chat_message_service.dart` - 元数据解析
- `appflowy_flutter/lib/plugins/ai_chat/widgets/citation_display.dart` - 网络搜索引用显示
- `appflowy_flutter/lib/plugins/ai_chat/widgets/mcp_reference_display.dart` - MCP工具引用显示
- `appflowy_flutter/lib/plugins/ai_chat/presentation/message/ai_metadata.dart` - 文档检索引用显示
- `appflowy_flutter/lib/plugins/ai_chat/presentation/message/ai_text_message.dart` - 引用集成

## 常见问题

### Q: MCP工具引用不显示服务器名称？

A: 检查后端是否在tool_name中包含了服务器信息（格式：`server.tool`），或者在metadata中添加了`server_id`字段。如果都没有，引用将显示为"unknown.tool_name"。

### Q: 如何区分不同类型的引用？

A: 通过`ChatMessageRefSource.source`字段：
- `"web"` + URL → 网络搜索
- `"mcp:*"` → MCP工具
- `"appflowy"` → 文档检索

### Q: 引用顺序能否调整？

A: 可以，在`ai_text_message.dart`中调整各个引用显示组件的顺序即可。

## 总结

通过统一的引用系统，AI聊天现在能够清晰地展示信息来源，提高了回答的可信度和透明度。每种引用源都有专门的显示组件，提供一致且美观的用户体验。

