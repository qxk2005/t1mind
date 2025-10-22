import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/changelog_widget.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/update_check_widget.dart';
import 'package:appflowy/workspace/presentation/settings/pages/about/version_info_widget.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category_spacer.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// T1Mind关于页面
/// 整合所有子组件创建完整的关于页面体验
/// 使用SettingsBody布局和AppFlowy主题
/// 提供版本信息、更新检查、更新历史等功能
class AboutT1MindPage extends StatelessWidget {
  const AboutT1MindPage({super.key});

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: LocaleKeys.settings_aboutT1mindPage_title.tr(),
      description: LocaleKeys.settings_aboutT1mindPage_description.tr(),
      autoSeparate: false,
      children: [
        // 版本信息部分
        const VersionInfoWidget(),
        const SettingsCategorySpacer(),
        
        // 更新检查部分
        const UpdateCheckWidget(),
        const SettingsCategorySpacer(),
        
        // 更新历史部分
        const ChangelogWidget(),
        
        // 底部间距
        const VSpace(32),
        
        // 版权信息
        _buildCopyrightSection(context),
      ],
    );
  }

  /// 构建版权信息部分
  Widget _buildCopyrightSection(BuildContext context) {
    return SettingsCategory(
      title: LocaleKeys.settings_aboutT1mindPage_copyright_title.tr(),
      children: [
        _buildCopyrightItem(
          context,
          LocaleKeys.settings_aboutT1mindPage_copyright_t1mind.tr(),
          LocaleKeys.settings_aboutT1mindPage_copyright_t1mindDescription.tr(),
        ),
        const VSpace(8),
        _buildCopyrightItem(
          context,
          LocaleKeys.settings_aboutT1mindPage_copyright_appflowy.tr(),
          LocaleKeys.settings_aboutT1mindPage_copyright_appflowyDescription.tr(),
        ),
        const VSpace(8),
        _buildCopyrightItem(
          context,
          LocaleKeys.settings_aboutT1mindPage_copyright_flutter.tr(),
          LocaleKeys.settings_aboutT1mindPage_copyright_flutterDescription.tr(),
        ),
      ],
    );
  }

  /// 构建版权信息项
  Widget _buildCopyrightItem(
    BuildContext context,
    String name,
    String description,
  ) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          flex: 2,
          child: Text(
            name,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              fontWeight: FontWeight.w500,
            ),
          ),
        ),
        const HSpace(16),
        Expanded(
          flex: 3,
          child: Text(
            description,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).hintColor,
            ),
          ),
        ),
      ],
    );
  }
}
