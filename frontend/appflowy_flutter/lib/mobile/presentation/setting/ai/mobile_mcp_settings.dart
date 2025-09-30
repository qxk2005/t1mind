import 'package:appflowy/mobile/presentation/bottom_sheet/show_mobile_bottom_sheet.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy/mobile/presentation/widgets/flowy_option_tile.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_profile.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

/// 移动端MCP配置组件
/// 适配小屏幕，优化触摸操作
class MobileMCPSettings extends StatelessWidget {
  const MobileMCPSettings({
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
    // 检查用户权限
    final canConfigureMCP = currentWorkspaceMemberRole?.isOwner == true ||
        currentWorkspaceMemberRole == AFRolePB.Member;

    if (!canConfigureMCP) {
      return _buildNoPermissionView(context);
    }

    return MobileSettingGroup(
      groupTitle: "MCP 配置",
      settingItemList: [
        // MCP服务器管理
        MobileSettingItem(
          name: "MCP 服务器",
          subtitle: _buildSubtitle(context, "管理 MCP 服务器连接"),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _showMCPServerList(context),
        ),
        // 连接状态
        MobileSettingItem(
          name: "连接状态",
          subtitle: _buildSubtitle(context, "查看服务器连接状态"),
          trailing: _buildConnectionStatus(context),
          onTap: () => _showConnectionStatus(context),
        ),
        // 权限设置（仅Owner可见）
        if (currentWorkspaceMemberRole?.isOwner == true)
          MobileSettingItem(
            name: "权限设置",
            subtitle: _buildSubtitle(context, "配置成员访问权限"),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => _showPermissionSettings(context),
          ),
        // 配置管理
        MobileSettingItem(
          name: "配置管理",
          subtitle: _buildSubtitle(context, "导入/导出配置"),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _showConfigurationManagement(context),
        ),
      ],
    );
  }

  Widget _buildNoPermissionView(BuildContext context) {
    return MobileSettingGroup(
      groupTitle: "MCP 配置",
      settingItemList: [
        Container(
          padding: const EdgeInsets.all(16),
          margin: const EdgeInsets.symmetric(vertical: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
            ),
          ),
          child: Row(
            children: [
              Icon(
                Icons.warning_rounded,
                color: Theme.of(context).colorScheme.primary,
                size: 20,
              ),
              const HSpace(12),
              Expanded(
                child: Text(
                  "您没有权限配置 MCP 设置。请联系工作空间管理员。",
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildSubtitle(BuildContext context, String text) {
    return Text(
      text,
      style: Theme.of(context).textTheme.bodySmall?.copyWith(
        color: Theme.of(context).colorScheme.onSurfaceVariant,
      ),
    );
  }

  Widget _buildConnectionStatus(BuildContext context) {
    // TODO: 从实际状态获取连接信息
    final isConnected = false; // 模拟状态
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: isConnected ? Colors.green : Colors.red,
            shape: BoxShape.circle,
          ),
        ),
        const HSpace(8),
        Text(
          isConnected ? "已连接" : "未连接",
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: isConnected ? Colors.green : Colors.red,
          ),
        ),
        const HSpace(4),
        const Icon(Icons.chevron_right),
      ],
    );
  }

  void _showMCPServerList(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "MCP 服务器",
      builder: (context) => _MCPServerListView(
        workspaceId: workspaceId,
        userRole: currentWorkspaceMemberRole!,
      ),
    );
  }

  void _showConnectionStatus(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "连接状态",
      builder: (context) => _MCPConnectionStatusView(
        workspaceId: workspaceId,
      ),
    );
  }

  void _showPermissionSettings(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "权限设置",
      builder: (context) => _MCPPermissionSettingsView(
        workspaceId: workspaceId,
      ),
    );
  }

  void _showConfigurationManagement(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "配置管理",
      builder: (context) => _MCPConfigurationManagementView(
        workspaceId: workspaceId,
        userRole: currentWorkspaceMemberRole!,
      ),
    );
  }
}

/// MCP服务器列表视图
class _MCPServerListView extends StatelessWidget {
  const _MCPServerListView({
    required this.workspaceId,
    required this.userRole,
  });

  final String workspaceId;
  final AFRolePB userRole;

  @override
  Widget build(BuildContext context) {
    // TODO: 从实际数据源获取服务器列表
    final servers = <String>[]; // 模拟空列表

    if (servers.isEmpty) {
      return _buildEmptyState(context);
    }

    return Column(
      children: [
        ...servers.map((server) => _buildServerItem(context, server)),
        if (userRole.isOwner || userRole == AFRolePB.Member) ...[
          const VSpace(16),
          _buildAddServerButton(context),
        ],
      ],
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          Icon(
            Icons.dns_outlined,
            size: 48,
            color: Theme.of(context).colorScheme.primary.withOpacity(0.6),
          ),
          const VSpace(16),
          Text(
            "暂无 MCP 服务器",
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const VSpace(8),
          Text(
            "添加 MCP 服务器以扩展 AI 助手的功能",
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          const VSpace(24),
          if (userRole.isOwner || userRole == AFRolePB.Member) ...[
            _buildAddServerButton(context),
          ],
        ],
      ),
    );
  }

  Widget _buildServerItem(BuildContext context, String server) {
    return FlowyOptionTile.text(
      text: server,
      showTopBorder: false,
      onTap: () => _editServer(context, server),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          IconButton(
            icon: const Icon(Icons.edit_outlined, size: 20),
            onPressed: () => _editServer(context, server),
          ),
          IconButton(
            icon: const Icon(Icons.delete_outline, size: 20),
            onPressed: () => _deleteServer(context, server),
          ),
        ],
      ),
    );
  }

  Widget _buildAddServerButton(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: ElevatedButton.icon(
        onPressed: () => _addServer(context),
        icon: const Icon(Icons.add),
        label: const Text("添加服务器"),
        style: ElevatedButton.styleFrom(
          padding: const EdgeInsets.symmetric(vertical: 12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
        ),
      ),
    );
  }

  void _addServer(BuildContext context) {
    // TODO: 实现添加服务器对话框
    context.pop();
  }

  void _editServer(BuildContext context, String server) {
    // TODO: 实现编辑服务器对话框
  }

  void _deleteServer(BuildContext context, String server) {
    // TODO: 实现删除服务器确认对话框
  }
}

/// MCP连接状态视图
class _MCPConnectionStatusView extends StatelessWidget {
  const _MCPConnectionStatusView({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    // TODO: 从实际数据源获取连接状态
    final connections = <Map<String, dynamic>>[]; // 模拟空列表

    if (connections.isEmpty) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            Icon(
              Icons.cloud_off_outlined,
              size: 48,
              color: Theme.of(context).colorScheme.primary.withOpacity(0.6),
            ),
            const VSpace(16),
            Text(
              "暂无连接",
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const VSpace(8),
            Text(
              "配置 MCP 服务器后将显示连接状态",
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      );
    }

    return Column(
      children: connections.map((connection) => _buildConnectionItem(context, connection)).toList(),
    );
  }

  Widget _buildConnectionItem(BuildContext context, Map<String, dynamic> connection) {
    final isConnected = connection['connected'] as bool? ?? false;
    return ListTile(
      leading: Container(
        width: 12,
        height: 12,
        decoration: BoxDecoration(
          color: isConnected ? Colors.green : Colors.red,
          shape: BoxShape.circle,
        ),
      ),
      title: Text(connection['name'] as String? ?? ''),
      subtitle: Text(
        isConnected ? "已连接" : "连接失败",
        style: TextStyle(
          color: isConnected ? Colors.green : Colors.red,
        ),
      ),
      trailing: IconButton(
        icon: const Icon(Icons.refresh),
        onPressed: () => _reconnect(context, connection),
      ),
    );
  }

  void _reconnect(BuildContext context, Map<String, dynamic> connection) {
    // TODO: 实现重新连接功能
  }
}

/// MCP权限设置视图
class _MCPPermissionSettingsView extends StatelessWidget {
  const _MCPPermissionSettingsView({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _buildPermissionItem(
          context,
          "允许成员配置 MCP",
          "工作空间成员可以添加和配置 MCP 服务器",
          true, // TODO: 从配置中读取
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const Divider(),
        _buildPermissionItem(
          context,
          "允许访客使用",
          "访客可以使用已配置的 MCP 服务器",
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
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const VSpace(4),
                Text(
                  description,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
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
}

/// MCP配置管理视图
class _MCPConfigurationManagementView extends StatelessWidget {
  const _MCPConfigurationManagementView({
    required this.workspaceId,
    required this.userRole,
  });

  final String workspaceId;
  final AFRolePB userRole;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        FlowyOptionTile.text(
          text: "导出配置",
          showTopBorder: false,
          onTap: () => _exportConfiguration(context),
          trailing: const Icon(Icons.download_outlined),
        ),
        FlowyOptionTile.text(
          text: "导入配置",
          showTopBorder: false,
          onTap: () => _importConfiguration(context),
          trailing: const Icon(Icons.upload_outlined),
        ),
        if (userRole.isOwner) ...[
          FlowyOptionTile.text(
            text: "重置配置",
            showTopBorder: false,
            onTap: () => _resetConfiguration(context),
            trailing: Icon(
              Icons.restore_outlined,
              color: Theme.of(context).colorScheme.error,
            ),
          ),
        ],
      ],
    );
  }

  void _exportConfiguration(BuildContext context) {
    // TODO: 实现配置导出
    context.pop();
  }

  void _importConfiguration(BuildContext context) {
    // TODO: 实现配置导入
    context.pop();
  }

  void _resetConfiguration(BuildContext context) {
    // TODO: 实现配置重置确认对话框
    context.pop();
  }
}
