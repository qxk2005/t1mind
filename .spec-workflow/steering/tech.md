# 技术舵手文档：AppFlowy AI 全局模型与 OpenAI 兼容服务器支持

## 技术目标
在保持现有 LAI/Ollama 能力的基础上，引入“OpenAI 兼容服务器”作为可选全局 AI 提供方，统一打通：
- 聊天/补全推理链路（流式与非流式）
- 嵌入生成链路
- 全局模型类型选择与持久化
- UI 配置、测试、错误反馈与 i18n

## 架构与模块改造点

1) Flutter 设置 UI（新增 Tab + 下拉选择）
- 位置：`frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/`
- 变更：
  - 在 `SettingsAIView` 与移动端 `AiSettingsGroup` 增加“全局模型类型”下拉（Ollama / OpenAI 兼容）。
  - 新增 `OpenAICompatibleSetting` 组件（新文件）与 Tab：
    - 聊天配置：base/chat 端点、API Key、模型名、模型类型（推理/普通）、max tokens、temperature、timeout。
    - 嵌入配置：embed 端点、API Key、模型名。
    - “测试聊天模型”“测试嵌入模型”按钮。
  - i18n：在 `resources/translations/zh-CN.json` 与 `en-US.json` 补充新 keys。

2) Flutter/BLoC 层
- 新增：`openai_compatible_setting_bloc.dart`：
  - 事件：加载、编辑、保存、测试聊天、测试嵌入、切换模型类型。
  - 状态：表单字段、保存中/已保存、测试中、测试结果、错误。
  - 与后端交互：通过 FFI 事件发送/接收（新增 proto）。
- 调整：`settings_ai_bloc.dart` 增加“全局模型类型”字段与事件，桥接到本地/OAI 两个子配置面板。

3) Proto 与 FFI 事件
- 在 `appflowy_backend/protobuf/flowy-ai/` 增加：
  - `OpenAICompatibleSettingPB`（聊天与嵌入子结构），`GlobalAIModelTypePB`（Ollama/OpenAICompatible），`TestResultPB`。
  - 事件：
    - Get/Save 全局模型类型
    - Get/Save OpenAI 兼容设置（聊天/嵌入）
    - Test Chat / Test Embedding（返回 `TestResultPB`）
- 在 Dart 侧 `AIEvent*` 封装事件；Rust 侧在 `flowy-ai` 暴露 handler。

4) Rust 后端（flowy-ai）
- 位置：`frontend/rust-lib/flowy-ai`
- 新增：
  - `openai_compatible` 模块：
    - `client.rs`：轻量 HTTP 客户端（基于 reqwest）封装 Chat 与 Embedding（兼容 OpenAI 格式）。
    - `chat.rs`：按配置构建请求（支持流式 SSE/分块与非流式）。
    - `embeddings.rs`：调用 embeddings 端点。
    - `types.rs`：配置、请求/响应结构（与 PB 对应）。
    - `controller.rs`：对外提供测试函数与实际推理调用（在中间件里按全局模型类型分发）。
  - `persistence` 扩展：在 `KVStorePreferences` 或表结构中新增键：
    - `global_ai_model_type:v1`
    - `openai_compatible_setting:v1`（含聊天与嵌入）
- 改造：
  - `ChatServiceMiddleware::stream_answer` 根据 `AIModel` 或全局类型选择：
    - 本地：沿用 LAI/Ollama 流程
    - 远程：走 `openai_compatible::controller` 的流式接口
  - 嵌入：在创建/更新向量索引处，根据全局类型择用 `Embedder::OpenAICompatible` 或现有 `Embedder::Ollama`。

5) 错误与可观测性
- 统一错误码与用户可读提示（超时、401/403、404、模型不可用）。
- Rust 侧 tracing 打点；Dart 侧 Log 与 UI Toast。

## 数据模型
- `GlobalAIModelTypePB`：`LOCAL_OLLAMA` | `OPENAI_COMPATIBLE`
- `OpenAIChatSettingPB`：`base_url`、`api_key`、`model`、`model_kind`（`REASONING`/`STANDARD`）、`max_tokens`、`temperature`、`timeout_ms`
- `OpenAIEmbeddingSettingPB`：`base_url`、`api_key`、`model`
- `OpenAICompatibleSettingPB`：`chat`、`embedding`
- `TestResultPB`：`ok`、`message`

## 跨平台要点
- macOS/Windows：证书链错误时给出清晰提示；
- Android：确保网络权限与受信 CA，必要时支持自定义超时与重试；
- 流式：优先使用 text/event-stream 或 chunk 响应，前端沿用现有流式消费通道。

## 安全
- API Key 仅本地存储（系统密钥链可选后续增强），传输仅用于直连所配置端点。
- 日志避免输出密钥，错误上报脱敏。

## 验收与可视化
- 通过“测试聊天/嵌入”按钮实时反馈；
- 切换全局模型后，在聊天界面发起一次提问即可验证请求落点（本地/远端）。

## 变更清单（实现级任务锚点）
- Flutter：新增 Setting 面板与 BLoC、i18n
- Proto：新增消息与事件
- Rust(flowy-ai)：新增 openai_compatible 模块、持久化、路由分发
- 可观测性与错误提示


