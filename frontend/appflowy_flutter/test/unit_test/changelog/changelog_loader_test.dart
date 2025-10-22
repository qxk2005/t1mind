import 'package:appflowy/shared/changelog/changelog_loader.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

// Mock classes
class MockRootBundle extends Mock implements ServicesBinding {}

void main() {
  group('ChangelogLoader', () {
    late ChangelogLoader changelogLoader;

    setUp(() {
      changelogLoader = ChangelogLoader();
      // Clear cache before each test
      changelogLoader.clearCache();
    });

    group('loadChangelogContent', () {
      test('should return cached content when available', () async {
        // Arrange
        const testContent = '# Test Changelog\n\n## Version 1.0.0\n- Initial release';
        
        // Mock rootBundle to return test content
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              return testContent;
            }
            return null;
          },
        );

        // Act - Load content first time
        final firstResult = await changelogLoader.loadChangelogContent();
        
        // Act - Load content second time (should use cache)
        final secondResult = await changelogLoader.loadChangelogContent();

        // Assert
        expect(firstResult, testContent);
        expect(secondResult, testContent);
        expect(firstResult, equals(secondResult));
      });

      test('should return default content when file is empty', () async {
        // Arrange
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              return ''; // Empty content
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.loadChangelogContent();

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('T1Mind 更新历史'));
        expect(result, contains('感谢您使用T1Mind'));
      });

      test('should return default content when file loading fails', () async {
        // Arrange
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              throw PlatformException(code: 'FILE_NOT_FOUND');
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.loadChangelogContent();

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('T1Mind 更新历史'));
        expect(result, contains('感谢您使用T1Mind'));
      });

      test('should handle general exceptions gracefully', () async {
        // Arrange
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              throw Exception('General error');
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.loadChangelogContent();

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('T1Mind 更新历史'));
      });
    });

    group('parseChangelog', () {
      test('should parse markdown content to HTML', () {
        // Arrange
        const markdownContent = '''
# Test Changelog

## Version 1.0.0
- **Feature**: Added new functionality
- *Bugfix*: Fixed critical issue

### Code Example
```dart
void main() {
  print('Hello World');
}
```
''';

        // Act
        final result = changelogLoader.parseChangelog(markdownContent);

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('<h1>'));
        expect(result, contains('<h2>'));
        expect(result, contains('<strong>'));
        expect(result, contains('<em>'));
        expect(result, contains('<code>'));
      });

      test('should return default content when input is empty', () {
        // Act
        final result = changelogLoader.parseChangelog('');

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('T1Mind 更新历史'));
      });

      test('should handle parsing errors gracefully', () {
        // Arrange
        const invalidMarkdown = '''
# Invalid Markdown
- Unclosed list item
- Another item
  - Nested item without proper indentation
''';

        // Act
        final result = changelogLoader.parseChangelog(invalidMarkdown);

        // Assert
        expect(result, isNotEmpty);
        // Should still return some formatted content
        expect(result, contains('<'));
      });

      test('should format plain text when markdown parsing fails', () {
        // Arrange
        const plainText = '''
# Title
This is a paragraph.

## Subtitle
Another paragraph with **bold** text.
''';

        // Act
        final result = changelogLoader.parseChangelog(plainText);

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('<h1>Title</h1>'));
        expect(result, contains('<h2>Subtitle</h2>'));
        expect(result, contains('<p>'));
      });
    });

    group('getFormattedChangelog', () {
      test('should return formatted HTML content', () async {
        // Arrange
        const testContent = '# Test Changelog\n\n## Version 1.0.0\n- New feature';
        
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              return testContent;
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.getFormattedChangelog();

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('<h1>'));
        expect(result, contains('<h2>'));
      });

      test('should return default content when loading fails', () async {
        // Arrange
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              throw Exception('Loading failed');
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.getFormattedChangelog();

        // Assert
        expect(result, isNotEmpty);
        expect(result, contains('T1Mind 更新历史'));
      });
    });

    group('getRawChangelog', () {
      test('should return raw markdown content', () async {
        // Arrange
        const testContent = '# Raw Changelog\n\n## Version 1.0.0\n- Raw content';
        
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              return testContent;
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.getRawChangelog();

        // Assert
        expect(result, testContent);
        expect(result, contains('# Raw Changelog'));
        expect(result, contains('## Version 1.0.0'));
      });
    });

    group('isChangelogAvailable', () {
      test('should return true when changelog file exists', () async {
        // Arrange
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              return 'File content';
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.isChangelogAvailable();

        // Assert
        expect(result, true);
      });

      test('should return false when changelog file does not exist', () async {
        // Arrange
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              throw PlatformException(code: 'FILE_NOT_FOUND');
            }
            return null;
          },
        );

        // Act
        final result = await changelogLoader.isChangelogAvailable();

        // Assert
        expect(result, false);
      });
    });

    group('clearCache', () {
      test('should clear cached content', () async {
        // Arrange
        const testContent = '# Cached Content';
        
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
          const MethodChannel('flutter/assets'),
          (MethodCall methodCall) async {
            if (methodCall.method == 'loadString' && 
                methodCall.arguments == 'assets/changelog.md') {
              return testContent;
            }
            return null;
          },
        );

        // Load content to cache it
        await changelogLoader.loadChangelogContent();

        // Act
        changelogLoader.clearCache();

        // Assert - Next load should not use cache
        final result = await changelogLoader.loadChangelogContent();
        expect(result, testContent);
      });
    });
  });

  group('ChangelogEntry', () {
    test('should create instance with all required fields', () {
      // Arrange
      final entry = ChangelogEntry(
        version: '1.0.0',
        date: DateTime(2024, 1, 1),
        changes: ['Added feature A', 'Fixed bug B'],
        type: ChangelogEntryType.feature,
      );

      // Assert
      expect(entry.version, '1.0.0');
      expect(entry.date, DateTime(2024, 1, 1));
      expect(entry.changes, ['Added feature A', 'Fixed bug B']);
      expect(entry.type, ChangelogEntryType.feature);
    });

    test('should have correct toString representation', () {
      // Arrange
      final entry = ChangelogEntry(
        version: '1.0.0',
        date: DateTime(2024, 1, 1),
        changes: ['Added feature A', 'Fixed bug B'],
        type: ChangelogEntryType.feature,
      );

      // Act
      final stringRepresentation = entry.toString();

      // Assert
      expect(stringRepresentation, contains('ChangelogEntry'));
      expect(stringRepresentation, contains('version: 1.0.0'));
      expect(stringRepresentation, contains('changes: 2 items'));
      expect(stringRepresentation, contains('type: ChangelogEntryType.feature'));
    });

    test('should use default type when not specified', () {
      // Arrange
      final entry = ChangelogEntry(
        version: '1.0.0',
        changes: ['Change'],
      );

      // Assert
      expect(entry.type, ChangelogEntryType.feature);
    });
  });

  group('ChangelogEntryType', () {
    test('should have all expected enum values', () {
      // Assert
      expect(ChangelogEntryType.values, contains(ChangelogEntryType.feature));
      expect(ChangelogEntryType.values, contains(ChangelogEntryType.bugfix));
      expect(ChangelogEntryType.values, contains(ChangelogEntryType.breaking));
      expect(ChangelogEntryType.values, contains(ChangelogEntryType.security));
      expect(ChangelogEntryType.values, contains(ChangelogEntryType.performance));
      expect(ChangelogEntryType.values, contains(ChangelogEntryType.other));
    });
  });

  group('ChangelogParseResult', () {
    test('should create instance with all required fields', () {
      // Arrange
      final entries = [
        ChangelogEntry(
          version: '1.0.0',
          changes: ['Feature A'],
        ),
      ];
      const rawContent = '# Changelog';
      const formattedContent = '<h1>Changelog</h1>';

      final result = ChangelogParseResult(
        entries: entries,
        rawContent: rawContent,
        formattedContent: formattedContent,
        isFromCache: true,
      );

      // Assert
      expect(result.entries, entries);
      expect(result.rawContent, rawContent);
      expect(result.formattedContent, formattedContent);
      expect(result.isFromCache, true);
    });

    test('should have correct toString representation', () {
      // Arrange
      final entries = [
        ChangelogEntry(
          version: '1.0.0',
          changes: ['Feature A'],
        ),
      ];
      const rawContent = '# Changelog';
      const formattedContent = '<h1>Changelog</h1>';

      final result = ChangelogParseResult(
        entries: entries,
        rawContent: rawContent,
        formattedContent: formattedContent,
        isFromCache: true,
      );

      // Act
      final stringRepresentation = result.toString();

      // Assert
      expect(stringRepresentation, contains('ChangelogParseResult'));
      expect(stringRepresentation, contains('entries: 1'));
      expect(stringRepresentation, contains('formattedContent: 17 chars'));
      expect(stringRepresentation, contains('isFromCache: true'));
    });

    test('should use default isFromCache value when not specified', () {
      // Arrange
      final entries = <ChangelogEntry>[];
      const rawContent = '';
      const formattedContent = '';

      final result = ChangelogParseResult(
        entries: entries,
        rawContent: rawContent,
        formattedContent: formattedContent,
      );

      // Assert
      expect(result.isFromCache, false);
    });
  });
}
