import 'dart:convert';

import 'package:appflowy/core/config/kv.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:universal_platform/universal_platform.dart';

/// A lightweight MCP selector that loads available endpoints from SharedPreferences
/// and allows the user to multi-select endpoints to be used in the next chat send.
class McpSelector extends StatefulWidget {
  const McpSelector({
    super.key,
    required this.onChanged,
    this.initialSelectedNames = const [],
    this.iconSize = 20.0,
  });

  final ValueChanged<List<String>> onChanged;
  final List<String> initialSelectedNames;
  final double iconSize;

  @override
  State<McpSelector> createState() => _McpSelectorState();
}

class _McpSelectorState extends State<McpSelector> {
  static const String _mcpPrefsKey = 'settings.mcp.endpoints';
  final DartKeyValue _kv = DartKeyValue();

  List<_McpEndpointLite> _available = const [];
  late List<String> _selected;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _selected = [...widget.initialSelectedNames];
    _load();
  }

  Future<void> _load() async {
    try {
      final str = await _kv.get(_mcpPrefsKey);
      final List<_McpEndpointLite> eps;
      if (str?.isNotEmpty == true) {
        final raw = jsonDecode(str!) as List<dynamic>;
        eps = raw
            .map((e) => _McpEndpointLite.fromJson(e as Map<String, dynamic>))
            .where((e) => (e.checkedOk ?? false) == true)
            .toList();
      } else {
        eps = const [];
      }
      if (mounted) {
        setState(() {
          _available = eps;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Failed to load MCP endpoints: $e');
      }
      if (mounted) {
        setState(() {
          _available = const [];
          _isLoading = false;
        });
      }
    }
  }

  void _toggle(String name, bool checked) {
    setState(() {
      if (checked) {
        if (!_selected.contains(name)) _selected = [..._selected, name];
      } else {
        _selected = _selected.where((e) => e != name).toList();
      }
    });
    widget.onChanged(_selected);
  }

  @override
  Widget build(BuildContext context) {
    final icon = Icon(
      Icons.extension,
      size: widget.iconSize,
      color: Theme.of(context).colorScheme.onSurfaceVariant,
    );

    if (_isLoading) {
      return SizedBox(
        width: widget.iconSize + 8,
        height: widget.iconSize + 8,
        child: Center(
          child: SizedBox(
            width: widget.iconSize - 6,
            height: widget.iconSize - 6,
            child: const CircularProgressIndicator(strokeWidth: 2.0),
          ),
        ),
      );
    }

    if (_available.isEmpty) {
      // If no available endpoints, show a disabled icon to keep layout stable
      return Opacity(opacity: 0.5, child: icon);
    }

    return _McpPopover(
      icon: icon,
      contentBuilder: (ctx) => _McpList(
        items: _available,
        selected: _selected,
        onToggle: _toggle,
      ),
    );
  }
}

class _McpEndpointLite {
  _McpEndpointLite({
    required this.name,
    this.checkedOk,
  });

  final String name;
  final bool? checkedOk;

  static _McpEndpointLite fromJson(Map<String, dynamic> json) =>
      _McpEndpointLite(
        name: json['name'] as String,
        checkedOk: json['checkedOk'] as bool?,
      );
}

class _McpList extends StatelessWidget {
  const _McpList({
    required this.items,
    required this.selected,
    required this.onToggle,
  });

  final List<_McpEndpointLite> items;
  final List<String> selected;
  final void Function(String, bool) onToggle;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    final list = items
        .map(
          (e) => Row(
            mainAxisSize: MainAxisSize.max,
            children: [
              Expanded(
                child: Text(
                  e.name,
                  style: theme.textStyle.body
                      .standard(color: theme.textColorScheme.primary),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              Checkbox(
                value: selected.contains(e.name),
                onChanged: (v) => onToggle(e.name, v == true),
              )
            ],
          ),
        )
        .toList();

    final body = ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 320, maxHeight: 360),
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(8.0),
        child: SeparatedColumn(
          separatorBuilder: () => const VSpace(4.0),
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: list,
        ),
      ),
    );

    return Material(
      color: Theme.of(context).colorScheme.surface,
      elevation: UniversalPlatform.isDesktop ? 6.0 : 0.0,
      child: body,
    );
  }
}

class _McpPopover extends StatefulWidget {
  const _McpPopover({
    required this.icon,
    required this.contentBuilder,
  });

  final Widget icon;
  final WidgetBuilder contentBuilder;

  @override
  State<_McpPopover> createState() => _McpPopoverState();
}

class _McpPopoverState extends State<_McpPopover> {
  final overlayController = OverlayPortalController();
  final link = LayerLink();

  @override
  void dispose() {
    overlayController.hide();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return OverlayPortal(
      controller: overlayController,
      overlayChildBuilder: (context) => _PopoverBody(
        link: link,
        onDismiss: overlayController.hide,
        child: widget.contentBuilder(context),
      ),
      child: CompositedTransformTarget(
        link: link,
        child: IconButton(
          onPressed: () {
            if (overlayController.isShowing) {
              overlayController.hide();
            } else {
              overlayController.show();
            }
          },
          icon: widget.icon,
          tooltip: 'MCP',
        ),
      ),
    );
  }
}

class _PopoverBody extends StatelessWidget {
  const _PopoverBody({
    required this.link,
    required this.onDismiss,
    required this.child,
  });

  final LayerLink link;
  final VoidCallback onDismiss;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        Positioned.fill(
          child: GestureDetector(
            onTap: onDismiss,
            behavior: HitTestBehavior.opaque,
          ),
        ),
        CompositedTransformFollower(
          link: link,
          offset: const Offset(0, -8),
          targetAnchor: Alignment.topRight,
          followerAnchor: Alignment.bottomRight,
          showWhenUnlinked: false,
          child: child,
        ),
      ],
    );
  }
}


