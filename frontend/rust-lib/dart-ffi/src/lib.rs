#![allow(clippy::not_unsafe_ptr_arg_deref)]

use allo_isolate::Isolate;
use futures::ready;
use lazy_static::lazy_static;
use semver::Version;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use std::{ffi::CStr, os::raw::c_char};
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tracing::{debug, error, info, trace, warn};

use flowy_core::config::AppFlowyCoreConfig;
use flowy_core::*;
use flowy_notification::{register_notification_sender, unregister_all_notification_sender};
use flowy_server_pub::AuthenticatorType;
use lib_dispatch::prelude::ToBytes;
use lib_dispatch::prelude::*;
use lib_dispatch::runtime::AFPluginRuntime;
use lib_log::stream_log::StreamLogSender;

use crate::appflowy_yaml::save_appflowy_cloud_config;
use crate::env_serde::AppFlowyDartConfiguration;
use crate::notification::DartNotificationSender;
use crate::{
  c::{extend_front_four_bytes_into_bytes, forget_rust, reclaim_rust},
  model::{FFIRequest, FFIResponse},
};
use flowy_ai::mcp::manager::MCPClientManager;
use flowy_ai::mcp::sse::sse_tools_list;
use tokio::runtime::Builder;
use std::time::Duration;
use std::sync::OnceLock;

mod appflowy_yaml;
mod c;
mod env_serde;
mod model;
mod notification;
mod protobuf;

lazy_static! {
  static ref DART_APPFLOWY_CORE: DartAppFlowyCore = DartAppFlowyCore::new();
  static ref LOG_STREAM_ISOLATE: RwLock<Option<Isolate>> = RwLock::new(None);
}

static MCP_MANAGER: OnceLock<MCPClientManager> = OnceLock::new();

pub struct Task {
  dispatcher: Arc<AFPluginDispatcher>,
  request: AFPluginRequest,
  port: i64,
  ret: Option<mpsc::Sender<AFPluginEventResponse>>,
}

unsafe impl Send for Task {}
unsafe impl Sync for DartAppFlowyCore {}

struct DartAppFlowyCore {
  core: Arc<RwLock<Option<AppFlowyCore>>>,
  handle: RwLock<Option<std::thread::JoinHandle<()>>>,
  sender: RwLock<Option<mpsc::UnboundedSender<Task>>>,
}

impl DartAppFlowyCore {
  fn new() -> Self {
    Self {
      #[allow(clippy::arc_with_non_send_sync)]
      core: Arc::new(RwLock::new(None)),
      handle: RwLock::new(None),
      sender: RwLock::new(None),
    }
  }

  fn dispatcher(&self) -> Option<Arc<AFPluginDispatcher>> {
    let binding = self
      .core
      .read()
      .expect("Failed to acquire read lock for core");
    let core = binding.as_ref();
    core.map(|core| core.event_dispatcher.clone())
  }

  fn dispatch(
    &self,
    request: AFPluginRequest,
    port: i64,
    ret: Option<mpsc::Sender<AFPluginEventResponse>>,
  ) {
    if let Ok(sender_guard) = self.sender.read() {
      let dispatcher = match self.dispatcher() {
        Some(dispatcher) => dispatcher,
        None => {
          error!("Failed to get dispatcher: dispatcher is None");
          return;
        },
      };

      if let Some(sender) = sender_guard.as_ref() {
        if let Err(e) = sender.send(Task {
          dispatcher,
          request,
          port,
          ret,
        }) {
          error!("Failed to send task: {}", e);
        }
      } else {
        error!("Failed to send task: sender is None");
      }
    } else {
      warn!("Failed to acquire read lock for sender");
    }
  }
}

#[no_mangle]
pub extern "C" fn init_sdk(_port: i64, data: *mut c_char) -> i64 {
  let c_str = unsafe {
    if data.is_null() {
      return -1;
    }
    CStr::from_ptr(data)
  };
  let serde_str = c_str
    .to_str()
    .expect("Failed to convert C string to Rust string");
  let configuration = AppFlowyDartConfiguration::from_str(serde_str);
  configuration.write_env();

  if configuration.authenticator_type == AuthenticatorType::AppFlowyCloud {
    let _ = save_appflowy_cloud_config(&configuration.root, &configuration.appflowy_cloud_config);
  }

  let mut app_version =
    Version::parse(&configuration.app_version).unwrap_or_else(|_| Version::new(0, 5, 8));

  let min_version = Version::new(0, 5, 8);
  if app_version < min_version {
    app_version = min_version;
  }

  let config = AppFlowyCoreConfig::new(
    app_version,
    configuration.custom_app_path,
    configuration.origin_app_path,
    configuration.device_id,
    configuration.platform,
    DEFAULT_NAME.to_string(),
  );

  if let Some(core) = &*DART_APPFLOWY_CORE.core.write().unwrap() {
    core.close_db();
  }

  let log_stream = LOG_STREAM_ISOLATE
    .write()
    .unwrap()
    .take()
    .map(|isolate| Arc::new(LogStreamSenderImpl { isolate }) as Arc<dyn StreamLogSender>);
  let (sender, task_rx) = mpsc::unbounded_channel::<Task>();
  let runtime = Arc::new(AFPluginRuntime::new().unwrap());
  let cloned_runtime = runtime.clone();
  let handle = std::thread::spawn(move || {
    let local_set = LocalSet::new();
    cloned_runtime.block_on(local_set.run_until(Runner { rx: task_rx }));
  });

  *DART_APPFLOWY_CORE.sender.write().unwrap() = Some(sender);
  *DART_APPFLOWY_CORE.handle.write().unwrap() = Some(handle);
  let cloned_runtime = runtime.clone();
  *DART_APPFLOWY_CORE.core.write().unwrap() = runtime
    .block_on(async move { Some(AppFlowyCore::new(config, cloned_runtime, log_stream).await) });
  MCP_MANAGER.get_or_init(|| MCPClientManager::new());
  0
}

#[no_mangle]
#[allow(clippy::let_underscore_future)]
pub extern "C" fn async_event(port: i64, input: *const u8, len: usize) {
  let request: AFPluginRequest = FFIRequest::from_u8_pointer(input, len).into();
  #[cfg(feature = "verbose_log")]
  trace!(
    "[FFI]: {} Async Event: {:?} with {} port",
    &request.id,
    &request.event,
    port
  );

  DART_APPFLOWY_CORE.dispatch(request, port, None);
}

/// A persistent future that processes [Arbiter] commands.
struct Runner {
  rx: mpsc::UnboundedReceiver<Task>,
}

impl Future for Runner {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    loop {
      match ready!(self.rx.poll_recv(cx)) {
        None => return Poll::Ready(()),
        Some(task) => {
          let Task {
            dispatcher,
            request,
            port,
            ret,
          } = task;

          tokio::task::spawn_local(async move {
            let resp = AFPluginDispatcher::boxed_async_send_with_callback(
              dispatcher.as_ref(),
              request,
              move |resp: AFPluginEventResponse| {
                #[cfg(feature = "verbose_log")]
                trace!("[FFI]: Post data to dart through {} port", port);
                Box::pin(post_to_flutter(resp, port))
              },
            )
            .await;

            if let Some(ret) = ret {
              let _ = ret.send(resp).await;
            }
          });
        },
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn sync_event(_input: *const u8, _len: usize) -> *const u8 {
  error!("unimplemented sync_event");

  let response_bytes = vec![];
  let result = extend_front_four_bytes_into_bytes(&response_bytes);
  forget_rust(result)
}

#[no_mangle]
pub extern "C" fn set_stream_port(notification_port: i64) -> i32 {
  unregister_all_notification_sender();
  register_notification_sender(DartNotificationSender::new(notification_port));
  0
}

#[no_mangle]
pub extern "C" fn set_log_stream_port(port: i64) -> i32 {
  *LOG_STREAM_ISOLATE.write().unwrap() = Some(Isolate::new(port));
  0
}

#[inline(never)]
#[no_mangle]
pub extern "C" fn link_me_please() {}

#[inline(always)]
#[allow(clippy::blocks_in_conditions)]
async fn post_to_flutter(response: AFPluginEventResponse, port: i64) {
  let isolate = allo_isolate::Isolate::new(port);
  match isolate
    .catch_unwind(async {
      let ffi_resp = FFIResponse::from(response);
      ffi_resp.into_bytes().unwrap().to_vec()
    })
    .await
  {
    Ok(_) => {
      #[cfg(feature = "verbose_log")]
      trace!("[FFI]: Post data to dart success");
    },
    Err(err) => {
      error!("[FFI]: allo_isolate post failed: {:?}", err);
    },
  }
}

#[no_mangle]
pub extern "C" fn rust_log(level: i64, data: *const c_char) {
  if data.is_null() {
    error!("[flutter error]: null pointer provided to backend_log");
    return;
  }

  let log_result = unsafe { CStr::from_ptr(data) }.to_str();

  let log_str = match log_result {
    Ok(str) => str,
    Err(e) => {
      error!(
        "[flutter error]: Failed to convert C string to Rust string: {:?}",
        e
      );
      return;
    },
  };

  match level {
    0 => info!("[Flutter]: {}", log_str),
    1 => debug!("[Flutter]: {}", log_str),
    2 => trace!("[Flutter]: {}", log_str),
    3 => warn!("[Flutter]: {}", log_str),
    4 => error!("[Flutter]: {}", log_str),
    _ => warn!("[flutter error]: Unsupported log level: {}", level),
  }
}

#[no_mangle]
pub extern "C" fn set_env(_data: *const c_char) {
  // Deprecated
}

#[no_mangle]
pub extern "C" fn free_bytes(ptr: *mut u8, len: u32) {
  reclaim_rust(ptr, len);
}

#[no_mangle]
pub extern "C" fn mcp_check_streamable_http(url: *const c_char, headers_json: *const c_char) -> *const u8 {
  let url = unsafe { CStr::from_ptr(url) }.to_string_lossy().to_string();
  let headers_json = unsafe { CStr::from_ptr(headers_json) }.to_string_lossy().to_string();
  let headers: Vec<(String, String)> = serde_json::from_str(&headers_json).unwrap_or_default();

  let (ok, response_json, tool_count, server) = match flowy_ai::streamable_http_tools_list(url.clone(), headers.clone()) {
    Ok(result) => (true, result.to_string(), result.get("tools").and_then(|t| t.as_array()).map(|arr| arr.len()).unwrap_or(0), Some("streamable-http-mcp".to_string())),
    Err(e) => (false, format!(r#"{{"error":"code:Internal error, message:Streamable HTTP tools/list {}"}}"#, e), 0, None),
  };

  let request = serde_json::json!({
    "transport": "streamableHttp",
    "url": url,
    "headers": headers,
    "method": "tools/list",
  });

  let now = chrono::Utc::now().to_rfc3339();
  let body = serde_json::json!({
    "ok": ok,
    "requestJson": request.to_string(),
    "responseJson": response_json,
    "toolCount": tool_count,
    "server": server,
    "checkedAtIso": now,
  })
  .to_string();

  let bytes = body.into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

#[no_mangle]
pub extern "C" fn mcp_check_sse(url: *const c_char, headers_json: *const c_char) -> *const u8 {
  let url = unsafe { CStr::from_ptr(url) }.to_string_lossy().to_string();
  let headers_json = unsafe { CStr::from_ptr(headers_json) }.to_string_lossy().to_string();

  // Parse headers_json which can be either a map {"k":"v"} or a vec [["k","v"], ...]
  let headers_vec: Vec<(String, String)> = serde_json::from_str::<Vec<(String, String)>>(&headers_json)
    .or_else(|_| {
      serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&headers_json).map(|m| {
        m.into_iter()
          .map(|(k, v)| (k, v.as_str().unwrap_or(&v.to_string()).to_string()))
          .collect::<Vec<(String, String)>>()
      })
    })
    .unwrap_or_default();

  // Build a lightweight tokio runtime to perform the network call
  let rt = Builder::new_current_thread().enable_all().build();

  let (ok, response_json, server, tool_count) = match rt {
    Ok(rt) => {
      let res = rt.block_on(sse_tools_list(&url, &headers_vec, Duration::from_secs(15)));
      match res {
        Ok(v) => {
          // Try to extract server and tools count
          let server = v.get("server").and_then(|s| s.as_str()).unwrap_or("").to_string();
          let tool_count = v
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .map(|a| a.len() as i32)
            .unwrap_or(0);
          (true, v.to_string(), server, tool_count)
        },
        Err(e) => {
          let err = serde_json::json!({"error": e.to_string()}).to_string();
          (false, err, String::new(), 0)
        },
      }
    },
    Err(e) => {
      let err = serde_json::json!({"error": format!("failed to init runtime: {}", e)}).to_string();
      (false, err, String::new(), 0)
    },
  };

  let request = serde_json::json!({
    "transport": "sse",
    "url": url,
    "headers": headers_vec,
    "method": "tools/list",
  });

  let now = chrono::Utc::now().to_rfc3339();
  let body = serde_json::json!({
    "ok": ok,
    "requestJson": request.to_string(),
    "responseJson": response_json,
    "toolCount": tool_count,
    "server": server,
    "checkedAtIso": now,
  })
  .to_string();

  let bytes = body.into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

#[no_mangle]
pub extern "C" fn mcp_connect_sse(id: *const c_char, url: *const c_char, headers_json: *const c_char) -> i32 {
  let id = unsafe { CStr::from_ptr(id) }.to_string_lossy().to_string();
  let url = unsafe { CStr::from_ptr(url) }.to_string_lossy().to_string();
  let headers_json = unsafe { CStr::from_ptr(headers_json) }.to_string_lossy().to_string();
  let headers: Vec<(String, String)> = serde_json::from_str(&headers_json).unwrap_or_default();
  let mgr = MCP_MANAGER.get().expect("MCP manager not initialized");
  match futures::executor::block_on(mgr.connect_sse(id, url, headers)) {
    Ok(_) => 0,
    Err(_) => -1,
  }
}

#[no_mangle]
pub extern "C" fn mcp_disconnect_sse(id: *const c_char) -> i32 {
  let id = unsafe { CStr::from_ptr(id) }.to_string_lossy().to_string();
  let mgr = MCP_MANAGER.get().expect("MCP manager not initialized");
  match futures::executor::block_on(mgr.remove_sse(&id)) {
    Ok(_) => 0,
    Err(_) => -1,
  }
}

#[no_mangle]
pub extern "C" fn mcp_check_stdio(command: *const c_char, args_json: *const c_char, env_json: *const c_char) -> *const u8 {
  let cmd = unsafe { CStr::from_ptr(command) }.to_string_lossy().to_string();
  let args_json = unsafe { CStr::from_ptr(args_json) }.to_string_lossy().to_string();
  let env_json = unsafe { CStr::from_ptr(env_json) }.to_string_lossy().to_string();

  let request = serde_json::json!({
    "transport": "stdio",
    "command": cmd,
    "args": args_json,
    "env": env_json,
    "method": "tools/list",
  });

  #[cfg(feature = "mcp_stdio")]
  {
    use af_mcp::client::MCPServerConfig;
    let mgr = MCP_MANAGER.get().expect("MCP manager not initialized");
    let args_vec: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
    let env_map: std::collections::HashMap<String, String> = serde_json::from_str(&env_json).unwrap_or_default();
    let cfg = MCPServerConfig { server_cmd: cmd.clone(), args: args_vec.clone(), env: env_map.clone() };
    let rt = Builder::new_current_thread().enable_all().build();
    let (ok, response_json, tool_count, server) = match rt {
      Ok(rt) => {
        let res = rt.block_on(async {
          let fut = async {
            let _ = mgr.connect_stdio(cfg.clone()).await?;
            let v = mgr.tool_list_stdio(&cfg.server_cmd).await?;
            let _ = mgr.remove_stdio(cfg.clone()).await;
            // normalize into JSON
            let obj = serde_json::to_value(&v).unwrap_or_else(|_| serde_json::json!({"server":"mcp-stdio","result":{"tools":[]}}));
            let count = obj.get("result").and_then(|r| r.get("tools")).and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0);
            let server = obj.get("server").and_then(|s| s.as_str()).unwrap_or("").to_string();
            tracing::debug!(?cfg, count, server, "mcp_check_stdio: tools listed");
            Ok::<(serde_json::Value, usize, String), flowy_error::FlowyError>((obj, count, server))
          };
          match tokio::time::timeout(Duration::from_secs(30), fut).await {
            Ok(r) => r,
            Err(_) => Err(flowy_error::FlowyError::internal().with_context("mcp stdio timeout")),
          }
        });
        match res {
          Ok((obj, count, server)) => (true, obj.to_string(), count as i32, server),
          Err(e) => (false, serde_json::json!({"error": e.to_string()}).to_string(), 0, String::new()),
        }
      },
      Err(e) => (false, serde_json::json!({"error": format!("failed to init runtime: {}", e)}).to_string(), 0, String::new()),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let body = serde_json::json!({
      "ok": ok,
      "requestJson": request.to_string(),
      "responseJson": response_json,
      "toolCount": tool_count,
      "server": server,
      "checkedAtIso": now,
    }).to_string();

    let bytes = body.into_bytes();
    let result = extend_front_four_bytes_into_bytes(&bytes);
    return forget_rust(result);
  }

  #[cfg(not(feature = "mcp_stdio"))]
  {
    let now = chrono::Utc::now().to_rfc3339();
    let body = serde_json::json!({
      "ok": false,
      "requestJson": request.to_string(),
      "responseJson": serde_json::json!({"error": "mcp_stdio feature not enabled"}).to_string(),
      "toolCount": 0,
      "server": "mcp-stdio",
      "checkedAtIso": now,
    }).to_string();
    let bytes = body.into_bytes();
    let result = extend_front_four_bytes_into_bytes(&bytes);
    forget_rust(result)
  }
}

struct LogStreamSenderImpl {
  isolate: Isolate,
}
impl StreamLogSender for LogStreamSenderImpl {
  fn send(&self, message: &[u8]) {
    self.isolate.post(message.to_vec());
  }
}
