# MCP 工具发现调试指南

## 问题：日志中没有出现工具发现提示

### 可能的原因

1. **智能体工具列表不为空**：如果智能体已经有工具列表，系统不会重新发现
2. **工具调用未启用**：智能体的 `enable_tool_calling` 可能为 `false`
3. **MCP 服务器未连接**：没有可用的 MCP 服务器

### 调试步骤

#### 1. 检查智能体配置

查看日志中的这一行：
```
[Chat] Agent has X tools, tool_calling enabled: true/false
```

- 如果 `X > 0`：智能体已经有工具，不会触发发现
- 如果 `tool_calling enabled: false`：需要启用工具调用功能

#### 2. 检查 MCP 服务器状态

查看日志中的这一行：
```
[Tool Discovery] 开始扫描 X 个 MCP 服务器...
[Tool Discovery] 检查服务器: xxx (状态: Connected/Disconnected)
```

如果没有这些日志，说明：
- 智能体工具列表不为空，或
- 工具调用未启用

#### 3. 验证智能体能力配置

**重要**：智能体的能力配置（capabilities）需要正确持久化，包括：

- `enable_planning`: 启用任务规划
- `enable_tool_calling`: 启用工具调用 ⚠️ **必须为 true 才能使用工具**
- `enable_reflection`: 启用反思机制
- `enable_memory`: 启用会话记忆

### 持久化验证

#### 智能体配置持久化路径

```
存储键格式: "agent_config:agent:{agent_id}"
持久化内容:
- AgentConfigPB 完整结构
  - capabilities (AgentCapabilitiesPB)
    - enable_planning
    - enable_tool_calling  ⭐
    - enable_reflection
    - enable_memory
    - max_planning_steps
    - max_tool_calls
    - memory_limit
  - available_tools (Vec<String>)
```

#### 全局设置持久化路径

```
存储键: "agent_global_settings"
持久化内容:
- AgentGlobalSettings
  - enabled
  - default_max_planning_steps
  - default_max_tool_calls
  - default_memory_limit
  - debug_logging
  - execution_timeout
```

### 测试工具发现的三种方式

#### 方式 1: 创建新智能体（推荐）

1. 确保至少有一个 MCP 服务器已连接
2. 创建新智能体，启用"工具调用"功能
3. 不要手动添加工具列表
4. 查看日志应该看到工具发现过程

#### 方式 2: 更新现有智能体

1. 选择一个现有智能体
2. 将"工具调用"功能从关闭改为开启
3. 或者如果原本就开启但工具列表为空
4. 保存更新
5. 查看日志应该看到工具发现过程

#### 方式 3: 直接使用智能体聊天

1. 选择工具列表为空的智能体
2. 开始聊天
3. 系统会在聊天启动时自动发现工具
4. 查看日志应该看到工具发现过程

### 手动测试步骤

#### 测试 1: 检查现有智能体配置

在 Dart 侧添加日志查看智能体配置：

```dart
// 在加载智能体时
print('Agent capabilities: ${agent.capabilities}');
print('  - enable_tool_calling: ${agent.capabilities.enableToolCalling}');
print('  - enable_planning: ${agent.capabilities.enablePlanning}');
print('  - enable_reflection: ${agent.capabilities.enableReflection}');
print('  - enable_memory: ${agent.capabilities.enableMemory}');
print('Available tools: ${agent.availableTools}');
```

#### 测试 2: 创建测试智能体

```dart
final testAgent = CreateAgentRequestPB(
  name: '工具测试智能体',
  description: '用于测试MCP工具调用',
  personality: '你是一个测试助手',
  capabilities: AgentCapabilitiesPB(
    enablePlanning: true,
    enableToolCalling: true,  // ⚠️ 必须为 true
    enableReflection: true,
    enableMemory: true,
    maxPlanningSteps: 10,
    maxToolCalls: 20,
    memoryLimit: 100,
  ),
  availableTools: [], // 留空，让系统自动发现
);

await backend.createAgent(testAgent);
```

#### 测试 3: 验证工具发现

创建智能体后，检查日志应该看到：

```
[Tool Discovery] 开始扫描 X 个 MCP 服务器...
[Tool Discovery] 检查服务器: excel-server (状态: Connected)
从 MCP 服务器 'excel-server' 发现 20 个工具
为新智能体 '工具测试智能体' 自动发现了 20 个工具
```

### 问题诊断

如果仍然没有看到工具发现日志，按以下顺序排查：

#### 场景 1: 智能体已有工具列表

**现象**：日志显示 `Agent has 8 tools`

**原因**：智能体配置中已经有工具，系统不会重新发现

**解决**：
1. 删除现有智能体重新创建
2. 或手动清空智能体的 `available_tools`

#### 场景 2: 工具调用未启用

**现象**：日志显示 `tool_calling enabled: false`

**原因**：智能体的 `enable_tool_calling` 为 `false`

**解决**：
1. 在智能体设置中启用"工具调用"功能
2. 或在代码中确保创建时 `enableToolCalling: true`

#### 场景 3: 没有 MCP 服务器

**现象**：日志显示 `开始扫描 0 个 MCP 服务器`

**原因**：系统中没有配置或连接 MCP 服务器

**解决**：
1. 在设置中添加 MCP 服务器配置
2. 确保 MCP 服务器已连接（状态为 Connected）

### 如何确认配置已持久化

#### 方法 1: 重启应用测试

1. 创建/更新智能体配置
2. 完全退出应用
3. 重新启动应用
4. 查看智能体配置是否保持

#### 方法 2: 查看数据库

```sql
-- 查看智能体配置
SELECT key, value FROM kv_table 
WHERE key LIKE 'agent_config:agent:%';

-- 查看全局设置
SELECT key, value FROM kv_table 
WHERE key = 'agent_global_settings';
```

#### 方法 3: 导出配置验证

```dart
// 导出智能体配置
final exportResult = await backend.exportAgentConfig();
print('Exported config: $exportResult');
```

### 关键代码位置

#### 能力配置保存

```rust
// rust-lib/flowy-ai/src/agent/config_manager.rs

fn save_agent_config(&self, config: &AgentConfigPB) -> FlowyResult<()> {
    // 保存智能体配置（包含 capabilities）
    let key = self.agent_config_key(&config.id);
    self.store_preferences.set_object(&key, config)?;
    // ✅ capabilities 作为 AgentConfigPB 的一部分被完整序列化保存
}
```

#### 能力配置加载

```rust
pub fn get_agent_config(&self, agent_id: &str) -> Option<AgentConfigPB> {
    let key = self.agent_config_key(agent_id);
    self.store_preferences.get_object::<AgentConfigPB>(&key)
    // ✅ 完整反序列化包括 capabilities
}
```

### 总结

智能体能力配置（包括 `enable_tool_calling`、`enable_planning`、`enable_reflection`、`enable_memory`）**已经正确持久化**：

✅ **持久化机制**：
- 使用 `KVStorePreferences.set_object()` 完整序列化
- `AgentConfigPB` 包含 `AgentCapabilitiesPB`
- Serde 自动处理所有字段的序列化/反序列化

✅ **全局设置**：
- 通过 `AGENT_GLOBAL_SETTINGS_KEY` 单独持久化
- 包含默认的能力参数

✅ **数据完整性**：
- 创建时保存完整配置
- 更新时保存修改后的配置
- 加载时完整恢复配置

**如果日志中没有工具发现提示**，最可能的原因是智能体已经有工具列表，或工具调用功能未启用。请按照上述调试步骤逐一排查。

