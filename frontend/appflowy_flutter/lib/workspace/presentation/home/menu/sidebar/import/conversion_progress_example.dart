import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/conversion_progress_dialog.dart';
import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/import_type.dart';
import 'package:flutter/material.dart';

/// 转换进度对话框使用示例
/// 
/// 这个示例展示了如何使用 ConversionProgressDialog 组件
/// 来显示文档转换的进度
class ConversionProgressExample {
  /// 显示转换进度对话框的示例
  static Future<void> showExample(BuildContext context) async {
    // 创建示例转换任务
    final tasks = [
      ConversionTask(
        fileName: 'document1.docx',
        filePath: '/path/to/document1.docx',
        importType: ImportType.word,
        status: ConversionTaskStatus.completed,
        progress: 1.0,
        startTime: DateTime.now().subtract(const Duration(minutes: 2)),
        endTime: DateTime.now().subtract(const Duration(minutes: 1)),
      ),
      ConversionTask(
        fileName: 'document2.pdf',
        filePath: '/path/to/document2.pdf',
        importType: ImportType.pdf,
        status: ConversionTaskStatus.processing,
        progress: 0.6,
        startTime: DateTime.now().subtract(const Duration(minutes: 1)),
      ),
      ConversionTask(
        fileName: 'document3.docx',
        filePath: '/path/to/document3.docx',
        importType: ImportType.word,
        status: ConversionTaskStatus.error,
        error: '文件格式不支持',
        startTime: DateTime.now().subtract(const Duration(minutes: 3)),
        endTime: DateTime.now().subtract(const Duration(minutes: 2)),
      ),
    ];

    // 显示进度对话框
    await showConversionProgressDialog(
      context,
      tasks: tasks,
      onCancel: () {
        // 处理取消操作
        print('用户取消了转换');
      },
      onRetry: () {
        // 处理重试操作
        print('用户重试转换');
      },
      onClose: () {
        // 处理关闭操作
        print('用户关闭了对话框');
      },
    );
  }

  /// 创建单个转换任务的示例
  static ConversionTask createTask({
    required String fileName,
    required String filePath,
    required ImportType importType,
    ConversionTaskStatus status = ConversionTaskStatus.pending,
    double progress = 0.0,
    String? error,
  }) {
    return ConversionTask(
      fileName: fileName,
      filePath: filePath,
      importType: importType,
      status: status,
      progress: progress,
      error: error,
    );
  }

  /// 更新任务状态的示例
  static ConversionTask updateTaskStatus(
    ConversionTask task,
    ConversionTaskStatus newStatus, {
    double? progress,
    String? error,
  }) {
    return task.copyWith(
      status: newStatus,
      progress: progress,
      error: error,
      startTime: newStatus == ConversionTaskStatus.processing && task.startTime == null
          ? DateTime.now()
          : task.startTime,
      endTime: newStatus == ConversionTaskStatus.completed || newStatus == ConversionTaskStatus.error
          ? DateTime.now()
          : task.endTime,
    );
  }
}
