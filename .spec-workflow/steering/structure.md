# 结构舵手文档：AppFlowy AI 全局模型与 OpenAI 兼容服务器支持

## 目录结构与关注点

### Flutter（Settings 与交互）
- 根：`frontend/appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/`
- 现有文件：
  - `settings_ai_view.dart`：AI 设置页容器
  - `local_ai_setting.dart`：本地 LAI/Ollama 设置
  - `model_selection.dart`：模型选择（需扩展全局模型类型）
- 新增文件（建议）：
  - `openai_compatible_setting.dart`：OpenAI 兼容服务器配置 UI（聊天/嵌入、测试按钮）
- 移动端：`frontend/appflowy_flutter/lib/mobile/presentation/setting/ai/ai_settings_group.dart`

### Flutter（BLoC 与状态）
- 现有：
  - `workspace/application/settings/ai/settings_ai_bloc.dart`：顶层 AI 设置状态（需扩展全局模型类型）
  - `workspace/application/settings/ai/ollama_setting_bloc.dart`：Ollama 配置逻辑
- 新增：
  - `workspace/application/settings/ai/openai_compatible_setting_bloc.dart`
    - 事件：加载、编辑、保存、测试聊天、测试嵌入
    - 状态：表单字段、保存/测试状态与结果

### 国际化资源
- `frontend/resources/translations/zh-CN.json`、`en-US.json`：补充 AI 设置相关 keys（下拉项、字段标签、测试反馈、错误信息）。

### 后端（Rust - flowy-ai）
- 根：`frontend/rust-lib/flowy-ai/`
- 现有：
  - `local_ai/`：Ollama 聊天与嵌入
  - `embeddings/`：嵌入抽象
  - `middleware/chat_service_mw.rs`：按 `AIModel` 路由本地/云端
- 新增建议：
  - `openai_compatible/`
    - `types.rs`：PB 对应的配置结构与请求/响应
    - `client.rs`：reqwest 客户端与流式/非流式解析
    - `chat.rs`：聊天推理请求封装
    - `embeddings.rs`：嵌入请求封装
    - `controller.rs`：测试与实际调用接口
- 配置与持久化：
  - 通过 `KVStorePreferences` 或现有表：
    - `global_ai_model_type:v1`
    - `openai_compatible_setting:v1`

### Proto 与 FFI
- 目录：`appflowy_backend/protobuf/flowy-ai/`
- 新增：
  - `openai_compatible.proto`：`OpenAICompatibleSettingPB`、`OpenAIChatSettingPB`、`OpenAIEmbeddingSettingPB`、`TestResultPB`、`GlobalAIModelTypePB`
  - 事件：Get/Save/Test（聊天/嵌入）与 Get/Save 全局模型类型
- Dart 侧：生成后的 `*.pb.dart` 与 `AIEvent*` 封装；Rust 侧实现 handler。

## 依赖与第三方
- Rust：`reqwest`（HTTP）、`tokio-stream`/SSE 解析（若使用）、`serde`、`tracing`
- Flutter：`flutter_bloc`、`easy_localization`

## 约定与代码风格
- 遵循现有命名与目录结构；新增模块与 BLoC 采用与现有一致的 Freezed/Equatable 模式。
- 错误处理统一走 `FlowyError` 映射，用户文案经 i18n。

## 可视化检查点（与 SPEC 任务映射）
- UI 渲染：Tab/表单/按钮可见
- 持久化：保存后重启仍在
- 测试：按钮返回成功/失败
- 全局生效：聊天/补全走所选提供方


