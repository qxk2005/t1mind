# 智能体更新时工具发现测试指南

## 问题现象

在智能体设置对话框中更新智能体选项后，没有看到工具发现的日志提示。

## 增强的调试日志

我已经添加了详细的调试日志，现在更新智能体时会输出：

```
🔄 [Agent Update] 开始更新智能体: {agent_id}
🔄 [Agent Update] 请求工具列表长度: X
🔄 [Agent Update] 请求是否包含 capabilities: true/false
🔄 [Agent Update] 现有智能体: {name}
🔄 [Agent Update] 现有工具列表长度: X
🔄 [Agent Update] 现有 enable_tool_calling: true/false
🔄 [Agent Update] 新能力配置 - enable_tool_calling: true/false
🔄 [Agent Update] 条件满足：工具调用已启用且工具列表为空
🔄 [Agent Update] 是否需要发现工具: true/false
```

## 测试步骤

### 第一步：检查当前智能体状态

打开智能体设置对话框，记录当前状态：
- 任务规划：开/关
- 工具调用：开/关  ⚠️ 关键
- 反思机制：开/关
- 会话记忆：开/关

### 第二步：执行更新操作

**测试场景 A**：如果工具调用当前是**关闭**的

1. 将"工具调用"开关打开
2. 点击"保存"
3. 查看日志，应该看到：
   ```
   🔄 [Agent Update] 新能力配置 - enable_tool_calling: true
   🔄 [Agent Update] 条件满足：工具调用已启用且工具列表为空
   🔄 [Agent Update] 是否需要发现工具: true
   ✨ [Agent Update] 检测到工具调用能力变更或工具列表为空，开始自动发现工具...
   [Tool Discovery] 开始扫描 X 个 MCP 服务器...
   ✅ [Agent Update] 为智能体 'XXX' 自动发现了 X 个工具
   ```

**测试场景 B**：如果工具调用当前是**开启**的

1. 修改其他选项（如任务规划、反思机制等）
2. 点击"保存"
3. 查看日志，应该看到：
   ```
   🔄 [Agent Update] 新能力配置 - enable_tool_calling: true
   ```
   
   **分支判断**：
   - 如果智能体已有工具 → `ℹ️ 智能体已有工具且能力未变更，跳过工具发现`
   - 如果智能体无工具 → 触发工具发现

### 第三步：检查可能的问题

#### 问题 1: 没有看到任何 `[Agent Update]` 日志

**原因**：更新请求可能没有到达 `update_agent()` 方法

**检查**：
```
# 查看是否有这行日志
INFO flowy_ai::agent::event_handler: ✅ Successfully updated agent
```

如果看到这行但没有 `[Agent Update]` 日志，说明日志级别过滤了 info 级别。

#### 问题 2: 看到 `未更新能力配置，跳过工具发现`

**原因**：更新请求中没有包含 `capabilities` 字段

**检查 Flutter UI**：
```dart
final request = UpdateAgentRequestPB()
  ..id = widget.existingAgent!.id
  ..name = name
  ..description = _descriptionController.text.trim()
  ..personality = _personalityController.text.trim()
  ..avatar = _avatarController.text.trim()
  ..capabilities = capabilities;  // ⚠️ 必须包含这一行
```

#### 问题 3: 看到 `请求中已包含 X 个工具`

**原因**：UI 代码仍然在添加工具列表

**检查**：确保 UI 代码中**没有**这些行：
```dart
..availableTools.addAll(['default_tool']);  // ❌ 应该移除
..availableTools.addAll(someList);          // ❌ 应该移除
```

应该是：
```dart
..capabilities = capabilities;  // ✅ 不设置 availableTools
```

## 预期的完整日志流程

### 从打开对话框到保存

```
# 1. 打开智能体设置对话框
DEBUG appflowy_flutter: Opening agent edit dialog for: 段子高手

# 2. 用户修改设置，点击保存
INFO  flowy_ai::agent::event_handler: 🤖 Processing update agent request for: fbe524fc-5fb4-470e-bb0b-c9c98d058860

# 3. 进入 update_agent 方法
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 开始更新智能体: fbe524fc-5fb4-470e-bb0b-c9c98d058860
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 请求工具列表长度: 0
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 请求是否包含 capabilities: true
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 现有智能体: 段子高手
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 现有工具列表长度: 0
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 现有 enable_tool_calling: false
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 新能力配置 - enable_tool_calling: true
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 条件满足：工具调用已启用且工具列表为空
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 是否需要发现工具: true

# 4. 开始工具发现
INFO  flowy_ai::ai_manager: ✨ [Agent Update] 检测到工具调用能力变更或工具列表为空，开始自动发现工具...
INFO  flowy_ai::ai_manager: [Tool Discovery] 开始扫描 1 个 MCP 服务器...
INFO  flowy_ai::ai_manager: [Tool Discovery] 检查服务器: excel-mcp (状态: Connected)
INFO  flowy_ai::ai_manager: 从 MCP 服务器 'excel-mcp' 发现 20 个工具
INFO  flowy_ai::ai_manager: 共从 1 个 MCP 服务器发现 20 个可用工具

# 5. 填充工具并保存
INFO  flowy_ai::ai_manager: ✅ [Agent Update] 为智能体 '段子高手' 自动发现了 20 个工具
INFO  flowy_ai::agent::config_manager: Agent updated successfully: 段子高手 (fbe524fc-5fb4-470e-bb0b-c9c98d058860)
INFO  flowy_ai::ai_manager: 🔄 [Agent Update] 更新完成
INFO  flowy_ai::agent::event_handler: ✅ Successfully updated agent: 段子高手 (fbe524fc-5fb4-470e-bb0b-c9c98d058860)
```

## 如果还是没有工具发现日志

请提供以下信息：

1. **完整的 `[Agent Update]` 日志**：
   ```
   从 "🔄 [Agent Update] 开始更新智能体" 
   到 "🔄 [Agent Update] 更新完成"
   ```

2. **MCP 服务器状态**：
   ```
   在设置中查看 MCP 服务器列表
   - 服务器名称
   - 连接状态（已连接/未连接）
   - 工具数量
   ```

3. **智能体当前配置**：
   - 在对话框中看到的各个开关状态
   - 是否有已存在的工具列表

## 快速解决方案

如果工具发现仍然不触发，可以尝试：

### 方案 1: 强制重新发现工具

1. 打开智能体设置
2. 将"工具调用"关闭 → 保存
3. 再次打开设置，将"工具调用"打开 → 保存
4. 这样会触发能力变更检测

### 方案 2: 删除重建

1. 删除现有智能体
2. 重新创建同名智能体
3. 确保启用"工具调用"
4. 不手动添加工具
5. 保存 → 应该会自动发现工具

### 方案 3: 使用聊天时自动发现

1. 直接使用该智能体开始聊天
2. 系统会在聊天启动时自动检查并发现工具
3. 这是最可靠的兜底机制

## 测试完成后的验证

更新完成后，再次打开智能体设置，应该看到：
- "工具调用" 开关：✓ 已启用
- 系统已自动填充工具列表（虽然在 UI 上可能看不到具体列表）

使用该智能体聊天，请求 Excel 操作：
```
查看 excel 文件 myfile.xlsx 的内容有什么
```

AI 应该能够：
1. 识别需要使用工具
2. 调用 `mcp_excel_read_data_from_excel`
3. 成功读取文件内容

