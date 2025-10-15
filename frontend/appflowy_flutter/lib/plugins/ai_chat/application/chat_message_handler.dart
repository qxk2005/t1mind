import 'dart:collection';
import 'dart:convert';

import 'package:appflowy/util/int64_extension.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:collection/collection.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:nanoid/nanoid.dart';

import 'chat_entity.dart';
import 'sources_manager.dart';
import 'reasoning_manager.dart';
import 'chat_message_stream.dart';

/// Returns current Unix timestamp (seconds since epoch)
int timestamp() {
  return DateTime.now().millisecondsSinceEpoch ~/ 1000;
}

/// Handles message creation and manipulation for the chat system
class ChatMessageHandler {
  ChatMessageHandler({
    required this.chatId,
    required this.userId,
    required this.chatController,
  });
  final String chatId;
  final String userId;
  final ChatController chatController;

  /// Maps real message IDs to temporary streaming message IDs
  final HashMap<String, String> _temporaryMessageIDMap = HashMap();

  /// Gets the effective message ID from the temporary map
  String getEffectiveMessageId(String messageId) {
    return _temporaryMessageIDMap.entries
            .firstWhereOrNull((entry) => entry.value == messageId)
            ?.key ??
        messageId;
  }

  String answerStreamMessageId = '';
  String questionStreamMessageId = '';

  /// Create a message from ChatMessagePB object
  Message createTextMessage(ChatMessagePB message) {
    String messageId = message.messageId.toString();
    
    // 🔍 调试：打印消息信息
    print('🔍 [CREATE-MESSAGE] messageId: $messageId, authorType: ${message.authorType}, hasReplyMessageId: ${message.hasReplyMessageId()}, replyMessageId: ${message.hasReplyMessageId() ? message.replyMessageId.toString() : "null"}');

    /// If the message id is in the temporary map, we will use the previous fake message id
    if (_temporaryMessageIDMap.containsKey(messageId)) {
      messageId = _temporaryMessageIDMap[messageId]!;
    }
    
    String finalMetadata = message.metadata == 'null' ? '[]' : message.metadata;
    
    // 🔧 关键修复：从数据库 metadata 恢复完整的执行日志信息
    // 包括: sources, reasoning_text, tool_calls, task_plan
    if (message.hasReplyMessageId() && finalMetadata.isNotEmpty && finalMetadata != '[]') {
      try {
        final sourcesManager = SourcesManager();
        final reasoningManager = ReasoningManager();
        final questionId = message.replyMessageId.toString();
        
        // 从 metadata 解析
        final metadataJson = jsonDecode(finalMetadata);
        
        // 🔹 情况1: metadata 是 Map（完整的结构化数据）
        if (metadataJson is Map<String, dynamic>) {
          // 恢复 sources
          if (metadataJson.containsKey('sources') && metadataJson['sources'] is List) {
            final sourcesData = metadataJson['sources'] as List;
            final sources = sourcesData
                .where((json) => json != null && json is Map)
                .map((json) => ChatMessageRefSource.fromJson(json as Map<String, dynamic>))
                .toList();
            
            if (sources.isNotEmpty) {
              sourcesManager.setSources(chatId, questionId, sources);
              Log.info(
                "📥 [RESTORE] 从数据库恢复 ${sources.length} 个 sources"
                " (chatId: $chatId, questionId: $questionId)"
              );
            }
          }
          
          // 🔹 恢复 reasoning_text（推理文本）
          if (metadataJson.containsKey('reasoning_text') && metadataJson['reasoning_text'] is String) {
            final reasoningText = metadataJson['reasoning_text'] as String;
            if (reasoningText.isNotEmpty) {
              reasoningManager.setReasoningText(chatId, reasoningText);
              reasoningManager.setReasoningComplete(chatId, true);
              Log.info(
                "📥 [RESTORE] 从数据库恢复推理文本"
                " (长度: ${reasoningText.length})"
              );
            }
          }
          
          // 🔹 恢复 tool_calls（工具调用信息）
          // 注意：tool_calls 和 task_plan 的恢复需要通过 ChatAIMessageBloc 的 state
          // 因为它们不是全局管理器管理的，而是消息级别的状态
          // 这部分会在 chat_ai_message_bloc 初始化时自动从 rawMetadata 恢复
          
          Log.info("📥 [RESTORE] 元数据恢复完成 (chatId: $chatId, questionId: $questionId)");
        } 
        // 🔹 情况2: metadata 是 List（旧格式，仅 sources）
        else if (metadataJson is List) {
          final sources = metadataJson
              .where((json) => json != null && json is Map)
              .map((json) => ChatMessageRefSource.fromJson(json as Map<String, dynamic>))
              .toList();
          
          if (sources.isNotEmpty) {
            sourcesManager.setSources(chatId, questionId, sources);
            Log.info(
              "📥 [RESTORE] 从数据库恢复 ${sources.length} 个 sources（旧格式）"
              " (chatId: $chatId, questionId: $questionId)"
            );
          }
        }
      } catch (e) {
        Log.warn("⚠️ [RESTORE] 解析 metadata 失败: $e");
      }
    }
    
    // 尝试从 SourcesManager 获取 sources（用于没有保存到数据库的情况）
    bool shouldRestoreFromGlobal = false;
    String? questionIdForRestore;
    
    if (message.hasReplyMessageId()) {
      questionIdForRestore = message.replyMessageId.toString();
      shouldRestoreFromGlobal = true;
    } else if (message.metadata.isNotEmpty && message.metadata != 'null') {
      final sourcesManager = SourcesManager();
      final recentQuestionId = sourcesManager.getMostRecentQuestionId(chatId);
      if (recentQuestionId != null) {
        questionIdForRestore = recentQuestionId;
        shouldRestoreFromGlobal = true;
      }
    }
    
    if (shouldRestoreFromGlobal && questionIdForRestore != null) {
      final sourcesManager = SourcesManager();
      final globalSources = sourcesManager.getSources(chatId, questionIdForRestore);
      
      if (globalSources != null && globalSources.isNotEmpty) {
        final sourcesJson = globalSources.map((s) => s.toJson()).toList();
        finalMetadata = jsonEncode(sourcesJson);
      }
    }

    // ✅ 构建 metadata，包含 question_id（来自 reply_message_id）
    final messageMetadata = <String, dynamic>{
      messageRefSourceJsonStringKey: finalMetadata,
    };
    
    // ✅ 如果有 reply_message_id，将其作为 question_id 添加到 metadata（保持 Int64 类型）
    if (message.hasReplyMessageId()) {
      messageMetadata[messageQuestionIdKey] = message.replyMessageId;
    }

    return TextMessage(
      author: User(id: message.authorId),
      id: messageId,
      text: message.content,
      createdAt: message.createdAt.toDateTime(),
      metadata: messageMetadata,
    );
  }

  /// Create a streaming answer message
  Message createAnswerStreamMessage({
    required AnswerStream stream,
    required Int64 questionMessageId,
    String? fakeQuestionMessageId,
  }) {
    answerStreamMessageId = fakeQuestionMessageId == null
        ? (questionMessageId + 1).toString()
        : "${fakeQuestionMessageId}_ans";

    return TextMessage(
      id: answerStreamMessageId,
      text: '',
      author: User(id: "streamId:${nanoid()}"),
      metadata: {
        "$AnswerStream": stream,
        messageQuestionIdKey: questionMessageId,
        "chatId": chatId,
      },
      createdAt: DateTime.now(),
    );
  }

  /// Create a streaming question message
  Message createQuestionStreamMessage(
    QuestionStream stream,
    Map<String, dynamic>? sentMetadata,
  ) {
    final now = DateTime.now();
    questionStreamMessageId = timestamp().toString();

    return TextMessage(
      author: User(id: userId),
      metadata: {
        "$QuestionStream": stream,
        "chatId": chatId,
        if (sentMetadata != null)
          messageChatFileListKey: sentMetadata[messageChatFileListKey],
      },
      id: questionStreamMessageId,
      createdAt: now,
      text: stream.text, // 使用QuestionStream的文本内容
    );
  }

  /// Clear error messages from the chat
  void clearErrorMessages() {
    final errorMessages = chatController.messages
        .where(
          (message) =>
              onetimeMessageTypeFromMeta(message.metadata) ==
              OnetimeShotType.error,
        )
        .toList();

    for (final message in errorMessages) {
      chatController.remove(message);
    }
  }

  /// Clear related questions from the chat
  void clearRelatedQuestions() {
    final relatedQuestionMessages = chatController.messages
        .where(
          (message) =>
              onetimeMessageTypeFromMeta(message.metadata) ==
              OnetimeShotType.relatedQuestion,
        )
        .toList();

    for (final message in relatedQuestionMessages) {
      chatController.remove(message);
    }
  }

  /// Checks if a message is a one-time message
  bool isOneTimeMessage(Message message) {
    return message.metadata != null &&
        message.metadata!.containsKey(onetimeShotType);
  }

  /// Get the oldest message that is not a one-time message
  Message? getOldestMessage() {
    return chatController.messages
        .firstWhereOrNull((message) => !isOneTimeMessage(message));
  }

  /// Add a message to the temporary ID map when receiving from server
  void processReceivedMessage(ChatMessagePB pb) {
    // 3 means message response from AI
    if (pb.authorType == 3 && answerStreamMessageId.isNotEmpty) {
      _temporaryMessageIDMap.putIfAbsent(
        pb.messageId.toString(),
        () => answerStreamMessageId,
      );
      answerStreamMessageId = '';
    }

    // 1 means message response from User
    if (pb.authorType == 1 && questionStreamMessageId.isNotEmpty) {
      _temporaryMessageIDMap.putIfAbsent(
        pb.messageId.toString(),
        () => questionStreamMessageId,
      );
      questionStreamMessageId = '';
    }
  }
}
