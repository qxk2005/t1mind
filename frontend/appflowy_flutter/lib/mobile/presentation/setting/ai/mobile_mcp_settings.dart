import 'package:appflowy/mobile/presentation/bottom_sheet/show_mobile_bottom_sheet.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy/mobile/presentation/widgets/flowy_option_tile.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
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
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: const BoxDecoration(
            color: Colors.red, // 暂时硬编码为未连接状态
            shape: BoxShape.circle,
          ),
        ),
        const HSpace(8),
        Text(
          "未连接",
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: Colors.red,
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
    context.pop(); // 先关闭当前底部表单
    showDialog(
      context: context,
      builder: (context) => const _AddMobileMCPServerDialog(),
    );
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

/// MCP传输类型枚举
enum MCPTransportType {
  stdio,
  sse,
  http,
}

/// 移动端添加MCP服务器对话框
class _AddMobileMCPServerDialog extends StatefulWidget {
  const _AddMobileMCPServerDialog();

  @override
  State<_AddMobileMCPServerDialog> createState() => _AddMobileMCPServerDialogState();
}

class _AddMobileMCPServerDialogState extends State<_AddMobileMCPServerDialog> {
  final _nameController = TextEditingController();
  final _urlController = TextEditingController();
  
  MCPTransportType _selectedTransport = MCPTransportType.stdio;
  bool _isTestingConnection = false;
  String? _testResult;
  
  // 参数列表 (仅STDIO类型使用)
  List<Map<String, String>> _arguments = [];
  
  // 环境变量列表
  List<Map<String, String>> _environmentVariables = [];

  @override
  void dispose() {
    _nameController.dispose();
    _urlController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final screenSize = MediaQuery.of(context).size;
    final dialogWidth = screenSize.width * 0.9 > 420 ? 420.0 : screenSize.width * 0.9;
    final dialogHeight = screenSize.height * 0.85;
    
    return Dialog(
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
      ),
      child: Container(
        constraints: BoxConstraints(
          maxWidth: dialogWidth,
          maxHeight: dialogHeight,
          minHeight: 400,
        ),
        margin: const EdgeInsets.all(8),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // 标题栏
            Container(
              padding: const EdgeInsets.fromLTRB(24, 20, 16, 16),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
                borderRadius: const BorderRadius.only(
                  topLeft: Radius.circular(16),
                  topRight: Radius.circular(16),
                ),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      "添加 MCP 服务器",
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.of(context).pop(),
                    style: IconButton.styleFrom(
                      backgroundColor: Theme.of(context).colorScheme.surface,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            // 内容区域
            Flexible(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SettingsInputField(
                      label: "服务器名称",
                      placeholder: "输入服务器名称",
                      textController: _nameController,
                      hideActions: true,
                    ),
                    const VSpace(20),
                    _buildTransportTypeSelector(),
                    const VSpace(20),
                    _buildTransportSpecificFields(),
                    const VSpace(20),
                    _buildAdvancedOptions(),
                    if (_testResult != null) ...[
                      const VSpace(20),
                      _buildTestResult(),
                    ],
                    const VSpace(32),
                    _buildActionButtons(),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTransportTypeSelector() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          "传输类型",
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
            fontWeight: FontWeight.w600,
            fontSize: 15,
          ),
        ),
        const VSpace(12),
        Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
              width: 1,
            ),
            color: Theme.of(context).colorScheme.surface,
          ),
          child: Row(
            children: [
              for (final transport in MCPTransportType.values) ...[
                Expanded(
                  child: Container(
                    margin: const EdgeInsets.all(4),
                    child: Material(
                      color: Colors.transparent,
                      child: InkWell(
                        onTap: () => setState(() => _selectedTransport = transport),
                        borderRadius: BorderRadius.circular(8),
                        child: Container(
                          padding: const EdgeInsets.symmetric(vertical: 14, horizontal: 16),
                          decoration: BoxDecoration(
                            color: _selectedTransport == transport
                                ? Theme.of(context).colorScheme.primary
                                : Colors.transparent,
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Text(
                            _getTransportTypeName(transport),
                            textAlign: TextAlign.center,
                            style: TextStyle(
                              color: _selectedTransport == transport
                                  ? Colors.white
                                  : Theme.of(context).colorScheme.onSurface,
                              fontWeight: _selectedTransport == transport
                                  ? FontWeight.w600
                                  : FontWeight.w500,
                              fontSize: 14,
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildTransportSpecificFields() {
    switch (_selectedTransport) {
      case MCPTransportType.stdio:
        return Column(
          children: [
            SettingsInputField(
              label: "命令路径",
              placeholder: "例如: /usr/local/bin/mcp-server",
              textController: _urlController,
              hideActions: true,
            ),
            const VSpace(12),
            _buildArgumentsSection(),
          ],
        );
      case MCPTransportType.sse:
      case MCPTransportType.http:
        return SettingsInputField(
          label: "服务器URL",
          placeholder: _selectedTransport == MCPTransportType.sse
              ? "例如: http://localhost:3000/sse"
              : "例如: http://localhost:3000/mcp",
          textController: _urlController,
          hideActions: true,
        );
    }
  }

  Widget _buildAdvancedOptions() {
    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
          width: 1,
        ),
        color: Theme.of(context).colorScheme.surface,
      ),
      child: ExpansionTile(
        title: Text(
          "高级选项",
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
            fontWeight: FontWeight.w600,
            fontSize: 15,
          ),
        ),
        tilePadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
        ),
        children: [
          _buildEnvironmentVariablesSection(),
        ],
      ),
    );
  }

  Widget _buildTestResult() {
    final isSuccess = _testResult!.contains('成功') || _testResult!.contains('successful');
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: isSuccess ? Colors.green.withOpacity(0.1) : Colors.red.withOpacity(0.1),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: isSuccess ? Colors.green : Colors.red,
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(
            isSuccess ? Icons.check_circle : Icons.error,
            color: isSuccess ? Colors.green : Colors.red,
            size: 16,
          ),
          const HSpace(8),
          Expanded(
            child: Text(
              _testResult!,
              style: TextStyle(
                color: isSuccess ? Colors.green : Colors.red,
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons() {
    return Column(
      children: [
        // 测试连接按钮
        SizedBox(
          width: double.infinity,
          child: ElevatedButton.icon(
            onPressed: _isTestingConnection ? null : _testConnection,
            icon: _isTestingConnection
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      color: Colors.white,
                    ),
                  )
                : const Icon(Icons.wifi_tethering, size: 18),
            label: Text(
              _isTestingConnection ? "测试中..." : "测试连接",
              style: const TextStyle(
                fontSize: 15,
                fontWeight: FontWeight.w600,
              ),
            ),
            style: ElevatedButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.surfaceVariant,
              foregroundColor: Theme.of(context).colorScheme.onSurfaceVariant,
              padding: const EdgeInsets.symmetric(vertical: 16),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12),
              ),
              elevation: 0,
            ),
          ),
        ),
        const VSpace(16),
        // 取消和保存按钮
        Row(
          children: [
            Expanded(
              child: OutlinedButton(
                onPressed: () => Navigator.of(context).pop(),
                style: OutlinedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(vertical: 16),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
                  side: BorderSide(
                    color: Theme.of(context).colorScheme.outline.withOpacity(0.5),
                    width: 1.5,
                  ),
                ),
                child: Text(
                  "取消",
                  style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: Theme.of(context).colorScheme.onSurface,
                  ),
                ),
              ),
            ),
            const HSpace(16),
            Expanded(
              child: ElevatedButton(
                onPressed: _canSave() ? _saveServer : null,
                style: ElevatedButton.styleFrom(
                  backgroundColor: _canSave()
                      ? Theme.of(context).colorScheme.primary
                      : Theme.of(context).colorScheme.surfaceVariant,
                  foregroundColor: _canSave()
                      ? Colors.white
                      : Theme.of(context).colorScheme.onSurfaceVariant,
                  padding: const EdgeInsets.symmetric(vertical: 16),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
                  elevation: _canSave() ? 2 : 0,
                ),
                child: Text(
                  "保存",
                  style: TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: _canSave() ? Colors.white : Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }

  String _getTransportTypeName(MCPTransportType transport) {
    switch (transport) {
      case MCPTransportType.stdio:
        return "STDIO";
      case MCPTransportType.sse:
        return "SSE";
      case MCPTransportType.http:
        return "HTTP";
    }
  }

  bool _canSave() {
    return _nameController.text.trim().isNotEmpty &&
           _urlController.text.trim().isNotEmpty;
  }

  void _testConnection() async {
    if (!_canSave()) return;

    setState(() {
      _isTestingConnection = true;
      _testResult = null;
    });

    try {
      // 模拟连接测试过程
      await Future.delayed(const Duration(seconds: 2));
      
      // 这里可以添加实际的连接测试逻辑
      final serverConfig = _buildServerConfig();
      print('测试连接配置: $serverConfig');
      
      // 模拟成功结果
      if (mounted) {
        setState(() {
          _isTestingConnection = false;
          _testResult = "✅ 连接测试成功！服务器响应正常。";
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isTestingConnection = false;
          _testResult = "❌ 连接测试失败：${e.toString()}";
        });
      }
    }
  }

  void _saveServer() {
    if (!_canSave()) return;

    try {
      final serverConfig = _buildServerConfig();
      
      // 这里可以添加实际的保存逻辑
      print('保存服务器配置: $serverConfig');
      
      Navigator.of(context).pop();
      
      // 显示成功消息
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Row(
            children: [
              const Icon(Icons.check_circle, color: Colors.white, size: 20),
              const SizedBox(width: 8),
              Expanded(
                child: Text("MCP 服务器 '${_nameController.text}' 已成功保存"),
              ),
            ],
          ),
          backgroundColor: Colors.green,
          duration: const Duration(seconds: 3),
        ),
      );
    } catch (e) {
      // 显示错误消息
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Row(
            children: [
              const Icon(Icons.error, color: Colors.white, size: 20),
              const SizedBox(width: 8),
              Expanded(
                child: Text("保存失败：${e.toString()}"),
              ),
            ],
          ),
          backgroundColor: Colors.red,
          duration: const Duration(seconds: 3),
        ),
      );
    }
  }

  Widget _buildArgumentsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text(
              "参数",
              style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
                fontSize: 15,
              ),
            ),
            const Spacer(),
            TextButton.icon(
              onPressed: () => _addArgument(),
              icon: const Icon(Icons.add, size: 16),
              label: const Text("添加参数", style: TextStyle(fontSize: 12)),
              style: TextButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              ),
            ),
          ],
        ),
        const VSpace(8),
        ..._arguments.asMap().entries.map((entry) {
          final index = entry.key;
          final arg = entry.value;
          return _buildKeyValueRow(
            key: arg['key'] ?? '',
            value: arg['value'] ?? '',
            keyHint: "参数名",
            valueHint: "参数值",
            onKeyChanged: (value) => _updateArgument(index, 'key', value),
            onValueChanged: (value) => _updateArgument(index, 'value', value),
            onDelete: () => _removeArgument(index),
          );
        }),
        if (_arguments.isEmpty)
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
                width: 1,
              ),
            ),
            child: Center(
              child: Text(
                "暂无参数，点击上方按钮添加",
                style: TextStyle(
                  fontSize: 12,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildEnvironmentVariablesSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text(
              "环境变量",
              style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
                fontSize: 15,
              ),
            ),
            const Spacer(),
            TextButton.icon(
              onPressed: () => _addEnvironmentVariable(),
              icon: const Icon(Icons.add, size: 16),
              label: const Text("添加变量", style: TextStyle(fontSize: 12)),
              style: TextButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              ),
            ),
          ],
        ),
        const VSpace(8),
        ..._environmentVariables.asMap().entries.map((entry) {
          final index = entry.key;
          final env = entry.value;
          return _buildKeyValueRow(
            key: env['key'] ?? '',
            value: env['value'] ?? '',
            keyHint: "变量名",
            valueHint: "变量值",
            onKeyChanged: (value) => _updateEnvironmentVariable(index, 'key', value),
            onValueChanged: (value) => _updateEnvironmentVariable(index, 'value', value),
            onDelete: () => _removeEnvironmentVariable(index),
          );
        }),
        if (_environmentVariables.isEmpty)
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
                width: 1,
              ),
            ),
            child: Center(
              child: Text(
                "暂无环境变量，点击上方按钮添加",
                style: TextStyle(
                  fontSize: 12,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildKeyValueRow({
    required String key,
    required String value,
    required String keyHint,
    required String valueHint,
    required Function(String) onKeyChanged,
    required Function(String) onValueChanged,
    required VoidCallback onDelete,
  }) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
          width: 1,
        ),
      ),
      child: Column(
        children: [
          Row(
            children: [
              Expanded(
                flex: 2,
                child: TextField(
                  controller: TextEditingController(text: key),
                  decoration: InputDecoration(
                    hintText: keyHint,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(6),
                      borderSide: BorderSide(
                        color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
                      ),
                    ),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    isDense: true,
                  ),
                  onChanged: onKeyChanged,
                ),
              ),
              const HSpace(8),
              IconButton(
                icon: Icon(
                  Icons.delete_outline,
                  size: 18,
                  color: Colors.red.shade400,
                ),
                onPressed: onDelete,
                style: IconButton.styleFrom(
                  backgroundColor: Colors.red.shade50,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6),
                  ),
                ),
              ),
            ],
          ),
          const VSpace(8),
          TextField(
            controller: TextEditingController(text: value),
            decoration: InputDecoration(
              hintText: valueHint,
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(6),
                borderSide: BorderSide(
                  color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
                ),
              ),
              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              isDense: true,
            ),
            onChanged: onValueChanged,
          ),
        ],
      ),
    );
  }

  void _addArgument() {
    setState(() {
      _arguments.add({'key': '', 'value': ''});
    });
  }

  void _updateArgument(int index, String field, String value) {
    setState(() {
      _arguments[index][field] = value;
    });
  }

  void _removeArgument(int index) {
    setState(() {
      _arguments.removeAt(index);
    });
  }

  void _addEnvironmentVariable() {
    setState(() {
      _environmentVariables.add({'key': '', 'value': ''});
    });
  }

  void _updateEnvironmentVariable(int index, String field, String value) {
    setState(() {
      _environmentVariables[index][field] = value;
    });
  }

  void _removeEnvironmentVariable(int index) {
    setState(() {
      _environmentVariables.removeAt(index);
    });
  }

  Map<String, dynamic> _buildServerConfig() {
    final config = <String, dynamic>{
      'name': _nameController.text.trim(),
      'transport': _selectedTransport.name,
    };

    switch (_selectedTransport) {
      case MCPTransportType.stdio:
        config['command'] = _urlController.text.trim();
        if (_arguments.isNotEmpty) {
          config['args'] = _arguments
              .where((arg) => arg['key']?.isNotEmpty == true)
              .map((arg) => '${arg['key']}=${arg['value'] ?? ''}')
              .toList();
        }
        break;
      case MCPTransportType.sse:
      case MCPTransportType.http:
        config['url'] = _urlController.text.trim();
        break;
    }

    if (_environmentVariables.isNotEmpty) {
      config['env'] = Map.fromEntries(
        _environmentVariables
            .where((env) => env['key']?.isNotEmpty == true)
            .map((env) => MapEntry(env['key']!, env['value'] ?? '')),
      );
    }

    return config;
  }
}
