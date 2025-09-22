use crate::local_ai::controller::LocalAIController;
use crate::persistence::AIPersistence;
use crate::entities::GlobalAIModelTypePB;
use crate::openai_compatible::{OpenAICompatibleChatClient, OpenAICompatibleConfig, ChatMessage as OpenAIChatMessage, OpenAICompatibleEmbedder};
use flowy_ai_pub::persistence::select_message_content;
use std::collections::HashMap;

use flowy_ai_pub::cloud::{
  AIModel, ChatCloudService, ChatMessage, ChatMessageType, ChatSettings, CompleteTextParams,
  MessageCursor, ModelList, RelatedQuestion, RepeatedChatMessage, RepeatedRelatedQuestion,
  ResponseFormat, StreamAnswer, StreamComplete, UpdateChatParams, QuestionStreamValue, CompletionStreamValue,
};
use flowy_error::{FlowyError, FlowyResult};
use lib_infra::async_trait::async_trait;

use flowy_ai_pub::user_service::AIUserService;
use flowy_storage_pub::storage::StorageService;
use flowy_sqlite::kv::KVStorePreferences;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Weak};
use tracing::{info, trace, debug, warn};
use uuid::Uuid;
use futures::stream::StreamExt;

pub struct ChatServiceMiddleware {
  cloud_service: Arc<dyn ChatCloudService>,
  user_service: Arc<dyn AIUserService>,
  local_ai: Arc<LocalAIController>,
  store_preferences: Weak<KVStorePreferences>,
  #[allow(dead_code)]
  storage_service: Weak<dyn StorageService>,
}

impl ChatServiceMiddleware {
  pub fn new(
    user_service: Arc<dyn AIUserService>,
    cloud_service: Arc<dyn ChatCloudService>,
    local_ai: Arc<LocalAIController>,
    store_preferences: Weak<KVStorePreferences>,
    storage_service: Weak<dyn StorageService>,
  ) -> Self {
    Self {
      user_service,
      cloud_service,
      local_ai,
      store_preferences,
      storage_service,
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

  /// Get the global AI model type from persistence
  fn get_global_model_type(&self) -> FlowyResult<GlobalAIModelTypePB> {
    let persistence = AIPersistence::new(self.store_preferences.clone());
    persistence.load_global_model_type()
  }

  /// Create OpenAI compatible chat client if configured
  fn create_openai_chat_client(&self) -> FlowyResult<Option<OpenAICompatibleChatClient>> {
    let persistence = AIPersistence::new(self.store_preferences.clone());
    
    match persistence.load_openai_compatible_setting()? {
      Some(setting) => {
        let config = OpenAICompatibleConfig {
          chat_endpoint: format!("{}/chat/completions", setting.chat_setting.api_endpoint.trim_end_matches('/')),
          chat_api_key: setting.chat_setting.api_key,
          chat_model: setting.chat_setting.model_name,
          embedding_endpoint: format!("{}/embeddings", setting.embedding_setting.api_endpoint.trim_end_matches('/')),
          embedding_api_key: setting.embedding_setting.api_key,
          embedding_model: setting.embedding_setting.model_name,
          timeout_ms: Some(setting.chat_setting.timeout_seconds as u64 * 1000),
          max_tokens: if setting.chat_setting.max_tokens > 0 {
            Some(setting.chat_setting.max_tokens as u32)
          } else {
            None
          },
          temperature: if setting.chat_setting.temperature >= 0.0 {
            Some(setting.chat_setting.temperature as f32)
          } else {
            None
          },
        };

        match OpenAICompatibleChatClient::new(config) {
          Ok(client) => {
            debug!("Successfully created OpenAI compatible chat client");
            Ok(Some(client))
          }
          Err(e) => {
            warn!("Failed to create OpenAI compatible chat client: {}", e);
            Err(FlowyError::internal().with_context(format!("Failed to create OpenAI chat client: {}", e)))
          }
        }
      }
      None => {
        debug!("No OpenAI compatible settings found");
        Ok(None)
      }
    }
  }

  /// Create OpenAI compatible embedding client if configured
  fn create_openai_embedding_client(&self) -> FlowyResult<Option<OpenAICompatibleEmbedder>> {
    let persistence = AIPersistence::new(self.store_preferences.clone());
    
    match persistence.load_openai_compatible_setting()? {
      Some(setting) => {
        let config = OpenAICompatibleConfig {
          chat_endpoint: format!("{}/chat/completions", setting.chat_setting.api_endpoint.trim_end_matches('/')),
          chat_api_key: setting.chat_setting.api_key,
          chat_model: setting.chat_setting.model_name,
          embedding_endpoint: format!("{}/embeddings", setting.embedding_setting.api_endpoint.trim_end_matches('/')),
          embedding_api_key: setting.embedding_setting.api_key,
          embedding_model: setting.embedding_setting.model_name,
          timeout_ms: Some(setting.chat_setting.timeout_seconds as u64 * 1000),
          max_tokens: if setting.chat_setting.max_tokens > 0 {
            Some(setting.chat_setting.max_tokens as u32)
          } else {
            None
          },
          temperature: if setting.chat_setting.temperature >= 0.0 {
            Some(setting.chat_setting.temperature as f32)
          } else {
            None
          },
        };

        match OpenAICompatibleEmbedder::new(config) {
          Ok(client) => {
            debug!("Successfully created OpenAI compatible embedding client");
            Ok(Some(client))
          }
          Err(e) => {
            warn!("Failed to create OpenAI compatible embedding client: {}", e);
            Err(FlowyError::internal().with_context(format!("Failed to create OpenAI embedding client: {}", e)))
          }
        }
      }
      None => {
        debug!("No OpenAI compatible settings found");
        Ok(None)
      }
    }
  }

  /// Determine if we should use OpenAI compatible service based on global settings
  async fn should_use_openai_compatible(&self) -> bool {
    match self.get_global_model_type() {
      Ok(GlobalAIModelTypePB::GlobalOpenAICompatible) => {
        debug!("Global model type is set to OpenAI compatible");
        true
      }
      Ok(GlobalAIModelTypePB::GlobalLocalAI) => {
        debug!("Global model type is set to Local AI");
        false
      }
      Err(e) => {
        warn!("Failed to get global model type, defaulting to Local AI: {}", e);
        false
      }
    }
  }

  /// Convert QuestionStreamValue to CompletionStreamValue for text completion
  fn convert_question_to_completion_stream(stream: StreamAnswer) -> StreamComplete {
    Box::pin(stream.map(|result| {
      result.map(|question_value| {
        match question_value {
          QuestionStreamValue::Answer { value } => CompletionStreamValue::Answer { value },
          QuestionStreamValue::Metadata { value } => CompletionStreamValue::Comment { value: value.to_string() },
          QuestionStreamValue::SuggestedQuestion { .. } => CompletionStreamValue::Comment { value: "".to_string() },
          QuestionStreamValue::FollowUp { .. } => CompletionStreamValue::Comment { value: "".to_string() },
        }
      })
    }))
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
    
    // Check global routing configuration
    if self.should_use_openai_compatible().await {
      info!("Routing to OpenAI compatible service based on global configuration");
      
      // Try to create OpenAI compatible client
      match self.create_openai_chat_client()? {
        Some(client) => {
          let content = self.get_message_content(question_id)?;
          debug!("Using OpenAI compatible client for streaming chat with content length: {}", content.len());
          
          // Convert content to OpenAI chat messages format
          let messages = vec![OpenAIChatMessage {
            role: "user".to_string(),
            content,
          }];
          
          match client.stream_chat_completion(messages, None, None, None).await {
            Ok(stream) => {
              info!("Successfully created OpenAI compatible stream");
              Ok(stream)
            }
            Err(e) => {
              warn!("OpenAI compatible streaming failed, falling back to local AI: {}", e);
              // Fallback to local AI if OpenAI fails
              if self.local_ai.is_ready().await {
                let content = self.get_message_content(question_id)?;
                self
                  .local_ai
                  .stream_question(chat_id, &content, format, &ai_model.name)
                  .await
              } else {
                Err(FlowyError::local_ai_not_ready())
              }
            }
          }
        }
        None => {
          warn!("OpenAI compatible client not configured, falling back to local AI");
          // Fallback to local AI if no OpenAI configuration
          if self.local_ai.is_ready().await {
            let content = self.get_message_content(question_id)?;
            self
              .local_ai
              .stream_question(chat_id, &content, format, &ai_model.name)
              .await
          } else {
            Err(FlowyError::local_ai_not_ready())
          }
        }
      }
    } else {
      // Use original logic for local AI or cloud service
      if ai_model.is_local {
        if self.local_ai.is_ready().await {
          let content = self.get_message_content(question_id)?;
          info!("Using local AI for streaming chat");
          self
            .local_ai
            .stream_question(chat_id, &content, format, &ai_model.name)
            .await
        } else {
          Err(FlowyError::local_ai_not_ready())
        }
      } else {
        info!("Using cloud service for streaming chat");
        self
          .cloud_service
          .stream_answer(workspace_id, chat_id, question_id, format, ai_model)
          .await
      }
    }
  }

  async fn get_answer(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
  ) -> Result<ChatMessage, FlowyError> {
    if self.local_ai.is_ready().await {
      let content = self.get_message_content(question_id)?;
      let answer = self.local_ai.ask_question(chat_id, &content).await?;

      let message = self
        .cloud_service
        .create_answer(workspace_id, chat_id, &answer, question_id, None)
        .await?;
      Ok(message)
    } else {
      self
        .cloud_service
        .get_answer(workspace_id, chat_id, question_id)
        .await
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
        Ok(RepeatedRelatedQuestion {
          message_id,
          items: vec![],
        })
      }
    } else {
      self
        .cloud_service
        .get_related_message(workspace_id, chat_id, message_id, ai_model)
        .await
    }
  }

  async fn stream_complete(
    &self,
    workspace_id: &Uuid,
    params: CompleteTextParams,
    ai_model: AIModel,
  ) -> Result<StreamComplete, FlowyError> {
    info!("stream_complete use custom model: {:?}", ai_model);
    
    // Check global routing configuration
    if self.should_use_openai_compatible().await {
      info!("Routing text completion to OpenAI compatible service based on global configuration");
      
      // Try to create OpenAI compatible client
      match self.create_openai_chat_client()? {
        Some(client) => {
          debug!("Using OpenAI compatible client for text completion");
          
          // Convert text completion to chat completion format
          let messages = vec![OpenAIChatMessage {
            role: "user".to_string(),
            content: params.text.clone(),
          }];
          
          match client.stream_chat_completion(messages, None, None, None).await {
            Ok(stream) => {
              info!("Successfully created OpenAI compatible completion stream");
              Ok(Self::convert_question_to_completion_stream(stream))
            }
            Err(e) => {
              warn!("OpenAI compatible completion failed, falling back to local AI: {}", e);
              // Fallback to local AI if OpenAI fails
              if self.local_ai.is_ready().await {
                self.local_ai.complete_text(&ai_model.name, params).await
              } else {
                Err(FlowyError::local_ai_not_ready())
              }
            }
          }
        }
        None => {
          warn!("OpenAI compatible client not configured, falling back to local AI");
          // Fallback to local AI if no OpenAI configuration
          if self.local_ai.is_ready().await {
            self.local_ai.complete_text(&ai_model.name, params).await
          } else {
            Err(FlowyError::local_ai_not_ready())
          }
        }
      }
    } else {
      // Use original logic for local AI or cloud service
      if ai_model.is_local {
        if self.local_ai.is_ready().await {
          info!("Using local AI for text completion");
          self.local_ai.complete_text(&ai_model.name, params).await
        } else {
          Err(FlowyError::local_ai_not_ready())
        }
      } else {
        info!("Using cloud service for text completion");
        self
          .cloud_service
          .stream_complete(workspace_id, params, ai_model)
          .await
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
    // Check global routing configuration
    if self.should_use_openai_compatible().await {
      info!("Routing file embedding to OpenAI compatible service based on global configuration");
      
      // TODO: Implement OpenAI compatible file embedding in future tasks
      // For now, fall back to local AI or cloud service
      warn!("OpenAI compatible file embedding not yet implemented, falling back to local AI");
      if self.local_ai.is_ready().await {
        self
          .local_ai
          .embed_file(chat_id, file_path.to_path_buf(), metadata)
          .await?;
        Ok(())
      } else {
        self
          .cloud_service
          .embed_file(workspace_id, file_path, chat_id, metadata)
          .await
      }
    } else {
      // Use original logic for local AI or cloud service
      if self.local_ai.is_ready().await {
        info!("Using local AI for file embedding");
        self
          .local_ai
          .embed_file(chat_id, file_path.to_path_buf(), metadata)
          .await?;
        Ok(())
      } else {
        info!("Using cloud service for file embedding");
        self
          .cloud_service
          .embed_file(workspace_id, file_path, chat_id, metadata)
          .await
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
