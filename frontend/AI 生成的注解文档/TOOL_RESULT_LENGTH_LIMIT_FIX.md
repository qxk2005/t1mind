# 工具结果长度限制修复

## 问题描述

用户配置的「工具结果最大长度」(max_tool_result_length) 设置似乎没有生效，MCP 工具返回的内容在 UI 中仍然显示完整的长内容。

## 问题分析

### 根本原因

通过代码审查发现，之前的实现存在**分层截断不一致**的问题：

1. **工具执行层** (`tool_call_handler.rs`): 
   - ❌ 没有应用长度限制
   - 直接返回完整的工具结果

2. **UI 显示层** (`chat.rs`):
   - ❌ 接收到完整结果后直接发送给 UI
   - 用户能看到所有内容

3. **AI 上下文层** (`chat.rs` 多轮对话部分):
   - ✅ 有长度限制（之前实现的）
   - 但用户看不到这个截断

### 数据流分析

```
MCP 工具执行
    ↓ (完整结果，未截断)
ToolCallHandler::execute_tool_call
    ↓ (完整结果，未截断)
chat.rs 接收工具结果
    ├─→ 发送给 UI (完整结果) ❌ 用户看到完整内容
    └─→ 构建 AI 上下文 (截断结果) ✅ 但用户看不到
```

## 解决方案

### 修复策略

在**工具执行返回点**就应用长度限制，确保所有下游（UI 显示和 AI 上下文）都收到统一截断后的结果。

### 优势

1. **统一处理**: 在源头截断，避免多处重复逻辑
2. **减少资源占用**: 
   - 减少内存占用
   - 减少网络传输
   - 减少日志大小
3. **用户体验**: 
   - 配置真正生效
   - 清晰的截断提示
   - 引导用户调整配置

## 实现细节

### 修改文件

`rust-lib/flowy-ai/src/agent/tool_call_handler.rs` - `execute_tool_call()` 方法

### 核心逻辑

```rust
// 🔧 应用工具结果最大长度限制（从智能体配置中获取）
let max_result_length = agent_config
    .map(|config| {
        // 确保值在合理范围内：最小 1000，默认 4000
        let configured = config.capabilities.max_tool_result_length;
        if configured <= 0 {
            4000 // 默认值
        } else if configured < 1000 {
            1000 // 最小值
        } else {
            configured as usize
        }
    })
    .unwrap_or(4000); // 如果没有配置，使用默认值 4000

// 智能截断长结果
let final_content = if content.len() > max_result_length {
    // 安全截断，考虑 UTF-8 字符边界
    let mut truncate_len = max_result_length.min(content.len());
    while truncate_len > 0 && !content.is_char_boundary(truncate_len) {
        truncate_len -= 1;
    }
    let truncated = &content[..truncate_len];
    
    warn!("🔧 [TOOL EXEC] ⚠️ Tool result truncated from {} to {} chars (max: {})", 
          content.len(), truncate_len, max_result_length);
    
    format!(
        "{}\n\n--- 结果已截断 ---\n原始长度: {} 字符\n显示长度: {} 字符\n配置限制: {} 字符\n\n💡 提示：如需查看完整结果，请在智能体配置中增加「工具结果最大长度」",
        truncated,
        content.len(),
        truncate_len,
        max_result_length
    )
} else {
    info!("🔧 [TOOL EXEC]   Result within limit (max: {} chars)", max_result_length);
    content
};
```

### 关键特性

#### 1. 配置优先级
```rust
max_tool_result_length = agent_config.max_tool_result_length
if max_tool_result_length <= 0:
    使用默认值 4000
else if max_tool_result_length < 1000:
    使用最小值 1000
else:
    使用配置值
```

#### 2. UTF-8 安全截断
```rust
let mut truncate_len = max_result_length.min(content.len());
while truncate_len > 0 && !content.is_char_boundary(truncate_len) {
    truncate_len -= 1;
}
```

#### 3. 用户友好的截断提示
```
[工具执行结果的前 N 个字符...]

--- 结果已截断 ---
原始长度: 35840 字符
显示长度: 4000 字符
配置限制: 4000 字符

💡 提示：如需查看完整结果，请在智能体配置中增加「工具结果最大长度」
```

#### 4. 详细的日志记录
```
🔧 [TOOL EXEC]   Original result size: 35840 chars
🔧 [TOOL EXEC] ⚠️ Tool result truncated from 35840 to 4000 chars (max: 4000)
```

## 测试验证

### 测试场景 1: 工具结果超过限制

**配置**: `max_tool_result_length = 4000`

**预期**:
1. ✅ UI 显示 4000 字符 + 截断提示
2. ✅ 日志显示原始长度和截断信息
3. ✅ AI 收到 4000 字符的结果

### 测试场景 2: 工具结果在限制内

**配置**: `max_tool_result_length = 4000`  
**结果**: 500 字符

**预期**:
1. ✅ UI 显示完整 500 字符
2. ✅ 日志显示 "Result within limit"
3. ✅ AI 收到完整 500 字符

### 测试场景 3: 配置不同长度

**测试矩阵**:
- 1000 字符 → 适用于小型模型 (gpt-3.5-turbo)
- 4000 字符 → 标准配置 (默认)
- 8000 字符 → 中等配置 (gpt-4)
- 16000 字符 → 大型配置 (gpt-4-turbo, claude-3)

### 测试步骤

1. **编译项目**:
   ```bash
   cd rust-lib/flowy-ai && cargo build
   ```

2. **配置智能体**:
   - 打开全局设置 → 智能体配置
   - 创建或编辑智能体
   - 启用「工具调用」
   - 设置「工具结果最大长度」为 4000

3. **测试 MCP 工具**:
   - 使用 Readwise 等返回长结果的 MCP 工具
   - 观察 UI 显示的结果长度
   - 检查是否有截断提示

4. **检查日志**:
   ```
   grep "TOOL EXEC" logs.txt
   ```

## 业界最佳实践参考

### 1. LangChain 的做法
- 使用 `max_iterations` 和 `max_execution_time` 限制工具执行
- 支持自定义 `output_parser` 来处理长输出
- 提供 `trim_intermediate_steps` 来控制上下文大小

### 2. OpenAI Function Calling
- 建议工具返回结果控制在 2000-4000 tokens 以内
- 对于长结果，建议返回摘要或关键信息
- 提供结果 ID，允许后续查询详细内容

### 3. Anthropic Claude
- Claude 3 系列支持 200K 上下文，但仍建议限制工具结果
- 推荐使用分页、摘要等技术
- 建议工具返回结构化数据而非大段文本

### 我们的实现

我们采用了**混合策略**：

1. **硬截断**: 超过限制直接截断（简单有效）
2. **清晰提示**: 告知用户截断信息和原始长度
3. **可配置**: 用户可根据 AI 模型能力调整
4. **UTF-8 安全**: 避免截断导致乱码
5. **日志完整**: 开发者可通过日志查看原始长度

## 配置建议

### 根据 AI 模型选择长度限制

| AI 模型 | 上下文窗口 | 推荐配置 | 说明 |
|---------|-----------|---------|------|
| GPT-3.5 Turbo | 16K tokens | 1000-2000 字符 | 小模型，需要精简上下文 |
| GPT-4 | 8K tokens | 2000-4000 字符 | 标准配置 |
| GPT-4 Turbo | 128K tokens | 4000-8000 字符 | 可以更宽松 |
| Claude 3 Opus | 200K tokens | 8000-16000 字符 | 大上下文模型 |
| Claude 3 Sonnet | 200K tokens | 4000-8000 字符 | 平衡性能和质量 |

### 根据工具类型选择

| 工具类型 | 推荐配置 | 说明 |
|---------|---------|------|
| 搜索工具 | 2000-4000 字符 | 摘要性内容 |
| 文档检索 | 4000-8000 字符 | 需要较完整内容 |
| 数据库查询 | 1000-2000 字符 | 结构化数据，无需太长 |
| API 调用 | 2000-4000 字符 | JSON 结果，通常适中 |
| Excel 处理 | 2000-4000 字符 | 表格数据，需要合理长度 |

## 后续优化方向

### 1. 智能摘要（高级）

当结果过长时，不是简单截断，而是：
- 提取关键信息
- 生成结构化摘要
- 保留重要部分（开头、结尾、关键段落）

```rust
// 未来可能的实现
fn intelligent_summarize(content: &str, max_length: usize) -> String {
    // 1. 提取关键信息（标题、列表、表格）
    // 2. 使用 NLP 技术识别重要段落
    // 3. 生成摘要
    // 4. 附加元数据（总长度、截断位置等）
}
```

### 2. 分页支持（高级）

对于超长结果，支持分页获取：

```rust
ToolCallResponse {
    id: "call_001",
    result: "第一页内容...",
    pagination: Some(PaginationInfo {
        current_page: 1,
        total_pages: 5,
        has_more: true,
    })
}
```

### 3. 针对工具的自定义限制（中级）

允许为不同的工具配置不同的长度限制：

```rust
tool_length_limits: HashMap<String, usize> {
    "search_readwise_highlights" => 8000,
    "excel_read_data" => 4000,
    "web_search" => 2000,
}
```

### 4. 动态调整（高级）

根据 AI 模型的剩余上下文空间动态调整限制：

```rust
let available_context = model_max_tokens - current_context_tokens;
let dynamic_limit = calculate_optimal_limit(available_context);
```

## 注意事项

### 1. 截断位置选择

当前实现在字符边界截断，未来可以考虑：
- 在句子边界截断（避免句子被切断）
- 在段落边界截断（保持语义完整）
- 在 JSON 结构边界截断（保持数据有效性）

### 2. 多内容项处理

MCP 工具可能返回多个 content 项，当前按顺序截断。未来可以考虑：
- 智能分配长度给各个 content 项
- 优先保留重要的 content 项
- 为不同类型的 content (text, image, resource) 使用不同策略

### 3. 性能影响

截断操作对性能影响很小，但需要注意：
- UTF-8 边界检测是 O(k)，k 为退后字符数（通常 < 4）
- 字符串拷贝是 O(n)，n 为截断后长度
- 对于极长结果（> 1MB），可能需要优化

## 相关文件

- `rust-lib/flowy-ai/src/agent/tool_call_handler.rs` - 工具调用处理和结果截断
- `rust-lib/flowy-ai/src/chat.rs` - AI 聊天流程和多轮对话
- `rust-lib/flowy-ai/src/entities.rs` - AgentCapabilitiesPB 定义
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/widgets/agent_dialog.dart` - UI 配置
- `MAX_TOOL_RESULT_LENGTH_CONFIG.md` - 配置功能的实现文档

## 总结

这次修复解决了「工具结果最大长度」配置不生效的问题，通过在工具执行返回点统一应用长度限制，确保了：

1. ✅ **配置真正生效** - UI 和 AI 都收到截断后的结果
2. ✅ **用户体验良好** - 清晰的截断提示和配置引导
3. ✅ **性能优化** - 在源头截断，减少资源占用
4. ✅ **安全可靠** - UTF-8 安全处理，避免乱码
5. ✅ **可观测性** - 详细的日志记录便于调试

用户现在可以根据使用的 AI 模型和工具类型，灵活配置工具结果的最大长度，有效控制上下文大小，提升 AI 对话的质量和稳定性。

