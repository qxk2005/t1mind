# AI 未生成工具调用 - 调试指南

## 📋 问题描述

工具详情已成功加载并传递给 AI,但 AI 生成的响应中没有包含 `<tool_call>` 标签,因此没有执行任何工具。

从日志可以看到:
- ✅ 工具发现成功: `[Tool Discovery] 共从 1 个已配置服务器发现 25 个可用工具`
- ✅ 工具详情加载成功: `[Chat] Discovered 25 tools with 25 tool details`
- ✅ 使用增强提示: `[Chat] 🔧 Using enhanced system prompt with 25 tool details`
- ✅ 使用自定义提示: `[Chat] 🔧 Using custom system prompt (with tool details)`
- ❌ **没有工具调用日志**: 缺少 `[TOOL]` 相关的日志

## 🔍 可能的原因

### 1. AI 没有生成 `<tool_call>` 标签
**原因**: AI 模型可能:
- 不理解工具调用协议
- 系统提示中的工具调用指令不够明确
- 模型能力不足,无法正确使用工具
- 使用的模型(google/gemma-3-27b)未针对工具调用进行训练

### 2. 系统提示格式问题
**原因**: 增强的系统提示可能:
- 格式不符合模型期望
- 工具说明太长,导致模型困惑
- 缺少明确的工具调用示例

### 3. 模型配置问题
**原因**: 
- 模型可能不支持工具调用
- 需要额外的配置参数(如 function calling)
- OpenAI 兼容服务器未正确处理系统提示

## 🛠️ 调试步骤

### Step 1: 查看 AI 实际返回的内容

我已经添加了调试日志,重新运行应用并测试相同的问题,查找以下日志:

```
🔧 [DEBUG] Accumulated text length: X chars
🔧 [DEBUG] Current text: (AI返回的文本)
🔧 [DEBUG] Stream ended with accumulated text length: X chars
🔧 [DEBUG] Final text preview: (完整响应预览)
🔧 [DEBUG] Final check - has <tool_call>: false, has </tool_call>: false
```

**关键信息**:
- AI 是否生成了 `<tool_call>` 标签?
- AI 的实际响应内容是什么?
- AI 是否理解了用户的工具使用意图?

### Step 2: 检查系统提示内容

添加临时调试日志查看完整的系统提示:

```rust
// rust-lib/flowy-ai/src/chat.rs, line ~102
let system_prompt = if let Some(custom_prompt) = custom_system_prompt {
  info!("[Chat] 🔧 Using custom system prompt (with tool details)");
  // 🐛 临时添加: 打印完整的系统提示
  info!("=== SYSTEM PROMPT START ===\n{}\n=== SYSTEM PROMPT END ===", custom_prompt);
  Some(custom_prompt)
}
```

**检查内容**:
- 工具详情是否包含在提示中?
- 工具调用协议说明是否清晰?
- 是否有工具使用示例?

### Step 3: 验证工具调用协议

检查 `system_prompt.rs` 中的工具调用指令是否明确。当前应该包含:

```markdown
## Tool Usage Protocol

When you need to use a tool, output:
<tool_call>
{
  "id": "unique_call_id",
  "tool_name": "tool_name_here",
  "arguments": {
    "param1": "value1",
    "param2": "value2"
  }
}
</tool_call>

Do NOT include the "source" field.
```

### Step 4: 测试不同的 AI 模型

当前使用的是 `google/gemma-3-27b`,尝试其他模型:

1. **GPT-4 / GPT-3.5-turbo**: OpenAI 官方模型,支持 function calling
2. **Claude**: Anthropic 模型,支持工具使用
3. **Qwen**: 通义千问,支持工具调用
4. **GLM-4**: 智谱 AI,支持工具调用

检查不同模型是否能够生成 `<tool_call>` 标签。

## 🎯 修复方案

### 方案 1: 增强系统提示中的工具调用指令

修改 `build_agent_system_prompt_with_tools` 函数,添加更明确的指令:

```rust
// rust-lib/flowy-ai/src/agent/system_prompt.rs

pub fn build_agent_system_prompt_with_tools(
  config: &AgentConfigPB,
  tool_details: &HashMap<String, MCPTool>,
) -> String {
  let mut prompt = build_agent_system_prompt(config);
  
  if config.capabilities.enable_tool_calling && !tool_details.is_empty() {
    prompt.push_str("\n\n## 🔧 Available Tools (Detailed Information)\n\n");
    prompt.push_str("You have access to the following tools. **USE THEM ACTIVELY** when the user's request requires external data or operations.\n\n");
    
    // ... 添加工具详情 ...
    
    prompt.push_str("\n\n## ⚠️ IMPORTANT: Tool Calling Protocol\n\n");
    prompt.push_str("When you need to use a tool:\n");
    prompt.push_str("1. Analyze the user's request and identify which tool to use\n");
    prompt.push_str("2. Extract the required parameters from the user's message\n");
    prompt.push_str("3. Generate a tool call using this EXACT format:\n\n");
    prompt.push_str("```\n");
    prompt.push_str("<tool_call>\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"id\": \"call_001\",\n");
    prompt.push_str("  \"tool_name\": \"read_data_from_excel\",\n");
    prompt.push_str("  \"arguments\": {\n");
    prompt.push_str("    \"filepath\": \"myfile.xlsx\",\n");
    prompt.push_str("    \"sheet_name\": \"Sheet1\"\n");
    prompt.push_str("  }\n");
    prompt.push_str("}\n");
    prompt.push_str("</tool_call>\n");
    prompt.push_str("```\n\n");
    prompt.push_str("4. Do NOT add any explanation inside the tool_call tags\n");
    prompt.push_str("5. Do NOT include a \"source\" field\n");
    prompt.push_str("6. You can add explanations before or after the tool_call tags\n\n");
    
    prompt.push_str("Example conversation:\n");
    prompt.push_str("User: 查看 excel 文件 myfile.xlsx 的内容有什么\n");
    prompt.push_str("Assistant: 我来帮你查看这个文件的内容。\n\n");
    prompt.push_str("<tool_call>\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"id\": \"call_001\",\n");
    prompt.push_str("  \"tool_name\": \"read_data_from_excel\",\n");
    prompt.push_str("  \"arguments\": {\n");
    prompt.push_str("    \"filepath\": \"myfile.xlsx\",\n");
    prompt.push_str("    \"sheet_name\": \"Sheet1\"\n");
    prompt.push_str("  }\n");
    prompt.push_str("}\n");
    prompt.push_str("</tool_call>\n\n");
  }
  
  prompt
}
```

### 方案 2: 使用 OpenAI Function Calling API

如果使用 OpenAI 兼容的服务器,可以使用标准的 function calling API:

```rust
// 发送请求时包含 tools 参数
let request = CreateChatCompletionRequest {
    model: "google/gemma-3-27b",
    messages: vec![...],
    tools: Some(tools),  // 使用标准 OpenAI tools 格式
    tool_choice: Some(ToolChoice::Auto),
    ...
};
```

这需要修改 `middleware/chat_service_mw.rs` 中的请求构建逻辑。

### 方案 3: 简化工具描述

如果工具描述太长,尝试只包含最相关的工具:

```rust
// 只包含用户最可能需要的前 5 个工具
let mut relevant_tools = Vec::new();
for tool_name in &config.available_tools {
    if is_tool_relevant_for_query(&params.message, tool_name) {
        relevant_tools.push(tool_name);
        if relevant_tools.len() >= 5 {
            break;
        }
    }
}
```

### 方案 4: 添加提示词前缀

在用户消息前添加提示,引导 AI 使用工具:

```rust
let enhanced_message = if config.capabilities.enable_tool_calling {
    format!(
        "[System Note: You have {} tools available. Use them when appropriate.]\n\nUser: {}",
        config.available_tools.len(),
        params.message
    )
} else {
    params.message.clone()
};
```

## 🧪 测试验证

### 测试 1: 明确的工具使用请求

```
用户: "使用 read_data_from_excel 工具读取 myfile.xlsx 文件的 Sheet1 工作表"
```

预期: AI 应该生成 `<tool_call>` 标签

### 测试 2: 隐式的工具使用请求

```
用户: "查看 excel 文件 myfile.xlsx 的内容有什么"
```

预期: AI 应该理解需要使用 `read_data_from_excel` 工具

### 测试 3: 不需要工具的请求

```
用户: "你好,今天天气怎么样?"
```

预期: AI 应该直接回答,不使用工具

## 📊 日志分析清单

运行测试后,检查以下日志:

- [ ] `[DEBUG] Accumulated text length: X chars` - AI 返回的内容长度
- [ ] `[DEBUG] Current text: ...` - AI 返回的实际内容(前 200 字符)
- [ ] `[DEBUG] Final text preview: ...` - 完整响应的预览
- [ ] `[DEBUG] Final check - has <tool_call>: ?` - 是否包含工具调用标签
- [ ] `[TOOL] Complete tool call detected` - 是否检测到工具调用
- [ ] `[TOOL] Executing tool: ...` - 是否执行了工具

## 🎓 常见问题

### Q1: 为什么 AI 不使用工具?

**A**: 可能的原因:
1. 模型未针对工具调用训练
2. 系统提示不够明确
3. 工具描述太长或太复杂
4. 用户请求不够明确

### Q2: 如何验证系统提示是否正确?

**A**: 添加调试日志打印完整的系统提示,手动检查:
- 工具列表是否完整
- 工具描述是否清晰
- 协议说明是否明确
- 是否包含示例

### Q3: 哪些模型支持工具调用?

**A**: 
- ✅ GPT-4, GPT-3.5-turbo (OpenAI)
- ✅ Claude 3 (Anthropic)
- ✅ Qwen-Plus, Qwen-Turbo (阿里云)
- ✅ GLM-4 (智谱 AI)
- ❓ Gemma (Google) - 需要验证

### Q4: 如何测试特定模型是否支持工具调用?

**A**: 
1. 在系统提示中添加非常明确的指令
2. 使用示例请求测试
3. 检查日志中是否出现 `<tool_call>` 标签
4. 如果不支持,考虑切换模型或使用标准 function calling API

## 📝 下一步

1. **重新运行测试**: 使用相同的用户输入,查看新的调试日志
2. **分析 AI 响应**: 确定 AI 是否理解了工具调用协议
3. **调整系统提示**: 根据日志分析结果优化提示词
4. **测试不同模型**: 如果当前模型不支持,尝试其他模型
5. **考虑使用标准 API**: 如果自定义协议不work,使用 OpenAI function calling

请运行测试并提供新的日志,我将帮助进一步分析! 🔍

