import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/global_model_type_selector.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/local_ai_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
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
class _LocalGlobalModelTypeSelectionWithPanel extends StatelessWidget {
  const _LocalGlobalModelTypeSelectionWithPanel();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<SettingsAIBloc, SettingsAIState>(
      builder: (context, state) {
        return Column(
          children: [
            // 使用新的全局模型类型选择器组件
            const GlobalModelTypeSelector(),
            
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
        return const LocalAISetting();
    }
  }
}
