# RAG 文档引用 UI 显示修复 - 多轮对话版本

## 问题描述

在 AI 聊天中使用 RAG（检索增强生成）功能时，虽然 AI 能够成功使用文档内容回答问题，但 UI 中没有显示引用的文档列表。

**具体场景**：
- 使用智能体（Agent）进行多轮对话
- AI 调用了工具（如 web_search）
- RAG 功能正常工作，找到了相关文档片段
- 但 UI 中只显示网络搜索引用，没有显示文档引用

**日志证据**：
```
[RAG] 📖 OpenAI 兼容模式：找到 1 个相关文档片段
[RAG] ✅ OpenAI 兼容模式：找到 1 个文档片段，将添加到 system prompt
[RAG] 📋 已将 1 个文档片段添加到 system prompt (898字符)
🔄 [AUTO-MULTI-TURN] Detected tool: web_search
✅ web_search 完成 (7ms)
```

## 根本原因

问题在于多轮对话的流程中，RAG 文档的 metadata 发送时机不正确：

### 原始流程
1. AI 调用了 `web_search` 工具
2. 工具执行完成后，继续下一轮
3. 在下一轮中，AI 生成了最终回答（没有工具调用）
4. 代码检测到有内容生成，就 `break` 退出了循环
5. **RAG 文档的 metadata 是在循环结束后才发送的，但循环已经退出了**

### 问题代码位置
**文件**: `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`
**位置**: 第 1542-1545 行和第 1549-1581 行

```rust
// 问题：在循环内部检测到内容生成时就退出
if has_content || is_final_iteration {
  info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations", iteration);
  break; // 直接退出，没有执行到后面的 RAG metadata 发送代码
}

// 问题：RAG metadata 发送代码在循环外部，永远不会执行
if !rag_documents.is_empty() {
  // 发送 RAG 文档 metadata...
}
```

## 解决方案

将 RAG 文档 metadata 的发送逻辑移到循环内部，在检测到对话结束时立即发送：

### 修改内容

**修改前**:
```rust
// 无工具调用回退可用：若本轮产生了内容或达到最终迭代，则视为完成
if has_content || is_final_iteration {
  info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations", iteration);
  break;
}

// RAG metadata 发送代码在循环外部（永远不会执行）
if !rag_documents.is_empty() {
  // 发送 metadata...
}
```

**修改后**:
```rust
// 无工具调用回退可用：若本轮产生了内容或达到最终迭代，则视为完成
if has_content || is_final_iteration {
  info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations", iteration);
  
  // 🔧 在对话结束前发送文档来源 metadata（如果有检索到的文档）
  if !rag_documents.is_empty() {
    info!("[RAG] 📤 发送 {} 个文档来源的 metadata", rag_documents.len());
    
    // 使用 HashMap 去重（按 object_id）
    let mut deduplicated_sources: HashMap<String, serde_json::Value> = HashMap::new();
    for doc in &rag_documents {
      if let Some(object_id) = doc.metadata.get("object_id").and_then(|v| v.as_str()) {
        let document_name = "document".to_string();
        
        // 构建 metadata，格式与前端期望的 SOURCE_ID/SOURCE/SOURCE_NAME 匹配
        deduplicated_sources.insert(
          object_id.to_string(),
          json!({
            "SOURCE_ID": object_id,
            "SOURCE": "appflowy",
            "SOURCE_NAME": document_name
          })
        );
      }
    }
    
    // 发送每个文档来源的 metadata
    for source_meta in deduplicated_sources.values() {
      yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
        value: source_meta.clone()
      };
    }
    
    info!("[RAG] ✅ 已发送 {} 个文档来源 metadata", deduplicated_sources.len());
  }
  
  break;
}
```

## 修复效果

修复后，当使用多轮对话模式时：

1. **RAG 功能正常工作**：成功检索文档并将内容添加到 system prompt
2. **工具调用正常工作**：AI 可以调用 web_search 等工具
3. **RAG 文档 metadata 正确发送**：在对话结束时立即发送文档引用信息
4. **UI 正确显示**：同时显示网络搜索引用和文档引用

## 测试验证

修复后，日志应该显示：
```
[RAG] 📖 OpenAI 兼容模式：找到 1 个相关文档片段
🔄 [AUTO-MULTI-TURN] Detected tool: web_search
✅ web_search 完成 (7ms)
🔄 [AUTO-MULTI-TURN] Completed after 2 iterations
[RAG] 📤 发送 1 个文档来源的 metadata
[RAG] ✅ 已发送 1 个文档来源 metadata
```

前端日志应该显示：
```
📄 [DOC_RETRIEVAL] Found document reference: id=xxx, source=appflowy
```

UI 中应该同时显示：
- **网络搜索引用**：来自 web_search 工具的搜索结果
- **文档引用**：来自 RAG 检索的 AppFlowy 文档

## 相关文件

- **后端**: `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`
- **前端**: `appflowy_flutter/lib/plugins/ai_chat/application/chat_message_service.dart`
- **UI组件**: `appflowy_flutter/lib/plugins/ai_chat/widgets/unified_reference_display.dart`

