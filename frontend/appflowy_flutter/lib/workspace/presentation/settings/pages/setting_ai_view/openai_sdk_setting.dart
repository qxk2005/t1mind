import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/openai_sdk_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// OpenAI SDK 设置面板
/// 包含聊天配置区域、嵌入配置区域和操作按钮
class OpenAISDKSetting extends StatelessWidget {
  const OpenAISDKSetting({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => OpenAISDKBloc()
        ..add(const OpenAISDKEvent.started()),
      child: BlocBuilder<OpenAISDKBloc, OpenAISDKState>(
        builder: (context, state) {
          if (state.loadingState == LoadingState.loading) {
            return const Center(
              child: CircularProgressIndicator(),
            );
          }

          if (state.loadingState == LoadingState.error) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(
                    Icons.error_outline,
                    size: 48,
                    color: Colors.red,
                  ),
                  const VSpace(16),
                  FlowyText.medium(
                    'Failed to load OpenAI SDK settings',
                    fontSize: 16,
                    color: Theme.of(context).colorScheme.error,
                  ),
                  const VSpace(16),
                  FlowyButton(
                    text: FlowyText.medium('Retry'),
                    onTap: () {
                      context.read<OpenAISDKBloc>().add(
                            const OpenAISDKEvent.started(),
                          );
                    },
                  ),
                ],
              ),
            );
          }

          return Container(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: const BorderRadius.all(Radius.circular(8.0)),
            ),
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 聊天配置区域
                _ChatConfigSection(submittedItems: state.submittedItems),
                const VSpace(16),
                // 嵌入配置区域
                _EmbeddingConfigSection(submittedItems: state.submittedItems),
                const VSpace(16),
                // 操作按钮
                _ActionButtons(
                  isEdited: state.isEdited,
                  submitState: state.submitState,
                  chatTestState: state.chatTestState,
                  embeddingTestState: state.embeddingTestState,
                  chatTestResult: state.chatTestResult,
                  embeddingTestResult: state.embeddingTestResult,
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}

/// 聊天配置区域
class _ChatConfigSection extends StatelessWidget {
  const _ChatConfigSection({
    required this.submittedItems,
  });

  final List<OpenAISDKSubmittedItem> submittedItems;

  String _getItemValue(OpenAISDKSettingType settingType) {
    return submittedItems
        .firstWhere(
          (item) => item.settingType == settingType,
          orElse: () => const OpenAISDKSubmittedItem(content: '', settingType: OpenAISDKSettingType.chatEndpoint),
        )
        .content;
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 区域标题
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_chatConfiguration.tr(),
          fontSize: 14,
          color: Theme.of(context).colorScheme.onSurface,
        ),
        const VSpace(12),
        
        // API 端点
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_apiEndpoint.tr(),
          value: _getItemValue(OpenAISDKSettingType.chatEndpoint),
          placeholder: kDefaultSDKChatEndpoint,
          tooltip: LocaleKeys.settings_aiPage_keys_chatEndpointTooltip.tr(),
          settingType: OpenAISDKSettingType.chatEndpoint,
        ),
        const VSpace(12),
        
        // API 密钥
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_apiKey.tr(),
          value: _getItemValue(OpenAISDKSettingType.chatApiKey),
          placeholder: LocaleKeys.settings_aiPage_keys_apiKeyPlaceholder.tr(),
          obscureText: true,
          tooltip: LocaleKeys.settings_aiPage_keys_apiKeyTooltip.tr(),
          settingType: OpenAISDKSettingType.chatApiKey,
        ),
        const VSpace(12),
        
        // 模型名称和类型（行布局）
        Row(
          children: [
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_modelName.tr(),
                value: _getItemValue(OpenAISDKSettingType.chatModel),
                placeholder: kDefaultSDKChatModel,
                tooltip: LocaleKeys.settings_aiPage_keys_chatModelTooltip.tr(),
                settingType: OpenAISDKSettingType.chatModel,
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_modelType.tr(),
                value: _getItemValue(OpenAISDKSettingType.chatModelType),
                placeholder: 'chat',
                tooltip: LocaleKeys.settings_aiPage_keys_modelTypeTooltip.tr(),
                settingType: OpenAISDKSettingType.chatModelType,
              ),
            ),
          ],
        ),
        const VSpace(12),
        
        // Tokens、温度和超时（行布局）
        Row(
          children: [
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_maxTokens.tr(),
                value: _getItemValue(OpenAISDKSettingType.chatMaxTokens),
                placeholder: kDefaultSDKMaxTokens.toString(),
                tooltip: LocaleKeys.settings_aiPage_keys_maxTokensTooltip.tr(),
                settingType: OpenAISDKSettingType.chatMaxTokens,
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_temperature.tr(),
                value: _getItemValue(OpenAISDKSettingType.chatTemperature),
                placeholder: kDefaultSDKTemperature.toString(),
                tooltip: LocaleKeys.settings_aiPage_keys_temperatureTooltip.tr(),
                settingType: OpenAISDKSettingType.chatTemperature,
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_requestTimeout.tr(),
                value: _getItemValue(OpenAISDKSettingType.chatTimeout),
                placeholder: kDefaultSDKTimeoutSeconds.toString(),
                tooltip: LocaleKeys.settings_aiPage_keys_timeoutTooltip.tr(),
                settingType: OpenAISDKSettingType.chatTimeout,
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildInputField({
    required BuildContext context,
    required String label,
    required String value,
    required OpenAISDKSettingType settingType,
    String? placeholder,
    String? tooltip,
    bool obscureText = false,
  }) {
    return SettingsInputField(
      label: label,
      value: value,
      placeholder: placeholder,
      tooltip: tooltip,
      obscureText: obscureText,
      hideActions: true, // 隐藏保存/取消按钮，使用统一的操作按钮
      onChanged: (newValue) {
        context.read<OpenAISDKBloc>().add(
              OpenAISDKEvent.onEdit(newValue, settingType),
            );
      },
    );
  }
}

/// 嵌入配置区域
class _EmbeddingConfigSection extends StatelessWidget {
  const _EmbeddingConfigSection({
    required this.submittedItems,
  });

  final List<OpenAISDKSubmittedItem> submittedItems;

  String _getItemValue(OpenAISDKSettingType settingType) {
    return submittedItems
        .firstWhere(
          (item) => item.settingType == settingType,
          orElse: () => const OpenAISDKSubmittedItem(content: '', settingType: OpenAISDKSettingType.embeddingEndpoint),
        )
        .content;
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 区域标题
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_embeddingConfiguration.tr(),
          fontSize: 14,
          color: Theme.of(context).colorScheme.onSurface,
        ),
        const VSpace(12),
        
        // API 端点
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_apiEndpoint.tr(),
          value: _getItemValue(OpenAISDKSettingType.embeddingEndpoint),
          placeholder: kDefaultSDKEmbeddingEndpoint,
          tooltip: LocaleKeys.settings_aiPage_keys_embeddingEndpointTooltip.tr(),
          settingType: OpenAISDKSettingType.embeddingEndpoint,
        ),
        const VSpace(12),
        
        // API 密钥
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_apiKey.tr(),
          value: _getItemValue(OpenAISDKSettingType.embeddingApiKey),
          placeholder: LocaleKeys.settings_aiPage_keys_apiKeyPlaceholder.tr(),
          obscureText: true,
          tooltip: LocaleKeys.settings_aiPage_keys_apiKeyTooltip.tr(),
          settingType: OpenAISDKSettingType.embeddingApiKey,
        ),
        const VSpace(12),
        
        // 模型名称
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_modelName.tr(),
          value: _getItemValue(OpenAISDKSettingType.embeddingModel),
          placeholder: kDefaultSDKEmbeddingModel,
          tooltip: LocaleKeys.settings_aiPage_keys_embeddingModelTooltip.tr(),
          settingType: OpenAISDKSettingType.embeddingModel,
        ),
      ],
    );
  }

  Widget _buildInputField({
    required BuildContext context,
    required String label,
    required String value,
    required OpenAISDKSettingType settingType,
    String? placeholder,
    String? tooltip,
    bool obscureText = false,
  }) {
    return SettingsInputField(
      label: label,
      value: value,
      placeholder: placeholder,
      tooltip: tooltip,
      obscureText: obscureText,
      hideActions: true, // 隐藏保存/取消按钮，使用统一的操作按钮
      onChanged: (newValue) {
        context.read<OpenAISDKBloc>().add(
              OpenAISDKEvent.onEdit(newValue, settingType),
            );
      },
    );
  }
}

/// 操作按钮区域
class _ActionButtons extends StatelessWidget {
  const _ActionButtons({
    required this.isEdited,
    required this.submitState,
    required this.chatTestState,
    required this.embeddingTestState,
    this.chatTestResult,
    this.embeddingTestResult,
  });

  final bool isEdited;
  final SubmitState submitState;
  final TestState chatTestState;
  final TestState embeddingTestState;
  final TestResultPB? chatTestResult;
  final TestResultPB? embeddingTestResult;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 保存按钮行
        Row(
          children: [
            // 保存按钮
            FlowyButton(
              text: FlowyText.medium(
                submitState == SubmitState.submitting
                    ? LocaleKeys.settings_aiPage_keys_saving.tr()
                    : LocaleKeys.settings_aiPage_keys_saveSettings.tr(),
              ),
              onTap: isEdited && submitState != SubmitState.submitting
                  ? () {
                      context.read<OpenAISDKBloc>().add(
                            const OpenAISDKEvent.submit(),
                          );
                    }
                  : null,
              leftIcon: submitState == SubmitState.submitting
                  ? const SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.save, size: 14),
            ),
            const HSpace(12),
            
            // 保存状态指示
            if (submitState == SubmitState.success) ...[
              const Icon(Icons.check_circle, color: Colors.green, size: 16),
              const HSpace(4),
              FlowyText.medium(
                LocaleKeys.settings_aiPage_keys_testSuccess.tr(),
                color: Colors.green,
              ),
            ] else if (submitState == SubmitState.error) ...[
              const Icon(Icons.error, color: Colors.red, size: 16),
              const HSpace(4),
              FlowyText.medium(
                LocaleKeys.settings_aiPage_keys_testFailed.tr(),
                color: Colors.red,
              ),
            ],
          ],
        ),
        const VSpace(16),
        
        // 测试按钮行
        Row(
          children: [
            // 测试聊天按钮
            FlowyButton(
              text: FlowyText.medium(
                chatTestState == TestState.testing
                    ? LocaleKeys.settings_aiPage_keys_testing.tr()
                    : LocaleKeys.settings_aiPage_keys_testChat.tr(),
              ),
              onTap: chatTestState != TestState.testing
                  ? () {
                      context.read<OpenAISDKBloc>().add(
                            const OpenAISDKEvent.testChat(),
                          );
                    }
                  : null,
              leftIcon: chatTestState == TestState.testing
                  ? const SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.chat, size: 14),
            ),
            const HSpace(12),
            
            // 测试嵌入按钮
            FlowyButton(
              text: FlowyText.medium(
                embeddingTestState == TestState.testing
                    ? LocaleKeys.settings_aiPage_keys_testing.tr()
                    : LocaleKeys.settings_aiPage_keys_testEmbedding.tr(),
              ),
              onTap: embeddingTestState != TestState.testing
                  ? () {
                      context.read<OpenAISDKBloc>().add(
                            const OpenAISDKEvent.testEmbedding(),
                          );
                    }
                  : null,
              leftIcon: embeddingTestState == TestState.testing
                  ? const SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.psychology, size: 14),
            ),
          ],
        ),
        
        // 测试结果显示
        if (chatTestResult != null || embeddingTestResult != null) ...[
          const VSpace(16),
          _TestResultsSection(
            chatTestResult: chatTestResult,
            embeddingTestResult: embeddingTestResult,
            chatTestState: chatTestState,
            embeddingTestState: embeddingTestState,
          ),
        ],
      ],
    );
  }
}

/// 测试结果显示区域
class _TestResultsSection extends StatelessWidget {
  const _TestResultsSection({
    required this.chatTestResult,
    required this.embeddingTestResult,
    required this.chatTestState,
    required this.embeddingTestState,
  });

  final TestResultPB? chatTestResult;
  final TestResultPB? embeddingTestResult;
  final TestState chatTestState;
  final TestState embeddingTestState;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withValues(alpha: 0.2),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
        FlowyText.medium(
          'Test Results', // 使用硬编码文本，因为这个key不存在
          fontSize: 14,
          color: Theme.of(context).colorScheme.onSurface,
        ),
          const VSpace(8),
          
          // 聊天测试结果
          if (chatTestResult != null) ...[
            _buildTestResult(
              context: context,
              title: 'Chat Test', // 使用硬编码文本
              result: chatTestResult!,
              testState: chatTestState,
            ),
            const VSpace(8),
          ],
          
          // 嵌入测试结果
          if (embeddingTestResult != null) ...[
            _buildTestResult(
              context: context,
              title: 'Embedding Test', // 使用硬编码文本
              result: embeddingTestResult!,
              testState: embeddingTestState,
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildTestResult({
    required BuildContext context,
    required String title,
    required TestResultPB result,
    required TestState testState,
  }) {
    final isSuccess = result.success;
    final statusColor = isSuccess ? Colors.green : Colors.red;
    final statusIcon = isSuccess ? Icons.check_circle : Icons.error;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(statusIcon, color: statusColor, size: 16),
            const HSpace(4),
            FlowyText.medium(
              title,
              color: statusColor,
            ),
            if (result.responseTimeMs.isNotEmpty) ...[
              const HSpace(8),
              FlowyText.regular(
                '(${result.responseTimeMs}ms)',
                color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
              ),
            ],
          ],
        ),
        if (result.errorMessage.isNotEmpty) ...[
          const VSpace(4),
          FlowyText.regular(
            result.errorMessage,
            color: Colors.red,
            fontSize: 12,
          ),
        ],
        if (result.serverResponse.isNotEmpty && isSuccess) ...[
          const VSpace(4),
          FlowyText.regular(
            'Server Response: ${result.serverResponse}', // 使用硬编码文本
            color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
            fontSize: 12,
            maxLines: 2,
          ),
        ],
      ],
    );
  }
}
