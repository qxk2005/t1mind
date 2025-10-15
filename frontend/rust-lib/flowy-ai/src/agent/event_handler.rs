use crate::ai_manager::AIManager;
use crate::entities::*;
use flowy_error::{FlowyError, FlowyResult};
use lib_dispatch::prelude::{AFPluginData, AFPluginState, DataResult, data_result_ok};
use std::sync::{Arc, Weak};
use std::time::Instant;
use validator::Validate;
use tracing::{debug, error, info, warn, trace};

/// 升级AI管理器弱引用为强引用，包含详细的错误处理
fn upgrade_ai_manager(ai_manager: AFPluginState<Weak<AIManager>>) -> FlowyResult<Arc<AIManager>> {
  let ai_manager = ai_manager
    .upgrade()
    .ok_or_else(|| {
      error!("AI manager has been dropped, cannot process Agent request");
      FlowyError::internal().with_context("The AI manager is already dropped")
    })?;
  trace!("Successfully upgraded AI manager reference");
  Ok(ai_manager)
}

/// 记录操作性能指标的辅助函数
fn log_operation_duration(operation: &str, start_time: Instant) {
  let duration = start_time.elapsed();
  if duration.as_millis() > 1000 {
    warn!("Slow Agent operation: {} took {}ms", operation, duration.as_millis());
  } else {
    debug!("Agent operation: {} completed in {}ms", operation, duration.as_millis());
  }
}

/// 获取智能体列表
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_agent_list_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentListPB, FlowyError> {
  let start_time = Instant::now();
  info!("🤖 Processing get agent list request");

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.get_agent_list().await {
    Ok(agent_list) => {
      info!("✅ Successfully retrieved {} agents", agent_list.agents.len());
      log_operation_duration("get_agent_list", start_time);
      data_result_ok(agent_list)
    }
    Err(err) => {
      error!("❌ Failed to get agent list: {}", err);
      log_operation_duration("get_agent_list", start_time);
      Err(err)
    }
  }
}

/// 创建智能体
/// 支持完整的配置验证和错误处理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn create_agent_handler(
  data: AFPluginData<CreateAgentRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentConfigPB, FlowyError> {
  let start_time = Instant::now();
  let data = data.into_inner();
  
  info!("🤖 Processing create agent request for: {}", data.name);
  debug!("Agent creation details: name={}, description={}, tools={:?}", 
         data.name, data.description, data.available_tools);

  // 验证输入数据
  if let Err(validation_err) = data.validate() {
    error!("❌ Agent creation validation failed: {}", validation_err);
    return Err(FlowyError::invalid_data().with_context(format!("Validation failed: {}", validation_err)));
  }

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.create_agent(data).await {
    Ok(agent_config) => {
      info!("✅ Successfully created agent: {} ({})", agent_config.name, agent_config.id);
      log_operation_duration("create_agent", start_time);
      data_result_ok(agent_config)
    }
    Err(err) => {
      error!("❌ Failed to create agent: {}", err);
      log_operation_duration("create_agent", start_time);
      Err(err)
    }
  }
}

/// 获取智能体配置
/// 支持根据ID获取特定智能体的详细配置
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_agent_handler(
  data: AFPluginData<GetAgentRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentConfigPB, FlowyError> {
  let start_time = Instant::now();
  let data = data.into_inner();
  
  info!("🤖 Processing get agent request for ID: {}", data.id);

  // 验证输入数据
  if let Err(validation_err) = data.validate() {
    error!("❌ Agent get validation failed: {}", validation_err);
    return Err(FlowyError::invalid_data().with_context(format!("Validation failed: {}", validation_err)));
  }

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.get_agent(data).await {
    Ok(agent_config) => {
      info!("✅ Successfully retrieved agent: {} ({})", agent_config.name, agent_config.id);
      log_operation_duration("get_agent", start_time);
      data_result_ok(agent_config)
    }
    Err(err) => {
      error!("❌ Failed to get agent: {}", err);
      log_operation_duration("get_agent", start_time);
      Err(err)
    }
  }
}

/// 更新智能体配置
/// 支持部分更新和完整的验证机制
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_agent_handler(
  data: AFPluginData<UpdateAgentRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentConfigPB, FlowyError> {
  let start_time = Instant::now();
  let data = data.into_inner();
  
  info!("🤖 Processing update agent request for ID: {}", data.id);
  debug!("Agent update details: {:?}", data);

  // 验证输入数据
  if let Err(validation_err) = data.validate() {
    error!("❌ Agent update validation failed: {}", validation_err);
    return Err(FlowyError::invalid_data().with_context(format!("Validation failed: {}", validation_err)));
  }

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.update_agent(data).await {
    Ok(agent_config) => {
      info!("✅ Successfully updated agent: {} ({})", agent_config.name, agent_config.id);
      log_operation_duration("update_agent", start_time);
      data_result_ok(agent_config)
    }
    Err(err) => {
      error!("❌ Failed to update agent: {}", err);
      log_operation_duration("update_agent", start_time);
      Err(err)
    }
  }
}

/// 删除智能体
/// 支持软删除和硬删除，包含依赖检查
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn delete_agent_handler(
  data: AFPluginData<DeleteAgentRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let start_time = Instant::now();
  let data = data.into_inner();
  
  info!("🤖 Processing delete agent request for ID: {}", data.id);

  // 验证输入数据
  if let Err(validation_err) = data.validate() {
    error!("❌ Agent delete validation failed: {}", validation_err);
    return Err(FlowyError::invalid_data().with_context(format!("Validation failed: {}", validation_err)));
  }

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.delete_agent(data).await {
    Ok(_) => {
      info!("✅ Successfully deleted agent");
      log_operation_duration("delete_agent", start_time);
      Ok(())
    }
    Err(err) => {
      error!("❌ Failed to delete agent: {}", err);
      log_operation_duration("delete_agent", start_time);
      Err(err)
    }
  }
}

/// 验证智能体配置
/// 提供详细的配置验证和建议
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn validate_agent_config_handler(
  data: AFPluginData<AgentConfigPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentSuccessResponsePB, FlowyError> {
  let start_time = Instant::now();
  let data = data.into_inner();
  
  info!("🤖 Processing validate agent config request for: {}", data.name);

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.validate_agent_config(data).await {
    Ok(validation_result) => {
      info!("✅ Successfully validated agent config");
      log_operation_duration("validate_agent_config", start_time);
      data_result_ok(validation_result)
    }
    Err(err) => {
      error!("❌ Failed to validate agent config: {}", err);
      log_operation_duration("validate_agent_config", start_time);
      Err(err)
    }
  }
}

/// 获取智能体全局设置
/// 返回系统级别的智能体配置参数
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_agent_global_settings_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentGlobalSettingsPB, FlowyError> {
  let start_time = Instant::now();
  info!("🤖 Processing get agent global settings request");

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.get_agent_global_settings().await {
    Ok(global_settings) => {
      info!("✅ Successfully retrieved agent global settings");
      log_operation_duration("get_agent_global_settings", start_time);
      data_result_ok(global_settings)
    }
    Err(err) => {
      error!("❌ Failed to get agent global settings: {}", err);
      log_operation_duration("get_agent_global_settings", start_time);
      Err(err)
    }
  }
}

/// 更新智能体全局设置
/// 支持系统级别配置的更新和验证
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_agent_global_settings_handler(
  data: AFPluginData<AgentGlobalSettingsPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let start_time = Instant::now();
  let data = data.into_inner();
  
  info!("🤖 Processing update agent global settings request");
  debug!("Global settings update: enabled={}, debug={}", data.enabled, data.debug_logging);

  // 验证输入数据
  if let Err(validation_err) = data.validate() {
    error!("❌ Agent global settings validation failed: {}", validation_err);
    return Err(FlowyError::invalid_data().with_context(format!("Validation failed: {}", validation_err)));
  }

  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  match ai_manager.update_agent_global_settings(data).await {
    Ok(_) => {
      info!("✅ Successfully updated agent global settings");
      log_operation_duration("update_agent_global_settings", start_time);
      Ok(())
    }
    Err(err) => {
      error!("❌ Failed to update agent global settings: {}", err);
      log_operation_duration("update_agent_global_settings", start_time);
      Err(err)
    }
  }
}

// ==================== 执行日志相关事件处理器 ====================

/// 获取执行日志列表
/// 支持分页、过滤和搜索功能
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_execution_logs_handler(
  data: AFPluginData<GetExecutionLogsRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<AgentExecutionLogListPB, FlowyError> {
  let start_time = Instant::now();
  let data = data.try_into_inner()?;
  data.validate()?;
  
  info!("📋 [EXECUTION-LOG-API] Processing get execution logs request for session: {}", data.session_id);
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 从AI管理器获取执行日志
  match ai_manager.get_execution_logs(&data).await {
    Ok(logs) => {
      info!("✅ [EXECUTION-LOG-API] Successfully retrieved {} execution logs", logs.logs.len());
      log_operation_duration("get_execution_logs", start_time);
      data_result_ok(logs)
    }
    Err(err) => {
      error!("❌ Failed to get execution logs: {}", err);
      log_operation_duration("get_execution_logs", start_time);
      Err(err)
    }
  }
}

/// 添加执行日志
/// 用于智能体执行过程中记录日志
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn add_execution_log_handler(
  data: AFPluginData<AgentExecutionLogPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let start_time = Instant::now();
  let data = data.try_into_inner()?;
  
  info!("📝 [EXECUTION-LOG-API] Adding execution log for session: {}, phase: {:?}", data.session_id, data.phase);
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 添加执行日志到AI管理器
  match ai_manager.add_execution_log(data).await {
    Ok(_) => {
      info!("✅ [EXECUTION-LOG-API] Successfully added execution log");
      log_operation_duration("add_execution_log", start_time);
      Ok(())
    }
    Err(err) => {
      error!("❌ Failed to add execution log: {}", err);
      log_operation_duration("add_execution_log", start_time);
      Err(err)
    }
  }
}

/// 清空执行日志
/// 清理指定会话或消息的执行日志
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn clear_execution_logs_handler(
  data: AFPluginData<ClearExecutionLogsRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> Result<(), FlowyError> {
  let start_time = Instant::now();
  let data = data.try_into_inner()?;
  data.validate()?;
  
  info!("🗑️ Clearing execution logs for session: {}", data.session_id);
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 清空执行日志
  match ai_manager.clear_execution_logs(&data).await {
    Ok(_) => {
      info!("✅ Successfully cleared execution logs");
      log_operation_duration("clear_execution_logs", start_time);
      Ok(())
    }
    Err(err) => {
      error!("❌ Failed to clear execution logs: {}", err);
      log_operation_duration("clear_execution_logs", start_time);
      Err(err)
    }
  }
}
