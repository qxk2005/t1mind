import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/mobile/presentation/bottom_sheet/show_mobile_bottom_sheet.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy/mobile/presentation/widgets/flowy_option_tile.dart';
import 'package:appflowy/plugins/ai_chat/application/agent_config_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_profile.pb.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/primary_rounded_button.dart';
import 'package:flowy_infra_ui/style_widget/text_field.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

class MobileAgentConfigPage extends StatelessWidget {
  const MobileAgentConfigPage({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  static const routeName = '/settings/ai/agent_config';

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => AgentConfigBloc(userProfile, workspaceId)
        ..add(const AgentConfigEvent.loadConfigs()),
      child: Scaffold(
        appBar: AppBar(
          title: Text('settings.aiPage.keys.agentConfigTitle'.tr()),
          actions: [
            IconButton(
              icon: const Icon(Icons.add),
              tooltip: LocaleKeys.button_create.tr(),
              onPressed: () => _onCreateAgent(context),
            ),
          ],
        ),
        body: BlocBuilder<AgentConfigBloc, AgentConfigState>(
          builder: (context, state) {
            if (state.isLoading) {
              return const Center(child: CircularProgressIndicator());
            }

            if (state.errorMessage != null) {
              return Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      state.errorMessage!,
                      style: AppFlowyTheme.of(context).textStyle.body.standard(
                        color: AppFlowyTheme.of(context).textColorScheme.error,
                      ),
                    ),
                    const SizedBox(height: 16),
                    OutlinedRoundedButton(
                      text: LocaleKeys.button_retry.tr(),
                      onTap: () => context
                          .read<AgentConfigBloc>()
                          .add(const AgentConfigEvent.loadConfigs()),
                    ),
                  ],
                ),
              );
            }

            return ListView(
              padding: const EdgeInsets.all(12),
              children: [
                MobileSettingGroup(
                  groupTitle: 'settings.aiPage.keys.agentConfigTitle'.tr(),
                  showDivider: false,
                  settingItemList: [
                    if (state.configs.isEmpty)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 16),
                        child: Text(
                          'settings.aiPage.keys.agentConfigEmptyHint'.tr(),
                          style: AppFlowyTheme.of(context)
                              .textStyle
                              .body
                              .standard(
                                color: AppFlowyTheme.of(context)
                                    .textColorScheme
                                    .secondary,
                              ),
                          textAlign: TextAlign.center,
                        ),
                      )
                    else
                      ...state.configs.map(
                        (config) => MobileSettingItem(
                          title: _AgentConfigTile(
                            config: config,
                            isDefault: config.id == state.defaultConfigId,
                            onTap: () => _onEditAgent(context, config),
                            onSetDefault: () => _onSetDefaultAgent(context, config),
                            onDelete: () => _onDeleteAgent(context, config),
                          ),
                        ),
                      ),
                  ],
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  void _onCreateAgent(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      title: 'settings.aiPage.keys.createAgentTitle'.tr(),
      builder: (_) => _MobileAgentConfigForm(
        onSave: (config) {
          context.read<AgentConfigBloc>().add(
                AgentConfigEvent.createConfig(config),
              );
          context.pop();
        },
      ),
    );
  }

  void _onEditAgent(BuildContext context, AgentConfig config) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      title: 'settings.aiPage.keys.editAgentTitle'.tr(),
      builder: (_) => _MobileAgentConfigForm(
        initialConfig: config,
        onSave: (updatedConfig) {
          context.read<AgentConfigBloc>().add(
                AgentConfigEvent.updateConfig(updatedConfig),
              );
          context.pop();
        },
      ),
    );
  }

  void _onSetDefaultAgent(BuildContext context, AgentConfig config) {
    context.read<AgentConfigBloc>().add(
          AgentConfigEvent.setDefaultConfig(config.id),
        );
  }

  void _onDeleteAgent(BuildContext context, AgentConfig config) {
    showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text('settings.aiPage.keys.deleteAgentTitle'.tr()),
        content: Text(
          'settings.aiPage.keys.deleteAgentConfirm'.tr(args: [config.name]),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: Text(LocaleKeys.button_cancel.tr()),
          ),
          TextButton(
            onPressed: () {
              context.read<AgentConfigBloc>().add(
                    AgentConfigEvent.deleteConfig(config.id),
                  );
              Navigator.of(dialogContext).pop();
            },
            child: Text(
              LocaleKeys.button_delete.tr(),
              style: TextStyle(
                color: AppFlowyTheme.of(context).textColorScheme.error,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _AgentConfigTile extends StatelessWidget {
  const _AgentConfigTile({
    required this.config,
    required this.isDefault,
    required this.onTap,
    required this.onSetDefault,
    required this.onDelete,
  });

  final AgentConfig config;
  final bool isDefault;
  final VoidCallback onTap;
  final VoidCallback onSetDefault;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return InkWell(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Expanded(
                            child: Text(
                              config.name,
                              style: theme.textStyle.heading4.standard(
                                color: theme.textColorScheme.primary,
                              ),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                          if (isDefault) ...[
                            const SizedBox(width: 8),
                            Container(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 8,
                                vertical: 2,
                              ),
                              decoration: BoxDecoration(
                                color: theme.surfaceContainerColorScheme.layer03,
                                borderRadius: BorderRadius.circular(12),
                              ),
                              child: Text(
                                'settings.aiPage.keys.defaultAgent'.tr(),
                                style: theme.textStyle.caption.standard(
                                  color: theme.textColorScheme.primary,
                                ),
                              ),
                            ),
                          ],
                        ],
                      ),
                      if (config.description.isNotEmpty) ...[
                        const SizedBox(height: 4),
                        Text(
                          config.description,
                          style: theme.textStyle.body.standard(
                            color: theme.textColorScheme.secondary,
                          ),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                PopupMenuButton<String>(
                  icon: const Icon(Icons.more_vert),
                  onSelected: (value) {
                    switch (value) {
                      case 'edit':
                        onTap();
                        break;
                      case 'setDefault':
                        onSetDefault();
                        break;
                      case 'delete':
                        onDelete();
                        break;
                    }
                  },
                  itemBuilder: (context) => [
                    PopupMenuItem(
                      value: 'edit',
                      child: Text('settings.aiPage.keys.editAgent'.tr()),
                    ),
                    if (!isDefault)
                      PopupMenuItem(
                        value: 'setDefault',
                        child: Text('settings.aiPage.keys.setAsDefault'.tr()),
                      ),
                    PopupMenuItem(
                      value: 'delete',
                      child: Text(
                        'settings.aiPage.keys.deleteAgent'.tr(),
                        style: TextStyle(
                          color: theme.textColorScheme.error,
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                _StatusChip(
                  label: config.isEnabled
                      ? 'settings.aiPage.keys.enabled'.tr()
                      : 'settings.aiPage.keys.disabled'.tr(),
                  color: config.isEnabled
                      ? theme.fillColorScheme.successLight
                      : theme.surfaceContainerColorScheme.layer02,
                ),
                const SizedBox(width: 8),
                _StatusChip(
                  label: config.languagePreference,
                  color: theme.surfaceContainerColorScheme.layer03,
                ),
                const SizedBox(width: 8),
                _StatusChip(
                  label: '${config.maxConcurrentTools} tools',
                  color: theme.surfaceContainerColorScheme.layer03,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _StatusChip extends StatelessWidget {
  const _StatusChip({
    required this.label,
    required this.color,
  });

  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        label,
        style: theme.textStyle.caption.standard(),
      ),
    );
  }
}

class _MobileAgentConfigForm extends StatefulWidget {
  const _MobileAgentConfigForm({
    this.initialConfig,
    required this.onSave,
  });

  final AgentConfig? initialConfig;
  final Function(AgentConfig) onSave;

  @override
  State<_MobileAgentConfigForm> createState() => _MobileAgentConfigFormState();
}

class _MobileAgentConfigFormState extends State<_MobileAgentConfigForm> {
  late final TextEditingController _nameController;
  late final TextEditingController _descriptionController;
  late final TextEditingController _personalityController;
  late final TextEditingController _systemPromptController;
  late String _languagePreference;
  late bool _isEnabled;
  late int _maxConcurrentTools;
  late int _toolTimeoutSeconds;

  final _formKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    final config = widget.initialConfig;
    _nameController = TextEditingController(text: config?.name ?? '');
    _descriptionController = TextEditingController(text: config?.description ?? '');
    _personalityController = TextEditingController(text: config?.personality ?? '');
    _systemPromptController = TextEditingController(text: config?.systemPrompt ?? '');
    _languagePreference = config?.languagePreference ?? 'zh-CN';
    _isEnabled = config?.isEnabled ?? true;
    _maxConcurrentTools = config?.maxConcurrentTools ?? 3;
    _toolTimeoutSeconds = config?.toolTimeoutSeconds ?? 30;
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _personalityController.dispose();
    _systemPromptController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      constraints: BoxConstraints(
        maxHeight: MediaQuery.of(context).size.height * 0.8,
      ),
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildTextField(
                controller: _nameController,
                label: 'settings.aiPage.keys.agentName'.tr(),
                hint: 'settings.aiPage.keys.agentNameHint'.tr(),
                validator: (value) {
                  if (value?.trim().isEmpty ?? true) {
                    return 'settings.aiPage.keys.agentNameRequired'.tr();
                  }
                  return null;
                },
              ),
              const SizedBox(height: 16),
              _buildTextField(
                controller: _descriptionController,
                label: 'settings.aiPage.keys.agentDescription'.tr(),
                hint: 'settings.aiPage.keys.agentDescriptionHint'.tr(),
                maxLines: 2,
              ),
              const SizedBox(height: 16),
              _buildTextField(
                controller: _personalityController,
                label: 'settings.aiPage.keys.agentPersonality'.tr(),
                hint: 'settings.aiPage.keys.agentPersonalityHint'.tr(),
                maxLines: 3,
              ),
              const SizedBox(height: 16),
              _buildTextField(
                controller: _systemPromptController,
                label: 'settings.aiPage.keys.systemPrompt'.tr(),
                hint: 'settings.aiPage.keys.systemPromptHint'.tr(),
                maxLines: 4,
              ),
              const SizedBox(height: 16),
              _buildLanguageSelector(),
              const SizedBox(height: 16),
              _buildSwitchTile(
                title: 'settings.aiPage.keys.enableAgent'.tr(),
                value: _isEnabled,
                onChanged: (value) => setState(() => _isEnabled = value),
              ),
              const SizedBox(height: 16),
              _buildSliderTile(
                title: 'settings.aiPage.keys.maxConcurrentTools'.tr(),
                value: _maxConcurrentTools.toDouble(),
                min: 1,
                max: 10,
                divisions: 9,
                onChanged: (value) => setState(() => _maxConcurrentTools = value.round()),
              ),
              const SizedBox(height: 16),
              _buildSliderTile(
                title: 'settings.aiPage.keys.toolTimeout'.tr(),
                value: _toolTimeoutSeconds.toDouble(),
                min: 5,
                max: 300,
                divisions: 59,
                onChanged: (value) => setState(() => _toolTimeoutSeconds = value.round()),
              ),
              const SizedBox(height: 24),
              Row(
                children: [
                  Expanded(
                    child: OutlinedRoundedButton(
                      text: LocaleKeys.button_cancel.tr(),
                      onTap: () => Navigator.of(context).pop(),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: AFFilledTextButton.primary(
                      text: LocaleKeys.button_save.tr(),
                      onTap: _onSave,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    required String label,
    String? hint,
    int maxLines = 1,
    String? Function(String?)? validator,
  }) {
    final theme = AppFlowyTheme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: theme.textStyle.body.standard(
            color: theme.textColorScheme.primary,
          ),
        ),
        const SizedBox(height: 8),
        TextFormField(
          controller: controller,
          maxLines: maxLines,
          validator: validator,
          decoration: InputDecoration(
            hintText: hint,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
            ),
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 12,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildLanguageSelector() {
    final theme = AppFlowyTheme.of(context);
    final languages = [
      {'code': 'zh-CN', 'name': '中文'},
      {'code': 'en-US', 'name': 'English'},
      {'code': 'ja-JP', 'name': '日本語'},
      {'code': 'ko-KR', 'name': '한국어'},
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'settings.aiPage.keys.languagePreference'.tr(),
          style: theme.textStyle.body.standard(
            color: theme.textColorScheme.primary,
          ),
        ),
        const SizedBox(height: 8),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(
            border: Border.all(color: theme.borderColorScheme.primary),
            borderRadius: BorderRadius.circular(8),
          ),
          child: DropdownButtonHideUnderline(
            child: DropdownButton<String>(
              value: _languagePreference,
              onChanged: (value) {
                if (value != null) {
                  setState(() => _languagePreference = value);
                }
              },
              items: languages.map((lang) {
                return DropdownMenuItem<String>(
                  value: lang['code'],
                  child: Text(lang['name']!),
                );
              }).toList(),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildSwitchTile({
    required String title,
    required bool value,
    required ValueChanged<bool> onChanged,
  }) {
    final theme = AppFlowyTheme.of(context);
    return Row(
      children: [
        Expanded(
          child: Text(
            title,
            style: theme.textStyle.body.standard(
              color: theme.textColorScheme.primary,
            ),
          ),
        ),
        Switch(
          value: value,
          onChanged: onChanged,
        ),
      ],
    );
  }

  Widget _buildSliderTile({
    required String title,
    required double value,
    required double min,
    required double max,
    required int divisions,
    required ValueChanged<double> onChanged,
  }) {
    final theme = AppFlowyTheme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                title,
                style: theme.textStyle.body.standard(
                  color: theme.textColorScheme.primary,
                ),
              ),
            ),
            Text(
              value.round().toString(),
              style: theme.textStyle.body.standard(
                color: theme.textColorScheme.secondary,
              ),
            ),
          ],
        ),
        Slider(
          value: value,
          min: min,
          max: max,
          divisions: divisions,
          onChanged: onChanged,
        ),
      ],
    );
  }

  void _onSave() {
    if (!_formKey.currentState!.validate()) {
      return;
    }

    final config = AgentConfig(
      id: widget.initialConfig?.id ?? 
          'agent_${DateTime.now().millisecondsSinceEpoch}',
      name: _nameController.text.trim(),
      description: _descriptionController.text.trim(),
      personality: _personalityController.text.trim(),
      systemPrompt: _systemPromptController.text.trim(),
      languagePreference: _languagePreference,
      createdAt: widget.initialConfig?.createdAt ?? DateTime.now(),
      updatedAt: DateTime.now(),
      isEnabled: _isEnabled,
      maxConcurrentTools: _maxConcurrentTools,
      toolTimeoutSeconds: _toolTimeoutSeconds,
    );

    widget.onSave(config);
  }
}
