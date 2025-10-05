#!/usr/bin/env python3
"""
集成自动多轮对话到 chat.rs
"""
import re

# 读取 chat.rs
with open("rust-lib/flowy-ai/src/chat.rs", "r", encoding="utf-8") as f:
    content = f.read()

# 查找并替换调用位置
old_call = r'''      \};
      
      match cloud_service
        \.stream_answer_with_system_prompt\(&workspace_id, &chat_id, question_id, format\.clone\(\), ai_model\.clone\(\), system_prompt\.clone\(\), tool_definitions\.clone\(\)\)
        \.await
      \{'''

new_call = r'''      };
      
      // 🔄 使用自动多轮对话（如果启用了工具调用）
      let stream_result = if has_agent && has_tool_handler && tool_definitions.is_some() {
        info!("🔄 [AUTO-MULTI-TURN] Using auto multi-turn conversation with {} tools", 
              tool_definitions.as_ref().unwrap().len());
        
        cloud_service
          .stream_answer_with_auto_multi_turn(
            &workspace_id, 
            &chat_id, 
            question_id, 
            format.clone(), 
            ai_model.clone(), 
            system_prompt.clone(), 
            tool_definitions.clone(),
            tool_call_handler.clone(),
            agent_config.clone().map(Arc::new),
            5  // 最大迭代次数
          )
          .await
      } else {
        // 使用普通流式响应（无工具或未启用）
        cloud_service
          .stream_answer_with_system_prompt(
            &workspace_id, 
            &chat_id, 
            question_id, 
            format.clone(), 
            ai_model.clone(), 
            system_prompt.clone(), 
            tool_definitions.clone()
          )
          .await
      };
      
      match stream_result {'''

if re.search(old_call, content):
    content = re.sub(old_call, new_call, content)
    print("✅ 替换成功")
else:
    print("❌ 未找到匹配的调用位置")
    print("\n尝试查找相关代码...")
    # 查找相关代码
    pattern = r'match cloud_service\s*\.stream_answer_with_system_prompt'
    matches = list(re.finditer(pattern, content))
    print(f"找到 {len(matches)} 处调用")
    
    # 显示前后文
    for i, match in enumerate(matches):
        start = max(0, match.start() - 200)
        end = min(len(content), match.end() + 200)
        print(f"\n=== 调用 {i+1} ===")
        print(content[start:end])

# 保存文件
with open("rust-lib/flowy-ai/src/chat.rs", "w", encoding="utf-8") as f:
    f.write(content)

print("\n✅ chat.rs 已更新")


