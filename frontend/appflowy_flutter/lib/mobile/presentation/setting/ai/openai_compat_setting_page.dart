import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/openai_compat_setting.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';

class OpenAICompatSettingMobilePage extends StatelessWidget {
  const OpenAICompatSettingMobilePage({super.key, this.workspaceId});

  static const routeName = '/settings/ai/openai_compat';

  final String? workspaceId;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('settings.aiPage.keys.openAICompatTitle'.tr())),
      body: ListView(
        padding: const EdgeInsets.all(12),
        children: [
          MobileSettingGroup(
            groupTitle: '',
            settingItemList: [
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: OpenAICompatSetting(workspaceId: workspaceId),
              ),
            ],
          ),
        ],
      ),
    );
  }
}


