use crate::embeddings::scheduler::{EmbeddingScheduler, OpenAIEmbeddingConfig};
use arc_swap::ArcSwapOption;
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite_vec::db::VectorSqliteDB;
use lib_infra::util::get_operating_system;
use ollama_rs::Ollama;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::{error, info, trace, warn};

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
        ollama: ArcSwapOption::new(None),
        vector_db: ArcSwapOption::new(None),
        scheduler: ArcSwapOption::new(None),
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

    info!("[Embedding] 🔄 Initializing vector database at {:?}", db_path);
    match VectorSqliteDB::new(db_path.clone()) {
      Ok(db) => {
        info!("[Embedding] ✅ Vector database initialized successfully at: {:?}", db_path);
        self.vector_db.store(Some(Arc::new(db)));
        info!("[Embedding] 📦 Stored vector database instance, now creating scheduler...");
        self.try_create_scheduler();
      },
      Err(err) => {
        error!("[Embedding] ❌ Failed to create vector database: {}", err);
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
    // 🔧 使用 load_full() 来获取 scheduler
    let scheduler_opt = self.scheduler.load_full();
    trace!("[Embedding] 🔍 尝试获取 scheduler: {:?}", scheduler_opt.is_some());
    
    match scheduler_opt {
      Some(scheduler) => {
        trace!("[Embedding] ✅ 成功获取 scheduler 实例");
        Ok(scheduler)
      },
      None => {
        // 🔧 检查是否可以创建 scheduler
        let vector_db_available = self.vector_db.load_full().is_some();
        let ollama_available = self.ollama.load_full().is_some();
        
        if vector_db_available {
          // 如果 vector_db 可用，尝试重新创建 scheduler
          warn!("[Embedding] ⚠️ Scheduler 未初始化但 vector_db 可用，尝试重新创建...");
          self.try_create_scheduler();
          
          // 再次尝试获取
          if let Some(scheduler) = self.scheduler.load_full() {
            info!("[Embedding] ✅ 重新创建 scheduler 成功");
            return Ok(scheduler);
          }
        }
        
        warn!("[Embedding] ❌ Scheduler 未初始化，无法获取");
        warn!("[Embedding] 🔍 调试信息: vector_db={}, ollama={}", vector_db_available, ollama_available);
        
        Err(FlowyError::new(
          ErrorCode::LocalEmbeddingNotReady,
          "向量索引服务未就绪。请确保已配置嵌入服务（OpenAI 兼容服务或 Ollama）并且向量数据库已初始化。",
        ))
      }
    }
  }

  /// 设置 OpenAI 兼容嵌入服务配置
  pub fn set_openai_embedding_config(&self, config: Option<OpenAIEmbeddingConfig>) {
    let config_clone = config.clone();
    if let Some(scheduler) = self.scheduler.load_full() {
      scheduler.set_openai_config(config);
      if config_clone.is_some() {
        info!("[Embedding] ✅ OpenAI 嵌入服务配置已设置");
      } else {
        info!("[Embedding] 🔧 已清除 OpenAI 嵌入服务配置，将使用 Ollama");
      }
    } else {
      warn!("[Embedding] ⚠️ Scheduler 尚未初始化，OpenAI 配置将在 scheduler 创建后设置");
      // 注意：这里不设置配置，因为 scheduler 还没有创建
      // 配置会在 scheduler 创建后通过其他方式设置
    }
  }

  fn try_create_scheduler(&self) {
    // 🔧 修复：添加更多调试信息
    info!("[Embedding] 🔍 开始尝试创建 scheduler...");
    
    // 🔧 修复：只要有 vector_db 就可以创建调度器
    // Ollama 是可选的，因为可以使用 OpenAI 兼容服务
    if let Some(vector_db) = self.vector_db.load_full() {
      let ollama = self.ollama.load_full();
      
      if ollama.is_some() {
        info!("[Embedding] 🔧 Creating scheduler with Ollama");
      } else {
        info!("[Embedding] 🔧 Creating scheduler without Ollama (will use OpenAI-compatible service)");
      }
      
      // 创建一个虚拟的 Ollama 实例（如果没有的话）
      let ollama = ollama.unwrap_or_else(|| {
        // 创建一个占位的 Ollama 实例，不会被实际使用
        Arc::new(Ollama::new("http://localhost".to_string(), 11434))
      });
      
      info!("[Embedding] 🔍 调用 EmbeddingScheduler::new...");
      match EmbeddingScheduler::new(ollama, vector_db) {
        Ok(s) => {
          info!("[Embedding] ✅ Scheduler created successfully!");
          info!("[Embedding] 🔍 准备存储 scheduler 到 ArcSwapOption...");
          self.scheduler.store(Some(s));
          info!("[Embedding] ✅ Scheduler 已成功存储到 ArcSwapOption");
          
          // 🔧 验证存储是否成功
          if let Some(_) = self.scheduler.load_full() {
            info!("[Embedding] ✅ 验证：scheduler 存储成功，可以获取");
          } else {
            error!("[Embedding] ❌ 验证失败：scheduler 存储后无法获取");
          }
        },
        Err(err) => error!("[Embedding] ❌ Failed to create scheduler: {}", err),
      }
    } else {
      warn!("[Embedding] ⚠️ Vector db is not initialized, cannot create scheduler");
      self.scheduler.store(None);
    }
  }
}
