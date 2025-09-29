# 工具注册表 (Tool Registry)

工具注册表是AppFlowy MCP AI Agent系统的核心组件，负责统一管理所有类型的工具，包括MCP工具、原生工具、搜索工具和外部API工具。

## 功能特性

### 1. 统一工具管理
- **多类型支持**: 支持MCP、原生、搜索、外部API等多种工具类型
- **动态注册**: 支持运行时动态注册和注销工具
- **版本管理**: 完整的工具版本控制和兼容性管理
- **状态管理**: 工具状态跟踪（可用、不可用、已禁用、维护中、已弃用）

### 2. 权限和安全
- **权限控制**: 集成工具安全管理器，确保工具执行安全
- **权限检查**: 基于工具类型和用户权限的细粒度访问控制
- **安全策略**: 支持自定义安全策略和执行确认机制

### 3. 工具发现
- **自动发现**: 自动发现MCP服务器提供的工具
- **智能搜索**: 支持基于名称、描述、类型等多维度搜索
- **过滤功能**: 丰富的过滤选项（类型、状态、权限、来源等）

### 4. 使用统计
- **调用统计**: 记录工具调用次数、成功率、执行时间等
- **性能监控**: 平均执行时间和性能指标跟踪
- **用户评分**: 支持用户对工具进行评分

### 5. 配置管理
- **工具配置**: 支持超时、重试、缓存等配置选项
- **依赖管理**: 工具间依赖关系管理
- **导入导出**: 支持注册表的导入导出功能

## 核心组件

### ToolRegistry
主要的工具注册表类，提供以下核心功能：

```rust
// 初始化注册表
registry.initialize().await?;

// 注册工具
let request = ToolRegistrationRequest {
    definition: tool_definition,
    config: Some(tool_config),
    dependencies: vec![],
    overwrite: false,
};
registry.register_tool(request).await?;

// 搜索工具
let results = registry.search_tools("search query", Some(filter)).await;

// 检查权限
let permission = registry.check_tool_permission(
    "tool_name", 
    ToolTypePB::MCP, 
    Some("server_id")
).await?;
```

### RegisteredTool
注册的工具信息，包含：
- 工具定义 (ToolDefinitionPB)
- 注册时间和更新时间
- 工具状态 (ToolStatus)
- 使用统计 (ToolUsageStats)
- 工具配置 (ToolConfig)
- 依赖关系

### ToolStatus
工具状态枚举：
- `Available`: 可用
- `Unavailable`: 不可用
- `Disabled`: 已禁用
- `Maintenance`: 维护中
- `Deprecated`: 已弃用

### ToolConfig
工具配置选项：
- 超时设置
- 重试次数
- 缓存策略
- 并发限制
- 自定义配置

## 使用示例

### 1. 基本使用

```rust
use crate::agent::tool_registry::{ToolRegistry, ToolRegistrationRequest};
use crate::mcp::tool_security::ToolSecurityManager;

// 创建工具注册表
let security_manager = Arc::new(ToolSecurityManager::new(store_preferences.clone()));
let registry = Arc::new(ToolRegistry::new(security_manager, store_preferences));

// 初始化（会自动注册内置工具）
registry.initialize().await?;
```

### 2. 注册自定义工具

```rust
let tool_definition = ToolDefinitionPB {
    name: "my_custom_tool".to_string(),
    description: "我的自定义工具".to_string(),
    tool_type: ToolTypePB::Native,
    source: "my_app".to_string(),
    parameters_schema: json!({
        "type": "object",
        "properties": {
            "input": {"type": "string", "description": "输入参数"}
        },
        "required": ["input"]
    }).to_string(),
    permissions: vec!["custom.execute".to_string()],
    is_available: true,
    metadata: HashMap::new(),
};

let request = ToolRegistrationRequest {
    definition: tool_definition,
    config: Some(ToolConfig {
        timeout_seconds: Some(30),
        retry_count: Some(3),
        cache_policy: CachePolicy::Short,
        concurrency_limit: Some(5),
        custom_config: HashMap::new(),
    }),
    dependencies: Vec::new(),
    overwrite: false,
};

registry.register_tool(request).await?;
```

### 3. 搜索和过滤工具

```rust
// 基本搜索
let results = registry.search_tools("文档", None).await;

// 带过滤条件的搜索
let filter = ToolSearchFilter {
    tool_types: Some(vec![ToolTypePB::Native, ToolTypePB::MCP]),
    statuses: Some(vec![ToolStatus::Available]),
    required_permissions: Some(vec!["document.read".to_string()]),
    sources: None,
    tags: None,
    min_rating: Some(4.0),
};

let filtered_results = registry.search_tools("", Some(filter)).await;
```

### 4. 权限检查

```rust
match registry.check_tool_permission("tool_name", ToolTypePB::MCP, Some("server_id")).await? {
    ToolExecutionPermission::AutoExecute => {
        // 可以自动执行
        execute_tool().await?;
    },
    ToolExecutionPermission::RequireConfirmation(msg) => {
        // 需要用户确认
        if confirm_with_user(&msg).await? {
            execute_tool().await?;
        }
    },
    ToolExecutionPermission::Denied(reason) => {
        // 被拒绝执行
        return Err(FlowyError::permission_denied().with_context(reason));
    }
}
```

### 5. MCP工具集成

```rust
// 当MCP服务器连接时自动注册工具
registry.discover_mcp_tools("server_id", mcp_tools).await?;

// 当MCP服务器断开时清理工具
registry.cleanup_server_tools("server_id").await?;
```

### 6. 统计和监控

```rust
// 更新工具使用统计
registry.update_tool_usage("tool_name", ToolTypePB::Native, 150, true).await?;

// 获取统计信息
let stats = registry.get_tool_statistics().await;
println!("总工具数: {}", stats.total_tools);
println!("成功调用: {}", stats.successful_calls);
println!("失败调用: {}", stats.failed_calls);
```

## 集成指南

### 1. 在智能体管理器中使用

工具注册表已集成到 `AgentManager` 中：

```rust
let agent_manager = AgentManager::new(ai_manager);
agent_manager.initialize().await?;

// 搜索工具
let tools = agent_manager.search_tools("搜索", None).await;

// 获取统计信息
let stats = agent_manager.get_tool_statistics().await;
```

### 2. MCP服务器事件处理

```rust
// 服务器连接时
agent_manager.on_mcp_server_connected("server_id").await?;

// 服务器断开时
agent_manager.on_mcp_server_disconnected("server_id").await?;
```

### 3. 导入导出

```rust
// 导出注册表
let export_data = registry.export_registry().await?;

// 导入注册表（合并模式）
registry.import_registry(&export_data, true).await?;
```

## 扩展性

### 1. 自定义工具发现监听器

```rust
struct MyToolDiscoveryListener;

impl ToolDiscoveryListener for MyToolDiscoveryListener {
    fn on_tool_discovered(&self, tool: &RegisteredTool) {
        println!("发现新工具: {}", tool.definition.name);
    }
    
    fn on_tool_removed(&self, tool_name: &str, tool_type: ToolTypePB) {
        println!("工具已移除: {} ({:?})", tool_name, tool_type);
    }
    
    fn on_tool_status_changed(&self, tool_name: &str, old_status: ToolStatus, new_status: ToolStatus) {
        println!("工具状态变更: {} {:?} -> {:?}", tool_name, old_status, new_status);
    }
}

// 添加监听器
registry.add_discovery_listener(Box::new(MyToolDiscoveryListener)).await;
```

### 2. 自定义缓存策略

```rust
let config = ToolConfig {
    cache_policy: CachePolicy::Custom(3600), // 1小时缓存
    ..Default::default()
};
```

## 最佳实践

1. **初始化**: 在应用启动时调用 `initialize()` 方法
2. **权限检查**: 在执行工具前始终检查权限
3. **统计更新**: 在工具执行后更新使用统计
4. **状态管理**: 及时更新工具状态以反映实际可用性
5. **错误处理**: 妥善处理工具注册和执行中的错误
6. **性能监控**: 定期检查工具统计信息以优化性能

## 注意事项

1. 工具注册表使用异步操作，确保在异步上下文中使用
2. 大量工具注册可能影响启动性能，考虑延迟加载
3. 工具权限检查会增加执行开销，在性能敏感场景中需要权衡
4. 定期清理不再使用的工具以节省内存和存储空间
5. 在多线程环境中，工具注册表是线程安全的，但要注意避免死锁
