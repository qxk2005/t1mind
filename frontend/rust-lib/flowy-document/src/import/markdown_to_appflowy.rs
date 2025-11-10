use crate::import::converter::{ExtractedImage, ImageFormat, ImageDimensions};
use crate::parser::constant::*;
use crate::parser::parser_entities::{InsertDelta, NestedBlock};
use base64::{engine::general_purpose, Engine as _};
use flowy_error::FlowyResult;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Markdown 到 AppFlowy 文档转换器
/// 
/// 使用 pulldown-cmark 解析 Markdown，并将其转换为 AppFlowy NestedBlock 结构。
/// 支持标题（H1-H6）、表格、列表（有序/无序）、代码块、图片引用等元素。
pub struct MarkdownToAppFlowyConverter {
    /// 提取的图片列表，用于匹配 Markdown 中的图片引用
    extracted_images: Vec<ExtractedImage>,
    /// 已使用的图片 ID 集合，用于避免重复嵌入
    used_image_ids: HashSet<String>,
}

impl MarkdownToAppFlowyConverter {
    /// 创建新的转换器实例
    pub fn new(extracted_images: Vec<ExtractedImage>) -> Self {
        Self {
            extracted_images,
            used_image_ids: HashSet::new(),
        }
    }

    /// 将 Markdown 文本转换为 AppFlowy NestedBlock 结构
    /// 
    /// # 参数
    /// - `markdown`: 要转换的 Markdown 文本
    /// 
    /// # 返回
    /// - `Ok(NestedBlock)`: 转换后的文档根节点（类型为 "page"）
    /// - `Err(FlowyError)`: 转换过程中的错误
    pub fn convert(&mut self, markdown: &str) -> FlowyResult<NestedBlock> {
        if markdown.trim().is_empty() {
            return Ok(NestedBlock {
                ty: PAGE.to_string(),
                data: HashMap::new(),
                children: vec![],
            });
        }

        // 配置 pulldown-cmark 解析选项
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown, options);

        // 状态变量
        let mut root = NestedBlock {
            ty: PAGE.to_string(),
            data: HashMap::new(),
            children: vec![],
        };

        let mut current_paragraph_text: Vec<InsertDelta> = vec![];
        let mut current_list_items: Vec<NestedBlock> = vec![];
        let mut current_list_type: Option<String> = None; // "numbered_list" 或 "bulleted_list"
        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;
        let mut code_block_content = String::new();
        let mut in_table = false;
        let mut table_headers: Vec<String> = vec![];
        let mut table_rows: Vec<Vec<String>> = vec![];
        let mut current_table_row: Vec<String> = vec![];
        let mut current_table_cell = String::new();
        let mut in_table_header = false;
        let mut in_table_row = false;
        
        // 内联格式属性栈（用于处理嵌套的格式，如加粗、斜体、链接等）
        let mut inline_attributes_stack: Vec<HashMap<String, Value>> = vec![];
        let mut current_link: Option<String> = None;
        
        // 标记上一个块是否是图片，用于跳过图片后的空段落
        let mut last_was_image = false;

        for event in parser {
            match event {
                // 标题开始
                Event::Start(Tag::Heading(level, _, _)) => {
                    // 刷新当前段落或列表
                    self.flush_paragraph(
                        &mut current_paragraph_text,
                        &mut current_list_items,
                        &mut current_list_type,
                        &mut root,
                    )?;
                }
                // 标题结束
                Event::End(Tag::Heading(level, _, _)) => {
                    if !current_paragraph_text.is_empty() {
                        let level_u32 = level as u32;
                        let heading_block = self.create_heading_block_from_deltas(level_u32, &current_paragraph_text)?;
                        root.children.push(heading_block);
                    }
                    current_paragraph_text.clear();
                    last_was_image = false; // 重置标记
                }
                // 段落开始
                Event::Start(Tag::Paragraph) => {
                    current_paragraph_text.clear();
                }
                // 段落结束
                Event::End(Tag::Paragraph) => {
                    if !in_table && !in_code_block {
                        // 如果上一个块是图片，且当前段落为空，则跳过这个空段落
                        // 这样可以避免在图片后创建空白段落
                        if last_was_image && current_paragraph_text.is_empty() {
                            current_paragraph_text.clear();
                            last_was_image = false; // 重置标记
                        } else if !current_paragraph_text.is_empty() {
                            let paragraph_block = self.create_paragraph_block_from_deltas(&current_paragraph_text)?;
                            root.children.push(paragraph_block);
                            last_was_image = false; // 重置标记
                        } else {
                            last_was_image = false; // 重置标记
                        }
                        current_paragraph_text.clear();
                    }
                }
                // 有序列表开始
                Event::Start(Tag::List(Some(_))) => {
                    self.flush_paragraph(
                        &mut current_paragraph_text,
                        &mut current_list_items,
                        &mut current_list_type,
                        &mut root,
                    )?;
                    current_list_type = Some(NUMBERED_LIST.to_string());
                    current_list_items.clear();
                }
                // 无序列表开始
                Event::Start(Tag::List(None)) => {
                    self.flush_paragraph(
                        &mut current_paragraph_text,
                        &mut current_list_items,
                        &mut current_list_type,
                        &mut root,
                    )?;
                    current_list_type = Some(BULLETED_LIST.to_string());
                    current_list_items.clear();
                }
                // 列表结束
                Event::End(Tag::List(_)) => {
                    if let Some(list_type) = &current_list_type {
                        if !current_list_items.is_empty() {
                            let list_block = self.create_list_block(list_type, &current_list_items)?;
                            root.children.push(list_block);
                        }
                        current_list_items.clear();
                        current_list_type = None;
                    }
                }
                // 列表项开始
                Event::Start(Tag::Item) => {
                    // 列表项开始，准备收集内容
                }
                // 列表项结束
                Event::End(Tag::Item) => {
                    if !current_paragraph_text.is_empty() {
                        let list_item_block = self.create_list_item_block_from_deltas(&current_paragraph_text)?;
                        current_list_items.push(list_item_block);
                    }
                    current_paragraph_text.clear();
                }
                // 代码块开始（围栏式）
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                    in_code_block = true;
                    code_block_lang = Some(lang.to_string());
                    code_block_content.clear();
                }
                // 代码块开始（缩进式）
                Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                    in_code_block = true;
                    code_block_lang = None;
                    code_block_content.clear();
                }
                // 代码块结束
                Event::End(Tag::CodeBlock(_)) => {
                    if !code_block_content.trim().is_empty() {
                        let code_block = self.create_code_block(&code_block_content, &code_block_lang)?;
                        root.children.push(code_block);
                    }
                    in_code_block = false;
                    code_block_lang = None;
                    code_block_content.clear();
                    last_was_image = false; // 重置标记
                }
                // 表格开始
                Event::Start(Tag::Table(_)) => {
                    self.flush_paragraph(
                        &mut current_paragraph_text,
                        &mut current_list_items,
                        &mut current_list_type,
                        &mut root,
                    )?;
                    in_table = true;
                    table_headers.clear();
                    table_rows.clear();
                }
                // 表格结束
                Event::End(Tag::Table(_)) => {
                    if !table_headers.is_empty() || !table_rows.is_empty() {
                        let table_block = self.create_table_block(&table_headers, &table_rows)?;
                        root.children.push(table_block);
                    }
                    in_table = false;
                    in_table_header = false;
                    in_table_row = false;
                    table_headers.clear();
                    table_rows.clear();
                    current_table_row.clear();
                    current_table_cell.clear();
                    last_was_image = false; // 重置标记
                }
                // 表头开始
                Event::Start(Tag::TableHead) => {
                    in_table_header = true;
                }
                // 表头结束
                Event::End(Tag::TableHead) => {
                    in_table_header = false;
                }
                // 表格行开始
                Event::Start(Tag::TableRow) => {
                    in_table_row = true;
                    current_table_row.clear();
                }
                // 表格行结束
                Event::End(Tag::TableRow) => {
                    if in_table_header {
                        // 表头行
                        if !current_table_row.is_empty() {
                            table_headers = current_table_row.clone();
                        }
                    } else {
                        // 数据行
                        if !current_table_row.is_empty() {
                            table_rows.push(current_table_row.clone());
                        }
                    }
                    in_table_row = false;
                    current_table_row.clear();
                }
                // 表格单元格开始
                Event::Start(Tag::TableCell) => {
                    current_table_cell.clear();
                }
                // 表格单元格结束
                Event::End(Tag::TableCell) => {
                    current_table_row.push(current_table_cell.clone());
                    current_table_cell.clear();
                }
                // 文本内容
                Event::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(&text);
                    } else if in_table {
                        current_table_cell.push_str(&text);
                    } else {
                        // 合并当前的内联属性
                        let mut attributes = HashMap::new();
                        for attr_map in &inline_attributes_stack {
                            for (k, v) in attr_map {
                                attributes.insert(k.clone(), v.clone());
                            }
                        }
                        // 如果有链接，添加链接属性
                        if let Some(ref href) = current_link {
                            attributes.insert(HREF.to_string(), Value::String(href.clone()));
                        }
                        
                        current_paragraph_text.push(InsertDelta {
                            insert: text.to_string(),
                            attributes: if attributes.is_empty() {
                                None
                            } else {
                                Some(attributes)
                            },
                        });
                    }
                }
                // 代码片段（行内代码）
                Event::Code(code) => {
                    if !in_code_block {
                        current_paragraph_text.push(InsertDelta {
                            insert: code.to_string(),
                            attributes: Some({
                                let mut attrs = HashMap::new();
                                attrs.insert(CODE.to_string(), Value::Bool(true));
                                attrs
                            }),
                        });
                    }
                }
                // 图片
                Event::Start(Tag::Image(_, url, title)) => {
                    // 图片开始，我们会在 End 事件中处理
                    // 清空当前段落文本，避免图片后的空段落被创建
                    current_paragraph_text.clear();
                }
                Event::End(Tag::Image(_, url, alt_text)) => {
                    // 创建图片块（注意：需要可变引用）
                    let image_block = self.create_image_block(&url, &alt_text)?;
                    root.children.push(image_block);
                    // 确保清空当前段落文本，避免图片后的空段落被创建
                    current_paragraph_text.clear();
                    // 标记上一个块是图片，用于跳过后续的空段落
                    last_was_image = true;
                }
                // 链接
                Event::Start(Tag::Link(_, href, _)) => {
                    current_link = Some(href.to_string());
                }
                Event::End(Tag::Link(_, _, _)) => {
                    current_link = None;
                }
                // 强调（斜体）
                Event::Start(Tag::Emphasis) => {
                    let mut attrs = HashMap::new();
                    attrs.insert(ITALIC.to_string(), Value::Bool(true));
                    inline_attributes_stack.push(attrs);
                }
                Event::End(Tag::Emphasis) => {
                    inline_attributes_stack.pop();
                }
                // 加粗
                Event::Start(Tag::Strong) => {
                    let mut attrs = HashMap::new();
                    attrs.insert(BOLD.to_string(), Value::Bool(true));
                    inline_attributes_stack.push(attrs);
                }
                Event::End(Tag::Strong) => {
                    inline_attributes_stack.pop();
                }
                // 删除线
                Event::Start(Tag::Strikethrough) => {
                    let mut attrs = HashMap::new();
                    attrs.insert(STRIKETHROUGH.to_string(), Value::Bool(true));
                    inline_attributes_stack.push(attrs);
                }
                Event::End(Tag::Strikethrough) => {
                    inline_attributes_stack.pop();
                }
                // 软换行
                Event::SoftBreak => {
                    if !in_code_block && !in_table {
                        current_paragraph_text.push(InsertDelta {
                            insert: " ".to_string(),
                            attributes: None,
                        });
                    } else if in_code_block {
                        code_block_content.push('\n');
                    } else if in_table {
                        current_table_cell.push(' ');
                    }
                }
                // 硬换行
                Event::HardBreak => {
                    if !in_code_block && !in_table {
                        current_paragraph_text.push(InsertDelta {
                            insert: "\n".to_string(),
                            attributes: None,
                        });
                    } else if in_code_block {
                        code_block_content.push('\n');
                    } else if in_table {
                        current_table_cell.push('\n');
                    }
                }
                // 其他事件（如 HTML、脚注等）
                _ => {
                    debug!("未处理的 Markdown 事件: {:?}", event);
                }
            }
        }

        // 刷新剩余的段落或列表
        self.flush_paragraph(
            &mut current_paragraph_text,
            &mut current_list_items,
            &mut current_list_type,
            &mut root,
        )?;

        Ok(root)
    }

    /// 刷新当前段落或列表到根节点
    fn flush_paragraph(
        &self,
        current_paragraph_text: &mut Vec<InsertDelta>,
        current_list_items: &mut Vec<NestedBlock>,
        current_list_type: &mut Option<String>,
        root: &mut NestedBlock,
    ) -> FlowyResult<()> {
        // 如果有待处理的列表，先处理列表
        if let Some(list_type) = current_list_type.take() {
            if !current_list_items.is_empty() {
                let list_block = self.create_list_block(&list_type, current_list_items)?;
                root.children.push(list_block);
                current_list_items.clear();
            }
        }

        // 如果有待处理的段落，处理段落
        if !current_paragraph_text.is_empty() {
            let paragraph_block = self.create_paragraph_block_from_deltas(current_paragraph_text)?;
            root.children.push(paragraph_block);
            current_paragraph_text.clear();
        }

        Ok(())
    }

    /// 创建标题块（从 InsertDelta 向量）
    fn create_heading_block_from_deltas(&self, level: u32, deltas: &[InsertDelta]) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();
        data.insert(LEVEL.to_string(), Value::Number(level.min(6).into()));

        // 创建 delta JSON
        if let Ok(delta) = serde_json::to_value(deltas) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: HEADING.to_string(),
            data,
            children: vec![],
        })
    }

    /// 创建标题块（从纯文本，用于向后兼容）
    fn create_heading_block(&self, level: u32, text: &str) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();
        data.insert(LEVEL.to_string(), Value::Number(level.min(6).into()));

        // 创建文本 delta
        if let Ok(delta) = serde_json::to_value(vec![InsertDelta {
            insert: text.to_string(),
            attributes: None,
        }]) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: HEADING.to_string(),
            data,
            children: vec![],
        })
    }

    /// 创建段落块（从 InsertDelta 向量）
    fn create_paragraph_block_from_deltas(&self, deltas: &[InsertDelta]) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();

        // 创建 delta JSON
        if let Ok(delta) = serde_json::to_value(deltas) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: PARAGRAPH.to_string(),
            data,
            children: vec![],
        })
    }

    /// 创建段落块（从纯文本，用于向后兼容）
    fn create_paragraph_block(&self, text: &str) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();

        // 创建文本 delta
        if let Ok(delta) = serde_json::to_value(vec![InsertDelta {
            insert: text.to_string(),
            attributes: None,
        }]) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: PARAGRAPH.to_string(),
            data,
            children: vec![],
        })
    }

    /// 创建列表块
    fn create_list_block(&self, list_type: &str, items: &[NestedBlock]) -> FlowyResult<NestedBlock> {
        Ok(NestedBlock {
            ty: list_type.to_string(),
            data: HashMap::new(),
            children: items.to_vec(),
        })
    }

    /// 创建列表项块（从 InsertDelta 向量）
    fn create_list_item_block_from_deltas(&self, deltas: &[InsertDelta]) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();

        // 创建 delta JSON
        if let Ok(delta) = serde_json::to_value(deltas) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: PARAGRAPH.to_string(), // 列表项使用 paragraph 类型
            data,
            children: vec![],
        })
    }

    /// 创建列表项块（从纯文本，用于向后兼容）
    fn create_list_item_block(&self, text: &str) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();

        // 创建文本 delta
        if let Ok(delta) = serde_json::to_value(vec![InsertDelta {
            insert: text.to_string(),
            attributes: None,
        }]) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: PARAGRAPH.to_string(), // 列表项使用 paragraph 类型
            data,
            children: vec![],
        })
    }

    /// 创建代码块
    fn create_code_block(&self, content: &str, lang: &Option<String>) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();

        // 设置语言
        if let Some(language) = lang {
            data.insert(LANGUAGE.to_string(), Value::String(language.clone()));
        } else {
            data.insert(LANGUAGE.to_string(), Value::String(String::new()));
        }

        // 创建代码内容 delta
        if let Ok(delta) = serde_json::to_value(vec![InsertDelta {
            insert: content.to_string(),
            attributes: None,
        }]) {
            data.insert(DELTA.to_string(), delta);
        }

        Ok(NestedBlock {
            ty: CODE.to_string(),
            data,
            children: vec![],
        })
    }

    /// 创建表格块
    fn create_table_block(&self, headers: &[String], rows: &[Vec<String>]) -> FlowyResult<NestedBlock> {
        // 计算列数
        let col_count = headers
            .len()
            .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

        let mut data = HashMap::new();
        data.insert("col_count".to_string(), Value::Number(col_count.into()));
        data.insert(
            "row_count".to_string(),
            Value::Number((rows.len() + if headers.is_empty() { 0 } else { 1 }).into()),
        );

        let mut children = vec![];

        // 创建表头行（如果有）
        if !headers.is_empty() {
            let header_row = self.create_table_row(headers)?;
            children.push(header_row);
        }

        // 创建数据行
        for row in rows {
            let table_row = self.create_table_row(row)?;
            children.push(table_row);
        }

        Ok(NestedBlock {
            ty: "simple_table".to_string(),
            data,
            children,
        })
    }

    /// 创建表格行块
    fn create_table_row(&self, cells: &[String]) -> FlowyResult<NestedBlock> {
        let mut row_children = vec![];

        for cell_text in cells {
            let cell_block = self.create_table_cell(cell_text)?;
            row_children.push(cell_block);
        }

        Ok(NestedBlock {
            ty: "table_row".to_string(),
            data: HashMap::new(),
            children: row_children,
        })
    }

    /// 创建表格单元格块
    fn create_table_cell(&self, text: &str) -> FlowyResult<NestedBlock> {
        let mut cell_data = HashMap::new();

        // 创建单元格文本 delta
        if let Ok(delta) = serde_json::to_value(vec![InsertDelta {
            insert: text.to_string(),
            attributes: None,
        }]) {
            cell_data.insert(DELTA.to_string(), delta);
        }

        // 单元格包含一个段落子块
        let paragraph_block = NestedBlock {
            ty: PARAGRAPH.to_string(),
            data: cell_data.clone(),
            children: vec![],
        };

        Ok(NestedBlock {
            ty: "table_cell".to_string(),
            data: HashMap::new(),
            children: vec![paragraph_block],
        })
    }

    /// 创建图片块
    fn create_image_block(&mut self, url: &str, alt_text: &str) -> FlowyResult<NestedBlock> {
        let mut data = HashMap::new();

        // 尝试匹配提取的图片
        // 先获取匹配结果并克隆需要的数据，避免借用冲突
        let image_info = self.match_image(url).map(|(image, image_url)| {
            // 克隆所有需要的数据，避免持有引用
            (image.id.clone(), image.filename.clone(), image.data.clone(), 
             image.format.clone(), image.dimensions.clone(), image_url)
        });
        
        match image_info {
            Some((image_id, image_filename, image_data, image_format, image_dimensions, image_url)) => {
                // 检查是否已经使用过此图片（避免重复）
                if self.used_image_ids.contains(&image_id) {
                    debug!("图片 {} 已使用，跳过重复嵌入", image_id);
                    // 即使已使用，仍然创建图片块，但使用已存在的图片 ID
                    data.insert(URL.to_string(), Value::String(image_url));
                } else {
                    // 将图片数据编码为 base64 data URL
                    let mime_type = match image_format {
                        ImageFormat::Png => "image/png",
                        ImageFormat::Jpeg => "image/jpeg",
                        ImageFormat::Gif => "image/gif",
                        ImageFormat::Bmp => "image/bmp",
                        ImageFormat::WebP => "image/webp",
                    };
                    
                    let base64_data = general_purpose::STANDARD.encode(&image_data);
                    let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                    
                    data.insert(URL.to_string(), Value::String(data_url));
                    
                    // 添加图片尺寸信息
                    // 优先使用已有的尺寸信息，如果没有则从图片数据中读取
                    let final_dimensions = if let Some(dimensions) = &image_dimensions {
                        Some(dimensions.clone())
                    } else {
                        // 尝试从图片数据中读取尺寸
                        self.get_image_dimensions_from_data(&image_data, &image_format)
                            .unwrap_or(None)
                    };
                    
                    if let Some(dimensions) = &final_dimensions {
                        // 使用实际图片尺寸，而不是固定大小
                        data.insert(WIDTH.to_string(), Value::Number(dimensions.width.into()));
                        data.insert(HEIGHT.to_string(), Value::Number(dimensions.height.into()));
                        debug!("设置图片尺寸: {}x{}", dimensions.width, dimensions.height);
                    } else {
                        warn!("无法获取图片尺寸，图片块可能使用默认大小: {}", image_filename);
                    }
                    
                    // 标记图片为已使用（现在可以安全地可变借用）
                    self.used_image_ids.insert(image_id.clone());
                    debug!("成功嵌入图片: {} ({} bytes, {:?})", 
                        image_filename, 
                        image_data.len(),
                        image_format
                    );
                }
            }
            None => {
                // 图片未找到，记录警告但继续处理
                warn!("无法匹配图片引用: {}，将使用原始 URL", url);
                data.insert(URL.to_string(), Value::String(url.to_string()));
            }
        }

        // 添加图片说明文字（如果有）
        if !alt_text.is_empty() {
            data.insert(CAPTION.to_string(), Value::String(alt_text.to_string()));
        }

        Ok(NestedBlock {
            ty: IMAGE.to_string(),
            data,
            children: vec![],
        })
    }

    /// 匹配图片
    /// 
    /// 尝试从提取的图片列表中找到匹配的图片。
    /// 支持多种匹配策略：
    /// 1. 精确文件名匹配
    /// 2. 路径规范化后的匹配（处理相对路径、绝对路径）
    /// 3. 文件名部分匹配（忽略路径前缀）
    /// 
    /// # 参数
    /// - `url`: Markdown 中的图片 URL 或路径
    /// 
    /// # 返回
    /// - `Some((ExtractedImage, String))`: 匹配到的图片和要使用的 URL（base64 data URL）
    /// - `None`: 未找到匹配的图片
    fn match_image(&self, url: &str) -> Option<(&ExtractedImage, String)> {
        // 规范化 URL 路径
        let normalized_url = self.normalize_path(url);
        
        // 策略 1: 精确文件名匹配
        for image in &self.extracted_images {
            if image.filename == normalized_url || image.filename == url {
                let mime_type = match image.format {
                    ImageFormat::Png => "image/png",
                    ImageFormat::Jpeg => "image/jpeg",
                    ImageFormat::Gif => "image/gif",
                    ImageFormat::Bmp => "image/bmp",
                    ImageFormat::WebP => "image/webp",
                };
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                return Some((image, data_url));
            }
        }
        
        // 策略 2: 路径规范化后的匹配
        for image in &self.extracted_images {
            let normalized_filename = self.normalize_path(&image.filename);
            if normalized_filename == normalized_url {
                let mime_type = match image.format {
                    ImageFormat::Png => "image/png",
                    ImageFormat::Jpeg => "image/jpeg",
                    ImageFormat::Gif => "image/gif",
                    ImageFormat::Bmp => "image/bmp",
                    ImageFormat::WebP => "image/webp",
                };
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                return Some((image, data_url));
            }
        }
        
        // 策略 3: 文件名部分匹配（提取文件名部分进行比较）
        let url_filename = Path::new(url)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(url);
        
        for image in &self.extracted_images {
            let image_filename = Path::new(&image.filename)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&image.filename);
            
            if image_filename == url_filename {
                let mime_type = match image.format {
                    ImageFormat::Png => "image/png",
                    ImageFormat::Jpeg => "image/jpeg",
                    ImageFormat::Gif => "image/gif",
                    ImageFormat::Bmp => "image/bmp",
                    ImageFormat::WebP => "image/webp",
                };
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                return Some((image, data_url));
            }
        }
        
        // 策略 4: 包含匹配（作为最后的回退方案）
        for image in &self.extracted_images {
            if url.contains(&image.filename) || image.filename.contains(url) {
                let mime_type = match image.format {
                    ImageFormat::Png => "image/png",
                    ImageFormat::Jpeg => "image/jpeg",
                    ImageFormat::Gif => "image/gif",
                    ImageFormat::Bmp => "image/bmp",
                    ImageFormat::WebP => "image/webp",
                };
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                return Some((image, data_url));
            }
        }
        
        None
    }

    /// 从图片数据中读取尺寸
    /// 
    /// 如果图片的 dimensions 字段为 None，尝试从图片数据中读取实际尺寸。
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
        format: &ImageFormat,
    ) -> FlowyResult<Option<ImageDimensions>> {
        // 尝试使用 image crate 加载图片以获取尺寸
        match image::load_from_memory(data) {
            Ok(img) => {
                Ok(Some(ImageDimensions {
                    width: img.width(),
                    height: img.height(),
                }))
            }
            Err(e) => {
                debug!("无法从图片数据读取尺寸: {}", e);
                Ok(None)
            }
        }
    }

    /// 规范化路径
    /// 
    /// 处理相对路径、绝对路径，去除 `./`、`../` 等前缀，
    /// 统一路径分隔符，并提取文件名部分。
    /// 
    /// # 参数
    /// - `path`: 要规范化的路径字符串
    /// 
    /// # 返回
    /// - 规范化后的路径字符串
    fn normalize_path(&self, path: &str) -> String {
        // 移除 URL 协议前缀（如 file://, http:// 等）
        let path = path
            .trim_start_matches("file://")
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        
        // 使用 Path 来规范化路径
        let path_buf = PathBuf::from(path);
        
        // 提取文件名部分（如果路径是目录，尝试获取最后一个组件）
        let normalized = if path_buf.is_absolute() {
            // 绝对路径：提取文件名
            path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| path.to_string())
        } else {
            // 相对路径：规范化并提取文件名
            let components: Vec<_> = path_buf
                .components()
                .filter_map(|c| {
                    match c {
                        std::path::Component::Normal(s) => s.to_str(),
                        _ => None,
                    }
                })
                .collect();
            
            if components.is_empty() {
                path.to_string()
            } else {
                components.join("/")
            }
        };
        
        // 转换为小写以进行不区分大小写的匹配（在某些系统上）
        normalized.to_lowercase()
    }

    /// 将 InsertDelta 向量转换为纯文本
    fn deltas_to_text(&self, deltas: &[InsertDelta]) -> String {
        deltas.iter().map(|d| d.insert.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_empty_markdown() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let result = converter.convert("").unwrap();
        assert_eq!(result.ty, PAGE);
        assert!(result.children.is_empty());
    }

    #[test]
    fn test_convert_heading() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let result = converter.convert("# Hello World").unwrap();
        assert_eq!(result.ty, PAGE);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].ty, HEADING);
        assert_eq!(
            result.children[0]
                .data
                .get(LEVEL)
                .and_then(|v| v.as_u64())
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_convert_paragraph() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let result = converter.convert("This is a paragraph.").unwrap();
        assert_eq!(result.ty, PAGE);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].ty, PARAGRAPH);
    }

    #[test]
    fn test_convert_unordered_list() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let result = converter.convert("- Item 1\n- Item 2").unwrap();
        assert_eq!(result.ty, PAGE);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].ty, BULLETED_LIST);
        assert_eq!(result.children[0].children.len(), 2);
    }

    #[test]
    fn test_convert_ordered_list() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let result = converter.convert("1. Item 1\n2. Item 2").unwrap();
        assert_eq!(result.ty, PAGE);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].ty, NUMBERED_LIST);
        assert_eq!(result.children[0].children.len(), 2);
    }

    #[test]
    fn test_convert_code_block() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let result = converter.convert("```rust\nfn main() {}\n```").unwrap();
        assert_eq!(result.ty, PAGE);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].ty, CODE);
        assert_eq!(
            result.children[0]
                .data
                .get(LANGUAGE)
                .and_then(|v| v.as_str())
                .unwrap(),
            "rust"
        );
    }

    #[test]
    fn test_convert_table() {
        let mut converter = MarkdownToAppFlowyConverter::new(vec![]);
        let markdown = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let result = converter.convert(markdown).unwrap();
        assert_eq!(result.ty, PAGE);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].ty, "simple_table");
    }
}

