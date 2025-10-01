# MCP一键检查工具功能实现

## 功能概述

为MCP配置页面添加了一键检查所有服务器并自动探测工具的功能，并在服务器卡片上用标签形式显示工具列表，支持鼠标悬停查看工具描述。

## 实现的功能

### 1. 一键检查所有服务器 ✅

**位置**：服务器列表标题栏右侧

**功能**：
- 点击"一键检查"按钮自动连接所有未连接的服务器
- 自动加载所有已连接服务器的工具列表
- 对于已连接但未加载工具的服务器，自动重新加载工具
- 显示检查进度提示

**实现方法**：
```dart
void _checkAllServers(BuildContext context, MCPSettingsState state) {
  final bloc = context.read<MCPSettingsBloc>();
  
  for (final server in state.servers) {
    final isConnected = state.serverStatuses[server.id]?.isConnected ?? false;
    final hasTools = state.serverTools[server.id]?.isNotEmpty ?? false;
    
    if (!isConnected) {
      bloc.add(MCPSettingsEvent.connectServer(server.id));
    } else if (!hasTools && !state.loadingTools.contains(server.id)) {
      bloc.add(MCPSettingsEvent.refreshTools(server.id));
    }
  }
  
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text('正在检查 ${state.servers.length} 个服务器...')),
  );
}
```

### 2. 工具标签显示 ✅

**位置**：服务器卡片底部

**功能**：
- 显示最多5个工具标签
- 如果工具超过5个，显示"+N"标签表示剩余数量
- 每个标签包含工具图标和名称
- 标签使用圆角矩形设计，视觉美观

**显示效果**：
```
┌────────────────────────────────────────┐
│ 服务器名称                      STDIO  │
│ 描述信息...                           │
│ 命令: /path/to/server                 │
│                                        │
│ [🔧 search] [🔧 query] [🔧 analyze]   │
│ [🔧 fetch] [🔧 write] [+3]            │
└────────────────────────────────────────┘
```

### 3. 工具描述悬停显示 ✅

**交互方式**：鼠标悬停在工具标签上

**功能**：
- 使用Tooltip显示完整的工具名称和描述
- 悬停时标签背景色和边框颜色变化，提供视觉反馈
- 标签文字加粗，图标颜色变为主题色
- 平滑的动画过渡效果（150ms）

**实现的Widget**：
```dart
class _ToolTag extends StatefulWidget {
  // 支持悬停状态
  // 显示Tooltip
  // AnimatedContainer动画效果
  // MouseRegion处理鼠标事件
}
```

**Tooltip内容格式**：
```
工具名称

工具的详细描述说明...
```

## 文件修改

### 修改的文件
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/workspace_mcp_settings_v2.dart`
  - 添加 `_checkAllServers` 方法
  - 修改 `_buildServerList` 添加"一键检查"按钮
  - 在 `_ServerCard` 中添加 `_buildToolTags` 方法
  - 新增 `_ToolTag` widget类

### 代码统计
- 新增代码：约80行
- 新增Widget：1个（`_ToolTag`）
- 新增方法：2个（`_checkAllServers`, `_buildToolTags`）

## 用户体验改进

### 1. 操作便捷性
- **之前**：需要逐个点击服务器的连接按钮，再点击加载工具
- **现在**：一键完成所有服务器的连接和工具探测

### 2. 信息可视化
- **之前**：只显示工具数量徽章，需要点击查看具体工具
- **现在**：直接在卡片上显示工具标签，一目了然

### 3. 交互反馈
- **之前**：无法快速了解工具用途
- **现在**：鼠标悬停即可查看工具描述，无需额外点击

## 视觉设计

### 一键检查按钮
- **样式**：OutlinedButton
- **图标**：search（搜索图标）
- **位置**：服务器列表标题右侧，"添加服务器"按钮左侧
- **颜色**：跟随主题色

### 工具标签
- **默认状态**：
  - 背景：`secondaryContainer`
  - 边框：半透明outline
  - 文字：`onSecondaryContainer`
  - 图标：functions图标

- **悬停状态**：
  - 背景：`primaryContainer`
  - 边框：半透明primary
  - 文字：加粗，`primary`颜色
  - 图标：`primary`颜色

## 技术实现要点

### 1. 状态管理
- 利用现有的`MCPSettingsBloc`
- 使用已有的连接和加载工具事件
- 无需添加新的BLoC事件

### 2. UI组件
- 使用Flutter的`Tooltip` widget
- 使用`MouseRegion`监听鼠标事件
- 使用`AnimatedContainer`实现平滑动画

### 3. 性能优化
- 只显示前5个工具标签，避免UI过于拥挤
- 使用`Wrap`布局自动换行
- 悬停状态使用局部重建

## 后续优化建议

1. **移动端适配**
   - 移动端可以使用长按显示工具描述
   - 或者使用GestureDetector实现点击展开

2. **工具过滤**
   - 添加工具搜索/过滤功能
   - 支持按工具类型分类显示

3. **批量操作**
   - 添加"断开所有连接"功能
   - 添加"刷新所有工具"功能

4. **状态指示**
   - 为"一键检查"按钮添加loading状态
   - 显示检查进度条

## 测试建议

### 功能测试
1. ✅ 点击"一键检查"按钮，验证所有服务器是否连接
2. ✅ 验证工具标签是否正确显示
3. ✅ 鼠标悬停在工具标签上，验证Tooltip是否显示
4. ✅ 验证超过5个工具时是否显示"+N"标签

### 交互测试
1. ✅ 验证标签悬停时的视觉反馈
2. ✅ 验证动画过渡是否流畅
3. ✅ 验证Tooltip显示延迟是否合理（300ms）

### 边界测试
1. ✅ 0个工具时不显示标签区域
2. ✅ 1-5个工具时正常显示
3. ✅ 超过5个工具时显示"+N"
4. ✅ 工具描述为空时Tooltip只显示名称

## 完成日期

2025-10-01

## 相关文件

- 实现文件：`workspace_mcp_settings_v2.dart`
- BLoC文件：`mcp_settings_bloc.dart`
- 相关文档：`MCP_FOLD_AWAIT_FIX.md`

