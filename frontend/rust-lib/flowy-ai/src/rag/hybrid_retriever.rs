use crate::rag::keyword_retriever::{KeywordDocument, KeywordStore, SQLiteKeywordStore};
use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite_vec::db::VectorSqliteDB;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Weak;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 向量检索中间结果
#[derive(Debug, Clone)]
struct VectorSearchItem {
    object_id: String,
    content: String,
    metadata: Option<Value>,
    score: f32,
}

/// 混合检索结果
#[derive(Debug, Clone)]
pub struct HybridRetrievedDocument {
    /// 文档内容
    pub content: String,
    
    /// 文档ID（来自vec: object_id, 来自keyword: document_id）
    pub document_id: String,
    
    /// 元数据
    pub metadata: HashMap<String, String>,
    
    /// 向量检索分数（0.0-1.0）
    pub vector_score: Option<f32>,
    
    /// 关键词检索分数（BM25分数，已标准化到0.0-1.0）
    pub keyword_score: Option<f32>,
    
    /// 综合分数（加权组合）
    pub combined_score: f32,
}

/// 混合检索器配置
#[derive(Debug, Clone)]
pub struct HybridRetrieverConfig {
    /// 向量检索权重 (默认 0.7)
    pub vector_weight: f32,
    
    /// 关键词检索权重 (默认 0.3)
    pub keyword_weight: f32,
    
    /// 返回结果的最大数量
    pub top_k: usize,
    
    /// 相似度分数阈值
    pub score_threshold: f32,
}

impl Default for HybridRetrieverConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            keyword_weight: 0.3,
            top_k: 10,
            score_threshold: 0.25,
        }
    }
}

/// 混合检索器
/// 
/// 并行执行向量检索和关键词检索，合并结果并去重。
pub struct HybridRetriever {
    /// 向量数据库
    vector_db: Weak<VectorSqliteDB>,
    
    /// 关键词存储
    keyword_store: SQLiteKeywordStore,
    
    /// 检索配置
    config: HybridRetrieverConfig,
}

impl HybridRetriever {
    /// 创建新的混合检索器
    pub fn new(
        vector_db: Weak<VectorSqliteDB>,
        config: HybridRetrieverConfig,
    ) -> Self {
        let keyword_store = SQLiteKeywordStore::new(vector_db.clone());
        Self {
            vector_db,
            keyword_store,
            config,
        }
    }
    
    /// 执行混合检索
    pub async fn search(
        &self,
        query: &str,
        workspace_id: &Uuid,
        rag_ids: &[String],
        query_embedding: &[f32],
    ) -> FlowyResult<Vec<HybridRetrievedDocument>> {
        debug!(
            "[HybridRetriever] Starting hybrid search: query='{}', rag_ids={:?}, weights={},{}",
            query, rag_ids, self.config.vector_weight, self.config.keyword_weight
        );
        
        // 并行执行向量检索和关键词检索
        let (vector_results, keyword_results) = futures::future::join(
            self.search_vector(query, workspace_id, rag_ids, query_embedding),
            self.search_keywords(query, workspace_id, rag_ids),
        )
        .await;
        
        // 合并结果
        let merged = self.merge_results(vector_results, keyword_results)?;
        
        info!(
            "[HybridRetriever] Search complete: query='{}', returned {} results",
            query, merged.len()
        );
        
        Ok(merged)
    }
    
    /// 执行向量检索
    async fn search_vector(
        &self,
        query: &str,
        workspace_id: &Uuid,
        rag_ids: &[String],
        query_embedding: &[f32],
    ) -> Result<Vec<VectorSearchItem>, FlowyError> {
        let vector_db = self.vector_db.upgrade()
            .ok_or_else(|| FlowyError::internal().with_context("Vector database not initialized"))?;
        
        match vector_db.search_with_score(
            &workspace_id.to_string(),
            rag_ids,
            query_embedding,
            self.config.top_k as i32,
            self.config.score_threshold,
        ).await {
            Ok(results) => {
                debug!("[HybridRetriever] Vector search returned {} results", results.len());
                Ok(results.into_iter()
                    .map(|result| VectorSearchItem {
                        object_id: result.oid.to_string(),
                        content: result.content,
                        metadata: result.metadata,
                        score: result.score,
                    })
                    .collect())
            },
            Err(e) => {
                error!("[HybridRetriever] Vector search failed: {}", e);
                // 返回空结果而不是错误，允许关键词检索单独工作
                Ok(Vec::new())
            },
        }
    }
    
    /// 执行关键词检索
    async fn search_keywords(
        &self,
        query: &str,
        workspace_id: &Uuid,
        rag_ids: &[String],
    ) -> Result<Vec<KeywordDocument>, FlowyError> {
        match self.keyword_store.search(
            query,
            &workspace_id.to_string(),
            rag_ids,
            self.config.top_k,
        ).await {
            Ok(docs) => {
                debug!("[HybridRetriever] Keyword search returned {} results", docs.len());
                Ok(docs)
            },
            Err(e) => {
                error!("[HybridRetriever] Keyword search failed: {}", e);
                // 返回空结果而不是错误，允许向量检索单独工作
                Ok(Vec::new())
            },
        }
    }
    
    /// 合并向量检索和关键词检索结果
    fn merge_results(
        &self,
        vector_results: Result<Vec<VectorSearchItem>, FlowyError>,
        keyword_results: Result<Vec<KeywordDocument>, FlowyError>,
    ) -> FlowyResult<Vec<HybridRetrievedDocument>> {
        // 解包结果，错误情况下使用空列表
        let vector_docs = vector_results.unwrap_or_default();
        let keyword_docs = keyword_results.unwrap_or_default();
        
        if vector_docs.is_empty() && keyword_docs.is_empty() {
            warn!("[HybridRetriever] Both retrieval methods returned empty results");
            return Ok(Vec::new());
        }
        
        debug!(
            "[HybridRetriever] Merging results: {} vector + {} keyword results",
            vector_docs.len(), keyword_docs.len()
        );
        
        // 归一化分数到 0.0-1.0 范围
        let normalized_vector = self.normalize_vector_scores(&vector_docs);
        let normalized_keyword = self.normalize_keyword_scores(&keyword_docs);
        
        // 使用 HashMap 进行去重和合并
        let mut merged_map: HashMap<String, HybridRetrievedDocument> = HashMap::new();
        
        // 添加向量检索结果
        for doc in normalized_vector {
            let combined_score = self.config.vector_weight * doc.score;
            
            // 将元数据转换为 HashMap
            let metadata = match &doc.metadata {
                Some(Value::Object(map)) => {
                    map.iter()
                        .filter_map(|(k, v)| {
                            Some((k.clone(), v.as_str()?.to_string()))
                        })
                        .collect()
                },
                _ => HashMap::new(),
            };
            
            merged_map.insert(
                doc.object_id.clone(),
                HybridRetrievedDocument {
                    document_id: doc.object_id.clone(),
                    content: doc.content.clone(),
                    metadata,
                    vector_score: Some(doc.score),
                    keyword_score: None,
                    combined_score,
                },
            );
        }
        
        // 合并关键词检索结果
        for doc in normalized_keyword {
            let entry = merged_map.entry(doc.document_id.clone());
            
            match entry {
                std::collections::hash_map::Entry::Occupied(mut occupied) => {
                    // 文档已存在，合并分数
                    let existing = occupied.get_mut();
                    let keyword_score = doc.score;
                    existing.keyword_score = Some(keyword_score);
                    
                    // 更新综合分数：向量分数 + 关键词分数（按权重加权）
                    if let Some(vec_score) = existing.vector_score {
                        existing.combined_score = 
                            self.config.vector_weight * vec_score +
                            self.config.keyword_weight * keyword_score;
                    } else {
                        existing.combined_score = self.config.keyword_weight * keyword_score;
                    }
                },
                std::collections::hash_map::Entry::Vacant(vacant) => {
                    // 新文档
                    let combined_score = self.config.keyword_weight * doc.score;
                    vacant.insert(HybridRetrievedDocument {
                        document_id: doc.document_id,
                        content: doc.content,
                        metadata: doc.metadata,
                        vector_score: None,
                        keyword_score: Some(doc.score),
                        combined_score,
                    });
                },
            }
        }
        
        // 转换为向量并按综合分数排序
        let mut results: Vec<HybridRetrievedDocument> = merged_map.into_values().collect();
        results.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 记录合并后的总数（在截断前）
        let before_truncate = results.len();
        
        // 限制返回数量
        results.truncate(self.config.top_k);
        
        info!(
            "[HybridRetriever] Merged to {} results, top {} retained (config.top_k={})",
            before_truncate,
            results.len(),
            self.config.top_k
        );
        
        Ok(results)
    }
    
    /// 归一化向量检索分数到 0.0-1.0 范围
    fn normalize_vector_scores(
        &self,
        docs: &[VectorSearchItem],
    ) -> Vec<VectorSearchItem> {
        if docs.is_empty() {
            return Vec::new();
        }
        
        // 找到最高分和最低分
        let scores: Vec<f32> = docs.iter().map(|d| d.score).collect();
        let max_score = scores
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let score_range = max_score - min_score;
        
        if score_range <= 0.0 {
            // 所有分数相同，直接返回
            return docs.to_vec();
        }
        
        // 归一化所有分数
        docs.iter()
            .map(|doc| VectorSearchItem {
                score: (doc.score - min_score) / score_range,
                ..doc.clone()
            })
            .collect()
    }
    
    /// 归一化关键词检索分数到 0.0-1.0 范围
    fn normalize_keyword_scores(
        &self,
        docs: &[KeywordDocument],
    ) -> Vec<KeywordDocument> {
        if docs.is_empty() {
            return Vec::new();
        }
        
        // 找到最高分和最低分
        let scores: Vec<f32> = docs.iter().map(|d| d.score).collect();
        let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let score_range = max_score - min_score;
        
        if score_range <= 0.0 {
            // 所有分数相同，设置为1.0
            return docs.iter()
                .map(|doc| KeywordDocument {
                    score: 1.0,
                    ..doc.clone()
                })
                .collect();
        }
        
        // 归一化所有分数
        docs.iter()
            .map(|doc| KeywordDocument {
                score: (doc.score - min_score) / score_range,
                ..doc.clone()
            })
            .collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = HybridRetrieverConfig::default();
        assert_eq!(config.vector_weight, 0.7);
        assert_eq!(config.keyword_weight, 0.3);
        assert_eq!(config.top_k, 10);
        assert_eq!(config.score_threshold, 0.25);
    }
    
    #[test]
    fn test_config_weights_sum() {
        let config = HybridRetrieverConfig {
            vector_weight: 0.6,
            keyword_weight: 0.4,
            ..Default::default()
        };
        assert!((config.vector_weight + config.keyword_weight - 1.0).abs() < 0.01);
    }
}

