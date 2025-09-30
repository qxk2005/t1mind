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

/// 移动端智能体配置组件
/// 适配小屏幕，优化触摸操作和界面布局
class MobileAgentSettings extends StatelessWidget {
  const MobileAgentSettings({
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
    final canConfigureAgent = currentWorkspaceMemberRole?.isOwner == true ||
        currentWorkspaceMemberRole == AFRolePB.Member;

    if (!canConfigureAgent) {
      return _buildNoPermissionView(context);
    }

    return MobileSettingGroup(
      groupTitle: "智能体配置",
      settingItemList: [
        // 智能体管理
        MobileSettingItem(
          name: "我的智能体",
          subtitle: _buildSubtitle(context, "管理个人智能体"),
          trailing: _buildAgentCount(context, 0), // TODO: 从实际数据获取
          onTap: () => _showAgentList(context),
        ),
        // 智能体模板
        MobileSettingItem(
          name: "智能体模板",
          subtitle: _buildSubtitle(context, "浏览预设智能体模板"),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _showAgentTemplates(context),
        ),
        // 工具配置
        MobileSettingItem(
          name: "工具配置",
          subtitle: _buildSubtitle(context, "管理智能体可用工具"),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _showToolConfiguration(context),
        ),
        // 权限设置（仅Owner可见）
        if (currentWorkspaceMemberRole?.isOwner == true)
          MobileSettingItem(
            name: "权限设置",
            subtitle: _buildSubtitle(context, "配置智能体访问权限"),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => _showPermissionSettings(context),
          ),
        // 智能体管理
        MobileSettingItem(
          name: "智能体管理",
          subtitle: _buildSubtitle(context, "导入/导出智能体"),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _showAgentManagement(context),
        ),
      ],
    );
  }

  Widget _buildNoPermissionView(BuildContext context) {
    return MobileSettingGroup(
      groupTitle: "智能体配置",
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
                  "您没有权限配置智能体设置。请联系工作空间管理员。",
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

  Widget _buildAgentCount(BuildContext context, int count) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Text(
            count.toString(),
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.onPrimaryContainer,
              fontWeight: FontWeight.w500,
            ),
          ),
        ),
        const HSpace(8),
        const Icon(Icons.chevron_right),
      ],
    );
  }

  void _showAgentList(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "我的智能体",
      builder: (context) => _AgentListView(
        workspaceId: workspaceId,
        userRole: currentWorkspaceMemberRole!,
      ),
    );
  }

  void _showAgentTemplates(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "智能体模板",
      builder: (context) => _AgentTemplatesView(
        workspaceId: workspaceId,
      ),
    );
  }

  void _showToolConfiguration(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "工具配置",
      builder: (context) => _ToolConfigurationView(
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
      builder: (context) => _AgentPermissionSettingsView(
        workspaceId: workspaceId,
      ),
    );
  }

  void _showAgentManagement(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: "智能体管理",
      builder: (context) => _AgentManagementView(
        workspaceId: workspaceId,
        userRole: currentWorkspaceMemberRole!,
      ),
    );
  }
}

/// 智能体列表视图
class _AgentListView extends StatelessWidget {
  const _AgentListView({
    required this.workspaceId,
    required this.userRole,
  });

  final String workspaceId;
  final AFRolePB userRole;

  @override
  Widget build(BuildContext context) {
    // TODO: 从实际数据源获取智能体列表
    final agents = <Map<String, dynamic>>[]; // 模拟空列表

    if (agents.isEmpty) {
      return _buildEmptyState(context);
    }

    return Column(
      children: [
        ...agents.map((agent) => _buildAgentItem(context, agent)),
        const VSpace(16),
        _buildCreateAgentButton(context),
      ],
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          Icon(
            Icons.smart_toy_outlined,
            size: 48,
            color: Theme.of(context).colorScheme.primary.withOpacity(0.6),
          ),
          const VSpace(16),
          Text(
            "暂无智能体",
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const VSpace(8),
          Text(
            "创建智能体以提供专业化的 AI 助手服务",
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          const VSpace(24),
          _buildCreateAgentButton(context),
        ],
      ),
    );
  }

  Widget _buildAgentItem(BuildContext context, Map<String, dynamic> agent) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: Theme.of(context).colorScheme.primaryContainer,
          child: Icon(
            Icons.smart_toy,
            color: Theme.of(context).colorScheme.onPrimaryContainer,
          ),
        ),
        title: Text(agent['name'] as String? ?? ''),
        subtitle: Text(agent['description'] as String? ?? ''),
        trailing: PopupMenuButton<String>(
          onSelected: (value) => _handleAgentAction(context, agent, value),
          itemBuilder: (context) => [
            const PopupMenuItem(
              value: 'edit',
              child: Row(
                children: [
                  Icon(Icons.edit_outlined, size: 20),
                  HSpace(8),
                  Text('编辑'),
                ],
              ),
            ),
            const PopupMenuItem(
              value: 'duplicate',
              child: Row(
                children: [
                  Icon(Icons.copy_outlined, size: 20),
                  HSpace(8),
                  Text('复制'),
                ],
              ),
            ),
            const PopupMenuItem(
              value: 'delete',
              child: Row(
                children: [
                  Icon(Icons.delete_outline, size: 20, color: Colors.red),
                  HSpace(8),
                  Text('删除', style: TextStyle(color: Colors.red)),
                ],
              ),
            ),
          ],
        ),
        onTap: () => _editAgent(context, agent),
      ),
    );
  }

  Widget _buildCreateAgentButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: SizedBox(
        width: double.infinity,
        child: ElevatedButton.icon(
          onPressed: () => _createAgent(context),
          icon: const Icon(Icons.add),
          label: const Text("创建智能体"),
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 12),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(8),
            ),
          ),
        ),
      ),
    );
  }

  void _createAgent(BuildContext context) {
    // TODO: 实现创建智能体对话框
    context.pop();
  }

  void _editAgent(BuildContext context, Map<String, dynamic> agent) {
    // TODO: 实现编辑智能体对话框
  }

  void _handleAgentAction(BuildContext context, Map<String, dynamic> agent, String action) {
    switch (action) {
      case 'edit':
        _editAgent(context, agent);
        break;
      case 'duplicate':
        // TODO: 实现复制智能体
        break;
      case 'delete':
        // TODO: 实现删除智能体确认对话框
        break;
    }
  }
}

/// 智能体模板视图
class _AgentTemplatesView extends StatelessWidget {
  const _AgentTemplatesView({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    // TODO: 从实际数据源获取模板列表
    final templates = [
      {
        'name': '文档助手',
        'description': '专业的文档编写和编辑助手',
        'category': '办公',
        'icon': Icons.description_outlined,
      },
      {
        'name': '代码助手',
        'description': '代码审查、重构和优化专家',
        'category': '开发',
        'icon': Icons.code_outlined,
      },
      {
        'name': '数据分析师',
        'description': '数据处理和分析专家',
        'category': '分析',
        'icon': Icons.analytics_outlined,
      },
    ];

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            "选择模板快速创建智能体",
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        ...templates.map((template) => _buildTemplateItem(context, template)),
      ],
    );
  }

  Widget _buildTemplateItem(BuildContext context, Map<String, dynamic> template) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: Theme.of(context).colorScheme.secondaryContainer,
          child: Icon(
            template['icon'] as IconData,
            color: Theme.of(context).colorScheme.onSecondaryContainer,
          ),
        ),
        title: Text(template['name'] as String),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(template['description'] as String),
            const VSpace(4),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceVariant,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                template['category'] as String,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ),
          ],
        ),
        trailing: const Icon(Icons.add_circle_outline),
        onTap: () => _useTemplate(context, template),
      ),
    );
  }

  void _useTemplate(BuildContext context, Map<String, dynamic> template) {
    // TODO: 实现使用模板创建智能体
    context.pop();
  }
}

/// 工具配置视图
class _ToolConfigurationView extends StatelessWidget {
  const _ToolConfigurationView({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    // TODO: 从实际数据源获取工具列表
    final tools = [
      {'name': '文档操作', 'enabled': true, 'category': '内置'},
      {'name': '搜索引擎', 'enabled': false, 'category': 'MCP'},
      {'name': '代码执行', 'enabled': true, 'category': 'MCP'},
    ];

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            "配置智能体可使用的工具",
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        ...tools.map((tool) => _buildToolItem(context, tool)),
      ],
    );
  }

  Widget _buildToolItem(BuildContext context, Map<String, dynamic> tool) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  tool['name'] as String,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const VSpace(2),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.surfaceVariant,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    tool['category'] as String,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
              ],
            ),
          ),
          Switch(
            value: tool['enabled'] as bool,
            onChanged: (value) => _toggleTool(context, tool, value),
          ),
        ],
      ),
    );
  }

  void _toggleTool(BuildContext context, Map<String, dynamic> tool, bool enabled) {
    // TODO: 实现工具启用/禁用
  }
}

/// 智能体权限设置视图
class _AgentPermissionSettingsView extends StatelessWidget {
  const _AgentPermissionSettingsView({
    required this.workspaceId,
  });

  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _buildPermissionItem(
          context,
          "允许成员创建智能体",
          "工作空间成员可以创建和配置智能体",
          true, // TODO: 从配置中读取
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const Divider(),
        _buildPermissionItem(
          context,
          "允许成员修改智能体",
          "工作空间成员可以修改其他人创建的智能体",
          false, // TODO: 从配置中读取
          (value) {
            // TODO: 更新权限配置
          },
        ),
        const Divider(),
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

/// 智能体管理视图
class _AgentManagementView extends StatelessWidget {
  const _AgentManagementView({
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
          text: "导出智能体",
          showTopBorder: false,
          onTap: () => _exportAgents(context),
          trailing: const Icon(Icons.download_outlined),
        ),
        FlowyOptionTile.text(
          text: "导入智能体",
          showTopBorder: false,
          onTap: () => _importAgents(context),
          trailing: const Icon(Icons.upload_outlined),
        ),
        FlowyOptionTile.text(
          text: "智能体市场",
          showTopBorder: false,
          onTap: () => _openAgentMarket(context),
          trailing: const Icon(Icons.store_outlined),
        ),
        if (userRole.isOwner) ...[
          FlowyOptionTile.text(
            text: "重置智能体",
            showTopBorder: false,
            onTap: () => _resetAgents(context),
            trailing: Icon(
              Icons.restore_outlined,
              color: Theme.of(context).colorScheme.error,
            ),
          ),
        ],
      ],
    );
  }

  void _exportAgents(BuildContext context) {
    // TODO: 实现智能体导出
    context.pop();
  }

  void _importAgents(BuildContext context) {
    // TODO: 实现智能体导入
    context.pop();
  }

  void _openAgentMarket(BuildContext context) {
    // TODO: 实现智能体市场
    context.pop();
  }

  void _resetAgents(BuildContext context) {
    // TODO: 实现智能体重置确认对话框
    context.pop();
  }
}
