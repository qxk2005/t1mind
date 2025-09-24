import 'dart:convert';

import 'package:appflowy/core/config/kv.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
// spacing unused after we switched to SizedBox
import 'package:flowy_infra_ui/widget/separated_flex.dart';
import 'package:flowy_infra_ui/style_widget/primary_rounded_button.dart';
import 'package:flowy_infra_ui/style_widget/text_field.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart' show FlowyDialog;
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';
import 'package:appflowy/plugins/mcp/settings/mcp_check_dialog.dart';

class SettingsMCPView extends StatefulWidget {
  const SettingsMCPView({
    super.key,
    required this.workspaceId,
    required this.userProfile,
  });

  final String workspaceId;
  final UserProfilePB userProfile;

  @override
  State<SettingsMCPView> createState() => _SettingsMCPViewState();
}

class _SettingsMCPViewState extends State<SettingsMCPView> {
  static const String _prefsKey = 'settings.mcp.endpoints';
  final DartKeyValue _kv = DartKeyValue();

  List<_McpEndpoint> _items = const [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final str = await _kv.get(_prefsKey);
    if (str == null) {
      setState(() => _items = const []);
      return;
    }
    try {
      final list = (jsonDecode(str) as List<dynamic>)
          .map((e) => _McpEndpoint.fromJson(e as Map<String, dynamic>))
          .toList();
      setState(() => _items = list);
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Failed to decode MCP endpoints: $e');
      }
      setState(() => _items = const []);
    }
  }

  Future<void> _save() async {
    final str = jsonEncode(_items.map((e) => e.toJson()).toList());
    await _kv.set(_prefsKey, str);
  }

  void _onCreate() async {
    final result = await showDialog<_McpEndpoint>(
      context: context,
      builder: (context) => _EditMcpEndpointDialog(),
    );
    if (result != null) {
      setState(() => _items = [..._items, result]);
      await _save();
    }
  }

  void _onEdit(int index) async {
    final result = await showDialog<_McpEndpoint>(
      context: context,
      builder: (context) => _EditMcpEndpointDialog(initial: _items[index]),
    );
    if (result != null) {
      setState(() {
        _items = [..._items]..[index] = result;
      });
      await _save();
    }
  }

  void _onDelete(int index) async {
    setState(() {
      _items = [..._items]..removeAt(index);
    });
    await _save();
  }

  void _onCheck(int index) async {
    final ep = _items[index];
    final cfg = McpEndpointConfig(
      name: ep.name,
      transport: ep.transport == _McpTransport.sse
          ? McpTransport.sse
          : McpTransport.stdio,
      url: ep.url,
      command: ep.command,
      args: ep.args,
      env: ep.env,
      headers: ep.headers,
    );
    final result = await showDialog<McpCheckResult>(
      context: context,
      builder: (context) => McpCheckDialog(config: cfg),
    );
    if (result != null) {
      setState(() {
        _items = [..._items]
          ..[index] = ep.copyWith(
            checkedOk: result.ok,
            toolsCount: result.toolCount,
            lastCheckedAt: result.checkedAtIso,
          );
      });
      await _save();
    }
  }

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: LocaleKeys.settings_mcpPage_title.tr(),
      description: LocaleKeys.settings_mcpPage_description.tr(),
      children: [
        Row(
          children: [
            AFFilledTextButton.primary(
              text: LocaleKeys.button_create.tr(),
              onTap: _onCreate,
            ),
          ],
        ),
        const SizedBox(height: 8),
        _McpEndpointList(
          items: _items,
          onEdit: _onEdit,
          onDelete: _onDelete,
          onCheck: _onCheck,
        ),
      ],
    );
  }
}

class _McpEndpointList extends StatelessWidget {
  const _McpEndpointList({
    required this.items,
    required this.onEdit,
    required this.onDelete,
    required this.onCheck,
  });

  final List<_McpEndpoint> items;
  final void Function(int index) onEdit;
  final void Function(int index) onDelete;
  final void Function(int index) onCheck;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    if (items.isEmpty) {
      return Container(
        padding: EdgeInsets.all(theme.spacing.l),
        decoration: BoxDecoration(
          color: theme.surfaceContainerColorScheme.layer02,
          borderRadius: BorderRadius.circular(8),
        ),
        child: Text(
          LocaleKeys.settings_mcpPage_emptyHint.tr(),
          style: theme.textStyle.body
              .standard(color: theme.textColorScheme.secondary),
        ),
      );
    }
    return SeparatedColumn(
      separatorBuilder: () => SizedBox(height: theme.spacing.m),
      children: [
        for (int i = 0; i < items.length; i++)
          _McpEndpointItem(
            endpoint: items[i],
            onEdit: () => onEdit(i),
            onDelete: () => onDelete(i),
            onCheck: () => onCheck(i),
          ),
      ],
    );
  }
}

class _McpEndpointItem extends StatelessWidget {
  const _McpEndpointItem({
    required this.endpoint,
    required this.onEdit,
    required this.onDelete,
    required this.onCheck,
  });

  final _McpEndpoint endpoint;
  final VoidCallback onEdit;
  final VoidCallback onDelete;
  final VoidCallback onCheck;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      padding: EdgeInsets.all(theme.spacing.l),
      decoration: BoxDecoration(
        color: theme.surfaceContainerColorScheme.layer01,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: theme.borderColorScheme.primary),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  endpoint.name,
                  style: theme.textStyle.heading4.standard(
                    color: theme.textColorScheme.primary,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              OutlinedRoundedButton(
                text: 'Check',
                onTap: onCheck,
              ),
              const SizedBox(width: 8),
              OutlinedRoundedButton(
                text: LocaleKeys.button_edit.tr(),
                onTap: onEdit,
              ),
              const SizedBox(width: 8),
              AFGhostTextButton.primary(
                text: LocaleKeys.button_delete.tr(),
                onTap: onDelete,
              ),
            ],
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              _Badge(text: endpoint.transport.name.toUpperCase()),
              if (endpoint.toolsCount != null)
                _Badge(text: 'TOOLS: ${endpoint.toolsCount}')
              else
                const SizedBox.shrink(),
              if (endpoint.disabledOnAndroid)
                _Badge(text: LocaleKeys.settings_mcpPage_androidDisabled.tr()),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            endpoint.description ?? '',
            style: theme.textStyle.body.standard(
              color: theme.textColorScheme.secondary,
            ),
          ),
        ],
      ),
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

class _EditMcpEndpointDialog extends StatefulWidget {
  const _EditMcpEndpointDialog({this.initial});

  final _McpEndpoint? initial;

  @override
  State<_EditMcpEndpointDialog> createState() => _EditMcpEndpointDialogState();
}

class _EditMcpEndpointDialogState extends State<_EditMcpEndpointDialog> {
  late final TextEditingController _name;
  late final TextEditingController _url;
  late final TextEditingController _command;
  late final TextEditingController _args;
  late _McpTransport _transport;
  bool _disabledOnAndroid = true;
  late final TextEditingController _description;
  late List<_KVRow> _envRows;
  late List<_KVRow> _headerRows;
  bool? _checkedOk;
  int? _toolsCount;
  String? _lastCheckedAt;

  @override
  void initState() {
    super.initState();
    _name = TextEditingController(text: widget.initial?.name ?? '');
    _url = TextEditingController(text: widget.initial?.url ?? '');
    _command = TextEditingController(text: widget.initial?.command ?? '');
    _args = TextEditingController(text: widget.initial?.args?.join(' ') ?? '');
    _transport = widget.initial?.transport ?? _McpTransport.stdio;
    _disabledOnAndroid = widget.initial?.disabledOnAndroid ?? true;
    _description = TextEditingController(text: widget.initial?.description ?? '');

    _envRows = _rowsFromMap(widget.initial?.env);
    _headerRows = _rowsFromMap(widget.initial?.headers);
    _checkedOk = widget.initial?.checkedOk;
    _toolsCount = widget.initial?.toolsCount;
    _lastCheckedAt = widget.initial?.lastCheckedAt;
  }

  List<_KVRow> _rowsFromMap(Map<String, String>? map) {
    if (map == null || map.isEmpty) {
      return [
        _KVRow(keyController: TextEditingController(), valueController: TextEditingController()),
      ];
    }
    return map.entries
        .map((e) => _KVRow(
              keyController: TextEditingController(text: e.key),
              valueController: TextEditingController(text: e.value),
            ))
        .toList();
  }

  Map<String, String>? _mapFromRows(List<_KVRow> rows) {
    final entries = <MapEntry<String, String>>[];
    for (final r in rows) {
      final k = r.keyController.text.trim();
      final v = r.valueController.text.trim();
      if (k.isNotEmpty) {
        entries.add(MapEntry(k, v));
      }
    }
    if (entries.isEmpty) return null;
    return Map<String, String>.fromEntries(entries);
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxHeight: MediaQuery.of(context).size.height * 0.8,
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
                  ? LocaleKeys.settings_mcpPage_newTitle.tr()
                  : LocaleKeys.settings_mcpPage_editTitle.tr(),
              style: AppFlowyTheme.of(context)
                  .textStyle
                  .heading4
                  .standard(color: AppFlowyTheme.of(context).textColorScheme.primary),
            ),
            const SizedBox(height: 16),
            SeparatedColumn(
              separatorBuilder: () => const SizedBox(height: 12),
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _LabeledField(
                  label: LocaleKeys.settings_mcpPage_field_name.tr(),
                  child: FlowyTextField(
                    controller: _name,
                    autoFocus: true,
                  ),
                ),
                _LabeledField(
                  label: LocaleKeys.settings_mcpPage_field_transport.tr(),
                  child: DropdownButton<_McpTransport>(
                    value: _transport,
                    items: _McpTransport.values
                        .map((e) => DropdownMenuItem<_McpTransport>(
                              value: e,
                              child: Text(e.name.toUpperCase()),
                            ))
                        .toList(),
                    onChanged: (v) => setState(() => _transport = v!),
                  ),
                ),
                if (_transport == _McpTransport.sse)
                  _LabeledField(
                    label: LocaleKeys.settings_mcpPage_field_url.tr(),
                    child: FlowyTextField(
                      controller: _url,
                      hintText: 'https://example.com/sse',
                    ),
                  ),
                if (_transport == _McpTransport.stdio) ...[
                  _LabeledField(
                    label: LocaleKeys.settings_mcpPage_field_command.tr(),
                    child: FlowyTextField(
                      controller: _command,
                      hintText: 'node',
                    ),
                  ),
                  _LabeledField(
                    label: LocaleKeys.settings_mcpPage_field_args.tr(),
                    child: FlowyTextField(
                      controller: _args,
                      hintText: 'server.js --port 8080',
                    ),
                  ),
                ],
                if (_transport == _McpTransport.stdio)
                  _KVEditor(
                    title: LocaleKeys.settings_mcpPage_field_env.tr(),
                    keyPlaceholder: LocaleKeys.settings_mcpPage_field_key.tr(),
                    valuePlaceholder: LocaleKeys.settings_mcpPage_field_value.tr(),
                    rows: _envRows,
                    onAdd: () => setState(() => _envRows.add(_KVRow.empty())),
                    onRemove: (i) => setState(() => _envRows.removeAt(i)),
                  ),
                if (_transport == _McpTransport.sse)
                  _KVEditor(
                    title: LocaleKeys.settings_mcpPage_field_headers.tr(),
                    keyPlaceholder: LocaleKeys.settings_mcpPage_field_key.tr(),
                    valuePlaceholder: LocaleKeys.settings_mcpPage_field_value.tr(),
                    rows: _headerRows,
                    onAdd: () => setState(() => _headerRows.add(_KVRow.empty())),
                    onRemove: (i) => setState(() => _headerRows.removeAt(i)),
                  ),
                if (defaultTargetPlatform == TargetPlatform.android)
                  _LabeledField(
                    label: LocaleKeys.settings_mcpPage_androidDisabled.tr(),
                    child: Switch(
                      value: _disabledOnAndroid,
                      onChanged: (v) => setState(() => _disabledOnAndroid = v),
                    ),
                  ),
                _LabeledField(
                  label: LocaleKeys.settings_mcpPage_field_description.tr(),
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
                        borderSide: BorderSide(color: AppFlowyTheme.of(context).borderColorScheme.primary),
                      ),
                      focusedBorder: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                        borderSide: BorderSide(color: AppFlowyTheme.of(context).textColorScheme.primary),
                      ),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                OutlinedRoundedButton(
                  text: LocaleKeys.button_cancel.tr(),
                  onTap: () => Navigator.of(context).pop(),
                ),
                const SizedBox(width: 8),
                OutlinedRoundedButton(
                  text: '检查',
                  onTap: () async {
                    final args = _args.text
                        .split(' ')
                        .map((e) => e.trim())
                        .where((e) => e.isNotEmpty)
                        .toList();
                    final cfg = McpEndpointConfig(
                      name: _name.text.trim(),
                      transport: _transport == _McpTransport.sse
                          ? McpTransport.sse
                          : McpTransport.stdio,
                      url: _url.text.trim().isEmpty ? null : _url.text.trim(),
                      command: _command.text.trim().isEmpty
                          ? null
                          : _command.text.trim(),
                      args: args.isEmpty ? null : args,
                      env: _transport == _McpTransport.stdio
                          ? _mapFromRows(_envRows)
                          : null,
                      headers: _transport == _McpTransport.sse
                          ? _mapFromRows(_headerRows)
                          : null,
                    );
                    final result = await showDialog<McpCheckResult>(
                      context: context,
                      builder: (context) => McpCheckDialog(config: cfg),
                    );
                    if (result != null && mounted) {
                      setState(() {
                        _checkedOk = result.ok;
                        _toolsCount = result.toolCount;
                        _lastCheckedAt = result.checkedAtIso;
                      });
                    }
                  },
                ),
                const SizedBox(width: 8),
                AFFilledTextButton.primary(
                  text: LocaleKeys.button_save.tr(),
                  onTap: () {
                    final args = _args.text
                        .split(' ')
                        .map((e) => e.trim())
                        .where((e) => e.isNotEmpty)
                        .toList();
                    final endpoint = _McpEndpoint(
                      name: _name.text.trim(),
                      transport: _transport,
                      url: _url.text.trim().isEmpty ? null : _url.text.trim(),
                      command: _command.text.trim().isEmpty
                          ? null
                          : _command.text.trim(),
                      args: args.isEmpty ? null : args,
                      disabledOnAndroid: _disabledOnAndroid,
                      description: _description.text.trim().isEmpty
                          ? null
                          : _description.text.trim(),
                      env: _transport == _McpTransport.stdio
                          ? _mapFromRows(_envRows)
                          : null,
                      headers: _transport == _McpTransport.sse
                          ? _mapFromRows(_headerRows)
                          : null,
                      checkedOk: _checkedOk,
                      toolsCount: _toolsCount,
                      lastCheckedAt: _lastCheckedAt,
                    );
                    Navigator.of(context).pop(endpoint);
                  },
                ),
              ],
            ),
            const SizedBox(height: 8),
            if (_toolsCount != null)
              Text(
                '工具: ${_toolsCount} · ${_checkedOk == true ? 'OK' : '未通过'}' +
                    (_lastCheckedAt != null ? ' · ${_lastCheckedAt}' : ''),
                style: AppFlowyTheme.of(context)
                    .textStyle
                    .caption
                    .standard(color: AppFlowyTheme.of(context).textColorScheme.secondary),
              ),
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
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: theme.textStyle.caption
              .standard(color: theme.textColorScheme.secondary),
        ),
        const SizedBox(height: 6),
        if (height != null) SizedBox(height: height, child: child) else SizedBox(height: 36, child: child),
      ],
    );
  }
}

class _KVRow {
  _KVRow({required this.keyController, required this.valueController});

  final TextEditingController keyController;
  final TextEditingController valueController;

  factory _KVRow.empty() => _KVRow(
        keyController: TextEditingController(),
        valueController: TextEditingController(),
      );
}

class _KVEditor extends StatelessWidget {
  const _KVEditor({
    required this.title,
    required this.keyPlaceholder,
    required this.valuePlaceholder,
    required this.rows,
    required this.onAdd,
    required this.onRemove,
  });

  final String title;
  final String keyPlaceholder;
  final String valuePlaceholder;
  final List<_KVRow> rows;
  final VoidCallback onAdd;
  final void Function(int index) onRemove;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: theme.textStyle.caption
              .standard(color: theme.textColorScheme.secondary),
        ),
        const SizedBox(height: 6),
        SeparatedColumn(
          separatorBuilder: () => const SizedBox(height: 8),
          children: [
            for (int i = 0; i < rows.length; i++)
              Row(
                children: [
                  Expanded(
                    child: FlowyTextField(
                      controller: rows[i].keyController,
                      hintText: keyPlaceholder,
                      autoFocus: false,
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: FlowyTextField(
                      controller: rows[i].valueController,
                      hintText: valuePlaceholder,
                      autoFocus: false,
                    ),
                  ),
                  const SizedBox(width: 8),
                  AFGhostTextButton.primary(
                    text: '—',
                    onTap: () => onRemove(i),
                  ),
                ],
              ),
            Row(
              children: [
                AFGhostTextButton.primary(
                  text: '+',
                  onTap: onAdd,
                ),
              ],
            ),
          ],
        ),
      ],
    );
  }
}

enum _McpTransport { stdio, sse }

class _McpEndpoint {
  _McpEndpoint({
    required this.name,
    required this.transport,
    this.url,
    this.command,
    this.args,
    this.disabledOnAndroid = true,
    this.description,
    this.env,
    this.headers,
    this.checkedOk,
    this.toolsCount,
    this.lastCheckedAt,
  });

  final String name;
  final _McpTransport transport;
  final String? url;
  final String? command;
  final List<String>? args;
  final bool disabledOnAndroid;
  final String? description;
  final Map<String, String>? env;
  final Map<String, String>? headers;
  final bool? checkedOk;
  final int? toolsCount;
  final String? lastCheckedAt;

  Map<String, dynamic> toJson() => {
        'name': name,
        'transport': transport.name,
        'url': url,
        'command': command,
        'args': args,
        'disabledOnAndroid': disabledOnAndroid,
        'description': description,
        'env': env,
        'headers': headers,
        if (checkedOk != null) 'checkedOk': checkedOk,
        if (toolsCount != null) 'toolsCount': toolsCount,
        if (lastCheckedAt != null) 'lastCheckedAt': lastCheckedAt,
      };

  static _McpEndpoint fromJson(Map<String, dynamic> json) => _McpEndpoint(
        name: json['name'] as String,
        transport: _McpTransport.values
            .byName((json['transport'] as String?) ?? 'stdio'),
        url: json['url'] as String?,
        command: json['command'] as String?,
        args: (json['args'] as List?)?.map((e) => e.toString()).toList(),
        disabledOnAndroid: (json['disabledOnAndroid'] as bool?) ?? true,
        description: json['description'] as String?,
        env: (json['env'] as Map?)?.map((k, v) => MapEntry(k.toString(), v.toString())),
        headers: (json['headers'] as Map?)?.map((k, v) => MapEntry(k.toString(), v.toString())),
        checkedOk: json['checkedOk'] as bool?,
        toolsCount: (json['toolsCount'] as num?)?.toInt(),
        lastCheckedAt: json['lastCheckedAt'] as String?,
      );

  _McpEndpoint copyWith({
    bool? checkedOk,
    int? toolsCount,
    String? lastCheckedAt,
  }) {
    return _McpEndpoint(
      name: name,
      transport: transport,
      url: url,
      command: command,
      args: args,
      disabledOnAndroid: disabledOnAndroid,
      description: description,
      env: env,
      headers: headers,
      checkedOk: checkedOk ?? this.checkedOk,
      toolsCount: toolsCount ?? this.toolsCount,
      lastCheckedAt: lastCheckedAt ?? this.lastCheckedAt,
    );
  }
}


