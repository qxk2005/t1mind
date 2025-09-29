import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_body.dart';
import 'package:appflowy/workspace/presentation/settings/shared/settings_input_field.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flutter/material.dart';

class MCPSettingsPage extends StatelessWidget {
  const MCPSettingsPage({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  Widget build(BuildContext context) {
    return SettingsBody(
      title: LocaleKeys.settings_aiPage_keys_mcpTitle.tr(),
      description: LocaleKeys.settings_aiPage_keys_mcpDescription.tr(),
      children: [
        const _MCPServerList(),
        const VSpace(16),
        const _AddMCPServerSection(),
      ],
    );
  }
}

class _MCPServerList extends StatelessWidget {
  const _MCPServerList();

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
          Row(
            children: [
              FlowyText.medium(
                LocaleKeys.settings_aiPage_keys_mcpServerList.tr(),
                color: AFThemeExtension.of(context).strongText,
              ),
              const Spacer(),
              FlowyTextButton(
                LocaleKeys.settings_aiPage_keys_addMCPServer.tr(),
                fontColor: Theme.of(context).colorScheme.primary,
                onPressed: () => _showAddServerDialog(context),
              ),
            ],
          ),
          const VSpace(12),
          // TODO: Replace with actual server list from BLoC
          _buildServerListItem(
            context,
            name: "Example Server",
            url: "stdio://path/to/server",
            status: MCPServerStatus.connected,
          ),
          const VSpace(8),
          _buildServerListItem(
            context,
            name: "HTTP Server",
            url: "http://localhost:3000",
            status: MCPServerStatus.disconnected,
          ),
        ],
      ),
    );
  }

  Widget _buildServerListItem(
    BuildContext context, {
    required String name,
    required String url,
    required MCPServerStatus status,
  }) {
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
                FlowyText.medium(
                  name,
                  color: AFThemeExtension.of(context).strongText,
                ),
                const VSpace(4),
                FlowyText.regular(
                  url,
                  color: AFThemeExtension.of(context).secondaryTextColor,
                  fontSize: 12,
                ),
              ],
            ),
          ),
          const HSpace(12),
          _buildStatusIndicator(context, status),
          const HSpace(8),
          FlowyIconButton(
            icon: const FlowySvg(FlowySvgs.settings_s),
            onPressed: () => _showServerConfigDialog(context, name),
            tooltipText: LocaleKeys.settings_aiPage_keys_configureServer.tr(),
          ),
          const HSpace(4),
          FlowyIconButton(
            icon: const FlowySvg(FlowySvgs.delete_s),
            onPressed: () => _showDeleteConfirmDialog(context, name),
            tooltipText: LocaleKeys.button_delete.tr(),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusIndicator(BuildContext context, MCPServerStatus status) {
    Color color;
    String text;
    
    switch (status) {
      case MCPServerStatus.connected:
        color = Colors.green;
        text = LocaleKeys.settings_aiPage_keys_mcpStatusConnected.tr();
        break;
      case MCPServerStatus.connecting:
        color = Colors.orange;
        text = LocaleKeys.settings_aiPage_keys_mcpStatusConnecting.tr();
        break;
      case MCPServerStatus.disconnected:
        color = Colors.red;
        text = LocaleKeys.settings_aiPage_keys_mcpStatusDisconnected.tr();
        break;
      case MCPServerStatus.error:
        color = Colors.red;
        text = LocaleKeys.settings_aiPage_keys_mcpStatusError.tr();
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

  void _showAddServerDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const _AddMCPServerDialog(),
    );
  }

  void _showServerConfigDialog(BuildContext context, String serverName) {
    showDialog(
      context: context,
      builder: (context) => _ConfigureMCPServerDialog(serverName: serverName),
    );
  }

  void _showDeleteConfirmDialog(BuildContext context, String serverName) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(LocaleKeys.settings_aiPage_keys_deleteMCPServerTitle.tr()),
        content: Text(LocaleKeys.settings_aiPage_keys_deleteMCPServerMessage.tr(args: [serverName])),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text(LocaleKeys.button_cancel.tr()),
          ),
          TextButton(
            onPressed: () {
              // TODO: Implement server deletion
              Navigator.of(context).pop();
            },
            child: Text(LocaleKeys.button_delete.tr()),
          ),
        ],
      ),
    );
  }
}

class _AddMCPServerSection extends StatelessWidget {
  const _AddMCPServerSection();

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
            LocaleKeys.settings_aiPage_keys_mcpQuickStart.tr(),
            color: AFThemeExtension.of(context).strongText,
          ),
          const VSpace(8),
          FlowyText.regular(
            LocaleKeys.settings_aiPage_keys_mcpQuickStartDescription.tr(),
            color: AFThemeExtension.of(context).secondaryTextColor,
            maxLines: 3,
          ),
          const VSpace(12),
          Row(
            children: [
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(
                    LocaleKeys.settings_aiPage_keys_addMCPServer.tr(),
                  ),
                  onTap: () => _showAddServerDialog(context),
                ),
              ),
              const HSpace(12),
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(
                    LocaleKeys.settings_aiPage_keys_testAllConnections.tr(),
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

  void _showAddServerDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => const _AddMCPServerDialog(),
    );
  }

  void _testAllConnections(BuildContext context) {
    // TODO: Implement connection testing for all servers
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(LocaleKeys.settings_aiPage_keys_testingConnections.tr()),
      ),
    );
  }
}

class _AddMCPServerDialog extends StatefulWidget {
  const _AddMCPServerDialog();

  @override
  State<_AddMCPServerDialog> createState() => _AddMCPServerDialogState();
}

class _AddMCPServerDialogState extends State<_AddMCPServerDialog> {
  final _nameController = TextEditingController();
  final _urlController = TextEditingController();
  final _argsController = TextEditingController();
  final _envController = TextEditingController();
  
  MCPTransportType _selectedTransport = MCPTransportType.stdio;
  bool _isTestingConnection = false;
  String? _testResult;

  @override
  void dispose() {
    _nameController.dispose();
    _urlController.dispose();
    _argsController.dispose();
    _envController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      title: FlowyText.medium(LocaleKeys.settings_aiPage_keys_addMCPServer.tr()),
      child: SizedBox(
        width: 500,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SettingsInputField(
              label: LocaleKeys.settings_aiPage_keys_mcpServerName.tr(),
              placeholder: LocaleKeys.settings_aiPage_keys_mcpServerNameHint.tr(),
              textController: _nameController,
            ),
            const VSpace(16),
            _buildTransportTypeSelector(),
            const VSpace(16),
            _buildTransportSpecificFields(),
            const VSpace(16),
            _buildAdvancedOptions(),
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

  Widget _buildTransportTypeSelector() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_mcpTransportType.tr(),
          color: AFThemeExtension.of(context).secondaryTextColor,
        ),
        const VSpace(8),
        Row(
          children: [
            for (final transport in MCPTransportType.values) ...[
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(_getTransportTypeName(transport)),
                  backgroundColor: _selectedTransport == transport
                      ? Theme.of(context).colorScheme.primary
                      : Theme.of(context).colorScheme.surface,
                  onTap: () => setState(() => _selectedTransport = transport),
                ),
              ),
              if (transport != MCPTransportType.values.last) const HSpace(8),
            ],
          ],
        ),
      ],
    );
  }

  Widget _buildTransportSpecificFields() {
    switch (_selectedTransport) {
      case MCPTransportType.stdio:
        return Column(
          children: [
            SettingsInputField(
              label: LocaleKeys.settings_aiPage_keys_mcpCommandPath.tr(),
              placeholder: LocaleKeys.settings_aiPage_keys_mcpCommandPathHint.tr(),
              textController: _urlController,
            ),
            const VSpace(12),
            SettingsInputField(
              label: LocaleKeys.settings_aiPage_keys_mcpArguments.tr(),
              placeholder: LocaleKeys.settings_aiPage_keys_mcpArgumentsHint.tr(),
              textController: _argsController,
            ),
          ],
        );
      case MCPTransportType.sse:
      case MCPTransportType.http:
        return SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_mcpServerUrl.tr(),
          placeholder: _selectedTransport == MCPTransportType.sse
              ? LocaleKeys.settings_aiPage_keys_mcpSSEUrlHint.tr()
              : LocaleKeys.settings_aiPage_keys_mcpHTTPUrlHint.tr(),
          textController: _urlController,
        );
    }
  }

  Widget _buildAdvancedOptions() {
    return ExpansionTile(
      title: FlowyText.medium(
        LocaleKeys.settings_aiPage_keys_mcpAdvancedOptions.tr(),
        color: AFThemeExtension.of(context).secondaryTextColor,
      ),
      children: [
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_mcpEnvironmentVars.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_mcpEnvironmentVarsHint.tr(),
          textController: _envController,
        ),
      ],
    );
  }

  Widget _buildTestResult() {
    final isSuccess = _testResult!.contains('success') || _testResult!.contains('successful');
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
                  ? LocaleKeys.settings_aiPage_keys_testingConnection.tr()
                  : LocaleKeys.settings_aiPage_keys_testConnection.tr(),
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
            onTap: _canSave() ? _saveServer : null,
          ),
        ),
      ],
    );
  }

  String _getTransportTypeName(MCPTransportType transport) {
    switch (transport) {
      case MCPTransportType.stdio:
        return LocaleKeys.settings_aiPage_keys_mcpTransportSTDIO.tr();
      case MCPTransportType.sse:
        return LocaleKeys.settings_aiPage_keys_mcpTransportSSE.tr();
      case MCPTransportType.http:
        return LocaleKeys.settings_aiPage_keys_mcpTransportHTTP.tr();
    }
  }

  bool _canSave() {
    return _nameController.text.trim().isNotEmpty &&
           _urlController.text.trim().isNotEmpty;
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
      _testResult = LocaleKeys.settings_aiPage_keys_connectionTestSuccess.tr();
    });
  }

  void _saveServer() {
    if (!_canSave()) return;

    // TODO: Implement server saving
    Navigator.of(context).pop();
    
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          LocaleKeys.settings_aiPage_keys_mcpServerSaved.tr(args: [_nameController.text]),
        ),
      ),
    );
  }
}

class _ConfigureMCPServerDialog extends StatefulWidget {
  const _ConfigureMCPServerDialog({required this.serverName});

  final String serverName;

  @override
  State<_ConfigureMCPServerDialog> createState() => _ConfigureMCPServerDialogState();
}

class _ConfigureMCPServerDialogState extends State<_ConfigureMCPServerDialog> {
  final _nameController = TextEditingController();
  final _urlController = TextEditingController();
  final _argsController = TextEditingController();
  final _envController = TextEditingController();
  
  MCPTransportType _selectedTransport = MCPTransportType.stdio;
  bool _isTestingConnection = false;
  String? _testResult;

  @override
  void initState() {
    super.initState();
    // TODO: Load existing server configuration
    _nameController.text = widget.serverName;
    _urlController.text = "stdio://path/to/server";
  }

  @override
  void dispose() {
    _nameController.dispose();
    _urlController.dispose();
    _argsController.dispose();
    _envController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      title: FlowyText.medium(LocaleKeys.settings_aiPage_keys_configureMCPServer.tr(args: [widget.serverName])),
      child: SizedBox(
        width: 500,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SettingsInputField(
              label: LocaleKeys.settings_aiPage_keys_mcpServerName.tr(),
              placeholder: LocaleKeys.settings_aiPage_keys_mcpServerNameHint.tr(),
              textController: _nameController,
            ),
            const VSpace(16),
            _buildTransportTypeSelector(),
            const VSpace(16),
            _buildTransportSpecificFields(),
            const VSpace(16),
            _buildAdvancedOptions(),
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

  Widget _buildTransportTypeSelector() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          LocaleKeys.settings_aiPage_keys_mcpTransportType.tr(),
          color: AFThemeExtension.of(context).secondaryTextColor,
        ),
        const VSpace(8),
        Row(
          children: [
            for (final transport in MCPTransportType.values) ...[
              Expanded(
                child: FlowyButton(
                  text: FlowyText.regular(_getTransportTypeName(transport)),
                  backgroundColor: _selectedTransport == transport
                      ? Theme.of(context).colorScheme.primary
                      : Theme.of(context).colorScheme.surface,
                  onTap: () => setState(() => _selectedTransport = transport),
                ),
              ),
              if (transport != MCPTransportType.values.last) const HSpace(8),
            ],
          ],
        ),
      ],
    );
  }

  Widget _buildTransportSpecificFields() {
    switch (_selectedTransport) {
      case MCPTransportType.stdio:
        return Column(
          children: [
            SettingsInputField(
              label: LocaleKeys.settings_aiPage_keys_mcpCommandPath.tr(),
              placeholder: LocaleKeys.settings_aiPage_keys_mcpCommandPathHint.tr(),
              textController: _urlController,
            ),
            const VSpace(12),
            SettingsInputField(
              label: LocaleKeys.settings_aiPage_keys_mcpArguments.tr(),
              placeholder: LocaleKeys.settings_aiPage_keys_mcpArgumentsHint.tr(),
              textController: _argsController,
            ),
          ],
        );
      case MCPTransportType.sse:
      case MCPTransportType.http:
        return SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_mcpServerUrl.tr(),
          placeholder: _selectedTransport == MCPTransportType.sse
              ? LocaleKeys.settings_aiPage_keys_mcpSSEUrlHint.tr()
              : LocaleKeys.settings_aiPage_keys_mcpHTTPUrlHint.tr(),
          textController: _urlController,
        );
    }
  }

  Widget _buildAdvancedOptions() {
    return ExpansionTile(
      title: FlowyText.medium(
        LocaleKeys.settings_aiPage_keys_mcpAdvancedOptions.tr(),
        color: AFThemeExtension.of(context).secondaryTextColor,
      ),
      children: [
        SettingsInputField(
          label: LocaleKeys.settings_aiPage_keys_mcpEnvironmentVars.tr(),
          placeholder: LocaleKeys.settings_aiPage_keys_mcpEnvironmentVarsHint.tr(),
          textController: _envController,
        ),
      ],
    );
  }

  Widget _buildTestResult() {
    final isSuccess = _testResult!.contains('success') || _testResult!.contains('successful');
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
                  ? LocaleKeys.settings_aiPage_keys_testingConnection.tr()
                  : LocaleKeys.settings_aiPage_keys_testConnection.tr(),
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
            onTap: _canSave() ? _saveServer : null,
          ),
        ),
      ],
    );
  }

  String _getTransportTypeName(MCPTransportType transport) {
    switch (transport) {
      case MCPTransportType.stdio:
        return LocaleKeys.settings_aiPage_keys_mcpTransportSTDIO.tr();
      case MCPTransportType.sse:
        return LocaleKeys.settings_aiPage_keys_mcpTransportSSE.tr();
      case MCPTransportType.http:
        return LocaleKeys.settings_aiPage_keys_mcpTransportHTTP.tr();
    }
  }

  bool _canSave() {
    return _nameController.text.trim().isNotEmpty &&
           _urlController.text.trim().isNotEmpty;
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
      _testResult = LocaleKeys.settings_aiPage_keys_connectionTestSuccess.tr();
    });
  }

  void _saveServer() {
    if (!_canSave()) return;

    // TODO: Implement server saving
    Navigator.of(context).pop();
    
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          LocaleKeys.settings_aiPage_keys_mcpServerUpdated.tr(args: [_nameController.text]),
        ),
      ),
    );
  }
}

// Enums for MCP configuration
enum MCPTransportType {
  stdio,
  sse,
  http,
}

enum MCPServerStatus {
  connected,
  connecting,
  disconnected,
  error,
}