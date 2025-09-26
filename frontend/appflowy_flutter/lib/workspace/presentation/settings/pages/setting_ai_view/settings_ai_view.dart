import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/model_selection.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/provider_selector.dart';
import 'package:appflowy/workspace/application/settings/ai/ai_provider_cubit.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/widgets/toggle/toggle.dart';
import 'package:appflowy/plugins/ai_chat/application/agent_config_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart' as log_entities;
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class SettingsAIView extends StatelessWidget {
  const SettingsAIView({
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
    return BlocProvider<SettingsAIBloc>(
      create: (_) => SettingsAIBloc(userProfile, workspaceId)
        ..add(const SettingsAIEvent.started()),
      child: MultiBlocProvider(
        providers: [
          BlocProvider(
            create: (_) => AiProviderCubit(workspaceId: workspaceId),
          ),
          BlocProvider(
            create: (_) => AgentConfigBloc(userProfile, workspaceId)
              ..add(const AgentConfigEvent.loadConfigs()),
          ),
        ],
        child: SettingsBody(
          title: LocaleKeys.settings_aiPage_title.tr(),
          description: LocaleKeys.settings_aiPage_keys_aiSettingsDescription.tr(),
          children: [
            const ProviderDropdown(),
            const AIModelSelection(),
            const _AISearchToggle(value: false),
            ProviderTabSwitcher(workspaceId: workspaceId),
            const SizedBox(height: 20),
            const _AgentConfigSection(),
            const SizedBox(height: 20),
            const _McpToolsSection(),
          ],
        ),
      ),
    );
  }
}

class _AISearchToggle extends StatelessWidget {
  const _AISearchToggle({required this.value});

  final bool value;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            FlowyText.medium(
              LocaleKeys.settings_aiPage_keys_enableAISearchTitle.tr(),
            ),
            const Spacer(),
            BlocBuilder<SettingsAIBloc, SettingsAIState>(
              builder: (context, state) {
                if (state.aiSettings == null) {
                  return const Padding(
                    padding: EdgeInsets.only(top: 6),
                    child: SizedBox(
                      height: 26,
                      width: 26,
                      child: CircularProgressIndicator.adaptive(),
                    ),
                  );
                } else {
                  return Toggle(
                    value: state.enableSearchIndexing,
                    onChanged: (_) => context
                        .read<SettingsAIBloc>()
                        .add(const SettingsAIEvent.toggleAISearch()),
                  );
                }
              },
            ),
          ],
        ),
      ],
    );
  }
}

// Provider UI 已迁移到 provider_selector.dart

/// 智能体配置管理区域
class _AgentConfigSection extends StatelessWidget {
  const _AgentConfigSection();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          '智能体配置',
          fontSize: 16,
        ),
        const SizedBox(height: 8),
        FlowyText.regular(
          '管理和配置AI智能体，包括个性化设置、工具权限和行为参数',
          fontSize: 12,
          color: Theme.of(context).hintColor,
        ),
        const SizedBox(height: 16),
        BlocBuilder<AgentConfigBloc, AgentConfigState>(
          builder: (context, state) {
            if (state.isLoading) {
              return const Center(
                child: CircularProgressIndicator.adaptive(),
              );
            }

            return Column(
              children: [
                // 默认智能体选择
                _DefaultAgentSelector(
                  configs: state.configs,
                  defaultConfigId: state.defaultConfigId,
                  onDefaultChanged: (configId) {
                    context.read<AgentConfigBloc>().add(
                      AgentConfigEvent.setDefaultConfig(configId),
                    );
                  },
                ),
                const SizedBox(height: 12),
                
                // 智能体列表
                _AgentConfigList(
                  configs: state.displayConfigs,
                  onEdit: (config) => _showAgentConfigDialog(context, config),
                  onDelete: (config) {
                    context.read<AgentConfigBloc>().add(
                      AgentConfigEvent.deleteConfig(config.id),
                    );
                  },
                  onToggleEnabled: (config) {
                    context.read<AgentConfigBloc>().add(
                      AgentConfigEvent.updateConfig(
                        config.copyWith(isEnabled: !config.isEnabled),
                      ),
                    );
                  },
                ),
                const SizedBox(height: 12),
                
                // 操作按钮
                Row(
                  children: [
                    FlowyButton(
                      text: FlowyText.regular('新建智能体'),
                      onTap: () => _showAgentConfigDialog(context, null),
                    ),
                    const SizedBox(width: 8),
                    FlowyButton(
                      text: FlowyText.regular('导入配置'),
                      onTap: () => _showImportDialog(context),
                    ),
                    const SizedBox(width: 8),
                    FlowyButton(
                      text: FlowyText.regular('导出配置'),
                      onTap: () {
                        context.read<AgentConfigBloc>().add(
                          const AgentConfigEvent.exportConfigs(),
                        );
                      },
                    ),
                  ],
                ),
                
                // 错误和成功消息
                if (state.errorMessage != null) ...[
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.error.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Row(
                      children: [
                        Icon(
                          Icons.error_outline,
                          size: 16,
                          color: Theme.of(context).colorScheme.error,
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: FlowyText.regular(
                            state.errorMessage!,
                            color: Theme.of(context).colorScheme.error,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
                
                if (state.successMessage != null) ...[
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: Colors.green.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Row(
                      children: [
                        const Icon(
                          Icons.check_circle_outline,
                          size: 16,
                          color: Colors.green,
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: FlowyText.regular(
                            state.successMessage!,
                            color: Colors.green,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            );
          },
        ),
      ],
    );
  }

  void _showAgentConfigDialog(BuildContext context, AgentConfig? config) {
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: context.read<AgentConfigBloc>(),
        child: _AgentConfigDialog(config: config),
      ),
    );
  }

  void _showImportDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: context.read<AgentConfigBloc>(),
        child: const _ImportConfigDialog(),
      ),
    );
  }
}

/// 默认智能体选择器
class _DefaultAgentSelector extends StatelessWidget {
  const _DefaultAgentSelector({
    required this.configs,
    required this.defaultConfigId,
    required this.onDefaultChanged,
  });

  final List<AgentConfig> configs;
  final String? defaultConfigId;
  final ValueChanged<String?> onDefaultChanged;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        FlowyText.medium('默认智能体:'),
        const SizedBox(width: 12),
        Expanded(
          child: DropdownButton<String?>(
            value: defaultConfigId,
            hint: const Text('选择默认智能体'),
            isExpanded: true,
            items: [
              const DropdownMenuItem<String?>(
                value: null,
                child: Text('无'),
              ),
              ...configs.where((c) => c.isEnabled).map(
                (config) => DropdownMenuItem<String?>(
                  value: config.id,
                  child: Text(config.name),
                ),
              ),
            ],
            onChanged: onDefaultChanged,
          ),
        ),
      ],
    );
  }
}

/// 智能体配置列表
class _AgentConfigList extends StatelessWidget {
  const _AgentConfigList({
    required this.configs,
    required this.onEdit,
    required this.onDelete,
    required this.onToggleEnabled,
  });

  final List<AgentConfig> configs;
  final ValueChanged<AgentConfig> onEdit;
  final ValueChanged<AgentConfig> onDelete;
  final ValueChanged<AgentConfig> onToggleEnabled;

  @override
  Widget build(BuildContext context) {
    if (configs.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          border: Border.all(color: Theme.of(context).dividerColor),
          borderRadius: BorderRadius.circular(4),
        ),
        child: const Center(
          child: FlowyText.regular('暂无智能体配置'),
        ),
      );
    }

    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(4),
      ),
      child: ListView.separated(
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        itemCount: configs.length,
        separatorBuilder: (context, index) => Divider(
          height: 1,
          color: Theme.of(context).dividerColor,
        ),
        itemBuilder: (context, index) {
          final config = configs[index];
          return ListTile(
            leading: CircleAvatar(
              backgroundImage: config.avatarUrl != null
                  ? NetworkImage(config.avatarUrl!)
                  : null,
              child: config.avatarUrl == null
                  ? Text(config.name.isNotEmpty ? config.name[0] : 'A')
                  : null,
            ),
            title: FlowyText.medium(config.name),
            subtitle: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (config.description.isNotEmpty)
                  FlowyText.regular(
                    config.description,
                    fontSize: 12,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                Row(
                  children: [
                    Icon(
                      config.isEnabled ? Icons.check_circle : Icons.cancel,
                      size: 12,
                      color: config.isEnabled ? Colors.green : Colors.grey,
                    ),
                    const SizedBox(width: 4),
                    FlowyText.regular(
                      config.isEnabled ? '已启用' : '已禁用',
                      fontSize: 10,
                      color: config.isEnabled ? Colors.green : Colors.grey,
                    ),
                    const SizedBox(width: 12),
                    FlowyText.regular(
                      '工具: ${config.allowedTools.length}',
                      fontSize: 10,
                      color: Theme.of(context).hintColor,
                    ),
                  ],
                ),
              ],
            ),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                IconButton(
                  icon: Icon(
                    config.isEnabled ? Icons.toggle_on : Icons.toggle_off,
                    color: config.isEnabled ? Colors.green : Colors.grey,
                  ),
                  onPressed: () => onToggleEnabled(config),
                  tooltip: config.isEnabled ? '禁用' : '启用',
                ),
                IconButton(
                  icon: const Icon(Icons.edit),
                  onPressed: () => onEdit(config),
                  tooltip: '编辑',
                ),
                IconButton(
                  icon: const Icon(Icons.delete),
                  onPressed: () => onDelete(config),
                  tooltip: '删除',
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}

/// MCP工具管理区域
class _McpToolsSection extends StatefulWidget {
  const _McpToolsSection();

  @override
  State<_McpToolsSection> createState() => _McpToolsSectionState();
}

class _McpToolsSectionState extends State<_McpToolsSection> {
  List<log_entities.McpToolInfo> _availableTools = [];
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _loadAvailableTools();
  }

  Future<void> _loadAvailableTools() async {
    setState(() {
      _isLoading = true;
    });

    try {
      // TODO: 从Rust层获取可用的MCP工具列表
      // 这里暂时使用模拟数据
      await Future.delayed(const Duration(milliseconds: 500));
      
      _availableTools = [
        log_entities.McpToolInfo(
          id: 'file-manager',
          name: 'File Manager',
          displayName: '文件管理器',
          description: '管理文件和目录操作',
          category: 'System',
          status: log_entities.McpToolStatus.available,
          provider: 'AppFlowy',
          version: '1.0.0',
          usageCount: 15,
          successCount: 14,
          failureCount: 1,
          averageExecutionTimeMs: 250,
          lastChecked: DateTime.now().subtract(const Duration(minutes: 5)),
          lastUsed: DateTime.now().subtract(const Duration(hours: 2)),
        ),
        log_entities.McpToolInfo(
          id: 'web-search',
          name: 'Web Search',
          displayName: '网络搜索',
          description: '搜索互联网信息',
          category: 'Search',
          status: log_entities.McpToolStatus.available,
          provider: 'Google',
          version: '2.1.0',
          usageCount: 42,
          successCount: 40,
          failureCount: 2,
          averageExecutionTimeMs: 1200,
          lastChecked: DateTime.now().subtract(const Duration(minutes: 1)),
          lastUsed: DateTime.now().subtract(const Duration(minutes: 30)),
        ),
        log_entities.McpToolInfo(
          id: 'code-analyzer',
          name: 'Code Analyzer',
          displayName: '代码分析器',
          description: '分析和理解代码结构',
          category: 'Development',
          status: log_entities.McpToolStatus.unavailable,
          provider: 'AppFlowy',
          version: '1.5.0',
          usageCount: 8,
          successCount: 6,
          failureCount: 2,
          averageExecutionTimeMs: 800,
          lastChecked: DateTime.now().subtract(const Duration(minutes: 10)),
          lastUsed: DateTime.now().subtract(const Duration(days: 1)),
        ),
      ];
    } catch (e) {
      // 处理错误
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          'MCP工具管理',
          fontSize: 16,
        ),
        const SizedBox(height: 8),
        FlowyText.regular(
          '管理和配置MCP（Model Context Protocol）工具，控制AI可以使用的外部工具',
          fontSize: 12,
          color: Theme.of(context).hintColor,
        ),
        const SizedBox(height: 16),
        
        if (_isLoading)
          const Center(child: CircularProgressIndicator.adaptive())
        else ...[
          // 工具统计信息
          _McpToolsStats(tools: _availableTools),
          const SizedBox(height: 16),
          
          // 工具列表
          _McpToolsList(
            tools: _availableTools,
            onRefresh: _loadAvailableTools,
          ),
          const SizedBox(height: 16),
          
          // 操作按钮
          Row(
            children: [
              FlowyButton(
                text: FlowyText.regular('刷新工具列表'),
                onTap: _loadAvailableTools,
              ),
              const SizedBox(width: 8),
              FlowyButton(
                text: FlowyText.regular('添加工具'),
                onTap: () => _showAddToolDialog(context),
              ),
              const SizedBox(width: 8),
              FlowyButton(
                text: FlowyText.regular('工具设置'),
                onTap: () => _showToolSettingsDialog(context),
              ),
            ],
          ),
        ],
      ],
    );
  }

  void _showAddToolDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const _AddMcpToolDialog(),
    );
  }

  void _showToolSettingsDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => _McpToolSettingsDialog(tools: _availableTools),
    );
  }
}

/// MCP工具统计信息
class _McpToolsStats extends StatelessWidget {
  const _McpToolsStats({required this.tools});

  final List<log_entities.McpToolInfo> tools;

  @override
  Widget build(BuildContext context) {
    final availableCount = tools.where((t) => t.status == log_entities.McpToolStatus.available).length;
    final totalUsage = tools.fold(0, (sum, tool) => sum + tool.usageCount);
    final totalSuccess = tools.fold(0, (sum, tool) => sum + tool.successCount);
    final successRate = totalUsage > 0 ? (totalSuccess / totalUsage * 100).toStringAsFixed(1) : '0.0';

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Row(
        children: [
          Expanded(
            child: _StatItem(
              label: '可用工具',
              value: '$availableCount/${tools.length}',
              icon: Icons.build,
            ),
          ),
          Expanded(
            child: _StatItem(
              label: '总使用次数',
              value: totalUsage.toString(),
              icon: Icons.analytics,
            ),
          ),
          Expanded(
            child: _StatItem(
              label: '成功率',
              value: '$successRate%',
              icon: Icons.check_circle,
            ),
          ),
        ],
      ),
    );
  }
}

/// 统计项组件
class _StatItem extends StatelessWidget {
  const _StatItem({
    required this.label,
    required this.value,
    required this.icon,
  });

  final String label;
  final String value;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Icon(icon, size: 20, color: Theme.of(context).primaryColor),
        const SizedBox(height: 4),
        FlowyText.medium(value),
        FlowyText.regular(
          label,
          fontSize: 10,
          color: Theme.of(context).hintColor,
        ),
      ],
    );
  }
}

/// MCP工具列表
class _McpToolsList extends StatelessWidget {
  const _McpToolsList({
    required this.tools,
    required this.onRefresh,
  });

  final List<log_entities.McpToolInfo> tools;
  final VoidCallback onRefresh;

  @override
  Widget build(BuildContext context) {
    if (tools.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          border: Border.all(color: Theme.of(context).dividerColor),
          borderRadius: BorderRadius.circular(4),
        ),
        child: const Center(
          child: FlowyText.regular('暂无可用的MCP工具'),
        ),
      );
    }

    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(4),
      ),
      child: ListView.separated(
        shrinkWrap: true,
        physics: const NeverScrollableScrollPhysics(),
        itemCount: tools.length,
        separatorBuilder: (context, index) => Divider(
          height: 1,
          color: Theme.of(context).dividerColor,
        ),
        itemBuilder: (context, index) {
          final tool = tools[index];
          return _McpToolListItem(tool: tool);
        },
      ),
    );
  }
}

/// MCP工具列表项
class _McpToolListItem extends StatelessWidget {
  const _McpToolListItem({required this.tool});

  final log_entities.McpToolInfo tool;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: _getStatusIcon(tool.status),
      title: FlowyText.medium(tool.displayName ?? tool.name),
      subtitle: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FlowyText.regular(
            tool.description,
            fontSize: 12,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 4),
          Row(
            children: [
              FlowyText.regular(
                '${tool.provider} • v${tool.version}',
                fontSize: 10,
                color: Theme.of(context).hintColor,
              ),
              const SizedBox(width: 12),
              FlowyText.regular(
                '使用: ${tool.usageCount}次',
                fontSize: 10,
                color: Theme.of(context).hintColor,
              ),
              if (tool.usageCount > 0) ...[
                const SizedBox(width: 12),
                FlowyText.regular(
                  '成功率: ${(tool.successCount / tool.usageCount * 100).toStringAsFixed(1)}%',
                  fontSize: 10,
                  color: Theme.of(context).hintColor,
                ),
              ],
            ],
          ),
        ],
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (tool.status == log_entities.McpToolStatus.available)
            IconButton(
              icon: const Icon(Icons.settings),
              onPressed: () => _showToolConfigDialog(context, tool),
              tooltip: '配置',
            ),
          IconButton(
            icon: const Icon(Icons.info),
            onPressed: () => _showToolInfoDialog(context, tool),
            tooltip: '详情',
          ),
        ],
      ),
    );
  }

  Widget _getStatusIcon(log_entities.McpToolStatus status) {
    switch (status) {
      case log_entities.McpToolStatus.available:
        return const Icon(Icons.check_circle, color: Colors.green);
      case log_entities.McpToolStatus.unavailable:
        return const Icon(Icons.cancel, color: Colors.red);
      case log_entities.McpToolStatus.error:
        return const Icon(Icons.error, color: Colors.orange);
      case log_entities.McpToolStatus.unknown:
      default:
        return const Icon(Icons.help, color: Colors.grey);
    }
  }

  void _showToolConfigDialog(BuildContext context, log_entities.McpToolInfo tool) {
    showDialog(
      context: context,
      builder: (context) => _McpToolConfigDialog(tool: tool),
    );
  }

  void _showToolInfoDialog(BuildContext context, log_entities.McpToolInfo tool) {
    showDialog(
      context: context,
      builder: (context) => _McpToolInfoDialog(tool: tool),
    );
  }
}

// 占位符对话框组件 - 这些将在后续实现中完善
class _AgentConfigDialog extends StatelessWidget {
  const _AgentConfigDialog({this.config});

  final AgentConfig? config;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(config == null ? '新建智能体' : '编辑智能体'),
      content: const Text('智能体配置对话框 - 待实现'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('保存'),
        ),
      ],
    );
  }
}

class _ImportConfigDialog extends StatelessWidget {
  const _ImportConfigDialog();

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('导入配置'),
      content: const Text('导入配置对话框 - 待实现'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('导入'),
        ),
      ],
    );
  }
}

class _AddMcpToolDialog extends StatelessWidget {
  const _AddMcpToolDialog();

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('添加MCP工具'),
      content: const Text('添加MCP工具对话框 - 待实现'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('添加'),
        ),
      ],
    );
  }
}

class _McpToolSettingsDialog extends StatelessWidget {
  const _McpToolSettingsDialog({required this.tools});

  final List<log_entities.McpToolInfo> tools;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('工具设置'),
      content: const Text('工具设置对话框 - 待实现'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('关闭'),
        ),
      ],
    );
  }
}

class _McpToolConfigDialog extends StatelessWidget {
  const _McpToolConfigDialog({required this.tool});

  final log_entities.McpToolInfo tool;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text('配置 ${tool.displayName ?? tool.name}'),
      content: const Text('工具配置对话框 - 待实现'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('保存'),
        ),
      ],
    );
  }
}

class _McpToolInfoDialog extends StatelessWidget {
  const _McpToolInfoDialog({required this.tool});

  final log_entities.McpToolInfo tool;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(tool.displayName ?? tool.name),
      content: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            _InfoRow('描述', tool.description),
            _InfoRow('提供者', tool.provider),
            _InfoRow('版本', tool.version),
            _InfoRow('分类', tool.category),
            _InfoRow('状态', _getStatusText(tool.status)),
            _InfoRow('使用次数', tool.usageCount.toString()),
            _InfoRow('成功次数', tool.successCount.toString()),
            _InfoRow('失败次数', tool.failureCount.toString()),
            if (tool.averageExecutionTimeMs > 0)
              _InfoRow('平均执行时间', '${tool.averageExecutionTimeMs}ms'),
            if (tool.lastUsed != null)
              _InfoRow('最后使用', _formatDateTime(tool.lastUsed!)),
            if (tool.lastChecked != null)
              _InfoRow('最后检查', _formatDateTime(tool.lastChecked!)),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('关闭'),
        ),
      ],
    );
  }

  Widget _InfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 80,
            child: FlowyText.medium('$label:'),
          ),
          Expanded(
            child: FlowyText.regular(value),
          ),
        ],
      ),
    );
  }

  String _getStatusText(log_entities.McpToolStatus status) {
    switch (status) {
      case log_entities.McpToolStatus.available:
        return '可用';
      case log_entities.McpToolStatus.unavailable:
        return '不可用';
      case log_entities.McpToolStatus.error:
        return '错误';
      case log_entities.McpToolStatus.unknown:
      default:
        return '未知';
    }
  }

  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}-${dateTime.month.toString().padLeft(2, '0')}-${dateTime.day.toString().padLeft(2, '0')} '
           '${dateTime.hour.toString().padLeft(2, '0')}:${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
