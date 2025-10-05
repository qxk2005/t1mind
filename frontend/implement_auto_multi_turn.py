#!/usr/bin/env python3
"""
实现自动多轮对话包装方法
在 chat_service_mw.rs 中添加一个智能包装方法
"""

auto_multi_turn_method = '''
  /// 🔄 自动多轮对话包装方法
  /// 
  /// 此方法会：
  /// 1. 循环调用 OpenAI API
  /// 2. 检测每轮响应中的 tool_calls
  /// 3. 自动执行工具并继续对话
  /// 4. 返回合并后的流式响应
  /// 
  /// 对上层完全透明，使用方式与普通流式响应完全一致。
  pub async fn stream_answer_with_auto_multi_turn(
    &self,
    workspace_id: &Uuid,
    chat_id: &Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
    system_prompt: Option<String>,
    tools: Option<Vec<ToolDefinitionPB>>,
    tool_handler: Option<Arc<crate::agent::ToolCallHandler>>,
    agent_config: Option<&crate::entities::AgentConfigPB>,
    max_iterations: usize,
  ) -> Result<StreamAnswer, FlowyError> {
    // 如果没有工具或工具处理器，使用普通模式
    if tools.is_none() || tools.as_ref().unwrap().is_empty() || tool_handler.is_none() {
      return self.stream_answer_with_system_prompt(
        workspace_id, chat_id, question_id, format, ai_model, system_prompt, tools
      ).await;
    }
    
    info!("🔄 [AUTO-MULTI-TURN] Starting auto multi-turn with {} tools", 
          tools.as_ref().unwrap().len());
    
    // 获取 OpenAI 配置
    let cfg = self.read_openai_compat_chat_config(workspace_id)
      .ok_or_else(|| FlowyError::internal().with_context("No OpenAI config found"))?;
    
    let model_name = ai_model.name.as_str();
    let content = self.get_message_content(question_id)?;
    let tools_clone = tools.clone();
    let tool_handler = tool_handler.unwrap();
    
    // 构建初始消息
    let mut messages = Vec::new();
    if let Some(ref prompt) = system_prompt {
      messages.push(json!({
        "role": "system",
        "content": prompt
      }));
    }
    messages.push(json!({
      "role": "user",
      "content": content
    }));
    
    // 创建多轮对话流
    let s = try_stream! {
      let mut current_messages = messages.clone();
      let mut iteration = 0;
      
      loop {
        iteration += 1;
        if iteration > max_iterations {
          warn!("🔄 [AUTO-MULTI-TURN] Max iterations ({}) reached", max_iterations);
          yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
            value: format!("\\n\\n⚠️ 已达到最大对话轮次 ({})\\n", max_iterations)
          };
          break;
        }
        
        info!("🔄 [AUTO-MULTI-TURN] Starting iteration {}/{}", iteration, max_iterations);
        
        // 调用 OpenAI API
        let url = Self::join_openai_url(&cfg.base_url, "/chat/completions");
        let mut payload = json!({
          "model": model_name,
          "messages": current_messages,
          "stream": true
        });
        
        // 添加工具定义
        if let Some(ref tools) = tools_clone {
          if !tools.is_empty() {
            let openai_tools = Self::convert_tools_to_openai_format(tools);
            payload.as_object_mut().unwrap().insert("tools".into(), json!(openai_tools));
            payload.as_object_mut().unwrap().insert("tool_choice".into(), json!("auto"));
          }
        }
        
        // 添加可选参数
        if let Some(temp) = cfg.temperature {
          payload.as_object_mut().unwrap().insert("temperature".into(), json!(temp));
        }
        if let Some(max_tok) = cfg.max_tokens {
          payload.as_object_mut().unwrap().insert("max_tokens".into(), json!(max_tok));
        }
        
        // 发送请求
        let client = reqwest::Client::new();
        let resp = client
          .post(&url)
          .header("Content-Type", "application/json")
          .header("Authorization", format!("Bearer {}", cfg.api_key))
          .header("Accept", "text/event-stream")
          .json(&payload)
          .send()
          .await
          .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
        
        if !resp.status().is_success() {
          return Err(FlowyError::server_error()
            .with_context(format!("Request error: {}", resp.status())));
        }
        
        // 解析流式响应并收集 tool_calls
        let accumulated_tool_call = Arc::new(tokio::sync::Mutex::new(Option::<OpenAIToolCall>::None));
        let has_content_flag = Arc::new(tokio::sync::Mutex::new(false));
        let tc_clone = accumulated_tool_call.clone();
        let content_flag_clone = has_content_flag.clone();
        
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
          let bytes = chunk.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
          let s = String::from_utf8_lossy(&bytes);
          
          for line in s.lines() {
            let l = line.trim_start();
            if !l.starts_with("data:") { continue; }
            let data = l.trim_start_matches("data:").trim();
            if data == "[DONE]" { break; }
            
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
              if let Some(delta) = v.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
                // 检查 tool_calls
                if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                  for tc_delta in tool_calls_arr {
                    let index = tc_delta.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                    if index == 0 {
                      let mut tc_guard = tc_clone.lock().await;
                      if tc_guard.is_none() {
                        *tc_guard = Some(OpenAIToolCall {
                          id: tc_delta.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                          tool_type: "function".to_string(),
                          function: OpenAIFunctionCall {
                            name: String::new(),
                            arguments: String::new(),
                          },
                        });
                      }
                      if let Some(ref mut tc) = *tc_guard {
                        if let Some(id) = tc_delta.get("id").and_then(|s| s.as_str()) {
                          tc.id = id.to_string();
                        }
                        if let Some(func) = tc_delta.get("function") {
                          if let Some(name) = func.get("name").and_then(|s| s.as_str()) {
                            tc.function.name.push_str(name);
                          }
                          if let Some(args) = func.get("arguments").and_then(|s| s.as_str()) {
                            tc.function.arguments.push_str(args);
                          }
                        }
                      }
                    }
                  }
                }
                
                // 检查普通内容并转发
                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                  if !content.is_empty() {
                    *content_flag_clone.lock().await = true;
                    yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
                      value: content.to_string()
                    };
                  }
                }
              }
            }
          }
        }
        
        // 检查是否收集到 tool_call
        let tool_call_opt = accumulated_tool_call.lock().await.take();
        let has_content = *has_content_flag.lock().await;
        
        if let Some(tc) = tool_call_opt {
          if !tc.function.name.is_empty() {
            info!("🔄 [AUTO-MULTI-TURN] Detected tool_call: {}", tc.function.name);
            
            // 添加 assistant 消息
            current_messages.push(json!({
              "role": "assistant",
              "content": null,
              "tool_calls": [{
                "id": tc.id,
                "type": "function",
                "function": {
                  "name": tc.function.name,
                  "arguments": tc.function.arguments
                }
              }]
            }));
            
            // 发送工具执行提示
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: format!("\\n\\n🔧 **正在执行工具: {}...**\\n\\n", tc.function.name)
            };
            
            // 执行工具
            let arguments: serde_json::Value = serde_json::from_str(&tc.function.arguments)
              .unwrap_or_else(|_| json!({}));
            
            let request = crate::agent::ToolCallRequest {
              id: tc.id.clone(),
              tool_name: tc.function.name.clone(),
              arguments,
              source: None,
            };
            
            let response = tool_handler.execute_tool_call(&request, agent_config).await;
            
            let result_content = if response.success {
              response.result.unwrap_or_else(|| "Success".to_string())
            } else {
              format!("Error: {}", response.error.unwrap_or_else(|| "Unknown".to_string()))
            };
            
            // 发送执行结果提示
            let status_icon = if response.success { "✅" } else { "❌" };
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: format!("{} {} 完成 ({}ms)\\n", status_icon, tc.function.name, response.duration_ms)
            };
            
            // 添加工具结果到消息历史
            current_messages.push(json!({
              "role": "tool",
              "tool_call_id": tc.id,
              "name": tc.function.name,
              "content": result_content
            }));
            
            // 发送分析提示
            yield flowy_ai_pub::cloud::QuestionStreamValue::Answer {
              value: "\\n🤔 **正在分析结果...**\\n\\n".to_string()
            };
            
            // 继续下一轮循环
            continue;
          }
        }
        
        // 没有 tool_calls 或有内容输出，结束循环
        if has_content || tool_call_opt.is_none() {
          info!("🔄 [AUTO-MULTI-TURN] Conversation completed after {} iterations", iteration);
          break;
        }
      }
    };
    
    Ok(Box::pin(s))
  }
'''

print("=" * 80)
print("自动多轮对话方法生成完成")
print("=" * 80)
print()
print("📝 需要添加到 chat_service_mw.rs 的方法：")
print()
print(auto_multi_turn_method)
print()
print("=" * 80)
print("✅ 方法生成完成")
print()
print("下一步：")
print("1. 将上述方法添加到 ChatServiceMiddleware impl 块中")
print("2. 在 ai_manager.rs 中调用此方法（需要传入 tool_handler）")
print("3. 测试自动多轮对话功能")
print("=" * 80)

