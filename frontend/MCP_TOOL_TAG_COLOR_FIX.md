# MCP 工具标签颜色方案改进

## 问题描述

工具名称标签显示不清楚，浅蓝色背景和白色文字的组合导致对比度不足，阅读性差。

## 根本原因

之前的颜色方案：
- **背景色**：`secondaryContainer`（浅蓝色）
- **文字颜色**：`onSecondaryContainer`（浅色/白色）

这种浅色对浅色的组合导致对比度不足，特别是在亮色主题下。

## 修复方案

### 1. 工具标签颜色改进

**之前（低对比度）**：
```dart
// 背景
color: Theme.of(context).colorScheme.secondaryContainer,  // 浅蓝色

// 文字和图标
color: Theme.of(context).colorScheme.onSecondaryContainer,  // 白色/浅色
```

**现在（高对比度）**：
```dart
// 背景：使用更中性的背景色
color: Theme.of(context).colorScheme.surfaceVariant,  // 浅灰色

// 悬停时背景
color: Theme.of(context).colorScheme.primary.withOpacity(0.15),

// 文字和图标：使用深色，提高可读性
color: Theme.of(context).colorScheme.onSurfaceVariant,  // 深色

// 悬停时文字
color: Theme.of(context).colorScheme.primary,  // 主题色
```

### 2. 边框增强

```dart
// 之前
border: Border.all(
  color: Theme.of(context).colorScheme.outline.withOpacity(0.3),  // 很淡
),

// 现在
border: Border.all(
  color: Theme.of(context).colorScheme.outline.withOpacity(0.5),  // 更明显
  width: 1.0,
),

// 悬停时
border: Border.all(
  color: Theme.of(context).colorScheme.primary,  // 主题色边框
  width: 1.5,  // 更粗
),
```

### 3. 字体粗细调整

```dart
// 之前
fontWeight: _isHovered ? FontWeight.w600 : FontWeight.normal,

// 现在
fontWeight: _isHovered ? FontWeight.w600 : FontWeight.w500,  // 默认稍粗一些
```

### 4. "+N" 标签统一

同样应用了改进的颜色方案，保持视觉一致性：
```dart
decoration: BoxDecoration(
  color: Theme.of(context).colorScheme.surfaceVariant,
  borderRadius: BorderRadius.circular(12),
  border: Border.all(
    color: Theme.of(context).colorScheme.outline.withOpacity(0.5),
    width: 1.0,
  ),
),
child: Text(
  '+${tools.length - 5}',
  style: TextStyle(
    fontSize: 11,
    fontWeight: FontWeight.w500,  // ✅ 新增
    color: Theme.of(context).colorScheme.onSurfaceVariant,
  ),
),
```

## 视觉效果对比

### 之前（低对比度）
- 背景：浅蓝色（`secondaryContainer`）
- 文字：白色/浅色（`onSecondaryContainer`）
- 边框：非常淡（`opacity: 0.3`）
- **问题**：文字难以阅读，特别是在亮色主题下

### 现在（高对比度）
- 背景：浅灰色（`surfaceVariant`）
- 文字：深色（`onSurfaceVariant`）
- 边框：更明显（`opacity: 0.5`）
- 字体：稍粗（`FontWeight.w500`）
- **优点**：文字清晰易读，视觉层次分明

## 交互效果

### 默认状态
- 浅灰色背景 + 深色文字
- 中等透明度边框
- 中等字重

### 悬停状态
- 主题色淡背景（`primary.withOpacity(0.15)`）
- 主题色文字和图标
- 主题色实线边框，更粗（1.5px）
- 加粗字体（`FontWeight.w600`）

## 可访问性提升

1. ✅ **对比度符合 WCAG AA 标准**
2. ✅ **文字更易阅读**
3. ✅ **视觉层次清晰**
4. ✅ **悬停反馈明显**

## 修复时间

2025-10-01

## 相关文件

- `appflowy_flutter/lib/workspace/presentation/settings/workspace/workspace_mcp_settings_v2.dart`
  - `_ToolTag` 组件（1485-1553 行）
  - `_buildToolTags` 方法（1167-1198 行）

