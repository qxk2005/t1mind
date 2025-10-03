import 'package:appflowy/plugins/ai_chat/application/chat_bloc.dart';
import 'package:appflowy/plugins/ai_chat/presentation/agent_selector.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_message_selector_banner.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/chat_animation_list_widget.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/chat_footer.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/chat_message_widget.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/text_message_widget.dart';
import 'package:appflowy/plugins/ai_chat/presentation/scroll_to_bottom.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_chat_ui/flutter_chat_ui.dart' hide ChatMessage;
import 'package:universal_platform/universal_platform.dart';

class LoadChatMessageStatusReady extends StatefulWidget {
  const LoadChatMessageStatusReady({
    super.key,
    required this.view,
    required this.userProfile,
    required this.chatController,
  });

  final ViewPB view;
  final UserProfilePB userProfile;
  final ChatController chatController;

  @override
  State<LoadChatMessageStatusReady> createState() => _LoadChatMessageStatusReadyState();
}

class _LoadChatMessageStatusReadyState extends State<LoadChatMessageStatusReady> {
  AgentConfigPB? selectedAgent;
  bool isAgentExecuting = false;
  String? currentAgentTask;
  double? executionProgress;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Chat header, banner
        _buildHeader(context),
        // Chat body, a list of messages
        _buildBody(context),
        // Chat footer, a text input field with toolbar, send button, etc.
        _buildFooter(context),
      ],
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Column(
      children: [
        // 原有的消息选择横幅
        ChatMessageSelectorBanner(
          view: widget.view,
          allMessages: widget.chatController.messages,
        ),
        
        // 智能体选择器（包含执行状态）
        _wrapConstraints(
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              children: [
                Expanded(
                  child: AgentSelector(
                    selectedAgent: selectedAgent,
                    onAgentSelected: (agent) {
                      setState(() {
                        selectedAgent = agent;
                      });
                      // 通知聊天BLoC智能体已更改
                      context.read<ChatBloc>().add(
                        ChatEvent.selectAgent(agent?.id),
                      );
                    },
                    showStatus: true,
                    compact: UniversalPlatform.isMobile,
                    // 执行状态 - 直接集成在选择器内
                    isExecuting: isAgentExecuting && selectedAgent != null,
                    executionStatus: isAgentExecuting ? (currentAgentTask ?? '思考中') : null,
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildBody(BuildContext context) {
    final bool enableAnimation = true;
    return Expanded(
      child: Align(
        alignment: Alignment.topCenter,
        child: _wrapConstraints(
          SelectionArea(
            child: ScrollConfiguration(
              behavior: ScrollConfiguration.of(context).copyWith(
                scrollbars: false,
              ),
              child: Chat(
                chatController: widget.chatController,
                user: User(id: widget.userProfile.id.toString()),
                darkTheme: ChatTheme.fromThemeData(Theme.of(context)),
                theme: ChatTheme.fromThemeData(Theme.of(context)),
                builders: Builders(
                  // we have a custom input builder, so we don't need the default one
                  inputBuilder: (_) => const SizedBox.shrink(),
                  textMessageBuilder: (
                    context,
                    message,
                  ) =>
                      TextMessageWidget(
                    message: message,
                    userProfile: widget.userProfile,
                    view: widget.view,
                    enableAnimation: enableAnimation,
                  ),
                  chatMessageBuilder: (
                    context,
                    message,
                    animation,
                    child,
                  ) =>
                      ChatMessage(
                    message: message,
                    padding: const EdgeInsets.symmetric(vertical: 18.0),
                    child: child,
                  ),
                  scrollToBottomBuilder: (
                    context,
                    animation,
                    onPressed,
                  ) =>
                      CustomScrollToBottom(
                    animation: animation,
                    onPressed: onPressed,
                  ),
                  chatAnimatedListBuilder: (
                    context,
                    scrollController,
                    itemBuilder,
                  ) =>
                      ChatAnimationListWidget(
                    userProfile: widget.userProfile,
                    scrollController: scrollController,
                    itemBuilder: itemBuilder,
                    enableReversedList: !enableAnimation,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildFooter(BuildContext context) {
    return _wrapConstraints(
      ChatFooter(
        view: widget.view,
        selectedAgent: selectedAgent,
        onAgentExecutionChanged: (isExecuting, task, progress) {
          setState(() {
            isAgentExecuting = isExecuting;
            currentAgentTask = task;
            executionProgress = progress;
          });
        },
      ),
    );
  }

  Widget _wrapConstraints(Widget child) {
    return Container(
      constraints: const BoxConstraints(maxWidth: 784),
      margin: UniversalPlatform.isDesktop
          ? const EdgeInsets.symmetric(horizontal: 60.0)
          : null,
      child: child,
    );
  }
}
