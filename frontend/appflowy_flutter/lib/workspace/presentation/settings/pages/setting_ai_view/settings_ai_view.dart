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
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

extension GlobalAIModelTypePBExtension on GlobalAIModelTypePB {
  String get displayName {
    switch (this) {
      case GlobalAIModelTypePB.GlobalLocalAI:
        return LocaleKeys.settings_aiPage_keys_ollamaLocal.tr();
      case GlobalAIModelTypePB.GlobalOpenAICompatible:
        return LocaleKeys.settings_aiPage_keys_openaiCompatible.tr();
      default:
        return 'Unknown';
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
class _GlobalModelTypeSelectionWithPanel extends StatelessWidget {
  const _GlobalModelTypeSelectionWithPanel();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<SettingsAIBloc, SettingsAIState>(
      builder: (context, state) {
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
                    child: SettingsDropdown<GlobalAIModelTypePB>(
                      selectedOption: state.globalModelType,
                      onChanged: (type) {
                        context
                            .read<SettingsAIBloc>()
                            .add(SettingsAIEvent.saveGlobalModelType(type));
                      },
                      options: GlobalAIModelTypePB.values
                          .map(
                            (type) => buildDropdownMenuEntry<GlobalAIModelTypePB>(
                              context,
                              value: type,
                              label: type.displayName,
                              selectedValue: state.globalModelType,
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
            _buildConfigurationPanel(state.globalModelType),
          ],
        );
      },
    );
  }

  /// 根据选择的全局模型类型构建相应的配置面板
  Widget _buildConfigurationPanel(GlobalAIModelTypePB modelType) {
    switch (modelType) {
      case GlobalAIModelTypePB.GlobalLocalAI:
        return const LocalAISetting();
      case GlobalAIModelTypePB.GlobalOpenAICompatible:
        return const OpenAICompatibleSetting();
      default:
        return const LocalAISetting(); // 默认显示本地 AI 设置
    }
  }
}
