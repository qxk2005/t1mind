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
  async fn get_message_content_with_rag(
    &self,
    chat_id: &Uuid,
    question: &str,
  ) -> FlowyResult<String> {
    // 获取 rag_ids
    let uid = self.user_service.user_id()?;
    let mut conn = self.user_service.sqlite_connection(uid)?;
    let rag_ids = match select_chat_rag_ids(&mut conn, &chat_id.to_string()) {
      Ok(ids) => ids,
      Err(_) => Vec::new(),
    };

    if rag_ids.is_empty() {
      trace!("[RAG] 📚 OpenAI 兼容模式：没有选择文档，直接使用用户问题");
      return Ok(question.to_string());
    }

    info!(
      "[RAG] 📚 OpenAI 兼容模式：检索文档 - rag_ids={:?}",
      rag_ids
    );

    // 尝试直接使用嵌入调度器进行搜索（不依赖 local_ai.is_ready()）
    // 这样即使 Ollama 聊天客户端未初始化，只要配置了嵌入服务就能工作
    match self.try_search_documents_via_embeddings(chat_id, question, &rag_ids).await {
      Ok(documents) if !documents.is_empty() => {
        info!(
          "[RAG] 📖 OpenAI 兼容模式：找到 {} 个相关文档片段",
          documents.len()
        );
        
        // 构建包含文档上下文的消息
        let context = documents
          .iter()
          .map(|doc| doc.page_content.clone())
          .collect::<Vec<_>>()
          .join("\n\n");
        
        let enhanced_message = format!(
          r#"Use the following context to answer the question. Only use information from the context provided.

##Context##
{}

##Question##
{}"#,
          context, question
        );
        
        trace!("[RAG] ✅ OpenAI 兼容模式：已添加文档上下文到消息");
        return Ok(enhanced_message);
      }
      Ok(_) => {
        warn!(
          "[RAG] ⚠️ OpenAI 兼容模式：未找到相关文档，使用原始问题"
        );
      }
      Err(err) => {
        warn!(
          "[RAG] ⚠️ OpenAI 兼容模式：文档检索失败: {}，使用原始问题",
          err
        );
      }
    }

    Ok(question.to_string())
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
  fn build_messages_with_system_prompt(
    &self,
    content: String,
    system_prompt: Option<String>,
  ) -> Vec<serde_json::Value> {
    let mut messages = Vec::new();
    
    // 如果有系统提示词，作为独立的系统消息添加
    if let Some(prompt) = system_prompt {
      messages.push(json!({
        "role": "system",
        "content": prompt
      }));
    }
    
    // 用户消息
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
    let content = self.get_message_content_with_rag(chat_id, &question).await?;
    
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
        let final_content = if let Some(ref prompt) = system_prompt {
          format!("{}\n\n{}", prompt, content)
        } else {
          content
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
                .openai_chat_stream_with_system(&cfg, Some(&server_model.name), content, system_prompt, tools.as_deref())
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
        let content_with_rag = self.get_message_content_with_rag(chat_id, &content).await?;
        let (_init_reasoning, stream) = self
          .openai_chat_stream_with_system(&cfg, Some(&ai_model.name), content_with_rag, system_prompt, tools.as_deref())
          .await?;
        return Ok(stream);
      }

      // 默认：走现有 cloud_service（不支持系统提示词）
      warn!("System prompt not supported for AppFlowy Cloud, falling back to standard stream_answer");
      self.cloud_service
        .stream_answer(workspace_id, chat_id, question_id, format, ai_model)
        .await
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
  ) -> Result<(Option<String>, StreamAnswer), FlowyError> {
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
    
    // 构建包含系统提示词的消息数组（使用标准 OpenAI 格式）
    let messages = self.build_messages_with_system_prompt(content, system_prompt);
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
    
    let s = try_stream! {
      let mut inside_think = false;
      let mut tool_call_buffer: Option<OpenAIToolCall> = None;  // 🆕 用于累积流式 tool_call
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
              // 🆕 处理 tool_calls（OpenAI Function Call API）
              if let Some(tool_calls) = delta.get("tool_calls") {
                if let Some(tool_call_array) = tool_calls.as_array() {
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
                }
                
                // 如果累积的 tool_call 已完整，发送元数据
                if let Some(ref tc) = tool_call_buffer {
                  if !tc.function.name.is_empty() && !tc.id.is_empty() {
                    debug!("[OpenAI] Tool call detected: {} (id: {})", tc.function.name, tc.id);
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
                continue;
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
                        yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                          value: t.to_string()
                        };
                      }
                    },
                    _ => {},
                  }
                }
                continue;
              }

              // 2) 字符串结构：DeepSeek <think> ... </think>
              if let Some(token) = delta.get("content").and_then(|c| c.as_str()) {
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
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: after.to_string()
                    };
                  }
                  continue;
                }
                if inside_think {
                  if !text.is_empty() {
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata {
                      value: json!({"reasoning_delta": text})
                    };
                  }
                } else {
                  if !text.is_empty() {
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: text
                    };
                  }
                }
                continue;
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
          }
        }
      }
    };
    Ok((None, Box::pin(s)))
  }

  /// 原有的 openai_chat_stream 方法（向后兼容，不带系统提示词）
  async fn openai_chat_stream(&self, cfg: &OpenAICompatConfig, model_override: Option<&str>, content: String) -> FlowyResult<(Option<String>, StreamAnswer)> {
    // 调用新方法，不传系统提示词和工具
    self.openai_chat_stream_with_system(cfg, model_override, content, None, None).await
  }

  /// 废弃的实现（保留用于参考）
  #[allow(dead_code)]
  async fn openai_chat_stream_old(&self, cfg: &OpenAICompatConfig, model_override: Option<&str>, content: String) -> FlowyResult<(Option<String>, StreamAnswer)> {
    let client = reqwest::Client::new();
    let base = cfg.base_url.trim_end_matches('/');
    // 仅对 chat.completions 走 SSE；responses 不同供应商差异大，暂不做 SSE
    let url = if base.ends_with("/v1") { format!("{}/chat/completions", base) } else { format!("{}/v1/chat/completions", base) };

    let model_name = match model_override {
      Some(name) if !name.is_empty() && name != DEFAULT_AI_MODEL_NAME => name.to_string(),
      _ => cfg.model.clone(),
    };

    let messages = vec![json!({"role": "user", "content": content})];
    let mut payload = Self::openai_chat_payload(&model_name, messages);
    if let Some(t) = cfg.temperature { payload.as_object_mut().unwrap().insert("temperature".into(), json!(t)); }
    if let Some(m) = cfg.max_tokens { payload.as_object_mut().unwrap().insert("max_tokens".into(), json!(m)); }
    let resp = client
      .post(&url)
      .bearer_auth(&cfg.api_key)
      .header("Content-Type", "application/json")
      .header("Accept", "text/event-stream")
      .json(&payload)
      .send()
      .await
      .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
    if !resp.status().is_success() {
      return Err(FlowyError::server_error().with_context(format!("OpenAI compat error: {}", resp.status())));
    }

    let s = try_stream! {
      let mut inside_think = false;
      let mut stream = resp.bytes_stream();
      info!("🔧 [AI-SERVICE] Starting to process stream response");
      let mut chunk_count = 0;
      while let Some(chunk) = stream.next().await {
        chunk_count += 1;
        info!("🔧 [AI-SERVICE] Processing chunk #{}", chunk_count);
        let bytes = chunk.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
        let s = String::from_utf8_lossy(&bytes);
        info!("🔧 [AI-SERVICE] Received chunk: '{}'", s);
        for line in s.lines() {
          let l = line.trim_start();
          if !l.starts_with("data:") { 
            info!("🔧 [AI-SERVICE] Skipping non-data line: '{}'", l);
            continue; 
          }
          let data = l.trim_start_matches("data:").trim();
          info!("🔧 [AI-SERVICE] Processing data: '{}'", data);
          if data == "[DONE]" { 
            info!("🔧 [AI-SERVICE] Stream completed with [DONE]");
            break; 
          }
          if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(delta) = v.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
              // 1) 数组结构：显式 type
              if let Some(arr) = delta.get("content").and_then(|a| a.as_array()) {
                for item in arr {
                  let ty = item.get("type").and_then(|s| s.as_str()).unwrap_or("");
                  match ty {
                    "reasoning" => {
                      if let Some(t) = item.get("text").and_then(|s| s.as_str()) { yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: json!({"reasoning_delta": t}) }; }
                    },
                    "output_text" | "text" => {
                      if let Some(t) = item.get("text").and_then(|s| s.as_str()) { 
                        info!("🔧 [AI-SERVICE] Received text from AI: '{}'", t);
                        yield flowy_ai_pub::cloud::QuestionStreamValue::Answer { value: t.to_string() }; 
                      }
                    },
                    _ => {},
                  }
                }
                continue;
              }

              // 2) 字符串结构：DeepSeek <think> ... </think>
              if let Some(token) = delta.get("content").and_then(|c| c.as_str()) {
                let mut text = token.to_string();
                // 处理开始标签
                if let Some(idx) = text.find("<think>") { inside_think = true; text.replace_range(idx..idx+7, ""); }
                // 处理结束标签（可能与内容同一块）
                if let Some(end_idx) = text.find("</think>") {
                  let (before, after) = text.split_at(end_idx);
                  let after = after.trim_start_matches("</think>");
                  if !before.is_empty() { yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: json!({"reasoning_delta": before}) }; }
                  inside_think = false;
                  if !after.is_empty() { yield flowy_ai_pub::cloud::QuestionStreamValue::Answer { value: after.to_string() }; }
                  continue;
                }
                if inside_think {
                  if !text.is_empty() { yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: json!({"reasoning_delta": text}) }; }
                } else {
                  if !text.is_empty() { yield flowy_ai_pub::cloud::QuestionStreamValue::Answer { value: text }; }
                }
                continue;
              }

              // 3) 其他兼容字段
              if let Some(r) = delta.get("reasoning_content").and_then(|s| s.as_str()) { if !r.is_empty() { yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: json!({"reasoning_delta": r}) }; } }
              if let Some(r) = delta.get("reasoning").and_then(|s| s.as_str()) { if !r.is_empty() { yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: json!({"reasoning_delta": r}) }; } }
            }
          }
        }
      }
    };
    Ok((None, Box::pin(s)))
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
    let content = self.get_message_content_with_rag(chat_id, &question).await?;
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
    let mut messages = Vec::new();
    if let Some(ref prompt) = system_prompt {
      messages.push(json!({
        "role": "system",
        "content": prompt
      }));
    }
    messages.push(json!({
      "role": "user",
      "content": content
    }));
    
    // 创建多轮对话流
    let s = try_stream! {
      let mut current_messages = messages.clone();
      let mut iteration = 0;
      // 累积本轮文本内容，用于不支持 Function Call 的回退解析（<tool_call> 标签）
      let answer_buffer = std::sync::Arc::new(tokio::sync::Mutex::new(String::new()));
      
      loop {
        iteration += 1;
        if iteration > effective_max_iterations {
          warn!("🔄 [AUTO-MULTI-TURN] Max iterations reached");
          yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
            value: format!("\n\n⚠️ 已达到最大对话轮次 ({})\n", effective_max_iterations)
          };
          break;
        }
        
        info!("🔄 [AUTO-MULTI-TURN] Iteration {}/{}", iteration, effective_max_iterations);
        
        // 如果是第2轮或更高，显示继续提示
        if iteration > 1 {
          yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
            value: "\n🤔 **正在综合分析结果...**\n\n".to_string()
          };
        }
        
        // 调用 OpenAI API
        let url = Self::join_openai_url(&cfg.base_url, "/v1/chat/completions");
        let mut payload = json!({
          "model": model_name,
          "messages": current_messages,
          "stream": true
        });
        
        // 添加工具定义
        if let Some(ref tools) = tools_clone {
          if !tools.is_empty() {
            let openai_tools = Self::convert_tools_to_openai_format(tools);
            payload.as_object_mut().unwrap().insert("tools".into(), json!(openai_tools));
            payload.as_object_mut().unwrap().insert("tool_choice".into(), json!("auto"));
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
            
            // 继续下一轮
            continue;
          }
        }

        // 没有 Function Call 的情况下，尝试回退：解析文本中的 <tool_call> 标签
        if tool_call_opt.is_none() {
          let accumulated_text = answer_buffer.lock().await.clone();
          let extracted_calls = crate::agent::ToolCallHandler::extract_tool_calls(&accumulated_text);
          if !extracted_calls.is_empty() {
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
            }

            // 继续下一轮
            continue;
          }

          // 无工具调用回退可用：若本轮产生了内容，则视为完成
          if has_content {
            info!("🔄 [AUTO-MULTI-TURN] Completed after {} iterations", iteration);
            break;
          }
        }
      }
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
    self
      .cloud_service
      .create_answer(workspace_id, chat_id, message, question_id, metadata)
      .await
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
        let content_with_rag = self.get_message_content_with_rag(chat_id, &content).await?;
        let (_init_reasoning, stream) = self
          .openai_chat_stream(&cfg, Some(&ai_model.name), content_with_rag)
          .await?;
        return Ok(stream);
      }

      // 默认：走现有 cloud_service（AppFlowy Cloud 或本地服务封装）
      match self
        .cloud_service
        .stream_answer(workspace_id, chat_id, question_id, format.clone(), ai_model)
        .await
      {
        Ok(ok) => Ok(ok),
        Err(err) => {
          if self.local_ai.is_ready().await {
            let content = self.get_message_content(question_id)?;
            return self
              .local_ai
              .stream_question(chat_id, &content, format.clone(), &self.local_ai.get_local_ai_setting().chat_model_name)
              .await
              .map_err(|e| e.with_context("云端 AI 不可用，已回退到本地 / Remote AI unavailable, fallback to local"));
          }
          Err(err)
        },
      }
    }
  }

  async fn get_answer(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
  ) -> Result<ChatMessage, FlowyError> {
    let prefer_local = self.user_service.is_local_model().await.unwrap_or(false);
    if prefer_local && self.local_ai.is_ready().await {
      let content = self.get_message_content(question_id)?;
      let answer = self.local_ai.ask_question(chat_id, &content).await?;

      let message = self
        .cloud_service
        .create_answer(workspace_id, chat_id, &answer, question_id, None)
        .await?;
      Ok(message)
    } else {
      match self
        .cloud_service
        .get_answer(workspace_id, chat_id, question_id)
        .await
      {
        Ok(ok) => Ok(ok),
        Err(err) => {
          if self.local_ai.is_ready().await {
            let content = self.get_message_content(question_id)?;
            let answer = self.local_ai.ask_question(chat_id, &content).await?;
            let message = self
              .cloud_service
              .create_answer(workspace_id, chat_id, &answer, question_id, None)
              .await?;
            Ok(message)
          } else {
            Err(err)
          }
        },
      }
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

      match self
        .cloud_service
        .stream_complete(workspace_id, params.clone(), ai_model)
        .await
      {
        Ok(ok) => Ok(ok),
        Err(err) => {
          if self.local_ai.is_ready().await {
            return self
              .local_ai
              .complete_text(&self.local_ai.get_local_ai_setting().chat_model_name, params)
              .await
              .map_err(|e| e.with_context("云端 AI 不可用，已回退到本地 / Remote AI unavailable, fallback to local"));
          }
          Err(err)
        },
      }
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
