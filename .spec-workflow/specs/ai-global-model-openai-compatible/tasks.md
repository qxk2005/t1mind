# Tasks — ai-global-model-openai-compatible

## 里程碑 A：UI 与 i18n（可视化：界面元素可见、交互无报错）

- [x] A1: 添加全局模型类型下拉选择器
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart`
  - Implements: Requirements §4.1, Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: 在 SettingsAIView 中添加"全局使用的模型类型"下拉选择器，选项包括"ollama 本地"（默认）和"openai 兼容服务器"
    - Restrictions: 不破坏现有 LocalAISetting 组件；保持现有布局结构；使用临时硬编码选项（后续任务会连接 BLoC）
    - _Leverage: 现有 AIModelSelection 组件样式；AppFlowy 下拉组件库
    - _Requirements: Requirements §4.1
    - Success: 下拉选择器可见；选项可选择；界面无异常

- [x] A2: 创建 OpenAI 兼容服务器设置面板骨架
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart` (new)
  - Implements: Requirements §4.2, §4.3; Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: 创建 OpenAI 兼容服务器设置面板，包含聊天配置区域（API端点、密钥、模型名、模型类型、tokens、温度、超时）和嵌入配置区域（API端点、密钥、模型名），以及保存/测试按钮
    - Restrictions: 仅创建 UI 骨架，使用临时状态管理；密钥字段要遮蔽显示；表单验证暂时跳过
    - _Leverage: 参考 LocalAISetting 的布局风格；使用 AppFlowy 表单组件
    - _Requirements: Requirements §4.2, §4.3
    - Success: 配置面板可见；所有字段可输入；按钮可点击（暂无功能）

- [x] A3: 添加 Tab 切换逻辑
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart`
  - Implements: Requirements §4.1; Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: 根据全局模型类型下拉选择，切换显示 LocalAISetting 或 OpenAICompatibleSetting 面板
    - Restrictions: 使用简单的条件渲染；不连接持久化状态
    - _Leverage: Flutter 条件组件渲染；现有 LocalAISetting 组件
    - _Requirements: Requirements §4.1
    - Success: 选择"ollama 本地"显示现有面板；选择"openai 兼容服务器"显示新面板

- [ ] A4: 移动端设置入口
  - Files: `frontend/appflowy_flutter/lib/mobile/presentation/setting/ai/ai_settings_group.dart`
  - Implements: Requirements §4.1; Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter mobile UI engineer
    - Task: 在移动端 AI 设置组中添加全局模型类型选择入口
    - Restrictions: 保持移动端交互模式；与桌面端功能对齐
    - _Leverage: 现有移动端设置组件模式
    - _Requirements: Requirements §4.1
    - Success: 移动端可访问全局模型类型设置

- [ ] A5: 基础 i18n 文案
  - Files: `frontend/resources/translations/zh-CN.json`, `frontend/resources/translations/en-US.json`
  - Implements: Requirements §4.5; Design §2.3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Localization engineer
    - Task: 添加全局模型类型、OpenAI 兼容服务器相关的中英文翻译键值
    - Restrictions: 遵循现有 aiPage.keys.* 命名规范；包含字段标签、按钮文案、基础错误提示
    - _Leverage: 现有 `LocaleKeys.settings_aiPage_*` 结构
    - _Requirements: Requirements §4.5
    - Success: 中英文界面文案正确显示

## 里程碑 B：持久化（可视化：字段保存成功并可在重启后读取）

- [ ] B1: 定义 Proto 消息结构
  - Files: `frontend/appflowy_flutter/appflowy_backend/protobuf/flowy-ai/entities.proto`
  - Implements: Requirements §4.2-§4.4; Design §3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Protobuf engineer
    - Task: 定义 GlobalAIModelTypePB 枚举、OpenAIChatSettingPB、OpenAIEmbeddingSettingPB、OpenAICompatibleSettingPB、TestResultPB 消息结构
    - Restrictions: 保持向后兼容；字段命名清晰；添加版本注释
    - _Leverage: 现有 AI 相关 protobuf 模式
    - _Requirements: Requirements Design §3
    - Success: Proto 文件编译成功；生成 Dart/Rust 代码

- [ ] B2: 添加 FFI 事件定义
  - Files: `frontend/appflowy_flutter/appflowy_backend/protobuf/flowy-ai/event_map.proto`
  - Implements: Requirements §4.2-§4.4; Design §3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Protobuf/FFI engineer
    - Task: 添加 GetGlobalAIModelType、SaveGlobalAIModelType、GetOpenAICompatibleSetting、SaveOpenAICompatibleSetting、TestOpenAIChat、TestOpenAIEmbedding 事件
    - Restrictions: 遵循现有事件命名规范；保持事件 ID 唯一性
    - _Leverage: 现有 AIEvent 模式
    - _Requirements: Requirements Design §3
    - Success: 事件定义完整；代码生成无错误

- [ ] B3: 实现 Rust 事件处理器骨架
  - Files: `frontend/rust-lib/flowy-ai/src/event_handler.rs`
  - Implements: Requirements §4.2-§4.4; Design §4
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust FFI engineer
    - Task: 为新定义的事件添加处理器函数骨架，实现基本的参数解析和返回
    - Restrictions: 暂时返回默认值或 mock 数据；不实现具体业务逻辑
    - _Leverage: 现有 AI 事件处理器模式
    - _Requirements: Requirements Design §4
    - Success: Dart 调用 Rust 事件不报错；返回预期数据结构

- [ ] B4: 创建 OpenAI 兼容设置 BLoC
  - Files: `frontend/appflowy_flutter/lib/workspace/application/settings/ai/openai_compatible_setting_bloc.dart` (new)
  - Implements: Requirements §4.1-§4.4; Design §2.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter state engineer
    - Task: 创建 OpenAICompatibleSettingBloc，包含加载、编辑、保存、测试事件和相应状态管理
    - Restrictions: 使用 Freezed/Equatable 模式；暂时调用 mock FFI 事件
    - _Leverage: 参考 OllamaSettingBloc 结构
    - _Requirements: Requirements §4.1-§4.4
    - Success: BLoC 状态更新正确；UI 可响应状态变化

- [ ] B5: 扩展主设置 BLoC
  - Files: `frontend/appflowy_flutter/lib/workspace/application/settings/ai/settings_ai_bloc.dart`
  - Implements: Requirements §4.1; Design §2.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter state engineer
    - Task: 在 SettingsAIBloc 中添加 globalModelType 字段和相关事件处理
    - Restrictions: 不破坏现有状态结构；保持向后兼容
    - _Leverage: 现有状态管理模式
    - _Requirements: Requirements §4.1
    - Success: 全局模型类型状态正确管理；UI 响应切换

- [ ] B6: 连接 UI 与 BLoC
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart`, `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart`
  - Implements: Requirements §4.1-§4.4; Design §2.1, §2.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: 将 UI 组件连接到对应的 BLoC，实现状态驱动的界面更新和事件响应
    - Restrictions: 确保状态变化正确反映到 UI；处理加载和错误状态
    - _Leverage: 现有 BlocBuilder/BlocListener 模式
    - _Requirements: Requirements §4.1-§4.4
    - Success: UI 与状态同步；表单操作触发正确事件

## 里程碑 C：测试能力（可视化：测试按钮返回成功/错误，错误含具体原因）

- [ ] C1: 实现 Rust 持久化层
  - Files: `frontend/rust-lib/flowy-ai/src/persistence.rs` (new or extend existing)
  - Implements: Requirements §4.4; Design §4.3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust storage engineer
    - Task: 实现 KVStorePreferences 中的配置读写，添加 global_ai_model_type:v1 和 openai_compatible_setting:v1 键
    - Restrictions: 使用版本化键名；API Key 存储安全处理；提供迁移兼容性
    - _Leverage: 现有 LOCAL_AI_SETTING_KEY 实现模式
    - _Requirements: Requirements §4.4
    - Success: 配置可保存和读取；重启应用后配置保持

- [ ] C2: 实现 OpenAI 兼容 HTTP 客户端
  - Files: `frontend/rust-lib/flowy-ai/src/openai_compatible/client.rs` (new), `frontend/rust-lib/flowy-ai/src/openai_compatible/types.rs` (new)
  - Implements: Requirements §4.2, §4.3; Design §4.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust HTTP engineer
    - Task: 基于 reqwest 实现 OpenAI 兼容的 HTTP 客户端，支持聊天和嵌入 API 调用
    - Restrictions: 支持超时配置；API Key 在日志中脱敏；处理常见 HTTP 错误
    - _Leverage: 现有 HTTP 客户端模式；reqwest 库
    - _Requirements: Requirements §4.2, §4.3
    - Success: 能够发送 HTTP 请求；错误映射清晰

- [ ] C3: 实现测试功能
  - Files: `frontend/rust-lib/flowy-ai/src/openai_compatible/controller.rs` (new)
  - Implements: Requirements §4.2, §4.3; Design §4.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust engineer
    - Task: 实现 test_chat 和 test_embedding 函数，调用 HTTP 客户端并返回 TestResultPB
    - Restrictions: 测试请求要简短；超时要明确；错误信息要用户友好
    - _Leverage: 现有错误处理模式；tracing 日志
    - _Requirements: Requirements §4.2, §4.3
    - Success: 测试函数返回明确的成功/失败结果

- [ ] C4: 完善事件处理器实现
  - Files: `frontend/rust-lib/flowy-ai/src/event_handler.rs`
  - Implements: Requirements §4.2-§4.4; Design §4
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust FFI engineer
    - Task: 完善之前创建的事件处理器，连接持久化层和测试功能
    - Restrictions: 确保线程安全；错误处理完整；日志记录适当
    - _Leverage: 现有事件处理模式
    - _Requirements: Requirements §4.2-§4.4
    - Success: 所有 FFI 事件功能完整；Dart 调用返回正确结果

- [ ] C5: 完善错误提示文案
  - Files: `frontend/resources/translations/zh-CN.json`, `frontend/resources/translations/en-US.json`
  - Implements: Requirements §4.5; Design §2.3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Localization engineer
    - Task: 添加详细的错误提示文案，包括网络错误、认证错误、模型不存在等常见错误
    - Restrictions: 错误信息要用户友好；避免技术术语；提供解决建议
    - _Leverage: 现有错误提示模式
    - _Requirements: Requirements §4.5
    - Success: 各种错误场景都有清晰的中英文提示

- [ ] C6: 实现 UI 测试反馈
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart`
  - Implements: Requirements §4.2, §4.3; Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: 实现测试按钮的加载状态、成功提示和错误提示显示
    - Restrictions: 使用 Toast 或 Dialog 显示结果；按钮要显示加载状态；错误要显示具体原因
    - _Leverage: AppFlowy 通知组件；BLoC 状态管理
    - _Requirements: Requirements §4.2, §4.3
    - Success: 测试按钮有明确的视觉反馈；用户能看到测试结果

## 里程碑 D：全局生效（可视化：切换后端正确，聊天/补全走对应配置）

- [ ] D1: 实现聊天流式支持
  - Files: `frontend/rust-lib/flowy-ai/src/openai_compatible/chat.rs` (new)
  - Implements: Requirements §4.2; Design §4.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust streaming engineer
    - Task: 实现 OpenAI 兼容的流式聊天支持，包括 SSE 和 chunk 解析
    - Restrictions: 支持流式和非流式回退；处理连接中断；与现有流式接口兼容
    - _Leverage: 现有流式处理模式；tokio-stream
    - _Requirements: Requirements §4.2
    - Success: 流式聊天功能正常；与现有体验一致

- [ ] D2: 实现嵌入功能
  - Files: `frontend/rust-lib/flowy-ai/src/openai_compatible/embeddings.rs` (new)
  - Implements: Requirements §4.3; Design §4.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust engineer
    - Task: 实现 OpenAI 兼容的嵌入生成功能
    - Restrictions: 返回格式要与现有嵌入接口兼容；支持批量处理
    - _Leverage: 现有嵌入处理模式
    - _Requirements: Requirements §4.3
    - Success: 嵌入生成功能正常；向量格式正确

- [ ] D3: 修改中间件路由逻辑
  - Files: `frontend/rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`
  - Implements: Requirements §4.4; Design §4.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust middleware engineer
    - Task: 修改 ChatServiceMiddleware，根据 GlobalAIModelType 路由到本地或 OpenAI 兼容服务
    - Restrictions: 保持现有本地路径不变；添加适当的日志记录；确保错误处理一致
    - _Leverage: 现有路由逻辑；全局配置读取
    - _Requirements: Requirements §4.4
    - Success: 全局切换能正确路由聊天请求

- [ ] D4: 修改嵌入选择逻辑
  - Files: `frontend/rust-lib/flowy-core/src/server_layer.rs` (或相关嵌入选择文件)
  - Implements: Requirements §4.4; Design §4.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust core engineer
    - Task: 修改嵌入器选择逻辑，根据全局配置选择 Ollama 或 OpenAI 兼容嵌入器
    - Restrictions: 保持现有嵌入接口不变；确保向量索引兼容性
    - _Leverage: 现有 Embedder 抽象；全局配置
    - _Requirements: Requirements §4.4
    - Success: 全局切换能正确路由嵌入请求

- [ ] D5: 端到端验收测试
  - Files: N/A (runtime verification)
  - Implements: Requirements §6 全部
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: QA engineer
    - Task: 验证所有里程碑功能：UI 显示、配置保存、测试功能、全局路由切换
    - Restrictions: 在三个平台验证；使用安全的测试配置；不在代码中硬编码真实 API Key
    - _Leverage: 应用内测试功能；日志验证；网络请求监控
    - _Requirements: Requirements §6
    - Success: 所有验收标准通过；无功能回归；三平台一致体验
