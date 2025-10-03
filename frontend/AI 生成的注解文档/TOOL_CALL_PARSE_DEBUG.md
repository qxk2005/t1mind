# 工具调用解析失败调试指南

## 问题现象

从日志可以看到：

1. ✅ **工具调用被检测到**：
   ```
   🔧 [TOOL] Tool call detected in response
   ```

2. ❌ **但没有执行日志**：
   - 没有看到 `🔧 [TOOL] Executing tool`
   - 没有看到 `🔧 [TOOL] Tool execution completed`

这说明 `<tool_call>` 标签被检测到了，但**解析失败**，导致 `extract_tool_calls()` 返回空列表。

## 已添加的调试日志

我已经添加了详细的工具调用解析日志，现在重新测试时会看到：

### 成功解析的日志

```
🔧 [TOOL] Tool call detected in response
🔧 [TOOL] Extracted 1 tool calls from accumulated text
🔍 [TOOL PARSE] Found <tool_call> tag at position 123
🔍 [TOOL PARSE] Found </tool_call> tag, JSON content length: 156
🔍 [TOOL PARSE] JSON content: {"id":"call_001","tool_name":"read_data_from_excel",...}
✅ [TOOL PARSE] Successfully parsed tool call: read_data_from_excel (id: call_001)
🔍 [TOOL PARSE] Extraction complete: 1 valid tool calls found
🔧 [TOOL] Executing tool: read_data_from_excel (id: call_001)
```

### 解析失败的日志

**情况 1：JSON 格式错误**
```
🔧 [TOOL] Tool call detected in response
🔧 [TOOL] Extracted 0 tool calls from accumulated text
🔧 [TOOL] ⚠️ Tool call tag found but extraction failed!
🔧 [TOOL] Accumulated text preview (first 500 chars):
🔧 [TOOL] ...现在需要查看文件...<tool_call>
  "tool_name": "read_data_from_excel",
  "arguments": {...
  
🔍 [TOOL PARSE] Found <tool_call> tag at position 45
🔍 [TOOL PARSE] Found </tool_call> tag, JSON content length: 89
🔍 [TOOL PARSE] JSON content: "tool_name": "read_data_from_excel",...
❌ [TOOL PARSE] Failed to parse tool call JSON: expected value at line 1 column 1
❌ [TOOL PARSE] Invalid JSON (first 200 chars): "tool_name": "read_data_from_excel",...
```

**原因**：JSON 缺少开头的 `{`

**情况 2：标签不完整**
```
🔧 [TOOL] Tool call detected in response
🔧 [TOOL] Extracted 0 tool calls from accumulated text
🔍 [TOOL PARSE] Found <tool_call> tag at position 45
❌ [TOOL PARSE] Found <tool_call> but no matching </tool_call> tag
🔍 [TOOL PARSE] Extraction complete: 0 valid tool calls found
```

**原因**：AI 还在生成，`</tool_call>` 还没有到达

## 测试步骤

### 1. 重新编译并启动应用

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
cargo build --manifest-path rust-lib/Cargo.toml
# 然后重启 Flutter 应用
```

### 2. 发送测试消息

```
查看 excel 文件 myfile.xlsx 的内容有什么
```

### 3. 查看详细日志

应该会看到类似这样的日志序列：

```
[Chat] Using agent: 段子高手
[Chat] Agent has 25 tools, tool_calling enabled: true
[Chat] Tool usage recommended for this request
🔧 [TOOL] Tool call detected in response
🔧 [TOOL] Extracted N tool calls from accumulated text
```

**关键检查点**：
- `Extracted N tool calls` 中的 N 是多少？
- 如果 N = 0，查看 `⚠️ Tool call tag found but extraction failed!` 下面的内容
- 如果 N > 0，应该会看到后续的执行日志

### 4. 分析失败原因

#### 原因 A：JSON 格式问题

如果看到：
```
❌ [TOOL PARSE] Failed to parse tool call JSON: expected value at line 1 column 1
❌ [TOOL PARSE] Invalid JSON (first 200 chars): "tool_name": ...
```

**问题**：AI 生成的 JSON 格式不正确，缺少 `{` 或其他必需的语法元素。

**解决方案**：
1. 检查系统提示词中的工具调用示例是否正确
2. 确认 AI 模型是否支持工具调用（某些模型可能需要特殊训练）
3. 尝试更明确的提示词格式

#### 原因 B：流式响应不完整

如果看到：
```
❌ [TOOL PARSE] Found <tool_call> but no matching </tool_call> tag
```

**问题**：在检测时，AI 还在生成，结束标签还没到达。

**当前代码的行为**：
- 每次收到新的流式数据时，都会累积文本并检测工具调用
- 如果检测到 `<tool_call>` 但没有 `</tool_call>`，会继续等待
- 下一个数据块到达时，会再次检测

**这应该不是问题**，因为代码会持续累积直到完整标签到达。

#### 原因 C：多个不完整的工具调用

从您之前提供的日志：
```
<tool_call>
  "tool_name": "get_workbook_metadata  "arguments": {
</tool_call>
<tool_call{
  "tool_name": "read_data_from_excel",
</tool_call>
```

**问题**：
1. 第一个调用的 JSON 不完整：`"get_workbook_metadata  "arguments": {` （缺少 `}`）
2. 第二个调用格式错误：`<tool_call{` （应该是 `<tool_call>`，然后是 `{`）

**这说明 AI 模型生成的格式有问题。**

## 潜在的根本原因

### 1. 系统提示词格式不清晰

让我检查系统提示词中的工具调用指南：

<function_calls>
<invoke name="codebase_search">
<parameter name="query">What does the system prompt tell the AI about tool call format and XML tags?
