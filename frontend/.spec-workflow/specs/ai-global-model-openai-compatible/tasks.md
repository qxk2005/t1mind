# AppFlowy OpenAI SDK集成任务规范

## 任务概述
将AppFlowy的OpenAI兼容服务器实现从自定义HTTP客户端迁移到OpenAI官方Dart SDK，分为8个主要任务，每个任务都有明确的可视化产出。

---

## Task 1: 添加OpenAI Dart SDK依赖和基础配置

**状态**: [x] 已完成

**描述**: 在项目中添加OpenAI官方Dart SDK依赖，并创建基础的配置结构。

**文件路径**:
- `appflowy_flutter/pubspec.yaml`
- `rust-lib/flowy-ai/src/entities.rs`

**需求关联**: REQ-001, REQ-002, REQ-003

**验收标准**:
- [x] 在pubspec.yaml中添加openai_dart依赖
- [x] 在entities.rs中添加OpenAI SDK相关的protobuf定义
- [x] 创建OpenAISDKSettingPB、OpenAISDKChatSettingPB、OpenAISDKEmbeddingSettingPB数据结构
- [x] 编译通过，无依赖冲突

**可视化产出**: 
- 依赖添加成功，项目可以正常编译
- 新的protobuf消息类型可以在Dart代码中使用

**_Prompt**:
```
Role: Flutter/Rust 依赖管理专家
Task: 在AppFlowy项目中添加OpenAI Dart SDK依赖并创建基础protobuf配置结构

Context: 
- 当前项目使用自定义HTTP客户端实现OpenAI兼容API
- 需要迁移到官方OpenAI Dart SDK
- 保持与现有配置结构的兼容性

Restrictions:
- 不要破坏现有的依赖关系
- 确保跨平台兼容性（macOS、Windows、Android）
- 遵循AppFlowy现有的代码规范

_Leverage:
- 参考现有的OpenAICompatibleSettingPB结构
- 使用AppFlowy现有的protobuf模式
- 查看rust-lib/flowy-ai/src/entities.rs中的现有定义

_Requirements: REQ-001, REQ-002, REQ-003

Success: 
- pubspec.yaml包含openai_dart依赖
- entities.rs包含新的OpenAI SDK protobuf定义
- 项目编译成功
- 可以在Dart代码中导入和使用OpenAI SDK

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 2: 创建全局模型类型选择器UI组件

**状态**: [x] 已完成

**描述**: 在AI设置界面创建全局模型类型选择下拉框，支持"Ollama本地"和"OpenAI兼容服务器"选择。

**文件路径**:
- `appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/global_model_type_selector.dart`
- `appflowy_flutter/lib/workspace/application/settings/ai/settings_ai_bloc.dart`

**需求关联**: REQ-001

**验收标准**:
- [x] 创建GlobalModelTypeSelector组件
- [x] 下拉框显示两个选项：Ollama本地、OpenAI兼容服务器
- [x] 选择后触发状态更新
- [x] 支持中英文显示
- [x] 集成到现有的AI设置页面

**可视化产出**:
- AI设置页面显示全局模型类型选择器
- 可以通过下拉框切换模型类型
- 选择后界面状态正确更新

**_Prompt**:
```
Role: Flutter UI开发专家
Task: 创建全局AI模型类型选择器UI组件，集成到AppFlowy的AI设置界面

Context:
- 需要在现有AI设置界面添加全局模型类型选择功能
- 支持Ollama本地和OpenAI兼容服务器两种选项
- 需要与现有的SettingsAIBloc集成

Restrictions:
- 遵循AppFlowy现有的UI设计规范
- 保持与现有设置界面的一致性
- 支持中英文国际化
- 不要破坏现有的AI设置功能

_Leverage:
- 参考现有的settings_ai_bloc.dart实现
- 使用AppFlowy的UI组件库（flowy_infra_ui）
- 参考现有的下拉框组件实现

_Requirements: REQ-001

Success:
- 全局模型类型选择器显示在AI设置页面
- 下拉框包含正确的选项
- 选择后触发BLoC状态更新
- 支持中英文界面
- 与现有UI风格一致

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 3: 实现OpenAI SDK配置界面

**状态**: [x] 已完成

**描述**: 创建OpenAI SDK的详细配置界面，包括聊天模型和嵌入模型的所有配置选项。

**文件路径**:
- `appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_sdk_setting.dart`
- `appflowy_flutter/lib/workspace/application/settings/ai/openai_sdk_bloc.dart`

**需求关联**: REQ-002, REQ-003

**验收标准**:
- [x] 创建OpenAISDKSetting组件
- [x] 聊天配置区域：API端点、API密钥、模型名称、模型类型、最大tokens、温度、超时时间
- [x] 嵌入配置区域：API端点、API密钥、模型名称
- [x] 表单验证和错误提示
- [x] 支持配置的实时编辑和保存

**可视化产出**:
- 完整的OpenAI SDK配置界面
- 所有配置项都可以正常编辑
- 表单验证工作正常
- 界面布局美观且易用

**_Prompt**:
```
Role: Flutter表单和状态管理专家
Task: 创建OpenAI SDK的详细配置界面，包含聊天和嵌入模型的所有配置选项

Context:
- 需要创建类似现有OpenAI兼容设置的界面
- 包含聊天模型和嵌入模型两个配置区域
- 需要支持表单验证和实时编辑

Restrictions:
- 遵循AppFlowy的表单设计模式
- 确保所有输入字段都有适当的验证
- 支持中英文标签和提示
- 保持与现有设置界面的一致性

_Leverage:
- 参考现有的openai_compatible_setting.dart实现
- 使用AppFlowy的SettingsInputField组件
- 参考现有的BLoC模式实现

_Requirements: REQ-002, REQ-003

Success:
- OpenAI SDK配置界面完整显示
- 所有配置字段都可以编辑
- 表单验证正确工作
- 界面响应用户操作
- 支持配置的保存和加载

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 4: 实现后端OpenAI SDK事件处理

**状态**: [x] 已完成

**描述**: 在Rust后端实现OpenAI SDK相关的事件处理器，包括配置的获取、保存和测试功能。

**文件路径**:
- `rust-lib/flowy-ai/src/event_handler.rs`
- `rust-lib/flowy-ai/src/event_map.rs`
- `rust-lib/flowy-ai/src/openai_sdk/mod.rs`

**需求关联**: REQ-002, REQ-003, REQ-004, REQ-005

**验收标准**:
- [x] 实现AIEventGetOpenAISDKSetting事件处理
- [x] 实现AIEventSaveOpenAISDKSetting事件处理
- [x] 实现AIEventTestOpenAISDKChat事件处理
- [x] 实现AIEventTestOpenAISDKEmbedding事件处理
- [x] 事件映射正确注册

**可视化产出**:
- Flutter可以成功调用后端API
- 配置可以正确保存和读取
- 测试功能返回正确的结果

**_Prompt**:
```
Role: Rust后端API开发专家
Task: 实现OpenAI SDK相关的事件处理器，支持配置管理和测试功能

Context:
- 需要在现有的AI事件处理框架中添加OpenAI SDK支持
- 包括配置的CRUD操作和连接测试功能
- 需要与Flutter前端的API调用对接

Restrictions:
- 遵循AppFlowy现有的事件处理模式
- 确保错误处理和日志记录完整
- 保持与现有AI功能的兼容性
- 使用现有的protobuf消息格式

_Leverage:
- 参考现有的AI事件处理器实现
- 使用AppFlowy的事件分发框架
- 参考现有的配置持久化模式

_Requirements: REQ-002, REQ-003, REQ-004, REQ-005

Success:
- 所有OpenAI SDK事件处理器正确实现
- Flutter可以成功调用后端API
- 配置数据可以正确持久化
- 测试功能返回有意义的结果
- 错误处理完整且用户友好

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 5: 实现OpenAI SDK聊天服务

**状态**: [x] 已完成

**描述**: 使用OpenAI Dart SDK实现聊天功能，替换现有的HTTP客户端实现。

**文件路径**:
- `rust-lib/flowy-ai/src/openai_sdk/chat_service.rs`
- `rust-lib/flowy-ai/src/openai_sdk/controller.rs`

**需求关联**: REQ-002, REQ-007

**验收标准**:
- [x] 创建OpenAISDKChatService
- [x] 实现流式聊天响应
- [x] 支持所有配置参数（模型、tokens、温度等）
- [x] 错误处理和重试机制
- [x] 与现有AI聊天接口兼容

**可视化产出**:
- AI聊天功能使用新的SDK实现
- 流式响应正常工作
- 配置参数正确应用
- 错误处理用户友好

**_Prompt**:
```
Role: Rust AI服务集成专家
Task: 使用OpenAI Dart SDK实现聊天服务，替换现有的HTTP客户端

Context:
- 需要将现有的OpenAI兼容聊天实现迁移到官方SDK
- 保持现有的流式响应和错误处理功能
- 确保与AppFlowy现有AI聊天接口的兼容性

Restrictions:
- 保持现有API接口不变
- 确保流式响应的性能
- 实现完整的错误处理和重试逻辑
- 支持所有现有的配置参数

_Leverage:
- 参考现有的openai_compatible/chat.rs实现
- 使用AppFlowy的流式响应框架
- 参考现有的错误处理模式

_Requirements: REQ-002, REQ-007

Success:
- OpenAI SDK聊天服务正确实现
- AI聊天功能使用新的实现
- 流式响应工作正常
- 所有配置参数正确应用
- 错误处理完整且用户友好

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 6: 实现OpenAI SDK嵌入服务

**状态**: [x] 已完成

**描述**: 使用OpenAI Dart SDK实现嵌入功能，支持文档索引和语义搜索。

**文件路径**:
- `rust-lib/flowy-ai/src/openai_sdk/embedding_service.rs`
- `rust-lib/flowy-ai/src/embeddings/context.rs`

**需求关联**: REQ-003, REQ-007

**验收标准**:
- [x] 创建OpenAISDKEmbeddingService
- [x] 实现批量嵌入处理
- [x] 支持嵌入缓存优化
- [x] 与现有嵌入接口兼容
- [x] 性能优化和错误处理

**可视化产出**:
- 文档嵌入功能使用新的SDK实现
- 批量处理正常工作
- 嵌入缓存提升性能
- 语义搜索功能正常

**_Prompt**:
```
Role: Rust嵌入服务开发专家
Task: 使用OpenAI Dart SDK实现嵌入服务，支持文档索引和语义搜索

Context:
- 需要将现有的OpenAI兼容嵌入实现迁移到官方SDK
- 保持现有的批量处理和缓存优化功能
- 确保与AppFlowy现有嵌入接口的兼容性

Restrictions:
- 保持现有嵌入API接口不变
- 确保批量处理的效率
- 实现适当的缓存策略
- 支持现有的嵌入配置参数

_Leverage:
- 参考现有的openai_compatible/embeddings.rs实现
- 使用AppFlowy的嵌入上下文框架
- 参考现有的缓存和优化策略

_Requirements: REQ-003, REQ-007

Success:
- OpenAI SDK嵌入服务正确实现
- 文档嵌入功能使用新的实现
- 批量处理效率良好
- 嵌入缓存正确工作
- 语义搜索功能正常

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 7: 实现模型测试功能

**状态**: [ ] 待完成

**描述**: 实现聊天模型和嵌入模型的连接测试功能，提供详细的测试结果和错误信息。

**文件路径**:
- `appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_sdk_test_panel.dart`
- `rust-lib/flowy-ai/src/openai_sdk/test_service.rs`

**需求关联**: REQ-004

**验收标准**:
- [ ] 实现聊天模型测试功能
- [ ] 实现嵌入模型测试功能
- [ ] 显示详细的测试结果（响应时间、状态码等）
- [ ] 错误信息本地化和用户友好
- [ ] 测试过程中的加载状态显示

**可视化产出**:
- 测试按钮可以正常工作
- 测试结果详细且易懂
- 错误信息帮助用户诊断问题
- 加载状态提供良好的用户体验

**_Prompt**:
```
Role: 测试功能和用户体验专家
Task: 实现OpenAI SDK模型的连接测试功能，提供详细的测试结果和用户友好的错误信息

Context:
- 需要为聊天和嵌入模型提供连接测试功能
- 测试结果需要包含响应时间、状态码等详细信息
- 错误信息需要本地化且帮助用户诊断问题

Restrictions:
- 确保测试不会影响正常使用
- 提供清晰的加载状态指示
- 错误信息需要可操作的建议
- 遵循AppFlowy的UI设计规范

_Leverage:
- 参考现有的测试功能实现
- 使用AppFlowy的错误处理框架
- 参考现有的加载状态组件

_Requirements: REQ-004

Success:
- 聊天和嵌入模型测试功能正常工作
- 测试结果显示详细且准确
- 错误信息本地化且用户友好
- 加载状态提供良好的用户反馈
- 测试功能帮助用户验证配置

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## Task 8: 集成多平台设置界面和配置迁移

**状态**: [x] 已完成

**描述**: 将OpenAI SDK配置集成到所有平台的设置界面，并实现从现有配置的平滑迁移。

**文件路径**:
- `appflowy_flutter/lib/mobile/presentation/setting/ai/ai_settings_group.dart`
- `appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/setting_ai_view.dart`
- `rust-lib/flowy-ai/src/migration/config_migrator.rs`

**需求关联**: REQ-005, REQ-006

**验收标准**:
- [x] 桌面端设置界面集成OpenAI SDK配置
- [x] 移动端设置界面集成OpenAI SDK配置
- [x] 服务器端设置界面集成OpenAI SDK配置
- [x] 实现配置数据迁移逻辑
- [x] 向后兼容性验证

**可视化产出**:
- 所有平台的设置界面都支持OpenAI SDK配置
- 现有用户的配置可以无缝迁移
- 界面在不同屏幕尺寸下都正常显示
- 配置同步在多平台间正常工作

**_Prompt**:
```
Role: 跨平台集成和数据迁移专家
Task: 将OpenAI SDK配置集成到所有平台设置界面，并实现配置数据的平滑迁移

Context:
- 需要在桌面端、移动端、服务器端都支持OpenAI SDK配置
- 现有用户的OpenAI兼容配置需要迁移到新的SDK配置
- 确保跨平台的配置同步正常工作

Restrictions:
- 保持现有配置数据的完整性
- 确保迁移过程不会丢失用户数据
- 适配不同平台的界面布局
- 保持向后兼容性

_Leverage:
- 参考现有的多平台设置界面实现
- 使用AppFlowy的配置同步机制
- 参考现有的数据迁移模式

_Requirements: REQ-005, REQ-006

Success:
- 所有平台都支持OpenAI SDK配置
- 配置迁移逻辑正确且安全
- 界面在不同平台都正常显示
- 配置同步在多平台间正常工作
- 现有用户体验无缝升级

Instructions: 首先将此任务标记为进行中，完成后标记为已完成。
```

---

## 任务依赖关系

```
Task 1 (依赖和配置) 
    ↓
Task 2 (全局选择器) ← Task 3 (配置界面)
    ↓                    ↓
Task 4 (后端事件处理)
    ↓
Task 5 (聊天服务) ← Task 6 (嵌入服务)
    ↓                ↓
Task 7 (测试功能)
    ↓
Task 8 (多平台集成)
```

## 完成标准

所有任务完成后，应该实现：
1. ✅ 完整的OpenAI SDK集成
2. ✅ 用户友好的配置界面
3. ✅ 可靠的测试功能
4. ✅ 跨平台兼容性
5. ✅ 平滑的配置迁移
6. ✅ 良好的用户体验

每个任务完成后都应该有可视化的验证方式，确保功能正常工作且用户可以直观地看到改进效果。
