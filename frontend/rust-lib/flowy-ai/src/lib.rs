mod event_handler;
pub mod event_map;

pub mod ai_manager;
mod chat;
mod completion;
pub mod entities;
pub mod execution_logger;
pub mod task_orchestrator;
pub mod local_ai;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux", target_os = "android"))]
pub mod mcp;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux", target_os = "android"))]
pub use mcp::streamable_http::streamable_http_tools_list;

#[cfg(feature = "ai-tool")]
mod ai_tool;
pub mod embeddings;
pub use embeddings::store::SqliteVectorStore;

mod middleware;
mod model_select;
#[cfg(test)]
mod model_select_test;
pub mod notification;
pub mod offline;
mod protobuf;
mod search;
mod stream_message;
