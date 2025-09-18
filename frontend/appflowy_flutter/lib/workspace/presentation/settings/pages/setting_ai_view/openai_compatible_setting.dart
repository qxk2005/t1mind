import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// OpenAI 兼容服务器设置面板骨架
/// 包含聊天配置区域、嵌入配置区域和操作按钮
class OpenAICompatibleSetting extends StatefulWidget {
  const OpenAICompatibleSetting({super.key});

  @override
  State<OpenAICompatibleSetting> createState() => _OpenAICompatibleSettingState();
}

class _OpenAICompatibleSettingState extends State<OpenAICompatibleSetting> {
  // 临时状态管理 - 聊天配置
  final TextEditingController _chatEndpointController = TextEditingController(
    text: 'https://api.openai.com/v1/chat/completions',
  );
  final TextEditingController _chatApiKeyController = TextEditingController();
  final TextEditingController _chatModelController = TextEditingController(
    text: 'gpt-3.5-turbo',
  );
  final TextEditingController _chatModelTypeController = TextEditingController(
    text: 'openai',
  );
  final TextEditingController _chatMaxTokensController = TextEditingController(
    text: '4096',
  );
  final TextEditingController _chatTemperatureController = TextEditingController(
    text: '0.7',
  );
  final TextEditingController _chatTimeoutController = TextEditingController(
    text: '30',
  );

  // 临时状态管理 - 嵌入配置
  final TextEditingController _embeddingEndpointController = TextEditingController(
    text: 'https://api.openai.com/v1/embeddings',
  );
  final TextEditingController _embeddingApiKeyController = TextEditingController();
  final TextEditingController _embeddingModelController = TextEditingController(
    text: 'text-embedding-ada-002',
  );

  @override
  void dispose() {
    _chatEndpointController.dispose();
    _chatApiKeyController.dispose();
    _chatModelController.dispose();
    _chatModelTypeController.dispose();
    _chatMaxTokensController.dispose();
    _chatTemperatureController.dispose();
    _chatTimeoutController.dispose();
    _embeddingEndpointController.dispose();
    _embeddingApiKeyController.dispose();
    _embeddingModelController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
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
          _ChatConfigSection(
            endpointController: _chatEndpointController,
            apiKeyController: _chatApiKeyController,
            modelController: _chatModelController,
            modelTypeController: _chatModelTypeController,
            maxTokensController: _chatMaxTokensController,
            temperatureController: _chatTemperatureController,
            timeoutController: _chatTimeoutController,
          ),
          const VSpace(16),
          // 嵌入配置区域
          _EmbeddingConfigSection(
            endpointController: _embeddingEndpointController,
            apiKeyController: _embeddingApiKeyController,
            modelController: _embeddingModelController,
          ),
          const VSpace(16),
          // 操作按钮
          _ActionButtons(),
        ],
      ),
    );
  }
}

/// 聊天配置区域
class _ChatConfigSection extends StatelessWidget {
  const _ChatConfigSection({
    required this.endpointController,
    required this.apiKeyController,
    required this.modelController,
    required this.modelTypeController,
    required this.maxTokensController,
    required this.temperatureController,
    required this.timeoutController,
  });

  final TextEditingController endpointController;
  final TextEditingController apiKeyController;
  final TextEditingController modelController;
  final TextEditingController modelTypeController;
  final TextEditingController maxTokensController;
  final TextEditingController temperatureController;
  final TextEditingController timeoutController;

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
          label: LocaleKeys.settings_aiPage_keys_apiEndpoint.tr(),
          controller: endpointController,
          placeholder: 'https://api.openai.com/v1/chat/completions',
          tooltip: '聊天 API 的完整端点 URL', // TODO: 使用国际化
        ),
        const VSpace(12),
        
        // API 密钥
        _buildInputField(
          label: LocaleKeys.settings_aiPage_keys_apiKey.tr(),
          controller: apiKeyController,
          placeholder: LocaleKeys.settings_aiPage_keys_apiKeyPlaceholder.tr(),
          obscureText: true,
          tooltip: 'API 访问密钥，将被安全存储', // TODO: 使用国际化
        ),
        const VSpace(12),
        
        // 模型名称和类型（行布局）
        Row(
          children: [
            Expanded(
              child: _buildInputField(
                label: LocaleKeys.settings_aiPage_keys_modelName.tr(),
                controller: modelController,
                placeholder: LocaleKeys.settings_aiPage_keys_modelNamePlaceholder.tr(),
                tooltip: '要使用的聊天模型名称', // TODO: 使用国际化
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                label: LocaleKeys.settings_aiPage_keys_modelType.tr(),
                controller: modelTypeController,
                placeholder: 'openai',
                tooltip: '模型提供商类型', // TODO: 使用国际化
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
                label: LocaleKeys.settings_aiPage_keys_maxTokens.tr(),
                controller: maxTokensController,
                placeholder: '4096',
                tooltip: '响应的最大 token 数量', // TODO: 使用国际化
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                label: LocaleKeys.settings_aiPage_keys_temperature.tr(),
                controller: temperatureController,
                placeholder: '0.7',
                tooltip: '控制响应的随机性（0.0-2.0）', // TODO: 使用国际化
              ),
            ),
            const HSpace(12),
            Expanded(
              child: _buildInputField(
                label: LocaleKeys.settings_aiPage_keys_requestTimeout.tr(),
                controller: timeoutController,
                placeholder: '30',
                tooltip: '请求超时时间', // TODO: 使用国际化
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildInputField({
    required String label,
    required TextEditingController controller,
    String? placeholder,
    String? tooltip,
    bool obscureText = false,
  }) {
    return SettingsInputField(
      label: label,
      textController: controller,
      placeholder: placeholder,
      tooltip: tooltip,
      obscureText: obscureText,
      hideActions: true, // 隐藏保存/取消按钮，使用统一的操作按钮
      onChanged: (value) {
        // TODO: 在后续任务中连接到 BLoC 状态管理
      },
    );
  }
}

/// 嵌入配置区域
class _EmbeddingConfigSection extends StatelessWidget {
  const _EmbeddingConfigSection({
    required this.endpointController,
    required this.apiKeyController,
    required this.modelController,
  });

  final TextEditingController endpointController;
  final TextEditingController apiKeyController;
  final TextEditingController modelController;

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
          label: LocaleKeys.settings_aiPage_keys_apiEndpoint.tr(),
          controller: endpointController,
          placeholder: 'https://api.openai.com/v1/embeddings',
          tooltip: '嵌入 API 的完整端点 URL', // TODO: 使用国际化
        ),
        const VSpace(12),
        
        // API 密钥
        _buildInputField(
          label: LocaleKeys.settings_aiPage_keys_apiKey.tr(),
          controller: apiKeyController,
          placeholder: LocaleKeys.settings_aiPage_keys_apiKeyPlaceholder.tr(),
          obscureText: true,
          tooltip: 'API 访问密钥，将被安全存储', // TODO: 使用国际化
        ),
        const VSpace(12),
        
        // 模型名称
        _buildInputField(
          label: LocaleKeys.settings_aiPage_keys_modelName.tr(),
          controller: modelController,
          placeholder: LocaleKeys.settings_aiPage_keys_embeddingModelPlaceholder.tr(),
          tooltip: '要使用的嵌入模型名称', // TODO: 使用国际化
        ),
      ],
    );
  }

  Widget _buildInputField({
    required String label,
    required TextEditingController controller,
    String? placeholder,
    String? tooltip,
    bool obscureText = false,
  }) {
    return SettingsInputField(
      label: label,
      textController: controller,
      placeholder: placeholder,
      tooltip: tooltip,
      obscureText: obscureText,
      hideActions: true, // 隐藏保存/取消按钮，使用统一的操作按钮
      onChanged: (value) {
        // TODO: 在后续任务中连接到 BLoC 状态管理
      },
    );
  }
}

/// 操作按钮区域
class _ActionButtons extends StatelessWidget {
  const _ActionButtons();

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        // 测试聊天按钮
        FlowyButton(
          text: FlowyText.regular(
            LocaleKeys.settings_aiPage_keys_testChat.tr(),
            fontSize: 14,
          ),
          margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          onTap: () {
            // TODO: 在后续任务中实现测试功能
            _showTestDialog(context, '聊天测试', '聊天连接测试功能将在后续任务中实现');
          },
        ),
        const HSpace(8),
        
        // 测试嵌入按钮
        FlowyButton(
          text: FlowyText.regular(
            LocaleKeys.settings_aiPage_keys_testEmbedding.tr(),
            fontSize: 14,
          ),
          margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          onTap: () {
            // TODO: 在后续任务中实现测试功能
            _showTestDialog(context, '嵌入测试', '嵌入连接测试功能将在后续任务中实现');
          },
        ),
        const HSpace(8),
        
        // 保存按钮
        FlowyButton(
          text: FlowyText.regular(
            LocaleKeys.settings_aiPage_keys_saveSettings.tr(),
            fontSize: 14,
            color: Theme.of(context).colorScheme.onPrimary,
          ),
          margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
          backgroundColor: Theme.of(context).colorScheme.primary,
          hoverColor: Theme.of(context).colorScheme.primary.withAlpha(200),
          onTap: () {
            // TODO: 在后续任务中连接到 BLoC 保存功能
            _showSaveDialog(context);
          },
        ),
      ],
    );
  }

  void _showTestDialog(BuildContext context, String title, String message) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  void _showSaveDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('保存配置'),
        content: const Text('配置保存功能将在后续任务中实现'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }
}
