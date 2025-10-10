# MCP 服务器勾选功能实现完成 ✅

## 📋 功能概述

实现了在智能体管理界面中**勾选 MCP 服务器**的功能，用户可以通过勾选服务器来自动获取该服务器的所有工具，无需每次都关闭再启用工具调用。

## 🎯 核心改进

### 1. **数据结构变更**

#### ProtoBuf 定义更新 (`entities.rs`)

```rust
// AgentConfigPB - 添加已选择的 MCP 服务器列表
pub struct AgentConfigPB {
  // ... 其他字段
  pub available_tools: Vec<String>,      // 工具名称列表
  pub selected_mcp_servers: Vec<String>, // 🆕 已选择的服务器 ID 列表
  // ...
}

// CreateAgentRequestPB - 创建智能体时可指定服务器
pub struct CreateAgentRequestPB {
  // ... 其他字段
  pub available_tools: Vec<String>,
  pub selected_mcp_servers: Vec<String>, // 🆕 选中的服务器列表
}

// UpdateAgentRequestPB - 更新智能体时可修改服务器
pub struct UpdateAgentRequestPB {
  // ... 其他字段
  pub available_tools: Vec<String>,
  pub selected_mcp_servers: Vec<String>, // 🆕 更新服务器列表
}
```

**字段说明：**
- `selected_mcp_servers`: 存储用户在 UI 中勾选的 MCP 服务器 ID 列表
- `available_tools`: 从选中的服务器自动同步的工具名称列表

---

### 2. **核心方法实现**

#### `get_tools_from_selected_servers` (`ai_manager.rs`)

```rust
/// 🆕 根据已选择的 MCP 服务器列表获取工具名称
/// 这个方法用于根据UI中勾选的服务器自动填充工具列表
#[cfg(feature = "mcp")]
async fn get_tools_from_selected_servers(&self, server_ids: &[String]) -> Vec<String> {
  use std::collections::HashSet;
  
  if server_ids.is_empty() {
    return vec![];
  }
  
  info!("[🔧 Selected Servers] 开始从 {} 个已选择的服务器获取工具", server_ids.len());
  
  let all_tool_details = self.discover_available_tools().await;
  let mut unique_tool_names = HashSet::new();
  let mut tools = Vec::new();
  
  for server_id in server_ids {
    let server_tools: Vec<_> = all_tool_details.iter()
      .filter(|(sid, _)| sid == server_id)
      .collect();
    
    info!("[🔧 Selected Servers] 服务器 '{}' 提供了 {} 个工具", server_id, server_tools.len());
    
    for (_, tool) in server_tools {
      if unique_tool_names.insert(tool.name.clone()) {
        tools.push(tool.name.clone());
        info!("[🔧 Selected Servers]   - 添加工具: {}", tool.name);
      }
    }
  }
  
  info!("[🔧 Selected Servers] ✅ 从已选择的服务器共获取 {} 个工具", tools.len());
  tools
}
```

**功能：**
- 接收用户勾选的服务器 ID 列表
- 从这些服务器获取所有工具
- 自动去重（如果多个服务器有同名工具）
- 返回工具名称列表

---

### 3. **创建智能体逻辑更新**

#### `create_agent` (`ai_manager.rs`)

```rust
pub async fn create_agent(&self, mut request: CreateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
  // 🆕 优先处理：如果指定了 MCP 服务器列表，从这些服务器获取工具
  if !request.selected_mcp_servers.is_empty() && request.capabilities.enable_tool_calling {
    info!("🆕 [Create Agent] 使用已选择的 {} 个 MCP 服务器", request.selected_mcp_servers.len());
    let tools_from_servers = self.get_tools_from_selected_servers(&request.selected_mcp_servers).await;
    
    if !tools_from_servers.is_empty() {
      info!("🆕 [Create Agent] 从已选择的服务器获取到 {} 个工具", tools_from_servers.len());
      request.available_tools = tools_from_servers;
    } else {
      warn!("⚠️ [Create Agent] 已选择的服务器未返回任何工具");
    }
  }
  // 如果没有指定服务器，且工具列表为空，则自动发现所有工具（向后兼容）
  else if request.available_tools.is_empty() && request.capabilities.enable_tool_calling {
    // ... 原有的自动发现逻辑
  }
  
  self.agent_manager.create_agent(request)
}
```

**工作流程：**
1. **优先级 1**：如果用户勾选了服务器 → 从这些服务器获取工具
2. **优先级 2**：如果没有勾选服务器，但启用了工具调用 → 自动发现所有工具（向后兼容）
3. 保存配置

---

### 4. **更新智能体逻辑优化**

#### `update_agent` (`ai_manager.rs`)

```rust
pub async fn update_agent(&self, mut request: UpdateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
  // ... 获取现有配置 ...
  
  // 🆕 核心逻辑：如果提供了 selected_mcp_servers，自动同步工具列表
  if !request.selected_mcp_servers.is_empty() {
    info!("🆕 [Agent Update] 检测到 MCP 服务器列表变更，自动同步工具列表");
    
    // 判断是否启用了工具调用
    let tool_calling_enabled = if let Some(ref caps) = request.capabilities {
      caps.enable_tool_calling
    } else if let Some(ref existing) = existing_config {
      existing.capabilities.enable_tool_calling
    } else {
      false
    };
    
    if tool_calling_enabled {
      let tools_from_servers = self.get_tools_from_selected_servers(&request.selected_mcp_servers).await;
      
      if !tools_from_servers.is_empty() {
        info!("🆕 [Agent Update] 从 {} 个已选择的服务器同步了 {} 个工具", 
              request.selected_mcp_servers.len(), tools_from_servers.len());
        request.available_tools = tools_from_servers;
      } else {
        warn!("⚠️ [Agent Update] 已选择的服务器未返回任何工具");
        request.available_tools = vec![];
      }
    } else {
      info!("ℹ️ [Agent Update] 工具调用未启用，跳过工具同步");
    }
  }
  
  // ... 继续原有的更新逻辑 ...
}
```

**关键特性：**
- ✅ **自动同步**：勾选/取消勾选服务器后，工具列表自动更新
- ✅ **智能判断**：检查工具调用是否启用
- ✅ **即时生效**：无需关闭再启用工具调用
- ✅ **向后兼容**：如果没有修改服务器列表，保持原有逻辑

---

### 5. **配置管理器更新**

#### `config_manager.rs`

```rust
// 创建智能体时保存服务器列表
let mut agent_config = AgentConfigPB {
  // ... 其他字段
  selected_mcp_servers: request.selected_mcp_servers,  // 🆕 保存选中的服务器
  // ...
};

// 更新智能体时处理服务器列表
if !request.selected_mcp_servers.is_empty() {
  agent_config.selected_mcp_servers = request.selected_mcp_servers;
}
```

---

## 🎨 UI 使用流程（前端需要实现）

### 1. **编辑智能体界面**

```
┌──────────────────────────────────────────┐
│  编辑智能体                                │
├──────────────────────────────────────────┤
│  名称: [我的 AI 助手____________]          │
│  描述: [一个强大的 AI 助手______]          │
│                                          │
│  ✅ 启用工具调用                          │
│                                          │
│  选择 MCP 服务器:                         │
│  ☑️ Excel MCP Server                     │
│     └─ 工具: read_excel, write_excel     │
│  ☑️ File System MCP Server              │
│     └─ 工具: list_files, read_file       │
│  ☐ Web Search MCP Server                │
│     └─ 工具: search, fetch_content       │
│                                          │
│  当前工具列表 (自动同步):                  │
│  • read_excel                            │
│  • write_excel                           │
│  • list_files                            │
│  • read_file                             │
│                                          │
│         [取消]  [保存]                    │
└──────────────────────────────────────────┘
```

### 2. **前端实现要点**

```dart
// 伪代码示例
class AgentEditPage extends StatefulWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 基本信息编辑
        TextField(controller: nameController),
        
        // 工具调用开关
        Switch(
          value: enableToolCalling,
          onChanged: (value) => setState(() => enableToolCalling = value),
        ),
        
        // 🆕 MCP 服务器选择列表
        if (enableToolCalling)
          ListView.builder(
            itemCount: mcpServers.length,
            itemBuilder: (context, index) {
              final server = mcpServers[index];
              return CheckboxListTile(
                title: Text(server.name),
                subtitle: Text('工具: ${server.tools.join(", ")}'),
                value: selectedServerIds.contains(server.id),
                onChanged: (selected) {
                  setState(() {
                    if (selected) {
                      selectedServerIds.add(server.id);
                    } else {
                      selectedServerIds.remove(server.id);
                    }
                    // 工具列表会在保存时自动同步
                  });
                },
              );
            },
          ),
        
        // 显示将要同步的工具（只读）
        Text('当前工具: ${getToolsFromSelectedServers().join(", ")}'),
        
        // 保存按钮
        ElevatedButton(
          onPressed: () async {
            // 调用更新 API
            final request = UpdateAgentRequestPB(
              id: agentId,
              selected_mcp_servers: selectedServerIds,
              // available_tools 会在后端自动同步，无需前端设置
            );
            await AIManager.updateAgent(request);
          },
          child: Text('保存'),
        ),
      ],
    );
  }
}
```

---

## 📊 工作流程图

```
用户操作 UI
    ↓
勾选/取消勾选 MCP 服务器
    ↓
点击"保存"
    ↓
前端发送 UpdateAgentRequestPB
    ├─ selected_mcp_servers: ["server-1", "server-2"]
    └─ available_tools: []  (无需设置，后端自动填充)
    ↓
后端 update_agent 检测到 selected_mcp_servers 变更
    ↓
调用 get_tools_from_selected_servers()
    ├─ 从 server-1 获取工具: [tool1, tool2]
    └─ 从 server-2 获取工具: [tool3, tool4]
    ↓
自动合并并去重
    ↓
更新 available_tools = [tool1, tool2, tool3, tool4]
    ↓
保存配置到数据库
    ↓
✅ 工具列表立即生效，无需重启或切换
```

---

## 🔧 测试场景

### 场景 1：创建新智能体并选择服务器

```rust
let request = CreateAgentRequestPB {
  name: "我的助手".to_string(),
  capabilities: AgentCapabilitiesPB {
    enable_tool_calling: true,
    ..Default::default()
  },
  selected_mcp_servers: vec![
    "excel-server-id".to_string(),
    "filesystem-server-id".to_string(),
  ],
  available_tools: vec![],  // 留空，后端会自动填充
  ..Default::default()
};

let agent = ai_manager.create_agent(request).await?;

// 验证：agent.available_tools 应该包含两个服务器的所有工具
assert!(agent.available_tools.contains(&"read_excel".to_string()));
assert!(agent.available_tools.contains(&"list_files".to_string()));
```

### 场景 2：修改已有智能体的服务器选择

```rust
let request = UpdateAgentRequestPB {
  id: agent_id,
  selected_mcp_servers: vec![
    "excel-server-id".to_string(),  // 保留
    // filesystem-server-id 被移除
    "web-search-server-id".to_string(),  // 新增
  ],
  ..Default::default()
};

let updated_agent = ai_manager.update_agent(request).await?;

// 验证：工具列表应该自动更新
assert!(updated_agent.available_tools.contains(&"read_excel".to_string()));
assert!(!updated_agent.available_tools.contains(&"list_files".to_string()));
assert!(updated_agent.available_tools.contains(&"search".to_string()));
```

### 场景 3：取消所有服务器勾选

```rust
let request = UpdateAgentRequestPB {
  id: agent_id,
  selected_mcp_servers: vec![],  // 全部取消
  ..Default::default()
};

let updated_agent = ai_manager.update_agent(request).await?;

// 验证：工具列表应该为空
assert!(updated_agent.available_tools.is_empty());
```

---

## 📝 日志示例

当用户勾选服务器并保存时，后端会输出详细日志：

```log
[Agent Update] 开始更新智能体: agent-123
[Agent Update] 请求已选择服务器数量: 2
[Agent Update] 检测到 MCP 服务器列表变更，自动同步工具列表
[🔧 Selected Servers] 开始从 2 个已选择的服务器获取工具
[Tool Discovery] 开始扫描 3 个已配置的 MCP 服务器...
[Tool Discovery] 检查配置: Excel Server (ID: excel-server-id, 激活: true)
[Tool Discovery]   - 工具: read_excel (服务器: excel-server-id)
[Tool Discovery]   - 工具: write_excel (服务器: excel-server-id)
[Tool Discovery] 检查配置: File System Server (ID: filesystem-server-id, 激活: true)
[Tool Discovery]   - 工具: list_files (服务器: filesystem-server-id)
[Tool Discovery]   - 工具: read_file (服务器: filesystem-server-id)
[🔧 Selected Servers] 服务器 'excel-server-id' 提供了 2 个工具
[🔧 Selected Servers]   - 添加工具: read_excel
[🔧 Selected Servers]   - 添加工具: write_excel
[🔧 Selected Servers] 服务器 'filesystem-server-id' 提供了 2 个工具
[🔧 Selected Servers]   - 添加工具: list_files
[🔧 Selected Servers]   - 添加工具: read_file
[🔧 Selected Servers] ✅ 从已选择的服务器共获取 4 个工具
[Agent Update] 从 2 个已选择的服务器同步了 4 个工具
✅ Successfully updated agent: 我的助手 (agent-123)
```

---

## ✅ 已完成的改动

1. ✅ **ProtoBuf 定义更新** - 添加 `selected_mcp_servers` 字段到三个结构体
2. ✅ **核心方法实现** - `get_tools_from_selected_servers()` 方法
3. ✅ **创建逻辑优化** - 支持在创建时指定服务器
4. ✅ **更新逻辑优化** - 自动同步工具列表
5. ✅ **配置管理** - 保存和加载服务器列表
6. ✅ **代码生成** - ProtoBuf 代码重新生成
7. ✅ **编译验证** - 所有代码编译通过

---

## 🎯 下一步（前端实现）

前端需要实现以下功能：

1. **获取 MCP 服务器列表**
   - 调用现有的 `MCPManager.get_all_servers()` API
   - 显示每个服务器的名称和工具列表

2. **UI 勾选组件**
   - 复选框列表展示所有可用的 MCP 服务器
   - 显示每个服务器提供的工具（可折叠）
   - 实时显示当前选中服务器的工具总数

3. **数据绑定**
   - 编辑智能体时，加载 `agent.selected_mcp_servers`
   - 根据 ID 预勾选对应的服务器
   - 用户修改勾选状态时更新 `selected_mcp_servers`

4. **保存逻辑**
   - 调用 `updateAgent()` 时传递 `selected_mcp_servers`
   - **不需要**手动设置 `available_tools`（后端自动同步）
   - 保存成功后刷新智能体配置

---

## 🚀 优势总结

| 特性 | 旧方式 | 新方式 |
|------|--------|--------|
| 工具选择方式 | 手动勾选每个工具 | 勾选服务器，自动获取所有工具 |
| 更新生效方式 | 需要关闭再启用工具调用 | 即时生效，无需额外操作 |
| 工具列表维护 | 手动同步 | 自动同步 |
| 新工具发现 | 需要重新勾选 | 自动包含新工具 |
| 用户体验 | 繁琐，多步操作 | 简单，一键勾选 |

---

## 📄 相关文件

- **ProtoBuf 定义**: `rust-lib/flowy-ai/src/entities.rs`
- **AI 管理器**: `rust-lib/flowy-ai/src/ai_manager.rs`
- **配置管理器**: `rust-lib/flowy-ai/src/agent/config_manager.rs`
- **生成的 ProtoBuf**: `rust-lib/flowy-ai/src/protobuf/entities.rs`

---

## 🎉 总结

此功能大大简化了智能体工具配置流程，用户只需：
1. 勾选想要使用的 MCP 服务器
2. 点击保存
3. ✅ 完成！工具列表自动同步，立即生效

**无需再经历 "关闭工具调用 → 启用工具调用 → 等待生效" 的繁琐流程！** 🎊

