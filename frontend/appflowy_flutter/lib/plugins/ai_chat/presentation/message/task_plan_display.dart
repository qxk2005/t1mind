import 'package:flutter/material.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';

/// 任务规划显示组件
/// 
/// 显示AI创建的任务计划及其执行进度
class TaskPlanDisplay extends StatelessWidget {
  const TaskPlanDisplay({
    super.key,
    required this.plan,
  });

  final TaskPlanInfo plan;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Container(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [
              Colors.purple.withOpacity(0.05),
              Colors.blue.withOpacity(0.05),
            ],
          ),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: Colors.purple.withOpacity(0.3),
            width: 1.5,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 计划头部
            _PlanHeader(plan: plan),
            const Divider(height: 1),
            // 步骤列表
            _PlanStepsList(steps: plan.steps),
            // 计划底部（进度条）
            if (plan.status != TaskPlanStatus.pending)
              _PlanFooter(plan: plan),
          ],
        ),
      ),
    );
  }
}

/// 计划头部
class _PlanHeader extends StatelessWidget {
  const _PlanHeader({required this.plan});

  final TaskPlanInfo plan;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Row(
        children: [
          // 状态图标
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: _getStatusColor().withOpacity(0.2),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(
              _getStatusIcon(),
              size: 24,
              color: _getStatusColor(),
            ),
          ),
          const HSpace(12),
          // 计划信息
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  plan.goal,
                  style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: Theme.of(context).colorScheme.onSurface,
                  ),
                ),
                const VSpace(4),
                Text(
                  _getStatusText(),
                  style: TextStyle(
                    fontSize: 12,
                    color: AFThemeExtension.of(context).textColor.withOpacity(0.6),
                  ),
                ),
              ],
            ),
          ),
          // 步骤计数
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              color: Colors.purple.withOpacity(0.1),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Text(
              '${plan.completedSteps}/${plan.steps.length} 步骤',
              style: const TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: Colors.purple,
              ),
            ),
          ),
        ],
      ),
    );
  }

  IconData _getStatusIcon() {
    switch (plan.status) {
      case TaskPlanStatus.pending:
        return Icons.pending_outlined;
      case TaskPlanStatus.running:
        return Icons.play_circle_outline;
      case TaskPlanStatus.completed:
        return Icons.check_circle_outline;
      case TaskPlanStatus.failed:
        return Icons.error_outline;
      case TaskPlanStatus.cancelled:
        return Icons.cancel_outlined;
    }
  }

  Color _getStatusColor() {
    switch (plan.status) {
      case TaskPlanStatus.pending:
        return Colors.grey;
      case TaskPlanStatus.running:
        return Colors.blue;
      case TaskPlanStatus.completed:
        return Colors.green;
      case TaskPlanStatus.failed:
        return Colors.red;
      case TaskPlanStatus.cancelled:
        return Colors.orange;
    }
  }

  String _getStatusText() {
    switch (plan.status) {
      case TaskPlanStatus.pending:
        return '等待开始';
      case TaskPlanStatus.running:
        return '正在执行...';
      case TaskPlanStatus.completed:
        return '已完成';
      case TaskPlanStatus.failed:
        return '执行失败';
      case TaskPlanStatus.cancelled:
        return '已取消';
    }
  }
}

/// 步骤列表
class _PlanStepsList extends StatelessWidget {
  const _PlanStepsList({required this.steps});

  final List<TaskStepInfo> steps;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      child: Column(
        children: List.generate(
          steps.length,
          (index) => _TaskStepItem(
            step: steps[index],
            stepNumber: index + 1,
            isLast: index == steps.length - 1,
          ),
        ),
      ),
    );
  }
}

/// 单个任务步骤
class _TaskStepItem extends StatelessWidget {
  const _TaskStepItem({
    required this.step,
    required this.stepNumber,
    required this.isLast,
  });

  final TaskStepInfo step;
  final int stepNumber;
  final bool isLast;

  @override
  Widget build(BuildContext context) {
    return IntrinsicHeight(
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 步骤指示器和连接线
          Column(
            children: [
              // 步骤圆圈
              Container(
                width: 28,
                height: 28,
                decoration: BoxDecoration(
                  color: _getStepColor(),
                  shape: BoxShape.circle,
                  border: Border.all(
                    color: _getStepColor().withOpacity(0.3),
                    width: 2,
                  ),
                ),
                child: _buildStepIndicator(),
              ),
              // 连接线
              if (!isLast)
                Expanded(
                  child: Container(
                    width: 2,
                    margin: const EdgeInsets.symmetric(vertical: 4),
                    decoration: BoxDecoration(
                      color: _getStepColor().withOpacity(0.3),
                    ),
                  ),
                ),
            ],
          ),
          const HSpace(12),
          // 步骤内容
          Expanded(
            child: Padding(
              padding: EdgeInsets.only(bottom: isLast ? 0 : 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // 步骤标题
                  Text(
                    step.description,
                    style: TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                      color: Theme.of(context).colorScheme.onSurface,
                      decoration: step.status == TaskStepStatus.completed
                          ? TextDecoration.lineThrough
                          : null,
                    ),
                  ),
                  // 步骤详情
                  if (step.tools.isNotEmpty) ...[
                    const VSpace(4),
                    Wrap(
                      spacing: 6,
                      runSpacing: 4,
                      children: step.tools.map((tool) {
                        return Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            color: Colors.blue.withOpacity(0.1),
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              const Icon(
                                Icons.build_outlined,
                                size: 12,
                                color: Colors.blue,
                              ),
                              const HSpace(4),
                              Text(
                                tool,
                                style: const TextStyle(
                                  fontSize: 11,
                                  color: Colors.blue,
                                ),
                              ),
                            ],
                          ),
                        );
                      }).toList(),
                    ),
                  ],
                  // 错误信息
                  if (step.error != null) ...[
                    const VSpace(6),
                    Container(
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
                        step.error!,
                        style: const TextStyle(
                          fontSize: 12,
                          color: Colors.red,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStepIndicator() {
    switch (step.status) {
      case TaskStepStatus.pending:
        return Center(
          child: Text(
            '$stepNumber',
            style: const TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w600,
              color: Colors.white,
            ),
          ),
        );
      case TaskStepStatus.running:
        return const Center(
          child: SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
            ),
          ),
        );
      case TaskStepStatus.completed:
        return const Icon(
          Icons.check,
          size: 16,
          color: Colors.white,
        );
      case TaskStepStatus.failed:
        return const Icon(
          Icons.close,
          size: 16,
          color: Colors.white,
        );
    }
  }

  Color _getStepColor() {
    switch (step.status) {
      case TaskStepStatus.pending:
        return Colors.grey;
      case TaskStepStatus.running:
        return Colors.blue;
      case TaskStepStatus.completed:
        return Colors.green;
      case TaskStepStatus.failed:
        return Colors.red;
    }
  }
}

/// 计划底部（进度条）
class _PlanFooter extends StatelessWidget {
  const _PlanFooter({required this.plan});

  final TaskPlanInfo plan;

  @override
  Widget build(BuildContext context) {
    final progress = plan.steps.isEmpty
        ? 0.0
        : plan.completedSteps / plan.steps.length;

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 进度条
          ClipRRect(
            borderRadius: BorderRadius.circular(4),
            child: LinearProgressIndicator(
              value: progress,
              minHeight: 6,
              backgroundColor: Colors.grey.withOpacity(0.2),
              valueColor: AlwaysStoppedAnimation<Color>(
                plan.status == TaskPlanStatus.completed
                    ? Colors.green
                    : Colors.blue,
              ),
            ),
          ),
          const VSpace(8),
          // 进度文本
          Text(
            '${(progress * 100).toInt()}% 完成',
            style: TextStyle(
              fontSize: 12,
              color: AFThemeExtension.of(context).textColor.withOpacity(0.6),
            ),
          ),
        ],
      ),
    );
  }
}

/// 任务计划信息
class TaskPlanInfo {
  const TaskPlanInfo({
    required this.id,
    required this.goal,
    required this.steps,
    required this.status,
  });

  final String id;
  final String goal;
  final List<TaskStepInfo> steps;
  final TaskPlanStatus status;

  int get completedSteps =>
      steps.where((s) => s.status == TaskStepStatus.completed).length;
}

/// 任务步骤信息
class TaskStepInfo {
  const TaskStepInfo({
    required this.id,
    required this.description,
    required this.status,
    this.tools = const [],
    this.error,
  });

  final String id;
  final String description;
  final TaskStepStatus status;
  final List<String> tools;
  final String? error;
}

/// 任务计划状态
enum TaskPlanStatus {
  pending,
  running,
  completed,
  failed,
  cancelled,
}

/// 任务步骤状态
enum TaskStepStatus {
  pending,
  running,
  completed,
  failed,
}


