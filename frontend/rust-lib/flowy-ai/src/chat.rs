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
      
      match cloud_service
        .stream_answer_with_system_prompt(&workspace_id, &chat_id, question_id, format, ai_model, system_prompt)
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
                      if accumulated_text.len() % 100 == 0 || accumulated_text.len() < 50 {
                        info!("🔧 [DEBUG] Accumulated text length: {} chars", accumulated_text.len());
                        if accumulated_text.len() < 200 {
                          info!("🔧 [DEBUG] Current text: {}", accumulated_text);
                        } else if accumulated_text.len() <= 300 {
                          // 安全截取前 200 字符
                          let mut preview_len = std::cmp::min(200, accumulated_text.len());
                          while preview_len > 0 && !accumulated_text.is_char_boundary(preview_len) {
                            preview_len -= 1;
                          }
                          info!("🔧 [DEBUG] Current text preview: {}", &accumulated_text[..preview_len]);
                        }
                      }
                      
                      // 检测是否包含**完整的**工具调用（必须有开始和结束标签）
                      let has_start_tag = accumulated_text.contains("<tool_call>");
                      let has_end_tag = accumulated_text.contains("</tool_call>");
                      
                      // 🔧 同时检测 markdown 代码块格式 (AI 可能误用)
                      let has_markdown_tool_call = accumulated_text.contains("```tool_call") && 
                                                   accumulated_text.contains("```\n");
                      
                      // 🐛 DEBUG: 如果检测到标签,打印状态
                      if has_start_tag || has_end_tag || has_markdown_tool_call {
                        info!("🔧 [DEBUG] Tool call tags detected - XML start: {}, XML end: {}, Markdown: {}", 
                              has_start_tag, has_end_tag, has_markdown_tool_call);
                      }
                      
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
                              
                              // ✅ 将工具执行结果发送给用户
                              // ⚠️ 注意：这个结果只发送到UI，AI模型在单轮对话中看不到
                              // TODO: 实现多轮对话，将工具结果反馈给AI生成后续响应
                              if response.success {
                                if let Some(result_text) = response.result {
                                  let formatted_result = format!(
                                    "\n<tool_result>\n工具执行成功：{}\n结果：{}\n</tool_result>\n",
                                    request.tool_name,
                                    result_text
                                  );
                                  
                                  info!("🔧 [TOOL] Sending tool result to UI ({}ms): {}", 
                                        response.duration_ms, 
                                        if result_text.len() > 100 { 
                                          format!("{}...", &result_text[..100]) 
                                        } else { 
                                          result_text.clone() 
                                        });
                                  
                                  // 发送工具结果到 UI
                                  answer_stream_buffer.lock().await.push_str(&formatted_result);
                                  let _ = answer_sink
                                    .send(StreamMessage::OnData(formatted_result).to_string())
                                    .await;
                                  
                                  info!("🔧 [TOOL] ⚠️ Tool result sent to UI - AI model won't see this in current conversation turn");
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
          
          // 🐛 DEBUG: 流结束时打印完整的累积文本
          if has_agent && !accumulated_text.is_empty() {
            info!("🔧 [DEBUG] Stream ended with accumulated text length: {} chars", accumulated_text.len());
            
            // 安全地截取前 500 字符(考虑 UTF-8 字符边界)
            let preview = if accumulated_text.len() <= 500 {
              accumulated_text.as_str()
            } else {
              // 找到最近的 UTF-8 字符边界
              let mut preview_len = 500;
              while preview_len > 0 && !accumulated_text.is_char_boundary(preview_len) {
                preview_len -= 1;
              }
              &accumulated_text[..preview_len]
            };
            info!("🔧 [DEBUG] Final text preview (first {} chars): {}", preview.len(), preview);
            
            // 检查是否包含任何工具调用标签
            let has_start = accumulated_text.contains("<tool_call>");
            let has_end = accumulated_text.contains("</tool_call>");
            info!("🔧 [DEBUG] Final check - has <tool_call>: {}, has </tool_call>: {}", has_start, has_end);
            
            // 如果文本很长,也打印最后 200 字符
            if accumulated_text.len() > 500 {
              let end_preview_start = accumulated_text.len().saturating_sub(200);
              let mut end_start = end_preview_start;
              while end_start < accumulated_text.len() && !accumulated_text.is_char_boundary(end_start) {
                end_start += 1;
              }
              info!("🔧 [DEBUG] Final text ending (last {} chars): {}", 
                    accumulated_text.len() - end_start, 
                    &accumulated_text[end_start..]);
            }
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
