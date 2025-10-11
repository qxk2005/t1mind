use crate::embeddings::context::EmbedContext;
use crate::notification::{ChatNotification, chat_notification_builder};
use collab::core::collab::DataSource;
use collab::core::origin::CollabOrigin;
use collab::preclude::Collab;
use collab_entity::CollabType;
use collab_integrate::instant_indexed_data_provider::unindexed_data_form_collab;
use flowy_ai_pub::entities::{UnindexedCollab, UnindexedCollabMetadata};
use flowy_error::{FlowyError, FlowyResult};
use flowy_folder_pub::query::FolderService;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorIndexState {
  Idle,
  Running,
  Completed,
  Failed,
  Stopping,
}

pub struct VectorIndexManager {
  folder_service: Arc<dyn FolderService>,
  state: Arc<RwLock<VectorIndexState>>,
  total_documents: Arc<AtomicU32>,
  indexed_documents: Arc<AtomicU32>,
  recent_logs: Arc<RwLock<Vec<String>>>,
  error: Arc<RwLock<Option<String>>>,
  start_time: Arc<AtomicI64>,
  last_update_time: Arc<AtomicI64>,
  should_stop: Arc<AtomicBool>,
}

impl VectorIndexManager {
  const MAX_LOGS: usize = 50; // 最多保留 50 条日志

  pub fn new(folder_service: Arc<dyn FolderService>) -> Self {
    Self {
      folder_service,
      state: Arc::new(RwLock::new(VectorIndexState::Idle)),
      total_documents: Arc::new(AtomicU32::new(0)),
      indexed_documents: Arc::new(AtomicU32::new(0)),
      recent_logs: Arc::new(RwLock::new(Vec::new())),
      error: Arc::new(RwLock::new(None)),
      start_time: Arc::new(AtomicI64::new(0)),
      last_update_time: Arc::new(AtomicI64::new(0)),
      should_stop: Arc::new(AtomicBool::new(false)),
    }
  }

  pub async fn get_state(&self) -> VectorIndexState {
    *self.state.read().await
  }

  pub fn get_total_documents(&self) -> u32 {
    self.total_documents.load(Ordering::Relaxed)
  }

  pub fn get_indexed_documents(&self) -> u32 {
    self.indexed_documents.load(Ordering::Relaxed)
  }

  pub async fn get_recent_logs(&self) -> Vec<String> {
    self.recent_logs.read().await.clone()
  }

  pub async fn get_error(&self) -> Option<String> {
    self.error.read().await.clone()
  }

  pub fn get_start_time(&self) -> i64 {
    self.start_time.load(Ordering::Relaxed)
  }

  pub fn get_last_update_time(&self) -> i64 {
    self.last_update_time.load(Ordering::Relaxed)
  }

  async fn add_log(&self, log: String) {
    let mut logs = self.recent_logs.write().await;
    logs.push(log);
    // 只保留最近的日志
    if logs.len() > Self::MAX_LOGS {
      let excess = logs.len() - Self::MAX_LOGS;
      logs.drain(0..excess);
    }
    
    // 更新时间戳
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;
    self.last_update_time.store(now, Ordering::Relaxed);
    
    // 发送通知
    self.send_status_notification().await;
  }

  async fn send_status_notification(&self) {
    // 发送状态更新通知到前端
    // 前端可以监听此通知来更新 UI
    chat_notification_builder("", ChatNotification::VectorIndexStatusUpdated)
      .send();
  }

  pub async fn stop_indexing(&self) {
    info!("[VectorIndex] 🛑 用户请求停止索引");
    self.should_stop.store(true, Ordering::Relaxed);
    *self.state.write().await = VectorIndexState::Stopping;
    self.add_log("用户请求停止索引...".to_string()).await;
  }

  pub async fn rebuild_index(
    &self,
    workspace_id: Uuid,
    document_ids: Option<Vec<String>>,
  ) -> FlowyResult<()> {
    // 检查是否已经在运行
    {
      let current_state = self.get_state().await;
      if current_state == VectorIndexState::Running {
        return Err(FlowyError::internal().with_context("索引重建已在进行中"));
      }
    }

    // 重置状态
    self.should_stop.store(false, Ordering::Relaxed);
    *self.state.write().await = VectorIndexState::Running;
    *self.error.write().await = None;
    self.indexed_documents.store(0, Ordering::Relaxed);
    self.recent_logs.write().await.clear();
    
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;
    self.start_time.store(now, Ordering::Relaxed);
    self.last_update_time.store(now, Ordering::Relaxed);

    info!("[VectorIndex] 🚀 开始重建向量索引");
    self.add_log("开始重建向量索引...".to_string()).await;

    // 克隆必要的数据以在 tokio::spawn 中使用
    let folder_service = self.folder_service.clone();
    let state = self.state.clone();
    let total_documents = self.total_documents.clone();
    let indexed_documents = self.indexed_documents.clone();
    let recent_logs = self.recent_logs.clone();
    let error = self.error.clone();
    let last_update_time = self.last_update_time.clone();
    let should_stop = self.should_stop.clone();

    // 在后台执行索引重建
    tokio::spawn(async move {
      let result = Self::rebuild_index_internal(
        workspace_id,
        document_ids,
        folder_service,
        total_documents.clone(),
        indexed_documents.clone(),
        recent_logs.clone(),
        last_update_time.clone(),
        should_stop.clone(),
      )
      .await;

      match result {
        Ok(_) => {
          if should_stop.load(Ordering::Relaxed) {
            *state.write().await = VectorIndexState::Idle;
            let mut logs = recent_logs.write().await;
            logs.push("索引重建已停止".to_string());
            info!("[VectorIndex] 🛑 索引重建已停止");
          } else {
            *state.write().await = VectorIndexState::Completed;
            let mut logs = recent_logs.write().await;
            logs.push(format!(
              "✅ 索引重建完成！共索引 {} 个文档",
              indexed_documents.load(Ordering::Relaxed)
            ));
            info!("[VectorIndex] ✅ 索引重建完成");
          }
        }
        Err(err) => {
          *state.write().await = VectorIndexState::Failed;
          *error.write().await = Some(err.to_string());
          let mut logs = recent_logs.write().await;
          logs.push(format!("❌ 索引重建失败: {}", err));
          error!("[VectorIndex] ❌ 索引重建失败: {}", err);
        }
      }

      // 发送最终状态通知
      chat_notification_builder("", ChatNotification::VectorIndexStatusUpdated)
        .send();
    });

    Ok(())
  }

  async fn rebuild_index_internal(
    workspace_id: Uuid,
    document_ids: Option<Vec<String>>,
    folder_service: Arc<dyn FolderService>,
    total_documents: Arc<AtomicU32>,
    indexed_documents: Arc<AtomicU32>,
    recent_logs: Arc<RwLock<Vec<String>>>,
    last_update_time: Arc<AtomicI64>,
    should_stop: Arc<AtomicBool>,
  ) -> FlowyResult<()> {
    // 获取嵌入调度器
    let scheduler = EmbedContext::shared().get_scheduler()?;

    // 获取要索引的文档列表
    let document_uuids: Vec<Uuid> = if let Some(ids) = document_ids {
      ids
        .into_iter()
        .filter_map(|id| Uuid::parse_str(&id).ok())
        .collect()
    } else {
      // 如果没有提供文档 ID，返回错误
      return Err(FlowyError::invalid_data().with_context(
        "请在 AI 聊天设置中选择要索引的文档（信息源），然后再重建索引"
      ));
    };

    total_documents.store(document_uuids.len() as u32, Ordering::Relaxed);

    {
      let mut logs = recent_logs.write().await;
      logs.push(format!("📝 找到 {} 个文档需要索引", document_uuids.len()));
    }

    info!(
      "[VectorIndex] 📝 开始索引 {} 个文档",
      document_uuids.len()
    );

    // 逐个索引文档
    for (index, document_id) in document_uuids.iter().enumerate() {
      // 检查是否需要停止
      if should_stop.load(Ordering::Relaxed) {
        info!("[VectorIndex] 🛑 检测到停止信号，停止索引");
        return Ok(());
      }

      {
        let mut logs = recent_logs.write().await;
        logs.push(format!(
          "🔍 [{}/{}] 正在索引文档: {}",
          index + 1,
          document_uuids.len(),
          document_id
        ));
      }

      // 获取文档的 Collab 数据
      let query_collab = match folder_service
        .get_collab(document_id, CollabType::Document)
        .await
      {
        Some(collab) => collab,
        None => {
          warn!("[VectorIndex] ⚠️ 文档 {} 不存在，跳过", document_id);
          let mut logs = recent_logs.write().await;
          logs.push(format!("⚠️ 文档 {} 不存在，跳过", document_id));
          continue;
        }
      };

      // 重建 Collab 对象
      let collab = match Collab::new_with_source(
        CollabOrigin::Empty,
        &document_id.to_string(),
        DataSource::DocStateV1(query_collab.encoded_collab.doc_state.to_vec()),
        vec![],
        false,
      ) {
        Ok(c) => c,
        Err(err) => {
          error!("[VectorIndex] ❌ 无法重建 Collab 对象 {}: {}", document_id, err);
          let mut logs = recent_logs.write().await;
          logs.push(format!("❌ 文档 {} 解析失败，跳过", document_id));
          continue;
        }
      };

      // 从 Collab 提取数据
      let data = unindexed_data_form_collab(&collab, &CollabType::Document);

      if data.is_none() {
        warn!("[VectorIndex] ⚠️ 文档 {} 没有可索引的内容", document_id);
        let mut logs = recent_logs.write().await;
        logs.push(format!("⚠️ 文档 {} 内容为空，跳过", document_id));
        continue;
      }

      let unindexed_collab = UnindexedCollab {
        workspace_id,
        object_id: *document_id,
        collab_type: CollabType::Document,
        data,
        metadata: UnindexedCollabMetadata::default(),
      };

      // 提交到索引队列
      if let Err(err) = scheduler.index_collab(unindexed_collab).await {
        error!("[VectorIndex] ❌ 索引文档 {} 失败: {}", document_id, err);
        let mut logs = recent_logs.write().await;
        logs.push(format!("❌ 文档 {} 索引失败: {}", document_id, err));
      } else {
        indexed_documents.fetch_add(1, Ordering::Relaxed);
        
        // 每索引 5 个文档记录一次日志
        if (index + 1) % 5 == 0 {
          let mut logs = recent_logs.write().await;
          logs.push(format!(
            "✅ 已索引 {}/{} 个文档",
            indexed_documents.load(Ordering::Relaxed),
            total_documents.load(Ordering::Relaxed)
          ));
        }
      }

      // 更新时间戳并发送通知
      let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
      last_update_time.store(now, Ordering::Relaxed);
      chat_notification_builder("", ChatNotification::VectorIndexStatusUpdated)
        .send();

      // 短暂延迟，避免过载
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("[VectorIndex] 🎉 所有文档已提交到索引队列");

    Ok(())
  }
}

