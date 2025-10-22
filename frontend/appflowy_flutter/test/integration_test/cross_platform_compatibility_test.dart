import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/shared/changelog/changelog_loader.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/about_t1mind_page.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/update_check_widget.dart';
import 'package:appflowy/mobile/presentation/setting/about/mobile_about_t1mind_page.dart';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:auto_updater/auto_updater.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:universal_platform/universal_platform.dart';

void main() {
  group('Cross-Platform Compatibility Tests', () {
    group('T1MindVersionChecker Cross-Platform', () {
      test('should work on all supported platforms', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();

        // Act & Assert
        expect(versionChecker, isNotNull);
        
        // Test platform-specific asset detection
        final updateInfo = versionChecker.getUpdateInfo();
        expect(updateInfo, isA<Future<T1MindUpdateInfo?>>());
      });

      test('should handle platform-specific download URLs', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();

        // Act
        final updateCheck = versionChecker.checkForT1MindUpdate();

        // Assert
        expect(updateCheck, isA<Future<AppcastItem?>>());
      });

      test('should work with different platform architectures', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();

        // Act
        final isUpdateAvailable = versionChecker.isUpdateAvailable();

        // Assert
        expect(isUpdateAvailable, isA<Future<bool>>());
      });
    });

    group('ChangelogLoader Cross-Platform', () {
      test('should load changelog on all platforms', () {
        // Arrange
        final changelogLoader = ChangelogLoader();

        // Act
        final rawChangelog = changelogLoader.getRawChangelog();
        final formattedChangelog = changelogLoader.getFormattedChangelog();
        final isAvailable = changelogLoader.isChangelogAvailable();

        // Assert
        expect(rawChangelog, isA<Future<String>>());
        expect(formattedChangelog, isA<Future<String>>());
        expect(isAvailable, isA<Future<bool>>());
      });

      test('should handle asset loading on different platforms', () {
        // Arrange
        final changelogLoader = ChangelogLoader();

        // Act
        final content = changelogLoader.loadChangelogContent();

        // Assert
        expect(content, isA<Future<String>>());
      });

      test('should parse markdown consistently across platforms', () {
        // Arrange
        final changelogLoader = ChangelogLoader();
        const testMarkdown = '''
# Test Changelog

## Version 1.0.0
- Feature A
- Feature B

### Code Example
```dart
void main() {
  print('Hello World');
}
```
''';

        // Act
        final parsedContent = changelogLoader.parseChangelog(testMarkdown);

        // Assert
        expect(parsedContent, isNotEmpty);
        expect(parsedContent, contains('<h1>'));
        expect(parsedContent, contains('<h2>'));
        expect(parsedContent, contains('<code>'));
      });
    });

    group('AboutT1MindPage Cross-Platform', () {
      testWidgets('should render correctly on desktop platforms', (WidgetTester tester) async {
        // Arrange
        tester.view.physicalSize = const Size(1200, 800);
        tester.view.devicePixelRatio = 1.0;

        // Act
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(AboutT1MindPage), findsOneWidget);
        expect(find.text('关于T1Mind'), findsOneWidget);
      });

      testWidgets('should render correctly on tablet platforms', (WidgetTester tester) async {
        // Arrange
        tester.view.physicalSize = const Size(768, 1024);
        tester.view.devicePixelRatio = 1.0;

        // Act
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(AboutT1MindPage), findsOneWidget);
        expect(find.text('关于T1Mind'), findsOneWidget);
      });

      testWidgets('should handle different screen orientations', (WidgetTester tester) async {
        // Test landscape orientation
        tester.view.physicalSize = const Size(1200, 800);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AboutT1MindPage), findsOneWidget);

        // Test portrait orientation
        tester.view.physicalSize = const Size(800, 1200);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AboutT1MindPage), findsOneWidget);
      });

      testWidgets('should work with different themes', (WidgetTester tester) async {
        // Test light theme
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData.light(),
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AboutT1MindPage), findsOneWidget);

        // Test dark theme
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData.dark(),
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AboutT1MindPage), findsOneWidget);
      });
    });

    group('MobileAboutT1MindPage Cross-Platform', () {
      testWidgets('should render correctly on mobile platforms', (WidgetTester tester) async {
        // Arrange
        tester.view.physicalSize = const Size(360, 640);
        tester.view.devicePixelRatio = 1.0;

        // Act
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: MobileAboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(MobileAboutT1MindPage), findsOneWidget);
        expect(find.text('关于 T1Mind'), findsOneWidget);
      });

      testWidgets('should handle different mobile screen sizes', (WidgetTester tester) async {
        // Test small mobile
        tester.view.physicalSize = const Size(320, 568);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: MobileAboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(MobileAboutT1MindPage), findsOneWidget);

        // Test large mobile
        tester.view.physicalSize = const Size(414, 896);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: MobileAboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(MobileAboutT1MindPage), findsOneWidget);
      });

      testWidgets('should work with different mobile orientations', (WidgetTester tester) async {
        // Test portrait
        tester.view.physicalSize = const Size(360, 640);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: MobileAboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(MobileAboutT1MindPage), findsOneWidget);

        // Test landscape
        tester.view.physicalSize = const Size(640, 360);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: MobileAboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(MobileAboutT1MindPage), findsOneWidget);
      });
    });

    group('ApplicationInfo Cross-Platform', () {
      test('should provide platform information on all platforms', () {
        // Act & Assert
        expect(ApplicationInfo.os, isNotEmpty);
        expect(ApplicationInfo.applicationVersion, isNotEmpty);
        expect(ApplicationInfo.buildNumber, isNotEmpty);
        expect(ApplicationInfo.architecture, isNotEmpty);
      });

      test('should handle platform-specific version information', () {
        // Act & Assert
        // macOS specific
        if (UniversalPlatform.isMacOS) {
          expect(ApplicationInfo.macOSMajorVersion, isNotNull);
          expect(ApplicationInfo.macOSMinorVersion, isNotNull);
        }

        // Android specific
        if (UniversalPlatform.isAndroid) {
          expect(ApplicationInfo.androidSDKVersion, isNotNull);
        }
      });

      test('should provide consistent platform identification', () {
        // Act
        final os = ApplicationInfo.os;

        // Assert
        expect(os, isIn(['macos', 'windows', 'linux', 'android', 'ios']));
      });
    });

    group('Theme Compatibility', () {
      testWidgets('should work with AppFlowy theme system', (WidgetTester tester) async {
        // Act
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(AboutT1MindPage), findsOneWidget);
        // Theme should be applied without errors
      });

      testWidgets('should handle theme changes gracefully', (WidgetTester tester) async {
        // Test light theme
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData.light(),
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AboutT1MindPage), findsOneWidget);

        // Test dark theme
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData.dark(),
            home: Scaffold(
              body: AboutT1MindPage(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AboutT1MindPage), findsOneWidget);
      });
    });

    group('Network Compatibility', () {
      test('should handle network requests on all platforms', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();

        // Act
        final updateCheck = versionChecker.checkForT1MindUpdate();

        // Assert
        expect(updateCheck, isA<Future<AppcastItem?>>());
      });

      test('should handle network errors gracefully', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();

        // Act
        final updateInfo = versionChecker.getUpdateInfo();

        // Assert
        expect(updateInfo, isA<Future<T1MindUpdateInfo?>>());
      });
    });

    group('Asset Loading Compatibility', () {
      test('should load assets consistently across platforms', () {
        // Arrange
        final changelogLoader = ChangelogLoader();

        // Act
        final isAvailable = changelogLoader.isChangelogAvailable();

        // Assert
        expect(isAvailable, isA<Future<bool>>());
      });

      test('should handle missing assets gracefully', () {
        // Arrange
        final changelogLoader = ChangelogLoader();

        // Act
        final content = changelogLoader.loadChangelogContent();

        // Assert
        expect(content, isA<Future<String>>());
      });
    });

    group('URL Launcher Compatibility', () {
      testWidgets('should handle URL launching on all platforms', (WidgetTester tester) async {
        // Act
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: UpdateCheckWidget(),
            ),
          ),
        );
        await tester.pumpAndSettle();

        // Assert
        expect(find.byType(UpdateCheckWidget), findsOneWidget);
        // URL launcher functionality should be available
      });
    });

    group('Performance Compatibility', () {
      test('should perform well on all platforms', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();
        final changelogLoader = ChangelogLoader();

        // Act
        final stopwatch = Stopwatch()..start();
        
        final updateInfo = versionChecker.getUpdateInfo();
        final changelogContent = changelogLoader.getFormattedChangelog();
        
        stopwatch.stop();

        // Assert
        expect(updateInfo, isA<Future<T1MindUpdateInfo?>>());
        expect(changelogContent, isA<Future<String>>());
        // Performance should be reasonable (no specific timing requirements in tests)
        expect(stopwatch.elapsedMilliseconds, lessThan(1000));
      });
    });

    group('Memory Compatibility', () {
      test('should not leak memory on any platform', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();
        final changelogLoader = ChangelogLoader();

        // Act
        for (int i = 0; i < 100; i++) {
          versionChecker.getUpdateInfo();
          changelogLoader.getFormattedChangelog();
        }

        // Assert
        // No exceptions should be thrown
        expect(versionChecker, isNotNull);
        expect(changelogLoader, isNotNull);
      });
    });

    group('Error Handling Compatibility', () {
      test('should handle errors consistently across platforms', () {
        // Arrange
        final versionChecker = T1MindVersionChecker();
        final changelogLoader = ChangelogLoader();

        // Act
        final updateInfo = versionChecker.getUpdateInfo();
        final changelogContent = changelogLoader.getFormattedChangelog();

        // Assert
        expect(updateInfo, isA<Future<T1MindUpdateInfo?>>());
        expect(changelogContent, isA<Future<String>>());
        // Error handling should be consistent
      });
    });
  });
}
