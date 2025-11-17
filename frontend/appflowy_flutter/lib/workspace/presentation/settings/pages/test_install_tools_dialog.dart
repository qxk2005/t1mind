import 'dart:async';

import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../import/import_settings_bloc.dart';
import 'install_tools_progress_dialog.dart';

/// 测试安装工具对话框的页面
/// 用于验证安装组件的UI窗口是否正确显示
class TestInstallToolsDialogPage extends StatefulWidget {
  const TestInstallToolsDialogPage({
    super.key,
    required this.userProfile,
    required this.workspaceId,
  });

  final UserProfilePB userProfile;
  final String workspaceId;

  @override
  State<TestInstallToolsDialogPage> createState() => _TestInstallToolsDialogPageState();
}

class _TestInstallToolsDialogPageState extends State<TestInstallToolsDialogPage> {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<ImportSettingsBloc>(
      create: (context) => ImportSettingsBloc(
        userProfile: widget.userProfile,
        workspaceId: widget.workspaceId,
      )..add(const ImportSettingsEvent.initial()),
      child: Scaffold(
        appBar: AppBar(
          title: const Text('测试安装工具对话框'),
          backgroundColor: Colors.blue,
          foregroundColor: Colors.white,
        ),
        body: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                '安装工具对话框测试',
                style: TextStyle(
                  fontSize: 24,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 16),
              const Text(
                '此页面用于测试安装缺失组件的UI窗口显示。点击下面的按钮可以模拟安装过程。',
                style: TextStyle(fontSize: 14, color: Colors.grey),
              ),
              const SizedBox(height: 32),
              
              // 测试按钮
              Builder(
                builder: (buttonContext) => ElevatedButton.icon(
                  onPressed: () => _testInstallDialog(buttonContext),
                  icon: const Icon(Icons.bug_report),
                  label: const Text('测试安装对话框（模拟 marker-pdf）'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.blue,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                  ),
                ),
              ),
              const SizedBox(height: 16),
              
              Builder(
                builder: (buttonContext) => ElevatedButton.icon(
                  onPressed: () => _testInstallDialogMultiple(buttonContext),
                  icon: const Icon(Icons.bug_report),
                  label: const Text('测试安装对话框（多个工具）'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.green,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                  ),
                ),
              ),
              const SizedBox(height: 16),
              
              Builder(
                builder: (buttonContext) => ElevatedButton.icon(
                  onPressed: () => _testInstallDialogWithError(buttonContext),
                  icon: const Icon(Icons.error_outline),
                  label: const Text('测试安装对话框（模拟失败）'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.red,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                  ),
                ),
              ),
              const SizedBox(height: 32),
              
              const Divider(),
              const SizedBox(height: 16),
              
              const Text(
                '说明：',
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                '• 这些按钮会模拟安装过程，展示流式反馈效果\n'
                '• 对话框会显示安装步骤、进度条和执行日志\n'
                '• 可以验证UI是否正确显示和更新',
                style: TextStyle(fontSize: 14),
              ),
            ],
          ),
        ),
      ),
    );
  }

  /// 测试安装对话框（单个工具）
  void _testInstallDialog(BuildContext context) async {
    try {
      final bloc = context.read<ImportSettingsBloc>();
      
      // 模拟安装过程
      await _simulateInstallation(
        context,
        bloc,
        ['marker-pdf'],
        shouldFail: false,
      );
    } catch (e, stackTrace) {
      Log.error('测试安装对话框失败: $e', stackTrace);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('测试失败: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  /// 测试安装对话框（多个工具）
  void _testInstallDialogMultiple(BuildContext context) async {
    try {
      final bloc = context.read<ImportSettingsBloc>();
      
      // 模拟安装过程
      await _simulateInstallation(
        context,
        bloc,
        ['Marker (精准 PDF 导入)', 'marker-pdf (PDF 转换引擎)'],
        shouldFail: false,
      );
    } catch (e, stackTrace) {
      Log.error('测试安装对话框（多个工具）失败: $e', stackTrace);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('测试失败: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  /// 测试安装对话框（失败场景）
  void _testInstallDialogWithError(BuildContext context) async {
    try {
      final bloc = context.read<ImportSettingsBloc>();
      
      // 模拟安装过程（失败）
      await _simulateInstallation(
        context,
        bloc,
        ['marker-pdf'],
        shouldFail: true,
      );
    } catch (e, stackTrace) {
      Log.error('测试安装对话框（失败场景）失败: $e', stackTrace);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('测试失败: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  /// 模拟安装过程
  Future<void> _simulateInstallation(
    BuildContext context,
    ImportSettingsBloc bloc,
    List<String> toolNames, {
    required bool shouldFail,
  }) async {
    try {
      Log.info('开始模拟安装过程: ${toolNames.join(", ")}');
      
      // 累积的日志列表
      final accumulatedLogs = <String>[];
      
      // 先设置初始状态，避免与 installMissingTools 事件冲突
      // 注意：在测试场景中，我们不调用 installMissingTools，只使用 updateInstallProgress
      final initialProgress = InstallToolProgressPB()
        ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
        ..status = InstallToolStatusPB.InstallToolInstalling
        ..progress = 0.0
        ..message = '准备安装...'
        ..logs.add('开始安装工具: ${toolNames.join(", ")}');
      
      // 设置安装状态为 true，这样对话框会显示安装中的状态
      bloc.add(ImportSettingsEvent.updateInstallProgress(initialProgress));
      
      // 等待一小段时间，确保状态已更新
      await Future.delayed(const Duration(milliseconds: 100));
      
      // 显示对话框（不自动触发安装，因为我们已经手动设置了状态）
      // 使用 autoStartInstall: false 避免重复触发安装事件
      // 直接传入 bloc，避免从 context 读取时找不到 Provider
      final dialogFuture = showInstallToolsProgressDialog(
        context,
        toolNames: toolNames,
        autoStartInstall: false,
        bloc: bloc,
      );
      
      // 等待对话框显示完成（使用 unawaited 因为我们不想阻塞，但需要捕获错误）
      unawaited(dialogFuture.catchError((error, stackTrace) {
        Log.error('显示对话框失败: $error', stackTrace);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('显示对话框失败: $error'),
              backgroundColor: Colors.red,
            ),
          );
        }
      }));
      
      // 等待一小段时间，确保对话框已显示
      await Future.delayed(const Duration(milliseconds: 500));
      
      // 模拟安装步骤
      final steps = [
      {
        'progress': 0.0,
        'message': '准备安装...',
        'status': InstallToolStatusPB.InstallToolInstalling,
        'logs': ['开始安装工具: ${toolNames.join(", ")}'],
      },
      {
        'progress': 0.1,
        'message': '检查 Homebrew...',
        'status': InstallToolStatusPB.InstallToolInstalling,
        'logs': ['步骤 1/4: 检查 Homebrew...', '✓ Homebrew 已安装，跳过此步骤'],
      },
      {
        'progress': 0.3,
        'message': '安装 pipx...',
        'status': InstallToolStatusPB.InstallToolInstalling,
        'logs': [
          '步骤 2/4: 检查 pipx...',
          'pipx 未安装，开始安装...',
          '执行命令: brew install pipx',
          '正在安装 pipx，这可能需要几分钟...',
          '输出: Updating Homebrew...',
          '输出: Installing pipx...',
        ],
      },
      {
        'progress': 0.6,
        'message': '安装依赖库...',
        'status': InstallToolStatusPB.InstallToolInstalling,
        'logs': [
          '步骤 3/4: 安装 marker-pdf 依赖库...',
          '需要安装的依赖: jpeg, libpng, freetype, openjpeg, libtiff, webp',
          '执行命令: brew install jpeg libpng freetype openjpeg libtiff webp',
          '正在安装依赖库，这可能需要几分钟...',
          '输出: Installing jpeg...',
          '输出: Installing libpng...',
          '✓ 依赖库安装成功',
        ],
      },
      {
        'progress': 0.9,
        'message': '安装 marker-pdf...',
        'status': InstallToolStatusPB.InstallToolInstalling,
        'logs': [
          '步骤 4/4: 安装 marker-pdf...',
          '执行命令: pipx install marker-pdf',
          '正在安装 marker-pdf，这可能需要 5-10 分钟，请耐心等待...',
          '注意: marker-pdf 会下载模型文件，首次安装时间较长',
          '输出: Installing marker-pdf...',
          '输出: Downloading models...',
        ],
      },
    ];
    
      // 逐步发送进度更新（累积日志）
      for (var i = 0; i < steps.length; i++) {
        // 检查上下文是否仍然有效
        if (!mounted) {
          Log.warn('上下文已失效，停止模拟安装');
          break;
        }
        
        await Future.delayed(const Duration(seconds: 2));
        
        final step = steps[i];
        final newLogs = step['logs'] as List<String>;
        accumulatedLogs.addAll(newLogs);
        
        final progress = InstallToolProgressPB()
          ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
          ..status = step['status'] as InstallToolStatusPB
          ..progress = step['progress'] as double
          ..message = step['message'] as String
          ..logs.addAll(accumulatedLogs);
        
        Log.info('更新安装进度: ${progress.progress * 100}% - ${progress.message}');
        bloc.add(ImportSettingsEvent.updateInstallProgress(progress));
      }
      
      // 最终结果
      if (mounted) {
        await Future.delayed(const Duration(seconds: 2));
        
        if (shouldFail) {
          // 模拟失败
          accumulatedLogs.addAll([
            '✗ marker-pdf 安装失败',
            '错误: pipx install marker-pdf failed with exit code 1',
            '请检查网络连接或手动安装',
          ]);
          
          final failedProgress = InstallToolProgressPB()
            ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
            ..status = InstallToolStatusPB.InstallToolFailed
            ..progress = 0.9
            ..message = '安装失败: marker-pdf 安装出错'
            ..logs.addAll(accumulatedLogs);
          
          Log.info('模拟安装失败');
          bloc.add(ImportSettingsEvent.updateInstallProgress(failedProgress));
          bloc.add(ImportSettingsEvent.installFailed('安装失败: marker-pdf 安装出错'));
        } else {
          // 模拟成功
          accumulatedLogs.addAll([
            '',
            '✓ marker-pdf 安装成功',
            '=== 所有工具安装完成 ===',
          ]);
          
          final successProgress = InstallToolProgressPB()
            ..toolName = toolNames.isNotEmpty ? toolNames[0] : 'unknown'
            ..status = InstallToolStatusPB.InstallToolCompleted
            ..progress = 1.0
            ..message = '所有工具安装完成！'
            ..logs.addAll(accumulatedLogs);
          
          Log.info('模拟安装成功');
          bloc.add(ImportSettingsEvent.updateInstallProgress(successProgress));
        }
      }
    } catch (e, stackTrace) {
      Log.error('模拟安装过程出错: $e', stackTrace);
      rethrow;
    }
  }
}

/// 显示测试页面
Future<void> showTestInstallToolsDialogPage(
  BuildContext context, {
  required UserProfilePB userProfile,
  required String workspaceId,
}) async {
  await Navigator.of(context).push(
    MaterialPageRoute(
      builder: (context) => TestInstallToolsDialogPage(
        userProfile: userProfile,
        workspaceId: workspaceId,
      ),
    ),
  );
}

