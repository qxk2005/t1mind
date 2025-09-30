import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';

/// 移动端MCP设置页面
class MobileMCPSettingsPage extends StatelessWidget {
  const MobileMCPSettingsPage({
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
        title: const Text("MCP 配置"), // TODO: 使用翻译
        backgroundColor: Theme.of(context).colorScheme.surface,
        elevation: 0,
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    // 检查用户权限
    final canConfigureMCP = currentWorkspaceMemberRole?.isOwner == true ||
        currentWorkspaceMemberRole == AFRolePB.Member;

    if (!canConfigureMCP) {
      return _buildNoPermissionView(context);
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildDescription(context),
          const VSpace(24),
          _buildMCPServerList(context),
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
              const FlowySvg(
                FlowySvgs.ai_summary_generate_s,
                size: Size.square(24),
              ),
              const HSpace(12),
              Expanded(
                child: FlowyText.medium(
                  "Model Context Protocol",
                  fontSize: 16,
                ),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            "MCP 允许 AI 助手连接外部工具和服务，扩展其能力边界。您可以配置各种 MCP 服务器来增强 AI 的功能。",
            fontSize: 14,
            color: AFThemeExtension.of(context).secondaryTextColor,
          ),
        ],
      ),
    );
  }

  Widget _buildMCPServerList(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "MCP 服务器",
          fontSize: 18,
        ),
        const VSpace(16),
        _buildEmptyServerList(context),
      ],
    );
  }

  Widget _buildEmptyServerList(BuildContext context) {
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
          const FlowySvg(
            FlowySvgs.ai_summary_generate_s,
            size: Size.square(48),
          ),
          const VSpace(16),
          FlowyText.medium(
            "暂无 MCP 服务器",
            fontSize: 16,
          ),
          const VSpace(8),
          FlowyText.regular(
            "添加 MCP 服务器以扩展 AI 助手的功能",
            fontSize: 14,
            color: AFThemeExtension.of(context).secondaryTextColor,
            textAlign: TextAlign.center,
          ),
          const VSpace(16),
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: () => _showAddServerDialog(context),
              icon: const FlowySvg(
                FlowySvgs.add_s,
                color: Colors.white,
              ),
              label: FlowyText.regular(
                "添加服务器",
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
          "允许成员配置 MCP",
          "工作空间成员可以添加和配置 MCP 服务器",
          true,
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const VSpace(12),
        _buildPermissionCard(
          context,
          "允许访客使用",
          "访客可以使用已配置的 MCP 服务器",
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
          "导出配置",
          "将当前 MCP 配置导出为文件",
          FlowySvgs.share_s,
          () => _exportConfiguration(context),
        ),
        const VSpace(12),
        _buildActionButton(
          context,
          "导入配置",
          "从文件导入 MCP 配置",
          FlowySvgs.import_s,
          () => _importConfiguration(context),
        ),
        const VSpace(12),
        _buildActionButton(
          context,
          "重置配置",
          "清除所有 MCP 配置并恢复默认设置",
          FlowySvgs.restore_s,
          () => _resetConfiguration(context),
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
              "您没有权限配置 MCP 设置。请联系工作空间管理员。",
              fontSize: 14,
              color: AFThemeExtension.of(context).secondaryTextColor,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  void _showAddServerDialog(BuildContext context) {
    // TODO: 实现添加服务器对话框
  }

  void _exportConfiguration(BuildContext context) {
    // TODO: 实现配置导出
  }

  void _importConfiguration(BuildContext context) {
    // TODO: 实现配置导入
  }

  void _resetConfiguration(BuildContext context) {
    // TODO: 实现配置重置
  }
}
