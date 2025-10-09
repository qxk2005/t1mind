use crate::web_search::entities::*;
use crate::web_search::{CacheConfig, CacheStats};
use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::prelude::*;
use flowy_sqlite::schema::{web_search_cache_table, web_search_cache_stats_table};
use flowy_sqlite::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// 数据库缓存条目
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = web_search_cache_table)]
pub struct WebSearchCacheEntry {
    pub cache_key: String,
    pub query: String,
    pub provider_id: String,
    pub search_response: String, // JSON序列化的搜索结果
    pub created_at: i64,
    pub expires_at: i64,
    pub hit_count: i64,
    pub metadata: Option<String>,
}

/// 缓存统计条目
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = web_search_cache_stats_table)]
pub struct WebSearchCacheStatsEntry {
    pub id: i32,
    pub stat_name: String,
    pub stat_value: String,
    pub updated_at: i64,
}

/// 增强的搜索结果缓存管理器
pub struct EnhancedWebSearchCacheManager {
    /// 数据库连接
    db_connection: Arc<Database>,
    /// 内存缓存（LRU缓存）
    memory_cache: Arc<RwLock<HashMap<String, (WebSearchResponsePB, CacheEntryMetadata)>>>,
    /// 缓存配置
    config: CacheConfig,
    /// 缓存统计
    stats: Arc<RwLock<CacheStats>>,
    /// 缓存键前缀
    cache_key_prefix: String,
}

/// 缓存条目元数据（用于内存缓存）
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

impl EnhancedWebSearchCacheManager {
    /// 创建新的增强缓存管理器
    pub fn new(db_connection: Arc<Database>) -> Self {
        Self {
            db_connection,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            config: CacheConfig::default(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            cache_key_prefix: "web_search_cache".to_string(),
        }
    }

    /// 使用自定义配置创建增强缓存管理器
    pub fn with_config(db_connection: Arc<Database>, config: CacheConfig) -> Self {
        Self {
            db_connection,
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

        // 然后检查数据库缓存
        if let Some(result) = self.get_from_database_cache(&cache_key)? {
            debug!("Cache hit in database for query: {}", query);
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
        
        // 存储到数据库缓存
        self.store_in_database_cache(&cache_key, query, provider_id, result)?;
        
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

    /// 从数据库缓存获取结果
    fn get_from_database_cache(&self, cache_key_param: &str) -> FlowyResult<Option<WebSearchResponsePB>> {
        use flowy_sqlite::schema::web_search_cache_table::dsl::*;
        
        let mut conn = self.db_connection.get_connection().map_err(|e| {
            error!("Failed to get database connection: {}", e);
            FlowyError::internal().with_context("获取数据库连接失败")
        })?;

        let result: Result<WebSearchCacheEntry, _> = web_search_cache_table
            .filter(cache_key.eq(cache_key_param))
            .first(&mut *conn);

        match result {
            Ok(entry) => {
                // 检查是否过期
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                if entry.expires_at > now {
                    // 更新命中次数
                    self.update_hit_count_in_database(&mut conn, cache_key_param)?;
                    
                    // 反序列化搜索结果
                    let response: WebSearchResponsePB = serde_json::from_str(&entry.search_response)
                        .map_err(|e| FlowyError::internal().with_context(format!("反序列化搜索结果失败: {}", e)))?;
                    
                    Ok(Some(response))
                } else {
                    // 过期了，从数据库中移除
                    self.remove_from_database_cache(&mut conn, cache_key_param)?;
                    Ok(None)
                }
            }
            Err(diesel::NotFound) => Ok(None),
            Err(e) => {
                error!("Database error when getting cache entry: {}", e);
                Err(FlowyError::internal().with_context(format!("数据库查询失败: {}", e)))
            }
        }
    }

    /// 存储到数据库缓存
    fn store_in_database_cache(
        &self,
        cache_key_param: &str,
        query_param: &str,
        provider_id_param: &str,
        result: WebSearchResponsePB,
    ) -> FlowyResult<()> {
        use flowy_sqlite::schema::web_search_cache_table::dsl::*;
        
        let mut conn = self.db_connection.get_connection().map_err(|e| {
            error!("Failed to get database connection: {}", e);
            FlowyError::internal().with_context("获取数据库连接失败")
        })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let expires_at_timestamp = now + self.config.default_ttl_seconds as i64;

        let search_response_json = serde_json::to_string(&result)
            .map_err(|e| FlowyError::internal().with_context(format!("序列化搜索结果失败: {}", e)))?;

        let cache_entry = WebSearchCacheEntry {
            cache_key: cache_key_param.to_string(),
            query: query_param.to_string(),
            provider_id: provider_id_param.to_string(),
            search_response: search_response_json,
            created_at: now,
            expires_at: expires_at_timestamp,
            hit_count: 0,
            metadata: None,
        };

        diesel::replace_into(web_search_cache_table)
            .values(&cache_entry)
            .execute(&mut *conn)
            .map_err(|e| {
                error!("Failed to store cache entry in database: {}", e);
                FlowyError::internal().with_context(format!("存储缓存条目到数据库失败: {}", e))
            })?;

        Ok(())
    }

    /// 更新数据库中的命中次数
    fn update_hit_count_in_database(&self, conn: &mut DBConnection, cache_key: &str) -> FlowyResult<()> {
        use flowy_sqlite::schema::web_search_cache_table::dsl::*;
        
        diesel::update(web_search_cache_table.filter(cache_key.eq(cache_key)))
            .set(hit_count.eq(hit_count + 1))
            .execute(conn)
            .map_err(|e| {
                error!("Failed to update hit count: {}", e);
                FlowyError::internal().with_context(format!("更新命中次数失败: {}", e))
            })?;

        Ok(())
    }

    /// 从数据库缓存移除条目
    fn remove_from_database_cache(&self, conn: &mut DBConnection, cache_key: &str) -> FlowyResult<()> {
        use flowy_sqlite::schema::web_search_cache_table::dsl::*;
        
        diesel::delete(web_search_cache_table.filter(cache_key.eq(cache_key)))
            .execute(conn)
            .map_err(|e| {
                error!("Failed to remove cache entry from database: {}", e);
                FlowyError::internal().with_context(format!("从数据库移除缓存条目失败: {}", e))
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
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 清理内存缓存
        {
            let mut cache = self.memory_cache.write().map_err(|e| {
                error!("Failed to acquire write lock on memory cache: {}", e);
                FlowyError::internal().with_context("获取内存缓存写入锁失败")
            })?;

            let expired_keys: Vec<String> = cache
                .iter()
                .filter(|(_, (_, metadata))| {
                    metadata.expires_at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64 <= now
                })
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                cache.remove(&key);
                cleaned_count += 1;
            }
        }

        // 清理数据库缓存
        {
            use flowy_sqlite::schema::web_search_cache_table::dsl::*;
            
            let mut conn = self.db_connection.get_connection().map_err(|e| {
                error!("Failed to get database connection: {}", e);
                FlowyError::internal().with_context("获取数据库连接失败")
            })?;

            let deleted_count = diesel::delete(web_search_cache_table.filter(expires_at.le(now)))
                .execute(&mut *conn)
                .map_err(|e| {
                    error!("Failed to clean up expired cache entries: {}", e);
                    FlowyError::internal().with_context(format!("清理过期缓存条目失败: {}", e))
                })?;

            cleaned_count += deleted_count;
        }

        // 更新统计信息
        if let Ok(mut stats) = self.stats.write() {
            stats.last_cleanup = Some(SystemTime::now());
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

        // 清理数据库缓存
        {
            use flowy_sqlite::schema::web_search_cache_table::dsl::*;
            
            let mut conn = self.db_connection.get_connection().map_err(|e| {
                error!("Failed to get database connection: {}", e);
                FlowyError::internal().with_context("获取数据库连接失败")
            })?;

            diesel::delete(web_search_cache_table)
                .execute(&mut *conn)
                .map_err(|e| {
                    error!("Failed to clear all cache entries: {}", e);
                    FlowyError::internal().with_context(format!("清理所有缓存条目失败: {}", e))
                })?;
        }

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

        // 检查数据库缓存
        if let Ok(Some(_)) = self.get_from_database_cache(&cache_key) {
            return true;
        }

        false
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

    /// 获取数据库缓存统计
    pub fn get_database_cache_stats(&self) -> FlowyResult<DatabaseCacheStats> {
        use flowy_sqlite::schema::web_search_cache_table::dsl::*;
        
        let mut conn = self.db_connection.get_connection().map_err(|e| {
            error!("Failed to get database connection: {}", e);
            FlowyError::internal().with_context("获取数据库连接失败")
        })?;

        let total_entries: i64 = web_search_cache_table
            .count()
            .get_result(&mut *conn)
            .map_err(|e| {
                error!("Failed to count cache entries: {}", e);
                FlowyError::internal().with_context(format!("统计缓存条目失败: {}", e))
            })?;

        let total_hits: i64 = web_search_cache_table
            .select(hit_count)
            .load::<i64>(&mut *conn)
            .map_err(|e| {
                error!("Failed to sum hit counts: {}", e);
                FlowyError::internal().with_context(format!("统计命中次数失败: {}", e))
            })?
            .iter()
            .sum();

        Ok(DatabaseCacheStats {
            total_entries: total_entries as usize,
            total_hits,
            average_hits_per_entry: if total_entries > 0 { total_hits as f64 / total_entries as f64 } else { 0.0 },
        })
    }
}

/// 数据库缓存统计
#[derive(Debug, Clone)]
pub struct DatabaseCacheStats {
    /// 总条目数
    pub total_entries: usize,
    /// 总命中次数
    pub total_hits: i64,
    /// 平均每个条目的命中次数
    pub average_hits_per_entry: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowy_sqlite::init;
    use tempfile::TempDir;

    fn create_test_enhanced_cache_manager() -> (EnhancedWebSearchCacheManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let db = init(path).unwrap();
        let conn = Arc::new(db.get_connection().unwrap());
        let manager = EnhancedWebSearchCacheManager::new(conn);
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
    fn test_enhanced_cache_store_and_retrieve() {
        let (manager, _tempdir) = create_test_enhanced_cache_manager();
        
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
    fn test_enhanced_cache_cleanup() {
        let (manager, _tempdir) = create_test_enhanced_cache_manager();
        
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
    fn test_database_cache_stats() {
        let (manager, _tempdir) = create_test_enhanced_cache_manager();
        
        let query = "test query";
        let provider_id = "test_provider";
        let params = HashMap::new();
        let response = create_test_search_response();
        
        // 存储到缓存
        manager.store_cached_result(query, provider_id, &params, response).unwrap();
        
        // 获取数据库统计
        let stats = manager.get_database_cache_stats().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_hits, 0); // 还没有命中
    }
}
