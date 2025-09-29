#[cfg(test)]
mod simple_tests {
    use crate::agent::{ToolRegistry, ToolRegistrationRequest, ToolStatus};
    use crate::entities::{ToolDefinitionPB, ToolTypePB};
    use crate::mcp::tool_security::ToolSecurityManager;
    use flowy_sqlite::kv::KVStorePreferences;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_tool_registry_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store_preferences = Arc::new(KVStorePreferences::new(temp_dir.path().to_str().unwrap()).unwrap());
        let security_manager = Arc::new(ToolSecurityManager::new(store_preferences.clone()));
        let _registry = ToolRegistry::new(security_manager, store_preferences);
        
        // 如果能创建成功，说明基本结构是正确的
        assert!(true);
    }

    #[test]
    fn test_tool_definition_creation() {
        let tool_def = ToolDefinitionPB {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            tool_type: ToolTypePB::Native,
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
        };
        
        assert_eq!(tool_def.name, "test_tool");
        assert_eq!(tool_def.tool_type, ToolTypePB::Native);
        assert!(tool_def.is_available);
    }

    #[test]
    fn test_tool_registration_request_creation() {
        let tool_def = ToolDefinitionPB {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            tool_type: ToolTypePB::Native,
            source: "test".to_string(),
            parameters_schema: "{}".to_string(),
            permissions: vec![],
            is_available: true,
            metadata: HashMap::new(),
        };
        
        let request = ToolRegistrationRequest {
            definition: tool_def.clone(),
            config: None,
            dependencies: Vec::new(),
            overwrite: false,
        };
        
        assert_eq!(request.definition.name, "test_tool");
        assert!(!request.overwrite);
        assert!(request.dependencies.is_empty());
    }

    #[test]
    fn test_tool_status_enum() {
        let status = ToolStatus::Available;
        assert_eq!(status, ToolStatus::Available);
        
        let status2 = ToolStatus::Maintenance;
        assert_ne!(status, status2);
    }
}
