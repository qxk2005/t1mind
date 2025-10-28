use crate::embeddings::embedder::{Embedder, OllamaEmbedder, OpenAIEmbedder};
use crate::embeddings::indexer::IndexerProvider;
use crate::rag::{HybridRetriever, HybridRetrieverConfig};
use crate::rag::RAGConfigManager;
use crate::search::summary::{LLMDocument, summarize_documents};
use flowy_ai_pub::cloud::search_dto::{
  SearchContentType, SearchDocumentResponseItem, SearchResult, SearchSummaryResult, Summary,
};
use flowy_ai_pub::entities::{EmbeddingRecord, UnindexedCollab, UnindexedData};
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite::internal::derives::multiconnection::chrono::Utc;
use flowy_sqlite_vec::db::VectorSqliteDB;
use ollama_rs::Ollama;
use std::sync::{Arc, Weak};
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::{broadcast, mpsc};
use arc_swap::ArcSwapOption;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

type UnindexedCollabContext = UnindexedCollab;

/// OpenAI 兼容嵌入服务配置
#[derive(Debug, Clone)]
pub struct OpenAIEmbeddingConfig {
  pub base_url: String,
  pub api_key: String,
  pub model: String,
  pub rag_score_threshold: f32,  // RAG 文档检索相似度阈值 (0.0-1.0)
}

pub struct EmbeddingScheduler {
  indexer_provider: Arc<IndexerProvider>,
  write_embedding_tx: UnboundedSender<EmbeddingRecord>,
  generate_embedding_tx: mpsc::Sender<UnindexedCollab>,
  ollama: Arc<Ollama>,
  openai_config: ArcSwapOption<OpenAIEmbeddingConfig>,
  vector_db: Arc<VectorSqliteDB>,
  pub(crate) stop_tx: tokio::sync::broadcast::Sender<()>,
}

impl EmbeddingScheduler {
  pub fn new(
    ollama: Arc<Ollama>,
    vector_db: Arc<VectorSqliteDB>,
  ) -> FlowyResult<Arc<EmbeddingScheduler>> {
    let indexer_provider = IndexerProvider::new();
    let (write_embedding_tx, write_embedding_rx) = unbounded_channel::<EmbeddingRecord>();
    let (generate_embedding_tx, gen_embedding_rx) = mpsc::channel::<UnindexedCollabContext>(100);
    let (stop_tx, _) = broadcast::channel::<()>(1);

    let this = Arc::new(Self {
      indexer_provider,
      write_embedding_tx,
      generate_embedding_tx,
      ollama,
      openai_config: ArcSwapOption::new(None),
      vector_db,
      stop_tx,
    });

    let weak_this = Arc::downgrade(&this);
    let stop_rx = this.stop_tx.subscribe();
    tokio::spawn(spawn_generate_embeddings(
      gen_embedding_rx,
      weak_this.clone(),
      stop_rx,
    ));

    let weak_this = Arc::downgrade(&this);
    let stop_rx = this.stop_tx.subscribe();
    tokio::spawn(spawn_write_embeddings(
      write_embedding_rx,
      weak_this,
      stop_rx,
    ));

    Ok(this)
  }

  pub fn new_with_config(
    ollama: Arc<Ollama>,
    vector_db: Arc<VectorSqliteDB>,
    rag_config: Arc<RAGConfigManager>,
  ) -> FlowyResult<Arc<EmbeddingScheduler>> {
    let indexer_provider = IndexerProvider::new_with_config(rag_config);
    let (write_embedding_tx, write_embedding_rx) = unbounded_channel::<EmbeddingRecord>();
    let (generate_embedding_tx, gen_embedding_rx) = mpsc::channel::<UnindexedCollabContext>(100);
    let (stop_tx, _) = broadcast::channel::<()>(1);

    let this = Arc::new(Self {
      indexer_provider,
      write_embedding_tx,
      generate_embedding_tx,
      ollama,
      openai_config: ArcSwapOption::new(None),
      vector_db,
      stop_tx,
    });

    let weak_this = Arc::downgrade(&this);
    let stop_rx = this.stop_tx.subscribe();
    tokio::spawn(spawn_generate_embeddings(
      gen_embedding_rx,
      weak_this.clone(),
      stop_rx,
    ));

    let weak_this = Arc::downgrade(&this);
    let stop_rx = this.stop_tx.subscribe();
    tokio::spawn(spawn_write_embeddings(
      write_embedding_rx,
      weak_this,
      stop_rx,
    ));

    Ok(this)
  }

  /// 设置 OpenAI 兼容嵌入服务配置
  pub fn set_openai_config(&self, config: Option<OpenAIEmbeddingConfig>) {
    if let Some(ref cfg) = config {
      info!(
        "[Embedding] 🔄 切换到 OpenAI 兼容嵌入服务: {} (模型: {}), RAG阈值: {:.2}",
        cfg.base_url, cfg.model, cfg.rag_score_threshold
      );
      self.openai_config.store(Some(Arc::new(cfg.clone())));
    } else {
      trace!("[Embedding] 🔄 切换回 Ollama 嵌入服务");
      self.openai_config.store(None);
    }
  }
  
  /// 更新相似度阈值（从外部调用）
  pub fn update_score_threshold(&self, new_threshold: f32) {
    if let Some(current_config) = self.openai_config.load_full() {
      let mut updated_config = (*current_config).clone();
      updated_config.rag_score_threshold = new_threshold.clamp(0.0, 1.0);
      info!(
        "[Embedding] 🔄 更新相似度阈值: {:.2} -> {:.2}",
        current_config.rag_score_threshold, updated_config.rag_score_threshold
      );
      self.openai_config.store(Some(Arc::new(updated_config)));
    }
  }

  /// 获取当前 OpenAI 配置
  pub fn get_openai_config(&self) -> Option<OpenAIEmbeddingConfig> {
    self.openai_config.load_full().map(|config| (*config).clone())
  }

  pub(crate) fn create_embedder(&self) -> Result<Embedder, FlowyError> {
    // 优先使用 OpenAI 兼容配置
    if let Some(config) = self.openai_config.load_full() {
      info!(
        "[Embedding] 📤 使用 OpenAI 兼容嵌入器: {} (模型: {})",
        config.base_url, config.model
      );
      return Ok(Embedder::OpenAI(OpenAIEmbedder {
        base_url: config.base_url.clone(),
        api_key: config.api_key.clone(),
        model: config.model.clone(),
      }));
    }

    // 默认使用 Ollama
    trace!("[Embedding] 📤 使用 Ollama 嵌入器");
    Ok(Embedder::Ollama(OllamaEmbedder {
      ollama: self.ollama.clone(),
    }))
  }

  pub async fn index_collab(&self, data: UnindexedCollab) -> FlowyResult<()> {
    trace!("[Embedding] got {} unindexd data", data.object_id);
    if let Err(err) = self.generate_embedding_tx.send(data).await {
      error!("[Embedding] error generating embedding: {}", err);
    }
    Ok(())
  }

  pub async fn delete_collab(&self, workspace_id: &Uuid, object_id: &Uuid) -> FlowyResult<()> {
    self
      .vector_db
      .delete_collab(&workspace_id.to_string(), &object_id.to_string())
      .await
      .map_err(|err| {
        error!("[Embedding] Failed to delete collab: {}", err);
        FlowyError::new(ErrorCode::LocalEmbeddingNotReady, "Failed to delete collab")
      })?;
    Ok(())
  }

  pub async fn search(
    &self,
    workspace_id: &Uuid,
    query: &str,
  ) -> FlowyResult<Vec<SearchDocumentResponseItem>> {
    self.search_with_filter(workspace_id, query, 10, None).await
  }

  /// 搜索文档，支持按 object_ids 过滤和限制结果数量
  pub async fn search_with_filter(
    &self,
    workspace_id: &Uuid,
    query: &str,
    limit: usize,
    object_ids: Option<Vec<String>>,
  ) -> FlowyResult<Vec<SearchDocumentResponseItem>> {
    let embedder = self.create_embedder()?;
    
    // 使用 embed_texts 来支持 OpenAI 和 Ollama
    let embeddings = embedder.embed_texts(vec![query.to_string()]).await?;
    
    match embeddings.first() {
      None => Ok(vec![]),
      Some(query_embed) => {
        let object_ids_slice = object_ids.as_deref().unwrap_or(&[]);
        
        // 尝试从 RAG 配置管理器读取配置
        let rag_config = self.indexer_provider.get_rag_config();
        let (initial_top_k, final_top_k, enable_hybrid, vector_weight, keyword_weight) = if let Some(config) = rag_config {
          let settings = config.get_rag_settings();
          let result = (settings.initial_top_k as usize, settings.final_top_k as usize, 
           settings.enable_hybrid_search, settings.vector_weight, settings.keyword_weight);
          info!("[RAG Config] 从配置管理器读取: initial_top_k={}, final_top_k={}, enable_hybrid={}, vector_weight={}, keyword_weight={}", 
                result.0, result.1, result.2, result.3, result.4);
          result
        } else {
          // 如果没有配置，使用传入的 limit 作为两阶段的 top_k
          let result = (limit, limit, false, 1.0, 0.0);
          warn!("[RAG Config] 未找到 RAG 配置管理器，使用默认值: initial_top_k={}, final_top_k={}", result.0, result.1);
          result
        };
        
        // 从配置中读取相似度阈值，如果未配置则使用默认值 0.25
        let score_threshold = self.openai_config
          .load()
          .as_ref()
          .map(|cfg| cfg.rag_score_threshold)
          .unwrap_or(0.25);
        
        trace!(
          "[Embedding] 🔍 执行检索: query='{}', initial_top_k={}, final_top_k={}, enable_hybrid={}, limit={}, score_threshold={:.2} (来源: {}), object_ids={:?}",
          query, 
          initial_top_k,
          final_top_k,
          enable_hybrid,
          limit, 
          score_threshold, 
          if self.openai_config.load().is_some() { "配置" } else { "默认" },
          object_ids
        );
        
        // 根据配置决定使用混合检索还是单向量检索
        let results = if enable_hybrid {
          // 使用混合检索
          info!("[Embedding] 🎯 启用混合检索模式");
          
          let hybrid_config = HybridRetrieverConfig {
            vector_weight,
            keyword_weight,
            top_k: initial_top_k,
            score_threshold,
          };
          
          let hybrid_retriever = HybridRetriever::new(
            Arc::downgrade(&self.vector_db),
            hybrid_config,
          );
          
          // 准备对象ID列表
          let object_ids_vec: Vec<String> = object_ids_slice.iter().map(|s| s.to_string()).collect();
          
          let hybrid_results = hybrid_retriever.search(
            query,
            workspace_id,
            &object_ids_vec,
            query_embed,
          ).await.map_err(|err| {
            error!("[Embedding] Hybrid search failed: {}", err);
            FlowyError::new(ErrorCode::LocalEmbeddingNotReady, "Hybrid search failed")
          })?;
          
          // 转换为 SearchDocumentResponseItem
          hybrid_results.into_iter()
            .take(final_top_k)
            .filter_map(|doc| {
              // 将 document_id (String) 解析为 Uuid
              match Uuid::parse_str(&doc.document_id) {
                Ok(object_id) => Some(SearchDocumentResponseItem {
                  object_id,
                  workspace_id: *workspace_id,
                  score: doc.combined_score as f64,
                  content_type: Some(SearchContentType::PlainText),
                  content: doc.content,
                  preview: None,
                  created_by: "".to_string(),
                  created_at: Utc::now(),
                }),
                Err(e) => {
                  warn!("[Embedding] Failed to parse document_id '{}' as Uuid: {}", doc.document_id, e);
                  None
                }
              }
            })
            .collect::<Vec<_>>()
        } else {
          // 使用单向量检索（向后兼容）
          info!("[Embedding] 📍 使用单向量检索模式");
          
          let effective_limit = if limit < initial_top_k { limit } else { initial_top_k };
          
          let result = self
            .vector_db
            .search_with_score(
              &workspace_id.to_string(),
              object_ids_slice,
              query_embed,
              effective_limit as i32,
              score_threshold,
            )
            .await
            .map_err(|err| {
              error!("[Embedding] Failed to search: {}", err);
              FlowyError::new(ErrorCode::LocalEmbeddingNotReady, "Failed to search")
            })?;
          
          // 取前 final_top_k 个结果
          let rows: Vec<_> = result
            .into_iter()
            .take(final_top_k.min(limit))
            .map(|v| SearchDocumentResponseItem {
              object_id: v.oid,
              workspace_id: *workspace_id,
              score: v.score as f64,
              content_type: Some(SearchContentType::PlainText),
              content: v.content,
              preview: None,
              created_by: "".to_string(),
              created_at: Utc::now(),
            })
            .collect();
          
          rows
        };
        
        // 添加调试：显示搜索结果的详细信息
        if results.is_empty() {
          warn!(
            "[Embedding] ⚠️ 搜索返回0个结果 - 可能原因: 1)相似度分数低于阈值{:.2} 2)指定的object_ids不存在 3)向量数据库为空",
            score_threshold
          );
        } else {
          info!(
            "[Embedding] 🎯 搜索完成: 找到 {} 个结果 (初始: {} 个候选项)",
            results.len(),
            initial_top_k
          );
        }
        
        // 输出每个结果的分数，帮助调试
        for (idx, item) in results.iter().enumerate() {
          trace!(
            "[Embedding] 📄 结果 #{}: object_id={}, score={:.4}, content_preview='{}'",
            idx + 1,
            item.object_id,
            item.score,
            item.content.chars().take(100).collect::<String>()
          );
        }

        Ok(results)
      },
    }
  }

  pub async fn generate_summary(
    &self,
    question: &str,
    model_name: &str,
    search_results: Vec<SearchResult>,
  ) -> FlowyResult<SearchSummaryResult> {
    if search_results.is_empty() {
      return Ok(SearchSummaryResult { summaries: vec![] });
    }

    trace!("[Search] generate local ai overview");
    let docs = search_results
      .into_iter()
      .map(|v| LLMDocument {
        content: v.content,
        object_id: v.object_id,
      })
      .collect::<Vec<_>>();

    let resp = summarize_documents(&self.ollama, question, model_name, docs)
      .await
      .map_err(|err| {
        error!("[Embedding] Failed to generate summary: {}", err);
        FlowyError::new(
          ErrorCode::LocalEmbeddingNotReady,
          "Failed to generate summary",
        )
      })?;

    let summaries = resp
      .summaries
      .into_iter()
      .flat_map(|s| {
        if s.content.is_empty() {
          None
        } else {
          Some(Summary {
            content: s.content,
            highlights: s.highlights,
            sources: s.sources,
          })
        }
      })
      .collect::<Vec<_>>();
    Ok(SearchSummaryResult { summaries })
  }
}

const EMBEDDING_RECORD_BUFFER_SIZE: usize = 10;

pub async fn spawn_write_embeddings(
  mut rx: UnboundedReceiver<EmbeddingRecord>,
  scheduler: Weak<EmbeddingScheduler>,
  mut stop_rx: broadcast::Receiver<()>,
) {
  let mut buf = Vec::with_capacity(EMBEDDING_RECORD_BUFFER_SIZE);
  trace!("[Embedding] spawn embedding writer");

  loop {
    select! {
      // Shutdown signal arrives
      _ = stop_rx.recv() => {
          trace!("[Embedding] Received stop signal; shutting down embedding writer");
          break;
      }
      // Next batch from the input channel
      n = rx.recv_many(&mut buf, EMBEDDING_RECORD_BUFFER_SIZE) => {
        // channel closed
        if n == 0 {
          trace!("[Embedding] Input channel closed; stopping write embeddings");
          break;
        }

        // upgrade scheduler reference
        let scheduler = match scheduler.upgrade() {
          Some(db) => db,
          None => {
              error!("[Embedding] EmbeddingScheduler dropped; stopping write embeddings");
              break;
          }
        };

        // drain and process exactly `n` records
        let records = buf.drain(..n).collect::<Vec<_>>();
        for record in records {
          // debug!("[Embedding] Writing {} chunks for {}", record.chunks.len(), record.object_id);
          match scheduler
              .vector_db
              .upsert_collabs_embeddings(&record.workspace_id.to_string(), &record.object_id.to_string(), record.chunks)
              .await
          {
            Ok(_) => trace!("[Embedding] Successfully wrote embeddings for {}", record.object_id),
            Err(err) => error!("[Embedding] Failed to write embeddings for {}: {}", record.object_id, err),
          }
        }
      }
    }
  }

  info!("spawn_write_embeddings exited");
}

async fn spawn_generate_embeddings(
  mut rx: mpsc::Receiver<UnindexedCollab>,
  scheduler: Weak<EmbeddingScheduler>,
  mut stop_rx: broadcast::Receiver<()>,
) {
  let mut buf = Vec::with_capacity(EMBEDDING_RECORD_BUFFER_SIZE);
  trace!("[Embedding] spawn embedding generator");
  loop {
    select! {
      _ = stop_rx.recv() => {
        trace!("[Embedding] Received stop signal; shutting down embedding writer");
        break;
      }
      n = rx.recv_many(&mut buf, EMBEDDING_RECORD_BUFFER_SIZE) => {
        let scheduler = match scheduler.upgrade() {
          Some(scheduler) => scheduler,
          None => {
            trace!("[Embedding] Failed to upgrade scheduler connection, break loop");
            break;
          },
        };
        if n == 0 {
          trace!("[Embedding] Stop generating embeddings");
          break;
        }

        let records = buf.drain(..n).collect::<Vec<_>>();
        let indexer_provider = scheduler.indexer_provider.clone();
        let write_embedding_tx = scheduler.write_embedding_tx.clone();
        let embedder = scheduler.create_embedder();

        match embedder {
          Ok(embedder) => {
            let params: Vec<_> = records.iter().map(|r| r.object_id.to_string()).collect();
            let existing_embeddings = scheduler
                .vector_db
                .select_collabs_fragment_ids(&params)
                .await
                .unwrap_or_else(|err| {
                  error!("[Embedding] failed to get existing embeddings: {}", err);
                  Default::default()
                });

            for record in records {
              if let Some(indexer) = indexer_provider.indexer_for(record.collab_type) {
                let paragraphs = match record.data {

                None => continue, Some(data) => {
                    match data {
                      UnindexedData::Paragraphs(paragraphs) => paragraphs,
                  UnindexedData::Text(text) => text.split('\n').map(|s| s.to_string()).collect(),
                    }
                  }};
                let embedder = embedder.clone();
                match indexer.create_embedded_chunks_from_text(
                  record.object_id,
                  paragraphs,
                  embedder.model(),
                ) {
                  Ok(mut chunks) => {
                    info!(
                        "[Embedding] 📊 Document {}: Created {} chunks",
                        record.object_id,
                        chunks.len()
                    );
                    
                    if let Some(fragment_ids) = existing_embeddings.get(&record.object_id) {
                      info!(
                          "[Embedding] 🔄 Document {}: Found {} existing fragments, checking for updates",
                          record.object_id,
                          fragment_ids.len()
                      );
                      for chunk in chunks.iter_mut() {
                        if fragment_ids.contains(&chunk.fragment_id) {
                          chunk.content = None;
                        }
                      }
                    }

                    if chunks.iter().all(|c| c.content.is_none()) {
                      info!(
                        "[Embedding] ⏭️ Document {}: No content changes, skip regenerating embeddings",
                        record.object_id
                      );
                      continue;
                    }
                    
                    let new_chunks_count = chunks.iter().filter(|c| c.content.is_some()).count();
                    if new_chunks_count > 0 {
                      info!(
                          "[Embedding] ✨ Document {}: Will generate embeddings for {} new/updated chunks",
                          record.object_id,
                          new_chunks_count
                      );
                    }

                    let result = indexer.embed(&embedder, chunks).await;
                    match result {
                      Ok(chunks) => {
                        let record = EmbeddingRecord {
                          workspace_id: record.workspace_id,
                          object_id: record.object_id,
                          chunks,
                        };
                        if let Err(err) = write_embedding_tx.send(record) {
                          error!("Failed to send embedding record: {}", err);
                        }
                      },
                      Err(err) => {
                        error!(
                          "[Embedding] Failed to create embeddings content for collab: {}, error:{}",
                          record.object_id, err
                        );
                      },
                    }
                  },
                  Err(err) => {
                    warn!(
                      "Failed to create embedded chunks for collab: {}, error:{}",
                      record.object_id, err
                    );
                    continue;
                  },
                }
              }
            }
          },
          Err(err) => error!("[Embedding] Failed to create embedder: {}", err),
        }
      }
    }
  }
  info!("spawn_generate_embeddings exited");
}
