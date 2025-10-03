import 'package:flutter/material.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';

/// 工具调用显示组件
/// 
/// 显示AI使用的工具调用状态和结果
class ToolCallDisplay extends StatelessWidget {
  const ToolCallDisplay({
    super.key,
    required this.toolCalls,
  });

  final List<ToolCallInfo> toolCalls;

  @override
  Widget build(BuildContext context) {
    if (toolCalls.isEmpty) {
      return const SizedBox.shrink();
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const VSpace(8),
        ...toolCalls.map((call) => _ToolCallItem(toolCall: call)),
      ],
    );
  }
}

/// 单个工具调用项
class _ToolCallItem extends StatefulWidget {
  const _ToolCallItem({required this.toolCall});

  final ToolCallInfo toolCall;

  @override
  State<_ToolCallItem> createState() => _ToolCallItemState();
}

class _ToolCallItemState extends State<_ToolCallItem>
    with SingleTickerProviderStateMixin {
  bool _isExpanded = false;
  late AnimationController _animationController;
  late Animation<double> _expandAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
    _expandAnimation = CurvedAnimation(
      parent: _animationController,
      curve: Curves.easeInOut,
    );
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  void _toggleExpanded() {
    setState(() {
      _isExpanded = !_isExpanded;
      if (_isExpanded) {
        _animationController.forward();
      } else {
        _animationController.reverse();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8.0),
      child: Container(
        decoration: BoxDecoration(
          color: _getStatusColor(context).withOpacity(0.1),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: _getStatusColor(context).withOpacity(0.3),
            width: 1,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 工具调用头部
            InkWell(
              onTap: _toggleExpanded,
              borderRadius: BorderRadius.circular(8),
              child: Padding(
                padding: const EdgeInsets.all(12.0),
                child: Row(
                  children: [
                    // 状态图标
                    _buildStatusIcon(),
                    const HSpace(8),
                    // 工具名称
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            widget.toolCall.toolName,
                            style: TextStyle(
                              fontWeight: FontWeight.w600,
                              fontSize: 14,
                              color: Theme.of(context).colorScheme.onSurface,
                            ),
                          ),
                          if (widget.toolCall.description != null) ...[
                            const VSpace(2),
                            Text(
                              widget.toolCall.description!,
                              style: TextStyle(
                                fontSize: 12,
                                color: AFThemeExtension.of(context).textColor.withOpacity(0.6),
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                    // 展开/折叠图标
                    RotationTransition(
                      turns: Tween(begin: 0.0, end: 0.5).animate(_expandAnimation),
                      child: Icon(
                        Icons.keyboard_arrow_down,
                        size: 20,
                        color: AFThemeExtension.of(context).textColor,
                      ),
                    ),
                  ],
                ),
              ),
            ),
            // 展开的详细内容
            SizeTransition(
              sizeFactor: _expandAnimation,
              child: Padding(
                padding: const EdgeInsets.fromLTRB(12, 0, 12, 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Divider(height: 1),
                    const VSpace(8),
                    // 参数
                    if (widget.toolCall.arguments.isNotEmpty) ...[
                      _buildSectionTitle(context, '参数'),
                      const VSpace(4),
                      _buildArgumentsList(context),
                      const VSpace(8),
                    ],
                    // 结果
                    if (widget.toolCall.result != null) ...[
                      _buildSectionTitle(context, '结果'),
                      const VSpace(4),
                      _buildResult(context),
                    ],
                    // 错误信息
                    if (widget.toolCall.error != null) ...[
                      _buildSectionTitle(context, '错误'),
                      const VSpace(4),
                      _buildError(context),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusIcon() {
    IconData icon;
    Color color = _getStatusColor(context);

    switch (widget.toolCall.status) {
      case ToolCallStatus.pending:
        icon = Icons.schedule;
        break;
      case ToolCallStatus.running:
        icon = Icons.refresh;
        break;
      case ToolCallStatus.success:
        icon = Icons.check_circle;
        break;
      case ToolCallStatus.failed:
        icon = Icons.error;
        break;
    }

    return widget.toolCall.status == ToolCallStatus.running
        ? SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              valueColor: AlwaysStoppedAnimation<Color>(color),
            ),
          )
        : Icon(icon, size: 20, color: color);
  }

  Color _getStatusColor(BuildContext context) {
    switch (widget.toolCall.status) {
      case ToolCallStatus.pending:
        return Colors.grey;
      case ToolCallStatus.running:
        return Colors.blue;
      case ToolCallStatus.success:
        return Colors.green;
      case ToolCallStatus.failed:
        return Colors.red;
    }
  }

  Widget _buildSectionTitle(BuildContext context, String title) {
    return Text(
      title,
      style: TextStyle(
        fontSize: 12,
        fontWeight: FontWeight.w600,
        color: AFThemeExtension.of(context).textColor.withOpacity(0.7),
      ),
    );
  }

  Widget _buildArgumentsList(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface.withOpacity(0.5),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: widget.toolCall.arguments.entries.map((entry) {
          return Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '${entry.key}: ',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w500,
                    color: AFThemeExtension.of(context).textColor,
                  ),
                ),
                Expanded(
                  child: Text(
                    entry.value.toString(),
                    style: TextStyle(
                      fontSize: 12,
                      color: AFThemeExtension.of(context).textColor.withOpacity(0.8),
                    ),
                  ),
                ),
              ],
            ),
          );
        }).toList(),
      ),
    );
  }

  Widget _buildResult(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.green.withOpacity(0.05),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: Colors.green.withOpacity(0.2),
          width: 1,
        ),
      ),
      child: Text(
        widget.toolCall.result!,
        style: TextStyle(
          fontSize: 12,
          color: AFThemeExtension.of(context).textColor,
        ),
      ),
    );
  }

  Widget _buildError(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.red.withOpacity(0.05),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: Colors.red.withOpacity(0.2),
          width: 1,
        ),
      ),
      child: Text(
        widget.toolCall.error!,
        style: const TextStyle(
          fontSize: 12,
          color: Colors.red,
        ),
      ),
    );
  }
}

/// 工具调用信息
class ToolCallInfo {
  const ToolCallInfo({
    required this.id,
    required this.toolName,
    required this.status,
    required this.arguments,
    this.description,
    this.result,
    this.error,
    this.startTime,
    this.endTime,
  });

  final String id;
  final String toolName;
  final ToolCallStatus status;
  final Map<String, dynamic> arguments;
  final String? description;
  final String? result;
  final String? error;
  final DateTime? startTime;
  final DateTime? endTime;

  Duration? get duration {
    if (startTime != null && endTime != null) {
      return endTime!.difference(startTime!);
    }
    return null;
  }
}

/// 工具调用状态
enum ToolCallStatus {
  pending,
  running,
  success,
  failed,
}


