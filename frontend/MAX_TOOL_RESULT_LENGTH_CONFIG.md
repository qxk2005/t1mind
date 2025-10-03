# 工具结果最大长度配置

## 📋 功能说明

添加了可配置的工具结果最大长度设置，用于控制多轮对话时传递给 AI 的上下文大小，避免超出模型的上下文窗口限制。

## 🔧 配置位置

### 后端 Protobuf 定义

**文件**: `rust-lib/flowy-ai/resources/proto/entities.proto`

```protobuf
message AgentCapabilitiesPB {
    bool enable_planning = 1;
    bool enable_tool_calling = 2;
    bool enable_reflection = 3;
    bool enable_memory = 4;
    int32 max_planning_steps = 5;
    int32 max_tool_calls = 6;
    int32 memory_limit = 7;
    int32 max_tool_result_length = 8;  // ⭐ 新增字段
}
```

### 后端 Rust 结构

**文件**: `rust-lib/flowy-ai/src/entities.rs`

```rust
pub struct AgentCapabilitiesPB {
  // ... 其他字段 ...
  
  /// 工具结果最大长度（字符数）
  /// 用于多轮对话时控制上下文长度，避免超出模型限制
  /// 默认 4000 字符，最小 1000 字符
  #[pb(index = 8)]
  pub max_tool_result_length: i32,
}
```

## 📊 配置参数

| 参数 | 说明 | 默认值 | 最小值 | 推荐值 |
|------|------|--------|--------|--------|
| `max_tool_result_length` | 每个工具结果的最大字符数 | 4000 | 1000 | 2000-8000 |

### 值的含义

- **0 或负数**: 使用默认值 4000
- **1-999**: 自动调整为最小值 1000
- **≥ 1000**: 使用配置的值

## 💡 使用场景

### 场景 1: 短结果工具（如计算器）
```
max_tool_result_length: 1000
```
适用于返回简短结果的工具，如数学计算、简单查询等。

### 场景 2: 标准结果（默认）
```
max_tool_result_length: 4000
```
适用于大多数工具，如文件读取、数据查询等。

### 场景 3: 长结果工具（如大型文档）
```
max_tool_result_length: 8000
```
适用于返回大量数据的工具，如搜索引擎、数据库查询等。

### 场景 4: 超长结果（需要大上下文模型）
```
max_tool_result_length: 16000
```
适用于支持超大上下文的模型（如 Claude、GPT-4 等）。

## ⚙️ 工作原理

### 多轮对话流程

```
用户提问
  ↓
AI 调用工具（第一轮）
  ↓
工具返回结果（可能很长）
  ↓
[智能截断] ← max_tool_result_length 配置在这里生效
  ↓
将截断后的结果传递给 AI（第二轮）
  ↓
AI 基于结果生成最终回答
```

### 截断逻辑

**文件**: `rust-lib/flowy-ai/src/chat.rs`

```rust
// 从智能体配置中获取工具结果最大长度限制
let max_result_length = agent_config.as_ref()
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

// 智能截断长结果（考虑 UTF-8 字符边界）
let truncated_result = if result_text.len() > max_result_length {
  // 安全截断，避免在多字节字符中间切割
  let mut truncate_len = max_result_length.min(result_text.len());
  while truncate_len > 0 && !result_text.is_char_boundary(truncate_len) {
    truncate_len -= 1;
  }
  let truncated = &result_text[..truncate_len];
  format!("{}...\n[结果已截断，原始长度: {} 字符]", truncated, result_text.len())
} else {
  result_text.clone()
};
```

## 🎯 配置建议

### 根据模型选择

| 模型 | 上下文窗口 | 推荐配置 |
|------|-----------|---------|
| GPT-3.5 | 4K tokens | 2000-3000 |
| GPT-4 | 8K tokens | 4000-6000 |
| GPT-4-32K | 32K tokens | 8000-16000 |
| Claude 2 | 100K tokens | 16000-32000 |
| 本地模型 | 2K tokens | 1000-2000 |

**注意**: 1 token ≈ 0.75 个英文单词 ≈ 0.5 个中文字符

### 根据工具类型

| 工具类型 | 典型结果大小 | 推荐配置 |
|---------|-------------|---------|
| 搜索引擎 | 10K-50K chars | 4000-8000 |
| 文件读取 | 1K-100K chars | 2000-8000 |
| 数据库查询 | 100-10K chars | 2000-4000 |
| API 调用 | 100-5K chars | 1000-3000 |
| 计算器 | 10-100 chars | 1000 |

## 📝 UI 配置界面（待实现）

### 智能体配置页面

在智能体能力配置部分添加：

```dart
// 工具结果最大长度配置
Row(
  children: [
    Expanded(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('工具结果最大长度'),
          Text(
            '控制多轮对话时传递给 AI 的工具结果大小',
            style: TextStyle(fontSize: 12, color: Colors.grey),
          ),
        ],
      ),
    ),
    SizedBox(
      width: 120,
      child: TextField(
        keyboardType: TextInputType.number,
        decoration: InputDecoration(
          hintText: '4000',
          suffixText: '字符',
        ),
        onChanged: (value) {
          // 更新配置
          final length = int.tryParse(value) ?? 4000;
          // TODO: 更新 agent_config.capabilities.max_tool_result_length
        },
      ),
    ),
  ],
)
```

## 🔍 日志追踪

当配置生效时，会输出以下日志：

```
INFO  flowy_ai::chat  🔧 [MULTI-TURN] Using max_tool_result_length: 4000 chars
```

如果结果被截断，会输出：

```
INFO  flowy_ai::chat  🔧 [MULTI-TURN] Truncating tool result from 35870 to 4000 chars
```

在工具结果中也会显示截断提示：

```
[结果内容...]
[结果已截断，原始长度: 35870 字符]
```

## ⚠️ 注意事项

### 1. 截断可能影响结果质量

如果工具结果被大幅截断，AI 可能无法获得完整信息，导致回答不准确。

**解决方案**：
- 增加 `max_tool_result_length` 值
- 优化工具，返回更精简的结果
- 使用支持更大上下文的模型

### 2. 太大的值可能超出模型限制

如果配置值过大，加上系统提示词和用户问题，可能超出模型的上下文窗口。

**解决方案**：
- 参考上面的推荐配置表
- 监控日志中的警告信息：
  ```
  WARN  flowy_ai::chat  🔧 [MULTI-TURN] ⚠️ System prompt is very long (42175 chars), may exceed model limit
  ```

### 3. UTF-8 字符边界安全

代码已经处理了 UTF-8 字符边界问题，不会在中文字符中间截断。

### 4. 多个工具调用

如果一次对话调用多个工具，每个工具的结果都会被独立截断，总上下文长度 = 系统提示词 + 多个工具结果总和。

## 🧪 测试方法

### 测试用例 1: 默认配置

```rust
// 不设置 max_tool_result_length（值为 0）
agent_config.capabilities.max_tool_result_length = 0;
```

**预期**: 使用默认值 4000

### 测试用例 2: 最小值

```rust
agent_config.capabilities.max_tool_result_length = 500;
```

**预期**: 自动调整为 1000

### 测试用例 3: 自定义值

```rust
agent_config.capabilities.max_tool_result_length = 8000;
```

**预期**: 使用 8000

### 测试用例 4: 长结果截断

1. 配置 `max_tool_result_length = 2000`
2. 调用返回 > 2000 字符的工具（如 Readwise 搜索）
3. 检查日志确认截断
4. 验证 AI 仍能基于截断结果生成回答

## 📚 相关文档

- [多轮对话实现文档](MULTI_TURN_TOOL_CALL_IMPLEMENTATION.md)
- [UTF-8 字符边界修复](UTF8_CHAR_BOUNDARY_FIX.md)
- [智能体能力配置说明](../../docs/agent_capabilities.md)

## 🚀 部署检查清单

- [x] 后端 protobuf 定义已添加
- [x] 后端 Rust 代码已实现
- [x] 截断逻辑已实现（考虑 UTF-8 边界）
- [x] 日志追踪已添加
- [x] 配置验证（最小值、默认值）已实现
- [ ] 前端 UI 配置界面（待实现）
- [ ] 用户文档更新
- [ ] 测试用例验证

---

**更新时间**: 2025-10-03  
**配置版本**: v1.0  
**兼容性**: AppFlowy AI v0.1.0+

