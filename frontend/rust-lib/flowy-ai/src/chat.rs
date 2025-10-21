use crate::entities::{
  AgentConfigPB, ChatMessageErrorPB, ChatMessageListPB, ChatMessagePB, PredefinedFormatPB,
  RepeatedRelatedQuestionPB, StreamMessageParams, AgentExecutionLogPB, ToolDefinitionPB,
};
use crate::middleware::chat_service_mw::{ChatServiceMiddleware, OpenAIToolCall, OpenAIFunctionCall};
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
use dashmap::DashMap;

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
    execution_logs: Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>,  // 📝 执行日志存储
    tool_definitions: Option<Vec<ToolDefinitionPB>>,  // 🆕 工具定义列表（用于 OpenAI Function Call）
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
      // trace!("[Chat] 🔧 Using custom system prompt (with tool details)");
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
        trace!("[Chat] Complex task detected, task planning recommended");
        // TODO: 集成任务规划器
      }
      
      // 检查是否需要工具调用
      if capability_executor.should_use_tools(&config.capabilities, &params.message) {
        trace!("[Chat] Tool usage recommended for this request");
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

    // 🔧 删除此处的 notify_message 调用，避免用户消息重复
    // 用户消息会作为函数返回值 (line 203) 发送给前端，不需要在这里再次通知
    // notify_message(&self.chat_id, question.clone())?;
    let format = params.format.clone().map(Into::into).unwrap_or_default();
    
    // 传递系统提示词、智能体配置和工具调用处理器给 stream_response
    // Disabled debug logging to reduce noise
    // info!("🔧 [CHAT] About to call stream_response: chat_id={}, question_id={}, answer_stream_port={}", 
    //       self.chat_id, question.message_id, params.answer_stream_port);
    
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
      execution_logs,  // 📝 传递执行日志存储
      tool_definitions,  // 🆕 传递工具定义列表
    );
    
    // Disabled debug logging to reduce noise
    // info!("🔧 [CHAT] stream_response call completed");

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
      None, // 📝 重新生成时不使用执行日志
      None, // 🆕 重新生成时不使用工具定义
    );

    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  fn stream_response(
    &self,
    answer_stream_port: i64,
    answer_stream_buffer: Arc<Mutex<StringBuffer>>,
    uid: i64,
    workspace_id: Uuid,
    question_id: i64,
    format: ResponseFormat,
    ai_model: AIModel,
    system_prompt: Option<String>,
    agent_config: Option<AgentConfigPB>,
    tool_call_handler: Option<Arc<crate::agent::ToolCallHandler>>,  // 🔧 新增工具调用处理器
    execution_logs: Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>,  // 📝 执行日志存储
    tool_definitions: Option<Vec<ToolDefinitionPB>>,  // 🆕 工具定义列表
  ) {
    let stop_stream = self.stop_stream.clone();
    let chat_id = self.chat_id;
    let cloud_service = self.chat_service.clone();
    let user_service = self.user_service.clone();  // 🔧 捕获 user_service 用于保存消息到数据库
    
    // 🔧 工具调用支持
    let has_agent = agent_config.is_some();
    let has_tool_handler = tool_call_handler.is_some();
      
      // 🔧 多轮对话支持：记录工具调用和结果（OpenAI Function Call API 将自动处理，此变量保留用于向后兼容）
      let tool_calls_and_results: Vec<(crate::agent::ToolCallRequest, crate::agent::ToolCallResponse)> = Vec::new();
      
      // 🔄 收集流式响应中的 tool_calls（用于多轮对话）
      let collected_tool_calls = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    
    // 📝 调试：检查执行日志是否被传递
    let has_execution_logs = execution_logs.is_some();
    let has_tool_definitions = tool_definitions.is_some();
    let tool_count = tool_definitions.as_ref().map(|t| t.len()).unwrap_or(0);
    info!("🔧 [EXECUTION] Starting stream_response: chat_id={}, question_id={}, has_agent={}, has_tool_handler={}, has_tool_definitions={}, tool_count={}, has_execution_logs={}", 
          chat_id, question_id, has_agent, has_tool_handler, has_tool_definitions, tool_count, has_execution_logs);
    
    // 📝 详细调试信息 - Disabled to reduce noise
    // if let Some(ref config) = agent_config {
    //   info!("🔧 [AGENT] Using agent: {} ({}), tool_calling_enabled: {}, available_tools: {:?}", 
    //         config.name, config.id, config.capabilities.enable_tool_calling, config.available_tools);
    // } else {
    //   warn!("🔧 [AGENT] No agent config provided!");
    // }
    
    // if let Some(ref tools) = tool_definitions {
    //   info!("🔧 [TOOLS] Tool definitions loaded: {} tools", tools.len());
    //   for tool in tools {
    //     info!("🔧 [TOOL] - {}: {} ({:?})", tool.name, tool.description, tool.tool_type);
    //   }
    // } else {
    //   warn!("🔧 [TOOLS] No tool definitions provided!");
    // }
    
    tokio::spawn(async move {
      let mut answer_sink = IsolateSink::new(Isolate::new(answer_stream_port));
                  
      // 🔧 多轮对话支持：记录工具调用和结果
            
      // 📝 日志记录辅助函数
      let add_log = |logs: &Option<Arc<DashMap<String, Vec<AgentExecutionLogPB>>>>, log: AgentExecutionLogPB| {
        if let Some(logs_map) = logs {
          let session_key = format!("{}_{}", log.session_id, log.message_id);
          info!("📝 [EXECUTION-LOG] Recording log: session_key={}, phase={:?}, step={}", 
                session_key, log.phase, log.step);
          logs_map.entry(session_key.clone())
            .or_insert_with(Vec::new)
            .push(log);
          let count = logs_map.get(&session_key).map(|v| v.len()).unwrap_or(0);
          info!("📝 [EXECUTION-LOG] Total logs for session: {}", count);
        } else {
          warn!("📝 [EXECUTION-LOG] Cannot record log - execution_logs is None! phase={:?}, step={}", 
                log.phase, log.step);
        }
      };
      
      // 🔄 使用自动多轮对话（如果启用了工具调用）
      let stream_result = if has_agent && has_tool_handler && tool_definitions.is_some() {
        // Disabled debug logging to reduce noise
        // info!("🔄 [AUTO-MULTI-TURN] Using auto multi-turn conversation with {} tools", 
        //       tool_definitions.as_ref().unwrap().len());
        
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
        trace!("🔄 [SIMPLE-STREAM] Using simple stream response (no agent/tools)");
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
      
      match stream_result {
        Ok(mut stream) => {
          // info!("🔧 [CHAT] ✅ Stream received successfully, starting to consume");
          // 📝 记录聊天开始日志
          {
            let mut log = AgentExecutionLogPB::new(
              chat_id.to_string(),
              question_id.to_string(),
              crate::entities::ExecutionPhasePB::ExecExecution,
              "AI聊天开始".to_string(),
            );
            log.input = format!("模型: {}, 格式: {:?}", ai_model.name, format);
            log.status = crate::entities::ExecutionStatusPB::ExecRunning;
            add_log(&execution_logs, log);
          }
          
          // info!("🔧 [CHAT] About to consume stream messages");
          while let Some(message) = stream.next().await {
            // info!("🔧 [CHAT] Received message from stream");
            match message {
              Ok(message) => {
                if stop_stream.load(std::sync::atomic::Ordering::Relaxed) {
                  trace!("[Chat] client stop streaming message");
                  break;
                }
                match message {
                  QuestionStreamValue::Answer { value } => {
                    // 🆕 使用 OpenAI Function Call API，无需手工检测 <tool_call> 标签
                    // Metadata 中的 tool_call 信息由 middleware 自动处理
                    // Disabled debug logging to reduce noise
                    // info!("🔧 [STREAM-DATA] Received answer data: '{}'", value);
                    answer_stream_buffer.lock().await.push_str(&value);
                    if let Err(err) = answer_sink
                      .send(StreamMessage::OnData(value).to_string())
                      .await
                    {
                      error!("Failed to stream answer via IsolateSink: {}", err);
                    } else {
                      // Disabled debug logging to reduce noise
                      // info!("🔧 [STREAM-DATA] Successfully sent data to Flutter");
                    }
                  },
                  QuestionStreamValue::Metadata { value } => {
                    // 🔄 检查是否有 tool_call 并收集
                    if let Some(tool_call_obj) = value.get("tool_call") {
                      if let Some(status) = tool_call_obj.get("status").and_then(|s| s.as_str()) {
                        if status == "pending" {
                          // 收集 tool_call 信息
                          if let (Some(id), Some(tool_name), Some(arguments)) = (
                            tool_call_obj.get("id").and_then(|v| v.as_str()),
                            tool_call_obj.get("tool_name").and_then(|v| v.as_str()),
                            tool_call_obj.get("arguments").and_then(|v| v.as_str()),
                          ) {
                            let tc = OpenAIToolCall {
                              id: id.to_string(),
                              tool_type: "function".to_string(),
                              function: OpenAIFunctionCall {
                                name: tool_name.to_string(),
                                arguments: arguments.to_string(),
                              },
                            };
                            collected_tool_calls.lock().await.push(tc);
                            info!("🔄 [MULTI-TURN] Collected tool_call: {} (total: {})", 
                                  tool_name, collected_tool_calls.lock().await.len());
                            
                            // 📝 记录工具调用开始日志
                            {
                              let mut log = AgentExecutionLogPB::new(
                                chat_id.to_string(),
                                question_id.to_string(),
                                crate::entities::ExecutionPhasePB::ExecToolCall,
                                format!("工具调用开始: {}", tool_name),
                              );
                              log.input = arguments.to_string();
                              log.status = crate::entities::ExecutionStatusPB::ExecRunning;
                              add_log(&execution_logs, log);
                            }
                          }
                        } else if status == "success" || status == "failed" {
                          // 📝 记录工具调用完成日志
                          if let (Some(_id), Some(tool_name)) = (
                            tool_call_obj.get("id").and_then(|v| v.as_str()),
                            tool_call_obj.get("tool_name").and_then(|v| v.as_str()),
                          ) {
                            let mut log = AgentExecutionLogPB::new(
                              chat_id.to_string(),
                              question_id.to_string(),
                              crate::entities::ExecutionPhasePB::ExecToolCall,
                              format!("工具调用完成: {}", tool_name),
                            );
                            
                            if status == "success" {
                              if let Some(result) = tool_call_obj.get("result").and_then(|v| v.as_str()) {
                                log.mark_completed(result.to_string());
                              } else {
                                log.mark_completed("工具执行成功".to_string());
                              }
                            } else {
                              if let Some(error) = tool_call_obj.get("error").and_then(|v| v.as_str()) {
                                log.mark_failed(error.to_string());
                              } else {
                                log.mark_failed("工具执行失败".to_string());
                              }
                            }
                            
                            add_log(&execution_logs, log);
                          }
                        }
                      }
                    }
                    
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
          
          // 📝 记录聊天完成日志
          {
            let final_content = answer_stream_buffer.lock().await.content.clone();
            let mut log = AgentExecutionLogPB::new(
              chat_id.to_string(),
              question_id.to_string(),
              crate::entities::ExecutionPhasePB::ExecCompletion,
              "AI聊天完成".to_string(),
            );
            log.mark_completed(format!("生成了 {} 字符的回复", final_content.len()));
            add_log(&execution_logs, log);
          }
          
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
            // TODO: 多轮对话将在 middleware 层自动处理
            // 当前版本仅记录检测到的 tool_calls
            info!("🔄 [MULTI-TURN] Tool calls will be handled by middleware layer");
            
            // 发送提示信息
            let hint = format!("

🔄 **检测到 {} 个工具调用，将在后续版本中自动处理**

", tool_calls_vec.len());
            let _ = answer_sink.send(StreamMessage::OnData(hint).to_string()).await;
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
              
              // 📝 记录反思迭代开始日志
              {
                let mut log = AgentExecutionLogPB::new(
                  chat_id.to_string(),
                  question_id.to_string(),
                  crate::entities::ExecutionPhasePB::ExecReflection,
                  format!("反思迭代 {}/{}", current_iteration, max_iterations),
                );
                log.input = format!("工具结果数量: {}", all_tool_results.len());
                log.status = crate::entities::ExecutionStatusPB::ExecRunning;
                add_log(&execution_logs, log);
              }
              
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
                follow_up_context.push_str("- 如果结果不足或需要更多信息，你**必须**使用工具调用格式继续调用其他可用工具\n");
                follow_up_context.push_str("- 避免调用已经尝试过的工具或重复的查询\n\n");
                follow_up_context.push_str("**重要提醒**：如果你需要更多信息，不要只是描述你要做什么，而是**直接输出工具调用**：\n");
                follow_up_context.push_str("<tool_call>\n");
                follow_up_context.push_str("{\n");
                follow_up_context.push_str("  \"id\": \"unique_call_id\",\n");
                follow_up_context.push_str("  \"tool_name\": \"工具名称\",\n");
                follow_up_context.push_str("  \"arguments\": { ... }\n");
                follow_up_context.push_str("}\n");
                follow_up_context.push_str("</tool_call>\n");
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
                  Some(follow_up_system_prompt),
                  None  // 反思流程不传递工具定义
                )
                .await
              {
                    Ok(mut follow_up_stream) => {
                      info!("🔧 [REFLECTION] Follow-up stream started for iteration {}", current_iteration);
                      let mut message_count = 0;
                      let answer_chunks = 0;
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
                    // 🔄 检查是否有 tool_call 并收集
                    if let Some(tool_call_obj) = value.get("tool_call") {
                      if let Some(status) = tool_call_obj.get("status").and_then(|s| s.as_str()) {
                        if status == "pending" {
                          // 收集 tool_call 信息
                          if let (Some(id), Some(tool_name), Some(arguments)) = (
                            tool_call_obj.get("id").and_then(|v| v.as_str()),
                            tool_call_obj.get("tool_name").and_then(|v| v.as_str()),
                            tool_call_obj.get("arguments").and_then(|v| v.as_str()),
                          ) {
                            let tc = OpenAIToolCall {
                              id: id.to_string(),
                              tool_type: "function".to_string(),
                              function: OpenAIFunctionCall {
                                name: tool_name.to_string(),
                                arguments: arguments.to_string(),
                              },
                            };
                            collected_tool_calls.lock().await.push(tc);
                            info!("🔄 [MULTI-TURN] Collected tool_call: {} (total: {})", 
                                  tool_name, collected_tool_calls.lock().await.len());
                            
                            // 📝 记录工具调用开始日志
                            {
                              let mut log = AgentExecutionLogPB::new(
                                chat_id.to_string(),
                                question_id.to_string(),
                                crate::entities::ExecutionPhasePB::ExecToolCall,
                                format!("工具调用开始: {}", tool_name),
                              );
                              log.input = arguments.to_string();
                              log.status = crate::entities::ExecutionStatusPB::ExecRunning;
                              add_log(&execution_logs, log);
                            }
                          }
                        } else if status == "success" || status == "failed" {
                          // 📝 记录工具调用完成日志
                          if let (Some(_id), Some(tool_name)) = (
                            tool_call_obj.get("id").and_then(|v| v.as_str()),
                            tool_call_obj.get("tool_name").and_then(|v| v.as_str()),
                          ) {
                            let mut log = AgentExecutionLogPB::new(
                              chat_id.to_string(),
                              question_id.to_string(),
                              crate::entities::ExecutionPhasePB::ExecToolCall,
                              format!("工具调用完成: {}", tool_name),
                            );
                            
                            if status == "success" {
                              if let Some(result) = tool_call_obj.get("result").and_then(|v| v.as_str()) {
                                log.mark_completed(result.to_string());
                              } else {
                                log.mark_completed("工具执行成功".to_string());
                              }
                            } else {
                              if let Some(error) = tool_call_obj.get("error").and_then(|v| v.as_str()) {
                                log.mark_failed(error.to_string());
                              } else {
                                log.mark_failed("工具执行失败".to_string());
                              }
                            }
                            
                            add_log(&execution_logs, log);
                          }
                        }
                      }
                    }
                    
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
                      
                      // 🐛 DEBUG: 打印AI响应内容预览（用于调试）
                      if !new_tool_calls_detected && !reflection_accumulated_text.is_empty() {
                        let preview_len = std::cmp::min(500, reflection_accumulated_text.len());
                        let mut safe_preview_len = preview_len;
                        while safe_preview_len > 0 && !reflection_accumulated_text.is_char_boundary(safe_preview_len) {
                          safe_preview_len -= 1;
                        }
                        info!("🔧 [REFLECTION] AI response preview (no tool calls detected): {}...", 
                              &reflection_accumulated_text[..safe_preview_len]);
                        info!("🔧 [REFLECTION] Total response length: {} chars", reflection_accumulated_text.len());
                      }
                      
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
                            
                            // 📝 记录反思中新工具调用开始日志
                            {
                              let mut log = AgentExecutionLogPB::new(
                                chat_id.to_string(),
                                question_id.to_string(),
                                crate::entities::ExecutionPhasePB::ExecReflection,
                                format!("反思中执行工具: {} (迭代 {})", call.tool_name, current_iteration),
                              );
                              log.input = serde_json::to_string(&call.arguments).unwrap_or_default();
                              log.status = crate::entities::ExecutionStatusPB::ExecRunning;
                              add_log(&execution_logs, log);
                            }
                            
                            if let Some(ref handler) = tool_call_handler {
                              let response = handler.execute_tool_call(&call, agent_config.as_ref()).await;
                              
                              // 📝 记录反思中工具调用完成日志
                              if response.success {
                                info!("🔧 [REFLECTION] Tool {} executed successfully in iteration {}", call.tool_name, current_iteration);
                                if let Some(ref result_text) = response.result {
                                  let mut log = AgentExecutionLogPB::new(
                                    chat_id.to_string(),
                                    question_id.to_string(),
                                    crate::entities::ExecutionPhasePB::ExecReflection,
                                    format!("反思中工具执行成功: {} (迭代 {})", call.tool_name, current_iteration),
                                  );
                                  log.mark_completed(result_text.clone());
                                  add_log(&execution_logs, log);
                                }
                              } else {
                                warn!("🔧 [REFLECTION] Tool {} execution returned success=false in iteration {}", call.tool_name, current_iteration);
                                let error_text = response.error.clone().unwrap_or_else(|| "Unknown error".to_string());
                                let mut log = AgentExecutionLogPB::new(
                                  chat_id.to_string(),
                                  question_id.to_string(),
                                  crate::entities::ExecutionPhasePB::ExecReflection,
                                  format!("反思中工具执行失败: {} (迭代 {})", call.tool_name, current_iteration),
                                );
                                log.mark_failed(error_text);
                                add_log(&execution_logs, log);
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
      let mut metadata = answer_stream_buffer.lock().await.take_metadata();
      
      // 🔧 在保存前，将执行日志添加到 metadata 中
      if let Some(logs_map) = &execution_logs {
        let session_key = format!("{}_{}", chat_id, question_id);
        
        if let Some(logs_entry) = logs_map.get(&session_key) {
          let logs = logs_entry.value();
          
          if !logs.is_empty() {
            // 将执行日志序列化为 JSON
            if let Ok(logs_json) = serde_json::to_value(logs) {
              // 如果 metadata 还不是对象，创建一个
              if metadata.is_none() {
                metadata = Some(serde_json::json!({}));
              }
              
              if let Some(metadata_obj) = metadata.as_mut() {
                if let Some(obj) = metadata_obj.as_object_mut() {
                  obj.insert("execution_logs".to_string(), logs_json);
                }
              }
            }
          }
        }
      }
      
      // 🔧 关键修复：从 tool_calls 中提取引用信息并添加到 sources 字段
      // 这样重启后可以从数据库恢复引用信息（支持 web_search 和 MCP 工具）
      if let Some(metadata_obj) = metadata.as_mut() {
        if let Some(obj) = metadata_obj.as_object() {
          let mut sources = Vec::new();
          
          // 检查是否有 tool_calls 数组
          if let Some(tool_calls) = obj.get("tool_calls") {
            if let Some(calls_array) = tool_calls.as_array() {
              for tool_call in calls_array {
                if let Some(tool_obj) = tool_call.as_object() {
                  // 获取工具调用的基本信息
                  let tool_name = tool_obj.get("tool_name").and_then(|v| v.as_str());
                  let status = tool_obj.get("status").and_then(|v| v.as_str());
                  let tool_id = tool_obj.get("id").and_then(|v| v.as_str());
                  
                  // 只处理成功的工具调用
                  if let (Some(name), Some("success")) = (tool_name, status) {
                    // 处理 AppFlowy 内置的 web_search 工具
                    if name == "web_search" {
                      if let Some(result) = tool_obj.get("result").and_then(|v| v.as_str()) {
                        // 从搜索结果中提取 URL 引用
                        let citations = extract_citations_from_search_result(result);
                        if !citations.is_empty() {
                          info!("📎 [METADATA] 从 web_search 结果中提取了 {} 个引用", citations.len());
                          sources.extend(citations);
                        }
                      }
                    } 
                    // 处理 MCP 工具（非 web_search 的所有其他工具）
                    else {
                      // 提取 server 信息（如果 tool_name 包含 server 前缀）
                      // 格式: "server_name.tool_name" 或直接 "tool_name"
                      let (display_name, server_id) = if name.contains('.') {
                        let parts: Vec<&str> = name.split('.').collect();
                        if parts.len() >= 2 {
                          let server = parts[0];
                          let tool = parts[1..].join(".");
                          (tool, Some(server))
                        } else {
                          (name.to_string(), None)
                        }
                      } else {
                        (name.to_string(), None)
                      };
                      
                      // 创建 MCP 引用
                      let mcp_source = format!("mcp:{}", server_id.unwrap_or("unknown"));
                      let citation = serde_json::json!({
                        "id": tool_id.unwrap_or(name),
                        "name": display_name,
                        "source": mcp_source
                      });
                      
                      sources.push(citation);
                      info!("📎 [METADATA] 添加 MCP 工具引用: {} (source: {})", display_name, mcp_source);
                    }
                  }
                }
              }
            }
          }
          
          // 如果提取到了引用，合并到 sources 字段（而不是覆盖）
          if !sources.is_empty() {
            if let Some(metadata_obj_mut) = metadata.as_mut() {
              if let Some(obj_mut) = metadata_obj_mut.as_object_mut() {
                // 🔧 修复：合并sources而不是覆盖
                // 获取现有的sources数组
                let existing_sources = obj_mut
                  .entry("sources")
                  .or_insert_with(|| serde_json::Value::Array(vec![]));
                
                if let Some(sources_array) = existing_sources.as_array_mut() {
                  // 合并新的sources，根据id去重
                  let sources_count = sources.len();
                  for new_source in sources {
                    let source_id = new_source.get("id").and_then(|v| v.as_str());
                    let already_exists = sources_array.iter().any(|s| {
                      s.get("id").and_then(|v| v.as_str()) == source_id
                    });
                    if !already_exists {
                      sources_array.push(new_source);
                    }
                  }
                  info!("✅ [METADATA] 已合并 {} 个引用到 metadata.sources（当前总数: {}）", sources_count, sources_array.len());
                }
              }
            }
          }
        }
      }
      
      let answer = cloud_service
        .create_answer(
          &workspace_id,
          &chat_id,
          content.trim(),
          question_id,
          metadata,
        )
        .await?;
      
      // 🔧 关键修复：将回答消息保存到本地数据库（确保 reply_message_id 被持久化）
      // 这样在程序重启后，从数据库加载消息时，reply_message_id 仍然存在
      if let Err(err) = save_chat_message_disk(
        user_service.sqlite_connection(uid)?,
        &chat_id,
        vec![answer.clone()],
        true,
      ) {
        error!("Failed to save answer message to disk: {}", err);
      }
      
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

/// 从网络搜索结果字符串中提取引用信息
/// 
/// 搜索结果格式示例：
/// ```
/// 搜索结果 (查询词):
/// 1. 标题1
///    链接: https://example.com/1
/// 2. 标题2
///    链接: https://example.com/2
/// ```
fn extract_citations_from_search_result(result: &str) -> Vec<serde_json::Value> {
  use regex::Regex;
  
  let mut citations = Vec::new();
  
  // 使用正则表达式匹配引用模式
  // 匹配格式: 数字. 标题\n   链接: URL
  let pattern = Regex::new(r"(\d+)\.\s+([^\n]+)\s+链接:\s+(https?://[^\s]+)").unwrap();
  
  for cap in pattern.captures_iter(result) {
    if let (Some(_index), Some(title), Some(url)) = (cap.get(1), cap.get(2), cap.get(3)) {
      let title_str = title.as_str().trim();
      let url_str = url.as_str().trim();
      
      if !title_str.is_empty() && !url_str.is_empty() {
        citations.push(serde_json::json!({
          "id": url_str,
          "name": title_str,
          "source": "web"
        }));
      }
    }
  }
  
  citations
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

  /// 🔧 修改为累积 metadata 而不是替换
  /// 这样可以保存完整的执行日志：推理文本、工具调用、任务规划等
  fn set_metadata(&mut self, value: serde_json::Value) {
    if let Some(existing) = &mut self.metadata {
      // 如果已有 metadata，进行智能合并
      if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), value.as_object()) {
        for (key, new_value) in new_obj {
          match key.as_str() {
            // 🔹 reasoning_delta: 累积成 reasoning_text
            "reasoning_delta" => {
              if let Some(delta_str) = new_value.as_str() {
                let current_text = existing_obj
                  .get("reasoning_text")
                  .and_then(|v| v.as_str())
                  .unwrap_or("");
                existing_obj.insert(
                  "reasoning_text".to_string(),
                  serde_json::Value::String(format!("{}{}", current_text, delta_str)),
                );
              }
            }
            // 🔹 tool_call: 累积到 tool_calls 数组
            "tool_call" => {
              let tool_calls = existing_obj
                .entry("tool_calls")
                .or_insert_with(|| serde_json::Value::Array(vec![]));
              
              if let Some(calls_array) = tool_calls.as_array_mut() {
                // 检查是否已存在相同 ID 的工具调用
                if let Some(call_id) = new_value.get("id").and_then(|v| v.as_str()) {
                  if let Some(existing_call) = calls_array.iter_mut().find(|c| {
                    c.get("id").and_then(|v| v.as_str()) == Some(call_id)
                  }) {
                    // 更新现有工具调用
                    *existing_call = new_value.clone();
                  } else {
                    // 添加新工具调用
                    calls_array.push(new_value.clone());
                  }
                }
              }
            }
            // 🔹 task_plan: 更新任务规划（直接替换）
            "task_plan" => {
              existing_obj.insert(key.clone(), new_value.clone());
            }
            // 🔹 sources: 合并源列表（去重）
            "sources" => {
              if let Some(new_sources) = new_value.as_array() {
                let existing_sources = existing_obj
                  .entry("sources")
                  .or_insert_with(|| serde_json::Value::Array(vec![]));
                
                if let Some(sources_array) = existing_sources.as_array_mut() {
                  for new_source in new_sources {
                    // 根据 id 去重
                    let source_id = new_source.get("id").and_then(|v| v.as_str());
                    let already_exists = sources_array.iter().any(|s| {
                      s.get("id").and_then(|v| v.as_str()) == source_id
                    });
                    if !already_exists {
                      sources_array.push(new_source.clone());
                    }
                  }
                }
              }
            }
            // 🔹 其他字段：直接更新或添加
            _ => {
              existing_obj.insert(key.clone(), new_value.clone());
            }
          }
        }
      }
    } else {
      // 如果还没有 metadata，直接设置
      self.metadata = Some(value);
    }
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
