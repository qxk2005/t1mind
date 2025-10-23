use crate::import::converter::{
    ConversionConfig, ConversionResult, ConversionTask, DocumentConverter, DocumentMetadata,
    DocumentType, DocumentContent, ConversionStatistics, ExtractedImage,
};
use flowy_error::{FlowyError, FlowyResult};
use lopdf::{Document, Object};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use image::DynamicImage;
use uuid::Uuid;
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// PDF文档转换器
#[derive(Debug)]
pub struct PdfConverter {
    /// 转换配置
    config: ConversionConfig,
    /// 是否启用OCR
    enable_ocr: bool,
}

impl PdfConverter {
    /// 创建新的PDF转换器
    pub fn new(config: ConversionConfig) -> Self {
        Self { 
            config,
            enable_ocr: true, // 默认启用OCR
        }
    }

    /// 创建新的PDF转换器（带OCR选项）
    pub fn new_with_ocr(config: ConversionConfig, enable_ocr: bool) -> Self {
        Self { 
            config,
            enable_ocr,
        }
    }

    /// 提取PDF文本内容，保持布局
    fn extract_text_with_layout(&self, document: &mut Document, page_number: u32) -> FlowyResult<String> {
        let mut content = String::new();
        
        // 获取页面对象
        let pages = document.get_pages();
        if let Some(page_id) = pages.get(&page_number) {
            // 尝试多种文本提取方法
            content = self.try_multiple_extraction_methods(document, *page_id, page_number)?;
        } else {
            warn!("Page {} not found", page_number);
        }

        Ok(content)
    }

    /// 尝试多种文本提取方法
    fn try_multiple_extraction_methods(&self, document: &mut Document, page_id: lopdf::ObjectId, page_number: u32) -> FlowyResult<String> {
        let mut content = String::new();
        
        // 方法1: 使用 lopdf 的 extract_text
        if let Ok(page_content) = document.extract_text(&[page_id.0]) {
            if !page_content.trim().is_empty() {
                info!("Successfully extracted text from page {} using lopdf extract_text", page_number);
                return Ok(page_content);
            }
        }
        
        // 方法2: 手动解析页面内容
        if let Ok(manual_content) = self.extract_text_manually(document, page_id, page_number) {
            if !manual_content.trim().is_empty() {
                info!("Successfully extracted text from page {} using manual extraction", page_number);
                return Ok(manual_content);
            }
        }
        
        // 方法3: 尝试提取所有文本对象
        if let Ok(object_content) = self.extract_text_from_objects(document, page_id, page_number) {
            if !object_content.trim().is_empty() {
                info!("Successfully extracted text from page {} using object extraction", page_number);
                return Ok(object_content);
            }
        }
        
        // 方法4: 如果启用OCR，尝试OCR提取
        if self.enable_ocr {
            if let Ok(ocr_content) = self.extract_text_with_ocr_internal(document, page_id, page_number) {
                if !ocr_content.trim().is_empty() {
                    info!("Successfully extracted text from page {} using OCR", page_number);
                    return Ok(ocr_content);
                }
            }
        }
        
        warn!("Failed to extract text from page {} using all methods", page_number);
        Ok(content)
    }

    /// 手动解析页面内容
    fn extract_text_manually(&self, document: &Document, page_id: lopdf::ObjectId, _page_number: u32) -> FlowyResult<String> {
        let mut content = String::new();
        
        // 获取页面对象
        if let Ok(page_obj) = document.get_object(page_id) {
            if let lopdf::Object::Dictionary(page_dict) = page_obj {
                // 获取页面内容流
                if let Ok(contents_obj) = page_dict.get(b"Contents") {
                    match contents_obj {
                        lopdf::Object::Reference(ref_id) => {
                            if let Ok(stream_obj) = document.get_object(*ref_id) {
                                content = self.extract_text_from_stream(&stream_obj)?;
                            }
                        },
                        lopdf::Object::Array(refs) => {
                            // 多个内容流
                            for ref_obj in refs {
                                if let lopdf::Object::Reference(ref_id) = ref_obj {
                                    if let Ok(stream_obj) = document.get_object(*ref_id) {
                                        let stream_content = self.extract_text_from_stream(&stream_obj)?;
                                        content.push_str(&stream_content);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        
        Ok(content)
    }

    /// 从流对象中提取文本
    fn extract_text_from_stream(&self, stream_obj: &lopdf::Object) -> FlowyResult<String> {
        let mut content = String::new();
        
        if let lopdf::Object::Stream(stream) = stream_obj {
            // 解码流内容
            let decoded_content = &stream.content;
            
            // 使用更强大的PDF文本解析
            content = self.parse_pdf_content_stream(decoded_content)?;
        }
        
        Ok(content)
    }

    /// 解析PDF内容流中的文本
    fn parse_pdf_content_stream(&self, content: &[u8]) -> FlowyResult<String> {
        let mut extracted_text = String::new();
        let content_str = String::from_utf8_lossy(content);
        
        // 使用简单的字符串搜索方法提取文本
        extracted_text = self.extract_text_simple(&content_str);
        
        Ok(extracted_text)
    }

    /// 使用简单方法提取PDF文本
    fn extract_text_simple(&self, content: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            // 查找文本操作符 Tj, TJ, ', "
            if line.contains("Tj") || line.contains("TJ") || line.contains("'") || line.contains("\"") {
                // 提取括号内的文本
                if let Some(text) = self.extract_text_from_line(line) {
                    if !text.trim().is_empty() {
                        // 放宽验证条件，先看看实际提取的内容
                        if self.is_valid_text(&text) {
                            result.push_str(&text);
                            result.push(' ');
                        } else {
                            // 调试：记录被过滤的文本
                            info!("Filtered text: '{}'", text);
                        }
                    }
                }
            }
        }
        
        // 调试：记录最终提取的文本
        if result.trim().is_empty() {
            warn!("No text extracted from PDF content");
        } else {
            info!("Extracted text length: {}", result.len());
            // 调试：显示前100个字符
            let preview = if result.len() > 100 {
                format!("{}...", &result[..100])
            } else {
                result.clone()
            };
            info!("Extracted text preview: '{}'", preview);
        }
        
        result
    }

    /// 从单行中提取文本
    fn extract_text_from_line(&self, line: &str) -> Option<String> {
        // 查找括号内的文本
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                if start < end {
                    let text = &line[start + 1..end];
                    return Some(self.clean_text(text));
                }
            }
        }
        
        // 查找尖括号内的十六进制文本
        if let Some(start) = line.find('<') {
            if let Some(end) = line.rfind('>') {
                if start < end {
                    let hex_text = &line[start + 1..end];
                    if let Ok(decoded) = self.decode_hex_simple(hex_text) {
                        return Some(decoded);
                    }
                }
            }
        }
        
        None
    }

    /// 清理文本
    fn clean_text(&self, text: &str) -> String {
        let mut cleaned = text.to_string();
        
        // 移除PDF转义字符
        cleaned = cleaned.replace("\\(", "(");
        cleaned = cleaned.replace("\\)", ")");
        cleaned = cleaned.replace("\\\\", "\\");
        cleaned = cleaned.replace("\\n", "\n");
        cleaned = cleaned.replace("\\r", "\r");
        cleaned = cleaned.replace("\\t", "\t");
        
        // 移除控制字符，只保留可打印字符和基本空白字符
        cleaned = cleaned.chars()
            .filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n' || *c == '\r' || *c == '\t')
            .collect();
        
        cleaned
    }

    /// 简单十六进制解码
    fn decode_hex_simple(&self, hex_text: &str) -> FlowyResult<String> {
        let mut result = String::new();
        let chars: Vec<char> = hex_text.chars().collect();
        
        for i in (0..chars.len()).step_by(2) {
            if i + 1 < chars.len() {
                let hex_pair = format!("{}{}", chars[i], chars[i + 1]);
                if let Ok(byte_val) = u8::from_str_radix(&hex_pair, 16) {
                    // 只保留可打印的ASCII字符
                    if byte_val >= 32 && byte_val <= 126 {
                        result.push(byte_val as char);
                    }
                }
            }
        }
        
        Ok(result)
    }

    /// 验证文本是否有效
    fn is_valid_text(&self, text: &str) -> bool {
        // 过滤掉明显不是文本的内容
        if text.len() < 1 {
            return false;
        }
        
        // 进一步放宽特殊字符限制
        let special_chars = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
        let total_chars = text.len();
        
        // 如果特殊字符超过90%，可能不是有效文本（进一步放宽限制）
        if special_chars as f32 / total_chars as f32 > 0.9 {
            return false;
        }
        
        // 过滤掉包含代码模式的内容
        if text.contains("\\(") || text.contains("\\)") || text.contains("Tj") || text.contains("TJ") {
            return false;
        }
        
        // 过滤掉明显的错误信息
        if text.contains("Identity-H") || text.contains("Unimplemented") {
            return false;
        }
        
        // 过滤掉纯符号的文本
        if text.chars().all(|c| !c.is_alphanumeric()) {
            return false;
        }
        
        true
    }


    /// 从页面对象中提取文本
    fn extract_text_from_objects(&self, document: &Document, page_id: lopdf::ObjectId, _page_number: u32) -> FlowyResult<String> {
        let mut content = String::new();
        
        // 获取页面对象
        if let Ok(page_obj) = document.get_object(page_id) {
            if let lopdf::Object::Dictionary(page_dict) = page_obj {
                // 递归搜索文本对象
                content = self.search_text_in_dict(document, page_dict)?;
            }
        }
        
        Ok(content)
    }

    /// 在字典中搜索文本
    fn search_text_in_dict(&self, document: &Document, dict: &lopdf::Dictionary) -> FlowyResult<String> {
        let mut content = String::new();
        
        for (_key, value) in dict.iter() {
            match value {
                lopdf::Object::String(bytes, _) => {
                    // 尝试解码字符串
                    if let Ok(text) = String::from_utf8(bytes.clone()) {
                        if !text.trim().is_empty() && text.len() > 1 {
                            content.push_str(&text);
                            content.push(' ');
                        }
                    }
                },
                lopdf::Object::Reference(ref_id) => {
                    if let Ok(obj) = document.get_object(*ref_id) {
                        match obj {
                            lopdf::Object::Dictionary(sub_dict) => {
                                let sub_content = self.search_text_in_dict(document, sub_dict)?;
                                content.push_str(&sub_content);
                            },
                            lopdf::Object::String(bytes, _) => {
                                if let Ok(text) = String::from_utf8(bytes.clone()) {
                                    if !text.trim().is_empty() && text.len() > 1 {
                                        content.push_str(&text);
                                        content.push(' ');
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                },
                lopdf::Object::Array(arr) => {
                    for item in arr {
                        if let lopdf::Object::String(bytes, _) = item {
                            if let Ok(text) = String::from_utf8(bytes.clone()) {
                                if !text.trim().is_empty() && text.len() > 1 {
                                    content.push_str(&text);
                                    content.push(' ');
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        Ok(content)
    }

    /// 提取页面中的图像
    fn extract_images(&self, _document: &Document, _page_number: u32) -> FlowyResult<Vec<ExtractedImage>> {
        let images = Vec::new();
        
        // lopdf的图像提取比较复杂，这里先返回空列表
        // 在实际应用中，可能需要更复杂的逻辑来提取图像
        warn!("Image extraction not implemented for lopdf yet");
        
        Ok(images)
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
                html.push_str(&format!("<p>{}</p>", html_escape::encode_text(line)));
            }
        }
        
        html.push_str("</div>");
        html
    }

    /// 提取PDF文档元数据
    async fn extract_document_metadata(&self, document: &Document, file_path: &str) -> FlowyResult<DocumentMetadata> {
        let mut metadata = DocumentMetadata {
            author: None,
            created_at: None,
            modified_at: None,
            page_count: None,
            word_count: None,
            language: None,
        };

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
                        
                        // 提取创建时间
                        if let Ok(Object::String(_creation_bytes, _)) = info_dict.get(b"CreationDate") {
                            // PDF日期格式解析比较复杂，这里先跳过
                        }
                        
                        // 提取修改时间
                        if let Ok(Object::String(_modification_bytes, _)) = info_dict.get(b"ModDate") {
                            // PDF日期格式解析比较复杂，这里先跳过
                        }
                    }
                }
            }
        }

        // 获取页面数量
        let pages = document.get_pages();
        metadata.page_count = Some(pages.len() as u32);

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
}

#[async_trait::async_trait]
impl DocumentConverter for PdfConverter {
    fn name(&self) -> &str {
        "PDF Document Converter"
    }

    fn supported_types(&self) -> Vec<DocumentType> {
        vec![DocumentType::Pdf]
    }

    async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        
        info!("Starting PDF conversion for file: {}", task.source_path);

        // 读取PDF文件数据
        let pdf_data = if let Some(bytes) = &task.bytes_data {
            // 使用提供的字节数据
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

        // 加载PDF文档
        let mut document = Document::load_mem(&pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to load PDF document: {}", e),
            ))?;

        let pages = document.get_pages();
        let page_count = pages.len() as u32;
        info!("PDF document loaded with {} pages", page_count);

        // 提取文档元数据
        let metadata = self.extract_document_metadata(&document, &task.source_path).await?;

        // 提取所有页面的文本和图像
        let mut all_text = String::new();
        let mut all_images = Vec::new();
        let mut total_word_count = 0;

        for (page_number, _page_id) in pages.iter() {
            // 提取文本
            match self.extract_text_with_layout(&mut document, *page_number) {
                Ok(page_text) => {
                    if !page_text.trim().is_empty() {
                        all_text.push_str(&format!("Page {}:\n{}\n\n", page_number, page_text));
                        total_word_count += page_text.split_whitespace().count();
                    }
                }
                Err(e) => {
                    warn!("Failed to extract text from page {}: {}", page_number, e);
                }
            }

            // 提取图像
            match self.extract_images(&document, *page_number) {
                Ok(mut page_images) => {
                    all_images.append(&mut page_images);
                }
                Err(e) => {
                    warn!("Failed to extract images from page {}: {}", page_number, e);
                }
            }
        }

        // 转换为HTML格式
        let html_content = self.text_to_html(&all_text);

        let processing_time = start_time.elapsed().as_millis() as u64;
        let file_size = task.bytes_data.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0);
        let image_count = all_images.len() as u32;

        info!("PDF conversion completed in {}ms", processing_time);

        let result = ConversionResult {
            task_id: task.id,
            document_content: DocumentContent {
                title: task.target_name.clone(),
                content: html_content,
                metadata: DocumentMetadata {
                    author: metadata.author,
                    created_at: metadata.created_at,
                    modified_at: metadata.modified_at,
                    page_count: Some(page_count),
                    word_count: Some(total_word_count as u32),
                    language: metadata.language,
                },
            },
            extracted_images: all_images,
            statistics: ConversionStatistics {
                processing_time_ms: processing_time,
                text_length: all_text.len(),
                image_count,
                table_count: 0, // PDF中表格检测需要更复杂的逻辑
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

        // 加载PDF文档
        let document = Document::load_mem(&pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to load PDF document: {}", e),
            ))?;

        // 提取元数据
        self.extract_document_metadata(&document, &file_path.to_string_lossy()).await
    }
}

impl PdfConverter {
    /// 使用OCR提取文本（内部方法）
    fn extract_text_with_ocr_internal(&self, document: &mut Document, page_id: lopdf::ObjectId, page_number: u32) -> FlowyResult<String> {
        info!("Attempting OCR extraction for page {}", page_number);
        
        // 将PDF页面转换为图像
        let image = self.pdf_page_to_image_internal(document, page_id)?;
        
        // 使用OCR引擎提取文本
        let ocr_text = self.perform_ocr_on_image_internal(&image)?;
        
        Ok(ocr_text)
    }

    /// 将PDF页面转换为图像（内部方法）
    fn pdf_page_to_image_internal(&self, document: &mut Document, _page_id: lopdf::ObjectId) -> FlowyResult<DynamicImage> {
        info!("Converting PDF page to image using system tools");
        
        // 创建临时PDF文件
        let mut temp_pdf = NamedTempFile::new()?;
        let pdf_data = self.extract_pdf_data(document)?;
        temp_pdf.write_all(&pdf_data)?;
        temp_pdf.flush()?;
        
        // 使用系统工具将PDF转换为图片
        let image = self.convert_pdf_to_image_with_system_tools(temp_pdf.path())?;
        
        Ok(image)
    }

    /// 提取PDF数据
    fn extract_pdf_data(&self, document: &mut Document) -> FlowyResult<Vec<u8>> {
        // 将PDF文档序列化回字节数据
        // 这是为了将lopdf::Document转换回PDF字节流，供系统工具使用
        let mut pdf_data = Vec::new();
        
        // 使用lopdf的save_to方法将文档保存到字节向量
        document.save_to(&mut pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to serialize PDF document: {}", e),
            ))?;
        
        info!("Extracted PDF data: {} bytes", pdf_data.len());
        Ok(pdf_data)
    }

    /// 使用系统工具将PDF转换为图片
    fn convert_pdf_to_image_with_system_tools(&self, pdf_path: &Path) -> FlowyResult<DynamicImage> {
        // 尝试使用不同的系统工具
        if let Ok(image) = self.try_convert_with_pdftoppm(pdf_path) {
            return Ok(image);
        }
        
        if let Ok(image) = self.try_convert_with_gs(pdf_path) {
            return Ok(image);
        }
        
        // 如果都失败了，创建一个占位符图像
        warn!("Failed to convert PDF to image with system tools, using placeholder");
        self.create_placeholder_image()
    }

    /// 尝试使用pdftoppm转换
    fn try_convert_with_pdftoppm(&self, pdf_path: &Path) -> FlowyResult<DynamicImage> {
        let output_dir = tempfile::tempdir()?;
        let output_prefix = output_dir.path().join("page");
        
        let output = Command::new("pdftoppm")
            .arg("-png")
            .arg("-r")
            .arg("300") // 300 DPI
            .arg("-f")
            .arg("1") // 第一页
            .arg("-l")
            .arg("1") // 只转换第一页
            .arg(pdf_path)
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

    /// 尝试使用Ghostscript转换
    fn try_convert_with_gs(&self, pdf_path: &Path) -> FlowyResult<DynamicImage> {
        let output_path = tempfile::tempdir()?.path().join("page.png");
        
        let output = Command::new("gs")
            .arg("-dNOPAUSE")
            .arg("-dBATCH")
            .arg("-sDEVICE=png16m")
            .arg("-r300")
            .arg("-dFirstPage=1")
            .arg("-dLastPage=1")
            .arg(format!("-sOutputFile={}", output_path.display()))
            .arg(pdf_path)
            .output()?;
        
        if !output.status.success() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Ghostscript failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        if output_path.exists() {
            let image_data = std::fs::read(&output_path)?;
            let image = image::load_from_memory(&image_data)
                .map_err(|e| FlowyError::new(flowy_error::ErrorCode::Internal, format!("Failed to load image: {}", e)))?;
            return Ok(image);
        }
        
        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "Ghostscript did not generate expected output file"
        ))
    }

    /// 创建占位符图像
    fn create_placeholder_image(&self) -> FlowyResult<DynamicImage> {
        let width = 800;
        let height = 1000;
        
        // 创建一个白色背景的图像
        let img = image::ImageBuffer::from_fn(width, height, |_, _| {
            image::Rgb([255, 255, 255])
        });
        
        Ok(DynamicImage::ImageRgb8(img))
    }

    /// 在图像上执行OCR（内部方法）
    fn perform_ocr_on_image_internal(&self, image: &DynamicImage) -> FlowyResult<String> {
        info!("Performing OCR on image using system Tesseract");
        
        // 将图像保存到临时文件
        let mut temp_image = NamedTempFile::with_prefix("ocr_image_")?;
        image.write_to(&mut temp_image, image::ImageFormat::Png)
            .map_err(|e| FlowyError::new(flowy_error::ErrorCode::Internal, format!("Failed to write image: {}", e)))?;
        temp_image.flush()?;
        
        // 使用系统Tesseract进行OCR
        let ocr_text = self.run_tesseract_ocr(temp_image.path())?;
        
        info!("OCR extracted {} characters", ocr_text.len());
        Ok(ocr_text)
    }

    /// 运行Tesseract OCR
    fn run_tesseract_ocr(&self, image_path: &Path) -> FlowyResult<String> {
        let output = Command::new("tesseract")
            .arg(image_path)
            .arg("stdout")
            .arg("-l")
            .arg("chi_sim+eng") // 中英文混合识别
            .arg("--psm")
            .arg("6") // 统一文本块
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
        
        Ok(cleaned_text)
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

    /// 提取表格结构（内部方法）
    fn extract_tables_internal(&self, _document: &Document, _page_id: lopdf::ObjectId, page_number: u32) -> FlowyResult<Vec<String>> {
        info!("Extracting tables from page {}", page_number);
        
        // 这里需要实现表格检测算法
        // 暂时返回空结果
        Ok(Vec::new())
    }

    /// 提取图片（内部方法）
    fn extract_images_internal(&self, document: &Document, page_id: lopdf::ObjectId, page_number: u32) -> FlowyResult<Vec<ExtractedImage>> {
        info!("Extracting images from page {}", page_number);
        
        let mut images = Vec::new();
        
        // 获取页面对象
        if let Ok(page_obj) = document.get_object(page_id) {
            if let lopdf::Object::Dictionary(page_dict) = page_obj {
                // 获取页面资源
                if let Ok(resources_obj) = page_dict.get(b"Resources") {
                    if let lopdf::Object::Dictionary(resources) = resources_obj {
                        // 查找XObject（可能包含图片）
                        if let Ok(xobjects_obj) = resources.get(b"XObject") {
                            if let lopdf::Object::Dictionary(xobjects) = xobjects_obj {
                                for (name, obj_ref) in xobjects.iter() {
                                    if let lopdf::Object::Reference(obj_id) = obj_ref {
                                        if let Ok(obj) = document.get_object(*obj_id) {
                                            if let lopdf::Object::Stream(stream) = obj {
                                                // 检查是否是图片流
                                                if self.is_image_stream_internal(stream) {
                                                    if let Ok(image_data) = self.extract_image_from_stream_internal(stream) {
                                                        let extracted_image = ExtractedImage {
                                                            id: Uuid::new_v4().to_string(),
                                                            filename: format!("image_{}.png", name.len()),
                                                            data: image_data,
                                                            format: crate::import::converter::ImageFormat::Png,
                                                            dimensions: None,
                                                        };
                                                        images.push(extracted_image);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        info!("Extracted {} images from page {}", images.len(), page_number);
        Ok(images)
    }

    /// 检查是否是图片流（内部方法）
    fn is_image_stream_internal(&self, stream: &lopdf::Stream) -> bool {
        if let Ok(subtype) = stream.dict.get(b"Subtype") {
            if let lopdf::Object::Name(subtype) = subtype {
                return subtype == b"Image";
            }
        }
        false
    }

    /// 从流中提取图片数据（内部方法）
    fn extract_image_from_stream_internal(&self, stream: &lopdf::Stream) -> FlowyResult<Vec<u8>> {
        // 获取图片数据
        let image_data = &stream.content;
        
        // 这里需要根据图片格式进行解码
        // 暂时返回原始数据
        Ok(image_data.clone())
    }
}
