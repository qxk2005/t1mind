import 'dart:async';

import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_setting.pb.dart';
import 'package:bloc/bloc.dart';

import 'package:appflowy/user/application/user_settings_service.dart' as user_settings;

enum AiProviderType {
  local,
  openaiCompatible,
}

class AiProviderState {
  AiProviderState({required this.provider, this.isLoading = false});

  final AiProviderType provider;
  final bool isLoading;

  AiProviderState copyWith({AiProviderType? provider, bool? isLoading}) {
    return AiProviderState(
      provider: provider ?? this.provider,
      isLoading: isLoading ?? this.isLoading,
    );
  }
}

class AiProviderCubit extends Cubit<AiProviderState> {
  AiProviderCubit({user_settings.UserSettingsBackendService? service})
      : _service = service ?? const user_settings.UserSettingsBackendService(),
        super(AiProviderState(provider: AiProviderType.local, isLoading: true)) {
    _load();
  }

  static const String _kvKey = 'ai.global.provider';

  final user_settings.UserSettingsBackendService _service;
  AppearanceSettingsPB? _appearance;

  Future<void> _load() async {
    try {
      final appearance = await _service.getAppearanceSetting();
      _appearance = appearance;
      final value = appearance.settingKeyValue[_kvKey];
      final provider = _parseProvider(value);
      emit(state.copyWith(provider: provider, isLoading: false));
    } catch (e) {
      Log.warn('Failed to load appearance settings for AI provider: $e');
      emit(state.copyWith(provider: AiProviderType.local, isLoading: false));
    }
  }

  Future<void> setProvider(AiProviderType provider) async {
    emit(state.copyWith(isLoading: true));
    try {
      _appearance ??= await _service.getAppearanceSetting();
      final appearance = _appearance!;
      appearance.settingKeyValue[_kvKey] = _toString(provider);
      await _service.setAppearanceSetting(appearance);
      emit(state.copyWith(provider: provider, isLoading: false));
    } catch (e) {
      Log.error('Failed to save AI provider: $e');
      emit(state.copyWith(isLoading: false));
    }
  }

  AiProviderType _parseProvider(String? value) {
    switch (value) {
      case 'openai_compat':
        return AiProviderType.openaiCompatible;
      case 'local':
      default:
        return AiProviderType.local;
    }
  }

  String _toString(AiProviderType provider) {
    switch (provider) {
      case AiProviderType.local:
        return 'local';
      case AiProviderType.openaiCompatible:
        return 'openai_compat';
    }
  }
}


