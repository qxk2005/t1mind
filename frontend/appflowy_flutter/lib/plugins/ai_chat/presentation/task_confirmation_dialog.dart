import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';

/// 任务确认对话框的操作类型
enum TaskConfirmationAction {
  /// 确认执行
  confirm,
  /// 拒绝执行
  reject,
  /// 修改任务
  modify,
}

/// 任务确认对话框
/// 
/// 用于展示AI生成的任务规划详情，并让用户选择确认、拒绝或修改
class TaskConfirmationDialog extends StatelessWidget {
  const TaskConfirmationDialog({
    super.key,
    required this.taskPlan,
    required this.onAction,
  });

  /// 要确认的任务规划
  final TaskPlan taskPlan;
  
  /// 用户操作回调
  final Future<void> Function(TaskConfirmationAction action) onAction;

  @override
  Widget build(BuildContext context) {
    final screenHeight = MediaQuery.of(context).size.height;
    final maxHeight = screenHeight * 0.8; // 最大高度为屏幕高度的80%
    
    return Dialog(
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12.0),
      ),
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxWidth: 600,
          maxHeight: maxHeight,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // 标题和描述部分
            _buildHeader(context),
            
            // 可滚动的内容部分
            Flexible(
              child: _buildScrollableTaskPlanDetails(context),
            ),
            
            // 底部按钮部分
            _buildActions(context),
          ],
        ),
      ),
    );
  }

  /// 构建对话框头部
  Widget _buildHeader(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(12.0),
          topRight: Radius.circular(12.0),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: FlowyText.medium(
                  '任务确认',
                  fontSize: 18,
                  color: Theme.of(context).colorScheme.onSurface,
                ),
              ),
              IconButton(
                onPressed: () => onAction(TaskConfirmationAction.reject),
                icon: Icon(
                  Icons.close,
                  color: Theme.of(context).colorScheme.onSurface,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            '请查看AI生成的任务规划，确认是否执行：',
            fontSize: 14,
            color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
          ),
        ],
      ),
    );
  }

  /// 构建底部操作按钮
  Widget _buildActions(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: const BorderRadius.only(
          bottomLeft: Radius.circular(12.0),
          bottomRight: Radius.circular(12.0),
        ),
        border: Border(
          top: BorderSide(
            color: Theme.of(context).dividerColor,
            width: 1,
          ),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          // 修改按钮
          FlowyButton(
            text: FlowyText.medium(
              '修改任务',
              fontSize: 14,
              color: Theme.of(context).colorScheme.primary,
            ),
            onTap: () => onAction(TaskConfirmationAction.modify),
            useIntrinsicWidth: true,
            margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            radius: BorderRadius.circular(8),
            hoverColor: Theme.of(context).colorScheme.primaryContainer,
          ),
          
          // 取消和确认按钮
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              FlowyButton(
                text: FlowyText.medium(
                  '取消',
                  fontSize: 14,
                  color: Theme.of(context).colorScheme.onSurface,
                ),
                onTap: () => onAction(TaskConfirmationAction.reject),
                useIntrinsicWidth: true,
                margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                radius: BorderRadius.circular(8),
                hoverColor: Theme.of(context).colorScheme.surfaceContainerHighest,
              ),
              const HSpace(8),
              FlowyButton(
                text: FlowyText.medium(
                  '确认执行',
                  fontSize: 14,
                  color: Colors.white,
                ),
                onTap: () => onAction(TaskConfirmationAction.confirm),
                useIntrinsicWidth: true,
                margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                radius: BorderRadius.circular(8),
                backgroundColor: Theme.of(context).colorScheme.primary,
                hoverColor: Theme.of(context).colorScheme.primary.withValues(alpha: 0.8),
              ),
            ],
          ),
        ],
      ),
    );
  }

  /// 构建可滚动的任务规划详情内容
  Widget _buildScrollableTaskPlanDetails(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24.0),
      child: _buildTaskPlanDetails(context),
    );
  }

  /// 构建任务规划详情内容
  Widget _buildTaskPlanDetails(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // 用户查询
        _buildSection(
          context,
          title: '用户查询',
          content: taskPlan.userQuery,
        ),
        const VSpace(16),
        
        // 整体策略
        _buildSection(
          context,
          title: '执行策略',
          content: taskPlan.overallStrategy,
        ),
        const VSpace(16),
        
        // 执行步骤
        _buildStepsSection(context),
        const VSpace(16),
        
        // 所需工具
        _buildRequiredToolsSection(context),
        const VSpace(16),
        
        // 预估时间
        _buildEstimatedTimeSection(context),
      ],
    );
  }

  /// 构建通用信息段落
  Widget _buildSection(
    BuildContext context, {
    required String title,
    required String content,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          title,
          fontSize: 14,
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(8),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: Theme.of(context).dividerColor,
              width: 1,
            ),
          ),
          child: FlowyText.regular(
            content,
            fontSize: 13,
            maxLines: null,
            color: AFThemeExtension.of(context).textColor,
          ),
        ),
      ],
    );
  }

  /// 构建执行步骤段落
  Widget _buildStepsSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          '执行步骤',
          fontSize: 14,
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(8),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: Theme.of(context).dividerColor,
              width: 1,
            ),
          ),
          child: Column(
            children: taskPlan.steps.asMap().entries.map((entry) {
              final index = entry.key;
              final step = entry.value;
              return _buildStepItem(context, index + 1, step);
            }).toList(),
          ),
        ),
      ],
    );
  }

  /// 构建单个步骤项
  Widget _buildStepItem(BuildContext context, int stepNumber, TaskStep step) {
    return Padding(
      padding: EdgeInsets.only(bottom: stepNumber < taskPlan.steps.length ? 12 : 0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 步骤编号
          Container(
            width: 24,
            height: 24,
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primary,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Center(
              child: FlowyText.medium(
                stepNumber.toString(),
                fontSize: 12,
                color: Colors.white,
              ),
            ),
          ),
          const HSpace(12),
          
          // 步骤内容
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  step.description,
                  fontSize: 13,
                  maxLines: null,
                  color: AFThemeExtension.of(context).textColor,
                ),
                const VSpace(4),
                Row(
                  children: [
                    FlowyText.regular(
                      '工具: ',
                      fontSize: 12,
                      color: AFThemeExtension.of(context).secondaryTextColor,
                    ),
                    FlowyText.medium(
                      step.mcpToolId ?? step.mcpEndpointId ?? 'AI自动选择',
                      fontSize: 12,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    if (step.estimatedDurationSeconds > 0) ...[
                      const HSpace(16),
                      FlowyText.regular(
                        '预计时长: ',
                        fontSize: 12,
                        color: AFThemeExtension.of(context).secondaryTextColor,
                      ),
                      FlowyText.medium(
                        _formatDuration(step.estimatedDurationSeconds),
                        fontSize: 12,
                        color: AFThemeExtension.of(context).textColor,
                      ),
                    ],
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// 构建所需端点段落
  Widget _buildRequiredToolsSection(BuildContext context) {
    if (taskPlan.requiredMcpEndpoints.isEmpty) {
      return const SizedBox.shrink();
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          '所需端点',
          fontSize: 14,
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: taskPlan.requiredMcpEndpoints.map((endpointId) {
            return Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(
                  color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.3),
                  width: 1,
                ),
              ),
              child: FlowyText.medium(
                endpointId,
                fontSize: 12,
                color: Theme.of(context).colorScheme.primary,
              ),
            );
          }).toList(),
        ),
      ],
    );
  }

  /// 构建预估时间段落
  Widget _buildEstimatedTimeSection(BuildContext context) {
    if (taskPlan.estimatedDurationSeconds <= 0) {
      return const SizedBox.shrink();
    }

    return Row(
      children: [
        FlowyText.medium(
          '预计总时长',
          fontSize: 14,
          color: AFThemeExtension.of(context).strongText,
        ),
        const HSpace(8),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.secondaryContainer,
            borderRadius: BorderRadius.circular(16),
          ),
          child: FlowyText.medium(
            _formatDuration(taskPlan.estimatedDurationSeconds),
            fontSize: 13,
            color: Theme.of(context).colorScheme.onSecondaryContainer,
          ),
        ),
      ],
    );
  }

  /// 格式化持续时间
  String _formatDuration(int seconds) {
    if (seconds < 60) {
      return '${seconds}s';
    } else if (seconds < 3600) {
      final minutes = seconds ~/ 60;
      final remainingSeconds = seconds % 60;
      if (remainingSeconds == 0) {
        return '${minutes}m';
      }
      return '${minutes}m ${remainingSeconds}s';
    } else {
      final hours = seconds ~/ 3600;
      final remainingMinutes = (seconds % 3600) ~/ 60;
      if (remainingMinutes == 0) {
        return '${hours}h';
      }
      return '${hours}h ${remainingMinutes}m';
    }
  }
}

/// 显示任务确认对话框的便捷函数
Future<TaskConfirmationAction?> showTaskConfirmationDialog({
  required BuildContext context,
  required TaskPlan taskPlan,
}) {
  return showDialog<TaskConfirmationAction>(
    context: context,
    barrierDismissible: true, // 允许点击遮罩关闭
    builder: (context) {
      return PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, result) async {
          if (!didPop) {
            // 当用户尝试关闭对话框时（点击遮罩或返回键），显示确认对话框
            final shouldClose = await _showCloseConfirmationDialog(context);
            if (shouldClose && context.mounted) {
              Navigator.of(context).pop(TaskConfirmationAction.reject);
            }
          }
        },
        child: TaskConfirmationDialog(
          taskPlan: taskPlan,
          onAction: (action) async {
            if (action == TaskConfirmationAction.confirm) {
              // 确认执行，直接关闭
              if (context.mounted) {
                Navigator.of(context).pop(action);
              }
            } else if (action == TaskConfirmationAction.reject) {
              // 取消操作，显示确认对话框
              final shouldClose = await _showCloseConfirmationDialog(context);
              if (shouldClose && context.mounted) {
                Navigator.of(context).pop(action);
              }
            } else {
              // 修改任务，直接关闭
              if (context.mounted) {
                Navigator.of(context).pop(action);
              }
            }
          },
        ),
      );
    },
  );
}

/// 显示关闭确认对话框
Future<bool> _showCloseConfirmationDialog(BuildContext context) async {
  return await showDialog<bool>(
    context: context,
    barrierDismissible: true,
    builder: (context) {
      return AlertDialog(
        title: const Text('确认关闭'),
        content: const Text('您确定要关闭任务确认窗口吗？未确认的任务规划将会丢失。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('继续编辑'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('确认关闭'),
          ),
        ],
      );
    },
  ) ?? false; // 如果用户点击遮罩关闭确认对话框，默认返回false
}
