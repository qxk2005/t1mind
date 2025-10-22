import 'package:flutter/services.dart';
import 'package:appflowy_backend/log.dart';
import 'package:markdown/markdown.dart' as md;

/// Changelog加载器，负责从assets加载和解析changelog.md文件
class ChangelogLoader {
  factory ChangelogLoader() => _instance;

  ChangelogLoader._internal();

  static final ChangelogLoader _instance = ChangelogLoader._internal();

  /// changelog.md文件的asset路径
  static const String _changelogAssetPath = 'assets/changelog.md';

  /// 缓存的changelog内容
  String? _cachedContent;

  /// 加载changelog.md文件内容
  /// 如果文件不存在，返回默认的版本信息
  /// 如果加载失败，记录错误并返回空字符串
  Future<String> loadChangelogContent() async {
    // 如果已经缓存了内容，直接返回
    if (_cachedContent != null) {
      Log.info('[ChangelogLoader] Returning cached changelog content');
      return _cachedContent!;
    }

    try {
      Log.info('[ChangelogLoader] Loading changelog from assets: $_changelogAssetPath');
      
      // 使用rootBundle加载asset文件
      final content = await rootBundle.loadString(_changelogAssetPath);
      
      if (content.isNotEmpty) {
        _cachedContent = content;
        Log.info('[ChangelogLoader] Successfully loaded changelog content (${content.length} characters)');
        return content;
      } else {
        Log.info('[ChangelogLoader] Changelog file is empty');
        return _getDefaultChangelogContent();
      }
    } on PlatformException catch (e) {
      Log.error('[ChangelogLoader] Platform error loading changelog: ${e.message}');
      return _getDefaultChangelogContent();
    } catch (e) {
      Log.error('[ChangelogLoader] Failed to load changelog: $e');
      return _getDefaultChangelogContent();
    }
  }

  /// 解析changelog内容，将markdown转换为HTML
  /// 如果解析失败，返回原始内容
  String parseChangelog(String rawContent) {
    try {
      Log.info('[ChangelogLoader] Parsing changelog markdown content');
      
      if (rawContent.isEmpty) {
        return _getDefaultChangelogContent();
      }

      // 使用markdown包解析内容
      final html = md.markdownToHtml(
        rawContent,
        extensionSet: md.ExtensionSet.gitHubFlavored,
        inlineSyntaxes: [
          md.StrikethroughSyntax(),
          md.AutolinkExtensionSyntax(),
          md.EmojiSyntax(),
        ],
        blockSyntaxes: [
          md.FencedCodeBlockSyntax(),
          md.TableSyntax(),
        ],
      );

      Log.info('[ChangelogLoader] Successfully parsed changelog to HTML (${html.length} characters)');
      return html;
    } catch (e) {
      Log.error('[ChangelogLoader] Failed to parse changelog markdown: $e');
      // 解析失败时返回原始内容，保持文本格式
      return _formatPlainText(rawContent);
    }
  }

  /// 获取格式化的changelog内容（HTML格式）
  /// 这是主要的公共接口，结合了加载和解析功能
  Future<String> getFormattedChangelog() async {
    try {
      final rawContent = await loadChangelogContent();
      return parseChangelog(rawContent);
    } catch (e) {
      Log.error('[ChangelogLoader] Failed to get formatted changelog: $e');
      return _getDefaultChangelogContent();
    }
  }

  /// 获取原始changelog内容（markdown格式）
  Future<String> getRawChangelog() async {
    return loadChangelogContent();
  }

  /// 检查changelog文件是否存在
  Future<bool> isChangelogAvailable() async {
    try {
      await rootBundle.loadString(_changelogAssetPath);
      return true;
    } catch (e) {
      Log.info('[ChangelogLoader] Changelog file not available: $e');
      return false;
    }
  }

  /// 清除缓存的内容
  void clearCache() {
    _cachedContent = null;
    Log.info('[ChangelogLoader] Cache cleared');
  }

  /// 获取默认的changelog内容
  /// 当changelog.md文件不存在或加载失败时使用
  String _getDefaultChangelogContent() {
    return '''
# T1Mind 更新历史

## 当前版本
感谢您使用T1Mind！这是一个基于AppFlowy的AI协作工作空间。

### 主要功能
- 📝 智能文档编辑
- 🤖 AI助手集成
- 📊 数据管理
- 🔄 实时协作

### 获取更新
请访问 [T1Mind GitHub仓库](https://github.com/qxk2005/t1mind) 查看最新版本和更新说明。

---
*此changelog将在下次更新时包含详细的版本历史记录。*
''';
  }

  /// 将纯文本内容格式化为基本的HTML
  /// 当markdown解析失败时使用
  String _formatPlainText(String content) {
    if (content.isEmpty) {
      return _getDefaultChangelogContent();
    }

    // 简单的文本格式化
    return content
        .split('\n')
        .map((line) {
          // 处理标题行（以#开头）
          if (line.trim().startsWith('#')) {
            final level = line.trim().split(' ').first.length;
            final text = line.trim().substring(level).trim();
            return '<h$level>$text</h$level>';
          }
          // 处理空行
          else if (line.trim().isEmpty) {
            return '<br>';
          }
          // 处理普通文本
          else {
            return '<p>${line.trim()}</p>';
          }
        })
        .join('\n');
  }
}

/// Changelog条目数据类
class ChangelogEntry {
  const ChangelogEntry({
    required this.version,
    this.date,
    required this.changes,
    this.type = ChangelogEntryType.feature,
  });

  final String version;
  final DateTime? date;
  final List<String> changes;
  final ChangelogEntryType type;

  @override
  String toString() {
    return 'ChangelogEntry(version: $version, date: $date, changes: ${changes.length} items, type: $type)';
  }
}

/// Changelog条目类型枚举
enum ChangelogEntryType {
  feature,
  bugfix,
  breaking,
  security,
  performance,
  other,
}

/// Changelog解析结果
class ChangelogParseResult {
  const ChangelogParseResult({
    required this.entries,
    required this.rawContent,
    required this.formattedContent,
    this.isFromCache = false,
  });

  final List<ChangelogEntry> entries;
  final String rawContent;
  final String formattedContent;
  final bool isFromCache;

  @override
  String toString() {
    return 'ChangelogParseResult(entries: ${entries.length}, formattedContent: ${formattedContent.length} chars, isFromCache: $isFromCache)';
  }
}
