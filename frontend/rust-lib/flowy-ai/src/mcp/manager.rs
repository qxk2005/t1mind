use dashmap::DashMap;
use flowy_error::{FlowyError, FlowyResult, ErrorCode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, trace};

#[cfg(feature = "mcp_stdio")]
use af_mcp::client::{MCPClient, MCPServerConfig};
#[cfg(feature = "mcp_stdio")]
use af_mcp::entities::{ToolsList, ToolInvokeResponse};
use super::sse::{SseClient, SseConfig};

pub struct MCPClientManager {
  #[cfg(feature = "mcp_stdio")]
  stdio_clients: Arc<DashMap<String, MCPClient>>,
  sse_clients: Arc<DashMap<String, SseClient>>,
  concurrency: Arc<Semaphore>,
}

impl MCPClientManager {
  pub fn new() -> MCPClientManager {
    Self {
      #[cfg(feature = "mcp_stdio")]
      stdio_clients: Arc::new(DashMap::new()),
      sse_clients: Arc::new(DashMap::new()),
      concurrency: Arc::new(Semaphore::new(3)),
    }
  }

  #[cfg(feature = "mcp_stdio")]
  pub async fn connect_stdio(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
    let mut client = MCPClient::new_stdio(config.clone())
      .await
      .map_err(|e| FlowyError::internal().with_context(format!("mcp stdio spawn: {}", e)))?;
    client
      .initialize()
      .await
      .map_err(|e| FlowyError::internal().with_context(format!("mcp stdio init: {}", e)))?;
    self
      .stdio_clients
      .insert(config.server_cmd.clone(), client);
    Ok(())
  }

  pub async fn connect_sse(&self, id: String, url: String, headers: Vec<(String, String)>) -> FlowyResult<()> {
    let client = SseClient::new(SseConfig { url, headers });
    self.sse_clients.insert(id, client);
    Ok(())
  }

  #[cfg(feature = "mcp_stdio")]
  pub async fn remove_stdio(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
    let entry = self.stdio_clients.remove(&config.server_cmd);
    if let Some((_, mut client)) = entry {
      client
        .stop()
        .await
        .map_err(|e| FlowyError::internal().with_context(format!("mcp stdio stop: {}", e)))?;
    }
    Ok(())
  }

  pub async fn remove_sse(&self, id: &str) -> FlowyResult<()> {
    self.sse_clients.remove(id);
    Ok(())
  }

  #[cfg(feature = "mcp_stdio")]
  pub async fn tool_list_stdio(&self, server_cmd: &str) -> FlowyResult<ToolsList> {
    let mut client = self
      .stdio_clients
      .get_mut(server_cmd)
      .ok_or_else(|| FlowyError::internal().with_context("stdio client not found"))?;
    let tools = client
      .list_tools()
      .await
      .map_err(|e| FlowyError::internal().with_context(format!("mcp stdio tools/list: {}", e)))?;
    trace!("{}: tool list: {:?}", server_cmd, tools);
    Ok(tools)
  }

  pub async fn tool_list_sse(&self, id: &str) -> Option<serde_json::Value> {
    // Placeholder: implement real SSE discovery handshake with MCP
    let _client = self.sse_clients.get(id)?;
    None
  }

  #[cfg(feature = "mcp_stdio")]
  pub async fn invoke_tool_stdio(&self, server_cmd: &str, tool: &str, input: serde_json::Value, timeout: Duration) -> FlowyResult<ToolInvokeResponse> {
    let _permit = self
      .concurrency
      .acquire()
      .await
      .map_err(|_| FlowyError::new(ErrorCode::Internal, "semaphore closed"))?;
    let mut client = self
      .stdio_clients
      .get_mut(server_cmd)
      .ok_or_else(|| FlowyError::internal().with_context("stdio client not found"))?;
    debug!(%server_cmd, %tool, "invoking tool via stdio");
    let resp = tokio::time::timeout(timeout, client.invoke_tool(tool, input))
      .await
      .map_err(|e| FlowyError::internal().with_context(format!("mcp stdio timeout: {}", e)))
      .and_then(|r| r.map_err(|e| FlowyError::internal().with_context(format!("mcp stdio invoke: {}", e))))?;
    Ok(resp)
  }

  pub async fn invoke_tool_sse(&self, id: &str, tool: &str, input: serde_json::Value, _timeout: Duration) -> FlowyResult<serde_json::Value> {
    let _permit = self
      .concurrency
      .acquire()
      .await
      .map_err(|_| FlowyError::new(ErrorCode::Internal, "semaphore closed"))?;
    let _client = self
      .sse_clients
      .get(id)
      .ok_or_else(|| FlowyError::internal().with_context("sse client not found"))?;
    // TODO: implement SSE invoke: send POST and stream results back
    Err(FlowyError::new(ErrorCode::NotSupportYet, "SSE invoke not implemented"))
  }
}
