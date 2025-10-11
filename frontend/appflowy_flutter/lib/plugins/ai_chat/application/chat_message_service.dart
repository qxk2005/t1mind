import 'dart:convert';

import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy/workspace/application/view/view_ext.dart';
import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-document/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/protobuf.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:nanoid/nanoid.dart';

/// Indicate file source from appflowy document
const appflowySource = "appflowy";

List<ChatFile> fileListFromMessageMetadata(
  Map<String, dynamic>? map,
) {
  final List<ChatFile> metadata = [];
  if (map != null) {
    for (final entry in map.entries) {
      if (entry.value is ChatFile) {
        metadata.add(entry.value);
      }
    }
  }

  return metadata;
}

List<ChatFile> chatFilesFromMetadataString(String? s) {
  if (s == null || s.isEmpty || s == "null") {
    return [];
  }

  final metadataJson = jsonDecode(s);
  if (metadataJson is Map<String, dynamic>) {
    final file = chatFileFromMap(metadataJson);
    if (file != null) {
      return [file];
    } else {
      return [];
    }
  } else if (metadataJson is List) {
    return metadataJson
        .map((e) => e as Map<String, dynamic>)
        .map(chatFileFromMap)
        .where((file) => file != null)
        .cast<ChatFile>()
        .toList();
  } else {
    Log.error("Invalid metadata: $metadataJson");
    return [];
  }
}

ChatFile? chatFileFromMap(Map<String, dynamic>? map) {
  if (map == null) return null;

  final filePath = map['source'] as String?;
  final fileName = map['name'] as String?;

  if (filePath == null || fileName == null) {
    return null;
  }
  return ChatFile.fromFilePath(filePath);
}

class MetadataCollection {
  MetadataCollection({
    required this.sources,
    this.progress,
    this.reasoningDelta,
    this.rawMetadata,
  });
  final List<ChatMessageRefSource> sources;
  final AIChatProgress? progress;
  final String? reasoningDelta;
  // 🔧 新增字段：原始 Metadata 用于解析工具调用和任务规划
  final Map<String, dynamic>? rawMetadata;
}

MetadataCollection parseMetadata(String? s) {
  if (s == null || s.trim().isEmpty || s.toLowerCase() == "null") {
    return MetadataCollection(sources: []);
  }

  final List<ChatMessageRefSource> metadata = [];
  AIChatProgress? progress;
  String? reasoningDelta;
  Map<String, dynamic>? rawMetadata;

  try {
    final dynamic decodedJson = jsonDecode(s);
    if (decodedJson == null) {
      return MetadataCollection(sources: []);
    }

    // 🔧 保存原始 Metadata
    if (decodedJson is Map<String, dynamic>) {
      rawMetadata = Map<String, dynamic>.from(decodedJson);
    } else if (decodedJson is List && decodedJson.isNotEmpty && decodedJson.first is Map) {
      rawMetadata = Map<String, dynamic>.from(decodedJson.first as Map);
    }

    void processMap(Map<String, dynamic> map) {
      if (map.containsKey("step") && map["step"] != null) {
        progress = AIChatProgress.fromJson(map);
      } else if (map.containsKey("id") && map["id"] != null) {
        metadata.add(ChatMessageRefSource.fromJson(map));
      } else if (map.containsKey("reasoning_delta")) {
        // 处理推理过程的增量数据
        final delta = map["reasoning_delta"]?.toString();
        if (delta != null && delta.isNotEmpty) {
          reasoningDelta = delta;
          // Log.debug("📝 [REALTIME] Received reasoning delta: '$delta'");
        }
      } else if (map.containsKey("tool_call")) {
        // 🔧 处理工具调用数据，特别是网络搜索结果
        final toolCallData = map["tool_call"] as Map<String, dynamic>?;
        if (toolCallData != null) {
          final toolName = toolCallData["tool_name"] as String?;
          final result = toolCallData["result"] as String?;
          final status = toolCallData["status"] as String?;
          
          // 检查是否是成功的网络搜索调用
          if (toolName == "web_search" && status == "success" && result != null) {
            Log.info("🔍 [WEB_SEARCH] Parsing web search result from tool_call");
            // 从result字符串中提取URL引用
            final citations = _extractCitationsFromSearchResult(result);
            metadata.addAll(citations);
            Log.info("🔍 [WEB_SEARCH] Extracted ${citations.length} citations");
          }
        }
      } else {
        Log.info("Unsupported metadata format: $map");
      }
    }

    if (decodedJson is Map<String, dynamic>) {
      processMap(decodedJson);
    } else if (decodedJson is List) {
      for (final element in decodedJson) {
        if (element is Map<String, dynamic>) {
          processMap(element);
        } else {
          Log.error("Invalid metadata element: $element");
        }
      }
    } else {
      Log.error("Invalid metadata format: $decodedJson");
    }
  } catch (e, stacktrace) {
    Log.error("Failed to parse metadata: $e, input: $s");
    Log.debug(stacktrace.toString());
  }

  return MetadataCollection(
    sources: metadata, 
    progress: progress, 
    reasoningDelta: reasoningDelta,
    rawMetadata: rawMetadata,
  );
}

/// 从网络搜索结果字符串中提取引用信息
/// 
/// 搜索结果格式示例：
/// ```
/// 搜索结果 (查询词):
/// 1. 标题1
///    链接: https://example.com/1
/// 2. 标题2
///    链接: https://example.com/2
/// ```
List<ChatMessageRefSource> _extractCitationsFromSearchResult(String result) {
  final List<ChatMessageRefSource> citations = [];
  
  try {
    // 使用正则表达式匹配引用模式
    // 匹配格式: 数字. 标题\n   链接: URL
    final pattern = RegExp(
      r'(\d+)\.\s+([^\n]+)\s+链接:\s+(https?://[^\s]+)',
      multiLine: true,
    );
    
    final matches = pattern.allMatches(result);
    
    for (final match in matches) {
      final index = match.group(1); // 引用序号
      final title = match.group(2)?.trim(); // 标题
      final url = match.group(3)?.trim(); // URL
      
      if (title != null && url != null) {
        citations.add(ChatMessageRefSource(
          id: url,
          name: title,
          source: 'web', // 标记为网络来源
        ));
        Log.debug("🔍 [WEB_SEARCH] Extracted citation $index: $title -> $url");
      }
    }
  } catch (e) {
    Log.error("Failed to extract citations from search result: $e");
  }
  
  return citations;
}

Future<List<ChatMessageMetaPB>> metadataPBFromMetadata(
  Map<String, dynamic>? map,
) async {
  if (map == null) return [];

  final List<ChatMessageMetaPB> metadata = [];

  for (final value in map.values) {
    switch (value) {
      case ViewPB _ when value.layout.isDocumentView:
        final payload = OpenDocumentPayloadPB(documentId: value.id);
        await DocumentEventGetDocumentText(payload).send().fold(
          (pb) {
            metadata.add(
              ChatMessageMetaPB(
                id: value.id,
                name: value.name,
                data: pb.text,
                loaderType: ContextLoaderTypePB.Txt,
                source: appflowySource,
              ),
            );
          },
          (err) => Log.error('Failed to get document text: $err'),
        );
        break;
      case ChatFile(
          filePath: final filePath,
          fileName: final fileName,
          fileType: final fileType,
        ):
        metadata.add(
          ChatMessageMetaPB(
            id: nanoid(8),
            name: fileName,
            data: filePath,
            loaderType: fileType,
            source: filePath,
          ),
        );
        break;
    }
  }

  return metadata;
}

List<ChatFile> chatFilesFromMessageMetadata(
  Map<String, dynamic>? map,
) {
  final List<ChatFile> metadata = [];
  if (map != null) {
    for (final entry in map.entries) {
      if (entry.value is ChatFile) {
        metadata.add(entry.value);
      }
    }
  }

  return metadata;
}
