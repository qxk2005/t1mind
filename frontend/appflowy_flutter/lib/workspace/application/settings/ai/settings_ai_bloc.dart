import 'package:appflowy/plugins/ai_chat/application/ai_model_switch_listener.dart';
import 'package:appflowy/user/application/user_listener.dart';
import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-error/errors.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'settings_ai_bloc.freezed.dart';

const String aiModelsGlobalActiveModel = "global_active_model";

class SettingsAIBloc extends Bloc<SettingsAIEvent, SettingsAIState> {
  SettingsAIBloc(
    this.userProfile,
    this.workspaceId,
  )   : _userListener = UserListener(userProfile: userProfile),
        _aiModelSwitchListener =
            AIModelSwitchListener(objectId: aiModelsGlobalActiveModel),
        super(
          SettingsAIState(
            userProfile: userProfile,
          ),
        ) {
    _aiModelSwitchListener.start(
      onUpdateSelectedModel: (model) {
        if (!isClosed) {
          _loadModelList();
        }
      },
    );
    _dispatch();
  }

  final UserListener _userListener;
  final UserProfilePB userProfile;
  final String workspaceId;
  final AIModelSwitchListener _aiModelSwitchListener;

  @override
  Future<void> close() async {
    await _userListener.stop();
    await _aiModelSwitchListener.stop();
    return super.close();
  }

  void _dispatch() {
    on<SettingsAIEvent>((event, emit) async {
      await event.when(
        started: () {
          _userListener.start(
            onProfileUpdated: _onProfileUpdated,
            onUserWorkspaceSettingUpdated: (settings) {
              if (!isClosed) {
                add(SettingsAIEvent.didLoadWorkspaceSetting(settings));
              }
            },
          );
          _loadModelList();
          _loadUserWorkspaceSetting();
          // 获取当前嵌入维度
          add(const SettingsAIEvent.getCurrentEmbeddingDimension());
        },
        didReceiveUserProfile: (userProfile) {
          emit(state.copyWith(userProfile: userProfile));
        },
        toggleAISearch: () {
          emit(
            state.copyWith(enableSearchIndexing: !state.enableSearchIndexing),
          );
          _updateUserWorkspaceSetting(
            disableSearchIndexing:
                !(state.aiSettings?.disableSearchIndexing ?? false),
          );
        },
        selectModel: (AIModelPB model) async {
          await AIEventUpdateSelectedModel(
            UpdateSelectedModelPB(
              source: aiModelsGlobalActiveModel,
              selectedModel: model,
            ),
          ).send();
        },
        didLoadWorkspaceSetting: (WorkspaceSettingsPB settings) {
          emit(
            state.copyWith(
              aiSettings: settings,
              enableSearchIndexing: !settings.disableSearchIndexing,
            ),
          );
        },
        didLoadAvailableModels: (ModelSelectionPB models) {
          emit(
            state.copyWith(
              availableModels: models,
            ),
          );
        },
        resetVectorDatabase: () async {
          try {
            emit(state.copyWith(isResettingVectorDB: true));
            Log.info('[AI Settings] 🔄 开始智能重置向量数据库（包含测试）...');
            
            // 调用后端智能重置向量数据库（包含测试）
            final result = await AIEventSmartResetVectorDatabaseWithTest().send();
            
            await result.fold(
              (_) async {
                Log.info('[AI Settings] ✅ 向量数据库智能重置完成');
                
                // 获取新维度
                await _getCurrentEmbeddingDimension(emit);
                
                emit(state.copyWith(
                  isResettingVectorDB: false,
                  resetCompletionMessage: '向量数据库智能重置完成！已自动检测并应用正确的维度',
                ));
              },
              (error) async {
                Log.error('[AI Settings] ❌ 智能重置向量数据库失败: $error');
                emit(state.copyWith(
                  isResettingVectorDB: false,
                  resetCompletionMessage: '智能重置失败：$error',
                ));
              },
            );
          } catch (e) {
            Log.error('[AI Settings] ❌ 重置向量数据库失败: $e');
            emit(state.copyWith(
              isResettingVectorDB: false,
              resetCompletionMessage: '重置失败：$e',
            ));
          }
        },
        resetVectorDatabaseWithDimension: (int dimension) async {
          try {
            emit(state.copyWith(isResettingVectorDB: true));
            Log.info('[AI Settings] 🔄 开始手工重置向量数据库，指定维度: ${dimension}...');
            
            // 调用后端手工重置向量数据库，使用指定维度
            final result = await AIEventManualResetVectorDatabase(
              ManualResetVectorDatabaseRequestPB(embeddingDimension: dimension),
            ).send();
            
            await result.fold(
              (_) async {
                Log.info('[AI Settings] ✅ 向量数据库手工重置完成，维度: ${dimension}');
                
                emit(state.copyWith(
                  isResettingVectorDB: false,
                  currentEmbeddingDimension: dimension,
                  resetCompletionMessage: '向量数据库重置完成！手工指定维度：${dimension}维',
                ));
              },
              (error) async {
                Log.error('[AI Settings] ❌ 手工重置向量数据库失败: $error');
                emit(state.copyWith(
                  isResettingVectorDB: false,
                  resetCompletionMessage: '手工重置失败：$error',
                ));
              },
            );
          } catch (e) {
            Log.error('[AI Settings] ❌ 手工重置向量数据库失败: $e');
            emit(state.copyWith(
              isResettingVectorDB: false,
              resetCompletionMessage: '手工重置失败：$e',
            ));
          }
        },
        getCurrentEmbeddingDimension: () async {
          await _getCurrentEmbeddingDimension(emit);
        },
        didResetVectorDatabase: (int newDimension) {
          emit(state.copyWith(
            currentEmbeddingDimension: newDimension,
            resetCompletionMessage: '向量数据库重置完成！新维度：${newDimension}维',
          ));
        },
        configureEmbeddingDimension: (int dimension) async {
          try {
            Log.info('[AI Settings] 🔧 配置嵌入模型维度: ${dimension}');
            
            // 保存配置的维度到用户设置
            await _updateUserWorkspaceSetting(
              embeddingDimension: dimension,
            );
            
            emit(state.copyWith(
              configuredEmbeddingDimension: dimension,
            ));
            
            Log.info('[AI Settings] ✅ 嵌入模型维度配置完成: ${dimension}');
          } catch (e) {
            Log.error('[AI Settings] ❌ 配置嵌入模型维度失败: $e');
          }
        },
        checkDimensionCompatibility: () async {
          try {
            Log.info('[AI Settings] 🔍 检查维度兼容性...');
            
            final result = await AIEventCheckDimensionCompatibility().send();
            
            await result.fold(
              (compatibility) async {
                if (compatibility.isCompatible) {
                  Log.info('[AI Settings] ✅ 维度兼容性检查通过');
                  emit(state.copyWith(
                    resetCompletionMessage: '✅ 维度兼容性检查通过！当前数据库维度：${compatibility.currentDbDimension}维，模型维度：${compatibility.modelDimension}维',
                  ));
                } else {
                  Log.warn('[AI Settings] ⚠️ 维度不兼容！数据库：${compatibility.currentDbDimension}维，模型：${compatibility.modelDimension}维');
                  emit(state.copyWith(
                    resetCompletionMessage: '⚠️ 维度不兼容！数据库维度：${compatibility.currentDbDimension}维，模型维度：${compatibility.modelDimension}维。建议使用智能重置功能。',
                  ));
                }
              },
              (error) async {
                Log.error('[AI Settings] ❌ 维度兼容性检查失败: $error');
                emit(state.copyWith(
                  resetCompletionMessage: '❌ 维度兼容性检查失败：$error',
                ));
              },
            );
          } catch (e) {
            Log.error('[AI Settings] ❌ 维度兼容性检查异常: $e');
            emit(state.copyWith(
              resetCompletionMessage: '❌ 维度兼容性检查异常：$e',
            ));
          }
        },
        testEmbeddingModel: (String baseUrl, String apiKey, String model) async {
          try {
            Log.info('[AI Settings] 🧪 测试嵌入模型: $model');
            
            final result = await AIEventTestEmbeddingModel(
              TestEmbeddingModelRequestPB(
                baseUrl: baseUrl,
                apiKey: apiKey,
                model: model,
              ),
            ).send();
            
            await result.fold(
              (testResult) async {
                if (testResult.success) {
                  Log.info('[AI Settings] ✅ 嵌入模型测试成功，维度: ${testResult.dimension}');
                  emit(state.copyWith(
                    resetCompletionMessage: '✅ 嵌入模型测试成功！模型：${testResult.modelName}，维度：${testResult.dimension}维',
                  ));
                } else {
                  Log.error('[AI Settings] ❌ 嵌入模型测试失败: ${testResult.error}');
                  emit(state.copyWith(
                    resetCompletionMessage: '❌ 嵌入模型测试失败：${testResult.error}',
                  ));
                }
              },
              (error) async {
                Log.error('[AI Settings] ❌ 嵌入模型测试请求失败: $error');
                emit(state.copyWith(
                  resetCompletionMessage: '❌ 嵌入模型测试请求失败：$error',
                ));
              },
            );
          } catch (e) {
            Log.error('[AI Settings] ❌ 嵌入模型测试异常: $e');
            emit(state.copyWith(
              resetCompletionMessage: '❌ 嵌入模型测试异常：$e',
            ));
          }
        },
      );
    });
  }

  Future<FlowyResult<void, FlowyError>> _updateUserWorkspaceSetting({
    bool? disableSearchIndexing,
    String? model,
    int? embeddingDimension,
  }) async {
    final payload = UpdateUserWorkspaceSettingPB(
      workspaceId: workspaceId,
    );
    if (disableSearchIndexing != null) {
      payload.disableSearchIndexing = disableSearchIndexing;
    }
    if (model != null) {
      payload.aiModel = model;
    }
    final result = await UserEventUpdateWorkspaceSetting(payload).send();
    result.fold(
      (ok) => Log.info('Update workspace setting success'),
      (err) => Log.error('Update workspace setting failed: $err'),
    );
    return result;
  }

  void _onProfileUpdated(
    FlowyResult<UserProfilePB, FlowyError> userProfileOrFailed,
  ) =>
      userProfileOrFailed.fold(
        (profile) => add(SettingsAIEvent.didReceiveUserProfile(profile)),
        (err) => Log.error(err),
      );

  void _loadModelList() {
    final payload = ModelSourcePB(source: aiModelsGlobalActiveModel);
    AIEventGetSettingModelSelection(payload).send().then((result) {
      result.fold((models) {
        if (!isClosed) {
          add(SettingsAIEvent.didLoadAvailableModels(models));
        }
      }, (err) {
        Log.error(err);
      });
    });
  }

  void _loadUserWorkspaceSetting() {
    final payload = UserWorkspaceIdPB(workspaceId: workspaceId);
    UserEventGetWorkspaceSetting(payload).send().then((result) {
      result.fold((settings) {
        if (!isClosed) {
          add(SettingsAIEvent.didLoadWorkspaceSetting(settings));
        }
      }, (err) {
        Log.error(err);
      });
    });
  }

  Future<void> _getCurrentEmbeddingDimension(Emitter<SettingsAIState> emit) async {
    try {
      Log.info('[AI Settings] 🔍 获取当前嵌入维度...');
      
      // 调用后端获取当前嵌入维度
      final result = await AIEventGetCurrentEmbeddingDimension().send();
      
      await result.fold(
        (dimensionInfo) async {
          emit(state.copyWith(currentEmbeddingDimension: dimensionInfo.dimension));
          Log.info('[AI Settings] 📏 当前嵌入维度: ${dimensionInfo.dimension} (模型: ${dimensionInfo.modelName})');
        },
        (error) async {
          Log.error('[AI Settings] ❌ 获取嵌入维度失败: $error');
          // 如果获取失败，使用默认值
          emit(state.copyWith(currentEmbeddingDimension: 1536));
        },
      );
    } catch (e) {
      Log.error('[AI Settings] ❌ 获取嵌入维度异常: $e');
      // 如果发生异常，使用默认值
      emit(state.copyWith(currentEmbeddingDimension: 1536));
    }
  }
}

@freezed
class SettingsAIEvent with _$SettingsAIEvent {
  const factory SettingsAIEvent.started() = _Started;
  const factory SettingsAIEvent.didLoadWorkspaceSetting(
    WorkspaceSettingsPB settings,
  ) = _DidLoadWorkspaceSetting;

  const factory SettingsAIEvent.toggleAISearch() = _toggleAISearch;

  const factory SettingsAIEvent.selectModel(AIModelPB model) = _SelectAIModel;

  const factory SettingsAIEvent.didReceiveUserProfile(
    UserProfilePB newUserProfile,
  ) = _DidReceiveUserProfile;

  const factory SettingsAIEvent.didLoadAvailableModels(
    ModelSelectionPB models,
  ) = _DidLoadAvailableModels;
  const factory SettingsAIEvent.resetVectorDatabase() = _ResetVectorDatabase;
  const factory SettingsAIEvent.resetVectorDatabaseWithDimension(int dimension) = _ResetVectorDatabaseWithDimension;
  const factory SettingsAIEvent.getCurrentEmbeddingDimension() = _GetCurrentEmbeddingDimension;
  const factory SettingsAIEvent.didResetVectorDatabase(int newDimension) = _DidResetVectorDatabase;
  const factory SettingsAIEvent.configureEmbeddingDimension(int dimension) = _ConfigureEmbeddingDimension;
  const factory SettingsAIEvent.checkDimensionCompatibility() = _CheckDimensionCompatibility;
  const factory SettingsAIEvent.testEmbeddingModel(String baseUrl, String apiKey, String model) = _TestEmbeddingModel;
}

@freezed
class SettingsAIState with _$SettingsAIState {
  const factory SettingsAIState({
    required UserProfilePB userProfile,
    WorkspaceSettingsPB? aiSettings,
    ModelSelectionPB? availableModels,
    @Default(true) bool enableSearchIndexing,
    @Default(false) bool isResettingVectorDB,
    @Default(null) int? currentEmbeddingDimension,
    @Default(null) String? resetCompletionMessage,
    @Default(null) int? configuredEmbeddingDimension,
  }) = _SettingsAIState;
}
