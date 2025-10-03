# Agent Backend Implementation Plan - 智能体后端实现计划

## 当前状态

### 已完成 ✅
1. **Protobuf 定义已更新**
   - `StreamChatPayloadPB` 添加了 `agent_id` 字段 (index=8)
   - `StreamMessageParams` 添加了 `agent_id` 字段
   - Protobuf 代码已重新生成

2. **前端已完成**
   - `ChatStreamManager` 更新支持 `agent_id` 参数
   - `ChatBloc` 添加了 `selectedAgentId` 字段和 `selectAgent` 事件
   - `AgentSelector` 现在通知 `ChatBloc` 智能体选择变化
   - 发送消息时传递 `agent_id` 到后端

3. **后端事件处理已更新**
   - `event_handler.rs` 的 `stream_chat_message_handler` 提取 `agent_id`
   - `agent_id` 传递给 `StreamMessageParams`

### 待实施 🔧

#### P1.3 - 后端处理 agent_id 并加载配置

需要在以下位置实现：

1. **ai_manager.rs:stream_chat_message**
   ```rust
   pub async fn stream_chat_message(
     &self,
     params: StreamMessageParams,
   ) -> Result<ChatMessagePB, FlowyError> {
     // 1. 如果有 agent_id，加载智能体配置
     let agent_config = if let Some(ref agent_id) = params.agent_id {
       match self.agent_manager.get_agent_config(agent_id) {
         Ok(config) => {
           info!("[Chat] Using agent: {} ({})", config.name, config.id);
           Some(config)
         },
         Err(err) => {
           warn!("[Chat] Failed to load agent {}: {:?}", agent_id, err);
           None
         }
       }
     } else {
       None
     };
     
     let chat = self.get_or_create_chat_instance(&params.chat_id).await?;
     let ai_model = self.get_active_model(&params.chat_id.to_string()).await;
     
     // 2. 将智能体配置传递给 chat
     let question = chat.stream_chat_message(&params, ai_model, agent_config).await?;
     
     // ... 其余代码不变
   }
   ```

2. **chat.rs:stream_chat_message**
   ```rust
   pub async fn stream_chat_message(
     &self,
     params: &StreamMessageParams,
     preferred_ai_model: AIModel,
     agent_config: Option<AgentConfigPB>,
   ) -> Result<ChatMessagePB, FlowyError> {
     // ... 现有代码 ...
     
     // 在调用 create_question 时传递 agent_config
     let question = self
       .chat_service
       .create_question(
         &workspace_id,
         &self.chat_id,
         &params.message,
         params.message_type.clone(),
         params.prompt_id.clone(),
         agent_config.clone(),  // 新增
       )
       .await?;
     
     // ... 其余代码不变
   }
   ```

3. **middleware/chat_service_mw.rs:create_question**
   ```rust
   pub async fn create_question(
     &self,
     workspace_id: &Uuid,
     chat_id: &Uuid,
     message: &str,
     message_type: ChatMessageType,
     prompt_id: Option<String>,
     agent_config: Option<AgentConfigPB>,  // 新增参数
   ) -> Result<ChatMessage, FlowyError> {
     // ... 现有代码 ...
     
     // 如果有智能体配置，构建系统提示词
     let system_prompt = if let Some(ref config) = agent_config {
       build_agent_system_prompt(config)
     } else {
       None
     };
     
     // 将系统提示词传递到云服务或本地AI
     self.cloud_service
       .create_question(
         workspace_id,
         chat_id,
         message,
         message_type,
         prompt_id,
         system_prompt,  // 新增
       )
       .await
   }
   ```

#### P1.4 - 实现根据智能体配置构建系统提示词

创建新文件 `rust-lib/flowy-ai/src/agent/system_prompt.rs`:

```rust
use crate::entities::AgentConfigPB;

/// 根据智能体配置构建系统提示词
pub fn build_agent_system_prompt(config: &AgentConfigPB) -> String {
  let mut prompt = String::new();
  
  // 1. 基础描述
  if !config.description.is_empty() {
    prompt.push_str(&format!("# Agent Description\n{}\n\n", config.description));
  }
  
  // 2. 个性设置
  if !config.personality.is_empty() {
    prompt.push_str(&format!("# Personality\n{}\n\n", config.personality));
  }
  
  // 3. 能力说明
  if config.has_capabilities() {
    let cap = &config.capabilities;
    prompt.push_str("# Capabilities\n");
    
    if cap.enable_planning {
      prompt.push_str(&format!(
        "- Task Planning: You can break down complex tasks into steps (max {} steps)\n",
        cap.max_planning_steps
      ));
    }
    
    if cap.enable_tool_calling && !config.available_tools.is_empty() {
      prompt.push_str(&format!(
        "- Tool Calling: You have access to the following tools: {:?}\n",
        config.available_tools
      ));
      prompt.push_str(&format!(
        "  (max {} tool calls per conversation)\n",
        cap.max_tool_calls
      ));
    }
    
    if cap.enable_reflection {
      prompt.push_str("- Self-Reflection: You should reflect on your responses and improve them\n");
    }
    
    if cap.enable_memory {
      prompt.push_str(&format!(
        "- Conversation Memory: Remember the last {} messages in the conversation\n",
        cap.memory_limit
      ));
    }
    
    prompt.push_str("\n");
  }
  
  // 4. 额外元数据
  if !config.metadata.is_empty() {
    prompt.push_str("# Additional Information\n");
    for (key, value) in &config.metadata {
      prompt.push_str(&format!("- {}: {}\n", key, value));
    }
    prompt.push_str("\n");
  }
  
  // 5. 总结
  prompt.push_str(&format!(
    "Please act according to the above description and capabilities as the agent \"{}\".",
    config.name
  ));
  
  prompt
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::entities::{AgentCapabilitiesPB, AgentStatusPB};
  
  #[test]
  fn test_build_system_prompt() {
    let mut config = AgentConfigPB {
      id: "test-1".to_string(),
      name: "Test Agent".to_string(),
      description: "A helpful coding assistant".to_string(),
      personality: "Friendly and professional".to_string(),
      capabilities: AgentCapabilitiesPB {
        enable_planning: true,
        enable_tool_calling: true,
        enable_reflection: true,
        enable_memory: true,
        max_planning_steps: 10,
        max_tool_calls: 20,
        memory_limit: 100,
      },
      available_tools: vec!["calculator".to_string(), "search".to_string()],
      status: AgentStatusPB::AgentActive,
      ..Default::default()
    };
    
    let prompt = build_agent_system_prompt(&config);
    
    assert!(prompt.contains("# Agent Description"));
    assert!(prompt.contains("A helpful coding assistant"));
    assert!(prompt.contains("# Personality"));
    assert!(prompt.contains("Friendly and professional"));
    assert!(prompt.contains("# Capabilities"));
    assert!(prompt.contains("Task Planning"));
    assert!(prompt.contains("Tool Calling"));
    assert!(prompt.contains("Self-Reflection"));
    assert!(prompt.contains("Conversation Memory"));
  }
}
```

## 实施步骤

### 步骤 1: 创建系统提示词构建器
```bash
# 创建新文件
touch rust-lib/flowy-ai/src/agent/system_prompt.rs

# 在 rust-lib/flowy-ai/src/agent/mod.rs 中添加模块
echo "pub mod system_prompt;" >> rust-lib/flowy-ai/src/agent/mod.rs
```

### 步骤 2: 更新 AIManager
在 `rust-lib/flowy-ai/src/ai_manager.rs` 的 `stream_chat_message` 方法中添加智能体配置加载逻辑。

### 步骤 3: 更新 Chat
在 `rust-lib/flowy-ai/src/chat.rs` 的 `stream_chat_message` 方法签名中添加 `agent_config` 参数。

### 步骤 4: 更新中间件
在 `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs` 的 `create_question` 中处理智能体系统提示词。

### 步骤 5: 更新云服务接口
可能需要在 `flowy-ai-pub` 中更新 `ChatCloudService` trait 以支持系统提示词。

## 对话历史限制实现

### 方案 1: 在消息创建时限制（推荐）
```rust
// 在 create_question 方法中
if let Some(ref config) = agent_config {
  if config.capabilities.enable_memory {
    // 从数据库加载历史消息
    let history = load_chat_history(chat_id, config.capabilities.memory_limit).await?;
    
    // 将历史消息传递给云服务
    message_with_history = build_message_with_history(message, &history);
  }
}
```

### 方案 2: 在云服务层限制
在 `ChatCloudService` 实现中，根据智能体配置限制发送到AI的历史消息数量。

## 工具调用集成（P2 优先级）

### 实现概要
1. **工具注册**: 在智能体配置中注册可用工具
2. **工具解析**: 解析AI返回中的工具调用请求
3. **工具执行**: 执行工具并返回结果
4. **结果整合**: 将工具执行结果反馈给AI

### 示例实现
```rust
// 在 stream_response 中
if let Some(tool_call) = parse_tool_call(&ai_response) {
  if agent_config.available_tools.contains(&tool_call.name) {
    let tool_result = execute_tool(&tool_call).await?;
    // 将结果返回给AI继续对话
  }
}
```

## 测试计划

### 单元测试
- [ ] `build_agent_system_prompt` 正确构建系统提示词
- [ ] 智能体配置加载正确
- [ ] 对话历史限制生效

### 集成测试
- [ ] 选择智能体后，系统提示词正确传递
- [ ] 不同智能体配置生成不同的回复风格
- [ ] 对话历史限制按配置工作

### 端到端测试
- [ ] 创建具有特定个性的智能体
- [ ] 在聊天中使用智能体
- [ ] 验证回复符合智能体配置

## 已知限制

1. **OpenAI兼容服务器**: 需要确认是否支持系统提示词
2. **本地AI**: 需要确认 Ollama 如何处理系统提示词
3. **工具调用**: 需要完整的工具注册和执行机制
4. **对话历史**: 可能需要优化数据库查询性能

## 下一步行动

1. ✅ 创建 `system_prompt.rs` 文件
2. ✅ 实现 `build_agent_system_prompt` 函数
3. ✅ 更新 `AIManager::stream_chat_message` 加载智能体配置
4. ✅ 更新 `Chat::stream_chat_message` 接受智能体配置
5. ✅ 在 Chat 层面处理系统提示词（简化实现）
6. ✅ 编译测试成功

**注**: 步骤5从"更新中间件"改为"在Chat层面处理"，因为修改 ChatCloudService trait 会影响所有实现。采用更简单的方案：在 Chat::stream_chat_message 中将系统提示词附加到用户消息前面。

---

**创建时间**: 2025-10-01
**完成时间**: 2025-10-02
**状态**: ✅ 完成
**优先级**: P1 (高)

