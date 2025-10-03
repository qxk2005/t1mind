# 工具调用流式响应修复

## 问题诊断

### 发现的问题

从日志中发现了根本原因：

```
🔧 [TOOL] Accumulated text length: 87 chars
🔧 [TOOL] Number of <tool_call> tags: 1
🔧 [TOOL] Number of </tool_call> tags: 0  ← 关键问题
```

**问题**：
1. 累积文本只有 87 个字符
2. 包含 `<tool_call>` 开始标签
3. **没有** `</tool_call>` 结束标签
4. 导致解析失败，返回 0 个有效工具调用

### 原因分析

这是**流式响应的时机问题**：

#### 情况 1：响应被截断
AI 开始生成工具调用：
```
<tool_call>
{
  "id": "call_001",
  ...
```

但在生成完 `</tool_call>` 之前，某些原因导致：
- 流式响应暂停
- 数据块边界正好在标签中间
- 触发了工具调用检测逻辑

#### 情况 2：检测过早触发
原始代码的问题：
```rust
// ❌ 原始代码：只检查开始标签
if crate::agent::ToolCallHandler::contains_tool_call(&accumulated_text) {
    // 立即尝试提取，即使结束标签还没到
    let calls = extract_tool_calls(&accumulated_text);
}
```

只要检测到 `<tool_call>`，就会立即尝试提取，但此时 `</tool_call>` 可能还在后续的流式数据块中。

## 解决方案

### 修复：等待完整标签

修改检测逻辑，只有当**开始和结束标签都存在**时才尝试提取：

```rust
// ✅ 修复后的代码
let has_start_tag = accumulated_text.contains("<tool_call>");
let has_end_tag = accumulated_text.contains("</tool_call>");

if has_start_tag && has_end_tag {
    // 只有标签完整时才提取
    info!("🔧 [TOOL] Complete tool call detected in response");
    let calls = extract_tool_calls(&accumulated_text);
}
```

### 工作流程

**正常的流式响应过程**：

```
第1个数据块: "现在我需要查看文件..."
  → has_start_tag: false, has_end_tag: false
  → 不触发提取，继续累积

第2个数据块: "<tool_call>\n{\n  \"id\":"
  → has_start_tag: true, has_end_tag: false
  → 不触发提取，继续累积 ✅ 关键改进

第3个数据块: "\"call_001\",\n  \"tool_name\":"
  → has_start_tag: true, has_end_tag: false
  → 不触发提取，继续累积

第4个数据块: "\"read_data\",\n  ...\n}\n</tool_call>"
  → has_start_tag: true, has_end_tag: true ✅
  → 触发提取，成功解析工具调用

第5个数据块: "\n\n正在为您查看文件..."
  → 已处理过的文本被清除，继续正常流式输出
```

## 预期改进

### 修复前的日志

```
🔧 [TOOL] Tool call detected in response  ← 只检测到开始标签
🔧 [TOOL] Extracted 0 tool calls  ← 解析失败
🔧 [TOOL] Number of </tool_call> tags: 0  ← 结束标签还没到
```

### 修复后的日志

**情况 1：标签不完整（继续等待）**
```
(没有日志输出，静默等待下一个数据块)
```

**情况 2：标签完整（成功提取）**
```
🔧 [TOOL] Complete tool call detected in response  ← 新日志
🔧 [TOOL] Extracted 1 tool calls  ← 成功提取
🔍 [TOOL PARSE] Text contains 1 <tool_call> tags
🔍 [TOOL PARSE] Text contains 1 </tool_call> tags  ← 标签完整
✅ [TOOL PARSE] Successfully parsed tool call: read_data_from_excel
```

## 测试步骤

### 1. 重启应用

编译已完成，请重新启动 Flutter 应用。

### 2. 发送测试消息

```
查看 excel 文件 myfile.xlsx 的内容有什么
```

### 3. 观察新的日志行为

#### 预期：不再有误报

**不应该看到**：
```
❌ 🔧 [TOOL] ⚠️ Tool call tag found but extraction failed!
```

#### 预期：标签完整时才提取

**应该看到**：
```
🔧 [TOOL] Complete tool call detected in response
🔧 [TOOL] Extracted 1 tool calls from accumulated text
```

然后是完整的执行日志：
```
═══════════════════════════════════════════════════════════
🔧 [TOOL EXEC] Starting tool execution
🔧 [TOOL EXEC]   Tool: read_data_from_excel
...
🔧 [TOOL EXEC] ✅ Tool call SUCCEEDED
═══════════════════════════════════════════════════════════
```

## 其他可能的问题

### 如果仍然看到解析失败

**检查 1：AI 是否真的生成了完整的工具调用**

查看日志中的 `Accumulated text preview`，确认：
- 有完整的 `<tool_call>` 标签
- 有完整的 `</tool_call>` 标签
- JSON 格式正确

**检查 2：是否有多个工具调用**

如果 AI 生成了多个工具调用，确保它们都格式正确：
```
<tool_call>
{...}
</tool_call>

<tool_call>
{...}
</tool_call>
```

### 如果看到执行但没有结果

这是已知的架构限制（单轮对话）：
- ✅ 工具会被执行
- ✅ 结果会发送到 UI
- ❌ AI 无法看到结果并继续生成分析

**临时查看结果**：
工具结果会以 `<tool_result>` 标签的形式出现在 UI 中。

**长期解决方案**：
需要实现多轮对话机制（参见 `TOOL_CALL_CONTINUATION_ISSUE.md`）。

## 代码变更总结

### 修改文件
- `rust-lib/flowy-ai/src/chat.rs` (第 274-278 行)

### 修改内容
```rust
// 修改前
if crate::agent::ToolCallHandler::contains_tool_call(&accumulated_text) {

// 修改后
let has_start_tag = accumulated_text.contains("<tool_call>");
let has_end_tag = accumulated_text.contains("</tool_call>");

if has_start_tag && has_end_tag {
```

### 影响
- ✅ 避免在标签不完整时尝试解析
- ✅ 减少解析失败的警告日志
- ✅ 等待完整的工具调用再提取
- ✅ 提高工具调用的成功率

## 后续优化建议

1. **超时机制**：
   - 如果检测到开始标签后，长时间（如 5 秒）没有收到结束标签
   - 可能是 AI 生成异常
   - 应该记录错误并清除部分标签

2. **容错处理**：
   - 如果 JSON 格式有小错误，尝试修复
   - 例如缺少引号、逗号等

3. **多轮对话**：
   - 实现工具结果反馈机制
   - 让 AI 能够基于工具结果继续生成

## 验证清单

重新测试后，请确认：

- [ ] 不再看到 "Tool call tag found but extraction failed" 警告
- [ ] 看到 "Complete tool call detected" 信息
- [ ] 看到 "Extracted N tool calls" (N > 0)
- [ ] 看到工具执行日志（`🔧 [TOOL EXEC]`）
- [ ] 看到 MCP 工具调用日志（`🔧 [MCP TOOL]`）
- [ ] 看到工具执行成功信息
- [ ] 在 UI 中看到工具执行结果

如果所有项目都通过，说明工具调用已经成功！

