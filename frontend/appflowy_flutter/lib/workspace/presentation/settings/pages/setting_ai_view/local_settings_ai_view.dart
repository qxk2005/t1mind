import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/local_ai_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart';
import 'package:appflowy/workspace/presentation/settings/shared/af_dropdown_menu_entry.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_dropdown.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
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
          const _LocalGlobalModelTypeSelectionWithPanel(),
        ],
      ),
    );
  }
}

/// 全局模型类型选择器和相应的配置面板（本地版本）
class _LocalGlobalModelTypeSelectionWithPanel extends StatefulWidget {
  const _LocalGlobalModelTypeSelectionWithPanel();

  @override
  State<_LocalGlobalModelTypeSelectionWithPanel> createState() => _LocalGlobalModelTypeSelectionWithPanelState();
}

class _LocalGlobalModelTypeSelectionWithPanelState extends State<_LocalGlobalModelTypeSelectionWithPanel> {
  // 临时状态管理，默认选择 Ollama 本地
  GlobalAIModelTypePB selectedType = GlobalAIModelTypePB.GlobalLocalAI;

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
                  '全局使用的模型类型', // TODO: 使用国际化 LocaleKeys.settings_aiPage_keys_globalModelType.tr()
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              Flexible(
                child: SettingsDropdown<GlobalAIModelTypePB>(
                  selectedOption: selectedType,
                  onChanged: (type) {
                    setState(() {
                      selectedType = type;
                    });
                    // TODO: 在后续任务中连接到 BLoC
                  },
                  options: GlobalAIModelTypePB.values
                      .map(
                        (type) => buildDropdownMenuEntry<GlobalAIModelTypePB>(
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
      case GlobalAIModelTypePB.GlobalLocalAI:
        return const LocalAISetting();
      case GlobalAIModelTypePB.GlobalOpenAICompatible:
        return const OpenAICompatibleSetting();
      default:
        return const LocalAISetting();
    }
  }
}
