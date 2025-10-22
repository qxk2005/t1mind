import 'package:appflowy/core/helpers/url_launcher.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

class SettingsAppVersion extends StatefulWidget {
  const SettingsAppVersion({
    super.key,
  });

  @override
  State<SettingsAppVersion> createState() => _SettingsAppVersionState();
}

class _SettingsAppVersionState extends State<SettingsAppVersion> {
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
    if (_isLoading) {
      return _buildLoadingState(context);
    }

    if (_updateInfo?.isUpdateAvailable == true && 
        _updateInfo!.latestVersion != 'No releases available') {
      return _UpdateAppSection(
        updateInfo: _updateInfo!,
        onRefresh: _loadUpdateInfo,
      );
    } else {
      return _buildIsUpToDate(context);
    }
  }

  Widget _buildLoadingState(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Row(
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
            '检查更新中...',
            fontSize: 14,
            color: theme.textColorScheme.primary,
          ),
        ),
      ],
    );
  }

  Widget _buildIsUpToDate(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          LocaleKeys.settings_accountPage_isUpToDate.tr(),
          style: theme.textStyle.body.enhanced(
            color: theme.textColorScheme.primary,
          ),
        ),
        VSpace(theme.spacing.s),
        Text(
          LocaleKeys.settings_accountPage_officialVersion.tr(
            namedArgs: {
              'version': ApplicationInfo.applicationVersion,
            },
          ),
          style: theme.textStyle.caption.standard(
            color: theme.textColorScheme.secondary,
          ),
        ),
      ],
    );
  }
}

class _UpdateAppSection extends StatelessWidget {
  const _UpdateAppSection({
    required this.updateInfo,
    required this.onRefresh,
  });

  final T1MindUpdateInfo updateInfo;
  final VoidCallback onRefresh;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(child: _buildDescription(context)),
        _buildUpdateButton(),
      ],
    );
  }

  Widget _buildUpdateButton() {
    return PrimaryRoundedButton(
      text: '更新',
      margin: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
      fontWeight: FontWeight.w500,
      radius: 8.0,
      onTap: () {
        Log.info('[T1MindUpdater] Opening download link');
        if (updateInfo.downloadUrl.isNotEmpty) {
          afLaunchUrlString(updateInfo.downloadUrl);
        }
      },
    );
  }

  Widget _buildDescription(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            _buildRedDot(),
            const HSpace(6),
            Flexible(
              child: FlowyText.medium(
                '新版本 (${updateInfo.latestVersion}) 可用！',
                figmaLineHeight: 17,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        const VSpace(4),
        _buildCurrentVersionAndLatestVersion(context),
      ],
    );
  }

  Widget _buildCurrentVersionAndLatestVersion(BuildContext context) {
    return Row(
      children: [
        Flexible(
          child: Opacity(
            opacity: 0.7,
            child: FlowyText.regular(
              '当前版本: ${updateInfo.currentVersion} -> ${updateInfo.latestVersion}',
              fontSize: 12,
              figmaLineHeight: 13,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ),
        const HSpace(6),
        MouseRegion(
          cursor: SystemMouseCursors.click,
          child: GestureDetector(
            onTap: () {
              if (updateInfo.releaseNotesUrl.isNotEmpty) {
                afLaunchUrlString(updateInfo.releaseNotesUrl);
              }
            },
            child: FlowyText.regular(
              '查看更新内容',
              decoration: TextDecoration.underline,
              color: Theme.of(context).colorScheme.primary,
              fontSize: 12,
              figmaLineHeight: 13,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildRedDot() {
    return Container(
      width: 8,
      height: 8,
      decoration: const BoxDecoration(
        color: Color(0xFFFB006D),
        shape: BoxShape.circle,
      ),
    );
  }
}
