use crate::entities::AgentConfigPB;
#[cfg(feature = "mcp")]
use crate::mcp::entities::MCPTool;
use std::collections::HashMap;

/// 根据智能体配置构建系统提示词
pub fn build_agent_system_prompt(config: &AgentConfigPB) -> String {
  let mut prompt = String::new();
  
  // 1. 基础描述
  if !config.description.is_empty() {
    prompt.push_str(&format!("# Agent Description\n{}\n\n", config.description));
  }
  
  // 2. 个性设置
  if !config.personality.is_empty() {
    prompt.push_str(&format!("# Personality\n{}\n\n", config.personality));
  }
  
  // 3. 能力说明
  let cap = &config.capabilities;
  if cap.enable_planning || cap.enable_tool_calling || cap.enable_reflection || cap.enable_memory {
    prompt.push_str("# Capabilities\n");
    
    if cap.enable_planning {
      prompt.push_str(&format!(
        "- Task Planning: Break down complex tasks systematically (max {} steps)\n",
        cap.max_planning_steps
      ));
      prompt.push_str("  **When to Create a Plan:**\n");
      prompt.push_str("    • Complex multi-step tasks\n");
      prompt.push_str("    • Tasks requiring multiple tools or resources\n");
      prompt.push_str("    • Tasks with dependencies between steps\n");
      prompt.push_str("    • Tasks that need careful sequencing\n\n");
      prompt.push_str("  **Planning Process:**\n");
      prompt.push_str("    1. Analyze the goal and identify key requirements\n");
      prompt.push_str("    2. Break down into logical, sequential steps\n");
      prompt.push_str("    3. Identify required tools and resources for each step\n");
      prompt.push_str("    4. Execute steps methodically, one at a time\n");
      prompt.push_str("    5. Validate results after each step\n");
      prompt.push_str("    6. Adjust plan if needed based on intermediate results\n");
      prompt.push_str("    7. Summarize final outcome for the user\n\n");
    }
    
    if cap.enable_tool_calling && !config.available_tools.is_empty() {
      // 🆕 使用 OpenAI Function Call API，无需手工说明调用格式
      // 工具定义通过 API 的 tools 参数传递，模型会自动按照标准格式返回 function calls
      prompt.push_str("- Tool Calling: You have access to external tools to help accomplish tasks\n");
      prompt.push_str(&format!(
        "  Available tools: {}\n",
        config.available_tools.join(", ")
      ));
      prompt.push_str(&format!(
        "  Max {} tool calls per conversation\n",
        cap.max_tool_calls
      ));
      prompt.push_str("\n  **Guidelines for using tools:**\n");
      prompt.push_str("    • Use tools when they can help accomplish the user's request\n");
      prompt.push_str("    • Explain to the user what tool you're using and why\n");
      prompt.push_str("    • Interpret and summarize tool results for the user\n");
      prompt.push_str("    • Handle errors gracefully with helpful messages\n");
      prompt.push_str("    • Consider using multiple tools if needed to complete the task\n\n");
    }
    
    if cap.enable_reflection {
      prompt.push_str("- Self-Reflection: Review and improve your responses continuously\n");
      prompt.push_str("  After generating responses:\n");
      prompt.push_str("    • Check for accuracy and completeness\n");
      prompt.push_str("    • Consider alternative approaches\n");
      prompt.push_str("    • Identify potential improvements\n");
      prompt.push_str("    • Be transparent about uncertainties\n");
    }
    
    if cap.enable_memory {
      prompt.push_str(&format!(
        "- Conversation Memory: Remember the last {} messages in the conversation\n",
        cap.memory_limit
      ));
    }
    
    prompt.push_str("\n");
  }
  
  // 4. 额外元数据
  if !config.metadata.is_empty() {
    prompt.push_str("# Additional Information\n");
    for (key, value) in &config.metadata {
      prompt.push_str(&format!("- {}: {}\n", key, value));
    }
    prompt.push_str("\n");
  }
  
  // 5. 总结
  prompt.push_str(&format!(
    "Please act according to the above description and capabilities as the agent \"{}\".",
    config.name
  ));
  
  prompt
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::entities::{AgentCapabilitiesPB, AgentStatusPB};
  use std::collections::HashMap;
  
  #[test]
  fn test_build_system_prompt_basic() {
    let config = AgentConfigPB {
      id: "test-1".to_string(),
      name: "Test Agent".to_string(),
      description: "A helpful coding assistant".to_string(),
      personality: "Friendly and professional".to_string(),
      capabilities: AgentCapabilitiesPB {
        enable_planning: true,
        enable_tool_calling: true,
        enable_reflection: true,
        enable_memory: true,
        max_planning_steps: 10,
        max_tool_calls: 20,
        memory_limit: 100,
        max_tool_result_length: 4000,
      },
      available_tools: vec!["calculator".to_string(), "search".to_string()],
      status: AgentStatusPB::AgentActive,
      avatar: String::new(),
      created_at: 0,
      updated_at: 0,
      metadata: HashMap::new(),
    };
    
    let prompt = build_agent_system_prompt(&config);
    
    assert!(prompt.contains("# Agent Description"));
    assert!(prompt.contains("A helpful coding assistant"));
    assert!(prompt.contains("# Personality"));
    assert!(prompt.contains("Friendly and professional"));
    assert!(prompt.contains("# Capabilities"));
    assert!(prompt.contains("Task Planning"));
    assert!(prompt.contains("Tool Calling"));
    assert!(prompt.contains("Self-Reflection"));
    assert!(prompt.contains("Conversation Memory"));
    assert!(prompt.contains("Test Agent"));
  }
  
  #[test]
  fn test_build_system_prompt_minimal() {
    let config = AgentConfigPB {
      id: "minimal-1".to_string(),
      name: "Minimal Agent".to_string(),
      description: String::new(),
      personality: String::new(),
      capabilities: AgentCapabilitiesPB {
        enable_planning: false,
        enable_tool_calling: false,
        enable_reflection: false,
        enable_memory: false,
        max_planning_steps: 0,
        max_tool_calls: 0,
        memory_limit: 0,
        max_tool_result_length: 0,
      },
      available_tools: vec![],
      status: AgentStatusPB::AgentActive,
      avatar: String::new(),
      created_at: 0,
      updated_at: 0,
      metadata: HashMap::new(),
    };
    
    let prompt = build_agent_system_prompt(&config);
    
    assert!(prompt.contains("Minimal Agent"));
    assert!(!prompt.contains("# Agent Description"));
    assert!(!prompt.contains("# Personality"));
  }
  
  #[test]
  fn test_build_system_prompt_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("domain".to_string(), "software engineering".to_string());
    metadata.insert("language".to_string(), "Rust".to_string());
    
    let config = AgentConfigPB {
      id: "meta-1".to_string(),
      name: "Meta Agent".to_string(),
      description: "Test".to_string(),
      personality: String::new(),
      capabilities: AgentCapabilitiesPB::default(),
      available_tools: vec![],
      status: AgentStatusPB::AgentActive,
      avatar: String::new(),
      created_at: 0,
      updated_at: 0,
      metadata,
    };
    
    let prompt = build_agent_system_prompt(&config);
    
    assert!(prompt.contains("# Additional Information"));
    assert!(prompt.contains("domain"));
    assert!(prompt.contains("software engineering"));
  }
}

/// 格式化单个工具的详细信息
#[cfg(feature = "mcp")]
fn format_tool_details(tool: &MCPTool) -> String {
  let mut details = String::new();
  
  details.push_str(&format!("**{}**\n", tool.name));
  details.push_str(&format!("  {}\n", tool.description.as_deref().unwrap_or("No description available")));
  
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
      
      // 基本参数描述
      details.push_str(&format!(
        "    - {} ({}): {} [{}]\n",
        name, param_type, description, required_mark
      ));
      
      // 对于数组类型，展示数组元素的结构
      if param_type == "array" {
        if let Some(items) = schema.get("items") {
          if let Some(item_type) = items.get("type").and_then(|t| t.as_str()) {
            if item_type == "object" {
              // 数组元素是对象，展示对象的属性
              if let Some(item_props) = items.get("properties").and_then(|p| p.as_object()) {
                details.push_str(&format!("      Array items must be objects with:\n"));
                for (prop_name, prop_schema) in item_props {
                  let prop_type = prop_schema.get("type").and_then(|t| t.as_str()).unwrap_or("any");
                  let prop_desc = prop_schema.get("description").and_then(|d| d.as_str()).unwrap_or("");
                  
                  // 检查枚举值
                  let enum_hint = if let Some(enum_vals) = prop_schema.get("enum").and_then(|e| e.as_array()) {
                    let vals: Vec<String> = enum_vals.iter()
                      .filter_map(|v| v.as_str().map(|s| format!("\"{}\"", s)))
                      .collect();
                    if !vals.is_empty() {
                      format!(" (enum: {})", vals.join(", "))
                    } else {
                      String::new()
                    }
                  } else {
                    String::new()
                  };
                  
                  details.push_str(&format!(
                    "        • {} ({}){}: {}\n",
                    prop_name, prop_type, enum_hint, prop_desc
                  ));
                }
              }
            } else {
              details.push_str(&format!("      Array of: {}\n", item_type));
            }
          }
        }
      }
      
      // 展示枚举值
      if let Some(enum_vals) = schema.get("enum").and_then(|e| e.as_array()) {
        let vals: Vec<String> = enum_vals.iter()
          .filter_map(|v| v.as_str().map(|s| s.to_string()))
          .collect();
        if !vals.is_empty() {
          details.push_str(&format!("      Allowed values: {}\n", vals.join(", ")));
        }
      }
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

#[cfg(not(feature = "mcp"))]
fn format_tool_details(_tool: &()) -> String {
  "MCP support is disabled".to_string()
}

/// 构建包含工具详细信息的增强系统提示
#[cfg(feature = "mcp")]
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

#[cfg(not(feature = "mcp"))]
pub fn build_agent_system_prompt_with_tools(
  config: &AgentConfigPB,
  _tool_details: &HashMap<String, ()>,
) -> String {
  build_agent_system_prompt(config)
}

