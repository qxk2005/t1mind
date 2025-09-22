import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:bloc/bloc.dart';
import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'openai_sdk_bloc.freezed.dart';

// Type aliases for OpenAI SDK - using existing compatible types
typedef OpenAISDKSettingPB = OpenAICompatibleSettingPB;
typedef OpenAISDKChatSettingPB = OpenAIChatSettingPB;
typedef OpenAISDKEmbeddingSettingPB = OpenAIEmbeddingSettingPB;

/// Default values for OpenAI SDK settings
const kDefaultSDKChatEndpoint = 'https://api.openai.com/v1';
const kDefaultSDKEmbeddingEndpoint = 'https://api.openai.com/v1';
const kDefaultSDKChatModel = 'gpt-3.5-turbo';
const kDefaultSDKEmbeddingModel = 'text-embedding-ada-002';
const kDefaultSDKMaxTokens = 4096;
const kDefaultSDKTemperature = 0.7;
const kDefaultSDKTimeoutSeconds = 30;

class OpenAISDKBloc extends Bloc<OpenAISDKEvent, OpenAISDKState> {
  OpenAISDKBloc() : super(const OpenAISDKState()) {
    on<_Started>(_handleStarted);
    on<_DidLoadSetting>(_onLoadSetting);
    on<_OnEdit>(_onEdit);
    on<_OnSubmit>(_onSubmit);
    on<_OnTestChat>(_onTestChat);
    on<_OnTestEmbedding>(_onTestEmbedding);
  }

  Future<void> _handleStarted(
    _Started event,
    Emitter<OpenAISDKState> emit,
  ) async {
    try {
      emit(state.copyWith(loadingState: LoadingState.loading));
      
      final result = await AIEventGetOpenAICompatibleSetting().send();
      result.fold(
        (setting) {
          if (!isClosed) {
            add(OpenAISDKEvent.didLoadSetting(setting));
          }
        },
        (error) {
          Log.error('Failed to load OpenAI SDK setting: $error');
          emit(state.copyWith(loadingState: LoadingState.error));
        },
      );
    } catch (e, st) {
      Log.error('Failed to load OpenAI SDK setting: $e\n$st');
      emit(state.copyWith(loadingState: LoadingState.error));
    }
  }

  void _onLoadSetting(
    _DidLoadSetting event,
    Emitter<OpenAISDKState> emit,
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
    Emitter<OpenAISDKState> emit,
  ) {
    final updated = state.submittedItems
        .map(
          (item) => item.settingType == event.settingType
              ? item.copyWith(content: event.content)
              : item,
        )
        .toList();

    final currentMap = {for (final i in updated) i.settingType: i.content};
    final isEdited = !const MapEquality<OpenAISDKSettingType, String>()
        .equals(state.originalMap, currentMap);

    emit(state.copyWith(submittedItems: updated, isEdited: isEdited));
  }

  void _onSubmit(
    _OnSubmit event,
    Emitter<OpenAISDKState> emit,
  ) async {
    try {
      emit(state.copyWith(submitState: SubmitState.submitting));
      
      final chatSetting = OpenAISDKChatSettingPB();
      final embeddingSetting = OpenAISDKEmbeddingSettingPB();
      
      // Build settings from submitted items
      for (final item in state.submittedItems) {
        switch (item.settingType) {
          case OpenAISDKSettingType.chatEndpoint:
            chatSetting.apiEndpoint = item.content;
            break;
          case OpenAISDKSettingType.chatApiKey:
            chatSetting.apiKey = item.content;
            break;
          case OpenAISDKSettingType.chatModel:
            chatSetting.modelName = item.content;
            break;
          case OpenAISDKSettingType.chatModelType:
            chatSetting.modelType = item.content;
            break;
          case OpenAISDKSettingType.chatMaxTokens:
            chatSetting.maxTokens = int.tryParse(item.content) ?? kDefaultSDKMaxTokens;
            break;
          case OpenAISDKSettingType.chatTemperature:
            chatSetting.temperature = double.tryParse(item.content) ?? kDefaultSDKTemperature;
            break;
          case OpenAISDKSettingType.chatTimeout:
            chatSetting.timeoutSeconds = int.tryParse(item.content) ?? kDefaultSDKTimeoutSeconds;
            break;
          case OpenAISDKSettingType.embeddingEndpoint:
            embeddingSetting.apiEndpoint = item.content;
            break;
          case OpenAISDKSettingType.embeddingApiKey:
            embeddingSetting.apiKey = item.content;
            break;
          case OpenAISDKSettingType.embeddingModel:
            embeddingSetting.modelName = item.content;
            break;
        }
      }
      
      final pb = OpenAISDKSettingPB()
        ..chatSetting = chatSetting
        ..embeddingSetting = embeddingSetting;

      final result = await AIEventSaveOpenAICompatibleSetting(pb).send();
      result.fold(
        (_) {
          Log.info('OpenAI SDK setting updated successfully');
          emit(state.copyWith(
            setting: pb,
            submitState: SubmitState.success,
            originalMap: {for (final i in state.submittedItems) i.settingType: i.content},
            isEdited: false,
          ),);
        },
        (error) {
          Log.error('Update OpenAI SDK setting failed: $error');
          emit(state.copyWith(submitState: SubmitState.error));
        },
      );
    } catch (e, st) {
      Log.error('Update OpenAI SDK setting failed: $e\n$st');
      emit(state.copyWith(submitState: SubmitState.error));
    }
  }

  void _onTestChat(
    _OnTestChat event,
    Emitter<OpenAISDKState> emit,
  ) async {
    try {
      emit(state.copyWith(chatTestState: TestState.testing));
      
      // Build chat setting from current submitted items
      final chatSetting = OpenAISDKChatSettingPB();
      for (final item in state.submittedItems) {
        switch (item.settingType) {
          case OpenAISDKSettingType.chatEndpoint:
            chatSetting.apiEndpoint = item.content;
            break;
          case OpenAISDKSettingType.chatApiKey:
            chatSetting.apiKey = item.content;
            break;
          case OpenAISDKSettingType.chatModel:
            chatSetting.modelName = item.content;
            break;
          case OpenAISDKSettingType.chatModelType:
            chatSetting.modelType = item.content;
            break;
          case OpenAISDKSettingType.chatMaxTokens:
            chatSetting.maxTokens = int.tryParse(item.content) ?? kDefaultSDKMaxTokens;
            break;
          case OpenAISDKSettingType.chatTemperature:
            chatSetting.temperature = double.tryParse(item.content) ?? kDefaultSDKTemperature;
            break;
          case OpenAISDKSettingType.chatTimeout:
            chatSetting.timeoutSeconds = int.tryParse(item.content) ?? kDefaultSDKTimeoutSeconds;
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
          ),);
        },
        (error) {
          Log.error('Test OpenAI SDK chat failed: $error');
          emit(state.copyWith(
            chatTestState: TestState.error,
            chatTestResult: TestResultPB()
              ..success = false
              ..errorMessage = error.msg,
          ),);
        },
      );
    } catch (e, st) {
      Log.error('Test OpenAI SDK chat failed: $e\n$st');
      emit(state.copyWith(
        chatTestState: TestState.error,
        chatTestResult: TestResultPB()
          ..success = false
          ..errorMessage = e.toString(),
      ),);
    }
  }

  void _onTestEmbedding(
    _OnTestEmbedding event,
    Emitter<OpenAISDKState> emit,
  ) async {
    try {
      emit(state.copyWith(embeddingTestState: TestState.testing));
      
      // Build embedding setting from current submitted items
      final embeddingSetting = OpenAISDKEmbeddingSettingPB();
      for (final item in state.submittedItems) {
        switch (item.settingType) {
          case OpenAISDKSettingType.embeddingEndpoint:
            embeddingSetting.apiEndpoint = item.content;
            break;
          case OpenAISDKSettingType.embeddingApiKey:
            embeddingSetting.apiKey = item.content;
            break;
          case OpenAISDKSettingType.embeddingModel:
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
          ),);
        },
        (error) {
          Log.error('Test OpenAI SDK embedding failed: $error');
          emit(state.copyWith(
            embeddingTestState: TestState.error,
            embeddingTestResult: TestResultPB()
              ..success = false
              ..errorMessage = error.msg,
          ),);
        },
      );
    } catch (e, st) {
      Log.error('Test OpenAI SDK embedding failed: $e\n$st');
      emit(state.copyWith(
        embeddingTestState: TestState.error,
        embeddingTestResult: TestResultPB()
          ..success = false
          ..errorMessage = e.toString(),
      ),);
    }
  }
}

/// Setting types for OpenAI SDK configuration
enum OpenAISDKSettingType {
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
      case OpenAISDKSettingType.chatEndpoint:
        return 'Chat API Endpoint';
      case OpenAISDKSettingType.chatApiKey:
        return 'Chat API Key';
      case OpenAISDKSettingType.chatModel:
        return 'Chat Model Name';
      case OpenAISDKSettingType.chatModelType:
        return 'Chat Model Type';
      case OpenAISDKSettingType.chatMaxTokens:
        return 'Max Tokens';
      case OpenAISDKSettingType.chatTemperature:
        return 'Temperature';
      case OpenAISDKSettingType.chatTimeout:
        return 'Timeout (seconds)';
      case OpenAISDKSettingType.embeddingEndpoint:
        return 'Embedding API Endpoint';
      case OpenAISDKSettingType.embeddingApiKey:
        return 'Embedding API Key';
      case OpenAISDKSettingType.embeddingModel:
        return 'Embedding Model Name';
    }
  }

  bool get isPassword {
    switch (this) {
      case OpenAISDKSettingType.chatApiKey:
      case OpenAISDKSettingType.embeddingApiKey:
        return true;
      default:
        return false;
    }
  }
}

/// Input field representation
class OpenAISDKSettingItem extends Equatable {
  const OpenAISDKSettingItem({
    required this.content,
    required this.hintText,
    required this.settingType,
    this.editable = true,
  });

  final String content;
  final String hintText;
  final OpenAISDKSettingType settingType;
  final bool editable;

  @override
  List<Object?> get props => [content, settingType, editable];
}

/// Items pending submission
class OpenAISDKSubmittedItem extends Equatable {
  const OpenAISDKSubmittedItem({
    required this.content,
    required this.settingType,
  });

  final String content;
  final OpenAISDKSettingType settingType;

  /// Returns a copy of this SubmittedItem with given fields updated.
  OpenAISDKSubmittedItem copyWith({
    String? content,
    OpenAISDKSettingType? settingType,
  }) {
    return OpenAISDKSubmittedItem(
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
class OpenAISDKEvent with _$OpenAISDKEvent {
  const factory OpenAISDKEvent.started() = _Started;
  const factory OpenAISDKEvent.didLoadSetting(
    OpenAISDKSettingPB setting,
  ) = _DidLoadSetting;
  const factory OpenAISDKEvent.onEdit(
    String content,
    OpenAISDKSettingType settingType,
  ) = _OnEdit;
  const factory OpenAISDKEvent.submit() = _OnSubmit;
  const factory OpenAISDKEvent.testChat() = _OnTestChat;
  const factory OpenAISDKEvent.testEmbedding() = _OnTestEmbedding;
}

@freezed
class OpenAISDKState with _$OpenAISDKState {
  const factory OpenAISDKState({
    OpenAISDKSettingPB? setting,
    @Default([]) List<OpenAISDKSettingItem> inputItems,
    @Default([]) List<OpenAISDKSubmittedItem> submittedItems,
    @Default(false) bool isEdited,
    @Default({}) Map<OpenAISDKSettingType, String> originalMap,
    @Default(LoadingState.idle) LoadingState loadingState,
    @Default(SubmitState.idle) SubmitState submitState,
    @Default(TestState.idle) TestState chatTestState,
    @Default(TestState.idle) TestState embeddingTestState,
    TestResultPB? chatTestResult,
    TestResultPB? embeddingTestResult,
  }) = _OpenAISDKState;
}

extension on OpenAISDKSettingPB {
  List<OpenAISDKSettingItem> toInputItems() => [
        // Chat settings
        OpenAISDKSettingItem(
          content: chatSetting.apiEndpoint,
          hintText: kDefaultSDKChatEndpoint,
          settingType: OpenAISDKSettingType.chatEndpoint,
        ),
        OpenAISDKSettingItem(
          content: chatSetting.apiKey,
          hintText: 'Enter your API key',
          settingType: OpenAISDKSettingType.chatApiKey,
        ),
        OpenAISDKSettingItem(
          content: chatSetting.modelName,
          hintText: kDefaultSDKChatModel,
          settingType: OpenAISDKSettingType.chatModel,
        ),
        OpenAISDKSettingItem(
          content: chatSetting.modelType,
          hintText: 'chat',
          settingType: OpenAISDKSettingType.chatModelType,
        ),
        OpenAISDKSettingItem(
          content: chatSetting.maxTokens.toString(),
          hintText: kDefaultSDKMaxTokens.toString(),
          settingType: OpenAISDKSettingType.chatMaxTokens,
        ),
        OpenAISDKSettingItem(
          content: chatSetting.temperature.toString(),
          hintText: kDefaultSDKTemperature.toString(),
          settingType: OpenAISDKSettingType.chatTemperature,
        ),
        OpenAISDKSettingItem(
          content: chatSetting.timeoutSeconds.toString(),
          hintText: kDefaultSDKTimeoutSeconds.toString(),
          settingType: OpenAISDKSettingType.chatTimeout,
        ),
        // Embedding settings
        OpenAISDKSettingItem(
          content: embeddingSetting.apiEndpoint,
          hintText: kDefaultSDKEmbeddingEndpoint,
          settingType: OpenAISDKSettingType.embeddingEndpoint,
        ),
        OpenAISDKSettingItem(
          content: embeddingSetting.apiKey,
          hintText: 'Enter your API key',
          settingType: OpenAISDKSettingType.embeddingApiKey,
        ),
        OpenAISDKSettingItem(
          content: embeddingSetting.modelName,
          hintText: kDefaultSDKEmbeddingModel,
          settingType: OpenAISDKSettingType.embeddingModel,
        ),
      ];

  List<OpenAISDKSubmittedItem> toSubmittedItems() => [
        // Chat settings
        OpenAISDKSubmittedItem(
          content: chatSetting.apiEndpoint,
          settingType: OpenAISDKSettingType.chatEndpoint,
        ),
        OpenAISDKSubmittedItem(
          content: chatSetting.apiKey,
          settingType: OpenAISDKSettingType.chatApiKey,
        ),
        OpenAISDKSubmittedItem(
          content: chatSetting.modelName,
          settingType: OpenAISDKSettingType.chatModel,
        ),
        OpenAISDKSubmittedItem(
          content: chatSetting.modelType,
          settingType: OpenAISDKSettingType.chatModelType,
        ),
        OpenAISDKSubmittedItem(
          content: chatSetting.maxTokens.toString(),
          settingType: OpenAISDKSettingType.chatMaxTokens,
        ),
        OpenAISDKSubmittedItem(
          content: chatSetting.temperature.toString(),
          settingType: OpenAISDKSettingType.chatTemperature,
        ),
        OpenAISDKSubmittedItem(
          content: chatSetting.timeoutSeconds.toString(),
          settingType: OpenAISDKSettingType.chatTimeout,
        ),
        // Embedding settings
        OpenAISDKSubmittedItem(
          content: embeddingSetting.apiEndpoint,
          settingType: OpenAISDKSettingType.embeddingEndpoint,
        ),
        OpenAISDKSubmittedItem(
          content: embeddingSetting.apiKey,
          settingType: OpenAISDKSettingType.embeddingApiKey,
        ),
        OpenAISDKSubmittedItem(
          content: embeddingSetting.modelName,
          settingType: OpenAISDKSettingType.embeddingModel,
        ),
      ];
}
