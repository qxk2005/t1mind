# OpenAI Compatible HTTP Client

This module provides an HTTP client for OpenAI compatible APIs, supporting both chat completions and embeddings.

## Features

- **Chat Completions**: Send chat messages and receive responses
- **Embeddings**: Generate embeddings for text input
- **Error Handling**: Comprehensive error handling with user-friendly messages
- **Security**: API key masking in logs for security
- **Timeout Configuration**: Configurable request timeouts
- **Testing**: Built-in test methods for validating API connectivity

## Usage

### Basic Setup

```rust
use flowy_ai::openai_compatible::{OpenAICompatibleClient, OpenAICompatibleConfig};

let config = OpenAICompatibleConfig {
    chat_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
    chat_api_key: "your-api-key".to_string(),
    chat_model: "gpt-3.5-turbo".to_string(),
    embedding_endpoint: "https://api.openai.com/v1/embeddings".to_string(),
    embedding_api_key: "your-api-key".to_string(),
    embedding_model: "text-embedding-ada-002".to_string(),
    timeout_ms: Some(30000),
    max_tokens: Some(4096),
    temperature: Some(0.7),
};

let client = OpenAICompatibleClient::new(config)?;
```

### Testing Connectivity

```rust
// Test chat functionality
match client.test_chat().await {
    Ok(result) => println!("Chat test: {}", result),
    Err(e) => println!("Chat test failed: {}", e),
}

// Test embedding functionality
match client.test_embedding().await {
    Ok(result) => println!("Embedding test: {}", result),
    Err(e) => println!("Embedding test failed: {}", e),
}
```

### Chat Completions

```rust
use flowy_ai::openai_compatible::{ChatCompletionRequest, ChatMessage};

let request = ChatCompletionRequest {
    model: "gpt-3.5-turbo".to_string(),
    messages: vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
        }
    ],
    max_tokens: Some(100),
    temperature: Some(0.7),
    stream: Some(false),
};

let response = client.chat_completion(request).await?;
println!("Response: {}", response.choices[0].message.content);
```

### Embeddings

```rust
use flowy_ai::openai_compatible::{EmbeddingRequest, EmbeddingInput};

let request = EmbeddingRequest {
    model: "text-embedding-ada-002".to_string(),
    input: EmbeddingInput::String("Hello world".to_string()),
};

let response = client.create_embedding(request).await?;
println!("Embedding dimensions: {}", response.data[0].embedding.len());
```

## Error Handling

The client provides detailed error messages for common scenarios:

- **401 Unauthorized**: "Authentication failed. Please check your API key."
- **404 Not Found**: "API endpoint not found. Please check your endpoint URL."
- **429 Too Many Requests**: "Rate limit exceeded. Please try again later."
- **500 Internal Server Error**: "Server error. Please try again later."
- **400 Bad Request**: Includes specific error details from the API

## Security

- API keys are automatically masked in logs (shows only first 4 and last 4 characters)
- All HTTP requests use secure headers
- Timeout protection prevents hanging requests

## Configuration

The `OpenAICompatibleConfig` struct supports:

- **chat_endpoint**: URL for chat completions API
- **chat_api_key**: API key for chat requests
- **chat_model**: Model name for chat completions
- **embedding_endpoint**: URL for embeddings API
- **embedding_api_key**: API key for embedding requests
- **embedding_model**: Model name for embeddings
- **timeout_ms**: Request timeout in milliseconds (default: 30000)
- **max_tokens**: Maximum tokens for chat completions (default: 4096)
- **temperature**: Temperature for chat completions (default: 0.7)
