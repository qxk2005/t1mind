# 工具调用流式 JSON 解析修复

## 问题描述

在 AI 聊天中使用 MCP 工具调用时，出现 JSON 解析失败的问题，导致工具无法正常执行。

### 错误表现

```
JSON parse error: EOF while parsing an object at line 16 column 3
```

### 根本原因

这是一个**流式传输的时序问题**：

1. AI 模型以流式方式返回响应，内容分块传输
2. 系统检测到 `</tool_call>` 结束标签时立即尝试解析 JSON
3. 但由于流式传输的特性，完整的 JSON 内容可能还未完全到达
4. 导致解析时 JSON 对象不完整，缺少闭合括号 `}`

### 失败的 JSON 示例

```json
{
  "id": "call_001",
  "tool_name": "search_readwise_highlights",
  "arguments": {
    "full_text_queries": [
      {
        "field_name": "highlight_plaintext",
        "search_term": "禅宗"
      },
      {
        "field_name": "highlight_plaintext",
        "search_term": "Zen"
      }
    ],
    "vector_search_term": "禅宗书籍"
  }
  // ❌ 缺少闭合的 }
```

## 解决方案

在 `rust-lib/flowy-ai/src/agent/tool_call_handler.rs` 的 `fix_common_json_errors()` 函数中新增自动修复逻辑：

### 修复策略

1. **检测不完整 JSON**：统计所有开放和闭合的括号数量
   - 大括号：`{` vs `}`
   - 方括号：`[` vs `]`

2. **自动补全缺失括号**：
   - 如果开放括号多于闭合括号，自动在末尾添加缺失的闭合括号
   - 先补全方括号 `]`，再补全大括号 `}`

3. **日志记录**：记录检测到的问题和修复动作

### 核心代码

```rust
// 修复 2: 检查并修复不完整的 JSON（缺少闭合括号）
// 这是流式传输常见的问题：AI 返回了完整标签但 JSON 内容不完整
let trimmed = fixed.trim();

// 统计括号数量
let open_braces = trimmed.matches('{').count();
let close_braces = trimmed.matches('}').count();
let open_brackets = trimmed.matches('[').count();
let close_brackets = trimmed.matches(']').count();

// 如果缺少闭合括号，尝试补全
if open_braces > close_braces || open_brackets > close_brackets {
    warn!("🔧 [JSON FIX] Detected incomplete JSON - open_braces: {}, close_braces: {}, open_brackets: {}, close_brackets: {}", 
          open_braces, close_braces, open_brackets, close_brackets);
    
    // 补全缺少的括号
    let mut fixed_with_braces = fixed.clone();
    
    // 先补全方括号
    for _ in 0..(open_brackets - close_brackets) {
        fixed_with_braces.push_str("\n]");
    }
    
    // 再补全大括号
    for _ in 0..(open_braces - close_braces) {
        fixed_with_braces.push_str("\n}");
    }
    
    info!("🔧 [JSON FIX] Added {} closing brackets and {} closing braces", 
          open_brackets - close_brackets, open_braces - close_braces);
    
    // 使用修复后的文本继续后续处理
    fixed = fixed_with_braces;
}
```

## 测试验证

### 预期效果

当遇到流式传输导致的不完整 JSON 时：

1. **自动检测**：系统会检测到括号不匹配
2. **自动修复**：自动补全缺失的闭合括号
3. **日志输出**：
   ```
   🔧 [JSON FIX] Detected incomplete JSON - open_braces: 3, close_braces: 2, open_brackets: 1, close_brackets: 1
   🔧 [JSON FIX] Added 0 closing brackets and 1 closing braces
   ✅ [TOOL PARSE] Successfully parsed tool call: search_readwise_highlights (id: call_001)
   ```
4. **正常执行**：工具调用能够正常解析并执行

### 测试步骤

1. 编译项目：
   ```bash
   cd rust-lib/flowy-ai && cargo build
   ```

2. 启动应用并测试 MCP 工具调用

3. 观察日志输出，确认：
   - 不完整 JSON 被正确检测
   - 自动补全闭合括号
   - 工具调用成功执行

## 优势

### 1. 鲁棒性增强
- 自动处理流式传输的时序问题
- 无需修改 AI 模型或流式处理逻辑
- 向后兼容已有的正确 JSON

### 2. 用户体验改善
- 工具调用成功率提升
- 减少因格式问题导致的失败
- 用户无感知的自动修复

### 3. 可维护性
- 修复逻辑集中在一处
- 详细的日志便于问题排查
- 易于扩展其他修复规则

## 注意事项

### 适用场景
- ✅ 括号数量不匹配的不完整 JSON
- ✅ 流式传输导致的截断问题
- ✅ AI 生成的格式不规范 JSON

### 局限性
- ❌ 无法修复语义错误（如类型不匹配）
- ❌ 无法修复完全损坏的 JSON 结构
- ❌ 假设括号的嵌套顺序是正确的

### 最佳实践
1. **保持日志监控**：观察 `[JSON FIX]` 相关日志
2. **分析根因**：如果频繁触发自动修复，需要检查 AI 模型或流式处理逻辑
3. **渐进增强**：根据实际遇到的新问题，持续改进修复规则

## 相关文件

- `rust-lib/flowy-ai/src/agent/tool_call_handler.rs` - 工具调用协议和 JSON 修复逻辑
- `rust-lib/flowy-ai/src/chat.rs` - 流式响应处理和工具调用检测
- `TOOL_CALL_STREAMING_FIX.md` - 之前的流式工具调用实现文档

## 下一步

如果此修复成功解决问题，可以考虑：

1. **统计分析**：记录自动修复的触发频率和成功率
2. **优化流式处理**：在流式传输层面增加完整性检查
3. **扩展修复规则**：根据实际遇到的其他格式问题，继续扩展自动修复能力

