import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/agent_settings_bloc.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 智能体选择器组件
/// 用于在聊天界面中选择和切换智能体
class AgentSelector extends StatefulWidget {
  const AgentSelector({
    super.key,
    this.selectedAgent,
    this.onAgentSelected,
    this.showStatus = true,
    this.compact = false,
  });

  /// 当前选中的智能体
  final AgentConfigPB? selectedAgent;
  
  /// 智能体选择回调
  final Function(AgentConfigPB?)? onAgentSelected;
  
  /// 是否显示智能体状态
  final bool showStatus;
  
  /// 是否使用紧凑模式
  final bool compact;

  @override
  State<AgentSelector> createState() => _AgentSelectorState();
}

class _AgentSelectorState extends State<AgentSelector> {
  @override
  void initState() {
    super.initState();
    // 初始化时加载智能体列表
    context.read<AgentSettingsBloc>().add(const AgentSettingsEvent.loadAgentList());
  }

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AgentSettingsBloc, AgentSettingsState>(
      builder: (context, state) {
        if (state.isLoading) {
          return _buildLoadingSelector();
        }

        if (state.error != null) {
          return _buildErrorSelector(state.error!);
        }

        return _buildAgentDropdown(state.agents);
      },
    );
  }

  /// 构建加载中的选择器
  Widget _buildLoadingSelector() {
    return Container(
      height: widget.compact ? 32 : 40,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
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
          const SizedBox(width: 8),
          Text(
            '加载智能体...',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }

  /// 构建错误状态的选择器
  Widget _buildErrorSelector(String error) {
    return Container(
      height: widget.compact ? 32 : 40,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).colorScheme.error),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          FlowySvg(
            FlowySvgs.warning_s,
            color: Theme.of(context).colorScheme.error,
            size: const Size(16, 16),
          ),
          const SizedBox(width: 8),
          Flexible(
            child:             Text(
              '加载失败',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                color: Theme.of(context).colorScheme.error,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          const SizedBox(width: 4),
          FlowyTooltip(
            message: error,
            child: FlowyIconButton(
              icon: FlowySvg(
                FlowySvgs.information_s,
                color: Theme.of(context).colorScheme.error,
                size: const Size(16, 16),
              ),
              width: 20,
              onPressed: () {},
            ),
          ),
        ],
      ),
    );
  }

  /// 构建智能体下拉选择器
  Widget _buildAgentDropdown(List<AgentConfigPB> agents) {
    // 添加"无智能体"选项
    final allOptions = <AgentConfigPB?>[null, ...agents];
    
    return PopupMenuButton<AgentConfigPB?>(
      child: _buildSelectedAgentDisplay(),
      itemBuilder: (context) => allOptions.map((agent) {
        return PopupMenuItem<AgentConfigPB?>(
          value: agent,
          child: _buildAgentOption(agent),
        );
      }).toList(),
      onSelected: (agent) {
        widget.onAgentSelected?.call(agent);
      },
    );
  }

  /// 构建当前选中智能体的显示
  Widget _buildSelectedAgentDisplay() {
    final agent = widget.selectedAgent;
    
    return Container(
      constraints: BoxConstraints(
        minHeight: widget.compact ? 32 : 40,
      ),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(8),
        color: Theme.of(context).colorScheme.surface,
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _getAgentIcon(agent),
          const SizedBox(width: 8),
          Flexible(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: [
          Text(
            agent?.name ?? '无智能体',
            style: widget.compact 
              ? Theme.of(context).textTheme.bodySmall
              : Theme.of(context).textTheme.bodyMedium,
            overflow: TextOverflow.ellipsis,
          ),
                if (!widget.compact && widget.showStatus && agent != null)
                  _buildAgentStatus(agent),
              ],
            ),
          ),
          const SizedBox(width: 4),
          Icon(
            Icons.arrow_drop_down,
            color: Theme.of(context).iconTheme.color,
            size: 16,
          ),
        ],
      ),
    );
  }

  /// 构建智能体选项
  Widget _buildAgentOption(AgentConfigPB? agent) {
    if (agent == null) {
      return Row(
        children: [
          _getAgentIcon(null),
          const SizedBox(width: 8),
          Expanded(
            child:             Text(
              '无智能体',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
          ),
        ],
      );
    }

    return Row(
      children: [
        _getAgentIcon(agent),
        const SizedBox(width: 8),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                agent.name,
                style: Theme.of(context).textTheme.bodyMedium,
                overflow: TextOverflow.ellipsis,
              ),
              if (widget.showStatus)
                _buildAgentStatus(agent),
              if (agent.hasDescription() && agent.description.isNotEmpty)
                Text(
                  agent.description,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).textTheme.bodySmall?.color?.withOpacity(0.7),
                  ),
                  overflow: TextOverflow.ellipsis,
                  maxLines: 1,
                ),
            ],
          ),
        ),
      ],
    );
  }

  /// 构建智能体状态显示
  Widget _buildAgentStatus(AgentConfigPB agent) {
    final statusText = context.read<AgentSettingsBloc>().getAgentStatusText(agent.status);
    final statusColor = _getStatusColor(agent.status);
    
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 6,
          height: 6,
          decoration: BoxDecoration(
            color: statusColor,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 4),
        Text(
          statusText,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: statusColor,
            fontSize: 10,
          ),
        ),
      ],
    );
  }

  /// 获取智能体图标
  Widget _getAgentIcon(AgentConfigPB? agent) {
    if (agent == null) {
      return FlowySvg(
        FlowySvgs.ai_chat_logo_s,
        color: Theme.of(context).iconTheme.color?.withOpacity(0.5),
        size: const Size(16, 16),
      );
    }

    // 根据智能体状态显示不同的图标
    switch (agent.status) {
      case AgentStatusPB.AgentActive:
        return FlowySvg(
          FlowySvgs.ai_chat_logo_s,
          color: Theme.of(context).colorScheme.primary,
          size: const Size(16, 16),
        );
      case AgentStatusPB.AgentPaused:
        return FlowySvg(
          FlowySvgs.ai_chat_logo_s,
          color: Theme.of(context).colorScheme.secondary,
          size: const Size(16, 16),
        );
      case AgentStatusPB.AgentDeleted:
        return FlowySvg(
          FlowySvgs.ai_chat_logo_s,
          color: Theme.of(context).disabledColor,
          size: const Size(16, 16),
        );
      default:
        return FlowySvg(
          FlowySvgs.ai_chat_logo_s,
          color: Theme.of(context).iconTheme.color,
          size: const Size(16, 16),
        );
    }
  }

  /// 获取状态颜色
  Color _getStatusColor(AgentStatusPB status) {
    switch (status) {
      case AgentStatusPB.AgentActive:
        return Colors.green;
      case AgentStatusPB.AgentPaused:
        return Colors.orange;
      case AgentStatusPB.AgentDeleted:
        return Colors.red;
      default:
        return Theme.of(context).disabledColor;
    }
  }
}

/// 智能体执行状态显示组件
class AgentExecutionStatus extends StatelessWidget {
  const AgentExecutionStatus({
    super.key,
    required this.agent,
    this.isExecuting = false,
    this.currentTask,
    this.progress,
    this.compact = false,
  });

  /// 当前执行的智能体
  final AgentConfigPB agent;
  
  /// 是否正在执行
  final bool isExecuting;
  
  /// 当前执行的任务描述
  final String? currentTask;
  
  /// 执行进度 (0.0 - 1.0)
  final double? progress;
  
  /// 是否使用紧凑模式
  final bool compact;

  @override
  Widget build(BuildContext context) {
    if (!isExecuting) {
      return const SizedBox.shrink();
    }

    return Container(
      padding: EdgeInsets.all(compact ? 8 : 12),
      margin: const EdgeInsets.symmetric(vertical: 4),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.1),
        border: Border.all(
          color: Theme.of(context).colorScheme.primary.withOpacity(0.3),
        ),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          SizedBox(
            width: compact ? 16 : 20,
            height: compact ? 16 : 20,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              value: progress,
              valueColor: AlwaysStoppedAnimation<Color>(
                Theme.of(context).colorScheme.primary,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '智能体 ${agent.name} 正在执行...',
                  style: compact 
                    ? Theme.of(context).textTheme.bodySmall
                    : Theme.of(context).textTheme.bodyMedium,
                ),
                if (currentTask != null && currentTask!.isNotEmpty)
                  Text(
                    currentTask!,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).textTheme.bodySmall?.color?.withOpacity(0.7),
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
