use crate::entities::*;
use crate::openai_sdk::chat_service::OpenAISDKChatService;
use crate::openai_compatible::types::ChatMessage;
use flowy_ai_pub::cloud::StreamAnswer;
use flowy_error::{FlowyError, FlowyResult};
use std::time::Instant;
use tracing::{debug, error};

/// OpenAI SDK controller for managing OpenAI SDK configurations and testing
pub struct OpenAISDKController;

impl OpenAISDKController {
  pub fn new() -> Self {
    Self
  }

  /// Stream chat completion using OpenAI SDK
  pub async fn stream_chat_completion(
    setting: OpenAISDKChatSettingPB,
    messages: Vec<ChatMessage>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
  ) -> FlowyResult<StreamAnswer> {
    debug!(
      "Starting OpenAI SDK stream chat completion with model: {:?}",
      model.as_ref().unwrap_or(&setting.model_name)
    );

    let service = OpenAISDKChatService::from_setting(setting)
      .map_err(|e| FlowyError::internal().with_context(format!("Failed to create OpenAI SDK chat service: {}", e)))?;

    service
      .stream_chat_completion(messages, model, max_tokens, temperature)
      .await
      .map_err(|e| FlowyError::internal().with_context(format!("OpenAI SDK stream chat failed: {}", e)))
  }

  /// Non-streaming chat completion using OpenAI SDK
  pub async fn chat_completion(
    setting: OpenAISDKChatSettingPB,
    messages: Vec<ChatMessage>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
  ) -> FlowyResult<String> {
    debug!(
      "Starting OpenAI SDK chat completion with model: {:?}",
      model.as_ref().unwrap_or(&setting.model_name)
    );

    let service = OpenAISDKChatService::from_setting(setting)
      .map_err(|e| FlowyError::internal().with_context(format!("Failed to create OpenAI SDK chat service: {}", e)))?;

    service
      .chat_completion_non_streaming(messages, model, max_tokens, temperature)
      .await
      .map_err(|e| FlowyError::internal().with_context(format!("OpenAI SDK chat completion failed: {}", e)))
  }

  /// Test OpenAI SDK chat configuration
  pub async fn test_chat(setting: OpenAISDKChatSettingPB) -> TestResultPB {
    debug!("Testing OpenAI SDK chat with endpoint: {}", setting.api_endpoint);
    
    let start_time = Instant::now();
    
    match OpenAISDKChatService::from_setting(setting) {
      Ok(service) => {
        match service.test_streaming().await {
          Ok(response) => {
            let elapsed = start_time.elapsed();
            debug!("OpenAI SDK chat test successful: {}", response);
            TestResultPB {
              success: true,
              error_message: String::new(),
              response_time_ms: elapsed.as_millis().to_string(),
              status_code: 200,
              server_response: response,
              request_details: "OpenAI SDK chat streaming test".to_string(),
            }
          }
          Err(e) => {
            let elapsed = start_time.elapsed();
            error!("OpenAI SDK chat test failed: {}", e);
            TestResultPB {
              success: false,
              error_message: format!("OpenAI SDK chat test failed: {}", e),
              response_time_ms: elapsed.as_millis().to_string(),
              status_code: 500,
              server_response: String::new(),
              request_details: "OpenAI SDK chat streaming test".to_string(),
            }
          }
        }
      }
      Err(e) => {
        let elapsed = start_time.elapsed();
        error!("Failed to create OpenAI SDK chat service: {}", e);
        TestResultPB {
          success: false,
          error_message: format!("Failed to create OpenAI SDK chat service: {}", e),
          response_time_ms: elapsed.as_millis().to_string(),
          status_code: 500,
          server_response: String::new(),
          request_details: "OpenAI SDK chat service creation".to_string(),
        }
      }
    }
  }

  /// Test OpenAI SDK embedding configuration
  pub async fn test_embedding(setting: OpenAISDKEmbeddingSettingPB) -> TestResultPB {
    debug!("Testing OpenAI SDK embedding with endpoint: {}", setting.api_endpoint);
    
    let start_time = Instant::now();
    
    // Create embedding service from setting
    let config = crate::openai_sdk::embedding_service::OpenAISDKEmbeddingConfig::from(setting);
    
    match crate::openai_sdk::embedding_service::OpenAISDKEmbeddingService::new(config) {
      Ok(service) => {
        match service.test_connection().await {
          Ok(test_response) => {
            let elapsed = start_time.elapsed();
            debug!("OpenAI SDK embedding test completed: success={}", test_response.success);
            
            if test_response.success {
              TestResultPB {
                success: true,
                error_message: String::new(),
                response_time_ms: elapsed.as_millis().to_string(),
                status_code: 200,
                server_response: test_response.message,
                request_details: format!("OpenAI SDK embedding test - Model: {}", test_response.model_name),
              }
            } else {
              TestResultPB {
                success: false,
                error_message: test_response.message,
                response_time_ms: elapsed.as_millis().to_string(),
                status_code: 500,
                server_response: String::new(),
                request_details: format!("OpenAI SDK embedding test - Model: {}", test_response.model_name),
              }
            }
          }
          Err(e) => {
            let elapsed = start_time.elapsed();
            error!("OpenAI SDK embedding test failed: {}", e);
            TestResultPB {
              success: false,
              error_message: format!("OpenAI SDK embedding test failed: {}", e),
              response_time_ms: elapsed.as_millis().to_string(),
              status_code: 500,
              server_response: String::new(),
              request_details: "OpenAI SDK embedding test".to_string(),
            }
          }
        }
      }
      Err(e) => {
        let elapsed = start_time.elapsed();
        error!("Failed to create OpenAI SDK embedding service: {}", e);
        TestResultPB {
          success: false,
          error_message: format!("Failed to create OpenAI SDK embedding service: {}", e),
          response_time_ms: elapsed.as_millis().to_string(),
          status_code: 500,
          server_response: String::new(),
          request_details: "OpenAI SDK embedding service creation".to_string(),
        }
      }
    }
  }
}

impl Default for OpenAISDKController {
  fn default() -> Self {
    Self::new()
  }
}
