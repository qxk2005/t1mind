import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:appflowy/plugins/ai_chat/application/mcp_settings_bloc.dart';
import 'package:appflowy/shared/af_role_pb_extension.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_category.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:http/http.dart' as http;

/// 工作空间级别的MCP配置组件（V2 - 使用BLoC和后端持久化）
class WorkspaceMCPSettingsV2 extends StatelessWidget {
  const WorkspaceMCPSettingsV2({
    super.key,
    required this.userProfile,
    required this.workspaceId,
    required this.currentWorkspaceMemberRole,
  });

  final UserProfilePB userProfile;
  final String workspaceId;
  final AFRolePB? currentWorkspaceMemberRole;

  @override
  Widget build(BuildContext context) {
    // 检查用户权限
    final canConfigureMCP = currentWorkspaceMemberRole?.isOwner == true ||
        currentWorkspaceMemberRole == AFRolePB.Member;

    if (!canConfigureMCP) {
      return _buildNoPermissionView(context);
    }

    return BlocProvider(
      create: (_) => MCPSettingsBloc()..add(const MCPSettingsEvent.started()),
      child: SettingsCategory(
        title: "MCP 配置",
        description: "管理工作空间级别的 MCP 服务器配置和权限",
        children: [
          _WorkspaceMCPServerListV2(
            workspaceId: workspaceId,
            userRole: currentWorkspaceMemberRole!,
          ),
        ],
      ),
    );
  }

  Widget _buildNoPermissionView(BuildContext context) {
    return SettingsCategory(
      title: "MCP 配置",
      description: "您没有权限配置MCP服务器",
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Icon(Icons.lock, color: Theme.of(context).colorScheme.error),
              const SizedBox(width: 12),
              Expanded(
                child: FlowyText.regular(
                  "只有工作空间所有者和成员可以配置MCP服务器",
                  color: Theme.of(context).colorScheme.error,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

/// MCP服务器列表（使用BLoC）
class _WorkspaceMCPServerListV2 extends StatelessWidget {
  const _WorkspaceMCPServerListV2({
    required this.workspaceId,
    required this.userRole,
  });

  final String workspaceId;
  final AFRolePB userRole;

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<MCPSettingsBloc, MCPSettingsState>(
      listener: (context, state) {
        // 显示错误信息
        if (state.error != null) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(state.error!),
              backgroundColor: Colors.red,
              duration: const Duration(seconds: 3),
            ),
          );
        }
      },
      builder: (context, state) {
        if (state.isLoading && state.servers.isEmpty) {
          return const Center(
            child: Padding(
              padding: EdgeInsets.all(32.0),
              child: CircularProgressIndicator(),
            ),
          );
        }

        if (state.servers.isEmpty) {
          return _buildEmptyState(context);
        }

        return _buildServerList(context, state);
      },
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const VSpace(32),
        Icon(
          Icons.dns_outlined,
          size: 64,
          color: Theme.of(context).colorScheme.outline,
        ),
        const VSpace(16),
        FlowyText.medium(
          "暂无MCP服务器",
          fontSize: 16,
          color: Theme.of(context).colorScheme.outline,
        ),
        const VSpace(8),
        FlowyText.regular(
          "点击下方按钮添加您的第一个MCP服务器",
          color: Theme.of(context).colorScheme.outline,
        ),
        const VSpace(24),
        if (userRole.isOwner || userRole == AFRolePB.Member) ...[
          ElevatedButton.icon(
            onPressed: () => _showAddServerDialog(context),
            icon: const Icon(Icons.add),
            label: const Text('添加 MCP 服务器'),
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
            ),
          ),
        ] else ...[
          FlowyText.regular(
            "您没有权限添加服务器",
            color: Theme.of(context).colorScheme.error,
          ),
        ],
        const VSpace(32),
      ],
    );
  }

  Widget _buildServerList(BuildContext context, MCPSettingsState state) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            FlowyText.medium("MCP 服务器列表", fontSize: 16),
            const Spacer(),
            // 一键检查所有服务器按钮
            OutlinedButton.icon(
              onPressed: () => _checkAllServers(context, state),
              icon: const Icon(Icons.search, size: 18),
              label: const Text('一键检查'),
              style: OutlinedButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              ),
            ),
            const SizedBox(width: 8),
            if (userRole.isOwner || userRole == AFRolePB.Member) ...[
              ElevatedButton.icon(
                onPressed: () => _showAddServerDialog(context),
                icon: const Icon(Icons.add, size: 18),
                label: const Text('添加服务器'),
                style: ElevatedButton.styleFrom(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                ),
              ),
            ],
          ],
        ),
        const VSpace(16),
        ...state.servers.map((server) {
          // 优先使用实时获取的工具，否则使用缓存
          final realTimeTools = state.serverTools[server.id];
          final cachedTools = server.hasCachedTools() ? server.cachedTools.tools : <MCPToolPB>[];
          final tools = realTimeTools ?? cachedTools;
          final loadingTools = state.loadingTools.contains(server.id);
          
          // 🔍 调试日志
          print('🔍 [UI Debug] Server: ${server.name} (id: ${server.id})');
          print('  hasCachedTools: ${server.hasCachedTools()}');
          if (server.hasCachedTools()) {
            print('  cachedTools.tools.length: ${server.cachedTools.tools.length}');
            for (var i = 0; i < server.cachedTools.tools.length && i < 3; i++) {
              print('    - Tool ${i + 1}: ${server.cachedTools.tools[i].name}');
            }
          }
          print('  hasLastToolsCheckAt: ${server.hasLastToolsCheckAt()}');
          if (server.hasLastToolsCheckAt()) {
            print('  lastToolsCheckAt: ${server.lastToolsCheckAt}');
          }
          print('  realTimeTools: ${realTimeTools?.length ?? 0}');
          print('  cachedTools: ${cachedTools.length}');
          print('  final tools: ${tools.length}');
          print('  isConnected: ${state.serverStatuses[server.id]?.isConnected ?? false}');

          return _ServerCard(
            server: server,
            serverStatus: state.serverStatuses[server.id],
            tools: tools,
            loadingTools: loadingTools,
            onDelete: () {
              _showDeleteConfirmation(context, server);
            },
            onEdit: () {
              _showEditServerDialog(context, server);
            },
            onConnect: () {
              context.read<MCPSettingsBloc>().add(
                    MCPSettingsEvent.connectServer(server.id),
                  );
            },
            onDisconnect: () {
              context.read<MCPSettingsBloc>().add(
                    MCPSettingsEvent.disconnectServer(server.id),
                  );
            },
            onViewTools: () {
              _showToolListDialog(context, server.name, tools);
            },
            onRefreshTools: () {
              context.read<MCPSettingsBloc>().add(
                    MCPSettingsEvent.refreshTools(server.id),
                  );
            },
          );
        }),
      ],
    );
  }

  /// 一键检查所有服务器
  void _checkAllServers(BuildContext context, MCPSettingsState state) {
    final bloc = context.read<MCPSettingsBloc>();
    
    // 连接所有未连接的服务器并加载工具
    int checkCount = 0;
    for (final server in state.servers) {
      final isConnected = state.serverStatuses[server.id]?.isConnected ?? false;
      final hasTools = state.serverTools[server.id]?.isNotEmpty ?? false;
      
      if (!isConnected) {
        // 先连接服务器
        bloc.add(MCPSettingsEvent.connectServer(server.id));
        // 连接成功后会自动加载工具（在bloc中已实现）
        checkCount++;
      } else if (!hasTools && !state.loadingTools.contains(server.id)) {
        // 已连接但没有工具数据，重新加载
        bloc.add(MCPSettingsEvent.refreshTools(server.id));
        checkCount++;
      }
    }
    
    // 显示提示
    if (checkCount > 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('正在检查 $checkCount 个服务器的工具...'),
          duration: const Duration(seconds: 2),
        ),
      );
      
      // 延迟后重新加载服务器列表以获取更新的缓存数据
      Future.delayed(const Duration(seconds: 3), () {
        if (context.mounted) {
          bloc.add(const MCPSettingsEvent.loadServerList());
        }
      });
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('所有服务器已连接且工具已加载'),
          duration: Duration(seconds: 1),
        ),
      );
    }
  }

  void _showAddServerDialog(BuildContext context) {
    showDialog(
      context: context,
      barrierDismissible: true,
      builder: (dialogContext) => Center(
        child: Container(
          width: 700,
          height: 600,
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(16),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withOpacity(0.3),
                blurRadius: 10,
                offset: const Offset(0, 5),
              ),
            ],
          ),
          child: BlocProvider.value(
            value: context.read<MCPSettingsBloc>(),
            child: const _AddMCPServerDialog(),
          ),
        ),
      ),
    );
  }

  void _showEditServerDialog(BuildContext context, MCPServerConfigPB server) {
    showDialog(
      context: context,
      barrierDismissible: true,
      builder: (dialogContext) => Center(
        child: Container(
          width: 700,
          height: 600,
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(16),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withOpacity(0.3),
                blurRadius: 10,
                offset: const Offset(0, 5),
              ),
            ],
          ),
          child: BlocProvider.value(
            value: context.read<MCPSettingsBloc>(),
            child: _AddMCPServerDialog(existingServer: server),
          ),
        ),
      ),
    );
  }

  void _showToolListDialog(
      BuildContext context, String serverName, List<MCPToolPB> tools) {
    showDialog(
      context: context,
      builder: (context) => _MCPToolListDialog(
        serverName: serverName,
        tools: tools,
      ),
    );
  }

  void _showDeleteConfirmation(BuildContext context, MCPServerConfigPB server) {
    showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('确认删除'),
        content: Text('确定要删除服务器 "${server.name}" 吗？此操作无法撤销。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('取消'),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.red,
              foregroundColor: Colors.white,
            ),
            onPressed: () {
              Navigator.of(dialogContext).pop();
              context.read<MCPSettingsBloc>().add(
                    MCPSettingsEvent.removeServer(server.id),
                  );
            },
            child: const Text('删除'),
          ),
        ],
      ),
    );
  }
}

/// 添加/编辑MCP服务器对话框
class _AddMCPServerDialog extends StatefulWidget {
  const _AddMCPServerDialog({this.existingServer});

  final MCPServerConfigPB? existingServer;

  @override
  State<_AddMCPServerDialog> createState() => _AddMCPServerDialogState();
}

class _AddMCPServerDialogState extends State<_AddMCPServerDialog> {
  final _nameController = TextEditingController();
  final _urlController = TextEditingController();
  final _commandController = TextEditingController();
  final _descriptionController = TextEditingController();

  String _transportType = 'STDIO';
  List<Map<String, String>> _arguments = [];
  List<Map<String, String>> _environmentVariables = [];
  List<Map<String, String>> _httpHeaders = [];
  bool _isTestingConnection = false;
  String? _testResult;

  @override
  void initState() {
    super.initState();
    
    // 如果是编辑模式，预填充数据
    if (widget.existingServer != null) {
      final server = widget.existingServer!;
      _nameController.text = server.name;
      _descriptionController.text = server.description;
      _transportType = _transportTypeToString(server.transportType);
      
      if (server.hasStdioConfig()) {
        _commandController.text = server.stdioConfig.command;
        _arguments = server.stdioConfig.args.map((arg) => {'value': arg}).toList();
        _environmentVariables = server.stdioConfig.envVars.entries
            .map((e) => {'key': e.key, 'value': e.value})
            .toList();
      } else if (server.hasHttpConfig()) {
        _urlController.text = server.httpConfig.url;
        _httpHeaders = server.httpConfig.headers.entries
            .map((e) => {'key': e.key, 'value': e.value})
            .toList();
      }
    }
  }

  String _transportTypeToString(MCPTransportTypePB type) {
    switch (type) {
      case MCPTransportTypePB.Stdio:
        return 'STDIO';
      case MCPTransportTypePB.SSE:
        return 'SSE';
      case MCPTransportTypePB.HTTP:
        return 'HTTP';
      default:
        return 'STDIO';
    }
  }

  @override
  void dispose() {
    _nameController.dispose();
    _urlController.dispose();
    _commandController.dispose();
    _descriptionController.dispose();
    super.dispose();
  }

  MCPServerConfigPB _buildServerConfig() {
    // 如果是编辑模式，使用现有ID，否则生成新ID
    final id = widget.existingServer?.id ?? 'mcp_${DateTime.now().millisecondsSinceEpoch}';

    final config = MCPServerConfigPB()
      ..id = id
      ..name = _nameController.text.trim()
      ..icon = ''
      ..transportType = _convertTransportType(_transportType)
      ..isActive = true
      ..description = _descriptionController.text.trim();

    if (_transportType == 'STDIO') {
      config.stdioConfig = MCPStdioConfigPB()
        ..command = _commandController.text.trim()
        ..args.addAll(
            _arguments.map((a) => a['value'] ?? '').where((v) => v.isNotEmpty))
        ..envVars.addAll(Map.fromEntries(
          _environmentVariables
              .where((e) => (e['key'] ?? '').isNotEmpty)
              .map((e) => MapEntry(e['key']!, e['value'] ?? '')),
        ));
    } else {
      config.httpConfig = MCPHttpConfigPB()
        ..url = _urlController.text.trim()
        ..headers.addAll(Map.fromEntries(
          _httpHeaders
              .where((h) => (h['key'] ?? '').isNotEmpty)
              .map((h) => MapEntry(h['key']!, h['value'] ?? '')),
        ));
    }

    return config;
  }

  MCPTransportTypePB _convertTransportType(String type) {
    switch (type.toUpperCase()) {
      case 'STDIO':
        return MCPTransportTypePB.Stdio;
      case 'SSE':
        return MCPTransportTypePB.SSE;
      case 'HTTP':
        return MCPTransportTypePB.HTTP;
      default:
        return MCPTransportTypePB.Stdio;
    }
  }

  void _saveServer() {
    final config = _buildServerConfig();

    // 基本验证
    if (config.name.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('服务器名称不能为空')),
      );
      return;
    }

    if (config.transportType == MCPTransportTypePB.Stdio &&
        config.stdioConfig.command.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('STDIO命令不能为空')),
      );
      return;
    }

    if ((config.transportType == MCPTransportTypePB.HTTP ||
            config.transportType == MCPTransportTypePB.SSE) &&
        config.httpConfig.url.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('URL不能为空')),
      );
      return;
    }

    // 根据是否为编辑模式发送不同的事件
    if (widget.existingServer != null) {
      context.read<MCPSettingsBloc>().add(
            MCPSettingsEvent.updateServer(config),
          );
    } else {
      context.read<MCPSettingsBloc>().add(
            MCPSettingsEvent.addServer(config),
          );
    }

    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      body: Column(
        children: [
          // 标题栏
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
                ),
              ),
            ),
            child: Row(
              children: [
                FlowyText.medium(
                  widget.existingServer != null ? "编辑MCP服务器" : "添加MCP服务器", 
                  fontSize: 18,
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.close),
                  onPressed: () => Navigator.of(context).pop(),
                ),
              ],
            ),
          ),
          // 内容区域
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // 服务器名称
                  TextField(
                    controller: _nameController,
                    decoration: const InputDecoration(
                      labelText: '服务器名称 *',
                      hintText: '例如：我的MCP服务器',
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 16),
                  // 描述
                  TextField(
                    controller: _descriptionController,
                    decoration: const InputDecoration(
                      labelText: '描述（可选）',
                      hintText: '服务器描述',
                      border: OutlineInputBorder(),
                    ),
                    maxLines: 2,
                  ),
                  const SizedBox(height: 16),
                  // 传输类型
                  DropdownButtonFormField<String>(
                    value: _transportType,
                    decoration: const InputDecoration(
                      labelText: '传输类型',
                      border: OutlineInputBorder(),
                    ),
                    items: const [
                      DropdownMenuItem(value: 'STDIO', child: Text('STDIO')),
                      DropdownMenuItem(value: 'HTTP', child: Text('HTTP')),
                      DropdownMenuItem(value: 'SSE', child: Text('SSE')),
                    ],
                    onChanged: (value) {
                      if (value != null) {
                        setState(() => _transportType = value);
                      }
                    },
                  ),
                  const SizedBox(height: 16),
                  // 根据传输类型显示不同的配置
                  if (_transportType == 'STDIO') ...[
                    TextField(
                      controller: _commandController,
                      decoration: const InputDecoration(
                        labelText: '命令路径 *',
                        hintText: '例如：/usr/local/bin/mcp-server',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 16),
                    // 参数列表
                    Row(
                      children: [
                        FlowyText.regular("命令参数", fontSize: 14),
                        const SizedBox(width: 8),
                        Tooltip(
                          message: '每个参数单独添加，例如：第一个参数填"-y"，第二个参数填"@readwise/readwise-mcp"',
                          child: Icon(
                            Icons.info_outline,
                            size: 16,
                            color: Theme.of(context).hintColor,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 4),
                    FlowyText.regular(
                      "提示：每个参数需要单独添加，不要在一个框中输入多个参数",
                      fontSize: 11,
                      color: Theme.of(context).hintColor,
                    ),
                    const SizedBox(height: 8),
                    ..._arguments.asMap().entries.map((entry) {
                      final index = entry.key;
                      final argValue = entry.value['value'] ?? '';
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: Row(
                          children: [
                            Expanded(
                              child: TextField(
                                controller: TextEditingController(text: argValue),
                                decoration: InputDecoration(
                                    hintText: '单个参数，例如：-y 或 @readwise/readwise-mcp',
                                    helperText: '参数 ${index + 1}',
                                    helperStyle: TextStyle(fontSize: 10),
                                    border: OutlineInputBorder(),
                                    contentPadding: EdgeInsets.symmetric(
                                      horizontal: 12,
                                      vertical: 8,
                                    ),
                                  ),
                                onChanged: (value) {
                                  _arguments[index]['value'] = value;
                                },
                              ),
                            ),
                            IconButton(
                              icon: const Icon(Icons.remove_circle,
                                  color: Colors.red),
                              onPressed: () {
                                setState(() => _arguments.removeAt(index));
                              },
                            ),
                          ],
                        ),
                      );
                    }),
                    OutlinedButton.icon(
                      onPressed: () {
                        setState(() => _arguments.add({'value': ''}));
                      },
                      icon: const Icon(Icons.add),
                      label: const Text('添加参数'),
                    ),
                    const SizedBox(height: 16),
                    // 环境变量列表
                    Row(
                      children: [
                        FlowyText.regular("环境变量", fontSize: 14),
                        const SizedBox(width: 8),
                        Tooltip(
                          message: 'PATH会自动包含命令所在目录。如需自定义PATH，可手动添加PATH变量（Windows用;分隔，macOS/Linux用:分隔）',
                          child: Icon(
                            Icons.info_outline,
                            size: 16,
                            color: Theme.of(context).hintColor,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 4),
                    FlowyText.regular(
                      "提示：PATH会自动设置。如需自定义，可手动添加PATH变量；否则只需添加特殊变量（如ACCESS_TOKEN）",
                      fontSize: 11,
                      color: Theme.of(context).hintColor,
                    ),
                    const SizedBox(height: 8),
                    ..._environmentVariables.asMap().entries.map((entry) {
                      final index = entry.key;
                      final envKey = entry.value['key'] ?? '';
                      final envValue = entry.value['value'] ?? '';
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: Row(
                          children: [
                            Expanded(
                              child: TextField(
                                controller: TextEditingController(text: envKey),
                                decoration: const InputDecoration(
                                    hintText: '变量名',
                                    border: OutlineInputBorder(),
                                    contentPadding: EdgeInsets.symmetric(
                                      horizontal: 12,
                                      vertical: 8,
                                    ),
                                  ),
                                onChanged: (value) {
                                  _environmentVariables[index]['key'] = value;
                                },
                              ),
                            ),
                            const SizedBox(width: 8),
                            Expanded(
                              child: TextField(
                                controller: TextEditingController(text: envValue),
                                decoration: const InputDecoration(
                                    hintText: '变量值',
                                    border: OutlineInputBorder(),
                                    contentPadding: EdgeInsets.symmetric(
                                      horizontal: 12,
                                      vertical: 8,
                                    ),
                                  ),
                                onChanged: (value) {
                                  _environmentVariables[index]['value'] = value;
                                },
                              ),
                            ),
                            IconButton(
                              icon: const Icon(Icons.remove_circle,
                                  color: Colors.red),
                              onPressed: () {
                                setState(() =>
                                    _environmentVariables.removeAt(index));
                              },
                            ),
                          ],
                        ),
                      );
                    }),
                    OutlinedButton.icon(
                      onPressed: () {
                        setState(() => _environmentVariables
                            .add({'key': '', 'value': ''}));
                      },
                      icon: const Icon(Icons.add),
                      label: const Text('添加环境变量'),
                    ),
                  ] else ...[
                    TextField(
                      controller: _urlController,
                      decoration: const InputDecoration(
                        labelText: 'URL *',
                        hintText:
                            'http://localhost:3000 或 https://api.example.com/mcp',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 16),
                    // HTTP Headers列表
                    FlowyText.regular("HTTP Headers", fontSize: 14),
                    const SizedBox(height: 8),
                    ..._httpHeaders.asMap().entries.map((entry) {
                      final index = entry.key;
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: Row(
                          children: [
                            Expanded(
                              child: TextField(
                                decoration: const InputDecoration(
                                  hintText: 'Header名称',
                                  border: OutlineInputBorder(),
                                  contentPadding: EdgeInsets.symmetric(
                                    horizontal: 12,
                                    vertical: 8,
                                  ),
                                ),
                                onChanged: (value) {
                                  _httpHeaders[index]['key'] = value;
                                },
                              ),
                            ),
                            const SizedBox(width: 8),
                            Expanded(
                              child: TextField(
                                decoration: const InputDecoration(
                                  hintText: 'Header值',
                                  border: OutlineInputBorder(),
                                  contentPadding: EdgeInsets.symmetric(
                                    horizontal: 12,
                                    vertical: 8,
                                  ),
                                ),
                                onChanged: (value) {
                                  _httpHeaders[index]['value'] = value;
                                },
                              ),
                            ),
                            IconButton(
                              icon: const Icon(Icons.remove_circle,
                                  color: Colors.red),
                              onPressed: () {
                                setState(() => _httpHeaders.removeAt(index));
                              },
                            ),
                          ],
                        ),
                      );
                    }),
                    OutlinedButton.icon(
                      onPressed: () {
                        setState(
                            () => _httpHeaders.add({'key': '', 'value': ''}));
                      },
                      icon: const Icon(Icons.add),
                      label: const Text('添加Header'),
                    ),
                  ],
                ],
              ),
            ),
          ),
          // 测试结果显示
          if (_testResult != null)
            Container(
              margin: const EdgeInsets.symmetric(horizontal: 20),
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: _testResult!.contains('✅')
                    ? Colors.green.withOpacity(0.1)
                    : Colors.red.withOpacity(0.1),
                border: Border.all(
                  color: _testResult!.contains('✅') ? Colors.green : Colors.red,
                ),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Icon(
                    _testResult!.contains('✅')
                        ? Icons.check_circle
                        : Icons.error,
                    color:
                        _testResult!.contains('✅') ? Colors.green : Colors.red,
                    size: 20,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      _testResult!,
                      style: TextStyle(
                        color: _testResult!.contains('✅')
                            ? Colors.green
                            : Colors.red,
                        fontSize: 12,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          // 底部按钮
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              border: Border(
                top: BorderSide(
                  color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
                ),
              ),
            ),
            child: Column(
              children: [
                // 测试连接按钮
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton.icon(
                    onPressed: _isTestingConnection ? null : _testConnection,
                    icon: _isTestingConnection
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.wifi_tethering, size: 18),
                    label: Text(_isTestingConnection ? '测试中...' : '测试连接'),
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                // 取消和保存按钮
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    OutlinedButton(
                      onPressed: () => Navigator.of(context).pop(),
                      child: const Text('取消'),
                    ),
                    const SizedBox(width: 12),
                    ElevatedButton(
                      onPressed: _saveServer,
                      child: const Text('保存'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _testConnection() async {
    setState(() {
      _isTestingConnection = true;
      _testResult = null;
    });

    try {
      String result;
      switch (_transportType) {
        case 'STDIO':
          result = await _testStdioConnection();
          break;
        case 'HTTP':
          result = await _testHttpConnection();
          break;
        case 'SSE':
          result = await _testSseConnection();
          break;
        default:
          result = '❌ 不支持的传输类型';
      }
      setState(() {
        _isTestingConnection = false;
        _testResult = result;
      });
    } catch (e) {
      setState(() {
        _isTestingConnection = false;
        _testResult = '❌ 测试失败: ${e.toString()}';
      });
    }
  }

  Future<String> _testStdioConnection() async {
    final command = _commandController.text.trim();
    if (command.isEmpty) {
      throw Exception('命令路径不能为空');
    }

    try {
      ProcessResult result;
      if (Platform.isWindows) {
        result = await Process.run('where', [command], runInShell: true)
            .timeout(const Duration(seconds: 5));
      } else {
        result = await Process.run('which', [command])
            .timeout(const Duration(seconds: 5));
      }

      if (result.exitCode == 0) {
        final commandPath = result.stdout.toString().trim();
        return '✅ STDIO测试成功！命令路径: $commandPath';
      } else {
        throw Exception('命令未找到: $command');
      }
    } on TimeoutException {
      throw Exception('测试超时');
    } catch (e) {
      throw Exception('命令验证失败: ${e.toString()}');
    }
  }

  Future<String> _testHttpConnection() async {
    final url = _urlController.text.trim();
    if (url.isEmpty) {
      throw Exception('URL不能为空');
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      throw Exception('URL必须以http://或https://开头');
    }

    try {
      final uri = Uri.parse(url);

      // 发送 MCP initialize 请求来测试连接
      final mcpRequest = {
        'jsonrpc': '2.0',
        'id': 0,
        'method': 'initialize',
        'params': {
          'protocolVersion': '2024-11-05',
          'capabilities': {},
          'clientInfo': {
            'name': 'AppFlowy Test Client',
            'version': '1.0.0',
          },
        },
      };

      final response = await http
          .post(
            uri,
            headers: {
              'Content-Type': 'application/json',
              'Accept': 'application/json, text/event-stream',
            },
            body: json.encode(mcpRequest),
          )
          .timeout(const Duration(seconds: 10));

      if (response.statusCode == 200) {
        try {
          final jsonData = json.decode(response.body);
          if (jsonData is Map && jsonData.containsKey('jsonrpc')) {
            return '✅ HTTP连接成功！检测到MCP服务器 (JSON-RPC 2.0)';
          }
        } catch (_) {}
        return '✅ HTTP连接成功！状态码: ${response.statusCode}';
      } else if (response.statusCode == 400) {
        // 400 可能是因为缺少某些参数，但服务器在运行
        return '⚠️ 服务器响应400，但服务器正在运行。请检查MCP服务器配置。';
      } else if (response.statusCode == 406) {
        // 406 Not Acceptable - 可能需要SSE
        return '⚠️ 服务器返回406，可能需要使用SSE传输类型';
      } else {
        throw Exception('HTTP错误: 状态码 ${response.statusCode}');
      }
    } on TimeoutException {
      throw Exception('连接超时');
    } on SocketException {
      throw Exception('无法连接到服务器');
    } catch (e) {
      throw Exception('连接失败: ${e.toString()}');
    }
  }

  Future<String> _testSseConnection() async {
    final url = _urlController.text.trim();
    if (url.isEmpty) {
      throw Exception('URL不能为空');
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      throw Exception('URL必须以http://或https://开头');
    }

    try {
      final uri = Uri.parse(url);

      // 发送 MCP initialize 请求（SSE服务器也接受POST请求）
      final mcpRequest = {
        'jsonrpc': '2.0',
        'id': 0,
        'method': 'initialize',
        'params': {
          'protocolVersion': '2024-11-05',
          'capabilities': {},
          'clientInfo': {
            'name': 'AppFlowy Test Client',
            'version': '1.0.0',
          },
        },
      };

      final response = await http
          .post(
            uri,
            headers: {
              'Content-Type': 'application/json',
              'Accept': 'application/json, text/event-stream',
            },
            body: json.encode(mcpRequest),
          )
          .timeout(const Duration(seconds: 10));

      if (response.statusCode == 200) {
        final contentType = response.headers['content-type'] ?? '';

        // 检查是否是SSE响应
        if (contentType.contains('text/event-stream')) {
          // 尝试解析SSE响应
          final body = response.body;
          if (body.contains('event:') || body.contains('data:')) {
            return '✅ SSE连接成功！服务器支持Server-Sent Events (MCP)';
          }
          return '✅ SSE连接成功！Content-Type: text/event-stream';
        } else if (contentType.contains('application/json')) {
          // 有些MCP服务器可能先返回JSON，然后切换到SSE
          try {
            final jsonData = json.decode(response.body);
            if (jsonData is Map && jsonData.containsKey('jsonrpc')) {
              return '✅ 连接成功！检测到MCP服务器 (可能支持SSE)';
            }
          } catch (_) {}
          return '✅ 连接成功！状态码: ${response.statusCode}';
        } else {
          return '⚠️ 服务器响应成功，但Content-Type不是text/event-stream (实际: $contentType)';
        }
      } else if (response.statusCode == 400) {
        return '⚠️ 服务器响应400，但服务器正在运行。请检查MCP服务器配置。';
      } else if (response.statusCode == 406) {
        return '⚠️ 服务器返回406，请尝试使用HTTP传输类型';
      } else {
        throw Exception('SSE错误: 状态码 ${response.statusCode}');
      }
    } on TimeoutException {
      throw Exception('连接超时');
    } on SocketException {
      throw Exception('无法连接到服务器');
    } catch (e) {
      throw Exception('连接失败: ${e.toString()}');
    }
  }
}

/// 服务器卡片
class _ServerCard extends StatelessWidget {
  const _ServerCard({
    required this.server,
    this.serverStatus,
    required this.tools,
    required this.loadingTools,
    required this.onDelete,
    required this.onEdit,
    required this.onConnect,
    required this.onDisconnect,
    required this.onViewTools,
    required this.onRefreshTools,
  });

  final MCPServerConfigPB server;
  final MCPServerStatusPB? serverStatus;
  final List<MCPToolPB> tools;
  final bool loadingTools;
  final VoidCallback onDelete;
  final VoidCallback onEdit;
  final VoidCallback onConnect;
  final VoidCallback onDisconnect;
  final VoidCallback onViewTools;
  final VoidCallback onRefreshTools;

  @override
  Widget build(BuildContext context) {
    final isConnected = serverStatus?.isConnected ?? false;
    final errorMessage = serverStatus?.errorMessage;

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Row(
                  children: [
                    Flexible(
                      child: FlowyText.medium(server.name, fontSize: 16),
                    ),
                    const SizedBox(width: 8),
                    // 连接状态徽章 - 更显眼的状态显示
                    _buildConnectionStatusBadge(context, isConnected, errorMessage),
                    const SizedBox(width: 8),
                    // 工具数量徽章 - 只要有工具就显示
                    if (tools.isNotEmpty) ...[
                      Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          // 根据连接状态显示不同颜色：已连接=蓝色，未连接=灰色
                          color: isConnected ? Colors.blue : Colors.grey,
                          borderRadius: BorderRadius.circular(12),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            const Icon(Icons.build,
                                size: 12, color: Colors.white),
                            const SizedBox(width: 4),
                            Text(
                              '${tools.length}',
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 11,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                    if (loadingTools) ...[
                      const SizedBox(width: 8),
                      const SizedBox(
                        width: 12,
                        height: 12,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      ),
                    ],
                  ],
                ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.primary,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: FlowyText.medium(
                  _transportTypeToString(server.transportType),
                  fontSize: 12,
                  color: Colors.white,
                ),
              ),
              const SizedBox(width: 8),
              // 查看工具按钮 - 只要有工具就显示
              if (tools.isNotEmpty)
                IconButton(
                  icon: const Icon(Icons.list_alt, size: 20),
                  onPressed: onViewTools,
                  tooltip: "查看工具 (${tools.length})",
                  color: Colors.blue,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                )
              else if (isConnected && !loadingTools)
                IconButton(
                  icon: const Icon(Icons.refresh, size: 20),
                  onPressed: onRefreshTools,
                  tooltip: "加载工具",
                  color: Colors.grey,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.edit, color: Colors.blue, size: 20),
                onPressed: onEdit,
                tooltip: "编辑服务器",
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(),
              ),
              IconButton(
                icon: const Icon(Icons.delete, color: Colors.red, size: 20),
                onPressed: onDelete,
                tooltip: "删除服务器",
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(),
              ),
            ],
          ),
          if (server.description.isNotEmpty) ...[
            const SizedBox(height: 8),
            FlowyText.regular(
              server.description,
              fontSize: 12,
              color: Theme.of(context).colorScheme.outline,
            ),
          ],
          if (server.hasStdioConfig()) ...[
            const SizedBox(height: 8),
            FlowyText.regular(
              "命令: ${server.stdioConfig.command}",
              fontSize: 12,
            ),
          ],
          if (server.hasHttpConfig()) ...[
            const SizedBox(height: 8),
            FlowyText.regular(
              "URL: ${server.httpConfig.url}",
              fontSize: 12,
            ),
          ],
          // 工具标签展示 - 只要有工具就显示，不管是否连接
          if (tools.isNotEmpty) ...[
            const SizedBox(height: 12),
            _buildToolTags(context, tools),
          ],
          // 最后检查时间
          if (server.hasLastToolsCheckAt()) ...[
            const SizedBox(height: 8),
            _buildLastCheckTime(context, server.lastToolsCheckAt.toInt()),
          ],
        ],
      ),
    );
  }

  /// 构建最后检查时间显示
  Widget _buildLastCheckTime(BuildContext context, int timestamp) {
    final checkTime = DateTime.fromMillisecondsSinceEpoch(timestamp * 1000);
    final now = DateTime.now();
    final difference = now.difference(checkTime);
    
    String timeText;
    if (difference.inMinutes < 1) {
      timeText = '刚刚';
    } else if (difference.inMinutes < 60) {
      timeText = '${difference.inMinutes}分钟前';
    } else if (difference.inHours < 24) {
      timeText = '${difference.inHours}小时前';
    } else if (difference.inDays < 7) {
      timeText = '${difference.inDays}天前';
    } else {
      timeText = '${checkTime.year}-${checkTime.month.toString().padLeft(2, '0')}-${checkTime.day.toString().padLeft(2, '0')} ${checkTime.hour.toString().padLeft(2, '0')}:${checkTime.minute.toString().padLeft(2, '0')}';
    }
    
    return Row(
      children: [
        Icon(
          Icons.schedule,
          size: 12,
          color: Theme.of(context).hintColor,
        ),
        const SizedBox(width: 4),
        FlowyText.regular(
          '最后检查: $timeText',
          fontSize: 11,
          color: Theme.of(context).hintColor,
        ),
      ],
    );
  }
  
  /// 构建工具标签
  Widget _buildToolTags(BuildContext context, List<MCPToolPB> tools) {
    // 最多显示5个工具标签
    final displayTools = tools.take(5).toList();
    final hasMore = tools.length > 5;
    
    return Wrap(
      spacing: 6,
      runSpacing: 6,
      children: [
        ...displayTools.map((tool) => _ToolTag(tool: tool)),
        if (hasMore)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceVariant,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(
                color: Theme.of(context).colorScheme.outline.withOpacity(0.5),
                width: 1.0,
              ),
            ),
            child: Text(
              '+${tools.length - 5}',
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w500,
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
      ],
    );
  }

  String _transportTypeToString(MCPTransportTypePB type) {
    switch (type) {
      case MCPTransportTypePB.Stdio:
        return 'STDIO';
      case MCPTransportTypePB.SSE:
        return 'SSE';
      case MCPTransportTypePB.HTTP:
        return 'HTTP';
      default:
        return 'UNKNOWN';
    }
  }

  /// 构建连接状态徽章
  Widget _buildConnectionStatusBadge(
    BuildContext context,
    bool isConnected,
    String? errorMessage,
  ) {
    final Color bgColor;
    final Color textColor;
    final IconData icon;
    final String statusText;

    if (errorMessage != null && errorMessage.isNotEmpty) {
      // 连接错误状态
      bgColor = Colors.red.shade50;
      textColor = Colors.red.shade700;
      icon = Icons.error_outline;
      statusText = '错误';
    } else if (isConnected) {
      // 已连接状态
      bgColor = Colors.green.shade50;
      textColor = Colors.green.shade700;
      icon = Icons.check_circle_outline;
      statusText = '已连接';
    } else {
      // 未连接状态
      bgColor = Colors.grey.shade200;
      textColor = Colors.grey.shade700;
      icon = Icons.radio_button_unchecked;
      statusText = '未连接';
    }

    return Tooltip(
      message: errorMessage ?? statusText,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: bgColor,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: textColor.withOpacity(0.3),
            width: 1.0,
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 12, color: textColor),
            const SizedBox(width: 4),
            Text(
              statusText,
              style: TextStyle(
                color: textColor,
                fontSize: 11,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// MCP工具列表对话框
class _MCPToolListDialog extends StatelessWidget {
  const _MCPToolListDialog({
    required this.serverName,
    required this.tools,
  });

  final String serverName;
  final List<MCPToolPB> tools;

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: Container(
        width: 700,
        height: 600,
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surface,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          children: [
            // 标题栏
            Container(
              padding: const EdgeInsets.all(20),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(
                    color:
                        Theme.of(context).colorScheme.outline.withOpacity(0.2),
                  ),
                ),
              ),
              child: Row(
                children: [
                  const Icon(Icons.build_circle, size: 24),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        FlowyText.medium(
                          '$serverName - MCP 工具',
                          fontSize: 18,
                        ),
                        const SizedBox(height: 4),
                        FlowyText.regular(
                          '共 ${tools.length} 个工具',
                          fontSize: 12,
                          color: Theme.of(context).colorScheme.outline,
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
            ),
            // 工具列表
            Expanded(
              child: tools.isEmpty
                  ? Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            Icons.build_outlined,
                            size: 64,
                            color: Theme.of(context).colorScheme.outline,
                          ),
                          const SizedBox(height: 16),
                          FlowyText.regular(
                            '暂无工具',
                            color: Theme.of(context).colorScheme.outline,
                          ),
                        ],
                      ),
                    )
                  : ListView.separated(
                      padding: const EdgeInsets.all(20),
                      itemCount: tools.length,
                      separatorBuilder: (context, index) =>
                          const Divider(height: 1),
                      itemBuilder: (context, index) {
                        final tool = tools[index];
                        return _ToolCard(tool: tool);
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }
}

/// 工具卡片
class _ToolCard extends StatefulWidget {
  const _ToolCard({required this.tool});

  final MCPToolPB tool;

  @override
  State<_ToolCard> createState() => _ToolCardState();
}

class _ToolCardState extends State<_ToolCard> {
  bool _isExpanded = false;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InkWell(
            onTap: () => setState(() => _isExpanded = !_isExpanded),
            borderRadius: BorderRadius.circular(8),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Icon(
                              Icons.functions,
                              size: 16,
                              color: Theme.of(context).colorScheme.primary,
                            ),
                            const SizedBox(width: 8),
                            Flexible(
                              child: FlowyText.medium(
                                widget.tool.name,
                                fontSize: 15,
                              ),
                            ),
                            // 安全标签
                            if (widget.tool.hasAnnotations()) ...[
                              const SizedBox(width: 8),
                              _buildSafetyBadge(context),
                            ],
                          ],
                        ),
                        const SizedBox(height: 8),
                        FlowyText.regular(
                          widget.tool.description,
                          fontSize: 13,
                          color: Theme.of(context).colorScheme.outline,
                          maxLines: _isExpanded ? null : 2,
                        ),
                      ],
                    ),
                  ),
                  Icon(
                    _isExpanded ? Icons.expand_less : Icons.expand_more,
                    color: Theme.of(context).colorScheme.outline,
                  ),
                ],
              ),
            ),
          ),
          if (_isExpanded) ...[
            const Divider(height: 1),
            Container(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  FlowyText.medium('输入参数', fontSize: 13),
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Theme.of(context)
                          .colorScheme
                          .surfaceVariant
                          .withOpacity(0.3),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: SelectableText(
                        _formatJsonSchema(widget.tool.inputSchema),
                        style: TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 12,
                          color: Theme.of(context).colorScheme.onSurface,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildSafetyBadge(BuildContext context) {
    if (!widget.tool.hasAnnotations()) return const SizedBox.shrink();

    final annotations = widget.tool.annotations;
    String label;
    Color color;

    if (annotations.destructiveHint == true) {
      label = '破坏性';
      color = Colors.red;
    } else if (annotations.openWorldHint == true) {
      label = '外部';
      color = Colors.orange;
    } else if (annotations.readOnlyHint == true) {
      label = '只读';
      color = Colors.green;
    } else {
      return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        border: Border.all(color: color.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  String _formatJsonSchema(String schema) {
    try {
      final decoded = json.decode(schema);
      const encoder = JsonEncoder.withIndent('  ');
      return encoder.convert(decoded);
    } catch (e) {
      return schema;
    }
  }
}

/// 工具标签组件 - 支持悬停显示描述
class _ToolTag extends StatefulWidget {
  const _ToolTag({required this.tool});

  final MCPToolPB tool;

  @override
  State<_ToolTag> createState() => _ToolTagState();
}

class _ToolTagState extends State<_ToolTag> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: Tooltip(
        message: widget.tool.description.isNotEmpty 
            ? '${widget.tool.name}\n\n${widget.tool.description}'
            : widget.tool.name,
        preferBelow: false,
        waitDuration: const Duration(milliseconds: 300),
        textStyle: const TextStyle(
          fontSize: 12,
          color: Colors.white,
        ),
        decoration: BoxDecoration(
          color: Colors.black87,
          borderRadius: BorderRadius.circular(6),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            // 改进：使用更深的背景色，提高对比度
            color: _isHovered 
                ? Theme.of(context).colorScheme.primary.withOpacity(0.15)
                : Theme.of(context).colorScheme.surfaceVariant,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: _isHovered
                  ? Theme.of(context).colorScheme.primary
                  : Theme.of(context).colorScheme.outline.withOpacity(0.5),
              width: _isHovered ? 1.5 : 1.0,
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.functions,
                size: 12,
                // 改进：使用深色图标，提高可读性
                color: _isHovered
                    ? Theme.of(context).colorScheme.primary
                    : Theme.of(context).colorScheme.onSurfaceVariant,
              ),
              const SizedBox(width: 4),
              Text(
                widget.tool.name,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: _isHovered ? FontWeight.w600 : FontWeight.w500,
                  // 改进：使用深色文字，提高可读性
                  color: _isHovered
                      ? Theme.of(context).colorScheme.primary
                      : Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
