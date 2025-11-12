import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_dropdown.dart';
import 'package:appflowy/workspace/presentation/settings/widgets/settings_switch.dart';
import 'package:appflowy/workspace/presentation/settings/widgets/settings_text_field.dart';
import 'package:appflowy_backend/protobuf/flowy-user/import_settings.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_profile.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../import/import_settings_bloc.dart';

class SettingsImportView extends StatefulWidget {
  const SettingsImportView({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  State<SettingsImportView> createState() => _SettingsImportViewState();
}

class _SettingsImportViewState extends State<SettingsImportView> {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<ImportSettingsBloc>(
      create: (context) => ImportSettingsBloc(
        userProfile: widget.userProfile,
        workspaceId: widget.workspaceId,
      )..add(const ImportSettingsEvent.initial()),
      child: BlocBuilder<ImportSettingsBloc, ImportSettingsState>(
        builder: (context, state) {
          return SettingsBody(
            title: LocaleKeys.importSettings_title.tr(),
            children: [
              // 转换设置
              SettingsCategory(
                title: LocaleKeys.importSettings_conversionSettings.tr(),
                children: [
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_preserveFormatting.tr(),
                    value: state.settings.preserveFormatting,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updatePreserveFormatting(value),
                      );
                    },
                  ),
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_extractImages.tr(),
                    value: state.settings.extractImages,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateExtractImages(value),
                      );
                    },
                  ),
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_extractTables.tr(),
                    value: state.settings.extractTables,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateExtractTables(value),
                      );
                    },
                  ),
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_autoCreateFolder.tr(),
                    value: state.settings.autoCreateFolder,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateAutoCreateFolder(value),
                      );
                    },
                  ),
                ],
              ),
              
              // 性能设置
              SettingsCategory(
                title: LocaleKeys.importSettings_conversionSettings.tr(),
                children: [
                  SettingsTextField(
                    label: LocaleKeys.importSettings_maxConcurrentConversions.tr(),
                    value: state.settings.maxConcurrentConversions.toString(),
                    onChanged: (value) {
                      final intValue = int.tryParse(value) ?? 3;
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateMaxConcurrentConversions(intValue),
                      );
                    },
                    keyboardType: TextInputType.number,
                  ),
                  SettingsTextField(
                    label: LocaleKeys.importSettings_conversionTimeoutSeconds.tr(),
                    value: state.settings.conversionTimeoutSeconds.toString(),
                    onChanged: (value) {
                      final intValue = int.tryParse(value) ?? 300;
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateConversionTimeoutSeconds(intValue),
                      );
                    },
                    keyboardType: TextInputType.number,
                  ),
                ],
              ),
              
              // 重试设置
              SettingsCategory(
                title: LocaleKeys.importSettings_conversionSettings.tr(),
                children: [
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_autoRetryOnFailure.tr(),
                    value: state.settings.autoRetryOnFailure,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateAutoRetryOnFailure(value),
                      );
                    },
                  ),
                  SettingsTextField(
                    label: LocaleKeys.importSettings_maxRetryAttempts.tr(),
                    value: state.settings.maxRetryAttempts.toString(),
                    onChanged: (value) {
                      final intValue = int.tryParse(value) ?? 3;
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateMaxRetryAttempts(intValue),
                      );
                    },
                    keyboardType: TextInputType.number,
                  ),
                ],
              ),
              
              // 通知设置
              SettingsCategory(
                title: LocaleKeys.importSettings_conversionSettings.tr(),
                children: [
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_enableProgressNotifications.tr(),
                    value: state.settings.enableProgressNotifications,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateEnableProgressNotifications(value),
                      );
                    },
                  ),
                ],
              ),
              
              // 日志设置
              SettingsCategory(
                title: LocaleKeys.importSettings_conversionSettings.tr(),
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: FlowyText.regular(
                          LocaleKeys.importSettings_logLevel.tr(),
                          fontSize: 16,
                        ),
                      ),
                      const SizedBox(width: 16),
                      Expanded(
                        child: SettingsDropdown<LogLevelPB>(
                          selectedOption: state.settings.logLevel,
                          options: LogLevelPB.values.map((level) => 
                            DropdownMenuEntry<LogLevelPB>(
                              value: level,
                              label: _getLogLevelLabel(level),
                            ),
                          ).toList(),
                          onChanged: (value) {
                            context.read<ImportSettingsBloc>().add(
                              ImportSettingsEvent.updateLogLevel(value),
                            );
                          },
                        ),
                      ),
                    ],
                  ),
                  SettingsSwitch(
                    label: "保存转换日志",
                    value: state.settings.saveConversionLogs,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateSaveConversionLogs(value),
                      );
                    },
                  ),
                  SettingsTextField(
                    label: LocaleKeys.importSettings_logRetentionDays.tr(),
                    value: state.settings.logRetentionDays.toString(),
                    onChanged: (value) {
                      final intValue = int.tryParse(value) ?? 30;
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateLogRetentionDays(intValue),
                      );
                    },
                    keyboardType: TextInputType.number,
                  ),
                ],
              ),
              
              // 路径设置
              SettingsCategory(
                title: LocaleKeys.importSettings_pathSettings.tr(),
                children: [
                  SettingsTextField(
                    label: LocaleKeys.importSettings_defaultImportPath.tr(),
                    value: state.settings.defaultImportPath,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateDefaultImportPath(value),
                      );
                    },
                  ),
                ],
              ),
              
              // 云端设置
              SettingsCategory(
                title: LocaleKeys.importSettings_cloudSettings.tr(),
                children: [
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_enableCloudSync.tr(),
                    value: state.settings.enableCloudSync,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateEnableCloudSync(value),
                      );
                    },
                    description: LocaleKeys.importSettings_descriptions_enableCloudSync.tr(),
                  ),
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_enableCloudBackup.tr(),
                    value: state.settings.enableCloudBackup,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateEnableCloudBackup(value),
                      );
                    },
                    description: LocaleKeys.importSettings_descriptions_enableCloudBackup.tr(),
                  ),
                  SettingsTextField(
                    label: LocaleKeys.importSettings_cloudStoragePath.tr(),
                    value: state.settings.cloudStoragePath,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateCloudStoragePath(value),
                      );
                    },
                    description: LocaleKeys.importSettings_descriptions_cloudStoragePath.tr(),
                  ),
                  SettingsSwitch(
                    label: LocaleKeys.importSettings_enableCompression.tr(),
                    value: state.settings.enableCompression,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateEnableCompression(value),
                      );
                    },
                    description: LocaleKeys.importSettings_descriptions_enableCompression.tr(),
                  ),
                ],
              ),
              
              // 高级设置
              SettingsCategory(
                title: LocaleKeys.importSettings_advancedSettings.tr(),
                children: [
                  SettingsSwitch(
                    label: "启用调试模式",
                    value: state.settings.enableDebugMode,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateEnableDebugMode(value),
                      );
                    },
                    description: "启用详细的调试信息和日志",
                  ),
                  SettingsSwitch(
                    label: "自动清理临时文件",
                    value: state.settings.autoCleanupTempFiles,
                    onChanged: (value) {
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateAutoCleanupTempFiles(value),
                      );
                    },
                    description: "转换完成后自动清理临时文件",
                  ),
                  SettingsTextField(
                    label: "临时文件保留时间（小时）",
                    value: state.settings.tempFileRetentionHours.toString(),
                    onChanged: (value) {
                      final intValue = int.tryParse(value) ?? 24;
                      context.read<ImportSettingsBloc>().add(
                        ImportSettingsEvent.updateTempFileRetentionHours(intValue),
                      );
                    },
                    keyboardType: TextInputType.number,
                    description: "临时文件在清理前的保留时间",
                  ),
                ],
              ),
              
              // 导入工具检查
              SettingsCategory(
                title: "导入工具检查",
                children: [
                  _ImportToolsStatusWidget(
                    toolsStatus: state.toolsStatus,
                    isChecking: state.isCheckingTools,
                    onCheck: () {
                      context.read<ImportSettingsBloc>().add(
                        const ImportSettingsEvent.checkImportTools(),
                      );
                    },
                  ),
                ],
              ),
            ],
          );
        },
      ),
    );
  }

  String _getLogLevelLabel(LogLevelPB level) {
    switch (level) {
      case LogLevelPB.Error:
        return "错误";
      case LogLevelPB.Warn:
        return "警告";
      case LogLevelPB.Info:
        return "信息";
      case LogLevelPB.Debug:
        return "调试";
      case LogLevelPB.Trace:
        return "跟踪";
      default:
        return "未知";
    }
  }
}

/// 导入工具状态显示组件
class _ImportToolsStatusWidget extends StatelessWidget {
  const _ImportToolsStatusWidget({
    required this.toolsStatus,
    required this.isChecking,
    required this.onCheck,
  });

  final ImportToolsStatusPB? toolsStatus;
  final bool isChecking;
  final VoidCallback onCheck;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: FlowyText.regular(
                "检查系统工具状态",
                fontSize: 16,
              ),
            ),
            const SizedBox(width: 16),
            ElevatedButton(
              onPressed: isChecking ? null : onCheck,
              child: isChecking
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Text("检查"),
            ),
          ],
        ),
        const SizedBox(height: 16),
        if (toolsStatus != null) ...[
          // 检查是否有缺失的工具
          BlocBuilder<ImportSettingsBloc, ImportSettingsState>(
            builder: (context, blocState) {
              final missingTools = toolsStatus!.tools
                  .where((tool) => tool.status == ImportToolStatusPB.ToolNotInstalled)
                  .map((tool) => tool.name)
                  .toList();
              
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (missingTools.isNotEmpty && !blocState.isInstallingTools) ...[
                    const SizedBox(height: 8),
                    ElevatedButton.icon(
                      onPressed: () {
                        context.read<ImportSettingsBloc>().add(
                          ImportSettingsEvent.installMissingTools(missingTools),
                        );
                      },
                      icon: const Icon(Icons.download, size: 16),
                      label: const Text("安装缺失组件"),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.blue,
                        foregroundColor: Colors.white,
                      ),
                    ),
                    const SizedBox(height: 16),
                  ],
                  
                  // 显示安装进度
                  if (blocState.isInstallingTools && blocState.installProgress != null) ...[
                    const SizedBox(height: 8),
                    Container(
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
                              const SizedBox(
                                width: 16,
                                height: 16,
                                child: CircularProgressIndicator(strokeWidth: 2),
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: FlowyText.semibold(
                                  "正在安装: ${blocState.installProgress!.toolName}",
                                  fontSize: 14,
                                  color: Colors.blue.shade900,
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 8),
                          LinearProgressIndicator(
                            value: blocState.installProgress!.progress,
                            backgroundColor: Colors.blue.shade100,
                            minHeight: 6,
                          ),
                          const SizedBox(height: 8),
                          FlowyText.regular(
                            blocState.installProgress!.message,
                            fontSize: 12,
                            color: Colors.blue.shade800,
                          ),
                          if (blocState.installProgress!.logs.isNotEmpty) ...[
                            const SizedBox(height: 8),
                            Container(
                              constraints: const BoxConstraints(maxHeight: 200),
                              child: SingleChildScrollView(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: blocState.installProgress!.logs.map((log) {
                                    return Padding(
                                      padding: const EdgeInsets.only(bottom: 4),
                                      child: FlowyText.regular(
                                        log,
                                        fontSize: 11,
                                        color: Colors.blue.shade700,
                                        fontFamily: 'monospace',
                                      ),
                                    );
                                  }).toList(),
                                ),
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                    const SizedBox(height: 16),
                  ],
                ],
              );
            },
          ),
          ...toolsStatus!.tools.map((tool) => _ToolStatusItem(tool: tool)),
          if (toolsStatus!.checkedAt > 0) ...[
            const SizedBox(height: 8),
            FlowyText.regular(
              "检查时间: ${DateTime.fromMillisecondsSinceEpoch(toolsStatus!.checkedAt.toInt() * 1000).toString().substring(0, 19)}",
              fontSize: 12,
              color: Colors.grey,
            ),
          ],
        ] else ...[
          FlowyText.regular(
            "点击\"检查\"按钮检查系统工具状态",
            fontSize: 14,
            color: Colors.grey,
          ),
        ],
      ],
    );
  }
}

/// 单个工具状态显示项
class _ToolStatusItem extends StatelessWidget {
  const _ToolStatusItem({required this.tool});

  final ImportToolInfoPB tool;

  @override
  Widget build(BuildContext context) {
    final statusColor = _getStatusColor(tool.status);
    final statusText = _getStatusText(tool.status);
    
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.shade300),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 12,
                height: 12,
                decoration: BoxDecoration(
                  color: statusColor,
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: FlowyText.semibold(
                  tool.name,
                  fontSize: 15,
                ),
              ),
              FlowyText.regular(
                statusText,
                fontSize: 13,
                color: statusColor,
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (tool.description.isNotEmpty) ...[
            FlowyText.regular(
              tool.description,
              fontSize: 13,
              color: Colors.grey.shade700,
            ),
            const SizedBox(height: 4),
          ],
          if (tool.hasVersion() && tool.version.isNotEmpty) ...[
            FlowyText.regular(
              "版本: ${tool.version}",
              fontSize: 12,
              color: Colors.grey.shade600,
            ),
            const SizedBox(height: 4),
          ],
          if (tool.hasPath() && tool.path.isNotEmpty) ...[
            FlowyText.regular(
              "路径: ${tool.path}",
              fontSize: 12,
              color: Colors.grey.shade600,
            ),
            const SizedBox(height: 4),
          ],
          if (tool.hasModelStatus() && tool.modelStatus.isNotEmpty) ...[
            const SizedBox(height: 4),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: _getModelStatusColor(tool.modelStatus).withOpacity(0.1),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: _getModelStatusColor(tool.modelStatus).withOpacity(0.3),
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(
                        _getModelStatusIcon(tool.modelStatus),
                        size: 16,
                        color: _getModelStatusColor(tool.modelStatus),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: FlowyText.semibold(
                          "模型状态",
                          fontSize: 13,
                          color: _getModelStatusColor(tool.modelStatus),
                        ),
                      ),
                      // 如果模型未就绪或部分就绪，显示下载按钮
                      if (tool.modelStatus.contains('⚠️') || 
                          tool.modelStatus.contains('部分就绪') || 
                          tool.modelStatus.contains('未下载') ||
                          (!tool.modelStatus.contains('✅') && !tool.modelStatus.contains('已就绪')))
                        Builder(
                          builder: (context) {
                            final bloc = context.read<ImportSettingsBloc>();
                            final isDownloading = bloc.state.isDownloadingModels;
                            return ElevatedButton.icon(
                              onPressed: isDownloading
                                  ? null
                                  : () {
                                      bloc.add(const ImportSettingsEvent.downloadMarkerModels(false));
                                    },
                              icon: isDownloading
                                  ? const SizedBox(
                                      width: 16,
                                      height: 16,
                                      child: CircularProgressIndicator(strokeWidth: 2),
                                    )
                                  : const Icon(Icons.download, size: 16),
                              label: Text(isDownloading ? "下载中..." : "下载模型"),
                              style: ElevatedButton.styleFrom(
                                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                                textStyle: const TextStyle(fontSize: 12),
                              ),
                            );
                          },
                        ),
                    ],
                  ),
                  const SizedBox(height: 6),
                  FlowyText.regular(
                    tool.modelStatus,
                    fontSize: 12,
                    color: Colors.grey.shade700,
                  ),
                  // 显示下载进度和日志
                  if (tool.name.contains('marker-pdf'))
                    Builder(
                      builder: (context) {
                        final bloc = context.read<ImportSettingsBloc>();
                        final progress = bloc.state.modelDownloadProgress;
                        final isDownloading = bloc.state.isDownloadingModels;
                        
                        if (progress != null || isDownloading) {
                          return Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              const SizedBox(height: 8),
                              // 进度条
                              if (progress != null && progress.progress > 0)
                                Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    LinearProgressIndicator(
                                      value: progress.progress,
                                      backgroundColor: Colors.grey.shade200,
                                      minHeight: 6,
                                    ),
                                    const SizedBox(height: 4),
                                    Row(
                                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                      children: [
                                        FlowyText.regular(
                                          "进度: ${(progress.progress * 100).toStringAsFixed(0)}%",
                                          fontSize: 11,
                                          color: Colors.grey.shade600,
                                        ),
                                        if (progress.downloadedBytes > 0)
                                          FlowyText.regular(
                                            "已下载: ${(progress.downloadedBytes.toInt() / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB",
                                            fontSize: 11,
                                            color: Colors.grey.shade600,
                                          ),
                                      ],
                                    ),
                                    const SizedBox(height: 8),
                                  ],
                                ),
                              // 状态消息和日志
                              Container(
                                padding: const EdgeInsets.all(8),
                                decoration: BoxDecoration(
                                  color: _getDownloadStatusColor(progress?.status).withOpacity(0.1),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(
                                    color: _getDownloadStatusColor(progress?.status).withOpacity(0.3),
                                  ),
                                ),
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Row(
                                      children: [
                                        Icon(
                                          _getDownloadStatusIcon(progress?.status, isDownloading),
                                          size: 14,
                                          color: _getDownloadStatusColor(progress?.status),
                                        ),
                                        const SizedBox(width: 6),
                                        Expanded(
                                          child: FlowyText.semibold(
                                            _getDownloadStatusText(progress?.status, isDownloading),
                                            fontSize: 12,
                                            color: _getDownloadStatusColor(progress?.status),
                                          ),
                                        ),
                                      ],
                                    ),
                                    if (progress != null && progress.message.isNotEmpty) ...[
                                      const SizedBox(height: 6),
                                      Container(
                                        constraints: const BoxConstraints(maxHeight: 200),
                                        child: SingleChildScrollView(
                                          child: FlowyText.regular(
                                            progress.message,
                                            fontSize: 10,
                                            color: Colors.grey.shade700,
                                            fontFamily: 'monospace',
                                          ),
                                        ),
                                      ),
                                    ] else if (isDownloading) ...[
                                      const SizedBox(height: 6),
                                      FlowyText.regular(
                                        "正在执行 marker 命令，触发模型下载...\n这可能需要 10-30 分钟，请耐心等待。",
                                        fontSize: 11,
                                        color: Colors.grey.shade600,
                                      ),
                                    ],
                                  ],
                                ),
                              ),
                            ],
                          );
                        }
                        return const SizedBox.shrink();
                      },
                    ),
                ],
              ),
            ),
          ],
          if (tool.hasInstallInstruction() && tool.installInstruction.isNotEmpty) ...[
            const SizedBox(height: 4),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: Colors.orange.shade50,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: Colors.orange.shade200),
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(
                    Icons.info_outline,
                    size: 16,
                    color: Colors.orange.shade700,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: FlowyText.regular(
                      "安装: ${tool.installInstruction}",
                      fontSize: 12,
                      color: Colors.orange.shade900,
                    ),
                  ),
                ],
              ),
            ),
          ],
          if (tool.checkLogs.isNotEmpty) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey.shade50,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: Colors.grey.shade300),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(
                        Icons.description_outlined,
                        size: 16,
                        color: Colors.grey.shade700,
                      ),
                      const SizedBox(width: 8),
                      FlowyText.semibold(
                        "检查日志",
                        fontSize: 13,
                        color: Colors.grey.shade800,
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Container(
                    constraints: const BoxConstraints(maxHeight: 300),
                    child: SingleChildScrollView(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: tool.checkLogs.map((log) {
                          final isSuccess = log.contains('✓');
                          final isError = log.contains('✗') || log.contains('✗');
                          final isWarning = log.contains('⚠');
                          final isIndented = log.startsWith('  ');
                          
                          Color textColor = Colors.grey.shade700;
                          if (isSuccess) {
                            textColor = Colors.green.shade700;
                          } else if (isError) {
                            textColor = Colors.red.shade700;
                          } else if (isWarning) {
                            textColor = Colors.orange.shade700;
                          }
                          
                          return Padding(
                            padding: EdgeInsets.only(
                              bottom: 4,
                              left: isIndented ? 16.0 : 0.0,
                            ),
                            child: FlowyText.regular(
                              log,
                              fontSize: 11,
                              color: textColor,
                              fontFamily: 'monospace',
                            ),
                          );
                        }).toList(),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Color _getModelStatusColor(String modelStatus) {
    if (modelStatus.contains('✅') || modelStatus.contains('已就绪')) {
      return Colors.green;
    } else if (modelStatus.contains('⚠️') || modelStatus.contains('部分就绪')) {
      return Colors.orange;
    } else if (modelStatus.contains('未下载')) {
      return Colors.red;
    }
    return Colors.grey;
  }

  IconData _getModelStatusIcon(String modelStatus) {
    if (modelStatus.contains('✅') || modelStatus.contains('已就绪')) {
      return Icons.check_circle_outline;
    } else if (modelStatus.contains('⚠️') || modelStatus.contains('部分就绪')) {
      return Icons.warning_amber_rounded;
    } else if (modelStatus.contains('未下载')) {
      return Icons.error_outline;
    }
    return Icons.info_outline;
  }

  Color _getStatusColor(ImportToolStatusPB status) {
    switch (status) {
      case ImportToolStatusPB.ToolAvailable:
        return Colors.green;
      case ImportToolStatusPB.ToolNotInstalled:
        return Colors.red;
      case ImportToolStatusPB.ToolUnavailable:
        return Colors.orange;
      case ImportToolStatusPB.ToolUnknown:
      default:
        return Colors.grey;
    }
  }

  String _getStatusText(ImportToolStatusPB status) {
    switch (status) {
      case ImportToolStatusPB.ToolAvailable:
        return "已安装";
      case ImportToolStatusPB.ToolNotInstalled:
        return "未安装";
      case ImportToolStatusPB.ToolUnavailable:
        return "不可用";
      case ImportToolStatusPB.ToolUnknown:
      default:
        return "未知";
    }
  }

  Color _getDownloadStatusColor(ModelDownloadStatusPB? status) {
    if (status == null) {
      return Colors.blue;
    }
    switch (status) {
      case ModelDownloadStatusPB.ModelDownloadDownloading:
        return Colors.blue;
      case ModelDownloadStatusPB.ModelDownloadCompleted:
        return Colors.green;
      case ModelDownloadStatusPB.ModelDownloadFailed:
        return Colors.red;
      case ModelDownloadStatusPB.ModelDownloadCancelled:
        return Colors.orange;
      case ModelDownloadStatusPB.ModelDownloadIdle:
      default:
        return Colors.grey;
    }
  }

  IconData _getDownloadStatusIcon(ModelDownloadStatusPB? status, bool isDownloading) {
    if (isDownloading || status == ModelDownloadStatusPB.ModelDownloadDownloading) {
      return Icons.download;
    }
    if (status == null) {
      return Icons.info_outline;
    }
    switch (status) {
      case ModelDownloadStatusPB.ModelDownloadCompleted:
        return Icons.check_circle;
      case ModelDownloadStatusPB.ModelDownloadFailed:
        return Icons.error;
      case ModelDownloadStatusPB.ModelDownloadCancelled:
        return Icons.cancel;
      case ModelDownloadStatusPB.ModelDownloadIdle:
      default:
        return Icons.info_outline;
    }
  }

  String _getDownloadStatusText(ModelDownloadStatusPB? status, bool isDownloading) {
    if (isDownloading || status == ModelDownloadStatusPB.ModelDownloadDownloading) {
      return "正在下载模型...";
    }
    if (status == null) {
      return "准备下载";
    }
    switch (status) {
      case ModelDownloadStatusPB.ModelDownloadCompleted:
        return "下载完成";
      case ModelDownloadStatusPB.ModelDownloadFailed:
        return "下载失败";
      case ModelDownloadStatusPB.ModelDownloadCancelled:
        return "下载已取消";
      case ModelDownloadStatusPB.ModelDownloadIdle:
      default:
        return "等待下载";
    }
  }
}

