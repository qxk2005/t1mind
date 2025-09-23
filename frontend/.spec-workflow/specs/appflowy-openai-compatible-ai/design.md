## AppFlowy OpenAI 兼容服务器与全局模型选型 — 设计说明书

Spec 名称: appflowy-openai-compatible-ai

### 架构总览
- 入口：沿用现有 `SettingsAIView` / `LocalSettingsAIView` 页面框架，新增“全局使用的模型类型”切换与“OpenAI 兼容服务器”配置 Tab。
- UI 层：Flutter（Desktop/Mobile）
  - 复用现有 `SettingsAIBloc`、`LocalAiPluginBloc` 模式；新增 `OpenAICompatSettingBloc` 与视图。
  - i18n：新增 zh-CN/en-US 键值。
- 应用层（Flutter ↔ Rust）：沿用现有 `AIEvent*` 事件桥，新增：
  - 获取/更新 OpenAI 兼容聊天与嵌入配置事件
  - 测试聊天与测试嵌入事件（返回结构化结果：success/err, latency, summary）
- 核心层（Rust）：
  - 偏好存储：继续使用 `KVStorePreferences`（工作空间/设备作用域）
  - 模型选择：在 `AIManager` 的 `is_local_model` 判定链中引入“OpenAI 兼容服务器”作为远端来源（非 local）。
  - 调度：`ChatServiceMiddleware.stream_answer` 已以 `ai_model.is_local` 分流；选择 OpenAI 兼容时走 `cloud_service.stream_answer` 路径。
  - 云服务实现：在 `flowy-core` 的 `ServerProvider` 链路中扩展 Chat 服务以支持 OpenAI 兼容 API 调用（Chat/Embeddings）。

### UI/交互设计
1) 全局使用的模型类型
   - 在 `SettingsAIView` 顶部新增下拉：`Ollama 本地` / `OpenAI 兼容服务器`。
   - 切换后，显示对应 Tab：
     - `Ollama`（沿用 `LocalAISetting` + `OllamaSettingPage`）
     - `OpenAI 兼容服务器`（新增 `OpenAICompatSettingPage`）

2) OpenAI 兼容服务器设置页（两组表单）
   - 聊天设置：Base URL、API Key、Model、Model Type（推理/普通）、Max Tokens、Temperature、Timeout(s)、[测试按钮]
   - 嵌入设置：Base URL、API Key、Model、[测试按钮]
   - 表单校验：必填校验、URL 规范、数值范围；保存按钮仅在变更后可用。
   - 测试（增强版可视化）：
     - 结果卡片包含：成功/失败状态、耗时(ms)、模型信息/向量维度（嵌入）、请求 ID（若有）。
     - 展开“详细信息”区域，显示已脱敏的请求/响应明细（便于排障）：
       - 请求：方法、URL、查询参数/路径、请求体摘要（截断）、请求头（Authorization 仅显示前后各 3 位，中间以***遮蔽）。
       - 响应：HTTP 状态码、错误码/错误类型、响应体摘要（截断）、关键响应头（如 `x-request-id`）。
       - 流式（SSE）测试：展示事件序列与时间线（首包时延、最后一包时间），截断长内容并支持复制。
     - 提供“复制调试信息”按钮（自动去除密钥），用于提交 issue 或内部排障。
     - 超时或网络错误给出明确分类（超时/断网/证书/跨域），并提供“重试”和“打开设置”快捷入口。

3) 三类设置界面
   - 桌面端“服务器端设置”：复用 `SettingsAIView`，含类型选择与两个 Tab。
   - 移动端：在 `AiSettingsGroup` 入口显示当前选型与模型名，进入详情页编辑（新增移动端详情页）。
   - 工作空间设置：与服务器端一致；优先级遵循“工作空间 > 设备 > 默认”。

### 数据模型与存储
- Flutter 侧临时模型
  - `OpenAICompatChatSetting { baseUrl, apiKey, model, modelType, maxTokens?, temperature?, timeoutSec? }`
  - `OpenAICompatEmbedSetting { baseUrl, apiKey, model }`
- Rust 偏好键（KVStorePreferences）
  - `ai.global.modelProvider` → `"ollama" | "openai_compat"`
  - `ai.openai_compat.chat` → JSON 同上（密钥不落明文日志）
  - `ai.openai_compat.embed` → JSON 同上
  - 作用域：workspace 优先；否则 device-local；未配置走默认（ollama）

### 全局 Provider 生效范围（关键保证）
- 全局 Provider（`ai.global.modelProvider`）为单一真实来源：任何 AI 能力调用（聊天、补全、相关问题、RAG 检索触发的嵌入/召回、文件嵌入、工具调用等）均通过统一选择器读取。
- Flutter 层：不直接分流 Provider，仅通过既有事件（如 `AIEventStreamMessage`、`AIEventCompleteText` 等）调用；Provider 解析在 Rust 层集中处理，确保一致性。
- Rust 层：
  - `AIUserService::is_local_model()` 基于 KV/workspace 设置返回是否本地；
  - `AIManager` 内部所有入口（含 `get_available_models`、`stream_answer`、`stream_complete`、`get_related_message`、嵌入相关函数）统一以 Provider 决策：
    - `is_local` → `LocalAIController`
    - 非 `is_local` → `ServerProvider.chat_service()`，其中对 OpenAI 兼容接口做适配。
- 当 Provider 切换时：
  - 通过 `AIModelSwitchListener`/事件总线触发模型列表与选中模型刷新；
  - 已打开的聊天会在下一次发送前采用最新 Provider，必要时提示用户刷新或重开会话。

### 事件与接口（Flutter ↔ Rust）
- 读取/保存：
  - `AIEventGetAIProvider()` → { provider: "ollama" | "openai_compat" }
  - `AIEventUpdateAIProvider(provider)`
  - `AIEventGetOpenAICompatChatSetting()` / `AIEventUpdateOpenAICompatChatSetting(OpenAICompatChatSettingPB)`
  - `AIEventGetOpenAICompatEmbedSetting()` / `AIEventUpdateOpenAICompatEmbedSetting(OpenAICompatEmbedSettingPB)`
- 测试：
  - `AIEventTestOpenAICompatChat(TestPromptPB)` → `TestResultPB { ok, latencyMs, error?, modelInfo? }`
  - `AIEventTestOpenAICompatEmbed(TestTextPB)` → `TestResultPB { ok, latencyMs, dim?, error? }`

全局使用声明：上述 Provider 读取接口影响所有 AI 事件处理路径（聊天/补全/相关问题/嵌入/索引），调用方无需关心 Provider 细节。

说明：若已有统一 AI 事件通道（如 `AIEvent*` 命名空间）则按现有风格实现，尽量不破坏现有 API。

### Rust 层实现要点
1) 偏好结构
   - 新增 `OpenAICompatSetting` 结构体（chat/embed），序列化存储于 KV。
   - 新增 Provider 选择项；影响 `AIUserService::is_local_model()` 的结果与模型列表来源。

2) 模型列表融合
   - `AIManager::get_available_models` 中：当 provider==openai_compat 时，将云端模型列表用 `ServerProvider` 的 `get_setting_model_selection` 提供（或由 OpenAI 兼容服务返回）。
   - 选中模型存入“全局 active model”监听通道，沿用 `AIModelSwitchListener`。

3) Chat/Embeddings 调用
   - 在 `ServerProvider.chat_service()` 处扩展实现：
     - Chat: 调用 `{baseUrl}/v1/chat/completions`，Authorization: Bearer {apiKey}
       - 超时：`timeoutSec`；温度、max_tokens、model 字段按 OpenAI 兼容约定。
       - 流式：SSE 解析 → 转换为 `StreamAnswer`/`QuestionStreamValue`。
     - Embeddings: 调用 `{baseUrl}/v1/embeddings`，返回向量维度与耗时。
   - 错误分类：认证失败（401/403）、模型不存在（404/Invalid model）、超时（408/Client timeout）、服务端错误（5xx）。
   - 其它：`stream_complete`（纯补全）、`get_related_message`（相关问题）、索引/嵌入文件均复用相同 Provider 选择与错误分类策略。

4) 安全存储
   - 密钥在桌面端尝试 Keychain/Windows Credential；不可用则加密存 KV 并标注“降级”。
   - Android 使用 Keystore；Flutter 侧通过平台通道读写。

### i18n
- 新增 zh-CN/en-US 键值：按需求文档“i18n 关键文案（示例）”落库。

### 可视化与里程碑对应
- M1：`SettingsAIView` 新增“全局使用的模型类型”与 Tab；`OpenAICompatSettingPage` 骨架（禁用保存）。
- M2：聊天表单完整 + 测试按钮可用 + 详细调试信息（脱敏）面板；保存/加载可用。
- M3：嵌入表单完整 + 测试按钮可用 + 详细调试信息（脱敏）面板；保存/加载可用。
- M4：三界面同步（桌面/移动/工作空间），作用域优先级生效。
- M5：所有 AI 功能默认走所选 Provider（聊天/补全/相关问题/嵌入/索引）；错误与回退提示完善；文案中英完成。

### 兼容与回退
- 若 `OpenAI 兼容服务器` 配置不完整或测试失败：
  - 在使用时提示并引导跳转设置；
  - 若开启 LAI，可回退到 Ollama；否则给出清晰错误。

### 风险控制
- 平台 SDK 兼容性：统一走 Rust HTTP 客户端实现，Flutter 侧仅发事件；避免多端 SDK 差异。
- SSE 解析复杂度：抽象统一解析器并加入健壮错误处理与重试（手动重试）。

### 目录与文件变更（概述）
- Flutter：
  - `appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/openai_compat_setting.dart`（新）
  - `appflowy_flutter/lib/workspace/application/settings/ai/openai_compat_setting_bloc.dart`（新）
  - `appflowy_flutter/lib/mobile/presentation/setting/ai/openai_compat_setting_page.dart`（新）
  - i18n：`resources/translations/en-US.json`、`zh-CN.json`（增量键）
- Rust：
  - `rust-lib/flowy-ai/src/openai_compat/`（新：settings.rs、client.rs、chat.rs、embed.rs）
  - KV keys 常量与读取写入逻辑（新/改）
  - `flowy-core` 中 `ServerProvider` 的 ChatCloudService 扩展调用

### 验收清单（与需求对齐）
- UI 可切换 Provider，默认 LAI 开启；两套表单、测试按钮、保存持久化。
- 三界面一致性与作用域优先级可验证。
- AI 聊天/嵌入按所选 Provider 生效；失败提示明确。
- 中英文本齐全；Windows/macOS/Android 可运行。


