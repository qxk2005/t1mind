use std::collections::HashMap;
use std::sync::Arc;

use flowy_error::{FlowyError, FlowyResult};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::entities::{ToolDefinitionPB, ToolTypePB};
use crate::web_search::hub::{WebSearchHub, WebSearchHubStatus};
use crate::web_search::entities::{WebSearchRequestPB, WebSearchGlobalConfigPB};
use flowy_sqlite::kv::KVStorePreferences;

/// 网络搜索工具管理器
/// 负责将WebSearchHub的功能封装为Agent可用的工具
pub struct WebSearchToolManager {
    web_search_hub: Arc<WebSearchHub>,
    store_preferences: Arc<KVStorePreferences>,
}

impl WebSearchToolManager {
    /// 创建新的网络搜索工具管理器
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        let hub = Arc::new(WebSearchHub::new(store_preferences.clone()));
        Self {
            web_search_hub: hub,
            store_preferences,
        }
    }

    /// 创建带有自定义配置的网络搜索工具管理器
    pub fn with_config(
        store_preferences: Arc<KVStorePreferences>,
        global_config: WebSearchGlobalConfigPB,
    ) -> Self {
        let hub = Arc::new(WebSearchHub::with_config(
            store_preferences.clone(),
            global_config,
            Default::default(), // 使用默认的CacheConfig
        ));
        Self {
            web_search_hub: hub,
            store_preferences,
        }
    }

    /// 使用现有的网络搜索中心实例创建工具管理器
    pub fn with_hub(
        web_search_hub: Arc<WebSearchHub>,
        store_preferences: Arc<KVStorePreferences>,
    ) -> Self {
        Self {
            web_search_hub,
            store_preferences,
        }
    }

    /// 获取所有可用的网络搜索工具定义
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinitionPB> {
        let mut tools = Vec::new();
        
        // 主网络搜索工具
        tools.push(ToolDefinitionPB {
            name: "web_search".to_string(),
            description: "使用网络搜索引擎进行通用搜索。".to_string(),
            tool_type: ToolTypePB::Search,
            source: "web_search".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "搜索查询"},
                    "max_results": {"type": "integer", "description": "最大结果数", "default": 10},
                    "language": {"type": "string", "description": "搜索语言，例如 'en' 或 'zh-CN'", "default": "en"},
                    "region": {"type": "string", "description": "搜索区域，例如 'US' 或 'CN'", "default": "US"},
                    "include_content": {"type": "boolean", "description": "是否包含搜索结果的详细内容", "default": true},
                },
                "required": ["query"]
            }).to_string(),
            permissions: vec!["search.web".to_string()],
            is_available: self.web_search_hub.get_status().enabled,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "search".to_string());
                meta.insert("safe_mode".to_string(), "true".to_string());
                meta
            },
        });

        // 快速搜索工具 (示例)
        tools.push(ToolDefinitionPB {
            name: "quick_search".to_string(),
            description: "执行快速网络搜索以获取简短、直接的答案。".to_string(),
            tool_type: ToolTypePB::Search,
            source: "web_search".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "快速搜索查询"},
                },
                "required": ["query"]
            }).to_string(),
            permissions: vec!["search.web".to_string()],
            is_available: self.web_search_hub.get_status().enabled,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "search".to_string());
                meta.insert("safe_mode".to_string(), "true".to_string());
                meta.insert("speed".to_string(), "fast".to_string());
                meta
            },
        });

        // 新闻搜索工具 (示例)
        tools.push(ToolDefinitionPB {
            name: "news_search".to_string(),
            description: "搜索最新新闻文章。".to_string(),
            tool_type: ToolTypePB::Search,
            source: "web_search".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "新闻搜索查询"},
                    "language": {"type": "string", "description": "新闻语言", "default": "en"},
                },
                "required": ["query"]
            }).to_string(),
            permissions: vec!["search.web".to_string()],
            is_available: self.web_search_hub.get_status().enabled,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "news".to_string());
                meta.insert("safe_mode".to_string(), "true".to_string());
                meta
            },
        });

        // 学术搜索工具 (示例)
        tools.push(ToolDefinitionPB {
            name: "academic_search".to_string(),
            description: "搜索学术论文和研究。".to_string(),
            tool_type: ToolTypePB::Search,
            source: "web_search".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "学术搜索查询"},
                    "year_start": {"type": "integer", "description": "起始年份"},
                    "year_end": {"type": "integer", "description": "结束年份"},
                },
                "required": ["query"]
            }).to_string(),
            permissions: vec!["search.web".to_string()],
            is_available: self.web_search_hub.get_status().enabled,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "academic".to_string());
                meta.insert("safe_mode".to_string(), "true".to_string());
                meta
            },
        });

        tools
    }

    /// 执行网络搜索工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: &Value,
        safe_mode: bool,
    ) -> FlowyResult<String> {
        info!("执行网络搜索工具: {} (安全模式: {})", tool_name, safe_mode);
        debug!("工具参数: {}", arguments);

        if !self.web_search_hub.get_status().enabled {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::InvalidRequest,
                "网络搜索功能未启用".to_string(),
            ));
        }

        // 根据 tool_name 映射到不同的搜索逻辑或参数调整
        let mut request_pb = WebSearchRequestPB {
            query: arguments["query"].as_str().unwrap_or_default().to_string(),
            provider_id: None, // 默认由Hub选择活跃供应商
            max_results: arguments["max_results"].as_i64().unwrap_or(10) as i32,
            language: arguments["language"].as_str().unwrap_or("en").to_string(),
            region: arguments["region"].as_str().unwrap_or("US").to_string(),
            include_content: arguments["include_content"].as_bool().unwrap_or(true),
            metadata: HashMap::new(),
        };

        match tool_name {
            "web_search" => {
                // 使用默认参数
            },
            "quick_search" => {
                request_pb.max_results = 3; // 快速搜索只返回少量结果
                request_pb.include_content = false; // 快速搜索不包含详细内容
            },
            "news_search" => {
                request_pb.max_results = arguments["max_results"].as_i64().unwrap_or(5) as i32;
                // 可以添加特定于新闻搜索的元数据或参数
            },
            "academic_search" => {
                request_pb.max_results = arguments["max_results"].as_i64().unwrap_or(5) as i32;
                // 可以添加年份过滤等学术搜索特有参数
            },
            _ => {
                return Err(FlowyError::invalid_data()
                    .with_context(format!("未知的网络搜索工具: {}", tool_name)));
            }
        }

        let response = self.web_search_hub.search(request_pb).await?;

        // 格式化结果
        let mut result_str = String::new();
        result_str.push_str(&format!("搜索结果 ({}): \n", response.query));
        for (i, res) in response.results.iter().enumerate() {
            result_str.push_str(&format!("{}. {}\n", i + 1, res.title));
            if !res.url.is_empty() {
                result_str.push_str(&format!("   链接: {}\n", res.url));
            }
            if let Some(content) = &res.content {
                result_str.push_str(&format!("   内容: {}\n", content));
            }
        }
        Ok(result_str)
    }

    /// 获取网络搜索工具的状态
    pub fn get_tool_status(&self) -> WebSearchToolStatus {
        let hub_status = self.web_search_hub.get_status();
        WebSearchToolStatus {
            enabled: hub_status.enabled,
            active_providers: hub_status.active_providers,
            total_providers: hub_status.total_providers,
            cache_enabled: hub_status.cache_enabled,
            content_filter_enabled: hub_status.content_filter_enabled,
        }
    }
}

/// 网络搜索工具状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSearchToolStatus {
    pub enabled: bool,
    pub active_providers: usize,
    pub total_providers: usize,
    pub cache_enabled: bool,
    pub content_filter_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowy_sqlite::kv::KVStorePreferences;
    use crate::entities::ToolTypePB;
    use crate::web_search::entities::WebSearchGlobalConfigPB;

    #[tokio::test]
    async fn test_web_search_tool_definitions() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_web_search_db").unwrap());
        let web_search_manager = Arc::new(WebSearchToolManager::new(store_preferences.clone()));
        
        let tool_definitions = web_search_manager.get_tool_definitions();
        assert!(!tool_definitions.is_empty());
        
        assert!(tool_definitions.iter().any(|t| t.name == "web_search"));
        assert!(tool_definitions.iter().any(|t| t.name == "quick_search"));
        assert!(tool_definitions.iter().any(|t| t.name == "news_search"));
        assert!(tool_definitions.iter().any(|t| t.name == "academic_search"));
        
        for tool in &tool_definitions {
            assert_eq!(tool.tool_type, ToolTypePB::Search);
            assert_eq!(tool.source, "web_search");
            // is_available depends on hub status, which is false by default
            // assert!(tool.is_available); 
        }
    }

    #[tokio::test]
    async fn test_web_search_tool_execution_disabled_hub() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_disabled_hub_db").unwrap());
        let web_search_manager = Arc::new(WebSearchToolManager::new(store_preferences.clone()));
        
        let args = json!({
            "query": "test search query"
        });
        
        let result = web_search_manager.execute_tool("web_search", &args, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("网络搜索功能未启用"));
    }

    #[tokio::test]
    async fn test_web_search_tool_execution_invalid_tool() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_invalid_tool_db").unwrap());
        let mut global_config = WebSearchGlobalConfigPB::default_config();
        global_config.enabled = true;
        let web_search_manager = Arc::new(WebSearchToolManager::with_config(store_preferences.clone(), global_config));

        let args = json!({
            "query": "test query"
        });

        let result = web_search_manager.execute_tool("unknown_tool", &args, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("未知的网络搜索工具"));
    }

    #[tokio::test]
    async fn test_web_search_tool_execution_missing_query() {
        let store_preferences = Arc::new(KVStorePreferences::new("test_missing_query_db").unwrap());
        let mut global_config = WebSearchGlobalConfigPB::default_config();
        global_config.enabled = true;
        let web_search_manager = Arc::new(WebSearchToolManager::with_config(store_preferences.clone(), global_config));

        let args = json!({}); // Missing query

        let result = web_search_manager.execute_tool("web_search", &args, true).await;
        assert!(result.is_ok()); // Query defaults to empty string, hub will handle it
    }
}