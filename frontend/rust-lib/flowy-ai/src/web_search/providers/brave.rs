use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use flowy_error::{FlowyError, FlowyResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use url::Url;

use crate::web_search::entities::{
    WebSearchProviderConfigPB, WebSearchRequestPB,
    WebSearchResponsePB, WebSearchResultPB,
    WebSearchTestResultPB, TestWebSearchProviderResponsePB,
};
use crate::entities::{WebSearchProviderTypePB, ProviderTestStatusPB};

/// Brave Search API 搜索请求
#[derive(Debug, Serialize)]
struct BraveSearchRequest {
    q: String,
    count: Option<i32>,
    offset: Option<i32>,
    mkt: Option<String>,
    safesearch: Option<String>,
    freshness: Option<String>,
    text_decorations: Option<bool>,
    text_format: Option<String>,
}

/// Brave Search API 搜索结果
#[derive(Debug, Deserialize)]
struct BraveSearchResult {
    title: String,
    url: String,
    description: String,
    #[serde(rename = "extra_snippets")]
    extra_snippets: Option<Vec<String>>,
    #[serde(rename = "page_age")]
    page_age: Option<String>,
    #[serde(rename = "language")]
    language: Option<String>,
    #[serde(rename = "location")]
    location: Option<String>,
    #[serde(rename = "family_friendly")]
    family_friendly: Option<bool>,
}

/// Brave Search API 搜索响应
#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    #[serde(rename = "type")]
    response_type: String,
    results: Vec<BraveSearchResult>,
    web: Option<BraveWebResults>,
}

/// Brave Search API Web 结果
#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveSearchResult>,
}

/// Brave Search API 错误响应
#[derive(Debug, Deserialize)]
struct BraveErrorResponse {
    error: String,
    message: Option<String>,
}

/// Brave Search 搜索供应商实现
pub struct BraveSearchProvider {
    config: WebSearchProviderConfigPB,
    client: Client,
    base_url: String,
}

impl BraveSearchProvider {
    /// 创建新的 Brave Search 搜索供应商
    pub fn new(config: WebSearchProviderConfigPB) -> FlowyResult<Self> {
        if config.provider_type != WebSearchProviderTypePB::BraveSearch {
            return Err(FlowyError::invalid_data()
                .with_context("Invalid provider type for Brave Search provider"));
        }

        if config.api_key.trim().is_empty() {
            return Err(FlowyError::invalid_data()
                .with_context("Brave Search API key is required"));
        }

        if config.base_url.trim().is_empty() {
            return Err(FlowyError::invalid_data()
                .with_context("Brave Search base URL is required"));
        }

        // 验证 URL 格式
        let _ = Url::parse(&config.base_url)
            .map_err(|e| FlowyError::invalid_data()
                .with_context(format!("Invalid Brave Search base URL: {}", e)))?;

        // 创建 HTTP 客户端
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("AppFlowy/1.0")
            .build()
            .map_err(|e| FlowyError::internal()
                .with_context(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config: config.clone(),
            client,
            base_url: config.base_url.clone(),
        })
    }

    /// 执行搜索
    pub async fn search(&self, request: WebSearchRequestPB) -> FlowyResult<WebSearchResponsePB> {
        let start_time = std::time::Instant::now();

        // 构建 Brave Search API 请求
        let brave_request = BraveSearchRequest {
            q: request.query.clone(),
            count: Some(request.max_results.max(1).min(20)), // Brave Search 限制最多 20 个结果
            offset: Some(0),
            mkt: Some(request.region.clone()),
            safesearch: Some("moderate".to_string()),
            freshness: None,
            text_decorations: Some(true),
            text_format: Some("Raw".to_string()),
        };

        // 发送请求到 Brave Search API
        let response = self.send_search_request(brave_request).await?;
        
        let execution_time = start_time.elapsed().as_millis() as i64;

        // 转换响应格式
        let mut search_response = WebSearchResponsePB::new(request.query, self.config.id.clone());
        search_response.execution_time_ms = execution_time;

        // 提取搜索结果
        let results = if let Some(web_results) = response.web {
            web_results.results
        } else {
            response.results
        };

        search_response.total_results = results.len() as i64;

        // 转换搜索结果
        for brave_result in results {
            let mut result = WebSearchResultPB::new(
                brave_result.title,
                brave_result.url.clone(),
                brave_result.description,
            );
            
            result.domain = self.extract_domain(&brave_result.url);
            result.language = brave_result.language.unwrap_or_else(|| request.language.clone());
            result.relevance_score = 0.8; // Brave Search 不提供相关性评分，使用默认值
            
            // 添加额外摘要（如果有）
            if let Some(extra_snippets) = brave_result.extra_snippets {
                if !extra_snippets.is_empty() {
                    result.metadata.insert("extra_snippets".to_string(), extra_snippets.join("; "));
                }
            }

            // 添加页面年龄信息
            if let Some(page_age) = brave_result.page_age {
                result.metadata.insert("page_age".to_string(), page_age);
            }

            // 添加位置信息
            if let Some(location) = brave_result.location {
                result.metadata.insert("location".to_string(), location);
            }

            // 添加家庭友好标记
            if let Some(family_friendly) = brave_result.family_friendly {
                result.metadata.insert("family_friendly".to_string(), family_friendly.to_string());
            }

            search_response.add_result(result);
        }

        info!("Brave Search completed: {} results in {}ms", 
              search_response.result_count(), execution_time);

        Ok(search_response)
    }

    /// 发送搜索请求到 Brave Search API
    async fn send_search_request(&self, request: BraveSearchRequest) -> FlowyResult<BraveSearchResponse> {
        let url = format!("{}/web/search", self.base_url.trim_end_matches('/'));
        
        debug!("Sending Brave Search request to: {}", url);
        debug!("Request payload: {:?}", request);

        let response = self.client
            .get(&url)
            .header("X-Subscription-Token", &self.config.api_key)
            .query(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Brave Search API: {}", e);
                FlowyError::internal().with_context(format!("Brave Search API 请求失败: {}", e))
            })?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Brave Search API error response: {} - {}", status, error_text);
            
            // 尝试解析错误响应
            if let Ok(error_response) = serde_json::from_str::<BraveErrorResponse>(&error_text) {
                return Err(FlowyError::internal()
                    .with_context(format!("Brave Search API 错误: {} - {}", 
                        error_response.error, 
                        error_response.message.unwrap_or_default())));
            }
            
            return Err(FlowyError::internal()
                .with_context(format!("Brave Search API 返回错误状态: {} - {}", status, error_text)));
        }

        let brave_response: BraveSearchResponse = response
            .json()
            .await
            .map_err(|e| {
                error!("Failed to parse Brave Search API response: {}", e);
                FlowyError::internal().with_context(format!("解析 Brave Search API 响应失败: {}", e))
            })?;

        debug!("Brave Search API response received: {} results", 
               brave_response.web.as_ref().map(|w| w.results.len()).unwrap_or(brave_response.results.len()));
        Ok(brave_response)
    }

    /// 验证 API 密钥
    pub async fn validate_api_key(&self) -> FlowyResult<()> {
        // 使用一个简单的搜索请求来验证 API 密钥
        let test_request = BraveSearchRequest {
            q: "test".to_string(),
            count: Some(1),
            offset: Some(0),
            mkt: Some("en-US".to_string()),
            safesearch: Some("moderate".to_string()),
            freshness: None,
            text_decorations: Some(true),
            text_format: Some("Raw".to_string()),
        };

        match self.send_search_request(test_request).await {
            Ok(_) => {
                info!("Brave Search API key validation successful");
                Ok(())
            }
            Err(e) => {
                error!("Brave Search API key validation failed: {}", e);
                Err(e)
            }
        }
    }

    /// 测试连接
    pub async fn test_connection(&self) -> FlowyResult<()> {
        // 检查网络连接 - Brave Search API 没有专门的健康检查端点
        // 使用一个简单的搜索请求来测试连接
        let test_request = BraveSearchRequest {
            q: "connection test".to_string(),
            count: Some(1),
            offset: Some(0),
            mkt: Some("en-US".to_string()),
            safesearch: Some("moderate".to_string()),
            freshness: None,
            text_decorations: Some(true),
            text_format: Some("Raw".to_string()),
        };

        debug!("Testing Brave Search connection");

        match self.send_search_request(test_request).await {
            Ok(_) => {
                info!("Brave Search connection test successful");
                Ok(())
            }
            Err(e) => {
                error!("Brave Search connection test failed: {}", e);
                Err(FlowyError::internal()
                    .with_context(format!("Brave Search 连接测试失败: {}", e)))
            }
        }
    }

    /// 测试搜索功能
    pub async fn test_search_functionality(&self) -> FlowyResult<()> {
        let test_request = WebSearchRequestPB {
            query: "AppFlowy".to_string(),
            provider_id: Some(self.config.id.clone()),
            max_results: 3,
            language: "en".to_string(),
            region: "us".to_string(),
            include_content: false,
            metadata: HashMap::new(),
        };

        match self.search(test_request).await {
            Ok(response) => {
                if response.is_successful() && response.result_count() > 0 {
                    info!("Brave Search functionality test successful: {} results", response.result_count());
                    Ok(())
                } else {
                    error!("Brave Search functionality test failed: no results");
                    Err(FlowyError::internal().with_context("Brave Search 搜索功能测试失败：无搜索结果"))
                }
            }
            Err(e) => {
                error!("Brave Search functionality test failed: {}", e);
                Err(e)
            }
        }
    }

    /// 执行完整的供应商测试
    pub async fn run_tests(&self) -> FlowyResult<TestWebSearchProviderResponsePB> {
        let start_time = std::time::Instant::now();
        let mut test_results = Vec::new();
        let mut overall_success = true;
        let mut error_message = None;

        // 测试1: API密钥验证
        let test_start = std::time::Instant::now();
        match self.validate_api_key().await {
            Ok(_) => {
                let response_time = test_start.elapsed().as_millis() as i64;
                test_results.push(WebSearchTestResultPB {
                    test_name: "API密钥验证".to_string(),
                    success: true,
                    error_message: None,
                    response_time_ms: response_time,
                    details: "API密钥有效".to_string(),
                });
            }
            Err(e) => {
                let response_time = test_start.elapsed().as_millis() as i64;
                test_results.push(WebSearchTestResultPB {
                    test_name: "API密钥验证".to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                    response_time_ms: response_time,
                    details: "API密钥无效或已过期".to_string(),
                });
                overall_success = false;
                error_message = Some(e.to_string());
            }
        }

        // 测试2: 连接测试
        let test_start = std::time::Instant::now();
        match self.test_connection().await {
            Ok(_) => {
                let response_time = test_start.elapsed().as_millis() as i64;
                test_results.push(WebSearchTestResultPB {
                    test_name: "连接测试".to_string(),
                    success: true,
                    error_message: None,
                    response_time_ms: response_time,
                    details: "连接成功".to_string(),
                });
            }
            Err(e) => {
                let response_time = test_start.elapsed().as_millis() as i64;
                test_results.push(WebSearchTestResultPB {
                    test_name: "连接测试".to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                    response_time_ms: response_time,
                    details: "连接失败".to_string(),
                });
                overall_success = false;
                if error_message.is_none() {
                    error_message = Some(e.to_string());
                }
            }
        }

        // 测试3: 搜索功能测试
        let test_start = std::time::Instant::now();
        match self.test_search_functionality().await {
            Ok(_) => {
                let response_time = test_start.elapsed().as_millis() as i64;
                test_results.push(WebSearchTestResultPB {
                    test_name: "搜索功能测试".to_string(),
                    success: true,
                    error_message: None,
                    response_time_ms: response_time,
                    details: "搜索功能正常".to_string(),
                });
            }
            Err(e) => {
                let response_time = test_start.elapsed().as_millis() as i64;
                test_results.push(WebSearchTestResultPB {
                    test_name: "搜索功能测试".to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                    response_time_ms: response_time,
                    details: "搜索功能异常".to_string(),
                });
                overall_success = false;
                if error_message.is_none() {
                    error_message = Some(e.to_string());
                }
            }
        }

        let total_response_time = start_time.elapsed().as_millis() as i64;

        Ok(TestWebSearchProviderResponsePB {
            success: overall_success,
            error_message,
            response_time_ms: total_response_time,
            test_results,
        })
    }

    /// 提取域名
    fn extract_domain(&self, url: &str) -> String {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                return host.to_string();
            }
        }
        String::new()
    }

    /// 获取供应商配置
    pub fn get_config(&self) -> &WebSearchProviderConfigPB {
        &self.config
    }

    /// 检查供应商是否可用
    pub fn is_available(&self) -> bool {
        self.config.is_available()
    }
}

#[async_trait]
impl crate::web_search::providers::WebSearchProvider for BraveSearchProvider {
    async fn search(&self, request: WebSearchRequestPB) -> FlowyResult<WebSearchResponsePB> {
        self.search(request).await
    }

    async fn run_tests(&self) -> FlowyResult<TestWebSearchProviderResponsePB> {
        self.run_tests().await
    }

    fn get_config(&self) -> &WebSearchProviderConfigPB {
        self.get_config()
    }

    fn is_available(&self) -> bool {
        self.is_available()
    }
}

/// 创建 Brave Search 搜索供应商的工厂函数
pub fn create_brave_search_provider(config: WebSearchProviderConfigPB) -> FlowyResult<Box<dyn crate::web_search::providers::WebSearchProvider>> {
    let provider = BraveSearchProvider::new(config)?;
    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_brave_config() -> WebSearchProviderConfigPB {
        WebSearchProviderConfigPB {
            id: "test-brave".to_string(),
            name: "Test Brave Search".to_string(),
            provider_type: WebSearchProviderTypePB::BraveSearch,
            description: "Test Brave Search provider".to_string(),
            icon: "search".to_string(),
            api_key: "test_api_key".to_string(),
            base_url: "https://api.search.brave.com/res/v1".to_string(),
            is_active: true,
            is_enabled: true,
            max_results: 10,
            timeout_seconds: 30,
            created_at: 1234567890,
            updated_at: 1234567890,
            last_tested_at: None,
            test_status: ProviderTestStatusPB::NotTested,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_brave_search_provider_creation() {
        let config = create_test_brave_config();
        let provider = BraveSearchProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_brave_search_provider_invalid_type() {
        let mut config = create_test_brave_config();
        config.provider_type = WebSearchProviderTypePB::Tavily;
        let provider = BraveSearchProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_brave_search_provider_empty_api_key() {
        let mut config = create_test_brave_config();
        config.api_key = "".to_string();
        let provider = BraveSearchProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_brave_search_provider_invalid_url() {
        let mut config = create_test_brave_config();
        config.base_url = "invalid_url".to_string();
        let provider = BraveSearchProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_extract_domain() {
        let config = create_test_brave_config();
        let provider = BraveSearchProvider::new(config).unwrap();
        
        assert_eq!(provider.extract_domain("https://example.com/path"), "example.com");
        assert_eq!(provider.extract_domain("http://test.org"), "test.org");
        assert_eq!(provider.extract_domain("invalid_url"), "");
    }

    #[tokio::test]
    async fn test_brave_search_request_serialization() {
        let request = BraveSearchRequest {
            q: "test query".to_string(),
            count: Some(5),
            offset: Some(0),
            mkt: Some("en-US".to_string()),
            safesearch: Some("moderate".to_string()),
            freshness: None,
            text_decorations: Some(true),
            text_format: Some("Raw".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test query"));
        assert!(json.contains("count"));
        assert!(json.contains("offset"));
    }

    #[tokio::test]
    async fn test_brave_search_response_deserialization() {
        let json = r#"{
            "type": "search",
            "results": [
                {
                    "title": "Test Result",
                    "url": "https://example.com/test",
                    "description": "This is test content",
                    "extra_snippets": ["Additional info"],
                    "page_age": "2023-01-01",
                    "language": "en",
                    "location": "US",
                    "family_friendly": true
                }
            ],
            "web": {
                "results": [
                    {
                        "title": "Web Test Result",
                        "url": "https://example.com/web",
                        "description": "Web test content"
                    }
                ]
            }
        }"#;

        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.response_type, "search");
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Test Result");
        assert!(response.web.is_some());
        assert_eq!(response.web.unwrap().results.len(), 1);
    }
}
