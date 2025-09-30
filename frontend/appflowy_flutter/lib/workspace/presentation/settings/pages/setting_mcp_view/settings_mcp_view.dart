import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/workspace/workspace_mcp_settings.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flutter/material.dart';

/// 服务器端MCP设置页面
class SettingsMCPView extends StatelessWidget {
  const SettingsMCPView({
    super.key,
    required this.userProfile,
    required this.currentWorkspaceMemberRole,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final AFRolePB? currentWorkspaceMemberRole;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: "MCP 配置", // TODO: 使用 LocaleKeys.settings_mcpPage_title.tr()
      description: "配置和管理 Model Context Protocol 服务器", // TODO: 使用 LocaleKeys.settings_mcpPage_description.tr()
      children: [
        WorkspaceMCPSettings(
          userProfile: userProfile,
          workspaceId: workspaceId,
          currentWorkspaceMemberRole: currentWorkspaceMemberRole,
        ),
      ],
    );
  }
}

/// 本地工作空间MCP设置页面
class LocalSettingsMCPView extends StatelessWidget {
  const LocalSettingsMCPView({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: "MCP 配置", // TODO: 使用 LocaleKeys.settings_mcpPage_title.tr()
      description: "配置和管理 Model Context Protocol 服务器", // TODO: 使用 LocaleKeys.settings_mcpPage_description.tr()
      children: [
        WorkspaceMCPSettings(
          userProfile: userProfile,
          workspaceId: workspaceId,
          currentWorkspaceMemberRole: AFRolePB.Owner, // 本地工作空间默认Owner权限
        ),
      ],
    );
  }
}
