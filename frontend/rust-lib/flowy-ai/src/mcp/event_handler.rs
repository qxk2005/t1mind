use crate::ai_manager::AIManager;
use crate::entities::*;
use crate::mcp::entities::*;
use flowy_error::{FlowyError, FlowyResult};
use lib_dispatch::prelude::{AFPluginData, AFPluginState, DataResult, data_result_ok};
use std::sync::{Arc, Weak};
use validator::Validate;

fn upgrade_ai_manager(ai_manager: AFPluginState<Weak<AIManager>>) -> FlowyResult<Arc<AIManager>> {
  let ai_manager = ai_manager
    .upgrade()
    .ok_or(FlowyError::internal().with_context("The AI manager is already dropped"))?;
  Ok(ai_manager)
}

/// 获取MCP服务器列表
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_mcp_server_list_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPServerListPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 从配置管理器获取服务器列表
  let configs = ai_manager.mcp_manager.config_manager().get_all_servers();
  
  let servers = configs.into_iter().map(|config| {
    MCPServerConfigPB {
      id: config.id,
      name: config.name,
      icon: config.icon,
      transport_type: match config.transport_type {
        MCPTransportType::Stdio => MCPTransportTypePB::Stdio,
        MCPTransportType::SSE => MCPTransportTypePB::SSE,
        MCPTransportType::HTTP => MCPTransportTypePB::HTTP,
      },
      is_active: config.is_active,
      description: config.description,
      stdio_config: config.stdio_config.map(|stdio| MCPStdioConfigPB {
        command: stdio.command,
        args: stdio.args,
        env_vars: stdio.env_vars,
      }),
      http_config: config.http_config.map(|http| MCPHttpConfigPB {
        url: http.url,
        headers: http.headers,
      }),
    }
  }).collect();

  data_result_ok(MCPServerListPB { servers })
}

/// 添加MCP服务器配置
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn add_mcp_server_handler(
  data: AFPluginData<MCPServerConfigPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 转换为内部配置格式
  let config = MCPServerConfig {
    id: data.id,
    name: data.name,
    icon: data.icon,
    transport_type: match data.transport_type {
      MCPTransportTypePB::Stdio => MCPTransportType::Stdio,
      MCPTransportTypePB::SSE => MCPTransportType::SSE,
      MCPTransportTypePB::HTTP => MCPTransportType::HTTP,
    },
    is_active: data.is_active,
    description: data.description,
    created_at: std::time::SystemTime::now(),
    updated_at: std::time::SystemTime::now(),
    stdio_config: data.stdio_config.map(|stdio| MCPStdioConfig {
      command: stdio.command,
      args: stdio.args,
      env_vars: stdio.env_vars,
    }),
    http_config: data.http_config.map(|http| MCPHttpConfig {
      url: http.url,
      headers: http.headers,
    }),
  };

  // 保存配置
  ai_manager.mcp_manager.config_manager().save_server(config.clone())?;
  
  // 如果配置为激活状态，尝试连接
  if config.is_active {
    if let Err(e) = ai_manager.mcp_manager.connect_server(config).await {
      tracing::warn!("Failed to connect to MCP server after adding: {}", e);
    }
  }

  Ok(())
}

/// 更新MCP服务器配置
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_mcp_server_handler(
  data: AFPluginData<MCPServerConfigPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  // 转换为内部配置格式
  let server_id = data.id.clone();
  let config = MCPServerConfig {
    id: server_id.clone(),
    name: data.name,
    icon: data.icon,
    transport_type: match data.transport_type {
      MCPTransportTypePB::Stdio => MCPTransportType::Stdio,
      MCPTransportTypePB::SSE => MCPTransportType::SSE,
      MCPTransportTypePB::HTTP => MCPTransportType::HTTP,
    },
    is_active: data.is_active,
    description: data.description,
    created_at: std::time::SystemTime::now(), // 这里应该保留原始创建时间，但为了简化先用当前时间
    updated_at: std::time::SystemTime::now(),
    stdio_config: data.stdio_config.map(|stdio| MCPStdioConfig {
      command: stdio.command,
      args: stdio.args,
      env_vars: stdio.env_vars,
    }),
    http_config: data.http_config.map(|http| MCPHttpConfig {
      url: http.url,
      headers: http.headers,
    }),
  };

  // 先断开现有连接
  if let Err(e) = ai_manager.mcp_manager.remove_server(&server_id).await {
    tracing::warn!("Failed to remove existing MCP server connection: {}", e);
  }

  // 保存更新的配置
  ai_manager.mcp_manager.config_manager().save_server(config.clone())?;
  
  // 如果配置为激活状态，重新连接
  if config.is_active {
    if let Err(e) = ai_manager.mcp_manager.connect_server(config).await {
      tracing::warn!("Failed to reconnect to MCP server after updating: {}", e);
    }
  }

  Ok(())
}

/// 删除MCP服务器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn remove_mcp_server_handler(
  data: AFPluginData<MCPDisconnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  
  // 断开连接
  ai_manager.mcp_manager.remove_server(server_id).await?;
  
  // 删除配置
  ai_manager.mcp_manager.config_manager().delete_server(server_id)?;

  Ok(())
}

/// 连接MCP服务器
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn connect_mcp_server_handler(
  data: AFPluginData<MCPConnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPServerStatusPB, FlowyError> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  
  // 获取服务器配置
  let config = ai_manager.mcp_manager.config_manager()
    .get_server(server_id)
    .ok_or_else(|| FlowyError::record_not_found()
      .with_context(format!("MCP server config not found: {}", server_id)))?;

  // 尝试连接
  let mut status = MCPServerStatusPB {
    server_id: server_id.clone(),
    is_connected: false,
    error_message: None,
    tool_count: 0,
  };

  match ai_manager.mcp_manager.connect_server(config).await {
    Ok(()) => {
      status.is_connected = true;
      // 获取工具数量
      if let Ok(tools) = ai_manager.mcp_manager.tool_list(server_id).await {
        status.tool_count = tools.tools.len() as i32;
      }
    }
    Err(e) => {
      status.error_message = Some(e.to_string());
    }
  }

  data_result_ok(status)
}

/// 断开MCP服务器连接
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn disconnect_mcp_server_handler(
  data: AFPluginData<MCPDisconnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  ai_manager.mcp_manager.remove_server(&data.server_id).await?;

  Ok(())
}

/// 获取MCP服务器状态
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_mcp_server_status_handler(
  data: AFPluginData<MCPConnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPServerStatusPB, FlowyError> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  let mut status = MCPServerStatusPB {
    server_id: server_id.clone(),
    is_connected: false,
    error_message: None,
    tool_count: 0,
  };

  // 检查连接状态
  if ai_manager.mcp_manager.is_server_connected(server_id) {
    status.is_connected = true;
    // 获取工具数量
    if let Ok(tools) = ai_manager.mcp_manager.tool_list(server_id).await {
      status.tool_count = tools.tools.len() as i32;
    }
  }

  data_result_ok(status)
}

/// 获取MCP工具列表
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_mcp_tool_list_handler(
  data: AFPluginData<MCPConnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPToolListPB, FlowyError> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  let tools_list = ai_manager.mcp_manager.tool_list(server_id).await?;
  
  let tools = tools_list.tools.into_iter().map(|tool| {
    MCPToolPB {
      name: tool.name,
      description: tool.description,
      input_schema: serde_json::to_string(&tool.input_schema).unwrap_or_default(),
      annotations: tool.annotations.map(|ann| MCPToolAnnotationsPB {
        title: ann.title,
        read_only_hint: ann.read_only_hint,
        destructive_hint: ann.destructive_hint,
        idempotent_hint: ann.idempotent_hint,
        open_world_hint: ann.open_world_hint,
      }),
    }
  }).collect();

  data_result_ok(MCPToolListPB { tools })
}

/// 调用MCP工具
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn call_mcp_tool_handler(
  data: AFPluginData<MCPToolCallRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPToolCallResponsePB, FlowyError> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  let tool_name = &data.tool_name;
  let arguments_str = &data.arguments;
  
  // 解析参数
  let arguments: serde_json::Value = serde_json::from_str(arguments_str)
    .map_err(|e| FlowyError::invalid_data()
      .with_context(format!("Invalid JSON arguments: {}", e)))?;

  // 调用工具
  match ai_manager.mcp_manager.call_tool(server_id, tool_name, arguments).await {
    Ok(response) => {
      let content = response.content.into_iter().map(|item| {
        MCPContentPB {
          content_type: match item.r#type.as_str() {
            "text" => MCPContentTypePB::Text,
            "image" => MCPContentTypePB::Image,
            "resource" => MCPContentTypePB::Resource,
            _ => MCPContentTypePB::Text, // 默认为文本类型
          },
          text: item.text.unwrap_or_default(),
          mime_type: None, // ToolCallContent没有mime_type字段
        }
      }).collect();

      data_result_ok(MCPToolCallResponsePB {
        success: true,
        content,
        error: None,
      })
    }
    Err(e) => {
      data_result_ok(MCPToolCallResponsePB {
        success: false,
        content: vec![],
        error: Some(e.to_string()),
      })
    }
  }
}
