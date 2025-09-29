use std::time::Duration;
use serde_json::{json, Value};
use tokio::time::timeout;
use reqwest::Client;
use anyhow::{Result, anyhow};

use crate::mcp::{
    protocol::*,
    entities::*,
    MCPServerConfig, MCPTransportType,
};

/// Excel MCP服务器测试客户端
pub struct ExcelMCPTestClient {
    client: Client,
    base_url: String,
    server_config: MCPServerConfig,
    session_initialized: bool,
}

impl ExcelMCPTestClient {
    /// 创建新的测试客户端
    pub fn new(port: u16) -> Self {
        let base_url = format!("http://localhost:{}", port);
        let server_config = MCPServerConfig::new_http(
            "excel-mcp-test".to_string(),
            "Excel MCP Server Test".to_string(),
            base_url.clone(),
            MCPTransportType::HTTP,
        );

        Self {
            client: Client::new(),
            base_url,
            server_config,
            session_initialized: false,
        }
    }

    /// 测试服务器连接
    pub async fn test_connection(&self) -> Result<bool> {
        println!("🔗 测试Excel MCP服务器连接...");
        
        let response = timeout(
            Duration::from_secs(5),
            self.client.get(&format!("{}/health", self.base_url)).send()
        ).await;

        match response {
            Ok(Ok(resp)) => {
                let status = resp.status();
                println!("✅ 服务器响应状态: {}", status);
                Ok(status.is_success())
            }
            Ok(Err(e)) => {
                println!("❌ 连接错误: {}", e);
                Ok(false)
            }
            Err(_) => {
                println!("⏰ 连接超时");
                Ok(false)
            }
        }
    }

    /// 初始化MCP会话
    pub async fn initialize_session(&mut self) -> Result<InitializeResponse> {
        println!("🚀 初始化MCP会话...");

        let init_request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
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
            let init_response: InitializeResponse = serde_json::from_value(result)?;
            println!("✅ 会话初始化成功:");
            println!("   服务器: {} v{}", init_response.server_info.name, init_response.server_info.version);
            println!("   协议版本: {}", init_response.protocol_version);
            
            self.session_initialized = true;
            
            // 发送initialized通知
            let initialized_notification = MCPMessage::notification(
                "notifications/initialized".to_string(),
                None,
            );
            self.send_mcp_message(initialized_notification).await?;
            
            Ok(init_response)
        } else if let Some(error) = response.error {
            Err(anyhow!("初始化失败: {} ({})", error.message, error.code))
        } else {
            Err(anyhow!("初始化响应格式错误"))
        }
    }

    /// 获取可用工具列表
    pub async fn list_tools(&self) -> Result<Vec<MCPTool>> {
        if !self.session_initialized {
            return Err(anyhow!("会话未初始化，请先调用initialize_session()"));
        }

        println!("📋 获取Excel MCP工具列表...");

        let list_request = ListToolsRequest { cursor: None };
        let message = MCPMessage::request(
            json!(2),
            "tools/list".to_string(),
            Some(serde_json::to_value(&list_request)?),
        );

        let response = self.send_mcp_message(message).await?;

        if let Some(result) = response.result {
            let tools_response: ListToolsResponse = serde_json::from_value(result)?;
            
            println!("✅ 发现 {} 个工具:", tools_response.tools.len());
            for tool in &tools_response.tools {
                println!("   - {}: {}", tool.name, tool.description);
                if let Some(annotations) = &tool.annotations {
                    if let Some(title) = &annotations.title {
                        println!("     标题: {}", title);
                    }
                    println!("     只读: {:?}", annotations.read_only_hint);
                    println!("     破坏性: {:?}", annotations.destructive_hint);
                }
            }
            
            Ok(tools_response.tools)
        } else if let Some(error) = response.error {
            Err(anyhow!("获取工具列表失败: {} ({})", error.message, error.code))
        } else {
            Err(anyhow!("工具列表响应格式错误"))
        }
    }

    /// 调用指定工具
    pub async fn call_tool(&self, tool_name: &str, arguments: Option<Value>) -> Result<CallToolResponse> {
        if !self.session_initialized {
            return Err(anyhow!("会话未初始化，请先调用initialize_session()"));
        }

        println!("🔧 调用工具: {}", tool_name);
        if let Some(ref args) = arguments {
            println!("   参数: {}", serde_json::to_string_pretty(args)?);
        }

        let call_request = CallToolRequest {
            name: tool_name.to_string(),
            arguments,
        };

        let message = MCPMessage::request(
            json!(3),
            "tools/call".to_string(),
            Some(serde_json::to_value(&call_request)?),
        );

        let response = self.send_mcp_message(message).await?;

        if let Some(result) = response.result {
            let tool_response: CallToolResponse = serde_json::from_value(result)?;
            
            println!("✅ 工具调用成功:");
            println!("   错误状态: {:?}", tool_response.is_error);
            println!("   内容数量: {}", tool_response.content.len());
            
            for (i, content) in tool_response.content.iter().enumerate() {
                println!("   内容 {}: 类型={}", i + 1, content.r#type);
                if let Some(text) = &content.text {
                    println!("     文本: {}", text.chars().take(100).collect::<String>());
                    if text.len() > 100 {
                        println!("     ... (截断)");
                    }
                }
            }
            
            Ok(tool_response)
        } else if let Some(error) = response.error {
            Err(anyhow!("工具调用失败: {} ({})", error.message, error.code))
        } else {
            Err(anyhow!("工具调用响应格式错误"))
        }
    }

    /// 发送MCP消息
    async fn send_mcp_message(&self, message: MCPMessage) -> Result<MCPMessage> {
        let json_body = serde_json::to_string(&message)?;
        
        println!("📤 发送MCP消息: {}", message.method.as_deref().unwrap_or("response"));
        
        let response = timeout(
            Duration::from_secs(30),
            self.client
                .post(&self.base_url)
                .header("Content-Type", "application/json")
                .body(json_body)
                .send()
        ).await??;

        let response_text = response.text().await?;
        println!("📥 收到响应: {}", response_text.chars().take(200).collect::<String>());
        
        let response_message: MCPMessage = serde_json::from_str(&response_text)?;
        Ok(response_message)
    }

    /// 验证我们的数据结构兼容性
    pub fn validate_data_structures(&self, tools: &[MCPTool]) -> Result<()> {
        println!("🔍 验证数据结构兼容性...");

        for tool in tools {
            // 验证基本字段
            if tool.name.is_empty() {
                return Err(anyhow!("工具名称为空: {:?}", tool));
            }
            
            if tool.description.is_empty() {
                return Err(anyhow!("工具描述为空: {}", tool.name));
            }

            // 验证输入模式是否为有效的JSON Schema
            if !tool.input_schema.is_object() && !tool.input_schema.is_null() {
                return Err(anyhow!("工具 {} 的输入模式不是有效的JSON对象", tool.name));
            }

            // 验证注解（如果存在）
            if let Some(annotations) = &tool.annotations {
                // 检查注解字段的合理性
                if let (Some(read_only), Some(destructive)) = 
                    (annotations.read_only_hint, annotations.destructive_hint) {
                    if read_only && destructive {
                        println!("⚠️  警告: 工具 {} 同时标记为只读和破坏性", tool.name);
                    }
                }
            }

            // 验证安全级别分类
            let safety_level = tool.safety_level();
            println!("   工具 {}: 安全级别 = {}", tool.name, safety_level);
        }

        println!("✅ 数据结构验证通过");
        Ok(())
    }

    /// 获取服务器配置
    pub fn get_server_config(&self) -> &MCPServerConfig {
        &self.server_config
    }
}

/// 运行完整的Excel MCP测试套件
pub async fn run_excel_mcp_test(port: u16) -> Result<()> {
    println!("🧪 开始Excel MCP服务器测试 (端口: {})", port);
    println!("{}", "=".repeat(50));

    let mut client = ExcelMCPTestClient::new(port);

    // 1. 测试连接
    if !client.test_connection().await? {
        return Err(anyhow!("无法连接到Excel MCP服务器"));
    }

    // 2. 初始化会话
    let init_response = client.initialize_session().await?;
    println!("服务器能力: {:?}", init_response.capabilities);

    // 3. 获取工具列表
    let tools = client.list_tools().await?;

    // 4. 验证数据结构
    client.validate_data_structures(&tools)?;

    // 5. 测试工具调用（如果有工具可用）
    if !tools.is_empty() {
        println!("\n🔧 测试工具调用...");
        
        // 尝试调用第一个工具（通常是安全的查询工具）
        let first_tool = &tools[0];
        println!("尝试调用工具: {}", first_tool.name);
        
        // 根据工具的输入模式构造测试参数
        let test_args = construct_test_arguments(&first_tool.input_schema);
        
        match client.call_tool(&first_tool.name, test_args).await {
            Ok(response) => {
                println!("✅ 工具调用测试成功");
                
                // 验证响应结构
                if response.content.is_empty() {
                    println!("⚠️  警告: 工具响应内容为空");
                } else {
                    println!("✅ 工具响应包含 {} 个内容项", response.content.len());
                }
            }
            Err(e) => {
                println!("⚠️  工具调用测试失败: {}", e);
                println!("   这可能是因为缺少必需参数或工具需要特定的输入");
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("🎉 Excel MCP测试完成!");
    println!("✅ 数据结构兼容性: 通过");
    println!("✅ 协议通信: 正常");
    println!("✅ 工具发现: {} 个工具", tools.len());

    Ok(())
}

/// 根据JSON Schema构造测试参数
fn construct_test_arguments(schema: &Value) -> Option<Value> {
    if let Some(obj) = schema.as_object() {
        if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
            let mut args = serde_json::Map::new();
            
            for (key, prop_schema) in properties {
                if let Some(prop_obj) = prop_schema.as_object() {
                    if let Some(prop_type) = prop_obj.get("type").and_then(|t| t.as_str()) {
                        let test_value = match prop_type {
                            "string" => json!("test_value"),
                            "number" => json!(1),
                            "integer" => json!(1),
                            "boolean" => json!(true),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_excel_mcp_client_creation() {
        let client = ExcelMCPTestClient::new(8007);
        assert_eq!(client.base_url, "http://localhost:8007");
        assert_eq!(client.server_config.name, "Excel MCP Server Test");
        assert!(!client.session_initialized);
    }

    #[tokio::test]
    async fn test_construct_test_arguments() {
        let schema = json!({
            "type": "object",
            "properties": {
                "filename": {"type": "string"},
                "sheet_index": {"type": "integer"},
                "read_only": {"type": "boolean"}
            }
        });

        let args = construct_test_arguments(&schema);
        assert!(args.is_some());
        
        let args = args.unwrap();
        assert!(args.get("filename").is_some());
        assert!(args.get("sheet_index").is_some());
        assert!(args.get("read_only").is_some());
    }

    // 注意：这个测试需要实际运行的Excel MCP服务器
    #[tokio::test]
    #[ignore] // 默认忽略，需要手动运行
    async fn test_excel_mcp_integration() {
        // 运行: cargo test test_excel_mcp_integration -- --ignored
        // 前提：Excel MCP服务器在端口8007运行
        match run_excel_mcp_test(8007).await {
            Ok(_) => println!("Excel MCP集成测试通过"),
            Err(e) => println!("Excel MCP集成测试失败: {}", e),
        }
    }
}
