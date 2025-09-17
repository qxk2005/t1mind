# Tasks — ai-global-model-openai-compatible

- [ ] A1: 增加“全局使用的模型类型”下拉与 Tab 框架（Flutter）
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart`, `frontend/appflowy_flutter/lib/mobile/presentation/setting/ai/ai_settings_group.dart`, `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/model_selection.dart`
  - Implements: Requirements §4.1, Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: Add dropdown for Global Model Type (LOCAL_OLLAMA/OPENAI_COMPATIBLE), show corresponding Tab; wire to BLoC
    - Restrictions: Do not break existing LocalAISetting; keep i18n keys separate
    - _Leverage: `SettingsAIBloc`, `LocaleKeys.settings_aiPage_keys_llmModelType`
    - _Requirements: §4.1
    - Success: Dropdown renders; selecting switches Tab; no regressions

- [ ] A2: 新增 OpenAI 兼容服务器设置 UI（聊天/嵌入）
  - Files: `frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_compatible_setting.dart` (new)
  - Implements: Requirements §4.2, §4.3; Design §2.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter UI engineer
    - Task: Build form fields (chat & embedding), add Save/Test buttons, validation
    - Restrictions: Sensitive values masked; no network on save; test button triggers backend test
    - _Leverage: easy_localization, AppFlowy UI components
    - _Requirements: §4.2, §4.3
    - Success: Forms render, save triggers event, test returns toast

- [ ] B1: BLoC 扩展与新增（globalModelType + OpenAICompatibleSettingBloc）
  - Files: `frontend/appflowy_flutter/lib/workspace/application/settings/ai/settings_ai_bloc.dart`, `frontend/appflowy_flutter/lib/workspace/application/settings/ai/openai_compatible_setting_bloc.dart` (new)
  - Implements: Requirements §4.1-§4.4; Design §2.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Flutter state engineer
    - Task: Add fields/events/states; integrate with FFI events for get/save/test
    - Restrictions: Keep Freezed/Equatable patterns; no breaking changes to existing bloc APIs
    - _Leverage: existing `OllamaSettingBloc` patterns
    - _Requirements: §4.1-§4.4
    - Success: State updates correctly; persistence roundtrip verified

- [ ] C1: Proto 与 FFI 事件新增
  - Files: `frontend/appflowy_flutter/appflowy_backend/protobuf/flowy-ai/*.proto`, Dart generated, Rust handlers
  - Implements: Requirements §4.2-§4.4; Design §3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Protobuf/FFI engineer
    - Task: Define messages/enums/events, regenerate code, implement dispatch plumbing
    - Restrictions: Backward compatible; version comments; avoid leaking api_key to logs
    - _Leverage: existing AIEvent* patterns
    - _Requirements: §3
    - Success: Dart/Rust compile; events invoked from UI hit Rust handlers

- [ ] D1: Rust 后端 openai_compatible 模块（client/chat/embeddings/controller）
  - Files: `frontend/rust-lib/flowy-ai/src/openai_compatible/types.rs`, `frontend/rust-lib/flowy-ai/src/openai_compatible/client.rs`, `frontend/rust-lib/flowy-ai/src/openai_compatible/chat.rs`, `frontend/rust-lib/flowy-ai/src/openai_compatible/embeddings.rs`, `frontend/rust-lib/flowy-ai/src/openai_compatible/controller.rs`
  - Implements: Requirements §4.2-§4.4; Design §4.1
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust engineer
    - Task: Implement HTTP client via reqwest, streaming parsing (SSE/chunk), error mapping, test_* functions
    - Restrictions: Respect timeout_ms; redact api_key in logs; unit tests for error mapping
    - _Leverage: existing `local_ai` patterns, `tracing`
    - _Requirements: §4.2-§4.4
    - Success: Test endpoints return ok/error; chat streaming works

- [ ] D2: 中间件路由与嵌入选择
  - Files: `frontend/rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`, `frontend/rust-lib/flowy-core/*` for embedder selection
  - Implements: Requirements §4.4; Design §4.2
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Rust engineer
    - Task: Route by GlobalAIModelType; choose OpenAICompatible vs Ollama for chat/embedding
    - Restrictions: Preserve existing local path; add tracing
    - _Leverage: current `is_local` logic; `KVStorePreferences`
    - _Requirements: §4.4
    - Success: Global toggle switches runtime path

- [ ] E1: 持久化与配置管理
  - Files: Rust: `frontend/rust-lib/flowy-ai/*` (KV keys); Dart: getters/setters & dispatch
  - Implements: Requirements §4.4; Design §4.3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Full-stack engineer
    - Task: Add KV keys, read/write; ensure reload after save; mask secrets in logs
    - Restrictions: Migration-safe keys with :v1 suffix
    - _Leverage: `LOCAL_AI_SETTING_KEY` references
    - _Requirements: §4.4
    - Success: Restarted app retains values; save-read roundtrip

- [ ] F1: i18n 文案补全
  - Files: `frontend/resources/translations/zh-CN.json`, `frontend/resources/translations/en-US.json`
  - Implements: Requirements §4.5; Design §2.3
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: Localization engineer
    - Task: Add keys for labels, dropdown, buttons, errors; verify rendering
    - Restrictions: Keep keys consistent with existing aiPage.* schema
    - _Leverage: `LocaleKeys.settings_aiPage_*`
    - _Requirements: §4.5
    - Success: Both languages show correct strings

- [ ] G1: 可视化验收与回归
  - Files: N/A (runtime verification)
  - Implements: Requirements §6 全部
  - _Prompt:
    Implement the task for spec ai-global-model-openai-compatible, first run spec-workflow-guide to get the workflow guide then implement the task:
    - Role: QA/Developer
    - Task: Verify A-D 里程碑；三平台验证；切换后端正确；错误提示到位
    - Restrictions: Avoid manual secrets in repo; use env/secured storage if possible
    - _Leverage: Test buttons; logs; toasts
    - _Requirements: §6
    - Success: All milestones pass, no regressions
