# MCP 服务器连接判断机制详解

## 核心问题解答

### 1. 连接判断机制是什么?

MCP 服务器的连接状态判断采用**客户端实例 + 状态标志**的机制:

#### 层级 1: 客户端实例存在性
```rust
// rust-lib/flowy-ai/src/mcp/manager.rs:182-184
pub fn is_server_connected(&self, server_id: &str) -> bool {
    self.client_pool.has_client(server_id)
}
```
- 检查服务器 ID 是否在内存 `HashMap<String, Arc<RwLock<Box<dyn MCPClient>>>>` 中
- 如果客户端实例不存在 → 未连接
- 如果客户端实例存在 → 需要进一步检查状态

#### 层级 2: 连接状态标志
```rust
// rust-lib/flowy-ai/src/mcp/client.rs:21
fn get_status(&self) -> MCPConnectionStatus;

pub enum MCPConnectionStatus {
    Disconnected,    // 未连接
    Connecting,      // 连接中
    Connected,       // 已连接
    Error(String),   // 连接错误
}
```
- 每个客户端维护一个 `status` 字段
- 通过 `client.get_status()` 获取实时状态

#### 层级 3: 传输层验证
不同传输类型有不同的判断标准:

**STDIO**:
```rust
// 子进程是否成功启动
match cmd.spawn() {
    Ok(process) => {
        self.process = Some(process);
        self.status = MCPConnectionStatus::Connected;
    }
    Err(e) => {
        self.status = MCPConnectionStatus::Error(error_msg);
    }
}
```

**HTTP/SSE**:
```rust
// HTTP 请求是否成功
match request.send().await {
    Ok(response) if response.status().is_success() => {
        self.status = MCPConnectionStatus::Connected;
    }
    _ => {
        self.status = MCPConnectionStatus::Error(error_msg);
    }
}
```

### 2. 状态刷新频率

**❌ 没有自动定期刷新机制**

系统**不会**主动定期检查连接状态。连接状态只在以下情况更新:

#### 主动触发场景:

1. **用户手动连接**
   ```dart
   // UI 触发连接
   bloc.add(MCPSettingsEvent.connectServer(serverId));
   ```

2. **用户手动断开**
   ```dart
   bloc.add(MCPSettingsEvent.disconnectServer(serverId));
   ```

3. **一键检查按钮**
   ```dart
   // 批量连接所有未连接的服务器
   _checkAllServers(context, state);
   ```

4. **页面加载时**
   ```dart
   // 页面初始化
   on<_Started>((event, emit) async {
     await _handleLoadServerList(emit);
   });
   ```

5. **添加/更新服务器配置**
   ```rust
   // 如果配置为激活状态，自动尝试连接
   if config.is_active {
       ai_manager.mcp_manager.connect_server(config).await;
   }
   ```

#### 被动更新场景:

连接状态在客户端内部操作时可能会变化,但**不会主动通知 UI**,除非:
- 客户端在调用工具时发现连接失败,会更新状态为 `Error`
- 进程意外终止(STDIO 类型),状态可能变为 `Disconnected`

### 3. 重试机制

**❌ 没有自动重试机制**

当连接失败时,系统**不会**自动重试。需要手动操作:

#### 手动重连方法:

**方法 1: 用户点击"连接"按钮**
```rust
// rust-lib/flowy-ai/src/mcp/event_handler.rs:306-360
pub async fn connect_mcp_server_handler(...) -> DataResult<MCPServerStatusPB, FlowyError> {
    // 尝试连接
    match ai_manager.mcp_manager.connect_server(config).await {
        Ok(()) => {
            status.is_connected = true;
            // 成功
        }
        Err(e) => {
            // 失败,但不会自动重试
            status.error_message = Some(format!("Connection failed: {}", e));
        }
    }
}
```

**方法 2: 调用重连 API**
```rust
// rust-lib/flowy-ai/src/mcp/manager.rs:138-154
pub async fn reconnect_server(&self, server_id: &str) -> Result<(), FlowyError> {
    self.client_pool.reconnect_client(server_id).await?;
    // 重新发现工具
    self.tool_discovery.discover_tools(server_id).await;
}
```

**重连逻辑**:
```rust
// rust-lib/flowy-ai/src/mcp/client_pool.rs:139-169
pub async fn reconnect_client(&self, server_id: &str) -> Result<(), FlowyError> {
    // 1. 获取原配置
    let config = get_existing_config(server_id)?;
    
    // 2. 移除旧客户端
    self.remove_client(server_id).await?;
    
    // 3. 创建新客户端
    self.create_client(config).await?;
    
    // 4. 更新重连统计
    metadata.connection_attempts += 1;
    metadata.last_connected = Some(SystemTime::now());
}
```

#### 为什么没有自动重试?

1. **设计理念**: MCP 服务器通常是用户主动配置和管理的外部服务,不应在后台静默重连
2. **资源考虑**: 自动重试可能导致不必要的资源消耗(特别是 STDIO 类型会启动进程)
3. **用户感知**: 让用户明确知道连接状态,而不是隐藏在后台

### 4. 调用时自动连接机制

**✅ 存在按需连接机制**

虽然没有自动重试,但在**调用工具时**会尝试自动连接:

#### find_tool_by_name 的智能连接

```rust
// rust-lib/flowy-ai/src/mcp/tool_discovery.rs:99-134
pub async fn find_tool_by_name(&self, tool_name: &str) -> Option<(String, MCPTool)> {
    // 1. 先从缓存查找(已连接的服务器)
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
        // ✅ 尝试连接并获取工具列表 (按需连接)
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

**按需连接的触发场景**:

1. **AI 调用工具时**:
   ```rust
   // rust-lib/flowy-ai/src/agent/tool_call_handler.rs:428-442
   async fn execute_auto_detected_tool(...) -> FlowyResult<String> {
       // 自动查找工具,如果服务器未连接,会尝试连接
       match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
           Some((server_id, tool)) => {
               // 找到工具,服务器已自动连接
               self.execute_mcp_tool(&server_id, request).await
           }
           None => {
               // 未找到工具
               self.execute_native_tool(request).await
           }
       }
   }
   ```

2. **智能体创建/更新时**:
   ```rust
   // rust-lib/flowy-ai/src/ai_manager.rs
   async fn discover_available_tools(&self) -> Vec<String> {
       // 从所有配置的 MCP 服务器中获取工具
       // 如果服务器未连接,会尝试连接
       let all_servers = self.mcp_manager.config_manager().get_all_servers();
       
       for server in all_servers {
           // 按需连接
           if let Some((_, tools)) = self.mcp_manager.find_tool_by_name("*").await {
               // ...
           }
       }
   }
   ```

## 连接生命周期完整流程

### 1. 初始状态
```
配置已保存 → 客户端实例不存在 → 状态: 未连接
```

### 2. 用户触发连接
```
用户点击"连接"
    ↓
UI: bloc.add(MCPSettingsEvent.connectServer(serverId))
    ↓
Bloc: AIEventConnectMCPServer(request).send()
    ↓
Rust: connect_mcp_server_handler()
    ↓
Manager: connect_server(config)
    ↓
ClientPool: create_client(config)
    ↓
Client: initialize() - 尝试建立连接
    ↓
成功: status = Connected
失败: status = Error(message)
    ↓
返回 MCPServerStatusPB 给 UI
    ↓
UI: 更新状态徽章显示
```

### 3. 连接保持
```
客户端实例存在于内存中
    ↓
状态标志: Connected
    ↓
可以调用工具
    ↓
如果调用失败,状态可能变为 Error
```

### 4. 应用重启
```
应用关闭
    ↓
stop_all_clients() - 停止所有客户端
    ↓
内存清空,客户端实例消失
    ↓
配置仍保存在 SQLite
    ↓
应用重启
    ↓
需要重新连接
```

## 健康检查机制

虽然没有自动定期检查,但系统提供了健康检查 API:

```rust
// rust-lib/flowy-ai/src/mcp/manager.rs:157-159
pub async fn health_check(&self) -> Vec<(String, MCPConnectionStatus)> {
    self.client_pool.health_check().await
}
```

```rust
// rust-lib/flowy-ai/src/mcp/client_pool.rs:200-211
pub async fn health_check(&self) -> Vec<(String, MCPConnectionStatus)> {
    let clients = self.clients.read().await;
    let mut results = Vec::new();
    
    for (server_id, client) in clients.iter() {
        let client_guard = client.read().await;
        let status = client_guard.get_status();
        results.push((server_id.clone(), status));
    }
    
    results
}
```

**健康检查只读取客户端的当前状态,不会主动测试连接。**

## UI 状态更新机制

### 状态获取流程

```dart
// 1. UI 请求连接
bloc.add(MCPSettingsEvent.connectServer(serverId));

// 2. Bloc 调用后端
final result = await AIEventConnectMCPServer(request).send();

// 3. 接收状态并更新
result.fold(
    (status) {
        // 更新 BLoC 状态
        add(MCPSettingsEvent.didReceiveServerStatus(status));
    },
    (error) {
        // 显示错误
    },
);

// 4. BLoC 发射新状态
void _handleDidReceiveServerStatus(MCPServerStatusPB status, emit) {
    final updatedStatuses = Map.from(state.serverStatuses);
    updatedStatuses[status.serverId] = status;
    emit(state.copyWith(serverStatuses: updatedStatuses));
}

// 5. UI 自动重建
BlocConsumer<MCPSettingsBloc, MCPSettingsState>(
    builder: (context, state) {
        final serverStatus = state.serverStatuses[server.id];
        final isConnected = serverStatus?.isConnected ?? false;
        // 显示连接状态徽章
        return _buildConnectionStatusBadge(context, isConnected, ...);
    },
)
```

### 状态缓存

```dart
// BLoC 状态中缓存所有服务器的连接状态
@freezed
class MCPSettingsState with _$MCPSettingsState {
    const factory MCPSettingsState({
        @Default([]) List<MCPServerConfigPB> servers,
        @Default({}) Map<String, MCPServerStatusPB> serverStatuses,  // ← 状态缓存
        @Default({}) Map<String, List<MCPToolPB>> serverTools,
        // ...
    }) = _MCPSettingsState;
}
```

## 改进建议

### 潜在改进 1: 添加定期健康检查

```rust
// 可以在 AIManager 中添加后台任务
pub struct AIManager {
    // ...
    health_check_interval: Duration,
}

impl AIManager {
    pub async fn start_health_check_loop(&self) {
        let interval = tokio::time::interval(Duration::from_secs(60)); // 每分钟检查一次
        
        loop {
            interval.tick().await;
            
            let statuses = self.mcp_manager.health_check().await;
            for (server_id, status) in statuses {
                if matches!(status, MCPConnectionStatus::Error(_)) {
                    // 可以选择自动重连或通知 UI
                    tracing::warn!("Server {} is in error state", server_id);
                }
            }
        }
    }
}
```

### 潜在改进 2: 添加自动重连策略

```rust
pub struct ReconnectPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl MCPClientPool {
    pub async fn connect_with_retry(
        &self,
        config: MCPServerConfig,
        policy: ReconnectPolicy,
    ) -> Result<(), FlowyError> {
        let mut attempts = 0;
        let mut delay = policy.base_delay;
        
        loop {
            attempts += 1;
            
            match self.create_client(config.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) if attempts >= policy.max_attempts => return Err(e),
                Err(e) => {
                    tracing::warn!("Connection attempt {} failed: {}", attempts, e);
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(policy.max_delay); // 指数退避
                }
            }
        }
    }
}
```

### 潜在改进 3: 连接状态变化通知

```rust
pub trait MCPConnectionObserver {
    fn on_connection_changed(&self, server_id: &str, status: MCPConnectionStatus);
}

// 在状态变化时通知观察者
impl MCPClientPool {
    pub fn add_observer(&self, observer: Arc<dyn MCPConnectionObserver>) {
        // ...
    }
    
    fn notify_status_change(&self, server_id: &str, status: MCPConnectionStatus) {
        for observer in &self.observers {
            observer.on_connection_changed(server_id, status);
        }
    }
}
```

## 总结

### 当前机制特点

✅ **优点**:
- 简单明确:用户完全掌控连接状态
- 资源高效:不浪费资源在无用的重连上
- 按需连接:调用工具时自动连接未连接的服务器
- 状态透明:UI 清晰显示每个服务器的连接状态

❌ **限制**:
- 无自动重试:连接失败需手动重连
- 无定期检查:无法主动发现连接断开
- 状态滞后:只在主动操作时更新状态
- 应用重启需重连:连接不持久化

### 最佳实践

1. **启动应用时**:使用"一键检查"连接所有服务器
2. **配置新服务器时**:确保 `is_active = true` 以自动连接
3. **发现连接错误时**:查看 Tooltip 中的错误信息,修复配置后手动重连
4. **调用工具前**:无需担心连接状态,系统会自动按需连接
5. **长时间运行后**:偶尔手动刷新连接状态确保可用性

这种设计在用户控制和自动化之间取得了平衡,适合 MCP 这种外部服务管理的场景。

