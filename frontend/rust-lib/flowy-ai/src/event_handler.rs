use crate::ai_manager::AIManager;
use crate::completion::AICompletion;
use crate::entities::*;
use flowy_ai_pub::cloud::{AIModel, ChatMessageType};
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use lib_dispatch::prelude::{AFPluginData, AFPluginState, DataResult, data_result_ok};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Weak};
use tracing::{trace, info};
use uuid::Uuid;
use validator::Validate;

fn upgrade_ai_manager(ai_manager: AFPluginState<Weak<AIManager>>) -> FlowyResult<Arc<AIManager>> {
  let ai_manager = ai_manager
    .upgrade()
    .ok_or(FlowyError::internal().with_context("The chat manager is already dropped"))?;
  Ok(ai_manager)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn stream_chat_message_handler(
  data: AFPluginData<StreamChatPayloadPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ChatMessagePB, FlowyError> {
  let data = data.into_inner();
  data.validate()?;

  let StreamChatPayloadPB {
    chat_id,
    message,
    message_type,
    answer_stream_port,
    question_stream_port,
    format,
    prompt_id,
    agent_id,
  } = data;

  let message_type = match message_type {
    ChatMessageTypePB::System => ChatMessageType::System,
    ChatMessageTypePB::User => ChatMessageType::User,
  };

  let chat_id = Uuid::from_str(&chat_id)?;
  
  trace!("🔧 [HANDLER] About to call ai_manager.stream_chat_message: chat_id={}, message='{}', agent_id={:?}", 
        chat_id, message, agent_id);
  
  let params = StreamMessageParams {
    chat_id,
    message,
    message_type,
    answer_stream_port,
    question_stream_port,
    format,
    prompt_id,
    agent_id,
  };

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 添加调试信息到返回的消息中
  let mut debug_result = ai_manager.stream_chat_message(params).await?;
  debug_result.content = format!("🔧 [DEBUG] Handler called successfully: {}", debug_result.content);
  
  trace!("🔧 [HANDLER] ai_manager.stream_chat_message completed successfully");
  data_result_ok(debug_result)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn regenerate_response_handler(
  data: AFPluginData<RegenerateResponsePB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  let chat_id = Uuid::from_str(&data.chat_id)?;

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager
    .stream_regenerate_response(
      &chat_id,
      data.answer_message_id,
      data.answer_stream_port,
      data.format,
      data.model,
    )
    .await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_setting_model_selection_handler(
  data: AFPluginData<ModelSourcePB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ModelSelectionPB, FlowyError> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let models = ai_manager.get_available_models(data.source, true).await?;
  data_result_ok(models)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_source_model_selection_handler(
  data: AFPluginData<ModelSourcePB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ModelSelectionPB, FlowyError> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let models = ai_manager.get_available_models(data.source, false).await?;
  data_result_ok(models)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_selected_model_handler(
  data: AFPluginData<UpdateSelectedModelPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager
    .update_selected_model(data.source, AIModel::from(data.selected_model))
    .await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn load_prev_message_handler(
  data: AFPluginData<LoadPrevChatMessagePB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ChatMessageListPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let data = data.into_inner();
  data.validate()?;

  let chat_id = Uuid::from_str(&data.chat_id)?;
  let messages = ai_manager
    .load_prev_chat_messages(&chat_id, data.limit as u64, data.before_message_id)
    .await?;
  data_result_ok(messages)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn load_next_message_handler(
  data: AFPluginData<LoadNextChatMessagePB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ChatMessageListPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let data = data.into_inner();
  data.validate()?;

  let chat_id = Uuid::from_str(&data.chat_id)?;
  let messages = ai_manager
    .load_latest_chat_messages(&chat_id, data.limit as u64, data.after_message_id)
    .await?;
  data_result_ok(messages)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_related_question_handler(
  data: AFPluginData<ChatMessageIdPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<RepeatedRelatedQuestionPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let data = data.into_inner();
  let chat_id = Uuid::from_str(&data.chat_id)?;
  let messages = ai_manager
    .get_related_questions(&chat_id, data.message_id)
    .await?;
  data_result_ok(messages)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_answer_handler(
  data: AFPluginData<ChatMessageIdPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ChatMessagePB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let data = data.into_inner();
  let chat_id = Uuid::from_str(&data.chat_id)?;
  let message = ai_manager
    .generate_answer(&chat_id, data.message_id)
    .await?;
  data_result_ok(message)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn stop_stream_handler(
  data: AFPluginData<StopStreamPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let data = data.into_inner();
  data.validate()?;

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let chat_id = Uuid::from_str(&data.chat_id)?;
  ai_manager.stop_stream(&chat_id).await?;
  Ok(())
}

pub(crate) async fn start_complete_text_handler(
  data: AFPluginData<CompleteTextPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
  tools: AFPluginState<Arc<AICompletion>>,
) -> DataResult<CompleteTextTaskPB, FlowyError> {
  let data = data.into_inner();
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let ai_model = ai_manager.get_active_model(&data.object_id).await;
  let task = tools.create_complete_task(data, ai_model).await?;
  data_result_ok(task)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn stop_complete_text_handler(
  data: AFPluginData<CompleteTextTaskPB>,
  tools: AFPluginState<Arc<AICompletion>>,
) -> Result<(), FlowyError> {
  let data = data.into_inner();
  tools.cancel_complete_task(&data.task_id).await;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn chat_file_handler(
  data: AFPluginData<ChatFilePB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let data = data.try_into_inner()?;
  let file_path = PathBuf::from(&data.file_path);

  let allowed_extensions = ["pdf", "md", "txt"];
  let extension = file_path
    .extension()
    .and_then(|ext| ext.to_str())
    .ok_or_else(|| {
      FlowyError::new(
        ErrorCode::UnsupportedFileFormat,
        "Can't find file extension",
      )
    })?;

  if !allowed_extensions.contains(&extension) {
    return Err(FlowyError::new(
      ErrorCode::UnsupportedFileFormat,
      "Only support pdf,md and txt",
    ));
  }
  let file_size = fs::metadata(&file_path)
    .map_err(|_| {
      FlowyError::new(
        ErrorCode::UnsupportedFileFormat,
        "Failed to get file metadata",
      )
    })?
    .len();

  const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
  if file_size > MAX_FILE_SIZE {
    return Err(FlowyError::new(
      ErrorCode::PayloadTooLarge,
      "File size is too large. Max file size is 10MB",
    ));
  }

  tracing::debug!("File size: {} bytes", file_size);
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let chat_id = Uuid::from_str(&data.chat_id)?;
  ai_manager.chat_with_file(&chat_id, file_path).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn restart_local_ai_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.local_ai.restart_plugin().await;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn toggle_local_ai_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<LocalAIPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.toggle_local_ai().await?;
  let state = ai_manager.local_ai.get_local_ai_state().await;
  data_result_ok(state)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_local_ai_state_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<LocalAIPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let state = ai_manager.local_ai.get_local_ai_state().await;
  data_result_ok(state)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn create_chat_context_handler(
  data: AFPluginData<CreateChatContextPB>,
  _ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let _data = data.try_into_inner()?;

  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_chat_info_handler(
  data: AFPluginData<ChatId>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ChatInfoPB, FlowyError> {
  let chat_id = data.try_into_inner()?.value;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let pb = ai_manager.get_chat_info(&chat_id).await?;
  data_result_ok(pb)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_chat_settings_handler(
  data: AFPluginData<ChatId>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ChatSettingsPB, FlowyError> {
  let chat_id = data.try_into_inner()?.value;
  let chat_id = Uuid::from_str(&chat_id)?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let uid = ai_manager.user_service.user_id()?;
  let mut conn = ai_manager.user_service.sqlite_connection(uid)?;
  let rag_ids = ai_manager.get_rag_ids(&chat_id, &mut conn).await?;
  let pb = ChatSettingsPB { rag_ids };
  data_result_ok(pb)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_chat_settings_handler(
  data: AFPluginData<UpdateChatSettingsPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let params = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let chat_id = Uuid::from_str(&params.chat_id.value)?;
  ai_manager.update_rag_ids(&chat_id, params.rag_ids).await?;

  Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn get_local_ai_setting_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<LocalAISettingPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let setting = ai_manager.local_ai.get_local_ai_setting();
  let pb = LocalAISettingPB::from(setting);
  data_result_ok(pb)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn get_local_ai_models_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<ModelSelectionPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let data = ai_manager.get_local_available_models(None).await?;
  data_result_ok(data)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_local_ai_setting_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
  data: AFPluginData<LocalAISettingPB>,
) -> Result<(), FlowyError> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.update_local_ai_setting(data.into()).await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_custom_prompt_database_configuration_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<CustomPromptDatabaseConfigurationPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let configuration = ai_manager
    .get_custom_prompt_database_configuration()
    .await?
    .ok_or_else(|| {
      FlowyError::new(
        ErrorCode::RecordNotFound,
        "Custom prompt configuration not found",
      )
    })?;

  data_result_ok(configuration)
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn set_custom_prompt_database_configuration_handler(
  data: AFPluginData<CustomPromptDatabaseConfigurationPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let config = data.into_inner();

  ai_manager
    .set_custom_prompt_database_configuration(config)
    .await?;

  Ok(())
}

// ==================== 向量索引管理 ====================

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn rebuild_vector_index_handler(
  data: AFPluginData<RebuildVectorIndexRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<RebuildVectorIndexResponsePB, FlowyError> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let workspace_id = ai_manager.user_service.workspace_id()?;
  let document_ids = if data.document_ids.is_empty() {
    None
  } else {
    Some(data.document_ids)
  };
  
  match ai_manager
    .vector_index_manager
    .rebuild_index(workspace_id, document_ids)
    .await
  {
    Ok(_) => data_result_ok(RebuildVectorIndexResponsePB {
      success: true,
      error: None,
    }),
    Err(err) => data_result_ok(RebuildVectorIndexResponsePB {
      success: false,
      error: Some(err.to_string()),
    }),
  }
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_vector_index_status_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<VectorIndexStatusPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let manager = &ai_manager.vector_index_manager;
  
  let state = match manager.get_state().await {
    crate::vector_index_manager::VectorIndexState::Idle => VectorIndexStatePB::IndexIdle,
    crate::vector_index_manager::VectorIndexState::Running => VectorIndexStatePB::IndexRunning,
    crate::vector_index_manager::VectorIndexState::Completed => VectorIndexStatePB::IndexCompleted,
    crate::vector_index_manager::VectorIndexState::Failed => VectorIndexStatePB::IndexFailed,
    crate::vector_index_manager::VectorIndexState::Stopping => VectorIndexStatePB::IndexStopping,
  };
  
  data_result_ok(VectorIndexStatusPB {
    state,
    total_documents: manager.get_total_documents(),
    indexed_documents: manager.get_indexed_documents(),
    recent_logs: manager.get_recent_logs().await,
    error: manager.get_error().await,
    start_time: manager.get_start_time(),
    last_update_time: manager.get_last_update_time(),
  })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn stop_vector_indexing_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.vector_index_manager.stop_indexing().await;
  Ok(())
}

// ==================== 向量数据库重置管理 ====================

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn reset_vector_database_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.reset_vector_database().await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn smart_reset_vector_database_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.smart_reset_vector_database().await?;
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn manual_reset_vector_database_handler(
  data: AFPluginData<ManualResetVectorDatabaseRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  use crate::embeddings::context::EmbedContext;
  
  let embedding_dimension = data.embedding_dimension as usize;
  EmbedContext::shared()
    .rebuild_vector_database(embedding_dimension)
    .await?;
  
  Ok(())
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_current_embedding_dimension_handler(
  _ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<EmbeddingDimensionPB, FlowyError> {
  
  use crate::embeddings::context::EmbedContext;
  
  let dimension = EmbedContext::shared()
    .get_current_embedding_dimension()?;
  
  // 获取模型信息
  let model_name = if let Some(config) = EmbedContext::shared().get_openai_config() {
    config.model.clone()
  } else {
    "ollama-nomic-embed-text".to_string()
  };
  
  let is_openai_compatible = EmbedContext::shared()
    .get_openai_config()
    .is_some();
  
  data_result_ok(EmbeddingDimensionPB {
    dimension: dimension as u32,
    model_name,
    is_openai_compatible,
  })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn test_embedding_model_handler(
  data: AFPluginData<TestEmbeddingModelRequestPB>,
  _ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<TestEmbeddingModelResponsePB, FlowyError> {
  let data = data.try_into_inner()?;
  
  use crate::embeddings::context::test_embedding_model_dimension;
  
  match test_embedding_model_dimension(
    &data.base_url,
    &data.api_key,
    &data.model,
  ).await {
    Ok(dimension) => {
      data_result_ok(TestEmbeddingModelResponsePB {
        success: true,
        dimension: dimension as u32,
        error: None,
        model_name: data.model.clone(),
      })
    },
    Err(e) => {
      data_result_ok(TestEmbeddingModelResponsePB {
        success: false,
        dimension: 0,
        error: Some(e.to_string()),
        model_name: data.model.clone(),
      })
    }
  }
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn check_dimension_compatibility_handler(
  _ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<DimensionCompatibilityPB, FlowyError> {
  
  use crate::embeddings::context::EmbedContext;
  
  match EmbedContext::shared().check_dimension_compatibility().await {
    Ok(result) => {
      data_result_ok(DimensionCompatibilityPB {
        current_db_dimension: result.current_db_dimension as u32,
        model_dimension: result.model_dimension as u32,
        is_compatible: result.is_compatible,
        model_name: result.model_name,
        error: None,
      })
    },
    Err(e) => {
      data_result_ok(DimensionCompatibilityPB {
        current_db_dimension: 0,
        model_dimension: 0,
        is_compatible: false,
        model_name: String::new(),
        error: Some(e.to_string()),
      })
    }
  }
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn smart_reset_vector_database_with_test_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  use crate::embeddings::context::EmbedContext;
  
  info!("[AI Manager] 🔄 开始智能重置向量数据库（包含测试）...");
  
  // 获取智能检测的维度
  let smart_dimension = EmbedContext::shared()
    .get_smart_embedding_dimension()
    .await?;
  
  info!("[AI Manager] 📏 智能检测到的维度: {}", smart_dimension);
  
  // 重建向量数据库以匹配新维度
  EmbedContext::shared()
    .rebuild_vector_database(smart_dimension)
    .await?;
  
  info!("[AI Manager] ✅ 向量数据库智能重置完成，新维度: {}", smart_dimension);
  Ok(())
}
