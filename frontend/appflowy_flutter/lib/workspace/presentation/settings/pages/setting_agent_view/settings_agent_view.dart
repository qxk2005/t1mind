import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/workspace/workspace_agent_settings.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flutter/material.dart';

/// 服务器端智能体设置页面
class SettingsAgentView extends StatelessWidget {
  const SettingsAgentView({
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
      title: "智能体配置", // TODO: 使用 LocaleKeys.settings_agentPage_title.tr()
      description: "创建和管理 AI 智能体助手", // TODO: 使用 LocaleKeys.settings_agentPage_description.tr()
      children: [
        WorkspaceAgentSettings(
          userProfile: userProfile,
          workspaceId: workspaceId,
          currentWorkspaceMemberRole: currentWorkspaceMemberRole,
        ),
      ],
    );
  }
}

/// 本地工作空间智能体设置页面
class LocalSettingsAgentView extends StatelessWidget {
  const LocalSettingsAgentView({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: "智能体配置", // TODO: 使用 LocaleKeys.settings_agentPage_title.tr()
      description: "创建和管理 AI 智能体助手", // TODO: 使用 LocaleKeys.settings_agentPage_description.tr()
      children: [
        WorkspaceAgentSettings(
          userProfile: userProfile,
          workspaceId: workspaceId,
          currentWorkspaceMemberRole: AFRolePB.Owner, // 本地工作空间默认Owner权限
        ),
      ],
    );
  }
}
