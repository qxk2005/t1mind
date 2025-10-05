# 多轮对话简化方案

## 🎯 核心思路

**在 middleware 层自动处理多轮对话**，无需在 chat.rs 手动调用。

## ✅ 优势

1. **架构简洁**：逻辑集中在一处
2. **类型安全**：无需 trait object downcast
3. **透明集成**：对上层完全透明
4. **易于维护**：减少跨层调用

## 🔧 实现方案

### 方案 A：在 `openai_chat_stream_with_system` 中处理（推荐）

修改 `openai_chat_stream_with_system` 方法，在流结束后：
1. 检查是否累积了完整的 tool_calls
2. 如果有，执行工具
3. 构建新的 messages 数组
4. 递归调用自己（或循环调用）
5. 返回合并后的流

```rust
pub async fn stream_answer_with_system_prompt(...) -> Result<StreamAnswer, FlowyError> {
  // ...现有逻辑...
  
  // 如果是 OpenAI 兼容服务
  if ai_model.is_openai_compatible {
    return self.openai_chat_stream_with_multi_turn(
      cfg, model, system_prompt, content, tools, max_iterations
    ).await;
  }
}

async fn openai_chat_stream_with_multi_turn(...) -> Result<StreamAnswer, FlowyError> {
  let mut messages = vec![...];
  let mut iteration = 0;
  
  loop {
    // 调用 API
    let (tool_calls, stream) = self.openai_chat_stream_internal(...).await?;
    
    // 如果没有 tool_calls，返回流
    if tool_calls.is_empty() {
      return Ok(stream);
    }
    
    // 执行工具
    for tc in tool_calls {
      let result = execute_tool(tc).await;
      messages.push(tool_result_message);
    }
    
    iteration += 1;
    if iteration > max_iterations {
      break;
    }
    
    // 继续下一轮（循环会自动重新调用 API）
  }
}
```

### 方案 B：创建包装流（复杂但优雅）

创建一个智能流包装器，自动检测 tool_calls 并触发新的请求：

```rust
struct MultiTurnStream {
  inner: StreamAnswer,
  tool_calls_buffer: Vec<OpenAIToolCall>,
  service: Arc<ChatServiceMiddleware>,
  // ...其他状态
}

impl Stream for MultiTurnStream {
  // 自动处理多轮逻辑
}
```

## 📋 当前状态

- ✅ `continue_conversation_with_tool_results` 方法已实现
- ✅ 工具执行逻辑已实现
- ❌ 自动触发机制未实现
- ❌ 流合并逻辑未实现

## 🚀 下一步

### 立即执行：方案 A（最简单）

1. 在 `stream_answer_with_system_prompt` 中检测 AI 模型类型
2. 如果支持 tool calling，调用新的 `stream_with_auto_multi_turn` 方法
3. 该方法内部处理所有多轮对话逻辑
4. 对外返回单一的流

### 实现步骤

```bash
# 1. 修改 chat_service_mw.rs
#    - 添加 stream_with_auto_multi_turn 方法
#    - 内部使用 continue_conversation_with_tool_results

# 2. 修改 stream_answer_with_system_prompt
#    - 检测是否启用 tool calling
#    - 如果是，调用新方法
#    - 否则，使用原有流程

# 3. 测试验证
#    - 简单工具调用
#    - 多轮工具调用（工具A -> 工具B -> 最终答案）
#    - 错误处理
```

## ⚠️ 注意事项

1. **流的生命周期**：确保流在整个多轮对话过程中保持活跃
2. **错误处理**：工具执行失败时的降级策略
3. **超时控制**：多轮对话可能耗时较长
4. **用户反馈**：在 UI 显示多轮对话进度

## 🎨 用户体验

理想的用户体验：

```
用户：请帮我查询天气并根据天气推荐活动

AI：🔧 正在查询天气...
    [调用 weather_tool]
    
    根据查询结果，今天北京晴天，温度20°C
    
    🔧 正在推荐活动...
    [调用 activity_recommend_tool]
    
    根据天气情况，我推荐您：
    1. 去公园散步
    2. 户外运动
    3. ...
```

全程流式输出，用户可以实时看到进度。

