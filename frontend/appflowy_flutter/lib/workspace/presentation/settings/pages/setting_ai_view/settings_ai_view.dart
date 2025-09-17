import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/local_ai_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/model_selection.dart';
import 'package:appflowy/workspace/presentation/settings/shared/af_dropdown_menu_entry.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_dropdown.dart';
import 'package:appflowy/workspace/presentation/widgets/toggle/toggle.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

// 临时的全局模型类型枚举
enum GlobalModelType {
  ollama,
  openaiCompatible,
}

extension GlobalModelTypeExtension on GlobalModelType {
  String get displayName {
    switch (this) {
      case GlobalModelType.ollama:
        return 'Ollama 本地'; // TODO: 使用国际化
      case GlobalModelType.openaiCompatible:
        return 'OpenAI 兼容服务器'; // TODO: 使用国际化
    }
  }
}

class SettingsAIView extends StatelessWidget {
  const SettingsAIView({
    super.key,
    required this.userProfile,
    required this.currentWorkspaceMemberRole,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final AFRolePB? currentWorkspaceMemberRole;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SettingsAIBloc>(
      create: (_) => SettingsAIBloc(userProfile, workspaceId)
        ..add(const SettingsAIEvent.started()),
      child: SettingsBody(
        title: LocaleKeys.settings_aiPage_title.tr(),
        description: LocaleKeys.settings_aiPage_keys_aiSettingsDescription.tr(),
        children: [
          const _GlobalModelTypeSelection(),
          const AIModelSelection(),
          const _AISearchToggle(value: false),
          const LocalAISetting(),
        ],
      ),
    );
  }
}

class _AISearchToggle extends StatelessWidget {
  const _AISearchToggle({required this.value});

  final bool value;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            FlowyText.medium(
              LocaleKeys.settings_aiPage_keys_enableAISearchTitle.tr(),
            ),
            const Spacer(),
            BlocBuilder<SettingsAIBloc, SettingsAIState>(
              builder: (context, state) {
                if (state.aiSettings == null) {
                  return const Padding(
                    padding: EdgeInsets.only(top: 6),
                    child: SizedBox(
                      height: 26,
                      width: 26,
                      child: CircularProgressIndicator.adaptive(),
                    ),
                  );
                } else {
                  return Toggle(
                    value: state.enableSearchIndexing,
                    onChanged: (_) => context
                        .read<SettingsAIBloc>()
                        .add(const SettingsAIEvent.toggleAISearch()),
                  );
                }
              },
            ),
          ],
        ),
      ],
    );
  }
}

class _GlobalModelTypeSelection extends StatefulWidget {
  const _GlobalModelTypeSelection();

  @override
  State<_GlobalModelTypeSelection> createState() => _GlobalModelTypeSelectionState();
}

class _GlobalModelTypeSelectionState extends State<_GlobalModelTypeSelection> {
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
