import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/conversion_logs_dialog.dart';
import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/conversion_progress_dialog.dart';
import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/import_type.dart';
import 'package:flutter/material.dart';

/// 转换日志功能使用示例
/// 
/// 这个文件展示了如何使用转换日志查看功能
class ConversionLogsExample extends StatefulWidget {
  const ConversionLogsExample({super.key});

  @override
  State<ConversionLogsExample> createState() => _ConversionLogsExampleState();
}

class _ConversionLogsExampleState extends State<ConversionLogsExample> {
  List<ConversionTask> _tasks = [];

  @override
  void initState() {
    super.initState();
    _generateSampleTasks();
  }

  void _generateSampleTasks() {
    _tasks = [
      ConversionTask(
        fileName: 'document1.docx',
        filePath: '/path/to/document1.docx',
        importType: ImportType.word,
        status: ConversionTaskStatus.completed,
        progress: 1.0,
        startTime: DateTime.now().subtract(const Duration(minutes: 5)),
        endTime: DateTime.now().subtract(const Duration(minutes: 2)),
      ),
      ConversionTask(
        fileName: 'document2.pdf',
        filePath: '/path/to/document2.pdf',
        importType: ImportType.pdf,
        status: ConversionTaskStatus.error,
        progress: 0.3,
        error: 'PDF文件损坏，无法解析',
        startTime: DateTime.now().subtract(const Duration(minutes: 3)),
        endTime: DateTime.now().subtract(const Duration(minutes: 1)),
      ),
      ConversionTask(
        fileName: 'document3.docx',
        filePath: '/path/to/document3.docx',
        importType: ImportType.word,
        status: ConversionTaskStatus.processing,
        progress: 0.7,
        startTime: DateTime.now().subtract(const Duration(minutes: 2)),
      ),
      ConversionTask(
        fileName: 'document4.pdf',
        filePath: '/path/to/document4.pdf',
        importType: ImportType.pdf,
        status: ConversionTaskStatus.pending,
        progress: 0.0,
      ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('转换日志功能示例'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              '转换日志功能演示',
              style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            const Text(
              '这个示例展示了转换日志查看功能的各种特性：',
              style: TextStyle(fontSize: 16),
            ),
            const SizedBox(height: 8),
            const Text('• 实时日志显示'),
            const Text('• 按级别和任务过滤'),
            const Text('• 关键词搜索'),
            const Text('• 日志导出功能'),
            const Text('• 大日志量性能优化'),
            const SizedBox(height: 24),
            ElevatedButton(
              onPressed: _showProgressDialog,
              child: const Text('显示转换进度对话框'),
            ),
            const SizedBox(height: 12),
            ElevatedButton(
              onPressed: _showLogsDialog,
              child: const Text('直接显示日志对话框'),
            ),
            const SizedBox(height: 12),
            ElevatedButton(
              onPressed: _generateMoreTasks,
              child: const Text('生成更多任务（测试性能）'),
            ),
            const SizedBox(height: 24),
            Expanded(
              child: ListView.builder(
                itemCount: _tasks.length,
                itemBuilder: (context, index) {
                  final task = _tasks[index];
                  return Card(
                    child: ListTile(
                      leading: Icon(_getStatusIcon(task.status)),
                      title: Text(task.fileName),
                      subtitle: Text('状态: ${_getStatusText(task.status)}'),
                      trailing: Text('${(task.progress * 100).toStringAsFixed(1)}%'),
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _showProgressDialog() {
    showConversionProgressDialog(
      context,
      tasks: _tasks,
      onCancel: () {
        print('用户取消了转换');
      },
      onRetry: () {
        print('用户重试转换');
        _generateSampleTasks();
      },
      onClose: () {
        print('用户关闭了进度对话框');
      },
    );
  }

  void _showLogsDialog() {
    showConversionLogsDialog(
      context,
      tasks: _tasks,
      onClose: () {
        print('用户关闭了日志对话框');
      },
    );
  }

  void _generateMoreTasks() {
    final moreTasks = <ConversionTask>[];
    for (int i = 0; i < 100; i++) {
      moreTasks.add(ConversionTask(
        fileName: 'document_$i.docx',
        filePath: '/path/to/document_$i.docx',
        importType: ImportType.word,
        status: ConversionTaskStatus.values[i % ConversionTaskStatus.values.length],
        progress: (i % 10) / 10.0,
        error: i % 5 == 0 ? '模拟错误 $i' : null,
        startTime: DateTime.now().subtract(Duration(minutes: i)),
        endTime: i % 3 == 0 ? DateTime.now().subtract(Duration(minutes: i - 1)) : null,
      ));
    }
    setState(() {
      _tasks.addAll(moreTasks);
    });
  }

  IconData _getStatusIcon(ConversionTaskStatus status) {
    switch (status) {
      case ConversionTaskStatus.pending:
        return Icons.schedule;
      case ConversionTaskStatus.processing:
        return Icons.autorenew;
      case ConversionTaskStatus.completed:
        return Icons.check_circle;
      case ConversionTaskStatus.error:
        return Icons.error;
      case ConversionTaskStatus.cancelled:
        return Icons.cancel;
    }
  }

  String _getStatusText(ConversionTaskStatus status) {
    switch (status) {
      case ConversionTaskStatus.pending:
        return '等待转换';
      case ConversionTaskStatus.processing:
        return '正在转换';
      case ConversionTaskStatus.completed:
        return '转换完成';
      case ConversionTaskStatus.error:
        return '转换失败';
      case ConversionTaskStatus.cancelled:
        return '已取消';
    }
  }
}
