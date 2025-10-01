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

/// SSE MCP客户端实现（参考test_excel_mcp.rs）
pub struct SSEMCPClient {
    config: MCPServerConfig,
    status: MCPConnectionStatus,
    tools: Vec<MCPTool>,
    client: reqwest::Client,
    session_id: Option<String>,  // 会话ID，通过HTTP头传递
}

impl SSEMCPClient {
    pub fn new(config: MCPServerConfig) -> Result<Self, FlowyError> {
        if config.transport_type != MCPTransportType::SSE {
            return Err(FlowyError::invalid_data().with_context("Invalid transport type for SSE client"));
        }
        
        // 创建HTTP客户端
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| FlowyError::http().with_context(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            config,
            status: MCPConnectionStatus::Disconnected,
            tools: Vec::new(),
            client,
            session_id: None,  // 初始化时没有会话ID
        })
    }
    
    /// 解析MCP响应（支持SSE格式和纯JSON格式）
    /// 参考test_excel_mcp.rs的handle_sse_response实现
    fn parse_mcp_response(&self, response_text: &str) -> Result<serde_json::Value, FlowyError> {
        // 先尝试直接解析JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_text) {
            tracing::debug!("Parsed as direct JSON response");
            return Ok(json);
        }
        
        // 如果失败，尝试解析SSE格式
        // SSE格式: event: message\ndata: {json}\n\n
        tracing::debug!("Attempting to parse as SSE format");
        
        for line in response_text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();
                if !data.is_empty() && data != "[DONE]" {
                    match serde_json::from_str::<serde_json::Value>(data) {
                        Ok(json) => {
                            tracing::debug!("Successfully parsed SSE data line as JSON");
                            return Ok(json);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse SSE data line: {} - {}", e, data);
                        }
                    }
                }
            }
        }
        
        Err(FlowyError::http().with_context(format!(
            "Failed to parse response as JSON or SSE format. Response: {}", 
            response_text.chars().take(200).collect::<String>()
        )))
    }
}

#[async_trait]
impl MCPClient for SSEMCPClient {
    async fn initialize(&mut self) -> Result<(), FlowyError> {
        self.status = MCPConnectionStatus::Connecting;
        
        let http_config = self.config.http_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
        
        // 1. 发送 initialize 请求（参考test_excel_mcp.rs第207-224行）
        let init_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "AppFlowy",
                    "version": "1.0.0"
                }
            }
        });
        
        let mut request = self.client
            .post(&http_config.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&init_message);
        
        for (key, value) in &http_config.headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await
            .map_err(|e| FlowyError::http().with_context(format!("Failed to send initialize: {}", e)))?;
        
        let status = response.status();
        
        // 2. 提取会话ID（参考test_excel_mcp.rs第366-372行）
        if let Some(session_id) = response.headers().get("mcp-session-id") {
            if let Ok(session_id_str) = session_id.to_str() {
                tracing::info!("Got session ID from initialize: {}", session_id_str);
                self.session_id = Some(session_id_str.to_string());
            }
        }
        
        if !status.is_success() {
            let error_msg = format!("Initialize failed with status: {}", status);
            self.status = MCPConnectionStatus::Error(error_msg.clone());
            return Err(FlowyError::http().with_context(error_msg));
        }
        
        let response_text = response.text().await
            .map_err(|e| FlowyError::http().with_context(format!("Failed to read initialize response: {}", e)))?;
        tracing::debug!("Initialize response: {}", response_text);
        
        // 3. 发送 notifications/initialized 通知（参考test_excel_mcp.rs第232-237行）
        // 注意：params 是 None，不是空对象！
        let initialized_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        
        let mut notify_request = self.client
            .post(&http_config.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");
        
        // 添加会话ID头（参考test_excel_mcp.rs第359-362行）
        if let Some(ref session_id) = self.session_id {
            notify_request = notify_request.header("mcp-session-id", session_id);
        }
        
        for (key, value) in &http_config.headers {
            notify_request = notify_request.header(key, value);
        }
        
        notify_request = notify_request.json(&initialized_notification);
        
        let notify_response = notify_request.send().await
            .map_err(|e| FlowyError::http().with_context(format!("Failed to send initialized notification: {}", e)))?;
        
        let notify_status = notify_response.status();
        
        // 4. 再次提取会话ID
        if let Some(session_id) = notify_response.headers().get("mcp-session-id") {
            if let Ok(session_id_str) = session_id.to_str() {
                tracing::info!("Got session ID from initialized notification: {}", session_id_str);
                self.session_id = Some(session_id_str.to_string());
            }
        }
        
        // 读取响应（可能为空）
        if let Ok(response_text) = notify_response.text().await {
            if !response_text.is_empty() {
                if notify_status.is_success() {
                    tracing::info!("Initialized notification response (status: {}): {}", notify_status, response_text);
                } else {
                    tracing::warn!("Initialized notification failed (status: {}): {}", notify_status, response_text);
                }
            } else {
                tracing::info!("Initialized notification sent (status: {}, empty response)", notify_status);
            }
        }
        
        self.status = MCPConnectionStatus::Connected;
        tracing::info!("SSE MCP client initialized for: {} with session_id: {:?}", 
            self.config.name, self.session_id);
        Ok(())
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
        
        let http_config = self.config.http_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
        
        // 构建MCP tools/list请求
        let list_tools_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });
        
        let mut request = self.client
            .post(&http_config.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");
        
        // 添加会话ID头（参考test_excel_mcp.rs第359-362行）
        if let Some(ref session_id) = self.session_id {
            request = request.header("mcp-session-id", session_id);
            tracing::debug!("Adding session ID to tools/list request: {}", session_id);
        }
        
        // 添加用户自定义头信息
        for (key, value) in &http_config.headers {
            request = request.header(key, value);
        }
        
        request = request.json(&list_tools_message);
        
        match request.send().await {
            Ok(response) => {
                let response_text = response.text().await
                    .map_err(|e| FlowyError::http().with_context(format!("Failed to read response: {}", e)))?;
                
                tracing::debug!("SSE tools/list raw response: {}", response_text);
                
                // 尝试解析SSE格式或纯JSON格式
                let response_json = self.parse_mcp_response(&response_text)?;
                
                if let Some(result) = response_json.get("result") {
                    if let Some(tools_array) = result.get("tools").and_then(|t| t.as_array()) {
                        let tools: Vec<MCPTool> = tools_array.iter()
                            .filter_map(|tool_value| {
                                serde_json::from_value(tool_value.clone()).ok()
                            })
                            .collect();
                        
                        tracing::info!("SSE MCP client found {} tools for: {}", tools.len(), self.config.name);
                        return Ok(ToolsList { tools });
                    }
                }
                
                tracing::error!("Invalid tools/list response format, response: {}", response_text);
                Err(FlowyError::http().with_context("Invalid tools/list response format"))
            }
            Err(e) => {
                Err(FlowyError::http().with_context(format!("Failed to list tools: {}", e)))
            }
        }
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
