import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-user/import_settings.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_profile.pb.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'import_settings_bloc.freezed.dart';

/// 导入设置 Bloc
/// 管理文档导入功能的配置选项
class ImportSettingsBloc extends Bloc<ImportSettingsEvent, ImportSettingsState> {
  final UserProfilePB _userProfile;
  final String _workspaceId;

  ImportSettingsBloc({
    required UserProfilePB userProfile,
    required String workspaceId,
  }) : _userProfile = userProfile,
       _workspaceId = workspaceId,
       super(ImportSettingsState.initial()) {
    on<ImportSettingsEvent>(
      (event, emit) async {
        await event.when(
          initial: () async {
            await _loadSettings(emit);
          },
          updatePreserveFormatting: (value) async {
            await _updatePreserveFormatting(emit, value);
          },
          updateExtractImages: (value) async {
            await _updateExtractImages(emit, value);
          },
          updateExtractTables: (value) async {
            await _updateExtractTables(emit, value);
          },
          updateAutoCreateFolder: (value) async {
            await _updateAutoCreateFolder(emit, value);
          },
          updateMaxConcurrentConversions: (value) async {
            await _updateMaxConcurrentConversions(emit, value);
          },
          updateConversionTimeoutSeconds: (value) async {
            await _updateConversionTimeoutSeconds(emit, value);
          },
          updateAutoRetryOnFailure: (value) async {
            await _updateAutoRetryOnFailure(emit, value);
          },
          updateMaxRetryAttempts: (value) async {
            await _updateMaxRetryAttempts(emit, value);
          },
          updateEnableProgressNotifications: (value) async {
            await _updateEnableProgressNotifications(emit, value);
          },
          updateLogLevel: (value) async {
            await _updateLogLevel(emit, value);
          },
          updateSaveConversionLogs: (value) async {
            await _updateSaveConversionLogs(emit, value);
          },
          updateLogRetentionDays: (value) async {
            await _updateLogRetentionDays(emit, value);
          },
          updateDefaultImportPath: (value) async {
            await _updateDefaultImportPath(emit, value);
          },
          updateEnableCloudSync: (value) async {
            await _updateEnableCloudSync(emit, value);
          },
          updateEnableCloudBackup: (value) async {
            await _updateEnableCloudBackup(emit, value);
          },
          updateCloudStoragePath: (value) async {
            await _updateCloudStoragePath(emit, value);
          },
          updateEnableCompression: (value) async {
            await _updateEnableCompression(emit, value);
          },
          updateEnableDebugMode: (value) async {
            await _updateEnableDebugMode(emit, value);
          },
          updateAutoCleanupTempFiles: (value) async {
            await _updateAutoCleanupTempFiles(emit, value);
          },
          updateTempFileRetentionHours: (value) async {
            await _updateTempFileRetentionHours(emit, value);
          },
          checkImportTools: () async {
            await _checkImportTools(emit);
          },
          downloadMarkerModels: (force) async {
            await _downloadMarkerModels(emit, force);
          },
          installMissingTools: (toolNames) async {
            await _installMissingTools(emit, toolNames);
          },
          updateInstallProgress: (progress) async {
            await _updateInstallProgress(emit, progress);
          },
          installFailed: (error) async {
            await _installFailed(emit, error);
          },
        );
      },
    );
  }

  Future<void> _loadSettings(Emitter<ImportSettingsState> emit) async {
    try {
      // TODO: 从后端加载设置
      // 目前使用默认设置
      final settings = ImportSettingsPB.defaultSettings();
      emit(state.copyWith(settings: settings));
    } catch (e) {
      Log.error('Failed to load import settings: $e');
      // 使用默认设置
      final settings = ImportSettingsPB.defaultSettings();
      emit(state.copyWith(settings: settings));
    }
  }

  Future<void> _updatePreserveFormatting(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(preserveFormatting: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update preserve formatting: $e');
    }
  }

  Future<void> _updateExtractImages(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(extractImages: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update extract images: $e');
    }
  }

  Future<void> _updateExtractTables(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(extractTables: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update extract tables: $e');
    }
  }

  Future<void> _updateAutoCreateFolder(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(autoCreateFolder: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update auto create folder: $e');
    }
  }

  Future<void> _updateMaxConcurrentConversions(
    Emitter<ImportSettingsState> emit,
    int value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(maxConcurrentConversions: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update max concurrent conversions: $e');
    }
  }

  Future<void> _updateConversionTimeoutSeconds(
    Emitter<ImportSettingsState> emit,
    int value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(conversionTimeoutSeconds: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update conversion timeout seconds: $e');
    }
  }

  Future<void> _updateAutoRetryOnFailure(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(autoRetryOnFailure: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update auto retry on failure: $e');
    }
  }

  Future<void> _updateMaxRetryAttempts(
    Emitter<ImportSettingsState> emit,
    int value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(maxRetryAttempts: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update max retry attempts: $e');
    }
  }

  Future<void> _updateEnableProgressNotifications(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(enableProgressNotifications: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update enable progress notifications: $e');
    }
  }

  Future<void> _updateLogLevel(
    Emitter<ImportSettingsState> emit,
    LogLevelPB value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(logLevel: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update log level: $e');
    }
  }

  Future<void> _updateSaveConversionLogs(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(saveConversionLogs: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update save conversion logs: $e');
    }
  }

  Future<void> _updateLogRetentionDays(
    Emitter<ImportSettingsState> emit,
    int value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(logRetentionDays: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update log retention days: $e');
    }
  }

  Future<void> _updateDefaultImportPath(
    Emitter<ImportSettingsState> emit,
    String value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(defaultImportPath: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update default import path: $e');
    }
  }

  Future<void> _updateEnableCloudSync(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(enableCloudSync: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update enable cloud sync: $e');
    }
  }

  Future<void> _updateEnableCloudBackup(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(enableCloudBackup: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update enable cloud backup: $e');
    }
  }

  Future<void> _updateCloudStoragePath(
    Emitter<ImportSettingsState> emit,
    String value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(cloudStoragePath: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update cloud storage path: $e');
    }
  }

  Future<void> _updateEnableCompression(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(enableCompression: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update enable compression: $e');
    }
  }

  Future<void> _updateEnableDebugMode(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(enableDebugMode: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update enable debug mode: $e');
    }
  }

  Future<void> _updateAutoCleanupTempFiles(
    Emitter<ImportSettingsState> emit,
    bool value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(autoCleanupTempFiles: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update auto cleanup temp files: $e');
    }
  }

  Future<void> _updateTempFileRetentionHours(
    Emitter<ImportSettingsState> emit,
    int value,
  ) async {
    try {
      final newSettings = state.settings.copyWith(tempFileRetentionHours: value);
      emit(state.copyWith(settings: newSettings));
      // TODO: 保存到后端
    } catch (e) {
      Log.error('Failed to update temp file retention hours: $e');
    }
  }

  /// 检查导入工具状态
  Future<void> _checkImportTools(Emitter<ImportSettingsState> emit) async {
    try {
      emit(state.copyWith(isCheckingTools: true));
      
      final result = await UserEventCheckImportToolsStatus().send();
      
      result.fold(
        (toolsStatus) {
          emit(state.copyWith(
            toolsStatus: toolsStatus,
            isCheckingTools: false,
          ));
        },
        (error) {
          Log.error('Failed to check import tools: $error');
          emit(state.copyWith(isCheckingTools: false));
        },
      );
    } catch (e) {
      Log.error('Failed to check import tools: $e');
      emit(state.copyWith(isCheckingTools: false));
    }
  }

  /// 下载 Marker 模型
  Future<void> _downloadMarkerModels(
    Emitter<ImportSettingsState> emit,
    bool force,
  ) async {
    try {
      emit(state.copyWith(isDownloadingModels: true, modelDownloadProgress: null));
      
      // 启动定期刷新任务，在下载过程中定期检查工具状态以获取最新进度
      final refreshTimer = Stream.periodic(const Duration(seconds: 3), (i) => i)
          .takeWhile((_) => !isClosed && !emit.isDone)
          .listen((_) async {
        if (!isClosed && !emit.isDone) {
          Log.info('[模型下载] 定期刷新工具状态以获取最新进度');
          await _checkImportTools(emit);
        }
      });
      
      final request = DownloadMarkerModelsPB()..force = force;
      final result = await UserEventDownloadMarkerModels(request).send();
      
      await result.fold(
        (progress) async {
          // 停止定期刷新任务
          await refreshTimer.cancel();
          
          // 先更新进度状态
          if (!emit.isDone) {
            emit(state.copyWith(
              modelDownloadProgress: progress,
              isDownloadingModels: progress.status != ModelDownloadStatusPB.ModelDownloadCompleted &&
                                   progress.status != ModelDownloadStatusPB.ModelDownloadFailed &&
                                   progress.status != ModelDownloadStatusPB.ModelDownloadCancelled,
            ));
          }
          
          // 如果下载完成或失败，重新检查工具状态
          if (progress.status == ModelDownloadStatusPB.ModelDownloadCompleted ||
              progress.status == ModelDownloadStatusPB.ModelDownloadFailed) {
            // 检查 emit 是否仍然有效，避免在事件处理器完成后调用
            if (!emit.isDone) {
              await _checkImportTools(emit);
            }
          }
        },
        (error) async {
          // 停止定期刷新任务
          await refreshTimer.cancel();
          
          Log.error('Failed to download marker models: $error');
          if (!emit.isDone) {
            emit(state.copyWith(
              isDownloadingModels: false,
              modelDownloadProgress: ModelDownloadProgressPB()
                ..status = ModelDownloadStatusPB.ModelDownloadFailed
                ..message = '下载失败: ${error.msg}',
            ));
          }
        },
      );
    } catch (e) {
      Log.error('Failed to download marker models: $e');
      emit(state.copyWith(isDownloadingModels: false));
    }
  }

  /// 安装缺失的工具（带流式反馈）
  Future<void> _installMissingTools(
    Emitter<ImportSettingsState> emit,
    List<String> toolNames,
  ) async {
    try {
      emit(state.copyWith(isInstallingTools: true, installProgress: null));
      
      // 先发送初始状态
      final initialProgress = InstallToolProgressPB()
        ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
        ..status = InstallToolStatusPB.InstallToolInstalling
        ..progress = 0.0
        ..message = '准备安装...'
        ..logs.add('开始安装工具: ${toolNames.join(", ")}');
      emit(state.copyWith(installProgress: initialProgress));
      
      // 在后台异步执行安装，同时提供流式反馈
      _installWithStreamingFeedback(emit, toolNames);
    } catch (e) {
      Log.error('Failed to install missing tools: $e');
      emit(state.copyWith(isInstallingTools: false));
    }
  }
  
  /// 带流式反馈的安装过程（使用真实的后端进度）
  Future<void> _installWithStreamingFeedback(
    Emitter<ImportSettingsState> emit,
    List<String> toolNames,
  ) async {
    Log.info('开始安装流程，工具: ${toolNames.join(", ")}');
    
    // 显示初始状态
    final initialProgress = InstallToolProgressPB()
      ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
      ..status = InstallToolStatusPB.InstallToolInstalling
      ..progress = 0.0
      ..message = '准备安装...'
      ..logs.add('开始安装工具: ${toolNames.join(", ")}');
    
    if (!isClosed && !emit.isDone) {
      emit(state.copyWith(installProgress: initialProgress));
    }
    
    // 执行安装（直接调用后端 API，不使用模拟步骤）
    final request = InstallMissingToolsPB()..toolNames.addAll(toolNames);
    
    Log.info('启动后端 API 调用');
    final result = await UserEventInstallMissingTools(request).send();
    
    // 处理后端返回的真实结果
    result.fold(
      (backendProgress) {
        Log.info('后端 API 返回成功: ${backendProgress.message}');
        Log.info('后端返回状态: ${backendProgress.status}');
        Log.info('后端返回日志数量: ${backendProgress.logs.length}');
        Log.info('后端返回工具名: ${backendProgress.toolName}');
        Log.info('后端返回进度: ${backendProgress.progress}');
        
        // 记录 emit 状态，用于诊断
        Log.info('emit.isDone: ${emit.isDone}, isClosed: $isClosed');
        
        // 使用 add 事件来更新状态，而不是直接使用 emit
        // 这样可以确保状态更新在正确的事件处理器中执行，避免 emit.isDone 的问题
        final isFailed = backendProgress.status == InstallToolStatusPB.InstallToolFailed;
        final isCompleted = backendProgress.status == InstallToolStatusPB.InstallToolCompleted;
        
        Log.info('安装状态 - 失败: $isFailed, 完成: $isCompleted');
        Log.info('准备通过 add 事件更新状态: isInstallingTools=${!isCompleted && !isFailed}');
        
        // 使用 add 事件来更新状态
        if (!isClosed) {
          add(ImportSettingsEvent.updateInstallProgress(backendProgress));
          Log.info('已发送 updateInstallProgress 事件，等待事件处理器响应');
          
          // 如果安装完成或失败，重新检查工具状态
          if (isCompleted || isFailed) {
            // 使用 Future.microtask 确保状态更新后再检查工具状态
            Future.microtask(() {
              if (!isClosed) {
                Log.info('安装完成或失败，重新检查工具状态');
                add(const ImportSettingsEvent.checkImportTools());
              }
            });
          }
        } else {
          Log.warn('Bloc 已关闭，无法更新状态');
        }
      },
      (error) {
        Log.error('后端 API 调用失败: $error');
        Log.info('错误处理 - emit.isDone: ${emit.isDone}, isClosed: $isClosed');
        
        // 使用 add 事件来更新状态，而不是直接使用 emit
        if (!isClosed) {
          final errorLogs = <String>[];
          errorLogs.add('开始安装工具: ${toolNames.join(", ")}');
          errorLogs.add('✗ 安装失败: ${error.msg}');
          
          final errorProgress = InstallToolProgressPB()
            ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
            ..status = InstallToolStatusPB.InstallToolFailed
            ..progress = 0.0
            ..message = '安装失败: ${error.msg}'
            ..logs.addAll(errorLogs);
          
          add(ImportSettingsEvent.updateInstallProgress(errorProgress));
          Log.info('已发送 updateInstallProgress 事件（错误），等待事件处理器响应');
        } else {
          Log.warn('Bloc 已关闭，无法更新错误状态');
        }
      },
    );
  }
  
  /// 更新安装进度
  Future<void> _updateInstallProgress(
    Emitter<ImportSettingsState> emit,
    InstallToolProgressPB progress,
  ) async {
    Log.info('[updateInstallProgress] 开始更新状态');
    Log.info('[updateInstallProgress] emit.isDone: ${emit.isDone}, isClosed: $isClosed');
    Log.info('[updateInstallProgress] 进度状态: ${progress.status}, 消息: ${progress.message}, 日志数量: ${progress.logs.length}');
    
    if (!isClosed && !emit.isDone) {
      final isCompleted = progress.status == InstallToolStatusPB.InstallToolCompleted;
      final isFailed = progress.status == InstallToolStatusPB.InstallToolFailed;
      final isInstalling = !isCompleted && !isFailed;
      
      Log.info('[updateInstallProgress] 安装状态 - 完成: $isCompleted, 失败: $isFailed, 进行中: $isInstalling');
      
      emit(state.copyWith(
        installProgress: progress,
        isInstallingTools: isInstalling,
      ));
      
      Log.info('[updateInstallProgress] 状态已更新');
    } else {
      Log.warn('[updateInstallProgress] 无法更新状态 - emit.isDone: ${emit.isDone}, isClosed: $isClosed');
    }
  }
  
  /// 安装失败处理
  Future<void> _installFailed(
    Emitter<ImportSettingsState> emit,
    String error,
  ) async {
    if (!emit.isDone) {
      emit(state.copyWith(
        isInstallingTools: false,
        installProgress: InstallToolProgressPB()
          ..status = InstallToolStatusPB.InstallToolFailed
          ..message = '安装失败: $error',
      ));
    }
  }
}

@freezed
class ImportSettingsEvent with _$ImportSettingsEvent {
  const factory ImportSettingsEvent.initial() = _Initial;
  const factory ImportSettingsEvent.updatePreserveFormatting(bool value) = _UpdatePreserveFormatting;
  const factory ImportSettingsEvent.updateExtractImages(bool value) = _UpdateExtractImages;
  const factory ImportSettingsEvent.updateExtractTables(bool value) = _UpdateExtractTables;
  const factory ImportSettingsEvent.updateAutoCreateFolder(bool value) = _UpdateAutoCreateFolder;
  const factory ImportSettingsEvent.updateMaxConcurrentConversions(int value) = _UpdateMaxConcurrentConversions;
  const factory ImportSettingsEvent.updateConversionTimeoutSeconds(int value) = _UpdateConversionTimeoutSeconds;
  const factory ImportSettingsEvent.updateAutoRetryOnFailure(bool value) = _UpdateAutoRetryOnFailure;
  const factory ImportSettingsEvent.updateMaxRetryAttempts(int value) = _UpdateMaxRetryAttempts;
  const factory ImportSettingsEvent.updateEnableProgressNotifications(bool value) = _UpdateEnableProgressNotifications;
  const factory ImportSettingsEvent.updateLogLevel(LogLevelPB value) = _UpdateLogLevel;
  const factory ImportSettingsEvent.updateSaveConversionLogs(bool value) = _UpdateSaveConversionLogs;
  const factory ImportSettingsEvent.updateLogRetentionDays(int value) = _UpdateLogRetentionDays;
  const factory ImportSettingsEvent.updateDefaultImportPath(String value) = _UpdateDefaultImportPath;
  const factory ImportSettingsEvent.updateEnableCloudSync(bool value) = _UpdateEnableCloudSync;
  const factory ImportSettingsEvent.updateEnableCloudBackup(bool value) = _UpdateEnableCloudBackup;
  const factory ImportSettingsEvent.updateCloudStoragePath(String value) = _UpdateCloudStoragePath;
  const factory ImportSettingsEvent.updateEnableCompression(bool value) = _UpdateEnableCompression;
  const factory ImportSettingsEvent.updateEnableDebugMode(bool value) = _UpdateEnableDebugMode;
  const factory ImportSettingsEvent.updateAutoCleanupTempFiles(bool value) = _UpdateAutoCleanupTempFiles;
  const factory ImportSettingsEvent.updateTempFileRetentionHours(int value) = _UpdateTempFileRetentionHours;
  const factory ImportSettingsEvent.checkImportTools() = _CheckImportTools;
  const factory ImportSettingsEvent.downloadMarkerModels(bool force) = _DownloadMarkerModels;
  const factory ImportSettingsEvent.installMissingTools(List<String> toolNames) = _InstallMissingTools;
  const factory ImportSettingsEvent.updateInstallProgress(InstallToolProgressPB progress) = _UpdateInstallProgress;
  const factory ImportSettingsEvent.installFailed(String error) = _InstallFailed;
}

@freezed
class ImportSettingsState with _$ImportSettingsState {
  const factory ImportSettingsState({
    required ImportSettingsPB settings,
    ImportToolsStatusPB? toolsStatus,
    @Default(false) bool isCheckingTools,
    @Default(false) bool isDownloadingModels,
    ModelDownloadProgressPB? modelDownloadProgress,
    @Default(false) bool isInstallingTools,
    InstallToolProgressPB? installProgress,
  }) = _ImportSettingsState;

  factory ImportSettingsState.initial() => ImportSettingsState(
        settings: ImportSettingsPB.defaultSettings(),
        toolsStatus: null,
        isCheckingTools: false,
        isDownloadingModels: false,
        modelDownloadProgress: null,
        isInstallingTools: false,
        installProgress: null,
      );
}

/// 导入设置数据类
/// 临时的数据类，实际应该从 protobuf 生成
class ImportSettingsPB {
  const ImportSettingsPB({
    required this.maxConcurrentConversions,
    required this.defaultImportPath,
    required this.preserveFormatting,
    required this.extractImages,
    required this.extractTables,
    required this.logLevel,
    required this.autoCreateFolder,
    required this.enableProgressNotifications,
    required this.conversionTimeoutSeconds,
    required this.autoRetryOnFailure,
    required this.maxRetryAttempts,
    required this.saveConversionLogs,
    required this.logRetentionDays,
    required this.enableCloudSync,
    required this.enableCloudBackup,
    required this.cloudStoragePath,
    required this.enableCompression,
    required this.enableDebugMode,
    required this.autoCleanupTempFiles,
    required this.tempFileRetentionHours,
  });

  final int maxConcurrentConversions;
  final String defaultImportPath;
  final bool preserveFormatting;
  final bool extractImages;
  final bool extractTables;
  final LogLevelPB logLevel;
  final bool autoCreateFolder;
  final bool enableProgressNotifications;
  final int conversionTimeoutSeconds;
  final bool autoRetryOnFailure;
  final int maxRetryAttempts;
  final bool saveConversionLogs;
  final int logRetentionDays;
  final bool enableCloudSync;
  final bool enableCloudBackup;
  final String cloudStoragePath;
  final bool enableCompression;
  final bool enableDebugMode;
  final bool autoCleanupTempFiles;
  final int tempFileRetentionHours;

  factory ImportSettingsPB.defaultSettings() => const ImportSettingsPB(
        maxConcurrentConversions: 3,
        defaultImportPath: "",
        preserveFormatting: true,
        extractImages: true,
        extractTables: true,
        logLevel: LogLevelPB.Info,
        autoCreateFolder: true,
        enableProgressNotifications: true,
        conversionTimeoutSeconds: 300,
        autoRetryOnFailure: true,
        maxRetryAttempts: 3,
        saveConversionLogs: true,
        logRetentionDays: 30,
        enableCloudSync: false,
        enableCloudBackup: false,
        cloudStoragePath: "",
        enableCompression: true,
        enableDebugMode: false,
        autoCleanupTempFiles: true,
        tempFileRetentionHours: 24,
      );

  ImportSettingsPB copyWith({
    int? maxConcurrentConversions,
    String? defaultImportPath,
    bool? preserveFormatting,
    bool? extractImages,
    bool? extractTables,
    LogLevelPB? logLevel,
    bool? autoCreateFolder,
    bool? enableProgressNotifications,
    int? conversionTimeoutSeconds,
    bool? autoRetryOnFailure,
    int? maxRetryAttempts,
    bool? saveConversionLogs,
    int? logRetentionDays,
    bool? enableCloudSync,
    bool? enableCloudBackup,
    String? cloudStoragePath,
    bool? enableCompression,
    bool? enableDebugMode,
    bool? autoCleanupTempFiles,
    int? tempFileRetentionHours,
  }) {
    return ImportSettingsPB(
      maxConcurrentConversions: maxConcurrentConversions ?? this.maxConcurrentConversions,
      defaultImportPath: defaultImportPath ?? this.defaultImportPath,
      preserveFormatting: preserveFormatting ?? this.preserveFormatting,
      extractImages: extractImages ?? this.extractImages,
      extractTables: extractTables ?? this.extractTables,
      logLevel: logLevel ?? this.logLevel,
      autoCreateFolder: autoCreateFolder ?? this.autoCreateFolder,
      enableProgressNotifications: enableProgressNotifications ?? this.enableProgressNotifications,
      conversionTimeoutSeconds: conversionTimeoutSeconds ?? this.conversionTimeoutSeconds,
      autoRetryOnFailure: autoRetryOnFailure ?? this.autoRetryOnFailure,
      maxRetryAttempts: maxRetryAttempts ?? this.maxRetryAttempts,
      saveConversionLogs: saveConversionLogs ?? this.saveConversionLogs,
      logRetentionDays: logRetentionDays ?? this.logRetentionDays,
      enableCloudSync: enableCloudSync ?? this.enableCloudSync,
      enableCloudBackup: enableCloudBackup ?? this.enableCloudBackup,
      cloudStoragePath: cloudStoragePath ?? this.cloudStoragePath,
      enableCompression: enableCompression ?? this.enableCompression,
      enableDebugMode: enableDebugMode ?? this.enableDebugMode,
      autoCleanupTempFiles: autoCleanupTempFiles ?? this.autoCleanupTempFiles,
      tempFileRetentionHours: tempFileRetentionHours ?? this.tempFileRetentionHours,
    );
  }
}

