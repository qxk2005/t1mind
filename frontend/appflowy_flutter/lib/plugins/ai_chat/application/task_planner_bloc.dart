import 'dart:async';
import 'dart:convert';

import 'package:appflowy_backend/log.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:nanoid/nanoid.dart';
import 'package:appflowy/ai/service/appflowy_ai_service.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';

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
  
  // AI服务
  final AppFlowyAIService _aiService = AppFlowyAIService();

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
        
        // AI工具选择
        updateWithAIToolSelection: (taskPlanId, aiSelectionResult) async =>
            _updateStepsWithAISelection(taskPlanId, aiSelectionResult, emit),
        
        // 更新AI思考过程
        updateAIThinkingProcess: (thinkingText) async =>
            emit(state.copyWith(aiThinkingProcess: thinkingText)),
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
            inputSchema: {
              'type': 'object',
              'properties': {
                for (final field in tool.fields)
                  field.name: {
                    'type': field.type,
                    'description': field.name, // 可以添加更详细的描述
                    if (field.defaultValue != null) 'default': field.defaultValue,
                  }
              },
              'required': tool.fields.where((f) => f.required).map((f) => f.name).toList(),
            },
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
      
      // 步骤1：分析用户需求
      steps.add(TaskStep(
        id: nanoid(),
        description: '分析用户查询意图，理解任务需求',
        objective: '深入理解用户查询"$userQuery"的具体需求和目标',
        mcpToolId: 'ai-assistant',
        availableTools: [],
        parameters: {
          'step': 1, 
          'action': 'analyze_intent',
          'user_query': userQuery,
        },
        order: 0,
        estimatedDurationSeconds: 10,
      ));
      
      // 步骤2-N：为每个端点智能选择最合适的工具
      int stepOrder = 1;
      for (final endpointId in mcpEndpoints) {
        final endpointTools = availableToolsMap[endpointId] ?? [];
        
        if (endpointTools.isNotEmpty) {
          // 让AI分析并选择最合适的工具
          final toolSelection = await _selectBestToolWithAI(
            userQuery: userQuery,
            endpointId: endpointId,
            availableTools: endpointTools,
          );
          
          steps.add(TaskStep(
            id: nanoid(),
            description: '使用 $endpointId 端点的 ${toolSelection.toolName} 工具执行任务',
            objective: '${toolSelection.objective}',
            mcpEndpointId: endpointId,
            mcpToolId: toolSelection.toolName,
            availableTools: endpointTools,
            parameters: {
              'step': stepOrder + 1,
              'endpoint': endpointId,
              'selected_tool': toolSelection.toolName,
              'selection_reason': toolSelection.reason,
              'user_query': userQuery,
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

  // 使用AI智能选择最合适的工具
  Future<({String toolName, String reason, String objective})> _selectBestToolWithAI({
    required String userQuery,
    required String endpointId,
    required List<McpToolSchema> availableTools,
  }) async {
    Log.info('开始为端点 $endpointId 选择工具，可用工具数量: ${availableTools.length}');
    
    // 准备工具信息供AI分析
    final toolsInfo = availableTools.map((tool) => {
      'name': tool.name,
      'description': tool.description,
      'inputSchema': tool.inputSchema,
      'type': tool.type,
      'required': tool.required,
    }).toList();
    
    // 打印所有可用工具的信息
    for (final tool in availableTools) {
      Log.info('可用工具: ${tool.name} - ${tool.description}');
    }
    
    // 构建AI提示词
    final prompt = _buildToolSelectionPrompt(
      userQuery: userQuery,
      endpointId: endpointId,
      availableTools: toolsInfo,
    );
    
    Log.info('生成的AI提示词长度: ${prompt.length} 字符');
    Log.debug('AI提示词内容:\n$prompt');
    
    // 调用AI服务
    final aiResult = await _callAIForToolSelection(prompt);
    
    if (aiResult != null) {
      Log.info('AI返回结果长度: ${aiResult.length} 字符');
      Log.debug('AI返回内容:\n$aiResult');
      
      // 解析AI返回的结果
      try {
        final selection = _parseAIToolSelection(aiResult, availableTools);
        if (selection != null) {
          Log.info('AI选择了工具: ${selection.toolName}');
          Log.info('选择理由: ${selection.reason}');
          return selection;
        }
      } catch (e) {
        Log.warn('Failed to parse AI tool selection result: $e');
      }
    } else {
      Log.warn('AI服务返回空结果');
    }
    
    // 如果AI调用失败，使用后备逻辑
    return _fallbackToolSelection(userQuery, endpointId, availableTools);
  }
  
  // 构建工具选择的AI提示词
  String _buildToolSelectionPrompt({
    required String userQuery,
    required String endpointId,
    required List<Map<String, dynamic>> availableTools,
  }) {
    final toolsJson = const JsonEncoder.withIndent('  ').convert(availableTools);
    
    return '''
你是一个智能任务规划助手，负责为用户的查询选择最合适的MCP工具。

用户查询：$userQuery
MCP端点：$endpointId

可用的工具列表：
$toolsJson

请分析用户的查询意图，并从可用工具中选择最合适的一个。返回JSON格式的结果：

{
  "toolName": "选中的工具名称",
  "reason": "简洁理由（20字内）",
  "objective": "使用这个工具要达成的具体目标"
}

注意事项：
1. 仔细分析用户查询的真实意图
2. 考虑工具的功能描述和参数要求
3. 选择最能准确完成用户需求的工具
4. 如果有多个工具都可以使用，选择最直接、最高效的那个
5. 理由必须简洁，控制在20个字以内，只说明核心原因
6. 目标要明确且可执行

请只返回JSON格式的结果，不要包含其他说明文字。
''';
  }
  
  // 调用AI服务进行工具选择
  Future<String?> _callAIForToolSelection(String prompt) async {
    final completer = Completer<String?>();
    final buffer = StringBuffer();
    
    try {
      final result = await _aiService.streamCompletion(
        text: prompt,
        completionType: CompletionTypePB.UserQuestion,
        onStart: () async {
          Log.info('AI tool selection started');
          // 发送AI思考开始的更新
          add(const TaskPlannerEvent.updateAIThinkingProcess(
            thinkingText: '正在分析您的需求...',
          ));
        },
        processMessage: (text) async {
          buffer.write(text);
          // 发送流式更新
          add(TaskPlannerEvent.updateAIThinkingProcess(
            thinkingText: buffer.toString(),
          ));
        },
        processAssistMessage: (text) async {
          buffer.write(text);
          // 发送流式更新
          add(TaskPlannerEvent.updateAIThinkingProcess(
            thinkingText: buffer.toString(),
          ));
        },
        onEnd: () async {
          final result = buffer.toString().trim();
          Log.info('AI tool selection completed: ${result.length} chars');
          completer.complete(result.isNotEmpty ? result : null);
        },
        onError: (error) {
          Log.error('AI tool selection error: $error');
          add(TaskPlannerEvent.updateAIThinkingProcess(
            thinkingText: '工具选择出错: $error',
          ));
          completer.complete(null);
        },
        onLocalAIStreamingStateChange: (state) {
          Log.info('Local AI state change: $state');
        },
      );

      if (result == null) {
        Log.warn('AI streamCompletion returned null');
        return null;
      }

      // 设置超时时间
      return await completer.future.timeout(
        const Duration(seconds: 30),
        onTimeout: () {
          Log.warn('AI tool selection timeout');
          add(const TaskPlannerEvent.updateAIThinkingProcess(
            thinkingText: '工具选择超时，使用默认选择...',
          ));
          return null;
        },
      );
    } catch (e) {
      Log.error('Failed to call AI for tool selection: $e');
      return null;
    }
  }
  
  // 解析AI返回的工具选择结果
  ({String toolName, String reason, String objective})? _parseAIToolSelection(
    String aiResult,
    List<McpToolSchema> availableTools,
  ) {
    try {
      // 提取JSON部分（AI可能会返回额外的文本）
      final jsonMatch = RegExp(r'\{[^}]+\}', multiLine: true, dotAll: true).firstMatch(aiResult);
      if (jsonMatch == null) {
        Log.warn('No JSON found in AI result');
        return null;
      }
      
      final jsonStr = jsonMatch.group(0)!;
      final json = jsonDecode(jsonStr) as Map<String, dynamic>;
      
      final toolName = json['toolName'] as String?;
      final reason = json['reason'] as String?;
      final objective = json['objective'] as String?;
      
      if (toolName == null || reason == null || objective == null) {
        Log.warn('Incomplete AI tool selection result');
        return null;
      }
      
      // 验证工具名称是否有效
      final validTool = availableTools.any((tool) => tool.name == toolName);
      if (!validTool) {
        Log.warn('AI selected invalid tool: $toolName');
        return null;
      }
      
      return (
        toolName: toolName,
        reason: reason,
        objective: objective,
      );
    } catch (e) {
      Log.error('Failed to parse AI tool selection: $e');
      return null;
    }
  }
  
  // 后备工具选择逻辑
  ({String toolName, String reason, String objective}) _fallbackToolSelection(
    String userQuery,
    String endpointId,
    List<McpToolSchema> availableTools,
  ) {
    Log.warn('使用后备工具选择逻辑');
    
    final queryLower = userQuery.toLowerCase();
    McpToolSchema? selectedTool;
    String reason = '';
    int bestScore = 0;
    
    // 为每个工具计算相关性得分
    for (final tool in availableTools) {
      final descLower = tool.description.toLowerCase();
      final nameLower = tool.name.toLowerCase();
      int score = 0;
      
      // 计算关键词匹配得分
      if (queryLower.contains('读取') || queryLower.contains('read') || queryLower.contains('获取') || queryLower.contains('get')) {
        if (nameLower.contains('read') || nameLower.contains('get')) {
          score += 5;
          reason = '适合读取操作';
        }
        if (descLower.contains('读取') || descLower.contains('获取')) score += 3;
      }
      
      if (queryLower.contains('写入') || queryLower.contains('write') || queryLower.contains('保存') || queryLower.contains('save')) {
        if (nameLower.contains('write') || nameLower.contains('save')) {
          score += 5;
          reason = '适合写入操作';
        }
        if (descLower.contains('写入') || descLower.contains('保存')) score += 3;
      }
      
      if (queryLower.contains('查询') || queryLower.contains('query') || queryLower.contains('搜索') || queryLower.contains('search')) {
        if (nameLower.contains('query') || nameLower.contains('search')) {
          score += 5;
          reason = '适合查询操作';
        }
        if (descLower.contains('查询') || descLower.contains('搜索')) score += 3;
      }
      
      if (queryLower.contains('列表') || queryLower.contains('list') || queryLower.contains('显示') || queryLower.contains('show')) {
        if (nameLower.contains('list') || nameLower.contains('show')) {
          score += 5;
          reason = '适合列表操作';
        }
        if (descLower.contains('列表') || descLower.contains('显示')) score += 3;
      }
      
      // 检查查询中的其他关键词
      final queryWords = queryLower.split(RegExp(r'[\s,.!?;:，。！？；：]+'))
          .where((w) => w.length > 1)
          .toList();
      
      for (final word in queryWords) {
        if (nameLower.contains(word)) score += 2;
        if (descLower.contains(word)) score += 1;
      }
      
      Log.debug('工具 ${tool.name} 的相关性得分: $score');
      
      if (score > bestScore) {
        bestScore = score;
        selectedTool = tool;
        // reason已经在上面的匹配中设置了，如果没有设置则使用默认值
        if (reason.isEmpty) {
          reason = '关键词匹配度最高';
        }
      }
    }
    
    // 如果没有匹配，选择描述最相关的工具
    if (selectedTool == null && availableTools.isNotEmpty) {
      selectedTool = availableTools.first;
      reason = '默认选择';
    }
    
    final objective = '使用${selectedTool!.name}处理用户查询';
    
    Log.info('后备逻辑选择了工具: ${selectedTool.name}，理由: $reason');
    
    return (
      toolName: selectedTool.name,
      reason: reason,
      objective: objective,
    );
  }
  
  // 处理AI模型的工具选择结果，更新任务步骤
  Future<void> _updateStepsWithAISelection(
    String taskPlanId,
    Map<String, dynamic> aiSelectionResult,
    Emitter<TaskPlannerState> emit,
  ) async {
    // 解析AI返回的工具选择结果
    // 期望格式：
    // {
    //   "tool_selections": [
    //     {
    //       "endpoint_id": "excel",
    //       "tool_name": "excel_reader",
    //       "reason": "用户需要读取Excel文件内容，excel_reader工具专门用于读取Excel文件",
    //       "order": 1
    //     }
    //   ]
    // }
    
    try {
      final toolSelections = aiSelectionResult['tool_selections'] as List<dynamic>?;
      if (toolSelections == null || toolSelections.isEmpty) {
        Log.warn('AI did not return any tool selections');
        return;
      }
      
      // 获取当前任务计划
      if (_currentTaskPlan == null) return;
      
      // 更新相应的任务步骤
      final updatedSteps = _currentTaskPlan!.steps.map((step) {
        // 跳过非工具调用步骤
        if (step.parameters['await_ai_selection'] != true) {
          return step;
        }
        
        // 查找匹配的工具选择
        final selection = toolSelections.firstWhere(
          (s) => s['endpoint_id'] == step.mcpEndpointId,
          orElse: () => null,
        );
        
        if (selection != null) {
          final toolName = selection['tool_name'] as String;
          final reason = selection['reason'] as String;
          
          // 找到对应的工具schema
          final tool = step.availableTools.firstWhere(
            (t) => t.name == toolName,
            orElse: () => step.availableTools.first,
          );
          
          // 更新步骤信息
          return step.copyWith(
            mcpToolId: toolName,
            description: '使用 ${step.mcpEndpointId} 端点的 $toolName 工具执行任务',
            objective: '${tool.description}。AI选择理由：$reason',
            parameters: {
              ...step.parameters,
              'selected_tool': toolName,
              'selection_reason': reason,
              'await_ai_selection': false,
            },
          );
        }
        
        return step;
      }).toList();
      
      // 更新任务计划
      _currentTaskPlan = _currentTaskPlan!.copyWith(steps: updatedSteps);
      
      emit(state.copyWith(
        currentTaskPlan: _currentTaskPlan,
      ));
      
    } catch (e) {
      Log.error('Failed to update steps with AI selection: $e');
    }
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
  
  // AI工具选择
  const factory TaskPlannerEvent.updateWithAIToolSelection({
    required String taskPlanId,
    required Map<String, dynamic> aiSelectionResult,
  }) = _UpdateWithAIToolSelection;
  
  // 更新AI思考过程
  const factory TaskPlannerEvent.updateAIThinkingProcess({
    required String thinkingText,
  }) = _UpdateAIThinkingProcess;
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
    /// AI思考过程文本（用于流式显示）
    String? aiThinkingProcess,
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

