import 'dart:convert';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart' show FlowyDialog;
import 'package:flowy_infra_ui/style_widget/primary_rounded_button.dart';
import 'package:flowy_infra_ui/widget/separated_flex.dart';
import 'package:flutter/material.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';

class McpCheckDialog extends StatefulWidget {
  const McpCheckDialog({
    super.key,
    required this.config,
    this.showRawInitially = false,
  });

  final McpEndpointConfig config;
  final bool showRawInitially;

  @override
  State<McpCheckDialog> createState() => _McpCheckDialogState();
}

class _McpCheckDialogState extends State<McpCheckDialog> {
  bool _showRaw = false;
  McpCheckResult? _result;
  Object? _error;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _showRaw = widget.showRawInitially;
    _run();
  }

  Future<void> _run() async {
    setState(() {
      _loading = true;
      _error = null;
      _result = null;
    });
    try {
      final res = await McpFfi.check(widget.config);
      if (!mounted) return;
      setState(() {
        _result = res;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e;
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return FlowyDialog(
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxHeight: MediaQuery.of(context).size.height * 0.9,
          maxWidth: 800,
        ),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('一键检查',
                      style: theme.textStyle.heading4
                          .standard(color: theme.textColorScheme.primary)),
                  Row(children: [
                    Text('显示原始 I/O',
                        style: theme.textStyle.body.standard()),
                    const SizedBox(width: 8),
                    Switch(
                      value: _showRaw,
                      onChanged: (v) => setState(() => _showRaw = v),
                    ),
                  ])
                ],
              ),
              const SizedBox(height: 12),
              if (_loading)
                Center(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: SizedBox(
                      height: 32,
                      width: 32,
                      child: const CircularProgressIndicator(),
                    ),
                  ),
                )
              else if (_error != null)
                _ErrorView(error: _error!)
              else if (_result != null)
                Expanded(
                  child: Scrollbar(
                    thumbVisibility: true,
                    child: SingleChildScrollView(
                      physics: const ClampingScrollPhysics(),
                      child: _showRaw
                          ? _RawIOView(result: _result!)
                          : _ParsedView(result: _result!),
                    ),
                  ),
                ),
              const SizedBox(height: 12),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  OutlinedRoundedButton(
                    text: 'Close',
                    onTap: () => Navigator.of(context).pop(_result),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ErrorView extends StatelessWidget {
  const _ErrorView({required this.error});
  final Object error;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      padding: EdgeInsets.all(theme.spacing.l),
      decoration: BoxDecoration(
        color: theme.surfaceContainerColorScheme.layer02,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        error.toString(),
        style:
            theme.textStyle.body.standard(color: theme.textColorScheme.secondary),
      ),
    );
  }
}

class _RawIOView extends StatelessWidget {
  const _RawIOView({required this.result});
  final McpCheckResult result;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final bool twoCols = constraints.maxWidth >= 640;
        if (!twoCols) {
          return SeparatedColumn(
            separatorBuilder: () => const SizedBox(height: 12),
            children: [
              _KVPanel(title: 'Request', content: result.requestJson),
              _KVPanel(title: 'Response', content: result.responseJson),
            ],
          );
        }
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: _KVPanel(title: 'Request', content: result.requestJson),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: _KVPanel(title: 'Response', content: result.responseJson),
            ),
          ],
        );
      },
    );
  }
}

class _ParsedView extends StatelessWidget {
  const _ParsedView({required this.result});
  final McpCheckResult result;

  @override
  Widget build(BuildContext context) {
    final tools = result.parsed?.tools ?? const [];
    return LayoutBuilder(
      builder: (context, constraints) {
        const double cardMinWidth = 340;
        const double gap = 12;
        final int cols = (constraints.maxWidth / (cardMinWidth + gap))
            .floor()
            .clamp(1, 3);
        final double itemWidth =
            (constraints.maxWidth - gap * (cols - 1)) / cols;

        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                _Badge(text: '工具数量: ${result.toolCount}'),
                const SizedBox(width: 8),
                if (result.parsed?.server != null)
                  _Badge(text: result.parsed!.server!),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: gap,
              runSpacing: gap,
              children: [
                for (final t in tools)
                  SizedBox(width: itemWidth, child: _ToolCard(schema: t)),
              ],
            ),
          ],
        );
      },
    );
  }
}

class _ToolCard extends StatelessWidget {
  const _ToolCard({required this.schema});
  final McpToolSchema schema;

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
          Text(
            schema.name,
            style: theme.textStyle.heading4
                .standard(color: theme.textColorScheme.primary),
            overflow: TextOverflow.ellipsis,
            maxLines: 1,
          ),
          if ((schema.description ?? '').isNotEmpty) ...[
            const SizedBox(height: 4),
            Text(
              schema.description!,
              style: theme.textStyle.body
                  .standard(color: theme.textColorScheme.secondary),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
          const SizedBox(height: 8),
          _KVPanel(
            title: 'Schema',
            content: _schemaToPrettyJson(schema),
          ),
        ],
      ),
    );
  }

  String _schemaToPrettyJson(McpToolSchema s) {
    return const JsonEncoder.withIndent('  ').convert(s.toJson());
  }
}

class _KVPanel extends StatelessWidget {
  const _KVPanel({required this.title, required this.content});
  final String title;
  final String content;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: theme.textStyle.caption
            .standard(color: theme.textColorScheme.secondary)),
        const SizedBox(height: 6),
        Container(
          padding: EdgeInsets.all(theme.spacing.m),
          decoration: BoxDecoration(
            color: theme.surfaceContainerColorScheme.layer02,
            borderRadius: BorderRadius.circular(6),
          ),
          constraints: const BoxConstraints(
            // cap each JSON panel's height to allow scrolling inside
            maxHeight: 260,
          ),
          child: Scrollbar(
            thumbVisibility: true,
            child: SingleChildScrollView(
              physics: const ClampingScrollPhysics(),
              child: SelectableText(
                content,
                style: theme.textStyle.body
                    .standard(color: theme.textColorScheme.secondary),
              ),
            ),
          ),
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


