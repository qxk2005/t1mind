import 'dart:async';
import 'dart:convert';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart' show FlowyDialog;
import 'package:flowy_infra_ui/style_widget/primary_rounded_button.dart';
import 'package:flowy_infra_ui/widget/separated_flex.dart';
import 'package:flutter/material.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';
import 'package:appflowy/ai/service/mcp_ai_helper.dart';
import 'package:appflowy/ai/service/mcp_cache_service.dart';
import 'package:appflowy/ai/service/appflowy_ai_service.dart';
import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';

class McpCheckDialog extends StatefulWidget {
  const McpCheckDialog({
    super.key,
    required this.config,
    this.showRawInitially = false,
    this.forceRefresh = false,
  });

  final McpEndpointConfig config;
  final bool showRawInitially;
  final bool forceRefresh;

  @override
  State<McpCheckDialog> createState() => _McpCheckDialogState();
}

class _McpCheckDialogState extends State<McpCheckDialog> {
  bool _showRaw = false;
  McpCheckResult? _result;
  Object? _error;
  bool _loading = true;
  final McpCacheService _cacheService = McpCacheService();
  bool _isFromCache = false;

  @override
  void initState() {
    super.initState();
    _showRaw = widget.showRawInitially;
    _loadResultWithCache();
  }

  /// Load result with cache support
  Future<void> _loadResultWithCache() async {
    setState(() {
      _loading = true;
      _error = null;
      _result = null;
      _isFromCache = false;
    });

    try {
      // Try to load from cache first (unless force refresh is requested)
      if (!widget.forceRefresh) {
        final cachedResult = await _cacheService.loadMcpCheckResult(widget.config);
        if (cachedResult != null) {
          print('MCP Check: Loaded result from cache for ${widget.config.name}');
          if (!mounted) return;
          setState(() {
            _result = cachedResult;
            _loading = false;
            _isFromCache = true;
          });
          return;
        }
      }

      // If no cache or force refresh, perform actual check
      await _performFreshCheck();
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e;
        _loading = false;
      });
    }
  }

  /// Perform fresh MCP check and save to cache
  Future<void> _performFreshCheck() async {
    print('MCP Check: Performing fresh check for ${widget.config.name}');
    
    final res = await McpFfi.check(widget.config);
    if (!mounted) return;
    
    // Save to cache
    await _cacheService.saveMcpCheckResult(widget.config, res);
    
    setState(() {
      _result = res;
      _loading = false;
      _isFromCache = false;
    });
  }

  /// Refresh check (force fresh check)
  Future<void> _refreshCheck() async {
    await _cacheService.clearAllCache(widget.config);
    await _performFreshCheck();
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
                  Row(
                    children: [
                      Text('一键检查',
                          style: theme.textStyle.heading4
                              .standard(color: theme.textColorScheme.primary)),
                      const SizedBox(width: 12),
                      if (_isFromCache)
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          decoration: BoxDecoration(
                            color: theme.textColorScheme.secondary.withValues(alpha: 0.1),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            '缓存',
                            style: theme.textStyle.caption.standard(
                              color: theme.textColorScheme.secondary,
                            ),
                          ),
                        ),
                      const SizedBox(width: 8),
                      if (!_loading)
                        IconButton(
                          icon: Icon(
                            Icons.refresh,
                            size: 20,
                            color: theme.textColorScheme.primary,
                          ),
                          onPressed: _refreshCheck,
                          tooltip: '刷新检查',
                        ),
                    ],
                  ),
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

class _ParsedView extends StatefulWidget {
  const _ParsedView({required this.result});
  final McpCheckResult result;

  @override
  State<_ParsedView> createState() => _ParsedViewState();
}

class _ParsedViewState extends State<_ParsedView> {
  Map<String, String>? _aiDescriptions;
  bool _isGeneratingDescriptions = false;
  String? _generationStatus;
  final McpCacheService _cacheService = McpCacheService();

  @override
  void initState() {
    super.initState();
    _checkAndLoadAIDescriptions();
  }

  /// Check and load AI descriptions intelligently
  Future<void> _checkAndLoadAIDescriptions() async {
    final tools = widget.result.parsed?.tools ?? const [];
    if (tools.isEmpty) {
      print('MCP AI: No tools to generate descriptions for');
      return;
    }

    // Get config and check force refresh
    final config = _getEndpointConfig();
    final dialogState = context.findAncestorStateOfType<_McpCheckDialogState>();
    final forceRefresh = dialogState?.widget.forceRefresh ?? false;
    
    if (config == null) {
      print('MCP AI: No endpoint config available');
      return;
    }

    // Check if we already have AI descriptions cached
    final hasAiDescriptions = await _cacheService.hasCachedAiDescriptions(config);
    
    if (hasAiDescriptions && !forceRefresh) {
      // Load existing descriptions from cache
      print('MCP AI: Found existing AI descriptions, loading from cache');
      final cachedDescriptions = await _cacheService.loadAiDescriptions(config);
      if (cachedDescriptions != null && mounted) {
        setState(() {
          _aiDescriptions = cachedDescriptions;
        });
      }
    } else if (forceRefresh || !hasAiDescriptions) {
      // Generate new descriptions only if forced or no cache exists
      if (forceRefresh) {
        print('MCP AI: Force refresh requested, generating fresh AI descriptions');
      } else {
        print('MCP AI: No cached AI descriptions found, generating new ones');
      }
      await _generateAIDescriptionsBatch();
    }
  }


  /// Generate AI descriptions using batch method
  Future<void> _generateAIDescriptionsBatch() async {
    final tools = widget.result.parsed?.tools ?? const [];
    if (tools.isEmpty) {
      print('MCP AI: No tools to generate descriptions for');
      return;
    }

    print('MCP AI: Starting AI availability check...');
    final isAIAvailable = await _isAIAvailable();
    print('MCP AI: AI availability check result: $isAIAvailable');
    
    if (!isAIAvailable) {
      print('MCP AI: AI service not available, skipping description generation');
      if (mounted) {
        setState(() {
          _isGeneratingDescriptions = true;
          _generationStatus = 'AI服务不可用';
        });
        
        Future.delayed(const Duration(seconds: 2), () {
          if (mounted) {
            setState(() {
              _isGeneratingDescriptions = false;
              _generationStatus = null;
            });
          }
        });
      }
      return;
    }

    print('MCP AI: AI service available, starting batch description generation for ${tools.length} tools');

    setState(() {
      _isGeneratingDescriptions = true;
      _generationStatus = '准备批量生成...';
    });

    try {
      final aiService = AppFlowyAIService();
      final aiHelper = McpAiHelper(aiService);
      
      // Use batch generation method
      final descriptions = await aiHelper.generateToolDescriptionsBatch(
        tools,
        onProgress: (status) {
          print('MCP AI: Batch progress - $status');
          if (mounted) {
            setState(() {
              _generationStatus = status;
            });
          }
        },
      );
      
      print('MCP AI: Batch description generation completed, got ${descriptions.length} descriptions');
      
      // Save to cache
      final config = _getEndpointConfig();
      if (config != null && descriptions.isNotEmpty) {
        await _cacheService.saveAiDescriptions(config, descriptions);
      }
      
      if (mounted) {
        setState(() {
          _aiDescriptions = descriptions;
          _isGeneratingDescriptions = false;
          _generationStatus = null;
        });
      }
    } catch (e) {
      print('MCP AI: Error during batch description generation: $e');
      if (mounted) {
        setState(() {
          _isGeneratingDescriptions = false;
          _generationStatus = '生成失败: $e';
        });
        
        // Clear error message after 3 seconds
        Future.delayed(const Duration(seconds: 3), () {
          if (mounted) {
            setState(() {
              _generationStatus = null;
            });
          }
        });
      }
    }
  }

  /// Get endpoint config from parent context
  McpEndpointConfig? _getEndpointConfig() {
    try {
      // Try to find the parent McpCheckDialog
      final dialogState = context.findAncestorStateOfType<_McpCheckDialogState>();
      return dialogState?.widget.config;
    } catch (e) {
      print('MCP AI: Could not get endpoint config: $e');
      return null;
    }
  }

  Future<bool> _isAIAvailable() async {
    try {
      print('MCP AI: Checking AI availability...');
      
      // Check if there's a selected AI model
      final result = await AIEventGetSourceModelSelection(
        ModelSourcePB(source: 'mcp_check'),
      ).send();
      
      return result.fold(
        (modelSelection) {
          final selectedModel = modelSelection.selectedModel;
          final hasSelectedModel = selectedModel != null;
          print('MCP AI: Has selected model: $hasSelectedModel');
          if (hasSelectedModel) {
            print('MCP AI: Selected model: ${selectedModel.name}');
          }
          return hasSelectedModel;
        },
        (error) {
          print('MCP AI: Error getting model selection: $error');
          return false;
        },
      );
    } catch (e) {
      print('MCP AI: Error checking AI availability: $e');
      return false;
    }
  }

  @override
  Widget build(BuildContext context) {
    final tools = widget.result.parsed?.tools ?? const [];
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
                _Badge(text: '工具数量: ${widget.result.toolCount}'),
                const SizedBox(width: 8),
                if (widget.result.parsed?.server != null)
                  _Badge(text: widget.result.parsed!.server!),
                const SizedBox(width: 8),
                if (_isGeneratingDescriptions)
                  _AIBatchProgressBadge(
                    status: _generationStatus ?? 'AI 批量生成中...',
                  ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: gap,
              runSpacing: gap,
              children: [
                for (final t in tools)
                  SizedBox(
                    width: itemWidth, 
                    child: _ToolCard(
                      schema: t,
                      aiDescription: _aiDescriptions?[t.name],
                    ),
                  ),
              ],
            ),
          ],
        );
      },
    );
  }
}

class _ToolCard extends StatefulWidget {
  const _ToolCard({required this.schema, this.aiDescription});
  final McpToolSchema schema;
  final String? aiDescription;

  @override
  State<_ToolCard> createState() => _ToolCardState();
}

class _ToolCardState extends State<_ToolCard> {
  bool _showJson = false;

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
                  widget.schema.name,
                  style: theme.textStyle.heading4
                      .standard(color: theme.textColorScheme.primary),
                  overflow: TextOverflow.ellipsis,
                  maxLines: 1,
                ),
              ),
              TextButton(
                onPressed: () => setState(() => _showJson = !_showJson),
                child: Text(
                  _showJson ? '查看参数' : '查看JSON',
                  style: theme.textStyle.caption.standard(
                    color: theme.textColorScheme.primary,
                  ),
                ),
              ),
            ],
          ),
          if ((widget.schema.description ?? '').isNotEmpty) ...[
            const SizedBox(height: 4),
            Text(
              widget.schema.description!,
              style: theme.textStyle.body
                  .standard(color: theme.textColorScheme.secondary),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
          const SizedBox(height: 8),
          if (_showJson)
            _KVPanel(
              title: 'JSON Schema',
              content: _schemaToPrettyJson(widget.schema),
            )
          else if (widget.aiDescription != null)
            _AIGeneratedDescription(description: widget.aiDescription!)
          else
            _HumanReadableSchema(schema: widget.schema),
        ],
      ),
    );
  }

  String _schemaToPrettyJson(McpToolSchema s) {
    return const JsonEncoder.withIndent('  ').convert(s.toJson());
  }
}

class _AIBatchProgressBadge extends StatelessWidget {
  const _AIBatchProgressBadge({required this.status});

  final String status;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: theme.spacing.m,
        vertical: theme.spacing.xs,
      ),
      decoration: BoxDecoration(
        color: theme.textColorScheme.primary.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: theme.textColorScheme.primary.withValues(alpha: 0.3),
          width: 1,
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            width: 12,
            height: 12,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              color: theme.textColorScheme.primary,
            ),
          ),
          const SizedBox(width: 6),
          Text(
            status,
            style: theme.textStyle.caption.standard(
              color: theme.textColorScheme.primary,
            ),
          ),
        ],
      ),
    );
  }
}

class _AIProgressBadge extends StatefulWidget {
  const _AIProgressBadge({
    required this.progress,
    required this.total,
    this.currentTool,
  });

  final int progress;
  final int total;
  final String? currentTool;

  @override
  State<_AIProgressBadge> createState() => _AIProgressBadgeState();
}

class _AIProgressBadgeState extends State<_AIProgressBadge> {
  bool _showDetails = false;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    final progressText = 'AI 生成中 ${widget.progress}/${widget.total}';
    
    return GestureDetector(
      onTap: () {
        setState(() {
          _showDetails = !_showDetails;
        });
      },
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: theme.spacing.m,
          vertical: theme.spacing.xs,
        ),
        decoration: BoxDecoration(
          color: theme.textColorScheme.primary.withOpacity(0.1),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: theme.textColorScheme.primary.withOpacity(0.3),
            width: 1,
          ),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: 12,
                  height: 12,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    value: widget.total > 0 ? widget.progress / widget.total : null,
                    color: theme.textColorScheme.primary,
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  progressText,
                  style: theme.textStyle.caption.standard(
                    color: theme.textColorScheme.primary,
                  ),
                ),
                const SizedBox(width: 4),
                Icon(
                  _showDetails ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                  color: theme.textColorScheme.primary,
                ),
              ],
            ),
            if (_showDetails) ...[
              const SizedBox(height: 4),
              if (widget.currentTool != null)
                Container(
                  padding: EdgeInsets.symmetric(
                    horizontal: theme.spacing.xs,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: theme.surfaceContainerColorScheme.layer03,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    '正在处理: ${widget.currentTool}',
                    style: theme.textStyle.caption.standard(
                      color: theme.textColorScheme.secondary,
                    ),
                  ),
                )
              else
                Text(
                  '等待下一个工具...',
                  style: theme.textStyle.caption.standard(
                    color: theme.textColorScheme.secondary,
                  ),
                ),
            ],
          ],
        ),
      ),
    );
  }
}

class _AIGeneratedDescription extends StatelessWidget {
  const _AIGeneratedDescription({required this.description});
  final String description;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return Container(
      padding: EdgeInsets.all(theme.spacing.m),
      decoration: BoxDecoration(
        color: theme.surfaceContainerColorScheme.layer02,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: theme.textColorScheme.primary.withOpacity(0.2),
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.auto_awesome,
                size: 16,
                color: theme.textColorScheme.primary,
              ),
              const SizedBox(width: 4),
              Text(
                'AI 生成的说明',
                style: theme.textStyle.caption.standard(
                  color: theme.textColorScheme.primary,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          SelectableText(
            description,
            style: theme.textStyle.body.standard(
              color: theme.textColorScheme.primary,
            ),
          ),
        ],
      ),
    );
  }
}

class _HumanReadableSchema extends StatelessWidget {
  const _HumanReadableSchema({required this.schema});
  final McpToolSchema schema;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    if (schema.fields.isEmpty) {
      return Container(
        padding: EdgeInsets.all(theme.spacing.m),
        decoration: BoxDecoration(
          color: theme.surfaceContainerColorScheme.layer02,
          borderRadius: BorderRadius.circular(6),
        ),
        child: Text(
          '此工具无需参数',
          style: theme.textStyle.body.standard(
            color: theme.textColorScheme.secondary,
          ),
        ),
      );
    }

    return Container(
      padding: EdgeInsets.all(theme.spacing.m),
      decoration: BoxDecoration(
        color: theme.surfaceContainerColorScheme.layer02,
        borderRadius: BorderRadius.circular(6),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '参数 (${schema.fields.length})',
            style: theme.textStyle.caption.standard(
              color: theme.textColorScheme.secondary,
            ),
          ),
          const SizedBox(height: 8),
          ...schema.fields.map((field) => Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  padding: EdgeInsets.symmetric(
                    horizontal: theme.spacing.xs,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: field.required 
                        ? theme.textColorScheme.error.withOpacity(0.1)
                        : theme.textColorScheme.secondary.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    field.required ? '必需' : '可选',
                    style: theme.textStyle.caption.standard(
                      color: field.required 
                          ? theme.textColorScheme.error
                          : theme.textColorScheme.secondary,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        field.name,
                        style: theme.textStyle.body.standard(
                          color: theme.textColorScheme.primary,
                        ),
                      ),
                      Text(
                        '类型: ${field.type}',
                        style: theme.textStyle.caption.standard(
                          color: theme.textColorScheme.secondary,
                        ),
                      ),
                      if (field.defaultValue != null)
                        Text(
                          '默认值: ${field.defaultValue}',
                          style: theme.textStyle.caption.standard(
                            color: theme.textColorScheme.secondary,
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          )),
        ],
      ),
    );
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


