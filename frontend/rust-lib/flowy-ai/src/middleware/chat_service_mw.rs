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
use tracing::{info, trace};
use uuid::Uuid;
use flowy_sqlite::kv::KVStorePreferences;
use futures_util::{StreamExt, stream};
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
      "stream": false,
      "messages": [
        {"role": "user", "content": content}
      ]
    })
  }

  async fn openai_chat_once(&self, cfg: &OpenAICompatConfig, model_override: Option<&str>, content: String) -> FlowyResult<String> {
    let client = reqwest::Client::new();
    let base = cfg.base_url.trim_end_matches('/');
    let candidates = if base.ends_with("/v1") {
      vec![format!("{}/chat/completions", base), format!("{}/responses", base)]
    } else {
      vec![format!("{}/v1/chat/completions", base), format!("{}/v1/responses", base)]
    };

    let model_name = match model_override {
      Some(name) if !name.is_empty() && name != DEFAULT_AI_MODEL_NAME => name.to_string(),
      _ => cfg.model.clone(),
    };

    let mut last_status = None;
    for url in candidates {
      let mut payload = Self::openai_chat_payload(&model_name, content.clone());
      if let Some(t) = cfg.temperature { payload.as_object_mut().unwrap().insert("temperature".into(), json!(t)); }
      if let Some(m) = cfg.max_tokens { payload.as_object_mut().unwrap().insert("max_tokens".into(), json!(m)); }

      match client
        .post(&url)
        .bearer_auth(&cfg.api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
      {
        Ok(resp) => {
          let status = resp.status();
          if status.is_success() {
            let v: serde_json::Value = resp.json().await.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
            // Prefer chat.completions style
            if let Some(text) = v
              .get("choices")
              .and_then(|c| c.get(0))
              .and_then(|c| c.get("message").or_else(|| c.get("delta")))
              .and_then(|m| m.get("content"))
              .and_then(|s| s.as_str())
            {
              return Ok(text.to_string());
            }
            // Fallback for responses API
            if let Some(text) = v
              .get("output")
              .and_then(|o| o.get("choices"))
              .and_then(|c| c.get(0))
              .and_then(|c| c.get("message"))
              .and_then(|m| m.get("content"))
              .and_then(|arr| arr.get(0))
              .and_then(|obj| obj.get("text"))
              .and_then(|t| t.get("value"))
              .and_then(|s| s.as_str())
            {
              return Ok(text.to_string());
            }
            // Unknown format
            return Ok(String::new());
          } else {
            last_status = Some(status);
            continue;
          }
        },
        Err(e) => {
          // try next
          tracing::warn!("OpenAI compat request error: {}", e);
          continue;
        },
      }
    }

    Err(FlowyError::server_error().with_context(format!(
      "OpenAI compat error: {}",
      last_status.map(|s| s.to_string()).unwrap_or_else(|| "Unknown".to_string())
    )))
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
      // 如果配置了 OpenAI 兼容服务器，则优先直接调用
      if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
        let content = self.get_message_content(question_id)?;
        let text = self
          .openai_chat_once(&cfg, Some(&ai_model.name), content)
          .await?;
        // 将一次性结果转成单次消息流
        let s = stream::once(async move {
          Ok(flowy_ai_pub::cloud::QuestionStreamValue::Answer { value: text })
        });
        return Ok(Box::pin(s));
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
      // 若配置了 OpenAI 兼容服务器，则直接调用
      if let Some(cfg) = self.read_openai_compat_chat_config(workspace_id) {
        let text = self
          .openai_chat_once(&cfg, Some(&ai_model.name), params.text.clone())
          .await?;
        let s = stream::once(async move {
          Ok(flowy_ai_pub::cloud::CompletionStreamValue::Answer { value: text })
        });
        return Ok(Box::pin(s));
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
