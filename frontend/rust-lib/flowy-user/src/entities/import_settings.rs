use serde::{Deserialize, Serialize};

use flowy_derive::{ProtoBuf, ProtoBuf_Enum};

/// 导入设置配置结构
/// 用于管理文档导入功能的各项配置参数
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ImportSettingsPB {
  /// 最大并发转换任务数
  #[pb(index = 1)]
  pub max_concurrent_conversions: i32,

  /// 默认导入路径
  #[pb(index = 2)]
  pub default_import_path: String,

  /// 是否保留原始格式
  #[pb(index = 3)]
  pub preserve_formatting: bool,

  /// 是否提取图片
  #[pb(index = 4)]
  pub extract_images: bool,

  /// 是否提取表格
  #[pb(index = 5)]
  pub extract_tables: bool,

  /// 日志级别
  #[pb(index = 6)]
  pub log_level: LogLevelPB,

  /// 是否自动创建文件夹
  #[pb(index = 7)]
  pub auto_create_folder: bool,

  /// 是否启用转换进度通知
  #[pb(index = 8)]
  pub enable_progress_notifications: bool,

  /// 转换超时时间（秒）
  #[pb(index = 9)]
  pub conversion_timeout_seconds: i32,

  /// 是否在转换失败时自动重试
  #[pb(index = 10)]
  pub auto_retry_on_failure: bool,

  /// 最大重试次数
  #[pb(index = 11)]
  pub max_retry_attempts: i32,

  /// 是否保存转换日志
  #[pb(index = 12)]
  pub save_conversion_logs: bool,

  /// 日志保留天数
  #[pb(index = 13)]
  pub log_retention_days: i32,
}

/// 日志级别枚举
#[derive(ProtoBuf_Enum, Serialize, Deserialize, Clone, Debug, Default)]
pub enum LogLevelPB {
  #[default]
  Error = 0,
  Warn = 1,
  Info = 2,
  Debug = 3,
  Trace = 4,
}

/// 导入类型枚举
#[derive(ProtoBuf_Enum, Serialize, Deserialize, Clone, Debug, Default)]
pub enum ImportTypePB {
  #[default]
  Word = 0,
  PDF = 1,
  Text = 2,
}

/// 转换状态枚举
#[derive(ProtoBuf_Enum, Serialize, Deserialize, Clone, Debug, Default)]
pub enum ConversionStatusPB {
  #[default]
  Pending = 0,
  Processing = 1,
  Completed = 2,
  Failed = 3,
  Cancelled = 4,
}

/// 转换任务信息
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ConversionTaskPB {
  /// 任务唯一标识符
  #[pb(index = 1)]
  pub id: String,

  /// 文件路径
  #[pb(index = 2)]
  pub file_path: String,

  /// 文件名
  #[pb(index = 3)]
  pub file_name: String,

  /// 文件类型
  #[pb(index = 4)]
  pub file_type: ImportTypePB,

  /// 转换状态
  #[pb(index = 5)]
  pub status: ConversionStatusPB,

  /// 转换进度 (0.0 到 1.0)
  #[pb(index = 6)]
  pub progress: f64,

  /// 创建时间戳
  #[pb(index = 7)]
  pub created_at: i64,

  /// 开始时间戳
  #[pb(index = 8, one_of)]
  pub started_at: Option<i64>,

  /// 完成时间戳
  #[pb(index = 9, one_of)]
  pub completed_at: Option<i64>,

  /// 错误信息
  #[pb(index = 10, one_of)]
  pub error_message: Option<String>,

  /// 父视图ID
  #[pb(index = 11)]
  pub parent_view_id: String,

  /// 转换结果文档ID
  #[pb(index = 12, one_of)]
  pub result_document_id: Option<String>,
}

/// 转换结果信息
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ConversionResultPB {
  /// 任务ID
  #[pb(index = 1)]
  pub task_id: String,

  /// 生成的文档ID
  #[pb(index = 2)]
  pub document_id: String,

  /// 文档名称
  #[pb(index = 3)]
  pub document_name: String,

  /// 转换是否成功
  #[pb(index = 4)]
  pub success: bool,

  /// 错误信息
  #[pb(index = 5, one_of)]
  pub error_message: Option<String>,

  /// 处理时间（毫秒）
  #[pb(index = 6)]
  pub processing_time_ms: i64,

  /// 转换的元素数量
  #[pb(index = 7)]
  pub converted_elements_count: i32,

  /// 提取的图片数量
  #[pb(index = 8)]
  pub extracted_images_count: i32,

  /// 提取的表格数量
  #[pb(index = 9)]
  pub extracted_tables_count: i32,
}

/// 转换元素信息
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ConvertedElementPB {
  /// 元素类型
  #[pb(index = 1)]
  pub element_type: ElementTypePB,

  /// 元素内容
  #[pb(index = 2)]
  pub content: String,

  /// 格式化信息（JSON字符串）
  #[pb(index = 3, one_of)]
  pub formatting: Option<String>,

  /// 位置信息（JSON字符串）
  #[pb(index = 4)]
  pub position: String,

  /// 元数据（JSON字符串）
  #[pb(index = 5)]
  pub metadata: String,
}

/// 元素类型枚举
#[derive(ProtoBuf_Enum, Serialize, Deserialize, Clone, Debug, Default)]
pub enum ElementTypePB {
  #[default]
  PlainText = 0,
  Image = 1,
  Table = 2,
  List = 3,
  Heading = 4,
  Paragraph = 5,
}

/// 更新导入设置的请求
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct UpdateImportSettingsPB {
  /// 最大并发转换任务数
  #[pb(index = 1, one_of)]
  pub max_concurrent_conversions: Option<i32>,

  /// 默认导入路径
  #[pb(index = 2, one_of)]
  pub default_import_path: Option<String>,

  /// 是否保留原始格式
  #[pb(index = 3, one_of)]
  pub preserve_formatting: Option<bool>,

  /// 是否提取图片
  #[pb(index = 4, one_of)]
  pub extract_images: Option<bool>,

  /// 是否提取表格
  #[pb(index = 5, one_of)]
  pub extract_tables: Option<bool>,

  /// 日志级别
  #[pb(index = 6, one_of)]
  pub log_level: Option<LogLevelPB>,

  /// 是否自动创建文件夹
  #[pb(index = 7, one_of)]
  pub auto_create_folder: Option<bool>,

  /// 是否启用转换进度通知
  #[pb(index = 8, one_of)]
  pub enable_progress_notifications: Option<bool>,

  /// 转换超时时间（秒）
  #[pb(index = 9, one_of)]
  pub conversion_timeout_seconds: Option<i32>,

  /// 是否在转换失败时自动重试
  #[pb(index = 10, one_of)]
  pub auto_retry_on_failure: Option<bool>,

  /// 最大重试次数
  #[pb(index = 11, one_of)]
  pub max_retry_attempts: Option<i32>,

  /// 是否保存转换日志
  #[pb(index = 12, one_of)]
  pub save_conversion_logs: Option<bool>,

  /// 日志保留天数
  #[pb(index = 13, one_of)]
  pub log_retention_days: Option<i32>,
}

/// 获取导入设置的请求
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct GetImportSettingsPB {
  /// 用户ID
  #[pb(index = 1)]
  pub user_id: String,
}

/// 获取转换任务列表的请求
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct GetConversionTasksPB {
  /// 用户ID
  #[pb(index = 1)]
  pub user_id: String,

  /// 任务状态过滤
  #[pb(index = 2, one_of)]
  pub status_filter: Option<ConversionStatusPB>,

  /// 分页偏移
  #[pb(index = 3)]
  pub offset: i32,

  /// 分页大小
  #[pb(index = 4)]
  pub limit: i32,
}

/// 转换任务列表响应
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ConversionTasksPB {
  /// 任务列表
  #[pb(index = 1)]
  pub tasks: Vec<ConversionTaskPB>,

  /// 总任务数
  #[pb(index = 2)]
  pub total_count: i32,
}

/// 取消转换任务的请求
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct CancelConversionTaskPB {
  /// 任务ID
  #[pb(index = 1)]
  pub task_id: String,

  /// 用户ID
  #[pb(index = 2)]
  pub user_id: String,
}

/// 获取转换日志的请求
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct GetConversionLogsPB {
  /// 任务ID
  #[pb(index = 1)]
  pub task_id: String,

  /// 用户ID
  #[pb(index = 2)]
  pub user_id: String,

  /// 日志级别过滤
  #[pb(index = 3, one_of)]
  pub level_filter: Option<LogLevelPB>,
}

/// 转换日志条目
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ConversionLogEntryPB {
  /// 时间戳
  #[pb(index = 1)]
  pub timestamp: i64,

  /// 日志级别
  #[pb(index = 2)]
  pub level: LogLevelPB,

  /// 日志消息
  #[pb(index = 3)]
  pub message: String,

  /// 任务ID
  #[pb(index = 4)]
  pub task_id: String,
}

/// 转换日志列表响应
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ConversionLogsPB {
  /// 日志条目列表
  #[pb(index = 1)]
  pub logs: Vec<ConversionLogEntryPB>,

  /// 总日志数
  #[pb(index = 2)]
  pub total_count: i32,
}

// 默认值常量
const DEFAULT_MAX_CONCURRENT_CONVERSIONS: i32 = 3;
const DEFAULT_DEFAULT_IMPORT_PATH: &str = "";
const DEFAULT_PRESERVE_FORMATTING: bool = true;
const DEFAULT_EXTRACT_IMAGES: bool = true;
const DEFAULT_EXTRACT_TABLES: bool = true;
const DEFAULT_AUTO_CREATE_FOLDER: bool = true;
const DEFAULT_ENABLE_PROGRESS_NOTIFICATIONS: bool = true;
const DEFAULT_CONVERSION_TIMEOUT_SECONDS: i32 = 300; // 5分钟
const DEFAULT_AUTO_RETRY_ON_FAILURE: bool = true;
const DEFAULT_MAX_RETRY_ATTEMPTS: i32 = 3;
const DEFAULT_SAVE_CONVERSION_LOGS: bool = true;
const DEFAULT_LOG_RETENTION_DAYS: i32 = 30;

impl std::default::Default for ImportSettingsPB {
  fn default() -> Self {
    ImportSettingsPB {
      max_concurrent_conversions: DEFAULT_MAX_CONCURRENT_CONVERSIONS,
      default_import_path: DEFAULT_DEFAULT_IMPORT_PATH.to_owned(),
      preserve_formatting: DEFAULT_PRESERVE_FORMATTING,
      extract_images: DEFAULT_EXTRACT_IMAGES,
      extract_tables: DEFAULT_EXTRACT_TABLES,
      log_level: LogLevelPB::default(),
      auto_create_folder: DEFAULT_AUTO_CREATE_FOLDER,
      enable_progress_notifications: DEFAULT_ENABLE_PROGRESS_NOTIFICATIONS,
      conversion_timeout_seconds: DEFAULT_CONVERSION_TIMEOUT_SECONDS,
      auto_retry_on_failure: DEFAULT_AUTO_RETRY_ON_FAILURE,
      max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
      save_conversion_logs: DEFAULT_SAVE_CONVERSION_LOGS,
      log_retention_days: DEFAULT_LOG_RETENTION_DAYS,
    }
  }
}

impl std::default::Default for ConversionTaskPB {
  fn default() -> Self {
    ConversionTaskPB {
      id: String::new(),
      file_path: String::new(),
      file_name: String::new(),
      file_type: ImportTypePB::default(),
      status: ConversionStatusPB::default(),
      progress: 0.0,
      created_at: 0,
      started_at: None,
      completed_at: None,
      error_message: None,
      parent_view_id: String::new(),
      result_document_id: None,
    }
  }
}

impl std::default::Default for ConversionResultPB {
  fn default() -> Self {
    ConversionResultPB {
      task_id: String::new(),
      document_id: String::new(),
      document_name: String::new(),
      success: false,
      error_message: None,
      processing_time_ms: 0,
      converted_elements_count: 0,
      extracted_images_count: 0,
      extracted_tables_count: 0,
    }
  }
}

impl std::default::Default for ConvertedElementPB {
  fn default() -> Self {
    ConvertedElementPB {
      element_type: ElementTypePB::default(),
      content: String::new(),
      formatting: None,
      position: String::new(),
      metadata: String::new(),
    }
  }
}

impl std::default::Default for UpdateImportSettingsPB {
  fn default() -> Self {
    UpdateImportSettingsPB {
      max_concurrent_conversions: None,
      default_import_path: None,
      preserve_formatting: None,
      extract_images: None,
      extract_tables: None,
      log_level: None,
      auto_create_folder: None,
      enable_progress_notifications: None,
      conversion_timeout_seconds: None,
      auto_retry_on_failure: None,
      max_retry_attempts: None,
      save_conversion_logs: None,
      log_retention_days: None,
    }
  }
}

impl std::default::Default for GetImportSettingsPB {
  fn default() -> Self {
    GetImportSettingsPB {
      user_id: String::new(),
    }
  }
}

impl std::default::Default for GetConversionTasksPB {
  fn default() -> Self {
    GetConversionTasksPB {
      user_id: String::new(),
      status_filter: None,
      offset: 0,
      limit: 50,
    }
  }
}

impl std::default::Default for ConversionTasksPB {
  fn default() -> Self {
    ConversionTasksPB {
      tasks: Vec::new(),
      total_count: 0,
    }
  }
}

impl std::default::Default for CancelConversionTaskPB {
  fn default() -> Self {
    CancelConversionTaskPB {
      task_id: String::new(),
      user_id: String::new(),
    }
  }
}

impl std::default::Default for GetConversionLogsPB {
  fn default() -> Self {
    GetConversionLogsPB {
      task_id: String::new(),
      user_id: String::new(),
      level_filter: None,
    }
  }
}

impl std::default::Default for ConversionLogEntryPB {
  fn default() -> Self {
    ConversionLogEntryPB {
      timestamp: 0,
      level: LogLevelPB::default(),
      message: String::new(),
      task_id: String::new(),
    }
  }
}

impl std::default::Default for ConversionLogsPB {
  fn default() -> Self {
    ConversionLogsPB {
      logs: Vec::new(),
      total_count: 0,
    }
  }
}

/// PDF 导入工具状态
#[derive(ProtoBuf_Enum, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum ImportToolStatusPB {
  #[default]
  ToolUnknown = 0,
  /// 工具已安装且可用
  ToolAvailable = 1,
  /// 工具未安装
  ToolNotInstalled = 2,
  /// 工具安装但无法使用
  ToolUnavailable = 3,
}

/// PDF 导入工具信息
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ImportToolInfoPB {
  /// 工具名称
  #[pb(index = 1)]
  pub name: String,

  /// 工具状态
  #[pb(index = 2)]
  pub status: ImportToolStatusPB,

  /// 工具版本（如果可用）
  #[pb(index = 3, one_of)]
  pub version: Option<String>,

  /// 工具路径（如果可用）
  #[pb(index = 4, one_of)]
  pub path: Option<String>,

  /// 安装说明（如果未安装）
  #[pb(index = 5, one_of)]
  pub install_instruction: Option<String>,

  /// 工具描述
  #[pb(index = 6)]
  pub description: String,
}

/// PDF 导入工具检查结果
#[derive(ProtoBuf, Serialize, Deserialize, Debug, Clone)]
pub struct ImportToolsStatusPB {
  /// 工具列表
  #[pb(index = 1)]
  pub tools: Vec<ImportToolInfoPB>,

  /// 检查时间戳
  #[pb(index = 2)]
  pub checked_at: i64,
}

impl std::default::Default for ImportToolInfoPB {
  fn default() -> Self {
    ImportToolInfoPB {
      name: String::new(),
      status: ImportToolStatusPB::ToolUnknown,
      version: None,
      path: None,
      install_instruction: None,
      description: String::new(),
    }
  }
}

impl std::default::Default for ImportToolsStatusPB {
  fn default() -> Self {
    ImportToolsStatusPB {
      tools: Vec::new(),
      checked_at: 0,
    }
  }
}
