# MCP 工具详情集成到系统提示 - 实现计划

## ✅ 已完成

### 1. 工具详情获取机制
修改了 `discover_available_tools` 方法,现在返回:
```rust
async fn discover_available_tools(&self) -> (Vec<String>, HashMap<String, MCPTool>)
```

- `Vec<String>`: 工具名称列表(用于向后兼容)
- `HashMap<String, MCPTool>`: 工具名称 → 完整工具详情的映射

### 2. 数据来源
直接使用现有的持久化数据:
```rust
// 从 MCPServerConfig 获取缓存的工具
pub struct MCPServerConfig {
    pub cached_tools: Option<Vec<MCPTool>>,  // ✅ 已持久化!
    pub last_tools_check_at: Option<SystemTime>,
}
```

### 3. 工具信息包含
`MCPTool` 已经包含所有需要的信息:
```rust
pub struct MCPTool {
    pub name: String,              // 工具名称
    pub description: String,        // 功能描述
    pub input_schema: Value,        // JSON Schema 参数定义
    pub annotations: Option<MCPToolAnnotations>,  // 元数据
}
```

## 🎯 下一步:将工具详情集成到系统提示

### 方案选择:临时方案(快速实现)

在智能体聊天时,动态获取工具详情并增强系统提示:

```rust
// rust-lib/flowy-ai/src/ai_manager.rs
pub async fn stream_chat_message(&self, ...) -> FlowyResult<StreamAnswer> {
    // ...
    if let Some(mut config) = self.agent_manager.get_agent_config(agent_id) {
        // 获取工具详情
        let (_, tool_details) = self.discover_available_tools().await;
        
        // 增强系统提示
        let enhanced_prompt = self.build_enhanced_prompt_with_tool_details(
            &config,
            &tool_details
        );
        
        // 使用增强后的提示
        chat.stream_chat_message(..., enhanced_prompt, ...).await
    }
}
```

### 实现步骤

#### Step 1: 创建工具详情格式化函数

```rust
// rust-lib/flowy-ai/src/agent/system_prompt.rs

/// 格式化单个工具的详细信息
fn format_tool_details(tool: &MCPTool) -> String {
    let mut details = String::new();
    
    details.push_str(&format!("**{}**\n", tool.name));
    details.push_str(&format!("  {}\n", tool.description));
    
    // 解析 JSON Schema 获取参数
    if let Some(properties) = tool.input_schema.get("properties").and_then(|p| p.as_object()) {
        details.push_str("  Parameters:\n");
        
        let required_fields = tool.input_schema.get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        
        for (name, schema) in properties {
            let param_type = schema.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("any");
            
            let description = schema.get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            
            let required_mark = if required_fields.contains(&name.as_str()) {
                "required"
            } else {
                "optional"
            };
            
            details.push_str(&format!(
                "    - {} ({}): {} [{}]\n",
                name, param_type, description, required_mark
            ));
        }
    }
    
    details.push_str("\n");
    details
}

/// 构建包含工具详情的系统提示
pub fn build_agent_system_prompt_with_tools(
    config: &AgentConfigPB,
    tool_details: &HashMap<String, MCPTool>,
) -> String {
    let mut prompt = build_agent_system_prompt(config);
    
    // 如果启用工具调用且有工具详情,替换简单的工具列表
    if config.capabilities.enable_tool_calling && !tool_details.is_empty() {
        // 查找并替换工具列表部分
        prompt.push_str("\n\n## Available Tools (详细信息)\n\n");
        
        for tool_name in &config.available_tools {
            if let Some(tool) = tool_details.get(tool_name) {
                prompt.push_str(&format_tool_details(tool));
            }
        }
    }
    
    prompt
}
```

#### Step 2: 修改聊天流程使用增强提示

```rust
// rust-lib/flowy-ai/src/ai_manager.rs

pub async fn stream_chat_message(
    &self,
    workspace_id: String,
    chat_id: String,
    message: String,
    message_id: i64,
    metadata: String,
    ai_model: AIModel,
    reply_message_id: i64,
) -> FlowyResult<StreamAnswer> {
    // ...
    
    match self.agent_manager.get_agent_config(agent_id) {
        Some(mut config) => {
            info!("[Chat] Using agent: {} ({})", config.name, config.id);
            
            // 🆕 获取工具详情
            let (_, tool_details) = self.discover_available_tools().await;
            
            info!("[Chat] Agent has {} tools with {} tool details", 
                  config.available_tools.len(), tool_details.len());
            
            // 自动填充工具列表(如果为空)
            // ... (现有逻辑)
            
            // 🆕 构建增强的系统提示
            let base_prompt = crate::agent::system_prompt::build_agent_system_prompt(&config);
            let enhanced_prompt = if !tool_details.is_empty() {
                crate::agent::system_prompt::build_agent_system_prompt_with_tools(&config, &tool_details)
            } else {
                base_prompt
            };
            
            // 使用增强提示
            let mut chat = Chat::new(
                self.chat_manager.clone(),
                // ...
            );
            
            chat.stream_chat_message(
                workspace_id.clone(),
                chat_id.clone(),
                message,
                ai_model,
                Some(agent_id.to_string()),
                Some(enhanced_prompt),  // 使用增强的提示
                // ...
            ).await
        }
        None => {
            // 无智能体的情况
        }
    }
}
```

#### Step 3: 修改 Chat 接受自定义系统提示

```rust
// rust-lib/flowy-ai/src/chat.rs

pub async fn stream_chat_message(
    &mut self,
    workspace_id: String,
    chat_id: String,
    message: String,
    ai_model: AIModel,
    agent_id: Option<String>,
    custom_system_prompt: Option<String>,  // 🆕 添加参数
    // ...
) -> Result<StreamAnswer, FlowyError> {
    // ...
    
    let system_prompt = if let Some(custom_prompt) = custom_system_prompt {
        // 使用自定义提示(已包含工具详情)
        custom_prompt
    } else if let Some(agent_id) = &agent_id {
        // 从智能体配置生成提示
        // ...
    } else {
        // 默认提示
        // ...
    };
    
    // ...
}
```

## 📝 生成的系统提示示例

### 之前
```
Available tools: read_data_from_excel, write_data_to_excel, apply_formula
```

### 之后
```
## Available Tools (详细信息)

**read_data_from_excel**
  Read data from Excel worksheet with cell metadata including validation rules.
  Parameters:
    - filepath (string): Path to Excel file [required]
    - sheet_name (string): Name of worksheet [required]
    - start_cell (string): Starting cell (default A1) [optional]
    - end_cell (string): Ending cell (auto-expands if not provided) [optional]
    - preview_only (boolean): Whether to return preview only [optional]

**write_data_to_excel**
  Write data to Excel worksheet
  Parameters:
    - filepath (string): Path to Excel file [required]
    - sheet_name (string): Name of worksheet [required]
    - data (array): List of lists containing data to write [required]
    - start_cell (string): Cell to start writing to (default "A1") [optional]

**apply_formula**
  Apply Excel formula to cell with verification
  Parameters:
    - filepath (string): Path to Excel file [required]
    - sheet_name (string): Name of worksheet [required]
    - cell (string): Target cell address [required]
    - formula (string): Excel formula to apply [required]
```

## 🎓 AI 现在可以

1. **知道工具的作用**: 通过 description 了解工具功能
2. **知道何时使用**: 根据用户问题匹配工具描述
3. **知道如何使用**: 看到完整的参数列表和类型
4. **知道哪些参数必需**: 标记为 [required] 或 [optional]

## 🧪 测试验证

### 1. 检查日志
```
[Chat] Agent has 25 tools with 25 tool details
```

### 2. 测试对话
```
用户: "帮我读取 test.xlsx 的 Sheet1 工作表"
AI: <tool_call>
{
  "id": "call_001",
  "tool_name": "read_data_from_excel",
  "arguments": {
    "filepath": "test.xlsx",
    "sheet_name": "Sheet1"
  }
}
</tool_call>
```

### 3. 验证系统提示
添加日志输出完整的系统提示:
```rust
debug!("=== Enhanced System Prompt ===\n{}\n===", enhanced_prompt);
```

## ⚡ 性能考虑

### 当前方案优势
- ✅ 使用缓存的工具信息(不需要重新连接 MCP 服务器)
- ✅ 只在聊天开始时获取一次
- ✅ 工具详情已经持久化在配置中

### 可能的优化
- 将工具详情缓存在 AIManager 中,避免每次聊天都查询
- 只格式化实际使用的工具(延迟加载)

## 📊 影响范围

### 修改的文件
1. ✅ `rust-lib/flowy-ai/src/ai_manager.rs` - 获取工具详情
2. ⏳ `rust-lib/flowy-ai/src/agent/system_prompt.rs` - 格式化工具详情
3. ⏳ `rust-lib/flowy-ai/src/chat.rs` - 接受自定义提示

### 不需要修改
- ❌ Protocol Buffers 定义(继续使用 `Vec<String>` 存储工具名称)
- ❌ 数据库 schema(工具详情已在 `MCPServerConfig.cached_tools` 中)
- ❌ Flutter UI(不需要改动)

## 🚀 实施优先级

### 高优先级 (立即实施)
1. 实现 `format_tool_details` 函数
2. 实现 `build_agent_system_prompt_with_tools` 函数
3. 修改 `stream_chat_message` 使用增强提示

### 中优先级 (后续优化)
1. 添加工具详情缓存
2. 优化提示词长度(只包含相关工具)
3. 添加工具使用示例

### 低优先级 (可选)
1. UI 显示工具详情
2. 工具详情的国际化
3. 工具分类和组织

## 📖 总结

通过这个临时方案,我们可以:
- ✅ **重用现有数据**: 直接使用已持久化的 `cached_tools`
- ✅ **最小改动**: 只需修改 3 个文件
- ✅ **快速见效**: AI 立即能看到工具详情并正确使用
- ✅ **向后兼容**: 不影响现有的工具名称列表机制

这是最实用的解决方案!

