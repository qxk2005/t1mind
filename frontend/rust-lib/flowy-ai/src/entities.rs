use crate::local_ai::controller::LocalAISetting;
use crate::local_ai::resource::PendingResource;
use flowy_ai_pub::cloud::{
  AIModel, ChatMessage, ChatMessageType, CompletionMessage, LLMModel, OutputContent, OutputLayout,
  RelatedQuestion, RepeatedChatMessage, RepeatedRelatedQuestion, ResponseFormat,
};
use flowy_derive::{ProtoBuf, ProtoBuf_Enum};
use lib_infra::validator_fn::required_not_empty_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;
use chrono::Utc;

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ChatId {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub value: String,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ChatInfoPB {
  #[pb(index = 1)]
  pub chat_id: String,

  #[pb(index = 2)]
  pub files: Vec<FilePB>,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct FilePB {
  #[pb(index = 1)]
  pub id: String,
  #[pb(index = 2)]
  pub name: String,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct SendChatPayloadPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub message: String,

  #[pb(index = 3)]
  pub message_type: ChatMessageTypePB,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct StreamChatPayloadPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub message: String,

  #[pb(index = 3)]
  pub message_type: ChatMessageTypePB,

  #[pb(index = 4)]
  pub answer_stream_port: i64,

  #[pb(index = 5)]
  pub question_stream_port: i64,

  #[pb(index = 6, one_of)]
  pub format: Option<PredefinedFormatPB>,

  #[pb(index = 7, one_of)]
  pub prompt_id: Option<String>,

  #[pb(index = 8, one_of)]
  pub agent_id: Option<String>,
}

#[derive(Default, Debug)]
pub struct StreamMessageParams {
  pub chat_id: Uuid,
  pub message: String,
  pub message_type: ChatMessageType,
  pub answer_stream_port: i64,
  pub question_stream_port: i64,
  pub format: Option<PredefinedFormatPB>,
  pub prompt_id: Option<String>,
  pub agent_id: Option<String>,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct RegenerateResponsePB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,

  #[pb(index = 2)]
  pub answer_message_id: i64,

  #[pb(index = 3)]
  pub answer_stream_port: i64,

  #[pb(index = 4, one_of)]
  pub format: Option<PredefinedFormatPB>,

  #[pb(index = 5, one_of)]
  pub model: Option<AIModelPB>,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ChatMessageMetaPB {
  #[pb(index = 1)]
  pub id: String,

  #[pb(index = 2)]
  pub name: String,

  #[pb(index = 3)]
  pub data: String,

  #[pb(index = 4)]
  pub loader_type: ContextLoaderTypePB,

  #[pb(index = 5)]
  pub source: String,
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum, PartialEq, Eq, Copy)]
pub enum ContextLoaderTypePB {
  #[default]
  UnknownLoaderType = 0,
  Txt = 1,
  Markdown = 2,
  PDF = 3,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct StopStreamPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum, PartialEq, Eq, Copy)]
pub enum ChatMessageTypePB {
  #[default]
  System = 0,
  User = 1,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct LoadPrevChatMessagePB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,

  #[pb(index = 2)]
  pub limit: i64,

  #[pb(index = 4, one_of)]
  pub before_message_id: Option<i64>,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct LoadNextChatMessagePB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,

  #[pb(index = 2)]
  pub limit: i64,

  #[pb(index = 4, one_of)]
  pub after_message_id: Option<i64>,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ChatMessageListPB {
  #[pb(index = 1)]
  pub has_more: bool,

  #[pb(index = 2)]
  pub messages: Vec<ChatMessagePB>,

  /// If the total number of messages is 0, then the total number of messages is unknown.
  #[pb(index = 3)]
  pub total: i64,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ServerModelSelectionPB {
  #[pb(index = 1)]
  pub models: Vec<AvailableModelPB>,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AvailableModelPB {
  #[pb(index = 1)]
  pub name: String,

  #[pb(index = 2)]
  pub is_default: bool,

  #[pb(index = 3)]
  pub desc: String,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ModelSourcePB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub source: String,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct UpdateSelectedModelPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub source: String,

  #[pb(index = 2)]
  pub selected_model: AIModelPB,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ModelSelectionPB {
  #[pb(index = 1)]
  pub models: Vec<AIModelPB>,

  #[pb(index = 2)]
  pub selected_model: AIModelPB,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct RepeatedAIModelPB {
  #[pb(index = 1)]
  pub items: Vec<AIModelPB>,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AIModelPB {
  #[pb(index = 1)]
  pub name: String,

  #[pb(index = 2)]
  pub is_local: bool,

  #[pb(index = 3)]
  pub desc: String,
}

impl From<AIModel> for AIModelPB {
  fn from(model: AIModel) -> Self {
    Self {
      name: model.name,
      is_local: model.is_local,
      desc: model.desc,
    }
  }
}

impl From<AIModelPB> for AIModel {
  fn from(value: AIModelPB) -> Self {
    AIModel {
      name: value.name,
      is_local: value.is_local,
      desc: value.desc,
    }
  }
}

impl From<RepeatedChatMessage> for ChatMessageListPB {
  fn from(repeated_chat_message: RepeatedChatMessage) -> Self {
    let messages = repeated_chat_message
      .messages
      .into_iter()
      .map(ChatMessagePB::from)
      .collect();
    ChatMessageListPB {
      has_more: repeated_chat_message.has_more,
      messages,
      total: repeated_chat_message.total,
    }
  }
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct ChatMessagePB {
  #[pb(index = 1)]
  pub message_id: i64,

  #[pb(index = 2)]
  pub content: String,

  #[pb(index = 3)]
  pub created_at: i64,

  #[pb(index = 4)]
  pub author_type: i64,

  #[pb(index = 5)]
  pub author_id: String,

  #[pb(index = 6, one_of)]
  pub reply_message_id: Option<i64>,

  #[pb(index = 7, one_of)]
  pub metadata: Option<String>,
  // #[pb(index = 8)]
  // pub should_fetch_related_question: bool,
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct ChatMessageErrorPB {
  #[pb(index = 1)]
  pub chat_id: String,

  #[pb(index = 2)]
  pub error_message: String,
}

impl From<ChatMessage> for ChatMessagePB {
  fn from(chat_message: ChatMessage) -> Self {
    ChatMessagePB {
      message_id: chat_message.message_id,
      content: chat_message.content,
      created_at: chat_message.created_at.timestamp(),
      author_type: chat_message.author.author_type as i64,
      author_id: chat_message.author.author_id.to_string(),
      reply_message_id: chat_message.reply_message_id,  // ✅ 使用实际的 reply_message_id
      metadata: Some(serde_json::to_string(&chat_message.metadata).unwrap_or_default()),
    }
  }
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct RepeatedChatMessagePB {
  #[pb(index = 1)]
  items: Vec<ChatMessagePB>,
}

impl From<Vec<ChatMessage>> for RepeatedChatMessagePB {
  fn from(messages: Vec<ChatMessage>) -> Self {
    RepeatedChatMessagePB {
      items: messages.into_iter().map(ChatMessagePB::from).collect(),
    }
  }
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct ChatMessageIdPB {
  #[pb(index = 1)]
  pub chat_id: String,

  #[pb(index = 2)]
  pub message_id: i64,
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct RelatedQuestionPB {
  #[pb(index = 1)]
  pub content: String,
}

impl From<RelatedQuestion> for RelatedQuestionPB {
  fn from(value: RelatedQuestion) -> Self {
    RelatedQuestionPB {
      content: value.content,
    }
  }
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct RepeatedRelatedQuestionPB {
  #[pb(index = 1)]
  pub message_id: i64,

  #[pb(index = 2)]
  pub items: Vec<RelatedQuestionPB>,
}

impl From<RepeatedRelatedQuestion> for RepeatedRelatedQuestionPB {
  fn from(value: RepeatedRelatedQuestion) -> Self {
    RepeatedRelatedQuestionPB {
      message_id: value.message_id,
      items: value
        .items
        .into_iter()
        .map(RelatedQuestionPB::from)
        .collect(),
    }
  }
}

#[derive(Debug, Clone, Default, ProtoBuf)]
pub struct LLMModelPB {
  #[pb(index = 1)]
  pub llm_id: i64,

  #[pb(index = 2)]
  pub embedding_model: String,

  #[pb(index = 3)]
  pub chat_model: String,

  #[pb(index = 4)]
  pub requirement: String,

  #[pb(index = 5)]
  pub file_size: i64,
}

impl From<LLMModel> for LLMModelPB {
  fn from(value: LLMModel) -> Self {
    LLMModelPB {
      llm_id: value.llm_id,
      embedding_model: value.embedding_model.name,
      chat_model: value.chat_model.name,
      requirement: value.chat_model.requirements,
      file_size: value.chat_model.file_size,
    }
  }
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct CompleteTextPB {
  #[pb(index = 1)]
  pub text: String,

  #[pb(index = 2)]
  pub completion_type: CompletionTypePB,

  #[pb(index = 3, one_of)]
  pub format: Option<PredefinedFormatPB>,

  #[pb(index = 4)]
  pub stream_port: i64,

  #[pb(index = 5)]
  pub object_id: String,

  #[pb(index = 6)]
  pub rag_ids: Vec<String>,

  #[pb(index = 7)]
  pub history: Vec<CompletionRecordPB>,

  #[pb(index = 8, one_of)]
  pub custom_prompt: Option<String>,

  #[pb(index = 9, one_of)]
  pub prompt_id: Option<String>,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct CompleteTextTaskPB {
  #[pb(index = 1)]
  pub task_id: String,
}

#[derive(Clone, Debug, ProtoBuf_Enum, Default)]
pub enum CompletionTypePB {
  #[default]
  UserQuestion = 0,
  ExplainSelected = 1,
  ContinueWriting = 2,
  SpellingAndGrammar = 3,
  ImproveWriting = 4,
  MakeShorter = 5,
  MakeLonger = 6,
  CustomPrompt = 7,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct CompletionRecordPB {
  #[pb(index = 1)]
  pub role: ChatMessageTypePB,

  #[pb(index = 2)]
  pub content: String,
}

impl From<&CompletionRecordPB> for CompletionMessage {
  fn from(value: &CompletionRecordPB) -> Self {
    CompletionMessage {
      role: match value.role {
        // Coerce ChatMessageTypePB::System to AI
        ChatMessageTypePB::System => "ai".to_string(),
        ChatMessageTypePB::User => "human".to_string(),
      },
      content: value.content.clone(),
    }
  }
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ChatStatePB {
  #[pb(index = 1)]
  pub model_type: ModelTypePB,

  #[pb(index = 2)]
  pub available: bool,
}

#[derive(Clone, Debug, ProtoBuf_Enum, Default)]
pub enum ModelTypePB {
  LocalAI = 0,
  #[default]
  RemoteAI = 1,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ChatFilePB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub file_path: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct LocalModelStatePB {
  #[pb(index = 1)]
  pub model_name: String,

  #[pb(index = 2)]
  pub model_size: String,

  #[pb(index = 3)]
  pub need_download: bool,

  #[pb(index = 4)]
  pub requirements: String,

  #[pb(index = 5)]
  pub is_downloading: bool,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct PendingResourcePB {
  #[pb(index = 1)]
  pub name: String,

  #[pb(index = 2)]
  pub file_size: String,

  #[pb(index = 3)]
  pub requirements: String,

  #[pb(index = 4)]
  pub res_type: PendingResourceTypePB,
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum, PartialEq, Eq, Copy)]
pub enum PendingResourceTypePB {
  #[default]
  LocalAIAppRes = 0,
  ModelRes = 1,
}

impl From<PendingResource> for PendingResourceTypePB {
  fn from(value: PendingResource) -> Self {
    match value {
      PendingResource::PluginExecutableNotReady { .. } => PendingResourceTypePB::LocalAIAppRes,
      _ => PendingResourceTypePB::ModelRes,
    }
  }
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum, PartialEq, Eq, Copy)]
pub enum RunningStatePB {
  #[default]
  ReadyToRun = 0,
  Connecting = 1,
  Connected = 2,
  Running = 3,
  Stopped = 4,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct LocalAIPB {
  #[pb(index = 1)]
  pub enabled: bool,

  #[pb(index = 2, one_of)]
  pub lack_of_resource: Option<LackOfAIResourcePB>,

  #[pb(index = 3)]
  pub is_ready: bool,
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct CreateChatContextPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub content_type: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub text: String,

  #[pb(index = 3)]
  pub metadata: HashMap<String, String>,

  #[pb(index = 4)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub chat_id: String,
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ChatSettingsPB {
  #[pb(index = 1)]
  pub rag_ids: Vec<String>,
}

#[derive(Default, ProtoBuf, Clone, Debug, Validate)]
pub struct UpdateChatSettingsPB {
  #[pb(index = 1)]
  #[validate(nested)]
  pub chat_id: ChatId,

  #[pb(index = 2)]
  pub rag_ids: Vec<String>,
}

#[derive(Debug, Default, Clone, ProtoBuf)]
pub struct PredefinedFormatPB {
  #[pb(index = 1)]
  pub image_format: ResponseImageFormatPB,

  #[pb(index = 2, one_of)]
  pub text_format: Option<ResponseTextFormatPB>,
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum)]
pub enum ResponseImageFormatPB {
  #[default]
  TextOnly = 0,
  ImageOnly = 1,
  TextAndImage = 2,
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum)]
pub enum ResponseTextFormatPB {
  #[default]
  Paragraph = 0,
  BulletedList = 1,
  NumberedList = 2,
  Table = 3,
}

impl From<PredefinedFormatPB> for ResponseFormat {
  fn from(value: PredefinedFormatPB) -> Self {
    Self {
      output_layout: match value.text_format {
        Some(format) => match format {
          ResponseTextFormatPB::Paragraph => OutputLayout::Paragraph,
          ResponseTextFormatPB::BulletedList => OutputLayout::BulletList,
          ResponseTextFormatPB::NumberedList => OutputLayout::NumberedList,
          ResponseTextFormatPB::Table => OutputLayout::SimpleTable,
        },
        None => OutputLayout::Paragraph,
      },
      output_content: match value.image_format {
        ResponseImageFormatPB::TextOnly => OutputContent::TEXT,
        ResponseImageFormatPB::ImageOnly => OutputContent::IMAGE,
        ResponseImageFormatPB::TextAndImage => OutputContent::RichTextImage,
      },
      output_content_metadata: None,
    }
  }
}

#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct LocalAISettingPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub server_url: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub global_chat_model: String,

  #[pb(index = 3)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub embedding_model_name: String,
}

impl From<LocalAISetting> for LocalAISettingPB {
  fn from(value: LocalAISetting) -> Self {
    LocalAISettingPB {
      server_url: value.ollama_server_url,
      global_chat_model: value.chat_model_name,
      embedding_model_name: value.embedding_model_name,
    }
  }
}

impl From<LocalAISettingPB> for LocalAISetting {
  fn from(value: LocalAISettingPB) -> Self {
    LocalAISetting {
      ollama_server_url: value.server_url,
      chat_model_name: value.global_chat_model,
      embedding_model_name: value.embedding_model_name,
    }
  }
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct LackOfAIResourcePB {
  #[pb(index = 1)]
  pub resource_type: LackOfAIResourceTypePB,

  #[pb(index = 2)]
  pub missing_model_names: Vec<String>,
}

#[derive(Debug, Default, Clone, ProtoBuf_Enum)]
pub enum LackOfAIResourceTypePB {
  #[default]
  PluginExecutableNotReady = 0,
  OllamaServerNotReady = 1,
  MissingModel = 2,
}

impl From<PendingResource> for LackOfAIResourcePB {
  fn from(value: PendingResource) -> Self {
    match value {
      PendingResource::PluginExecutableNotReady => Self {
        resource_type: LackOfAIResourceTypePB::PluginExecutableNotReady,
        missing_model_names: vec![],
      },
      PendingResource::OllamaServerNotReady => Self {
        resource_type: LackOfAIResourceTypePB::OllamaServerNotReady,
        missing_model_names: vec![],
      },
      PendingResource::MissingModel(model_name) => Self {
        resource_type: LackOfAIResourceTypePB::MissingModel,
        missing_model_names: vec![model_name],
      },
    }
  }
}

#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct CustomPromptDatabaseViewIdPB {
  #[pb(index = 1)]
  pub id: String,
}

#[derive(Default, ProtoBuf, Clone, Debug, Serialize, Deserialize)]
pub struct CustomPromptDatabaseConfigurationPB {
  #[pb(index = 1)]
  pub view_id: String,

  #[pb(index = 2)]
  pub title_field_id: String,

  #[pb(index = 3)]
  pub content_field_id: String,

  #[pb(index = 4, one_of)]
  pub example_field_id: Option<String>,

  #[pb(index = 5, one_of)]
  pub category_field_id: Option<String>,
}

// ==================== MCP相关实体定义 ====================

/// MCP传输方式枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum)]
pub enum MCPTransportTypePB {
  Stdio = 0,
  SSE = 1,
  HTTP = 2,
}

impl Default for MCPTransportTypePB {
  fn default() -> Self {
    MCPTransportTypePB::Stdio
  }
}

/// STDIO传输配置
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPStdioConfigPB {
  #[pb(index = 1)]
  pub command: String,

  #[pb(index = 2)]
  pub args: Vec<String>,

  #[pb(index = 3)]
  pub env_vars: HashMap<String, String>,
}

/// HTTP/SSE传输配置
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPHttpConfigPB {
  #[pb(index = 1)]
  pub url: String,

  #[pb(index = 2)]
  pub headers: HashMap<String, String>,
}

/// MCP服务器配置
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct MCPServerConfigPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub id: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub name: String,

  #[pb(index = 3)]
  pub icon: String,

  #[pb(index = 4)]
  pub transport_type: MCPTransportTypePB,

  #[pb(index = 5)]
  pub is_active: bool,

  #[pb(index = 6)]
  pub description: String,

  #[pb(index = 7, one_of)]
  pub stdio_config: Option<MCPStdioConfigPB>,

  #[pb(index = 8, one_of)]
  pub http_config: Option<MCPHttpConfigPB>,

  #[pb(index = 9, one_of)]
  pub cached_tools: Option<MCPToolListPB>,

  #[pb(index = 10, one_of)]
  pub last_tools_check_at: Option<i64>,  // Unix timestamp in seconds
}

/// MCP服务器列表响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPServerListPB {
  #[pb(index = 1)]
  pub servers: Vec<MCPServerConfigPB>,
}

/// MCP工具注解
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPToolAnnotationsPB {
  #[pb(index = 1, one_of)]
  pub title: Option<String>,

  #[pb(index = 2, one_of)]
  pub read_only_hint: Option<bool>,

  #[pb(index = 3, one_of)]
  pub destructive_hint: Option<bool>,

  #[pb(index = 4, one_of)]
  pub idempotent_hint: Option<bool>,

  #[pb(index = 5, one_of)]
  pub open_world_hint: Option<bool>,
}

/// MCP工具定义
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPToolPB {
  #[pb(index = 1)]
  pub name: String,

  #[pb(index = 2)]
  pub description: String,

  #[pb(index = 3)]
  pub input_schema: String, // JSON字符串

  #[pb(index = 4, one_of)]
  pub annotations: Option<MCPToolAnnotationsPB>,
}

/// MCP工具列表响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPToolListPB {
  #[pb(index = 1)]
  pub tools: Vec<MCPToolPB>,
}

/// MCP工具调用请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct MCPToolCallRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub server_id: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub tool_name: String,

  #[pb(index = 3)]
  pub arguments: String, // JSON字符串
}

/// MCP内容类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum)]
pub enum MCPContentTypePB {
  Text = 0,
  Image = 1,
  Resource = 2,
}

impl Default for MCPContentTypePB {
  fn default() -> Self {
    MCPContentTypePB::Text
  }
}

/// MCP内容项
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPContentPB {
  #[pb(index = 1)]
  pub content_type: MCPContentTypePB,

  #[pb(index = 2)]
  pub text: String,

  #[pb(index = 3, one_of)]
  pub mime_type: Option<String>,
}

/// MCP工具调用响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPToolCallResponsePB {
  #[pb(index = 1)]
  pub success: bool,

  #[pb(index = 2)]
  pub content: Vec<MCPContentPB>,

  #[pb(index = 3, one_of)]
  pub error: Option<String>,
}

/// MCP服务器连接请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct MCPConnectServerRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub server_id: String,
}

/// MCP服务器断开连接请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct MCPDisconnectServerRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub server_id: String,
}

/// MCP服务器状态响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct MCPServerStatusPB {
  #[pb(index = 1)]
  pub server_id: String,

  #[pb(index = 2)]
  pub is_connected: bool,

  #[pb(index = 3, one_of)]
  pub error_message: Option<String>,

  #[pb(index = 4)]
  pub tool_count: i32,
}

// ==================== 智能体配置相关实体 ====================

/// 智能体配置
#[derive(Default, ProtoBuf, Validate, Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfigPB {
  /// 智能体唯一标识符
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub id: String,

  /// 智能体名称
  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub name: String,

  /// 智能体描述
  #[pb(index = 3)]
  pub description: String,

  /// 智能体头像/图标
  #[pb(index = 4)]
  pub avatar: String,

  /// 智能体个性描述（系统提示词）
  #[pb(index = 5)]
  pub personality: String,

  /// 智能体能力配置
  #[pb(index = 6)]
  pub capabilities: AgentCapabilitiesPB,

  /// 可用工具列表
  #[pb(index = 7)]
  pub available_tools: Vec<String>,

  /// 智能体状态
  #[pb(index = 8)]
  pub status: AgentStatusPB,

  /// 创建时间（时间戳）
  #[pb(index = 9)]
  pub created_at: i64,

  /// 更新时间（时间戳）
  #[pb(index = 10)]
  pub updated_at: i64,

  /// 智能体配置元数据
  #[pb(index = 11)]
  pub metadata: HashMap<String, String>,
}

/// 智能体能力配置
#[derive(Default, ProtoBuf, Clone, Debug, Serialize, Deserialize)]
pub struct AgentCapabilitiesPB {
  /// 是否启用任务规划
  #[pb(index = 1)]
  pub enable_planning: bool,

  /// 是否启用工具调用
  #[pb(index = 2)]
  pub enable_tool_calling: bool,

  /// 是否启用反思机制
  #[pb(index = 3)]
  pub enable_reflection: bool,

  /// 是否启用会话记忆
  #[pb(index = 4)]
  pub enable_memory: bool,

  /// 最大规划步骤数
  #[pb(index = 5)]
  pub max_planning_steps: i32,

  /// 最大工具调用次数
  #[pb(index = 6)]
  pub max_tool_calls: i32,

  /// 会话记忆长度限制
  #[pb(index = 7)]
  pub memory_limit: i32,

  /// 工具结果最大长度（字符数）
  /// 用于多轮对话时控制上下文长度，避免超出模型限制
  /// 默认 4000 字符，最小 1000 字符
  #[pb(index = 8)]
  pub max_tool_result_length: i32,
  
  /// 最大反思迭代次数
  /// 当启用反思机制时，智能体可以多次调用工具直到问题解决
  /// 默认 3 次，最大 10 次，设为 0 则禁用反思
  #[pb(index = 9)]
  pub max_reflection_iterations: i32,
}

/// 智能体状态枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum AgentStatusPB {
  /// 活跃状态
  AgentActive = 0,
  /// 暂停状态
  AgentPaused = 1,
  /// 已删除状态
  AgentDeleted = 2,
}

impl Default for AgentStatusPB {
  fn default() -> Self {
    AgentStatusPB::AgentActive
  }
}

/// 智能体列表响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AgentListPB {
  #[pb(index = 1)]
  pub agents: Vec<AgentConfigPB>,
}

/// 创建智能体请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct CreateAgentRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub name: String,

  #[pb(index = 2)]
  pub description: String,

  #[pb(index = 3)]
  pub avatar: String,

  #[pb(index = 4)]
  pub personality: String,

  #[pb(index = 5)]
  pub capabilities: AgentCapabilitiesPB,

  #[pb(index = 6)]
  pub available_tools: Vec<String>,

  #[pb(index = 7)]
  pub metadata: HashMap<String, String>,
}

/// 更新智能体请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct UpdateAgentRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub id: String,

  #[pb(index = 2, one_of)]
  pub name: Option<String>,

  #[pb(index = 3, one_of)]
  pub description: Option<String>,

  #[pb(index = 4, one_of)]
  pub avatar: Option<String>,

  #[pb(index = 5, one_of)]
  pub personality: Option<String>,

  #[pb(index = 6, one_of)]
  pub capabilities: Option<AgentCapabilitiesPB>,

  #[pb(index = 7)]
  pub available_tools: Vec<String>,

  #[pb(index = 8, one_of)]
  pub status: Option<AgentStatusPB>,

  #[pb(index = 9)]
  pub metadata: HashMap<String, String>,
}

/// 删除智能体请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct DeleteAgentRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub id: String,
}

/// 获取智能体请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct GetAgentRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub id: String,
}

// ==================== 智能体会话相关实体 ====================

/// 智能体会话
#[derive(Default, ProtoBuf, Validate, Clone, Debug, Serialize, Deserialize)]
pub struct AgentSessionPB {
  /// 会话唯一标识符
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub id: String,

  /// 关联的智能体ID
  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub agent_id: String,

  /// 会话标题
  #[pb(index = 3)]
  pub title: String,

  /// 会话状态
  #[pb(index = 4)]
  pub status: SessionStatusPB,

  /// 创建时间（时间戳）
  #[pb(index = 5)]
  pub created_at: i64,

  /// 更新时间（时间戳）
  #[pb(index = 6)]
  pub updated_at: i64,

  /// 会话元数据
  #[pb(index = 7)]
  pub metadata: HashMap<String, String>,
}

/// 会话状态枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum SessionStatusPB {
  /// 活跃状态
  SessionActive = 0,
  /// 已完成状态
  SessionCompleted = 1,
  /// 已暂停状态
  SessionPaused = 2,
  /// 已关闭状态
  SessionClosed = 3,
}

impl Default for SessionStatusPB {
  fn default() -> Self {
    SessionStatusPB::SessionActive
  }
}

/// 智能体消息
#[derive(Default, ProtoBuf, Clone, Debug, Serialize, Deserialize)]
pub struct AgentMessagePB {
  /// 消息唯一标识符
  #[pb(index = 1)]
  pub id: String,

  /// 会话ID
  #[pb(index = 2)]
  pub session_id: String,

  /// 消息类型
  #[pb(index = 3)]
  pub message_type: AgentMessageTypePB,

  /// 消息内容
  #[pb(index = 4)]
  pub content: String,

  /// 发送者ID（用户ID或智能体ID）
  #[pb(index = 5)]
  pub sender_id: String,

  /// 创建时间（时间戳）
  #[pb(index = 6)]
  pub created_at: i64,

  /// 消息元数据
  #[pb(index = 7)]
  pub metadata: HashMap<String, String>,

  /// 关联的执行日志ID（如果有）
  #[pb(index = 8, one_of)]
  pub execution_log_id: Option<String>,
}

/// 智能体消息类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum AgentMessageTypePB {
  /// 用户消息
  AgentUser = 0,
  /// 智能体消息
  Agent = 1,
  /// 系统消息
  AgentSystem = 2,
  /// 工具调用消息
  AgentToolCall = 3,
  /// 工具响应消息
  AgentToolResponse = 4,
}

impl Default for AgentMessageTypePB {
  fn default() -> Self {
    AgentMessageTypePB::AgentUser
  }
}

/// 会话消息列表
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AgentMessageListPB {
  #[pb(index = 1)]
  pub messages: Vec<AgentMessagePB>,

  #[pb(index = 2)]
  pub has_more: bool,

  #[pb(index = 3)]
  pub total: i64,
}

/// 创建会话请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct CreateSessionRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub agent_id: String,

  #[pb(index = 2)]
  pub title: String,

  #[pb(index = 3)]
  pub metadata: HashMap<String, String>,
}

/// 发送消息请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct SendAgentMessageRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub session_id: String,

  #[pb(index = 2)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub content: String,

  #[pb(index = 3)]
  pub message_type: AgentMessageTypePB,

  #[pb(index = 4)]
  pub metadata: HashMap<String, String>,
}

// ==================== 智能体执行日志相关实体 ====================

/// 智能体执行日志
#[derive(Default, ProtoBuf, Validate, Clone, Debug, Serialize, Deserialize)]
pub struct AgentExecutionLogPB {
  /// 日志唯一标识符
  #[pb(index = 1)]
  pub id: String,

  /// 会话ID
  #[pb(index = 2)]
  pub session_id: String,

  /// 消息ID
  #[pb(index = 3)]
  pub message_id: String,

  /// 执行阶段
  #[pb(index = 4)]
  pub phase: ExecutionPhasePB,

  /// 执行步骤
  #[pb(index = 5)]
  pub step: String,

  /// 输入数据
  #[pb(index = 6)]
  pub input: String,

  /// 输出数据
  #[pb(index = 7)]
  pub output: String,

  /// 执行状态
  #[pb(index = 8)]
  pub status: ExecutionStatusPB,

  /// 开始时间（时间戳）
  #[pb(index = 9)]
  pub started_at: i64,

  /// 结束时间（时间戳）
  #[pb(index = 10, one_of)]
  pub completed_at: Option<i64>,

  /// 执行耗时（毫秒）
  #[pb(index = 11)]
  pub duration_ms: i64,

  /// 错误信息（如果有）
  #[pb(index = 12, one_of)]
  pub error_message: Option<String>,

  /// 日志元数据
  #[pb(index = 13)]
  pub metadata: HashMap<String, String>,
}

/// 执行阶段枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum ExecutionPhasePB {
  /// 规划阶段
  ExecPlanning = 0,
  /// 执行阶段
  ExecExecution = 1,
  /// 工具调用阶段
  ExecToolCall = 2,
  /// 反思阶段
  ExecReflection = 3,
  /// 完成阶段
  ExecCompletion = 4,
}

impl Default for ExecutionPhasePB {
  fn default() -> Self {
    ExecutionPhasePB::ExecPlanning
  }
}

/// 执行状态枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum ExecutionStatusPB {
  /// 进行中
  ExecRunning = 0,
  /// 成功完成
  ExecSuccess = 1,
  /// 失败
  ExecFailed = 2,
  /// 已取消
  ExecCancelled = 3,
}

impl Default for ExecutionStatusPB {
  fn default() -> Self {
    ExecutionStatusPB::ExecRunning
  }
}

/// 执行日志列表
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AgentExecutionLogListPB {
  #[pb(index = 1)]
  pub logs: Vec<AgentExecutionLogPB>,

  #[pb(index = 2)]
  pub has_more: bool,

  #[pb(index = 3)]
  pub total: i64,
}

/// 获取执行日志请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct GetExecutionLogsRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub session_id: String,

  #[pb(index = 2, one_of)]
  pub message_id: Option<String>,

  #[pb(index = 3, one_of)]
  pub phase: Option<ExecutionPhasePB>,

  #[pb(index = 4)]
  pub limit: i32,

  #[pb(index = 5)]
  pub offset: i32,
}

/// 清空执行日志请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ClearExecutionLogsRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub session_id: String,

  #[pb(index = 2, one_of)]
  pub message_id: Option<String>,
}

// ==================== 智能体任务规划相关实体 ====================

/// 智能体任务计划
#[derive(Default, ProtoBuf, Clone, Debug, Serialize, Deserialize)]
pub struct AgentTaskPlanPB {
  /// 计划唯一标识符
  #[pb(index = 1)]
  pub id: String,

  /// 会话ID
  #[pb(index = 2)]
  pub session_id: String,

  /// 用户问题/目标
  #[pb(index = 3)]
  pub user_goal: String,

  /// 任务步骤列表
  #[pb(index = 4)]
  pub steps: Vec<TaskStepPB>,

  /// 计划状态
  #[pb(index = 5)]
  pub status: PlanStatusPB,

  /// 创建时间（时间戳）
  #[pb(index = 6)]
  pub created_at: i64,

  /// 更新时间（时间戳）
  #[pb(index = 7)]
  pub updated_at: i64,
}

/// 任务步骤
#[derive(Default, ProtoBuf, Clone, Debug, Serialize, Deserialize)]
pub struct TaskStepPB {
  /// 步骤唯一标识符
  #[pb(index = 1)]
  pub id: String,

  /// 步骤序号
  #[pb(index = 2)]
  pub order: i32,

  /// 步骤描述
  #[pb(index = 3)]
  pub description: String,

  /// 需要使用的工具
  #[pb(index = 4, one_of)]
  pub tool_name: Option<String>,

  /// 工具参数
  #[pb(index = 5)]
  pub tool_arguments: HashMap<String, String>,

  /// 步骤状态
  #[pb(index = 6)]
  pub status: StepStatusPB,

  /// 执行结果
  #[pb(index = 7, one_of)]
  pub result: Option<String>,

  /// 错误信息（如果有）
  #[pb(index = 8, one_of)]
  pub error_message: Option<String>,
}

/// 计划状态枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum PlanStatusPB {
  /// 待执行
  PlanPending = 0,
  /// 执行中
  PlanExecuting = 1,
  /// 已完成
  PlanCompleted = 2,
  /// 已失败
  PlanFailed = 3,
  /// 已取消
  PlanCancelled = 4,
}

impl Default for PlanStatusPB {
  fn default() -> Self {
    PlanStatusPB::PlanPending
  }
}

/// 步骤状态枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum StepStatusPB {
  /// 待执行
  StepPending = 0,
  /// 执行中
  StepRunning = 1,
  /// 已完成
  StepCompleted = 2,
  /// 已失败
  StepFailed = 3,
  /// 已跳过
  StepSkipped = 4,
}

impl Default for StepStatusPB {
  fn default() -> Self {
    StepStatusPB::StepPending
  }
}

// ==================== 工具注册表相关实体 ====================

/// 工具定义
#[derive(Default, ProtoBuf, Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinitionPB {
  /// 工具名称
  #[pb(index = 1)]
  pub name: String,

  /// 工具描述
  #[pb(index = 2)]
  pub description: String,

  /// 工具类型
  #[pb(index = 3)]
  pub tool_type: ToolTypePB,

  /// 工具来源（MCP服务器ID或内置标识）
  #[pb(index = 4)]
  pub source: String,

  /// 工具参数schema（JSON字符串）
  #[pb(index = 5)]
  pub parameters_schema: String,

  /// 工具权限要求
  #[pb(index = 6)]
  pub permissions: Vec<String>,

  /// 是否可用
  #[pb(index = 7)]
  pub is_available: bool,

  /// 工具元数据
  #[pb(index = 8)]
  pub metadata: HashMap<String, String>,
}

/// 工具类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, ProtoBuf_Enum, Serialize, Deserialize)]
pub enum ToolTypePB {
  /// MCP工具
  MCP = 0,
  /// AppFlowy原生工具
  Native = 1,
  /// 搜索工具
  Search = 2,
  /// 外部API工具
  ExternalAPI = 3,
}

impl Default for ToolTypePB {
  fn default() -> Self {
    ToolTypePB::Native
  }
}

/// 工具列表响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ToolListPB {
  #[pb(index = 1)]
  pub tools: Vec<ToolDefinitionPB>,
}

/// 工具调用请求
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct ToolCallRequestPB {
  #[pb(index = 1)]
  #[validate(custom(function = "required_not_empty_str"))]
  pub tool_name: String,

  #[pb(index = 2)]
  pub arguments: HashMap<String, String>,

  #[pb(index = 3, one_of)]
  pub session_id: Option<String>,

  #[pb(index = 4)]
  pub metadata: HashMap<String, String>,
}

/// 工具调用响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct ToolCallResponsePB {
  #[pb(index = 1)]
  pub success: bool,

  #[pb(index = 2)]
  pub result: String,

  #[pb(index = 3, one_of)]
  pub error_message: Option<String>,

  #[pb(index = 4)]
  pub execution_time_ms: i64,

  #[pb(index = 5)]
  pub metadata: HashMap<String, String>,
}

// ==================== 通用响应实体 ====================

/// 通用成功响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AgentSuccessResponsePB {
  #[pb(index = 1)]
  pub success: bool,

  #[pb(index = 2, one_of)]
  pub message: Option<String>,

  #[pb(index = 3)]
  pub data: HashMap<String, String>,
}

/// 通用错误响应
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct AgentErrorResponsePB {
  #[pb(index = 1)]
  pub error_code: String,

  #[pb(index = 2)]
  pub error_message: String,

  #[pb(index = 3)]
  pub details: HashMap<String, String>,
}

/// 智能体全局设置
#[derive(Default, ProtoBuf, Validate, Clone, Debug)]
pub struct AgentGlobalSettingsPB {
  /// 是否启用智能体功能
  #[pb(index = 1)]
  pub enabled: bool,

  /// 默认最大规划步骤数
  #[pb(index = 2)]
  pub default_max_planning_steps: i32,

  /// 默认最大工具调用次数
  #[pb(index = 3)]
  pub default_max_tool_calls: i32,

  /// 默认会话记忆长度限制
  #[pb(index = 4)]
  pub default_memory_limit: i32,

  /// 是否启用调试日志
  #[pb(index = 5)]
  pub debug_logging: bool,

  /// 智能体执行超时时间（秒）
  #[pb(index = 6)]
  pub execution_timeout: u64,

  /// 创建时间
  #[pb(index = 7)]
  pub created_at: i64,

  /// 更新时间
  #[pb(index = 8)]
  pub updated_at: i64,
}

// ==================== 实用工具函数和转换 ====================

impl AgentConfigPB {
  /// 创建新的智能体配置
  pub fn new(name: String, description: String) -> Self {
    let now = Utc::now().timestamp();
    Self {
      id: Uuid::new_v4().to_string(),
      name,
      description,
      avatar: String::new(),
      personality: String::new(),
      capabilities: AgentCapabilitiesPB::default(),
      available_tools: Vec::new(),
      status: AgentStatusPB::AgentActive,
      created_at: now,
      updated_at: now,
      metadata: HashMap::new(),
    }
  }

  /// 检查智能体是否活跃
  pub fn is_active(&self) -> bool {
    self.status == AgentStatusPB::AgentActive
  }
}

impl AgentCapabilitiesPB {
  /// 创建默认能力配置
  pub fn default_capabilities() -> Self {
    Self {
      enable_planning: true,
      enable_tool_calling: true,
      enable_reflection: true,
      enable_memory: true,
      max_planning_steps: 10,
      max_tool_calls: 20,
      memory_limit: 100,
      max_tool_result_length: 4000,
      max_reflection_iterations: 3,
    }
  }
}

impl AgentSessionPB {
  /// 创建新的会话
  pub fn new(agent_id: String, title: String) -> Self {
    let now = Utc::now().timestamp();
    Self {
      id: Uuid::new_v4().to_string(),
      agent_id,
      title,
      status: SessionStatusPB::SessionActive,
      created_at: now,
      updated_at: now,
      metadata: HashMap::new(),
    }
  }

  /// 检查会话是否活跃
  pub fn is_active(&self) -> bool {
    matches!(self.status, SessionStatusPB::SessionActive | SessionStatusPB::SessionPaused)
  }
}

impl AgentMessagePB {
  /// 创建新的消息
  pub fn new(
    session_id: String,
    message_type: AgentMessageTypePB,
    content: String,
    sender_id: String,
  ) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      session_id,
      message_type,
      content,
      sender_id,
      created_at: Utc::now().timestamp(),
      metadata: HashMap::new(),
      execution_log_id: None,
    }
  }
}

impl AgentExecutionLogPB {
  /// 创建新的执行日志
  pub fn new(
    session_id: String,
    message_id: String,
    phase: ExecutionPhasePB,
    step: String,
  ) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      session_id,
      message_id,
      phase,
      step,
      input: String::new(),
      output: String::new(),
      status: ExecutionStatusPB::ExecRunning,
      started_at: Utc::now().timestamp(),
      completed_at: None,
      duration_ms: 0,
      error_message: None,
      metadata: HashMap::new(),
    }
  }

  /// 标记日志为完成状态
  pub fn mark_completed(&mut self, output: String) {
    let now = Utc::now().timestamp();
    self.output = output;
    self.status = ExecutionStatusPB::ExecSuccess;
    self.completed_at = Some(now);
    if let Some(started_at) = Some(self.started_at) {
      self.duration_ms = (now - started_at) * 1000;
    }
  }

  /// 标记日志为失败状态
  pub fn mark_failed(&mut self, error_message: String) {
    let now = Utc::now().timestamp();
    self.error_message = Some(error_message);
    self.status = ExecutionStatusPB::ExecFailed;
    self.completed_at = Some(now);
    if let Some(started_at) = Some(self.started_at) {
      self.duration_ms = (now - started_at) * 1000;
    }
  }
}
