import 'package:appflowy/plugins/ai_chat/application/chat_input_control_cubit.dart';
import 'package:extended_text_library/extended_text_library.dart';
import 'package:flutter/material.dart';

/// Text span builder that identifies and highlights @文档名称 patterns
/// Extends SpecialTextSpanBuilder to detect @ mentions and style them as blue and bold
class AtMentionTextSpanBuilder extends SpecialTextSpanBuilder {
  AtMentionTextSpanBuilder({
    required this.inputControlCubit,
    this.atMentionTextStyle,
  });

  final ChatInputControlCubit inputControlCubit;
  final TextStyle? atMentionTextStyle;

  @override
  SpecialText? createSpecialText(
    String flag, {
    TextStyle? textStyle,
    SpecialTextGestureTapCallback? onTap,
    int? index,
  }) {
    if (flag.isEmpty) {
      return null;
    }

    // Detect @ symbol
    if (isStart(flag, AtMentionText.flag)) {
      return AtMentionText(
        inputControlCubit: inputControlCubit,
        textStyle: atMentionTextStyle ?? textStyle,
        onTap: onTap,
        start: index! - (AtMentionText.flag.length - 1),
      );
    }

    return null;
  }
}

/// Special text class that handles @文档名称 mentions
/// Validates that the text after @ is a valid document name and styles it accordingly
class AtMentionText extends SpecialText {
  AtMentionText({
    required this.inputControlCubit,
    TextStyle? textStyle,
    SpecialTextGestureTapCallback? onTap,
    required this.start,
  }) : super(
          flag,
          '',
          textStyle,
          onTap: onTap,
        );

  static const String flag = '@';

  final ChatInputControlCubit inputControlCubit;
  final int start;

  /// Determine if the mention has ended
  /// A mention ends when we encounter whitespace, another @, or when the text
  /// matches the pattern used by DocumentMentionExtractor (non-whitespace characters after @)
  @override
  bool isEnd(String value) {
    // Mention pattern is @ followed by non-whitespace characters
    // We need at least @ + one character to form a valid mention
    
    // If value is too short, continue collecting
    if (value.length <= 1) {
      return false;
    }

    // Check if value ends with whitespace (space, newline, tab)
    // If it does and we have @ + at least one character, the mention has ended
    final lastChar = value[value.length - 1];
    if (lastChar == ' ' || lastChar == '\n' || lastChar == '\t') {
      // Extract text before the last whitespace
      final textBeforeWhitespace = value.substring(0, value.length - 1);
      // If we have @ followed by at least one character, it's a potential mention
      if (textBeforeWhitespace.length > 1) {
        return true;
      }
      return false;
    }

    // Check for another @ symbol (nested @ mentions like @doc1@doc2)
    // Only check if we have @ + at least one character before the second @
    if (value.length > 2) {
      final indexOfSecondAt = value.indexOf('@', 1);
      if (indexOfSecondAt > 1) {
        // We have @ + at least one character before second @
        return true;
      }
    }

    // Continue collecting if no whitespace or second @ found
    // The finishText method will validate if it's a real document name
    return false;
  }

  @override
  InlineSpan finishText() {
    final String actualText = toString();
    
    // Extract document name from @documentName
    if (actualText.length <= 1) {
      // Only @ symbol, not a complete mention
      return TextSpan(
        text: actualText,
        style: textStyle,
      );
    }

    final documentName = actualText.substring(1); // Remove @
    
    // Check if it's a valid document name
    final documentId = inputControlCubit.getDocumentIdByName(documentName);
    
    if (documentId != null && documentId.isNotEmpty) {
      // Valid mention - style as blue and bold
      return SpecialTextSpan(
        text: actualText,
        actualText: actualText,
        start: start,
        style: (textStyle ?? const TextStyle()).copyWith(
          color: Colors.blue,
          fontWeight: FontWeight.bold,
        ),
      );
    }

    // Not a valid mention (document doesn't exist or name doesn't match)
    // Return with default style (no highlighting)
    return TextSpan(
      text: actualText,
      style: textStyle,
    );
  }
}

