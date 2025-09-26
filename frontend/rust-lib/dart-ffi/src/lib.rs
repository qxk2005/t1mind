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
use flowy_ai::task_orchestrator::{TaskOrchestrator, ExecutionContext, AgentConfig, ExecutionProgress};
use flowy_ai::execution_logger::{ExecutionLogger, ExecutionLogSearchCriteria, ExecutionLogExportOptions};
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
static TASK_ORCHESTRATOR: OnceLock<Arc<TaskOrchestrator>> = OnceLock::new();
static EXECUTION_LOGGER: OnceLock<Arc<ExecutionLogger>> = OnceLock::new();

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
  
  // 初始化MCP管理器
  let mcp_manager = Arc::new(MCPClientManager::new());
  MCP_MANAGER.get_or_init(|| mcp_manager.as_ref().clone());
  
  // 初始化任务编排器和执行日志记录器
  if let Some(core) = DART_APPFLOWY_CORE.core.read().unwrap().as_ref() {
    let ai_manager = core.ai_manager.clone();
    let user_service = core.user_manager.cloud_service().ok();
    
    let task_orchestrator = Arc::new(TaskOrchestrator::new(
      ai_manager,
      mcp_manager.clone(),
      5, // 最大并发执行数
    ));
    
    TASK_ORCHESTRATOR.get_or_init(|| task_orchestrator);
    
    // TODO: 初始化执行日志记录器需要正确的用户服务类型
    // let execution_logger = Arc::new(ExecutionLogger::new(user_service));
    // EXECUTION_LOGGER.get_or_init(|| execution_logger);
  }
  
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

// ============================================================================
// 任务编排相关FFI函数
// ============================================================================

/// 创建任务规划
#[no_mangle]
pub extern "C" fn task_create_plan(
  user_query: *const c_char,
  session_id: *const c_char,
  agent_id: *const c_char,
) -> *const u8 {
  let user_query = unsafe { CStr::from_ptr(user_query) }.to_string_lossy().to_string();
  let session_id = if session_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(session_id) }.to_string_lossy().to_string())
  };
  let agent_id = if agent_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(agent_id) }.to_string_lossy().to_string())
  };

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.create_task_plan(user_query, session_id, agent_id).await {
      Ok(plan) => serde_json::to_value(&plan).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize plan"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 确认任务规划
#[no_mangle]
pub extern "C" fn task_confirm_plan(plan_id: *const c_char) -> *const u8 {
  let plan_id = unsafe { CStr::from_ptr(plan_id) }.to_string_lossy().to_string();

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.confirm_task_plan(&plan_id).await {
      Ok(_) => serde_json::json!({"success": true}),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 执行任务规划
#[no_mangle]
pub extern "C" fn task_execute_plan(
  plan_id: *const c_char,
  context_json: *const c_char,
) -> *const u8 {
  let plan_id = unsafe { CStr::from_ptr(plan_id) }.to_string_lossy().to_string();
  let context_json = unsafe { CStr::from_ptr(context_json) }.to_string_lossy().to_string();

  let context: ExecutionContext = match serde_json::from_str(&context_json) {
    Ok(context) => context,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Invalid context JSON: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.execute_task_plan(&plan_id, context).await {
      Ok(execution_result) => serde_json::to_value(&execution_result).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize result"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 取消任务执行
#[no_mangle]
pub extern "C" fn task_cancel_execution(plan_id: *const c_char) -> *const u8 {
  let plan_id = unsafe { CStr::from_ptr(plan_id) }.to_string_lossy().to_string();

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.cancel_task_execution(&plan_id).await {
      Ok(_) => serde_json::json!({"success": true}),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 获取任务规划
#[no_mangle]
pub extern "C" fn task_get_plan(plan_id: *const c_char) -> *const u8 {
  let plan_id = unsafe { CStr::from_ptr(plan_id) }.to_string_lossy().to_string();

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.get_task_plan(&plan_id).await {
      Ok(plan) => serde_json::to_value(&plan).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize plan"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 获取所有活跃的任务规划
#[no_mangle]
pub extern "C" fn task_get_active_plans() -> *const u8 {
  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    let plans = orchestrator.get_active_task_plans().await;
    serde_json::to_value(&plans).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize plans"}))
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

// ============================================================================
// 智能体配置相关FFI函数
// ============================================================================

/// 添加智能体配置
#[no_mangle]
pub extern "C" fn agent_add_config(config_json: *const c_char) -> *const u8 {
  let config_json = unsafe { CStr::from_ptr(config_json) }.to_string_lossy().to_string();

  let config: AgentConfig = match serde_json::from_str(&config_json) {
    Ok(config) => config,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Invalid config JSON: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.add_agent_config(config).await {
      Ok(_) => serde_json::json!({"success": true}),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 获取智能体配置
#[no_mangle]
pub extern "C" fn agent_get_config(agent_id: *const c_char) -> *const u8 {
  let agent_id = unsafe { CStr::from_ptr(agent_id) }.to_string_lossy().to_string();

  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => {
      let error = serde_json::json!({"error": "Task orchestrator not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match orchestrator.get_agent_config(&agent_id).await {
      Ok(config) => serde_json::to_value(&config).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize config"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

// ============================================================================
// 执行日志相关FFI函数
// ============================================================================

/// 创建执行日志
#[no_mangle]
pub extern "C" fn execution_log_create(
  session_id: *const c_char,
  user_query: *const c_char,
  task_plan_id: *const c_char,
  agent_id: *const c_char,
  user_id: *const c_char,
  workspace_id: *const c_char,
) -> *const u8 {
  let session_id = unsafe { CStr::from_ptr(session_id) }.to_string_lossy().to_string();
  let user_query = unsafe { CStr::from_ptr(user_query) }.to_string_lossy().to_string();
  let task_plan_id = if task_plan_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(task_plan_id) }.to_string_lossy().to_string())
  };
  let agent_id = if agent_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(agent_id) }.to_string_lossy().to_string())
  };
  let user_id = if user_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(user_id) }.to_string_lossy().to_string())
  };
  let workspace_id = if workspace_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(workspace_id) }.to_string_lossy().to_string())
  };

  let logger = match EXECUTION_LOGGER.get() {
    Some(logger) => logger,
    None => {
      let error = serde_json::json!({"error": "Execution logger not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match logger.create_execution_log(session_id, user_query, task_plan_id, agent_id, user_id, workspace_id).await {
      Ok(execution_id) => serde_json::json!({"execution_id": execution_id}),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 搜索执行日志
#[no_mangle]
pub extern "C" fn execution_log_search(criteria_json: *const c_char) -> *const u8 {
  let criteria_json = unsafe { CStr::from_ptr(criteria_json) }.to_string_lossy().to_string();

  let criteria: ExecutionLogSearchCriteria = match serde_json::from_str(&criteria_json) {
    Ok(criteria) => criteria,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Invalid criteria JSON: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let logger = match EXECUTION_LOGGER.get() {
    Some(logger) => logger,
    None => {
      let error = serde_json::json!({"error": "Execution logger not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match logger.search_execution_logs(criteria).await {
      Ok(logs) => serde_json::to_value(&logs).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize logs"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 获取执行日志详情
#[no_mangle]
pub extern "C" fn execution_log_get_details(execution_id: *const c_char) -> *const u8 {
  let execution_id = unsafe { CStr::from_ptr(execution_id) }.to_string_lossy().to_string();

  let logger = match EXECUTION_LOGGER.get() {
    Some(logger) => logger,
    None => {
      let error = serde_json::json!({"error": "Execution logger not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match logger.get_execution_log_with_details(&execution_id).await {
      Ok(details) => serde_json::to_value(&details).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize details"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 导出执行日志
#[no_mangle]
pub extern "C" fn execution_log_export(
  criteria_json: *const c_char,
  options_json: *const c_char,
) -> *const u8 {
  let criteria_json = unsafe { CStr::from_ptr(criteria_json) }.to_string_lossy().to_string();
  let options_json = unsafe { CStr::from_ptr(options_json) }.to_string_lossy().to_string();

  let criteria: ExecutionLogSearchCriteria = match serde_json::from_str(&criteria_json) {
    Ok(criteria) => criteria,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Invalid criteria JSON: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let options: ExecutionLogExportOptions = match serde_json::from_str(&options_json) {
    Ok(options) => options,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Invalid options JSON: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let logger = match EXECUTION_LOGGER.get() {
    Some(logger) => logger,
    None => {
      let error = serde_json::json!({"error": "Execution logger not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let result = rt.block_on(async {
    match logger.export_execution_logs(criteria, options).await {
      Ok(export_data) => serde_json::json!({"data": export_data}),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

/// 获取执行统计信息
#[no_mangle]
pub extern "C" fn execution_log_get_statistics(
  start_time: i64,
  end_time: i64,
  workspace_id: *const c_char,
) -> *const u8 {
  let workspace_id = if workspace_id.is_null() {
    None
  } else {
    Some(unsafe { CStr::from_ptr(workspace_id) }.to_string_lossy().to_string())
  };

  let logger = match EXECUTION_LOGGER.get() {
    Some(logger) => logger,
    None => {
      let error = serde_json::json!({"error": "Execution logger not initialized"});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(e) => {
      let error = serde_json::json!({"error": format!("Failed to create runtime: {}", e)});
      let bytes = error.to_string().into_bytes();
      let result = extend_front_four_bytes_into_bytes(&bytes);
      return forget_rust(result);
    }
  };

  let start_time_opt = if start_time == 0 { None } else { Some(start_time) };
  let end_time_opt = if end_time == 0 { None } else { Some(end_time) };

  let result = rt.block_on(async {
    match logger.get_execution_statistics(start_time_opt, end_time_opt, workspace_id).await {
      Ok(stats) => serde_json::to_value(&stats).unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize statistics"})),
      Err(e) => serde_json::json!({"error": e.to_string()}),
    }
  });

  let bytes = result.to_string().into_bytes();
  let result = extend_front_four_bytes_into_bytes(&bytes);
  forget_rust(result)
}

// ============================================================================
// 事件通知相关FFI函数
// ============================================================================

/// 设置任务执行进度通知端口
#[no_mangle]
pub extern "C" fn task_set_progress_port(port: i64) -> i32 {
  let orchestrator = match TASK_ORCHESTRATOR.get() {
    Some(orchestrator) => orchestrator,
    None => return -1,
  };

  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(_) => return -1,
  };

  let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ExecutionProgress>();
  
  // 设置进度接收器
  rt.block_on(async {
    orchestrator.set_progress_receiver(sender).await;
  });

  // 启动进度通知任务
  tokio::spawn(async move {
    while let Some(progress) = receiver.recv().await {
      let isolate = allo_isolate::Isolate::new(port);
      let progress_json = match serde_json::to_string(&progress) {
        Ok(json) => json,
        Err(_) => continue,
      };
      
      let _ = isolate.catch_unwind(async {
        progress_json.into_bytes()
      }).await;
    }
  });

  0
}

/// 发送自定义事件通知
#[no_mangle]
pub extern "C" fn task_send_notification(
  port: i64,
  event_type: *const c_char,
  data_json: *const c_char,
) -> i32 {
  let event_type = unsafe { CStr::from_ptr(event_type) }.to_string_lossy().to_string();
  let data_json = unsafe { CStr::from_ptr(data_json) }.to_string_lossy().to_string();

  let isolate = allo_isolate::Isolate::new(port);
  let rt = match Builder::new_current_thread().enable_all().build() {
    Ok(rt) => rt,
    Err(_) => return -1,
  };

  let result = rt.block_on(async {
    let notification = serde_json::json!({
      "event_type": event_type,
      "data": serde_json::from_str::<serde_json::Value>(&data_json).unwrap_or(serde_json::json!({})),
      "timestamp": std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
    });
    
    isolate.catch_unwind(async move {
      notification.to_string().into_bytes()
    }).await
  });

  match result {
    Ok(_) => 0,
    Err(_) => -1,
  }
}
