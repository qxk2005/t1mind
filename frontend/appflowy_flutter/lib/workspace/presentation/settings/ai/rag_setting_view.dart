import 'package:appflowy/workspace/application/settings/ai/rag_setting_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/widgets/toggle/toggle.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// RAG设置视图
/// 提供RAG配置的用户界面，包括切块、检索和反思设置
class RAGSettingView extends StatelessWidget {
  const RAGSettingView({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => RAGSettingBloc()..add(const RAGSettingEvent.load()),
      child: SettingsBody(
        title: 'RAG 设置',
        description: '配置检索增强生成（RAG）的参数以优化文档检索和答案质量',
        children: const [
          _ChunkingSection(),
          VSpace(16),
          _RetrievalSection(),
          VSpace(16),
          _ReflectionSection(),
          VSpace(24),
          _ActionButtons(),
        ],
      ),
    );
  }
}

/// 切块配置区块
class _ChunkingSection extends StatelessWidget {
  const _ChunkingSection();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<RAGSettingBloc, RAGSettingState>(
      builder: (context, state) {
        final settings = state.settings;
        return Container(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(12),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    Icons.view_agenda,
                    size: 20,
                    color: Theme.of(context).colorScheme.primary,
                  ),
                  const HSpace(8),
                  FlowyText.medium(
                    '文本切块配置',
                    color: AFThemeExtension.of(context).strongText,
                  ),
                ],
              ),
              const VSpace(16),
              _buildNumberInput(
                context,
                label: '块大小',
                value: settings.chunkSize,
                hint: '推荐: 1000',
                description: '每个文本块包含的字符数。较大的块捕获更多上下文，但可能错过细节。',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateChunkSize(value));
                },
                min: 100,
                max: 5000,
              ),
              const VSpace(16),
              _buildNumberInput(
                context,
                label: '块重叠',
                value: settings.chunkOverlap,
                hint: '推荐: 200',
                description:
                    '相邻块之间重叠的字符数。有助于保持上下文连续性。',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateChunkOverlap(value));
                },
                min: 0,
                max: settings.chunkSize ~/ 2,
              ),
              const VSpace(16),
              _buildToggle(
                context,
                label: '启用语义分割',
                value: settings.enableSemanticSplitting,
                description: '使用语义分析智能切分文本，而不是简单的字符切片',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateEnableSemanticSplitting(value));
                },
              ),
            ],
          ),
        );
      },
    );
  }
}

/// 检索配置区块
class _RetrievalSection extends StatelessWidget {
  const _RetrievalSection();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<RAGSettingBloc, RAGSettingState>(
      builder: (context, state) {
        final settings = state.settings;
        return Container(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(12),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    Icons.search,
                    size: 20,
                    color: Theme.of(context).colorScheme.primary,
                  ),
                  const HSpace(8),
                  FlowyText.medium(
                    '检索配置',
                    color: AFThemeExtension.of(context).strongText,
                  ),
                ],
              ),
              const VSpace(16),
              _buildToggle(
                context,
                label: '启用混合检索',
                value: settings.enableHybridSearch,
                description: '同时使用向量检索和关键词检索，结合两者优势',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateEnableHybridSearch(value));
                },
              ),
              if (settings.enableHybridSearch) ...[
                const VSpace(16),
                _buildSlider(
                  context,
                  label: '向量权重',
                  value: settings.vectorWeight,
                  onChanged: (value) {
                    context
                        .read<RAGSettingBloc>()
                        .add(RAGSettingEvent.updateVectorWeight(value));
                  },
                ),
                const VSpace(16),
                _buildSlider(
                  context,
                  label: '关键词权重',
                  value: settings.keywordWeight,
                  onChanged: (value) {
                    context
                        .read<RAGSettingBloc>()
                        .add(RAGSettingEvent.updateKeywordWeight(value));
                  },
                ),
              ],
              const VSpace(16),
              _buildNumberInput(
                context,
                label: '初始检索数量',
                value: settings.initialTopK,
                hint: '推荐: 20',
                description:
                    '初步检索时返回的文档数量。建议设置为最终数量的3-4倍。',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateInitialTopK(value));
                },
                min: 5,
                max: 100,
              ),
              const VSpace(16),
              _buildNumberInput(
                context,
                label: '最终返回数量',
                value: settings.finalTopK,
                hint: '推荐: 5',
                description:
                    '经过重排序后最终返回的文档数量，不应超过初始检索数量。',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateFinalTopK(value));
                },
                min: 1,
                max: settings.initialTopK,
              ),
              const VSpace(16),
              _buildToggle(
                context,
                label: '启用重排序',
                value: settings.enableReranking,
                description:
                    '使用高级模型重新评估和排序检索结果，提升相关性',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateEnableReranking(value));
                },
              ),
              if (settings.enableReranking) ...[
                const VSpace(16),
                _buildTextInput(
                  context,
                  label: '重排序模型',
                  value: settings.rerankerModel ?? '',
                  hint: 'model-name',
                  description: '重排序使用的模型名称（可选，默认使用当前聊天模型）',
                  onChanged: (value) {
                    context
                        .read<RAGSettingBloc>()
                        .add(RAGSettingEvent.updateRerankerModel(value));
                  },
                ),
                const VSpace(16),
                _buildTextInput(
                  context,
                  label: '重排序 API URL',
                  value: settings.rerankerApiUrl ?? '',
                  hint: 'http://example.com/v1',
                  description: '独立重排序服务的 API 端点（可选，留空则使用主模型）',
                  onChanged: (value) {
                    context
                        .read<RAGSettingBloc>()
                        .add(RAGSettingEvent.updateRerankerApiUrl(value));
                  },
                ),
                const VSpace(16),
                _buildTextInput(
                  context,
                  label: '重排序 API Key',
                  value: settings.rerankerApiKey ?? '',
                  hint: 'sk-xxxx',
                  description: '重排序 API 的认证密钥（可选）',
                  onChanged: (value) {
                    context
                        .read<RAGSettingBloc>()
                        .add(RAGSettingEvent.updateRerankerApiKey(value));
                  },
                ),
              ],
            ],
          ),
        );
      },
    );
  }
}

/// 反思配置区块
class _ReflectionSection extends StatelessWidget {
  const _ReflectionSection();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<RAGSettingBloc, RAGSettingState>(
      builder: (context, state) {
        final settings = state.settings;
        return Container(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(12),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    Icons.psychology,
                    size: 20,
                    color: Theme.of(context).colorScheme.primary,
                  ),
                  const HSpace(8),
                  FlowyText.medium(
                    '智能体反思',
                    color: AFThemeExtension.of(context).strongText,
                  ),
                ],
              ),
              const VSpace(16),
              _buildToggle(
                context,
                label: '启用智能体反思',
                value: settings.enableAgentReflection,
                description:
                    '让AI评估自己的回答质量，并在答案不完整时主动补充',
                onChanged: (value) {
                  context
                      .read<RAGSettingBloc>()
                      .add(RAGSettingEvent.updateEnableAgentReflection(value));
                },
              ),
              if (settings.enableAgentReflection) ...[
                const VSpace(16),
                _buildSlider(
                  context,
                  label: '反思阈值',
                  value: settings.reflectionThreshold,
                  min: 0.0,
                  max: 1.0,
                  divisions: 20,
                  description:
                      '置信度阈值。当答案的置信度低于此值时，会触发反思和补充搜索。',
                  onChanged: (value) {
                    context
                        .read<RAGSettingBloc>()
                        .add(RAGSettingEvent.updateReflectionThreshold(value));
                  },
                ),
              ],
            ],
          ),
        );
      },
    );
  }
}

/// 操作按钮区域
class _ActionButtons extends StatelessWidget {
  const _ActionButtons();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<RAGSettingBloc, RAGSettingState>(
      builder: (context, state) {
        return Column(
          children: [
            if (state.error != null) ...[
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.red.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.red, width: 1),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.error, color: Colors.red, size: 20),
                    const HSpace(8),
                    Expanded(
                      child: FlowyText.regular(
                        state.error!,
                        color: Colors.red,
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.close, size: 16),
                      onPressed: () {
                        context
                            .read<RAGSettingBloc>()
                            .add(const RAGSettingEvent.clearError());
                      },
                    ),
                  ],
                ),
              ),
              const VSpace(12),
            ],
            if (state.successMessage != null) ...[
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.green.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.green, width: 1),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.check_circle, color: Colors.green, size: 20),
                    const HSpace(8),
                    Expanded(
                      child: FlowyText.regular(
                        state.successMessage!,
                        color: Colors.green,
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.close, size: 16),
                      onPressed: () {
                        context
                            .read<RAGSettingBloc>()
                            .add(const RAGSettingEvent.clearSuccess());
                      },
                    ),
                  ],
                ),
              ),
              const VSpace(12),
            ],
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: state.isSaving ? null : () {
                    context
                        .read<RAGSettingBloc>()
                        .add(const RAGSettingEvent.reset());
                  },
                  child: const Text('重置'),
                ),
                const HSpace(12),
                ElevatedButton(
                  onPressed: state.isSaving ? null : () {
                    context
                        .read<RAGSettingBloc>()
                        .add(const RAGSettingEvent.save());
                  },
                  child: state.isSaving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('保存'),
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}

/// 构建数字输入框
Widget _buildNumberInput(
  BuildContext context, {
  required String label,
  required int value,
  required String hint,
  required String description,
  required ValueChanged<int> onChanged,
  required int min,
  required int max,
}) {
  final controller = TextEditingController(text: value.toString());

  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Row(
        children: [
          FlowyText.regular(label),
          const Spacer(),
          FlowyText.regular(
            '推荐: $hint',
            fontSize: 12,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
        ],
      ),
      const VSpace(8),
      FlowyText.small(
        description,
        color: Theme.of(context).colorScheme.onSurfaceVariant,
      ),
      const VSpace(8),
      Row(
        children: [
          Expanded(
            child: SizedBox(
              height: 40,
              child: TextField(
                controller: controller,
                keyboardType: TextInputType.number,
                inputFormatters: [
                  FilteringTextInputFormatter.digitsOnly,
                ],
                decoration: InputDecoration(
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(8),
                  ),
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 8,
                  ),
                ),
                onChanged: (text) {
                  final parsedValue = int.tryParse(text);
                  if (parsedValue != null) {
                    final clampedValue = parsedValue.clamp(min, max);
                    if (clampedValue != parsedValue) {
                      controller.value = TextEditingValue(
                        text: clampedValue.toString(),
                        selection: TextSelection.collapsed(offset: clampedValue.toString().length),
                      );
                    }
                    onChanged(clampedValue);
                  }
                },
              ),
            ),
          ),
          const HSpace(8),
          SizedBox(
            width: 60,
            child: FlowyText.small(
              '范围: $min-$max',
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      ),
    ],
  );
}

/// 构建文本输入框
Widget _buildTextInput(
  BuildContext context, {
  required String label,
  required String value,
  required String hint,
  required String description,
  required ValueChanged<String?> onChanged,
}) {
  final controller = TextEditingController(text: value);

  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      FlowyText.regular(label),
      const VSpace(8),
      FlowyText.small(
        description,
        color: Theme.of(context).colorScheme.onSurfaceVariant,
      ),
      const VSpace(8),
      SizedBox(
        height: 40,
        child: TextField(
          controller: controller,
          decoration: InputDecoration(
            hintText: hint,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
            ),
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
          ),
          onChanged: (text) {
            onChanged(text.isEmpty ? null : text);
          },
        ),
      ),
    ],
  );
}

/// 构建开关组件
Widget _buildToggle(
  BuildContext context, {
  required String label,
  required bool value,
  required String description,
  required ValueChanged<bool> onChanged,
}) {
  return Row(
    children: [
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            FlowyText.regular(label),
            const VSpace(4),
            FlowyText.small(
              description,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ],
        ),
      ),
      const HSpace(16),
      Toggle(
        value: value,
        onChanged: onChanged,
      ),
    ],
  );
}

/// 构建滑块组件
Widget _buildSlider(
  BuildContext context, {
  required String label,
  required double value,
  required ValueChanged<double> onChanged,
  double min = 0.0,
  double max = 1.0,
  int? divisions,
  String? description,
}) {
  return Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Row(
        children: [
          FlowyText.regular(label),
          const Spacer(),
          FlowyText.regular(
            '${(value * 100).toStringAsFixed(0)}%',
            color: Theme.of(context).colorScheme.primary,
          ),
        ],
      ),
      if (description != null) ...[
        const VSpace(4),
        FlowyText.small(
          description,
          color: Theme.of(context).colorScheme.onSurfaceVariant,
        ),
      ],
      const VSpace(8),
      Slider(
        value: value,
        min: min,
        max: max,
        divisions: divisions,
        label: '${(value * 100).toStringAsFixed(0)}%',
        onChanged: onChanged,
      ),
    ],
  );
}

