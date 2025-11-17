import 'dart:async';

import 'package:appflowy_backend/protobuf/flowy-user/import_settings.pb.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../import/import_settings_bloc.dart';

/// 工具安装进度对话框
/// 
/// 显示工具安装的实时进度，包括：
/// - 当前安装步骤说明
/// - 进度条显示
/// - 实时执行日志
/// - 状态消息
class InstallToolsProgressDialog extends StatefulWidget {
  const InstallToolsProgressDialog({
    super.key,
    required this.toolNames,
    this.autoStartInstall = true,
  });

  final List<String> toolNames;
  final bool autoStartInstall;

  @override
  State<InstallToolsProgressDialog> createState() => _InstallToolsProgressDialogState();
}

class _InstallToolsProgressDialogState extends State<InstallToolsProgressDialog> {
  final ScrollController _scrollController = ScrollController();
  Timer? _autoScrollTimer;

  @override
  void initState() {
    super.initState();
    // 自动滚动到底部，显示最新日志
    _autoScrollTimer = Timer.periodic(const Duration(milliseconds: 500), (timer) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
    
    // 如果自动开始安装，在对话框显示后立即触发安装事件
    // 使用 Future.microtask 确保在 build 完成后才触发
    if (widget.autoStartInstall) {
      Future.microtask(() {
        if (mounted) {
          context.read<ImportSettingsBloc>().add(
            ImportSettingsEvent.installMissingTools(widget.toolNames),
          );
        }
      });
    }
  }

  @override
  void dispose() {
    _autoScrollTimer?.cancel();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<ImportSettingsBloc, ImportSettingsState>(
      listener: (context, state) {
        // 监听状态变化，添加详细的调试日志
        final progress = state.installProgress;
        final isInstalling = state.isInstallingTools;
        
        debugPrint('[InstallDialog] Listener 触发: progress=${progress != null}, isInstalling=$isInstalling');
        
        if (progress != null) {
          debugPrint('[InstallDialog] Listener - 状态: ${progress.status}, 消息: ${progress.message}, 日志数量: ${progress.logs.length}');
          debugPrint('[InstallDialog] Listener - 工具名: ${progress.toolName}, 进度: ${progress.progress}');
          
          // 强制重建 UI
          if (mounted) {
            setState(() {});
          }
        } else {
          debugPrint('[InstallDialog] Listener - progress 为 null');
        }
      },
      // 移除 buildWhen 条件，让所有状态变化都触发重建，确保不会遗漏任何更新
      // buildWhen: null, // 不设置 buildWhen，让所有状态变化都触发重建
      builder: (context, state) {
        final progress = state.installProgress;
        final isInstalling = state.isInstallingTools;
        final isCheckingTools = state.isCheckingTools;
        
        // 调试日志
        debugPrint('[InstallDialog] Builder 被调用: progress=${progress != null}, isInstalling=$isInstalling, isCheckingTools=$isCheckingTools');
        if (progress != null) {
          debugPrint('[InstallDialog] Builder - 状态更新: status=${progress.status}, message=${progress.message}, logs=${progress.logs.length}');
        }
        
        // 如果安装完成或失败，显示相应的状态
        final isCompleted = progress != null && 
            progress.status == InstallToolStatusPB.InstallToolCompleted;
        // 检查失败状态：既检查状态，也检查消息中是否包含错误关键词
        final isFailed = progress != null && (
            progress.status == InstallToolStatusPB.InstallToolFailed ||
            progress.message.toLowerCase().contains('失败') ||
            progress.message.toLowerCase().contains('错误') ||
            progress.message.toLowerCase().contains('权限') ||
            progress.message.toLowerCase().contains('administrator')
        );

        return FlowyDialog(
          backgroundColor: Theme.of(context).colorScheme.surface,
          expandHeight: false,
          width: 600,
          constraints: const BoxConstraints(
            maxWidth: 600,
            maxHeight: 700,
          ),
          title: FlowyText.semibold(
            "安装缺失组件",
            fontSize: 20,
            color: Theme.of(context).colorScheme.tertiary,
          ),
          child: SingleChildScrollView(
            child: Padding(
              padding: const EdgeInsets.symmetric(
                vertical: 10.0,
                horizontal: 20.0,
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                // 安装步骤说明
                _buildStepsOverview(widget.toolNames),
                const SizedBox(height: 16),
                
                // 当前状态
                if (progress != null) ...[
                  _buildCurrentStatus(progress, isCompleted, isFailed),
                  const SizedBox(height: 16),
                  // 如果失败，显示详细的错误信息
                  if (isFailed) ...[
                    _buildErrorMessage(progress.message, progress.logs),
                    const SizedBox(height: 16),
                  ],
                ] else if (isInstalling) ...[
                  _buildInitialStatus(),
                  const SizedBox(height: 16),
                ],
                
                // 进度条
                if (progress != null) ...[
                  _buildProgressBar(progress.progress, isCompleted, isFailed),
                  const SizedBox(height: 16),
                ],
                
                // 执行日志
                if (progress != null && progress.logs.isNotEmpty) ...[
                  _buildLogsSection(progress.logs),
                  const SizedBox(height: 16),
                ] else if (isInstalling) ...[
                  _buildWaitingLogs(),
                  const SizedBox(height: 16),
                ],
                
                // 操作按钮
                _buildActionButtons(isCompleted, isFailed, isInstalling, isCheckingTools),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  /// 构建步骤概览
  Widget _buildStepsOverview(List<String> toolNames) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.blue.shade50,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.blue.shade200),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.info_outline, size: 18, color: Colors.blue.shade700),
              const SizedBox(width: 8),
              FlowyText.semibold(
                "安装步骤",
                fontSize: 14,
                color: Colors.blue.shade900,
              ),
            ],
          ),
          const SizedBox(height: 8),
          ...toolNames.map((toolName) {
            String stepDescription;
            if (toolName.contains('marker')) {
              stepDescription = "1. 检查并安装 Homebrew（如果未安装）\n2. 安装 pipx（Python 包管理工具）\n3. 安装 marker-pdf 依赖库\n4. 安装 marker-pdf（PDF 转换工具）";
            } else if (toolName == 'pipx') {
              stepDescription = "1. 检查并安装 Homebrew（如果未安装）\n2. 使用 Homebrew 安装 pipx";
            } else if (toolName == 'brew') {
              stepDescription = "安装 Homebrew（需要用户交互，可能无法自动完成）";
            } else {
              stepDescription = "安装 $toolName";
            }
            
            return Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Container(
                    width: 20,
                    height: 20,
                    decoration: BoxDecoration(
                      color: Colors.blue.shade200,
                      shape: BoxShape.circle,
                    ),
                    child: Center(
                      child: FlowyText.semibold(
                        toolName.substring(0, 1).toUpperCase(),
                        fontSize: 10,
                        color: Colors.blue.shade900,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        FlowyText.semibold(
                          toolName,
                          fontSize: 13,
                          color: Colors.blue.shade900,
                        ),
                        const SizedBox(height: 4),
                        FlowyText.regular(
                          stepDescription,
                          fontSize: 11,
                          color: Colors.blue.shade700,
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            );
          }),
        ],
      ),
    );
  }

  /// 构建当前状态
  Widget _buildCurrentStatus(
    InstallToolProgressPB progress,
    bool isCompleted,
    bool isFailed,
  ) {
    Color statusColor;
    IconData statusIcon;
    String statusText;

    if (isCompleted) {
      statusColor = Colors.green;
      statusIcon = Icons.check_circle;
      statusText = "安装完成";
    } else if (isFailed) {
      statusColor = Colors.red;
      statusIcon = Icons.error;
      statusText = "安装失败";
    } else {
      statusColor = Colors.blue;
      statusIcon = Icons.download;
      statusText = "正在安装: ${progress.toolName}";
    }

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: statusColor.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: statusColor.withValues(alpha: 0.3)),
      ),
      child: Row(
        children: [
          Icon(statusIcon, size: 20, color: statusColor),
          const SizedBox(width: 8),
          Expanded(
            child: FlowyText.semibold(
              statusText,
              fontSize: 14,
              color: statusColor,
            ),
          ),
        ],
      ),
    );
  }

  /// 构建错误信息显示区域
  Widget _buildErrorMessage(String message, List<String> logs) {
    // 提取错误相关的日志（包含 ✗、失败、出错、解决方案等关键词）
    // 如果日志为空或过滤后为空，则显示所有日志（可能都是错误相关的）
    final errorLogs = logs.isEmpty ? [] : (logs.where((log) {
      // 跳过空行，但保留其他所有行（因为错误信息可能很重要）
      if (log.trim().isEmpty) return false;
      final lowerLog = log.toLowerCase();
      // 检查是否包含数字编号的步骤（如 "1."、"2."、"3."），这些应该被保留
      final hasStepNumber = RegExp(r'\s+[123]\s*\.\s+').hasMatch(log);
      return hasStepNumber ||
             lowerLog.contains('✗') || 
             lowerLog.contains('失败') || 
             lowerLog.contains('出错') ||
             lowerLog.contains('原因') ||
             lowerLog.contains('解决方案') ||
             lowerLog.contains('注意') ||
             lowerLog.contains('权限') ||
             lowerLog.contains('administrator') ||
             lowerLog.contains('sudo') ||
             lowerLog.contains('homebrew') ||
             lowerLog.contains('安装') ||
             lowerLog.contains('命令') ||
             lowerLog.contains('步骤') ||
             lowerLog.contains('echo') ||
             lowerLog.contains('eval') ||
             lowerLog.contains('shellenv') ||
             lowerLog.contains('zprofile') ||
             lowerLog.contains('bash_profile') ||
             lowerLog.contains('.profile');
    }).toList());
    
    // 如果过滤后没有日志，但原始日志不为空，显示所有日志（可能是重要的错误信息）
    final logsToShow = errorLogs.isEmpty && logs.isNotEmpty ? logs : errorLogs;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.red.shade50,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.red.shade300, width: 1.5),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.error_outline, size: 20, color: Colors.red.shade700),
              const SizedBox(width: 8),
              FlowyText.semibold(
                "错误详情",
                fontSize: 15,
                color: Colors.red.shade900,
              ),
            ],
          ),
          const SizedBox(height: 12),
          // 显示主要错误消息（完整显示，不截断）
          if (message.isNotEmpty) ...[
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: Colors.red.shade200),
              ),
              child: SelectableText(
                message,
                style: TextStyle(
                  fontSize: 13,
                  color: Colors.red.shade900,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
            const SizedBox(height: 12),
          ],
          // 显示错误相关的日志
          if (logsToShow.isNotEmpty) ...[
            FlowyText.semibold(
              "详细说明：",
              fontSize: 13,
              color: Colors.red.shade800,
            ),
            const SizedBox(height: 8),
            ...logsToShow.map((log) {
              // 判断日志类型
              final isSolution = log.contains('解决方案') || 
                                log.contains('1.') || 
                                log.contains('2.') || 
                                log.contains('3.');
              final isReason = log.contains('原因');
              final isNote = log.contains('注意');
              
              Color textColor = Colors.red.shade800;
              FontWeight fontWeight = FontWeight.normal;
              
              if (isSolution) {
                textColor = Colors.blue.shade800;
                fontWeight = FontWeight.w600;
              } else if (isReason) {
                textColor = Colors.orange.shade800;
                fontWeight = FontWeight.w500;
              } else if (isNote) {
                textColor = Colors.grey.shade700;
              }
              
              return Padding(
                padding: const EdgeInsets.only(bottom: 6),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (isSolution || isReason || isNote)
                      Icon(
                        isSolution ? Icons.lightbulb_outline : 
                        isReason ? Icons.info_outline : 
                        Icons.note_outlined,
                        size: 14,
                        color: textColor,
                      )
                    else
                      const SizedBox(width: 14),
                    if (isSolution || isReason || isNote)
                      const SizedBox(width: 6)
                    else
                      const SizedBox(width: 0),
                    Expanded(
                      child: SelectableText(
                        log,
                        style: TextStyle(
                          fontSize: 12,
                          color: textColor,
                          fontWeight: fontWeight,
                          fontFamily: 'monospace',
                        ),
                      ),
                    ),
                  ],
                ),
              );
            }),
          ],
        ],
      ),
    );
  }

  /// 构建初始状态
  Widget _buildInitialStatus() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.blue.shade50,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.blue.shade200),
      ),
      child: Row(
        children: [
          const SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: FlowyText.semibold(
              "正在准备安装...",
              fontSize: 14,
              color: Colors.blue.shade900,
            ),
          ),
        ],
      ),
    );
  }

  /// 构建进度条
  Widget _buildProgressBar(double progress, bool isCompleted, bool isFailed) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            FlowyText.regular(
              "安装进度",
              fontSize: 12,
              color: Colors.grey.shade700,
            ),
            FlowyText.regular(
              "${(progress * 100).toStringAsFixed(0)}%",
              fontSize: 12,
              color: Colors.grey.shade700,
            ),
          ],
        ),
        const SizedBox(height: 8),
        LinearProgressIndicator(
          value: progress,
          backgroundColor: Colors.grey.shade200,
          minHeight: 8,
          valueColor: AlwaysStoppedAnimation<Color>(
            isFailed ? Colors.red : (isCompleted ? Colors.green : Colors.blue),
          ),
        ),
      ],
    );
  }

  /// 构建日志部分
  Widget _buildLogsSection(List<String> logs) {
    return Container(
      constraints: const BoxConstraints(maxHeight: 300),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.grey.shade50,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey.shade300),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              Icon(Icons.description_outlined, size: 16, color: Colors.grey.shade700),
              const SizedBox(width: 8),
              FlowyText.semibold(
                "执行日志",
                fontSize: 13,
                color: Colors.grey.shade800,
              ),
            ],
          ),
          const SizedBox(height: 8),
          SizedBox(
            height: 200,
            child: SingleChildScrollView(
              controller: _scrollController,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: logs.asMap().entries.map((entry) {
                  final index = entry.key;
                  final log = entry.value;
                  
                  // 判断日志类型
                  final isSuccess = log.contains('✓') || log.contains('成功');
                  final isError = log.contains('✗') || log.contains('失败') || log.contains('出错');
                  final isWarning = log.contains('⚠') || log.contains('警告');
                  final isStep = log.contains('步骤') || log.contains('开始') || log.contains('检查') || log.contains('安装');
                  
                  Color textColor = Colors.grey.shade700;
                  if (isSuccess) {
                    textColor = Colors.green.shade700;
                  } else if (isError) {
                    textColor = Colors.red.shade700;
                  } else if (isWarning) {
                    textColor = Colors.orange.shade700;
                  } else if (isStep) {
                    textColor = Colors.blue.shade700;
                  }
                  
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        SizedBox(
                          width: 40,
                          child: FlowyText.regular(
                            '${index + 1}.',
                            fontSize: 10,
                            color: Colors.grey.shade500,
                            fontFamily: 'monospace',
                          ),
                        ),
                        Expanded(
                          child: FlowyText.regular(
                            log,
                            fontSize: 11,
                            color: textColor,
                            fontFamily: 'monospace',
                          ),
                        ),
                      ],
                    ),
                  );
                }).toList(),
              ),
            ),
          ),
        ],
      ),
    );
  }

  /// 构建等待日志
  Widget _buildWaitingLogs() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.grey.shade50,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey.shade300),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.description_outlined, size: 16, color: Colors.grey.shade700),
              const SizedBox(width: 8),
              FlowyText.semibold(
                "执行日志",
                fontSize: 13,
                color: Colors.grey.shade800,
              ),
            ],
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              const SizedBox(
                width: 12,
                height: 12,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
              const SizedBox(width: 8),
              FlowyText.regular(
                "正在连接后端服务，准备开始安装...",
                fontSize: 11,
                color: Colors.grey.shade600,
              ),
            ],
          ),
        ],
      ),
    );
  }

  /// 构建操作按钮
  Widget _buildActionButtons(bool isCompleted, bool isFailed, bool isInstalling, bool isCheckingTools) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        // 左侧：刷新按钮（始终显示）
        ElevatedButton.icon(
          onPressed: isCheckingTools ? null : () {
            // 重新检查工具状态，获取最新日志
            debugPrint('[InstallDialog] 用户点击刷新按钮');
            context.read<ImportSettingsBloc>().add(
              const ImportSettingsEvent.checkImportTools(),
            );
          },
          icon: isCheckingTools
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.refresh, size: 16),
          label: Text(isCheckingTools ? "刷新中..." : "刷新状态"),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.blue.shade100,
            foregroundColor: Colors.blue.shade900,
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          ),
        ),
        // 右侧：关闭/完成按钮
        if (isInstalling) ...[
          ElevatedButton.icon(
            onPressed: () {
              // 取消安装（如果需要的话）
              Navigator.of(context).pop();
            },
            icon: const Icon(Icons.close, size: 16),
            label: const Text("关闭（后台继续）"),
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.grey.shade300,
              foregroundColor: Colors.grey.shade800,
            ),
          ),
        ] else ...[
          ElevatedButton(
            onPressed: () {
              Navigator.of(context).pop();
              // 如果安装完成，刷新工具状态
              if (isCompleted) {
                context.read<ImportSettingsBloc>().add(
                  const ImportSettingsEvent.checkImportTools(),
                );
              }
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: isCompleted ? Colors.green : Colors.blue,
              foregroundColor: Colors.white,
            ),
            child: Text(isCompleted ? "完成" : (isFailed ? "关闭" : "确定")),
          ),
        ],
      ],
    );
  }
}

/// 显示工具安装进度对话框
Future<void> showInstallToolsProgressDialog(
  BuildContext context, {
  required List<String> toolNames,
  bool autoStartInstall = true,
  ImportSettingsBloc? bloc,
}) async {
  // 如果提供了 bloc，直接使用；否则从 context 中读取
  final importSettingsBloc = bloc ?? context.read<ImportSettingsBloc>();
  
  // 先显示对话框，确保 BlocProvider 可用
  // 使用 Future.microtask 确保对话框先显示，然后再触发安装事件
  await FlowyOverlay.show(
    context: context,
    builder: (dialogContext) => BlocProvider<ImportSettingsBloc>.value(
      value: importSettingsBloc,
      child: InstallToolsProgressDialog(
        toolNames: toolNames,
        autoStartInstall: autoStartInstall,
      ),
    ),
  );
}

