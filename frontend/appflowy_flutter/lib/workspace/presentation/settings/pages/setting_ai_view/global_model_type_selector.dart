import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/shared/af_dropdown_menu_entry.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_dropdown.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 全局AI模型类型选择器组件
/// 
/// 支持在"Ollama本地"和"OpenAI兼容服务器"之间选择
/// 选择后会触发BLoC状态更新，并支持中英文国际化
class GlobalModelTypeSelector extends StatelessWidget {
  const GlobalModelTypeSelector({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<SettingsAIBloc, SettingsAIState>(
      builder: (context, state) {
        return Padding(
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
        );
      },
    );
  }
}

/// GlobalAIModelTypePB 扩展，提供显示名称
extension GlobalAIModelTypePBExtension on GlobalAIModelTypePB {
  /// 获取模型类型的本地化显示名称
  String get displayName {
    switch (this) {
      case GlobalAIModelTypePB.GlobalLocalAI:
        return LocaleKeys.settings_aiPage_keys_ollamaLocal.tr();
      case GlobalAIModelTypePB.GlobalOpenAICompatible:
        return LocaleKeys.settings_aiPage_keys_openaiCompatible.tr();
      // TODO: Add GlobalOpenAISDK support when protobuf is updated
      // case GlobalAIModelTypePB.GlobalOpenAISDK:
      //   return LocaleKeys.settings_aiPage_keys_openaiSDK.tr();
      default:
        return 'Unknown';
    }
  }
}
