import 'package:integration_test/integration_test.dart';

import 'ai_chat_mcp_test.dart' as ai_chat_mcp_test;

/// AI聊天MCP工具编排集成测试运行器
/// 
/// 该运行器专门用于执行AI聊天MCP工具编排功能的集成测试，
/// 包括任务规划、执行监控、日志记录等完整的用户工作流程验证。
/// 
/// 使用方法：
/// ```bash
/// flutter test integration_test/ai_chat_mcp_test_runner.dart
/// ```
/// 
/// 或者运行特定的测试组：
/// ```bash
/// flutter test integration_test/ai_chat_mcp_test_runner.dart --name="任务规划工作流程"
/// ```
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  
  // 运行AI聊天MCP工具编排集成测试
  ai_chat_mcp_test.main();
}
