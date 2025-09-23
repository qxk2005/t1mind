## appflowy-openai-compatible-ai — 任务清单

说明：每个任务均需有可视化产出或可验证的接口行为；支持 Windows/macOS/Android；中英双语。完成一个任务后请在本文件中将其状态从 [ ] 改为 [x]。

### M1：Provider 切换与 Tab 骨架
- [ ] 在 `SettingsAIView` 增加“全局使用的模型类型”下拉（Ollama 本地 / OpenAI 兼容服务器）
- [ ] 新增 `OpenAICompatSettingPage` 选项卡骨架（禁用保存），能够在两种 Provider 间切换可见

_Prompt_
Implement the task for spec appflowy-openai-compatible-ai, first run spec-workflow-guide to get the workflow guide then implement the task:
Role: Flutter UI engineer
Task: Add provider dropdown and OpenAI tab skeleton in `SettingsAIView`. Ensure persistent selection in KV via Rust bridge. Update i18n keys zh-CN/en-US.
Restrictions: Do not break existing LAI settings. Keep styling consistent.
_Leverage: `settings_ai_view.dart`, `local_ai_setting.dart`, i18n jsons, bloc patterns.
_Requirements: Requirements.M1, Design.UI/交互
Success: Dropdown switches tabs; selection persisted and reloaded.

### M2：OpenAI 兼容聊天表单与测试（脱敏调试信息）
- [ ] `OpenAICompatSettingPage` 中实现聊天表单字段与校验
- [ ] “测试聊天模型”按钮：成功/失败/耗时、展开查看脱敏请求/响应、SSE 时间线、复制调试信息
- [ ] 保存/加载配置（workspace 优先，device 次之）

_Prompt_
Implement the task for spec appflowy-openai-compatible-ai, first run spec-workflow-guide to get the workflow guide then implement the task:
Role: Full-stack (Flutter + Rust) engineer
Task: Implement chat settings form and test action using AI events. Add PBs and KV keys. Rust side hits {baseUrl}/v1/chat/completions with timeout/temperature/max_tokens.
Restrictions: Mask API key in any UI/logs. Respect timeouts. SSE parsing robust.
_Leverage: `openai_compat_setting_bloc.dart`(new), `ServerProvider` chat service, `KVStorePreferences`.
_Requirements: Requirements.M2, Design.测试（增强版）/事件与接口
Success: Test shows structured results; save/reload works across app restarts.

### M3：OpenAI 兼容嵌入表单与测试（脱敏调试信息）
- [ ] 嵌入表单与校验；独立测试按钮，展示维度、耗时与错误详情（脱敏）
- [ ] 保存/加载嵌入配置

_Prompt_
Implement the task for spec appflowy-openai-compatible-ai, first run spec-workflow-guide to get the workflow guide then implement the task:
Role: Full-stack (Flutter + Rust) engineer
Task: Implement embeddings settings and test action calling {baseUrl}/v1/embeddings, display vector dim and latency; add PBs and KV keys.
Restrictions: No key leakage; robust error categories; copy debug info.
_Leverage: New Rust openai_compat::embed, Flutter bloc page.
_Requirements: Requirements.M3, Design.事件与接口
Success: Test produces expected dim; save persists; i18n done.

### M4：三界面支持与作用域优先级
- [ ] 桌面端服务器设置与工作空间设置均具备 M2/M3 功能
- [ ] 移动端新增 `openai_compat_setting_page.dart` 并接入 `AiSettingsGroup`
- [ ] 作用域：workspace > device > default；校验覆盖行为

_Prompt_
Implement the task for spec appflowy-openai-compatible-ai, first run spec-workflow-guide to get the workflow guide then implement the task:
Role: Cross-platform engineer
Task: Wire the same settings pages on desktop and Android; ensure scope precedence when reading/writing KV.
Restrictions: Keep UI consistent; avoid blocking main thread.
_Leverage: Existing `SettingsAIBloc`, mobile setting group, KV utilities.
_Requirements: Requirements.M4
Success: Values persist and override correctly per scope on all platforms.

### M5：全局 Provider 路由与默认生效、错误与回退
- [ ] 在 Rust 层统一根据 `ai.global.modelProvider` 路由聊天、补全、相关问题、嵌入/索引
- [ ] Model 列表融合与 `AIModelSwitchListener` 同步更新
- [ ] 所有 AI 使用处默认走当前 Provider；LAI 可作为回退并提示
- [ ] 错误分类与 UI 提示完善；中英双语

_Prompt_
Implement the task for spec appflowy-openai-compatible-ai, first run spec-workflow-guide to get the workflow guide then implement the task:
Role: Rust core engineer
Task: Implement provider routing in AIManager/ServerProvider, ensure all AI entrypoints honor global provider; add errors mapping.
Restrictions: Backwards compatible; do not break existing LAI.
_Leverage: `ai_manager.rs`, `chat_service_mw.rs`, `cloud_service_impl.rs`.
_Requirements: Requirements.M5, Design.全局 Provider 生效范围
Success: Chats/embeddings use selected provider; robust errors; bilingual strings.


