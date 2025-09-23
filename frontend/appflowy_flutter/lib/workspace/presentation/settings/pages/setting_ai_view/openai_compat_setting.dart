import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra/size.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

class OpenAICompatSetting extends StatelessWidget {
  const OpenAICompatSetting({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          "settings.aiPage.keys.openAICompatTitle".tr(),
          style: theme.textStyle.body.enhanced(
            color: theme.textColorScheme.primary,
          ),
        ),
        const VSpace(4),
        FlowyText(
          "settings.aiPage.keys.openAICompatSubTitle".tr(),
          maxLines: 3,
          fontSize: 12,
        ),
        const VSpace(10),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            border: Border.all(color: Theme.of(context).colorScheme.outline),
            borderRadius: Corners.s8Border,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.medium(
                "settings.aiPage.keys.openAICompatComingSoon".tr(),
              ),
            ],
          ),
        ),
      ],
    );
  }
}


