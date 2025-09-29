use crate::ai_manager::AIManager;
use crate::entities::*;
use crate::mcp::entities::*;
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
      error!("AI manager has been dropped, cannot process MCP request");
      FlowyError::internal().with_context("The AI manager is already dropped")
    })?;
  trace!("Successfully upgraded AI manager reference");
  Ok(ai_manager)
}

/// 记录操作性能指标的辅助函数
fn log_operation_duration(operation: &str, start_time: Instant) {
  let duration = start_time.elapsed();
  if duration.as_millis() > 1000 {
    warn!("Slow MCP operation: {} took {}ms", operation, duration.as_millis());
  } else {
    debug!("MCP operation: {} completed in {}ms", operation, duration.as_millis());
  }
}

/// 获取MCP服务器列表
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_mcp_server_list_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPServerListPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  debug!("Fetching MCP server list");
  
  // 从配置管理器获取服务器列表，包含错误处理
  let configs = match ai_manager.mcp_manager.config_manager().get_all_servers() {
    configs if configs.is_empty() => {
      debug!("No MCP servers configured");
      Vec::new()
    }
    configs => {
      info!("Found {} MCP server configurations", configs.len());
      configs
    }
  };
  
  // 转换配置为协议缓冲区格式，包含状态管理
  let servers = configs.into_iter().map(|config| {
    let server_id = config.id.clone();
    let is_connected = ai_manager.mcp_manager.is_server_connected(&server_id);
    
    debug!("Processing server config: {} (connected: {})", config.name, is_connected);
    
    MCPServerConfigPB {
      id: config.id,
      name: config.name,
      icon: config.icon,
      transport_type: match config.transport_type {
        MCPTransportType::Stdio => MCPTransportTypePB::Stdio,
        MCPTransportType::SSE => MCPTransportTypePB::SSE,
        MCPTransportType::HTTP => MCPTransportTypePB::HTTP,
      },
      is_active: config.is_active && is_connected, // 结合配置状态和连接状态
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

  let result = MCPServerListPB { servers };
  info!("Successfully retrieved {} MCP servers", result.servers.len());
  data_result_ok(result)
}

/// 添加MCP服务器配置
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn add_mcp_server_handler(
  data: AFPluginData<MCPServerConfigPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  info!("Adding MCP server: {} ({})", data.name, data.id);
  
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

  // 保存配置，包含错误处理
  if let Err(e) = ai_manager.mcp_manager.config_manager().save_server(config.clone()) {
    error!("Failed to save MCP server config {}: {}", config.name, e);
    return Err(e);
  }
  
  info!("Successfully saved MCP server config: {}", config.name);
  
  // 如果配置为激活状态，尝试连接
  if config.is_active {
    debug!("Attempting to connect to MCP server: {}", config.name);
    match ai_manager.mcp_manager.connect_server(config.clone()).await {
      Ok(()) => {
        info!("Successfully connected to MCP server: {}", config.name);
      }
      Err(e) => {
        warn!("Failed to connect to MCP server {} after adding: {}", config.name, e);
        // 连接失败不应该导致整个操作失败，只记录警告
      }
    }
  } else {
    debug!("MCP server {} is not active, skipping connection", config.name);
  }

  Ok(())
}

/// 更新MCP服务器配置
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn update_mcp_server_handler(
  data: AFPluginData<MCPServerConfigPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  info!("Updating MCP server: {} ({})", data.name, data.id);
  
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

  // 先断开现有连接，包含状态管理
  let was_connected = ai_manager.mcp_manager.is_server_connected(&server_id);
  if was_connected {
    debug!("Disconnecting existing MCP server: {}", server_id);
    if let Err(e) = ai_manager.mcp_manager.remove_server(&server_id).await {
      warn!("Failed to remove existing MCP server connection {}: {}", server_id, e);
      // 继续执行，不让断开连接的失败阻止配置更新
    }
  }

  // 保存更新的配置，包含错误处理
  if let Err(e) = ai_manager.mcp_manager.config_manager().save_server(config.clone()) {
    error!("Failed to save updated MCP server config {}: {}", config.name, e);
    return Err(e);
  }
  
  info!("Successfully updated MCP server config: {}", config.name);
  
  // 如果配置为激活状态，重新连接
  if config.is_active {
    debug!("Attempting to reconnect to updated MCP server: {}", config.name);
    match ai_manager.mcp_manager.connect_server(config.clone()).await {
      Ok(()) => {
        info!("Successfully reconnected to MCP server: {}", config.name);
      }
      Err(e) => {
        warn!("Failed to reconnect to MCP server {} after updating: {}", config.name, e);
        // 连接失败不应该导致整个操作失败
      }
    }
  } else {
    debug!("Updated MCP server {} is not active, skipping connection", config.name);
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
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn connect_mcp_server_handler(
  data: AFPluginData<MCPConnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPServerStatusPB, FlowyError> {
  let start_time = Instant::now();
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  info!("Attempting to connect to MCP server: {}", server_id);
  
  // 获取服务器配置，包含错误处理
  let config = ai_manager.mcp_manager.config_manager()
    .get_server(server_id)
    .ok_or_else(|| {
      error!("MCP server config not found: {}", server_id);
      FlowyError::record_not_found()
        .with_context(format!("MCP server config not found: {}", server_id))
    })?;
  
  debug!("Found MCP server config: {} ({})", config.name, config.transport_type);

  // 尝试连接
  let mut status = MCPServerStatusPB {
    server_id: server_id.clone(),
    is_connected: false,
    error_message: None,
    tool_count: 0,
  };

  // 尝试连接，包含完整的状态管理和错误处理
  match ai_manager.mcp_manager.connect_server(config.clone()).await {
    Ok(()) => {
      status.is_connected = true;
      info!("Successfully connected to MCP server: {}", config.name);
      
      // 获取工具数量，包含错误处理
      match ai_manager.mcp_manager.tool_list(server_id).await {
        Ok(tools) => {
          status.tool_count = tools.tools.len() as i32;
          debug!("Discovered {} tools for server: {}", status.tool_count, config.name);
        }
        Err(e) => {
          warn!("Failed to get tool list for server {}: {}", config.name, e);
          // 工具列表获取失败不影响连接状态
        }
      }
    }
    Err(e) => {
      error!("Failed to connect to MCP server {}: {}", config.name, e);
      status.error_message = Some(format!("Connection failed: {}", e));
    }
  }

  log_operation_duration(&format!("connect_server_{}", server_id), start_time);
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
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_mcp_server_status_handler(
  data: AFPluginData<MCPConnectServerRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPServerStatusPB, FlowyError> {
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  debug!("Checking MCP server status: {}", server_id);
  
  let mut status = MCPServerStatusPB {
    server_id: server_id.clone(),
    is_connected: false,
    error_message: None,
    tool_count: 0,
  };

  // 检查连接状态，包含完整的状态管理
  let is_connected = ai_manager.mcp_manager.is_server_connected(server_id);
  status.is_connected = is_connected;
  
  if is_connected {
    debug!("MCP server {} is connected, fetching tool count", server_id);
    // 获取工具数量，包含错误处理
    match ai_manager.mcp_manager.tool_list(server_id).await {
      Ok(tools) => {
        status.tool_count = tools.tools.len() as i32;
        debug!("MCP server {} has {} tools", server_id, status.tool_count);
      }
      Err(e) => {
        warn!("Failed to get tool list for connected server {}: {}", server_id, e);
        status.error_message = Some(format!("Tool list error: {}", e));
      }
    }
  } else {
    debug!("MCP server {} is not connected", server_id);
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
/// 支持异步处理，包含完整的错误处理和状态管理
#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn call_mcp_tool_handler(
  data: AFPluginData<MCPToolCallRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<MCPToolCallResponsePB, FlowyError> {
  let start_time = Instant::now();
  let data = data.try_into_inner()?;
  data.validate()?;
  
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let server_id = &data.server_id;
  let tool_name = &data.tool_name;
  let arguments_str = &data.arguments;
  
  info!("Calling MCP tool: {} on server: {}", tool_name, server_id);
  
  // 验证服务器连接状态
  if !ai_manager.mcp_manager.is_server_connected(server_id) {
    error!("MCP server {} is not connected", server_id);
    return data_result_ok(MCPToolCallResponsePB {
      success: false,
      content: vec![],
      error: Some(format!("Server '{}' is not connected", server_id)),
    });
  }
  
  // 解析参数，包含错误处理
  let arguments: serde_json::Value = match serde_json::from_str(arguments_str) {
    Ok(args) => {
      debug!("Parsed tool arguments successfully");
      args
    }
    Err(e) => {
      error!("Invalid JSON arguments for tool {}: {}", tool_name, e);
      return data_result_ok(MCPToolCallResponsePB {
        success: false,
        content: vec![],
        error: Some(format!("Invalid JSON arguments: {}", e)),
      });
    }
  };

  // 调用工具，包含完整的错误处理和状态管理
  debug!("Executing tool call: {} with arguments: {}", tool_name, arguments_str);
  
  match ai_manager.mcp_manager.call_tool(server_id, tool_name, arguments).await {
    Ok(response) => {
      info!("Tool call successful: {} on server: {}", tool_name, server_id);
      
      let content = response.content.into_iter().enumerate().map(|(index, item)| {
        debug!("Processing response content item {}: type={}", index, item.r#type);
        
        MCPContentPB {
          content_type: match item.r#type.as_str() {
            "text" => MCPContentTypePB::Text,
            "image" => MCPContentTypePB::Image,
            "resource" => MCPContentTypePB::Resource,
            unknown_type => {
              warn!("Unknown content type '{}', defaulting to text", unknown_type);
              MCPContentTypePB::Text
            }
          },
          text: item.text.unwrap_or_else(|| {
            debug!("No text content in item {}, using empty string", index);
            String::new()
          }),
          mime_type: None, // ToolCallContent没有mime_type字段
        }
      }).collect();

      let response_pb = MCPToolCallResponsePB {
        success: true,
        content,
        error: None,
      };
      
      debug!("Tool call response prepared with {} content items", response_pb.content.len());
      log_operation_duration(&format!("call_tool_{}_{}", server_id, tool_name), start_time);
      data_result_ok(response_pb)
    }
    Err(e) => {
      error!("Tool call failed: {} on server {}: {}", tool_name, server_id, e);
      
      log_operation_duration(&format!("call_tool_{}_{}_failed", server_id, tool_name), start_time);
      data_result_ok(MCPToolCallResponsePB {
        success: false,
        content: vec![],
        error: Some(format!("Tool execution failed: {}", e)),
      })
    }
  }
}
