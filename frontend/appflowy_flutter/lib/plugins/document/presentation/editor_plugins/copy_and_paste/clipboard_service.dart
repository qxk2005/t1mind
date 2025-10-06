import 'dart:async';
import 'dart:convert';

import 'package:appflowy_backend/log.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// Used for in-app copy and paste without losing the format.
///
/// It's a Json string representing the copied editor nodes.
class InAppJsonFormat {
  static const String applicationId = 'io.appflowy.InAppJsonType';
}

/// Used for table nodes when coping a row or a column.
class TableJsonFormat {
  static const String applicationId = 'io.appflowy.TableJsonType';
}

class ClipboardServiceData {
  const ClipboardServiceData({
    this.plainText,
    this.html,
    this.image,
    this.inAppJson,
    this.tableJson,
  });

  /// The [plainText] is the plain text string.
  ///
  /// It should be used for pasting the plain text from the clipboard.
  final String? plainText;

  /// The [html] is the html string.
  ///
  /// It should be used for pasting the html from the clipboard.
  /// For example, copy the content in the browser, and paste it in the editor.
  final String? html;

  /// The [image] is the image data.
  ///
  /// It should be used for pasting the image from the clipboard.
  /// For example, copy the image in the browser or other apps, and paste it in the editor.
  final (String, Uint8List?)? image;

  /// The [inAppJson] is the json string of the editor nodes.
  ///
  /// It should be used for pasting the content in-app.
  /// For example, pasting the content from document A to document B.
  final String? inAppJson;

  /// The [tableJson] is the json string of the table nodes.
  ///
  /// It only works for the table nodes when coping a row or a column.
  /// Don't use it for another scenario.
  final String? tableJson;
}

class ClipboardService {
  static ClipboardServiceData? _mockData;

  @visibleForTesting
  static void mockSetData(ClipboardServiceData? data) {
    _mockData = data;
  }

  Future<void> setData(ClipboardServiceData data) async {
    if (_mockData != null) {
      return;
    }

    // 使用Flutter内置的剪贴板功能
    if (data.plainText != null) {
      await Clipboard.setData(ClipboardData(text: data.plainText!));
    }
  }

  Future<void> setPlainText(String text) async {
    await Clipboard.setData(ClipboardData(text: text));
  }

  Future<ClipboardServiceData> getData() async {
    if (_mockData != null) {
      return _mockData!;
    }

    // 使用Flutter内置的剪贴板功能
    final clipboardData = await Clipboard.getData('text/plain');
    return ClipboardServiceData(
      plainText: clipboardData?.text,
    );
  }
}

/// The default decode function for the clipboard service.
Future<String?> _defaultDecode(Object value, String platformType) async {
  if (value is List<int>) {
    return utf8.decode(value);
  }
  return null;
}

/// The default encode function for the clipboard service.
Future<Object> _defaultEncode(String value, String platformType) async {
  return utf8.encode(value);
}
