import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/openai_compatible_setting_bloc.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// OpenAI 兼容服务器设置面板
/// 包含聊天配置区域、嵌入配置区域和操作按钮
class OpenAICompatibleSetting extends StatelessWidget {
  const OpenAICompatibleSetting({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => OpenAICompatibleSettingBloc()
        ..add(const OpenAICompatibleSettingEvent.started()),
      child: BlocConsumer<OpenAICompatibleSettingBloc, OpenAICompatibleSettingState>(
        listener: (context, state) {
          // 处理保存成功的消息
          if (state.submitState == SubmitState.success) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: FlowyText.regular(
                  LocaleKeys.settings_aiPage_keys_settingsSaved.tr(),
                  color: Colors.white,
                ),
                backgroundColor: Colors.green,
                duration: const Duration(seconds: 2),
              ),
            );
          }
          
          // 处理保存失败的消息
          if (state.submitState == SubmitState.error) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: FlowyText.regular(
                  LocaleKeys.settings_aiPage_keys_settingsSaveFailed.tr(),
                  color: Colors.white,
                ),
                backgroundColor: Colors.red,
                duration: const Duration(seconds: 3),
              ),
            );
          }
        },
        builder: (context, state) {
          if (state.loadingState == LoadingState.loading) {
            return Container(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: const BorderRadius.all(Radius.circular(8.0)),
              ),
              padding: const EdgeInsets.all(12),
              child: const Center(
                child: CircularProgressIndicator.adaptive(),
              ),
            );
          }

          if (state.loadingState == LoadingState.error) {
            return Container(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: const BorderRadius.all(Radius.circular(8.0)),
              ),
              padding: const EdgeInsets.all(12),
              child: Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    FlowyText.medium(LocaleKeys.settings_aiPage_keys_loadConfigError.tr()),
                    const VSpace(8),
                    FlowyButton(
                      text: FlowyText.regular(LocaleKeys.button_retry.tr()),
                      onTap: () => context
                          .read<OpenAICompatibleSettingBloc>()
                          .add(const OpenAICompatibleSettingEvent.started()),
                    ),
                  ],
                ),
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

  final List<OpenAISubmittedItem> submittedItems;

  String _getItemValue(OpenAISettingType settingType) {
    return submittedItems
        .firstWhere(
          (item) => item.settingType == settingType,
          orElse: () => const OpenAISubmittedItem(content: '', settingType: OpenAISettingType.chatEndpoint),
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
          value: _getItemValue(OpenAISettingType.chatEndpoint),
          placeholder: kDefaultChatEndpoint,
          tooltip: LocaleKeys.settings_aiPage_keys_chatEndpointTooltip.tr(),
          settingType: OpenAISettingType.chatEndpoint,
        ),
        const VSpace(12),
        
        // API 密钥
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_apiKey.tr(),
          value: _getItemValue(OpenAISettingType.chatApiKey),
          placeholder: LocaleKeys.settings_aiPage_keys_apiKeyPlaceholder.tr(),
          obscureText: true,
          tooltip: LocaleKeys.settings_aiPage_keys_apiKeyTooltip.tr(),
          settingType: OpenAISettingType.chatApiKey,
        ),
        const VSpace(12),
        
        // 模型名称和类型（行布局）
        Row(
          children: [
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_modelName.tr(),
                value: _getItemValue(OpenAISettingType.chatModel),
                placeholder: kDefaultChatModel,
                tooltip: LocaleKeys.settings_aiPage_keys_chatModelTooltip.tr(),
                settingType: OpenAISettingType.chatModel,
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_modelType.tr(),
                value: _getItemValue(OpenAISettingType.chatModelType),
                placeholder: 'chat',
                tooltip: LocaleKeys.settings_aiPage_keys_modelTypeTooltip.tr(),
                settingType: OpenAISettingType.chatModelType,
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
                value: _getItemValue(OpenAISettingType.chatMaxTokens),
                placeholder: kDefaultMaxTokens.toString(),
                tooltip: LocaleKeys.settings_aiPage_keys_maxTokensTooltip.tr(),
                settingType: OpenAISettingType.chatMaxTokens,
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_temperature.tr(),
                value: _getItemValue(OpenAISettingType.chatTemperature),
                placeholder: kDefaultTemperature.toString(),
                tooltip: LocaleKeys.settings_aiPage_keys_temperatureTooltip.tr(),
                settingType: OpenAISettingType.chatTemperature,
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                context: context,
                label: LocaleKeys.settings_aiPage_keys_requestTimeout.tr(),
                value: _getItemValue(OpenAISettingType.chatTimeout),
                placeholder: kDefaultTimeoutSeconds.toString(),
                tooltip: LocaleKeys.settings_aiPage_keys_timeoutTooltip.tr(),
                settingType: OpenAISettingType.chatTimeout,
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
    required OpenAISettingType settingType,
    String? placeholder,
    String? tooltip,
    bool obscureText = false,
  }) {
    return SettingsInputField(
      label: label,
      value: value, // 使用value参数而不是textController
      placeholder: placeholder,
      tooltip: tooltip,
      obscureText: obscureText,
      hideActions: true, // 隐藏保存/取消按钮，使用统一的操作按钮
      onChanged: (newValue) {
        context.read<OpenAICompatibleSettingBloc>().add(
              OpenAICompatibleSettingEvent.onEdit(newValue, settingType),
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

  final List<OpenAISubmittedItem> submittedItems;

  String _getItemValue(OpenAISettingType settingType) {
    return submittedItems
        .firstWhere(
          (item) => item.settingType == settingType,
          orElse: () => const OpenAISubmittedItem(content: '', settingType: OpenAISettingType.embeddingEndpoint),
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
          value: _getItemValue(OpenAISettingType.embeddingEndpoint),
          placeholder: kDefaultEmbeddingEndpoint,
          tooltip: LocaleKeys.settings_aiPage_keys_embeddingEndpointTooltip.tr(),
          settingType: OpenAISettingType.embeddingEndpoint,
        ),
        const VSpace(12),
        
        // API 密钥
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_apiKey.tr(),
          value: _getItemValue(OpenAISettingType.embeddingApiKey),
          placeholder: LocaleKeys.settings_aiPage_keys_apiKeyPlaceholder.tr(),
          obscureText: true,
          tooltip: 'API 访问密钥，将被安全存储', // TODO: 使用国际化
          settingType: OpenAISettingType.embeddingApiKey,
        ),
        const VSpace(12),
        
        // 模型名称
        _buildInputField(
          context: context,
          label: LocaleKeys.settings_aiPage_keys_modelName.tr(),
          value: _getItemValue(OpenAISettingType.embeddingModel),
          placeholder: kDefaultEmbeddingModel,
          tooltip: LocaleKeys.settings_aiPage_keys_embeddingModelTooltip.tr(),
          settingType: OpenAISettingType.embeddingModel,
        ),
      ],
    );
  }

  Widget _buildInputField({
    required BuildContext context,
    required String label,
    required String value,
    required OpenAISettingType settingType,
    String? placeholder,
    String? tooltip,
    bool obscureText = false,
  }) {
    return SettingsInputField(
      label: label,
      value: value, // 使用value参数而不是textController
      placeholder: placeholder,
      tooltip: tooltip,
      obscureText: obscureText,
      hideActions: true, // 隐藏保存/取消按钮，使用统一的操作按钮
      onChanged: (newValue) {
        context.read<OpenAICompatibleSettingBloc>().add(
              OpenAICompatibleSettingEvent.onEdit(newValue, settingType),
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
      children: [
        // 测试结果显示
        if (chatTestResult != null && chatTestState != TestState.idle)
          _buildTestResultDisplay(
            context,
            '聊天测试',
            chatTestResult!,
            chatTestState,
          ),
        if (embeddingTestResult != null && embeddingTestState != TestState.idle)
          _buildTestResultDisplay(
            context,
            '嵌入测试',
            embeddingTestResult!,
            embeddingTestState,
          ),
        
        // 按钮行
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            // 测试聊天按钮
            FlowyButton(
              text: FlowyText.regular(
                chatTestState == TestState.testing 
                    ? LocaleKeys.settings_aiPage_keys_testing.tr() 
                    : LocaleKeys.settings_aiPage_keys_testChat.tr(),
                fontSize: 14,
              ),
              margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              isSelected: chatTestState == TestState.testing,
              onTap: chatTestState == TestState.testing 
                  ? null 
                  : () {
                      context.read<OpenAICompatibleSettingBloc>().add(
                            const OpenAICompatibleSettingEvent.testChat(),
                          );
                    },
            ),
            const HSpace(8),
            
            // 测试嵌入按钮
            FlowyButton(
              text: FlowyText.regular(
                embeddingTestState == TestState.testing 
                    ? LocaleKeys.settings_aiPage_keys_testing.tr() 
                    : LocaleKeys.settings_aiPage_keys_testEmbedding.tr(),
                fontSize: 14,
              ),
              margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              isSelected: embeddingTestState == TestState.testing,
              onTap: embeddingTestState == TestState.testing 
                  ? null 
                  : () {
                      context.read<OpenAICompatibleSettingBloc>().add(
                            const OpenAICompatibleSettingEvent.testEmbedding(),
                          );
                    },
            ),
            const HSpace(8),
            
            // 保存按钮
            FlowyButton(
              text: FlowyText.regular(
                submitState == SubmitState.submitting 
                    ? LocaleKeys.settings_aiPage_keys_saving.tr() 
                    : LocaleKeys.settings_aiPage_keys_saveSettings.tr(),
                fontSize: 14,
                color: isEdited || submitState == SubmitState.submitting
                    ? Theme.of(context).colorScheme.onPrimary
                    : Theme.of(context).colorScheme.onSurface.withOpacity(0.6),
              ),
              margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
              backgroundColor: isEdited || submitState == SubmitState.submitting
                  ? Theme.of(context).colorScheme.primary
                  : Theme.of(context).colorScheme.surface,
              hoverColor: isEdited
                  ? Theme.of(context).colorScheme.primary.withAlpha(200)
                  : null,
              isSelected: submitState == SubmitState.submitting,
              onTap: !isEdited || submitState == SubmitState.submitting 
                  ? null 
                  : () {
                      context.read<OpenAICompatibleSettingBloc>().add(
                            const OpenAICompatibleSettingEvent.submit(),
                          );
                    },
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildTestResultDisplay(
    BuildContext context,
    String title,
    TestResultPB result,
    TestState state,
  ) {
    if (state == TestState.idle) return const SizedBox.shrink();

    Color backgroundColor;
    Color textColor;
    IconData icon;
    String message;

    if (state == TestState.testing) {
      backgroundColor = Theme.of(context).colorScheme.surfaceContainerHighest;
      textColor = Theme.of(context).colorScheme.onSurface;
      icon = Icons.hourglass_empty;
      message = LocaleKeys.settings_aiPage_keys_testingConnection.tr();
    } else if (result.success) {
      backgroundColor = Colors.green.withOpacity(0.1);
      textColor = Colors.green.shade700;
      icon = Icons.check_circle;
      message = LocaleKeys.settings_aiPage_keys_testSuccess.tr();
    } else {
      backgroundColor = Colors.red.withOpacity(0.1);
      textColor = Colors.red.shade700;
      icon = Icons.error;
      message = '${LocaleKeys.settings_aiPage_keys_testFailed.tr()}: ${_getLocalizedErrorMessage(result.errorMessage)}';
    }

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: backgroundColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: textColor.withOpacity(0.3),
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(icon, color: textColor, size: 16),
          const HSpace(8),
          Expanded(
            child: FlowyText.medium(
              message,
              color: textColor,
              fontSize: 13,
            ),
          ),
          if (state != TestState.testing)
            GestureDetector(
              onTap: () {
                // 清除测试结果
                context.read<OpenAICompatibleSettingBloc>().add(
                      const OpenAICompatibleSettingEvent.started(),
                    );
              },
              child: Icon(
                Icons.close,
                color: textColor.withOpacity(0.7),
                size: 16,
              ),
            ),
        ],
      ),
    );
  }

  /// 获取本地化的错误消息
  String _getLocalizedErrorMessage(String errorMessage) {
    // 根据错误消息内容返回对应的本地化文案
    if (errorMessage.toLowerCase().contains('connection') || 
        errorMessage.toLowerCase().contains('connect') ||
        errorMessage.toLowerCase().contains('network')) {
      return LocaleKeys.settings_aiPage_keys_connectionError.tr();
    } else if (errorMessage.toLowerCase().contains('auth') || 
               errorMessage.toLowerCase().contains('unauthorized') ||
               errorMessage.toLowerCase().contains('401')) {
      return LocaleKeys.settings_aiPage_keys_authenticationError.tr();
    } else if (errorMessage.toLowerCase().contains('model') || 
               errorMessage.toLowerCase().contains('404')) {
      return LocaleKeys.settings_aiPage_keys_modelNotFoundError.tr();
    } else if (errorMessage.toLowerCase().contains('timeout')) {
      return LocaleKeys.settings_aiPage_keys_timeoutError.tr();
    } else if (errorMessage.toLowerCase().contains('rate') || 
               errorMessage.toLowerCase().contains('limit') ||
               errorMessage.toLowerCase().contains('429')) {
      return LocaleKeys.settings_aiPage_keys_rateLimitError.tr();
    } else if (errorMessage.toLowerCase().contains('server') || 
               errorMessage.toLowerCase().contains('500') ||
               errorMessage.toLowerCase().contains('502') ||
               errorMessage.toLowerCase().contains('503')) {
      return LocaleKeys.settings_aiPage_keys_serverError.tr();
    } else if (errorMessage.toLowerCase().contains('endpoint') || 
               errorMessage.toLowerCase().contains('url')) {
      return LocaleKeys.settings_aiPage_keys_invalidEndpointError.tr();
    } else {
      // 如果无法匹配到特定错误类型，返回原始错误消息
      return errorMessage;
    }
  }
}
