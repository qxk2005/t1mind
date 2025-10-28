use crate::embeddings::document_indexer::split_text_into_chunks;
use crate::embeddings::embedder::{Embedder, OllamaEmbedder};
use crate::embeddings::indexer::{EmbeddingModel, IndexerProvider};
use crate::local_ai::chat::retriever::MultipleSourceRetrieverStore;
use crate::rag::RAGConfigManager;
use async_trait::async_trait;
use flowy_ai_pub::cloud::CollabType;
use flowy_ai_pub::entities::{RAG_IDS, SOURCE_ID};
use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite_vec::db::VectorSqliteDB;
use flowy_sqlite_vec::entities::{EmbeddedContent, SqliteEmbeddedDocument};
use tracing::{info, warn, trace, error};
use futures::stream::{self, StreamExt};
use langchain_rust::llm::client::OllamaClient;
use langchain_rust::schemas::Document;
use langchain_rust::vectorstore::{VecStoreOptions, VectorStore};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Weak};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteVectorStore {
  ollama: Weak<OllamaClient>,
  vector_db: Weak<VectorSqliteDB>,
  indexer_provider: Arc<IndexerProvider>,
  rag_config: Option<Arc<RAGConfigManager>>,
}

impl SqliteVectorStore {
  pub fn new(ollama: Weak<OllamaClient>, vector_db: Weak<VectorSqliteDB>) -> Self {
    Self {
      ollama,
      vector_db,
      indexer_provider: IndexerProvider::new(),
      rag_config: None,
    }
  }

  pub fn new_with_config(ollama: Weak<OllamaClient>, vector_db: Weak<VectorSqliteDB>, rag_config: Arc<RAGConfigManager>) -> Self {
    Self {
      ollama,
      vector_db,
      indexer_provider: IndexerProvider::new_with_config(rag_config.clone()),
      rag_config: Some(rag_config),
    }
  }

  pub(crate) fn create_embedder(&self) -> Result<Embedder, FlowyError> {
    let ollama = self
      .ollama
      .upgrade()
      .ok_or_else(|| FlowyError::internal().with_context("Ollama reference was dropped"))?;

    let embedder = Embedder::Ollama(OllamaEmbedder { ollama });
    Ok(embedder)
  }

  pub(crate) async fn select_all_embedded_documents(
    &self,
    workspace_id: &str,
    rag_ids: &[String],
  ) -> FlowyResult<Vec<SqliteEmbeddedDocument>> {
    // Get the vector database
    let vector_db = match self.vector_db.upgrade() {
      Some(db) => db,
      None => return Err(FlowyError::internal().with_context("Vector database not initialized")),
    };

    vector_db
      .select_all_embedded_documents(workspace_id, rag_ids)
      .await
      .map_err(|err| {
        FlowyError::internal().with_context(format!("Failed to select embedded documents: {}", err))
      })
  }

  pub async fn select_all_embedded_content(
    &self,
    workspace_id: &str,
    rag_ids: &[String],
    limit: usize,
  ) -> FlowyResult<Vec<EmbeddedContent>> {
    let vector_db = match self.vector_db.upgrade() {
      Some(db) => db,
      None => return Err(FlowyError::internal().with_context("Vector database not initialized")),
    };

    vector_db
      .select_all_embedded_content(workspace_id, rag_ids, limit)
      .await
      .map_err(|err| {
        FlowyError::internal().with_context(format!("Failed to select embedded content: {}", err))
      })
  }
}

#[async_trait]
impl MultipleSourceRetrieverStore for SqliteVectorStore {
  fn retriever_name(&self) -> &'static str {
    "Sqlite Multiple Source Retriever"
  }

  async fn read_documents(
    &self,
    workspace_id: &Uuid,
    query: &str,
    limit: usize,
    rag_ids: &[String],
    score_threshold: f32,
    _full_search: bool,
  ) -> FlowyResult<Vec<Document>> {
    let vector_db = match self.vector_db.upgrade() {
      Some(db) => db,
      None => return Err(FlowyError::internal().with_context("Vector database not initialized")),
    };

    // Create embedder and generate embedding for query
    let embedder = self.create_embedder()?;
    
    // 使用 embed_texts 来支持 OpenAI 和 Ollama
    let embeddings = embedder.embed_texts(vec![query.to_string()]).await?;
    if embeddings.is_empty() {
      return Ok(Vec::new());
    }

    debug_assert!(embeddings.len() == 1);
    let query_embedding = embeddings.first().unwrap();

    // 添加调试：检查rag_ids是否为空
    if rag_ids.is_empty() {
      warn!("[VectorStore] ⚠️ rag_ids为空，将搜索所有文档");
    } else {
      info!("[VectorStore] 📋 将搜索指定的 {} 个文档ID", rag_ids.len());
    }
    
    trace!(
      "[VectorStore] 🔍 执行向量搜索: query='{}', limit={}, score_threshold={:.2}, rag_ids={:?}",
      query, 
      limit, 
      score_threshold, 
      rag_ids
    );

    // Perform similarity search in the database
    let results = vector_db
      .search_with_score(
        &workspace_id.to_string(),
        rag_ids,
        query_embedding,
        limit as i32,
        score_threshold,
      )
      .await?;

    trace!(
      "[VectorStore] Found {} results for query:{}, rag_ids: {:?}, score_threshold: {}",
      results.len(),
      query,
      rag_ids,
      score_threshold
    );

    // 添加调试：显示搜索结果的详细信息
    if results.is_empty() {
      warn!(
        "[VectorStore] ⚠️ 搜索返回0个结果 - 可能原因: 1)相似度分数低于阈值{:.2} 2)指定的rag_ids不存在 3)向量数据库为空",
        score_threshold
      );
    } else {
      info!(
        "[VectorStore] 🎯 搜索完成: 找到 {} 个结果",
        results.len()
      );
    }

    // Convert results to Documents
    let documents = results
      .into_iter()
      .map(|result| {
        let mut metadata = HashMap::new();

        if let Some(map) = result.metadata.as_ref().and_then(|v| v.as_object()) {
          for (key, value) in map {
            metadata.insert(key.clone(), value.clone());
          }
        }

        Document::new(result.content).with_metadata(metadata)
      })
      .collect();

    Ok(documents)
  }
}

#[async_trait]
impl VectorStore for SqliteVectorStore {
  type Options = VecStoreOptions<Value>;
  async fn add_documents(
    &self,
    docs: &[Document],
    _opt: &Self::Options,
  ) -> Result<Vec<String>, Box<dyn Error>> {
    let vector_db = match self.vector_db.upgrade() {
      Some(db) => db,
      None => return Err("Vector database not initialized".into()),
    };

    let embedder = self.create_embedder()?;

    let indexer = self
      .indexer_provider
      .indexer_for(CollabType::Document)
      .ok_or_else(|| Box::<dyn Error>::from("Failed to get indexer for Document"))?;

    // Parse documents and filter out invalid ones early
    let documents = docs
      .iter()
      .filter_map(|v| {
        let workspace_id = match v.metadata.get("workspace_id") {
          Some(value) => value.as_str().and_then(|s| Uuid::parse_str(s).ok()),
          None => None,
        }?;

        let object_id = match v.metadata.get(SOURCE_ID) {
          Some(value) => value.as_str().and_then(|s| Uuid::parse_str(s).ok()),
          None => None,
        }?;

        Some((workspace_id, object_id, v.page_content.clone()))
      })
      .collect::<Vec<(Uuid, Uuid, String)>>();

    let concurrency_limit = 4;
    let rag_config_clone = self.rag_config.clone();
    let document_ids = stream::iter(documents)
      .map(|(workspace_id, object_id, paragraph)| {
        // Clone values that need to be moved into the async block
        let object_id_str = object_id.to_string();
        let workspace_id_str = workspace_id.to_string();
        let vector_db_clone = vector_db.clone();
        let embedder_clone = embedder.clone();
        let indexer_clone = indexer.clone();
        let rag_config = rag_config_clone.clone();

        async move {
          // 从配置中读取 chunk_size 和 overlap，如果没有配置则使用默认值
          // 默认值与 DocumentIndexer 保持一致
          let (chunk_size, overlap) = if let Some(ref config) = rag_config {
            let settings = config.get_rag_settings();
            (settings.chunk_size as usize, settings.chunk_overlap as usize)
          } else {
            (1000, 200)  // 默认值与 DocumentIndexer 保持一致
          };
          
          let chunks_result = split_text_into_chunks(
            &object_id_str,
            vec![paragraph],
            EmbeddingModel::NomicEmbedText,
            chunk_size,
            overlap,
          );

          match chunks_result {
            Ok(chunks) => match indexer_clone.embed(&embedder_clone, chunks).await {
              Ok(chunks) => {
                if let Err(e) = vector_db_clone
                  .upsert_collabs_embeddings(&workspace_id_str, &object_id_str, chunks)
                  .await
                {
                  error!(
                    "[Embedding] Failed to upsert document `{}`: {}",
                    object_id_str, e
                  );
                  return None;
                }

                Some(object_id_str)
              },
              Err(err) => {
                error!(
                  "[Embedding] Failed to embed document `{}`: {}",
                  object_id_str, err
                );
                None
              },
            },
            Err(err) => {
              error!(
                "[Embedding] Failed to split document `{}`: {}",
                object_id_str, err
              );
              None
            },
          }
        }
      })
      .buffer_unordered(concurrency_limit)
      .filter_map(|result| async move { result })
      .collect()
      .await;

    Ok(document_ids)
  }

  async fn similarity_search(
    &self,
    query: &str,
    limit: usize,
    opt: &Self::Options,
  ) -> Result<Vec<Document>, Box<dyn Error>> {
    // Extract rag_ids from filters
    let rag_ids = opt
      .filters
      .as_ref()
      .and_then(|filters| filters.get(RAG_IDS))
      .and_then(|value| value.as_array())
      .map(|array| {
        array
          .iter()
          .filter_map(|item| item.as_str().map(String::from))
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    // Extract workspace_id from filters
    let workspace_id = opt
      .filters
      .as_ref()
      .and_then(|filters| filters.get("workspace_id"))
      .and_then(|value| value.as_str())
      .and_then(|s| Uuid::parse_str(s).ok());

    // Return empty result if workspace_id is missing
    let workspace_id = match workspace_id {
      Some(id) => id,
      None => {
        warn!("[VectorStore] Missing workspace_id in filters. Returning empty result.");
        return Ok(Vec::new());
      },
    };

    self
      .read_documents(
        &workspace_id,
        query,
        limit,
        &rag_ids,
        opt.score_threshold.unwrap_or(0.25),
        true,
      )
      .await
      .map_err(|err| Box::new(err) as Box<dyn Error>)
  }
}
