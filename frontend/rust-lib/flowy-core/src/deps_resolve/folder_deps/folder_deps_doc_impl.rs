use crate::deps_resolve::folder_deps::get_encoded_collab_v1_from_disk;
use bytes::Bytes;
use collab::entity::EncodedCollab;
use collab_entity::CollabType;
use collab_folder::hierarchy_builder::NestedViewBuilder;
use collab_folder::ViewLayout;
use flowy_document::entities::{DocumentDataPB, BlockPB, ChildrenPB};
use flowy_document::manager::DocumentManager;
use flowy_document::parser::json::parser::JsonToDocumentParser;
use flowy_document::import::{DefaultConverterFactory, ConverterFactory, ConversionTask, DocumentType};
use collab_document::blocks::DocumentData;
use flowy_folder::manager::{ImportProgress, FolderManager};
use flowy_error::{FlowyError, FlowyResult};
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
use nanoid::nanoid;
use tracing::info;

// 调试日志中文本预览的最大字符数
const DEBUG_TEXT_PREVIEW_LENGTH: usize = 50;
// HTML内容预览的最大字符数
const HTML_CONTENT_PREVIEW_LENGTH: usize = 500;

/// 合并后的文本信息
#[derive(Debug, Clone)]
struct MergedTextInfo {
  text: String,
  is_heading: bool,
  heading_level: i32, // 1-6 for h1-h6, 0 for paragraph
  is_list_item: bool, // 是否为列表项
  top: f64, // 位置信息，用于图片插入
}

pub struct DocumentFolderOperation {
  document_manager: Weak<DocumentManager>,
  folder_manager: Option<Weak<FolderManager>>,
}

impl DocumentFolderOperation {
  pub fn new(document_manager: Weak<DocumentManager>) -> Self {
    Self {
      document_manager,
      folder_manager: None,
    }
  }

  pub fn with_folder_manager(
    document_manager: Weak<DocumentManager>,
    folder_manager: Weak<FolderManager>,
  ) -> Self {
    Self {
      document_manager,
      folder_manager: Some(folder_manager),
    }
  }

  fn document_manager(&self) -> Result<Arc<DocumentManager>, FlowyError> {
    self.document_manager.upgrade().ok_or_else(FlowyError::ref_drop)
  }

  fn send_progress(&self, import_id: &str, file_name: &str, progress: f64, step: &str) {
    if let Some(weak_folder) = &self.folder_manager {
      if let Some(folder_manager) = weak_folder.upgrade() {
        folder_manager.send_import_progress(ImportProgress::new(
          import_id.to_string(),
          file_name.to_string(),
          progress,
          step.to_string(),
        ));
      }
    }
  }

  fn send_progress_error(&self, import_id: &str, file_name: &str, error: &str) {
    if let Some(weak_folder) = &self.folder_manager {
      if let Some(folder_manager) = weak_folder.upgrade() {
        folder_manager.send_import_progress(ImportProgress::new_error(
          import_id.to_string(),
          file_name.to_string(),
          error.to_string(),
        ));
      }
    }
  }

  fn add_import_log(&self, import_id: &str, level: &str, message: &str) {
    if let Some(weak_folder) = &self.folder_manager {
      if let Some(folder_manager) = weak_folder.upgrade() {
        use flowy_folder::manager::ImportLogEntry;
        let log = match level {
          "error" => ImportLogEntry::error(message.to_string()),
          "warn" => ImportLogEntry::warn(message.to_string()),
          "debug" => ImportLogEntry::debug(message.to_string()),
          _ => ImportLogEntry::info(message.to_string()),
        };
        folder_manager.add_import_log(import_id, log);
      }
    }
  }
}

impl DocumentFolderOperation {
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
        self.convert_result_to_document_data_with_progress(result, "", "")
    }

    fn convert_result_to_document_data_with_progress(
        &self,
        result: &flowy_document::import::ConversionResult,
        import_id: &str,
        file_name: &str,
    ) -> Result<DocumentDataPB, FlowyError> {
        use std::collections::HashMap;
        use nanoid::nanoid;
        use flowy_document::entities::{BlockPB, ChildrenPB, MetaPB};
        
        let page_id = nanoid!(10);
        let mut blocks = HashMap::new();
        let mut children_map = HashMap::new();
        let mut text_map = HashMap::new();
        
        // 处理提取的图片
        let mut image_map = HashMap::new();
        if !result.extracted_images.is_empty() {
            self.add_import_log(import_id, "info", &format!("开始处理 {} 张图片", result.extracted_images.len()));
        }
        
        for (idx, image) in result.extracted_images.iter().enumerate() {
            // 将图片数据转换为base64编码
            use base64::{Engine as _, engine::general_purpose};
            let base64_data = general_purpose::STANDARD.encode(&image.data);
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
            
            // 发送图片处理日志
            let image_size_mb = image.data.len() as f64 / (1024.0 * 1024.0);
            self.add_import_log(import_id, "info", &format!(
                "处理图片 {}/{}: {} ({} bytes, {:.2} MB, 格式: {:?})",
                idx + 1,
                result.extracted_images.len(),
                image.filename,
                image.data.len(),
                image_size_mb,
                image.format
            ));
        }
        
        if !result.extracted_images.is_empty() {
            self.add_import_log(import_id, "info", &format!("图片处理完成: {} 张图片已转换为 base64 格式", result.extracted_images.len()));
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
        
        // 解析内容并转换为AppFlowy块结构
        // 内容可能是HTML或Markdown格式
        let content = &result.document_content.content;
        let mut child_blocks = Vec::new();
        
        // 检测内容类型：Markdown还是HTML
        let is_markdown = self.detect_markdown_format(content);
        
        if is_markdown {
            tracing::info!("Detected Markdown format, parsing Markdown to AppFlowy blocks");
            tracing::info!("Markdown content length: {}", content.len());
            tracing::info!("Markdown content preview: {}", 
                self.safe_string_preview(content, HTML_CONTENT_PREVIEW_LENGTH));
            
            self.add_import_log(import_id, "info", "检测到 Markdown 格式，开始解析 Markdown 并转换为 AppFlowy 文档格式");
            self.add_import_log(import_id, "info", &format!("Markdown 内容长度: {} 字符", content.len()));
            
            // 解析Markdown并转换为AppFlowy块
            self.add_import_log(import_id, "info", "开始解析 Markdown 语法（标题、段落、列表、表格等）");
            let (parsed_blocks, inserted_image_ids) = self.parse_markdown_to_blocks(content, &result.extracted_images, &page_id, 
                &mut blocks, &mut text_map, &mut children_map)?;
            let parsed_blocks_count = parsed_blocks.len();
            child_blocks = parsed_blocks;
            
            self.add_import_log(import_id, "info", &format!("Markdown 解析完成: 创建了 {} 个文档块", parsed_blocks_count));
            self.add_import_log(import_id, "info", &format!("在 Markdown 中插入了 {} 张图片", inserted_image_ids.len()));
            
            // 跟踪已在Markdown中插入的图片ID
            let mut inserted_image_set = std::collections::HashSet::new();
            for image_id in inserted_image_ids {
                inserted_image_set.insert(image_id);
            }
            
            // 只追加那些没有在Markdown中插入的图片
            let mut images_to_append = Vec::new();
            for (idx, image) in result.extracted_images.iter().enumerate() {
                if !inserted_image_set.contains(&image.id) {
                    images_to_append.push((idx, image));
                }
            }
            
            if !images_to_append.is_empty() {
                let images_to_append_count = images_to_append.len();
                tracing::info!("Appending {} images that were not inserted in Markdown", images_to_append_count);
                self.add_import_log(import_id, "info", &format!("追加 {} 张未在 Markdown 中插入的图片", images_to_append_count));
                for (idx, image) in images_to_append {
                    let image_number = idx + 1;
                    
                    // 1. 先添加【图片 N】文本标签
                    let label_block_id = nanoid!(10);
                    let label_text_id = nanoid!(10);
                    let label_block = BlockPB {
                        id: label_block_id.clone(),
                        ty: "paragraph".to_string(),
                        data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
                        parent_id: page_id.to_string(),
                        children_id: nanoid!(10),
                        external_id: Some(label_text_id.clone()),
                        external_type: Some("text".to_string()),
                    };
                    
                    text_map.insert(label_text_id, serde_json::to_string(&serde_json::json!([
                        {
                            "insert": format!("【图片 {}】", image_number)
                        }
                    ])).unwrap_or_default());
                    
                    child_blocks.push(label_block_id.clone());
                    blocks.insert(label_block_id, label_block);
                    
                    // 2. 添加图片块
                    let image_id = nanoid!(10);
                    use base64::{Engine as _, engine::general_purpose};
                    let base64_data = general_purpose::STANDARD.encode(&image.data);
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
                    
                    // 使用图片的实际尺寸，如果没有尺寸信息则尝试从图片数据中读取
                    // 使用轻量级方法只读取图片头部信息，避免加载整个图片导致栈溢出
                    let (width, height) = if let Some(dimensions) = &image.dimensions {
                        (dimensions.width, dimensions.height)
                    } else {
                        // 尝试从图片数据中读取尺寸（只读取头部，不加载整个图片）
                        match Self::get_image_dimensions_lightweight(&image.data, &image.format) {
                            Some((w, h)) => (w, h),
                            None => {
                                tracing::warn!("无法从图片数据读取尺寸，使用默认值 300x200");
                                (300, 200)
                            }
                        }
                    };
                    
                    let image_block = BlockPB {
                        id: image_id.clone(),
                        ty: "image".to_string(),
                        data: serde_json::to_string(&serde_json::json!({
                            "url": data_url,
                            "width": width,
                            "height": height,
                            "align": "center",
                            "image_type": 2
                        })).unwrap_or_default(),
                        parent_id: page_id.to_string(),
                        children_id: nanoid!(10),
                        external_id: None,
                        external_type: None,
                    };
                    
                    child_blocks.push(image_id.clone());
                    blocks.insert(image_id, image_block);
                    
                    tracing::info!("✅ Appended image {}: {} ({} bytes)", image_number, image.filename, image.data.len());
                    self.add_import_log(import_id, "info", &format!("追加图片 {}: {} ({} bytes)", image_number, image.filename, image.data.len()));
                }
                self.add_import_log(import_id, "info", &format!("图片追加完成: {} 张图片已添加到文档末尾", images_to_append_count));
            }
        } else {
            tracing::info!("Detected HTML format, parsing HTML to AppFlowy blocks");
            
            // 使用scraper解析HTML
            let document = scraper::Html::parse_document(content);
            
            // 添加调试信息
            tracing::info!("HTML content length: {}", content.len());
            tracing::info!("HTML content preview: {}", 
                self.safe_string_preview(content, HTML_CONTENT_PREVIEW_LENGTH));
        
        // 使用单一通用选择器按文档顺序收集所有元素
        // 这样可以保持图文混排的原始顺序
        // 注意：需要排除包装div（如pdf-content），避免包含所有子元素导致重复
        let selector_str = "p, h1, h2, h3, h4, h5, h6, ul, ol, li, table, blockquote, img";
        let selector = scraper::Selector::parse(selector_str)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to parse selector: {}", e)
            ))?;
        
        // 按文档顺序收集所有元素（scraper 的 select 方法会按文档顺序返回）
        let all_elements: Vec<scraper::ElementRef> = document.select(&selector).collect();
        tracing::info!("Selected {} elements using selector: {}", all_elements.len(), selector_str);
        
        // 直接使用所有选择的元素，不再进行去重检查
        // 因为目前只有一种导入功能，不存在重复导入的情况
        let all_selected_elements: Vec<scraper::ElementRef> = all_elements;
        
        tracing::info!("Using all {} elements directly (no deduplication)", all_selected_elements.len());
        
        // 统计收集到的元素类型
        let text_elements_count = all_selected_elements.iter().filter(|e| {
            let name = e.value().name();
            name != "img" && name != "table" && name != "div"
        }).count();
        let image_elements_count = all_selected_elements.iter().filter(|e| e.value().name() == "img").count();
        let table_elements_count = all_selected_elements.iter().filter(|e| e.value().name() == "table").count();
        
        // 统计文本元素的标签类型
        let mut tag_counts = std::collections::HashMap::new();
        for e in &all_selected_elements {
            let tag = e.value().name();
            *tag_counts.entry(tag).or_insert(0) += 1;
        }
        
        tracing::info!("Total elements collected: {} (text: {}, images: {}, tables: {})", 
            all_selected_elements.len(), text_elements_count, image_elements_count, table_elements_count);
        tracing::info!("Element tag breakdown: {:?}", tag_counts);
        
        // 处理所有元素
        tracing::info!("Processing {} elements to convert to blocks", all_selected_elements.len());
        
        // 检测是否有绝对定位的元素（pdftohtml生成的HTML通常使用绝对定位）
        let has_absolute_positioning = all_selected_elements.iter().any(|e| {
            if let Some(style) = e.value().attr("style") {
                style.contains("position:absolute") || style.contains("position: absolute")
            } else {
                false
            }
        });
        
        // 如果检测到绝对定位，合并文本片段
        let (mut merged_texts, skip_indices) = if has_absolute_positioning {
            tracing::info!("Detected absolute positioning in HTML, merging text fragments by position");
            // 按位置合并绝对定位的文本片段
            self.merge_absolute_positioned_elements_in_html(content, &all_selected_elements)
        } else {
            // 没有绝对定位，直接使用原始元素
            (Vec::new(), std::collections::HashSet::new())
        };
        
        // 去重：移除重复的文本（特别是"弹性算力"）
        let mut seen_texts: std::collections::HashSet<String> = std::collections::HashSet::new();
        merged_texts.retain(|info| {
            let text_key = info.text.trim().to_string();
            if seen_texts.contains(&text_key) {
                tracing::info!("Removing duplicate text: '{}'", text_key);
                false
            } else {
                seen_texts.insert(text_key);
                true
            }
        });
        
        // 收集图片元素的位置信息，用于后续插入
        let mut image_positions: Vec<(usize, f64)> = Vec::new(); // (index, top)
        let top_regex = regex::Regex::new(r"top:\s*(\d+(?:\.\d+)?)px").unwrap();
        for (idx, element) in all_selected_elements.iter().enumerate() {
          if element.value().name() == "img" {
            if let Some(style) = element.value().attr("style") {
              let top = top_regex.captures(style)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(0.0);
              image_positions.push((idx, top));
            }
          }
        }
        
        // 按位置排序合并后的文本和图片，以便按正确的顺序插入
        let total_elements = all_selected_elements.len() + merged_texts.len();
        
        let progress_update_interval = (total_elements / 20).max(1); // 每5%更新一次进度
        
        // 创建一个包含所有元素（合并文本和原始元素）的有序列表，按位置排序
        // (position, is_merged_or_image, merged_index_or_element_index)
        // is_merged_or_image: true = 合并文本, false = 普通元素, 特殊处理图片
        let mut ordered_items: Vec<(f64, i32, usize)> = Vec::new();
        // 使用i32来区分：0 = 普通元素, 1 = 合并文本, 2 = 图片
        
        // 添加合并后的文本
        for (idx, merged_text) in merged_texts.iter().enumerate() {
          ordered_items.push((merged_text.top, 1, idx));
        }
        
        // 添加原始元素（排除已合并的）
        // 根据图片在HTML文档中的顺序位置，使用前后文字元素的位置信息来推断图片位置
        
        // 首先，收集所有有位置信息的文字元素（包括合并文本和原始元素）
        // 用于后续查找图片前后的文字元素位置
        let mut text_positions_with_index: Vec<(usize, f64, bool)> = Vec::new(); // (index_in_all_elements, top, is_merged)
        
        // 添加合并文本的位置
        // 注意：合并文本的索引需要映射到它们在文档中的大概位置
        // 由于合并文本是从绝对定位的元素合并而来，它们的位置信息是准确的
        for (merged_idx, merged_text) in merged_texts.iter().enumerate() {
          // 使用一个虚拟索引来表示合并文本在文档中的位置
          // 这个索引应该大致对应它们在文档中的顺序
          text_positions_with_index.push((merged_idx + 10000, merged_text.top, true));
        }
        
        // 添加原始元素的位置（非图片、非跳过、有位置信息的元素）
        for (idx, element) in all_selected_elements.iter().enumerate() {
          if skip_indices.contains(&idx) {
            continue;
          }
          
          let tag_name = element.value().name();
          if tag_name == "img" {
            continue; // 跳过图片
          }
          
          // 获取元素的位置信息
          if let Some(style) = element.value().attr("style") {
            if let Some(captures) = top_regex.captures(style) {
              if let Some(m) = captures.get(1) {
                if let Ok(top) = m.as_str().parse::<f64>() {
                  text_positions_with_index.push((idx, top, false));
                }
              }
            }
          }
        }
        
        // 按位置排序，建立位置索引
        text_positions_with_index.sort_by(|a, b| {
          a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 为没有位置信息的图片推断位置
        // 基于图片在文档中的顺序位置，找到它前后最近的文字元素
        let mut image_position_map: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
        
        // 收集所有图片元素（包括有位置和无位置的）
        let mut all_image_indices: Vec<usize> = Vec::new();
        for (idx, element) in all_selected_elements.iter().enumerate() {
          if skip_indices.contains(&idx) {
            continue;
          }
          if element.value().name() == "img" {
            all_image_indices.push(idx);
          }
        }
        
        // 为每张图片推断位置
        for &img_idx in &all_image_indices {
          let element = &all_selected_elements[img_idx];
          
          // 检查图片是否已有位置信息
          let mut has_position = false;
          let mut existing_top = 0.0;
          if let Some(style) = element.value().attr("style") {
            if let Some(captures) = top_regex.captures(style) {
              if let Some(m) = captures.get(1) {
                if let Ok(top) = m.as_str().parse::<f64>() {
                  has_position = true;
                  existing_top = top;
                }
              }
            }
          }
          
          if !has_position {
            // 图片没有位置信息，需要根据上下文推断
            
            // 找到图片在文档中的位置（在所有元素中的索引）
            // 计算图片在所有非跳过元素中的顺序位置
            let mut img_position_in_doc = 0;
            for (idx, e) in all_selected_elements.iter().enumerate() {
              if skip_indices.contains(&idx) {
                continue;
              }
              if idx == img_idx {
                break;
              }
              if e.value().name() != "img" {
                img_position_in_doc += 1;
              }
            }
            
            // 找到图片前后最近的文字元素位置
            // 方法1：在all_selected_elements中直接查找前后元素
            let mut prev_text_pos: Option<f64> = None;
            let mut next_text_pos: Option<f64> = None;
            
            // 向前查找：在all_selected_elements中向前查找有位置信息的文字元素
            for i in (0..img_idx).rev() {
              if skip_indices.contains(&i) {
                continue;
              }
              let e = &all_selected_elements[i];
              if e.value().name() == "img" {
                continue; // 跳过其他图片
              }
              if let Some(style) = e.value().attr("style") {
                if let Some(captures) = top_regex.captures(style) {
                  if let Some(m) = captures.get(1) {
                    if let Ok(top) = m.as_str().parse::<f64>() {
                      prev_text_pos = Some(top);
                      break;
                    }
                  }
                }
              }
            }
            
            // 向后查找：在all_selected_elements中向后查找有位置信息的文字元素
            for i in (img_idx + 1)..all_selected_elements.len() {
              if skip_indices.contains(&i) {
                continue;
              }
              let e = &all_selected_elements[i];
              if e.value().name() == "img" {
                continue; // 跳过其他图片
              }
              if let Some(style) = e.value().attr("style") {
                if let Some(captures) = top_regex.captures(style) {
                  if let Some(m) = captures.get(1) {
                    if let Ok(top) = m.as_str().parse::<f64>() {
                      next_text_pos = Some(top);
                      break;
                    }
                  }
                }
              }
            }
            
            // 方法2：如果方法1没有找到，使用合并文本的位置信息
            // 根据图片在文档中的索引位置，估算它在合并文本序列中的大概位置
            if prev_text_pos.is_none() || next_text_pos.is_none() {
              // 计算图片在所有非跳过元素中的相对位置
              let mut elements_before_img = 0;
              for i in 0..img_idx {
                if skip_indices.contains(&i) {
                  continue;
                }
                if all_selected_elements[i].value().name() != "img" {
                  elements_before_img += 1;
                }
              }
              
              // 估算：如果图片前面有元素，它应该在某个合并文本附近
              // 使用合并文本的位置作为参考
              if !merged_texts.is_empty() {
                // 根据elements_before_img估算在合并文本中的位置
                let estimated_merged_idx = (elements_before_img * merged_texts.len() / all_selected_elements.len().max(1)).min(merged_texts.len() - 1);
                
                if prev_text_pos.is_none() && estimated_merged_idx > 0 {
                  // 使用前一个合并文本的位置
                  prev_text_pos = Some(merged_texts[estimated_merged_idx - 1].top);
                } else if prev_text_pos.is_none() && estimated_merged_idx == 0 {
                  // 图片在第一个合并文本之前
                  prev_text_pos = Some(merged_texts[0].top - 100.0);
                }
                
                if next_text_pos.is_none() && estimated_merged_idx < merged_texts.len() {
                  // 使用当前或下一个合并文本的位置
                  next_text_pos = Some(merged_texts[estimated_merged_idx].top);
                }
              }
            }
            
            // 根据前后文字位置推断图片位置
            let inferred_top = match (prev_text_pos, next_text_pos) {
              (Some(prev), Some(next)) => {
                // 前后都有文字，放在中间
                (prev + next) / 2.0
              }
              (Some(prev), None) => {
                // 只有前面的文字，放在它之后
                prev + 100.0
              }
              (None, Some(next)) => {
                // 只有后面的文字，放在它之前
                (next - 100.0).max(0.0)
              }
              (None, None) => {
                // 没有任何参考位置，使用默认值
                (all_image_indices.iter().position(|&x| x == img_idx).unwrap_or(0) as f64 + 1.0) * 200.0
              }
            };
            
            image_position_map.insert(img_idx, inferred_top);
            tracing::info!("Image at index {} has no position, inferred top={} (prev_text={:?}, next_text={:?})", 
              img_idx, inferred_top, prev_text_pos, next_text_pos);
          } else {
            // 图片已有位置信息，直接使用
            image_position_map.insert(img_idx, existing_top);
          }
        }
        
        for (idx, element) in all_selected_elements.iter().enumerate() {
          if skip_indices.contains(&idx) {
            continue;
          }
          
          // 过滤无效的列表元素（如只包含单个单词或短文本的ul/li，可能是误识别）
          let tag_name = element.value().name();
          if tag_name == "ul" || tag_name == "li" {
            let text = element.text().collect::<String>().trim().to_string();
            // 如果列表内容很短（少于3个字符），可能是无效的列表元素
            if text.len() < 3 {
              tracing::info!("Filtering out invalid list element at index {} with short text '{}'", idx, text);
              continue;
            }
          }
          
          // 获取元素的位置信息
          let mut element_top = 0.0;
          let mut has_position = false;
          if let Some(style) = element.value().attr("style") {
            if let Some(captures) = top_regex.captures(style) {
              if let Some(m) = captures.get(1) {
                element_top = m.as_str().parse::<f64>().unwrap_or(0.0);
                has_position = true;
              }
            }
          }
          
          // 如果是图片且没有位置信息，使用推断的位置
          if tag_name == "img" && !has_position {
            if let Some(&inferred_top) = image_position_map.get(&idx) {
              element_top = inferred_top;
            } else {
              // 如果映射中没有（理论上不应该发生），使用默认值
              element_top = 200.0;
              tracing::warn!("Image at index {} not found in position map, using default top=200.0", idx);
            }
          }
          
          // 标记：0 = 普通元素, 2 = 图片
          let item_type = if tag_name == "img" { 2 } else { 0 };
          ordered_items.push((element_top, item_type, idx));
        }
        
        // 按位置排序
        ordered_items.sort_by(|a, b| {
          a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let mut processed_count = 0;
        
        // 按排序后的顺序处理元素
        for (_pos, item_type, index) in ordered_items {
          if item_type == 1 {
            // 处理合并后的文本
            // index 是 merged_texts 中的索引，已经在 ordered_items 中正确设置
            if index < merged_texts.len() {
              let merged_info = &merged_texts[index];
              
              let merged_block_id = nanoid!(10);
              let merged_text_id = nanoid!(10);
              
              // 根据类型确定块类型：标题、列表项或段落
              let block_type = if merged_info.is_heading {
                "heading".to_string()
              } else if merged_info.is_list_item {
                "bulleted_list".to_string()
              } else {
                "paragraph".to_string()
              };
              
              let block_data = if merged_info.is_heading {
                serde_json::to_string(&serde_json::json!({
                  "level": merged_info.heading_level
                })).unwrap_or_default()
              } else {
                serde_json::to_string(&serde_json::json!({})).unwrap_or_default()
              };
              
              let merged_block = BlockPB {
                id: merged_block_id.clone(),
                ty: block_type,
                data: block_data,
                parent_id: page_id.to_string(),
                children_id: nanoid!(10),
                external_id: Some(merged_text_id.clone()),
                external_type: Some("text".to_string()),
              };
              
              text_map.insert(merged_text_id, serde_json::to_string(&serde_json::json!([
                {
                  "insert": &merged_info.text
                }
              ])).unwrap_or_default());
              
              child_blocks.push(merged_block_id.clone());
              blocks.insert(merged_block_id.clone(), merged_block);
              
              let block_type_str = if merged_info.is_heading {
                "heading"
              } else if merged_info.is_list_item {
                "list item"
              } else {
                "paragraph"
              };
              
              tracing::info!("✅ Created merged {} block: id='{}', text='{}'", 
                block_type_str,
                merged_block_id, self.safe_string_preview(&merged_info.text, DEBUG_TEXT_PREVIEW_LENGTH));
              
              processed_count += 1;
            }
          } else if item_type == 2 {
            // 处理图片元素
            let element = &all_selected_elements[index];
            let tag_name = element.value().name();
            if tag_name == "img" {
              if let Some(block) = self.html_element_to_block(*element, &page_id, &mut text_map, &mut children_map, &mut blocks, &image_map) {
                tracing::info!("✅ Created image block: type='{}', id='{}'", block.ty, block.id);
                child_blocks.push(block.id.clone());
                blocks.insert(block.id.clone(), block);
                processed_count += 1;
              }
            }
          } else {
            // 处理其他原始元素
            let element = &all_selected_elements[index];
            
            let tag_name = element.value().name();
            let text_preview = element.text().collect::<String>();
            let text_trimmed = text_preview.trim();
            let text_display = self.safe_string_preview(text_trimmed, DEBUG_TEXT_PREVIEW_LENGTH);
            
            // 过滤掉"Document Outline"这种不需要的元素
            let text_lower = text_trimmed.to_lowercase();
            if text_lower.contains("document outline") {
              tracing::info!("Filtering out 'Document Outline' element at index {}", index);
              continue;
            }
            
            // 对于表格元素，输出更多调试信息
            if tag_name == "table" {
                let html_snippet = format!("{}", element.html());
                tracing::info!("Element {}: <{}> - HTML snippet (first 500 chars): {}", 
                    index, tag_name, self.safe_string_preview(&html_snippet, 500));
            } else if tag_name == "img" {
                let src = element.value().attr("src").unwrap_or("");
                // 如果 src 是 data URL，只显示前50个字符
                let src_preview = if src.starts_with("data:") {
                    format!("{}...", &src[..std::cmp::min(50, src.len())])
                } else if src.len() > 100 {
                    format!("{}...", &src[..100])
                } else {
                    src.to_string()
                };
                tracing::info!("Element {}: <{}> src='{}' (full length: {})", 
                    index, tag_name, src_preview, src.len());
            } else {
                tracing::info!("Element {}: <{}> '{}' (text length: {})", 
                    index, tag_name, text_display, text_trimmed.len());
            }
            
            if let Some(block) = self.html_element_to_block(*element, &page_id, &mut text_map, &mut children_map, &mut blocks, &image_map) {
                tracing::info!("✅ Created block: type='{}', id='{}'", block.ty, block.id);
                child_blocks.push(block.id.clone());
                blocks.insert(block.id.clone(), block);
                processed_count += 1;
            } else {
                tracing::warn!("❌ Failed to convert element <{}> to block (text: '{}')", tag_name, text_display);
            }
            
            // 每处理一定数量的元素后发送进度更新
            if processed_count > 0 && processed_count % progress_update_interval == 0 && !import_id.is_empty() {
                let progress = 0.7 + (processed_count as f64 / total_elements as f64) * 0.2;
                self.send_progress(
                    import_id,
                    file_name,
                    progress,
                    &format!("正在处理元素 {}/{}...", processed_count, total_elements),
                );
            }
          }
        }
        
        // 所有合并文本都已在 ordered_items 中处理，无需额外处理
        
        tracing::info!("Total blocks created: {} (expected: {} elements)", 
            child_blocks.len(), all_selected_elements.len());
        
        // 发送处理完成的进度更新
        if !import_id.is_empty() {
            self.send_progress(import_id, file_name, 0.85, "格式解析完成，正在准备创建文档...");
        }
        
        
        // 如果没有找到任何块，创建一个默认的段落块
        if child_blocks.is_empty() {
            tracing::warn!("No HTML elements found, creating fallback paragraph block");
            
            // 检查 HTML 中是否有文本内容
            let plain_text = self.html_to_plain_text(content);
            let text_trimmed = plain_text.trim();
            
            if !text_trimmed.is_empty() {
                // 有文本内容，创建段落块
                let content_block_id = nanoid!(10);
                let content_text_id = nanoid!(10);
                let content_block = BlockPB {
                    id: content_block_id.clone(),
                    ty: "paragraph".to_string(),
                    data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
                    parent_id: page_id.to_string(),
                    children_id: nanoid!(10),
                    external_id: Some(content_text_id.clone()),
                    external_type: Some("text".to_string()),
                };
                
                blocks.insert(content_block_id.clone(), content_block);
                child_blocks.push(content_block_id.clone());
                
                // 设置内容文本
                text_map.insert(content_text_id, serde_json::to_string(&serde_json::json!([
                    {
                        "insert": text_trimmed
                    }
                ])).unwrap_or_default());
                
                tracing::info!("Created fallback paragraph block with {} characters of text", text_trimmed.len());
            } else {
                // 没有文本内容，只创建空块
                tracing::warn!("HTML content is empty, no fallback block created");
            }
        }
        
        tracing::info!("Created {} blocks from HTML content", child_blocks.len());
            
            // 在HTML格式下，追加所有图片到文档末尾
            if !result.extracted_images.is_empty() {
                tracing::info!("Appending {} images to the end of document", result.extracted_images.len());
                
                for (idx, image) in result.extracted_images.iter().enumerate() {
                    let image_number = idx + 1;
                    
                    // 1. 先添加【图片 N】文本标签
                    let label_block_id = nanoid!(10);
                    let label_text_id = nanoid!(10);
                    let label_block = BlockPB {
                        id: label_block_id.clone(),
                        ty: "paragraph".to_string(),
                        data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
                        parent_id: page_id.to_string(),
                        children_id: nanoid!(10),
                        external_id: Some(label_text_id.clone()),
                        external_type: Some("text".to_string()),
                    };
                    
                    text_map.insert(label_text_id, serde_json::to_string(&serde_json::json!([
                        {
                            "insert": format!("【图片 {}】", image_number)
                        }
                    ])).unwrap_or_default());
                    
                    child_blocks.push(label_block_id.clone());
                    blocks.insert(label_block_id, label_block);
                    
                    // 2. 添加图片块
                    let image_id = nanoid!(10);
                    use base64::{Engine as _, engine::general_purpose};
                    let base64_data = general_purpose::STANDARD.encode(&image.data);
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
                    
                    // 使用图片的实际尺寸，如果没有尺寸信息则尝试从图片数据中读取
                    // 使用轻量级方法只读取图片头部信息，避免加载整个图片导致栈溢出
                    let (width, height) = if let Some(dimensions) = &image.dimensions {
                        (dimensions.width, dimensions.height)
                    } else {
                        // 尝试从图片数据中读取尺寸（只读取头部，不加载整个图片）
                        match Self::get_image_dimensions_lightweight(&image.data, &image.format) {
                            Some((w, h)) => (w, h),
                            None => {
                                tracing::warn!("无法从图片数据读取尺寸，使用默认值 300x200");
                                (300, 200)
                            }
                        }
                    };
                    
                    let image_block = BlockPB {
                        id: image_id.clone(),
                        ty: "image".to_string(),
                        data: serde_json::to_string(&serde_json::json!({
                            "url": data_url,
                            "width": width,
                            "height": height,
                            "align": "center",
                            "image_type": 2  // CustomImageType.external for base64 data URLs
                        })).unwrap_or_default(),
                        parent_id: page_id.to_string(),
                        children_id: nanoid!(10),
                        external_id: None,
                        external_type: None,
                    };
                    
                    child_blocks.push(image_id.clone());
                    blocks.insert(image_id, image_block);
                    
                    tracing::info!("✅ Appended image {}: {} ({} bytes)", image_number, image.filename, image.data.len());
                }
                
                tracing::info!("Successfully appended {} images to document end", result.extracted_images.len());
            }
        } // 结束 else (HTML格式) 块
        
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
        
        // 发送文档创建完成的日志
        let total_blocks = blocks.len();
        let total_text_blocks = text_map.len();
        self.add_import_log(import_id, "info", &format!("AppFlowy 文档结构创建完成: {} 个块, {} 个文本块", total_blocks, total_text_blocks));
        
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
  /// 检测内容是否为Markdown格式
  fn detect_markdown_format(&self, content: &str) -> bool {
    let trimmed = content.trim();
    
    // 检查是否以HTML标签开头（明确的HTML标记）
    if trimmed.starts_with('<') && (trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<?xml") || trimmed.starts_with("<html") || trimmed.starts_with("<head")) {
      return false;
    }
    
    // 检查是否包含明显的HTML结构标记
    if trimmed.contains("<html") || trimmed.contains("<head") || trimmed.contains("<body") || trimmed.contains("<!DOCTYPE") {
      return false;
    }
    
    // 检查是否包含Markdown特征：
    // 1. 标题标记 (# 开头)
    // 2. 列表标记 (-, *, +, 或数字. 开头)
    // 3. 代码块标记 (``` 或 `)
    // 4. 链接标记 ([text](url))
    // 5. 图片标记 (![alt](url))
    // 6. 表格标记 (|)
    let has_markdown_features = trimmed.contains("# ") 
      || trimmed.contains("- ") 
      || trimmed.contains("* ") 
      || trimmed.contains("```")
      || trimmed.contains("![")
      || (trimmed.contains("|") && trimmed.lines().any(|line| {
        let line_trimmed = line.trim();
        line_trimmed.starts_with('|') && line_trimmed.matches('|').count() >= 3
      }));
    
    // 如果没有明显的HTML标记，但有文本内容，可能是Markdown（纯文本Markdown）
    // 特别是如果内容包含中文字符且没有HTML标签，很可能是Markdown
    let has_text_content = trimmed.len() > 0;
    let has_html_tags = trimmed.contains('<') && trimmed.contains('>');
    let has_chinese_chars = trimmed.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}');
    
    // 如果包含Markdown特征，肯定是Markdown
    if has_markdown_features {
      return true;
    }
    
    // 如果有文本内容，没有HTML标签，且不是HTML文档结构，可能是纯文本Markdown
    // 特别是包含中文字符的情况（从PDF转换来的Markdown通常是纯文本）
    if has_text_content && !has_html_tags && has_chinese_chars {
      return true;
    }
    
    // 默认情况下，如果内容不是HTML文档结构，可能是Markdown
    !has_html_tags && has_text_content
  }
  
  /// 解析Markdown内容并转换为AppFlowy块结构
  fn parse_markdown_to_blocks(
    &self,
    markdown: &str,
    extracted_images: &[flowy_document::import::ExtractedImage],
    page_id: &str,
    blocks: &mut HashMap<String, BlockPB>,
    text_map: &mut HashMap<String, String>,
    children_map: &mut HashMap<String, ChildrenPB>,
  ) -> FlowyResult<(Vec<String>, Vec<String>)> {
    use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind, Options};
    
    let mut child_blocks = Vec::new();
    // 启用表格解析选项（ENABLE_TABLES），否则 pulldown-cmark 不会解析表格
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown, options);
    
    let mut current_paragraph_text = Vec::new();
    let mut current_list_items: Vec<String> = Vec::new();
    let mut current_list_type: Option<String> = None; // "bullet" or "number"
    let mut in_code_block = false;
    let mut code_block_lang: Option<String> = None;
    let mut code_block_content = String::new();
    let mut current_table: Vec<Vec<String>> = Vec::new();
    let mut in_table = false;
    let mut in_table_header = false;
    let mut in_table_row = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut current_table_row: Vec<String> = Vec::new();
    let mut current_table_cell = String::new();
    let mut inserted_image_ids = Vec::new(); // 跟踪已插入的图片ID
    
    for event in parser {
      match event {
        Event::Start(Tag::Heading(level, _, _)) => {
          // 结束当前段落或列表
          self.flush_paragraph(&mut current_paragraph_text, &mut current_list_items, &mut current_list_type, 
            page_id, &mut child_blocks, blocks, text_map, children_map)?;
          
          // 开始新的标题
          current_paragraph_text.clear();
        }
        Event::End(Tag::Heading(level, _, _)) => {
          // 创建标题块
          let text = current_paragraph_text.join("");
          if !text.trim().is_empty() {
            let heading_id = nanoid!(10);
            let text_id = nanoid!(10);
            
            let block = BlockPB {
              id: heading_id.clone(),
              ty: "heading".to_string(),
              data: serde_json::to_string(&serde_json::json!({
                "level": level as u32
              })).unwrap_or_default(),
              parent_id: page_id.to_string(),
              children_id: nanoid!(10),
              external_id: Some(text_id.clone()),
              external_type: Some("text".to_string()),
            };
            
            text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
              { "insert": text }
            ])).unwrap_or_default());
            
            child_blocks.push(heading_id.clone());
            blocks.insert(heading_id, block);
          }
          // 标题结束后，确保清空段落文本，避免残留文本进入后续内容
          current_paragraph_text.clear();
        }
        Event::Start(Tag::Paragraph) => {
          // 开始新段落
          // 如果在列表中，段落文本将作为列表项的内容
          // 清空段落文本，准备收集新的段落内容
          // 注意：在列表项中，每个段落开始都应该清空，避免残留文本
          if current_list_type.is_some() {
            // 在列表中，确保清空段落文本，避免残留文本进入列表项
            current_paragraph_text.clear();
          } else {
            // 不在列表中，正常清空段落文本
            current_paragraph_text.clear();
          }
        }
        Event::End(Tag::Paragraph) => {
          // 结束段落，创建段落块
          // 但如果当前在列表中，段落文本应该保留给列表项使用，而不是创建独立的段落块
          if current_list_type.is_some() {
            // 在列表中，段落文本将作为列表项的内容，不创建独立的段落块
            // 文本保留在 current_paragraph_text 中，等待 Event::End(Tag::Item) 时添加到列表项
          } else {
            // 不在列表中，创建段落块
            let text = current_paragraph_text.join("");
            if !text.trim().is_empty() {
              let para_id = nanoid!(10);
              let text_id = nanoid!(10);
              
              let block = BlockPB {
                id: para_id.clone(),
                ty: "paragraph".to_string(),
                data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
                parent_id: page_id.to_string(),
                children_id: nanoid!(10),
                external_id: Some(text_id.clone()),
                external_type: Some("text".to_string()),
              };
              
              text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
                { "insert": text }
              ])).unwrap_or_default());
              
              child_blocks.push(para_id.clone());
              blocks.insert(para_id, block);
              current_paragraph_text.clear();
            }
          }
        }
        Event::Start(Tag::List(Some(_))) => {
          // 有序列表开始（从任何数字开始，包括0, 1, 2等）
          // 先刷新并清空之前的段落，避免残留文本进入列表
          // 注意：flush_paragraph 可能会创建列表，所以我们需要先处理它
          self.flush_paragraph(&mut current_paragraph_text, &mut current_list_items, &mut current_list_type, 
            page_id, &mut child_blocks, blocks, text_map, children_map)?;
          // 确保段落文本和列表项都被清空，避免残留数据进入新列表
          current_paragraph_text.clear();
          current_list_items.clear();
          current_list_type = Some("number".to_string());
        }
        Event::Start(Tag::List(None)) => {
          // 无序列表开始
          // 先刷新并清空之前的段落，避免残留文本进入列表
          // 注意：flush_paragraph 可能会创建列表，所以我们需要先处理它
          self.flush_paragraph(&mut current_paragraph_text, &mut current_list_items, &mut current_list_type, 
            page_id, &mut child_blocks, blocks, text_map, children_map)?;
          // 确保段落文本和列表项都被清空，避免残留数据进入新列表
          current_paragraph_text.clear();
          current_list_items.clear();
          current_list_type = Some("bullet".to_string());
        }
        Event::End(Tag::List(_)) => {
          // 列表结束，创建列表块
          self.flush_list(&current_list_items, &current_list_type, page_id, 
            &mut child_blocks, blocks, text_map, children_map)?;
          current_list_items.clear();
          current_list_type = None;
        }
        Event::Start(Tag::Item) => {
          // 列表项开始，清空段落文本，确保每个列表项从干净状态开始
          // 这很重要，因为列表项开始前可能有残留文本
          let prev_text = current_paragraph_text.join("");
          if !prev_text.trim().is_empty() {
            tracing::warn!("[列表解析] 列表项开始前有残留文本: '{}'，将被清空", prev_text);
          }
          current_paragraph_text.clear();
        }
        Event::End(Tag::Item) => {
          // 列表项结束
          let text = current_paragraph_text.join("");
          let trimmed_text = text.trim();
          // 只有当文本不为空时才添加到列表项中
          // 过滤掉可能的残留文本，如"列表"、"List"等常见的误识别文本
          if !trimmed_text.is_empty() 
            && trimmed_text != "列表" 
            && trimmed_text != "List"
            && trimmed_text != "list" {
            current_list_items.push(trimmed_text.to_string());
          } else if !trimmed_text.is_empty() {
            tracing::warn!("[列表解析] 过滤掉残留文本: '{}'", trimmed_text);
          } else {
            tracing::warn!("[列表解析] 列表项为空，跳过添加");
          }
          current_paragraph_text.clear();
        }
        Event::Start(Tag::BlockQuote) => {
          // 引用块开始，暂时作为普通段落处理
          // 可以在后续改进中创建专门的引用块
        }
        Event::End(Tag::BlockQuote) => {
          // 引用块结束，文本已在段落中处理
        }
        Event::Start(Tag::FootnoteDefinition(_)) => {
          // 脚注定义开始，暂时忽略
        }
        Event::End(Tag::FootnoteDefinition(_)) => {
          // 脚注定义结束
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
          in_code_block = true;
          code_block_lang = Some(lang.to_string());
          code_block_content.clear();
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
          // 缩进式代码块（4个空格开始）
          in_code_block = true;
          code_block_lang = None;
          code_block_content.clear();
        }
        Event::End(Tag::CodeBlock(_)) => {
          // 创建代码块
          if !code_block_content.trim().is_empty() {
            let code_id = nanoid!(10);
            let text_id = nanoid!(10);
            
            let mut data = serde_json::json!({
              "language": code_block_lang.as_ref().unwrap_or(&"".to_string())
            });
            
            let block = BlockPB {
              id: code_id.clone(),
              ty: "code".to_string(),
              data: serde_json::to_string(&data).unwrap_or_default(),
              parent_id: page_id.to_string(),
              children_id: nanoid!(10),
              external_id: Some(text_id.clone()),
              external_type: Some("text".to_string()),
            };
            
            text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
              { "insert": code_block_content.clone() }
            ])).unwrap_or_default());
            
            child_blocks.push(code_id.clone());
            blocks.insert(code_id, block);
          }
          in_code_block = false;
          code_block_lang = None;
          code_block_content.clear();
        }
        Event::Start(Tag::Table(_)) => {
          in_table = true;
          current_table.clear();
          table_headers.clear();
        }
        Event::End(Tag::Table(_)) => {
          // 创建表格块
          // 如果没有表头但有数据行，将第一行作为表头
          let (final_headers, final_rows) = if table_headers.is_empty() && !current_table.is_empty() {
            tracing::info!("[表格解析] 没有检测到表头，将第一行作为表头");
            let first_row = current_table[0].clone();
            let remaining_rows = current_table[1..].to_vec();
            (first_row, remaining_rows)
          } else {
            (table_headers.clone(), current_table.clone())
          };
          
          if !final_headers.is_empty() || !final_rows.is_empty() {
            tracing::info!("[表格解析] 创建表格块: {} 个表头, {} 行数据", final_headers.len(), final_rows.len());
            self.create_table_block(&final_headers, &final_rows, page_id,
              &mut child_blocks, blocks, text_map, children_map)?;
            tracing::info!("[表格解析] 表格块创建成功");
          } else {
            tracing::warn!("[表格解析] 表格为空，跳过创建");
          }
          in_table = false;
          in_table_header = false;
          in_table_row = false;
          current_table.clear();
          table_headers.clear();
          current_table_row.clear();
          current_table_cell.clear();
        }
        Event::Start(Tag::TableHead) => {
          in_table_header = true;
        }
        Event::End(Tag::TableHead) => {
          in_table_header = false;
        }
        Event::Start(Tag::TableRow) => {
          in_table_row = true;
          current_table_row.clear();
        }
        Event::End(Tag::TableRow) => {
          if in_table_header {
            // 表头行
            if !current_table_row.is_empty() {
              table_headers = current_table_row.clone();
            }
          } else {
            // 数据行
            if !current_table_row.is_empty() {
              current_table.push(current_table_row.clone());
            }
          }
          in_table_row = false;
          current_table_row.clear();
        }
        Event::Start(Tag::TableCell) => {
          current_table_cell.clear();
        }
        Event::End(Tag::TableCell) => {
          // 表格单元格结束，添加到当前行
          let cell_text = current_table_cell.trim().to_string();
          current_table_row.push(cell_text);
          current_table_cell.clear();
        }
        Event::Text(text) => {
          if in_code_block {
            code_block_content.push_str(&text);
          } else if in_table && in_table_row {
            // 表格单元格文本
            current_table_cell.push_str(&text);
          } else {
            // 将文本添加到当前段落，但保留换行信息
            // 如果文本包含换行符，需要特殊处理
            let trimmed_text = text.trim();
            if !trimmed_text.is_empty() {
              // 如果当前段落不为空且文本不是以空格开头，添加空格
              if !current_paragraph_text.is_empty() && !trimmed_text.starts_with(' ') {
                let last_text = current_paragraph_text.last().unwrap();
                if !last_text.ends_with(' ') && !last_text.ends_with('\n') {
                  current_paragraph_text.push(" ".to_string());
                }
              }
              current_paragraph_text.push(trimmed_text.to_string());
            }
          }
        }
        Event::Code(code) => {
          // 行内代码，不添加 Markdown 语法字符（反引号）
          // 直接添加代码文本，格式信息会在后续的文本处理中应用（如果需要支持代码格式）
          current_paragraph_text.push(code.to_string());
        }
        Event::Html(_) => {
          // 忽略HTML标签（已在转换前清理）
        }
        Event::SoftBreak => {
          // 软换行，在段落内部添加空格而不是换行
          if !current_paragraph_text.is_empty() {
            let last_text = current_paragraph_text.last().unwrap();
            if !last_text.ends_with(' ') {
              current_paragraph_text.push(" ".to_string());
            }
          }
        }
        Event::HardBreak => {
          // 硬换行，在段落内部添加换行符
          // 但为了保持段落格式，我们将其转换为空格
          if !current_paragraph_text.is_empty() {
            let last_text = current_paragraph_text.last().unwrap();
            if !last_text.ends_with(' ') && !last_text.ends_with('\n') {
              current_paragraph_text.push(" ".to_string());
            }
          }
        }
        Event::TaskListMarker(_) => {
          // TODO列表标记
        }
        Event::Start(Tag::Image(link_type, url, title)) => {
          // 图片开始，保存URL和alt文本
          // 图片URL和alt文本在Start事件中就有，title是标题（可能是空字符串）
          let alt_text = title.to_string();
          if let Ok(Some(image_id)) = self.create_image_block_from_markdown(&url.to_string(), &alt_text, extracted_images,
            page_id, &mut child_blocks, blocks) {
            inserted_image_ids.push(image_id);
          }
        }
        Event::End(Tag::Image(link_type, url, title)) => {
          // 图片结束，已经在Start事件中处理
        }
        Event::Start(Tag::Link(link_type, url, title)) => {
          // 链接开始（在处理文本时保留）
        }
        Event::End(Tag::Link(link_type, url, title)) => {
          // 链接结束（文本已处理，URL信息可能需要保留）
        }
        Event::Start(Tag::Emphasis) => {
          // 斜体开始，不添加 Markdown 语法字符
          // 格式信息会在后续的文本处理中应用（如果需要支持格式）
        }
        Event::End(Tag::Emphasis) => {
          // 斜体结束，不添加 Markdown 语法字符
        }
        Event::Start(Tag::Strong) => {
          // 粗体开始，不添加 Markdown 语法字符
          // 格式信息会在后续的文本处理中应用（如果需要支持格式）
        }
        Event::End(Tag::Strong) => {
          // 粗体结束，不添加 Markdown 语法字符
        }
        Event::Start(Tag::Strikethrough) => {
          // 删除线开始，不添加 Markdown 语法字符
          // 格式信息会在后续的文本处理中应用（如果需要支持格式）
        }
        Event::End(Tag::Strikethrough) => {
          // 删除线结束，不添加 Markdown 语法字符
        }
        Event::FootnoteReference(_) => {
          // 脚注引用，暂时忽略
        }
        Event::Rule => {
          // 水平线（---），创建分隔符块
          let divider_id = nanoid!(10);
          let divider_block = BlockPB {
            id: divider_id.clone(),
            ty: "divider".to_string(),
            data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
            parent_id: page_id.to_string(),
            children_id: nanoid!(10),
            external_id: None,
            external_type: None,
          };
          child_blocks.push(divider_id.clone());
          blocks.insert(divider_id, divider_block);
        }
      }
    }
    
    // 处理剩余的段落或列表
    self.flush_paragraph(&mut current_paragraph_text, &mut current_list_items, &mut current_list_type,
      page_id, &mut child_blocks, blocks, text_map, children_map)?;
    
    tracing::info!("Parsed Markdown: created {} blocks, inserted {} images", child_blocks.len(), inserted_image_ids.len());
    Ok((child_blocks, inserted_image_ids))
  }
  
  /// 刷新段落和列表（辅助方法）
  fn flush_paragraph(
    &self,
    current_paragraph_text: &mut Vec<String>,
    current_list_items: &mut Vec<String>,
    current_list_type: &mut Option<String>,
    page_id: &str,
    child_blocks: &mut Vec<String>,
    blocks: &mut HashMap<String, BlockPB>,
    text_map: &mut HashMap<String, String>,
    children_map: &mut HashMap<String, ChildrenPB>,
  ) -> FlowyResult<()> {
    // 如果当前有列表项，先创建列表
    if !current_list_items.is_empty() {
      self.flush_list(current_list_items, current_list_type, page_id, 
        child_blocks, blocks, text_map, children_map)?;
      current_list_items.clear();
      *current_list_type = None;
    }
    
    // 如果当前有段落文本，创建段落
    let text = current_paragraph_text.join("");
    if !text.trim().is_empty() {
      let para_id = nanoid!(10);
      let text_id = nanoid!(10);
      
      let block = BlockPB {
        id: para_id.clone(),
        ty: "paragraph".to_string(),
        data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
        parent_id: page_id.to_string(),
        children_id: nanoid!(10),
        external_id: Some(text_id.clone()),
        external_type: Some("text".to_string()),
      };
      
      text_map.insert(text_id, serde_json::to_string(&serde_json::json!([
        { "insert": text }
      ])).unwrap_or_default());
      
      child_blocks.push(para_id.clone());
      blocks.insert(para_id, block);
      current_paragraph_text.clear();
    }
    
    Ok(())
  }
  
  /// 刷新列表（辅助方法）
  fn flush_list(
    &self,
    list_items: &[String],
    list_type: &Option<String>,
    page_id: &str,
    child_blocks: &mut Vec<String>,
    blocks: &mut HashMap<String, BlockPB>,
    text_map: &mut HashMap<String, String>,
    children_map: &mut HashMap<String, ChildrenPB>,
  ) -> FlowyResult<()> {
    if list_items.is_empty() {
      tracing::warn!("[列表创建] 列表项为空，跳过创建列表块");
      return Ok(());
    }
    
    tracing::info!("[列表创建] 开始创建列表块: {} 个列表项, 类型: {:?}", list_items.len(), list_type);
    
    let list_ty = match list_type.as_deref() {
      Some("number") => "numbered_list",
      _ => "bulleted_list", // 修复：使用正确的类型名称 "bulleted_list" 而不是 "bullet_list"
    };
    
    // 过滤并收集有效的列表项
    let valid_items: Vec<(usize, String)> = list_items.iter()
      .enumerate()
      .filter_map(|(idx, item_text)| {
        let trimmed = item_text.trim();
        if trimmed.is_empty() {
          tracing::warn!("[列表创建] 跳过空的列表项 #{}", idx + 1);
          None
        } else {
          Some((idx, trimmed.to_string()))
        }
      })
      .collect();
    
    if valid_items.is_empty() {
      tracing::warn!("[列表创建] 所有列表项都为空，跳过创建列表块");
      return Ok(());
    }
    
    // AppFlowy 的列表块结构：
    // 平级列表项应该是独立的列表块（bulleted_list 或 numbered_list），都直接放在父级的 children 中
    // 每个列表项都有自己的 data.delta，包含该项的内容
    // 参考 range_2.json：多个平级的 numbered_list 块都直接放在 page 的 children 中
    
    // 为每个列表项创建独立的列表块
    for (idx, (_, item_text)) in valid_items.iter().enumerate() {
      let item_id = nanoid!(10);
      let item_text_id = nanoid!(10);
      let item_children_id = nanoid!(10);
      
      // 每个列表项都是一个独立的列表块
      let item_block = BlockPB {
        id: item_id.clone(),
        ty: list_ty.to_string(),
        data: serde_json::to_string(&serde_json::json!({
          "delta": [
            { "insert": item_text }
          ]
        })).unwrap_or_default(),
        parent_id: page_id.to_string(), // 所有项都直接放在父级（page）中，而不是嵌套
        children_id: item_children_id.clone(),
        external_id: Some(item_text_id.clone()),
        external_type: Some("text".to_string()),
      };
      
      text_map.insert(item_text_id, serde_json::to_string(&serde_json::json!([
        { "insert": item_text.clone() }
      ])).unwrap_or_default());
      
      // 所有列表项都直接添加到父级的 children 中，作为平级项
      child_blocks.push(item_id.clone());
      blocks.insert(item_id, item_block);
      children_map.insert(item_children_id, ChildrenPB { children: vec![] });
    }
    
    tracing::info!("[列表创建] 创建了 {} 个平级列表项，都直接放在父级 children 中", valid_items.len());
    
    Ok(())
  }
  
  /// 创建表格块（辅助方法）
  fn create_table_block(
    &self,
    headers: &[String],
    rows: &[Vec<String>],
    page_id: &str,
    child_blocks: &mut Vec<String>,
    blocks: &mut HashMap<String, BlockPB>,
    text_map: &mut HashMap<String, String>,
    children_map: &mut HashMap<String, ChildrenPB>,
  ) -> FlowyResult<()> {
    tracing::info!("[表格创建] 开始创建表格: {} 个表头, {} 行数据", headers.len(), rows.len());
    
    // 创建简单表格块
    let table_id = nanoid!(10);
    let table_children_id = nanoid!(10);
    
    // 计算列数
    let col_count = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    
    // 表格的 data 字段需要包含所有必要的属性
    let has_headers = !headers.is_empty();
    let table_block = BlockPB {
      id: table_id.clone(),
      ty: "simple_table".to_string(),
      data: serde_json::to_string(&serde_json::json!({
        "col_count": col_count,
        "row_count": rows.len() + if has_headers { 1 } else { 0 },
        "enable_header_row": has_headers,
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
      parent_id: page_id.to_string(),
      children_id: table_children_id.clone(),
      external_id: None,
      external_type: None,
    };
    
    // 创建表头行（如果有）
    let mut table_row_blocks = Vec::new();
    if !headers.is_empty() {
      let header_row_id = nanoid!(10);
      let header_row_children_id = nanoid!(10);
      let header_row_block = BlockPB {
        id: header_row_id.clone(),
        ty: "simple_table_row".to_string(),
        data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
        parent_id: table_id.clone(),
        children_id: header_row_children_id.clone(),
        external_id: None,
        external_type: None,
      };
      
      let mut header_cells = Vec::new();
      for header_text in headers {
        let cell_id = nanoid!(10);
        let cell_children_id = nanoid!(10);
        
        let cell_block = BlockPB {
          id: cell_id.clone(),
          ty: "simple_table_cell".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: header_row_id.clone(),
          children_id: cell_children_id.clone(),
          external_id: None,
          external_type: None,
        };
        
        // 为单元格创建段落子块
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
          { "insert": header_text.clone() }
        ])).unwrap_or_default());
        
        // 设置单元格的children（包含段落块）
        children_map.insert(cell_children_id.clone(), ChildrenPB {
          children: vec![paragraph_id.clone()]
        });
        
        blocks.insert(paragraph_id, paragraph_block);
        header_cells.push(cell_id.clone());
        blocks.insert(cell_id, cell_block);
      }
      
      children_map.insert(header_row_children_id.clone(), ChildrenPB {
        children: header_cells
      });
      
      blocks.insert(header_row_id.clone(), header_row_block);
      table_row_blocks.push(header_row_id.clone());
    }
    
    // 创建数据行
    for (row_idx, row) in rows.iter().enumerate() {
      let row_id = nanoid!(10);
      let row_children_id = nanoid!(10);
      let row_block = BlockPB {
        id: row_id.clone(),
        ty: "simple_table_row".to_string(),
        data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
        parent_id: table_id.clone(),
        children_id: row_children_id.clone(),
        external_id: None,
        external_type: None,
      };
      
      let mut row_cells = Vec::new();
      for cell_text in row {
        let cell_id = nanoid!(10);
        let cell_children_id = nanoid!(10);
        
        let cell_block = BlockPB {
          id: cell_id.clone(),
          ty: "simple_table_cell".to_string(),
          data: serde_json::to_string(&serde_json::json!({})).unwrap_or_default(),
          parent_id: row_id.clone(),
          children_id: cell_children_id.clone(),
          external_id: None,
          external_type: None,
        };
        
        // 为单元格创建段落子块
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
          { "insert": cell_text.clone() }
        ])).unwrap_or_default());
        
        // 设置单元格的children（包含段落块）
        children_map.insert(cell_children_id.clone(), ChildrenPB {
          children: vec![paragraph_id.clone()]
        });
        
        blocks.insert(paragraph_id, paragraph_block);
        row_cells.push(cell_id.clone());
        blocks.insert(cell_id, cell_block);
      }
      
      children_map.insert(row_children_id.clone(), ChildrenPB {
        children: row_cells
      });
      
      blocks.insert(row_id.clone(), row_block);
      table_row_blocks.push(row_id.clone());
    }
    
    children_map.insert(table_children_id.clone(), ChildrenPB {
      children: table_row_blocks.clone()
    });
    
    child_blocks.push(table_id.clone());
    blocks.insert(table_id.clone(), table_block);
    
    tracing::info!("[表格创建] 表格创建成功: ID={}, {} 行, {} 列", table_id, table_row_blocks.len(), col_count);
    
    Ok(())
  }
  
  /// 轻量级方法：只读取图片头部信息获取尺寸，避免加载整个图片导致栈溢出
  /// 
  /// 限制图片大小，只处理小于 2MB 的图片，避免在处理大图片时导致栈溢出。
  /// 对于大图片，返回 None，使用默认尺寸。
  /// 
  /// # 参数
  /// - `data`: 图片数据
  /// - `format`: 图片格式
  /// 
  /// # 返回
  /// - `Option<(u32, u32)>`: 图片尺寸 (width, height)，如果无法读取则返回 None
  fn get_image_dimensions_lightweight(
    data: &[u8],
    _format: &flowy_document::import::ImageFormat,
  ) -> Option<(u32, u32)> {
    // 限制图片大小，只处理小于 2MB 的图片，避免栈溢出
    // 对于更大的图片，直接使用默认尺寸，避免栈溢出风险
    const MAX_SIZE_FOR_DIMENSION_READ: usize = 2 * 1024 * 1024; // 2MB
    if data.len() > MAX_SIZE_FOR_DIMENSION_READ {
      tracing::debug!("图片过大 ({} bytes)，跳过尺寸读取以避免栈溢出，使用默认值 300x200", data.len());
      return None;
    }
    
    // 对于小图片，尝试使用 image::load_from_memory 读取尺寸
    // 由于图片较小（< 2MB），不会导致栈溢出
    match image::load_from_memory(data) {
      Ok(img) => {
        let (w, h) = (img.width(), img.height());
        tracing::debug!("成功读取图片尺寸: {}x{} (数据大小: {} bytes)", w, h, data.len());
        Some((w, h))
      }
      Err(e) => {
        tracing::debug!("无法读取图片尺寸: {} (数据大小: {} bytes)", e, data.len());
        None
      }
    }
  }

  /// 从Markdown图片URL创建图片块
  /// 返回匹配的原始图片ID（如果找到），或None（如果找不到图片）
  fn create_image_block_from_markdown(
    &self,
    url: &str,
    alt_text: &str,
    extracted_images: &[flowy_document::import::ExtractedImage],
    page_id: &str,
    child_blocks: &mut Vec<String>,
    blocks: &mut HashMap<String, BlockPB>,
  ) -> FlowyResult<Option<String>> {
    use base64::{Engine as _, engine::general_purpose};
    
    // 查找匹配的图片
    let matched_image = if url.starts_with("data:") {
      // 如果是data URL，尝试从extracted_images中查找匹配的图片
      // 通过比较base64数据来匹配
      extracted_images.iter().find(|img| {
        let img_base64 = general_purpose::STANDARD.encode(&img.data);
        url.contains(&img_base64[..std::cmp::min(100, img_base64.len())])
      })
    } else {
      // 从extracted_images中查找，通过文件名或ID匹配
      extracted_images.iter().find(|img| {
        img.filename == url || 
        url.contains(&img.filename) || 
        url.contains(&img.id) ||
        img.filename.contains(url) ||
        url.contains("IMAGE_PLACEHOLDER")
      })
    };
    
    let (data_url, width, height, matched_image_id) = if let Some(image) = matched_image {
      let base64_data = general_purpose::STANDARD.encode(&image.data);
      let mime_type = match image.format {
        flowy_document::import::ImageFormat::Png => "png",
        flowy_document::import::ImageFormat::Jpeg => "jpeg",
        flowy_document::import::ImageFormat::Gif => "gif",
        flowy_document::import::ImageFormat::Bmp => "bmp",
        flowy_document::import::ImageFormat::WebP => "webp",
      };
      let data_url = format!("data:image/{};base64,{}", mime_type, base64_data);
      let (w, h) = if let Some(dimensions) = &image.dimensions {
        (dimensions.width, dimensions.height)
      } else {
        // 尝试从图片数据中读取尺寸（只读取头部，不加载整个图片）
        match Self::get_image_dimensions_lightweight(&image.data, &image.format) {
          Some((w, h)) => (w, h),
          None => {
            tracing::warn!("无法从图片数据读取尺寸，使用默认值 300x200");
            (300, 200)
          }
        }
      };
      (data_url, w, h, Some(image.id.clone()))
    } else if url.starts_with("data:") {
      // 直接使用data URL，但没有匹配的图片
      (url.to_string(), 300, 200, None)
    } else {
      // 无法找到图片，创建占位符
      tracing::warn!("Image not found for URL: {}, creating placeholder", url);
      // 创建一个占位符图片块
      let placeholder_url = format!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==");
      (placeholder_url, 300, 200, None)
    };
    
    let image_id = nanoid!(10);
    let image_block = BlockPB {
      id: image_id.clone(),
      ty: "image".to_string(),
      data: serde_json::to_string(&serde_json::json!({
        "url": data_url,
        "width": width,
        "height": height,
        "align": "center",
        "image_type": 2
      })).unwrap_or_default(),
      parent_id: page_id.to_string(),
      children_id: nanoid!(10),
      external_id: None,
      external_type: None,
    };
    
    child_blocks.push(image_id.clone());
    blocks.insert(image_id.clone(), image_block);
    
    // 返回匹配的原始图片ID，用于跟踪（如果找到了匹配的图片）
    Ok(matched_image_id)
  }

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
        // 提取文本内容并清理
        let raw_text = element.text().collect::<String>();
        let text_content = self.clean_text_for_merging(&raw_text);
        
        // 如果清理后文本为空或只有单个标点符号，跳过
        if text_content.trim().is_empty() {
          tracing::debug!("Skipping empty paragraph element");
          return None;
        }
        
        // 如果只有单个标点符号（非字母数字），也跳过
        let trimmed = text_content.trim();
        if trimmed.len() == 1 && !trimmed.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
          tracing::debug!("Skipping paragraph with single punctuation: '{}'", trimmed);
          return None;
        }
        
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
        
        tracing::debug!("Created paragraph block: text='{}'", text_content.trim());
        Some(block)
      },
      "ul" => {
        // 检查是否是误判的列表元素
        // 如果<ul>只包含文本（没有真正的<li>子元素），这可能是pdftohtml的误判
        // 应该跳过，让文本被合并逻辑处理
        let text_content = element.text().collect::<String>();
        let has_li_children = element.children().any(|child| {
          if let scraper::Node::Element(elem) = child.value() {
            elem.name() == "li"
          } else {
            false
          }
        });
        
        // 如果没有真正的<li>子元素，且文本内容看起来像标题或段落（不是列表），跳过
        if !has_li_children && !text_content.trim().is_empty() {
          tracing::debug!("Skipping <ul> element that appears to be misidentified (text: '{}'). It will be handled by merged text logic.", text_content.trim());
          return None;
        }
        
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
        // 检查是否是误判的列表元素
        let text_content = element.text().collect::<String>();
        let has_li_children = element.children().any(|child| {
          if let scraper::Node::Element(elem) = child.value() {
            elem.name() == "li"
          } else {
            false
          }
        });
        
        // 如果没有真正的<li>子元素，且文本内容看起来像标题或段落（不是列表），跳过
        if !has_li_children && !text_content.trim().is_empty() {
          tracing::debug!("Skipping <ol> element that appears to be misidentified (text: '{}'). It will be handled by merged text logic.", text_content.trim());
          return None;
        }
        
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
        // 检查<li>是否有父元素<ul>或<ol>
        // 如果父元素不是真正的列表，这可能是误判，应该跳过
        let parent = element.parent();
        let is_in_real_list = parent.and_then(|p| {
          if let scraper::Node::Element(elem) = p.value() {
            let parent_tag = elem.name();
            Some(parent_tag == "ul" || parent_tag == "ol")
          } else {
            None
          }
        }).unwrap_or(false);
        
        // 如果没有在真正的列表中，且文本内容看起来像标题或段落，跳过
        // 让合并文本逻辑处理它
        let text_content = element.text().collect::<String>();
        if !is_in_real_list && !text_content.trim().is_empty() {
          // 检查文本是否看起来像标题（短文本，通常是标题）
          let text_trimmed = text_content.trim();
          // 如果文本简短（少于50字符）且不包含列表特征（如多个换行、项目符号等），可能是标题
          if text_trimmed.len() < 50 && !text_trimmed.contains('\n') {
            tracing::debug!("Skipping <li> element that appears to be misidentified (text: '{}'). It will be handled by merged text logic.", text_trimmed);
            return None;
          }
        }
        
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
        tracing::info!("🔍 Processing table element");
        // 解析表格结构并创建AppFlowy表格块
        let mut table_rows = Vec::new();
        let mut max_cols = 0;
        
        // 先尝试解析 tbody > tr，如果没有则直接解析 tr
        let row_selector = scraper::Selector::parse("tr").unwrap();
        let tbody_selector = scraper::Selector::parse("tbody").unwrap();
        
        // 检查是否有 tbody
        let rows: Vec<_> = if element.select(&tbody_selector).next().is_some() {
          // 有 tbody，从 tbody 中解析行
          tracing::debug!("Table has tbody, parsing rows from tbody");
          element.select(&tbody_selector)
            .flat_map(|tbody| tbody.select(&row_selector))
            .collect()
        } else {
          // 没有 tbody，直接解析 tr
          tracing::debug!("Table has no tbody, parsing rows directly");
          element.select(&row_selector).collect()
        };
        
        tracing::info!("Found {} rows in table", rows.len());
        
        // 解析表格行
        for (row_idx, row_element) in rows.iter().enumerate() {
          let mut cells = Vec::new();
          
          // 解析表格单元格（包括 td 和 th）
          let cell_selector = scraper::Selector::parse("td, th").unwrap();
          for (cell_idx, cell_element) in row_element.select(&cell_selector).enumerate() {
            let cell_text = cell_element.text().collect::<String>().trim().to_string();
            let cell_text_preview = self.safe_string_preview(&cell_text, 50);
            cells.push(cell_text.clone());
            tracing::debug!("Row {}, Cell {}: '{}'", row_idx, cell_idx, cell_text_preview);
          }
          
          let cells_count = cells.len();
          if !cells.is_empty() {
            max_cols = max_cols.max(cells_count);
            table_rows.push(cells);
            tracing::debug!("Added row {} with {} cells", row_idx, cells_count);
          } else {
            tracing::warn!("Skipping empty row {}", row_idx);
          }
        }
        
        if table_rows.is_empty() {
          tracing::warn!("Table has no valid rows, skipping table conversion");
          // 输出表格的 HTML 以便调试
          let table_html = format!("{}", element.html());
          tracing::debug!("Table HTML (first 1000 chars): {}", 
              self.safe_string_preview(&table_html, 1000));
          return None;
        }
        
        tracing::info!("Processing table with {} rows, max {} columns", table_rows.len(), max_cols);
        
        // 确保所有行都有相同的列数
        for row in &mut table_rows {
          while row.len() < max_cols {
            row.push(String::new());
          }
        }
        
        // 创建表格行和单元格的children
        let mut row_children = Vec::new();
        for (_row_idx, row_cells) in table_rows.iter().enumerate() {
          let row_id = nanoid!(10);
          let mut cell_children = Vec::new();
          
          // 为每个单元格创建simple_table_cell块
          for (_col_idx, cell_text) in row_cells.iter().enumerate() {
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
        
        // 创建 src 的预览（避免打印完整的 base64 数据）
        let src_preview = if src.starts_with("data:") {
            format!("{}...", &src[..std::cmp::min(50, src.len())])
        } else {
            src.to_string()
        };
        tracing::info!("🖼️ Processing image element: src='{}' (length: {}), alt='{}'", 
            src_preview, src.len(), alt);
        
        // 确定图片 URL
        // 1. 如果 src 已经是 data URL（base64），直接使用
        // 2. 否则尝试从 image_map 中查找（使用图片 ID）
        // 3. 如果还是找不到，使用原始 src
        let image_url = if src.starts_with("data:") {
          // 已经是 base64 data URL，直接使用
          tracing::info!("Using data URL directly from src attribute");
          src.to_string()
        } else if let Some(data_url) = image_map.get(src) {
          // 从 image_map 中找到（通过文件路径或图片 ID）
          tracing::info!("Found image in image_map");
          data_url.clone()
        } else {
          // 如果没有找到，使用原始src（可能是相对路径或其他）
          tracing::warn!("Image not found in image_map, using original src: {}", src);
          src.to_string()
        };
        
        // 尝试从 base64 data URL 中读取图片尺寸
        // 使用轻量级方法只读取图片头部信息，避免加载整个图片导致栈溢出
        let (width, height) = if image_url.starts_with("data:image/") {
          // 尝试从 base64 数据中提取并读取尺寸
          if let Some(comma_pos) = image_url.find(',') {
            let base64_data = &image_url[comma_pos + 1..];
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(decoded) = general_purpose::STANDARD.decode(base64_data) {
              // 尝试检测图片格式
              let format = if decoded.len() >= 4 {
                match &decoded[0..4] {
                  [0x89, 0x50, 0x4E, 0x47] => flowy_document::import::ImageFormat::Png,
                  [0xFF, 0xD8, 0xFF, _] => flowy_document::import::ImageFormat::Jpeg,
                  [0x47, 0x49, 0x46, 0x38] => flowy_document::import::ImageFormat::Gif,
                  [0x42, 0x4D, _, _] => flowy_document::import::ImageFormat::Bmp,
                  [0x52, 0x49, 0x46, 0x46] if decoded.len() >= 12 && &decoded[8..12] == b"WEBP" => flowy_document::import::ImageFormat::WebP,
                  _ => flowy_document::import::ImageFormat::Png, // 默认
                }
              } else {
                flowy_document::import::ImageFormat::Png
              };
              
              match Self::get_image_dimensions_lightweight(&decoded, &format) {
                Some((w, h)) => (w, h),
                None => (300, 200)
              }
            } else {
              (300, 200)
            }
          } else {
            (300, 200)
          }
        } else {
          // 不是 base64 data URL，使用默认值
          (300, 200)
        };
        
        // 创建图片块
        let block = BlockPB {
          id: block_id,
          ty: "image".to_string(),
          data: serde_json::to_string(&serde_json::json!({
            "url": image_url,
            "width": width,
            "height": height,
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
            format!("{}...", &image_url[..std::cmp::min(50, image_url.len())])
        } else {
            image_url.clone()
        };
        // 也截断 src 用于日志显示
        let src_display = if src.starts_with("data:") {
            format!("{}...", &src[..std::cmp::min(50, src.len())])
        } else if src.len() > 100 {
            format!("{}...", &src[..100])
        } else {
            src.to_string()
        };
        tracing::info!("✅ Created image block: src='{}' -> url='{}'", src_display, display_url);
        Some(block)
      },
      "div" => {
        // 对于div元素，特别是包装div（如pdf-content），不要提取文本
        // 因为它的文本内容是所有子元素文本的合并，会导致重复
        // 如果div有class="pdf-content"，直接跳过，让子元素被处理
        if element.value().attr("class").map(|c| c.contains("pdf-content")).unwrap_or(false) {
          tracing::debug!("Skipping pdf-content wrapper div to avoid duplicate content");
          None
        } else {
          // 对于其他div，通常也是容器，应该跳过让其子元素处理
          // 只有在div确实有直接文本内容且没有子元素时才提取
          let has_direct_text = element.text().next().is_some();
          let has_child_elements = element.children().any(|child| child.value().is_element());
          if has_direct_text && !has_child_elements {
            // div有直接文本且没有子元素，可以提取
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
              
              tracing::debug!("Created div block with direct text: text='{}'", text_content.trim());
              Some(block)
            } else {
              None
            }
          } else {
            tracing::debug!("Skipping container div element to avoid duplicate content");
            None
          }
        }
      },
      "span" => {
        // 对于span，检查是否有文本内容
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
          
          tracing::debug!("Created span block: text='{}'", text_content.trim());
          Some(block)
        } else {
          tracing::debug!("Skipping empty span element");
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

  /// 从HTML中提取CSS字体大小信息
  fn extract_font_size_from_css(&self, html_content: &str, class_name: &str) -> Option<f64> {
    use regex::Regex;
    
    // 查找CSS类定义，例如 .ft12{font-size:33px;...}
    // 注意：class_name可能包含多个类名（如 "ft12 ft11"），需要提取第一个
    let first_class = class_name.split_whitespace().next().unwrap_or(class_name);
    let escaped_class = regex::escape(first_class);
    let css_regex = Regex::new(&format!(r"\.{}\{{[^}}]*font-size:\s*(\d+(?:\.\d+)?)px", escaped_class)).ok()?;
    
    if let Some(captures) = css_regex.captures(html_content) {
      if let Some(m) = captures.get(1) {
        return m.as_str().parse::<f64>().ok();
      }
    }
    
    // 如果CSS中没有找到，根据class名称推断
    // ft12 = 33px (标题), ft11/ft10 = 16px (正文)
    match first_class {
      "ft12" => Some(33.0),
      "ft11" | "ft10" => Some(16.0),
      _ => None,
    }
  }

  /// 合并绝对定位的元素，按位置分组重建段落结构
  /// 返回 (合并后的文本信息列表, 应该跳过的元素索引集合)
  fn merge_absolute_positioned_elements_in_html(&self, html_content: &str, elements: &[scraper::ElementRef]) -> (Vec<MergedTextInfo>, std::collections::HashSet<usize>) {
    use regex::Regex;
    
    // 收集需要合并的文本元素信息
    // (index, top, left, text, font_size, class)
    let mut positioned_texts: Vec<(usize, f64, f64, String, Option<f64>, Option<String>)> = Vec::new();
    let mut skip_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    
    let position_regex = Regex::new(r"position:\s*absolute").unwrap();
    let top_regex = Regex::new(r"top:\s*(\d+(?:\.\d+)?)px").unwrap();
    let left_regex = Regex::new(r"left:\s*(\d+(?:\.\d+)?)px").unwrap();
    let font_size_regex = Regex::new(r"font-size:\s*(\d+(?:\.\d+)?)px").unwrap();
    
    for (idx, element) in elements.iter().enumerate() {
      let tag_name = element.value().name();
      
      // 非文本元素（图片、表格等）直接保留，不合并
      if tag_name == "img" || tag_name == "table" || tag_name == "h1" || tag_name == "h2" || 
         tag_name == "h3" || tag_name == "h4" || tag_name == "h5" || tag_name == "h6" ||
         tag_name == "ul" || tag_name == "ol" || tag_name == "li" {
        continue;
      }
      
      // 检查是否有绝对定位样式
      if let Some(style) = element.value().attr("style") {
        if position_regex.is_match(style) {
          // 提取位置信息
          let top = top_regex.captures(style)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
          
          let left = left_regex.captures(style)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
          
          // 提取字体大小（首先从style属性中提取）
          let mut font_size = font_size_regex.captures(style)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok());
          
          // 获取class属性（用于识别字体样式，如ft12表示33px标题字体）
          let class_attr = element.value().attr("class").map(|s| s.to_string());
          
          // 如果style中没有字体大小，从CSS类中提取
          if font_size.is_none() {
            if let Some(ref class_name) = class_attr {
              font_size = self.extract_font_size_from_css(html_content, class_name);
            }
          }
          
          let raw_text = element.text().collect::<String>();
          
          // 清理文本：移除控制字符、乱码字符和特殊符号
          let text = self.clean_text_for_merging(&raw_text);
          
          // 如果清理后文本为空，跳过
          if text.trim().is_empty() {
            continue;
          }
          
          // 过滤掉只有单个标点符号的元素（如单独的逗号、句号等）
          // 但保留可能是有意义的单字符（如字母、数字）
          if text.len() > 1 || (text.len() == 1 && text.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false)) {
            positioned_texts.push((idx, top, left, text, font_size, class_attr));
            skip_indices.insert(idx);
          }
        }
      }
    }
    
    // 如果没有需要合并的元素，直接返回
    if positioned_texts.is_empty() {
      return (Vec::new(), skip_indices);
    }
    
    // 按 top 位置分组（相似 top 值的元素在同一行）
    // 使用容差来判断是否在同一行（例如，top 值相差小于 5px 认为是同一行）
    let line_tolerance = 5.0;
    // (top, [(left, text, font_size, class), ...])
    let mut lines: Vec<(f64, Vec<(f64, String, Option<f64>, Option<String>)>)> = Vec::new();
    
    // 按 top 排序
    positioned_texts.sort_by(|a, b| {
      a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // 分组到行
    let positioned_texts_count = positioned_texts.len();
    for (_idx, top, left, text, font_size, class) in positioned_texts {
      // 查找是否有相似 top 值的行
      let mut found_line = false;
      for (line_top, line_items) in &mut lines {
        if (top - *line_top).abs() < line_tolerance {
          // 添加到这一行，按 left 位置插入
          let insert_pos = line_items.binary_search_by(|(l, _, _, _)| {
            l.partial_cmp(&left).unwrap_or(std::cmp::Ordering::Equal)
          }).unwrap_or_else(|pos| pos);
          line_items.insert(insert_pos, (left, text.clone(), font_size, class.clone()));
          found_line = true;
          break;
        }
      }
      
      if !found_line {
        // 创建新行，使用第一个元素的top值作为行的top值
        lines.push((top, vec![(left, text, font_size, class)]));
      }
    }
    
    // 为每一行创建合并后的段落文本
    // 按top值排序，保持行的顺序
    lines.sort_by(|a, b| {
      a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    let mut merged_paragraphs = Vec::new();
    for (line_top, line_items) in lines {
      // 合并这一行的文本，按left位置排序后智能连接
      let mut merged_parts = Vec::new();
      let mut max_font_size: Option<f64> = None;
      let mut has_ft12_class = false;
      
      for (i, (left, text, font_size, class)) in line_items.iter().enumerate() {
        if i > 0 {
          merged_parts.push(" ");
        }
        merged_parts.push(text.as_str());
        
        // 检查字体大小和class
        if let Some(fs) = font_size {
          if max_font_size.is_none() || max_font_size.unwrap() < *fs {
            max_font_size = Some(*fs);
          }
        }
        if let Some(ref cls) = class {
          // ft12 表示33px标题字体
          if cls.contains("ft12") {
            has_ft12_class = true;
          }
        }
      }
      
      let merged_text = merged_parts.join("");
      
      // 在清理前检查是否包含列表符号（因为clean_text_for_merging可能会移除它们）
      let has_list_marker = merged_text.trim().starts_with("●") || 
                           merged_text.trim().starts_with("•") ||
                           merged_text.trim().starts_with("-") ||
                           merged_text.trim().starts_with("*") ||
                           merged_text.trim().starts_with("○") ||
                           merged_text.trim().starts_with("▪") ||
                           merged_text.trim().starts_with("▫");
      
      let cleaned_text = self.clean_text_for_merging(&merged_text);
      
      if !cleaned_text.trim().is_empty() {
        // 过滤掉"Document Outline"这种不需要的元素
        let text_lower = cleaned_text.to_lowercase();
        if text_lower.contains("document outline") {
          tracing::info!("Filtering out 'Document Outline' element");
          continue;
        }
        
        // 检查是否为列表项：包含列表符号
        // 注意：这里只检查列表符号，不检查特定文本内容
        let is_list_item = has_list_marker;
        
        // 判断是否为标题：字体大小>=30px 或 class包含ft12
        let is_heading = (has_ft12_class || max_font_size.map(|fs| fs >= 30.0).unwrap_or(false)) && !is_list_item;
        let heading_level = if is_heading {
          // 根据字体大小确定标题级别
          if max_font_size.map(|fs| fs >= 30.0).unwrap_or(false) {
            1 // H1
          } else {
            2 // H2
          }
        } else {
          0 // 普通段落
        };
        
        // 如果是列表项，移除列表符号
        let final_text = if is_list_item {
          cleaned_text.trim()
            .trim_start_matches("●")
            .trim_start_matches("•")
            .trim_start_matches("-")
            .trim_start_matches("*")
            .trim()
            .to_string()
        } else {
          cleaned_text.trim().to_string()
        };
        
        merged_paragraphs.push(MergedTextInfo {
          text: final_text,
          is_heading,
          heading_level,
          is_list_item,
          top: line_top,
        });
      }
    }
    
    tracing::info!("Merged {} positioned text fragments into {} paragraphs, will skip {} elements",
      positioned_texts_count, merged_paragraphs.len(), skip_indices.len());
    
    (merged_paragraphs, skip_indices)
  }
  
  /// 清理文本，移除控制字符、乱码字符和特殊符号
  fn clean_text_for_merging(&self, text: &str) -> String {
    text.chars()
      .filter(|ch| {
        // 移除控制字符（除了换行、回车、制表符）
        if ch.is_control() && *ch != '\n' && *ch != '\r' && *ch != '\t' {
          return false;
        }
        
        // 移除私有使用区域的字符（通常是乱码）
        let code = *ch as u32;
        if (0xE000..=0xF8FF).contains(&code) || // 私有使用区域
           (0xF0000..=0xFFFFF).contains(&code) || // 补充私有使用区域-A
           (0x100000..=0x10FFFF).contains(&code) { // 补充私有使用区域-B
          return false;
        }
        
        // 移除一些常见的乱码字符范围
        if (0x0080..=0x009F).contains(&code) { // 控制字符补充
          return false;
        }
        
        // 保留正常字符
        true
      })
      .collect::<String>()
      .trim()
      .to_string()
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
    _name: &str,
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
    name: &str,
    path: String,
  ) -> Result<(), FlowyError> {
    let file_path = Path::new(&path);
    let import_id = view_id.to_string();
    let file_name = name.to_string();
    
    // 在开始处理之前注册日志收集器，确保所有日志都能被收集
    if let Some(weak_folder) = &self.folder_manager {
      if let Some(_folder_manager) = weak_folder.upgrade() {
        use flowy_folder::manager::{ImportLogEntry, ImportProgress};
        // 克隆 Weak 引用以便在闭包中使用
        let weak_folder_clone = weak_folder.clone();
        let log_callback: flowy_document::import::import_log_collector::LogCallback = Box::new(move |id, level, message| {
          if let Some(fm) = weak_folder_clone.upgrade() {
            let log = match level {
              "error" => ImportLogEntry::error(message),
              "warn" => ImportLogEntry::warn(message),
              "debug" => ImportLogEntry::debug(message),
              _ => ImportLogEntry::info(message),
            };
            fm.add_import_log(id, log);
          }
        });
        let weak_folder_clone2 = weak_folder.clone();
        let progress_callback: flowy_document::import::import_log_collector::ProgressCallback = Box::new(move |id, file_name, progress, step| {
          if let Some(fm) = weak_folder_clone2.upgrade() {
            fm.send_import_progress(ImportProgress::new(
              id.to_string(),
              file_name.to_string(),
              progress,
              step.to_string(),
            ));
          }
        });
        flowy_document::import::import_log_collector::register_import_log_collector(
          import_id.clone(),
          Arc::new(log_callback),
          Arc::new(progress_callback),
        );
      }
    }
    
    // 检查文件扩展名以确定导入类型
    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
      match ext.to_lowercase().as_str() {
        "docx" | "doc" | "pdf" => {
          // 对于Word和PDF文件，读取文件内容并调用import_from_bytes
          self.send_progress(&import_id, &file_name, 0.1, "正在读取文件...");
          info!("Reading {} file content for conversion", ext);
          
          let bytes = tokio::fs::read(&file_path).await
            .map_err(|e| {
              self.send_progress_error(&import_id, &file_name, &format!("读取文件失败: {}", e));
              FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to read file: {}", e),
              )
            })?;
          
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

          // 保存 document_type 的副本以便后续使用
          let is_pdf = document_type == DocumentType::Pdf;

          self.send_progress(&import_id, &file_name, 0.2, "正在初始化转换器...");
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
          )
          .with_import_id(import_id.clone());

          // 发送转换进度（针对 PDF）
          if is_pdf {
            // 注意：这里不发送进度，因为 MarkerPdfConverter 内部会发送更详细的进度
            // 只发送一个初始日志
            self.add_import_log(&import_id, "info", "开始使用 Marker 工具转换 PDF");
          } else {
            self.send_progress(&import_id, &file_name, 0.3, "正在转换文档...");
          }

          let result = converter.convert(&task).await
            .map_err(|e| {
              self.send_progress_error(&import_id, &file_name, &format!("转换失败: {}", e));
              e
            })?;

          // 对于 PDF，Marker 工具转换完成后，进度应该在 70% 左右
          // 这里更新进度到 75%，表示开始解析格式
          if is_pdf {
            self.send_progress(&import_id, &file_name, 0.75, "正在解析格式...");
            self.add_import_log(&import_id, "info", "开始解析转换后的 Markdown 格式");
          } else {
            self.send_progress(&import_id, &file_name, 0.7, "正在解析格式...");
          }
          
          // 将重计算部分移到 spawn_blocking 中执行，避免栈溢出
          // 注意：由于 convert_result_to_document_data_with_progress 需要访问 self 的方法，
          // 我们暂时保持原样，但可以考虑在后续优化中将解析逻辑提取为独立函数
          let document_data = self.convert_result_to_document_data_with_progress(
            &result,
            &import_id,
            &file_name,
          )
            .map_err(|e| {
              self.send_progress_error(&import_id, &file_name, &format!("解析格式失败: {}", e));
              e
            })?;

          if is_pdf {
            self.send_progress(&import_id, &file_name, 0.9, "正在创建文档...");
            self.add_import_log(&import_id, "info", "格式解析完成，开始创建文档");
          } else {
            self.send_progress(&import_id, &file_name, 0.9, "正在创建文档...");
          }
          // 注意：create_document 内部已经使用 spawn_blocking 来避免栈溢出
          tracing::info!("[IMPORT_SINGLE_FILE - EVENT] 开始创建文档，blocks数量: {}", document_data.blocks.len());
          let doc_manager = self.document_manager()?;
          
          // 尝试创建文档，如果文档已存在（可能由 create_view 创建了默认文档），则先删除再创建
          // 将 DocumentDataPB 转换为 DocumentData
          let document_data_internal: DocumentData = document_data.into();
          let create_result = doc_manager
            .create_document(0, &view_id_uuid, Some(document_data_internal))
            .await;
          
          match create_result {
            Ok(_) => {
              tracing::info!("[IMPORT_SINGLE_FILE - EVENT] 文档创建成功");
            }
            Err(e) if e.code == flowy_error::ErrorCode::RecordAlreadyExists => {
              // 文档已存在（可能是 create_view 创建的默认文档），删除后重新转换并创建
              tracing::info!("[IMPORT_SINGLE_FILE - EVENT] 文档已存在，删除默认文档以便重新创建");
              self.add_import_log(&import_id, "info", "检测到文档已存在，删除默认文档");
              
              if let Err(delete_err) = doc_manager.delete_document(&view_id_uuid).await {
                tracing::warn!("[IMPORT_SINGLE_FILE - EVENT] 删除现有文档失败: {}", delete_err);
                let error_msg = format!("删除现有文档失败: {}", delete_err);
                self.add_import_log(&import_id, "warn", &error_msg);
              }
              
              // 重新从 result 转换 document_data 并创建文档
              // 注意：这里会重复处理一次，但只在文档已存在时发生，且可以确保数据一致性
              let document_data = self.convert_result_to_document_data_with_progress(
                &result,
                &import_id,
                &file_name,
              )
                .map_err(|e| {
                  tracing::error!("[IMPORT_SINGLE_FILE - EVENT] 重新解析格式失败: {}", e);
                  self.send_progress_error(&import_id, &file_name, &format!("重新解析格式失败: {}", e));
                  e
                })?;
              let document_data_internal: DocumentData = document_data.into();
              doc_manager
                .create_document(0, &view_id_uuid, Some(document_data_internal))
                .await
                .map_err(|e| {
                  tracing::error!("[IMPORT_SINGLE_FILE - EVENT] 重新创建文档失败: {}", e);
                  self.send_progress_error(&import_id, &file_name, &format!("创建文档失败: {}", e));
                  e
                })?;
              tracing::info!("[IMPORT_SINGLE_FILE - EVENT] 文档重新创建成功");
            }
            Err(e) => {
              tracing::error!("[IMPORT_SINGLE_FILE - EVENT] 创建文档失败: {}", e);
              self.send_progress_error(&import_id, &file_name, &format!("创建文档失败: {}", e));
              return Err(e);
            }
          }
          
          self.send_progress(&import_id, &file_name, 1.0, "导入完成");
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

