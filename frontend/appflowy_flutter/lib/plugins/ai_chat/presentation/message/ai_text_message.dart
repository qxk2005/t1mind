import 'package:appflowy/ai/ai.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_ai_message_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_message_height_manager.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_message_stream.dart';
import 'package:appflowy/plugins/ai_chat/presentation/widgets/message_height_calculator.dart';
import 'package:appflowy/plugins/ai_chat/presentation/message/tool_call_display.dart';
import 'package:appflowy/plugins/ai_chat/presentation/message/task_plan_display.dart';
import 'package:appflowy/plugins/ai_chat/widgets/unified_reference_display.dart';
import 'package:string_validator/string_validator.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';

import '../layout_define.dart';
import 'ai_markdown_text.dart';
import 'ai_message_bubble.dart';
import 'error_text_message.dart';

/// [ChatAIMessageWidget] includes both the text of the AI response as well as
/// the avatar, decorations and hover effects that are also rendered. This is
/// different from [ChatUserMessageWidget] which only contains the message and
/// has to be separately wrapped with a bubble since the hover effects need to
/// know the current streaming status of the message.
class ChatAIMessageWidget extends StatelessWidget {
  const ChatAIMessageWidget({
    super.key,
    required this.user,
    required this.messageUserId,
    required this.message,
    required this.stream,
    required this.questionId,
    required this.chatId,
    required this.refSourceJsonString,
    required this.onStopStream,
    this.onSelectedMetadata,
    this.onRegenerate,
    this.onChangeFormat,
    this.onChangeModel,
    this.isLastMessage = false,
    this.isStreaming = false,
    this.isSelectingMessages = false,
    this.enableAnimation = true,
    this.hasRelatedQuestions = false,
  });

  final User user;
  final String messageUserId;

  final Message message;
  final AnswerStream? stream;
  final Int64? questionId;
  final String chatId;
  final String? refSourceJsonString;
  final void Function(ChatMessageRefSource metadata)? onSelectedMetadata;
  final void Function()? onRegenerate;
  final void Function() onStopStream;
  final void Function(PredefinedFormat)? onChangeFormat;
  final void Function(AIModelPB)? onChangeModel;
  final bool isStreaming;
  final bool isLastMessage;
  final bool isSelectingMessages;
  final bool enableAnimation;
  final bool hasRelatedQuestions;

  @override
  Widget build(BuildContext context) {
    // Log.debug("🏗️ [WIDGET] ChatAIMessageWidget building - message id: ${message.id}");
    return BlocProvider(
      key: ValueKey('chat_ai_message_${message.id}'), // 添加key防止重新创建
      create: (context) {
        // Log.debug("🏗️ [BLOC] Creating new ChatAIMessageBloc - message id: ${message.id}");
        return ChatAIMessageBloc(
          message: stream ?? (message as TextMessage).text,
          refSourceJsonString: refSourceJsonString,
          chatId: chatId,
          questionId: questionId,
        );
      },
      child: BlocConsumer<ChatAIMessageBloc, ChatAIMessageState>(
        listenWhen: (previous, current) =>
            previous.messageState != current.messageState,
        listener: (context, state) => _handleMessageState(state, context),
        buildWhen: (previous, current) {
          // 重建条件：messageState变化 OR reasoningText变化 OR isReasoningComplete变化
          final shouldRebuild = previous.messageState != current.messageState ||
              previous.reasoningText != current.reasoningText ||
              previous.isReasoningComplete != current.isReasoningComplete ||
              previous.text != current.text ||
              previous.sources != current.sources;
          
          // if (shouldRebuild) {
          //   Log.debug("🏗️ [UI] BlocConsumer triggering rebuild - reasoningText: ${current.reasoningText?.length ?? 0}, isReasoningComplete: ${current.isReasoningComplete}");
          // }
          
          return shouldRebuild;
        },
        builder: (context, blocState) {
          final loadingText = blocState.progress?.step ??
              LocaleKeys.chat_generatingResponse.tr();

          // Calculate minimum height only for the last AI answer message
          double minHeight = 0;
          if (isLastMessage && !hasRelatedQuestions) {
            final screenHeight = MediaQuery.of(context).size.height;
            minHeight = ChatMessageHeightManager().calculateMinHeight(
              messageId: message.id,
              screenHeight: screenHeight,
            );
          }

          return Container(
            alignment: Alignment.topLeft,
            constraints: BoxConstraints(
              minHeight: minHeight,
            ),
            padding: AIChatUILayout.messageMargin,
            child: MessageHeightCalculator(
              messageId: message.id,
              onHeightMeasured: (messageId, height) {
                ChatMessageHeightManager().cacheWithoutMinHeight(
                  messageId: messageId,
                  height: height,
                );
              },
              child: blocState.messageState.when(
                loading: () => ChatAIMessageBubble(
                  message: message,
                  showActions: false,
                  child: Padding(
                    padding: const EdgeInsets.only(top: 8.0),
                    child: AILoadingIndicator(text: loadingText),
                  ),
                ),
                ready: () {
                  // 如果有推理文本，即使主文本为空也显示_NonEmptyMessage
                  final hasReasoningText = blocState.reasoningText != null && blocState.reasoningText!.isNotEmpty;
                  
                  return (blocState.text.isEmpty && !hasReasoningText)
                      ? _LoadingMessage(
                          message: message,
                          loadingText: loadingText,
                        )
                      : _NonEmptyMessage(
                          user: user,
                          messageUserId: messageUserId,
                          message: message,
                          stream: stream,
                          questionId: questionId,
                          chatId: chatId,
                          refSourceJsonString: refSourceJsonString,
                          onStopStream: onStopStream,
                          onSelectedMetadata: onSelectedMetadata,
                          onRegenerate: onRegenerate,
                          onChangeFormat: onChangeFormat,
                          onChangeModel: onChangeModel,
                          isLastMessage: isLastMessage,
                          isStreaming: isStreaming,
                          isSelectingMessages: isSelectingMessages,
                          enableAnimation: enableAnimation,
                        );
                },
                onError: (error) {
                  return ChatErrorMessageWidget(
                    errorMessage: LocaleKeys.chat_aiServerUnavailable.tr(),
                  );
                },
                onAIResponseLimit: () {
                  return ChatErrorMessageWidget(
                    errorMessage:
                        LocaleKeys.sideBar_askOwnerToUpgradeToAIMax.tr(),
                  );
                },
                onAIImageResponseLimit: () {
                  return ChatErrorMessageWidget(
                    errorMessage: LocaleKeys.sideBar_purchaseAIMax.tr(),
                  );
                },
                onAIMaxRequired: (message) {
                  return ChatErrorMessageWidget(
                    errorMessage: message,
                  );
                },
                onInitializingLocalAI: () {
                  onStopStream();

                  return ChatErrorMessageWidget(
                    errorMessage: LocaleKeys
                        .settings_aiPage_keys_localAIInitializing
                        .tr(),
                  );
                },
                aiFollowUp: (followUpData) {
                  return const SizedBox.shrink();
                },
              ),
            ),
          );
        },
      ),
    );
  }

  void _handleMessageState(ChatAIMessageState state, BuildContext context) {
    if (state.stream?.error?.isEmpty != false) {
      state.messageState.maybeMap(
        aiFollowUp: (messageState) {
          context
              .read<ChatBloc>()
              .add(ChatEvent.onAIFollowUp(messageState.followUpData));
        },
        orElse: () {
          // do nothing
        },
      );

      return;
    }
    context.read<ChatBloc>().add(ChatEvent.deleteMessage(message));
  }
}

class _LoadingMessage extends StatelessWidget {
  const _LoadingMessage({
    required this.message,
    required this.loadingText,
  });

  final Message message;
  final String loadingText;

  @override
  Widget build(BuildContext context) {
    return ChatAIMessageBubble(
      message: message,
      showActions: false,
      child: Padding(
        padding: EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
        child: AILoadingIndicator(text: loadingText),
      ),
    );
  }
}

class _NonEmptyMessage extends StatelessWidget {
  const _NonEmptyMessage({
    required this.user,
    required this.messageUserId,
    required this.message,
    required this.stream,
    required this.questionId,
    required this.chatId,
    required this.refSourceJsonString,
    required this.onStopStream,
    this.onSelectedMetadata,
    this.onRegenerate,
    this.onChangeFormat,
    this.onChangeModel,
    this.isLastMessage = false,
    this.isStreaming = false,
    this.isSelectingMessages = false,
    this.enableAnimation = true,
  });

  final User user;
  final String messageUserId;

  final Message message;
  final AnswerStream? stream;
  final Int64? questionId;
  final String chatId;
  final String? refSourceJsonString;
  final ValueChanged<ChatMessageRefSource>? onSelectedMetadata;
  final VoidCallback? onRegenerate;
  final VoidCallback onStopStream;
  final ValueChanged<PredefinedFormat>? onChangeFormat;
  final ValueChanged<AIModelPB>? onChangeModel;
  final bool isStreaming;
  final bool isLastMessage;
  final bool isSelectingMessages;
  final bool enableAnimation;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<ChatAIMessageBloc, ChatAIMessageState>(
      builder: (context, state) {
        // Log.debug("🏗️ [UI] Building _NonEmptyMessage widget - reasoningText length: ${state.reasoningText?.length ?? 0}, isReasoningComplete: ${state.isReasoningComplete}, text length: ${state.text.length}");
        final showActions = stream == null && state.text.isNotEmpty && !isStreaming;
        return ChatAIMessageBubble(
          message: message,
          isLastMessage: isLastMessage,
          showActions: showActions,
          isSelectingMessages: isSelectingMessages,
          onRegenerate: onRegenerate,
          onChangeFormat: onChangeFormat,
          onChangeModel: onChangeModel,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // 推理过程显示：正在推理时显示实时内容，推理完成后显示可折叠的摘要
              if (state.reasoningText != null && state.reasoningText!.isNotEmpty)
                Padding(
                  padding: EdgeInsetsDirectional.only(start: 4.0, bottom: 8.0),
                  child: _AIReasoningDisplay(
                    reasoningText: state.reasoningText!,
                    isReasoningComplete: state.isReasoningComplete,
                    isStreaming: isStreaming,
                  ),
                ),
              
              // 🔧 任务计划显示（在消息内容之前）
              if (state.taskPlan != null)
                Padding(
                  padding: EdgeInsetsDirectional.only(start: 4.0, bottom: 8.0),
                  child: TaskPlanDisplay(plan: state.taskPlan!),
                ),
              
              // 🔧 工具调用显示（在消息内容之前）
              if (state.toolCalls.isNotEmpty)
                Padding(
                  padding: EdgeInsetsDirectional.only(start: 4.0, bottom: 8.0),
                  child: ToolCallDisplay(toolCalls: state.toolCalls),
                ),
              
              Padding(
                padding: EdgeInsetsDirectional.only(start: 4.0),
                child: AIMarkdownText(
                  markdown: state.text,
                  withAnimation: enableAnimation && stream != null,
                ),
              ),
              
              // 网络搜索引用显示（统一风格）
              if (_hasWebSearchCitations(state.sources))
                Padding(
                  padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
                  child: UnifiedReferenceDisplay(
                    references: state.sources
                        .where((source) => source.source == 'web' && isURL(source.id))
                        .toList(),
                    referenceType: ReferenceType.webSearch,
                    maxVisibleReferences: 3,
                    showExpandButton: true,
                    onReferenceSelected: onSelectedMetadata,
                  ),
                ),
              
              // MCP工具引用显示（统一风格）
              if (_hasMCPReferences(state.sources))
                Padding(
                  padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
                  child: UnifiedReferenceDisplay(
                    references: _extractMCPReferences(state.sources),
                    referenceType: ReferenceType.mcpTool,
                    maxVisibleReferences: 3,
                    showExpandButton: true,
                    onReferenceSelected: onSelectedMetadata,
                  ),
                ),
              
              // 文档引用显示（统一风格）
              if (_hasDocumentSources(state.sources))
                Padding(
                  padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
                  child: UnifiedReferenceDisplay(
                    references: _extractDocumentSources(state.sources),
                    referenceType: ReferenceType.document,
                    maxVisibleReferences: 3,
                    showExpandButton: true,
                    onReferenceSelected: onSelectedMetadata,
                  ),
                ),
              
              if (state.sources.isNotEmpty && !isLastMessage) const VSpace(8.0),
            ],
          ),
        );
      },
    );
  }

  /// 检查是否有网络搜索引用
  bool _hasWebSearchCitations(List<ChatMessageRefSource> sources) {
    final hasWeb = sources.any((source) => source.source == 'web' && isURL(source.id));
    if (hasWeb) {
      final webCount = sources.where((source) => source.source == 'web' && isURL(source.id)).length;
    }
    return hasWeb;
  }

  /// 检查是否有MCP工具引用
  bool _hasMCPReferences(List<ChatMessageRefSource> sources) {
    final hasMCP = sources.any((source) => source.source.startsWith('mcp'));
    if (hasMCP) {
      final mcpSources = sources.where((source) => source.source.startsWith('mcp')).toList();
    } else {
      final allSources = sources.map((s) => '${s.source}:${s.name}').join(', ');
    }
    return hasMCP;
  }

  /// 检查是否有文档来源
  bool _hasDocumentSources(List<ChatMessageRefSource> sources) {
    return sources.any((source) => source.source == 'appflowy' || 
        (source.source == 'web' && !isURL(source.id)));
  }

  /// 提取MCP工具引用
  List<ChatMessageRefSource> _extractMCPReferences(List<ChatMessageRefSource> sources) {
    return sources
        .where((source) => source.source.startsWith('mcp'))
        .toList();
  }

  /// 提取文档来源
  List<ChatMessageRefSource> _extractDocumentSources(List<ChatMessageRefSource> sources) {
    return sources
        .where((source) => source.source == 'appflowy' || 
            (source.source == 'web' && !isURL(source.id)))
        .toList();
  }
}

class _AIReasoningDisplay extends StatefulWidget {
  const _AIReasoningDisplay({
    required this.reasoningText,
    required this.isReasoningComplete,
    required this.isStreaming,
  });

  final String reasoningText;
  final bool isReasoningComplete;
  final bool isStreaming;

  @override
  State<_AIReasoningDisplay> createState() => _AIReasoningDisplayState();
}

class _AIReasoningDisplayState extends State<_AIReasoningDisplay>
    with SingleTickerProviderStateMixin {
  bool _isExpanded = false; // 推理完成后默认折叠
  final ScrollController _scrollController = ScrollController();
  late AnimationController _animationController;
  late Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();
    // 如果正在推理，默认展开；如果推理完成，默认折叠
    _isExpanded = !widget.isReasoningComplete;
    
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 1000),
      vsync: this,
    );
    _pulseAnimation = Tween<double>(
      begin: 0.4,
      end: 1.0,
    ).animate(CurvedAnimation(
      parent: _animationController,
      curve: Curves.easeInOut,
    ));
    
    // 只有在推理进行中时才显示脉冲动画
    if (!widget.isReasoningComplete) {
      _animationController.repeat(reverse: true);
    }
  }

  @override
  void didUpdateWidget(_AIReasoningDisplay oldWidget) {
    super.didUpdateWidget(oldWidget);
    
    // Log.debug("🔄 [UI] Widget updated - isReasoningComplete: ${widget.isReasoningComplete}, textLength: ${widget.reasoningText.length}");
    
    // 推理状态改变时的处理
    if (widget.isReasoningComplete != oldWidget.isReasoningComplete) {
      // Log.debug("🔄 [UI] Reasoning state changed: ${oldWidget.isReasoningComplete} -> ${widget.isReasoningComplete}");
      if (widget.isReasoningComplete) {
        // 推理完成，停止动画并自动折叠
        _animationController.stop();
        setState(() {
          _isExpanded = false;
        });
        // Log.debug("🎯 [REALTIME] Reasoning completed, auto-collapsing");
      } else {
        // 开始推理，展开并开始动画
        _animationController.repeat(reverse: true);
        setState(() {
          _isExpanded = true;
        });
        // Log.debug("🚀 [REALTIME] Reasoning started, auto-expanding");
      }
    }
    
    // 推理文本更新时的处理（只在展开状态下滚动）
    if (widget.reasoningText != oldWidget.reasoningText) {
      // Log.debug("🎨 [REALTIME] UI text changed from length ${oldWidget.reasoningText.length} to ${widget.reasoningText.length}, isExpanded: $_isExpanded");
      if (_isExpanded) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (_scrollController.hasClients) {
            _scrollController.animateTo(
              _scrollController.position.maxScrollExtent,
              duration: const Duration(milliseconds: 100),
              curve: Curves.easeOut,
            );
          }
        });
      }
    }
  }

  @override
  void dispose() {
    _animationController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // 推理进行中：显示为聊天消息样式，实时更新
    if (!widget.isReasoningComplete) {
      return Container(
        padding: const EdgeInsets.all(12.0),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.primary.withOpacity(0.1),
          borderRadius: BorderRadius.circular(12.0),
          border: Border.all(
            color: Theme.of(context).colorScheme.primary.withOpacity(0.3),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                AnimatedBuilder(
                  animation: _pulseAnimation,
                  builder: (context, child) {
                    return Container(
                      width: 8,
                      height: 8,
                      decoration: BoxDecoration(
                        color: Theme.of(context).colorScheme.primary.withOpacity(_pulseAnimation.value),
                        shape: BoxShape.circle,
                      ),
                    );
                  },
                ),
                const SizedBox(width: 8),
                Text(
                  '正在思考...',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.primary,
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ],
            ),
            if (widget.reasoningText.isNotEmpty) ...[
              const SizedBox(height: 8),
              Container(
                constraints: const BoxConstraints(maxHeight: 150),
                child: SingleChildScrollView(
                  controller: _scrollController,
                  child: Text(
                    widget.reasoningText,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.onSurface,
                      height: 1.4,
                    ),
                  ),
                ),
              ),
            ],
          ],
        ),
      );
    }
    
    // 推理完成：显示为可折叠的摘要
    return Container(
      padding: const EdgeInsets.all(8.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        borderRadius: BorderRadius.circular(8.0),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          GestureDetector(
            onTap: () => setState(() => _isExpanded = !_isExpanded),
            child: Row(
              children: [
                Icon(
                  _isExpanded ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 4),
                Text(
                  '查看AI推理过程',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const Spacer(),
                Icon(
                  Icons.psychology,
                  size: 14,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ],
            ),
          ),
          if (_isExpanded && widget.reasoningText.isNotEmpty) ...[
            const SizedBox(height: 8),
            Container(
              constraints: const BoxConstraints(maxHeight: 200),
              child: SingleChildScrollView(
                controller: _scrollController,
                child: Text(
                  widget.reasoningText,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                    fontStyle: FontStyle.italic,
                    height: 1.4,
                  ),
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
