use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::web_search::entities::{
    WebSearchRequestPB, WebSearchResponsePB, WebSearchProviderConfigPB,
    WebSearchGlobalConfigPB, WebSearchProviderListPB,
    CreateWebSearchProviderRequestPB, UpdateWebSearchProviderRequestPB,
    DeleteWebSearchProviderRequestPB, TestWebSearchProviderRequestPB,
    TestWebSearchProviderResponsePB, WebSearchCacheStatsPB,
    UpdateWebSearchGlobalConfigRequestPB,
};
use crate::entities::{ProviderTestStatusPB, WebSearchProviderTypePB};
use crate::web_search::provider_manager::WebSearchProviderManager;
use crate::web_search::result_processor::WebSearchResultProcessor;
use crate::web_search::cache_manager::{WebSearchCacheManager, CacheConfig};
use crate::web_search::providers::{WebSearchProvider, TavilySearchProvider, BraveSearchProvider};

/// 网络搜索中心服务
/// 
/// 这是网络搜索功能的统一入口点，负责：
/// 1. 管理搜索供应商的选择和切换
/// 2. 协调搜索结果的处理和格式化
/// 3. 管理搜索结果的缓存
/// 4. 提供统一的搜索接口
/// 5. 处理错误和回退逻辑
pub struct WebSearchHub {
    /// 供应商管理器
    pub provider_manager: WebSearchProviderManager,
    /// 结果处理器
    result_processor: WebSearchResultProcessor,
    /// 缓存管理器
    cache_manager: WebSearchCacheManager,
    /// 全局配置
    global_config: WebSearchGlobalConfigPB,
}

impl WebSearchHub {
    /// 创建新的网络搜索中心服务
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        let provider_manager = WebSearchProviderManager::new(store_preferences.clone());
        let result_processor = WebSearchResultProcessor::new();
        let cache_manager = WebSearchCacheManager::new(store_preferences.clone());
        let global_config = provider_manager.get_global_settings();

        Self {
            provider_manager,
            result_processor,
            cache_manager,
            global_config,
        }
    }

    /// 创建带有自定义配置的网络搜索中心服务
    pub fn with_config(
        store_preferences: Arc<KVStorePreferences>,
        global_config: WebSearchGlobalConfigPB,
        cache_config: CacheConfig,
    ) -> Self {
        let provider_manager = WebSearchProviderManager::new(store_preferences.clone());
        let result_processor = WebSearchResultProcessor::with_config(
            global_config.enable_content_filter,
            global_config.content_filter_rules.clone(),
            global_config.default_max_results as usize,
            0.1, // 最小相关性阈值
        );
        let cache_manager = WebSearchCacheManager::with_config(store_preferences.clone(), cache_config);

        Self {
            provider_manager,
            result_processor,
            cache_manager,
            global_config,
        }
    }

    /// 执行网络搜索
    /// 
    /// # Arguments
    /// * `request` - 搜索请求
    /// 
    /// # Returns
    /// 统一的搜索结果响应
    pub async fn search(&self, request: WebSearchRequestPB) -> FlowyResult<WebSearchResponsePB> {
        debug!("WebSearchHub: Starting search for query: {}", request.query);

        // 检查全局配置是否启用
        if !self.global_config.enabled {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::InvalidRequest,
                "网络搜索功能未启用".to_string(),
            ));
        }

        // 检查缓存
        if self.global_config.enable_cache {
            if let Some(cached_result) = self.cache_manager.get_cached_result(&request.query, &request.provider_id.as_ref().unwrap_or(&"".to_string()), &HashMap::new())? {
                debug!("WebSearchHub: Cache hit for query: {}", request.query);
                return Ok(cached_result);
            }
        }

        // 获取活跃的供应商
        let active_provider = self.get_active_provider().await?;
        
        // 执行搜索
        let search_result = self.execute_search_with_provider(&request, &active_provider).await;
        
        match search_result {
            Ok(response) => {
                // 缓存结果
                if self.global_config.enable_cache {
                    if let Err(e) = self.cache_manager.store_cached_result(&request.query, &request.provider_id.as_ref().unwrap_or(&"".to_string()), &HashMap::new(), response.clone()) {
                        warn!("WebSearchHub: Failed to cache search result: {}", e);
                    }
                }
                
                // info!("WebSearchHub: Search completed successfully for query: {}", request.query);
                Ok(response)
            }
            Err(e) => {
                error!("WebSearchHub: Search failed for query: {}: {}", request.query, e);
                Err(e)
            }
        }
    }

    /// 获取活跃的搜索供应商
    async fn get_active_provider(&self) -> FlowyResult<Box<dyn WebSearchProvider>> {
        let providers = self.provider_manager.get_all_providers()?.providers;
        
        // 查找活跃的供应商
        let active_provider = providers.iter()
            .find(|p| p.is_active && p.is_enabled)
            .ok_or_else(|| FlowyError::new(
                flowy_error::ErrorCode::RecordNotFound,
                "未找到活跃的网络搜索供应商".to_string(),
            ))?;

        debug!("WebSearchHub: Using active provider: {} ({})", 
               active_provider.name, active_provider.provider_type);

        // 根据供应商类型创建相应的供应商实例
        match active_provider.provider_type {
            WebSearchProviderTypePB::Tavily => {
                let provider = TavilySearchProvider::new(active_provider.clone())?;
                Ok(Box::new(provider))
            }
            WebSearchProviderTypePB::BraveSearch => {
                let provider = BraveSearchProvider::new(active_provider.clone())?;
                Ok(Box::new(provider))
            }
            WebSearchProviderTypePB::Custom => {
                Err(FlowyError::new(
                    flowy_error::ErrorCode::NotSupportYet,
                    "自定义搜索供应商暂不支持".to_string(),
                ))
            }
        }
    }

    /// 使用指定供应商执行搜索
    async fn execute_search_with_provider(
        &self,
        request: &WebSearchRequestPB,
        provider: &Box<dyn WebSearchProvider>,
    ) -> FlowyResult<WebSearchResponsePB> {
        debug!("WebSearchHub: Executing search with provider: {}", provider.get_config().name);

        // 检查供应商是否可用
        if !provider.is_available() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::AIServiceUnavailable,
                format!("搜索供应商 {} 当前不可用", provider.get_config().name),
            ));
        }

        // 设置超时
        let timeout_duration = Duration::from_secs(
            self.global_config.default_timeout_seconds as u64
        );

        // 执行搜索（带超时）
        let search_future = provider.search(request.clone());
        let result = tokio::time::timeout(timeout_duration, search_future).await;

        match result {
            Ok(Ok(response)) => {
                debug!("WebSearchHub: Provider search completed successfully");
                Ok(response)
            }
            Ok(Err(e)) => {
                error!("WebSearchHub: Provider search failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("WebSearchHub: Provider search timed out after {} seconds", 
                       self.global_config.default_timeout_seconds);
                Err(FlowyError::new(
                    flowy_error::ErrorCode::RequestTimeout,
                    format!("搜索超时，超过 {} 秒", self.global_config.default_timeout_seconds),
                ))
            }
        }
    }

    /// 测试所有供应商的连接
    pub async fn test_all_providers(&self) -> FlowyResult<Vec<WebSearchProviderConfigPB>> {
        debug!("WebSearchHub: Testing all providers");

        let providers = self.provider_manager.get_all_providers()?.providers;
        let mut tested_providers = Vec::new();

        for provider_config in providers {
            if !provider_config.is_enabled {
                continue;
            }

            debug!("WebSearchHub: Testing provider: {}", provider_config.name);

            let provider_result = match provider_config.provider_type {
                WebSearchProviderTypePB::Tavily => {
                    let provider = TavilySearchProvider::new(provider_config.clone())?;
                    provider.run_tests().await
                }
                WebSearchProviderTypePB::BraveSearch => {
                    let provider = BraveSearchProvider::new(provider_config.clone())?;
                    provider.run_tests().await
                }
                WebSearchProviderTypePB::Custom => {
                    continue; // 跳过自定义供应商
                }
            };

            match provider_result {
                Ok(test_response) => {
                    debug!("WebSearchHub: Provider {} test completed", provider_config.name);
                    // 更新供应商的测试状态
                    let mut updated_config = provider_config.clone();
                    updated_config.test_status = if test_response.success {
                        ProviderTestStatusPB::TestPassed
                    } else {
                        ProviderTestStatusPB::TestFailed
                    };
                    updated_config.last_tested_at = Some(
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                    );
                    
                    // 保存更新的配置
                    if let Err(e) = self.provider_manager.update_provider(
                        crate::web_search::entities::UpdateWebSearchProviderRequestPB {
                            id: updated_config.id.clone(),
                            name: Some(updated_config.name.clone()),
                            description: Some(updated_config.description.clone()),
                            icon: Some(updated_config.icon.clone()),
                            api_key: Some(updated_config.api_key.clone()),
                            base_url: Some(updated_config.base_url.clone()),
                            is_active: Some(updated_config.is_active),
                            is_enabled: Some(updated_config.is_enabled),
                            max_results: Some(updated_config.max_results),
                            timeout_seconds: Some(updated_config.timeout_seconds),
                            metadata: updated_config.metadata.clone(),
                        }
                    ) {
                        warn!("WebSearchHub: Failed to update provider config: {}", e);
                    }
                    
                    tested_providers.push(updated_config);
                }
                Err(e) => {
                    warn!("WebSearchHub: Provider {} test failed: {}", provider_config.name, e);
                    // 仍然添加失败的供应商，但标记为失败状态
                    let mut failed_config = provider_config.clone();
                    failed_config.test_status = ProviderTestStatusPB::TestFailed;
                    tested_providers.push(failed_config);
                }
            }
        }

        info!("WebSearchHub: Provider testing completed, {} providers tested", tested_providers.len());
        Ok(tested_providers)
    }

    /// 获取缓存统计信息
    pub async fn get_cache_stats(&self) -> FlowyResult<crate::web_search::cache_manager::CacheStats> {
        Ok(self.cache_manager.get_cache_stats())
    }

    /// 清理缓存
    pub async fn clear_cache(&self) -> FlowyResult<()> {
        debug!("WebSearchHub: Clearing cache");
        self.cache_manager.clear_all_cache()?;
        info!("WebSearchHub: Cache cleared successfully");
        Ok(())
    }

    /// 获取供应商列表
    pub async fn get_provider_list(&self) -> FlowyResult<WebSearchProviderListPB> {
        self.provider_manager.get_all_providers()
    }

    /// 创建供应商
    pub async fn create_provider(&self, request: CreateWebSearchProviderRequestPB) -> FlowyResult<WebSearchProviderConfigPB> {
        self.provider_manager.create_provider(request)
    }

    /// 获取供应商
    pub async fn get_provider(&self, provider_id: &str) -> FlowyResult<WebSearchProviderConfigPB> {
        self.provider_manager.get_provider(provider_id)
    }

    /// 更新供应商
    pub async fn update_provider(&self, request: UpdateWebSearchProviderRequestPB) -> FlowyResult<WebSearchProviderConfigPB> {
        self.provider_manager.update_provider(request)
    }

    /// 删除供应商
    pub async fn delete_provider(&self, provider_id: &str) -> FlowyResult<()> {
        let request = DeleteWebSearchProviderRequestPB {
            id: provider_id.to_string(),
        };
        self.provider_manager.delete_provider(request)
    }

    /// 测试供应商
    pub async fn test_provider(&self, request: TestWebSearchProviderRequestPB) -> FlowyResult<TestWebSearchProviderResponsePB> {
        self.provider_manager.test_provider(request).await
    }

    /// 获取全局配置
    pub async fn get_global_config(&self) -> FlowyResult<WebSearchGlobalConfigPB> {
        // 从provider_manager获取最新的全局配置
        Ok(self.provider_manager.get_global_settings())
    }
    pub fn get_status(&self) -> WebSearchHubStatus {
        let providers = match self.provider_manager.get_all_providers() {
            Ok(result) => result.providers,
            Err(_) => Vec::new(),
        };
        let active_providers = providers.iter()
            .filter(|p| p.is_active && p.is_enabled)
            .count();
        let total_providers = providers.len();

        WebSearchHubStatus {
            enabled: self.global_config.enabled,
            active_providers,
            total_providers,
            cache_enabled: self.global_config.enable_cache,
            content_filter_enabled: self.global_config.enable_content_filter,
        }
    }

    /// 更新全局配置
    pub async fn update_global_config(&self, request: UpdateWebSearchGlobalConfigRequestPB) -> FlowyResult<()> {
        debug!("WebSearchHub: Updating global config");
        // 将请求转换为全局配置
        let mut config = self.global_config.clone();
        if let Some(enabled) = request.enabled {
            config.enabled = enabled;
        }
        if let Some(default_provider_id) = request.default_provider_id {
            config.default_provider_id = Some(default_provider_id);
        }
        if let Some(default_max_results) = request.default_max_results {
            config.default_max_results = default_max_results;
        }
        if let Some(default_timeout_seconds) = request.default_timeout_seconds {
            config.default_timeout_seconds = default_timeout_seconds;
        }
        if let Some(enable_cache) = request.enable_cache {
            config.enable_cache = enable_cache;
        }
        if let Some(cache_expiry_seconds) = request.cache_expiry_seconds {
            config.cache_expiry_seconds = cache_expiry_seconds;
        }
        if let Some(enable_content_filter) = request.enable_content_filter {
            config.enable_content_filter = enable_content_filter;
        }
        config.content_filter_rules = request.content_filter_rules;
        config.metadata = request.metadata;
        
        self.provider_manager.save_global_settings(config)?;
        info!("WebSearchHub: Global config updated successfully");
        Ok(())
    }
}

/// 网络搜索中心服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchHubStatus {
    /// 是否启用
    pub enabled: bool,
    /// 活跃供应商数量
    pub active_providers: usize,
    /// 总供应商数量
    pub total_providers: usize,
    /// 是否启用缓存
    pub cache_enabled: bool,
    /// 是否启用内容过滤
    pub content_filter_enabled: bool,
}

impl Default for WebSearchHubStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            active_providers: 0,
            total_providers: 0,
            cache_enabled: false,
            content_filter_enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use flowy_sqlite::kv::KVStorePreferences;

    #[tokio::test]
    async fn test_web_search_hub_creation() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_db".to_string()));
        let hub = WebSearchHub::new(store_preferences);
        
        let status = hub.get_status();
        assert_eq!(status.enabled, false);
        assert_eq!(status.active_providers, 0);
        assert_eq!(status.total_providers, 0);
    }

    #[tokio::test]
    async fn test_web_search_hub_with_config() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_db".to_string()));
        let global_config = WebSearchGlobalConfigPB::default_config();
        let cache_config = CacheConfig::default();
        
        let hub = WebSearchHub::with_config(store_preferences, global_config, cache_config);
        
        let status = hub.get_status();
        assert_eq!(status.enabled, false);
    }

    #[tokio::test]
    async fn test_search_when_disabled() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_db".to_string()));
        let hub = WebSearchHub::new(store_preferences);
        
        let request = WebSearchRequestPB {
            query: "test query".to_string(),
            max_results: Some(10),
            timeout_seconds: Some(30),
            include_answer: Some(true),
            include_images: Some(false),
            include_raw_content: Some(false),
            search_depth: Some("basic".to_string()),
            include_domains: vec![],
            exclude_domains: vec![],
            category: None,
            metadata: std::collections::HashMap::new(),
        };
        
        let result = hub.search(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("未启用"));
    }
}
