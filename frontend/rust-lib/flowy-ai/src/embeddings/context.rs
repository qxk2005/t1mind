use crate::embeddings::scheduler::{EmbeddingScheduler, OpenAIEmbeddingConfig};
use arc_swap::ArcSwapOption;
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite_vec::db::VectorSqliteDB;
use lib_infra::util::get_operating_system;
use ollama_rs::Ollama;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::{error, trace, warn};

pub struct EmbedContext {
  ollama: ArcSwapOption<Ollama>,
  vector_db: ArcSwapOption<VectorSqliteDB>,
  scheduler: ArcSwapOption<EmbeddingScheduler>,
}

impl EmbedContext {
  pub fn shared() -> &'static Arc<EmbedContext> {
    static INSTANCE: OnceLock<Arc<EmbedContext>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
      Arc::new(EmbedContext {
        ollama: ArcSwapOption::empty(),
        vector_db: ArcSwapOption::empty(),
        scheduler: Default::default(),
      })
    })
  }

  pub fn get_vector_db(&self) -> Option<Arc<VectorSqliteDB>> {
    self.vector_db.load_full()
  }

  pub fn init_vector_db(&self, db_path: PathBuf) {
    let sys = get_operating_system();
    if !sys.is_desktop() {
      warn!("[Embedding] Vector db is not supported on {:?}", sys);
      return;
    }

    trace!("Initializing vector db");
    match VectorSqliteDB::new(db_path.clone()) {
      Ok(db) => {
        trace!("[Embedding] Vector db created at: {:?}", db_path);
        self.vector_db.store(Some(Arc::new(db)));
        self.try_create_scheduler();
      },
      Err(err) => {
        error!("[Embedding] Failed to create vector db: {}", err);
      },
    }
  }

  pub fn set_ollama(&self, ollama: Option<Arc<Ollama>>) {
    if let Some(ollama) = ollama {
      if let Some(o) = self.ollama.load().as_ref() {
        if o.uri() == ollama.uri() {
          trace!("[Embedding] Ollama does not change");
          return;
        }
      }

      self.ollama.store(Some(ollama));
      self.try_create_scheduler();
    } else {
      self.ollama.store(None);
      if let Some(s) = self.scheduler.swap(None) {
        trace!("[Embedding] Stopping scheduler");
        let _ = s.stop_tx.send(());
      }
    }
  }

  pub fn get_scheduler(&self) -> FlowyResult<Arc<EmbeddingScheduler>> {
    self.scheduler.load_full().ok_or_else(|| {
      FlowyError::new(
        ErrorCode::LocalEmbeddingNotReady,
        "Local embedding is not ready. Please check if the Ollama and vector db are initialized.",
      )
    })
  }

  /// 设置 OpenAI 兼容嵌入服务配置
  pub fn set_openai_embedding_config(&self, config: Option<OpenAIEmbeddingConfig>) {
    if let Some(scheduler) = self.scheduler.load_full() {
      scheduler.set_openai_config(config);
    } else {
      warn!("[Embedding] Scheduler not initialized, OpenAI config not set");
    }
  }

  fn try_create_scheduler(&self) {
    if let (Some(ollama), Some(vector_db)) = (self.ollama.load_full(), self.vector_db.load_full()) {
      trace!("[Embedding] Creating scheduler");
      match EmbeddingScheduler::new(ollama, vector_db) {
        Ok(s) => {
          trace!("[Embedding] create scheduler successfully");
          self.scheduler.store(Some(s));
        },
        Err(err) => error!("[Embedding] Failed to create scheduler: {}", err),
      }
    } else {
      trace!("[Embedding] Ollama or vector db is not initialized, remove embedding scheduler");
      self.scheduler.store(None);
    }
  }
}
