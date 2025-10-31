use crate::local_ai::controller::LocalAIController;
use flowy_ai_pub::persistence::{select_chat_rag_ids, select_message_content};
use std::collections::HashMap;

use flowy_ai_pub::cloud::{
  AIModel, ChatCloudService, ChatMessage, ChatMessageType, ChatSettings, CompleteTextParams,
  MessageCursor, ModelList, RelatedQuestion, RepeatedChatMessage, RepeatedRelatedQuestion,
  ResponseFormat, StreamAnswer, StreamComplete, UpdateChatParams, DEFAULT_AI_MODEL_NAME,
};
use flowy_error::{FlowyError, FlowyResult};
use lib_infra::async_trait::async_trait;

use flowy_ai_pub::user_service::AIUserService;
use flowy_storage_pub::storage::StorageService;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Weak};
use tracing::{error, info, trace, warn, debug};
use uuid::Uuid;
use flowy_sqlite::kv::KVStorePreferences;
use futures_util::StreamExt;
use async_stream::try_stream;
use serde_json::json;
use crate::entities::ToolDefinitionPB;
use serde::{Deserialize, Serialize};
use crate::agent::{
  TaskDecomposer, ParallelDispatcher, ResultSynthesizer, AvailableTools,
};

#[derive(Clone, Debug)]
struct OpenAICompatConfig {
  base_url: String,
  api_key: String,
  model: String,
  temperature: Option<f64>,
  max_tokens: Option<u32>,
}

/// OpenAI Tool Call 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
  pub id: String,
  #[serde(rename = "type")]
  pub tool_type: String,
  pub function: OpenAIFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCall {
  pub name: String,
  pub arguments: String, // JSON string
}

/// OpenAI Message 结构（用于多轮对话）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIMessage {
  pub role: String,
  pub content: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tool_calls: Option<Vec<OpenAIToolCall>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tool_call_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>, // tool name for tool role
}

pub struct ChatServiceMiddleware {
  cloud_service: Arc<dyn ChatCloudService>,
  user_service: Arc<dyn AIUserService>,
  local_ai: Arc<LocalAIController>,
  #[allow(dead_code)]
  storage_service: Weak<dyn StorageService>,
  // Used to read OpenAI 兼容服务器的配置（AppearanceSettingsPB.setting_key_value）
  store_preferences: Arc<KVStorePreferences>,
}

impl ChatServiceMiddleware {
  pub fn new(
    user_service: Arc<dyn AIUserService>,
    cloud_service: Arc<dyn ChatCloudService>,
    local_ai: Arc<LocalAIController>,
    storage_service: Weak<dyn StorageService>,
    store_preferences: Arc<KVStorePreferences>,
  ) -> Self {
    Self {
      user_service,
      cloud_service,
      local_ai,
      storage_service,
      store_preferences,
    }
  }

  /// 🔧 获取底层的 ChatCloudService
  pub fn cloud_service(&self) -> &Arc<dyn ChatCloudService> {
    &self.cloud_service
  }
  
  fn get_message_content(&self, message_id: i64) -> FlowyResult<String> {
    let uid = self.user_service.user_id()?;
    let conn = self.user_service.sqlite_connection(uid)?;
    let content = select_message_content(conn, message_id)?.ok_or_else(|| {
      FlowyError::record_not_found().with_context(format!("Message not found: {}", message_id))
    })?;
    Ok(content)
  }

  /// 从选中的文档中检索相关内容并添加到消息上下文中
  /// 用于 OpenAI 兼容服务器和云端 AI
  /// 
  /// 返回: (用户问题文本(保留@文档引用), 检索到的文档列表, 文档引用到名称的映射)
  async fn get_message_content_with_rag(
    &self,
    chat_id: &Uuid,
    question: &str,
  ) -> FlowyResult<(String, Vec<langchain_rust::schemas::Document>, std::collections::HashMap<String, String>)> {
    // 获取 rag_ids
    let uid = self.user_service.user_id()?;
    let mut conn = self.user_service.sqlite_connection(uid)?;
    let rag_ids = match select_chat_rag_ids(&mut conn, &chat_id.to_string()) {
      Ok(ids) => ids,
      Err(_) => Vec::new(),
    };

    // 提取 @文档引用和文档名称映射（保留 @文档引用在问题中）
    let (question_with_mentions, document_name_map) = self.extract_document_mentions(question, &rag_ids)?;

    if rag_ids.is_empty() {
      // trace!("[RAG] 📚 OpenAI 兼容模式：没有选择文档，直接使用用户问题");
      return Ok((question_with_mentions, Vec::new(), document_name_map));
    }

      info!(
        "[RAG] 📚 OpenAI 兼容模式：检索文档 - rag_ids={:?}",
        rag_ids
      );

    // 为了更好的检索效果，构建一个不包含 @符号但包含文档名称的查询
    // 例如 "从@文档1中获取姓名" -> "从文档1中获取姓名" 用于检索
    let query_for_retrieval = question_with_mentions
      .replace("@", "")
      .trim()
      .to_string();
    
    // 使用改进的查询进行检索
    match self.try_search_documents_via_embeddings(chat_id, &query_for_retrieval, &rag_ids).await {
      Ok(documents) if !documents.is_empty() => {
          info!(
            "[RAG] 📖 OpenAI 兼容模式：找到 {} 个相关文档片段",
            documents.len()
          );
        
        // 输出每个文档片段的详细信息
        for (idx, doc) in documents.iter().enumerate() {
          let score = doc.score;
          let preview = doc.page_content.chars().take(80).collect::<String>();
          trace!(
            "[RAG] 📄 片段 #{}: score={:.4}, preview='{}'...",
            idx + 1, score, preview
          );
        }
        
          info!(
            "[RAG] ✅ OpenAI 兼容模式：找到 {} 个文档片段，将添加到 system prompt",
            documents.len()
          );
        
        // 返回原始问题（包含 @文档引用）和文档列表
        return Ok((question_with_mentions, documents, document_name_map));
      }
      Ok(_) => {
          warn!(
            "[RAG] ⚠️ OpenAI 兼容模式：未找到相关文档（可能相似度分数低于阈值），使用原始问题"
          );
      }
      Err(err) => {
        warn!(
          "[RAG] ⚠️ OpenAI 兼容模式：文档检索失败: {}，使用原始问题",
          err
        );
      }
    }

    Ok((question_with_mentions, Vec::new(), document_name_map))
  }

  /// 从问题文本中提取 @文档引用，返回原始问题和文档名称映射
  /// 
  /// 支持的模式：
  /// - "@文档1"、"@文档2" 等中文文档名
  /// - "@document1"、"@doc2" 等英文文档名
  /// 
  /// 注意：保留 @文档引用在问题中，让模型能够理解文档的语义角色和上下文关系
  /// 同时在 system prompt 中明确说明 @文档引用是 RAG 文档名称，不是问题的一部分
  /// 
  /// 返回: (原始问题文本, 文档引用到名称的映射)
  fn extract_document_mentions(
    &self,
    question: &str,
    _rag_ids: &[String],
  ) -> FlowyResult<(String, std::collections::HashMap<String, String>)> {
    use std::collections::HashMap;
    
    let mut document_name_map = HashMap::new();
    
    // 使用简单的字符串扫描来查找 @文档引用
    // 匹配模式：@后面跟非空白、非@、非标点的字符序列
    let chars: Vec<char> = question.chars().collect();
    
    // 扫描：找到所有 @文档引用
    let mut i = 0;
    while i < chars.len() {
      if chars[i] == '@' && i + 1 < chars.len() {
        // 找到 @ 符号，查找文档名
        let start = i;
        let mut end = i + 1;
        
        // 文档名可以包含中文、英文、数字，但不能包含空白、@、某些标点
        let stop_chars = [' ', '\t', '\n', '\r', '@', '，', '。', '、', '；', '：'];
        
        while end < chars.len() && !stop_chars.contains(&chars[end]) {
          end += 1;
        }
        
        if end > start + 1 {
          // 找到了有效的 @文档引用
          let mention_text: String = chars[start..end].iter().collect();
          let document_name: String = chars[start + 1..end].iter().collect();
          
          if !document_name.is_empty() {
            document_name_map.insert(mention_text.clone(), document_name);
          }
        }
      }
      i += 1;
    }
    
    // 返回原始问题（保留 @文档引用）和映射
    Ok((question.to_string(), document_name_map))
  }
  
  /// 直接使用嵌入服务进行文档搜索
  /// 不依赖 local_ai.is_ready()，因为即使 Ollama 聊天客户端未初始化，
  /// 嵌入服务（无论是 Ollama 还是 OpenAI 兼容）仍然可能可用
  async fn try_search_documents_via_embeddings(
    &self,
    _chat_id: &Uuid,
    query: &str,
    rag_ids: &[String],
  ) -> FlowyResult<Vec<langchain_rust::schemas::Document>> {
    use crate::embeddings::context::EmbedContext;
    use langchain_rust::schemas::Document;
    use std::collections::HashMap;
    
    // 获取嵌入调度器
    let scheduler = EmbedContext::shared().get_scheduler()?;
    
    // 获取 workspace_id
    let workspace_id = self.user_service.workspace_id()?;
    
    // 🔧 在检索前，尝试从用户设置中读取最新的相似度阈值并更新
    if let Some(settings_json) = self.store_preferences.get_str("appearance_settings") {
      if let Ok(v) = serde_json::from_str::<serde_json::Value>(&settings_json) {
        let map = v
          .get("setting_key_value")
          .or_else(|| v.get("settingKeyValue"))
          .and_then(|v| v.as_object());
        
        if let Some(map) = map {
          let scoped = |k: &str| -> String { format!("{}.{}", k, workspace_id) };
          let get = |k: &str| -> Option<String> {
            let scoped_key = scoped(k);
            map.get(&scoped_key)
              .and_then(|v| v.as_str().map(|s| s.to_string()))
              .or_else(|| map.get(k).and_then(|v| v.as_str().map(|s| s.to_string())))
          };
          
          if let Some(threshold_str) = get("ai.openai.ragScoreThreshold") {
            if let Ok(new_threshold) = threshold_str.parse::<f32>() {
              scheduler.update_score_threshold(new_threshold);
            }
          }
        }
      }
    }
    
    trace!("[RAG] 🔍 使用嵌入调度器搜索文档: query='{}', rag_ids={:?}", query, rag_ids);
    
    // 使用调度器进行搜索，限制返回 5 个结果
    let results = scheduler
      .search_with_filter(&workspace_id, query, 5, Some(rag_ids.to_vec()))
      .await?;
    
    // 转换搜索结果为 Document 格式
    let documents: Vec<Document> = results
      .into_iter()
      .map(|item| {
        let mut metadata = HashMap::new();
        metadata.insert(
          "object_id".to_string(),
          serde_json::json!(item.object_id.to_string()),
        );
        
        Document {
          page_content: item.content,
          metadata,
          score: item.score,
        }
      })
      .collect();
    
    trace!("[RAG] ✅ 嵌入调度器返回 {} 个文档", documents.len());
    
    Ok(documents)
  }

  /// 构建包含系统提示词的消息数组
  /// OpenAI API 标准格式：独立的 system 和 user 消息
  /// 
  /// ⚠️ 关键优化：将 RAG 文档上下文添加到 system prompt 中，
  /// 并明确指示 AI 优先使用文档内容，其次才考虑工具调用
  /// 
  /// 同时明确说明 @文档引用的作用：它们是筛选范围，不是问题本身
  fn build_messages_with_system_prompt(
    &self,
    content: String,
    system_prompt: Option<String>,
    rag_documents: &[langchain_rust::schemas::Document],
    document_name_map: &std::collections::HashMap<String, String>,
  ) -> Vec<serde_json::Value> {
    let mut messages = Vec::new();
    
    // 构建增强的 system prompt
    let enhanced_system_prompt = if !rag_documents.is_empty() {
      // 提取文档内容并按文档分组（如果可能）
      let context = rag_documents
        .iter()
        .map(|doc| doc.page_content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
      
      // 构建文档引用说明（如果有 @文档引用）
      let mention_explanation = if !document_name_map.is_empty() {
        let document_list: Vec<String> = document_name_map.values().cloned().collect();
        let mention_examples: Vec<String> = document_name_map.keys().cloned().collect();
        format!(
          r#"

## 📌 IMPORTANT: Understanding @Document References in User Question:

The user's question contains @document_name references (e.g., {}).
These @mentions are **RAG document names/references**, NOT part of the actual question text.

### Critical Understanding:
1. **@document_name format** = RAG document identifier/filter
   - These identify WHICH documents from the RAG context to use
   - They specify the SCOPE/FILTER for document retrieval
   - They are metadata, NOT part of the semantic query itself

2. **The actual question** is the semantic meaning after understanding what @documents represent:
   - "@文档1" means "the document named 文档1 in the RAG context"
   - When user says "从@文档1中获取姓名", it means "extract names from the document called 文档1"
   - You should understand the semantic role each @document plays in the task

3. **Multiple @document references** indicate different documents may have different roles:
   - Pay attention to the context where each @document is mentioned
   - Understand the relationship between documents in the task
   - Example: "从@文档1中获取姓名，并且在@文档2中找到这些姓名的成绩"
     * @文档1 (document named '文档1') = source for extracting names/identities
     * @文档2 (document named '文档2') = source for finding grades/scores for those names
     * Task = combine information from both documents with specific semantic roles

4. **Your task**: 
   - Use the RAG document content provided below to answer the question
   - Understand that @document references point to specific documents in the RAG context
   - Extract information according to each document's semantic role as indicated by the question context
   - The @mentions help you understand WHICH documents to use for WHAT purpose

Referenced documents in question: {}
"#,
          mention_examples.join(", "),
          document_list.join(", ")
        )
      } else {
        String::new()
      };
      
      let rag_instruction = format!(
        r#"# 📚 IMPORTANT: Document Context Available

You have access to relevant documents that contain information to answer the user's question.{}
{}

## Priority Guidelines (READ CAREFULLY):
1. **FIRST PRIORITY**: Use the information from the provided documents below to answer questions
2. **SECOND PRIORITY**: Only use tools (web search, MCP tools, etc.) if:
   - The documents do NOT contain relevant information
   - Additional real-time or external data is needed
   - The user explicitly asks to use a specific tool

## Document Context:
{}

## Instructions:
- Analyze the document context carefully before deciding to use any tools
- If the answer is in the documents, provide it directly without using tools
- Cite the documents when answering based on their content
- Be explicit about whether you're using document knowledge or tool results
- When multiple documents are referenced, understand each document's semantic role in the task
"#,
        mention_explanation,
        if mention_explanation.is_empty() {
          "\nNote: The user may reference specific documents. These are scope filters, not part of the question text."
        } else {
          ""
        },
        context
      );
      
      info!(
        "[RAG] 📋 已将 {} 个文档片段添加到 system prompt ({}字符)",
        rag_documents.len(),
        rag_instruction.len()
      );
      
      // 合并原有 system prompt 和 RAG 指令
      match system_prompt {
        Some(original_prompt) => format!("{}\n\n{}", rag_instruction, original_prompt),
        None => rag_instruction,
      }
    } else {
      // 没有 RAG 文档，使用原始 system prompt
      system_prompt.unwrap_or_default()
    };
    
    // 添加 system 消息（如果有内容）
    if !enhanced_system_prompt.is_empty() {
      messages.push(json!({
        "role": "system",
        "content": enhanced_system_prompt
      }));
    }
    
    // 用户消息（保留 @文档引用，以便模型理解文档的语义角色和上下文关系）
    // System prompt 中已明确说明 @文档引用是 RAG 文档名称，不是问题的一部分
    messages.push(json!({
      "role": "user",
      "content": content
    }));
    
    messages
  }

  /// 将工具定义转换为 OpenAI tools 格式
  fn convert_tools_to_openai_format(tools: &[ToolDefinitionPB]) -> Vec<serde_json::Value> {
    tools.iter().map(|tool| {
      // 解析参数 schema
      let parameters_schema = if !tool.parameters_schema.is_empty() {
        serde_json::from_str::<serde_json::Value>(&tool.parameters_schema)
          .unwrap_or_else(|_| json!({
            "type": "object",
            "properties": {},
            "required": []
          }))
      } else {
        json!({
          "type": "object",
          "properties": {},
          "required": []
        })
      };

      json!({
        "type": "function",
        "function": {
          "name": tool.name,
          "description": tool.description,
          "parameters": parameters_schema
        }
      })
    }).collect()
  }

  /// 带系统提示词的流式应答
  pub async fn stream_answer_with_system_prompt(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
    system_prompt: Option<String>,
    tools: Option<Vec<ToolDefinitionPB>>,  // 🆕 添加工具参数
  ) -> Result<StreamAnswer, FlowyError> {
    // 获取消息内容（包含 RAG 文档检索）
    let question = self.get_message_content(question_id)?;
    let (content, rag_documents, document_name_map) = self.get_message_content_with_rag(chat_id, &question).await?;
    
    info!(
      "stream_answer_with_system_prompt use model: {:?}, has_system_prompt: {}, has_tools: {}",
      ai_model,
      system_prompt.is_some(),
      tools.is_some()
    );
    
    // 根据模型类型调用不同的服务
    if ai_model.is_local {
      if self.local_ai.is_ready().await {
        // 本地 AI: 简单合并系统提示词（本地模型可能不支持 system role）
        // 对于本地 AI，保留 @文档引用，在提示词中说明它们的作用
        let (_original_content, document_name_map) = self.extract_document_mentions(&content, &[])?;
        let mention_note = if !document_name_map.is_empty() {
          let docs: Vec<String> = document_name_map.values().cloned().collect();
          format!("\n\n注意：问题中的 @文档引用（如 {}) 是 RAG 文档名称，用于筛选检索范围，不是问题文本的一部分。", docs.join(", "))
        } else {
          String::new()
        };
        let final_content = if let Some(ref prompt) = system_prompt {
          format!("{}{}\n\n{}", prompt, mention_note, content)
        } else {
          format!("{}{}", mention_note, content)
        };
        self
          .local_ai
          .stream_question(chat_id, &final_content, format, &ai_model.name)
          .await
      } else {
        // Fallback to server provider
        match self
          .cloud_service
          .get_workspace_default_model(workspace_id)
          .await
        {
          Ok(name) => {
            let server_model = AIModel::server(name, String::new());
            if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
              let (_init_reasoning, stream) = self
                .openai_chat_stream_with_system(&cfg, Some(&server_model.name), content, system_prompt, tools.as_deref(), rag_documents.clone(), &document_name_map, Some(chat_id))
                .await?;
              return Ok(stream);
            }
            Err(FlowyError::local_ai_not_ready()
              .with_context("本地 AI 未就绪且无 OpenAI 兼容配置"))
          },
          Err(_) => Err(
            FlowyError::local_ai_not_ready()
              .with_context("本地 AI 未就绪 / Local AI not ready"),
          ),
        }
      }
    } else {
      // 如果配置了 OpenAI 兼容服务器，则优先直接调用（使用标准 system/user 消息格式）
      if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
        // 🔧 重要修复：添加 RAG 文档检索支持
        let (content_with_rag, rag_documents, document_name_map) = self.get_message_content_with_rag(chat_id, &content).await?;
        let (_init_reasoning, stream) = self
          .openai_chat_stream_with_system(&cfg, Some(&ai_model.name), content_with_rag, system_prompt, tools.as_deref(), rag_documents, &document_name_map, Some(chat_id))
          .await?;
        return Ok(stream);
      }

      // 🚫 AppFlowy Cloud AI 已禁用：只使用全局 AI 配置
      Err(FlowyError::internal()
        .with_context("未配置 OpenAI 兼容服务器。请在全局 AI 设置中配置 AI 供应商。"))
    }
  }

  fn read_openai_compat_chat_config(&self, workspace_id: &Uuid) -> Option<OpenAICompatConfig> {
    // Returns chat model config
    let settings_json = self.store_preferences.get_str("appearance_settings")?;
    let v: serde_json::Value = serde_json::from_str(&settings_json).ok()?;
    let map = v
      .get("setting_key_value")
      .or_else(|| v.get("settingKeyValue"))?
      .as_object()?;
    let scoped = |k: &str| -> String { format!("{}.{}", k, workspace_id) };

    let get = |k: &str| -> Option<String> {
      map.get(&scoped(k))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .or_else(|| map.get(k).and_then(|v| v.as_str().map(|s| s.to_string())))
    };

    let base_url = get("ai.openai.chatBaseUrl").or_else(|| get("ai.openai.baseUrl"))?;
    let api_key = get("ai.openai.apiKey").unwrap_or_default();
    let model = get("ai.openai.model").unwrap_or_else(|| "gpt-4o-mini".to_string());
    let temperature = get("ai.openai.temperature").and_then(|s| s.parse::<f64>().ok());
    let max_tokens = get("ai.openai.maxTokens").and_then(|s| s.parse::<u32>().ok());
    if base_url.is_empty() || api_key.is_empty() {
      return None;
    }
    Some(OpenAICompatConfig { base_url, api_key, model, temperature, max_tokens })
  }

  fn join_openai_url(base: &str, path: &str) -> String {
    if base.ends_with('/') {
      format!("{}{}", base.trim_end_matches('/'), path)
    } else {
      format!("{}/{}", base, path.trim_start_matches('/'))
    }
  }

  fn openai_chat_payload(model: &str, messages: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
      "model": model,
      "stream": true,
      "stream_options": {"include_reasoning": true},
      "messages": messages
    })
  }

  /// 提取推理文本与最终答案
  fn parse_reasoning_and_answer(v: &serde_json::Value) -> (Option<String>, Option<String>) {
    // chat.completions 风格
    if let Some(choices) = v.get("choices").and_then(|c| c.as_array()) {
      if let Some(first) = choices.get(0) {
        if let Some(text) = first
          .get("message")
          .and_then(|m| m.get("content"))
          .and_then(|s| s.as_str())
        {
          return (None, Some(text.to_string()));
        }
        // content 为数组的情况
        if let Some(arr) = first
          .get("message")
          .and_then(|m| m.get("content"))
          .and_then(|a| a.as_array())
        {
          let mut reasoning = String::new();
          let mut answer = String::new();
          for item in arr {
            let ty = item.get("type").and_then(|s| s.as_str()).unwrap_or("");
            if ty == "reasoning" {
              if let Some(t) = item.get("text").and_then(|s| s.as_str()) {
                reasoning.push_str(t);
              }
            } else if ty == "output_text" {
              if let Some(t) = item.get("text").and_then(|o| o.get("value")).and_then(|s| s.as_str()) {
                answer.push_str(t);
              }
            } else if ty == "text" {
              if let Some(t) = item.get("text").and_then(|s| s.as_str()) {
                answer.push_str(t);
              }
            }
          }
          return (
            if reasoning.is_empty() { None } else { Some(reasoning) },
            if answer.is_empty() { None } else { Some(answer) },
          );
        }
      }
    }

    // responses 风格
    if let Some(output) = v.get("output").and_then(|o| o.as_array()) {
      for item in output {
        if item.get("type").and_then(|s| s.as_str()) == Some("message") {
          if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
            let mut reasoning = String::new();
            let mut answer = String::new();
            for c in content {
              let ty = c.get("type").and_then(|s| s.as_str()).unwrap_or("");
              match ty {
                "reasoning" => {
                  if let Some(t) = c
                    .get("reasoning")
                    .and_then(|r| r.get("text"))
                    .and_then(|s| s.as_str())
                  {
                    reasoning.push_str(t);
                  }
                },
                "output_text" => {
                  if let Some(t) = c
                    .get("text")
                    .and_then(|o| o.get("value"))
                    .and_then(|s| s.as_str())
                  {
                    answer.push_str(t);
                  }
                },
                "text" => {
                  if let Some(t) = c.get("text").and_then(|s| s.as_str()) {
                    answer.push_str(t);
                  }
                },
                _ => {},
              }
            }
            return (
              if reasoning.is_empty() { None } else { Some(reasoning) },
              if answer.is_empty() { None } else { Some(answer) },
            );
          }
        }
      }
    }
    (None, None)
  }

  /// 带系统提示词和工具的 OpenAI 兼容流式调用（支持 Function Call）
  async fn openai_chat_stream_with_system(
    &self,
    cfg: &OpenAICompatConfig,
    model: Option<&str>,
    content: String,
    system_prompt: Option<String>,
    tools: Option<&[ToolDefinitionPB]>,  // 🆕 添加工具参数
    rag_documents: Vec<langchain_rust::schemas::Document>,  // 🆕 RAG文档列表（用于发送metadata）
    document_name_map: &std::collections::HashMap<String, String>,  // 🆕 文档名称映射（用于理解@文档引用）
    chat_id: Option<&Uuid>,  // 🆕 聊天ID（用于获取rag_ids，补充所有文档引用）
  ) -> Result<(Option<String>, StreamAnswer), FlowyError> {
    info!("🔧 [AI-SERVICE] 🚀 openai_chat_stream_with_system called with model: {:?}, system_prompt: {}, tools: {}", 
          model, system_prompt.is_some(), tools.is_some());
    let url = Self::join_openai_url(&cfg.base_url, "/v1/chat/completions");
    
    // 处理模型名称：如果是 "Auto" 或空，则使用配置中的模型
    let model_name = match model {
      Some(name) if !name.is_empty() && name != DEFAULT_AI_MODEL_NAME => name,
      _ => &cfg.model,
    };
    
    info!(
      "[OpenAI] Using model: {} (original: {:?}, config: {})",
      model_name,
      model,
      cfg.model
    );
    
    // 🔧 修复：在 content 被移动到 build_messages_with_system_prompt 之前，先提取文档名
    // 根据问题文本中@文档名的出现顺序构建有序的文档名列表
    let mut document_names_ordered: Vec<String> = Vec::new();
    let question_chars: Vec<char> = content.chars().collect();
    {
      let mut i = 0;
      while i < question_chars.len() {
        if question_chars[i] == '@' && i + 1 < question_chars.len() {
          let start = i;
          let mut end = i + 1;
          let stop_chars = [' ', '\t', '\n', '\r', '@', '，', '。', '、', '；', '：'];
          while end < question_chars.len() && !stop_chars.contains(&question_chars[end]) {
            end += 1;
          }
          if end > start + 1 {
            let document_name: String = question_chars[start + 1..end].iter().collect();
            if !document_name.is_empty() && !document_names_ordered.contains(&document_name) {
              document_names_ordered.push(document_name);
            }
          }
        }
        i += 1;
      }
    }
    
    // 如果没有从问题文本中提取到文档名，fallback到document_name_map中的值
    if document_names_ordered.is_empty() {
      document_names_ordered = document_name_map.values().cloned().collect();
    }
    
    // 构建包含系统提示词的消息数组（使用标准 OpenAI 格式）
    // ⚠️ 传入 rag_documents 和 document_name_map，将 RAG 上下文添加到 system prompt
    let messages = self.build_messages_with_system_prompt(content, system_prompt, &rag_documents, document_name_map);
    let mut payload = Self::openai_chat_payload(model_name, messages);
    
    // 🆕 添加工具定义（使用 OpenAI Function Call API）
    if let Some(tool_list) = tools {
      if !tool_list.is_empty() {
        let openai_tools = Self::convert_tools_to_openai_format(tool_list);
        payload.as_object_mut().unwrap().insert("tools".into(), json!(openai_tools));
        // 让模型自动决定是否调用工具
        payload.as_object_mut().unwrap().insert("tool_choice".into(), json!("auto"));
        info!("[OpenAI] Added {} tools to request", tool_list.len());
      }
    }
    
    // 添加可选参数
    if let Some(temp) = cfg.temperature {
      payload.as_object_mut().unwrap().insert("temperature".into(), json!(temp));
    }
    if let Some(max_tok) = cfg.max_tokens {
      payload.as_object_mut().unwrap().insert("max_tokens".into(), json!(max_tok));
    }
    
    info!("[OpenAI] Requesting {} with model: {}", url, model_name);
    
    let client = reqwest::Client::new();
    let resp = client
      .post(&url)
      .header("Content-Type", "application/json")
      .header("Authorization", format!("Bearer {}", cfg.api_key))
      .header("Accept", "text/event-stream")
      .json(&payload)
      .send()
      .await
      .map_err(|e| {
        error!("[OpenAI] Request failed: {}", e);
        FlowyError::server_error().with_context(e.to_string())
      })?;
    
    if !resp.status().is_success() {
      error!("[OpenAI] Non-200 response: {}", resp.status());
      return Err(FlowyError::server_error()
        .with_context(format!("OpenAI compat error: {}", resp.status())));
    }
    
    info!("🔧 [AI-SERVICE] Response status: {}, headers: {:?}", resp.status(), resp.headers());
    
    // 🔧 修复生命周期问题：在进入 try_stream! 之前，先获取所有需要的数据并克隆
    // 首先，从检索到的文档片段中提取文档引用
    let mut deduplicated_sources: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    let default_document_name = document_name_map.values().next()
      .cloned()
      .unwrap_or_else(|| "document".to_string());
    
    for doc in &rag_documents {
      if let Some(object_id) = doc.metadata.get("object_id").and_then(|v| v.as_str()) {
        info!(
          "[RAG] 📄 从检索结果提取文档引用: object_id={}, score={:.4}",
          object_id,
          doc.score
        );
        // 🔧 修复：后端只发送SOURCE_ID，不发送SOURCE_NAME，让前端通过ID自动获取文档名称
        let source_meta = json!({
          "SOURCE_ID": object_id,
          "SOURCE": "appflowy"
        });
        deduplicated_sources.insert(object_id.to_string(), source_meta);
      } else {
        warn!(
          "[RAG] ⚠️ 检索结果中的文档片段缺少 object_id metadata: score={:.4}",
          doc.score
        );
      }
    }
    
    // 🔧 修复问题：补充所有在 rag_ids 中但不在 rag_documents 中的文档
    // 在进入 try_stream! 之前获取 rag_ids，避免生命周期问题
    // 注意：document_names_ordered 已经在第760-786行提取完成，这里直接使用
    let document_names_for_closure = document_names_ordered.clone();
    let default_doc_name_for_closure = default_document_name.clone();
    let get_document_name_by_index = move |index: usize| -> String {
      if index < document_names_for_closure.len() {
        document_names_for_closure[index].clone()
      } else if !document_names_for_closure.is_empty() {
        // 如果索引超出范围，使用最后一个文档名（比第一个更合理）
        document_names_for_closure[document_names_for_closure.len() - 1].clone()
      } else {
        default_doc_name_for_closure.clone()
      }
    };
    
    // 🔧 修复：只在有@提及时才补充rag_ids中的其他文档
    // 当没有@提及时，只发送实际检索到的文档，避免发送错误的文档ID
    if !document_names_ordered.is_empty() {
      // 有@提及：补充rag_ids中的所有文档（用户明确指定的）
      if let Some(chat_id) = chat_id {
        let uid = self.user_service.user_id().ok();
        if let Some(uid) = uid {
          if let Ok(mut conn) = self.user_service.sqlite_connection(uid) {
            if let Ok(rag_ids) = select_chat_rag_ids(&mut conn, &chat_id.to_string()) {
              trace!(
                "[RAG] 从数据库获取 rag_ids: {:?}, 已从检索结果提取: {:?}",
                rag_ids,
                deduplicated_sources.keys().collect::<Vec<_>>()
              );
              
              let object_ids_from_rag_docs: std::collections::HashSet<String> = deduplicated_sources.keys().cloned().collect();
              let mut added: Vec<String> = Vec::new();
              
              // 🔧 修复：按顺序处理rag_ids，并根据索引从document_names_ordered中获取对应的文档名
              for (index, rag_id) in rag_ids.iter().enumerate() {
                if !object_ids_from_rag_docs.contains(rag_id) {
                  // 🔧 修复：后端只发送SOURCE_ID，不发送SOURCE_NAME，让前端通过ID自动获取文档名称
                  let source_meta = json!({
                    "SOURCE_ID": rag_id,
                    "SOURCE": "appflowy"
                  });
                  deduplicated_sources.insert(rag_id.clone(), source_meta);
                  added.push(rag_id.clone());
                }
                // 🔧 修复：如果文档已在检索结果中，不需要额外处理，因为检索结果已经包含该文档ID
                // 后端不发送SOURCE_NAME，前端会根据ID自动获取文档名称
              }
              
              if !added.is_empty() {
                trace!(
                  "[RAG] 补充了 {} 个文档引用（总计 {} 个）",
                  added.len(),
                  deduplicated_sources.len()
                );
              }
            }
          }
        }
      }
    } else {
      // 🔧 修复：没有@提及时，只使用检索结果中的文档ID，不补充rag_ids中的其他文档
      // 这样可以避免发送错误的文档ID（如聊天所在的文档ID）
      trace!(
        "[RAG] 没有@提及，只使用检索结果中的文档ID（不补充rag_ids）: {:?}",
        deduplicated_sources.keys().collect::<Vec<_>>()
      );
    }
    
    // 🔧 修复：后端不发送SOURCE_NAME，前端会根据SOURCE_ID自动获取文档名称
    // 这简化了逻辑，让前端统一处理文档名称的获取
    
    // 现在所有数据都已经准备好，可以安全地进入 try_stream!
    let deduplicated_sources_clone = deduplicated_sources.clone();
    // info!("🔧 [AI-SERVICE] About to create stream from response");
    let s = try_stream! {
      // 发送每个文档来源的 metadata（即使 rag_documents 为空，只要 deduplicated_sources 不为空就发送）
      if !deduplicated_sources_clone.is_empty() {
        info!(
          "[RAG] 📤 OpenAI 兼容模式：发送 {} 个文档来源的 metadata",
          deduplicated_sources_clone.len()
        );
        for (idx, (_doc_id, source_meta)) in deduplicated_sources_clone.iter().enumerate() {
          let source_id = source_meta.get("SOURCE_ID")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
          info!(
            "[RAG] 📤 [{}/{}] 发送文档引用 metadata: SOURCE_ID={}",
            idx + 1,
            deduplicated_sources_clone.len(),
            source_id
          );
          yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
            value: source_meta.clone()
          };
        }
      }
      
      let mut inside_think = false;
      let mut tool_call_buffer: Option<OpenAIToolCall> = None;  // 🆕 用于累积流式 tool_call
      let mut stream = resp.bytes_stream();
      let mut chunk_count = 0;
      let mut text_sent = false;
      // info!("🔧 [AI-SERVICE] Starting to process stream response");
      // info!("🔧 [AI-SERVICE] Stream created, waiting for chunks...");
      
      while let Some(chunk) = stream.next().await {
        chunk_count += 1;
        // info!("🔧 [AI-SERVICE] Processing chunk #{}", chunk_count);
        let bytes = chunk.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
        let s = String::from_utf8_lossy(&bytes);
        // info!("🔧 [AI-SERVICE] Received chunk: '{}'", s);
        for line in s.lines() {
          let l = line.trim_start();
          if !l.starts_with("data:") { continue; }
          let data = l.trim_start_matches("data:").trim();
          if data == "[DONE]" { break; }
          // info!("🔧 [AI-SERVICE] Processing data line: '{}'", data);
          if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
            // info!("🔧 [AI-SERVICE] Successfully parsed JSON: {:?}", v);
            if let Some(delta) = v.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
              // info!("🔧 [AI-SERVICE] Found delta: {:?}", delta);
              // 🆕 处理 tool_calls（OpenAI Function Call API）
              if let Some(tool_calls) = delta.get("tool_calls") {
                if let Some(tool_call_array) = tool_calls.as_array() {
                  // 🔧 修复：只有当tool_calls数组不为空时才处理
                  if !tool_call_array.is_empty() {
                    for tool_call_delta in tool_call_array {
                      let index = tool_call_delta.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                      
                      // 初始化或更新 tool_call
                      if index == 0 {
                        if tool_call_buffer.is_none() {
                          tool_call_buffer = Some(OpenAIToolCall {
                            id: tool_call_delta.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                            tool_type: tool_call_delta.get("type").and_then(|s| s.as_str()).unwrap_or("function").to_string(),
                            function: OpenAIFunctionCall {
                              name: String::new(),
                              arguments: String::new(),
                            },
                          });
                        }
                        
                        if let Some(ref mut tc) = tool_call_buffer {
                          if let Some(id) = tool_call_delta.get("id").and_then(|s| s.as_str()) {
                            tc.id = id.to_string();
                          }
                          if let Some(func) = tool_call_delta.get("function") {
                            if let Some(name) = func.get("name").and_then(|s| s.as_str()) {
                              tc.function.name.push_str(name);
                            }
                            if let Some(args) = func.get("arguments").and_then(|s| s.as_str()) {
                              tc.function.arguments.push_str(args);
                            }
                          }
                        }
                      }
                    }
                    
                    // 如果累积的 tool_call 已完整，发送元数据
                    if let Some(ref tc) = tool_call_buffer {
                      if !tc.function.name.is_empty() && !tc.id.is_empty() {
                        // debug!("[OpenAI] Tool call detected: {} (id: {})", tc.function.name, tc.id);
                        yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                          value: json!({
                            "tool_call": {
                              "id": tc.id,
                              "tool_name": tc.function.name,
                              "arguments": tc.function.arguments,
                              "status": "pending"
                            }
                          })
                        };
                      }
                    }
                    continue; // 🔧 只有真正处理了tool_calls才continue
                  }
                }
              }
              
              // 1) 数组结构：显式 type（o1/DeepSeek-R1）
              if let Some(arr) = delta.get("content").and_then(|a| a.as_array()) {
                for item in arr {
                  let ty = item.get("type").and_then(|s| s.as_str()).unwrap_or("");
                  match ty {
                    "reasoning" => {
                      if let Some(t) = item.get("text").and_then(|s| s.as_str()) {
                        yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                          value: json!({"reasoning_delta": t})
                        };
                      }
                    },
                    "output_text" | "text" => {
                      if let Some(t) = item.get("text").and_then(|s| s.as_str()) {
                        text_sent = true;
                        // info!("🔧 [AI-SERVICE] ✅ Yielding array text to Flutter: '{}'", t);
                        yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                          value: t.to_string()
                        };
                      }
                    },
                    _ => {},
                  }
                }
              } else if let Some(token) = delta.get("content").and_then(|c| c.as_str()) {
                // 2) 字符串结构：DeepSeek <think> ... </think>
                // info!("🔧 [AI-SERVICE] ✅ Processing string token: '{}'", token);
                let mut text = token.to_string();
                // 处理开始标签
                if let Some(idx) = text.find("<think>") {
                  inside_think = true;
                  text.replace_range(idx..idx+7, "");
                }
                // 处理结束标签（可能与内容同一块）
                if let Some(end_idx) = text.find("</think>") {
                  let (before, after) = text.split_at(end_idx);
                  let after = after.trim_start_matches("</think>");
                  if !before.is_empty() {
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                      value: json!({"reasoning_delta": before})
                    };
                  }
                  inside_think = false;
                  if !after.is_empty() {
                    text_sent = true;
                    // info!("🔧 [AI-SERVICE] ✅ Yielding after-think text to Flutter: '{}'", after);
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: after.to_string()
                    };
                  }
                }
                if inside_think {
                  if !text.is_empty() {
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                      value: json!({"reasoning_delta": text})
                    };
                  }
                } else {
                  if !text.is_empty() {
                    text_sent = true;
                    // info!("🔧 [AI-SERVICE] ✅ Yielding normal text to Flutter: '{}'", text);
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: text
                    };
                  }
                }
              }

              // 3) 其他兼容字段
              if let Some(r) = delta.get("reasoning_content").and_then(|s| s.as_str()) {
                if !r.is_empty() {
                  yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                    value: json!({"reasoning_delta": r})
                  };
                }
              }
              if let Some(r) = delta.get("reasoning").and_then(|s| s.as_str()) {
                if !r.is_empty() {
                  yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                    value: json!({"reasoning_delta": r})
                  };
                }
              }
            }
          } else {
            warn!("🔧 [AI-SERVICE] Failed to parse JSON: '{}'", data);
          }
        }
      }
      
      // 检查是否收到了任何数据块
      if chunk_count == 0 {
        warn!("🔧 [AI-SERVICE] ⚠️ No chunks received from AI service!");
      } else {
        // info!("🔧 [AI-SERVICE] ✅ Received {} chunks total", chunk_count);
      }
      
      if !text_sent {
        warn!("🔧 [AI-SERVICE] ⚠️ No text was sent to Flutter during this stream!");
      } else {
        // info!("🔧 [AI-SERVICE] ✅ Text was successfully sent to Flutter");
      }
      
    };
    Ok((None, Box::pin(s)))
  }

  /// 原有的 openai_chat_stream 方法（向后兼容，不带系统提示词）
  async fn openai_chat_stream(&self, cfg: &OpenAICompatConfig, model_override: Option<&str>, content: String) -> FlowyResult<(Option<String>, StreamAnswer)> {
    // 调用新方法，不传系统提示词、工具和RAG文档
    let empty_map = std::collections::HashMap::new();
    self.openai_chat_stream_with_system(cfg, model_override, content, None, None, Vec::new(), &empty_map, None).await
  }

  /// 🔄 多轮对话：执行工具并继续对话
  /// 
  /// 当 AI 返回 tool_calls 时，此方法会：
  /// 1. 执行所有工具调用
  /// 2. 构建包含工具结果的新消息历史
  /// 3. 再次调用 OpenAI API
  /// 4. 返回新的流式响应
  pub async fn continue_conversation_with_tool_results(
    &self,
    cfg: &OpenAICompatConfig,
    model: &str,
    initial_messages: Vec<serde_json::Value>,
    tool_calls: Vec<OpenAIToolCall>,
    tool_handler: &crate::agent::ToolCallHandler,
    agent_config: Option<&crate::entities::AgentConfigPB>,
    tools: &[ToolDefinitionPB],
    max_iterations: usize,
  ) -> Result<StreamAnswer, FlowyError> {
    let mut messages = initial_messages;
    let mut current_iteration = 0;
    
    // 添加 assistant 消息（包含 tool_calls）
    let tool_calls_json: Vec<serde_json::Value> = tool_calls.iter().map(|tc| {
      json!({
        "id": tc.id,
        "type": tc.tool_type,
        "function": {
          "name": tc.function.name,
          "arguments": tc.function.arguments
        }
      })
    }).collect();
    
    messages.push(json!({
      "role": "assistant",
      "content": null,
      "tool_calls": tool_calls_json
    }));
    
    info!("[Multi-Turn] Starting tool execution: {} tools", tool_calls.len());
    
    // 执行所有工具并添加结果到消息历史
    for tool_call in &tool_calls {
      info!("[Multi-Turn] Executing tool: {} (id: {})", 
            tool_call.function.name, tool_call.id);
      
      // 解析参数
      let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
        .unwrap_or_else(|_| json!({}));
      
      // 构建工具调用请求
      let request = crate::agent::ToolCallRequest {
        id: tool_call.id.clone(),
        tool_name: tool_call.function.name.clone(),
        arguments,
        source: None, // 自动检测
      };
      
      // 执行工具
      let response = tool_handler.execute_tool_call(&request, agent_config).await;
      
      let result_content = if response.success {
        response.result.unwrap_or_else(|| "Tool executed successfully".to_string())
      } else {
        format!("Tool execution failed: {}", 
                response.error.unwrap_or_else(|| "Unknown error".to_string()))
      };
      
      info!("[Multi-Turn] Tool result ({}ms): {} chars", 
            response.duration_ms, result_content.len());
      
      // 添加工具结果消息
      messages.push(json!({
        "role": "tool",
        "tool_call_id": tool_call.id,
        "name": tool_call.function.name,
        "content": result_content
      }));
    }
    
    // 开始多轮循环
    loop {
      current_iteration += 1;
      if current_iteration > max_iterations {
        warn!("[Multi-Turn] Max iterations ({}) reached", max_iterations);
        break;
      }
      
      info!("[Multi-Turn] Iteration {}/{}: Calling AI with {} messages", 
            current_iteration, max_iterations, messages.len());
      
      // 构建请求 payload
      let url = Self::join_openai_url(&cfg.base_url, "/v1/chat/completions");
      let mut payload = json!({
        "model": model,
        "messages": messages,
        "stream": true
      });
      
      // 添加工具定义
      if !tools.is_empty() {
        let openai_tools = Self::convert_tools_to_openai_format(tools);
        payload.as_object_mut().unwrap().insert("tools".into(), json!(openai_tools));
        payload.as_object_mut().unwrap().insert("tool_choice".into(), json!("auto"));
      }
      
      // 添加可选参数
      if let Some(temp) = cfg.temperature {
        payload.as_object_mut().unwrap().insert("temperature".into(), json!(temp));
      }
      if let Some(max_tok) = cfg.max_tokens {
        payload.as_object_mut().unwrap().insert("max_tokens".into(), json!(max_tok));
      }
      
      // 发送请求
      let client = reqwest::Client::new();
      let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .header("Accept", "text/event-stream")
        .json(&payload)
        .send()
        .await
        .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
      
      if !resp.status().is_success() {
        error!("[Multi-Turn] Non-200 response: {}", resp.status());
        return Err(FlowyError::server_error()
          .with_context(format!("Multi-turn request error: {}", resp.status())));
      }
      
      // 解析响应并检查是否有新的 tool_calls
      // 使用 Arc<Mutex<>> 来在流内外共享状态
      let accumulated_tool_call = Arc::new(tokio::sync::Mutex::new(Option::<OpenAIToolCall>::None));
      let has_content_flag = Arc::new(tokio::sync::Mutex::new(false));
      
      let tc_clone = accumulated_tool_call.clone();
      let content_flag_clone = has_content_flag.clone();
      
      // 创建流式响应
      let s = try_stream! {
        let mut stream = resp.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
          let bytes = chunk.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
          let s = String::from_utf8_lossy(&bytes);
          
          for line in s.lines() {
            let l = line.trim_start();
            if !l.starts_with("data:") { continue; }
            let data = l.trim_start_matches("data:").trim();
            if data == "[DONE]" { break; }
            
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
              if let Some(delta) = v.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
                // 检查 tool_calls
                if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                  for tc_delta in tool_calls_arr {
                    let index = tc_delta.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                    
                    if index == 0 {
                      let mut tc_guard = tc_clone.lock().await;
                      if tc_guard.is_none() {
                        *tc_guard = Some(OpenAIToolCall {
                          id: tc_delta.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                          tool_type: "function".to_string(),
                          function: OpenAIFunctionCall {
                            name: String::new(),
                            arguments: String::new(),
                          },
                        });
                      }
                      
                      if let Some(ref mut tc) = *tc_guard {
                        if let Some(id) = tc_delta.get("id").and_then(|s| s.as_str()) {
                          tc.id = id.to_string();
                        }
                        if let Some(func) = tc_delta.get("function") {
                          if let Some(name) = func.get("name").and_then(|s| s.as_str()) {
                            tc.function.name.push_str(name);
                          }
                          if let Some(args) = func.get("arguments").and_then(|s| s.as_str()) {
                            tc.function.arguments.push_str(args);
                          }
                        }
                      }
                    }
                  }
                }
                
                // 检查普通内容
                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                  if !content.is_empty() {
                    *content_flag_clone.lock().await = true;
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: content.to_string()
                    };
                  }
                }
              }
            }
          }
        }
      };
      
      // 如果累积了新的 tool_call，保存它
      let mut new_tool_calls = Vec::new();
      if let Some(tc) = accumulated_tool_call.lock().await.take() {
        if !tc.function.name.is_empty() {
          new_tool_calls.push(tc);
        }
      }
      
      let has_content = *has_content_flag.lock().await;
      
      // 如果有内容输出且没有新的 tool_calls，说明已完成
      if has_content && new_tool_calls.is_empty() {
        info!("[Multi-Turn] Conversation completed with final answer");
        return Ok(Box::pin(s));
      }
      
      // 如果有新的 tool_calls，添加到消息历史并继续循环
      if !new_tool_calls.is_empty() {
        info!("[Multi-Turn] Detected {} new tool calls, continuing...", new_tool_calls.len());
        
        // 添加 assistant 消息
        let new_tool_calls_json: Vec<serde_json::Value> = new_tool_calls.iter().map(|tc| {
          json!({
            "id": tc.id,
            "type": "function",
            "function": {
              "name": tc.function.name,
              "arguments": tc.function.arguments
            }
          })
        }).collect();
        
        messages.push(json!({
          "role": "assistant",
          "content": null,
          "tool_calls": new_tool_calls_json
        }));
        
        // 执行工具
        for tool_call in &new_tool_calls {
          let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .unwrap_or_else(|_| json!({}));
          
          let request = crate::agent::ToolCallRequest {
            id: tool_call.id.clone(),
            tool_name: tool_call.function.name.clone(),
            arguments,
            source: None,
          };
          
          let response = tool_handler.execute_tool_call(&request, agent_config).await;
          
          let result_content = if response.success {
            response.result.unwrap_or_else(|| "Success".to_string())
          } else {
            format!("Error: {}", response.error.unwrap_or_else(|| "Unknown".to_string()))
          };
          
          messages.push(json!({
            "role": "tool",
            "tool_call_id": tool_call.id,
            "name": tool_call.function.name,
            "content": result_content
          }));
        }
        
        // 继续下一轮循环
        continue;
      }
      
      // 如果既没有内容也没有 tool_calls，返回流
      return Ok(Box::pin(s));
    }
    
    // 达到最大迭代次数
    Err(FlowyError::internal().with_context("Max multi-turn iterations reached"))
  }

  /// 🔄 自动多轮对话（完全集成版）
  /// 
  /// 此方法包含了以下优化：
  /// - ✅ 优先级 1: Middleware 层自动触发
  /// - ✅ 优先级 2: 流合并优化
  /// - ✅ 优先级 3: 用户体验优化（进度显示）
  /// 
  /// 功能：
  /// 1. 自动检测 tool_calls
  /// 2. 执行工具
  /// 3. 继续对话
  /// 4. 实时显示进度
  /// 5. 无缝合并多轮响应
  pub async fn stream_answer_with_auto_multi_turn(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
    system_prompt: Option<String>,
    tools: Option<Vec<ToolDefinitionPB>>,
    tool_handler: Option<Arc<crate::agent::ToolCallHandler>>,
    agent_config: Option<Arc<crate::entities::AgentConfigPB>>,
    max_iterations: usize,
  ) -> Result<StreamAnswer, FlowyError> {
    // 如果没有工具或工具处理器，使用普通模式
    if tools.is_none() || tools.as_ref().unwrap().is_empty() || tool_handler.is_none() {
      return self.stream_answer_with_system_prompt(
        workspace_id, chat_id, question_id, format, ai_model, system_prompt, tools
      ).await;
    }
    
    info!("🔄 [AUTO-MULTI-TURN] Starting with {} tools", 
          tools.as_ref().unwrap().len());
    
    // 获取 OpenAI 配置
    let cfg = self.read_openai_compat_chat_config(workspace_id)
      .ok_or_else(|| FlowyError::internal().with_context("No OpenAI config"))?;
    
    // 规范化模型名称：当选择 "Auto" 或留空时，使用配置中的默认模型
    let model_name = if ai_model.name.is_empty() || ai_model.name == DEFAULT_AI_MODEL_NAME {
      cfg.model.clone()
    } else {
      ai_model.name.clone()
    };
    // 获取消息内容（包含 RAG 文档检索）
    let question = self.get_message_content(question_id)?;
    let (content, rag_documents, document_name_map) = self.get_message_content_with_rag(chat_id, &question).await?;
    let tools_clone = tools.clone();
    let tool_handler = tool_handler.unwrap();
    
    // 根据智能体配置确定有效的最大迭代次数：
    // 1) 工具调用上限 max_tool_calls（>0 生效）
    // 2) 若启用反思机制，则与 max_reflection_iterations 取较小值
    let effective_max_iterations: usize = agent_config
      .as_ref()
      .map(|cfg| {
        let tool_limit = if cfg.capabilities.max_tool_calls > 0 {
          (cfg.capabilities.max_tool_calls as usize).min(100)
        } else { max_iterations };
        if cfg.capabilities.enable_reflection && cfg.capabilities.max_reflection_iterations > 0 {
          let reflection_limit = (cfg.capabilities.max_reflection_iterations as usize).min(100);
          tool_limit.min(reflection_limit)
        } else {
          tool_limit
        }
      })
      .unwrap_or(max_iterations);
    
    // 构建初始消息
    // ⚠️ 使用 build_messages_with_system_prompt 来正确处理 RAG 上下文
    let messages = self.build_messages_with_system_prompt(
      content.clone(),  // 使用清理后的问题（已通过 get_message_content_with_rag 获取）
      system_prompt.clone(),
      &rag_documents,
      &document_name_map
    );
    
    // 🔧 修复问题1：获取 rag_ids，确保所有@的文档都出现在引用列表中
    let uid = self.user_service.user_id()?;
    let mut conn_for_rag_ids = self.user_service.sqlite_connection(uid)?;
    let rag_ids = match select_chat_rag_ids(&mut conn_for_rag_ids, &chat_id.to_string()) {
      Ok(ids) => ids,
      Err(_) => Vec::new(),
    };
    
    // 创建多轮对话流
    let s = try_stream! {
      // 🔧 修复问题1：确保所有@的文档都出现在引用列表中
      // 不仅从 rag_documents 中提取，还要确保 rag_ids 中的所有文档都被包含
      let mut deduplicated_sources: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
      
      // 首先，从检索到的文档片段中提取（这些文档肯定被使用了）
      for doc in &rag_documents {
        if let Some(object_id) = doc.metadata.get("object_id").and_then(|v| v.as_str()) {
          // 尝试从 document_name_map 中获取真实的文档名称
          // 由于 document_name_map 是 "@文档名" -> "文档名" 的映射，我们需要通过反向查找来匹配
          // 但由于 object_id 和文档名的关系不直接，我们暂时使用 document_name_map 中的值
          let document_name = document_name_map.values().next()
            .cloned()
            .unwrap_or_else(|| "document".to_string());
          
          // 构建 metadata，格式与前端期望的 SOURCE_ID/SOURCE/SOURCE_NAME 匹配
          let source_meta = json!({
            "SOURCE_ID": object_id,
            "SOURCE": "appflowy",
            "SOURCE_NAME": document_name
          });
          
          deduplicated_sources.insert(object_id.to_string(), source_meta.clone());
        }
      }
      
      // 🔧 修复问题1：补充所有在 rag_ids 中但不在 rag_documents 中的文档
      // 这确保即使用户@了文档但检索没有返回片段，这些文档也会出现在引用列表中
      let mut object_ids_from_rag_docs: std::collections::HashSet<String> = deduplicated_sources.keys().cloned().collect();
      for rag_id in &rag_ids {
        // 如果这个文档ID还没有在引用列表中，添加它
        if !object_ids_from_rag_docs.contains(rag_id) {
          // 尝试从 document_name_map 中匹配文档名称
          // 由于我们无法直接从 object_id 获取文档名，我们使用 document_name_map 中的第一个名称作为占位
          // 理想的解决方案是在 get_message_content_with_rag 中建立 object_id -> document_name 的映射
          let document_name = document_name_map.values().next()
            .cloned()
            .unwrap_or_else(|| "document".to_string());
          
          let source_meta = json!({
            "SOURCE_ID": rag_id,
            "SOURCE": "appflowy",
            "SOURCE_NAME": document_name
          });
          
          deduplicated_sources.insert(rag_id.clone(), source_meta);
          object_ids_from_rag_docs.insert(rag_id.clone());
        }
      }
      
      // 发送每个文档来源的 metadata
      if !deduplicated_sources.is_empty() {
        for source_meta in deduplicated_sources.values() {
          yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
            value: source_meta.clone()
          };
        }
      }
      
      let mut current_messages = messages.clone();
      let mut iteration = 0;
      // 累积本轮文本内容，用于不支持 Function Call 的回退解析（<tool_call> 标签）
      let answer_buffer = std::sync::Arc::new(tokio::sync::Mutex::new(String::new()));
      
      // 跟踪是否有工具结果（用于判断是否需要强制生成最终回答）
      let mut has_tool_results = false;
      let mut tool_result_count = 0;
      
      // 用于标记是否应该生成最终回答（在检测工具调用后设置）
      let mut should_generate_final_answer = false;
      
      loop {
        iteration += 1;
        // 判断是否达到最大迭代次数
        let is_final_iteration = iteration > effective_max_iterations;
        
        // 🔧 修复：如果达到最大迭代次数，设置为应该生成最终回答
        // 但如果之前已经设置了（因为信息足够），保持不变
        if is_final_iteration {
          should_generate_final_answer = true;
        }
        
        if is_final_iteration {
          warn!("🔄 [AUTO-MULTI-TURN] Max iterations reached, requesting final answer");
        } else {
          info!("🔄 [AUTO-MULTI-TURN] Iteration {}/{}", iteration, effective_max_iterations);
        }
        
        // 🔧 如果是最终迭代，准备生成最终回答（添加synthesis instruction）
        if should_generate_final_answer {
          yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
            value: "\n\n🎯 **正在综合信息生成最终回答...**\n\n".to_string()
          };
          
          // 🔧 关键修复：在最终迭代时，添加明确的user消息指示AI生成最终综合回答
          // 而不是输出规划步骤
          if has_tool_results && tool_result_count > 0 {
            let synthesis_instruction = format!(
              r#"请基于以上所有工具调用的结果，综合生成一个完整的最终回答。

重要指示：
1. **不要输出规划步骤或任务分解过程**
2. **直接基于工具调用结果生成综合回答**
3. **如果工具调用已获取足够信息，请综合这些信息回答用户的原始问题**
4. **如果信息不足，请明确说明哪些部分无法完成及原因**
5. **使用清晰、结构化的方式组织答案**

用户的原始问题是：{}

请现在生成最终回答（不要再次调用工具，不要输出规划步骤）："#,
              content
            );
            
            current_messages.push(json!({
              "role": "user",
              "content": synthesis_instruction
            }));
            
            info!("🔄 [AUTO-MULTI-TURN] Added final synthesis instruction ({} tool results available)", tool_result_count);
          }
        } else {
          // 如果是第2轮或更高（但不是最终轮），显示继续提示
          if iteration > 1 {
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: "\n🤔 **正在综合分析结果...**\n\n".to_string()
            };
          }
        }
        
        // 调用 OpenAI API
        let url = Self::join_openai_url(&cfg.base_url, "/v1/chat/completions");
        let mut payload = json!({
          "model": model_name,
          "messages": current_messages,
          "stream": true
        });
        
        // 添加工具定义（如果是最终迭代，不添加工具，强制生成文本回答）
        if !should_generate_final_answer {
          if let Some(ref tools) = tools_clone {
            if !tools.is_empty() {
              let openai_tools = Self::convert_tools_to_openai_format(tools);
              payload.as_object_mut().unwrap().insert("tools".into(), json!(openai_tools));
              payload.as_object_mut().unwrap().insert("tool_choice".into(), json!("auto"));
            }
          }
        } else {
          // 🔧 关键修复：在最终回答生成时，通过system prompt明确指示不要输出规划步骤
          // 修改payload中的messages，在system message中添加额外指示
          if let Some(system_msg) = payload["messages"].as_array_mut()
            .and_then(|msgs| msgs.iter_mut().find(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"))) {
            // 🔧 修复：JsonValue 不支持 as_str_mut，需要先读取再替换
            if let Some(current_content) = system_msg.get("content").and_then(|c| c.as_str()) {
              let additional_instruction = "\n\n## 重要提示：当前是最终回答生成阶段\n请直接基于工具调用结果综合生成最终回答，不要输出规划步骤、任务分解过程或思考过程。直接给出综合后的答案。";
              let new_content = format!("{}{}", current_content, additional_instruction);
              system_msg["content"] = json!(new_content);
            }
          }
        }
        
        if let Some(temp) = cfg.temperature {
          payload.as_object_mut().unwrap().insert("temperature".into(), json!(temp));
        }
        if let Some(max_tok) = cfg.max_tokens {
          payload.as_object_mut().unwrap().insert("max_tokens".into(), json!(max_tok));
        }
        
        // 发送请求
        let client = reqwest::Client::new();
        let resp = client
          .post(&url)
          .header("Content-Type", "application/json")
          .header("Authorization", format!("Bearer {}", cfg.api_key))
          .header("Accept", "text/event-stream")
          .json(&payload)
          .send()
          .await
          .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
        
        if !resp.status().is_success() {
          let err = FlowyError::server_error()
            .with_context(format!("Request error: {}", resp.status()));
          Err(err)?;
        }
        
        // 解析流式响应并收集 tool_calls
        let accumulated_tool_call = Arc::new(tokio::sync::Mutex::new(Option::<OpenAIToolCall>::None));
        let has_content_flag = Arc::new(tokio::sync::Mutex::new(false));
        let tc_clone = accumulated_tool_call.clone();
        let content_flag_clone = has_content_flag.clone();
        
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
          let bytes = chunk.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
          let s = String::from_utf8_lossy(&bytes);
          
          for line in s.lines() {
            let l = line.trim_start();
            if !l.starts_with("data:") { continue; }
            let data = l.trim_start_matches("data:").trim();
            if data == "[DONE]" { break; }
            
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
              if let Some(delta) = v.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
                // 检查 tool_calls
                if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                  for tc_delta in tool_calls_arr {
                    let index = tc_delta.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                    if index == 0 {
                      let mut tc_guard = tc_clone.lock().await;
                      if tc_guard.is_none() {
                        *tc_guard = Some(OpenAIToolCall {
                          id: tc_delta.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                          tool_type: "function".to_string(),
                          function: OpenAIFunctionCall {
                            name: String::new(),
                            arguments: String::new(),
                          },
                        });
                      }
                      if let Some(ref mut tc) = *tc_guard {
                        if let Some(id) = tc_delta.get("id").and_then(|s| s.as_str()) {
                          tc.id = id.to_string();
                        }
                        if let Some(func) = tc_delta.get("function") {
                          if let Some(name) = func.get("name").and_then(|s| s.as_str()) {
                            tc.function.name.push_str(name);
                          }
                          if let Some(args) = func.get("arguments").and_then(|s| s.as_str()) {
                            tc.function.arguments.push_str(args);
                          }
                        }
                      }
                    }
                  }
                }
                
                // 转发普通内容
                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                  if !content.is_empty() {
                    *content_flag_clone.lock().await = true;
                    // 累积文本用于回退路径的 <tool_call> 解析
                    {
                      let mut buf = answer_buffer.lock().await;
                      buf.push_str(content);
                    }
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: content.to_string()
                    };
                  }
                }
              }
            }
          }
        }
        
        // 检查是否收集到 tool_call
        let tool_call_opt = accumulated_tool_call.lock().await.take();
        let has_content = *has_content_flag.lock().await;
        
        if let Some(ref tc) = tool_call_opt {
          if !tc.function.name.is_empty() {
            info!("🔄 [AUTO-MULTI-TURN] Detected tool: {}", tc.function.name);
            
            // 🔧 修复：如果是最终迭代或应该生成最终回答，不应该再调用工具
            if should_generate_final_answer {
              info!("🔄 [AUTO-MULTI-TURN] Should generate final answer, ignoring tool call and finishing");
              // 移除刚才添加的synthesis instruction（如果添加了），直接结束
              break;
            }
            
            // 添加 assistant 消息
            current_messages.push(json!({
              "role": "assistant",
              "content": null,
              "tool_calls": [{
                "id": tc.id,
                "type": "function",
                "function": {
                  "name": tc.function.name,
                  "arguments": tc.function.arguments
                }
              }]
            }));
            
            // 发送元数据，便于前端展示工具状态（arguments 需为对象而非字符串）
            let meta_args = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
              .unwrap_or_else(|_| json!({}));
            yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
              value: json!({
                "tool_call": {
                  "id": tc.id,
                  "tool_name": tc.function.name,
                  "arguments": meta_args,
                  "status": "pending"
                }
              })
            };

            // 🎨 优化：显示工具执行提示
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: format!("\n\n🔧 **正在调用工具: {}**\n", tc.function.name)
            };
            
            // 执行工具
            let arguments: serde_json::Value = serde_json::from_str(&tc.function.arguments)
              .unwrap_or_else(|_| json!({}));
            
            let request = crate::agent::ToolCallRequest {
              id: tc.id.clone(),
              tool_name: tc.function.name.clone(),
              arguments,
              source: None,
            };
            
            let start_time = std::time::Instant::now();
            let response = tool_handler.execute_tool_call(&request, agent_config.as_ref().map(|c| c.as_ref())).await;
            let duration = start_time.elapsed().as_millis();
            
            let result_content = if response.success {
              response.result.as_deref().unwrap_or("Success").to_string()
            } else {
              format!("Error: {}", response.error.as_deref().unwrap_or("Unknown"))
            };
            
            // 执行完成后，发送成功/失败元数据（供前端记录执行过程）
            let result_status = if response.success { "success" } else { "failed" };
            let result_meta = json!({
              "tool_call": {
                "id": request.id,
                "tool_name": request.tool_name,
                "status": result_status,
                "result": if response.success { response.result.clone() } else { None::<String> },
                "error": if response.success { None::<String> } else { response.error.clone() }
              }
            });
            yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: result_meta };

            // 🎨 优化：显示执行结果
            let status_icon = if response.success { "✅" } else { "❌" };
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: format!("{} **{}** 完成 ({}ms)\n", status_icon, tc.function.name, duration)
            };
            
            // 添加工具结果到消息历史
            current_messages.push(json!({
              "role": "tool",
              "tool_call_id": tc.id,
              "name": tc.function.name,
              "content": result_content
            }));
            
            // 🔧 更新工具结果计数
            has_tool_results = true;
            tool_result_count += 1;
            info!("🔄 [AUTO-MULTI-TURN] Tool result added, total: {}", tool_result_count);
            
            // 继续下一轮
            continue;
          }
        }

        // 没有 Function Call 的情况下，尝试回退：解析文本中的 <tool_call> 标签
        if tool_call_opt.is_none() {
          let accumulated_text = answer_buffer.lock().await.clone();
          let extracted_calls = crate::agent::ToolCallHandler::extract_tool_calls(&accumulated_text);
          if !extracted_calls.is_empty() && !is_final_iteration {
            // 如果不是最终迭代，才执行工具调用
            // 构建 assistant 消息（包含解析得到的 tool_calls），并执行工具，再继续多轮
            let new_tool_calls_json: Vec<serde_json::Value> = extracted_calls.iter().map(|(req, _s, _e)| {
              json!({
                "id": req.id,
                "type": "function",
                "function": {
                  "name": req.tool_name,
                  "arguments": serde_json::to_string(&req.arguments).unwrap_or_else(|_| "{}".to_string())
                }
              })
            }).collect();

            current_messages.push(json!({
              "role": "assistant",
              "content": null,
              "tool_calls": new_tool_calls_json
            }));

            // 执行每个工具
            for (req, _s, _e) in extracted_calls {
              // 元数据通知
              let meta_args = req.arguments.clone();
              yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                value: json!({
                  "tool_call": {
                    "id": req.id,
                    "tool_name": req.tool_name,
                    "arguments": meta_args,
                    "status": "pending"
                  }
                })
              };

              yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                value: format!("\n\n🔧 **正在调用工具: {}**\n", req.tool_name)
              };

              let start_time = std::time::Instant::now();
              let response = tool_handler.execute_tool_call(&req, agent_config.as_ref().map(|c| c.as_ref())).await;
              let duration = start_time.elapsed().as_millis();

              let result_content = if response.success {
                response.result.as_deref().unwrap_or("Success").to_string()
              } else {
                format!("Error: {}", response.error.as_deref().unwrap_or("Unknown"))
              };

              // 执行完成后，发送成功/失败元数据
              let result_status = if response.success { "success" } else { "failed" };
              let result_meta = json!({
                "tool_call": {
                  "id": req.id,
                  "tool_name": req.tool_name,
                  "status": result_status,
                  "result": if response.success { response.result.clone() } else { None::<String> },
                  "error": if response.success { None::<String> } else { response.error.clone() }
                }
              });
              yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: result_meta };

              let status_icon = if response.success { "✅" } else { "❌" };
              yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                value: format!("{} **{}** 完成 ({}ms)\n", status_icon, req.tool_name, duration)
              };

              current_messages.push(json!({
                "role": "tool",
                "tool_call_id": req.id,
                "name": req.tool_name,
                "content": result_content
              }));
              
              // 🔧 更新工具结果计数
              has_tool_results = true;
              tool_result_count += 1;
              info!("🔄 [AUTO-MULTI-TURN] Tool result added (fallback path), total: {}", tool_result_count);
            }

            // 继续下一轮
            continue;
          }

          // 无工具调用回退可用：若本轮产生了内容或达到最终迭代，则视为完成
          // 🔧 关键修复：如果有工具结果但没有新工具调用，应该生成最终回答而不是结束
          if has_content {
            if has_tool_results && tool_result_count > 0 && !is_final_iteration {
              // 有工具结果但AI没有调用新工具，说明信息可能已经足够
              // 设置标志以便下一轮生成最终回答
              info!("🔄 [AUTO-MULTI-TURN] AI generated content without new tools, but has {} tool results. Will generate final answer in next iteration.", tool_result_count);
              
              // 添加明确的synthesis instruction，要求AI综合结果生成最终回答
              let synthesis_instruction = format!(
                r#"请基于以上所有工具调用的结果，综合生成一个完整的最终回答。

重要指示：
1. **不要输出规划步骤或任务分解过程**
2. **直接基于工具调用结果生成综合回答**
3. **如果工具调用已获取足够信息，请综合这些信息回答用户的原始问题**
4. **如果信息不足，请明确说明哪些部分无法完成及原因**
5. **使用清晰、结构化的方式组织答案**

用户的原始问题是：{}

请现在生成最终回答（不要再次调用工具，不要输出规划步骤）："#,
                content
              );
              
              current_messages.push(json!({
                "role": "user",
                "content": synthesis_instruction
              }));
              
              // 标记下次迭代应该生成最终回答
              should_generate_final_answer = true;
              
              // 继续下一轮，生成最终回答
              continue;
            } else if is_final_iteration {
              info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations (final iteration with content)", iteration);
              break;
            } else {
              info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations (content generated, no tools)", iteration);
              break;
            }
          } else if is_final_iteration {
            info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations (final iteration, no content)", iteration);
            break;
          }
        }
      }
    };
    
    Ok(Box::pin(s))
  }

  /// 🌟 多路召回：使用任务分解和并行执行
  /// 
  /// 此方法实现了完整的多路召回流程：
  /// 1. 任务分解：将用户问题分解为多个子任务
  /// 2. 并行执行：同时调用多个工具（RAG、Web Search、MCP等）
  /// 3. 结果综合：将所有结果综合成最终答案
  /// 
  /// 适用场景：
  /// - 需要多个信息源的复杂问题
  /// - 需要对比内外部信息
  /// - 需要综合多个工具的能力
  pub async fn stream_answer_with_multi_retrieval(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
    system_prompt: Option<String>,
    tool_handler: Arc<crate::agent::ToolCallHandler>,
    agent_config: Option<Arc<crate::entities::AgentConfigPB>>,
    available_tools: AvailableTools,
  ) -> Result<StreamAnswer, FlowyError> {
    info!("🌟 [MULTI-RETRIEVAL] Starting multi-retrieval flow");

    // 获取 OpenAI 配置
    let cfg = self.read_openai_compat_chat_config(workspace_id)
      .ok_or_else(|| FlowyError::internal().with_context("No OpenAI config for multi-retrieval"))?;

    // 规范化模型名称
    let model_name = if ai_model.name.is_empty() || ai_model.name == DEFAULT_AI_MODEL_NAME {
      cfg.model.clone()
    } else {
      ai_model.name.clone()
    };

    // 获取用户问题
    let question = self.get_message_content(question_id)?;

    // 1. 任务分解
    info!("🔀 [MULTI-RETRIEVAL] Step 1: Task decomposition");
    let decomposer = TaskDecomposer::new(
      cfg.base_url.clone(),
      cfg.api_key.clone(),
      model_name.clone(),
    );

    let decomposition = decomposer
      .decompose(&question, &available_tools, None)
      .await?;

    info!(
      "🔀 [MULTI-RETRIEVAL] Decomposition result: needs_decomposition={}, {} sub-tasks",
      decomposition.needs_decomposition,
      decomposition.sub_tasks.len()
    );

    // 如果不需要分解，使用原有流程
    if !decomposition.needs_decomposition || decomposition.sub_tasks.is_empty() {
      info!("🔀 [MULTI-RETRIEVAL] Simple question, using standard flow");
      return self.stream_answer_with_auto_multi_turn(
        workspace_id,
        chat_id,
        question_id,
        format,
        ai_model,
        system_prompt,
        None, // 不使用工具
        None, // 不使用工具处理器
        None, // 不使用智能体配置
        5,    // 最大迭代次数
      )
      .await;
    }

    // 2. 并行执行
    info!("🚀 [MULTI-RETRIEVAL] Step 2: Parallel execution");
    let dispatcher = ParallelDispatcher::new(tool_handler.clone())
      .with_rag_context(chat_id.clone(), workspace_id.clone());

    // 保存sub_tasks信息，因为后面要用
    let sub_tasks_info = (decomposition.sub_tasks.len(), decomposition.reasoning.clone());
    
    let dispatch_result = dispatcher
      .dispatch_parallel(decomposition.sub_tasks, agent_config.as_ref().map(|c| c.as_ref()))
      .await?;

    info!(
      "🚀 [MULTI-RETRIEVAL] Parallel execution complete: {}/{} success",
      dispatch_result.success_count,
      dispatch_result.results.len()
    );

    // 3. 结果综合（流式）
    info!("🔄 [MULTI-RETRIEVAL] Step 3: Result synthesis (streaming)");
    let synthesizer = ResultSynthesizer::new(
      cfg.base_url.clone(),
      cfg.api_key.clone(),
      model_name.clone(),
    );

    // 创建流式响应
    let s = try_stream! {
      // 发送任务分解元数据
      yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
        value: json!({
          "multi_retrieval": {
            "phase": "decomposition",
            "needs_decomposition": decomposition.needs_decomposition,
            "sub_tasks_count": sub_tasks_info.0,
            "reasoning": sub_tasks_info.1
          }
        })
      };

      // 发送并行执行元数据
      yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
        value: json!({
          "multi_retrieval": {
            "phase": "execution",
            "total_tasks": dispatch_result.results.len(),
            "success_count": dispatch_result.success_count,
            "failure_count": dispatch_result.failure_count,
            "duration_ms": dispatch_result.total_duration_ms
          }
        })
      };

      // 发送每个子任务的结果（用于前端展示）
      for result in &dispatch_result.results {
        yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
          value: json!({
            "sub_task_result": {
              "task_id": result.task_id,
              "task_description": result.task_description,
              "tool_used": result.tool_used,
              "success": result.success,
              "source": result.source,
              "duration_ms": result.duration_ms,
              "error": result.error
            }
          })
        };
      }

      // 开始综合阶段
      yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
        value: json!({
          "multi_retrieval": {
            "phase": "synthesis",
            "status": "starting"
          }
        })
      };

      yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
        value: "\n\n🔄 **正在综合多个信息源的结果...**\n\n".to_string()
      };

      // 流式输出综合结果
      use futures_util::pin_mut;
      let synthesis_stream = synthesizer
        .synthesize_stream(question.clone(), dispatch_result.clone(), system_prompt.clone())
        .await?;
      
      pin_mut!(synthesis_stream);

      while let Some(chunk_result) = synthesis_stream.next().await {
        match chunk_result {
          Ok(chunk) => {
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: chunk
            };
          }
          Err(e) => {
            warn!("❌ [MULTI-RETRIEVAL] Synthesis stream error: {}", e);
            Err(e)?;
          }
        }
      }

      // 综合完成
      yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
        value: json!({
          "multi_retrieval": {
            "phase": "completed"
          }
        })
      };

      info!("✅ [MULTI-RETRIEVAL] Multi-retrieval flow completed");
    };

    Ok(Box::pin(s))
  }
}

#[async_trait]
impl ChatCloudService for ChatServiceMiddleware {
  async fn create_chat(
    &self,
    uid: &i64,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    rag_ids: Vec<Uuid>,
    name: &str,
    metadata: serde_json::Value,
  ) -> Result<(), FlowyError> {
    self
      .cloud_service
      .create_chat(uid, workspace_id, chat_id, rag_ids, name, metadata)
      .await
  }

  async fn create_question(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    message: &str,
    message_type: ChatMessageType,
    prompt_id: Option<String>,
  ) -> Result<ChatMessage, FlowyError> {
    self
      .cloud_service
      .create_question(workspace_id, chat_id, message, message_type, prompt_id)
      .await
  }

  async fn create_answer(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    message: &str,
    question_id: i64,
    metadata: Option<serde_json::Value>,
  ) -> Result<ChatMessage, FlowyError> {
    let mut chat_message = self
      .cloud_service
      .create_answer(workspace_id, chat_id, message, question_id, metadata)
      .await?;
    
    // 🔧 关键修复：确保 reply_message_id 被正确设置
    chat_message.reply_message_id = Some(question_id);
    
    tracing::info!(
      "🔍 [CHAT-SERVICE-MW-CREATE-ANSWER] message_id: {}, question_id: {}, reply_message_id: {:?}",
      chat_message.message_id, question_id, chat_message.reply_message_id
    );
    
    Ok(chat_message)
  }

  async fn stream_answer(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
  ) -> Result<StreamAnswer, FlowyError> {
    info!("stream_answer use model: {:?}", ai_model);
    // Honor global provider: if user prefers local but local not ready -> translated error
    if ai_model.is_local {
      if self.local_ai.is_ready().await {
        let content = self.get_message_content(question_id)?;
        self
          .local_ai
          .stream_question(chat_id, &content, format, &ai_model.name)
          .await
      } else {
        // Fallback to server provider with workspace default model
        match self
          .cloud_service
          .get_workspace_default_model(workspace_id)
          .await
        {
          Ok(name) => {
            let server_model = AIModel::server(name, String::new());
            self
              .cloud_service
              .stream_answer(workspace_id, chat_id, question_id, format.clone(), server_model)
              .await
          },
          Err(_) => Err(
            FlowyError::local_ai_not_ready()
              .with_context("本地 AI 未就绪 / Local AI not ready"),
          ),
        }
      }
    } else {
      // 如果配置了 OpenAI 兼容服务器，则优先直接调用（SSE）
      if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
        let content = self.get_message_content(question_id)?;
        // 🔧 重要修复：添加 RAG 文档检索支持
        let (content_with_rag, _rag_documents, _document_name_map) = self.get_message_content_with_rag(chat_id, &content).await?;
        let (_init_reasoning, stream) = self
          .openai_chat_stream(&cfg, Some(&ai_model.name), content_with_rag)
          .await?;
        return Ok(stream);
      }

      // 🚫 AppFlowy Cloud AI 已禁用：只使用全局 AI 配置
      Err(FlowyError::internal()
        .with_context("未配置 OpenAI 兼容服务器。请在全局 AI 设置中配置 AI 供应商。"))
    }
  }

  async fn get_answer(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
  ) -> Result<ChatMessage, FlowyError> {
    // 🚫 AppFlowy Cloud AI 已禁用：只使用本地 AI 或 OpenAI 兼容服务器
    // 注意：此方法不支持 OpenAI 兼容服务器（需要流式 API）
    if self.local_ai.is_ready().await {
      let content = self.get_message_content(question_id)?;
      let answer = self.local_ai.ask_question(chat_id, &content).await?;

      let message = self
        .cloud_service
        .create_answer(workspace_id, chat_id, &answer, question_id, None)
        .await?;
      Ok(message)
    } else {
      Err(FlowyError::internal()
        .with_context("本地 AI 未就绪。此方法不支持 OpenAI 兼容服务器，请使用流式 AI 接口。"))
    }
  }

  async fn get_chat_messages(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    offset: MessageCursor,
    limit: u64,
  ) -> Result<RepeatedChatMessage, FlowyError> {
    self
      .cloud_service
      .get_chat_messages(workspace_id, chat_id, offset, limit)
      .await
  }

  async fn get_question_from_answer_id(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    answer_message_id: i64,
  ) -> Result<ChatMessage, FlowyError> {
    self
      .cloud_service
      .get_question_from_answer_id(workspace_id, chat_id, answer_message_id)
      .await
  }

  async fn get_related_message(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    message_id: i64,
    ai_model: AIModel,
  ) -> Result<RepeatedRelatedQuestion, FlowyError> {
    if ai_model.is_local {
      if self.local_ai.is_ready().await {
        let questions = self
          .local_ai
          .get_related_question(&ai_model.name, chat_id, message_id)
          .await?;
        trace!("LocalAI related questions: {:?}", questions);
        let items = questions
          .into_iter()
          .map(|content| RelatedQuestion {
            content,
            metadata: None,
          })
          .collect::<Vec<_>>();

        Ok(RepeatedRelatedQuestion { message_id, items })
      } else {
        // Fallback to server provider with workspace default model
        match self
          .cloud_service
          .get_workspace_default_model(workspace_id)
          .await
        {
          Ok(name) => {
            let server_model = AIModel::server(name, String::new());
            self
              .cloud_service
              .get_related_message(workspace_id, chat_id, message_id, server_model)
              .await
          },
          Err(_) => Ok(RepeatedRelatedQuestion { message_id, items: vec![] }),
        }
      }
    } else {
      match self
        .cloud_service
        .get_related_message(workspace_id, chat_id, message_id, ai_model)
        .await
      {
        Ok(ok) => Ok(ok),
        Err(err) => {
          if self.local_ai.is_ready().await {
            let questions = self
              .local_ai
              .get_related_question(
                &self.local_ai.get_local_ai_setting().chat_model_name,
                chat_id,
                message_id,
              )
              .await?;
            let items = questions
              .into_iter()
              .map(|content| RelatedQuestion { content, metadata: None })
              .collect::<Vec<_>>();
            Ok(RepeatedRelatedQuestion { message_id, items })
          } else {
            Err(err)
          }
        },
      }
    }
  }

  async fn stream_complete(
    &self,
    workspace_id: &Uuid,
    params: CompleteTextParams,
    ai_model: AIModel,
  ) -> Result<StreamComplete, FlowyError> {
    info!("stream_complete use custom model: {:?}", ai_model);
    if ai_model.is_local {
      if self.local_ai.is_ready().await {
        self.local_ai.complete_text(&ai_model.name, params).await
      } else {
        // Fallback to server provider with workspace default model
        match self
          .cloud_service
          .get_workspace_default_model(workspace_id)
          .await
        {
          Ok(name) => {
            let server_model = AIModel::server(name, String::new());
            self
              .cloud_service
              .stream_complete(workspace_id, params.clone(), server_model)
              .await
          },
          Err(_) => Err(
            FlowyError::local_ai_not_ready()
              .with_context("本地 AI 未就绪 / Local AI not ready"),
          ),
        }
      }
    } else {
      // 若配置了 OpenAI 兼容服务器，则直接调用（SSE -> 写作流）
      if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
        let (_init_reasoning, s) = self
          .openai_chat_stream(&cfg, Some(&ai_model.name), params.text.clone())
          .await?;
        let mapped = s.map(|item| match item {
          Ok(flowy_ai_pub::cloud::QuestionStreamValue::Answer { value }) => Ok(flowy_ai_pub::cloud::CompletionStreamValue::Answer { value }),
          Ok(flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value }) => Ok(flowy_ai_pub::cloud::CompletionStreamValue::Comment { value: value.get("reasoning_delta").and_then(|s| s.as_str()).unwrap_or("").to_string() }),
          Ok(_) => Ok(flowy_ai_pub::cloud::CompletionStreamValue::Answer { value: String::new() }),
          Err(e) => Err(e),
        });
        return Ok(Box::pin(mapped));
      }

      // 🚫 AppFlowy Cloud AI 已禁用：只使用全局 AI 配置
      Err(FlowyError::internal()
        .with_context("未配置 OpenAI 兼容服务器。请在全局 AI 设置中配置 AI 供应商。"))
    }
  }

  async fn embed_file(
    &self,
    workspace_id: &Uuid,
    file_path: &Path,
    chat_id: &Uuid,
    metadata: Option<HashMap<String, Value>>,
  ) -> Result<(), FlowyError> {
    let prefer_local = self.user_service.is_local_model().await.unwrap_or(false);
    if prefer_local && self.local_ai.is_ready().await {
      self
        .local_ai
        .embed_file(chat_id, file_path.to_path_buf(), metadata)
        .await?;
      Ok(())
    } else {
      match self
        .cloud_service
        .embed_file(workspace_id, file_path, chat_id, metadata)
        .await
      {
        Ok(ok) => Ok(ok),
        Err(err) => {
          if self.local_ai.is_ready().await {
            self
              .local_ai
              .embed_file(chat_id, file_path.to_path_buf(), None)
              .await?;
            Ok(())
          } else {
            Err(err)
          }
        },
      }
    }
  }

  async fn get_chat_settings(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
  ) -> Result<ChatSettings, FlowyError> {
    self
      .cloud_service
      .get_chat_settings(workspace_id, chat_id)
      .await
  }

  async fn update_chat_settings(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    params: UpdateChatParams,
  ) -> Result<(), FlowyError> {
    self
      .cloud_service
      .update_chat_settings(workspace_id, chat_id, params)
      .await
  }

  async fn get_available_models(&self, workspace_id: &Uuid) -> Result<ModelList, FlowyError> {
    self.cloud_service.get_available_models(workspace_id).await
  }

  async fn get_workspace_default_model(&self, workspace_id: &Uuid) -> Result<String, FlowyError> {
    self
      .cloud_service
      .get_workspace_default_model(workspace_id)
      .await
  }

  async fn set_workspace_default_model(
    &self,
    workspace_id: &Uuid,
    model: &str,
  ) -> Result<(), FlowyError> {
    self
      .cloud_service
      .set_workspace_default_model(workspace_id, model)
      .await
  }
}

