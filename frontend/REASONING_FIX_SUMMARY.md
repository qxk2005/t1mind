# AI 推理显示问题修复总结

## 问题分析

### 根本原因
当前的实现将系统提示词和用户消息**合并成一个字符串**发送给 OpenAI API：

```rust
fn apply_system_prompt(&self, content: String, system_prompt: Option<String>) -> String {
  if let Some(prompt) = system_prompt {
    format!("System Instructions:\n{}\n\n---\n\nUser Message:\n{}", prompt, content)
  } else {
    content
  }
}
```

这导致的问题：
1. **破坏了 reasoning 显示** - AI 模型无法正确识别系统指令
2. **响应质量下降** - 系统提示词格式不规范
3. **干扰 AI 的思考过程** - 长文本混合影响 AI 判断

### OpenAI API 正确格式

应该使用**独立的系统消息**：

```json
{
  "model": "deepseek-reasoner",
  "stream": true,
  "stream_options": {"include_reasoning": true},
  "messages": [
    {"role": "system", "content": "系统提示词..."},
    {"role": "user", "content": "用户消息"}
  ]
}
```

## 解决方案

### 修改点 1: 构建消息数组而非合并字符串

**位置**: `chat_service_mw.rs`

```rust
// 旧方法（错误）
fn apply_system_prompt(&self, content: String, system_prompt: Option<String>) -> String {
  if let Some(prompt) = system_prompt {
    format!("System Instructions:\n{}\n\n---\n\nUser Message:\n{}", prompt, content)
  } else {
    content
  }
}

// 新方法（正确）
fn build_messages_with_system_prompt(
  &self, 
  content: String, 
  system_prompt: Option<String>
) -> Vec<serde_json::Value> {
  let mut messages = Vec::new();
  
  // 系统消息
  if let Some(prompt) = system_prompt {
    messages.push(json!({
      "role": "system",
      "content": prompt
    }));
  }
  
  // 用户消息
  messages.push(json!({
    "role": "user",
    "content": content
  }));
  
  messages
}
```

### 修改点 2: 更新 OpenAI payload 构建

```rust
// 旧签名
fn openai_chat_payload(model: &str, content: String) -> serde_json::Value

// 新签名
fn openai_chat_payload(model: &str, messages: Vec<serde_json::Value>) -> serde_json::Value
```

### 修改点 3: 添加带系统提示词的流式调用方法

```rust
async fn openai_chat_stream_with_system(
  &self,
  cfg: &OpenAICompatConfig,
  model: Option<&str>,
  content: String,
  system_prompt: Option<String>,
) -> Result<(Option<String>, StreamAnswer), FlowyError> {
  // 构建消息数组
  let messages = self.build_messages_with_system_prompt(content, system_prompt);
  let payload = Self::openai_chat_payload(model_name, messages);
  
  // 发送请求...
  // 处理 reasoning 流式响应...
}
```

### 修改点 4: 更新调用点

在 `stream_answer_with_system_prompt` 中：

```rust
// 调用新方法，传递独立的系统提示词和内容
let (_init_reasoning, stream) = self
  .openai_chat_stream_with_system(&cfg, Some(&ai_model.name), content, system_prompt)
  .await?;
```

## 预期效果

修复后：
- ✅ **Reasoning 正常显示** - DeepSeek/o1 的思考过程会正确显示
- ✅ **系统提示词生效** - AI 会遵循智能体的个性和能力配置
- ✅ **响应质量提升** - AI 能够正确理解系统指令
- ✅ **保持兼容性** - 不影响没有智能体的普通聊天

## 实施状态

由于编辑冲突，当前修改需要手动应用。建议步骤：

1. **备份当前文件**
   ```bash
   cp rust-lib/flowy-ai/src/middleware/chat_service_mw.rs chat_service_mw.rs.backup
   ```

2. **手动应用上述修改**
   - 将 `apply_system_prompt` 改为 `build_messages_with_system_prompt`
   - 更新 `openai_chat_payload` 签名
   - 添加 `openai_chat_stream_with_system` 方法
   - 更新调用点

3. **测试验证**
   - 编译确认无错误
   - 测试普通聊天（无智能体）
   - 测试智能体聊天
   - 验证 reasoning 显示

## 关键文件

- `rust-lib/flowy-ai/src/middleware/chat_service_mw.rs` - 需要修改
- `rust-lib/flowy-ai/src/chat.rs` - 调用方（已正确）
- `rust-lib/flowy-ai/src/agent/agent_capabilities.rs` - 生成系统提示词（已正确）

## 技术要点

### 为什么要独立系统消息？

1. **OpenAI API 标准** - 官方推荐方式
2. **Token 计算准确** - 系统消息有不同的权重
3. **上下文管理清晰** - 便于 AI 理解角色
4. **Reasoning 显示正常** - 不影响思考过程的识别

### 本地 AI 兼容性

本地 AI（Ollama）可能不支持 `system` role，所以我们对本地 AI 仍然使用简单合并：

```rust
if ai_model.is_local {
  // 本地 AI: 简单合并
  let final_content = if let Some(ref prompt) = system_prompt {
    format!("{}\n\n{}", prompt, content)
  } else {
    content
  };
  self.local_ai.stream_question(chat_id, &final_content, format, &ai_model.name).await
}
```

## 下一步

1. ⏳ 手动应用修改（文件已恢复，等待重新编辑）
2. ⏳ 编译测试
3. ⏳ 端到端测试
4. ⏳ 更新文档

---

**创建时间**: 2025-10-02
**状态**: 待实施（需要手动应用修改）

