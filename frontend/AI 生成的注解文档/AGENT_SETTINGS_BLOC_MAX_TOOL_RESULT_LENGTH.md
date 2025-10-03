# 智能体设置 BLoC - 工具结果最大长度配置

## 📋 更新说明

为智能体设置 BLoC (`agent_settings_bloc.dart`) 添加了工具结果最大长度 (`maxToolResultLength`) 的配置验证和辅助方法。

## ✅ 新增功能

### 1. 配置验证

**位置**: `_handleValidateAgentConfig` 方法

```dart
// 工具结果最大长度验证
// 0 或负数会使用默认值 4000，但建议明确设置
// 最小值 1000，推荐值 2000-16000
if (capabilities.maxToolResultLength > 0 && 
    (capabilities.maxToolResultLength < 1000 || capabilities.maxToolResultLength > 32000)) {
  validationErrors.add('工具结果最大长度必须在1000-32000字符之间（默认4000）');
}
```

**验证规则**:
- ✅ 允许值：1000 - 32000 字符
- ✅ 特殊值：0 或负数（表示使用默认值 4000）
- ❌ 错误值：1-999（太小）或 > 32000（太大）

### 2. 能力摘要增强

**位置**: `getCapabilitiesSummary` 方法

```dart
String getCapabilitiesSummary(AgentCapabilitiesPB capabilities) {
  final features = <String>[];
  
  if (capabilities.enablePlanning) features.add('规划');
  if (capabilities.enableToolCalling) {
    features.add('工具调用');
    // 显示工具结果最大长度配置
    if (capabilities.maxToolResultLength > 0) {
      features.add('结果长度: ${capabilities.maxToolResultLength}');
    }
  }
  if (capabilities.enableReflection) features.add('反思');
  if (capabilities.enableMemory) features.add('记忆');
  
  return features.isEmpty ? '无特殊能力' : features.join(', ');
}
```

**输出示例**:
```
"规划, 工具调用, 结果长度: 4000, 记忆"
```

### 3. 推荐说明方法

**方法**: `getMaxToolResultLengthRecommendation`

```dart
String getMaxToolResultLengthRecommendation(int? length)
```

**返回值示例**:

| 输入值 | 返回说明 |
|--------|---------|
| null/0/-1 | "使用默认值 (4000字符)" |
| 500 | "过小，将自动调整为最小值 (1000字符)" |
| 1500 | "适用于小型模型 (GPT-3.5等)" |
| 4000 | "标准配置 (推荐)" |
| 8000 | "适用于标准模型 (GPT-4等)" |
| 16000 | "适用于大上下文模型 (Claude等)" |
| 32000 | "超大上下文配置" |
| 50000 | "超出推荐范围，可能超出模型限制" |

### 4. 实际使用值计算

**方法**: `getEffectiveMaxToolResultLength`

```dart
int getEffectiveMaxToolResultLength(int? configuredLength)
```

这个方法计算配置值经过自动修正后的实际使用值：

```dart
// 示例
getEffectiveMaxToolResultLength(null)   // → 4000 (默认值)
getEffectiveMaxToolResultLength(0)      // → 4000 (默认值)
getEffectiveMaxToolResultLength(-10)    // → 4000 (默认值)
getEffectiveMaxToolResultLength(500)    // → 1000 (最小值)
getEffectiveMaxToolResultLength(8000)   // → 8000 (使用配置值)
```

## 🎯 使用示例

### 在 UI 中使用验证

```dart
// 验证智能体配置
final agentConfig = AgentConfigPB()
  ..name = '我的助手'
  ..capabilities = (AgentCapabilitiesPB()
    ..enableToolCalling = true
    ..maxToolResultLength = 2000);

// 触发验证
bloc.add(AgentSettingsEvent.validateAgentConfig(agentConfig));

// 监听验证结果
BlocListener<AgentSettingsBloc, AgentSettingsState>(
  listener: (context, state) {
    if (state.validationResult != null) {
      if (state.validationResult!.isValid) {
        // 验证通过
      } else {
        // 显示错误
        showDialog(
          context: context,
          builder: (_) => AlertDialog(
            title: Text('配置错误'),
            content: Text(state.validationResult!.errorMessage ?? ''),
          ),
        );
      }
    }
  },
);
```

### 显示推荐说明

```dart
// 在配置界面显示推荐说明
final bloc = context.read<AgentSettingsBloc>();
final currentLength = capabilities.maxToolResultLength;
final recommendation = bloc.getMaxToolResultLengthRecommendation(currentLength);

Text(
  recommendation,
  style: TextStyle(
    fontSize: 12,
    color: Colors.grey,
  ),
);
```

### 显示实际使用值

```dart
// 显示实际会使用的值
final effectiveValue = bloc.getEffectiveMaxToolResultLength(
  capabilities.maxToolResultLength
);

Text('实际使用: $effectiveValue 字符');
```

## 🖼️ UI 集成示例

### 配置输入框

```dart
Row(
  children: [
    Expanded(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('工具结果最大长度'),
          SizedBox(height: 4),
          Text(
            '控制多轮对话时传递给 AI 的工具结果大小',
            style: TextStyle(fontSize: 12, color: Colors.grey),
          ),
          SizedBox(height: 4),
          // 显示推荐说明
          Text(
            bloc.getMaxToolResultLengthRecommendation(
              capabilities.maxToolResultLength
            ),
            style: TextStyle(
              fontSize: 11,
              color: Theme.of(context).primaryColor,
            ),
          ),
        ],
      ),
    ),
    SizedBox(width: 16),
    SizedBox(
      width: 120,
      child: TextField(
        keyboardType: TextInputType.number,
        controller: _maxToolResultLengthController,
        decoration: InputDecoration(
          hintText: '4000',
          suffixText: '字符',
          helperText: '范围: 1000-32000',
        ),
        onChanged: (value) {
          final length = int.tryParse(value);
          // 更新配置
          setState(() {
            capabilities.maxToolResultLength = length ?? 0;
          });
        },
      ),
    ),
  ],
)
```

### 能力摘要显示

```dart
// 在智能体列表中显示能力摘要
ListTile(
  title: Text(agent.name),
  subtitle: Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(agent.description),
      SizedBox(height: 4),
      Text(
        bloc.getCapabilitiesSummary(agent.capabilities),
        style: TextStyle(
          fontSize: 11,
          color: Colors.blue,
        ),
      ),
    ],
  ),
)
```

## 📊 验证规则总结

| 配置值 | 验证结果 | 实际使用值 | 说明 |
|--------|---------|-----------|------|
| null | ✅ 通过 | 4000 | 使用默认值 |
| 0 | ✅ 通过 | 4000 | 使用默认值 |
| -10 | ✅ 通过 | 4000 | 使用默认值 |
| 500 | ❌ 失败 | 1000 | 低于最小值 |
| 1000 | ✅ 通过 | 1000 | 最小值 |
| 4000 | ✅ 通过 | 4000 | 推荐值 |
| 8000 | ✅ 通过 | 8000 | 大上下文 |
| 32000 | ✅ 通过 | 32000 | 最大值 |
| 50000 | ❌ 失败 | - | 超出最大值 |

## 🔍 错误消息

当验证失败时，用户会看到：

```
"工具结果最大长度必须在1000-32000字符之间（默认4000）"
```

## 🎨 推荐 UI 布局

### 智能体编辑对话框

```
┌─────────────────────────────────────────┐
│  编辑智能体                               │
├─────────────────────────────────────────┤
│                                         │
│  基本信息                                │
│  ├─ 名称: [           ]                 │
│  └─ 描述: [           ]                 │
│                                         │
│  能力配置                                │
│  ├─ [√] 启用工具调用                     │
│  ├─ 最大工具调用次数: [  20  ]          │
│  └─ 工具结果最大长度: [ 4000 ] 字符      │
│      └─ 标准配置 (推荐) ← 动态提示       │
│                                         │
│  ├─ [√] 启用记忆                        │
│  └─ 会话记忆长度: [ 100 ]               │
│                                         │
│           [取消]    [保存]               │
└─────────────────────────────────────────┘
```

## 📚 相关文档

- [工具结果最大长度配置指南](MAX_TOOL_RESULT_LENGTH_CONFIG.md)
- [多轮对话实现文档](MULTI_TURN_TOOL_CALL_IMPLEMENTATION.md)
- [智能体配置 BLoC 完整文档](agent_settings_bloc.dart)

## ✅ 测试要点

### 单元测试

```dart
test('验证工具结果最大长度 - 正常值', () {
  final config = AgentConfigPB()
    ..capabilities = (AgentCapabilitiesPB()
      ..maxToolResultLength = 4000);
  
  // 触发验证
  bloc.add(AgentSettingsEvent.validateAgentConfig(config));
  
  // 验证结果应该通过
  expect(state.validationResult?.isValid, true);
});

test('验证工具结果最大长度 - 过小', () {
  final config = AgentConfigPB()
    ..capabilities = (AgentCapabilitiesPB()
      ..maxToolResultLength = 500);
  
  bloc.add(AgentSettingsEvent.validateAgentConfig(config));
  
  expect(state.validationResult?.isValid, false);
  expect(state.error, contains('1000-32000'));
});

test('获取推荐说明 - 标准值', () {
  final recommendation = bloc.getMaxToolResultLengthRecommendation(4000);
  expect(recommendation, '标准配置 (推荐)');
});

test('计算实际使用值 - 默认值', () {
  final effective = bloc.getEffectiveMaxToolResultLength(0);
  expect(effective, 4000);
});
```

## 🚀 部署清单

- [x] BLoC 验证逻辑已添加
- [x] 能力摘要方法已更新
- [x] 推荐说明方法已添加
- [x] 实际使用值计算方法已添加
- [ ] UI 配置界面（待前端实现）
- [ ] 单元测试（待添加）
- [ ] 集成测试（待添加）

---

**更新时间**: 2025-10-03  
**文件**: `appflowy_flutter/lib/plugins/ai_chat/application/agent_settings_bloc.dart`  
**状态**: ✅ 后端逻辑完成，UI 待实现

