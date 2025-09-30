import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';

import '../execution_log_viewer.dart';
import '../../application/chat_bloc.dart';

/// AI消息组件，集成了执行日志查看器
/// 
/// 在AI回复消息下方显示一个可展开的执行日志查看器，
/// 让用户能够查看智能体的执行过程
class AIMessageWithExecutionLogs extends StatefulWidget {
  const AIMessageWithExecutionLogs({
    super.key,
    required this.message,
    required this.sessionId,
    this.enableAnimation = true,
  });

  final TextMessage message;
  final String sessionId;
  final bool enableAnimation;

  @override
  State<AIMessageWithExecutionLogs> createState() => _AIMessageWithExecutionLogsState();
}

class _AIMessageWithExecutionLogsState extends State<AIMessageWithExecutionLogs> {
  bool _showExecutionLogs = false;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 原始的AI消息（这里应该使用实际的AI消息组件）
        Container(
          padding: const EdgeInsets.all(12),
          margin: const EdgeInsets.only(left: 60),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerLow,
            borderRadius: BorderRadius.circular(8),
          ),
          child: FlowyText.regular(
            widget.message.text,
            fontSize: 14,
            maxLines: null,
          ),
        ),
        
        // 执行日志展开按钮
        if (_shouldShowExecutionLogButton()) ...[
          const VSpace(8),
          _buildExecutionLogToggle(),
        ],
        
        // 执行日志查看器
        if (_showExecutionLogs) ...[
          const VSpace(8),
          _buildExecutionLogViewer(),
        ],
      ],
    );
  }

  /// 判断是否应该显示执行日志按钮
  bool _shouldShowExecutionLogButton() {
    // 只有AI消息且不是错误消息才显示
    return widget.message.author.id != 'user' && 
           !widget.message.text.isEmpty;
  }

  /// 构建执行日志展开/收起按钮
  Widget _buildExecutionLogToggle() {
    return Container(
      margin: const EdgeInsets.only(left: 60), // 与AI消息对齐
      child: FlowyButton(
        text: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              _showExecutionLogs ? Icons.expand_less : Icons.expand_more,
              size: 16,
              color: Theme.of(context).hintColor,
            ),
            const HSpace(4),
            FlowyText.regular(
              _showExecutionLogs ? '隐藏执行日志' : '查看执行过程',
              fontSize: 12,
              color: Theme.of(context).hintColor,
            ),
          ],
        ),
        onTap: () {
          setState(() {
            _showExecutionLogs = !_showExecutionLogs;
          });
        },
      ),
    );
  }

  /// 构建执行日志查看器
  Widget _buildExecutionLogViewer() {
    return Container(
      margin: const EdgeInsets.only(left: 60), // 与AI消息对齐
      child: ExecutionLogViewer(
        sessionId: widget.sessionId,
        messageId: widget.message.id,
        height: 300,
        showHeader: true,
      ),
    );
  }
}

/// 智能体消息气泡组件
/// 
/// 在消息气泡右上角添加一个执行日志图标，点击后显示执行日志
class SmartMessageBubble extends StatefulWidget {
  const SmartMessageBubble({
    super.key,
    required this.message,
    required this.sessionId,
    required this.child,
  });

  final TextMessage message;
  final String sessionId;
  final Widget child;

  @override
  State<SmartMessageBubble> createState() => _SmartMessageBubbleState();
}

class _SmartMessageBubbleState extends State<SmartMessageBubble> {
  bool _showExecutionLogs = false;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        widget.child,
        
        // 执行日志图标按钮
        if (_shouldShowExecutionLogIcon())
          Positioned(
            top: 8,
            right: 8,
            child: _buildExecutionLogIcon(),
          ),
        
        // 执行日志弹出层
        if (_showExecutionLogs)
          Positioned(
            top: 40,
            right: 8,
            child: _buildExecutionLogPopup(),
          ),
      ],
    );
  }

  bool _shouldShowExecutionLogIcon() {
    return widget.message.author.id != 'user';
  }

  Widget _buildExecutionLogIcon() {
    return Container(
      width: 24,
      height: 24,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: Theme.of(context).dividerColor,
          width: 1,
        ),
      ),
      child: FlowyIconButton(
        icon: const Icon(Icons.timeline, size: 14),
        onPressed: () {
          setState(() {
            _showExecutionLogs = !_showExecutionLogs;
          });
        },
      ),
    );
  }

  Widget _buildExecutionLogPopup() {
    return Container(
      width: 400,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 8,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // 标题栏
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Row(
              children: [
                FlowyText.medium('执行过程', fontSize: 14),
                const Spacer(),
                FlowyIconButton(
                  icon: const Icon(Icons.close, size: 16),
                  onPressed: () {
                    setState(() {
                      _showExecutionLogs = false;
                    });
                  },
                ),
              ],
            ),
          ),
          
          // 执行日志查看器
          ExecutionLogViewer(
            sessionId: widget.sessionId,
            messageId: widget.message.id,
            height: 250,
            showHeader: false,
          ),
        ],
      ),
    );
  }
}

/// 聊天界面底部面板中的执行日志查看器
/// 
/// 在聊天界面底部添加一个可切换的面板，显示当前会话的所有执行日志
class ChatExecutionLogPanel extends StatelessWidget {
  const ChatExecutionLogPanel({
    super.key,
    required this.sessionId,
    required this.isVisible,
    required this.onToggle,
  });

  final String sessionId;
  final bool isVisible;
  final VoidCallback onToggle;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 切换按钮
        Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            border: Border(
              top: BorderSide(
                color: Theme.of(context).dividerColor,
                width: 1,
              ),
            ),
          ),
          child: FlowyButton(
            text: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  isVisible ? Icons.expand_more : Icons.expand_less,
                  size: 16,
                ),
                const HSpace(8),
                FlowyText.regular(
                  isVisible ? '隐藏执行日志' : '显示执行日志',
                  fontSize: 14,
                ),
              ],
            ),
            onTap: onToggle,
          ),
        ),
        
        // 执行日志查看器
        if (isVisible)
          ExecutionLogViewer(
            sessionId: sessionId,
            height: 300,
            showHeader: true,
          ),
      ],
    );
  }
}

/// 使用示例：在聊天页面中集成执行日志
class ChatPageWithExecutionLogs extends StatefulWidget {
  const ChatPageWithExecutionLogs({
    super.key,
    required this.sessionId,
  });

  final String sessionId;

  @override
  State<ChatPageWithExecutionLogs> createState() => _ChatPageWithExecutionLogsState();
}

class _ChatPageWithExecutionLogsState extends State<ChatPageWithExecutionLogs> {
  bool _showExecutionLogPanel = false;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<ChatBloc, ChatState>(
      builder: (context, state) {
        return Column(
          children: [
            // 聊天消息列表（示例实现）
            Expanded(
              child: Container(
                padding: const EdgeInsets.all(16),
                child: FlowyText.regular(
                  '这里是聊天消息列表的示例实现\n'
                  '在实际集成中，需要根据具体的ChatState结构来实现\n'
                  '会话ID: ${widget.sessionId}',
                  fontSize: 14,
                  maxLines: null,
                ),
              ),
            ),
            
            // 执行日志面板
            ChatExecutionLogPanel(
              sessionId: widget.sessionId,
              isVisible: _showExecutionLogPanel,
              onToggle: () {
                setState(() {
                  _showExecutionLogPanel = !_showExecutionLogPanel;
                });
              },
            ),
          ],
        );
      },
    );
  }
}
