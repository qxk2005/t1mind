import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:percent_indicator/percent_indicator.dart';

import '../application/task_execution_notifier.dart';
import '../application/task_planner_entities.dart';

/// 任务执行进度显示组件
/// 
/// 在AI聊天界面中显示任务执行的实时进度，包括：
/// - 整体进度条
/// - 当前步骤信息
/// - 执行通知列表
/// - 执行时间统计
class TaskExecutionProgressWidget extends StatefulWidget {
  const TaskExecutionProgressWidget({
    super.key,
    required this.taskPlanId,
    this.showNotifications = true,
    this.showDetailedProgress = true,
    this.compact = false,
  });

  final String taskPlanId;
  final bool showNotifications;
  final bool showDetailedProgress;
  final bool compact;

  @override
  State<TaskExecutionProgressWidget> createState() => _TaskExecutionProgressWidgetState();
}

class _TaskExecutionProgressWidgetState extends State<TaskExecutionProgressWidget>
    with TickerProviderStateMixin {
  late AnimationController _progressAnimationController;
  late AnimationController _pulseAnimationController;
  late Animation<double> _progressAnimation;
  late Animation<double> _pulseAnimation;
  
  Timer? _updateTimer;

  @override
  void initState() {
    super.initState();
    
    _progressAnimationController = AnimationController(
      duration: const Duration(milliseconds: 800),
      vsync: this,
    );
    
    _pulseAnimationController = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    );
    
    _progressAnimation = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(
      parent: _progressAnimationController,
      curve: Curves.easeInOut,
    ));
    
    _pulseAnimation = Tween<double>(
      begin: 0.8,
      end: 1.2,
    ).animate(CurvedAnimation(
      parent: _pulseAnimationController,
      curve: Curves.easeInOut,
    ));
    
    // 启动定时器更新界面
    _updateTimer = Timer.periodic(const Duration(milliseconds: 500), (_) {
      if (mounted) {
        setState(() {});
      }
    });
  }

  @override
  void dispose() {
    _progressAnimationController.dispose();
    _pulseAnimationController.dispose();
    _updateTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<TaskExecutionNotifier, TaskExecutionNotifierState>(
      builder: (context, state) {
        final progress = context.read<TaskExecutionNotifier>().getTaskProgress(widget.taskPlanId);
        final notifications = context.read<TaskExecutionNotifier>().getTaskNotifications(widget.taskPlanId);
        
        if (progress == null) {
          return const SizedBox.shrink();
        }

        // 更新动画
        _updateAnimations(progress);

        return Card(
          margin: const EdgeInsets.symmetric(vertical: 8, horizontal: 4),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                // 标题和状态
                _buildHeader(progress),
                
                const SizedBox(height: 12),
                
                // 进度条
                _buildProgressBar(progress),
                
                if (widget.showDetailedProgress) ...[
                  const SizedBox(height: 12),
                  _buildDetailedProgress(progress),
                ],
                
                if (widget.showNotifications && notifications.isNotEmpty) ...[
                  const SizedBox(height: 16),
                  _buildNotificationsList(notifications),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  /// 构建头部信息
  Widget _buildHeader(TaskExecutionProgress progress) {
    return Row(
      children: [
        // 状态图标
        AnimatedBuilder(
          animation: _pulseAnimation,
          builder: (context, child) {
            return Transform.scale(
              scale: progress.isRunning ? _pulseAnimation.value : 1.0,
              child: _buildStatusIcon(progress.status),
            );
          },
        ),
        
        const SizedBox(width: 12),
        
        // 标题和状态文本
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '任务执行进度',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                _getStatusText(progress),
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: _getStatusColor(progress.status),
                ),
              ),
            ],
          ),
        ),
        
        // 时间信息
        if (progress.executionDurationMs != null)
          Text(
            _formatDuration(progress.executionDurationMs!),
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Colors.grey[600],
            ),
          ),
      ],
    );
  }

  /// 构建进度条
  Widget _buildProgressBar(TaskExecutionProgress progress) {
    return Column(
      children: [
        // 进度条
        AnimatedBuilder(
          animation: _progressAnimation,
          builder: (context, child) {
            return LinearPercentIndicator(
              width: MediaQuery.of(context).size.width - 80,
              lineHeight: 8.0,
              percent: progress.progressPercentage * _progressAnimation.value,
              backgroundColor: Colors.grey[300],
              progressColor: _getStatusColor(progress.status),
              barRadius: const Radius.circular(4),
              animation: false,
            );
          },
        ),
        
        const SizedBox(height: 8),
        
        // 进度文本
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              '${progress.completedSteps}/${progress.totalSteps} 步骤',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            Text(
              '${(progress.progressPercentage * 100).toInt()}%',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ],
    );
  }

  /// 构建详细进度信息
  Widget _buildDetailedProgress(TaskExecutionProgress progress) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.grey[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey[200]!),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 当前步骤
          if (progress.currentStepDescription.isNotEmpty) ...[
            Row(
              children: [
                Icon(
                  Icons.play_circle_outline,
                  size: 16,
                  color: Colors.blue[600],
                ),
                const SizedBox(width: 8),
                Text(
                  '当前步骤',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    fontWeight: FontWeight.w500,
                    color: Colors.blue[600],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              progress.currentStepDescription,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            
            if (progress.currentStepDurationMs != null) ...[
              const SizedBox(height: 4),
              Text(
                '执行时间: ${_formatDuration(progress.currentStepDurationMs!)}',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Colors.grey[600],
                ),
              ),
            ],
          ],
          
          // 错误信息
          if (progress.errorMessage != null) ...[
            const SizedBox(height: 8),
            Row(
              children: [
                Icon(
                  Icons.error_outline,
                  size: 16,
                  color: Colors.red[600],
                ),
                const SizedBox(width: 8),
                Text(
                  '错误信息',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    fontWeight: FontWeight.w500,
                    color: Colors.red[600],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              progress.errorMessage!,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Colors.red[700],
              ),
            ),
          ],
        ],
      ),
    );
  }

  /// 构建通知列表
  Widget _buildNotificationsList(List<TaskExecutionNotification> notifications) {
    // 只显示最近的几条通知
    final recentNotifications = notifications.reversed.take(5).toList();
    
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(
              Icons.notifications_outlined,
              size: 16,
              color: Colors.grey[600],
            ),
            const SizedBox(width: 8),
            Text(
              '执行日志',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontWeight: FontWeight.w500,
                color: Colors.grey[600],
              ),
            ),
          ],
        ),
        
        const SizedBox(height: 8),
        
        Container(
          constraints: const BoxConstraints(maxHeight: 200),
          child: ListView.builder(
            shrinkWrap: true,
            itemCount: recentNotifications.length,
            itemBuilder: (context, index) {
              final notification = recentNotifications[index];
              return _buildNotificationItem(notification);
            },
          ),
        ),
      ],
    );
  }

  /// 构建单个通知项
  Widget _buildNotificationItem(TaskExecutionNotification notification) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 时间
          Text(
            _formatTime(notification.timestamp),
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Colors.grey[500],
              fontSize: 10,
            ),
          ),
          
          const SizedBox(width: 8),
          
          // 类型图标
          _buildNotificationIcon(notification.type),
          
          const SizedBox(width: 8),
          
          // 消息内容
          Expanded(
            child: Text(
              notification.message,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontSize: 12,
              ),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }

  /// 构建状态图标
  Widget _buildStatusIcon(TaskExecutionStatus status) {
    switch (status) {
      case TaskExecutionStatus.running:
        return Icon(Icons.play_circle, color: Colors.blue[600], size: 24);
      case TaskExecutionStatus.completed:
        return Icon(Icons.check_circle, color: Colors.green[600], size: 24);
      case TaskExecutionStatus.failed:
      case TaskExecutionStatus.error:
        return Icon(Icons.error, color: Colors.red[600], size: 24);
      default:
        return Icon(Icons.radio_button_unchecked, color: Colors.grey[600], size: 24);
    }
  }

  /// 构建通知图标
  Widget _buildNotificationIcon(TaskNotificationType type) {
    IconData iconData;
    Color color;
    
    switch (type) {
      case TaskNotificationType.stepStarted:
        iconData = Icons.play_arrow;
        color = Colors.blue;
        break;
      case TaskNotificationType.stepCompleted:
        iconData = Icons.check;
        color = Colors.green;
        break;
      case TaskNotificationType.stepFailed:
      case TaskNotificationType.taskFailed:
        iconData = Icons.error;
        color = Colors.red;
        break;
      case TaskNotificationType.toolCallStarted:
        iconData = Icons.build;
        color = Colors.orange;
        break;
      case TaskNotificationType.toolCallCompleted:
        iconData = Icons.build_circle;
        color = Colors.green;
        break;
      case TaskNotificationType.warning:
        iconData = Icons.warning;
        color = Colors.orange;
        break;
      default:
        iconData = Icons.info;
        color = Colors.grey;
    }
    
    return Icon(iconData, size: 12, color: color);
  }

  /// 获取状态文本
  String _getStatusText(TaskExecutionProgress progress) {
    switch (progress.status) {
      case TaskExecutionStatus.running:
        return '正在执行...';
      case TaskExecutionStatus.completed:
        return '执行完成';
      case TaskExecutionStatus.failed:
        return '执行失败';
      case TaskExecutionStatus.error:
        return '执行错误';
      default:
        return '等待执行';
    }
  }

  /// 获取状态颜色
  Color _getStatusColor(TaskExecutionStatus status) {
    switch (status) {
      case TaskExecutionStatus.running:
        return Colors.blue;
      case TaskExecutionStatus.completed:
        return Colors.green;
      case TaskExecutionStatus.failed:
      case TaskExecutionStatus.error:
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  /// 更新动画
  void _updateAnimations(TaskExecutionProgress progress) {
    // 更新进度动画
    _progressAnimationController.animateTo(1.0);
    
    // 更新脉冲动画
    if (progress.isRunning) {
      if (!_pulseAnimationController.isAnimating) {
        _pulseAnimationController.repeat(reverse: true);
      }
    } else {
      _pulseAnimationController.stop();
      _pulseAnimationController.reset();
    }
  }

  /// 格式化时长
  String _formatDuration(int milliseconds) {
    final duration = Duration(milliseconds: milliseconds);
    if (duration.inHours > 0) {
      return '${duration.inHours}:${(duration.inMinutes % 60).toString().padLeft(2, '0')}:${(duration.inSeconds % 60).toString().padLeft(2, '0')}';
    } else if (duration.inMinutes > 0) {
      return '${duration.inMinutes}:${(duration.inSeconds % 60).toString().padLeft(2, '0')}';
    } else {
      return '${duration.inSeconds}秒';
    }
  }

  /// 格式化时间
  String _formatTime(DateTime dateTime) {
    return '${dateTime.hour.toString().padLeft(2, '0')}:${dateTime.minute.toString().padLeft(2, '0')}:${dateTime.second.toString().padLeft(2, '0')}';
  }
}
