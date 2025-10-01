import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/agent_settings_bloc.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy/workspace/presentation/settings/workspace/widgets/agent_dialog.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 工作空间级别的智能体配置组件
/// 支持配置作用域管理和权限控制
class WorkspaceAgentSettings extends StatelessWidget {
  const WorkspaceAgentSettings({
    super.key,
    required this.userProfile,
    required this.workspaceId,
    required this.currentWorkspaceMemberRole,
  });

  final UserProfilePB userProfile;
  final String workspaceId;
  final AFRolePB? currentWorkspaceMemberRole;

  @override
  Widget build(BuildContext context) {
    // 检查用户权限 - 只有Owner和Member可以配置智能体
    final canConfigureAgent = currentWorkspaceMemberRole?.isOwner == true ||
        currentWorkspaceMemberRole == AFRolePB.Member;

    if (!canConfigureAgent) {
      return _buildNoPermissionView(context);
    }

    return BlocProvider(
      create: (_) => AgentSettingsBloc()..add(const AgentSettingsEvent.started()),
      child: SettingsCategory(
        title: "智能体配置",
        description: "管理工作空间级别的智能体配置和权限",
        children: [
          _WorkspaceAgentList(
            workspaceId: workspaceId,
            userRole: currentWorkspaceMemberRole!,
          ),
        ],
      ),
    );
  }

  Widget _buildNoPermissionView(BuildContext context) {
    return SettingsCategory(
      title: "智能体配置",
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: AFThemeExtension.of(context).lightGreyHover,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            children: [
              const FlowySvg(
                FlowySvgs.warning_s,
                size: Size.square(20),
              ),
              const HSpace(12),
              Expanded(
                child: FlowyText.regular(
                  "您没有权限配置智能体设置。请联系工作空间管理员。",
                  fontSize: 14,
                  color: AFThemeExtension.of(context).secondaryTextColor,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

/// 工作空间级别的智能体列表
class _WorkspaceAgentList extends StatelessWidget {
  const _WorkspaceAgentList({
    required this.workspaceId,
    required this.userRole,
  });

  final String workspaceId;
  final AFRolePB userRole;

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<AgentSettingsBloc, AgentSettingsState>(
      listener: (context, state) {
        // 显示错误信息
        if (state.error != null) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(state.error!),
              backgroundColor: Colors.red,
            ),
          );
        }
      },
      builder: (context, state) {
        // 🔍 调试：打印UI层接收到的state
        print('🎨 [UI] AgentList builder - isLoading: ${state.isLoading}, agents.length: ${state.agents.length}');
        if (state.agents.isNotEmpty) {
          for (var i = 0; i < state.agents.length; i++) {
            print('  🎨 Agent ${i + 1}: ${state.agents[i].name}');
          }
        }
        
        if (state.isLoading && state.agents.isEmpty) {
          return const Center(
            child: Padding(
              padding: EdgeInsets.all(32),
              child: CircularProgressIndicator(),
            ),
          );
        }

        if (state.agents.isEmpty) {
          print('🎨 [UI] 显示空状态');
          return _buildEmptyState(context);
        }

        print('🎨 [UI] 显示智能体列表');
        return _buildAgentList(context, state.agents);
      },
    );
  }

  Widget _buildAgentList(BuildContext context, List<AgentConfigPB> agents) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            FlowyText.medium("我的智能体", fontSize: 16),
            const Spacer(),
            if (userRole.isOwner || userRole == AFRolePB.Member) ...[
              _CreateWorkspaceAgentButton(workspaceId: workspaceId),
            ],
          ],
        ),
        const VSpace(16),
        ...agents.map((agent) => _AgentCard(
          agent: agent,
          canEdit: userRole.isOwner || userRole == AFRolePB.Member,
          onEdit: () => _showEditAgentDialog(context, agent),
          onDelete: () => _showDeleteConfirmation(context, agent),
        )),
      ],
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        border: Border.all(
          color: AFThemeExtension.of(context).lightGreyHover,
        ),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        children: [
          const FlowySvg(
            FlowySvgs.ai_summary_generate_s,
            size: Size.square(48),
          ),
          const VSpace(16),
          FlowyText.medium(
            "暂无智能体",
            fontSize: 16,
          ),
          const VSpace(8),
          FlowyText.regular(
            "创建智能体以提供专业化的 AI 助手服务",
            fontSize: 14,
            color: AFThemeExtension.of(context).secondaryTextColor,
            textAlign: TextAlign.center,
          ),
          const VSpace(16),
          if (userRole.isOwner || userRole == AFRolePB.Member) ...[
            _CreateWorkspaceAgentButton(workspaceId: workspaceId),
          ],
        ],
      ),
    );
  }

  void _showEditAgentDialog(BuildContext context, AgentConfigPB agent) {
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: context.read<AgentSettingsBloc>(),
        child: AgentDialog(existingAgent: agent),
      ),
    );
  }

  void _showDeleteConfirmation(BuildContext context, AgentConfigPB agent) {
    showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('确认删除'),
        content: Text('确定要删除智能体 "${agent.name}" 吗？此操作无法撤销。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('取消'),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.red,
              foregroundColor: Colors.white,
            ),
            onPressed: () {
              Navigator.of(dialogContext).pop();
              context.read<AgentSettingsBloc>().add(
                AgentSettingsEvent.deleteAgent(agent.id),
              );
            },
            child: const Text('删除'),
          ),
        ],
      ),
    );
  }
}

/// 创建工作空间智能体按钮
class _CreateWorkspaceAgentButton extends StatelessWidget {
  const _CreateWorkspaceAgentButton({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return ElevatedButton.icon(
      onPressed: () => _showCreateAgentDialog(context),
      icon: const Icon(Icons.add, size: 18),
      label: const Text('添加智能体'),
      style: ElevatedButton.styleFrom(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      ),
    );
  }

  void _showCreateAgentDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: context.read<AgentSettingsBloc>(),
        child: const AgentDialog(),
      ),
    );
  }
}

/// 智能体卡片
class _AgentCard extends StatelessWidget {
  const _AgentCard({
    required this.agent,
    required this.canEdit,
    required this.onEdit,
    required this.onDelete,
  });

  final AgentConfigPB agent;
  final bool canEdit;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    print('🃏 [UI] AgentCard building for: ${agent.name}');
    print('🃏 [UI] AgentCard - agent.name: ${agent.name}, agent.description: ${agent.description}');
    
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          // Avatar
          if (agent.avatar.isNotEmpty)
            Text(agent.avatar, style: const TextStyle(fontSize: 24))
          else
            const FlowySvg(
              FlowySvgs.ai_summary_generate_s,
              size: Size.square(24),
            ),
          const HSpace(12),
          // Name and description
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(agent.name, fontSize: 16),
                if (agent.description.isNotEmpty) ...[
                  const VSpace(4),
                  FlowyText.regular(
                    agent.description,
                    fontSize: 12,
                    color: Theme.of(context).colorScheme.outline,
                    maxLines: 2,
                  ),
                ],
              ],
            ),
          ),
          // Action buttons
          if (canEdit) ...[
            IconButton(
              icon: const Icon(Icons.edit, size: 20, color: Colors.blue),
              onPressed: onEdit,
              tooltip: "编辑智能体",
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
            ),
            const SizedBox(width: 4),
            IconButton(
              icon: const Icon(Icons.delete, size: 20, color: Colors.red),
              onPressed: onDelete,
              tooltip: "删除智能体",
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
            ),
          ],
        ],
      ),
    );
  }
}