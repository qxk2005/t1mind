use flowy_error::{FlowyError, FlowyResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

/// 文档转换任务的状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionStatus {
    /// 等待转换
    Pending,
    /// 正在转换
    Processing,
    /// 转换完成
    Completed,
    /// 转换失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 文档类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Word 文档 (.docx, .doc)
    Word,
    /// PDF 文档 (.pdf)
    Pdf,
}

impl DocumentType {
    /// 从文件扩展名确定文档类型
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "docx" | "doc" => Some(Self::Word),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    /// 获取支持的文件扩展名
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Word => vec!["docx", "doc"],
            Self::Pdf => vec!["pdf"],
        }
    }
}

/// 转换任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionConfig {
    /// 是否保留原始格式
    pub preserve_formatting: bool,
    /// 是否提取图片
    pub extract_images: bool,
    /// 是否保留表格结构
    pub preserve_tables: bool,
    /// 最大文件大小（字节）
    pub max_file_size: Option<u64>,
    /// 转换超时时间（秒）
    pub timeout_seconds: Option<u64>,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            preserve_formatting: true,
            extract_images: true,
            preserve_tables: true,
            max_file_size: Some(50 * 1024 * 1024), // 50MB
            timeout_seconds: Some(300), // 5分钟
        }
    }
}

/// 转换任务
#[derive(Clone, Serialize, Deserialize)]
pub struct ConversionTask {
    /// 任务ID
    pub id: Uuid,
    /// 源文件路径
    pub source_path: String,
    /// 文档类型
    pub document_type: DocumentType,
    /// 目标文档名称
    pub target_name: String,
    /// 转换配置
    pub config: ConversionConfig,
    /// 任务状态
    pub status: ConversionStatus,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 开始时间
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 完成时间
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 进度百分比 (0-100)
    pub progress: u8,
    /// 可选的字节数据（用于临时导入）
    pub bytes_data: Option<Vec<u8>>,
}

impl std::fmt::Debug for ConversionTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversionTask")
            .field("id", &self.id)
            .field("source_path", &self.source_path)
            .field("document_type", &self.document_type)
            .field("target_name", &self.target_name)
            .field("config", &self.config)
            .field("status", &self.status)
            .field("created_at", &self.created_at)
            .field("started_at", &self.started_at)
            .field("completed_at", &self.completed_at)
            .field("error_message", &self.error_message)
            .field("progress", &self.progress)
            .field("bytes_data", &self.bytes_data.as_ref().map(|_| "[...]"))
            .finish()
    }
}

impl ConversionTask {
    /// 创建新的转换任务
    pub fn new(
        source_path: String,
        document_type: DocumentType,
        target_name: String,
        config: ConversionConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_path,
            document_type,
            target_name,
            config,
            status: ConversionStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress: 0,
            bytes_data: None,
        }
    }

    /// 创建带有字节数据的转换任务
    pub fn new_with_bytes(
        source_path: String,
        document_type: DocumentType,
        target_name: String,
        config: ConversionConfig,
        bytes_data: Vec<u8>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_path,
            document_type,
            target_name,
            config,
            status: ConversionStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress: 0,
            bytes_data: Some(bytes_data),
        }
    }

    /// 标记任务为处理中
    pub fn mark_processing(&mut self) {
        self.status = ConversionStatus::Processing;
        self.started_at = Some(chrono::Utc::now());
        self.progress = 0;
    }

    /// 更新进度
    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }

    /// 标记任务为完成
    pub fn mark_completed(&mut self) {
        self.status = ConversionStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
        self.progress = 100;
    }

    /// 标记任务为失败
    pub fn mark_failed(&mut self, error_message: String) {
        self.status = ConversionStatus::Failed;
        self.completed_at = Some(chrono::Utc::now());
        self.error_message = Some(error_message);
    }

    /// 取消任务
    pub fn cancel(&mut self) {
        self.status = ConversionStatus::Cancelled;
        self.completed_at = Some(chrono::Utc::now());
    }
}

/// 转换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    /// 任务ID
    pub task_id: Uuid,
    /// 转换后的文档内容
    pub document_content: DocumentContent,
    /// 提取的图片
    pub extracted_images: Vec<ExtractedImage>,
    /// 转换统计信息
    pub statistics: ConversionStatistics,
}

/// 文档内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// 文档标题
    pub title: String,
    /// 文档正文内容（HTML格式）
    pub content: String,
    /// 文档元数据
    pub metadata: DocumentMetadata,
}

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// 作者
    pub author: Option<String>,
    /// 创建时间
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 修改时间
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 页数
    pub page_count: Option<u32>,
    /// 字数
    pub word_count: Option<u32>,
    /// 语言
    pub language: Option<String>,
}

/// 提取的图片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedImage {
    /// 图片ID
    pub id: String,
    /// 图片文件名
    pub filename: String,
    /// 图片数据
    pub data: Vec<u8>,
    /// 图片格式
    pub format: ImageFormat,
    /// 图片尺寸
    pub dimensions: Option<ImageDimensions>,
}

/// 图片格式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Bmp,
    WebP,
}

/// 图片尺寸
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// 转换统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStatistics {
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 提取的文本长度
    pub text_length: usize,
    /// 提取的图片数量
    pub image_count: u32,
    /// 提取的表格数量
    pub table_count: u32,
    /// 源文件大小（字节）
    pub source_file_size: u64,
}

/// 文档转换器trait
#[async_trait::async_trait]
pub trait DocumentConverter: Send + Sync {
    /// 转换器名称
    fn name(&self) -> &str;

    /// 支持的文档类型
    fn supported_types(&self) -> Vec<DocumentType>;

    /// 检查文件是否可以被此转换器处理
    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            if let Some(doc_type) = DocumentType::from_extension(ext) {
                return self.supported_types().contains(&doc_type);
            }
        }
        false
    }

    /// 转换文档
    async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult>;

    /// 验证文件
    async fn validate_file(&self, file_path: &Path) -> FlowyResult<()>;

    /// 获取文件信息
    async fn get_file_info(&self, file_path: &Path) -> FlowyResult<DocumentMetadata>;
}

/// 转换错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(u64, u64),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Conversion timeout after {0} seconds")]
    Timeout(u64),

    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ConversionError> for FlowyError {
    fn from(err: ConversionError) -> Self {
        match err {
            ConversionError::UnsupportedFormat(_) => {
                FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, err)
            }
            ConversionError::FileNotFound(_) => {
                FlowyError::new(flowy_error::ErrorCode::RecordNotFound, err)
            }
            ConversionError::FileTooLarge(_, _) => {
                FlowyError::new(flowy_error::ErrorCode::SingleUploadLimitExceeded, err)
            }
            ConversionError::InvalidFormat(_) => {
                FlowyError::new(flowy_error::ErrorCode::InvalidParams, err)
            }
            ConversionError::Timeout(_) => {
                FlowyError::new(flowy_error::ErrorCode::ResponseTimeout, err)
            }
            ConversionError::ConversionFailed(_) => {
                FlowyError::new(flowy_error::ErrorCode::Internal, err)
            }
            ConversionError::IoError(_) => {
                FlowyError::new(flowy_error::ErrorCode::Internal, err)
            }
            ConversionError::Internal(_) => {
                FlowyError::new(flowy_error::ErrorCode::Internal, err)
            }
        }
    }
}

/// 转换器枚举，用于解决trait对象安全问题
#[derive(Debug)]
pub enum DocumentConverterEnum {
    Word(crate::import::word_converter::WordConverter),
    Pdf(crate::import::pdf_converter::PdfConverter),
}

impl DocumentConverterEnum {
    pub fn name(&self) -> &str {
        match self {
            Self::Word(converter) => converter.name(),
            Self::Pdf(converter) => converter.name(),
        }
    }

    pub fn supported_types(&self) -> Vec<DocumentType> {
        match self {
            Self::Word(converter) => converter.supported_types(),
            Self::Pdf(converter) => converter.supported_types(),
        }
    }

    pub fn can_handle(&self, file_path: &Path) -> bool {
        match self {
            Self::Word(converter) => converter.can_handle(file_path),
            Self::Pdf(converter) => converter.can_handle(file_path),
        }
    }

    pub async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult> {
        match self {
            Self::Word(converter) => converter.convert(task).await,
            Self::Pdf(converter) => converter.convert(task).await,
        }
    }

    pub async fn validate_file(&self, file_path: &Path) -> FlowyResult<()> {
        match self {
            Self::Word(converter) => converter.validate_file(file_path).await,
            Self::Pdf(converter) => converter.validate_file(file_path).await,
        }
    }

    pub async fn get_file_info(&self, file_path: &Path) -> FlowyResult<DocumentMetadata> {
        match self {
            Self::Word(converter) => converter.get_file_info(file_path).await,
            Self::Pdf(converter) => converter.get_file_info(file_path).await,
        }
    }
}

/// 转换器工厂trait
pub trait ConverterFactory: Send + Sync {
    /// 创建转换器实例
    fn create_converter(&self, doc_type: DocumentType) -> Option<DocumentConverterEnum>;

    /// 获取所有支持的转换器
    fn get_all_converters(&self) -> Vec<DocumentConverterEnum>;
}

/// 默认转换器工厂实现
pub struct DefaultConverterFactory;

impl ConverterFactory for DefaultConverterFactory {
    fn create_converter(&self, doc_type: DocumentType) -> Option<DocumentConverterEnum> {
        match doc_type {
            DocumentType::Word => {
                let config = ConversionConfig::default();
                Some(DocumentConverterEnum::Word(crate::import::word_converter::WordConverter::new(config)))
            }
            DocumentType::Pdf => {
                let config = ConversionConfig::default();
                Some(DocumentConverterEnum::Pdf(crate::import::pdf_converter::PdfConverter::new(config)))
            }
        }
    }

    fn get_all_converters(&self) -> Vec<DocumentConverterEnum> {
        vec![
            DocumentConverterEnum::Word(crate::import::word_converter::WordConverter::new(ConversionConfig::default())),
            DocumentConverterEnum::Pdf(crate::import::pdf_converter::PdfConverter::new(ConversionConfig::default())),
        ]
    }
}
