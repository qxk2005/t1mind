use std::sync::{Arc, Weak};
use tracing::{error, info};
use uuid::Uuid;

use crate::ai_manager::AIManager;
use crate::entities::*;
use crate::web_search::hub::WebSearchHub;
use flowy_error::{FlowyError, FlowyResult};
use lib_dispatch::prelude::*;
use validator::Validate;

/// 获取网络搜索供应商列表处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_web_search_provider_list_handler(
    _data: AFPluginData<EmptyRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchProviderListPB, FlowyError> {
    // info!("[DEBUG] 开始处理获取网络搜索供应商列表请求");
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    // info!("[DEBUG] AI Manager升级成功");
    
    let web_search_hub = match ai_manager.get_web_search_hub().await {
        Ok(hub) => {
            // info!("[DEBUG] WebSearchHub获取成功");
            hub
        },
        Err(e) => {
            error!("获取WebSearchHub失败: {}", e);
            // 返回空列表而不是错误，让UI能够正常显示
            let empty_list = WebSearchProviderListPB { providers: vec![] };
            // info!("[DEBUG] 返回空供应商列表");
            return data_result_ok(empty_list);
        }
    };
    
    // info!("[DEBUG] 开始从Hub获取供应商列表");
    let provider_list = match web_search_hub.get_provider_list().await {
        Ok(list) => {
            // info!("[DEBUG] 成功获取供应商列表，数量: {}", list.providers.len());
            list
        },
        Err(e) => {
            error!("获取供应商列表失败: {}", e);
            return Err(e);
        }
    };
    
    // 打印每个供应商的详细信息（用于调试）
    // for (idx, provider) in provider_list.providers.iter().enumerate() {
    //     info!("[DEBUG] Provider {}: id={}, name={}, type={:?}, is_active={}, is_enabled={}", 
    //           idx, provider.id, provider.name, provider.provider_type, provider.is_active, provider.is_enabled);
    // }
    
    // info!("[DEBUG] 准备返回供应商列表，开始序列化为ProtoBuf");
    data_result_ok(provider_list)
}

/// 创建网络搜索供应商处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn create_web_search_provider_handler(
    data: AFPluginData<CreateWebSearchProviderRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchProviderConfigPB, FlowyError> {
    let data = data.into_inner();
    data.validate()?;
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    let created_provider = web_search_hub.create_provider(data).await?;
    
    data_result_ok(created_provider)
}

/// 获取网络搜索供应商处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_web_search_provider_handler(
    data: AFPluginData<GetWebSearchProviderRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchProviderConfigPB, FlowyError> {
    let data = data.into_inner();
    data.validate()?;
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    let provider = web_search_hub.get_provider(&data.id).await?;
    
    data_result_ok(provider)
}

/// 更新网络搜索供应商处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_web_search_provider_handler(
    data: AFPluginData<UpdateWebSearchProviderRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchProviderConfigPB, FlowyError> {
    let data = data.into_inner();
    data.validate()?;
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    let update_config = UpdateWebSearchProviderRequestPB {
        id: data.id,
        name: data.name,
        description: data.description,
        icon: data.icon,
        api_key: data.api_key,
        base_url: data.base_url,
        is_active: data.is_active,
        is_enabled: data.is_enabled,
        max_results: data.max_results,
        timeout_seconds: data.timeout_seconds,
        metadata: data.metadata,
    };
    
    let updated_provider = web_search_hub.update_provider(update_config).await?;
    
    data_result_ok(updated_provider)
}

/// 删除网络搜索供应商处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn delete_web_search_provider_handler(
    data: AFPluginData<DeleteWebSearchProviderRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
    let data = data.try_into_inner()?;
    data.validate()?;
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    web_search_hub.delete_provider(&data.id).await?;
    
    Ok(())
}

/// 测试网络搜索供应商处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn test_web_search_provider_handler(
    data: AFPluginData<TestWebSearchProviderRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<TestWebSearchProviderResponsePB, FlowyError> {
    let data = data.into_inner();
    data.validate()?;
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    let test_request = crate::web_search::entities::TestWebSearchProviderRequestPB {
        id: data.id,
    };
    
    let test_response = web_search_hub.test_provider(test_request).await?;
    
    let response = TestWebSearchProviderResponsePB {
        success: test_response.success,
        error_message: test_response.error_message,
        response_time_ms: test_response.response_time_ms,
        test_results: test_response.test_results.into_iter().map(|result| {
            WebSearchTestResultPB {
                test_name: result.test_name,
                success: result.success,
                error_message: result.error_message,
                response_time_ms: result.response_time_ms,
                details: result.details,
            }
        }).collect(),
    };
    
    data_result_ok(response)
}

/// 执行网络搜索处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn execute_web_search_handler(
    data: AFPluginData<WebSearchRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchResponsePB, FlowyError> {
    let data = data.into_inner();
    data.validate()?;
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    let search_request = crate::web_search::entities::WebSearchRequestPB {
        query: data.query,
        provider_id: data.provider_id,
        max_results: data.max_results,
        language: data.language,
        region: data.region,
        include_content: data.include_content,
        metadata: data.metadata,
    };
    
    let search_response = web_search_hub.search(search_request).await?;
    
    let response = WebSearchResponsePB {
        query: search_response.query,
        results: search_response.results.into_iter().map(|result| {
            WebSearchResultPB {
                id: result.id,
                title: result.title,
                url: result.url,
                snippet: result.snippet,
                content: result.content,
                published_date: result.published_date,
                domain: result.domain,
                language: result.language,
                relevance_score: result.relevance_score,
                metadata: result.metadata,
            }
        }).collect(),
        provider_id: search_response.provider_id,
        execution_time_ms: search_response.execution_time_ms,
        total_results: search_response.total_results,
        success: search_response.success,
        error_message: search_response.error_message,
        metadata: search_response.metadata,
    };
    
    data_result_ok(response)
}

/// 获取网络搜索全局配置处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_web_search_global_config_handler(
    _data: AFPluginData<EmptyRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchGlobalConfigPB, FlowyError> {
    // info!("[DEBUG] 开始处理获取网络搜索全局配置请求");
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    // info!("[DEBUG] AI Manager升级成功");
    
    let web_search_hub = match ai_manager.get_web_search_hub().await {
        Ok(hub) => {
            // info!("[DEBUG] WebSearchHub获取成功");
            hub
        },
        Err(e) => {
            error!("获取WebSearchHub失败: {}", e);
            // 返回默认配置而不是错误，让UI能够正常显示
            let default_config = WebSearchGlobalConfigPB::default_config();
            // info!("[DEBUG] 返回默认全局配置");
            return data_result_ok(default_config);
        }
    };
    
    // info!("[DEBUG] 开始从Hub获取全局配置");
    let global_config = match web_search_hub.get_global_config().await {
        Ok(config) => {
            // info!("[DEBUG] 成功获取全局配置: enabled={}, default_max_results={}, enable_cache={}", 
            //       config.enabled, config.default_max_results, config.enable_cache);
            config
        },
        Err(e) => {
            error!("获取全局配置失败: {}", e);
            return Err(e);
        }
    };
    
    // info!("[DEBUG] 准备返回全局配置，开始序列化为ProtoBuf");
    data_result_ok(global_config)
}

/// 更新网络搜索全局配置处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_web_search_global_config_handler(
    data: AFPluginData<UpdateWebSearchGlobalConfigRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchGlobalConfigPB, FlowyError> {
    let data = data.into_inner();
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    let update_request = crate::web_search::entities::UpdateWebSearchGlobalConfigRequestPB {
        enabled: data.enabled,
        default_provider_id: data.default_provider_id,
        default_max_results: data.default_max_results,
        default_timeout_seconds: data.default_timeout_seconds,
        enable_cache: data.enable_cache,
        cache_expiry_seconds: data.cache_expiry_seconds,
        enable_content_filter: data.enable_content_filter,
        content_filter_rules: data.content_filter_rules,
        metadata: data.metadata,
    };
    
    let _updated_config = web_search_hub.update_global_config(update_request).await?;
    
    // 获取更新后的配置
    let updated_config = web_search_hub.get_global_config().await?;
    
    let response = WebSearchGlobalConfigPB {
        enabled: updated_config.enabled,
        default_provider_id: updated_config.default_provider_id,
        default_max_results: updated_config.default_max_results,
        default_timeout_seconds: updated_config.default_timeout_seconds,
        enable_cache: updated_config.enable_cache,
        cache_expiry_seconds: updated_config.cache_expiry_seconds,
        enable_content_filter: updated_config.enable_content_filter,
        content_filter_rules: updated_config.content_filter_rules,
        created_at: updated_config.created_at,
        updated_at: updated_config.updated_at,
        metadata: updated_config.metadata,
    };
    
    data_result_ok(response)
}

/// 获取网络搜索缓存统计处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_web_search_cache_stats_handler(
    _data: AFPluginData<EmptyRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<WebSearchCacheStatsPB, FlowyError> {
    // info!("[DEBUG] 开始处理获取网络搜索缓存统计请求");
    
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    // info!("[DEBUG] AI Manager升级成功");
    
    let web_search_hub = match ai_manager.get_web_search_hub().await {
        Ok(hub) => {
            // info!("[DEBUG] WebSearchHub获取成功");
            hub
        },
        Err(e) => {
            error!("获取WebSearchHub失败: {}", e);
            // 返回空统计而不是错误，让UI能够正常显示
            let response = WebSearchCacheStatsPB {
                total_entries: 0,
                hit_count: 0,
                miss_count: 0,
                hit_rate: 0.0,
                cache_size_bytes: 0,
                last_cleanup_at: None,
            };
            // info!("[DEBUG] 返回空缓存统计");
            return data_result_ok(response);
        }
    };
    
    // info!("[DEBUG] 开始从Hub获取缓存统计");
    let cache_stats = match web_search_hub.get_cache_stats().await {
        Ok(stats) => {
            // info!("[DEBUG] 成功获取缓存统计: total_entries={}, hits={}, misses={}", 
            //       stats.total_entries, stats.hits, stats.misses);
            stats
        },
        Err(e) => {
            error!("获取缓存统计失败: {}", e);
            return Err(e);
        }
    };
    
    use std::time::UNIX_EPOCH;
    
    let response = WebSearchCacheStatsPB {
        total_entries: cache_stats.total_entries as i64,
        hit_count: cache_stats.hits as i64,
        miss_count: cache_stats.misses as i64,
        hit_rate: cache_stats.hit_rate(),
        cache_size_bytes: cache_stats.cache_size_bytes as i64,
        last_cleanup_at: cache_stats.last_cleanup.map(|t| {
            t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
        }),
    };
    
    // info!("[DEBUG] 准备返回缓存统计: total_entries={}, hit_count={}, miss_count={}, hit_rate={}", 
    //       response.total_entries, response.hit_count, response.miss_count, response.hit_rate);
    // info!("[DEBUG] 开始序列化为ProtoBuf");
    
    data_result_ok(response)
}

/// 清空网络搜索缓存处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn clear_web_search_cache_handler(
    _data: AFPluginData<EmptyRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    let web_search_hub = ai_manager.get_web_search_hub().await?;
    
    web_search_hub.clear_cache().await?;
    
    Ok(())
}

/// 升级 AI 管理器的辅助函数
fn upgrade_ai_manager(ai_manager: AFPluginState<Weak<AIManager>>) -> Result<Arc<AIManager>, FlowyError> {
    ai_manager
        .upgrade()
        .ok_or_else(|| FlowyError::internal().with_context("AI manager has been dropped"))
}
