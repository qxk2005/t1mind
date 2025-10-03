# MCP 工具调用问题诊断 - 系统提示缺失工具详情

## 问题现象

虽然智能体配置了 25 个工具且工具调用已启用,但 AI 并没有生成任何工具调用请求。

```
Agent has 25 tools, tool_calling enabled: true
```

但在 AI 的回复中,没有看到任何 `<tool_call>` 标签。

## 根本原因

### 当前系统提示的问题

系统提示中只包含了**工具名称列表**,没有包含**工具的详细信息**:

```rust
// rust-lib/flowy-ai/src/agent/system_prompt.rs:42-46
if cap.enable_tool_calling && !config.available_tools.is_empty() {
    prompt.push_str("- Tool Calling: You can use external tools to accomplish tasks\n");
    prompt.push_str(&format!(
        "  Available tools: {}\n",
        config.available_tools.join(", ")  // ❌ 只是名称列表!
    ));
}
```

**实际效果:**
```
Available tools: read_data_from_excel, write_data_to_excel, apply_formula, ...
```

### AI 无法知道的关键信息

1. **工具的功能描述**: 每个工具是做什么的?
2. **使用场景**: 什么情况下应该使用这个工具?
3. **参数列表**: 需要传入哪些参数?
4. **参数类型**: 每个参数是什么类型?
5. **参数是否必需**: 哪些参数是必需的,哪些是可选的?

### 数据结构对比

**当前存储的数据 (AgentConfigPB):**
```rust
pub struct AgentConfigPB {
    pub available_tools: Vec<String>,  // ❌ 只存储工具名称!
    // ...
}
```

**实际需要的数据 (MCPTool):**
```rust
pub struct MCPTool {
    pub name: String,              // ✅ 工具名称
    pub description: String,        // ✅ 功能描述
    pub input_schema: Value,        // ✅ 参数 JSON Schema
    pub annotations: Option<...>,   // ✅ 附加信息
}
```

## 解决方案

### 方案 1: 在系统提示中包含工具详细信息 (推荐)

修改智能体配置,存储完整的工具定义:

```rust
// entities.rs
pub struct AgentConfigPB {
    pub available_tools: Vec<ToolDefinitionPB>,  // ✅ 存储完整定义
    // ...
}

pub struct ToolDefinitionPB {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameterPB>,
    pub source: String,  // MCP server ID
    pub tool_type: ToolTypePB,
}

pub struct ToolParameterPB {
    pub name: String,
    pub description: String,
    pub param_type: String,  // string, number, boolean, object, array
    pub required: bool,
    pub default_value: Option<String>,
}
```

修改系统提示生成:

```rust
// system_prompt.rs
if cap.enable_tool_calling && !config.available_tools.is_empty() {
    prompt.push_str("- Tool Calling: You have access to these tools:\n\n");
    
    for tool in &config.available_tools {
        prompt.push_str(&format!("  **{}**\n", tool.name));
        prompt.push_str(&format!("    Description: {}\n", tool.description));
        prompt.push_str("    Parameters:\n");
        
        for param in &tool.parameters {
            let required_mark = if param.required { "(required)" } else { "(optional)" };
            prompt.push_str(&format!(
                "      - {} {}: {} {}\n",
                param.name,
                param.param_type,
                param.description,
                required_mark
            ));
        }
        
        prompt.push_str(&format!("    Source: {}\n\n", tool.source));
    }
}
```

**生成的系统提示示例:**
```
- Tool Calling: You have access to these tools:

  **read_data_from_excel**
    Description: Read data from Excel worksheet with cell metadata including validation rules
    Parameters:
      - filepath string: Path to Excel file (required)
      - sheet_name string: Name of worksheet (required)
      - start_cell string: Starting cell (optional)
      - end_cell string: Ending cell (optional)
      - preview_only boolean: Whether to return preview only (optional)
    Source: excel-mcp

  **write_data_to_excel**
    Description: Write data to Excel worksheet
    Parameters:
      - filepath string: Path to Excel file (required)
      - sheet_name string: Name of worksheet (required)
      - data array: List of lists containing data (required)
      - start_cell string: Cell to start writing to (optional)
    Source: excel-mcp
```

### 方案 2: 实时查询工具信息 (备选)

在构建系统提示时,动态查询 MCP 服务器获取工具详情:

```rust
async fn build_tool_usage_guide(&self, config: &AgentConfigPB) -> String {
    let mut guide = String::from("\n\n# Available Tools\n\n");
    
    for tool_name in &config.available_tools {
        // 从 MCP 服务器查询工具详情
        if let Some((server_id, tool)) = self.mcp_manager.find_tool_by_name(tool_name).await {
            guide.push_str(&format!("**{}**\n", tool.name));
            guide.push_str(&format!("  {}\n", tool.description));
            
            // 解析 input_schema 获取参数信息
            if let Some(properties) = tool.input_schema.get("properties") {
                guide.push_str("  Parameters:\n");
                // ... 解析参数详情
            }
        }
    }
    
    guide
}
```

### 方案 3: 使用工具缓存 (折中方案)

在工具发现时缓存工具详情,在构建提示时使用缓存:

```rust
// config_manager.rs
pub struct AgentConfigPB {
    pub available_tools: Vec<String>,  // 工具名称
    pub tool_cache: HashMap<String, MCPTool>,  // 工具详情缓存
}
```

## 实现步骤 (方案 1)

### 1. 修改协议定义

```protobuf
// flowy-ai/entities.proto
message AgentConfigPB {
    repeated ToolDefinitionPB available_tools = 5;  // 改为完整定义
    // ...
}

message ToolDefinitionPB {
    string name = 1;
    string description = 2;
    repeated ToolParameterPB parameters = 3;
    string source = 4;  // MCP server ID
    ToolTypePB tool_type = 5;
}

message ToolParameterPB {
    string name = 1;
    string description = 2;
    string param_type = 3;  // string, number, boolean, object, array
    bool required = 4;
    optional string default_value = 5;
}

enum ToolTypePB {
    MCP = 0;
    NATIVE = 1;
}
```

### 2. 修改工具发现逻辑

```rust
// ai_manager.rs
async fn discover_available_tools(&self) -> Vec<ToolDefinitionPB> {
    let mut tool_definitions = Vec::new();
    
    // 从所有配置的 MCP 服务器获取工具
    let all_servers = self.mcp_manager.config_manager().get_all_servers();
    
    for server in all_servers {
        if let Some(cached_tools) = server.get_cached_tools() {
            for tool in cached_tools {
                // 解析 input_schema 转换为 ToolDefinitionPB
                let parameters = self.parse_tool_parameters(&tool.input_schema);
                
                tool_definitions.push(ToolDefinitionPB {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters,
                    source: server.id.clone(),
                    tool_type: ToolTypePB::MCP,
                });
            }
        }
    }
    
    tool_definitions
}

fn parse_tool_parameters(&self, input_schema: &Value) -> Vec<ToolParameterPB> {
    let mut parameters = Vec::new();
    
    if let Some(properties) = input_schema.get("properties").and_then(|p| p.as_object()) {
        let required_fields = input_schema.get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        
        for (name, schema) in properties {
            let param_type = schema.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("string");
            
            let description = schema.get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            
            let required = required_fields.contains(&name.as_str());
            
            parameters.push(ToolParameterPB {
                name: name.clone(),
                description,
                param_type: param_type.to_string(),
                required,
                default_value: None,
            });
        }
    }
    
    parameters
}
```

### 3. 修改系统提示生成

```rust
// system_prompt.rs
if cap.enable_tool_calling && !config.available_tools.is_empty() {
    prompt.push_str("- Tool Calling: You have access to these tools:\n\n");
    
    for (idx, tool) in config.available_tools.iter().enumerate() {
        prompt.push_str(&format!("{}. **{}**\n", idx + 1, tool.name));
        prompt.push_str(&format!("   {}\n", tool.description));
        
        if !tool.parameters.is_empty() {
            prompt.push_str("   Parameters:\n");
            for param in &tool.parameters {
                let req = if param.required { "required" } else { "optional" };
                prompt.push_str(&format!(
                    "     - {} ({}): {} [{}]\n",
                    param.name,
                    param.param_type,
                    param.description,
                    req
                ));
            }
        }
        prompt.push_str("\n");
    }
}
```

## 测试验证

### 1. 查看生成的系统提示

在日志中添加系统提示的输出:

```rust
// ai_manager.rs
info!("=== System Prompt ===\n{}\n=== End ===", system_prompt);
```

验证系统提示包含:
- ✅ 工具名称
- ✅ 工具描述
- ✅ 参数列表
- ✅ 参数类型
- ✅ 参数是否必需

### 2. 测试 AI 响应

向 AI 提问:
```
"帮我读取 test.xlsx 文件的 Sheet1 工作表的内容"
```

期待 AI 生成:
```xml
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

## 临时解决方案

在完整实现之前,可以手动在智能体的描述或个性中添加工具说明:

```
Description:
你是一个Excel数据助手。你可以使用read_data_from_excel工具读取Excel文件,
该工具需要filepath(文件路径)和sheet_name(工作表名称)两个参数。

示例:
用户: "读取test.xlsx的Sheet1"
你应该: 使用read_data_from_excel工具,参数为{"filepath": "test.xlsx", "sheet_name": "Sheet1"}
```

## 总结

**当前问题:**
- ❌ AI 不知道有哪些工具可用
- ❌ AI 不知道每个工具的作用
- ❌ AI 不知道如何使用工具(参数)

**解决关键:**
- ✅ 在系统提示中包含完整的工具定义
- ✅ 包含工具的描述和参数信息
- ✅ 提供清晰的使用示例

**优先级:**
1. **高优先级**: 修改系统提示包含工具详情
2. **中优先级**: 修改数据结构存储完整工具定义
3. **低优先级**: 优化工具信息的展示格式

这是 AI 无法调用工具的根本原因!

