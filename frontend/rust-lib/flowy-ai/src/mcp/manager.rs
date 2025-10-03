use crate::mcp::client_pool::MCPClientPool;
use crate::mcp::config::MCPConfigManager;
use crate::mcp::entities::*;
use crate::mcp::tool_discovery::ToolDiscoveryManager;
use flowy_error::FlowyError;
use flowy_sqlite::kv::KVStorePreferences;
use std::sync::Arc;

/// MCP客户端管理器 - 支持多种传输方式和连接池管理
pub struct MCPClientManager {
    client_pool: Arc<MCPClientPool>,
    tool_discovery: Arc<ToolDiscoveryManager>,
    config_manager: Arc<MCPConfigManager>,
}

impl MCPClientManager {
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        let client_pool = Arc::new(MCPClientPool::new());
        let tool_discovery = Arc::new(ToolDiscoveryManager::new(client_pool.clone()));
        let config_manager = Arc::new(MCPConfigManager::new(store_preferences));
        
        Self {
            client_pool,
            tool_discovery,
            config_manager,
        }
    }

    /// 获取配置管理器
    pub fn config_manager(&self) -> &Arc<MCPConfigManager> {
        &self.config_manager
    }

    /// 连接MCP服务器（支持所有传输方式）
    pub async fn connect_server(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
        tracing::info!("Connecting to MCP server: {} ({})", config.name, config.transport_type);
        
        // 如果客户端已存在，先移除
        if self.client_pool.has_client(&config.id) {
            self.remove_server(&config.id).await?;
        }
        
        // 创建新客户端
        self.client_pool.create_client(config.clone()).await?;
        
        // 发现工具并保存到缓存
        match self.tool_discovery.discover_tools(&config.id).await {
            Ok(tools) => {
                tracing::info!("Discovered {} tools for server: {}", tools.len(), config.name);
                // 保存工具缓存到配置
                if let Err(e) = self.config_manager.save_tools_cache(&config.id, tools) {
                    tracing::warn!("Failed to save tools cache for server {}: {}", config.name, e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to discover tools for server {}: {}", config.name, e);
            }
        }
        
        Ok(())
    }

    /// 移除MCP服务器连接
    pub async fn remove_server(&self, server_id: &str) -> Result<(), FlowyError> {
        tracing::info!("Removing MCP server: {}", server_id);
        
        // 清理工具注册表
        self.tool_discovery.clear_server_tools(server_id).await;
        
        // 移除客户端
        self.client_pool.remove_client(server_id).await?;
        
        Ok(())
    }

    /// 获取服务器工具列表
    pub async fn tool_list(&self, server_id: &str) -> Result<ToolsList, FlowyError> {
        let tools = self.tool_discovery.get_tools(server_id).await
            .unwrap_or_default();
        
        tracing::trace!("{}: tool list: {:?}", server_id, tools);
        
        Ok(ToolsList { tools })
    }

    /// 调用MCP工具
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolCallResponse, FlowyError> {
        // 验证工具可用性
        let is_valid = self.tool_discovery.validate_tool(server_id, tool_name).await;
        
        // 如果工具不可用，检查是否需要重新发现工具
        if let Err(e) = &is_valid {
            tracing::warn!("Tool validation failed: {}", e);
            
            // 检查服务器是否已连接但工具注册表为空
            if self.is_server_connected(server_id) {
                let tools = self.tool_discovery.get_tools(server_id).await;
                if tools.is_none() || tools.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                    tracing::info!("🔄 Server '{}' is connected but has no tools in registry, attempting to rediscover tools...", server_id);
                    
                    match self.tool_discovery.discover_tools(server_id).await {
                        Ok(discovered_tools) => {
                            tracing::info!("✅ Rediscovered {} tools for server '{}'", discovered_tools.len(), server_id);
                            
                            // 保存工具缓存
                            if let Err(cache_err) = self.config_manager.save_tools_cache(server_id, discovered_tools) {
                                tracing::warn!("Failed to save tools cache: {}", cache_err);
                            }
                            
                            // 重新验证工具
                            if !self.tool_discovery.validate_tool(server_id, tool_name).await? {
                                return Err(FlowyError::invalid_data()
                                    .with_context(format!("Tool '{}' not found even after rediscovery on server '{}'", tool_name, server_id)));
                            }
                        }
                        Err(rediscover_err) => {
                            tracing::error!("Failed to rediscover tools: {}", rediscover_err);
                            return Err(FlowyError::invalid_data()
                                .with_context(format!("Tool '{}' not available and rediscovery failed: {}", tool_name, rediscover_err)));
                        }
                    }
                } else {
                    return Err(FlowyError::invalid_data()
                        .with_context(format!("Tool '{}' not available on server '{}'", tool_name, server_id)));
                }
            } else {
                return Err(FlowyError::invalid_data()
                    .with_context(format!("Server '{}' is not connected", server_id)));
            }
        } else if !is_valid.unwrap() {
            return Err(FlowyError::invalid_data()
                .with_context(format!("Tool '{}' not available on server '{}'", tool_name, server_id)));
        }
        
        // 获取客户端并调用工具
        let client = self.client_pool.get_client(server_id)
            .ok_or_else(|| FlowyError::record_not_found()
                .with_context(format!("Client not found: {}", server_id)))?;
        
        let client_guard = client.read().await;
        
        let request = ToolCallRequest {
            name: tool_name.to_string(),
            arguments,
        };
        
        let response = client_guard.call_tool(request).await?;
        
        tracing::info!("Tool call completed: {} on {}", tool_name, server_id);
        Ok(response)
    }

    /// 获取所有连接的服务器信息
    pub async fn list_servers(&self) -> Vec<MCPClientInfo> {
        self.client_pool.get_all_clients_info().await
    }

    /// 搜索工具
    pub async fn search_tools(&self, query: &str) -> Vec<(String, MCPTool)> {
        self.tool_discovery.search_tools(query).await
    }

    /// 根据名称查找工具
    pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
        // 🔍 优先从工具注册表中查找(已连接的服务器)
        if let Some(result) = self.tool_discovery.find_tool_by_name(tool_name).await {
            tracing::info!("🔍 [FIND TOOL] Found '{}' in connected server '{}'", tool_name, result.0);
            return Some(result);
        }
        
        // 🔍 如果注册表中没有,从配置的缓存中查找
        tracing::info!("🔍 [FIND TOOL] Tool '{}' not in registry, searching cached tools...", tool_name);
        
        let all_servers = self.config_manager.get_all_servers();
        for server in all_servers {
            if let Some(cached_tools) = &server.cached_tools {
                for tool in cached_tools {
                    if tool.name == tool_name {
                        tracing::info!("🔍 [FIND TOOL] Found '{}' in cached tools of server '{}'", 
                                     tool_name, server.id);
                        return Some((server.id.clone(), tool.clone()));
                    }
                }
            }
        }
        
        tracing::warn!("🔍 [FIND TOOL] Tool '{}' not found in any server (registry or cache)", tool_name);
        None
    }

    /// 获取工具统计信息
    pub async fn get_tool_statistics(&self) -> crate::mcp::tool_discovery::ToolStatistics {
        self.tool_discovery.get_tool_statistics().await
    }

    /// 重连服务器
    pub async fn reconnect_server(&self, server_id: &str) -> Result<(), FlowyError> {
        tracing::info!("Reconnecting MCP server: {}", server_id);
        
        self.client_pool.reconnect_client(server_id).await?;
        
        // 重新发现工具
        match self.tool_discovery.discover_tools(server_id).await {
            Ok(tools) => {
                tracing::info!("Rediscovered {} tools for server: {}", tools.len(), server_id);
            }
            Err(e) => {
                tracing::warn!("Failed to rediscover tools for server {}: {}", server_id, e);
            }
        }
        
        Ok(())
    }

    /// 健康检查
    pub async fn health_check(&self) -> Vec<(String, MCPConnectionStatus)> {
        self.client_pool.health_check().await
    }

    /// 刷新所有工具
    pub async fn refresh_all_tools(&self) -> Result<(), FlowyError> {
        self.tool_discovery.refresh_registry().await
    }

    /// 停止所有连接
    pub async fn stop_all(&self) -> Result<(), FlowyError> {
        tracing::info!("Stopping all MCP connections");
        
        self.tool_discovery.clear_all_tools().await;
        self.client_pool.stop_all_clients().await?;
        
        Ok(())
    }

    /// 获取客户端数量
    pub fn client_count(&self) -> usize {
        self.client_pool.client_count()
    }

    /// 检查服务器是否已连接
    pub fn is_server_connected(&self, server_id: &str) -> bool {
        self.client_pool.has_client(server_id)
    }

    /// 从配置连接服务器
    pub async fn connect_server_from_config(&self, server_id: &str) -> Result<(), FlowyError> {
        let config = self.config_manager.get_server(server_id)
            .ok_or_else(|| FlowyError::record_not_found()
                .with_context(format!("Server config not found: {}", server_id)))?;
        
        if !config.is_active {
            return Err(FlowyError::invalid_data()
                .with_context("Server is not active"));
        }
        
        self.connect_server(config).await
    }

    /// 连接所有激活的服务器
    pub async fn connect_all_active_servers(&self) -> Result<Vec<String>, FlowyError> {
        let active_servers = self.config_manager.get_active_servers();
        let mut connected_servers = Vec::new();
        let mut errors = Vec::new();
        
        for server in active_servers {
            match self.connect_server(server.clone()).await {
                Ok(_) => {
                    connected_servers.push(server.id.clone());
                    tracing::info!("Successfully connected to server: {}", server.name);
                }
                Err(e) => {
                    errors.push(format!("Failed to connect {}: {}", server.name, e));
                    tracing::error!("Failed to connect to server {}: {}", server.name, e);
                }
            }
        }
        
        if !errors.is_empty() {
            tracing::warn!("Some servers failed to connect: {:?}", errors);
        }
        
        Ok(connected_servers)
    }

    /// 保存服务器配置并连接
    pub async fn save_and_connect_server(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
        // 保存配置
        self.config_manager.save_server(config.clone())?;
        
        // 如果服务器是激活的，尝试连接
        if config.is_active {
            self.connect_server(config).await?;
        }
        
        Ok(())
    }

    /// 删除服务器配置并断开连接
    pub async fn delete_server_config(&self, server_id: &str) -> Result<(), FlowyError> {
        // 先断开连接
        if self.is_server_connected(server_id) {
            self.remove_server(server_id).await?;
        }
        
        // 删除配置
        self.config_manager.delete_server(server_id)?;
        
        Ok(())
    }

    /// 测试服务器配置（不保存连接）
    pub async fn test_server_config(&self, config: MCPServerConfig) -> Result<Vec<MCPTool>, FlowyError> {
        tracing::info!("Testing MCP server config: {} ({})", config.name, config.transport_type);
        
        // 创建临时客户端进行测试
        let temp_pool = MCPClientPool::new();
        temp_pool.create_client(config.clone()).await?;
        
        // 获取工具列表
        let client = temp_pool.get_client(&config.id)
            .ok_or_else(|| FlowyError::internal().with_context("Failed to create test client"))?;
        
        let client_guard = client.read().await;
        let tools_list = client_guard.list_tools().await?;
        
        // 清理临时客户端
        drop(client_guard);
        temp_pool.remove_client(&config.id).await?;
        
        tracing::info!("Test successful: found {} tools", tools_list.tools.len());
        Ok(tools_list.tools)
    }
}

// Note: Default implementation removed because MCPClientManager now requires KVStorePreferences
