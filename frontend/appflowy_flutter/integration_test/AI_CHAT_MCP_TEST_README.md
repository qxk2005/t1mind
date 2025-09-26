# AI聊天MCP工具编排集成测试

## 概述

本测试套件验证AI聊天MCP工具编排功能的完整用户工作流程，包括：

- 任务规划创建和确认
- MCP工具选择和配置
- 任务执行监控
- 执行日志记录和查看
- 智能体配置管理
- 跨平台兼容性

## 测试文件结构

```
integration_test/
├── ai_chat_mcp_test.dart           # 主要测试文件
├── ai_chat_mcp_test_runner.dart    # 测试运行器
└── AI_CHAT_MCP_TEST_README.md      # 本说明文件
```

## 运行测试

### 运行所有测试

```bash
cd appflowy_flutter
flutter test integration_test/ai_chat_mcp_test_runner.dart
```

### 运行特定测试组

```bash
# 运行任务规划工作流程测试
flutter test integration_test/ai_chat_mcp_test_runner.dart --name="任务规划工作流程"

# 运行智能体配置管理测试
flutter test integration_test/ai_chat_mcp_test_runner.dart --name="智能体配置管理"

# 运行执行日志和追溯测试
flutter test integration_test/ai_chat_mcp_test_runner.dart --name="执行日志和追溯"
```

### 运行特定测试用例

```bash
# 运行完整的任务规划到执行流程测试
flutter test integration_test/ai_chat_mcp_test_runner.dart --name="完整的任务规划到执行流程"

# 运行错误处理流程测试
flutter test integration_test/ai_chat_mcp_test_runner.dart --name="任务规划错误处理流程"

# 运行跨平台兼容性测试
flutter test integration_test/ai_chat_mcp_test_runner.dart --name="跨平台兼容性测试"
```

## 测试覆盖范围

### 1. 任务规划工作流程
- ✅ MCP工具选择器功能
- ✅ 任务规划创建和生成
- ✅ 任务确认对话框交互
- ✅ 任务执行监控和进度显示
- ✅ 执行日志查看和搜索
- ✅ 错误处理和异常情况
- ✅ 跨平台UI适配

### 2. 智能体配置管理
- ✅ 智能体配置的CRUD操作
- ✅ 配置验证和错误处理
- ✅ 配置导入导出功能

### 3. 执行日志和追溯
- ✅ 执行日志记录
- ✅ 日志查询和过滤
- ✅ 日志导出功能
- ✅ 执行追溯和调试支持

## 测试环境要求

### 前置条件
- Flutter SDK (最新稳定版)
- AppFlowy开发环境已配置
- 集成测试依赖已安装

### 依赖包
```yaml
dev_dependencies:
  integration_test:
    sdk: flutter
  flutter_test:
    sdk: flutter
```

## 测试数据和模拟

### 测试用户
- 用户ID: `457037009907617792`
- 用户名: `TestUser`

### 测试场景
1. **正常流程**: 完整的任务规划到执行流程
2. **错误处理**: 无效输入和异常情况
3. **边界测试**: 空输入、特殊字符等
4. **性能测试**: 大量数据和长时间运行
5. **兼容性测试**: 不同屏幕尺寸和平台

## 测试稳定性保证

### 等待机制
- 使用`pumpAndSettle()`确保UI完全渲染
- 实现状态轮询等待机制
- 设置合理的超时时间

### 错误处理
- 捕获和记录测试异常
- 提供详细的错误信息
- 支持测试重试机制

### 清理机制
- 测试后清理临时数据
- 重置应用状态
- 释放资源和连接

## 故障排除

### 常见问题

1. **测试超时**
   - 检查网络连接
   - 增加超时时间设置
   - 确认后端服务正常

2. **UI元素找不到**
   - 检查UI组件是否正确渲染
   - 验证选择器是否正确
   - 确认测试数据是否有效

3. **状态不一致**
   - 检查BLoC状态管理
   - 验证事件触发顺序
   - 确认异步操作完成

### 调试技巧

1. **启用详细日志**
   ```bash
   flutter test integration_test/ai_chat_mcp_test_runner.dart --verbose
   ```

2. **单步调试**
   - 在测试代码中添加断点
   - 使用`debugger()`语句
   - 检查UI状态和数据

3. **截图调试**
   - 在关键步骤添加截图
   - 比较期望和实际UI状态
   - 记录测试执行过程

## 持续集成

### CI/CD配置
```yaml
# .github/workflows/integration_test.yml
name: Integration Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: subosito/flutter-action@v2
      - run: flutter test integration_test/ai_chat_mcp_test_runner.dart
```

### 测试报告
- 生成详细的测试报告
- 记录测试覆盖率
- 跟踪测试性能指标

## 维护和更新

### 定期维护
- 更新测试数据和场景
- 修复因UI变更导致的测试失败
- 优化测试性能和稳定性

### 版本兼容性
- 跟踪Flutter和依赖包版本
- 测试新功能的兼容性
- 维护向后兼容性

## 贡献指南

### 添加新测试
1. 在`ai_chat_mcp_test.dart`中添加测试用例
2. 遵循现有的测试模式和命名规范
3. 添加适当的文档和注释
4. 确保测试稳定性和可重复性

### 代码规范
- 使用描述性的测试名称
- 添加详细的测试步骤注释
- 遵循Flutter测试最佳实践
- 保持测试代码的可维护性

---

**注意**: 这些集成测试需要完整的AppFlowy环境和AI聊天MCP工具编排功能。确保在运行测试前已正确配置所有必要的组件和依赖。
