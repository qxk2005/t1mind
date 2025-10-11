import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/plugins/ai_chat/widgets/mcp_reference_display.dart';
import 'package:flutter/material.dart';

/// MCP引用显示组件使用示例
/// 
/// 本文件展示如何在不同场景下使用MCPReferenceDisplay组件
class MCPReferenceDisplayExample extends StatelessWidget {
  const MCPReferenceDisplayExample({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('MCP引用显示示例'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildSection(
              title: '示例 1: 单个MCP工具引用',
              child: MCPReferenceDisplay(
                references: [
                  ChatMessageRefSource(
                    id: 'call_001',
                    name: 'read_data_from_excel',
                    source: 'mcp:excel',
                  ),
                ],
                showExpandButton: false,
                onReferenceSelected: (ref) {
                  _showReferenceDetails(context, ref);
                },
              ),
            ),
            const SizedBox(height: 24),
            
            _buildSection(
              title: '示例 2: 多个MCP工具引用（展开/折叠）',
              child: MCPReferenceDisplay(
                references: [
                  ChatMessageRefSource(
                    id: 'call_002',
                    name: 'read_data_from_excel',
                    source: 'mcp:excel',
                  ),
                  ChatMessageRefSource(
                    id: 'call_003',
                    name: 'create_chart',
                    source: 'mcp:excel',
                  ),
                  ChatMessageRefSource(
                    id: 'call_004',
                    name: 'read_file',
                    source: 'mcp:filesystem',
                  ),
                  ChatMessageRefSource(
                    id: 'call_005',
                    name: 'write_file',
                    source: 'mcp:filesystem',
                  ),
                  ChatMessageRefSource(
                    id: 'call_006',
                    name: 'get_highlights',
                    source: 'mcp:readwise',
                  ),
                ],
                showExpandButton: true,
                onReferenceSelected: (ref) {
                  _showReferenceDetails(context, ref);
                },
              ),
            ),
            const SizedBox(height: 24),
            
            _buildSection(
              title: '示例 3: 未知服务器的工具',
              child: MCPReferenceDisplay(
                references: [
                  ChatMessageRefSource(
                    id: 'call_007',
                    name: 'some_custom_tool',
                    source: 'mcp:unknown',
                  ),
                ],
                showExpandButton: false,
                onReferenceSelected: (ref) {
                  _showReferenceDetails(context, ref);
                },
              ),
            ),
            const SizedBox(height: 24),
            
            _buildSection(
              title: '示例 4: 不带点击回调',
              description: '引用项不可点击',
              child: MCPReferenceDisplay(
                references: [
                  ChatMessageRefSource(
                    id: 'call_008',
                    name: 'read_data',
                    source: 'mcp:database',
                  ),
                ],
                showExpandButton: false,
                // 不提供onReferenceSelected，引用项不可点击
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSection({
    required String title,
    String? description,
    required Widget child,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: const TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
          ),
        ),
        if (description != null) ...[
          const SizedBox(height: 4),
          Text(
            description,
            style: const TextStyle(
              fontSize: 14,
              color: Colors.grey,
            ),
          ),
        ],
        const SizedBox(height: 8),
        child,
      ],
    );
  }

  void _showReferenceDetails(BuildContext context, ChatMessageRefSource ref) {
    // 提取服务器ID
    String? serverId;
    if (ref.source.startsWith('mcp:')) {
      serverId = ref.source.substring(4);
      if (serverId == 'unknown') {
        serverId = null;
      }
    }

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('MCP工具引用详情'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildDetailRow('工具名称', ref.name),
            _buildDetailRow('调用ID', ref.id),
            _buildDetailRow('服务器', serverId ?? '未知'),
            _buildDetailRow('完整source', ref.source),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('关闭'),
          ),
        ],
      ),
    );
  }

  Widget _buildDetailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              '$label:',
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(fontFamily: 'monospace'),
            ),
          ),
        ],
      ),
    );
  }
}

/// 用于快速测试的简化版本
class QuickMCPReferenceTest extends StatelessWidget {
  const QuickMCPReferenceTest({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('MCP引用快速测试')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: MCPReferenceDisplay(
            references: _generateTestReferences(),
            showExpandButton: true,
            onReferenceSelected: (ref) {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text('点击了: ${ref.name}'),
                  duration: const Duration(seconds: 1),
                ),
              );
            },
          ),
        ),
      ),
    );
  }

  List<ChatMessageRefSource> _generateTestReferences() {
    return [
      ChatMessageRefSource(
        id: 'test_001',
        name: 'read_data_from_excel',
        source: 'mcp:excel',
      ),
      ChatMessageRefSource(
        id: 'test_002',
        name: 'write_data_to_excel',
        source: 'mcp:excel',
      ),
      ChatMessageRefSource(
        id: 'test_003',
        name: 'create_chart',
        source: 'mcp:excel',
      ),
      ChatMessageRefSource(
        id: 'test_004',
        name: 'read_file',
        source: 'mcp:filesystem',
      ),
      ChatMessageRefSource(
        id: 'test_005',
        name: 'search_highlights',
        source: 'mcp:readwise',
      ),
    ];
  }
}

