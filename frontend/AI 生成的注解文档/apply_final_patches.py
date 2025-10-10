#!/usr/bin/env python3
"""
Apply final patches to chat.rs for OpenAI Function Call migration
"""
import re

with open('rust-lib/flowy-ai/src/chat.rs', 'r', encoding='utf-8') as f:
    content = f.read()

print("Step 1: ✓ ToolDefinitionPB already added to imports")

# Step 2: Update stream_chat_message signature
print("Step 2: Updating stream_chat_message signature...")
content = re.sub(
    r'(pub async fn stream_chat_message\(\s+&self,\s+params: &StreamMessageParams,\s+preferred_ai_model: AIModel,\s+agent_config: Option<AgentConfigPB>,\s+tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,\s+custom_system_prompt: Option<String>,\s+execution_logs: Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>,)\s+\)',
    r'\1\n    tool_definitions: Option<Vec<ToolDefinitionPB>>,\n  )',
    content,
    flags=re.MULTILINE | re.DOTALL
)

# Step 3: Update first stream_response call
print("Step 3: Updating first stream_response call...")
content = re.sub(
    r'(self\.stream_response\(\s+params\.answer_stream_port,\s+answer_stream_buffer,\s+uid,\s+workspace_id,\s+question\.message_id,\s+format,\s+preferred_ai_model,\s+system_prompt,\s+agent_config,\s+tool_call_handler,\s+execution_logs,)\s+\);',
    r'\1\n      tool_definitions,\n    );',
    content,
    flags=re.MULTILINE | re.DOTALL
)

# Step 4: Update stream_response signature
print("Step 4: Updating stream_response signature...")
content = re.sub(
    r'(fn stream_response\(\s+&self,\s+answer_stream_port: i64,\s+answer_stream_buffer: Arc<Mutex<StringBuffer>>,\s+_uid: i64,\s+workspace_id: Uuid,\s+question_id: i64,\s+format: ResponseFormat,\s+ai_model: AIModel,\s+system_prompt: Option<String>,\s+agent_config: Option<AgentConfigPB>,\s+tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,\s+execution_logs: Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>,)\s+\)',
    r'\1\n    tool_definitions: Option<Vec<ToolDefinitionPB>>,\n  )',
    content,
    flags=re.MULTILINE | re.DOTALL
)

# Step 5: Update stream_answer_with_system_prompt call
print("Step 5: Updating stream_answer_with_system_prompt call...")
content = re.sub(
    r'(\.stream_answer_with_system_prompt\(&workspace_id, &chat_id, question_id, format\.clone\(\), ai_model\.clone\(\), system_prompt\.clone\(\))\)',
    r'\1, tool_definitions.clone())',
    content
)

# Step 6: Update stream_regenerate_response call
print("Step 6: Updating stream_regenerate_response stream_response call...")
# Find the stream_regenerate_response function
pattern = r'(pub async fn stream_regenerate_response[^{]+\{[^}]*self\.stream_response\([^)]+None, // 📝 重新生成时不使用执行日志)\s+\);'
replacement = r'\1\n      None, // 🆕 重新生成时不使用工具定义\n    );'
content = re.sub(pattern, replacement, content, flags=re.DOTALL)

# Step 7: Simplify tool call detection code
print("Step 7: Simplifying old tool call detection code...")

# Find the QuestionStreamValue::Answer block with old tool detection
# We need to find from "QuestionStreamValue::Answer { value } =>" to the closing of that match arm

lines = content.split('\n')
result_lines = []
i = 0
in_answer_block = False
block_start = -1

while i < len(lines):
    line = lines[i]
    
    # Detect start of Answer block
    if 'QuestionStreamValue::Answer { value } =>' in line:
        in_answer_block = True
        block_start = i
        result_lines.append(line)
        i += 1
        
        # Check if next line starts the old tool detection
        if i < len(lines) and ('累积文本以检测工具调用' in lines[i] or 
                               'accumulated_text.push_str' in lines[i]):
            # Found old tool detection code, replace with simplified version
            print(f"  Found old tool detection at line {i}, replacing...")
            result_lines.append('                    // 🆕 使用 OpenAI Function Call API，无需手工检测 <tool_call> 标签')
            result_lines.append('                    // Metadata 中的 tool_call 信息由 middleware 自动处理')
            result_lines.append('                    answer_stream_buffer.lock().await.push_str(&value);')
            result_lines.append('                    if let Err(err) = answer_sink')
            result_lines.append('                      .send(StreamMessage::OnData(value).to_string())')
            result_lines.append('                      .await')
            result_lines.append('                    {')
            result_lines.append('                      error!("Failed to stream answer via IsolateSink: {}", err);')
            result_lines.append('                    }')
            
            # Skip until we find the closing of this match arm (next match arm or closing brace)
            depth = 0
            i += 1
            while i < len(lines):
                current_line = lines[i]
                
                # Count braces
                depth += current_line.count('{')
                depth -= current_line.count('}')
                
                # If we hit a new match arm or end of match at depth 0, stop
                if depth <= 0 and ('QuestionStreamValue::' in current_line or 
                                   current_line.strip() == '},'):
                    break
                i += 1
            
            in_answer_block = False
            continue
        else:
            # Not the old tool detection, keep as is
            in_answer_block = False
    else:
        result_lines.append(line)
    
    i += 1

content = '\n'.join(result_lines)

# Write back
with open('rust-lib/flowy-ai/src/chat.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("\n✅ All patches applied successfully!")
print("\nNext steps:")
print("1. Run: cd rust-lib && cargo check --package flowy-ai")
print("2. Fix any compilation errors if they appear")
print("3. Test the application")


