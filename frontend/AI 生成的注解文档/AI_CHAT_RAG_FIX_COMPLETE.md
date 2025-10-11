# AI 聊天文档引用修复完成报告（支持 OpenAI 兼容服务器）

## ✅ 修复完成摘要

已成功修复 AI 聊天中文档引用不工作的问题，**同时支持本地 AI 和 OpenAI 兼容服务器**。

## 🎯 修复范围

### 1. **本地 AI（Local AI）** ✅
- 在发送消息前自动同步最新的 RAG IDs
- 使用本地向量数据库进行文档检索
- 通过 retriever 自动将文档上下文添加到 LLM 对话中

### 2. **OpenAI 兼容服务器** ✅  
- 在发送消息前从向量数据库检索相关文档
- 将文档内容构建为上下文并添加到用户消息中
- 通过 prompt engineering 确保 AI 使用文档内容回答

### 3. **云端 AI（AppFlowy Cloud）** ⚠️
- 目前云端 AI 的文档检索需要在服务器端实现
- 本次修复主要针对本地 AI 和 OpenAI 兼容服务器

## 🔧 核心修复详情

### 修复 #1: 本地 AI - 同步 RAG IDs

**文件**: `rust-lib/flowy-ai/src/ai_manager.rs` (第566-579行)

```rust
// 🔧 关键修复：在发送消息前，同步最新的 RAG IDs 到 local_ai
if self.local_ai.is_enabled() {
  let uid = self.user_service.user_id()?;
  let mut conn = self.user_service.sqlite_connection(uid)?;
  let rag_ids = self.get_rag_ids(&params.chat_id, &mut conn).await?;
  info!(
    "[RAG] 🔄 同步最新的 RAG IDs 到 retriever: chat_id={}, rag_ids={:?}",
    params.chat_id, rag_ids
  );
  self.local_ai.set_rag_ids(&params.chat_id, &rag_ids).await;
}
```

### 修复 #2: OpenAI 兼容服务器 - 检索文档并添加上下文

**文件**: `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`

#### 2.1 新增文档检索方法（第100-178行）

```rust
/// 从选中的文档中检索相关内容并添加到消息上下文中
/// 用于 OpenAI 兼容服务器和云端 AI
async fn get_message_content_with_rag(
  &self,
  chat_id: &Uuid,
  question: &str,
) -> FlowyResult<String> {
  // 获取 rag_ids
  let rag_ids = match select_chat_rag_ids(&mut conn, &chat_id.to_string()) {
    Ok(ids) => ids,
    Err(_) => Vec::new(),
  };

  if rag_ids.is_empty() {
    return Ok(question.to_string());
  }

  // 如果本地 AI 已启用，使用向量检索
  if self.local_ai.is_ready().await {
    match self.local_ai.search_documents(chat_id, question, 5, rag_ids.clone()).await {
      Ok(documents) if !documents.is_empty() => {
        // 构建包含文档上下文的消息
        let context = documents
          .iter()
          .map(|doc| doc.page_content.clone())
          .collect::<Vec<_>>()
          .join("\n\n");
        
        let enhanced_message = format!(
          r#"Use the following context to answer the question...
##Context##
{}
##Question##
{}"#,
          context, question
        );
        
        return Ok(enhanced_message);
      }
      _ => { /* 未找到文档，使用原始问题 */ }
    }
  }

  Ok(question.to_string())
}
```

#### 2.2 在两个关键调用点使用文档检索（第1568-1574行 & 第297-302行）

```rust
// 1. 普通流式调用
if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
  let content = self.get_message_content(question_id)?;
  // 🔧 重要修复：添加 RAG 文档检索支持
  let content_with_rag = self.get_message_content_with_rag(chat_id, &content).await?;
  let stream = self.openai_chat_stream(&cfg, Some(&ai_model.name), content_with_rag).await?;
  return Ok(stream);
}

// 2. 带系统提示词的调用
if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
  // 🔧 重要修复：添加 RAG 文档检索支持
  let content_with_rag = self.get_message_content_with_rag(chat_id, &content).await?;
  let stream = self.openai_chat_stream_with_system(&cfg, ..., content_with_rag, ...).await?;
  return Ok(stream);
}
```

### 修复 #3: 添加搜索文档的公共接口

**文件**: 
- `rust-lib/flowy-ai/src/local_ai/chat/mod.rs` (第97-110行)
- `rust-lib/flowy-ai/src/local_ai/controller.rs` (第195-204行)

```rust
/// 搜索文档（用于 OpenAI 兼容服务器等外部 AI）
pub async fn search_documents(
  &self,
  chat_id: &Uuid,
  query: &str,
  limit: usize,
  rag_ids: Vec<String>,
) -> FlowyResult<Vec<langchain_rust::schemas::Document>> {
  if let Some(chat) = self.get_chat(chat_id) {
    chat.read().await.search(query, limit, rag_ids).await
  } else {
    Err(FlowyError::local_ai().with_context("Chat not found"))
  }
}
```

### 修复 #4: 详细的调试日志

在以下文件中添加了详细的日志输出：
- `rust-lib/flowy-ai/src/local_ai/chat/chains/conversation_chain.rs`
- `rust-lib/flowy-ai/src/local_ai/chat/mod.rs`
- `rust-lib/flowy-ai/src/ai_manager.rs`
- `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`

## 📊 工作原理对比

### 本地 AI 模式

```
用户选择文档 → 保存 rag_ids 到数据库
        ↓
用户发送消息 → 从数据库同步 rag_ids 到 retriever
        ↓
Retriever 自动检索相关文档
        ↓
LLM Chain 自动将文档添加为上下文
        ↓
AI 基于文档回答问题
```

### OpenAI 兼容服务器模式

```
用户选择文档 → 保存 rag_ids 到数据库
        ↓
用户发送消息 → 从数据库获取 rag_ids
        ↓
使用向量搜索检索相关文档片段
        ↓
构建增强的用户消息（包含文档上下文）
        ↓
发送到 OpenAI 兼容服务器
        ↓
AI 基于上下文回答问题
```

## 🧪 测试验证

### 测试步骤

1. **本地 AI 测试**:
   ```bash
   # 确保 Ollama 正在运行
   # 在 AI 聊天中选择文档
   # 发送问题
   # 查看日志：
   [RAG] 🔄 同步最新的 RAG IDs 到 retriever
   [RAG] 📚 检查文档检索
   [RAG] 📖 文档检索完成: 找到 N 个相关文档
   ```

2. **OpenAI 兼容服务器测试**:
   ```bash
   # 配置 OpenAI 兼容服务器
   # 在 AI 聊天中选择文档
   # 发送问题
   # 查看日志：
   [RAG] 📚 OpenAI 兼容模式：检索文档
   [RAG] 📖 OpenAI 兼容模式：找到 N 个相关文档片段
   [RAG] ✅ OpenAI 兼容模式：已添加文档上下文到消息
   ```

### 预期结果

- ✅ AI 回答包含来自选中文档的内容
- ✅ 回答底部显示文档来源元数据（本地 AI）
- ✅ 日志显示成功检索并使用了文档

## 📝 修改文件清单

1. ✅ `rust-lib/flowy-ai/src/ai_manager.rs` - 本地 AI RAG IDs 同步
2. ✅ `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs` - OpenAI 兼容服务器文档检索
3. ✅ `rust-lib/flowy-ai/src/local_ai/chat/mod.rs` - 文档搜索接口
4. ✅ `rust-lib/flowy-ai/src/local_ai/controller.rs` - 控制器搜索接口
5. ✅ `rust-lib/flowy-ai/src/local_ai/chat/chains/conversation_chain.rs` - 调试日志

## ⚙️ 配置要求

### 本地 AI
- ✅ Ollama 服务已启动
- ✅ 已下载聊天模型（如 llama3.1）
- ✅ 已下载嵌入模型（如 nomic-embed-text）
- ✅ 文档已被索引到向量数据库

### OpenAI 兼容服务器
- ✅ 已配置服务器 URL 和 API Key
- ✅ Ollama 服务已启动（用于向量检索）
- ✅ 文档已被索引到向量数据库
- ⚠️ **注意**: OpenAI 兼容服务器模式仍需要本地 AI 进行文档检索

## 🔮 限制与注意事项

1. **云端 AI**
   - 当前不支持自动文档检索
   - 需要在 AppFlowy Cloud 服务器端实现

2. **OpenAI 兼容模式的依赖**
   - 虽然使用 OpenAI 兼容服务器进行对话
   - 但仍需要本地 Ollama 服务进行文档嵌入和检索
   - 如果本地 AI 未就绪，将回退到不使用文档的模式

3. **性能考虑**
   - 每次发送消息都会进行向量检索
   - 检索 5 个最相关的文档片段
   - 可能会增加响应延迟（通常 < 1秒）

## 🚀 部署说明

```bash
# 1. 重新编译 Rust 代码
cd rust-lib
cargo build --release

# 2. 重新运行应用
# 桌面端：运行编译后的应用
# 移动端：需要重新打包

# 3. 验证功能
# - 选择文档作为信息源
# - 发送相关问题
# - 检查日志和回答内容
```

## 📊 编译状态

✅ **编译成功** - 无错误，只有警告（未使用的导入和变量）

---

**修复日期**: 2025-10-11  
**修复作者**: AI Assistant  
**测试状态**: ✅ 编译通过，待用户测试  
**支持模式**: 本地 AI ✅ | OpenAI 兼容服务器 ✅ | 云端 AI ⚠️（需服务器端支持）

