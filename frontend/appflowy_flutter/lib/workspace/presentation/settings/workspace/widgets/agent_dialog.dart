import 'package:appflowy/plugins/ai_chat/application/agent_settings_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/mcp_settings_bloc.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 智能体创建/编辑对话框
class AgentDialog extends StatefulWidget {
  const AgentDialog({super.key, this.existingAgent});

  final AgentConfigPB? existingAgent;

  @override
  State<AgentDialog> createState() => _AgentDialogState();
}

class _AgentDialogState extends State<AgentDialog> {
  late final TextEditingController _nameController;
  late final TextEditingController _descriptionController;
  late final TextEditingController _personalityController;
  late final TextEditingController _avatarController;
  late final TextEditingController _maxToolResultLengthController;
  late final TextEditingController _maxReflectionIterationsController;
  
  bool _enablePlanning = true;
  bool _enableToolCalling = true;
  bool _enableReflection = false;
  bool _enableMemory = true;
  bool _enableWebSearch = true; // 🆕 网络搜索工具开关
  
  // 🆕 选中的 MCP 服务器 ID 列表
  final Set<String> _selectedMCPServerIds = {};

  @override
  void initState() {
    super.initState();
    
    _nameController = TextEditingController(text: widget.existingAgent?.name ?? '');
    _descriptionController = TextEditingController(text: widget.existingAgent?.description ?? '');
    _personalityController = TextEditingController(text: widget.existingAgent?.personality ?? '');
    _avatarController = TextEditingController(text: widget.existingAgent?.avatar ?? '');
    
    // 初始化工具结果最大长度，默认 4000
    int defaultLength = 4000;
    int defaultReflectionIterations = 3;
    if (widget.existingAgent?.hasCapabilities() == true) {
      final cap = widget.existingAgent!.capabilities;
      _enablePlanning = cap.enablePlanning;
      _enableToolCalling = cap.enableToolCalling;
      _enableReflection = cap.enableReflection;
      _enableMemory = cap.enableMemory;
      if (cap.maxToolResultLength > 0) {
        defaultLength = cap.maxToolResultLength;
      }
      if (cap.maxReflectionIterations > 0) {
        defaultReflectionIterations = cap.maxReflectionIterations;
      }
    }
    
    // 🆕 检查现有智能体是否已启用网络搜索工具
    if (widget.existingAgent != null) {
      _enableWebSearch = widget.existingAgent!.availableTools.contains('web_search') || 
                        widget.existingAgent!.availableTools.contains('quick_search');
    }
    _maxToolResultLengthController = TextEditingController(text: defaultLength.toString());
    _maxReflectionIterationsController = TextEditingController(text: defaultReflectionIterations.toString());
    
    // 🆕 加载已选择的 MCP 服务器
    if (widget.existingAgent != null) {
      _selectedMCPServerIds.addAll(widget.existingAgent!.selectedMcpServers);
    }
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _personalityController.dispose();
    _avatarController.dispose();
    _maxToolResultLengthController.dispose();
    _maxReflectionIterationsController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isEditing = widget.existingAgent != null;
    
    return Dialog(
      child: Container(
        width: 600,
        constraints: const BoxConstraints(maxHeight: 700),
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                FlowyText.medium(isEditing ? "编辑智能体" : "创建智能体", fontSize: 20),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.close),
                  onPressed: () => Navigator.of(context).pop(),
                ),
              ],
            ),
            const VSpace(20),
            Flexible(
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    FlowyText.medium("基本信息", fontSize: 16),
                    const VSpace(12),
                    TextField(
                      controller: _nameController,
                      decoration: const InputDecoration(
                        labelText: '名称 *',
                        hintText: '例如：代码助手',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const VSpace(12),
                    TextField(
                      controller: _descriptionController,
                      decoration: const InputDecoration(
                        labelText: '描述',
                        hintText: '简要描述智能体的用途',
                        border: OutlineInputBorder(),
                      ),
                      maxLines: 2,
                    ),
                    const VSpace(12),
                    TextField(
                      controller: _avatarController,
                      decoration: const InputDecoration(
                        labelText: '头像 (Emoji)',
                        hintText: '🤖',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const VSpace(20),
                    FlowyText.medium("能力配置", fontSize: 16),
                    const VSpace(12),
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("任务规划", fontSize: 14)),
                        Switch(value: _enablePlanning, onChanged: (v) => setState(() => _enablePlanning = v)),
                      ],
                    ),
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("工具调用", fontSize: 14)),
                        Switch(value: _enableToolCalling, onChanged: (v) => setState(() => _enableToolCalling = v)),
                      ],
                    ),
                    // 工具结果最大长度配置（仅在启用工具调用时显示）
                    if (_enableToolCalling) ...[
                      const VSpace(12),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          FlowyText.regular(
                            "工具结果最大长度 (字符)",
                            fontSize: 13,
                            color: Theme.of(context).textTheme.bodySmall?.color,
                          ),
                          const VSpace(4),
                          TextField(
                            controller: _maxToolResultLengthController,
                            keyboardType: TextInputType.number,
                            decoration: InputDecoration(
                              hintText: '默认: 4000',
                              helperText: '推荐范围: 1000-16000，根据模型上下文调整',
                              helperMaxLines: 2,
                              border: const OutlineInputBorder(),
                              contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                            ),
                          ),
                          const VSpace(4),
                          BlocBuilder<AgentSettingsBloc, AgentSettingsState>(
                            builder: (context, state) {
                              final length = int.tryParse(_maxToolResultLengthController.text);
                              final recommendation = context.read<AgentSettingsBloc>().getMaxToolResultLengthRecommendation(length);
                              return FlowyText.regular(
                                recommendation,
                                fontSize: 11,
                                color: Theme.of(context).colorScheme.primary,
                              );
                            },
                          ),
                        ],
                      ),
                    ],
                    const VSpace(8),
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("反思机制", fontSize: 14)),
                        Switch(value: _enableReflection, onChanged: (v) => setState(() => _enableReflection = v)),
                      ],
                    ),
                    // 反思迭代次数配置（仅在启用反思时显示）
                    if (_enableReflection) ...[
                      const VSpace(12),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          FlowyText.regular(
                            "反思迭代次数",
                            fontSize: 13,
                            color: Theme.of(context).textTheme.bodySmall?.color,
                          ),
                          const VSpace(4),
                          TextField(
                            controller: _maxReflectionIterationsController,
                            keyboardType: TextInputType.number,
                            decoration: const InputDecoration(
                              hintText: '默认: 3',
                              helperText: '推荐范围: 1-10，每次迭代都会调用一次 AI',
                              helperMaxLines: 2,
                              border: OutlineInputBorder(),
                              contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                            ),
                          ),
                        ],
                      ),
                    ],
                    const VSpace(8),
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("会话记忆", fontSize: 14)),
                        Switch(value: _enableMemory, onChanged: (v) => setState(() => _enableMemory = v)),
                      ],
                    ),
                    const VSpace(8),
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("网络搜索", fontSize: 14)),
                        Switch(value: _enableWebSearch, onChanged: (v) => setState(() => _enableWebSearch = v)),
                      ],
                    ),
                    // 🆕 MCP 服务器选择（仅在启用工具调用时显示）
                    if (_enableToolCalling) ...[
                      const VSpace(20),
                      FlowyText.medium("选择 MCP 服务器", fontSize: 16),
                      const VSpace(8),
                      FlowyText.regular(
                        "勾选服务器后，将自动使用该服务器的所有工具",
                        fontSize: 12,
                        color: Theme.of(context).textTheme.bodySmall?.color,
                      ),
                      const VSpace(12),
                      _buildMCPServerSelector(),
                    ],
                  ],
                ),
              ),
            ),
            const VSpace(20),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                OutlinedButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('取消'),
                ),
                const HSpace(12),
                ElevatedButton(
                  onPressed: _saveAgent,
                  child: Text(isEditing ? '保存' : '创建'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _saveAgent() {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入智能体名称')),
      );
      return;
    }

    // 解析工具结果最大长度
    final maxToolResultLength = int.tryParse(_maxToolResultLengthController.text) ?? 4000;
    
    // 验证工具结果最大长度
    if (_enableToolCalling && maxToolResultLength > 0 && 
        (maxToolResultLength < 1000 || maxToolResultLength > 32000)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('工具结果最大长度必须在 1000-32000 字符之间')),
      );
      return;
    }

    // 解析反思迭代次数
    final maxReflectionIterations = int.tryParse(_maxReflectionIterationsController.text) ?? 3;
    
    // 验证反思迭代次数
    if (_enableReflection && (maxReflectionIterations < 1 || maxReflectionIterations > 10)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('反思迭代次数必须在 1-10 次之间')),
      );
      return;
    }

    // 🆕 构建可用工具列表
    final availableTools = <String>[];
    if (_enableWebSearch) {
      availableTools.addAll(['web_search', 'quick_search']);
    }

    final capabilities = AgentCapabilitiesPB()
      ..enablePlanning = _enablePlanning
      ..enableToolCalling = _enableToolCalling
      ..enableReflection = _enableReflection
      ..enableMemory = _enableMemory
      ..maxPlanningSteps = 10
      ..maxToolCalls = 50
      ..memoryLimit = 100
      ..maxToolResultLength = maxToolResultLength
      ..maxReflectionIterations = _enableReflection ? maxReflectionIterations : 0;

    if (widget.existingAgent != null) {
      final request = UpdateAgentRequestPB()
        ..id = widget.existingAgent!.id
        ..name = name
        ..description = _descriptionController.text.trim()
        ..personality = _personalityController.text.trim()
        ..avatar = _avatarController.text.trim()
        ..capabilities = capabilities
        ..availableTools.addAll(availableTools)  // 🆕 添加可用工具列表
        ..selectedMcpServers.addAll(_selectedMCPServerIds);  // 🆕 传递选中的服务器列表

      context.read<AgentSettingsBloc>().add(
        AgentSettingsEvent.updateAgent(request),
      );
    } else {
      final request = CreateAgentRequestPB()
        ..name = name
        ..description = _descriptionController.text.trim()
        ..personality = _personalityController.text.trim()
        ..avatar = _avatarController.text.trim()
        ..capabilities = capabilities
        ..availableTools.addAll(availableTools)  // 🆕 添加可用工具列表
        ..selectedMcpServers.addAll(_selectedMCPServerIds);  // 🆕 传递选中的服务器列表

      context.read<AgentSettingsBloc>().add(
        AgentSettingsEvent.createAgent(request),
      );
    }

    Navigator.of(context).pop();
  }

  // 🆕 构建 MCP 服务器选择器
  Widget _buildMCPServerSelector() {
    return BlocProvider(
      create: (context) => MCPSettingsBloc()..add(const MCPSettingsEvent.loadServerList()),
      child: BlocBuilder<MCPSettingsBloc, MCPSettingsState>(
        builder: (context, state) {
          if (state.isLoading) {
            return const Center(
              child: Padding(
                padding: EdgeInsets.all(20),
                child: CircularProgressIndicator(),
              ),
            );
          }

          if (state.servers.isEmpty) {
            return Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: Theme.of(context).dividerColor,
                ),
              ),
              child: Column(
                children: [
                  Icon(
                    Icons.info_outline,
                    size: 32,
                    color: Theme.of(context).textTheme.bodySmall?.color,
                  ),
                  const VSpace(8),
                  FlowyText.regular(
                    "暂无可用的 MCP 服务器",
                    fontSize: 13,
                    color: Theme.of(context).textTheme.bodySmall?.color,
                  ),
                  const VSpace(4),
                  FlowyText.regular(
                    "请先在 MCP 设置中添加和配置服务器",
                    fontSize: 11,
                    color: Theme.of(context).textTheme.bodySmall?.color,
                  ),
                ],
              ),
            );
          }

          // 🔧 修改：显示所有服务器，不再过滤 isActive
          // 让用户可以选择任何已配置的服务器
          final availableServers = state.servers;

          return Container(
            constraints: const BoxConstraints(maxHeight: 200),
            decoration: BoxDecoration(
              border: Border.all(color: Theme.of(context).dividerColor),
              borderRadius: BorderRadius.circular(8),
            ),
            child: ListView.separated(
              shrinkWrap: true,
              itemCount: availableServers.length,
              separatorBuilder: (context, index) => Divider(height: 1, color: Theme.of(context).dividerColor),
              itemBuilder: (context, index) {
                final server = availableServers[index];
                final isSelected = _selectedMCPServerIds.contains(server.id);
                final toolCount = server.hasCachedTools() ? server.cachedTools.tools.length : 0;
                
                return CheckboxListTile(
                  dense: true,
                  value: isSelected,
                  onChanged: (checked) {
                    setState(() {
                      if (checked == true) {
                        _selectedMCPServerIds.add(server.id);
                      } else {
                        _selectedMCPServerIds.remove(server.id);
                      }
                    });
                  },
                  title: Row(
                    children: [
                      if (server.icon.isNotEmpty) ...[
                        Text(server.icon, style: const TextStyle(fontSize: 16)),
                        const HSpace(8),
                      ],
                      Expanded(
                        child: FlowyText.medium(server.name, fontSize: 13),
                      ),
                      // 🔧 新增：显示服务器状态标签
                      if (!server.isActive) ...[
                        const HSpace(4),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Theme.of(context).colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(4),
                            border: Border.all(
                              color: Theme.of(context).dividerColor,
                            ),
                          ),
                          child: FlowyText.regular(
                            '未激活',
                            fontSize: 10,
                            color: Theme.of(context).textTheme.bodySmall?.color,
                          ),
                        ),
                      ],
                    ],
                  ),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (server.description.isNotEmpty) ...[
                        const VSpace(2),
                        FlowyText.regular(
                          server.description,
                          fontSize: 11,
                          color: Theme.of(context).textTheme.bodySmall?.color,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                      if (toolCount > 0) ...[
                        const VSpace(4),
                        Row(
                          children: [
                            Icon(
                              Icons.build_circle_outlined,
                              size: 12,
                              color: Theme.of(context).colorScheme.primary,
                            ),
                            const HSpace(4),
                            FlowyText.regular(
                              '$toolCount 个工具',
                              fontSize: 11,
                              color: Theme.of(context).colorScheme.primary,
                            ),
                          ],
                        ),
                      ],
                    ],
                  ),
                );
              },
            ),
          );
        },
      ),
    );
  }
}

