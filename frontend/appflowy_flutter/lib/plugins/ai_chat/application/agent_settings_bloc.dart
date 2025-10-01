import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'agent_settings_bloc.freezed.dart';

/// 智能体配置BLoC - 管理智能体配置的状态和业务逻辑
class AgentSettingsBloc extends Bloc<AgentSettingsEvent, AgentSettingsState> {
  AgentSettingsBloc() : super(const AgentSettingsState()) {
    _dispatch();
  }

  void _dispatch() {
    on<AgentSettingsEvent>((event, emit) async {
      event.when(
        started: () => _handleStarted(emit),
        loadAgentList: () => _handleLoadAgentList(emit),
        createAgent: (request) => _handleCreateAgent(request, emit),
        updateAgent: (request) => _handleUpdateAgent(request, emit),
        deleteAgent: (agentId) => _handleDeleteAgent(agentId, emit),
        getAgent: (agentId) => _handleGetAgent(agentId, emit),
        validateAgentConfig: (config) => _handleValidateAgentConfig(config, emit),
        didReceiveAgentList: (agents) => _handleDidReceiveAgentList(agents, emit),
        didReceiveAgent: (agent) => _handleDidReceiveAgent(agent, emit),
        didReceiveValidationResult: (result) => _handleDidReceiveValidationResult(result, emit),
        didReceiveError: (error) => _handleDidReceiveError(error, emit),
      );
    });
  }

  /// 处理初始化事件
  Future<void> _handleStarted(Emitter<AgentSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadAgentList();
  }

  /// 处理加载智能体列表事件
  Future<void> _handleLoadAgentList(Emitter<AgentSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadAgentList();
  }

  /// 处理创建智能体事件
  Future<void> _handleCreateAgent(
    CreateAgentRequestPB request,
    Emitter<AgentSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      // 验证请求
      final validationError = _validateCreateRequest(request);
      if (validationError != null) {
        emit(state.copyWith(
          isOperating: false,
          error: validationError,
        ));
        return;
      }

      final result = await AIEventCreateAgent(request).send();
      await result.fold(
        (agent) async {
          Log.info('智能体创建成功: ${agent.name}');
          // 重新加载智能体列表
          await _loadAgentList();
          // 检查emit是否已完成，避免在event handler完成后调用emit
          if (!emit.isDone) {
            emit(state.copyWith(isOperating: false));
          }
        },
        (error) {
          Log.error('创建智能体失败: $error');
          if (!emit.isDone) {
            emit(state.copyWith(
              isOperating: false,
              error: '创建智能体失败: ${error.msg}',
            ));
          }
        },
      );
    } catch (e) {
      Log.error('创建智能体异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '创建智能体异常: $e',
        ));
      }
    }
  }

  /// 处理更新智能体事件
  Future<void> _handleUpdateAgent(
    UpdateAgentRequestPB request,
    Emitter<AgentSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      // 验证请求
      final validationError = _validateUpdateRequest(request);
      if (validationError != null) {
        emit(state.copyWith(
          isOperating: false,
          error: validationError,
        ));
        return;
      }

      final result = await AIEventUpdateAgent(request).send();
      await result.fold(
        (agent) async {
          Log.info('智能体更新成功: ${agent.name}');
          // 重新加载智能体列表
          await _loadAgentList();
          // 检查emit是否已完成，避免在event handler完成后调用emit
          if (!emit.isDone) {
            emit(state.copyWith(isOperating: false));
          }
        },
        (error) {
          Log.error('更新智能体失败: $error');
          if (!emit.isDone) {
            emit(state.copyWith(
              isOperating: false,
              error: '更新智能体失败: ${error.msg}',
            ));
          }
        },
      );
    } catch (e) {
      Log.error('更新智能体异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '更新智能体异常: $e',
        ));
      }
    }
  }

  /// 处理删除智能体事件
  Future<void> _handleDeleteAgent(
    String agentId,
    Emitter<AgentSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final request = DeleteAgentRequestPB()..id = agentId;
      final result = await AIEventDeleteAgent(request).send();
      await result.fold(
        (_) async {
          Log.info('智能体删除成功: $agentId');
          // 重新加载智能体列表
          await _loadAgentList();
          // 检查emit是否已完成，避免在event handler完成后调用emit
          if (!emit.isDone) {
            emit(state.copyWith(isOperating: false));
          }
        },
        (error) {
          Log.error('删除智能体失败: $error');
          if (!emit.isDone) {
            emit(state.copyWith(
              isOperating: false,
              error: '删除智能体失败: ${error.msg}',
            ));
          }
        },
      );
    } catch (e) {
      Log.error('删除智能体异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '删除智能体异常: $e',
        ));
      }
    }
  }

  /// 处理获取智能体事件
  Future<void> _handleGetAgent(
    String agentId,
    Emitter<AgentSettingsState> emit,
  ) async {
    emit(state.copyWith(isLoading: true, error: null));
    
    try {
      final request = GetAgentRequestPB()..id = agentId;
      final result = await AIEventGetAgent(request).send();
      result.fold(
        (agent) {
          add(AgentSettingsEvent.didReceiveAgent(agent));
        },
        (error) {
          Log.error('获取智能体失败: $error');
          add(AgentSettingsEvent.didReceiveError('获取智能体失败: ${error.msg}'));
        },
      );
      
    } catch (e) {
      Log.error('获取智能体异常: $e');
      if (!isClosed) {
        add(AgentSettingsEvent.didReceiveError('获取智能体异常: $e'));
      }
    }
  }

  /// 处理验证智能体配置事件
  Future<void> _handleValidateAgentConfig(
    AgentConfigPB config,
    Emitter<AgentSettingsState> emit,
  ) async {
    final validationErrors = <String>[];
    
    // 基本验证
    if (config.name.trim().isEmpty) {
      validationErrors.add('智能体名称不能为空');
    } else if (config.name.length > 50) {
      validationErrors.add('智能体名称不能超过50个字符');
    }
    
    if (config.description.length > 500) {
      validationErrors.add('智能体描述不能超过500个字符');
    }
    
    if (config.personality.length > 2000) {
      validationErrors.add('个性描述不能超过2000个字符');
    }
    
    // 能力配置验证
    if (config.hasCapabilities()) {
      final capabilities = config.capabilities;
      
      if (capabilities.maxPlanningSteps < 1 || capabilities.maxPlanningSteps > 100) {
        validationErrors.add('最大规划步骤数必须在1-100之间');
      }
      
      if (capabilities.maxToolCalls < 1 || capabilities.maxToolCalls > 1000) {
        validationErrors.add('最大工具调用次数必须在1-1000之间');
      }
      
      if (capabilities.memoryLimit < 10 || capabilities.memoryLimit > 10000) {
        validationErrors.add('会话记忆长度限制必须在10-10000之间');
      }
    }
    
    // 工具配置验证
    if (config.availableTools.isEmpty) {
      validationErrors.add('至少需要选择一个可用工具');
    }
    
    final validationResult = validationErrors.isEmpty;
    final errorMessage = validationErrors.isNotEmpty ? validationErrors.join('; ') : null;
    
    add(AgentSettingsEvent.didReceiveValidationResult(
      AgentValidationResult(isValid: validationResult, errorMessage: errorMessage)
    ));
  }

  /// 处理接收到智能体列表事件
  void _handleDidReceiveAgentList(
    AgentListPB agents,
    Emitter<AgentSettingsState> emit,
  ) {
    Log.info('接收到智能体列表，数量: ${agents.agents.length}');
    
    // 调试：打印每个智能体的信息
    for (var i = 0; i < agents.agents.length; i++) {
      final agent = agents.agents[i];
      Log.info('  智能体 ${i + 1}: ${agent.name} (${agent.id})');
    }
    
    emit(state.copyWith(
      agents: agents.agents,
      isLoading: false,
      error: null,
    ));
    
    // 调试：确认state已更新
    Log.info('State更新后agents数量: ${state.agents.length}');
  }

  /// 处理接收到智能体事件
  void _handleDidReceiveAgent(
    AgentConfigPB agent,
    Emitter<AgentSettingsState> emit,
  ) {
    Log.info('接收到智能体: ${agent.name} (${agent.id})');
    emit(state.copyWith(
      selectedAgent: agent,
      isLoading: false,
      error: null,
    ));
  }

  /// 处理接收到验证结果事件
  void _handleDidReceiveValidationResult(
    AgentValidationResult result,
    Emitter<AgentSettingsState> emit,
  ) {
    Log.info('智能体配置验证结果: ${result.isValid}');
    emit(state.copyWith(
      validationResult: result,
      error: result.isValid ? null : result.errorMessage,
    ));
  }

  /// 处理接收到错误事件
  void _handleDidReceiveError(
    String error,
    Emitter<AgentSettingsState> emit,
  ) {
    Log.error('智能体设置错误: $error');
    emit(state.copyWith(
      isLoading: false,
      isOperating: false,
      error: error,
    ));
  }

  /// 加载智能体列表的私有方法
  Future<void> _loadAgentList() async {
    try {
      final result = await AIEventGetAgentList().send();
      result.fold(
        (agents) {
          if (!isClosed) {
            add(AgentSettingsEvent.didReceiveAgentList(agents));
          }
        },
        (error) {
          Log.error('加载智能体列表失败: $error');
          if (!isClosed) {
            add(AgentSettingsEvent.didReceiveError('加载智能体列表失败: ${error.msg}'));
          }
        },
      );
      
    } catch (e) {
      Log.error('加载智能体列表异常: $e');
      if (!isClosed) {
        add(AgentSettingsEvent.didReceiveError('加载智能体列表异常: $e'));
      }
    }
  }

  /// 验证创建请求
  String? _validateCreateRequest(CreateAgentRequestPB request) {
    if (request.name.trim().isEmpty) {
      return '智能体名称不能为空';
    }
    
    if (request.name.length > 50) {
      return '智能体名称不能超过50个字符';
    }
    
    if (request.availableTools.isEmpty) {
      return '至少需要选择一个可用工具';
    }
    
    return null;
  }

  /// 验证更新请求
  String? _validateUpdateRequest(UpdateAgentRequestPB request) {
    if (request.id.trim().isEmpty) {
      return '智能体ID不能为空';
    }
    
    if (request.hasName() && request.name.trim().isEmpty) {
      return '智能体名称不能为空';
    }
    
    if (request.hasName() && request.name.length > 50) {
      return '智能体名称不能超过50个字符';
    }
    
    return null;
  }

  /// 获取智能体状态显示文本
  String getAgentStatusText(AgentStatusPB status) {
    switch (status) {
      case AgentStatusPB.AgentActive:
        return '活跃';
      case AgentStatusPB.AgentPaused:
        return '暂停';
      case AgentStatusPB.AgentDeleted:
        return '已删除';
      default:
        return '未知';
    }
  }

  /// 获取智能体能力摘要
  String getCapabilitiesSummary(AgentCapabilitiesPB capabilities) {
    final features = <String>[];
    
    if (capabilities.enablePlanning) features.add('规划');
    if (capabilities.enableToolCalling) features.add('工具调用');
    if (capabilities.enableReflection) features.add('反思');
    if (capabilities.enableMemory) features.add('记忆');
    
    return features.isEmpty ? '无特殊能力' : features.join(', ');
  }

  /// 检查智能体是否可以编辑
  bool canEditAgent(AgentConfigPB agent) {
    return agent.status == AgentStatusPB.AgentActive || 
           agent.status == AgentStatusPB.AgentPaused;
  }

  /// 检查智能体是否可以删除
  bool canDeleteAgent(AgentConfigPB agent) {
    return agent.status != AgentStatusPB.AgentDeleted;
  }
}

/// 智能体验证结果
class AgentValidationResult {
  final bool isValid;
  final String? errorMessage;
  
  const AgentValidationResult({
    required this.isValid,
    this.errorMessage,
  });
}

/// 智能体设置事件定义
@freezed
class AgentSettingsEvent with _$AgentSettingsEvent {
  /// 开始初始化
  const factory AgentSettingsEvent.started() = _Started;

  /// 加载智能体列表
  const factory AgentSettingsEvent.loadAgentList() = _LoadAgentList;

  /// 创建智能体
  const factory AgentSettingsEvent.createAgent(CreateAgentRequestPB request) = _CreateAgent;

  /// 更新智能体
  const factory AgentSettingsEvent.updateAgent(UpdateAgentRequestPB request) = _UpdateAgent;

  /// 删除智能体
  const factory AgentSettingsEvent.deleteAgent(String agentId) = _DeleteAgent;

  /// 获取智能体
  const factory AgentSettingsEvent.getAgent(String agentId) = _GetAgent;

  /// 验证智能体配置
  const factory AgentSettingsEvent.validateAgentConfig(AgentConfigPB config) = _ValidateAgentConfig;

  /// 接收到智能体列表
  const factory AgentSettingsEvent.didReceiveAgentList(AgentListPB agents) = _DidReceiveAgentList;

  /// 接收到智能体
  const factory AgentSettingsEvent.didReceiveAgent(AgentConfigPB agent) = _DidReceiveAgent;

  /// 接收到验证结果
  const factory AgentSettingsEvent.didReceiveValidationResult(AgentValidationResult result) = _DidReceiveValidationResult;

  /// 接收到错误
  const factory AgentSettingsEvent.didReceiveError(String error) = _DidReceiveError;
}

/// 智能体设置状态定义
@freezed
class AgentSettingsState with _$AgentSettingsState {
  const factory AgentSettingsState({
    /// 智能体列表
    @Default([]) List<AgentConfigPB> agents,
    
    /// 当前选中的智能体
    AgentConfigPB? selectedAgent,
    
    /// 验证结果
    AgentValidationResult? validationResult,
    
    /// 是否正在加载
    @Default(false) bool isLoading,
    
    /// 是否正在执行操作（创建、更新、删除）
    @Default(false) bool isOperating,
    
    /// 错误信息
    String? error,
  }) = _AgentSettingsState;
}
