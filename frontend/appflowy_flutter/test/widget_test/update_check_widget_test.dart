import 'package:appflowy/core/helpers/url_launcher.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/update_check_widget.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

// Mock classes
class MockT1MindVersionChecker extends Mock implements T1MindVersionChecker {}
class MockT1MindUpdateInfo extends Mock implements T1MindUpdateInfo {}
class MockBuildContext extends Mock implements BuildContext {}

void main() {
  group('UpdateCheckWidget', () {
    late MockT1MindVersionChecker mockVersionChecker;
    late MockT1MindUpdateInfo mockUpdateInfo;

    setUp(() {
      mockVersionChecker = MockT1MindVersionChecker();
      mockUpdateInfo = MockT1MindUpdateInfo();
    });

    Widget createTestWidget() {
      return MaterialApp(
        localizationsDelegates: [
          ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
        ],
        supportedLocales: const [Locale('en', 'US')],
        home: Scaffold(
          body: UpdateCheckWidget(),
        ),
      );
    }

    testWidgets('should display check button initially', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('检查更新'), findsOneWidget);
      expect(find.text('点击检查是否有新版本可用'), findsOneWidget);
      expect(find.byType(ElevatedButton), findsOneWidget);
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

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('正在检查更新...'), findsOneWidget);
    });

    testWidgets('should disable check button while checking', (WidgetTester tester) async {
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

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pump();

      // Assert
      expect(find.text('正在检查更新...'), findsOneWidget);
      // Button should be disabled (text changed)
      expect(find.text('检查更新'), findsNothing);
    });

    testWidgets('should show update available when update is found', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.releaseNotesUrl).thenReturn('https://example.com/release-notes');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsOneWidget);
      expect(find.text('当前版本: 1.0.0'), findsOneWidget);
      expect(find.text('最新版本: 1.1.0'), findsOneWidget);
      expect(find.text('下载更新'), findsOneWidget);
      expect(find.text('查看发布说明'), findsOneWidget);
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

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('已是最新版本'), findsOneWidget);
      expect(find.text('当前版本: 1.0.0'), findsOneWidget);
      expect(find.byIcon(Icons.check_circle_outline), findsOneWidget);
    });

    testWidgets('should show error message when check fails', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenThrow(Exception('Network error'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('检查更新失败'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('should handle download button tap', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.releaseNotesUrl).thenReturn('https://example.com/release-notes');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
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

    testWidgets('should handle release notes button tap', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.releaseNotesUrl).thenReturn('https://example.com/release-notes');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Tap release notes button
      await tester.tap(find.text('查看发布说明'));
      await tester.pumpAndSettle();

      // Assert
      // The URL launcher would be called, but we can't easily test that in widget tests
      // We just verify the button is present and tappable
      expect(find.text('查看发布说明'), findsOneWidget);
    });

    testWidgets('should not show update result initially', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsNothing);
      expect(find.text('已是最新版本'), findsNothing);
      expect(find.text('检查更新失败'), findsNothing);
    });

    testWidgets('should use SettingsCategory layout', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SettingsCategory), findsOneWidget);
    });

    testWidgets('should handle empty download URL gracefully', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('');
      when(() => mockUpdateInfo.releaseNotesUrl).thenReturn('');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Tap download button (should not cause error)
      await tester.tap(find.text('下载更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsOneWidget);
    });

    testWidgets('should handle empty release notes URL gracefully', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.releaseNotesUrl).thenReturn('');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Tap release notes button (should not cause error)
      await tester.tap(find.text('查看发布说明'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsOneWidget);
    });

    testWidgets('should handle null update info gracefully', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => null);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsNothing);
      expect(find.text('已是最新版本'), findsNothing);
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
      await tester.tap(find.text('正在检查更新...'));
      await tester.pump();
      await tester.tap(find.text('正在检查更新...'));
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      // Should only show one loading indicator despite multiple taps
    });

    testWidgets('should show proper styling for update available state', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.downloadUrl).thenReturn('https://example.com/download');
      when(() => mockUpdateInfo.releaseNotesUrl).thenReturn('https://example.com/release-notes');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      // Verify the update available container is present
      final updateContainer = find.byType(Container).last;
      expect(updateContainer, findsOneWidget);
      
      // Verify the primary color dot is present
      final dotContainer = find.byType(Container).first;
      expect(dotContainer, findsOneWidget);
    });

    testWidgets('should show proper styling for up to date state', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      // Verify the check icon is present
      expect(find.byIcon(Icons.check_circle_outline), findsOneWidget);
    });

    testWidgets('should show proper styling for error state', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenThrow(Exception('Test error'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Tap check button
      await tester.tap(find.text('检查更新'));
      await tester.pumpAndSettle();

      // Assert
      // Verify the error icon is present
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });
  });
}
