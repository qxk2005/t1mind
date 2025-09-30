import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';

import 'execution_log_viewer.dart';
import '../application/chat_bloc.dart';

/// 执行日志集成示例
/// 
/// 展示如何在聊天界面中集成执行日志查看器
class ExecutionLogIntegrationExample extends StatefulWidget {
  const ExecutionLogIntegrationExample({
    super.key,
    required this.sessionId,
    this.messageId,
  });

  final String sessionId;
  final String? messageId;

  @override
  State<ExecutionLogIntegrationExample> createState() => _ExecutionLogIntegrationExampleState();
}

class _ExecutionLogIntegrationExampleState extends State<ExecutionLogIntegrationExample> {
  bool _showExecutionLogs = false;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 聊天消息区域
        Expanded(
          flex: _showExecutionLogs ? 2 : 3,
          child: Container(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              border: Border.all(
                color: Theme.of(context).dividerColor,
                width: 1,
              ),
              borderRadius: BorderRadius.circular(8),
            ),
            child: const Center(
              child: FlowyText.medium(
                '聊天消息区域',
                fontSize: 16,
              ),
            ),
          ),
        ),
        
        // 执行日志切换按钮
        Container(
          padding: const EdgeInsets.symmetric(vertical: 8),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              FlowyButton(
                text: FlowyText.regular(
                  _showExecutionLogs ? '隐藏执行日志' : '显示执行日志',
                  fontSize: 12,
                ),
                leftIcon: Icon(
                  _showExecutionLogs ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                ),
                onTap: () {
                  setState(() {
                    _showExecutionLogs = !_showExecutionLogs;
                  });
                },
              ),
            ],
          ),
        ),
        
        // 执行日志查看器
        if (_showExecutionLogs)
          Expanded(
            flex: 1,
            child: ExecutionLogViewer(
              sessionId: widget.sessionId,
              messageId: widget.messageId,
              height: 300,
              showHeader: true,
            ),
          ),
      ],
    );
  }
}

/// 在聊天页面中集成执行日志的扩展方法
extension ChatPageExecutionLogExtension on Widget {
  /// 为聊天页面添加执行日志功能
  Widget withExecutionLog({
    required String sessionId,
    String? messageId,
  }) {
    return Builder(
      builder: (context) {
        return Column(
          children: [
            // 原始聊天界面
            Expanded(child: this),
            
            // 执行日志查看器（可折叠）
            // 执行日志查看器（可根据需要显示）
            ExecutionLogViewer(
              sessionId: sessionId,
              messageId: messageId,
              height: 200,
              showHeader: false,
            ),
          ],
        );
      },
    );
  }
}

/// 智能体消息气泡中的执行日志按钮
class AgentMessageExecutionLogButton extends StatelessWidget {
  const AgentMessageExecutionLogButton({
    super.key,
    required this.sessionId,
    required this.messageId,
    this.onPressed,
  });

  final String sessionId;
  final String messageId;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    return FlowyIconButton(
      icon: const Icon(
        Icons.timeline,
        size: 16,
      ),
      tooltipText: '查看执行日志',
      onPressed: onPressed ?? () => _showExecutionLogDialog(context),
    );
  }

  void _showExecutionLogDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('执行日志'),
        content: SizedBox(
          width: 800,
          height: 600,
          child: ExecutionLogViewer(
            sessionId: sessionId,
            messageId: messageId,
            height: 600,
            showHeader: false,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('关闭'),
          ),
        ],
      ),
    );
  }
}

/// 执行日志底部面板
class ExecutionLogBottomPanel extends StatefulWidget {
  const ExecutionLogBottomPanel({
    super.key,
    required this.sessionId,
    this.messageId,
    this.initialHeight = 250,
  });

  final String sessionId;
  final String? messageId;
  final double initialHeight;

  @override
  State<ExecutionLogBottomPanel> createState() => _ExecutionLogBottomPanelState();
}

class _ExecutionLogBottomPanelState extends State<ExecutionLogBottomPanel> {
  bool _isExpanded = false;
  double _height = 250;

  @override
  void initState() {
    super.initState();
    _height = widget.initialHeight;
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      height: _isExpanded ? _height : 40,
      child: Column(
        children: [
          // 拖拽手柄和标题栏
          GestureDetector(
            onTap: () {
              setState(() {
                _isExpanded = !_isExpanded;
              });
            },
            child: Container(
              height: 40,
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: const BorderRadius.only(
                  topLeft: Radius.circular(8),
                  topRight: Radius.circular(8),
                ),
              ),
              child: Row(
                children: [
                  const HSpace(12),
                  const Icon(Icons.timeline, size: 16),
                  const HSpace(8),
                  const FlowyText.medium('执行日志', fontSize: 12),
                  const Spacer(),
                  Icon(
                    _isExpanded ? Icons.expand_more : Icons.expand_less,
                    size: 16,
                  ),
                  const HSpace(12),
                ],
              ),
            ),
          ),
          
          // 执行日志内容
          if (_isExpanded)
            Expanded(
              child: ExecutionLogViewer(
                sessionId: widget.sessionId,
                messageId: widget.messageId,
                height: _height - 40,
                showHeader: false,
              ),
            ),
        ],
      ),
    );
  }
}
