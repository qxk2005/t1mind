import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:appflowy/workspace/presentation/settings/entities.dart';

part 'rag_setting_bloc.freezed.dart';

/// RAG设置Bloc
/// 
/// 管理RAG设置的状态，包括加载、更新和保存功能。
/// 遵循OpenAICompatSettingBloc的实现模式。
class RAGSettingBloc extends Bloc<RAGSettingEvent, RAGSettingState> {
  RAGSettingBloc() : super(RAGSettingState.initial()) {
    on<RAGSettingEvent>(_onEvent);
  }

  Future<void> _onEvent(
    RAGSettingEvent event,
    Emitter<RAGSettingState> emit,
  ) async {
    await event.map(
      load: (e) async => _handleLoad(emit),
      updateChunkSize: (e) async => _handleUpdateChunkSize(e, emit),
      updateChunkOverlap: (e) async => _handleUpdateChunkOverlap(e, emit),
      updateEnableSemanticSplitting: (e) async =>
          _handleUpdateEnableSemanticSplitting(e, emit),
      updateEnableHybridSearch: (e) async => _handleUpdateEnableHybridSearch(e, emit),
      updateVectorWeight: (e) async => _handleUpdateVectorWeight(e, emit),
      updateKeywordWeight: (e) async => _handleUpdateKeywordWeight(e, emit),
      updateInitialTopK: (e) async => _handleUpdateInitialTopK(e, emit),
      updateFinalTopK: (e) async => _handleUpdateFinalTopK(e, emit),
      updateEnableReranking: (e) async => _handleUpdateEnableReranking(e, emit),
      updateRerankerModel: (e) async => _handleUpdateRerankerModel(e, emit),
      updateRerankerApiUrl: (e) async => _handleUpdateRerankerApiUrl(e, emit),
      updateRerankerApiKey: (e) async => _handleUpdateRerankerApiKey(e, emit),
      updateEnableAgentReflection: (e) async =>
          _handleUpdateEnableAgentReflection(e, emit),
      updateReflectionThreshold: (e) async => _handleUpdateReflectionThreshold(e, emit),
      save: (e) async => _handleSave(emit),
      reset: (e) async => _handleReset(emit),
      clearError: (e) async => _handleClearError(emit),
      clearSuccess: (e) async => _handleClearSuccess(emit),
    );
  }

  /// 加载RAG设置
  Future<void> _handleLoad(Emitter<RAGSettingState> emit) async {
    try {
      emit(state.copyWith(isLoading: true, error: null));

      final result = await AIEventGetRAGSettings(GetRAGSettingsRequestPB())
          .send();
      
      result.fold(
        (response) {
          // 直接使用返回的设置，如果不存在则使用默认设置
          RAGSettingsData settings;
          try {
            settings = RAGSettingsData.fromProtobuf(response.settings);
          } catch (_) {
            settings = RAGSettingsData.defaultSettings();
          }
          
          emit(state.copyWith(
            settings: settings,
            isLoading: false,
          ));
        },
        (error) {
          Log.error('Failed to load RAG settings: $error');
          emit(state.copyWith(
            isLoading: false,
            error: '加载RAG设置失败：$error',
          ));
        },
      );
    } catch (e) {
      Log.error('Error loading RAG settings: $e');
      emit(state.copyWith(
        isLoading: false,
        error: '加载RAG设置时发生错误：$e',
      ));
    }
  }

  /// 更新块大小
  void _handleUpdateChunkSize(
    _UpdateChunkSize event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(chunkSize: event.value);
    if (newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 更新块重叠
  void _handleUpdateChunkOverlap(
    _UpdateChunkOverlap event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(chunkOverlap: event.value);
    if (newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 更新语义分割启用状态
  void _handleUpdateEnableSemanticSplitting(
    _UpdateEnableSemanticSplitting event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings =
        state.settings.copyWith(enableSemanticSplitting: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新混合检索启用状态
  void _handleUpdateEnableHybridSearch(
    _UpdateEnableHybridSearch event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings =
        state.settings.copyWith(enableHybridSearch: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新向量权重
  void _handleUpdateVectorWeight(
    _UpdateVectorWeight event,
    Emitter<RAGSettingState> emit,
  ) {
    // 确保权重在0.0-1.0之间
    final clampedValue = event.value.clamp(0.0, 1.0);
    final newSettings = state.settings.copyWith(vectorWeight: clampedValue);

    // 如果启用了混合检索，调整另一个权重以确保总和接近1.0
    if (newSettings.enableHybridSearch) {
      final keywordWeight = (1.0 - clampedValue).clamp(0.0, 1.0);
      final adjustedSettings = newSettings.copyWith(keywordWeight: keywordWeight);
      if (adjustedSettings.isValid()) {
        emit(state.copyWith(settings: adjustedSettings));
      }
    } else if (newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 更新关键词权重
  void _handleUpdateKeywordWeight(
    _UpdateKeywordWeight event,
    Emitter<RAGSettingState> emit,
  ) {
    // 确保权重在0.0-1.0之间
    final clampedValue = event.value.clamp(0.0, 1.0);
    final newSettings = state.settings.copyWith(keywordWeight: clampedValue);

    // 如果启用了混合检索，调整另一个权重以确保总和接近1.0
    if (newSettings.enableHybridSearch) {
      final vectorWeight = (1.0 - clampedValue).clamp(0.0, 1.0);
      final adjustedSettings = newSettings.copyWith(vectorWeight: vectorWeight);
      if (adjustedSettings.isValid()) {
        emit(state.copyWith(settings: adjustedSettings));
      }
    } else if (newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 更新初始TopK
  void _handleUpdateInitialTopK(
    _UpdateInitialTopK event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(initialTopK: event.value);
    // 确保finalTopK不大于initialTopK
    if (newSettings.finalTopK > event.value) {
      final adjustedSettings = newSettings.copyWith(finalTopK: event.value);
      if (adjustedSettings.isValid()) {
        emit(state.copyWith(settings: adjustedSettings));
      }
    } else if (newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 更新最终TopK
  void _handleUpdateFinalTopK(
    _UpdateFinalTopK event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(finalTopK: event.value);
    // 确保finalTopK不大于initialTopK
    if (event.value <= state.settings.initialTopK && newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 更新重排序启用状态
  void _handleUpdateEnableReranking(
    _UpdateEnableReranking event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings =
        state.settings.copyWith(enableReranking: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新重排序模型
  void _handleUpdateRerankerModel(
    _UpdateRerankerModel event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(rerankerModel: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新重排序 API URL
  void _handleUpdateRerankerApiUrl(
    _UpdateRerankerApiUrl event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(rerankerApiUrl: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新重排序 API Key
  void _handleUpdateRerankerApiKey(
    _UpdateRerankerApiKey event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings = state.settings.copyWith(rerankerApiKey: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新智能体反思启用状态
  void _handleUpdateEnableAgentReflection(
    _UpdateEnableAgentReflection event,
    Emitter<RAGSettingState> emit,
  ) {
    final newSettings =
        state.settings.copyWith(enableAgentReflection: event.value);
    emit(state.copyWith(settings: newSettings));
  }

  /// 更新反思阈值
  void _handleUpdateReflectionThreshold(
    _UpdateReflectionThreshold event,
    Emitter<RAGSettingState> emit,
  ) {
    final clampedValue = event.value.clamp(0.0, 1.0);
    final newSettings =
        state.settings.copyWith(reflectionThreshold: clampedValue);
    if (newSettings.isValid()) {
      emit(state.copyWith(settings: newSettings));
    }
  }

  /// 保存RAG设置
  Future<void> _handleSave(Emitter<RAGSettingState> emit) async {
    try {
      // 验证设置
      if (!state.settings.isValid()) {
        emit(state.copyWith(error: '设置无效，请检查配置值'));
        return;
      }

      emit(state.copyWith(isSaving: true, error: null, successMessage: null));

      final request = UpdateRAGSettingsRequestPB()
        ..chunkSize = state.settings.chunkSize
        ..chunkOverlap = state.settings.chunkOverlap
        ..enableSemanticSplitting = state.settings.enableSemanticSplitting
        ..enableHybridSearch = state.settings.enableHybridSearch
        ..vectorWeight = state.settings.vectorWeight
        ..keywordWeight = state.settings.keywordWeight
        ..initialTopK = state.settings.initialTopK
        ..finalTopK = state.settings.finalTopK
        ..enableReranking = state.settings.enableReranking
        ..enableAgentReflection = state.settings.enableAgentReflection
        ..reflectionThreshold = state.settings.reflectionThreshold
        ..metadata.addAll(state.settings.metadata);

      if (state.settings.rerankerModel != null) {
        request.rerankerModel = state.settings.rerankerModel!;
      }
      
      final result = await AIEventUpdateRAGSettings(request).send();
      
      result.fold(
        (response) {
          if (response.success) {
            // 更新设置或保持当前设置
            RAGSettingsData updatedSettings;
            try {
              updatedSettings = RAGSettingsData.fromProtobuf(response.settings);
            } catch (_) {
              updatedSettings = state.settings;
            }
            
            emit(state.copyWith(
              isSaving: false,
              successMessage: 'RAG设置已保存',
              settings: updatedSettings,
            ));
          } else {
            emit(state.copyWith(
              isSaving: false,
              error: '保存失败',
            ));
          }
        },
        (error) {
          Log.error('Failed to save RAG settings: $error');
          emit(state.copyWith(
            isSaving: false,
            error: '保存RAG设置失败：$error',
          ));
        },
      );
    } catch (e) {
      Log.error('Error saving RAG settings: $e');
      emit(state.copyWith(
        isSaving: false,
        error: '保存RAG设置时发生错误：$e',
      ));
    }
  }

  /// 重置为默认设置
  Future<void> _handleReset(Emitter<RAGSettingState> emit) async {
    try {
      emit(state.copyWith(
        settings: RAGSettingsData.defaultSettings(),
        error: null,
        successMessage: null,
      ));
    } catch (e) {
      Log.error('Error resetting RAG settings: $e');
    }
  }

  /// 清除错误消息
  void _handleClearError(Emitter<RAGSettingState> emit) {
    emit(state.copyWith(error: null));
  }

  /// 清除成功消息
  void _handleClearSuccess(Emitter<RAGSettingState> emit) {
    emit(state.copyWith(successMessage: null));
  }
}

/// RAG设置事件
/// 
/// 定义所有可能的RAG设置事件。
@freezed
class RAGSettingEvent with _$RAGSettingEvent {
  const factory RAGSettingEvent.load() = _LoadRAGSettings;
  const factory RAGSettingEvent.updateChunkSize(int value) = _UpdateChunkSize;
  const factory RAGSettingEvent.updateChunkOverlap(int value) =
      _UpdateChunkOverlap;
  const factory RAGSettingEvent.updateEnableSemanticSplitting(bool value) =
      _UpdateEnableSemanticSplitting;
  const factory RAGSettingEvent.updateEnableHybridSearch(bool value) =
      _UpdateEnableHybridSearch;
  const factory RAGSettingEvent.updateVectorWeight(double value) =
      _UpdateVectorWeight;
  const factory RAGSettingEvent.updateKeywordWeight(double value) =
      _UpdateKeywordWeight;
  const factory RAGSettingEvent.updateInitialTopK(int value) =
      _UpdateInitialTopK;
  const factory RAGSettingEvent.updateFinalTopK(int value) = _UpdateFinalTopK;
  const factory RAGSettingEvent.updateEnableReranking(bool value) =
      _UpdateEnableReranking;
  const factory RAGSettingEvent.updateRerankerModel(String? value) =
      _UpdateRerankerModel;
  const factory RAGSettingEvent.updateRerankerApiUrl(String? value) =
      _UpdateRerankerApiUrl;
  const factory RAGSettingEvent.updateRerankerApiKey(String? value) =
      _UpdateRerankerApiKey;
  const factory RAGSettingEvent.updateEnableAgentReflection(bool value) =
      _UpdateEnableAgentReflection;
  const factory RAGSettingEvent.updateReflectionThreshold(double value) =
      _UpdateReflectionThreshold;
  const factory RAGSettingEvent.save() = _SaveRAGSettings;
  const factory RAGSettingEvent.reset() = _ResetRAGSettings;
  const factory RAGSettingEvent.clearError() = _ClearError;
  const factory RAGSettingEvent.clearSuccess() = _ClearSuccess;
}

/// RAG设置状态
/// 
/// 包含当前RAG设置、加载状态、保存状态、错误和成功消息。
@freezed
class RAGSettingState with _$RAGSettingState {
  const factory RAGSettingState({
    required RAGSettingsData settings,
    @Default(false) bool isLoading,
    @Default(false) bool isSaving,
    String? error,
    String? successMessage,
  }) = _RAGSettingState;

  /// 初始状态
  factory RAGSettingState.initial() {
    return RAGSettingState(
      settings: RAGSettingsData.defaultSettings(),
      isLoading: true,
    );
  }
}
