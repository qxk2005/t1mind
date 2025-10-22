import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// T1Mind版本信息显示组件
/// 显示当前版本信息，包括应用版本、构建号、平台信息等
/// 使用t1mind品牌而不是AppFlowy品牌
class VersionInfoWidget extends StatefulWidget {
  const VersionInfoWidget({super.key});

  @override
  State<VersionInfoWidget> createState() => _VersionInfoWidgetState();
}

class _VersionInfoWidgetState extends State<VersionInfoWidget> {
  T1MindUpdateInfo? _updateInfo;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadUpdateInfo();
  }

  Future<void> _loadUpdateInfo() async {
    try {
      final updateInfo = await T1MindVersionChecker().getUpdateInfo();
      if (mounted) {
        setState(() {
          _updateInfo = updateInfo;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return SettingsCategory(
      title: LocaleKeys.settings_aboutT1mindPage_versionInfo_title.tr(),
      description: LocaleKeys.settings_aboutT1mindPage_versionInfo_description.tr(),
      children: [
        _buildVersionSection(context, theme),
        const VSpace(16),
        _buildSystemInfoSection(context, theme),
        if (_updateInfo != null && _updateInfo!.isUpdateAvailable) ...[
          const VSpace(16),
          _buildUpdateSection(context, theme),
        ],
      ],
    );
  }

  Widget _buildVersionSection(BuildContext context, AppFlowyThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildInfoRow(
          context: context,
          label: LocaleKeys.settings_aboutT1mindPage_versionInfo_t1mindVersion.tr(),
          value: ApplicationInfo.applicationVersion,
          theme: theme,
        ),
        const VSpace(8),
        _buildInfoRow(
          context: context,
          label: LocaleKeys.settings_aboutT1mindPage_versionInfo_buildNumber.tr(),
          value: ApplicationInfo.buildNumber,
          theme: theme,
        ),
        if (_isLoading) ...[
          const VSpace(8),
          const SizedBox(
            height: 16,
            width: 16,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        ] else if (_updateInfo != null) ...[
          const VSpace(8),
          _buildInfoRow(
            context: context,
            label: LocaleKeys.settings_aboutT1mindPage_versionInfo_latestVersion.tr(),
            value: _updateInfo!.latestVersion,
            theme: theme,
            isHighlighted: _updateInfo!.isUpdateAvailable,
          ),
        ],
      ],
    );
  }

  Widget _buildSystemInfoSection(BuildContext context, AppFlowyThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aboutT1mindPage_versionInfo_systemInfo.tr(),
          fontSize: 14,
          color: theme.textColorScheme.secondary,
        ),
        const VSpace(8),
        _buildInfoRow(
          context: context,
          label: LocaleKeys.settings_aboutT1mindPage_versionInfo_operatingSystem.tr(),
          value: _getOSDisplayName(),
          theme: theme,
        ),
        const VSpace(4),
        _buildInfoRow(
          context: context,
          label: LocaleKeys.settings_aboutT1mindPage_versionInfo_architecture.tr(),
          value: ApplicationInfo.architecture,
          theme: theme,
        ),
        if (ApplicationInfo.macOSMajorVersion != null) ...[
          const VSpace(4),
          _buildInfoRow(
            context: context,
            label: LocaleKeys.settings_aboutT1mindPage_versionInfo_macosVersion.tr(),
            value: '${ApplicationInfo.macOSMajorVersion}.${ApplicationInfo.macOSMinorVersion}',
            theme: theme,
          ),
        ],
        if (ApplicationInfo.androidSDKVersion != -1) ...[
          const VSpace(4),
          _buildInfoRow(
            context: context,
            label: LocaleKeys.settings_aboutT1mindPage_versionInfo_androidSDK.tr(),
            value: '${ApplicationInfo.androidSDKVersion}',
            theme: theme,
          ),
        ],
      ],
    );
  }

  Widget _buildUpdateSection(BuildContext context, AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.primaryContainer.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.3),
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 8,
                height: 8,
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.primary,
                  shape: BoxShape.circle,
                ),
              ),
              const HSpace(8),
              Expanded(
                child: FlowyText.medium(
                  LocaleKeys.settings_aboutT1mindPage_versionInfo_updateAvailable.tr(),
                  fontSize: 14,
                  color: theme.textColorScheme.primary,
                ),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            LocaleKeys.settings_aboutT1mindPage_versionInfo_currentVersionToLatest.tr(
              namedArgs: {
                'currentVersion': _updateInfo!.currentVersion,
                'latestVersion': _updateInfo!.latestVersion,
              },
            ),
            fontSize: 12,
            color: theme.textColorScheme.secondary,
          ),
        ],
      ),
    );
  }

  Widget _buildInfoRow({
    required BuildContext context,
    required String label,
    required String value,
    required AppFlowyThemeData theme,
    bool isHighlighted = false,
  }) {
    return Row(
      children: [
        SizedBox(
          width: 100,
          child: FlowyText.regular(
            label,
            fontSize: 12,
            color: theme.textColorScheme.secondary,
          ),
        ),
        const HSpace(16),
        Expanded(
          child: FlowyText.medium(
            value,
            fontSize: 12,
            color: isHighlighted 
                ? Theme.of(context).colorScheme.primary 
                : theme.textColorScheme.primary,
          ),
        ),
      ],
    );
  }

  String _getOSDisplayName() {
    switch (ApplicationInfo.os) {
      case 'macos':
        return 'macOS';
      case 'windows':
        return 'Windows';
      case 'linux':
        return 'Linux';
      case 'android':
        return 'Android';
      case 'ios':
        return 'iOS';
      default:
        return ApplicationInfo.os.isNotEmpty ? ApplicationInfo.os : '未知';
    }
  }
}
