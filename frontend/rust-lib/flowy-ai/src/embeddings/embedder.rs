use crate::embeddings::indexer::EmbeddingModel;
use crate::openai_compatible::{OpenAICompatibleConfig, OpenAICompatibleEmbedder};
use flowy_error::FlowyResult;
use ollama_rs::Ollama;
use ollama_rs::generation::embeddings::GenerateEmbeddingsResponse;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Embedder {
  Ollama(OllamaEmbedder),
  OpenAICompatible(OpenAICompatibleEmbedder),
}

impl Embedder {
  /// Create a new Ollama embedder
  pub fn new_ollama(ollama: Arc<Ollama>) -> Self {
    Embedder::Ollama(OllamaEmbedder { ollama })
  }

  /// Create a new OpenAI compatible embedder
  pub fn new_openai_compatible(config: OpenAICompatibleConfig) -> FlowyResult<Self> {
    let embedder = OpenAICompatibleEmbedder::new(config)
      .map_err(|e| flowy_error::FlowyError::internal().with_context(format!("Failed to create OpenAI compatible embedder: {}", e)))?;
    Ok(Embedder::OpenAICompatible(embedder))
  }

  pub async fn embed(
    &self,
    request: GenerateEmbeddingsRequest,
  ) -> FlowyResult<GenerateEmbeddingsResponse> {
    match self {
      Embedder::Ollama(ollama) => ollama.embed(request).await,
      Embedder::OpenAICompatible(openai) => openai.embed_ollama_request(request).await,
    }
  }

  pub fn model(&self) -> EmbeddingModel {
    match self {
      Embedder::Ollama(_) => EmbeddingModel::NomicEmbedText,
      Embedder::OpenAICompatible(_) => EmbeddingModel::OpenAICompatible,
    }
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
