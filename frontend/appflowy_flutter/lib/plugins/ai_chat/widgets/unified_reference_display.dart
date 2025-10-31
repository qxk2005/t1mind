import 'dart:io';

import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/workspace/application/view/view_ext.dart';
import 'package:appflowy/workspace/application/view/view_service.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/protobuf.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';
import 'package:string_validator/string_validator.dart';
import 'package:url_launcher/url_launcher.dart';

/// 统一的引用显示组件
/// 
/// 用于显示所有类型的引用（网络搜索、文档、MCP工具），
/// 提供一致的UI风格和用户体验。
class UnifiedReferenceDisplay extends StatefulWidget {
  const UnifiedReferenceDisplay({
    super.key,
    required this.references,
    required this.referenceType,
    this.maxVisibleReferences = 3,
    this.showExpandButton = true,
    this.onReferenceSelected,
  });

  /// 引用列表
  final List<ChatMessageRefSource> references;

  /// 引用类型
  final ReferenceType referenceType;
  
  /// 默认显示的最大引用数量
  final int maxVisibleReferences;
  
  /// 是否显示展开/折叠按钮
  final bool showExpandButton;
  
  /// 引用被点击时的回调
  final void Function(ChatMessageRefSource reference)? onReferenceSelected;

  @override
  State<UnifiedReferenceDisplay> createState() => _UnifiedReferenceDisplayState();
}

class _UnifiedReferenceDisplayState extends State<UnifiedReferenceDisplay> {
  bool _isExpanded = false;

  @override
  Widget build(BuildContext context) {
    if (widget.references.isEmpty) {
      return const SizedBox.shrink();
    }

    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    
    // 根据展开状态决定显示的引用数量
    final maxVisible = _isExpanded 
        ? widget.references.length 
        : widget.maxVisibleReferences;
    final visibleReferences = widget.references.take(maxVisible).toList();
    final hasMore = widget.references.length > widget.maxVisibleReferences;

    return Container(
      padding: const EdgeInsets.all(12.0),
      decoration: BoxDecoration(
        color: isDark 
            ? Colors.grey[850]?.withOpacity(0.5) 
            : Colors.grey[100],
        borderRadius: BorderRadius.circular(8.0),
        border: Border.all(
          color: isDark 
              ? Colors.grey[700]! 
              : Colors.grey[300]!,
          width: 1.0,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // 标题行
          _buildHeader(theme, isDark),
          const VSpace(8.0),
          
          // 引用列表
          ...visibleReferences.asMap().entries.map((entry) {
            final index = entry.key;
            final ref = entry.value;
            return Padding(
              padding: EdgeInsets.only(top: index > 0 ? 6.0 : 0),
              child: _buildReferenceItem(ref, index + 1, theme, isDark),
            );
          }),
          
          // 展开/折叠按钮
          if (hasMore && widget.showExpandButton)
            _buildExpandButton(theme),
        ],
      ),
    );
  }

  /// 构建标题行
  Widget _buildHeader(ThemeData theme, bool isDark) {
    IconData icon;
    String title;

    // TODO: 使用LocaleKeys替换硬编码字符串
    // 目前locale key生成有问题，暂时使用硬编码
    switch (widget.referenceType) {
      case ReferenceType.webSearch:
        icon = Icons.language;
        title = '网络搜索引用';
        break;
      case ReferenceType.document:
        icon = Icons.description_outlined;
        title = '文档引用';
        break;
      case ReferenceType.mcpTool:
        icon = Icons.extension_outlined;
        title = 'MCP 工具调用';
        break;
    }

    return Row(
      children: [
        Icon(
          icon,
          size: 16,
          color: theme.colorScheme.primary,
        ),
        const HSpace(8.0),
        FlowyText.medium(
          title,
          fontSize: 13,
          color: isDark ? Colors.grey[300] : Colors.grey[700],
        ),
        const Spacer(),
        FlowyText.regular(
          '${widget.references.length}${_getReferenceUnit()}',
          fontSize: 12,
          color: isDark ? Colors.grey[400] : Colors.grey[600],
        ),
      ],
    );
  }

  /// 获取引用计数单位
  String _getReferenceUnit() {
    switch (widget.referenceType) {
      case ReferenceType.webSearch:
        return ' 个';
      case ReferenceType.document:
        return ' 个文档';
      case ReferenceType.mcpTool:
        return ' 个工具';
    }
  }

  /// 构建单个引用项
  Widget _buildReferenceItem(
    ChatMessageRefSource ref,
    int index,
    ThemeData theme,
    bool isDark,
  ) {
    return InkWell(
      onTap: () => _handleReferenceTap(ref),
      borderRadius: BorderRadius.circular(6.0),
      child: Container(
        padding: const EdgeInsets.all(10.0),
        decoration: BoxDecoration(
          color: isDark ? Colors.grey[800] : Colors.white,
          borderRadius: BorderRadius.circular(6.0),
          border: Border.all(
            color: isDark ? Colors.grey[700]! : Colors.grey[300]!,
            width: 1.0,
          ),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 圆形序号
            _buildIndexCircle(index, theme),
            const HSpace(10.0),
            
            // 引用内容
            Expanded(
              child: _buildReferenceContent(ref, theme, isDark),
            ),
          ],
        ),
      ),
    );
  }

  /// 构建圆形序号
  Widget _buildIndexCircle(int index, ThemeData theme) {
    return Container(
      width: 20,
      height: 20,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: theme.colorScheme.primary.withOpacity(0.1),
        border: Border.all(
          color: theme.colorScheme.primary,
          width: 1.5,
        ),
      ),
      alignment: Alignment.center,
      child: FlowyText(
        '$index',
        fontSize: 11,
        fontWeight: FontWeight.w600,
        color: theme.colorScheme.primary,
      ),
    );
  }

  /// 构建引用内容
  Widget _buildReferenceContent(
    ChatMessageRefSource ref,
    ThemeData theme,
    bool isDark,
  ) {
    switch (widget.referenceType) {
      case ReferenceType.webSearch:
        return _buildWebSearchContent(ref, theme, isDark);
      case ReferenceType.document:
        return _buildDocumentContent(ref, theme, isDark);
      case ReferenceType.mcpTool:
        return _buildMCPToolContent(ref, theme, isDark);
    }
  }

  /// 构建网络搜索引用内容
  Widget _buildWebSearchContent(
    ChatMessageRefSource ref,
    ThemeData theme,
    bool isDark,
  ) {
    // 从URL提取域名
    String domain = '';
    try {
      final uri = Uri.parse(ref.id);
      domain = uri.host;
    } catch (e) {
      domain = ref.id;
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 标题
        Row(
          children: [
            FlowySvg(
              FlowySvgs.toolbar_link_earth_m,
              size: const Size.square(14),
              color: theme.hintColor,
            ),
            const HSpace(6.0),
            Expanded(
              child: FlowyText(
                ref.name.isNotEmpty ? ref.name : ref.id,
                fontSize: 13,
                fontWeight: FontWeight.w500,
                color: theme.colorScheme.primary,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        
        const VSpace(4.0),
        
        // 域名
        if (domain.isNotEmpty)
          FlowyText(
            domain,
            fontSize: 11,
            color: theme.hintColor,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
      ],
    );
  }

  /// 构建文档引用内容
  Widget _buildDocumentContent(
    ChatMessageRefSource ref,
    ThemeData theme,
    bool isDark,
  ) {
    // 基于 source==appflowy 时，通过 id 异步解析文档名称
    // 🔧 修复：后端只发送SOURCE_ID，前端统一通过ID自动获取文档名称
    // 这样可以确保前端总是使用正确的文档ID来获取文档名称，同时支持@mention文档（有名称）和RAG文档（无名称）
    if (ref.source == 'appflowy') {
      // 如果已经有有效的名称（来自@mention），直接使用
      // 如果没有名称或为空（来自RAG），通过ID异步获取
      final hasValidName = ref.name.isNotEmpty && 
                          ref.name != "Loading..." && 
                          ref.name != "加载中...";
      
      if (hasValidName) {
        // 已经有有效的文档名（来自@mention），直接使用，不重新加载
        return _buildDocumentContentInternal(
          ref.name,
          ref,
          theme,
          isDark,
          null, // 不需要view对象
        );
      }
      
      // 没有有效的文档名（来自RAG），通过 id 异步解析真实文档名称
      return FutureBuilder<ViewPB?>(
        future: ViewBackendService.getView(ref.id).then((f) => f.toNullable()),
        builder: (context, snapshot) {
          final view = snapshot.data;
          String displayName;
          
          if (snapshot.hasData &&
              snapshot.connectionState == ConnectionState.done &&
              view != null) {
            displayName = view.nameOrDefault;
          } else {
            // 加载中时显示占位
            displayName = '加载中...';
          }

          return _buildDocumentContentInternal(
            displayName,
            ref,
            theme,
            isDark,
            view,
          );
        },
      );
    }

    return _buildDocumentContentInternal(
      ref.name,
      ref,
      theme,
      isDark,
      null,
    );
  }

  /// 构建文档引用内容（内部实现）
  Widget _buildDocumentContentInternal(
    String displayName,
    ChatMessageRefSource ref,
    ThemeData theme,
    bool isDark,
    ViewPB? view,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 文档标题
        Row(
          children: [
            FlowySvg(
              FlowySvgs.document_s,
              size: const Size.square(14),
              color: theme.hintColor,
            ),
            const HSpace(6.0),
            Expanded(
              child: FlowyText(
                displayName,
                fontSize: 13,
                fontWeight: FontWeight.w500,
                color: theme.colorScheme.primary,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        
        const VSpace(4.0),
        
        // 文档类型提示
        FlowyText(
          'AppFlowy 文档',
          fontSize: 11,
          color: theme.hintColor,
        ),
      ],
    );
  }

  /// 构建MCP工具引用内容
  Widget _buildMCPToolContent(
    ChatMessageRefSource ref,
    ThemeData theme,
    bool isDark,
  ) {
    // 从source中提取服务器ID (格式: "mcp:server_id")
    String? serverId;
    if (ref.source.startsWith('mcp:')) {
      serverId = ref.source.substring(4);
      if (serverId == 'unknown') {
        serverId = null;
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 工具名称
        Row(
          children: [
            Icon(
              Icons.extension,
              size: 14,
              color: theme.colorScheme.primary,
            ),
            const HSpace(6.0),
            Expanded(
              child: RichText(
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                text: TextSpan(
                  children: [
                    if (serverId != null) ...[
                      TextSpan(
                        text: serverId,
                        style: TextStyle(
                          fontSize: 13,
                          fontWeight: FontWeight.w600,
                          color: theme.colorScheme.primary,
                        ),
                      ),
                      TextSpan(
                        text: '.',
                        style: TextStyle(
                          fontSize: 13,
                          color: isDark ? Colors.grey[400] : Colors.grey[600],
                        ),
                      ),
                    ],
                    TextSpan(
                      text: ref.name,
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                        color: isDark ? Colors.grey[200] : Colors.grey[800],
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const HSpace(4.0),
            // 成功标记
            Icon(
              Icons.check_circle,
              size: 14,
              color: Colors.green[600],
            ),
          ],
        ),
        
        const VSpace(4.0),
        
        // 工具类型提示
        FlowyText(
          'MCP 工具',
          fontSize: 11,
          color: theme.hintColor,
        ),
      ],
    );
  }

  /// 构建展开/折叠按钮
  Widget _buildExpandButton(ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.only(top: 8.0),
      child: InkWell(
        onTap: () {
          setState(() {
            _isExpanded = !_isExpanded;
          });
        },
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            FlowyText.regular(
              _isExpanded 
                  ? '收起' 
                  : '查看全部 ${widget.references.length}${_getReferenceUnit()}',
              fontSize: 12,
              color: theme.colorScheme.primary,
            ),
            const HSpace(4.0),
            Icon(
              _isExpanded 
                  ? Icons.keyboard_arrow_up 
                  : Icons.keyboard_arrow_down,
              size: 16,
              color: theme.colorScheme.primary,
            ),
          ],
        ),
      ),
    );
  }

  /// 处理引用点击事件
  Future<void> _handleReferenceTap(ChatMessageRefSource ref) async {
    // 如果有回调，先调用
    widget.onReferenceSelected?.call(ref);

    // 根据引用类型执行特定操作
    switch (widget.referenceType) {
      case ReferenceType.webSearch:
        await _openWebUrl(ref.id);
        break;
      case ReferenceType.document:
      case ReferenceType.mcpTool:
        // 文档和MCP工具的点击由回调处理
        break;
    }
  }

  /// 打开网络URL
  Future<void> _openWebUrl(String url) async {
    if (!isURL(url)) {
      Log.warn('Invalid URL in reference: $url');
      return;
    }

    try {
      final uri = Uri.parse(url);
      
      if (!uri.hasScheme) {
        Log.warn('URL missing scheme: $url');
        return;
      }

      // 检查本地主机
      if (uri.scheme == 'localhost' || uri.host == 'localhost') {
        final localUri = Uri.parse('http://$url');
        try {
          await InternetAddress.lookup(localUri.host);
          await launchUrl(localUri, mode: LaunchMode.externalApplication);
        } catch (e) {
          Log.error('Failed to resolve localhost address: $e');
        }
        return;
      }

      // 启动外部浏览器
      final launched = await launchUrl(
        uri,
        mode: LaunchMode.externalApplication,
        webOnlyWindowName: '_blank',
      );

      if (!launched) {
        Log.error('Failed to launch URL: $url');
      }
    } catch (e) {
      Log.error('Error launching URL $url: $e');
    }
  }
}

/// 引用类型枚举
enum ReferenceType {
  /// 网络搜索引用
  webSearch,
  
  /// 文档引用
  document,
  
  /// MCP工具引用
  mcpTool,
}

