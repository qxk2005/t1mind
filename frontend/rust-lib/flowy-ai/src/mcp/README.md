# MCP配置存储与工具管理实现

本模块实现了AppFlowy MCP AI Agent规范中的配置存储和工具管理功能，完全符合MCP（Model Context Protocol）标准规范。

## 功能概述

### 1. 全局MCP设置管理
- 启用/禁用MCP功能
- 连接超时配置
- 工具调用超时配置
- 最大并发连接数
- 调试日志开关

### 2. MCP服务器配置管理
- 支持STDIO、HTTP、SSE三种传输方式
- 服务器基本信息（ID、名称、图标、描述）
- 激活状态管理
- 传输方式特定配置：
  - STDIO: 命令、参数、环境变量
  - HTTP/SSE: URL、HTTP头信息

### 3. 数据持久化
- 使用现有的KVStorePreferences进行SQLite存储
- 配置验证和序列化
- 支持配置迁移和版本控制

### 4. 批量操作
- 获取所有/激活的服务器
- 按传输类型过滤服务器
- 配置导出导入功能

## 核心组件

### MCPConfigManager
主要的配置管理器，提供以下功能：
- `get_global_settings()` - 获取全局设置
- `save_global_settings()` - 保存全局设置
- `get_all_servers()` - 获取所有服务器配置
- `save_server()` - 保存服务器配置
- `delete_server()` - 删除服务器配置
- `get_active_servers()` - 获取激活的服务器
- `export_config()` - 导出配置
- `import_config()` - 导入配置

### MCPGlobalSettings
全局MCP设置结构：
```rust
pub struct MCPGlobalSettings {
    pub enabled: bool,
    pub connection_timeout: u64,
    pub tool_call_timeout: u64,
    pub max_concurrent_connections: u32,
    pub debug_logging: bool,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
```

## 集成到MCPClientManager

MCPClientManager现在集成了配置管理功能：
- `config_manager()` - 获取配置管理器
- `connect_server_from_config()` - 从配置连接服务器
- `connect_all_active_servers()` - 连接所有激活的服务器
- `save_and_connect_server()` - 保存配置并连接
- `delete_server_config()` - 删除配置并断开连接
- `test_server_config()` - 测试服务器配置

## 使用示例

参见 `config_example.rs` 文件，展示了完整的使用流程：

1. 创建配置管理器
2. 配置全局设置
3. 添加STDIO和HTTP服务器
4. 批量查询和管理
5. 配置导出导入

## 数据存储结构

配置数据使用以下键存储在KVStorePreferences中：
- `mcp_global_settings` - 全局设置
- `mcp_server_list` - 服务器ID列表
- `mcp_config:server:{server_id}` - 单个服务器配置

## 配置验证

实现了完整的配置验证：
- 必填字段检查
- 传输方式特定配置验证
- URL格式验证
- 数据一致性检查

## 测试覆盖

包含全面的单元测试：
- 全局设置管理
- 服务器配置CRUD
- 配置验证
- 激活状态管理
- 传输类型过滤
- 导出导入功能

## MCP工具标准处理

### 符合MCP协议的工具定义

根据MCP标准规范，每个工具包含以下属性：

#### 基本属性
- **name**: 工具的唯一标识符
- **description**: 工具功能的描述
- **inputSchema**: JSON Schema定义的输入参数

#### 工具注解（Annotations）
- **title**: 工具的可读标题，适用于UI显示
- **readOnlyHint**: 指示工具是否为只读操作
- **destructiveHint**: 指示工具是否可能执行破坏性操作
- **idempotentHint**: 指示工具的操作是否为幂等的
- **openWorldHint**: 指示工具是否可能与外部实体交互

### 工具安全级别分类

系统根据工具注解自动分类安全级别：

1. **只读工具** - 安全的查询操作，可自动执行
2. **安全工具** - 一般操作，根据策略决定是否需要确认
3. **外部交互工具** - 与外部服务交互，通常需要用户确认
4. **破坏性工具** - 可能造成数据丢失或系统变更，必须用户确认

### 工具安全管理

#### ToolSecurityManager
- 工具权限检查和执行控制
- 安全策略配置管理
- 工具调用记录和审计
- 速率限制和访问控制

#### 安全策略配置
- 自动执行策略（按安全级别）
- 工具禁用/信任列表
- 速率限制配置
- 用户确认要求

#### 工具调用审计
- 完整的调用记录
- 安全级别统计
- 用户确认跟踪
- 执行结果记录

### 工具发现和管理

#### ToolDiscoveryManager
- 自动发现MCP端点提供的工具
- 工具注册表管理
- 基于安全级别的工具过滤
- 工具搜索和分类

## 使用示例

### 创建符合MCP标准的工具

```rust
use crate::mcp::{MCPTool, MCPToolAnnotations};
use serde_json::json;

// 只读工具
let read_tool = MCPTool::with_annotations(
    "read_file".to_string(),
    "读取文件内容".to_string(),
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "文件路径"}
        },
        "required": ["path"]
    }),
    MCPToolAnnotations {
        title: Some("文件读取器".to_string()),
        read_only_hint: Some(true),
        destructive_hint: Some(false),
        idempotent_hint: Some(true),
        open_world_hint: Some(false),
    }
);

// 破坏性工具
let delete_tool = MCPTool::with_annotations(
    "delete_file".to_string(),
    "删除文件".to_string(),
    json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "要删除的文件路径"}
        },
        "required": ["path"]
    }),
    MCPToolAnnotations::destructive_tool()
);
```

### 工具安全检查

```rust
use crate::mcp::{ToolSecurityManager, ToolExecutionPermission};

let security_manager = ToolSecurityManager::new(store_preferences);

match security_manager.check_tool_permission(&tool, "server_id") {
    ToolExecutionPermission::AutoExecute => {
        // 可以自动执行
    }
    ToolExecutionPermission::RequireConfirmation(msg) => {
        // 需要用户确认，显示确认消息
    }
    ToolExecutionPermission::Denied(reason) => {
        // 被拒绝执行，显示拒绝原因
    }
}
```

## 下一步

这个配置存储和工具管理模块为后续的MCP功能提供了完整的基础：
1. 事件处理器将使用这些配置和安全管理功能
2. Flutter UI将通过事件系统调用这些功能
3. 智能体系统将使用这些配置来安全地管理和调用MCP工具

## 遵循的设计原则

1. **MCP标准兼容** - 完全符合MCP协议规范
2. **安全优先** - 多层次的安全检查和用户确认机制
3. **使用现有架构** - 复用KVStorePreferences和SQLite
4. **数据一致性** - 完整的验证和事务支持
5. **配置迁移** - 支持版本控制和数据迁移
6. **线程安全** - 使用Arc包装，支持并发访问
7. **错误处理** - 详细的错误信息和恢复机制
8. **审计跟踪** - 完整的工具调用记录和统计
