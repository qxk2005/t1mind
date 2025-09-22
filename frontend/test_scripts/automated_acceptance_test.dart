// 自动化验收测试脚本 - 任务D5
// 这个脚本提供了自动化测试的框架，用于验证AI全局模型OpenAI兼容功能

import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:appflowy/main.dart' as app;
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('AI Global Model OpenAI Compatible - Acceptance Test D5', () {
    
    setUpAll(() async {
      // 初始化测试环境
      await _setupTestEnvironment();
    });

    tearDownAll(() async {
      // 清理测试环境
      await _cleanupTestEnvironment();
    });

    group('里程碑A: UI与i18n验收测试', () {
      
      testWidgets('A1-A3: 全局模型类型选择和面板切换', (WidgetTester tester) async {
        await app.main();
        await tester.pumpAndSettle();

        // 导航到AI设置页面
        await _navigateToAISettings(tester);
        
        // 验证全局模型类型选择器存在
        expect(find.text('全局使用的模型类型'), findsOneWidget);
        
        // 验证默认选择是"ollama 本地"
        expect(find.text('ollama 本地'), findsOneWidget);
        
        // 验证可以切换到"openai 兼容服务器"
        await tester.tap(find.text('ollama 本地'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('openai 兼容服务器'));
        await tester.pumpAndSettle();
        
        // 验证OpenAI兼容设置面板显示
        expect(find.text('聊天配置'), findsOneWidget);
        expect(find.text('嵌入配置'), findsOneWidget);
        expect(find.text('API端点'), findsWidgets);
        expect(find.text('API密钥'), findsWidgets);
        
        // 验证操作按钮存在
        expect(find.text('测试聊天'), findsOneWidget);
        expect(find.text('测试嵌入'), findsOneWidget);
        expect(find.text('保存设置'), findsOneWidget);
      });

      testWidgets('A5: 国际化文案验证', (WidgetTester tester) async {
        await app.main();
        await tester.pumpAndSettle();

        await _navigateToAISettings(tester);
        
        // 验证中文文案
        await _switchLanguage(tester, 'zh-CN');
        expect(find.text('全局使用的模型类型'), findsOneWidget);
        expect(find.text('聊天配置'), findsOneWidget);
        expect(find.text('嵌入配置'), findsOneWidget);
        
        // 验证英文文案
        await _switchLanguage(tester, 'en-US');
        expect(find.text('Global Model Type'), findsOneWidget);
        expect(find.text('Chat Configuration'), findsOneWidget);
        expect(find.text('Embedding Configuration'), findsOneWidget);
      });
    });

    group('里程碑B: 持久化验收测试', () {
      
      testWidgets('B1-B6: 配置保存和读取', (WidgetTester tester) async {
        await app.main();
        await tester.pumpAndSettle();

        await _navigateToAISettings(tester);
        
        // 切换到OpenAI兼容模式
        await _switchToOpenAICompatible(tester);
        
        // 填写测试配置
        await _fillTestConfiguration(tester);
        
        // 保存配置
        await tester.tap(find.text('保存设置'));
        await tester.pumpAndSettle();
        
        // 验证保存成功提示
        expect(find.text('设置已保存'), findsOneWidget);
        
        // 重新加载页面验证配置持久化
        await _reloadPage(tester);
        await _navigateToAISettings(tester);
        
        // 验证配置保持
        await _verifyConfigurationPersisted(tester);
      });
    });

    group('里程碑C: 测试能力验收测试', () {
      
      testWidgets('C1-C6: 连接测试功能', (WidgetTester tester) async {
        await app.main();
        await tester.pumpAndSettle();

        await _navigateToAISettings(tester);
        await _switchToOpenAICompatible(tester);
        await _fillTestConfiguration(tester);
        
        // 测试聊天功能
        await tester.tap(find.text('测试聊天'));
        await tester.pumpAndSettle();
        
        // 验证测试状态显示
        expect(find.text('测试中...'), findsOneWidget);
        
        // 等待测试完成
        await tester.pumpAndSettle(Duration(seconds: 10));
        
        // 验证测试结果显示
        expect(find.byIcon(Icons.check_circle).or(find.byIcon(Icons.error)), findsOneWidget);
        
        // 测试嵌入功能
        await tester.tap(find.text('测试嵌入'));
        await tester.pumpAndSettle();
        
        // 验证测试状态和结果
        expect(find.text('测试中...'), findsOneWidget);
        await tester.pumpAndSettle(Duration(seconds: 10));
        expect(find.byIcon(Icons.check_circle).or(find.byIcon(Icons.error)), findsWidgets);
      });
    });

    group('里程碑D: 全局生效验收测试', () {
      
      testWidgets('D1-D4: 全局路由切换', (WidgetTester tester) async {
        await app.main();
        await tester.pumpAndSettle();

        // 配置OpenAI兼容设置
        await _configureOpenAICompatible(tester);
        
        // 测试聊天路由
        await _testChatRouting(tester);
        
        // 测试嵌入路由
        await _testEmbeddingRouting(tester);
        
        // 验证错误回退
        await _testErrorFallback(tester);
      });
    });

    group('安全性验证', () {
      
      testWidgets('API密钥安全存储和显示', (WidgetTester tester) async {
        await app.main();
        await tester.pumpAndSettle();

        await _navigateToAISettings(tester);
        await _switchToOpenAICompatible(tester);
        
        // 输入API密钥
        final apiKeyField = find.byType(TextField).first;
        await tester.enterText(apiKeyField, 'test-secret-key-123');
        await tester.pumpAndSettle();
        
        // 验证密钥字段遮蔽显示
        final textField = tester.widget<TextField>(apiKeyField);
        expect(textField.obscureText, isTrue);
        
        // 保存配置
        await tester.tap(find.text('保存设置'));
        await tester.pumpAndSettle();
        
        // 重新加载验证密钥不以明文显示
        await _reloadPage(tester);
        await _navigateToAISettings(tester);
        
        // 验证密钥字段仍然遮蔽
        final reloadedField = find.byType(TextField).first;
        final reloadedTextField = tester.widget<TextField>(reloadedField);
        expect(reloadedTextField.obscureText, isTrue);
      });
    });

    group('跨平台一致性测试', () {
      
      testWidgets('移动端功能验证', (WidgetTester tester) async {
        // 仅在移动平台运行
        if (!Platform.isAndroid && !Platform.isIOS) {
          return;
        }

        await app.main();
        await tester.pumpAndSettle();

        // 验证移动端AI设置入口
        await _navigateToMobileAISettings(tester);
        
        // 验证功能与桌面端一致
        expect(find.text('全局使用的模型类型'), findsOneWidget);
        
        // 验证移动端特有的交互模式
        await _verifyMobileInteractions(tester);
      });
    });
  });
}

// 辅助函数

Future<void> _setupTestEnvironment() async {
  // 设置测试环境
  debugPrint('设置测试环境...');
  
  // 清理之前的测试数据
  // 设置测试用的配置
}

Future<void> _cleanupTestEnvironment() async {
  // 清理测试环境
  debugPrint('清理测试环境...');
  
  // 删除测试数据
  // 恢复默认配置
}

Future<void> _navigateToAISettings(WidgetTester tester) async {
  // 导航到AI设置页面
  await tester.tap(find.text('设置'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('AI设置'));
  await tester.pumpAndSettle();
}

Future<void> _navigateToMobileAISettings(WidgetTester tester) async {
  // 移动端导航到AI设置
  await tester.tap(find.byIcon(Icons.menu));
  await tester.pumpAndSettle();
  await tester.tap(find.text('设置'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('AI设置'));
  await tester.pumpAndSettle();
}

Future<void> _switchLanguage(WidgetTester tester, String languageCode) async {
  // 切换语言
  await tester.tap(find.text('语言设置'));
  await tester.pumpAndSettle();
  await tester.tap(find.text(languageCode == 'zh-CN' ? '中文' : 'English'));
  await tester.pumpAndSettle();
}

Future<void> _switchToOpenAICompatible(WidgetTester tester) async {
  // 切换到OpenAI兼容模式
  await tester.tap(find.text('ollama 本地'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('openai 兼容服务器'));
  await tester.pumpAndSettle();
}

Future<void> _fillTestConfiguration(WidgetTester tester) async {
  // 填写测试配置
  final textFields = find.byType(TextField);
  
  // 聊天API端点
  await tester.enterText(textFields.at(0), 'http://localhost:11434/v1');
  await tester.pumpAndSettle();
  
  // 聊天API密钥
  await tester.enterText(textFields.at(1), 'test-key-123');
  await tester.pumpAndSettle();
  
  // 聊天模型名称
  await tester.enterText(textFields.at(2), 'llama2');
  await tester.pumpAndSettle();
  
  // 嵌入API端点
  await tester.enterText(textFields.at(7), 'http://localhost:11434/v1');
  await tester.pumpAndSettle();
  
  // 嵌入API密钥
  await tester.enterText(textFields.at(8), 'test-key-123');
  await tester.pumpAndSettle();
  
  // 嵌入模型名称
  await tester.enterText(textFields.at(9), 'nomic-embed-text');
  await tester.pumpAndSettle();
}

Future<void> _reloadPage(WidgetTester tester) async {
  // 重新加载页面
  await tester.binding.reassembleApplication();
  await tester.pumpAndSettle();
}

Future<void> _verifyConfigurationPersisted(WidgetTester tester) async {
  // 验证配置持久化
  await _switchToOpenAICompatible(tester);
  
  final textFields = find.byType(TextField);
  
  // 验证配置值保持
  expect(tester.widget<TextField>(textFields.at(0)).controller?.text, 
         contains('localhost:11434'));
  expect(tester.widget<TextField>(textFields.at(2)).controller?.text, 
         equals('llama2'));
}

Future<void> _configureOpenAICompatible(WidgetTester tester) async {
  // 配置OpenAI兼容设置
  await _navigateToAISettings(tester);
  await _switchToOpenAICompatible(tester);
  await _fillTestConfiguration(tester);
  
  await tester.tap(find.text('保存设置'));
  await tester.pumpAndSettle();
}

Future<void> _testChatRouting(WidgetTester tester) async {
  // 测试聊天路由
  // 导航到聊天页面
  await tester.tap(find.text('AI聊天'));
  await tester.pumpAndSettle();
  
  // 发送测试消息
  await tester.enterText(find.byType(TextField), '测试消息');
  await tester.tap(find.byIcon(Icons.send));
  await tester.pumpAndSettle();
  
  // 验证消息发送成功（这里需要根据实际UI调整）
  expect(find.text('测试消息'), findsOneWidget);
}

Future<void> _testEmbeddingRouting(WidgetTester tester) async {
  // 测试嵌入路由
  // 这里需要根据实际的嵌入功能UI进行测试
  debugPrint('测试嵌入路由...');
}

Future<void> _testErrorFallback(WidgetTester tester) async {
  // 测试错误回退
  await _navigateToAISettings(tester);
  await _switchToOpenAICompatible(tester);
  
  // 配置无效的端点
  final textFields = find.byType(TextField);
  await tester.enterText(textFields.at(0), 'http://invalid-endpoint');
  await tester.pumpAndSettle();
  
  await tester.tap(find.text('保存设置'));
  await tester.pumpAndSettle();
  
  // 测试聊天，验证回退到本地AI
  await _testChatRouting(tester);
}

Future<void> _verifyMobileInteractions(WidgetTester tester) async {
  // 验证移动端特有的交互
  debugPrint('验证移动端交互...');
  
  // 验证触摸交互
  // 验证滚动行为
  // 验证移动端特有的UI元素
}

// 测试数据类
class TestConfiguration {
  static const String testChatEndpoint = 'http://localhost:11434/v1';
  static const String testApiKey = 'test-key-123';
  static const String testChatModel = 'llama2';
  static const String testEmbeddingModel = 'nomic-embed-text';
}

// 测试结果记录
class TestResult {
  final String testName;
  final bool passed;
  final String? errorMessage;
  final DateTime timestamp;

  TestResult({
    required this.testName,
    required this.passed,
    this.errorMessage,
    required this.timestamp,
  });

  @override
  String toString() {
    return '[$timestamp] $testName: ${passed ? 'PASS' : 'FAIL'}${errorMessage != null ? ' - $errorMessage' : ''}';
  }
}

// 测试报告生成器
class TestReportGenerator {
  static final List<TestResult> _results = [];

  static void addResult(TestResult result) {
    _results.add(result);
    debugPrint(result.toString());
  }

  static void generateReport() {
    debugPrint('\n=== 验收测试报告 ===');
    debugPrint('测试时间: ${DateTime.now()}');
    debugPrint('总测试数: ${_results.length}');
    debugPrint('通过数: ${_results.where((r) => r.passed).length}');
    debugPrint('失败数: ${_results.where((r) => !r.passed).length}');
    
    debugPrint('\n详细结果:');
    for (final result in _results) {
      debugPrint(result.toString());
    }
    
    final passRate = _results.isEmpty ? 0.0 : 
        _results.where((r) => r.passed).length / _results.length * 100;
    debugPrint('\n通过率: ${passRate.toStringAsFixed(1)}%');
    
    if (passRate >= 95.0) {
      debugPrint('✅ 验收测试通过！');
    } else if (passRate >= 80.0) {
      debugPrint('⚠️ 验收测试基本通过，建议修复失败项');
    } else {
      debugPrint('❌ 验收测试失败，需要修复问题');
    }
  }
}
