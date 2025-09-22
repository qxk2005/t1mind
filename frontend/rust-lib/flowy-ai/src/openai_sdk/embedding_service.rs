use crate::entities::*;
use crate::openai_compatible::types::{EmbeddingInput, EmbeddingRequest, EmbeddingResponse, ErrorResponse};
use anyhow::{anyhow, Result};
use flowy_error::{FlowyError, FlowyResult};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, instrument};

/// OpenAI SDK embedding service
#[derive(Debug, Clone)]
pub struct OpenAISDKEmbeddingService {
    client: Client,
    config: OpenAISDKEmbeddingConfig,
}

/// Internal configuration for OpenAI SDK embedding service
#[derive(Debug, Clone)]
pub struct OpenAISDKEmbeddingConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub model_name: String,
}

/// Embedding cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    embedding: Vec<f32>,
    timestamp: Instant,
}

/// Embedding cache for optimization
#[derive(Debug)]
pub struct EmbeddingCache {
    cache: HashMap<String, CacheEntry>,
    max_size: usize,
    ttl: Duration,
}

impl From<OpenAISDKEmbeddingSettingPB> for OpenAISDKEmbeddingConfig {
    fn from(setting: OpenAISDKEmbeddingSettingPB) -> Self {
        Self {
            api_endpoint: setting.api_endpoint,
            api_key: setting.api_key,
            model_name: setting.model_name,
        }
    }
}

impl EmbeddingCache {
    /// Create a new embedding cache
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            ttl,
        }
    }

    /// Get embedding from cache if available and not expired
    pub fn get(&mut self, text: &str) -> Option<Vec<f32>> {
        let now = Instant::now();
        
        if let Some(entry) = self.cache.get(text) {
            if now.duration_since(entry.timestamp) < self.ttl {
                debug!("Cache hit for text: {}", self.truncate_text(text));
                return Some(entry.embedding.clone());
            } else {
                // Entry expired, remove it
                self.cache.remove(text);
                debug!("Cache entry expired for text: {}", self.truncate_text(text));
            }
        }
        
        None
    }

    /// Store embedding in cache
    pub fn put(&mut self, text: String, embedding: Vec<f32>) {
        // If cache is full, remove oldest entries
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }

        let entry = CacheEntry {
            embedding,
            timestamp: Instant::now(),
        };
        
        self.cache.insert(text.clone(), entry);
        debug!("Cached embedding for text: {}", self.truncate_text(&text));
    }

    /// Remove oldest cache entries to make room
    fn evict_oldest(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // Find the oldest entry
        let oldest_key = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            self.cache.remove(&key);
            debug!("Evicted oldest cache entry for text: {}", self.truncate_text(&key));
        }
    }

    /// Clear expired entries from cache
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.timestamp) >= self.ttl)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.remove(&key);
            debug!("Removed expired cache entry for text: {}", self.truncate_text(&key));
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_size)
    }

    /// Truncate text for logging
    fn truncate_text(&self, text: &str) -> String {
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        }
    }
}

impl OpenAISDKEmbeddingService {
    /// Create a new OpenAI SDK embedding service
    pub fn new(config: OpenAISDKEmbeddingConfig) -> Result<Self> {
        let timeout_duration = Duration::from_secs(30); // Default 30 seconds timeout
        let client = Client::builder()
            .timeout(timeout_duration)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Generate single embedding
    #[instrument(level = "debug", skip(self, text), err)]
    pub async fn generate_embedding(&self, text: String) -> FlowyResult<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("Text cannot be empty"));
        }

        debug!("Generating embedding for text: {}", self.truncate_text(&text));

        let request = EmbeddingRequest {
            model: self.config.model_name.clone(),
            input: EmbeddingInput::String(text),
        };

        let response = self.send_embedding_request(request).await?;

        if response.data.is_empty() {
            return Err(FlowyError::internal().with_context("No embedding data in response"));
        }

        let embedding = response.data[0].embedding.clone();
        debug!("Successfully generated embedding with {} dimensions", embedding.len());
        
        Ok(embedding)
    }

    /// Generate embeddings for multiple texts (batch processing)
    #[instrument(level = "debug", skip(self, texts), err)]
    pub async fn generate_embeddings(&self, texts: Vec<String>) -> FlowyResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Filter out empty texts
        let valid_texts: Vec<String> = texts
            .into_iter()
            .filter(|text| !text.trim().is_empty())
            .collect();

        if valid_texts.is_empty() {
            return Ok(vec![]);
        }

        debug!("Generating embeddings for {} texts", valid_texts.len());

        let input = if valid_texts.len() == 1 {
            EmbeddingInput::String(valid_texts[0].clone())
        } else {
            EmbeddingInput::StringArray(valid_texts)
        };

        let request = EmbeddingRequest {
            model: self.config.model_name.clone(),
            input,
        };

        let response = self.send_embedding_request(request).await?;

        // Sort by index to maintain order
        let mut sorted_data = response.data;
        sorted_data.sort_by_key(|d| d.index);

        let embeddings: Vec<Vec<f32>> = sorted_data
            .into_iter()
            .map(|d| d.embedding)
            .collect();

        debug!("Successfully generated {} embeddings", embeddings.len());
        Ok(embeddings)
    }

    /// Generate embeddings with caching support
    #[instrument(level = "debug", skip(self, texts, cache), err)]
    pub async fn generate_embeddings_with_cache(
        &self,
        texts: Vec<String>,
        cache: &mut EmbeddingCache,
    ) -> FlowyResult<HashMap<String, Vec<f32>>> {
        if texts.is_empty() {
            return Ok(HashMap::new());
        }

        let mut results = HashMap::new();
        let mut texts_to_process = Vec::new();

        // Check cache first
        for text in texts {
            if text.trim().is_empty() {
                continue;
            }

            if let Some(cached_embedding) = cache.get(&text) {
                results.insert(text, cached_embedding);
            } else {
                texts_to_process.push(text);
            }
        }

        // Process uncached texts
        let uncached_count = texts_to_process.len();
        if !texts_to_process.is_empty() {
            debug!("Processing {} uncached texts", uncached_count);
            
            let embeddings = self.generate_embeddings(texts_to_process.clone()).await?;
            
            // Store results in cache and return map
            for (text, embedding) in texts_to_process.into_iter().zip(embeddings.into_iter()) {
                cache.put(text.clone(), embedding.clone());
                results.insert(text, embedding);
            }
        }

        debug!("Returned {} embeddings ({} from cache)", results.len(), results.len() - uncached_count);
        Ok(results)
    }

    /// Process embeddings in batches for better performance
    #[instrument(level = "debug", skip(self, texts), err)]
    pub async fn process_batch(&self, texts: Vec<String>, batch_size: usize) -> FlowyResult<HashMap<String, Vec<f32>>> {
        if texts.is_empty() {
            return Ok(HashMap::new());
        }

        let batch_size = batch_size.max(1).min(100); // Ensure reasonable batch size
        debug!("Processing {} texts in batches of {}", texts.len(), batch_size);

        let mut results = HashMap::new();
        
        // Process texts in batches
        for chunk in texts.chunks(batch_size) {
            let chunk_texts = chunk.to_vec();
            let embeddings = self.generate_embeddings(chunk_texts.clone()).await?;

            // Map texts to their embeddings
            for (text, embedding) in chunk_texts.into_iter().zip(embeddings.into_iter()) {
                results.insert(text, embedding);
            }

            // Small delay between batches to avoid rate limiting
            if results.len() < texts.len() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        debug!("Successfully processed {} embeddings in batches", results.len());
        Ok(results)
    }

    /// Test the embedding service connection
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_connection(&self) -> FlowyResult<TestEmbeddingResponse> {
        let start_time = Instant::now();
        let test_text = "This is a test embedding request.".to_string();

        debug!("Testing embedding service connection with test text");

        match self.generate_embedding(test_text).await {
            Ok(embedding) => {
                let duration = start_time.elapsed();
                let response = TestEmbeddingResponse {
                    success: true,
                    message: format!("Connection successful. Generated embedding with {} dimensions.", embedding.len()),
                    response_time_ms: duration.as_millis() as u64,
                    model_name: self.config.model_name.clone(),
                    embedding_dimensions: Some(embedding.len() as u32),
                    error_code: None,
                };
                
                debug!("Embedding service test successful: {} dimensions in {}ms", 
                       embedding.len(), duration.as_millis());
                Ok(response)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                let error_msg = format!("Connection failed: {}", e);
                
                let response = TestEmbeddingResponse {
                    success: false,
                    message: error_msg.clone(),
                    response_time_ms: duration.as_millis() as u64,
                    model_name: self.config.model_name.clone(),
                    embedding_dimensions: None,
                    error_code: Some(self.extract_error_code(&e)),
                };
                
                error!("Embedding service test failed: {}", error_msg);
                Ok(response) // Return Ok with error details instead of Err
            }
        }
    }

    /// Send embedding request to OpenAI SDK API
    async fn send_embedding_request(&self, request: EmbeddingRequest) -> FlowyResult<EmbeddingResponse> {
        let masked_key = self.mask_api_key(&self.config.api_key);
        debug!(
            "Sending embedding request to {} with model {} (API key: {})",
            self.config.api_endpoint, request.model, masked_key
        );

        let response = self
            .client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send embedding request: {}", e);
                if e.is_timeout() {
                    FlowyError::internal().with_context("Request timeout. Please check your network connection.")
                } else if e.is_connect() {
                    FlowyError::internal().with_context("Connection failed. Please check the API endpoint.")
                } else {
                    FlowyError::internal().with_context(format!("Network error: {}", e))
                }
            })?;

        self.handle_response(response).await
    }

    /// Handle HTTP response and parse embedding data
    async fn handle_response(&self, response: Response) -> FlowyResult<EmbeddingResponse> {
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            error!("Embedding API error {}: {}", status, error_text);

            // Try to parse as OpenAI error format
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(FlowyError::internal()
                    .with_context(format!("API error: {}", error_response.error.message)));
            }

            // Fallback to generic error message based on status code
            let error_msg = match status.as_u16() {
                401 => "Authentication failed. Please check your API key.",
                403 => "Access forbidden. Please check your API key permissions.",
                404 => "API endpoint not found. Please check your endpoint URL.",
                429 => "Rate limit exceeded. Please try again later.",
                500 => "Server error. Please try again later.",
                502 => "Bad gateway. The server is temporarily unavailable.",
                503 => "Service unavailable. Please try again later.",
                400 => &format!("Bad request: {}", error_text),
                _ => &format!("HTTP error {}: {}", status, error_text),
            };

            return Err(FlowyError::internal().with_context(error_msg));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| {
                error!("Failed to read response body: {}", e);
                FlowyError::internal().with_context("Failed to read response")
            })?;

        let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("Failed to parse embedding response: {} (body: {})", e, response_text);
                FlowyError::internal()
                    .with_context(format!("Invalid response format: {}", e))
            })?;

        Ok(embedding_response)
    }

    /// Extract error code from FlowyError for categorization
    fn extract_error_code(&self, error: &FlowyError) -> String {
        if error.msg.contains("timeout") || error.msg.contains("Timeout") {
            "TIMEOUT".to_string()
        } else if error.msg.contains("401") || error.msg.contains("Authentication") {
            "AUTH_ERROR".to_string()
        } else if error.msg.contains("404") || error.msg.contains("not found") {
            "NOT_FOUND".to_string()
        } else if error.msg.contains("429") || error.msg.contains("rate limit") {
            "RATE_LIMIT".to_string()
        } else if error.msg.contains("500") || error.msg.contains("Server error") {
            "SERVER_ERROR".to_string()
        } else if error.msg.contains("network") || error.msg.contains("Connection") {
            "NETWORK_ERROR".to_string()
        } else {
            "UNKNOWN_ERROR".to_string()
        }
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

    /// Truncate text for logging
    fn truncate_text(&self, text: &str) -> String {
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        }
    }
}

/// Response for embedding service testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEmbeddingResponse {
    pub success: bool,
    pub message: String,
    pub response_time_ms: u64,
    pub model_name: String,
    pub embedding_dimensions: Option<u32>,
    pub error_code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_cache_basic() {
        let mut cache = EmbeddingCache::new(2, Duration::from_secs(60));
        
        // Test empty cache
        assert!(cache.get("test").is_none());
        
        // Test put and get
        let embedding = vec![0.1, 0.2, 0.3];
        cache.put("test".to_string(), embedding.clone());
        assert_eq!(cache.get("test"), Some(embedding));
        
        // Test cache stats
        let (size, max_size) = cache.stats();
        assert_eq!(size, 1);
        assert_eq!(max_size, 2);
    }

    #[test]
    fn test_embedding_cache_eviction() {
        let mut cache = EmbeddingCache::new(2, Duration::from_secs(60));
        
        // Fill cache to capacity
        cache.put("test1".to_string(), vec![0.1]);
        cache.put("test2".to_string(), vec![0.2]);
        
        // Add one more item, should evict oldest
        cache.put("test3".to_string(), vec![0.3]);
        
        let (size, _) = cache.stats();
        assert_eq!(size, 2);
        
        // test1 should be evicted
        assert!(cache.get("test1").is_none());
        assert!(cache.get("test2").is_some());
        assert!(cache.get("test3").is_some());
    }

    #[test]
    fn test_mask_api_key() {
        let config = OpenAISDKEmbeddingConfig {
            api_endpoint: "https://api.openai.com/v1/embeddings".to_string(),
            api_key: "sk-1234567890abcdef".to_string(),
            model_name: "text-embedding-ada-002".to_string(),
        };
        
        let service = OpenAISDKEmbeddingService::new(config).unwrap();
        
        assert_eq!(service.mask_api_key(""), "");
        assert_eq!(service.mask_api_key("short"), "*****");
        assert_eq!(service.mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(service.mask_api_key("sk-proj-1234567890abcdefghijklmnop"), "sk-p...mnop");
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = OpenAISDKEmbeddingConfig {
            api_endpoint: "https://api.openai.com/v1/embeddings".to_string(),
            api_key: "test-key".to_string(),
            model_name: "text-embedding-ada-002".to_string(),
        };
        
        let service = OpenAISDKEmbeddingService::new(config);
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_empty_text_handling() {
        let config = OpenAISDKEmbeddingConfig {
            api_endpoint: "https://api.openai.com/v1/embeddings".to_string(),
            api_key: "test-key".to_string(),
            model_name: "text-embedding-ada-002".to_string(),
        };
        
        let service = OpenAISDKEmbeddingService::new(config).unwrap();
        
        // Test empty string
        let result = service.generate_embedding("".to_string()).await;
        assert!(result.is_err());
        
        // Test whitespace only
        let result = service.generate_embedding("   ".to_string()).await;
        assert!(result.is_err());
        
        // Test empty vector
        let result = service.generate_embeddings(vec![]).await.unwrap();
        assert!(result.is_empty());
    }
}
