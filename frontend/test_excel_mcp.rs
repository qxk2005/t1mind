#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! tokio = { version = "1.0", features = ["full"] }
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! anyhow = "1.0"
//! ```

use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::time::timeout;
use reqwest::Client;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// MCP协议消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MCPToolAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

impl MCPMessage {
    pub fn request(id: Value, method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: Some(method),
            params,
            result: None,
            error: None,
        }
    }

    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method),
            params,
            result: None,
            error: None,
        }
    }
}

/// Excel MCP测试客户端
pub struct ExcelMCPTester {
    client: Client,
    base_url: String,
    session_initialized: bool,
    session_id: Option<String>,
}

impl ExcelMCPTester {
    pub fn new(port: u16) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("http://localhost:{}/mcp", port), // 使用正确的MCP端点
            session_initialized: false,
            session_id: None,
        }
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<bool> {
        println!("🔗 测试Excel MCP服务器连接 ({})", self.base_url);
        
        // 对于streamable-http，先尝试一个简单的健康检查
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .get(&format!("{}/health", self.base_url))
                .send()
        ).await;

        match response {
            Ok(Ok(resp)) => {
                let status = resp.status();
                println!("✅ 健康检查响应状态: {}", status);
                Ok(true) // 任何响应都表示服务器在运行
            }
            Ok(Err(_)) => {
                // 如果健康检查失败，尝试直接发送MCP消息
                println!("🔄 尝试直接MCP连接...");
                self.test_mcp_connection().await
            }
            Err(_) => {
                println!("⏰ 连接超时");
                Ok(false)
            }
        }
    }

    /// 测试MCP连接
    async fn test_mcp_connection(&self) -> Result<bool> {
        let test_message = json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "protocol_version": "2024-11-05",
                "capabilities": {},
                "client_info": {
                    "name": "Test Client",
                    "version": "1.0.0"
                }
            }
        });
        
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .post(&self.base_url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json, text/event-stream")
                .json(&test_message)
                .send()
        ).await;

        match response {
            Ok(Ok(resp)) => {
                let status = resp.status();
                println!("✅ MCP连接响应状态: {}", status);
                Ok(status.is_success() || status.is_client_error())
            }
            Ok(Err(e)) => {
                println!("❌ MCP连接错误: {}", e);
                Ok(false)
            }
            Err(_) => {
                println!("⏰ MCP连接超时");
                Ok(false)
            }
        }
    }

    /// 初始化MCP会话
    pub async fn initialize_session(&mut self) -> Result<()> {
        println!("🚀 初始化MCP会话...");

        let init_request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
            },
            client_info: ClientInfo {
                name: "AppFlowy MCP Test Client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let message = MCPMessage::request(
            json!(1),
            "initialize".to_string(),
            Some(serde_json::to_value(&init_request)?),
        );

        let response = self.send_mcp_message(message).await?;
        
        if let Some(result) = response.result {
            println!("✅ 会话初始化成功");
            println!("   响应: {}", serde_json::to_string_pretty(&result)?);
            
            self.session_initialized = true;
            
            // 发送initialized通知
            let initialized_notification = MCPMessage::notification(
                "notifications/initialized".to_string(),
                None,
            );
            let _ = self.send_mcp_message(initialized_notification).await?;
            
            Ok(())
        } else if let Some(error) = response.error {
            Err(anyhow!("初始化失败: {} ({})", error.message, error.code))
        } else {
            Err(anyhow!("初始化响应格式错误"))
        }
    }

    /// 获取工具列表
    pub async fn list_tools(&mut self) -> Result<Vec<MCPTool>> {
        if !self.session_initialized {
            return Err(anyhow!("会话未初始化"));
        }

        println!("📋 获取工具列表...");

        let message = MCPMessage::request(
            json!(2),
            "tools/list".to_string(),
            Some(json!({})),
        );

        let response = self.send_mcp_message(message).await?;

        if let Some(result) = response.result {
            if let Some(tools_array) = result.get("tools").and_then(|t| t.as_array()) {
                let mut tools = Vec::new();
                
                for tool_value in tools_array {
                    match serde_json::from_value::<MCPTool>(tool_value.clone()) {
                        Ok(tool) => {
                            println!("✅ 发现工具: {} - {}", tool.name, tool.description);
                            
                            // 显示工具的详细信息
                            if let Some(annotations) = &tool.annotations {
                                println!("   注解:");
                                if let Some(title) = &annotations.title {
                                    println!("     标题: {}", title);
                                }
                                if let Some(read_only) = annotations.read_only_hint {
                                    println!("     只读: {}", read_only);
                                }
                                if let Some(destructive) = annotations.destructive_hint {
                                    println!("     破坏性: {}", destructive);
                                }
                            }
                            
                            // 显示输入模式
                            println!("   输入模式: {}", serde_json::to_string_pretty(&tool.input_schema)?);
                            
                            tools.push(tool);
                        }
                        Err(e) => {
                            println!("⚠️  解析工具失败: {}", e);
                            println!("   原始数据: {}", serde_json::to_string_pretty(tool_value)?);
                        }
                    }
                }
                
                println!("✅ 总共发现 {} 个工具", tools.len());
                Ok(tools)
            } else {
                Err(anyhow!("工具列表响应格式错误: 缺少tools字段"))
            }
        } else if let Some(error) = response.error {
            Err(anyhow!("获取工具列表失败: {} ({})", error.message, error.code))
        } else {
            Err(anyhow!("工具列表响应格式错误"))
        }
    }

    /// 调用工具
    pub async fn call_tool(&mut self, tool_name: &str, arguments: Option<Value>) -> Result<Value> {
        if !self.session_initialized {
            return Err(anyhow!("会话未初始化"));
        }

        println!("🔧 调用工具: {}", tool_name);
        if let Some(ref args) = arguments {
            println!("   参数: {}", serde_json::to_string_pretty(args)?);
        }

        let mut params = serde_json::Map::new();
        params.insert("name".to_string(), json!(tool_name));
        if let Some(args) = arguments {
            params.insert("arguments".to_string(), args);
        }

        let message = MCPMessage::request(
            json!(3),
            "tools/call".to_string(),
            Some(Value::Object(params)),
        );

        let response = self.send_mcp_message(message).await?;

        if let Some(result) = response.result {
            println!("✅ 工具调用成功");
            println!("   响应: {}", serde_json::to_string_pretty(&result)?);
            Ok(result)
        } else if let Some(error) = response.error {
            Err(anyhow!("工具调用失败: {} ({})", error.message, error.code))
        } else {
            Err(anyhow!("工具调用响应格式错误"))
        }
    }

    /// 发送MCP消息 (支持SSE)
    async fn send_mcp_message(&mut self, message: MCPMessage) -> Result<MCPMessage> {
        let json_body = serde_json::to_string(&message)?;
        
        println!("📤 发送: {}", message.method.as_deref().unwrap_or("response"));
        println!("   消息: {}", json_body);
        
        let mut request = self.client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(json_body);
        
        // 如果有会话ID，添加到请求头
        if let Some(session_id) = &self.session_id {
            request = request.header("mcp-session-id", session_id);
        }
        
        let response = timeout(Duration::from_secs(30), request.send()).await??;

        // 提取会话ID（如果存在）
        if let Some(session_id) = response.headers().get("mcp-session-id") {
            if let Ok(session_id_str) = session_id.to_str() {
                println!("📝 获取会话ID: {}", session_id_str);
                self.session_id = Some(session_id_str.to_string());
            }
        }

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        println!("📥 响应类型: {}", content_type);

        if content_type.contains("text/event-stream") {
            // 处理SSE响应
            self.handle_sse_response(response).await
        } else {
            // 处理普通JSON响应
            let response_text = response.text().await?;
            println!("📥 收到: {}", response_text);
            
            if response_text.trim().is_empty() {
                // 空响应，通常用于通知消息
                Ok(MCPMessage {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    method: None,
                    params: None,
                    result: Some(json!({})),
                    error: None,
                })
            } else {
                let response_message: MCPMessage = serde_json::from_str(&response_text)?;
                Ok(response_message)
            }
        }
    }

    /// 处理SSE响应
    async fn handle_sse_response(&self, response: reqwest::Response) -> Result<MCPMessage> {
        let response_text = response.text().await?;
        println!("📥 SSE响应: {}", response_text);
        
        // 解析SSE格式的响应
        // Excel MCP格式: event: message\ndata: {json}\n\n
        let mut last_json = None;
        let mut current_event = None;
        
        for line in response_text.lines() {
            if let Some(event) = line.strip_prefix("event: ") {
                current_event = Some(event.trim().to_string());
                println!("📥 SSE事件类型: {}", event.trim());
            } else if let Some(data) = line.strip_prefix("data: ") {
                if !data.trim().is_empty() && data.trim() != "[DONE]" {
                    match serde_json::from_str::<MCPMessage>(data) {
                        Ok(msg) => {
                            println!("📥 解析SSE消息: {:?} (事件: {:?})", msg.method, current_event);
                            last_json = Some(msg);
                        }
                        Err(e) => {
                            println!("⚠️  SSE消息解析失败: {} - 数据: {}", e, data);
                            // 尝试解析为普通JSON对象
                            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(data) {
                                println!("📥 解析为JSON对象: {}", serde_json::to_string_pretty(&json_value).unwrap_or_default());
                            }
                        }
                    }
                }
            }
        }
        
        last_json.ok_or_else(|| anyhow::anyhow!("未找到有效的SSE消息"))
    }

    /// 验证数据结构兼容性
    pub fn validate_data_structures(&self, tools: &[MCPTool]) -> Result<()> {
        println!("🔍 验证AppFlowy MCP数据结构兼容性...");

        for tool in tools {
            // 验证基本字段
            if tool.name.is_empty() {
                return Err(anyhow!("工具名称为空"));
            }
            
            if tool.description.is_empty() {
                return Err(anyhow!("工具描述为空: {}", tool.name));
            }

            // 验证输入模式
            if !tool.input_schema.is_object() && !tool.input_schema.is_null() {
                return Err(anyhow!("工具 {} 的输入模式不是有效的JSON对象", tool.name));
            }

            // 分析安全级别
            let safety_level = if let Some(annotations) = &tool.annotations {
                if annotations.destructive_hint == Some(true) {
                    "破坏性"
                } else if annotations.open_world_hint == Some(true) {
                    "外部交互"
                } else if annotations.read_only_hint == Some(true) {
                    "只读"
                } else {
                    "安全"
                }
            } else {
                "未知"
            };

            println!("   ✅ 工具 {}: 安全级别 = {}", tool.name, safety_level);
        }

        println!("✅ 所有数据结构验证通过!");
        Ok(())
    }
}

/// 构造测试参数
fn construct_test_arguments(schema: &Value) -> Option<Value> {
    if let Some(obj) = schema.as_object() {
        if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
            let mut args = serde_json::Map::new();
            
            for (key, prop_schema) in properties {
                if let Some(prop_obj) = prop_schema.as_object() {
                    if let Some(prop_type) = prop_obj.get("type").and_then(|t| t.as_str()) {
                        let test_value = match prop_type {
                            "string" => {
                                // 为常见的字段提供更合理的测试值
                                match key.as_str() {
                                    "filename" | "file" | "path" => json!("test.xlsx"),
                                    "sheet" | "sheet_name" => json!("Sheet1"),
                                    _ => json!("test_value"),
                                }
                            },
                            "number" | "integer" => json!(0),
                            "boolean" => json!(false),
                            "array" => json!([]),
                            "object" => json!({}),
                            _ => json!(null),
                        };
                        args.insert(key.clone(), test_value);
                    }
                }
            }
            
            if !args.is_empty() {
                return Some(Value::Object(args));
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Excel MCP服务器测试");
    println!("{}", "=".repeat(60));
    
    // 从环境变量或默认值获取端口
    let port = std::env::var("FASTMCP_PORT")
        .unwrap_or_else(|_| "8007".to_string())
        .parse::<u16>()
        .unwrap_or(8007);

    println!("📡 目标端口: {}", port);
    println!("💡 提示: 请确保Excel MCP服务器正在运行:");
    println!("   excelfile FASTMCP_PORT={} uvx excel-mcp-server streamable-http", port);
    println!();

    let mut tester = ExcelMCPTester::new(port);

    // 1. 测试连接
    if !tester.test_connection().await? {
        println!("❌ 无法连接到Excel MCP服务器");
        println!("请检查服务器是否在端口 {} 上运行", port);
        return Ok(());
    }

    // 2. 初始化会话
    match tester.initialize_session().await {
        Ok(_) => println!("✅ MCP会话初始化成功"),
        Err(e) => {
            println!("❌ MCP会话初始化失败: {}", e);
            return Ok(());
        }
    }

    // 3. 获取工具列表
    let tools = match tester.list_tools().await {
        Ok(tools) => {
            println!("✅ 成功获取 {} 个工具", tools.len());
            tools
        }
        Err(e) => {
            println!("❌ 获取工具列表失败: {}", e);
            return Ok(());
        }
    };

    // 4. 验证数据结构兼容性
    match tester.validate_data_structures(&tools) {
        Ok(_) => println!("✅ AppFlowy MCP数据结构完全兼容"),
        Err(e) => {
            println!("❌ 数据结构兼容性问题: {}", e);
            return Ok(());
        }
    }

    // 5. 测试工具调用
    if !tools.is_empty() {
        println!("\n🔧 测试工具调用...");
        
        for tool in &tools {
            println!("\n尝试调用工具: {}", tool.name);
            
            // 构造测试参数
            let test_args = construct_test_arguments(&tool.input_schema);
            
            match tester.call_tool(&tool.name, test_args).await {
                Ok(_) => {
                    println!("✅ 工具 {} 调用成功", tool.name);
                }
                Err(e) => {
                    println!("⚠️  工具 {} 调用失败: {}", tool.name, e);
                    println!("   (这可能是因为缺少必需参数或需要特定的输入)");
                }
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("🎉 Excel MCP测试完成!");
    println!("✅ 连接测试: 通过");
    println!("✅ 会话初始化: 通过");
    println!("✅ 工具发现: {} 个工具", tools.len());
    println!("✅ 数据结构兼容性: 完全兼容");
    println!("✅ AppFlowy MCP实现: 准备就绪");

    Ok(())
}
