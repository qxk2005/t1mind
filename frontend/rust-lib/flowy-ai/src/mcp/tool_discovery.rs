use crate::mcp::client_pool::MCPClientPool;
use crate::mcp::entities::*;
use flowy_error::FlowyError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工具发现和注册管理器
pub struct ToolDiscoveryManager {
    client_pool: Arc<MCPClientPool>,
    tool_registry: Arc<RwLock<HashMap<String, Vec<MCPTool>>>>,
}

impl ToolDiscoveryManager {
    pub fn new(client_pool: Arc<MCPClientPool>) -> Self {
        Self {
            client_pool,
            tool_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发现指定服务器的工具
    pub async fn discover_tools(&self, server_id: &str) -> Result<Vec<MCPTool>, FlowyError> {
        let client = self.client_pool.get_client(server_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context(format!("Client not found: {}", server_id)))?;
        
        let client_guard = client.read().await;
        
        if !client_guard.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        let tools_list = client_guard.list_tools().await?;
        let tools = tools_list.tools;
        
        // 更新工具注册表
        {
            let mut registry = self.tool_registry.write().await;
            registry.insert(server_id.to_string(), tools.clone());
        }
        
        tracing::info!("Discovered {} tools for server: {}", tools.len(), server_id);
        Ok(tools)
    }

    /// 发现所有服务器的工具
    pub async fn discover_all_tools(&self) -> Result<HashMap<String, Vec<MCPTool>>, FlowyError> {
        let clients_info = self.client_pool.get_all_clients_info().await;
        let mut all_tools = HashMap::new();
        
        for client_info in clients_info {
            if client_info.status == MCPConnectionStatus::Connected {
                match self.discover_tools(&client_info.server_id).await {
                    Ok(tools) => {
                        all_tools.insert(client_info.server_id.clone(), tools);
                    }
                    Err(e) => {
                        tracing::error!("Failed to discover tools for {}: {}", client_info.server_id, e);
                    }
                }
            }
        }
        
        Ok(all_tools)
    }

    /// 获取指定服务器的工具
    pub async fn get_tools(&self, server_id: &str) -> Option<Vec<MCPTool>> {
        let registry = self.tool_registry.read().await;
        registry.get(server_id).cloned()
    }

    /// 获取所有工具
    pub async fn get_all_tools(&self) -> HashMap<String, Vec<MCPTool>> {
        let registry = self.tool_registry.read().await;
        registry.clone()
    }

    /// 搜索工具
    pub async fn search_tools(&self, query: &str) -> Vec<(String, MCPTool)> {
        let registry = self.tool_registry.read().await;
        let mut results = Vec::new();
        
        let query_lower = query.to_lowercase();
        
        for (server_id, tools) in registry.iter() {
            for tool in tools {
                if tool.name.to_lowercase().contains(&query_lower) ||
                   tool.description.as_deref().unwrap_or("").to_lowercase().contains(&query_lower) {
                    results.push((server_id.clone(), tool.clone()));
                }
            }
        }
        
        results
    }

    /// 根据名称查找工具
    pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
        let registry = self.tool_registry.read().await;
        
        for (server_id, tools) in registry.iter() {
            for tool in tools {
                if tool.name == tool_name {
                    return Some((server_id.clone(), tool.clone()));
                }
            }
        }
        
        None
    }

    /// 获取工具统计信息
    pub async fn get_tool_statistics(&self) -> ToolStatistics {
        let registry = self.tool_registry.read().await;
        let mut total_tools = 0;
        let mut servers_with_tools = 0;
        let mut tool_categories = HashMap::new();
        let mut safety_levels = HashMap::new();
        
        for (_server_id, tools) in registry.iter() {
            if !tools.is_empty() {
                servers_with_tools += 1;
                total_tools += tools.len();
                
                // 工具分类（基于工具名称前缀）
                for tool in tools {
                    let category = self.categorize_tool(&tool.name);
                    *tool_categories.entry(category).or_insert(0) += 1;
                    
                    // 安全级别统计
                    let safety_level = tool.safety_level().to_string();
                    *safety_levels.entry(safety_level).or_insert(0) += 1;
                }
            }
        }
        
        ToolStatistics {
            total_servers: registry.len(),
            servers_with_tools,
            total_tools,
            tool_categories,
            safety_levels,
        }
    }

    /// 根据安全级别过滤工具
    pub async fn get_tools_by_safety_level(&self, safety_level: crate::mcp::entities::ToolSafetyLevel) -> Vec<(String, MCPTool)> {
        let registry = self.tool_registry.read().await;
        let mut filtered_tools = Vec::new();
        
        for (server_id, tools) in registry.iter() {
            for tool in tools {
                if tool.safety_level() == safety_level {
                    filtered_tools.push((server_id.clone(), tool.clone()));
                }
            }
        }
        
        filtered_tools
    }

    /// 获取只读工具列表
    pub async fn get_read_only_tools(&self) -> Vec<(String, MCPTool)> {
        let registry = self.tool_registry.read().await;
        let mut read_only_tools = Vec::new();
        
        for (server_id, tools) in registry.iter() {
            for tool in tools {
                if tool.is_read_only() {
                    read_only_tools.push((server_id.clone(), tool.clone()));
                }
            }
        }
        
        read_only_tools
    }

    /// 获取破坏性工具列表（需要特殊权限）
    pub async fn get_destructive_tools(&self) -> Vec<(String, MCPTool)> {
        let registry = self.tool_registry.read().await;
        let mut destructive_tools = Vec::new();
        
        for (server_id, tools) in registry.iter() {
            for tool in tools {
                if tool.is_destructive() {
                    destructive_tools.push((server_id.clone(), tool.clone()));
                }
            }
        }
        
        destructive_tools
    }

    /// 获取外部交互工具列表
    pub async fn get_external_interaction_tools(&self) -> Vec<(String, MCPTool)> {
        let registry = self.tool_registry.read().await;
        let mut external_tools = Vec::new();
        
        for (server_id, tools) in registry.iter() {
            for tool in tools {
                if tool.interacts_with_external_world() {
                    external_tools.push((server_id.clone(), tool.clone()));
                }
            }
        }
        
        external_tools
    }

    /// 验证工具可用性
    pub async fn validate_tool(&self, server_id: &str, tool_name: &str) -> Result<bool, FlowyError> {
        let tools = self.get_tools(server_id).await
            .ok_or_else(|| FlowyError::record_not_found().with_context(format!("No tools found for server: {}", server_id)))?;
        
        let tool_exists = tools.iter().any(|tool| tool.name == tool_name);
        
        if !tool_exists {
            return Ok(false);
        }
        
        // 检查客户端连接状态
        let client = self.client_pool.get_client(server_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context(format!("Client not found: {}", server_id)))?;
        
        let client_guard = client.read().await;
        Ok(client_guard.is_connected())
    }

    /// 刷新工具注册表
    pub async fn refresh_registry(&self) -> Result<(), FlowyError> {
        tracing::info!("Refreshing tool registry...");
        
        let _ = self.discover_all_tools().await?;
        
        tracing::info!("Tool registry refreshed successfully");
        Ok(())
    }

    /// 清理指定服务器的工具
    pub async fn clear_server_tools(&self, server_id: &str) {
        let mut registry = self.tool_registry.write().await;
        registry.remove(server_id);
        tracing::info!("Cleared tools for server: {}", server_id);
    }

    /// 清理所有工具
    pub async fn clear_all_tools(&self) {
        let mut registry = self.tool_registry.write().await;
        registry.clear();
        tracing::info!("Cleared all tools from registry");
    }

    /// 工具分类辅助函数
    fn categorize_tool(&self, tool_name: &str) -> String {
        let name_lower = tool_name.to_lowercase();
        
        if name_lower.contains("file") || name_lower.contains("read") || name_lower.contains("write") {
            "file_operations".to_string()
        } else if name_lower.contains("search") || name_lower.contains("find") {
            "search".to_string()
        } else if name_lower.contains("web") || name_lower.contains("http") || name_lower.contains("api") {
            "web_services".to_string()
        } else if name_lower.contains("data") || name_lower.contains("database") || name_lower.contains("sql") {
            "data_management".to_string()
        } else if name_lower.contains("text") || name_lower.contains("format") || name_lower.contains("parse") {
            "text_processing".to_string()
        } else {
            "general".to_string()
        }
    }
}

/// 工具统计信息
#[derive(Debug, Clone)]
pub struct ToolStatistics {
    pub total_servers: usize,
    pub servers_with_tools: usize,
    pub total_tools: usize,
    pub tool_categories: HashMap<String, usize>,
    pub safety_levels: HashMap<String, usize>,
}
