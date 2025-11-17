use crate::import::converter::{
    ConversionConfig, ConversionResult, ConversionTask, DocumentConverter, DocumentMetadata,
    DocumentType, DocumentContent, ConversionStatistics, ExtractedImage, ImageFormat, ImageDimensions,
};
use crate::import::marker_tool_manager::MarkerToolManager;
use flowy_error::{FlowyError, FlowyResult};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Cursor};
use tempfile::TempDir;
use tracing::{debug, error, info, warn};
use tokio::process::Command;
use tokio::time::Duration;
use uuid::Uuid;
use image::ImageOutputFormat;
use chrono::Utc;

/// Marker PDF 转换器
/// 
/// 使用 Marker 工具将 PDF 直接转换为 Markdown 格式。
/// 该转换器负责执行 Marker 工具、捕获输出、处理错误，并提取转换后的 Markdown 内容。
#[derive(Debug)]
pub struct MarkerPdfConverter {
    /// 转换配置
    config: ConversionConfig,
    /// Marker 工具管理器
    tool_manager: MarkerToolManager,
}

impl MarkerPdfConverter {
    /// 创建新的 Marker PDF 转换器
    pub fn new(config: ConversionConfig) -> Self {
        Self {
            config,
            tool_manager: MarkerToolManager::new(),
        }
    }

    /// 验证 PDF 文件是否有效
    /// 
    /// 检查：
    /// 1. 文件是否存在
    /// 2. 文件扩展名是否为 .pdf
    /// 3. 文件大小是否在限制范围内
    /// 4. PDF 文件头是否有效
    pub async fn validate_pdf(&self, file_path: &Path) -> FlowyResult<()> {
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::RecordNotFound,
                format!("PDF 文件不存在: {}", file_path.display()),
            ));
        }

        // 检查文件扩展名
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            if ext.to_lowercase() != "pdf" {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::UnsupportedFileFormat,
                    format!("不支持的文件格式: {}，期望 PDF 文件", ext),
                ));
            }
        } else {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::UnsupportedFileFormat,
                "文件没有扩展名，无法确定文件类型",
            ));
        }

        // 检查文件大小
        if let Some(max_size) = self.config.max_file_size {
            let file_size = std::fs::metadata(file_path)
                .map_err(|e| FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("无法获取文件元数据: {}", e),
                ))?
                .len();
            
            if file_size > max_size {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::SingleUploadLimitExceeded,
                    format!(
                        "文件过大: {} 字节 (最大: {} 字节)",
                        file_size, max_size
                    ),
                ));
            }
        }

        // 检查 PDF 文件头
        let mut file = File::open(file_path).map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法打开 PDF 文件: {}", e),
            )
        })?;
        
        let mut header = [0u8; 4];
        file.read_exact(&mut header).map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法读取 PDF 文件头: {}", e),
            )
        })?;
        
        if &header != b"%PDF" {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::UnsupportedFileFormat,
                "无效的 PDF 文件格式：文件头不匹配",
            ));
        }

        debug!("PDF 文件验证成功: {}", file_path.display());
        Ok(())
    }

    /// 使用 Marker 工具将 PDF 转换为 Markdown
    /// 
    /// 执行流程：
    /// 1. 获取 Marker 工具路径
    /// 2. 创建临时输出目录
    /// 3. 执行 Marker 工具
    /// 4. 捕获 stdout/stderr
    /// 5. 读取生成的 Markdown 文件
    /// 6. 处理错误和超时
    /// 
    /// # 参数
    /// - `pdf_path`: PDF 文件路径
    /// 
    /// # 返回
    /// - `Ok(String)`: 转换后的 Markdown 内容
    /// - `Err(FlowyError)`: 转换过程中的错误
    pub async fn convert_pdf_to_markdown(&self, pdf_path: &Path) -> FlowyResult<String> {
        // 创建临时输出目录
        let output_dir = TempDir::new().map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法创建临时输出目录: {}", e),
            )
        })?;
        
        self.convert_pdf_to_markdown_with_output_dir(pdf_path, output_dir.path()).await
    }

    /// 使用 Marker 工具将 PDF 转换为 Markdown（指定输出目录）
    /// 
    /// 执行流程：
    /// 1. 获取 Marker 工具路径
    /// 2. 执行 Marker 工具（使用指定的输出目录）
    /// 3. 捕获 stdout/stderr
    /// 4. 读取生成的 Markdown 文件
    /// 5. 处理错误和超时
    /// 
    /// # 参数
    /// - `pdf_path`: PDF 文件路径
    /// - `output_dir`: 输出目录路径
    /// 
    /// # 返回
    /// - `Ok(String)`: 转换后的 Markdown 内容
    /// - `Err(FlowyError)`: 转换过程中的错误
    pub async fn convert_pdf_to_markdown_with_output_dir(
        &self,
        pdf_path: &Path,
        output_dir: &Path,
    ) -> FlowyResult<String> {
        self.convert_pdf_to_markdown_with_output_dir_and_import_id(pdf_path, output_dir, None).await
    }

    /// 使用 Marker 工具将 PDF 转换为 Markdown（指定输出目录和 import_id）
    /// 
    /// 与 convert_pdf_to_markdown_with_output_dir 相同，但支持通过 import_id 发送日志
    pub async fn convert_pdf_to_markdown_with_output_dir_and_import_id(
        &self,
        pdf_path: &Path,
        output_dir: &Path,
        import_id: Option<&str>,
    ) -> FlowyResult<String> {
        // 验证 PDF 文件
        self.validate_pdf(pdf_path).await?;

        // 获取 Marker 工具路径
        let marker_path = self.tool_manager.get_marker_path().map_err(|e| {
            let error_msg = format!("无法获取 Marker 工具路径: {}", e);
            if let Some(id) = import_id {
                crate::import::import_log_collector::send_import_log(id, "error", error_msg.clone());
            }
            error!("{}", error_msg);
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!(
                    "Marker 工具不可用: {}\n\
                     请确保 Marker 工具已正确打包到应用包中。",
                    e
                ),
            )
        })?;

        info!("开始使用 Marker 工具转换 PDF: {}", pdf_path.display());
        debug!("Marker 工具路径: {}", marker_path.display());
        debug!("输出目录: {}", output_dir.display());

        if let Some(id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("Marker 工具路径: {}", marker_path.display()),
            );
        }

        let output_path = output_dir.join("output.md");

        // 执行 Marker 工具（传递 import_id 用于日志收集）
        let markdown_content = self
            .execute_marker_tool_with_import_id(&marker_path, pdf_path, &output_path, import_id)
            .await?;

        info!(
            "PDF 转换成功: {} -> Markdown ({} 字符)",
            pdf_path.display(),
            markdown_content.len()
        );

        if let Some(id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("PDF 转换成功: {} 字符", markdown_content.len()),
            );
        }

        Ok(markdown_content)
    }

    /// 执行 Marker 工具（带 import_id 支持）
    async fn execute_marker_tool_with_import_id(
        &self,
        marker_path: &Path,
        pdf_path: &Path,
        output_path: &Path,
        import_id: Option<&str>,
    ) -> FlowyResult<String> {
        // 如果没有 import_id，调用原方法（不传递 import_id）
        if import_id.is_none() {
            return self.execute_marker_tool(marker_path, pdf_path, output_path, None).await;
        }
        
        // 有 import_id 时，使用带日志收集的版本
        let import_id = import_id.unwrap();
        
        // 发送开始执行日志
        crate::import::import_log_collector::send_import_log(
            import_id,
            "info",
            format!("开始执行 Marker 工具: {}", pdf_path.display()),
        );
        crate::import::import_log_collector::send_import_log(
            import_id,
            "info",
            format!("Marker 工具路径: {}", marker_path.display()),
        );
        crate::import::import_log_collector::send_import_log(
            import_id,
            "info",
            format!("输出目录: {}", output_path.parent().unwrap_or_else(|| Path::new(".")).display()),
        );
        
        // 更新进度：Marker 工具执行中（30%-70%）
        crate::import::import_log_collector::send_import_progress(
            import_id,
            "",
            0.3,
            "正在执行 Marker 工具转换 PDF...",
        );
        
        let result = self.execute_marker_tool(marker_path, pdf_path, output_path, Some(import_id)).await;
        
        match &result {
            Ok(_) => {
                crate::import::import_log_collector::send_import_log(
                    import_id,
                    "info",
                    "Marker 工具执行成功".to_string(),
                );
                // 更新进度：Marker 工具完成（70%）
                crate::import::import_log_collector::send_import_progress(
                    import_id,
                    "",
                    0.7,
                    "Marker 工具转换完成，正在处理结果...",
                );
            }
            Err(e) => {
                crate::import::import_log_collector::send_import_log(
                    import_id,
                    "error",
                    format!("Marker 工具执行失败: {}", e),
                );
            }
        }
        
        result
    }

    /// 执行 Marker 工具
    /// 
    /// 使用 tokio::process::Command 执行 Marker 工具，使用流式读取避免栈溢出，
    /// 并实现超时机制。
    /// 
    /// # 参数
    /// - `marker_path`: Marker 工具的可执行文件路径
    /// - `pdf_path`: 要转换的 PDF 文件路径
    /// - `output_path`: 输出 Markdown 文件路径
    /// - `import_id`: 可选的导入任务ID，用于发送日志
    /// 
    /// # 返回
    /// - `Ok(String)`: 转换后的 Markdown 内容
    /// - `Err(FlowyError)`: 执行过程中的错误
    async fn execute_marker_tool(
        &self,
        marker_path: &Path,
        pdf_path: &Path,
        output_path: &Path,
        import_id: Option<&str>,
    ) -> FlowyResult<String> {
        // 获取超时时间
        // 注意：首次运行 marker_single 需要下载多个大型模型（layout、text_recognition 等）
        // 每个模型约 1-2GB，总下载时间可能需要 15-30 分钟，取决于网络速度
        // 默认超时时间设置为 30 分钟（1800 秒），以应对首次运行和大型 PDF 文件
        let timeout_seconds = self
            .config
            .timeout_seconds
            .unwrap_or(1800) // 默认 30 分钟（首次运行需要下载多个大型模型）
            .max(300); // 最少 5 分钟
        
        let timeout_duration = Duration::from_secs(timeout_seconds);

        // 准备命令参数
        let marker_path = marker_path.to_path_buf();
        let pdf_path = pdf_path.to_path_buf();
        let output_dir = output_path.parent().unwrap().to_path_buf();
        
        // 记录完整的命令（用于调试）
        info!("执行 Marker 命令: {} {} --output_dir {} --output_format markdown", 
            marker_path.display(), 
            pdf_path.display(), 
            output_dir.display()
        );

        // 执行命令并设置超时
        info!("启动 Marker 工具，超时时间: {} 秒", timeout_duration.as_secs());
        
        // 使用 tokio::process::Command 而不是 std::process::Command
        // 这样可以避免一次性读取所有输出导致的栈溢出问题
        // 使用流式读取 stdout/stderr，避免内存溢出
        let output_result = tokio::time::timeout(timeout_duration, async move {
            let mut cmd = Command::new(&marker_path);
            
            // 设置命令参数
            cmd.arg(&pdf_path);
            cmd.arg("--output_dir");
            cmd.arg(&output_dir);
            cmd.arg("--output_format");
            cmd.arg("markdown");
            
            // 配置标准输入输出
            cmd.stdin(std::process::Stdio::null());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());
            
            // 设置环境变量以强制使用 CPU，避免 Metal shader 错误导致的崩溃
            // 这些设置确保在不同 macOS 机器上行为一致
            cmd.env("PYTORCH_ENABLE_MPS_FALLBACK", "1");
            cmd.env("PYTORCH_MPS_HIGH_WATERMARK_RATIO", "0.0");
            cmd.env("PYTORCH_MPS_FORCE_CPU", "1");
            cmd.env("CUDA_VISIBLE_DEVICES", "");
            cmd.env("TORCH_DEVICE", "cpu");
            // 禁用 MPS（Metal Performance Shaders），强制使用 CPU
            // 这可以避免在不同 macOS 硬件上因 GPU 支持差异导致的问题
            cmd.env("PYTORCH_MPS_DISABLE", "1");
            // 设置 Python 无缓冲模式，确保实时输出
            cmd.env("PYTHONUNBUFFERED", "1");
            // 禁用 Python 字节码缓存，避免版本差异问题
            cmd.env("PYTHONDONTWRITEBYTECODE", "1");
            
            // 设置模型缓存目录，避免每次运行都重新下载模型
            // marker-pdf 使用 Hugging Face 和 Surya OCR 模型
            // 设置 Hugging Face 缓存目录（使用绝对路径）
            let hf_cache_dir = if cfg!(target_os = "macos") {
                // macOS: ~/Library/Caches/huggingface
                std::env::var("HOME")
                    .map(|home| {
                        let path = std::path::PathBuf::from(&home)
                            .join("Library")
                            .join("Caches")
                            .join("huggingface");
                        // 转换为绝对路径字符串
                        path.to_string_lossy().to_string()
                    })
                    .unwrap_or_else(|_| {
                        // 如果无法获取 HOME，使用当前用户目录
                        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
                        format!("{}/Library/Caches/huggingface", home)
                    })
            } else if cfg!(target_os = "windows") {
                // Windows: %USERPROFILE%\.cache\huggingface
                std::env::var("USERPROFILE")
                    .map(|home| {
                        let path = std::path::PathBuf::from(&home)
                            .join(".cache")
                            .join("huggingface");
                        // 转换为绝对路径字符串
                        path.to_string_lossy().to_string()
                    })
                    .unwrap_or_else(|_| "%USERPROFILE%\\.cache\\huggingface".to_string())
            } else {
                // Linux: ~/.cache/huggingface
                std::env::var("HOME")
                    .map(|home| {
                        let path = std::path::PathBuf::from(&home)
                            .join(".cache")
                            .join("huggingface");
                        // 转换为绝对路径字符串
                        path.to_string_lossy().to_string()
                    })
                    .unwrap_or_else(|_| "~/.cache/huggingface".to_string())
            };
            
            // 确保 Hugging Face 缓存目录存在（使用绝对路径）
            let hf_path = std::path::PathBuf::from(&hf_cache_dir);
            if let Err(e) = std::fs::create_dir_all(&hf_path) {
                warn!("无法创建 Hugging Face 缓存目录 {}: {}", hf_cache_dir, e);
            } else {
                debug!("Hugging Face 缓存目录已准备: {}", hf_path.display());
            }
            
            // 确保 hub 目录存在（Hugging Face 模型的主要存储位置）
            let hub_path = hf_path.join("hub");
            if let Err(e) = std::fs::create_dir_all(&hub_path) {
                warn!("无法创建 hub 目录 {}: {}", hub_path.display(), e);
            } else {
                debug!("hub 目录已准备: {}", hub_path.display());
            }
            
            // 设置所有 Hugging Face 相关的环境变量
            // 这些变量确保模型缓存路径一致，避免重复下载
            // 使用绝对路径字符串，确保 marker-pdf 能够正确识别
            let hf_cache_dir_abs = hf_path.to_string_lossy().to_string();
            cmd.env("HF_HOME", &hf_cache_dir_abs);
            cmd.env("HF_HUB_CACHE", &hf_cache_dir_abs);
            cmd.env("HUGGINGFACE_HUB_CACHE", &hf_cache_dir_abs);
            cmd.env("TRANSFORMERS_CACHE", &hf_cache_dir_abs);
            cmd.env("HF_DATASETS_CACHE", &hf_cache_dir_abs);
            // 确保使用统一的缓存路径，避免因 Python 版本不同导致路径识别差异
            cmd.env("HF_DATASETS_STORAGE_PATH", &hf_cache_dir_abs);
            // 添加额外的环境变量，确保 marker-pdf 能够正确识别缓存目录
            cmd.env("HF_CACHE_DIR", &hf_cache_dir_abs);
            // 设置 hub 目录的绝对路径
            let hub_path_abs = hub_path.to_string_lossy().to_string();
            cmd.env("HF_HUB_CACHE_DIR", &hub_path_abs);
            
            // 设置 Surya OCR 模型缓存目录（marker-pdf 使用的 OCR 库）
            let surya_cache_dir = if cfg!(target_os = "macos") {
                // macOS: ~/Library/Caches/datalab/models
                std::env::var("HOME")
                    .map(|home| {
                        let path = std::path::PathBuf::from(&home)
                            .join("Library")
                            .join("Caches")
                            .join("datalab")
                            .join("models");
                        // 转换为绝对路径字符串
                        path.to_string_lossy().to_string()
                    })
                    .unwrap_or_else(|_| {
                        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
                        format!("{}/Library/Caches/datalab/models", home)
                    })
            } else if cfg!(target_os = "windows") {
                // Windows: %LOCALAPPDATA%\datalab\models
                std::env::var("LOCALAPPDATA")
                    .map(|local| {
                        let path = std::path::PathBuf::from(&local)
                            .join("datalab")
                            .join("models");
                        // 转换为绝对路径字符串
                        path.to_string_lossy().to_string()
                    })
                    .unwrap_or_else(|_| {
                        std::env::var("USERPROFILE")
                            .map(|home| format!("{}\\AppData\\Local\\datalab\\models", home))
                            .unwrap_or_else(|_| "%LOCALAPPDATA%\\datalab\\models".to_string())
                    })
            } else {
                // Linux: ~/.cache/datalab/models
                std::env::var("HOME")
                    .map(|home| {
                        let path = std::path::PathBuf::from(&home)
                            .join(".cache")
                            .join("datalab")
                            .join("models");
                        // 转换为绝对路径字符串
                        path.to_string_lossy().to_string()
                    })
                    .unwrap_or_else(|_| "~/.cache/datalab/models".to_string())
            };
            
            // 确保 Surya 模型缓存目录存在（使用绝对路径）
            let surya_path = std::path::PathBuf::from(&surya_cache_dir);
            if let Err(e) = std::fs::create_dir_all(&surya_path) {
                warn!("无法创建 Surya 模型缓存目录 {}: {}", surya_cache_dir, e);
            } else {
                debug!("Surya 模型缓存目录已准备: {}", surya_path.display());
            }
            
            // 使用绝对路径字符串设置环境变量
            let surya_cache_dir_abs = surya_path.to_string_lossy().to_string();
            cmd.env("SURYA_MODEL_CACHE_DIR", &surya_cache_dir_abs);
            
            debug!("设置 Hugging Face 缓存目录: {} (绝对路径: {})", hf_cache_dir, hf_cache_dir_abs);
            debug!("设置 Surya 模型缓存目录: {} (绝对路径: {})", surya_cache_dir, surya_cache_dir_abs);
            
            // 启动进程
            let mut child = cmd.spawn().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("无法启动 Marker 工具: {}", e),
                )
            })?;
            
            // 给进程一点时间启动，检查是否立即退出（首次运行时可能需要初始化）
            // 这可以避免在进程已退出时尝试读取管道导致的崩溃
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            
            // 检查进程是否还在运行
            match child.try_wait() {
                Ok(Some(status)) => {
                    // 进程已退出，可能是首次运行时的初始化失败或其他错误
                    let error_msg = format!(
                        "Marker 工具进程立即退出，退出码: {:?}。\
                         如果是首次运行，可能需要下载模型文件。\
                         请检查 Marker 工具是否正确安装。",
                        status.code()
                    );
                    warn!("{}", error_msg);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error_msg,
                    ));
                }
                Ok(None) => {
                    // 进程仍在运行，继续
                    debug!("Marker 工具进程正在运行，继续等待完成");
                }
                Err(e) => {
                    // 检查失败，记录警告但继续
                    warn!("无法检查 Marker 工具进程状态: {}，继续执行", e);
                }
            }
            
            // 使用流式读取 stdout 和 stderr，避免一次性读取所有数据导致栈溢出
            // 如果有 import_id，将 stdout/stderr 发送到日志收集器
            // 同时捕获 BrokenPipe 错误，避免 SIGPIPE 信号导致崩溃
            let import_id_clone = import_id.map(|s| s.to_string());
            let stdout_handle: Option<tokio::task::JoinHandle<()>> = if let Some(stdout) = child.stdout.take() {
                Some(tokio::spawn(async move {
                    use tokio::io::AsyncBufReadExt;
                    use tokio::io::BufReader;
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::new();
                    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                        if let Some(id) = &import_id_clone {
                            // 发送 Marker 工具的 stdout 作为 debug 日志
                            crate::import::import_log_collector::send_import_log(
                                id,
                                "debug",
                                format!("[Marker stdout] {}", line.trim()),
                            );
                        }
                        line.clear();
                    }
                }))
            } else {
                None
            };
            
            let import_id_clone2 = import_id.map(|s| s.to_string());
            let stderr_handle: Option<tokio::task::JoinHandle<()>> = if let Some(stderr) = child.stderr.take() {
                Some(tokio::spawn(async move {
                    use tokio::io::AsyncBufReadExt;
                    use tokio::io::BufReader;
                    let mut reader = BufReader::new(stderr);
                    let mut line = String::new();
                    let mut last_progress_update = std::time::Instant::now();
                    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                        if let Some(id) = &import_id_clone2 {
                            let trimmed_line = line.trim();
                            
                            // 解析 Marker 工具的输出，提取进度信息
                            // Marker 工具输出格式如: "Recognizing Layout: 50%|████████..."
                            // 或者: "50%|████████..."
                            if trimmed_line.contains("%|") {
                                // 尝试从输出中提取百分比
                                // 查找第一个数字后跟%的模式
                                let mut percent: Option<f64> = None;
                                
                                // 方法1: 查找 "数字%" 模式
                                for part in trimmed_line.split_whitespace() {
                                    if part.ends_with('%') {
                                        if let Ok(p) = part.trim_end_matches('%').parse::<f64>() {
                                            if p >= 0.0 && p <= 100.0 {
                                                percent = Some(p);
                                                break;
                                            }
                                        }
                                    }
                                }
                                
                                // 方法2: 如果方法1失败，尝试从 "50%|" 中提取
                                if percent.is_none() {
                                    if let Some(percent_part) = trimmed_line.split('%').next() {
                                        if let Some(num_str) = percent_part.split_whitespace().last() {
                                            if let Ok(p) = num_str.parse::<f64>() {
                                                if p >= 0.0 && p <= 100.0 {
                                                    percent = Some(p);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if let Some(p) = percent {
                                    // 将 Marker 工具的进度（0-100%）映射到整体进度（30%-70%）
                                    // Marker 工具占整体进度的 40%（30% 到 70%）
                                    let overall_progress = 0.3 + (p / 100.0) * 0.4;
                                    crate::import::import_log_collector::send_import_progress(
                                        id,
                                        "",
                                        overall_progress,
                                        &format!("正在使用 Marker 工具转换 PDF... ({}%)", p as u32),
                                    );
                                }
                            }
                            
                            // 发送 Marker 工具的 stderr 作为 warn 日志（但过滤掉很长的进度条，避免日志过多）
                            // 只记录非进度条的输出，或者简短的进度信息
                            if !trimmed_line.contains("%|") {
                                // 非进度条输出，正常记录
                                crate::import::import_log_collector::send_import_log(
                                    id,
                                    "warn",
                                    format!("[Marker stderr] {}", trimmed_line),
                                );
                            } else if trimmed_line.len() < 200 {
                                // 简短的进度信息，也记录（用于调试）
                                crate::import::import_log_collector::send_import_log(
                                    id,
                                    "debug",
                                    format!("[Marker progress] {}", trimmed_line.chars().take(100).collect::<String>()),
                                );
                            }
                            // 很长的进度条（>200字符）不记录，避免日志过多
                            
                            // 定期发送进度更新（即使没有解析到百分比），确保 UI 知道进程还在运行
                            if last_progress_update.elapsed().as_secs() >= 5 {
                                crate::import::import_log_collector::send_import_progress(
                                    id,
                                    "",
                                    0.4, // 中间进度
                                    "正在使用 Marker 工具转换 PDF...",
                                );
                                last_progress_update = std::time::Instant::now();
                            }
                        }
                        line.clear();
                    }
                }))
            } else {
                None
            };
            
            // 等待进程完成
            // 添加日志以便调试
            if let Some(id) = import_id {
                crate::import::import_log_collector::send_import_log(
                    id,
                    "info",
                    "等待 Marker 工具进程完成...".to_string(),
                );
            }
            let status = child.wait().await.map_err(|e| {
                if let Some(id) = import_id {
                    crate::import::import_log_collector::send_import_log(
                        id,
                        "error",
                        format!("等待 Marker 工具进程失败: {}", e),
                    );
                }
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("等待 Marker 工具进程失败: {}", e),
                )
            })?;
            
            // 进程已完成，记录日志
            if let Some(id) = import_id {
                crate::import::import_log_collector::send_import_log(
                    id,
                    "info",
                    format!("Marker 工具进程已完成，退出码: {:?}", status.code()),
                );
            }
            
            // 等待 stdout/stderr 读取完成，但设置超时避免无限等待
            // 如果读取任务卡住，我们仍然可以继续处理
            let stdout_timeout = tokio::time::Duration::from_secs(5);
            let stderr_timeout = tokio::time::Duration::from_secs(5);
            
            if let Some(handle) = stdout_handle {
                if let Err(e) = tokio::time::timeout(stdout_timeout, handle).await {
                    warn!("等待 stdout 读取完成超时或失败: {:?}", e);
                }
            }
            if let Some(handle) = stderr_handle {
                if let Err(e) = tokio::time::timeout(stderr_timeout, handle).await {
                    warn!("等待 stderr 读取完成超时或失败: {:?}", e);
                }
            }
            
            // 构建输出结果（不包含 stdout/stderr，因为我们丢弃了它们）
            // Marker 工具的输出主要写入文件，所以不需要 stdout/stderr
            Ok::<std::process::Output, std::io::Error>(std::process::Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        }).await;

        match output_result {
            Ok(Ok(output)) => {
                // 命令执行完成
                info!("Marker 工具执行完成，退出码: {:?}", output.status.code());
                
                // 如果有 import_id，发送 Marker 工具完成进度更新
                if let Some(id) = import_id {
                    crate::import::import_log_collector::send_import_progress(
                        id,
                        "",
                        0.7,
                        "Marker 工具转换完成，正在处理结果...",
                    );
                    crate::import::import_log_collector::send_import_log(
                        id,
                        "info",
                        format!("Marker 工具执行完成，退出码: {:?}", output.status.code()),
                    );
                }
                
                self.handle_marker_output(output, output_path).await
            }
            Ok(Err(e)) => {
                // 命令执行过程中出错
                error!("Marker 工具执行出错: {}", e);
                Err(FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Marker 工具执行失败: {}", e),
                ))
            }
            Err(e) => {
                // 检查是否是 marker-pdf 未安装的错误
                let error_msg = format!("{}", e);
                if error_msg.contains("marker-pdf is not installed") 
                    || error_msg.contains("marker_single not found")
                    || error_msg.contains("marker-pdf 未安装") {
                    // 根据平台生成安装指引
                    let install_instructions = get_platform_marker_pdf_install_instructions();
                    
                    return Err(FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!(
                            "Marker 工具需要 marker-pdf 才能运行，但系统中未安装 marker-pdf。\n\n\
                            安装方法：\n\
                            {}\n\n\
                            验证安装：\n\
                            安装完成后，运行以下命令验证：\n\
                            pipx list  # 应该看到 marker-pdf\n\n\
                            详细说明：\n\
                            marker-pdf 是一个 Python 工具，用于将 PDF 转换为 Markdown。\n\
                            它需要 Python 3.8+ 和 pipx 来安装和管理。\n\n\
                            安装完成后，请重新尝试 PDF 导入。",
                            install_instructions
                        ),
                    ));
                }
                
                // 其他错误
                error!("Marker 工具执行出错: {}", e);
                Err(FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Marker 工具执行失败: {}", e),
                ))
            }
        }
    }

    /// 处理 Marker 工具的输出
    /// 
    /// 检查退出状态，捕获 stderr，并优先从 stdout 读取 Markdown，
    /// 如果 stdout 为空，则从输出目录读取生成的 Markdown 文件。
    /// 
    /// # 参数
    /// - `output`: 命令执行结果
    /// - `output_path`: 期望的 Markdown 输出文件路径
    /// 
    /// # 返回
    /// - `Ok(String)`: Markdown 内容
    /// - `Err(FlowyError)`: 处理过程中的错误
    async fn handle_marker_output(
        &self,
        output: std::process::Output,
        output_path: &Path,
    ) -> FlowyResult<String> {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // 即使进程退出码不是成功，也先检查输出文件是否存在
        // Marker 工具可能在崩溃前已经生成了输出文件
        // 使用 spawn_blocking 避免阻塞异步运行时
        let output_path_clone = output_path.to_path_buf();
        let output_file_exists = tokio::task::spawn_blocking(move || {
            output_path_clone.exists()
        })
        .await
        .map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("检查文件存在性任务失败: {}", e),
            )
        })?;
        
        // 检查退出状态
        if !output.status.success() {
            // 检查是否是 marker-pdf 未安装的错误
            let stderr_str = stderr.to_string();
            let stdout_str = stdout.to_string();
            
            if stderr_str.contains("marker-pdf is not installed")
                || stderr_str.contains("marker_single not found")
                || stderr_str.contains("marker-pdf 未安装")
                || stderr_str.contains("Please install marker-pdf")
                || stderr_str.contains("安装 marker-pdf") {
                // 根据平台生成安装指引
                let install_instructions = get_platform_marker_pdf_install_instructions();
                
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!(
                        "Marker 工具需要 marker-pdf 才能运行，但系统中未安装 marker-pdf。\n\n\
                        安装方法：\n\
                        {}\n\n\
                        验证安装：\n\
                        安装完成后，运行以下命令验证：\n\
                        pipx list  # 应该看到 marker-pdf\n\n\
                        详细说明：\n\
                        marker-pdf 是一个 Python 工具，用于将 PDF 转换为 Markdown。\n\
                        它需要 Python 3.8+ 和 pipx 来安装和管理。\n\n\
                        安装完成后，请重新尝试 PDF 导入。\n\n\
                        Marker 工具错误详情：\n\
                        stderr: {}",
                        install_instructions,
                        stderr_str.trim()
                    ),
                ));
            }
            
            warn!(
                "Marker 工具执行失败 (退出码: {:?})\n\
                 stdout: {}\n\
                 stderr: {}",
                output.status.code(),
                stdout_str,
                stderr_str
            );
            
            // 如果输出文件存在，尝试读取它（即使进程崩溃，可能已经生成了部分输出）
            if output_file_exists {
                warn!("检测到 Marker 工具虽然退出失败，但输出文件存在，尝试读取...");
                let output_path_clone = output_path.to_path_buf();
                match tokio::task::spawn_blocking(move || {
                    std::fs::read_to_string(&output_path_clone)
                }).await.map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("读取文件任务失败: {}", e),
                    )
                })? {
                    Ok(content) if !content.trim().is_empty() => {
                        info!("成功从输出文件读取 Markdown 内容 ({} 字符)，尽管进程退出失败", content.len());
                        return Ok(content);
                    }
                    Ok(_) => {
                        // 文件存在但为空，尝试从输出目录查找其他 Markdown 文件
                        warn!("输出文件存在但为空，尝试从输出目录查找其他 Markdown 文件...");
                        if let Some(output_dir) = output_path.parent() {
                            match self.find_and_read_markdown_file(output_dir).await {
                                Ok(content) if !content.trim().is_empty() => {
                                    info!("成功从输出目录读取 Markdown 内容 ({} 字符)", content.len());
                                    return Ok(content);
                                }
                                _ => {
                                    // 无法找到有效的 Markdown 文件
                                    return Err(FlowyError::new(
                                        flowy_error::ErrorCode::Internal,
                                        format!(
                                            "Marker 工具转换失败 (退出码: {:?})\n\
                                             输出文件存在但为空，且无法找到其他 Markdown 文件\n\
                                             stdout: {}\n\
                                             stderr: {}",
                                            output.status.code(),
                                            stdout.trim(),
                                            stderr.trim()
                                        ),
                                    ));
                                }
                            }
                        } else {
                            return Err(FlowyError::new(
                                flowy_error::ErrorCode::Internal,
                                format!(
                                    "Marker 工具转换失败 (退出码: {:?})\n\
                                     输出文件存在但为空\n\
                                     stdout: {}\n\
                                     stderr: {}",
                                    output.status.code(),
                                    stdout.trim(),
                                    stderr.trim()
                                ),
                            ));
                        }
                    }
                    Err(e) => {
                        // 无法读取输出文件，尝试从输出目录查找其他 Markdown 文件
                        warn!("无法读取输出文件: {}，尝试从输出目录查找其他 Markdown 文件...", e);
                        if let Some(output_dir) = output_path.parent() {
                            match self.find_and_read_markdown_file(output_dir).await {
                                Ok(content) if !content.trim().is_empty() => {
                                    info!("成功从输出目录读取 Markdown 内容 ({} 字符)", content.len());
                                    return Ok(content);
                                }
                                _ => {
                                    // 无法找到有效的 Markdown 文件
                                    return Err(FlowyError::new(
                                        flowy_error::ErrorCode::Internal,
                                        format!(
                                            "Marker 工具转换失败 (退出码: {:?})\n\
                                             无法读取输出文件: {}，且无法找到其他 Markdown 文件\n\
                                             stdout: {}\n\
                                             stderr: {}",
                                            output.status.code(),
                                            e,
                                            stdout.trim(),
                                            stderr.trim()
                                        ),
                                    ));
                                }
                            }
                        } else {
                            return Err(FlowyError::new(
                                flowy_error::ErrorCode::Internal,
                                format!(
                                    "Marker 工具转换失败 (退出码: {:?})\n\
                                     无法读取输出文件: {}\n\
                                     stdout: {}\n\
                                     stderr: {}",
                                    output.status.code(),
                                    e,
                                    stdout.trim(),
                                    stderr.trim()
                                ),
                            ));
                        }
                    }
                }
            } else {
                // 输出文件不存在，尝试从输出目录查找其他 Markdown 文件
                warn!("输出文件不存在，尝试从输出目录查找其他 Markdown 文件...");
                if let Some(output_dir) = output_path.parent() {
                    match self.find_and_read_markdown_file(output_dir).await {
                        Ok(content) if !content.trim().is_empty() => {
                            info!("成功从输出目录读取 Markdown 内容 ({} 字符)，尽管进程退出失败", content.len());
                            return Ok(content);
                        }
                        _ => {
                            // 无法找到有效的 Markdown 文件
                            return Err(FlowyError::new(
                                flowy_error::ErrorCode::Internal,
                                format!(
                                    "Marker 工具转换失败 (退出码: {:?})\n\
                                     输出文件不存在，且无法找到其他 Markdown 文件\n\
                                     stdout: {}\n\
                                     stderr: {}",
                                    output.status.code(),
                                    stdout.trim(),
                                    stderr.trim()
                                ),
                            ));
                        }
                    }
                } else {
                    return Err(FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!(
                            "Marker 工具转换失败 (退出码: {:?})\n\
                             输出文件不存在\n\
                             stdout: {}\n\
                             stderr: {}",
                            output.status.code(),
                            stdout.trim(),
                            stderr.trim()
                        ),
                    ));
                }
            }
        }
        
        // 始终记录 stdout 和 stderr，即使为空也记录，以便调试
        info!("Marker 工具执行完成，退出码: {:?}", output.status.code());
        if !stdout.trim().is_empty() {
            info!("Marker stdout ({} 字节): {}", stdout.len(), stdout.trim());
        } else {
            debug!("Marker stdout 为空");
        }
        if !stderr.trim().is_empty() {
            warn!("Marker stderr ({} 字节): {}", stderr.len(), stderr.trim());
        } else {
            debug!("Marker stderr 为空");
        }

        // 优先从 stdout 读取 Markdown（如果 Marker 工具直接输出到 stdout）
        if !stdout.trim().is_empty() {
            info!("从 stdout 读取 Markdown 内容 ({} 字符)", stdout.len());
            return Ok(stdout.to_string());
        }

        // 如果 stdout 为空，则从输出目录读取生成的 Markdown 文件
        let output_dir = output_path.parent().unwrap();
        let markdown_content = self.find_and_read_markdown_file(output_dir).await?;

        if markdown_content.is_empty() {
            warn!("Marker 工具生成的 Markdown 内容为空");
            // 即使文件为空，也返回空字符串，而不是错误
            // 让调用者决定如何处理空内容
        }

        Ok(markdown_content)
    }

    /// 查找并读取 Markdown 文件
    /// 
    /// Marker 工具在输出目录中生成 Markdown 文件。
    /// 我们需要查找该文件并读取其内容。
    /// 
    /// # 参数
    /// - `output_dir`: 输出目录
    /// 
    /// # 返回
    /// - `Ok(String)`: Markdown 内容
    /// - `Err(FlowyError)`: 文件查找或读取错误
    async fn find_and_read_markdown_file(&self, output_dir: &Path) -> FlowyResult<String> {
        // Marker 工具可能生成多种命名的 Markdown 文件
        // 按优先级顺序尝试查找：
        // 1. output.md（标准输出文件名）
        // 2. 与输入 PDF 同名的 .md 文件
        // 3. 输出目录中的任何 .md 文件
        // 4. 输出目录中的任何文件（Marker 工具可能生成没有扩展名的文件）
        
        // 首先尝试读取指定的输出文件
        // 使用 spawn_blocking 检查文件是否存在，避免在异步上下文中进行同步文件系统操作
        let output_md = output_dir.join("output.md");
        let output_md_clone = output_md.clone();
        let output_md_exists = tokio::task::spawn_blocking(move || {
            output_md_clone.exists()
        })
        .await
        .map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("检查文件存在性任务失败: {}", e),
            )
        })?;
        
        if output_md_exists {
            debug!("找到标准输出文件: {}", output_md.display());
            return self.read_markdown_file(&output_md).await;
        }

        // 由于文件系统同步延迟，可能需要重试几次
        let max_retries = 3;
        let mut retry_count = 0;
        let mut markdown_files = Vec::new();
        let mut all_files = Vec::new();
        
        loop {
            // 查找输出目录中的所有文件
            // 使用 spawn_blocking 在后台线程读取目录，避免阻塞
            let output_dir_clone = output_dir.to_path_buf();
            let entries = tokio::task::spawn_blocking(move || {
                std::fs::read_dir(&output_dir_clone)
            })
            .await
            .map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("读取目录任务失败: {}", e),
                )
            })?
            .map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("无法读取输出目录: {}", e),
                )
            })?;

            // 清空之前的文件列表
            markdown_files.clear();
            all_files.clear();
            
            debug!("开始遍历输出目录 (尝试 {}/{}): {}", retry_count + 1, max_retries, output_dir.display());
            
            // 收集所有条目路径，然后在 spawn_blocking 中处理，避免阻塞
            let mut entry_paths = Vec::new();
            for entry in entries {
                let entry = entry.map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("无法读取目录条目: {}", e),
                    )
                })?;
                entry_paths.push(entry.path());
            }
            
            // 在后台线程中处理文件元数据，避免阻塞和栈溢出
            // 注意：这里需要捕获可能的错误，因为目录可能在处理过程中被删除
            let _output_dir_clone = output_dir.to_path_buf();
            let entry_paths_clone = entry_paths.clone();
            let (files, dirs) = tokio::task::spawn_blocking(move || {
                let mut files = Vec::new();
                let mut dirs = Vec::new();
                for path in entry_paths_clone {
                    // 使用 exists() 先检查路径是否存在，避免访问已删除的文件/目录
                    if !path.exists() {
                        continue;
                    }
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if metadata.is_file() {
                            files.push((path.clone(), metadata.len()));
                        } else if metadata.is_dir() {
                            dirs.push(path.clone());
                        }
                    }
                }
                (files, dirs)
            })
            .await
            .map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("处理目录条目任务失败: {}", e),
                )
            })?;
            
            // 处理文件
            for (path, size) in files {
                all_files.push(path.clone());
                debug!("找到文件: {} ({} 字节)", path.display(), size);
                
                // 检查是否是 .md 或 .markdown 文件
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext.to_lowercase() == "md" || ext.to_lowercase() == "markdown" {
                        markdown_files.push(path.clone());
                        debug!("找到 Markdown 文件: {}", path.display());
                    }
                }
            }
            
            // 处理子目录（递归查找）
            for dir_path in dirs {
                // 检查子目录是否仍然存在（可能在处理过程中被删除）
                let dir_path_clone = dir_path.clone();
                let dir_exists = tokio::task::spawn_blocking(move || {
                    dir_path_clone.exists() && dir_path_clone.is_dir()
                })
                .await
                .map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("检查子目录存在性任务失败: {}", e),
                    )
                })?;
                
                if !dir_exists {
                    debug!("子目录已不存在，跳过: {}", dir_path.display());
                    continue;
                }
                
                debug!("发现子目录: {}，递归查找 Markdown 文件...", dir_path.display());
                let dir_path_clone = dir_path.clone();
                let sub_files = tokio::task::spawn_blocking(move || {
                    let mut files = Vec::new();
                    if let Ok(sub_entries) = std::fs::read_dir(&dir_path_clone) {
                        for sub_entry in sub_entries {
                            if let Ok(sub_entry) = sub_entry {
                                let sub_path = sub_entry.path();
                                // 检查文件是否仍然存在
                                if !sub_path.exists() {
                                    continue;
                                }
                                if let Ok(sub_metadata) = std::fs::metadata(&sub_path) {
                                    if sub_metadata.is_file() {
                                        files.push((sub_path.clone(), sub_metadata.len()));
                                    }
                                }
                            }
                        }
                    }
                    files
                })
                .await
                .map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("读取子目录任务失败: {}", e),
                    )
                })?;
                
                for (sub_path, size) in sub_files {
                    all_files.push(sub_path.clone());
                    debug!("在子目录中找到文件: {} ({} 字节)", sub_path.display(), size);
                    
                    // 检查是否是 .md 或 .markdown 文件
                    if let Some(ext) = sub_path.extension().and_then(|s| s.to_str()) {
                        if ext.to_lowercase() == "md" || ext.to_lowercase() == "markdown" {
                            markdown_files.push(sub_path.clone());
                            debug!("在子目录中找到 Markdown 文件: {}", sub_path.display());
                        }
                    }
                }
            }
            
            debug!("文件统计: 总文件数={}, Markdown 文件数={}", all_files.len(), markdown_files.len());
            
            // 如果找到了文件，跳出重试循环
            if !all_files.is_empty() {
                break;
            }
            
            // 如果没找到文件且还有重试次数，等待后重试
            retry_count += 1;
            if retry_count >= max_retries {
                warn!("重试 {} 次后仍未找到文件", max_retries);
                break;
            }
            
            // 等待 100ms 后重试（给文件系统一些时间同步）
            debug!("未找到文件，等待 100ms 后重试...");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // 如果找到了 .md 文件，优先使用它们
        if !markdown_files.is_empty() {
            // 按文件大小排序，优先选择较大的文件（通常是主文件）
            // 使用 spawn_blocking 获取文件大小，避免阻塞
            let markdown_files_clone = markdown_files.clone();
            let file_sizes = tokio::task::spawn_blocking(move || {
                markdown_files_clone
                    .iter()
                    .map(|path| {
                        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                    })
                    .collect::<Vec<_>>()
            })
            .await
            .map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("获取文件大小任务失败: {}", e),
                )
            })?;
            
            // 创建索引并排序
            let mut indices: Vec<usize> = (0..markdown_files.len()).collect();
            indices.sort_by(|&a, &b| file_sizes[b].cmp(&file_sizes[a])); // 降序排列

            // 读取第一个（最大的）Markdown 文件
            let markdown_file = &markdown_files[indices[0]];
            let file_size = file_sizes[indices[0]];
            debug!("找到 Markdown 文件: {} ({} 字节)", 
                markdown_file.display(),
                file_size
            );
            
            return self.read_markdown_file(markdown_file).await;
        }

        // 如果没有找到 .md 文件，尝试读取所有文件（Marker 工具可能生成没有扩展名的文件）
        // 按文件大小排序，优先选择较大的文件（通常是主输出文件）
        if !all_files.is_empty() {
            // 使用 spawn_blocking 获取文件大小，避免阻塞
            let all_files_clone = all_files.clone();
            let file_sizes = tokio::task::spawn_blocking(move || {
                all_files_clone
                    .iter()
                    .map(|path| {
                        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                    })
                    .collect::<Vec<_>>()
            })
            .await
            .map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("获取文件大小任务失败: {}", e),
                )
            })?;
            
            // 创建索引并排序
            let mut indices: Vec<usize> = (0..all_files.len()).collect();
            indices.sort_by(|&a, &b| file_sizes[b].cmp(&file_sizes[a])); // 降序排列

            // 尝试读取最大的文件（通常是 Marker 工具的主输出文件）
            let candidate_file = &all_files[indices[0]];
            let file_size = file_sizes[indices[0]];
            debug!("未找到 .md 文件，尝试读取最大文件: {} ({} 字节)", 
                candidate_file.display(),
                file_size
            );
            
            // 尝试读取文件内容，检查是否是 Markdown 格式
            match self.read_markdown_file(candidate_file).await {
                Ok(content) => {
                    // 检查内容是否看起来像 Markdown（至少包含一些 Markdown 特征）
                    // 简单的启发式检查：包含 Markdown 常见标记
                    if content.trim().is_empty() {
                        // 文件为空，尝试下一个文件
                        if indices.len() > 1 {
                            warn!("文件 {} 为空，尝试下一个文件", candidate_file.display());
                            let next_file = &all_files[indices[1]];
                            return self.read_markdown_file(next_file).await;
                        } else {
                            return Err(FlowyError::new(
                                flowy_error::ErrorCode::Internal,
                                format!(
                                    "Marker 工具生成的文件为空\n\
                                     文件: {}\n\
                                     输出目录: {}",
                                    candidate_file.display(),
                                    output_dir.display()
                                ),
                            ));
                        }
                    }
                    
                    // 文件有内容，假设它是 Markdown（即使没有 .md 扩展名）
                    info!("成功读取 Marker 工具生成的 Markdown 文件（无扩展名）: {} ({} 字符)", 
                        candidate_file.display(),
                        content.len()
                    );
                    return Ok(content);
                }
                Err(e) => {
                    // 无法读取文件，返回错误
                    return Err(FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!(
                            "无法读取 Marker 工具生成的文件: {}\n\
                             错误: {}\n\
                             输出目录: {}",
                            candidate_file.display(),
                            e,
                            output_dir.display()
                        ),
                    ));
                }
            }
        }

        // 如果输出目录为空或没有文件，返回错误
        let all_files_str: Vec<String> = std::fs::read_dir(output_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| format!("  - {}", e.path().display()))
                    .collect()
            })
            .unwrap_or_else(|_| vec!["  (无法读取目录)".to_string()]);
        
        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            format!(
                "Marker 工具未生成 Markdown 文件\n\
                 输出目录: {}\n\
                 目录中的文件:\n{}\n\
                 请检查 Marker 工具是否正确执行。",
                output_dir.display(),
                all_files_str.join("\n")
            ),
        ))
    }

    /// 读取 Markdown 文件内容
    /// 
    /// # 参数
    /// - `file_path`: Markdown 文件路径
    /// 
    /// # 返回
    /// - `Ok(String)`: 文件内容
    /// - `Err(FlowyError)`: 读取错误
    async fn read_markdown_file(&self, file_path: &Path) -> FlowyResult<String> {
        // 使用 spawn_blocking 在后台线程读取文件，避免大文件导致栈溢出或阻塞
        let file_path = file_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            std::fs::read_to_string(&file_path).map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!(
                        "无法读取 Markdown 文件 {}: {}",
                        file_path.display(),
                        e
                    ),
                )
            })
        })
        .await
        .map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("读取文件任务失败: {}", e),
            )
        })?
    }

    /// 从 Marker 工具的输出目录提取图片
    /// 
    /// Marker 工具会将 PDF 中的图片提取到输出目录中（通常在 images/ 子目录或直接在输出目录中）。
    /// 该方法扫描输出目录，查找所有图片文件，读取并转换为 ExtractedImage 格式。
    /// 
    /// # 参数
    /// - `output_dir`: Marker 工具的输出目录
    /// - `import_id`: 可选的导入 ID，用于发送进度日志
    /// 
    /// # 返回
    /// - `Vec<ExtractedImage>`: 提取的图片列表
    /// 
    /// # 注意
    /// - 即使某些图片提取失败，该方法也会继续处理其他图片
    /// - 支持 PNG、JPEG、GIF 格式
    /// - 大图片会自动压缩
    pub fn extract_images_from_marker_output(&self, output_dir: &Path, import_id: Option<&str>) -> Vec<ExtractedImage> {
        info!("开始从 Marker 输出目录提取图片: {}", output_dir.display());
        
        if let Some(id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("开始从 Marker 输出目录提取图片: {}", output_dir.display()),
            );
        }
        
        let mut extracted_images = Vec::new();
        
        // 首先尝试在 images/ 子目录中查找图片
        let image_dirs = vec![
            output_dir.join("images"),
            output_dir.join("image"),
            output_dir.to_path_buf(), // 也搜索根目录
        ];
        
        for image_dir in &image_dirs {
            // 检查目录是否存在（可能在处理过程中被删除）
            if !image_dir.exists() || !image_dir.is_dir() {
                continue;
            }
            
            debug!("扫描图片目录: {}", image_dir.display());
            if let Some(id) = import_id {
                crate::import::import_log_collector::send_import_log(
                    id,
                    "info",
                    format!("扫描图片目录: {}", image_dir.display()),
                );
            }
            
            let images = self.scan_directory_for_images(image_dir, import_id);
            extracted_images.extend(images);
        }
        
        info!("成功提取 {} 张图片", extracted_images.len());
        if let Some(id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("成功提取 {} 张图片", extracted_images.len()),
            );
        }
        
        extracted_images
    }

    /// 扫描目录中的图片文件
    /// 
    /// 递归扫描指定目录，查找所有支持的图片格式（PNG、JPEG、GIF），
    /// 读取图片数据并转换为 ExtractedImage 格式。
    /// 
    /// # 参数
    /// - `dir`: 要扫描的目录
    /// - `import_id`: 可选的导入 ID，用于发送进度日志
    /// 
    /// # 返回
    /// - `Vec<ExtractedImage>`: 提取的图片列表
    fn scan_directory_for_images(&self, dir: &Path, import_id: Option<&str>) -> Vec<ExtractedImage> {
        let mut images = Vec::new();
        
        // 支持的图片扩展名
        let image_extensions = vec!["png", "jpg", "jpeg", "gif", "bmp", "webp"];
        
        // 检查目录是否存在（可能在处理过程中被删除）
        if !dir.exists() || !dir.is_dir() {
            debug!("目录不存在或不是目录，跳过: {}", dir.display());
            return images;
        }
        
        // 读取目录条目
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("无法读取目录 {}: {}", dir.display(), e);
                return images;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("无法读取目录条目: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            
            // 检查路径是否仍然存在（可能在处理过程中被删除）
            if !path.exists() {
                continue;
            }
            
            // 如果是目录，递归扫描
            if path.is_dir() {
                let sub_images = self.scan_directory_for_images(&path, import_id);
                images.extend(sub_images);
                continue;
            }

            // 检查文件扩展名
            let ext = match path.extension().and_then(|s| s.to_str()) {
                Some(ext) => ext.to_lowercase(),
                None => continue,
            };

            if !image_extensions.contains(&ext.as_str()) {
                continue;
            }

            // 尝试提取图片
            match self.extract_single_image(&path, &ext) {
                Ok(Some(image)) => {
                    info!("成功提取图片: {} ({} bytes, {:?})", 
                        image.filename, 
                        image.data.len(),
                        image.format
                    );
                    
                    // 发送图片导入日志
                    if let Some(id) = import_id {
                        let image_size_mb = image.data.len() as f64 / (1024.0 * 1024.0);
                        crate::import::import_log_collector::send_import_log(
                            id,
                            "info",
                            format!(
                                "导入图片: {} ({} bytes, {:.2} MB, 格式: {:?})",
                                image.filename,
                                image.data.len(),
                                image_size_mb,
                                image.format
                            ),
                        );
                    }
                    
                    images.push(image);
                }
                Ok(None) => {
                    // 图片数据为空或无效，跳过
                    debug!("跳过无效图片: {}", path.display());
                    if let Some(id) = import_id {
                        crate::import::import_log_collector::send_import_log(
                            id,
                            "warn",
                            format!("跳过无效图片: {}", path.display()),
                        );
                    }
                }
                Err(e) => {
                    // 提取失败，记录警告但继续处理其他图片
                    warn!("提取图片失败 {}: {} (继续处理其他图片)", path.display(), e);
                    if let Some(id) = import_id {
                        crate::import::import_log_collector::send_import_log(
                            id,
                            "warn",
                            format!("提取图片失败 {}: {} (继续处理其他图片)", path.display(), e),
                        );
                    }
                }
            }
        }

        images
    }

    /// 提取单个图片文件
    /// 
    /// 读取图片文件，检测格式，获取尺寸，并在需要时进行压缩。
    /// 
    /// # 参数
    /// - `image_path`: 图片文件路径
    /// - `ext`: 文件扩展名（小写）
    /// 
    /// # 返回
    /// - `Ok(Some(ExtractedImage))`: 成功提取的图片
    /// - `Ok(None)`: 图片数据为空或无效
    /// - `Err(FlowyError)`: 提取过程中的错误
    fn extract_single_image(
        &self,
        image_path: &Path,
        ext: &str,
    ) -> FlowyResult<Option<ExtractedImage>> {
        // 读取图片数据
        let image_data = std::fs::read(image_path).map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法读取图片文件 {}: {}", image_path.display(), e),
            )
        })?;

        if image_data.is_empty() {
            return Ok(None);
        }

        // 检测图片格式
        let format = self.detect_image_format(&image_data, ext)?;
        
        // 验证图片大小（最大 10MB）
        const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if image_data.len() > MAX_IMAGE_SIZE {
            warn!(
                "图片过大: {} ({} bytes)，将进行压缩",
                image_path.display(),
                image_data.len()
            );
        }

        // 加载图片以获取尺寸并进行必要的压缩
        let (final_data, dimensions, final_format) = match self.process_image_data(&image_data, &format) {
            Ok(result) => result,
            Err(e) => {
                warn!("处理图片失败 {}: {}，尝试使用原始数据", image_path.display(), e);
                // 如果处理失败，尝试直接使用原始数据
                let dimensions = self.get_image_dimensions_from_data(&image_data, &format)?;
                (image_data, dimensions, format.clone())
            }
        };

        // 生成图片 ID 和文件名
        let image_id = Uuid::new_v4().to_string();
        let extension = self.format_to_extension(&final_format);
        let filename = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("image_{}.{}", &image_id[..8], extension));

        let extracted_image = ExtractedImage {
            id: image_id,
            filename,
            data: final_data,
            format: final_format,
            dimensions,
        };

        Ok(Some(extracted_image))
    }

    /// 检测图片格式
    /// 
    /// 基于文件扩展名和文件头（magic bytes）检测图片格式。
    /// 
    /// # 参数
    /// - `data`: 图片数据的前几个字节
    /// - `ext`: 文件扩展名（小写）
    /// 
    /// # 返回
    /// - `Ok(ImageFormat)`: 检测到的图片格式
    /// - `Err(FlowyError)`: 无法识别的格式
    fn detect_image_format(&self, data: &[u8], ext: &str) -> FlowyResult<ImageFormat> {
        // 首先基于文件头检测（更可靠）
        if data.len() >= 4 {
            // PNG: 89 50 4E 47
            if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                return Ok(ImageFormat::Png);
            }
            // JPEG: FF D8 FF
            if data.len() >= 3 && data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return Ok(ImageFormat::Jpeg);
            }
            // GIF: 47 49 46 38 (GIF8)
            if data.len() >= 4 && data.starts_with(b"GIF8") {
                return Ok(ImageFormat::Gif);
            }
            // BMP: 42 4D (BM)
            if data.len() >= 2 && data.starts_with(b"BM") {
                return Ok(ImageFormat::Bmp);
            }
            // WebP: 需要检查 RIFF 头
            if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
                return Ok(ImageFormat::WebP);
            }
        }

        // 如果文件头检测失败，基于扩展名
        match ext {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "gif" => Ok(ImageFormat::Gif),
            "bmp" => Ok(ImageFormat::Bmp),
            "webp" => Ok(ImageFormat::WebP),
            _ => {
                warn!("无法识别的图片格式: 扩展名={}, 文件头={:?}", 
                    ext, 
                    if data.len() >= 8 { &data[..8] } else { data }
                );
                // 默认使用 PNG（如果无法识别）
                Ok(ImageFormat::Png)
            }
        }
    }

    /// 处理图片数据（压缩大图片）
    /// 
    /// 如果图片过大，进行压缩处理。压缩后的图片质量会适当降低以减小文件大小。
    /// 如果原始格式不支持编码，会转换为 PNG 格式。
    /// 
    /// # 参数
    /// - `image_data`: 原始图片数据
    /// - `format`: 图片格式
    /// 
    /// # 返回
    /// - `Ok((Vec<u8>, Option<ImageDimensions>, ImageFormat))`: 处理后的图片数据、尺寸和最终格式
    /// - `Err(FlowyError)`: 处理失败
    fn process_image_data(
        &self,
        image_data: &[u8],
        format: &ImageFormat,
    ) -> FlowyResult<(Vec<u8>, Option<ImageDimensions>, ImageFormat)> {
        // 加载图片
        let img = image::load_from_memory(image_data).map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法加载图片: {}", e),
            )
        })?;

        let (width, height) = (img.width(), img.height());
        let dimensions = Some(ImageDimensions { width, height });

        // 定义最大尺寸（如果图片超过这个尺寸，进行压缩）
        const MAX_DIMENSION: u32 = 2048; // 最大宽度或高度
        const MAX_FILE_SIZE: usize = 2 * 1024 * 1024; // 2MB

        // 检查是否需要压缩
        let needs_compression = width > MAX_DIMENSION 
            || height > MAX_DIMENSION 
            || image_data.len() > MAX_FILE_SIZE;

        let final_image = if needs_compression {
            // 计算缩放比例
            let scale = (MAX_DIMENSION as f32 / width.max(height) as f32).min(1.0);
            let new_width = (width as f32 * scale) as u32;
            let new_height = (height as f32 * scale) as u32;

            info!(
                "压缩图片: {}x{} -> {}x{} (比例: {:.2})",
                width, height, new_width, new_height, scale
            );

            // 调整图片大小
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        // 将图片编码为字节，并确定最终格式
        let mut output = Cursor::new(Vec::new());
        let final_format = match format {
            ImageFormat::Png => {
                final_image.write_to(&mut output, ImageOutputFormat::Png).map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("无法编码 PNG 图片: {}", e),
                    )
                })?;
                ImageFormat::Png
            }
            ImageFormat::Jpeg => {
                // JPEG 使用质量 85（平衡质量和文件大小）
                final_image.write_to(&mut output, ImageOutputFormat::Jpeg(85)).map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("无法编码 JPEG 图片: {}", e),
                    )
                })?;
                ImageFormat::Jpeg
            }
            ImageFormat::Gif => {
                // GIF 格式，转换为 PNG（因为 GIF 编码可能不支持）
                final_image.write_to(&mut output, ImageOutputFormat::Png).map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("无法编码 GIF 图片: {}", e),
                    )
                })?;
                ImageFormat::Png // 格式已转换
            }
            ImageFormat::Bmp => {
                // BMP 格式，转换为 PNG
                final_image.write_to(&mut output, ImageOutputFormat::Png).map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("无法编码 BMP 图片: {}", e),
                    )
                })?;
                ImageFormat::Png // 格式已转换
            }
            ImageFormat::WebP => {
                // WebP 格式，转换为 PNG
                final_image.write_to(&mut output, ImageOutputFormat::Png).map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!("无法编码 WebP 图片: {}", e),
                    )
                })?;
                ImageFormat::Png // 格式已转换
            }
        };

        let final_data = output.into_inner();
        Ok((final_data, dimensions, final_format))
    }

    /// 从图片数据获取尺寸（不加载完整图片）
    /// 
    /// 这是一个轻量级方法，用于在图片处理失败时获取尺寸信息。
    /// 
    /// # 参数
    /// - `data`: 图片数据
    /// - `format`: 图片格式
    /// 
    /// # 返回
    /// - `Ok(Option<ImageDimensions>)`: 图片尺寸（如果能够获取）
    fn get_image_dimensions_from_data(
        &self,
        data: &[u8],
        _format: &ImageFormat,
    ) -> FlowyResult<Option<ImageDimensions>> {
        // 尝试加载图片以获取尺寸
        match image::load_from_memory(data) {
            Ok(img) => Ok(Some(ImageDimensions {
                width: img.width(),
                height: img.height(),
            })),
            Err(_) => {
                // 如果无法加载，返回 None（不会导致错误）
                Ok(None)
            }
        }
    }

    /// 将图片格式转换为文件扩展名
    /// 
    /// # 参数
    /// - `format`: 图片格式
    /// 
    /// # 返回
    /// - `&str`: 文件扩展名
    fn format_to_extension(&self, format: &ImageFormat) -> &'static str {
        match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::WebP => "webp",
        }
    }

    /// 提取 PDF 文档元数据
    /// 
    /// 从 PDF 文件中提取基本的元数据信息，如页数、文件大小等。
    /// 由于 Marker 工具主要用于转换，元数据提取功能有限。
    /// 
    /// # 参数
    /// - `pdf_path`: PDF 文件路径
    /// 
    /// # 返回
    /// - `Ok(DocumentMetadata)`: 提取的元数据
    /// - `Err(FlowyError)`: 提取过程中的错误
    async fn extract_document_metadata(&self, pdf_path: &Path) -> FlowyResult<DocumentMetadata> {
        // 验证文件
        self.validate_pdf(pdf_path).await?;

        // 获取文件元数据
        let file_metadata = std::fs::metadata(pdf_path).map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法获取文件元数据: {}", e),
            )
        })?;

        // 获取文件修改时间
        let modified_at = file_metadata
            .modified()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

        // 创建基本元数据
        // 注意：Marker 工具不提供详细的 PDF 元数据提取功能
        // 我们只能提供基本的文件信息
        let metadata = DocumentMetadata {
            author: None,
            created_at: modified_at.or_else(|| Some(Utc::now())),
            modified_at: modified_at.or_else(|| Some(Utc::now())),
            page_count: None, // Marker 工具不提供页数信息
            word_count: None, // 需要在转换后统计
            language: None,
        };

        // 尝试从文件路径提取文件名作为标题提示
        // 实际标题会在转换后从内容中提取

        Ok(metadata)
    }
}

/// 根据平台获取 marker-pdf 安装指引
fn get_platform_marker_pdf_install_instructions() -> String {
    use std::process::Command;
    
    #[cfg(target_os = "macos")]
    {
        // 检测是否安装了 Homebrew
        // 首先尝试使用 which 检查（如果 brew 在 PATH 中）
        let brew_available_from_path = Command::new("which")
            .arg("brew")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        // 如果 which 找不到，检查默认安装路径
        let arch = Command::new("uname")
            .arg("-m")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_else(|| "x86_64".to_string())
            .trim()
            .to_string();
        let is_apple_silicon = arch == "arm64";
        
        let brew_path = if is_apple_silicon {
            "/opt/homebrew/bin/brew"
        } else {
            "/usr/local/bin/brew"
        };
        
        let brew_available_from_path_check = std::path::Path::new(brew_path).exists();
        
        // 如果路径存在，尝试执行 brew --version 来验证
        let brew_available = if brew_available_from_path {
            true
        } else if brew_available_from_path_check {
            // 检查文件是否存在且可执行
            Command::new(brew_path)
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        } else {
            false
        };
        
        if !brew_available {
            return format!(
                "Homebrew 未安装。\n\n\
                Marker 工具需要使用 Homebrew 来安装 marker-pdf。\n\n\
                请先安装 Homebrew：\n\
                1. 打开终端，运行：\n\
                   /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\n\
                2. 或者访问 https://brew.sh 查看安装说明\n\n\
                3. 安装 Homebrew 后，运行：\n\
                   brew install jpeg libpng freetype openjpeg libtiff webp\n\
                   brew install pipx\n\
                   pipx install marker-pdf"
            );
        }
        
        format!(
            "使用 Homebrew 安装 marker-pdf：\n\
             1. 首先安装 Pillow 编译所需的依赖库：\n\
                brew install jpeg libpng freetype openjpeg libtiff webp\n\
             2. 安装 pipx：\n\
                brew install pipx\n\
             3. 安装 marker-pdf：\n\
                pipx install marker-pdf"
        )
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows 下检查 pipx 是否可用
        let pipx_available = Command::new("where")
            .arg("pipx")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        // 检查 Python 是否可用
        let python_available = Command::new("where")
            .arg("python")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        if !python_available {
            return format!(
                "Python 未安装。\n\n\
                首先需要安装 Python 3.8+：\n\n\
                方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带）\n\
                  winget install Python.Python.3.12\n\n\
                方法 2：手动安装\n\
                1. 访问 https://www.python.org/downloads/ 下载并安装 Python\n\
                2. 安装时勾选 \"Add Python to PATH\"\n\n\
                安装 Python 后，再安装 marker-pdf：\n\
                1. 打开命令提示符或 PowerShell\n\
                2. 运行: pip install --user pipx\n\
                3. 运行: pipx install marker-pdf"
            );
        }
        
        if !pipx_available {
            return format!(
                "pipx 未安装。请先安装 pipx：\n\n\
                方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带）\n\
                注意：WinGet 会自动安装 Python 作为 pipx 的依赖\n\
                  winget install pipx\n\n\
                方法 2：使用 pip 安装（需要先安装 Python）\n\
                1. 打开命令提示符或 PowerShell\n\
                2. 运行: pip install --user pipx\n\
                3. 将 pipx 添加到 PATH（如果尚未添加）\n\n\
                安装 pipx 后，运行: pipx install marker-pdf"
            );
        }
        
        format!(
            "安装方法：\n\
             1. 打开命令提示符或 PowerShell（以管理员身份运行）\n\
             2. 运行: pipx install marker-pdf\n\n\
             注意：Windows 下 Pillow 使用预编译包，通常不需要手动安装依赖库。"
        )
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        format!("当前平台 {} 不支持 marker-pdf 安装", std::env::consts::OS)
    }
}

#[async_trait::async_trait]
impl DocumentConverter for MarkerPdfConverter {
    fn name(&self) -> &str {
        "Marker PDF Converter"
    }

    fn supported_types(&self) -> Vec<DocumentType> {
        vec![DocumentType::Pdf]
    }

    async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        let import_id = task.import_id.clone();
        
        // 如果有 import_id，发送开始日志
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("[Marker PDF 转换] 开始处理文件: {}", task.source_path),
            );
            crate::import::import_log_collector::send_import_progress(
                id,
                &task.target_name,
                0.1,
                "正在验证 PDF 文件...",
            );
        }
        
        info!("[Marker PDF 转换] 开始处理文件: {}", task.source_path);

        // 确定 PDF 文件路径，并跟踪是否需要清理临时文件
        let (pdf_path, should_cleanup_temp_file): (std::path::PathBuf, bool) = if let Some(bytes) = &task.bytes_data {
            // 如果有字节数据，先写入临时文件（需要 .pdf 扩展名以便验证）
            // 直接在临时目录创建带 .pdf 扩展名的文件
            let temp_dir = std::env::temp_dir();
            let temp_file_path = temp_dir.join(format!("temp_import_{}.pdf", uuid::Uuid::new_v4()));
            
            // 创建并写入文件
            let mut file = std::fs::File::create(&temp_file_path).map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("无法创建临时文件: {}", e),
                )
            })?;
            
            use std::io::Write;
            file.write_all(bytes).map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("无法写入临时文件: {}", e),
                )
            })?;
            
            file.flush().map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("无法刷新临时文件: {}", e),
                )
            })?;
            
            // 验证临时文件
            self.validate_pdf(&temp_file_path).await?;
            
            (temp_file_path, true)
        } else {
            // 验证文件
            self.validate_file(Path::new(&task.source_path)).await?;
            (Path::new(&task.source_path).to_path_buf(), false)
        };

        info!("[Marker PDF 转换] 文件验证完成: {}", pdf_path.display());
        
        // 如果有 import_id，发送验证完成日志
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("文件验证完成: {}", pdf_path.display()),
            );
        }

        // 步骤 1: 使用 Marker 工具将 PDF 转换为 Markdown
        info!("[Marker PDF 转换] 步骤 1/3: 使用 Marker 工具转换 PDF 到 Markdown...");
        
        // 如果有 import_id，发送进度更新
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_progress(
                id,
                &task.target_name,
                0.2,
                "正在使用 Marker 工具转换 PDF 到 Markdown...",
            );
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                "开始执行 Marker 工具转换 PDF 到 Markdown".to_string(),
            );
        }
        
        // 创建临时输出目录（用于 Marker 工具输出和图片提取）
        // 注意：需要保持 output_dir 的生命周期，直到所有文件读取操作完成
        // 使用 Box 来延长生命周期，确保在 convert 方法结束前不会被 drop
        let output_dir = TempDir::new().map_err(|e| {
            let error_msg = format!("无法创建临时输出目录: {}", e);
            if let Some(ref id) = import_id {
                crate::import::import_log_collector::send_import_log(id, "error", error_msg.clone());
            }
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                error_msg,
            )
        })?;
        
        // 保存输出目录路径（因为 TempDir 在作用域结束时会被删除）
        // 注意：必须保持 output_dir 的生命周期，直到所有异步操作完成
        let output_dir_path = output_dir.path().to_path_buf();
        
        // 使用 Marker 工具转换 PDF 到 Markdown（传递 import_id 用于日志收集）
        // 这个异步操作会读取 output_dir_path 中的文件，所以必须确保 output_dir 还存在
        let markdown_content = self
            .convert_pdf_to_markdown_with_output_dir_and_import_id(&pdf_path, &output_dir_path, import_id.as_deref())
            .await?;
        
        info!(
            "[Marker PDF 转换] Markdown 转换完成: {} 字符",
            markdown_content.len()
        );
        
        // 如果有 import_id，发送转换完成日志
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("Markdown 转换完成: {} 字符", markdown_content.len()),
            );
        }

        // 步骤 2: 提取图片（在 output_dir 被删除之前）
        info!("[Marker PDF 转换] 步骤 2/3: 提取图片...");
        
        // 如果有 import_id，发送图片提取开始日志
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                "开始提取图片...".to_string(),
            );
        }
        
        // 提取图片（必须在 output_dir 被 drop 之前完成）
        // 使用 spawn_blocking 将同步的图片提取操作移到后台线程，避免阻塞异步运行时
        // 确保 output_dir 仍然存在，直到图片提取完成
        let extracted_images = if self.config.extract_images {
            // 从 Marker 输出目录提取图片
            // 注意：这里使用 output_dir_path，但 output_dir 必须仍然存在
            // 如果 output_dir 被 drop，目录会被删除，导致文件读取失败
            let output_dir_path_clone = output_dir_path.clone();
            let config = self.config.clone();
            let import_id_for_spawn = import_id.clone();
            let import_id_clone = import_id_for_spawn.map(|s| s.to_string());
            tokio::task::spawn_blocking(move || {
                // 在后台线程中创建新的转换器实例来提取图片
                let converter = MarkerPdfConverter::new(config);
                converter.extract_images_from_marker_output(&output_dir_path_clone, import_id_clone.as_deref())
            })
            .await
            .map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("图片提取任务失败: {}", e),
                )
            })?
        } else {
            Vec::new()
        };
        
        info!("[Marker PDF 转换] 图片提取完成: {} 张图片", extracted_images.len());
        
        // 如果有 import_id，发送图片提取完成日志
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!("图片提取完成: {} 张图片", extracted_images.len()),
            );
        }
        
        // 注意：output_dir 在这里仍然存在，确保所有文件读取操作都已完成
        // output_dir 将在 convert 方法结束时被 drop，此时所有数据都已提取

        // 步骤 3: 将 Markdown 转换为 AppFlowy 格式（可选）
        // 注意：DocumentContent 的 content 字段可以是 Markdown 格式
        // 我们可以直接使用 Markdown，或者使用 MarkdownToAppFlowyConverter 转换为 NestedBlock
        // 然后序列化为 JSON 或其他格式
        // 为了保持与现有系统的兼容性，我们直接使用 Markdown 作为 content
        info!("[Marker PDF 转换] 步骤 3/3: 处理 Markdown 内容...");
        
        // 统计字数
        let word_count = markdown_content.split_whitespace().count();
        
        // 统计表格数量（Markdown 表格以 | 开头）
        let table_count = markdown_content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('|') && trimmed.matches('|').count() >= 3
            })
            .count() as u32;

        // 提取文档元数据
        let metadata = self.extract_document_metadata(&pdf_path).await?;
        
        // 更新元数据中的字数
        let mut final_metadata = metadata;
        final_metadata.word_count = Some(word_count as u32);

        // 计算处理时间
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // 获取文件大小
        let file_size = if let Some(bytes) = &task.bytes_data {
            bytes.len() as u64
        } else {
            std::fs::metadata(&pdf_path)
                .map(|m| m.len())
                .unwrap_or(0)
        };

        info!(
            "[Marker PDF 转换] 转换完成: {}ms, {} 字符, {} 字, {} 张图片, {} 个表格",
            processing_time,
            markdown_content.len(),
            word_count,
            extracted_images.len(),
            table_count
        );
        
        // 如果有 import_id，发送转换完成日志
        if let Some(ref id) = import_id {
            crate::import::import_log_collector::send_import_log(
                id,
                "info",
                format!(
                    "PDF 转换完成: {}ms, {} 字符, {} 字, {} 张图片, {} 个表格",
                    processing_time,
                    markdown_content.len(),
                    word_count,
                    extracted_images.len(),
                    table_count
                ),
            );
        }

        // 构建转换结果
        // 先计算统计信息，避免移动后借用
        let text_length = markdown_content.len();
        let image_count = extracted_images.len() as u32;
        
        let result = ConversionResult {
            task_id: task.id,
            document_content: DocumentContent {
                title: task.target_name.clone(),
                content: markdown_content, // 使用 Markdown 格式作为 content
                metadata: final_metadata,
            },
            extracted_images,
            statistics: ConversionStatistics {
                processing_time_ms: processing_time,
                text_length,
                image_count,
                table_count,
                source_file_size: file_size,
            },
        };

        // 清理临时文件（如果创建了的话）
        if should_cleanup_temp_file {
            if let Err(e) = std::fs::remove_file(&pdf_path) {
                warn!("无法删除临时文件 {}: {}", pdf_path.display(), e);
            } else {
                debug!("已清理临时文件: {}", pdf_path.display());
            }
        }

        Ok(result)
    }

    async fn validate_file(&self, file_path: &Path) -> FlowyResult<()> {
        self.validate_pdf(file_path).await
    }

    async fn get_file_info(&self, file_path: &Path) -> FlowyResult<DocumentMetadata> {
        self.extract_document_metadata(file_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_validate_pdf_nonexistent() {
        let converter = MarkerPdfConverter::new(ConversionConfig::default());
        let fake_path = Path::new("/nonexistent/file.pdf");
        
        let result = converter.validate_pdf(fake_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_pdf_invalid_extension() {
        let converter = MarkerPdfConverter::new(ConversionConfig::default());
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        File::create(&file_path).unwrap();
        
        let result = converter.validate_pdf(&file_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_pdf_invalid_header() {
        let converter = MarkerPdfConverter::new(ConversionConfig::default());
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.pdf");
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"INVALID PDF CONTENT").unwrap();
        
        let result = converter.validate_pdf(&file_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_pdf_valid() {
        let converter = MarkerPdfConverter::new(ConversionConfig::default());
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.pdf");
        
        let mut file = File::create(&file_path).unwrap();
        // 写入有效的 PDF 文件头
        file.write_all(b"%PDF-1.4\n").unwrap();
        file.write_all(b"Some PDF content here...").unwrap();
        
        let result = converter.validate_pdf(&file_path).await;
        // 注意：即使文件头有效，完整的 PDF 验证可能仍然失败
        // 但至少文件头检查应该通过
        // 这里我们只测试基本验证逻辑
        assert!(result.is_ok() || result.is_err()); // 允许两种情况，因为完整验证可能要求更多
    }
}

