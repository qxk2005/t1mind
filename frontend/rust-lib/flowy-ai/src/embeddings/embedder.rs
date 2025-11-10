use crate::embeddings::indexer::EmbeddingModel;
use flowy_error::{FlowyError, FlowyResult};
use ollama_rs::Ollama;
use ollama_rs::generation::embeddings::GenerateEmbeddingsResponse;
use ollama_rs::generation::embeddings::request::{GenerateEmbeddingsRequest, EmbeddingsInput};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

#[derive(Debug, Clone)]
pub enum Embedder {
  Ollama(OllamaEmbedder),
  OpenAI(OpenAIEmbedder),
}

impl Embedder {
  pub async fn embed(
    &self,
    request: GenerateEmbeddingsRequest,
  ) -> FlowyResult<GenerateEmbeddingsResponse> {
    match self {
      Embedder::Ollama(ollama) => ollama.embed(request).await,
      Embedder::OpenAI(_) => {
        // OpenAI embedder 不应该通过这个方法调用
        // 应该使用 embed_texts 方法并传入文本列表
        Err(FlowyError::internal().with_context(
          "OpenAI embedder requires direct text input. Use embed_texts instead."
        ))
      }
    }
  }
  
  /// 从文本列表生成嵌入（支持 OpenAI 和 Ollama）
  pub async fn embed_texts(&self, texts: Vec<String>) -> FlowyResult<Vec<Vec<f32>>> {
    match self {
      Embedder::Ollama(ollama) => {
        let request = GenerateEmbeddingsRequest::new(
          "nomic-embed-text:latest".to_string(),
          EmbeddingsInput::Multiple(texts),
        );
        let resp = ollama.embed(request).await?;
        Ok(resp.embeddings.into_iter().map(|v| v.into_iter().map(|f| f as f32).collect()).collect())
      }
      Embedder::OpenAI(openai) => {
        openai.embed_texts(texts).await
      }
    }
  }

  pub fn model(&self) -> EmbeddingModel {
    EmbeddingModel::NomicEmbedText
  }
}

#[derive(Debug, Clone)]
pub struct OllamaEmbedder {
  pub ollama: Arc<Ollama>,
}

impl OllamaEmbedder {
  pub async fn embed(
    &self,
    request: GenerateEmbeddingsRequest,
  ) -> FlowyResult<GenerateEmbeddingsResponse> {
    let resp = self.ollama.generate_embeddings(request).await?;
    Ok(resp)
  }
}

#[derive(Debug, Clone)]
pub struct OpenAIEmbedder {
  pub base_url: String,
  pub api_key: String,
  pub model: String,
}

#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
  model: String,
  input: Vec<String>,
  encoding_format: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
  data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
  embedding: Vec<f32>,
  index: usize,
}

impl OpenAIEmbedder {
  /// 从文本列表生成嵌入
  pub async fn embed_texts(&self, input_texts: Vec<String>) -> FlowyResult<Vec<Vec<f32>>> {
    trace!(
      "[Embedding] 📤 使用 OpenAI 兼容服务器生成嵌入: {} 个文本片段",
      input_texts.len()
    );

    // 构建 OpenAI API 请求
    let openai_request = OpenAIEmbeddingRequest {
      model: self.model.clone(),
      input: input_texts,
      encoding_format: "float".to_string(),
    };

    // 发送请求到 OpenAI 兼容服务器
    // 尝试标准 OpenAI 路径 /v1/embeddings
    let url = if self.base_url.contains("/v1") {
      // 如果 base_url 已经包含 /v1，直接添加 /embeddings
      if self.base_url.ends_with('/') {
        format!("{}embeddings", self.base_url.trim_end_matches('/'))
      } else {
        format!("{}/embeddings", self.base_url)
      }
    } else {
      // 否则添加 /v1/embeddings
      if self.base_url.ends_with('/') {
        format!("{}v1/embeddings", self.base_url.trim_end_matches('/'))
      } else {
        format!("{}/v1/embeddings", self.base_url)
      }
    };
    
    // 请求体和响应体可能很长，不打印完整内容以避免日志过长
    trace!("[Embedding] 🌐 请求 URL: {}", url);
    trace!("[Embedding] 📤 请求模型: {}, 文本片段数: {}", openai_request.model, openai_request.input.len());

    let client = reqwest::Client::new();
    let response = client
      .post(&url)
      .header("Authorization", format!("Bearer {}", self.api_key))
      .header("Content-Type", "application/json")
      .json(&openai_request)
      .send()
      .await
      .map_err(|e| {
        error!("[Embedding] ❌ OpenAI 兼容服务器请求失败: {}", e);
        FlowyError::internal().with_context(format!("嵌入请求失败: {}", e))
      })?;

    if !response.status().is_success() {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_default();
      error!(
        "[Embedding] ❌ OpenAI 兼容服务器返回错误: {} - {}",
        status, error_text
      );
      return Err(
        FlowyError::internal()
          .with_context(format!("嵌入服务返回错误 {}: {}", status, error_text)),
      );
    }

    // 先读取响应文本，以便在解析失败时打印日志
    let response_text = response.text().await.map_err(|e| {
      error!("[Embedding] ❌ 读取响应失败: {}", e);
      FlowyError::internal().with_context(format!("读取嵌入响应失败: {}", e))
    })?;
    
    // 响应体可能很长（包含大量嵌入向量），不打印完整内容以避免日志过长
    trace!("[Embedding] 📥 收到响应，长度: {} 字符", response_text.len());
    
    let openai_response: OpenAIEmbeddingResponse = serde_json::from_str(&response_text).map_err(|e| {
      error!("[Embedding] ❌ 解析 OpenAI 响应失败: {}", e);
      error!("[Embedding] ❌ 响应内容: {}", response_text);
      FlowyError::internal().with_context(format!("解析嵌入响应失败: {}", e))
    })?;

    trace!(
      "[Embedding] ✅ OpenAI 兼容服务器返回 {} 个嵌入向量",
      openai_response.data.len()
    );

    // 返回嵌入向量
    let embeddings: Vec<Vec<f32>> = openai_response
      .data
      .into_iter()
      .map(|d| d.embedding)
      .collect();

    Ok(embeddings)
  }
}
