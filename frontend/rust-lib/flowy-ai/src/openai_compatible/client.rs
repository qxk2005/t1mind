use crate::openai_compatible::types::*;
use anyhow::{anyhow, Result};
use reqwest::{Client, Response, StatusCode};
use std::time::Duration;
use tracing::{debug, error, instrument, warn};

/// OpenAI compatible HTTP client
#[derive(Debug, Clone)]
pub struct OpenAICompatibleClient {
    client: Client,
    config: OpenAICompatibleConfig,
}

impl OpenAICompatibleClient {
    /// Create a new OpenAI compatible client with the given configuration
    pub fn new(config: OpenAICompatibleConfig) -> Result<Self> {
        let timeout = Duration::from_millis(config.timeout_ms.unwrap_or(30000));
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Test chat functionality with a simple message
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_chat(&self) -> Result<String> {
        let request = ChatCompletionRequest {
            model: self.config.chat_model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello, this is a test message.".to_string(),
            }],
            max_tokens: Some(50),
            temperature: Some(0.1),
            stream: Some(false),
        };

        match self.chat_completion(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    Ok(format!("Chat test successful: {}", choice.message.content.trim()))
                } else {
                    Err(anyhow!("No response choices returned"))
                }
            }
            Err(e) => {
                error!("Chat test failed: {}", e);
                Err(e)
            }
        }
    }

    /// Test chat functionality with detailed result information
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_chat_detailed(&self) -> Result<(String, Option<String>, Option<String>)> {
        let request = ChatCompletionRequest {
            model: self.config.chat_model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello, this is a test message.".to_string(),
            }],
            max_tokens: Some(50),
            temperature: Some(0.1),
            stream: Some(false),
        };

        // Serialize request for logging
        let request_json = serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());

        match self.chat_completion_detailed(request).await {
            Ok((response, response_json)) => {
                if let Some(choice) = response.choices.first() {
                    let result_msg = format!("Chat test successful: {}", choice.message.content.trim());
                    Ok((result_msg, Some(response_json), Some(request_json)))
                } else {
                    Err(anyhow!("No response choices returned"))
                }
            }
            Err(e) => {
                error!("Chat test failed: {}", e);
                Err(e)
            }
        }
    }

    /// Test embedding functionality with a simple text
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_embedding(&self) -> Result<String> {
        let request = EmbeddingRequest {
            model: self.config.embedding_model.clone(),
            input: EmbeddingInput::String("Hello, this is a test embedding.".to_string()),
        };

        match self.create_embedding(request).await {
            Ok(response) => {
                if let Some(data) = response.data.first() {
                    Ok(format!(
                        "Embedding test successful: {} dimensions",
                        data.embedding.len()
                    ))
                } else {
                    Err(anyhow!("No embedding data returned"))
                }
            }
            Err(e) => {
                error!("Embedding test failed: {}", e);
                Err(e)
            }
        }
    }

    /// Test embedding functionality with detailed result information
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_embedding_detailed(&self) -> Result<(String, Option<String>, Option<String>)> {
        let request = EmbeddingRequest {
            model: self.config.embedding_model.clone(),
            input: EmbeddingInput::String("Hello, this is a test embedding.".to_string()),
        };

        // Serialize request for logging
        let request_json = serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());

        match self.create_embedding_detailed(request).await {
            Ok((response, response_json)) => {
                if let Some(data) = response.data.first() {
                    let result_msg = format!(
                        "Embedding test successful: {} dimensions",
                        data.embedding.len()
                    );
                    Ok((result_msg, Some(response_json), Some(request_json)))
                } else {
                    Err(anyhow!("No embedding data returned"))
                }
            }
            Err(e) => {
                error!("Embedding test failed: {}", e);
                Err(e)
            }
        }
    }

    /// Send a chat completion request
    #[instrument(level = "debug", skip(self, request), err)]
    pub async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let masked_key = self.mask_api_key(&self.config.chat_api_key);
        debug!(
            "Sending chat completion request to {} with model {} (API key: {})",
            self.config.chat_endpoint, request.model, masked_key
        );

        let response = self
            .client
            .post(&self.config.chat_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.chat_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send chat request: {}", e))?;

        self.handle_response(response).await
    }

    /// Send a chat completion request with detailed response information
    #[instrument(level = "debug", skip(self, request), err)]
    pub async fn chat_completion_detailed(&self, request: ChatCompletionRequest) -> Result<(ChatCompletionResponse, String)> {
        let masked_key = self.mask_api_key(&self.config.chat_api_key);
        debug!(
            "Sending chat completion request to {} with model {} (API key: {})",
            self.config.chat_endpoint, request.model, masked_key
        );

        let response = self
            .client
            .post(&self.config.chat_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.chat_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send chat request: {}", e))?;

        self.handle_response_detailed(response).await
    }

    /// Create embeddings for the given input
    #[instrument(level = "debug", skip(self, request), err)]
    pub async fn create_embedding(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
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
            .map_err(|e| anyhow!("Failed to send embedding request: {}", e))?;

        self.handle_response(response).await
    }

    /// Create embeddings for the given input with detailed response information
    #[instrument(level = "debug", skip(self, request), err)]
    pub async fn create_embedding_detailed(&self, request: EmbeddingRequest) -> Result<(EmbeddingResponse, String)> {
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
            .map_err(|e| anyhow!("Failed to send embedding request: {}", e))?;

        self.handle_response_detailed(response).await
    }

    /// Handle HTTP response and parse JSON or error
    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let url = response.url().clone();

        if status.is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

            serde_json::from_str(&text)
                .map_err(|e| anyhow!("Failed to parse JSON response: {} (body: {})", e, text))
        } else {
            let text = response
                .text()
                .await
                .map_err(|e| anyhow!("Failed to read error response body: {}", e))?;

            // Try to parse as OpenAI error format
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&text) {
                Err(anyhow!(
                    "API error ({}): {}",
                    status,
                    error_response.error.message
                ))
            } else {
                // Fallback to generic error message
                let error_msg = match status {
                    StatusCode::UNAUTHORIZED => "Authentication failed. Please check your API key.".to_string(),
                    StatusCode::NOT_FOUND => "API endpoint not found. Please check your endpoint URL.".to_string(),
                    StatusCode::TOO_MANY_REQUESTS => "Rate limit exceeded. Please try again later.".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR => "Server error. Please try again later.".to_string(),
                    StatusCode::BAD_REQUEST => format!("Bad request: {}", text),
                    _ => format!("HTTP error {}: {}", status, text),
                };

                warn!("API request failed: {} (URL: {})", error_msg, url);
                Err(anyhow!(error_msg))
            }
        }
    }

    /// Handle HTTP response and parse JSON or error with detailed response information
    async fn handle_response_detailed<T>(&self, response: Response) -> Result<(T, String)>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let url = response.url().clone();

        let text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        if status.is_success() {
            match serde_json::from_str(&text) {
                Ok(parsed) => Ok((parsed, text)),
                Err(e) => Err(anyhow!("Failed to parse JSON response: {} (body: {})", e, text))
            }
        } else {
            // Try to parse as OpenAI error format
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&text) {
                Err(anyhow!(
                    "API error ({}): {}",
                    status,
                    error_response.error.message
                ))
            } else {
                // Fallback to generic error message
                let error_msg = match status {
                    StatusCode::UNAUTHORIZED => "Authentication failed. Please check your API key.".to_string(),
                    StatusCode::NOT_FOUND => "API endpoint not found. Please check your endpoint URL.".to_string(),
                    StatusCode::TOO_MANY_REQUESTS => "Rate limit exceeded. Please try again later.".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR => "Server error. Please try again later.".to_string(),
                    StatusCode::BAD_REQUEST => format!("Bad request: {}", text),
                    _ => format!("HTTP error {}: {}", status, text),
                };

                warn!("API request failed: {} (URL: {})", error_msg, url);
                Err(anyhow!(error_msg))
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        let client = OpenAICompatibleClient::new(OpenAICompatibleConfig::default()).unwrap();
        
        assert_eq!(client.mask_api_key(""), "");
        assert_eq!(client.mask_api_key("short"), "*****");
        assert_eq!(client.mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(client.mask_api_key("sk-proj-1234567890abcdefghijklmnop"), "sk-p...mnop");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = OpenAICompatibleConfig::default();
        let client = OpenAICompatibleClient::new(config);
        assert!(client.is_ok());
    }
}
