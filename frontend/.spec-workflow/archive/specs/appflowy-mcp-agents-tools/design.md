# AppFlowy MCP 集成与全局 AI 工具/智能体 — 设计说明（Design）

## 1. 设计目标与范围
- 实现全局 MCP（Model Context Protocol）管理与检查能力；
- 建立“全局 AI 工具 + 助手智能体（Agent）”框架（规划器/执行器）；
- 将聊天回答链路切换为“Agent 执行 + 工具编排”，支持从输入区选择 MCP；
- 覆盖桌面/移动/工作区三类设置页面；支持中英文；支持 macOS、Windows、Android（Android 优先 SSE）。

不在本期：远程队列编排、计费、云端统一鉴权策略。

## 2. 架构与模块
- 分层：UI（Flutter） → 应用层（Dart Service/BLoC） → 核心（Rust FFI：AI、MCP 管理与调用、持久化） → 资源（i18n、图标）。
- 新增模块：
  - MCP 管理：端点 CRUD、可用性检查（握手/工具发现/工具 schema）、日志；
  - AI 工具与智能体：统一 Tool 抽象、Agent 定义、Planner/Executor、RunLog；
  - 聊天融合：输入区 MCP 选择器；Agent 驱动回答；运行态日志展示。
- 关键现有挂点：
  - 聊天 UI：`appflowy_flutter/lib/plugins/ai_chat/presentation/chat_page/` 下 `chat_footer.dart`（输入区）、`load_chat_message_status_ready.dart`（Chat 构建）、`text_message_widget.dart`/`ai_text_message.dart`（渲染回复）。
  - 设置页：工作区对话框 `workspace/presentation/settings/settings_dialog.dart`（`SettingsPage.ai` → `SettingsAIView`/`LocalSettingsAIView`）；移动端主页设置 `mobile/presentation/home/mobile_home_setting_page.dart`（`AiSettingsGroup`）。
  - i18n：`assets/translations/*.json`，生成脚本见 `scripts/code_generation/language_files`；调用 `LocaleKeys.*.tr()`。
  - Rust AI/MCP：`rust-lib/flowy-ai/src/mcp/manager.rs`（已有 stdio 客户端管理器原型），`rust-lib/flowy-ai/src/ai_manager.rs`（模型与聊天管理）。

## 3. 数据模型与持久化
- Dart（工作区/设备范围按字段区分）：
  - `McpEndpoint`：id、name、iconKey、transport(stdio|sse)、isActive、createdAt、updatedAt、lastCheckAt、lastStatus、toolNames[]、platformNotes、ownerScope(server|mobile|workspace)。
  - `McpStdioConfig`：endpointId、command、args[]、env[]{key,value}。
  - `McpSseConfig`：endpointId、url、headers[]{key,value}。
  - `McpToolSchema`：endpointId、toolName、schema(json)、generatedKvTemplate(json)、requiredKeys[]、optionalKeys[]、defaults(json)。
  - `Agent`：id、title、personaPrompt、toolAllowlist[]/denylist[]、languagePref、createdAt、updatedAt。
  - `AgentSession`：id、agentId、createdAt、status。
  - `AgentTurn`：id、sessionId、role(user|assistant)、content、selectedMcpIds[]、createdAt。
  - `AgentRunLog`：id、sessionId、turnId、planText、steps[], toolCalls[], ioRaw(json, masked)、durationMs、error(optional)。
- Rust（`flowy_ai_pub::persistence` KV/表复用）：缓存工具清单与 schema、存储检查结果摘要与错误信息；RunLog 结构化落库（原始 I/O 可选）。

## 4. FFI 接口（拟）
- `mcp_check(config, options) -> CheckResult`：
  - 输入：`transport`、stdio（cmd/args/env）或 sse（url/headers）、`withRawIO`、`timeoutMs`；
  - 输出：可用性、工具列表、每个工具的 schema、KV 模板（默认值/必填/可选）、原始 I/O（可选）。
- `mcp_tools(endpoint_id) -> ToolsList`：读取缓存或直连刷新。
- `mcp_invoke(endpoint_id, tool_name, args_kv, ctx) -> InvokeResultStream`：按 schema 将 KV 映射为参数，支持流式输出、可取消、带结构化日志。
- `agent_plan_and_execute(session_id, turn_id, question, selected_endpoint_ids[]) -> AgentRunSummary`：Planner 产出一步子任务，Executor 调用工具并汇总答案与日志引用。

## 5. Rust 端设计
- `flowy-ai/src/mcp/manager.rs`：
  - 现有 `MCPClientManager{stdio_clients}` 扩展为 `{stdio_clients, sse_clients}`；
  - `connect_stdio(MCPServerConfig)`、`connect_sse(MCPServerConfigSse)`；
  - `initialize()`、`list_tools()`、`get_tool_schema()`、`invoke_tool()`；
  - 统一错误枚举与超时；日志 trace。
- `flowy-ai/src/mcp/sse.rs`：SSE 客户端（基于 reqwest/EventSource）；
- `flowy-ai/src/agent/*`：Planner/Executor 框架与 RunLog 生成；
- FFI 层导出上述函数，屏蔽平台差异；Android 仅启用 SSE 分支。

## 6. Dart 服务与 BLoC
- `McpRepository`：端点 CRUD、检查/缓存、读取工具与 schema、持久化默认 KV 模板与状态；
- `AgentService`：管理 Agent 定义；封装 `plan_and_execute`，推进会话、生成 RunLog、将流式结果推给聊天 UI；
- BLoC：
  - `McpListBloc`、`McpEditBloc`：列表与编辑对话框；一键检查弹窗流；
  - `AgentListBloc`、`AgentEditBloc`：Agent 列表/表单；
  - 聊天整合：在 `ChatBloc.sendMessage` 处理链中承载 `selectedMcpIds[]`，改由 `AgentService` 执行并回写文本/状态流；渲染“工具调用中…”与错误提示。

## 7. UI/UX 设计
- 设置页（桌面/工作区：`SettingsPage.ai`；移动端：`AiSettingsGroup`）：
  - 分区“工具与 MCP”：表格列（名称/传输/工具/状态/激活/时间/图标），新增/编辑/删除；编辑表单按传输方式动态展示字段；“一键检查”按钮 → 日志弹窗（显示完整请求/响应与解析结果）。
  - 分区“助手智能体”：列表 + 编辑（称呼、个性化描述、工具范围选择器）。
- 聊天页：
  - 输入区添加“MCP 选择器”（多选，仅显示状态可用端点）；
  - 回答区显示运行进度与日志概要；失败可“复制日志/重试”。

## 8. 平台与本地化
- 平台：macOS/Windows 支持 stdio+SSE；Android 默认仅 SSE，UI 灰化 stdio 配置并加提示；
- i18n：新增 `settings.mcp.*`、`agent.*`、`chat.mcpSelector.*`、`logs.*`；在 `resources/translations` 中添加中英文并运行生成脚本；原始 I/O 不做机器翻译。

## 9. 安全与隐私
- 环境变量与 HTTP 头敏感值安全存储与掩码显示；导出/日志默认脱敏；
- 原始 I/O 展示为显式开关；
- 进程/网络调用超时与并发上限；资源释放与取消。

## 10. 性能与异常处理
- 一键检查默认超时 20s；工具调用并发上限 3；
- Dart 侧取消令牌；Rust 侧 tokio 超时+背压；
- 错误分级：配置错误/网络错误/协议错误/工具执行错误，统一用户提示与可重试策略。

## 11. 失败回退策略
- 所选 MCP 全不可用：回退为纯模型回答；
- 单工具缺少必填参数：跳过并记录原因；
- SSE/stdio 失败：自动重试一次，仍失败则提示并保留日志。

## 12. 里程碑（与可视化产出）
- D1：MCP 列表与编辑基础 UI（无检查）。
- D2：一键检查 + 日志弹窗（含解析工具与 KV 模板）。
- D3：助手智能体列表与编辑。
- D4：聊天输入 MCP 选择器 + Agent 执行最小链路（可用模拟/真实工具）。
- D5：执行日志浏览与导出。

## 13. 风险与对策
- Android 对 stdio 受限 → 默认仅 SSE；
- MCP 服务实现不一 → schema 解析做容错，KV 转换保底策略；
- 日志体量增长 → 设置保留期限与大小限制，默认脱敏。

## 14. 开放问题
- 工作空间与设备级配置的合并/优先级具体规则；
- 远/近端 RAG 数据源的统一鉴权与配额（后续迭代）。

