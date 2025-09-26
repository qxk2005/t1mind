import 'dart:async';

import 'package:appflowy_backend/log.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:nanoid/nanoid.dart';

import 'task_planner_entities.dart';
import 'mcp_endpoint_service.dart';

part 'task_planner_bloc.freezed.dart';

/// TaskPlannerBloc - 管理任务规划的完整状态流程
/// 
/// 负责处理：
/// - 任务规划的创建和管理
/// - 状态转换逻辑
/// - 与后端服务的通信
/// - 错误处理和恢复
class TaskPlannerBloc extends Bloc<TaskPlannerEvent, TaskPlannerState> {
  TaskPlannerBloc({
    required this.sessionId,
    required this.userId,
  }) : super(TaskPlannerState.initial()) {
    _dispatch();
  }

  final String sessionId;
  final String userId;

  // 当前活动的任务规划
  TaskPlan? _currentTaskPlan;
  
  // 任务规划历史记录
  final List<TaskPlan> _planHistory = [];

  // 取消令牌，用于取消正在进行的操作
  Completer<void>? _currentOperation;

  @override
  Future<void> close() async {
    // 取消所有正在进行的操作
    _currentOperation?.complete();
    return super.close();
  }

  void _dispatch() {
    on<TaskPlannerEvent>((event, emit) async {
      await event.when(
        // 任务规划相关事件
        createTaskPlan: (userQuery, mcpEndpoints, agentId) async =>
            _handleCreateTaskPlan(userQuery, mcpEndpoints, agentId, emit),
        updateTaskPlan: (taskPlan) async =>
            _handleUpdateTaskPlan(taskPlan, emit),
        confirmTaskPlan: (taskPlanId) async =>
            _handleConfirmTaskPlan(taskPlanId, emit),
        rejectTaskPlan: (taskPlanId, reason) async =>
            _handleRejectTaskPlan(taskPlanId, reason, emit),
        
        // 任务步骤管理
        addTaskStep: (taskPlanId, step) async =>
            _handleAddTaskStep(taskPlanId, step, emit),
        updateTaskStep: (taskPlanId, stepId, step) async =>
            _handleUpdateTaskStep(taskPlanId, stepId, step, emit),
        removeTaskStep: (taskPlanId, stepId) async =>
            _handleRemoveTaskStep(taskPlanId, stepId, emit),
        reorderTaskSteps: (taskPlanId, stepIds) async =>
            _handleReorderTaskSteps(taskPlanId, stepIds, emit),
        
        // 执行控制
        startExecution: (taskPlanId) async =>
            _handleStartExecution(taskPlanId, emit),
        pauseExecution: (taskPlanId) async =>
            _handlePauseExecution(taskPlanId, emit),
        resumeExecution: (taskPlanId) async =>
            _handleResumeExecution(taskPlanId, emit),
        cancelExecution: (taskPlanId) async =>
            _handleCancelExecution(taskPlanId, emit),
        
        // 状态管理
        clearCurrentPlan: () async =>
            _handleClearCurrentPlan(emit),
        loadPlanHistory: () async =>
            _handleLoadPlanHistory(emit),
        deletePlan: (taskPlanId) async =>
            _handleDeletePlan(taskPlanId, emit),
        
        // 错误处理
        retryLastOperation: () async =>
            _handleRetryLastOperation(emit),
        clearError: () async =>
            _handleClearError(emit),
      );
    });
  }

  // 创建任务规划
  Future<void> _handleCreateTaskPlan(
    String userQuery,
    List<String> mcpEndpoints,
    String? agentId,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (state.status.isProcessing) {
      Log.warn('TaskPlannerBloc: Cannot create plan while another operation is in progress');
      return;
    }

    emit(state.copyWith(
      status: TaskPlannerStatus.planning,
      errorMessage: null,
    ),);

    try {
      // 取消之前的操作
      _currentOperation?.complete();
      _currentOperation = Completer<void>();

      final taskPlan = TaskPlan(
        id: nanoid(),
        userQuery: userQuery,
        overallStrategy: '', // 将由AI生成
        requiredMcpEndpoints: mcpEndpoints,
        createdAt: DateTime.now(),
        agentId: agentId,
        sessionId: sessionId,
      );

      // TODO: 调用后端服务生成任务规划
      // 这里应该调用Rust层的任务编排器来生成详细的任务规划
      // final result = await _taskOrchestratorService.createTaskPlan(taskPlan);
      
      // 模拟异步操作
      await Future.delayed(const Duration(milliseconds: 500));
      
      if (_currentOperation?.isCompleted == true) {
        return; // 操作已被取消
      }

      // 生成智能化的任务规划步骤
      final steps = await _generateIntelligentSteps(userQuery, mcpEndpoints);
      final totalEstimatedTime = steps.fold<int>(
        0, 
        (sum, step) => sum + step.estimatedDurationSeconds,
      );
      
      final generatedPlan = taskPlan.copyWith(
        overallStrategy: '基于用户查询"$userQuery"生成的任务规划策略',
        steps: steps,
        estimatedDurationSeconds: totalEstimatedTime,
        status: TaskPlanStatus.pendingConfirmation,
      );

      _currentTaskPlan = generatedPlan;
      _planHistory.add(generatedPlan);

      emit(state.copyWith(
        status: TaskPlannerStatus.waitingConfirmation,
        currentTaskPlan: generatedPlan,
        planHistory: List.from(_planHistory),
        errorMessage: null,
      ),);

      _currentOperation = null;
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to create task plan: $error');
      emit(state.copyWith(
        status: TaskPlannerStatus.planFailed,
        errorMessage: '创建任务规划失败: $error',
      ),);
      _currentOperation = null;
    }
  }

  // 更新任务规划
  Future<void> _handleUpdateTaskPlan(
    TaskPlan taskPlan,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlan.id) {
      Log.warn('TaskPlannerBloc: Cannot update non-current task plan');
      return;
    }

    try {
      final updatedPlan = taskPlan.copyWith(
        updatedAt: DateTime.now(),
      );

      _currentTaskPlan = updatedPlan;
      
      // 更新历史记录
      final historyIndex = _planHistory.indexWhere((p) => p.id == taskPlan.id);
      if (historyIndex != -1) {
        _planHistory[historyIndex] = updatedPlan;
      }

      emit(state.copyWith(
        currentTaskPlan: updatedPlan,
        planHistory: List.from(_planHistory),
      ),);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to update task plan: $error');
      emit(state.copyWith(
        errorMessage: '更新任务规划失败: $error',
      ),);
    }
  }

  // 确认任务规划
  Future<void> _handleConfirmTaskPlan(
    String taskPlanId,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot confirm non-current task plan');
      return;
    }

    if (_currentTaskPlan?.status != TaskPlanStatus.pendingConfirmation) {
      Log.warn('TaskPlannerBloc: Task plan is not in pending confirmation state');
      return;
    }

    try {
      final confirmedPlan = _currentTaskPlan!.copyWith(
        status: TaskPlanStatus.confirmed,
        updatedAt: DateTime.now(),
      );

      _currentTaskPlan = confirmedPlan;
      
      // 更新历史记录
      final historyIndex = _planHistory.indexWhere((p) => p.id == taskPlanId);
      if (historyIndex != -1) {
        _planHistory[historyIndex] = confirmedPlan;
      }

      emit(state.copyWith(
        status: TaskPlannerStatus.planReady,
        currentTaskPlan: confirmedPlan,
        planHistory: List.from(_planHistory),
        errorMessage: null,
      ),);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to confirm task plan: $error');
      emit(state.copyWith(
        errorMessage: '确认任务规划失败: $error',
      ),);
    }
  }

  // 拒绝任务规划
  Future<void> _handleRejectTaskPlan(
    String taskPlanId,
    String? reason,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot reject non-current task plan');
      return;
    }

    try {
      final rejectedPlan = _currentTaskPlan!.copyWith(
        status: TaskPlanStatus.rejected,
        errorMessage: reason,
        updatedAt: DateTime.now(),
      );

      _currentTaskPlan = rejectedPlan;
      
      // 更新历史记录
      final historyIndex = _planHistory.indexWhere((p) => p.id == taskPlanId);
      if (historyIndex != -1) {
        _planHistory[historyIndex] = rejectedPlan;
      }

      emit(state.copyWith(
        status: TaskPlannerStatus.idle,
        currentTaskPlan: rejectedPlan,
        planHistory: List.from(_planHistory),
        errorMessage: null,
      ),);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to reject task plan: $error');
      emit(state.copyWith(
        errorMessage: '拒绝任务规划失败: $error',
      ),);
    }
  }

  // 添加任务步骤
  Future<void> _handleAddTaskStep(
    String taskPlanId,
    TaskStep step,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot add step to non-current task plan');
      return;
    }

    try {
      final updatedSteps = List<TaskStep>.from(_currentTaskPlan!.steps)
        ..add(step.copyWith(order: _currentTaskPlan!.steps.length));

      final updatedPlan = _currentTaskPlan!.copyWith(
        steps: updatedSteps,
        updatedAt: DateTime.now(),
      );

      await _handleUpdateTaskPlan(updatedPlan, emit);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to add task step: $error');
      emit(state.copyWith(
        errorMessage: '添加任务步骤失败: $error',
      ),);
    }
  }

  // 更新任务步骤
  Future<void> _handleUpdateTaskStep(
    String taskPlanId,
    String stepId,
    TaskStep step,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot update step in non-current task plan');
      return;
    }

    try {
      final stepIndex = _currentTaskPlan!.steps.indexWhere((s) => s.id == stepId);
      if (stepIndex == -1) {
        Log.warn('TaskPlannerBloc: Step not found: $stepId');
        return;
      }

      final updatedSteps = List<TaskStep>.from(_currentTaskPlan!.steps);
      updatedSteps[stepIndex] = step;

      final updatedPlan = _currentTaskPlan!.copyWith(
        steps: updatedSteps,
        updatedAt: DateTime.now(),
      );

      await _handleUpdateTaskPlan(updatedPlan, emit);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to update task step: $error');
      emit(state.copyWith(
        errorMessage: '更新任务步骤失败: $error',
      ),);
    }
  }

  // 删除任务步骤
  Future<void> _handleRemoveTaskStep(
    String taskPlanId,
    String stepId,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot remove step from non-current task plan');
      return;
    }

    try {
      final updatedSteps = _currentTaskPlan!.steps
          .where((step) => step.id != stepId)
          .toList();

      // 重新排序
      for (int i = 0; i < updatedSteps.length; i++) {
        updatedSteps[i] = updatedSteps[i].copyWith(order: i);
      }

      final updatedPlan = _currentTaskPlan!.copyWith(
        steps: updatedSteps,
        updatedAt: DateTime.now(),
      );

      await _handleUpdateTaskPlan(updatedPlan, emit);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to remove task step: $error');
      emit(state.copyWith(
        errorMessage: '删除任务步骤失败: $error',
      ),);
    }
  }

  // 重新排序任务步骤
  Future<void> _handleReorderTaskSteps(
    String taskPlanId,
    List<String> stepIds,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot reorder steps in non-current task plan');
      return;
    }

    try {
      final currentSteps = _currentTaskPlan!.steps;
      final reorderedSteps = <TaskStep>[];

      for (int i = 0; i < stepIds.length; i++) {
        final stepId = stepIds[i];
        final step = currentSteps.firstWhere((s) => s.id == stepId);
        reorderedSteps.add(step.copyWith(order: i));
      }

      final updatedPlan = _currentTaskPlan!.copyWith(
        steps: reorderedSteps,
        updatedAt: DateTime.now(),
      );

      await _handleUpdateTaskPlan(updatedPlan, emit);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to reorder task steps: $error');
      emit(state.copyWith(
        errorMessage: '重新排序任务步骤失败: $error',
      ),);
    }
  }

  // 开始执行
  Future<void> _handleStartExecution(
    String taskPlanId,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot start execution of non-current task plan');
      return;
    }

    if (!_currentTaskPlan!.status.canExecute) {
      Log.warn('TaskPlannerBloc: Task plan cannot be executed in current state: ${_currentTaskPlan!.status}');
      return;
    }

    try {
      final executingPlan = _currentTaskPlan!.copyWith(
        status: TaskPlanStatus.executing,
        updatedAt: DateTime.now(),
      );

      await _handleUpdateTaskPlan(executingPlan, emit);

      // TODO: 启动实际的执行流程
      // await _taskExecutorService.startExecution(taskPlanId);
      
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to start execution: $error');
      emit(state.copyWith(
        errorMessage: '开始执行失败: $error',
      ),);
    }
  }

  // 暂停执行
  Future<void> _handlePauseExecution(
    String taskPlanId,
    Emitter<TaskPlannerState> emit,
  ) async {
    // TODO: 实现暂停逻辑
    Log.info('TaskPlannerBloc: Pause execution not yet implemented');
  }

  // 恢复执行
  Future<void> _handleResumeExecution(
    String taskPlanId,
    Emitter<TaskPlannerState> emit,
  ) async {
    // TODO: 实现恢复逻辑
    Log.info('TaskPlannerBloc: Resume execution not yet implemented');
  }

  // 取消执行
  Future<void> _handleCancelExecution(
    String taskPlanId,
    Emitter<TaskPlannerState> emit,
  ) async {
    if (_currentTaskPlan?.id != taskPlanId) {
      Log.warn('TaskPlannerBloc: Cannot cancel execution of non-current task plan');
      return;
    }

    try {
      // 取消当前操作
      _currentOperation?.complete();

      final cancelledPlan = _currentTaskPlan!.copyWith(
        status: TaskPlanStatus.cancelled,
        updatedAt: DateTime.now(),
      );

      await _handleUpdateTaskPlan(cancelledPlan, emit);

      // TODO: 通知后端取消执行
      // await _taskExecutorService.cancelExecution(taskPlanId);
      
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to cancel execution: $error');
      emit(state.copyWith(
        errorMessage: '取消执行失败: $error',
      ),);
    }
  }

  // 清除当前规划
  Future<void> _handleClearCurrentPlan(
    Emitter<TaskPlannerState> emit,
  ) async {
    _currentTaskPlan = null;
    emit(state.copyWith(
      status: TaskPlannerStatus.idle,
      currentTaskPlan: null,
      errorMessage: null,
    ),);
  }

  // 加载规划历史
  Future<void> _handleLoadPlanHistory(
    Emitter<TaskPlannerState> emit,
  ) async {
    try {
      // TODO: 从数据库加载历史记录
      // final history = await _taskPlanRepository.getHistory(sessionId);
      // _planHistory.clear();
      // _planHistory.addAll(history);
      
      emit(state.copyWith(
        planHistory: List.from(_planHistory),
      ),);
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to load plan history: $error');
      emit(state.copyWith(
        errorMessage: '加载规划历史失败: $error',
      ),);
    }
  }

  // 删除规划
  Future<void> _handleDeletePlan(
    String taskPlanId,
    Emitter<TaskPlannerState> emit,
  ) async {
    try {
      _planHistory.removeWhere((plan) => plan.id == taskPlanId);
      
      if (_currentTaskPlan?.id == taskPlanId) {
        _currentTaskPlan = null;
      }

      emit(state.copyWith(
        currentTaskPlan: _currentTaskPlan,
        planHistory: List.from(_planHistory),
      ),);

      // TODO: 从数据库删除
      // await _taskPlanRepository.delete(taskPlanId);
      
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to delete plan: $error');
      emit(state.copyWith(
        errorMessage: '删除规划失败: $error',
      ),);
    }
  }

  // 重试上次操作
  Future<void> _handleRetryLastOperation(
    Emitter<TaskPlannerState> emit,
  ) async {
    // TODO: 实现重试逻辑
    Log.info('TaskPlannerBloc: Retry last operation not yet implemented');
    emit(state.copyWith(
      errorMessage: null,
    ),);
  }

  // 清除错误
  Future<void> _handleClearError(
    Emitter<TaskPlannerState> emit,
  ) async {
    emit(state.copyWith(
      errorMessage: null,
    ),);
  }

  // 生成智能化任务步骤，基于MCP端点工具进行分析和选择
  Future<List<TaskStep>> _generateIntelligentSteps(
    String userQuery,
    List<String> mcpEndpoints,
  ) async {
    final steps = <TaskStep>[];
    
    if (mcpEndpoints.isNotEmpty) {
      // 获取所有端点的工具信息
      final mcpEndpointService = McpEndpointService();
      final availableToolsMap = <String, List<McpToolSchema>>{};
      
      for (final endpointId in mcpEndpoints) {
        try {
          final tools = await mcpEndpointService.getEndpointTools(endpointId);
          availableToolsMap[endpointId] = tools.map((tool) => McpToolSchema(
            name: tool.name,
            description: tool.description ?? '',
            inputSchema: {}, // 从tool.fields转换而来
            outputSchema: {}, // 目前为空，可以后续扩展
            type: 'function',
            required: tool.fields.where((f) => f.required).map((f) => f.name).toList(),
            examples: [],
            tags: [],
            version: '1.0.0',
          )).toList();
        } catch (e) {
          Log.warn('Failed to get tools for endpoint $endpointId: $e');
          availableToolsMap[endpointId] = [];
        }
      }
      
      // 步骤1：分析用户需求和工具匹配
      steps.add(TaskStep(
        id: nanoid(),
        description: '分析用户查询意图，理解任务需求',
        objective: '理解用户的具体需求，确定需要完成的任务类型和目标',
        mcpToolId: 'ai-assistant',
        availableTools: [], // AI助手不需要外部工具
        parameters: {
          'step': 1, 
          'action': 'analyze_intent',
          'user_query': userQuery,
          'available_endpoints': mcpEndpoints,
          'available_tools': availableToolsMap,
        },
        order: 0,
        estimatedDurationSeconds: 15,
      ));
      
      // 步骤2-N：为每个端点创建智能化的任务步骤
      int stepOrder = 1;
      for (final endpointId in mcpEndpoints) {
        final endpointTools = availableToolsMap[endpointId] ?? [];
        
        if (endpointTools.isNotEmpty) {
          // 根据用户查询和可用工具，AI智能选择最合适的工具
          final selectedTool = _selectBestToolForQuery(userQuery, endpointTools);
          final stepObjective = _generateStepObjective(userQuery, endpointId, endpointTools);
          
          steps.add(TaskStep(
            id: nanoid(),
            description: '使用 ${selectedTool.name} 工具完成特定任务',
            objective: stepObjective,
            mcpEndpointId: endpointId,
            mcpToolId: selectedTool.name, // AI预选的具体工具
            availableTools: endpointTools,
            parameters: {
              'step': stepOrder + 1,
              'endpoint': endpointId,
              'selected_tool': selectedTool.name,
              'user_query': userQuery,
              'step_objective': stepObjective,
            },
            order: stepOrder,
            estimatedDurationSeconds: 45,
            allowToolSubstitution: true,
          ));
          stepOrder++;
        }
      }
      
      // 最后步骤：整合和总结结果
      steps.add(TaskStep(
        id: nanoid(),
        description: '整合所有步骤的执行结果，生成最终回答',
        objective: '将各个步骤的执行结果进行整合，形成完整、准确的最终回答',
        mcpToolId: 'ai-assistant',
        availableTools: [],
        parameters: {
          'step': 'final', 
          'action': 'integrate_results',
          'user_query': userQuery,
        },
        order: stepOrder,
        estimatedDurationSeconds: 20,
      ));
    } else {
      // 如果没有选择MCP端点，生成默认的AI助手步骤
      final defaultSteps = [
        ('分析用户查询内容', '理解用户的问题和需求'),
        ('基于内置知识生成回答', '使用AI的内置知识库来回答用户问题'),
        ('验证和优化结果', '检查回答的准确性和完整性，进行必要的优化'),
      ];
      
      for (int i = 0; i < defaultSteps.length; i++) {
        final (description, objective) = defaultSteps[i];
        steps.add(TaskStep(
          id: nanoid(),
          description: description,
          objective: objective,
          mcpToolId: 'ai-assistant',
          availableTools: [],
          parameters: {
            'step': i + 1, 
            'description': description,
            'user_query': userQuery,
          },
          order: i,
          estimatedDurationSeconds: 15 + (i * 5),
        ));
      }
    }

    return steps;
  }

  // 根据用户查询智能选择最合适的MCP工具
  McpToolSchema _selectBestToolForQuery(String userQuery, List<McpToolSchema> tools) {
    if (tools.isEmpty) {
      throw Exception('No tools available for selection');
    }
    
    if (tools.length == 1) {
      return tools.first;
    }
    
    // 简化的工具选择逻辑 - 基于关键词匹配
    final queryLower = userQuery.toLowerCase();
    
    // 优先级匹配规则
    final priorityRules = <String, List<String>>{
      'excel': [],
      'file': [],
      'search': [],
      'data': [],
      'analysis': [],
    };
    
    // 计算每个工具的匹配分数
    var bestTool = tools.first;
    var bestScore = 0;
    
    for (final tool in tools) {
      var score = 0;
      final toolNameLower = tool.name.toLowerCase();
      final toolDescLower = tool.description.toLowerCase();
      
      // 检查工具名称和描述中的关键词匹配
      for (final entry in priorityRules.entries) {
        final keywords = entry.value;
        for (final keyword in keywords) {
          if (queryLower.contains(keyword)) {
            if (toolNameLower.contains(entry.key) || toolDescLower.contains(entry.key)) {
              score += 10; // 高优先级匹配
            }
            if (toolNameLower.contains(keyword) || toolDescLower.contains(keyword)) {
              score += 5; // 直接关键词匹配
            }
          }
        }
      }
      
      // 如果用户查询中包含工具名称，给予额外分数
      if (queryLower.contains(toolNameLower) || toolNameLower.contains(queryLower.split(' ').first)) {
        score += 15;
      }
      
      if (score > bestScore) {
        bestScore = score;
        bestTool = tool;
      }
    }
    
    // 如果没有明显的匹配，选择第一个工具
    return bestTool;
  }

  // 根据用户查询和可用工具生成步骤目标
  String _generateStepObjective(String userQuery, String endpointId, List<McpToolSchema> tools) {
    // 这里可以使用更智能的逻辑来分析用户查询和工具能力
    // 目前使用简化的逻辑
    
    if (tools.isEmpty) {
      return '使用 $endpointId 端点完成相关任务';
    }
    
    // 分析工具类型和描述，生成合适的目标
    final toolDescriptions = tools.map((t) => t.description).where((d) => d.isNotEmpty).toList();
    
    if (toolDescriptions.isNotEmpty) {
      final combinedDescription = toolDescriptions.take(3).join('、');
      return '使用 $endpointId 端点的工具（如：$combinedDescription）来处理用户请求的相关部分';
    }
    
    return '使用 $endpointId 端点的可用工具来完成任务的特定部分';
  }
}

/// TaskPlannerEvent - 任务规划器事件定义
@freezed
class TaskPlannerEvent with _$TaskPlannerEvent {
  // 任务规划管理
  const factory TaskPlannerEvent.createTaskPlan({
    required String userQuery,
    required List<String> mcpEndpoints,
    String? agentId,
  }) = _CreateTaskPlan;

  const factory TaskPlannerEvent.updateTaskPlan({
    required TaskPlan taskPlan,
  }) = _UpdateTaskPlan;

  const factory TaskPlannerEvent.confirmTaskPlan({
    required String taskPlanId,
  }) = _ConfirmTaskPlan;

  const factory TaskPlannerEvent.rejectTaskPlan({
    required String taskPlanId,
    String? reason,
  }) = _RejectTaskPlan;

  // 任务步骤管理
  const factory TaskPlannerEvent.addTaskStep({
    required String taskPlanId,
    required TaskStep step,
  }) = _AddTaskStep;

  const factory TaskPlannerEvent.updateTaskStep({
    required String taskPlanId,
    required String stepId,
    required TaskStep step,
  }) = _UpdateTaskStep;

  const factory TaskPlannerEvent.removeTaskStep({
    required String taskPlanId,
    required String stepId,
  }) = _RemoveTaskStep;

  const factory TaskPlannerEvent.reorderTaskSteps({
    required String taskPlanId,
    required List<String> stepIds,
  }) = _ReorderTaskSteps;

  // 执行控制
  const factory TaskPlannerEvent.startExecution({
    required String taskPlanId,
  }) = _StartExecution;

  const factory TaskPlannerEvent.pauseExecution({
    required String taskPlanId,
  }) = _PauseExecution;

  const factory TaskPlannerEvent.resumeExecution({
    required String taskPlanId,
  }) = _ResumeExecution;

  const factory TaskPlannerEvent.cancelExecution({
    required String taskPlanId,
  }) = _CancelExecution;

  // 状态管理
  const factory TaskPlannerEvent.clearCurrentPlan() = _ClearCurrentPlan;

  const factory TaskPlannerEvent.loadPlanHistory() = _LoadPlanHistory;

  const factory TaskPlannerEvent.deletePlan({
    required String taskPlanId,
  }) = _DeletePlan;

  // 错误处理
  const factory TaskPlannerEvent.retryLastOperation() = _RetryLastOperation;

  const factory TaskPlannerEvent.clearError() = _ClearError;
}

/// TaskPlannerState - 任务规划器状态定义
@freezed
class TaskPlannerState with _$TaskPlannerState {
  const factory TaskPlannerState({
    /// 当前状态
    required TaskPlannerStatus status,
    /// 当前任务规划
    TaskPlan? currentTaskPlan,
    /// 规划历史记录
    @Default([]) List<TaskPlan> planHistory,
    /// 错误信息
    String? errorMessage,
    /// 最后更新时间
    DateTime? lastUpdated,
  }) = _TaskPlannerState;

  /// 初始状态
  factory TaskPlannerState.initial() => TaskPlannerState(
        status: TaskPlannerStatus.idle,
        lastUpdated: DateTime.now(),
      );
}

/// TaskPlannerState 扩展方法
extension TaskPlannerStateExtension on TaskPlannerState {
  /// 是否有当前任务规划
  bool get hasCurrentPlan => currentTaskPlan != null;

  /// 是否可以创建新规划
  bool get canCreateNewPlan => status == TaskPlannerStatus.idle;

  /// 是否正在处理
  bool get isProcessing => status.isProcessing;

  /// 是否需要用户确认
  bool get needsConfirmation => status.needsUserAction;

  /// 是否有错误
  bool get hasError => errorMessage != null;

  /// 当前规划是否可以执行
  bool get canExecute => currentTaskPlan?.status.canExecute == true;

  /// 当前规划是否正在执行
  bool get isExecuting => currentTaskPlan?.status.isExecuting == true;

  /// 获取当前规划的步骤数
  int get currentPlanStepCount => currentTaskPlan?.steps.length ?? 0;

  /// 获取历史规划数量
  int get historyCount => planHistory.length;
}
