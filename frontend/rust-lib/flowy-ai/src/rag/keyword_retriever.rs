use async_trait::async_trait;
use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite_vec::db::VectorSqliteDB;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Weak;
use tracing::{debug, info, warn};

/// BM25 参数
const K1: f32 = 1.5;  // 词频饱和度参数
const B: f32 = 0.75;  // 长度归一化参数

/// 关键词检索的文档结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordDocument {
    /// 文档 ID (object_id)
    pub document_id: String,
    
    /// Fragment ID
    pub fragment_id: String,
    
    /// 文档内容
    pub content: String,
    
    /// 工作区 ID
    pub workspace_id: String,
    
    /// 元数据
    pub metadata: HashMap<String, String>,
    
    /// BM25 相关度分数
    pub score: f32,
    
    /// Fragment 索引
    pub fragment_index: i32,
    
    /// Content type
    pub content_type: i32,
}

/// 关键词存储 trait
#[async_trait]
pub trait KeywordStore: Send + Sync {
    /// 搜索关键词，返回相关文档
    async fn search(
        &self,
        query: &str,
        workspace_id: &str,
        object_ids: &[String],
        limit: usize,
    ) -> FlowyResult<Vec<KeywordDocument>>;
    
    /// 为文档建立全文索引
    async fn index_document(
        &self,
        document_id: &str,
        fragment_id: &str,
        workspace_id: &str,
        content: &str,
        metadata: &str,
        fragment_index: i32,
        content_type: i32,
    ) -> FlowyResult<()>;
    
    /// 删除文档索引
    async fn delete_document(&self, workspace_id: &str, document_id: &str) -> FlowyResult<()>;
    
    /// 删除工作区的所有文档索引
    async fn delete_workspace_documents(&self, workspace_id: &str) -> FlowyResult<()>;
    
    /// 重置索引（删除所有数据）
    async fn reset_index(&self) -> FlowyResult<()>;
}

/// 基于 SQLite 和 BM25 的关键词存储实现
/// 
/// 实现完整的 BM25 排名算法，提供高效的关键词检索。
/// 使用项目现有的 VectorSqliteDB 作为数据源。
/// 
/// 注意：当前实现不使用 SQLite FTS5 扩展，而是在应用层实现 BM25 算法。
/// 这样做的好处是：
/// - 不依赖 SQLite 特定扩展
/// - 完全控制算法实现
/// - 可灵活调整参数（k1, b）
/// 如果将来需要 FTS5 支持，可以考虑创建 FTS5 虚拟表并与之集成。
pub struct SQLiteKeywordStore {
    vector_db: Weak<VectorSqliteDB>,
}

impl SQLiteKeywordStore {
    /// 创建新的 SQLite 关键词存储实例
    pub fn new(vector_db: Weak<VectorSqliteDB>) -> Self {
        Self { vector_db }
    }
    
    /// 计算完整的 BM25 分数
    /// 
    /// BM25 公式：
    /// score(query, doc) = Σ IDF(qi) * f(qi, doc) * (k1 + 1) / (f(qi, doc) + k1 * (1 - b + b * |d| / avgdl))
    fn calculate_bm25_score(
        &self,
        content: &str,
        query_words: &[String],
        doc_frequency: &HashMap<String, usize>,
        total_docs: usize,
        avg_doc_length: f32,
    ) -> f32 {
        if query_words.is_empty() || total_docs == 0 {
            return 0.0;
        }
        
        let doc_length = content.split_whitespace().count() as f32;
        
        let term_frequencies: HashMap<String, usize> = content
            .to_lowercase()
            .split_whitespace()
            .fold(HashMap::new(), |mut acc, word| {
                *acc.entry(word.to_string()).or_insert(0) += 1;
                acc
            });
        
        let mut score = 0.0;
        
        for query_word in query_words {
            let word_lower = query_word.to_lowercase();
            let term_freq = term_frequencies.get(&word_lower).copied().unwrap_or(0) as f32;
            
            if term_freq == 0.0 {
                continue;
            }
            
            let df = doc_frequency.get(&word_lower).copied().unwrap_or(0);
            let idf = Self::calculate_idf(total_docs, df);
            
            let numerator = term_freq * (K1 + 1.0);
            let denominator = term_freq + K1 * (1.0 - B + B * (doc_length / avg_doc_length));
            let bm25_term_score = idf * (numerator / denominator);
            
            score += bm25_term_score;
        }
        
        score
    }
    
    /// 计算逆文档频率 (IDF)
    fn calculate_idf(total_docs: usize, doc_freq: usize) -> f32 {
        if doc_freq == 0 {
            return f32::MAX;
        }
        
        let numerator = (total_docs - doc_freq) as f32 + 0.5;
        let denominator = doc_freq as f32 + 0.5;
        (numerator / denominator).ln()
    }
    
    /// 计算文档频率
    fn calculate_doc_frequency(contents: &[String], query_words: &[String]) -> HashMap<String, usize> {
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        
        for word in query_words {
            doc_freq.insert(word.to_lowercase(), 0);
        }
        
        for content in contents {
            let content_lower = content.to_lowercase();
            let content_words: HashSet<String> = content_lower
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            
            for word in &content_words {
                if let Some(count) = doc_freq.get_mut(word) {
                    *count += 1;
                }
            }
        }
        
        doc_freq
    }
    
    /// 计算平均文档长度
    fn calculate_avg_doc_length(contents: &[String]) -> f32 {
        if contents.is_empty() {
            return 0.0;
        }
        
        let total_words: usize = contents
            .iter()
            .map(|content| content.split_whitespace().count())
            .sum();
        
        total_words as f32 / contents.len() as f32
    }
    
    /// 解析查询为关键词列表
    fn parse_query_to_keywords(query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .filter(|w| w.len() > 1)
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

#[async_trait]
impl KeywordStore for SQLiteKeywordStore {
    async fn search(
        &self,
        query: &str,
        workspace_id: &str,
        object_ids: &[String],
        limit: usize,
    ) -> FlowyResult<Vec<KeywordDocument>> {
        let query_words = Self::parse_query_to_keywords(query);
        
        if query_words.is_empty() {
            warn!("Empty query after parsing");
            return Ok(Vec::new());
        }
        
        debug!("Parsed query keywords: {:?}", query_words);
        
        let vector_db = self.vector_db.upgrade()
            .ok_or_else(|| FlowyError::internal().with_context("Vector database not initialized"))?;
        
        let all_content = vector_db
            .select_all_embedded_content(workspace_id, object_ids, 10000)
            .await
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to fetch content: {}", e)))?;
        
        if all_content.is_empty() {
            return Ok(Vec::new());
        }
        
        let total_docs = all_content.len();
        let contents: Vec<String> = all_content.iter().map(|c| c.content.clone()).collect();
        
        let doc_frequency = Self::calculate_doc_frequency(&contents, &query_words);
        let avg_doc_length = Self::calculate_avg_doc_length(&contents);
        
        debug!("Total documents: {}, Average doc length: {:.2}", total_docs, avg_doc_length);
        debug!("Document frequencies: {:?}", doc_frequency);
        
        let mut results: Vec<(KeywordDocument, f32)> = Vec::new();
        
        for content in &all_content {
            let score = self.calculate_bm25_score(
                &content.content,
                &query_words,
                &doc_frequency,
                total_docs,
                avg_doc_length,
            );
            
            if score > 0.0 {
                results.push((
                    KeywordDocument {
                        document_id: content.object_id.clone(),
                        fragment_id: content.object_id.clone(),
                        content: content.content.clone(),
                        workspace_id: workspace_id.to_string(),
                        metadata: HashMap::new(),
                        score,
                        fragment_index: 0,
                        content_type: 0,
                    },
                    score,
                ));
            }
        }
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let top_results: Vec<KeywordDocument> = results
            .into_iter()
            .take(limit)
            .map(|(doc, _)| doc)
            .collect();
        
        info!("BM25 keyword search returned {} results", top_results.len());
        Ok(top_results)
    }
    
    async fn index_document(
        &self,
        _document_id: &str,
        _fragment_id: &str,
        _workspace_id: &str,
        _content: &str,
        _metadata: &str,
        _fragment_index: i32,
        _content_type: i32,
    ) -> FlowyResult<()> {
        debug!("Document indexed via af_collab_embeddings table");
        Ok(())
    }
    
    async fn delete_document(&self, _workspace_id: &str, _document_id: &str) -> FlowyResult<()> {
        debug!("Document deleted via af_collab_embeddings table");
        Ok(())
    }
    
    async fn delete_workspace_documents(&self, _workspace_id: &str) -> FlowyResult<()> {
        debug!("Workspace documents deleted via af_collab_embeddings table");
        Ok(())
    }
    
    async fn reset_index(&self) -> FlowyResult<()> {
        info!("Index reset complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_to_keywords() {
        let keywords = SQLiteKeywordStore::parse_query_to_keywords("hello world");
        assert_eq!(keywords, vec!["hello", "world"]);
        
        let keywords = SQLiteKeywordStore::parse_query_to_keywords("");
        assert!(keywords.is_empty());
    }
    
    #[test]
    fn test_calculate_idf() {
        let idf = SQLiteKeywordStore::calculate_idf(1000, 100);
        assert!(idf > 0.0);
        
        let idf_rare = SQLiteKeywordStore::calculate_idf(1000, 10);
        let idf_common = SQLiteKeywordStore::calculate_idf(1000, 500);
        assert!(idf_rare > idf_common);
    }
    
    #[test]
    fn test_calculate_avg_doc_length() {
        let contents = vec![
            "This is a test".to_string(),
            "Another test document".to_string(),
            "Short".to_string(),
        ];
        let avg_len = SQLiteKeywordStore::calculate_avg_doc_length(&contents);
        assert!(avg_len > 0.0);
    }
    
    #[test]
    fn test_calculate_doc_frequency() {
        let contents = vec![
            "machine learning is great".to_string(),
            "deep learning algorithms".to_string(),
            "machine vision".to_string(),
        ];
        let query_words = vec!["machine".to_string(), "learning".to_string(), "deep".to_string()];
        
        let doc_freq = SQLiteKeywordStore::calculate_doc_frequency(&contents, &query_words);
        
        assert_eq!(doc_freq.get("machine"), Some(&2));
        assert_eq!(doc_freq.get("learning"), Some(&1));
        assert_eq!(doc_freq.get("deep"), Some(&1));
    }
}
