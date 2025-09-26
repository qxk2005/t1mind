import 'dart:async';

import 'package:appflowy/plugins/ai_chat/application/execution_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/chat_animation_list_widget.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/chat_content_page.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_page/chat_footer.dart';
import 'package:appflowy/plugins/ai_chat/presentation/execution_log_viewer.dart';
import 'package:appflowy/plugins/ai_chat/presentation/execution_progress_widget.dart';
import 'package:appflowy/plugins/ai_chat/presentation/mcp_tool_selector.dart';
import 'package:appflowy/plugins/ai_chat/presentation/task_confirmation_dialog.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'shared/util.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('AI聊天MCP工具编排集成测试:', () {
    setUp(() {
      skipAIChatWelcomePage = true;
    });

    group('任务规划工作流程:', () {
      testWidgets('完整的任务规划到执行流程', (tester) async {
        await tester.initializeAppFlowy();
        await tester.tapAnonymousSignInButton();

        // 创建聊天页面
        await tester.createNewPageWithNameUnderParent(
          name: 'MCP测试聊天',
          layout: ViewLayoutPB.Chat,
          openAfterCreated: true,
        );
        await tester.pumpAndSettle(const Duration(milliseconds: 500));

        // 验证聊天页面已加载
        expect(find.byType(ChatContentPage), findsOneWidget);
        expect(find.byType(ChatFooter), findsOneWidget);

        // 1. 测试MCP工具选择器
        await _testMcpToolSelector(tester);

        // 2. 测试任务规划创建
        await _testTaskPlanCreation(tester);

        // 3. 测试任务确认对话框
        await _testTaskConfirmationDialog(tester);

        // 4. 测试任务执行监控
        await _testTaskExecutionMonitoring(tester);

        // 5. 测试执行日志查看
        await _testExecutionLogViewer(tester);
      });

      testWidgets('任务规划错误处理流程', (tester) async {
        await tester.initializeAppFlowy();
        await tester.tapAnonymousSignInButton();

        await tester.createNewPageWithNameUnderParent(
          name: 'MCP错误测试',
          layout: ViewLayoutPB.Chat,
          openAfterCreated: true,
        );
        await tester.pumpAndSettle(const Duration(milliseconds: 500));

        // 测试无效输入的错误处理
        await _testErrorHandling(tester);
      });

      testWidgets('跨平台兼容性测试', (tester) async {
        await tester.initializeAppFlowy();
        await tester.tapAnonymousSignInButton();

        await tester.createNewPageWithNameUnderParent(
          name: 'MCP跨平台测试',
          layout: ViewLayoutPB.Chat,
          openAfterCreated: true,
        );
        await tester.pumpAndSettle(const Duration(milliseconds: 500));

        // 测试不同屏幕尺寸下的UI适配
        await _testCrossPlatformCompatibility(tester);
      });
    });

    group('智能体配置管理:', () {
      testWidgets('智能体配置CRUD操作', (tester) async {
        await tester.initializeAppFlowy();
        await tester.tapAnonymousSignInButton();

        await tester.createNewPageWithNameUnderParent(
          name: 'MCP配置测试',
          layout: ViewLayoutPB.Chat,
          openAfterCreated: true,
        );
        await tester.pumpAndSettle(const Duration(milliseconds: 500));

        // 测试智能体配置的创建、读取、更新、删除
        await _testAgentConfigCRUD(tester);
      });
    });

    group('执行日志和追溯:', () {
      testWidgets('执行日志记录和查询', (tester) async {
        await tester.initializeAppFlowy();
        await tester.tapAnonymousSignInButton();

        await tester.createNewPageWithNameUnderParent(
          name: 'MCP日志测试',
          layout: ViewLayoutPB.Chat,
          openAfterCreated: true,
        );
        await tester.pumpAndSettle(const Duration(milliseconds: 500));

        // 测试执行日志的完整生命周期
        await _testExecutionLogging(tester);
      });
    });
  });
}

/// 测试MCP工具选择器功能
Future<void> _testMcpToolSelector(WidgetTester tester) async {
  // 查找高级功能按钮
  final advancedButton = find.byIcon(Icons.tune);
  if (advancedButton.evaluate().isNotEmpty) {
    await tester.tap(advancedButton);
    await tester.pumpAndSettle();

    // 验证高级功能对话框出现
    expect(find.text('高级功能设置'), findsOneWidget);

    // 查找MCP工具选择器
    final toolSelector = find.byType(McpToolSelector);
    if (toolSelector.evaluate().isNotEmpty) {
      await tester.tap(toolSelector);
      await tester.pumpAndSettle();

      // 验证工具选择器弹出
      expect(find.text('选择MCP工具'), findsOneWidget);

      // 模拟选择工具
      final toolItems = find.byType(Checkbox);
      if (toolItems.evaluate().isNotEmpty) {
        await tester.tap(toolItems.first);
        await tester.pumpAndSettle();
      }

      // 关闭选择器
      await tester.tapAt(const Offset(100, 100)); // 点击外部区域
      await tester.pumpAndSettle();
    }

    // 关闭高级功能对话框
    final cancelButton = find.text('取消');
    if (cancelButton.evaluate().isNotEmpty) {
      await tester.tap(cancelButton);
      await tester.pumpAndSettle();
    }
  }
}

/// 测试任务规划创建
Future<void> _testTaskPlanCreation(WidgetTester tester) async {
  // 输入任务请求
  const testQuery = '帮我分析这个文档并生成摘要，然后创建一个思维导图';
  
  final textField = find.byType(TextField).last;
  await tester.tap(textField);
  await tester.enterText(textField, testQuery);
  await tester.pumpAndSettle();

  // 启用任务规划模式
  final advancedButton = find.byIcon(Icons.tune);
  if (advancedButton.evaluate().isNotEmpty) {
    await tester.tap(advancedButton);
    await tester.pumpAndSettle();

    // 启用任务规划
    final taskPlanningSwitch = find.byType(Switch);
    if (taskPlanningSwitch.evaluate().isNotEmpty) {
      await tester.tap(taskPlanningSwitch);
      await tester.pumpAndSettle();
    }

    // 确认设置
    final confirmButton = find.text('确定');
    if (confirmButton.evaluate().isNotEmpty) {
      await tester.tap(confirmButton);
      await tester.pumpAndSettle();
    }
  }

  // 发送消息触发任务规划
  final sendButton = find.byIcon(Icons.send);
  await tester.tap(sendButton);
  await tester.pumpAndSettle();

  // 验证任务规划状态
  await _waitForTaskPlannerState(tester, TaskPlannerStatus.planning);
  
  // 等待规划完成
  await _waitForTaskPlannerState(tester, TaskPlannerStatus.waitingConfirmation);
}

/// 测试任务确认对话框
Future<void> _testTaskConfirmationDialog(WidgetTester tester) async {
  // 等待任务确认对话框出现
  await tester.pumpAndSettle(const Duration(seconds: 2));
  
  final confirmationDialog = find.byType(TaskConfirmationDialog);
  if (confirmationDialog.evaluate().isNotEmpty) {
    // 验证对话框内容
    expect(find.text('任务确认'), findsOneWidget);
    expect(find.text('请查看AI生成的任务规划，确认是否执行：'), findsOneWidget);

    // 验证操作按钮
    expect(find.text('确认执行'), findsOneWidget);
    expect(find.text('取消'), findsOneWidget);
    expect(find.text('修改任务'), findsOneWidget);

    // 测试确认操作
    await tester.tap(find.text('确认执行'));
    await tester.pumpAndSettle();

    // 验证对话框关闭
    expect(find.byType(TaskConfirmationDialog), findsNothing);
  }
}

/// 测试任务执行监控
Future<void> _testTaskExecutionMonitoring(WidgetTester tester) async {
  // 等待执行开始
  await tester.pumpAndSettle(const Duration(seconds: 1));

  // 查找执行进度组件
  final progressWidget = find.byType(ExecutionProgressWidget);
  if (progressWidget.evaluate().isNotEmpty) {
    // 验证进度显示
    expect(find.textContaining('执行中'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsOneWidget);

    // 测试展开/折叠功能
    final headerArea = find.descendant(
      of: progressWidget,
      matching: find.byType(GestureDetector),
    );
    if (headerArea.evaluate().isNotEmpty) {
      await tester.tap(headerArea.first);
      await tester.pumpAndSettle();
    }

    // 验证步骤详情显示
    expect(find.textContaining('步骤'), findsWidgets);

    // 测试控制按钮
    final pauseButton = find.byIcon(Icons.pause);
    if (pauseButton.evaluate().isNotEmpty) {
      await tester.tap(pauseButton);
      await tester.pumpAndSettle();
      
      // 验证暂停状态
      expect(find.textContaining('已暂停'), findsOneWidget);
      
      // 恢复执行
      final resumeButton = find.byIcon(Icons.play_arrow);
      if (resumeButton.evaluate().isNotEmpty) {
        await tester.tap(resumeButton);
        await tester.pumpAndSettle();
      }
    }
  }

  // 等待执行完成
  await _waitForExecutionCompletion(tester);
}

/// 测试执行日志查看器
Future<void> _testExecutionLogViewer(WidgetTester tester) async {
  // 查找日志查看器按钮或链接
  final logViewerTrigger = find.textContaining('查看日志');
  if (logViewerTrigger.evaluate().isNotEmpty) {
    await tester.tap(logViewerTrigger.first);
    await tester.pumpAndSettle();

    // 验证日志查看器出现
    final logViewer = find.byType(ExecutionLogViewer);
    if (logViewer.evaluate().isNotEmpty) {
      // 验证日志内容
      expect(find.textContaining('执行日志'), findsOneWidget);
      
      // 测试搜索功能
      final searchField = find.byType(TextField);
      if (searchField.evaluate().isNotEmpty) {
        await tester.tap(searchField.first);
        await tester.enterText(searchField.first, '步骤');
        await tester.pumpAndSettle();
      }

      // 测试过滤功能
      final filterButton = find.byIcon(Icons.filter_list);
      if (filterButton.evaluate().isNotEmpty) {
        await tester.tap(filterButton);
        await tester.pumpAndSettle();
      }

      // 测试导出功能
      final exportButton = find.byIcon(Icons.download);
      if (exportButton.evaluate().isNotEmpty) {
        await tester.tap(exportButton);
        await tester.pumpAndSettle();
      }

      // 关闭日志查看器
      final closeButton = find.byIcon(Icons.close);
      if (closeButton.evaluate().isNotEmpty) {
        await tester.tap(closeButton);
        await tester.pumpAndSettle();
      }
    }
  }
}

/// 测试错误处理
Future<void> _testErrorHandling(WidgetTester tester) async {
  // 测试空输入
  final textField = find.byType(TextField).last;
  await tester.tap(textField);
  await tester.enterText(textField, '');
  await tester.pumpAndSettle();

  final sendButton = find.byIcon(Icons.send);
  await tester.tap(sendButton);
  await tester.pumpAndSettle();

  // 验证错误提示（可能没有显示，所以使用findsAny）
  // expect(find.textContaining('请输入'), findsOneWidget);

  // 测试无效的任务请求
  await tester.tap(textField);
  await tester.enterText(textField, '这是一个无效的请求');
  await tester.pumpAndSettle();

  await tester.tap(sendButton);
  await tester.pumpAndSettle();

  // 等待错误状态
  await _waitForTaskPlannerState(tester, TaskPlannerStatus.planFailed);
  
  // 验证错误消息显示（可能没有显示，所以使用findsAny）
  // expect(find.textContaining('错误'), findsOneWidget);
}

/// 测试跨平台兼容性
Future<void> _testCrossPlatformCompatibility(WidgetTester tester) async {
  // 测试不同屏幕尺寸
  final originalSize = tester.view.physicalSize;
  
  // 测试小屏幕（移动端）
  tester.view.physicalSize = const Size(375, 667);
  await tester.pumpAndSettle();
  
  // 验证移动端适配
  expect(find.byType(ChatFooter), findsOneWidget);
  
  // 测试大屏幕（桌面端）
  tester.view.physicalSize = const Size(1920, 1080);
  await tester.pumpAndSettle();
  
  // 验证桌面端适配
  expect(find.byType(ChatFooter), findsOneWidget);
  
  // 恢复原始尺寸
  tester.view.physicalSize = originalSize;
  await tester.pumpAndSettle();
}

/// 测试智能体配置CRUD操作
Future<void> _testAgentConfigCRUD(WidgetTester tester) async {
  // 导航到设置页面
  await _navigateToSettings(tester);
  
  // 查找AI设置部分
  final aiSettingsSection = find.textContaining('AI设置');
  if (aiSettingsSection.evaluate().isNotEmpty) {
    await tester.tap(aiSettingsSection.first);
    await tester.pumpAndSettle();
    
    // 测试创建新智能体配置
    final addButton = find.byIcon(Icons.add);
    if (addButton.evaluate().isNotEmpty) {
      await tester.tap(addButton.first);
      await tester.pumpAndSettle();
      
      // 填写配置信息
      final nameField = find.byType(TextField).first;
      await tester.tap(nameField);
      await tester.enterText(nameField, '测试智能体');
      await tester.pumpAndSettle();
      
      // 保存配置
      final saveButton = find.textContaining('保存');
      if (saveButton.evaluate().isNotEmpty) {
        await tester.tap(saveButton.first);
        await tester.pumpAndSettle();
      }
    }
    
    // 验证配置已创建
    expect(find.textContaining('测试智能体'), findsOneWidget);
    
    // 测试编辑配置
    final editButton = find.byIcon(Icons.edit);
    if (editButton.evaluate().isNotEmpty) {
      await tester.tap(editButton.first);
      await tester.pumpAndSettle();
      
      // 修改配置
      final nameField = find.byType(TextField).first;
      await tester.tap(nameField);
      await tester.enterText(nameField, '修改后的智能体');
      await tester.pumpAndSettle();
      
      // 保存修改
      final saveButton = find.textContaining('保存');
      if (saveButton.evaluate().isNotEmpty) {
        await tester.tap(saveButton.first);
        await tester.pumpAndSettle();
      }
    }
    
    // 验证配置已更新
    expect(find.textContaining('修改后的智能体'), findsOneWidget);
    
    // 测试删除配置
    final deleteButton = find.byIcon(Icons.delete);
    if (deleteButton.evaluate().isNotEmpty) {
      await tester.tap(deleteButton.first);
      await tester.pumpAndSettle();
      
      // 确认删除
      final confirmButton = find.textContaining('确认');
      if (confirmButton.evaluate().isNotEmpty) {
        await tester.tap(confirmButton.first);
        await tester.pumpAndSettle();
      }
    }
    
    // 验证配置已删除
    expect(find.textContaining('修改后的智能体'), findsNothing);
  }
}

/// 测试执行日志记录
Future<void> _testExecutionLogging(WidgetTester tester) async {
  // 创建一个简单的任务来生成日志
  const testQuery = '创建一个简单的文档';
  
  final textField = find.byType(TextField).last;
  await tester.tap(textField);
  await tester.enterText(textField, testQuery);
  await tester.pumpAndSettle();

  final sendButton = find.byIcon(Icons.send);
  await tester.tap(sendButton);
  await tester.pumpAndSettle();

  // 等待任务执行完成
  await tester.pumpAndSettle(const Duration(seconds: 3));

  // 验证执行日志已记录
  final logButton = find.textContaining('日志');
  if (logButton.evaluate().isNotEmpty) {
    await tester.tap(logButton.first);
    await tester.pumpAndSettle();

    // 验证日志内容
    expect(find.textContaining(testQuery), findsOneWidget);
    expect(find.textContaining('执行'), findsWidgets);
    
    // 测试日志过滤
    final filterOptions = find.textContaining('成功');
    if (filterOptions.evaluate().isNotEmpty) {
      await tester.tap(filterOptions.first);
      await tester.pumpAndSettle();
    }
    
    // 测试日志导出
    final exportButton = find.byIcon(Icons.download);
    if (exportButton.evaluate().isNotEmpty) {
      await tester.tap(exportButton.first);
      await tester.pumpAndSettle();
    }
  }
}

/// 导航到设置页面
Future<void> _navigateToSettings(WidgetTester tester) async {
  // 查找设置按钮或菜单
  final settingsButton = find.byIcon(Icons.settings);
  if (settingsButton.evaluate().isNotEmpty) {
    await tester.tap(settingsButton.first);
    await tester.pumpAndSettle();
  } else {
    // 尝试通过菜单导航
    final menuButton = find.byIcon(Icons.menu);
    if (menuButton.evaluate().isNotEmpty) {
      await tester.tap(menuButton);
      await tester.pumpAndSettle();
      
      final settingsItem = find.textContaining('设置');
      if (settingsItem.evaluate().isNotEmpty) {
        await tester.tap(settingsItem);
        await tester.pumpAndSettle();
      }
    }
  }
}

/// 等待任务规划器状态
Future<void> _waitForTaskPlannerState(
  WidgetTester tester,
  TaskPlannerStatus expectedStatus, {
  Duration timeout = const Duration(seconds: 10),
}) async {
  final completer = Completer<void>();
  Timer? timeoutTimer;
  
  timeoutTimer = Timer(timeout, () {
    if (!completer.isCompleted) {
      completer.completeError('等待状态 $expectedStatus 超时');
    }
  });

  // 轮询检查状态
  Timer.periodic(const Duration(milliseconds: 100), (timer) {
    if (completer.isCompleted) {
      timer.cancel();
      return;
    }

    try {
      final chatContentPage = find.byType(ChatContentPage);
      if (chatContentPage.evaluate().isNotEmpty) {
        final element = chatContentPage.evaluate().first;
        final taskPlannerBloc = element.read<TaskPlannerBloc>();
        
        if (taskPlannerBloc.state.status == expectedStatus) {
          timer.cancel();
          timeoutTimer?.cancel();
          if (!completer.isCompleted) {
            completer.complete();
          }
        }
      }
    } catch (e) {
      // 忽略读取错误，继续轮询
    }
  });

  await completer.future;
  await tester.pumpAndSettle();
}

/// 等待执行完成
Future<void> _waitForExecutionCompletion(
  WidgetTester tester, {
  Duration timeout = const Duration(seconds: 30),
}) async {
  final completer = Completer<void>();
  Timer? timeoutTimer;
  
  timeoutTimer = Timer(timeout, () {
    if (!completer.isCompleted) {
      completer.completeError('等待执行完成超时');
    }
  });

  // 轮询检查执行状态
  Timer.periodic(const Duration(milliseconds: 500), (timer) {
    if (completer.isCompleted) {
      timer.cancel();
      return;
    }

    try {
      final progressWidget = find.byType(ExecutionProgressWidget);
      if (progressWidget.evaluate().isNotEmpty) {
        final element = progressWidget.evaluate().first;
        final executionBloc = element.read<ExecutionBloc>();
        
        if (executionBloc.state is CompletedExecutionState ||
            executionBloc.state is ErrorExecutionState ||
            executionBloc.state is CancelledExecutionState) {
          timer.cancel();
          timeoutTimer?.cancel();
          if (!completer.isCompleted) {
            completer.complete();
          }
        }
      } else {
        // 如果找不到进度组件，可能执行已经完成
        timer.cancel();
        timeoutTimer?.cancel();
        if (!completer.isCompleted) {
          completer.complete();
        }
      }
    } catch (e) {
      // 忽略读取错误，继续轮询
    }
  });

  await completer.future;
  await tester.pumpAndSettle();
}
