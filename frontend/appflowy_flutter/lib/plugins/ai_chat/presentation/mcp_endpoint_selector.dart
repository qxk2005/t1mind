import 'package:appflowy/plugins/ai_chat/application/mcp_endpoint_service.dart';
import 'package:flutter/material.dart';

/// MCP端点选择器
/// 
/// 用于任务规划时选择可用的MCP端点，而不是具体的工具
class McpEndpointSelector extends StatefulWidget {
  const McpEndpointSelector({
    super.key,
    required this.availableEndpoints,
    required this.selectedEndpointIds,
    required this.onSelectionChanged,
    this.maxHeight = 300,
  });

  final List<McpEndpointInfo> availableEndpoints;
  final List<String> selectedEndpointIds;
  final ValueChanged<List<String>> onSelectionChanged;
  final double maxHeight;

  @override
  State<McpEndpointSelector> createState() => _McpEndpointSelectorState();
}

class _McpEndpointSelectorState extends State<McpEndpointSelector> {
  late List<String> _selectedEndpointIds;

  @override
  void initState() {
    super.initState();
    _selectedEndpointIds = List.from(widget.selectedEndpointIds);
  }

  @override
  void didUpdateWidget(McpEndpointSelector oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.selectedEndpointIds != widget.selectedEndpointIds) {
      _selectedEndpointIds = List.from(widget.selectedEndpointIds);
    }
  }

  void _onEndpointToggled(String endpointId, bool selected) {
    setState(() {
      if (selected) {
        if (!_selectedEndpointIds.contains(endpointId)) {
          _selectedEndpointIds.add(endpointId);
        }
      } else {
        _selectedEndpointIds.remove(endpointId);
      }
    });
    widget.onSelectionChanged(_selectedEndpointIds);
  }

  @override
  Widget build(BuildContext context) {
    if (widget.availableEndpoints.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Colors.grey.withOpacity(0.1),
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Column(
          children: [
            Icon(Icons.info_outline, size: 32, color: Colors.grey),
            SizedBox(height: 8),
            Text(
              '暂无可用的MCP端点',
              style: TextStyle(color: Colors.grey),
            ),
            SizedBox(height: 4),
            Text(
              'AI将根据问题需要自动选择合适的工具',
              style: TextStyle(color: Colors.grey, fontSize: 12),
            ),
          ],
        ),
      );
    }

    return Container(
      constraints: BoxConstraints(maxHeight: widget.maxHeight),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // 头部
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Colors.grey.withOpacity(0.1),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '选择MCP端点 (${_selectedEndpointIds.length}/${widget.availableEndpoints.length})',
                        style: const TextStyle(fontWeight: FontWeight.w500),
                      ),
                      const SizedBox(height: 2),
                      const Text(
                        'AI将从选中的端点中自动选择合适的工具',
                        style: TextStyle(fontSize: 11, color: Colors.grey),
                      ),
                    ],
                  ),
                ),
                TextButton(
                  onPressed: () {
                    setState(() {
                      _selectedEndpointIds.clear();
                    });
                    widget.onSelectionChanged(_selectedEndpointIds);
                  },
                  child: const Text('清空', style: TextStyle(fontSize: 12)),
                ),
                TextButton(
                  onPressed: () {
                    setState(() {
                      _selectedEndpointIds = widget.availableEndpoints.map((e) => e.id).toList();
                    });
                    widget.onSelectionChanged(_selectedEndpointIds);
                  },
                  child: const Text('全选', style: TextStyle(fontSize: 12)),
                ),
              ],
            ),
          ),
          
          // 端点列表
          Flexible(
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: widget.availableEndpoints.length,
              itemBuilder: (context, index) {
                final endpoint = widget.availableEndpoints[index];
                final isSelected = _selectedEndpointIds.contains(endpoint.id);
                
                return CheckboxListTile(
                  value: isSelected,
                  onChanged: (value) => _onEndpointToggled(endpoint.id, value ?? false),
                  title: Text(
                    endpoint.name,
                    style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w500),
                  ),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (endpoint.description.isNotEmpty)
                        Text(
                          endpoint.description,
                          style: const TextStyle(fontSize: 12),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      const SizedBox(height: 2),
                      Row(
                        children: [
                          Icon(
                            Icons.build,
                            size: 12,
                            color: Colors.grey[600],
                          ),
                          const SizedBox(width: 4),
                          Text(
                            '${endpoint.toolCount} 个工具',
                            style: TextStyle(
                              fontSize: 11,
                              color: Colors.grey[600],
                            ),
                          ),
                          if (endpoint.lastChecked != null) ...[
                            const SizedBox(width: 12),
                            Icon(
                              Icons.access_time,
                              size: 12,
                              color: Colors.grey[600],
                            ),
                            const SizedBox(width: 4),
                            Text(
                              _formatLastChecked(endpoint.lastChecked!),
                              style: TextStyle(
                                fontSize: 11,
                                color: Colors.grey[600],
                              ),
                            ),
                          ],
                        ],
                      ),
                    ],
                  ),
                  secondary: _buildStatusIcon(endpoint.isAvailable),
                  dense: true,
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusIcon(bool isAvailable) {
    return Icon(
      isAvailable ? Icons.check_circle : Icons.cancel,
      color: isAvailable ? Colors.green : Colors.red,
      size: 20,
    );
  }

  String _formatLastChecked(DateTime lastChecked) {
    final now = DateTime.now();
    final difference = now.difference(lastChecked);
    
    if (difference.inMinutes < 1) {
      return '刚刚检查';
    } else if (difference.inMinutes < 60) {
      return '${difference.inMinutes}分钟前';
    } else if (difference.inHours < 24) {
      return '${difference.inHours}小时前';
    } else {
      return '${difference.inDays}天前';
    }
  }
}
