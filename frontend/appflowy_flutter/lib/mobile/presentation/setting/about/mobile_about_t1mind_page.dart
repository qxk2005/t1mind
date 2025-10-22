import 'package:appflowy/core/helpers/url_launcher.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_group_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_item_widget.dart';
import 'package:appflowy/mobile/presentation/setting/widgets/mobile_setting_trailing.dart';
import 'package:appflowy/shared/changelog/changelog_loader.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// 移动端T1Mind关于页面
/// 为移动端创建适配的关于t1mind页面
/// 使用MobileSettingGroup布局模式，优化触摸操作
/// 提供版本信息、更新检查、更新历史等功能
class MobileAboutT1MindPage extends StatefulWidget {
  const MobileAboutT1MindPage({super.key});

  static const routeName = '/mobile_about_t1mind';

  @override
  State<MobileAboutT1MindPage> createState() => _MobileAboutT1MindPageState();
}

class _MobileAboutT1MindPageState extends State<MobileAboutT1MindPage> {
  T1MindUpdateInfo? _updateInfo;
  bool _isChecking = false;
  String? _errorMessage;
  bool _hasChecked = false;
  
  String? _changelogContent;
  bool _isLoadingChangelog = true;
  bool _hasChangelogError = false;

  @override
  void initState() {
    super.initState();
    _loadChangelog();
  }

  /// 加载changelog内容
  Future<void> _loadChangelog() async {
    try {
      setState(() {
        _isLoadingChangelog = true;
        _hasChangelogError = false;
      });

      final changelogLoader = ChangelogLoader();
      final content = await changelogLoader.getRawChangelog();
      
      if (mounted) {
        setState(() {
          _changelogContent = content;
          _isLoadingChangelog = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoadingChangelog = false;
          _hasChangelogError = true;
        });
      }
    }
  }

  /// 检查更新
  Future<void> _checkForUpdates() async {
    if (_isChecking) return;

    setState(() {
      _isChecking = true;
      _errorMessage = null;
    });

    try {
      final updateInfo = await T1MindVersionChecker().getUpdateInfo();
      if (mounted) {
        setState(() {
          _updateInfo = updateInfo;
          _isChecking = false;
          _hasChecked = true;
          _errorMessage = null;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isChecking = false;
          _hasChecked = true;
          _errorMessage = e.toString();
        });
      }
    }
  }

  /// 打开下载链接
  Future<void> _openDownloadLink() async {
    if (_updateInfo?.downloadUrl != null) {
      await afLaunchUrlString(_updateInfo!.downloadUrl);
    }
  }

  /// 显示changelog内容
  void _showChangelog(BuildContext context) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _buildChangelogBottomSheet(context),
    );
  }

  /// 构建changelog底部弹窗
  Widget _buildChangelogBottomSheet(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return Container(
      height: MediaQuery.of(context).size.height * 0.8,
      decoration: BoxDecoration(
        color: theme.backgroundColorScheme.primary,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
      ),
      child: Column(
        children: [
          // 拖拽指示器
          Container(
            margin: const EdgeInsets.only(top: 8),
            width: 40,
            height: 4,
            decoration: BoxDecoration(
              color: theme.textColorScheme.tertiary,
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          // 标题栏
          Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Text(
                  '更新历史',
                  style: theme.textStyle.heading3.enhanced(
                    color: theme.textColorScheme.primary,
                  ),
                ),
                const Spacer(),
                IconButton(
                  onPressed: () => Navigator.of(context).pop(),
                  icon: Icon(
                    Icons.close,
                    color: theme.textColorScheme.primary,
                  ),
                ),
              ],
            ),
          ),
          // 内容区域
          Expanded(
            child: _buildChangelogContent(context),
          ),
        ],
      ),
    );
  }

  /// 构建changelog内容
  Widget _buildChangelogContent(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    if (_isLoadingChangelog) {
      return Center(
        child: CircularProgressIndicator(
          color: theme.textColorScheme.primary,
        ),
      );
    }
    
    if (_hasChangelogError || _changelogContent == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.error_outline,
              size: 48,
              color: theme.textColorScheme.tertiary,
            ),
            const VSpace(16),
            Text(
              '加载失败',
              style: theme.textStyle.body.enhanced(
                color: theme.textColorScheme.tertiary,
              ),
            ),
          ],
        ),
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Text(
        _changelogContent!,
        style: theme.textStyle.body.enhanced(
          color: theme.textColorScheme.primary,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    return Scaffold(
      backgroundColor: theme.backgroundColorScheme.primary,
      appBar: AppBar(
        backgroundColor: theme.backgroundColorScheme.primary,
        elevation: 0,
        leading: IconButton(
          onPressed: () => Navigator.of(context).pop(),
          icon: Icon(
            Icons.arrow_back,
            color: theme.textColorScheme.primary,
          ),
        ),
        title: Text(
          '关于 T1Mind',
          style: theme.textStyle.heading3.enhanced(
            color: theme.textColorScheme.primary,
          ),
        ),
        centerTitle: true,
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              // 版本信息组
              _buildVersionInfoGroup(context),
              
              // 更新检查组
              _buildUpdateCheckGroup(context),
              
              // 更新历史组
              _buildChangelogGroup(context),
              
              // 版权信息组
              _buildCopyrightGroup(context),
              
              const VSpace(32),
            ],
          ),
        ),
      ),
    );
  }

  /// 构建版本信息组
  Widget _buildVersionInfoGroup(BuildContext context) {
    return MobileSettingGroup(
      groupTitle: '版本信息',
      settingItemList: [
        MobileSettingItem(
          name: '应用版本',
          trailing: MobileSettingTrailing(
            text: ApplicationInfo.applicationVersion,
            showArrow: false,
          ),
        ),
        MobileSettingItem(
          name: '构建号',
          trailing: MobileSettingTrailing(
            text: ApplicationInfo.buildNumber,
            showArrow: false,
          ),
        ),
        MobileSettingItem(
          name: '平台',
          trailing: MobileSettingTrailing(
            text: ApplicationInfo.os,
            showArrow: false,
          ),
        ),
      ],
    );
  }

  /// 构建更新检查组
  Widget _buildUpdateCheckGroup(BuildContext context) {
    return MobileSettingGroup(
      groupTitle: '更新检查',
      settingItemList: [
        MobileSettingItem(
          name: '检查更新',
          subtitle: _buildUpdateStatusSubtitle(context),
          trailing: _isChecking
              ? SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    color: AppFlowyTheme.of(context).textColorScheme.primary,
                  ),
                )
              : const Icon(Icons.refresh),
          onTap: _isChecking ? null : _checkForUpdates,
        ),
        if (_updateInfo != null && _hasChecked) ...[
          MobileSettingItem(
            name: '最新版本',
            trailing: MobileSettingTrailing(
              text: _updateInfo!.latestVersion,
              showArrow: false,
            ),
          ),
          if (_updateInfo?.downloadUrl != null)
            MobileSettingItem(
              name: '下载更新',
              trailing: const Icon(Icons.download),
              onTap: _openDownloadLink,
            ),
        ],
        if (_errorMessage != null && _hasChecked)
          MobileSettingItem(
            name: '检查失败',
            subtitle: Text(
              _errorMessage!,
              style: AppFlowyTheme.of(context).textStyle.caption.enhanced(
                color: AppFlowyTheme.of(context).textColorScheme.error,
              ),
            ),
            trailing: MobileSettingTrailing(
              text: '',
              showArrow: false,
            ),
          ),
      ],
    );
  }

  /// 构建更新状态副标题
  Widget _buildUpdateStatusSubtitle(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    if (_isChecking) {
      return Text(
        '检查中...',
        style: theme.textStyle.caption.enhanced(
          color: theme.textColorScheme.tertiary,
        ),
      );
    }
    
    if (_hasChecked && _updateInfo != null) {
      final isUpdateAvailable = _updateInfo!.isUpdateAvailable;
      return Text(
        isUpdateAvailable
            ? '有新版本可用'
            : '已是最新版本',
        style: theme.textStyle.caption.enhanced(
          color: isUpdateAvailable 
              ? theme.textColorScheme.error
              : theme.textColorScheme.success,
        ),
      );
    }
    
    return Text(
      '点击检查',
      style: theme.textStyle.caption.enhanced(
        color: theme.textColorScheme.tertiary,
      ),
    );
  }

  /// 构建更新历史组
  Widget _buildChangelogGroup(BuildContext context) {
    return MobileSettingGroup(
      groupTitle: '更新历史',
      settingItemList: [
        MobileSettingItem(
          name: '查看更新历史',
          subtitle: _buildChangelogSubtitle(context),
          trailing: const Icon(Icons.history),
          onTap: () => _showChangelog(context),
        ),
      ],
    );
  }

  /// 构建changelog副标题
  Widget _buildChangelogSubtitle(BuildContext context) {
    final theme = AppFlowyTheme.of(context);
    
    if (_isLoadingChangelog) {
      return Text(
        '加载中...',
        style: theme.textStyle.caption.enhanced(
          color: theme.textColorScheme.tertiary,
        ),
      );
    }
    
    if (_hasChangelogError) {
      return Text(
        '加载失败',
        style: theme.textStyle.caption.enhanced(
          color: theme.textColorScheme.error,
        ),
      );
    }
    
    return Text(
      '点击查看',
      style: theme.textStyle.caption.enhanced(
        color: theme.textColorScheme.tertiary,
      ),
    );
  }

  /// 构建版权信息组
  Widget _buildCopyrightGroup(BuildContext context) {
    return MobileSettingGroup(
      groupTitle: '版权信息',
      showDivider: false,
      settingItemList: [
        MobileSettingItem(
          name: 'T1Mind',
          subtitle: Text(
            '基于AppFlowy构建的智能笔记应用',
            style: AppFlowyTheme.of(context).textStyle.caption.enhanced(
              color: AppFlowyTheme.of(context).textColorScheme.tertiary,
            ),
          ),
          trailing: MobileSettingTrailing(
            text: '',
            showArrow: false,
          ),
        ),
        MobileSettingItem(
          name: 'AppFlowy',
          subtitle: Text(
            '开源协作平台，提供强大的文档编辑功能',
            style: AppFlowyTheme.of(context).textStyle.caption.enhanced(
              color: AppFlowyTheme.of(context).textColorScheme.tertiary,
            ),
          ),
          trailing: MobileSettingTrailing(
            text: '',
            showArrow: false,
          ),
        ),
        MobileSettingItem(
          name: 'Flutter',
          subtitle: Text(
            'Google开发的跨平台UI框架',
            style: AppFlowyTheme.of(context).textStyle.caption.enhanced(
              color: AppFlowyTheme.of(context).textColorScheme.tertiary,
            ),
          ),
          trailing: MobileSettingTrailing(
            text: '',
            showArrow: false,
          ),
        ),
      ],
    );
  }
}