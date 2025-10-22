import 'package:appflowy/core/helpers/url_launcher.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// T1Mind更新检查组件
/// 显示更新检查和下载链接的UI组件
/// 集成T1MindVersionChecker和URL启动器
/// 处理网络状态和离线情况
class UpdateCheckWidget extends StatefulWidget {
  const UpdateCheckWidget({super.key});

  @override
  State<UpdateCheckWidget> createState() => _UpdateCheckWidgetState();
}

class _UpdateCheckWidgetState extends State<UpdateCheckWidget> {
  T1MindUpdateInfo? _updateInfo;
  bool _isChecking = false;
  String? _errorMessage;
  bool _hasChecked = false;

  @override
  void initState() {
    super.initState();
    // 不自动检查更新，等待用户手动触发
  }

  /// 检查更新
  Future<void> _checkForUpdates() async {
    if (_isChecking) return;

    setState(() {
      _isChecking = true;
      _errorMessage = null;
    });

    try {
      final updateInfo = await T1MindVersionChecker().getUpdateInfo();
      if (mounted) {
        setState(() {
          _updateInfo = updateInfo;
          _isChecking = false;
          _hasChecked = true;
          _errorMessage = null;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isChecking = false;
          _hasChecked = true;
          _errorMessage = LocaleKeys.settings_aboutT1mindPage_updateCheck_checkFailed.tr(namedArgs: {'error': e.toString()});
        });
      }
    }
  }

  /// 打开下载链接
  Future<void> _openDownloadLink() async {
    if (_updateInfo?.downloadUrl.isEmpty ?? true) return;

    try {
      final success = await afLaunchUrlString(
        _updateInfo!.downloadUrl,
        context: context,
        onFailure: (uri) {
          if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text(LocaleKeys.settings_aboutT1mindPage_updateCheck_cannotOpenDownloadLink.tr(namedArgs: {'uri': uri.toString()})),
                backgroundColor: Theme.of(context).colorScheme.error,
              ),
            );
          }
        },
      );

      if (!success && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(LocaleKeys.settings_aboutT1mindPage_updateCheck_cannotOpenDownloadLinkGeneric.tr()),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(LocaleKeys.settings_aboutT1mindPage_updateCheck_errorOpeningDownloadLink.tr(namedArgs: {'error': e.toString()})),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }

  /// 打开发布说明链接
  Future<void> _openReleaseNotes() async {
    if (_updateInfo?.releaseNotesUrl.isEmpty ?? true) return;

    try {
      final success = await afLaunchUrlString(
        _updateInfo!.releaseNotesUrl,
        context: context,
        onFailure: (uri) {
          if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text(LocaleKeys.settings_aboutT1mindPage_updateCheck_cannotOpenReleaseNotes.tr(namedArgs: {'uri': uri.toString()})),
                backgroundColor: Theme.of(context).colorScheme.error,
              ),
            );
          }
        },
      );

      if (!success && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(LocaleKeys.settings_aboutT1mindPage_updateCheck_cannotOpenReleaseNotesGeneric.tr()),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(LocaleKeys.settings_aboutT1mindPage_updateCheck_errorOpeningReleaseNotes.tr(namedArgs: {'error': e.toString()})),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return SettingsCategory(
      title: LocaleKeys.settings_aboutT1mindPage_updateCheck_title.tr(),
      description: LocaleKeys.settings_aboutT1mindPage_updateCheck_description.tr(),
      children: [
        _buildCheckButton(context, theme),
        if (_isChecking) ...[
          const VSpace(16),
          _buildLoadingIndicator(context, theme),
        ],
        if (_errorMessage != null) ...[
          const VSpace(16),
          _buildErrorMessage(context, theme),
        ],
        if (_hasChecked && _updateInfo != null) ...[
          const VSpace(16),
          _buildUpdateResult(context, theme),
        ],
      ],
    );
  }

  Widget _buildCheckButton(BuildContext context, AppFlowyThemeData theme) {
    return Row(
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.medium(
                LocaleKeys.settings_aboutT1mindPage_updateCheck_checkForUpdates.tr(),
                fontSize: 14,
                color: theme.textColorScheme.primary,
              ),
              const VSpace(4),
              FlowyText.regular(
                LocaleKeys.settings_aboutT1mindPage_updateCheck_checkingUpdatesDescription.tr(),
                fontSize: 12,
                color: theme.textColorScheme.secondary,
              ),
            ],
          ),
        ),
        const HSpace(16),
        SizedBox(
          height: 32,
          child: FlowyTextButton(
            _isChecking ? LocaleKeys.settings_aboutT1mindPage_updateCheck_checkingUpdates.tr() : LocaleKeys.settings_aboutT1mindPage_updateCheck_checkForUpdates.tr(),
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 7),
            fillColor: _isChecking 
                ? Theme.of(context).colorScheme.surface 
                : Theme.of(context).colorScheme.primary,
            radius: BorderRadius.circular(8),
            hoverColor: _isChecking 
                ? Theme.of(context).colorScheme.surface 
                : Theme.of(context).colorScheme.primary.withValues(alpha: 0.9),
            fontColor: _isChecking 
                ? theme.textColorScheme.secondary 
                : Colors.white,
            fontHoverColor: _isChecking 
                ? theme.textColorScheme.secondary 
                : Colors.white,
            fontSize: 12,
            onPressed: _isChecking ? null : _checkForUpdates,
            lineHeight: 1.0,
          ),
        ),
      ],
    );
  }

  Widget _buildLoadingIndicator(BuildContext context, AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline,
          width: 1,
        ),
      ),
      child: Row(
        children: [
          SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              color: theme.iconColorScheme.primary,
            ),
          ),
          const HSpace(12),
          Expanded(
            child: FlowyText.regular(
              LocaleKeys.settings_aboutT1mindPage_updateCheck_checkingUpdatesInProgress.tr(),
              fontSize: 14,
              color: theme.textColorScheme.primary,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildErrorMessage(BuildContext context, AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.errorContainer.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.error.withValues(alpha: 0.3),
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(
            Icons.error_outline,
            size: 16,
            color: Theme.of(context).colorScheme.error,
          ),
          const HSpace(8),
          Expanded(
            child: FlowyText.regular(
              _errorMessage!,
              fontSize: 12,
              color: Theme.of(context).colorScheme.error,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildUpdateResult(BuildContext context, AppFlowyThemeData theme) {
    if (_updateInfo!.latestVersion == 'No releases available') {
      return _buildNoReleasesAvailable(context, theme);
    } else if (_updateInfo!.isUpdateAvailable) {
      return _buildUpdateAvailable(context, theme);
    } else {
      return _buildUpToDate(context, theme);
    }
  }

  Widget _buildUpdateAvailable(BuildContext context, AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
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
                  LocaleKeys.settings_aboutT1mindPage_updateCheck_updateAvailable.tr(),
                  fontSize: 14,
                  color: theme.textColorScheme.primary,
                ),
              ),
            ],
          ),
          const VSpace(8),
          FlowyText.regular(
            '当前版本: ${_updateInfo!.currentVersion}',
            fontSize: 12,
            color: theme.textColorScheme.secondary,
          ),
          const VSpace(2),
          FlowyText.regular(
            '最新版本: ${_updateInfo!.latestVersion}',
            fontSize: 12,
            color: theme.textColorScheme.secondary,
          ),
          const VSpace(16),
          Row(
            children: [
              Expanded(
                child: SizedBox(
                  height: 32,
                  child: FlowyTextButton(
                    LocaleKeys.settings_aboutT1mindPage_updateCheck_downloadUpdate.tr(),
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 7),
                    fillColor: Theme.of(context).colorScheme.primary,
                    radius: BorderRadius.circular(8),
                    hoverColor: Theme.of(context).colorScheme.primary.withValues(alpha: 0.8),
                    fontColor: Theme.of(context).colorScheme.onPrimary,
                    fontHoverColor: Theme.of(context).colorScheme.onPrimary,
                    fontSize: 12,
                    onPressed: _openDownloadLink,
                    lineHeight: 1.0,
                  ),
                ),
              ),
              const HSpace(8),
              SizedBox(
                height: 32,
                child: FlowyTextButton(
                  LocaleKeys.settings_aboutT1mindPage_updateCheck_viewReleaseNotes.tr(),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 7),
                  fillColor: Colors.transparent,
                  radius: BorderRadius.circular(8),
                  hoverColor: theme.fillColorScheme.contentHover,
                  fontColor: theme.textColorScheme.primary,
                  fontHoverColor: theme.textColorScheme.primary,
                  borderColor: Theme.of(context).colorScheme.outline,
                  fontSize: 12,
                  onPressed: _openReleaseNotes,
                  lineHeight: 1.0,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildNoReleasesAvailable(BuildContext context, AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline,
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(
            Icons.info_outline,
            size: 16,
            color: theme.textColorScheme.secondary,
          ),
          const HSpace(8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  '暂无可用版本',
                  fontSize: 14,
                  color: theme.textColorScheme.primary,
                ),
                const VSpace(2),
                FlowyText.regular(
                  '当前版本: ${_updateInfo!.currentVersion}',
                  fontSize: 12,
                  color: theme.textColorScheme.secondary,
                ),
                const VSpace(4),
                FlowyText.regular(
                  'GitHub仓库存在但暂无发布版本',
                  fontSize: 12,
                  color: theme.textColorScheme.secondary,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildUpToDate(BuildContext context, AppFlowyThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline,
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(
            Icons.check_circle_outline,
            size: 16,
            color: Theme.of(context).colorScheme.primary,
          ),
          const HSpace(8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  LocaleKeys.settings_aboutT1mindPage_updateCheck_upToDate.tr(),
                  fontSize: 14,
                  color: theme.textColorScheme.primary,
                ),
                const VSpace(2),
                FlowyText.regular(
                  '当前版本: ${_updateInfo!.currentVersion}',
                  fontSize: 12,
                  color: theme.textColorScheme.secondary,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
