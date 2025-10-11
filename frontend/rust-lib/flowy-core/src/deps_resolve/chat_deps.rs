use collab::core::collab::DataSource;
use collab::core::origin::CollabOrigin;
use collab::preclude::updates::decoder::Decode;
use collab::preclude::{Collab, StateVector};
use collab::util::is_change_since_sv;
use collab_entity::CollabType;
use collab_integrate::instant_indexed_data_provider::unindexed_data_form_collab;
use flowy_ai::ai_manager::{AIExternalService, AIManager};
use flowy_ai::local_ai::chat::retriever::{LangchainDocument, MultipleSourceRetrieverStore};
use flowy_ai::local_ai::controller::LocalAIController;
use flowy_ai_pub::cloud::ChatCloudService;
use flowy_ai_pub::entities::{SOURCE, SOURCE_ID, SOURCE_NAME, UnindexedCollab, UnindexedCollabMetadata};
use flowy_ai_pub::persistence::AFCollabMetadata;
use flowy_ai_pub::user_service::AIUserService;
use flowy_error::{FlowyError, FlowyResult};
use flowy_folder::ViewLayout;
use collab::entity::EncodedCollab;
use flowy_folder_pub::cloud::{FolderCloudService, FullSyncCollabParams};
use flowy_folder_pub::query::FolderService;
use flowy_search_pub::tantivy_state::DocumentTantivyState;
use flowy_server::util::tanvity_local_search;
use flowy_sqlite::kv::KVStorePreferences;
use flowy_sqlite::DBConnection;
use flowy_storage_pub::storage::StorageService;
use flowy_user::services::authenticate_user::AuthenticateUser;
use flowy_user_pub::entities::WorkspaceType;
use lib_infra::async_trait::async_trait;
use lib_infra::util::timestamp;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct ChatDepsResolver;

impl ChatDepsResolver {
  pub fn resolve(
    authenticate_user: Weak<AuthenticateUser>,
    cloud_service: Arc<dyn ChatCloudService>,
    store_preferences: Arc<KVStorePreferences>,
    storage_service: Weak<dyn StorageService>,
    folder_cloud_service: Arc<dyn FolderCloudService>,
    folder_service: impl FolderService,
    local_ai: Arc<LocalAIController>,
  ) -> Arc<AIManager> {
    let user_service = ChatUserServiceImpl(authenticate_user);
    let folder_service_arc = Arc::new(folder_service);
    Arc::new(AIManager::new(
      cloud_service,
      user_service,
      store_preferences,
      storage_service,
      ChatQueryServiceImpl {
        folder_service: folder_service_arc.clone(),
        folder_cloud_service,
      },
      local_ai,
      folder_service_arc,
    ))
  }
}

struct ChatQueryServiceImpl {
  folder_service: Arc<dyn FolderService>,
  folder_cloud_service: Arc<dyn FolderCloudService>,
}

#[async_trait]
impl AIExternalService for ChatQueryServiceImpl {
  async fn query_chat_rag_ids(
    &self,
    parent_view_id: &Uuid,
    chat_id: &Uuid,
  ) -> Result<Vec<Uuid>, FlowyError> {
    let mut ids = self
      .folder_service
      .get_surrounding_view_ids_with_view_layout(parent_view_id, ViewLayout::Document)
      .await;

    if !ids.is_empty() {
      ids.retain(|id| id != chat_id);
    }

    Ok(ids)
  }
  async fn sync_rag_documents(
    &self,
    workspace_id: &Uuid,
    rag_ids: Vec<Uuid>,
    mut rag_metadata_map: HashMap<Uuid, AFCollabMetadata>,
  ) -> Result<Vec<AFCollabMetadata>, FlowyError> {
    let mut result = Vec::new();
    let mut documents_to_index = Vec::new(); // 收集需要索引的文档

    info!("[Embedding] sync rag documents: {:?}", rag_ids);
    for rag_id in rag_ids {
      // Retrieve the collab object for the current rag_id
      let query_collab = match self
        .folder_service
        .get_collab(&rag_id, CollabType::Document)
        .await
      {
        Some(collab) => collab,
        None => {
          debug!(
            "[Embedding] can not find collab data, skip sync rag document: {}",
            rag_id
          );
          continue;
        },
      };

      // Check if the state vector exists and detect changes
      let should_sync = if let Some(metadata) = rag_metadata_map.remove(&rag_id) {
        if let Ok(prev_sv) = StateVector::decode_v1(&metadata.prev_sync_state_vector) {
          if let Ok(collab) = Collab::new_with_source(
            CollabOrigin::Empty,
            &rag_id.to_string(),
            DataSource::DocStateV1(query_collab.encoded_collab.doc_state.to_vec()),
            vec![],
            false,
          ) {
            is_change_since_sv(&collab, &prev_sv)
          } else {
            true
          }
        } else {
          true
        }
      } else {
        true
      };

      if !should_sync {
        info!(
          "[Embedding] skip full sync rag document {}, no changes",
          rag_id
        );
        continue;
      }

      // Perform full sync if changes are detected or no state vector is found
      let params = FullSyncCollabParams {
        object_id: rag_id,
        collab_type: CollabType::Document,
        encoded_collab: query_collab.encoded_collab.clone(),
      };

      info!("[Embedding] full sync rag document: {}", params.object_id);
      if let Err(err) = self
        .folder_cloud_service
        .full_sync_collab_object(workspace_id, params)
        .await
      {
        error!(
          "[Embedding] failed to sync rag document: {} error: {}",
          rag_id, err
        );
      } else {
        result.push(AFCollabMetadata {
          object_id: rag_id.to_string(),
          updated_at: timestamp(),
          prev_sync_state_vector: query_collab.encoded_collab.state_vector.to_vec(),
          collab_type: CollabType::Document as i32,
        });
        
        // 收集成功同步的文档，稍后触发索引
        documents_to_index.push((rag_id, query_collab.encoded_collab.clone()));
      }
    }

    // 🔧 关键修复：同步完成后，触发本地向量索引
    if !documents_to_index.is_empty() {
      info!(
        "[Embedding] 📝 开始为 {} 个文档触发向量索引",
        documents_to_index.len()
      );
      trigger_vector_indexing(*workspace_id, documents_to_index).await;
    }

    Ok(result)
  }

  async fn notify_did_send_message(&self, chat_id: &Uuid, message: &str) -> Result<(), FlowyError> {
    // info!(
    //   "notify_did_send_message: chat_id: {}, message: {}",
    //   chat_id, message
    // );
    self
      .folder_service
      .set_view_title_if_empty(chat_id, message)
      .await?;
    Ok(())
  }
}

pub struct ChatUserServiceImpl(Weak<AuthenticateUser>);
impl ChatUserServiceImpl {
  fn upgrade_user(&self) -> Result<Arc<AuthenticateUser>, FlowyError> {
    let user = self
      .0
      .upgrade()
      .ok_or(FlowyError::internal().with_context("Unexpected error: UserSession is None"))?;
    Ok(user)
  }
}

#[async_trait]
impl AIUserService for ChatUserServiceImpl {
  fn user_id(&self) -> Result<i64, FlowyError> {
    self.upgrade_user()?.user_id()
  }

  async fn is_local_model(&self) -> FlowyResult<bool> {
    self.upgrade_user()?.is_local_mode().await
  }

  fn workspace_id(&self) -> Result<Uuid, FlowyError> {
    self.upgrade_user()?.workspace_id()
  }

  fn workspace_type(&self) -> FlowyResult<WorkspaceType> {
    self.upgrade_user()?.workspace_type()
  }

  fn sqlite_connection(&self, uid: i64) -> Result<DBConnection, FlowyError> {
    self.upgrade_user()?.get_sqlite_connection(uid)
  }

  fn application_root_dir(&self) -> Result<PathBuf, FlowyError> {
    Ok(PathBuf::from(
      self.upgrade_user()?.get_application_root_dir(),
    ))
  }
}

#[derive(Clone)]
pub struct MultiSourceVSTanvityImpl {
  state: Option<Weak<RwLock<DocumentTantivyState>>>,
}

impl MultiSourceVSTanvityImpl {
  pub fn new(state: Option<Weak<RwLock<DocumentTantivyState>>>) -> Self {
    Self { state }
  }
}

#[async_trait]
impl MultipleSourceRetrieverStore for MultiSourceVSTanvityImpl {
  fn retriever_name(&self) -> &'static str {
    "Tanvity Multiple Source Retriever"
  }

  async fn read_documents(
    &self,
    workspace_id: &Uuid,
    query: &str,
    limit: usize,
    rag_ids: &[String],
    score_threshold: f32,
    _full_search: bool,
  ) -> FlowyResult<Vec<LangchainDocument>> {
    let docs = tanvity_local_search(
      &self.state,
      workspace_id,
      query,
      Some(rag_ids.to_vec()),
      limit,
      score_threshold,
    )
    .await;

    match docs {
      None => Ok(vec![]),
      Some(docs) => Ok(
        docs
          .into_iter()
          .map(|v| LangchainDocument {
            page_content: v.content,
            metadata: json!({
                SOURCE_ID: v.object_id,
                SOURCE: "appflowy",
                SOURCE_NAME: "document",
            })
            .as_object()
            .unwrap()
            .clone()
            .into_iter()
            .collect(),
            score: v.score,
          })
          .collect(),
      ),
    }
  }
}

/// 触发文档向量索引
async fn trigger_vector_indexing(workspace_id: Uuid, documents: Vec<(Uuid, EncodedCollab)>) {
  use flowy_ai::embeddings::context::EmbedContext;
  
  // 获取嵌入调度器
  let scheduler = match EmbedContext::shared().get_scheduler() {
    Ok(s) => s,
    Err(err) => {
      warn!(
        "[Embedding] ⚠️ 无法获取调度器，跳过索引: {}",
        err
      );
      return;
    },
  };
  
  for (object_id, encoded_collab) in documents {
    info!("[Embedding] 🔍 准备索引文档: {}", object_id);
    
    // 从 EncodedCollab 重建 Collab 对象
    let collab = match Collab::new_with_source(
      CollabOrigin::Empty,
      &object_id.to_string(),
      DataSource::DocStateV1(encoded_collab.doc_state.to_vec()),
      vec![],
      false,
    ) {
      Ok(c) => c,
      Err(err) => {
        error!(
          "[Embedding] ❌ 无法重建 Collab 对象 {}: {}",
          object_id, err
        );
        continue;
      },
    };
    
    // 从 Collab 提取数据
    let data = unindexed_data_form_collab(&collab, &CollabType::Document);
    
    if data.is_none() {
      warn!(
        "[Embedding] ⚠️ 文档 {} 没有可索引的内容",
        object_id
      );
      continue;
    }
    
    let unindexed_collab = UnindexedCollab {
      workspace_id,
      object_id,
      collab_type: CollabType::Document,
      data,
      metadata: UnindexedCollabMetadata::default(),
    };
    
    if let Err(err) = scheduler.index_collab(unindexed_collab).await {
      error!(
        "[Embedding] ❌ 索引文档 {} 失败: {}",
        object_id, err
      );
    } else {
      info!(
        "[Embedding] ✅ 已提交文档 {} 到索引队列",
        object_id
      );
    }
  }
  
  info!("[Embedding] 🎉 所有文档已提交到索引队列");
}
