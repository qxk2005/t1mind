import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra/size.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter/services.dart';

import 'package:appflowy/workspace/application/settings/ai/openai_compat_setting_bloc.dart';

class OpenAICompatSetting extends StatelessWidget {
  const OpenAICompatSetting({super.key, this.workspaceId});

  final String? workspaceId;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return BlocProvider(
      create: (_) => OpenAICompatSettingBloc(workspaceId: workspaceId),
      child: BlocBuilder<OpenAICompatSettingBloc, OpenAICompatSettingState>(
        builder: (context, state) {
          if (state.isLoading) {
            return const SizedBox(
              height: 160,
              child: Center(
                child: CircularProgressIndicator.adaptive(),
              ),
            );
          }
          return Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                "settings.aiPage.keys.openAICompatTitle".tr(),
                style: theme.textStyle.body.enhanced(
                  color: theme.textColorScheme.primary,
                ),
              ),
              const VSpace(4),
              FlowyText(
                "settings.aiPage.keys.openAICompatSubTitle".tr(),
                maxLines: 3,
                fontSize: 12,
              ),
              const VSpace(10),
              _Form(state: state),
              const VSpace(10),
              _Actions(state: state),
              if (state.testResult != null) ...[
                const VSpace(10),
                _TestResultView(result: state.testResult!),
              ],
              const VSpace(10),
              _EmbedForm(state: state),
              const VSpace(10),
              _EmbedActions(state: state),
              if (state.embedTestResult != null) ...[
                const VSpace(10),
                _EmbedTestResultView(result: state.embedTestResult!),
              ],
              const VSpace(10),
              _RagConfigForm(state: state),
            ],
          );
        },
      ),
    );
  }
}

class _Form extends StatelessWidget {
  const _Form({required this.state});
  final OpenAICompatSettingState state;

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<OpenAICompatSettingBloc>();
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).colorScheme.outline),
        borderRadius: Corners.s8Border,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _LabeledField(
            label: 'settings.aiPage.keys.baseUrlLabel'.tr(),
            child: TextFormField(
              initialValue: state.baseUrl,
              onChanged: bloc.updateBaseUrl,
              decoration: const InputDecoration(hintText: 'https://api.openai.com'),
            ),
          ),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.chatBaseUrlLabel'.tr(),
            child: TextFormField(
              initialValue: state.chatBaseUrl,
              onChanged: bloc.updateChatBaseUrl,
              decoration: const InputDecoration(hintText: '可选，覆盖聊天接口 Base URL'),
            ),
          ),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.embedBaseUrlLabel'.tr(),
            child: TextFormField(
              initialValue: state.embedBaseUrl,
              onChanged: bloc.updateEmbedBaseUrl,
              decoration: const InputDecoration(hintText: '可选，覆盖嵌入接口 Base URL'),
            ),
          ),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.apiKeyLabel'.tr(),
            child: TextFormField(
              initialValue: state.apiKey.isEmpty ? '' : '••••••••',
              obscureText: true,
              enableSuggestions: false,
              autocorrect: false,
              onChanged: bloc.updateApiKey,
              decoration: const InputDecoration(hintText: 'sk-...'),
            ),
          ),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.modelLabel'.tr(),
            child: TextFormField(
              initialValue: state.model,
              onChanged: bloc.updateModel,
              decoration: const InputDecoration(hintText: 'gpt-4o-mini'),
            ),
          ),
          const VSpace(8),
          Row(children: [
            Expanded(
              child: _LabeledField(
                label: 'settings.aiPage.keys.temperatureLabel'.tr(),
                child: TextFormField(
                  initialValue: state.temperature.toString(),
                  keyboardType: TextInputType.number,
                  onChanged: (v) => bloc.updateTemperature(double.tryParse(v) ?? 0.7),
                ),
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _LabeledField(
                label: 'settings.aiPage.keys.maxTokensLabel'.tr(),
                child: TextFormField(
                  initialValue: state.maxTokens.toString(),
                  keyboardType: TextInputType.number,
                  onChanged: (v) => bloc.updateMaxTokens(int.tryParse(v) ?? 1024),
                ),
              ),
            ),
          ]),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.timeoutMsLabel'.tr(),
            child: TextFormField(
              initialValue: state.timeoutMs.toString(),
              keyboardType: TextInputType.number,
              onChanged: (v) => bloc.updateTimeoutMs(int.tryParse(v) ?? 60000),
            ),
          ),
        ],
      ),
    );
  }
}

class _Actions extends StatelessWidget {
  const _Actions({required this.state});
  final OpenAICompatSettingState state;

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<OpenAICompatSettingBloc>();
    return Row(
      children: [
        FilledButton(
          onPressed: state.isSaving ? null : () => bloc.save(),
          child: state.isSaving
              ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
              : Text('button.save'.tr()),
        ),
        const HSpace(8),
        OutlinedButton(
          onPressed: state.isTesting ? null : () => bloc.testChat(),
          child: state.isTesting
              ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
              : Text('settings.aiPage.keys.testChat'.tr()),
        ),
        if (state.error != null) ...[
          const HSpace(12),
          Flexible(child: Text(state.error!, style: const TextStyle(color: Colors.red))),
        ],
      ],
    );
  }
}

class _LabeledField extends StatelessWidget {
  const _LabeledField({required this.label, required this.child});
  final String label;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(label),
        const VSpace(6),
        child,
      ],
    );
  }
}

class _TestResultView extends StatelessWidget {
  const _TestResultView({required this.result});
  final OpenAITestResult result;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).colorScheme.outline),
        borderRadius: Corners.s8Border,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            result.success ? 'settings.aiPage.keys.success'.tr() : 'settings.aiPage.keys.failed'.tr(),
            style: theme.textStyle.body.enhanced(),
          ),
          const VSpace(4),
          FlowyText('${'settings.aiPage.keys.latency'.tr()}: ${result.latencyMs}ms'),
          if (result.text.isNotEmpty) ...[
            const VSpace(6),
            FlowyText('Text: ${result.text}'),
          ],
          if (result.error != null) ...[
            const VSpace(6),
            FlowyText('${'settings.aiPage.keys.errorLabel'.tr()}: ${result.error}'),
          ],
          const VSpace(6),
          Row(
            children: [
              if (result.timeline.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(right: 12),
                  child: FlowyText('Timeline: ${result.timeline.length}'),
                ),
              OutlinedButton(
                onPressed: () {
                  final text = result.debug.isEmpty ? 'no_debug' : result.debug;
                  Clipboard.setData(ClipboardData(text: text));
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text('settings.aiPage.keys.debugInfoCopied'.tr())),
                  );
                },
                child: Text('settings.aiPage.keys.copyDebugInfo'.tr()),
              ),
            ],
          ),
          if (result.debug.isNotEmpty) ...[
            const VSpace(6),
            FlowyText('Debug:'),
            const VSpace(4),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.4),
                borderRadius: Corners.s6Border,
              ),
              child: SelectableText(result.debug, style: const TextStyle(fontFamily: 'monospace', fontSize: 12)),
            ),
          ],
        ],
      ),
    );
  }
}

class _EmbedForm extends StatelessWidget {
  const _EmbedForm({required this.state});
  final OpenAICompatSettingState state;

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<OpenAICompatSettingBloc>();
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).colorScheme.outline),
        borderRadius: Corners.s8Border,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FlowyText.medium('settings.aiPage.keys.embeddingsTitle'.tr()),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.embedBaseUrlLabel'.tr(),
            child: TextFormField(
              initialValue: state.embedBaseUrl,
              onChanged: bloc.updateEmbedBaseUrl,
              decoration: const InputDecoration(hintText: '可选，覆盖嵌入接口 Base URL'),
            ),
          ),
          const VSpace(8),
          _LabeledField(
            label: 'settings.aiPage.keys.embeddingModelLabel'.tr(),
            child: TextFormField(
              initialValue: state.embeddingModel,
              onChanged: bloc.updateEmbeddingModel,
              decoration: const InputDecoration(hintText: 'text-embedding-3-small'),
            ),
          ),
        ],
      ),
    );
  }
}

class _EmbedActions extends StatelessWidget {
  const _EmbedActions({required this.state});
  final OpenAICompatSettingState state;

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<OpenAICompatSettingBloc>();
    return Row(
      children: [
        OutlinedButton(
          onPressed: state.isTestingEmbed ? null : () => bloc.testEmbeddings(),
          child: state.isTestingEmbed
              ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
              : Text('settings.aiPage.keys.testEmbeddings'.tr()),
        ),
        const HSpace(8),
        if (state.embedTestResult != null)
          OutlinedButton(
            onPressed: () {
              final text = state.embedTestResult!.debug;
              Clipboard.setData(ClipboardData(text: text));
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('settings.aiPage.keys.debugInfoCopied'.tr())),
              );
            },
            child: Text('settings.aiPage.keys.copyDebugInfo'.tr()),
          ),
      ],
    );
  }
}

class _EmbedTestResultView extends StatelessWidget {
  const _EmbedTestResultView({required this.result});
  final OpenAIEmbedTestResult result;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).colorScheme.outline),
        borderRadius: Corners.s8Border,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            result.success ? 'settings.aiPage.keys.success'.tr() : 'settings.aiPage.keys.failed'.tr(),
            style: theme.textStyle.body.enhanced(),
          ),
          const VSpace(4),
          FlowyText('${'settings.aiPage.keys.latency'.tr()}: ${result.latencyMs}ms'),
          const VSpace(6),
          FlowyText('${'settings.aiPage.keys.vectorDim'.tr()}: ${result.vectorDim}'),
          if (result.error != null) ...[
            const VSpace(6),
            FlowyText('${'settings.aiPage.keys.errorLabel'.tr()}: ${result.error}'),
          ],
        ],
      ),
    );
  }
}

class _RagConfigForm extends StatelessWidget {
  const _RagConfigForm({required this.state});
  final OpenAICompatSettingState state;

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<OpenAICompatSettingBloc>();
    final theme = AppFlowyTheme.of(context);
    
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          "RAG 文档检索配置",
          style: theme.textStyle.body.enhanced(
            color: theme.textColorScheme.primary,
          ),
        ),
        const VSpace(4),
        FlowyText(
          "配置文档检索的相似度阈值。较低的值会检索更多文档，但可能包含不相关的内容。",
          maxLines: 3,
          fontSize: 12,
        ),
        const VSpace(8),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            border: Border.all(color: Theme.of(context).colorScheme.outline),
            borderRadius: Corners.s8Border,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.medium('相似度阈值 (Similarity Threshold)'),
              const VSpace(6),
              TextFormField(
                initialValue: state.ragScoreThreshold.toString(),
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                onChanged: (v) => bloc.updateRagScoreThreshold(double.tryParse(v) ?? 0.25),
                decoration: const InputDecoration(
                  hintText: '0.25',
                  helperText: '推荐范围: 0.2 - 0.4，默认: 0.25',
                  helperMaxLines: 2,
                ),
              ),
              const VSpace(4),
              FlowyText(
                '当前值: ${state.ragScoreThreshold.toStringAsFixed(2)} (${_getThresholdDescription(state.ragScoreThreshold)})',
                fontSize: 12,
              ),
            ],
          ),
        ),
      ],
    );
  }

  String _getThresholdDescription(double threshold) {
    if (threshold < 0.2) return '极低 - 会检索大量可能不相关的文档';
    if (threshold < 0.3) return '较低 - 检索范围广泛';
    if (threshold < 0.4) return '适中 - 推荐设置';
    if (threshold < 0.5) return '较高 - 只检索高度相关的文档';
    return '很高 - 可能漏检相关文档';
  }
}

