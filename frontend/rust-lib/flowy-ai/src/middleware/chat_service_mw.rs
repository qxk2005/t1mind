use crate::local_ai::controller::LocalAIController;
use flowy_ai_pub::persistence::select_message_content;
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
use tracing::{info, trace, warn};
use uuid::Uuid;
use flowy_sqlite::kv::KVStorePreferences;
use futures_util::StreamExt;
use async_stream::try_stream;
use serde_json::json;

#[derive(Clone, Debug)]
struct OpenAICompatConfig {
  base_url: String,
  api_key: String,
  model: String,
  temperature: Option<f64>,
  max_tokens: Option<u32>,
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

  /// 附加系统提示词到消息内容
  fn apply_system_prompt(&self, content: String, system_prompt: Option<String>) -> String {
    if let Some(prompt) = system_prompt {
      format!("System Instructions:\n{}\n\n---\n\nUser Message:\n{}", prompt, content)
    } else {
      content
    }
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
  ) -> Result<StreamAnswer, FlowyError> {
    // 获取原始消息内容
    let content = self.get_message_content(question_id)?;
    // 附加系统提示词
    let final_content = self.apply_system_prompt(content, system_prompt);
    
    info!("stream_answer_with_system_prompt use model: {:?}", ai_model);
    
    // 根据模型类型调用不同的服务
    if ai_model.is_local {
      if self.local_ai.is_ready().await {
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
            // 注意：这里调用原始的 stream_answer 会再次从数据库读取，所以我们需要直接调用 openai_chat_stream
            if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
              let (_init_reasoning, stream) = self
                .openai_chat_stream(&cfg, Some(&server_model.name), final_content)
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
      // 如果配置了 OpenAI 兼容服务器，则优先直接调用
      if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
        let (_init_reasoning, stream) = self
          .openai_chat_stream(&cfg, Some(&ai_model.name), final_content)
          .await?;
        return Ok(stream);
      }

      // 默认：走现有 cloud_service（需要另想办法，因为它会从数据库读取）
      // 这里我们暂时不支持 AppFlowy Cloud 的系统提示词
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

  fn openai_chat_payload(model: &str, content: String) -> serde_json::Value {
    json!({
      "model": model,
      "stream": true,
      "stream_options": {"include_reasoning": true},
      "messages": [
        {"role": "user", "content": content}
      ]
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

  async fn openai_chat_stream(&self, cfg: &OpenAICompatConfig, model_override: Option<&str>, content: String) -> FlowyResult<(Option<String>, StreamAnswer)> {
    let client = reqwest::Client::new();
    let base = cfg.base_url.trim_end_matches('/');
    // 仅对 chat.completions 走 SSE；responses 不同供应商差异大，暂不做 SSE
    let url = if base.ends_with("/v1") { format!("{}/chat/completions", base) } else { format!("{}/v1/chat/completions", base) };

    let model_name = match model_override {
      Some(name) if !name.is_empty() && name != DEFAULT_AI_MODEL_NAME => name.to_string(),
      _ => cfg.model.clone(),
    };

    let mut payload = Self::openai_chat_payload(&model_name, content.clone());
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
              // 1) 数组结构：显式 type
              if let Some(arr) = delta.get("content").and_then(|a| a.as_array()) {
                for item in arr {
                  let ty = item.get("type").and_then(|s| s.as_str()).unwrap_or("");
                  match ty {
                    "reasoning" => {
                      if let Some(t) = item.get("text").and_then(|s| s.as_str()) { yield flowy_ai_pub::cloud::QuestionStreamValue::Metadata { value: json!({"reasoning_delta": t}) }; }
                    },
                    "output_text" | "text" => {
                      if let Some(t) = item.get("text").and_then(|s| s.as_str()) { yield flowy_ai_pub::cloud::QuestionStreamValue::Answer { value: t.to_string() }; }
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
        let (_init_reasoning, stream) = self
          .openai_chat_stream(&cfg, Some(&ai_model.name), content)
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
