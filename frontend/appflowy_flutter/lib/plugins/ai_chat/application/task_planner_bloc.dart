import 'dart:async';

import 'package:appflowy_backend/log.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:uuid/uuid.dart';

import 'task_planner_entities.dart';

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
  final Uuid _uuid = const Uuid();

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
        createTaskPlan: (userQuery, mcpTools, agentId) async =>
            _handleCreateTaskPlan(userQuery, mcpTools, agentId, emit),
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
    List<String> mcpTools,
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
    ));

    try {
      // 取消之前的操作
      _currentOperation?.complete();
      _currentOperation = Completer<void>();

      final taskPlan = TaskPlan(
        id: _uuid.v4(),
        userQuery: userQuery,
        overallStrategy: '', // 将由AI生成
        requiredMcpTools: mcpTools,
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

      // 模拟生成的任务规划（实际应该从后端获取）
      final generatedPlan = taskPlan.copyWith(
        overallStrategy: '基于用户查询"$userQuery"生成的任务规划策略',
        steps: _generateMockSteps(mcpTools),
        estimatedDurationSeconds: 120,
        status: TaskPlanStatus.pendingConfirmation,
      );

      _currentTaskPlan = generatedPlan;
      _planHistory.add(generatedPlan);

      emit(state.copyWith(
        status: TaskPlannerStatus.waitingConfirmation,
        currentTaskPlan: generatedPlan,
        planHistory: List.from(_planHistory),
        errorMessage: null,
      ));

      _currentOperation = null;
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to create task plan: $error');
      emit(state.copyWith(
        status: TaskPlannerStatus.planFailed,
        errorMessage: '创建任务规划失败: $error',
      ));
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
      ));
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to update task plan: $error');
      emit(state.copyWith(
        errorMessage: '更新任务规划失败: $error',
      ));
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
      ));
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to confirm task plan: $error');
      emit(state.copyWith(
        errorMessage: '确认任务规划失败: $error',
      ));
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
      ));
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to reject task plan: $error');
      emit(state.copyWith(
        errorMessage: '拒绝任务规划失败: $error',
      ));
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
      ));
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
      ));
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
      ));
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
      ));
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
      ));
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
      ));
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
    ));
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
      ));
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to load plan history: $error');
      emit(state.copyWith(
        errorMessage: '加载规划历史失败: $error',
      ));
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
      ));

      // TODO: 从数据库删除
      // await _taskPlanRepository.delete(taskPlanId);
      
    } catch (error) {
      Log.error('TaskPlannerBloc: Failed to delete plan: $error');
      emit(state.copyWith(
        errorMessage: '删除规划失败: $error',
      ));
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
    ));
  }

  // 清除错误
  Future<void> _handleClearError(
    Emitter<TaskPlannerState> emit,
  ) async {
    emit(state.copyWith(
      errorMessage: null,
    ));
  }

  // 生成模拟任务步骤（用于测试）
  List<TaskStep> _generateMockSteps(List<String> mcpTools) {
    final steps = <TaskStep>[];
    
    for (int i = 0; i < mcpTools.length; i++) {
      final toolId = mcpTools[i];
      steps.add(TaskStep(
        id: _uuid.v4(),
        description: '使用 $toolId 工具执行步骤 ${i + 1}',
        mcpToolId: toolId,
        parameters: {'step': i + 1, 'tool': toolId},
        order: i,
        estimatedDurationSeconds: 30,
      ));
    }

    return steps;
  }
}

/// TaskPlannerEvent - 任务规划器事件定义
@freezed
class TaskPlannerEvent with _$TaskPlannerEvent {
  // 任务规划管理
  const factory TaskPlannerEvent.createTaskPlan({
    required String userQuery,
    required List<String> mcpTools,
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
