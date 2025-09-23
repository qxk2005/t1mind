import 'package:appflowy/workspace/application/settings/ai/ai_provider_cubit.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/local_ai_setting.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/openai_compat_setting.dart';
import 'package:appflowy/workspace/presentation/settings/shared/af_dropdown_menu_entry.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_dropdown.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class ProviderDropdown extends StatelessWidget {
  const ProviderDropdown({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AiProviderCubit, AiProviderState>(
      builder: (context, state) {
        final isLoading = state.isLoading;
        final provider = state.provider;
        return Padding(
          padding: const EdgeInsets.symmetric(vertical: 6),
          child: Row(
            children: [
              Expanded(
                child: FlowyText.medium(
                  'settings.aiPage.keys.providerType'.tr(),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (isLoading)
                const SizedBox(
                  height: 26,
                  width: 26,
                  child: CircularProgressIndicator.adaptive(),
                )
              else
                Flexible(
                  child: SettingsDropdown<AiProviderType>(
                    key: ValueKey(provider),
                    onChanged: (value) =>
                        context.read<AiProviderCubit>().setProvider(value),
                    selectedOption: provider,
                    selectOptionCompare: (l, r) => l == r,
                    options: [
                      buildDropdownMenuEntry<AiProviderType>(
                        context,
                        value: AiProviderType.local,
                        label: 'settings.aiPage.keys.providerType_local'.tr(),
                      ),
                      buildDropdownMenuEntry<AiProviderType>(
                        context,
                        value: AiProviderType.openaiCompatible,
                        label: 'settings.aiPage.keys.providerType_openai'.tr(),
                      ),
                    ],
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}

class ProviderTabSwitcher extends StatelessWidget {
  const ProviderTabSwitcher({super.key, this.workspaceId});

  final String? workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AiProviderCubit, AiProviderState>(
      builder: (context, state) {
        switch (state.provider) {
          case AiProviderType.local:
            return const LocalAISetting();
          case AiProviderType.openaiCompatible:
            return OpenAICompatSetting(workspaceId: workspaceId);
        }
      },
    );
  }
}


