use crate::entities::AgentConfigPB;

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
        "- Task Planning: You can break down complex tasks into steps (max {} steps)\n",
        cap.max_planning_steps
      ));
    }
    
    if cap.enable_tool_calling && !config.available_tools.is_empty() {
      prompt.push_str(&format!(
        "- Tool Calling: You have access to the following tools: {:?}\n",
        config.available_tools
      ));
      prompt.push_str(&format!(
        "  (max {} tool calls per conversation)\n",
        cap.max_tool_calls
      ));
    }
    
    if cap.enable_reflection {
      prompt.push_str("- Self-Reflection: You should reflect on your responses and improve them\n");
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

