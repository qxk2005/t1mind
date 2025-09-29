# 智能体配置管理器实现

本模块实现了AppFlowy MCP AI Agent规范中的智能体配置管理功能，提供完整的CRUD操作、数据验证和个性化设置支持。

## 功能概述

### 1. 智能体配置管理
- 创建、读取、更新、删除智能体配置
- 支持智能体状态管理（活跃、暂停、已删除）
- 配置验证和数据一致性保证
- 个性化设置和元数据支持

### 2. 全局设置管理
- 智能体功能开关控制
- 默认能力配置管理
- 执行超时和调试设置
- 系统级参数配置

### 3. 数据持久化
- 使用KVStorePreferences进行SQLite存储
- 支持配置版本控制和迁移
- 事务操作保证数据一致性
- 配置导出导入功能

### 4. 验证机制
- 创建和更新请求验证
- 智能体配置完整性检查
- 能力配置范围验证
- 数据格式和约束验证

## 核心组件

### AgentConfigManager
主要的配置管理器，提供以下功能：
- `create_agent()` - 创建智能体配置
- `get_agent()` - 获取智能体配置
- `update_agent()` - 更新智能体配置
- `delete_agent()` - 删除智能体配置
- `get_all_agents()` - 获取所有智能体
- `get_active_agents()` - 获取活跃智能体
- `export_config()` - 导出配置
- `import_config()` - 导入配置

### AgentGlobalSettings
全局智能体设置结构：
```rust
pub struct AgentGlobalSettings {
    pub enabled: bool,
    pub default_max_planning_steps: i32,
    pub default_max_tool_calls: i32,
    pub default_memory_limit: i32,
    pub debug_logging: bool,
    pub execution_timeout: u64,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
```

## 数据模型

### 智能体配置 (AgentConfigPB)
- **id**: 唯一标识符
- **name**: 智能体名称
- **description**: 功能描述
- **avatar**: 头像/图标
- **personality**: 个性描述（系统提示词）
- **capabilities**: 能力配置
- **available_tools**: 可用工具列表
- **status**: 状态（活跃/暂停/已删除）
- **created_at/updated_at**: 时间戳
- **metadata**: 扩展元数据

### 能力配置 (AgentCapabilitiesPB)
- **enable_planning**: 启用任务规划
- **enable_tool_calling**: 启用工具调用
- **enable_reflection**: 启用反思机制
- **enable_memory**: 启用会话记忆
- **max_planning_steps**: 最大规划步骤数
- **max_tool_calls**: 最大工具调用次数
- **memory_limit**: 会话记忆长度限制

## 验证规则

### 基本验证
- 智能体ID和名称不能为空
- 名称长度和格式检查
- 必填字段完整性验证

### 能力配置验证
- 最大规划步骤数：1-100
- 最大工具调用次数：1-1000
- 会话记忆长度限制：10-10000

### 数据一致性
- 唯一ID生成和冲突检查
- 状态转换合法性验证
- 配置更新原子性保证

## 版本控制

### 配置版本管理
- 当前版本：v1
- 支持配置迁移机制
- 向前兼容性保证
- 版本升级自动处理

### 导出导入
- 完整配置导出（全局设置+所有智能体）
- 选择性导入和错误处理
- 版本兼容性检查
- 导入结果详细报告

## 使用示例

```rust
use flowy_ai::agent::{AgentConfigManager, AgentGlobalSettings};
use flowy_ai::entities::{CreateAgentRequestPB, AgentCapabilitiesPB};

// 创建配置管理器
let manager = AgentConfigManager::new(store_preferences);

// 创建智能体
let request = CreateAgentRequestPB {
    name: "我的助手".to_string(),
    description: "智能助手".to_string(),
    personality: "你是一个友好的助手".to_string(),
    capabilities: AgentCapabilitiesPB::default_capabilities(),
    available_tools: vec!["search".to_string()],
    // ...
};

let agent = manager.create_agent(request)?;

// 获取智能体
let get_request = GetAgentRequestPB { id: agent.id.clone() };
let retrieved_agent = manager.get_agent(get_request)?;

// 更新智能体
let update_request = UpdateAgentRequestPB {
    id: agent.id.clone(),
    name: Some("更新后的名称".to_string()),
    // ...
};
let updated_agent = manager.update_agent(update_request)?;

// 删除智能体
let delete_request = DeleteAgentRequestPB { id: agent.id };
manager.delete_agent(delete_request)?;
```

## 测试覆盖

实现包含完整的单元测试：
- 全局设置管理测试
- CRUD操作测试
- 列表和过滤测试
- 验证机制测试
- 导出导入测试

所有测试均通过，确保功能稳定可靠。

## 集成说明

该配置管理器已集成到flowy-ai模块中，可通过以下方式使用：

```rust
use flowy_ai::agent::{AgentConfigManager, AgentGlobalSettings};
```

配置管理器遵循现有的数据库访问模式，与AppFlowy的整体架构保持一致。
