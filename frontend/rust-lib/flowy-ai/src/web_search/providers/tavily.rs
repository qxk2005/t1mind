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

/// Tavily API 搜索请求
#[derive(Debug, Serialize)]
struct TavilySearchRequest {
    query: String,
    search_depth: String,
    include_answer: bool,
    include_images: bool,
    include_raw_content: bool,
    max_results: i32,
    include_domains: Vec<String>,
    exclude_domains: Vec<String>,
    category: Option<String>,
}

/// Tavily API 搜索结果
#[derive(Debug, Deserialize)]
struct TavilySearchResult {
    title: String,
    url: String,
    content: String,
    score: f64,
    published_date: Option<String>,
}

/// Tavily API 搜索响应
#[derive(Debug, Deserialize)]
struct TavilySearchResponse {
    query: String,
    follow_up_questions: Option<Vec<String>>,
    answer: Option<String>,
    images: Option<Vec<String>>,
    results: Option<Vec<TavilySearchResult>>,
    response_time: f64,
}

/// Tavily API 错误响应
#[derive(Debug, Deserialize)]
struct TavilyErrorResponse {
    error: String,
    message: Option<String>,
}

/// Tavily 搜索供应商实现
pub struct TavilySearchProvider {
    config: WebSearchProviderConfigPB,
    client: Client,
    base_url: String,
}

impl TavilySearchProvider {
    /// 创建新的 Tavily 搜索供应商
    pub fn new(config: WebSearchProviderConfigPB) -> FlowyResult<Self> {
        if config.provider_type != WebSearchProviderTypePB::Tavily {
            return Err(FlowyError::invalid_data()
                .with_context("Invalid provider type for Tavily search provider"));
        }

        if config.api_key.trim().is_empty() {
            return Err(FlowyError::invalid_data()
                .with_context("Tavily API key is required"));
        }

        if config.base_url.trim().is_empty() {
            return Err(FlowyError::invalid_data()
                .with_context("Tavily base URL is required"));
        }

        // 验证 URL 格式
        let _ = Url::parse(&config.base_url)
            .map_err(|e| FlowyError::invalid_data()
                .with_context(format!("Invalid Tavily base URL: {}", e)))?;

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

        // 构建 Tavily API 请求
        let tavily_request = TavilySearchRequest {
            query: request.query.clone(),
            search_depth: "basic".to_string(),
            include_answer: true,
            include_images: false,
            include_raw_content: false,
            max_results: request.max_results.max(1).min(20), // Tavily 限制最多 20 个结果
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
            category: None,
        };

        // 发送请求到 Tavily API
        let response = self.send_search_request(tavily_request).await?;
        
        let execution_time = start_time.elapsed().as_millis() as i64;

        // 转换响应格式
        let mut search_response = WebSearchResponsePB::new(request.query, self.config.id.clone());
        search_response.execution_time_ms = execution_time;
        
        // 处理搜索结果，如果 results 为 None 则使用空数组
        let results = response.results.unwrap_or_default();
        search_response.total_results = results.len() as i64;

        // 转换搜索结果
        for tavily_result in results {
            let mut result = WebSearchResultPB::new(
                tavily_result.title,
                tavily_result.url.clone(),
                tavily_result.content,
            );
            
            result.relevance_score = tavily_result.score;
            result.domain = self.extract_domain(&tavily_result.url);
            result.language = "en".to_string(); // Tavily 主要支持英文
            
            // 解析发布时间
            if let Some(published_date) = tavily_result.published_date {
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(&published_date) {
                    result.published_date = Some(parsed_date.timestamp());
                }
            }

            search_response.add_result(result);
        }

        // 添加答案（如果有）
        if let Some(answer) = response.answer {
            search_response.metadata.insert("answer".to_string(), answer);
        }

        // 添加后续问题（如果有）
        if let Some(follow_up_questions) = &response.follow_up_questions {
            if !follow_up_questions.is_empty() {
                search_response.metadata.insert(
                    "follow_up_questions".to_string(),
                    follow_up_questions.join("; ")
                );
            }
        }

        info!("Tavily search completed: {} results in {}ms", 
              search_response.result_count(), execution_time);

        Ok(search_response)
    }

    /// 发送搜索请求到 Tavily API
    async fn send_search_request(&self, request: TavilySearchRequest) -> FlowyResult<TavilySearchResponse> {
        let url = format!("{}/search", self.base_url.trim_end_matches('/'));
        
        debug!("Sending Tavily search request to: {}", url);
        debug!("Request payload: {:?}", request);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Tavily API: {}", e);
                FlowyError::internal().with_context(format!("Tavily API 请求失败: {}", e))
            })?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Tavily API error response: {} - {}", status, error_text);
            
            // 尝试解析错误响应
            if let Ok(error_response) = serde_json::from_str::<TavilyErrorResponse>(&error_text) {
                return Err(FlowyError::internal()
                    .with_context(format!("Tavily API 错误: {} - {}", 
                        error_response.error, 
                        error_response.message.unwrap_or_default())));
            }
            
            return Err(FlowyError::internal()
                .with_context(format!("Tavily API 返回错误状态: {} - {}", status, error_text)));
        }

        let tavily_response: TavilySearchResponse = response
            .json()
            .await
            .map_err(|e| {
                error!("Failed to parse Tavily API response: {}", e);
                FlowyError::internal().with_context(format!("解析 Tavily API 响应失败: {}", e))
            })?;

        debug!("Tavily API response received: {} results", tavily_response.results.as_ref().map_or(0, |r| r.len()));
        Ok(tavily_response)
    }

    /// 验证 API 密钥
    pub async fn validate_api_key(&self) -> FlowyResult<()> {
        // 使用一个简单的搜索请求来验证 API 密钥
        let test_request = TavilySearchRequest {
            query: "test".to_string(),
            search_depth: "basic".to_string(),
            include_answer: false,
            include_images: false,
            include_raw_content: false,
            max_results: 1,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
            category: None,
        };

        match self.send_search_request(test_request).await {
            Ok(_) => {
                info!("Tavily API key validation successful");
                Ok(())
            }
            Err(e) => {
                error!("Tavily API key validation failed: {}", e);
                Err(e)
            }
        }
    }

    /// 测试连接
    pub async fn test_connection(&self) -> FlowyResult<()> {
        // 使用一个简单的搜索请求来测试连接，而不是依赖可能不存在的 /health 端点
        let test_request = TavilySearchRequest {
            query: "test".to_string(),
            search_depth: "basic".to_string(),
            include_answer: false,
            include_images: false,
            include_raw_content: false,
            max_results: 1,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
            category: None,
        };

        debug!("Testing Tavily connection with simple search request");

        let response = self.client
            .post(&format!("{}/search", self.base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&test_request)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                error!("Tavily connection test failed: {}", e);
                FlowyError::internal().with_context(format!("Tavily 连接测试失败: {}", e))
            })?;

        let status = response.status();
        
        // 如果状态码是 200 或者 400（参数错误但连接正常），都认为连接成功
        if status.is_success() || status == 400 {
            info!("Tavily connection test successful");
            Ok(())
        } else {
            error!("Tavily connection test failed with status: {}", status);
            Err(FlowyError::internal()
                .with_context(format!("Tavily 连接测试失败，状态码: {}", status)))
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
                    info!("Tavily search functionality test successful: {} results", response.result_count());
                    Ok(())
                } else {
                    error!("Tavily search functionality test failed: no results");
                    Err(FlowyError::internal().with_context("Tavily 搜索功能测试失败：无搜索结果"))
                }
            }
            Err(e) => {
                error!("Tavily search functionality test failed: {}", e);
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
impl crate::web_search::providers::WebSearchProvider for TavilySearchProvider {
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

/// 创建 Tavily 搜索供应商的工厂函数
pub fn create_tavily_provider(config: WebSearchProviderConfigPB) -> FlowyResult<Box<dyn crate::web_search::providers::WebSearchProvider>> {
    let provider = TavilySearchProvider::new(config)?;
    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_tavily_config() -> WebSearchProviderConfigPB {
        WebSearchProviderConfigPB {
            id: "test-tavily".to_string(),
            name: "Test Tavily".to_string(),
            provider_type: WebSearchProviderTypePB::Tavily,
            description: "Test Tavily provider".to_string(),
            icon: "search".to_string(),
            api_key: "test_api_key".to_string(),
            base_url: "https://api.tavily.com".to_string(),
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
    fn test_tavily_provider_creation() {
        let config = create_test_tavily_config();
        let provider = TavilySearchProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_tavily_provider_invalid_type() {
        let mut config = create_test_tavily_config();
        config.provider_type = WebSearchProviderTypePB::BraveSearch;
        let provider = TavilySearchProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_tavily_provider_empty_api_key() {
        let mut config = create_test_tavily_config();
        config.api_key = "".to_string();
        let provider = TavilySearchProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_tavily_provider_invalid_url() {
        let mut config = create_test_tavily_config();
        config.base_url = "invalid_url".to_string();
        let provider = TavilySearchProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_extract_domain() {
        let config = create_test_tavily_config();
        let provider = TavilySearchProvider::new(config).unwrap();
        
        assert_eq!(provider.extract_domain("https://example.com/path"), "example.com");
        assert_eq!(provider.extract_domain("http://test.org"), "test.org");
        assert_eq!(provider.extract_domain("invalid_url"), "");
    }

    #[tokio::test]
    async fn test_tavily_search_request_serialization() {
        let request = TavilySearchRequest {
            query: "test query".to_string(),
            search_depth: "basic".to_string(),
            include_answer: true,
            include_images: false,
            include_raw_content: false,
            max_results: 5,
            include_domains: vec!["example.com".to_string()],
            exclude_domains: vec!["spam.com".to_string()],
            category: Some("general".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test query"));
        assert!(json.contains("basic"));
        assert!(json.contains("include_answer"));
    }

    #[tokio::test]
    async fn test_tavily_search_response_deserialization() {
        let json = r#"{
            "query": "test query",
            "follow_up_questions": ["What is test?"],
            "answer": "Test is a procedure",
            "images": [],
            "results": [
                {
                    "title": "Test Result",
                    "url": "https://example.com/test",
                    "content": "This is test content",
                    "score": 0.95,
                    "published_date": "2023-01-01T00:00:00Z"
                }
            ],
            "response_time": 0.5
        }"#;

        let response: TavilySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.query, "test query");
        assert_eq!(response.results.as_ref().unwrap().len(), 1);
        assert_eq!(response.results.as_ref().unwrap()[0].title, "Test Result");
        assert_eq!(response.results.as_ref().unwrap()[0].score, 0.95);
    }
}
