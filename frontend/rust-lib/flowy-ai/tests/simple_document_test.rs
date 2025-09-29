use std::sync::Weak;
use uuid::Uuid;
use serde_json::json;

use flowy_ai::agent::native_tools::NativeToolsManager;

/// 简单的文档创建测试
/// 这个测试专注于验证工具接口和参数处理，而不需要真实的文档管理器
#[tokio::test]
async fn test_document_creation_tool_interface() {
    println!("🚀 开始测试AppFlowy文档创建工具接口");

    // 创建原生工具管理器（使用空的文档管理器引用）
    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);

    println!("📋 获取可用工具列表");
    let tools = native_tools.get_tool_definitions();
    println!("✅ 发现 {} 个可用工具", tools.len());
    
    // 打印所有可用工具
    for tool in &tools {
        println!("  🔧 {}: {}", tool.name, tool.description);
    }

    // 验证创建文档工具存在
    let create_tool = tools.iter().find(|t| t.name == "create_document");
    assert!(create_tool.is_some(), "应该包含创建文档工具");
    let create_tool = create_tool.unwrap();
    println!("✅ 找到创建文档工具: {}", create_tool.description);

    // 验证工具的参数模式
    println!("📋 验证工具参数模式");
    let schema: serde_json::Value = serde_json::from_str(&create_tool.parameters_schema)
        .expect("参数模式应该是有效的JSON");
    
    println!("📄 创建文档工具的参数模式:");
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    // 验证基本结构
    assert_eq!(schema["type"], "object", "参数模式应该是对象类型");
    assert!(schema.get("properties").is_some(), "应该有properties字段");

    // 测试1: 使用最小参数创建文档
    println!("\n📝 测试1: 使用最小参数创建文档");
    let document_id = Uuid::new_v4();
    let minimal_args = json!({
        "document_id": document_id.to_string()
    });

    println!("📋 参数: {}", serde_json::to_string_pretty(&minimal_args).unwrap());
    let result = native_tools.execute_tool("create_document", &minimal_args, false).await;
    
    match result {
        Ok(response) => {
            println!("✅ 工具执行成功（意外）: {}", response);
        }
        Err(e) => {
            println!("❌ 工具执行失败（预期）: {}", e);
            // 验证这是因为缺少文档管理器而不是参数问题
            assert!(e.to_string().contains("文档管理器已被释放") || 
                   e.to_string().contains("dropped") ||
                   e.to_string().contains("已被释放"), 
                   "应该是因为文档管理器不可用而失败");
            println!("✅ 错误原因正确：缺少文档管理器");
        }
    }

    // 测试2: 使用完整参数创建文档
    println!("\n📝 测试2: 使用完整参数创建文档");
    let document_id2 = Uuid::new_v4();
    let full_args = json!({
        "document_id": document_id2.to_string(),
        "initial_data": {
            "page_id": document_id2.to_string(),
            "blocks": {
                "root": {
                    "id": "root",
                    "type": "page",
                    "data": {},
                    "parent_id": "",
                    "children_id": "children_1"
                }
            },
            "meta": {
                "children_map": {
                    "children_1": {
                        "children": ["root"]
                    }
                },
                "text_map": {}
            }
        }
    });

    println!("📋 参数: {}", serde_json::to_string_pretty(&full_args).unwrap());
    let result2 = native_tools.execute_tool("create_document", &full_args, false).await;
    
    match result2 {
        Ok(response) => {
            println!("✅ 工具执行成功（意外）: {}", response);
        }
        Err(e) => {
            println!("❌ 工具执行失败（预期）: {}", e);
            assert!(e.to_string().contains("文档管理器已被释放") || 
                   e.to_string().contains("dropped") ||
                   e.to_string().contains("已被释放"), 
                   "应该是因为文档管理器不可用而失败");
            println!("✅ 错误原因正确：缺少文档管理器");
        }
    }

    // 测试3: 测试安全模式
    println!("\n🔒 测试3: 安全模式下的工具执行");
    let result3 = native_tools.execute_tool("create_document", &minimal_args, true).await;
    
    match result3 {
        Ok(_) => {
            println!("✅ 创建文档在安全模式下被允许（正确）");
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("安全模式") {
                panic!("❌ 创建文档不应该在安全模式下被阻止");
            } else {
                println!("✅ 因其他原因失败（文档管理器不可用）: {}", e);
            }
        }
    }

    println!("\n✅ 所有接口测试完成");
}

#[tokio::test]
async fn test_all_document_tools() {
    println!("🚀 测试所有文档工具的接口");

    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);

    let document_id = Uuid::new_v4();
    
    // 测试所有工具
    let test_cases = vec![
        ("create_document", json!({"document_id": document_id.to_string()}), false),
        ("get_document", json!({"document_id": document_id.to_string(), "format": "text"}), false),
        ("update_document", json!({"document_id": document_id.to_string(), "actions": []}), true), // 危险工具
        ("delete_document", json!({"document_id": document_id.to_string(), "confirm": true}), true), // 危险工具
        ("open_document", json!({"document_id": document_id.to_string()}), false),
        ("close_document", json!({"document_id": document_id.to_string()}), false),
        ("get_document_text", json!({"document_id": document_id.to_string()}), false),
    ];

    for (tool_name, args, is_dangerous) in test_cases {
        println!("\n🔧 测试工具: {}", tool_name);
        
        // 测试正常模式
        let result = native_tools.execute_tool(tool_name, &args, false).await;
        match result {
            Ok(_) => println!("  ✅ 正常模式: 成功（意外）"),
            Err(e) => {
                if e.to_string().contains("文档管理器已被释放") || 
                   e.to_string().contains("dropped") ||
                   e.to_string().contains("已被释放") {
                    println!("  ✅ 正常模式: 因缺少文档管理器而失败（预期）");
                } else {
                    println!("  ❌ 正常模式: 其他错误 - {}", e);
                }
            }
        }

        // 测试安全模式
        let safe_result = native_tools.execute_tool(tool_name, &args, true).await;
        match safe_result {
            Ok(_) => {
                if is_dangerous {
                    panic!("❌ 危险工具 '{}' 不应该在安全模式下成功", tool_name);
                } else {
                    println!("  ✅ 安全模式: 成功（意外，但工具不危险）");
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("安全模式") && is_dangerous {
                    println!("  ✅ 安全模式: 危险工具被正确阻止");
                } else if error_msg.contains("文档管理器已被释放") || 
                         error_msg.contains("dropped") ||
                         error_msg.contains("已被释放") {
                    println!("  ✅ 安全模式: 因缺少文档管理器而失败（预期）");
                } else {
                    println!("  ❌ 安全模式: 其他错误 - {}", e);
                }
            }
        }
    }

    println!("\n✅ 所有工具接口测试完成");
}

#[test]
fn test_tool_metadata_and_permissions() {
    println!("🔐 测试工具元数据和权限");

    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);

    let tools = native_tools.get_tool_definitions();

    for tool in &tools {
        println!("\n🔧 工具: {}", tool.name);
        println!("  📝 描述: {}", tool.description);
        println!("  🏷️  来源: {}", tool.source);
        println!("  ✅ 可用: {}", tool.is_available);
        
        // 验证权限
        println!("  🔐 权限: {:?}", tool.permissions);
        assert!(!tool.permissions.is_empty(), "工具应该有权限定义");
        
        // 验证元数据
        println!("  📊 元数据: {:?}", tool.metadata);
        assert!(tool.metadata.contains_key("category"), "应该有类别元数据");
        assert_eq!(tool.metadata.get("category").unwrap(), "document", "类别应该是document");
        
        // 验证参数模式
        let schema: serde_json::Value = serde_json::from_str(&tool.parameters_schema)
            .expect("参数模式应该是有效JSON");
        assert_eq!(schema["type"], "object", "参数应该是对象类型");
        
        // 验证危险工具标记
        let is_marked_dangerous = tool.metadata.get("dangerous").map(|v| v == "true").unwrap_or(false);
        let is_actually_dangerous = native_tools.is_dangerous_tool(&tool.name);
        
        if is_actually_dangerous {
            assert!(is_marked_dangerous, "危险工具 '{}' 应该在元数据中标记为危险", tool.name);
            println!("  ⚠️  危险工具已正确标记");
        } else {
            println!("  ✅ 安全工具");
        }
    }

    println!("\n✅ 元数据和权限验证完成");
}

#[tokio::test]
async fn test_parameter_validation() {
    println!("📋 测试参数验证");

    let weak_doc_manager = Weak::new();
    let native_tools = NativeToolsManager::new(weak_doc_manager);

    // 测试无效的工具名称
    let invalid_tool_result = native_tools.execute_tool("nonexistent_tool", &json!({}), false).await;
    assert!(invalid_tool_result.is_err(), "不存在的工具应该返回错误");
    println!("✅ 无效工具名称被正确拒绝");

    // 测试权限验证
    assert!(native_tools.validate_permission("create_document", "document.create").is_ok());
    assert!(native_tools.validate_permission("create_document", "document.delete").is_err());
    assert!(native_tools.validate_permission("nonexistent_tool", "any.permission").is_err());
    println!("✅ 权限验证正常工作");

    // 测试工具可用性
    assert!(native_tools.is_tool_available("create_document"));
    assert!(!native_tools.is_tool_available("nonexistent_tool"));
    println!("✅ 工具可用性检查正常工作");

    println!("\n✅ 参数验证测试完成");
}
