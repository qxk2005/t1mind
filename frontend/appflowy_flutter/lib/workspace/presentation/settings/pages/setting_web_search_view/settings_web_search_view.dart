import 'package:appflowy/plugins/ai_chat/widgets/web_search_settings.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flutter/material.dart';

/// 全局设置中的网络搜索配置视图
class SettingsWebSearchView extends StatelessWidget {
  const SettingsWebSearchView({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return WebSearchSettingsPage(
      userProfile: userProfile,
      workspaceId: workspaceId,
    );
  }
}

