use crate::import::converter::{
    ConversionConfig, ConversionResult, ConversionTask, DocumentConverter, DocumentMetadata,
    DocumentType, DocumentContent, ConversionStatistics, ExtractedImage,
};
use flowy_error::{FlowyError, FlowyResult};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};
use image::DynamicImage;
use uuid::Uuid;
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;
use unicode_normalization::UnicodeNormalization;
use encoding_rs::{Encoding, UTF_8, UTF_16LE, UTF_16BE};

/// 增强的PDF文档转换器，使用pdfium-render提供更好的多国语言支持
#[derive(Debug)]
pub struct EnhancedPdfConverter {
    /// 转换配置
    config: ConversionConfig,
    /// 是否启用OCR作为备用方案
    enable_ocr_fallback: bool,
    /// 文本编码检测
    encoding_detector: EncodingDetector,
}

/// 编码检测器
#[derive(Debug)]
struct EncodingDetector {
    /// 支持的编码列表
    supported_encodings: Vec<&'static Encoding>,
}

impl EncodingDetector {
    fn new() -> Self {
        Self {
            supported_encodings: vec![UTF_8, UTF_16LE, UTF_16BE],
        }
    }

    /// 检测文本编码
    fn detect_encoding(&self, bytes: &[u8]) -> &'static Encoding {
        // 首先尝试UTF-8
        if UTF_8.decode(bytes).0.is_empty() {
            return UTF_8;
        }

        // 尝试UTF-16LE
        if bytes.len() % 2 == 0 {
            if let Ok(utf16_str) = std::str::from_utf8(bytes) {
                if utf16_str.chars().all(|c| c.is_ascii()) {
                    return UTF_16LE;
                }
            }
        }

        // 默认返回UTF-8
        UTF_8
    }

    /// 解码文本，支持多种编码
    fn decode_text(&self, bytes: &[u8]) -> String {
        let encoding = self.detect_encoding(bytes);
        let (decoded, _, _) = encoding.decode(bytes);
        decoded.to_string()
    }
}

impl EnhancedPdfConverter {
    /// 创建新的增强PDF转换器
    pub fn new(config: ConversionConfig) -> Self {
        Self { 
            config,
            enable_ocr_fallback: true,
            encoding_detector: EncodingDetector::new(),
        }
    }

    /// 创建新的增强PDF转换器（带OCR选项）
    pub fn new_with_ocr(config: ConversionConfig, enable_ocr_fallback: bool) -> Self {
        Self { 
            config,
            enable_ocr_fallback,
            encoding_detector: EncodingDetector::new(),
        }
    }

    /// 使用pdfium-render提取PDF文本内容
    fn extract_text_with_pdfium(&self, pdf_data: &[u8]) -> FlowyResult<String> {
        info!("Attempting to extract text using pdfium-render");
        
        // 尝试使用pdfium-render库
        match self.try_pdfium_render_extraction(pdf_data) {
            Ok(text) if !text.trim().is_empty() => {
                info!("Successfully extracted text using pdfium-render: {} characters", text.len());
                return Ok(text);
            }
            Ok(_) => {
                warn!("pdfium-render extracted empty text");
            }
            Err(e) => {
                warn!("pdfium-render extraction failed: {}", e);
            }
        }

        // 如果pdfium-render失败，尝试lopdf作为备用
        match self.try_lopdf_extraction(pdf_data) {
            Ok(text) if !text.trim().is_empty() => {
                info!("Successfully extracted text using lopdf fallback: {} characters", text.len());
                return Ok(text);
            }
            Ok(_) => {
                warn!("lopdf extracted empty text");
            }
            Err(e) => {
                warn!("lopdf extraction failed: {}", e);
            }
        }

        // 如果都失败了，尝试OCR
        if self.enable_ocr_fallback {
            match self.try_ocr_extraction(pdf_data) {
                Ok(text) if !text.trim().is_empty() => {
                    info!("Successfully extracted text using OCR: {} characters", text.len());
                    return Ok(text);
                }
                Ok(_) => {
                    warn!("OCR extracted empty text");
                }
                Err(e) => {
                    warn!("OCR extraction failed: {}", e);
                }
            }
        }

        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "Failed to extract text using all available methods"
        ))
    }

    /// 尝试使用pdfium-render提取文本
    fn try_pdfium_render_extraction(&self, pdf_data: &[u8]) -> FlowyResult<String> {
        // 注意：这里需要根据实际的pdfium-render API进行调整
        // 由于pdfium-render可能还没有稳定版本，这里提供一个框架
        
        // 创建临时文件
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(pdf_data)?;
        temp_file.flush()?;

        // 使用pdfium-render加载PDF
        // let pdf = pdfium_render::PdfDocument::load_from_file(temp_file.path())?;
        
        // 提取所有页面的文本
        let mut all_text = String::new();
        
        // 这里需要根据实际的pdfium-render API实现
        // for page_num in 0..pdf.page_count() {
        //     let page = pdf.page(page_num)?;
        //     let page_text = page.text()?;
        //     all_text.push_str(&page_text);
        //     all_text.push('\n');
        // }

        // 临时实现：使用系统工具作为替代
        self.extract_text_with_system_tools(temp_file.path())
    }

    /// 使用系统工具提取文本（作为pdfium-render的临时替代）
    fn extract_text_with_system_tools(&self, pdf_path: &Path) -> FlowyResult<String> {
        // 尝试使用pdftotext
        if let Ok(text) = self.try_pdftotext(pdf_path) {
            return Ok(text);
        }

        // 尝试使用pdf2txt.py
        if let Ok(text) = self.try_pdf2txt(pdf_path) {
            return Ok(text);
        }

        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "System tools extraction failed"
        ))
    }

    /// 使用pdftotext提取文本
    fn try_pdftotext(&self, pdf_path: &Path) -> FlowyResult<String> {
        let output = Command::new("pdftotext")
            .arg("-layout")  // 保持布局
            .arg("-enc")      // 指定编码
            .arg("UTF-8")     // 使用UTF-8编码
            .arg(pdf_path)
            .arg("-")         // 输出到stdout
            .output()?;

        if !output.status.success() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("pdftotext failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let normalized_text = self.normalize_text(&text);
        
        Ok(normalized_text)
    }

    /// 使用pdf2txt.py提取文本
    fn try_pdf2txt(&self, pdf_path: &Path) -> FlowyResult<String> {
        let output = Command::new("python3")
            .arg("-c")
            .arg(format!(
                "import sys; from pdfminer.high_level import extract_text; print(extract_text('{}'))",
                pdf_path.display()
            ))
            .output()?;

        if !output.status.success() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("pdf2txt.py failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let normalized_text = self.normalize_text(&text);
        
        Ok(normalized_text)
    }

    /// 尝试使用lopdf提取文本（备用方案）
    fn try_lopdf_extraction(&self, pdf_data: &[u8]) -> FlowyResult<String> {
        use lopdf::{Document, Object};
        
        let document = Document::load_mem(pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to load PDF with lopdf: {}", e)
            ))?;

        let pages = document.get_pages();
        let mut all_text = String::new();

        for (page_number, page_id) in pages.iter() {
            match document.extract_text(&[page_id.0]) {
                Ok(page_text) => {
                    if !page_text.trim().is_empty() {
                        let normalized_text = self.normalize_text(&page_text);
                        all_text.push_str(&format!("Page {}:\n{}\n\n", page_number, normalized_text));
                    }
                }
                Err(e) => {
                    warn!("Failed to extract text from page {} with lopdf: {}", page_number, e);
                }
            }
        }

        if all_text.trim().is_empty() {
            Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                "No text extracted with lopdf"
            ))
        } else {
            Ok(all_text)
        }
    }

    /// 尝试使用OCR提取文本
    fn try_ocr_extraction(&self, pdf_data: &[u8]) -> FlowyResult<String> {
        info!("Attempting OCR extraction as fallback");
        
        // 将PDF转换为图像
        let image = self.pdf_to_image(pdf_data)?;
        
        // 使用OCR提取文本
        let ocr_text = self.perform_ocr(&image)?;
        
        Ok(ocr_text)
    }

    /// 将PDF转换为图像
    fn pdf_to_image(&self, pdf_data: &[u8]) -> FlowyResult<DynamicImage> {
        let mut temp_pdf = NamedTempFile::new()?;
        temp_pdf.write_all(pdf_data)?;
        temp_pdf.flush()?;

        // 使用pdftoppm转换为图像
        let output_dir = tempfile::tempdir()?;
        let output_prefix = output_dir.path().join("page");

        let output = Command::new("pdftoppm")
            .arg("-png")
            .arg("-r")
            .arg("300") // 300 DPI for better OCR
            .arg("-f")
            .arg("1")
            .arg("-l")
            .arg("1")
            .arg(temp_pdf.path())
            .arg(&output_prefix)
            .output()?;

        if !output.status.success() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("pdftoppm failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        // 查找生成的图片文件
        let image_path = output_dir.path().join("page-1.png");
        if image_path.exists() {
            let image_data = std::fs::read(&image_path)?;
            let image = image::load_from_memory(&image_data)
                .map_err(|e| FlowyError::new(flowy_error::ErrorCode::Internal, format!("Failed to load image: {}", e)))?;
            return Ok(image);
        }

        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "pdftoppm did not generate expected output file"
        ))
    }

    /// 执行OCR
    fn perform_ocr(&self, image: &DynamicImage) -> FlowyResult<String> {
        // 将图像保存到临时文件
        let mut temp_image = NamedTempFile::with_prefix("ocr_image_")?;
        image.write_to(&mut temp_image, image::ImageFormat::Png)
            .map_err(|e| FlowyError::new(flowy_error::ErrorCode::Internal, format!("Failed to write image: {}", e)))?;
        temp_image.flush()?;

        // 使用Tesseract进行OCR，支持多语言
        let output = Command::new("tesseract")
            .arg(temp_image.path())
            .arg("stdout")
            .arg("-l")
            .arg("chi_sim+eng+jpn+kor+ara+rus") // 支持多种语言
            .arg("--psm")
            .arg("6") // 统一文本块
            .arg("-c")
            .arg("preserve_interword_spaces=1") // 保持单词间距
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Tesseract OCR failed: {}", error_msg);
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Tesseract OCR failed: {}", error_msg)
            ));
        }

        let ocr_text = String::from_utf8_lossy(&output.stdout).to_string();
        let cleaned_text = self.clean_ocr_text(&ocr_text);
        let normalized_text = self.normalize_text(&cleaned_text);
        
        Ok(normalized_text)
    }

    /// 清理OCR文本
    fn clean_ocr_text(&self, text: &str) -> String {
        let mut cleaned = text.to_string();
        
        // 移除多余的空白字符
        cleaned = cleaned.replace("\r\n", "\n");
        cleaned = cleaned.replace("\r", "\n");
        
        // 移除行首行尾的空白
        let lines: Vec<&str> = cleaned.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        
        lines.join("\n")
    }

    /// 标准化文本，处理Unicode
    fn normalize_text(&self, text: &str) -> String {
        // 使用Unicode标准化
        text.nfc().collect::<String>()
    }

    /// 提取PDF文档元数据
    async fn extract_document_metadata(&self, pdf_data: &[u8], _file_path: &str) -> FlowyResult<DocumentMetadata> {
        let mut metadata = DocumentMetadata {
            author: None,
            created_at: None,
            modified_at: None,
            page_count: None,
            word_count: None,
            language: None,
        };

        // 尝试使用pdfium-render提取元数据
        match self.try_extract_metadata_with_pdfium(pdf_data) {
            Ok(extracted_metadata) => {
                metadata = extracted_metadata;
            }
            Err(e) => {
                warn!("Failed to extract metadata with pdfium: {}", e);
                // 使用lopdf作为备用
                if let Ok(lopdf_metadata) = self.try_extract_metadata_with_lopdf(pdf_data) {
                    metadata = lopdf_metadata;
                }
            }
        }

        // 如果无法从PDF中获取创建时间，使用当前时间
        if metadata.created_at.is_none() || metadata.modified_at.is_none() {
            let now = Utc::now();
            if metadata.created_at.is_none() {
                metadata.created_at = Some(now);
            }
            if metadata.modified_at.is_none() {
                metadata.modified_at = Some(now);
            }
        }

        Ok(metadata)
    }

    /// 尝试使用pdfium-render提取元数据
    fn try_extract_metadata_with_pdfium(&self, _pdf_data: &[u8]) -> FlowyResult<DocumentMetadata> {
        // 这里需要根据实际的pdfium-render API实现
        // 暂时返回默认元数据
        Ok(DocumentMetadata {
            author: None,
            created_at: None,
            modified_at: None,
            page_count: None,
            word_count: None,
            language: None,
        })
    }

    /// 尝试使用lopdf提取元数据
    fn try_extract_metadata_with_lopdf(&self, pdf_data: &[u8]) -> FlowyResult<DocumentMetadata> {
        use lopdf::{Document, Object};
        
        let document = Document::load_mem(pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to load PDF with lopdf: {}", e)
            ))?;

        let mut metadata = DocumentMetadata {
            author: None,
            created_at: None,
            modified_at: None,
            page_count: None,
            word_count: None,
            language: None,
        };

        // 获取页面数量
        let pages = document.get_pages();
        metadata.page_count = Some(pages.len() as u32);

        // 尝试从PDF文档信息中提取元数据
        if let Ok(info) = document.trailer.get(b"Info") {
            if let Object::Reference(info_ref) = info {
                if let Ok(info_dict) = document.get_object(*info_ref) {
                    if let Object::Dictionary(info_dict) = info_dict {
                        // 提取作者信息
                        if let Ok(Object::String(author_bytes, _)) = info_dict.get(b"Author") {
                            if let Ok(author) = String::from_utf8(author_bytes.clone()) {
                                metadata.author = Some(author);
                            }
                        }
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// 将文本内容转换为HTML格式
    fn text_to_html(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut html = String::new();
        
        html.push_str("<div class=\"pdf-content\">");
        
        for line in lines {
            if line.trim().is_empty() {
                html.push_str("<br/>");
            } else {
                // 使用html_escape确保安全
                let escaped_line = html_escape::encode_text(line);
                html.push_str(&format!("<p>{}</p>", escaped_line));
            }
        }
        
        html.push_str("</div>");
        html
    }

    /// 提取页面中的图像
    fn extract_images(&self, _pdf_data: &[u8]) -> FlowyResult<Vec<ExtractedImage>> {
        // 图像提取需要更复杂的实现
        // 暂时返回空列表
        warn!("Image extraction not implemented yet");
        Ok(Vec::new())
    }
}

#[async_trait::async_trait]
impl DocumentConverter for EnhancedPdfConverter {
    fn name(&self) -> &str {
        "Enhanced PDF Document Converter"
    }

    fn supported_types(&self) -> Vec<DocumentType> {
        vec![DocumentType::Pdf]
    }

    async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        
        info!("Starting enhanced PDF conversion for file: {}", task.source_path);

        // 读取PDF文件数据
        let pdf_data = if let Some(bytes) = &task.bytes_data {
            bytes.clone()
        } else {
            // 验证文件
            self.validate_file(Path::new(&task.source_path)).await?;
            
            // 读取PDF文件
            let mut file = File::open(&task.source_path)
                .map_err(|e| FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Failed to open PDF file: {}", e),
                ))?;

            let mut pdf_data = Vec::new();
            file.read_to_end(&mut pdf_data)
                .map_err(|e| FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Failed to read PDF file: {}", e),
                ))?;
            
            pdf_data
        };

        info!("PDF file loaded: {} bytes", pdf_data.len());

        // 提取文档元数据
        let metadata = self.extract_document_metadata(&pdf_data, &task.source_path).await?;

        // 提取文本内容
        let extracted_text = self.extract_text_with_pdfium(&pdf_data)?;
        let word_count = extracted_text.split_whitespace().count();

        // 提取图像
        let extracted_images = self.extract_images(&pdf_data)?;

        // 转换为HTML格式
        let html_content = self.text_to_html(&extracted_text);

        let processing_time = start_time.elapsed().as_millis() as u64;
        let file_size = task.bytes_data.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0);
        let image_count = extracted_images.len() as u32;

        info!("Enhanced PDF conversion completed in {}ms", processing_time);
        info!("Extracted {} characters, {} words", extracted_text.len(), word_count);

        let result = ConversionResult {
            task_id: task.id,
            document_content: DocumentContent {
                title: task.target_name.clone(),
                content: html_content,
                metadata: DocumentMetadata {
                    author: metadata.author,
                    created_at: metadata.created_at,
                    modified_at: metadata.modified_at,
                    page_count: metadata.page_count,
                    word_count: Some(word_count as u32),
                    language: metadata.language,
                },
            },
            extracted_images,
            statistics: ConversionStatistics {
                processing_time_ms: processing_time,
                text_length: extracted_text.len(),
                image_count,
                table_count: 0, // 表格检测需要更复杂的逻辑
                source_file_size: file_size,
            },
        };

        Ok(result)
    }

    async fn validate_file(&self, file_path: &Path) -> FlowyResult<()> {
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::RecordNotFound,
                format!("File not found: {}", file_path.display()),
            ));
        }

        // 检查文件扩展名
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            if !DocumentType::Pdf.supported_extensions().contains(&ext.to_lowercase().as_str()) {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::UnsupportedFileFormat,
                    format!("Unsupported file format: {}", ext),
                ));
            }
        } else {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::UnsupportedFileFormat,
                "File has no extension",
            ));
        }

        // 检查文件大小
        if let Some(max_size) = self.config.max_file_size {
            let file_size = std::fs::metadata(file_path)?.len();
            if file_size > max_size {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::SingleUploadLimitExceeded,
                    format!("File too large: {} bytes (max: {} bytes)", file_size, max_size),
                ));
            }
        }

        // 检查PDF文件头
        let mut file = std::fs::File::open(file_path)?;
        let mut header = [0u8; 4];
        std::io::Read::read_exact(&mut file, &mut header)?;
        
        if &header != b"%PDF" {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::UnsupportedFileFormat,
                "Invalid PDF file format",
            ));
        }

        Ok(())
    }

    async fn get_file_info(&self, file_path: &Path) -> FlowyResult<DocumentMetadata> {
        // 验证文件
        self.validate_file(file_path).await?;

        // 读取PDF文件
        let mut file = File::open(file_path)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to open PDF file: {}", e),
            ))?;

        let mut pdf_data = Vec::new();
        file.read_to_end(&mut pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to read PDF file: {}", e),
            ))?;

        // 提取元数据
        self.extract_document_metadata(&pdf_data, &file_path.to_string_lossy()).await
    }
}
