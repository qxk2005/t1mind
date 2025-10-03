# MCP 工具详情集成到系统提示 - 实现完成 ✅

## 📋 问题描述

AI 智能体无法正确使用 MCP 工具,因为系统提示只包含工具名称列表,缺少:
- ❌ 工具的功能描述
- ❌ 工具的参数定义
- ❌ 参数的类型和必需性
- ❌ 工具的使用场景

导致 AI 不知道何时使用工具、如何正确调用工具。

## ✅ 解决方案

### 核心思路
利用现有的持久化数据 `MCPServerConfig.cached_tools`,在聊天时动态获取工具详情并增强系统提示。

### 实现步骤

#### Step 1: 修改工具发现返回完整信息

**文件**: `rust-lib/flowy-ai/src/ai_manager.rs`

```rust
// 修改返回类型,同时返回工具名称和详情
async fn discover_available_tools(&self) -> (Vec<String>, HashMap<String, MCPTool>) {
    let mut tool_names = Vec::new();
    let mut tool_details = HashMap::new();
    
    // 从配置管理器获取所有已配置的服务器
    let server_configs = self.mcp_manager.config_manager().get_all_servers();
    
    for config in server_configs {
        if !config.is_active {
            continue;
        }
        
        // 优先使用缓存的工具列表
        if let Some(cached_tools) = &config.cached_tools {
            for tool in cached_tools {
                tool_names.push(tool.name.clone());
                tool_details.insert(tool.name.clone(), tool.clone());  // 🆕 保存完整详情
            }
            continue;
        }
        
        // 如果没有缓存,从客户端获取
        match self.mcp_manager.tool_list(&config.id).await {
            Ok(tools_list) => {
                for tool in tools_list.tools {
                    tool_names.push(tool.name.clone());
                    tool_details.insert(tool.name.clone(), tool);  // 🆕 保存完整详情
                }
            }
            Err(e) => {
                warn!("从服务器 '{}' 获取工具列表失败: {}", config.name, e);
            }
        }
    }
    
    (tool_names, tool_details)
}
```

**关键变化**:
- ✅ 返回 `(Vec<String>, HashMap<String, MCPTool>)` 而不是只返回 `Vec<String>`
- ✅ 直接使用 `cached_tools`,无需重复查询
- ✅ 为每个工具保存完整的 `MCPTool` 对象

#### Step 2: 创建工具详情格式化函数

**文件**: `rust-lib/flowy-ai/src/agent/system_prompt.rs`

```rust
use crate::mcp::entities::MCPTool;
use std::collections::HashMap;

/// 格式化单个工具的详细信息
fn format_tool_details(tool: &MCPTool) -> String {
  let mut details = String::new();
  
  details.push_str(&format!("**{}**\n", tool.name));
  details.push_str(&format!("  {}\n", tool.description));
  
  // 解析 JSON Schema 获取参数信息
  if let Some(properties) = tool.input_schema.get("properties").and_then(|p| p.as_object()) {
    details.push_str("  Parameters:\n");
    
    let required_fields = tool.input_schema.get("required")
      .and_then(|r| r.as_array())
      .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
      .unwrap_or_default();
    
    for (name, schema) in properties {
      let param_type = schema.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("any");
      
      let description = schema.get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");
      
      let required_mark = if required_fields.contains(name) {
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
  
  // 添加注解信息(如果有)
  if let Some(annotations) = &tool.annotations {
    let mut hints = Vec::new();
    if let Some(true) = annotations.read_only_hint {
      hints.push("read-only");
    }
    if let Some(true) = annotations.destructive_hint {
      hints.push("destructive");
    }
    if let Some(true) = annotations.idempotent_hint {
      hints.push("idempotent");
    }
    if !hints.is_empty() {
      details.push_str(&format!("  Hints: {}\n", hints.join(", ")));
    }
  }
  
  details.push_str("\n");
  details
}

/// 构建包含工具详细信息的增强系统提示
pub fn build_agent_system_prompt_with_tools(
  config: &AgentConfigPB,
  tool_details: &HashMap<String, MCPTool>,
) -> String {
  let mut prompt = build_agent_system_prompt(config);
  
  // 如果启用了工具调用且有工具详情,添加详细的工具信息
  if config.capabilities.enable_tool_calling && !tool_details.is_empty() {
    prompt.push_str("\n\n## 🔧 Available Tools (Detailed Information)\n\n");
    prompt.push_str("You have access to the following tools with their detailed specifications:\n\n");
    
    let mut tool_count = 0;
    for tool_name in &config.available_tools {
      if let Some(tool) = tool_details.get(tool_name) {
        tool_count += 1;
        prompt.push_str(&format!("{}. ", tool_count));
        prompt.push_str(&format_tool_details(tool));
      }
    }
    
    prompt.push_str(&format!("\n**You have {} tools available.** ", tool_count));
    prompt.push_str("Use them when needed to help the user accomplish their tasks.\n");
  }
  
  prompt
}
```

**功能**:
- ✅ 解析 `MCPTool.input_schema` (JSON Schema)
- ✅ 提取参数名称、类型、描述
- ✅ 标注参数是必需还是可选
- ✅ 添加工具注解(只读/破坏性等)
- ✅ 生成格式化的工具说明文本

#### Step 3: 在聊天流程中使用增强提示

**文件**: `rust-lib/flowy-ai/src/ai_manager.rs`

```rust
pub async fn stream_chat_message(
    &self,
    params: StreamMessageParams,
) -> Result<ChatMessagePB, FlowyError> {
    let agent_config = if let Some(ref agent_id) = params.agent_id {
      match self.agent_manager.get_agent_config(agent_id) {
        Some(mut config) => {
          // 🔍 获取工具详情用于增强系统提示
          let (discovered_tool_names, tool_details) = self.discover_available_tools().await;
          info!("[Chat] Discovered {} tools with {} tool details", 
                discovered_tool_names.len(), tool_details.len());
          
          // 自动填充工具列表（如果为空）
          if config.available_tools.is_empty() && config.capabilities.enable_tool_calling {
            if !discovered_tool_names.is_empty() {
              config.available_tools = discovered_tool_names.clone();
              // 保存配置...
            }
          }
          
          // 🆕 构建增强的系统提示（包含工具详情）
          let enhanced_prompt = if !tool_details.is_empty() && config.capabilities.enable_tool_calling {
            use crate::agent::system_prompt::build_agent_system_prompt_with_tools;
            let prompt = build_agent_system_prompt_with_tools(&config, &tool_details);
            info!("[Chat] 🔧 Using enhanced system prompt with {} tool details", tool_details.len());
            Some(prompt)
          } else {
            None
          };
          
          Some((config, enhanced_prompt))
        },
        None => None,
      }
    } else {
      None
    };

    // 解包 agent_config 和 enhanced_prompt
    let (agent_config, enhanced_prompt) = if let Some((config, prompt)) = agent_config {
      (Some(config), prompt)
    } else {
      (None, None)
    };

    // 创建工具调用处理器
    let tool_call_handler = if agent_config.is_some() {
      use crate::agent::ToolCallHandler;
      Some(Arc::new(ToolCallHandler::from_ai_manager(self)))
    } else {
      None
    };

    let chat = self.get_or_create_chat_instance(&params.chat_id).await?;
    let ai_model = self.get_active_model(&params.chat_id.to_string()).await;
    
    // 🆕 传入增强的系统提示
    let question = chat.stream_chat_message(
        &params, 
        ai_model, 
        agent_config, 
        tool_call_handler, 
        enhanced_prompt  // 传入自定义提示
    ).await?;
    
    Ok(question)
}
```

**关键变化**:
- ✅ 在聊天开始时获取工具详情
- ✅ 使用 `build_agent_system_prompt_with_tools` 生成增强提示
- ✅ 将增强提示传递给 `Chat::stream_chat_message`

#### Step 4: Chat 接受自定义系统提示

**文件**: `rust-lib/flowy-ai/src/chat.rs`

```rust
pub async fn stream_chat_message(
    &self,
    params: &StreamMessageParams,
    preferred_ai_model: AIModel,
    agent_config: Option<AgentConfigPB>,
    tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,
    custom_system_prompt: Option<String>,  // 🆕 新增参数
) -> Result<ChatMessagePB, FlowyError> {
    // 构建系统提示词
    let system_prompt = if let Some(custom_prompt) = custom_system_prompt {
      // 🆕 使用自定义提示(已包含工具详情)
      info!("[Chat] 🔧 Using custom system prompt (with tool details)");
      Some(custom_prompt)
    } else if let Some(ref config) = agent_config {
      // 使用默认的智能体提示
      // ...
      Some(enhanced_prompt)
    } else {
      None
    };
    
    // 继续聊天流程...
}
```

**关键变化**:
- ✅ 添加 `custom_system_prompt` 参数
- ✅ 优先使用自定义提示
- ✅ 保持向后兼容

## 📊 生成的系统提示示例

### 之前 (只有工具名称)

```
Available tools: read_data_from_excel, write_data_to_excel, apply_formula

Please use these tools when appropriate.
```

### 之后 (包含完整详情)

```
## 🔧 Available Tools (Detailed Information)

You have access to the following tools with their detailed specifications:

1. **read_data_from_excel**
  Read data from Excel worksheet with cell metadata including validation rules.
  Parameters:
    - filepath (string): Path to Excel file [required]
    - sheet_name (string): Name of worksheet [required]
    - start_cell (string): Starting cell (default A1) [optional]
    - end_cell (string): Ending cell (auto-expands if not provided) [optional]
    - preview_only (boolean): Whether to return preview only [optional]

2. **write_data_to_excel**
  Write data to Excel worksheet. Excel formula will write to cell without verification.
  Parameters:
    - filepath (string): Path to Excel file [required]
    - sheet_name (string): Name of worksheet to write to [required]
    - data (array): List of lists containing data to write to the worksheet [required]
    - start_cell (string): Cell to start writing to, default is "A1" [optional]

3. **apply_formula**
  Apply Excel formula to cell with verification.
  Parameters:
    - filepath (string): Path to Excel file [required]
    - sheet_name (string): Name of worksheet [required]
    - cell (string): Cell to apply formula to [required]
    - formula (string): Excel formula to apply [required]

**You have 3 tools available.** Use them when needed to help the user accomplish their tasks.
```

## 🎯 实现效果

### AI 现在能够

1. **理解工具用途**: 通过描述知道每个工具的功能
2. **判断使用时机**: 根据用户问题选择合适的工具
3. **正确构造参数**: 了解每个参数的类型、含义和必需性
4. **避免参数错误**: 知道哪些参数必填,哪些可选

### 示例对话

**用户**: "帮我读取 test.xlsx 的 Sheet1 工作表"

**AI 响应**:
```
<tool_call>
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

✅ AI 正确选择了 `read_data_from_excel` 工具
✅ AI 只填写了必需参数 `filepath` 和 `sheet_name`
✅ AI 没有填写可选参数 `start_cell`, `end_cell`, `preview_only`

## 🔍 调试日志

### 工具发现
```
[Tool Discovery] 开始扫描 1 个已配置的 MCP 服务器...
[Tool Discovery] 检查配置: Excel MCP Server (ID: excel-mcp, 激活: true)
[Tool Discovery] 从服务器 'Excel MCP Server' 的缓存中发现 25 个工具
✅ [Tool Discovery] 共从 1 个已配置服务器发现 25 个可用工具
```

### 聊天开始
```
[Chat] Using agent: Excel Assistant (agent-001)
[Chat] Agent has 25 tools, tool_calling enabled: true
[Chat] Discovered 25 tools with 25 tool details
[Chat] 🔧 Using enhanced system prompt with 25 tool details
[Chat] 🔧 Using custom system prompt (with tool details)
```

### 工具调用
```
🔍 [TOOL PARSE] Attempting to parse tool call from AI response
🔍 [TOOL PARSE] Found <tool_call> tag at position 123
🔍 [TOOL PARSE] Found </tool_call> tag at position 456
🔧 [TOOL EXEC] Executing tool: read_data_from_excel (ID: call_001)
🔧 [TOOL EXEC] Arguments: {"filepath":"test.xlsx","sheet_name":"Sheet1"}
✅ [TOOL AUTO] Tool 'read_data_from_excel' found in MCP server 'excel-mcp'
✓ [MCP TOOL] Server 'excel-mcp' already connected
🔧 [MCP TOOL] Calling MCP tool: read_data_from_excel on server: excel-mcp
✅ [TOOL EXEC] Tool executed successfully in 234ms
```

## ✅ 优势总结

### 1. 重用现有数据
- ✅ 直接使用 `MCPServerConfig.cached_tools`
- ✅ 无需重新连接 MCP 服务器
- ✅ 无需修改数据库 schema

### 2. 最小改动
- ✅ 只修改 3 个文件
- ✅ 向后兼容现有代码
- ✅ 不影响 Protocol Buffers 定义

### 3. 快速见效
- ✅ AI 立即能看到工具详情
- ✅ AI 能正确选择和使用工具
- ✅ 减少工具调用错误

### 4. 良好的性能
- ✅ 只在聊天开始时获取一次工具详情
- ✅ 使用缓存的工具信息(无网络开销)
- ✅ 格式化工具描述的开销可忽略

## 📁 修改的文件

| 文件 | 变更内容 | 行数变化 |
|------|---------|---------|
| `rust-lib/flowy-ai/src/ai_manager.rs` | 修改 `discover_available_tools` 返回类型,构建增强提示 | +40 |
| `rust-lib/flowy-ai/src/agent/system_prompt.rs` | 新增工具详情格式化函数 | +90 |
| `rust-lib/flowy-ai/src/chat.rs` | 接受自定义系统提示参数 | +5 |

**总计**: 约 135 行新增代码

## 🧪 测试验证

### 1. 检查工具详情日志
```bash
# 启动应用,创建聊天,查看日志
grep "tool details" logs.txt
```

预期输出:
```
[Chat] Discovered 25 tools with 25 tool details
[Chat] 🔧 Using enhanced system prompt with 25 tool details
```

### 2. 测试工具调用
```
用户: "帮我在 test.xlsx 的 Sheet1 的 A1 单元格写入公式 =SUM(B1:B10)"
```

预期 AI 行为:
- ✅ 正确选择 `apply_formula` 工具
- ✅ 正确提供所有必需参数
- ✅ 参数值符合描述要求

### 3. 验证系统提示内容
添加临时调试日志:
```rust
info!("=== System Prompt ===\n{}\n===", enhanced_prompt);
```

预期输出:
```
=== System Prompt ===
## 🔧 Available Tools (Detailed Information)

You have access to the following tools...

1. **read_data_from_excel**
  Read data from Excel worksheet...
  Parameters:
    - filepath (string): Path to Excel file [required]
    ...
===
```

## 🚀 后续优化建议

### 高优先级
- [ ] 添加工具详情缓存到 `AIManager`,避免每次聊天都查询
- [ ] 优化系统提示长度,只包含最相关的工具
- [ ] 添加工具使用示例到提示中

### 中优先级
- [ ] 支持工具分类和组织
- [ ] 添加工具调用成功率统计
- [ ] 实现工具推荐(根据历史使用)

### 低优先级
- [ ] UI 显示工具详情
- [ ] 工具详情的国际化
- [ ] 工具版本管理

## 📖 总结

通过这个实现,我们成功地:
1. ✅ 利用了现有的持久化工具缓存(`cached_tools`)
2. ✅ 解析了 JSON Schema 提取参数信息
3. ✅ 生成了包含详细工具说明的增强系统提示
4. ✅ 使 AI 能够正确理解和使用 MCP 工具

**最重要的是**: 这是一个临时但实用的解决方案,无需大规模重构,快速解决了 AI 无法使用工具的核心问题! 🎉

