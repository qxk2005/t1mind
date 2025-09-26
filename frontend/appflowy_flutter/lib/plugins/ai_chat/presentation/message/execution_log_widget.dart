import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

/// 执行日志显示组件
class ExecutionLogWidget extends StatefulWidget {
  const ExecutionLogWidget({
    super.key,
    required this.executionLog,
    this.isExpanded = false,
  });

  final ExecutionLog executionLog;
  final bool isExpanded;

  @override
  State<ExecutionLogWidget> createState() => _ExecutionLogWidgetState();
}

class _ExecutionLogWidgetState extends State<ExecutionLogWidget> {
  bool _isExpanded = false;

  @override
  void initState() {
    super.initState();
    _isExpanded = widget.isExpanded;
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(top: 8.0),
      decoration: BoxDecoration(
        border: Border.all(
          color: Theme.of(context).dividerColor.withValues(alpha: 0.3),
        ),
        borderRadius: BorderRadius.circular(8.0),
        color: Theme.of(context).colorScheme.surface.withValues(alpha: 0.5),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildHeader(),
          if (_isExpanded) ...[
            const Divider(height: 1),
            _buildExpandedContent(),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return InkWell(
      onTap: () {
        setState(() {
          _isExpanded = !_isExpanded;
        });
      },
      borderRadius: BorderRadius.circular(8.0),
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Row(
          children: [
            _buildStatusIcon(),
            const HSpace(8.0),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '执行追溯信息',
                    style: Theme.of(context).textTheme.titleSmall?.copyWith(
                          fontWeight: FontWeight.w600,
                        ),
                  ),
                  const VSpace(2.0),
                  Text(
                    _buildSummaryText(),
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
                        ),
                  ),
                ],
              ),
            ),
            Icon(
              _isExpanded ? Icons.expand_less : Icons.expand_more,
              size: 20,
              color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusIcon() {
    IconData iconData;
    Color iconColor;

    switch (widget.executionLog.status) {
      case ExecutionLogStatus.completed:
        iconData = Icons.check_circle;
        iconColor = Colors.green;
        break;
      case ExecutionLogStatus.failed:
        iconData = Icons.error;
        iconColor = Colors.red;
        break;
      case ExecutionLogStatus.running:
        iconData = Icons.play_circle;
        iconColor = Colors.blue;
        break;
      case ExecutionLogStatus.cancelled:
        iconData = Icons.cancel;
        iconColor = Colors.orange;
        break;
      default:
        iconData = Icons.info;
        iconColor = Colors.grey;
    }

    return Icon(
      iconData,
      size: 16,
      color: iconColor,
    );
  }

  String _buildSummaryText() {
    final duration = widget.executionLog.endTime?.difference(widget.executionLog.startTime);
    
    final parts = <String>[];
    
    // 状态信息
    parts.add('状态: ${_getStatusText(widget.executionLog.status)}');
    
    // 步骤信息
    if (widget.executionLog.totalSteps > 0) {
      parts.add('步骤: ${widget.executionLog.completedSteps}/${widget.executionLog.totalSteps}');
    }
    
    // 执行时间
    if (duration != null) {
      parts.add('耗时: ${_formatDuration(duration)}');
    }
    
    // 使用的工具
    if (widget.executionLog.usedMcpTools.isNotEmpty) {
      parts.add('工具: ${widget.executionLog.usedMcpTools.length}个');
    }

    return parts.join(' • ');
  }

  String _getStatusText(ExecutionLogStatus status) {
    switch (status) {
      case ExecutionLogStatus.initialized:
        return '已初始化';
      case ExecutionLogStatus.preparing:
        return '准备中';
      case ExecutionLogStatus.running:
        return '执行中';
      case ExecutionLogStatus.paused:
        return '已暂停';
      case ExecutionLogStatus.completed:
        return '已完成';
      case ExecutionLogStatus.failed:
        return '执行失败';
      case ExecutionLogStatus.cancelled:
        return '已取消';
      case ExecutionLogStatus.timeout:
        return '执行超时';
    }
  }

  String _formatDuration(Duration duration) {
    if (duration.inHours > 0) {
      return '${duration.inHours}小时${duration.inMinutes.remainder(60)}分钟';
    } else if (duration.inMinutes > 0) {
      return '${duration.inMinutes}分钟${duration.inSeconds.remainder(60)}秒';
    } else {
      return '${duration.inSeconds}秒';
    }
  }

  Widget _buildExpandedContent() {
    return Padding(
      padding: const EdgeInsets.all(12.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildBasicInfo(),
          if (widget.executionLog.steps.isNotEmpty) ...[
            const VSpace(16.0),
            _buildStepsSection(),
          ],
          if (widget.executionLog.errorMessage != null) ...[
            const VSpace(16.0),
            _buildErrorSection(),
          ],
        ],
      ),
    );
  }

  Widget _buildBasicInfo() {
    final dateFormat = DateFormat('yyyy-MM-dd HH:mm:ss');
    
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '基本信息',
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
              ),
        ),
        const VSpace(8.0),
        _buildInfoRow('执行ID', widget.executionLog.id),
        _buildInfoRow('会话ID', widget.executionLog.sessionId),
        if (widget.executionLog.taskPlanId != null)
          _buildInfoRow('任务规划ID', widget.executionLog.taskPlanId!),
        _buildInfoRow('开始时间', dateFormat.format(widget.executionLog.startTime)),
        if (widget.executionLog.endTime != null)
          _buildInfoRow('结束时间', dateFormat.format(widget.executionLog.endTime!)),
        if (widget.executionLog.usedMcpTools.isNotEmpty)
          _buildInfoRow('使用工具', widget.executionLog.usedMcpTools.join(', ')),
      ],
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 4.0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 80,
            child: Text(
              '$label:',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
                  ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStepsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '执行步骤',
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
              ),
        ),
        const VSpace(8.0),
        ...widget.executionLog.steps.map((step) => _buildStepItem(step)),
      ],
    );
  }

  Widget _buildStepItem(ExecutionStep step) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8.0),
      padding: const EdgeInsets.all(8.0),
      decoration: BoxDecoration(
        border: Border.all(
          color: Theme.of(context).dividerColor.withValues(alpha: 0.2),
        ),
        borderRadius: BorderRadius.circular(6.0),
        color: Theme.of(context).colorScheme.surface.withValues(alpha: 0.3),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              _buildStepStatusIcon(step.status),
              const HSpace(6.0),
              Expanded(
                child: Text(
                  step.name,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        fontWeight: FontWeight.w500,
                      ),
                ),
              ),
              if (step.executionTimeMs > 0)
                Text(
                  '${step.executionTimeMs}ms',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
                      ),
                ),
            ],
          ),
          if (step.description.isNotEmpty) ...[
            const VSpace(4.0),
            Text(
              step.description,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
                  ),
            ),
          ],
          if (step.mcpTool.name.isNotEmpty) ...[
            const VSpace(4.0),
            Row(
              children: [
                Icon(
                  Icons.build,
                  size: 12,
                  color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
                ),
                const HSpace(4.0),
                Text(
                  '工具: ${step.mcpTool.displayName ?? step.mcpTool.name}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
                      ),
                ),
              ],
            ),
          ],
          if (step.references.isNotEmpty) ...[
            const VSpace(4.0),
            _buildReferencesSection(step.references),
          ],
          if (step.errorMessage != null) ...[
            const VSpace(4.0),
            Container(
              padding: const EdgeInsets.all(6.0),
              decoration: BoxDecoration(
                color: Colors.red.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(4.0),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    size: 12,
                    color: Colors.red,
                  ),
                  const HSpace(4.0),
                  Expanded(
                    child: Text(
                      step.errorMessage!,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.red,
                          ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildStepStatusIcon(ExecutionStepStatus status) {
    IconData iconData;
    Color iconColor;

    switch (status) {
      case ExecutionStepStatus.success:
        iconData = Icons.check_circle;
        iconColor = Colors.green;
        break;
      case ExecutionStepStatus.error:
        iconData = Icons.error;
        iconColor = Colors.red;
        break;
      case ExecutionStepStatus.executing:
        iconData = Icons.play_circle;
        iconColor = Colors.blue;
        break;
      case ExecutionStepStatus.skipped:
        iconData = Icons.skip_next;
        iconColor = Colors.orange;
        break;
      case ExecutionStepStatus.cancelled:
        iconData = Icons.cancel;
        iconColor = Colors.orange;
        break;
      default:
        iconData = Icons.radio_button_unchecked;
        iconColor = Colors.grey;
    }

    return Icon(
      iconData,
      size: 12,
      color: iconColor,
    );
  }

  Widget _buildReferencesSection(List<ExecutionReference> references) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(
              Icons.link,
              size: 12,
              color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
            ),
            const HSpace(4.0),
            Text(
              '引用信息 (${references.length})',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
                  ),
            ),
          ],
        ),
        const VSpace(4.0),
        ...references.take(3).map((ref) => _buildReferenceItem(ref)),
        if (references.length > 3)
          Text(
            '... 还有 ${references.length - 3} 个引用',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
                  fontStyle: FontStyle.italic,
                ),
          ),
      ],
    );
  }

  Widget _buildReferenceItem(ExecutionReference reference) {
    return Padding(
      padding: const EdgeInsets.only(left: 16.0, bottom: 2.0),
      child: Row(
        children: [
          Icon(
            _getReferenceIcon(reference.type),
            size: 10,
                        color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
          ),
          const HSpace(4.0),
          Expanded(
            child: Text(
              reference.title,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.8),
                  ),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getReferenceIcon(ExecutionReferenceType type) {
    switch (type) {
      case ExecutionReferenceType.document:
        return Icons.description;
      case ExecutionReferenceType.webpage:
        return Icons.web;
      case ExecutionReferenceType.api:
        return Icons.api;
      case ExecutionReferenceType.database:
        return Icons.storage;
      case ExecutionReferenceType.file:
        return Icons.insert_drive_file;
      case ExecutionReferenceType.image:
        return Icons.image;
      case ExecutionReferenceType.video:
        return Icons.video_file;
      case ExecutionReferenceType.other:
        return Icons.help_outline;
    }
  }

  Widget _buildErrorSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '错误信息',
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
                color: Colors.red,
              ),
        ),
        const VSpace(8.0),
        Container(
          padding: const EdgeInsets.all(8.0),
          decoration: BoxDecoration(
            color: Colors.red.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(6.0),
            border: Border.all(
              color: Colors.red.withValues(alpha: 0.3),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (widget.executionLog.errorType != null)
                Text(
                  '错误类型: ${widget.executionLog.errorType!.description}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Colors.red,
                        fontWeight: FontWeight.w500,
                      ),
                ),
              if (widget.executionLog.errorMessage != null) ...[
                const VSpace(4.0),
                Text(
                  widget.executionLog.errorMessage!,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Colors.red,
                      ),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }
}
