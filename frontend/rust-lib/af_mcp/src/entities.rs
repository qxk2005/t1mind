use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchemaProperty {
  pub r#type: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
  pub name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  // Accept both `input` and `inputSchema` from different MCP servers
  #[serde(alias = "inputSchema", default)]
  pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
  pub tools: Vec<ToolSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResponse {
  pub jsonrpc: String,
  pub id: serde_json::Value,
  pub result: ToolsListResult,
}

pub type ToolsList = ToolsListResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvokeResponse {
  pub jsonrpc: String,
  pub id: serde_json::Value,
  pub result: serde_json::Value,
}


