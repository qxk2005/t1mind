import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/local_ai_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/model_selection.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart';
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
        return LocaleKeys.settings_aiPage_keys_ollamaLocal.tr();
      case GlobalModelType.openaiCompatible:
        return LocaleKeys.settings_aiPage_keys_openaiCompatible.tr();
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
          const _GlobalModelTypeSelectionWithPanel(),
          const AIModelSelection(),
          const _AISearchToggle(value: false),
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

/// 全局模型类型选择器和相应的配置面板
class _GlobalModelTypeSelectionWithPanel extends StatefulWidget {
  const _GlobalModelTypeSelectionWithPanel();

  @override
  State<_GlobalModelTypeSelectionWithPanel> createState() => _GlobalModelTypeSelectionWithPanelState();
}

class _GlobalModelTypeSelectionWithPanelState extends State<_GlobalModelTypeSelectionWithPanel> {
  // 临时状态管理，默认选择 Ollama 本地
  GlobalModelType selectedType = GlobalModelType.ollama;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 全局模型类型选择器
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 6),
          child: Row(
            children: [
              Expanded(
                child: FlowyText.medium(
                  LocaleKeys.settings_aiPage_keys_globalModelType.tr(),
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
        ),
        
        // 根据选择的模型类型显示对应的配置面板
        const SizedBox(height: 8),
        _buildConfigurationPanel(),
      ],
    );
  }

  /// 根据选择的全局模型类型构建相应的配置面板
  Widget _buildConfigurationPanel() {
    switch (selectedType) {
      case GlobalModelType.ollama:
        return const LocalAISetting();
      case GlobalModelType.openaiCompatible:
        return const OpenAICompatibleSetting();
    }
  }
}
