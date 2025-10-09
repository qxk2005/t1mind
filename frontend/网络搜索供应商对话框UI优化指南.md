# 网络搜索供应商对话框 UI 优化指南

## 概述

本文档提供了优化添加供应商对话框界面的详细指导，使其更美观、更符合最佳实践。

## 已完成的优化

### 1. 对话框标题优化 ✅
- ✅ 添加了图标和副标题
- ✅ 添加了关闭按钮
- ✅ 使用卡片式标题布局

### 2. 供应商选择器优化 ✅
- ✅ 改为卡片式布局，带图标
- ✅ 添加每个供应商的描述
- ✅ 选中状态有明显的视觉反馈

### 3. 信息卡片 ✅
- ✅ 添加了 "如何获取 API 密钥" 的帮助卡片
- ✅ 包含每个供应商的详细说明
- ✅ 添加了文档链接

### 4. 输入字段优化 ✅
- ✅ 添加了必填标记 (*)
- ✅ API 密钥字段添加了显示/隐藏切换
- ✅ 优化了输入框样式和焦点状态

## 需要手动完成的优化

### 5. 测试结果显示优化

在 `_AddWebSearchProviderDialogState` 类中，找到 `_buildTestResult()` 方法（约在第 843 行），替换为：

```dart
Widget _buildTestResult() {
  final isSuccess = _testResult!.contains('success') || _testResult!.contains('successful') || _testResult!.contains('成功');
  
  return Container(
    padding: const EdgeInsets.all(14),
    decoration: BoxDecoration(
      color: isSuccess 
          ? Colors.green.withOpacity(0.08) 
          : Colors.red.withOpacity(0.08),
      borderRadius: BorderRadius.circular(8),
      border: Border.all(
        color: isSuccess 
            ? Colors.green.withOpacity(0.3) 
            : Colors.red.withOpacity(0.3),
        width: 1.5,
      ),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 24,
          height: 24,
          decoration: BoxDecoration(
            color: isSuccess ? Colors.green : Colors.red,
            shape: BoxShape.circle,
          ),
          child: Icon(
            isSuccess ? Icons.check : Icons.close,
            color: Colors.white,
            size: 16,
          ),
        ),
        const HSpace(12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              FlowyText.medium(
                isSuccess ? "连接成功" : "连接失败",
                fontSize: 13,
                color: isSuccess ? Colors.green : Colors.red,
              ),
              const VSpace(4),
              FlowyText.regular(
                _testResult!,
                fontSize: 12,
                color: AFThemeExtension.of(context).secondaryTextColor,
                maxLines: 3,
              ),
            ],
          ),
        ),
      ],
    ),
  );
}
```

### 6. 按钮布局优化

在 `_AddWebSearchProviderDialogState` 类中，找到 `_buildActionButtons()` 方法（约在第 875 行），替换为：

```dart
Widget _buildActionButtons() {
  final canSave = _canSave();
  
  return Row(
    mainAxisAlignment: MainAxisAlignment.end,
    children: [
      // 测试连接按钮
      OutlinedButton.icon(
        onPressed: _isTestingConnection ? null : _testConnection,
        icon: _isTestingConnection
            ? const SizedBox(
                width: 16,
                height: 16,
                child: CircularProgressIndicator(strokeWidth: 2),
              )
            : const Icon(Icons.network_check, size: 18),
        label: Text(_isTestingConnection ? "测试中..." : "测试连接"),
        style: OutlinedButton.styleFrom(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
          side: BorderSide(
            color: Theme.of(context).colorScheme.outline,
          ),
        ),
      ),
      const HSpace(12),
      
      // 取消按钮
      TextButton(
        onPressed: () => Navigator.of(context).pop(),
        style: TextButton.styleFrom(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
        ),
        child: Text(LocaleKeys.button_cancel.tr()),
      ),
      const HSpace(8),
      
      // 保存按钮
      FilledButton.icon(
        onPressed: canSave ? _saveProvider : null,
        icon: const Icon(Icons.check, size: 18),
        label: Text(LocaleKeys.button_save.tr()),
        style: FilledButton.styleFrom(
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
          backgroundColor: canSave 
              ? Theme.of(context).colorScheme.primary 
              : Theme.of(context).colorScheme.surfaceContainerHighest,
        ),
      ),
    ],
  );
}
```

### 7. 添加辅助方法

在 `_AddWebSearchProviderDialogState` 类中，在 `_getProviderTypeName()` 方法之后添加（或替换已有的方法）：

```dart
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
```

同时更新 `_getApiKeyPlaceholder()` 方法：

```dart
String _getApiKeyPlaceholder() {
  switch (_selectedProvider) {
    case WebSearchProviderType.tavily:
      return "tvly-xxxxxxxxxxxxxxxxxxxxxxxx";
    case WebSearchProviderType.brave:
      return "BSAxxxxxxxxxxxxxxxxxxxxxxxx";
  }
}
```

## UI 优化效果

### 改进前
- 简单的下拉框选择
- 普通的输入框
- 平铺的按钮布局
- 缺少视觉引导

### 改进后
- 📱 卡片式供应商选择，带图标和描述
- 🎨 现代化的输入框设计
- 📝 详细的帮助信息和文档链接
- 🔐 API 密钥显示/隐藏切换
- ✨ 优雅的按钮布局和视觉反馈
- 📏 更好的间距和视觉层次

## 文件位置

```
/Users/niuzhidao/Documents/Program/t1mind/frontend/appflowy_flutter/lib/plugins/ai_chat/widgets/web_search_settings.dart
```

主要修改类：
- `_AddWebSearchProviderDialog` (第 459 行)
- `_AddWebSearchProviderDialogState` (第 466 行)

## 注意事项

1. **保持一致性**: 确保修改只在 `_AddWebSearchProviderDialogState` 类中进行，不要影响 `_ConfigureWebSearchProviderDialog` 类
2. **测试功能**: 修改后测试所有功能是否正常工作
3. **响应式布局**: 确保在不同屏幕尺寸下都能正常显示
4. **主题适配**: 新的 UI 应该适配亮色和暗色主题

## 后续建议

1. **打开文档链接**: 实现 "查看详细文档" 链接的功能，打开对应供应商的 API 文档页面
2. **实际测试连接**: 将模拟的测试连接改为调用实际的 API 测试
3. **表单验证**: 添加更详细的表单验证，比如检查 API 密钥格式
4. **加载动画**: 在保存时添加更明显的加载反馈

---

**最后更新**: 2025-10-08
**状态**: 部分完成（核心 UI 已优化，需手动完成最后几步）



