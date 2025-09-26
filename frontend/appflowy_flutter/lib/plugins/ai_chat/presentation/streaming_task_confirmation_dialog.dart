import 'package:appflowy/plugins/ai_chat/application/task_planner_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:appflowy/plugins/ai_chat/presentation/task_confirmation_dialog.dart';
import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 流式任务确认对话框
/// 
/// 支持实时显示AI的思考和规划过程
class StreamingTaskConfirmationDialog extends StatefulWidget {
  const StreamingTaskConfirmationDialog({
    super.key,
    required this.taskPlannerBloc,
    required this.onAction,
  });

  /// TaskPlannerBloc实例
  final TaskPlannerBloc taskPlannerBloc;
  
  /// 用户操作回调
  final Future<void> Function(TaskConfirmationAction action) onAction;

  @override
  State<StreamingTaskConfirmationDialog> createState() => _StreamingTaskConfirmationDialogState();
}

class _StreamingTaskConfirmationDialogState extends State<StreamingTaskConfirmationDialog> 
    with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _fadeAnimation;
  
  // AI思考过程的文本
  String _aiThinkingText = '';
  bool _isThinking = true;
  TaskPlan? _finalTaskPlan;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(seconds: 2),
      vsync: this,
    )..repeat();
    _fadeAnimation = CurvedAnimation(
      parent: _animationController,
      curve: Curves.easeIn,
    );
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final screenHeight = MediaQuery.of(context).size.height;
    final maxHeight = screenHeight * 0.8;
    
    return BlocListener<TaskPlannerBloc, TaskPlannerState>(
      bloc: widget.taskPlannerBloc,
      listener: (context, state) {
        // 监听AI思考过程
        if (state.aiThinkingProcess != null) {
          setState(() {
            _aiThinkingText = state.aiThinkingProcess!;
          });
        }
        
        // 监听规划完成
        if (state.status == TaskPlannerStatus.waitingConfirmation && 
            state.currentTaskPlan != null) {
          setState(() {
            _isThinking = false;
            _finalTaskPlan = state.currentTaskPlan;
          });
        }
      },
      child: Dialog(
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
              // 标题
              _buildHeader(context),
              
              // 内容区域
              Flexible(
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 500),
                  child: _isThinking
                      ? _buildThinkingView(context)
                      : _buildTaskPlanView(context),
                ),
              ),
              
              // 底部按钮
              _buildActions(context),
            ],
          ),
        ),
      ),
    );
  }

  /// 构建对话框头部
  Widget _buildHeader(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: AFThemeExtension.of(context).lightGreyHover,
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(12),
          topRight: Radius.circular(12),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const FlowySvg(
                FlowySvgs.ai_summary_generate_s,
                size: Size.square(20),
              ),
              const HSpace(8),
              FlowyText.semibold(
                _isThinking ? 'AI 正在规划任务...' : '任务规划确认',
                fontSize: 16,
              ),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.close, size: 20),
                onPressed: () async {
                  final shouldClose = await _showCloseConfirmationDialog(context);
                  if (shouldClose && mounted) {
                    Navigator.of(context).pop(TaskConfirmationAction.reject);
                  }
                },
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            _isThinking 
                ? 'AI正在分析您的需求并制定执行计划...'
                : '请确认以下任务规划是否符合您的需求',
            fontSize: 13,
            color: AFThemeExtension.of(context).secondaryTextColor,
            maxLines: 2,
          ),
        ],
      ),
    );
  }

  /// 构建AI思考过程视图
  Widget _buildThinkingView(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // AI思考动画
          Center(
            child: Column(
              children: [
                _buildThinkingAnimation(),
                const VSpace(20),
                FlowyText.medium(
                  'AI 正在分析您的需求',
                  fontSize: 14,
                  color: AFThemeExtension.of(context).secondaryTextColor,
                ),
              ],
            ),
          ),
          const VSpace(30),
          
          // AI思考过程文本
          if (_aiThinkingText.isNotEmpty) ...[
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: AFThemeExtension.of(context).lightGreyHover,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: AFThemeExtension.of(context).borderColor,
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  FlowyText.medium(
                    'AI 思考过程：',
                    fontSize: 12,
                    color: AFThemeExtension.of(context).secondaryTextColor,
                  ),
                  const VSpace(8),
                  AnimatedBuilder(
                    animation: _animationController,
                    builder: (context, child) {
                      return FadeTransition(
                        opacity: _fadeAnimation,
                        child: FlowyText.regular(
                          _aiThinkingText,
                          fontSize: 12,
                          maxLines: 10,
                          overflow: TextOverflow.ellipsis,
                        ),
                      );
                    },
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  /// 构建思考动画
  Widget _buildThinkingAnimation() {
    return SizedBox(
      width: 60,
      height: 60,
      child: Stack(
        alignment: Alignment.center,
        children: [
          // 外圈旋转
          AnimatedBuilder(
            animation: _animationController,
            builder: (context, child) {
              return Transform.rotate(
                angle: _animationController.value * 2 * 3.14159,
                child: Container(
                  width: 60,
                  height: 60,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    border: Border.all(
                      color: Theme.of(context).colorScheme.primary.withOpacity(0.3),
                      width: 2,
                    ),
                  ),
                  child: Align(
                    alignment: Alignment.topCenter,
                    child: Container(
                      width: 8,
                      height: 8,
                      margin: const EdgeInsets.all(4),
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: Theme.of(context).colorScheme.primary,
                      ),
                    ),
                  ),
                ),
              );
            },
          ),
          // 中心图标
          Icon(
            Icons.psychology,
            size: 30,
            color: Theme.of(context).colorScheme.primary,
          ),
        ],
      ),
    );
  }

  /// 构建任务规划视图
  Widget _buildTaskPlanView(BuildContext context) {
    if (_finalTaskPlan == null) {
      return const Center(child: CircularProgressIndicator());
    }
    
    return SingleChildScrollView(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 用户查询
          _buildSection(
            context: context,
            title: '您的需求',
            icon: FlowySvgs.ai_text_s,
            content: FlowyText.regular(
              _finalTaskPlan!.userQuery,
              fontSize: 13,
              maxLines: null,
            ),
          ),
          const VSpace(20),
          
          // 总体策略
          _buildSection(
            context: context,
            title: 'AI 规划策略',
            icon: FlowySvgs.ai_summary_generate_s,
            content: FlowyText.regular(
              _finalTaskPlan!.overallStrategy,
              fontSize: 13,
              maxLines: null,
            ),
          ),
          const VSpace(20),
          
          // 执行步骤
          _buildSection(
            context: context,
            title: '执行步骤',
            icon: FlowySvgs.ai_page_s,
            content: _buildStepsList(context),
          ),
          
          // 预计时长
          if (_finalTaskPlan!.estimatedDurationSeconds > 0) ...[
            const VSpace(20),
            _buildEstimatedTime(context),
          ],
        ],
      ),
    );
  }

  /// 构建内容区块
  Widget _buildSection({
    required BuildContext context,
    required String title,
    required FlowySvgData icon,
    required Widget content,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            FlowySvg(icon, size: const Size.square(16)),
            const HSpace(8),
            FlowyText.semibold(title, fontSize: 14),
          ],
        ),
        const VSpace(12),
        content,
      ],
    );
  }

  /// 构建步骤列表
  Widget _buildStepsList(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (int i = 0; i < _finalTaskPlan!.steps.length; i++)
          _buildStepItem(context, i + 1, _finalTaskPlan!.steps[i]),
      ],
    );
  }

  /// 构建单个步骤项
  Widget _buildStepItem(BuildContext context, int stepNumber, TaskStep step) {
    return Padding(
      padding: EdgeInsets.only(bottom: stepNumber < _finalTaskPlan!.steps.length ? 12 : 0),
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
                // 显示工具信息
                _buildToolInfo(context, step),
                if (step.estimatedDurationSeconds > 0) ...[
                  const VSpace(4),
                  Row(
                    children: [
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
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// 构建工具信息显示
  Widget _buildToolInfo(BuildContext context, TaskStep step) {
    final selectionReason = step.parameters['selection_reason'] as String?;
    final awaitAISelection = step.parameters['await_ai_selection'] as bool? ?? false;
    
    String toolDisplay;
    if (step.mcpToolId != null) {
      toolDisplay = step.mcpToolId!;
    } else if (step.mcpEndpointId != null) {
      if (awaitAISelection) {
        toolDisplay = '${step.mcpEndpointId} (待AI选择具体工具)';
      } else {
        toolDisplay = step.mcpEndpointId!;
      }
    } else {
      toolDisplay = 'AI助手';
    }
    
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            FlowyText.regular(
              '工具: ',
              fontSize: 12,
              color: AFThemeExtension.of(context).secondaryTextColor,
            ),
            Expanded(
              child: FlowyText.medium(
                toolDisplay,
                fontSize: 12,
                color: Theme.of(context).colorScheme.primary,
                maxLines: null,
              ),
            ),
          ],
        ),
        
        if (selectionReason != null && selectionReason.isNotEmpty) ...[
          const VSpace(4),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.regular(
                '选择理由: ',
                fontSize: 11,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
              Expanded(
                child: FlowyText.regular(
                  selectionReason,
                  fontSize: 11,
                  color: AFThemeExtension.of(context).textColor.withOpacity(0.8),
                  maxLines: null,
                ),
              ),
            ],
          ),
        ],
        
        if (step.objective.isNotEmpty) ...[
          const VSpace(4),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.regular(
                '目标: ',
                fontSize: 11,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
              Expanded(
                child: FlowyText.regular(
                  step.objective,
                  fontSize: 11,
                  color: AFThemeExtension.of(context).textColor.withOpacity(0.8),
                  maxLines: null,
                ),
              ),
            ],
          ),
        ],
      ],
    );
  }

  /// 构建预计时长
  Widget _buildEstimatedTime(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: AFThemeExtension.of(context).lightGreyHover,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.schedule, size: 16),
          const HSpace(8),
          FlowyText.regular(
            '预计总时长: ',
            fontSize: 13,
            color: AFThemeExtension.of(context).secondaryTextColor,
          ),
          FlowyText.semibold(
            _formatDuration(_finalTaskPlan!.estimatedDurationSeconds),
            fontSize: 13,
            color: AFThemeExtension.of(context).textColor,
          ),
        ],
      ),
    );
  }

  /// 构建底部操作按钮
  Widget _buildActions(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(
            color: AFThemeExtension.of(context).borderColor,
          ),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          // 取消按钮
          FlowyTextButton(
            '取消',
            fillColor: Colors.transparent,
            hoverColor: AFThemeExtension.of(context).lightGreyHover,
            fontColor: AFThemeExtension.of(context).textColor,
            onPressed: () async {
              await widget.onAction(TaskConfirmationAction.reject);
            },
          ),
          const HSpace(12),
          
          // 修改按钮（当前不可用）
          FlowyTextButton(
            '修改',
            fillColor: Colors.transparent,
            hoverColor: AFThemeExtension.of(context).lightGreyHover,
            fontColor: AFThemeExtension.of(context).textColor,
            onPressed: null, // null makes it disabled
          ),
          const HSpace(12),
          
          // 确认按钮
          FlowyTextButton(
            _isThinking ? '规划中...' : '确认执行',
            fillColor: Theme.of(context).colorScheme.primary,
            hoverColor: Theme.of(context).colorScheme.primary.withOpacity(0.9),
            fontColor: Colors.white,
            onPressed: _isThinking ? null : () async {
              await widget.onAction(TaskConfirmationAction.confirm);
            },
          ),
        ],
      ),
    );
  }

  /// 格式化时长
  String _formatDuration(int seconds) {
    if (seconds < 60) {
      return '$seconds秒';
    } else if (seconds < 3600) {
      final minutes = seconds ~/ 60;
      final remainingSeconds = seconds % 60;
      return remainingSeconds > 0 ? '$minutes分${remainingSeconds}秒' : '$minutes分钟';
    } else {
      final hours = seconds ~/ 3600;
      final remainingMinutes = (seconds % 3600) ~/ 60;
      return remainingMinutes > 0 ? '$hours小时${remainingMinutes}分钟' : '$hours小时';
    }
  }

  /// 显示关闭确认对话框
  Future<bool> _showCloseConfirmationDialog(BuildContext context) async {
    return await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('确认取消'),
        content: const Text('确定要取消这次任务规划吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('继续规划'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('确认取消'),
          ),
        ],
      ),
    ) ?? false;
  }
}

/// 显示流式任务确认对话框
Future<TaskConfirmationAction?> showStreamingTaskConfirmationDialog({
  required BuildContext context,
  required TaskPlannerBloc taskPlannerBloc,
}) {
  return showDialog<TaskConfirmationAction>(
    context: context,
    barrierDismissible: false,
    builder: (context) {
      return PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, result) async {
          if (!didPop) {
            // 当用户尝试关闭对话框时，显示确认对话框
            final shouldClose = await showDialog<bool>(
              context: context,
              builder: (context) => AlertDialog(
                title: const Text('确认取消'),
                content: const Text('确定要取消这次任务规划吗？'),
                actions: [
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(false),
                    child: const Text('继续规划'),
                  ),
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(true),
                    child: const Text('确认取消'),
                  ),
                ],
              ),
            ) ?? false;
            
            if (shouldClose && context.mounted) {
              Navigator.of(context).pop(TaskConfirmationAction.reject);
            }
          }
        },
        child: StreamingTaskConfirmationDialog(
          taskPlannerBloc: taskPlannerBloc,
          onAction: (action) async {
            if (context.mounted) {
              Navigator.of(context).pop(action);
            }
          },
        ),
      );
    },
  );
}
