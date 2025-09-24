import 'package:appflowy/ai/ai.dart';
import 'package:appflowy/plugins/ai_chat/application/ai_chat_prelude.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_input/mobile_chat_input.dart';
import 'package:appflowy/plugins/ai_chat/presentation/layout_define.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/plugins/mcp/chat/mcp_selector.dart';
import 'package:appflowy/workspace/presentation/home/home_stack.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:universal_platform/universal_platform.dart';

class ChatFooter extends StatefulWidget {
  const ChatFooter({
    super.key,
    required this.view,
  });

  final ViewPB view;

  @override
  State<ChatFooter> createState() => _ChatFooterState();
}

class _ChatFooterState extends State<ChatFooter> {
  final textController = AiPromptInputTextEditingController();
  final ValueNotifier<List<String>> _selectedMcpNames = ValueNotifier([]);

  @override
  void dispose() {
    textController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocSelector<ChatSelectMessageBloc, ChatSelectMessageState, bool>(
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
        final m = {...metadata};
        if (_selectedMcpNames.value.isNotEmpty) {
          m[messageSelectedMcpNamesKey] = _selectedMcpNames.value;
        }
        chatBloc.add(
          ChatEvent.sendMessage(
            message: text,
            format: format,
            metadata: m,
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
      leadingExtra: McpSelector(
        onChanged: (names) => _selectedMcpNames.value = names,
      ),
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
        final m = {...metadata};
        if (_selectedMcpNames.value.isNotEmpty) {
          m[messageSelectedMcpNamesKey] = _selectedMcpNames.value;
        }
        chatBloc.add(
          ChatEvent.sendMessage(
            message: text,
            format: format,
            metadata: m,
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
      leadingExtra: McpSelector(
        onChanged: (names) => _selectedMcpNames.value = names,
        iconSize: 18.0,
      ),
    );
  }
}
