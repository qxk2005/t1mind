# RAG 文档引用 UI 显示修复

## 问题描述

在 AI 聊天中使用 RAG（检索增强生成）功能时，虽然 AI 能够成功使用文档内容回答问题，但 UI 中没有显示引用的文档列表。

**日志证据**：
```
[RAG] 📖 OpenAI 兼容模式：找到 1 个相关文档片段
[RAG] ✅ OpenAI 兼容模式：找到 1 个文档片段，将添加到 system prompt
flutter: 📊 [METADATA] Parsing metadata: []
flutter: 📊 [METADATA] Parsed 0 sources:
```

## 根本原因

问题在于后端发送的 metadata 格式与前端期望的格式不匹配：

### 后端发送的格式（修复前）
```rust
json!({
  "id": object_id,
  "source": "appflowy",
  "name": document_name
})
```

### 前端期望的格式
```dart
// chat_message_service.dart 第 113-127 行
else if (map.containsKey("SOURCE_ID") && map["SOURCE_ID"] != null) {
  // 处理文档检索引用（RAG）
  final sourceId = map["SOURCE_ID"].toString();
  final source = map["SOURCE"]?.toString() ?? "appflowy";
  
  metadata.add(ChatMessageRefSource(
    id: sourceId,
    name: "Loading...",
    source: source,
  ));
}
```

## 解决方案

修改 `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs` 中的 metadata 格式，使其与前端期望的格式匹配：

### 修改内容

**文件**: `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`
**位置**: 第 822-830 行和第 1561-1569 行

**修改前**:
```rust
// 构建 metadata，格式与前端期望的 id/source/name 匹配
deduplicated_sources.insert(
  object_id.to_string(),
  json!({
    "id": object_id,
    "source": "appflowy",
    "name": document_name
  })
);
```

**修改后**:
```rust
// 构建 metadata，格式与前端期望的 SOURCE_ID/SOURCE/SOURCE_NAME 匹配
deduplicated_sources.insert(
  object_id.to_string(),
  json!({
    "SOURCE_ID": object_id,
    "SOURCE": "appflowy",
    "SOURCE_NAME": document_name
  })
);
```

## 修复效果

修复后，当使用 RAG 功能时：

1. **后端**：成功检索文档并将内容添加到 system prompt
2. **后端**：发送正确格式的 metadata 到前端
3. **前端**：正确解析 metadata 并创建 `ChatMessageRefSource`
4. **UI**：显示文档引用列表，用户可以点击查看原始文档

## 测试验证

修复后，日志应该显示：
```
[RAG] 📖 OpenAI 兼容模式：找到 1 个相关文档片段
flutter: 📄 [DOC_RETRIEVAL] Found document reference: id=xxx, source=appflowy
flutter: 📊 [METADATA] Parsed 1 sources:
```

UI 中应该显示文档引用卡片，用户可以点击查看原始文档。

## 相关文件

- **后端**: `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`
- **前端**: `appflowy_flutter/lib/plugins/ai_chat/application/chat_message_service.dart`
- **UI组件**: `appflowy_flutter/lib/plugins/ai_chat/widgets/unified_reference_display.dart`

