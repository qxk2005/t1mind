import 'package:appflowy/plugins/ai_chat/widgets/citation_display.dart';
import 'package:appflowy/plugins/ai_chat/presentation/message/ai_metadata.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:flutter/material.dart';

/// 引用显示组件集成示例
/// 
/// 展示如何在AI消息中集成网络搜索结果引用显示组件。
/// 这个文件提供了集成示例和最佳实践。
class CitationDisplayIntegrationExample {
  
  /// 示例：在AI消息中集成引用显示组件
  /// 
  /// 这个示例展示了如何在现有的AI消息显示中添加网络搜索结果引用。
  /// 引用显示组件应该在消息内容之后、现有元数据之前显示。
  static Widget buildAIMessageWithCitations({
    required String messageText,
    required List<CitationInfo> webSearchCitations,
    required List<ChatMessageRefSource> documentSources,
    required void Function(ChatMessageRefSource)? onSelectedMetadata,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 消息内容
        Padding(
          padding: const EdgeInsetsDirectional.only(start: 4.0),
          child: Text(messageText),
        ),
        
        // 网络搜索结果引用（新增）
        if (webSearchCitations.isNotEmpty)
          Padding(
            padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
            child: CitationDisplay(
              citations: webSearchCitations,
              maxVisibleCitations: 3,
              showExpandButton: true,
            ),
          ),
        
        // 现有文档来源元数据
        if (documentSources.isNotEmpty)
          Padding(
            padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
            child: AIMessageMetadata(
              sources: documentSources,
              onSelectedMetadata: onSelectedMetadata,
            ),
          ),
      ],
    );
  }

  /// 示例：从ChatMessageRefSource创建CitationInfo
  /// 
  /// 展示如何将现有的ChatMessageRefSource转换为CitationInfo，
  /// 以便在网络搜索场景中使用引用显示组件。
  static List<CitationInfo> convertRefSourcesToCitations(
    List<ChatMessageRefSource> refSources,
  ) {
    return refSources
        .where((ref) => ref.source == 'web')
        .map((ref) => CitationInfo.fromRefSource(ref))
        .toList();
  }

  /// 示例：从网络搜索结果数据创建CitationInfo
  /// 
  /// 展示如何从Rust后端返回的网络搜索结果数据创建CitationInfo列表。
  static List<CitationInfo> createCitationsFromWebSearchResults(
    List<Map<String, dynamic>> searchResults,
  ) {
    return searchResults
        .map((result) => CitationInfo.fromMap(result))
        .toList();
  }

  /// 示例：混合显示网络搜索引用和文档来源
  /// 
  /// 展示如何在同一个AI消息中同时显示网络搜索结果引用和文档来源，
  /// 提供清晰的视觉区分。
  static Widget buildMixedSourcesDisplay({
    required List<CitationInfo> webSearchCitations,
    required List<ChatMessageRefSource> documentSources,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 网络搜索结果引用
        if (webSearchCitations.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsetsDirectional.only(start: 4.0, top: 8.0),
            child: CitationDisplay(
              citations: webSearchCitations,
              maxVisibleCitations: 5,
              showExpandButton: true,
            ),
          ),
          const SizedBox(height: 8.0),
        ],
        
        // 文档来源
        if (documentSources.isNotEmpty)
          Padding(
            padding: const EdgeInsetsDirectional.only(start: 4.0),
            child: AIMessageMetadata(
              sources: documentSources,
              onSelectedMetadata: null, // 根据需要设置回调
            ),
          ),
      ],
    );
  }
}

/// 示例数据生成器
/// 
/// 提供用于测试和演示的示例数据。
class CitationDisplayExampleData {
  
  /// 生成示例网络搜索结果引用
  static List<CitationInfo> generateSampleWebSearchCitations() {
    return [
      CitationInfo(
        title: "Flutter官方文档 - 状态管理",
        url: "https://docs.flutter.dev/development/data-and-backend/state-mgmt",
        domain: "docs.flutter.dev",
        snippet: "了解Flutter中的状态管理最佳实践，包括Provider、Bloc、Riverpod等状态管理解决方案。",
        relevanceScore: 0.95,
        index: 1,
      ),
      CitationInfo(
        title: "Dart语言指南 - 异步编程",
        url: "https://dart.dev/guides/language/language-tour#asynchrony-support",
        domain: "dart.dev",
        snippet: "Dart提供了强大的异步编程支持，包括Future、Stream和async/await语法。",
        relevanceScore: 0.88,
        index: 2,
      ),
      CitationInfo(
        title: "AppFlowy架构设计文档",
        url: "https://docs.appflowy.io/docs/architecture/overview",
        domain: "docs.appflowy.io",
        snippet: "AppFlowy采用Flutter前端和Rust后端的架构设计，提供高性能的协作体验。",
        relevanceScore: 0.92,
        index: 3,
      ),
      CitationInfo(
        title: "网络搜索API集成最佳实践",
        url: "https://example.com/web-search-integration",
        domain: "example.com",
        snippet: "学习如何安全地集成网络搜索API，包括错误处理、缓存策略和用户体验优化。",
        relevanceScore: 0.85,
        index: 4,
      ),
      CitationInfo(
        title: "Flutter UI组件设计指南",
        url: "https://flutter.dev/docs/development/ui/widgets",
        domain: "flutter.dev",
        snippet: "Flutter提供了丰富的UI组件库，帮助开发者快速构建美观的用户界面。",
        relevanceScore: 0.78,
        index: 5,
      ),
    ];
  }

  /// 生成示例文档来源
  static List<ChatMessageRefSource> generateSampleDocumentSources() {
    return [
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
  }

  /// 生成示例网络搜索引用（从ChatMessageRefSource格式）
  static List<ChatMessageRefSource> generateSampleWebRefSources() {
    return [
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
    ];
  }
}

/// 集成指南和最佳实践
/// 
/// 提供在AI消息中集成引用显示组件的详细指南。
class CitationDisplayIntegrationGuide {
  
  /// 集成步骤
  /// 
  /// 1. 在AI消息组件中导入引用显示组件
  /// 2. 在消息内容之后添加引用显示
  /// 3. 处理网络搜索结果的点击事件
  /// 4. 确保与现有元数据显示的一致性
  
  /// 最佳实践
  /// 
  /// 1. **视觉层次**: 网络搜索引用应该与文档来源有清晰的视觉区分
  /// 2. **性能优化**: 对于大量引用，使用分页或折叠显示
  /// 3. **安全性**: 确保所有链接都经过适当的验证和清理
  /// 4. **可访问性**: 提供适当的语义标签和键盘导航支持
  /// 5. **响应式设计**: 确保在不同屏幕尺寸下都能良好显示
  
  /// 注意事项
  /// 
  /// 1. 网络搜索引用和文档来源应该分别显示，避免混淆
  /// 2. 引用显示组件会自动处理URL验证和安全性检查
  /// 3. 组件支持展开/折叠功能，适合处理大量引用
  /// 4. 引用按相关性评分排序，最重要的结果优先显示
}
