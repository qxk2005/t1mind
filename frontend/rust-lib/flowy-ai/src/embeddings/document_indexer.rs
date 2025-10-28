use crate::embeddings::embedder::Embedder;
use crate::embeddings::indexer::{EmbeddingModel, Indexer};
use crate::rag::RAGConfigManager;
use flowy_ai_pub::entities::{EmbeddedChunk, SOURCE, SOURCE_ID, SOURCE_NAME};
use flowy_error::FlowyError;
use lib_infra::async_trait::async_trait;
use serde_json::json;
use std::sync::{Arc, Weak};
use text_splitter::{ChunkConfig, TextSplitter};
use tracing::{debug, error, info, trace, warn};
use twox_hash::xxhash64::Hasher;
use uuid::Uuid;

pub struct DocumentIndexer {
  rag_config: Option<Weak<RAGConfigManager>>,
}

impl DocumentIndexer {
  pub fn new(rag_config: Option<Weak<RAGConfigManager>>) -> Self {
    Self { rag_config }
  }
}

#[async_trait]
impl Indexer for DocumentIndexer {
  fn create_embedded_chunks_from_text(
    &self,
    object_id: Uuid,
    paragraphs: Vec<String>,
    model: EmbeddingModel,
  ) -> Result<Vec<EmbeddedChunk>, FlowyError> {
    if paragraphs.is_empty() {
      // 将警告级别降低为调试信息，因为这是正常情况
      debug!(
        "[Embedding] Document `{}` has no text content to index. This may be normal for documents with only images, tables, or other non-text elements.",
        object_id
      );

      return Ok(vec![]);
    }
    
    // 过滤掉空段落和只包含空白字符的段落
    let filtered_paragraphs: Vec<String> = paragraphs
      .into_iter()
      .filter(|p| !p.trim().is_empty())
      .collect();
    
    if filtered_paragraphs.is_empty() {
      debug!(
        "[Embedding] Document `{}` has no meaningful text content after filtering empty paragraphs.",
        object_id
      );
      return Ok(vec![]);
    }
    
    trace!(
      "[Embedding] Processing document `{}` with {} paragraphs",
      object_id,
      filtered_paragraphs.len()
    );
    
    // 尝试从配置中读取 chunk_size、chunk_overlap 和 semantic splitting 设置
    let (chunk_size, overlap, enable_semantic) = if let Some(ref weak_config) = self.rag_config {
      if let Some(config) = weak_config.upgrade() {
        let settings = config.get_rag_settings();
        info!(
          "[Embedding] Using configured chunk_size={}, chunk_overlap={}, enable_semantic_splitting={}",
          settings.chunk_size, settings.chunk_overlap, settings.enable_semantic_splitting
        );
        (settings.chunk_size as usize, settings.chunk_overlap as usize, settings.enable_semantic_splitting)
      } else {
        warn!("[Embedding] RAG config dropped, using defaults: chunk_size=1000, overlap=200, enable_semantic=false");
        (1000, 200, false)
      }
    } else {
      trace!("[Embedding] No RAG config available, using defaults: chunk_size=1000, overlap=200, enable_semantic=false");
      (1000, 200, false)
    };
    
    // 根据配置使用不同的切片策略
    if enable_semantic {
      info!("[Embedding] 🧠 Using semantic splitting with TextSplitter");
      split_text_into_chunks_with_semantic(&object_id.to_string(), filtered_paragraphs, model, chunk_size, overlap)
    } else {
      info!("[Embedding] 📏 Using character-based splitting");
      split_text_into_chunks(&object_id.to_string(), filtered_paragraphs, model, chunk_size, overlap)
    }
  }

  async fn embed(
    &self,
    embedder: &Embedder,
    mut chunks: Vec<EmbeddedChunk>,
  ) -> Result<Vec<EmbeddedChunk>, FlowyError> {
    let mut valid_indices = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
      if let Some(ref content) = chunk.content {
        if !content.is_empty() {
          valid_indices.push(i);
        }
      }
    }

    if valid_indices.is_empty() {
      return Ok(vec![]);
    }

    let mut contents = Vec::with_capacity(valid_indices.len());
    for &i in &valid_indices {
      contents.push(chunks[i].content.as_ref().unwrap().to_owned());
    }

    // 使用新的 embed_texts 方法（支持 OpenAI 和 Ollama）
    let embeddings = embedder.embed_texts(contents).await?;
    
    if embeddings.len() != valid_indices.len() {
      error!(
        "[Embedding] requested {} embeddings, received {} embeddings",
        valid_indices.len(),
        embeddings.len()
      );
      return Err(FlowyError::internal().with_context(format!(
        "Mismatch in number of embeddings requested and received: {} vs {}",
        valid_indices.len(),
        embeddings.len()
      )));
    }

    for (index, embedding) in embeddings.into_iter().enumerate() {
      let chunk_idx = valid_indices[index];
      // 直接使用 Vec<f32>（无需转换）
      chunks[chunk_idx].embeddings = Some(embedding);
    }

    Ok(chunks)
  }
}

/// chunk_size:
/// Small Chunks (50–256 tokens): Best for precision-focused tasks (e.g., Q&A, technical docs) where specific details matter.
/// Medium Chunks (256–1,024 tokens): Ideal for balanced tasks like RAG or contextual search, providing enough context without noise.
/// Large Chunks (1,024–2,048 tokens): Suited for analysis or thematic tasks where broad understanding is key.
///
/// overlap:
/// Add 10–20% overlap for larger chunks (e.g., 50–100 tokens for 512-token chunks) to preserve context across boundaries.
pub fn split_text_into_chunks(
  object_id: &str,
  paragraphs: Vec<String>,
  embedding_model: EmbeddingModel,
  chunk_size: usize,
  overlap: usize,
) -> Result<Vec<EmbeddedChunk>, FlowyError> {
  debug_assert!(matches!(embedding_model, EmbeddingModel::NomicEmbedText));

  if paragraphs.is_empty() {
    return Ok(vec![]);
  }
  let split_contents = group_paragraphs_by_max_content_len(paragraphs, chunk_size, overlap);
  let metadata = json!({
      SOURCE_ID: object_id,
      SOURCE: "appflowy",
      SOURCE_NAME: "document",
  });

  let mut seen = std::collections::HashSet::new();
  let mut chunks = Vec::new();

  for (index, content) in split_contents.into_iter().enumerate() {
    let metadata_string = metadata.to_string();
    let combined_data = format!("{}{}", content, metadata_string);
    let consistent_hash = Hasher::oneshot(0, combined_data.as_bytes());
    let fragment_id = format!("{:x}", consistent_hash);
    if seen.insert(fragment_id.clone()) {
      chunks.push(EmbeddedChunk {
        fragment_id,
        object_id: object_id.to_string(),
        content_type: 0,
        content: Some(content),
        embeddings: None,
        metadata: Some(metadata_string),
        fragment_index: index as i32,
        embedder_type: 0,
      });
    } else {
      debug!(
        "[Embedding] Duplicate fragment_id detected: {}. This fragment will not be added.",
        fragment_id
      );
    }
  }

  info!(
    "[Embedding] ✅ Created {} chunks for object_id `{}`, chunk_size: {}, overlap: {}",
    chunks.len(),
    object_id,
    chunk_size,
    overlap
  );
  
  // 显示前3个 chunk 的预览，帮助用户了解切片情况
  if !chunks.is_empty() {
    info!(
      "[Embedding] 📄 Preview of first chunk (length: {} chars): '{}'",
      chunks[0].content.as_ref().map(|c| c.len()).unwrap_or(0),
      chunks[0].content.as_ref()
        .map(|c| c.chars().take(100).collect::<String>())
        .unwrap_or_else(|| "empty".to_string())
    );
    if chunks.len() > 1 {
      info!(
        "[Embedding] 📄 Preview of last chunk (index {}): '{}'",
        chunks.len() - 1,
        chunks.last().unwrap().content.as_ref()
          .map(|c| c.chars().take(100).collect::<String>())
          .unwrap_or_else(|| "empty".to_string())
      );
    }
  }
  
  Ok(chunks)
}

pub fn group_paragraphs_by_max_content_len(
  paragraphs: Vec<String>,
  mut context_size: usize,
  overlap: usize,
) -> Vec<String> {
  if paragraphs.is_empty() {
    return vec![];
  }

  let mut result = Vec::new();
  let mut current = String::with_capacity(context_size.min(4096));

  if overlap > context_size {
    warn!("context_size is smaller than overlap, which may lead to unexpected behavior.");
    context_size = 2 * overlap;
  }

  let chunk_config = ChunkConfig::new(context_size)
    .with_overlap(overlap)
    .unwrap();
  let splitter = TextSplitter::new(chunk_config);

  for paragraph in paragraphs {
    if current.len() + paragraph.len() > context_size {
      if !current.is_empty() {
        result.push(std::mem::take(&mut current));
      }

      if paragraph.len() > context_size {
        // Use TextSplitter for proper text splitting
        let paragraph_chunks = splitter.chunks(&paragraph);
        result.extend(paragraph_chunks.map(String::from));
      } else {
        current.push_str(&paragraph);
      }
    } else {
      // Add paragraph to current chunk
      current.push_str(&paragraph);
    }
  }

  if !current.is_empty() {
    result.push(current);
  }

  result
}

/// 使用纯语义分割的文本切片函数
/// 
/// 这个函数完全依赖 TextSplitter 进行语义感知的文本分割，
/// 而不是简单的字符计数方式。
pub fn split_text_into_chunks_with_semantic(
  object_id: &str,
  paragraphs: Vec<String>,
  embedding_model: EmbeddingModel,
  chunk_size: usize,
  overlap: usize,
) -> Result<Vec<EmbeddedChunk>, FlowyError> {
  debug_assert!(matches!(embedding_model, EmbeddingModel::NomicEmbedText));

  if paragraphs.is_empty() {
    return Ok(vec![]);
  }

  info!("[Embedding] 🧠 Semantic splitting: processing {} paragraphs with chunk_size={}, overlap={}", 
        paragraphs.len(), chunk_size, overlap);

  let mut split_contents = Vec::new();

  // 配置 TextSplitter
  let chunk_config = ChunkConfig::new(chunk_size)
    .with_overlap(overlap)
    .unwrap();
  let splitter = TextSplitter::new(chunk_config);

  // 将段落合并为完整文本
  let full_text = paragraphs.join("\n\n");
  
  // 使用 TextSplitter 进行语义分割
  let chunks = splitter.chunks(&full_text);
  for chunk in chunks {
    split_contents.push(chunk.to_string());
  }

  info!("[Embedding] 🧠 Semantic splitting: created {} semantic chunks", split_contents.len());

  let metadata = json!({
      SOURCE_ID: object_id,
      SOURCE: "appflowy",
      SOURCE_NAME: "document",
  });

  let mut seen = std::collections::HashSet::new();
  let mut embedded_chunks = Vec::new();

  for (index, content) in split_contents.into_iter().enumerate() {
    let metadata_string = metadata.to_string();
    let combined_data = format!("{}{}", content, metadata_string);
    let consistent_hash = Hasher::oneshot(0, combined_data.as_bytes());
    let fragment_id = format!("{:x}", consistent_hash);
    
    if seen.insert(fragment_id.clone()) {
      embedded_chunks.push(EmbeddedChunk {
        fragment_id,
        object_id: object_id.to_string(),
        content_type: 0,
        content: Some(content),
        embeddings: None,
        metadata: Some(metadata_string),
        fragment_index: index as i32,
        embedder_type: 0,
      });
    } else {
      debug!(
        "[Embedding] 🧠 Semantic: Duplicate fragment_id detected: {}. This fragment will not be added.",
        fragment_id
      );
    }
  }

  info!(
    "[Embedding] ✅ Semantic splitting: Created {} chunks for object_id `{}`, chunk_size: {}, overlap: {}",
    embedded_chunks.len(),
    object_id,
    chunk_size,
    overlap
  );
  
  // 显示第一个和最后一个 chunk 的预览
  if !embedded_chunks.is_empty() {
    info!(
      "[Embedding] 📄 Semantic first chunk (length: {} chars): '{}'",
      embedded_chunks[0].content.as_ref().map(|c| c.len()).unwrap_or(0),
      embedded_chunks[0].content.as_ref()
        .map(|c| c.chars().take(100).collect::<String>())
        .unwrap_or_else(|| "empty".to_string())
    );
    if embedded_chunks.len() > 1 {
      info!(
        "[Embedding] 📄 Semantic last chunk (index {}, length: {} chars): '{}'",
        embedded_chunks.len() - 1,
        embedded_chunks.last().unwrap().content.as_ref().map(|c| c.len()).unwrap_or(0),
        embedded_chunks.last().unwrap().content.as_ref()
          .map(|c| c.chars().take(100).collect::<String>())
          .unwrap_or_else(|| "empty".to_string())
      );
    }
  }
  
  Ok(embedded_chunks)
}