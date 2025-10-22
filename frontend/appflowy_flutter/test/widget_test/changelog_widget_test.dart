import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/shared/changelog/changelog_loader.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/changelog_widget.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

// Mock classes
class MockChangelogLoader extends Mock implements ChangelogLoader {}
class MockBuildContext extends Mock implements BuildContext {}

void main() {
  group('ChangelogWidget', () {
    late MockChangelogLoader mockChangelogLoader;

    setUp(() {
      mockChangelogLoader = MockChangelogLoader();
    });

    Widget createTestWidget() {
      return MaterialApp(
        localizationsDelegates: [
          ...EasyLocalization.of(MockBuildContext())?.delegates ?? [],
        ],
        supportedLocales: const [Locale('en', 'US')],
        home: Scaffold(
          body: ChangelogWidget(),
        ),
      );
    }

    testWidgets('should display loading state initially', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer(
        (_) async {
          await Future.delayed(const Duration(milliseconds: 100));
          return '# Test Changelog';
        },
      );

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pump();

      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('正在加载更新历史...'), findsOneWidget);
    });

    testWidgets('should display changelog content when loaded successfully', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# T1Mind 更新历史

## 版本 1.0.0
- 新增功能A
- 修复问题B

### 代码示例
```dart
void main() {
  print('Hello World');
}
```
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('T1Mind 更新历史'), findsOneWidget);
      expect(find.text('版本 1.0.0'), findsOneWidget);
      expect(find.text('新增功能A'), findsOneWidget);
      expect(find.text('修复问题B'), findsOneWidget);
    });

    testWidgets('should display error state when loading fails', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenThrow(Exception('Loading failed'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('加载失败'), findsOneWidget);
      expect(find.text('无法加载更新历史'), findsOneWidget);
      expect(find.text('重新加载'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('should display empty state when content is empty', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('暂无更新历史'), findsOneWidget);
      expect(find.text('当前版本没有可用的更新历史记录'), findsOneWidget);
      expect(find.byIcon(Icons.history), findsOneWidget);
    });

    testWidgets('should handle reload button tap', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenThrow(Exception('Loading failed'));

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Verify error state is shown
      expect(find.text('加载失败'), findsOneWidget);

      // Arrange for successful reload
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '# Test Changelog');

      // Tap reload button
      await tester.tap(find.text('重新加载'));
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('Test Changelog'), findsOneWidget);
    });

    testWidgets('should format markdown headings correctly', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# Heading 1
## Heading 2
### Heading 3
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('Heading 1'), findsOneWidget);
      expect(find.text('Heading 2'), findsOneWidget);
      expect(find.text('Heading 3'), findsOneWidget);
    });

    testWidgets('should format list items correctly', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# Test
- Item 1
- Item 2
* Item 3
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('Item 1'), findsOneWidget);
      expect(find.text('Item 2'), findsOneWidget);
      expect(find.text('Item 3'), findsOneWidget);
    });

    testWidgets('should format ordered list items correctly', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# Test
1. First item
2. Second item
3. Third item
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('First item'), findsOneWidget);
      expect(find.text('Second item'), findsOneWidget);
      expect(find.text('Third item'), findsOneWidget);
    });

    testWidgets('should format code blocks correctly', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# Test
```dart
void main() {
  print('Hello');
}
```
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text("void main() {\n  print('Hello');\n}"), findsOneWidget);
    });

    testWidgets('should format paragraphs correctly', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# Test
This is a paragraph with some text.

This is another paragraph.
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('This is a paragraph with some text.'), findsOneWidget);
      expect(find.text('This is another paragraph.'), findsOneWidget);
    });

    testWidgets('should enable scrolling for long content', (WidgetTester tester) async {
      // Arrange
      final testContent = '''
# Test
${List.generate(50, (i) => '- Item $i').join('\n')}
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SingleChildScrollView), findsOneWidget);
    });

    testWidgets('should use SettingsCategory layout', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '# Test');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SettingsCategory), findsOneWidget);
    });

    testWidgets('should dispose scroll controller properly', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '# Test');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();
      
      // Remove widget to trigger dispose
      await tester.pumpWidget(const SizedBox.shrink());

      // Assert - No exception should be thrown during dispose
      expect(tester.takeException(), isNull);
    });

    testWidgets('should handle null content gracefully', (WidgetTester tester) async {
      // Arrange
      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => '');

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('暂无更新历史'), findsOneWidget);
    });

    testWidgets('should show scrollbar for scrollable content', (WidgetTester tester) async {
      // Arrange
      final testContent = '''
# Test
${List.generate(20, (i) => '- Item $i').join('\n')}
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.byType(SingleChildScrollView), findsOneWidget);
    });

    testWidgets('should handle mixed content formatting', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# T1Mind 更新历史

## 版本 1.0.0
- 新增功能A
- 修复问题B

### 代码示例
```dart
void main() {
  print('Hello World');
}
```

这是一个段落文本。

1. 有序列表项1
2. 有序列表项2

* 无序列表项1
* 无序列表项2
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('T1Mind 更新历史'), findsOneWidget);
      expect(find.text('版本 1.0.0'), findsOneWidget);
      expect(find.text('新增功能A'), findsOneWidget);
      expect(find.text('修复问题B'), findsOneWidget);
      expect(find.text('代码示例'), findsOneWidget);
      expect(find.text('这是一个段落文本。'), findsOneWidget);
      expect(find.text('有序列表项1'), findsOneWidget);
      expect(find.text('有序列表项2'), findsOneWidget);
      expect(find.text('无序列表项1'), findsOneWidget);
      expect(find.text('无序列表项2'), findsOneWidget);
    });

    testWidgets('should handle empty lines correctly', (WidgetTester tester) async {
      // Arrange
      const testContent = '''
# Test

## Subtitle

- Item 1

- Item 2
''';

      when(() => mockChangelogLoader.getRawChangelog()).thenAnswer((_) async => testContent);

      // Act
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Assert
      expect(find.text('Test'), findsOneWidget);
      expect(find.text('Subtitle'), findsOneWidget);
      expect(find.text('Item 1'), findsOneWidget);
      expect(find.text('Item 2'), findsOneWidget);
    });
  });
}
