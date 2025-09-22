use crate::entities::*;
use crate::openai_compatible::types::{ChatMessage, ChatCompletionRequest};
use anyhow::{anyhow, Result};
use async_stream::stream;
use flowy_ai_pub::cloud::{QuestionStreamValue, StreamAnswer};
use flowy_error::FlowyError;
use futures::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, instrument, warn};

/// OpenAI SDK streaming chat client
#[derive(Debug, Clone)]
pub struct OpenAISDKChatService {
    client: Client,
    config: OpenAISDKChatConfig,
}

/// Internal configuration for OpenAI SDK chat service
#[derive(Debug, Clone)]
pub struct OpenAISDKChatConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub model_name: String,
    pub model_type: String,
    pub max_tokens: i32,
    pub temperature: f64,
    pub timeout_seconds: i32,
}

/// Streaming response chunk from OpenAI SDK API
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

/// Non-streaming response for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<OpenAISDKChatSettingPB> for OpenAISDKChatConfig {
    fn from(setting: OpenAISDKChatSettingPB) -> Self {
        Self {
            api_endpoint: setting.api_endpoint,
            api_key: setting.api_key,
            model_name: setting.model_name,
            model_type: setting.model_type,
            max_tokens: setting.max_tokens,
            temperature: setting.temperature,
            timeout_seconds: setting.timeout_seconds,
        }
    }
}

impl OpenAISDKChatService {
    /// Create a new OpenAI SDK chat service
    pub fn new(config: OpenAISDKChatConfig) -> Result<Self> {
        let timeout_duration = Duration::from_secs(config.timeout_seconds.max(1) as u64);
        let client = Client::builder()
            .timeout(timeout_duration)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Create from protobuf setting
    pub fn from_setting(setting: OpenAISDKChatSettingPB) -> Result<Self> {
        let config = OpenAISDKChatConfig::from(setting);
        Self::new(config)
    }

    /// Stream chat completion with OpenAI SDK API
    #[instrument(level = "debug", skip(self, messages), err)]
    pub async fn stream_chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<StreamAnswer> {
        let request = ChatCompletionRequest {
            model: model.unwrap_or_else(|| self.config.model_name.clone()),
            messages,
            max_tokens: max_tokens.or_else(|| {
                if self.config.max_tokens > 0 {
                    Some(self.config.max_tokens as u32)
                } else {
                    None
                }
            }),
            temperature: temperature.or_else(|| {
                if self.config.temperature >= 0.0 {
                    Some(self.config.temperature as f32)
                } else {
                    None
                }
            }),
            stream: Some(true),
        };

        let masked_key = self.mask_api_key(&self.config.api_key);
        debug!(
            "Starting OpenAI SDK streaming chat completion to {} with model {} (API key: {})",
            self.config.api_endpoint, request.model, masked_key
        );

        let response = self
            .client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send OpenAI SDK streaming chat request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            return Err(anyhow!("OpenAI SDK HTTP error {}: {}", status, text));
        }

        let stream = self.clone().create_stream_from_response(response)?;
        Ok(stream)
    }

    /// Create a stream from HTTP response with retry mechanism
    fn create_stream_from_response(self, response: Response) -> Result<StreamAnswer> {
        let mut bytes_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut retry_count = 0;
        let max_retries = 3;

        let stream = stream! {
            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = match String::from_utf8(bytes.to_vec()) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Failed to decode chunk as UTF-8: {}", e);
                                retry_count += 1;
                                if retry_count < max_retries {
                                    continue;
                                } else {
                                    yield Err(FlowyError::internal().with_context("Invalid UTF-8 in response"));
                                    break;
                                }
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
                                    debug!("Received [DONE] signal, ending OpenAI SDK stream");
                                    break;
                                }

                                match self.parse_chunk(data_content).await {
                                    Ok(Some(stream_value)) => {
                                        retry_count = 0; // Reset retry count on successful parse
                                        yield Ok(stream_value);
                                    }
                                    Ok(None) => {
                                        // Skip empty or non-content chunks
                                        continue;
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse OpenAI SDK chunk: {}, content: {}", e, data_content);
                                        retry_count += 1;
                                        if retry_count >= max_retries {
                                            yield Err(FlowyError::internal().with_context(format!("Max retries exceeded: {}", e)));
                                            break;
                                        }
                                        // Continue processing other chunks instead of failing the entire stream
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("OpenAI SDK stream error: {}", e);
                        retry_count += 1;
                        if retry_count < max_retries {
                            warn!("Retrying stream connection ({}/{})", retry_count, max_retries);
                            continue;
                        } else {
                            yield Err(FlowyError::internal().with_context(format!("Stream error after {} retries: {}", max_retries, e)));
                            break;
                        }
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
                                warn!("Failed to parse final OpenAI SDK chunk: {}, content: {}", e, data_content);
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
            .map_err(|e| anyhow!("Failed to parse OpenAI SDK JSON chunk: {} (content: {})", e, data_content))?;

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
                debug!("OpenAI SDK chat completion finished with reason: {}", finish_reason);
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
            model: model.unwrap_or_else(|| self.config.model_name.clone()),
            messages,
            max_tokens: max_tokens.or_else(|| {
                if self.config.max_tokens > 0 {
                    Some(self.config.max_tokens as u32)
                } else {
                    None
                }
            }),
            temperature: temperature.or_else(|| {
                if self.config.temperature >= 0.0 {
                    Some(self.config.temperature as f32)
                } else {
                    None
                }
            }),
            stream: Some(false),
        };

        let response = self
            .client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send OpenAI SDK chat request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            return Err(anyhow!("OpenAI SDK HTTP error {}: {}", status, text));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        let chat_response: ChatCompletionResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse OpenAI SDK response JSON: {} (body: {})", e, response_text))?;

        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response choices returned from OpenAI SDK"))
        }
    }

    /// Test streaming functionality with a simple message
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_streaming(&self) -> Result<String> {
        let start_time = Instant::now();
        
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, this is a test message. Please respond briefly.".to_string(),
        }];

        let mut stream = self
            .stream_chat_completion(messages, None, Some(50), Some(0.1))
            .await?;

        let mut result = String::new();
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds.max(10) as u64);

        while let Ok(Some(chunk_result)) = timeout(timeout_duration, stream.next()).await {
            match chunk_result {
                Ok(QuestionStreamValue::Answer { value }) => {
                    result.push_str(&value);
                }
                Ok(_) => {
                    // Skip other types of stream values for testing
                }
                Err(e) => {
                    return Err(anyhow!("OpenAI SDK stream error: {}", e));
                }
            }
        }

        let elapsed = start_time.elapsed();
        
        if result.is_empty() {
            Err(anyhow!("No content received from OpenAI SDK streaming"))
        } else {
            Ok(format!(
                "OpenAI SDK streaming test successful ({}ms): {}", 
                elapsed.as_millis(),
                result.trim()
            ))
        }
    }

    /// Test non-streaming functionality
    #[instrument(level = "debug", skip(self), err)]
    pub async fn test_non_streaming(&self) -> Result<String> {
        let start_time = Instant::now();
        
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, this is a test message. Please respond briefly.".to_string(),
        }];

        let result = self
            .chat_completion_non_streaming(messages, None, Some(50), Some(0.1))
            .await?;

        let elapsed = start_time.elapsed();
        
        Ok(format!(
            "OpenAI SDK non-streaming test successful ({}ms): {}", 
            elapsed.as_millis(),
            result.trim()
        ))
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

    fn create_test_config() -> OpenAISDKChatConfig {
        OpenAISDKChatConfig {
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: "sk-test1234567890abcdef".to_string(),
            model_name: "gpt-3.5-turbo".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout_seconds: 30,
        }
    }

    #[test]
    fn test_mask_api_key() {
        let service = OpenAISDKChatService::new(create_test_config()).unwrap();
        
        assert_eq!(service.mask_api_key(""), "");
        assert_eq!(service.mask_api_key("short"), "*****");
        assert_eq!(service.mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(service.mask_api_key("sk-proj-1234567890abcdefghijklmnop"), "sk-p...mnop");
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = create_test_config();
        let service = OpenAISDKChatService::new(config);
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_from_setting() {
        let setting = OpenAISDKChatSettingPB {
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: "sk-test1234567890abcdef".to_string(),
            model_name: "gpt-3.5-turbo".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout_seconds: 30,
        };
        
        let service = OpenAISDKChatService::from_setting(setting);
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_parse_chunk() {
        let service = OpenAISDKChatService::new(create_test_config()).unwrap();
        
        // Test valid chunk with content
        let chunk_data = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let result = service.parse_chunk(chunk_data).await.unwrap();
        assert!(matches!(result, Some(QuestionStreamValue::Answer { value }) if value == "Hello"));

        // Test chunk with finish_reason
        let finish_chunk = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#;
        let result = service.parse_chunk(finish_chunk).await.unwrap();
        assert!(result.is_none());

        // Test empty content
        let empty_chunk = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-3.5-turbo","choices":[{"index":0,"delta":{"content":""},"finish_reason":null}]}"#;
        let result = service.parse_chunk(empty_chunk).await.unwrap();
        assert!(result.is_none());
    }
}
