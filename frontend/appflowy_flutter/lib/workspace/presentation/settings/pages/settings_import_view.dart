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
        ],
      ),
    );
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
}

