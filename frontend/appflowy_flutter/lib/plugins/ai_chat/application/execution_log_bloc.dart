import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'execution_log_bloc.freezed.dart';

/// 执行日志查看器的BLoC状态管理
class ExecutionLogBloc extends Bloc<ExecutionLogEvent, ExecutionLogState> {
  ExecutionLogBloc({
    required String sessionId,
    String? messageId,
  }) : _sessionId = sessionId,
       _messageId = messageId,
       super(ExecutionLogState.initial()) {
    on<ExecutionLogEvent>(_handleEvent);
  }

  final String _sessionId;
  final String? _messageId;
  Timer? _refreshTimer;

  @override
  Future<void> close() {
    print('🔍 [ExecutionLogBloc] ❌ CLOSING BLOC for session: $_sessionId, message: $_messageId');
    print('🔍 [ExecutionLogBloc] ❌ Stack trace: ${StackTrace.current}');
    _refreshTimer?.cancel();
    return super.close();
  }

  Future<void> _handleEvent(
    ExecutionLogEvent event,
    Emitter<ExecutionLogState> emit,
  ) async {
    // ⚠️ 关键修复：使用类型检查并 await 异步操作
    // 这样可以确保异步操作完成后才标记 emitter 为 done
    if (event is _LoadLogs) {
      await _loadLogs(emit);
    } else if (event is _LoadMoreLogs) {
      await _loadMoreLogs(emit);
    } else if (event is _RefreshLogs) {
      await _refreshLogs(emit);
    } else if (event is _SearchLogs) {
      await _searchLogs(emit, event.query);
    } else if (event is _FilterByPhase) {
      await _filterByPhase(emit, event.phase);
    } else if (event is _FilterByStatus) {
      await _filterByStatus(emit, event.status);
    } else if (event is _ToggleAutoScroll) {
      _toggleAutoScroll(emit, event.enabled);
    } else if (event is _AddLog) {
      _addLog(emit, event.log);
    }
  }

  Future<void> _loadLogs(Emitter<ExecutionLogState> emit) async {
    print('🔍 [ExecutionLogBloc] 🔵 _loadLogs called');
    print('🔍 [ExecutionLogBloc] 🔵 emit.isDone: ${emit.isDone}');
    print('🔍 [ExecutionLogBloc] 🔵 isClosed: $isClosed');
    print('🔍 [ExecutionLogBloc] 🔵 session: $_sessionId, message: $_messageId');
    
    if (emit.isDone) {
      print('🔍 [ExecutionLogBloc] ⚠️ emit.isDone is true at start, returning');
      return;
    }
    
    print('🔍 [ExecutionLogBloc] Starting to load logs...');
    emit(state.copyWith(isLoading: true));
    print('🔍 [ExecutionLogBloc] Emitted isLoading: true');

    final request = GetExecutionLogsRequestPB()
      ..sessionId = _sessionId
      ..limit = 50
      ..offset = 0;

    if (_messageId != null) {
      request.messageId = _messageId;
    }

    if (state.phaseFilter != null) {
      request.phase = state.phaseFilter!;
    }

    print('🔍 [ExecutionLogBloc] Calling AIEventGetExecutionLogs...');
    print('🔍 [ExecutionLogBloc] 🔵 Before API call - emit.isDone: ${emit.isDone}, isClosed: $isClosed');
    
    // 🔌 使用真实的后端API
    final result = await AIEventGetExecutionLogs(request).send();
    
    print('🔍 [ExecutionLogBloc] 🔵 Received response from backend');
    print('🔍 [ExecutionLogBloc] 🔵 After API call - emit.isDone: ${emit.isDone}, isClosed: $isClosed');
    
    // 检查emit是否仍然可用
    if (emit.isDone) {
      print('🔍 [ExecutionLogBloc] ❌ Emit is done, returning early');
      print('🔍 [ExecutionLogBloc] ❌ This means the Bloc was closed during the async operation!');
      return;
    }
    
    print('🔍 [ExecutionLogBloc] Processing result...');
    result.fold(
      (logs) {
        print('🔍 [ExecutionLogBloc] Result is success with ${logs.logs.length} logs');
        if (!emit.isDone) {
          print('🔍 [ExecutionLogBloc] Successfully loaded ${logs.logs.length} logs');
          final newState = state.copyWith(
            isLoading: false,
            logs: logs.logs,
            hasMore: logs.hasMore,
            totalCount: logs.total.toInt(),
            offset: logs.logs.length,
          );
          print('🔍 [ExecutionLogBloc] Emitting new state with ${newState.logs.length} logs, isLoading: ${newState.isLoading}');
          emit(newState);

          // 如果启用了自动滚动，开始定时刷新
          if (state.autoScroll) {
            _startAutoRefresh();
          }
        } else {
          print('🔍 [ExecutionLogBloc] Emit is done, cannot emit new state');
        }
      },
      (error) {
        print('🔍 [ExecutionLogBloc] Result is error: ${error.hasMsg() ? error.msg : 'Unknown error'}');
        if (!emit.isDone) {
          print('🔍 [ExecutionLogBloc] Error loading logs: ${error.hasMsg() ? error.msg : 'Unknown error'}');
          emit(state.copyWith(
            isLoading: false,
            error: error.hasMsg() ? error.msg : 'Unknown error',
          ));
        }
      },
    );
  }

  Future<void> _loadMoreLogs(Emitter<ExecutionLogState> emit) async {
    if (state.isLoading || !state.hasMore) return;
    if (emit.isDone) return;

    emit(state.copyWith(isLoadingMore: true));

    final request = GetExecutionLogsRequestPB()
      ..sessionId = _sessionId
      ..limit = 50
      ..offset = state.offset;

    if (_messageId != null) {
      request.messageId = _messageId;
    }

    if (state.phaseFilter != null) {
      request.phase = state.phaseFilter!;
    }

    // 🔌 使用真实的后端API
    final result = await AIEventGetExecutionLogs(request).send();
    
    // 检查emit是否仍然可用
    if (emit.isDone) return;
    
    result.fold(
      (logs) {
        if (!emit.isDone) {
          final allLogs = [...state.logs, ...logs.logs];
          emit(state.copyWith(
            isLoadingMore: false,
            logs: allLogs,
            hasMore: logs.hasMore,
            totalCount: logs.total.toInt(),
            offset: allLogs.length,
          ));
        }
      },
      (error) {
        if (!emit.isDone) {
          emit(state.copyWith(
            isLoadingMore: false,
            error: error.hasMsg() ? error.msg : 'Unknown error',
          ));
        }
      },
    );
  }

  Future<void> _refreshLogs(Emitter<ExecutionLogState> emit) async {
    if (emit.isDone) return;
    
    final newState = state.copyWith(offset: 0);
    emit(newState);
    
    if (!emit.isDone) {
      await _loadLogs(emit);
    }
  }

  Future<void> _searchLogs(Emitter<ExecutionLogState> emit, String query) async {
    if (emit.isDone) return;
    
    emit(state.copyWith(
      searchQuery: query,
      offset: 0,
    ));

    // 重新加载日志（在实际实现中，应该在后端支持搜索）
    if (!emit.isDone) {
      await _loadLogs(emit);
    }
  }

  Future<void> _filterByPhase(
    Emitter<ExecutionLogState> emit,
    ExecutionPhasePB? phase,
  ) async {
    if (emit.isDone) return;
    
    emit(state.copyWith(
      phaseFilter: phase,
      offset: 0,
    ));
    
    if (!emit.isDone) {
      await _loadLogs(emit);
    }
  }

  Future<void> _filterByStatus(
    Emitter<ExecutionLogState> emit,
    ExecutionStatusPB? status,
  ) async {
    if (emit.isDone) return;
    
    emit(state.copyWith(
      statusFilter: status,
      offset: 0,
    ));
    
    if (!emit.isDone) {
      await _loadLogs(emit);
    }
  }

  void _toggleAutoScroll(Emitter<ExecutionLogState> emit, bool enabled) {
    if (emit.isDone) return;
    
    emit(state.copyWith(autoScroll: enabled));
    
    if (enabled) {
      _startAutoRefresh();
    } else {
      _stopAutoRefresh();
    }
  }

  void _addLog(Emitter<ExecutionLogState> emit, AgentExecutionLogPB log) {
    if (emit.isDone) return;
    
    // 实时添加新日志
    final updatedLogs = [log, ...state.logs];
    emit(state.copyWith(
      logs: updatedLogs,
      totalCount: state.totalCount + 1,
    ));
  }

  void _startAutoRefresh() {
    _refreshTimer?.cancel();
    _refreshTimer = Timer.periodic(
      const Duration(seconds: 2),
      (_) => add(const ExecutionLogEvent.refreshLogs()),
    );
  }

  void _stopAutoRefresh() {
    _refreshTimer?.cancel();
  }
}

/// 执行日志事件
@freezed
class ExecutionLogEvent with _$ExecutionLogEvent {
  const factory ExecutionLogEvent.loadLogs() = _LoadLogs;
  const factory ExecutionLogEvent.loadMoreLogs() = _LoadMoreLogs;
  const factory ExecutionLogEvent.refreshLogs() = _RefreshLogs;
  const factory ExecutionLogEvent.searchLogs(String query) = _SearchLogs;
  const factory ExecutionLogEvent.filterByPhase(ExecutionPhasePB? phase) = _FilterByPhase;
  const factory ExecutionLogEvent.filterByStatus(ExecutionStatusPB? status) = _FilterByStatus;
  const factory ExecutionLogEvent.toggleAutoScroll(bool enabled) = _ToggleAutoScroll;
  const factory ExecutionLogEvent.addLog(AgentExecutionLogPB log) = _AddLog;
}

/// 执行日志状态
@freezed
class ExecutionLogState with _$ExecutionLogState {
  const factory ExecutionLogState({
    required List<AgentExecutionLogPB> logs,
    required bool isLoading,
    required bool isLoadingMore,
    required bool hasMore,
    required int totalCount,
    required int offset,
    required String searchQuery,
    required ExecutionPhasePB? phaseFilter,
    required ExecutionStatusPB? statusFilter,
    required bool autoScroll,
    String? error,
  }) = _ExecutionLogState;

  factory ExecutionLogState.initial() => const ExecutionLogState(
    logs: [],
    isLoading: false,
    isLoadingMore: false,
    hasMore: true,
    totalCount: 0,
    offset: 0,
    searchQuery: '',
    phaseFilter: null,
    statusFilter: null,
    autoScroll: false,
  );
}

