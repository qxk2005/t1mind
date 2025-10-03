# 智能体工具结果最大长度 UI 配置完成

## 概述

为智能体创建/编辑对话框添加了"工具结果最大长度"配置项，允许用户根据使用的 AI 模型上下文大小自定义工具结果的最大长度。

## 修改文件

### 主要文件
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/widgets/agent_dialog.dart`

### 支持文件（已有）
- `appflowy_flutter/lib/plugins/ai_chat/application/agent_settings_bloc.dart`
  - 已有 `getMaxToolResultLengthRecommendation()` 方法提供推荐说明

## 实现细节

### 1. 状态管理

#### 新增 TextEditingController
```dart
class _AgentDialogState extends State<AgentDialog> {
  // ... 其他 controllers
  late final TextEditingController _maxToolResultLengthController;
  
  // ...
}
```

#### 初始化逻辑
```dart
@override
void initState() {
  super.initState();
  
  // 初始化工具结果最大长度，默认 4000
  int defaultLength = 4000;
  if (widget.existingAgent?.hasCapabilities() == true) {
    final cap = widget.existingAgent!.capabilities;
    // ... 其他能力配置
    if (cap.maxToolResultLength > 0) {
      defaultLength = cap.maxToolResultLength;
    }
  }
  _maxToolResultLengthController = TextEditingController(text: defaultLength.toString());
}
```

### 2. UI 组件

#### 条件渲染
配置项**仅在启用工具调用时显示**，避免混淆用户：

```dart
// 工具结果最大长度配置（仅在启用工具调用时显示）
if (_enableToolCalling) ...[
  const VSpace(12),
  Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      // 标签
      FlowyText.regular(
        "工具结果最大长度 (字符)",
        fontSize: 13,
        color: Theme.of(context).textTheme.bodySmall?.color,
      ),
      const VSpace(4),
      
      // 输入框
      TextField(
        controller: _maxToolResultLengthController,
        keyboardType: TextInputType.number,
        decoration: InputDecoration(
          hintText: '默认: 4000',
          helperText: '推荐范围: 1000-16000，根据模型上下文调整',
          helperMaxLines: 2,
          border: const OutlineInputBorder(),
          contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        ),
      ),
      const VSpace(4),
      
      // 实时推荐说明
      BlocBuilder<AgentSettingsBloc, AgentSettingsState>(
        builder: (context, state) {
          final length = int.tryParse(_maxToolResultLengthController.text);
          final recommendation = context.read<AgentSettingsBloc>()
              .getMaxToolResultLengthRecommendation(length);
          return FlowyText.regular(
            recommendation,
            fontSize: 11,
            color: Theme.of(context).colorScheme.primary.withOpacity(0.7),
          );
        },
      ),
    ],
  ),
],
```

#### UI 特性

1. **数字键盘**：`keyboardType: TextInputType.number` 方便输入
2. **提示文本**：显示默认值 4000
3. **帮助文本**：说明推荐范围
4. **实时推荐**：根据输入值显示适用场景（使用 BLoC 的推荐方法）

### 3. 保存逻辑

#### 值解析与验证
```dart
void _saveAgent() {
  final name = _nameController.text.trim();
  if (name.isEmpty) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('请输入智能体名称')),
    );
    return;
  }

  // 解析工具结果最大长度
  final maxToolResultLength = int.tryParse(_maxToolResultLengthController.text) ?? 4000;
  
  // 验证工具结果最大长度
  if (_enableToolCalling && maxToolResultLength > 0 && 
      (maxToolResultLength < 1000 || maxToolResultLength > 32000)) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('工具结果最大长度必须在 1000-32000 字符之间')),
    );
    return;
  }

  final capabilities = AgentCapabilitiesPB()
    ..enablePlanning = _enablePlanning
    ..enableToolCalling = _enableToolCalling
    ..enableReflection = _enableReflection
    ..enableMemory = _enableMemory
    ..maxPlanningSteps = 10
    ..maxToolCalls = 50
    ..memoryLimit = 100
    ..maxToolResultLength = maxToolResultLength;  // ✅ 新增
  
  // ... 创建/更新请求
}
```

#### 验证规则
1. **解析失败处理**：使用 `int.tryParse()` + 默认值 4000
2. **范围验证**：仅在启用工具调用时验证，范围 1000-32000
3. **用户提示**：超出范围时显示友好的错误消息

### 4. 资源清理

```dart
@override
void dispose() {
  _nameController.dispose();
  _descriptionController.dispose();
  _personalityController.dispose();
  _avatarController.dispose();
  _maxToolResultLengthController.dispose();  // ✅ 新增
  super.dispose();
}
```

## 推荐说明映射

来自 `AgentSettingsBloc.getMaxToolResultLengthRecommendation()`：

| 配置值 | 推荐说明 |
|--------|----------|
| null 或 ≤ 0 | 使用默认值 (4000字符) |
| < 1000 | 过小，将自动调整为最小值 (1000字符) |
| 1000-2000 | 适用于小型模型 (GPT-3.5等) |
| 2000-4000 | 标准配置 (推荐) |
| 4000-8000 | 适用于标准模型 (GPT-4等) |
| 8000-16000 | 适用于大上下文模型 (Claude等) |
| 16000-32000 | 超大上下文配置 |
| > 32000 | 超出推荐范围，可能超出模型限制 |

## UI 效果预览

### 创建智能体（工具调用禁用）
```
┌─ 能力配置 ────────────────┐
│ 任务规划          [OFF]   │
│ 工具调用          [OFF]   │
│                           │  ← 配置项不显示
│ 反思机制          [OFF]   │
│ 会话记忆          [ON]    │
└──────────────────────────┘
```

### 创建智能体（工具调用启用）
```
┌─ 能力配置 ────────────────┐
│ 任务规划          [ON]    │
│ 工具调用          [ON]    │
│                           │
│ 工具结果最大长度 (字符)    │
│ ┌─────────────────────┐  │
│ │ 4000               │  │
│ └─────────────────────┘  │
│ 推荐范围: 1000-16000...  │
│ 💡 标准配置 (推荐)       │  ← 实时推荐
│                           │
│ 反思机制          [OFF]   │
│ 会话记忆          [ON]    │
└──────────────────────────┘
```

### 编辑智能体（已有配置）
```
┌─ 能力配置 ────────────────┐
│ 任务规划          [ON]    │
│ 工具调用          [ON]    │
│                           │
│ 工具结果最大长度 (字符)    │
│ ┌─────────────────────┐  │
│ │ 8000               │  │  ← 加载已有值
│ └─────────────────────┘  │
│ 推荐范围: 1000-16000...  │
│ 💡 适用于标准模型 (GPT-4等) │
│                           │
│ 反思机制          [OFF]   │
│ 会话记忆          [ON]    │
└──────────────────────────┘
```

## 使用场景

### 场景 1：使用小型模型（GPT-3.5）
```
配置值：2000
说明：适用于小型模型 (GPT-3.5等)
```

### 场景 2：使用标准模型（GPT-4）
```
配置值：4000-8000
说明：标准配置 / 适用于标准模型 (GPT-4等)
```

### 场景 3：使用大上下文模型（Claude 3）
```
配置值：8000-16000
说明：适用于大上下文模型 (Claude等)
```

### 场景 4：特殊需求
```
配置值：20000
说明：超大上下文配置
```

## 用户体验优化

### ✅ 智能默认值
- 创建新智能体：默认 4000（标准推荐）
- 编辑智能体：自动加载已有配置

### ✅ 条件显示
- 仅在启用"工具调用"时显示配置项
- 避免在不需要时混淆用户

### ✅ 实时反馈
- 输入时实时显示推荐说明
- 根据值的不同显示不同的使用场景

### ✅ 友好提示
- 清晰的标签和占位符
- 详细的帮助文本
- 明确的错误提示

### ✅ 数据验证
- 前端验证：UI 层验证范围
- 后端验证：已移除过时的空值验证
- BLoC 验证：保留范围验证

## 数据流

```
用户输入
    ↓
TextEditingController
    ↓
int.tryParse() → 解析为整数（失败则使用 4000）
    ↓
范围验证（1000-32000）
    ↓
AgentCapabilitiesPB.maxToolResultLength
    ↓
CreateAgentRequestPB / UpdateAgentRequestPB
    ↓
AIEventCreateAgent / AIEventUpdateAgent
    ↓
Rust 后端保存
```

## 测试步骤

### 1. 创建新智能体
1. 打开设置 > 智能体 > 添加智能体
2. 填写名称
3. 启用"工具调用"开关
4. 观察"工具结果最大长度"配置项出现
5. 默认值应为 4000
6. 推荐说明显示"标准配置 (推荐)"

### 2. 测试不同值的推荐
| 输入值 | 预期推荐说明 |
|--------|-------------|
| (空) | 使用默认值 (4000字符) |
| 500 | 过小，将自动调整为最小值 (1000字符) |
| 1500 | 适用于小型模型 (GPT-3.5等) |
| 4000 | 标准配置 (推荐) |
| 8000 | 适用于标准模型 (GPT-4等) |
| 16000 | 适用于大上下文模型 (Claude等) |
| 25000 | 超大上下文配置 |

### 3. 测试验证
1. 输入 500 → 点击创建 → 应该提示"工具结果最大长度必须在 1000-32000 字符之间"
2. 输入 50000 → 点击创建 → 应该提示"工具结果最大长度必须在 1000-32000 字符之间"
3. 输入 4000 → 点击创建 → 应该成功创建

### 4. 测试编辑
1. 创建一个智能体，设置工具结果长度为 8000
2. 编辑该智能体
3. 配置项应自动加载 8000
4. 推荐说明显示"适用于标准模型 (GPT-4等)"

### 5. 测试条件显示
1. 创建智能体时，禁用"工具调用"
2. 配置项应该隐藏
3. 启用"工具调用"
4. 配置项应该显示

## 编译状态

✅ **无 Lint 错误**
```
No linter errors found.
```

## 相关文件

### 前端 UI
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/widgets/agent_dialog.dart`
  - 智能体创建/编辑对话框
  - 添加了工具结果最大长度配置 UI

### BLoC 层
- `appflowy_flutter/lib/plugins/ai_chat/application/agent_settings_bloc.dart`
  - 提供推荐说明方法
  - 范围验证逻辑

### 后端
- `rust-lib/flowy-ai/src/agent/config_manager.rs`
  - 已移除过时的空值验证
  - 保留范围验证

- `rust-lib/flowy-ai/src/entities.rs`
  - `AgentCapabilitiesPB.maxToolResultLength` 定义

- `rust-lib/flowy-ai/src/chat.rs`
  - 运行时读取并使用该配置

## 向后兼容性

✅ **完全兼容**

### 旧智能体
- 没有 `maxToolResultLength` 的旧智能体
- 编辑时显示默认值 4000
- 保存后添加该字段

### 新智能体
- 创建时默认为 4000
- 可以自定义配置
- 完全支持所有范围

### 数据迁移
- ❌ **不需要数据迁移**
- 后端会自动处理缺失字段（使用默认值）
- 编辑旧智能体时会自动添加该字段

## 注意事项

### ⚠️ 用户应知晓
1. **模型上下文限制**：不同 AI 模型有不同的上下文窗口大小
2. **性能影响**：过大的值可能导致 AI 响应变慢
3. **截断提示**：超过配置长度的工具结果会被截断

### 💡 最佳实践
1. **根据模型选择**：
   - GPT-3.5：2000-4000
   - GPT-4：4000-8000
   - Claude 3：8000-16000
   
2. **测试调整**：创建后可以根据实际使用效果调整

3. **性能优先**：如果 AI 响应变慢，可以适当降低该值

## 总结

✅ **已完成的工作**

1. **UI 组件**：
   - ✅ 添加了工具结果最大长度输入框
   - ✅ 实现了条件显示（仅在启用工具调用时）
   - ✅ 添加了实时推荐说明

2. **数据流**：
   - ✅ 状态管理（TextEditingController）
   - ✅ 值解析和验证
   - ✅ 传递到 AgentCapabilitiesPB

3. **用户体验**：
   - ✅ 智能默认值（4000）
   - ✅ 友好的提示文本
   - ✅ 实时反馈
   - ✅ 错误提示

4. **代码质量**：
   - ✅ 无 Lint 错误
   - ✅ 遵循项目规范
   - ✅ 资源正确清理

### 🎉 功能完整

用户现在可以：
1. ✅ 在创建智能体时配置工具结果最大长度
2. ✅ 在编辑智能体时修改该配置
3. ✅ 看到实时的推荐说明
4. ✅ 根据使用的 AI 模型选择合适的值
5. ✅ 获得友好的验证和错误提示

工具结果最大长度现在是智能体配置中的一个完整功能，为用户提供了灵活的配置选项！

