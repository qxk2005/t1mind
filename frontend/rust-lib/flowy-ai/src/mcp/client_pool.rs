use crate::mcp::client::*;
use crate::mcp::entities::*;
use flowy_error::FlowyError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// MCP客户端池 - 管理多个MCP服务器连接
/// 支持STDIO、SSE、HTTP三种传输方式的客户端创建和管理
/// 包含生命周期管理，确保线程安全，避免连接泄漏
pub struct MCPClientPool {
    /// 客户端存储 - 使用RwLock确保线程安全
    clients: Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn MCPClient>>>>>>,
    /// 客户端元数据存储
    client_metadata: Arc<RwLock<HashMap<String, ClientMetadata>>>,
}

/// 客户端元数据
#[derive(Debug, Clone)]
struct ClientMetadata {
    server_id: String,
    config: MCPServerConfig,
    created_at: SystemTime,
    last_connected: Option<SystemTime>,
    connection_attempts: u32,
    error_message: Option<String>,
}

impl MCPClientPool {
    /// 创建新的客户端池
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            client_metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建并添加客户端到池中
    pub async fn create_client(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
        let server_id = config.id.clone();
        
        tracing::info!("Creating MCP client for server: {} ({})", config.name, config.transport_type);
        
        // 创建对应类型的客户端
        let mut client: Box<dyn MCPClient> = match config.transport_type {
            MCPTransportType::Stdio => {
                Box::new(StdioMCPClient::new(config.clone())?)
            }
            MCPTransportType::SSE => {
                Box::new(SSEMCPClient::new(config.clone())?)
            }
            MCPTransportType::HTTP => {
                Box::new(HttpMCPClient::new(config.clone())?)
            }
        };

        // 初始化客户端连接
        let init_result = client.initialize().await;
        
        // 创建客户端元数据
        let metadata = ClientMetadata {
            server_id: server_id.clone(),
            config: config.clone(),
            created_at: SystemTime::now(),
            last_connected: if init_result.is_ok() { Some(SystemTime::now()) } else { None },
            connection_attempts: 1,
            error_message: init_result.as_ref().err().map(|e| e.to_string()),
        };

        // 存储客户端和元数据
        {
            let mut clients = self.clients.write().await;
            let mut client_metadata = self.client_metadata.write().await;
            
            clients.insert(server_id.clone(), Arc::new(RwLock::new(client)));
            client_metadata.insert(server_id.clone(), metadata);
        }

        // 如果初始化失败，记录错误但不移除客户端（允许后续重连）
        if let Err(e) = init_result {
            tracing::warn!("Failed to initialize client for {}: {}", config.name, e);
            return Err(e);
        }

        tracing::info!("Successfully created and initialized MCP client: {}", config.name);
        Ok(())
    }

    /// 获取客户端
    pub fn get_client(&self, server_id: &str) -> Option<Arc<RwLock<Box<dyn MCPClient>>>> {
        // 注意：这里使用try_read()来避免阻塞
        // 如果无法获取锁，返回None
        match self.clients.try_read() {
            Ok(clients) => clients.get(server_id).cloned(),
            Err(_) => {
                tracing::warn!("Failed to acquire read lock for client: {}", server_id);
                None
            }
        }
    }

    /// 检查客户端是否存在
    pub fn has_client(&self, server_id: &str) -> bool {
        match self.clients.try_read() {
            Ok(clients) => clients.contains_key(server_id),
            Err(_) => false,
        }
    }

    /// 移除客户端
    pub async fn remove_client(&self, server_id: &str) -> Result<(), FlowyError> {
        tracing::info!("Removing MCP client: {}", server_id);
        
        let client = {
            let mut clients = self.clients.write().await;
            clients.remove(server_id)
        };

        if let Some(client) = client {
            // 优雅地停止客户端
            let mut client_guard = client.write().await;
            if let Err(e) = client_guard.stop().await {
                tracing::warn!("Error stopping client {}: {}", server_id, e);
            }
        }

        // 移除元数据
        {
            let mut client_metadata = self.client_metadata.write().await;
            client_metadata.remove(server_id);
        }

        tracing::info!("Successfully removed MCP client: {}", server_id);
        Ok(())
    }

    /// 重连客户端
    pub async fn reconnect_client(&self, server_id: &str) -> Result<(), FlowyError> {
        tracing::info!("Reconnecting MCP client: {}", server_id);
        
        // 获取客户端配置
        let config = {
            let client_metadata = self.client_metadata.read().await;
            client_metadata.get(server_id)
                .map(|metadata| metadata.config.clone())
                .ok_or_else(|| FlowyError::record_not_found()
                    .with_context(format!("Client metadata not found: {}", server_id)))?
        };

        // 移除旧客户端
        self.remove_client(server_id).await?;
        
        // 创建新客户端
        self.create_client(config).await?;
        
        // 更新重连统计
        {
            let mut client_metadata = self.client_metadata.write().await;
            if let Some(metadata) = client_metadata.get_mut(server_id) {
                metadata.connection_attempts += 1;
                metadata.last_connected = Some(SystemTime::now());
                metadata.error_message = None;
            }
        }

        tracing::info!("Successfully reconnected MCP client: {}", server_id);
        Ok(())
    }

    /// 获取所有客户端信息
    pub async fn get_all_clients_info(&self) -> Vec<MCPClientInfo> {
        let clients = self.clients.read().await;
        let client_metadata = self.client_metadata.read().await;
        
        let mut infos = Vec::new();
        
        for (server_id, client) in clients.iter() {
            let client_guard = client.read().await;
            let metadata = client_metadata.get(server_id);
            
            let info = MCPClientInfo {
                server_id: server_id.clone(),
                status: client_guard.get_status(),
                tools: match client_guard.list_tools().await {
                    Ok(tools_list) => tools_list.tools,
                    Err(_) => Vec::new(),
                },
                last_connected: metadata.and_then(|m| m.last_connected),
                error_message: metadata.and_then(|m| m.error_message.clone()),
            };
            
            infos.push(info);
        }
        
        infos
    }

    /// 健康检查 - 检查所有客户端的连接状态
    pub async fn health_check(&self) -> Vec<(String, MCPConnectionStatus)> {
        let clients = self.clients.read().await;
        let mut results = Vec::new();
        
        for (server_id, client) in clients.iter() {
            let client_guard = client.read().await;
            let status = client_guard.get_status();
            results.push((server_id.clone(), status));
        }
        
        results
    }

    /// 停止所有客户端
    pub async fn stop_all_clients(&self) -> Result<(), FlowyError> {
        tracing::info!("Stopping all MCP clients");
        
        let clients = {
            let mut clients = self.clients.write().await;
            let all_clients = clients.drain().collect::<Vec<_>>();
            all_clients
        };

        // 并发停止所有客户端
        let stop_tasks: Vec<_> = clients.into_iter().map(|(server_id, client)| {
            tokio::spawn(async move {
                let mut client_guard = client.write().await;
                if let Err(e) = client_guard.stop().await {
                    tracing::warn!("Error stopping client {}: {}", server_id, e);
                }
                server_id
            })
        }).collect();

        // 等待所有停止任务完成
        for task in stop_tasks {
            if let Ok(server_id) = task.await {
                tracing::debug!("Stopped client: {}", server_id);
            }
        }

        // 清理元数据
        {
            let mut client_metadata = self.client_metadata.write().await;
            client_metadata.clear();
        }

        tracing::info!("All MCP clients stopped successfully");
        Ok(())
    }

    /// 获取客户端数量
    pub fn client_count(&self) -> usize {
        match self.clients.try_read() {
            Ok(clients) => clients.len(),
            Err(_) => 0,
        }
    }

    /// 清理断开连接的客户端
    pub async fn cleanup_disconnected_clients(&self) -> Result<(), FlowyError> {
        tracing::info!("Cleaning up disconnected clients");
        
        let disconnected_clients = {
            let clients = self.clients.read().await;
            let mut disconnected = Vec::new();
            
            for (server_id, client) in clients.iter() {
                let client_guard = client.read().await;
                if matches!(client_guard.get_status(), MCPConnectionStatus::Disconnected | MCPConnectionStatus::Error(_)) {
                    disconnected.push(server_id.clone());
                }
            }
            
            disconnected
        };

        // 移除断开连接的客户端
        for server_id in disconnected_clients {
            tracing::info!("Cleaning up disconnected client: {}", server_id);
            self.remove_client(&server_id).await?;
        }

        Ok(())
    }

    /// 获取客户端统计信息
    pub async fn get_statistics(&self) -> ClientPoolStatistics {
        let clients = self.clients.read().await;
        let client_metadata = self.client_metadata.read().await;
        
        let mut stats = ClientPoolStatistics {
            total_clients: clients.len(),
            connected_clients: 0,
            disconnected_clients: 0,
            error_clients: 0,
            total_connection_attempts: 0,
        };

        for (server_id, client) in clients.iter() {
            let client_guard = client.read().await;
            
            match client_guard.get_status() {
                MCPConnectionStatus::Connected => stats.connected_clients += 1,
                MCPConnectionStatus::Disconnected => stats.disconnected_clients += 1,
                MCPConnectionStatus::Error(_) => stats.error_clients += 1,
                MCPConnectionStatus::Connecting => {}, // 不计入统计
            }
            
            if let Some(metadata) = client_metadata.get(server_id) {
                stats.total_connection_attempts += metadata.connection_attempts as usize;
            }
        }

        stats
    }
}

/// 客户端池统计信息
#[derive(Debug, Clone)]
pub struct ClientPoolStatistics {
    pub total_clients: usize,
    pub connected_clients: usize,
    pub disconnected_clients: usize,
    pub error_clients: usize,
    pub total_connection_attempts: usize,
}

impl Default for MCPClientPool {
    fn default() -> Self {
        Self::new()
    }
}

// 确保MCPClientPool是线程安全的
unsafe impl Send for MCPClientPool {}
unsafe impl Sync for MCPClientPool {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_pool_creation() {
        let pool = MCPClientPool::new();
        assert_eq!(pool.client_count(), 0);
    }

    #[tokio::test]
    async fn test_client_pool_statistics() {
        let pool = MCPClientPool::new();
        let stats = pool.get_statistics().await;
        
        assert_eq!(stats.total_clients, 0);
        assert_eq!(stats.connected_clients, 0);
        assert_eq!(stats.disconnected_clients, 0);
        assert_eq!(stats.error_clients, 0);
    }

    #[tokio::test]
    async fn test_client_pool_cleanup() {
        let pool = MCPClientPool::new();
        
        // 测试清理空池
        assert!(pool.cleanup_disconnected_clients().await.is_ok());
        
        // 测试停止所有客户端
        assert!(pool.stop_all_clients().await.is_ok());
    }

    #[tokio::test]
    async fn test_client_pool_thread_safety() {
        let pool = Arc::new(MCPClientPool::new());
        let mut handles = Vec::new();

        // 创建多个并发任务来测试线程安全
        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let config = MCPServerConfig::new_stdio(
                    format!("test-{}", i),
                    format!("Test Server {}", i),
                    "echo".to_string(),
                    vec!["hello".to_string()],
                );
                
                // 这个测试可能会失败，因为echo命令可能不支持MCP协议
                // 但我们主要测试的是线程安全性，不是实际的连接
                let _ = pool_clone.create_client(config).await;
                
                // 测试并发访问
                let _ = pool_clone.client_count();
                let _ = pool_clone.has_client(&format!("test-{}", i));
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 清理
        pool.stop_all_clients().await.unwrap();
    }
}