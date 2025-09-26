import 'dart:async';

import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart' as log_entities;
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart' as planner_entities;
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:percent_indicator/percent_indicator.dart';

/// 执行进度显示组件
/// 
/// 提供直观的任务执行状态显示，包括：
/// - 实时进度更新
/// - 步骤详情展示
/// - 动画效果
/// - 控制功能（暂停、取消、重试）
/// - 性能优化的渲染
class ExecutionProgressWidget extends StatefulWidget {
  const ExecutionProgressWidget({
    super.key,
    required this.executionBloc,
    this.onCancel,
    this.onRetry,
    this.showDetails = true,
    this.compact = false,
    this.maxHeight,
  });

  /// 执行BLoC实例
  final ExecutionBloc executionBloc;
  
  /// 取消回调
  final VoidCallback? onCancel;
  
  /// 重试回调
  final VoidCallback? onRetry;
  
  /// 是否显示详细信息
  final bool showDetails;
  
  /// 是否使用紧凑模式
  final bool compact;
  
  /// 最大高度限制
  final double? maxHeight;

  @override
  State<ExecutionProgressWidget> createState() => _ExecutionProgressWidgetState();
}

class _ExecutionProgressWidgetState extends State<ExecutionProgressWidget>
    with TickerProviderStateMixin {
  late AnimationController _progressAnimationController;
  late AnimationController _pulseAnimationController;
  late AnimationController _stepAnimationController;
  
  late Animation<double> _progressAnimation;
  late Animation<double> _pulseAnimation;
  late Animation<double> _stepAnimation;
  
  Timer? _updateTimer;
  bool _isExpanded = true;
  double _lastProgress = 0.0;

  @override
  void initState() {
    super.initState();
    _initializeAnimations();
    _startUpdateTimer();
  }

  @override
  void dispose() {
    _progressAnimationController.dispose();
    _pulseAnimationController.dispose();
    _stepAnimationController.dispose();
    _updateTimer?.cancel();
    super.dispose();
  }

  void _initializeAnimations() {
    // 进度条动画控制器
    _progressAnimationController = AnimationController(
      duration: const Duration(milliseconds: 800),
      vsync: this,
    );
    
    // 脉冲动画控制器（用于运行状态指示）
    _pulseAnimationController = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    );
    
    // 步骤切换动画控制器
    _stepAnimationController = AnimationController(
      duration: const Duration(milliseconds: 400),
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

    _stepAnimation = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(
      parent: _stepAnimationController,
      curve: Curves.elasticOut,
    ));

    // 启动脉冲动画循环
    _pulseAnimationController.repeat(reverse: true);
  }

  void _startUpdateTimer() {
    _updateTimer = Timer.periodic(const Duration(milliseconds: 100), (timer) {
      if (mounted) {
        setState(() {
          // 触发重绘以更新时间显示
        });
      }
    });
  }

  void _animateProgress(double newProgress) {
    if (newProgress != _lastProgress) {
      _progressAnimationController.reset();
      _progressAnimation = Tween<double>(
        begin: _lastProgress,
        end: newProgress,
      ).animate(CurvedAnimation(
        parent: _progressAnimationController,
        curve: Curves.easeInOut,
      ));
      _progressAnimationController.forward();
      _lastProgress = newProgress;
    }
  }


  @override
  Widget build(BuildContext context) {
    return BlocBuilder<ExecutionBloc, ExecutionState>(
      bloc: widget.executionBloc,
      builder: (context, state) {
        return Container(
          constraints: widget.maxHeight != null 
              ? BoxConstraints(maxHeight: widget.maxHeight!)
              : null,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surface,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: AFThemeExtension.of(context).borderColor,
              width: 1,
            ),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withOpacity(0.05),
                blurRadius: 8,
                offset: const Offset(0, 2),
              ),
            ],
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildHeader(context, state),
              if (_isExpanded && widget.showDetails) ...[
                _buildProgressSection(context, state),
                if (state is RunningExecutionState || state is PausedExecutionState)
                  _buildStepsSection(context, state),
                _buildControlsSection(context, state),
              ],
            ],
          ),
        );
      },
    );
  }

  Widget _buildHeader(BuildContext context, ExecutionState state) {
    final statusInfo = _getStatusInfo(state);
    
    return Container(
      padding: const EdgeInsets.all(16),
      child: Row(
        children: [
          // 状态图标
          AnimatedBuilder(
            animation: _pulseAnimation,
            builder: (context, child) {
              return Transform.scale(
                scale: statusInfo.isRunning ? _pulseAnimation.value : 1.0,
                child: Container(
                  width: 32,
                  height: 32,
                  decoration: BoxDecoration(
                    color: statusInfo.color.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: Icon(
                    statusInfo.icon,
                    color: statusInfo.color,
                    size: 18,
                  ),
                ),
              );
            },
          ),
          const SizedBox(width: 12),
          
          // 状态文本
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  statusInfo.title,
                  fontSize: 14,
                  color: statusInfo.color,
                ),
                if (statusInfo.subtitle != null) ...[
                  const SizedBox(height: 2),
                  FlowyText.regular(
                    statusInfo.subtitle!,
                    fontSize: 12,
                    color: AFThemeExtension.of(context).textColor,
                  ),
                ],
              ],
            ),
          ),
          
          // 展开/折叠按钮
          if (widget.showDetails)
            FlowyIconButton(
              icon: FlowySvg(
                _isExpanded ? FlowySvgs.m_expand_s : FlowySvgs.m_expand_s,
                size: const Size.square(16),
              ),
              onPressed: () {
                setState(() {
                  _isExpanded = !_isExpanded;
                });
              },
            ),
        ],
      ),
    );
  }

  Widget _buildProgressSection(BuildContext context, ExecutionState state) {
    final progress = _getProgress(state);
    final percentage = progress.percentage;
    
    // 触发进度动画
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _animateProgress(percentage);
    });

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        children: [
          // 进度条
          Row(
            children: [
              Expanded(
                child: AnimatedBuilder(
                  animation: _progressAnimation,
                  builder: (context, child) {
                    return LinearPercentIndicator(
                      lineHeight: 8.0,
                      percent: _progressAnimation.value.clamp(0.0, 1.0),
                      padding: EdgeInsets.zero,
                      progressColor: _getProgressColor(state),
                      backgroundColor: AFThemeExtension.of(context).progressBarBGColor,
                      barRadius: const Radius.circular(4),
                      animation: false, // 我们使用自定义动画
                    );
                  },
                ),
              ),
              const SizedBox(width: 12),
              SizedBox(
                width: 50,
                child: FlowyText.medium(
                  "${(percentage * 100).round()}%",
                  fontSize: 12,
                  textAlign: TextAlign.end,
                ),
              ),
            ],
          ),
          
          const SizedBox(height: 8),
          
          // 进度信息
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              FlowyText.regular(
                "${progress.currentStep}/${progress.totalSteps} 步骤",
                fontSize: 11,
                color: AFThemeExtension.of(context).textColor,
              ),
              if (progress.estimatedRemainingSeconds != null)
                FlowyText.regular(
                  "预计剩余: ${_formatDuration(Duration(seconds: progress.estimatedRemainingSeconds!))}",
                  fontSize: 11,
                  color: AFThemeExtension.of(context).textColor,
                ),
            ],
          ),
          
          const SizedBox(height: 16),
        ],
      ),
    );
  }

  Widget _buildStepsSection(BuildContext context, ExecutionState state) {
    final executionLog = _getExecutionLog(state);
    if (executionLog == null || executionLog.steps.isEmpty) {
      return const SizedBox.shrink();
    }

    final visibleSteps = widget.compact 
        ? executionLog.steps.take(3).toList()
        : executionLog.steps;

    return Container(
      constraints: const BoxConstraints(maxHeight: 200),
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Row(
              children: [
                FlowyText.medium(
                  "执行步骤",
                  fontSize: 12,
                  color: AFThemeExtension.of(context).textColor,
                ),
                const Spacer(),
                if (executionLog.steps.length > 3 && widget.compact)
                  FlowyText.regular(
                    "+${executionLog.steps.length - 3} 更多",
                    fontSize: 11,
                    color: AFThemeExtension.of(context).textColor,
                  ),
              ],
            ),
          ),
          const SizedBox(height: 8),
          
          Flexible(
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: visibleSteps.length,
              itemBuilder: (context, index) {
                final step = visibleSteps[index];
                return AnimatedBuilder(
                  animation: _stepAnimation,
                  builder: (context, child) {
                    return Transform.translate(
                      offset: Offset(0, (1 - _stepAnimation.value) * 20),
                      child: Opacity(
                        opacity: _stepAnimation.value,
                        child: _buildStepItem(context, step, index),
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStepItem(BuildContext context, log_entities.ExecutionStep step, int index) {
    final stepStatusInfo = _getStepStatusInfo(step.status);
    
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: stepStatusInfo.color.withOpacity(0.05),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: stepStatusInfo.color.withOpacity(0.2),
          width: 1,
        ),
      ),
      child: Row(
        children: [
          // 步骤序号
          Container(
            width: 24,
            height: 24,
            decoration: BoxDecoration(
              color: stepStatusInfo.color,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Center(
              child: step.status == log_entities.ExecutionStepStatus.executing
                  ? SizedBox(
                      width: 12,
                      height: 12,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        valueColor: AlwaysStoppedAnimation<Color>(
                          Theme.of(context).colorScheme.surface,
                        ),
                      ),
                    )
                  : Icon(
                      stepStatusInfo.icon,
                      color: Theme.of(context).colorScheme.surface,
                      size: 12,
                    ),
            ),
          ),
          const SizedBox(width: 12),
          
          // 步骤信息
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  step.name,
                  fontSize: 12,
                ),
                if (step.description.isNotEmpty) ...[
                  const SizedBox(height: 2),
                  FlowyText.regular(
                    step.description,
                    fontSize: 11,
                    color: AFThemeExtension.of(context).textColor,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
                if (step.executionTimeMs > 0) ...[
                  const SizedBox(height: 2),
                  FlowyText.regular(
                    "耗时: ${step.executionTimeMs}ms",
                    fontSize: 10,
                    color: AFThemeExtension.of(context).textColor,
                  ),
                ],
              ],
            ),
          ),
          
          // 工具信息
          if (step.mcpTool.name.isNotEmpty)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: AFThemeExtension.of(context).borderColor,
              borderRadius: BorderRadius.circular(4),
            ),
              child: FlowyText.regular(
                step.mcpTool.name,
                fontSize: 10,
                color: AFThemeExtension.of(context).textColor,
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildControlsSection(BuildContext context, ExecutionState state) {
    final controls = _getAvailableControls(state);
    if (controls.isEmpty) return const SizedBox.shrink();

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(
            color: AFThemeExtension.of(context).borderColor,
            width: 1,
          ),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: controls.map((control) {
          return Padding(
            padding: const EdgeInsets.only(left: 8),
            child: FlowyButton(
              text: FlowyText.regular(
                control.label,
                fontSize: 12,
                color: control.isDestructive 
                    ? Theme.of(context).colorScheme.error
                    : Theme.of(context).colorScheme.primary,
              ),
              onTap: control.onTap,
              leftIcon: control.icon != null 
                  ? FlowySvg(
                      control.icon!,
                      size: const Size.square(14),
                      color: control.isDestructive 
                          ? Theme.of(context).colorScheme.error
                          : Theme.of(context).colorScheme.primary,
                    )
                  : null,
              hoverColor: control.isDestructive 
                  ? Theme.of(context).colorScheme.error.withOpacity(0.1)
                  : Theme.of(context).colorScheme.primary.withOpacity(0.1),
            ),
          );
        }).toList(),
      ),
    );
  }

  // 辅助方法

  _StatusInfo _getStatusInfo(ExecutionState state) {
    switch (state.runtimeType) {
      case InitialExecutionState:
        return _StatusInfo(
          title: "准备就绪",
          icon: Icons.play_circle_outline,
          color: AFThemeExtension.of(context).textColor,
          isRunning: false,
        );
      case PreparingExecutionState:
        return _StatusInfo(
          title: "准备中...",
          icon: Icons.hourglass_empty,
          color: Theme.of(context).colorScheme.primary,
          isRunning: true,
        );
      case RunningExecutionState:
        final runningState = state as RunningExecutionState;
        return _StatusInfo(
          title: "执行中",
          subtitle: runningState.progress.currentStepDescription.isNotEmpty 
              ? runningState.progress.currentStepDescription
              : null,
          icon: Icons.play_circle,
          color: Theme.of(context).colorScheme.primary,
          isRunning: true,
        );
      case PausedExecutionState:
        return _StatusInfo(
          title: "已暂停",
          icon: Icons.pause_circle,
          color: Colors.orange,
          isRunning: false,
        );
      case CompletedExecutionState:
        return _StatusInfo(
          title: "执行完成",
          icon: Icons.check_circle,
          color: AFThemeExtension.of(context).success ?? Colors.green,
          isRunning: false,
        );
      case ErrorExecutionState:
        final errorState = state as ErrorExecutionState;
        return _StatusInfo(
          title: "执行失败",
          subtitle: errorState.error,
          icon: Icons.error,
          color: Theme.of(context).colorScheme.error,
          isRunning: false,
        );
      case CancelledExecutionState:
        return _StatusInfo(
          title: "已取消",
          icon: Icons.cancel,
          color: AFThemeExtension.of(context).textColor,
          isRunning: false,
        );
      default:
        return _StatusInfo(
          title: "未知状态",
          icon: Icons.help_outline,
          color: AFThemeExtension.of(context).textColor,
          isRunning: false,
        );
    }
  }

  planner_entities.ExecutionProgress _getProgress(ExecutionState state) {
    switch (state.runtimeType) {
      case RunningExecutionState:
        return (state as RunningExecutionState).progress;
      case PausedExecutionState:
        return (state as PausedExecutionState).progress;
      case CompletedExecutionState:
        final completedState = state as CompletedExecutionState;
        return planner_entities.ExecutionProgress(
          currentStep: completedState.executionLog.totalSteps,
          totalSteps: completedState.executionLog.totalSteps,
          status: planner_entities.ExecutionStatus.completed,
        );
      default:
        return const planner_entities.ExecutionProgress();
    }
  }

  log_entities.ExecutionLog? _getExecutionLog(ExecutionState state) {
    switch (state.runtimeType) {
      case RunningExecutionState:
        return (state as RunningExecutionState).executionLog;
      case PausedExecutionState:
        return (state as PausedExecutionState).executionLog;
      case CompletedExecutionState:
        return (state as CompletedExecutionState).executionLog;
      default:
        return null;
    }
  }

  Color _getProgressColor(ExecutionState state) {
    switch (state.runtimeType) {
      case RunningExecutionState:
        return Theme.of(context).colorScheme.primary;
      case PausedExecutionState:
        return Colors.orange;
      case CompletedExecutionState:
        return AFThemeExtension.of(context).success ?? Colors.green;
      case ErrorExecutionState:
        return Theme.of(context).colorScheme.error;
      default:
        return AFThemeExtension.of(context).textColor;
    }
  }

  _StepStatusInfo _getStepStatusInfo(log_entities.ExecutionStepStatus status) {
    switch (status) {
      case log_entities.ExecutionStepStatus.pending:
        return _StepStatusInfo(
          icon: Icons.schedule,
          color: AFThemeExtension.of(context).textColor,
        );
      case log_entities.ExecutionStepStatus.executing:
        return _StepStatusInfo(
          icon: Icons.play_arrow,
          color: Theme.of(context).colorScheme.primary,
        );
      case log_entities.ExecutionStepStatus.success:
        return _StepStatusInfo(
          icon: Icons.check,
          color: AFThemeExtension.of(context).success ?? Colors.green,
        );
      case log_entities.ExecutionStepStatus.error:
        return _StepStatusInfo(
          icon: Icons.close,
          color: Theme.of(context).colorScheme.error,
        );
      case log_entities.ExecutionStepStatus.skipped:
        return _StepStatusInfo(
          icon: Icons.skip_next,
          color: Colors.orange,
        );
      case log_entities.ExecutionStepStatus.timeout:
        return _StepStatusInfo(
          icon: Icons.access_time,
          color: Theme.of(context).colorScheme.error,
        );
      case log_entities.ExecutionStepStatus.cancelled:
        return _StepStatusInfo(
          icon: Icons.cancel,
          color: AFThemeExtension.of(context).textColor,
        );
    }
  }

  List<_ControlInfo> _getAvailableControls(ExecutionState state) {
    final controls = <_ControlInfo>[];

    switch (state.runtimeType) {
      case RunningExecutionState:
        controls.addAll([
          _ControlInfo(
            label: "暂停",
            icon: FlowySvgs.close_s, // 使用可用的图标
            onTap: () => widget.executionBloc.add(const PauseExecutionEvent()),
          ),
          _ControlInfo(
            label: "取消",
            icon: FlowySvgs.close_s,
            isDestructive: true,
            onTap: () => _showCancelDialog(context),
          ),
        ]);
        break;
      case PausedExecutionState:
        controls.addAll([
          _ControlInfo(
            label: "继续",
            icon: FlowySvgs.add_s, // 使用可用的图标
            onTap: () => widget.executionBloc.add(const ResumeExecutionEvent()),
          ),
          _ControlInfo(
            label: "取消",
            icon: FlowySvgs.close_s,
            isDestructive: true,
            onTap: () => _showCancelDialog(context),
          ),
        ]);
        break;
      case ErrorExecutionState:
        final errorState = state as ErrorExecutionState;
        if (errorState.canRetry) {
          controls.add(
            _ControlInfo(
              label: "重试",
              icon: FlowySvgs.add_s, // 使用可用的图标
              onTap: () {
                widget.executionBloc.add(const RetryExecutionEvent());
                widget.onRetry?.call();
              },
            ),
          );
        }
        break;
    }

    return controls;
  }

  void _showCancelDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text("取消执行"),
        content: const Text("确定要取消当前执行吗？此操作无法撤销。"),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text("取消"),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(dialogContext).pop();
              widget.executionBloc.add(const CancelExecutionEvent(reason: "用户取消"));
              widget.onCancel?.call();
            },
            child: Text(
              "确定取消",
              style: TextStyle(
                color: Theme.of(context).colorScheme.error,
              ),
            ),
          ),
        ],
      ),
    );
  }

  String _formatDuration(Duration duration) {
    if (duration.inHours > 0) {
      return "${duration.inHours}h ${duration.inMinutes.remainder(60)}m";
    } else if (duration.inMinutes > 0) {
      return "${duration.inMinutes}m ${duration.inSeconds.remainder(60)}s";
    } else {
      return "${duration.inSeconds}s";
    }
  }
}

// 辅助数据类

class _StatusInfo {
  const _StatusInfo({
    required this.title,
    this.subtitle,
    required this.icon,
    required this.color,
    required this.isRunning,
  });

  final String title;
  final String? subtitle;
  final IconData icon;
  final Color color;
  final bool isRunning;
}

class _StepStatusInfo {
  const _StepStatusInfo({
    required this.icon,
    required this.color,
  });

  final IconData icon;
  final Color color;
}

class _ControlInfo {
  const _ControlInfo({
    required this.label,
    this.icon,
    required this.onTap,
    this.isDestructive = false,
  });

  final String label;
  final FlowySvgData? icon;
  final VoidCallback onTap;
  final bool isDestructive;
}
