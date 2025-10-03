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
    // 使用Arc<Mutex>来安全地共享stdin/stdout
    stdin: std::sync::Arc<tokio::sync::Mutex<Option<tokio::process::ChildStdin>>>,
    stdout: std::sync::Arc<tokio::sync::Mutex<Option<tokio::io::BufReader<tokio::process::ChildStdout>>>>,
    request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
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
            stdin: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            stdout: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }
    
    /// 发送JSON-RPC消息到子进程的stdin
    async fn send_message(&self, message: &crate::mcp::protocol::MCPMessage) -> Result<(), FlowyError> {
        use tokio::io::AsyncWriteExt;
        
        let mut stdin_guard = self.stdin.lock().await;
        let stdin = stdin_guard.as_mut()
            .ok_or_else(|| FlowyError::internal().with_context("Process stdin not available"))?;
        
        let json = serde_json::to_string(message)
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to serialize message: {}", e)))?;
        
        tracing::debug!("Sending STDIO message: {}", json);
        
        // 写入JSON + 换行符，捕获BrokenPipe错误避免SIGPIPE信号
        match stdin.write_all(json.as_bytes()).await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                return Err(FlowyError::internal().with_context("Process stdin closed (broken pipe). The MCP server process may have exited."));
            },
            Err(e) => {
                return Err(FlowyError::internal().with_context(format!("Failed to write to stdin: {}", e)));
            }
        }
        
        match stdin.write_all(b"\n").await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                return Err(FlowyError::internal().with_context("Process stdin closed (broken pipe). The MCP server process may have exited."));
            },
            Err(e) => {
                return Err(FlowyError::internal().with_context(format!("Failed to write newline: {}", e)));
            }
        }
        
        match stdin.flush().await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                return Err(FlowyError::internal().with_context("Process stdin closed (broken pipe). The MCP server process may have exited."));
            },
            Err(e) => {
                return Err(FlowyError::internal().with_context(format!("Failed to flush stdin: {}", e)));
            }
        }
        
        Ok(())
    }
    
    /// 从子进程的stdout读取JSON-RPC响应
    async fn read_response(&self) -> Result<crate::mcp::protocol::MCPMessage, FlowyError> {
        use tokio::io::AsyncBufReadExt;
        
        let mut stdout_guard = self.stdout.lock().await;
        let stdout = stdout_guard.as_mut()
            .ok_or_else(|| FlowyError::internal().with_context("Process stdout not available"))?;
        
        let mut line = String::new();
        let bytes_read = stdout.read_line(&mut line).await
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to read from stdout: {}", e)))?;
        
        if bytes_read == 0 {
            return Err(FlowyError::internal().with_context("Process closed stdout"));
        }
        
        tracing::debug!("Received STDIO response: {}", line.trim());
        
        serde_json::from_str::<crate::mcp::protocol::MCPMessage>(&line)
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to parse response: {}", e)))
    }
    
    /// 发送请求并等待响应
    async fn send_request(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, FlowyError> {
        use crate::mcp::protocol::MCPMessage;
        
        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let request = MCPMessage::request(
            serde_json::json!(id),
            method.to_string(),
            params,
        );
        
        self.send_message(&request).await?;
        
        // 等待响应，设置超时
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.read_response()
        ).await
            .map_err(|_| FlowyError::internal().with_context("Request timeout"))?
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to read response: {}", e)))?;
        
        // 检查是否有错误
        if let Some(error) = response.error {
            return Err(FlowyError::internal().with_context(format!("MCP error: {}", error.message)));
        }
        
        response.result
            .ok_or_else(|| FlowyError::internal().with_context("No result in response"))
    }
}

#[async_trait]
impl MCPClient for StdioMCPClient {
    async fn initialize(&mut self) -> Result<(), FlowyError> {
        use tokio::io::BufReader;
        
        self.status = MCPConnectionStatus::Connecting;
        
        let stdio_config = self.config.stdio_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing STDIO config"))?;
        
        tracing::info!("Starting STDIO MCP process: {} {:?}", stdio_config.command, stdio_config.args);
        
        // 启动子进程
        let mut cmd = tokio::process::Command::new(&stdio_config.command);
        cmd.args(&stdio_config.args);
        
        // 检查用户是否手动设置了 PATH 环境变量
        let user_set_path = stdio_config.env_vars.contains_key("PATH");
        
        // 如果用户没有手动设置 PATH，则自动添加命令所在目录到 PATH
        if !user_set_path {
            let command_path = std::path::Path::new(&stdio_config.command);
            if let Some(command_dir) = command_path.parent() {
                let command_dir_str = command_dir.to_string_lossy();
                
                // 获取当前的PATH环境变量
                let current_path = std::env::var("PATH").unwrap_or_default();
                
                // 获取平台相关的路径分隔符
                #[cfg(target_os = "windows")]
                let path_separator = ";";
                #[cfg(not(target_os = "windows"))]
                let path_separator = ":";
                
                // 构建新的PATH: 命令目录 + 当前PATH
                let new_path = if current_path.is_empty() {
                    // 如果没有PATH，创建一个基础的PATH
                    #[cfg(target_os = "windows")]
                    let default_path = format!("{};C:\\Windows\\System32;C:\\Windows", command_dir_str);
                    #[cfg(not(target_os = "windows"))]
                    let default_path = format!("{}:/usr/local/bin:/usr/bin:/bin", command_dir_str);
                    default_path
                } else if !current_path.contains(command_dir_str.as_ref()) {
                    // 如果命令目录不在PATH中，添加到最前面
                    format!("{}{}{}", command_dir_str, path_separator, current_path)
                } else {
                    // 命令目录已经在PATH中
                    current_path
                };
                
                tracing::info!("Auto-setting PATH for STDIO process: {}", new_path);
                cmd.env("PATH", new_path);
            }
        } else {
            tracing::info!("User manually set PATH, skipping auto-configuration");
        }
        
        // 设置用户配置的所有环境变量（包括用户手动设置的PATH）
        for (key, value) in &stdio_config.env_vars {
            tracing::debug!("Setting env var: {}={}", key, if key == "PATH" { "(user configured)" } else { value });
            cmd.env(key, value);
        }
        
        // 配置标准输入输出
        cmd.stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());
        
        let mut process = cmd.spawn()
            .map_err(|e| {
                let error_msg = format!("Failed to start STDIO process: {}", e);
                self.status = MCPConnectionStatus::Error(error_msg.clone());
                FlowyError::internal().with_context(error_msg)
            })?;
        
        // 获取stdin、stdout和stderr句柄
        let stdin = process.stdin.take()
            .ok_or_else(|| FlowyError::internal().with_context("Failed to get stdin"))?;
        let stdout = process.stdout.take()
            .ok_or_else(|| FlowyError::internal().with_context("Failed to get stdout"))?;
        let stderr = process.stderr.take()
            .ok_or_else(|| FlowyError::internal().with_context("Failed to get stderr"))?;
        
        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        self.process = Some(process);
        
        // 在后台任务中捕获stderr输出
        let server_name = self.config.name.clone();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut stderr_reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(n) = stderr_reader.read_line(&mut line).await {
                if n == 0 { break; }
                if !line.trim().is_empty() {
                    tracing::warn!("[STDIO stderr] {}: {}", server_name, line.trim());
                }
                line.clear();
            }
        });
        
        // 给进程一点时间启动，检查是否立即退出
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // 检查进程是否还在运行
        if let Some(ref mut proc) = self.process {
            match proc.try_wait() {
                Ok(Some(status)) => {
                    let error_msg = format!(
                        "MCP server process exited immediately with status: {}. Check stderr logs above for details.",
                        status
                    );
                    tracing::error!("{}", error_msg);
                    self.status = MCPConnectionStatus::Error(error_msg.clone());
                    return Err(FlowyError::internal().with_context(error_msg));
                }
                Ok(None) => {
                    tracing::debug!("MCP server process is running, proceeding with initialization");
                }
                Err(e) => {
                    tracing::warn!("Failed to check process status: {}", e);
                }
            }
        }
        
        // 发送MCP initialize请求
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "AppFlowy",
                "version": "1.0.0"
            }
        });
        
        tracing::info!("Sending initialize request to MCP server: {}", self.config.name);
        
        match self.send_request("initialize", Some(init_params)).await {
            Ok(result) => {
                tracing::info!("MCP server initialized successfully: {} - {:?}", self.config.name, result);
                
                // 发送initialized通知
                let notification = crate::mcp::protocol::MCPMessage::notification(
                    "notifications/initialized".to_string(),
                    None,
                );
                if let Err(e) = self.send_message(&notification).await {
                    tracing::warn!("Failed to send initialized notification: {}", e);
                }
                
                self.status = MCPConnectionStatus::Connected;
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize MCP server: {}", e);
                tracing::error!("{} - Please check stderr logs above for details", error_msg);
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
        
        tracing::info!("Requesting tool list from MCP server: {}", self.config.name);
        
        let result = self.send_request("tools/list", None).await?;
        
        tracing::debug!("Received tools/list response: {:?}", result);
        
        // 解析工具列表
        let tools_response: crate::mcp::protocol::ListToolsResponse = serde_json::from_value(result)
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to parse tools list: {}", e)))?;
        
        tracing::info!("Discovered {} tools from MCP server: {}", tools_response.tools.len(), self.config.name);
        
        Ok(ToolsList {
            tools: tools_response.tools,
        })
    }
    
    async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        tracing::info!("Calling tool '{}' on MCP server: {}", request.name, self.config.name);
        
        let params = serde_json::json!({
            "name": request.name,
            "arguments": request.arguments,
        });
        
        let result = self.send_request("tools/call", Some(params)).await?;
        
        tracing::debug!("Received tools/call response: {:?}", result);
        
        // 解析工具调用响应
        let call_response: crate::mcp::protocol::CallToolResponse = serde_json::from_value(result)
            .map_err(|e| FlowyError::internal().with_context(format!("Failed to parse tool call response: {}", e)))?;
        
        // 转换为我们的响应格式
        let content = call_response.content.into_iter().map(|c| {
            ToolCallContent {
                r#type: c.r#type,
                text: c.text,
                data: c.data,
            }
        }).collect();
        
        Ok(ToolCallResponse {
            content,
            is_error: call_response.is_error.unwrap_or(false),
        })
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
    
    /// 解析 SSE 响应为 MCPMessage
    fn parse_sse_response(&self, response_text: &str) -> Result<crate::mcp::protocol::MCPMessage, FlowyError> {
        use crate::mcp::protocol::MCPMessage;
        
        // 解析 SSE 格式: event: message\ndata: {...}\n\n
        let mut last_json = None;
        
        for line in response_text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if !data.trim().is_empty() && data.trim() != "[DONE]" {
                    match serde_json::from_str::<MCPMessage>(data) {
                        Ok(msg) => {
                            tracing::debug!("Parsed SSE message: {:?}", msg.method);
                            last_json = Some(msg);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse SSE message: {} - data: {}", e, data);
                        }
                    }
                }
            }
        }
        
        last_json.ok_or_else(|| FlowyError::internal().with_context("No valid SSE message found"))
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
    
    async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        let http_config = self.config.http_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config for SSE client"))?;
        
        // 构建 MCP 协议的 tools/call 请求
        use crate::mcp::protocol::{MCPMessage, CallToolRequest as ProtocolCallToolRequest};
        
        let mcp_request = ProtocolCallToolRequest {
            name: request.name.clone(),
            arguments: Some(request.arguments.clone()),
        };
        
        let message = MCPMessage::request(
            serde_json::json!(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()),
            "tools/call".to_string(),
            Some(serde_json::to_value(&mcp_request).map_err(|e| {
                FlowyError::internal().with_context(format!("Failed to serialize request: {}", e))
            })?),
        );
        
        // 发送 HTTP POST 请求
        let mut http_request = self.client
            .post(&http_config.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");  // SSE 需要支持 text/event-stream
        
        // 添加会话ID (如果有) - 使用 mcp-session-id 头
        if let Some(ref session_id) = self.session_id {
            http_request = http_request.header("mcp-session-id", session_id);
        }
        
        // 添加自定义头信息
        for (key, value) in &http_config.headers {
            http_request = http_request.header(key, value);
        }
        
        let json_body = serde_json::to_string(&message).map_err(|e| {
            FlowyError::internal().with_context(format!("Failed to serialize message: {}", e))
        })?;
        
        tracing::info!("🔧 [SSE CALL] Calling tool: {} on {}", request.name, http_config.url);
        tracing::info!("🔧 [SSE CALL] Request body: {}", json_body);
        tracing::info!("🔧 [SSE CALL] Session ID: {:?}", self.session_id);
        
        let response = http_request
            .body(json_body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("🔧 [SSE CALL] HTTP request error: {}", e);
                FlowyError::http().with_context(format!("HTTP request failed: {}", e))
            })?;
        
        let status = response.status();
        tracing::info!("🔧 [SSE CALL] Response status: {}", status);
        
        if !status.is_success() {
            // 尝试读取响应体以获取更多错误信息
            if let Ok(error_body) = response.text().await {
                tracing::error!("🔧 [SSE CALL] Error response body: {}", error_body);
                return Err(FlowyError::http()
                    .with_context(format!("HTTP {} - {}", status, error_body)));
            }
            return Err(FlowyError::http()
                .with_context(format!("HTTP request failed with status: {}", status)));
        }
        
        let response_text = response.text().await.map_err(|e| {
            FlowyError::http().with_context(format!("Failed to read response: {}", e))
        })?;
        
        tracing::info!("🔧 [SSE CALL] Response body: {}", 
                      response_text.chars().take(500).collect::<String>());
        
        // 解析响应 - 可能是 SSE 格式或普通 JSON
        let response_message: MCPMessage = if response_text.contains("event:") || response_text.contains("data:") {
            // SSE 格式,需要解析
            tracing::info!("🔧 [SSE CALL] Parsing SSE format response");
            self.parse_sse_response(&response_text)?
        } else {
            // 普通 JSON 格式
            tracing::info!("🔧 [SSE CALL] Parsing JSON format response");
            serde_json::from_str(&response_text).map_err(|e| {
                FlowyError::internal().with_context(format!("Failed to parse JSON response: {}", e))
            })?
        };
        
        tracing::info!("🔧 [SSE CALL] Parsed response message - has error: {}, has result: {}", 
                      response_message.error.is_some(), response_message.result.is_some());
        
        // 检查错误
        if let Some(error) = response_message.error {
            tracing::error!("🔧 [SSE CALL] MCP protocol error: {} (code: {})", error.message, error.code);
            return Err(FlowyError::internal()
                .with_context(format!("MCP error: {} (code: {})", error.message, error.code)));
        }
        
        // 提取工具调用结果
        let result = response_message.result.ok_or_else(|| {
            tracing::error!("🔧 [SSE CALL] No result field in MCP response");
            FlowyError::internal().with_context("No result in MCP response")
        })?;
        
        tracing::info!("🔧 [SSE CALL] Extracting tool response from result: {}", 
                      serde_json::to_string(&result).unwrap_or_default().chars().take(200).collect::<String>());
        
        use crate::mcp::protocol::CallToolResponse as ProtocolCallToolResponse;
        let tool_response: ProtocolCallToolResponse = serde_json::from_value(result).map_err(|e| {
            tracing::error!("🔧 [SSE CALL] Failed to parse CallToolResponse: {}", e);
            FlowyError::internal().with_context(format!("Failed to parse tool response: {}", e))
        })?;
        
        tracing::info!("🔧 [SSE CALL] Tool response parsed - content items: {}, is_error: {:?}", 
                      tool_response.content.len(), tool_response.is_error);
        
        // 转换为我们的响应格式
        let content = tool_response.content.into_iter().map(|c| {
            ToolCallContent {
                r#type: c.r#type,
                text: c.text,
                data: None,
            }
        }).collect();
        
        let final_response = ToolCallResponse {
            content,
            is_error: tool_response.is_error.unwrap_or(false),
        };
        
        tracing::info!("🔧 [SSE CALL] ✅ Tool call completed successfully - is_error: {}", final_response.is_error);
        
        Ok(final_response)
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
    
    async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, FlowyError> {
        if !self.is_connected() {
            return Err(FlowyError::invalid_data().with_context("Client not connected"));
        }
        
        let http_config = self.config.http_config.as_ref()
            .ok_or_else(|| FlowyError::invalid_data().with_context("Missing HTTP config"))?;
        
        // 构建 MCP 协议的 tools/call 请求
        use crate::mcp::protocol::{MCPMessage, CallToolRequest as ProtocolCallToolRequest};
        
        let mcp_request = ProtocolCallToolRequest {
            name: request.name.clone(),
            arguments: Some(request.arguments.clone()),
        };
        
        let message = MCPMessage::request(
            serde_json::json!(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()),
            "tools/call".to_string(),
            Some(serde_json::to_value(&mcp_request).map_err(|e| {
                FlowyError::internal().with_context(format!("Failed to serialize request: {}", e))
            })?),
        );
        
        // 发送 HTTP 请求
        let mut http_request = self.client
            .post(&http_config.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        
        // 添加自定义头信息
        for (key, value) in &http_config.headers {
            http_request = http_request.header(key, value);
        }
        
        let json_body = serde_json::to_string(&message).map_err(|e| {
            FlowyError::internal().with_context(format!("Failed to serialize message: {}", e))
        })?;
        
        tracing::debug!("Sending MCP tool call request: {}", json_body.chars().take(200).collect::<String>());
        
        let response = http_request
            .body(json_body)
            .send()
            .await
            .map_err(|e| {
                FlowyError::http().with_context(format!("HTTP request failed: {}", e))
            })?;
        
        if !response.status().is_success() {
            return Err(FlowyError::http()
                .with_context(format!("HTTP request failed with status: {}", response.status())));
        }
        
        let response_text = response.text().await.map_err(|e| {
            FlowyError::http().with_context(format!("Failed to read response: {}", e))
        })?;
        
        tracing::debug!("Received MCP response: {}", response_text.chars().take(200).collect::<String>());
        
        // 解析 MCP 响应
        let response_message: MCPMessage = serde_json::from_str(&response_text).map_err(|e| {
            FlowyError::internal().with_context(format!("Failed to parse MCP response: {}", e))
        })?;
        
        // 检查错误
        if let Some(error) = response_message.error {
            return Err(FlowyError::internal()
                .with_context(format!("MCP error: {} (code: {})", error.message, error.code)));
        }
        
        // 提取工具调用结果
        let result = response_message.result.ok_or_else(|| {
            FlowyError::internal().with_context("No result in MCP response")
        })?;
        
        use crate::mcp::protocol::CallToolResponse as ProtocolCallToolResponse;
        let tool_response: ProtocolCallToolResponse = serde_json::from_value(result).map_err(|e| {
            FlowyError::internal().with_context(format!("Failed to parse tool response: {}", e))
        })?;
        
        // 转换为我们的响应格式
        let content = tool_response.content.into_iter().map(|c| {
            ToolCallContent {
                r#type: c.r#type,
                text: c.text,
                data: None,
            }
        }).collect();
        
        Ok(ToolCallResponse {
            content,
            is_error: tool_response.is_error.unwrap_or(false),
        })
    }
    
    fn get_status(&self) -> MCPConnectionStatus {
        self.status.clone()
    }
    
    fn get_config(&self) -> &MCPServerConfig {
        &self.config
    }
}
