# Tasks — appflowy-mcp-agents-tools

说明：实现阶段请先将目标任务从 [ ] 改为 [-]（进行中），完成后改为 [x]（已完成）。每个任务均附带 Prompt，包含角色、限制、可用资源与成功标准。

## 任务清单

- [x] T1 设置页（桌面/工作区/移动）：新增独立“MCP 管理”配置页面（与 AI 同级）+ 基础 CRUD UI（无检查）
  - 文件：
    - 桌面/工作区导航：`appflowy_flutter/lib/workspace/application/settings/settings_dialog_bloc.dart`（在 `SettingsPage` 中新增 `mcp` 枚举）
    - 桌面/工作区路由：`appflowy_flutter/lib/workspace/presentation/settings/settings_dialog.dart`（新增 `SettingsPage.mcp` case）
    - 桌面/工作区页面：`appflowy_flutter/lib/plugins/mcp/settings/settings_mcp_view.dart`（新建，列表+新增/编辑/删除；编辑含 stdio/SSE 字段但不做检查）
    - 移动端页面：`appflowy_flutter/lib/mobile/presentation/setting/mcp/mcp_settings_page.dart`（新建，功能等价）
    - 移动端入口：`appflowy_flutter/lib/mobile/presentation/home/mobile_home_setting_page.dart`（新增入口项，跳转到 MCP 设置页面）
  - 关联需求：R-1（1.1/1.2/1.3）
  - Prompt:
    - Role: Flutter 开发工程师
    - Task: 在设置体系内新增与 AI 同级的“MCP 管理”独立页面；桌面/工作区通过 `SettingsPage` 导航，移动端提供独立页面入口；页面支持列表与基础 CRUD，编辑表单支持传输方式切换（stdio、SSE）。
    - Restrictions: 初期采用本地内存/简易持久化；i18n 完整；Android 灰化 stdio；保持现有 UI 风格与导航一致性。
    - _Leverage: `SettingsPage` 导航、`Flowy*` UI 组件、`LocaleKeys`、现有设置页面样式。
    - _Requirements: R-1。
    - Success: 桌面/工作区左侧菜单出现“MCP”；移动端设置出现入口；页面可新建/编辑/删除记录，字段切换与校验正常。

- [x] T2 一键检查：检查弹窗与 I/O 日志（模拟数据打通）
  - 文件：
    - `appflowy_flutter/lib/plugins/mcp/settings/mcp_edit_dialog.dart`（加“检查”触发）
    - `appflowy_flutter/lib/plugins/mcp/settings/mcp_check_dialog.dart`（新建弹窗）
    - `rust-lib/flowy-ai/src/mcp/manager.rs`（提供检查桩：握手/工具/示例 schema）
    - `appflowy_flutter/lib/ai/service/mcp_ffi.dart`（新增 FFI 封装桩）
  - 关联需求：R-1.4/1.5、R-4
  - Prompt:
    - Role: Flutter + Rust FFI 工程师
    - Task: 点击“检查”后展示请求/响应原文与解析结果（工具清单、schema→KV 模板/默认值/必填标注），并把状态写回端点列表。
    - Restrictions: 先用模拟数据返回结构，20s 超时；原始 I/O 展示开关；持久化检查摘要。
    - _Leverage: `MCPClientManager`、对话框与 Bloc 模式。
    - _Requirements: R-1.4/1.5、R-4。
    - Success: 弹窗显示完整 I/O 与解析结果；列表状态与工具数量更新。

- [x] T3 助手智能体设置（桌面/工作区/移动）：新增独立“助手智能体管理”配置页面（与 AI 同级）
  - 文件：
    - 桌面/工作区导航：`appflowy_flutter/lib/workspace/application/settings/settings_dialog_bloc.dart`（在 `SettingsPage` 中新增 `agents` 枚举）
    - 桌面/工作区路由：`appflowy_flutter/lib/workspace/presentation/settings/settings_dialog.dart`（新增 `SettingsPage.agents` case）
    - 桌面/工作区页面：`appflowy_flutter/lib/plugins/agent/settings/settings_agents_view.dart`（新建，Agent 列表+新增/编辑：称呼、个性描述、工具范围）
    - 移动端页面：`appflowy_flutter/lib/mobile/presentation/setting/agent/agent_settings_page.dart`（新建，功能等价）
    - 移动端入口：`appflowy_flutter/lib/mobile/presentation/home/mobile_home_setting_page.dart`（新增入口项，跳转到 Agent 设置页面）
  - 关联需求：R-2.2
  - Prompt:
    - Role: Flutter 开发工程师
    - Task: 提供与 AI 同级的“助手智能体管理”独立页面；桌面/工作区通过 `SettingsPage` 导航；移动端提供页面入口；支持创建/编辑 Agent 并选择可用的 MCP 工具范围（基于检查缓存）。
    - Restrictions: 初期简单持久化；i18n 完整；Android 不展示 stdio 提示。
    - _Leverage: 现有设置页面样式与校验；`plugins/agent` 目录结构。
    - _Requirements: R-2.2。
    - Success: 桌面/工作区左侧菜单出现“助手智能体”；移动端设置出现入口；可新增/编辑 Agent 并保存工具范围。

- [x] T4 聊天输入：新增“MCP 选择器”多选控件
  - 文件：
    - `appflowy_flutter/lib/plugins/ai_chat/presentation/chat_page/chat_footer.dart`（挂载入口）
    - `appflowy_flutter/lib/plugins/mcp/chat/mcp_selector.dart`（新建）
  - 关联需求：R-3.1
  - Prompt:
    - Role: Flutter 开发工程师
    - Task: 在输入区域添加 MCP 多选器（仅显示状态为可用的端点），勾选结果随发送事件传入会话。
    - Restrictions: 不影响现有快捷键与输入布局；i18n；桌面/移动均可用。
    - _Leverage: `ChatBloc` 事件、`AIChatUILayout`。
    - _Requirements: R-3.1。
    - Success: 可多选 MCP，发送消息后携带所选端点列表。

- [x] T5 Agent 执行最小链路（模拟工具）
  - 文件：
    - `appflowy_flutter/lib/ai/agent/agent_service.dart`（新建）
    - `appflowy_flutter/lib/ai/agent/planner.dart`、`executor.dart`（新建）
    - 集成 `ChatBloc`：由 Agent 接管回答链路，返回流式文本与日志概要
  - 关联需求：R-2.2、R-3.2
  - Prompt:
    - Role: Dart 平台工程师
    - Task: 读取 `selectedMcpIds[]` 与 Agent；Planner 产出一步子任务；Executor 调用“模拟工具”，生成结果并以流方式写回聊天。
    - Restrictions: 接口与结构按设计对齐，后续可替换真实 FFI；错误与取消可控。
    - _Leverage: `ChatBloc`、`ai_text_message.dart`。
    - _Requirements: R-3.2。
    - Success: 聊天能看到“工具调用中…”提示，随后得到模拟结果与日志概要。

- [x] T6 FFI 与 Rust 实现：SSE + stdio（真实检查与调用）
  - 文件：
    - Rust：`rust-lib/flowy-ai/src/mcp/sse.rs`（新建）、扩展 `manager.rs`；
    - Dart：`appflowy_flutter/lib/ai/service/mcp_ffi.dart`（真实绑定）
  - 关联需求：R-1.4、R-3.2、平台策略
  - Prompt:
    - Role: Rust + FFI 工程师
    - Task: 打通真实 `mcp_check/mcp_tools/mcp_invoke`，支持超时/取消与结构化日志；Android 仅 SSE。
    - Restrictions: 敏感字段脱敏；错误码覆盖常见异常；并发上限 3。
    - _Leverage: 现有 `MCPClientManager`、tokio、reqwest。
    - _Requirements: R-1.4/R-3.2。
    - Success: 实机检查可列出工具；可调用至少一个示例工具并返回数据。

- [ ] T7 执行日志浏览与导出
  - 文件：
    - `appflowy_flutter/lib/plugins/ai_chat/presentation/logs/agent_run_log_view.dart`（新建）
  - 关联需求：R-2.4、M5
  - Prompt:
    - Role: Flutter 开发工程师
    - Task: 在会话内按消息展示规划文本、工具调用记录与 I/O 摘要；支持导出 JSON；默认脱敏。
    - Restrictions: 原始 I/O 展示须显式开启；注意滚动性能。
    - _Leverage: 现有消息/高度管理器组件。
    - _Requirements: R-2.4。
    - Success: 可视化查看与导出日志。

---

执行须知：
- 实施任务前，请先“运行 spec-workflow-guide”，然后将任务标记为 [-]；完成后标记为 [x]；
- 严格遵循 AppFlowy 代码风格与跨平台策略；
- 新增文案覆盖中文与英文，更新 `resources/translations` 并运行生成脚本；
- 每个任务结束需具备可视化产出，便于对比预期。
