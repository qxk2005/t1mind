import 'dart:io';

import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy_backend/log.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';
import 'package:string_validator/string_validator.dart';
import 'package:url_launcher/url_launcher.dart';

/// 网络搜索结果引用显示组件
/// 
/// 用于在AI聊天消息中显示网络搜索结果的引用信息，
/// 包括可点击的链接、标题、摘要和域名信息。
class CitationDisplay extends StatefulWidget {
  const CitationDisplay({
    super.key,
    required this.citations,
    this.maxVisibleCitations = 5,
    this.showExpandButton = true,
  });

  /// 引用列表
  final List<CitationInfo> citations;
  
  /// 默认显示的最大引用数量
  final int maxVisibleCitations;
  
  /// 是否显示展开/折叠按钮
  final bool showExpandButton;

  @override
  State<CitationDisplay> createState() => _CitationDisplayState();
}

class _CitationDisplayState extends State<CitationDisplay> {
  bool _isExpanded = false;

  @override
  Widget build(BuildContext context) {
    if (widget.citations.isEmpty) {
      return const SizedBox.shrink();
    }

    final visibleCitations = _isExpanded 
        ? widget.citations 
        : widget.citations.take(widget.maxVisibleCitations).toList();
    
    final hasMoreCitations = widget.citations.length > widget.maxVisibleCitations;

    return AnimatedSize(
      duration: const Duration(milliseconds: 200),
      alignment: AlignmentDirectional.topStart,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const VSpace(8.0),
          
          // 引用标题和展开按钮
          if (widget.showExpandButton && hasMoreCitations)
            _buildCitationHeader()
          else if (widget.citations.length > 1)
            _buildCitationTitle(),
          
          const VSpace(4.0),
          
          // 引用列表
          ...visibleCitations.map((citation) => 
            _CitationItem(
              citation: citation,
              onTap: () => _handleCitationTap(citation),
            ),
          ),
          
          // 展开/折叠提示
          if (widget.showExpandButton && hasMoreCitations)
            _buildExpandHint(),
        ],
      ),
    );
  }

  Widget _buildCitationHeader() {
    return ConstrainedBox(
      constraints: const BoxConstraints(
        maxHeight: 24,
        maxWidth: 240,
      ),
      child: FlowyButton(
        margin: const EdgeInsets.all(4.0),
        useIntrinsicWidth: true,
        hoverColor: Colors.transparent,
        radius: BorderRadius.circular(8.0),
              text: FlowyText(
                "找到 ${widget.citations.length} 个来源",
                fontSize: 12,
                color: Theme.of(context).hintColor,
              ),
        rightIcon: FlowySvg(
          _isExpanded ? FlowySvgs.arrow_up_s : FlowySvgs.arrow_down_s,
          size: const Size.square(10),
        ),
        onTap: () {
          setState(() => _isExpanded = !_isExpanded);
        },
      ),
    );
  }

  Widget _buildCitationTitle() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 4.0),
      child: FlowyText(
        "找到 ${widget.citations.length} 个来源",
        fontSize: 12,
        color: Theme.of(context).hintColor,
      ),
    );
  }

  Widget _buildExpandHint() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 4.0, vertical: 2.0),
      child: FlowyText(
        _isExpanded 
            ? "收起结果"
            : "展开 ${widget.citations.length - widget.maxVisibleCitations} 个更多结果",
        fontSize: 11,
        color: Theme.of(context).hintColor.withOpacity(0.7),
      ),
    );
  }

  /// 处理引用点击事件
  Future<void> _handleCitationTap(CitationInfo citation) async {
    if (!isURL(citation.url)) {
      Log.warn('Invalid URL in citation: ${citation.url}');
      return;
    }

    try {
      final uri = Uri.parse(citation.url);
      
      // 验证URL格式
      if (!uri.hasScheme) {
        Log.warn('URL missing scheme: ${citation.url}');
        return;
      }

      // 检查是否为本地主机地址
      if (uri.scheme == 'localhost' || uri.host == 'localhost') {
        // 对于本地主机，尝试添加http协议
        final localUri = Uri.parse('http://${citation.url}');
        try {
          await InternetAddress.lookup(localUri.host);
          await launchUrl(localUri, mode: LaunchMode.externalApplication);
        } catch (e) {
          Log.error('Failed to resolve localhost address: $e');
        }
        return;
      }

      // 启动外部浏览器打开链接
      final launched = await launchUrl(
        uri,
        mode: LaunchMode.externalApplication,
        webOnlyWindowName: '_blank',
      );

      if (!launched) {
        Log.error('Failed to launch URL: ${citation.url}');
      }
    } catch (e) {
      Log.error('Error launching URL ${citation.url}: $e');
    }
  }
}

/// 单个引用项组件
class _CitationItem extends StatelessWidget {
  const _CitationItem({
    required this.citation,
    required this.onTap,
  });

  final CitationInfo citation;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 2.0, horizontal: 4.0),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(8.0),
          onTap: onTap,
          child: Container(
            padding: const EdgeInsets.all(8.0),
            decoration: BoxDecoration(
              border: Border.all(
                color: Theme.of(context).dividerColor.withOpacity(0.3),
                width: 1.0,
              ),
              borderRadius: BorderRadius.circular(8.0),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 标题和链接图标
                Row(
                  children: [
                    FlowySvg(
                      FlowySvgs.toolbar_link_earth_m,
                      size: const Size.square(14),
                      color: Theme.of(context).hintColor,
                    ),
                    const HSpace(6.0),
                    Expanded(
                      child: FlowyText(
                        citation.title.isNotEmpty ? citation.title : citation.url,
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: Theme.of(context).colorScheme.primary,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ],
                ),
                
                const VSpace(4.0),
                
                // 域名信息
                if (citation.domain.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 4.0),
                    child: FlowyText(
                      citation.domain,
                      fontSize: 11,
                      color: Theme.of(context).hintColor,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                
                // 摘要内容
                if (citation.snippet.isNotEmpty)
                  FlowyText(
                    citation.snippet,
                    fontSize: 12,
                    color: Theme.of(context).textTheme.bodyMedium?.color,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                    lineHeight: 1.3,
                  ),
                
                // 相关性评分（可选显示）
                if (citation.relevanceScore > 0)
                  Padding(
                    padding: const EdgeInsets.only(top: 4.0),
                    child: Row(
                      children: [
                        Icon(
                          Icons.star,
                          size: 12,
                          color: Theme.of(context).hintColor,
                        ),
                        const HSpace(2.0),
                        FlowyText(
                          '${(citation.relevanceScore * 100).toInt()}%',
                          fontSize: 10,
                          color: Theme.of(context).hintColor,
                        ),
                      ],
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// 引用信息数据类
/// 
/// 表示一个网络搜索结果的引用信息，包含标题、URL、域名、摘要和相关性评分。
class CitationInfo {
  const CitationInfo({
    required this.title,
    required this.url,
    required this.domain,
    required this.snippet,
    this.relevanceScore = 0.0,
    this.index = 0,
  });

  /// 引用标题
  final String title;
  
  /// 引用URL
  final String url;
  
  /// 引用域名
  final String domain;
  
  /// 引用摘要
  final String snippet;
  
  /// 相关性评分 (0.0 - 1.0)
  final double relevanceScore;
  
  /// 引用索引
  final int index;

  /// 从ChatMessageRefSource创建CitationInfo
  factory CitationInfo.fromRefSource(ChatMessageRefSource refSource) {
    return CitationInfo(
      title: refSource.name,
      url: refSource.id,
      domain: _extractDomain(refSource.id),
      snippet: '', // ChatMessageRefSource 不包含摘要信息
      relevanceScore: 0.0,
    );
  }

  /// 从Map创建CitationInfo（用于从Rust后端接收数据）
  factory CitationInfo.fromMap(Map<String, dynamic> map) {
    return CitationInfo(
      title: map['title']?.toString() ?? '',
      url: map['url']?.toString() ?? '',
      domain: map['domain']?.toString() ?? '',
      snippet: map['snippet']?.toString() ?? '',
      relevanceScore: (map['relevance_score'] as num?)?.toDouble() ?? 0.0,
      index: (map['index'] as num?)?.toInt() ?? 0,
    );
  }

  /// 转换为Map（用于发送到Rust后端）
  Map<String, dynamic> toMap() {
    return {
      'title': title,
      'url': url,
      'domain': domain,
      'snippet': snippet,
      'relevance_score': relevanceScore,
      'index': index,
    };
  }

  /// 获取格式化的引用文本
  String get formattedText {
    if (title.isNotEmpty) {
      return '[$title]($url)';
    }
    return url;
  }

  /// 获取短格式引用文本
  String get shortFormattedText {
    if (title.isNotEmpty) {
      return '${index > 0 ? '$index. ' : ''}$title';
    }
    return '${index > 0 ? '$index. ' : ''}$domain';
  }

  @override
  String toString() {
    return 'CitationInfo(title: $title, url: $url, domain: $domain, relevanceScore: $relevanceScore)';
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is CitationInfo &&
        other.title == title &&
        other.url == url &&
        other.domain == domain &&
        other.snippet == snippet &&
        other.relevanceScore == relevanceScore &&
        other.index == index;
  }

  @override
  int get hashCode {
    return Object.hash(title, url, domain, snippet, relevanceScore, index);
  }

  /// 从URL提取域名
  static String _extractDomain(String url) {
    try {
      final uri = Uri.parse(url);
      return uri.host;
    } catch (e) {
      // 如果解析失败，尝试简单提取
      final parts = url.split('/');
      if (parts.length >= 3) {
        return parts[2];
      }
      return url;
    }
  }
}

/// 引用显示组件的扩展方法
extension CitationDisplayExtensions on CitationDisplay {
  /// 从ChatMessageRefSource列表创建CitationInfo列表
  static List<CitationInfo> fromRefSources(List<ChatMessageRefSource> refSources) {
    return refSources
        .where((ref) => ref.source == 'web' && isURL(ref.id))
        .map((ref) => CitationInfo.fromRefSource(ref))
        .toList();
  }

  /// 从Map列表创建CitationInfo列表
  static List<CitationInfo> fromMapList(List<Map<String, dynamic>> maps) {
    return maps.map((map) => CitationInfo.fromMap(map)).toList();
  }
}
