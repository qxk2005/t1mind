use serde::{Deserialize, Serialize};

/// OpenAI compatible API request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: EmbeddingInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}

/// Configuration for OpenAI compatible client
#[derive(Debug, Clone)]
pub struct OpenAICompatibleConfig {
    pub chat_endpoint: String,
    pub chat_api_key: String,
    pub chat_model: String,
    pub embedding_endpoint: String,
    pub embedding_api_key: String,
    pub embedding_model: String,
    pub timeout_ms: Option<u64>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for OpenAICompatibleConfig {
    fn default() -> Self {
        Self {
            chat_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            chat_api_key: String::new(),
            chat_model: "gpt-3.5-turbo".to_string(),
            embedding_endpoint: "https://api.openai.com/v1/embeddings".to_string(),
            embedding_api_key: String::new(),
            embedding_model: "text-embedding-ada-002".to_string(),
            timeout_ms: Some(30000), // 30 seconds
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }
}
