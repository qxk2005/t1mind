use crate::entities::{OpenAIChatSettingPB, OpenAIEmbeddingSettingPB, TestResultPB};
use crate::openai_compatible::{OpenAICompatibleClient, OpenAICompatibleConfig};
use std::time::Instant;
use tracing::{debug, error, instrument};
use serde_json;

/// Detailed test result with additional information
#[derive(Debug, Clone)]
pub struct DetailedTestResult {
    pub success: bool,
    pub error_message: String,
    pub response_time_ms: u128,
    pub status_code: Option<u16>,
    pub server_response: Option<String>,
    pub request_details: Option<String>,
}

/// Test OpenAI compatible chat functionality
#[instrument(level = "debug", skip(chat_setting))]
pub async fn test_chat(chat_setting: OpenAIChatSettingPB) -> TestResultPB {
    debug!("Starting OpenAI compatible chat test");
    let start_time = Instant::now();

    // Validate required fields
    if chat_setting.api_endpoint.trim().is_empty() {
        return TestResultPB {
            success: false,
            error_message: "API endpoint is required".to_string(),
            response_time_ms: "0".to_string(),
            status_code: 0,
            server_response: String::new(),
            request_details: String::new(),
        };
    }

    if chat_setting.api_key.trim().is_empty() {
        return TestResultPB {
            success: false,
            error_message: "API key is required".to_string(),
            response_time_ms: "0".to_string(),
            status_code: 0,
            server_response: String::new(),
            request_details: String::new(),
        };
    }

    if chat_setting.model_name.trim().is_empty() {
        return TestResultPB {
            success: false,
            error_message: "Model name is required".to_string(),
            response_time_ms: "0".to_string(),
            status_code: 0,
            server_response: String::new(),
            request_details: String::new(),
        };
    }

    // Create configuration from settings
    let config = OpenAICompatibleConfig {
        chat_endpoint: format!("{}/chat/completions", chat_setting.api_endpoint.trim_end_matches('/')),
        chat_api_key: chat_setting.api_key.clone(),
        chat_model: chat_setting.model_name.clone(),
        embedding_endpoint: String::new(), // Not needed for chat test
        embedding_api_key: String::new(),   // Not needed for chat test
        embedding_model: String::new(),     // Not needed for chat test
        timeout_ms: Some(30000), // 30 seconds timeout for test
        max_tokens: if chat_setting.max_tokens > 0 {
            Some(chat_setting.max_tokens as u32)
        } else {
            Some(50) // Small number for test
        },
        temperature: if chat_setting.temperature >= 0.0 {
            Some(chat_setting.temperature as f32)
        } else {
            Some(0.1) // Low temperature for consistent test results
        },
    };

    // Create client and test
    match OpenAICompatibleClient::new(config) {
        Ok(client) => {
            match client.test_chat_detailed().await {
                Ok((response, server_response, request_details)) => {
                    let elapsed = start_time.elapsed();
                    debug!("Chat test successful: {}", response);
                    TestResultPB {
                        success: true,
                        error_message: String::new(),
                        response_time_ms: elapsed.as_millis().to_string(),
                        status_code: 200,
                        server_response: server_response.unwrap_or_default(),
                        request_details: request_details.unwrap_or_default(),
                    }
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    let error_msg = format_user_friendly_error(&e);
                    error!("Chat test failed: {}", e);
                    
                    // Try to extract status code from error message
                    let status_code = extract_status_code_from_error(&e);
                    
                    TestResultPB {
                        success: false,
                        error_message: error_msg,
                        response_time_ms: elapsed.as_millis().to_string(),
                        status_code: status_code.unwrap_or(0) as i32,
                        server_response: e.to_string(),
                        request_details: String::new(),
                    }
                }
            }
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            let error_msg = format!("Failed to create HTTP client: {}", e);
            error!("{}", error_msg);
            TestResultPB {
                success: false,
                error_message: error_msg,
                response_time_ms: elapsed.as_millis().to_string(),
                status_code: 0,
                server_response: e.to_string(),
                request_details: String::new(),
            }
        }
    }
}

/// Test OpenAI compatible embedding functionality
#[instrument(level = "debug", skip(embedding_setting))]
pub async fn test_embedding(embedding_setting: OpenAIEmbeddingSettingPB) -> TestResultPB {
    debug!("Starting OpenAI compatible embedding test");
    let start_time = Instant::now();

    // Validate required fields
    if embedding_setting.api_endpoint.trim().is_empty() {
        return TestResultPB {
            success: false,
            error_message: "API endpoint is required".to_string(),
            response_time_ms: "0".to_string(),
            status_code: 0,
            server_response: String::new(),
            request_details: String::new(),
        };
    }

    if embedding_setting.api_key.trim().is_empty() {
        return TestResultPB {
            success: false,
            error_message: "API key is required".to_string(),
            response_time_ms: "0".to_string(),
            status_code: 0,
            server_response: String::new(),
            request_details: String::new(),
        };
    }

    if embedding_setting.model_name.trim().is_empty() {
        return TestResultPB {
            success: false,
            error_message: "Model name is required".to_string(),
            response_time_ms: "0".to_string(),
            status_code: 0,
            server_response: String::new(),
            request_details: String::new(),
        };
    }

    // Create configuration from settings
    let config = OpenAICompatibleConfig {
        chat_endpoint: String::new(),       // Not needed for embedding test
        chat_api_key: String::new(),        // Not needed for embedding test
        chat_model: String::new(),          // Not needed for embedding test
        embedding_endpoint: format!("{}/embeddings", embedding_setting.api_endpoint.trim_end_matches('/')),
        embedding_api_key: embedding_setting.api_key.clone(),
        embedding_model: embedding_setting.model_name.clone(),
        timeout_ms: Some(30000), // 30 seconds timeout for test
        max_tokens: None,
        temperature: None,
    };

    // Create client and test
    match OpenAICompatibleClient::new(config) {
        Ok(client) => {
            match client.test_embedding_detailed().await {
                Ok((response, server_response, request_details)) => {
                    let elapsed = start_time.elapsed();
                    debug!("Embedding test successful: {}", response);
                    TestResultPB {
                        success: true,
                        error_message: String::new(),
                        response_time_ms: elapsed.as_millis().to_string(),
                        status_code: 200,
                        server_response: server_response.unwrap_or_default(),
                        request_details: request_details.unwrap_or_default(),
                    }
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    let error_msg = format_user_friendly_error(&e);
                    error!("Embedding test failed: {}", e);
                    
                    // Try to extract status code from error message
                    let status_code = extract_status_code_from_error(&e);
                    
                    TestResultPB {
                        success: false,
                        error_message: error_msg,
                        response_time_ms: elapsed.as_millis().to_string(),
                        status_code: status_code.unwrap_or(0) as i32,
                        server_response: e.to_string(),
                        request_details: String::new(),
                    }
                }
            }
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            let error_msg = format!("Failed to create HTTP client: {}", e);
            error!("{}", error_msg);
            TestResultPB {
                success: false,
                error_message: error_msg,
                response_time_ms: elapsed.as_millis().to_string(),
                status_code: 0,
                server_response: e.to_string(),
                request_details: String::new(),
            }
        }
    }
}

/// Extract HTTP status code from error message
fn extract_status_code_from_error(error: &anyhow::Error) -> Option<u16> {
    let error_str = error.to_string();
    
    // Try to extract status code from common patterns
    if error_str.contains("401") {
        Some(401)
    } else if error_str.contains("404") {
        Some(404)
    } else if error_str.contains("429") {
        Some(429)
    } else if error_str.contains("500") {
        Some(500)
    } else if error_str.contains("502") {
        Some(502)
    } else if error_str.contains("503") {
        Some(503)
    } else if error_str.contains("400") {
        Some(400)
    } else {
        None
    }
}

/// Format error messages to be user-friendly
fn format_user_friendly_error(error: &anyhow::Error) -> String {
    let error_str = error.to_string().to_lowercase();
    
    if error_str.contains("authentication failed") || error_str.contains("unauthorized") {
        "Authentication failed. Please check your API key.".to_string()
    } else if error_str.contains("not found") || error_str.contains("404") {
        "API endpoint not found. Please check your endpoint URL.".to_string()
    } else if error_str.contains("rate limit") || error_str.contains("too many requests") {
        "Rate limit exceeded. Please try again later.".to_string()
    } else if error_str.contains("timeout") || error_str.contains("timed out") {
        "Request timed out. Please check your network connection or try again later.".to_string()
    } else if error_str.contains("connection") || error_str.contains("network") {
        "Network connection failed. Please check your internet connection and endpoint URL.".to_string()
    } else if error_str.contains("invalid") && error_str.contains("model") {
        "Invalid model name. Please check that the model exists and is available.".to_string()
    } else if error_str.contains("server error") || error_str.contains("500") {
        "Server error. Please try again later.".to_string()
    } else if error_str.contains("bad request") || error_str.contains("400") {
        "Bad request. Please check your configuration settings.".to_string()
    } else {
        // Return the original error message but limit its length
        let msg = error.to_string();
        if msg.len() > 200 {
            format!("{}...", &msg[..200])
        } else {
            msg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_user_friendly_error() {
        let auth_error = anyhow::anyhow!("Authentication failed. Please check your API key.");
        assert_eq!(
            format_user_friendly_error(&auth_error),
            "Authentication failed. Please check your API key."
        );

        let timeout_error = anyhow::anyhow!("Request timed out after 30 seconds");
        assert_eq!(
            format_user_friendly_error(&timeout_error),
            "Request timed out. Please check your network connection or try again later."
        );

        let generic_error = anyhow::anyhow!("Some unexpected error occurred");
        assert_eq!(
            format_user_friendly_error(&generic_error),
            "Some unexpected error occurred"
        );
    }

    #[tokio::test]
    async fn test_chat_validation() {
        // Test empty endpoint
        let setting = OpenAIChatSettingPB {
            api_endpoint: "".to_string(),
            api_key: "test-key".to_string(),
            model_name: "test-model".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            timeout_seconds: 30,
        };
        let result = test_chat(setting).await;
        assert!(!result.success);
        assert!(result.error_message.contains("API endpoint is required"));

        // Test empty API key
        let setting = OpenAIChatSettingPB {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: "".to_string(),
            model_name: "test-model".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            timeout_seconds: 30,
        };
        let result = test_chat(setting).await;
        assert!(!result.success);
        assert!(result.error_message.contains("API key is required"));

        // Test empty model name
        let setting = OpenAIChatSettingPB {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: "test-key".to_string(),
            model_name: "".to_string(),
            model_type: "chat".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            timeout_seconds: 30,
        };
        let result = test_chat(setting).await;
        assert!(!result.success);
        assert!(result.error_message.contains("Model name is required"));
    }

    #[tokio::test]
    async fn test_embedding_validation() {
        // Test empty endpoint
        let setting = OpenAIEmbeddingSettingPB {
            api_endpoint: "".to_string(),
            api_key: "test-key".to_string(),
            model_name: "test-model".to_string(),
        };
        let result = test_embedding(setting).await;
        assert!(!result.success);
        assert!(result.error_message.contains("API endpoint is required"));

        // Test empty API key
        let setting = OpenAIEmbeddingSettingPB {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: "".to_string(),
            model_name: "test-model".to_string(),
        };
        let result = test_embedding(setting).await;
        assert!(!result.success);
        assert!(result.error_message.contains("API key is required"));

        // Test empty model name
        let setting = OpenAIEmbeddingSettingPB {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: "test-key".to_string(),
            model_name: "".to_string(),
        };
        let result = test_embedding(setting).await;
        assert!(!result.success);
        assert!(result.error_message.contains("Model name is required"));
    }
}
