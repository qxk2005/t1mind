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
    on<MCPSettingsEvent>((event, emit) async {
      event.when(
        started: () => _handleStarted(emit),
        loadServerList: () => _handleLoadServerList(emit),
        addServer: (config) => _handleAddServer(config, emit),
        updateServer: (config) => _handleUpdateServer(config, emit),
        removeServer: (serverId) => _handleRemoveServer(serverId, emit),
        connectServer: (serverId) => _handleConnectServer(serverId, emit),
        disconnectServer: (serverId) => _handleDisconnectServer(serverId, emit),
        testConnection: (serverId) => _handleTestConnection(serverId, emit),
        didReceiveServerList: (servers) => _handleDidReceiveServerList(servers, emit),
        didReceiveServerStatus: (status) => _handleDidReceiveServerStatus(status, emit),
        didReceiveError: (error) => _handleDidReceiveError(error, emit),
      );
    });
  }

  /// 处理初始化事件
  Future<void> _handleStarted(Emitter<MCPSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadServerList();
  }

  /// 处理加载服务器列表事件
  Future<void> _handleLoadServerList(Emitter<MCPSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadServerList();
  }

  /// 处理添加服务器事件
  Future<void> _handleAddServer(
    MCPServerConfigPB config,
    Emitter<MCPSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final result = await AIEventAddMCPServer(config).send();
      await result.fold(
        (success) async {
          Log.info('MCP服务器添加成功: ${config.name}');
          // 重新加载服务器列表
          await _loadServerList();
          emit(state.copyWith(isOperating: false));
        },
        (error) {
          Log.error('添加MCP服务器失败: $error');
          emit(state.copyWith(
            isOperating: false,
            error: '添加服务器失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('添加MCP服务器异常: $e');
      emit(state.copyWith(
        isOperating: false,
        error: '添加服务器异常: $e',
      ));
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
      await result.fold(
        (success) async {
          Log.info('MCP服务器更新成功: ${config.name}');
          // 重新加载服务器列表
          await _loadServerList();
          emit(state.copyWith(isOperating: false));
        },
        (error) {
          Log.error('更新MCP服务器失败: $error');
          emit(state.copyWith(
            isOperating: false,
            error: '更新服务器失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('更新MCP服务器异常: $e');
      emit(state.copyWith(
        isOperating: false,
        error: '更新服务器异常: $e',
      ));
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
      await result.fold(
        (success) async {
          Log.info('MCP服务器删除成功: $serverId');
          // 重新加载服务器列表
          await _loadServerList();
          emit(state.copyWith(isOperating: false));
        },
        (error) {
          Log.error('删除MCP服务器失败: $error');
          emit(state.copyWith(
            isOperating: false,
            error: '删除服务器失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('删除MCP服务器异常: $e');
      emit(state.copyWith(
        isOperating: false,
        error: '删除服务器异常: $e',
      ));
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
      result.fold(
        (status) {
          Log.info('MCP服务器连接结果: ${status.serverId}, 连接状态: ${status.isConnected}');
          add(MCPSettingsEvent.didReceiveServerStatus(status));
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
      final connectingServers = Set<String>.from(state.connectingServers);
      connectingServers.remove(serverId);
      emit(state.copyWith(
        connectingServers: connectingServers,
        error: '连接服务器异常: $e',
      ));
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
      await result.fold(
        (success) async {
          Log.info('MCP服务器断开连接成功: $serverId');
          // 更新服务器状态
          final updatedStatuses = Map<String, MCPServerStatusPB>.from(state.serverStatuses);
          updatedStatuses[serverId] = MCPServerStatusPB()
            ..serverId = serverId
            ..isConnected = false;
          
          final connectingServers = Set<String>.from(state.connectingServers);
          connectingServers.remove(serverId);
          
          emit(state.copyWith(
            serverStatuses: updatedStatuses,
            connectingServers: connectingServers,
          ));
        },
        (error) {
          Log.error('断开MCP服务器连接失败: $error');
          final connectingServers = Set<String>.from(state.connectingServers);
          connectingServers.remove(serverId);
          emit(state.copyWith(
            connectingServers: connectingServers,
            error: '断开连接失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('断开MCP服务器连接异常: $e');
      final connectingServers = Set<String>.from(state.connectingServers);
      connectingServers.remove(serverId);
      emit(state.copyWith(
        connectingServers: connectingServers,
        error: '断开连接异常: $e',
      ));
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
      final testingServers = Set<String>.from(state.testingServers);
      testingServers.remove(serverId);
      emit(state.copyWith(
        testingServers: testingServers,
        error: '测试连接异常: $e',
      ));
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

  /// 加载服务器列表的私有方法
  Future<void> _loadServerList() async {
    try {
      final result = await AIEventGetMCPServerList().send();
      result.fold(
        (servers) {
          if (!isClosed) {
            add(MCPSettingsEvent.didReceiveServerList(servers));
          }
        },
        (error) {
          Log.error('加载MCP服务器列表失败: $error');
          if (!isClosed) {
            add(MCPSettingsEvent.didReceiveError('加载服务器列表失败: ${error.msg}'));
          }
        },
      );
    } catch (e) {
      Log.error('加载MCP服务器列表异常: $e');
      if (!isClosed) {
        add(MCPSettingsEvent.didReceiveError('加载服务器列表异常: $e'));
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

  /// 接收到服务器列表
  const factory MCPSettingsEvent.didReceiveServerList(MCPServerListPB servers) = _DidReceiveServerList;

  /// 接收到服务器状态
  const factory MCPSettingsEvent.didReceiveServerStatus(MCPServerStatusPB status) = _DidReceiveServerStatus;

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
    
    /// 正在连接的服务器ID集合
    @Default({}) Set<String> connectingServers,
    
    /// 正在测试的服务器ID集合
    @Default({}) Set<String> testingServers,
    
    /// 是否正在加载
    @Default(false) bool isLoading,
    
    /// 是否正在执行操作（添加、更新、删除）
    @Default(false) bool isOperating,
    
    /// 错误信息
    String? error,
  }) = _MCPSettingsState;
}
