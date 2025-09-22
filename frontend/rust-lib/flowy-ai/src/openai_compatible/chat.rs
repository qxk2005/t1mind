use crate::openai_compatible::types::*;
use anyhow::{anyhow, Result};
use async_stream::stream;
use flowy_ai_pub::cloud::{QuestionStreamValue, StreamAnswer};
use flowy_error::FlowyError;
use futures::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, instrument, warn};

/// OpenAI compatible streaming chat client
#[derive(Debug, Clone)]
pub struct OpenAICompatibleChatClient {
    client: Client,
    config: OpenAICompatibleConfig,
}

/// Streaming response chunk from OpenAI compatible API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoiceChunk>,
}

/// Choice in streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoiceChunk {
    pub index: u32,
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

/// Delta content in streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

impl OpenAICompatibleChatClient {
    /// Create a new streaming chat client
    pub fn new(config: OpenAICompatibleConfig) -> Result<Self> {
        let timeout_duration = Duration::from_millis(config.timeout_ms.unwrap_or(30000));
        let client = Client::builder()
            .timeout(timeout_duration)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Stream chat completion with OpenAI compatible API
    #[instrument(level = "debug", skip(self, messages), err)]
    pub async fn stream_chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<StreamAnswer> {
        let request = ChatCompletionRequest {
            model: model.unwrap_or_else(|| self.config.chat_model.clone()),
            messages,
            max_tokens: max_tokens.or(self.config.max_tokens),
            temperature: temperature.or(self.config.temperature),
            stream: Some(true),
        };

        let masked_key = self.mask_api_key(&self.config.chat_api_key);
        debug!(
            "Starting streaming chat completion to {} with model {} (API key: {})",
            self.config.chat_endpoint, request.model, masked_key
        );

        let response = self
            .client
            .post(&self.config.chat_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.chat_api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send streaming chat request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            return Err(anyhow!("HTTP error {}: {}", status, text));
        }

        let stream = self.clone().create_stream_from_response(response)?;
        Ok(stream)
    }

    /// Create a stream from HTTP response
    fn create_stream_from_response(self, response: Response) -> Result<StreamAnswer> {
        let mut bytes_stream = response.bytes_stream();
        let mut buffer = String::new();

        let stream = stream! {
            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = match String::from_utf8(bytes.to_vec()) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to decode chunk as UTF-8: {}", e);
                                yield Err(FlowyError::internal().with_context("Invalid UTF-8 in response"));
                                continue;
                            }
                        };

                        buffer.push_str(&chunk_str);

                        // Process complete lines in buffer
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer = buffer[line_end + 1..].to_string();

                            if line.is_empty() {
                                continue;
                            }

                            // Handle SSE format: "data: {...}"
                            if let Some(data_content) = line.strip_prefix("data: ") {
                                if data_content.trim() == "[DONE]" {
                                    debug!("Received [DONE] signal, ending stream");
                                    break;
                                }

                                match self.parse_chunk(data_content).await {
                                    Ok(Some(stream_value)) => {
                                        yield Ok(stream_value);
                                    }
                                    Ok(None) => {
                                        // Skip empty or non-content chunks
                                        continue;
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse chunk: {}, content: {}", e, data_content);
                                        // Continue processing other chunks instead of failing the entire stream
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        yield Err(FlowyError::internal().with_context(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }

            // Process any remaining content in buffer
            if !buffer.trim().is_empty() {
                if let Some(data_content) = buffer.trim().strip_prefix("data: ") {
                    if data_content.trim() != "[DONE]" {
                        match self.parse_chunk(data_content).await {
                            Ok(Some(stream_value)) => {
                                yield Ok(stream_value);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                warn!("Failed to parse final chunk: {}, content: {}", e, data_content);
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Parse a single SSE chunk into QuestionStreamValue
    async fn parse_chunk(&self, data_content: &str) -> Result<Option<QuestionStreamValue>> {
        let chunk: ChatCompletionChunk = serde_json::from_str(data_content)
            .map_err(|e| anyhow!("Failed to parse JSON chunk: {} (content: {})", e, data_content))?;

        // Extract content from the first choice
        if let Some(choice) = chunk.choices.first() {
            if let Some(content) = &choice.delta.content {
                if !content.is_empty() {
                    return Ok(Some(QuestionStreamValue::Answer {
                        value: content.clone(),
                    }));
                }
            }

            // Handle finish_reason
            if let Some(finish_reason) = &choice.finish_reason {
                debug!("Chat completion finished with reason: {}", finish_reason);
                // We don't need to yield anything for finish_reason, just log it
                return Ok(None);
            }
        }

        // Skip chunks without content
        Ok(None)
    }

    /// Non-streaming chat completion for testing
    #[instrument(level = "debug", skip(self, messages), err)]
    pub async fn chat_completion_non_streaming(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<String> {
        let request = ChatCompletionRequest {
            model: model.unwrap_or_else(|| self.config.chat_model.clone()),
            messages,
            max_tokens: max_tokens.or(self.config.max_tokens),
            temperature: temperature.or(self.config.temperature),
            stream: Some(false),
        };

        let response = self
            .client
            .post(&self.config.chat_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.chat_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send chat request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            return Err(anyhow!("HTTP error {}: {}", status, text));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        let chat_response: ChatCompletionResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse response JSON: {} (body: {})", e, response_text))?;

        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response choices returned"))
        }
    }

    /// Test streaming functionality with a simple message
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_streaming(&self) -> Result<String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, this is a test message. Please respond briefly.".to_string(),
        }];

        let mut stream = self
            .stream_chat_completion(messages, None, Some(50), Some(0.1))
            .await?;

        let mut result = String::new();
        let timeout_duration = Duration::from_secs(30);

        while let Ok(Some(chunk_result)) = timeout(timeout_duration, stream.next()).await {
            match chunk_result {
                Ok(QuestionStreamValue::Answer { value }) => {
                    result.push_str(&value);
                }
                Ok(_) => {
                    // Skip other types of stream values for testing
                }
                Err(e) => {
                    return Err(anyhow!("Stream error: {}", e));
                }
            }
        }

        if result.is_empty() {
            Err(anyhow!("No content received from streaming"))
        } else {
            Ok(format!("Streaming test successful: {}", result.trim()))
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
        let client = OpenAICompatibleChatClient::new(OpenAICompatibleConfig::default()).unwrap();
        
        assert_eq!(client.mask_api_key(""), "");
        assert_eq!(client.mask_api_key("short"), "*****");
        assert_eq!(client.mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(client.mask_api_key("sk-proj-1234567890abcdefghijklmnop"), "sk-p...mnop");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = OpenAICompatibleConfig::default();
        let client = OpenAICompatibleChatClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_parse_chunk() {
        let config = OpenAICompatibleConfig::default();
        let client = OpenAICompatibleChatClient::new(config).unwrap();
        
        // Test valid chunk with content
        let chunk_data = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let result = client.parse_chunk(chunk_data).await.unwrap();
        assert!(matches!(result, Some(QuestionStreamValue::Answer { value }) if value == "Hello"));

        // Test chunk with finish_reason
        let finish_chunk = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#;
        let result = client.parse_chunk(finish_chunk).await.unwrap();
        assert!(result.is_none());

        // Test empty content
        let empty_chunk = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":""},"finish_reason":null}]}"#;
        let result = client.parse_chunk(empty_chunk).await.unwrap();
        assert!(result.is_none());
    }
}
