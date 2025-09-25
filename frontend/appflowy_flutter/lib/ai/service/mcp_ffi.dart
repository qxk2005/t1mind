import 'dart:async';
import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:typed_data';
import 'dart:isolate';
import 'package:flutter/foundation.dart';
import 'package:ffi/ffi.dart' as pkgffi;
import 'package:appflowy_backend/ffi.dart' as rust;

/// Transport for communicating with MCP server
enum McpTransport { stdio, sse, streamableHttp }

/// Minimal endpoint configuration used for check/invoke
class McpEndpointConfig {
  const McpEndpointConfig({
    required this.name,
    required this.transport,
    this.url,
    this.command,
    this.args,
    this.env,
    this.headers,
  });

  final String name;
  final McpTransport transport;
  final String? url;
  final String? command;
  final List<String>? args;
  final Map<String, String>? env;
  final Map<String, String>? headers;
}

class McpToolField {
  const McpToolField({
    required this.name,
    required this.type,
    this.required = false,
    this.defaultValue,
  });

  final String name;
  final String type;
  final bool required;
  final Object? defaultValue;

  Map<String, Object?> toJson() => {
        'name': name,
        'type': type,
        'required': required,
        if (defaultValue != null) 'default': defaultValue,
      };
}

class McpToolSchema {
  const McpToolSchema({
    required this.name,
    this.description,
    required this.fields,
  });

  final String name;
  final String? description;
  final List<McpToolField> fields;

  Map<String, Object?> toJson() => {
        'name': name,
        if (description != null) 'description': description,
        'input': {
          'type': 'object',
          'properties': {
            for (final f in fields) f.name: {
              'type': f.type,
              if (f.defaultValue != null) 'default': f.defaultValue,
            }
          },
          'required': [for (final f in fields.where((e) => e.required)) f.name],
        },
      };

  static McpToolSchema fromJson(Map<String, dynamic> json) {
    final input = json['input'] as Map<String, dynamic>? ?? {};
    final properties = input['properties'] as Map<String, dynamic>? ?? {};
    final required = (input['required'] as List?)?.cast<String>() ?? <String>[];
    
    final fields = properties.entries.map((entry) {
      final fieldName = entry.key;
      final fieldData = entry.value as Map<String, dynamic>? ?? {};
      return McpToolField(
        name: fieldName,
        type: fieldData['type'] as String? ?? 'string',
        required: required.contains(fieldName),
        defaultValue: fieldData['default'],
      );
    }).toList();

    return McpToolSchema(
      name: json['name'] as String,
      description: json['description'] as String?,
      fields: fields,
    );
  }
}

class McpCheckParsed {
  const McpCheckParsed({required this.tools, this.server});
  final List<McpToolSchema> tools;
  final String? server;

  Map<String, Object?> toJson() => {
        'tools': tools.map((t) => t.toJson()).toList(),
        if (server != null) 'server': server,
      };

  static McpCheckParsed fromJson(Map<String, dynamic> json) {
    final toolsData = json['tools'] as List? ?? [];
    final tools = toolsData
        .map((t) => McpToolSchema.fromJson(t as Map<String, dynamic>))
        .toList();
    
    return McpCheckParsed(
      tools: tools,
      server: json['server'] as String?,
    );
  }
}

class McpCheckResult {
  const McpCheckResult({
    required this.ok,
    required this.requestJson,
    required this.responseJson,
    required this.toolCount,
    this.parsed,
    this.error,
    required this.checkedAtIso,
  });

  final bool ok;
  final String requestJson;
  final String responseJson;
  final int toolCount;
  final McpCheckParsed? parsed;
  final String? error;
  final String checkedAtIso;

  Map<String, Object?> toJson() => {
        'ok': ok,
        'requestJson': requestJson,
        'responseJson': responseJson,
        'toolCount': toolCount,
        if (parsed != null) 'parsed': parsed!.toJson(),
        if (error != null) 'error': error,
        'checkedAtIso': checkedAtIso,
      };

  static McpCheckResult fromJson(Map<String, dynamic> json) {
    return McpCheckResult(
      ok: json['ok'] as bool,
      requestJson: json['requestJson'] as String,
      responseJson: json['responseJson'] as String,
      toolCount: json['toolCount'] as int,
      parsed: json['parsed'] != null
          ? McpCheckParsed.fromJson(json['parsed'] as Map<String, dynamic>)
          : null,
      error: json['error'] as String?,
      checkedAtIso: json['checkedAtIso'] as String,
    );
  }
}

class McpCheckSummary {
  const McpCheckSummary({
    required this.ok,
    required this.toolCount,
    required this.checkedAtIso,
  });

  final bool ok;
  final int toolCount;
  final String checkedAtIso;
}

class McpFfi {
  static Future<McpCheckResult> check(
    McpEndpointConfig config, {
    Duration timeout = const Duration(seconds: 20),
  }) async {
    if (kDebugMode) {
      debugPrint('[MCP] check start: transport=${config.transport}, ' 
          'url=${config.url ?? ''}, cmd=${config.command ?? ''}, ' 
          'args=${config.args?.join(' ') ?? ''}, ' 
          'envKeys=${config.env?.keys.toList() ?? const []}, ' 
          'hdrKeys=${config.headers?.keys.toList() ?? const []}');
    }
    switch (config.transport) {
      case McpTransport.sse:
        final r = await _checkSseFfi(config).timeout(timeout);
        if (kDebugMode) {
          debugPrint('[MCP] check end (sse): ok=${r.ok}, tools=${r.toolCount}');
        }
        return r;
      case McpTransport.streamableHttp:
        final r = await _checkStreamableHttpFfi(config).timeout(timeout);
        if (kDebugMode) {
          debugPrint('[MCP] check end (streamable-http): ok=${r.ok}, tools=${r.toolCount}');
        }
        return r;
      case McpTransport.stdio:
        final r = await _checkStdioFfi(config).timeout(timeout);
        if (kDebugMode) {
          debugPrint('[MCP] check end (stdio): ok=${r.ok}, tools=${r.toolCount}');
        }
        return r;
    }
  }

  static Future<McpCheckResult> _checkSseFfi(McpEndpointConfig config) async {
    final url = (config.url ?? '').trim();
    if (url.isEmpty) {
      throw ArgumentError('SSE url is empty');
    }

    final headersJson = jsonEncode(config.headers ?? const <String, String>{});
    if (kDebugMode) {
      debugPrint('[MCP] sse request → tools/list url=$url headersKeys=${config.headers?.keys.toList() ?? const []}');
    }

    final Map<String, Object?> map = await Isolate.run(() => _ffiCheckSseWorker(url, headersJson));
      final ok = (map['ok'] as bool?) ?? false;
      final requestJson = (map['requestJson'] as String?) ?? '{}';
      final responseJson = (map['responseJson'] as String?) ?? '{}';
      final toolCount = (map['toolCount'] as num?)?.toInt() ?? 0;
      final checkedAtIso = (map['checkedAtIso'] as String?) ?? DateTime.now().toUtc().toIso8601String();
      final server = map['server'] as String?;

      final parsed = _buildParsedFromResponseJson(responseJson, server: server);

      return McpCheckResult(
        ok: ok,
        requestJson: requestJson,
        responseJson: responseJson,
        toolCount: toolCount,
        parsed: parsed,
        checkedAtIso: checkedAtIso,
      );
    
  }

  static Future<McpCheckResult> _checkStreamableHttpFfi(McpEndpointConfig config) async {
    final url = (config.url ?? '').trim();
    if (url.isEmpty) {
      throw ArgumentError('Streamable HTTP url is empty');
    }

    final headersJson = jsonEncode(config.headers ?? const <String, String>{});
    if (kDebugMode) {
      debugPrint('[MCP] streamable-http request → tools/list url=$url headersKeys=${config.headers?.keys.toList() ?? const []}');
    }

    final Map<String, Object?> map = await Isolate.run(() => _ffiCheckStreamableHttpWorker(url, headersJson));
      final ok = (map['ok'] as bool?) ?? false;
      final requestJson = (map['requestJson'] as String?) ?? '{}';
      final responseJson = (map['responseJson'] as String?) ?? '{}';
      final toolCount = (map['toolCount'] as num?)?.toInt() ?? 0;
      final checkedAtIso = (map['checkedAtIso'] as String?) ?? DateTime.now().toUtc().toIso8601String();
      final server = map['server'] as String?;

      final parsed = _buildParsedFromResponseJson(responseJson, server: server);

      return McpCheckResult(
        ok: ok,
        requestJson: requestJson,
        responseJson: responseJson,
        toolCount: toolCount,
        parsed: parsed,
        checkedAtIso: checkedAtIso,
      );
    
  }

  static McpCheckParsed _buildParsedFromResponseJson(String responseJson, {String? server}) {
    try {
      final Map<String, Object?> resp = jsonDecode(responseJson) as Map<String, Object?>;
      final Object? toolsRaw = resp['tools'] ?? (resp['result'] is Map
          ? (resp['result'] as Map)['tools']
          : null);
      final List<McpToolSchema> tools = <McpToolSchema>[];
      if (toolsRaw is List) {
        for (final t in toolsRaw) {
          if (t is Map<String, Object?>) {
            final name = (t['name'] ?? '').toString();
            final description = t['description']?.toString();
            final input = (t['input'] as Map?)?.cast<String, Object?>();
            final props = (input?['properties'] as Map?)?.cast<String, Object?>() ?? const {};
            final requiredList = (input?['required'] is List)
                ? List<String>.from(input?['required'] as List)
                : const <String>[];
            final fields = <McpToolField>[];
            props.forEach((key, val) {
              if (val is Map<String, Object?>) {
                final type = (val['type'] ?? 'string').toString();
                final def = val['default'];
                final isReq = requiredList.contains(key);
                fields.add(McpToolField(name: key, type: type, required: isReq, defaultValue: def));
              }
            });
            tools.add(McpToolSchema(name: name, description: description, fields: fields));
          }
        }
      }
      return McpCheckParsed(tools: tools, server: server);
    } catch (_) {
      return McpCheckParsed(tools: const <McpToolSchema>[], server: server);
    }
  }

  static Future<McpCheckResult> _checkStdioFfi(McpEndpointConfig config) async {
    final command = (config.command ?? '').trim();
    if (command.isEmpty) {
      throw ArgumentError('STDIO command is empty');
    }
    if (kDebugMode) {
      debugPrint('[MCP] stdio spawn → $command ${config.args?.join(' ') ?? ''} envKeys=${config.env?.keys.toList() ?? const []}');
    }
    final argsJson = jsonEncode(config.args ?? const <String>[]);
    final envJson = jsonEncode(config.env ?? const <String, String>{});

    final Map<String, Object?> map = await Isolate.run(() => _ffiCheckStdioWorker(command, argsJson, envJson));
      final ok = (map['ok'] as bool?) ?? false;
      final requestJson = (map['requestJson'] as String?) ?? '{}';
      final responseJson = (map['responseJson'] as String?) ?? '{}';
      final toolCount = (map['toolCount'] as num?)?.toInt() ?? 0;
      final checkedAtIso = (map['checkedAtIso'] as String?) ?? DateTime.now().toUtc().toIso8601String();
      final server = map['server'] as String?;
      final parsed = _buildParsedFromResponseJson(responseJson, server: server);

      return McpCheckResult(
        ok: ok,
        requestJson: requestJson,
        responseJson: responseJson,
        toolCount: toolCount,
        parsed: parsed,
        checkedAtIso: checkedAtIso,
      );
    
  }

  // _simulateCheck removed after real implementations for SSE and STDIO
}

// Run blocking FFI in background isolates to avoid UI jank during checks
Map<String, Object?> _ffiCheckSseWorker(String url, String headersJson) {
  final urlPtr = url.toNativeUtf8(allocator: pkgffi.malloc);
  final headersPtr = headersJson.toNativeUtf8(allocator: pkgffi.malloc);
  try {
    final ffi.Pointer<ffi.Uint8> ptr = rust.mcp_check_sse(urlPtr, headersPtr);
    final int len = (ptr.elementAt(0).value << 24) |
        (ptr.elementAt(1).value << 16) |
        (ptr.elementAt(2).value << 8) |
        (ptr.elementAt(3).value);
    final int totalLen = len + 4;
    final Uint8List all = ptr.asTypedList(totalLen);
    final Uint8List body = all.sublist(4, totalLen);
    final String bodyStr = utf8.decode(body);
    rust.free_bytes(ptr, totalLen);
    return jsonDecode(bodyStr) as Map<String, Object?>;
  } finally {
    pkgffi.malloc.free(urlPtr);
    pkgffi.malloc.free(headersPtr);
  }
}

Map<String, Object?> _ffiCheckStreamableHttpWorker(String url, String headersJson) {
  final urlPtr = url.toNativeUtf8(allocator: pkgffi.malloc);
  final headersPtr = headersJson.toNativeUtf8(allocator: pkgffi.malloc);
  try {
    final ffi.Pointer<ffi.Uint8> ptr = rust.mcp_check_streamable_http(urlPtr, headersPtr);
    final int len = (ptr.elementAt(0).value << 24) |
        (ptr.elementAt(1).value << 16) |
        (ptr.elementAt(2).value << 8) |
        (ptr.elementAt(3).value);
    final int totalLen = len + 4;
    final Uint8List all = ptr.asTypedList(totalLen);
    final Uint8List body = all.sublist(4, totalLen);
    final String bodyStr = utf8.decode(body);
    rust.free_bytes(ptr, totalLen);
    return jsonDecode(bodyStr) as Map<String, Object?>;
  } finally {
    pkgffi.malloc.free(urlPtr);
    pkgffi.malloc.free(headersPtr);
  }
}

Map<String, Object?> _ffiCheckStdioWorker(String command, String argsJson, String envJson) {
  final cmdPtr = command.toNativeUtf8(allocator: pkgffi.malloc);
  final argsPtr = argsJson.toNativeUtf8(allocator: pkgffi.malloc);
  final envPtr = envJson.toNativeUtf8(allocator: pkgffi.malloc);
  try {
    final ffi.Pointer<ffi.Uint8> ptr = rust.mcp_check_stdio(cmdPtr, argsPtr, envPtr);
    final int len = (ptr.elementAt(0).value << 24) |
        (ptr.elementAt(1).value << 16) |
        (ptr.elementAt(2).value << 8) |
        (ptr.elementAt(3).value);
    final int totalLen = len + 4;
    final Uint8List all = ptr.asTypedList(totalLen);
    final Uint8List body = all.sublist(4, totalLen);
    final String bodyStr = utf8.decode(body);
    rust.free_bytes(ptr, totalLen);
    return jsonDecode(bodyStr) as Map<String, Object?>;
  } finally {
    pkgffi.malloc.free(cmdPtr);
    pkgffi.malloc.free(argsPtr);
    pkgffi.malloc.free(envPtr);
  }
}


