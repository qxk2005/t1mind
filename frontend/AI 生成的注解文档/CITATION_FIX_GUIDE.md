# 网络搜索引用显示修复指南

## 问题描述

网络搜索功能正常工作并返回结果，但引用组件没有在AI聊天页面中显示。

## 根本原因

网络搜索结果包含在 `tool_call` metadata 中，格式如下：

```json
{
  "tool_call": {
    "tool_name": "web_search",
    "status": "success",
    "result": "搜索结果 (查询词):\n1. 标题1\n   链接: https://example.com/1\n2. 标题2\n   链接: https://example.com/2"
  }
}
```

但 `parseMetadata` 函数只处理旧格式的 `ChatMessageRefSource`，导致网络搜索结果被标记为"不支持的格式"并丢弃。

## 解决方案

### 修改的文件

1. **appflowy_flutter/lib/plugins/ai_chat/application/chat_message_service.dart**
   - 扩展 `parseMetadata` 函数，添加对 `tool_call` 格式的支持
   - 新增 `_extractCitationsFromSearchResult` 函数，从搜索结果字符串中提取URL引用
   - 将提取的URL转换为 `ChatMessageRefSource` 对象，source字段设置为 `'web'`

2. **appflowy_flutter/lib/plugins/ai_chat/presentation/message/ai_text_message.dart**
   - 添加详细的调试日志（临时性，可以在验证后移除）
   - 添加可视化调试信息框（临时性，可以在验证后移除）

### 关键代码改动

#### 1. 处理 tool_call metadata

```dart
else if (map.containsKey("tool_call")) {
  // 🔧 处理工具调用数据，特别是网络搜索结果
  final toolCallData = map["tool_call"] as Map<String, dynamic>?;
  if (toolCallData != null) {
    final toolName = toolCallData["tool_name"] as String?;
    final result = toolCallData["result"] as String?;
    final status = toolCallData["status"] as String?;
    
    // 检查是否是成功的网络搜索调用
    if (toolName == "web_search" && status == "success" && result != null) {
      Log.info("🔍 [WEB_SEARCH] Parsing web search result from tool_call");
      // 从result字符串中提取URL引用
      final citations = _extractCitationsFromSearchResult(result);
      metadata.addAll(citations);
      Log.info("🔍 [WEB_SEARCH] Extracted ${citations.length} citations");
    }
  }
}
```

#### 2. 提取引用信息

```dart
List<ChatMessageRefSource> _extractCitationsFromSearchResult(String result) {
  final List<ChatMessageRefSource> citations = [];
  
  try {
    // 使用正则表达式匹配引用模式
    // 匹配格式: 数字. 标题\n   链接: URL
    final pattern = RegExp(
      r'(\d+)\.\s+([^\n]+)\s+链接:\s+(https?://[^\s]+)',
      multiLine: true,
    );
    
    final matches = pattern.allMatches(result);
    
    for (final match in matches) {
      final index = match.group(1);
      final title = match.group(2)?.trim();
      final url = match.group(3)?.trim();
      
      if (title != null && url != null) {
        citations.add(ChatMessageRefSource(
          id: url,
          name: title,
          source: 'web',
        ));
      }
    }
  } catch (e) {
    Log.error("Failed to extract citations from search result: $e");
  }
  
  return citations;
}
```

## 验证步骤

1. **重启应用**
   ```bash
   # 停止当前运行的应用
   # 重新启动
   flutter run
   ```

2. **进行AI聊天测试**
   - 发送一个需要网络搜索的问题
   - 例如："珠江天河都荟的开发商是谁？"

3. **查看控制台日志**
   - 应该看到以下日志：
     ```
     🔍 [WEB_SEARCH] Parsing web search result from tool_call
     🔍 [WEB_SEARCH] Extracted X citations
     🔍 [CITATION DEBUG] Total sources: X
     🔍 [CITATION DEBUG] Source 0: id="https://...", name="...", source="web"
     🔍 [CITATION DEBUG] Has web search citations: true
     ```

4. **查看UI界面**
   - 应该看到蓝色调试框显示"🔍 网络搜索引用: X 个"
   - 应该看到引用显示组件，显示所有搜索结果链接
   - 可以点击链接在浏览器中打开

## 清理调试代码

验证成功后，可以移除以下调试代码：

### ai_text_message.dart

移除或注释掉：
- 蓝色调试信息框（第342-360行）
- 橙色调试信息框（第369-404行）
- `_hasWebSearchCitations` 中的 print 语句
- `_extractWebSearchCitations` 中的 print 语句

保留正常的引用显示逻辑：

```dart
// 网络搜索结果引用显示
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

## 后续改进建议

1. **优化引用提取**
   - 当前从文本字符串中提取引用，可以考虑在Rust后端直接返回结构化的引用数据

2. **添加更多元数据**
   - 搜索结果的snippet（摘要）
   - 相关性评分
   - 发布时间等

3. **错误处理**
   - 处理malformed的搜索结果
   - 处理无效的URL

4. **性能优化**
   - 缓存解析结果
   - 优化正则表达式匹配

## 技术细节

### 数据流

```
Rust Backend (web_search)
  ↓ (tool_call result)
parseMetadata (提取URL)
  ↓ (ChatMessageRefSource with source='web')
ChatAIMessageBloc (state.sources)
  ↓
_hasWebSearchCitations (检查source='web' && isURL)
  ↓
_extractWebSearchCitations (转换为CitationInfo)
  ↓
CitationDisplay (显示引用)
```

### 关键判断条件

引用被识别为"网络搜索引用"需要满足：
1. `source == 'web'`
2. `id` 字段是有效的URL（通过 `isURL()` 验证）

这样可以区分网络搜索引用和普通的文档来源。

