use std::sync::Arc;
use std::collections::HashMap;

use flowy_sqlite::kv::KVStorePreferences;
use tracing::{info, debug};

use crate::web_search::{
    WebSearchHub, WebSearchHubStatus, WebSearchGlobalConfigPB,
    CacheConfig,
};
use crate::web_search::entities::{
    WebSearchProviderConfigPB,
    CreateWebSearchProviderRequestPB,
    WebSearchRequestPB,
};
use crate::entities::{WebSearchProviderTypePB, ProviderTestStatusPB};

/// 演示网络搜索中心服务的基本功能
pub async fn demonstrate_web_search_hub() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== 网络搜索中心服务演示 ===");

    // 创建存储偏好设置
    let store_preferences = Arc::new(KVStorePreferences::new("web_search_demo")?);
    
    // 创建网络搜索中心服务
    let hub = WebSearchHub::new(store_preferences.clone());
    
    // 显示初始状态
    let initial_status = hub.get_status();
    info!("初始状态: {:?}", initial_status);
    
    // 创建全局配置
    let mut global_config = WebSearchGlobalConfigPB::default_config();
    global_config.enabled = true;
    global_config.enable_cache = true;
    global_config.default_max_results = 10;
    global_config.default_timeout_seconds = 30;
    
    // 更新全局配置
    let update_request = crate::web_search::entities::UpdateWebSearchGlobalConfigRequestPB {
        enabled: Some(true),
        default_provider_id: None,
        default_max_results: Some(10),
        default_timeout_seconds: Some(30),
        enable_cache: Some(true),
        cache_expiry_seconds: Some(3600),
        enable_content_filter: Some(true),
        content_filter_rules: vec!["adult".to_string(), "violence".to_string()],
        metadata: HashMap::new(),
    };
    hub.update_global_config(update_request).await?;
    info!("全局配置已更新");
    
    // 创建 Tavily 供应商
    let tavily_request = CreateWebSearchProviderRequestPB {
        name: "Tavily Demo".to_string(),
        provider_type: WebSearchProviderTypePB::Tavily,
        description: "Tavily 搜索引擎演示".to_string(),
        icon: "🔍".to_string(),
        api_key: "demo_api_key".to_string(),
        base_url: "https://api.tavily.com".to_string(),
        max_results: 10,
        timeout_seconds: 30,
        metadata: HashMap::new(),
    };
    
    let tavily_provider = hub.provider_manager.create_provider(tavily_request)?;
    info!("创建 Tavily 供应商: {}", tavily_provider.name);
    
    // 激活供应商
    let update_request = crate::web_search::entities::UpdateWebSearchProviderRequestPB {
        id: tavily_provider.id.clone(),
        name: Some(tavily_provider.name.clone()),
        description: Some(tavily_provider.description.clone()),
        icon: Some(tavily_provider.icon.clone()),
        api_key: Some(tavily_provider.api_key.clone()),
        base_url: Some(tavily_provider.base_url.clone()),
        is_active: Some(true), // 激活供应商
        is_enabled: Some(true),
        max_results: Some(tavily_provider.max_results),
        timeout_seconds: Some(tavily_provider.timeout_seconds),
        metadata: tavily_provider.metadata.clone(),
    };
    
    let updated_provider = hub.provider_manager.update_provider(update_request)?;
    info!("激活供应商: {}", updated_provider.name);
    
    // 显示更新后的状态
    let updated_status = hub.get_status();
    info!("更新后状态: {:?}", updated_status);
    
    // 创建搜索请求
    let search_request = WebSearchRequestPB {
        query: "Rust 编程语言".to_string(),
        provider_id: Some(updated_provider.id.clone()),
        max_results: 5,
        language: "zh-CN".to_string(),
        region: "CN".to_string(),
        include_content: true,
        metadata: HashMap::new(),
    };
    
    info!("准备执行搜索: {}", search_request.query);
    
    // 注意：由于没有真实的 API 密钥，搜索会失败，但我们可以演示错误处理
    match hub.search(search_request.clone()).await {
        Ok(response) => {
            info!("搜索成功！");
            info!("查询: {}", response.query);
            info!("供应商: {}", response.provider_id);
            info!("结果数量: {}", response.results.len());
            info!("执行时间: {}ms", response.execution_time_ms);
            
            for (i, result) in response.results.iter().enumerate() {
                info!("结果 {}: {}", i + 1, result.title);
                info!("  URL: {}", result.url);
                if let Some(content) = &result.content {
                    info!("  摘要: {}", content);
                }
            }
        }
        Err(e) => {
            info!("搜索失败（预期行为，因为没有真实的 API 密钥）: {}", e);
            info!("这演示了中心服务的错误处理能力");
        }
    }
    
    // 演示缓存功能
    info!("=== 缓存功能演示 ===");
    
    let cache_stats = hub.get_cache_stats().await?;
    info!("缓存统计: {:?}", cache_stats);
    
    // 清理缓存
    hub.clear_cache().await?;
    info!("缓存已清理");
    
    // 演示供应商测试功能
    info!("=== 供应商测试演示 ===");
    
    let tested_providers = hub.test_all_providers().await?;
    info!("测试了 {} 个供应商", tested_providers.len());
    
    for provider in tested_providers {
        info!("供应商: {} - 测试状态: {}", provider.name, provider.test_status);
    }
    
    info!("=== 网络搜索中心服务演示完成 ===");
    Ok(())
}

/// 演示网络搜索中心服务的高级功能
pub async fn demonstrate_advanced_hub_features() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== 网络搜索中心服务高级功能演示 ===");

    // 创建自定义配置的中心服务
    let store_preferences = Arc::new(KVStorePreferences::new("advanced_hub_demo")?);
    
    let mut global_config = WebSearchGlobalConfigPB::default_config();
    global_config.enabled = true;
    global_config.enable_cache = true;
    global_config.enable_content_filter = true;
    global_config.default_max_results = 20;
    global_config.default_timeout_seconds = 60;
    global_config.cache_expiry_seconds = 7200; // 2小时
    global_config.content_filter_rules = vec![
        "adult_content".to_string(),
        "malware".to_string(),
        "spam".to_string(),
    ];
    
    let cache_config = CacheConfig {
        enabled: true,
        default_ttl_seconds: 3600, // 1小时
        max_entries: 500,
        cleanup_interval_seconds: 300, // 5分钟
        enable_compression: true,
    };
    
    let hub = WebSearchHub::with_config(store_preferences, global_config, cache_config);
    
    // 显示配置状态
    let status = hub.get_status();
    info!("高级配置状态: {:?}", status);
    
    // 演示多个供应商管理
    info!("=== 多供应商管理演示 ===");
    
    // 创建多个供应商
    let providers = vec![
        ("Tavily Advanced", WebSearchProviderTypePB::Tavily),
        ("Brave Advanced", WebSearchProviderTypePB::BraveSearch),
    ];
    
    for (name, provider_type) in providers {
        let request = CreateWebSearchProviderRequestPB {
            name: name.to_string(),
            provider_type,
            description: format!("{} 高级配置", name),
            icon: "🔍".to_string(),
            api_key: format!("{}_demo_key", name.to_lowercase().replace(" ", "_")),
            base_url: "https://api.example.com".to_string(),
            max_results: 15,
            timeout_seconds: 45,
            metadata: HashMap::new(),
        };
        
        let provider = hub.provider_manager.create_provider(request)?;
        info!("创建供应商: {} ({})", provider.name, provider.provider_type);
    }
    
    // 列出所有供应商
    let provider_list = hub.provider_manager.get_all_providers()?;
    info!("总供应商数量: {}", provider_list.providers.len());
    
    for provider in &provider_list.providers {
        info!("供应商: {} - 类型: {} - 状态: {} - 测试: {}", 
              provider.name, 
              provider.provider_type,
              if provider.is_active { "活跃" } else { "非活跃" },
              provider.test_status);
    }
    
    // 演示缓存性能
    info!("=== 缓存性能演示 ===");
    
    let cache_stats = hub.get_cache_stats().await?;
    info!("缓存性能统计:");
    info!("  命中次数: {}", cache_stats.hits);
    info!("  未命中次数: {}", cache_stats.misses);
    info!("  命中率: {:.2}%", cache_stats.hit_rate() * 100.0);
    info!("  缓存条目数: {}", cache_stats.total_entries);
    info!("  缓存大小: {} 字节", cache_stats.cache_size_bytes);
    
    info!("=== 网络搜索中心服务高级功能演示完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demonstrate_web_search_hub() {
        // 这个测试演示了中心服务的基本功能
        // 注意：由于没有真实的 API 密钥，搜索会失败，但错误处理会被演示
        let result = demonstrate_web_search_hub().await;
        // 我们期望这个演示能够运行，即使搜索失败
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("API"));
    }

    #[tokio::test]
    async fn test_demonstrate_advanced_hub_features() {
        // 这个测试演示了中心服务的高级功能
        let result = demonstrate_advanced_hub_features().await;
        assert!(result.is_ok());
    }
}
