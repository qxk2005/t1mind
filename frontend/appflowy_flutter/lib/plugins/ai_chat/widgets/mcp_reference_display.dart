import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';

/// MCP工具引用显示组件
/// 
/// 用于在AI聊天消息中显示MCP工具调用的引用信息，
/// 包括工具名称、调用状态等。
class MCPReferenceDisplay extends StatefulWidget {
  const MCPReferenceDisplay({
    super.key,
    required this.references,
    this.showExpandButton = true,
    this.onReferenceSelected,
  });

  /// MCP引用列表
  final List<ChatMessageRefSource> references;
  
  /// 是否显示展开/折叠按钮（当引用数量较多时）
  final bool showExpandButton;
  
  /// 引用被点击时的回调（可选）
  final void Function(ChatMessageRefSource reference)? onReferenceSelected;

  @override
  State<MCPReferenceDisplay> createState() => _MCPReferenceDisplayState();
}

class _MCPReferenceDisplayState extends State<MCPReferenceDisplay> {
  bool _isExpanded = false;
  
  @override
  Widget build(BuildContext context) {
    if (widget.references.isEmpty) {
      return const SizedBox.shrink();
    }

    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    
    // 默认显示前3个引用
    final maxVisible = _isExpanded ? widget.references.length : 3;
    final visibleReferences = widget.references.take(maxVisible).toList();
    final hasMore = widget.references.length > 3;

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
          Row(
            children: [
              Icon(
                Icons.extension_outlined,
                size: 16,
                color: theme.colorScheme.primary,
              ),
              const HSpace(8.0),
              FlowyText.medium(
                'MCP工具调用',
                fontSize: 13,
                color: isDark ? Colors.grey[300] : Colors.grey[700],
              ),
              const Spacer(),
              FlowyText.regular(
                '${widget.references.length}个工具',
                fontSize: 12,
                color: isDark ? Colors.grey[400] : Colors.grey[600],
              ),
            ],
          ),
          const VSpace(8.0),
          
          // 引用列表
          ...visibleReferences.asMap().entries.map((entry) {
            final index = entry.key;
            final ref = entry.value;
            return Padding(
              padding: EdgeInsets.only(
                top: index > 0 ? 6.0 : 0,
              ),
              child: _buildMCPReferenceItem(ref, index + 1, isDark),
            );
          }),
          
          // 展开/折叠按钮
          if (hasMore && widget.showExpandButton)
            Padding(
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
                          : '查看全部 ${widget.references.length} 个工具',
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
            ),
        ],
      ),
    );
  }

  /// 构建单个MCP引用项
  Widget _buildMCPReferenceItem(
    ChatMessageRefSource ref, 
    int index, 
    bool isDark,
  ) {
    final theme = Theme.of(context);
    
    // 从source中提取服务器ID (格式: "mcp:server_id")
    String? serverId;
    if (ref.source.startsWith('mcp:')) {
      serverId = ref.source.substring(4);
      if (serverId == 'unknown') {
        serverId = null;
      }
    }
    
    return InkWell(
      onTap: widget.onReferenceSelected != null
          ? () => widget.onReferenceSelected!(ref)
          : null,
      borderRadius: BorderRadius.circular(6.0),
      child: Container(
        padding: const EdgeInsets.symmetric(
          horizontal: 8.0,
          vertical: 6.0,
        ),
        decoration: BoxDecoration(
          color: isDark 
              ? Colors.grey[800]?.withOpacity(0.5) 
              : Colors.white,
          borderRadius: BorderRadius.circular(6.0),
          border: Border.all(
            color: isDark 
                ? Colors.grey[700]! 
                : Colors.grey[200]!,
            width: 0.5,
          ),
        ),
        child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 序号
          Container(
            width: 20,
            height: 20,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: theme.colorScheme.primary.withOpacity(0.1),
              borderRadius: BorderRadius.circular(4.0),
            ),
            child: FlowyText.regular(
              '$index',
              fontSize: 11,
              color: theme.colorScheme.primary,
            ),
          ),
          const HSpace(8.0),
          
          // 工具信息
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 工具名称（格式: 服务器名称.工具名称）
                Row(
                  children: [
                    Icon(
                      Icons.extension,
                      size: 14,
                      color: theme.colorScheme.primary,
                    ),
                    const HSpace(4.0),
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
                                  fontSize: 12,
                                  fontWeight: FontWeight.w600,
                                  color: theme.colorScheme.primary,
                                ),
                              ),
                              TextSpan(
                                text: '.',
                                style: TextStyle(
                                  fontSize: 12,
                                  color: isDark ? Colors.grey[400] : Colors.grey[600],
                                ),
                              ),
                            ],
                            TextSpan(
                              text: ref.name,
                              style: TextStyle(
                                fontSize: 12,
                                fontWeight: FontWeight.w500,
                                color: isDark ? Colors.grey[200] : Colors.grey[800],
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          
          // 成功标识
          const HSpace(8.0),
          Container(
            padding: const EdgeInsets.symmetric(
              horizontal: 6.0,
              vertical: 2.0,
            ),
            decoration: BoxDecoration(
              color: Colors.green.withOpacity(0.1),
              borderRadius: BorderRadius.circular(4.0),
            ),
            child: const Icon(
              Icons.check,
              size: 12,
              color: Colors.green,
            ),
          ),
        ],
      ),
      ),
    );
  }
}

