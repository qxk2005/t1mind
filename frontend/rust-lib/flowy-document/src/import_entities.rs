use flowy_derive::{ProtoBuf, ProtoBuf_Enum};
use std::collections::HashMap;

// ==================== 基础枚举类型 ====================

#[derive(Default, ProtoBuf_Enum)]
pub enum DocumentTypePB {
  #[default]
  Word = 0,
  Pdf = 1,
}

#[derive(Default, ProtoBuf_Enum)]
pub enum ConversionStatusPB {
  #[default]
  Pending = 0,
  Processing = 1,
  Completed = 2,
  Failed = 3,
  Cancelled = 4,
}

#[derive(Default, ProtoBuf_Enum)]
pub enum ConversionPriorityPB {
  #[default]
  Low = 0,
  Normal = 1,
  High = 2,
  Urgent = 3,
}

#[derive(Default, ProtoBuf_Enum)]
pub enum ImportElementTypePB {
  #[default]
  ImportText = 0,
  ImportImage = 1,
  ImportTable = 2,
  ImportList = 3,
  ImportHeading = 4,
  ImportParagraph = 5,
}

#[derive(Default, ProtoBuf_Enum)]
pub enum LogLevelPB {
  #[default]
  Error = 0,
  Warn = 1,
  Info = 2,
  Debug = 3,
  Trace = 4,
}

// ==================== 配置和设置 ====================

#[derive(Default, ProtoBuf)]
pub struct ConversionConfigPB {
  #[pb(index = 1)]
  pub preserve_formatting: bool,
  
  #[pb(index = 2)]
  pub extract_images: bool,
  
  #[pb(index = 3)]
  pub preserve_tables: bool,
  
  #[pb(index = 4)]
  pub max_file_size: Option<i64>,
  
  #[pb(index = 5)]
  pub timeout_seconds: Option<i64>,
}

#[derive(Default, ProtoBuf)]
pub struct DocumentImportSettingsPB {
  #[pb(index = 1)]
  pub max_concurrent_conversions: i32,
  
  #[pb(index = 2)]
  pub default_import_path: String,
  
  #[pb(index = 3)]
  pub preserve_formatting: bool,
  
  #[pb(index = 4)]
  pub extract_images: bool,
  
  #[pb(index = 5)]
  pub extract_tables: bool,
  
  #[pb(index = 6)]
  pub auto_create_folder: bool,
  
  #[pb(index = 7)]
  pub enable_progress_notifications: bool,
  
  #[pb(index = 8)]
  pub conversion_timeout_seconds: i32,
  
  #[pb(index = 9)]
  pub auto_retry_on_failure: bool,
  
  #[pb(index = 10)]
  pub max_retry_attempts: i32,
  
  #[pb(index = 11)]
  pub save_conversion_logs: bool,
  
  #[pb(index = 12)]
  pub log_retention_days: i32,
}

// ==================== 任务相关 ====================

#[derive(Default, ProtoBuf)]
pub struct ConversionTaskPB {
  #[pb(index = 1)]
  pub id: String,
  
  #[pb(index = 2)]
  pub source_path: String,
  
  #[pb(index = 3)]
  pub document_type: DocumentTypePB,
  
  #[pb(index = 4)]
  pub target_name: String,
  
  #[pb(index = 5)]
  pub config: ConversionConfigPB,
  
  #[pb(index = 6)]
  pub status: ConversionStatusPB,
  
  #[pb(index = 7)]
  pub created_at: i64,
  
  #[pb(index = 8)]
  pub started_at: Option<i64>,
  
  #[pb(index = 9)]
  pub completed_at: Option<i64>,
  
  #[pb(index = 10)]
  pub error_message: Option<String>,
  
  #[pb(index = 11)]
  pub progress: i32,
  
  #[pb(index = 12)]
  pub parent_view_id: String,
  
  #[pb(index = 13)]
  pub result_document_id: Option<String>,
  
  #[pb(index = 14)]
  pub priority: ConversionPriorityPB,
  
  #[pb(index = 15)]
  pub retry_count: i32,
}

#[derive(Default, ProtoBuf)]
pub struct ConversionResultPB {
  #[pb(index = 1)]
  pub task_id: String,
  
  #[pb(index = 2)]
  pub document_id: String,
  
  #[pb(index = 3)]
  pub document_name: String,
  
  #[pb(index = 4)]
  pub success: bool,
  
  #[pb(index = 5)]
  pub error_message: Option<String>,
  
  #[pb(index = 6)]
  pub processing_time_ms: i64,
  
  #[pb(index = 7)]
  pub converted_elements_count: i32,
  
  #[pb(index = 8)]
  pub extracted_images_count: i32,
  
  #[pb(index = 9)]
  pub extracted_tables_count: i32,
  
  #[pb(index = 10)]
  pub elements: Vec<ConvertedElementPB>,
}

#[derive(Default, ProtoBuf)]
pub struct ConvertedElementPB {
  #[pb(index = 1)]
  pub element_type: ImportElementTypePB,
  
  #[pb(index = 2)]
  pub content: String,
  
  #[pb(index = 3)]
  pub formatting: Option<String>,
  
  #[pb(index = 4)]
  pub position: String,
  
  #[pb(index = 5)]
  pub metadata: String,
}

// ==================== 队列管理 ====================

#[derive(Default, ProtoBuf)]
pub struct ConversionQueueItemPB {
  #[pb(index = 1)]
  pub task_id: String,
  
  #[pb(index = 2)]
  pub priority: ConversionPriorityPB,
  
  #[pb(index = 3)]
  pub created_at: i64,
  
  #[pb(index = 4)]
  pub retry_count: i32,
}

#[derive(Default, ProtoBuf)]
pub struct QueueStatusPB {
  #[pb(index = 1)]
  pub pending_tasks: i32,
  
  #[pb(index = 2)]
  pub processing_tasks: i32,
  
  #[pb(index = 3)]
  pub completed_tasks: i32,
  
  #[pb(index = 4)]
  pub failed_tasks: i32,
  
  #[pb(index = 5)]
  pub max_concurrent_tasks: i32,
  
  #[pb(index = 6)]
  pub paused: bool,
}

// ==================== 进度更新 ====================

#[derive(Default, ProtoBuf)]
pub struct ProgressUpdatePB {
  #[pb(index = 1)]
  pub task_id: String,
  
  #[pb(index = 2)]
  pub progress: i32,
  
  #[pb(index = 3)]
  pub status: ConversionStatusPB,
  
  #[pb(index = 4)]
  pub error_message: Option<String>,
  
  #[pb(index = 5)]
  pub timestamp: i64,
}

// ==================== 请求和响应 ====================

#[derive(Default, ProtoBuf)]
pub struct CreateConversionTaskPB {
  #[pb(index = 1)]
  pub source_path: String,
  
  #[pb(index = 2)]
  pub document_type: DocumentTypePB,
  
  #[pb(index = 3)]
  pub target_name: String,
  
  #[pb(index = 4)]
  pub config: ConversionConfigPB,
  
  #[pb(index = 5)]
  pub parent_view_id: String,
  
  #[pb(index = 6)]
  pub priority: ConversionPriorityPB,
}

#[derive(Default, ProtoBuf)]
pub struct CreateConversionTaskResponsePB {
  #[pb(index = 1)]
  pub task_id: String,
  
  #[pb(index = 2)]
  pub success: bool,
  
  #[pb(index = 3)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct GetConversionTaskPB {
  #[pb(index = 1)]
  pub task_id: String,
}

#[derive(Default, ProtoBuf)]
pub struct GetConversionTaskResponsePB {
  #[pb(index = 1)]
  pub task: Option<ConversionTaskPB>,
  
  #[pb(index = 2)]
  pub success: bool,
  
  #[pb(index = 3)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct GetConversionTasksPB {
  #[pb(index = 1)]
  pub user_id: String,
  
  #[pb(index = 2)]
  pub status_filter: Option<ConversionStatusPB>,
  
  #[pb(index = 3)]
  pub offset: i32,
  
  #[pb(index = 4)]
  pub limit: i32,
}

#[derive(Default, ProtoBuf)]
pub struct GetConversionTasksResponsePB {
  #[pb(index = 1)]
  pub tasks: Vec<ConversionTaskPB>,
  
  #[pb(index = 2)]
  pub total_count: i32,
  
  #[pb(index = 3)]
  pub success: bool,
  
  #[pb(index = 4)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct CancelConversionTaskPB {
  #[pb(index = 1)]
  pub task_id: String,
  
  #[pb(index = 2)]
  pub user_id: String,
}

#[derive(Default, ProtoBuf)]
pub struct CancelConversionTaskResponsePB {
  #[pb(index = 1)]
  pub success: bool,
  
  #[pb(index = 2)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct GetQueueStatusPB {
  #[pb(index = 1)]
  pub user_id: String,
}

#[derive(Default, ProtoBuf)]
pub struct GetQueueStatusResponsePB {
  #[pb(index = 1)]
  pub status: QueueStatusPB,
  
  #[pb(index = 2)]
  pub success: bool,
  
  #[pb(index = 3)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct PauseResumeQueuePB {
  #[pb(index = 1)]
  pub user_id: String,
  
  #[pb(index = 2)]
  pub pause: bool,
}

#[derive(Default, ProtoBuf)]
pub struct PauseResumeQueueResponsePB {
  #[pb(index = 1)]
  pub success: bool,
  
  #[pb(index = 2)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct UpdateDocumentImportSettingsPB {
  #[pb(index = 1)]
  pub settings: DocumentImportSettingsPB,
  
  #[pb(index = 2)]
  pub user_id: String,
}

#[derive(Default, ProtoBuf)]
pub struct UpdateImportSettingsResponsePB {
  #[pb(index = 1)]
  pub success: bool,
  
  #[pb(index = 2)]
  pub error_message: Option<String>,
}

#[derive(Default, ProtoBuf)]
pub struct GetImportSettingsPB {
  #[pb(index = 1)]
  pub user_id: String,
}

#[derive(Default, ProtoBuf)]
pub struct GetImportSettingsResponsePB {
  #[pb(index = 1)]
  pub settings: DocumentImportSettingsPB,
  
  #[pb(index = 2)]
  pub success: bool,
  
  #[pb(index = 3)]
  pub error_message: Option<String>,
}

// ==================== 日志相关 ====================

#[derive(Default, ProtoBuf)]
pub struct ConversionLogEntryPB {
  #[pb(index = 1)]
  pub timestamp: i64,
  
  #[pb(index = 2)]
  pub level: LogLevelPB,
  
  #[pb(index = 3)]
  pub message: String,
  
  #[pb(index = 4)]
  pub task_id: String,
}

#[derive(Default, ProtoBuf)]
pub struct GetConversionLogsPB {
  #[pb(index = 1)]
  pub task_id: String,
  
  #[pb(index = 2)]
  pub user_id: String,
  
  #[pb(index = 3)]
  pub level_filter: Option<LogLevelPB>,
  
  #[pb(index = 4)]
  pub offset: i32,
  
  #[pb(index = 5)]
  pub limit: i32,
}

#[derive(Default, ProtoBuf)]
pub struct GetConversionLogsResponsePB {
  #[pb(index = 1)]
  pub logs: Vec<ConversionLogEntryPB>,
  
  #[pb(index = 2)]
  pub total_count: i32,
  
  #[pb(index = 3)]
  pub success: bool,
  
  #[pb(index = 4)]
  pub error_message: Option<String>,
}
