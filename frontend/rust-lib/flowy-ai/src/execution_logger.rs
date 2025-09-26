use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::upsert::excluded;
use flowy_sqlite::{
  AsChangeset, DBConnection, ExpressionMethods, Identifiable, Insertable, OptionalExtension,
  Queryable, diesel, TextExpressionMethods,
  query_dsl::*,
  schema::{
    execution_log_table, execution_step_table, execution_reference_table, mcp_tool_info_table,
    execution_log_table::dsl as log_dsl,
    execution_step_table::dsl as step_dsl,
    execution_reference_table::dsl as ref_dsl,
    mcp_tool_info_table::dsl as tool_dsl,
  },
};
use lib_infra::util::timestamp;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug};
use uuid::Uuid;

/// 执行日志状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExecutionLogStatus {
  Initialized = 0,
  Preparing = 1,
  Running = 2,
  Paused = 3,
  Completed = 4,
  Failed = 5,
  Cancelled = 6,
  Timeout = 7,
}

impl From<i32> for ExecutionLogStatus {
  fn from(value: i32) -> Self {
    match value {
      0 => ExecutionLogStatus::Initialized,
      1 => ExecutionLogStatus::Preparing,
      2 => ExecutionLogStatus::Running,
      3 => ExecutionLogStatus::Paused,
      4 => ExecutionLogStatus::Completed,
      5 => ExecutionLogStatus::Failed,
      6 => ExecutionLogStatus::Cancelled,
      7 => ExecutionLogStatus::Timeout,
      _ => ExecutionLogStatus::Initialized,
    }
  }
}

impl ExecutionLogStatus {
  pub fn is_running(&self) -> bool {
    matches!(self, ExecutionLogStatus::Preparing | ExecutionLogStatus::Running)
  }

  pub fn is_finished(&self) -> bool {
    matches!(
      self,
      ExecutionLogStatus::Completed
        | ExecutionLogStatus::Failed
        | ExecutionLogStatus::Cancelled
        | ExecutionLogStatus::Timeout
    )
  }

  pub fn is_successful(&self) -> bool {
    matches!(self, ExecutionLogStatus::Completed)
  }

  pub fn can_retry(&self) -> bool {
    matches!(
      self,
      ExecutionLogStatus::Failed | ExecutionLogStatus::Timeout | ExecutionLogStatus::Cancelled
    )
  }
}

/// 执行步骤状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExecutionStepStatus {
  Pending = 0,
  Executing = 1,
  Success = 2,
  Error = 3,
  Skipped = 4,
  Timeout = 5,
  Cancelled = 6,
}

impl From<i32> for ExecutionStepStatus {
  fn from(value: i32) -> Self {
    match value {
      0 => ExecutionStepStatus::Pending,
      1 => ExecutionStepStatus::Executing,
      2 => ExecutionStepStatus::Success,
      3 => ExecutionStepStatus::Error,
      4 => ExecutionStepStatus::Skipped,
      5 => ExecutionStepStatus::Timeout,
      6 => ExecutionStepStatus::Cancelled,
      _ => ExecutionStepStatus::Pending,
    }
  }
}

impl ExecutionStepStatus {
  pub fn is_finished(&self) -> bool {
    matches!(
      self,
      ExecutionStepStatus::Success
        | ExecutionStepStatus::Error
        | ExecutionStepStatus::Skipped
        | ExecutionStepStatus::Timeout
        | ExecutionStepStatus::Cancelled
    )
  }

  pub fn is_successful(&self) -> bool {
    matches!(self, ExecutionStepStatus::Success)
  }

  pub fn can_retry(&self) -> bool {
    matches!(self, ExecutionStepStatus::Error | ExecutionStepStatus::Timeout)
  }
}

/// 错误类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExecutionErrorType {
  Network = 0,
  Authentication = 1,
  Authorization = 2,
  InvalidParameters = 3,
  ToolUnavailable = 4,
  Timeout = 5,
  System = 6,
  UserCancelled = 7,
  Dependency = 8,
  Configuration = 9,
  Unknown = 10,
}

impl From<i32> for ExecutionErrorType {
  fn from(value: i32) -> Self {
    match value {
      0 => ExecutionErrorType::Network,
      1 => ExecutionErrorType::Authentication,
      2 => ExecutionErrorType::Authorization,
      3 => ExecutionErrorType::InvalidParameters,
      4 => ExecutionErrorType::ToolUnavailable,
      5 => ExecutionErrorType::Timeout,
      6 => ExecutionErrorType::System,
      7 => ExecutionErrorType::UserCancelled,
      8 => ExecutionErrorType::Dependency,
      9 => ExecutionErrorType::Configuration,
      10 => ExecutionErrorType::Unknown,
      _ => ExecutionErrorType::Unknown,
    }
  }
}

impl ExecutionErrorType {
  pub fn description(&self) -> &'static str {
    match self {
      ExecutionErrorType::Network => "网络连接错误",
      ExecutionErrorType::Authentication => "身份认证失败",
      ExecutionErrorType::Authorization => "权限不足",
      ExecutionErrorType::InvalidParameters => "参数无效",
      ExecutionErrorType::ToolUnavailable => "工具不可用",
      ExecutionErrorType::Timeout => "执行超时",
      ExecutionErrorType::System => "系统错误",
      ExecutionErrorType::UserCancelled => "用户取消",
      ExecutionErrorType::Dependency => "依赖错误",
      ExecutionErrorType::Configuration => "配置错误",
      ExecutionErrorType::Unknown => "未知错误",
    }
  }

  pub fn can_retry(&self) -> bool {
    matches!(
      self,
      ExecutionErrorType::Network
        | ExecutionErrorType::Timeout
        | ExecutionErrorType::System
        | ExecutionErrorType::ToolUnavailable
    )
  }
}

/// MCP工具状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum McpToolStatus {
  Unknown = 0,
  Available = 1,
  Unavailable = 2,
  Connecting = 3,
  Connected = 4,
  Disconnected = 5,
  Error = 6,
}

impl From<i32> for McpToolStatus {
  fn from(value: i32) -> Self {
    match value {
      0 => McpToolStatus::Unknown,
      1 => McpToolStatus::Available,
      2 => McpToolStatus::Unavailable,
      3 => McpToolStatus::Connecting,
      4 => McpToolStatus::Connected,
      5 => McpToolStatus::Disconnected,
      6 => McpToolStatus::Error,
      _ => McpToolStatus::Unknown,
    }
  }
}

impl McpToolStatus {
  pub fn is_available(&self) -> bool {
    matches!(self, McpToolStatus::Available | McpToolStatus::Connected)
  }

  pub fn needs_reconnection(&self) -> bool {
    matches!(
      self,
      McpToolStatus::Unavailable | McpToolStatus::Disconnected | McpToolStatus::Error
    )
  }
}

/// 引用类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExecutionReferenceType {
  Document = 0,
  Webpage = 1,
  Api = 2,
  Database = 3,
  File = 4,
  Image = 5,
  Video = 6,
  Other = 7,
}

impl From<i32> for ExecutionReferenceType {
  fn from(value: i32) -> Self {
    match value {
      0 => ExecutionReferenceType::Document,
      1 => ExecutionReferenceType::Webpage,
      2 => ExecutionReferenceType::Api,
      3 => ExecutionReferenceType::Database,
      4 => ExecutionReferenceType::File,
      5 => ExecutionReferenceType::Image,
      6 => ExecutionReferenceType::Video,
      7 => ExecutionReferenceType::Other,
      _ => ExecutionReferenceType::Other,
    }
  }
}

impl ExecutionReferenceType {
  pub fn display_name(&self) -> &'static str {
    match self {
      ExecutionReferenceType::Document => "文档",
      ExecutionReferenceType::Webpage => "网页",
      ExecutionReferenceType::Api => "API",
      ExecutionReferenceType::Database => "数据库",
      ExecutionReferenceType::File => "文件",
      ExecutionReferenceType::Image => "图片",
      ExecutionReferenceType::Video => "视频",
      ExecutionReferenceType::Other => "其他",
    }
  }
}

/// 执行日志数据库模型
#[derive(Clone, Default, Queryable, Insertable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = execution_log_table)]
#[diesel(primary_key(id))]
pub struct ExecutionLogTable {
  pub id: String,
  pub session_id: String,
  pub task_plan_id: Option<String>,
  pub user_query: String,
  pub start_time: i64,
  pub end_time: Option<i64>,
  pub status: i32,
  pub error_message: Option<String>,
  pub error_type: Option<i32>,
  pub agent_id: Option<String>,
  pub user_id: Option<String>,
  pub workspace_id: Option<String>,
  pub total_steps: i32,
  pub completed_steps: i32,
  pub failed_steps: i32,
  pub skipped_steps: i32,
  pub context: String,
  pub result_summary: Option<String>,
  pub used_mcp_tools: String,
  pub tags: String,
  pub retry_count: i32,
  pub max_retries: i32,
  pub parent_execution_id: Option<String>,
  pub child_execution_ids: String,
  pub created_at: i64,
  pub updated_at: i64,
}

impl ExecutionLogTable {
  pub fn new(
    id: String,
    session_id: String,
    user_query: String,
    task_plan_id: Option<String>,
    agent_id: Option<String>,
    user_id: Option<String>,
    workspace_id: Option<String>,
  ) -> Self {
    let now = timestamp();
    Self {
      id,
      session_id,
      task_plan_id,
      user_query,
      start_time: now,
      end_time: None,
      status: ExecutionLogStatus::Initialized as i32,
      error_message: None,
      error_type: None,
      agent_id,
      user_id,
      workspace_id,
      total_steps: 0,
      completed_steps: 0,
      failed_steps: 0,
      skipped_steps: 0,
      context: "{}".to_string(),
      result_summary: None,
      used_mcp_tools: "[]".to_string(),
      tags: "[]".to_string(),
      retry_count: 0,
      max_retries: 3,
      parent_execution_id: None,
      child_execution_ids: "[]".to_string(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn get_status(&self) -> ExecutionLogStatus {
    ExecutionLogStatus::from(self.status)
  }

  pub fn get_error_type(&self) -> Option<ExecutionErrorType> {
    self.error_type.map(ExecutionErrorType::from)
  }

  pub fn get_context(&self) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&self.context)
  }

  pub fn get_used_mcp_tools(&self) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(&self.used_mcp_tools)
  }

  pub fn get_tags(&self) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(&self.tags)
  }

  pub fn get_child_execution_ids(&self) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(&self.child_execution_ids)
  }
}

/// 执行步骤数据库模型
#[derive(Clone, Default, Queryable, Insertable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = execution_step_table)]
#[diesel(primary_key(id))]
pub struct ExecutionStepTable {
  pub id: String,
  pub execution_log_id: String,
  pub name: String,
  pub description: String,
  pub mcp_tool_id: String,
  pub mcp_tool_name: String,
  pub mcp_tool_config: String,
  pub input_parameters: String,
  pub output_result: Option<String>,
  pub execution_time_ms: i32,
  pub status: i32,
  pub start_time: Option<i64>,
  pub end_time: Option<i64>,
  pub error_message: Option<String>,
  pub error_type: Option<i32>,
  pub error_stack: Option<String>,
  pub step_order: i32,
  pub retry_count: i32,
  pub max_retries: i32,
  pub dependencies: String,
  pub tags: String,
  pub metadata: String,
  pub can_skip: bool,
  pub is_critical: bool,
  pub created_at: i64,
  pub updated_at: i64,
}

impl ExecutionStepTable {
  pub fn new(
    id: String,
    execution_log_id: String,
    name: String,
    description: String,
    mcp_tool_id: String,
    mcp_tool_name: String,
    step_order: i32,
  ) -> Self {
    let now = timestamp();
    Self {
      id,
      execution_log_id,
      name,
      description,
      mcp_tool_id,
      mcp_tool_name,
      mcp_tool_config: "{}".to_string(),
      input_parameters: "{}".to_string(),
      output_result: None,
      execution_time_ms: 0,
      status: ExecutionStepStatus::Pending as i32,
      start_time: None,
      end_time: None,
      error_message: None,
      error_type: None,
      error_stack: None,
      step_order,
      retry_count: 0,
      max_retries: 3,
      dependencies: "[]".to_string(),
      tags: "[]".to_string(),
      metadata: "{}".to_string(),
      can_skip: false,
      is_critical: false,
      created_at: now,
      updated_at: now,
    }
  }

  pub fn get_status(&self) -> ExecutionStepStatus {
    ExecutionStepStatus::from(self.status)
  }

  pub fn get_error_type(&self) -> Option<ExecutionErrorType> {
    self.error_type.map(ExecutionErrorType::from)
  }

  pub fn get_input_parameters(&self) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&self.input_parameters)
  }

  pub fn get_output_result(&self) -> Result<Option<Value>, serde_json::Error> {
    match &self.output_result {
      Some(result) => serde_json::from_str(result).map(Some),
      None => Ok(None),
    }
  }

  pub fn get_dependencies(&self) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(&self.dependencies)
  }

  pub fn get_tags(&self) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(&self.tags)
  }

  pub fn get_metadata(&self) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&self.metadata)
  }
}

/// 执行引用数据库模型
#[derive(Clone, Default, Queryable, Insertable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = execution_reference_table)]
#[diesel(primary_key(id))]
pub struct ExecutionReferenceTable {
  pub id: String,
  pub execution_step_id: String,
  pub reference_type: i32,
  pub title: String,
  pub content: Option<String>,
  pub url: Option<String>,
  pub source: Option<String>,
  pub timestamp: i64,
  pub metadata: String,
  pub relevance_score: f64,
  pub created_at: i64,
}

impl ExecutionReferenceTable {
  pub fn new(
    id: String,
    execution_step_id: String,
    reference_type: ExecutionReferenceType,
    title: String,
    content: Option<String>,
    url: Option<String>,
    source: Option<String>,
    relevance_score: f64,
  ) -> Self {
    let now = timestamp();
    Self {
      id,
      execution_step_id,
      reference_type: reference_type as i32,
      title,
      content,
      url,
      source,
      timestamp: now,
      metadata: "{}".to_string(),
      relevance_score,
      created_at: now,
    }
  }

  pub fn get_reference_type(&self) -> ExecutionReferenceType {
    ExecutionReferenceType::from(self.reference_type)
  }

  pub fn get_metadata(&self) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&self.metadata)
  }
}

/// MCP工具信息数据库模型
#[derive(Clone, Default, Queryable, Insertable, Identifiable, Debug)]
#[diesel(table_name = mcp_tool_info_table)]
#[diesel(primary_key(id))]
pub struct McpToolInfoTable {
  pub id: String,
  pub name: String,
  pub display_name: Option<String>,
  pub description: String,
  pub version: String,
  pub provider: String,
  pub category: String,
  pub status: i32,
  pub config: String,
  pub schema: String,
  pub requires_auth: bool,
  pub auth_config: Option<String>,
  pub icon_url: Option<String>,
  pub documentation_url: Option<String>,
  pub last_checked: Option<i64>,
  pub last_used: Option<i64>,
  pub usage_count: i32,
  pub success_count: i32,
  pub failure_count: i32,
  pub average_execution_time_ms: i32,
  pub created_at: i64,
  pub updated_at: i64,
}

impl McpToolInfoTable {
  pub fn new(id: String, name: String, description: String) -> Self {
    let now = timestamp();
    Self {
      id,
      name,
      display_name: None,
      description,
      version: "".to_string(),
      provider: "".to_string(),
      category: "".to_string(),
      status: McpToolStatus::Unknown as i32,
      config: "{}".to_string(),
      schema: "{}".to_string(),
      requires_auth: false,
      auth_config: None,
      icon_url: None,
      documentation_url: None,
      last_checked: None,
      last_used: None,
      usage_count: 0,
      success_count: 0,
      failure_count: 0,
      average_execution_time_ms: 0,
      created_at: now,
      updated_at: now,
    }
  }

  pub fn get_status(&self) -> McpToolStatus {
    McpToolStatus::from(self.status)
  }

  pub fn get_config(&self) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&self.config)
  }

  pub fn get_schema(&self) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&self.schema)
  }

  pub fn get_auth_config(&self) -> Result<Option<Value>, serde_json::Error> {
    match &self.auth_config {
      Some(config) => serde_json::from_str(config).map(Some),
      None => Ok(None),
    }
  }
}

/// 执行日志搜索条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionLogSearchCriteria {
  pub session_id: Option<String>,
  pub agent_id: Option<String>,
  pub user_id: Option<String>,
  pub workspace_id: Option<String>,
  pub status: Option<ExecutionLogStatus>,
  pub error_type: Option<ExecutionErrorType>,
  pub start_time: Option<i64>,
  pub end_time: Option<i64>,
  pub keyword: Option<String>,
  pub mcp_tool_name: Option<String>,
  pub tags: Option<Vec<String>>,
  pub limit: i64,
  pub offset: i64,
}

impl ExecutionLogSearchCriteria {
  pub fn new() -> Self {
    Self {
      limit: 100,
      offset: 0,
      ..Default::default()
    }
  }

  pub fn with_session_id(mut self, session_id: String) -> Self {
    self.session_id = Some(session_id);
    self
  }

  pub fn with_status(mut self, status: ExecutionLogStatus) -> Self {
    self.status = Some(status);
    self
  }

  pub fn with_limit(mut self, limit: i64) -> Self {
    self.limit = limit;
    self
  }

  pub fn with_offset(mut self, offset: i64) -> Self {
    self.offset = offset;
    self
  }
}

/// 执行日志导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionLogExportFormat {
  Json,
  Csv,
  Excel,
  Pdf,
  Html,
  Text,
}

impl ExecutionLogExportFormat {
  pub fn extension(&self) -> &'static str {
    match self {
      ExecutionLogExportFormat::Json => "json",
      ExecutionLogExportFormat::Csv => "csv",
      ExecutionLogExportFormat::Excel => "xlsx",
      ExecutionLogExportFormat::Pdf => "pdf",
      ExecutionLogExportFormat::Html => "html",
      ExecutionLogExportFormat::Text => "txt",
    }
  }

  pub fn mime_type(&self) -> &'static str {
    match self {
      ExecutionLogExportFormat::Json => "application/json",
      ExecutionLogExportFormat::Csv => "text/csv",
      ExecutionLogExportFormat::Excel => {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
      },
      ExecutionLogExportFormat::Pdf => "application/pdf",
      ExecutionLogExportFormat::Html => "text/html",
      ExecutionLogExportFormat::Text => "text/plain",
    }
  }
}

/// 执行日志导出选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogExportOptions {
  pub format: ExecutionLogExportFormat,
  pub include_steps: bool,
  pub include_references: bool,
  pub include_metadata: bool,
  pub include_error_details: bool,
  pub date_format: String,
  pub max_records: Option<usize>,
}

impl Default for ExecutionLogExportOptions {
  fn default() -> Self {
    Self {
      format: ExecutionLogExportFormat::Json,
      include_steps: true,
      include_references: true,
      include_metadata: false,
      include_error_details: true,
      date_format: "yyyy-MM-dd HH:mm:ss".to_string(),
      max_records: None,
    }
  }
}

/// 执行统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionStatistics {
  pub total_executions: i64,
  pub successful_executions: i64,
  pub failed_executions: i64,
  pub cancelled_executions: i64,
  pub average_execution_time_ms: i64,
  pub min_execution_time_ms: i64,
  pub max_execution_time_ms: i64,
  pub most_used_tools: Vec<String>,
  pub common_error_types: Vec<ExecutionErrorType>,
  pub period_start: Option<i64>,
  pub period_end: Option<i64>,
}

/// ExecutionLogger - 执行日志记录器
pub struct ExecutionLogger {
  user_service: Arc<dyn flowy_ai_pub::user_service::AIUserService>,
}

impl ExecutionLogger {
  pub fn new(user_service: Arc<dyn flowy_ai_pub::user_service::AIUserService>) -> Self {
    Self { user_service }
  }

  fn get_connection(&self) -> FlowyResult<DBConnection> {
    let uid = self.user_service.user_id()?;
    self.user_service.sqlite_connection(uid)
  }

  /// 创建新的执行日志
  pub async fn create_execution_log(
    &self,
    session_id: String,
    user_query: String,
    task_plan_id: Option<String>,
    agent_id: Option<String>,
    user_id: Option<String>,
    workspace_id: Option<String>,
  ) -> FlowyResult<String> {
    let execution_id = Uuid::new_v4().to_string();
    let log = ExecutionLogTable::new(
      execution_id.clone(),
      session_id,
      user_query,
      task_plan_id,
      agent_id,
      user_id,
      workspace_id,
    );

    let mut conn = self.get_connection()?;
    diesel::insert_into(execution_log_table::table)
      .values(&log)
      .execute(&mut *conn)?;

    info!("Created execution log: {}", execution_id);
    Ok(execution_id)
  }

  /// 更新执行日志状态
  pub async fn update_execution_status(
    &self,
    execution_id: &str,
    status: ExecutionLogStatus,
    error_message: Option<String>,
    error_type: Option<ExecutionErrorType>,
  ) -> FlowyResult<()> {
    let mut conn = self.get_connection()?;
    let now = timestamp();

    let mut changeset = ExecutionLogChangeset {
      id: execution_id.to_string(),
      status: Some(status as i32),
      updated_at: Some(now),
      ..Default::default()
    };

    if status.is_finished() {
      changeset.end_time = Some(Some(now));
    }

    if let Some(msg) = error_message {
      changeset.error_message = Some(Some(msg));
    }

    if let Some(err_type) = error_type {
      changeset.error_type = Some(Some(err_type as i32));
    }

    diesel::update(execution_log_table::table.filter(log_dsl::id.eq(execution_id)))
      .set(&changeset)
      .execute(&mut *conn)?;

    debug!("Updated execution log status: {} -> {:?}", execution_id, status);
    Ok(())
  }

  /// 添加执行步骤
  pub async fn add_execution_step(
    &self,
    execution_id: &str,
    name: String,
    description: String,
    mcp_tool_id: String,
    mcp_tool_name: String,
    step_order: i32,
  ) -> FlowyResult<String> {
    let step_id = Uuid::new_v4().to_string();
    let step = ExecutionStepTable::new(
      step_id.clone(),
      execution_id.to_string(),
      name,
      description,
      mcp_tool_id,
      mcp_tool_name,
      step_order,
    );

    let mut conn = self.get_connection()?;
    diesel::insert_into(execution_step_table::table)
      .values(&step)
      .execute(&mut *conn)?;

    // 更新执行日志的总步骤数
    diesel::update(execution_log_table::table.filter(log_dsl::id.eq(execution_id)))
      .set((
        execution_log_table::total_steps.eq(execution_log_table::total_steps + 1),
        execution_log_table::updated_at.eq(timestamp()),
      ))
      .execute(&mut *conn)?;

    debug!("Added execution step: {} for log: {}", step_id, execution_id);
    Ok(step_id)
  }

  /// 更新执行步骤状态
  pub async fn update_step_status(
    &self,
    step_id: &str,
    status: ExecutionStepStatus,
    execution_time_ms: Option<i32>,
    output_result: Option<Value>,
    error_message: Option<String>,
    error_type: Option<ExecutionErrorType>,
  ) -> FlowyResult<()> {
    let mut conn = self.get_connection()?;
    let now = timestamp();

    let mut changeset = ExecutionStepChangeset {
      id: step_id.to_string(),
      status: Some(status as i32),
      updated_at: Some(now),
      ..Default::default()
    };

    if status == ExecutionStepStatus::Executing {
      changeset.start_time = Some(Some(now));
    } else if status.is_finished() {
      changeset.end_time = Some(Some(now));
    }

    if let Some(time_ms) = execution_time_ms {
      changeset.execution_time_ms = Some(time_ms);
    }

    if let Some(result) = output_result {
      changeset.output_result = Some(Some(serde_json::to_string(&result).unwrap_or_default()));
    }

    if let Some(msg) = error_message {
      changeset.error_message = Some(Some(msg));
    }

    if let Some(err_type) = error_type {
      changeset.error_type = Some(Some(err_type as i32));
    }

    diesel::update(execution_step_table::table.filter(step_dsl::id.eq(step_id)))
      .set(&changeset)
      .execute(&mut *conn)?;

    // 更新执行日志的步骤计数
    if status.is_finished() {
      let step: ExecutionStepTable = execution_step_table::table
        .filter(step_dsl::id.eq(step_id))
        .first(&mut *conn)?;

      let execution_id = &step.execution_log_id;
      match status {
        ExecutionStepStatus::Success => {
          diesel::update(execution_log_table::table.filter(log_dsl::id.eq(execution_id)))
            .set((
              execution_log_table::completed_steps.eq(execution_log_table::completed_steps + 1),
              execution_log_table::updated_at.eq(now),
            ))
            .execute(&mut *conn)?;
        },
        ExecutionStepStatus::Error => {
          diesel::update(execution_log_table::table.filter(log_dsl::id.eq(execution_id)))
            .set((
              execution_log_table::failed_steps.eq(execution_log_table::failed_steps + 1),
              execution_log_table::updated_at.eq(now),
            ))
            .execute(&mut *conn)?;
        },
        ExecutionStepStatus::Skipped => {
          diesel::update(execution_log_table::table.filter(log_dsl::id.eq(execution_id)))
            .set((
              execution_log_table::skipped_steps.eq(execution_log_table::skipped_steps + 1),
              execution_log_table::updated_at.eq(now),
            ))
            .execute(&mut *conn)?;
        },
        _ => {},
      }
    }

    debug!("Updated execution step status: {} -> {:?}", step_id, status);
    Ok(())
  }

  /// 添加执行引用
  pub async fn add_execution_reference(
    &self,
    step_id: &str,
    reference_type: ExecutionReferenceType,
    title: String,
    content: Option<String>,
    url: Option<String>,
    source: Option<String>,
    relevance_score: f64,
  ) -> FlowyResult<String> {
    let reference_id = Uuid::new_v4().to_string();
    let reference = ExecutionReferenceTable::new(
      reference_id.clone(),
      step_id.to_string(),
      reference_type,
      title,
      content,
      url,
      source,
      relevance_score,
    );

    let mut conn = self.get_connection()?;
    diesel::insert_into(execution_reference_table::table)
      .values(&reference)
      .execute(&mut *conn)?;

    debug!("Added execution reference: {} for step: {}", reference_id, step_id);
    Ok(reference_id)
  }

  /// 查询执行日志
  pub async fn search_execution_logs(
    &self,
    criteria: ExecutionLogSearchCriteria,
  ) -> FlowyResult<Vec<ExecutionLogTable>> {
    let mut conn = self.get_connection()?;
    let mut query = execution_log_table::table.into_boxed();

    if let Some(session_id) = &criteria.session_id {
      query = query.filter(log_dsl::session_id.eq(session_id));
    }

    if let Some(agent_id) = &criteria.agent_id {
      query = query.filter(log_dsl::agent_id.eq(agent_id));
    }

    if let Some(user_id) = &criteria.user_id {
      query = query.filter(log_dsl::user_id.eq(user_id));
    }

    if let Some(workspace_id) = &criteria.workspace_id {
      query = query.filter(log_dsl::workspace_id.eq(workspace_id));
    }

    if let Some(status) = criteria.status {
      query = query.filter(log_dsl::status.eq(status as i32));
    }

    if let Some(error_type) = criteria.error_type {
      query = query.filter(log_dsl::error_type.eq(error_type as i32));
    }

    if let Some(start_time) = criteria.start_time {
      query = query.filter(log_dsl::start_time.ge(start_time));
    }

    if let Some(end_time) = criteria.end_time {
      query = query.filter(log_dsl::start_time.le(end_time));
    }

    if let Some(keyword) = &criteria.keyword {
      query = query.filter(log_dsl::user_query.like(format!("%{}%", keyword)));
    }

    let logs = query
      .order(log_dsl::created_at.desc())
      .limit(criteria.limit)
      .offset(criteria.offset)
      .load::<ExecutionLogTable>(&mut *conn)?;

    Ok(logs)
  }

  /// 获取执行日志详情（包含步骤和引用）
  pub async fn get_execution_log_with_details(
    &self,
    execution_id: &str,
  ) -> FlowyResult<Option<(ExecutionLogTable, Vec<(ExecutionStepTable, Vec<ExecutionReferenceTable>)>)>> {
    let mut conn = self.get_connection()?;

    // 获取执行日志
    let log: Option<ExecutionLogTable> = execution_log_table::table
      .filter(log_dsl::id.eq(execution_id))
      .first(&mut *conn)
      .optional()?;

    let Some(log) = log else {
      return Ok(None);
    };

    // 获取执行步骤
    let steps: Vec<ExecutionStepTable> = execution_step_table::table
      .filter(step_dsl::execution_log_id.eq(execution_id))
      .order(step_dsl::step_order.asc())
      .load(&mut *conn)?;

    // 获取每个步骤的引用
    let mut steps_with_references = Vec::new();
    for step in steps {
      let references: Vec<ExecutionReferenceTable> = execution_reference_table::table
        .filter(ref_dsl::execution_step_id.eq(&step.id))
        .order(ref_dsl::relevance_score.desc())
        .load(&mut *conn)?;

      steps_with_references.push((step, references));
    }

    Ok(Some((log, steps_with_references)))
  }

  /// 获取执行统计信息
  pub async fn get_execution_statistics(
    &self,
    start_time: Option<i64>,
    end_time: Option<i64>,
    workspace_id: Option<String>,
  ) -> FlowyResult<ExecutionStatistics> {
    let mut conn = self.get_connection()?;
    let mut query = execution_log_table::table.into_boxed();

    if let Some(start) = start_time {
      query = query.filter(log_dsl::start_time.ge(start));
    }

    if let Some(end) = end_time {
      query = query.filter(log_dsl::start_time.le(end));
    }

    if let Some(workspace_id) = &workspace_id {
      query = query.filter(log_dsl::workspace_id.eq(workspace_id));
    }

    let logs: Vec<ExecutionLogTable> = query.load(&mut *conn)?;

    let total_executions = logs.len() as i64;
    let successful_executions = logs
      .iter()
      .filter(|log| log.get_status().is_successful())
      .count() as i64;
    let failed_executions = logs
      .iter()
      .filter(|log| log.get_status() == ExecutionLogStatus::Failed)
      .count() as i64;
    let cancelled_executions = logs
      .iter()
      .filter(|log| log.get_status() == ExecutionLogStatus::Cancelled)
      .count() as i64;

    // 计算执行时间统计
    let execution_times: Vec<i64> = logs
      .iter()
      .filter_map(|log| {
        log.end_time.map(|end| end - log.start_time)
      })
      .collect();

    let (average_time, min_time, max_time) = if execution_times.is_empty() {
      (0, 0, 0)
    } else {
      let sum: i64 = execution_times.iter().sum();
      let avg = sum / execution_times.len() as i64;
      let min = *execution_times.iter().min().unwrap_or(&0);
      let max = *execution_times.iter().max().unwrap_or(&0);
      (avg, min, max)
    };

    // 统计最常用的工具
    let mut tool_usage: HashMap<String, i32> = HashMap::new();
    for log in &logs {
      if let Ok(tools) = log.get_used_mcp_tools() {
        for tool in tools {
          *tool_usage.entry(tool).or_insert(0) += 1;
        }
      }
    }

    let mut most_used_tools: Vec<(String, i32)> = tool_usage.into_iter().collect();
    most_used_tools.sort_by(|a, b| b.1.cmp(&a.1));
    let most_used_tools: Vec<String> = most_used_tools
      .into_iter()
      .take(10)
      .map(|(tool, _)| tool)
      .collect();

    // 统计常见错误类型
    let mut error_counts: HashMap<ExecutionErrorType, i32> = HashMap::new();
    for log in &logs {
      if let Some(error_type) = log.get_error_type() {
        *error_counts.entry(error_type).or_insert(0) += 1;
      }
    }

    let mut common_error_types: Vec<(ExecutionErrorType, i32)> = error_counts.into_iter().collect();
    common_error_types.sort_by(|a, b| b.1.cmp(&a.1));
    let common_error_types: Vec<ExecutionErrorType> = common_error_types
      .into_iter()
      .take(5)
      .map(|(error_type, _)| error_type)
      .collect();

    Ok(ExecutionStatistics {
      total_executions,
      successful_executions,
      failed_executions,
      cancelled_executions,
      average_execution_time_ms: average_time,
      min_execution_time_ms: min_time,
      max_execution_time_ms: max_time,
      most_used_tools,
      common_error_types,
      period_start: start_time,
      period_end: end_time,
    })
  }

  /// 删除执行日志
  pub async fn delete_execution_log(&self, execution_id: &str) -> FlowyResult<()> {
    let mut conn = self.get_connection()?;

    // 由于设置了外键约束，删除执行日志会自动删除相关的步骤和引用
    diesel::delete(execution_log_table::table.filter(log_dsl::id.eq(execution_id)))
      .execute(&mut *conn)?;

    info!("Deleted execution log: {}", execution_id);
    Ok(())
  }

  /// 清理旧的执行日志
  pub async fn cleanup_old_logs(&self, older_than_days: i32) -> FlowyResult<usize> {
    let mut conn = self.get_connection()?;
    let cutoff_time = timestamp() - (older_than_days as i64 * 24 * 60 * 60 * 1000);

    let deleted_count = diesel::delete(
      execution_log_table::table.filter(log_dsl::created_at.lt(cutoff_time))
    )
    .execute(&mut *conn)?;

    info!("Cleaned up {} old execution logs", deleted_count);
    Ok(deleted_count)
  }
}

/// 执行日志更新结构
#[derive(AsChangeset, Identifiable, Default, Debug)]
#[diesel(table_name = execution_log_table)]
#[diesel(primary_key(id))]
pub struct ExecutionLogChangeset {
  pub id: String,
  pub end_time: Option<Option<i64>>,
  pub status: Option<i32>,
  pub error_message: Option<Option<String>>,
  pub error_type: Option<Option<i32>>,
  pub total_steps: Option<i32>,
  pub completed_steps: Option<i32>,
  pub failed_steps: Option<i32>,
  pub skipped_steps: Option<i32>,
  pub context: Option<String>,
  pub result_summary: Option<Option<String>>,
  pub used_mcp_tools: Option<String>,
  pub tags: Option<String>,
  pub retry_count: Option<i32>,
  pub child_execution_ids: Option<String>,
  pub updated_at: Option<i64>,
}

/// 执行步骤更新结构
#[derive(AsChangeset, Identifiable, Default, Debug)]
#[diesel(table_name = execution_step_table)]
#[diesel(primary_key(id))]
pub struct ExecutionStepChangeset {
  pub id: String,
  pub mcp_tool_config: Option<String>,
  pub input_parameters: Option<String>,
  pub output_result: Option<Option<String>>,
  pub execution_time_ms: Option<i32>,
  pub status: Option<i32>,
  pub start_time: Option<Option<i64>>,
  pub end_time: Option<Option<i64>>,
  pub error_message: Option<Option<String>>,
  pub error_type: Option<Option<i32>>,
  pub error_stack: Option<Option<String>>,
  pub retry_count: Option<i32>,
  pub dependencies: Option<String>,
  pub tags: Option<String>,
  pub metadata: Option<String>,
  pub updated_at: Option<i64>,
}

/// MCP工具信息更新结构
#[derive(AsChangeset, Identifiable, Default, Debug)]
#[diesel(table_name = mcp_tool_info_table)]
#[diesel(primary_key(id))]
pub struct McpToolInfoChangeset {
  pub id: String,
  pub display_name: Option<Option<String>>,
  pub description: Option<String>,
  pub version: Option<String>,
  pub provider: Option<String>,
  pub category: Option<String>,
  pub status: Option<i32>,
  pub config: Option<String>,
  pub schema: Option<String>,
  pub requires_auth: Option<bool>,
  pub auth_config: Option<Option<String>>,
  pub icon_url: Option<Option<String>>,
  pub documentation_url: Option<Option<String>>,
  pub last_checked: Option<Option<i64>>,
  pub last_used: Option<Option<i64>>,
  pub usage_count: Option<i32>,
  pub success_count: Option<i32>,
  pub failure_count: Option<i32>,
  pub average_execution_time_ms: Option<i32>,
  pub updated_at: Option<i64>,
}

/// MCP工具管理器
impl ExecutionLogger {
  /// 注册或更新MCP工具信息
  pub async fn upsert_mcp_tool_info(
    &self,
    tool_info: McpToolInfoTable,
  ) -> FlowyResult<()> {
    let mut conn = self.get_connection()?;

    diesel::insert_into(mcp_tool_info_table::table)
      .values(&tool_info)
      .on_conflict(mcp_tool_info_table::id)
      .do_update()
      .set((
        mcp_tool_info_table::name.eq(excluded(mcp_tool_info_table::name)),
        mcp_tool_info_table::display_name.eq(excluded(mcp_tool_info_table::display_name)),
        mcp_tool_info_table::description.eq(excluded(mcp_tool_info_table::description)),
        mcp_tool_info_table::version.eq(excluded(mcp_tool_info_table::version)),
        mcp_tool_info_table::provider.eq(excluded(mcp_tool_info_table::provider)),
        mcp_tool_info_table::category.eq(excluded(mcp_tool_info_table::category)),
        mcp_tool_info_table::status.eq(excluded(mcp_tool_info_table::status)),
        mcp_tool_info_table::config.eq(excluded(mcp_tool_info_table::config)),
        mcp_tool_info_table::schema.eq(excluded(mcp_tool_info_table::schema)),
        mcp_tool_info_table::requires_auth.eq(excluded(mcp_tool_info_table::requires_auth)),
        mcp_tool_info_table::auth_config.eq(excluded(mcp_tool_info_table::auth_config)),
        mcp_tool_info_table::icon_url.eq(excluded(mcp_tool_info_table::icon_url)),
        mcp_tool_info_table::documentation_url.eq(excluded(mcp_tool_info_table::documentation_url)),
        mcp_tool_info_table::updated_at.eq(excluded(mcp_tool_info_table::updated_at)),
      ))
      .execute(&mut *conn)?;

    debug!("Upserted MCP tool info: {}", tool_info.id);
    Ok(())
  }

  /// 更新MCP工具使用统计
  pub async fn update_mcp_tool_usage(
    &self,
    tool_id: &str,
    success: bool,
    execution_time_ms: i32,
  ) -> FlowyResult<()> {
    let mut conn = self.get_connection()?;
    let now = timestamp();

    // 获取当前工具信息
    let tool: Option<McpToolInfoTable> = mcp_tool_info_table::table
      .filter(tool_dsl::id.eq(tool_id))
      .first(&mut *conn)
      .optional()?;

    if let Some(mut tool) = tool {
      tool.usage_count += 1;
      tool.last_used = Some(now);

      if success {
        tool.success_count += 1;
      } else {
        tool.failure_count += 1;
      }

      // 更新平均执行时间
      let total_time = (tool.average_execution_time_ms as i64 * (tool.usage_count - 1) as i64) + execution_time_ms as i64;
      tool.average_execution_time_ms = (total_time / tool.usage_count as i64) as i32;
      tool.updated_at = now;

      diesel::update(mcp_tool_info_table::table.filter(tool_dsl::id.eq(tool_id)))
        .set((
          mcp_tool_info_table::usage_count.eq(tool.usage_count),
          mcp_tool_info_table::success_count.eq(tool.success_count),
          mcp_tool_info_table::failure_count.eq(tool.failure_count),
          mcp_tool_info_table::average_execution_time_ms.eq(tool.average_execution_time_ms),
          mcp_tool_info_table::last_used.eq(tool.last_used),
          mcp_tool_info_table::updated_at.eq(tool.updated_at),
        ))
        .execute(&mut *conn)?;

      debug!("Updated MCP tool usage: {} (success: {})", tool_id, success);
    }

    Ok(())
  }

  /// 获取MCP工具信息
  pub async fn get_mcp_tool_info(&self, tool_id: &str) -> FlowyResult<Option<McpToolInfoTable>> {
    let mut conn = self.get_connection()?;
    let tool = mcp_tool_info_table::table
      .filter(tool_dsl::id.eq(tool_id))
      .first(&mut *conn)
      .optional()?;

    Ok(tool)
  }

  /// 获取所有MCP工具信息
  pub async fn get_all_mcp_tools(&self) -> FlowyResult<Vec<McpToolInfoTable>> {
    let mut conn = self.get_connection()?;
    let tools = mcp_tool_info_table::table
      .order(tool_dsl::name.asc())
      .load(&mut *conn)?;

    Ok(tools)
  }

  /// 导出执行日志
  pub async fn export_execution_logs(
    &self,
    criteria: ExecutionLogSearchCriteria,
    options: ExecutionLogExportOptions,
  ) -> FlowyResult<String> {
    let logs = self.search_execution_logs(criteria).await?;
    
    // 如果需要包含步骤信息，获取详细数据
    let detailed_logs = if options.include_steps {
      let mut detailed = Vec::new();
      for log in logs {
        if let Some((log, steps)) = self.get_execution_log_with_details(&log.id).await? {
          detailed.push((log, steps));
        }
      }
      detailed
    } else {
      logs.into_iter().map(|log| (log, Vec::new())).collect()
    };

    // 根据格式导出
    match options.format {
      ExecutionLogExportFormat::Json => self.export_to_json(detailed_logs, &options).await,
      ExecutionLogExportFormat::Csv => self.export_to_csv(detailed_logs, &options).await,
      ExecutionLogExportFormat::Html => self.export_to_html(detailed_logs, &options).await,
      ExecutionLogExportFormat::Text => self.export_to_text(detailed_logs, &options).await,
      ExecutionLogExportFormat::Excel => Err(FlowyError::not_support().with_context("Excel export not implemented")),
      ExecutionLogExportFormat::Pdf => Err(FlowyError::not_support().with_context("PDF export not implemented")),
    }
  }

  /// 导出为JSON格式
  async fn export_to_json(
    &self,
    logs: Vec<(ExecutionLogTable, Vec<(ExecutionStepTable, Vec<ExecutionReferenceTable>)>)>,
    options: &ExecutionLogExportOptions,
  ) -> FlowyResult<String> {
    let mut export_data = Vec::new();

    for (log, steps) in logs {
      let mut log_data = json!({
        "id": log.id,
        "session_id": log.session_id,
        "user_query": log.user_query,
        "start_time": log.start_time,
        "end_time": log.end_time,
        "status": format!("{:?}", log.get_status()),
        "total_steps": log.total_steps,
        "completed_steps": log.completed_steps,
        "failed_steps": log.failed_steps,
        "skipped_steps": log.skipped_steps,
        "created_at": log.created_at,
        "updated_at": log.updated_at,
      });

      if options.include_steps && !steps.is_empty() {
        let steps_data: Vec<Value> = steps
          .into_iter()
          .map(|(step, _)| {
            json!({
              "id": step.id,
              "name": step.name,
              "description": step.description,
              "mcp_tool_name": step.mcp_tool_name,
              "status": format!("{:?}", step.get_status()),
              "execution_time_ms": step.execution_time_ms,
              "step_order": step.step_order,
            })
          })
          .collect();
        log_data["steps"] = json!(steps_data);
      }

      export_data.push(log_data);
    }

    let result = json!({
      "execution_logs": export_data,
      "export_info": {
        "format": "json",
        "exported_at": timestamp(),
        "total_records": export_data.len(),
      }
    });

    Ok(serde_json::to_string_pretty(&result)?)
  }

  /// 导出为CSV格式
  async fn export_to_csv(
    &self,
    logs: Vec<(ExecutionLogTable, Vec<(ExecutionStepTable, Vec<ExecutionReferenceTable>)>)>,
    _options: &ExecutionLogExportOptions,
  ) -> FlowyResult<String> {
    let mut csv_content = String::new();

    // CSV头部
    let headers = vec![
      "ID", "Session ID", "User Query", "Status", "Start Time", "End Time",
      "Total Steps", "Completed Steps", "Failed Steps", "Created At"
    ];
    csv_content.push_str(&headers.join(","));
    csv_content.push('\n');

    // CSV数据行
    for (log, _) in logs {
      let row = vec![
        log.id.clone(),
        log.session_id.clone(),
        format!("\"{}\"", log.user_query.replace("\"", "\"\"")),
        format!("{:?}", log.get_status()),
        log.start_time.to_string(),
        log.end_time.map(|t| t.to_string()).unwrap_or_default(),
        log.total_steps.to_string(),
        log.completed_steps.to_string(),
        log.failed_steps.to_string(),
        log.created_at.to_string(),
      ];
      csv_content.push_str(&row.join(","));
      csv_content.push('\n');
    }

    Ok(csv_content)
  }

  /// 导出为HTML格式
  async fn export_to_html(
    &self,
    logs: Vec<(ExecutionLogTable, Vec<(ExecutionStepTable, Vec<ExecutionReferenceTable>)>)>,
    _options: &ExecutionLogExportOptions,
  ) -> FlowyResult<String> {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n");
    html.push_str("<title>执行日志导出</title>\n</head>\n<body>\n");
    html.push_str("<h1>执行日志导出报告</h1>\n");
    html.push_str(&format!("<p>导出时间: {}</p>\n", timestamp()));
    html.push_str(&format!("<p>总记录数: {}</p>\n", logs.len()));
    
    for (log, _) in logs {
      html.push_str(&format!("<div><h3>{}</h3>\n", log.id));
      html.push_str(&format!("<p>会话ID: {}</p>\n", log.session_id));
      html.push_str(&format!("<p>用户查询: {}</p>\n", log.user_query));
      html.push_str(&format!("<p>状态: {:?}</p>\n", log.get_status()));
      html.push_str("</div>\n");
    }
    
    html.push_str("</body>\n</html>");
    Ok(html)
  }

  /// 导出为纯文本格式
  async fn export_to_text(
    &self,
    logs: Vec<(ExecutionLogTable, Vec<(ExecutionStepTable, Vec<ExecutionReferenceTable>)>)>,
    _options: &ExecutionLogExportOptions,
  ) -> FlowyResult<String> {
    let mut text = String::new();
    
    text.push_str("执行日志导出报告\n");
    text.push_str("==================\n\n");
    text.push_str(&format!("导出时间: {}\n", timestamp()));
    text.push_str(&format!("总记录数: {}\n\n", logs.len()));
    
    for (i, (log, _)) in logs.iter().enumerate() {
      text.push_str(&format!("日志 #{}: {}\n", i + 1, log.id));
      text.push_str("----------------------------------------\n");
      text.push_str(&format!("会话ID: {}\n", log.session_id));
      text.push_str(&format!("用户查询: {}\n", log.user_query));
      text.push_str(&format!("状态: {:?}\n", log.get_status()));
      text.push_str(&format!("开始时间: {}\n", log.start_time));
      text.push_str("\n");
    }
    
    Ok(text)
  }
}
