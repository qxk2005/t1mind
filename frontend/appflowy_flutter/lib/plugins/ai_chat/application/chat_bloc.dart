import 'dart:async';

import 'package:appflowy/ai/ai.dart';
import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-error/code.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:collection/collection.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import 'chat_entity.dart';
import 'chat_message_handler.dart';
import 'chat_message_listener.dart';
import 'chat_message_stream.dart';
import 'reasoning_manager.dart';
import 'chat_settings_manager.dart';
import 'chat_stream_manager.dart';

part 'chat_bloc.freezed.dart';

/// Returns current Unix timestamp (seconds since epoch)
int timestamp() {
  return DateTime.now().millisecondsSinceEpoch ~/ 1000;
}

class ChatBloc extends Bloc<ChatEvent, ChatState> {
  ChatBloc({
    required this.chatId,
    required this.userId,
  })  : chatController = InMemoryChatController(),
        listener = ChatMessageListener(chatId: chatId),
        super(ChatState.initial()) {
    // Initialize managers
    _messageHandler = ChatMessageHandler(
      chatId: chatId,
      userId: userId,
      chatController: chatController,
    );

    _streamManager = ChatStreamManager(chatId);
    _settingsManager = ChatSettingsManager(chatId: chatId);

    _startListening();
    _dispatch();
    _loadMessages();
    _loadSettings();
    _checkDefaultAgent();
  }

  final String chatId;
  final String userId;
  final ChatMessageListener listener;
  final ChatController chatController;

  // Managers
  late final ChatMessageHandler _messageHandler;
  late final ChatStreamManager _streamManager;
  late final ChatSettingsManager _settingsManager;

  ChatMessagePB? lastSentMessage;
  
  // 当前选中的智能体ID
  String? selectedAgentId;

  bool isLoadingPreviousMessages = false;
  bool hasMorePreviousMessages = true;
  bool isFetchingRelatedQuestions = false;
  bool shouldFetchRelatedQuestions = false;
  bool _isSendingMessage = false;

  // Accessor for selected sources
  ValueNotifier<List<String>> get selectedSourcesNotifier =>
      _settingsManager.selectedSourcesNotifier;

  @override
  Future<void> close() async {
    // Safely dispose all resources
    await _streamManager.dispose();
    await listener.stop();

    final request = ViewIdPB(value: chatId);
    unawaited(FolderEventCloseView(request).send());

    _settingsManager.dispose();
    chatController.dispose();
    return super.close();
  }

  void _dispatch() {
    on<ChatEvent>((event, emit) async {
      await event.when(
        // Chat settings
        didReceiveChatSettings: (settings) async =>
            _handleChatSettings(settings),
        updateSelectedSources: (selectedSourcesIds) async =>
            _handleUpdateSources(selectedSourcesIds),

        // Agent selection
        selectAgent: (agentId) async => _handleSelectAgent(agentId),

        // Message loading
        didLoadLatestMessages: (messages) async =>
            _handleLatestMessages(messages, emit),
        loadPreviousMessages: () async => _loadPreviousMessagesIfNeeded(),
        didLoadPreviousMessages: (messages, hasMore) async =>
            _handlePreviousMessages(messages, hasMore),

        // Message handling
        receiveMessage: (message) async => _handleReceiveMessage(message),

        // Sending messages
        sendMessage: (message, format, metadata, promptId) async =>
            _handleSendMessage(message, format, metadata, promptId, emit),
        finishSending: () async => emit(
          state.copyWith(
            promptResponseState: PromptResponseState.streamingAnswer,
          ),
        ),

        // Stream control
        stopStream: () async => _handleStopStream(emit),
        failedSending: () async => _handleFailedSending(emit),

        // Answer regeneration
        regenerateAnswer: (id, format, model) async =>
            _handleRegenerateAnswer(id, format, model, emit),

        // Streaming completion
        didFinishAnswerStream: () async {
          _isSendingMessage = false;
          emit(
            state.copyWith(
              promptResponseState: PromptResponseState.ready,
            ),
          );
        },

        // Related questions
        didReceiveRelatedQuestions: (questions) async =>
            _handleRelatedQuestions(
          questions,
          emit,
        ),

        // Message management
        deleteMessage: (message) async => chatController.remove(message),

        // AI follow-up
        onAIFollowUp: (followUpData) async {
          shouldFetchRelatedQuestions =
              followUpData.shouldGenerateRelatedQuestion;
        },
      );
    });
  }

  // Chat settings handlers
  void _handleChatSettings(ChatSettingsPB settings) {
    _settingsManager.selectedSourcesNotifier.value = settings.ragIds;
  }

  Future<void> _handleUpdateSources(List<String> selectedSourcesIds) async {
    await _settingsManager.updateSelectedSources(selectedSourcesIds);
  }

  // Agent selection handler
  Future<void> _handleSelectAgent(String? agentId) async {
    selectedAgentId = agentId;
    Log.info('[ChatBloc] Selected agent: ${agentId ?? "None"}');
  }

  // Message loading handlers
  Future<void> _handleLatestMessages(
    List<Message> messages,
    Emitter<ChatState> emit,
  ) async {
    // 🔧 去重：避免添加已存在的消息（特别是刚发送的用户消息）
    for (final message in messages) {
      // 检查消息是否已存在
      final existingMessage = chatController.messages.firstWhereOrNull(
        (m) => m.id == message.id,
      );
      
      if (existingMessage == null) {
        await chatController.insert(message, index: 0);
      }
    }

    // Check if emit is still valid after async operations
    if (emit.isDone) {
      return;
    }

    switch (state.loadingState) {
      case LoadChatMessageStatus.loading when chatController.messages.isEmpty:
        emit(state.copyWith(loadingState: LoadChatMessageStatus.loadingRemote));
        break;
      case LoadChatMessageStatus.loading:
      case LoadChatMessageStatus.loadingRemote:
        emit(state.copyWith(loadingState: LoadChatMessageStatus.ready));
        break;
      default:
        break;
    }
  }

  void _handlePreviousMessages(List<Message> messages, bool hasMore) {
    // 🔧 去重：避免添加已存在的消息
    for (final message in messages) {
      // 检查消息是否已存在
      final existingMessage = chatController.messages.firstWhereOrNull(
        (m) => m.id == message.id,
      );
      
      if (existingMessage == null) {
        chatController.insert(message, index: 0);
      }
    }

    isLoadingPreviousMessages = false;
    hasMorePreviousMessages = hasMore;
  }

  // Message handling
  void _handleReceiveMessage(Message message) {
    final oldMessage =
        chatController.messages.firstWhereOrNull((m) => m.id == message.id);
    if (oldMessage == null) {
      chatController.insert(message);
    } else {
      chatController.update(oldMessage, message);
    }
  }

  // Message sending handlers
  void _handleSendMessage(
    String message,
    PredefinedFormat? format,
    Map<String, dynamic>? metadata,
    String? promptId,
    Emitter<ChatState> emit,
  ) {
    // 防止重复发送消息
    if (_isSendingMessage) {
      Log.warn("Message sending already in progress, ignoring duplicate request");
      return;
    }
    
    _isSendingMessage = true;
    
    _messageHandler.clearErrorMessages();
    emit(state.copyWith(clearErrorMessages: !state.clearErrorMessages));

    _messageHandler.clearRelatedQuestions();
    
    // 清理之前的推理状态，为新的对话做准备
    final reasoningManager = ReasoningManager();
    reasoningManager.clearReasoning(chatId);
    
    _startStreamingMessage(message, format, metadata, promptId);
    lastSentMessage = null;

    isFetchingRelatedQuestions = false;
    shouldFetchRelatedQuestions = format == null || format.imageFormat.hasText;

    emit(
      state.copyWith(
        promptResponseState: PromptResponseState.sendingQuestion,
      ),
    );
  }

  // Stream control handlers
  Future<void> _handleStopStream(Emitter<ChatState> emit) async {
    await _streamManager.stopStream();

    // Reset sending flag
    _isSendingMessage = false;

    // Allow user input
    emit(state.copyWith(promptResponseState: PromptResponseState.ready));

    // No need to remove old message if stream has started already
    if (_streamManager.hasAnswerStreamStarted) {
      return;
    }

    // Remove the non-started message from the list
    final message = chatController.messages.lastWhereOrNull(
      (e) => e.id == _messageHandler.answerStreamMessageId,
    );
    if (message != null) {
      await chatController.remove(message);
    }

    await _streamManager.disposeAnswerStream();
  }

  void _handleFailedSending(Emitter<ChatState> emit) {
    // Reset sending flag
    _isSendingMessage = false;
    
    final lastMessage = chatController.messages.lastOrNull;
    if (lastMessage != null) {
      chatController.remove(lastMessage);
    }
    emit(state.copyWith(promptResponseState: PromptResponseState.ready));
  }

  // Answer regeneration handler
  void _handleRegenerateAnswer(
    String id,
    PredefinedFormat? format,
    AIModelPB? model,
    Emitter<ChatState> emit,
  ) {
    _messageHandler.clearRelatedQuestions();
    _regenerateAnswer(id, format, model);
    lastSentMessage = null;

    isFetchingRelatedQuestions = false;
    shouldFetchRelatedQuestions = false;

    emit(
      state.copyWith(
        promptResponseState: PromptResponseState.sendingQuestion,
      ),
    );
  }

  // Related questions handler
  void _handleRelatedQuestions(
    List<String> questions,
    Emitter<ChatState> emit,
  ) {
    if (questions.isEmpty) {
      return;
    }

    final metadata = {
      onetimeShotType: OnetimeShotType.relatedQuestion,
      'questions': questions,
    };

    final createdAt = DateTime.now();
    final message = TextMessage(
      id: "related_question_$createdAt",
      text: '',
      metadata: metadata,
      author: const User(id: systemUserId),
      createdAt: createdAt,
    );

    chatController.insert(message);

    emit(
      state.copyWith(
        promptResponseState: PromptResponseState.relatedQuestionsReady,
      ),
    );
  }

  void _startListening() {
    listener.start(
      chatMessageCallback: (pb) {
        if (isClosed) {
          return;
        }

        // 🔧 总是调用 processReceivedMessage 建立临时ID映射
        _messageHandler.processReceivedMessage(pb);
        
        // 🔧 只处理AI消息，用户消息已经在_startStreamingMessage中添加了
        // 这样避免用户消息重复显示，同时保证ID映射正确
        if (pb.authorType == 3) { // 3 means AI message
          final message = _messageHandler.createTextMessage(pb);
          add(ChatEvent.receiveMessage(message));
        }
      },
      chatErrorMessageCallback: (err) {
        if (!isClosed) {
          Log.error("chat error: ${err.errorMessage}");
          add(const ChatEvent.didFinishAnswerStream());
        }
      },
      latestMessageCallback: (list) {
        if (!isClosed) {
          // 加载所有消息（包括用户消息和AI消息）
          final messages = list.messages.map(_messageHandler.createTextMessage).toList();
          add(ChatEvent.didLoadLatestMessages(messages));
        }
      },
      prevMessageCallback: (list) {
        if (!isClosed) {
          // 加载所有消息（包括用户消息和AI消息）
          final messages = list.messages.map(_messageHandler.createTextMessage).toList();
          add(ChatEvent.didLoadPreviousMessages(messages, list.hasMore));
        }
      },
      finishStreamingCallback: () async {
        if (isClosed) {
          return;
        }

        add(const ChatEvent.didFinishAnswerStream());
        unawaited(_fetchRelatedQuestionsIfNeeded());
      },
    );
  }

  // Split method to handle related questions
  Future<void> _fetchRelatedQuestionsIfNeeded() async {
    // Don't fetch related questions if conditions aren't met
    // Log.debug("🔍 [RELATED_Q] Checking if should fetch related questions");
    // Log.debug("🔍 [RELATED_Q] answerStream: ${_streamManager.answerStream != null}");
    // Log.debug("🔍 [RELATED_Q] lastSentMessage: ${lastSentMessage != null}");
    // Log.debug("🔍 [RELATED_Q] shouldFetch: $shouldFetchRelatedQuestions");
    
    if (_streamManager.answerStream == null ||
        lastSentMessage == null ||
        !shouldFetchRelatedQuestions) {
      // Log.debug("🔍 [RELATED_Q] Conditions not met, skipping fetch");
      return;
    }

    final payload = ChatMessageIdPB(
      chatId: chatId,
      messageId: lastSentMessage!.messageId,
    );

    // Log.debug("🔍 [RELATED_Q] Fetching related questions...");
    isFetchingRelatedQuestions = true;
    await AIEventGetRelatedQuestion(payload).send().fold(
      (list) {
        // Log.debug("🔍 [RELATED_Q] Received ${list.items.length} related questions");
        // while fetching related questions, the user might enter a new
        // question or regenerate a previous response. In such cases, don't
        // display the relatedQuestions
        if (!isClosed && isFetchingRelatedQuestions) {
          add(
            ChatEvent.didReceiveRelatedQuestions(
              list.items.map((e) => e.content).toList(),
            ),
          );
          isFetchingRelatedQuestions = false;
        }
      },
      (err) => Log.error("Failed to get related questions: $err"),
    );
  }

  void _loadSettings() async {
    final getChatSettingsPayload =
        AIEventGetChatSettings(ChatId(value: chatId));

    await getChatSettingsPayload.send().fold(
      (settings) {
        if (!isClosed) {
          add(ChatEvent.didReceiveChatSettings(settings: settings));
        }
      },
      (err) => Log.error("Failed to load chat settings: $err"),
    );
  }

  /// 🆕 检查并设置默认智能体
  void _checkDefaultAgent() async {
    try {
      // 获取智能体列表
      final getAgentListPayload = AIEventGetAgentList();
      
      await getAgentListPayload.send().fold(
        (agentList) {
          if (!isClosed && agentList.agents.isNotEmpty) {
            // 自动选择第一个智能体作为默认智能体
            final defaultAgent = agentList.agents.first;
            selectedAgentId = defaultAgent.id;
            Log.info('[ChatBloc] 🆕 自动设置默认智能体: ${defaultAgent.name} (${defaultAgent.id})');
          } else if (!isClosed) {
            Log.info('[ChatBloc] 🆕 没有可用的智能体，不设置默认智能体');
          }
        },
        (err) => Log.error("Failed to load agent list for default selection: $err"),
      );
    } catch (e) {
      Log.error("Error checking default agent: $e");
    }
  }

  void _loadMessages() async {
    final loadMessagesPayload = LoadNextChatMessagePB(
      chatId: chatId,
      limit: Int64(10),
    );

    await AIEventLoadNextMessage(loadMessagesPayload).send().fold(
      (list) {
        if (!isClosed) {
          // 加载所有消息（包括用户消息和AI消息）
          final messages = list.messages.map(_messageHandler.createTextMessage).toList();
          add(ChatEvent.didLoadLatestMessages(messages));
        }
      },
      (err) => Log.error("Failed to load messages: $err"),
    );
  }

  void _loadPreviousMessagesIfNeeded() {
    if (isLoadingPreviousMessages) {
      return;
    }

    final oldestMessage = _messageHandler.getOldestMessage();

    if (oldestMessage != null) {
      final oldestMessageId = Int64.tryParseInt(oldestMessage.id);
      if (oldestMessageId == null) {
        Log.error("Failed to parse message_id: ${oldestMessage.id}");
        return;
      }
      isLoadingPreviousMessages = true;
      _loadPreviousMessages(oldestMessageId);
    }
  }

  void _loadPreviousMessages(Int64? beforeMessageId) {
    final payload = LoadPrevChatMessagePB(
      chatId: chatId,
      limit: Int64(10),
      beforeMessageId: beforeMessageId,
    );
    AIEventLoadPrevMessage(payload).send();
  }

  Future<void> _startStreamingMessage(
    String message,
    PredefinedFormat? format,
    Map<String, dynamic>? metadata,
    String? promptId,
  ) async {
    // Disabled debug logging to reduce noise
    // Log.info("🚀 [STREAM] Starting streaming message: '$message', agentId: $selectedAgentId");
    
    // Prepare streams
    await _streamManager.prepareStreams();

    // 设置QuestionStream的文本内容
    _streamManager.questionStream!.setText(message);

    // Create and add question message
    final questionStreamMessage = _messageHandler.createQuestionStreamMessage(
      _streamManager.questionStream!,
      metadata,
    );
    add(ChatEvent.receiveMessage(questionStreamMessage));

    // Disabled debug logging to reduce noise
    // Log.info("📤 [STREAM] Sending stream request with agentId: $selectedAgentId");
    
    // Send stream request with agent_id
    await _streamManager.sendStreamRequest(message, format, promptId, selectedAgentId).fold(
      (question) {
        if (!isClosed) {
          // Disabled debug logging to reduce noise
          // Log.info("✅ [STREAM] Stream request successful, question ID: ${question.messageId}");
          
          // Create and add answer stream message
          final streamAnswer = _messageHandler.createAnswerStreamMessage(
            stream: _streamManager.answerStream!,
            questionMessageId: question.messageId,
            fakeQuestionMessageId: questionStreamMessage.id,
          );

          lastSentMessage = question;
          add(const ChatEvent.finishSending());
          add(ChatEvent.receiveMessage(streamAnswer));
          
          // Disabled debug logging to reduce noise
          // Log.info("🎯 [STREAM] Answer stream message created and added");
        }
      },
      (err) {
        if (!isClosed) {
          Log.error("❌ [STREAM] Failed to send message: ${err.msg} (code: ${err.code})");

          final metadata = {
            onetimeShotType: OnetimeShotType.error,
            if (err.code != ErrorCode.Internal) errorMessageTextKey: err.msg,
          };

          final error = TextMessage(
            text: '',
            metadata: metadata,
            author: const User(id: systemUserId),
            id: systemUserId,
            createdAt: DateTime.now(),
          );

          add(const ChatEvent.failedSending());
          add(ChatEvent.receiveMessage(error));
        }
      },
    );
  }

  // Refactored method to handle answer regeneration
  void _regenerateAnswer(
    String answerMessageIdString,
    PredefinedFormat? format,
    AIModelPB? model,
  ) async {
    final id = _messageHandler.getEffectiveMessageId(answerMessageIdString);
    final answerMessageId = Int64.tryParseInt(id);
    if (answerMessageId == null) {
      return;
    }

    await _streamManager.prepareStreams();
    await _streamManager
        .sendRegenerateRequest(
      answerMessageId,
      format,
      model,
    )
        .fold(
      (_) {
        if (!isClosed) {
          final streamAnswer = _messageHandler
              .createAnswerStreamMessage(
                stream: _streamManager.answerStream!,
                questionMessageId: answerMessageId - 1,
              )
              .copyWith(id: answerMessageIdString);

          add(ChatEvent.receiveMessage(streamAnswer));
          add(const ChatEvent.finishSending());
        }
      },
      (err) => Log.error("Failed to regenerate answer: ${err.msg}"),
    );
  }
}

@freezed
class ChatEvent with _$ChatEvent {
  // chat settings
  const factory ChatEvent.didReceiveChatSettings({
    required ChatSettingsPB settings,
  }) = _DidReceiveChatSettings;
  const factory ChatEvent.updateSelectedSources({
    required List<String> selectedSourcesIds,
  }) = _UpdateSelectedSources;

  // agent selection
  const factory ChatEvent.selectAgent(String? agentId) = _SelectAgent;

  // send message
  const factory ChatEvent.sendMessage({
    required String message,
    PredefinedFormat? format,
    Map<String, dynamic>? metadata,
    String? promptId,
  }) = _SendMessage;
  const factory ChatEvent.finishSending() = _FinishSendMessage;
  const factory ChatEvent.failedSending() = _FailSendMessage;

  // regenerate
  const factory ChatEvent.regenerateAnswer(
    String id,
    PredefinedFormat? format,
    AIModelPB? model,
  ) = _RegenerateAnswer;

  // streaming answer
  const factory ChatEvent.stopStream() = _StopStream;
  const factory ChatEvent.didFinishAnswerStream() = _DidFinishAnswerStream;

  // receive message
  const factory ChatEvent.receiveMessage(Message message) = _ReceiveMessage;

  // loading messages
  const factory ChatEvent.didLoadLatestMessages(List<Message> messages) =
      _DidLoadMessages;
  const factory ChatEvent.loadPreviousMessages() = _LoadPreviousMessages;
  const factory ChatEvent.didLoadPreviousMessages(
    List<Message> messages,
    bool hasMore,
  ) = _DidLoadPreviousMessages;

  // related questions
  const factory ChatEvent.didReceiveRelatedQuestions(
    List<String> questions,
  ) = _DidReceiveRelatedQueston;

  const factory ChatEvent.deleteMessage(Message message) = _DeleteMessage;

  const factory ChatEvent.onAIFollowUp(AIFollowUpData followUpData) =
      _OnAIFollowUp;
}

@freezed
class ChatState with _$ChatState {
  const factory ChatState({
    required LoadChatMessageStatus loadingState,
    required PromptResponseState promptResponseState,
    required bool clearErrorMessages,
  }) = _ChatState;

  factory ChatState.initial() => const ChatState(
        loadingState: LoadChatMessageStatus.loading,
        promptResponseState: PromptResponseState.ready,
        clearErrorMessages: false,
      );
}

bool isOtherUserMessage(Message message) {
  return message.author.id != aiResponseUserId &&
      message.author.id != systemUserId &&
      !message.author.id.startsWith("streamId:");
}
