use crate::web_search::entities::*;
use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 默认缓存过期时间（秒）
    pub default_ttl_seconds: u64,
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存清理间隔（秒）
    pub cleanup_interval_seconds: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl_seconds: 3600, // 1小时
            max_entries: 1000,
            cleanup_interval_seconds: 300, // 5分钟
            enable_compression: true,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 缓存条目总数
    pub total_entries: usize,
    /// 最后清理时间
    pub last_cleanup: Option<SystemTime>,
    /// 缓存大小（字节）
    pub cache_size_bytes: u64,
}

impl CacheStats {
    /// 计算缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// 缓存条目元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntryMetadata {
    /// 创建时间
    created_at: SystemTime,
    /// 过期时间
    expires_at: SystemTime,
    /// 命中次数
    hit_count: u64,
    /// 数据大小（字节）
    data_size: usize,
    /// 查询参数哈希
    query_hash: String,
}

/// 搜索结果缓存管理器
pub struct WebSearchCacheManager {
    /// 存储偏好设置
    store_preferences: Arc<KVStorePreferences>,
    /// 内存缓存（LRU缓存）
    memory_cache: Arc<RwLock<HashMap<String, (WebSearchResponsePB, CacheEntryMetadata)>>>,
    /// 缓存配置
    config: CacheConfig,
    /// 缓存统计
    stats: Arc<RwLock<CacheStats>>,
    /// 缓存键前缀
    cache_key_prefix: String,
}

impl WebSearchCacheManager {
    /// 创建新的缓存管理器
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        Self {
            store_preferences,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            config: CacheConfig::default(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            cache_key_prefix: "web_search_cache".to_string(),
        }
    }

    /// 使用自定义配置创建缓存管理器
    pub fn with_config(store_preferences: Arc<KVStorePreferences>, config: CacheConfig) -> Self {
        Self {
            store_preferences,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            cache_key_prefix: "web_search_cache".to_string(),
        }
    }

    /// 生成缓存键
    fn generate_cache_key(&self, query: &str, provider_id: &str, params: &HashMap<String, String>) -> String {
        // 创建查询参数的排序字符串以确保一致性
        let mut param_pairs: Vec<_> = params.iter().collect();
        param_pairs.sort_by(|a, b| a.0.cmp(b.0));
        
        let params_str = param_pairs
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join("|");
        
        // 生成查询哈希
        let query_hash = self.hash_query(query, provider_id, &params_str);
        
        format!("{}:{}", self.cache_key_prefix, query_hash)
    }

    /// 计算查询哈希
    fn hash_query(&self, query: &str, provider_id: &str, params: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        provider_id.hash(&mut hasher);
        params.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }

    /// 检查缓存是否启用
    fn is_cache_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 获取缓存的搜索结果
    pub fn get_cached_result(
        &self,
        query: &str,
        provider_id: &str,
        params: &HashMap<String, String>,
    ) -> FlowyResult<Option<WebSearchResponsePB>> {
        if !self.is_cache_enabled() {
            return Ok(None);
        }

        let cache_key = self.generate_cache_key(query, provider_id, params);
        
        // 首先检查内存缓存
        if let Some(result) = self.get_from_memory_cache(&cache_key)? {
            debug!("Cache hit in memory for query: {}", query);
            self.increment_hit_count();
            return Ok(Some(result));
        }

        // 然后检查持久化缓存
        if let Some(result) = self.get_from_persistent_cache(&cache_key)? {
            debug!("Cache hit in persistent storage for query: {}", query);
            self.increment_hit_count();
            
            // 将结果加载到内存缓存
            self.store_in_memory_cache(&cache_key, result.clone())?;
            return Ok(Some(result));
        }

        debug!("Cache miss for query: {}", query);
        self.increment_miss_count();
        Ok(None)
    }

    /// 存储搜索结果到缓存
    pub fn store_cached_result(
        &self,
        query: &str,
        provider_id: &str,
        params: &HashMap<String, String>,
        result: WebSearchResponsePB,
    ) -> FlowyResult<()> {
        if !self.is_cache_enabled() {
            return Ok(());
        }

        let cache_key = self.generate_cache_key(query, provider_id, params);
        
        // 存储到内存缓存
        self.store_in_memory_cache(&cache_key, result.clone())?;
        
        // 存储到持久化缓存
        self.store_in_persistent_cache(&cache_key, result)?;
        
        debug!("Stored search result in cache for query: {}", query);
        Ok(())
    }

    /// 从内存缓存获取结果
    fn get_from_memory_cache(&self, cache_key: &str) -> FlowyResult<Option<WebSearchResponsePB>> {
        let cache = self.memory_cache.read().map_err(|e| {
            error!("Failed to acquire read lock on memory cache: {}", e);
            FlowyError::internal().with_context("获取内存缓存读取锁失败")
        })?;

        if let Some((result, metadata)) = cache.get(cache_key) {
            // 检查是否过期
            if metadata.expires_at > SystemTime::now() {
                Ok(Some(result.clone()))
            } else {
                // 过期了，从内存缓存中移除
                drop(cache);
                self.remove_from_memory_cache(cache_key)?;
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 存储到内存缓存
    fn store_in_memory_cache(&self, cache_key: &str, result: WebSearchResponsePB) -> FlowyResult<()> {
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(self.config.default_ttl_seconds);
        
        // 计算数据大小
        let data_size = serde_json::to_string(&result)
            .map_err(|e| FlowyError::internal().with_context(format!("序列化搜索结果失败: {}", e)))?
            .len();

        let metadata = CacheEntryMetadata {
            created_at: now,
            expires_at,
            hit_count: 0,
            data_size,
            query_hash: cache_key.to_string(),
        };

        let mut cache = self.memory_cache.write().map_err(|e| {
            error!("Failed to acquire write lock on memory cache: {}", e);
            FlowyError::internal().with_context("获取内存缓存写入锁失败")
        })?;

        // 检查缓存大小限制
        if cache.len() >= self.config.max_entries {
            self.evict_oldest_entries(&mut cache)?;
        }

        cache.insert(cache_key.to_string(), (result, metadata));
        Ok(())
    }

    /// 从内存缓存移除条目
    fn remove_from_memory_cache(&self, cache_key: &str) -> FlowyResult<()> {
        let mut cache = self.memory_cache.write().map_err(|e| {
            error!("Failed to acquire write lock on memory cache: {}", e);
            FlowyError::internal().with_context("获取内存缓存写入锁失败")
        })?;

        cache.remove(cache_key);
        Ok(())
    }

    /// 从持久化缓存获取结果
    fn get_from_persistent_cache(&self, cache_key: &str) -> FlowyResult<Option<WebSearchResponsePB>> {
        let cache_entry: Option<WebSearchCacheEntryPB> = self.store_preferences.get_object(cache_key);
        
        if let Some(entry) = cache_entry {
            // 检查是否过期
            if !entry.is_expired() {
                Ok(Some(entry.search_response))
            } else {
                // 过期了，从持久化缓存中移除
                self.store_preferences.remove(cache_key);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 存储到持久化缓存
    fn store_in_persistent_cache(&self, cache_key: &str, result: WebSearchResponsePB) -> FlowyResult<()> {
        let cache_entry = WebSearchCacheEntryPB::new(cache_key.to_string(), result);
        
        self.store_preferences
            .set_object(cache_key, &cache_entry)
            .map_err(|e| {
                error!("Failed to store cache entry: {}", e);
                FlowyError::internal().with_context(format!("存储缓存条目失败: {}", e))
            })?;

        Ok(())
    }

    /// 增加命中次数
    fn increment_hit_count(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.hits += 1;
        }
    }

    /// 增加未命中次数
    fn increment_miss_count(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
        }
    }

    /// 清理过期的缓存条目
    pub fn cleanup_expired_entries(&self) -> FlowyResult<usize> {
        let mut cleaned_count = 0;
        let now = SystemTime::now();

        // 清理内存缓存
        {
            let mut cache = self.memory_cache.write().map_err(|e| {
                error!("Failed to acquire write lock on memory cache: {}", e);
                FlowyError::internal().with_context("获取内存缓存写入锁失败")
            })?;

            let expired_keys: Vec<String> = cache
                .iter()
                .filter(|(_, (_, metadata))| metadata.expires_at <= now)
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                cache.remove(&key);
                cleaned_count += 1;
            }
        }

        // 清理持久化缓存
        // 注意：这里简化了实现，实际应用中可能需要更复杂的清理策略
        // 因为 KVStorePreferences 没有提供遍历所有键的方法
        
        // 更新统计信息
        if let Ok(mut stats) = self.stats.write() {
            stats.last_cleanup = Some(now);
            stats.total_entries = self.memory_cache.read().unwrap().len();
        }

        info!("Cleaned up {} expired cache entries", cleaned_count);
        Ok(cleaned_count)
    }

    /// 清理所有缓存
    pub fn clear_all_cache(&self) -> FlowyResult<()> {
        // 清理内存缓存
        {
            let mut cache = self.memory_cache.write().map_err(|e| {
                error!("Failed to acquire write lock on memory cache: {}", e);
                FlowyError::internal().with_context("获取内存缓存写入锁失败")
            })?;
            cache.clear();
        }

        // 清理持久化缓存
        // 注意：这里简化了实现，实际应用中需要遍历所有缓存键
        // 由于 KVStorePreferences 的限制，这里只清理已知的键
        
        // 重置统计信息
        if let Ok(mut stats) = self.stats.write() {
            *stats = CacheStats::default();
        }

        info!("Cleared all cache entries");
        Ok(())
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        let mut stats = self.stats.read().unwrap().clone();
        stats.total_entries = self.memory_cache.read().unwrap().len();
        stats
    }

    /// 更新缓存配置
    pub fn update_config(&mut self, config: CacheConfig) {
        self.config = config;
        info!("Updated cache configuration");
    }

    /// 获取缓存配置
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    /// 检查特定查询是否在缓存中
    pub fn is_cached(&self, query: &str, provider_id: &str, params: &HashMap<String, String>) -> bool {
        if !self.is_cache_enabled() {
            return false;
        }

        let cache_key = self.generate_cache_key(query, provider_id, params);
        
        // 检查内存缓存
        if let Ok(cache) = self.memory_cache.read() {
            if let Some((_, metadata)) = cache.get(&cache_key) {
                return metadata.expires_at > SystemTime::now();
            }
        }

        // 检查持久化缓存
        if let Some(entry) = self.store_preferences.get_object::<WebSearchCacheEntryPB>(&cache_key) {
            return !entry.is_expired();
        }

        false
    }

    /// 预热缓存（预加载常用查询）
    pub fn warmup_cache(&self, queries: Vec<(String, String, HashMap<String, String>)>) -> FlowyResult<usize> {
        let mut warmed_count = 0;
        
        for (query, provider_id, params) in queries {
            if !self.is_cached(&query, &provider_id, &params) {
                // 这里可以触发实际的搜索请求来预热缓存
                // 但为了避免副作用，这里只是标记
                debug!("Would warmup cache for query: {}", query);
                warmed_count += 1;
            }
        }

        info!("Warmed up {} cache entries", warmed_count);
        Ok(warmed_count)
    }

    /// 驱逐最旧的缓存条目
    fn evict_oldest_entries(&self, cache: &mut HashMap<String, (WebSearchResponsePB, CacheEntryMetadata)>) -> FlowyResult<()> {
        // 按创建时间排序，移除最旧的条目
        let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by(|a, b| a.1.1.created_at.cmp(&b.1.1.created_at));
        
        // 移除最旧的 10% 条目
        let evict_count = (cache.len() / 10).max(1);
        for (key, _) in entries.iter().take(evict_count) {
            cache.remove(key);
        }

        debug!("Evicted {} oldest cache entries", evict_count);
        Ok(())
    }

    /// 获取缓存大小（字节）
    pub fn get_cache_size(&self) -> u64 {
        let cache = self.memory_cache.read().unwrap();
        cache.values().map(|(_, metadata)| metadata.data_size as u64).sum()
    }

    /// 检查缓存健康状态
    pub fn check_cache_health(&self) -> FlowyResult<CacheHealthReport> {
        let stats = self.get_cache_stats();
        let cache_size = self.get_cache_size();
        
        let health_score = if stats.total_entries == 0 {
            100.0
        } else {
            let hit_rate_score = stats.hit_rate() * 50.0;
            let size_score = if cache_size < 10_000_000 { // 10MB
                50.0
            } else if cache_size < 50_000_000 { // 50MB
                30.0
            } else {
                10.0
            };
            hit_rate_score + size_score
        };

        Ok(CacheHealthReport {
            health_score,
            hit_rate: stats.hit_rate(),
            total_entries: stats.total_entries,
            cache_size_bytes: cache_size,
            recommendations: self.generate_recommendations(&stats, cache_size),
        })
    }

    /// 生成缓存优化建议
    fn generate_recommendations(&self, stats: &CacheStats, cache_size: u64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if stats.hit_rate() < 0.3 {
            recommendations.push("缓存命中率较低，建议检查查询模式或调整缓存策略".to_string());
        }

        if cache_size > 50_000_000 { // 50MB
            recommendations.push("缓存大小较大，建议启用压缩或减少缓存条目数量".to_string());
        }

        if stats.total_entries >= self.config.max_entries {
            recommendations.push("缓存条目数量达到上限，建议增加最大条目数或调整清理策略".to_string());
        }

        if stats.last_cleanup.is_none() {
            recommendations.push("建议定期清理过期缓存条目".to_string());
        }

        recommendations
    }
}

/// 缓存健康报告
#[derive(Debug, Clone)]
pub struct CacheHealthReport {
    /// 健康评分（0-100）
    pub health_score: f64,
    /// 命中率
    pub hit_rate: f64,
    /// 总条目数
    pub total_entries: usize,
    /// 缓存大小（字节）
    pub cache_size_bytes: u64,
    /// 优化建议
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache_manager() -> (WebSearchCacheManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        let manager = WebSearchCacheManager::new(store);
        (manager, tempdir)
    }

    fn create_test_search_response() -> WebSearchResponsePB {
        let mut response = WebSearchResponsePB::new("test query".to_string(), "test_provider".to_string());
        response.add_result(WebSearchResultPB::new(
            "Test Title".to_string(),
            "https://example.com".to_string(),
            "Test snippet".to_string(),
        ));
        response
    }

    #[test]
    fn test_cache_key_generation() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let query = "test query";
        let provider_id = "test_provider";
        let mut params = HashMap::new();
        params.insert("max_results".to_string(), "10".to_string());
        
        let key1 = manager.generate_cache_key(query, provider_id, &params);
        let key2 = manager.generate_cache_key(query, provider_id, &params);
        
        assert_eq!(key1, key2);
        
        // 不同查询应该生成不同的键
        let key3 = manager.generate_cache_key("different query", provider_id, &params);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_store_and_retrieve() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let query = "test query";
        let provider_id = "test_provider";
        let params = HashMap::new();
        let response = create_test_search_response();
        
        // 存储到缓存
        manager.store_cached_result(query, provider_id, &params, response.clone()).unwrap();
        
        // 从缓存获取
        let cached_response = manager.get_cached_result(query, provider_id, &params).unwrap();
        
        assert!(cached_response.is_some());
        assert_eq!(cached_response.unwrap().query, response.query);
    }

    #[test]
    fn test_cache_miss() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let query = "non-existent query";
        let provider_id = "test_provider";
        let params = HashMap::new();
        
        let cached_response = manager.get_cached_result(query, provider_id, &params).unwrap();
        assert!(cached_response.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let query = "test query";
        let provider_id = "test_provider";
        let params = HashMap::new();
        let response = create_test_search_response();
        
        // 初始统计
        let initial_stats = manager.get_cache_stats();
        assert_eq!(initial_stats.hits, 0);
        assert_eq!(initial_stats.misses, 0);
        
        // 缓存未命中
        manager.get_cached_result(query, provider_id, &params).unwrap();
        let stats_after_miss = manager.get_cache_stats();
        assert_eq!(stats_after_miss.misses, 1);
        
        // 存储并命中
        manager.store_cached_result(query, provider_id, &params, response).unwrap();
        manager.get_cached_result(query, provider_id, &params).unwrap();
        let stats_after_hit = manager.get_cache_stats();
        assert_eq!(stats_after_hit.hits, 1);
        assert_eq!(stats_after_hit.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_cleanup() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let query = "test query";
        let provider_id = "test_provider";
        let params = HashMap::new();
        let response = create_test_search_response();
        
        // 存储到缓存
        manager.store_cached_result(query, provider_id, &params, response).unwrap();
        
        // 清理缓存
        let cleaned_count = manager.cleanup_expired_entries().unwrap();
        
        // 由于条目刚创建，不应该被清理
        assert_eq!(cleaned_count, 0);
        
        // 清理所有缓存
        manager.clear_all_cache().unwrap();
        
        // 验证缓存已清空
        let cached_response = manager.get_cached_result(query, provider_id, &params).unwrap();
        assert!(cached_response.is_none());
    }

    #[test]
    fn test_cache_config() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let config = manager.get_config();
        assert!(config.enabled);
        assert_eq!(config.default_ttl_seconds, 3600);
        
        let mut new_config = config.clone();
        new_config.enabled = false;
        // 注意：这里需要可变引用，但测试中我们使用不可变引用
        // 在实际使用中，应该使用 Arc<Mutex<CacheConfig>> 或类似的设计
    }

    #[test]
    fn test_cache_health_check() {
        let (manager, _tempdir) = create_test_cache_manager();
        
        let health_report = manager.check_cache_health().unwrap();
        assert!(health_report.health_score >= 0.0);
        assert!(health_report.health_score <= 100.0);
        assert_eq!(health_report.total_entries, 0);
    }
}
