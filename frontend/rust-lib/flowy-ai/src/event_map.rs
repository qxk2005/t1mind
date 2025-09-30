use std::sync::{Arc, Weak};

use strum_macros::Display;

use crate::completion::AICompletion;
use flowy_derive::{Flowy_Event, ProtoBuf_Enum};
use lib_dispatch::prelude::*;

use crate::ai_manager::AIManager;
use crate::event_handler::*;
use crate::mcp::event_handler::*;
use crate::agent::event_handler::*;

pub fn init(ai_manager: Weak<AIManager>) -> AFPlugin {
  let strong_ai_manager = ai_manager.upgrade().unwrap();
  let user_service = Arc::downgrade(&strong_ai_manager.user_service);
  let cloud_service = Arc::downgrade(&strong_ai_manager.cloud_service_wm);
  let ai_tools = Arc::new(AICompletion::new(cloud_service, user_service));
  AFPlugin::new()
    .name("flowy-ai")
    .state(ai_manager)
    .state(ai_tools)
    .event(AIEvent::StreamMessage, stream_chat_message_handler)
    .event(AIEvent::LoadPrevMessage, load_prev_message_handler)
    .event(AIEvent::LoadNextMessage, load_next_message_handler)
    .event(AIEvent::GetRelatedQuestion, get_related_question_handler)
    .event(AIEvent::GetAnswerForQuestion, get_answer_handler)
    .event(AIEvent::StopStream, stop_stream_handler)
    .event(AIEvent::CompleteText, start_complete_text_handler)
    .event(AIEvent::StopCompleteText, stop_complete_text_handler)
    .event(AIEvent::ChatWithFile, chat_file_handler)
    .event(AIEvent::RestartLocalAI, restart_local_ai_handler)
    .event(AIEvent::ToggleLocalAI, toggle_local_ai_handler)
    .event(AIEvent::GetLocalAIState, get_local_ai_state_handler)
    .event(AIEvent::GetLocalAISetting, get_local_ai_setting_handler)
    .event(AIEvent::GetLocalModelSelection, get_local_ai_models_handler)
    .event(
      AIEvent::GetSourceModelSelection,
      get_source_model_selection_handler,
    )
    .event(
      AIEvent::UpdateLocalAISetting,
      update_local_ai_setting_handler,
    )
    .event(AIEvent::CreateChatContext, create_chat_context_handler)
    .event(AIEvent::GetChatInfo, create_chat_context_handler)
    .event(AIEvent::GetChatSettings, get_chat_settings_handler)
    .event(AIEvent::UpdateChatSettings, update_chat_settings_handler)
    .event(AIEvent::RegenerateResponse, regenerate_response_handler)
    .event(
      AIEvent::GetSettingModelSelection,
      get_setting_model_selection_handler,
    )
    .event(AIEvent::UpdateSelectedModel, update_selected_model_handler)
    .event(
      AIEvent::GetCustomPromptDatabaseConfiguration,
      get_custom_prompt_database_configuration_handler,
    )
    .event(
      AIEvent::SetCustomPromptDatabaseConfiguration,
      set_custom_prompt_database_configuration_handler,
    )
    // MCP事件注册
    .event(AIEvent::GetMCPServerList, get_mcp_server_list_handler)
    .event(AIEvent::AddMCPServer, add_mcp_server_handler)
    .event(AIEvent::UpdateMCPServer, update_mcp_server_handler)
    .event(AIEvent::RemoveMCPServer, remove_mcp_server_handler)
    .event(AIEvent::ConnectMCPServer, connect_mcp_server_handler)
    .event(AIEvent::DisconnectMCPServer, disconnect_mcp_server_handler)
    .event(AIEvent::GetMCPServerStatus, get_mcp_server_status_handler)
    .event(AIEvent::GetMCPToolList, get_mcp_tool_list_handler)
    .event(AIEvent::CallMCPTool, call_mcp_tool_handler)
    // 智能体事件注册
    .event(AIEvent::GetAgentList, get_agent_list_handler)
    .event(AIEvent::CreateAgent, create_agent_handler)
    .event(AIEvent::GetAgent, get_agent_handler)
    .event(AIEvent::UpdateAgent, update_agent_handler)
    .event(AIEvent::DeleteAgent, delete_agent_handler)
    .event(AIEvent::ValidateAgentConfig, validate_agent_config_handler)
    .event(AIEvent::GetAgentGlobalSettings, get_agent_global_settings_handler)
    .event(AIEvent::UpdateAgentGlobalSettings, update_agent_global_settings_handler)
    // 执行日志事件注册
    .event(AIEvent::GetExecutionLogs, get_execution_logs_handler)
    .event(AIEvent::AddExecutionLog, add_execution_log_handler)
    .event(AIEvent::ClearExecutionLogs, clear_execution_logs_handler)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, Hash, ProtoBuf_Enum, Flowy_Event)]
#[event_err = "FlowyError"]
pub enum AIEvent {
  /// Create a new workspace
  #[event(input = "LoadPrevChatMessagePB", output = "ChatMessageListPB")]
  LoadPrevMessage = 0,

  #[event(input = "LoadNextChatMessagePB", output = "ChatMessageListPB")]
  LoadNextMessage = 1,

  #[event(input = "StreamChatPayloadPB", output = "ChatMessagePB")]
  StreamMessage = 2,

  #[event(input = "StopStreamPB")]
  StopStream = 3,

  #[event(input = "ChatMessageIdPB", output = "RepeatedRelatedQuestionPB")]
  GetRelatedQuestion = 4,

  #[event(input = "ChatMessageIdPB", output = "ChatMessagePB")]
  GetAnswerForQuestion = 5,

  #[event(input = "CompleteTextPB", output = "CompleteTextTaskPB")]
  CompleteText = 9,

  #[event(input = "CompleteTextTaskPB")]
  StopCompleteText = 10,

  #[event(input = "ChatFilePB")]
  ChatWithFile = 11,

  /// Restart local AI chat. When plugin quit or user terminate in task manager or activity monitor,
  /// the plugin will need to restart.
  #[event()]
  RestartLocalAI = 17,

  /// Enable or disable local AI
  #[event(output = "LocalAIPB")]
  ToggleLocalAI = 18,

  /// Return LocalAIPB that contains the current state of the local AI
  #[event(output = "LocalAIPB")]
  GetLocalAIState = 19,

  #[event(input = "CreateChatContextPB")]
  CreateChatContext = 23,

  #[event(input = "ChatId", output = "ChatInfoPB")]
  GetChatInfo = 24,

  #[event(input = "ChatId", output = "ChatSettingsPB")]
  GetChatSettings = 25,

  #[event(input = "UpdateChatSettingsPB")]
  UpdateChatSettings = 26,

  #[event(input = "RegenerateResponsePB")]
  RegenerateResponse = 27,

  #[event(output = "LocalAISettingPB")]
  GetLocalAISetting = 29,

  #[event(input = "LocalAISettingPB")]
  UpdateLocalAISetting = 30,

  #[event(input = "ModelSourcePB", output = "ModelSelectionPB")]
  GetSettingModelSelection = 31,

  #[event(input = "UpdateSelectedModelPB")]
  UpdateSelectedModel = 32,

  #[event(output = "ModelSelectionPB")]
  GetLocalModelSelection = 33,

  #[event(input = "ModelSourcePB", output = "ModelSelectionPB")]
  GetSourceModelSelection = 34,

  #[event(output = "CustomPromptDatabaseConfigurationPB")]
  GetCustomPromptDatabaseConfiguration = 35,

  #[event(input = "CustomPromptDatabaseConfigurationPB")]
  SetCustomPromptDatabaseConfiguration = 36,

  // ==================== MCP相关事件 ====================
  
  /// 获取MCP服务器列表
  #[event(output = "MCPServerListPB")]
  GetMCPServerList = 37,

  /// 添加MCP服务器配置
  #[event(input = "MCPServerConfigPB")]
  AddMCPServer = 38,

  /// 更新MCP服务器配置
  #[event(input = "MCPServerConfigPB")]
  UpdateMCPServer = 39,

  /// 删除MCP服务器
  #[event(input = "MCPDisconnectServerRequestPB")]
  RemoveMCPServer = 40,

  /// 连接MCP服务器
  #[event(input = "MCPConnectServerRequestPB", output = "MCPServerStatusPB")]
  ConnectMCPServer = 41,

  /// 断开MCP服务器连接
  #[event(input = "MCPDisconnectServerRequestPB")]
  DisconnectMCPServer = 42,

  /// 获取MCP服务器状态
  #[event(input = "MCPConnectServerRequestPB", output = "MCPServerStatusPB")]
  GetMCPServerStatus = 43,

  /// 获取MCP工具列表
  #[event(input = "MCPConnectServerRequestPB", output = "MCPToolListPB")]
  GetMCPToolList = 44,

  /// 调用MCP工具
  #[event(input = "MCPToolCallRequestPB", output = "MCPToolCallResponsePB")]
  CallMCPTool = 45,

  // ==================== 智能体相关事件 ====================
  
  /// 获取智能体列表
  #[event(output = "AgentListPB")]
  GetAgentList = 46,

  /// 创建智能体
  #[event(input = "CreateAgentRequestPB", output = "AgentConfigPB")]
  CreateAgent = 47,

  /// 获取智能体配置
  #[event(input = "GetAgentRequestPB", output = "AgentConfigPB")]
  GetAgent = 48,

  /// 更新智能体配置
  #[event(input = "UpdateAgentRequestPB", output = "AgentConfigPB")]
  UpdateAgent = 49,

  /// 删除智能体
  #[event(input = "DeleteAgentRequestPB")]
  DeleteAgent = 50,

  /// 验证智能体配置
  #[event(input = "AgentConfigPB", output = "AgentSuccessResponsePB")]
  ValidateAgentConfig = 51,

  /// 获取智能体全局设置
  #[event(output = "AgentGlobalSettingsPB")]
  GetAgentGlobalSettings = 52,

  /// 更新智能体全局设置
  #[event(input = "AgentGlobalSettingsPB")]
  UpdateAgentGlobalSettings = 53,

  // ==================== 执行日志相关事件 ====================
  
  /// 获取执行日志列表
  #[event(input = "GetExecutionLogsRequestPB", output = "AgentExecutionLogListPB")]
  GetExecutionLogs = 54,

  /// 添加执行日志
  #[event(input = "AgentExecutionLogPB")]
  AddExecutionLog = 55,

  /// 清空执行日志
  #[event(input = "ClearExecutionLogsRequestPB")]
  ClearExecutionLogs = 56,
}
