import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'mcp_settings_bloc.freezed.dart';

/// MCP配置BLoC - 管理MCP服务器配置的状态和业务逻辑
class MCPSettingsBloc extends Bloc<MCPSettingsEvent, MCPSettingsState> {
  MCPSettingsBloc() : super(const MCPSettingsState()) {
    _dispatch();
  }

  void _dispatch() {
    // 异步事件处理
    on<_Started>((event, emit) async => await _handleStarted(emit));
    on<_LoadServerList>((event, emit) async => await _handleLoadServerList(emit));
    on<_AddServer>((event, emit) async => await _handleAddServer(event.config, emit));
    on<_UpdateServer>((event, emit) async => await _handleUpdateServer(event.config, emit));
    on<_RemoveServer>((event, emit) async => await _handleRemoveServer(event.serverId, emit));
    on<_ConnectServer>((event, emit) async => await _handleConnectServer(event.serverId, emit));
    on<_DisconnectServer>((event, emit) async => await _handleDisconnectServer(event.serverId, emit));
    on<_TestConnection>((event, emit) async => await _handleTestConnection(event.serverId, emit));
    on<_LoadToolList>((event, emit) async => await _handleLoadToolList(event.serverId, emit));
    on<_CallTool>((event, emit) async => await _handleCallTool(event.serverId, event.toolName, event.arguments, emit));
    on<_RefreshTools>((event, emit) async => await _handleRefreshTools(event.serverId, emit));
    
    // 同步事件处理
    on<_DidReceiveServerList>((event, emit) => _handleDidReceiveServerList(event.servers, emit));
    on<_DidReceiveServerStatus>((event, emit) => _handleDidReceiveServerStatus(event.status, emit));
    on<_DidReceiveToolList>((event, emit) => _handleDidReceiveToolList(event.serverId, event.tools, emit));
    on<_DidReceiveToolCallResponse>((event, emit) => _handleDidReceiveToolCallResponse(event.response, emit));
    on<_DidReceiveError>((event, emit) => _handleDidReceiveError(event.error, emit));
  }

  /// 处理初始化事件
  Future<void> _handleStarted(Emitter<MCPSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadServerListAndEmit(emit);
  }

  /// 处理加载服务器列表事件
  Future<void> _handleLoadServerList(Emitter<MCPSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadServerListAndEmit(emit);
  }

  /// 处理添加服务器事件
  Future<void> _handleAddServer(
    MCPServerConfigPB config,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final result = await AIEventAddMCPServer(config).send();
      
      // 使用临时变量保存fold结果，然后执行异步操作
      final isSuccess = result.fold(
        (success) {
          Log.info('MCP服务器添加成功: ${config.name}');
          return true;
        },
        (error) {
          Log.error('添加MCP服务器失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        // 直接加载服务器列表
        await _loadServerListAndEmit(emit);
      } else {
        // 获取错误信息
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '添加服务器失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('添加MCP服务器异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '添加服务器异常: $e',
        ));
      }
    }
  }

  /// 处理更新服务器事件
  Future<void> _handleUpdateServer(
    MCPServerConfigPB config,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final result = await AIEventUpdateMCPServer(config).send();
      
      // 使用临时变量保存fold结果，然后执行异步操作
      final isSuccess = result.fold(
        (success) {
          Log.info('MCP服务器更新成功: ${config.name}');
          return true;
        },
        (error) {
          Log.error('更新MCP服务器失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        // 重新加载服务器列表
        await _loadServerListAndEmit(emit);
      } else {
        // 获取错误信息
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '更新服务器失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('更新MCP服务器异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '更新服务器异常: $e',
        ));
      }
    }
  }

  /// 处理删除服务器事件
  Future<void> _handleRemoveServer(
    String serverId,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final request = MCPDisconnectServerRequestPB()..serverId = serverId;
      final result = await AIEventRemoveMCPServer(request).send();
      
      // 使用临时变量保存fold结果，然后执行异步操作
      final isSuccess = result.fold(
        (success) {
          Log.info('MCP服务器删除成功: $serverId');
          return true;
        },
        (error) {
          Log.error('删除MCP服务器失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        // 重新加载服务器列表
        await _loadServerListAndEmit(emit);
      } else {
        // 获取错误信息
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '删除服务器失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('删除MCP服务器异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '删除服务器异常: $e',
        ));
      }
    }
  }

  /// 处理连接服务器事件
  Future<void> _handleConnectServer(
    String serverId,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(
      connectingServers: {...state.connectingServers, serverId},
      error: null,
    ));
    
    try {
      final request = MCPConnectServerRequestPB()..serverId = serverId;
      final result = await AIEventConnectMCPServer(request).send();
      
      // 检查emit是否还有效
      if (emit.isDone) return;
      
      result.fold(
        (status) {
          Log.info('MCP服务器连接结果: ${status.serverId}, 连接状态: ${status.isConnected}');
          add(MCPSettingsEvent.didReceiveServerStatus(status));
          
          // 连接成功后自动加载工具列表
          if (status.isConnected) {
            Log.info('连接成功，自动加载工具列表: $serverId');
            add(MCPSettingsEvent.loadToolList(serverId));
          }
        },
        (error) {
          Log.error('连接MCP服务器失败: $error');
          final connectingServers = Set<String>.from(state.connectingServers);
          connectingServers.remove(serverId);
          emit(state.copyWith(
            connectingServers: connectingServers,
            error: '连接服务器失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('连接MCP服务器异常: $e');
      if (!emit.isDone) {
        final connectingServers = Set<String>.from(state.connectingServers);
        connectingServers.remove(serverId);
        emit(state.copyWith(
          connectingServers: connectingServers,
          error: '连接服务器异常: $e',
        ));
      }
    }
  }

  /// 处理断开服务器连接事件
  Future<void> _handleDisconnectServer(
    String serverId,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(
      connectingServers: {...state.connectingServers, serverId},
      error: null,
    ));
    
    try {
      final request = MCPDisconnectServerRequestPB()..serverId = serverId;
      final result = await AIEventDisconnectMCPServer(request).send();
      
      // 使用临时变量保存fold结果，然后执行异步操作
      final isSuccess = result.fold(
        (success) {
          Log.info('MCP服务器断开连接成功: $serverId');
          return true;
        },
        (error) {
          Log.error('断开MCP服务器连接失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        // 更新服务器状态
        final updatedStatuses = Map<String, MCPServerStatusPB>.from(state.serverStatuses);
        updatedStatuses[serverId] = MCPServerStatusPB()
          ..serverId = serverId
          ..isConnected = false;
        
        final connectingServers = Set<String>.from(state.connectingServers);
        connectingServers.remove(serverId);
        
        if (!emit.isDone) {
          emit(state.copyWith(
            serverStatuses: updatedStatuses,
            connectingServers: connectingServers,
          ));
        }
      } else {
        // 获取错误信息
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        final connectingServers = Set<String>.from(state.connectingServers);
        connectingServers.remove(serverId);
        if (!emit.isDone) {
          emit(state.copyWith(
            connectingServers: connectingServers,
            error: '断开连接失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('断开MCP服务器连接异常: $e');
      final connectingServers = Set<String>.from(state.connectingServers);
      connectingServers.remove(serverId);
      if (!emit.isDone) {
        emit(state.copyWith(
          connectingServers: connectingServers,
          error: '断开连接异常: $e',
        ));
      }
    }
  }

  /// 处理测试连接事件
  Future<void> _handleTestConnection(
    String serverId,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(
      testingServers: {...state.testingServers, serverId},
      error: null,
    ));
    
    try {
      final request = MCPConnectServerRequestPB()..serverId = serverId;
      final result = await AIEventGetMCPServerStatus(request).send();
      
      // 检查emit是否还有效
      if (emit.isDone) return;
      
      result.fold(
        (status) {
          Log.info('MCP服务器状态测试完成: ${status.serverId}');
          add(MCPSettingsEvent.didReceiveServerStatus(status));
        },
        (error) {
          Log.error('测试MCP服务器状态失败: $error');
          final testingServers = Set<String>.from(state.testingServers);
          testingServers.remove(serverId);
          emit(state.copyWith(
            testingServers: testingServers,
            error: '测试连接失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('测试MCP服务器连接异常: $e');
      if (!emit.isDone) {
        final testingServers = Set<String>.from(state.testingServers);
        testingServers.remove(serverId);
        emit(state.copyWith(
          testingServers: testingServers,
          error: '测试连接异常: $e',
        ));
      }
    }
  }

  /// 处理接收到服务器列表事件
  void _handleDidReceiveServerList(
    MCPServerListPB servers,
    Emitter<MCPSettingsState> emit,
  ) {
    Log.info('接收到MCP服务器列表，数量: ${servers.servers.length}');
    emit(state.copyWith(
      servers: servers.servers,
      isLoading: false,
      error: null,
    ));
  }

  /// 处理接收到服务器状态事件
  void _handleDidReceiveServerStatus(
    MCPServerStatusPB status,
    Emitter<MCPSettingsState> emit,
  ) {
    Log.info('接收到MCP服务器状态: ${status.serverId}, 连接: ${status.isConnected}');
    
    final updatedStatuses = Map<String, MCPServerStatusPB>.from(state.serverStatuses);
    updatedStatuses[status.serverId] = status;
    
    final connectingServers = Set<String>.from(state.connectingServers);
    connectingServers.remove(status.serverId);
    
    final testingServers = Set<String>.from(state.testingServers);
    testingServers.remove(status.serverId);
    
    emit(state.copyWith(
      serverStatuses: updatedStatuses,
      connectingServers: connectingServers,
      testingServers: testingServers,
    ));
  }

  /// 处理接收到错误事件
  void _handleDidReceiveError(
    String error,
    Emitter<MCPSettingsState> emit,
  ) {
    Log.error('MCP设置错误: $error');
    emit(state.copyWith(
      isLoading: false,
      isOperating: false,
      connectingServers: {},
      testingServers: {},
      error: error,
    ));
  }

  /// 加载服务器列表并直接emit（避免嵌套事件）
  Future<void> _loadServerListAndEmit(Emitter<MCPSettingsState> emit) async {
    try {
      Log.info('开始加载MCP服务器列表...');
      final result = await AIEventGetMCPServerList().send();
      
      Log.info('MCP服务器列表请求完成，检查emit状态: isDone=${emit.isDone}');
      
      // 检查emit是否还有效
      if (emit.isDone) {
        Log.warn('emit已完成，无法更新状态');
        return;
      }
      
      result.fold(
        (servers) {
          Log.info('接收到MCP服务器列表，数量: ${servers.servers.length}');
          emit(state.copyWith(
            servers: servers.servers,
            isLoading: false,
            isOperating: false,
            error: null,
          ));
        },
        (error) {
          Log.error('加载MCP服务器列表失败: $error');
          emit(state.copyWith(
            isLoading: false,
            isOperating: false,
            error: '加载服务器列表失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('加载MCP服务器列表异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isLoading: false,
          isOperating: false,
          error: '加载服务器列表异常: $e',
        ));
      }
    }
  }

  /// 获取服务器连接状态
  bool isServerConnected(String serverId) {
    return state.serverStatuses[serverId]?.isConnected ?? false;
  }

  /// 获取服务器是否正在连接
  bool isServerConnecting(String serverId) {
    return state.connectingServers.contains(serverId);
  }

  /// 获取服务器是否正在测试
  bool isServerTesting(String serverId) {
    return state.testingServers.contains(serverId);
  }

  /// 获取服务器工具数量
  int getServerToolCount(String serverId) {
    return state.serverStatuses[serverId]?.toolCount ?? 0;
  }

  /// 获取服务器错误信息
  String? getServerError(String serverId) {
    return state.serverStatuses[serverId]?.errorMessage;
  }

  /// 处理加载工具列表事件
  Future<void> _handleLoadToolList(
    String serverId,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(
      loadingTools: {...state.loadingTools, serverId},
      error: null,
    ));

    try {
      final request = MCPConnectServerRequestPB()..serverId = serverId;
      final result = await AIEventGetMCPToolList(request).send();
      
      // 检查emit是否还有效
      if (emit.isDone) return;
      
      result.fold(
        (toolList) {
          Log.info('获取到MCP工具列表: ${toolList.tools.length} 个工具');
          add(MCPSettingsEvent.didReceiveToolList(serverId, toolList));
        },
        (error) {
          Log.error('获取MCP工具列表失败: $error');
          final loadingTools = Set<String>.from(state.loadingTools);
          loadingTools.remove(serverId);
          emit(state.copyWith(
            loadingTools: loadingTools,
            error: '获取工具列表失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('获取MCP工具列表异常: $e');
      if (!emit.isDone) {
        final loadingTools = Set<String>.from(state.loadingTools);
        loadingTools.remove(serverId);
        emit(state.copyWith(
          loadingTools: loadingTools,
          error: '获取工具列表异常: $e',
        ));
      }
    }
  }

  /// 处理调用工具事件
  Future<void> _handleCallTool(
    String serverId,
    String toolName,
    String arguments,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(isCallingTool: true, error: null, lastToolResponse: null));

    try {
      final request = MCPToolCallRequestPB()
        ..serverId = serverId
        ..toolName = toolName
        ..arguments = arguments;
      
      final result = await AIEventCallMCPTool(request).send();
      
      // 检查emit是否还有效
      if (emit.isDone) return;
      
      result.fold(
        (response) {
          Log.info('MCP工具调用成功: $toolName');
          add(MCPSettingsEvent.didReceiveToolCallResponse(response));
        },
        (error) {
          Log.error('MCP工具调用失败: $error');
          emit(state.copyWith(
            isCallingTool: false,
            error: '调用工具失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('MCP工具调用异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isCallingTool: false,
          error: '调用工具异常: $e',
        ));
      }
    }
  }

  /// 处理刷新工具列表事件
  Future<void> _handleRefreshTools(
    String serverId,
    Emitter<MCPSettingsState> emit,
  ) async {
    // 清除缓存的工具列表
    final serverTools = Map<String, List<MCPToolPB>>.from(state.serverTools);
    serverTools.remove(serverId);
    emit(state.copyWith(serverTools: serverTools));
    
    // 重新加载
    await _handleLoadToolList(serverId, emit);
  }

  /// 处理接收到工具列表
  void _handleDidReceiveToolList(
    String serverId,
    MCPToolListPB toolList,
    Emitter<MCPSettingsState> emit,
  ) {
    Log.info('处理接收到的工具列表: $serverId, ${toolList.tools.length} 个工具');
    
    final serverTools = Map<String, List<MCPToolPB>>.from(state.serverTools);
    serverTools[serverId] = toolList.tools;
    
    final loadingTools = Set<String>.from(state.loadingTools);
    loadingTools.remove(serverId);
    
    emit(state.copyWith(
      serverTools: serverTools,
      loadingTools: loadingTools,
    ));
  }

  /// 处理接收到工具调用响应
  void _handleDidReceiveToolCallResponse(
    MCPToolCallResponsePB response,
    Emitter<MCPSettingsState> emit,
  ) {
    emit(state.copyWith(
      isCallingTool: false,
      lastToolResponse: response,
    ));
  }
}

/// MCP设置事件定义
@freezed
class MCPSettingsEvent with _$MCPSettingsEvent {
  /// 开始初始化
  const factory MCPSettingsEvent.started() = _Started;

  /// 加载服务器列表
  const factory MCPSettingsEvent.loadServerList() = _LoadServerList;

  /// 添加服务器
  const factory MCPSettingsEvent.addServer(MCPServerConfigPB config) = _AddServer;

  /// 更新服务器
  const factory MCPSettingsEvent.updateServer(MCPServerConfigPB config) = _UpdateServer;

  /// 删除服务器
  const factory MCPSettingsEvent.removeServer(String serverId) = _RemoveServer;

  /// 连接服务器
  const factory MCPSettingsEvent.connectServer(String serverId) = _ConnectServer;

  /// 断开服务器连接
  const factory MCPSettingsEvent.disconnectServer(String serverId) = _DisconnectServer;

  /// 测试连接
  const factory MCPSettingsEvent.testConnection(String serverId) = _TestConnection;

  /// 加载工具列表
  const factory MCPSettingsEvent.loadToolList(String serverId) = _LoadToolList;

  /// 调用工具
  const factory MCPSettingsEvent.callTool(String serverId, String toolName, String arguments) = _CallTool;

  /// 刷新工具列表
  const factory MCPSettingsEvent.refreshTools(String serverId) = _RefreshTools;

  /// 接收到服务器列表
  const factory MCPSettingsEvent.didReceiveServerList(MCPServerListPB servers) = _DidReceiveServerList;

  /// 接收到服务器状态
  const factory MCPSettingsEvent.didReceiveServerStatus(MCPServerStatusPB status) = _DidReceiveServerStatus;

  /// 接收到工具列表
  const factory MCPSettingsEvent.didReceiveToolList(String serverId, MCPToolListPB tools) = _DidReceiveToolList;

  /// 接收到工具调用结果
  const factory MCPSettingsEvent.didReceiveToolCallResponse(MCPToolCallResponsePB response) = _DidReceiveToolCallResponse;

  /// 接收到错误
  const factory MCPSettingsEvent.didReceiveError(String error) = _DidReceiveError;
}

/// MCP设置状态定义
@freezed
class MCPSettingsState with _$MCPSettingsState {
  const factory MCPSettingsState({
    /// 服务器列表
    @Default([]) List<MCPServerConfigPB> servers,
    
    /// 服务器状态映射
    @Default({}) Map<String, MCPServerStatusPB> serverStatuses,
    
    /// 服务器工具映射 (serverId -> 工具列表)
    @Default({}) Map<String, List<MCPToolPB>> serverTools,
    
    /// 正在加载工具的服务器ID集合
    @Default({}) Set<String> loadingTools,
    
    /// 正在连接的服务器ID集合
    @Default({}) Set<String> connectingServers,
    
    /// 正在测试的服务器ID集合
    @Default({}) Set<String> testingServers,
    
    /// 是否正在加载
    @Default(false) bool isLoading,
    
    /// 是否正在执行操作（添加、更新、删除）
    @Default(false) bool isOperating,
    
    /// 是否正在调用工具
    @Default(false) bool isCallingTool,
    
    /// 最后的工具调用响应
    MCPToolCallResponsePB? lastToolResponse,
    
    /// 选中的服务器ID（用于查看工具）
    String? selectedServerId,
    
    /// 错误信息
    String? error,
  }) = _MCPSettingsState;
}
