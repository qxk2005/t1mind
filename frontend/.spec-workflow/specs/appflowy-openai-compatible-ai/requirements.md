## AppFlowy OpenAI 兼容服务器与全局模型选型 — 需求说明书

Spec 名称: appflowy-openai-compatible-ai

### 背景与目标
在现有仅支持 Ollama 本地推理（LAI）的基础上，引入“OpenAI 兼容服务器”作为可选的全局推理来源，并提供独立的嵌入模型配置与可用性测试能力。通过统一的“AI 设置”在三类设置界面（服务器端设置、移动端设置、工作空间设置）中完成配置与持久化，确保 AppFlowy 在 macOS、Windows、Android 上一致可用，并支持中英双语。

### 术语
- OpenAI 兼容服务器: 指实现 OpenAI Chat/Embeddings API 协议（路径与字段兼容）的任意服务（如自建、第三方）。
- 模型类型: 指“推理模型（Inference）”与“普通模型（Standard）”，用于区分具备工具使用/复杂推理能力的模型与基础对话模型。
- 全局使用的模型类型: 在“AI 设置”中二选一（Ollama 本地 / OpenAI 兼容服务器），一处配置，处处生效。

### 不在本期范围（Out of Scope）
- 在线模型市场/自动拉取模型清单。
- 细粒度到每个文档/会话的模型覆盖配置（本期以“全局”生效为主）。
- iOS 首发（如需后续支持，与 Android 共用 Flutter 层方案，可复用大部分实现）。

### 受众与使用场景（EARS 风格）
- 当用户需要在离线或内网环境下使用时，系统应允许选择 Ollama 本地作为全局模型来源。
- 当用户需要调用云端/自建 OpenAI 兼容服务时，系统应允许选择 OpenAI 兼容服务器作为全局模型来源并完成连接测试。
- 当用户需要进行问答/改写/总结等对话任务时，系统应默认使用“全局使用的模型类型”的聊天配置。
- 当用户需要向量检索/相似度匹配时，系统应默认使用“嵌入模型”的独立配置。
- 当管理员/用户在不同设置界面（服务器端/移动端/工作空间）修改 AI 配置时，系统应进行持久化并在对应作用域生效。

### 功能性需求（FR）
1) 全局模型类型选择与默认
   - 在“AI 设置”中新增“全局使用的模型类型”下拉框：
     - 选项：Ollama 本地、OpenAI 兼容服务器（二选一）。
     - 默认：开启 AppFlowy LAI（Ollama 本地）为 On，且全局模型类型初始为 Ollama 本地。
   - 选择不同类型后，显示对应的详情配置选项卡（Tab）。

2) OpenAI 兼容服务器 — 聊天模型配置
   - 字段：聊天 API 端点（Base URL）、聊天 API 密钥、聊天模型名称、模型类型（推理/普通）、最大 tokens、温度、超时时间（秒）。
   - 使用“OpenAI 官方客户端”语义的请求契约进行调用；若目标平台无官方 SDK，则通过标准 HTTP 调用与 OpenAI 兼容接口，遵循官方 API 契约与错误码语义。
   - 提供“测试聊天模型”按钮：输入示例提示词（可内置），展示返回耗时、状态与响应摘要。

3) OpenAI 兼容服务器 — 嵌入模型独立配置
   - 字段：嵌入 API 端点（Base URL）、嵌入 API 密钥、嵌入模型名称。
   - 提供“测试嵌入模型”按钮：对固定示例文本生成向量（不展示向量，展示维度、耗时与是否成功）。

4) 配置持久化与作用域
   - 三类设置界面均需支持上述能力与字段：
     - 服务器端设置（Desktop/Server 桌面端视角）
     - 移动端设置（Android）
     - 工作空间设置（Workspace）
   - 按照 AppFlowy 现有配置体系实现作用域优先级（建议：工作空间 > 设备本地 > 默认）。
   - 保存时进行字段校验（必填项、端点格式、数值范围）。

5) 运行时使用规则
   - AI 聊天或其它使用问答能力的地方，默认使用“全局使用的模型类型”对应的聊天配置。
   - 内容检索/索引使用嵌入配置，若缺失则尝试回退策略并给出可见提醒。

6) 多语言与可视化产出
   - 至少支持简体中文与英文；所有新增 UI 文案提供 i18n 键值。
   - 每个里程碑任务均需有看得见的 UI 产出（新增 Tab、表单、测试按钮、提示反馈等）。

7) 安全与隐私
   - API 密钥在本地安全存储（平台可用时使用 Keychain/Keystore 等），传输仅通过 HTTPS。
   - 日志/崩溃上报不得包含明文密钥与用户数据。

### 非功能性需求（NFR）
- 跨平台：macOS、Windows、Android 一致工作，遵循 AppFlowy 代码规范。
- 稳定性：错误提示明确（网络失败、认证失败、模型不存在、超时等）。
- 性能：配置测试在 5 秒超时内给出反馈（可配置）。
- 易用性：必填项缺失时阻止保存并给出定位提示。

### 字段与校验（建议）
- 聊天 API 端点：必填，https:// 开头，允许端口；保存前探测可选。
- 聊天 API 密钥：必填，非空字符串；保存时仅做存在性校验。
- 聊天 模型名称：必填，非空字符串。
- 模型类型：必填，枚举 {inference, standard}。
- 最大 tokens：可选，整数范围 [64, 32768]，默认按模型能力建议值。
- 温度：可选，浮点范围 [0, 2]，默认 0.7。
- 超时时间：可选，整数秒，范围 [5, 60]，默认 15。
- 嵌入 API 端点：必填，https:// 开头。
- 嵌入 API 密钥：必填，非空字符串。
- 嵌入 模型名称：必填，非空字符串。

### i18n 关键文案（示例，需同步 en-US 与 zh-CN）
- settings.ai.globalModelType: 全局使用的模型类型 / Global model type
- settings.ai.option.ollama: Ollama 本地 / Ollama Local
- settings.ai.option.openaiCompat: OpenAI 兼容服务器 / OpenAI-compatible Server
- settings.ai.tab.openaiCompat: OpenAI 兼容服务器 / OpenAI-compatible
- settings.ai.chat.endpoint: 聊天 API 端点 / Chat API Endpoint
- settings.ai.chat.apiKey: 聊天 API 密钥 / Chat API Key
- settings.ai.chat.model: 聊天模型名称 / Chat Model Name
- settings.ai.chat.modelType: 模型类型 / Model Type
- settings.ai.chat.maxTokens: 最大 tokens / Max Tokens
- settings.ai.chat.temperature: 温度 / Temperature
- settings.ai.chat.timeout: 超时时间（秒） / Timeout (s)
- settings.ai.chat.test: 测试聊天模型 / Test Chat Model
- settings.ai.embed.endpoint: 嵌入 API 端点 / Embeddings API Endpoint
- settings.ai.embed.apiKey: 嵌入 API 密钥 / Embeddings API Key
- settings.ai.embed.model: 嵌入模型名称 / Embeddings Model Name
- settings.ai.embed.test: 测试嵌入模型 / Test Embeddings
- settings.ai.save: 保存配置 / Save
- settings.ai.saved: 已保存 / Saved
- settings.ai.validation.required: 该字段为必填 / This field is required
- settings.ai.validation.url: 请输入合法的 URL / Please enter a valid URL
- settings.ai.test.success: 测试成功 / Test succeeded
- settings.ai.test.failed: 测试失败 / Test failed

### 数据与存储
- 配置结构（逻辑示例）：
  - ai.global.modelProvider: "ollama" | "openai_compat"
  - ai.openaiCompat.chat: { baseUrl, apiKey, model, modelType, maxTokens?, temperature?, timeoutSec? }
  - ai.openaiCompat.embeddings: { baseUrl, apiKey, model }
  - 作用域：workspace / device-local；遵循现有优先级合并策略。
- 密钥加密存储，序列化时密钥仅在安全容器中持久化（若平台不可用则退化为本地加密文件并提示）。

### 兼容性与调用约定
- Chat: POST {baseUrl}/v1/chat/completions（或服务声明的兼容路径），Authorization: Bearer {apiKey}。
- Embeddings: POST {baseUrl}/v1/embeddings，Authorization 同上。
- 请求与响应字段遵循 OpenAI 官方定义；对常见兼容差异（如路径、模型字段名）提供最小适配层。

### 错误与空态
- 未配置或配置不完整：在使用处给出提醒与快捷入口（跳转 AI 设置）。
- 测试失败：展示错误码/信息摘要、请求 ID（若有）、耗时与重试入口。

### 里程碑与可视化产出（用于后续 Tasks 制定）
M1：新增“全局使用的模型类型”下拉框与 Tab 框架；可切换 Ollama/OpenAI 兼容服务器，界面可见。
M2：OpenAI 兼容服务器“聊天模型”表单与“测试”按钮可用，保存/加载配置正常。
M3：OpenAI 兼容服务器“嵌入模型”表单与“测试”按钮可用，保存/加载配置正常。
M4：三类设置界面（服务器端/移动端/工作空间）均具备上述功能并统一持久化策略。
M5：AI 聊天与检索实际调用对应配置；新增错误处理与回退提示；中英双语文案齐全。

### 验收标准（关键用例）
- 切换全局模型类型后，AI 聊天默认走对应聊天配置；嵌入任务默认走嵌入配置。
- 提交无效配置时，保存被阻止并高亮具体字段，错误信息本地化。
- “测试聊天模型”能返回响应摘要或明确错误；超时在可配置阈值内发生。
- “测试嵌入模型”返回维度、耗时与成功状态；失败可见错误摘要。
- 三类设置界面分别保存后，进入对应作用域再次打开能看到一致配置。
- Windows、macOS、Android 上均通过上述测试。

### 风险与缓解
- 官方 SDK 平台可用性差异：以标准 HTTP 兼容协议降级，抽象调用层，避免平台耦合。
- 第三方服务兼容细节差异：加入最小适配层与错误分类提示。
- 密钥存储限制：优先使用平台安全容器，不可用时显式提示“降级存储风险”。

### 依赖与约束
- 复用 AppFlowy 现有设置与持久化框架、i18n 与 UI 组件体系。
- 遵循 AppFlowy 代码规范，提交代码需通过现有 linter 与测试规范。


