import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:bloc/bloc.dart';
import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'openai_compatible_setting_bloc.freezed.dart';

/// Default values for OpenAI compatible settings
const kDefaultChatEndpoint = 'https://api.openai.com/v1';
const kDefaultEmbeddingEndpoint = 'https://api.openai.com/v1';
const kDefaultChatModel = 'gpt-3.5-turbo';
const kDefaultEmbeddingModel = 'text-embedding-ada-002';
const kDefaultMaxTokens = 4096;
const kDefaultTemperature = 0.7;
const kDefaultTimeoutSeconds = 30;

class OpenAICompatibleSettingBloc extends Bloc<OpenAICompatibleSettingEvent, OpenAICompatibleSettingState> {
  OpenAICompatibleSettingBloc() : super(const OpenAICompatibleSettingState()) {
    on<_Started>(_handleStarted);
    on<_DidLoadSetting>(_onLoadSetting);
    on<_OnEdit>(_onEdit);
    on<_OnSubmit>(_onSubmit);
    on<_OnTestChat>(_onTestChat);
    on<_OnTestEmbedding>(_onTestEmbedding);
  }

  Future<void> _handleStarted(
    _Started event,
    Emitter<OpenAICompatibleSettingState> emit,
  ) async {
    try {
      emit(state.copyWith(loadingState: LoadingState.loading));
      
      final result = await AIEventGetOpenAICompatibleSetting().send();
      result.fold(
        (setting) {
          if (!isClosed) {
            add(OpenAICompatibleSettingEvent.didLoadSetting(setting));
          }
        },
        (error) {
          Log.error('Failed to load OpenAI compatible setting: $error');
          emit(state.copyWith(loadingState: LoadingState.error));
        },
      );
    } catch (e, st) {
      Log.error('Failed to load OpenAI compatible setting: $e\n$st');
      emit(state.copyWith(loadingState: LoadingState.error));
    }
  }

  void _onLoadSetting(
    _DidLoadSetting event,
    Emitter<OpenAICompatibleSettingState> emit,
  ) {
    final setting = event.setting;
    final submitted = setting.toSubmittedItems();
    
    emit(
      state.copyWith(
        setting: setting,
        inputItems: setting.toInputItems(),
        submittedItems: submitted,
        originalMap: {
          for (final item in submitted) item.settingType: item.content,
        },
        isEdited: false,
        loadingState: LoadingState.loaded,
      ),
    );
  }

  void _onEdit(
    _OnEdit event,
    Emitter<OpenAICompatibleSettingState> emit,
  ) {
    final updated = state.submittedItems
        .map(
          (item) => item.settingType == event.settingType
              ? item.copyWith(content: event.content)
              : item,
        )
        .toList();

    final currentMap = {for (final i in updated) i.settingType: i.content};
    final isEdited = !const MapEquality<OpenAISettingType, String>()
        .equals(state.originalMap, currentMap);

    emit(state.copyWith(submittedItems: updated, isEdited: isEdited));
  }

  void _onSubmit(
    _OnSubmit event,
    Emitter<OpenAICompatibleSettingState> emit,
  ) async {
    try {
      emit(state.copyWith(submitState: SubmitState.submitting));
      
      final chatSetting = OpenAIChatSettingPB();
      final embeddingSetting = OpenAIEmbeddingSettingPB();
      
      // Build settings from submitted items
      for (final item in state.submittedItems) {
        switch (item.settingType) {
          case OpenAISettingType.chatEndpoint:
            chatSetting.apiEndpoint = item.content;
            break;
          case OpenAISettingType.chatApiKey:
            chatSetting.apiKey = item.content;
            break;
          case OpenAISettingType.chatModel:
            chatSetting.modelName = item.content;
            break;
          case OpenAISettingType.chatModelType:
            chatSetting.modelType = item.content;
            break;
          case OpenAISettingType.chatMaxTokens:
            chatSetting.maxTokens = int.tryParse(item.content) ?? kDefaultMaxTokens;
            break;
          case OpenAISettingType.chatTemperature:
            chatSetting.temperature = double.tryParse(item.content) ?? kDefaultTemperature;
            break;
          case OpenAISettingType.chatTimeout:
            chatSetting.timeoutSeconds = int.tryParse(item.content) ?? kDefaultTimeoutSeconds;
            break;
          case OpenAISettingType.embeddingEndpoint:
            embeddingSetting.apiEndpoint = item.content;
            break;
          case OpenAISettingType.embeddingApiKey:
            embeddingSetting.apiKey = item.content;
            break;
          case OpenAISettingType.embeddingModel:
            embeddingSetting.modelName = item.content;
            break;
        }
      }
      
      final pb = OpenAICompatibleSettingPB()
        ..chatSetting = chatSetting
        ..embeddingSetting = embeddingSetting;

      final result = await AIEventSaveOpenAICompatibleSetting(pb).send();
      result.fold(
        (_) {
          Log.info('OpenAI compatible setting updated successfully');
          emit(state.copyWith(
            setting: pb,
            submitState: SubmitState.success,
            originalMap: {for (final i in state.submittedItems) i.settingType: i.content},
            isEdited: false,
          ));
        },
        (error) {
          Log.error('Update OpenAI compatible setting failed: $error');
          emit(state.copyWith(submitState: SubmitState.error));
        },
      );
    } catch (e, st) {
      Log.error('Update OpenAI compatible setting failed: $e\n$st');
      emit(state.copyWith(submitState: SubmitState.error));
    }
  }

  void _onTestChat(
    _OnTestChat event,
    Emitter<OpenAICompatibleSettingState> emit,
  ) async {
    try {
      emit(state.copyWith(chatTestState: TestState.testing));
      
      // Build chat setting from current submitted items
      final chatSetting = OpenAIChatSettingPB();
      for (final item in state.submittedItems) {
        switch (item.settingType) {
          case OpenAISettingType.chatEndpoint:
            chatSetting.apiEndpoint = item.content;
            break;
          case OpenAISettingType.chatApiKey:
            chatSetting.apiKey = item.content;
            break;
          case OpenAISettingType.chatModel:
            chatSetting.modelName = item.content;
            break;
          case OpenAISettingType.chatModelType:
            chatSetting.modelType = item.content;
            break;
          case OpenAISettingType.chatMaxTokens:
            chatSetting.maxTokens = int.tryParse(item.content) ?? kDefaultMaxTokens;
            break;
          case OpenAISettingType.chatTemperature:
            chatSetting.temperature = double.tryParse(item.content) ?? kDefaultTemperature;
            break;
          case OpenAISettingType.chatTimeout:
            chatSetting.timeoutSeconds = int.tryParse(item.content) ?? kDefaultTimeoutSeconds;
            break;
          default:
            break;
        }
      }

      final result = await AIEventTestOpenAIChat(chatSetting).send();
      result.fold(
        (testResult) {
          emit(state.copyWith(
            chatTestState: testResult.success ? TestState.success : TestState.error,
            chatTestResult: testResult,
          ));
        },
        (error) {
          Log.error('Test OpenAI chat failed: $error');
          emit(state.copyWith(
            chatTestState: TestState.error,
            chatTestResult: TestResultPB()
              ..success = false
              ..errorMessage = error.msg,
          ));
        },
      );
    } catch (e, st) {
      Log.error('Test OpenAI chat failed: $e\n$st');
      emit(state.copyWith(
        chatTestState: TestState.error,
        chatTestResult: TestResultPB()
          ..success = false
          ..errorMessage = e.toString(),
      ));
    }
  }

  void _onTestEmbedding(
    _OnTestEmbedding event,
    Emitter<OpenAICompatibleSettingState> emit,
  ) async {
    try {
      emit(state.copyWith(embeddingTestState: TestState.testing));
      
      // Build embedding setting from current submitted items
      final embeddingSetting = OpenAIEmbeddingSettingPB();
      for (final item in state.submittedItems) {
        switch (item.settingType) {
          case OpenAISettingType.embeddingEndpoint:
            embeddingSetting.apiEndpoint = item.content;
            break;
          case OpenAISettingType.embeddingApiKey:
            embeddingSetting.apiKey = item.content;
            break;
          case OpenAISettingType.embeddingModel:
            embeddingSetting.modelName = item.content;
            break;
          default:
            break;
        }
      }

      final result = await AIEventTestOpenAIEmbedding(embeddingSetting).send();
      result.fold(
        (testResult) {
          emit(state.copyWith(
            embeddingTestState: testResult.success ? TestState.success : TestState.error,
            embeddingTestResult: testResult,
          ));
        },
        (error) {
          Log.error('Test OpenAI embedding failed: $error');
          emit(state.copyWith(
            embeddingTestState: TestState.error,
            embeddingTestResult: TestResultPB()
              ..success = false
              ..errorMessage = error.msg,
          ));
        },
      );
    } catch (e, st) {
      Log.error('Test OpenAI embedding failed: $e\n$st');
      emit(state.copyWith(
        embeddingTestState: TestState.error,
        embeddingTestResult: TestResultPB()
          ..success = false
          ..errorMessage = e.toString(),
      ));
    }
  }
}

/// Setting types for OpenAI compatible configuration
enum OpenAISettingType {
  chatEndpoint,
  chatApiKey,
  chatModel,
  chatModelType,
  chatMaxTokens,
  chatTemperature,
  chatTimeout,
  embeddingEndpoint,
  embeddingApiKey,
  embeddingModel;

  String get title {
    switch (this) {
      case OpenAISettingType.chatEndpoint:
        return 'Chat API Endpoint';
      case OpenAISettingType.chatApiKey:
        return 'Chat API Key';
      case OpenAISettingType.chatModel:
        return 'Chat Model Name';
      case OpenAISettingType.chatModelType:
        return 'Chat Model Type';
      case OpenAISettingType.chatMaxTokens:
        return 'Max Tokens';
      case OpenAISettingType.chatTemperature:
        return 'Temperature';
      case OpenAISettingType.chatTimeout:
        return 'Timeout (seconds)';
      case OpenAISettingType.embeddingEndpoint:
        return 'Embedding API Endpoint';
      case OpenAISettingType.embeddingApiKey:
        return 'Embedding API Key';
      case OpenAISettingType.embeddingModel:
        return 'Embedding Model Name';
    }
  }

  bool get isPassword {
    switch (this) {
      case OpenAISettingType.chatApiKey:
      case OpenAISettingType.embeddingApiKey:
        return true;
      default:
        return false;
    }
  }
}

/// Input field representation
class OpenAISettingItem extends Equatable {
  const OpenAISettingItem({
    required this.content,
    required this.hintText,
    required this.settingType,
    this.editable = true,
  });

  final String content;
  final String hintText;
  final OpenAISettingType settingType;
  final bool editable;

  @override
  List<Object?> get props => [content, settingType, editable];
}

/// Items pending submission
class OpenAISubmittedItem extends Equatable {
  const OpenAISubmittedItem({
    required this.content,
    required this.settingType,
  });

  final String content;
  final OpenAISettingType settingType;

  /// Returns a copy of this SubmittedItem with given fields updated.
  OpenAISubmittedItem copyWith({
    String? content,
    OpenAISettingType? settingType,
  }) {
    return OpenAISubmittedItem(
      content: content ?? this.content,
      settingType: settingType ?? this.settingType,
    );
  }

  @override
  List<Object?> get props => [content, settingType];
}

/// Loading states
enum LoadingState {
  idle,
  loading,
  loaded,
  error,
}

/// Submit states
enum SubmitState {
  idle,
  submitting,
  success,
  error,
}

/// Test states
enum TestState {
  idle,
  testing,
  success,
  error,
}

@freezed
class OpenAICompatibleSettingEvent with _$OpenAICompatibleSettingEvent {
  const factory OpenAICompatibleSettingEvent.started() = _Started;
  const factory OpenAICompatibleSettingEvent.didLoadSetting(
    OpenAICompatibleSettingPB setting,
  ) = _DidLoadSetting;
  const factory OpenAICompatibleSettingEvent.onEdit(
    String content,
    OpenAISettingType settingType,
  ) = _OnEdit;
  const factory OpenAICompatibleSettingEvent.submit() = _OnSubmit;
  const factory OpenAICompatibleSettingEvent.testChat() = _OnTestChat;
  const factory OpenAICompatibleSettingEvent.testEmbedding() = _OnTestEmbedding;
}

@freezed
class OpenAICompatibleSettingState with _$OpenAICompatibleSettingState {
  const factory OpenAICompatibleSettingState({
    OpenAICompatibleSettingPB? setting,
    @Default([]) List<OpenAISettingItem> inputItems,
    @Default([]) List<OpenAISubmittedItem> submittedItems,
    @Default(false) bool isEdited,
    @Default({}) Map<OpenAISettingType, String> originalMap,
    @Default(LoadingState.idle) LoadingState loadingState,
    @Default(SubmitState.idle) SubmitState submitState,
    @Default(TestState.idle) TestState chatTestState,
    @Default(TestState.idle) TestState embeddingTestState,
    TestResultPB? chatTestResult,
    TestResultPB? embeddingTestResult,
  }) = _OpenAICompatibleSettingState;
}

extension on OpenAICompatibleSettingPB {
  List<OpenAISettingItem> toInputItems() => [
        // Chat settings
        OpenAISettingItem(
          content: chatSetting.apiEndpoint,
          hintText: kDefaultChatEndpoint,
          settingType: OpenAISettingType.chatEndpoint,
        ),
        OpenAISettingItem(
          content: chatSetting.apiKey,
          hintText: 'Enter your API key',
          settingType: OpenAISettingType.chatApiKey,
        ),
        OpenAISettingItem(
          content: chatSetting.modelName,
          hintText: kDefaultChatModel,
          settingType: OpenAISettingType.chatModel,
        ),
        OpenAISettingItem(
          content: chatSetting.modelType,
          hintText: 'chat',
          settingType: OpenAISettingType.chatModelType,
        ),
        OpenAISettingItem(
          content: chatSetting.maxTokens.toString(),
          hintText: kDefaultMaxTokens.toString(),
          settingType: OpenAISettingType.chatMaxTokens,
        ),
        OpenAISettingItem(
          content: chatSetting.temperature.toString(),
          hintText: kDefaultTemperature.toString(),
          settingType: OpenAISettingType.chatTemperature,
        ),
        OpenAISettingItem(
          content: chatSetting.timeoutSeconds.toString(),
          hintText: kDefaultTimeoutSeconds.toString(),
          settingType: OpenAISettingType.chatTimeout,
        ),
        // Embedding settings
        OpenAISettingItem(
          content: embeddingSetting.apiEndpoint,
          hintText: kDefaultEmbeddingEndpoint,
          settingType: OpenAISettingType.embeddingEndpoint,
        ),
        OpenAISettingItem(
          content: embeddingSetting.apiKey,
          hintText: 'Enter your API key',
          settingType: OpenAISettingType.embeddingApiKey,
        ),
        OpenAISettingItem(
          content: embeddingSetting.modelName,
          hintText: kDefaultEmbeddingModel,
          settingType: OpenAISettingType.embeddingModel,
        ),
      ];

  List<OpenAISubmittedItem> toSubmittedItems() => [
        // Chat settings
        OpenAISubmittedItem(
          content: chatSetting.apiEndpoint,
          settingType: OpenAISettingType.chatEndpoint,
        ),
        OpenAISubmittedItem(
          content: chatSetting.apiKey,
          settingType: OpenAISettingType.chatApiKey,
        ),
        OpenAISubmittedItem(
          content: chatSetting.modelName,
          settingType: OpenAISettingType.chatModel,
        ),
        OpenAISubmittedItem(
          content: chatSetting.modelType,
          settingType: OpenAISettingType.chatModelType,
        ),
        OpenAISubmittedItem(
          content: chatSetting.maxTokens.toString(),
          settingType: OpenAISettingType.chatMaxTokens,
        ),
        OpenAISubmittedItem(
          content: chatSetting.temperature.toString(),
          settingType: OpenAISettingType.chatTemperature,
        ),
        OpenAISubmittedItem(
          content: chatSetting.timeoutSeconds.toString(),
          settingType: OpenAISettingType.chatTimeout,
        ),
        // Embedding settings
        OpenAISubmittedItem(
          content: embeddingSetting.apiEndpoint,
          settingType: OpenAISettingType.embeddingEndpoint,
        ),
        OpenAISubmittedItem(
          content: embeddingSetting.apiKey,
          settingType: OpenAISettingType.embeddingApiKey,
        ),
        OpenAISubmittedItem(
          content: embeddingSetting.modelName,
          settingType: OpenAISettingType.embeddingModel,
        ),
      ];
}
