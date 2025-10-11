# 智能体模式下 RAG 文档检索问题修复

## 🐛 问题描述

**现象：**
- ✅ 不选择智能体时：AI 可以使用选中的文档回答问题
- ❌ 选择智能体时：AI 无法使用选中的文档回答问题

**用户报告：**
> "我在AI 聊天中选择了智能体，也选择了文档，但是不会使用文档回答问题。反而是不选择智能体可以使用文档回答问题"

---

## 🔍 根本原因

在 `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs` 中，智能体模式使用的两个方法**没有调用 RAG 文档检索**：

### 1. `stream_answer_with_system_prompt` (第 288 行)

```rust
// ❌ 修复前：只获取原始消息，不检索 RAG 文档
let content = self.get_message_content(question_id)?;
```

**用途：**
- 智能体模式，但没有启用工具时使用

---

### 2. `stream_answer_with_auto_multi_turn` (第 1162 行)

```rust
// ❌ 修复前：只获取原始消息，不检索 RAG 文档
let content = self.get_message_content(question_id)?;
```

**用途：**
- 智能体模式 + 工具调用时使用
- 支持自动多轮对话

---

## ✅ 修复方案

将这两个方法中的 `get_message_content` 替换为 `get_message_content_with_rag`：

### 修复 1：`stream_answer_with_system_prompt`

```rust
// ✅ 修复后：先获取问题文本，再检索 RAG 文档
let question = self.get_message_content(question_id)?;
let content = self.get_message_content_with_rag(chat_id, &question).await?;
```

### 修复 2：`stream_answer_with_auto_multi_turn`

```rust
// ✅ 修复后：先获取问题文本，再检索 RAG 文档
let question = self.get_message_content(question_id)?;
let content = self.get_message_content_with_rag(chat_id, &question).await?;
```

**注意：** `get_message_content_with_rag` 的第二个参数类型是 `&str`（问题文本），需要先用 `get_message_content(question_id)` 获取问题内容。

---

## 🔧 `get_message_content_with_rag` 方法说明

该方法位于 `chat_service_mw.rs` 第 108-169 行，负责：

1. **获取原始问题**
   ```rust
   let question = self.get_message_content(question_id)?.to_string();
   ```

2. **检索 RAG IDs**
   ```rust
   let rag_ids = select_chat_rag_ids(&mut conn, chat_id)?;
   ```

3. **使用嵌入调度器搜索文档**
   ```rust
   let results = scheduler
     .search_with_filter(&workspace_id, query, 5, Some(rag_ids.to_vec()))
     .await?;
   ```

4. **构建增强的消息**
   ```rust
   let enhanced_message = format!(
     r#"Use the following context to answer the question. Only use information from the context provided.

##Context##
{}

##Question##
{}"#,
     context, question
   );
   ```

---

## 📊 修复验证

### 测试场景

**场景 1：智能体 + 文档**
- ✅ 选择智能体
- ✅ 选择文档
- ✅ 提问：文档内容相关问题
- **预期：** AI 使用文档内容回答

**场景 2：智能体 + 工具 + 文档**
- ✅ 选择智能体
- ✅ 启用工具调用
- ✅ 选择文档
- ✅ 提问：需要工具和文档的复杂问题
- **预期：** AI 结合工具和文档回答

**场景 3：无智能体 + 文档**
- ❌ 不选择智能体
- ✅ 选择文档
- ✅ 提问：文档内容相关问题
- **预期：** AI 使用文档内容回答（原本就正常）

---

## 🎯 技术细节

### 智能体模式的消息流

```
用户发送消息（选择了智能体+文档）
    ↓
ai_manager.rs::stream_chat_message
    ↓
chat.rs::stream_chat_message
    ↓
chat.rs::stream_response
    ↓
    ├─ 有工具？ → stream_answer_with_auto_multi_turn (✅ 已修复)
    └─ 无工具？ → stream_answer_with_system_prompt (✅ 已修复)
         ↓
    get_message_content_with_rag  ← 检索 RAG 文档
         ↓
    调用 OpenAI 兼容服务器（带文档上下文）
```

### 普通模式的消息流（原本就正常）

```
用户发送消息（未选择智能体+文档）
    ↓
ai_manager.rs::stream_chat_message
    ├─ Local AI 就绪？
    │   ├─ 是 → local_ai.stream_question (✅ 本地 RAG)
    │   └─ 否 → 回退到云服务
    └─ OpenAI 兼容模式？
        └─ openai_chat_stream_with_system (✅ get_message_content_with_rag)
```

---

## 📝 相关代码文件

1. **`rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`**
   - `stream_answer_with_system_prompt` (第 277-368 行)
   - `stream_answer_with_auto_multi_turn` (第 1129-1519 行)
   - `get_message_content_with_rag` (第 108-169 行)

2. **`rust-lib/flowy-ai/src/chat.rs`**
   - `stream_response` (第 250-488 行)

3. **`rust-lib/flowy-ai/src/ai_manager.rs`**
   - `stream_chat_message` (第 438-687 行)

---

## 🚀 后续测试建议

### 测试步骤

1. **编译应用**
   ```bash
   cd /Users/niuzhidao/Documents/Program/t1mind/frontend
   cargo make --profile development-mac-arm64 appflowy-core-dev
   cd appflowy_flutter
   flutter run
   ```

2. **准备测试数据**
   - 创建一个文档，写入一些测试内容（如"公司名称是 XYZ"）
   - 创建一个智能体

3. **测试智能体 + 文档**
   - 选择智能体
   - 选择刚才创建的文档
   - 提问："公司名称是什么？"
   - **预期：** AI 回答"公司名称是 XYZ"（而不是说"我不知道"）

4. **测试智能体 + 工具 + 文档**
   - 选择智能体
   - 启用工具调用（如网络搜索）
   - 选择文档
   - 提问一个复杂问题（既需要文档，又可能需要工具）
   - **预期：** AI 正确使用文档和工具回答

---

## ✅ 修复完成

**修复日期：** 2025-10-11  
**修复文件数：** 1 (`chat_service_mw.rs`)  
**修复行数：** 2 行  
**影响范围：** 智能体模式下的 RAG 文档检索

---

## 🔗 相关问题

- [x] RAG 文档在本地 AI 模式下工作正常
- [x] RAG 文档在 OpenAI 兼容模式（无智能体）下工作正常
- [x] RAG 文档在智能体模式下不工作 ← **本次修复**

---

**修复状态：** ✅ 已完成，等待编译和测试验证

