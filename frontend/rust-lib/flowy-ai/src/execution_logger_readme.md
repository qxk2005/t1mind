# ExecutionLogger - 执行日志记录器

ExecutionLogger是一个完整的执行日志管理系统，支持任务执行的完整生命周期管理，包括记录、查询、过滤和导出功能。

## 功能特性

### 核心功能
- **执行日志管理**: 创建、更新、查询和删除执行日志
- **执行步骤追踪**: 记录每个执行步骤的详细信息
- **引用信息管理**: 支持多种类型的引用信息记录
- **MCP工具管理**: 管理和统计MCP工具的使用情况
- **高效查询**: 支持多种条件的复合查询和过滤
- **多格式导出**: 支持JSON、CSV、HTML、纯文本等多种导出格式

### 数据模型
- `ExecutionLogTable`: 执行日志主表
- `ExecutionStepTable`: 执行步骤表
- `ExecutionReferenceTable`: 执行引用表
- `McpToolInfoTable`: MCP工具信息表

## 使用示例

### 基本使用

```rust
use flowy_ai::execution_logger::{ExecutionLogger, ExecutionLogStatus, ExecutionStepStatus};

// 创建ExecutionLogger实例
let logger = ExecutionLogger::new(user_service);

// 创建执行日志
let execution_id = logger.create_execution_log(
    "session_123".to_string(),
    "用户查询内容".to_string(),
    Some("task_plan_456".to_string()),
    Some("agent_789".to_string()),
    Some("user_001".to_string()),
    Some("workspace_002".to_string()),
).await?;

// 更新执行状态
logger.update_execution_status(
    &execution_id,
    ExecutionLogStatus::Running,
    None,
    None,
).await?;

// 添加执行步骤
let step_id = logger.add_execution_step(
    &execution_id,
    "步骤名称".to_string(),
    "步骤描述".to_string(),
    "mcp_tool_001".to_string(),
    "文件搜索工具".to_string(),
    1,
).await?;

// 更新步骤状态
logger.update_step_status(
    &step_id,
    ExecutionStepStatus::Success,
    Some(1500), // 执行时间1.5秒
    Some(serde_json::json!({"result": "success"})),
    None,
    None,
).await?;
```

### 查询和过滤

```rust
use flowy_ai::execution_logger::{ExecutionLogSearchCriteria, ExecutionLogStatus};

// 创建搜索条件
let criteria = ExecutionLogSearchCriteria::new()
    .with_session_id("session_123".to_string())
    .with_status(ExecutionLogStatus::Completed)
    .with_limit(50);

// 搜索执行日志
let logs = logger.search_execution_logs(criteria).await?;

// 获取详细信息（包含步骤和引用）
if let Some((log, steps)) = logger.get_execution_log_with_details(&execution_id).await? {
    println!("执行日志: {:?}", log);
    for (step, references) in steps {
        println!("步骤: {:?}", step);
        for reference in references {
            println!("引用: {:?}", reference);
        }
    }
}
```

### 导出功能

```rust
use flowy_ai::execution_logger::{ExecutionLogExportFormat, ExecutionLogExportOptions};

// 创建导出选项
let export_options = ExecutionLogExportOptions {
    format: ExecutionLogExportFormat::Json,
    include_steps: true,
    include_references: true,
    include_metadata: true,
    include_error_details: true,
    ..Default::default()
};

// 导出执行日志
let exported_data = logger.export_execution_logs(criteria, export_options).await?;
println!("导出的数据: {}", exported_data);
```

### MCP工具管理

```rust
use flowy_ai::execution_logger::{McpToolInfoTable, McpToolStatus};

// 注册MCP工具
let tool_info = McpToolInfoTable {
    id: "tool_001".to_string(),
    name: "文件搜索工具".to_string(),
    description: "用于搜索文件的MCP工具".to_string(),
    status: McpToolStatus::Available as i32,
    ..Default::default()
};

logger.upsert_mcp_tool_info(tool_info).await?;

// 更新工具使用统计
logger.update_mcp_tool_usage("tool_001", true, 1500).await?;

// 获取所有工具信息
let tools = logger.get_all_mcp_tools().await?;
```

### 统计信息

```rust
// 获取执行统计信息
let stats = logger.get_execution_statistics(
    Some(start_timestamp),
    Some(end_timestamp),
    Some("workspace_002".to_string()),
).await?;

println!("总执行次数: {}", stats.total_executions);
println!("成功执行次数: {}", stats.successful_executions);
println!("平均执行时间: {}ms", stats.average_execution_time_ms);
```

## 数据库架构

### 表结构
- `execution_log_table`: 执行日志主表，存储执行的基本信息
- `execution_step_table`: 执行步骤表，存储每个步骤的详细信息
- `execution_reference_table`: 执行引用表，存储步骤相关的引用信息
- `mcp_tool_info_table`: MCP工具信息表，存储工具的元数据和统计信息

### 索引优化
- 为常用查询字段创建了索引，包括session_id、status、start_time等
- 支持高效的范围查询和复合条件查询

## 性能特性

- **批量操作**: 支持批量插入和更新操作
- **索引优化**: 为常用查询字段建立了合适的索引
- **内存效率**: 使用流式处理避免大量数据的内存占用
- **异步处理**: 所有数据库操作都是异步的，不会阻塞主线程

## 错误处理

ExecutionLogger提供了完整的错误处理机制：
- 数据库连接错误
- 数据验证错误
- 序列化/反序列化错误
- 业务逻辑错误

所有方法都返回`FlowyResult<T>`，可以方便地进行错误处理和传播。

## 扩展性

ExecutionLogger的设计支持未来的扩展：
- 新的导出格式可以通过实现相应的导出方法来添加
- 新的查询条件可以通过扩展`ExecutionLogSearchCriteria`来支持
- 新的引用类型可以通过扩展`ExecutionReferenceType`枚举来添加

## 注意事项

1. **数据库迁移**: 使用ExecutionLogger前需要运行相应的数据库迁移
2. **权限管理**: 确保用户服务提供正确的数据库连接权限
3. **数据清理**: 定期使用`cleanup_old_logs`方法清理旧的日志数据
4. **性能监控**: 在高并发场景下注意监控数据库性能
