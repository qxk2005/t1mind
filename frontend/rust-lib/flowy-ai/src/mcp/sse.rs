use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error};

use flowy_error::{FlowyError, FlowyResult};
use std::time::Duration;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
  pub url: String,
  pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct SseClient {
  http: Client,
  config: SseConfig,
}

impl SseClient {
  pub fn new(config: SseConfig) -> Self {
    Self { http: Client::new(), config }
  }

  pub async fn connect(&self) -> FlowyResult<SseStream> {
    let mut req = self.http.get(&self.config.url)
      .header("accept", "text/event-stream");
    for (k, v) in &self.config.headers {
      req = req.header(k, v);
    }

    let resp = req.send().await.map_err(FlowyError::from)?;
    let mut lines = resp.bytes_stream();

    let (tx, rx) = unbounded_channel::<SseEvent>();
    tokio::spawn(async move {
      let mut event_name: Option<String> = None;
      let mut data_lines: Vec<String> = Vec::new();
      let flush_event = |tx: &UnboundedSender<SseEvent>, name: &mut Option<String>, data_lines: &mut Vec<String>| {
        if !data_lines.is_empty() {
          let data = data_lines.join("\n");
          let _ = tx.send(SseEvent { event: name.clone(), data });
          data_lines.clear();
        }
        *name = None;
      };

      while let Some(chunk) = lines.next().await {
        match chunk {
          Ok(bytes) => {
            if let Ok(text) = String::from_utf8(bytes.to_vec()) {
              for raw in text.split_inclusive(['\n', '\r']).collect::<Vec<_>>() {
                for line in raw.lines() {
                  let line_trim = line.trim_end_matches(['\r']);
                  if line_trim.is_empty() {
                    flush_event(&tx, &mut event_name, &mut data_lines);
                    continue;
                  }
                  if let Some(v) = line_trim.strip_prefix("event:") {
                    event_name = Some(v.trim().to_string());
                    continue;
                  }
                  if let Some(v) = line_trim.strip_prefix("data:") {
                    data_lines.push(v.trim().to_string());
                    continue;
                  }
                  // ignore other fields (id:, retry:, comments)
                }
              }
            }
          },
          Err(err) => {
            error!(?err, "SSE stream error");
            break;
          },
        }
      }
      // flush remaining
      if !data_lines.is_empty() {
        let data = data_lines.join("\n");
        let _ = tx.send(SseEvent { event: event_name.clone(), data });
      }
      debug!("SSE stream completed");
    });

    Ok(SseStream { rx })
  }
}

#[derive(Debug, Clone)]
pub struct SseEvent {
  pub event: Option<String>,
  pub data: String,
}

pub struct SseStream {
  rx: UnboundedReceiver<SseEvent>,
}

impl SseStream {
  pub fn into_stream(self) -> UnboundedReceiverStream<SseEvent> {
    UnboundedReceiverStream::new(self.rx)
  }
}


/// Perform a minimal MCP tools discovery over SSE transport.
/// It will:
/// 1) Open the SSE stream at `url`
/// 2) POST a JSON-RPC request `{ "jsonrpc": "2.0", "id": "tools_list", "method": "tools/list" }` to the same `url`
/// 3) Wait up to `timeout` for a response line on the SSE stream that contains a JSON object
///    with a `result.tools` array, then return that JSON object.
pub async fn sse_tools_list(url: &str, headers: &[(String, String)], timeout: Duration) -> FlowyResult<Value> {
  // 1) Connect SSE stream
  let client = SseClient::new(SseConfig {
    url: url.to_string(),
    headers: headers.to_vec(),
  });
  let sse = client.connect().await?;
  let mut stream = sse.into_stream();

  // 2) Optionally wait briefly for server to announce endpoint via SSE, then send initialize + tools/list via POST
  let http = Client::new();
  // Allow specifying a different POST endpoint via:
  //   - query param:  .../sse?post=http://host/rpc
  //   - header:       X-Post-Url: http://host/rpc
  let mut post_url = url.to_string();
  let base_url = reqwest::Url::parse(url).ok();
  if let Ok(parsed) = reqwest::Url::parse(url) {
    if let Some(v) = parsed
      .query_pairs()
      .find(|(k, _)| k == "post")
      .map(|(_, v)| v.to_string())
    {
      post_url = v;
    }
  }
  for (k, v) in headers {
    if k.eq_ignore_ascii_case("x-post-url") {
      post_url = v.clone();
    }
  }

  let build_req = |url: &str| {
    let mut r = http.post(url.to_string());
    for (k, v) in headers { r = r.header(k, v); }
    r.header("content-type", "application/json")
  };

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

  let tools_payload = serde_json::json!({
    "jsonrpc": "2.0",
    "id": "tools_list",
    "method": "tools/list",
    "params": {},
  });
  // First, try to read an endpoint announcement quickly (e.g. event:endpoint)
  let mut initial_frame: Option<SseEvent> = None;
  if let Ok(Some(frame)) = tokio::time::timeout(Duration::from_millis(800), stream.next()).await {
    initial_frame = Some(frame);
  }
  if let Some(frame) = initial_frame.clone() {
    if let Some(ev) = frame.event.as_deref() {
      if ev.eq_ignore_ascii_case("endpoint") {
        let data = frame.data.trim();
        if let Some(url2) = base_url.clone() {
          if data.starts_with("http://") || data.starts_with("https://") {
            post_url = data.to_string();
          } else if let Ok(joined) = url2.join(data) { post_url = joined.to_string(); }
        }
      }
    }
  } else {
    // No immediate endpoint event; keep default (may be overridden later when event arrives)
  }

  // send initialize then tools/list to current post_url
  let _ = build_req(&post_url).body(init_payload.to_string()).send().await.map_err(FlowyError::from)?;
  let _ = build_req(&post_url).body(tools_payload.to_string()).send().await.map_err(FlowyError::from)?;

  // 3) Wait for SSE response containing result.tools
  let result = tokio::time::timeout(timeout, async {
    while let Some(frame) = stream.next().await {
      let data = frame.data.trim();
      // Special handshake: event: endpoint -> update post url dynamically
      if let Some(ev) = frame.event.as_deref() {
        if ev.eq_ignore_ascii_case("endpoint") {
          if let Some(url2) = base_url.clone() {
            if data.starts_with("http://") || data.starts_with("https://") {
              post_url = data.to_string();
            } else {
              // join relative to base
              if let Ok(joined) = url2.join(data) { post_url = joined.to_string(); }
            }
            // re-send initialize + tools/list to the new post_url
            let _ = build_req(&post_url).body(init_payload.to_string()).send().await.map_err(FlowyError::from)?;
            let _ = build_req(&post_url).body(tools_payload.to_string()).send().await.map_err(FlowyError::from)?;
          }
          continue;
        }
      }
      if data.is_empty() { continue; }
      if let Ok(v) = serde_json::from_str::<Value>(data) {
        if v.get("result").and_then(|r| r.get("tools")).is_some() {
          return Ok(v);
        }
      }
    }
    Err(FlowyError::internal().with_context("SSE ended without tools/list result"))
  })
  .await
  .map_err(|_| FlowyError::internal().with_context("SSE tools/list timed out"))??;

  Ok(result)
}


