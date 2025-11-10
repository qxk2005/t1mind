use crate::entities::ViewLayoutPB;
use crate::entities::parser::empty_str::NotEmptyStr;
use crate::share::{ImportData, ImportItem, ImportParams, ImportType};
use flowy_derive::{ProtoBuf, ProtoBuf_Enum};
use flowy_error::FlowyError;
use lib_infra::validator_fn::required_not_empty_str;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, ProtoBuf_Enum)]
pub enum ImportTypePB {
  HistoryDocument = 0,
  HistoryDatabase = 1,
  Markdown = 2,
  AFDatabase = 3,
  CSV = 4,
  Word = 5,
  Pdf = 6,
}

impl From<ImportTypePB> for ImportType {
  fn from(pb: ImportTypePB) -> Self {
    match pb {
      ImportTypePB::HistoryDocument => ImportType::HistoryDocument,
      ImportTypePB::HistoryDatabase => ImportType::HistoryDatabase,
      ImportTypePB::Markdown => ImportType::Markdown,
      ImportTypePB::AFDatabase => ImportType::AFDatabase,
      ImportTypePB::CSV => ImportType::CSV,
      ImportTypePB::Word => ImportType::Word,
      ImportTypePB::Pdf => ImportType::Pdf,
    }
  }
}

impl Default for ImportTypePB {
  fn default() -> Self {
    Self::Markdown
  }
}

#[derive(Clone, Debug, ProtoBuf, Default)]
pub struct ImportItemPayloadPB {
  // the name of the import page
  #[pb(index = 1)]
  pub name: String,

  // the data of the import page
  // if the data is empty, the file_path must be provided
  #[pb(index = 2, one_of)]
  pub data: Option<Vec<u8>>,

  // the file path of the import page
  // if the file_path is empty, the data must be provided
  #[pb(index = 3, one_of)]
  pub file_path: Option<String>,

  // the layout of the import page
  #[pb(index = 4)]
  pub view_layout: ViewLayoutPB,

  // the type of the import page
  #[pb(index = 5)]
  pub import_type: ImportTypePB,

  // the view id for the import page (optional)
  // if provided, this will be used as import_id and view_id
  #[pb(index = 6, one_of)]
  pub view_id: Option<String>,
}

#[derive(Clone, Debug, Validate, ProtoBuf, Default)]
pub struct ImportPayloadPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub parent_view_id: String,

  #[pb(index = 2)]
  pub items: Vec<ImportItemPayloadPB>,
}

impl TryInto<ImportParams> for ImportPayloadPB {
  type Error = FlowyError;

  fn try_into(self) -> Result<ImportParams, Self::Error> {
    let parent_view_id = NotEmptyStr::parse(self.parent_view_id)
      .map_err(|_| FlowyError::invalid_view_id())?
      .0;

    let parent_view_id = Uuid::from_str(&parent_view_id)?;

    let items = self
      .items
      .into_iter()
      .map(|item| {
        let name = if item.name.is_empty() {
          "Untitled".to_string()
        } else {
          item.name
        };

        let data = match (item.file_path, item.data) {
          (Some(file_path), None) => ImportData::FilePath { file_path },
          (None, Some(bytes)) => ImportData::Bytes { bytes },
          (None, None) => {
            return Err(FlowyError::invalid_data().with_context("The import data is empty"));
          },
          (Some(_), Some(_)) => {
            return Err(FlowyError::invalid_data().with_context("The import data is ambiguous"));
          },
        };

        let view_id = item.view_id
          .and_then(|id| Uuid::from_str(&id).ok());

        Ok(ImportItem {
          name,
          data,
          view_layout: item.view_layout.into(),
          import_type: item.import_type.into(),
          view_id,
        })
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(ImportParams {
      parent_view_id,
      items,
    })
  }
}

#[derive(Clone, Debug, Validate, ProtoBuf, Default)]
pub struct ImportZipPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub file_path: String,
}

/// 导入进度通知的注册请求
#[derive(Clone, Debug, ProtoBuf, Default)]
pub struct RegisterImportProgressStreamPB {
  #[pb(index = 1)]
  pub port: i64,
}

/// 获取导入进度的请求
#[derive(Clone, Debug, ProtoBuf, Default)]
pub struct GetImportProgressPB {
  #[pb(index = 1)]
  pub import_id: String, // 导入任务的唯一标识
}

/// 导入日志条目
#[derive(Clone, Debug, ProtoBuf, Default)]
pub struct ImportLogEntryPB {
  #[pb(index = 1)]
  pub timestamp: i64, // 时间戳（毫秒）

  #[pb(index = 2)]
  pub level: String,  // 'info', 'debug', 'warn', 'error'

  #[pb(index = 3)]
  pub message: String,
}

/// 导入进度数据结构
#[derive(Clone, Debug, ProtoBuf, Default)]
pub struct ImportProgressPB {
  #[pb(index = 1)]
  pub import_id: String, // 导入任务的唯一标识

  #[pb(index = 2)]
  pub file_name: String, // 文件名

  #[pb(index = 3)]
  pub progress: f64, // 进度 0.0 - 1.0

  #[pb(index = 4)]
  pub current_step: String, // 当前步骤描述

  #[pb(index = 5, one_of)]
  pub error: Option<String>, // 错误信息（如果有）

  #[pb(index = 6)]
  pub logs: Vec<ImportLogEntryPB>, // 日志列表
}

/// 导入进度步骤枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, ProtoBuf_Enum)]
pub enum ImportProgressStep {
  Preparing = 0,        // 准备导入
  ReadingFile = 1,       // 正在读取文件
  ExtractingText = 2,    // 正在提取文本
  ExtractingImages = 3,  // 正在提取图片
  ParsingFormat = 4,    // 正在解析格式
  Converting = 5,       // 正在转换
  CreatingDocument = 6, // 正在创建文档
  Completed = 7,        // 完成
  Error = 8,           // 错误
}

impl Default for ImportProgressStep {
  fn default() -> Self {
    Self::Preparing
  }
}

impl ImportProgressStep {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Preparing => "准备导入...",
      Self::ReadingFile => "正在读取文件...",
      Self::ExtractingText => "正在提取文本内容...",
      Self::ExtractingImages => "正在提取图片...",
      Self::ParsingFormat => "正在解析格式...",
      Self::Converting => "正在转换为文档格式...",
      Self::CreatingDocument => "正在创建文档...",
      Self::Completed => "导入完成",
      Self::Error => "导入出错",
    }
  }

  pub fn progress(&self) -> f64 {
    match self {
      Self::Preparing => 0.0,
      Self::ReadingFile => 0.1,
      Self::ExtractingText => 0.3,
      Self::ExtractingImages => 0.5,
      Self::ParsingFormat => 0.6,
      Self::Converting => 0.8,
      Self::CreatingDocument => 0.9,
      Self::Completed => 1.0,
      Self::Error => 0.0,
    }
  }
}
