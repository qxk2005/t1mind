# AI 聊天智能体 MCP 工具调用修复

## 问题描述

在 AI 聊天中选择智能体后，智能体无法调用 MCP 工具，AI 响应显示"无法直接访问文件或外部数据"。

## 根本原因分析

从日志分析发现，问题的根源是：

1. **智能体工具列表为空**：智能体配置中的 `available_tools` 字段为空
2. **系统提示词缺失工具信息**：在 `system_prompt.rs` 第42行，只有当 `!config.available_tools.is_empty()` 时才会添加工具调用协议到系统提示词
3. **AI 模型未收到工具信息**：因为系统提示词中没有工具调用协议，AI 模型不知道有哪些工具可用

## 修复方案

### 1. 自动填充默认工具列表

**文件：`rust-lib/flowy-ai/src/agent/config_manager.rs`**

- 在创建智能体时，如果 `available_tools` 为空且启用了工具调用，自动填充默认工具列表
- 添加 `get_default_tools()` 方法，提供基础的 MCP Excel 工具和原生工具
- 修改验证逻辑，只在启用工具调用时才要求工具列表非空

**文件：`rust-lib/flowy-ai/src/ai_manager.rs`**

- 在加载智能体配置时，检查并自动填充空的工具列表
- 确保现有智能体也能获得默认工具

**文件：`rust-lib/flowy-ai/src/agent/config_manager.rs`**

- 添加 `auto_populate_agent_tools()` 方法，为现有智能体自动填充工具

### 2. 动态工具发现机制

修复后，智能体将**动态**从已配置的 MCP 服务器获取所有可用工具：

- **自动发现**：系统会遍历所有已连接的 MCP 服务器
- **实时同步**：获取每个服务器上的所有可用工具
- **无需手动配置**：工具列表完全基于系统中实际配置的 MCP 服务器
- **灵活扩展**：添加新的 MCP 服务器后，工具会自动可用

例如，如果配置了 Excel MCP 服务器，智能体将自动获得：
- `mcp_excel_read_data_from_excel`
- `mcp_excel_write_data_to_excel`
- `mcp_excel_create_workbook`
- 以及该服务器提供的所有其他工具

## 修复的关键代码变更

### 1. 智能体创建时动态发现工具

```rust
pub async fn create_agent(&self, mut request: CreateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
    // 如果工具列表为空且启用了工具调用，动态发现工具
    if request.available_tools.is_empty() && request.capabilities.enable_tool_calling {
        let discovered_tools = self.discover_available_tools().await;
        
        if !discovered_tools.is_empty() {
            info!("为新智能体自动发现了 {} 个工具", discovered_tools.len());
            request.available_tools = discovered_tools;
        }
    }
    
    self.agent_manager.create_agent(request)
}
```

### 2. 现有智能体运行时动态发现工具

```rust
// 在聊天启动时自动填充工具列表（如果为空）
if config.available_tools.is_empty() && config.capabilities.enable_tool_calling {
    // 从 MCP 服务器动态发现可用工具
    let discovered_tools = self.discover_available_tools().await;
    
    if !discovered_tools.is_empty() {
        config.available_tools = discovered_tools;
        // 保存更新的配置
    }
}
```

**修复位置**：`stream_chat_message()` 方法中加载智能体配置时

### 2.1 更新智能体时也支持动态发现

```rust
pub async fn update_agent(&self, mut request: UpdateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
    // 如果更新了能力配置，且启用了工具调用，但工具列表为空，则自动发现工具
    if let Some(ref capabilities) = request.capabilities {
        if capabilities.enable_tool_calling && request.available_tools.is_empty() {
            // 获取现有配置
            if let Some(existing_config) = self.agent_manager.get_agent_config(&request.id) {
                // 如果现有配置也没有工具，或者工具调用能力发生了变化
                if existing_config.available_tools.is_empty() || 
                   capabilities.enable_tool_calling != existing_config.capabilities.enable_tool_calling {
                    // 自动发现工具
                    let discovered_tools = self.discover_available_tools().await;
                    if !discovered_tools.is_empty() {
                        request.available_tools = discovered_tools;
                    }
                }
            }
        }
    }
    
    self.agent_manager.update_agent(request)
}
```

**触发场景**：
- 用户将 `enable_tool_calling` 从 `false` 改为 `true`
- 现有智能体的工具列表为空
- 系统自动发现并填充工具

### 3. 动态工具发现实现

```rust
async fn discover_available_tools(&self) -> Vec<String> {
    let mut tool_names = Vec::new();
    
    // 获取所有已连接的 MCP 服务器
    let servers = self.mcp_manager.list_servers().await;
    
    for server in servers {
        // 只从已连接的服务器获取工具
        if server.status == MCPConnectionStatus::Connected {
            match self.mcp_manager.tool_list(&server.server_id).await {
                Ok(tools_list) => {
                    for tool in tools_list.tools {
                        tool_names.push(tool.name);
                    }
                }
                Err(e) => {
                    warn!("获取工具列表失败: {}", e);
                }
            }
        }
    }
    
    tool_names
}
```

### 4. 修改验证逻辑

```rust
// 之前：要求所有智能体都有工具
if config.available_tools.is_empty() {
    errors.push("至少需要选择一个可用工具".to_string());
}

// 修复后：只在启用工具调用时才要求工具
if config.available_tools.is_empty() && config.capabilities.enable_tool_calling {
    errors.push("启用工具调用时至少需要选择一个可用工具".to_string());
}
```

## 修复效果

修复后，智能体将能够：

1. **动态发现 MCP 工具**：从所有已配置且已连接的 MCP 服务器自动发现工具
2. **灵活适应配置变化**：
   - 添加新 MCP 服务器时，其工具会自动对智能体可用
   - 无需手动维护工具列表
   - 符合"配置驱动"的设计原则
3. **正确的系统提示词**：系统提示词会包含所有发现的工具和工具调用协议
4. **成功调用 MCP 工具**：AI 能够识别用户请求并调用适当的 MCP 工具

## 工具发现的三个触发时机

1. **创建智能体时** → `create_agent()`
   - 工具列表为空
   - `enable_tool_calling` 为 `true`
   - → 自动发现并填充工具

2. **更新智能体时** → `update_agent()`
   - 工具列表为空
   - `enable_tool_calling` 从 `false` 改为 `true`（或原本就为空）
   - → 自动发现并填充工具

3. **聊天启动时** → `stream_chat_message()`
   - 加载智能体配置
   - 工具列表为空且 `enable_tool_calling` 为 `true`
   - → 自动发现并填充工具（运行时兜底）

## 关键设计优势

采用动态工具发现机制的优势：

1. **配置驱动**：工具列表完全由系统中配置的 MCP 服务器决定
2. **零手动维护**：无需手动维护固定的工具列表
3. **自动扩展**：添加新 MCP 服务器后，工具自动对所有智能体可用
4. **实时同步**：在创建、更新、使用智能体时都会获取最新的工具列表
5. **灵活适应**：符合系统的整体架构设计原则
6. **智能触发**：只在需要时才发现工具，避免不必要的开销

## 测试验证

用户可以通过以下方式验证修复：

1. **确保 MCP 服务器已连接**：在设置中检查 MCP 服务器状态
2. **创建或使用智能体**：
   - 创建新智能体时，系统会自动发现并填充工具
   - 使用现有智能体时，系统会自动更新空的工具列表
3. **测试工具调用**：发送需要工具的请求，如"查看 excel 文件 myfile.xlsx 的内容有什么"
4. **观察日志**：查看日志中的工具发现信息：
   ```
   从 MCP 服务器 'xxx' 发现 X 个工具
   为智能体 'xxx' 自动发现并填充了 X 个工具
   ```

## 前端 UI 修复

**问题**：Flutter UI 在创建/更新智能体时强制添加 `['default_tool']`，导致工具列表不为空，阻止了自动工具发现。

**修复**：移除硬编码的工具列表，让后端自动从 MCP 服务器发现。

**文件**：`appflowy_flutter/lib/workspace/presentation/settings/workspace/widgets/agent_dialog.dart`

```dart
// 修复前：
..availableTools.addAll(['default_tool']);  // ❌ 硬编码工具

// 修复后：
..capabilities = capabilities;  // ✅ 不设置 availableTools，让后端自动发现
```

## 持久化确认

### 智能体能力配置持久化 ✅

智能体的能力配置（`AgentCapabilitiesPB`）已经正确持久化：

- **保存位置**：`agent_config:agent:{agent_id}` 键
- **包含字段**：
  - `enable_planning` - 任务规划
  - `enable_tool_calling` - 工具调用 ⚠️ 必须为 true
  - `enable_reflection` - 反思机制
  - `enable_memory` - 会话记忆
  - `max_planning_steps`, `max_tool_calls`, `memory_limit`

- **持久化机制**：
  - 创建智能体时：完整保存所有能力配置
  - 更新智能体时：保存修改后的能力配置
  - 加载智能体时：完整恢复所有能力配置

### 全局设置持久化 ✅

全局智能体设置（`AgentGlobalSettings`）也正确持久化：

- **保存位置**：`agent_global_settings` 键
- **包含字段**：默认能力参数、执行超时、调试选项等

## 相关文件

- `rust-lib/flowy-ai/src/agent/config_manager.rs` - 智能体配置管理和持久化
- `rust-lib/flowy-ai/src/ai_manager.rs` - AI 管理器和动态工具发现
- `rust-lib/flowy-ai/src/agent/system_prompt.rs` - 系统提示词构建
- `rust-lib/flowy-ai/src/agent/tool_call_handler.rs` - 工具调用处理器
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/widgets/agent_dialog.dart` - UI 智能体对话框

修复已完成！智能体现在会从已配置的 MCP 服务器动态发现工具，并且能力配置会正确持久化。
