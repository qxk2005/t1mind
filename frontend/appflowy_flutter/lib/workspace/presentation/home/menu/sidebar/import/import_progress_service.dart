import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:isolate';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/protobuf.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

/// 导入日志条目
class ImportLogEntry {
  ImportLogEntry({
    required this.timestamp,
    required this.level,
    required this.message,
  });

  final DateTime timestamp;
  final String level; // 'info', 'debug', 'warn', 'error'
  final String message;

  factory ImportLogEntry.fromJson(Map<String, dynamic> json) {
    return ImportLogEntry(
      timestamp: DateTime.fromMillisecondsSinceEpoch(
        json['timestamp'] as int? ?? DateTime.now().millisecondsSinceEpoch,
      ),
      level: json['level'] as String? ?? 'info',
      message: json['message'] as String? ?? '',
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'timestamp': timestamp.millisecondsSinceEpoch,
      'level': level,
      'message': message,
    };
  }
}

/// 导入进度数据模型
class ImportProgress {
  ImportProgress({
    required this.importId,
    required this.fileName,
    required this.progress,
    required this.currentStep,
    this.error,
    this.logs = const [],
  });

  final String importId;
  final String fileName;
  final double progress; // 0.0 - 1.0
  final String currentStep;
  final String? error;
  final List<ImportLogEntry> logs; // 日志列表

  factory ImportProgress.fromJsonString(String jsonString) {
    try {
      final json = jsonDecode(jsonString) as Map<String, dynamic>;
      final logsJson = json['logs'] as List<dynamic>? ?? [];
      final logs = logsJson
          .map((e) => ImportLogEntry.fromJson(e as Map<String, dynamic>))
          .toList();
      
      return ImportProgress(
        importId: json['import_id'] as String? ?? '',
        fileName: json['file_name'] as String? ?? '',
        progress: (json['progress'] as num?)?.toDouble() ?? 0.0,
        currentStep: json['current_step'] as String? ?? '',
        error: json['error'] as String?,
        logs: logs,
      );
    } catch (e) {
      Log.error('Failed to parse ImportProgress: $e');
      rethrow;
    }
  }

  Map<String, dynamic> toJson() {
    return {
      'import_id': importId,
      'file_name': fileName,
      'progress': progress,
      'current_step': currentStep,
      if (error != null) 'error': error,
      'logs': logs.map((e) => e.toJson()).toList(),
    };
  }
}

/// 导入进度服务
/// 类似于 FileStorageService，用于监听导入进度更新
class ImportProgressService {
  ImportProgressService() {
    _port.handler = _controller.add;
    _subscription = _controller.stream.listen(
      (event) {
        try {
          final progress = ImportProgress.fromJsonString(event);
          final notifier = _notifierList[progress.importId];
          if (notifier != null) {
            notifier.value = progress;
          }
        } catch (e) {
          Log.error('Failed to handle import progress: $e');
        }
      },
    );

    _registerStream();
  }

  final Map<String, AutoRemoveNotifier<ImportProgress>> _notifierList = {};
  final RawReceivePort _port = RawReceivePort();
  final StreamController<String> _controller = StreamController.broadcast();
  late StreamSubscription<String> _subscription;

  void _registerStream() {
    final payload = RegisterImportProgressStreamPB()
      ..port = Int64(_port.sendPort.nativePort);
    FolderEventRegisterImportProgressStream(payload).send();
    Log.debug('Import progress stream registered with port: ${_port.sendPort.nativePort}');
  }

  /// 获取导入进度的历史数据（包括所有累积的日志）
  Future<ImportProgress?> getImportProgress(String importId) async {
    try {
      final request = GetImportProgressPB()..importId = importId;
      final result = await FolderEventGetImportProgress(request).send();
      return result.fold(
        (pb) {
          final logs = pb.logs.map((log) {
            return ImportLogEntry(
              timestamp: DateTime.fromMillisecondsSinceEpoch(log.timestamp.toInt()),
              level: log.level,
              message: log.message,
            );
          }).toList();
          return ImportProgress(
            importId: pb.importId,
            fileName: pb.fileName,
            progress: pb.progress,
            currentStep: pb.currentStep,
            error: pb.hasError() ? pb.error : null,
            logs: logs,
          );
        },
        (error) {
          Log.error('Failed to get import progress: $error');
          return null;
        },
      );
    } catch (e) {
      Log.error('Failed to get import progress: $e');
      return null;
    }
  }

  /// 监听指定导入任务的进度
  AutoRemoveNotifier<ImportProgress> onImportProgress({
    required String importId,
  }) {
    _notifierList.remove(importId)?.dispose();

    final notifier = AutoRemoveNotifier<ImportProgress>(
      ImportProgress(
        importId: importId,
        fileName: '',
        progress: 0.0,
        currentStep: '准备导入...',
      ),
      notifierList: _notifierList,
      fileId: importId,
    );
    _notifierList[importId] = notifier;

    return notifier;
  }

  Future<void> dispose() async {
    // dispose all notifiers
    for (final notifier in _notifierList.values) {
      notifier.dispose();
    }

    await _controller.close();
    await _subscription.cancel();
    _port.close();
  }
}

/// 自动移除的通知器
/// 当不再有监听者时自动从列表中移除
class AutoRemoveNotifier<T> extends ValueNotifier<T> {
  AutoRemoveNotifier(
    T initialValue, {
    required this.notifierList,
    required this.fileId,
  }) : super(initialValue);

  final Map<String, AutoRemoveNotifier<T>> notifierList;
  final String fileId;

  @override
  void dispose() {
    notifierList.remove(fileId);
    super.dispose();
  }
}

