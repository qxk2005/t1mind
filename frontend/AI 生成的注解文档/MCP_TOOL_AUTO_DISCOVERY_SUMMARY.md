# MCP 工具自动发现功能总结

## 功能概述

智能体现在支持从已配置的 MCP 服务器**自动发现**可用工具，无需手动配置工具列表。

## 自动发现的三个触发时机

### 1. ✅ 创建智能体时

**条件**：
- `available_tools` 列表为空
- `enable_tool_calling` 设置为 `true`

**行为**：
```
→ 扫描所有已连接的 MCP 服务器
→ 收集所有可用工具
→ 自动填充到智能体配置
→ 保存配置
```

**日志输出**：
```
[Tool Discovery] 开始扫描 1 个 MCP 服务器...
[Tool Discovery] 检查服务器: excel-server (状态: Connected)
从 MCP 服务器 'excel-server' 发现 20 个工具
为新智能体 'XXX' 自动发现了 20 个工具
```

### 2. ✅ 更新智能体时（新增功能）

**条件**：
- 更新了 `capabilities` 配置
- `enable_tool_calling` 为 `true`
- `available_tools` 为空
- 满足以下之一：
  - 现有配置的工具列表也为空
  - `enable_tool_calling` 从 `false` 改为 `true`

**行为**：
```
→ 检测到工具调用能力变更
→ 自动发现 MCP 工具
→ 填充工具列表
→ 保存更新
```

**日志输出**：
```
[Agent Update] 检测到工具调用能力变更，开始自动发现工具...
[Tool Discovery] 开始扫描 1 个 MCP 服务器...
[Agent Update] 为智能体 'XXX' 自动发现了 20 个工具
```

**典型使用场景**：
- 用户创建智能体时忘记启用工具调用，后来在设置中打开
- 用户想刷新智能体的工具列表（可以先清空工具然后更新）

### 3. ✅ 聊天启动时（运行时兜底）

**条件**：
- 加载智能体配置
- `available_tools` 为空
- `enable_tool_calling` 为 `true`

**行为**：
```
→ 在开始聊天前检查工具列表
→ 如果为空则自动发现
→ 保存更新的配置
→ 使用更新后的配置继续聊天
```

**日志输出**：
```
[Chat] Using agent: XXX (xxx-xxx-xxx)
[Chat] Agent has 0 tools, tool_calling enabled: true
[Chat] 智能体工具列表为空，开始自动发现 MCP 工具...
[Tool Discovery] 开始扫描 1 个 MCP 服务器...
为智能体 'XXX' 自动发现并填充了 20 个工具
```

## 工具发现逻辑

```rust
async fn discover_available_tools(&self) -> Vec<String> {
    // 1. 获取所有 MCP 服务器
    let servers = self.mcp_manager.list_servers().await;
    
    // 2. 遍历已连接的服务器
    for server in servers {
        if server.status == Connected {
            // 3. 获取服务器的工具列表
            let tools_list = self.mcp_manager.tool_list(&server.server_id).await;
            
            // 4. 收集所有工具名称
            for tool in tools_list.tools {
                tool_names.push(tool.name);
            }
        }
    }
    
    // 5. 返回所有发现的工具
    tool_names
}
```

## 快速测试

### 测试场景 1: 创建新智能体

```
1. 确保 Excel MCP 服务器已连接
2. 创建新智能体：
   - 名称：工具测试
   - 启用"工具调用"：✓
   - 不手动添加工具
3. 查看日志 → 应该看到工具发现过程
4. 使用该智能体聊天，请求"读取 excel 文件"
5. 观察是否能调用 MCP Excel 工具
```

### 测试场景 2: 更新现有智能体

```
1. 选择现有智能体（如"段子高手"）
2. 打开智能体设置
3. 将"工具调用"开关从关闭改为开启
4. 保存更新
5. 查看日志 → 应该看到：
   [Agent Update] 检测到工具调用能力变更，开始自动发现工具...
6. 使用该智能体测试工具调用
```

### 测试场景 3: 聊天时自动发现

```
1. 使用工具列表为空的智能体
2. 直接开始聊天
3. 查看日志 → 应该看到：
   [Chat] 智能体工具列表为空，开始自动发现 MCP 工具...
4. 工具会在聊天开始前自动填充
```

## 预期日志输出

### 完整的工具发现流程

```
INFO  flowy_ai::ai_manager: [Chat] Using agent: 段子高手 (fbe524fc-5fb4-470e-bb0b-c9c98d058860)
INFO  flowy_ai::ai_manager: [Chat] Agent has 0 tools, tool_calling enabled: true
INFO  flowy_ai::ai_manager: [Chat] 智能体工具列表为空，开始自动发现 MCP 工具...
INFO  flowy_ai::ai_manager: [Tool Discovery] 开始扫描 1 个 MCP 服务器...
INFO  flowy_ai::ai_manager: [Tool Discovery] 检查服务器: excel-mcp (状态: Connected)
INFO  flowy_ai::ai_manager: 从 MCP 服务器 'excel-mcp' 发现 20 个工具
INFO  flowy_ai::ai_manager: 共从 1 个 MCP 服务器发现 20 个可用工具
INFO  flowy_ai::ai_manager: 为智能体 段子高手 自动发现并填充了 20 个工具
```

## 故障排查

### 如果没有看到工具发现日志

检查以下条件：

1. **智能体已有工具** → 日志显示 `Agent has X tools`（X > 0）
   - 解决：删除智能体重新创建，或清空工具列表后更新

2. **工具调用未启用** → 日志显示 `tool_calling enabled: false`
   - 解决：在智能体设置中启用"工具调用"开关

3. **没有 MCP 服务器** → 日志显示 `开始扫描 0 个 MCP 服务器`
   - 解决：在 MCP 设置中添加并连接服务器

4. **MCP 服务器未连接** → 日志显示 `状态: Disconnected`
   - 解决：在 MCP 设置中连接服务器

## 持久化验证

### 智能体配置完全持久化 ✅

所有配置（包括能力开关）都会持久化到 SQLite：

```
存储结构:
├── agent_config:agent:{agent_id}
│   ├── id, name, description, avatar, personality
│   ├── capabilities
│   │   ├── enable_planning ✓
│   │   ├── enable_tool_calling ✓
│   │   ├── enable_reflection ✓
│   │   ├── enable_memory ✓
│   │   ├── max_planning_steps
│   │   ├── max_tool_calls
│   │   └── memory_limit
│   ├── available_tools (动态发现并保存)
│   └── status, created_at, updated_at, metadata
│
└── agent_global_settings
    ├── enabled
    ├── default_max_planning_steps
    ├── default_max_tool_calls
    └── default_memory_limit
```

### 重启后配置保持 ✅

重启应用后：
1. 智能体配置从 SQLite 加载
2. 所有能力开关保持原有状态
3. 工具列表保持（如果之前已发现）

## 修复的文件清单

### 后端 (Rust)
- ✅ `rust-lib/flowy-ai/src/ai_manager.rs`
  - 添加 `discover_available_tools()` 方法
  - 在 `create_agent()` 中集成工具发现
  - 在 `update_agent()` 中集成工具发现
  - 在 `stream_chat_message()` 中集成工具发现

- ✅ `rust-lib/flowy-ai/src/agent/config_manager.rs`
  - 修改验证逻辑，允许空工具列表（如果将自动发现）
  - 添加 `auto_populate_agent_tools()` 辅助方法

### 前端 (Flutter)
- ✅ `appflowy_flutter/lib/workspace/presentation/settings/workspace/widgets/agent_dialog.dart`
  - 移除硬编码的 `['default_tool']`
  - 让后端自动发现工具

## 总结

现在智能体的工具发现是**完全自动化**的：

✅ 创建智能体 → 自动发现工具  
✅ 更新智能体能力 → 自动发现工具  
✅ 启动聊天 → 运行时兜底发现工具  
✅ 配置持久化 → 能力开关和工具列表都正确保存  
✅ 配置驱动 → 完全基于实际的 MCP 服务器配置

用户体验：**零配置，开箱即用！** 🎉

