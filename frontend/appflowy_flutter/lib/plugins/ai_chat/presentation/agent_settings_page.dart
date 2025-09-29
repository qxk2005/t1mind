import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
import 'package:appflowy/workspace/presentation/widgets/dialogs.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';

/// 智能体配置页面
/// 提供智能体的创建、编辑、删除功能，支持个性化配置和工具选择
class AgentSettingsPage extends StatelessWidget {
  const AgentSettingsPage({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: LocaleKeys.settings_aiPage_keys_agentTitle.tr(),
      description: LocaleKeys.settings_aiPage_keys_agentDescription.tr(),
      children: [
        const _AgentList(),
        const VSpace(16),
        const _AddAgentSection(),
      ],
    );
  }
}

/// 智能体列表组件
class _AgentList extends StatelessWidget {
  const _AgentList();

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: const BorderRadius.all(Radius.circular(8.0)),
      ),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              FlowyText.medium(
                LocaleKeys.settings_aiPage_keys_agentList.tr(),
                color: AFThemeExtension.of(context).strongText,
              ),
              const Spacer(),
              FlowyTextButton(
                LocaleKeys.settings_aiPage_keys_addAgent.tr(),
                fontColor: Theme.of(context).colorScheme.primary,
                onPressed: () => _showAddAgentDialog(context),
              ),
            ],
          ),
          const VSpace(12),
          // TODO: Replace with actual agent list from BLoC
          _buildAgentListItem(
            context,
            name: "通用助手",
            description: "一个通用的AI助手，可以帮助处理各种任务",
            toolCount: 5,
            isActive: true,
          ),
          const VSpace(8),
          _buildAgentListItem(
            context,
            name: "文档专家",
            description: "专门处理文档相关任务的智能体",
            toolCount: 3,
            isActive: false,
          ),
          const VSpace(8),
          _buildAgentListItem(
            context,
            name: "数据分析师",
            description: "专注于数据分析和可视化的智能体",
            toolCount: 7,
            isActive: true,
          ),
        ],
      ),
    );
  }

  Widget _buildAgentListItem(
    BuildContext context, {
    required String name,
    required String description,
    required int toolCount,
    required bool isActive,
  }) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: const BorderRadius.all(Radius.circular(6.0)),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Row(
        children: [
          // 智能体图标
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color: isActive 
                ? Theme.of(context).colorScheme.primary.withOpacity(0.1)
                : Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: const BorderRadius.all(Radius.circular(8.0)),
            ),
            child: Icon(
              Icons.smart_toy_outlined,
              color: isActive 
                ? Theme.of(context).colorScheme.primary
                : AFThemeExtension.of(context).secondaryTextColor,
              size: 20,
            ),
          ),
          const HSpace(12),
          // 智能体信息
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    FlowyText.medium(
                      name,
                      color: AFThemeExtension.of(context).strongText,
                    ),
                    const HSpace(8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: isActive 
                          ? Theme.of(context).colorScheme.primary.withOpacity(0.1)
                          : Theme.of(context).colorScheme.surfaceContainerHighest,
                        borderRadius: const BorderRadius.all(Radius.circular(4.0)),
                      ),
                      child: FlowyText.regular(
                        isActive 
                          ? LocaleKeys.settings_aiPage_keys_agentStatusActive.tr()
                          : LocaleKeys.settings_aiPage_keys_agentStatusInactive.tr(),
                        fontSize: 10,
                        color: isActive 
                          ? Theme.of(context).colorScheme.primary
                          : AFThemeExtension.of(context).secondaryTextColor,
                      ),
                    ),
                  ],
                ),
                const VSpace(4),
                FlowyText.regular(
                  description,
                  fontSize: 12,
                  color: AFThemeExtension.of(context).secondaryTextColor,
                  maxLines: 2,
                ),
                const VSpace(4),
                FlowyText.regular(
                  LocaleKeys.settings_aiPage_keys_agentToolCount.tr(args: [toolCount.toString()]),
                  fontSize: 11,
                  color: AFThemeExtension.of(context).secondaryTextColor,
                ),
              ],
            ),
          ),
          // 操作按钮
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              FlowyIconButton(
                icon: const FlowySvg(FlowySvgs.settings_s),
                iconColorOnHover: Theme.of(context).colorScheme.primary,
                onPressed: () => _showConfigureAgentDialog(context, name),
                tooltipText: LocaleKeys.settings_aiPage_keys_configureAgent.tr(),
              ),
              const HSpace(4),
              FlowyIconButton(
                icon: const FlowySvg(FlowySvgs.delete_s),
                iconColorOnHover: Theme.of(context).colorScheme.error,
                onPressed: () => _showDeleteAgentDialog(context, name),
                tooltipText: LocaleKeys.settings_aiPage_keys_deleteAgent.tr(),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _showAddAgentDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const _AddAgentDialog(),
    );
  }

  void _showConfigureAgentDialog(BuildContext context, String agentName) {
    showDialog(
      context: context,
      builder: (context) => _ConfigureAgentDialog(agentName: agentName),
    );
  }

  void _showDeleteAgentDialog(BuildContext context, String agentName) {
    showConfirmDialog(
      context: context,
      title: LocaleKeys.settings_aiPage_keys_deleteAgentTitle.tr(),
      description: LocaleKeys.settings_aiPage_keys_deleteAgentMessage.tr(args: [agentName]),
      confirmLabel: LocaleKeys.button_delete.tr(),
      onConfirm: (_) {
        // TODO: Implement agent deletion
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(LocaleKeys.settings_aiPage_keys_agentDeleted.tr(args: [agentName])),
          ),
        );
      },
    );
  }
}

/// 添加智能体区域
class _AddAgentSection extends StatelessWidget {
  const _AddAgentSection();

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: const BorderRadius.all(Radius.circular(8.0)),
      ),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FlowyText.medium(
            LocaleKeys.settings_aiPage_keys_agentQuickStart.tr(),
            color: AFThemeExtension.of(context).strongText,
          ),
          const VSpace(8),
          FlowyText.regular(
            LocaleKeys.settings_aiPage_keys_agentQuickStartDescription.tr(),
            color: AFThemeExtension.of(context).secondaryTextColor,
            maxLines: 3,
          ),
          const VSpace(12),
          Row(
            children: [
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(
                    LocaleKeys.settings_aiPage_keys_addAgent.tr(),
                  ),
                  onTap: () => _showAddAgentDialog(context),
                ),
              ),
              const HSpace(12),
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(
                    LocaleKeys.settings_aiPage_keys_importAgentTemplate.tr(),
                  ),
                  onTap: () => _showImportTemplateDialog(context),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _showAddAgentDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const _AddAgentDialog(),
    );
  }

  void _showImportTemplateDialog(BuildContext context) {
    // TODO: Implement template import dialog
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(LocaleKeys.settings_aiPage_keys_featureComingSoon.tr()),
      ),
    );
  }
}

/// 添加智能体对话框
class _AddAgentDialog extends StatefulWidget {
  const _AddAgentDialog();

  @override
  State<_AddAgentDialog> createState() => _AddAgentDialogState();
}

class _AddAgentDialogState extends State<_AddAgentDialog> {
  final _nameController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _personalityController = TextEditingController();
  final _instructionsController = TextEditingController();
  
  bool _isActive = true;
  final Set<String> _selectedTools = <String>{};
  final List<String> _availableTools = [
    'AppFlowy文档操作',
    'Web搜索',
    '文件系统访问',
    '数据库查询',
    '图像生成',
    '代码执行',
    '邮件发送',
  ];

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _personalityController.dispose();
    _instructionsController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      title: FlowyText.medium(LocaleKeys.settings_aiPage_keys_addAgent.tr()),
      child: SizedBox(
        width: 600,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 基本信息
            _buildBasicInfoSection(),
            const VSpace(16),
            // 个性化配置
            _buildPersonalitySection(),
            const VSpace(16),
            // 工具选择
            _buildToolSelectionSection(),
            const VSpace(16),
            // 高级设置
            _buildAdvancedSection(),
            const VSpace(24),
            // 操作按钮
            _buildActionButtons(),
          ],
        ),
      ),
    );
  }

  Widget _buildBasicInfoSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentBasicInfo.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_agentName.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_agentNameHint.tr(),
          textController: _nameController,
          hideActions: true,
        ),
        const VSpace(12),
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_agentDescription.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_agentDescriptionHint.tr(),
          textController: _descriptionController,
          hideActions: true,
        ),
      ],
    );
  }

  Widget _buildPersonalitySection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentPersonality.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            border: Border.all(color: Theme.of(context).colorScheme.outline),
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.regular(
                LocaleKeys.settings_aiPage_keys_agentPersonalityDescription.tr(),
                fontSize: 12,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
              const VSpace(8),
              TextField(
                controller: _personalityController,
                maxLines: 3,
                decoration: InputDecoration(
                  hintText: LocaleKeys.settings_aiPage_keys_agentPersonalityHint.tr(),
                  border: InputBorder.none,
                ),
              ),
            ],
          ),
        ),
        const VSpace(12),
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_agentInstructions.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_agentInstructionsHint.tr(),
          textController: _instructionsController,
          hideActions: true,
        ),
      ],
    );
  }

  Widget _buildToolSelectionSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentTools.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(8),
        FlowyText.regular(
          LocaleKeys.settings_aiPage_keys_agentToolsDescription.tr(),
          fontSize: 12,
          color: AFThemeExtension.of(context).secondaryTextColor,
        ),
        const VSpace(12),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            border: Border.all(color: Theme.of(context).colorScheme.outline),
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          child: Column(
            children: _availableTools.map((tool) {
              final isSelected = _selectedTools.contains(tool);
              return CheckboxListTile(
                title: FlowyText.regular(tool),
                value: isSelected,
                onChanged: (value) {
                  setState(() {
                    if (value == true) {
                      _selectedTools.add(tool);
                    } else {
                      _selectedTools.remove(tool);
                    }
                  });
                },
                controlAffinity: ListTileControlAffinity.leading,
                contentPadding: EdgeInsets.zero,
              );
            }).toList(),
          ),
        ),
      ],
    );
  }

  Widget _buildAdvancedSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentAdvanced.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        Row(
          children: [
            Switch(
              value: _isActive,
              onChanged: (value) {
                setState(() {
                  _isActive = value;
                });
              },
            ),
            const HSpace(8),
            FlowyText.regular(
              LocaleKeys.settings_aiPage_keys_agentActiveOnCreation.tr(),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildActionButtons() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        FlowyTextButton(
          LocaleKeys.button_cancel.tr(),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          onPressed: () => Navigator.of(context).pop(),
        ),
        const HSpace(8),
        FlowyButton(
          text: FlowyText.regular(
            LocaleKeys.button_create.tr(),
            color: Colors.white,
          ),
          onTap: _createAgent,
        ),
      ],
    );
  }

  void _createAgent() {
    if (_nameController.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(LocaleKeys.settings_aiPage_keys_agentNameRequired.tr()),
        ),
      );
      return;
    }

    // TODO: Implement agent creation logic
    Navigator.of(context).pop();
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(LocaleKeys.settings_aiPage_keys_agentCreated.tr(args: [_nameController.text])),
      ),
    );
  }
}

/// 配置智能体对话框
class _ConfigureAgentDialog extends StatefulWidget {
  const _ConfigureAgentDialog({required this.agentName});

  final String agentName;

  @override
  State<_ConfigureAgentDialog> createState() => _ConfigureAgentDialogState();
}

class _ConfigureAgentDialogState extends State<_ConfigureAgentDialog> {
  final _nameController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _personalityController = TextEditingController();
  final _instructionsController = TextEditingController();
  
  bool _isActive = true;
  final Set<String> _selectedTools = <String>{'AppFlowy文档操作', 'Web搜索'};
  final List<String> _availableTools = [
    'AppFlowy文档操作',
    'Web搜索',
    '文件系统访问',
    '数据库查询',
    '图像生成',
    '代码执行',
    '邮件发送',
  ];

  @override
  void initState() {
    super.initState();
    // TODO: Load existing agent configuration
    _nameController.text = widget.agentName;
    _descriptionController.text = "一个通用的AI助手，可以帮助处理各种任务";
    _personalityController.text = "友好、专业、乐于助人";
    _instructionsController.text = "始终以用户需求为中心，提供准确和有用的帮助";
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _personalityController.dispose();
    _instructionsController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      title: FlowyText.medium(LocaleKeys.settings_aiPage_keys_configureAgent.tr(args: [widget.agentName])),
      child: SizedBox(
        width: 600,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 基本信息
            _buildBasicInfoSection(),
            const VSpace(16),
            // 个性化配置
            _buildPersonalitySection(),
            const VSpace(16),
            // 工具选择
            _buildToolSelectionSection(),
            const VSpace(16),
            // 高级设置
            _buildAdvancedSection(),
            const VSpace(24),
            // 操作按钮
            _buildActionButtons(),
          ],
        ),
      ),
    );
  }

  Widget _buildBasicInfoSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentBasicInfo.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_agentName.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_agentNameHint.tr(),
          textController: _nameController,
          hideActions: true,
        ),
        const VSpace(12),
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_agentDescription.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_agentDescriptionHint.tr(),
          textController: _descriptionController,
          hideActions: true,
        ),
      ],
    );
  }

  Widget _buildPersonalitySection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentPersonality.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            border: Border.all(color: Theme.of(context).colorScheme.outline),
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.regular(
                LocaleKeys.settings_aiPage_keys_agentPersonalityDescription.tr(),
                fontSize: 12,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
              const VSpace(8),
              TextField(
                controller: _personalityController,
                maxLines: 3,
                decoration: InputDecoration(
                  hintText: LocaleKeys.settings_aiPage_keys_agentPersonalityHint.tr(),
                  border: InputBorder.none,
                ),
              ),
            ],
          ),
        ),
        const VSpace(12),
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_agentInstructions.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_agentInstructionsHint.tr(),
          textController: _instructionsController,
          hideActions: true,
        ),
      ],
    );
  }

  Widget _buildToolSelectionSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentTools.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(8),
        FlowyText.regular(
          LocaleKeys.settings_aiPage_keys_agentToolsDescription.tr(),
          fontSize: 12,
          color: AFThemeExtension.of(context).secondaryTextColor,
        ),
        const VSpace(12),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            border: Border.all(color: Theme.of(context).colorScheme.outline),
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          child: Column(
            children: _availableTools.map((tool) {
              final isSelected = _selectedTools.contains(tool);
              return CheckboxListTile(
                title: FlowyText.regular(tool),
                value: isSelected,
                onChanged: (value) {
                  setState(() {
                    if (value == true) {
                      _selectedTools.add(tool);
                    } else {
                      _selectedTools.remove(tool);
                    }
                  });
                },
                controlAffinity: ListTileControlAffinity.leading,
                contentPadding: EdgeInsets.zero,
              );
            }).toList(),
          ),
        ),
      ],
    );
  }

  Widget _buildAdvancedSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_agentAdvanced.tr(),
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        Row(
          children: [
            Switch(
              value: _isActive,
              onChanged: (value) {
                setState(() {
                  _isActive = value;
                });
              },
            ),
            const HSpace(8),
            FlowyText.regular(
              LocaleKeys.settings_aiPage_keys_agentIsActive.tr(),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildActionButtons() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        FlowyTextButton(
          LocaleKeys.button_cancel.tr(),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          onPressed: () => Navigator.of(context).pop(),
        ),
        const HSpace(8),
        FlowyButton(
          text: FlowyText.regular(
            LocaleKeys.button_save.tr(),
            color: Colors.white,
          ),
          onTap: _saveAgent,
        ),
      ],
    );
  }

  void _saveAgent() {
    if (_nameController.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(LocaleKeys.settings_aiPage_keys_agentNameRequired.tr()),
        ),
      );
      return;
    }

    // TODO: Implement agent update logic
    Navigator.of(context).pop();
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(LocaleKeys.settings_aiPage_keys_agentUpdated.tr(args: [_nameController.text])),
      ),
    );
  }
}
