import 'dart:async';
import 'dart:math';

import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/import_progress_service.dart';
import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/pdf_import_detail_dialog.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;

/// PDF 导入进度对话框
/// 
/// 显示 PDF 导入的进度和当前操作状态
class PdfImportProgressDialog extends StatefulWidget {
  const PdfImportProgressDialog({
    super.key,
    required this.fileName,
    required this.fileSize,
    this.importId,
    this.onCancel,
    this.onDetailDialogOpened,
  });

  final String fileName;
  final int fileSize; // 文件大小（字节）
  final String? importId; // 导入任务 ID（用于监听真实进度）
  final VoidCallback? onCancel;
  final ValueChanged<bool>? onDetailDialogOpened; // 当详细对话框打开/关闭时调用

  @override
  State<PdfImportProgressDialog> createState() => _PdfImportProgressDialogState();
}

class _PdfImportProgressDialogState extends State<PdfImportProgressDialog> {
  String _currentStep = '准备导入...';
  double _progress = 0.0;
  Timer? _progressTimer;
  bool _isCancelled = false;
  DateTime? _startTime;
  ImportProgressService? _progressService;
  AutoRemoveNotifier<ImportProgress>? _progressNotifier;
  String? _importId;

  @override
  void initState() {
    super.initState();
    _startTime = DateTime.now();
    _progressService = ImportProgressService();
    _startListeningProgress();
    _startProgressSimulation(); // 作为后备，如果真实进度没有到来
  }

  @override
  void dispose() {
    _progressTimer?.cancel();
    _progressNotifier?.dispose();
    _progressService?.dispose();
    super.dispose();
  }

  void _startListeningProgress() {
    if (widget.importId != null && widget.importId!.isNotEmpty) {
      _importId = widget.importId;
      _progressNotifier = _progressService?.onImportProgress(importId: _importId!);
      _progressNotifier?.addListener(_onProgressChanged);
      // 立即更新一次当前值
      if (_progressNotifier != null) {
        _updateProgress(_progressNotifier!.value);
      }
    }
  }

  void _onProgressChanged() {
    if (_progressNotifier != null) {
      _updateProgress(_progressNotifier!.value);
    }
  }

  void _updateProgress(ImportProgress progress) {
    if (mounted) {
      setState(() {
        _progress = progress.progress;
        _currentStep = progress.currentStep;
      });
      
      // 如果有错误，停止定时器
      if (progress.error != null) {
        _progressTimer?.cancel();
      }
      
      // 如果完成，停止定时器
      if (progress.progress >= 1.0) {
        _progressTimer?.cancel();
      }
    }
  }

  void _startProgressSimulation() {
    // 根据文件大小估算时间
    // 假设：1MB 约需要 5 秒，大文件可能需要更长时间
    final estimatedSeconds = (widget.fileSize / (1024 * 1024) * 5).clamp(10.0, 300.0);
    final totalSteps = [
      '正在读取文件...',
      '正在提取文本内容...',
      '正在提取图片...',
      '正在解析格式...',
      '正在转换为文档格式...',
      '即将完成...',
    ];
    
    int currentStepIndex = 0;
    double progressPerStep = 1.0 / totalSteps.length;
    
    _progressTimer = Timer.periodic(const Duration(milliseconds: 200), (timer) {
      if (!mounted || _isCancelled) {
        timer.cancel();
        return;
      }

      final elapsed = DateTime.now().difference(_startTime!);
      final elapsedSeconds = elapsed.inSeconds;
      
      // 基于时间的进度估算（渐进式，越到后面越慢）
      // 使用平滑的曲线：progress = 1 - e^(-t/estimatedSeconds * 2)
      final timeProgress = (elapsedSeconds / estimatedSeconds).clamp(0.0, 0.95);
      // 使用平滑的指数曲线（渐进式，开始快，后面慢）
      final smoothProgress = 1.0 - exp(-elapsedSeconds / (estimatedSeconds * 0.8)).clamp(0.0, 1.0);
      
      // 结合步骤进度和时间进度（确保不会超过步骤进度）
      _progress = (timeProgress * 0.6 + smoothProgress * 0.4).clamp(0.0, 0.95);
      
      // 根据进度更新当前步骤
      final stepProgressThreshold = _progress / progressPerStep;
      if (stepProgressThreshold.floor() > currentStepIndex && currentStepIndex < totalSteps.length - 1) {
        currentStepIndex = stepProgressThreshold.floor().clamp(0, totalSteps.length - 1);
      }
      
      if (currentStepIndex < totalSteps.length) {
        _currentStep = totalSteps[currentStepIndex];
      }

      setState(() {});
    });
  }

  void _handleCancel() {
    setState(() {
      _isCancelled = true;
    });
    widget.onCancel?.call();
  }

  String _formatFileSize(int bytes) {
    if (bytes < 1024) {
      return '$bytes B';
    } else if (bytes < 1024 * 1024) {
      return '${(bytes / 1024).toStringAsFixed(1)} KB';
    } else {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
  }

  String _getEstimatedTimeRemaining() {
    if (_progress <= 0) return '计算中...';
    
    final elapsed = DateTime.now().difference(_startTime!);
    if (_progress < 0.05) return '计算中...';
    
    final estimatedTotal = elapsed.inSeconds / _progress;
    final remaining = (estimatedTotal - elapsed.inSeconds).round();
    
    if (remaining <= 0) return '即将完成';
    if (remaining < 60) return '约 $remaining 秒';
    
    final minutes = remaining ~/ 60;
    final seconds = remaining % 60;
    return '约 ${minutes} 分 ${seconds} 秒';
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      backgroundColor: Theme.of(context).colorScheme.surface,
      title: FlowyText.semibold(
        '正在导入 PDF',
        fontSize: 20,
        color: Theme.of(context).colorScheme.tertiary,
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          vertical: 20.0,
          horizontal: 24.0,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 文件名
            Row(
              children: [
                Icon(
                  Icons.picture_as_pdf,
                  size: 20,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const HSpace(8),
                Expanded(
                  child: FlowyText.medium(
                    p.basename(widget.fileName),
                    fontSize: 14,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                FlowyText.regular(
                  _formatFileSize(widget.fileSize),
                  fontSize: 12,
                  color: Theme.of(context).hintColor,
                ),
              ],
            ),
            const VSpace(20),
            
            // 当前步骤
            FlowyText.regular(
              _currentStep,
              fontSize: 13,
              color: Theme.of(context).colorScheme.tertiary,
            ),
            const VSpace(12),
            
            // 进度条
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: LinearProgressIndicator(
                value: _progress,
                backgroundColor: Theme.of(context).colorScheme.surfaceVariant,
                minHeight: 8,
                valueColor: AlwaysStoppedAnimation<Color>(
                  Theme.of(context).colorScheme.primary,
                ),
              ),
            ),
            const VSpace(12),
            
            // 进度信息和剩余时间
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                FlowyText.regular(
                  '${(_progress * 100).toStringAsFixed(1)}%',
                  fontSize: 12,
                  color: Theme.of(context).hintColor,
                ),
                FlowyText.regular(
                  _getEstimatedTimeRemaining(),
                  fontSize: 12,
                  color: Theme.of(context).hintColor,
                ),
              ],
            ),
            
            const VSpace(24),
            
            // 按钮行
            Row(
              children: [
                // 查看具体进度按钮
                if (_importId != null && _importId!.isNotEmpty)
                  Expanded(
                    child: OutlinedRoundedButton(
                      text: '查看具体进度',
                      onTap: () {
                        // 通知外部详细对话框已打开
                        widget.onDetailDialogOpened?.call(true);
                        
                        showPdfImportDetailDialog(
                          context,
                          importId: _importId!,
                          fileName: widget.fileName,
                        ).then((_) {
                          // 详细对话框关闭后，通知外部
                          widget.onDetailDialogOpened?.call(false);
                        });
                      },
                    ),
                  ),
                if (_importId != null && _importId!.isNotEmpty) const HSpace(12),
                // 取消按钮
                Expanded(
                  child: !_isCancelled
                      ? OutlinedRoundedButton(
                          text: '取消',
                          onTap: _handleCancel,
                        )
                      : OutlinedRoundedButton(
                          text: '正在取消...',
                          onTap: null,
                        ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

/// 显示 PDF 导入进度对话框
Future<void> showPdfImportProgressDialog(
  BuildContext context, {
  required String fileName,
  required int fileSize,
  String? importId,
  VoidCallback? onCancel,
  ValueChanged<bool>? onDetailDialogOpened,
}) async {
  await FlowyOverlay.show(
    context: context,
    builder: (context) => PdfImportProgressDialog(
      fileName: fileName,
      fileSize: fileSize,
      importId: importId,
      onCancel: onCancel,
      onDetailDialogOpened: onDetailDialogOpened,
    ),
  );
}

