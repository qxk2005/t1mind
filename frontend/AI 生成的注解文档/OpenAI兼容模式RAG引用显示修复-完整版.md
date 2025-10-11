# OpenAI兼容模式RAG引用显示修复 - 完整修复版

## 问题总结

OpenAI兼容模式下，虽然RAG功能正常（检索文档、增强prompt），但前端UI没有显示文档引用列表。

## 修复内容汇总

### 涉及的函数

1. **`get_message_content_with_rag`**（第104-171行）
   - 修改返回类型，同时返回content和documents

2. **`openai_chat_stream_with_system`**（第492-750行）
   - 添加 `rag_documents` 参数
   - 在stream结束后发送document metadata

3. **`stream_answer_with_system_prompt`**（第288-352行）
   - 使用 `rag_documents` 而不是 `_rag_documents`
   - 传递给 `openai_chat_stream_with_system`

4. **`stream_answer_with_multi_turn`**（第1140-1587行）
   - 在stream结束后发送document metadata

5. **`stream_answer`**（第1605-1682行）
   - 适配新的返回类型

### 完整修改列表

| 文件 | 行号 | 修改内容 |
|------|------|----------|
| chat_service_mw.rs | 104-171 | 修改 `get_message_content_with_rag` 返回类型 |
| chat_service_mw.rs | 291 | 使用 `rag_documents` 而非 `_rag_documents` |
| chat_service_mw.rs | 324 | 传递 `rag_documents.clone()` |
| chat_service_mw.rs | 341-343 | 传递 `rag_documents` |
| chat_service_mw.rs | 492-500 | 添加 `rag_documents` 参数 |
| chat_service_mw.rs | 719-747 | 发送document metadata |
| chat_service_mw.rs | 755 | 传递 `Vec::new()` |
| chat_service_mw.rs | 1167 | 获取 `rag_documents` |
| chat_service_mw.rs | 1525-1553 | 发送document metadata |
| chat_service_mw.rs | 1647 | 传递 `_rag_documents` |

## 测试验证

### 预期日志

**后端**：
```
[RAG] 📚 OpenAI 兼容模式：检索文档 - rag_ids=[...]
[RAG] 📖 OpenAI 兼容模式：找到 1 个相关文档片段
stream_answer_with_system_prompt use model: ... has_tools: false
[RAG] 📤 发送 1 个文档来源的 metadata
[RAG] ✅ 已发送 1 个文档来源 metadata
```

**前端**：
```
📊 [METADATA] Parsing metadata: {"SOURCE_ID":"...","SOURCE":"appflowy",...}
📄 [DOC_RETRIEVAL] Found document reference: id=..., source=appflowy
📊 [METADATA] Parsed 1 sources: appflowy:Loading...
```

### 测试步骤

1. 重新编译：`cd rust-lib && cargo build`
2. 重启应用
3. 使用OpenAI兼容模式
4. 选择文档并提问
5. 验证UI显示引用列表

## 技术要点

1. **两个流程都需要支持**：
   - `stream_answer_with_system_prompt` → `openai_chat_stream_with_system`（无工具）
   - `stream_answer_with_multi_turn`（有工具）

2. **Metadata格式保持一致**：
   ```json
   {
     "SOURCE_ID": "uuid",
     "SOURCE": "appflowy",
     "SOURCE_NAME": "document"
   }
   ```

3. **去重逻辑**：使用HashMap按object_id去重

4. **位置**：metadata必须在stream结束后发送（在 `try_stream` 的末尾）

## 完成状态

✅ 所有引用类型现已支持：
- ✅ 网络搜索引用
- ✅ MCP工具引用
- ✅ Appflowy文档引用（本地AI模式）
- ✅ Appflowy文档引用（OpenAI兼容模式）

🎉 引用显示功能完全修复！

