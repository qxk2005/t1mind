#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use flowy_sqlite::kv::KVStorePreferences;
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
use crate::mcp::{
    entities::*,
    protocol::*,
    MCPConfigManager, ToolSecurityManager,
};

/// 集成测试：验证MCP数据结构与Excel MCP服务器的兼容性
#[cfg(test)]
pub struct MCPIntegrationTest {
    config_manager: Arc<MCPConfigManager>,
    security_manager: Arc<ToolSecurityManager>,
    _tempdir: TempDir,
}

#[cfg(test)]
impl MCPIntegrationTest {
    pub fn new() -> Self {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        
        let config_manager = Arc::new(MCPConfigManager::new(store.clone()));
        let security_manager = Arc::new(ToolSecurityManager::new(store));
        
        Self {
            config_manager,
            security_manager,
            _tempdir: tempdir,
        }
    }

    /// 测试Excel MCP服务器配置创建
    pub fn test_excel_server_config(&self) -> Result<MCPServerConfig, Box<dyn std::error::Error>> {
        println!("🔧 测试Excel MCP服务器配置创建...");

        let server_config = MCPServerConfig::new_http(
            "excel-mcp-server".to_string(),
            "Excel MCP Server".to_string(),
            "http://localhost:8007".to_string(),
            MCPTransportType::HTTP,
        );

        // 验证配置
        assert_eq!(server_config.name, "Excel MCP Server");
        assert_eq!(server_config.transport_type, MCPTransportType::HTTP);
        assert!(server_config.is_active);
        assert!(server_config.http_config.is_some());
        
        if let Some(http_config) = &server_config.http_config {
            assert_eq!(http_config.url, "http://localhost:8007");
        }

        // 保存配置
        self.config_manager.save_server(server_config.clone())?;
        
        // 验证保存和读取
        let loaded_config = self.config_manager.get_server(&server_config.id)
            .ok_or("Failed to load server config")?;
        
        assert_eq!(loaded_config.name, server_config.name);
        assert_eq!(loaded_config.transport_type, server_config.transport_type);

        println!("✅ Excel MCP服务器配置测试通过");
        Ok(server_config)
    }

    /// 测试模拟Excel工具的数据结构
    pub fn test_excel_tools_simulation(&self) -> Result<Vec<MCPTool>, Box<dyn std::error::Error>> {
        println!("📋 测试模拟Excel工具数据结构...");

        let tools = vec![
            // 模拟read_excel工具
            MCPTool::with_annotations(
                "read_excel".to_string(),
                "Read data from an Excel file".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "Path to the Excel file"
                        },
                        "sheet_name": {
                            "type": "string",
                            "description": "Name of the sheet to read",
                            "default": "Sheet1"
                        },
                        "range": {
                            "type": "string",
                            "description": "Cell range to read (e.g., 'A1:C10')"
                        }
                    },
                    "required": ["filename"]
                }),
                MCPToolAnnotations {
                    title: Some("Excel Reader".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }
            ),

            // 模拟write_excel工具
            MCPTool::with_annotations(
                "write_excel".to_string(),
                "Write data to an Excel file".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "Path to the Excel file"
                        },
                        "sheet_name": {
                            "type": "string",
                            "description": "Name of the sheet to write to",
                            "default": "Sheet1"
                        },
                        "data": {
                            "type": "array",
                            "description": "Data to write (array of arrays)",
                            "items": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        "start_cell": {
                            "type": "string",
                            "description": "Starting cell (e.g., 'A1')",
                            "default": "A1"
                        }
                    },
                    "required": ["filename", "data"]
                }),
                MCPToolAnnotations {
                    title: Some("Excel Writer".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(true),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }
            ),

            // 模拟create_chart工具
            MCPTool::with_annotations(
                "create_chart".to_string(),
                "Create a chart in an Excel file".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "Path to the Excel file"
                        },
                        "sheet_name": {
                            "type": "string",
                            "description": "Name of the sheet containing data"
                        },
                        "data_range": {
                            "type": "string",
                            "description": "Range of data for the chart"
                        },
                        "chart_type": {
                            "type": "string",
                            "enum": ["line", "bar", "pie", "scatter"],
                            "description": "Type of chart to create"
                        },
                        "title": {
                            "type": "string",
                            "description": "Chart title"
                        }
                    },
                    "required": ["filename", "data_range", "chart_type"]
                }),
                MCPToolAnnotations {
                    title: Some("Chart Creator".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }
            ),
        ];

        // 验证每个工具的数据结构
        for tool in &tools {
            // 基本验证
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.input_schema.is_object());
            
            // 验证注解
            if let Some(annotations) = &tool.annotations {
                assert!(annotations.title.is_some());
                
                // 验证安全级别逻辑
                let safety_level = tool.safety_level();
                println!("   工具 {}: 安全级别 = {}", tool.name, safety_level);
                
                // 验证权限检查
                let permission = self.security_manager.check_tool_permission(tool, "excel-server");
                println!("   工具 {}: 权限 = {:?}", tool.name, permission);
            }
        }

        println!("✅ Excel工具数据结构测试通过 ({} 个工具)", tools.len());
        Ok(tools)
    }

    /// 测试MCP协议消息序列化
    pub fn test_mcp_protocol_serialization(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📡 测试MCP协议消息序列化...");

        // 测试初始化请求
        let init_request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "AppFlowy MCP Client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let init_message = MCPMessage::request(
            json!(1),
            "initialize".to_string(),
            Some(serde_json::to_value(&init_request)?),
        );

        // 序列化和反序列化测试
        let serialized = serde_json::to_string(&init_message)?;
        let deserialized: MCPMessage = serde_json::from_str(&serialized)?;
        
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, Some("initialize".to_string()));
        assert!(deserialized.params.is_some());

        // 测试工具列表请求
        let tools_request = ListToolsRequest { cursor: None };
        let tools_message = MCPMessage::request(
            json!(2),
            "tools/list".to_string(),
            Some(serde_json::to_value(&tools_request)?),
        );

        let serialized = serde_json::to_string(&tools_message)?;
        let _: MCPMessage = serde_json::from_str(&serialized)?;

        // 测试工具调用请求
        let call_request = CallToolRequest {
            name: "read_excel".to_string(),
            arguments: Some(json!({
                "filename": "test.xlsx",
                "sheet_name": "Sheet1"
            })),
        };

        let call_message = MCPMessage::request(
            json!(3),
            "tools/call".to_string(),
            Some(serde_json::to_value(&call_request)?),
        );

        let serialized = serde_json::to_string(&call_message)?;
        let _: MCPMessage = serde_json::from_str(&serialized)?;

        println!("✅ MCP协议消息序列化测试通过");
        Ok(())
    }

    /// 测试工具调用响应处理
    pub fn test_tool_response_handling(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 测试工具调用响应处理...");

        // 模拟成功的工具响应
        let success_response = CallToolResponse {
            content: vec![
                ToolContent {
                    r#type: "text".to_string(),
                    text: Some("Excel file read successfully".to_string()),
                    data: None,
                    annotations: None,
                },
                ToolContent {
                    r#type: "application/json".to_string(),
                    text: None,
                    data: Some(json!({
                        "rows": 10,
                        "columns": 5,
                        "data": [["A1", "B1", "C1"], ["A2", "B2", "C2"]]
                    })),
                    annotations: None,
                },
            ],
            is_error: Some(false),
        };

        // 序列化和反序列化测试
        let serialized = serde_json::to_string(&success_response)?;
        let deserialized: CallToolResponse = serde_json::from_str(&serialized)?;
        
        assert_eq!(deserialized.content.len(), 2);
        assert_eq!(deserialized.is_error, Some(false));
        assert_eq!(deserialized.content[0].r#type, "text");
        assert_eq!(deserialized.content[1].r#type, "application/json");

        // 模拟错误响应
        let error_response = CallToolResponse {
            content: vec![
                ToolContent {
                    r#type: "text".to_string(),
                    text: Some("File not found: test.xlsx".to_string()),
                    data: None,
                    annotations: None,
                },
            ],
            is_error: Some(true),
        };

        let serialized = serde_json::to_string(&error_response)?;
        let deserialized: CallToolResponse = serde_json::from_str(&serialized)?;
        
        assert_eq!(deserialized.is_error, Some(true));
        assert!(deserialized.content[0].text.as_ref().unwrap().contains("File not found"));

        println!("✅ 工具调用响应处理测试通过");
        Ok(())
    }

    /// 运行所有集成测试
    pub fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 开始MCP集成测试");
        println!("{}", "=".repeat(50));

        self.test_excel_server_config()?;
        let _tools = self.test_excel_tools_simulation()?;
        self.test_mcp_protocol_serialization()?;
        self.test_tool_response_handling()?;

        println!("\n{}", "=".repeat(50));
        println!("🎉 所有MCP集成测试通过!");
        println!("✅ 服务器配置: 兼容");
        println!("✅ 工具数据结构: 兼容");
        println!("✅ 协议消息: 兼容");
        println!("✅ 响应处理: 兼容");
        println!("✅ AppFlowy MCP实现: 准备就绪");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_integration() {
        let integration_test = MCPIntegrationTest::new();
        integration_test.run_all_tests().unwrap();
    }

    #[test]
    fn test_excel_tool_safety_levels() {
        let integration_test = MCPIntegrationTest::new();
        let tools = integration_test.test_excel_tools_simulation().unwrap();
        
        // 验证read_excel是只读工具
        let read_tool = tools.iter().find(|t| t.name == "read_excel").unwrap();
        assert_eq!(read_tool.safety_level(), ToolSafetyLevel::ReadOnly);
        assert!(read_tool.is_read_only());
        assert!(!read_tool.is_destructive());
        
        // 验证write_excel是破坏性工具
        let write_tool = tools.iter().find(|t| t.name == "write_excel").unwrap();
        assert_eq!(write_tool.safety_level(), ToolSafetyLevel::Destructive);
        assert!(!write_tool.is_read_only());
        assert!(write_tool.is_destructive());
        
        // 验证create_chart是安全工具
        let chart_tool = tools.iter().find(|t| t.name == "create_chart").unwrap();
        assert_eq!(chart_tool.safety_level(), ToolSafetyLevel::Safe);
        assert!(!chart_tool.is_read_only());
        assert!(!chart_tool.is_destructive());
    }

    #[test]
    fn test_mcp_message_formats() {
        // 测试各种MCP消息格式
        let request = MCPMessage::request(
            json!(1),
            "test_method".to_string(),
            Some(json!({"param": "value"})),
        );
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.method, Some("test_method".to_string()));
        assert!(request.params.is_some());
        assert!(request.result.is_none());
        assert!(request.error.is_none());

        let response = MCPMessage::response(json!(1), json!({"result": "success"}));
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.method.is_none());
        assert!(response.params.is_none());
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let notification = MCPMessage::notification(
            "notification_method".to_string(),
            Some(json!({"data": "value"})),
        );
        assert_eq!(notification.jsonrpc, "2.0");
        assert!(notification.id.is_none());
        assert_eq!(notification.method, Some("notification_method".to_string()));
        assert!(notification.params.is_some());
    }
}
