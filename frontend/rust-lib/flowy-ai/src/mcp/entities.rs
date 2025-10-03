use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// MCP传输方式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MCPTransportType {
    /// 标准输入输出传输
    Stdio,
    /// Server-Sent Events传输
    SSE,
    /// HTTP传输
    HTTP,
}

impl std::fmt::Display for MCPTransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCPTransportType::Stdio => write!(f, "STDIO"),
            MCPTransportType::SSE => write!(f, "SSE"),
            MCPTransportType::HTTP => write!(f, "HTTP"),
        }
    }
}

/// MCP服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub transport_type: MCPTransportType,
    pub is_active: bool,
    pub description: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub stdio_config: Option<MCPStdioConfig>,
    pub http_config: Option<MCPHttpConfig>,
    /// 缓存的工具列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tools: Option<Vec<MCPTool>>,
    /// 最后检查工具的时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tools_check_at: Option<SystemTime>,
}

/// STDIO传输配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPStdioConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
}

/// HTTP/SSE传输配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPHttpConfig {
    pub url: String,
    pub headers: HashMap<String, String>,
}

/// MCP工具定义 - 符合MCP标准规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    /// 工具的唯一标识符
    pub name: String,
    /// 工具功能的描述（可选，某些MCP服务器可能不提供）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 工具输入参数的JSON Schema定义
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    /// 工具行为的注解元数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MCPToolAnnotations>,
}

/// MCP工具注解 - 描述工具行为的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolAnnotations {
    /// 工具的可读标题，适用于UI显示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 指示工具是否为只读操作
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    /// 指示工具是否可能执行破坏性操作
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    /// 指示工具的操作是否为幂等的
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    /// 指示工具是否可能与外部实体交互
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// MCP工具列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsList {
    pub tools: Vec<MCPTool>,
}

/// MCP工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: Value,
}

/// MCP工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub content: Vec<ToolCallContent>,
    pub is_error: bool,
}

/// 工具调用内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallContent {
    pub r#type: String,
    pub text: Option<String>,
    pub data: Option<Value>,
}

/// MCP连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MCPConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// MCP客户端信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPClientInfo {
    pub server_id: String,
    pub status: MCPConnectionStatus,
    pub tools: Vec<MCPTool>,
    pub last_connected: Option<SystemTime>,
    pub error_message: Option<String>,
}

impl MCPTool {
    /// 创建一个基本的MCP工具
    pub fn new(name: String, description: Option<String>, input_schema: Value) -> Self {
        Self {
            name,
            description,
            input_schema,
            annotations: None,
        }
    }

    /// 创建带有注解的MCP工具
    pub fn with_annotations(
        name: String,
        description: Option<String>,
        input_schema: Value,
        annotations: MCPToolAnnotations,
    ) -> Self {
        Self {
            name,
            description,
            input_schema,
            annotations: Some(annotations),
        }
    }

    /// 检查工具是否为只读操作
    pub fn is_read_only(&self) -> bool {
        self.annotations
            .as_ref()
            .and_then(|a| a.read_only_hint)
            .unwrap_or(false)
    }

    /// 检查工具是否为破坏性操作
    pub fn is_destructive(&self) -> bool {
        self.annotations
            .as_ref()
            .and_then(|a| a.destructive_hint)
            .unwrap_or(false)
    }

    /// 检查工具是否为幂等操作
    pub fn is_idempotent(&self) -> bool {
        self.annotations
            .as_ref()
            .and_then(|a| a.idempotent_hint)
            .unwrap_or(false)
    }

    /// 检查工具是否与外部世界交互
    pub fn interacts_with_external_world(&self) -> bool {
        self.annotations
            .as_ref()
            .and_then(|a| a.open_world_hint)
            .unwrap_or(false)
    }

    /// 获取工具的显示标题
    pub fn display_title(&self) -> &str {
        self.annotations
            .as_ref()
            .and_then(|a| a.title.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(&self.name)
    }

    /// 获取工具的安全级别描述
    pub fn safety_level(&self) -> ToolSafetyLevel {
        if self.is_destructive() {
            ToolSafetyLevel::Destructive
        } else if self.interacts_with_external_world() {
            ToolSafetyLevel::ExternalInteraction
        } else if self.is_read_only() {
            ToolSafetyLevel::ReadOnly
        } else {
            ToolSafetyLevel::Safe
        }
    }
}

impl MCPToolAnnotations {
    /// 创建默认的安全工具注解
    pub fn safe_tool() -> Self {
        Self {
            title: None,
            read_only_hint: Some(true),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: Some(false),
        }
    }

    /// 创建破坏性工具注解
    pub fn destructive_tool() -> Self {
        Self {
            title: None,
            read_only_hint: Some(false),
            destructive_hint: Some(true),
            idempotent_hint: Some(false),
            open_world_hint: None,
        }
    }

    /// 创建外部交互工具注解
    pub fn external_tool() -> Self {
        Self {
            title: None,
            read_only_hint: None,
            destructive_hint: None,
            idempotent_hint: None,
            open_world_hint: Some(true),
        }
    }
}

/// 工具安全级别枚举
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolSafetyLevel {
    /// 只读安全工具
    ReadOnly,
    /// 一般安全工具
    Safe,
    /// 与外部世界交互的工具
    ExternalInteraction,
    /// 破坏性工具
    Destructive,
}

impl std::fmt::Display for ToolSafetyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolSafetyLevel::ReadOnly => write!(f, "只读"),
            ToolSafetyLevel::Safe => write!(f, "安全"),
            ToolSafetyLevel::ExternalInteraction => write!(f, "外部交互"),
            ToolSafetyLevel::Destructive => write!(f, "破坏性"),
        }
    }
}

impl MCPServerConfig {
    pub fn new_stdio(
        id: String,
        name: String,
        command: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            icon: "terminal".to_string(),
            transport_type: MCPTransportType::Stdio,
            is_active: true,
            description: format!("STDIO MCP server: {}", command),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            stdio_config: Some(MCPStdioConfig {
                command,
                args,
                env_vars: HashMap::new(),
            }),
            http_config: None,
            cached_tools: None,
            last_tools_check_at: None,
        }
    }

    pub fn new_http(
        id: String,
        name: String,
        url: String,
        transport_type: MCPTransportType,
    ) -> Self {
        Self {
            id,
            name,
            icon: "web".to_string(),
            transport_type: transport_type.clone(),
            is_active: true,
            description: format!("{:?} MCP server: {}", transport_type, url),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            stdio_config: None,
            http_config: Some(MCPHttpConfig {
                url,
                headers: HashMap::new(),
            }),
            cached_tools: None,
            last_tools_check_at: None,
        }
    }

    pub fn server_cmd(&self) -> String {
        match &self.transport_type {
            MCPTransportType::Stdio => {
                if let Some(stdio) = &self.stdio_config {
                    stdio.command.clone()
                } else {
                    self.id.clone()
                }
            }
            MCPTransportType::SSE | MCPTransportType::HTTP => {
                if let Some(http) = &self.http_config {
                    http.url.clone()
                } else {
                    self.id.clone()
                }
            }
        }
    }
}
