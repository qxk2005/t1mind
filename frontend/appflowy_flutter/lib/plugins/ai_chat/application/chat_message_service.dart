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
          Log.debug("📝 [REALTIME] Received reasoning delta: '$delta'");
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
