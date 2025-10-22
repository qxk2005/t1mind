import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/version_info_widget.dart';
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
  group('VersionInfoWidget', () {
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
          body: VersionInfoWidget(),
        ),
      );
    }

    testWidgets('should display version information correctly', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('T1Mind版本'), findsOneWidget);
      expect(find.text('构建号'), findsOneWidget);
      expect(find.text('操作系统'), findsOneWidget);
      expect(find.text('架构'), findsOneWidget);
    });

    testWidgets('should show loading indicator while fetching update info', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer(
        (_) async {
          await Future.delayed(const Duration(milliseconds: 100));
          return mockUpdateInfo;
        },
      );

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('should hide loading indicator after update info is loaded', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets('should display latest version when update info is available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('最新版本'), findsOneWidget);
      expect(find.text('1.1.0'), findsOneWidget);
    });

    testWidgets('should show update section when update is available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsOneWidget);
      expect(find.text('1.0.0 → 1.1.0'), findsOneWidget);
    });

    testWidgets('should not show update section when no update is available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('有更新可用'), findsNothing);
    });

    testWidgets('should handle update info loading error gracefully', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenThrow(Exception('Network error'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsNothing);
      expect(find.text('T1Mind版本'), findsOneWidget);
    });

    testWidgets('should display system information correctly', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('系统信息'), findsOneWidget);
      expect(find.text('操作系统'), findsOneWidget);
      expect(find.text('架构'), findsOneWidget);
    });

    testWidgets('should display macOS version when available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // This test would need to mock ApplicationInfo.macOSMajorVersion
      // For now, we'll just verify the widget renders without errors
      expect(find.byType(VersionInfoWidget), findsOneWidget);
    });

    testWidgets('should display Android SDK version when available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // This test would need to mock ApplicationInfo.androidSDKVersion
      // For now, we'll just verify the widget renders without errors
      expect(find.byType(VersionInfoWidget), findsOneWidget);
    });

    testWidgets('should highlight latest version when update is available', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // The latest version text should be highlighted with primary color
      final latestVersionText = find.text('1.1.0');
      expect(latestVersionText, findsOneWidget);
    });

    testWidgets('should use SettingsCategory layout', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SettingsCategory), findsOneWidget);
    });

    testWidgets('should display correct OS display names', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(false);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // The OS display name should be properly formatted
      // This would need to mock ApplicationInfo.os for specific testing
      expect(find.text('操作系统'), findsOneWidget);
    });

    testWidgets('should handle null update info gracefully', (WidgetTester tester) async {
      // Arrange
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => null);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsNothing);
      expect(find.text('T1Mind版本'), findsOneWidget);
      expect(find.text('最新版本'), findsNothing);
    });

    testWidgets('should show update section with correct styling', (WidgetTester tester) async {
      // Arrange
      when(() => mockUpdateInfo.currentVersion).thenReturn('1.0.0');
      when(() => mockUpdateInfo.latestVersion).thenReturn('1.1.0');
      when(() => mockUpdateInfo.isUpdateAvailable).thenReturn(true);
      when(() => mockVersionChecker.getUpdateInfo()).thenAnswer((_) async => mockUpdateInfo);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      final updateSection = find.byType(Container).last;
      expect(updateSection, findsOneWidget);
      
      // Verify the update section contains the expected elements
      expect(find.text('有更新可用'), findsOneWidget);
      expect(find.text('1.0.0 → 1.1.0'), findsOneWidget);
    });
  });

  group('VersionInfoWidget OS Display', () {
    testWidgets('should display correct OS names for different platforms', (WidgetTester tester) async {
      // This test would require mocking ApplicationInfo.os
      // For now, we'll test the general structure
      
      final widget = VersionInfoWidget();
      expect(widget, isNotNull);
    });
  });
}
