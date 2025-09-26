import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart' as log_entities;
import 'package:flutter/material.dart';

/// 简化版MCP工具选择器
/// 
/// 用于解决复杂版本可能导致的界面卡死问题
class SimpleMcpToolSelector extends StatefulWidget {
  const SimpleMcpToolSelector({
    super.key,
    required this.availableTools,
    required this.selectedToolIds,
    required this.onSelectionChanged,
    this.maxHeight = 300,
  });

  final List<log_entities.McpToolInfo> availableTools;
  final List<String> selectedToolIds;
  final ValueChanged<List<String>> onSelectionChanged;
  final double maxHeight;

  @override
  State<SimpleMcpToolSelector> createState() => _SimpleMcpToolSelectorState();
}

class _SimpleMcpToolSelectorState extends State<SimpleMcpToolSelector> {
  late List<String> _selectedToolIds;

  @override
  void initState() {
    super.initState();
    _selectedToolIds = List.from(widget.selectedToolIds);
  }

  @override
  void didUpdateWidget(SimpleMcpToolSelector oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.selectedToolIds != widget.selectedToolIds) {
      _selectedToolIds = List.from(widget.selectedToolIds);
    }
  }

  void _onToolToggled(String toolId, bool selected) {
    setState(() {
      if (selected) {
        if (!_selectedToolIds.contains(toolId)) {
          _selectedToolIds.add(toolId);
        }
      } else {
        _selectedToolIds.remove(toolId);
      }
    });
    widget.onSelectionChanged(_selectedToolIds);
  }

  @override
  Widget build(BuildContext context) {
    if (widget.availableTools.isEmpty) {
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
              '暂无可用的MCP工具',
              style: TextStyle(color: Colors.grey),
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
                  child: Text(
                    '选择MCP工具 (${_selectedToolIds.length}/${widget.availableTools.length})',
                    style: const TextStyle(fontWeight: FontWeight.w500),
                  ),
                ),
                TextButton(
                  onPressed: () {
                    setState(() {
                      _selectedToolIds.clear();
                    });
                    widget.onSelectionChanged(_selectedToolIds);
                  },
                  child: const Text('清空', style: TextStyle(fontSize: 12)),
                ),
                TextButton(
                  onPressed: () {
                    setState(() {
                      _selectedToolIds = widget.availableTools.map((t) => t.id).toList();
                    });
                    widget.onSelectionChanged(_selectedToolIds);
                  },
                  child: const Text('全选', style: TextStyle(fontSize: 12)),
                ),
              ],
            ),
          ),
          
          // 工具列表
          Flexible(
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: widget.availableTools.length,
              itemBuilder: (context, index) {
                final tool = widget.availableTools[index];
                final isSelected = _selectedToolIds.contains(tool.id);
                
                return CheckboxListTile(
                  value: isSelected,
                  onChanged: (value) => _onToolToggled(tool.id, value ?? false),
                  title: Text(
                    tool.displayName ?? tool.name,
                    style: const TextStyle(fontSize: 14),
                  ),
                  subtitle: tool.description.isNotEmpty
                      ? Text(
                          tool.description,
                          style: const TextStyle(fontSize: 12),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        )
                      : null,
                  secondary: _buildStatusIcon(tool.status),
                  dense: true,
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusIcon(log_entities.McpToolStatus status) {
    Color color;
    IconData icon;
    
    switch (status) {
      case log_entities.McpToolStatus.available:
      case log_entities.McpToolStatus.connected:
        color = Colors.green;
        icon = Icons.check_circle;
        break;
      case log_entities.McpToolStatus.unavailable:
      case log_entities.McpToolStatus.disconnected:
        color = Colors.red;
        icon = Icons.cancel;
        break;
      case log_entities.McpToolStatus.error:
        color = Colors.orange;
        icon = Icons.error;
        break;
      case log_entities.McpToolStatus.connecting:
        color = Colors.blue;
        icon = Icons.sync;
        break;
      default:
        color = Colors.grey;
        icon = Icons.help;
    }
    
    return Icon(icon, color: color, size: 20);
  }
}
