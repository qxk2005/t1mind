# 任务规划逻辑修复总结

## 问题描述

用户反馈当前AI聊天中的任务规划功能存在逻辑错误：

1. **问题1**：在任务规划中列出所有MCP工具进行选择，这样不够智能
2. **问题2**：AI应该根据用户问题意图自动选择合适的工具，而不是让用户手动选择所有工具

## 期望的改进方案

1. **端点选择**：用户只需选择MCP端点，而不是具体工具
2. **智能工具选择**：AI根据用户问题意图，从选中的端点中自动选择合适的工具
3. **按需分配**：工具按需分配到子任务中，而不是盲目使用所有选中的工具

## 实施的解决方案

### 1. 创建MCP端点服务 (`McpEndpointService`)

**文件**: `appflowy_flutter/lib/plugins/ai_chat/application/mcp_endpoint_service.dart`

```dart
/// MCP端点信息
class McpEndpointInfo {
  const McpEndpointInfo({
    required this.id,
    required this.name,
    required this.description,
    required this.isAvailable,
    required this.toolCount,
    this.lastChecked,
  });
  // ... 其他属性和方法
}

/// MCP端点服务
class McpEndpointService {
  /// 获取所有可用的MCP端点
  Future<List<McpEndpointInfo>> getAvailableEndpoints() async {
    // 从设置中读取端点配置
    // 只返回检查通过的端点
  }
  
  /// 获取端点的详细工具信息
  Future<List<McpToolSchema>> getEndpointTools(String endpointId) async {
    // 获取特定端点下的所有工具
  }
}
```

**核心功能**：
- 从MCP设置中读取端点配置
- 过滤出可用的端点
- 提供端点的工具统计信息
- 支持获取端点下的具体工具列表

### 2. 创建MCP端点选择器 (`McpEndpointSelector`)

**文件**: `appflowy_flutter/lib/plugins/ai_chat/presentation/mcp_endpoint_selector.dart`

```dart
/// MCP端点选择器
class McpEndpointSelector extends StatefulWidget {
  const McpEndpointSelector({
    required this.availableEndpoints,
    required this.selectedEndpointIds,
    required this.onSelectionChanged,
  });
  // ... 实现
}
```

**UI特性**：
- 显示端点名称、描述和工具数量
- 显示端点状态（可用/不可用）
- 显示最后检查时间
- 支持全选/清空操作
- 提供友好的用户提示："AI将从选中的端点中自动选择合适的工具"

### 3. 更新聊天界面逻辑

**文件**: `appflowy_flutter/lib/plugins/ai_chat/presentation/chat_page/chat_footer.dart`

**主要变更**：

```dart
// 原来：工具选择
final ValueNotifier<List<String>> _selectedToolIds = ValueNotifier([]);
final ValueNotifier<List<log_entities.McpToolInfo>> _availableTools = ValueNotifier([]);
final McpToolsService _mcpToolsService = McpToolsService();

// 现在：端点选择
final ValueNotifier<List<String>> _selectedEndpointIds = ValueNotifier([]);
final ValueNotifier<List<McpEndpointInfo>> _availableEndpoints = ValueNotifier([]);
final McpEndpointService _mcpEndpointService = McpEndpointService();
```

**高级功能对话框更新**：
- 标题改为"MCP端点选择"
- 说明文字："AI将从选中的端点中自动选择合适的工具来完成任务"
- 使用`McpEndpointSelector`替代`SimpleMcpToolSelector`
- 状态指示器显示"X个端点"而不是"X个工具"

### 4. 修改任务规划器 (`TaskPlannerBloc`)

**文件**: `appflowy_flutter/lib/plugins/ai_chat/application/task_planner_bloc.dart`

**事件定义更新**：
```dart
// 原来
const factory TaskPlannerEvent.createTaskPlan({
  required String userQuery,
  required List<String> mcpTools,  // 具体工具列表
  String? agentId,
}) = _CreateTaskPlan;

// 现在
const factory TaskPlannerEvent.createTaskPlan({
  required String userQuery,
  required List<String> mcpEndpoints,  // 端点列表
  String? agentId,
}) = _CreateTaskPlan;
```

**智能步骤生成逻辑**：
```dart
List<TaskStep> _generateMockSteps(List<String> mcpEndpoints) {
  final steps = <TaskStep>[];
  
  if (mcpEndpoints.isNotEmpty) {
    // 步骤1：分析用户需求
    steps.add(TaskStep(
      description: '分析用户查询意图，确定所需的工具和操作',
      mcpToolId: 'ai-assistant',
    ));
    
    // 步骤2-N：根据端点智能选择工具
    for (final endpointId in mcpEndpoints) {
      steps.add(TaskStep(
        description: '从 $endpointId 端点中选择合适的工具执行任务',
        mcpEndpointId: endpointId,  // 使用端点ID
        mcpToolId: null,  // 工具将由AI动态选择
        parameters: {
          'endpoint': endpointId,
          'auto_select_tool': true,  // 标记为自动选择
        },
      ));
    }
    
    // 最后步骤：整合结果
    steps.add(TaskStep(
      description: '整合所有步骤的结果，生成最终回答',
      mcpToolId: 'ai-assistant',
    ));
  }
  
  return steps;
}
```

### 5. 更新数据模型

**文件**: `appflowy_flutter/lib/plugins/ai_chat/application/task_planner_entities.dart`

**TaskPlan模型更新**：
```dart
class TaskPlan {
  // 原来
  @Default([]) List<String> requiredMcpTools,
  
  // 现在
  @Default([]) List<String> requiredMcpEndpoints,
}
```

**TaskStep模型更新**：
```dart
class TaskStep {
  // 原来
  required String mcpToolId,
  
  // 现在
  String? mcpToolId,  // 可选，如果为null则由AI自动选择
  String? mcpEndpointId,  // 新增端点ID字段
}
```

## 技术实现亮点

### 1. 智能化任务规划

**原来的逻辑**：
- 用户选择具体工具 → 为每个工具生成一个步骤 → 盲目执行所有工具

**现在的逻辑**：
- 用户选择端点 → AI分析问题意图 → 从端点中智能选择工具 → 按需执行

### 2. 灵活的工具选择机制

```dart
// 支持两种模式
TaskStep(
  mcpToolId: 'specific-tool',  // 明确指定工具
  mcpEndpointId: null,
)

TaskStep(
  mcpToolId: null,  // AI自动选择
  mcpEndpointId: 'endpoint-id',
  parameters: {'auto_select_tool': true},
)
```

### 3. 向后兼容性

- 保留了原有的MCP选择器（`McpSelector`）用于向后兼容
- 元数据中同时支持`mcpNames`和`selectedEndpointIds`
- 数据模型支持新旧两种方式

### 4. 用户体验改进

**更清晰的界面提示**：
- "AI将从选中的端点中自动选择合适的工具来完成任务"
- "已选择 X 个端点"而不是"已选择 X 个工具"
- 显示端点的工具数量和状态信息

**更智能的任务规划**：
- 任务步骤描述更加清晰和有意义
- 明确区分AI分析、工具执行和结果整合阶段
- 支持动态工具选择的参数配置

## 使用流程对比

### 原来的流程
1. 用户启用任务规划
2. 用户从所有MCP工具中选择要使用的工具
3. AI为每个选中的工具创建一个执行步骤
4. 按顺序执行所有工具，不管是否真正需要

### 现在的流程
1. 用户启用任务规划
2. 用户选择相关的MCP端点（而不是具体工具）
3. AI分析用户问题的意图和需求
4. AI从选中的端点中智能选择最合适的工具
5. 按需执行工具，只使用真正需要的功能

## 示例场景

**用户问题**：统计myfile.xlsx文件中美国总统的数量

### 原来的方式
1. 用户选择：excel_read, excel_write, excel_format等所有Excel相关工具
2. 任务规划：
   - 步骤1：使用excel_read工具
   - 步骤2：使用excel_write工具  
   - 步骤3：使用excel_format工具
   - （所有工具都会被执行，不管是否需要）

### 现在的方式
1. 用户选择：Excel端点
2. 任务规划：
   - 步骤1：分析用户查询意图，确定需要读取Excel文件并统计数据
   - 步骤2：从Excel端点中选择合适的工具（AI自动选择excel_read）
   - 步骤3：整合结果，生成最终回答
   - （只执行真正需要的工具）

## 技术优势

### 1. 更高的执行效率
- 避免执行不必要的工具
- 减少资源浪费和执行时间
- 提高任务完成的准确性

### 2. 更好的用户体验
- 用户不需要了解具体工具的功能
- 选择过程更简单直观
- AI承担了工具选择的复杂性

### 3. 更强的可扩展性
- 新增工具时用户界面不需要变更
- 支持端点级别的管理和配置
- 便于实现更复杂的工具编排逻辑

### 4. 更智能的决策
- AI可以根据上下文动态选择工具
- 支持工具之间的依赖关系处理
- 可以实现更复杂的执行策略

## 后续改进计划

### 短期改进
1. **实际AI集成**：将模拟的工具选择逻辑替换为真正的AI决策
2. **执行监控**：添加工具执行过程的监控和日志
3. **错误处理**：完善工具选择和执行失败时的处理逻辑

### 长期改进
1. **学习机制**：AI根据历史执行结果学习更好的工具选择策略
2. **依赖分析**：自动分析工具之间的依赖关系
3. **并行执行**：支持独立工具的并行执行
4. **成本优化**：根据工具执行成本进行智能选择

## 总结

通过这次重构，我们成功地将任务规划从"工具驱动"转变为"意图驱动"的模式：

- **用户视角**：从选择具体工具变为选择功能领域（端点）
- **AI视角**：从被动执行变为主动分析和智能选择
- **系统视角**：从固定流程变为灵活的动态编排

这种改进不仅解决了当前的逻辑问题，还为未来更高级的AI任务编排功能奠定了基础。用户现在可以更自然地表达需求，而AI可以更智能地理解和执行任务。
