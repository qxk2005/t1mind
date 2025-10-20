import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/settings_ai_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/model_selection.dart';
import 'package:appflowy/workspace/presentation/settings/pages/setting_ai_view/provider_selector.dart';
import 'package:appflowy/workspace/application/settings/ai/ai_provider_cubit.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/widgets/toggle/toggle.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

class SettingsAIView extends StatelessWidget {
  const SettingsAIView({
    super.key,
    required this.userProfile,
    required this.currentWorkspaceMemberRole,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final AFRolePB? currentWorkspaceMemberRole;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SettingsAIBloc>(
      create: (_) => SettingsAIBloc(userProfile, workspaceId)
        ..add(const SettingsAIEvent.started()),
      child: BlocProvider(
        create: (_) => AiProviderCubit(workspaceId: workspaceId),
        child: SettingsBody(
          title: LocaleKeys.settings_aiPage_title.tr(),
          description: LocaleKeys.settings_aiPage_keys_aiSettingsDescription.tr(),
          children: [
            const ProviderDropdown(),
            const AIModelSelection(),
            const _AISearchToggle(value: false),
            ProviderTabSwitcher(workspaceId: workspaceId),
            const SizedBox(height: 32),
            const _VectorDatabaseResetSection(),
          ],
        ),
      ),
    );
  }
}

class _AISearchToggle extends StatelessWidget {
  const _AISearchToggle({required this.value});

  final bool value;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            FlowyText.medium(
              LocaleKeys.settings_aiPage_keys_enableAISearchTitle.tr(),
            ),
            const Spacer(),
            BlocBuilder<SettingsAIBloc, SettingsAIState>(
              builder: (context, state) {
                if (state.aiSettings == null) {
                  return const Padding(
                    padding: EdgeInsets.only(top: 6),
                    child: SizedBox(
                      height: 26,
                      width: 26,
                      child: CircularProgressIndicator.adaptive(),
                    ),
                  );
                } else {
                  return Toggle(
                    value: state.enableSearchIndexing,
                    onChanged: (_) => context
                        .read<SettingsAIBloc>()
                        .add(const SettingsAIEvent.toggleAISearch()),
                  );
                }
              },
            ),
          ],
        ),
      ],
    );
  }
}

/// 重置向量数据库独立区块
class _VectorDatabaseResetSection extends StatefulWidget {
  const _VectorDatabaseResetSection();

  @override
  State<_VectorDatabaseResetSection> createState() => _VectorDatabaseResetSectionState();
}

class _VectorDatabaseResetSectionState extends State<_VectorDatabaseResetSection> {
  final TextEditingController _confirmationController = TextEditingController();

  @override
  void dispose() {
    _confirmationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.red[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.red[200]!),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.warning_amber_rounded,
                color: Colors.red[700],
                size: 20,
              ),
              const SizedBox(width: 8),
              FlowyText.medium(
                '危险操作：重置向量数据库',
                fontSize: 16,
                color: Colors.red[700],
              ),
            ],
          ),
          const SizedBox(height: 12),
          FlowyText.regular(
            '当嵌入模型维度发生变化时，需要重置向量数据库以匹配新的维度。\n\n'
            '⚠️ 此操作将永久删除所有已索引的文档和嵌入数据，无法恢复！\n'
            '⚠️ 重置后，系统将重新开始索引您的文档，这可能需要一些时间。\n'
            '✅ 支持自动检测当前嵌入模型维度并重建数据库结构。',
            fontSize: 13,
            color: Colors.red[600],
          ),
          const SizedBox(height: 12),
          BlocBuilder<SettingsAIBloc, SettingsAIState>(
            builder: (context, state) {
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // 显示当前维度和配置选项
                  if (state.currentEmbeddingDimension != null) ...[
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                      decoration: BoxDecoration(
                        color: Colors.blue[50],
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(color: Colors.blue[200]!),
                      ),
                      child: Row(
                        children: [
                          Icon(
                            Icons.info_outline,
                            color: Colors.blue[700],
                            size: 16,
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: FlowyText.regular(
                              '当前向量数据库维度：${state.currentEmbeddingDimension}维',
                              fontSize: 13,
                              color: Colors.blue[700],
                            ),
                          ),
                          TextButton(
                            onPressed: () => _showConfigureDimensionDialog(context),
                            child: FlowyText.regular(
                              '配置维度',
                              fontSize: 12,
                              color: Colors.blue[700],
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 12),
                  ],
                  
                  // 显示重置完成消息
                  if (state.resetCompletionMessage != null) ...[
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                      decoration: BoxDecoration(
                        color: Colors.green[50],
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(color: Colors.green[200]!),
                      ),
                      child: Row(
                        children: [
                          Icon(
                            Icons.check_circle_outline,
                            color: Colors.green[700],
                            size: 16,
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: FlowyText.regular(
                              state.resetCompletionMessage!,
                              fontSize: 13,
                              color: Colors.green[700],
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 12),
                  ],
                  
                  // 重置按钮组
                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton(
                          onPressed: state.isResettingVectorDB
                              ? null
                              : () => _showResetConfirmationDialog(context),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.red[600],
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                          ),
                          child: state.isResettingVectorDB
                              ? const Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    SizedBox(
                                      height: 16,
                                      width: 16,
                                      child: CircularProgressIndicator(
                                        strokeWidth: 2,
                                        color: Colors.white,
                                      ),
                                    ),
                                    SizedBox(width: 8),
                                    Text('重置中...'),
                                  ],
                                )
                              : const Text('智能重置'),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: ElevatedButton(
                          onPressed: state.isResettingVectorDB
                              ? null
                              : () => _showManualResetDialog(context),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.orange[600],
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                          ),
                          child: const Text('手工指定维度'),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  
                  // 诊断和测试按钮组
                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton(
                          onPressed: () => context.read<SettingsAIBloc>().add(
                            const SettingsAIEvent.checkDimensionCompatibility(),
                          ),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.blue[600],
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                          ),
                          child: const Text('检查维度兼容性'),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: ElevatedButton(
                          onPressed: () => _showTestEmbeddingDialog(context),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: Colors.purple[600],
                            foregroundColor: Colors.white,
                            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                          ),
                          child: const Text('测试嵌入模型'),
                        ),
                      ),
                    ],
                  ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }

        void _showResetConfirmationDialog(BuildContext context) {
          final bloc = context.read<SettingsAIBloc>();
          _confirmationController.clear();

          showDialog(
            context: context,
            builder: (dialogContext) => StatefulBuilder(
              builder: (context, setState) {
                final isConfirmed = _confirmationController.text.trim() == '我确认';

                return AlertDialog(
                  title: Row(
                    children: [
                      Icon(Icons.warning_amber_rounded, color: Colors.red[700]),
                      const SizedBox(width: 8),
                      const Text('确认智能重置向量数据库'),
                    ],
                  ),
                  content: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        '此操作将永久删除所有已索引的文档和嵌入数据，无法恢复！\n\n'
                        '系统将自动检测当前嵌入模型维度并重建数据库结构。\n\n'
                        '为了确认您了解此操作的危险性，请在下方输入框中输入"我确认"：',
                      ),
                      const SizedBox(height: 16),
                      TextField(
                        controller: _confirmationController,
                        decoration: const InputDecoration(
                          hintText: '请输入"我确认"',
                          border: OutlineInputBorder(),
                        ),
                        onChanged: (value) => setState(() {}),
                      ),
                    ],
                  ),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.of(dialogContext).pop(),
                      child: const Text('取消'),
                    ),
                    ElevatedButton(
                      onPressed: isConfirmed
                          ? () {
                              Navigator.of(dialogContext).pop();
                              bloc.add(const SettingsAIEvent.resetVectorDatabase());
                            }
                          : null,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.red,
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('确认智能重置'),
                    ),
                  ],
                );
              },
            ),
          );
        }

        void _showManualResetDialog(BuildContext context) {
          final bloc = context.read<SettingsAIBloc>();
          final dimensionController = TextEditingController();
          _confirmationController.clear();

          showDialog(
            context: context,
            builder: (dialogContext) => StatefulBuilder(
              builder: (context, setState) {
                final isConfirmed = _confirmationController.text.trim() == '我确认';
                final dimensionText = dimensionController.text.trim();
                final dimension = int.tryParse(dimensionText);
                final isValidDimension = dimension != null && dimension > 0 && dimension <= 10000;

                return AlertDialog(
                  title: Row(
                    children: [
                      Icon(Icons.warning_amber_rounded, color: Colors.orange[700]),
                      const SizedBox(width: 8),
                      const Text('手工指定维度重置'),
                    ],
                  ),
                  content: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        '此操作将永久删除所有已索引的文档和嵌入数据，无法恢复！\n\n'
                        '请指定新的向量数据库维度（1-10000）：',
                      ),
                      const SizedBox(height: 16),
                      TextField(
                        controller: dimensionController,
                        decoration: InputDecoration(
                          hintText: '请输入维度，如：1536',
                          border: const OutlineInputBorder(),
                          errorText: dimensionText.isNotEmpty && !isValidDimension
                              ? '请输入有效的维度（1-10000）'
                              : null,
                        ),
                        keyboardType: TextInputType.number,
                        onChanged: (value) => setState(() {}),
                      ),
                      const SizedBox(height: 16),
                      const Text(
                        '常用维度参考：\n'
                        '• OpenAI text-embedding-3-small: 1536\n'
                        '• OpenAI text-embedding-3-large: 3072\n'
                        '• OpenAI text-embedding-ada-002: 1536\n'
                        '• Ollama nomic-embed-text: 768\n'
                        '• BGE-M3: 1024',
                        style: TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                      const SizedBox(height: 16),
                      const Text(
                        '为了确认您了解此操作的危险性，请在下方输入框中输入"我确认"：',
                      ),
                      const SizedBox(height: 8),
                      TextField(
                        controller: _confirmationController,
                        decoration: const InputDecoration(
                          hintText: '请输入"我确认"',
                          border: OutlineInputBorder(),
                        ),
                        onChanged: (value) => setState(() {}),
                      ),
                    ],
                  ),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.of(dialogContext).pop(),
                      child: const Text('取消'),
                    ),
                    ElevatedButton(
                      onPressed: (isConfirmed && isValidDimension)
                          ? () {
                              Navigator.of(dialogContext).pop();
                              bloc.add(SettingsAIEvent.resetVectorDatabaseWithDimension(dimension));
                            }
                          : null,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.orange,
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('确认手工重置'),
                    ),
                  ],
                );
              },
            ),
          );
        }

        void _showConfigureDimensionDialog(BuildContext context) {
          final bloc = context.read<SettingsAIBloc>();
          final dimensionController = TextEditingController();

          showDialog(
            context: context,
            builder: (dialogContext) => StatefulBuilder(
              builder: (context, setState) {
                final dimensionText = dimensionController.text.trim();
                final dimension = int.tryParse(dimensionText);
                final isValidDimension = dimension != null && dimension > 0 && dimension <= 10000;

                return AlertDialog(
                  title: Row(
                    children: [
                      Icon(Icons.settings, color: Colors.blue[700]),
                      const SizedBox(width: 8),
                      const Text('配置嵌入模型维度'),
                    ],
                  ),
                  content: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        '配置嵌入模型的维度，这将影响新文档的索引和搜索。\n\n'
                        '注意：如果维度与当前数据库不匹配，需要重置向量数据库。',
                      ),
                      const SizedBox(height: 16),
                      TextField(
                        controller: dimensionController,
                        decoration: InputDecoration(
                          hintText: '请输入维度，如：1024',
                          border: const OutlineInputBorder(),
                          errorText: dimensionText.isNotEmpty && !isValidDimension
                              ? '请输入有效的维度（1-10000）'
                              : null,
                        ),
                        keyboardType: TextInputType.number,
                        onChanged: (value) => setState(() {}),
                      ),
                      const SizedBox(height: 16),
                      const Text(
                        '常用维度参考：\n'
                        '• OpenAI text-embedding-3-small: 1536\n'
                        '• OpenAI text-embedding-3-large: 3072\n'
                        '• OpenAI text-embedding-ada-002: 1536\n'
                        '• Ollama nomic-embed-text: 768\n'
                        '• BGE-M3: 1024',
                        style: TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                    ],
                  ),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.of(dialogContext).pop(),
                      child: const Text('取消'),
                    ),
                    ElevatedButton(
                      onPressed: isValidDimension
                          ? () {
                              Navigator.of(dialogContext).pop();
                              bloc.add(SettingsAIEvent.configureEmbeddingDimension(dimension));
                            }
                          : null,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.blue,
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('保存配置'),
                    ),
                  ],
                );
              },
            ),
          );
        }

  void _showTestEmbeddingDialog(BuildContext context) {
    final baseUrlController = TextEditingController(text: 'http://10.203.50.9:17015');
    final apiKeyController = TextEditingController(text: 'your-api-key');
    final modelController = TextEditingController(text: 'bge-m3');

    showDialog(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setState) {
          return AlertDialog(
            title: Row(
              children: [
                Icon(Icons.science, color: Colors.purple[700]),
                const SizedBox(width: 8),
                const Text('测试嵌入模型'),
              ],
            ),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  '输入嵌入模型的配置信息进行测试，系统将自动检测模型的实际维度。',
                  style: TextStyle(fontSize: 14),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: baseUrlController,
                  decoration: const InputDecoration(
                    labelText: 'API 基础 URL',
                    hintText: 'http://localhost:17015',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: apiKeyController,
                  decoration: const InputDecoration(
                    labelText: 'API Key',
                    hintText: 'your-api-key',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: modelController,
                  decoration: const InputDecoration(
                    labelText: '模型名称',
                    hintText: 'bge-m3',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 16),
                const Text(
                  '测试说明：\n'
                  '• 系统将发送测试文本到嵌入服务\n'
                  '• 自动检测返回向量的维度\n'
                  '• 可用于验证模型配置是否正确',
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(dialogContext).pop(),
                child: const Text('取消'),
              ),
              ElevatedButton(
                onPressed: () {
                  Navigator.of(dialogContext).pop();
                  context.read<SettingsAIBloc>().add(
                    SettingsAIEvent.testEmbeddingModel(
                      baseUrlController.text.trim(),
                      apiKeyController.text.trim(),
                      modelController.text.trim(),
                    ),
                  );
                },
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.purple,
                  foregroundColor: Colors.white,
                ),
                child: const Text('开始测试'),
              ),
            ],
          );
        },
      ),
    );
  }
}

// Provider UI 已迁移到 provider_selector.dart
