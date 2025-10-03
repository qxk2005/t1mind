import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_message_stream.dart';
import 'package:appflowy/plugins/ai_chat/application/reasoning_manager.dart';
import 'package:appflowy/plugins/ai_chat/presentation/message/tool_call_display.dart';
import 'package:appflowy/plugins/ai_chat/presentation/message/task_plan_display.dart';
import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import 'chat_message_service.dart';

part 'chat_ai_message_bloc.freezed.dart';

class ChatAIMessageBloc extends Bloc<ChatAIMessageEvent, ChatAIMessageState> {
  ChatAIMessageBloc({
    dynamic message,
    String? refSourceJsonString,
    required this.chatId,
    required this.questionId,
  }) : super(
          ChatAIMessageState.initial(
            message,
            parseMetadata(refSourceJsonString),
          ),
        ) {
    _registerEventHandlers();
    _initializeStreamListener();
    _checkInitialStreamState();
    _initializeReasoningFromGlobal();
  }

  final String chatId;
  final Int64? questionId;
  final ReasoningManager _reasoningManager = ReasoningManager();

  /// 从全局管理器初始化推理文本
  void _initializeReasoningFromGlobal() {
    final globalReasoningText = _reasoningManager.getReasoningText(chatId);
    final isComplete = _reasoningManager.isReasoningComplete(chatId);
    
    if (globalReasoningText != null && globalReasoningText.isNotEmpty) {
      // Log.debug("🌐 [GLOBAL] Initializing with existing reasoning text length: ${globalReasoningText.length}");
      // 使用add方法而不是直接emit
      add(ChatAIMessageEvent.initializeReasoning(globalReasoningText, isComplete));
    }
  }

  void _registerEventHandlers() {
    on<_UpdateText>((event, emit) {
      // Log.debug("🎯 [REALTIME] UpdateText received, marking reasoning as complete. Text length: ${event.text.length}");
      // Log.debug("🎯 [REALTIME] Current reasoning text length: ${state.reasoningText?.length ?? 0}");
      
      // 标记推理完成
      _reasoningManager.setReasoningComplete(chatId, true);
      
      // 获取全局推理文本
      final globalReasoningText = _reasoningManager.getReasoningText(chatId);
      // Log.debug("🌐 [GLOBAL] Retrieved reasoning text length: ${globalReasoningText?.length ?? 0}");
      
      emit(
        state.copyWith(
          text: event.text,
          messageState: const MessageState.ready(),
          isReasoningComplete: true, // 当开始接收实际回答时，推理完成
          reasoningText: globalReasoningText ?? state.reasoningText, // 使用全局推理文本
        ),
      );
    });

    on<_ReceiveError>((event, emit) {
      emit(state.copyWith(messageState: MessageState.onError(event.error)));
    });

    on<_Retry>((event, emit) async {
      if (questionId == null) {
        Log.error("Question id is not valid: $questionId");
        return;
      }
      emit(state.copyWith(messageState: const MessageState.loading()));
      final payload = ChatMessageIdPB(
        chatId: chatId,
        messageId: questionId,
      );
      final result = await AIEventGetAnswerForQuestion(payload).send();
      if (!isClosed) {
        result.fold(
          (answer) => add(ChatAIMessageEvent.retryResult(answer.content)),
          (err) {
            Log.error("Failed to get answer: $err");
            add(ChatAIMessageEvent.receiveError(err.toString()));
          },
        );
      }
    });

    on<_RetryResult>((event, emit) {
      emit(
        state.copyWith(
          text: event.text,
          messageState: const MessageState.ready(),
        ),
      );
    });

    on<_OnAIResponseLimit>((event, emit) {
      emit(
        state.copyWith(
          messageState: const MessageState.onAIResponseLimit(),
        ),
      );
    });

    on<_OnAIImageResponseLimit>((event, emit) {
      emit(
        state.copyWith(
          messageState: const MessageState.onAIImageResponseLimit(),
        ),
      );
    });

    on<_OnAIMaxRquired>((event, emit) {
      emit(
        state.copyWith(
          messageState: MessageState.onAIMaxRequired(event.message),
        ),
      );
    });

    on<_OnLocalAIInitializing>((event, emit) {
      emit(
        state.copyWith(
          messageState: const MessageState.onInitializingLocalAI(),
        ),
      );
    });

    on<_ReceiveMetadata>((event, emit) {
      Log.debug("AI Steps: ${event.metadata.progress?.step}");
      
      // 处理推理增量数据
      String? updatedReasoningText = state.reasoningText;
      bool isReasoningActive = false;
      
      if (event.metadata.reasoningDelta != null && event.metadata.reasoningDelta!.isNotEmpty) {
        // 更新全局推理文本
        _reasoningManager.appendReasoningText(chatId, event.metadata.reasoningDelta!);
        _reasoningManager.setReasoningComplete(chatId, false);
        
        // 获取更新后的全局推理文本
        updatedReasoningText = _reasoningManager.getReasoningText(chatId);
        isReasoningActive = true; // 接收到推理增量说明推理正在进行
        
        // Log.debug("🔄 [REALTIME] AI Reasoning Delta: '${event.metadata.reasoningDelta}'");
        // Log.debug("📊 [REALTIME] Updated global reasoning text length: ${updatedReasoningText?.length ?? 0}");
        // Log.debug("🌐 [GLOBAL] Stored reasoning text: '$updatedReasoningText'");
        // Log.debug("🚀 [REALTIME] Reasoning is active, isReasoningComplete: false");
      }
      
      // 🔧 处理工具调用 Metadata
      List<ToolCallInfo> updatedToolCalls = state.toolCalls;
      if (event.metadata.rawMetadata != null) {
        updatedToolCalls = _handleToolCallMetadata(event.metadata.rawMetadata!, state.toolCalls);
      }
      
      // 🔧 处理任务规划 Metadata
      TaskPlanInfo? updatedTaskPlan = state.taskPlan;
      if (event.metadata.rawMetadata != null) {
        updatedTaskPlan = _handleTaskPlanMetadata(event.metadata.rawMetadata!, state.taskPlan);
      }
      
      emit(
        state.copyWith(
          sources: event.metadata.sources,
          progress: event.metadata.progress,
          reasoningText: updatedReasoningText,
          isReasoningComplete: isReasoningActive ? false : state.isReasoningComplete, // 保持推理状态
          toolCalls: updatedToolCalls,
          taskPlan: updatedTaskPlan,
        ),
      );
    });

    on<_OnAIFollowUp>((event, emit) {
      emit(
        state.copyWith(
          messageState: MessageState.aiFollowUp(event.followUpData),
        ),
      );
    });

    on<_InitializeReasoning>((event, emit) {
      // Log.debug("🌐 [GLOBAL] Initializing reasoning - text length: ${event.reasoningText.length}, isComplete: ${event.isComplete}");
      emit(
        state.copyWith(
          reasoningText: event.reasoningText,
          isReasoningComplete: event.isComplete,
        ),
      );
    });
  }

  void _initializeStreamListener() {
    if (state.stream != null) {
      state.stream!.listen(
        onData: (text) => _safeAdd(ChatAIMessageEvent.updateText(text)),
        onError: (error) =>
            _safeAdd(ChatAIMessageEvent.receiveError(error.toString())),
        onEnd: () {
          // 流结束时，确保推理过程被标记为完成
          Log.debug("🎯 [STREAM] Stream ended, marking reasoning as complete");
          _reasoningManager.setReasoningComplete(chatId, true);
          
          // 获取最终的推理文本并更新状态
          final finalReasoningText = _reasoningManager.getReasoningText(chatId);
          if (finalReasoningText != null && finalReasoningText.isNotEmpty) {
            _safeAdd(ChatAIMessageEvent.initializeReasoning(finalReasoningText, true));
          }
        },
        onAIResponseLimit: () =>
            _safeAdd(const ChatAIMessageEvent.onAIResponseLimit()),
        onAIImageResponseLimit: () =>
            _safeAdd(const ChatAIMessageEvent.onAIImageResponseLimit()),
        onMetadata: (metadata) =>
            _safeAdd(ChatAIMessageEvent.receiveMetadata(metadata)),
        onAIMaxRequired: (message) {
          Log.info(message);
          _safeAdd(ChatAIMessageEvent.onAIMaxRequired(message));
        },
        onLocalAIInitializing: () =>
            _safeAdd(const ChatAIMessageEvent.onLocalAIInitializing()),
        onAIFollowUp: (data) {
          _safeAdd(ChatAIMessageEvent.onAIFollowUp(data));
        },
      );
    }
  }

  void _checkInitialStreamState() {
    if (state.stream != null) {
      if (state.stream!.aiLimitReached) {
        add(const ChatAIMessageEvent.onAIResponseLimit());
      } else if (state.stream!.error != null) {
        add(ChatAIMessageEvent.receiveError(state.stream!.error!));
      }
    }
  }

  void _safeAdd(ChatAIMessageEvent event) {
    if (!isClosed) {
      add(event);
    }
  }

  // 🔧 处理工具调用 Metadata
  List<ToolCallInfo> _handleToolCallMetadata(
    Map<String, dynamic> metadata,
    List<ToolCallInfo> currentToolCalls,
  ) {
    try {
      if (!metadata.containsKey('tool_call')) {
        return currentToolCalls;
      }

      final toolCallData = metadata['tool_call'] as Map<String, dynamic>?;
      if (toolCallData == null) return currentToolCalls;

      final callId = toolCallData['id'] as String?;
      if (callId == null) return currentToolCalls;

      // 查找是否已存在相同ID的工具调用
      final existingIndex = currentToolCalls.indexWhere((call) => call.id == callId);

      // 解析工具调用信息
      final toolCall = ToolCallInfo(
        id: callId,
        toolName: toolCallData['tool_name'] as String? ?? 'Unknown',
        status: _parseToolCallStatus(toolCallData['status'] as String?),
        arguments: (toolCallData['arguments'] as Map<String, dynamic>?) ?? {},
        description: toolCallData['description'] as String?,
        result: toolCallData['result'] as String?,
        error: toolCallData['error'] as String?,
        startTime: toolCallData['start_time'] != null 
            ? DateTime.tryParse(toolCallData['start_time'] as String)
            : null,
        endTime: toolCallData['end_time'] != null
            ? DateTime.tryParse(toolCallData['end_time'] as String)
            : null,
      );

      Log.debug("🔧 [TOOL] Tool call ${toolCall.status.name}: ${toolCall.toolName} (id: $callId)");

      // 更新或添加工具调用
      if (existingIndex != -1) {
        final updatedList = List<ToolCallInfo>.from(currentToolCalls);
        updatedList[existingIndex] = toolCall;
        return updatedList;
      } else {
        return [...currentToolCalls, toolCall];
      }
    } catch (e) {
      Log.error("Failed to parse tool call metadata: $e");
      return currentToolCalls;
    }
  }

  // 🔧 处理任务规划 Metadata
  TaskPlanInfo? _handleTaskPlanMetadata(
    Map<String, dynamic> metadata,
    TaskPlanInfo? currentPlan,
  ) {
    try {
      if (!metadata.containsKey('task_plan')) {
        return currentPlan;
      }

      final planData = metadata['task_plan'] as Map<String, dynamic>?;
      if (planData == null) return currentPlan;

      final planId = planData['id'] as String?;
      if (planId == null) return currentPlan;

      // 解析步骤列表
      final stepsData = planData['steps'] as List<dynamic>?;
      final steps = stepsData?.map((stepData) {
        final stepMap = stepData as Map<String, dynamic>;
        return TaskStepInfo(
          id: stepMap['id'] as String? ?? '',
          description: stepMap['description'] as String? ?? '',
          status: _parseTaskStepStatus(stepMap['status'] as String?),
          tools: (stepMap['tools'] as List<dynamic>?)
              ?.map((t) => t.toString())
              .toList() ?? [],
          error: stepMap['error'] as String?,
        );
      }).toList() ?? [];

      final plan = TaskPlanInfo(
        id: planId,
        goal: planData['goal'] as String? ?? '',
        status: _parseTaskPlanStatus(planData['status'] as String?),
        steps: steps,
      );

      Log.debug("📋 [PLAN] Task plan ${plan.status.name}: ${plan.goal} (${plan.completedSteps}/${plan.steps.length} steps)");

      return plan;
    } catch (e) {
      Log.error("Failed to parse task plan metadata: $e");
      return currentPlan;
    }
  }

  // 解析工具调用状态
  ToolCallStatus _parseToolCallStatus(String? status) {
    switch (status) {
      case 'pending':
        return ToolCallStatus.pending;
      case 'running':
        return ToolCallStatus.running;
      case 'success':
        return ToolCallStatus.success;
      case 'failed':
        return ToolCallStatus.failed;
      default:
        return ToolCallStatus.pending;
    }
  }

  // 解析任务计划状态
  TaskPlanStatus _parseTaskPlanStatus(String? status) {
    switch (status) {
      case 'pending':
        return TaskPlanStatus.pending;
      case 'running':
        return TaskPlanStatus.running;
      case 'completed':
        return TaskPlanStatus.completed;
      case 'failed':
        return TaskPlanStatus.failed;
      case 'cancelled':
        return TaskPlanStatus.cancelled;
      default:
        return TaskPlanStatus.pending;
    }
  }

  // 解析任务步骤状态
  TaskStepStatus _parseTaskStepStatus(String? status) {
    switch (status) {
      case 'pending':
        return TaskStepStatus.pending;
      case 'running':
        return TaskStepStatus.running;
      case 'completed':
        return TaskStepStatus.completed;
      case 'failed':
        return TaskStepStatus.failed;
      default:
        return TaskStepStatus.pending;
    }
  }
}

@freezed
class ChatAIMessageEvent with _$ChatAIMessageEvent {
  const factory ChatAIMessageEvent.updateText(String text) = _UpdateText;
  const factory ChatAIMessageEvent.receiveError(String error) = _ReceiveError;
  const factory ChatAIMessageEvent.retry() = _Retry;
  const factory ChatAIMessageEvent.retryResult(String text) = _RetryResult;
  const factory ChatAIMessageEvent.onAIResponseLimit() = _OnAIResponseLimit;
  const factory ChatAIMessageEvent.onAIImageResponseLimit() =
      _OnAIImageResponseLimit;
  const factory ChatAIMessageEvent.onAIMaxRequired(String message) =
      _OnAIMaxRquired;
  const factory ChatAIMessageEvent.onLocalAIInitializing() =
      _OnLocalAIInitializing;
  const factory ChatAIMessageEvent.receiveMetadata(
    MetadataCollection metadata,
  ) = _ReceiveMetadata;
  const factory ChatAIMessageEvent.onAIFollowUp(
    AIFollowUpData followUpData,
  ) = _OnAIFollowUp;
  const factory ChatAIMessageEvent.initializeReasoning(
    String reasoningText,
    bool isComplete,
  ) = _InitializeReasoning;
}

@freezed
class ChatAIMessageState with _$ChatAIMessageState {
  const factory ChatAIMessageState({
    AnswerStream? stream,
    required String text,
    required MessageState messageState,
    required List<ChatMessageRefSource> sources,
    required AIChatProgress? progress,
    String? reasoningText,
    @Default(false) bool isReasoningComplete,
    // 🔧 新增字段：工具调用和任务规划
    @Default([]) List<ToolCallInfo> toolCalls,
    TaskPlanInfo? taskPlan,
  }) = _ChatAIMessageState;

  factory ChatAIMessageState.initial(
    dynamic text,
    MetadataCollection metadata,
  ) {
    return ChatAIMessageState(
      text: text is String ? text : "",
      stream: text is AnswerStream ? text : null,
      messageState: const MessageState.ready(),
      sources: metadata.sources,
      progress: metadata.progress,
      reasoningText: null, // 初始状态为空，将从全局管理器获取
    );
  }
}

@freezed
class MessageState with _$MessageState {
  const factory MessageState.onError(String error) = _Error;
  const factory MessageState.onAIResponseLimit() = _AIResponseLimit;
  const factory MessageState.onAIImageResponseLimit() = _AIImageResponseLimit;
  const factory MessageState.onAIMaxRequired(String message) = _AIMaxRequired;
  const factory MessageState.onInitializingLocalAI() = _LocalAIInitializing;
  const factory MessageState.ready() = _Ready;
  const factory MessageState.loading() = _Loading;
  const factory MessageState.aiFollowUp(AIFollowUpData followUpData) =
      _AIFollowUp;
}
