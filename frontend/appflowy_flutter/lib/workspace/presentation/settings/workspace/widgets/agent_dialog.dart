import 'package:appflowy/plugins/ai_chat/application/agent_settings_bloc.dart';
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
  
  bool _enablePlanning = true;
  bool _enableToolCalling = true;
  bool _enableReflection = false;
  bool _enableMemory = true;

  @override
  void initState() {
    super.initState();
    
    _nameController = TextEditingController(text: widget.existingAgent?.name ?? '');
    _descriptionController = TextEditingController(text: widget.existingAgent?.description ?? '');
    _personalityController = TextEditingController(text: widget.existingAgent?.personality ?? '');
    _avatarController = TextEditingController(text: widget.existingAgent?.avatar ?? '');
    
    if (widget.existingAgent?.hasCapabilities() == true) {
      final cap = widget.existingAgent!.capabilities;
      _enablePlanning = cap.enablePlanning;
      _enableToolCalling = cap.enableToolCalling;
      _enableReflection = cap.enableReflection;
      _enableMemory = cap.enableMemory;
    }
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _personalityController.dispose();
    _avatarController.dispose();
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
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("反思机制", fontSize: 14)),
                        Switch(value: _enableReflection, onChanged: (v) => setState(() => _enableReflection = v)),
                      ],
                    ),
                    Row(
                      children: [
                        Expanded(child: FlowyText.regular("会话记忆", fontSize: 14)),
                        Switch(value: _enableMemory, onChanged: (v) => setState(() => _enableMemory = v)),
                      ],
                    ),
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

    final capabilities = AgentCapabilitiesPB()
      ..enablePlanning = _enablePlanning
      ..enableToolCalling = _enableToolCalling
      ..enableReflection = _enableReflection
      ..enableMemory = _enableMemory
      ..maxPlanningSteps = 10
      ..maxToolCalls = 50
      ..memoryLimit = 100;

    if (widget.existingAgent != null) {
      final request = UpdateAgentRequestPB()
        ..id = widget.existingAgent!.id
        ..name = name
        ..description = _descriptionController.text.trim()
        ..personality = _personalityController.text.trim()
        ..avatar = _avatarController.text.trim()
        ..capabilities = capabilities
        ..availableTools.addAll(['default_tool']);

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
        ..availableTools.addAll(['default_tool']);

      context.read<AgentSettingsBloc>().add(
        AgentSettingsEvent.createAgent(request),
      );
    }

    Navigator.of(context).pop();
  }
}

