import 'dart:convert';
import 'package:appflowy/core/config/kv.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';
import 'package:flutter/foundation.dart';

/// MCP工具服务
/// 
/// 负责从MCP设置中获取已配置的工具，并转换为AI聊天可用的格式
class McpToolsService {
  static const String _prefsKey = 'settings.mcp.endpoints';
  final DartKeyValue _kv = DartKeyValue();

  /// 获取所有可用的MCP工具
  Future<List<McpToolInfo>> getAvailableTools() async {
    try {
      final str = await _kv.get(_prefsKey);
      if (str == null) {
        return [];
      }

      final list = (jsonDecode(str) as List<dynamic>)
          .map((e) => _McpEndpoint.fromJson(e as Map<String, dynamic>))
          .toList();

      final tools = <McpToolInfo>[];
      
      for (final endpoint in list) {
        // 只包含检查通过且有工具的端点
        if (endpoint.checkedOk == true && 
            endpoint.tools != null && 
            endpoint.tools!.isNotEmpty) {
          
          for (final tool in endpoint.tools!) {
            tools.add(McpToolInfo(
              id: '${endpoint.name}_${tool.name}',
              name: tool.name,
              displayName: tool.name,
              description: tool.description ?? '',
              category: _getCategoryFromEndpoint(endpoint),
              status: McpToolStatus.available,
              provider: endpoint.name,
              version: '1.0.0',
              config: {
                'endpoint': endpoint.name,
                'transport': endpoint.transport.name,
                'url': endpoint.url,
                'command': endpoint.command,
                'args': endpoint.args,
                'env': endpoint.env,
                'headers': endpoint.headers,
              },
              schema: tool.toJson(),
              lastChecked: _parseDateTime(endpoint.lastCheckedAt),
              usageCount: 0,
              successCount: 0,
              failureCount: 0,
              averageExecutionTimeMs: 0,
            ));
          }
        }
      }

      return tools;
    } catch (e) {
      if (kDebugMode) {
        debugPrint('Failed to load MCP tools: $e');
      }
      return [];
    }
  }

  /// 获取特定端点的工具
  Future<List<McpToolInfo>> getToolsForEndpoint(String endpointName) async {
    final allTools = await getAvailableTools();
    return allTools.where((tool) => tool.provider == endpointName).toList();
  }

  /// 检查工具是否可用
  Future<bool> isToolAvailable(String toolId) async {
    final tools = await getAvailableTools();
    return tools.any((tool) => tool.id == toolId);
  }

  /// 根据端点类型获取分类
  String _getCategoryFromEndpoint(_McpEndpoint endpoint) {
    // 根据端点名称或描述推断分类
    final name = endpoint.name.toLowerCase();
    final description = (endpoint.description ?? '').toLowerCase();
    
    if (name.contains('file') || description.contains('file')) {
      return 'File Management';
    } else if (name.contains('web') || name.contains('search') || 
               description.contains('web') || description.contains('search')) {
      return 'Web & Search';
    } else if (name.contains('code') || name.contains('git') || 
               description.contains('code') || description.contains('git')) {
      return 'Development';
    } else if (name.contains('database') || name.contains('sql') || 
               description.contains('database') || description.contains('sql')) {
      return 'Database';
    } else {
      return 'General';
    }
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
