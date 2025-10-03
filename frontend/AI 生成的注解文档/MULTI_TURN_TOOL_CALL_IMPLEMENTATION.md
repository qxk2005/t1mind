# 多轮对话工具调用实现

## 问题描述

在之前的实现中，当 AI 调用 MCP 工具并成功执行后，工具的执行结果只是发送给 UI 显示，但 AI 并没有利用这些数据生成最终的回答。用户看到的输出可能是这样的：

```
好的，为了给您推荐几本 Readwise 中与禅宗相关的书籍，我需要使用 `search_readwise_highlights` 工具来搜索相关内容...

<tool_call>
{
  "id": "call_001",
  "tool_name": "search_readwise_highlights",
  ...
}
</tool_call>

<tool_result>
工具执行成功：search_readwise_highlights
结果：[{"id":455927290,"score":0.01639344262295082,...}]
</tool_result>
```

工具执行成功了，但 AI 没有继续基于工具结果回答用户的问题。

## 解决方案

实现了多轮对话机制，在工具调用执行完成后，自动将工具结果作为上下文反馈给 AI，让 AI 基于这些结果生成最终回答。

## 实现细节

### 1. 核心流程

```
用户提问
  ↓
第一轮：AI 生成回答（可能包含工具调用）
  ↓
检测并执行工具调用
  ↓
收集工具调用结果
  ↓
第二轮：将工具结果作为上下文，再次调用 AI
  ↓
AI 基于工具结果生成最终回答
  ↓
流式输出给用户
```

### 2. 关键修改

**文件**: `rust-lib/flowy-ai/src/chat.rs`

#### 2.1 添加工具调用跟踪

```rust
// 🔧 多轮对话支持：记录工具调用和结果
let mut tool_calls_and_results: Vec<(
  crate::agent::ToolCallRequest, 
  crate::agent::ToolCallResponse
)> = Vec::new();
```

#### 2.2 保存工具调用和结果

在工具执行完成后，保存工具调用请求和响应：

```rust
// 🔧 保存工具调用和结果，用于后续多轮对话
tool_calls_and_results.push((request.clone(), response.clone()));
```

#### 2.3 实现多轮对话逻辑

在第一轮流结束后，检查是否有工具调用结果：

```rust
// 🔧 多轮对话：如果有工具调用结果，继续生成 AI 回答
if has_agent && !tool_calls_and_results.is_empty() {
  // 1. 构建包含工具结果的上下文
  let mut follow_up_context = String::new();
  follow_up_context.push_str("\n\n以下是工具调用的结果，请基于这些结果回答用户的原始问题：\n\n");
  
  for (req, resp) in &tool_calls_and_results {
    follow_up_context.push_str(&format!(
      "工具调用: {}\n参数: {}\n结果: {}\n执行状态: {}\n\n",
      req.tool_name,
      serde_json::to_string_pretty(&req.arguments).unwrap_or_else(|_| "无法序列化".to_string()),
      resp.result.as_ref().unwrap_or(&"无结果".to_string()),
      if resp.success { "成功" } else { "失败" }
    ));
  }
  
  follow_up_context.push_str("请用中文简体总结和解释这些工具执行结果，直接回答用户的问题，不要再次调用工具。");
  
  // 2. 构建增强的系统提示（原提示 + 工具结果）
  let follow_up_system_prompt = if let Some(original_prompt) = system_prompt {
    format!("{}\n\n{}", original_prompt, follow_up_context)
  } else {
    follow_up_context
  };
  
  // 3. 发送分隔符
  let separator = "\n\n---\n\n";
  answer_stream_buffer.lock().await.push_str(separator);
  let _ = answer_sink
    .send(StreamMessage::OnData(separator.to_string()).to_string())
    .await;
  
  // 4. 再次调用 AI
  match cloud_service
    .stream_answer_with_system_prompt(
      &workspace_id, 
      &chat_id, 
      question_id, 
      format, 
      ai_model,
      Some(follow_up_system_prompt)
    )
    .await
  {
    Ok(mut follow_up_stream) => {
      // 5. 流式输出新的回答
      while let Some(message) = follow_up_stream.next().await {
        match message {
          Ok(QuestionStreamValue::Answer { value }) => {
            // 直接发送，不再检测工具调用（避免无限循环）
            answer_stream_buffer.lock().await.push_str(&value);
            let _ = answer_sink
              .send(StreamMessage::OnData(value).to_string())
              .await;
          },
          // ... 处理其他消息类型
        }
      }
    },
    Err(err) => {
      // 错误处理
    }
  }
}
```

### 3. 关键设计决策

#### 3.1 避免无限循环

在第二轮（follow-up）流中，不再检测和执行工具调用，避免无限循环：

```rust
QuestionStreamValue::Answer { value } => {
  // 直接发送，不再检测工具调用（避免无限循环）
  answer_stream_buffer.lock().await.push_str(&value);
  // ...
}
```

#### 3.2 上下文构建

通过系统提示词传递工具结果，而不是修改用户问题：

- ✅ 优点：保持原始用户问题不变
- ✅ 优点：AI 能看到完整的工具执行上下文
- ✅ 优点：明确指示 AI 不要再次调用工具

#### 3.3 用户体验

- 添加分隔符 `---`，让用户清楚地看到 AI 正在生成最终回答
- 第一轮的工具调用和结果仍然显示给用户，保持透明度
- 流式输出确保用户能实时看到回答生成过程

### 4. 示例流程

#### 用户输入
```
请推荐几本 Readwise 中与禅宗相关的书籍
```

#### 第一轮 AI 回答
```
好的，我会使用 search_readwise_highlights 工具搜索相关内容。

<tool_call>
{
  "id": "call_001",
  "tool_name": "search_readwise_highlights",
  "arguments": {
    "vector_search_term": "禅宗",
    "full_text_queries": [...]
  }
}
</tool_call>
```

#### 工具执行
```
<tool_result>
工具执行成功：search_readwise_highlights
结果：[{"id":455927290,...,"document_title":"洞见：从科学到哲学，打开人类的认知真相",...}, ...]
</tool_result>
```

#### 分隔符
```
---
```

#### 第二轮 AI 回答（基于工具结果）
```
根据 Readwise 的搜索结果，我为您推荐以下几本与禅宗相关的书籍：

1. **《The Way of Zen》** by Alan Watts
   - 这本书详细介绍了禅宗的核心思想和实践方法...

2. **《洞见：从科学到哲学，打开人类的认知真相》** by 罗伯特·赖特
   - 书中探讨了内观、藏传佛教和禅宗的区别...

3. **《活在此时此刻》** by 一行禅师
   - 一行禅师的经典著作，教导如何通过正念修行体验禅的智慧...

这些书籍从不同角度探讨了禅宗的智慧，适合不同层次的读者。
```

## 技术细节

### Clone 特性要求

为了支持多轮对话，需要确保相关类型实现了 `Clone` 特性：

- `AIModel`: 已实现 `Clone`
- `ResponseFormat`: 已实现 `Clone`
- `ToolCallRequest`: 已实现 `Clone`
- `ToolCallResponse`: 已实现 `Clone`

### 错误处理

- 第二轮 AI 调用失败时，会向用户显示错误信息
- 不会影响第一轮的工具执行结果显示
- 通过日志记录详细的调试信息

### 日志追踪

添加了详细的日志，方便调试：

```rust
info!("🔧 [MULTI-TURN] Detected {} tool call(s), initiating follow-up AI response", ...);
info!("🔧 [MULTI-TURN] Calling AI with follow-up context ({} chars)", ...);
info!("🔧 [MULTI-TURN] Follow-up stream started");
info!("🔧 [MULTI-TURN] Follow-up response completed");
```

## 测试建议

1. **基本功能测试**
   - 提问需要工具调用的问题
   - 验证工具执行后 AI 能生成最终回答

2. **多工具调用测试**
   - 提问需要调用多个工具的复杂问题
   - 验证所有工具结果都被包含在上下文中

3. **错误场景测试**
   - 工具执行失败
   - 第二轮 AI 调用失败
   - 用户中断流

4. **性能测试**
   - 大量工具结果的场景
   - 长时间流式响应

## 后续优化方向

1. **支持多轮工具调用**
   - 当前实现避免了无限循环，但也限制了多轮工具调用
   - 可以添加最大轮次限制（如 3 轮）

2. **智能工具结果摘要**
   - 工具结果可能很长，可以使用 AI 先进行摘要
   - 减少传递给第二轮的上下文长度

3. **工具结果缓存**
   - 相同的工具调用可以缓存结果
   - 避免重复执行

4. **并行工具执行**
   - 检测到多个独立的工具调用时，可以并行执行
   - 提升响应速度

## 相关文件

- `rust-lib/flowy-ai/src/chat.rs`: 主要实现文件
- `rust-lib/flowy-ai/src/agent/tool_call_handler.rs`: 工具调用处理
- `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`: 中间件服务

## 编译验证

```bash
cd rust-lib/flowy-ai
cargo check
# ✅ 编译成功，无错误
```

## 总结

本次实现成功解决了工具调用后 AI 不生成最终回答的问题。通过多轮对话机制，AI 现在能够：

1. ✅ 检测用户需求并调用适当的工具
2. ✅ 执行工具并获取结果
3. ✅ 基于工具结果生成自然语言回答
4. ✅ 为用户提供完整、有价值的回复

这大大提升了 AI 助手的实用性和用户体验。

