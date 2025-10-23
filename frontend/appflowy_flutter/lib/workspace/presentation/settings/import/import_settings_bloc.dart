import 'package:appflowy_backend/log.dart';
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
}

@freezed
class ImportSettingsState with _$ImportSettingsState {
  const factory ImportSettingsState({
    required ImportSettingsPB settings,
  }) = _ImportSettingsState;

  factory ImportSettingsState.initial() => ImportSettingsState(
        settings: ImportSettingsPB.defaultSettings(),
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
        logLevel: LogLevelPB.info,
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

/// 日志级别枚举
/// 临时的枚举，实际应该从 protobuf 生成
enum LogLevelPB {
  error,
  warn,
  info,
  debug,
  trace,
}
