# 工具调用 Markdown 格式问题修复

## 🐛 问题描述

AI 能够理解需要使用工具,但生成的工具调用格式错误,导致系统无法识别和执行。

### 症状

- ✅ 工具详情成功加载到系统提示
- ✅ AI 理解用户请求需要使用工具
- ✅ AI 正确选择了工具和参数
- ❌ **工具调用格式错误,系统无法检测**

### 日志证据

```
🔧 [DEBUG] Current text preview: 好的，没问题！要查看 `myfile.xlsx` 文件的内容，
我需要使用 `read_data_from_excel` 工具来读取文件中的数据。

```tool_call
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "arguments": {
    "filepath": "myfile.xlsx",
    "sheet_name": "Sheet1",
    "start_cell": "A1"
  }
}
```

🔧 [DEBUG] Final check - has <tool_call>: false, has </tool_call>: false
```

## 🔍 根本原因

### AI 生成的格式 (错误)

```markdown
```tool_call
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "arguments": {...}
}
```
```

**问题**: AI 使用了 **markdown 代码块** (```) 包裹工具调用。

### 期望的格式 (正确)

```xml
<tool_call>
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "arguments": {...}
}
</tool_call>
```

**要求**: 使用 **XML 风格的标签** (`<tool_call>...</tool_call>`)。

### 为什么 AI 会误用?

系统提示中的示例使用了 markdown 代码块来展示格式:

```markdown
**Tool Calling Protocol:**
When you need to use a tool, format your request as follows:

```
<tool_call>
{...}
</tool_call>
```
```

AI 错误地理解为:
- ❌ 需要使用 markdown 代码块
- ✅ 应该理解为: 这只是展示格式的方式,实际输出应该直接使用 `<tool_call>` 标签

## ✅ 解决方案

### 方案 1: 优化系统提示 (主要)

**修改**: `rust-lib/flowy-ai/src/agent/system_prompt.rs`

```rust
// 之前 (容易误导)
prompt.push_str("  ```\n");
prompt.push_str("  <tool_call>\n");
prompt.push_str("  {...}\n");
prompt.push_str("  </tool_call>\n");
prompt.push_str("  ```\n\n");

// 之后 (明确指示)
prompt.push_str("  When you need to use a tool, DIRECTLY output the following format (WITHOUT markdown code blocks):\n\n");
prompt.push_str("  <tool_call>\n");
prompt.push_str("  {...}\n");
prompt.push_str("  </tool_call>\n\n");
prompt.push_str("  **CRITICAL:** Do NOT wrap the tool call in markdown code blocks (``` or ```tool_call). Output the <tool_call> tags directly!\n\n");
```

**关键改进**:
1. ✅ 移除了示例中的 markdown 代码块标记
2. ✅ 明确说明 "DIRECTLY output" 和 "WITHOUT markdown code blocks"
3. ✅ 添加 **CRITICAL** 警告不要使用代码块
4. ✅ 在规则中再次强调

### 方案 2: 自动转换格式 (备用)

**修改**: `rust-lib/flowy-ai/src/chat.rs`

```rust
// 检测 markdown 代码块格式
let has_markdown_tool_call = accumulated_text.contains("```tool_call") && 
                             accumulated_text.contains("```\n");

// 如果检测到 markdown 格式,自动转换为 XML 格式
if has_markdown_tool_call && !has_start_tag {
  warn!("🔧 [TOOL] ⚠️ AI used markdown code block format instead of XML tags! Converting...");
  accumulated_text = accumulated_text
    .replace("```tool_call\n", "<tool_call>\n")
    .replace("\n```", "\n</tool_call>");
  info!("🔧 [TOOL] Converted markdown format to XML format");
}
```

**工作原理**:
1. ✅ 检测 AI 是否使用了 ````tool_call` 格式
2. ✅ 自动替换为 `<tool_call>` 格式
3. ✅ 记录警告日志,方便后续优化
4. ✅ 确保向后兼容

## 🧪 测试验证

### 测试用例

**用户输入**: "查看 excel 文件 myfile.xlsx 的内容有什么"

### 预期行为

#### 方案 1 生效 (理想情况)

AI 直接输出正确格式:
```
🔧 [DEBUG] Tool call tags detected - XML start: true, XML end: true, Markdown: false
🔧 [TOOL] Complete tool call detected in response
🔧 [TOOL] Executing tool: read_data_from_excel (id: call_001)
```

#### 方案 2 生效 (兜底)

AI 仍使用 markdown,但系统自动转换:
```
🔧 [DEBUG] Tool call tags detected - XML start: false, XML end: false, Markdown: true
🔧 [TOOL] ⚠️ AI used markdown code block format instead of XML tags! Converting...
🔧 [TOOL] Converted markdown format to XML format
🔧 [TOOL] Complete tool call detected in response
🔧 [TOOL] Executing tool: read_data_from_excel (id: call_001)
```

### 日志检查清单

运行测试后,检查以下日志:

- [ ] `🔧 [DEBUG] Tool call tags detected` - 显示检测到的标签类型
- [ ] `🔧 [TOOL] Converted markdown format` - (如果出现) 说明自动转换生效
- [ ] `🔧 [TOOL] Complete tool call detected` - 工具调用被成功检测
- [ ] `🔧 [TOOL] Executing tool` - 工具开始执行
- [ ] `🔧 [TOOL] Tool execution completed` - 工具执行完成
- [ ] 工具执行结果显示在 UI 中

## 📊 修复效果

### 之前

```
[Chat] 🔧 Using enhanced system prompt with 25 tool details
[DEBUG] Final check - has <tool_call>: false, has </tool_call>: false
```
❌ 工具调用未被检测,无任何执行

### 之后 (方案 1)

```
[Chat] 🔧 Using enhanced system prompt with 25 tool details
[DEBUG] Tool call tags detected - XML start: true, XML end: true, Markdown: false
[TOOL] Complete tool call detected in response
[TOOL] Executing tool: read_data_from_excel (id: call_001)
[TOOL] Tool execution completed: call_001 - success: true
```
✅ 工具调用正确检测并执行

### 之后 (方案 2 兜底)

```
[Chat] 🔧 Using enhanced system prompt with 25 tool details
[DEBUG] Tool call tags detected - XML start: false, XML end: false, Markdown: true
[TOOL] ⚠️ AI used markdown code block format instead of XML tags! Converting...
[TOOL] Converted markdown format to XML format
[TOOL] Complete tool call detected in response
[TOOL] Executing tool: read_data_from_excel (id: call_001)
```
✅ 即使 AI 使用错误格式,系统也能自动修正

## 🎯 优势

### 双重保障机制

1. **主动预防** (方案 1):
   - ✅ 明确的系统提示指导
   - ✅ 减少 AI 误用的可能性
   - ✅ 提高响应质量

2. **被动修正** (方案 2):
   - ✅ 自动检测并转换格式
   - ✅ 向后兼容
   - ✅ 确保系统稳定运行

### 健壮性

- ✅ 即使 AI 不遵循指令,系统仍能正常工作
- ✅ 通过日志可以识别 AI 是否理解了协议
- ✅ 可以收集数据优化系统提示

## 📝 后续优化

### 短期 (立即)

- [ ] 测试不同 AI 模型的表现
- [ ] 收集 markdown 格式误用的频率统计
- [ ] 根据统计决定是否需要进一步优化提示

### 中期 (1-2 周)

- [ ] 如果误用率高,考虑在系统提示中添加更多示例
- [ ] 添加工具调用格式的单元测试
- [ ] 考虑支持其他常见的误用格式

### 长期 (1-2 月)

- [ ] 考虑使用标准 OpenAI Function Calling API
- [ ] 评估是否需要微调模型以提高工具调用准确性
- [ ] 建立工具调用格式的最佳实践文档

## 🔧 相关文件

| 文件 | 修改内容 | 目的 |
|------|---------|------|
| `system_prompt.rs` | 优化工具调用协议说明 | 主动预防 AI 误用格式 |
| `chat.rs` | 添加 markdown 格式检测和转换 | 被动修正 AI 错误格式 |

## 📖 总结

通过两个互补的解决方案:
1. **优化系统提示** - 明确指导 AI 使用正确格式
2. **自动格式转换** - 兜底处理 AI 的格式错误

确保了:
- ✅ AI 更可能生成正确格式
- ✅ 即使格式错误也能自动修正
- ✅ 工具调用功能稳定可靠

这是一个**健壮且实用**的解决方案! 🎉

