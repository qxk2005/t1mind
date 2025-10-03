use crate::entities::{
  AgentConfigPB, ChatMessageErrorPB, ChatMessageListPB, ChatMessagePB, PredefinedFormatPB,
  RepeatedRelatedQuestionPB, StreamMessageParams,
};
use crate::middleware::chat_service_mw::ChatServiceMiddleware;
use crate::notification::{ChatNotification, chat_notification_builder};
use tracing::info;
use crate::stream_message::{AIFollowUpData, StreamMessage};
use allo_isolate::Isolate;
use flowy_ai_pub::cloud::{
  AIModel, ChatCloudService, ChatMessage, MessageCursor, QuestionStreamValue, ResponseFormat,
};
use flowy_ai_pub::persistence::{
  ChatMessageTable, select_answer_where_match_reply_message_id, select_chat_messages,
  upsert_chat_messages,
};
use flowy_ai_pub::user_service::AIUserService;
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite::DBConnection;
use futures::{SinkExt, StreamExt};
use lib_infra::isolate_stream::IsolateSink;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, instrument, trace, warn};
use uuid::Uuid;

enum PrevMessageState {
  HasMore,
  NoMore,
  Loading,
}

pub struct Chat {
  chat_id: Uuid,
  uid: i64,
  user_service: Arc<dyn AIUserService>,
  chat_service: Arc<ChatServiceMiddleware>,
  prev_message_state: Arc<RwLock<PrevMessageState>>,
  latest_message_id: Arc<AtomicI64>,
  stop_stream: Arc<AtomicBool>,
  stream_buffer: Arc<Mutex<StringBuffer>>,
}

impl Chat {
  pub fn new(
    uid: i64,
    chat_id: Uuid,
    user_service: Arc<dyn AIUserService>,
    chat_service: Arc<ChatServiceMiddleware>,
  ) -> Chat {
    Chat {
      uid,
      chat_id,
      chat_service,
      user_service,
      prev_message_state: Arc::new(RwLock::new(PrevMessageState::HasMore)),
      latest_message_id: Default::default(),
      stop_stream: Arc::new(AtomicBool::new(false)),
      stream_buffer: Arc::new(Mutex::new(StringBuffer::default())),
    }
  }

  pub fn close(&self) {}

  pub async fn stop_stream_message(&self) {
    self
      .stop_stream
      .store(true, std::sync::atomic::Ordering::SeqCst);
  }

  #[instrument(level = "info", skip_all, err)]
  pub async fn stream_chat_message(
    &self,
    params: &StreamMessageParams,
    preferred_ai_model: AIModel,
    agent_config: Option<AgentConfigPB>,
    tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,  // 🔧 工具调用处理器
    custom_system_prompt: Option<String>,  // 🆕 自定义系统提示(已包含工具详情)
  ) -> Result<ChatMessagePB, FlowyError> {
    let agent_name = agent_config.as_ref().map(|c| c.name.as_str()).unwrap_or("None");
    trace!(
      "[Chat] stream chat message: chat_id={}, message={}, message_type={:?}, format={:?}, agent={}",
      self.chat_id, params.message, params.message_type, params.format, agent_name,
    );

    // clear
    self
      .stop_stream
      .store(false, std::sync::atomic::Ordering::SeqCst);
    self.stream_buffer.lock().await.clear();

    let mut question_sink = IsolateSink::new(Isolate::new(params.question_stream_port));
    let answer_stream_buffer = self.stream_buffer.clone();
    let uid = self.user_service.user_id()?;
    let workspace_id = self.user_service.workspace_id()?;

    // 构建增强的系统提示词（如果有智能体配置）
    let system_prompt = if let Some(custom_prompt) = custom_system_prompt {
      // 🆕 使用自定义提示(已包含工具详情)
      info!("[Chat] 🔧 Using custom system prompt (with tool details)");
      Some(custom_prompt)
    } else if let Some(ref config) = agent_config {
      use crate::agent::{build_agent_system_prompt, AgentCapabilityExecutor};
      
      // 创建能力执行器
      let capability_executor = AgentCapabilityExecutor::new(self.user_service.clone());
      
      // 加载对话历史（如果启用了记忆功能）
      let conversation_history = capability_executor
        .load_conversation_history(&self.chat_id, &config.capabilities, uid)
        .unwrap_or_default();
      
      info!(
        "[Chat] Loaded {} messages from conversation history", 
        conversation_history.len()
      );
      
      // 构建基础系统提示词
      let base_prompt = build_agent_system_prompt(config);
      
      // 构建增强的系统提示词（包含历史、工具指南等）
      let enhanced_prompt = capability_executor.build_enhanced_system_prompt(
        base_prompt,
        config,
        &conversation_history,
      );
      
      info!(
        "[Chat] Using agent '{}' with enhanced system prompt ({} chars)",
        config.name,
        enhanced_prompt.len()
      );
      
      // 检查是否需要任务规划
      if capability_executor.should_create_plan(&config.capabilities, &params.message) {
        info!("[Chat] Complex task detected, task planning recommended");
        // TODO: 集成任务规划器
      }
      
      // 检查是否需要工具调用
      if capability_executor.should_use_tools(&config.capabilities, &params.message) {
        info!("[Chat] Tool usage recommended for this request");
        // TODO: 准备工具调用上下文
      }
      
      Some(enhanced_prompt)
    } else {
      None
    };

    // 保存原始用户消息到数据库（不包含系统提示词）
    let question = self
      .chat_service
      .create_question(
        &workspace_id,
        &self.chat_id,
        &params.message,  // 使用原始消息
        params.message_type.clone(),
        params.prompt_id.clone(),
      )
      .await
      .map_err(|err| {
        error!("Failed to send question: {}", err);
        FlowyError::server_error()
      })?;

    let _ = question_sink
      .send(StreamMessage::MessageId(question.message_id).to_string())
      .await;

    // Save message to disk
    notify_message(&self.chat_id, question.clone())?;
    let format = params.format.clone().map(Into::into).unwrap_or_default();
    
    // 传递系统提示词、智能体配置和工具调用处理器给 stream_response
    self.stream_response(
      params.answer_stream_port,
      answer_stream_buffer,
      uid,
      workspace_id,
      question.message_id,
      format,
      preferred_ai_model,
      system_prompt,
      agent_config,  // 🔧 传递智能体配置
      tool_call_handler,  // 🔧 传递工具调用处理器
    );

    let question_pb = ChatMessagePB::from(question);
    Ok(question_pb)
  }

  #[instrument(level = "info", skip_all, err)]
  pub async fn stream_regenerate_response(
    &self,
    question_id: i64,
    answer_stream_port: i64,
    format: Option<PredefinedFormatPB>,
    ai_model: AIModel,
  ) -> FlowyResult<()> {
    trace!(
      "[Chat] regenerate and stream chat message: chat_id={}",
      self.chat_id,
    );

    // clear
    self
      .stop_stream
      .store(false, std::sync::atomic::Ordering::SeqCst);
    self.stream_buffer.lock().await.clear();

    let format = format.map(Into::into).unwrap_or_default();
    let answer_stream_buffer = self.stream_buffer.clone();
    let uid = self.user_service.user_id()?;
    let workspace_id = self.user_service.workspace_id()?;

    self.stream_response(
      answer_stream_port,
      answer_stream_buffer,
      uid,
      workspace_id,
      question_id,
      format,
      ai_model,
      None, // 重新生成时不使用系统提示词
      None, // 🔧 重新生成时不使用智能体配置
      None, // 🔧 重新生成时不使用工具调用处理器
    );

    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  fn stream_response(
    &self,
    answer_stream_port: i64,
    answer_stream_buffer: Arc<Mutex<StringBuffer>>,
    _uid: i64,
    workspace_id: Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
    system_prompt: Option<String>,
    agent_config: Option<AgentConfigPB>,
    tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,  // 🔧 新增工具调用处理器
  ) {
    let stop_stream = self.stop_stream.clone();
    let chat_id = self.chat_id;
    let cloud_service = self.chat_service.clone();
    
    // 🔧 工具调用支持
    let has_agent = agent_config.is_some();
    let has_tool_handler = tool_call_handler.is_some();
    
    tokio::spawn(async move {
      let mut answer_sink = IsolateSink::new(Isolate::new(answer_stream_port));
      let mut accumulated_text = String::new();  // 🔧 累积文本用于检测工具调用
      
      // 🔧 多轮对话支持：记录工具调用和结果
      let mut tool_calls_and_results: Vec<(crate::agent::ToolCallRequest, crate::agent::ToolCallResponse)> = Vec::new();
      
      match cloud_service
        .stream_answer_with_system_prompt(&workspace_id, &chat_id, question_id, format.clone(), ai_model.clone(), system_prompt.clone())
        .await
      {
        Ok(mut stream) => {
          while let Some(message) = stream.next().await {
            match message {
              Ok(message) => {
                if stop_stream.load(std::sync::atomic::Ordering::Relaxed) {
                  trace!("[Chat] client stop streaming message");
                  break;
                }
                match message {
                  QuestionStreamValue::Answer { value } => {
                    // 🔧 累积文本以检测工具调用
                    if has_agent {
                      accumulated_text.push_str(&value);
                      
                      // 🐛 DEBUG: 每次接收到数据时打印累积文本的长度
                      // if accumulated_text.len() % 100 == 0 || accumulated_text.len() < 50 {
                      //   info!("🔧 [DEBUG] Accumulated text length: {} chars", accumulated_text.len());
                      //   if accumulated_text.len() < 200 {
                      //     info!("🔧 [DEBUG] Current text: {}", accumulated_text);
                      //   } else if accumulated_text.len() <= 300 {
                      //     // 安全截取前 200 字符
                      //     let mut preview_len = std::cmp::min(200, accumulated_text.len());
                      //     while preview_len > 0 && !accumulated_text.is_char_boundary(preview_len) {
                      //       preview_len -= 1;
                      //     }
                      //     info!("🔧 [DEBUG] Current text preview: {}", &accumulated_text[..preview_len]);
                      //   }
                      // }
                      
                      // 检测是否包含**完整的**工具调用（必须有开始和结束标签）
                      let has_start_tag = accumulated_text.contains("<tool_call>");
                      let has_end_tag = accumulated_text.contains("</tool_call>");
                      
                      // 🔧 同时检测 markdown 代码块格式 (AI 可能误用)
                      let has_markdown_tool_call = accumulated_text.contains("```tool_call") && 
                                                   accumulated_text.contains("```\n");
                      
                      // 🐛 DEBUG: 如果检测到标签,打印状态
                      // if has_start_tag || has_end_tag || has_markdown_tool_call {
                      //   info!("🔧 [DEBUG] Tool call tags detected - XML start: {}, XML end: {}, Markdown: {}", 
                      //         has_start_tag, has_end_tag, has_markdown_tool_call);
                      // }
                      
                      // 如果检测到 markdown 格式,转换为 XML 格式
                      if has_markdown_tool_call && !has_start_tag {
                        warn!("🔧 [TOOL] ⚠️ AI used markdown code block format instead of XML tags! Converting...");
                        accumulated_text = accumulated_text
                          .replace("```tool_call\n", "<tool_call>\n")
                          .replace("\n```", "\n</tool_call>");
                        info!("🔧 [TOOL] Converted markdown format to XML format");
                      }
                      
                      if has_start_tag && has_end_tag {
                        info!("🔧 [TOOL] Complete tool call detected in response");
                        
                        // 提取工具调用
                        let calls = crate::agent::ToolCallHandler::extract_tool_calls(&accumulated_text);
                        
                        info!("🔧 [TOOL] Extracted {} tool calls from accumulated text", calls.len());
                        
                        if calls.is_empty() {
                          warn!("🔧 [TOOL] ⚠️ Tool call tag found but extraction failed!");
                          warn!("🔧 [TOOL] Accumulated text length: {} chars", accumulated_text.len());
                          warn!("🔧 [TOOL] Number of <tool_call> tags: {}", accumulated_text.matches("<tool_call>").count());
                          warn!("🔧 [TOOL] Number of </tool_call> tags: {}", accumulated_text.matches("</tool_call>").count());
                          
                          // 显示更长的预览，包括可能的多个工具调用
                          let preview_len = std::cmp::min(accumulated_text.len(), 1500);
                          warn!("🔧 [TOOL] Accumulated text preview (first {} chars):", preview_len);
                          warn!("🔧 [TOOL] {}", &accumulated_text[..preview_len]);
                        }
                        
                        for (request, start, end) in calls {
                          // 发送工具调用前的文本
                          let before_text = &accumulated_text[..start];
                          if !before_text.is_empty() {
                            answer_stream_buffer.lock().await.push_str(before_text);
                            let _ = answer_sink
                              .send(StreamMessage::OnData(before_text.to_string()).to_string())
                              .await;
                          }
                          
                          // 发送工具调用元数据（通知 UI 工具正在执行）
                          let tool_metadata = serde_json::json!({
                            "tool_call": {
                              "id": request.id,
                              "tool_name": request.tool_name,
                              "status": "running",
                              "arguments": request.arguments,
                            }
                          });
                          let _ = answer_sink
                            .send(StreamMessage::Metadata(serde_json::to_string(&tool_metadata).unwrap()).to_string())
                            .await;
                          
                          info!("🔧 [TOOL] Executing tool: {} (id: {})", request.tool_name, request.id);
                          
                          // ✅ 实际执行工具
                          if has_tool_handler {
                            if let Some(ref handler) = tool_call_handler {
                              let response = handler.execute_tool_call(&request, agent_config.as_ref()).await;
                              
                              info!("🔧 [TOOL] Tool execution completed: {} - success: {}, has_result: {}",
                                    response.id, response.success, response.result.is_some());
                              
                              // 🔧 保存工具调用和结果，用于后续多轮对话
                              tool_calls_and_results.push((request.clone(), response.clone()));
                              info!("🔧 [TOOL] Saved tool result for multi-turn. Total saved: {}", tool_calls_and_results.len());
                              
                              // 发送工具执行结果元数据
                              let result_status = if response.success { "success" } else { "failed" };
                              let result_metadata = serde_json::json!({
                                "tool_call": {
                                  "id": response.id,
                                  "tool_name": request.tool_name,
                                  "status": result_status,
                                  "result": response.result,
                                  "error": response.error,
                                  "duration_ms": response.duration_ms,
                                }
                              });
                              let _ = answer_sink
                                .send(StreamMessage::Metadata(serde_json::to_string(&result_metadata).unwrap()).to_string())
                                .await;
                              
                              // ✅ 将工具执行结果发送给用户显示
                              // ⚠️ 注意：这个结果用于 UI 显示，实际的多轮对话逻辑在流结束后处理
                              if response.success {
                                if let Some(result_text) = response.result {
                                  let formatted_result = format!(
                                    "\n<tool_result>\n工具执行成功：{}\n结果：{}\n</tool_result>\n",
                                    request.tool_name,
                                    result_text
                                  );
                                  
                                  // 安全地生成预览，避免在 UTF-8 字符边界中间切割
                                  let preview = if result_text.len() > 100 {
                                    let mut preview_len = 100.min(result_text.len());
                                    while preview_len > 0 && !result_text.is_char_boundary(preview_len) {
                                      preview_len -= 1;
                                    }
                                    format!("{}...", &result_text[..preview_len])
                                  } else {
                                    result_text.clone()
                                  };
                                  
                                  info!("🔧 [TOOL] Sending tool result to UI ({}ms): {}", 
                                        response.duration_ms, 
                                        preview);
                                  
                                  // 发送工具结果到 UI
                                  answer_stream_buffer.lock().await.push_str(&formatted_result);
                                                let _ = answer_sink
                                    .send(StreamMessage::OnData(formatted_result).to_string())
                                                  .await;
                                  
                                  info!("🔧 [TOOL] Tool result sent to UI - will be used for follow-up AI response");
                                }
                              } else {
                                // 工具执行失败，通知用户
                                let error_msg = format!(
                                  "\n<tool_error>\n工具执行失败：{}\n错误：{}\n</tool_error>\n",
                                  request.tool_name,
                                  response.error.unwrap_or_else(|| "Unknown error".to_string())
                                );
                                
                                error!("🔧 [TOOL] Tool failed: {} - sending error to UI", response.id);
                                
                                answer_stream_buffer.lock().await.push_str(&error_msg);
                                let _ = answer_sink
                                  .send(StreamMessage::OnData(error_msg).to_string())
                                  .await;
                              }
                            }
                          } else {
                            // 没有工具处理器，发送占位消息
                            warn!("🔧 [TOOL] Tool handler not available, skipping execution");
                            let placeholder_metadata = serde_json::json!({
                              "tool_call": {
                                "id": request.id,
                                "tool_name": request.tool_name,
                                "status": "skipped",
                                "result": "Tool execution not configured",
                              }
                            });
                            let _ = answer_sink
                              .send(StreamMessage::Metadata(serde_json::to_string(&placeholder_metadata).unwrap()).to_string())
                              .await;
                          }
                          
                          // 清除已处理的文本
                          accumulated_text = accumulated_text[end..].to_string();
                        }
                        
                        // 发送剩余文本
                        if !accumulated_text.is_empty() {
                          answer_stream_buffer.lock().await.push_str(&accumulated_text);
                          let _ = answer_sink
                            .send(StreamMessage::OnData(accumulated_text.clone()).to_string())
                            .await;
                          accumulated_text.clear();
                        }
                      } else {
                        // 没有检测到工具调用，正常发送
                        answer_stream_buffer.lock().await.push_str(&value);
                        if let Err(err) = answer_sink
                          .send(StreamMessage::OnData(value).to_string())
                          .await
                        {
                          error!("Failed to stream answer via IsolateSink: {}", err);
                        }
                      }
                    } else {
                      // 没有智能体配置，正常发送
                      answer_stream_buffer.lock().await.push_str(&value);
                      if let Err(err) = answer_sink
                        .send(StreamMessage::OnData(value).to_string())
                        .await
                      {
                        error!("Failed to stream answer via IsolateSink: {}", err);
                      }
                    }
                  },
                  QuestionStreamValue::Metadata { value } => {
                    if let Ok(s) = serde_json::to_string(&value) {
                      answer_stream_buffer.lock().await.set_metadata(value);
                      let _ = answer_sink
                        .send(StreamMessage::Metadata(s).to_string())
                        .await;
                    }
                  },
                  QuestionStreamValue::SuggestedQuestion {
                    context_suggested_questions: _,
                  } => {},
                  QuestionStreamValue::FollowUp {
                    should_generate_related_question,
                  } => {
                    let _ = answer_sink
                      .send(
                        StreamMessage::OnFollowUp(AIFollowUpData {
                          should_generate_related_question,
                        })
                        .to_string(),
                      )
                      .await;
                  },
                }
              },
              Err(err) => {
                if err.code == ErrorCode::RequestTimeout || err.code == ErrorCode::Internal {
                  error!("[Chat] unexpected stream error: {}", err);
                  let _ = answer_sink.send(StreamMessage::Done.to_string()).await;
                } else {
                  error!("[Chat] failed to stream answer: {}", err);
                  let _ = answer_sink
                    .send(StreamMessage::OnError(err.msg.clone()).to_string())
                    .await;
                  let pb = ChatMessageErrorPB {
                    chat_id: chat_id.to_string(),
                    error_message: err.to_string(),
                  };
                  chat_notification_builder(chat_id, ChatNotification::StreamChatMessageError)
                    .payload(pb)
                    .send();
                  return Err(err);
                }
              },
            }
          }
          
          // 🔧 反思机制：如果有工具调用结果且启用了反思，进入反思循环
          info!("🔧 [REFLECTION] Stream ended - checking for reflection. has_agent: {}, tool_calls_count: {}", 
                has_agent, tool_calls_and_results.len());
          
          if has_agent && !tool_calls_and_results.is_empty() {
            // 🐛 DEBUG: 输出智能体配置信息
            if let Some(ref config) = agent_config {
              info!("🔧 [REFLECTION] ═══ Agent Configuration ═══");
              info!("🔧 [REFLECTION]   Agent ID: {}", config.id);
              info!("🔧 [REFLECTION]   Agent Name: {}", config.name);
              info!("🔧 [REFLECTION]   enable_reflection: {}", config.capabilities.enable_reflection);
              info!("🔧 [REFLECTION]   max_reflection_iterations: {}", config.capabilities.max_reflection_iterations);
              info!("🔧 [REFLECTION]   enable_tool_calling: {}", config.capabilities.enable_tool_calling);
              info!("🔧 [REFLECTION]   max_tool_calls: {}", config.capabilities.max_tool_calls);
              info!("🔧 [REFLECTION] ═══════════════════════════");
            } else {
              warn!("🔧 [REFLECTION] ⚠️ No agent config available!");
            }
            
            // 检查是否启用反思机制
            let enable_reflection = agent_config.as_ref()
              .map(|config| config.capabilities.enable_reflection)
              .unwrap_or(false);
            
            let max_iterations = agent_config.as_ref()
              .map(|config| {
                let configured = config.capabilities.max_reflection_iterations;
                if configured <= 0 || !enable_reflection {
                  1 // 如果未启用反思或配置为0，则只执行一次（传统模式）
                } else {
                  configured.min(10) as usize // 最大10次迭代
                }
              })
              .unwrap_or(1);
            
            info!("🔧 [REFLECTION] Calculated: enable_reflection={}, max_iterations={}", enable_reflection, max_iterations);
            info!("🔧 [REFLECTION] Starting reflection loop with {} initial tool call(s)", tool_calls_and_results.len());
            
            // 🔧 反思循环：多次迭代直到 AI 认为可以回答或达到限制
            let mut current_iteration = 0;
            let mut all_tool_results = tool_calls_and_results.clone();
            
            // 从智能体配置中获取工具结果最大长度限制，避免上下文过长
            let max_result_length = agent_config.as_ref()
              .map(|config| {
                // 确保值在合理范围内：最小 1000，默认 4000
                let configured = config.capabilities.max_tool_result_length;
                if configured <= 0 {
                  4000 // 默认值
                } else if configured < 1000 {
                  1000 // 最小值
            } else {
                  configured as usize
                }
              })
              .unwrap_or(4000); // 如果没有配置，使用默认值 4000
            
            // 🔁 开始反思循环
            while current_iteration < max_iterations {
              current_iteration += 1;
              info!("🔧 [REFLECTION] ═══ Iteration {}/{} ═══", current_iteration, max_iterations);
              info!("🔧 [REFLECTION] Current tool results count: {}", all_tool_results.len());
              
              // 构建包含所有工具结果的上下文消息
              let mut follow_up_context = String::new();
              if current_iteration == 1 {
                follow_up_context.push_str("\n\n以下是工具调用的结果，请基于这些结果回答用户的原始问题：\n\n");
              } else {
                follow_up_context.push_str(&format!("\n\n以下是第 {} 轮工具调用的所有结果：\n\n", current_iteration));
              }
            
              info!("🔧 [REFLECTION] Using max_tool_result_length: {} chars", max_result_length);
              
              // 遍历所有工具结果，构建上下文
              for (idx, (req, resp)) in all_tool_results.iter().enumerate() {
                // 使用 map 和 unwrap_or 避免临时值生命周期问题
                let result_text = resp.result.as_ref().map(|s| s.as_str()).unwrap_or("无结果");
                
                // 智能截断长结果
                let truncated_result = if result_text.len() > max_result_length {
                  // 安全截断，考虑 UTF-8 字符边界
                  let mut truncate_len = max_result_length.min(result_text.len());
                  while truncate_len > 0 && !result_text.is_char_boundary(truncate_len) {
                    truncate_len -= 1;
                  }
                  let truncated = &result_text[..truncate_len];
                  info!("🔧 [REFLECTION] Truncating tool result #{} from {} to {} chars", idx + 1, result_text.len(), truncate_len);
                  format!("{}...\n[结果已截断，原始长度: {} 字符]", truncated, result_text.len())
                } else {
                  result_text.to_string()
                };
                
                follow_up_context.push_str(&format!(
                  "工具调用 #{}: {}\n参数: {}\n结果: {}\n执行状态: {}\n\n",
                  idx + 1,
                  req.tool_name,
                  serde_json::to_string_pretty(&req.arguments).unwrap_or_else(|_| "无法序列化".to_string()),
                  truncated_result,
                  if resp.success { "成功" } else { "失败" }
                ));
              }
            
              // 根据是否启用反思机制和当前迭代，给 AI 不同的指示
              if enable_reflection && current_iteration < max_iterations {
                follow_up_context.push_str(&format!("请评估这些工具结果是否足以回答用户的问题（当前第 {}/{} 轮）：\n", current_iteration, max_iterations));
                follow_up_context.push_str("- 如果结果充分，请用中文简体总结并直接回答用户的问题\n");
                follow_up_context.push_str("- 如果结果不足或需要更多信息，可以继续调用其他可用工具\n");
                follow_up_context.push_str("- 避免调用已经尝试过的工具或重复的查询\n");
              } else {
                follow_up_context.push_str("请用中文简体总结和解释这些工具执行结果，直接回答用户的问题，不要再次调用工具。\n");
              }
              follow_up_context.push_str("注意：如果结果被截断，请基于可用信息给出最佳回答。");
            
              // 🐛 DEBUG: 打印 follow_up_context 的预览
              let context_preview_len = std::cmp::min(500, follow_up_context.len());
              let mut safe_preview_len = context_preview_len;
              while safe_preview_len > 0 && !follow_up_context.is_char_boundary(safe_preview_len) {
                safe_preview_len -= 1;
              }
              info!("🔧 [REFLECTION] Follow-up context preview: {}...", &follow_up_context[..safe_preview_len]);
              
              // 构建新的系统提示（包含原提示 + 工具结果上下文）
              let follow_up_system_prompt = if let Some(ref original_prompt) = system_prompt {
                format!("{}\n\n{}", original_prompt, follow_up_context)
              } else {
                follow_up_context
              };
              
              let prompt_len = follow_up_system_prompt.len();
              info!("🔧 [REFLECTION] Calling AI with follow-up context ({} chars)", prompt_len);
              
              // 检查上下文长度
              if prompt_len > 16000 {
                warn!("🔧 [REFLECTION] ⚠️ System prompt is very long ({} chars), may exceed model limit", prompt_len);
              }
              
              // 发送一个分隔符，让用户知道 AI 正在生成回答
              if current_iteration == 1 {
                let separator = "\n\n---\n\n";
                answer_stream_buffer.lock().await.push_str(separator);
                let _ = answer_sink
                  .send(StreamMessage::OnData(separator.to_string()).to_string())
                  .await;
              } else {
                let separator = format!("\n\n--- 第 {}/{} 轮反思 ---\n\n", current_iteration, max_iterations);
                answer_stream_buffer.lock().await.push_str(&separator);
                let _ = answer_sink
                  .send(StreamMessage::OnData(separator.clone()).to_string())
                  .await;
              }
            
              // 使用原始问题 + 工具结果上下文再次调用 AI
              info!("🔧 [REFLECTION] Calling AI with question_id: {}", question_id);
              
              match cloud_service
                .stream_answer_with_system_prompt(
                  &workspace_id, 
                  &chat_id, 
                  question_id, 
                  format.clone(), 
                  ai_model.clone(),
                  Some(follow_up_system_prompt)
                )
                .await
              {
                    Ok(mut follow_up_stream) => {
                      info!("🔧 [REFLECTION] Follow-up stream started for iteration {}", current_iteration);
                      let mut message_count = 0;
                      let mut answer_chunks = 0;
                      let mut has_received_data = false;
                      let mut reflection_accumulated_text = String::new(); // 🔧 累积文本用于检测新工具调用
                      let mut new_tool_calls_detected = false;
                      
                      while let Some(message) = follow_up_stream.next().await {
                        message_count += 1;
                        
                        if stop_stream.load(std::sync::atomic::Ordering::Relaxed) {
                          info!("🔧 [REFLECTION] Stream stopped by user after {} messages", message_count);
                          break;
                        }
                        
                        match message {
                          Ok(message) => {
                            match message {
                              QuestionStreamValue::Answer { value } => {
                                answer_chunks += 1;
                                has_received_data = true;
                                
                                // 🔧 反思机制：累积文本并检测新的工具调用
                                if enable_reflection && current_iteration < max_iterations {
                                  reflection_accumulated_text.push_str(&value);
                                  
                                  // 检测是否包含**完整的**工具调用
                                  let has_start_tag = reflection_accumulated_text.contains("<tool_call>");
                                  let has_end_tag = reflection_accumulated_text.contains("</tool_call>");
                                  
                                  if has_start_tag && has_end_tag && !new_tool_calls_detected {
                                    info!("🔧 [REFLECTION] Detected new tool call in iteration {} response!", current_iteration);
                                    new_tool_calls_detected = true;
                                    // 不立即退出循环，继续接收完整的响应
                                  }
                                }
                                
                                // 发送答案内容
                                answer_stream_buffer.lock().await.push_str(&value);
                                let _ = answer_sink
                                  .send(StreamMessage::OnData(value).to_string())
                                  .await;
                              },
                              QuestionStreamValue::Metadata { value } => {
                                if let Ok(s) = serde_json::to_string(&value) {
                                  answer_stream_buffer.lock().await.set_metadata(value);
                                  let _ = answer_sink
                                    .send(StreamMessage::Metadata(s).to_string())
                                    .await;
                                }
                              },
                              _ => {
                                // 忽略其他消息类型
                              }
                            }
                          },
                          Err(err) => {
                            error!("🔧 [REFLECTION] Stream error after {} messages: {}", message_count, err);
                            break;
                          }
                        }
                      }
                      
                      info!("🔧 [REFLECTION] Iteration {} completed: {} messages, {} answer chunks, has_data: {}, new_tools: {}", 
                            current_iteration, message_count, answer_chunks, has_received_data, new_tool_calls_detected);
                      
                      // 🔧 处理新检测到的工具调用
                      if new_tool_calls_detected && has_tool_handler && current_iteration < max_iterations {
                        info!("🔧 [REFLECTION] Processing new tool calls detected in iteration {}", current_iteration);
                        
                        // 提取新的工具调用（返回 Vec<(ToolCallRequest, usize, usize)>）
                        let new_calls_raw = crate::agent::ToolCallHandler::extract_tool_calls(&reflection_accumulated_text);
                        let new_calls: Vec<_> = new_calls_raw.into_iter().map(|(req, _, _)| req).collect();
                        info!("🔧 [REFLECTION] Extracted {} new tool calls", new_calls.len());
                        
                        if !new_calls.is_empty() {
                          // 执行新的工具调用
                          for call in new_calls {
                            info!("🔧 [REFLECTION] Executing new tool: {} (iteration {})", call.tool_name, current_iteration);
                            
                            if let Some(ref handler) = tool_call_handler {
                              let response = handler.execute_tool_call(&call, agent_config.as_ref()).await;
                              if response.success {
                                info!("🔧 [REFLECTION] Tool {} executed successfully in iteration {}", call.tool_name, current_iteration);
                              } else {
                                warn!("🔧 [REFLECTION] Tool {} execution returned success=false in iteration {}", call.tool_name, current_iteration);
                              }
                              all_tool_results.push((call, response));
                            }
                          }
                          
                          // 继续下一轮迭代
                          info!("🔧 [REFLECTION] New tools executed, continuing to iteration {}", current_iteration + 1);
                          continue; // 继续 while 循环
                        } else {
                          warn!("🔧 [REFLECTION] Tool call tags found but extraction failed in iteration {}", current_iteration);
                        }
                      }
                      
                      // 没有新工具调用，退出循环
                      info!("🔧 [REFLECTION] No new tool calls detected, ending reflection loop");
                      
                      // 如果没有收到数据，发送降级消息
                      if !has_received_data {
                        warn!("🔧 [REFLECTION] ⚠️ No data received from iteration {} stream!", current_iteration);
                        warn!("🔧 [REFLECTION]   Possible causes:");
                        warn!("🔧 [REFLECTION]   1. AI model returned empty response");
                        warn!("🔧 [REFLECTION]   2. System prompt too long ({} chars)", prompt_len);
                        warn!("🔧 [REFLECTION]   3. Original question not found for question_id: {}", question_id);
                        warn!("🔧 [REFLECTION] 💡 Fallback: Sending tool result summary to user");
                        
                        // 降级方案：直接发送工具结果的简单总结
                        let fallback_message = format!(
                          "\n\n📊 工具执行完成（第 {}/{} 轮）\n\n{} 工具已成功执行并返回结果（如上所示）。\n\n由于 AI 服务暂时无法生成详细总结，请您直接查看上方的工具执行结果。\n\n💡 提示：\n- 如果结果过长，请在智能体配置中增加「工具结果最大长度」\n- 或尝试使用支持更长上下文的 AI 模型\n- 当前 System Prompt 长度：{} 字符\n",
                          current_iteration,
                          max_iterations,
                          all_tool_results.len(),
                          prompt_len
                        );
                        
                        answer_stream_buffer.lock().await.push_str(&fallback_message);
                        let _ = answer_sink
                          .send(StreamMessage::OnData(fallback_message).to_string())
                          .await;
                      }
                      
                      break; // 退出 while 循环
                    },
                    Err(err) => {
                      error!("🔧 [REFLECTION] Failed to start stream for iteration {}: {}", current_iteration, err);
                      let error_msg = format!("\n\n生成回答时出错（第 {}/{} 轮）: {}\n", current_iteration, max_iterations, err);
                      answer_stream_buffer.lock().await.push_str(&error_msg);
                      let _ = answer_sink
                        .send(StreamMessage::OnData(error_msg).to_string())
                        .await;
                      break; // 退出 while 循环
                    }
              }
            } // end of while loop
            
            info!("🔧 [REFLECTION] Reflection loop ended after {} iterations with {} total tool results", 
                  current_iteration, all_tool_results.len());
          }
        },
        Err(err) => {
          error!("[Chat] failed to start streaming: {}", err);
          if err.is_ai_response_limit_exceeded() {
            let _ = answer_sink
              .send(StreamMessage::AIResponseLimitExceeded.to_string())
              .await;
          } else if err.is_ai_image_response_limit_exceeded() {
            let _ = answer_sink
              .send(StreamMessage::AIImageResponseLimitExceeded.to_string())
              .await;
          } else if err.is_ai_max_required() {
            let _ = answer_sink
              .send(StreamMessage::AIMaxRequired(err.msg.clone()).to_string())
              .await;
          } else if err.is_local_ai_not_ready() {
            let _ = answer_sink
              .send(StreamMessage::LocalAINotReady(err.msg.clone()).to_string())
              .await;
          } else if err.is_local_ai_disabled() {
            let _ = answer_sink
              .send(StreamMessage::LocalAIDisabled(err.msg.clone()).to_string())
              .await;
          } else {
            let _ = answer_sink
              .send(StreamMessage::OnError(err.msg.clone()).to_string())
              .await;
          }

          let pb = ChatMessageErrorPB {
            chat_id: chat_id.to_string(),
            error_message: err.to_string(),
          };
          chat_notification_builder(chat_id, ChatNotification::StreamChatMessageError)
            .payload(pb)
            .send();
          return Err(err);
        },
      }

      chat_notification_builder(chat_id, ChatNotification::FinishStreaming).send();
      trace!("[Chat] finish streaming");

      if answer_stream_buffer.lock().await.is_empty() {
        return Ok(());
      }
      let content = answer_stream_buffer.lock().await.take_content();
      let metadata = answer_stream_buffer.lock().await.take_metadata();
      let answer = cloud_service
        .create_answer(
          &workspace_id,
          &chat_id,
          content.trim(),
          question_id,
          metadata,
        )
        .await?;
      notify_message(&chat_id, answer)?;
      Ok::<(), FlowyError>(())
    });
  }

  /// Load chat messages for a given `chat_id`.
  ///
  /// 1. When opening a chat:
  ///    - Loads local chat messages.
  ///    - `after_message_id` and `before_message_id` are `None`.
  ///    - Spawns a task to load messages from the remote server, notifying the user when the remote messages are loaded.
  ///
  /// 2. Loading more messages in an existing chat with `after_message_id`:
  ///    - `after_message_id` is the last message ID in the current chat messages.
  ///
  /// 3. Loading more messages in an existing chat with `before_message_id`:
  ///    - `before_message_id` is the first message ID in the current chat messages.
  pub async fn load_prev_chat_messages(
    &self,
    limit: u64,
    before_message_id: Option<i64>,
  ) -> Result<ChatMessageListPB, FlowyError> {
    trace!(
      "[Chat] Loading messages from disk: chat_id={}, limit={}, before_message_id={:?}",
      self.chat_id, limit, before_message_id
    );

    let offset = before_message_id.map_or(MessageCursor::NextBack, MessageCursor::BeforeMessageId);
    let messages = self.load_local_chat_messages(limit, offset).await?;

    // If the number of messages equals the limit, then no need to load more messages from remote
    if messages.len() == limit as usize {
      let pb = ChatMessageListPB {
        messages,
        has_more: true,
        total: 0,
      };
      chat_notification_builder(self.chat_id, ChatNotification::DidLoadPrevChatMessage)
        .payload(pb.clone())
        .send();
      return Ok(pb);
    }

    if matches!(
      *self.prev_message_state.read().await,
      PrevMessageState::HasMore
    ) {
      *self.prev_message_state.write().await = PrevMessageState::Loading;
      if let Err(err) = self
        .load_remote_chat_messages(limit, before_message_id, None)
        .await
      {
        error!("Failed to load previous chat messages: {}", err);
      }
    }

    Ok(ChatMessageListPB {
      messages,
      has_more: true,
      total: 0,
    })
  }

  pub async fn load_latest_chat_messages(
    &self,
    limit: u64,
    after_message_id: Option<i64>,
  ) -> Result<ChatMessageListPB, FlowyError> {
    trace!(
      "[Chat] Loading new messages: chat_id={}, limit={}, after_message_id={:?}",
      self.chat_id, limit, after_message_id,
    );
    let offset = after_message_id.map_or(MessageCursor::NextBack, MessageCursor::AfterMessageId);
    let messages = self.load_local_chat_messages(limit, offset).await?;

    trace!(
      "[Chat] Loaded local chat messages: chat_id={}, messages={}",
      self.chat_id,
      messages.len()
    );

    // If the number of messages equals the limit, then no need to load more messages from remote
    let has_more = !messages.is_empty();
    let _ = self
      .load_remote_chat_messages(limit, None, after_message_id)
      .await;
    Ok(ChatMessageListPB {
      messages,
      has_more,
      total: 0,
    })
  }

  async fn load_remote_chat_messages(
    &self,
    limit: u64,
    before_message_id: Option<i64>,
    after_message_id: Option<i64>,
  ) -> FlowyResult<()> {
    trace!(
      "[Chat] start loading messages from remote: chat_id={}, limit={}, before_message_id={:?}, after_message_id={:?}",
      self.chat_id, limit, before_message_id, after_message_id
    );
    let workspace_id = self.user_service.workspace_id()?;
    let chat_id = self.chat_id;
    let cloud_service = self.chat_service.clone();
    let user_service = self.user_service.clone();
    let uid = self.uid;
    let prev_message_state = self.prev_message_state.clone();
    let latest_message_id = self.latest_message_id.clone();
    tokio::spawn(async move {
      let cursor = match (before_message_id, after_message_id) {
        (Some(bid), _) => MessageCursor::BeforeMessageId(bid),
        (_, Some(aid)) => MessageCursor::AfterMessageId(aid),
        _ => MessageCursor::NextBack,
      };
      match cloud_service
        .get_chat_messages(&workspace_id, &chat_id, cursor.clone(), limit)
        .await
      {
        Ok(resp) => {
          // Save chat messages to local disk
          if let Err(err) = save_chat_message_disk(
            user_service.sqlite_connection(uid)?,
            &chat_id,
            resp.messages.clone(),
            true,
          ) {
            error!("Failed to save chat:{} messages: {}", chat_id, err);
          }

          // Update latest message ID
          if !resp.messages.is_empty() {
            latest_message_id.store(
              resp.messages[0].message_id,
              std::sync::atomic::Ordering::Relaxed,
            );
          }

          let pb = ChatMessageListPB::from(resp);
          trace!(
            "[Chat] Loaded messages from remote: chat_id={}, messages={}, hasMore: {}, cursor:{:?}",
            chat_id,
            pb.messages.len(),
            pb.has_more,
            cursor,
          );
          if matches!(cursor, MessageCursor::BeforeMessageId(_)) {
            if pb.has_more {
              *prev_message_state.write().await = PrevMessageState::HasMore;
            } else {
              *prev_message_state.write().await = PrevMessageState::NoMore;
            }
            chat_notification_builder(chat_id, ChatNotification::DidLoadPrevChatMessage)
              .payload(pb)
              .send();
          } else {
            chat_notification_builder(chat_id, ChatNotification::DidLoadLatestChatMessage)
              .payload(pb)
              .send();
          }
        },
        Err(err) => error!("Failed to load chat messages: {}", err),
      }
      Ok::<(), FlowyError>(())
    });
    Ok(())
  }

  pub async fn get_question_id_from_answer_id(
    &self,
    chat_id: &Uuid,
    answer_message_id: i64,
  ) -> Result<i64, FlowyError> {
    let conn = self.user_service.sqlite_connection(self.uid)?;

    let local_result =
      select_answer_where_match_reply_message_id(conn, &chat_id.to_string(), answer_message_id)?
        .map(|message| message.message_id);

    if let Some(message_id) = local_result {
      return Ok(message_id);
    }

    let workspace_id = self.user_service.workspace_id()?;
    let chat_id = self.chat_id;
    let cloud_service = self.chat_service.clone();

    let question = cloud_service
      .get_question_from_answer_id(&workspace_id, &chat_id, answer_message_id)
      .await?;

    Ok(question.message_id)
  }

  pub async fn get_related_question(
    &self,
    message_id: i64,
    ai_model: AIModel,
  ) -> Result<RepeatedRelatedQuestionPB, FlowyError> {
    let workspace_id = self.user_service.workspace_id()?;
    let resp = self
      .chat_service
      .get_related_message(&workspace_id, &self.chat_id, message_id, ai_model)
      .await?;

    trace!(
      "[Chat] related messages: chat_id={}, message_id={}, messages:{:?}",
      self.chat_id, message_id, resp.items
    );
    Ok(RepeatedRelatedQuestionPB::from(resp))
  }

  #[instrument(level = "debug", skip_all, err)]
  pub async fn generate_answer(&self, question_message_id: i64) -> FlowyResult<ChatMessagePB> {
    trace!(
      "[Chat] generate answer: chat_id={}, question_message_id={}",
      self.chat_id, question_message_id
    );
    let workspace_id = self.user_service.workspace_id()?;
    let answer = self
      .chat_service
      .get_answer(&workspace_id, &self.chat_id, question_message_id)
      .await?;

    notify_message(&self.chat_id, answer.clone())?;
    let pb = ChatMessagePB::from(answer);
    Ok(pb)
  }

  async fn load_local_chat_messages(
    &self,
    limit: u64,
    offset: MessageCursor,
  ) -> Result<Vec<ChatMessagePB>, FlowyError> {
    trace!(
      "[Chat] Loading messages from disk: chat_id={}, limit={}, offset={:?}",
      self.chat_id, limit, offset
    );
    let conn = self.user_service.sqlite_connection(self.uid)?;
    let rows = select_chat_messages(conn, &self.chat_id.to_string(), limit, offset)?.messages;
    let messages = rows
      .into_iter()
      .map(|record| ChatMessagePB {
        message_id: record.message_id,
        content: record.content,
        created_at: record.created_at,
        author_type: record.author_type,
        author_id: record.author_id,
        reply_message_id: record.reply_message_id,
        metadata: record.metadata,
      })
      .collect::<Vec<_>>();

    Ok(messages)
  }

  #[instrument(level = "debug", skip_all, err)]
  pub async fn index_file(&self, file_path: PathBuf) -> FlowyResult<()> {
    if !file_path.exists() {
      return Err(
        FlowyError::record_not_found().with_context(format!("{:?} not exist", file_path)),
      );
    }

    if !file_path.is_file() {
      return Err(
        FlowyError::invalid_data().with_context(format!("{:?} is not a file ", file_path)),
      );
    }

    trace!(
      "[Chat] index file: chat_id={}, file_path={:?}",
      self.chat_id, file_path
    );
    self
      .chat_service
      .embed_file(
        &self.user_service.workspace_id()?,
        &file_path,
        &self.chat_id,
        None,
      )
      .await?;

    trace!(
      "[Chat] created index file record: chat_id={}, file_path={:?}",
      self.chat_id, file_path
    );

    Ok(())
  }
}

fn save_chat_message_disk(
  conn: DBConnection,
  chat_id: &Uuid,
  messages: Vec<ChatMessage>,
  is_sync: bool,
) -> FlowyResult<()> {
  let records = messages
    .into_iter()
    .map(|message| ChatMessageTable {
      message_id: message.message_id,
      chat_id: chat_id.to_string(),
      content: message.content,
      created_at: message.created_at.timestamp(),
      author_type: message.author.author_type as i64,
      author_id: message.author.author_id.to_string(),
      reply_message_id: message.reply_message_id,
      metadata: Some(serde_json::to_string(&message.metadata).unwrap_or_default()),
      is_sync,
    })
    .collect::<Vec<_>>();
  upsert_chat_messages(conn, &records)?;
  Ok(())
}

#[derive(Debug, Default)]
struct StringBuffer {
  content: String,
  metadata: Option<serde_json::Value>,
}

impl StringBuffer {
  fn clear(&mut self) {
    self.content.clear();
    self.metadata = None;
  }

  fn push_str(&mut self, value: &str) {
    self.content.push_str(value);
  }

  fn set_metadata(&mut self, value: serde_json::Value) {
    self.metadata = Some(value);
  }

  fn is_empty(&self) -> bool {
    self.content.is_empty()
  }

  fn take_metadata(&mut self) -> Option<serde_json::Value> {
    self.metadata.take()
  }

  fn take_content(&mut self) -> String {
    std::mem::take(&mut self.content)
  }
}

pub(crate) fn notify_message(chat_id: &Uuid, message: ChatMessage) -> Result<(), FlowyError> {
  trace!("[Chat] save answer: answer={:?}", message);
  let pb = ChatMessagePB::from(message);
  chat_notification_builder(chat_id, ChatNotification::DidReceiveChatMessage)
    .payload(pb)
    .send();

  Ok(())
}
