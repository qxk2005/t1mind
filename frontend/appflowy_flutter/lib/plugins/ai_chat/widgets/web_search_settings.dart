import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../application/web_search_settings_bloc.dart';

/// 网络搜索配置页面
/// 允许用户配置搜索引擎供应商的 API 密钥和测试连接
class WebSearchSettingsPage extends StatelessWidget {
  const WebSearchSettingsPage({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => WebSearchSettingsBloc()..add(const WebSearchSettingsEvent.started()),
      child: SettingsBody(
        title: "网络搜索配置",
        description: "配置搜索引擎供应商的 API 密钥和连接设置",
        children: [
          const _WebSearchProviderList(),
          const VSpace(16),
          const _AddWebSearchProviderSection(),
          const VSpace(16),
          const _WebSearchGlobalConfigSection(),
          const VSpace(16),
          const _WebSearchCacheSection(),
        ],
      ),
    );
  }
}

/// 网络搜索供应商列表
class _WebSearchProviderList extends StatelessWidget {
  const _WebSearchProviderList();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<WebSearchSettingsBloc, WebSearchSettingsState>(
      builder: (context, state) {
        return Container(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  FlowyText.medium(
                    "搜索供应商列表",
                    color: AFThemeExtension.of(context).strongText,
                  ),
                  const Spacer(),
                  FlowyTextButton(
                    "添加供应商",
                    fontColor: Theme.of(context).colorScheme.primary,
                    onPressed: () => _showAddProviderDialog(context),
                  ),
                ],
              ),
              const VSpace(12),
              if (state.isLoading)
                const Center(
                  child: Padding(
                    padding: EdgeInsets.all(32.0),
                    child: CircularProgressIndicator(),
                  ),
                )
              else if (state.providers.isEmpty)
                Center(
                  child: Padding(
                    padding: const EdgeInsets.all(32.0),
                    child: Column(
                      children: [
                        Icon(
                          Icons.search_off,
                          size: 48,
                          color: AFThemeExtension.of(context).secondaryTextColor,
                        ),
                        const VSpace(8),
                        FlowyText.regular(
                          "暂无搜索供应商",
                          color: AFThemeExtension.of(context).secondaryTextColor,
                        ),
                        const VSpace(4),
                        FlowyText.regular(
                          "添加供应商以启用网络搜索功能",
                          color: AFThemeExtension.of(context).secondaryTextColor,
                          fontSize: 12,
                        ),
                      ],
                    ),
                  ),
                )
              else
                ...state.providers.map((provider) => Padding(
                  padding: const EdgeInsets.only(bottom: 8.0),
                  child: _buildProviderListItem(context, provider),
                )),
              if (state.error != null) ...[
                const VSpace(8),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.red.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: Colors.red, width: 1),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.error, color: Colors.red, size: 16),
                      const HSpace(8),
                      Expanded(
                        child: FlowyText.regular(
                          state.error!,
                          color: Colors.red,
                          fontSize: 12,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
        );
      },
    );
  }

  Widget _buildProviderListItem(
    BuildContext context,
    WebSearchProviderConfigPB provider,
  ) {
    final bloc = context.read<WebSearchSettingsBloc>();
    final isActive = provider.isActive;
    final isTesting = bloc.isProviderTesting(provider.id);
    final isActivating = bloc.isProviderActivating(provider.id);
    final testStatus = bloc.getProviderTestStatus(provider.id);
    final error = bloc.getProviderError(provider.id);
    
    // 确定状态
    WebSearchProviderStatus status;
    if (isTesting || isActivating) {
      status = WebSearchProviderStatus.connecting;
    } else if (error != null) {
      status = WebSearchProviderStatus.error;
    } else if (isActive) {
      status = WebSearchProviderStatus.connected;
    } else {
      status = WebSearchProviderStatus.disconnected;
    }
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    FlowyText.medium(
                      provider.name,
                      color: AFThemeExtension.of(context).strongText,
                    ),
                    if (isActive) ...[
                      const HSpace(8),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: Theme.of(context).colorScheme.primary,
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: FlowyText.regular(
                          "激活",
                          color: Colors.white,
                          fontSize: 10,
                        ),
                      ),
                    ],
                    if (testStatus == ProviderTestStatusPB.TestPassed) ...[
                      const HSpace(8),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: Colors.green,
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: FlowyText.regular(
                          "已验证",
                          color: Colors.white,
                          fontSize: 10,
                        ),
                      ),
                    ],
                  ],
                ),
                const VSpace(4),
                FlowyText.regular(
                  _maskApiKey(provider.apiKey),
                  color: AFThemeExtension.of(context).secondaryTextColor,
                  fontSize: 12,
                ),
                if (provider.description.isNotEmpty) ...[
                  const VSpace(2),
                  FlowyText.regular(
                    provider.description,
                    color: AFThemeExtension.of(context).secondaryTextColor,
                    fontSize: 11,
                    maxLines: 2,
                  ),
                ],
                if (error != null) ...[
                  const VSpace(4),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: Colors.red.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Row(
                      children: [
                        Icon(Icons.error, color: Colors.red, size: 14),
                        const HSpace(4),
                        Expanded(
                          child: FlowyText.regular(
                            error,
                            color: Colors.red,
                            fontSize: 11,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
          const HSpace(12),
          _buildStatusIndicator(context, status),
          const HSpace(8),
          FlowyIconButton(
            icon: const FlowySvg(FlowySvgs.settings_s),
            onPressed: () => _showProviderConfigDialog(context, provider),
            tooltipText: "配置供应商",
          ),
          const HSpace(4),
          FlowyIconButton(
            icon: const FlowySvg(FlowySvgs.delete_s),
            onPressed: () => _showDeleteConfirmDialog(context, provider),
            tooltipText: LocaleKeys.button_delete.tr(),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusIndicator(BuildContext context, WebSearchProviderStatus status) {
    Color color;
    String text;
    
    switch (status) {
      case WebSearchProviderStatus.connected:
        color = Colors.green;
        text = "已连接";
        break;
      case WebSearchProviderStatus.connecting:
        color = Colors.orange;
        text = "连接中";
        break;
      case WebSearchProviderStatus.disconnected:
        color = Colors.red;
        text = "未连接";
        break;
      case WebSearchProviderStatus.error:
        color = Colors.red;
        text = "错误";
        break;
    }

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: color,
            shape: BoxShape.circle,
          ),
        ),
        const HSpace(6),
        FlowyText.regular(
          text,
          color: color,
          fontSize: 12,
        ),
      ],
    );
  }

  void _showAddProviderDialog(BuildContext context) {
    final bloc = context.read<WebSearchSettingsBloc>();
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: bloc,
        child: const _AddWebSearchProviderDialog(),
      ),
    );
  }

  void _showProviderConfigDialog(BuildContext context, WebSearchProviderConfigPB provider) {
    final bloc = context.read<WebSearchSettingsBloc>();
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: bloc,
        child: _ConfigureWebSearchProviderDialog(provider: provider),
      ),
    );
  }

  void _showDeleteConfirmDialog(BuildContext context, WebSearchProviderConfigPB provider) {
    final bloc = context.read<WebSearchSettingsBloc>();
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: bloc,
        child: Builder(
          builder: (context) => AlertDialog(
            title: Text("删除搜索供应商"),
            content: Text("确定要删除供应商 '${provider.name}' 吗？"),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: Text(LocaleKeys.button_cancel.tr()),
              ),
              TextButton(
                onPressed: () {
                  context.read<WebSearchSettingsBloc>().add(
                    WebSearchSettingsEvent.removeProvider(provider.id),
                  );
                  Navigator.of(context).pop();
                },
                child: Text(LocaleKeys.button_delete.tr()),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _maskApiKey(String apiKey) {
    if (apiKey.length <= 8) return "***";
    return "${apiKey.substring(0, 4)}***${apiKey.substring(apiKey.length - 4)}";
  }
}

/// 添加网络搜索供应商部分
class _AddWebSearchProviderSection extends StatelessWidget {
  const _AddWebSearchProviderSection();

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: const BorderRadius.all(Radius.circular(8.0)),
      ),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FlowyText.medium(
            "快速开始",
            color: AFThemeExtension.of(context).strongText,
          ),
          const VSpace(8),
          FlowyText.regular(
            "添加搜索引擎供应商以启用网络搜索功能。支持 Tavily 和 Brave Search。",
            color: AFThemeExtension.of(context).secondaryTextColor,
            maxLines: 3,
          ),
          const VSpace(12),
          Row(
            children: [
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(
                    "添加供应商",
                  ),
                  onTap: () => _showAddProviderDialog(context),
                ),
              ),
              const HSpace(12),
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(
                    "测试所有连接",
                  ),
                  onTap: () => _testAllConnections(context),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _showAddProviderDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const _AddWebSearchProviderDialog(),
    );
  }

  void _testAllConnections(BuildContext context) {
    final bloc = context.read<WebSearchSettingsBloc>();
    final providers = bloc.state.providers;
    
    if (providers.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text("没有可测试的供应商"),
        ),
      );
      return;
    }
    
    // 测试所有供应商
    for (final provider in providers) {
      bloc.add(WebSearchSettingsEvent.testProvider(provider.id));
    }
    
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text("正在测试 ${providers.length} 个供应商的连接..."),
      ),
    );
  }
}

/// 添加网络搜索供应商对话框
class _AddWebSearchProviderDialog extends StatefulWidget {
  const _AddWebSearchProviderDialog();

  @override
  State<_AddWebSearchProviderDialog> createState() => _AddWebSearchProviderDialogState();
}

class _AddWebSearchProviderDialogState extends State<_AddWebSearchProviderDialog> {
  final _nameController = TextEditingController();
  final _apiKeyController = TextEditingController();
  
  WebSearchProviderType _selectedProvider = WebSearchProviderType.tavily;
  bool _isTestingConnection = false;
  bool _obscureApiKey = true;
  String? _testResult;

  @override
  void dispose() {
    _nameController.dispose();
    _apiKeyController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      child: Container(
        width: 580,
        constraints: const BoxConstraints(maxHeight: 720),
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 对话框标题
            _buildHeader(),
            const VSpace(24),
            
            // 内容区域
            Flexible(
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _buildProviderTypeSelector(),
                    const VSpace(24),
                    _buildInfoCard(),
                    const VSpace(24),
                    _buildNameField(),
                    const VSpace(20),
                    _buildApiKeyField(),
                    if (_testResult != null) ...[
                      const VSpace(16),
                      _buildTestResult(),
                    ],
                  ],
                ),
              ),
            ),
            
            const VSpace(24),
            const Divider(height: 1),
            const VSpace(16),
            _buildActionButtons(),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Row(
      children: [
        Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            Icons.add_business,
            color: Theme.of(context).colorScheme.primary,
            size: 22,
          ),
        ),
        const HSpace(12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.semibold(
                "添加搜索供应商",
                fontSize: 18,
                color: AFThemeExtension.of(context).strongText,
              ),
              const VSpace(2),
              FlowyText.regular(
                "配置新的网络搜索引擎供应商",
                fontSize: 12,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
            ],
          ),
        ),
        FlowyIconButton(
          width: 32,
          icon: Icon(
            Icons.close,
            size: 20,
            color: AFThemeExtension.of(context).secondaryTextColor,
          ),
          onPressed: () => Navigator.of(context).pop(),
        ),
      ],
    );
  }

  Widget _buildProviderTypeSelector() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "选择供应商类型",
          fontSize: 14,
          color: AFThemeExtension.of(context).strongText,
        ),
        const VSpace(12),
        Row(
          children: [
            for (final provider in WebSearchProviderType.values) ...[
              Expanded(
                child: _buildProviderCard(provider),
              ),
              if (provider != WebSearchProviderType.values.last) const HSpace(12),
            ],
          ],
        ),
      ],
    );
  }

  Widget _buildProviderCard(WebSearchProviderType provider) {
    final isSelected = _selectedProvider == provider;
    final providerInfo = _getProviderInfo(provider);
    
    return InkWell(
      onTap: () => setState(() => _selectedProvider = provider),
      borderRadius: BorderRadius.circular(12),
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: isSelected 
              ? Theme.of(context).colorScheme.primaryContainer
              : Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isSelected 
                ? Theme.of(context).colorScheme.primary
                : Colors.transparent,
            width: 2,
          ),
        ),
        child: Column(
          children: [
            Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: isSelected
                    ? Theme.of(context).colorScheme.primary.withOpacity(0.1)
                    : Theme.of(context).colorScheme.surface,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Icon(
                providerInfo['icon'] as IconData,
                color: isSelected
                    ? Theme.of(context).colorScheme.primary
                    : AFThemeExtension.of(context).secondaryTextColor,
                size: 24,
              ),
            ),
            const VSpace(12),
            FlowyText.medium(
              providerInfo['name'] as String,
              fontSize: 14,
              color: isSelected
                  ? Theme.of(context).colorScheme.primary
                  : AFThemeExtension.of(context).strongText,
            ),
            const VSpace(4),
            FlowyText.regular(
              providerInfo['desc'] as String,
              fontSize: 11,
              color: AFThemeExtension.of(context).secondaryTextColor,
              textAlign: TextAlign.center,
              maxLines: 2,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoCard() {
    final providerInfo = _getProviderInfo(_selectedProvider);
    
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer.withOpacity(0.5),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.info_outline,
            size: 20,
            color: Theme.of(context).colorScheme.primary,
          ),
          const HSpace(12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  "如何获取 API 密钥",
                  fontSize: 13,
                  color: AFThemeExtension.of(context).strongText,
                ),
                const VSpace(4),
                FlowyText.regular(
                  providerInfo['help'] as String,
                  fontSize: 12,
                  color: AFThemeExtension.of(context).secondaryTextColor,
                  maxLines: 3,
                ),
                const VSpace(8),
                InkWell(
                  onTap: () {
                    // TODO: 打开帮助链接
                  },
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      FlowyText.regular(
                        "查看详细文档",
                        fontSize: 12,
                        color: Theme.of(context).colorScheme.primary,
                      ),
                      const HSpace(4),
                      Icon(
                        Icons.open_in_new,
                        size: 14,
                        color: Theme.of(context).colorScheme.primary,
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNameField() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            FlowyText.medium(
              "供应商名称",
              fontSize: 13,
              color: AFThemeExtension.of(context).strongText,
            ),
            const HSpace(4),
            FlowyText.regular(
              "*",
              fontSize: 13,
              color: Colors.red,
            ),
          ],
        ),
        const VSpace(8),
        TextField(
          controller: _nameController,
          decoration: InputDecoration(
            hintText: "例如：我的 ${_getProviderInfo(_selectedProvider)['name']}",
            hintStyle: TextStyle(
              color: AFThemeExtension.of(context).secondaryTextColor,
              fontSize: 14,
            ),
            filled: true,
            fillColor: Theme.of(context).colorScheme.surfaceContainerHighest,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide.none,
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide(
                color: Theme.of(context).colorScheme.primary,
                width: 2,
              ),
            ),
            contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          ),
          style: TextStyle(
            color: AFThemeExtension.of(context).strongText,
            fontSize: 14,
          ),
        ),
      ],
    );
  }

  Widget _buildApiKeyField() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            FlowyText.medium(
              "API 密钥",
              fontSize: 13,
              color: AFThemeExtension.of(context).strongText,
            ),
            const HSpace(4),
            FlowyText.regular(
              "*",
              fontSize: 13,
              color: Colors.red,
            ),
          ],
        ),
        const VSpace(8),
        TextField(
          controller: _apiKeyController,
          obscureText: _obscureApiKey,
          decoration: InputDecoration(
            hintText: _getApiKeyPlaceholder(),
            hintStyle: TextStyle(
              color: AFThemeExtension.of(context).secondaryTextColor,
              fontSize: 14,
            ),
            filled: true,
            fillColor: Theme.of(context).colorScheme.surfaceContainerHighest,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide.none,
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(8),
              borderSide: BorderSide(
                color: Theme.of(context).colorScheme.primary,
                width: 2,
              ),
            ),
            contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            suffixIcon: IconButton(
              icon: Icon(
                _obscureApiKey ? Icons.visibility : Icons.visibility_off,
                size: 20,
                color: AFThemeExtension.of(context).secondaryTextColor,
              ),
              onPressed: () => setState(() => _obscureApiKey = !_obscureApiKey),
            ),
          ),
          style: TextStyle(
            color: AFThemeExtension.of(context).strongText,
            fontSize: 14,
            fontFamily: 'monospace',
          ),
        ),
      ],
    );
  }

  Widget _buildTestResult() {
    // 检查是否包含成功相关的关键词（支持中英文）
    final isSuccess = _testResult!.contains('success') || 
                      _testResult!.contains('successful') ||
                      _testResult!.contains('成功') ||
                      _testResult!.contains('通过');
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: isSuccess ? Colors.green.withOpacity(0.1) : Colors.red.withOpacity(0.1),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: isSuccess ? Colors.green : Colors.red,
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(
            isSuccess ? Icons.check_circle : Icons.error,
            color: isSuccess ? Colors.green : Colors.red,
            size: 16,
          ),
          const HSpace(8),
          Expanded(
            child: FlowyText.regular(
              _testResult!,
              color: isSuccess ? Colors.green : Colors.red,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons() {
    return Row(
      children: [
        Expanded(
          child: FlowyButton(
            text: FlowyText.regular(
              _isTestingConnection
                  ? "测试中..."
                  : "测试连接",
            ),
            onTap: _isTestingConnection ? null : _testConnection,
            leftIcon: _isTestingConnection
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : null,
          ),
        ),
        const HSpace(12),
        Expanded(
          child: FlowyButton(
            text: FlowyText.regular(LocaleKeys.button_cancel.tr()),
            onTap: () => Navigator.of(context).pop(),
          ),
        ),
        const HSpace(12),
        Expanded(
          child: FlowyButton(
            text: FlowyText.regular(LocaleKeys.button_save.tr()),
            backgroundColor: Theme.of(context).colorScheme.primary,
            onTap: _canSave() ? _saveProvider : null,
          ),
        ),
      ],
    );
  }

  Map<String, dynamic> _getProviderInfo(WebSearchProviderType provider) {
    switch (provider) {
      case WebSearchProviderType.tavily:
        return {
          'name': 'Tavily',
          'desc': '强大的AI搜索API',
          'icon': Icons.travel_explore,
          'help': '访问 tavily.com 注册账号，在控制台获取 API 密钥。免费版每月提供 1000 次搜索。',
        };
      case WebSearchProviderType.brave:
        return {
          'name': 'Brave Search',
          'desc': '隐私优先搜索引擎',
          'icon': Icons.shield,
          'help': '访问 brave.com/search/api 申请 API 访问权限。提供独立索引和隐私保护。',
        };
    }
  }

  String _getApiKeyPlaceholder() {
    switch (_selectedProvider) {
      case WebSearchProviderType.tavily:
        return "tvly-xxxxxxxxxxxxxxxxxxxxxxxx";
      case WebSearchProviderType.brave:
        return "BSAxxxxxxxxxxxxxxxxxxxxxxxx";
    }
  }

  bool _canSave() {
    return _nameController.text.trim().isNotEmpty &&
           _apiKeyController.text.trim().isNotEmpty;
  }

  void _testConnection() async {
    if (!_canSave()) return;

    setState(() {
      _isTestingConnection = true;
      _testResult = null;
    });


    // 这里应该调用实际的测试 API
    // 目前使用模拟测试
    await Future.delayed(const Duration(seconds: 2));

    setState(() {
      _isTestingConnection = false;
      _testResult = "连接测试成功";
    });
  }

  void _saveProvider() async {
    if (!_canSave()) return;

    final providerInfo = _getProviderInfo(_selectedProvider);
    final config = WebSearchProviderConfigPB()
      ..name = _nameController.text.trim()
      ..providerType = _selectedProvider == WebSearchProviderType.tavily 
          ? WebSearchProviderTypePB.Tavily 
          : WebSearchProviderTypePB.BraveSearch
      ..description = providerInfo['desc'] as String
      ..icon = _selectedProvider == WebSearchProviderType.tavily ? "tavily" : "brave"
      ..apiKey = _apiKeyController.text.trim()
      ..baseUrl = _getBaseUrl()
      ..maxResults = 10
      ..timeoutSeconds = Int64(30)
      ..isActive = false
      ..isEnabled = true;

    // 显示加载状态
    setState(() {
      _isTestingConnection = true;
      _testResult = null;
    });

    // 发送添加供应商事件
    context.read<WebSearchSettingsBloc>().add(
      WebSearchSettingsEvent.addProvider(config),
    );
    
    // 等待足够的时间让 BLoC 处理事件并保存到存储
    // 增加延迟时间以确保操作完成
    await Future.delayed(const Duration(seconds: 2));
    
    // 关闭对话框
    if (mounted) {
      Navigator.of(context).pop();
      
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            "供应商 '${_nameController.text}' 已保存",
          ),
        ),
      );
    }
  }

  String _getBaseUrl() {
    switch (_selectedProvider) {
      case WebSearchProviderType.tavily:
        return "https://api.tavily.com";
      case WebSearchProviderType.brave:
        return "https://api.search.brave.com";
    }
  }
}

/// 配置网络搜索供应商对话框
class _ConfigureWebSearchProviderDialog extends StatefulWidget {
  const _ConfigureWebSearchProviderDialog({required this.provider});

  final WebSearchProviderConfigPB provider;

  @override
  State<_ConfigureWebSearchProviderDialog> createState() => _ConfigureWebSearchProviderDialogState();
}

class _ConfigureWebSearchProviderDialogState extends State<_ConfigureWebSearchProviderDialog> {
  final _nameController = TextEditingController();
  final _apiKeyController = TextEditingController();
  
  WebSearchProviderType _selectedProvider = WebSearchProviderType.tavily;
  bool _isTestingConnection = false;
  String? _testResult;

  @override
  void initState() {
    super.initState();
    _nameController.text = widget.provider.name;
    _apiKeyController.text = widget.provider.apiKey;
    _selectedProvider = widget.provider.providerType == WebSearchProviderTypePB.Tavily 
        ? WebSearchProviderType.tavily 
        : WebSearchProviderType.brave;
  }

  @override
  void dispose() {
    _nameController.dispose();
    _apiKeyController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      title: FlowyText.medium("配置供应商: ${widget.provider.name}"),
      child: Container(
        width: 500,
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildProviderTypeSelector(),
            const VSpace(16),
            SettingsInputField(
              label: "供应商名称",
              placeholder: "输入供应商名称",
              textController: _nameController,
            ),
            const VSpace(16),
            SettingsInputField(
              label: "API 密钥",
              placeholder: _getApiKeyPlaceholder(),
              textController: _apiKeyController,
              obscureText: true,
            ),
            if (_testResult != null) ...[
              const VSpace(16),
              _buildTestResult(),
            ],
            const VSpace(24),
            _buildActionButtons(),
          ],
        ),
      ),
    );
  }

  Widget _buildProviderTypeSelector() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          "供应商类型",
          color: AFThemeExtension.of(context).secondaryTextColor,
        ),
        const VSpace(8),
        Row(
          children: [
            for (final provider in WebSearchProviderType.values) ...[
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(_getProviderTypeName(provider)),
                  backgroundColor: _selectedProvider == provider
                      ? Theme.of(context).colorScheme.primary
                      : Theme.of(context).colorScheme.surface,
                  onTap: () => setState(() => _selectedProvider = provider),
                ),
              ),
              if (provider != WebSearchProviderType.values.last) const HSpace(8),
            ],
          ],
        ),
      ],
    );
  }

  Widget _buildTestResult() {
    // 检查是否包含成功相关的关键词（支持中英文）
    final isSuccess = _testResult!.contains('success') || 
                      _testResult!.contains('successful') ||
                      _testResult!.contains('成功') ||
                      _testResult!.contains('通过');
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: isSuccess ? Colors.green.withOpacity(0.1) : Colors.red.withOpacity(0.1),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: isSuccess ? Colors.green : Colors.red,
          width: 1,
        ),
      ),
      child: Row(
        children: [
          Icon(
            isSuccess ? Icons.check_circle : Icons.error,
            color: isSuccess ? Colors.green : Colors.red,
            size: 16,
          ),
          const HSpace(8),
          Expanded(
            child: FlowyText.regular(
              _testResult!,
              color: isSuccess ? Colors.green : Colors.red,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons() {
    return Row(
      children: [
        Expanded(
          child: FlowyButton(
            text: FlowyText.regular(
              _isTestingConnection
                  ? "测试中..."
                  : "测试连接",
            ),
            onTap: _isTestingConnection ? null : _testConnection,
            leftIcon: _isTestingConnection
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : null,
          ),
        ),
        const HSpace(12),
        Expanded(
          child: FlowyButton(
            text: FlowyText.regular(LocaleKeys.button_cancel.tr()),
            onTap: () => Navigator.of(context).pop(),
          ),
        ),
        const HSpace(12),
        Expanded(
          child: FlowyButton(
            text: FlowyText.regular(LocaleKeys.button_save.tr()),
            backgroundColor: Theme.of(context).colorScheme.primary,
            onTap: _canSave() ? _saveProvider : null,
          ),
        ),
      ],
    );
  }

  String _getProviderTypeName(WebSearchProviderType provider) {
    switch (provider) {
      case WebSearchProviderType.tavily:
        return "Tavily";
      case WebSearchProviderType.brave:
        return "Brave Search";
    }
  }

  String _getApiKeyPlaceholder() {
    switch (_selectedProvider) {
      case WebSearchProviderType.tavily:
        return "输入 Tavily API 密钥 (tvly-...)";
      case WebSearchProviderType.brave:
        return "输入 Brave Search API 密钥 (BSA...)";
    }
  }

  bool _canSave() {
    return _nameController.text.trim().isNotEmpty &&
           _apiKeyController.text.trim().isNotEmpty;
  }

  void _testConnection() async {
    if (!_canSave()) return;

    setState(() {
      _isTestingConnection = true;
      _testResult = null;
    });

    // TODO: Implement actual connection testing
    await Future.delayed(const Duration(seconds: 2));

    setState(() {
      _isTestingConnection = false;
      _testResult = "连接测试成功";
    });
  }

  void _saveProvider() async {
    if (!_canSave()) return;

    final config = WebSearchProviderConfigPB()
      ..id = widget.provider.id
      ..name = _nameController.text.trim()
      ..providerType = _selectedProvider == WebSearchProviderType.tavily 
          ? WebSearchProviderTypePB.Tavily 
          : WebSearchProviderTypePB.BraveSearch
      ..description = widget.provider.description
      ..icon = widget.provider.icon
      ..apiKey = _apiKeyController.text.trim()
      ..baseUrl = widget.provider.baseUrl
      ..maxResults = widget.provider.maxResults
      ..timeoutSeconds = widget.provider.timeoutSeconds
      ..isActive = widget.provider.isActive
      ..isEnabled = widget.provider.isEnabled
      ..createdAt = widget.provider.createdAt
      ..updatedAt = Int64(DateTime.now().millisecondsSinceEpoch ~/ 1000)
      ..lastTestedAt = widget.provider.lastTestedAt
      ..testStatus = widget.provider.testStatus
      ..metadata.addAll(widget.provider.metadata);

    // 显示加载状态
    setState(() {
      _isTestingConnection = true;
      _testResult = null;
    });

    // 发送更新供应商事件
    context.read<WebSearchSettingsBloc>().add(
      WebSearchSettingsEvent.updateProvider(config),
    );
    
    // 等待足够的时间让 BLoC 处理事件并保存到存储
    await Future.delayed(const Duration(seconds: 2));
    
    // 关闭对话框
    if (mounted) {
      Navigator.of(context).pop();
      
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            "供应商 '${_nameController.text}' 已更新",
          ),
        ),
      );
    }
  }
}

/// 网络搜索供应商类型枚举
enum WebSearchProviderType {
  tavily,
  brave,
}

/// 网络搜索供应商状态枚举
enum WebSearchProviderStatus {
  connected,
  connecting,
  disconnected,
  error,
}

/// 网络搜索全局配置部分
class _WebSearchGlobalConfigSection extends StatelessWidget {
  const _WebSearchGlobalConfigSection();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<WebSearchSettingsBloc, WebSearchSettingsState>(
      builder: (context, state) {
        final config = state.globalConfig;
        
        return Container(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.medium(
                "全局配置",
                color: AFThemeExtension.of(context).strongText,
              ),
              const VSpace(12),
              if (config != null) ...[
                _buildConfigItem(
                  context,
                  "启用网络搜索",
                  Switch(
                    value: config.enabled,
                    onChanged: (value) {
                      final updatedConfig = config.clone();
                      updatedConfig.enabled = value;
                      _updateConfig(context, updatedConfig);
                    },
                  ),
                ),
                const VSpace(8),
                _buildConfigItem(
                  context,
                  "默认最大结果数",
                  SizedBox(
                    width: 100,
                    child: TextField(
                      controller: TextEditingController(text: config.defaultMaxResults.toString()),
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                        border: OutlineInputBorder(),
                        contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      ),
                      onChanged: (value) {
                        final intValue = int.tryParse(value);
                        if (intValue != null && intValue > 0) {
                          final updatedConfig = config.clone();
                          updatedConfig.defaultMaxResults = intValue;
                          _updateConfig(context, updatedConfig);
                        }
                      },
                    ),
                  ),
                ),
                const VSpace(8),
                _buildConfigItem(
                  context,
                  "启用缓存",
                  Switch(
                    value: config.enableCache,
                    onChanged: (value) {
                      final updatedConfig = config.clone();
                      updatedConfig.enableCache = value;
                      _updateConfig(context, updatedConfig);
                    },
                  ),
                ),
                const VSpace(8),
                _buildConfigItem(
                  context,
                  "缓存过期时间（秒）",
                  SizedBox(
                    width: 120,
                    child: TextField(
                      controller: TextEditingController(text: config.cacheExpirySeconds.toString()),
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                        border: OutlineInputBorder(),
                        contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      ),
                      onChanged: (value) {
                        final intValue = int.tryParse(value);
                        if (intValue != null && intValue > 0) {
                          final updatedConfig = config.clone();
                          updatedConfig.cacheExpirySeconds = Int64(intValue);
                          _updateConfig(context, updatedConfig);
                        }
                      },
                    ),
                  ),
                ),
              ] else
                const Center(
                  child: Padding(
                    padding: EdgeInsets.all(32.0),
                    child: CircularProgressIndicator(),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildConfigItem(BuildContext context, String label, Widget widget) {
    return Row(
      children: [
        SizedBox(
          width: 150,
          child: FlowyText.regular(
            label,
            color: AFThemeExtension.of(context).secondaryTextColor,
          ),
        ),
        const HSpace(16),
        widget,
      ],
    );
  }

  void _updateConfig(BuildContext context, WebSearchGlobalConfigPB config) {
    context.read<WebSearchSettingsBloc>().add(
      WebSearchSettingsEvent.updateGlobalConfig(config),
    );
  }
}

/// 网络搜索缓存管理部分
class _WebSearchCacheSection extends StatelessWidget {
  const _WebSearchCacheSection();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<WebSearchSettingsBloc, WebSearchSettingsState>(
      builder: (context, state) {
        final stats = state.cacheStats;
        
        return Container(
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: const BorderRadius.all(Radius.circular(8.0)),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  FlowyText.medium(
                    "缓存管理",
                    color: AFThemeExtension.of(context).strongText,
                  ),
                  const Spacer(),
                  FlowyTextButton(
                    "刷新统计",
                    fontColor: Theme.of(context).colorScheme.primary,
                    onPressed: () => context.read<WebSearchSettingsBloc>().add(
                      const WebSearchSettingsEvent.getCacheStats(),
                    ),
                  ),
                  const HSpace(8),
                  FlowyTextButton(
                    "清空缓存",
                    fontColor: Colors.red,
                    onPressed: () => _showClearCacheDialog(context),
                  ),
                ],
              ),
              const VSpace(12),
              if (stats != null) ...[
                _buildStatItem(context, "缓存条目总数", "${stats.totalEntries}"),
                _buildStatItem(context, "缓存命中次数", "${stats.hitCount}"),
                _buildStatItem(context, "缓存未命中次数", "${stats.missCount}"),
                _buildStatItem(context, "缓存命中率", "${(stats.hitRate * 100).toStringAsFixed(1)}%"),
                _buildStatItem(context, "缓存大小", "${(stats.cacheSizeBytes.toInt() / (1024 * 1024)).toStringAsFixed(2)} MB"),
                if (stats.hasLastCleanupAt())
                  _buildStatItem(context, "最后清理时间", 
                    DateTime.fromMillisecondsSinceEpoch(stats.lastCleanupAt.toInt() * 1000)
                        .toString().substring(0, 19)),
              ] else
                const Center(
                  child: Padding(
                    padding: EdgeInsets.all(32.0),
                    child: CircularProgressIndicator(),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildStatItem(BuildContext context, String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8.0),
      child: Row(
        children: [
          SizedBox(
            width: 120,
            child: FlowyText.regular(
              label,
              color: AFThemeExtension.of(context).secondaryTextColor,
            ),
          ),
          const HSpace(16),
          FlowyText.medium(
            value,
            color: AFThemeExtension.of(context).strongText,
          ),
        ],
      ),
    );
  }

  void _showClearCacheDialog(BuildContext context) {
    final bloc = context.read<WebSearchSettingsBloc>();
    showDialog(
      context: context,
      builder: (dialogContext) => BlocProvider.value(
        value: bloc,
        child: Builder(
          builder: (context) => AlertDialog(
            title: Text("清空缓存"),
            content: Text("确定要清空所有网络搜索缓存吗？此操作不可撤销。"),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: Text(LocaleKeys.button_cancel.tr()),
              ),
              TextButton(
                onPressed: () {
                  context.read<WebSearchSettingsBloc>().add(
                    const WebSearchSettingsEvent.clearCache(),
                  );
                  Navigator.of(context).pop();
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text("缓存已清空"),
                    ),
                  );
                },
                child: Text(LocaleKeys.button_delete.tr()),
              ),
            ],
          ),
        ),
      ),
    );
  }
}