# Design — AppFlowy: AI 全局模型与 OpenAI 兼容服务器支持

## 1. 总体架构
在保持现有 LAI/Ollama 能力稳定的前提下，引入“OpenAI 兼容服务器”作为第二条全局 AI 路径：
- 全局模型类型：`LOCAL_OLLAMA` | `OPENAI_COMPATIBLE`（持久化保存并被所有 AI 功能读取）。
- UI：设置页增加“全局使用的模型类型”下拉；选择不同类型进入对应详情配置 Tab（保留本地 LAI 面板，新增 OpenAI 兼容面板）。
- 后端：在 `flowy-ai` 新增 `openai_compatible` 模块，封装 Chat/Embedding 请求与测试；中间件按全局类型路由到本地或远端。
- i18n：新增中英文文案；跨平台一致（macOS、Windows、Android）。

## 2. 前端（Flutter）
### 2.1 UI 结构
- 位置：`frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/`
  - 修改 `settings_ai_view.dart`：在现有 `AIModelSelection` 附近增加“全局使用的模型类型”下拉；默认 `ollama 本地`；根据选择显示不同 Tab。
  - 新增 `openai_compatible_setting.dart`（新组件）：
    - 分区 1：聊天配置（聊天 API 端点、API 密钥、模型名称、模型类型（推理/普通）、最大 tokens、温度、超时时间、保存、测试聊天模型）。
    - 分区 2：嵌入配置（嵌入 API 端点、API 密钥、模型名称、保存、测试嵌入模型）。
  - 移动端：`lib/mobile/presentation/setting/ai/ai_settings_group.dart` 展示相同入口，跳转到对应设置页面。

### 2.2 状态管理（BLoC）
- 修改 `workspace/application/settings/ai/settings_ai_bloc.dart`：
  - 增加字段 `globalModelType`（默认 LOCAL_OLLAMA）。
  - 增加事件：加载/保存 `globalModelType`；切换触发 UI Tab 切换。
- 新增 `workspace/application/settings/ai/openai_compatible_setting_bloc.dart`：
  - 状态：聊天与嵌入配置表单、保存状态、测试状态、测试结果（成功/失败+消息）。
  - 事件：加载现有配置、编辑字段、保存、测试聊天、测试嵌入。
  - 与后端交互：通过 `AIEventGet/Save/Test*`（Proto/FFI）。

### 2.3 i18n
- 文件：`frontend/resources/translations/zh-CN.json`、`en-US.json`
  - 新增 keys：
    - 全局模型类型下拉项与说明文案。
    - OpenAI 兼容服务器 Tab 标题与各字段标签。
    - 保存成功/失败、测试成功/失败、错误类型提示（超时、401/403、404、模型不存在、响应不兼容等）。

## 3. 协议与 FFI（Proto）
- 目录：`appflowy_backend/protobuf/flowy-ai/`
- 新增/修改：
  - 枚举 `GlobalAIModelTypePB { LOCAL_OLLAMA = 0; OPENAI_COMPATIBLE = 1; }`
  - 消息：
    - `OpenAIChatSettingPB { string base_url; string api_key; string model; string model_kind; int32 max_tokens; double temperature; int32 timeout_ms; }`
    - `OpenAIEmbeddingSettingPB { string base_url; string api_key; string model; }`
    - `OpenAICompatibleSettingPB { OpenAIChatSettingPB chat; OpenAIEmbeddingSettingPB embedding; }`
    - `TestResultPB { bool ok; string message; }`
  - 事件：
    - Get/Save `GlobalAIModelTypePB`
    - Get/Save `OpenAICompatibleSettingPB`
    - Test Chat / Test Embedding → 返回 `TestResultPB`

## 4. 后端（Rust — flowy-ai）
### 4.1 模块与职责
- 新增目录：`frontend/rust-lib/flowy-ai/src/openai_compatible/`
  - `types.rs`：配置与请求/响应结构（与 PB 映射），含 `ModelKind`（Reasoning/Standard）。
  - `client.rs`：基于 `reqwest` 的 HTTP 客户端：
    - Chat：支持 SSE/EventStream 或 chunk 流式；无法流式则回退非流式。
    - Embeddings：非流式 JSON。
  - `chat.rs`：构造请求、处理流式增量、映射错误。
  - `embeddings.rs`：嵌入请求调用与错误映射。
  - `controller.rs`：对外统一接口：`test_chat`、`test_embedding`、`stream_answer`、`embed`。

### 4.2 中间件路由
- 文件：`frontend/rust-lib/flowy-ai/src/middleware/chat_service_mw.rs`
  - 在 `stream_answer` 等入口，根据 `GlobalAIModelTypePB` 路由：
    - `LOCAL_OLLAMA` → 走现有 LAI/Ollama 流程（保持不变）。
    - `OPENAI_COMPATIBLE` → 调用 `openai_compatible::controller.stream_answer`。
- 嵌入链路：在 `flowy-core` 相关位置（如 `server_layer.rs` 及索引写入处）选择 `Embedder::OpenAICompatible` 或 `Embedder::Ollama`。

### 4.3 持久化
- 使用 `KVStorePreferences`（或现有表）新增键：
  - `global_ai_model_type:v1`
  - `openai_compatible_setting:v1`（包含聊天与嵌入配置）

### 4.4 错误与观测
- 错误归一：超时、401/403、404、模型不存在、响应不兼容、网络错误。
- `tracing` 埋点；向 Dart 返回 `TestResultPB` 与明确 `FlowyError` 文案。

## 5. 安全与合规
- API Key 本地存储，日志脱敏；仅向用户配置端点发起请求。
- Android 网络权限与证书链提示；桌面平台证书错误信息标准化。

## 6. 性能
- 目标：
  - 流式首包 < 2s（在网络可用前提下）；
  - 请求 P95 < 5s；
- 可配置：`timeout_ms`；失败重试交由上层用户操作（测试按钮再次触发）。

## 7. 回归与兼容
- 不修改现有 Ollama 行为；默认保持 LAI 开启。
- 切换全局模型后，聊天/补全/嵌入自动使用对应配置，无需二次选择。

## 8. 交付与验收映射
- A：UI 与 i18n 可见；
- B：配置保存/读取 OK；
- C：测试聊天/嵌入按钮返回成功/错误；
- D：全局路由按选择生效，回归本地不受影响。
