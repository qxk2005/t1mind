#!/usr/bin/env python3
"""
Fix compilation errors in chat.rs
"""
import re

with open('rust-lib/flowy-ai/src/chat.rs', 'r', encoding='utf-8') as f:
    content = f.read()

print("Fixing chat.rs compilation errors...")

# Fix 1: Add tool_definitions parameter to stream_chat_message signature
# Find the exact location
pattern = r'(pub async fn stream_chat_message\(\s+&self,\s+params: &StreamMessageParams,\s+preferred_ai_model: AIModel,\s+agent_config: Option<AgentConfigPB>,\s+tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,  // 🔧 工具调用处理器\s+custom_system_prompt: Option<String>,  // 🆕 自定义系统提示\(已包含工具详情\)\s+execution_logs: Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>,  // 📝 执行日志存储)\s+\) -> Result<ChatMessagePB, FlowyError>'

replacement = r'\1\n    tool_definitions: Option<Vec<ToolDefinitionPB>>,  // 🆕 工具定义列表（用于 OpenAI Function Call）\n  ) -> Result<ChatMessagePB, FlowyError>'

content = re.sub(pattern, replacement, content, flags=re.MULTILINE)

# Fix 2: Find and fix stream_response call in stream_chat_message
# This should be around line 180
pattern2 = r'(self\.stream_response\(\s+params\.answer_stream_port,\s+answer_stream_buffer,\s+uid,\s+workspace_id,\s+question\.message_id,\s+format,\s+preferred_ai_model,\s+system_prompt,\s+agent_config,  // 🔧 传递智能体配置\s+tool_call_handler,  // 🔧 传递工具调用处理器\s+execution_logs,  // 📝 传递执行日志存储)\s+\);'

replacement2 = r'\1\n      tool_definitions,  // 🆕 传递工具定义列表\n    );'

content = re.sub(pattern2, replacement2, content, flags=re.MULTILINE)

# Fix 3: Update stream_response signature
pattern3 = r'(fn stream_response\(\s+&self,\s+answer_stream_port: i64,\s+answer_stream_buffer: Arc<Mutex<StringBuffer>>,\s+_uid: i64,\s+workspace_id: Uuid,\s+question_id: i64,\s+format: ResponseFormat,\s+ai_model: AIModel,\s+system_prompt: Option<String>,\s+agent_config: Option<AgentConfigPB>,\s+tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,  // 🔧 新增工具调用处理器\s+execution_logs: Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>,  // 📝 执行日志存储)\s+\) \{'

replacement3 = r'\1\n    tool_definitions: Option<Vec<ToolDefinitionPB>>,  // 🆕 工具定义列表\n  ) {'

content = re.sub(pattern3, replacement3, content, flags=re.MULTILINE)

# Fix 4: Update stream_regenerate_response call
pattern4 = r'(self\.stream_response\(\s+answer_stream_port,\s+answer_stream_buffer,\s+uid,\s+workspace_id,\s+question_id,\s+format,\s+ai_model,\s+None, // 重新生成时不使用系统提示词\s+None, // 🔧 重新生成时不使用智能体配置\s+None, // 🔧 重新生成时不使用工具调用处理器\s+None, // 📝 重新生成时不使用执行日志)\s+\);'

replacement4 = r'\1\n      None, // 🆕 重新生成时不使用工具定义\n    );'

content = re.sub(pattern4, replacement4, content, flags=re.MULTILINE)

# Fix 5: Add None for tool_definitions in reflection flow (around line 539)
# This is the call that's missing the tools parameter
content = re.sub(
    r'(\.stream_answer_with_system_prompt\(\s+&workspace_id,\s+&chat_id,\s+question_id,\s+format\.clone\(\),\s+ai_model\.clone\(\),\s+Some\(follow_up_system_prompt\))\s+\)',
    r'\1,\n                  None  // 反思流程不传递工具定义\n                )',
    content,
    flags=re.MULTILINE
)

with open('rust-lib/flowy-ai/src/chat.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("✅ Fixed chat.rs compilation errors")

# Now fix agent_manager.rs
print("\nFixing agent_manager.rs...")

with open('rust-lib/flowy-ai/src/agent/agent_manager.rs', 'r', encoding='utf-8') as f:
    agent_content = f.read()

# Check if get_all_available_tools exists, if not we need a different approach
if 'get_all_available_tools' not in agent_content:
    print("  get_all_available_tools not found in agent_manager")
    print("  Using alternative: get tools from tool_registry")
    
    # Update ai_manager.rs to use a different approach
    with open('rust-lib/flowy-ai/src/ai_manager.rs', 'r', encoding='utf-8') as f:
        ai_mgr_content = f.read()
    
    # Replace get_tool_definitions_by_names implementation
    ai_mgr_content = re.sub(
        r'pub async fn get_tool_definitions_by_names\(&self, tool_names: &\[String\]\) -> Vec<ToolDefinitionPB> \{\s+let all_tools = self\.agent_manager\.get_all_available_tools\(\)\.await;\s+all_tools\.into_iter\(\)\s+\.filter\(\|tool\| tool_names\.contains\(&tool\.name\)\)\s+\.collect\(\)\s+\}',
        '''pub async fn get_tool_definitions_by_names(&self, tool_names: &[String]) -> Vec<ToolDefinitionPB> {
    let mut result = Vec::new();
    
    // Get tools from MCP servers
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
  }''',
        ai_mgr_content,
        flags=re.MULTILINE | re.DOTALL
    )
    
    with open('rust-lib/flowy-ai/src/ai_manager.rs', 'w', encoding='utf-8') as f:
        f.write(ai_mgr_content)
    
    print("✅ Fixed ai_manager.rs to use discover_available_tools")

print("\nAll fixes applied!")


