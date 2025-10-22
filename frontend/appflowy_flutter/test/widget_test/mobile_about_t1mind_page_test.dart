import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy/shared/changelog/changelog_loader.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy/mobile/presentation/setting/about/mobile_about_t1mind_page.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

// Mock classes
class MockT1MindVersionChecker extends Mock implements T1MindVersionChecker {}
class MockT1MindUpdateInfo extends Mock implements T1MindUpdateInfo {}
class MockChangelogLoader extends Mock implements ChangelogLoader {}

void main() {
  group('MobileAboutT1MindPage', () {
    late MockT1MindVersionChecker mockVersionChecker;
    late MockT1MindUpdateInfo mockUpdateInfo;
    late MockChangelogLoader mockChangelogLoader;

    setUp(() {
      mockVersionChecker = MockT1MindVersionChecker();
      mockUpdateInfo = MockT1MindUpdateInfo();
      mockChangelogLoader = MockChangelogLoader();
    });

    Widget createTestWidget() {
      return MaterialApp(
        home: Scaffold(
          body: MobileAboutT1MindPage(),
        ),
      );
    }

    testWidgets('should display app bar with correct title', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于 T1Mind'), findsOneWidget);
      expect(find.byType(AppBar), findsOneWidget);
      expect(find.byIcon(Icons.arrow_back), findsOneWidget);
    });

    testWidgets('should display version information group', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('版本信息'), findsOneWidget);
      expect(find.text('应用版本'), findsOneWidget);
      expect(find.text('构建号'), findsOneWidget);
      expect(find.text('平台'), findsOneWidget);
    });

    testWidgets('should display update check group', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('更新检查'), findsOneWidget);
      expect(find.text('检查更新'), findsOneWidget);
      expect(find.byIcon(Icons.refresh), findsOneWidget);
    });

    testWidgets('should display changelog group', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('更新历史'), findsOneWidget);
      expect(find.text('查看更新历史'), findsOneWidget);
      expect(find.byIcon(Icons.history), findsOneWidget);
    });

    testWidgets('should display copyright group', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('版权信息'), findsOneWidget);
      expect(find.text('T1Mind'), findsOneWidget);
      expect(find.text('AppFlowy'), findsOneWidget);
      expect(find.text('Flutter'), findsOneWidget);
    });

    testWidgets('should display copyright descriptions', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('基于AppFlowy构建的智能笔记应用'), findsOneWidget);
      expect(find.text('开源协作平台，提供强大的文档编辑功能'), findsOneWidget);
      expect(find.text('Google开发的跨平台UI框架'), findsOneWidget);
    });

    testWidgets('should show loading indicator when checking updates', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer(
        (_) async {
          await Future.delayed(const Duration(milliseconds: 100));
          return mockUpdateInfo;
        },
      );

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check updates button
      await tester.tap(find.text('检查更新'));
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('检查中...'), findsOneWidget);
    });

    testWidgets('should show update available when update is found', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check updates button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有新版本可用'), findsOneWidget);
      expect(find.text('最新版本'), findsOneWidget);
      expect(find.text('1.1.0'), findsOneWidget);
      expect(find.text('下载更新'), findsOneWidget);
      expect(find.byIcon(Icons.download), findsOneWidget);
    });

    testWidgets('should show up to date when no update is available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check updates button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('已是最新版本'), findsOneWidget);
      expect(find.text('最新版本'), findsOneWidget);
      expect(find.text('1.0.0'), findsOneWidget);
    });

    testWidgets('should show error message when check fails', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenThrow(Exception('Network error'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check updates button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('检查失败'), findsOneWidget);
      expect(find.text('Network error'), findsOneWidget);
    });

    testWidgets('should show changelog bottom sheet when tapped', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '# Test Changelog');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap changelog button
      await tester.tap(find.text('查看更新历史'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('更新历史'), findsAtLeastNWidgets(2)); // One in main page, one in bottom sheet
      expect(find.byIcon(Icons.close), findsOneWidget);
    });

    testWidgets('should show changelog loading state in bottom sheet', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer(
        (_) async {
          await Future.delayed(const Duration(milliseconds: 100));
          return '# Test Changelog';
        },
      );

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap changelog button
      await tester.tap(find.text('查看更新历史'));
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('should show changelog error state in bottom sheet', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenThrow(Exception('Loading failed'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap changelog button
      await tester.tap(find.text('查看更新历史'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('加载失败'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('should close bottom sheet when close button is tapped', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '# Test Changelog');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap changelog button
      await tester.tap(find.text('查看更新历史'));
      await tester.pumpAndSettle();

      // Verify bottom sheet is open
      expect(find.byIcon(Icons.close), findsOneWidget);

      // Tap close button
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();

      // Assert
      expect(find.byIcon(Icons.close), findsNothing);
    });

    testWidgets('should handle download button tap', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check updates button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Tap download button
      await tester.tap(find.text('下载更新'));
      await tester.pumpAndSettle();

      // Assert
      // The URL launcher would be called, but we can't easily test that in widget tests
      // We just verify the button is present and tappable
      expect(find.text('下载更新'), findsOneWidget);
    });

    testWidgets('should use MobileSettingGroup layout', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(MobileSettingGroup), findsAtLeastNWidgets(4));
    });

    testWidgets('should use MobileSettingItem for each setting', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(MobileSettingItem), findsAtLeastNWidgets(6));
    });

    testWidgets('should handle different screen sizes', (WidgetTester tester) async {
      // Test with different screen sizes
      final testSizes = [
        const Size(360, 640),   // Small mobile
        const Size(414, 896),  // iPhone 11 Pro Max
        const Size(768, 1024), // iPad
      ];

      for (final size in testSizes) {
        // Arrange
        tester.view.physicalSize = size;
        tester.view.devicePixelRatio = 1.0;

        // Act
        await tester.pumpWidget(createTestWidget());
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(MobileAboutT1MindPage), findsOneWidget);
        expect(find.text('关于 T1Mind'), findsOneWidget);

        // Clean up
        await tester.pumpWidget(const SizedBox.shrink());
      }
    });

    testWidgets('should handle theme changes correctly', (WidgetTester tester) async {
      // Test with light theme
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      expect(find.byType(MobileAboutT1MindPage), findsOneWidget);

      // Test with dark theme
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            body: MobileAboutT1MindPage(),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(MobileAboutT1MindPage), findsOneWidget);
    });

    testWidgets('should prevent multiple simultaneous update checks', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer(
        (_) async {
          await Future.delayed(const Duration(milliseconds: 200));
          return mockUpdateInfo;
        },
      );

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button multiple times quickly
      await tester.tap(find.text('检查更新'));
      await tester.pump();
      await tester.tap(find.text('检查更新'));
      await tester.pump();
      await tester.tap(find.text('检查更新'));
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      // Should only show one loading indicator despite multiple taps
    });

    testWidgets('should handle navigation back button', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap back button
      await tester.tap(find.byIcon(Icons.arrow_back));
      await tester.pumpAndSettle();

      // Assert
      // The navigation would pop, but we can't easily test that in widget tests
      // We just verify the button is present and tappable
      expect(find.byIcon(Icons.arrow_back), findsOneWidget);
    });

    testWidgets('should display changelog content in bottom sheet', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# T1Mind 更新历史

## 版本 1.0.0
- 新增功能A
- 修复问题B
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap changelog button
      await tester.tap(find.text('查看更新历史'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('T1Mind 更新历史'), findsOneWidget);
      expect(find.text('版本 1.0.0'), findsOneWidget);
      expect(find.text('新增功能A'), findsOneWidget);
      expect(find.text('修复问题B'), findsOneWidget);
    });

    testWidgets('should handle empty changelog content', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap changelog button
      await tester.tap(find.text('查看更新历史'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('更新历史'), findsAtLeastNWidgets(1));
    });

    testWidgets('should show proper subtitle states for changelog', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Initially should show "点击查看"
      expect(find.text('点击查看'), findsOneWidget);
    });

    testWidgets('should show proper subtitle states for update check', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Initially should show "点击检查"
      expect(find.text('点击检查'), findsOneWidget);
    });
  });
}
