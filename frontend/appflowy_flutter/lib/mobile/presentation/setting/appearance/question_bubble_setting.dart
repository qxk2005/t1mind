import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/appearance/appearance_cubit.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../setting.dart';

class QuestionBubbleSetting extends StatelessWidget {
  const QuestionBubbleSetting({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AppearanceSettingsCubit, AppearanceSettingsState>(
      builder: (context, state) {
        return MobileSettingItem(
          name: LocaleKeys.settings_appearance_questionBubble_label.tr(),
          trailing: Switch(
            value: state.showQuestionBubble,
            onChanged: (value) {
              context.read<AppearanceSettingsCubit>().setShowQuestionBubble(value);
            },
          ),
        );
      },
    );
  }
}
