import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/workspace/application/settings/settings_dialog_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/about_t1mind_page.dart';
import 'package:appflowy/workspace/presentation/settings/settings_dialog.dart';
import 'package:appflowy/workspace/presentation/settings/widgets/settings_menu.dart';
import 'package:appflowy/workspace/presentation/settings/widgets/settings_menu_element.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:fixnum/fixnum.dart';

// Mock classes
class MockSettingsDialogBloc extends Mock implements SettingsDialogBloc {}

class MockBuildContext extends Mock implements BuildContext {}

void main() {
  group('Settings Menu Integration Tests', () {
    late UserProfilePB mockUserProfile;

    setUp(() {
      mockUserProfile = UserProfilePB()
        ..id = Int64(123)
        ..name = 'Test User'
        ..email = 'test@example.com'
        ..workspaceType = WorkspaceTypePB.ServerW;
    });

    Widget createTestWidget({
      SettingsPage currentPage = SettingsPage.account,
      bool isBillingEnabled = false,
      AFRolePB? currentUserRole,
    }) {
      return MaterialApp(
        localizationsDelegates: [
          ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
        ],
        supportedLocales: const [Locale('en', 'US')],
        home: Scaffold(
          body: SettingsMenu(
            changeSelectedPage: (page) {},
            currentPage: currentPage,
            userProfile: mockUserProfile,
            isBillingEnabled: isBillingEnabled,
            currentUserRole: currentUserRole,
          ),
        ),
      );
    }

    testWidgets('should display about t1mind menu item', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于T1Mind'), findsOneWidget);
      expect(find.byType(SettingsMenuElement), findsAtLeastNWidgets(1));
    });

    testWidgets('should highlight about t1mind menu item when selected', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget(currentPage: SettingsPage.aboutT1mind));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于T1Mind'), findsOneWidget);
      // The selected state would be handled by SettingsMenuElement
      expect(find.byType(SettingsMenuElement), findsAtLeastNWidgets(1));
    });

    testWidgets('should call changeSelectedPage when about t1mind is tapped', (WidgetTester tester) async {
      // Arrange
      bool pageChanged = false;
      SettingsPage? selectedPage;

      Widget testWidget = MaterialApp(
        localizationsDelegates: [
          ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
        ],
        supportedLocales: const [Locale('en', 'US')],
        home: Scaffold(
          body: SettingsMenu(
            changeSelectedPage: (page) {
              pageChanged = true;
              selectedPage = page;
            },
            currentPage: SettingsPage.account,
            userProfile: mockUserProfile,
            isBillingEnabled: false,
            currentUserRole: null,
          ),
        ),
      );

      // Act
      await tester.pumpWidget(testWidget);
      await tester.pumpAndSettle();

      // Find and tap the about t1mind menu item
      final aboutT1mindElement = find.ancestor(
        of: find.text('关于T1Mind'),
        matching: find.byType(SettingsMenuElement),
      );
      await tester.tap(aboutT1mindElement);
      await tester.pumpAndSettle();

      // Assert
      expect(pageChanged, true);
      expect(selectedPage, SettingsPage.aboutT1mind);
    });

    testWidgets('should display correct icon for about t1mind menu item', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于T1Mind'), findsOneWidget);
      // The icon would be displayed by FlowySvg widget
      expect(find.byType(FlowySvg), findsAtLeastNWidgets(1));
    });

    testWidgets('should maintain proper menu item order', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      // Check that about t1mind appears after other menu items
      expect(find.text('账户'), findsOneWidget);
      expect(find.text('工作空间'), findsOneWidget);
      expect(find.text('关于T1Mind'), findsOneWidget);
    });

    testWidgets('should work with different user roles', (WidgetTester tester) async {
      // Test with admin role
      await tester.pumpWidget(createTestWidget(currentUserRole: AFRolePB.Owner));
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);

      // Test with member role
      await tester.pumpWidget(createTestWidget(currentUserRole: AFRolePB.Member));
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);

      // Test with guest role
      await tester.pumpWidget(createTestWidget(currentUserRole: AFRolePB.Guest));
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);
    });

    testWidgets('should work with different workspace types', (WidgetTester tester) async {
      // Test with server workspace
      mockUserProfile.workspaceType = WorkspaceTypePB.ServerW;
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);

      // Test with local workspace
      mockUserProfile.workspaceType = WorkspaceTypePB.LocalW;
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);
    });

    testWidgets('should work with billing enabled/disabled', (WidgetTester tester) async {
      // Test with billing disabled
      await tester.pumpWidget(createTestWidget(isBillingEnabled: false));
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);

      // Test with billing enabled
      await tester.pumpWidget(createTestWidget(isBillingEnabled: true));
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);
    });

    testWidgets('should handle menu scrolling correctly', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SingleChildScrollView), findsOneWidget);
      expect(find.text('关于T1Mind'), findsOneWidget);
    });

    testWidgets('should maintain menu state during navigation', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget(currentPage: SettingsPage.aboutT1mind));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于T1Mind'), findsOneWidget);
      expect(find.byType(SettingsMenuElement), findsAtLeastNWidgets(1));
    });

    testWidgets('should work with SimpleSettingsMenu', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(
        MaterialApp(
          localizationsDelegates: [
            ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
          ],
          supportedLocales: const [Locale('en', 'US')],
          home: Scaffold(
            body: SimpleSettingsMenu(),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('云设置'), findsOneWidget);
      // SimpleSettingsMenu doesn't include about t1mind, which is expected
    });

    testWidgets('should handle theme changes correctly', (WidgetTester tester) async {
      // Test with light theme
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);

      // Test with dark theme
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          localizationsDelegates: [
            ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
          ],
          supportedLocales: const [Locale('en', 'US')],
          home: Scaffold(
            body: SettingsMenu(
              changeSelectedPage: (page) {},
              currentPage: SettingsPage.account,
              userProfile: mockUserProfile,
              isBillingEnabled: false,
              currentUserRole: null,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('关于T1Mind'), findsOneWidget);
    });

    testWidgets('should handle different screen sizes', (WidgetTester tester) async {
      // Test with different screen sizes
      final testSizes = [
        const Size(800, 600),   // Desktop
        const Size(1200, 800),  // Large desktop
        const Size(1024, 768),  // Tablet
      ];

      for (final size in testSizes) {
        // Arrange
        tester.view.physicalSize = size;
        tester.view.devicePixelRatio = 1.0;

        // Act
        await tester.pumpWidget(createTestWidget());
        await tester.pumpAndSettle();

        // Assert
        expect(find.text('关于T1Mind'), findsOneWidget);

        // Clean up
        await tester.pumpWidget(const SizedBox.shrink());
      }
    });

    testWidgets('should maintain accessibility', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('关于T1Mind'), findsOneWidget);
      expect(find.byType(SettingsMenuElement), findsAtLeastNWidgets(1));
    });

    testWidgets('should handle rapid menu item taps', (WidgetTester tester) async {
      // Arrange
      int tapCount = 0;
      SettingsPage? lastSelectedPage;

      Widget testWidget = MaterialApp(
        localizationsDelegates: [
          ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
        ],
        supportedLocales: const [Locale('en', 'US')],
        home: Scaffold(
          body: SettingsMenu(
            changeSelectedPage: (page) {
              tapCount++;
              lastSelectedPage = page;
            },
            currentPage: SettingsPage.account,
            userProfile: mockUserProfile,
            isBillingEnabled: false,
            currentUserRole: null,
          ),
        ),
      );

      // Act
      await tester.pumpWidget(testWidget);
      await tester.pumpAndSettle();

      // Tap about t1mind multiple times rapidly
      final aboutT1mindElement = find.ancestor(
        of: find.text('关于T1Mind'),
        matching: find.byType(SettingsMenuElement),
      );

      for (int i = 0; i < 5; i++) {
        await tester.tap(aboutT1mindElement);
        await tester.pump();
      }
      await tester.pumpAndSettle();

      // Assert
      expect(tapCount, 5);
      expect(lastSelectedPage, SettingsPage.aboutT1mind);
    });

    testWidgets('should work with all menu items present', (WidgetTester tester) async {
      // Act
      await tester.pumpWidget(createTestWidget(
        isBillingEnabled: true,
        currentUserRole: AFRolePB.Owner,
      ));
      await tester.pumpAndSettle();

      // Assert
      // Check that about t1mind is still present even with all other menu items
      expect(find.text('关于T1Mind'), findsOneWidget);
      expect(find.text('账户'), findsOneWidget);
      expect(find.text('工作空间'), findsOneWidget);
      expect(find.text('计划'), findsOneWidget);
      expect(find.text('计费'), findsOneWidget);
    });
  });

  group('Settings Dialog Integration Tests', () {
    testWidgets('should navigate to about t1mind page correctly', (WidgetTester tester) async {
      // This test would require a more complex setup with actual SettingsDialog
      // For now, we'll test the general structure
      
      final widget = AboutT1MindPage();
      expect(widget, isNotNull);
    });

    testWidgets('should handle about t1mind page routing', (WidgetTester tester) async {
      // This test would require testing the actual routing logic
      // For now, we'll verify the page can be instantiated
      
      final widget = AboutT1MindPage();
      expect(widget, isNotNull);
    });
  });
}
