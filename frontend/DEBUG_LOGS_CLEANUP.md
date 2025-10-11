# 调试日志清理总结

## 已注释的调试日志

### 1. Flutter 前端日志

#### 文件：`appflowy_flutter/lib/plugins/ai_chat/application/chat_bloc.dart`
- ✅ 已注释相关问题（Related Questions）的所有调试日志
- 位置：`_fetchRelatedQuestionsIfNeeded()` 方法
- 日志标识：`🔍 [RELATED_Q]`

```dart
// Log.debug("🔍 [RELATED_Q] Checking if should fetch related questions");
// Log.debug("🔍 [RELATED_Q] answerStream: ${_streamManager.answerStream != null}");
// Log.debug("🔍 [RELATED_Q] lastSentMessage: ${lastSentMessage != null}");
// Log.debug("🔍 [RELATED_Q] shouldFetch: $shouldFetchRelatedQuestions");
// Log.debug("🔍 [RELATED_Q] Conditions not met, skipping fetch");
// Log.debug("🔍 [RELATED_Q] Fetching related questions...");
// Log.debug("🔍 [RELATED_Q] Received ${list.items.length} related questions");
```

### 2. Rust 后端日志

#### 文件：`rust-lib/flowy-core/src/deps_resolve/chat_deps.rs`
- ✅ 已注释 `notify_did_send_message` 的info日志
- 位置：`ChatQueryServiceImpl::notify_did_send_message()` 方法

```rust
// info!(
//   "notify_did_send_message: chat_id: {}, message: {}",
//   chat_id, message
// );
```

### 3. 外部库日志（无法直接修改）

以下日志来自外部依赖库，无法直接注释：

#### client-api 库
- `🔵websocket start connecting`
- 来源：`client_api::retry` 模块
- 这是 AppFlowy Cloud 客户端库的日志

#### 工具调用日志
- 来源：`flowy_ai::agent::tool_call_handler` 模块
- 这些日志对于调试工具调用很有用，建议保留

## 如何控制外部库日志

### 方法1：环境变量控制日志级别

在运行时设置环境变量：

```bash
# 只显示错误和警告
export RUST_LOG=error,warn

# 或者更精确地控制特定模块
export RUST_LOG=error,client_api=warn,flowy_ai::agent=warn
```

### 方法2：修改日志配置

在 `rust-lib/lib-log/src/lib.rs` 或主程序入口修改日志过滤器：

```rust
// 示例：过滤特定模块的日志
tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::from_default_env()
            .add_directive("client_api::retry=warn".parse().unwrap())
            .add_directive("flowy_ai::agent::tool_call_handler=warn".parse().unwrap())
    )
    .init();
```

### 方法3：修改 Cargo.toml 依赖配置

如果有 `client-api` 库的源码访问权限，可以在该库中修改日志级别。

## 编译和生效

修改Rust代码后需要重新编译：

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend/rust-lib
cargo build --release

cd ..
flutter run
```

## 建议

### 保留的日志
- ✅ **错误日志（error!）** - 用于问题排查
- ✅ **警告日志（warn!）** - 用于注意潜在问题
- ✅ **关键业务日志** - 如工具调用结果、网络搜索结果

### 可以移除的日志
- ❌ **调试日志（debug!）** - 仅开发时需要
- ❌ **追踪日志（trace!）** - 详细的内部状态
- ❌ **频繁触发的info日志** - 如每次消息发送、WebSocket连接

### 条件编译日志

可以使用条件编译只在debug模式下输出日志：

```rust
#[cfg(debug_assertions)]
info!("这条日志只在debug模式下输出");
```

## 其他需要注意的日志

如果您在控制台看到其他不需要的日志，请告诉我具体的日志内容和模块名，我可以帮您定位并注释掉。

常见的日志模块：
- `flowy_ai::middleware::chat_service_mw` - AI中间件
- `flowy_ai::web_search` - 网络搜索
- `flowy_ai::agent` - 智能体
- `client_api` - 云服务客户端
- `collab` - 协作同步

## 验证

重新编译运行后，您应该不再看到：
- `🔍 [RELATED_Q]` 相关的日志
- `notify_did_send_message` 相关的日志

其他外部库的日志可以通过环境变量 `RUST_LOG` 控制。

