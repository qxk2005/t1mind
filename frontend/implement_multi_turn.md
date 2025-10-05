# 多轮对话优化实现方案

## 📋 目标

实现完整的 OpenAI Function Call 多轮对话流程：
1. 检测 AI 返回的 tool_calls
2. 自动执行工具
3. 将工具结果反馈给 AI
4. 继续对话直到获得最终答案

## 🔄 实现流程

### 方案 A：在 Middleware 层实现（推荐）

在 `chat_service_mw.rs` 中添加一个包装方法：

```rust
/// 带工具调用的多轮对话
pub async fn stream_answer_with_tools(
  &self,
  workspace_id: &Uuid,
  chat_id: &Uuid,
  question_id: i64,
  format: ResponseFormat,
  ai_model: AIModel,
  system_prompt: Option<String>,
  tools: Option<Vec<ToolDefinitionPB>>,
  tool_handler: Option<Arc<ToolCallHandler>>,
  max_iterations: usize,
) -> Result<StreamAnswer, FlowyError> {
  
  // 构建初始消息历史
  let mut messages = Vec::new();
  if let Some(prompt) = system_prompt {
    messages.push(OpenAIMessage {
      role: "system".to_string(),
      content: Some(prompt),
      tool_calls: None,
      tool_call_id: None,
      name: None,
    });
  }
  
  // 获取用户消息内容
  let user_content = self.get_message_content(question_id)?;
  messages.push(OpenAIMessage {
    role: "user".to_string(),
    content: Some(user_content),
    tool_calls: None,
    tool_call_id: None,
    name: None,
  });
  
  let mut iteration = 0;
  
  loop {
    iteration += 1;
    if iteration > max_iterations {
      return Err(FlowyError::internal()
        .with_context("Max tool call iterations reached"));
    }
    
    // 调用 AI
    let (tool_calls_detected, response_stream) = self
      .call_openai_and_detect_tools(cfg, model, &messages, tools)
      .await?;
    
    // 如果没有工具调用，返回最终响应流
    if tool_calls_detected.is_empty() {
      return Ok(response_stream);
    }
    
    // 执行工具并添加结果到消息历史
    let mut assistant_message = OpenAIMessage {
      role: "assistant".to_string(),
      content: None,
      tool_calls: Some(tool_calls_detected.clone()),
      tool_call_id: None,
      name: None,
    };
    messages.push(assistant_message);
    
    // 执行每个工具调用
    for tool_call in &tool_calls_detected {
      let result = if let Some(ref handler) = tool_handler {
        handler.execute_tool_call_from_openai(tool_call).await
      } else {
        Err(FlowyError::internal().with_context("Tool handler not available"))
      };
      
      let result_content = match result {
        Ok(response) => response.result.unwrap_or_else(|| "No result".to_string()),
        Err(e) => format!("Error: {}", e),
      };
      
      // 添加工具结果消息
      messages.push(OpenAIMessage {
        role: "tool".to_string(),
        content: Some(result_content),
        tool_calls: None,
        tool_call_id: Some(tool_call.id.clone()),
        name: Some(tool_call.function.name.clone()),
      });
    }
    
    // 继续下一轮
  }
}
```

### 方案 B：在 Chat 层实现

在 `chat.rs` 的流式响应处理中：
1. 收集完整响应和 tool_calls
2. 如果检测到 tool_calls，执行工具
3. 构建新的请求继续对话

### 方案 C：混合方案（最佳）

1. Middleware 提供非流式的多轮对话方法
2. Chat 层处理流式响应并在结束时触发多轮对话
3. 将多轮结果作为额外的流式数据发送给前端

## 🎯 推荐实现：方案 C 的简化版

### 步骤 1: 在 Middleware 添加工具调用检测和执行

```rust
// 在 chat_service_mw.rs 中添加

/// 检查响应中是否有 tool_calls（非流式）
async fn check_for_tool_calls(
  &self,
  response_json: &serde_json::Value
) -> Option<Vec<OpenAIToolCall>> {
  let choices = response_json.get("choices")?.as_array()?;
  let first_choice = choices.get(0)?;
  let message = first_choice.get("message")?;
  let tool_calls = message.get("tool_calls")?.as_array()?;
  
  let mut result = Vec::new();
  for tc in tool_calls {
    if let Ok(call) = serde_json::from_value::<OpenAIToolCall>(tc.clone()) {
      result.push(call);
    }
  }
  
  if result.is_empty() {
    None
  } else {
    Some(result)
  }
}

/// 执行多轮对话（在流结束后调用）
pub async fn continue_with_tool_results(
  &self,
  workspace_id: &Uuid,
  chat_id: &Uuid,
  question_id: i64,
  tool_calls: Vec<OpenAIToolCall>,
  tool_handler: &ToolCallHandler,
  original_messages: Vec<OpenAIMessage>,
  model: &str,
  tools: &[ToolDefinitionPB],
  max_iterations: usize,
) -> Result<StreamAnswer, FlowyError> {
  // 实现多轮对话逻辑
}
```

### 步骤 2: 在 Chat 层集成

```rust
// 在 chat.rs 的流式响应处理后

// 检查是否收集到 tool_calls
if !collected_tool_calls.is_empty() && has_tool_handler {
  info!("🔧 [MULTI-TURN] Detected {} tool calls, starting multi-turn conversation", 
        collected_tool_calls.len());
  
  // 调用 middleware 继续多轮对话
  match cloud_service.continue_with_tool_results(
    &workspace_id,
    &chat_id,
    question_id,
    collected_tool_calls,
    tool_call_handler.as_ref().unwrap(),
    original_messages,
    &ai_model.name,
    &tool_definitions,
    3  // max_iterations
  ).await {
    Ok(continuation_stream) => {
      // 发送分隔符
      let separator = "\n\n---\n\n";
      answer_stream_buffer.lock().await.push_str(separator);
      let _ = answer_sink.send(StreamMessage::OnData(separator.to_string()).to_string()).await;
      
      // 流式发送续接的响应
      while let Some(msg) = continuation_stream.next().await {
        // 处理续接响应...
      }
    }
    Err(e) => {
      error!("🔧 [MULTI-TURN] Failed to continue conversation: {}", e);
    }
  }
}
```

## ⚡ 快速实现：最小可行方案

由于完整实现较复杂，建议先实现一个简化版本：

1. **在流式响应中收集 tool_calls**
2. **流结束后自动执行工具**
3. **构建包含结果的新消息并再次调用 AI**
4. **将新响应追加到原始响应后**

这样可以：
- ✅ 快速实现基本的多轮对话
- ✅ 不需要大量修改现有架构
- ✅ 用户体验良好（看到工具调用 → 执行 → 最终答案）

## 📝 实现清单

- [ ] 在流式响应中收集 tool_calls 元数据
- [ ] 添加 tool_call 累积逻辑
- [ ] 在流结束时检查是否有 tool_calls
- [ ] 执行收集到的工具
- [ ] 构建新的 messages 数组（包含 assistant + tool results）
- [ ] 再次调用 OpenAI API
- [ ] 流式输出新的响应
- [ ] 添加最大迭代次数限制

## 🎯 下一步

让我知道你想采用哪种方案，我会立即开始实现！

推荐：**快速实现最小可行方案**，然后根据实际使用情况进行优化。

