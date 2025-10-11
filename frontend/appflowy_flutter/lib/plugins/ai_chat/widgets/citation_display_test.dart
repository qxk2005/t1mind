import 'package:appflowy/plugins/ai_chat/widgets/citation_display.dart';
import 'package:flutter/material.dart';

/// 引用显示组件测试页面
/// 
/// 用于测试和演示引用显示组件的功能。
/// 可以在开发过程中使用此页面来验证组件的正确性。
class CitationDisplayTestPage extends StatelessWidget {
  const CitationDisplayTestPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('引用显示组件测试'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              '网络搜索结果引用显示测试',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 16),
            
            // 测试1: 基本引用显示
            _buildTestSection(
              title: '测试1: 基本引用显示',
              description: '显示3个网络搜索结果引用',
              child: CitationDisplay(
                citations: _generateSampleCitations(3),
                maxVisibleCitations: 3,
                showExpandButton: false,
              ),
            ),
            
            const SizedBox(height: 24),
            
            // 测试2: 可展开引用显示
            _buildTestSection(
              title: '测试2: 可展开引用显示',
              description: '显示5个引用，默认显示3个，可展开显示更多',
              child: CitationDisplay(
                citations: _generateSampleCitations(5),
                maxVisibleCitations: 3,
                showExpandButton: true,
              ),
            ),
            
            const SizedBox(height: 24),
            
            // 测试3: 大量引用显示
            _buildTestSection(
              title: '测试3: 大量引用显示',
              description: '显示10个引用，测试性能和滚动',
              child: CitationDisplay(
                citations: _generateSampleCitations(10),
                maxVisibleCitations: 5,
                showExpandButton: true,
              ),
            ),
            
            const SizedBox(height: 24),
            
            // 测试4: 空引用列表
            _buildTestSection(
              title: '测试4: 空引用列表',
              description: '测试空引用列表的处理',
              child: const CitationDisplay(
                citations: [],
                maxVisibleCitations: 3,
                showExpandButton: true,
              ),
            ),
            
            const SizedBox(height: 24),
            
            // 测试5: 单个引用
            _buildTestSection(
              title: '测试5: 单个引用',
              description: '测试单个引用的显示',
              child: CitationDisplay(
                citations: _generateSampleCitations(1),
                maxVisibleCitations: 3,
                showExpandButton: true,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTestSection({
    required String title,
    required String description,
    required Widget child,
  }) {
    return Container(
      padding: const EdgeInsets.all(16.0),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.shade300),
        borderRadius: BorderRadius.circular(8.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            description,
            style: TextStyle(
              fontSize: 14,
              color: Colors.grey.shade600,
            ),
          ),
          const SizedBox(height: 16),
          child,
        ],
      ),
    );
  }

  List<CitationInfo> _generateSampleCitations(int count) {
    final samples = [
      {
        'title': 'Flutter官方文档 - 状态管理',
        'url': 'https://docs.flutter.dev/development/data-and-backend/state-mgmt',
        'domain': 'docs.flutter.dev',
        'snippet': '了解Flutter中的状态管理最佳实践，包括Provider、Bloc、Riverpod等状态管理解决方案。',
        'relevanceScore': 0.95,
      },
      {
        'title': 'Dart语言指南 - 异步编程',
        'url': 'https://dart.dev/guides/language/language-tour#asynchrony-support',
        'domain': 'dart.dev',
        'snippet': 'Dart提供了强大的异步编程支持，包括Future、Stream和async/await语法。',
        'relevanceScore': 0.88,
      },
      {
        'title': 'AppFlowy架构设计文档',
        'url': 'https://docs.appflowy.io/docs/architecture/overview',
        'domain': 'docs.appflowy.io',
        'snippet': 'AppFlowy采用Flutter前端和Rust后端的架构设计，提供高性能的协作体验。',
        'relevanceScore': 0.92,
      },
      {
        'title': '网络搜索API集成最佳实践',
        'url': 'https://example.com/web-search-integration',
        'domain': 'example.com',
        'snippet': '学习如何安全地集成网络搜索API，包括错误处理、缓存策略和用户体验优化。',
        'relevanceScore': 0.85,
      },
      {
        'title': 'Flutter UI组件设计指南',
        'url': 'https://flutter.dev/docs/development/ui/widgets',
        'domain': 'flutter.dev',
        'snippet': 'Flutter提供了丰富的UI组件库，帮助开发者快速构建美观的用户界面。',
        'relevanceScore': 0.78,
      },
      {
        'title': 'Rust编程语言官方文档',
        'url': 'https://doc.rust-lang.org/book/',
        'domain': 'doc.rust-lang.org',
        'snippet': 'Rust是一种系统编程语言，专注于安全性、速度和并发性。',
        'relevanceScore': 0.82,
      },
      {
        'title': 'HTTP客户端库比较',
        'url': 'https://example.com/http-client-comparison',
        'domain': 'example.com',
        'snippet': '比较不同编程语言中的HTTP客户端库，包括性能、易用性和功能特性。',
        'relevanceScore': 0.75,
      },
      {
        'title': '数据库设计最佳实践',
        'url': 'https://example.com/database-design',
        'domain': 'example.com',
        'snippet': '学习如何设计高效、可扩展的数据库架构，包括索引优化和查询性能。',
        'relevanceScore': 0.80,
      },
      {
        'title': '微服务架构模式',
        'url': 'https://example.com/microservices-patterns',
        'domain': 'example.com',
        'snippet': '探索微服务架构的各种设计模式，包括服务发现、负载均衡和容错处理。',
        'relevanceScore': 0.77,
      },
      {
        'title': '容器化部署指南',
        'url': 'https://example.com/container-deployment',
        'domain': 'example.com',
        'snippet': '使用Docker和Kubernetes进行应用程序的容器化部署和管理。',
        'relevanceScore': 0.73,
      },
    ];

    return samples
        .take(count)
        .map((sample) => CitationInfo(
              title: sample['title'] as String,
              url: sample['url'] as String,
              domain: sample['domain'] as String,
              snippet: sample['snippet'] as String,
              relevanceScore: sample['relevanceScore'] as double,
              index: samples.indexOf(sample) + 1,
            ))
        .toList();
  }
}

/// 测试页面入口
class CitationDisplayTestApp extends StatelessWidget {
  const CitationDisplayTestApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '引用显示组件测试',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const CitationDisplayTestPage(),
    );
  }
}
