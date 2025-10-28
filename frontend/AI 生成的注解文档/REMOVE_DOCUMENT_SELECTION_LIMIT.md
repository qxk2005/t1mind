# 移除AI聊天文档选择限制

## 📋 问题描述

用户在使用AI聊天时勾选文档作为信息源，系统提示"已达到信息源限制"。用户希望能够选择任意数量的文档或选择全部文档（作为默认选项）进行RAG检索。

## 🔍 根本原因

在文档选择器中存在硬编码的限制：
- **限制值**: `maxSelectedParentPageCount: 3`
- **位置**: 
  - `appflowy_flutter/lib/ai/widgets/prompt_input/select_sources_menu.dart` (桌面版)
  - `appflowy_flutter/lib/ai/widgets/prompt_input/select_sources_bottom_sheet.dart` (移动版)

这个限制导致用户最多只能选择3个顶级文档（及其子文档）。

## ✅ 解决方案

移除了文档选择的数量限制，将 `maxSelectedParentPageCount` 从 `3` 改为 `null`，允许用户选择任意数量的文档。

### 修改的文件

1. **桌面版文档选择器**
   - **文件**: `appflowy_flutter/lib/ai/widgets/prompt_input/select_sources_menu.dart`
   - **行数**: 43-44
   - **修改**: 将 `maxSelectedParentPageCount: 3` 改为 `maxSelectedParentPageCount: null`

2. **移动版文档选择器**
   - **文件**: `appflowy_flutter/lib/ai/widgets/prompt_input/select_sources_bottom_sheet.dart`
   - **行数**: 41-42
   - **修改**: 将 `maxSelectedParentPageCount: 3` 改为 `maxSelectedParentPageCount: null`

### 实现机制

限制在 `ViewSelectorCubit` 中实现（`appflowy_flutter/lib/ai/service/view_selector_cubit.dart` 第233-250行）：

```dart
void _restrictSelectionIfNecessary(List<ViewSelectorItem> sources) {
  if (maxSelectedParentPageCount == null) {
    return; // 如果没有限制，直接返回，不执行任何限制
  }
  // ... 限制逻辑
}
```

当 `maxSelectedParentPageCount` 为 `null` 时，`_restrictSelectionIfNecessary` 方法会直接返回，不会对文档选择进行任何限制。

## 🎯 影响范围

### ✅ 正面影响
1. **用户体验改善**: 用户不再受到文档选择数量的限制
2. **RAG功能增强**: 可以在更大范围内进行检索增强生成
3. **灵活性提升**: 支持任意数量的文档选择

### ⚠️ 潜在影响
1. **性能考虑**: 选择大量文档可能增加检索时间
2. **上下文大小**: 过多的文档可能超出模型上下文窗口限制
   - 注意：这个问题已经在 `maxToolResultLength` 配置中得到缓解（见 `MAX_TOOL_RESULT_LENGTH_CONFIG.md`）

## 📝 相关配置

如果将来需要设置文档选择限制，可以修改这两个文件中的 `maxSelectedParentPageCount` 参数：

- **设置为 `null`**: 无限制（当前实现）
- **设置为正整数**: 限制顶级文档数量（例如 `5` 表示最多选择5个顶级文档）

## 🔗 相关文档

- [工具结果长度配置](MAX_TOOL_RESULT_LENGTH_CONFIG.md) - 控制检索结果的最大长度
- [RAG设置文档](RAG_SETTINGS.md) - RAG相关配置说明

## ✅ 验证

修改完成后，用户可以：
1. 在AI聊天中选择任意数量的文档作为信息源
2. 不再看到"已达到信息源限制"的错误提示
3. 可以在选择的任意文档中进行RAG检索


