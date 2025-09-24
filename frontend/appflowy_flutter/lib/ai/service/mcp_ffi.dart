import 'dart:async';
import 'dart:convert';

/// Transport for communicating with MCP server
enum McpTransport { stdio, sse }

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
}

class McpCheckParsed {
  const McpCheckParsed({required this.tools, this.server});
  final List<McpToolSchema> tools;
  final String? server;
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
  /// Simulated check implementation with 20s timeout by default
  static Future< McpCheckResult > check(
    McpEndpointConfig config, {
    Duration timeout = const Duration(seconds: 20),
  }) async {
    return _simulateCheck(config).timeout(timeout);
  }

  static Future<McpCheckResult> _simulateCheck(McpEndpointConfig config) async {
    // Simulate some processing delay
    await Future<void>.delayed(const Duration(milliseconds: 700));

    // Fake tools
    final tools = <McpToolSchema>[
      McpToolSchema(
        name: 'search_docs',
        description: 'Search product documentation',
        fields: const [
          McpToolField(name: 'query', type: 'string', required: true),
          McpToolField(name: 'limit', type: 'integer', defaultValue: 5),
          McpToolField(name: 'include_snippets', type: 'boolean', defaultValue: true),
        ],
      ),
      McpToolSchema(
        name: 'create_issue',
        description: 'Create a bug/feature issue',
        fields: const [
          McpToolField(name: 'title', type: 'string', required: true),
          McpToolField(name: 'body', type: 'string'),
          McpToolField(name: 'labels', type: 'array'),
        ],
      ),
    ];

    final encoder = const JsonEncoder.withIndent('  ');
    final request = {
      'name': config.name,
      'transport': config.transport.name,
      if (config.transport == McpTransport.sse) 'url': config.url,
      if (config.transport == McpTransport.stdio) 'command': config.command,
      if (config.args != null) 'args': config.args,
      if (config.env != null) 'env': config.env,
      if (config.headers != null) 'headers': config.headers,
    };

    final response = {
      'server': 'mcp-mock/0.1.0',
      'tools': tools.map((t) => t.toJson()).toList(),
      'handshake': {
        'protocol': 'mcp/1.0',
        'features': ['tools', 'json-schema'],
      },
    };

    final now = DateTime.now().toUtc().toIso8601String();
    return McpCheckResult(
      ok: true,
      requestJson: encoder.convert(request),
      responseJson: encoder.convert(response),
      toolCount: tools.length,
      parsed: McpCheckParsed(tools: tools, server: 'mcp-mock/0.1.0'),
      checkedAtIso: now,
    );
  }
}


