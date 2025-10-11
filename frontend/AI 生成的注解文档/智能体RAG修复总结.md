# 智能体 + 文档 RAG 修复总结

## 🐛 问题

**用户报告：**
- ✅ 不选智能体 + 文档 → AI **能**使用文档
- ❌ 选择智能体 + 文档 → AI **不能**使用文档

---

## 🔍 原因

智能体模式的两个消息处理方法**未调用 RAG 文档检索**：

1. **`stream_answer_with_system_prompt`** (无工具的智能体)
2. **`stream_answer_with_auto_multi_turn`** (有工具的智能体)

这两个方法只调用了 `get_message_content(question_id)`，获取原始问题，**没有检索和添加文档上下文**。

---

## ✅ 修复

在这两个方法中添加 RAG 文档检索：

```rust
// ❌ 修复前
let content = self.get_message_content(question_id)?;

// ✅ 修复后
let question = self.get_message_content(question_id)?;
let content = self.get_message_content_with_rag(chat_id, &question).await?;
```

---

## 📝 修改文件

**文件：** `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`

**修改位置：**
- 第 288-289 行 (`stream_answer_with_system_prompt`)
- 第 1164-1165 行 (`stream_answer_with_auto_multi_turn`)

---

## 🎯 预期效果

修复后：
- ✅ **智能体 + 文档** → AI 能使用文档回答
- ✅ **智能体 + 工具 + 文档** → AI 能同时使用工具和文档
- ✅ **无智能体 + 文档** → 保持原有功能（不受影响）

---

## 🚀 下一步

1. **等待编译完成**
2. **运行应用测试：**
   ```bash
   cd appflowy_flutter
   flutter run
   ```
3. **测试场景：**
   - 创建文档，写入测试内容
   - 选择智能体 + 选择文档
   - 提问文档相关问题
   - **验证：** AI 应该使用文档内容回答

---

**修复时间：** 2025-10-11  
**修复状态：** ✅ 代码已修复，编译中...

