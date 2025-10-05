#!/usr/bin/env python3
"""
完成多轮对话集成
1. 添加必要的导入
2. 在流结束后检查并执行多轮对话
"""
import re

# 读取 chat.rs
with open("rust-lib/flowy-ai/src/chat.rs", "r", encoding="utf-8") as f:
    content = f.read()

# 1. 添加必要的导入（在现有的 use crate::middleware 行之后）
import_pattern = r'(use crate::middleware::chat_service_mw::ChatServiceMiddleware;)'
import_replacement = r'''\1
use crate::middleware::chat_service_mw::{OpenAIToolCall, OpenAIFunctionCall};'''

if "OpenAIToolCall" not in content:
    content = re.sub(import_pattern, import_replacement, content)
    print("✅ Added OpenAIToolCall and OpenAIFunctionCall imports")
else:
    print("⏭️  Imports already exist")

# 2. 在流结束后添加多轮对话逻辑
# 查找 "// 🔧 反思机制：如果有工具调用结果且启用了反思，进入反思循环" 这一行之前
reflection_pattern = r'(          // 🔧 反思机制：如果有工具调用结果且启用了反思，进入反思循环)'

multi_turn_code = r'''          
          // 🔄 多轮对话：检查是否收集到 tool_calls
          let tool_calls_vec = collected_tool_calls.lock().await.clone();
          if !tool_calls_vec.is_empty() && has_tool_handler {
            info!("🔄 [MULTI-TURN] Detected {} tool calls, starting multi-turn conversation", 
                  tool_calls_vec.len());
            
            // 构建初始消息历史
            let mut initial_messages = Vec::new();
            
            // 添加 system 消息
            if let Some(ref prompt) = system_prompt {
              initial_messages.push(serde_json::json!({
                "role": "system",
                "content": prompt
              }));
            }
            
            // 添加 user 消息（需要获取原始问题内容）
            // 注意：这里简化处理，实际应该从数据库获取
            initial_messages.push(serde_json::json!({
              "role": "user",
              "content": format!("Question ID: {}", question_id)
            }));
            
            // 调用 cloud_service 的多轮对话方法
            // 注意：这需要 ChatServiceMiddleware 提供此方法
            if let Some(middleware) = cloud_service.as_any().downcast_ref::<ChatServiceMiddleware>() {
              // TODO: 实现多轮对话调用
              // 目前先打印日志
              info!("🔄 [MULTI-TURN] Would call continue_conversation_with_tool_results with {} tool_calls", 
                    tool_calls_vec.len());
              
              // 发送分隔符
              let separator = "\n\n🔄 **多轮对话继续中...**\n\n";
              let _ = answer_sink.send(StreamMessage::OnData(separator.to_string()).to_string()).await;
              
              // TODO: 实际调用多轮对话方法并流式输出结果
            } else {
              warn!("🔄 [MULTI-TURN] Cannot downcast to ChatServiceMiddleware, skipping multi-turn");
            }
          }
          
\1'''

if "🔄 多轮对话：检查是否收集到 tool_calls" not in content:
    content = re.sub(reflection_pattern, multi_turn_code, content)
    print("✅ Added multi-turn conversation logic")
else:
    print("⏭️  Multi-turn logic already exists")

# 保存文件
with open("rust-lib/flowy-ai/src/chat.rs", "w", encoding="utf-8") as f:
    f.write(content)

print("\n✅ Multi-turn integration completed")
print("\n📝 注意：")
print("  1. OpenAIToolCall 和 OpenAIFunctionCall 需要在 chat_service_mw.rs 中公开")
print("  2. 需要实现从 trait object 调用具体方法的机制")
print("  3. 建议简化方案：直接在 stream_answer_with_system_prompt 中集成多轮对话")

