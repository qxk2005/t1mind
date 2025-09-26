import 'dart:convert';
import 'package:appflowy/core/config/kv.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';
import 'package:flutter/foundation.dart';

/// MCP端点信息
class McpEndpointInfo {
  const McpEndpointInfo({
    required this.id,
    required this.name,
    required this.description,
    required this.isAvailable,
    required this.toolCount,
    this.lastChecked,
  });

  final String id;
  final String name;
  final String description;
  final bool isAvailable;
  final int toolCount;
  final DateTime? lastChecked;

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'description': description,
    'isAvailable': isAvailable,
    'toolCount': toolCount,
    'lastChecked': lastChecked?.toIso8601String(),
  };

  static McpEndpointInfo fromJson(Map<String, dynamic> json) => McpEndpointInfo(
    id: json['id'] as String,
    name: json['name'] as String,
    description: json['description'] as String? ?? '',
    isAvailable: json['isAvailable'] as bool? ?? false,
    toolCount: json['toolCount'] as int? ?? 0,
    lastChecked: json['lastChecked'] != null 
        ? DateTime.tryParse(json['lastChecked'] as String)
        : null,
  );
}

/// MCP端点服务
/// 
/// 负责获取可用的MCP端点信息，用于任务规划时的端点选择
class McpEndpointService {
  static const String _prefsKey = 'settings.mcp.endpoints';
  final DartKeyValue _kv = DartKeyValue();

  /// 获取所有可用的MCP端点
  Future<List<McpEndpointInfo>> getAvailableEndpoints() async {
    try {
      final str = await _kv.get(_prefsKey);
      if (str == null) {
        return [];
      }

      final list = (jsonDecode(str) as List<dynamic>)
          .map((e) => _McpEndpoint.fromJson(e as Map<String, dynamic>))
          .toList();

      final endpoints = <McpEndpointInfo>[];
      
      for (final endpoint in list) {
        // 只包含检查通过的端点
        if (endpoint.checkedOk == true) {
          endpoints.add(McpEndpointInfo(
            id: endpoint.name,
            name: endpoint.name,
            description: endpoint.description ?? '${endpoint.name} MCP端点',
            isAvailable: true,
            toolCount: endpoint.toolsCount ?? 0,
            lastChecked: _parseDateTime(endpoint.lastCheckedAt),
          ));
        }
      }

      return endpoints;
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Failed to load MCP endpoints: $e');
      }
      return [];
    }
  }

  /// 获取端点的详细工具信息
  Future<List<McpToolSchema>> getEndpointTools(String endpointId) async {
    try {
      final str = await _kv.get(_prefsKey);
      if (str == null) {
        return [];
      }

      final list = (jsonDecode(str) as List<dynamic>)
          .map((e) => _McpEndpoint.fromJson(e as Map<String, dynamic>))
          .toList();

      final endpoint = list.firstWhere(
        (e) => e.name == endpointId,
        orElse: () => throw Exception('Endpoint not found: $endpointId'),
      );

      return endpoint.tools ?? [];
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Failed to get endpoint tools for $endpointId: $e');
      }
      return [];
    }
  }

  /// 检查端点是否可用
  Future<bool> isEndpointAvailable(String endpointId) async {
    final endpoints = await getAvailableEndpoints();
    return endpoints.any((endpoint) => endpoint.id == endpointId && endpoint.isAvailable);
  }

  /// 解析日期时间字符串
  DateTime? _parseDateTime(String? dateTimeStr) {
    if (dateTimeStr == null) return null;
    try {
      return DateTime.parse(dateTimeStr);
    } catch (e) {
      return null;
    }
  }
}

/// MCP端点传输类型
enum _McpTransport { stdio, sseHttp, streamableHttp }

/// MCP端点配置（从settings_mcp_view.dart复制）
class _McpEndpoint {
  _McpEndpoint({
    required this.name,
    required this.transport,
    this.url,
    this.command,
    this.args,
    this.disabledOnAndroid = true,
    this.description,
    this.env,
    this.headers,
    this.checkedOk,
    this.toolsCount,
    this.lastCheckedAt,
    this.tools,
  });

  final String name;
  final _McpTransport transport;
  final String? url;
  final String? command;
  final List<String>? args;
  final bool disabledOnAndroid;
  final String? description;
  final Map<String, String>? env;
  final Map<String, String>? headers;
  final bool? checkedOk;
  final int? toolsCount;
  final String? lastCheckedAt;
  final List<McpToolSchema>? tools;

  static _McpTransport _parseTransportFromJson(String transportStr) {
    switch (transportStr) {
      case 'sse': // backward compatibility
        return _McpTransport.sseHttp;
      case 'sseHttp':
        return _McpTransport.sseHttp;
      case 'streamableHttp':
        return _McpTransport.streamableHttp;
      case 'stdio':
      default:
        return _McpTransport.stdio;
    }
  }

  static _McpEndpoint fromJson(Map<String, dynamic> json) => _McpEndpoint(
        name: json['name'] as String,
        transport: _parseTransportFromJson((json['transport'] as String?) ?? 'stdio'),
        url: json['url'] as String?,
        command: json['command'] as String?,
        args: (json['args'] as List?)?.map((e) => e.toString()).toList(),
        disabledOnAndroid: (json['disabledOnAndroid'] as bool?) ?? true,
        description: json['description'] as String?,
        env: (json['env'] as Map?)?.map((k, v) => MapEntry(k.toString(), v.toString())),
        headers: (json['headers'] as Map?)?.map((k, v) => MapEntry(k.toString(), v.toString())),
        checkedOk: json['checkedOk'] as bool?,
        toolsCount: (json['toolsCount'] as num?)?.toInt(),
        lastCheckedAt: json['lastCheckedAt'] as String?,
        tools: (json['tools'] as List?)?.map((t) => McpToolSchema.fromJson(t as Map<String, dynamic>)).toList(),
      );
}
