use std::time::SystemTime;

use flowy_error::{FlowyError, FlowyResult};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use url::Url;

use crate::web_search::entities::{
    WebSearchRequestPB, WebSearchResponsePB, WebSearchResultPB,
};
use crate::entities::WebSearchProviderTypePB;

// ==================== 供应商特定结果结构 ====================

/// Tavily API 搜索结果（内部结构）
#[derive(Debug, Deserialize)]
struct TavilySearchResult {
    title: String,
    url: String,
    content: String,
    score: f64,
    published_date: Option<String>,
}

/// Tavily API 搜索响应（内部结构）
#[derive(Debug, Deserialize)]
struct TavilySearchResponse {
    query: String,
    follow_up_questions: Vec<String>,
    answer: Option<String>,
    images: Vec<String>,
    results: Vec<TavilySearchResult>,
    response_time: f64,
}

/// Brave Search API 搜索结果（内部结构）
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

/// Brave Search API 搜索响应（内部结构）
#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    #[serde(rename = "type")]
    response_type: String,
    results: Vec<BraveSearchResult>,
    web: Option<BraveWebResults>,
}

/// Brave Search API Web 结果（内部结构）
#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveSearchResult>,
}

// ==================== 搜索结果处理器 ====================

/// 搜索结果处理器
/// 
/// 负责将不同供应商的搜索结果转换为统一的格式，
/// 提取引用信息，并进行数据质量检查和格式化。
pub struct WebSearchResultProcessor {
    /// 是否启用内容过滤
    enable_content_filter: bool,
    /// 内容过滤规则
    content_filter_rules: Vec<String>,
    /// 最大结果数量限制
    max_results_limit: usize,
    /// 最小相关性评分阈值
    min_relevance_threshold: f64,
}

impl Default for WebSearchResultProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchResultProcessor {
    /// 创建新的搜索结果处理器
    pub fn new() -> Self {
        Self {
            enable_content_filter: true,
            content_filter_rules: vec![
                "adult_content".to_string(),
                "malware".to_string(),
                "phishing".to_string(),
                "spam".to_string(),
            ],
            max_results_limit: 50,
            min_relevance_threshold: 0.1,
        }
    }

    /// 创建带有自定义配置的搜索结果处理器
    pub fn with_config(
        enable_content_filter: bool,
        content_filter_rules: Vec<String>,
        max_results_limit: usize,
        min_relevance_threshold: f64,
    ) -> Self {
        Self {
            enable_content_filter,
            content_filter_rules,
            max_results_limit,
            min_relevance_threshold,
        }
    }

    /// 处理 Tavily 搜索结果
    /// 
    /// # Arguments
    /// * `raw_response` - Tavily API 的原始响应 JSON 字符串
    /// * `request` - 原始搜索请求
    /// * `provider_id` - 供应商 ID
    /// 
    /// # Returns
    /// 处理后的统一搜索结果响应
    pub fn process_tavily_results(
        &self,
        raw_response: &str,
        request: &WebSearchRequestPB,
        provider_id: &str,
    ) -> FlowyResult<WebSearchResponsePB> {
        debug!("Processing Tavily search results for query: {}", request.query);

        // 解析原始响应
        let tavily_response: TavilySearchResponse = serde_json::from_str(raw_response)
            .map_err(|e| FlowyError::internal()
                .with_context(format!("Failed to parse Tavily response: {}", e)))?;

        let start_time = SystemTime::now();
        let mut response = WebSearchResponsePB::new(request.query.clone(), provider_id.to_string());

        // 处理搜索结果
        let mut processed_results = Vec::new();
        for tavily_result in tavily_response.results {
            if let Ok(processed_result) = self.process_tavily_result(&tavily_result) {
                // 应用内容过滤
                if self.should_include_result(&processed_result) {
                    processed_results.push(processed_result);
                }
            }
        }

        // 按相关性评分排序
        processed_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // 限制结果数量
        processed_results.truncate(self.max_results_limit.min(request.max_results as usize));

        response.results = processed_results;
        response.total_results = response.results.len() as i64;
        response.success = true;

        // 计算执行时间
        let execution_time = start_time.elapsed().unwrap_or_default();
        response.execution_time_ms = execution_time.as_millis() as i64;

        // 添加元数据
        response.metadata.insert("provider".to_string(), "tavily".to_string());
        response.metadata.insert("response_time".to_string(), tavily_response.response_time.to_string());
        if let Some(answer) = tavily_response.answer {
            response.metadata.insert("answer".to_string(), answer);
        }
        if !tavily_response.follow_up_questions.is_empty() {
            response.metadata.insert("follow_up_questions".to_string(), 
                serde_json::to_string(&tavily_response.follow_up_questions).unwrap_or_default());
        }

        info!("Processed {} Tavily results for query: {}", response.results.len(), request.query);
        Ok(response)
    }

    /// 处理 Brave Search 搜索结果
    /// 
    /// # Arguments
    /// * `raw_response` - Brave Search API 的原始响应 JSON 字符串
    /// * `request` - 原始搜索请求
    /// * `provider_id` - 供应商 ID
    /// 
    /// # Returns
    /// 处理后的统一搜索结果响应
    pub fn process_brave_results(
        &self,
        raw_response: &str,
        request: &WebSearchRequestPB,
        provider_id: &str,
    ) -> FlowyResult<WebSearchResponsePB> {
        debug!("Processing Brave Search results for query: {}", request.query);

        // 解析原始响应
        let brave_response: BraveSearchResponse = serde_json::from_str(raw_response)
            .map_err(|e| FlowyError::internal()
                .with_context(format!("Failed to parse Brave Search response: {}", e)))?;

        let start_time = SystemTime::now();
        let mut response = WebSearchResponsePB::new(request.query.clone(), provider_id.to_string());

        // 处理搜索结果
        let mut processed_results = Vec::new();
        
        // 优先处理 web 结果，如果没有则使用通用结果
        let results_to_process = if let Some(web_results) = brave_response.web {
            web_results.results
        } else {
            brave_response.results
        };

        for brave_result in results_to_process {
            if let Ok(processed_result) = self.process_brave_result(&brave_result) {
                // 应用内容过滤
                if self.should_include_result(&processed_result) {
                    processed_results.push(processed_result);
                }
            }
        }

        // 按相关性评分排序
        processed_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // 限制结果数量
        processed_results.truncate(self.max_results_limit.min(request.max_results as usize));

        response.results = processed_results;
        response.total_results = response.results.len() as i64;
        response.success = true;

        // 计算执行时间
        let execution_time = start_time.elapsed().unwrap_or_default();
        response.execution_time_ms = execution_time.as_millis() as i64;

        // 添加元数据
        response.metadata.insert("provider".to_string(), "brave_search".to_string());
        response.metadata.insert("response_type".to_string(), brave_response.response_type);

        info!("Processed {} Brave Search results for query: {}", response.results.len(), request.query);
        Ok(response)
    }

    /// 处理通用搜索结果（用于未来扩展）
    /// 
    /// # Arguments
    /// * `raw_response` - 原始响应 JSON 字符串
    /// * `request` - 原始搜索请求
    /// * `provider_id` - 供应商 ID
    /// * `provider_type` - 供应商类型
    /// 
    /// # Returns
    /// 处理后的统一搜索结果响应
    pub fn process_generic_results(
        &self,
        raw_response: &str,
        request: &WebSearchRequestPB,
        provider_id: &str,
        provider_type: WebSearchProviderTypePB,
    ) -> FlowyResult<WebSearchResponsePB> {
        debug!("Processing generic search results for provider: {:?}", provider_type);

        // 尝试解析为通用 JSON 结构
        let json_value: serde_json::Value = serde_json::from_str(raw_response)
            .map_err(|e| FlowyError::internal()
                .with_context(format!("Failed to parse generic response: {}", e)))?;

        let start_time = SystemTime::now();
        let mut response = WebSearchResponsePB::new(request.query.clone(), provider_id.to_string());

        // 尝试提取结果数组
        let results_array = if let Some(results) = json_value.get("results").and_then(|v| v.as_array()) {
            results
        } else if let Some(results) = json_value.get("data").and_then(|v| v.as_array()) {
            results
        } else if let Some(results) = json_value.get("items").and_then(|v| v.as_array()) {
            results
        } else {
            warn!("No results array found in generic response");
            return Ok(response);
        };

        // 处理每个结果
        let mut processed_results = Vec::new();
        for result_value in results_array {
            if let Ok(processed_result) = self.process_generic_result(result_value) {
                if self.should_include_result(&processed_result) {
                    processed_results.push(processed_result);
                }
            }
        }

        // 按相关性评分排序
        processed_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // 限制结果数量
        processed_results.truncate(self.max_results_limit.min(request.max_results as usize));

        response.results = processed_results;
        response.total_results = response.results.len() as i64;
        response.success = true;

        // 计算执行时间
        let execution_time = start_time.elapsed().unwrap_or_default();
        response.execution_time_ms = execution_time.as_millis() as i64;

        // 添加元数据
        response.metadata.insert("provider".to_string(), format!("{:?}", provider_type).to_lowercase());
        response.metadata.insert("processing_method".to_string(), "generic".to_string());

        info!("Processed {} generic results for query: {}", response.results.len(), request.query);
        Ok(response)
    }

    /// 提取搜索结果中的引用信息
    /// 
    /// # Arguments
    /// * `response` - 搜索结果响应
    /// 
    /// # Returns
    /// 引用信息列表
    pub fn extract_citations(&self, response: &WebSearchResponsePB) -> Vec<CitationInfo> {
        let mut citations = Vec::new();

        for (index, result) in response.results.iter().enumerate() {
            let citation = CitationInfo {
                index: index + 1,
                title: result.title.clone(),
                url: result.url.clone(),
                domain: result.domain.clone(),
                snippet: result.snippet.clone(),
                relevance_score: result.relevance_score,
            };
            citations.push(citation);
        }

        // 按相关性评分排序
        citations.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        citations
    }

    /// 格式化引用信息为文本
    /// 
    /// # Arguments
    /// * `citations` - 引用信息列表
    /// * `max_citations` - 最大引用数量
    /// 
    /// # Returns
    /// 格式化的引用文本
    pub fn format_citations(&self, citations: &[CitationInfo], max_citations: usize) -> String {
        if citations.is_empty() {
            return String::new();
        }

        let mut formatted = String::from("\n\n**参考资料:**\n");
        let citations_to_show = citations.iter().take(max_citations);

        for citation in citations_to_show {
            formatted.push_str(&format!(
                "{}. [{}]({}) - {}\n",
                citation.index,
                citation.title,
                citation.url,
                citation.snippet
            ));
        }

        if citations.len() > max_citations {
            formatted.push_str(&format!("... 还有 {} 个结果", citations.len() - max_citations));
        }

        formatted
    }

    // ==================== 私有辅助方法 ====================

    /// 处理单个 Tavily 搜索结果
    fn process_tavily_result(&self, tavily_result: &TavilySearchResult) -> FlowyResult<WebSearchResultPB> {
        let mut result = WebSearchResultPB::new(
            tavily_result.title.clone(),
            tavily_result.url.clone(),
            tavily_result.content.clone(),
        );

        // 设置相关性评分
        result.relevance_score = tavily_result.score;

        // 提取域名
        if let Ok(url) = Url::parse(&tavily_result.url) {
            if let Some(domain) = url.host_str() {
                result.domain = domain.to_string();
            }
        }

        // 处理发布时间
        if let Some(published_date) = &tavily_result.published_date {
            if let Ok(timestamp) = self.parse_date_string(published_date) {
                result.published_date = Some(timestamp);
            }
        }

        // 设置语言（默认为英文）
        result.language = "en".to_string();

        // 添加元数据
        result.metadata.insert("provider".to_string(), "tavily".to_string());
        result.metadata.insert("raw_score".to_string(), tavily_result.score.to_string());

        Ok(result)
    }

    /// 处理单个 Brave Search 搜索结果
    fn process_brave_result(&self, brave_result: &BraveSearchResult) -> FlowyResult<WebSearchResultPB> {
        let mut result = WebSearchResultPB::new(
            brave_result.title.clone(),
            brave_result.url.clone(),
            brave_result.description.clone(),
        );

        // 计算相关性评分（基于描述长度和额外信息）
        let mut relevance_score: f64 = 0.5; // 基础分数
        
        // 如果有额外摘要，增加相关性
        if let Some(extra_snippets) = &brave_result.extra_snippets {
            if !extra_snippets.is_empty() {
                relevance_score += 0.2;
            }
        }

        // 如果是家庭友好内容，增加相关性
        if brave_result.family_friendly.unwrap_or(false) {
            relevance_score += 0.1;
        }

        result.relevance_score = relevance_score.min(1.0_f64);

        // 提取域名
        if let Ok(url) = Url::parse(&brave_result.url) {
            if let Some(domain) = url.host_str() {
                result.domain = domain.to_string();
            }
        }

        // 处理语言
        if let Some(language) = &brave_result.language {
            result.language = language.clone();
        } else {
            result.language = "en".to_string();
        }

        // 处理页面年龄
        if let Some(page_age) = &brave_result.page_age {
            result.metadata.insert("page_age".to_string(), page_age.clone());
        }

        // 处理位置信息
        if let Some(location) = &brave_result.location {
            result.metadata.insert("location".to_string(), location.clone());
        }

        // 添加额外摘要到内容中
        if let Some(extra_snippets) = &brave_result.extra_snippets {
            if !extra_snippets.is_empty() {
                let additional_content = extra_snippets.join(" ");
                result.content = Some(format!("{}\n\n{}", result.snippet, additional_content));
            }
        }

        // 添加元数据
        result.metadata.insert("provider".to_string(), "brave_search".to_string());
        result.metadata.insert("family_friendly".to_string(), 
            brave_result.family_friendly.unwrap_or(false).to_string());

        Ok(result)
    }

    /// 处理通用搜索结果
    fn process_generic_result(&self, result_value: &serde_json::Value) -> FlowyResult<WebSearchResultPB> {
        // 尝试提取常见字段
        let title = result_value.get("title")
            .or_else(|| result_value.get("name"))
            .or_else(|| result_value.get("headline"))
            .and_then(|v| v.as_str())
            .unwrap_or("无标题")
            .to_string();

        let url = result_value.get("url")
            .or_else(|| result_value.get("link"))
            .or_else(|| result_value.get("href"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let snippet = result_value.get("snippet")
            .or_else(|| result_value.get("description"))
            .or_else(|| result_value.get("summary"))
            .or_else(|| result_value.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut result = WebSearchResultPB::new(title, url, snippet);

        // 尝试提取相关性评分
        if let Some(score) = result_value.get("score")
            .or_else(|| result_value.get("relevance"))
            .or_else(|| result_value.get("rank"))
            .and_then(|v| v.as_f64()) {
            result.relevance_score = score.min(1.0).max(0.0);
        } else {
            result.relevance_score = 0.5; // 默认分数
        }

        // 提取域名
        if let Ok(url_parsed) = Url::parse(&result.url) {
            if let Some(domain) = url_parsed.host_str() {
                result.domain = domain.to_string();
            }
        }

        // 处理发布时间
        if let Some(published_date) = result_value.get("published_date")
            .or_else(|| result_value.get("date"))
            .or_else(|| result_value.get("created_at"))
            .and_then(|v| v.as_str()) {
            if let Ok(timestamp) = self.parse_date_string(published_date) {
                result.published_date = Some(timestamp);
            }
        }

        // 处理语言
        if let Some(language) = result_value.get("language")
            .or_else(|| result_value.get("lang"))
            .and_then(|v| v.as_str()) {
            result.language = language.to_string();
        } else {
            result.language = "en".to_string();
        }

        // 添加所有原始元数据
        if let Some(metadata_obj) = result_value.as_object() {
            for (key, value) in metadata_obj {
                if let Some(str_value) = value.as_str() {
                    result.metadata.insert(key.clone(), str_value.to_string());
                } else if let Some(num_value) = value.as_f64() {
                    result.metadata.insert(key.clone(), num_value.to_string());
                } else if let Some(bool_value) = value.as_bool() {
                    result.metadata.insert(key.clone(), bool_value.to_string());
                }
            }
        }

        Ok(result)
    }

    /// 检查是否应该包含某个结果
    fn should_include_result(&self, result: &WebSearchResultPB) -> bool {
        // 检查基本有效性
        if !result.is_valid() {
            return false;
        }

        // 检查相关性评分阈值
        if result.relevance_score < self.min_relevance_threshold {
            return false;
        }

        // 如果启用内容过滤，检查过滤规则
        if self.enable_content_filter {
            for rule in &self.content_filter_rules {
                if self.matches_filter_rule(result, rule) {
                    return false;
                }
            }
        }

        true
    }

    /// 检查结果是否匹配过滤规则
    fn matches_filter_rule(&self, result: &WebSearchResultPB, rule: &str) -> bool {
        let content_to_check = format!("{} {} {}", result.title, result.snippet, result.url);
        let content_lower = content_to_check.to_lowercase();
        let rule_lower = rule.to_lowercase();

        content_lower.contains(&rule_lower)
    }

    /// 解析日期字符串为时间戳
    fn parse_date_string(&self, date_str: &str) -> FlowyResult<i64> {
        // 尝试多种日期格式
        let formats = [
            "%Y-%m-%d",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%d %H:%M:%S",
            "%d/%m/%Y",
            "%m/%d/%Y",
        ];

        for format in &formats {
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(parsed.and_utc().timestamp());
            }
        }

        // 如果所有格式都失败，尝试 RFC3339
        if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(date_str) {
            return Ok(datetime.timestamp());
        }

        // 最后尝试 Unix 时间戳
        if let Ok(timestamp) = date_str.parse::<i64>() {
            return Ok(timestamp);
        }

        Err(FlowyError::invalid_data()
            .with_context(format!("Unable to parse date string: {}", date_str)))
    }
}

// ==================== 引用信息结构 ====================

/// 引用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationInfo {
    /// 引用索引
    pub index: usize,
    /// 引用标题
    pub title: String,
    /// 引用 URL
    pub url: String,
    /// 引用域名
    pub domain: String,
    /// 引用摘要
    pub snippet: String,
    /// 相关性评分
    pub relevance_score: f64,
}

impl CitationInfo {
    /// 创建新的引用信息
    pub fn new(
        index: usize,
        title: String,
        url: String,
        domain: String,
        snippet: String,
        relevance_score: f64,
    ) -> Self {
        Self {
            index,
            title,
            url,
            domain,
            snippet,
            relevance_score,
        }
    }

    /// 获取格式化的引用文本
    pub fn format(&self) -> String {
        format!(
            "{}. [{}]({}) - {}",
            self.index, self.title, self.url, self.snippet
        )
    }

    /// 获取短格式引用文本
    pub fn format_short(&self) -> String {
        format!("[{}. {}]({})", self.index, self.title, self.url)
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web_search::WebSearchProviderTypePB;

    #[test]
    fn test_processor_creation() {
        let processor = WebSearchResultProcessor::new();
        assert!(processor.enable_content_filter);
        assert_eq!(processor.max_results_limit, 50);
        assert_eq!(processor.min_relevance_threshold, 0.1);
    }

    #[test]
    fn test_processor_with_config() {
        let processor = WebSearchResultProcessor::with_config(
            false,
            vec!["test_rule".to_string()],
            100,
            0.2,
        );
        assert!(!processor.enable_content_filter);
        assert_eq!(processor.max_results_limit, 100);
        assert_eq!(processor.min_relevance_threshold, 0.2);
    }

    #[test]
    fn test_citation_info_creation() {
        let citation = CitationInfo::new(
            1,
            "Test Title".to_string(),
            "https://example.com".to_string(),
            "example.com".to_string(),
            "Test snippet".to_string(),
            0.8,
        );

        assert_eq!(citation.index, 1);
        assert_eq!(citation.title, "Test Title");
        assert_eq!(citation.url, "https://example.com");
        assert_eq!(citation.domain, "example.com");
        assert_eq!(citation.snippet, "Test snippet");
        assert_eq!(citation.relevance_score, 0.8);
    }

    #[test]
    fn test_citation_formatting() {
        let citation = CitationInfo::new(
            1,
            "Test Title".to_string(),
            "https://example.com".to_string(),
            "example.com".to_string(),
            "Test snippet".to_string(),
            0.8,
        );

        let formatted = citation.format();
        assert!(formatted.contains("1. [Test Title](https://example.com) - Test snippet"));

        let short_formatted = citation.format_short();
        assert!(short_formatted.contains("[1. Test Title](https://example.com)"));
    }

    #[test]
    fn test_date_parsing() {
        let processor = WebSearchResultProcessor::new();

        // 测试 RFC3339 格式
        let rfc3339_date = "2023-12-01T10:30:00Z";
        let result = processor.parse_date_string(rfc3339_date);
        assert!(result.is_ok());

        // 测试简单日期格式
        let simple_date = "2023-12-01";
        let result = processor.parse_date_string(simple_date);
        assert!(result.is_ok());

        // 测试 Unix 时间戳
        let timestamp = "1701430200";
        let result = processor.parse_date_string(timestamp);
        assert!(result.is_ok());

        // 测试无效日期
        let invalid_date = "invalid-date";
        let result = processor.parse_date_string(invalid_date);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_rule_matching() {
        let processor = WebSearchResultProcessor::new();

        let result = WebSearchResultPB::new(
            "Adult Content Title".to_string(),
            "https://example.com".to_string(),
            "This contains adult content".to_string(),
        );

        // 测试匹配过滤规则
        assert!(processor.matches_filter_rule(&result, "adult_content"));
        assert!(processor.matches_filter_rule(&result, "adult"));

        // 测试不匹配过滤规则
        assert!(!processor.matches_filter_rule(&result, "malware"));
        assert!(!processor.matches_filter_rule(&result, "phishing"));
    }

    #[test]
    fn test_should_include_result() {
        let processor = WebSearchResultProcessor::new();

        // 测试有效结果
        let mut valid_result = WebSearchResultPB::new(
            "Valid Title".to_string(),
            "https://example.com".to_string(),
            "Valid snippet".to_string(),
        );
        valid_result.relevance_score = 0.8;
        assert!(processor.should_include_result(&valid_result));

        // 测试相关性评分过低的结果
        let mut low_relevance_result = WebSearchResultPB::new(
            "Low Relevance Title".to_string(),
            "https://example.com".to_string(),
            "Low relevance snippet".to_string(),
        );
        low_relevance_result.relevance_score = 0.05; // 低于阈值 0.1
        assert!(!processor.should_include_result(&low_relevance_result));

        // 测试无效结果
        let invalid_result = WebSearchResultPB::new(
            "".to_string(), // 空标题
            "https://example.com".to_string(),
            "Valid snippet".to_string(),
        );
        assert!(!processor.should_include_result(&invalid_result));
    }
}
