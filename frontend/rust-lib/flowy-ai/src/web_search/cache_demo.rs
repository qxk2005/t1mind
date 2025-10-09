use crate::web_search::entities::*;
use crate::web_search::cache_manager::*;
use crate::web_search::enhanced_cache_manager::*;
use flowy_sqlite::kv::KVStorePreferences;
use flowy_sqlite::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// 演示基础缓存管理器的使用
pub fn demonstrate_basic_cache_manager() {
    println!("=== 基础缓存管理器演示 ===");
    
    // 创建临时目录和存储
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path().to_str().unwrap();
    let store = Arc::new(KVStorePreferences::new(path).unwrap());
    
    // 创建缓存管理器
    let cache_manager = WebSearchCacheManager::new(store);
    
    // 创建测试数据
    let query = "Rust programming language";
    let provider_id = "tavily";
    let mut params = HashMap::new();
    params.insert("max_results".to_string(), "10".to_string());
    params.insert("language".to_string(), "en".to_string());
    
    let mut search_response = WebSearchResponsePB::new(query.to_string(), provider_id.to_string());
    search_response.add_result(WebSearchResultPB::new(
        "The Rust Programming Language".to_string(),
        "https://doc.rust-lang.org/book/".to_string(),
        "The official Rust book for learning Rust programming.".to_string(),
    ));
    search_response.add_result(WebSearchResultPB::new(
        "Rust by Example".to_string(),
        "https://doc.rust-lang.org/rust-by-example/".to_string(),
        "Learn Rust with examples.".to_string(),
    ));
    
    // 存储到缓存
    println!("存储搜索结果到缓存...");
    cache_manager.store_cached_result(query, provider_id, &params, search_response.clone()).unwrap();
    
    // 从缓存获取
    println!("从缓存获取搜索结果...");
    let cached_result = cache_manager.get_cached_result(query, provider_id, &params).unwrap();
    
    if let Some(result) = cached_result {
        println!("✅ 缓存命中！");
        println!("查询: {}", result.query);
        println!("结果数量: {}", result.results.len());
        for (i, search_result) in result.results.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, search_result.title, search_result.url);
        }
    } else {
        println!("❌ 缓存未命中");
    }
    
    // 获取缓存统计
    let stats = cache_manager.get_cache_stats();
    println!("\n缓存统计:");
    println!("  命中次数: {}", stats.hits);
    println!("  未命中次数: {}", stats.misses);
    println!("  命中率: {:.2}%", stats.hit_rate() * 100.0);
    println!("  总条目数: {}", stats.total_entries);
    
    // 检查缓存健康状态
    let health_report = cache_manager.check_cache_health().unwrap();
    println!("\n缓存健康报告:");
    println!("  健康评分: {:.1}/100", health_report.health_score);
    println!("  命中率: {:.2}%", health_report.hit_rate * 100.0);
    println!("  缓存大小: {} 字节", health_report.cache_size_bytes);
    
    if !health_report.recommendations.is_empty() {
        println!("  建议:");
        for recommendation in health_report.recommendations {
            println!("    - {}", recommendation);
        }
    }
}

/// 演示增强缓存管理器的使用
pub fn demonstrate_enhanced_cache_manager() {
    println!("\n=== 增强缓存管理器演示 ===");
    
    // 创建临时目录和数据库
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path().to_str().unwrap();
    let db = flowy_sqlite::init(path).unwrap();
    let db_arc = Arc::new(db);
    
    // 创建增强缓存管理器
    let cache_manager = EnhancedWebSearchCacheManager::new(db_arc);
    
    // 创建测试数据
    let queries = vec![
        ("Rust programming", "tavily"),
        ("Flutter development", "brave"),
        ("Machine learning", "tavily"),
        ("Web development", "brave"),
    ];
    
    for (query, provider_id) in &queries {
        let mut params = HashMap::new();
        params.insert("max_results".to_string(), "5".to_string());
        
        let mut search_response = WebSearchResponsePB::new(query.to_string(), provider_id.to_string());
        search_response.add_result(WebSearchResultPB::new(
            format!("{} - Best Practices", query),
            format!("https://example.com/{}", query.replace(" ", "-")),
            format!("Learn {} with best practices and examples.", query),
        ));
        
        // 存储到缓存
        cache_manager.store_cached_result(query, provider_id, &params, search_response).unwrap();
        println!("✅ 存储查询 '{}' 到缓存", query);
    }
    
    // 测试缓存命中
    println!("\n测试缓存命中...");
    for (query, provider_id) in queries.iter().take(2) {
        let params = HashMap::new();
        let cached_result = cache_manager.get_cached_result(query, provider_id, &params).unwrap();
        
        if cached_result.is_some() {
            println!("✅ 查询 '{}' 缓存命中", query);
        } else {
            println!("❌ 查询 '{}' 缓存未命中", query);
        }
    }
    
    // 获取数据库缓存统计
    let db_stats = cache_manager.get_database_cache_stats().unwrap();
    println!("\n数据库缓存统计:");
    println!("  总条目数: {}", db_stats.total_entries);
    println!("  总命中次数: {}", db_stats.total_hits);
    println!("  平均命中次数: {:.2}", db_stats.average_hits_per_entry);
    
    // 获取内存缓存统计
    let mem_stats = cache_manager.get_cache_stats();
    println!("\n内存缓存统计:");
    println!("  命中次数: {}", mem_stats.hits);
    println!("  未命中次数: {}", mem_stats.misses);
    println!("  命中率: {:.2}%", mem_stats.hit_rate() * 100.0);
    
    // 清理过期条目
    let cleaned_count = cache_manager.cleanup_expired_entries().unwrap();
    println!("\n清理了 {} 个过期缓存条目", cleaned_count);
}

/// 演示缓存配置和性能优化
pub fn demonstrate_cache_configuration() {
    println!("\n=== 缓存配置和性能优化演示 ===");
    
    // 创建自定义配置
    let config = CacheConfig {
        enabled: true,
        default_ttl_seconds: 1800, // 30分钟
        max_entries: 500,
        cleanup_interval_seconds: 600, // 10分钟
        enable_compression: true,
    };
    
    println!("缓存配置:");
    println!("  启用缓存: {}", config.enabled);
    println!("  默认TTL: {} 秒", config.default_ttl_seconds);
    println!("  最大条目数: {}", config.max_entries);
    println!("  清理间隔: {} 秒", config.cleanup_interval_seconds);
    println!("  启用压缩: {}", config.enable_compression);
    
    // 创建临时目录和存储
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path().to_str().unwrap();
    let store = Arc::new(KVStorePreferences::new(path).unwrap());
    
    // 使用自定义配置创建缓存管理器
    let mut cache_manager = WebSearchCacheManager::with_config(store, config);
    
    // 测试缓存性能
    let query = "performance test";
    let provider_id = "test_provider";
    let params = HashMap::new();
    
    let mut search_response = WebSearchResponsePB::new(query.to_string(), provider_id.to_string());
    for i in 1..=10 {
        search_response.add_result(WebSearchResultPB::new(
            format!("Result {}", i),
            format!("https://example.com/result{}", i),
            format!("This is test result number {}", i),
        ));
    }
    
    // 测量存储性能
    let start = std::time::Instant::now();
    cache_manager.store_cached_result(query, provider_id, &params, search_response).unwrap();
    let store_duration = start.elapsed();
    
    // 测量检索性能
    let start = std::time::Instant::now();
    let cached_result = cache_manager.get_cached_result(query, provider_id, &params).unwrap();
    let retrieve_duration = start.elapsed();
    
    println!("\n性能测试结果:");
    println!("  存储耗时: {:?}", store_duration);
    println!("  检索耗时: {:?}", retrieve_duration);
    println!("  缓存命中: {}", cached_result.is_some());
    
    // 测试缓存大小
    let cache_size = cache_manager.get_cache_size();
    println!("  缓存大小: {} 字节", cache_size);
    
    // 更新配置
    let mut new_config = cache_manager.get_config().clone();
    new_config.max_entries = 1000;
    cache_manager.update_config(new_config);
    println!("\n✅ 更新缓存配置完成");
}

/// 运行所有缓存管理器演示
pub fn run_all_cache_demonstrations() {
    println!("🚀 开始缓存管理器演示...\n");
    
    demonstrate_basic_cache_manager();
    demonstrate_enhanced_cache_manager();
    demonstrate_cache_configuration();
    
    println!("\n✅ 所有缓存管理器演示完成！");
    println!("\n📋 总结:");
    println!("  - 基础缓存管理器：使用 KVStorePreferences 进行简单缓存");
    println!("  - 增强缓存管理器：使用 SQLite 数据库进行高性能缓存");
    println!("  - 缓存配置：支持自定义 TTL、大小限制等配置");
    println!("  - 性能优化：内存缓存 + 持久化存储的混合策略");
    println!("  - 健康监控：提供缓存健康状态和优化建议");
}
