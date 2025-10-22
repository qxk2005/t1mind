import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/shared/changelog/changelog_loader.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// T1Mind更新历史显示组件
/// 显示changelog内容，支持markdown格式化和滚动
/// 使用ChangelogLoader加载内容，提供良好的用户体验
class ChangelogWidget extends StatefulWidget {
  const ChangelogWidget({super.key});

  @override
  State<ChangelogWidget> createState() => _ChangelogWidgetState();
}

class _ChangelogWidgetState extends State<ChangelogWidget> {
  final ChangelogLoader _changelogLoader = ChangelogLoader();
  final ScrollController _scrollController = ScrollController();
  
  String? _changelogContent;
  bool _isLoading = true;
  bool _hasError = false;

  @override
  void initState() {
    super.initState();
    _loadChangelog();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _loadChangelog() async {
    try {
      setState(() {
        _isLoading = true;
        _hasError = false;
      });

      // 获取原始markdown内容
      final content = await _changelogLoader.getRawChangelog();
      
      if (mounted) {
        setState(() {
          _changelogContent = content;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _hasError = true;
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return SettingsCategory(
      title: LocaleKeys.settings_aboutT1mindPage_changelog_title.tr(),
      description: LocaleKeys.settings_aboutT1mindPage_changelog_description.tr(),
      children: [
        if (_isLoading) ...[
          _buildLoadingState(theme),
        ] else if (_hasError) ...[
          _buildErrorState(theme),
        ] else ...[
          _buildChangelogContent(theme),
        ],
      ],
    );
  }

  Widget _buildLoadingState(AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          const CircularProgressIndicator(),
          const VSpace(16),
          FlowyText.regular(
            LocaleKeys.settings_aboutT1mindPage_changelog_loadingChangelog.tr(),
            fontSize: 14,
            color: theme.textColorScheme.secondary,
          ),
        ],
      ),
    );
  }

  Widget _buildErrorState(AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.errorContainer.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.error.withValues(alpha: 0.3),
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.error_outline,
                size: 20,
                color: Theme.of(context).colorScheme.error,
              ),
              const HSpace(8),
              Expanded(
                child: FlowyText.medium(
                  LocaleKeys.settings_aboutT1mindPage_changelog_loadFailed.tr(),
                  fontSize: 14,
                  color: Theme.of(context).colorScheme.error,
                ),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            LocaleKeys.settings_aboutT1mindPage_changelog_loadFailedDescription.tr(),
            fontSize: 12,
            color: theme.textColorScheme.secondary,
          ),
          const VSpace(12),
          SizedBox(
            width: double.infinity,
            child: FlowyButton(
              text: Text(LocaleKeys.settings_aboutT1mindPage_changelog_reload.tr()),
              onTap: _loadChangelog,
              leftIcon: const Icon(Icons.refresh, size: 16),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildChangelogContent(AppFlowyThemeData theme) {
    if (_changelogContent == null || _changelogContent!.isEmpty) {
      return _buildEmptyState(theme);
    }

    return Container(
      height: 400, // 固定高度以启用滚动
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withValues(alpha: 0.2),
          width: 1,
        ),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: Scrollbar(
          controller: _scrollController,
          thumbVisibility: true,
          trackVisibility: true,
          child: SingleChildScrollView(
            controller: _scrollController,
            physics: const ClampingScrollPhysics(),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: _buildFormattedContent(theme),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildFormattedContent(AppFlowyThemeData theme) {
    if (_changelogContent == null) return const SizedBox.shrink();

    // 简单的markdown解析和格式化
    final lines = _changelogContent!.split('\n');
    final widgets = <Widget>[];

    for (int i = 0; i < lines.length; i++) {
      final line = lines[i].trim();
      
      if (line.isEmpty) {
        widgets.add(const VSpace(8));
        continue;
      }

      // 处理标题
      if (line.startsWith('#')) {
        final level = line.split(' ').first.length;
        final text = line.substring(level).trim();
        widgets.add(_buildHeading(text, level, theme));
        widgets.add(const VSpace(8));
      }
      // 处理列表项
      else if (line.startsWith('- ') || line.startsWith('* ')) {
        final text = line.substring(2).trim();
        widgets.add(_buildListItem(text, theme));
        widgets.add(const VSpace(4));
      }
      // 处理有序列表
      else if (RegExp(r'^\d+\.\s').hasMatch(line)) {
        final text = line.replaceFirst(RegExp(r'^\d+\.\s'), '').trim();
        widgets.add(_buildOrderedListItem(text, theme));
        widgets.add(const VSpace(4));
      }
      // 处理代码块
      else if (line.startsWith('```')) {
        final codeLines = <String>[];
        i++; // 跳过开始标记
        while (i < lines.length && !lines[i].startsWith('```')) {
          codeLines.add(lines[i]);
          i++;
        }
        if (codeLines.isNotEmpty) {
          widgets.add(_buildCodeBlock(codeLines.join('\n'), theme));
          widgets.add(const VSpace(8));
        }
      }
      // 处理普通段落
      else {
        widgets.add(_buildParagraph(line, theme));
        widgets.add(const VSpace(4));
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: widgets,
    );
  }

  Widget _buildHeading(String text, int level, AppFlowyThemeData theme) {
    double fontSize;
    
    switch (level) {
      case 1:
        fontSize = 20;
        break;
      case 2:
        fontSize = 18;
        break;
      case 3:
        fontSize = 16;
        break;
      case 4:
        fontSize = 14;
        break;
      case 5:
        fontSize = 13;
        break;
      case 6:
        fontSize = 12;
        break;
      default:
        fontSize = 14;
    }

    return FlowyText.medium(
      text,
      fontSize: fontSize,
      color: theme.textColorScheme.primary,
    );
  }

  Widget _buildParagraph(String text, AppFlowyThemeData theme) {
    return FlowyText.regular(
      text,
      fontSize: 13,
      color: theme.textColorScheme.primary,
    );
  }

  Widget _buildListItem(String text, AppFlowyThemeData theme) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          margin: const EdgeInsets.only(top: 6, right: 8),
          width: 4,
          height: 4,
          decoration: BoxDecoration(
            color: theme.textColorScheme.secondary,
            shape: BoxShape.circle,
          ),
        ),
        Expanded(
          child: FlowyText.regular(
            text,
            fontSize: 13,
            color: theme.textColorScheme.primary,
          ),
        ),
      ],
    );
  }

  Widget _buildOrderedListItem(String text, AppFlowyThemeData theme) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.regular(
          '•',
          fontSize: 13,
          color: theme.textColorScheme.secondary,
        ),
        const HSpace(8),
        Expanded(
          child: FlowyText.regular(
            text,
            fontSize: 13,
            color: theme.textColorScheme.primary,
          ),
        ),
      ],
    );
  }

  Widget _buildCodeBlock(String code, AppFlowyThemeData theme) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withValues(alpha: 0.2),
          width: 1,
        ),
      ),
      child: SelectableText(
        code,
        style: TextStyle(
          fontSize: 12,
          fontFamily: 'monospace',
          color: theme.textColorScheme.primary,
          height: 1.4,
        ),
      ),
    );
  }

  Widget _buildEmptyState(AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withValues(alpha: 0.2),
          width: 1,
        ),
      ),
      child: Column(
        children: [
          Icon(
            Icons.history,
            size: 48,
            color: theme.textColorScheme.secondary,
          ),
          const VSpace(16),
          FlowyText.medium(
            LocaleKeys.settings_aboutT1mindPage_changelog_noChangelog.tr(),
            fontSize: 16,
            color: theme.textColorScheme.primary,
          ),
          const VSpace(8),
          FlowyText.regular(
            LocaleKeys.settings_aboutT1mindPage_changelog_noChangelogDescription.tr(),
            fontSize: 14,
            color: theme.textColorScheme.secondary,
          ),
        ],
      ),
    );
  }
}