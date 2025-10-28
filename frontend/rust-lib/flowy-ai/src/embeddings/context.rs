use crate::embeddings::scheduler::{EmbeddingScheduler, OpenAIEmbeddingConfig};
use crate::rag::RAGConfigManager;
use arc_swap::ArcSwapOption;
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite_vec::db::VectorSqliteDB;
use lib_infra::util::get_operating_system;
use ollama_rs::Ollama;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::collections::HashMap;
use tracing::{error, info, trace, warn};

/// 维度兼容性检查结果
#[derive(Debug, Clone)]
pub struct DimensionCompatibilityResult {
  pub current_db_dimension: usize,
  pub model_dimension: usize,
  pub is_compatible: bool,
  pub model_name: String,
}

/// 获取 OpenAI 嵌入模型的维度（基于已知模型）
fn get_openai_embedding_dimension(model: &str) -> usize {
  match model {
    // OpenAI 官方模型
    "text-embedding-3-small" => 1536,
    "text-embedding-3-large" => 3072,
    "text-embedding-ada-002" => 1536,
    
    // 其他常见模型
    "text-embedding-002" => 1536,
    "text-similarity-davinci-001" => 12288,
    "text-similarity-curie-001" => 12288,
    "text-similarity-babbage-001" => 2048,
    "text-similarity-ada-001" => 1024,
    
    // BGE 模型系列
    "bge-m3" => 1024,
    "bge-large-en" => 1024,
    "bge-base-en" => 768,
    "bge-small-en" => 384,
    "bge-large-zh" => 1024,
    "bge-base-zh" => 768,
    "bge-small-zh" => 512,
    
    // 默认维度（对于未知模型，需要通过测试获取）
    _ => {
      warn!("[Embedding] ⚠️ 未知的 OpenAI 嵌入模型: {}，需要通过测试获取实际维度", model);
      0 // 返回0表示需要测试获取
    }
  }
}

/// 测试嵌入模型并获取实际维度
pub async fn test_embedding_model_dimension(
  base_url: &str,
  api_key: &str,
  model: &str,
) -> FlowyResult<usize> {
  use crate::embeddings::embedder::OpenAIEmbedder;
  
  info!("[Embedding] 🧪 开始测试嵌入模型: {} 的维度", model);
  
  let embedder = OpenAIEmbedder {
    base_url: base_url.to_string(),
    api_key: api_key.to_string(),
    model: model.to_string(),
  };
  
  // 使用通用测试文本生成嵌入（使用英文，大多数模型都能处理）
  let test_texts = vec!["test".to_string()];
  
  match embedder.embed_texts(test_texts).await {
    Ok(embeddings) => {
      if let Some(first_embedding) = embeddings.first() {
        let dimension = first_embedding.len();
        info!("[Embedding] ✅ 模型 {} 的实际维度: {}", model, dimension);
        Ok(dimension)
      } else {
        Err(FlowyError::new(
          ErrorCode::Internal,
          "嵌入测试返回空结果"
        ))
      }
    },
    Err(e) => {
      error!("[Embedding] ❌ 嵌入模型测试失败: {}", e);
      Err(e)
    }
  }
}

/// 获取 Ollama 嵌入模型的维度
fn get_ollama_embedding_dimension() -> usize {
  // Ollama 默认使用 nomic-embed-text 模型，维度为 768
  // 但这里我们使用 2560 以匹配当前的数据库结构
  2560
}

pub struct EmbedContext {
  ollama: ArcSwapOption<Ollama>,
  vector_db: ArcSwapOption<VectorSqliteDB>,
  scheduler: ArcSwapOption<EmbeddingScheduler>,
  // 维度缓存：存储已测试过的模型维度
  dimension_cache: ArcSwapOption<HashMap<String, usize>>,
  // RAG 配置管理器
  rag_config: ArcSwapOption<RAGConfigManager>,
}

impl EmbedContext {
  pub fn shared() -> &'static Arc<EmbedContext> {
    static INSTANCE: OnceLock<Arc<EmbedContext>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
      Arc::new(EmbedContext {
        ollama: ArcSwapOption::new(None),
        vector_db: ArcSwapOption::new(None),
        scheduler: ArcSwapOption::new(None),
        dimension_cache: ArcSwapOption::from(Some(Arc::new(HashMap::new()))),
        rag_config: ArcSwapOption::new(None),
      })
    })
  }

  pub fn get_vector_db(&self) -> Option<Arc<VectorSqliteDB>> {
    self.vector_db.load_full()
  }

  /// 设置 RAG 配置管理器
  pub fn set_rag_config(&self, rag_config: Option<Arc<RAGConfigManager>>) {
    self.rag_config.store(rag_config);
  }

  /// 重新创建 scheduler（用于应用新的 RAG 配置）
  pub fn try_recreate_scheduler(&self) {
    info!("[Embedding] 🔄 尝试重新创建 scheduler...");
    // 先清除旧的 scheduler
    if let Some(old_scheduler) = self.scheduler.swap(None) {
      info!("[Embedding] 🗑️ 清除旧的 scheduler");
      let _ = old_scheduler.stop_tx.send(());
    }
    // 重新创建
    self.try_create_scheduler();
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

  /// 重置向量数据库 - 清空所有嵌入数据
  /// 当嵌入模型维度发生变化时使用
  pub async fn reset_vector_database(&self) -> FlowyResult<()> {
    info!("[Embedding] 🔄 开始重置向量数据库...");
    
    if let Some(vector_db) = self.vector_db.load_full() {
      vector_db.reset_vector_database().await
        .map_err(|e| FlowyError::new(
          ErrorCode::Internal,
          format!("重置向量数据库失败: {}", e)
        ))?;
      
      info!("[Embedding] ✅ 向量数据库重置完成");
      Ok(())
    } else {
      warn!("[Embedding] ⚠️ 向量数据库未初始化，无法重置");
      Err(FlowyError::new(
        ErrorCode::LocalEmbeddingNotReady,
        "向量数据库未初始化，无法重置"
      ))
    }
  }

  /// 重建向量数据库 - 支持动态维度
  /// 当嵌入模型维度发生根本性变化时使用
  pub async fn rebuild_vector_database(&self, embedding_dimension: usize) -> FlowyResult<()> {
    info!("[Embedding] 🔄 开始重建向量数据库，新维度: {}", embedding_dimension);
    
    if let Some(vector_db) = self.vector_db.load_full() {
      vector_db.rebuild_vector_database(embedding_dimension).await
        .map_err(|e| FlowyError::new(
          ErrorCode::Internal,
          format!("重建向量数据库失败: {}", e)
        ))?;
      
      info!("[Embedding] ✅ 向量数据库重建完成，新维度: {}", embedding_dimension);
      Ok(())
    } else {
      warn!("[Embedding] ⚠️ 向量数据库未初始化，无法重建");
      Err(FlowyError::new(
        ErrorCode::LocalEmbeddingNotReady,
        "向量数据库未初始化，无法重建"
      ))
    }
  }

  /// 获取当前嵌入模型的维度
  pub fn get_current_embedding_dimension(&self) -> FlowyResult<usize> {
    if let Some(scheduler) = self.scheduler.load_full() {
      // 尝试从 OpenAI 配置获取维度
      if let Some(config) = scheduler.get_openai_config() {
        let known_dimension = get_openai_embedding_dimension(&config.model);
        if known_dimension > 0 {
          return Ok(known_dimension);
        } else {
          // 对于未知模型，返回0表示需要测试
          warn!("[Embedding] ⚠️ 模型 {} 维度未知，需要测试获取", config.model);
          return Ok(0);
        }
      }
      
      // 默认使用 Ollama 的维度
      return Ok(get_ollama_embedding_dimension());
    }
    
    // 如果没有 scheduler，使用默认维度
    warn!("[Embedding] ⚠️ Scheduler 未初始化，使用默认维度");
    Ok(get_ollama_embedding_dimension())
  }

  /// 获取当前嵌入模型的维度（智能检测版本）
  pub async fn get_current_embedding_dimension_smart(&self) -> FlowyResult<usize> {
    if let Some(scheduler) = self.scheduler.load_full() {
      // 尝试从 OpenAI 配置获取维度
      if let Some(config) = scheduler.get_openai_config() {
        let known_dimension = get_openai_embedding_dimension(&config.model);
        if known_dimension > 0 {
          return Ok(known_dimension);
        } else {
          // 检查缓存中是否已有该模型的维度
          if let Some(cache) = self.dimension_cache.load_full() {
            if let Some(&cached_dimension) = cache.get(&config.model) {
              info!("[Embedding] 📋 从缓存获取模型 {} 的维度: {}", config.model, cached_dimension);
              return Ok(cached_dimension);
            }
          }
          
          // 对于未知模型，进行实际测试获取维度
          info!("[Embedding] 🧪 模型 {} 维度未知，开始实际测试获取", config.model);
          let dimension = test_embedding_model_dimension(
            &config.base_url,
            &config.api_key,
            &config.model,
          ).await?;
          
          // 将测试结果缓存
          if let Some(cache) = self.dimension_cache.load_full() {
            let mut new_cache = (*cache).clone();
            new_cache.insert(config.model.clone(), dimension);
            self.dimension_cache.store(Some(Arc::new(new_cache)));
            info!("[Embedding] 💾 已缓存模型 {} 的维度: {}", config.model, dimension);
          }
          
          return Ok(dimension);
        }
      }
      
      // 默认使用 Ollama 的维度
      return Ok(get_ollama_embedding_dimension());
    }
    
    // 如果没有 scheduler，使用默认维度
    warn!("[Embedding] ⚠️ Scheduler 未初始化，使用默认维度");
    Ok(get_ollama_embedding_dimension())
  }

  /// 智能获取嵌入模型维度（包含测试）
  pub async fn get_smart_embedding_dimension(&self) -> FlowyResult<usize> {
    if let Some(scheduler) = self.scheduler.load_full() {
      // 尝试从 OpenAI 配置获取维度
      if let Some(config) = scheduler.get_openai_config() {
        let known_dimension = get_openai_embedding_dimension(&config.model);
        if known_dimension > 0 {
          return Ok(known_dimension);
        } else {
          // 对于未知模型，进行测试获取实际维度
          info!("[Embedding] 🧪 模型 {} 维度未知，开始测试获取", config.model);
          return test_embedding_model_dimension(
            &config.base_url,
            &config.api_key,
            &config.model,
          ).await;
        }
      }
      
      // 默认使用 Ollama 的维度
      return Ok(get_ollama_embedding_dimension());
    }
    
    // 如果没有 scheduler，使用默认维度
    warn!("[Embedding] ⚠️ Scheduler 未初始化，使用默认维度");
    Ok(get_ollama_embedding_dimension())
  }

  /// 检查向量数据库维度是否与当前模型匹配
  pub async fn check_dimension_compatibility(&self) -> FlowyResult<DimensionCompatibilityResult> {
    let current_db_dimension = self.get_vector_database_dimension().await?;
    let model_dimension = self.get_smart_embedding_dimension().await?;
    
    let is_compatible = current_db_dimension == model_dimension;
    
    Ok(DimensionCompatibilityResult {
      current_db_dimension,
      model_dimension,
      is_compatible,
      model_name: self.get_current_model_name().unwrap_or_default(),
    })
  }

  /// 获取向量数据库当前维度
  pub async fn get_vector_database_dimension(&self) -> FlowyResult<usize> {
    if let Some(vector_db) = self.vector_db.load_full() {
      // 从数据库获取实际维度
      vector_db.get_embedding_dimension().await
        .map_err(|e| FlowyError::new(
          ErrorCode::Internal,
          format!("获取向量数据库维度失败: {}", e)
        ))
    } else {
      Err(FlowyError::new(
        ErrorCode::LocalEmbeddingNotReady,
        "向量数据库未初始化，无法获取维度"
      ))
    }
  }

  /// 获取当前模型名称
  pub fn get_current_model_name(&self) -> Option<String> {
    if let Some(scheduler) = self.scheduler.load_full() {
      if let Some(config) = scheduler.get_openai_config() {
        return Some(config.model.clone());
      }
    }
    None
  }

  /// 获取 OpenAI 配置（用于事件处理器）
  pub fn get_openai_config(&self) -> Option<OpenAIEmbeddingConfig> {
    if let Some(scheduler) = self.scheduler.load_full() {
      scheduler.get_openai_config()
    } else {
      None
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
      
      // 检查是否有 RAG 配置，有则使用带配置的版本
      if let Some(rag_config) = self.rag_config.load_full() {
        info!("[Embedding] 🔍 调用 EmbeddingScheduler::new_with_config (使用 RAG 配置)...");
        match EmbeddingScheduler::new_with_config(ollama, vector_db, rag_config.clone()) {
          Ok(s) => {
            info!("[Embedding] ✅ Scheduler created with RAG config!");
            self.scheduler.store(Some(s));
          },
          Err(err) => error!("[Embedding] ❌ Failed to create scheduler with config: {}", err),
        }
      } else {
        info!("[Embedding] 🔍 调用 EmbeddingScheduler::new (无 RAG 配置)...");
        match EmbeddingScheduler::new(ollama, vector_db) {
          Ok(s) => {
            info!("[Embedding] ✅ Scheduler created successfully!");
            self.scheduler.store(Some(s));
          },
          Err(err) => error!("[Embedding] ❌ Failed to create scheduler: {}", err),
        }
      }
    } else {
      warn!("[Embedding] ⚠️ Vector db is not initialized, cannot create scheduler");
      self.scheduler.store(None);
    }
  }
}
