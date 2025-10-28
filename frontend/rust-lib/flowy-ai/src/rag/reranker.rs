use crate::rag::hybrid_retriever::HybridRetrievedDocument;
use flowy_error::FlowyResult;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// 重排序器配置
#[derive(Debug, Clone)]
pub struct RerankerConfig {
    /// 启用重排序（默认false，初始版本仅按分数排序）
    pub enabled: bool,
    
    /// 返回结果的最大数量
    pub top_k: usize,
    
    /// 最小相关度阈值
    pub score_threshold: f32,
    
    /// 是否使用外部模型进行重排序（预留接口）
    pub use_model: bool,
    
    /// 模型超参数（预留接口）
    pub model_params: HashMap<String, String>,
}

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            top_k: 10,
            score_threshold: 0.0,
            use_model: false,
            model_params: HashMap::new(),
        }
    }
}

/// 重排序器
///
/// 对检索结果进行重新排序以提高相关性。
/// 
/// 初始版本：
/// - 根据 `combined_score` 对结果进行降序排序
/// - 过滤低分文档
/// - 限制返回数量
///
/// 未来扩展：
/// - 支持外部重排序模型（如 CrossEncoder）
/// - 支持多样性重排序
/// - 支持个性化排序
pub struct Reranker {
    config: RerankerConfig,
}

impl Reranker {
    /// 创建新的重排序器
    pub fn new(config: RerankerConfig) -> Self {
        Self { config }
    }
    
    /// 对检索结果进行重排序
    ///
    /// # 参数
    /// - `candidates`: 混合检索返回的候选文档
    /// - `query`: 查询文本（用于未来模型重排序）
    ///
    /// # 返回
    /// 排序并过滤后的文档列表
    pub async fn rerank(
        &self,
        candidates: Vec<HybridRetrievedDocument>,
        _query: &str,
    ) -> FlowyResult<Vec<HybridRetrievedDocument>> {
        // 如果重排序被禁用，直接返回原始结果
        if !self.config.enabled {
            debug!("[Reranker] Reranking is disabled, returning original results");
            return Ok(candidates);
        }
        
        // 处理空结果集
        if candidates.is_empty() {
            warn!("[Reranker] Received empty candidate set");
            return Ok(Vec::new());
        }
        
        info!(
            "[Reranker] Starting reranking: {} candidates, top_k={}, threshold={}",
            candidates.len(),
            self.config.top_k,
            self.config.score_threshold
        );
        
        // 复制并过滤候选文档
        let mut reranked: Vec<HybridRetrievedDocument> = candidates
            .into_iter()
            .filter(|doc| doc.combined_score >= self.config.score_threshold)
            .collect();
        
        // 检查过滤后是否还有结果
        if reranked.is_empty() {
            warn!("[Reranker] All candidates filtered by threshold");
            return Ok(Vec::new());
        }
        
        // 根据配置选择重排序策略
        if self.config.use_model {
            self.rerank_with_model(&mut reranked, _query).await?;
        } else {
            self.rerank_by_score(&mut reranked);
        }
        
        // 限制返回数量
        let top_k = self.config.top_k.min(reranked.len());
        reranked.truncate(top_k);
        
        info!("[Reranker] Reranking complete: {} results returned", reranked.len());
        Ok(reranked)
    }
    
    /// 按分数排序（默认策略）
    fn rerank_by_score(&self, docs: &mut Vec<HybridRetrievedDocument>) {
        docs.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        debug!("[Reranker] Sorted {} documents by combined_score", docs.len());
    }
    
    /// 使用外部模型进行重排序（预留接口）
    ///
    /// 当前实现仅记录日志，实际模型集成可以在这里添加。
    ///
    /// 示例实现思路：
    /// 1. 调用外部重排序服务（如 CrossEncoder API）
    /// 2. 使用 query-document 对计算相关性分数
    /// 3. 用新分数替换或调整 combined_score
    /// 4. 按新分数排序
    async fn rerank_with_model(
        &self,
        docs: &mut Vec<HybridRetrievedDocument>,
        query: &str,
    ) -> FlowyResult<()> {
        warn!(
            "[Reranker] Model-based reranking requested but not implemented yet. Query: '{}', Docs: {}",
            query,
            docs.len()
        );
        
        // TODO: 实现模型重排序
        // 示例：
        // let model_scores = await call_reranking_model(query, docs);
        // for (doc, model_score) in docs.iter_mut().zip(model_scores.iter()) {
        //     doc.combined_score = model_score;
        // }
        
        // 暂时回退到分数排序
        self.rerank_by_score(docs);
        Ok(())
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: RerankerConfig) {
        self.config = config;
        debug!("[Reranker] Configuration updated");
    }
    
    /// 获取当前配置
    pub fn get_config(&self) -> &RerankerConfig {
        &self.config
    }
}

/// 构建器模式用于配置重排序器
pub struct RerankerBuilder {
    config: RerankerConfig,
}

impl RerankerBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: RerankerConfig::default(),
        }
    }
    
    /// 设置是否启用重排序
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }
    
    /// 设置返回结果数量
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.config.top_k = top_k;
        self
    }
    
    /// 设置最小分数阈值
    pub fn with_score_threshold(mut self, threshold: f32) -> Self {
        self.config.score_threshold = threshold;
        self
    }
    
    /// 设置是否使用外部模型（预留接口）
    pub fn with_model(mut self, use_model: bool) -> Self {
        self.config.use_model = use_model;
        self
    }
    
    /// 设置模型参数（预留接口）
    pub fn with_model_param(mut self, key: String, value: String) -> Self {
        self.config.model_params.insert(key, value);
        self
    }
    
    /// 构建重排序器
    pub fn build(self) -> Reranker {
        Reranker::new(self.config)
    }
}

impl Default for RerankerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    fn create_test_document(document_id: &str, score: f32) -> HybridRetrievedDocument {
        HybridRetrievedDocument {
            document_id: document_id.to_string(),
            content: format!("Content for {}", document_id),
            metadata: HashMap::new(),
            vector_score: Some(score),
            keyword_score: Some(score),
            combined_score: score,
        }
    }
    
    #[tokio::test]
    async fn test_rerank_empty_list() {
        let reranker = Reranker::new(RerankerConfig::default());
        let result = reranker.rerank(Vec::new(), "test").await.unwrap();
        assert_eq!(result.len(), 0);
    }
    
    #[tokio::test]
    async fn test_rerank_sorts_by_score() {
        let config = RerankerConfig {
            enabled: true,
            top_k: 10,
            score_threshold: 0.0,
            use_model: false,
            model_params: HashMap::new(),
        };
        let reranker = Reranker::new(config);
        
        let mut candidates = vec![
            create_test_document("doc1", 0.3),
            create_test_document("doc2", 0.9),
            create_test_document("doc3", 0.5),
        ];
        
        let results = reranker.rerank(candidates, "test").await.unwrap();
        
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].document_id, "doc2");
        assert_eq!(results[0].combined_score, 0.9);
        assert_eq!(results[1].document_id, "doc3");
        assert_eq!(results[2].document_id, "doc1");
    }
    
    #[tokio::test]
    async fn test_rerank_applies_score_threshold() {
        let config = RerankerConfig {
            enabled: true,
            top_k: 10,
            score_threshold: 0.4,
            use_model: false,
            model_params: HashMap::new(),
        };
        let reranker = Reranker::new(config);
        
        let candidates = vec![
            create_test_document("doc1", 0.3),
            create_test_document("doc2", 0.9),
            create_test_document("doc3", 0.5),
        ];
        
        let results = reranker.rerank(candidates, "test").await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].document_id, "doc2");
        assert_eq!(results[1].document_id, "doc3");
    }
    
    #[tokio::test]
    async fn test_rerank_applies_top_k() {
        let config = RerankerConfig {
            enabled: true,
            top_k: 2,
            score_threshold: 0.0,
            use_model: false,
            model_params: HashMap::new(),
        };
        let reranker = Reranker::new(config);
        
        let candidates = vec![
            create_test_document("doc1", 0.3),
            create_test_document("doc2", 0.9),
            create_test_document("doc3", 0.5),
        ];
        
        let results = reranker.rerank(candidates, "test").await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].document_id, "doc2");
        assert_eq!(results[1].document_id, "doc3");
    }
    
    #[tokio::test]
    async fn test_rerank_disabled() {
        let config = RerankerConfig {
            enabled: false,
            top_k: 10,
            score_threshold: 0.0,
            use_model: false,
            model_params: HashMap::new(),
        };
        let reranker = Reranker::new(config);
        
        let candidates = vec![
            create_test_document("doc1", 0.3),
            create_test_document("doc2", 0.9),
        ];
        
        let results = reranker.rerank(candidates.clone(), "test").await.unwrap();
        
        // 当disabled时，应该返回原始顺序
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].document_id, "doc1");
        assert_eq!(results[1].document_id, "doc2");
    }
    
    #[tokio::test]
    async fn test_rerank_preserves_metadata() {
        let config = RerankerConfig {
            enabled: true,
            top_k: 10,
            score_threshold: 0.0,
            use_model: false,
            model_params: HashMap::new(),
        };
        let reranker = Reranker::new(config);
        
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        
        let mut candidates = vec![
            create_test_document("doc1", 0.3),
            create_test_document("doc2", 0.9),
        ];
        candidates[0].metadata = metadata.clone();
        
        let results = reranker.rerank(candidates, "test").await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[1].metadata.get("test_key"), Some(&"test_value".to_string()));
    }
    
    #[test]
    fn test_reranker_builder() {
        let reranker = RerankerBuilder::new()
            .with_enabled(true)
            .with_top_k(5)
            .with_score_threshold(0.3)
            .build();
        
        let config = reranker.get_config();
        assert_eq!(config.enabled, true);
        assert_eq!(config.top_k, 5);
        assert_eq!(config.score_threshold, 0.3);
    }
    
    #[tokio::test]
    async fn test_rerank_all_filtered() {
        let config = RerankerConfig {
            enabled: true,
            top_k: 10,
            score_threshold: 0.99,
            use_model: false,
            model_params: HashMap::new(),
        };
        let reranker = Reranker::new(config);
        
        let candidates = vec![
            create_test_document("doc1", 0.3),
            create_test_document("doc2", 0.5),
        ];
        
        let results = reranker.rerank(candidates, "test").await.unwrap();
        
        assert_eq!(results.len(), 0);
    }
}
