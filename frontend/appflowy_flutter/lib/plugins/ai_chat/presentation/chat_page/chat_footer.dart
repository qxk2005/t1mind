import 'package:appflowy/ai/ai.dart';
import 'package:appflowy/plugins/ai_chat/application/ai_chat_prelude.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_input/mobile_chat_input.dart';
import 'package:appflowy/plugins/ai_chat/presentation/layout_define.dart';
import 'package:appflowy/workspace/presentation/home/home_stack.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:universal_platform/universal_platform.dart';

class ChatFooter extends StatefulWidget {
  const ChatFooter({
    super.key,
    required this.view,
    this.selectedAgent,
    this.onAgentExecutionChanged,
  });

  final ViewPB view;
  final AgentConfigPB? selectedAgent;
  final Function(bool isExecuting, String? task, double? progress)? onAgentExecutionChanged;

  @override
  State<ChatFooter> createState() => _ChatFooterState();
}

class _ChatFooterState extends State<ChatFooter> {
  final textController = AiPromptInputTextEditingController();

  @override
  void dispose() {
    textController.dispose();
    super.dispose();
  }

  void _handleChatStateChange(ChatState state) {
    // 监听聊天状态变化，更新智能体执行状态
    if (widget.selectedAgent != null && widget.onAgentExecutionChanged != null) {
      final isExecuting = !state.promptResponseState.isReady;
      
      if (isExecuting) {
        // 根据聊天状态确定当前任务
        String? currentTask;
        if (state.promptResponseState == PromptResponseState.streamingAnswer) {
          currentTask = '正在生成回复...';
        } else if (state.promptResponseState == PromptResponseState.sendingQuestion) {
          currentTask = '正在处理请求...';
        }
        
        widget.onAgentExecutionChanged!(true, currentTask, null);
      } else {
        widget.onAgentExecutionChanged!(false, null, null);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<ChatBloc, ChatState>(
      listener: (context, state) => _handleChatStateChange(state),
      child: BlocSelector<ChatSelectMessageBloc, ChatSelectMessageState, bool>(
        selector: (state) => state.isSelectingMessages,
        builder: (context, isSelectingMessages) {
          return AnimatedSwitcher(
            duration: const Duration(milliseconds: 150),
            transitionBuilder: (child, animation) {
              return NonClippingSizeTransition(
                sizeFactor: animation,
                axisAlignment: -1,
                child: child,
              );
            },
            child: isSelectingMessages
                ? const SizedBox.shrink()
                : Padding(
                    padding: AIChatUILayout.safeAreaInsets(context),
                    child: BlocSelector<ChatBloc, ChatState, bool>(
                      selector: (state) {
                        return state.promptResponseState.isReady;
                      },
                      builder: (context, canSendMessage) {
                        final chatBloc = context.read<ChatBloc>();

                        return UniversalPlatform.isDesktop
                            ? _buildDesktopInput(
                                context,
                                chatBloc,
                                canSendMessage,
                              )
                            : _buildMobileInput(
                                context,
                                chatBloc,
                                canSendMessage,
                              );
                      },
                    ),
                  ),
          );
        },
      ),
    );
  }

  Widget _buildDesktopInput(
    BuildContext context,
    ChatBloc chatBloc,
    bool canSendMessage,
  ) {
    return DesktopPromptInput(
      isStreaming: !canSendMessage,
      textController: textController,
      onStopStreaming: () {
        chatBloc.add(const ChatEvent.stopStream());
      },
      onSubmitted: (text, format, metadata, promptId) {
        // 如果选择了智能体，通知执行状态变化
        if (widget.selectedAgent != null) {
          widget.onAgentExecutionChanged?.call(true, '正在处理请求...', null);
        }
        
        chatBloc.add(
          ChatEvent.sendMessage(
            message: text,
            format: format,
            metadata: metadata,
            promptId: promptId,
          ),
        );
      },
      selectedSourcesNotifier: chatBloc.selectedSourcesNotifier,
      onUpdateSelectedSources: (ids) {
        chatBloc.add(
          ChatEvent.updateSelectedSources(
            selectedSourcesIds: ids,
          ),
        );
      },
    );
  }

  Widget _buildMobileInput(
    BuildContext context,
    ChatBloc chatBloc,
    bool canSendMessage,
  ) {
    return MobileChatInput(
      isStreaming: !canSendMessage,
      onStopStreaming: () {
        chatBloc.add(const ChatEvent.stopStream());
      },
      onSubmitted: (text, format, metadata) {
        // 如果选择了智能体，通知执行状态变化
        if (widget.selectedAgent != null) {
          widget.onAgentExecutionChanged?.call(true, '正在处理请求...', null);
        }
        
        chatBloc.add(
          ChatEvent.sendMessage(
            message: text,
            format: format,
            metadata: metadata,
          ),
        );
      },
      selectedSourcesNotifier: chatBloc.selectedSourcesNotifier,
      onUpdateSelectedSources: (ids) {
        chatBloc.add(
          ChatEvent.updateSelectedSources(
            selectedSourcesIds: ids,
          ),
        );
      },
    );
  }
}
