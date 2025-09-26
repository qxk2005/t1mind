import 'package:flutter/foundation.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import 'task_planner_entities.dart';

part 'task_execution_notifier.freezed.dart';

/// 任务执行通知管理器
/// 
/// 负责管理和分发任务执行过程中的通知，
/// 用于在AI聊天界面中实时显示执行状态
class TaskExecutionNotifier extends Cubit<TaskExecutionNotifierState> {
  TaskExecutionNotifier() : super(const TaskExecutionNotifierState.initial());

  final List<TaskExecutionNotification> _notifications = [];
  final Map<String, TaskExecutionProgress> _taskProgress = {};

  /// 添加通知
  void addNotification(TaskExecutionNotification notification) {
    _notifications.add(notification);
    _updateTaskProgress(notification);
    
    emit(TaskExecutionNotifierState.notificationAdded(
      notification: notification,
      allNotifications: List.from(_notifications),
      taskProgress: Map.from(_taskProgress),
    ));
  }

  /// 批量添加通知
  void addNotifications(List<TaskExecutionNotification> notifications) {
    _notifications.addAll(notifications);
    
    for (final notification in notifications) {
      _updateTaskProgress(notification);
    }
    
    emit(TaskExecutionNotifierState.notificationsAdded(
      notifications: notifications,
      allNotifications: List.from(_notifications),
      taskProgress: Map.from(_taskProgress),
    ));
  }

  /// 标记通知为已读
  void markNotificationAsRead(String notificationId) {
    final index = _notifications.indexWhere((n) => n.id == notificationId);
    if (index != -1) {
      _notifications[index] = _notifications[index].copyWith(isRead: true);
      
      emit(TaskExecutionNotifierState.notificationUpdated(
        notificationId: notificationId,
        allNotifications: List.from(_notifications),
        taskProgress: Map.from(_taskProgress),
      ));
    }
  }

  /// 清除指定任务的通知
  void clearTaskNotifications(String taskPlanId) {
    _notifications.removeWhere((n) => n.taskPlanId == taskPlanId);
    _taskProgress.remove(taskPlanId);
    
    emit(TaskExecutionNotifierState.notificationsCleared(
      taskPlanId: taskPlanId,
      allNotifications: List.from(_notifications),
      taskProgress: Map.from(_taskProgress),
    ));
  }

  /// 清除所有通知
  void clearAllNotifications() {
    _notifications.clear();
    _taskProgress.clear();
    
    emit(const TaskExecutionNotifierState.allNotificationsCleared());
  }

  /// 获取指定任务的通知
  List<TaskExecutionNotification> getTaskNotifications(String taskPlanId) {
    return _notifications.where((n) => n.taskPlanId == taskPlanId).toList();
  }

  /// 获取未读通知数量
  int getUnreadCount() {
    return _notifications.where((n) => !n.isRead).length;
  }

  /// 获取指定任务的未读通知数量
  int getTaskUnreadCount(String taskPlanId) {
    return _notifications
        .where((n) => n.taskPlanId == taskPlanId && !n.isRead)
        .length;
  }

  /// 获取任务执行进度
  TaskExecutionProgress? getTaskProgress(String taskPlanId) {
    return _taskProgress[taskPlanId];
  }

  /// 更新任务执行进度
  void _updateTaskProgress(TaskExecutionNotification notification) {
    final taskPlanId = notification.taskPlanId;
    final currentProgress = _taskProgress[taskPlanId] ?? TaskExecutionProgress(
      taskPlanId: taskPlanId,
      status: TaskExecutionStatus.idle,
      currentStep: 0,
      totalSteps: 0,
      startTime: DateTime.now(),
    );

    TaskExecutionProgress updatedProgress = currentProgress;

    switch (notification.type) {
      case TaskNotificationType.info:
        if (notification.title == '任务执行开始') {
          final totalSteps = notification.data['totalSteps'] as int? ?? 0;
          final estimatedDuration = notification.data['estimatedDuration'] as int? ?? 0;
          
          updatedProgress = currentProgress.copyWith(
            status: TaskExecutionStatus.running,
            totalSteps: totalSteps,
            estimatedDurationSeconds: estimatedDuration,
            startTime: notification.timestamp,
          );
        }
        break;

      case TaskNotificationType.stepStarted:
        final stepOrder = notification.data['stepOrder'] as int? ?? 0;
        final description = notification.data['description'] as String? ?? '';
        
        updatedProgress = currentProgress.copyWith(
          currentStep: stepOrder + 1, // stepOrder从0开始，显示时+1
          currentStepDescription: description,
          currentStepStartTime: notification.timestamp,
        );
        break;

      case TaskNotificationType.stepCompleted:
        final completedSteps = notification.data['completedSteps'] as int? ?? 0;
        
        updatedProgress = currentProgress.copyWith(
          completedSteps: completedSteps,
        );
        break;

      case TaskNotificationType.stepFailed:
        updatedProgress = currentProgress.copyWith(
          status: TaskExecutionStatus.error,
          errorMessage: notification.message,
        );
        break;

      case TaskNotificationType.taskCompleted:
        final completedSteps = notification.data['completedSteps'] as int? ?? 0;
        
        updatedProgress = currentProgress.copyWith(
          status: TaskExecutionStatus.completed,
          completedSteps: completedSteps,
          endTime: notification.timestamp,
        );
        break;

      case TaskNotificationType.taskFailed:
        updatedProgress = currentProgress.copyWith(
          status: TaskExecutionStatus.failed,
          errorMessage: notification.message,
          endTime: notification.timestamp,
        );
        break;

      default:
        // 其他类型的通知不更新进度
        break;
    }

    _taskProgress[taskPlanId] = updatedProgress;
  }
}

/// 任务执行通知器状态
@freezed
class TaskExecutionNotifierState with _$TaskExecutionNotifierState {
  const factory TaskExecutionNotifierState.initial() = _Initial;
  
  const factory TaskExecutionNotifierState.notificationAdded({
    required TaskExecutionNotification notification,
    required List<TaskExecutionNotification> allNotifications,
    required Map<String, TaskExecutionProgress> taskProgress,
  }) = _NotificationAdded;
  
  const factory TaskExecutionNotifierState.notificationsAdded({
    required List<TaskExecutionNotification> notifications,
    required List<TaskExecutionNotification> allNotifications,
    required Map<String, TaskExecutionProgress> taskProgress,
  }) = _NotificationsAdded;
  
  const factory TaskExecutionNotifierState.notificationUpdated({
    required String notificationId,
    required List<TaskExecutionNotification> allNotifications,
    required Map<String, TaskExecutionProgress> taskProgress,
  }) = _NotificationUpdated;
  
  const factory TaskExecutionNotifierState.notificationsCleared({
    required String taskPlanId,
    required List<TaskExecutionNotification> allNotifications,
    required Map<String, TaskExecutionProgress> taskProgress,
  }) = _NotificationsCleared;
  
  const factory TaskExecutionNotifierState.allNotificationsCleared() = _AllNotificationsCleared;
}

/// 任务执行进度
@freezed
class TaskExecutionProgress with _$TaskExecutionProgress {
  const TaskExecutionProgress._();
  
  const factory TaskExecutionProgress({
    /// 任务规划ID
    required String taskPlanId,
    /// 执行状态
    required TaskExecutionStatus status,
    /// 当前步骤（从1开始）
    @Default(0) int currentStep,
    /// 总步骤数
    @Default(0) int totalSteps,
    /// 已完成步骤数
    @Default(0) int completedSteps,
    /// 当前步骤描述
    @Default('') String currentStepDescription,
    /// 预估总时长（秒）
    @Default(0) int estimatedDurationSeconds,
    /// 开始时间
    DateTime? startTime,
    /// 结束时间
    DateTime? endTime,
    /// 当前步骤开始时间
    DateTime? currentStepStartTime,
    /// 错误信息
    String? errorMessage,
  }) = _TaskExecutionProgress;

  /// 计算进度百分比
  double get progressPercentage {
    if (totalSteps == 0) return 0.0;
    return completedSteps / totalSteps;
  }

  /// 是否已完成
  bool get isCompleted => status == TaskExecutionStatus.completed;

  /// 是否失败
  bool get isFailed => status == TaskExecutionStatus.failed;

  /// 是否正在运行
  bool get isRunning => status == TaskExecutionStatus.running;

  /// 获取执行时长（毫秒）
  int? get executionDurationMs {
    if (startTime == null) return null;
    final endTimeToUse = endTime ?? DateTime.now();
    return endTimeToUse.difference(startTime!).inMilliseconds;
  }

  /// 获取当前步骤执行时长（毫秒）
  int? get currentStepDurationMs {
    if (currentStepStartTime == null) return null;
    return DateTime.now().difference(currentStepStartTime!).inMilliseconds;
  }
}

/// 任务执行状态
enum TaskExecutionStatus {
  /// 空闲
  idle,
  /// 运行中
  running,
  /// 已完成
  completed,
  /// 失败
  failed,
  /// 错误
  error,
}
