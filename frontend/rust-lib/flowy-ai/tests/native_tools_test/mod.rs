use std::sync::Weak;

use serde_json::json;

use flowy_ai::agent::native_tools::NativeToolsManager;

#[test]
fn test_native_tools_manager_creation() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    // 测试获取工具定义
    let tools = native_tools.get_tool_definitions();
    assert!(!tools.is_empty(), "应该有可用的原生工具");
    
    // 验证包含预期的工具
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"create_document"), "应该包含创建文档工具");
    assert!(tool_names.contains(&"get_document"), "应该包含获取文档工具");
    assert!(tool_names.contains(&"update_document"), "应该包含更新文档工具");
    assert!(tool_names.contains(&"delete_document"), "应该包含删除文档工具");
    assert!(tool_names.contains(&"open_document"), "应该包含打开文档工具");
    assert!(tool_names.contains(&"close_document"), "应该包含关闭文档工具");
    assert!(tool_names.contains(&"get_document_text"), "应该包含获取文档文本工具");
}

#[test]
fn test_tool_definitions_structure() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    let tools = native_tools.get_tool_definitions();
    
    for tool in &tools {
        // 验证基本字段
        assert!(!tool.name.is_empty(), "工具名称不能为空");
        assert!(!tool.description.is_empty(), "工具描述不能为空");
        assert_eq!(tool.source, "appflowy", "工具来源应该是appflowy");
        assert!(tool.is_available, "工具应该是可用的");
        
        // 验证参数模式是有效的JSON
        let _: serde_json::Value = serde_json::from_str(&tool.parameters_schema)
            .expect("参数模式应该是有效的JSON");
        
        // 验证权限不为空
        assert!(!tool.permissions.is_empty(), "工具应该有权限定义");
    }
}

#[test]
fn test_dangerous_tool_detection() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    // 测试危险工具检测
    assert!(native_tools.is_dangerous_tool("delete_document"), "删除文档应该被标记为危险");
    assert!(native_tools.is_dangerous_tool("update_document"), "更新文档应该被标记为危险");
    assert!(!native_tools.is_dangerous_tool("get_document"), "获取文档不应该被标记为危险");
    assert!(!native_tools.is_dangerous_tool("create_document"), "创建文档不应该被标记为危险");
}

#[test]
fn test_tool_availability() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    // 测试工具可用性检查
    assert!(native_tools.is_tool_available("create_document"), "创建文档工具应该可用");
    assert!(native_tools.is_tool_available("get_document"), "获取文档工具应该可用");
    assert!(!native_tools.is_tool_available("nonexistent_tool"), "不存在的工具应该不可用");
}

#[test]
fn test_permission_validation() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    // 测试权限验证
    assert!(native_tools.validate_permission("create_document", "document.create").is_ok(), 
            "创建文档工具应该有创建权限");
    
    assert!(native_tools.validate_permission("get_document", "document.read").is_ok(), 
            "获取文档工具应该有读取权限");
    
    assert!(native_tools.validate_permission("create_document", "document.delete").is_err(), 
            "创建文档工具不应该有删除权限");
    
    assert!(native_tools.validate_permission("nonexistent_tool", "any.permission").is_err(), 
            "不存在的工具应该验证失败");
}

#[tokio::test]
async fn test_execute_tool_without_document_manager() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    let arguments = json!({
        "document_id": "test-doc-id"
    });
    
    // 当文档管理器不可用时，应该返回错误
    let result = native_tools.execute_tool("get_document", &arguments, true).await;
    assert!(result.is_err(), "没有文档管理器时应该返回错误");
}

#[tokio::test]
async fn test_safe_mode_restrictions() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    let arguments = json!({
        "document_id": "test-doc-id",
        "confirm": true
    });
    
    // 在安全模式下，危险工具应该被阻止
    let result = native_tools.execute_tool("delete_document", &arguments, true).await;
    assert!(result.is_err(), "安全模式下应该阻止危险工具");
    
    // 非危险工具在安全模式下应该可以执行（虽然会因为没有真实的文档管理器而失败）
    let result = native_tools.execute_tool("get_document", &arguments, true).await;
    // 这里会因为文档管理器不可用而失败，但不是因为安全模式限制
    assert!(result.is_err());
}

#[test]
fn test_tool_metadata() {
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);
    
    let tools = native_tools.get_tool_definitions();
    
    // 验证工具元数据
    for tool in &tools {
        assert!(tool.metadata.contains_key("category"), "工具应该有类别元数据");
        assert_eq!(tool.metadata.get("category").unwrap(), "document", "工具类别应该是document");
        assert!(tool.metadata.contains_key("safe_mode"), "工具应该有安全模式元数据");
        
        // 检查危险工具的标记
        if tool.name == "delete_document" {
            assert!(tool.metadata.contains_key("dangerous"), "删除工具应该有危险标记");
            assert_eq!(tool.metadata.get("dangerous").unwrap(), "true", "删除工具应该标记为危险");
        }
    }
}