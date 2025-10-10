#!/usr/bin/env python3
"""
Fix ai_manager.rs get_tool_definitions_by_names implementation
"""
import re

with open('rust-lib/flowy-ai/src/ai_manager.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Find and replace the get_tool_definitions_by_names implementation
old_impl = r'''  pub async fn get_tool_definitions_by_names\(&self, tool_names: &\[String\]\) -> Vec<ToolDefinitionPB> \{
    let all_tools = self\.agent_manager\.get_all_available_tools\(\)\.await;
    all_tools\.into_iter\(\)
      \.filter\(\|tool\| tool_names\.contains\(&tool\.name\)\)
      \.collect\(\)
  \}'''

new_impl = '''  pub async fn get_tool_definitions_by_names(&self, tool_names: &[String]) -> Vec<ToolDefinitionPB> {
    let mut result = Vec::new();
    
    // Get tools from MCP servers using discover_available_tools
    let (_, tool_details) = self.discover_available_tools().await;
    
    for tool_name in tool_names {
      if let Some(mcp_tool) = tool_details.get(tool_name) {
        // Convert MCP tool to ToolDefinitionPB
        let tool_def = ToolDefinitionPB {
          name: mcp_tool.name.clone(),
          description: mcp_tool.description.clone().unwrap_or_default(),
          tool_type: crate::entities::ToolTypePB::MCP,
          source: "mcp".to_string(),
          parameters_schema: serde_json::to_string(&mcp_tool.input_schema).unwrap_or_default(),
          permissions: Vec::new(),
          is_available: true,
          metadata: std::collections::HashMap::new(),
        };
        result.push(tool_def);
      }
    }
    
    result
  }'''

content = re.sub(old_impl, new_impl, content, flags=re.MULTILINE | re.DOTALL)

with open('rust-lib/flowy-ai/src/ai_manager.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("✅ Fixed ai_manager.rs get_tool_definitions_by_names implementation")


