import 'package:appflowy/plugins/ai_chat/application/chat_input_control_cubit.dart';

/// Data class representing a document mention in the text
class DocumentMention {
  final String documentId;
  final String documentName;
  final int startPosition;
  final int endPosition;

  const DocumentMention({
    required this.documentId,
    required this.documentName,
    required this.startPosition,
    required this.endPosition,
  });

  /// Get the mention text format (@documentName)
  String get mentionText => '@$documentName';

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is DocumentMention &&
        other.documentId == documentId &&
        other.documentName == documentName &&
        other.startPosition == startPosition &&
        other.endPosition == endPosition;
  }

  @override
  int get hashCode {
    return Object.hash(
      documentId,
      documentName,
      startPosition,
      endPosition,
    );
  }

  @override
  String toString() {
    return 'DocumentMention(documentId: $documentId, documentName: $documentName, '
        'startPosition: $startPosition, endPosition: $endPosition)';
  }
}

/// Utility class for extracting and parsing @mention patterns from text
class DocumentMentionExtractor {
  final ChatInputControlCubit _controlCubit;

  DocumentMentionExtractor(this._controlCubit);

  /// Regular expression pattern to match @文档名称 format
  /// Matches @ followed by non-whitespace characters
  /// Supports Chinese, English, numbers, and common punctuation in document names
  static final RegExp _mentionPattern = RegExp(
    r'@([^\s@]+)',
    unicode: true,
  );

  /// Extract all document IDs from text that match @mention format
  /// Returns a list of unique document IDs found in the text
  /// 
  /// Handles edge cases:
  /// - Empty text returns empty list
  /// - Malformed mentions are ignored
  /// - Multiple mentions with same document are deduplicated
  /// - Documents not found are skipped
  List<String> extractDocumentIds(String text) {
    if (text.isEmpty) {
      return [];
    }

    final mentions = parseMentions(text);
    final documentIds = mentions
        .map((mention) => mention.documentId)
        .where((id) => id.isNotEmpty)
        .toSet()
        .toList();

    return documentIds;
  }

  /// Parse all @mention patterns from text
  /// Returns a list of DocumentMention objects with position information
  /// 
  /// Handles edge cases:
  /// - Empty text returns empty list
  /// - Invalid mentions (non-existent documents) are still included but with empty documentId
  /// - Multiple mentions are all extracted
  /// - Overlapping mentions are handled correctly
  List<DocumentMention> parseMentions(String text) {
    if (text.isEmpty) {
      return [];
    }

    final mentions = <DocumentMention>[];
    final matches = _mentionPattern.allMatches(text);

    for (final match in matches) {
      final documentName = match.group(1);
      if (documentName == null || documentName.isEmpty) {
        continue;
      }

      final startPosition = match.start;
      final endPosition = match.end;

      // Get document ID from control cubit
      final documentId = _controlCubit.getDocumentIdByName(documentName) ?? '';

      mentions.add(
        DocumentMention(
          documentId: documentId,
          documentName: documentName,
          startPosition: startPosition,
          endPosition: endPosition,
        ),
      );
    }

    return mentions;
  }

  /// Find the mention at the cursor position
  /// Returns the DocumentMention if cursor is within a mention, null otherwise
  /// 
  /// Handles edge cases:
  /// - Cursor at start of text returns null
  /// - Cursor at end of text checks last mention
  /// - Cursor between mentions returns null
  /// - Cursor within mention returns that mention
  DocumentMention? findMentionAtCursor(String text, int cursorPosition) {
    if (text.isEmpty || cursorPosition < 0) {
      return null;
    }

    // Clamp cursor position to valid range
    final clampedPosition = cursorPosition > text.length
        ? text.length
        : cursorPosition;

    final mentions = parseMentions(text);

    // Find mention that contains the cursor position
    // Cursor is considered "in" a mention if it's between start (inclusive) and end (exclusive)
    // or if it's at the end position (after the mention ends)
    for (final mention in mentions) {
      if (clampedPosition >= mention.startPosition &&
          clampedPosition <= mention.endPosition) {
        return mention;
      }
    }

    return null;
  }
}

