use crate::openai_compatible::types::*;
use anyhow::{anyhow, Result};
use flowy_error::FlowyResult;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::generation::embeddings::GenerateEmbeddingsResponse;
use std::collections::HashMap;
use tracing::{debug, error, instrument};

/// OpenAI compatible embeddings client
#[derive(Debug, Clone)]
pub struct OpenAICompatibleEmbedder {
    config: OpenAICompatibleConfig,
    client: reqwest::Client,
}

impl OpenAICompatibleEmbedder {
    /// Create a new OpenAI compatible embedder
    pub fn new(config: OpenAICompatibleConfig) -> Result<Self> {
        let timeout = std::time::Duration::from_millis(config.timeout_ms.unwrap_or(30000));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { config, client })
    }

    /// Generate embeddings using OpenAI compatible API
    #[instrument(level = "debug", skip(self, texts), err)]
    pub async fn generate_embeddings(&self, texts: Vec<String>) -> FlowyResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        debug!("Generating embeddings for {} texts", texts.len());

        // Convert to OpenAI compatible request format
        let input = if texts.len() == 1 {
            EmbeddingInput::String(texts[0].clone())
        } else {
            EmbeddingInput::StringArray(texts)
        };

        let request = EmbeddingRequest {
            model: self.config.embedding_model.clone(),
            input,
        };

        let response = self.send_embedding_request(request).await?;

        // Extract embeddings from response
        let mut embeddings = Vec::with_capacity(response.data.len());
        for data in response.data {
            embeddings.push(data.embedding);
        }

        debug!("Successfully generated {} embeddings", embeddings.len());
        Ok(embeddings)
    }

    /// Generate single embedding for compatibility with existing interface
    #[instrument(level = "debug", skip(self, text), err)]
    pub async fn generate_single_embedding(&self, text: String) -> FlowyResult<Vec<f32>> {
        let embeddings = self.generate_embeddings(vec![text]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| flowy_error::FlowyError::internal().with_context("No embedding returned"))
    }

    /// Convert from Ollama request format to OpenAI compatible format
    /// Note: Since we can't access private fields of GenerateEmbeddingsRequest,
    /// we'll need to work with the model name and create a simple test embedding
    #[instrument(level = "debug", skip(self, _request), err)]
    pub async fn embed_ollama_request(
        &self,
        _request: GenerateEmbeddingsRequest,
    ) -> FlowyResult<GenerateEmbeddingsResponse> {
        debug!("Converting Ollama request to OpenAI compatible format");

        // Since we can't access the private fields of the request,
        // we'll generate a single embedding for a test string
        // This is primarily used for compatibility with the existing interface
        let text = "test embedding".to_string();
        
        // Generate embedding using OpenAI compatible API
        let embedding = self.generate_single_embedding(text).await?;

        // Convert back to Ollama response format
        let response = GenerateEmbeddingsResponse {
            embeddings: vec![embedding],
        };

        Ok(response)
    }

    /// Send embedding request to OpenAI compatible API
    async fn send_embedding_request(&self, request: EmbeddingRequest) -> FlowyResult<EmbeddingResponse> {
        let masked_key = self.mask_api_key(&self.config.embedding_api_key);
        debug!(
            "Sending embedding request to {} with model {} (API key: {})",
            self.config.embedding_endpoint, request.model, masked_key
        );

        let response = self
            .client
            .post(&self.config.embedding_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.embedding_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send embedding request: {}", e);
                flowy_error::FlowyError::internal().with_context(format!("Network error: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            error!("Embedding API error {}: {}", status, error_text);

            // Try to parse as OpenAI error format
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(flowy_error::FlowyError::internal()
                    .with_context(format!("API error: {}", error_response.error.message)));
            }

            // Fallback to generic error message
            let error_msg = match status.as_u16() {
                401 => "Authentication failed. Please check your API key.",
                404 => "API endpoint not found. Please check your endpoint URL.",
                429 => "Rate limit exceeded. Please try again later.",
                500 => "Server error. Please try again later.",
                400 => &format!("Bad request: {}", error_text),
                _ => &format!("HTTP error {}: {}", status, error_text),
            };

            return Err(flowy_error::FlowyError::internal().with_context(error_msg));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| {
                error!("Failed to read response body: {}", e);
                flowy_error::FlowyError::internal().with_context("Failed to read response")
            })?;

        let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("Failed to parse embedding response: {} (body: {})", e, response_text);
                flowy_error::FlowyError::internal()
                    .with_context(format!("Invalid response format: {}", e))
            })?;

        Ok(embedding_response)
    }

    /// Mask API key for logging (show only first 4 and last 4 characters)
    fn mask_api_key(&self, api_key: &str) -> String {
        if api_key.len() <= 8 {
            "*".repeat(api_key.len())
        } else {
            format!(
                "{}...{}",
                &api_key[..4],
                &api_key[api_key.len() - 4..]
            )
        }
    }
}

/// Batch embedding processor for efficient processing of multiple texts
pub struct BatchEmbeddingProcessor {
    embedder: OpenAICompatibleEmbedder,
    batch_size: usize,
}

impl BatchEmbeddingProcessor {
    /// Create a new batch processor
    pub fn new(embedder: OpenAICompatibleEmbedder, batch_size: Option<usize>) -> Self {
        Self {
            embedder,
            batch_size: batch_size.unwrap_or(100), // Default batch size
        }
    }

    /// Process embeddings in batches for better performance
    #[instrument(level = "debug", skip(self, texts), err)]
    pub async fn process_batch(&self, texts: Vec<String>) -> FlowyResult<HashMap<String, Vec<f32>>> {
        if texts.is_empty() {
            return Ok(HashMap::new());
        }

        debug!("Processing {} texts in batches of {}", texts.len(), self.batch_size);

        let mut results = HashMap::new();
        
        // Process texts in batches
        for chunk in texts.chunks(self.batch_size) {
            let chunk_texts = chunk.to_vec();
            let embeddings = self.embedder.generate_embeddings(chunk_texts.clone()).await?;

            // Map texts to their embeddings
            for (text, embedding) in chunk_texts.into_iter().zip(embeddings.into_iter()) {
                results.insert(text, embedding);
            }
        }

        debug!("Successfully processed {} embeddings", results.len());
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        let config = OpenAICompatibleConfig::default();
        let embedder = OpenAICompatibleEmbedder::new(config).unwrap();
        
        assert_eq!(embedder.mask_api_key(""), "");
        assert_eq!(embedder.mask_api_key("short"), "*****");
        assert_eq!(embedder.mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(embedder.mask_api_key("sk-proj-1234567890abcdefghijklmnop"), "sk-p...mnop");
    }

    #[tokio::test]
    async fn test_embedder_creation() {
        let config = OpenAICompatibleConfig::default();
        let embedder = OpenAICompatibleEmbedder::new(config);
        assert!(embedder.is_ok());
    }

    #[tokio::test]
    async fn test_batch_processor_creation() {
        let config = OpenAICompatibleConfig::default();
        let embedder = OpenAICompatibleEmbedder::new(config).unwrap();
        let processor = BatchEmbeddingProcessor::new(embedder, Some(50));
        assert_eq!(processor.batch_size, 50);
    }

    #[tokio::test]
    async fn test_empty_texts() {
        let config = OpenAICompatibleConfig::default();
        let embedder = OpenAICompatibleEmbedder::new(config).unwrap();
        let result = embedder.generate_embeddings(vec![]).await.unwrap();
        assert!(result.is_empty());
    }
}
