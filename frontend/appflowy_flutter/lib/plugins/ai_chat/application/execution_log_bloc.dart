import 'dart:async';

import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-error/errors.pb.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:fixnum/fixnum.dart';

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
    _refreshTimer?.cancel();
    return super.close();
  }

  Future<void> _handleEvent(
    ExecutionLogEvent event,
    Emitter<ExecutionLogState> emit,
  ) async {
    event.when(
      loadLogs: () async => await _loadLogs(emit),
      loadMoreLogs: () async => await _loadMoreLogs(emit),
      refreshLogs: () async => await _refreshLogs(emit),
      searchLogs: (query) async => await _searchLogs(emit, query),
      filterByPhase: (phase) async => await _filterByPhase(emit, phase),
      filterByStatus: (status) async => await _filterByStatus(emit, status),
      toggleAutoScroll: (enabled) => _toggleAutoScroll(emit, enabled),
      addLog: (log) => _addLog(emit, log),
    );
  }

  Future<void> _loadLogs(Emitter<ExecutionLogState> emit) async {
    if (emit.isDone) return;
    
    print('🔍 [ExecutionLogBloc] Starting to load logs...');
    emit(state.copyWith(isLoading: true));

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
    
    // 直接生成模拟数据进行测试
    final mockLogs = _generateMockLogs();
    final response = AgentExecutionLogListPB()
      ..logs.addAll(mockLogs)
      ..total = Int64(mockLogs.length)
      ..hasMore = false;
    
    final result = FlowyResult<AgentExecutionLogListPB, FlowyError>.success(response);
    print('🔍 [ExecutionLogBloc] Generated ${mockLogs.length} mock logs directly');
    
    // 检查emit是否仍然可用
    if (emit.isDone) {
      print('🔍 [ExecutionLogBloc] Emit is done, returning early');
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

  /// 生成模拟日志数据
  List<AgentExecutionLogPB> _generateMockLogs() {
    final now = DateTime.now().millisecondsSinceEpoch;
    
    return [
      AgentExecutionLogPB()
        ..id = 'log_1'
        ..sessionId = _sessionId
        ..messageId = _messageId ?? 'demo_msg_1'
        ..phase = ExecutionPhasePB.ExecPlanning
        ..step = '分析用户问题'
        ..input = '用户问题：请帮我创建一个新文档'
        ..output = '识别到用户需要创建文档，准备调用文档创建工具'
        ..status = ExecutionStatusPB.ExecSuccess
        ..startedAt = Int64(now - 5000)
        ..completedAt = Int64(now - 3000)
        ..durationMs = Int64(2000),
      
      AgentExecutionLogPB()
        ..id = 'log_2'
        ..sessionId = _sessionId
        ..messageId = _messageId ?? 'demo_msg_1'
        ..phase = ExecutionPhasePB.ExecToolCall
        ..step = '调用文档创建工具'
        ..input = '{"title": "新文档", "content": ""}'
        ..output = '{"document_id": "doc_123", "status": "created"}'
        ..status = ExecutionStatusPB.ExecSuccess
        ..startedAt = Int64(now - 3000)
        ..completedAt = Int64(now - 1000)
        ..durationMs = Int64(2000),
      
      AgentExecutionLogPB()
        ..id = 'log_3'
        ..sessionId = _sessionId
        ..messageId = _messageId ?? 'demo_msg_1'
        ..phase = ExecutionPhasePB.ExecCompletion
        ..step = '完成任务'
        ..input = '文档创建成功'
        ..output = '已为您创建新文档，文档ID: doc_123'
        ..status = ExecutionStatusPB.ExecSuccess
        ..startedAt = Int64(now - 1000)
        ..completedAt = Int64(now)
        ..durationMs = Int64(1000),
    ];
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

/// AI事件调度器扩展
class AIEventGetExecutionLogs {
  AIEventGetExecutionLogs(this.request);
  
  final GetExecutionLogsRequestPB request;
  
  Future<FlowyResult<AgentExecutionLogListPB, FlowyError>> send() {
    return _sendRequest();
  }
  
  Future<FlowyResult<AgentExecutionLogListPB, FlowyError>> _sendRequest() async {
    // 暂时直接返回模拟数据，因为后端API还未完全集成
    // 在真实环境中，这里会调用后端API
    print('🔍 [ExecutionLog] Loading logs for sessionId: ${request.sessionId}, messageId: ${request.hasMessageId() ? request.messageId : "none"}');
    await Future.delayed(const Duration(milliseconds: 100)); // 模拟网络延迟
    
    try {
      final response = _generateMockResponse();
      print('🔍 [ExecutionLog] Generated ${response.fold((logs) => logs.logs.length, (error) => 0)} mock logs');
      print('🔍 [ExecutionLog] Returning response to BLoC');
      return response;
    } catch (e) {
      print('🔍 [ExecutionLog] Error generating mock response: $e');
      return FlowyResult<AgentExecutionLogListPB, FlowyError>.failure(
        FlowyError()..msg = 'Failed to generate mock data: $e'
      );
    }
  }
  
  FlowyResult<AgentExecutionLogListPB, FlowyError> _generateMockResponse() {
    final mockLogs = _generateMockLogs();
    print('🔍 [ExecutionLog] Creating response with ${mockLogs.length} logs');
    
    final response = AgentExecutionLogListPB()
      ..logs.addAll(mockLogs)
      ..total = Int64(mockLogs.length)
      ..hasMore = false;
    
    print('🔍 [ExecutionLog] Response created - logs count: ${response.logs.length}, total: ${response.total}');
    final result = FlowyResult<AgentExecutionLogListPB, FlowyError>.success(response);
    print('🔍 [ExecutionLog] FlowyResult created successfully');
    return result;
  }
  
  List<AgentExecutionLogPB> _generateMockLogs() {
    final now = DateTime.now().millisecondsSinceEpoch;
    
    return [
      AgentExecutionLogPB()
        ..id = 'log_1'
        ..sessionId = request.sessionId
        ..messageId = request.hasMessageId() ? request.messageId : 'demo_msg_1'
        ..phase = ExecutionPhasePB.ExecPlanning
        ..step = '分析用户问题'
        ..input = '用户问题：请帮我创建一个新文档'
        ..output = '识别到用户需要创建文档，准备调用文档创建工具'
        ..status = ExecutionStatusPB.ExecSuccess
        ..startedAt = Int64(now - 5000)
        ..completedAt = Int64(now - 3000)
        ..durationMs = Int64(2000),
      
      AgentExecutionLogPB()
        ..id = 'log_2'
        ..sessionId = request.sessionId
        ..messageId = request.hasMessageId() ? request.messageId : 'demo_msg_1'
        ..phase = ExecutionPhasePB.ExecToolCall
        ..step = '调用文档创建工具'
        ..input = '{"title": "新文档", "content": ""}'
        ..output = '{"document_id": "doc_123", "status": "created"}'
        ..status = ExecutionStatusPB.ExecSuccess
        ..startedAt = Int64(now - 3000)
        ..completedAt = Int64(now - 1000)
        ..durationMs = Int64(2000),
      
      AgentExecutionLogPB()
        ..id = 'log_3'
        ..sessionId = request.sessionId
        ..messageId = request.hasMessageId() ? request.messageId : 'demo_msg_1'
        ..phase = ExecutionPhasePB.ExecCompletion
        ..step = '完成任务'
        ..input = '文档创建成功'
        ..output = '已为您创建新文档，文档ID: doc_123'
        ..status = ExecutionStatusPB.ExecSuccess
        ..startedAt = Int64(now - 1000)
        ..completedAt = Int64(now)
        ..durationMs = Int64(1000),
    ];
  }
}
