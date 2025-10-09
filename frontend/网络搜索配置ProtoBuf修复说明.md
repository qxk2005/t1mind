# 网络搜索配置 Protocol Buffer 修复说明

## 问题描述

运行应用时出现 Protocol Buffer 解析错误：
```
[error] Can not parse error bytes: InvalidProtocolBufferException: Protocol message end-group tag did not match expected tag.
[error] 加载网络搜索全局配置异常: InvalidProtocolBufferException: Protocol message end-group tag did not match expected tag.
```

## 问题原因

在 Rust 代码中存在类型定义冲突：

1. **`rust-lib/flowy-ai/src/entities.rs`**：定义了带有 `ProtoBuf` derive 的网络搜索实体，用于与 Flutter 前端通信
2. **`rust-lib/flowy-ai/src/web_search/entities.rs`**：定义了不带 `ProtoBuf` derive 的重复实体，用于内部业务逻辑

这导致 web_search 模块内部使用的类型和 event_handler 返回给前端的类型不一致，造成序列化/反序列化失败。

## 修复方案

### 1. 统一类型定义 (已完成)

修改 `rust-lib/flowy-ai/src/web_search/entities.rs`，从重复定义改为重新导出主 entities 模块中的类型：

```rust
// Re-export web search entities from main entities module
// This ensures all web search related types use the ProtoBuf-enabled versions

pub use crate::entities::{
    // Provider related
    WebSearchProviderConfigPB,
    WebSearchProviderListPB,
    CreateWebSearchProviderRequestPB,
    UpdateWebSearchProviderRequestPB,
    DeleteWebSearchProviderRequestPB,
    GetWebSearchProviderRequestPB,
    TestWebSearchProviderRequestPB,
    TestWebSearchProviderResponsePB,
    WebSearchTestResultPB,
    WebSearchProviderTypePB,
    ProviderTestStatusPB,
    
    // Search request/response
    WebSearchRequestPB,
    WebSearchResponsePB,
    WebSearchResultPB,
    
    // Global config
    WebSearchGlobalConfigPB,
    UpdateWebSearchGlobalConfigRequestPB,
    
    // Cache related
    WebSearchCacheEntryPB,
    WebSearchCacheStatsPB,
    
    // Event related
    WebSearchEventPB,
    WebSearchEventTypePB,
};
```

### 2. 重新编译 Rust 代码 (已完成)

```bash
cd rust-lib
cargo build
```

### 3. 重新生成 Protocol Buffer 代码 (已完成)

```bash
cargo make code_generation
```

### 4. 清理并重新构建 Flutter 项目 (已完成)

```bash
cd appflowy_flutter
flutter clean
flutter pub get
```

## 验证修复

现在你可以重新运行应用程序，应该不会再出现 Protocol Buffer 解析错误了。

如果仍然遇到问题，请检查：
1. Rust 代码编译是否成功
2. Protocol Buffer 代码生成是否完整
3. Flutter 依赖是否正确安装

## 影响范围

此修复影响以下模块：
- ✅ Web Search Hub
- ✅ Web Search Provider Manager
- ✅ Web Search Result Processor
- ✅ Web Search Cache Manager
- ✅ Web Search Event Handler
- ✅ Web Search Tools

所有这些模块现在统一使用带有 ProtoBuf derive 的类型定义。

## 测试建议

建议测试以下功能：
1. 加载网络搜索全局配置
2. 更新网络搜索全局配置
3. 添加/删除/更新搜索供应商
4. 执行网络搜索
5. 查看缓存统计

---

**修复完成时间**: 2025-10-08
**修复状态**: ✅ 完成



