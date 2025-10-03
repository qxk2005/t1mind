use crate::chat::Chat;
use crate::entities::{
  AIModelPB, ChatInfoPB, ChatMessageListPB, ChatMessagePB, ChatSettingsPB,
  CustomPromptDatabaseConfigurationPB, FilePB, ModelSelectionPB, PredefinedFormatPB,
  RepeatedRelatedQuestionPB, StreamMessageParams,
  AgentListPB, AgentConfigPB, CreateAgentRequestPB, GetAgentRequestPB, 
  UpdateAgentRequestPB, DeleteAgentRequestPB, AgentSuccessResponsePB, AgentGlobalSettingsPB,
  AgentExecutionLogPB, AgentExecutionLogListPB, GetExecutionLogsRequestPB, ClearExecutionLogsRequestPB,
};
use crate::local_ai::controller::{LocalAIController, LocalAISetting};
use crate::middleware::chat_service_mw::ChatServiceMiddleware;
use crate::mcp::manager::MCPClientManager;
use crate::agent::config_manager::AgentConfigManager;
use flowy_ai_pub::persistence::{
  ChatTableChangeset, select_chat_metadata, select_chat_rag_ids, select_chat_summary, update_chat,
};
use std::collections::HashMap;

use dashmap::DashMap;
use flowy_ai_pub::cloud::{AIModel, ChatCloudService, ChatSettings, UpdateChatParams};
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;

use crate::model_select::{
  GLOBAL_ACTIVE_MODEL_KEY, LocalAiSource, LocalModelStorageImpl, ModelSelectionControl,
  ServerAiSource, ServerModelStorageImpl, SourceKey,
};
use crate::notification::{ChatNotification, chat_notification_builder};
use flowy_ai_pub::persistence::{
  AFCollabMetadata, batch_insert_collab_metadata, batch_select_collab_metadata,
};
use flowy_ai_pub::user_service::AIUserService;
use flowy_sqlite::DBConnection;
use flowy_storage_pub::storage::StorageService;
use lib_infra::async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;
use tracing::{error, info, instrument, trace, warn};
use uuid::Uuid;

/// AIExternalService is an interface for external services that AI plugin can interact with.
#[async_trait]
pub trait AIExternalService: Send + Sync + 'static {
  async fn query_chat_rag_ids(
    &self,
    parent_view_id: &Uuid,
    chat_id: &Uuid,
  ) -> Result<Vec<Uuid>, FlowyError>;

  async fn sync_rag_documents(
    &self,
    workspace_id: &Uuid,
    rag_ids: Vec<Uuid>,
    rag_metadata_map: HashMap<Uuid, AFCollabMetadata>,
  ) -> Result<Vec<AFCollabMetadata>, FlowyError>;

  async fn notify_did_send_message(&self, chat_id: &Uuid, message: &str) -> Result<(), FlowyError>;
}

pub struct AIManager {
  pub cloud_service_wm: Arc<ChatServiceMiddleware>,
  pub user_service: Arc<dyn AIUserService>,
  pub external_service: Arc<dyn AIExternalService>,
  chats: Arc<DashMap<Uuid, Arc<Chat>>>,
  pub local_ai: Arc<LocalAIController>,
  pub store_preferences: Arc<KVStorePreferences>,
  model_control: Mutex<ModelSelectionControl>,
  pub mcp_manager: Arc<MCPClientManager>,
  pub agent_manager: Arc<AgentConfigManager>,
  execution_logs: Arc<DashMap<String, Vec<AgentExecutionLogPB>>>,
}
impl Drop for AIManager {
  fn drop(&mut self) {
    tracing::trace!("[Drop] drop ai manager");
  }
}

impl AIManager {
  pub fn new(
    chat_cloud_service: Arc<dyn ChatCloudService>,
    user_service: impl AIUserService,
    store_preferences: Arc<KVStorePreferences>,
    storage_service: Weak<dyn StorageService>,
    query_service: impl AIExternalService,
    local_ai: Arc<LocalAIController>,
  ) -> AIManager {
    let user_service = Arc::new(user_service);
    let external_service = Arc::new(query_service);
    let cloud_service_wm = Arc::new(ChatServiceMiddleware::new(
      user_service.clone(),
      chat_cloud_service,
      local_ai.clone(),
      storage_service,
      store_preferences.clone(),
    ));
    let mut model_control = ModelSelectionControl::new();
    model_control.set_local_storage(LocalModelStorageImpl(store_preferences.clone()));
    model_control.set_server_storage(ServerModelStorageImpl(cloud_service_wm.clone()));
    model_control.add_source(Box::new(ServerAiSource::new(cloud_service_wm.clone())));

    let mcp_manager = Arc::new(MCPClientManager::new(store_preferences.clone()));
    let agent_manager = Arc::new(AgentConfigManager::new(store_preferences.clone()));

    Self {
      cloud_service_wm,
      user_service,
      chats: Arc::new(DashMap::new()),
      local_ai,
      external_service,
      store_preferences,
      model_control: Mutex::new(model_control),
      mcp_manager,
      agent_manager,
      execution_logs: Arc::new(DashMap::new()),
    }
  }

  async fn reload_with_workspace_id(&self, workspace_id: &Uuid) {
    // Check if local AI is enabled for this workspace and if we're in local mode
    let result = self.user_service.is_local_model().await;
    if let Err(err) = &result {
      if matches!(err.code, ErrorCode::UserNotLogin) {
        info!("[AI Manager] User not logged in, skipping local AI reload");
        return;
      }
    }

    let is_local = result.unwrap_or(false);
    let is_enabled = self
      .local_ai
      .is_enabled_on_workspace(&workspace_id.to_string());
    let is_ready = self.local_ai.is_ready().await;
    info!(
      "[AI Manager] Reloading workspace: {}, is_local: {}, is_enabled: {}, is_ready: {}",
      workspace_id, is_local, is_enabled, is_ready
    );

    // Shutdown AI if it's running but shouldn't be (not enabled and not in local mode)
    if is_ready && !is_enabled && !is_local {
      info!("[AI Manager] Local AI is running but not enabled, shutting it down");
      let local_ai = self.local_ai.clone();
      tokio::spawn(async move {
        if let Err(err) = local_ai.toggle_plugin(false).await {
          error!("[AI Manager] failed to shutdown local AI: {:?}", err);
        }
      });
      return;
    }

    // Start AI if it's enabled but not running
    if is_enabled && !is_ready {
      info!("[AI Manager] Local AI is enabled but not running, starting it now");
      let local_ai = self.local_ai.clone();
      tokio::spawn(async move {
        if let Err(err) = local_ai.toggle_plugin(true).await {
          error!("[AI Manager] failed to start local AI: {:?}", err);
        }
      });
      return;
    }

    // Log status for other cases
    if is_ready {
      info!("[AI Manager] Local AI is already running");
    }
  }

  async fn prepare_local_ai(&self, workspace_id: &Uuid, is_enabled: bool) {
    self
      .local_ai
      .reload_ollama_client(&workspace_id.to_string())
      .await;
    if is_enabled {
      self
        .model_control
        .lock()
        .await
        .add_source(Box::new(LocalAiSource::new(self.local_ai.clone())));
    } else {
      self.model_control.lock().await.remove_local_source();
    }
  }

  #[instrument(skip_all, err)]
  pub async fn on_launch_if_authenticated(&self, workspace_id: &Uuid) -> Result<(), FlowyError> {
    let is_enabled = self
      .local_ai
      .is_enabled_on_workspace(&workspace_id.to_string());

    info!("{} local ai is enabled: {}", workspace_id, is_enabled);
    self.prepare_local_ai(workspace_id, is_enabled).await;
    self.reload_with_workspace_id(workspace_id).await;
    Ok(())
  }

  pub async fn initialize_after_sign_in(&self, workspace_id: &Uuid) -> Result<(), FlowyError> {
    self.on_launch_if_authenticated(workspace_id).await?;
    Ok(())
  }

  pub async fn initialize_after_sign_up(&self, workspace_id: &Uuid) -> Result<(), FlowyError> {
    self.on_launch_if_authenticated(workspace_id).await?;
    Ok(())
  }

  #[instrument(skip_all, err)]
  pub async fn initialize_after_open_workspace(
    &self,
    workspace_id: &Uuid,
  ) -> Result<(), FlowyError> {
    self.on_launch_if_authenticated(workspace_id).await?;
    Ok(())
  }

  pub async fn open_chat(&self, chat_id: &Uuid) -> Result<(), FlowyError> {
    self.chats.entry(*chat_id).or_insert_with(|| {
      Arc::new(Chat::new(
        self.user_service.user_id().unwrap(),
        *chat_id,
        self.user_service.clone(),
        self.cloud_service_wm.clone(),
      ))
    });

    if self.local_ai.is_enabled() {
      let workspace_id = self.user_service.workspace_id()?;
      let uid = self.user_service.user_id()?;
      let mut conn = self.user_service.sqlite_connection(uid)?;
      let rag_ids = self.get_rag_ids(chat_id, &mut conn).await?;
      let summary = select_chat_summary(&mut conn, chat_id).unwrap_or_default();

      let model = self.get_active_model(&chat_id.to_string()).await;
      self
        .local_ai
        .open_chat(&workspace_id, chat_id, &model.name, rag_ids, summary)
        .await?;
    }

    let user_service = self.user_service.clone();
    let cloud_service_wm = self.cloud_service_wm.clone();
    let store_preferences = self.store_preferences.clone();
    let external_service = self.external_service.clone();
    let local_ai = self.local_ai.clone();
    let chat_id = *chat_id;
    tokio::spawn(async move {
      match refresh_chat_setting(
        &user_service,
        &cloud_service_wm,
        &store_preferences,
        &chat_id,
      )
      .await
      {
        Ok(settings) => {
          local_ai.set_rag_ids(&chat_id, &settings.rag_ids).await;
          let rag_ids = settings
            .rag_ids
            .into_iter()
            .flat_map(|r| Uuid::from_str(&r).ok())
            .collect();
          let _ = sync_chat_documents(user_service, external_service, rag_ids).await;
        },
        Err(err) => {
          error!("failed to refresh chat settings: {}", err);
        },
      }
    });

    Ok(())
  }

  pub async fn close_chat(&self, chat_id: &Uuid) -> Result<(), FlowyError> {
    trace!("close chat: {}", chat_id);
    self.local_ai.close_chat(chat_id);
    Ok(())
  }

  pub async fn delete_chat(&self, chat_id: &Uuid) -> Result<(), FlowyError> {
    if let Some((_, chat)) = self.chats.remove(chat_id) {
      chat.close();
      self.local_ai.close_chat(chat_id);
    }
    Ok(())
  }

  pub async fn get_chat_info(&self, chat_id: &str) -> FlowyResult<ChatInfoPB> {
    let uid = self.user_service.user_id()?;
    let mut conn = self.user_service.sqlite_connection(uid)?;
    let metadata = select_chat_metadata(&mut conn, chat_id)?;
    let files = metadata
      .files
      .into_iter()
      .map(|file| FilePB {
        id: file.id,
        name: file.name,
      })
      .collect();

    Ok(ChatInfoPB {
      chat_id: chat_id.to_string(),
      files,
    })
  }

  pub async fn create_chat(
    &self,
    uid: &i64,
    parent_view_id: &Uuid,
    chat_id: &Uuid,
  ) -> Result<Arc<Chat>, FlowyError> {
    let workspace_id = self.user_service.workspace_id()?;
    let rag_ids = self
      .external_service
      .query_chat_rag_ids(parent_view_id, chat_id)
      .await
      .unwrap_or_default();
    info!("[Chat] create chat with rag_ids: {:?}", rag_ids);

    self
      .cloud_service_wm
      .create_chat(uid, &workspace_id, chat_id, rag_ids, "", json!({}))
      .await?;

    let chat = Arc::new(Chat::new(
      self.user_service.user_id()?,
      *chat_id,
      self.user_service.clone(),
      self.cloud_service_wm.clone(),
    ));
    self.chats.insert(*chat_id, chat.clone());
    Ok(chat)
  }

  pub async fn stream_chat_message(
    &self,
    params: StreamMessageParams,
  ) -> Result<ChatMessagePB, FlowyError> {
    // 如果有 agent_id，加载智能体配置
    let agent_config = if let Some(ref agent_id) = params.agent_id {
      match self.agent_manager.get_agent_config(agent_id) {
        Some(mut config) => {
          info!("[Chat] Using agent: {} ({})", config.name, config.id);
          info!("[Chat] Agent has {} tools, tool_calling enabled: {}", 
                config.available_tools.len(), config.capabilities.enable_tool_calling);
          
          // 🔍 获取工具详情用于增强系统提示
          let (discovered_tool_names, tool_details) = self.discover_available_tools().await;
          info!("[Chat] Discovered {} tools with {} tool details", 
                discovered_tool_names.len(), tool_details.len());
          
          // 自动填充工具列表（如果为空）
          if config.available_tools.is_empty() && config.capabilities.enable_tool_calling {
            info!("[Chat] 智能体工具列表为空，开始自动发现 MCP 工具...");
            
            if !discovered_tool_names.is_empty() {
              config.available_tools = discovered_tool_names.clone();
              config.updated_at = chrono::Utc::now().timestamp();
              
              // 使用更新方法保存配置
              let update_request = crate::entities::UpdateAgentRequestPB {
                id: config.id.clone(),
                name: None,
                description: None,
                avatar: None,
                personality: None,
                capabilities: None,
                available_tools: config.available_tools.clone(),
                status: None,
                metadata: std::collections::HashMap::new(),
              };
              
              if let Err(e) = self.agent_manager.update_agent(update_request) {
                warn!("Failed to save agent config after tool population: {}", e);
              } else {
                info!("为智能体 {} 自动发现并填充了 {} 个工具", 
                      config.name, config.available_tools.len());
              }
            } else {
              warn!("未发现任何可用的 MCP 工具，智能体 {} 将无法使用工具调用功能", config.name);
            }
          }
          
          // 🆕 构建增强的系统提示（包含工具详情）
          let enhanced_prompt = if !tool_details.is_empty() && config.capabilities.enable_tool_calling {
            use crate::agent::system_prompt::build_agent_system_prompt_with_tools;
            let prompt = build_agent_system_prompt_with_tools(&config, &tool_details);
            info!("[Chat] 🔧 Using enhanced system prompt with {} tool details", tool_details.len());
            Some(prompt)
          } else {
            None
          };
          
          Some((config, enhanced_prompt))
        },
        None => {
          warn!("[Chat] Agent not found: {}", agent_id);
          None
        }
      }
    } else {
      None
    };

    // 🔧 任务规划提示（实际规划由AI自动判断，通过系统提示词指导）
    // TODO: 如果需要自动任务规划，需要在 AIManager 中添加 plan_integration 字段
    // 目前任务规划功能通过增强的系统提示词实现，AI会根据需要创建计划

    // 解包 agent_config 和 enhanced_prompt
    let (agent_config, enhanced_prompt) = if let Some((config, prompt)) = agent_config {
      (Some(config), prompt)
    } else {
      (None, None)
    };

    // 🔧 创建工具调用处理器（如果有智能体配置）
    let tool_call_handler = if agent_config.is_some() {
      use crate::agent::ToolCallHandler;
      Some(Arc::new(ToolCallHandler::from_ai_manager(self)))
    } else {
      None
    };

    let chat = self.get_or_create_chat_instance(&params.chat_id).await?;
    let ai_model = self.get_active_model(&params.chat_id.to_string()).await;
    let question = chat.stream_chat_message(&params, ai_model, agent_config, tool_call_handler, enhanced_prompt).await?;
    let _ = self
      .external_service
      .notify_did_send_message(&params.chat_id, &params.message)
      .await;
    Ok(question)
  }

  pub async fn stream_regenerate_response(
    &self,
    chat_id: &Uuid,
    answer_message_id: i64,
    answer_stream_port: i64,
    format: Option<PredefinedFormatPB>,
    model: Option<AIModelPB>,
  ) -> FlowyResult<()> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    let question_message_id = chat
      .get_question_id_from_answer_id(chat_id, answer_message_id)
      .await?;

    let model = match model {
      None => self.get_active_model(&chat_id.to_string()).await,
      Some(model) => model.into(),
    };
    chat
      .stream_regenerate_response(question_message_id, answer_stream_port, format, model)
      .await?;
    Ok(())
  }

  pub async fn update_local_ai_setting(&self, setting: LocalAISetting) -> FlowyResult<()> {
    let workspace_id = self.user_service.workspace_id()?;
    let old_settings = self.local_ai.get_local_ai_setting();
    // Only restart if the server URL has changed and local AI is not running
    let need_restart = old_settings.ollama_server_url != setting.ollama_server_url;

    // Update settings first
    self
      .local_ai
      .update_local_ai_setting(setting.clone())
      .await?;

    // Handle model change if needed
    info!(
      "[AI Plugin] update global active model, previous: {}, current: {}",
      old_settings.chat_model_name, setting.chat_model_name
    );
    let model = AIModel::local(setting.chat_model_name, "".to_string());
    self
      .update_selected_model(GLOBAL_ACTIVE_MODEL_KEY.to_string(), model)
      .await?;

    if need_restart {
      self
        .local_ai
        .reload_ollama_client(&workspace_id.to_string())
        .await;
      self.local_ai.restart_plugin().await;
    }

    Ok(())
  }

  #[instrument(skip_all, level = "debug")]
  pub async fn update_selected_model(&self, source: String, model: AIModel) -> FlowyResult<()> {
    let workspace_id = self.user_service.workspace_id()?;
    let source_key = SourceKey::new(source.clone());
    self
      .model_control
      .lock()
      .await
      .set_active_model(&workspace_id, &source_key, model.clone())
      .await?;

    info!(
      "[Model Selection] selected model: {:?} for key:{}",
      model,
      source_key.storage_id()
    );

    let mut notify_source = vec![source.clone()];
    if source == GLOBAL_ACTIVE_MODEL_KEY {
      let ids = self
        .model_control
        .lock()
        .await
        .get_all_unset_sources()
        .await;
      info!("[Model Selection] notify all unset sources: {:?}", ids);
      notify_source.extend(ids);
    }

    trace!("[Model Selection] notify sources: {:?}", notify_source);
    for source in notify_source {
      chat_notification_builder(&source, ChatNotification::DidUpdateSelectedModel)
        .payload(AIModelPB::from(model.clone()))
        .send();
    }

    Ok(())
  }

  #[instrument(skip_all, level = "debug", err)]
  pub async fn toggle_local_ai(&self) -> FlowyResult<()> {
    let enabled = self.local_ai.toggle_local_ai().await?;
    let workspace_id = self.user_service.workspace_id()?;
    if enabled {
      self.prepare_local_ai(&workspace_id, enabled).await;

      if let Some(name) = self.local_ai.get_local_chat_model() {
        let model = AIModel::local(name, "".to_string());
        info!(
          "[Model Selection] Set global active model to local ai: {}",
          model.name
        );
        if let Err(err) = self
          .update_selected_model(GLOBAL_ACTIVE_MODEL_KEY.to_string(), model)
          .await
        {
          error!(
            "[Model Selection] Failed to set global active model: {}",
            err
          );
        }
      }
    } else {
      let mut model_control = self.model_control.lock().await;
      model_control.remove_local_source();

      let model = model_control.get_global_active_model(&workspace_id).await;
      let mut notify_source = model_control.get_all_unset_sources().await;
      notify_source.push(GLOBAL_ACTIVE_MODEL_KEY.to_string());
      drop(model_control);

      trace!(
        "[Model Selection] notify sources: {:?}, model:{}, when disable local ai",
        notify_source, model.name
      );
      for source in notify_source {
        chat_notification_builder(&source, ChatNotification::DidUpdateSelectedModel)
          .payload(AIModelPB::from(model.clone()))
          .send();
      }
    }

    Ok(())
  }

  pub async fn get_active_model(&self, source: &str) -> AIModel {
    match self.user_service.workspace_id() {
      Ok(workspace_id) => {
        let prefer_local = self.user_service.is_local_model().await.unwrap_or(false);
        let source_key = SourceKey::new(source.to_string());
        let current = self
          .model_control
          .lock()
          .await
          .get_active_model(&workspace_id, &source_key)
          .await;

        // Provider routing: enforce provider preference, with graceful fallback
        if prefer_local {
          if current.is_local {
            return current;
          }
          // prefer local model; if local AI not ready, fall back to server model
          if self.local_ai.is_ready().await {
            let name = self.local_ai.get_local_ai_setting().chat_model_name;
            return AIModel::local(name, "".to_string());
          } else if let Ok(name) = self
            .cloud_service_wm
            .get_workspace_default_model(&workspace_id)
            .await
          {
            return AIModel::server(name, "".to_string());
          }
          return current;
        } else {
          // prefer server model; if current is local, replace with server default when available
          if current.is_local {
            if let Ok(name) = self
              .cloud_service_wm
              .get_workspace_default_model(&workspace_id)
              .await
            {
              return AIModel::server(name, "".to_string());
            }
          }
          return current;
        }
      },
      Err(_) => AIModel::default(),
    }
  }

  pub async fn get_local_available_models(
    &self,
    source: Option<String>,
  ) -> FlowyResult<ModelSelectionPB> {
    let workspace_id = self.user_service.workspace_id()?;
    let mut models = self
      .model_control
      .lock()
      .await
      .get_local_models(&workspace_id)
      .await;

    let selected_model = match source {
      None => {
        let setting = self.local_ai.get_local_ai_setting();
        let selected_model = AIModel::local(setting.chat_model_name, "".to_string());
        if models.is_empty() {
          models.push(selected_model.clone());
        }
        selected_model
      },
      Some(source) => {
        let source_key = SourceKey::new(source);
        self
          .model_control
          .lock()
          .await
          .get_active_model(&workspace_id, &source_key)
          .await
      },
    };

    Ok(ModelSelectionPB {
      models: models.into_iter().map(AIModelPB::from).collect(),
      selected_model: AIModelPB::from(selected_model),
    })
  }

  pub async fn get_available_models(
    &self,
    source: String,
    setting_only: bool,
  ) -> FlowyResult<ModelSelectionPB> {
    let is_local_mode = self.user_service.is_local_model().await?;
    if is_local_mode {
      // 仅本地：返回本地模型列表，并将默认选中设为本地配置模型
      return self.get_local_available_models(Some(source)).await;
    }

    let workspace_id = self.user_service.workspace_id()?;
    let local_model_name = if setting_only {
      Some(self.local_ai.get_local_ai_setting().chat_model_name)
    } else {
      None
    };

    let source_key = SourceKey::new(source);
    let model_control = self.model_control.lock().await;
    let active_model = model_control
      .get_active_model(&workspace_id, &source_key)
      .await;

    trace!(
      "[Model Selection] {} active model: {:?}, global model:{:?}",
      source_key.storage_id(),
      active_model,
      local_model_name
    );

    // Server + 可选本地：融合模型列表（服务端 + 指定本地一项或全部）
    let all_models = model_control
      .get_models_with_specific_local_model(&workspace_id, local_model_name)
      .await;
    drop(model_control);

    Ok(ModelSelectionPB {
      models: all_models.into_iter().map(AIModelPB::from).collect(),
      selected_model: AIModelPB::from(active_model),
    })
  }

  pub async fn get_or_create_chat_instance(&self, chat_id: &Uuid) -> Result<Arc<Chat>, FlowyError> {
    let chat = self.chats.get(chat_id).as_deref().cloned();
    match chat {
      None => {
        let chat = Arc::new(Chat::new(
          self.user_service.user_id()?,
          *chat_id,
          self.user_service.clone(),
          self.cloud_service_wm.clone(),
        ));
        self.chats.insert(*chat_id, chat.clone());
        Ok(chat)
      },
      Some(chat) => Ok(chat),
    }
  }

  /// Load chat messages for a given `chat_id`.
  ///
  /// 1. When opening a chat:
  ///    - Loads local chat messages.
  ///    - `after_message_id` and `before_message_id` are `None`.
  ///    - Spawns a task to load messages from the remote server, notifying the user when the remote messages are loaded.
  ///
  /// 2. Loading more messages in an existing chat with `after_message_id`:
  ///    - `after_message_id` is the last message ID in the current chat messages.
  ///
  /// 3. Loading more messages in an existing chat with `before_message_id`:
  ///    - `before_message_id` is the first message ID in the current chat messages.
  ///
  /// 4. `after_message_id` and `before_message_id` cannot be specified at the same time.
  pub async fn load_prev_chat_messages(
    &self,
    chat_id: &Uuid,
    limit: u64,
    before_message_id: Option<i64>,
  ) -> Result<ChatMessageListPB, FlowyError> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    let list = chat
      .load_prev_chat_messages(limit, before_message_id)
      .await?;
    Ok(list)
  }

  pub async fn load_latest_chat_messages(
    &self,
    chat_id: &Uuid,
    limit: u64,
    after_message_id: Option<i64>,
  ) -> Result<ChatMessageListPB, FlowyError> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    let list = chat
      .load_latest_chat_messages(limit, after_message_id)
      .await?;
    Ok(list)
  }

  pub async fn get_related_questions(
    &self,
    chat_id: &Uuid,
    message_id: i64,
  ) -> Result<RepeatedRelatedQuestionPB, FlowyError> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    let ai_model = self.get_active_model(&chat_id.to_string()).await;
    let resp = chat.get_related_question(message_id, ai_model).await?;
    Ok(resp)
  }

  pub async fn generate_answer(
    &self,
    chat_id: &Uuid,
    question_message_id: i64,
  ) -> Result<ChatMessagePB, FlowyError> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    let resp = chat.generate_answer(question_message_id).await?;
    Ok(resp)
  }

  pub async fn stop_stream(&self, chat_id: &Uuid) -> Result<(), FlowyError> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    chat.stop_stream_message().await;
    Ok(())
  }

  pub async fn chat_with_file(&self, chat_id: &Uuid, file_path: PathBuf) -> FlowyResult<()> {
    let chat = self.get_or_create_chat_instance(chat_id).await?;
    chat.index_file(file_path).await?;
    Ok(())
  }

  pub async fn get_rag_ids(
    &self,
    chat_id: &Uuid,
    conn: &mut DBConnection,
  ) -> FlowyResult<Vec<String>> {
    match select_chat_rag_ids(&mut *conn, &chat_id.to_string()) {
      Ok(ids) => {
        return Ok(ids);
      },
      Err(_) => {
        // we no long use store_preferences to store chat settings
        warn!("[Chat] failed to get chat rag ids from sqlite, try to get from store_preferences");
        if let Some(settings) = self
          .store_preferences
          .get_object::<ChatSettings>(&setting_store_key(chat_id))
        {
          return Ok(settings.rag_ids);
        }
      },
    }

    let settings = refresh_chat_setting(
      &self.user_service,
      &self.cloud_service_wm,
      &self.store_preferences,
      chat_id,
    )
    .await?;
    Ok(settings.rag_ids)
  }

  pub async fn update_rag_ids(&self, chat_id: &Uuid, rag_ids: Vec<String>) -> FlowyResult<()> {
    info!("[Chat] update chat:{} rag ids: {:?}", chat_id, rag_ids);
    let workspace_id = self.user_service.workspace_id()?;
    let update_setting = UpdateChatParams {
      name: None,
      metadata: None,
      rag_ids: Some(rag_ids.clone()),
    };
    self
      .cloud_service_wm
      .update_chat_settings(&workspace_id, chat_id, update_setting)
      .await?;

    let uid = self.user_service.user_id()?;
    let conn = self.user_service.sqlite_connection(uid)?;
    update_chat(
      conn,
      ChatTableChangeset::rag_ids(chat_id.to_string(), rag_ids.clone()),
    )?;

    let user_service = self.user_service.clone();
    let external_service = self.external_service.clone();
    self.local_ai.set_rag_ids(chat_id, &rag_ids).await;

    let rag_ids = rag_ids
      .into_iter()
      .flat_map(|r| Uuid::from_str(&r).ok())
      .collect();
    sync_chat_documents(user_service, external_service, rag_ids).await?;
    Ok(())
  }

  pub async fn get_custom_prompt_database_configuration(
    &self,
  ) -> FlowyResult<Option<CustomPromptDatabaseConfigurationPB>> {
    let view_id = self
      .store_preferences
      .get_object::<CustomPromptDatabaseConfigurationPB>(CUSTOM_PROMPT_DATABASE_CONFIGURATION_KEY);

    Ok(view_id)
  }

  pub async fn set_custom_prompt_database_configuration(
    &self,
    config: CustomPromptDatabaseConfigurationPB,
  ) -> FlowyResult<()> {
    if let Err(err) = self
      .store_preferences
      .set_object(CUSTOM_PROMPT_DATABASE_CONFIGURATION_KEY, &config)
    {
      error!(
        "failed to set custom prompt database configuration settings: {}",
        err
      );
    }

    Ok(())
  }

  // ==================== 智能体管理方法 ====================

  /// 获取智能体列表
  pub async fn get_agent_list(&self) -> FlowyResult<AgentListPB> {
    self.agent_manager.get_all_agents()
  }

  /// 创建智能体
  pub async fn create_agent(&self, mut request: CreateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
    // 如果工具列表为空且启用了工具调用，动态发现工具
    if request.available_tools.is_empty() && request.capabilities.enable_tool_calling {
      let (discovered_tool_names, _tool_details) = self.discover_available_tools().await;
      
      if !discovered_tool_names.is_empty() {
        info!("为新智能体 '{}' 自动发现了 {} 个工具", request.name, discovered_tool_names.len());
        request.available_tools = discovered_tool_names;
      } else {
        warn!("未发现任何可用的 MCP 工具，智能体 '{}' 将以空工具列表创建", request.name);
      }
    }
    
    self.agent_manager.create_agent(request)
  }

  /// 获取智能体配置
  pub async fn get_agent(&self, request: GetAgentRequestPB) -> FlowyResult<AgentConfigPB> {
    self.agent_manager.get_agent(request)
  }

  /// 更新智能体配置
  pub async fn update_agent(&self, mut request: UpdateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
    // 获取现有配置用于调试和比较
    let existing_config = self.agent_manager.get_agent_config(&request.id);
    
    info!("🔄 [Agent Update] 开始更新智能体: {}", request.id);
    info!("🔄 [Agent Update] 请求工具列表长度: {}", request.available_tools.len());
    info!("🔄 [Agent Update] 请求是否包含 capabilities: {}", request.capabilities.is_some());
    
    if let Some(ref existing) = existing_config {
      info!("🔄 [Agent Update] 现有智能体: {}", existing.name);
      info!("🔄 [Agent Update] 现有工具列表长度: {}", existing.available_tools.len());
      info!("🔄 [Agent Update] 现有 enable_tool_calling: {}", existing.capabilities.enable_tool_calling);
    }
    
    // 如果更新了能力配置，且启用了工具调用，但请求中的工具列表为空
    if let Some(ref capabilities) = request.capabilities {
      info!("🔄 [Agent Update] 新能力配置 - enable_tool_calling: {}", capabilities.enable_tool_calling);
      
      if capabilities.enable_tool_calling && request.available_tools.is_empty() {
        info!("🔄 [Agent Update] 条件满足：工具调用已启用且工具列表为空");
        
        if let Some(existing) = existing_config {
          let should_discover = existing.available_tools.is_empty() || 
                                capabilities.enable_tool_calling != existing.capabilities.enable_tool_calling;
          
          info!("🔄 [Agent Update] 是否需要发现工具: {}", should_discover);
          
          if should_discover {
            info!("✨ [Agent Update] 检测到工具调用能力变更或工具列表为空，开始自动发现工具...");
            let (discovered_tool_names, _tool_details) = self.discover_available_tools().await;
            
            if !discovered_tool_names.is_empty() {
              info!("✅ [Agent Update] 为智能体 '{}' 自动发现了 {} 个工具", 
                    existing.name, discovered_tool_names.len());
              request.available_tools = discovered_tool_names;
            } else {
              warn!("⚠️  [Agent Update] 未发现任何可用的 MCP 工具");
            }
          } else {
            info!("ℹ️  [Agent Update] 智能体已有工具且能力未变更，跳过工具发现");
          }
        }
      } else if !capabilities.enable_tool_calling {
        info!("ℹ️  [Agent Update] 工具调用未启用，跳过工具发现");
      } else {
        info!("ℹ️  [Agent Update] 请求中已包含 {} 个工具，跳过自动发现", request.available_tools.len());
      }
    } else {
      info!("ℹ️  [Agent Update] 未更新能力配置，跳过工具发现");
    }
    
    let result = self.agent_manager.update_agent(request);
    info!("🔄 [Agent Update] 更新完成");
    result
  }

  /// 删除智能体
  pub async fn delete_agent(&self, request: DeleteAgentRequestPB) -> FlowyResult<()> {
    self.agent_manager.delete_agent(request)?;
    Ok(())
  }

  /// 验证智能体配置
  pub async fn validate_agent_config(&self, config: AgentConfigPB) -> FlowyResult<AgentSuccessResponsePB> {
    // 执行配置验证逻辑
    let validation_errors = self.agent_manager.validate_agent_config(&config)?;
    
    if validation_errors.is_empty() {
      Ok(AgentSuccessResponsePB {
        success: true,
        message: Some("智能体配置验证通过".to_string()),
        data: std::collections::HashMap::new(),
      })
    } else {
      Err(FlowyError::invalid_data().with_context(validation_errors.join("; ")))
    }
  }

  /// 获取智能体全局设置
  pub async fn get_agent_global_settings(&self) -> FlowyResult<AgentGlobalSettingsPB> {
    let settings = self.agent_manager.get_global_settings();
    Ok(AgentGlobalSettingsPB {
      enabled: settings.enabled,
      default_max_planning_steps: settings.default_max_planning_steps,
      default_max_tool_calls: settings.default_max_tool_calls,
      default_memory_limit: settings.default_memory_limit,
      debug_logging: settings.debug_logging,
      execution_timeout: settings.execution_timeout,
      created_at: settings.created_at.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs() as i64,
      updated_at: settings.updated_at.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs() as i64,
    })
  }

  /// 更新智能体全局设置
  pub async fn update_agent_global_settings(&self, settings: AgentGlobalSettingsPB) -> FlowyResult<()> {
    use crate::agent::AgentGlobalSettings;
    use std::time::{SystemTime, UNIX_EPOCH};

    let global_settings = AgentGlobalSettings {
      enabled: settings.enabled,
      default_max_planning_steps: settings.default_max_planning_steps,
      default_max_tool_calls: settings.default_max_tool_calls,
      default_memory_limit: settings.default_memory_limit,
      debug_logging: settings.debug_logging,
      execution_timeout: settings.execution_timeout,
      created_at: UNIX_EPOCH + std::time::Duration::from_secs(settings.created_at as u64),
      updated_at: SystemTime::now(),
    };

    self.agent_manager.save_global_settings(global_settings)?;
    Ok(())
  }

  // ==================== 执行日志管理方法 ====================

  /// 获取执行日志列表
  pub async fn get_execution_logs(&self, request: &GetExecutionLogsRequestPB) -> FlowyResult<AgentExecutionLogListPB> {
    let session_key = if let Some(message_id) = &request.message_id {
      format!("{}_{}", request.session_id, message_id)
    } else {
      request.session_id.clone()
    };

    let logs = self.execution_logs
      .get(&session_key)
      .map(|entry| entry.value().clone())
      .unwrap_or_default();

    // 应用过滤器
    let mut filtered_logs = logs;
    if let Some(phase) = &request.phase {
      filtered_logs.retain(|log| log.phase == *phase);
    }

    // 应用分页
    let offset = request.offset as usize;
    let limit = request.limit as usize;
    let total = filtered_logs.len();
    
    let paginated_logs = if offset < total {
      let end = std::cmp::min(offset + limit, total);
      filtered_logs[offset..end].to_vec()
    } else {
      vec![]
    };

    let has_more = offset + limit < total;

    Ok(AgentExecutionLogListPB {
      logs: paginated_logs,
      has_more,
      total: total as i64,
    })
  }

  /// 添加执行日志
  pub async fn add_execution_log(&self, log: AgentExecutionLogPB) -> FlowyResult<()> {
    let session_key = if !log.message_id.is_empty() {
      format!("{}_{}", log.session_id, log.message_id)
    } else {
      log.session_id.clone()
    };

    self.execution_logs
      .entry(session_key)
      .or_insert_with(Vec::new)
      .push(log);

    Ok(())
  }

  /// 清空执行日志
  pub async fn clear_execution_logs(&self, request: &ClearExecutionLogsRequestPB) -> FlowyResult<()> {
    if let Some(message_id) = &request.message_id {
      let session_key = format!("{}_{}", request.session_id, message_id);
      self.execution_logs.remove(&session_key);
    } else {
      // 清空整个会话的所有日志
      let keys_to_remove: Vec<String> = self.execution_logs
        .iter()
        .filter(|entry| entry.key().starts_with(&request.session_id))
        .map(|entry| entry.key().clone())
        .collect();
      
      for key in keys_to_remove {
        self.execution_logs.remove(&key);
      }
    }

    Ok(())
  }
}

async fn sync_chat_documents(
  user_service: Arc<dyn AIUserService>,
  external_service: Arc<dyn AIExternalService>,
  rag_ids: Vec<Uuid>,
) -> FlowyResult<()> {
  if rag_ids.is_empty() {
    return Ok(());
  }

  let uid = user_service.user_id()?;
  let conn = user_service.sqlite_connection(uid)?;
  let metadata_map = batch_select_collab_metadata(conn, &rag_ids)?;

  let user_service = user_service.clone();
  tokio::spawn(async move {
    if let Ok(workspace_id) = user_service.workspace_id() {
      if let Ok(metadatas) = external_service
        .sync_rag_documents(&workspace_id, rag_ids, metadata_map)
        .await
      {
        if let Ok(uid) = user_service.user_id() {
          if let Ok(conn) = user_service.sqlite_connection(uid) {
            batch_insert_collab_metadata(conn, &metadatas).unwrap();
          }
        }
      }
    }
  });

  Ok(())
}

async fn refresh_chat_setting(
  user_service: &Arc<dyn AIUserService>,
  cloud_service: &Arc<ChatServiceMiddleware>,
  store_preferences: &Arc<KVStorePreferences>,
  chat_id: &Uuid,
) -> FlowyResult<ChatSettings> {
  info!("[Chat] refresh chat:{} setting", chat_id);
  let workspace_id = user_service.workspace_id()?;
  let settings = cloud_service
    .get_chat_settings(&workspace_id, chat_id)
    .await?;

  if let Err(err) = store_preferences.set_object(&setting_store_key(chat_id), &settings) {
    error!("failed to set chat settings: {}", err);
  }

  chat_notification_builder(chat_id.to_string(), ChatNotification::DidUpdateChatSettings)
    .payload(ChatSettingsPB {
      rag_ids: settings.rag_ids.clone(),
    })
    .send();

  Ok(settings)
}

fn setting_store_key(chat_id: &Uuid) -> String {
  format!("chat_settings_{}", chat_id)
}

const CUSTOM_PROMPT_DATABASE_CONFIGURATION_KEY: &str = "custom_prompt_database_config";

impl AIManager {
  /// 从已配置的 MCP 服务器动态发现所有可用工具
  async fn discover_available_tools(&self) -> (Vec<String>, HashMap<String, crate::mcp::entities::MCPTool>) {
    let mut tool_names = Vec::new();
    let mut tool_details = HashMap::new();
    
    // 🔍 关键修复：从配置管理器获取所有已配置的服务器，而不是只查询已连接的客户端池
    let server_configs = self.mcp_manager.config_manager().get_all_servers();
    let config_count = server_configs.len();
    
    info!("[Tool Discovery] 开始扫描 {} 个已配置的 MCP 服务器...", config_count);
    
    if server_configs.is_empty() {
      info!("[Tool Discovery] 未找到任何已配置的 MCP 服务器");
      return (tool_names, tool_details);
    }
    
    // 遍历所有已配置且活跃的服务器
    for config in server_configs {
      info!("[Tool Discovery] 检查配置: {} (ID: {}, 激活: {})", 
            config.name, config.id, config.is_active);
      
      // 跳过未激活的服务器
      if !config.is_active {
        info!("[Tool Discovery] 跳过未激活的服务器: {}", config.name);
        continue;
      }
      
      // 优先使用缓存的工具列表（避免重复连接）
      if let Some(cached_tools) = &config.cached_tools {
        let tool_count = cached_tools.len();
        info!("[Tool Discovery] 从服务器 '{}' 的缓存中发现 {} 个工具", config.name, tool_count);
        
        for tool in cached_tools {
          tool_names.push(tool.name.clone());
          tool_details.insert(tool.name.clone(), tool.clone());
        }
        continue;
      }
      
      // 如果没有缓存，尝试从已连接的客户端获取
      info!("[Tool Discovery] 服务器 '{}' 没有缓存，尝试从客户端获取...", config.name);
      match self.mcp_manager.tool_list(&config.id).await {
        Ok(tools_list) => {
          let tool_count = tools_list.tools.len();
          if tool_count > 0 {
            info!("[Tool Discovery] 从服务器 '{}' 的客户端获取到 {} 个工具", config.name, tool_count);
            for tool in tools_list.tools {
              tool_names.push(tool.name.clone());
              tool_details.insert(tool.name.clone(), tool);
            }
          } else {
            warn!("[Tool Discovery] 服务器 '{}' 已激活但未返回任何工具", config.name);
          }
        }
        Err(e) => {
          warn!("[Tool Discovery] 从服务器 '{}' 获取工具列表失败: {} - 可能未连接", config.name, e);
        }
      }
    }
    
    info!("✅ [Tool Discovery] 共从 {} 个已配置服务器发现 {} 个可用工具", 
          config_count, tool_names.len());
    (tool_names, tool_details)
  }
}
