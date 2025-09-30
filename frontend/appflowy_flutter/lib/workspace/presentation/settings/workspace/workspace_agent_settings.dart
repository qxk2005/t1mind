import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category_spacer.dart';
import 'package:appflowy/workspace/presentation/settings/shared/single_setting_action.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';

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

    return SettingsCategory(
      title: "智能体配置",
      description: "管理工作空间级别的智能体配置和权限",
      children: [
        _WorkspaceAgentList(
          workspaceId: workspaceId,
          userRole: currentWorkspaceMemberRole!,
        ),
        const SettingsCategorySpacer(),
        if (currentWorkspaceMemberRole?.isOwner == true) ...[
          _WorkspaceAgentPermissionSettings(
            workspaceId: workspaceId,
          ),
          const SettingsCategorySpacer(),
        ],
        _WorkspaceAgentActions(
          workspaceId: workspaceId,
          userRole: currentWorkspaceMemberRole!,
        ),
      ],
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
    // 模拟空状态，因为我们还没有实际的数据
    return _buildEmptyState(context);
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
}

/// 创建工作空间智能体按钮
class _CreateWorkspaceAgentButton extends StatelessWidget {
  const _CreateWorkspaceAgentButton({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return FlowyButton(
      text: FlowyText.regular(
        "创建智能体",
        fontSize: 14,
      ),
      leftIcon: const FlowySvg(FlowySvgs.add_s),
      onTap: () => _showCreateAgentDialog(context),
    );
  }

  void _showCreateAgentDialog(BuildContext context) {
    // TODO: 实现创建智能体对话框
  }
}

/// 工作空间智能体权限设置
class _WorkspaceAgentPermissionSettings extends StatelessWidget {
  const _WorkspaceAgentPermissionSettings({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "权限设置",
          fontSize: 16,
        ),
        const VSpace(8),
        FlowyText.regular(
          "配置工作空间成员对智能体的访问权限",
          fontSize: 14,
          color: AFThemeExtension.of(context).secondaryTextColor,
        ),
        const VSpace(16),
        _buildPermissionItem(
          context,
          "允许成员创建智能体",
          "工作空间成员可以创建和配置智能体",
          true, // TODO: 从配置中读取
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const VSpace(12),
        _buildPermissionItem(
          context,
          "允许成员修改智能体",
          "工作空间成员可以修改其他人创建的智能体",
          false, // TODO: 从配置中读取
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const VSpace(12),
        _buildPermissionItem(
          context,
          "允许访客使用",
          "访客可以使用已配置的智能体",
          false, // TODO: 从配置中读取
          (value) {
            // TODO: 更新权限配置
          },
        ),
      ],
    );
  }

  Widget _buildPermissionItem(
    BuildContext context,
    String title,
    String description,
    bool value,
    ValueChanged<bool> onChanged,
  ) {
    return Row(
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.regular(
                title,
                fontSize: 14,
              ),
              const VSpace(4),
              FlowyText.regular(
                description,
                fontSize: 12,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
            ],
          ),
        ),
        const HSpace(16),
        Switch(
          value: value,
          onChanged: onChanged,
        ),
      ],
    );
  }
}

/// 工作空间智能体操作
class _WorkspaceAgentActions extends StatelessWidget {
  const _WorkspaceAgentActions({
    required this.workspaceId,
    required this.userRole,
  });

  final String workspaceId;
  final AFRolePB userRole;

  @override
  Widget build(BuildContext context) {
    if (!userRole.isOwner) {
      return const SizedBox.shrink();
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "操作",
          fontSize: 16,
        ),
        const VSpace(16),
        SingleSettingAction(
          label: "导出智能体",
          fontSize: 14,
          onPressed: () => _exportAgents(context),
          buttonType: SingleSettingsButtonType.primary,
          buttonLabel: "导出",
        ),
        const VSpace(8),
        SingleSettingAction(
          label: "导入智能体",
          fontSize: 14,
          onPressed: () => _importAgents(context),
          buttonType: SingleSettingsButtonType.primary,
          buttonLabel: "导入",
        ),
        const VSpace(8),
        SingleSettingAction(
          label: "重置智能体",
          fontSize: 14,
          onPressed: () => _resetAgents(context),
          buttonType: SingleSettingsButtonType.danger,
          buttonLabel: "重置",
        ),
      ],
    );
  }

  void _exportAgents(BuildContext context) {
    // TODO: 实现智能体导出
  }

  void _importAgents(BuildContext context) {
    // TODO: 实现智能体导入
  }

  void _resetAgents(BuildContext context) {
    // TODO: 实现智能体重置
  }
}