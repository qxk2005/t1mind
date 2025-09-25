use flowy_error::{FlowyError, FlowyResult};
use serde_json::Value;
use std::time::Duration;
use tracing::debug;

/// Parse SSE response to extract JSON data
fn parse_sse_response(sse_text: &str) -> Option<Value> {
  for line in sse_text.lines() {
    if line.starts_with("data: ") {
      let json_str = &line[6..]; // Remove "data: " prefix
      if let Ok(json) = serde_json::from_str::<Value>(json_str) {
        return Some(json);
      }
    }
  }
  None
}

/// Extract session ID from response headers
fn extract_session_id(response: &ureq::Response) -> Option<String> {
  response.header("mcp-session-id").map(|s| s.to_string())
}

/// Perform a minimal MCP tools discovery over streamable HTTP transport.
/// This is similar to SSE but uses regular HTTP streaming instead of Server-Sent Events.
/// The protocol expects JSON-RPC messages to be sent via POST and responses to be streamed back.
pub fn streamable_http_tools_list(
  url: String,
  headers: Vec<(String, String)>,
) -> FlowyResult<Value> {
  let timeout = Duration::from_secs(15);
  
  debug!("MCP streamable-http tools/list: url={}", url);
  
  // Use blocking HTTP client to avoid async runtime issues in FFI context
  let client = std::sync::Arc::new(ureq::AgentBuilder::new()
    .timeout(timeout)
    .build());
  
  // First, try to probe the server with a simple GET to understand what it expects
  debug!("Probing server at: {}", url);
  let probe_response = client.get(&url).call();
  match probe_response {
    Ok(response) => {
      let status = response.status();
      let headers: Vec<String> = response.headers_names();
      debug!("Server probe - Status: {}, Headers: {:?}", status, headers);
      if let Ok(body) = response.into_string() {
        debug!("Server probe response body: {}", body);
      }
    },
    Err(e) => {
      debug!("Server probe failed: {}", e);
    }
  }
  
  // Try different common MCP endpoints if the original fails
  let possible_endpoints = vec![
    url.clone(),
    format!("{}/mcp", url.trim_end_matches("/mcp")),
    format!("{}/rpc", url.trim_end_matches("/mcp")),
    format!("{}/jsonrpc", url.trim_end_matches("/mcp")),
    url.trim_end_matches("/mcp").to_string(),
  ];
  
  let mut working_url = url.clone();
  for test_url in &possible_endpoints {
    if test_url != &url {
      debug!("Testing alternative endpoint: {}", test_url);
      if let Ok(response) = client.get(test_url).call() {
        debug!("Alternative endpoint {} returned status: {}", test_url, response.status());
        if response.status() != 404 && response.status() != 405 {
          working_url = test_url.clone();
          debug!("Using endpoint: {}", working_url);
          break;
        }
      }
    }
  }
  
  // Prepare headers - MCP server requires both application/json and text/event-stream in Accept
  let mut req_headers = Vec::new();
  req_headers.push(("Content-Type", "application/json"));
  req_headers.push(("Accept", "application/json, text/event-stream"));
  req_headers.push(("User-Agent", "AppFlowy-MCP/1.0"));
  req_headers.push(("Connection", "keep-alive"));
  for (key, value) in &headers {
    req_headers.push((key.as_str(), value.as_str()));
  }
  
  // Try alternative content types if the first one fails
  let content_types = vec![
    "application/json",
    "application/json-rpc",
    "text/plain",
  ];
  
  // Send initialize request first
  let init_payload = serde_json::json!({
    "jsonrpc": "2.0",
    "id": "init_for_tools",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-05-16",
      "capabilities": { "tools": {}, "prompts": {}, "resources": {} },
      "clientInfo": { "name": "appflowy", "version": "0.1.0" }
    }
  });
  
  // Try different content types for initialize request
  let mut session_id: Option<String> = None;
  let mut last_error = String::new();
  
  for content_type in &content_types {
    debug!("Trying initialize with Content-Type: {}", content_type);
    let mut init_req = client.post(&working_url);
    
    // Set headers with current content type
    init_req = init_req.set("Content-Type", content_type);
    init_req = init_req.set("Accept", "application/json, text/event-stream");
    init_req = init_req.set("User-Agent", "AppFlowy-MCP/1.0");
    init_req = init_req.set("Connection", "keep-alive");
    for (key, value) in &headers {
      init_req = init_req.set(key, value);
    }
    
    match init_req.send_string(&init_payload.to_string()) {
      Ok(init_response) => {
        let status = init_response.status();
        if status >= 200 && status < 300 {
          debug!("Initialize succeeded with Content-Type: {}", content_type);
          
          // Extract session ID from headers
          session_id = extract_session_id(&init_response);
          debug!("Extracted session ID: {:?}", session_id);
          
          // Parse the SSE response to verify initialization
          let response_text = init_response.into_string().unwrap_or_default();
          debug!("Initialize response: {}", response_text);
          
          if let Some(json_data) = parse_sse_response(&response_text) {
            debug!("Parsed initialize response: {}", json_data);
            if json_data.get("result").is_some() {
              // Send notifications/initialized after successful initialize
              debug!("Sending notifications/initialized");
              let init_notification = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
              });
              
              let mut notify_req = client.post(&working_url);
              notify_req = notify_req.set("Content-Type", "application/json");
              notify_req = notify_req.set("Accept", "application/json, text/event-stream");
              notify_req = notify_req.set("User-Agent", "AppFlowy-MCP/1.0");
              notify_req = notify_req.set("Connection", "keep-alive");
              
              if let Some(ref sid) = session_id {
                notify_req = notify_req.set("mcp-session-id", sid);
              }
              
              for (key, value) in &headers {
                notify_req = notify_req.set(key, value);
              }
              
              match notify_req.send_string(&init_notification.to_string()) {
                Ok(_) => {
                  debug!("notifications/initialized sent successfully");
                },
                Err(e) => {
                  debug!("Failed to send notifications/initialized: {}", e);
                }
              }
              
              break;
            }
          }
        } else {
          let response_text = init_response.into_string().unwrap_or_default();
          last_error = format!("Status: {} - {}", status, response_text);
          debug!("Initialize failed with Content-Type: {} - {}", content_type, last_error);
        }
      },
      Err(e) => {
        last_error = format!("Request error: {}", e);
        debug!("Initialize request error with Content-Type: {} - {}", content_type, last_error);
      }
    }
  }
  
  if session_id.is_none() {
    return Err(FlowyError::internal().with_context(format!(
      "Initialize failed to get session ID. Last error: {}", 
      last_error
    )));
  }
  
  // Send tools/list request
  let tools_payload = serde_json::json!({
    "jsonrpc": "2.0",
    "id": "tools_list",
    "method": "tools/list",
    "params": {}
  });
  
  // Send tools/list request with session ID
  let mut tools_req = client.post(&working_url);
  tools_req = tools_req.set("Content-Type", "application/json");
  tools_req = tools_req.set("Accept", "application/json, text/event-stream");
  tools_req = tools_req.set("User-Agent", "AppFlowy-MCP/1.0");
  tools_req = tools_req.set("Connection", "keep-alive");
  
  // Add session ID header if we have one
  if let Some(ref sid) = session_id {
    tools_req = tools_req.set("mcp-session-id", sid);
    debug!("Using session ID for tools/list: {}", sid);
  }
  
  for (key, value) in &headers {
    tools_req = tools_req.set(key, value);
  }
  
  let tools_response = tools_req
    .send_string(&tools_payload.to_string())
    .map_err(|e| {
      debug!("Tools/list request error: {}", e);
      FlowyError::internal().with_context(format!("Tools/list request failed: {}", e))
    })?;
    
  let status = tools_response.status();
  if status < 200 || status >= 300 {
    let response_text = tools_response.into_string().unwrap_or_default();
    debug!("Tools/list failed - Status: {}, Response: {}", status, response_text);
    return Err(FlowyError::internal().with_context(format!(
      "Tools/list failed with status: {} - {}", 
      status, response_text
    )));
  }
  
  let response_text = tools_response.into_string()
    .map_err(|e| FlowyError::internal().with_context(format!("Failed to read tools/list response: {}", e)))?;
  debug!("MCP streamable-http response: {}", response_text);
  
  // Try to parse as SSE first, then as regular JSON
  let response_json = if let Some(sse_json) = parse_sse_response(&response_text) {
    debug!("Parsed SSE response: {}", sse_json);
    sse_json
  } else {
    // Try parsing as regular JSON
    serde_json::from_str::<Value>(&response_text)
      .map_err(|e| FlowyError::internal().with_context(format!("Failed to parse JSON: {}", e)))?
  };
  
  // Extract tools from the response
  if let Some(result) = response_json.get("result") {
    if let Some(tools) = result.get("tools") {
      return Ok(serde_json::json!({
        "tools": tools,
        "server": "streamable-http-mcp"
      }));
    }
  }
  
  // If no tools found in result, check if the response itself contains tools
  if let Some(tools) = response_json.get("tools") {
    return Ok(serde_json::json!({
      "tools": tools,
      "server": "streamable-http-mcp"
    }));
  }
  
  // Return empty tools if none found
  Ok(serde_json::json!({
    "tools": [],
    "server": "streamable-http-mcp"
  }))
}
