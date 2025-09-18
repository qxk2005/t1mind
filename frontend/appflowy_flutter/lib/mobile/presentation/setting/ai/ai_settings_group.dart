import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/mobile/presentation/bottom_sheet/show_mobile_bottom_sheet.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_trailing.dart';
import 'package:appflowy/mobile/presentation/widgets/flowy_option_tile.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_profile.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

// 临时的全局模型类型枚举，与桌面端保持一致
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

class AiSettingsGroup extends StatefulWidget {
  const AiSettingsGroup({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  State<AiSettingsGroup> createState() => _AiSettingsGroupState();
}

class _AiSettingsGroupState extends State<AiSettingsGroup> {
  // 临时状态管理，默认选择 Ollama 本地，与桌面端保持一致
  GlobalModelType _selectedGlobalModelType = GlobalModelType.ollama;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => SettingsAIBloc(
        widget.userProfile,
        widget.workspaceId,
      )..add(const SettingsAIEvent.started()),
      child: BlocBuilder<SettingsAIBloc, SettingsAIState>(
        builder: (context, state) {
          return MobileSettingGroup(
            groupTitle: LocaleKeys.settings_aiPage_title.tr(),
            settingItemList: [
              // 全局模型类型选择器
              MobileSettingItem(
                name: '全局使用的模型类型', // TODO: 使用国际化 LocaleKeys.settings_aiPage_keys_globalModelType.tr()
                trailing: MobileSettingTrailing(
                  text: _selectedGlobalModelType.displayName,
                ),
                onTap: () => _onGlobalModelTypeTap(context),
              ),
              // LLM模型类型选择（仅在选择Ollama本地时显示）
              if (_selectedGlobalModelType == GlobalModelType.ollama)
                MobileSettingItem(
                  name: LocaleKeys.settings_aiPage_keys_llmModelType.tr(),
                  trailing: MobileSettingTrailing(
                    text: state.availableModels?.selectedModel.name ?? "",
                  ),
                  onTap: () => _onLLMModelTypeTap(context, state),
                ),
              // enable AI search if needed
              // MobileSettingItem(
              //   name: LocaleKeys.settings_aiPage_keys_enableAISearchTitle.tr(),
              //   trailing: const Icon(
              //     Icons.chevron_right,
              //   ),
              //   onTap: () => context.push(AppFlowyCloudPage.routeName),
              // ),
            ],
          );
        },
      ),
    );
  }

  void _onGlobalModelTypeTap(BuildContext context) {
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: '全局使用的模型类型', // TODO: 使用国际化 LocaleKeys.settings_aiPage_keys_globalModelType.tr()
      builder: (_) {
        return Column(
          children: GlobalModelType.values
              .asMap()
              .entries
              .map(
                (entry) => FlowyOptionTile.checkbox(
                  text: entry.value.displayName,
                  showTopBorder: entry.key == 0,
                  isSelected: _selectedGlobalModelType == entry.value,
                  onTap: () {
                    setState(() {
                      _selectedGlobalModelType = entry.value;
                    });
                    // TODO: 在后续任务中连接到 BLoC
                    context.pop();
                  },
                ),
              )
              .toList(),
        );
      },
    );
  }

  void _onLLMModelTypeTap(BuildContext context, SettingsAIState state) {
    final availableModels = state.availableModels;
    showMobileBottomSheet(
      context,
      showHeader: true,
      showDragHandle: true,
      showDivider: false,
      title: LocaleKeys.settings_aiPage_keys_llmModelType.tr(),
      builder: (_) {
        return Column(
          children: (availableModels?.models ?? [])
              .asMap()
              .entries
              .map(
                (entry) => FlowyOptionTile.checkbox(
                  text: entry.value.name,
                  showTopBorder: entry.key == 0,
                  isSelected:
                      availableModels?.selectedModel.name == entry.value.name,
                  onTap: () {
                    context
                        .read<SettingsAIBloc>()
                        .add(SettingsAIEvent.selectModel(entry.value));
                    context.pop();
                  },
                ),
              )
              .toList(),
        );
      },
    );
  }
}
