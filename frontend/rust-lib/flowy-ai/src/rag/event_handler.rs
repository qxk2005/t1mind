use crate::ai_manager::AIManager;
use crate::entities::*;
use flowy_error::FlowyResult;
use lib_dispatch::prelude::*;
use std::sync::{Arc, Weak};
use tracing::{error, info};

fn upgrade_ai_manager(ai_manager: AFPluginState<Weak<AIManager>>) -> FlowyResult<Arc<AIManager>> {
    let ai_manager = ai_manager
        .upgrade()
        .ok_or_else(|| flowy_error::FlowyError::internal().with_context("The AI manager is already dropped"))?;
    Ok(ai_manager)
}

/// 获取RAG设置事件处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_rag_settings_handler(
    data: AFPluginData<GetRAGSettingsRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<GetRAGSettingsResponsePB, flowy_error::FlowyError> {
    let _data = data.into_inner();
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    
    info!("Getting RAG settings");
    
    let settings = ai_manager.rag_config_manager.get_rag_settings();
    let mut response = GetRAGSettingsResponsePB::default();
    response.settings = Some(settings);
    
    data_result_ok(response)
}

/// 更新RAG设置事件处理器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_rag_settings_handler(
    data: AFPluginData<UpdateRAGSettingsRequestPB>,
    ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<UpdateRAGSettingsResponsePB, flowy_error::FlowyError> {
    let data = data.into_inner();
    let ai_manager = upgrade_ai_manager(ai_manager)?;
    
    info!("Updating RAG settings");
    
    // 更新设置
    match ai_manager.rag_config_manager.update_rag_settings(data) {
        Ok(updated_settings) => {
            let response = UpdateRAGSettingsResponsePB {
                success: true,
                error_message: None,
                settings: Some(updated_settings),
            };
            data_result_ok(response)
        }
        Err(e) => {
            error!("Failed to update RAG settings: {}", e);
            let response = UpdateRAGSettingsResponsePB {
                success: false,
                error_message: Some(e.to_string()),
                settings: None,
            };
            data_result_ok(response)
        }
    }
}

