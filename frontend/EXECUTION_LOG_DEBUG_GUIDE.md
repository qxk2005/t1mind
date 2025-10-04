# 执行日志调试指南

## 问题描述

用户点击"查看执行过程"按钮后，显示为空（0条日志）。

## 已添加的调试日志

为了诊断问题，我在关键位置添加了详细的调试日志：

### 1. 响应开始时（`chat.rs` 第264行）

```
🔧 [RESPONSE] Starting stream_response: chat_id={}, question_id={}, has_agent={}, has_execution_logs={}
```

**检查点**：
- `has_agent`: 是否配置了智能体？
- `has_execution_logs`: 执行日志存储是否被传递？

**如果 `has_execution_logs=false`**：
- ❌ 问题：执行日志没有被传递到 `stream_response`
- 🔍 原因：可能没有启用智能体，或者在 `ai_manager.rs` 中传递逻辑有问题

### 2. 日志记录时（`chat.rs` 第278-284行）

**成功记录**：
```
📝 [LOG] Recording log: session_key={chat_id}_{question_id}, phase=ExecToolCall, step=执行工具: xxx
📝 [LOG] Total logs for session: {count}
```

**记录失败**：
```
⚠️  [LOG] Cannot record log - execution_logs is None! phase=ExecToolCall, step=执行工具: xxx
```

**检查点**：
- 如果看到 "Cannot record log"，说明 `execution_logs` 是 `None`
- 如果看到 "Recording log"，说明日志正在被记录

### 3. 查询日志时（`ai_manager.rs` 第1042-1065行）

```
📋 [QUERY] Stored execution log keys: ["{chat_id}_{question_id}", ...]
📋 [QUERY] Query session_id: {session_id}, message_id: None
📋 [QUERY] Looking for keys with prefix: {session_id}_
📋 [QUERY] Found matching key: {key}
```

**检查点**：
- `Stored execution log keys`: 当前存储了哪些日志的 key
- `Query session_id`: 查询的会话 ID 是什么
- `Found matching key`: 找到了哪些匹配的 key

## 诊断步骤

### 步骤1：检查智能体配置

1. 重新运行应用
2. 发送一条需要工具调用的消息
3. 查看日志中的第一条输出：

```
🔧 [RESPONSE] Starting stream_response: ...
```

**预期结果**：
- `has_agent=true`
- `has_execution_logs=true`

**如果不符合预期**：
- ✅ 确保在发送消息时选择了智能体
- ✅ 确保智能体配置了工具并启用了工具调用功能

### 步骤2：检查日志记录

观察是否有以下日志输出：

```
📝 [LOG] Recording log: session_key=...
📝 [LOG] Total logs for session: 1
```

**如果看到 "Cannot record log"**：
- ❌ 问题：`execution_logs` 是 `None`
- 🔧 解决：检查步骤1，确保智能体配置正确

**如果看到 "Recording log"**：
- ✅ 日志正在被记录
- 记录 `session_key` 的值，例如：`100df12c-536c-483b-9564-7aebf8fc0de1_1759505425`

### 步骤3：检查日志查询

当点击"查看执行过程"按钮时，观察以下日志：

```
📋 [QUERY] Stored execution log keys: [...]
📋 [QUERY] Query session_id: 100df12c-536c-483b-9564-7aebf8fc0de1, message_id: None
📋 [QUERY] Looking for keys with prefix: 100df12c-536c-483b-9564-7aebf8fc0de1_
```

**检查以下内容**：

1. **存储的 key 列表不为空**
   ```
   📋 [QUERY] Stored execution log keys: ["100df12c-536c-483b-9564-7aebf8fc0de1_1759505425"]
   ```
   ✅ 有日志被存储

2. **查询的 session_id 匹配存储的 key**
   - 存储的 key：`100df12c-536c-483b-9564-7aebf8fc0de1_1759505425`
   - 查询的前缀：`100df12c-536c-483b-9564-7aebf8fc0de1_`
   - ✅ 前缀匹配，应该能找到

3. **看到 "Found matching key"**
   ```
   📋 [QUERY] Found matching key: 100df12c-536c-483b-9564-7aebf8fc0de1_1759505425
   ```
   ✅ 查询成功

## 常见问题和解决方案

### 问题1：`has_execution_logs=false`

**症状**：
```
🔧 [RESPONSE] Starting stream_response: ... has_execution_logs=false
⚠️  [LOG] Cannot record log - execution_logs is None!
```

**原因**：
- 没有启用智能体
- 智能体配置为 `None`

**解决方案**：
1. 确保在发送消息时选择了智能体
2. 检查 `ai_manager.rs` 第427-431行的逻辑：
   ```rust
   let exec_logs = if agent_config.is_some() {
     Some(self.execution_logs.clone())
   } else {
     None
   };
   ```

### 问题2：日志被记录但查询不到

**症状**：
```
📝 [LOG] Recording log: session_key=xxx_123
📋 [QUERY] Stored execution log keys: ["xxx_123"]
📋 [QUERY] Query session_id: yyy, message_id: None
📋 [QUERY] Looking for keys with prefix: yyy_
✅ Successfully retrieved 0 execution logs
```

**原因**：
- 存储的 `chat_id` (xxx) 和查询的 `session_id` (yyy) 不匹配
- 可能是前端传递了错误的 `chatId`

**解决方案**：
1. 检查前端 `ExecutionLogButton` 中获取的 `chatId`：
   ```dart
   final chatId = context.read<ChatAIMessageBloc>().chatId;
   ```
2. 确保这个 `chatId` 和消息发送时的 `chat_id` 一致

### 问题3：存储的 key 列表为空

**症状**：
```
📋 [QUERY] Stored execution log keys: []
```

**原因**：
- 日志根本没有被记录
- 或者应用重启后日志丢失（内存存储）

**解决方案**：
1. 检查是否看到 "Recording log" 日志
2. 如果没有，回到步骤1检查智能体配置
3. 如果应用重启后丢失，需要实现日志持久化

### 问题4：工具没有被调用

**症状**：
```
🔧 [RESPONSE] Starting stream_response: has_agent=true, has_execution_logs=true
```
但没有看到任何 "Recording log" 输出

**原因**：
- AI 没有调用工具
- 或者工具调用检测失败

**解决方案**：
1. 检查 AI 响应中是否包含 `<tool_call>` 标签
2. 查看反思循环日志：
   ```
   🔧 [REFLECTION] Iteration 1 completed: ... new_tools: false
   ```
3. 参考 `REFLECTION_LOOP_FIX.md` 优化提示词

## 示例：完整的成功日志流程

### 发送消息时

```
🔧 [RESPONSE] Starting stream_response: chat_id=100df12c-536c-483b-9564-7aebf8fc0de1, question_id=1759505425, has_agent=true, has_execution_logs=true
🔧 [TOOL] Executing tool: get_workbook_metadata (id: call_001)
📝 [LOG] Recording log: session_key=100df12c-536c-483b-9564-7aebf8fc0de1_1759505425, phase=ExecToolCall, step=执行工具: get_workbook_metadata
📝 [LOG] Total logs for session: 1
🔧 [TOOL] Tool execution completed: call_001 - success: true
📝 [LOG] Recording log: session_key=100df12c-536c-483b-9564-7aebf8fc0de1_1759505425, phase=ExecToolCall, step=工具执行成功: get_workbook_metadata
📝 [LOG] Total logs for session: 2
```

### 查询日志时

```
📋 [QUERY] Stored execution log keys: ["100df12c-536c-483b-9564-7aebf8fc0de1_1759505425"]
📋 [QUERY] Query session_id: 100df12c-536c-483b-9564-7aebf8fc0de1, message_id: None
📋 [QUERY] Looking for keys with prefix: 100df12c-536c-483b-9564-7aebf8fc0de1_
📋 [QUERY] Found matching key: 100df12c-536c-483b-9564-7aebf8fc0de1_1759505425
✅ Successfully retrieved 2 execution logs
```

## 调试清单

使用以下清单来诊断问题：

- [ ] 1. 启动应用，确保启用智能体
- [ ] 2. 发送一条需要工具调用的消息
- [ ] 3. 检查日志：`has_agent=true`？
- [ ] 4. 检查日志：`has_execution_logs=true`？
- [ ] 5. 检查日志：看到 "Recording log"？
- [ ] 6. 记录 `session_key` 的值
- [ ] 7. 点击"查看执行过程"按钮
- [ ] 8. 检查日志：`Stored execution log keys` 不为空？
- [ ] 9. 检查日志：查询的 `session_id` 匹配存储的 key？
- [ ] 10. 检查日志：看到 "Found matching key"？

如果任何一步失败，查看相应的"常见问题和解决方案"部分。

## 后续优化

一旦找到问题根源并修复后，可以：

1. **移除或减少调试日志**
   - 保留关键的错误日志
   - 移除详细的 `info!` 日志

2. **实现日志持久化**
   - 将日志保存到数据库
   - 避免重启后丢失

3. **添加日志统计**
   - 显示总执行时间
   - 显示成功/失败率

## 修改的文件

### 1. `rust-lib/flowy-ai/src/chat.rs`

**第264-265行**：添加响应开始日志
**第274-289行**：添加日志记录详细日志

### 2. `rust-lib/flowy-ai/src/ai_manager.rs`

**第1041-1073行**：添加查询日志详细日志

## 编译状态

```bash
✅ cargo check --package flowy-ai
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.82s
```

---

**创建日期**：2025-10-03  
**创建者**：AI Assistant  
**用途**：诊断执行日志为空的问题

**使用说明**：
1. 重新编译并运行应用
2. 按照"诊断步骤"逐步检查日志输出
3. 根据日志内容定位问题根源
4. 参考"常见问题和解决方案"修复问题


