use crate::mcp::entities::*;
use async_trait::async_trait;
use flowy_error::FlowyError;

/// MCP客户端trait，定义所有传输方式的通用接口
#[async_trait]
pub trait MCPClient: Send + Sync {
    /// 初始化连接
    async fn initialize(&mut self) -> Result<(), FlowyError>;
    
    /// 停止连接
    async fn stop(&mut self) -> Result<(), FlowyError>;
    
    /// 获取工具列表
    async fn list_tools(&self) -> Result<ToolsList, FlowyError>;
    
    /// 调用工具
    async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError>;
    
    /// 获取连接状态
    fn get_status(&self) -> MCPConnectionStatus;
    
    /// 获取服务器配置
    fn get_config(&self) -> &MCPServerConfig;
    
    /// 检查连接是否活跃
    fn is_connected(&self) -> bool {
        matches!(self.get_status(), MCPConnectionStatus::Connected)
    }
}

/// STDIO MCP客户端实现
pub struct StdioMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    tools: Vec<MCPTool>,
    process: Option<tokio::process::Child>,
}

impl StdioMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        if config.transport_type != MCPTransportType::Stdio {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for STDIO client"));
        }
        
        Ok(Self {
            config,
            status: MCPConnectionStatus::Disconnected,
            tools: Vec::new(),
            process: None,
        })
    }
}

#[async_trait]
impl MCPClient for StdioMCPClient {
    async fn initialize(&mut self) -> Result<(), FlowyError> {
        self.status = MCPConnectionStatus::Connecting;
        
        let stdio_config = self.config.stdio_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing STDIO config"))?;
        
        // 启动子进程
        let mut cmd = tokio::process::Command::new(&stdio_config.command);
        cmd.args(&stdio_config.args);
        
        // 设置环境变量
        for (key, value) in &stdio_config.env_vars {
            cmd.env(key, value);
        }
        
        // 配置标准输入输出
        cmd.stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());
        
        match cmd.spawn() {
            Ok(process) => {
                self.process = Some(process);
                self.status = MCPConnectionStatus::Connected;
                tracing::info!("STDIO MCP client initialized for: {}", self.config.name);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to start STDIO process: {}", e);
                self.status = MCPConnectionStatus::Error(error_msg.clone());
                Err(FlowyError::internal().with_context(error_msg))
            }
        }
    }
    
    async fn stop(&mut self) -> Result<(), FlowyError> {
        if let Some(mut process) = self.process.take() {
            match process.kill().await {
                Ok(_) => {
                    self.status = MCPConnectionStatus::Disconnected;
                    tracing::info!("STDIO MCP client stopped for: {}", self.config.name);
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to stop STDIO process: {}", e);
                    self.status = MCPConnectionStatus::Error(error_msg.clone());
                    Err(FlowyError::internal().with_context(error_msg))
                }
            }
        } else {
            self.status = MCPConnectionStatus::Disconnected;
            Ok(())
        }
    }
    
    async fn list_tools(&self) -> Result<ToolsList, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        // TODO: 实现实际的MCP协议通信
        // 这里先返回缓存的工具列表
        Ok(ToolsList {
            tools: self.tools.clone(),
        })
    }
    
    async fn call_tool(&self, _request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        // TODO: 实现实际的工具调用
        Err(FlowyError::not_support().with_context("Tool calling not implemented yet"))
    }
    
    fn get_status(&self) -> MCPConnectionStatus {
        self.status.clone()
    }
    
    fn get_config(&self) -> &MCPServerConfig {
        &self.config
    }
}

/// SSE MCP客户端实现
pub struct SSEMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    tools: Vec<MCPTool>,
    client: reqwest::Client,
}

impl SSEMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        if config.transport_type != MCPTransportType::SSE {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for SSE client"));
        }
        
        Ok(Self {
            config,
            status: MCPConnectionStatus::Disconnected,
            tools: Vec::new(),
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl MCPClient for SSEMCPClient {
    async fn initialize(&mut self) -> Result<(), FlowyError> {
        self.status = MCPConnectionStatus::Connecting;
        
        let http_config = self.config.http_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
        
        // 测试连接
        let mut request = self.client.get(&http_config.url);
        
        // 添加头信息
        for (key, value) in &http_config.headers {
            request = request.header(key, value);
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    self.status = MCPConnectionStatus::Connected;
                    tracing::info!("SSE MCP client initialized for: {}", self.config.name);
                    Ok(())
                } else {
                    let error_msg = format!("SSE connection failed with status: {}", response.status());
                    self.status = MCPConnectionStatus::Error(error_msg.clone());
                    Err(FlowyError::http().with_context(error_msg))
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to connect to SSE endpoint: {}", e);
                self.status = MCPConnectionStatus::Error(error_msg.clone());
                Err(FlowyError::http().with_context(error_msg))
            }
        }
    }
    
    async fn stop(&mut self) -> Result<(), FlowyError> {
        self.status = MCPConnectionStatus::Disconnected;
        tracing::info!("SSE MCP client stopped for: {}", self.config.name);
        Ok(())
    }
    
    async fn list_tools(&self) -> Result<ToolsList, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        // TODO: 实现实际的MCP协议通信
        Ok(ToolsList {
            tools: self.tools.clone(),
        })
    }
    
    async fn call_tool(&self, _request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        // TODO: 实现实际的工具调用
        Err(FlowyError::not_support().with_context("Tool calling not implemented yet"))
    }
    
    fn get_status(&self) -> MCPConnectionStatus {
        self.status.clone()
    }
    
    fn get_config(&self) -> &MCPServerConfig {
        &self.config
    }
}

/// HTTP MCP客户端实现
pub struct HttpMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    tools: Vec<MCPTool>,
    client: reqwest::Client,
}

impl HttpMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        if config.transport_type != MCPTransportType::HTTP {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for HTTP client"));
        }
        
        Ok(Self {
            config,
            status: MCPConnectionStatus::Disconnected,
            tools: Vec::new(),
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl MCPClient for HttpMCPClient {
    async fn initialize(&mut self) -> Result<(), FlowyError> {
        self.status = MCPConnectionStatus::Connecting;
        
        let http_config = self.config.http_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
        
        // 测试连接
        let mut request = self.client.get(&http_config.url);
        
        // 添加头信息
        for (key, value) in &http_config.headers {
            request = request.header(key, value);
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    self.status = MCPConnectionStatus::Connected;
                    tracing::info!("HTTP MCP client initialized for: {}", self.config.name);
                    Ok(())
                } else {
                    let error_msg = format!("HTTP connection failed with status: {}", response.status());
                    self.status = MCPConnectionStatus::Error(error_msg.clone());
                    Err(FlowyError::http().with_context(error_msg))
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to connect to HTTP endpoint: {}", e);
                self.status = MCPConnectionStatus::Error(error_msg.clone());
                Err(FlowyError::http().with_context(error_msg))
            }
        }
    }
    
    async fn stop(&mut self) -> Result<(), FlowyError> {
        self.status = MCPConnectionStatus::Disconnected;
        tracing::info!("HTTP MCP client stopped for: {}", self.config.name);
        Ok(())
    }
    
    async fn list_tools(&self) -> Result<ToolsList, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        // TODO: 实现实际的MCP协议通信
        Ok(ToolsList {
            tools: self.tools.clone(),
        })
    }
    
    async fn call_tool(&self, _request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        // TODO: 实现实际的工具调用
        Err(FlowyError::not_support().with_context("Tool calling not implemented yet"))
    }
    
    fn get_status(&self) -> MCPConnectionStatus {
        self.status.clone()
    }
    
    fn get_config(&self) -> &MCPServerConfig {
        &self.config
    }
}
