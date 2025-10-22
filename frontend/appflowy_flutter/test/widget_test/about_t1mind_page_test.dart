import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/about_t1mind_page.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/changelog_widget.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/update_check_widget.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/version_info_widget.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category_spacer.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

// Mock classes
class MockBuildContext extends Mock implements BuildContext {}

void main() {
  group('AboutT1MindPage', () {
    Widget createTestWidget() {
      return MaterialApp(
        localizationsDelegates: [
          ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
        ],
        supportedLocales: const [Locale('en', 'US')],
        home: Scaffold(
          body: AboutT1MindPage(),
        ),
      );
    }

    testWidgets('should display all required components', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SettingsBody), findsOneWidget);
      expect(find.byType(VersionInfoWidget), findsOneWidget);
      expect(find.byType(UpdateCheckWidget), findsOneWidget);
      expect(find.byType(ChangelogWidget), findsOneWidget);
    });

    testWidgets('should display page title and description', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于T1Mind'), findsOneWidget);
      expect(find.text('T1Mind版本信息、更新检查和更新历史'), findsOneWidget);
    });

    testWidgets('should display copyright section', (WidgetTester tester) async {
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
      expect(find.text('基于AppFlowy的AI协作工作空间'), findsOneWidget);
      expect(find.text('开源协作工作空间平台'), findsOneWidget);
      expect(find.text('Google开发的跨平台UI框架'), findsOneWidget);
    });

    testWidgets('should use SettingsBody layout', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SettingsBody), findsOneWidget);
    });

    testWidgets('should include category spacers between components', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SettingsCategorySpacer), findsAtLeastNWidgets(2));
    });

    testWidgets('should have proper component order', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      final versionInfoWidget = find.byType(VersionInfoWidget);
      final updateCheckWidget = find.byType(UpdateCheckWidget);
      final changelogWidget = find.byType(ChangelogWidget);

      expect(versionInfoWidget, findsOneWidget);
      expect(updateCheckWidget, findsOneWidget);
      expect(changelogWidget, findsOneWidget);

      // Verify order by checking widget positions
      final versionInfoPosition = tester.getTopLeft(versionInfoWidget);
      final updateCheckPosition = tester.getTopLeft(updateCheckWidget);
      final changelogPosition = tester.getTopLeft(changelogWidget);

      expect(versionInfoPosition.dy, lessThan(updateCheckPosition.dy));
      expect(updateCheckPosition.dy, lessThan(changelogPosition.dy));
    });

    testWidgets('should display copyright items with proper layout', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Find copyright section
      final copyrightSection = find.byType(SettingsCategory).last;
      expect(copyrightSection, findsOneWidget);

      // Verify copyright items are displayed
      expect(find.text('T1Mind'), findsOneWidget);
      expect(find.text('AppFlowy'), findsOneWidget);
      expect(find.text('Flutter'), findsOneWidget);
    });

    testWidgets('should have proper spacing between components', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Check that spacers are present between main components
      expect(find.byType(SettingsCategorySpacer), findsAtLeastNWidgets(2));
      
      // Check that there's bottom spacing
      expect(find.byType(VSpace), findsAtLeastNWidgets(1));
    });

    testWidgets('should handle different screen sizes', (WidgetTester tester) async {
      // Test with different screen sizes
      final testSizes = [
        const Size(400, 600),   // Small screen
        const Size(800, 600),   // Medium screen
        const Size(1200, 800), // Large screen
      ];

      for (final size in testSizes) {
        // Arrange
        tester.view.physicalSize = size;
        tester.view.devicePixelRatio = 1.0;

        // Act
        await tester.pumpWidget(createTestWidget());
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(AboutT1MindPage), findsOneWidget);
        expect(find.byType(VersionInfoWidget), findsOneWidget);
        expect(find.byType(UpdateCheckWidget), findsOneWidget);
        expect(find.byType(ChangelogWidget), findsOneWidget);

        // Clean up
        await tester.pumpWidget(const SizedBox.shrink());
      }
    });

    testWidgets('should display all copyright information correctly', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Check that all copyright items are present
      expect(find.text('T1Mind'), findsOneWidget);
      expect(find.text('基于AppFlowy的AI协作工作空间'), findsOneWidget);
      
      expect(find.text('AppFlowy'), findsOneWidget);
      expect(find.text('开源协作工作空间平台'), findsOneWidget);
      
      expect(find.text('Flutter'), findsOneWidget);
      expect(find.text('Google开发的跨平台UI框架'), findsOneWidget);
    });

    testWidgets('should have proper text styling for copyright items', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Find copyright text widgets
      final t1mindText = find.text('T1Mind');
      final appflowyText = find.text('AppFlowy');
      final flutterText = find.text('Flutter');

      expect(t1mindText, findsOneWidget);
      expect(appflowyText, findsOneWidget);
      expect(flutterText, findsOneWidget);

      // Verify text widgets are properly styled
      final t1mindWidget = tester.widget<Text>(t1mindText);
      expect(t1mindWidget.style?.fontWeight, FontWeight.w500);
    });

    testWidgets('should handle theme changes correctly', (WidgetTester tester) async {
      // Test with light theme
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      expect(find.byType(AboutT1MindPage), findsOneWidget);

      // Test with dark theme
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          localizationsDelegates: [
            ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
          ],
          supportedLocales: const [Locale('en', 'US')],
          home: Scaffold(
            body: AboutT1MindPage(),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(AboutT1MindPage), findsOneWidget);
    });

    testWidgets('should be accessible', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Check that all main components are accessible
      expect(find.byType(VersionInfoWidget), findsOneWidget);
      expect(find.byType(UpdateCheckWidget), findsOneWidget);
      expect(find.byType(ChangelogWidget), findsOneWidget);
      
      // Check that copyright section is accessible
      expect(find.text('版权信息'), findsOneWidget);
    });

    testWidgets('should handle widget disposal correctly', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Verify widgets are present
      expect(find.byType(AboutT1MindPage), findsOneWidget);

      // Remove widget to trigger disposal
      await tester.pumpWidget(const SizedBox.shrink());

      // Assert - No exception should be thrown during disposal
      expect(tester.takeException(), isNull);
    });

    testWidgets('should maintain proper layout structure', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Check that SettingsBody contains all expected children
      final settingsBody = find.byType(SettingsBody);
      expect(settingsBody, findsOneWidget);

      // Verify the structure
      expect(find.byType(VersionInfoWidget), findsOneWidget);
      expect(find.byType(SettingsCategorySpacer), findsAtLeastNWidgets(2));
      expect(find.byType(UpdateCheckWidget), findsOneWidget);
      expect(find.byType(ChangelogWidget), findsOneWidget);
      expect(find.byType(SettingsCategory), findsAtLeastNWidgets(1)); // Copyright section
    });
  });
}
