import 'dart:async';

import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/conversion_logs_dialog.dart';
import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/import_type.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// 转换进度对话框
/// 
/// 显示文档转换的进度，包括：
/// - 进度条显示
/// - 任务状态显示
/// - 取消和重试功能
/// - 错误处理
class ConversionProgressDialog extends StatefulWidget {
  const ConversionProgressDialog({
    super.key,
    required this.tasks,
    this.onCancel,
    this.onRetry,
    this.onClose,
  });

  final List<ConversionTask> tasks;
  final VoidCallback? onCancel;
  final VoidCallback? onRetry;
  final VoidCallback? onClose;

  @override
  State<ConversionProgressDialog> createState() => _ConversionProgressDialogState();
}

class _ConversionProgressDialogState extends State<ConversionProgressDialog> {
  late Timer _progressTimer;
  bool _isCancelling = false;

  @override
  void initState() {
    super.initState();
    _startProgressTimer();
  }

  @override
  void dispose() {
    _progressTimer.cancel();
    super.dispose();
  }

  void _startProgressTimer() {
    _progressTimer = Timer.periodic(const Duration(milliseconds: 100), (timer) {
      if (mounted) {
        setState(() {});
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      backgroundColor: Theme.of(context).colorScheme.surface,
      title: FlowyText.semibold(
        LocaleKeys.conversionProgress_title.tr(),
        fontSize: 20,
        color: Theme.of(context).colorScheme.tertiary,
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          vertical: 10.0,
          horizontal: 20.0,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildProgressOverview(),
            const VSpace(16),
            _buildTaskList(),
            const VSpace(16),
            _buildActionButtons(),
          ],
        ),
      ),
    );
  }

  Widget _buildProgressOverview() {
    final completedTasks = widget.tasks.where((task) => task.status == ConversionTaskStatus.completed).length;
    final totalTasks = widget.tasks.length;
    final progress = totalTasks > 0 ? completedTasks / totalTasks : 0.0;
    final hasError = widget.tasks.any((task) => task.status == ConversionTaskStatus.error);
    final isCompleted = completedTasks == totalTasks;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              FlowyText.semibold(
                '转换进度',
                fontSize: 14,
              ),
              const Spacer(),
              FlowyText.regular(
                _getStatusText(isCompleted, hasError),
                fontSize: 12,
                color: _getStatusColor(isCompleted, hasError),
              ),
            ],
          ),
          const VSpace(8),
          LinearProgressIndicator(
            value: progress,
            backgroundColor: Theme.of(context).colorScheme.surface,
            minHeight: 8,
            borderRadius: BorderRadius.circular(4),
          ),
          const VSpace(8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              FlowyText.regular(
                '$completedTasks / $totalTasks 个文件',
                fontSize: 12,
                color: Theme.of(context).hintColor,
              ),
              if (!isCompleted && !hasError)
                FlowyText.regular(
                  '${(progress * 100).toStringAsFixed(1)}%',
                  fontSize: 12,
                  color: Theme.of(context).hintColor,
                ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTaskList() {
    return Container(
      constraints: const BoxConstraints(maxHeight: 200),
      child: ListView.builder(
        shrinkWrap: true,
        itemCount: widget.tasks.length,
        itemBuilder: (context, index) {
          final task = widget.tasks[index];
          return _buildTaskItem(task);
        },
      ),
    );
  }

  Widget _buildTaskItem(ConversionTask task) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
        ),
      ),
      child: Row(
        children: [
          _buildTaskIcon(task),
          const HSpace(12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  task.fileName,
                  fontSize: 14,
                  overflow: TextOverflow.ellipsis,
                ),
                const VSpace(4),
                FlowyText.regular(
                  _getTaskStatusText(task),
                  fontSize: 12,
                  color: _getTaskStatusColor(task),
                ),
                if (task.error != null) ...[
                  const VSpace(4),
                  FlowyText.regular(
                    task.error!,
                    fontSize: 11,
                    color: Theme.of(context).colorScheme.error,
                    maxLines: 2,
                  ),
                ],
              ],
            ),
          ),
          if (task.status == ConversionTaskStatus.processing)
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                valueColor: AlwaysStoppedAnimation<Color>(
                  Theme.of(context).colorScheme.primary,
                ),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildTaskIcon(ConversionTask task) {
    IconData iconData;
    Color iconColor;

    switch (task.status) {
      case ConversionTaskStatus.pending:
        iconData = Icons.schedule;
        iconColor = Theme.of(context).hintColor;
        break;
      case ConversionTaskStatus.processing:
        iconData = Icons.autorenew;
        iconColor = Theme.of(context).colorScheme.primary;
        break;
      case ConversionTaskStatus.completed:
        iconData = Icons.check_circle;
        iconColor = Colors.green;
        break;
      case ConversionTaskStatus.error:
        iconData = Icons.error;
        iconColor = Theme.of(context).colorScheme.error;
        break;
      case ConversionTaskStatus.cancelled:
        iconData = Icons.cancel;
        iconColor = Theme.of(context).hintColor;
        break;
    }

    return Icon(
      iconData,
      size: 20,
      color: iconColor,
    );
  }

  Widget _buildActionButtons() {
    final hasError = widget.tasks.any((task) => task.status == ConversionTaskStatus.error);
    final isCompleted = widget.tasks.every((task) => task.status == ConversionTaskStatus.completed);
    final isProcessing = widget.tasks.any((task) => task.status == ConversionTaskStatus.processing);

    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        OutlinedRoundedButton(
          text: '查看日志',
          onTap: _showLogsDialog,
        ),
        const HSpace(12),
        if (hasError && !isCompleted)
          OutlinedRoundedButton(
            text: '重试',
            onTap: _isCancelling ? null : widget.onRetry,
          ),
        if (hasError && !isCompleted) const HSpace(12),
        if (isProcessing || _isCancelling)
          OutlinedRoundedButton(
            text: _isCancelling ? '取消中...' : '取消',
            onTap: _isCancelling ? null : _handleCancel,
          ),
        if (isProcessing || _isCancelling) const HSpace(12),
        OutlinedRoundedButton(
          text: isCompleted ? '完成' : '关闭',
          onTap: widget.onClose,
        ),
      ],
    );
  }

  void _handleCancel() {
    setState(() {
      _isCancelling = true;
    });

    widget.onCancel?.call();
    
    // 延迟重置取消状态，给用户反馈
    Future.delayed(const Duration(milliseconds: 500), () {
      if (mounted) {
        setState(() {
          _isCancelling = false;
        });
      }
    });
  }

  void _showLogsDialog() {
    showConversionLogsDialog(
      context,
      tasks: widget.tasks,
      onClose: () {
        // 关闭日志对话框后，可以选择关闭进度对话框
        // widget.onClose?.call();
      },
    );
  }

  String _getStatusText(bool isCompleted, bool hasError) {
    if (isCompleted) return '已完成';
    if (hasError) return '部分失败';
    return '转换中...';
  }

  Color _getStatusColor(bool isCompleted, bool hasError) {
    if (isCompleted) return Colors.green;
    if (hasError) return Theme.of(context).colorScheme.error;
    return Theme.of(context).colorScheme.primary;
  }

  String _getTaskStatusText(ConversionTask task) {
    switch (task.status) {
      case ConversionTaskStatus.pending:
        return '等待转换';
      case ConversionTaskStatus.processing:
        return '正在转换...';
      case ConversionTaskStatus.completed:
        return '转换完成';
      case ConversionTaskStatus.error:
        return '转换失败';
      case ConversionTaskStatus.cancelled:
        return '已取消';
    }
  }

  Color _getTaskStatusColor(ConversionTask task) {
    switch (task.status) {
      case ConversionTaskStatus.pending:
        return Theme.of(context).hintColor;
      case ConversionTaskStatus.processing:
        return Theme.of(context).colorScheme.primary;
      case ConversionTaskStatus.completed:
        return Colors.green;
      case ConversionTaskStatus.error:
        return Theme.of(context).colorScheme.error;
      case ConversionTaskStatus.cancelled:
        return Theme.of(context).hintColor;
    }
  }
}

/// 转换任务状态
enum ConversionTaskStatus {
  pending,
  processing,
  completed,
  error,
  cancelled,
}

/// 转换任务数据模型
class ConversionTask {
  const ConversionTask({
    required this.fileName,
    required this.filePath,
    required this.importType,
    this.status = ConversionTaskStatus.pending,
    this.progress = 0.0,
    this.error,
    this.startTime,
    this.endTime,
  });

  final String fileName;
  final String filePath;
  final ImportType importType;
  final ConversionTaskStatus status;
  final double progress; // 0.0 to 1.0
  final String? error;
  final DateTime? startTime;
  final DateTime? endTime;

  ConversionTask copyWith({
    String? fileName,
    String? filePath,
    ImportType? importType,
    ConversionTaskStatus? status,
    double? progress,
    String? error,
    DateTime? startTime,
    DateTime? endTime,
  }) {
    return ConversionTask(
      fileName: fileName ?? this.fileName,
      filePath: filePath ?? this.filePath,
      importType: importType ?? this.importType,
      status: status ?? this.status,
      progress: progress ?? this.progress,
      error: error ?? this.error,
      startTime: startTime ?? this.startTime,
      endTime: endTime ?? this.endTime,
    );
  }
}

/// 显示转换进度对话框
Future<void> showConversionProgressDialog(
  BuildContext context, {
  required List<ConversionTask> tasks,
  VoidCallback? onCancel,
  VoidCallback? onRetry,
  VoidCallback? onClose,
}) async {
  await FlowyOverlay.show(
    context: context,
    builder: (context) => ConversionProgressDialog(
      tasks: tasks,
      onCancel: onCancel,
      onRetry: onRetry,
      onClose: onClose,
    ),
  );
}
