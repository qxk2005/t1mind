import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/import_progress_service.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

/// PDF 导入详细进度对话框
/// 
/// 显示导入过程中的详细日志信息
class PdfImportDetailDialog extends StatefulWidget {
  const PdfImportDetailDialog({
    super.key,
    required this.importId,
    required this.fileName,
  });

  final String importId;
  final String fileName;

  @override
  State<PdfImportDetailDialog> createState() => _PdfImportDetailDialogState();
}

class _PdfImportDetailDialogState extends State<PdfImportDetailDialog> {
  ImportProgressService? _progressService;
  AutoRemoveNotifier<ImportProgress>? _progressNotifier;
  final ScrollController _scrollController = ScrollController();
  bool _autoScroll = true;

  @override
  void initState() {
    super.initState();
    _progressService = ImportProgressService();
    _startListeningProgress();
  }

  @override
  void dispose() {
    _progressNotifier?.dispose();
    _progressService?.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _startListeningProgress() async {
    // 先获取历史进度（包括所有累积的日志）
    final historyProgress = await _progressService?.getImportProgress(widget.importId);
    if (historyProgress != null && mounted) {
      // 更新 notifier 的初始值
      _progressNotifier = _progressService?.onImportProgress(importId: widget.importId);
      if (_progressNotifier != null) {
        _progressNotifier!.value = historyProgress;
        _onProgressChanged();
      }
    } else {
      // 如果没有历史进度，使用默认值
      _progressNotifier = _progressService?.onImportProgress(importId: widget.importId);
    }
    
    _progressNotifier?.addListener(_onProgressChanged);
    // 立即更新一次当前值
    if (_progressNotifier != null) {
      _onProgressChanged();
    }
  }

  void _onProgressChanged() {
    if (mounted && _progressNotifier != null) {
      setState(() {});
      // 自动滚动到底部
      if (_autoScroll && _scrollController.hasClients) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (_scrollController.hasClients) {
            _scrollController.animateTo(
              _scrollController.position.maxScrollExtent,
              duration: const Duration(milliseconds: 200),
              curve: Curves.easeOut,
            );
          }
        });
      }
    }
  }

  Color _getLogLevelColor(String level) {
    switch (level.toLowerCase()) {
      case 'error':
        return Colors.red;
      case 'warn':
        return Colors.orange;
      case 'info':
        return Colors.blue;
      case 'debug':
        return Colors.grey;
      default:
        return Colors.grey;
    }
  }

  IconData _getLogLevelIcon(String level) {
    switch (level.toLowerCase()) {
      case 'error':
        return Icons.error;
      case 'warn':
        return Icons.warning;
      case 'info':
        return Icons.info;
      case 'debug':
        return Icons.bug_report;
      default:
        return Icons.circle;
    }
  }

  @override
  Widget build(BuildContext context) {
    final progress = _progressNotifier?.value;
    final logs = progress?.logs ?? [];

    return FlowyDialog(
      backgroundColor: Theme.of(context).colorScheme.surface,
      title: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Expanded(
            child: FlowyText.semibold(
              '导入详细进度',
              fontSize: 18,
              color: Theme.of(context).colorScheme.tertiary,
            ),
          ),
          Row(
            children: [
              FlowyText.regular(
                '自动滚动',
                fontSize: 12,
                color: Theme.of(context).hintColor,
              ),
              const HSpace(4),
              Switch(
                value: _autoScroll,
                onChanged: (value) {
                  setState(() {
                    _autoScroll = value;
                  });
                },
                materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
            ],
          ),
        ],
      ),
      child: Container(
        width: 600,
        height: 500,
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 文件信息
            Row(
              children: [
                Icon(
                  Icons.picture_as_pdf,
                  size: 16,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const HSpace(8),
                Expanded(
                  child: FlowyText.medium(
                    widget.fileName,
                    fontSize: 13,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            const VSpace(12),
            
            // 当前状态
            if (progress != null) ...[
              Row(
                children: [
                  FlowyText.regular(
                    '当前状态: ',
                    fontSize: 12,
                    color: Theme.of(context).hintColor,
                  ),
                  FlowyText.medium(
                    progress.currentStep,
                    fontSize: 12,
                    color: Theme.of(context).colorScheme.tertiary,
                  ),
                  const HSpace(16),
                  FlowyText.regular(
                    '进度: ${(progress.progress * 100).toStringAsFixed(1)}%',
                    fontSize: 12,
                    color: Theme.of(context).hintColor,
                  ),
                ],
              ),
              const VSpace(12),
            ],
            
            // 日志列表
            Expanded(
              child: Container(
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
                  ),
                ),
                child: logs.isEmpty
                    ? Center(
                        child: FlowyText.regular(
                          '暂无日志',
                          fontSize: 13,
                          color: Theme.of(context).hintColor,
                        ),
                      )
                    : ListView.builder(
                        controller: _scrollController,
                        padding: const EdgeInsets.all(8),
                        itemCount: logs.length,
                        itemBuilder: (context, index) {
                          final log = logs[index];
                          final timeFormat = DateFormat('HH:mm:ss.SSS');
                          final timeStr = timeFormat.format(log.timestamp);
                          
                          return Padding(
                            padding: const EdgeInsets.only(bottom: 8),
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                // 时间戳
                                SizedBox(
                                  width: 90,
                                  child: FlowyText.regular(
                                    timeStr,
                                    fontSize: 11,
                                    color: Theme.of(context).hintColor,
                                  ),
                                ),
                                const HSpace(8),
                                // 日志级别图标
                                Icon(
                                  _getLogLevelIcon(log.level),
                                  size: 14,
                                  color: _getLogLevelColor(log.level),
                                ),
                                const HSpace(8),
                                // 日志消息
                                Expanded(
                                  child: SelectableText(
                                    log.message,
                                    style: TextStyle(
                                      fontSize: 12,
                                      fontFamily: 'monospace',
                                      color: Theme.of(context).colorScheme.onSurface,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          );
                        },
                      ),
              ),
            ),
            const VSpace(12),
            
            // 底部按钮
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                  },
                  child: const Text('关闭'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

/// 显示 PDF 导入详细进度对话框
Future<void> showPdfImportDetailDialog(
  BuildContext context, {
  required String importId,
  required String fileName,
}) async {
  await FlowyOverlay.show(
    context: context,
    builder: (context) => PdfImportDetailDialog(
      importId: importId,
      fileName: fileName,
    ),
  );
}

