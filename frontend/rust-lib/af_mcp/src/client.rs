use crate::entities::{ToolsList, ToolInvokeResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::io::{Read, Write};
use thiserror::Error;
use which::which;
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
  pub server_cmd: String,
  #[serde(default)]
  pub args: Vec<String>,
  #[serde(default)]
  pub env: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum MCPError {
  #[error("spawn failed: {0}")]
  Spawn(String),
  #[error("io failed: {0}")]
  Io(String),
  #[error("json failed: {0}")]
  Json(String),
  #[error("protocol failed: {0}")]
  Protocol(String),
}

pub struct MCPClient {
  child: std::process::Child,
  initialized: bool,
  next_id: i64,
}

impl MCPClient {
  fn resolve_program(server_cmd: &str) -> Result<PathBuf, MCPError> {
    // Absolute/relative path provided
    if server_cmd.contains('/') {
      let p = PathBuf::from(server_cmd);
      if p.exists() { debug!(?p, "af_mcp: use absolute/relative program path"); return Ok(p); }
      return Err(MCPError::Spawn(format!("command path not found: {}", server_cmd)));
    }

    // Use which first
    if let Ok(p) = which(server_cmd) { debug!(?p, "af_mcp: which resolved program"); return Ok(p); }

    // Fallback to common PATHs (macOS/Homebrew, Linux)
    let candidates = [
      "/opt/homebrew/bin",
      "/usr/local/bin",
      "/usr/bin",
      "/bin",
    ];
    for dir in candidates.iter() {
      let p = Path::new(dir).join(server_cmd);
      if p.exists() { warn!(?p, "af_mcp: fallback resolved program"); return Ok(p); }
    }
    Err(MCPError::Spawn(format!("command not found: {}", server_cmd)))
  }

  pub async fn new_stdio(config: MCPServerConfig) -> Result<Self, MCPError> {
    // Resolve command path with fallbacks
    let program = Self::resolve_program(&config.server_cmd)?;
    let mut cmd = Command::new(program);
    cmd.args(&config.args)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());
    for (k, v) in &config.env {
      cmd.env(k, v);
    }
    debug!(program=?cmd.get_program(), args=?cmd.get_args().collect::<Vec<_>>(), env=?config.env.keys().collect::<Vec<_>>(), "af_mcp: spawning stdio server");
    let mut child = cmd.spawn().map_err(|e| MCPError::Spawn(e.to_string()))?;

    // Drain stderr in background to avoid deadlocks and surface diagnostics
    if let Some(mut stderr) = child.stderr.take() {
      std::thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
          match stderr.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
              if let Ok(s) = String::from_utf8(buf[..n].to_vec()) {
                tracing::debug!(target: "af_mcp::stdio", stderr = %s);
              }
            }
            Err(_) => break,
          }
        }
      });
    }
    Ok(Self { child, initialized: false, next_id: 1 })
  }

  pub async fn initialize(&mut self) -> Result<(), MCPError> {
    // For simplicity, assume server is ready upon spawn.
    Ok(())
  }

  pub async fn stop(&mut self) -> Result<(), MCPError> {
    let _ = self.child.kill();
    Ok(())
  }

  pub async fn list_tools(&mut self) -> Result<ToolsList, MCPError> {
    self.ensure_initialized()?;
    // send tools/list
    let id = self.next_req_id();
    let req = serde_json::json!({
      "jsonrpc": "2.0",
      "id": id,
      "method": "tools/list",
      "params": {},
    });
    trace!(?req, "af_mcp: → tools/list");
    self.write_jsonrpc(&req)?;
    // read until matching id
    loop {
      let msg = self.read_jsonrpc()?;
      trace!(?msg, "af_mcp: ← message");
      if msg.get("id") == Some(&serde_json::json!(id)) {
        let v: ToolsList = serde_json::from_value(msg)
          .map_err(|e| MCPError::Json(e.to_string()))?;
        trace!(count=?v.result.tools.len(), "af_mcp: tools/list done");
        return Ok(v);
      }
      // ignore notifications/other responses
    }
  }

  pub async fn invoke_tool(&mut self, tool: &str, input: serde_json::Value) -> Result<ToolInvokeResponse, MCPError> {
    self.ensure_initialized()?;
    let id = self.next_req_id();
    let req = serde_json::json!({
      "jsonrpc": "2.0",
      "id": id,
      "method": "tools/call",
      "params": { "name": tool, "arguments": input }
    });
    trace!(?req, "af_mcp: → tools/call");
    self.write_jsonrpc(&req)?;
    loop {
      let msg = self.read_jsonrpc()?;
      trace!(?msg, "af_mcp: ← message");
      if msg.get("id") == Some(&serde_json::json!(id)) {
        let v: ToolInvokeResponse = serde_json::from_value(msg)
          .map_err(|e| MCPError::Json(e.to_string()))?;
        return Ok(v);
      }
    }
  }

  fn next_req_id(&mut self) -> i64 {
    let id = self.next_id;
    self.next_id += 1;
    id
  }

  fn ensure_initialized(&mut self) -> Result<(), MCPError> {
    if self.initialized {
      return Ok(());
    }
    let id = self.next_req_id();
    let init = serde_json::json!({
      "jsonrpc": "2.0",
      "id": id,
      "method": "initialize",
      "params": {
        // Use a widely adopted MCP protocol date for maximum compatibility
        "protocolVersion": "2024-05-16",
        "capabilities": {
          "tools": {},
          "prompts": {},
          "resources": {}
        },
        "clientInfo": {"name": "appflowy", "version": "0.1.0"}
      }
    });
    trace!(?init, "af_mcp: → initialize");
    self.write_jsonrpc(&init)?;
    // wait for response with same id
    loop {
      let msg = self.read_jsonrpc()?;
      trace!(?msg, "af_mcp: ← message");
      if msg.get("id") == Some(&serde_json::json!(id)) {
        break;
      }
      // ignore others
    }
    // send initialized notification (no id)
    let inited = serde_json::json!({
      "jsonrpc": "2.0",
      "method": "initialized",
      "params": {}
    });
    trace!(?inited, "af_mcp: → initialized");
    self.write_jsonrpc(&inited)?;
    self.initialized = true;
    trace!("af_mcp: initialized handshake finished");
    Ok(())
  }

  fn write_jsonrpc(&mut self, v: &serde_json::Value) -> Result<(), MCPError> {
    let body = serde_json::to_string(v).map_err(|e| MCPError::Json(e.to_string()))?;
    let header = format!(
      "Content-Length: {}\r\nContent-Type: application/json; charset=utf-8\r\n\r\n",
      body.as_bytes().len()
    );
    let stdin: &mut ChildStdin = self
      .child
      .stdin
      .as_mut()
      .ok_or_else(|| MCPError::Protocol("no stdin".into()))?;
    stdin
      .write_all(header.as_bytes())
      .and_then(|_| stdin.write_all(body.as_bytes()))
      .and_then(|_| stdin.flush())
      .map_err(|e| MCPError::Io(e.to_string()))
  }

  fn read_jsonrpc(&mut self) -> Result<serde_json::Value, MCPError> {
    let stdout: &mut ChildStdout = self
      .child
      .stdout
      .as_mut()
      .ok_or_else(|| MCPError::Protocol("no stdout".into()))?;

    // Robust header parsing: tolerate noisy preface and split reads
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    let mut tmp = [0u8; 1024];
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;

    // Read until we see a header terminator after a Content-Length line
    loop {
      let n = stdout.read(&mut tmp).map_err(|e| MCPError::Io(e.to_string()))?;
      if n == 0 {
        return Err(MCPError::Io("unexpected EOF before header".into()));
      }
      buf.extend_from_slice(&tmp[..n]);

      // Search in a lossy string view to be resilient to non-UTF8 noise
      let text = String::from_utf8_lossy(&buf);
      let lower = text.to_ascii_lowercase();
      if let Some(cl_pos) = lower.find("content-length:") {
        // Find header end after the Content-Length line
        if let Some(end_pos) = lower[cl_pos..].find("\r\n\r\n").map(|p| cl_pos + p + 4)
          .or_else(|| lower[cl_pos..].find("\n\n").map(|p| cl_pos + p + 2))
        {
          header_end = Some(end_pos);
          // Parse the number from the original (non-lowered) slice
          let header_slice = &text[..end_pos];
          // Find the first digits after content-length:
          if let Some(after) = lower[cl_pos..].find(':').map(|p| cl_pos + p + 1) {
            let num_str = header_slice[after..]
              .lines()
              .next()
              .unwrap_or("")
              .trim();
            if let Ok(n) = num_str.parse::<usize>() { content_length = Some(n); }
          }
        }
      }

      if header_end.is_some() && content_length.is_some() { break; }
      if buf.len() > 64 * 1024 { return Err(MCPError::Protocol("header too large or malformed".into())); }
    }

    let header_end = header_end.unwrap();
    let len = content_length.ok_or_else(|| MCPError::Protocol("missing Content-Length".into()))?;

    // Body may already be partially in buf after header
    let mut body: Vec<u8> = Vec::with_capacity(len);
    let already = buf.len().saturating_sub(header_end);
    if already > 0 {
      let take = already.min(len);
      body.extend_from_slice(&buf[header_end..header_end + take]);
    }
    while body.len() < len {
      let n = stdout.read(&mut tmp).map_err(|e| MCPError::Io(e.to_string()))?;
      if n == 0 {
        return Err(MCPError::Io(format!("unexpected EOF: read {} of {} bytes", body.len(), len)));
      }
      let needed = len - body.len();
      body.extend_from_slice(&tmp[..n.min(needed)]);
    }

    let v: serde_json::Value = serde_json::from_slice(&body).map_err(|e| MCPError::Json(e.to_string()))?;
    Ok(v)
  }
}


