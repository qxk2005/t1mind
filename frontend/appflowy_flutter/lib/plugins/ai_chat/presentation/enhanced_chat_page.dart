import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../application/task_execution_notifier.dart';
import '../application/task_planner_bloc.dart';
import '../application/task_planner_entities.dart';
import '../application/intelligent_task_executor.dart';
import '../application/enhanced_mcp_tool_service.dart';
import '../chat_page.dart';

/// 增强的聊天页面
/// 
/// 集成了任务规划、执行通知和进度显示功能
class EnhancedChatPage extends StatefulWidget {
  const EnhancedChatPage({
    super.key,
    required this.chatId,
    required this.userId,
  });

  final String chatId;
  final String userId;

  @override
  State<EnhancedChatPage> createState() => _EnhancedChatPageState();
}

class _EnhancedChatPageState extends State<EnhancedChatPage> {
  late TaskExecutionNotifier _taskExecutionNotifier;
  late TaskPlannerBloc _taskPlannerBloc;
  late IntelligentTaskExecutor _taskExecutor;
  late EnhancedMcpToolService _mcpToolService;

  @override
  void initState() {
    super.initState();
    
    // 初始化组件
    _taskExecutionNotifier = TaskExecutionNotifier();
    _taskPlannerBloc = TaskPlannerBloc(
      sessionId: widget.chatId,
      userId: widget.userId,
    );
    
    // 初始化任务执行器，设置通知回调
    _taskExecutor = IntelligentTaskExecutor(
      sessionId: widget.chatId,
      userId: widget.userId,
      onNotification: (notification) {
        _taskExecutionNotifier.addNotification(notification);
      },
    );
    
    // 初始化MCP工具服务
    _mcpToolService = EnhancedMcpToolService(
      onToolCallStarted: (toolId, parameters) {
        // 可以在这里添加工具调用开始的处理逻辑
      },
      onToolCallCompleted: (toolId, result) {
        // 可以在这里添加工具调用完成的处理逻辑
      },
      onToolCallFailed: (toolId, error) {
        // 可以在这里添加工具调用失败的处理逻辑
      },
      onToolRetry: (toolId, retryCount, reason) {
        // 可以在这里添加工具重试的处理逻辑
      },
    );
    
    // 监听任务规划状态变化
    _taskPlannerBloc.stream.listen((state) {
      if (state.status == TaskPlannerStatus.planReady && 
          state.currentTaskPlan != null) {
        // 任务规划完成，开始执行
        _executeTaskPlan(state.currentTaskPlan!);
      }
    });
  }

  @override
  void dispose() {
    _taskExecutionNotifier.close();
    _taskPlannerBloc.close();
    super.dispose();
  }

  /// 执行任务规划
  Future<void> _executeTaskPlan(TaskPlan taskPlan) async {
    try {
      // 使用智能任务执行器执行任务
      final result = await _taskExecutor.executeTaskPlan(taskPlan);
      
      if (result.isSuccess) {
        // 任务执行成功
        debugPrint('任务执行成功: ${taskPlan.id}');
      } else {
        // 任务执行失败
        debugPrint('任务执行失败: ${taskPlan.id} - ${result.errorMessage}');
      }
    } catch (e) {
      debugPrint('任务执行异常: ${taskPlan.id} - $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<TaskExecutionNotifier>.value(
          value: _taskExecutionNotifier,
        ),
        BlocProvider<TaskPlannerBloc>.value(
          value: _taskPlannerBloc,
        ),
      ],
      child: Scaffold(
        body: Column(
          children: [
            // 任务执行状态栏（可选）
            _buildTaskExecutionStatusBar(),
            
            // 主聊天界面
            Expanded(
              child: Container(
                child: Text('Chat interface will be integrated here'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 构建任务执行状态栏
  Widget _buildTaskExecutionStatusBar() {
    return BlocBuilder<TaskExecutionNotifier, TaskExecutionNotifierState>(
      builder: (context, state) {
        // 获取当前活跃的任务进度
        final activeProgress = _getActiveTaskProgress(state);
        
        if (activeProgress == null) {
          return const SizedBox.shrink();
        }

        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.5),
            border: Border(
              bottom: BorderSide(
                color: Theme.of(context).dividerColor,
                width: 0.5,
              ),
            ),
          ),
          child: Row(
            children: [
              // 状态图标
              _buildStatusIcon(activeProgress.status),
              
              const SizedBox(width: 12),
              
              // 进度信息
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      _getStatusText(activeProgress.status),
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                    if (activeProgress.currentStepDescription.isNotEmpty)
                      Text(
                        activeProgress.currentStepDescription,
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.grey[600],
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                  ],
                ),
              ),
              
              // 进度百分比
              Text(
                '${(activeProgress.progressPercentage * 100).toInt()}%',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  fontWeight: FontWeight.w500,
                ),
              ),
              
              const SizedBox(width: 8),
              
              // 关闭按钮
              IconButton(
                icon: const Icon(Icons.close, size: 16),
                onPressed: () {
                  // 清除当前任务的通知
                  _taskExecutionNotifier.clearTaskNotifications(activeProgress.taskPlanId);
                },
                constraints: const BoxConstraints(
                  minWidth: 24,
                  minHeight: 24,
                ),
                padding: EdgeInsets.zero,
              ),
            ],
          ),
        );
      },
    );
  }

  /// 获取当前活跃的任务进度
  TaskExecutionProgress? _getActiveTaskProgress(TaskExecutionNotifierState state) {
    return state.when(
      initial: () => null,
      notificationAdded: (notification, allNotifications, taskProgress) {
        // 查找正在运行的任务
        return taskProgress.values
            .where((progress) => progress.isRunning)
            .isNotEmpty
            ? taskProgress.values.firstWhere((progress) => progress.isRunning)
            : null;
      },
      notificationsAdded: (notifications, allNotifications, taskProgress) {
        return taskProgress.values
            .where((progress) => progress.isRunning)
            .isNotEmpty
            ? taskProgress.values.firstWhere((progress) => progress.isRunning)
            : null;
      },
      notificationUpdated: (notificationId, allNotifications, taskProgress) {
        return taskProgress.values
            .where((progress) => progress.isRunning)
            .isNotEmpty
            ? taskProgress.values.firstWhere((progress) => progress.isRunning)
            : null;
      },
      notificationsCleared: (taskPlanId, allNotifications, taskProgress) {
        return taskProgress.values
            .where((progress) => progress.isRunning)
            .isNotEmpty
            ? taskProgress.values.firstWhere((progress) => progress.isRunning)
            : null;
      },
      allNotificationsCleared: () => null,
    );
  }

  /// 构建状态图标
  Widget _buildStatusIcon(TaskExecutionStatus status) {
    switch (status) {
      case TaskExecutionStatus.running:
        return SizedBox(
          width: 16,
          height: 16,
          child: CircularProgressIndicator(
            strokeWidth: 2,
            valueColor: AlwaysStoppedAnimation<Color>(
              Theme.of(context).colorScheme.primary,
            ),
          ),
        );
      case TaskExecutionStatus.completed:
        return Icon(
          Icons.check_circle,
          size: 16,
          color: Colors.green[600],
        );
      case TaskExecutionStatus.failed:
      case TaskExecutionStatus.error:
        return Icon(
          Icons.error,
          size: 16,
          color: Colors.red[600],
        );
      default:
        return Icon(
          Icons.radio_button_unchecked,
          size: 16,
          color: Colors.grey[600],
        );
    }
  }

  /// 获取状态文本
  String _getStatusText(TaskExecutionStatus status) {
    switch (status) {
      case TaskExecutionStatus.running:
        return '任务执行中...';
      case TaskExecutionStatus.completed:
        return '任务执行完成';
      case TaskExecutionStatus.failed:
        return '任务执行失败';
      case TaskExecutionStatus.error:
        return '任务执行错误';
      default:
        return '等待执行';
    }
  }
}

