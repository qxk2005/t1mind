// 搜索结果处理器演示
// 这个文件展示了如何使用 WebSearchResultProcessor 来处理不同供应商的搜索结果

use crate::web_search::{
    WebSearchResultProcessor, CitationInfo,
};
use crate::web_search::entities::{
    WebSearchRequestPB, WebSearchResponsePB, WebSearchResultPB,
};
use crate::entities::WebSearchProviderTypePB;

/// 演示搜索结果处理器的使用
pub fn demonstrate_result_processor() {
    println!("=== 搜索结果处理器演示 ===");
    
    // 创建处理器
    let processor = WebSearchResultProcessor::new();
    
    // 模拟 Tavily 搜索结果
    let tavily_response = r#"{
        "query": "Rust programming",
        "follow_up_questions": ["What is Rust?", "How to learn Rust?"],
        "answer": "Rust is a systems programming language",
        "images": [],
        "results": [
            {
                "title": "The Rust Programming Language",
                "url": "https://doc.rust-lang.org/book/",
                "content": "The official Rust book",
                "score": 0.95,
                "published_date": "2023-01-01"
            },
            {
                "title": "Rust by Example",
                "url": "https://doc.rust-lang.org/rust-by-example/",
                "content": "Learn Rust with examples",
                "score": 0.88,
                "published_date": "2023-02-01"
            }
        ],
        "response_time": 1.2
    }"#;
    
    // 创建搜索请求
    let request = WebSearchRequestPB {
        query: "Rust programming".to_string(),
        provider_id: Some("tavily-1".to_string()),
        max_results: 10,
        language: "en".to_string(),
        region: "US".to_string(),
        include_content: true,
        metadata: std::collections::HashMap::new(),
    };
    
    // 处理 Tavily 结果
    match processor.process_tavily_results(tavily_response, &request, "tavily-1") {
        Ok(response) => {
            println!("✅ Tavily 结果处理成功:");
            println!("   - 查询: {}", response.query);
            println!("   - 结果数量: {}", response.results.len());
            println!("   - 执行时间: {}ms", response.execution_time_ms);
            println!("   - 成功状态: {}", response.success);
            
            for (i, result) in response.results.iter().enumerate() {
                println!("   - 结果 {}: {} (相关性: {:.2})", 
                    i + 1, result.title, result.relevance_score);
            }
            
            // 提取引用信息
            let citations = processor.extract_citations(&response);
            println!("   - 引用数量: {}", citations.len());
            
            // 格式化引用
            let formatted_citations = processor.format_citations(&citations, 5);
            println!("   - 格式化引用:\n{}", formatted_citations);
        }
        Err(e) => {
            println!("❌ Tavily 结果处理失败: {}", e);
        }
    }
    
    // 模拟 Brave Search 搜索结果
    let brave_response = r#"{
        "type": "search",
        "results": [
            {
                "title": "Rust Programming Language",
                "url": "https://www.rust-lang.org/",
                "description": "A language empowering everyone to build reliable and efficient software.",
                "extra_snippets": ["Memory safety", "Zero-cost abstractions"],
                "page_age": "2023-12-01",
                "language": "en",
                "location": "US",
                "family_friendly": true
            },
            {
                "title": "Rust Documentation",
                "url": "https://doc.rust-lang.org/",
                "description": "Comprehensive documentation for Rust",
                "extra_snippets": ["API reference", "Guides"],
                "page_age": "2023-11-15",
                "language": "en",
                "location": "US",
                "family_friendly": true
            }
        ],
        "web": {
            "results": [
                {
                    "title": "Rust Programming Language",
                    "url": "https://www.rust-lang.org/",
                    "description": "A language empowering everyone to build reliable and efficient software.",
                    "extra_snippets": ["Memory safety", "Zero-cost abstractions"],
                    "page_age": "2023-12-01",
                    "language": "en",
                    "location": "US",
                    "family_friendly": true
                }
            ]
        }
    }"#;
    
    // 处理 Brave Search 结果
    match processor.process_brave_results(brave_response, &request, "brave-1") {
        Ok(response) => {
            println!("\n✅ Brave Search 结果处理成功:");
            println!("   - 查询: {}", response.query);
            println!("   - 结果数量: {}", response.results.len());
            println!("   - 执行时间: {}ms", response.execution_time_ms);
            println!("   - 成功状态: {}", response.success);
            
            for (i, result) in response.results.iter().enumerate() {
                println!("   - 结果 {}: {} (相关性: {:.2})", 
                    i + 1, result.title, result.relevance_score);
            }
            
            // 提取引用信息
            let citations = processor.extract_citations(&response);
            println!("   - 引用数量: {}", citations.len());
            
            // 格式化引用
            let formatted_citations = processor.format_citations(&citations, 5);
            println!("   - 格式化引用:\n{}", formatted_citations);
        }
        Err(e) => {
            println!("❌ Brave Search 结果处理失败: {}", e);
        }
    }
    
    // 演示通用结果处理
    let generic_response = r#"{
        "results": [
            {
                "title": "Generic Rust Tutorial",
                "url": "https://example.com/rust-tutorial",
                "description": "A comprehensive Rust tutorial",
                "score": 0.75,
                "published_date": "2023-10-01",
                "language": "en"
            }
        ]
    }"#;
    
    match processor.process_generic_results(
        generic_response, 
        &request, 
        "generic-1", 
        WebSearchProviderTypePB::Custom
    ) {
        Ok(response) => {
            println!("\n✅ 通用结果处理成功:");
            println!("   - 查询: {}", response.query);
            println!("   - 结果数量: {}", response.results.len());
            println!("   - 执行时间: {}ms", response.execution_time_ms);
            println!("   - 成功状态: {}", response.success);
            
            for (i, result) in response.results.iter().enumerate() {
                println!("   - 结果 {}: {} (相关性: {:.2})", 
                    i + 1, result.title, result.relevance_score);
            }
        }
        Err(e) => {
            println!("❌ 通用结果处理失败: {}", e);
        }
    }
    
    println!("\n=== 演示完成 ===");
}

/// 演示引用信息的创建和格式化
pub fn demonstrate_citations() {
    println!("\n=== 引用信息演示 ===");
    
    // 创建引用信息
    let citation = CitationInfo::new(
        1,
        "Rust Programming Language".to_string(),
        "https://www.rust-lang.org/".to_string(),
        "rust-lang.org".to_string(),
        "A language empowering everyone to build reliable and efficient software.".to_string(),
        0.95,
    );
    
    println!("引用信息:");
    println!("  - 索引: {}", citation.index);
    println!("  - 标题: {}", citation.title);
    println!("  - URL: {}", citation.url);
    println!("  - 域名: {}", citation.domain);
    println!("  - 摘要: {}", citation.snippet);
    println!("  - 相关性评分: {}", citation.relevance_score);
    
    println!("\n格式化输出:");
    println!("  - 完整格式: {}", citation.format());
    println!("  - 短格式: {}", citation.format_short());
    
    println!("\n=== 引用演示完成 ===");
}
