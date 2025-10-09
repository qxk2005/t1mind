use std::sync::Arc;
use std::time::SystemTime;

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::web_search::entities::{
    WebSearchProviderConfigPB, WebSearchProviderListPB,
    CreateWebSearchProviderRequestPB, UpdateWebSearchProviderRequestPB,
    DeleteWebSearchProviderRequestPB, TestWebSearchProviderRequestPB,
    TestWebSearchProviderResponsePB, WebSearchTestResultPB,
    WebSearchGlobalConfigPB, UpdateWebSearchGlobalConfigRequestPB,
};
use crate::entities::{WebSearchProviderTypePB, ProviderTestStatusPB};

/// 网络搜索供应商管理器的键前缀
const WEB_SEARCH_CONFIG_PREFIX: &str = "web_search_config";
const WEB_SEARCH_PROVIDER_LIST_KEY: &str = "web_search_provider_list";
const WEB_SEARCH_GLOBAL_SETTINGS_KEY: &str = "web_search_global_settings";
const WEB_SEARCH_VERSION_KEY: &str = "web_search_config_version";

/// 当前配置版本
const CURRENT_CONFIG_VERSION: u32 = 1;

/// 网络搜索供应商管理器
pub struct WebSearchProviderManager {
    store_preferences: Arc<KVStorePreferences>,
}

impl WebSearchProviderManager {
    /// 创建新的网络搜索供应商管理器
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        let manager = Self { store_preferences };
        
        // 检查并执行配置迁移
        if let Err(e) = manager.migrate_config_if_needed() {
            error!("Failed to migrate web search config: {}", e);
        }
        
        manager
    }

    /// 获取全局网络搜索设置
    pub fn get_global_settings(&self) -> WebSearchGlobalConfigPB {
        match self.store_preferences.get_object::<WebSearchGlobalConfigPB>(WEB_SEARCH_GLOBAL_SETTINGS_KEY) {
            Some(config) => config,
            None => {
                warn!("Failed to load web search global settings, using default config");
                // 清理可能损坏的数据
                self.store_preferences.remove(WEB_SEARCH_GLOBAL_SETTINGS_KEY);
                let default_config = WebSearchGlobalConfigPB::default_config();
                // 保存默认配置
                if let Err(e) = self.save_global_settings(default_config.clone()) {
                    error!("Failed to save default global settings: {}", e);
                }
                default_config
            }
        }
    }

    /// 保存全局网络搜索设置
    pub fn save_global_settings(&self, mut settings: WebSearchGlobalConfigPB) -> FlowyResult<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        settings.updated_at = now;
        
        self.store_preferences
            .set_object(WEB_SEARCH_GLOBAL_SETTINGS_KEY, &settings)
            .map_err(|e| {
                error!("Failed to save web search global settings: {}", e);
                FlowyError::internal().with_context(format!("保存网络搜索全局设置失败: {}", e))
            })?;
        
        info!("Web search global settings saved successfully");
        Ok(())
    }

    /// 更新全局网络搜索设置
    pub fn update_global_settings(&self, request: UpdateWebSearchGlobalConfigRequestPB) -> FlowyResult<WebSearchGlobalConfigPB> {
        let mut settings = self.get_global_settings();
        
        if let Some(enabled) = request.enabled {
            settings.enabled = enabled;
        }
        
        if let Some(default_provider_id) = request.default_provider_id {
            settings.default_provider_id = Some(default_provider_id);
        }
        
        if let Some(default_max_results) = request.default_max_results {
            settings.default_max_results = default_max_results;
        }
        
        if let Some(default_timeout_seconds) = request.default_timeout_seconds {
            settings.default_timeout_seconds = default_timeout_seconds;
        }
        
        if let Some(enable_cache) = request.enable_cache {
            settings.enable_cache = enable_cache;
        }
        
        if let Some(cache_expiry_seconds) = request.cache_expiry_seconds {
            settings.cache_expiry_seconds = cache_expiry_seconds;
        }
        
        if let Some(enable_content_filter) = request.enable_content_filter {
            settings.enable_content_filter = enable_content_filter;
        }
        
        if !request.content_filter_rules.is_empty() {
            settings.content_filter_rules = request.content_filter_rules;
        }
        
        if !request.metadata.is_empty() {
            settings.metadata.extend(request.metadata);
        }
        
        self.save_global_settings(settings.clone())?;
        Ok(settings)
    }

    /// 创建网络搜索供应商
    pub fn create_provider(&self, request: CreateWebSearchProviderRequestPB) -> FlowyResult<WebSearchProviderConfigPB> {
        // 验证请求
        self.validate_create_request(&request)?;
        
        // 生成唯一ID
        let provider_id = self.generate_provider_id();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        // 创建供应商配置
        let mut provider_config = WebSearchProviderConfigPB {
            id: provider_id.clone(),
            name: request.name,
            provider_type: request.provider_type,
            description: request.description,
            icon: request.icon,
            api_key: request.api_key,
            base_url: request.base_url,
            is_active: false, // 新创建的供应商默认不激活
            is_enabled: true,
            max_results: request.max_results.max(1).min(100), // 限制在1-100之间
            timeout_seconds: request.timeout_seconds.max(5).min(300), // 限制在5-300秒之间
            created_at: now,
            updated_at: now,
            last_tested_at: None,
            test_status: ProviderTestStatusPB::NotTested,
            metadata: request.metadata,
        };

        // 应用默认设置
        self.apply_default_settings(&mut provider_config);
        
        // 保存配置
        self.save_provider_config(&provider_config)?;
        
        info!("Web search provider created successfully: {} ({})", provider_config.name, provider_config.id);
        Ok(provider_config)
    }

    /// 获取网络搜索供应商
    pub fn get_provider(&self, provider_id: &str) -> FlowyResult<WebSearchProviderConfigPB> {
        let provider_config = self.get_provider_config(provider_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("网络搜索供应商配置不存在"))?;
        
        Ok(provider_config)
    }

    /// 更新网络搜索供应商
    pub fn update_provider(&self, request: UpdateWebSearchProviderRequestPB) -> FlowyResult<WebSearchProviderConfigPB> {
        // 验证请求
        self.validate_update_request(&request)?;
        
        // 获取现有配置
        let mut provider_config = self.get_provider_config(&request.id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("网络搜索供应商配置不存在"))?;
        
        // 更新字段
        if let Some(name) = request.name {
            if !name.trim().is_empty() {
                provider_config.name = name;
            }
        }
        
        if let Some(description) = request.description {
            provider_config.description = description;
        }
        
        if let Some(icon) = request.icon {
            provider_config.icon = icon;
        }
        
        if let Some(api_key) = request.api_key {
            provider_config.api_key = api_key;
        }
        
        if let Some(base_url) = request.base_url {
            provider_config.base_url = base_url;
        }
        
        if let Some(is_active) = request.is_active {
            provider_config.is_active = is_active;
        }
        
        if let Some(is_enabled) = request.is_enabled {
            provider_config.is_enabled = is_enabled;
        }
        
        if let Some(max_results) = request.max_results {
            provider_config.max_results = max_results.max(1).min(100);
        }
        
        if let Some(timeout_seconds) = request.timeout_seconds {
            provider_config.timeout_seconds = timeout_seconds.max(5).min(300);
        }
        
        if !request.metadata.is_empty() {
            provider_config.metadata.extend(request.metadata);
        }
        
        // 更新时间戳
        provider_config.updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        // 保存配置
        self.save_provider_config(&provider_config)?;
        
        info!("Web search provider updated successfully: {} ({})", provider_config.name, provider_config.id);
        Ok(provider_config)
    }

    /// 删除网络搜索供应商
    pub fn delete_provider(&self, request: DeleteWebSearchProviderRequestPB) -> FlowyResult<()> {
        // 检查供应商是否存在
        if !self.provider_exists(&request.id) {
            return Err(FlowyError::record_not_found().with_context("网络搜索供应商配置不存在"));
        }
        
        // 删除供应商配置
        let key = self.provider_config_key(&request.id);
        self.store_preferences.remove(&key);
        
        // 从供应商列表中移除
        self.update_provider_list(&request.id, false)?;
        
        info!("Web search provider deleted successfully: {}", request.id);
        Ok(())
    }

    /// 获取所有网络搜索供应商
    pub fn get_all_providers(&self) -> FlowyResult<WebSearchProviderListPB> {
        let provider_ids: Vec<String> = match self.store_preferences.get_object::<Vec<String>>(WEB_SEARCH_PROVIDER_LIST_KEY) {
            Some(ids) => ids,
            None => {
                warn!("Failed to load provider list, clearing and returning empty list");
                self.store_preferences.remove(WEB_SEARCH_PROVIDER_LIST_KEY);
                Vec::new()
            }
        };

        let mut providers = Vec::new();
        let mut orphaned_ids = Vec::new();
        
        for provider_id in provider_ids {
            if let Some(provider) = self.get_provider_config(&provider_id) {
                providers.push(provider);
            } else {
                warn!("Provider config not found or corrupted for ID: {}, will clean up", provider_id);
                orphaned_ids.push(provider_id);
            }
        }
        
        // 自动清理孤立的 provider ID
        if !orphaned_ids.is_empty() {
            warn!("Cleaning up {} orphaned/corrupted provider IDs", orphaned_ids.len());
            for orphaned_id in orphaned_ids {
                let key = self.provider_config_key(&orphaned_id);
                self.store_preferences.remove(&key);
                if let Err(e) = self.update_provider_list(&orphaned_id, false) {
                    error!("Failed to clean up orphaned provider ID {}: {}", orphaned_id, e);
                }
            }
        }
        
        debug!("Retrieved {} web search provider configurations", providers.len());
        Ok(WebSearchProviderListPB { providers })
    }

    /// 获取激活的网络搜索供应商列表
    pub fn get_active_providers(&self) -> FlowyResult<WebSearchProviderListPB> {
        let all_providers = self.get_all_providers()?;
        let active_providers = all_providers.providers
            .into_iter()
            .filter(|provider| provider.is_active && provider.is_enabled)
            .collect();
        
        Ok(WebSearchProviderListPB { providers: active_providers })
    }

    /// 获取可用的网络搜索供应商列表（已测试通过）
    pub fn get_available_providers(&self) -> FlowyResult<WebSearchProviderListPB> {
        let all_providers = self.get_all_providers()?;
        let available_providers = all_providers.providers
            .into_iter()
            .filter(|provider| provider.is_available())
            .collect();
        
        Ok(WebSearchProviderListPB { providers: available_providers })
    }

    /// 根据类型获取供应商列表
    pub fn get_providers_by_type(&self, provider_type: WebSearchProviderTypePB) -> FlowyResult<WebSearchProviderListPB> {
        let all_providers = self.get_all_providers()?;
        let filtered_providers = all_providers.providers
            .into_iter()
            .filter(|provider| provider.provider_type == provider_type)
            .collect();
        
        Ok(WebSearchProviderListPB { providers: filtered_providers })
    }

    /// 更新供应商激活状态
    pub fn update_provider_active_status(&self, provider_id: &str, is_active: bool) -> FlowyResult<()> {
        let mut provider_config = self.get_provider_config(provider_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("网络搜索供应商配置不存在"))?;
        
        provider_config.is_active = is_active;
        provider_config.updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        self.save_provider_config(&provider_config)?;
        info!("Web search provider {} active status updated to: {}", provider_id, is_active);
        Ok(())
    }

    /// 更新供应商启用状态
    pub fn update_provider_enabled_status(&self, provider_id: &str, is_enabled: bool) -> FlowyResult<()> {
        let mut provider_config = self.get_provider_config(provider_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("网络搜索供应商配置不存在"))?;
        
        provider_config.is_enabled = is_enabled;
        provider_config.updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        self.save_provider_config(&provider_config)?;
        info!("Web search provider {} enabled status updated to: {}", provider_id, is_enabled);
        Ok(())
    }

    /// 测试网络搜索供应商
    pub fn test_provider(&self, request: TestWebSearchProviderRequestPB) -> FlowyResult<TestWebSearchProviderResponsePB> {
        let mut provider_config = self.get_provider_config(&request.id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("网络搜索供应商配置不存在"))?;
        
        // 更新测试状态为测试中
        provider_config.update_test_status(ProviderTestStatusPB::Testing);
        self.save_provider_config(&provider_config)?;
        
        let start_time = SystemTime::now();
        let mut test_results = Vec::new();
        let mut overall_success = true;
        let mut error_message = None;
        
        // 执行测试
        match self.execute_provider_tests(&provider_config) {
            Ok(results) => {
                test_results = results;
                overall_success = test_results.iter().all(|r| r.success);
            }
            Err(e) => {
                overall_success = false;
                error_message = Some(e.to_string());
                test_results.push(WebSearchTestResultPB {
                    test_name: "连接测试".to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                    response_time_ms: 0,
                    details: "供应商连接失败".to_string(),
                });
            }
        }
        
        let response_time = start_time.elapsed().unwrap_or_default().as_millis() as i64;
        
        // 更新测试状态
        let test_status = if overall_success {
            ProviderTestStatusPB::TestPassed
        } else {
            ProviderTestStatusPB::TestFailed
        };
        
        provider_config.update_test_status(test_status);
        self.save_provider_config(&provider_config)?;
        
        Ok(TestWebSearchProviderResponsePB {
            success: overall_success,
            error_message,
            response_time_ms: response_time,
            test_results,
        })
    }

    /// 检查供应商ID是否已存在
    pub fn provider_exists(&self, provider_id: &str) -> bool {
        self.get_provider_config(provider_id).is_some()
    }

    /// 生成唯一的供应商ID
    pub fn generate_provider_id(&self) -> String {
        loop {
            let id = Uuid::new_v4().to_string();
            if !self.provider_exists(&id) {
                return id;
            }
        }
    }

    /// 导出所有配置
    pub fn export_config(&self) -> FlowyResult<WebSearchConfigExport> {
        let global_settings = self.get_global_settings();
        let providers = self.get_all_providers()?.providers;
        
        Ok(WebSearchConfigExport {
            version: CURRENT_CONFIG_VERSION,
            exported_at: SystemTime::now(),
            global_settings,
            providers,
        })
    }

    /// 导入配置
    pub fn import_config(&self, config: WebSearchConfigExport) -> FlowyResult<WebSearchImportResult> {
        let mut result = WebSearchImportResult::default();
        
        // 检查版本兼容性
        if config.version > CURRENT_CONFIG_VERSION {
            return Err(FlowyError::invalid_data()
                .with_context("配置版本不兼容，请升级应用程序"));
        }
        
        // 导入全局设置
        if let Err(e) = self.save_global_settings(config.global_settings) {
            result.errors.push(format!("导入全局设置失败: {}", e));
        } else {
            result.global_settings_imported = true;
        }
        
        // 导入供应商配置
        for provider in config.providers {
            match self.save_provider_config(&provider) {
                Ok(_) => {
                    result.providers_imported += 1;
                    result.imported_provider_ids.push(provider.id.clone());
                }
                Err(e) => {
                    result.errors.push(format!("导入供应商 {} 失败: {}", provider.name, e));
                }
            }
        }
        
        info!("Web search config import completed: {} providers imported, {} errors", 
              result.providers_imported, result.errors.len());
        Ok(result)
    }

    /// 清理所有配置（危险操作）
    pub fn clear_all_config(&self) -> FlowyResult<()> {
        warn!("Clearing all web search configuration data");
        
        // 获取所有供应商ID并删除
        let provider_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(WEB_SEARCH_PROVIDER_LIST_KEY)
            .unwrap_or_default();
        
        for provider_id in provider_ids {
            let key = self.provider_config_key(&provider_id);
            self.store_preferences.remove(&key);
        }
        
        // 清理供应商列表、全局设置和版本信息
        self.store_preferences.remove(WEB_SEARCH_PROVIDER_LIST_KEY);
        self.store_preferences.remove(WEB_SEARCH_GLOBAL_SETTINGS_KEY);
        self.store_preferences.remove(WEB_SEARCH_VERSION_KEY);
        
        info!("All web search configuration data cleared");
        Ok(())
    }

    /// 获取供应商配置（内部方法）
    fn get_provider_config(&self, provider_id: &str) -> Option<WebSearchProviderConfigPB> {
        let key = self.provider_config_key(provider_id);
        match self.store_preferences.get_object::<WebSearchProviderConfigPB>(&key) {
            Some(config) => Some(config),
            None => {
                // 如果反序列化失败，清理损坏的数据
                warn!("Failed to deserialize provider config for ID: {}, removing corrupted data", provider_id);
                self.store_preferences.remove(&key);
                None
            }
        }
    }

    /// 保存供应商配置（内部方法）
    fn save_provider_config(&self, config: &WebSearchProviderConfigPB) -> FlowyResult<()> {
        // 验证配置
        self.validate_provider_config_internal(config)?;
        
        // 保存供应商配置
        let key = self.provider_config_key(&config.id);
        self.store_preferences
            .set_object(&key, config)
            .map_err(|e| {
                error!("Failed to save web search provider config {}: {}", config.id, e);
                FlowyError::internal().with_context(format!("保存网络搜索供应商配置失败: {}", e))
            })?;

        // 更新供应商列表
        self.update_provider_list(&config.id, true)?;
        
        Ok(())
    }

    /// 验证创建请求
    fn validate_create_request(&self, request: &CreateWebSearchProviderRequestPB) -> FlowyResult<()> {
        if request.name.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("供应商名称不能为空"));
        }
        
        if request.api_key.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("API密钥不能为空"));
        }
        
        if request.base_url.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("基础URL不能为空"));
        }
        
        // 简单的URL格式验证
        if !request.base_url.starts_with("http://") && !request.base_url.starts_with("https://") {
            return Err(FlowyError::invalid_data().with_context("URL必须以http://或https://开头"));
        }
        
        if request.max_results < 1 || request.max_results > 100 {
            return Err(FlowyError::invalid_data().with_context("最大结果数量必须在1-100之间"));
        }
        
        if request.timeout_seconds < 5 || request.timeout_seconds > 300 {
            return Err(FlowyError::invalid_data().with_context("超时时间必须在5-300秒之间"));
        }
        
        Ok(())
    }

    /// 验证更新请求
    fn validate_update_request(&self, request: &UpdateWebSearchProviderRequestPB) -> FlowyResult<()> {
        if request.id.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("供应商ID不能为空"));
        }
        
        if let Some(ref name) = request.name {
            if name.trim().is_empty() {
                return Err(FlowyError::invalid_data().with_context("供应商名称不能为空"));
            }
        }
        
        if let Some(ref base_url) = request.base_url {
            if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                return Err(FlowyError::invalid_data().with_context("URL必须以http://或https://开头"));
            }
        }
        
        if let Some(max_results) = request.max_results {
            if max_results < 1 || max_results > 100 {
                return Err(FlowyError::invalid_data().with_context("最大结果数量必须在1-100之间"));
            }
        }
        
        if let Some(timeout_seconds) = request.timeout_seconds {
            if timeout_seconds < 5 || timeout_seconds > 300 {
                return Err(FlowyError::invalid_data().with_context("超时时间必须在5-300秒之间"));
            }
        }
        
        Ok(())
    }

    /// 验证供应商配置（内部方法）
    fn validate_provider_config_internal(&self, config: &WebSearchProviderConfigPB) -> FlowyResult<()> {
        if config.id.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("供应商ID不能为空"));
        }
        
        if config.name.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("供应商名称不能为空"));
        }
        
        if config.api_key.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("API密钥不能为空"));
        }
        
        if config.base_url.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("基础URL不能为空"));
        }
        
        if !config.base_url.starts_with("http://") && !config.base_url.starts_with("https://") {
            return Err(FlowyError::invalid_data().with_context("URL必须以http://或https://开头"));
        }
        
        if config.max_results < 1 || config.max_results > 100 {
            return Err(FlowyError::invalid_data().with_context("最大结果数量必须在1-100之间"));
        }
        
        if config.timeout_seconds < 5 || config.timeout_seconds > 300 {
            return Err(FlowyError::invalid_data().with_context("超时时间必须在5-300秒之间"));
        }
        
        Ok(())
    }

    /// 应用默认设置
    fn apply_default_settings(&self, config: &mut WebSearchProviderConfigPB) {
        let global_settings = self.get_global_settings();
        
        if config.max_results <= 0 {
            config.max_results = global_settings.default_max_results;
        }
        
        if config.timeout_seconds <= 0 {
            config.timeout_seconds = global_settings.default_timeout_seconds;
        }
    }

    /// 执行供应商测试
    fn execute_provider_tests(&self, config: &WebSearchProviderConfigPB) -> FlowyResult<Vec<WebSearchTestResultPB>> {
        // 根据供应商类型创建相应的供应商实例并执行测试
        match config.provider_type {
            WebSearchProviderTypePB::Tavily => {
                match crate::web_search::providers::create_tavily_provider(config.clone()) {
                    Ok(provider) => {
                        // 使用异步运行时执行测试
                        let rt = tokio::runtime::Handle::current();
                        let test_response = rt.block_on(provider.run_tests())?;
                        Ok(test_response.test_results)
                    }
                    Err(e) => Err(e),
                }
            }
            WebSearchProviderTypePB::BraveSearch => {
                match crate::web_search::providers::create_brave_search_provider(config.clone()) {
                    Ok(provider) => {
                        // 使用异步运行时执行测试
                        let rt = tokio::runtime::Handle::current();
                        let test_response = rt.block_on(provider.run_tests())?;
                        Ok(test_response.test_results)
                    }
                    Err(e) => Err(e),
                }
            }
            WebSearchProviderTypePB::Custom => {
                // 自定义供应商暂时返回成功，实际实现需要根据具体需求
                Ok(vec![
                    WebSearchTestResultPB {
                        test_name: "自定义供应商测试".to_string(),
                        success: true,
                        error_message: None,
                        response_time_ms: 0,
                        details: "自定义供应商暂不支持自动测试".to_string(),
                    }
                ])
            }
        }
    }


    /// 更新供应商列表
    fn update_provider_list(&self, provider_id: &str, add: bool) -> FlowyResult<()> {
        let mut provider_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(WEB_SEARCH_PROVIDER_LIST_KEY)
            .unwrap_or_default();

        if add {
            if !provider_ids.contains(&provider_id.to_string()) {
                provider_ids.push(provider_id.to_string());
            }
        } else {
            provider_ids.retain(|id| id != provider_id);
        }

        self.store_preferences
            .set_object(WEB_SEARCH_PROVIDER_LIST_KEY, &provider_ids)
            .map_err(|e| {
                error!("Failed to update provider list: {}", e);
                FlowyError::internal().with_context(format!("更新供应商列表失败: {}", e))
            })?;

        Ok(())
    }

    /// 生成供应商配置的存储键
    fn provider_config_key(&self, provider_id: &str) -> String {
        format!("{}:provider:{}", WEB_SEARCH_CONFIG_PREFIX, provider_id)
    }

    /// 检查并执行配置迁移
    fn migrate_config_if_needed(&self) -> FlowyResult<()> {
        let current_version = self.store_preferences
            .get_object::<u32>(WEB_SEARCH_VERSION_KEY)
            .unwrap_or(0);
        
        if current_version < CURRENT_CONFIG_VERSION {
            info!("Migrating web search config from version {} to {}", 
                  current_version, CURRENT_CONFIG_VERSION);
            
            // 这里可以添加版本迁移逻辑
            // 目前只是更新版本号
            self.store_preferences
                .set_object(WEB_SEARCH_VERSION_KEY, &CURRENT_CONFIG_VERSION)
                .map_err(|e| {
                    error!("Failed to update web search config version: {}", e);
                    FlowyError::internal().with_context("更新配置版本失败")
                })?;
            
            info!("Web search config migration completed");
        }
        
        Ok(())
    }
}

/// 配置导出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfigExport {
    pub version: u32,
    pub exported_at: SystemTime,
    pub global_settings: WebSearchGlobalConfigPB,
    pub providers: Vec<WebSearchProviderConfigPB>,
}

/// 配置导入结果
#[derive(Debug, Clone, Default)]
pub struct WebSearchImportResult {
    pub global_settings_imported: bool,
    pub providers_imported: usize,
    pub imported_provider_ids: Vec<String>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use flowy_sqlite::kv::KVStorePreferences;
    use tempfile::TempDir;

    fn create_test_provider_manager() -> (WebSearchProviderManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        let manager = WebSearchProviderManager::new(store);
        (manager, tempdir)
    }

    fn create_test_provider_request() -> CreateWebSearchProviderRequestPB {
        CreateWebSearchProviderRequestPB {
            name: "测试Tavily供应商".to_string(),
            provider_type: WebSearchProviderTypePB::Tavily,
            description: "这是一个测试Tavily供应商".to_string(),
            icon: "search".to_string(),
            api_key: "test_api_key".to_string(),
            base_url: "https://api.tavily.com".to_string(),
            max_results: 10,
            timeout_seconds: 30,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_global_settings() {
        let (manager, _tempdir) = create_test_provider_manager();
        
        // 测试默认设置
        let default_settings = manager.get_global_settings();
        assert!(default_settings.enabled);
        assert_eq!(default_settings.default_max_results, 10);
        
        // 测试保存和读取设置
        let mut custom_settings = default_settings.clone();
        custom_settings.enabled = false;
        custom_settings.default_max_results = 20;
        
        manager.save_global_settings(custom_settings.clone()).unwrap();
        let loaded_settings = manager.get_global_settings();
        
        assert!(!loaded_settings.enabled);
        assert_eq!(loaded_settings.default_max_results, 20);
    }

    #[test]
    fn test_provider_crud_operations() {
        let (manager, _tempdir) = create_test_provider_manager();
        
        // 测试创建供应商
        let request = create_test_provider_request();
        let provider = manager.create_provider(request).unwrap();
        assert_eq!(provider.name, "测试Tavily供应商");
        assert!(!provider.is_active); // 新创建的供应商默认不激活
        
        // 测试获取供应商
        let retrieved_provider = manager.get_provider(&provider.id).unwrap();
        assert_eq!(retrieved_provider.id, provider.id);
        assert_eq!(retrieved_provider.name, provider.name);
        
        // 测试更新供应商
        let update_request = UpdateWebSearchProviderRequestPB {
            id: provider.id.clone(),
            name: Some("更新后的供应商".to_string()),
            description: Some("更新后的描述".to_string()),
            icon: None,
            api_key: None,
            base_url: None,
            is_active: Some(true),
            is_enabled: None,
            max_results: None,
            timeout_seconds: None,
            metadata: HashMap::new(),
        };
        let updated_provider = manager.update_provider(update_request).unwrap();
        assert_eq!(updated_provider.name, "更新后的供应商");
        assert!(updated_provider.is_active);
        
        // 测试删除供应商
        let delete_request = DeleteWebSearchProviderRequestPB { id: provider.id.clone() };
        manager.delete_provider(delete_request).unwrap();
        
        assert!(manager.get_provider(&provider.id).is_err());
    }

    #[test]
    fn test_provider_list_operations() {
        let (manager, _tempdir) = create_test_provider_manager();
        
        // 创建多个供应商
        let request1 = create_test_provider_request();
        let provider1 = manager.create_provider(request1).unwrap();
        
        let mut request2 = create_test_provider_request();
        request2.name = "第二个供应商".to_string();
        request2.provider_type = WebSearchProviderTypePB::BraveSearch;
        let provider2 = manager.create_provider(request2).unwrap();
        
        // 测试获取所有供应商
        let all_providers = manager.get_all_providers().unwrap();
        assert_eq!(all_providers.providers.len(), 2);
        
        // 激活一个供应商
        manager.update_provider_active_status(&provider1.id, true).unwrap();
        
        // 测试获取激活的供应商
        let active_providers = manager.get_active_providers().unwrap();
        assert_eq!(active_providers.providers.len(), 1);
        assert_eq!(active_providers.providers[0].id, provider1.id);
        
        // 测试根据类型获取供应商
        let tavily_providers = manager.get_providers_by_type(WebSearchProviderTypePB::Tavily).unwrap();
        assert_eq!(tavily_providers.providers.len(), 1);
        assert_eq!(tavily_providers.providers[0].id, provider1.id);
    }

    #[test]
    fn test_validation() {
        let (manager, _tempdir) = create_test_provider_manager();
        
        // 测试空名称验证
        let mut invalid_request = create_test_provider_request();
        invalid_request.name = "".to_string();
        assert!(manager.create_provider(invalid_request).is_err());
        
        // 测试空API密钥验证
        let mut invalid_request = create_test_provider_request();
        invalid_request.api_key = "".to_string();
        assert!(manager.create_provider(invalid_request).is_err());
        
        // 测试无效URL验证
        let mut invalid_request = create_test_provider_request();
        invalid_request.base_url = "invalid_url".to_string();
        assert!(manager.create_provider(invalid_request).is_err());
        
        // 测试无效最大结果数量验证
        let mut invalid_request = create_test_provider_request();
        invalid_request.max_results = 0;
        assert!(manager.create_provider(invalid_request.clone()).is_err());
        
        invalid_request.max_results = 101;
        assert!(manager.create_provider(invalid_request).is_err());
        
        // 测试无效超时时间验证
        let mut invalid_request = create_test_provider_request();
        invalid_request.timeout_seconds = 4;
        assert!(manager.create_provider(invalid_request.clone()).is_err());
        
        invalid_request.timeout_seconds = 301;
        assert!(manager.create_provider(invalid_request).is_err());
    }

    #[test]
    fn test_provider_testing() {
        let (manager, _tempdir) = create_test_provider_manager();
        
        // 创建供应商
        let request = create_test_provider_request();
        let provider = manager.create_provider(request).unwrap();
        
        // 测试供应商
        let test_request = TestWebSearchProviderRequestPB { id: provider.id.clone() };
        let test_response = manager.test_provider(test_request).unwrap();
        
        // 验证测试结果
        assert!(test_response.success);
        assert_eq!(test_response.test_results.len(), 3);
        
        // 验证测试状态已更新
        let updated_provider = manager.get_provider(&provider.id).unwrap();
        assert_eq!(updated_provider.test_status, ProviderTestStatusPB::TestPassed);
        assert!(updated_provider.last_tested_at.is_some());
    }

    #[test]
    fn test_config_export_import() {
        let (manager, _tempdir) = create_test_provider_manager();
        
        // 创建测试数据
        let request = create_test_provider_request();
        manager.create_provider(request).unwrap();
        
        // 测试导出
        let export = manager.export_config().unwrap();
        assert_eq!(export.providers.len(), 1);
        assert_eq!(export.version, CURRENT_CONFIG_VERSION);
        
        // 清理配置
        manager.clear_all_config().unwrap();
        let all_providers = manager.get_all_providers().unwrap();
        assert_eq!(all_providers.providers.len(), 0);
        
        // 测试导入
        let import_result = manager.import_config(export).unwrap();
        assert!(import_result.global_settings_imported);
        assert_eq!(import_result.providers_imported, 1);
        
        let all_providers = manager.get_all_providers().unwrap();
        assert_eq!(all_providers.providers.len(), 1);
    }
}
