# 反思循环工具调用修复

## 问题描述

### 症状

反思循环只执行1轮就停止，虽然 AI 说它需要继续：

```
好的！
我查看了`myfile.xlsx`这个文件，发现它里面只有一个工作簿，名字叫做"总成绩"。
目前我只知道工作簿的名字，还不知道里面具体有什么内容。
为了更好地帮您，需要进一步查看"总成绩"工作簿里面的数据。
接下来我会尝试读取这个工作簿的内容，看看里面有什么信息。
```

但日志显示：
```
🔧 [REFLECTION] Iteration 1 completed: 75 messages, 75 answer chunks, has_data: true, new_tools: false
🔧 [REFLECTION] No new tool calls detected, ending reflection loop
```

### 根本原因

AI 模型在反思循环中：
1. ✅ 正确理解了需要继续调用工具
2. ✅ 知道应该调用什么工具（read_data_from_excel）
3. ❌ **但没有输出工具调用的 XML 标签格式**
4. ❌ 只是用自然语言描述了要做什么

因此，检测逻辑无法识别这是一个工具调用请求，反思循环就结束了。

## 问题分析

### 为什么初始调用成功，反思循环失败？

**初始调用**（成功）：
- 系统提示词包含详细的工具调用格式说明
- AI 正确输出 `<tool_call>` 标签

**反思循环**（失败）：
- 虽然包含原始系统提示词
- 但反思指令只说"可以继续调用其他可用工具"
- **没有再次强调必须使用 `<tool_call>` 格式**
- AI 误以为可以用自然语言描述工具调用

### 检测机制

工具调用检测代码（`chat.rs` 第794-802行）：

```rust
// 检测是否包含**完整的**工具调用
let has_start_tag = reflection_accumulated_text.contains("<tool_call>");
let has_end_tag = reflection_accumulated_text.contains("</tool_call>");

if has_start_tag && has_end_tag && !new_tool_calls_detected {
  info!("🔧 [REFLECTION] Detected new tool call in iteration {} response!", current_iteration);
  new_tool_calls_detected = true;
  // 不立即退出循环，继续接收完整的响应
}
```

**检测逻辑是正确的**，问题在于 AI 没有输出这些标签。

## 已实现的修复

### ✅ 修复：增强反思循环提示词

**文件**：`rust-lib/flowy-ai/src/chat.rs`（第704-721行）

**修改前**：
```rust
if enable_reflection && current_iteration < max_iterations {
  follow_up_context.push_str(&format!("请评估这些工具结果是否足以回答用户的问题（当前第 {}/{} 轮）：\n", current_iteration, max_iterations));
  follow_up_context.push_str("- 如果结果充分，请用中文简体总结并直接回答用户的问题\n");
  follow_up_context.push_str("- 如果结果不足或需要更多信息，可以继续调用其他可用工具\n");  // ⚠️ 不够明确
  follow_up_context.push_str("- 避免调用已经尝试过的工具或重复的查询\n");
}
```

**修改后**：
```rust
if enable_reflection && current_iteration < max_iterations {
  follow_up_context.push_str(&format!("请评估这些工具结果是否足以回答用户的问题（当前第 {}/{} 轮）：\n", current_iteration, max_iterations));
  follow_up_context.push_str("- 如果结果充分，请用中文简体总结并直接回答用户的问题\n");
  follow_up_context.push_str("- 如果结果不足或需要更多信息，你**必须**使用工具调用格式继续调用其他可用工具\n");  // ✅ 明确要求
  follow_up_context.push_str("- 避免调用已经尝试过的工具或重复的查询\n\n");
  
  // ✅ 新增：显式提供工具调用格式示例
  follow_up_context.push_str("**重要提醒**：如果你需要更多信息，不要只是描述你要做什么，而是**直接输出工具调用**：\n");
  follow_up_context.push_str("<tool_call>\n");
  follow_up_context.push_str("{\n");
  follow_up_context.push_str("  \"id\": \"unique_call_id\",\n");
  follow_up_context.push_str("  \"tool_name\": \"工具名称\",\n");
  follow_up_context.push_str("  \"arguments\": { ... }\n");
  follow_up_context.push_str("}\n");
  follow_up_context.push_str("</tool_call>\n");
}
```

### 改进要点

1. **从"可以"改为"必须"**：强制要求使用工具调用格式
2. **添加格式提醒**：在每轮反思中重新提供工具调用格式示例
3. **明确禁止描述性回答**："不要只是描述你要做什么"

## 预期效果

### 修复前的 AI 响应

```
目前我只知道工作簿的名字，还不知道里面具体有什么内容。
为了更好地帮您，需要进一步查看"总成绩"工作簿里面的数据。
接下来我会尝试读取这个工作簿的内容，看看里面有什么信息。
```
❌ 没有工具调用标签

### 修复后的预期 AI 响应

```
目前我只知道工作簿的名字，还不知道里面具体有什么内容。
我需要读取工作簿的数据来查看详细信息。

<tool_call>
{
  "id": "call_002",
  "tool_name": "read_data_from_excel",
  "arguments": {
    "filepath": "myfile.xlsx",
    "sheet_name": "总成绩",
    "start_cell": "A1"
  }
}
</tool_call>
```
✅ 包含正确的工具调用标签

## 测试建议

### 测试步骤

1. **重新编译并运行应用**
   ```bash
   cd rust-lib
   cargo build --package flowy-ai
   ```

2. **创建测试智能体**
   - 启用反思功能（`enable_reflection: true`）
   - 设置 `max_iterations: 3`
   - 配置 Excel 相关工具

3. **发送测试问题**
   ```
   myfile.xlsx 文件里有什么内容？
   ```

4. **观察执行日志**
   - 第1轮：调用 `get_workbook_metadata`
   - 应该检测到新工具调用：`new_tools: true`
   - 第2轮：调用 `read_data_from_excel`
   - 继续直到完整回答问题

### 预期日志输出

```
🔧 [REFLECTION] Iteration 1 completed: 75 messages, 75 answer chunks, has_data: true, new_tools: false
🔧 [REFLECTION] AI response preview (no tool calls detected): 目前我只知道工作簿的名字...
🔧 [REFLECTION] Total response length: 375 chars
🔧 [REFLECTION] No new tool calls detected, ending reflection loop
```

**修复后应该变为**：
```
🔧 [REFLECTION] Iteration 1 completed: 50 messages, 50 answer chunks, has_data: true, new_tools: true
🔧 [REFLECTION] Detected new tool call in iteration 1 response!
🔧 [REFLECTION] Processing new tool calls detected in iteration 1
🔧 [REFLECTION] Extracted 1 new tool calls
🔧 [REFLECTION] Executing new tool: read_data_from_excel (iteration 1)
🔧 [REFLECTION] New tools executed, continuing to iteration 2
```

## 其他相关修复

### 已修复的日志查询问题

详见 `EXECUTION_LOG_FIXES.md`，日志查询现在可以正常工作。

### 调试支持

添加了调试日志输出 AI 响应预览，帮助理解为什么没有检测到工具调用。

## 局限性和改进建议

### 当前方案的局限性

虽然提示词优化可以改善问题，但仍依赖 AI 模型的理解能力：

1. **模型能力差异**：某些模型可能仍然不遵循格式要求
2. **上下文干扰**：如果上下文太长，AI 可能忘记格式要求
3. **语言偏好**：某些模型倾向于用自然语言而非结构化格式

### 未来改进方向

#### 1. 后处理检测

如果 AI 说要调用工具但没有输出标签，自动帮它生成：

```rust
// 检测描述性的工具调用意图
if reflection_accumulated_text.contains("我需要") 
   && reflection_accumulated_text.contains("读取")
   && !new_tool_calls_detected {
  // 尝试从描述中提取工具调用意图
  warn!("🔧 [REFLECTION] AI described tool usage but didn't output <tool_call> tags");
  // TODO: 智能解析并生成工具调用
}
```

#### 2. 强制工具调用模式

配置选项 `force_tool_in_reflection: true`：
- 如果第N轮后问题未解决
- 自动分析缺失的信息
- 强制调用相关工具

#### 3. 使用 Function Calling API

对于支持的模型（OpenAI、Claude等），使用原生的 Function Calling：
```rust
// 使用模型原生的工具调用 API
let tools = convert_to_openai_function_format(available_tools);
let request = ChatCompletionRequest {
  tools: Some(tools),
  tool_choice: "auto",  // 或 "required"
  ...
};
```

#### 4. 多模型协作

- 使用强规划模型（如 GPT-4）进行任务分解
- 使用轻量级模型执行具体工具调用
- 规划模型直接输出结构化的工具调用序列

## 编译状态

```bash
✅ cargo check --package flowy-ai
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.26s
```

## 总结

### 修复内容

- ✅ 优化反思循环提示词，明确要求使用工具调用格式
- ✅ 添加格式示例提醒
- ✅ 编译成功，无错误

### 预期改进

- 🎯 AI 在反思循环中正确输出工具调用标签
- 🎯 支持多轮工具调用，直到完整回答问题
- 🎯 更好的任务规划和执行能力

### 测试要点

1. 测试多轮工具调用场景
2. 验证工具调用格式是否正确
3. 检查反思循环是否在适当时机结束

---

**修复日期**：2025-10-03  
**修复者**：AI Assistant  
**状态**：提示词优化完成 ✅  
**版本**：v2.2 - 反思循环增强版

**相关文档**：
- [执行日志修复](./EXECUTION_LOG_FIXES.md)
- [执行日志完成报告](./EXECUTION_LOG_COMPLETE.md)
- [反思功能文档](./AGENT_REFLECTION_COMPLETE.md)

**后续步骤**：
1. 重新运行应用测试反思循环
2. 观察 AI 是否正确输出工具调用标签
3. 根据实际效果决定是否需要进一步优化


