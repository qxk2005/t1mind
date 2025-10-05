# OpenAI Function Call 迁移指南

## 📋 项目概述

本项目正在将手工实现的 tool call 功能（基于 `<tool_call>` 标签检测）迁移到使用 OpenAI 标准的 Function Call API。

## ✅ 已完成的工作

### 1. 基础架构更新

#### 1.1 新增数据结构 (`chat_service_mw.rs`)

```rust
/// OpenAI Tool Call 响应结构
struct OpenAIToolCall {
  pub id: String,
  pub tool_type: String,
  pub function: OpenAIFunctionCall,
}

struct OpenAIFunctionCall {
  pub name: String,
  pub arguments: String, // JSON string
}

/// OpenAI Message 结构（用于多轮对话）
struct OpenAIMessage {
  pub role: String,
  pub content: Option<String>,
  pub tool_calls: Option<Vec<OpenAIToolCall>>,
  pub tool_call_id: Option<String>,
  pub name: Option<String>,
}
```

#### 1.2 工具定义转换

新增方法 `convert_tools_to_openai_format`，将内部的 `ToolDefinitionPB` 转换为 OpenAI tools 格式：

```rust
fn convert_tools_to_openai_format(tools: &[ToolDefinitionPB]) -> Vec<serde_json::Value> {
  // 转换为 OpenAI function 格式
  json!({
    "type": "function",
    "function": {
      "name": tool.name,
      "description": tool.description,
      "parameters": parameters_schema
    }
  })
}
```

### 2. API 请求更新

#### 2.1 修改 `stream_answer_with_system_prompt` 方法签名

```rust
pub async fn stream_answer_with_system_prompt(
  &self,
  workspace_id: &Uuid,
  chat_id: &Uuid,
  question_id: i64,
  format: ResponseFormat,
  ai_model: AIModel,
  system_prompt: Option<String>,
  tools: Option<Vec<ToolDefinitionPB>>,  // 🆕 新增参数
) -> Result<StreamAnswer, FlowyError>
```

#### 2.2 更新 `openai_chat_stream_with_system` 方法

- 添加 `tools` 参数
- 在请求 payload 中添加 `tools` 和 `tool_choice` 字段
- 启用 OpenAI Function Call API

```rust
if let Some(tool_list) = tools {
  if !tool_list.is_empty() {
    let openai_tools = Self::convert_tools_to_openai_format(tool_list);
    payload.insert("tools", json!(openai_tools));
    payload.insert("tool_choice", json!("auto"));  // 让模型自动决定
  }
}
```

### 3. 流式响应解析更新

在 `openai_chat_stream_with_system` 中添加了对 `tool_calls` 字段的解析：

```rust
// 处理 tool_calls（OpenAI Function Call API）
if let Some(tool_calls) = delta.get("tool_calls") {
  // 累积 tool_call 信息
  // 发送元数据通知前端
  yield QuestionStreamValue::Metadata {
    value: json!({
      "tool_call": {
        "id": tc.id,
        "tool_name": tc.function.name,
        "arguments": tc.function.arguments,
        "status": "pending"
      }
    })
  };
}
```

### 4. 工具获取辅助方法

在 `ai_manager.rs` 中添加：

```rust
/// 根据工具名称列表获取工具定义
pub async fn get_tool_definitions_by_names(&self, tool_names: &[String]) -> Vec<ToolDefinitionPB> {
  let all_tools = self.agent_manager.get_all_available_tools().await;
  all_tools.into_iter()
    .filter(|tool| tool_names.contains(&tool.name))
    .collect()
}
```

## ✅ 最新进展（2025-01-04）

### 已完成的基础架构（90%）

1. ✅ **数据结构** - 添加了 OpenAI Tool Call 相关结构体
2. ✅ **API 集成** - middleware 已支持发送 tools 参数和解析 tool_calls
3. ✅ **工具获取** - AIManager 可以获取并传递工具定义
4. ✅ **System Prompt** - 更新为 OpenAI Function Call 风格（无需手工协议说明）
5. ⚠️  **调用链路** - 需要完成 Chat::stream_chat_message 的参数传递（详见 MIGRATION_SUMMARY.md）

### 当前状态

基础架构已基本完成，middleware 层可以：
- ✅ 将工具定义转换为 OpenAI tools 格式
- ✅ 在请求中添加 tools 参数
- ✅ 流式解析 tool_calls 响应
- ✅ 通过 Metadata 通知前端

**待完成最后一步**：需要在 `chat.rs` 中完成参数传递链路，详细步骤见 `MIGRATION_SUMMARY.md`。

---

## 🚧 待完成的工作（遗留项）

### 1. 多轮对话实现

当前流式响应解析可以检测到 tool_call，但还需要实现完整的多轮对话流程：

1. **Tool Call 检测** ✅ （已完成）
   - 流式响应中累积 `tool_calls` 字段
   - 发送元数据通知前端

2. **Tool 执行** ⏳ （待实现）
   - 在检测到完整 tool_call 后暂停流式响应
   - 调用 `ToolCallHandler::execute_tool_call` 执行工具
   - 获取工具执行结果

3. **继续对话** ⏳ （待实现）
   - 构建包含工具结果的新消息：
     ```json
     {
       "role": "tool",
       "tool_call_id": "call_xxx",
       "content": "工具执行结果"
     }
     ```
   - 将完整对话历史（包括用户消息、assistant 的 tool_call、工具结果）发送回 AI
   - 继续流式接收 AI 的最终回答

4. **错误处理** ⏳ （待实现）
   - 工具执行失败时的错误消息构建
   - 超时处理
   - 重试机制

### 2. 更新调用点

需要更新所有调用 `stream_answer_with_system_prompt` 的地方，传递工具列表：

**位置：`rust-lib/flowy-ai/src/chat.rs`**

```rust
// 当前调用（第 292 行）
cloud_service.stream_answer_with_system_prompt(
  &workspace_id, 
  &chat_id, 
  question_id, 
  format.clone(), 
  ai_model.clone(), 
  system_prompt.clone()
)

// 需要改为
let tool_defs = if let Some(ref config) = agent_config {
  ai_manager.get_tool_definitions_by_names(&config.available_tools).await
} else {
  vec![]
};

cloud_service.stream_answer_with_system_prompt(
  &workspace_id, 
  &chat_id, 
  question_id, 
  format.clone(), 
  ai_model.clone(), 
  system_prompt.clone(),
  Some(tool_defs)  // 🆕 传递工具定义
)
```

### 3. 移除旧的 `<tool_call>` 标签检测

一旦 Function Call API 实现完成并测试通过，需要：

1. 移除 `chat.rs` 中的手工标签检测代码（约第 305-500 行）
2. 移除 `ToolCallProtocol` 中的标签相关常量和方法
3. 更新系统提示词，移除工具调用协议说明
4. 清理 `stream_tool_handler.rs` 中的旧实现

### 4. System Prompt 更新

由于使用 OpenAI Function Call API，不再需要在 system prompt 中包含工具调用格式说明：

**修改位置：`rust-lib/flowy-ai/src/agent/system_prompt.rs`**

```rust
// 移除以下部分（第 56-77 行）
prompt.push_str("\n  **Tool Calling Protocol:**\n");
prompt.push_str("  When you need to use a tool, DIRECTLY output...");
// ... 删除整个 tool calling protocol 部分
```

因为现在工具信息通过 API 的 `tools` 参数传递，AI 会自动按照 OpenAI 标准格式返回 function call。

### 5. 前端适配

可能需要更新前端代码以处理新的元数据格式：

```dart
// 新格式
{
  "tool_call": {
    "id": "call_xxx",
    "tool_name": "search_documents",
    "arguments": "{\"query\":\"xxx\"}",
    "status": "pending"  // or "running", "success", "failed"
  }
}
```

### 6. 测试

完成实现后需要进行全面测试：

1. **单工具调用测试**
   - 简单的工具调用（如搜索）
   - 验证参数解析正确性

2. **多工具调用测试**
   - 连续调用多个工具
   - 验证多轮对话正常工作

3. **错误处理测试**
   - 工具不存在
   - 参数错误
   - 工具执行失败
   - 超时场景

4. **性能测试**
   - 大参数工具调用
   - 高频工具调用
   - 并发场景

## 📝 实现建议

### 多轮对话实现方案

建议在 `chat_service_mw.rs` 中添加一个新方法处理完整的 tool call 流程：

```rust
async fn handle_function_call_conversation(
  &self,
  cfg: &OpenAICompatConfig,
  model: &str,
  initial_messages: Vec<OpenAIMessage>,
  tools: &[ToolDefinitionPB],
  tool_handler: &ToolCallHandler,
  max_iterations: usize,
) -> Result<StreamAnswer, FlowyError> {
  let mut messages = initial_messages;
  let mut iterations = 0;
  
  loop {
    if iterations >= max_iterations {
      return Err(FlowyError::internal()
        .with_context("Max tool call iterations reached"));
    }
    
    // 1. 调用 AI
    let response = self.call_openai_chat(cfg, model, &messages, tools).await?;
    
    // 2. 检查是否有 tool_calls
    if let Some(tool_calls) = response.tool_calls {
      // 3. 执行工具
      for tool_call in tool_calls {
        let result = tool_handler.execute_tool_call(&tool_call).await;
        
        // 4. 添加工具结果到消息历史
        messages.push(OpenAIMessage {
          role: "tool".to_string(),
          tool_call_id: Some(tool_call.id),
          content: Some(result),
          ..Default::default()
        });
      }
      
      iterations += 1;
      // 继续下一轮
      continue;
    }
    
    // 5. 没有 tool_calls，返回最终响应
    return Ok(response.content_stream);
  }
}
```

### 渐进式迁移策略

为了降低风险，建议采用渐进式迁移：

1. **阶段 1：双模式运行** ⏳
   - 通过配置开关控制使用新/旧实现
   - 默认使用旧实现，可选启用新实现
   - 便于对比测试和快速回退

2. **阶段 2：新实现为默认** ⏳
   - 新实现设为默认
   - 保留旧实现作为降级方案

3. **阶段 3：完全移除旧代码** ⏳
   - 新实现稳定后移除旧代码
   - 清理相关文档和注释

## 🔍 验证清单

在完成迁移后，请验证以下功能：

- [ ] AI 能正确调用单个工具
- [ ] AI 能连续调用多个工具
- [ ] 工具参数解析正确
- [ ] 工具执行结果正确返回给 AI
- [ ] AI 能基于工具结果生成最终答案
- [ ] 错误场景处理正常
- [ ] 前端 UI 正确显示工具调用状态
- [ ] 性能没有明显下降
- [ ] 日志完整清晰

## 📚 相关文档

- [OpenAI Function Calling 官方文档](https://platform.openai.com/docs/guides/function-calling)
- 项目内部文档：
  - `AI 生成的注解文档/TOOL_CALL_INTEGRATION_COMPLETE.md`
  - `AI 生成的注解文档/MULTI_TURN_TOOL_CALL_IMPLEMENTATION.md`

## 🤝 贡献指南

如果你要继续完成剩余工作，请：

1. 从"待完成的工作"部分选择一项任务
2. 创建功能分支：`git checkout -b feature/function-call-xxx`
3. 实现功能并添加测试
4. 提交 PR 并关联此文档

## 📞 联系方式

如有问题，请联系项目维护者或在 Issue 中讨论。

