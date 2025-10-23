use crate::import::converter::{
    ConversionConfig, ConversionResult, ConversionTask, DocumentConverter, DocumentMetadata,
    DocumentType, ExtractedImage, ImageFormat, ImageDimensions, ConversionStatistics,
    DocumentContent, ConversionError,
};
use flowy_error::{FlowyError, FlowyResult};
use std::path::Path;
use std::collections::HashMap;
use docx_rs::*;
use image::{ImageDecoder};

/// Word文档转换器
#[derive(Debug)]
pub struct WordConverter {
    /// 转换配置
    config: ConversionConfig,
}

impl WordConverter {
    /// 创建新的Word转换器
    pub fn new(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// 安全地预览字符串，避免UTF-8边界问题
    fn safe_string_preview(&self, text: &str, max_chars: usize) -> String {
        let preview: String = text.chars().take(max_chars).collect();
        if text.chars().count() > max_chars {
            format!("{}...", preview)
        } else {
            preview
        }
    }

    /// 从字节数据解析Word文档内容
    fn parse_word_document_from_bytes(&self, file_data: &[u8]) -> FlowyResult<(String, Vec<ExtractedImage>, DocumentMetadata)> {
        tracing::info!("📄 Starting to parse Word document from bytes ({} bytes)", file_data.len());
        
        // 使用docx-rs读取文档
        let docx = docx_rs::read_docx(file_data)
            .map_err(|e| ConversionError::ConversionFailed(format!("Failed to parse docx: {}", e)))?;
        
        tracing::info!("📄 Successfully parsed docx, found {} document children", docx.document.children.len());
        tracing::info!("📸 Found {} media files in document", docx.media.len());
        
        // 调试：检查文档关系
        tracing::info!("🔍 Document relationships:");
        tracing::info!("  - Images: {}", docx.document_rels.images.len());
        tracing::info!("  - Hyperlinks: {}", docx.document_rels.hyperlinks.len());
        tracing::info!("  - Has comments: {}", docx.document_rels.has_comments);
        tracing::info!("  - Has numberings: {}", docx.document_rels.has_numberings);
        tracing::info!("  - Has footnotes: {}", docx.document_rels.has_footnotes);
        
        // 调试：检查图片关系
        tracing::info!("🖼️ Found {} image relationships", docx.document_rels.images.len());
        for (id, target) in &docx.document_rels.images {
            tracing::info!("  - Image ID: {}, Target: {}", id, target);
        }
        
        // 调试：检查是否有Drawing元素包含图片
        tracing::info!("🔍 Checking for drawing elements with images...");
        let mut drawing_with_images = 0;
        for child in &docx.document.children {
            if let DocumentChild::Paragraph(para) = child {
                for run_child in &para.children {
                    if let ParagraphChild::Run(run) = run_child {
                        for run_child in &run.children {
                            if let RunChild::Drawing(drawing) = run_child {
                                if let Some(DrawingData::Pic(pic)) = &drawing.data {
                                    drawing_with_images += 1;
                                    tracing::info!("  - Found drawing with pic id: {}", pic.id);
                                }
                            }
                        }
                    }
                }
            }
        }
        tracing::info!("🎨 Found {} drawings with images", drawing_with_images);
        
        // 提取文本内容
        let mut html_content = String::new();
        let mut extracted_images = Vec::new();
        let mut table_count = 0u32;
        let mut drawing_count = 0u32;

        // 处理文档子元素
        for (i, child) in docx.document.children.iter().enumerate() {
            match child {
                DocumentChild::Paragraph(p) => {
                    tracing::debug!("Processing paragraph {}: {} runs", i, p.children.len());
                    
                    // 检查段落样式来确定是否为标题
                    let is_heading = self.is_heading_paragraph(p);
                    if is_heading.is_some() {
                        let heading_level = is_heading.unwrap();
                        html_content.push_str(&format!("<h{}>", heading_level));
                        self.process_paragraph(p, &mut html_content, &mut extracted_images)?;
                        html_content.push_str(&format!("</h{}>\n", heading_level));
                    } else {
                        html_content.push_str("<p>");
                        self.process_paragraph(p, &mut html_content, &mut extracted_images)?;
                        html_content.push_str("</p>\n");
                    }
                }
                DocumentChild::Table(t) => {
                    table_count += 1;
                    tracing::debug!("Processing table {}: {} rows", i, t.rows.len());
                    html_content.push_str("<table border='1'>");
                    self.process_table(t, &mut html_content, &mut extracted_images)?;
                    html_content.push_str("</table>\n");
                }
                _ => {
                    tracing::debug!("Skipping document child {}: {:?}", i, std::mem::discriminant(child));
                }
            }
        }

        // 处理媒体文件（图片） - 使用替代方案直接解析ZIP
        tracing::info!("🔧 Using alternative image extraction method...");
        let alternative_images = self.extract_images_from_zip(file_data)?;
        tracing::info!("📸 Alternative method found {} images", alternative_images.len());
        
        // 将替代方案提取的图片添加到结果中
        for image in alternative_images {
            tracing::info!("✅ Alternative image: {} ({} bytes)", image.id, image.data.len());
            extracted_images.push(image);
        }

        tracing::info!("📊 Document processing summary:");
        tracing::info!("  - Paragraphs processed: {}", docx.document.children.iter().filter(|c| matches!(c, DocumentChild::Paragraph(_))).count());
        tracing::info!("  - Tables processed: {}", table_count);
        tracing::info!("  - Drawings processed: {}", drawing_count);
        tracing::info!("  - Media files found: {}", docx.media.len());
        tracing::info!("  - Images extracted: {}", extracted_images.len());
        tracing::info!("  - HTML content length: {}", html_content.len());

        // 提取元数据
        let metadata = self.extract_metadata(&docx);

        Ok((html_content, extracted_images, metadata))
    }

    /// 解析Word文档内容
    fn parse_word_document(&self, file_path: &Path) -> FlowyResult<(String, Vec<ExtractedImage>, DocumentMetadata)> {
        // 读取文件内容
        let file_data = std::fs::read(file_path)?;
        
        // 使用docx-rs读取文档
        let docx = docx_rs::read_docx(&file_data)
            .map_err(|e| ConversionError::ConversionFailed(format!("Failed to parse docx: {}", e)))?;
        
        // 提取文本内容
        let mut html_content = String::new();
        let mut extracted_images = Vec::new();
        let mut table_count = 0u32;

        // 处理文档子元素
        for child in &docx.document.children {
            match child {
                DocumentChild::Paragraph(p) => {
                    // 检查段落样式来确定是否为标题
                    let is_heading = self.is_heading_paragraph(p);
                    if is_heading.is_some() {
                        let heading_level = is_heading.unwrap();
                        html_content.push_str(&format!("<h{}>", heading_level));
                        self.process_paragraph(p, &mut html_content, &mut extracted_images)?;
                        html_content.push_str(&format!("</h{}>\n", heading_level));
                    } else {
                        html_content.push_str("<p>");
                        self.process_paragraph(p, &mut html_content, &mut extracted_images)?;
                        html_content.push_str("</p>\n");
                    }
                }
                DocumentChild::Table(t) => {
                    table_count += 1;
                    html_content.push_str("<table border='1'>");
                    self.process_table(t, &mut html_content, &mut extracted_images)?;
                    html_content.push_str("</table>\n");
                }
                _ => {}
            }
        }

        // 提取元数据
        let metadata = self.extract_metadata(&docx);

        Ok((html_content, extracted_images, metadata))
    }

    /// 检查段落是否为标题 - 基于Word文档的实际样式信息
    fn is_heading_paragraph(&self, paragraph: &Paragraph) -> Option<u32> {
        // 1. 首先检查段落样式ID - 这是最可靠的方法
        if let Some(style_id) = &paragraph.property.style {
            let style_name = &style_id.val;
            let style_name_lower = style_name.to_lowercase();
            
            tracing::debug!("Checking paragraph style: '{}'", style_name);
            
            // 检查Word内置的标题样式
            if style_name_lower == "heading1" || style_name_lower == "heading 1" || 
               style_name_lower == "title" || style_name_lower == "titre 1" {
                tracing::debug!("Found H1 style: '{}'", style_name);
                return Some(1);
            }
            if style_name_lower == "heading2" || style_name_lower == "heading 2" || 
               style_name_lower == "titre 2" {
                tracing::debug!("Found H2 style: '{}'", style_name);
                return Some(2);
            }
            if style_name_lower == "heading3" || style_name_lower == "heading 3" || 
               style_name_lower == "titre 3" {
                tracing::debug!("Found H3 style: '{}'", style_name);
                return Some(3);
            }
            if style_name_lower == "heading4" || style_name_lower == "heading 4" || 
               style_name_lower == "titre 4" {
                tracing::debug!("Found H4 style: '{}'", style_name);
                return Some(4);
            }
            if style_name_lower == "heading5" || style_name_lower == "heading 5" || 
               style_name_lower == "titre 5" {
                tracing::debug!("Found H5 style: '{}'", style_name);
                return Some(5);
            }
            if style_name_lower == "heading6" || style_name_lower == "heading 6" || 
               style_name_lower == "titre 6" {
                tracing::debug!("Found H6 style: '{}'", style_name);
                return Some(6);
            }
            
            // 检查包含"heading"或"title"的样式名称
            if style_name_lower.contains("heading") || style_name_lower.contains("title") || 
               style_name_lower.contains("titre") {
                // 尝试从样式名称中提取数字
                for i in 1..=6 {
                    if style_name_lower.contains(&format!("heading{}", i)) || 
                       style_name_lower.contains(&format!("heading {}", i)) ||
                       style_name_lower.contains(&format!("titre{}", i)) ||
                       style_name_lower.contains(&format!("titre {}", i)) {
                        tracing::debug!("Found heading style with number {}: '{}'", i, style_name);
                        return Some(i);
                    }
                }
                // 如果没有找到具体级别，默认为H1
                tracing::debug!("Found generic heading style: '{}', defaulting to H1", style_name);
                return Some(1);
            }
        }
        
        // 2. 检查段落属性中的大纲级别 - 这是Word的层级结构
        if let Some(outline_lvl) = &paragraph.property.outline_lvl {
            let level = outline_lvl.v;
            // 大纲级别通常从0开始，转换为1-6的标题级别
            let heading_level = (level + 1).min(6).max(1) as u32;
            tracing::debug!("Found outline level {} -> H{}", level, heading_level);
            return Some(heading_level);
        }
        
        // 3. 检查段落属性中的其他标题相关属性
        let para_props = &paragraph.property;
        if para_props.outline_lvl.is_some() {
            // 如果有大纲级别，说明这是一个标题
            let level = para_props.outline_lvl.as_ref().unwrap().v;
            let heading_level = (level + 1).min(6).max(1) as u32;
            tracing::debug!("Found outline level in para_props {} -> H{}", level, heading_level);
            return Some(heading_level);
        }
        
        // 4. 最后才使用文本模式检测作为后备方案
        let mut full_text = String::new();
        for child in &paragraph.children {
            if let ParagraphChild::Run(run) = child {
                for run_child in &run.children {
                    if let RunChild::Text(text) = run_child {
                        full_text.push_str(&text.text);
                    }
                }
            }
        }
        
        let text_content = full_text.trim();
        
        // 只有在没有样式信息时才使用文本模式检测
        if self.is_heading_by_pattern(text_content) {
            tracing::debug!("Using text pattern detection for: '{}'", text_content);
            // 根据文本模式判断标题级别
            if text_content.starts_with("1.") || text_content.starts_with("2.") || 
               text_content.starts_with("3.") || text_content.starts_with("4.") ||
               text_content.starts_with("5.") || text_content.starts_with("6.") {
                tracing::debug!("Text pattern detected H1: '{}'", text_content);
                return Some(1); // 一级标题
            } else if text_content.starts_with("1.1") || text_content.starts_with("1.2") ||
                      text_content.starts_with("2.1") || text_content.starts_with("2.2") ||
                      text_content.starts_with("3.1") || text_content.starts_with("3.2") ||
                      text_content.starts_with("4.1") || text_content.starts_with("4.2") {
                tracing::debug!("Text pattern detected H2: '{}'", text_content);
                return Some(2); // 二级标题
            } else if text_content.starts_with("1.1.1") || text_content.starts_with("1.1.2") ||
                      text_content.starts_with("2.1.1") || text_content.starts_with("2.1.2") {
                tracing::debug!("Text pattern detected H3: '{}'", text_content);
                return Some(3); // 三级标题
            } else {
                // 根据文本长度和内容特征判断
                if text_content.len() < 30 && !text_content.contains('。') {
                    tracing::debug!("Text pattern detected H1 by length: '{}'", text_content);
                    return Some(1);
                } else if text_content.len() < 50 && !text_content.contains('。') {
                    tracing::debug!("Text pattern detected H2 by length: '{}'", text_content);
                    return Some(2);
                } else {
                    tracing::debug!("Text pattern detected H3 by length: '{}'", text_content);
                    return Some(3);
                }
            }
        }
        
        tracing::debug!("No heading detected for: '{}'", text_content);
        None
    }
    
    /// 基于文本模式判断是否为标题
    fn is_heading_by_pattern(&self, text: &str) -> bool {
        let text = text.trim();
        
        // 检查是否匹配标题模式
        if text.is_empty() {
            return false;
        }
        
        // 检查数字编号模式 (1., 2., 3., 1.1, 2.1, 3.1, 1.1.1, 2.1.1 等)
        if text.starts_with("1.") || text.starts_with("2.") || text.starts_with("3.") ||
           text.starts_with("4.") || text.starts_with("5.") || text.starts_with("6.") ||
           text.starts_with("7.") || text.starts_with("8.") || text.starts_with("9.") {
            return true;
        }
        
        // 检查子标题模式 (1.1, 1.2, 2.1, 2.2 等)
        if text.starts_with("1.1") || text.starts_with("1.2") || text.starts_with("1.3") ||
           text.starts_with("2.1") || text.starts_with("2.2") || text.starts_with("2.3") ||
           text.starts_with("3.1") || text.starts_with("3.2") || text.starts_with("3.3") ||
           text.starts_with("4.1") || text.starts_with("4.2") || text.starts_with("4.3") {
            return true;
        }
        
        // 检查三级标题模式 (1.1.1, 1.1.2, 2.1.1 等)
        if text.starts_with("1.1.1") || text.starts_with("1.1.2") || text.starts_with("1.1.3") ||
           text.starts_with("2.1.1") || text.starts_with("2.1.2") || text.starts_with("2.1.3") ||
           text.starts_with("3.1.1") || text.starts_with("3.1.2") || text.starts_with("3.1.3") {
            return true;
        }
        
        // 检查是否包含常见的标题关键词
        let heading_keywords = [
            "战略", "实践", "展望", "启示", "必要性", "趋势", "模式", "价值", "创新",
            "发展", "应用", "技术", "能力", "优势", "挑战", "机遇", "未来", "总结",
            "结论", "建议", "方案", "计划", "目标", "策略", "方法", "工具", "平台"
        ];
        
        for keyword in &heading_keywords {
            if text.contains(keyword) && text.len() < 100 && !text.contains('。') {
                return true;
            }
        }
        
        false
    }

    /// 处理段落内容
    fn process_paragraph(
        &self,
        paragraph: &Paragraph,
        html_content: &mut String,
        extracted_images: &mut Vec<ExtractedImage>,
    ) -> FlowyResult<()> {
        for (i, child) in paragraph.children.iter().enumerate() {
            match child {
                ParagraphChild::Run(run) => {
                    tracing::debug!("Processing run {}: {} children", i, run.children.len());
                    self.process_run(run, html_content, extracted_images)?;
                }
                ParagraphChild::Insert(_) => {
                    tracing::debug!("Processing insert in paragraph");
                    // 处理插入的内容
                }
                ParagraphChild::Delete(_) => {
                    tracing::debug!("Processing delete in paragraph");
                    // 处理删除的内容
                }
                _ => {
                    tracing::debug!("Skipping paragraph child: {:?}", std::mem::discriminant(child));
                }
            }
        }
        Ok(())
    }

    /// 处理运行内容
    fn process_run(
        &self,
        run: &Run,
        html_content: &mut String,
        extracted_images: &mut Vec<ExtractedImage>,
    ) -> FlowyResult<()> {
        let mut text_content = String::new();
        let mut is_bold = false;
        let mut is_italic = false;

        // 检查运行属性
        if let Some(bold) = &run.run_property.bold {
            is_bold = serde_json::to_value(bold).unwrap_or(serde_json::Value::Bool(false)).as_bool().unwrap_or(false);
        }
        if let Some(italic) = &run.run_property.italic {
            is_italic = serde_json::to_value(italic).unwrap_or(serde_json::Value::Bool(false)).as_bool().unwrap_or(false);
        }

        // 处理运行中的子元素
        for (i, child) in run.children.iter().enumerate() {
            match child {
                RunChild::Text(text) => {
                    tracing::debug!("Processing text {}: '{}'", i, self.safe_string_preview(&text.text, 50));
                    text_content.push_str(&text.text);
                }
                RunChild::Drawing(drawing) => {
                    tracing::debug!("Processing drawing {} in run", i);
                    if self.config.extract_images {
                        // 检查drawing是否有图片ID
                        if let Some(DrawingData::Pic(pic)) = &drawing.data {
                            tracing::debug!("Found drawing with pic id: {}", pic.id);
                            // 在HTML中插入图片标签，使用图片ID作为src
                            html_content.push_str(&format!("<img src='{}' alt='extracted image' />", pic.id));
                        }
                    }
                }
                _ => {
                    tracing::debug!("Skipping run child {}: {:?}", i, std::mem::discriminant(child));
                }
            }
        }

        // 应用格式
        if !text_content.is_empty() {
            if is_bold && is_italic {
                html_content.push_str(&format!("<strong><em>{}</em></strong>", text_content));
            } else if is_bold {
                html_content.push_str(&format!("<strong>{}</strong>", text_content));
            } else if is_italic {
                html_content.push_str(&format!("<em>{}</em>", text_content));
            } else {
                html_content.push_str(&text_content);
            }
        }

        Ok(())
    }

    /// 处理表格
    fn process_table(
        &self,
        table: &Table,
        html_content: &mut String,
        extracted_images: &mut Vec<ExtractedImage>,
    ) -> FlowyResult<()> {
        for row_child in &table.rows {
            let TableChild::TableRow(row) = row_child;
            html_content.push_str("<tr>");
            for cell_child in &row.cells {
                let TableRowChild::TableCell(cell) = cell_child;
                html_content.push_str("<td>");
                for content in &cell.children {
                    if let TableCellContent::Paragraph(p) = content {
                        self.process_paragraph(p, html_content, extracted_images)?;
                    }
                }
                html_content.push_str("</td>");
            }
            html_content.push_str("</tr>");
        }
        Ok(())
    }

    /// 提取绘图中的图片
    fn extract_drawing_image(
        &self,
        drawing: &Drawing,
    ) -> FlowyResult<Option<ExtractedImage>> {
        tracing::debug!("🔍 Processing drawing element");
        
        // 检查drawing是否包含图片数据
        if let Some(drawing_data) = &drawing.data {
            tracing::debug!("Drawing has data: {:?}", drawing_data);
            
            match drawing_data {
                DrawingData::Pic(pic) => {
                    tracing::debug!("Found Pic data: id='{}', image_data_len={}", pic.id, pic.image.len());
                    
                    // 检查图片数据是否为空
                    if pic.image.is_empty() {
                        tracing::warn!("Picture has empty image data for id: {}", pic.id);
                        return Ok(None);
                    }
                    
                    // 检测图片格式
                    let format = self.detect_image_format(&pic.image);
                    let extension = self.format_to_extension(format.clone());
                    
                    // 生成文件名
                    let filename = format!("image_{}.{}", pic.id, extension);
                    
                    // 获取图片尺寸
                    let dimensions = self.get_image_dimensions(&pic.image)?;
                    
                    // 创建ExtractedImage
                    let extracted_image = ExtractedImage {
                        id: pic.id.clone(),
                        filename,
                        data: pic.image.clone(),
                        format,
                        dimensions,
                    };
                    
                    tracing::info!("✅ Successfully extracted image: {} ({} bytes, format: {:?})", 
                        extracted_image.filename, 
                        extracted_image.data.len(),
                        extracted_image.format);
                    
                    Ok(Some(extracted_image))
                }
                DrawingData::TextBox(text_box) => {
                    tracing::debug!("Found TextBox data, skipping");
                    Ok(None)
                }
            }
        } else {
            tracing::debug!("Drawing has no data");
            Ok(None)
        }
    }

    /// 检测图片格式
    fn detect_image_format(&self, data: &[u8]) -> ImageFormat {
        if data.len() >= 4 {
            match &data[0..4] {
                [0x89, 0x50, 0x4E, 0x47] => ImageFormat::Png,
                [0xFF, 0xD8, 0xFF, _] => ImageFormat::Jpeg,
                [0x47, 0x49, 0x46, 0x38] => ImageFormat::Gif,
                [0x42, 0x4D, _, _] => ImageFormat::Bmp,
                [0x52, 0x49, 0x46, 0x46] if data.len() >= 12 && &data[8..12] == b"WEBP" => ImageFormat::WebP,
                _ => ImageFormat::Png, // 默认
            }
        } else {
            ImageFormat::Png
        }
    }

    /// 获取图片尺寸
    fn get_image_dimensions(&self, data: &[u8]) -> FlowyResult<Option<ImageDimensions>> {
        match self.detect_image_format(data) {
            ImageFormat::Png => {
                if let Ok(decoder) = image::codecs::png::PngDecoder::new(std::io::Cursor::new(data)) {
                    let dimensions = decoder.dimensions();
                    return Ok(Some(ImageDimensions {
                        width: dimensions.0,
                        height: dimensions.1,
                    }));
                }
            }
            ImageFormat::Jpeg => {
                if let Ok(decoder) = image::codecs::jpeg::JpegDecoder::new(std::io::Cursor::new(data)) {
                    let dimensions = decoder.dimensions();
                    return Ok(Some(ImageDimensions {
                        width: dimensions.0,
                        height: dimensions.1,
                    }));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// 格式转换为扩展名
    fn format_to_extension(&self, format: ImageFormat) -> &'static str {
        match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::WebP => "webp",
        }
    }

    /// 提取文档元数据
    fn extract_metadata(&self, docx: &Docx) -> DocumentMetadata {
        let mut metadata = DocumentMetadata {
            author: None,
            created_at: None,
            modified_at: None,
            page_count: None,
            word_count: None,
            language: None,
        };

        // 从文档属性中提取信息
        let props = &docx.doc_props.core;
        let props_json = serde_json::to_value(props).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let Some(config) = props_json.get("config") {
            metadata.author = config.get("creator").and_then(|v| v.as_str()).map(|s| s.to_string());
            metadata.created_at = config.get("created").and_then(|v| v.as_str()).and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
            metadata.modified_at = config.get("modified").and_then(|v| v.as_str()).and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
            metadata.language = config.get("language").and_then(|v| v.as_str()).map(|s| s.to_string());
        }

        // 计算字数（简化实现）
        let mut word_count = 0u32;
        for child in &docx.document.children {
            if let DocumentChild::Paragraph(p) = child {
                for paragraph_child in &p.children {
                    if let ParagraphChild::Run(run) = paragraph_child {
                        for run_child in &run.children {
                            if let RunChild::Text(text) = run_child {
                                word_count += text.text.split_whitespace().count() as u32;
                            }
                        }
                    }
                }
            }
        }
        metadata.word_count = Some(word_count);

        metadata
    }
    
    /// 直接从ZIP文件中提取图片（替代方案）
    fn extract_images_from_zip(&self, file_data: &[u8]) -> FlowyResult<Vec<ExtractedImage>> {
        use std::io::Cursor;
        use zip::ZipArchive;
        
        tracing::info!("🔧 Starting alternative image extraction from ZIP...");
        
        let cursor = Cursor::new(file_data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| ConversionError::ConversionFailed(format!("Failed to open ZIP archive: {}", e)))?;
        
        let mut extracted_images = Vec::new();
        
        // 1. 首先解析关系文件来获取图片ID到文件路径的映射
        let mut image_relations = HashMap::new();
        
        // 尝试读取 document.xml.rels 文件
        if let Ok(mut rels_file) = archive.by_name("word/_rels/document.xml.rels") {
            let mut rels_content = String::new();
            std::io::Read::read_to_string(&mut rels_file, &mut rels_content)
                .map_err(|e| ConversionError::ConversionFailed(format!("Failed to read rels file: {}", e)))?;
            
            tracing::info!("📄 Parsing document.xml.rels file...");
            
            // 解析XML来提取图片关系
            image_relations = self.parse_image_relations(&rels_content)?;
            tracing::info!("🔗 Found {} image relations", image_relations.len());
        } else {
            tracing::warn!("⚠️ Could not find word/_rels/document.xml.rels file");
        }
        
        // 2. 遍历ZIP文件中的所有文件，查找媒体文件
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| ConversionError::ConversionFailed(format!("Failed to read file {}: {}", i, e)))?;
            
            let file_name = file.name();
            
            // 检查是否是媒体文件
            if file_name.starts_with("word/media/") {
                tracing::info!("📸 Found media file: {}", file_name);
                
                // 尝试从关系映射中找到对应的图片ID
                let image_id = self.find_image_id_for_file(&image_relations, file_name);
                
                if let Some(id) = image_id {
                    tracing::info!("🔗 Found relation for {} -> {}", file_name, id);
                    
                    // 读取文件内容
                    let mut file_data = Vec::new();
                    std::io::Read::read_to_end(&mut file, &mut file_data)
                        .map_err(|e| ConversionError::ConversionFailed(format!("Failed to read media file: {}", e)))?;
                    
                    if !file_data.is_empty() {
                        let data_len = file_data.len();
                        
                        // 检测图片格式
                        let format = self.detect_image_format(&file_data);
                        let extension = self.format_to_extension(format.clone());
                        
                        // 生成文件名
                        let filename = format!("image_{}.{}", id, extension);
                        
                        // 获取图片尺寸
                        let dimensions = self.get_image_dimensions(&file_data)?;
                        
                        // 创建ExtractedImage
                        let extracted_image = ExtractedImage {
                            id: id.clone(),
                            filename,
                            data: file_data,
                            format,
                            dimensions,
                        };
                        
                        extracted_images.push(extracted_image);
                        tracing::info!("✅ Extracted image: {} ({} bytes)", id, data_len);
                    }
                } else {
                    tracing::warn!("⚠️ No relation found for media file: {}", file_name);
                }
            }
        }
        
        tracing::info!("📊 Alternative extraction completed: {} images found", extracted_images.len());
        Ok(extracted_images)
    }
    
    /// 解析图片关系XML文件
    fn parse_image_relations(&self, xml_content: &str) -> FlowyResult<HashMap<String, String>> {
        use xml::reader::{EventReader, XmlEvent};
        
        let mut relations = HashMap::new();
        let parser = EventReader::from_str(xml_content);
        
        let mut current_id = String::new();
        let mut current_target = String::new();
        let mut in_relationship = false;
        
        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    if name.local_name == "Relationship" {
                        in_relationship = true;
                        current_id.clear();
                        current_target.clear();
                        
                        // 提取属性
                        for attr in attributes {
                            if attr.name.local_name == "Id" {
                                current_id = attr.value;
                            } else if attr.name.local_name == "Target" {
                                current_target = attr.value;
                            }
                        }
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    if name.local_name == "Relationship" && in_relationship {
                        // 检查是否是图片关系
                        if current_target.contains("media/") && !current_id.is_empty() && !current_target.is_empty() {
                            relations.insert(current_target.clone(), current_id.clone());
                            tracing::debug!("🔗 Image relation: {} -> {}", current_id, current_target);
                        }
                        in_relationship = false;
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ XML parsing error: {}", e);
                }
                _ => {}
            }
        }
        
        Ok(relations)
    }
    
    /// 根据文件路径查找对应的图片ID
    fn find_image_id_for_file(&self, relations: &HashMap<String, String>, file_path: &str) -> Option<String> {
        // 直接查找
        if let Some(id) = relations.get(file_path) {
            return Some(id.clone());
        }
        
        // 尝试不同的路径格式
        let variations = vec![
            file_path.to_string(),
            format!("word/{}", file_path),
            file_path.replace("word/media/", "media/"),
        ];
        
        for variation in variations {
            if let Some(id) = relations.get(&variation) {
                return Some(id.clone());
            }
        }
        
        None
    }
}

#[async_trait::async_trait]
impl DocumentConverter for WordConverter {
    fn name(&self) -> &str {
        "Word Document Converter"
    }

    fn supported_types(&self) -> Vec<DocumentType> {
        vec![DocumentType::Word]
    }

    async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        
        // 解析Word文档 - 支持从字节数据或文件路径
        let (html_content, extracted_images, metadata) = if let Some(bytes_data) = &task.bytes_data {
            // 从字节数据解析
            self.parse_word_document_from_bytes(bytes_data)?
        } else {
            // 验证文件并从文件路径解析
            self.validate_file(Path::new(&task.source_path)).await?;
            self.parse_word_document(Path::new(&task.source_path))?
        };

        // 计算统计信息
        let processing_time = start_time.elapsed().as_millis() as u64;
        let file_size = if let Some(bytes_data) = &task.bytes_data {
            bytes_data.len() as u64
        } else {
            std::fs::metadata(&task.source_path)?.len()
        };
        let table_count = html_content.matches("<table").count() as u32;
        let text_length = html_content.len();
        let image_count = extracted_images.len() as u32;

        let result = ConversionResult {
            task_id: task.id,
            document_content: DocumentContent {
                title: task.target_name.clone(),
                content: html_content,
                metadata,
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
            if !DocumentType::Word.supported_extensions().contains(&ext.to_lowercase().as_str()) {
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

        // 尝试打开文件验证格式
        let file_data = std::fs::read(file_path)?;
        if let Err(_) = docx_rs::read_docx(&file_data) {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::InvalidParams,
                "Invalid Word document format",
            ));
        }

        Ok(())
    }

    async fn get_file_info(&self, file_path: &Path) -> FlowyResult<DocumentMetadata> {
        // 验证文件
        self.validate_file(file_path).await?;

        // 解析文档获取元数据
        let (_, _, metadata) = self.parse_word_document(file_path)?;
        
        Ok(metadata)
    }
}

