import 'dart:convert';
import 'package:appflowy/core/config/kv.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';

/// Cache service for MCP check results and AI-generated descriptions
class McpCacheService {
  static const String _cacheKeyPrefix = 'mcp_check_cache_';
  static const String _aiDescriptionsKeyPrefix = 'mcp_ai_descriptions_';
  
  final DartKeyValue _kv = DartKeyValue();

  /// Generate cache key for MCP endpoint
  String _getCacheKey(McpEndpointConfig config) {
    final configHash = config.hashCode.toString();
    return '$_cacheKeyPrefix$configHash';
  }

  /// Generate AI descriptions cache key for MCP endpoint
  String _getAiDescriptionsKey(McpEndpointConfig config) {
    final configHash = config.hashCode.toString();
    return '$_aiDescriptionsKeyPrefix$configHash';
  }

  /// Save MCP check result to cache
  Future<void> saveMcpCheckResult(
    McpEndpointConfig config,
    McpCheckResult result,
  ) async {
    try {
      final cacheKey = _getCacheKey(config);
      final cacheData = {
        'timestamp': DateTime.now().millisecondsSinceEpoch,
        'result': result.toJson(),
      };
      
      await _kv.set(cacheKey, jsonEncode(cacheData));
      print('MCP Cache: Saved check result for ${config.name}');
    } catch (e) {
      print('MCP Cache: Error saving check result: $e');
    }
  }

  /// Load MCP check result from cache
  Future<McpCheckResult?> loadMcpCheckResult(McpEndpointConfig config) async {
    try {
      final cacheKey = _getCacheKey(config);
      final cacheStr = await _kv.get(cacheKey);
      
      if (cacheStr == null || cacheStr.isEmpty) {
        return null;
      }
      
      final cacheData = jsonDecode(cacheStr) as Map<String, dynamic>;
      final timestamp = cacheData['timestamp'] as int;
      final resultData = cacheData['result'] as Map<String, dynamic>;
      
      // Check if cache is still valid (24 hours)
      final cacheAge = DateTime.now().millisecondsSinceEpoch - timestamp;
      const maxCacheAge = 24 * 60 * 60 * 1000; // 24 hours in milliseconds
      
      if (cacheAge > maxCacheAge) {
        print('MCP Cache: Cache expired for ${config.name}');
        await clearMcpCheckResult(config);
        return null;
      }
      
      print('MCP Cache: Loaded check result for ${config.name}');
      return McpCheckResult.fromJson(resultData);
    } catch (e) {
      print('MCP Cache: Error loading check result: $e');
      return null;
    }
  }

  /// Save AI-generated descriptions to cache
  Future<void> saveAiDescriptions(
    McpEndpointConfig config,
    Map<String, String> descriptions,
  ) async {
    try {
      final cacheKey = _getAiDescriptionsKey(config);
      final cacheData = {
        'timestamp': DateTime.now().millisecondsSinceEpoch,
        'descriptions': descriptions,
      };
      
      await _kv.set(cacheKey, jsonEncode(cacheData));
      print('MCP Cache: Saved AI descriptions for ${config.name} (${descriptions.length} tools)');
    } catch (e) {
      print('MCP Cache: Error saving AI descriptions: $e');
    }
  }

  /// Load AI-generated descriptions from cache
  Future<Map<String, String>?> loadAiDescriptions(McpEndpointConfig config) async {
    try {
      final cacheKey = _getAiDescriptionsKey(config);
      final cacheStr = await _kv.get(cacheKey);
      
      if (cacheStr == null || cacheStr.isEmpty) {
        return null;
      }
      
      final cacheData = jsonDecode(cacheStr) as Map<String, dynamic>;
      final timestamp = cacheData['timestamp'] as int;
      final descriptionsData = cacheData['descriptions'] as Map<String, dynamic>;
      
      // Check if cache is still valid (7 days for AI descriptions)
      final cacheAge = DateTime.now().millisecondsSinceEpoch - timestamp;
      const maxCacheAge = 7 * 24 * 60 * 60 * 1000; // 7 days in milliseconds
      
      if (cacheAge > maxCacheAge) {
        print('MCP Cache: AI descriptions cache expired for ${config.name}');
        await clearAiDescriptions(config);
        return null;
      }
      
      final descriptions = descriptionsData.map((key, value) => MapEntry(key, value.toString()));
      print('MCP Cache: Loaded AI descriptions for ${config.name} (${descriptions.length} tools)');
      return descriptions;
    } catch (e) {
      print('MCP Cache: Error loading AI descriptions: $e');
      return null;
    }
  }

  /// Clear MCP check result cache
  Future<void> clearMcpCheckResult(McpEndpointConfig config) async {
    try {
      final cacheKey = _getCacheKey(config);
      await _kv.remove(cacheKey);
      print('MCP Cache: Cleared check result for ${config.name}');
    } catch (e) {
      print('MCP Cache: Error clearing check result: $e');
    }
  }

  /// Clear AI descriptions cache
  Future<void> clearAiDescriptions(McpEndpointConfig config) async {
    try {
      final cacheKey = _getAiDescriptionsKey(config);
      await _kv.remove(cacheKey);
      print('MCP Cache: Cleared AI descriptions for ${config.name}');
    } catch (e) {
      print('MCP Cache: Error clearing AI descriptions: $e');
    }
  }

  /// Clear all cache for an endpoint
  Future<void> clearAllCache(McpEndpointConfig config) async {
    await Future.wait([
      clearMcpCheckResult(config),
      clearAiDescriptions(config),
    ]);
  }

  /// Check if cached result exists
  Future<bool> hasCachedResult(McpEndpointConfig config) async {
    final result = await loadMcpCheckResult(config);
    return result != null;
  }

  /// Check if cached AI descriptions exist
  Future<bool> hasCachedAiDescriptions(McpEndpointConfig config) async {
    final descriptions = await loadAiDescriptions(config);
    return descriptions != null && descriptions.isNotEmpty;
  }
}
