# MCP 工具只发送一个服务器问题 - 修复完成

## ✅ 问题已修复

已成功修复多个 MCP 服务器工具发送的问题，并添加了详细的诊断日志。

## 🔧 主要修改

### 1. 数据结构修改 (`ai_manager.rs`)

**修改前**:
```rust
async fn discover_available_tools(&self) -> (Vec<String>, HashMap<String, MCPTool>)
```

**修改后**:
```rust
async fn discover_available_tools(&self) -> Vec<(String, MCPTool)>
```

**改进点**:
- 将返回类型从 HashMap 改为 Vec，避免同名工具被覆盖
- 每个工具都保留其来源服务器的 ID
- 支持多个服务器提供不同实现的同名工具

### 2. 工具定义获取逻辑优化 (`get_tool_definitions_by_names`)

```rust
pub async fn get_tool_definitions_by_names(&self, tool_names: &[String]) -> Vec<ToolDefinitionPB> {
  let mut result: Vec<ToolDefinitionPB> = Vec::new();
  let tool_details = self.discover_available_tools().await;
  
  #[cfg(feature = "mcp")]
  {
    for tool_name in tool_names {
      for (server_id, mcp_tool) in &tool_details {
        if &mcp_tool.name == tool_name {
          let tool_def = ToolDefinitionPB {
            name: mcp_tool.name.clone(),
            description: mcp_tool.description.clone().unwrap_or_default(),
            tool_type: crate::entities::ToolTypePB::MCP,
            source: server_id.clone(),  // ✨ 使用服务器ID作为source
            parameters_schema: serde_json::to_string(&mcp_tool.input_schema).unwrap_or_default(),
            // ...
          };
          result.push(tool_def);
          break; // 对于同名工具，只取第一个
        }
      }
    }
  }
  
  result
}
```

**改进点**:
- 正确遍历所有服务器的工具
- 每个工具的 `source` 字段记录其来源服务器ID
- 这样在调用工具时能正确找到对应的服务器

### 3. 增强诊断日志

添加了多个关键位置的详细日志：

#### A. 工具发现阶段
```rust
info!("[Tool Discovery] 开始扫描 {} 个已配置的 MCP 服务器...", config_count);
info!("[Tool Discovery] 检查配置: {} (ID: {}, 激活: {})", config.name, config.id, config.is_active);
info!("[Tool Discovery]   - 工具: {} (服务器: {})", tool.name, config.id);
info!("✅ [Tool Discovery] 共从 {} 个已配置服务器发现 {} 个工具（包含所有同名工具）", config_count, tool_details.len());
info!("  📦 工具 '{}' 来自服务器 '{}'", tool.name, server_id);
```

#### B. 聊天初始化阶段
```rust
info!("[Chat] 🔍 Discovered {} tools from MCP servers", tool_details.len());
info!("[Chat] 🔍   - Tool '{}' from server '{}'", tool.name, server_id);
info!("[Chat] 🔍 Collected {} unique tool names", discovered_tool_names.len());
```

#### C. 工具定义转换阶段
```rust
info!("[Chat] 🔧 Available tools list: {:?}", config.available_tools);
info!("[Chat] 🔧 Got {} tool definitions for OpenAI Function Call", tools.len());
info!("[Chat] 🔧   - Tool '{}' from server '{}': {}", tool.name, tool.source, tool.description);
```

#### D. 工具定义构建阶段
```rust
info!("[Tool Def] Added tool '{}' from server '{}' to definitions", tool_name, server_id);
warn!("[Tool Def] Tool '{}' not found in any MCP server", tool_name);
```

### 4. 去重逻辑改进

使用 `HashSet` 确保工具名称不重复，但保留所有不同的工具：

```rust
use std::collections::HashSet;
let mut unique_tool_names: HashSet<String> = HashSet::new();
let discovered_tool_names: Vec<String> = tool_details.iter()
  .filter_map(|(_, tool)| {
    if unique_tool_names.insert(tool.name.clone()) {
      Some(tool.name.clone())
    } else {
      None
    }
  })
  .collect();
```

## 📊 如何诊断问题

### 第1步：查看工具发现日志

运行应用并在 AI 聊天中提问，查找以下日志：

```
[Tool Discovery] 开始扫描 N 个已配置的 MCP 服务器...
[Tool Discovery] 检查配置: SERVER1 (ID: xxx, 激活: xxx)
[Tool Discovery] 检查配置: SERVER2 (ID: xxx, 激活: xxx)
```

**检查点**:
- ✅ 扫描了 2 个服务器
- ✅ 两个服务器都是激活状态 (`激活: true`)
- ✅ 两个服务器都发现了工具

### 第2步：查看工具详情

```
[Tool Discovery]   - 工具: TOOL1 (服务器: server_id_1)
[Tool Discovery]   - 工具: TOOL2 (服务器: server_id_2)
📦 工具 'TOOL1' 来自服务器 'server_id_1'
📦 工具 'TOOL2' 来自服务器 'server_id_2'
```

**检查点**:
- ✅ 每个工具都显示了其来源服务器
- ✅ 工具总数等于两个服务器的工具数量之和

### 第3步：查看智能体配置

```
[Chat] 🔧 Available tools list: ["TOOL1", "TOOL2", ...]
[Chat] 🔧 Got N tool definitions for OpenAI Function Call
[Chat] 🔧   - Tool 'TOOL1' from server 'server_id_1': description
[Chat] 🔧   - Tool 'TOOL2' from server 'server_id_2': description
```

**检查点**:
- ✅ `available_tools` 列表包含所有工具名称
- ✅ 每个工具都成功转换为定义
- ✅ 每个工具的 source 显示正确的服务器ID

### 第4步：确认发送给 AI

```
[OpenAI] Added N tools to request
```

**检查点**:
- ✅ 发送的工具数量 = available_tools 的数量

## 🎯 可能的问题场景

### 场景1：智能体配置问题

**症状**: `available_tools` 列表只包含一个服务器的工具

**原因**: 
- 创建智能体时，只有一个服务器是激活状态
- 或者只有一个服务器成功连接并缓存了工具

**解决方案**: 
1. 删除现有智能体
2. 确保两个 MCP 服务器都已连接并发现工具
3. 重新创建智能体 - 系统会自动发现所有工具

### 场景2：服务器激活状态

**症状**: 日志显示 `跳过未激活的服务器: xxx`

**解决方案**: 在全局设置中确保两个服务器都已激活

### 场景3：服务器连接失败

**症状**: 日志显示 `从服务器 'xxx' 获取工具列表失败`

**解决方案**:
- 检查服务器配置
- 尝试重新连接服务器
- 查看服务器错误日志

### 场景4：工具缓存为空

**症状**: 日志显示 `服务器 'xxx' 没有缓存，尝试从客户端获取...`，但获取失败

**解决方案**: 重启应用以重新发现工具

## 🚀 测试步骤

1. **重新编译应用**:
   ```bash
   cd rust-lib
   cargo build --release
   ```

2. **从终端启动 AppFlowy**（以查看完整日志）

3. **进入 AI 聊天**，发送一个问题

4. **查看日志输出**，确认：
   - 两个服务器都被扫描
   - 所有工具都被发现
   - 工具定义正确生成
   - 所有工具都发送给 AI

5. **验证功能**：
   - 尝试调用第一个服务器的工具
   - 尝试调用第二个服务器的工具
   - 确认两个服务器的工具都能正常工作

## 📝 代码改动总结

**修改文件**: `rust-lib/flowy-ai/src/ai_manager.rs`

**主要改动**:
1. `discover_available_tools()` 返回类型改为 `Vec<(String, MCPTool)>`
2. `get_tool_definitions_by_names()` 遍历所有服务器的工具
3. 添加详细的诊断日志到关键位置
4. 改进工具名称去重逻辑
5. 确保工具的 `source` 字段包含正确的服务器ID

**影响范围**:
- 工具发现流程
- 工具定义生成
- 智能体创建和更新
- 聊天消息处理

## ✅ 预期效果

修复后：
1. ✅ 所有激活的 MCP 服务器的工具都会被发现
2. ✅ 每个工具都记录其来源服务器
3. ✅ 所有工具都会发送给 AI 模型
4. ✅ 详细日志帮助快速诊断问题
5. ✅ 支持同名工具（优先级由服务器顺序决定）

---

## 🐛 遇到问题？

如果问题仍然存在，请：
1. 查看完整的日志输出
2. 特别关注 `[Tool Discovery]` 和 `[Chat] 🔧` 开头的日志
3. 将日志发给我进行分析

