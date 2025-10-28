use crate::chat::Chat;
use crate::entities::ToolDefinitionPB;
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
#[cfg(feature = "mcp")]
use crate::mcp::manager::MCPClientManager;
use crate::agent::config_manager::AgentConfigManager;
#[cfg(feature = "web-search")]
use crate::web_search::hub::WebSearchHub;
use crate::vector_index_manager::VectorIndexManager;
use crate::rag::RAGConfigManager;
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
  #[cfg(feature = "mcp")]
  pub mcp_manager: Arc<MCPClientManager>,
  pub agent_manager: Arc<AgentConfigManager>,
  execution_logs: Arc<DashMap<String, Vec<AgentExecutionLogPB>>>,
  #[cfg(feature = "web-search")]
  pub web_search_hub: Arc<WebSearchHub>,
  pub vector_index_manager: Arc<VectorIndexManager>,
  pub rag_config_manager: Arc<crate::rag::RAGConfigManager>,
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
    folder_service: Arc<dyn flowy_folder_pub::query::FolderService>,
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

    #[cfg(feature = "mcp")]
    let mcp_manager = Arc::new(MCPClientManager::new(store_preferences.clone()));
    let agent_manager = Arc::new(AgentConfigManager::new(store_preferences.clone()));
    #[cfg(feature = "web-search")]
    let web_search_hub = Arc::new(WebSearchHub::new(store_preferences.clone()));
    
    // 初始化向量索引管理器
    let vector_index_manager = Arc::new(VectorIndexManager::new(
      folder_service,
      user_service.clone(),
    ));
    
    // 初始化RAG配置管理器
    let rag_config_manager = Arc::new(RAGConfigManager::new(store_preferences.clone()));

    Self {
      cloud_service_wm,
      user_service,
      chats: Arc::new(DashMap::new()),
      local_ai,
      external_service,
      store_preferences,
      model_control: Mutex::new(model_control),
      #[cfg(feature = "mcp")]
      mcp_manager,
      agent_manager,
      execution_logs: Arc::new(DashMap::new()),
      #[cfg(feature = "web-search")]
      web_search_hub,
      vector_index_manager,
      rag_config_manager,
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
    
    // 🔧 加载并应用 RAG 配置管理器
    use crate::embeddings::context::EmbedContext;
    EmbedContext::shared().set_rag_config(Some(self.rag_config_manager.clone()));
    
    // 🔧 如果 scheduler 已经存在，需要重新创建以应用 RAG 配置
    // 这确保了新的 scheduler 会使用 RAG 配置
    if let Ok(_scheduler) = EmbedContext::shared().get_scheduler() {
      info!("[AI Manager] 🔄 Scheduler 已存在，重新创建以应用 RAG 配置...");
      EmbedContext::shared().try_recreate_scheduler();
    }
    
    // 🔧 加载并应用 OpenAI 兼容的嵌入服务配置
    self.load_and_apply_embedding_config(workspace_id).await;
    
    Ok(())
  }
  
  /// 重新加载嵌入服务配置（公共方法，可在外部调用）
  pub async fn reload_embedding_config(&self, workspace_id: &Uuid) {
    self.load_and_apply_embedding_config(workspace_id).await;
  }

  /// 加载并应用嵌入服务配置
  async fn load_and_apply_embedding_config(&self, workspace_id: &Uuid) {
    use crate::embeddings::context::EmbedContext;
    use crate::embeddings::scheduler::OpenAIEmbeddingConfig;
    
    trace!("[Embedding] 🔍 开始加载嵌入服务配置 for workspace: {}", workspace_id);
    
    // 尝试从 store_preferences 读取 OpenAI 兼容嵌入服务配置
    if let Some(settings_json) = self.store_preferences.get_str("appearance_settings") {
      trace!("[Embedding] 📄 找到 appearance_settings 配置");
      if let Ok(v) = serde_json::from_str::<serde_json::Value>(&settings_json) {
        trace!("[Embedding] ✅ 成功解析 JSON");
        let map = v
          .get("setting_key_value")
          .or_else(|| v.get("settingKeyValue"))
          .and_then(|v| v.as_object());
        
        if let Some(map) = map {
          trace!("[Embedding] 🗂️ 找到 setting_key_value map，键数量: {}", map.len());
          trace!("[Embedding] 🔑 所有键名: {:?}", map.keys().collect::<Vec<_>>());
          let scoped = |k: &str| -> String { format!("{}.{}", k, workspace_id) };
          let get = |k: &str| -> Option<String> {
            let scoped_key = scoped(k);
            let result = map.get(&scoped_key)
              .and_then(|v| v.as_str().map(|s| s.to_string()))
              .or_else(|| map.get(k).and_then(|v| v.as_str().map(|s| s.to_string())));
            trace!("[Embedding] 🔑 尝试读取键 '{}' / '{}': {:?}", scoped_key, k, result.as_ref().map(|s| if s.len() > 20 { format!("{}...", &s[..20]) } else { s.clone() }));
            result
          };
          
          // 读取嵌入服务配置
          // 优先使用 embedBaseUrl，如果没有则回退到 baseUrl 或 chatBaseUrl
          let embedding_base_url = get("ai.openai.embedBaseUrl")
            .or_else(|| get("ai.openai.baseUrl"))
            .or_else(|| get("ai.openai.chatBaseUrl"));
          
          if let Some(embedding_base_url) = embedding_base_url {
            let api_key = get("ai.openai.apiKey").unwrap_or_default();
            let model = get("ai.openai.embeddingModel")
              .unwrap_or_else(|| "text-embedding-3-small".to_string());
            
            // 读取 RAG 相似度阈值配置，默认 0.25
            let rag_score_threshold = get("ai.openai.ragScoreThreshold")
              .and_then(|s| s.parse::<f32>().ok())
              .unwrap_or(0.25)
              .clamp(0.0, 1.0);  // 确保在 0-1 范围内
            
            if !embedding_base_url.is_empty() && !api_key.is_empty() {
              info!(
                "[Embedding] 🔧 检测到 OpenAI 兼容嵌入服务配置: {} (模型: {}, RAG阈值: {:.2})",
                embedding_base_url, model, rag_score_threshold
              );
              
              let config = OpenAIEmbeddingConfig {
                base_url: embedding_base_url,
                api_key,
                model,
                rag_score_threshold,
              };
              
              EmbedContext::shared().set_openai_embedding_config(Some(config.clone()));
              
              // 🔧 如果 scheduler 已经创建，立即设置配置
              // 添加重试机制，因为 scheduler 可能还在初始化中
              for attempt in 1..=10 {
                if let Ok(scheduler) = EmbedContext::shared().get_scheduler() {
                  scheduler.set_openai_config(Some(config));
                  info!("[Embedding] ✅ OpenAI 嵌入服务配置已直接设置到 scheduler (尝试 {})", attempt);
                  return;
                } else {
                  if attempt < 10 {
                    info!("[Embedding] ⏳ Scheduler 尚未就绪，等待重试 ({}/10)", attempt);
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                  } else {
                    warn!("[Embedding] ⚠️ 经过 10 次重试，scheduler 仍未就绪，配置将在 scheduler 创建后自动应用");
                  }
                }
              }
            } else {
              warn!("[Embedding] ⚠️ OpenAI 嵌入服务配置不完整: base_url={}, api_key_len={}", 
                    embedding_base_url, api_key.len());
            }
          } else {
            trace!("[Embedding] ℹ️ 未找到 ai.openai.embedBaseUrl 配置");
          }
        } else {
          warn!("[Embedding] ⚠️ 未找到 setting_key_value 或 settingKeyValue 字段");
        }
      } else {
        warn!("[Embedding] ⚠️ 无法解析 appearance_settings JSON");
      }
    } else {
      warn!("[Embedding] ⚠️ 未找到 appearance_settings 配置");
    }
    
    // 如果没有 OpenAI 配置或配置无效，清除配置（使用 Ollama）
    trace!("[Embedding] 🔧 使用默认 Ollama 嵌入服务");
    EmbedContext::shared().set_openai_embedding_config(None);
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
    trace!("[Chat] create chat with rag_ids: {:?}", rag_ids);

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
          trace!("[Chat] Using agent: {} ({})", config.name, config.id);
          trace!("[Chat] Agent has {} tools, tool_calling enabled: {}", 
                config.available_tools.len(), config.capabilities.enable_tool_calling);
          
          // 🔍 获取工具详情用于增强系统提示（仅在启用工具调用时）
          let tool_details = if config.capabilities.enable_tool_calling {
            self.discover_tools_from_selected_servers(&config.selected_mcp_servers).await
          } else {
            Vec::new()
          };
          trace!("[Chat] 🔍 Discovered {} tools from MCP servers", tool_details.len());
          
          // 打印每个发现的工具及其来源服务器
          #[cfg(feature = "mcp")]
          for (server_id, tool) in &tool_details {
            // trace!("[Chat] 🔍   - Tool '{}' from server '{}'", tool.name, server_id);
          }
          
          // 收集工具名称（去重）
          use std::collections::HashSet;
          let mut unique_tool_names: HashSet<String> = HashSet::new();
          #[cfg(feature = "mcp")]
          let discovered_tool_names: Vec<String> = tool_details.iter()
            .filter_map(|(_, tool)| {
              if unique_tool_names.insert(tool.name.clone()) {
                Some(tool.name.clone())
              } else {
                None
              }
            })
            .collect();
          #[cfg(not(feature = "mcp"))]
          let discovered_tool_names: Vec<String> = vec![];
          
          trace!("[Chat] 🔍 Collected {} unique tool names", discovered_tool_names.len());
          
          // 🆕 确保内置工具总是被包含（无论工具列表是否为空）
          if config.capabilities.enable_tool_calling {
            let mut needs_update = false;
            let mut all_tools = config.available_tools.clone();
            
            // 添加内置网络搜索工具（如果不存在）
            #[cfg(feature = "web-search")]
            {
              let web_search_tools = vec![
                "web_search".to_string(),
                // "quick_search".to_string(), // 已注释以避免重复调用
              ];
              for tool in web_search_tools {
                if !all_tools.contains(&tool) {
                  all_tools.push(tool.clone());
                  needs_update = true;
                  trace!("[Chat] ✅ 添加内置网络搜索工具: {}", tool);
                }
              }
            }
            
            // 添加其他内置工具（如果不存在）
            let other_builtin_tools = vec![
              "search_documents".to_string(),
              "create_document".to_string(),
              "update_document".to_string(),
              "delete_document".to_string(),
            ];
            for tool in other_builtin_tools {
              if !all_tools.contains(&tool) {
                all_tools.push(tool.clone());
                needs_update = true;
                trace!("[Chat] ✅ 添加内置工具: {}", tool);
              }
            }
            
            // 如果工具列表为空，添加发现的 MCP 工具
            if config.available_tools.is_empty() {
              trace!("[Chat] 智能体工具列表为空，开始自动发现工具...");
              all_tools.extend(discovered_tool_names.clone());
              needs_update = true;
            }
            
            if needs_update {
              config.available_tools = all_tools;
              config.updated_at = chrono::Utc::now().timestamp();
              trace!("[Chat] ✅ 已将 {} 个工具添加到智能体配置（包含内置工具）", config.available_tools.len());
              
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
                selected_mcp_servers: config.selected_mcp_servers.clone(),
                has_available_tools: true,
              };
              
              if let Err(e) = self.agent_manager.update_agent(update_request) {
                warn!("Failed to save agent config after tool population: {}", e);
              } else {
                info!("为智能体 {} 自动发现并填充了 {} 个工具", 
                      config.name, config.available_tools.len());
              }
            }
          }
          
          // 🆕 构建增强的系统提示（包含工具详情）
          let enhanced_prompt = if !tool_details.is_empty() && config.capabilities.enable_tool_calling {
            #[cfg(feature = "mcp")]
            {
              use crate::agent::system_prompt::build_agent_system_prompt_with_tools;
              // 将 Vec<(String, MCPTool)> 转换为 HashMap<String, MCPTool>
              let tool_map: std::collections::HashMap<String, crate::mcp::entities::MCPTool> = tool_details.iter()
                .map(|(_server_id, tool)| (tool.name.clone(), tool.clone()))
                .collect();
              let prompt = build_agent_system_prompt_with_tools(&config, &tool_map);
              // trace!("[Chat] 🔧 Using enhanced system prompt with {} tool details", tool_map.len());
              Some(prompt)
            }
            #[cfg(not(feature = "mcp"))]
            {
              None
            }
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
      #[cfg(feature = "mcp")]
      {
        use crate::agent::ToolCallHandler;
        #[cfg(feature = "web-search")]
        {
          // 创建 WebSearchToolManager 并传递给 ToolCallHandler
          if let Ok(web_search_hub) = self.get_web_search_hub().await {
            let web_search_tool_manager = crate::web_search::WebSearchToolManager::with_hub(
              web_search_hub.clone(),
              self.store_preferences.clone(),
            );
            Some(Arc::new(
              ToolCallHandler::from_ai_manager(self)
                .with_web_search_tools(Arc::new(web_search_tool_manager))
            ))
          } else {
            warn!("[Chat] Failed to get web search hub, creating ToolCallHandler without web search tools");
            Some(Arc::new(ToolCallHandler::from_ai_manager(self)))
          }
        }
        #[cfg(not(feature = "web-search"))]
        {
          Some(Arc::new(ToolCallHandler::from_ai_manager(self)))
        }
      }
      #[cfg(not(feature = "mcp"))]
      {
        None
      }
    } else {
      None
    };

    // 📝 传递执行日志存储（始终传递，确保基本日志记录）
    let exec_logs = Some(self.execution_logs.clone());

    // 🆕 获取工具定义列表（用于 OpenAI Function Call API）
    let tool_definitions = if let Some(ref config) = agent_config {
      // trace!("[Chat] 🔧 Agent config found: {} ({}), enable_tool_calling: {}, available_tools count: {}", 
      //       config.name, config.id, config.capabilities.enable_tool_calling, config.available_tools.len());
      // trace!("[Chat] 🔧 Available tools list: {:?}", config.available_tools);
      
      if config.capabilities.enable_tool_calling && !config.available_tools.is_empty() {
        let tools = self.get_tool_definitions_by_names(&config.available_tools).await;
        // trace!("[Chat] 🔧 Got {} tool definitions for OpenAI Function Call", tools.len());
        
        // 🔧 关键修复：过滤掉不可用的工具，只返回可用的工具
        let available_tools: Vec<ToolDefinitionPB> = tools.into_iter()
          .filter(|tool| tool.is_available)
          .collect();
        
        if available_tools.is_empty() {
          warn!("[Chat] 🔧 All tools are disabled, skipping tool definitions");
          None
        } else {
          info!("[Chat] 🔧 Using {} available tools ({} were disabled)", 
                available_tools.len(), config.available_tools.len() - available_tools.len());
          Some(available_tools)
        }
      } else {
        warn!("[Chat] 🔧 Tool calling disabled or no available tools: enable_tool_calling={}, available_tools_count={}", 
              config.capabilities.enable_tool_calling, config.available_tools.len());
        None
      }
    } else {
      warn!("[Chat] 🔧 No agent config provided, skipping tool definitions");
      None
    };

    let chat = self.get_or_create_chat_instance(&params.chat_id).await?;
    
    // 🔧 关键修复：在发送消息前，确保本地 AI chat 实例存在并同步最新的 RAG IDs
    // 这对于 RAG 功能和 OpenAI 兼容模式的文档检索都是必需的
    if self.local_ai.is_enabled() {
      let uid = self.user_service.user_id()?;
      let mut conn = self.user_service.sqlite_connection(uid)?;
      let rag_ids = self.get_rag_ids(&params.chat_id, &mut conn).await?;
      let workspace_id = self.user_service.workspace_id()?;
      let model = self.get_active_model(&params.chat_id.to_string()).await;
      let summary = select_chat_summary(&mut conn, &params.chat_id).unwrap_or_default();
      
      info!(
        "[RAG] 🔄 确保 LLMChat 实例存在: chat_id={}, rag_ids={:?}, model={}",
        params.chat_id, rag_ids, model.name
      );
      
      // 先确保 LLMChat 实例存在
      self.local_ai.open_chat(&workspace_id, &params.chat_id, &model.name, rag_ids.clone(), summary).await?;
      
      // 然后同步 RAG IDs（此时实例已存在）
      self.local_ai.set_rag_ids(&params.chat_id, &rag_ids).await;
      
      trace!("[RAG] ✅ LLMChat 实例已就绪并同步 RAG IDs");
    }
    
    let ai_model = self.get_active_model(&params.chat_id.to_string()).await;
    let question = chat.stream_chat_message(&params, ai_model, agent_config, tool_call_handler, enhanced_prompt, exec_logs, tool_definitions).await?;
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
    
    // 🔧 同样需要在重新生成回答时确保实例存在并同步 RAG IDs
    if self.local_ai.is_enabled() {
      let uid = self.user_service.user_id()?;
      let mut conn = self.user_service.sqlite_connection(uid)?;
      let rag_ids = self.get_rag_ids(chat_id, &mut conn).await?;
      let workspace_id = self.user_service.workspace_id()?;
      let active_model = self.get_active_model(&chat_id.to_string()).await;
      let summary = select_chat_summary(&mut conn, chat_id).unwrap_or_default();
      
      info!(
        "[RAG] 🔄 重新生成回答前确保实例存在: chat_id={}, rag_ids={:?}",
        chat_id, rag_ids
      );
      
      // 先确保 LLMChat 实例存在
      self.local_ai.open_chat(&workspace_id, chat_id, &active_model.name, rag_ids.clone(), summary).await?;
      
      // 然后同步 RAG IDs
      self.local_ai.set_rag_ids(chat_id, &rag_ids).await;
      
      trace!("[RAG] ✅ 重新生成：LLMChat 实例已就绪");
    }
    
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

  /// 重置向量数据库 - 清空所有嵌入数据
  /// 当嵌入模型维度发生变化时使用
  pub async fn reset_vector_database(&self) -> FlowyResult<()> {
    use crate::embeddings::context::EmbedContext;
    
    info!("[AI Manager] 🔄 开始重置向量数据库...");
    
    EmbedContext::shared()
      .reset_vector_database()
      .await?;
    
    info!("[AI Manager] ✅ 向量数据库重置完成");
    Ok(())
  }

  /// 智能重置向量数据库 - 根据当前嵌入模型维度重建
  /// 当嵌入模型维度发生根本性变化时使用
  pub async fn smart_reset_vector_database(&self) -> FlowyResult<()> {
    use crate::embeddings::context::EmbedContext;
    
    info!("[AI Manager] 🔄 开始智能重置向量数据库...");
    
    // 获取当前嵌入模型的维度
    let current_dimension = EmbedContext::shared()
      .get_current_embedding_dimension()?;
    
    info!("[AI Manager] 📏 当前嵌入模型维度: {}", current_dimension);
    
    // 重建向量数据库以匹配新维度
    EmbedContext::shared()
      .rebuild_vector_database(current_dimension)
      .await?;
    
    info!("[AI Manager] ✅ 向量数据库智能重置完成，新维度: {}", current_dimension);
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
    info!(
      "[RAG] 📥 尝试获取 chat {} 的 RAG IDs",
      chat_id
    );
    
    match select_chat_rag_ids(&mut *conn, &chat_id.to_string()) {
      Ok(ids) => {
        info!(
          "[RAG] ✅ 从数据库成功获取 RAG IDs: {:?}",
          ids
        );
        return Ok(ids);
      },
      Err(err) => {
        // we no long use store_preferences to store chat settings
        warn!(
          "[RAG] ⚠️ 从数据库获取 RAG IDs 失败: {}，尝试从 store_preferences 获取",
          err
        );
        if let Some(settings) = self
          .store_preferences
          .get_object::<ChatSettings>(&setting_store_key(chat_id))
        {
          info!(
            "[RAG] 📦 从 store_preferences 获取到 RAG IDs: {:?}",
            settings.rag_ids
          );
          return Ok(settings.rag_ids);
        }
      },
    }

    trace!("[RAG] 🔄 从云端刷新 chat settings");
    let settings = refresh_chat_setting(
      &self.user_service,
      &self.cloud_service_wm,
      &self.store_preferences,
      chat_id,
    )
    .await?;
    info!(
      "[RAG] 🌐 从云端获取到 RAG IDs: {:?}",
      settings.rag_ids
    );
    Ok(settings.rag_ids)
  }

  pub async fn update_rag_ids(&self, chat_id: &Uuid, rag_ids: Vec<String>) -> FlowyResult<()> {
    info!(
      "[RAG] 🔄 更新 chat {} 的 RAG IDs: {:?}",
      chat_id, rag_ids
    );
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
    info!(
      "[RAG] ✅ RAG IDs 已同步到 local_ai，准备同步文档"
    );

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
    // 🆕 优先处理：如果指定了 MCP 服务器列表，从这些服务器获取工具
    if !request.selected_mcp_servers.is_empty() && request.capabilities.enable_tool_calling {
      info!("🆕 [Create Agent] 使用已选择的 {} 个 MCP 服务器", request.selected_mcp_servers.len());
      let tools_from_servers = self.get_tools_from_selected_servers(&request.selected_mcp_servers).await;
      
      if !tools_from_servers.is_empty() {
        info!("🆕 [Create Agent] 从已选择的服务器获取到 {} 个工具", tools_from_servers.len());
        request.available_tools = tools_from_servers;
      } else {
        warn!("⚠️ [Create Agent] 已选择的服务器未返回任何工具");
      }
    }
    // 如果没有指定服务器，且工具列表为空，则自动发现所有工具
    else if request.available_tools.is_empty() && request.capabilities.enable_tool_calling {
      let tool_details = self.discover_available_tools().await;
      
      // 提取所有工具名称（去重）
      #[cfg(feature = "mcp")]
      let discovered_tool_names: Vec<String> = {
        use std::collections::HashSet;
        let mut unique_names = HashSet::new();
        tool_details.iter()
          .filter_map(|(_, tool)| {
            if unique_names.insert(tool.name.clone()) {
              Some(tool.name.clone())
            } else {
              None
            }
          })
          .collect()
      };
      #[cfg(not(feature = "mcp"))]
      let discovered_tool_names: Vec<String> = vec![];
      
      // 🆕 首先添加内置工具（包括网络搜索工具）
      let mut all_tools = Vec::new();
      
      // 添加内置网络搜索工具
      #[cfg(feature = "web-search")]
      {
        all_tools.extend(vec![
          "web_search".to_string(),
          "quick_search".to_string(),
        ]);
      }
      
      // 添加其他内置工具
      all_tools.extend(vec![
        "search_documents".to_string(),
        "create_document".to_string(),
        "update_document".to_string(),
        "delete_document".to_string(),
      ]);
      
      // 添加发现的 MCP 工具
      all_tools.extend(discovered_tool_names);
      
      if !all_tools.is_empty() {
        info!("为新智能体 '{}' 自动发现了 {} 个工具（包含内置工具）", request.name, all_tools.len());
        request.available_tools = all_tools;
      } else {
        warn!("未发现任何可用的工具，智能体 '{}' 将以空工具列表创建", request.name);
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
    info!("🔄 [Agent Update] 请求已选择服务器数量: {}", request.selected_mcp_servers.len());
    info!("🔄 [Agent Update] 请求是否包含 capabilities: {}", request.capabilities.is_some());
    info!("🔄 [Agent Update] 请求是否明确设置工具列表: {}", request.has_available_tools);
    
    if let Some(ref existing) = existing_config {
      info!("🔄 [Agent Update] 现有智能体: {}", existing.name);
      info!("🔄 [Agent Update] 现有工具列表长度: {}", existing.available_tools.len());
      info!("🔄 [Agent Update] 现有已选择服务器数量: {}", existing.selected_mcp_servers.len());
      info!("🔄 [Agent Update] 现有 enable_tool_calling: {}", existing.capabilities.enable_tool_calling);
    }

    // 🆕 核心逻辑：如果提供了 selected_mcp_servers，自动同步工具列表
    if !request.selected_mcp_servers.is_empty() {
      info!("🆕 [Agent Update] 检测到 MCP 服务器列表变更，自动同步工具列表");
      
      // 判断是否启用了工具调用
      let tool_calling_enabled = if let Some(ref caps) = request.capabilities {
        caps.enable_tool_calling
      } else if let Some(ref existing) = existing_config {
        existing.capabilities.enable_tool_calling
      } else {
        false
      };
      
      if tool_calling_enabled {
        let tools_from_servers = self.get_tools_from_selected_servers(&request.selected_mcp_servers).await;
        
        if !tools_from_servers.is_empty() {
          info!("🆕 [Agent Update] 从 {} 个已选择的服务器同步了 {} 个工具", 
                request.selected_mcp_servers.len(), tools_from_servers.len());
          request.available_tools = tools_from_servers;
        } else {
          warn!("⚠️ [Agent Update] 已选择的服务器未返回任何工具");
          request.available_tools = vec![];
        }
        request.has_available_tools = true;
      } else {
        info!("ℹ️ [Agent Update] 工具调用未启用，跳过工具同步");
      }
    }
    
    // 如果更新了能力配置，且启用了工具调用，但请求中的工具列表为空
    if let Some(ref capabilities) = request.capabilities {
      info!("🔄 [Agent Update] 新能力配置 - enable_tool_calling: {}", capabilities.enable_tool_calling);
      
      if capabilities.enable_tool_calling && request.available_tools.is_empty() && !request.has_available_tools {
        info!("🔄 [Agent Update] 条件满足：工具调用已启用且工具列表为空");
        
        if let Some(existing) = existing_config {
          let should_discover = existing.available_tools.is_empty() || 
                                capabilities.enable_tool_calling != existing.capabilities.enable_tool_calling;
          
          info!("🔄 [Agent Update] 是否需要发现工具: {}", should_discover);
          
          if should_discover {
            info!("✨ [Agent Update] 检测到工具调用能力变更或工具列表为空，开始自动发现工具...");
            let tool_details = self.discover_available_tools().await;
            
            // 提取所有工具名称（支持同名工具）
            #[cfg(feature = "mcp")]
            let discovered_tool_names: Vec<String> = {
              use std::collections::HashSet;
              let mut unique_names = HashSet::new();
              tool_details.iter()
                .filter_map(|(_, tool)| {
                  if unique_names.insert(tool.name.clone()) {
                    Some(tool.name.clone())
                  } else {
                    None
                  }
                })
                .collect()
            };
            #[cfg(not(feature = "mcp"))]
            let discovered_tool_names: Vec<String> = vec![];
            
            // 🆕 首先添加内置工具（包括网络搜索工具）
            let mut all_tools = Vec::new();
            
            // 添加内置网络搜索工具
            #[cfg(feature = "web-search")]
            {
              all_tools.extend(vec![
                "web_search".to_string(),
                "quick_search".to_string(),
              ]);
            }
            
            // 添加其他内置工具
            all_tools.extend(vec![
              "search_documents".to_string(),
              "create_document".to_string(),
              "update_document".to_string(),
              "delete_document".to_string(),
            ]);
            
            // 添加发现的 MCP 工具
            all_tools.extend(discovered_tool_names);
            
            if !all_tools.is_empty() {
              info!("✅ [Agent Update] 为智能体 '{}' 自动发现了 {} 个工具（包含内置工具）", 
                    existing.name, all_tools.len());
              request.available_tools = all_tools;
              request.has_available_tools = true;
            } else {
              warn!("⚠️  [Agent Update] 未发现任何可用的工具");
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

  /// 获取网络搜索中心
  #[cfg(feature = "web-search")]
  pub async fn get_web_search_hub(&self) -> FlowyResult<Arc<WebSearchHub>> {
    Ok(self.web_search_hub.clone())
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
    let logs = if let Some(message_id) = &request.message_id {
      // 首先尝试精确匹配
      let session_key = format!("{}_{}", request.session_id, message_id);
      
      if let Some(entry) = self.execution_logs.get(&session_key) {
        entry.value().clone()
      } else {
        // 如果内存中没有，尝试从数据库恢复
        if let Ok(chat_id) = Uuid::parse_str(&request.session_id) {
          if let Ok(msg_id) = message_id.parse::<i64>() {
            match self.restore_execution_logs_from_db(&chat_id, msg_id).await {
              Ok(Some(logs)) if !logs.is_empty() => {
                // 缓存到内存中
                self.execution_logs.insert(session_key.clone(), logs.clone());
                logs
              }
              _ => Vec::new()
            }
          } else {
            Vec::new()
          }
        } else {
          Vec::new()
        }
      }
    } else {
      // 查询会话中所有消息的日志
      let session_prefix = format!("{}_", request.session_id);
      let mut all_logs = Vec::new();
      
      for entry in self.execution_logs.iter() {
        if entry.key().starts_with(&session_prefix) {
          all_logs.extend(entry.value().clone());
        }
      }
      
      // 按开始时间排序
      all_logs.sort_by_key(|log| log.started_at);
      all_logs
    };

    // 应用过滤器
    let mut filtered_logs = logs;
    if let Some(phase) = &request.phase {
      filtered_logs.retain(|log| log.phase == *phase);
    }
    if let Some(status) = &request.status {
      filtered_logs.retain(|log| log.status == *status);
    }
    
    // 应用搜索过滤
    if let Some(query) = &request.search_query {
      if !query.is_empty() {
        let query_lower = query.to_lowercase();
        filtered_logs.retain(|log| {
          log.step.to_lowercase().contains(&query_lower) ||
          log.input.to_lowercase().contains(&query_lower) ||
          log.output.to_lowercase().contains(&query_lower) ||
          log.error_message.as_ref().map(|e| e.to_lowercase().contains(&query_lower)).unwrap_or(false)
        });
      }
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
  
  /// 🔧 从本地数据库恢复执行日志
  async fn restore_execution_logs_from_db(
    &self,
    chat_id: &Uuid,
    message_id: i64,
  ) -> FlowyResult<Option<Vec<AgentExecutionLogPB>>> {
    // 使用 UserService 获取 SQLite 连接，直接查询本地数据库
    let uid = self.user_service.user_id()?;
    let conn = self.user_service.sqlite_connection(uid)?;
    
    // execution_logs 存储在 AI 回答消息中，需要根据 question_id 查找对应的 answer message
    use flowy_ai_pub::persistence::select_answer_where_match_reply_message_id;
    match select_answer_where_match_reply_message_id(conn, &chat_id.to_string(), message_id) {
      Ok(Some(message_table)) => {
        // 解析 metadata
        if let Some(metadata_str) = message_table.metadata {
          if let Ok(metadata_value) = serde_json::from_str::<serde_json::Value>(&metadata_str) {
            // 尝试从 metadata 中提取 execution_logs
            if let Some(execution_logs_value) = metadata_value.get("execution_logs") {
              match serde_json::from_value::<Vec<AgentExecutionLogPB>>(execution_logs_value.clone()) {
                Ok(logs) => {
                  info!("📋 Successfully restored {} execution logs from database", logs.len());
                  return Ok(Some(logs));
                }
                Err(e) => {
                  error!("Failed to deserialize execution_logs: {}", e);
                }
              }
            }
          }
        }
        Ok(None)
      }
      Ok(None) => Ok(None),
      Err(e) => Err(FlowyError::from(e))
    }
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
  trace!("[Chat] refresh chat:{} setting", chat_id);
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
  /// 根据工具名称列表获取工具定义
  pub async fn get_tool_definitions_by_names(&self, _tool_names: &[String]) -> Vec<ToolDefinitionPB> {
    let mut result: Vec<ToolDefinitionPB> = Vec::new();
    
    // 🆕 首先从内置工具管理器获取工具定义（包括网络搜索工具）
    #[cfg(feature = "web-search")]
    {
      // 使用现有的网络搜索中心实例
      if let Ok(web_search_hub) = self.get_web_search_hub().await {
        let web_search_tool_manager = crate::web_search::WebSearchToolManager::with_hub(
          web_search_hub.clone(),
          self.store_preferences.clone(),
        );
        let web_search_tools = web_search_tool_manager.get_tool_definitions();
        info!("[Tool Def] WebSearchToolManager returned {} tool definitions", web_search_tools.len());
        for tool_def in &web_search_tools {
          info!("[Tool Def] Web search tool: '{}' (available: {})", tool_def.name, tool_def.is_available);
        }
        
        for tool_name in _tool_names {
          let mut found = false;
          
          // 在网络搜索工具中查找
          for tool_def in &web_search_tools {
            if tool_def.name == *tool_name {
              if tool_def.is_available {
                result.push(tool_def.clone());
                found = true;
                info!("[Tool Def] ✅ Added web search tool '{}' to definitions", tool_name);
              } else {
                info!("[Tool Def] ⚠️ Web search tool '{}' found but not available", tool_name);
              }
              break;
            }
          }
          
          if !found {
            // info!("[Tool Def] ❌ Tool '{}' not found in web search tools, checking MCP servers", tool_name);
          }
        }
      } else {
        warn!("[Tool Def] Failed to get web search hub, skipping web search tools");
      }
    }
    
    #[cfg(not(feature = "web-search"))]
    {
      info!("[Tool Def] Web search feature not enabled, skipping web search tools");
    }
    
    // 然后从 MCP 服务器获取工具（仅在需要时）
    #[cfg(feature = "mcp")]
    {
      // 只有在有工具名称需要查找时才进行工具发现
      if !_tool_names.is_empty() {
        let tool_details = self.discover_available_tools().await;
        
        for tool_name in _tool_names {
        // 检查是否已经在结果中
        let already_added = result.iter().any(|def| def.name == *tool_name);
        if already_added {
          continue;
        }
        
        // 查找所有匹配的工具（可能来自不同服务器）
        let mut found = false;
        for (server_id, mcp_tool) in &tool_details {
          if &mcp_tool.name == tool_name {
            // Convert MCP tool to ToolDefinitionPB
            let tool_def = ToolDefinitionPB {
              name: mcp_tool.name.clone(),
              description: mcp_tool.description.clone().unwrap_or_default(),
              tool_type: crate::entities::ToolTypePB::MCP,
              source: server_id.clone(),  // 使用服务器ID作为source
              parameters_schema: serde_json::to_string(&mcp_tool.input_schema).unwrap_or_default(),
              permissions: Vec::new(),
              is_available: true,
              metadata: std::collections::HashMap::new(),
            };
            result.push(tool_def);
            found = true;
            info!("[Tool Def] Added tool '{}' from server '{}' to definitions", tool_name, server_id);
            // 对于同名工具，我们只添加第一个找到的（优先级由服务器顺序决定）
            break;
          }
        }
        
        if !found {
          warn!("[Tool Def] Tool '{}' not found in any MCP server", tool_name);
        }
        }
      }
    }
    
    info!("[Tool Def] Got {} tool definitions for OpenAI Function Call", result.len());
    result
  }

  /// 从已配置的 MCP 服务器动态发现所有可用工具
  /// 🔧 修复：返回 Vec<(server_id, tool)> 而不是 HashMap，以支持多个服务器提供同名工具
  #[cfg(feature = "mcp")]
  async fn discover_available_tools(&self) -> Vec<(String, crate::mcp::entities::MCPTool)> {
    let mut tool_details = Vec::new();
    
    // 🔍 关键修复：从配置管理器获取所有已配置的服务器，而不是只查询已连接的客户端池
    let server_configs = self.mcp_manager.config_manager().get_all_servers();
    let config_count = server_configs.len();
    
    info!("[Tool Discovery] 开始扫描 {} 个已配置的 MCP 服务器...", config_count);
    
    if server_configs.is_empty() {
      info!("[Tool Discovery] 未找到任何已配置的 MCP 服务器");
      return tool_details;
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
          // info!("[Tool Discovery]   - 工具: {} (服务器: {})", tool.name, config.id);
          tool_details.push((config.id.clone(), tool.clone()));
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
              // info!("[Tool Discovery]   - 工具: {} (服务器: {})", tool.name, config.id);
              tool_details.push((config.id.clone(), tool));
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
    
    info!("✅ [Tool Discovery] 共从 {} 个已配置服务器发现 {} 个工具（包含所有同名工具）", 
          config_count, tool_details.len());
    
    // 打印所有发现的工具及其来源服务器
    for (server_id, tool) in &tool_details {
      // info!("  📦 工具 '{}' 来自服务器 '{}'", tool.name, server_id);
    }
    
    tool_details
  }
  
  #[cfg(not(feature = "mcp"))]
  async fn discover_available_tools(&self) -> Vec<(String, ())> {
    vec![]
  }

  /// 🆕 根据已选择的 MCP 服务器列表发现工具详情
  /// 这个方法只从智能体选择的服务器中发现工具，避免扫描所有服务器
  #[cfg(feature = "mcp")]
  async fn discover_tools_from_selected_servers(&self, selected_server_ids: &[String]) -> Vec<(String, crate::mcp::entities::MCPTool)> {
    let mut tool_details = Vec::new();
    
    if selected_server_ids.is_empty() {
      info!("[Tool Discovery] 智能体未选择任何MCP服务器，跳过工具发现");
      return tool_details;
    }
    
    info!("[Tool Discovery] 开始从 {} 个已选择的MCP服务器发现工具...", selected_server_ids.len());
    
    // 获取所有已配置的服务器
    let server_configs = self.mcp_manager.config_manager().get_all_servers();
    
    // 只处理智能体选择的服务器
    for server_id in selected_server_ids {
      if let Some(config) = server_configs.iter().find(|c| c.id == *server_id) {
        info!("[Tool Discovery] 检查已选择的服务器: {} (ID: {}, 激活: {})", 
              config.name, config.id, config.is_active);
        
        // 跳过未激活的服务器
        if !config.is_active {
          info!("[Tool Discovery] 跳过未激活的已选择服务器: {}", config.name);
          continue;
        }
        
        // 优先使用缓存的工具列表
        if let Some(cached_tools) = &config.cached_tools {
          let tool_count = cached_tools.len();
          info!("[Tool Discovery] 从已选择服务器 '{}' 的缓存中发现 {} 个工具", config.name, tool_count);
          
          for tool in cached_tools {
            tool_details.push((config.id.clone(), tool.clone()));
          }
          continue;
        }
        
        // 如果没有缓存，尝试从已连接的客户端获取
        info!("[Tool Discovery] 已选择服务器 '{}' 没有缓存，尝试从客户端获取...", config.name);
        match self.mcp_manager.tool_list(&config.id).await {
          Ok(tools_list) => {
            let tool_count = tools_list.tools.len();
            if tool_count > 0 {
              info!("[Tool Discovery] 从已选择服务器 '{}' 的客户端获取到 {} 个工具", config.name, tool_count);
              for tool in tools_list.tools {
                tool_details.push((config.id.clone(), tool));
              }
            } else {
              warn!("[Tool Discovery] 已选择服务器 '{}' 已激活但未返回任何工具", config.name);
            }
          }
          Err(e) => {
            warn!("[Tool Discovery] 从已选择服务器 '{}' 获取工具列表失败: {} - 可能未连接", config.name, e);
          }
        }
      } else {
        warn!("[Tool Discovery] 已选择的服务器 '{}' 在配置中未找到", server_id);
      }
    }
    
    info!("✅ [Tool Discovery] 共从 {} 个已选择服务器发现 {} 个工具", 
          selected_server_ids.len(), tool_details.len());
    
    tool_details
  }
  
  #[cfg(not(feature = "mcp"))]
  async fn discover_tools_from_selected_servers(&self, _selected_server_ids: &[String]) -> Vec<(String, ())> {
    vec![]
  }

  /// 🆕 根据已选择的 MCP 服务器列表获取工具名称
  /// 这个方法用于根据UI中勾选的服务器自动填充工具列表
  #[cfg(feature = "mcp")]
  async fn get_tools_from_selected_servers(&self, server_ids: &[String]) -> Vec<String> {
    use std::collections::HashSet;
    
    if server_ids.is_empty() {
      return vec![];
    }
    
    info!("[🔧 Selected Servers] 开始从 {} 个已选择的服务器获取工具", server_ids.len());
    
    let all_tool_details = self.discover_tools_from_selected_servers(server_ids).await;
    let mut unique_tool_names = HashSet::new();
    let mut tools = Vec::new();
    
    for server_id in server_ids {
      let server_tools: Vec<_> = all_tool_details.iter()
        .filter(|(sid, _)| sid == server_id)
        .collect();
      
      info!("[🔧 Selected Servers] 服务器 '{}' 提供了 {} 个工具", server_id, server_tools.len());
      
      for (_, tool) in server_tools {
        if unique_tool_names.insert(tool.name.clone()) {
          tools.push(tool.name.clone());
          info!("[🔧 Selected Servers]   - 添加工具: {}", tool.name);
        }
      }
    }
    
    info!("[🔧 Selected Servers] ✅ 从已选择的服务器共获取 {} 个工具", tools.len());
    tools
  }
  
  #[cfg(not(feature = "mcp"))]
  async fn get_tools_from_selected_servers(&self, _server_ids: &[String]) -> Vec<String> {
    vec![]
  }
}
