import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/plugins/ai_chat/widgets/citation_display.dart';
import 'package:flutter/material.dart';
import 'package:string_validator/string_validator.dart';

/// 引用显示调试工具
/// 
/// 这个工具帮助调试引用显示组件的问题，包括：
/// 1. 检查网络搜索功能是否启用
/// 2. 检查AI消息中是否有网络搜索数据
/// 3. 提供测试数据来验证引用显示组件
class CitationDebugTool extends StatefulWidget {
  const CitationDebugTool({super.key});

  @override
  State<CitationDebugTool> createState() => _CitationDebugToolState();
}

class _CitationDebugToolState extends State<CitationDebugTool> {
  List<ChatMessageRefSource> _testSources = [];
  bool _showDebugInfo = true;

  @override
  void initState() {
    super.initState();
    _generateTestData();
  }

  void _generateTestData() {
    setState(() {
      _testSources = [
        // 网络搜索引用
        ChatMessageRefSource(
          id: "https://docs.flutter.dev/development/data-and-backend/state-mgmt",
          name: "Flutter官方文档 - 状态管理",
          source: "web",
        ),
        ChatMessageRefSource(
          id: "https://dart.dev/guides/language/language-tour#asynchrony-support",
          name: "Dart语言指南 - 异步编程",
          source: "web",
        ),
        ChatMessageRefSource(
          id: "https://docs.appflowy.io/docs/architecture/overview",
          name: "AppFlowy架构设计文档",
          source: "web",
        ),
        // 文档来源
        ChatMessageRefSource(
          id: "page-123",
          name: "项目规划文档",
          source: "appflowy",
        ),
        ChatMessageRefSource(
          id: "page-456",
          name: "技术规范文档",
          source: "appflowy",
        ),
      ];
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('引用显示调试工具'),
        actions: [
          IconButton(
            icon: Icon(_showDebugInfo ? Icons.visibility_off : Icons.visibility),
            onPressed: () {
              setState(() {
                _showDebugInfo = !_showDebugInfo;
              });
            },
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 调试信息
            if (_showDebugInfo) ...[
              _buildDebugInfo(),
              const SizedBox(height: 24),
            ],
            
            // 测试数据生成
            _buildTestDataSection(),
            const SizedBox(height: 24),
            
            // 引用显示测试
            _buildCitationDisplayTest(),
            const SizedBox(height: 24),
            
            // 问题排查指南
            _buildTroubleshootingGuide(),
          ],
        ),
      ),
    );
  }

  Widget _buildDebugInfo() {
    final webSources = _testSources.where((s) => s.source == 'web' && isURL(s.id)).toList();
    final docSources = _testSources.where((s) => s.source == 'appflowy' || (s.source == 'web' && !isURL(s.id))).toList();
    
    return Container(
      padding: const EdgeInsets.all(16.0),
      decoration: BoxDecoration(
        color: Colors.blue.shade50,
        border: Border.all(color: Colors.blue.shade200),
        borderRadius: BorderRadius.circular(8.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '🔍 调试信息',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.bold,
              color: Colors.blue.shade700,
            ),
          ),
          const SizedBox(height: 8),
          Text('总引用源数量: ${_testSources.length}'),
          Text('网络搜索引用: ${webSources.length}'),
          Text('文档来源: ${docSources.length}'),
          const SizedBox(height: 8),
          Text(
            '网络搜索引用详情:',
            style: TextStyle(fontWeight: FontWeight.w500),
          ),
          ...webSources.map((source) => Padding(
            padding: const EdgeInsets.only(left: 8.0),
            child: Text('• ${source.name} (${source.id})'),
          )),
        ],
      ),
    );
  }

  Widget _buildTestDataSection() {
    return Container(
      padding: const EdgeInsets.all(16.0),
      decoration: BoxDecoration(
        color: Colors.green.shade50,
        border: Border.all(color: Colors.green.shade200),
        borderRadius: BorderRadius.circular(8.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '🧪 测试数据',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.bold,
              color: Colors.green.shade700,
            ),
          ),
          const SizedBox(height: 8),
          Text('当前有 ${_testSources.length} 个测试引用源'),
          const SizedBox(height: 8),
          Row(
            children: [
              ElevatedButton(
                onPressed: _generateTestData,
                child: const Text('重新生成测试数据'),
              ),
              const SizedBox(width: 8),
              ElevatedButton(
                onPressed: () {
                  setState(() {
                    _testSources = [];
                  });
                },
                child: const Text('清空测试数据'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildCitationDisplayTest() {
    final webSources = _testSources.where((s) => s.source == 'web' && isURL(s.id)).toList();
    final citations = webSources.map((source) => CitationInfo(
      index: webSources.indexOf(source) + 1,
      title: source.name,
      url: source.id,
      domain: Uri.parse(source.id).host,
      snippet: '这是来自 ${Uri.parse(source.id).host} 的搜索结果摘要。',
      relevanceScore: 0.9 - (webSources.indexOf(source) * 0.1),
    )).toList();

    return Container(
      padding: const EdgeInsets.all(16.0),
      decoration: BoxDecoration(
        color: Colors.orange.shade50,
        border: Border.all(color: Colors.orange.shade200),
        borderRadius: BorderRadius.circular(8.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '📋 引用显示测试',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.bold,
              color: Colors.orange.shade700,
            ),
          ),
          const SizedBox(height: 8),
          Text('网络搜索引用数量: ${citations.length}'),
          const SizedBox(height: 16),
          if (citations.isNotEmpty) ...[
            CitationDisplay(
              citations: citations,
              maxVisibleCitations: 2,
              onCitationTap: (citation) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('点击了引用: ${citation.title}')),
                );
              },
            ),
          ] else ...[
            Container(
              padding: const EdgeInsets.all(16.0),
              decoration: BoxDecoration(
                color: Colors.grey.shade100,
                borderRadius: BorderRadius.circular(8.0),
              ),
              child: const Text('没有网络搜索引用数据'),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildTroubleshootingGuide() {
    return Container(
      padding: const EdgeInsets.all(16.0),
      decoration: BoxDecoration(
        color: Colors.red.shade50,
        border: Border.all(color: Colors.red.shade200),
        borderRadius: BorderRadius.circular(8.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '🔧 问题排查指南',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.bold,
              color: Colors.red.shade700,
            ),
          ),
          const SizedBox(height: 8),
          const Text('如果引用组件没有显示，请检查：'),
          const SizedBox(height: 8),
          const Text('1. 网络搜索功能是否已启用'),
          const Text('   - 进入设置 → 网络搜索配置'),
          const Text('   - 确保"启用网络搜索"开关已打开'),
          const SizedBox(height: 4),
          const Text('2. 是否配置了搜索引擎供应商'),
          const Text('   - 添加Tavily或Brave Search供应商'),
          const Text('   - 配置有效的API密钥'),
          const SizedBox(height: 4),
          const Text('3. AI聊天时是否启用了网络搜索'),
          const Text('   - 确保AI模型支持工具调用'),
          const Text('   - 检查AI响应中是否包含网络搜索数据'),
          const SizedBox(height: 4),
          const Text('4. 数据格式是否正确'),
          const Text('   - 检查ChatMessageRefSource的source字段'),
          const Text('   - 确保网络搜索引用的id是有效URL'),
        ],
      ),
    );
  }
}
