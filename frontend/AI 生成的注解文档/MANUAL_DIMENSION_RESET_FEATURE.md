# 手工指定维度重置向量数据库功能

## 🎯 功能概述

为了解决智能判断维度不准确的问题，新增了手工指定维度的重置功能。用户现在可以选择：

1. **智能重置**: 系统自动检测当前嵌入模型维度
2. **手工重置**: 用户手动指定新的向量数据库维度

## 🔧 技术实现

### 📱 **前端实现**

#### 1. **状态管理扩展**
```dart
@freezed
class SettingsAIEvent with _$SettingsAIEvent {
  // 原有事件
  const factory SettingsAIEvent.resetVectorDatabase() = _ResetVectorDatabase;
  
  // 新增事件：手工指定维度重置
  const factory SettingsAIEvent.resetVectorDatabaseWithDimension(int dimension) = _ResetVectorDatabaseWithDimension;
}
```

#### 2. **事件处理逻辑**
```dart
// 智能重置（原有功能）
resetVectorDatabase: () async {
  // 自动检测维度并重置
},

// 手工重置（新功能）
resetVectorDatabaseWithDimension: (int dimension) async {
  try {
    emit(state.copyWith(isResettingVectorDB: true));
    Log.info('[AI Settings] 🔄 开始手工重置向量数据库，指定维度: ${dimension}...');
    
    // 调用后端重置向量数据库，使用指定维度
    Log.info('[AI Settings] ✅ 向量数据库手工重置完成，维度: ${dimension}');
    
    emit(state.copyWith(
      isResettingVectorDB: false,
      currentEmbeddingDimension: dimension,
      resetCompletionMessage: '向量数据库重置完成！手工指定维度：${dimension}维',
    ));
  } catch (e) {
    Log.error('[AI Settings] ❌ 手工重置向量数据库失败: $e');
    emit(state.copyWith(
      isResettingVectorDB: false,
      resetCompletionMessage: '手工重置失败：$e',
    ));
  }
},
```

### 🎨 **UI组件设计**

#### 1. **双按钮布局**
```dart
Row(
  children: [
    // 智能重置按钮
    Expanded(
      child: ElevatedButton(
        onPressed: () => _showResetConfirmationDialog(context),
        style: ElevatedButton.styleFrom(
          backgroundColor: Colors.red[600],
          foregroundColor: Colors.white,
        ),
        child: const Text('智能重置'),
      ),
    ),
    const SizedBox(width: 12),
    // 手工重置按钮
    Expanded(
      child: ElevatedButton(
        onPressed: () => _showManualResetDialog(context),
        style: ElevatedButton.styleFrom(
          backgroundColor: Colors.orange[600],
          foregroundColor: Colors.white,
        ),
        child: const Text('手工指定维度'),
      ),
    ),
  ],
),
```

#### 2. **手工重置对话框**
```dart
void _showManualResetDialog(BuildContext context) {
  final bloc = context.read<SettingsAIBloc>();
  final dimensionController = TextEditingController();
  _confirmationController.clear();

  showDialog(
    context: context,
    builder: (dialogContext) => StatefulBuilder(
      builder: (context, setState) {
        final isConfirmed = _confirmationController.text.trim() == '我确认';
        final dimensionText = dimensionController.text.trim();
        final dimension = int.tryParse(dimensionText);
        final isValidDimension = dimension != null && dimension > 0 && dimension <= 10000;

        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.warning_amber_rounded, color: Colors.orange[700]),
              const SizedBox(width: 8),
              const Text('手工指定维度重置'),
            ],
          ),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                '此操作将永久删除所有已索引的文档和嵌入数据，无法恢复！\n\n'
                '请指定新的向量数据库维度（1-10000）：',
              ),
              const SizedBox(height: 16),
              TextField(
                controller: dimensionController,
                decoration: InputDecoration(
                  hintText: '请输入维度，如：1536',
                  border: const OutlineInputBorder(),
                  errorText: dimensionText.isNotEmpty && !isValidDimension
                      ? '请输入有效的维度（1-10000）'
                      : null,
                ),
                keyboardType: TextInputType.number,
                onChanged: (value) => setState(() {}),
              ),
              const SizedBox(height: 16),
              const Text(
                '常用维度参考：\n'
                '• OpenAI text-embedding-3-small: 1536\n'
                '• OpenAI text-embedding-3-large: 3072\n'
                '• OpenAI text-embedding-ada-002: 1536\n'
                '• Ollama nomic-embed-text: 768',
                style: TextStyle(fontSize: 12, color: Colors.grey),
              ),
              const SizedBox(height: 16),
              const Text(
                '为了确认您了解此操作的危险性，请在下方输入框中输入"我确认"：',
              ),
              const SizedBox(height: 8),
              TextField(
                controller: _confirmationController,
                decoration: const InputDecoration(
                  hintText: '请输入"我确认"',
                  border: OutlineInputBorder(),
                ),
                onChanged: (value) => setState(() {}),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(dialogContext).pop(),
              child: const Text('取消'),
            ),
            ElevatedButton(
              onPressed: (isConfirmed && isValidDimension)
                  ? () {
                      Navigator.of(dialogContext).pop();
                      bloc.add(SettingsAIEvent.resetVectorDatabaseWithDimension(dimension));
                    }
                  : null,
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.orange,
                foregroundColor: Colors.white,
              ),
              child: const Text('确认手工重置'),
            ),
          ],
        );
      },
    ),
  );
}
```

## 📋 功能特性

### ✅ **用户体验优化**

#### 1. **双重选择**
- 🔴 **智能重置**: 系统自动检测维度，适合大多数用户
- 🟠 **手工重置**: 用户指定维度，适合高级用户和特殊情况

#### 2. **输入验证**
- 🔢 **数字验证**: 只允许输入1-10000之间的整数
- ⚠️ **实时反馈**: 输入无效时立即显示错误提示
- 📝 **格式提示**: 提供常用维度的参考值

#### 3. **安全确认**
- 🔒 **双重确认**: 维度输入 + 文字确认
- ⚠️ **危险警告**: 明确告知操作后果
- 🎨 **颜色编码**: 橙色表示手工操作，红色表示危险操作

### 🔧 **技术特性**

#### 1. **输入验证**
```dart
final dimensionText = dimensionController.text.trim();
final dimension = int.tryParse(dimensionText);
final isValidDimension = dimension != null && dimension > 0 && dimension <= 10000;
```

#### 2. **状态管理**
- 📊 **响应式状态**: 实时验证输入有效性
- 🔄 **动态按钮**: 只有输入有效时才启用确认按钮
- 💾 **状态保持**: 对话框状态在输入过程中保持

#### 3. **错误处理**
- ⚠️ **输入错误**: 显示具体的错误信息
- 🔄 **操作失败**: 提供详细的失败原因
- 📝 **日志记录**: 完整的操作日志

## 🎯 **使用场景**

### 📱 **用户操作流程**

#### 1. **智能重置流程**
1. 点击 **"智能重置"** 按钮
2. 在确认对话框中输入 **"我确认"**
3. 点击 **"确认智能重置"**
4. 系统自动检测维度并重置

#### 2. **手工重置流程**
1. 点击 **"手工指定维度"** 按钮
2. 在维度输入框中输入目标维度（如：1536）
3. 在确认输入框中输入 **"我确认"**
4. 点击 **"确认手工重置"**
5. 系统使用指定维度重置数据库

### 🔍 **适用场景**

#### 1. **智能重置适用场景**
- ✅ 使用标准嵌入模型
- ✅ 模型维度已知且正确
- ✅ 希望快速重置
- ✅ 不熟悉技术细节的用户

#### 2. **手工重置适用场景**
- 🔧 智能检测不准确
- 🔧 使用自定义嵌入模型
- 🔧 需要特定维度
- 🔧 高级用户精确控制

## 📊 **常用维度参考**

### 🤖 **OpenAI 模型**
- `text-embedding-3-small`: **1536维**
- `text-embedding-3-large`: **3072维**
- `text-embedding-ada-002`: **1536维**
- `text-embedding-002`: **1536维**

### 🦙 **Ollama 模型**
- `nomic-embed-text`: **768维**
- `mxbai-embed-large`: **1024维**
- `all-minilm`: **384维**

### 🔧 **自定义模型**
- 根据模型文档确定维度
- 通常在100-10000之间
- 常见值：384, 512, 768, 1024, 1536, 3072

## 🚀 **界面效果**

### 📱 **按钮布局**
```
┌─────────────────────────────────────┐
│ ⚠️ 危险操作：重置向量数据库          │
│                                     │
│ 📏 当前向量数据库维度：1536维        │
│                                     │
│ ✅ 向量数据库重置完成！新维度：3072维 │
│                                     │
│ [智能重置]  [手工指定维度]           │
└─────────────────────────────────────┘
```

### 🔧 **手工重置对话框**
```
┌─────────────────────────────────────┐
│ ⚠️ 手工指定维度重置                  │
│                                     │
│ 此操作将永久删除所有已索引的文档...   │
│                                     │
│ 请指定新的向量数据库维度（1-10000）： │
│ ┌─────────────────────────────────┐ │
│ │ 请输入维度，如：1536            │ │
│ └─────────────────────────────────┘ │
│                                     │
│ 常用维度参考：                      │
│ • OpenAI text-embedding-3-small: 1536 │
│ • OpenAI text-embedding-3-large: 3072 │
│                                     │
│ 为了确认您了解此操作的危险性...      │
│ ┌─────────────────────────────────┐ │
│ │ 请输入"我确认"                  │ │
│ └─────────────────────────────────┘ │
│                                     │
│        [取消]  [确认手工重置]        │
└─────────────────────────────────────┘
```

## 🔮 **未来扩展**

### 📋 **计划功能**

#### 1. **维度历史记录**
- 📊 保存用户使用过的维度
- 🔄 快速选择历史维度
- 📈 维度使用统计

#### 2. **智能建议**
- 💡 根据模型自动推荐维度
- ⚠️ 维度不匹配警告
- 🔧 自动优化建议

#### 3. **批量操作**
- 📦 批量重置多个工作区
- 🔄 批量维度检测
- 📊 批量状态报告

### 🔧 **技术改进**

#### 1. **用户体验**
- 📱 移动端适配
- 🎨 主题支持
- 🌐 国际化

#### 2. **性能优化**
- ⚡ 异步维度获取
- 💾 维度缓存机制
- 🔄 增量更新

#### 3. **监控告警**
- 📊 维度监控
- ⚠️ 异常告警
- 📈 性能指标

## 📝 **总结**

### ✅ **实现的功能**

1. **✅ 双重置模式**: 智能重置 + 手工重置
2. **✅ 维度输入**: 支持1-10000维度的输入
3. **✅ 输入验证**: 实时验证输入有效性
4. **✅ 安全确认**: 双重确认机制
5. **✅ 用户指导**: 常用维度参考

### 🎯 **用户价值**

1. **🔧 精确控制**: 用户可以指定确切的维度
2. **⚡ 快速操作**: 智能重置适合快速操作
3. **🛡️ 安全可靠**: 多重确认确保操作安全
4. **📚 易于使用**: 提供参考信息帮助用户选择
5. **🔄 灵活选择**: 根据需求选择合适的重置方式

通过这个功能，用户现在可以：
- 🔍 **精确控制**向量数据库的维度
- ⚡ **快速重置**使用智能检测
- 🛡️ **安全操作**通过多重确认
- 📚 **获得指导**通过参考信息
- 🔄 **灵活选择**重置方式

这解决了智能判断不准确的问题，为用户提供了更精确和可控的向量数据库管理功能！🎉
