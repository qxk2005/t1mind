# 工具调用和任务规划集成状态

**日期**: 2025-10-02  
**状态**: 基础框架已完成，待完整集成

## 概述

实现了智能体的工具调用和任务规划的基础框架，包括协议定义、解析器、处理器等核心组件。

## 已完成的功能

### 1. 工具调用处理器 ✅

**文件**: `rust-lib/flowy-ai/src/agent/tool_call_handler.rs`

#### 核心功能
- **工具调用协议**: 定义了AI响应中的工具调用格式
- **请求解析**: 从AI响应中提取工具调用请求
- **工具执行**: 支持MCP工具和原生工具的执行
- **权限验证**: 检查智能体是否被允许使用特定工具

#### 协议格式
```
<tool_call>
{
  "id": "call_123",
  "tool_name": "search_documents",
  "arguments": {
    "query": "搜索词",
    "limit": 10
  },
  "source": "appflowy"
}
</tool_call>
```

#### 关键API
```rust
// 检测是否包含工具调用
ToolCallHandler::contains_tool_call(text: &str) -> bool

// 提取所有工具调用请求
ToolCallHandler::extract_tool_calls(text: &str) -> Vec<(ToolCallRequest, usize, usize)>

// 执行工具调用
handler.execute_tool_call(request: &ToolCallRequest, agent_config: Option<&AgentConfigPB>) -> ToolCallResponse
```

### 2. 任务规划集成器 ✅

**文件**: `rust-lib/flowy-ai/src/agent/plan_integration.rs`

#### 核心功能
- **启发式检测**: 判断何时需要创建任务计划
- **计划格式化**: 将计划转换为可读文本
- **执行事件**: 定义计划执行的各种事件类型

#### 关键API
```rust
// 检查是否应该创建计划
integration.should_create_plan(message: &str, agent_config: &AgentConfigPB) -> bool

// 格式化计划为可读文本
PlanIntegration::format_plan_for_display(plan: &TaskPlan) -> String

// 格式化执行结果
PlanIntegration::format_execution_results(plan: &TaskPlan, results: &[String]) -> String
```

#### 执行事件类型
- `PlanCreated` - 计划已创建
- `StepStarted` - 步骤开始执行
- `StepCompleted` - 步骤完成
- `StepFailed` - 步骤失败
- `PlanCompleted` - 计划完成
- `PlanFailed` - 计划失败

### 3. 模块导出 ✅

**文件**: `rust-lib/flowy-ai/src/agent/mod.rs`

已导出新模块和类型：
```rust
pub use tool_call_handler::{
    ToolCallHandler,
    ToolCallRequest,
    ToolCallResponse,
    ToolCallProtocol,
};

pub use plan_integration::{
    PlanIntegration,
    PlanExecutionEvent,
};
```

## 待实现的功能

### 1. 流式响应中的工具调用集成 ⏳

**需求**: 在聊天流式响应中实时检测和执行工具调用

**实现位置**: `rust-lib/flowy-ai/src/chat.rs` 或 `middleware/chat_service_mw.rs`

**实现思路**:
```rust
// 在流式响应处理中
while let Some(chunk) = stream.next().await {
    // 检测工具调用
    if ToolCallHandler::contains_tool_call(&chunk) {
        let calls = ToolCallHandler::extract_tool_calls(&chunk);
        
        for (request, start, end) in calls {
            // 执行工具
            let response = tool_handler.execute_tool_call(&request, agent_config).await;
            
            // 将结果发送回AI
            let result_text = ToolCallProtocol::format_response(&response);
            
            // 继续流式响应
            yield result_text;
        }
    } else {
        // 正常文本输出
        yield chunk;
    }
}
```

### 2. 任务规划的实际创建和执行 ⏳

**当前状态**: 框架已完成，但返回 `not_support` 错误

**问题**: `AIManager` 中的 `agent_manager` 是 `AgentConfigManager`，不是 `AgentManager`

**解决方案**:
1. **选项A**: 在 `AIManager` 中添加对 `AgentManager` 的引用
2. **选项B**: 将规划功能移到 `AgentConfigManager`
3. **选项C**: 创建独立的规划服务

**推荐**: 选项A - 添加 `AgentManager` 引用

```rust
// 在 AIManager 中
pub struct AIManager {
    // ...
    pub agent_config_manager: Arc<AgentConfigManager>,
    pub agent_task_manager: Arc<AgentManager>, // 新增
    // ...
}
```

### 3. 系统提示词中的工具调用说明增强 ⏳

**需求**: 在系统提示词中添加如何使用工具的详细说明

**实现位置**: `rust-lib/flowy-ai/src/agent/system_prompt.rs`

**示例**:
```rust
if cap.enable_tool_calling && !config.available_tools.is_empty() {
    prompt.push_str("## Tool Usage Protocol\n\n");
    prompt.push_str("When you need to use a tool, format your request as:\n\n");
    prompt.push_str("<tool_call>\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"id\": \"unique_call_id\",\n");
    prompt.push_str("  \"tool_name\": \"tool_name_here\",\n");
    prompt.push_str("  \"arguments\": { /* tool arguments */ },\n");
    prompt.push_str("  \"source\": \"appflowy\" or \"mcp_server_id\"\n");
    prompt.push_str("}\n");
    prompt.push_str("</tool_call>\n\n");
    
    prompt.push_str(&format!("Available tools: {}\n", config.available_tools.join(", ")));
}
```

### 4. 前端UI集成 ⏳

**需求**:
- 显示工具调用状态
- 显示任务计划和执行进度
- 显示工具调用结果

**实现位置**: `appflowy_flutter/lib/plugins/ai_chat/`

**组件**:
- `ToolCallIndicator` - 工具调用指示器
- `PlanProgressWidget` - 计划执行进度
- `ToolResultDisplay` - 工具结果显示

## 架构设计

### 工具调用流程

```
用户消息
    ↓
AI 流式响应
    ↓
ToolCallHandler.contains_tool_call() → 检测工具调用
    ↓
ToolCallHandler.extract_tool_calls() → 提取请求
    ↓
ToolCallHandler.execute_tool_call() → 执行工具
    ├─> MCP工具 (mcp_manager.call_tool)
    └─> 原生工具 (native_tools.execute_tool)
    ↓
ToolCallProtocol.format_response() → 格式化结果
    ↓
继续AI响应流
```

### 任务规划流程

```
用户消息
    ↓
AgentCapabilityExecutor.should_create_plan() → 检测是否需要规划
    ↓
PlanIntegration.create_plan_for_message() → 创建计划
    ↓
通知前端显示计划 (PlanExecutionEvent::PlanCreated)
    ↓
PlanIntegration.execute_plan() → 执行计划
    ├─> StepStarted 事件
    ├─> 执行工具调用
    ├─> StepCompleted 事件
    └─> 继续下一步
    ↓
PlanExecutionEvent::PlanCompleted
```

## 技术挑战和解决方案

### 1. 流式响应中断问题

**挑战**: 如何在流式响应中插入工具调用而不破坏流

**解决方案**: 
- 使用特殊标记 `<tool_call>...</tool_call>` 包裹工具请求
- 在流处理器中检测并提取这些标记
- 执行工具后，将结果作为新的流片段返回

### 2. AgentManager vs AgentConfigManager

**挑战**: 当前架构中有两个管理器，职责不清晰

**当前状态**:
- `AgentConfigManager` - 管理智能体配置
- `AgentManager` - 管理任务规划和执行（但未暴露给AIManager）

**解决方案**: 需要重构以统一访问或明确分离职责

### 3. 异步工具执行

**挑战**: 工具执行可能很慢，如何不阻塞UI

**解决方案**: 
- 使用异步执行
- 发送进度事件给前端
- 支持工具执行超时和取消

## 测试计划

### 单元测试

1. **工具调用协议测试**
   ```rust
   #[test]
   fn test_parse_tool_call() { /* ... */ }
   
   #[test]
   fn test_extract_multiple_calls() { /* ... */ }
   
   #[test]
   fn test_invalid_tool_call_format() { /* ... */ }
   ```

2. **启发式检测测试**
   ```rust
   #[test]
   fn test_should_create_plan_keywords() { /* ... */ }
   
   #[test]
   fn test_should_use_tools_keywords() { /* ... */ }
   ```

### 集成测试

1. **端到端工具调用测试**
   - 发送包含工具调用请求的消息
   - 验证工具被正确执行
   - 验证结果正确返回

2. **任务规划测试**
   - 发送复杂任务请求
   - 验证计划被创建
   - 验证步骤按顺序执行
   - 验证最终结果正确

## 性能考虑

### 工具调用

- **延迟**: 工具执行可能增加 100-1000ms 延迟
- **优化**: 
  - 并行执行无依赖的工具调用
  - 缓存工具结果
  - 设置合理的超时时间

### 任务规划

- **延迟**: 计划创建需要额外的AI调用，可能增加 1-3秒
- **优化**:
  - 仅在真正需要时创建计划
  - 缓存相似任务的计划
  - 支持计划模板

## 下一步行动

### 高优先级
1. ⏳ 修复 `AgentManager` 访问问题
2. ⏳ 集成工具调用到流式响应
3. ⏳ 增强系统提示词的工具说明

### 中优先级
4. ⏳ 实现任务规划的实际创建和执行
5. ⏳ 添加工具执行的进度通知
6. ⏳ 实现工具调用的重试机制

### 低优先级
7. ⏳ 前端UI组件实现
8. ⏳ 添加工具使用统计
9. ⏳ 实现工具结果缓存

## 相关文档

- `AGENT_CAPABILITIES_IMPLEMENTATION_SUMMARY.md` - 智能体能力实现总结
- `AGENT_INTEGRATION_ISSUE_ANALYSIS.md` - 智能体集成问题分析
- `REASONING_FIX_COMPLETE.md` - Reasoning 显示修复
- `REASONING_FIX_PATCH_404.md` - 404 错误修复

## 代码结构

```
rust-lib/flowy-ai/src/agent/
├── tool_call_handler.rs        ← 工具调用处理器（新）
├── plan_integration.rs          ← 任务规划集成器（新）
├── agent_capabilities.rs        ← 智能体能力执行器
├── config_manager.rs            ← 配置管理器
├── agent_manager.rs             ← 任务管理器
├── planner.rs                   ← 任务规划器
├── executor.rs                  ← 任务执行器
├── tool_registry.rs             ← 工具注册表
├── native_tools.rs              ← 原生工具管理
└── system_prompt.rs             ← 系统提示词构建
```

---

**状态**: 基础框架完成，等待完整集成  
**最后更新**: 2025-10-02  
**版本**: v0.1.0-alpha


