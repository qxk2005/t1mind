import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';

import '../application/execution_log_bloc.dart';
import 'execution_log_viewer.dart';

/// 执行日志测试页面
/// 
/// 用于测试执行日志查看器的功能
class ExecutionLogTestPage extends StatefulWidget {
  const ExecutionLogTestPage({super.key});

  @override
  State<ExecutionLogTestPage> createState() => _ExecutionLogTestPageState();
}

class _ExecutionLogTestPageState extends State<ExecutionLogTestPage> {
  late final ExecutionLogBloc _bloc;
  final String _testSessionId = 'test_session_${DateTime.now().millisecondsSinceEpoch}';
  final String _testMessageId = 'test_message_${DateTime.now().millisecondsSinceEpoch}';

  @override
  void initState() {
    super.initState();
    _bloc = ExecutionLogBloc(
      sessionId: _testSessionId,
      messageId: _testMessageId,
    );
  }

  @override
  void dispose() {
    _bloc.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('执行日志测试'),
        backgroundColor: Theme.of(context).colorScheme.surfaceContainerHighest,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 测试信息
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerLow,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: Theme.of(context).dividerColor,
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  FlowyText.medium('测试信息', fontSize: 14),
                  const VSpace(8),
                  FlowyText.regular('会话ID: $_testSessionId', fontSize: 12),
                  const VSpace(4),
                  FlowyText.regular('消息ID: $_testMessageId', fontSize: 12),
                ],
              ),
            ),
            
            const VSpace(16),
            
            // 控制按钮
            Row(
              children: [
                FlowyButton(
                  text: FlowyText.regular('加载日志'),
                  onTap: () {
                    _bloc.add(const ExecutionLogEvent.loadLogs());
                  },
                ),
                const HSpace(8),
                FlowyButton(
                  text: FlowyText.regular('刷新日志'),
                  onTap: () {
                    _bloc.add(const ExecutionLogEvent.refreshLogs());
                  },
                ),
                const HSpace(8),
                FlowyButton(
                  text: FlowyText.regular('切换自动滚动'),
                  onTap: () {
                    _bloc.add(const ExecutionLogEvent.toggleAutoScroll(true));
                  },
                ),
              ],
            ),
            
            const VSpace(16),
            
            // 执行日志查看器
            Expanded(
              child: BlocProvider.value(
                value: _bloc,
                child: ExecutionLogViewer(
                  sessionId: _testSessionId,
                  messageId: _testMessageId,
                  height: double.infinity,
                  showHeader: true,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// 在现有应用中添加测试页面的扩展方法
extension ExecutionLogTestExtension on Widget {
  /// 添加执行日志测试按钮
  Widget withExecutionLogTest(BuildContext context) {
    return Stack(
      children: [
        this,
        Positioned(
          top: 50,
          right: 16,
          child: FlowyButton(
            text: FlowyText.regular('测试执行日志', fontSize: 12),
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (context) => const ExecutionLogTestPage(),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}
