import 'dart:convert';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:appflowy/user/application/user_listener.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-error/errors.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

part 'agent_config_bloc.freezed.dart';

/// 智能体配置存储键前缀
const String agentConfigStoragePrefix = "agent_config_";

/// 智能体配置列表存储键
const String agentConfigListKey = "agent_config_list";

/// 默认智能体配置存储键
const String defaultAgentConfigKey = "default_agent_config";

/// 智能体配置管理BLoC
/// 
/// 提供智能体配置的完整管理功能，包括：
/// - CRUD操作（创建、读取、更新、删除）
/// - 配置验证
/// - 导入导出
/// - 多智能体管理
/// - 配置持久化
class AgentConfigBloc extends Bloc<AgentConfigEvent, AgentConfigState> {
  AgentConfigBloc(
    this.userProfile,
    this.workspaceId,
  )   : _userListener = UserListener(userProfile: userProfile),
        super(
          const AgentConfigState(),
        ) {
    _dispatch();
  }

  final UserListener _userListener;
  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Future<void> close() async {
    await _userListener.stop();
    return super.close();
  }

  void _dispatch() {
    on<AgentConfigEvent>((event, emit) async {
      await event.when(
        started: () => _handleStarted(emit),
        loadConfigs: () => _handleLoadConfigs(emit),
        createConfig: (config) => _handleCreateConfig(config, emit),
        updateConfig: (config) => _handleUpdateConfig(config, emit),
        deleteConfig: (configId) => _handleDeleteConfig(configId, emit),
        setDefaultConfig: (configId) => _handleSetDefaultConfig(configId, emit),
        validateConfig: (config) => _handleValidateConfig(config, emit),
        importConfigs: (configsJson) => _handleImportConfigs(configsJson, emit),
        exportConfigs: () => _handleExportConfigs(emit),
        duplicateConfig: (configId) => _handleDuplicateConfig(configId, emit),
        toggleConfigEnabled: (configId) => _handleToggleConfigEnabled(configId, emit),
        searchConfigs: (query) => _handleSearchConfigs(query, emit),
        clearSearch: () => _handleClearSearch(emit),
        didReceiveUserProfile: (userProfile) => _handleDidReceiveUserProfile(userProfile, emit),
      );
    });
  }

  /// 处理启动事件
  Future<void> _handleStarted(Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    
    _userListener.start(
      onProfileUpdated: _onProfileUpdated,
    );
    
    await _loadConfigsFromStorage(emit);
  }

  /// 处理加载配置事件
  Future<void> _handleLoadConfigs(Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    await _loadConfigsFromStorage(emit);
  }

  /// 处理创建配置事件
  Future<void> _handleCreateConfig(AgentConfig config, Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    
    // 验证配置
    final validationResult = _validateAgentConfig(config);
    if (validationResult.isNotEmpty) {
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '配置验证失败: ${validationResult.join(', ')}',
      ));
      return;
    }
    
    // 检查名称是否重复
    if (state.configs.any((c) => c.name == config.name && c.id != config.id)) {
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '智能体名称已存在',
      ));
      return;
    }
    
    try {
      final updatedConfigs = [...state.configs, config];
      await _saveConfigsToStorage(updatedConfigs);
      
      emit(state.copyWith(
        isLoading: false,
        configs: updatedConfigs,
        errorMessage: null,
        successMessage: '智能体配置创建成功',
      ));
    } catch (e) {
      Log.error('Failed to create agent config: $e');
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '创建配置失败: $e',
      ));
    }
  }

  /// 处理更新配置事件
  Future<void> _handleUpdateConfig(AgentConfig config, Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    
    // 验证配置
    final validationResult = _validateAgentConfig(config);
    if (validationResult.isNotEmpty) {
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '配置验证失败: ${validationResult.join(', ')}',
      ));
      return;
    }
    
    // 检查名称是否重复（排除自身）
    if (state.configs.any((c) => c.name == config.name && c.id != config.id)) {
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '智能体名称已存在',
      ));
      return;
    }
    
    try {
      final updatedConfigs = state.configs.map((c) => c.id == config.id ? config : c).toList();
      await _saveConfigsToStorage(updatedConfigs);
      
      emit(state.copyWith(
        isLoading: false,
        configs: updatedConfigs,
        errorMessage: null,
        successMessage: '智能体配置更新成功',
      ));
    } catch (e) {
      Log.error('Failed to update agent config: $e');
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '更新配置失败: $e',
      ));
    }
  }

  /// 处理删除配置事件
  Future<void> _handleDeleteConfig(String configId, Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    
    try {
      final updatedConfigs = state.configs.where((c) => c.id != configId).toList();
      await _saveConfigsToStorage(updatedConfigs);
      
      // 如果删除的是默认配置，清除默认配置
      String? newDefaultConfigId = state.defaultConfigId;
      if (state.defaultConfigId == configId) {
        newDefaultConfigId = null;
        await _saveDefaultConfigId(null);
      }
      
      emit(state.copyWith(
        isLoading: false,
        configs: updatedConfigs,
        defaultConfigId: newDefaultConfigId,
        errorMessage: null,
        successMessage: '智能体配置删除成功',
      ));
    } catch (e) {
      Log.error('Failed to delete agent config: $e');
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '删除配置失败: $e',
      ));
    }
  }

  /// 处理设置默认配置事件
  Future<void> _handleSetDefaultConfig(String? configId, Emitter<AgentConfigState> emit) async {
    try {
      await _saveDefaultConfigId(configId);
      
      emit(state.copyWith(
        defaultConfigId: configId,
        successMessage: configId != null ? '默认智能体设置成功' : '默认智能体已清除',
      ));
    } catch (e) {
      Log.error('Failed to set default agent config: $e');
      emit(state.copyWith(
        errorMessage: '设置默认智能体失败: $e',
      ));
    }
  }

  /// 处理验证配置事件
  Future<void> _handleValidateConfig(AgentConfig config, Emitter<AgentConfigState> emit) async {
    final validationResult = _validateAgentConfig(config);
    
    if (validationResult.isEmpty) {
      emit(state.copyWith(
        successMessage: '配置验证通过',
        errorMessage: null,
      ));
    } else {
      emit(state.copyWith(
        errorMessage: '配置验证失败: ${validationResult.join(', ')}',
      ));
    }
  }

  /// 处理导入配置事件
  Future<void> _handleImportConfigs(String configsJson, Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    
    try {
      final jsonData = jsonDecode(configsJson);
      
      if (jsonData is! Map<String, dynamic>) {
        throw const FormatException('无效的JSON格式');
      }
      
      final List<AgentConfig> importedConfigs = [];
      
      // 处理单个配置导入
      if (jsonData.containsKey('id') && jsonData.containsKey('name')) {
        final config = AgentConfig.fromJson(jsonData);
        final validationResult = _validateAgentConfig(config);
        if (validationResult.isEmpty) {
          importedConfigs.add(config);
        }
      }
      // 处理批量配置导入
      else if (jsonData.containsKey('configs') && jsonData['configs'] is List) {
        final configList = jsonData['configs'] as List;
        for (final configData in configList) {
          if (configData is Map<String, dynamic>) {
            try {
              final config = AgentConfig.fromJson(configData);
              final validationResult = _validateAgentConfig(config);
              if (validationResult.isEmpty) {
                importedConfigs.add(config);
              }
            } catch (e) {
              Log.warn('Skipped invalid config during import: $e');
            }
          }
        }
      }
      
      if (importedConfigs.isEmpty) {
        emit(state.copyWith(
          isLoading: false,
          errorMessage: '没有找到有效的配置数据',
        ));
        return;
      }
      
      // 处理名称冲突
      final existingNames = state.configs.map((c) => c.name).toSet();
      final resolvedConfigs = <AgentConfig>[];
      
      for (final config in importedConfigs) {
        if (existingNames.contains(config.name)) {
          // 生成新名称
          String newName = config.name;
          int counter = 1;
          while (existingNames.contains(newName)) {
            newName = '${config.name} ($counter)';
            counter++;
          }
          resolvedConfigs.add(config.copyWith(
            name: newName,
            id: _generateConfigId(),
            createdAt: DateTime.now(),
          ));
          existingNames.add(newName);
        } else {
          resolvedConfigs.add(config.copyWith(
            id: _generateConfigId(),
            createdAt: DateTime.now(),
          ));
          existingNames.add(config.name);
        }
      }
      
      final updatedConfigs = [...state.configs, ...resolvedConfigs];
      await _saveConfigsToStorage(updatedConfigs);
      
      emit(state.copyWith(
        isLoading: false,
        configs: updatedConfigs,
        errorMessage: null,
        successMessage: '成功导入 ${resolvedConfigs.length} 个智能体配置',
      ));
    } catch (e) {
      Log.error('Failed to import agent configs: $e');
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '导入配置失败: $e',
      ));
    }
  }

  /// 处理导出配置事件
  Future<void> _handleExportConfigs(Emitter<AgentConfigState> emit) async {
    try {
      final exportData = {
        'version': '1.0',
        'exportedAt': DateTime.now().toIso8601String(),
        'configs': state.configs.map((c) => c.toJson()).toList(),
        'defaultConfigId': state.defaultConfigId,
      };
      
      final exportJson = jsonEncode(exportData);
      
      emit(state.copyWith(
        exportData: exportJson,
        successMessage: '配置导出成功',
      ));
    } catch (e) {
      Log.error('Failed to export agent configs: $e');
      emit(state.copyWith(
        errorMessage: '导出配置失败: $e',
      ));
    }
  }

  /// 处理复制配置事件
  Future<void> _handleDuplicateConfig(String configId, Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(isLoading: true));
    
    try {
      final originalConfig = state.configs.firstWhere((c) => c.id == configId);
      
      // 生成新名称
      final existingNames = state.configs.map((c) => c.name).toSet();
      String newName = '${originalConfig.name} 副本';
      int counter = 1;
      while (existingNames.contains(newName)) {
        newName = '${originalConfig.name} 副本 ($counter)';
        counter++;
      }
      
      final duplicatedConfig = originalConfig.copyWith(
        id: _generateConfigId(),
        name: newName,
        createdAt: DateTime.now(),
        updatedAt: null,
      );
      
      final updatedConfigs = [...state.configs, duplicatedConfig];
      await _saveConfigsToStorage(updatedConfigs);
      
      emit(state.copyWith(
        isLoading: false,
        configs: updatedConfigs,
        errorMessage: null,
        successMessage: '智能体配置复制成功',
      ));
    } catch (e) {
      Log.error('Failed to duplicate agent config: $e');
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '复制配置失败: $e',
      ));
    }
  }

  /// 处理切换配置启用状态事件
  Future<void> _handleToggleConfigEnabled(String configId, Emitter<AgentConfigState> emit) async {
    try {
      final updatedConfigs = state.configs.map((c) {
        if (c.id == configId) {
          return c.copyWith(
            isEnabled: !c.isEnabled,
            updatedAt: DateTime.now(),
          );
        }
        return c;
      }).toList();
      
      await _saveConfigsToStorage(updatedConfigs);
      
      emit(state.copyWith(
        configs: updatedConfigs,
        successMessage: '智能体状态更新成功',
      ));
    } catch (e) {
      Log.error('Failed to toggle agent config enabled: $e');
      emit(state.copyWith(
        errorMessage: '更新智能体状态失败: $e',
      ));
    }
  }

  /// 处理搜索配置事件
  Future<void> _handleSearchConfigs(String query, Emitter<AgentConfigState> emit) async {
    if (query.trim().isEmpty) {
      emit(state.copyWith(
        filteredConfigs: state.configs,
        searchQuery: '',
      ));
      return;
    }
    
    final lowercaseQuery = query.toLowerCase();
    final filteredConfigs = state.configs.where((config) {
      return config.name.toLowerCase().contains(lowercaseQuery) ||
             config.description.toLowerCase().contains(lowercaseQuery) ||
             config.personality.toLowerCase().contains(lowercaseQuery);
    }).toList();
    
    emit(state.copyWith(
      filteredConfigs: filteredConfigs,
      searchQuery: query,
    ));
  }

  /// 处理清除搜索事件
  Future<void> _handleClearSearch(Emitter<AgentConfigState> emit) async {
    emit(state.copyWith(
      filteredConfigs: state.configs,
      searchQuery: '',
    ));
  }

  /// 处理用户资料更新事件
  Future<void> _handleDidReceiveUserProfile(UserProfilePB userProfile, Emitter<AgentConfigState> emit) async {
    // 用户资料更新时可能需要重新加载配置
    await _loadConfigsFromStorage(emit);
  }

  /// 从存储加载配置
  Future<void> _loadConfigsFromStorage(Emitter<AgentConfigState> emit) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      
      // 加载配置列表
      final configsJson = prefs.getString(agentConfigListKey);
      List<AgentConfig> configs = [];
      
      if (configsJson != null) {
        final configList = jsonDecode(configsJson) as List;
        configs = configList
            .map((json) => AgentConfig.fromJson(json as Map<String, dynamic>))
            .toList();
      }
      
      // 如果没有配置，创建默认配置
      if (configs.isEmpty) {
        configs = [_createDefaultAgentConfig()];
        await _saveConfigsToStorage(configs);
      }
      
      // 加载默认配置ID
      final defaultConfigId = prefs.getString(defaultAgentConfigKey);
      
      emit(state.copyWith(
        isLoading: false,
        configs: configs,
        filteredConfigs: configs,
        defaultConfigId: defaultConfigId,
        errorMessage: null,
      ));
    } catch (e) {
      Log.error('Failed to load agent configs from storage: $e');
      emit(state.copyWith(
        isLoading: false,
        errorMessage: '加载配置失败: $e',
      ));
    }
  }

  /// 保存配置到存储
  Future<void> _saveConfigsToStorage(List<AgentConfig> configs) async {
    final prefs = await SharedPreferences.getInstance();
    final configsJson = jsonEncode(configs.map((c) => c.toJson()).toList());
    await prefs.setString(agentConfigListKey, configsJson);
  }

  /// 保存默认配置ID
  Future<void> _saveDefaultConfigId(String? configId) async {
    final prefs = await SharedPreferences.getInstance();
    if (configId != null) {
      await prefs.setString(defaultAgentConfigKey, configId);
    } else {
      await prefs.remove(defaultAgentConfigKey);
    }
  }

  /// 验证智能体配置
  List<String> _validateAgentConfig(AgentConfig config) {
    final errors = <String>[];
    
    // 验证名称
    if (config.name.trim().isEmpty) {
      errors.add('智能体名称不能为空');
    } else if (config.name.length > 50) {
      errors.add('智能体名称不能超过50个字符');
    }
    
    // 验证描述长度
    if (config.description.length > 500) {
      errors.add('智能体描述不能超过500个字符');
    }
    
    // 验证个性描述长度
    if (config.personality.length > 1000) {
      errors.add('个性描述不能超过1000个字符');
    }
    
    // 验证系统提示词长度
    if (config.systemPrompt.length > 5000) {
      errors.add('系统提示词不能超过5000个字符');
    }
    
    // 验证并发工具数
    if (config.maxConcurrentTools < 1 || config.maxConcurrentTools > 10) {
      errors.add('最大并发工具数必须在1-10之间');
    }
    
    // 验证超时时间
    if (config.toolTimeoutSeconds < 5 || config.toolTimeoutSeconds > 300) {
      errors.add('工具超时时间必须在5-300秒之间');
    }
    
    // 验证语言偏好
    final supportedLanguages = ['zh-CN', 'en-US', 'ja-JP', 'ko-KR'];
    if (!supportedLanguages.contains(config.languagePreference)) {
      errors.add('不支持的语言偏好');
    }
    
    return errors;
  }

  /// 创建默认智能体配置
  AgentConfig _createDefaultAgentConfig() {
    return AgentConfig(
      id: _generateConfigId(),
      name: '默认智能体',
      description: '通用智能体助手，适用于各种任务',
      personality: '友好、专业、乐于助人的智能助手',
      systemPrompt: '你是一个专业的AI助手，能够帮助用户完成各种任务。请始终保持友好、准确和有用的回复。',
      languagePreference: 'zh-CN',
      createdAt: DateTime.now(),
      isEnabled: true,
      maxConcurrentTools: 3,
      toolTimeoutSeconds: 30,
    );
  }

  /// 生成配置ID
  String _generateConfigId() {
    return 'agent_${DateTime.now().millisecondsSinceEpoch}_${userProfile.id}';
  }

  /// 用户资料更新回调
  void _onProfileUpdated(
    FlowyResult<UserProfilePB, FlowyError> userProfileOrFailed,
  ) =>
      userProfileOrFailed.fold(
        (profile) => add(AgentConfigEvent.didReceiveUserProfile(profile)),
        (err) => Log.error(err),
      );
}

/// 智能体配置事件
@freezed
class AgentConfigEvent with _$AgentConfigEvent {
  /// 启动事件
  const factory AgentConfigEvent.started() = _Started;
  
  /// 加载配置事件
  const factory AgentConfigEvent.loadConfigs() = _LoadConfigs;
  
  /// 创建配置事件
  const factory AgentConfigEvent.createConfig(AgentConfig config) = _CreateConfig;
  
  /// 更新配置事件
  const factory AgentConfigEvent.updateConfig(AgentConfig config) = _UpdateConfig;
  
  /// 删除配置事件
  const factory AgentConfigEvent.deleteConfig(String configId) = _DeleteConfig;
  
  /// 设置默认配置事件
  const factory AgentConfigEvent.setDefaultConfig(String? configId) = _SetDefaultConfig;
  
  /// 验证配置事件
  const factory AgentConfigEvent.validateConfig(AgentConfig config) = _ValidateConfig;
  
  /// 导入配置事件
  const factory AgentConfigEvent.importConfigs(String configsJson) = _ImportConfigs;
  
  /// 导出配置事件
  const factory AgentConfigEvent.exportConfigs() = _ExportConfigs;
  
  /// 复制配置事件
  const factory AgentConfigEvent.duplicateConfig(String configId) = _DuplicateConfig;
  
  /// 切换配置启用状态事件
  const factory AgentConfigEvent.toggleConfigEnabled(String configId) = _ToggleConfigEnabled;
  
  /// 搜索配置事件
  const factory AgentConfigEvent.searchConfigs(String query) = _SearchConfigs;
  
  /// 清除搜索事件
  const factory AgentConfigEvent.clearSearch() = _ClearSearch;
  
  /// 接收用户资料事件
  const factory AgentConfigEvent.didReceiveUserProfile(
    UserProfilePB newUserProfile,
  ) = _DidReceiveUserProfile;
}

/// 智能体配置状态
@freezed
class AgentConfigState with _$AgentConfigState {
  const AgentConfigState._();
  
  const factory AgentConfigState({
    /// 是否正在加载
    @Default(false) bool isLoading,
    /// 智能体配置列表
    @Default([]) List<AgentConfig> configs,
    /// 过滤后的配置列表（用于搜索）
    @Default([]) List<AgentConfig> filteredConfigs,
    /// 默认配置ID
    String? defaultConfigId,
    /// 搜索查询
    @Default('') String searchQuery,
    /// 错误信息
    String? errorMessage,
    /// 成功信息
    String? successMessage,
    /// 导出数据
    String? exportData,
  }) = _AgentConfigState;
  
  /// 获取默认配置
  AgentConfig? get defaultConfig {
    if (defaultConfigId == null) return null;
    try {
      return configs.firstWhere((c) => c.id == defaultConfigId);
    } catch (e) {
      return null;
    }
  }
  
  /// 获取启用的配置列表
  List<AgentConfig> get enabledConfigs {
    return configs.where((c) => c.isEnabled).toList();
  }
  
  /// 获取当前显示的配置列表（考虑搜索过滤）
  List<AgentConfig> get displayConfigs {
    return searchQuery.isEmpty ? configs : filteredConfigs;
  }
}
