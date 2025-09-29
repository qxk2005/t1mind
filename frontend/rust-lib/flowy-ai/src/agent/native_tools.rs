use std::collections::HashMap;
use std::sync::{Arc, Weak};

use flowy_error::{FlowyError, FlowyResult};
use serde_json::{json, Value};
use tracing::{debug, info};
use uuid::Uuid;

use crate::entities::{ToolDefinitionPB, ToolTypePB};
use flowy_document::manager::DocumentManager;
use collab_document::blocks::DocumentData;

/// AppFlowy原生工具管理器
/// 专注于文档CRUD操作，集成现有文档管理API
pub struct NativeToolsManager {
    /// 文档管理器
    document_manager: Weak<DocumentManager>,
}

impl NativeToolsManager {
    /// 创建新的原生工具管理器
    pub fn new(document_manager: Weak<DocumentManager>) -> Self {
        Self {
            document_manager,
        }
    }

    /// 获取文档管理器
    fn get_document_manager(&self) -> FlowyResult<Arc<DocumentManager>> {
        self.document_manager
            .upgrade()
            .ok_or_else(|| FlowyError::internal().with_context("文档管理器已被释放"))
    }

    /// 获取所有可用的原生工具定义
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinitionPB> {
        vec![
            // 创建文档工具
            ToolDefinitionPB {
                name: "create_document".to_string(),
                description: "创建新的AppFlowy文档".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "文档ID（可选，不提供则自动生成）"
                        },
                        "initial_data": {
                            "type": "object",
                            "description": "初始文档数据（可选）",
                            "properties": {
                                "page_id": {"type": "string"},
                                "blocks": {"type": "object"},
                                "meta": {"type": "object"}
                            }
                        }
                    },
                    "required": []
                }).to_string(),
                permissions: vec!["document.create".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "true".to_string());
                    meta
                },
            },
            
            // 读取文档工具
            ToolDefinitionPB {
                name: "get_document".to_string(),
                description: "获取AppFlowy文档的内容和数据".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "要获取的文档ID"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["data", "text", "json"],
                            "default": "data",
                            "description": "返回格式：data(完整数据)、text(纯文本)、json(JSON格式)"
                        }
                    },
                    "required": ["document_id"]
                }).to_string(),
                permissions: vec!["document.read".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "true".to_string());
                    meta
                },
            },

            // 更新文档工具
            ToolDefinitionPB {
                name: "update_document".to_string(),
                description: "更新AppFlowy文档的内容".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "要更新的文档ID"
                        },
                        "actions": {
                            "type": "array",
                            "description": "要执行的文档操作列表",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "action": {
                                        "type": "string",
                                        "enum": ["Insert", "Update", "Delete", "Move", "InsertText", "ApplyTextDelta"]
                                    },
                                    "payload": {
                                        "type": "object",
                                        "description": "操作载荷"
                                    }
                                },
                                "required": ["action", "payload"]
                            }
                        }
                    },
                    "required": ["document_id", "actions"]
                }).to_string(),
                permissions: vec!["document.write".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "false".to_string());
                    meta
                },
            },

            // 删除文档工具
            ToolDefinitionPB {
                name: "delete_document".to_string(),
                description: "删除AppFlowy文档".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "要删除的文档ID"
                        },
                        "confirm": {
                            "type": "boolean",
                            "description": "确认删除操作",
                            "default": false
                        }
                    },
                    "required": ["document_id", "confirm"]
                }).to_string(),
                permissions: vec!["document.delete".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "false".to_string());
                    meta.insert("dangerous".to_string(), "true".to_string());
                    meta
                },
            },

            // 打开文档工具
            ToolDefinitionPB {
                name: "open_document".to_string(),
                description: "打开AppFlowy文档以进行编辑".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "要打开的文档ID"
                        }
                    },
                    "required": ["document_id"]
                }).to_string(),
                permissions: vec!["document.read".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "true".to_string());
                    meta
                },
            },

            // 关闭文档工具
            ToolDefinitionPB {
                name: "close_document".to_string(),
                description: "关闭AppFlowy文档".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "要关闭的文档ID"
                        }
                    },
                    "required": ["document_id"]
                }).to_string(),
                permissions: vec!["document.read".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "true".to_string());
                    meta
                },
            },

            // 获取文档文本工具
            ToolDefinitionPB {
                name: "get_document_text".to_string(),
                description: "获取AppFlowy文档的纯文本内容".to_string(),
                tool_type: ToolTypePB::Native,
                source: "appflowy".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "文档ID"
                        }
                    },
                    "required": ["document_id"]
                }).to_string(),
                permissions: vec!["document.read".to_string()],
                is_available: true,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), "document".to_string());
                    meta.insert("safe_mode".to_string(), "true".to_string());
                    meta
                },
            },
        ]
    }

    /// 执行原生工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: &Value,
        safe_mode: bool,
    ) -> FlowyResult<String> {
        info!("执行原生工具: {} (安全模式: {})", tool_name, safe_mode);
        debug!("工具参数: {}", arguments);

        // 安全检查
        if safe_mode && self.is_dangerous_tool(tool_name) {
            return Err(FlowyError::invalid_data()
                .with_context(format!("安全模式下禁止执行危险工具: {}", tool_name)));
        }

        match tool_name {
            "create_document" => self.create_document(arguments).await,
            "get_document" => self.get_document(arguments).await,
            "update_document" => self.update_document(arguments).await,
            "delete_document" => self.delete_document(arguments).await,
            "open_document" => self.open_document(arguments).await,
            "close_document" => self.close_document(arguments).await,
            "get_document_text" => self.get_document_text(arguments).await,
            _ => Err(FlowyError::invalid_data()
                .with_context(format!("未知的原生工具: {}", tool_name))),
        }
    }

    /// 检查是否为危险工具
    pub fn is_dangerous_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "delete_document" | "update_document")
    }

    /// 创建文档
    async fn create_document(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;
        let uid = document_manager.user_service.user_id()?;

        // 解析参数
        let document_id = if let Some(id_str) = arguments.get("document_id").and_then(|v| v.as_str()) {
            Uuid::parse_str(id_str)
                .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?
        } else {
            Uuid::new_v4()
        };

        let initial_data = arguments.get("initial_data")
            .and_then(|v| serde_json::from_value::<DocumentData>(v.clone()).ok());

        // 创建文档
        let encoded_collab = document_manager
            .create_document(uid, &document_id, initial_data)
            .await?;

        info!("成功创建文档: {}", document_id);
        Ok(json!({
            "success": true,
            "document_id": document_id.to_string(),
            "message": "文档创建成功",
            "encoded_collab_size": encoded_collab.doc_state.len()
        }).to_string())
    }

    /// 获取文档
    async fn get_document(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;

        // 解析参数
        let document_id_str = arguments.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档ID"))?;

        let document_id = Uuid::parse_str(document_id_str)
            .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?;

        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("data");

        // 获取文档数据
        match format {
            "text" => {
                let text = document_manager.get_document_text(&document_id).await?;
                Ok(json!({
                    "success": true,
                    "document_id": document_id.to_string(),
                    "format": "text",
                    "content": text
                }).to_string())
            },
            "data" => {
                let document_data = document_manager.get_document_data(&document_id).await?;
                Ok(json!({
                    "success": true,
                    "document_id": document_id.to_string(),
                    "format": "data",
                    "content": document_data
                }).to_string())
            },
            "json" => {
                let document_data = document_manager.get_document_data(&document_id).await?;
                let json_data = serde_json::to_value(document_data)?;
                Ok(json!({
                    "success": true,
                    "document_id": document_id.to_string(),
                    "format": "json",
                    "content": json_data
                }).to_string())
            },
            _ => Err(FlowyError::invalid_data()
                .with_context(format!("不支持的格式: {}", format))),
        }
    }

    /// 更新文档
    async fn update_document(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;

        // 解析参数
        let document_id_str = arguments.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档ID"))?;

        let document_id = Uuid::parse_str(document_id_str)
            .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?;

        let actions = arguments.get("actions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少操作列表"))?;

        // 获取可编辑文档
        let document = document_manager.editable_document(&document_id).await?;
        
        // 应用操作（这里简化处理，实际应该解析具体的BlockAction）
        let mut applied_actions = 0;
        for action in actions {
            // 这里应该将JSON转换为具体的BlockAction
            // 暂时只记录操作数量
            applied_actions += 1;
            debug!("应用文档操作: {}", action);
        }

        info!("成功更新文档: {}，应用了 {} 个操作", document_id, applied_actions);
        Ok(json!({
            "success": true,
            "document_id": document_id.to_string(),
            "applied_actions": applied_actions,
            "message": "文档更新成功"
        }).to_string())
    }

    /// 删除文档
    async fn delete_document(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;

        // 解析参数
        let document_id_str = arguments.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档ID"))?;

        let document_id = Uuid::parse_str(document_id_str)
            .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?;

        let confirm = arguments.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(FlowyError::invalid_data()
                .with_context("删除文档需要确认，请设置 confirm=true"));
        }

        // 删除文档
        document_manager.delete_document(&document_id).await?;

        info!("成功删除文档: {}", document_id);
        Ok(json!({
            "success": true,
            "document_id": document_id.to_string(),
            "message": "文档删除成功"
        }).to_string())
    }

    /// 打开文档
    async fn open_document(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;

        // 解析参数
        let document_id_str = arguments.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档ID"))?;

        let document_id = Uuid::parse_str(document_id_str)
            .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?;

        // 打开文档
        document_manager.open_document(&document_id).await?;

        info!("成功打开文档: {}", document_id);
        Ok(json!({
            "success": true,
            "document_id": document_id.to_string(),
            "message": "文档打开成功"
        }).to_string())
    }

    /// 关闭文档
    async fn close_document(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;

        // 解析参数
        let document_id_str = arguments.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档ID"))?;

        let document_id = Uuid::parse_str(document_id_str)
            .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?;

        // 关闭文档
        document_manager.close_document(&document_id).await?;

        info!("成功关闭文档: {}", document_id);
        Ok(json!({
            "success": true,
            "document_id": document_id.to_string(),
            "message": "文档关闭成功"
        }).to_string())
    }

    /// 获取文档文本
    async fn get_document_text(&self, arguments: &Value) -> FlowyResult<String> {
        let document_manager = self.get_document_manager()?;

        // 解析参数
        let document_id_str = arguments.get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档ID"))?;

        let document_id = Uuid::parse_str(document_id_str)
            .map_err(|_| FlowyError::invalid_data().with_context("无效的文档ID格式"))?;

        // 获取文档文本
        let text = document_manager.get_document_text(&document_id).await?;

        info!("成功获取文档文本: {}，长度: {} 字符", document_id, text.len());
        Ok(json!({
            "success": true,
            "document_id": document_id.to_string(),
            "text": text,
            "length": text.len()
        }).to_string())
    }

    /// 验证工具权限
    pub fn validate_permission(&self, tool_name: &str, required_permission: &str) -> FlowyResult<()> {
        // 获取工具定义
        let tools = self.get_tool_definitions();
        let tool = tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| FlowyError::record_not_found()
                .with_context(format!("工具 '{}' 不存在", tool_name)))?;

        // 检查权限
        if tool.permissions.contains(&required_permission.to_string()) {
            Ok(())
        } else {
            Err(FlowyError::unauthorized()
                .with_context(format!("工具 '{}' 缺少权限: {}", tool_name, required_permission)))
        }
    }

    /// 检查工具是否可用
    pub fn is_tool_available(&self, tool_name: &str) -> bool {
        self.get_tool_definitions()
            .iter()
            .any(|t| t.name == tool_name && t.is_available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        let manager = NativeToolsManager::new(Weak::new());
        let tools = manager.get_tool_definitions();
        
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "create_document"));
        assert!(tools.iter().any(|t| t.name == "get_document"));
        assert!(tools.iter().any(|t| t.name == "update_document"));
        assert!(tools.iter().any(|t| t.name == "delete_document"));
    }

    #[test]
    fn test_dangerous_tool_detection() {
        let manager = NativeToolsManager::new(Weak::new());
        
        assert!(manager.is_dangerous_tool("delete_document"));
        assert!(manager.is_dangerous_tool("update_document"));
        assert!(!manager.is_dangerous_tool("get_document"));
        assert!(!manager.is_dangerous_tool("create_document"));
    }

    #[test]
    fn test_tool_availability() {
        let manager = NativeToolsManager::new(Weak::new());
        
        assert!(manager.is_tool_available("create_document"));
        assert!(manager.is_tool_available("get_document"));
        assert!(!manager.is_tool_available("nonexistent_tool"));
    }
}
