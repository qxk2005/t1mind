import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/plugins/ai_chat/application/web_search_settings_bloc.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/hover.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 信息源类型枚举
enum SourceType {
  /// 文档页面
  document,
  /// 网络搜索
  webSearch,
}

/// 信息源选择器组件
/// 提供信息源选择的下拉菜单，包括文档页面和网络搜索选项
class SourceSelector extends StatefulWidget {
  const SourceSelector({
    super.key,
    required this.selectedSourcesNotifier,
    required this.onUpdateSelectedSources,
    this.compact = false,
  });

  /// 当前选中的信息源ID列表
  final ValueNotifier<List<String>> selectedSourcesNotifier;
  
  /// 更新选中信息源的回调
  final void Function(List<String>) onUpdateSelectedSources;
  
  /// 是否使用紧凑模式
  final bool compact;

  @override
  State<SourceSelector> createState() => _SourceSelectorState();
}

class _SourceSelectorState extends State<SourceSelector> {
  final popoverController = PopoverController();
  
  /// 当前选中的信息源类型
  SourceType? _selectedSourceType;
  
  /// 网络搜索是否可用
  bool _isWebSearchAvailable = false;

  @override
  void initState() {
    super.initState();
    widget.selectedSourcesNotifier.addListener(_onSelectedSourcesChanged);
    _updateSourceType();
  }

  @override
  void dispose() {
    widget.selectedSourcesNotifier.removeListener(_onSelectedSourcesChanged);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => WebSearchSettingsBloc()..add(const WebSearchSettingsEvent.started()),
      child: BlocBuilder<WebSearchSettingsBloc, WebSearchSettingsState>(
        builder: (context, state) {
          _isWebSearchAvailable = state.globalConfig?.enabled ?? false;
          
          return AppFlowyPopover(
            controller: popoverController,
            constraints: BoxConstraints.loose(const Size(280, 200)),
            offset: const Offset(0.0, -10.0),
            direction: PopoverDirection.topWithCenterAligned,
            margin: EdgeInsets.zero,
            onOpen: () {
              _updateSourceType();
            },
            onClose: () {
              _applySelection();
            },
            popupBuilder: (_) => _SourceSelectorContent(
              selectedSourceType: _selectedSourceType,
              isWebSearchAvailable: _isWebSearchAvailable,
              onSourceTypeChanged: (sourceType) {
                setState(() {
                  _selectedSourceType = sourceType;
                });
              },
            ),
            child: _SourceSelectorButton(
              selectedSourceType: _selectedSourceType,
              isWebSearchAvailable: _isWebSearchAvailable,
              compact: widget.compact,
              onTap: () => popoverController.show(),
            ),
          );
        },
      ),
    );
  }

  void _onSelectedSourcesChanged() {
    _updateSourceType();
  }

  void _updateSourceType() {
    final selectedSources = widget.selectedSourcesNotifier.value;
    
    if (selectedSources.isEmpty) {
      _selectedSourceType = null;
    } else if (selectedSources.contains('web_search')) {
      _selectedSourceType = SourceType.webSearch;
    } else {
      _selectedSourceType = SourceType.document;
    }
    
    if (mounted) {
      setState(() {});
    }
  }

  void _applySelection() {
    List<String> newSources = [];
    
    switch (_selectedSourceType) {
      case SourceType.webSearch:
        newSources = ['web_search'];
        break;
      case SourceType.document:
        // 保持现有的文档选择
        newSources = widget.selectedSourcesNotifier.value
            .where((id) => id != 'web_search')
            .toList();
        break;
      case null:
        newSources = [];
        break;
    }
    
    widget.onUpdateSelectedSources(newSources);
  }
}

/// 信息源选择器按钮
class _SourceSelectorButton extends StatelessWidget {
  const _SourceSelectorButton({
    required this.selectedSourceType,
    required this.isWebSearchAvailable,
    required this.compact,
    required this.onTap,
  });

  final SourceType? selectedSourceType;
  final bool isWebSearchAvailable;
  final bool compact;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final height = compact ? 32.0 : 40.0;
    final fontSize = compact ? 12.0 : 14.0;
    
    return GestureDetector(
      onTap: onTap,
      behavior: HitTestBehavior.opaque,
      child: Semantics(
        label: _getSemanticLabel(),
        hint: LocaleKeys.chat_selectSource.tr(),
        button: true,
        child: SizedBox(
          height: height,
          child: FlowyHover(
            style: HoverStyle(
              borderRadius: BorderRadius.circular(8),
            ),
            builder: (context, isHovering) => Padding(
              padding: EdgeInsets.symmetric(
                horizontal: compact ? 8.0 : 12.0,
                vertical: compact ? 6.0 : 8.0,
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  _buildSourceIcon(context),
                  const HSpace(6.0),
                  _buildSourceLabel(context, fontSize),
                  const HSpace(4.0),
                  FlowySvg(
                    FlowySvgs.ai_source_drop_down_s,
                    color: Theme.of(context).hintColor,
                    size: Size.square(compact ? 8.0 : 10.0),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSourceIcon(BuildContext context) {
    switch (selectedSourceType) {
      case SourceType.webSearch:
        return FlowySvg(
          FlowySvgs.ai_sparks_s,
          color: Theme.of(context).hintColor,
          size: const Size.square(16),
        );
      case SourceType.document:
        return FlowySvg(
          FlowySvgs.ai_page_s,
          color: Theme.of(context).hintColor,
          size: const Size.square(16),
        );
      case null:
        return FlowySvg(
          FlowySvgs.ai_page_s,
          color: Theme.of(context).hintColor,
          size: const Size.square(16),
        );
    }
  }

  Widget _buildSourceLabel(BuildContext context, double fontSize) {
    String label;
    
    switch (selectedSourceType) {
      case SourceType.webSearch:
        label = LocaleKeys.chat_webSearch.tr();
        break;
      case SourceType.document:
        label = LocaleKeys.chat_documents.tr();
        break;
      case null:
        label = LocaleKeys.chat_selectSource.tr();
        break;
    }
    
    return FlowyText(
      label,
      fontSize: fontSize,
      figmaLineHeight: fontSize + 4,
      color: Theme.of(context).hintColor,
    );
  }

  String _getSemanticLabel() {
    switch (selectedSourceType) {
      case SourceType.webSearch:
        return LocaleKeys.chat_webSearch.tr();
      case SourceType.document:
        return LocaleKeys.chat_documents.tr();
      case null:
        return LocaleKeys.chat_selectSource.tr();
    }
  }
}

/// 信息源选择器内容
class _SourceSelectorContent extends StatelessWidget {
  const _SourceSelectorContent({
    required this.selectedSourceType,
    required this.isWebSearchAvailable,
    required this.onSourceTypeChanged,
  });

  final SourceType? selectedSourceType;
  final bool isWebSearchAvailable;
  final void Function(SourceType?) onSourceTypeChanged;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildSourceOption(
            context,
            SourceType.document,
            LocaleKeys.chat_documents.tr(),
            FlowySvgs.ai_page_s,
            LocaleKeys.chat_documentsDescription.tr(),
          ),
          if (isWebSearchAvailable) ...[
            const VSpace(4),
            _buildSourceOption(
              context,
              SourceType.webSearch,
              LocaleKeys.chat_webSearch.tr(),
              FlowySvgs.ai_sparks_s,
              LocaleKeys.chat_webSearchDescription.tr(),
            ),
          ],
          const VSpace(4),
          _buildSourceOption(
            context,
            null,
            LocaleKeys.chat_noSource.tr(),
            FlowySvgs.ai_page_s,
            LocaleKeys.chat_noSourceDescription.tr(),
          ),
        ],
      ),
    );
  }

  Widget _buildSourceOption(
    BuildContext context,
    SourceType? sourceType,
    String title,
    FlowySvgData icon,
    String description,
  ) {
    final isSelected = selectedSourceType == sourceType;
    
    return InkWell(
      onTap: () => onSourceTypeChanged(sourceType),
      borderRadius: BorderRadius.circular(8),
      child: Semantics(
        label: title,
        hint: description,
        button: true,
        selected: isSelected,
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(
            children: [
              FlowySvg(
                icon,
                color: isSelected 
                    ? Theme.of(context).colorScheme.primary
                    : Theme.of(context).hintColor,
                size: const Size.square(16),
              ),
              const HSpace(12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    FlowyText.medium(
                      title,
                      fontSize: 14,
                      color: isSelected 
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).colorScheme.onSurface,
                    ),
                    const VSpace(2),
                    FlowyText(
                      description,
                      fontSize: 12,
                      color: Theme.of(context).hintColor,
                    ),
                  ],
                ),
              ),
              if (isSelected)
                FlowySvg(
                  FlowySvgs.check_s,
                  color: Theme.of(context).colorScheme.primary,
                  size: const Size.square(16),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
