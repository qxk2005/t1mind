use crate::deps_resolve::folder_deps::get_encoded_collab_v1_from_disk;
use bytes::Bytes;
use collab::entity::EncodedCollab;
use collab_entity::CollabType;
use collab_folder::hierarchy_builder::NestedViewBuilder;
use collab_folder::ViewLayout;
use flowy_document::entities::{DocumentDataPB, BlockPB, ChildrenPB, MetaPB};
use flowy_document::manager::DocumentManager;
use flowy_document::parser::json::parser::JsonToDocumentParser;
use flowy_document::import::{DefaultConverterFactory, ConverterFactory, ConversionTask, DocumentType, ConversionStatus};
use flowy_error::FlowyError;
use flowy_folder::entities::{CreateViewParams, ViewLayoutPB};
use flowy_folder::manager::FolderUser;
use flowy_folder::share::ImportType;
use flowy_folder::view_operation::{
  FolderOperationHandler, GatherEncodedCollab, ImportedData, ViewData,
};
use flowy_search_pub::tantivy_state_init::get_document_tantivy_state;
use lib_dispatch::prelude::ToBytes;
use lib_infra::async_trait::async_trait;
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::{Arc, Weak};
use std::path::Path;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono;
use nanoid::nanoid;
use tracing::info;

// 调试日志中文本预览的最大字符数
const DEBUG_TEXT_PREVIEW_LENGTH: usize = 50;
// HTML内容预览的最大字符数
const HTML_CONTENT_PREVIEW_LENGTH: usize = 500;

pub struct DocumentFolderOperation(pub Weak<DocumentManager>);

impl DocumentFolderOperation {
  fn document_manager(&self) -> Result<Arc<DocumentManager>, FlowyError> {
    self.0.upgrade().ok_or_else(FlowyError::ref_drop)
  }

  /// 安全地截取字符串预览，避免UTF-8字符边界问题
  fn safe_string_preview(&self, text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
      return text.to_string();
    }
    
    // 使用字符迭代器安全地截取
    let mut chars = text.chars();
    let mut result = String::new();
    let mut char_count = 0;
    
    while let Some(ch) = chars.next() {
      if char_count >= max_chars {
        result.push_str("...");
        break;
      }
      result.push(ch);
      char_count += 1;
    }
    
    result
  }

    /// 将转换结果转换为DocumentDataPB
    fn convert_result_to_document_data(&self, result: &flowy_document::import::ConversionResult) -> Result<DocumentDataPB, FlowyError> {
        use std::collections::HashMap;
        use nanoid::nanoid;
        use flowy_document::entities::{BlockPB, ChildrenPB, MetaPB};
        
        let page_id = nanoid!(10);
        let mut blocks = HashMap::new();
        let mut children_map = HashMap::new();
        let mut text_map = HashMap::new();
        
        // 处理提取的图片
        let mut image_map = HashMap::new();
        for image in &result.extracted_images {
            // 将图片数据转换为base64编码
            let base64_data = base64::encode(&image.data);
            let data_url = format!("data:image/{};base64,{}", 
                match image.format {
                    flowy_document::import::ImageFormat::Png => "png",
                    flowy_document::import::ImageFormat::Jpeg => "jpeg",
                    flowy_document::import::ImageFormat::Gif => "gif",
                    flowy_document::import::ImageFormat::Bmp => "bmp",
                    flowy_document::import::ImageFormat::WebP => "webp",
                },
                base64_data
            );
            
            // 使用图片ID作为键，这样HTML中的src属性就能正确映射到图片数据
            image_map.insert(image.id.clone(), data_url);
            tracing::info!("📸 Processed image: {} (ID: {}, {} bytes) -> base64 data URL", 
                image.filename, image.id, image.data.len());
        }
        
        // 创建根页面块
        let root_block = BlockPB {
            id: page_id.clone(),
            ty: "page".to_string(),
            data: serde_json::to_string(&serde_json::json!({
                "page_width": 842,
                "page_height": 595
            })).unwrap_or_default(),
            parent_id: "".to_string(),
            children_id: nanoid!(10),
            external_id: None,
            external_type: None,
        };
        
        let root_children_id = root_block.children_id.clone();
        blocks.insert(page_id.clone(), root_block);
        
        // 解析HTML内容并转换为AppFlowy块结构
        let html_content = &result.document_content.content;
        let mut child_blocks = Vec::new();
        
        // 使用scraper解析HTML
        let document = scraper::Html::parse_document(html_content);
        
        // 添加调试信息
        tracing::info!("HTML content length: {}", html_content.len());
        tracing::info!("HTML content preview: {}", 
            self.safe_string_preview(html_content, HTML_CONTENT_PREVIEW_LENGTH));
        
        // 尝试多种选择器来找到HTML元素
        let selectors = vec![
            "p, h1, h2, h3, h4, h5, h6, ul, ol, li, table, blockquote, div, img", // 直接选择常见元素，包括图片
            "body > *",           // 如果有body标签
            "html > body > *",     // 完整的HTML结构
            "*"                    // 最后选择所有元素
        ];
        
        let mut found_elements = false;
        for (i, selector_str) in selectors.iter().enumerate() {
            if let Ok(selector) = scraper::Selector::parse(selector_str) {
                let elements: Vec<_> = document.select(&selector).collect();
                tracing::info!("Selector {} ({}): found {} elements", i, selector_str, elements.len());
                
                if !elements.is_empty() {
                    found_elements = true;
                    for (j, element) in elements.iter().enumerate() {
                        let tag_name = element.value().name();
                        let text_preview = element.text().collect::<String>();
                        let text_trimmed = text_preview.trim();
                        let text_display = self.safe_string_preview(text_trimmed, DEBUG_TEXT_PREVIEW_LENGTH);
                        tracing::info!("Element {}: <{}> '{}'", j, tag_name, text_display);
                        
                        if let Some(block) = self.html_element_to_block(*element, &page_id, &mut text_map, &mut children_map, &mut blocks, &image_map) {
                            tracing::info!("Created block: type='{}', id='{}'", block.ty, block.id);
                            child_blocks.push(block.id.clone());
                            blocks.insert(block.id.clone(), block);
                        } else {
                            tracing::warn!("Failed to convert element <{}> to block", tag_name);
                        }
                    }
                    break; // 找到元素后停止尝试其他选择器
                }
            }
        }
        
        // 如果没有找到任何块，创建一个默认的段落块
        if child_blocks.is_empty() {
            tracing::warn!("No HTML elements found, creating fallback paragraph block");
            let content_block_id = nanoid!(10);
            let content_text_id = nanoid!(10);
            let content_block = BlockPB {
                id: content_block_id.clone(),
                ty: "paragraph".to_string(),
                data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
                parent_id: page_id.clone(),
                children_id: nanoid!(10),
                external_id: Some(content_text_id.clone()),
                external_type: Some("text".to_string()),
            };
            
            blocks.insert(content_block_id.clone(), content_block);
            child_blocks.push(content_block_id.clone());
            
            // 设置内容文本 - 将HTML转换为纯文本
            let plain_text = self.html_to_plain_text(html_content);
            text_map.insert(content_text_id, serde_json::to_string(&serde_json::json!([
                {
                    "insert": plain_text
                }
            ])).unwrap_or_default());
        }
        
        tracing::info!("Created {} blocks from HTML content", child_blocks.len());
        
        // 设置根页面的子块
        children_map.insert(root_children_id.clone(), ChildrenPB {
            children: child_blocks
        });
        
        // 为每个块创建children_map条目
        for block in blocks.values() {
            if !block.children_id.is_empty() {
                children_map.entry(block.children_id.clone()).or_insert_with(|| ChildrenPB {
                    children: vec![]
                });
            }
        }
        
        Ok(DocumentDataPB {
            page_id,
            blocks,
            meta: MetaPB {
                children_map,
                text_map,
            },
        })
    }
  
  /// 将HTML元素转换为AppFlowy块
  fn html_element_to_block(&self, element: scraper::ElementRef, parent_id: &str, text_map: &mut HashMap<String, String>, children_map: &mut HashMap<String, ChildrenPB>, blocks: &mut HashMap<String, BlockPB>, image_map: &HashMap<String, String>) -> Option<BlockPB> {
    use nanoid::nanoid;
    use flowy_document::entities::BlockPB;
    
    let tag_name = element.value().name();
    let block_id = nanoid!(10);
    let children_id = nanoid!(10);
    
    tracing::debug!("Converting HTML element: {} with text: '{}'", tag_name, element.text().collect::<String>().trim());
    
    match tag_name {
      "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
        let level = tag_name.chars().last().unwrap_or('1').to_digit(10).unwrap_or(1) as i32;
        let text_id = nanoid!(10);
        
        let block = BlockPB {
          id: block_id,
          ty: "heading".to_string(),
          data: serde_json::to_string(&serde_json::json!({
            "level": level
          })).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: Some(text_id.clone()),
          external_type: Some("text".to_string()),
        };
        
        // 提取文本内容
        let text_content = element.text().collect::<String>();
        text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
          {
            "insert": text_content
          }
        ])).unwrap_or_default());
        
        tracing::debug!("Created heading block: level={}, text='{}'", level, text_content.trim());
        Some(block)
      },
      "p" => {
        let text_id = nanoid!(10);
        
        let block = BlockPB {
          id: block_id,
          ty: "paragraph".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: Some(text_id.clone()),
          external_type: Some("text".to_string()),
        };
        
        // 提取文本内容
        let text_content = element.text().collect::<String>();
        text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
          {
            "insert": text_content
          }
        ])).unwrap_or_default());
        
        tracing::debug!("Created paragraph block: text='{}'", text_content.trim());
        Some(block)
      },
      "ul" => {
        let block = BlockPB {
          id: block_id,
          ty: "bulleted_list".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: None,
          external_type: None,
        };
        
        tracing::debug!("Created bulleted list block");
        Some(block)
      },
      "ol" => {
        let block = BlockPB {
          id: block_id,
          ty: "numbered_list".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: None,
          external_type: None,
        };
        
        tracing::debug!("Created numbered list block");
        Some(block)
      },
      "li" => {
        let text_id = nanoid!(10);
        
        let block = BlockPB {
          id: block_id,
          ty: "paragraph".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: Some(text_id.clone()),
          external_type: Some("text".to_string()),
        };
        
        // 提取文本内容
        let text_content = element.text().collect::<String>();
        text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
          {
            "insert": text_content
          }
        ])).unwrap_or_default());
        
        tracing::debug!("Created list item block: text='{}'", text_content.trim());
        Some(block)
      },
      "blockquote" => {
        let text_id = nanoid!(10);
        
        let block = BlockPB {
          id: block_id,
          ty: "quote".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: Some(text_id.clone()),
          external_type: Some("text".to_string()),
        };
        
        // 提取文本内容
        let text_content = element.text().collect::<String>();
        text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
          {
            "insert": text_content
          }
        ])).unwrap_or_default());
        
        tracing::debug!("Created quote block: text='{}'", text_content.trim());
        Some(block)
      },
      "table" => {
        tracing::info!("🔍 Processing table element - NEW IMPLEMENTATION");
        // 解析表格结构并创建AppFlowy表格块
        let mut table_rows = Vec::new();
        let mut max_cols = 0;
        
        // 解析表格行
        for row_element in element.select(&scraper::Selector::parse("tr").unwrap()) {
          let mut cells = Vec::new();
          
          // 解析表格单元格
          for cell_element in row_element.select(&scraper::Selector::parse("td, th").unwrap()) {
            let cell_text = cell_element.text().collect::<String>().trim().to_string();
            cells.push(cell_text);
          }
          
          if !cells.is_empty() {
            max_cols = max_cols.max(cells.len());
            table_rows.push(cells);
          }
        }
        
        // 确保所有行都有相同的列数
        for row in &mut table_rows {
          while row.len() < max_cols {
            row.push(String::new());
          }
        }
        
        // 创建表格行和单元格的children
        let mut row_children = Vec::new();
        for (row_idx, row_cells) in table_rows.iter().enumerate() {
          let row_id = nanoid!(10);
          let mut cell_children = Vec::new();
          
          // 为每个单元格创建simple_table_cell块
          for (col_idx, cell_text) in row_cells.iter().enumerate() {
            let cell_id = nanoid!(10);
            let cell_text_id = nanoid!(10);
            
            let cell_children_id = nanoid!(10);
            let cell_block = BlockPB {
              id: cell_id.clone(),
              ty: "simple_table_cell".to_string(),
              data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
              parent_id: row_id.clone(),
              children_id: cell_children_id.clone(),
              external_id: Some(cell_text_id.clone()),
              external_type: Some("text".to_string()),
            };
            
            // 为单元格创建children_map条目（包含段落块）
            let paragraph_id = nanoid!(10);
            let paragraph_text_id = nanoid!(10);
            let paragraph_block = BlockPB {
              id: paragraph_id.clone(),
              ty: "paragraph".to_string(),
              data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
              parent_id: cell_id.clone(),
              children_id: nanoid!(10),
              external_id: Some(paragraph_text_id.clone()),
              external_type: Some("text".to_string()),
            };
            
            // 设置段落文本内容
            text_map.insert(paragraph_text_id, serde_json::to_string(&serde_json::json!([
              {
                "insert": cell_text
              }
            ])).unwrap_or_default());
            
            // 设置单元格的children（包含段落块）
            children_map.insert(cell_children_id, ChildrenPB {
              children: vec![paragraph_id.clone()]
            });
            
            blocks.insert(paragraph_id, paragraph_block);
            
            cell_children.push(cell_id.clone());
            blocks.insert(cell_id, cell_block);
          }
          
          // 创建表格行块
          let row_block = BlockPB {
            id: row_id.clone(),
            ty: "simple_table_row".to_string(),
            data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
            parent_id: block_id.clone(),
            children_id: nanoid!(10),
            external_id: None,
            external_type: None,
          };
          
          row_children.push(row_id.clone());
          
          // 设置行的children
          let row_children_id = row_block.children_id.clone();
          children_map.insert(row_children_id, ChildrenPB {
            children: cell_children
          });
          
          blocks.insert(row_id, row_block);
        }
        
        // 创建表格块
        let table_children_id = children_id.clone();
        let block = BlockPB {
          id: block_id,
          ty: "simple_table".to_string(),
          data: serde_json::to_string(&serde_json::json!({
            "enable_header_row": false,
            "enable_header_column": false,
            "column_colors": {},
            "row_colors": {},
            "column_aligns": {},
            "row_aligns": {},
            "column_bold_attributes": {},
            "row_bold_attributes": {},
            "column_text_colors": {},
            "row_text_colors": {}
          })).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: None,
          external_type: None,
        };
        
        // 设置表格的children（包含所有行）
        children_map.insert(table_children_id, ChildrenPB {
          children: row_children
        });
        
        tracing::info!("✅ Created table block with {} rows and {} columns", table_rows.len(), max_cols);
        tracing::debug!("Created table block: {}x{} table with {} rows", max_cols, table_rows.len(), table_rows.len());
        Some(block)
      },
      "img" => {
        // 处理图片元素
        let src = element.value().attr("src").unwrap_or("");
        let alt = element.value().attr("alt").unwrap_or("Image");
        
        tracing::info!("🖼️ Processing image element: src='{}', alt='{}'", src, alt);
        
        // 尝试从图片映射中找到对应的base64数据
        let image_url = if let Some(data_url) = image_map.get(src) {
          data_url.clone()
        } else {
          // 如果没有找到，使用原始src
          src.to_string()
        };
        
        // 创建图片块
        let block = BlockPB {
          id: block_id,
          ty: "image".to_string(),
          data: serde_json::to_string(&serde_json::json!({
            "url": image_url,
            "width": 300,
            "height": 200,
            "align": "center",
            "image_type": 2  // CustomImageType.external for base64 data URLs
          })).unwrap_or_default(),
          parent_id: parent_id.to_string(),
          children_id,
          external_id: None,
          external_type: None,
        };
        
        // 截断base64 URL用于日志显示，避免日志过长
        let display_url = if image_url.starts_with("data:") {
            format!("{}...", &image_url[..50])
        } else {
            image_url.clone()
        };
        tracing::info!("✅ Created image block: src='{}' -> url='{}'", src, display_url);
        Some(block)
      },
      "div" | "span" => {
        // 对于div和span，检查是否有文本内容
        let text_content = element.text().collect::<String>();
        if !text_content.trim().is_empty() {
          let text_id = nanoid!(10);
          
          let block = BlockPB {
            id: block_id,
            ty: "paragraph".to_string(),
            data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
            parent_id: parent_id.to_string(),
            children_id,
            external_id: Some(text_id.clone()),
            external_type: Some("text".to_string()),
          };
          
          text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
            {
              "insert": text_content
            }
          ])).unwrap_or_default());
          
          tracing::debug!("Created div/span block: text='{}'", text_content.trim());
          Some(block)
        } else {
          tracing::debug!("Skipping empty div/span element");
          None
        }
      },
      _ => {
        // 对于其他元素，尝试提取文本内容
        let text_content = element.text().collect::<String>();
        if !text_content.trim().is_empty() {
          let text_id = nanoid!(10);
          
          let block = BlockPB {
            id: block_id,
            ty: "paragraph".to_string(),
            data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
            parent_id: parent_id.to_string(),
            children_id,
            external_id: Some(text_id.clone()),
            external_type: Some("text".to_string()),
          };
          
          text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
            {
              "insert": text_content
            }
          ])).unwrap_or_default());
          
          tracing::debug!("Created generic block for {}: text='{}'", tag_name, text_content.trim());
          Some(block)
        } else {
          tracing::debug!("Skipping empty {} element", tag_name);
          None
        }
      }
    }
  }
  
  /// 将HTML转换为纯文本
  fn html_to_plain_text(&self, html: &str) -> String {
    // 简单的HTML标签移除
    html
      .replace("<br>", "\n")
      .replace("<br/>", "\n")
      .replace("<br />", "\n")
      .replace("<p>", "")
      .replace("</p>", "\n\n")
      .replace("<div>", "")
      .replace("</div>", "\n")
      .replace("<h1>", "")
      .replace("</h1>", "\n\n")
      .replace("<h2>", "")
      .replace("</h2>", "\n\n")
      .replace("<h3>", "")
      .replace("</h3>", "\n\n")
      .replace("<strong>", "")
      .replace("</strong>", "")
      .replace("<b>", "")
      .replace("</b>", "")
      .replace("<em>", "")
      .replace("</em>", "")
      .replace("<i>", "")
      .replace("</i>", "")
      .lines()
      .map(|line| line.trim())
      .filter(|line| !line.is_empty())
      .collect::<Vec<_>>()
      .join("\n")
  }

  /// 使用转换器导入文档
  async fn import_document_with_converter(
    &self,
    uid: i64,
    view_id: &Uuid,
    import_type: ImportType,
    source_path: String,
    bytes: Vec<u8>,
  ) -> Result<Vec<ImportedData>, FlowyError> {
    // 确定文档类型
    let document_type = match import_type {
      ImportType::Word => DocumentType::Word,
      ImportType::Pdf => DocumentType::Pdf,
      _ => return Err(FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported import type")),
    };

    // 创建转换器工厂
    let converter_factory = Arc::new(DefaultConverterFactory);
    
    // 创建转换器
    let converter = converter_factory
      .create_converter(document_type.clone())
      .ok_or_else(|| FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported document type"))?;

    // 创建转换任务
    let task = ConversionTask::new_with_bytes(
      source_path.clone(),
      document_type,
      source_path,
      Default::default(),
      bytes,
    );

    // 执行转换
    let result = converter.convert(&task).await?;

    // 将转换结果转换为 DocumentDataPB
    let document_data = self.convert_result_to_document_data(&result)?;

    // 创建文档
    let encoded_collab = self
      .document_manager()?
      .create_document(uid, view_id, Some(document_data.into()))
      .await?;

    Ok(vec![(
      view_id.to_string(),
      CollabType::Document,
      encoded_collab,
    )])
  }
}
#[async_trait]
impl FolderOperationHandler for DocumentFolderOperation {
  fn name(&self) -> &str {
    "DocumentFolderOperationHandler"
  }

  async fn create_workspace_view(
    &self,
    uid: i64,
    workspace_view_builder: Arc<RwLock<NestedViewBuilder>>,
  ) -> Result<(), FlowyError> {
    let manager = self.document_manager()?;

    let mut write_guard = workspace_view_builder.write().await;
    // Create a view named "Getting started" with an icon ⭐️ and the built-in README data.
    // Don't modify this code unless you know what you are doing.
    write_guard
      .with_view_builder(|view_builder| async {
        let view = view_builder
          .with_name("Getting started")
          .with_icon("⭐️")
          .build();
        // create a empty document
        let json_str = include_str!("../../../assets/read_me.json");
        let document_pb = JsonToDocumentParser::json_str_to_document(json_str).unwrap();
        let view_id = Uuid::from_str(&view.view.id).unwrap();
        manager
          .create_document(uid, &view_id, Some(document_pb.into()))
          .await
          .unwrap();
        view
      })
      .await;
    Ok(())
  }

  async fn open_view(&self, view_id: &Uuid) -> Result<(), FlowyError> {
    self.document_manager()?.open_document(view_id).await?;
    Ok(())
  }

  /// Close the document view.
  async fn close_view(&self, view_id: &Uuid) -> Result<(), FlowyError> {
    self.document_manager()?.close_document(view_id).await?;
    Ok(())
  }

  async fn delete_view(&self, view_id: &Uuid) -> Result<(), FlowyError> {
    let document_manager = self.document_manager()?;
    let workspace_id = document_manager.user_service.workspace_id()?;
    if let Some(state) = get_document_tantivy_state(&workspace_id).and_then(|v| v.upgrade()) {
      let _ = state.write().await.delete_document(&view_id.to_string());
    }

    match document_manager.delete_document(view_id).await {
      Ok(_) => tracing::trace!("Delete document: {}", view_id),
      Err(e) => tracing::error!("🔴delete document failed: {}", e),
    }
    Ok(())
  }

  async fn duplicate_view(&self, view_id: &Uuid) -> Result<Bytes, FlowyError> {
    let data: DocumentDataPB = self
      .document_manager()?
      .get_document_data(view_id)
      .await?
      .into();
    let data_bytes = data.into_bytes().map_err(|_| FlowyError::invalid_data())?;
    Ok(data_bytes)
  }

  async fn gather_publish_encode_collab(
    &self,
    user: &Arc<dyn FolderUser>,
    view_id: &Uuid,
  ) -> Result<GatherEncodedCollab, FlowyError> {
    let encoded_collab =
      get_encoded_collab_v1_from_disk(user, view_id.to_string().as_str(), CollabType::Document)
        .await?;
    Ok(GatherEncodedCollab::Document(encoded_collab))
  }

  async fn create_view_with_view_data(
    &self,
    user_id: i64,
    params: CreateViewParams,
  ) -> Result<Option<EncodedCollab>, FlowyError> {
    debug_assert_eq!(params.layout, ViewLayoutPB::Document);
    let data = match params.initial_data {
      ViewData::DuplicateData(data) => Some(DocumentDataPB::try_from(data)?),
      ViewData::Data(data) => Some(DocumentDataPB::try_from(data)?),
      ViewData::Empty => None,
    };
    let encoded_collab = self
      .document_manager()?
      .create_document(user_id, &params.view_id, data.map(|d| d.into()))
      .await?;
    Ok(Some(encoded_collab))
  }

  /// Create a view with built-in data.
  async fn create_default_view(
    &self,
    user_id: i64,
    _parent_view_id: &Uuid,
    view_id: &Uuid,
    name: &str,
    layout: ViewLayout,
  ) -> Result<(), FlowyError> {
    debug_assert_eq!(layout, ViewLayout::Document);
    
    // 检查文档是否已经存在（比如通过导入创建）
    // 尝试获取文档数据，如果成功说明文档已存在，跳过创建
    info!("Checking if document {} already exists", view_id);
    match self.document_manager()?.get_document_data(view_id).await {
      Ok(_) => {
        info!("Document {} already exists, skipping creation", view_id);
        return Ok(());
      },
      Err(err) => {
        // 如果是 RecordNotFound 错误，说明文档不存在，需要创建
        // 其他错误可能是网络或权限问题，但不应该阻止创建
        if err.is_record_not_found() {
          info!("Document {} does not exist, will create it", view_id);
        } else {
          info!("Document {} check failed with error: {}, will try to create anyway", view_id, err);
        }
      }
    }
    
    // 文档不存在，创建空白文档
    // 即使名称为空也应该创建，因为用户可能还没命名（比如通过"新页面"按钮创建的根节点文章）
    info!("Creating default document for view {}", view_id);
    match self
      .document_manager()?
      .create_document(user_id, view_id, None)
      .await
    {
      Ok(_) => {
        info!("Successfully created default document for view {}", view_id);
        Ok(())
      },
      Err(err) => {
        if err.is_already_exists() {
          // 文档在创建过程中被其他地方创建了，这不算错误
          info!("Document {} already exists (created concurrently)", view_id);
          Ok(())
        } else {
          Err(err)
        }
      },
    }
  }

  async fn import_from_bytes(
    &self,
    uid: i64,
    view_id: &Uuid,
    _name: &str,
    import_type: ImportType,
    bytes: Vec<u8>,
  ) -> Result<Vec<ImportedData>, FlowyError> {
    match import_type {
      ImportType::Word | ImportType::Pdf => {
        // 直接处理文件内容，不创建新文档
        let document_type = match import_type {
          ImportType::Word => DocumentType::Word,
          ImportType::Pdf => DocumentType::Pdf,
          _ => return Err(FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported import type")),
        };

        info!("Creating converter for document type: {:?}", document_type);
        let converter_factory = Arc::new(DefaultConverterFactory);
        let converter = converter_factory
          .create_converter(document_type.clone())
          .ok_or_else(|| FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported document type"))?;

        info!("Converter created successfully: {}", converter.name());
        let temp_path = format!("temp_import_{}", view_id);
        let task = ConversionTask::new_with_bytes(
          temp_path.clone(),
          document_type,
          temp_path,
          Default::default(),
          bytes,
        );
        info!("Starting conversion with task: {:?}", task);

        let result = converter.convert(&task).await?;
        info!("PDF conversion completed, result: {:?}", result);
        let document_data = self.convert_result_to_document_data(&result)?;
        info!("Document data created successfully");

        // 更新现有文档而不是创建新文档
        let encoded_collab = self
          .document_manager()?
          .create_document(uid, view_id, Some(document_data.into()))
          .await?;

        Ok(vec![(
          view_id.to_string(),
          CollabType::Document,
          encoded_collab,
        )])
      },
      _ => {
        // 保持原有的处理逻辑
        let data = DocumentDataPB::try_from(Bytes::from(bytes))?;
        let encoded_collab = self
          .document_manager()?
          .create_document(uid, view_id, Some(data.into()))
          .await?;
        Ok(vec![(
          view_id.to_string(),
          CollabType::Document,
          encoded_collab,
        )])
      }
    }
  }

  async fn import_from_file_path(
    &self,
    view_id: &str,
    _name: &str,
    path: String,
  ) -> Result<(), FlowyError> {
    let file_path = Path::new(&path);
    
    // 检查文件扩展名以确定导入类型
    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
      match ext.to_lowercase().as_str() {
        "docx" | "doc" | "pdf" => {
          // 对于Word和PDF文件，读取文件内容并调用import_from_bytes
          info!("Reading {} file content for conversion", ext);
          
          let bytes = tokio::fs::read(&file_path).await
            .map_err(|e| FlowyError::new(
              flowy_error::ErrorCode::Internal,
              format!("Failed to read file: {}", e),
            ))?;
          
          let import_type = match ext.to_lowercase().as_str() {
            "docx" | "doc" => ImportType::Word,
            "pdf" => ImportType::Pdf,
            _ => return Err(FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported file type")),
          };
          
          let view_id_uuid = Uuid::from_str(view_id)
            .map_err(|e| FlowyError::new(flowy_error::ErrorCode::InvalidParams, format!("Invalid view ID: {}", e)))?;
          
          // 直接处理文件内容，不创建文档，让create_view_with_params统一处理
          let document_type = match import_type {
            ImportType::Word => DocumentType::Word,
            ImportType::Pdf => DocumentType::Pdf,
            _ => return Err(FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported import type")),
          };

          let converter_factory = Arc::new(DefaultConverterFactory);
          let converter = converter_factory
            .create_converter(document_type.clone())
            .ok_or_else(|| FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported document type"))?;

          let temp_path = format!("temp_import_{}", view_id_uuid);
          let task = ConversionTask::new_with_bytes(
            temp_path.clone(),
            document_type,
            temp_path,
            Default::default(),
            bytes,
          );

          let result = converter.convert(&task).await?;
          let document_data = self.convert_result_to_document_data(&result)?;

          // 在这里创建文档，create_view_with_params会检查文档是否已存在，避免重复创建
          let _ = self
            .document_manager()?
            .create_document(0, &view_id_uuid, Some(document_data.into()))
            .await?;
          
          info!("Successfully processed {} file", ext);
          return Ok(());
        },
        _ => {
          // TODO(lucas): import file from local markdown file
          // 保持原有的处理逻辑
        }
      }
    }
    
    Ok(())
  }
}
