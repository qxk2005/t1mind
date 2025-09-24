import 'dart:convert';

import 'package:appflowy/core/config/kv.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart' show FlowyDialog;
import 'package:flowy_infra_ui/style_widget/primary_rounded_button.dart';
import 'package:flowy_infra_ui/style_widget/text_field.dart';
import 'package:flowy_infra_ui/widget/separated_flex.dart';
import 'package:flutter/material.dart';

class AgentSettingsMobilePage extends StatefulWidget {
  const AgentSettingsMobilePage({super.key});

  static const routeName = '/settings/agents';

  @override
  State<AgentSettingsMobilePage> createState() => _AgentSettingsMobilePageState();
}

class _AgentSettingsMobilePageState extends State<AgentSettingsMobilePage> {
  static const String _agentsPrefsKey = 'settings.agents.items';
  static const String _mcpPrefsKey = 'settings.mcp.endpoints';
  final DartKeyValue _kv = DartKeyValue();

  List<_AgentItem> _agents = const [];
  List<String> _availableEndpointNames = const [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final str = await _kv.get(_agentsPrefsKey);
      if (str?.isNotEmpty == true) {
        _agents = (jsonDecode(str!) as List<dynamic>)
            .map((e) => _AgentItem.fromJson(e as Map<String, dynamic>))
            .toList();
      } else {
        _agents = const [];
      }
    } catch (_) {
      _agents = const [];
    }

    try {
      final str = await _kv.get(_mcpPrefsKey);
      final List<String> eps;
      if (str?.isNotEmpty == true) {
        final raw = jsonDecode(str!) as List<dynamic>;
        eps = raw
            .where((e) => ((e as Map<String, dynamic>)['checkedOk'] as bool?) == true)
            .map((e) => (e as Map<String, dynamic>)['name'] as String)
            .toList();
      } else {
        eps = const [];
      }
      setState(() {
        _availableEndpointNames = eps;
        _agents = [..._agents];
      });
    } catch (_) {
      setState(() {
        _availableEndpointNames = const [];
        _agents = [..._agents];
      });
    }
  }

  Future<void> _save() async {
    final str = jsonEncode(_agents.map((e) => e.toJson()).toList());
    await _kv.set(_agentsPrefsKey, str);
  }

  Future<void> _onCreate() async {
    final result = await showDialog<_AgentItem>(
      context: context,
      builder: (context) => _EditAgentDialog(
        availableEndpointNames: _availableEndpointNames,
      ),
    );
    if (result != null) {
      setState(() => _agents = [..._agents, result]);
      await _save();
    }
  }

  Future<void> _onEdit(int index) async {
    final result = await showDialog<_AgentItem>(
      context: context,
      builder: (context) => _EditAgentDialog(
        initial: _agents[index],
        availableEndpointNames: _availableEndpointNames,
      ),
    );
    if (result != null) {
      setState(() => _agents = [..._agents]..[index] = result);
      await _save();
    }
  }

  Future<void> _onDelete(int index) async {
    setState(() => _agents = [..._agents]..removeAt(index));
    await _save();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('settings.agentsPage.title'.tr()),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: LocaleKeys.button_create.tr(),
            onPressed: _onCreate,
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(12),
        children: [
          MobileSettingGroup(
            groupTitle: 'settings.agentsPage.title'.tr(),
            showDivider: false,
            settingItemList: [
              if (_agents.isEmpty)
                Padding(
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  child: Text(
                    'settings.agentsPage.emptyHint'.tr(),
                    style: AppFlowyTheme.of(context)
                        .textStyle
                        .body
                        .standard(
                          color: AppFlowyTheme.of(context)
                              .textColorScheme
                              .secondary,
                        ),
                  ),
                )
              else ...[
                for (int i = 0; i < _agents.length; i++)
                  MobileSettingItem(
                    title: _AgentTile(
                      item: _agents[i],
                      onEdit: () => _onEdit(i),
                      onDelete: () => _onDelete(i),
                    ),
                  ),
              ],
            ],
          ),
        ],
      ),
    );
  }
}

class _AgentTile extends StatelessWidget {
  const _AgentTile({
    required this.item,
    required this.onEdit,
    required this.onDelete,
  });

  final _AgentItem item;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                item.name,
                style: theme.textStyle.heading4
                    .standard(color: theme.textColorScheme.primary),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            const SizedBox(width: 8),
            OutlinedRoundedButton(text: LocaleKeys.button_edit.tr(), onTap: onEdit),
            const SizedBox(width: 8),
            AFGhostTextButton.primary(
              text: LocaleKeys.button_delete.tr(),
              onTap: onDelete,
            ),
          ],
        ),
        if ((item.description ?? '').isNotEmpty) ...[
          const SizedBox(height: 8),
          Text(
            item.description!,
            style: theme.textStyle.body
                .standard(color: theme.textColorScheme.secondary),
          ),
        ],
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final name in item.allowedEndpointNames) _Badge(text: name),
          ],
        ),
      ],
    );
  }
}

class _Badge extends StatelessWidget {
  const _Badge({required this.text});
  final String text;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: theme.spacing.m,
        vertical: theme.spacing.xs,
      ),
      decoration: BoxDecoration(
        color: theme.surfaceContainerColorScheme.layer03,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(text, style: theme.textStyle.caption.standard()),
    );
  }
}

class _EditAgentDialog extends StatefulWidget {
  const _EditAgentDialog({this.initial, required this.availableEndpointNames});

  final _AgentItem? initial;
  final List<String> availableEndpointNames;

  @override
  State<_EditAgentDialog> createState() => _EditAgentDialogState();
}

class _EditAgentDialogState extends State<_EditAgentDialog> {
  late final TextEditingController _name;
  late final TextEditingController _description;
  late final Set<String> _selectedEndpointNames;

  @override
  void initState() {
    super.initState();
    _name = TextEditingController(text: widget.initial?.name ?? '');
    _description =
        TextEditingController(text: widget.initial?.description ?? '');
    _selectedEndpointNames =
        {...(widget.initial?.allowedEndpointNames ?? const [])};
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return FlowyDialog(
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxHeight: MediaQuery.of(context).size.height * 0.9,
        ),
        child: SingleChildScrollView(
          physics: const ClampingScrollPhysics(),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  widget.initial == null
                      ? 'settings.agentsPage.newTitle'.tr()
                      : 'settings.agentsPage.editTitle'.tr(),
                  style: theme.textStyle.heading4
                      .standard(color: theme.textColorScheme.primary),
                ),
                const SizedBox(height: 16),
                SeparatedColumn(
                  separatorBuilder: () => const SizedBox(height: 12),
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _LabeledField(
                      label: 'settings.agentsPage.field.name'.tr(),
                      child: FlowyTextField(
                        controller: _name,
                        autoFocus: true,
                      ),
                    ),
                    _LabeledField(
                      label: 'settings.agentsPage.field.description'.tr(),
                      height: 120,
                      child: TextField(
                        controller: _description,
                        minLines: 3,
                        maxLines: 10,
                        keyboardType: TextInputType.multiline,
                        scrollPhysics: const ClampingScrollPhysics(),
                        decoration: InputDecoration(
                          isDense: false,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                          enabledBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(8),
                            borderSide: BorderSide(
                              color: AppFlowyTheme.of(context).borderColorScheme.primary,
                            ),
                          ),
                          focusedBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(8),
                            borderSide: BorderSide(
                              color: AppFlowyTheme.of(context).textColorScheme.primary,
                            ),
                          ),
                        ),
                      ),
                    ),
                    _LabeledField(
                      label:
                          'settings.agentsPage.field.allowedEndpoints'.tr(),
                      child: _EndpointMultiSelect(
                        endpoints: widget.availableEndpointNames,
                        selected: _selectedEndpointNames,
                        onToggle: (name, v) {
                          setState(() {
                            if (v) {
                              _selectedEndpointNames.add(name);
                            } else {
                              _selectedEndpointNames.remove(name);
                            }
                          });
                        },
                      ),
                    ),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        OutlinedRoundedButton(
                          text: LocaleKeys.button_cancel.tr(),
                          onTap: () => Navigator.of(context).pop(),
                        ),
                        const SizedBox(width: 8),
                        AFFilledTextButton.primary(
                          text: LocaleKeys.button_save.tr(),
                          onTap: () {
                            final item = _AgentItem(
                              name: _name.text.trim(),
                              description: _description.text.trim().isEmpty
                                  ? null
                                  : _description.text.trim(),
                              allowedEndpointNames:
                                  _selectedEndpointNames.toList(),
                            );
                            Navigator.of(context).pop(item);
                          },
                        ),
                      ],
                    )
                  ],
                )
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _LabeledField extends StatelessWidget {
  const _LabeledField({required this.label, required this.child, this.height});
  final String label;
  final Widget child;
  final double? height;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return SeparatedColumn(
      separatorBuilder: () => const SizedBox(height: 6),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: theme.textStyle.body
              .standard(color: theme.textColorScheme.primary),
        ),
        if (height != null) SizedBox(height: height, child: child) else child,
      ],
    );
  }
}

class _EndpointMultiSelect extends StatelessWidget {
  const _EndpointMultiSelect({
    required this.endpoints,
    required this.selected,
    required this.onToggle,
  });

  final List<String> endpoints;
  final Set<String> selected;
  final void Function(String name, bool selected) onToggle;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    if (endpoints.isEmpty) {
      return Container(
        padding: EdgeInsets.all(theme.spacing.m),
        decoration: BoxDecoration(
          color: theme.surfaceContainerColorScheme.layer02,
          borderRadius: BorderRadius.circular(8),
        ),
        child: Text(
          'settings.mcpPage.emptyHint'.tr(),
          style: theme.textStyle.body
              .standard(color: theme.textColorScheme.secondary),
        ),
      );
    }
    return SeparatedColumn(
      separatorBuilder: () => const SizedBox(height: 8),
      children: [
        for (final e in endpoints)
          Row(
            children: [
              Expanded(
                child: Text(
                  e,
                  style: theme.textStyle.body
                      .standard(color: theme.textColorScheme.primary),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              Checkbox(
                value: selected.contains(e),
                onChanged: (v) => onToggle(e, v == true),
              )
            ],
          )
      ],
    );
  }
}

class _AgentItem {
  _AgentItem({
    required this.name,
    this.description,
    required this.allowedEndpointNames,
  });

  final String name;
  final String? description;
  final List<String> allowedEndpointNames;

  Map<String, dynamic> toJson() => {
        'name': name,
        if (description != null) 'description': description,
        'allowedEndpointNames': allowedEndpointNames,
      };

  static _AgentItem fromJson(Map<String, dynamic> json) => _AgentItem(
        name: json['name'] as String,
        description: json['description'] as String?,
        allowedEndpointNames: (json['allowedEndpointNames'] as List?)
                ?.map((e) => e.toString())
                .toList() ??
            const [],
      );
}


