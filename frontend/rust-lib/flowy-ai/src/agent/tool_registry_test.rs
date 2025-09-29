#[cfg(test)]
mod tests {
    use crate::agent::{ToolRegistry, ToolRegistrationRequest, ToolStatus, ToolSearchFilter};
    use crate::entities::{ToolDefinitionPB, ToolTypePB};
    use crate::mcp::tool_security::ToolSecurityManager;
    use flowy_sqlite::kv::KVStorePreferences;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_registry() -> (ToolRegistry, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        let store_preferences = Arc::new(KVStorePreferences::new(temp_dir.path().to_str().unwrap()).unwrap());
        let security_manager = Arc::new(ToolSecurityManager::new(store_preferences.clone()));
        let registry = ToolRegistry::new(security_manager, store_preferences);
        
        (registry, temp_dir)
    }

    fn create_test_tool_definition(name: &str, tool_type: ToolTypePB) -> ToolDefinitionPB {
        ToolDefinitionPB {
            name: name.to_string(),
            description: format!("Test tool: {}", name),
            tool_type,
            source: "test".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string", "description": "Test input"}
                },
                "required": ["input"]
            }).to_string(),
            permissions: vec!["test.execute".to_string()],
            is_available: true,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_tool_registry_initialization() {
        let (registry, _temp_dir) = create_test_registry().await;
        
        let result = registry.initialize().await;
        assert!(result.is_ok(), "Registry initialization should succeed");
        
        // 检查内置工具是否已注册
        let native_tools = registry.get_tools_by_type(ToolTypePB::Native).await;
        assert!(!native_tools.is_empty(), "Should have native tools registered");
        
        let search_tools = registry.get_tools_by_type(ToolTypePB::Search).await;
        assert!(!search_tools.is_empty(), "Should have search tools registered");
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        let tool_def = create_test_tool_definition("test_tool", ToolTypePB::Native);
        let request = ToolRegistrationRequest {
            definition: tool_def.clone(),
            config: None,
            dependencies: Vec::new(),
            overwrite: false,
        };
        
        let result = registry.register_tool(request).await;
        assert!(result.is_ok(), "Tool registration should succeed");
        
        // 验证工具已注册
        let registered_tool = registry.get_tool("test_tool", ToolTypePB::Native).await;
        assert!(registered_tool.is_some(), "Tool should be registered");
        
        let tool = registered_tool.unwrap();
        assert_eq!(tool.definition.name, "test_tool");
        assert_eq!(tool.definition.tool_type, ToolTypePB::Native);
        assert_eq!(tool.status, ToolStatus::Available);
    }

    #[tokio::test]
    async fn test_tool_search() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        // 注册测试工具
        let tools = vec![
            ("search_tool", ToolTypePB::Search),
            ("native_tool", ToolTypePB::Native),
            ("mcp_tool", ToolTypePB::MCP),
        ];
        
        for (name, tool_type) in tools {
            let tool_def = create_test_tool_definition(name, tool_type);
            let request = ToolRegistrationRequest {
                definition: tool_def,
                config: None,
                dependencies: Vec::new(),
                overwrite: false,
            };
            registry.register_tool(request).await.unwrap();
        }
        
        // 测试搜索
        let results = registry.search_tools("search", None).await;
        assert!(!results.is_empty(), "Should find tools matching 'search'");
        
        // 测试类型过滤
        let filter = ToolSearchFilter {
            tool_types: Some(vec![ToolTypePB::Native]),
            ..Default::default()
        };
        let filtered_results = registry.search_tools("", Some(filter)).await;
        
        // 应该包含内置工具和我们注册的native_tool
        let native_count = filtered_results.iter()
            .filter(|t| t.definition.tool_type == ToolTypePB::Native)
            .count();
        assert!(native_count > 0, "Should find native tools");
    }

    #[tokio::test]
    async fn test_tool_status_management() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        let tool_def = create_test_tool_definition("status_test_tool", ToolTypePB::Native);
        let request = ToolRegistrationRequest {
            definition: tool_def,
            config: None,
            dependencies: Vec::new(),
            overwrite: false,
        };
        registry.register_tool(request).await.unwrap();
        
        // 测试状态更新
        let result = registry.update_tool_status(
            "status_test_tool",
            ToolTypePB::Native,
            ToolStatus::Maintenance,
        ).await;
        assert!(result.is_ok(), "Status update should succeed");
        
        // 验证状态已更新
        let tool = registry.get_tool("status_test_tool", ToolTypePB::Native).await.unwrap();
        assert_eq!(tool.status, ToolStatus::Maintenance);
    }

    #[tokio::test]
    async fn test_tool_usage_statistics() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        let tool_def = create_test_tool_definition("stats_test_tool", ToolTypePB::Native);
        let request = ToolRegistrationRequest {
            definition: tool_def,
            config: None,
            dependencies: Vec::new(),
            overwrite: false,
        };
        registry.register_tool(request).await.unwrap();
        
        // 更新使用统计
        registry.update_tool_usage("stats_test_tool", ToolTypePB::Native, 100, true).await.unwrap();
        registry.update_tool_usage("stats_test_tool", ToolTypePB::Native, 200, false).await.unwrap();
        
        // 验证统计信息
        let tool = registry.get_tool("stats_test_tool", ToolTypePB::Native).await.unwrap();
        assert_eq!(tool.usage_stats.total_calls, 2);
        assert_eq!(tool.usage_stats.successful_calls, 1);
        assert_eq!(tool.usage_stats.failed_calls, 1);
        assert_eq!(tool.usage_stats.avg_execution_time_ms, 150.0);
    }

    #[tokio::test]
    async fn test_tool_unregistration() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        let tool_def = create_test_tool_definition("remove_test_tool", ToolTypePB::Native);
        let request = ToolRegistrationRequest {
            definition: tool_def,
            config: None,
            dependencies: Vec::new(),
            overwrite: false,
        };
        registry.register_tool(request).await.unwrap();
        
        // 验证工具存在
        assert!(registry.get_tool("remove_test_tool", ToolTypePB::Native).await.is_some());
        
        // 注销工具
        let result = registry.unregister_tool("remove_test_tool", ToolTypePB::Native).await;
        assert!(result.is_ok(), "Tool unregistration should succeed");
        
        // 验证工具已移除
        assert!(registry.get_tool("remove_test_tool", ToolTypePB::Native).await.is_none());
    }

    #[tokio::test]
    async fn test_registry_statistics() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        let stats = registry.get_tool_statistics().await;
        
        // 应该有内置工具
        assert!(stats.total_tools > 0, "Should have some tools registered");
        assert!(stats.native_tools > 0, "Should have native tools");
        assert!(stats.search_tools > 0, "Should have search tools");
        assert!(stats.available_tools > 0, "Should have available tools");
    }

    #[tokio::test]
    async fn test_export_import_registry() {
        let (registry, _temp_dir) = create_test_registry().await;
        registry.initialize().await.unwrap();
        
        // 添加自定义工具
        let tool_def = create_test_tool_definition("export_test_tool", ToolTypePB::Native);
        let request = ToolRegistrationRequest {
            definition: tool_def,
            config: None,
            dependencies: Vec::new(),
            overwrite: false,
        };
        registry.register_tool(request).await.unwrap();
        
        // 导出注册表
        let export_data = registry.export_registry().await.unwrap();
        assert!(!export_data.is_empty(), "Export data should not be empty");
        
        // 创建新的注册表并导入
        let (new_registry, _temp_dir2) = create_test_registry().await;
        let result = new_registry.import_registry(&export_data, false).await;
        assert!(result.is_ok(), "Import should succeed");
        
        // 验证导入的工具
        let imported_tool = new_registry.get_tool("export_test_tool", ToolTypePB::Native).await;
        assert!(imported_tool.is_some(), "Imported tool should exist");
    }
}
