import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/local_ai_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart';
import 'package:appflowy/workspace/presentation/settings/shared/af_dropdown_menu_entry.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_dropdown.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class LocalSettingsAIView extends StatelessWidget {
  const LocalSettingsAIView({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SettingsAIBloc>(
      create: (_) => SettingsAIBloc(userProfile, workspaceId)
        ..add(const SettingsAIEvent.started()),
      child: SettingsBody(
        title: LocaleKeys.settings_aiPage_title.tr(),
        description: "",
        children: [
          const _LocalGlobalModelTypeSelection(),
          const LocalAISetting(),
        ],
      ),
    );
  }
}

class _LocalGlobalModelTypeSelection extends StatefulWidget {
  const _LocalGlobalModelTypeSelection();

  @override
  State<_LocalGlobalModelTypeSelection> createState() => _LocalGlobalModelTypeSelectionState();
}

class _LocalGlobalModelTypeSelectionState extends State<_LocalGlobalModelTypeSelection> {
  // 临时状态管理，默认选择 Ollama 本地
  GlobalModelType selectedType = GlobalModelType.ollama;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          Expanded(
            child: FlowyText.medium(
              '全局使用的模型类型', // TODO: 使用国际化 LocaleKeys.settings_aiPage_keys_globalModelType.tr()
              overflow: TextOverflow.ellipsis,
            ),
          ),
          Flexible(
            child: SettingsDropdown<GlobalModelType>(
              selectedOption: selectedType,
              onChanged: (type) {
                setState(() {
                  selectedType = type;
                });
                // TODO: 在后续任务中连接到 BLoC
              },
              options: GlobalModelType.values
                  .map(
                    (type) => buildDropdownMenuEntry<GlobalModelType>(
                      context,
                      value: type,
                      label: type.displayName,
                      selectedValue: selectedType,
                    ),
                  )
                  .toList(),
            ),
          ),
        ],
      ),
    );
  }
}
