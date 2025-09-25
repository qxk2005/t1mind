import 'dart:async';
import 'dart:convert';
import 'package:appflowy/ai/service/appflowy_ai_service.dart';
import 'package:appflowy/ai/service/mcp_ffi.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';

/// AI-powered helper for generating human-friendly MCP tool descriptions
class McpAiHelper {
  static const String _systemPrompt = '''
你是一个专业的技术文档助手，专门为MCP（Model Context Protocol）工具生成清晰、易懂的参数说明。

请根据提供的工具JSON Schema，生成简洁明了的中文参数说明。要求：

1. 用通俗易懂的语言解释每个参数的作用
2. 突出必需参数和可选参数的区别
3. 提供参数的实际使用示例（如果适用）
4. 保持简洁，避免过于技术性的术语
5. 按照以下格式输出：

**工具名称**: [工具名称]
**功能描述**: [简短描述工具的主要功能]

**参数说明**:
- **[参数名]** (必需/可选) - [参数说明]
  - 类型: [类型]
  - 示例: [如果适用，提供示例值]

如果工具无需参数，则输出：
**工具名称**: [工具名称]
**功能描述**: [简短描述工具的主要功能]
**参数说明**: 此工具无需任何参数，可直接调用。
''';

  final AppFlowyAIService _aiService;

  McpAiHelper(this._aiService);

  /// Generate AI-powered description for MCP tool schema
  Future<String?> generateToolDescription(McpToolSchema schema) async {
    try {
      print('MCP AI Helper: Generating description for tool: ${schema.name}');
      final prompt = _buildPrompt(schema);
      final completer = Completer<String?>();
      final buffer = StringBuffer();
      
      final result = await _aiService.streamCompletion(
        text: prompt,
        completionType: CompletionTypePB.UserQuestion,
        onStart: () async {
          print('MCP AI Helper: AI completion started for ${schema.name}');
        },
        processMessage: (text) async {
          print('MCP AI Helper: Got message for ${schema.name}: ${text.length} chars');
          buffer.write(text);
        },
        processAssistMessage: (text) async {
          print('MCP AI Helper: Got assist message for ${schema.name}: ${text.length} chars');
          buffer.write(text);
        },
        onEnd: () async {
          final result = buffer.toString().trim();
          print('MCP AI Helper: Completion ended for ${schema.name}, result length: ${result.length}');
          completer.complete(result.isNotEmpty ? result : null);
        },
        onError: (error) {
          print('MCP AI Helper: Error for ${schema.name}: $error');
          completer.complete(null);
        },
        onLocalAIStreamingStateChange: (state) {
          print('MCP AI Helper: Local AI state change for ${schema.name}: $state');
        },
      );

      if (result == null) {
        print('MCP AI Helper: streamCompletion returned null for ${schema.name}');
        return null;
      }

      final description = await completer.future.timeout(
        const Duration(seconds: 30),
        onTimeout: () {
          print('MCP AI Helper: Timeout generating description for ${schema.name}');
          return null;
        },
      );
      
      print('MCP AI Helper: Final description for ${schema.name}: ${description?.length ?? 0} chars');
      return description;
    } catch (e) {
      print('MCP AI Helper: Exception generating description for ${schema.name}: $e');
      return null;
    }
  }

  String _buildPrompt(McpToolSchema schema) {
    final schemaJson = const JsonEncoder.withIndent('  ').convert(schema.toJson());
    
    return '''
$_systemPrompt

请为以下MCP工具生成参数说明：

```json
$schemaJson
```

请生成清晰易懂的中文参数说明。
''';
  }

  /// Build batch prompt for multiple tools
  String _buildBatchPrompt(List<McpToolSchema> tools) {
    final toolsJson = tools.map((tool) => tool.toJson()).toList();
    final batchJson = const JsonEncoder.withIndent('  ').convert(toolsJson);
    
    return '''
你是一个专业的技术文档助手，专门为MCP（Model Context Protocol）工具生成清晰、易懂的参数说明。

请为以下所有MCP工具批量生成参数说明。要求：

1. 用通俗易懂的语言解释每个参数的作用
2. 突出必需参数和可选参数的区别
3. 提供参数的实际使用示例（如果适用）
4. 保持简洁，避免过于技术性的术语
5. 按照以下JSON格式输出，每个工具一个条目：

```json
{
  "工具名称1": "**工具名称**: [工具名称]\\n**功能描述**: [简短描述工具的主要功能]\\n\\n**参数说明**:\\n- **[参数名]** (必需/可选) - [参数说明]\\n  - 类型: [类型]\\n  - 示例: [如果适用，提供示例值]",
  "工具名称2": "**工具名称**: [工具名称]\\n**功能描述**: [简短描述工具的主要功能]\\n\\n**参数说明**: 此工具无需任何参数，可直接调用。"
}
```

以下是需要处理的MCP工具列表：

```json
$batchJson
```

请生成清晰易懂的中文参数说明，并严格按照上述JSON格式输出。
''';
  }

  /// Parse batch AI result into individual tool descriptions
  Map<String, String> _parseBatchResult(String batchResult, List<McpToolSchema> tools) {
    try {
      // Try to extract JSON from the result
      final jsonMatch = RegExp(r'\{[\s\S]*\}').firstMatch(batchResult);
      if (jsonMatch == null) {
        print('MCP AI Helper: No JSON found in batch result');
        return {};
      }
      
      final jsonStr = jsonMatch.group(0)!;
      final decoded = jsonDecode(jsonStr) as Map<String, dynamic>;
      
      final descriptions = <String, String>{};
      for (final tool in tools) {
        final description = decoded[tool.name] as String?;
        if (description != null && description.isNotEmpty) {
          descriptions[tool.name] = description;
        }
      }
      
      return descriptions;
    } catch (e) {
      print('MCP AI Helper: Error parsing batch result: $e');
      
      // Fallback: try to parse line by line
      return _parseBatchResultFallback(batchResult, tools);
    }
  }

  /// Fallback parser for batch results
  Map<String, String> _parseBatchResultFallback(String batchResult, List<McpToolSchema> tools) {
    final descriptions = <String, String>{};
    
    // Try to find tool descriptions by tool names
    for (final tool in tools) {
      final toolPattern = RegExp(
        r'\*\*工具名称\*\*:\s*' + RegExp.escape(tool.name) + r'[\s\S]*?(?=\*\*工具名称\*\*:|$)',
        multiLine: true,
      );
      
      final match = toolPattern.firstMatch(batchResult);
      if (match != null) {
        descriptions[tool.name] = match.group(0)!.trim();
      }
    }
    
    return descriptions;
  }

  /// Generate descriptions for multiple tools in a single batch request
  Future<Map<String, String>> generateToolDescriptionsBatch(
    List<McpToolSchema> tools, {
    void Function(String status)? onProgress,
  }) async {
    if (tools.isEmpty) return {};
    
    try {
      onProgress?.call('正在准备批量生成请求...');
      
      final batchPrompt = _buildBatchPrompt(tools);
      final completer = Completer<String?>();
      final buffer = StringBuffer();
      
      onProgress?.call('正在调用AI生成工具说明...');
      
      final result = await _aiService.streamCompletion(
        text: batchPrompt,
        completionType: CompletionTypePB.UserQuestion,
        onStart: () async {
          print('MCP AI Helper: Batch AI completion started for ${tools.length} tools');
          onProgress?.call('AI开始生成工具说明...');
        },
        processMessage: (text) async {
          buffer.write(text);
          onProgress?.call('AI正在生成中...');
        },
        processAssistMessage: (text) async {
          buffer.write(text);
        },
        onEnd: () async {
          final result = buffer.toString().trim();
          print('MCP AI Helper: Batch completion ended, result length: ${result.length}');
          onProgress?.call('AI生成完成，正在解析结果...');
          completer.complete(result.isNotEmpty ? result : null);
        },
        onError: (error) {
          print('MCP AI Helper: Batch error: $error');
          onProgress?.call('AI生成出错: $error');
          completer.complete(null);
        },
        onLocalAIStreamingStateChange: (state) {
          print('MCP AI Helper: Batch local AI state change: $state');
        },
      );

      if (result == null) {
        print('MCP AI Helper: Batch streamCompletion returned null');
        return {};
      }

      final batchResult = await completer.future.timeout(
        const Duration(minutes: 2), // Longer timeout for batch processing
        onTimeout: () {
          print('MCP AI Helper: Batch timeout');
          onProgress?.call('AI生成超时');
          return null;
        },
      );
      
      if (batchResult == null) {
        return {};
      }
      
      onProgress?.call('正在解析AI生成的结果...');
      final descriptions = _parseBatchResult(batchResult, tools);
      print('MCP AI Helper: Parsed ${descriptions.length} descriptions from batch result');
      
      return descriptions;
    } catch (e) {
      print('MCP AI Helper: Exception in batch generation: $e');
      onProgress?.call('批量生成出错: $e');
      return {};
    }
  }

  /// Generate descriptions for multiple tools with progress callback (legacy method)
  Future<Map<String, String>> generateToolDescriptions(
    List<McpToolSchema> tools, {
    void Function(int completed, int total, String? currentTool)? onProgress,
  }) async {
    final descriptions = <String, String>{};
    int completed = 0;
    
    // Process tools in batches to avoid overwhelming the AI service
    const batchSize = 3;
    for (int i = 0; i < tools.length; i += batchSize) {
      final batch = tools.skip(i).take(batchSize).toList();
      
      for (final tool in batch) {
        onProgress?.call(completed, tools.length, tool.name);
        
        final description = await generateToolDescription(tool);
        if (description != null) {
          descriptions[tool.name] = description;
        }
        
        completed++;
        onProgress?.call(completed, tools.length, null);
      }
      
      // Add a small delay between batches to be respectful to the AI service
      if (i + batchSize < tools.length) {
        await Future.delayed(const Duration(milliseconds: 500));
      }
    }
    
    return descriptions;
  }
}
