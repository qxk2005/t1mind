import 'dart:async';

import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart' as log_entities;
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart' as planner_entities;
import 'package:appflowy_backend/log.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:equatable/equatable.dart';

/// 执行监控BLoC - 提供任务执行的实时监控和状态更新
/// 
/// 参考AI消息BLoC的流式处理模式，支持：
/// - 实时进度更新
/// - 执行状态监控
/// - 错误处理和重试
/// - 取消操作
/// - 长时间运行任务的处理
class ExecutionBloc extends Bloc<ExecutionEvent, ExecutionState> {
  ExecutionBloc({
    required this.sessionId,
    required this.executionId,
    this.taskPlanId,
  }) : super(const ExecutionState.initial()) {
    _registerEventHandlers();
    _initializeExecution();
  }

  final String sessionId;
  final String executionId;
  final String? taskPlanId;

  StreamSubscription<planner_entities.ExecutionProgress>? _progressSubscription;
  StreamSubscription<log_entities.ExecutionStep>? _stepSubscription;
  Timer? _timeoutTimer;
  Timer? _heartbeatTimer;

  @override
  Future<void> close() async {
    await _progressSubscription?.cancel();
    await _stepSubscription?.cancel();
    _timeoutTimer?.cancel();
    _heartbeatTimer?.cancel();
    return super.close();
  }

  void _registerEventHandlers() {
    // 开始执行
    on<StartExecutionEvent>((event, emit) async {
      emit(const ExecutionState.preparing());
      
      try {
        // 启动执行 - 这里应该调用FFI接口
        await Future.delayed(const Duration(milliseconds: 100)); // 模拟异步调用
        
        final executionLog = log_entities.ExecutionLog(
          id: executionId,
          sessionId: sessionId,
          userQuery: event.userQuery ?? 'Unknown query',
          startTime: DateTime.now(),
          totalSteps: event.totalSteps ?? 5,
        );
        
        if (!isClosed) {
          _startProgressMonitoring();
          _startHeartbeat();
          emit(ExecutionState.running(
            executionLog: executionLog,
            progress: planner_entities.ExecutionProgress(
              totalSteps: executionLog.totalSteps,
              status: planner_entities.ExecutionStatus.running,
              startTime: DateTime.now(),
            ),
          ));
        }
      } catch (e) {
        if (!isClosed) {
          Log.error("Exception starting execution: $e");
          emit(ExecutionState.error(
            error: e.toString(),
            errorType: log_entities.ExecutionErrorType.system,
            canRetry: true,
          ));
        }
      }
    });

    // 暂停执行
    on<PauseExecutionEvent>((event, emit) async {
      final currentState = state;
      if (currentState is! RunningExecutionState) return;
      
      try {
        // 调用暂停接口
        await Future.delayed(const Duration(milliseconds: 100)); // 模拟异步调用
        
        if (!isClosed) {
          _pauseMonitoring();
          emit(ExecutionState.paused(
            executionLog: currentState.executionLog,
            progress: currentState.progress.copyWith(
              status: planner_entities.ExecutionStatus.paused,
            ),
          ));
        }
      } catch (e) {
        if (!isClosed) {
          Log.error("Exception pausing execution: $e");
          add(ReceiveErrorEvent(
            error: e.toString(),
            errorType: log_entities.ExecutionErrorType.system,
          ));
        }
      }
    });

    // 恢复执行
    on<ResumeExecutionEvent>((event, emit) async {
      final currentState = state;
      if (currentState is! PausedExecutionState) return;
      
      try {
        // 调用恢复接口
        await Future.delayed(const Duration(milliseconds: 100)); // 模拟异步调用
        
        if (!isClosed) {
          _resumeMonitoring();
          emit(ExecutionState.running(
            executionLog: currentState.executionLog,
            progress: currentState.progress.copyWith(
              status: planner_entities.ExecutionStatus.running,
            ),
          ));
        }
      } catch (e) {
        if (!isClosed) {
          Log.error("Exception resuming execution: $e");
          add(ReceiveErrorEvent(
            error: e.toString(),
            errorType: log_entities.ExecutionErrorType.system,
          ));
        }
      }
    });

    // 取消执行
    on<CancelExecutionEvent>((event, emit) async {
      try {
        // 调用取消接口
        await Future.delayed(const Duration(milliseconds: 100)); // 模拟异步调用
        
        if (!isClosed) {
          _stopMonitoring();
          emit(ExecutionState.cancelled(
            reason: event.reason ?? 'User cancelled',
          ));
        }
      } catch (e) {
        if (!isClosed) {
          Log.error("Exception cancelling execution: $e");
          add(ReceiveErrorEvent(
            error: e.toString(),
            errorType: log_entities.ExecutionErrorType.system,
          ));
        }
      }
    });

    // 重试执行
    on<RetryExecutionEvent>((event, emit) async {
      final currentState = state;
      if (currentState is! ErrorExecutionState) return;
      
      emit(const ExecutionState.preparing());
      
      try {
        // 调用重试接口
        await Future.delayed(const Duration(milliseconds: 100)); // 模拟异步调用
        
        final executionLog = log_entities.ExecutionLog(
          id: executionId,
          sessionId: sessionId,
          userQuery: 'Retry execution',
          startTime: DateTime.now(),
          totalSteps: 5,
        );
        
        if (!isClosed) {
          _startProgressMonitoring();
          _startHeartbeat();
          emit(ExecutionState.running(
            executionLog: executionLog,
            progress: planner_entities.ExecutionProgress(
              currentStep: event.fromStep ?? 0,
              totalSteps: executionLog.totalSteps,
              status: planner_entities.ExecutionStatus.running,
              startTime: DateTime.now(),
            ),
          ));
        }
      } catch (e) {
        if (!isClosed) {
          Log.error("Exception retrying execution: $e");
          emit(ExecutionState.error(
            error: e.toString(),
            errorType: log_entities.ExecutionErrorType.system,
            canRetry: true,
          ));
        }
      }
    });

    // 接收进度更新
    on<ReceiveProgressEvent>((event, emit) {
      final currentState = state;
      
      if (currentState is RunningExecutionState) {
        // 检查是否完成
        if (event.progress.isCompleted) {
          _stopMonitoring();
          emit(ExecutionState.completed(
            executionLog: currentState.executionLog,
            result: const {},
          ));
        } else {
          emit(currentState.copyWith(progress: event.progress));
        }
      } else if (currentState is PausedExecutionState) {
        emit(currentState.copyWith(progress: event.progress));
      }
    });

    // 接收步骤更新
    on<ReceiveStepUpdateEvent>((event, emit) {
      final currentState = state;
      
      if (currentState is RunningExecutionState) {
        // 更新执行日志中的步骤
        final updatedLog = _updateExecutionLogStep(
          currentState.executionLog,
          event.step,
        );
        
        emit(currentState.copyWith(
          executionLog: updatedLog,
          progress: currentState.progress.copyWith(
            currentStep: event.step.order + 1,
            currentStepDescription: event.step.description,
          ),
        ));
      } else if (currentState is PausedExecutionState) {
        final updatedLog = _updateExecutionLogStep(
          currentState.executionLog,
          event.step,
        );
        
        emit(currentState.copyWith(executionLog: updatedLog));
      }
    });

    // 接收错误
    on<ReceiveErrorEvent>((event, emit) {
      _stopMonitoring();
      emit(ExecutionState.error(
        error: event.error,
        errorType: event.errorType,
        canRetry: event.errorType.canRetry,
        step: event.step,
      ));
    });

    // 接收超时
    on<ReceiveTimeoutEvent>((event, emit) {
      _stopMonitoring();
      emit(ExecutionState.error(
        error: '执行超时',
        errorType: log_entities.ExecutionErrorType.timeout,
        canRetry: true,
        step: event.step,
      ));
    });

    // 刷新状态
    on<RefreshStatusEvent>((event, emit) async {
      try {
        // 调用状态刷新接口
        await Future.delayed(const Duration(milliseconds: 100)); // 模拟异步调用
        
        // 根据获取的状态更新当前状态
        Log.debug("Status refreshed");
      } catch (e) {
        Log.error("Exception refreshing status: $e");
      }
    });
  }

  void _initializeExecution() {
    // 设置超时定时器（30分钟）
    _timeoutTimer = Timer(const Duration(minutes: 30), () {
      if (!isClosed) {
        add(const ReceiveTimeoutEvent());
      }
    });
  }

  void _startProgressMonitoring() {
    // 启动进度监控流
    _progressSubscription = _createProgressStream().listen(
      (progress) {
        if (!isClosed) {
          add(ReceiveProgressEvent(progress));
        }
      },
      onError: (error) {
        if (!isClosed) {
          Log.error("Progress stream error: $error");
          add(ReceiveErrorEvent(
            error: error.toString(),
            errorType: log_entities.ExecutionErrorType.system,
          ));
        }
      },
    );

    // 启动步骤监控流
    _stepSubscription = _createStepStream().listen(
      (step) {
        if (!isClosed) {
          add(ReceiveStepUpdateEvent(step));
        }
      },
      onError: (error) {
        if (!isClosed) {
          Log.error("Step stream error: $error");
          add(ReceiveErrorEvent(
            error: error.toString(),
            errorType: log_entities.ExecutionErrorType.system,
          ));
        }
      },
    );
  }

  void _pauseMonitoring() {
    _progressSubscription?.pause();
    _stepSubscription?.pause();
    _heartbeatTimer?.cancel();
  }

  void _resumeMonitoring() {
    _progressSubscription?.resume();
    _stepSubscription?.resume();
    _startHeartbeat();
  }

  void _stopMonitoring() {
    _progressSubscription?.cancel();
    _stepSubscription?.cancel();
    _heartbeatTimer?.cancel();
    _timeoutTimer?.cancel();
  }

  void _startHeartbeat() {
    // 每10秒发送心跳检查状态
    _heartbeatTimer = Timer.periodic(const Duration(seconds: 10), (timer) {
      if (!isClosed) {
        add(const RefreshStatusEvent());
      }
    });
  }

  Stream<planner_entities.ExecutionProgress> _createProgressStream() async* {
    // 这里应该连接到Rust层的进度流
    // 暂时使用模拟实现
    int currentStep = 0;
    const totalSteps = 5;
    
    while (currentStep < totalSteps) {
      await Future.delayed(const Duration(seconds: 2));
      currentStep++;
      
      yield planner_entities.ExecutionProgress(
        currentStep: currentStep,
        totalSteps: totalSteps,
        currentStepDescription: 'Step $currentStep of $totalSteps',
        status: currentStep >= totalSteps 
            ? planner_entities.ExecutionStatus.completed 
            : planner_entities.ExecutionStatus.running,
        startTime: DateTime.now().subtract(Duration(seconds: currentStep * 2)),
        estimatedRemainingSeconds: (totalSteps - currentStep) * 2,
      );
    }
  }

  Stream<log_entities.ExecutionStep> _createStepStream() async* {
    // 这里应该连接到Rust层的步骤更新流
    // 暂时使用模拟实现
    for (int i = 0; i < 5; i++) {
      await Future.delayed(const Duration(seconds: 2));
      
      yield log_entities.ExecutionStep(
        id: 'step_$i',
        executionLogId: executionId,
        name: 'Step ${i + 1}',
        description: 'Executing step ${i + 1}',
        mcpTool: log_entities.McpToolInfo(
          id: 'tool_$i',
          name: 'Tool ${i + 1}',
        ),
        status: log_entities.ExecutionStepStatus.success,
        order: i,
        startTime: DateTime.now().subtract(Duration(seconds: (5 - i) * 2)),
        endTime: DateTime.now().subtract(Duration(seconds: (5 - i - 1) * 2)),
      );
    }
  }

  log_entities.ExecutionLog _updateExecutionLogStep(
    log_entities.ExecutionLog log,
    log_entities.ExecutionStep step,
  ) {
    final updatedSteps = log.steps.map((existingStep) {
      if (existingStep.id == step.id) {
        return step;
      }
      return existingStep;
    }).toList();

    // 如果步骤不存在，添加它
    if (!updatedSteps.any((s) => s.id == step.id)) {
      updatedSteps.add(step);
    }

    // 更新统计信息
    final completedCount = updatedSteps
        .where((s) => s.status == log_entities.ExecutionStepStatus.success)
        .length;
    final failedCount = updatedSteps
        .where((s) => s.status == log_entities.ExecutionStepStatus.error)
        .length;

    return log.copyWith(
      steps: updatedSteps,
      completedSteps: completedCount,
      failedSteps: failedCount,
      status: _calculateLogStatus(updatedSteps),
    );
  }

  log_entities.ExecutionLogStatus _calculateLogStatus(List<log_entities.ExecutionStep> steps) {
    if (steps.isEmpty) return log_entities.ExecutionLogStatus.initialized;
    
    final hasRunning = steps.any((s) => s.status == log_entities.ExecutionStepStatus.executing);
    if (hasRunning) return log_entities.ExecutionLogStatus.running;
    
    final hasError = steps.any((s) => s.status == log_entities.ExecutionStepStatus.error);
    if (hasError) return log_entities.ExecutionLogStatus.failed;
    
    final allCompleted = steps.every((s) => s.status.isFinished);
    if (allCompleted) {
      final allSuccess = steps.every((s) => 
          s.status.isSuccessful || s.status == log_entities.ExecutionStepStatus.skipped);
      return allSuccess 
          ? log_entities.ExecutionLogStatus.completed 
          : log_entities.ExecutionLogStatus.failed;
    }
    
    return log_entities.ExecutionLogStatus.running;
  }
}

/// 执行事件基类
abstract class ExecutionEvent extends Equatable {
  const ExecutionEvent();
}

/// 开始执行事件
class StartExecutionEvent extends ExecutionEvent {
  const StartExecutionEvent({
    this.context = const {},
    this.userQuery,
    this.totalSteps,
  });

  final Map<String, dynamic> context;
  final String? userQuery;
  final int? totalSteps;

  @override
  List<Object?> get props => [context, userQuery, totalSteps];
}

/// 暂停执行事件
class PauseExecutionEvent extends ExecutionEvent {
  const PauseExecutionEvent();

  @override
  List<Object?> get props => [];
}

/// 恢复执行事件
class ResumeExecutionEvent extends ExecutionEvent {
  const ResumeExecutionEvent();

  @override
  List<Object?> get props => [];
}

/// 取消执行事件
class CancelExecutionEvent extends ExecutionEvent {
  const CancelExecutionEvent({this.reason});

  final String? reason;

  @override
  List<Object?> get props => [reason];
}

/// 重试执行事件
class RetryExecutionEvent extends ExecutionEvent {
  const RetryExecutionEvent({this.fromStep});

  final int? fromStep;

  @override
  List<Object?> get props => [fromStep];
}

/// 接收进度更新事件
class ReceiveProgressEvent extends ExecutionEvent {
  const ReceiveProgressEvent(this.progress);

  final planner_entities.ExecutionProgress progress;

  @override
  List<Object?> get props => [progress];
}

/// 接收步骤更新事件
class ReceiveStepUpdateEvent extends ExecutionEvent {
  const ReceiveStepUpdateEvent(this.step);

  final log_entities.ExecutionStep step;

  @override
  List<Object?> get props => [step];
}

/// 接收错误事件
class ReceiveErrorEvent extends ExecutionEvent {
  const ReceiveErrorEvent({
    required this.error,
    required this.errorType,
    this.step,
  });

  final String error;
  final log_entities.ExecutionErrorType errorType;
  final log_entities.ExecutionStep? step;

  @override
  List<Object?> get props => [error, errorType, step];
}

/// 接收超时事件
class ReceiveTimeoutEvent extends ExecutionEvent {
  const ReceiveTimeoutEvent({this.step});

  final log_entities.ExecutionStep? step;

  @override
  List<Object?> get props => [step];
}

/// 刷新状态事件
class RefreshStatusEvent extends ExecutionEvent {
  const RefreshStatusEvent();

  @override
  List<Object?> get props => [];
}

/// 执行状态基类
abstract class ExecutionState extends Equatable {
  const ExecutionState();

  /// 初始状态
  const factory ExecutionState.initial() = InitialExecutionState;

  /// 准备中
  const factory ExecutionState.preparing() = PreparingExecutionState;

  /// 运行中
  const factory ExecutionState.running({
    required log_entities.ExecutionLog executionLog,
    required planner_entities.ExecutionProgress progress,
  }) = RunningExecutionState;

  /// 暂停
  const factory ExecutionState.paused({
    required log_entities.ExecutionLog executionLog,
    required planner_entities.ExecutionProgress progress,
  }) = PausedExecutionState;

  /// 完成
  const factory ExecutionState.completed({
    required log_entities.ExecutionLog executionLog,
    Map<String, dynamic>? result,
  }) = CompletedExecutionState;

  /// 错误
  const factory ExecutionState.error({
    required String error,
    required log_entities.ExecutionErrorType errorType,
    required bool canRetry,
    log_entities.ExecutionStep? step,
  }) = ErrorExecutionState;

  /// 已取消
  const factory ExecutionState.cancelled({
    required String reason,
  }) = CancelledExecutionState;
}

/// 初始状态
class InitialExecutionState extends ExecutionState {
  const InitialExecutionState();

  @override
  List<Object?> get props => [];
}

/// 准备中状态
class PreparingExecutionState extends ExecutionState {
  const PreparingExecutionState();

  @override
  List<Object?> get props => [];
}

/// 运行中状态
class RunningExecutionState extends ExecutionState {
  const RunningExecutionState({
    required this.executionLog,
    required this.progress,
  });

  final log_entities.ExecutionLog executionLog;
  final planner_entities.ExecutionProgress progress;

  @override
  List<Object?> get props => [executionLog, progress];

  RunningExecutionState copyWith({
    log_entities.ExecutionLog? executionLog,
    planner_entities.ExecutionProgress? progress,
  }) {
    return RunningExecutionState(
      executionLog: executionLog ?? this.executionLog,
      progress: progress ?? this.progress,
    );
  }
}

/// 暂停状态
class PausedExecutionState extends ExecutionState {
  const PausedExecutionState({
    required this.executionLog,
    required this.progress,
  });

  final log_entities.ExecutionLog executionLog;
  final planner_entities.ExecutionProgress progress;

  @override
  List<Object?> get props => [executionLog, progress];

  PausedExecutionState copyWith({
    log_entities.ExecutionLog? executionLog,
    planner_entities.ExecutionProgress? progress,
  }) {
    return PausedExecutionState(
      executionLog: executionLog ?? this.executionLog,
      progress: progress ?? this.progress,
    );
  }
}

/// 完成状态
class CompletedExecutionState extends ExecutionState {
  const CompletedExecutionState({
    required this.executionLog,
    this.result,
  });

  final log_entities.ExecutionLog executionLog;
  final Map<String, dynamic>? result;

  @override
  List<Object?> get props => [executionLog, result];
}

/// 错误状态
class ErrorExecutionState extends ExecutionState {
  const ErrorExecutionState({
    required this.error,
    required this.errorType,
    required this.canRetry,
    this.step,
  });

  final String error;
  final log_entities.ExecutionErrorType errorType;
  final bool canRetry;
  final log_entities.ExecutionStep? step;

  @override
  List<Object?> get props => [error, errorType, canRetry, step];
}

/// 已取消状态
class CancelledExecutionState extends ExecutionState {
  const CancelledExecutionState({
    required this.reason,
  });

  final String reason;

  @override
  List<Object?> get props => [reason];
}