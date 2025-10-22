import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/shared/version_checker/t1mind_version_checker.dart';
import 'package:appflowy/util/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';

/// T1Mind专用的侧边栏更新按钮组件
/// 使用T1Mind版本检查器来显示更新通知
class T1MindSidebarUpgradeButton extends StatefulWidget {
  const T1MindSidebarUpgradeButton({
    super.key,
    required this.onCloseButtonTap,
  });

  final VoidCallback onCloseButtonTap;

  @override
  State<T1MindSidebarUpgradeButton> createState() => _T1MindSidebarUpgradeButtonState();
}

class _T1MindSidebarUpgradeButtonState extends State<T1MindSidebarUpgradeButton> {
  T1MindUpdateInfo? _updateInfo;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadUpdateInfo();
  }

  Future<void> _loadUpdateInfo() async {
    try {
      final updateInfo = await T1MindVersionChecker().getUpdateInfo();
      if (mounted) {
        setState(() {
          _updateInfo = updateInfo;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    // 如果正在加载或没有更新可用，不显示
    if (_isLoading || 
        _updateInfo?.isUpdateAvailable != true || 
        _updateInfo!.latestVersion == 'No releases available') {
      return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: context.sidebarUpgradeButtonBackground,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // title
          _buildTitle(),
          const VSpace(2),
          // description
          _buildDescription(),
          const VSpace(10),
          // update button
          _buildUpdateButton(),
        ],
      ),
    );
  }

  Widget _buildTitle() {
    return Row(
      children: [
        const FlowySvg(
          FlowySvgs.sidebar_upgrade_version_s,
          blendMode: null,
        ),
        const HSpace(6),
        FlowyText.medium(
          '新版本可用！',
          fontSize: 14,
          figmaLineHeight: 18,
        ),
        const Spacer(),
        FlowyButton(
          useIntrinsicWidth: true,
          text: const FlowySvg(FlowySvgs.upgrade_close_s),
          onTap: widget.onCloseButtonTap,
        ),
      ],
    );
  }

  Widget _buildDescription() {
    return Opacity(
      opacity: 0.7,
      child: FlowyText(
        '获取最新功能和修复。点击"更新"立即安装。',
        fontSize: 13,
        figmaLineHeight: 16,
        maxLines: null,
      ),
    );
  }

  Widget _buildUpdateButton() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: () {
          if (_updateInfo?.downloadUrl.isNotEmpty == true) {
            // 这里可以添加打开下载链接的逻辑
            // afLaunchUrlString(_updateInfo!.downloadUrl);
          }
        },
        child: Container(
          padding: const EdgeInsets.symmetric(
            horizontal: 10,
            vertical: 6,
          ),
          decoration: ShapeDecoration(
            color: const Color(0xFFA44AFD),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(9),
            ),
          ),
          child: FlowyText.medium(
            '更新',
            color: Colors.white,
            fontSize: 12.0,
            figmaLineHeight: 15.0,
          ),
        ),
      ),
    );
  }
}

extension on BuildContext {
  Color get sidebarUpgradeButtonBackground => Theme.of(this).isLightMode
      ? const Color(0xB2EBE4FF)
      : const Color(0xB239275B);
}
