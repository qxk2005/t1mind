# MCP 连接状态 UI 改进

## 改进目标

在全局设置的 MCP 配置列表中,为每个 MCP 服务器添加更清晰、更显眼的连接状态显示,帮助用户快速判断服务是否可用。

## 改进内容

### 1. 新增连接状态徽章

在每个 MCP 服务器卡片的标题区域添加了一个彩色状态徽章,显示三种状态:

#### ✅ 已连接 (Connected)
- **背景色**: 绿色浅色背景 (`Colors.green.shade50`)
- **文字颜色**: 深绿色 (`Colors.green.shade700`)
- **图标**: ✓ 勾选圆圈图标 (`Icons.check_circle_outline`)
- **文本**: "已连接"
- **含义**: MCP 服务器已成功连接并可正常使用

#### ❌ 错误 (Error)
- **背景色**: 红色浅色背景 (`Colors.red.shade50`)
- **文字颜色**: 深红色 (`Colors.red.shade700`)
- **图标**: ⚠ 错误图标 (`Icons.error_outline`)
- **文本**: "错误"
- **Tooltip**: 悬停时显示具体错误信息
- **含义**: 连接失败或服务器出现错误

#### ⭕ 未连接 (Disconnected)
- **背景色**: 灰色浅色背景 (`Colors.grey.shade200`)
- **文字颜色**: 深灰色 (`Colors.grey.shade700`)
- **图标**: ○ 空心圆圈图标 (`Icons.radio_button_unchecked`)
- **文本**: "未连接"
- **含义**: 服务器配置已保存但尚未建立连接

### 2. 状态徽章特性

#### 视觉特性
- **彩色背景**: 使用浅色背景配合深色文字,确保可读性
- **圆角边框**: 12px 圆角,配有半透明边框
- **紧凑布局**: 图标和文字紧密排列,占用空间小
- **统一大小**: 所有状态徽章尺寸一致,保持界面整齐

#### 交互特性
- **Tooltip 提示**: 鼠标悬停时显示详细状态信息
  - 已连接/未连接: 显示状态文本
  - 错误状态: 显示完整的错误信息
- **视觉反馈**: 通过颜色变化提供即时的状态反馈

### 3. 布局调整

状态徽章插入在服务器名称和工具数量徽章之间:

```
[服务器名称] [连接状态徽章] [工具数量徽章] [加载动画]
```

移除了原来右侧的小图标(`Icons.check_circle` / `Icons.circle`),因为新的状态徽章更加明显。

## 实现细节

### 代码位置
- **文件**: `appflowy_flutter/lib/workspace/presentation/settings/workspace/workspace_mcp_settings_v2.dart`
- **类**: `_ServerCard`
- **方法**: `_buildConnectionStatusBadge()`

### 核心实现

```dart
Widget _buildConnectionStatusBadge(
  BuildContext context,
  bool isConnected,
  String? errorMessage,
) {
  final Color bgColor;
  final Color textColor;
  final IconData icon;
  final String statusText;

  if (errorMessage != null && errorMessage.isNotEmpty) {
    // 连接错误状态
    bgColor = Colors.red.shade50;
    textColor = Colors.red.shade700;
    icon = Icons.error_outline;
    statusText = '错误';
  } else if (isConnected) {
    // 已连接状态
    bgColor = Colors.green.shade50;
    textColor = Colors.green.shade700;
    icon = Icons.check_circle_outline;
    statusText = '已连接';
  } else {
    // 未连接状态
    bgColor = Colors.grey.shade200;
    textColor = Colors.grey.shade700;
    icon = Icons.radio_button_unchecked;
    statusText = '未连接';
  }

  return Tooltip(
    message: errorMessage ?? statusText,
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: bgColor,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: textColor.withOpacity(0.3),
          width: 1.0,
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 12, color: textColor),
          const SizedBox(width: 4),
          Text(
            statusText,
            style: TextStyle(
              color: textColor,
              fontSize: 11,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    ),
  );
}
```

### 数据来源

连接状态信息来自 `MCPServerStatusPB`:

```dart
pub struct MCPServerStatusPB {
  pub server_id: String,        // 服务器ID
  pub is_connected: bool,        // 是否已连接
  pub error_message: Option<String>,  // 错误信息(可选)
  pub tool_count: i32,           // 工具数量
}
```

## UI 效果对比

### 改进前
- 右侧有一个小的绿色勾号或灰色圆圈图标
- 状态不够明显,容易被忽略
- 没有错误状态的视觉区分
- 无法快速了解错误原因

### 改进后
- 醒目的彩色状态徽章,位于服务器名称旁边
- 三种状态一目了然:绿色(已连接)、红色(错误)、灰色(未连接)
- 鼠标悬停可查看详细错误信息
- 视觉层次清晰,提升用户体验

## 用户场景

### 场景 1: 检查服务器可用性
用户打开 MCP 配置页面,快速扫视所有服务器的连接状态徽章:
- 看到绿色"已连接"徽章 → 服务器正常,可以使用
- 看到灰色"未连接"徽章 → 需要点击"连接"按钮
- 看到红色"错误"徽章 → 需要查看错误信息并修复配置

### 场景 2: 排查连接问题
用户发现某个服务器显示红色"错误"徽章:
1. 鼠标悬停在徽章上
2. Tooltip 显示具体错误信息,如: "Failed to connect to HTTP endpoint: Connection refused"
3. 根据错误信息调整服务器配置(URL、命令等)
4. 重新连接

### 场景 3: 批量管理服务器
用户配置了多个 MCP 服务器:
- 通过状态徽章快速识别哪些服务器需要注意
- 使用"一键检查"功能连接所有未连接的服务器
- 观察状态徽章从灰色变为绿色或红色

## 与其他功能的配合

### 1. 工具数量徽章
- 连接状态徽章和工具数量徽章互补
- 连接状态徽章显示服务器是否可用
- 工具数量徽章显示服务器提供的工具数量
- 工具徽章的颜色也根据连接状态变化(已连接=蓝色,未连接=灰色)

### 2. 一键检查功能
- 点击"一键检查"按钮后,会尝试连接所有未连接的服务器
- 连接过程中,状态徽章保持当前状态
- 连接完成后,状态徽章更新为最新状态

### 3. 连接/断开按钮
- 用户可以手动点击"连接"或"断开"按钮
- 状态徽章实时反映连接状态的变化

## 技术优势

### 1. 响应式设计
- 使用 `BlocConsumer` 监听 `MCPSettingsBloc` 的状态变化
- 服务器状态更新时,UI 自动刷新
- 无需手动触发界面更新

### 2. 可扩展性
- 状态判断逻辑集中在 `_buildConnectionStatusBadge` 方法中
- 如需添加新状态(如"连接中"),只需修改此方法
- 保持代码的可维护性

### 3. 性能优化
- 状态徽章是轻量级的 Widget
- 使用 `const` 构造函数减少重建
- 避免不必要的重绘

## 国际化支持

当前版本使用中文硬编码文本,未来可以改进为:

```dart
// 使用国际化
statusText = LocaleKeys.settings_mcpPage_status_connected.tr();
```

状态文本:
- `connected` → "已连接" / "Connected"
- `disconnected` → "未连接" / "Disconnected"
- `error` → "错误" / "Error"

## 测试建议

### 手动测试
1. **已连接状态**: 配置并连接一个 STDIO MCP 服务器,验证绿色徽章显示
2. **未连接状态**: 添加新服务器但不连接,验证灰色徽章显示
3. **错误状态**: 配置一个错误的 URL(如 `http://invalid-url:9999`),连接后验证红色徽章显示
4. **Tooltip 测试**: 鼠标悬停在各种状态的徽章上,验证提示信息正确
5. **状态切换**: 点击连接/断开按钮,验证徽章颜色实时更新

### 视觉回归测试
- 对比改进前后的 UI 截图
- 确保状态徽章不会影响其他元素的布局
- 验证在不同屏幕尺寸下的显示效果

## 总结

这次 UI 改进通过添加醒目的连接状态徽章,显著提升了 MCP 配置页面的可用性:
- ✅ **可见性提升**: 用户无需仔细寻找,一眼就能看到服务器状态
- ✅ **信息完整**: 显示连接状态并提供错误详情
- ✅ **视觉层次**: 使用颜色编码(绿/红/灰)提供直观的状态反馈
- ✅ **用户体验**: 减少配置错误和排查时间

这使得用户能够更高效地管理和监控 MCP 服务器,提升整体的系统可靠性。

