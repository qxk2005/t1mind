import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';

/// 移动端智能体设置页面
class MobileAgentSettingsPage extends StatelessWidget {
  const MobileAgentSettingsPage({
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
    return Scaffold(
      appBar: AppBar(
        title: const Text("智能体配置"), // TODO: 使用翻译
        backgroundColor: Theme.of(context).colorScheme.surface,
        elevation: 0,
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    // 检查用户权限
    final canConfigureAgent = currentWorkspaceMemberRole?.isOwner == true ||
        currentWorkspaceMemberRole == AFRolePB.Member;

    if (!canConfigureAgent) {
      return _buildNoPermissionView(context);
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildDescription(context),
          const VSpace(24),
          _buildAgentList(context),
          const VSpace(24),
          if (currentWorkspaceMemberRole?.isOwner == true) ...[
            _buildPermissionSettings(context),
            const VSpace(24),
          ],
          _buildActions(context),
        ],
      ),
    );
  }

  Widget _buildDescription(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AFThemeExtension.of(context).lightGreyHover,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                Icons.smart_toy_outlined,
                size: 24,
              ),
              const HSpace(12),
              Expanded(
                child: FlowyText.medium(
                  "AI 智能体助手",
                  fontSize: 16,
                ),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            "智能体是专业化的 AI 助手，可以根据特定领域和任务进行定制。您可以创建不同的智能体来处理各种专业任务。",
            fontSize: 14,
            color: AFThemeExtension.of(context).secondaryTextColor,
          ),
        ],
      ),
    );
  }

  Widget _buildAgentList(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "智能体列表",
          fontSize: 18,
        ),
        const VSpace(16),
        _buildEmptyAgentList(context),
      ],
    );
  }

  Widget _buildEmptyAgentList(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        border: Border.all(
          color: AFThemeExtension.of(context).lightGreyHover,
        ),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        children: [
          const Icon(
            Icons.smart_toy_outlined,
            size: 48,
            color: Colors.grey,
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
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: () => _showCreateAgentDialog(context),
              icon: const FlowySvg(
                FlowySvgs.add_s,
                color: Colors.white,
              ),
              label: FlowyText.regular(
                "创建智能体",
                fontSize: 14,
                color: Colors.white,
              ),
              style: ElevatedButton.styleFrom(
                backgroundColor: Theme.of(context).colorScheme.primary,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(8),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPermissionSettings(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "权限设置",
          fontSize: 18,
        ),
        const VSpace(16),
        _buildPermissionCard(
          context,
          "允许成员创建智能体",
          "工作空间成员可以创建和配置智能体",
          true,
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const VSpace(12),
        _buildPermissionCard(
          context,
          "允许成员修改智能体",
          "工作空间成员可以修改其他人创建的智能体",
          false,
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const VSpace(12),
        _buildPermissionCard(
          context,
          "允许访客使用",
          "访客可以使用已配置的智能体",
          false,
          (value) {
            // TODO: 更新权限配置
          },
        ),
      ],
    );
  }

  Widget _buildPermissionCard(
    BuildContext context,
    String title,
    String description,
    bool value,
    ValueChanged<bool> onChanged,
  ) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(
          color: AFThemeExtension.of(context).lightGreyHover,
        ),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  title,
                  fontSize: 16,
                ),
                const VSpace(4),
                FlowyText.regular(
                  description,
                  fontSize: 14,
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
      ),
    );
  }

  Widget _buildActions(BuildContext context) {
    if (currentWorkspaceMemberRole?.isOwner != true) {
      return const SizedBox.shrink();
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "操作",
          fontSize: 18,
        ),
        const VSpace(16),
        _buildActionButton(
          context,
          "导出智能体",
          "将当前智能体配置导出为文件",
          FlowySvgs.share_s,
          () => _exportAgents(context),
        ),
        const VSpace(12),
        _buildActionButton(
          context,
          "导入智能体",
          "从文件导入智能体配置",
          FlowySvgs.import_s,
          () => _importAgents(context),
        ),
        const VSpace(12),
        _buildActionButton(
          context,
          "重置智能体",
          "清除所有智能体配置并恢复默认设置",
          FlowySvgs.restore_s,
          () => _resetAgents(context),
          isDestructive: true,
        ),
      ],
    );
  }

  Widget _buildActionButton(
    BuildContext context,
    String title,
    String description,
    FlowySvgData icon,
    VoidCallback onTap, {
    bool isDestructive = false,
  }) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(
          color: AFThemeExtension.of(context).lightGreyHover,
        ),
        borderRadius: BorderRadius.circular(12),
      ),
      child: FlowyButton(
        margin: const EdgeInsets.all(16),
        text: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            FlowyText.medium(
              title,
              fontSize: 16,
              color: isDestructive ? Colors.red : null,
            ),
            const VSpace(4),
            FlowyText.regular(
              description,
              fontSize: 14,
              color: AFThemeExtension.of(context).secondaryTextColor,
            ),
          ],
        ),
        leftIcon: FlowySvg(
          icon,
          color: isDestructive ? Colors.red : null,
        ),
        onTap: onTap,
      ),
    );
  }

  Widget _buildNoPermissionView(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const FlowySvg(
              FlowySvgs.warning_s,
              size: Size.square(48),
            ),
            const VSpace(16),
            FlowyText.medium(
              "无权限访问",
              fontSize: 18,
            ),
            const VSpace(8),
            FlowyText.regular(
              "您没有权限配置智能体设置。请联系工作空间管理员。",
              fontSize: 14,
              color: AFThemeExtension.of(context).secondaryTextColor,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  void _showCreateAgentDialog(BuildContext context) {
    // TODO: 实现创建智能体对话框
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
