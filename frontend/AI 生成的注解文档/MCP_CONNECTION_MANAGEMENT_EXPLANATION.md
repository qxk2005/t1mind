# MCP 连接管理机制详解

## 概述

系统通过多层架构管理 MCP 服务器的连接状态,主要组件包括:
1. **MCPClientPool** - 客户端连接池,管理所有已创建的客户端实例
2. **MCPClient** - 具体的客户端实现(STDIO/HTTP/SSE),每个客户端维护自己的连接状态
3. **MCPConfigManager** - 配置管理器,持久化存储所有已配置的服务器信息

## 核心概念

### 1. 已配置 vs 已连接

- **已配置的服务器**:用户在设置中添加的所有 MCP 服务器配置,存储在 SQLite 数据库中
  - 由 `MCPConfigManager.get_all_servers()` 返回
  - 包含连接信息(URL、命令、参数等)
  - 即使未连接也会持久化保存

- **已连接的服务器**:实际创建了客户端实例并成功初始化的服务器
  - 由 `MCPClientPool` 管理,存储在内存中的 `HashMap`
  - 由 `MCPClientManager.list_servers()` 返回
  - 应用重启后需要重新连接

### 2. 连接状态枚举

```rust
// rust-lib/flowy-ai/src/mcp/entities.rs
pub enum MCPConnectionStatus {
    Disconnected,    // 未连接
    Connecting,      // 连接中
    Connected,       // 已连接
    Error(String),   // 连接错误
}
```

## 连接判断流程

### 1. 客户端创建和初始化

```rust
// rust-lib/flowy-ai/src/mcp/client_pool.rs:40-88
pub async fn create_client(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
    let server_id = config.id.clone();
    
    // 1. 根据传输类型创建对应的客户端
    let mut client: Box<dyn MCPClient> = match config.transport_type {
        MCPTransportType::Stdio => Box::new(StdioMCPClient::new(config.clone())?),
        MCPTransportType::SSE => Box::new(SSEMCPClient::new(config.clone())?),
        MCPTransportType::HTTP => Box::new(HttpMCPClient::new(config.clone())?),
    };

    // 2. 调用 initialize() 初始化连接
    let init_result = client.initialize().await;
    
    // 3. 创建元数据,记录连接时间和状态
    let metadata = ClientMetadata {
        server_id: server_id.clone(),
        config: config.clone(),
        created_at: SystemTime::now(),
        last_connected: if init_result.is_ok() { Some(SystemTime::now()) } else { None },
        connection_attempts: 1,
        error_message: init_result.as_ref().err().map(|e| e.to_string()),
    };

    // 4. 存储到连接池
    clients.insert(server_id.clone(), Arc::new(RwLock::new(client)));
    client_metadata.insert(server_id.clone(), metadata);
    
    // 5. 即使初始化失败,客户端也会被保留(允许后续重连)
    if let Err(e) = init_result {
        tracing::warn!("Failed to initialize client for {}: {}", config.name, e);
        return Err(e);
    }
    
    Ok(())
}
```

### 2. 不同传输类型的连接判断

#### STDIO 客户端

```rust
// rust-lib/flowy-ai/src/mcp/client.rs:57-87
async fn initialize(&mut self) -> Result<(), FlowyError> {
    self.status = MCPConnectionStatus::Connecting;
    
    let stdio_config = self.config.stdio_config.as_ref()
        .ok_or_else(|| FlowyError::invalid_data().with_context("Missing STDIO config"))?;
    
    // 启动子进程
    let mut cmd = tokio::process::Command::new(&stdio_config.command);
    cmd.args(&stdio_config.args);
    
    // 配置标准输入输出
    cmd.stdin(std::process::Stdio::piped())
       .stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());
    
    match cmd.spawn() {
        Ok(process) => {
            self.process = Some(process);
            // ✅ 进程启动成功 = 连接成功
            self.status = MCPConnectionStatus::Connected;
            Ok(())
        }
        Err(e) => {
            // ❌ 进程启动失败 = 连接失败
            self.status = MCPConnectionStatus::Error(error_msg);
            Err(FlowyError::internal().with_context(error_msg))
        }
    }
}
```

**判断标准**:子进程是否成功启动

#### HTTP 客户端

```rust
// rust-lib/flowy-ai/src/mcp/client.rs:210-240
async fn initialize(&mut self) -> Result<(), FlowyError> {
    self.status = MCPConnectionStatus::Connecting;
    
    // 发送 initialize 请求
    let init_message = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { /* ... */ }
    });
    
    let response = self.client
        .post(&http_config.url)
        .json(&init_message)
        .send()
        .await?;
    
    if response.status().is_success() {
        // ✅ HTTP 请求成功且返回成功状态码
        self.status = MCPConnectionStatus::Connected;
        Ok(())
    } else {
        // ❌ HTTP 请求失败
        self.status = MCPConnectionStatus::Error(error_msg);
        Err(FlowyError::http().with_context(error_msg))
    }
}
```

**判断标准**:HTTP 请求是否成功且返回 200 系列状态码

#### SSE 客户端

```rust
// rust-lib/flowy-ai/src/mcp/client.rs:434-464
async fn initialize(&mut self) -> Result<(), FlowyError> {
    self.status = MCPConnectionStatus::Connecting;
    
    // 测试连接
    match self.client.get(&http_config.url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                // ✅ SSE 端点可访问
                self.status = MCPConnectionStatus::Connected;
                Ok(())
            } else {
                // ❌ SSE 端点返回错误状态码
                self.status = MCPConnectionStatus::Error(error_msg);
                Err(FlowyError::http().with_context(error_msg))
            }
        }
        Err(e) => {
            // ❌ 无法连接到 SSE 端点
            self.status = MCPConnectionStatus::Error(error_msg);
            Err(FlowyError::http().with_context(error_msg))
        }
    }
}
```

**判断标准**:SSE 端点是否可访问

### 3. 获取连接状态

```rust
// rust-lib/flowy-ai/src/mcp/client.rs:27-29
fn is_connected(&self) -> bool {
    matches!(self.get_status(), MCPConnectionStatus::Connected)
}
```

每个客户端实例都维护一个 `status` 字段,通过 `get_status()` 方法返回当前状态。

### 4. list_servers() 的工作原理

```rust
// rust-lib/flowy-ai/src/mcp/client_pool.rs:172-197
pub async fn get_all_clients_info(&self) -> Vec<MCPClientInfo> {
    let clients = self.clients.read().await;  // 读取内存中的客户端池
    let client_metadata = self.client_metadata.read().await;
    
    let mut infos = Vec::new();
    
    // 遍历所有已创建的客户端
    for (server_id, client) in clients.iter() {
        let client_guard = client.read().await;
        let metadata = client_metadata.get(server_id);
        
        let info = MCPClientInfo {
            server_id: server_id.clone(),
            status: client_guard.get_status(),  // 获取客户端的当前状态
            tools: match client_guard.list_tools().await {
                Ok(tools_list) => tools_list.tools,
                Err(_) => Vec::new(),
            },
            last_connected: metadata.and_then(|m| m.last_connected),
            error_message: metadata.and_then(|m| m.error_message.clone()),
        };
        
        infos.push(info);
    }
    
    infos
}
```

```rust
// rust-lib/flowy-ai/src/mcp/manager.rs:118-120
pub async fn list_servers(&self) -> Vec<MCPClientInfo> {
    self.client_pool.get_all_clients_info().await
}
```

**关键点**:
- `list_servers()` 只返回**内存中存在的客户端实例**
- 即使客户端状态是 `Error` 或 `Disconnected`,只要实例被创建过,就会出现在列表中
- 但如果服务器配置了但从未调用 `connect_server()`,则不会出现在列表中

## 为什么之前的自动检测会失败?

### 问题场景

1. 用户在设置中配置了 Excel MCP 服务器
2. 配置被保存到 SQLite (`MCPConfigManager`)
3. 但客户端实例尚未创建(未调用 `connect_server()`)
4. `list_servers()` 返回空列表
5. 自动检测失败,找不到工具

### 旧代码的问题

```rust
// ❌ 旧代码:只查询已连接的服务器
async fn execute_auto_detected_tool(
    &self,
    request: &ToolCallRequest,
) -> FlowyResult<String> {
    // 只能找到已经调用过 connect_server() 的服务器
    let servers = self.mcp_manager.list_servers().await;
    
    for server in servers {
        // ...
    }
    
    self.execute_native_tool(request).await
}
```

### 新代码的解决方案

```rust
// ✅ 新代码:使用智能查找
async fn execute_auto_detected_tool(
    &self,
    request: &ToolCallRequest,
) -> FlowyResult<String> {
    // find_tool_by_name 会:
    // 1. 先查缓存(已连接的服务器的工具列表)
    // 2. 查询所有配置的服务器(从 MCPConfigManager)
    // 3. 如果需要,自动连接并获取工具列表
    match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
        Some((server_id, tool)) => {
            self.execute_mcp_tool(&server_id, request).await
        }
        None => {
            self.execute_native_tool(request).await
        }
    }
}
```

## find_tool_by_name 的工作流程

```rust
// rust-lib/flowy-ai/src/mcp/tool_discovery.rs:99-134
pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
    // 1. 首先从缓存查找(已连接的服务器)
    {
        let registry = self.tool_registry.read().await;
        for (server_id, tools) in registry.iter() {
            if let Some(tool) = tools.iter().find(|t| t.name == tool_name) {
                return Some((server_id.clone(), tool.clone()));
            }
        }
    }
    
    // 2. 如果缓存未找到,从所有配置的服务器中查找
    let all_configs = self.client_pool.get_all_server_configs().await;
    
    for config in all_configs {
        // 尝试连接并获取工具列表
        if let Ok(client) = self.client_pool.get_or_create_client(&config.id).await {
            if let Ok(tools_list) = client.list_tools().await {
                // 更新缓存
                self.update_tools_cache(&config.id, tools_list.tools.clone()).await;
                
                // 查找目标工具
                if let Some(tool) = tools_list.tools.iter().find(|t| t.name == tool_name) {
                    return Some((config.id.clone(), tool.clone()));
                }
            }
        }
    }
    
    None
}
```

**优势**:
1. **性能**:优先使用缓存,避免重复连接
2. **可靠性**:即使服务器未连接,也能找到并自动连接
3. **完整性**:查询所有已配置的服务器,不遗漏任何工具

## 连接生命周期

```
用户配置服务器
    ↓
MCPConfigManager 持久化配置
    ↓
(可选)调用 connect_server()
    ↓
MCPClientPool.create_client()
    ↓
client.initialize() - 设置连接状态
    ↓
客户端实例保存到内存池
    ↓
list_servers() 可以看到该服务器
    ↓
应用重启
    ↓
内存池清空,但配置仍在
    ↓
需要重新连接
```

## 总结

系统通过以下机制判断 MCP 服务器是否已连接:

1. **客户端实例存在性**:客户端是否在 `MCPClientPool` 的内存池中
2. **连接状态标志**:客户端的 `status` 字段是否为 `Connected`
3. **传输层验证**:
   - STDIO:子进程是否成功启动
   - HTTP/SSE:HTTP 请求是否成功

之前的自动检测失败是因为依赖 `list_servers()`,它只返回已创建实例的服务器。新的 `find_tool_by_name` 方法通过查询配置管理器和按需连接解决了这个问题。

