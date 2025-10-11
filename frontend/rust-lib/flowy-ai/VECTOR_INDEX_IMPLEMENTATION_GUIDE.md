# 向量索引重建功能实现指南

## ✅ 已完成

### 1. Protobuf 定义
- ✅ 添加了 `VectorIndexStatusPB`, `RebuildVectorIndexRequestPB`, `RebuildVectorIndexResponsePB` 到 `entities.proto`
- ✅ 添加了 `RebuildVectorIndex`, `GetVectorIndexStatus`, `StopVectorIndexing` 事件到 `event_map.proto`
- ✅ 添加了 `VectorIndexStatusUpdated` 通知到 `notification.proto`

### 2. 后端核心逻辑
- ✅ 创建了 `vector_index_manager.rs` 模块
- ✅ 实现了完整的索引状态管理和重建逻辑
- ✅ 实现了进度通知机制

## 🔧 需要手动完成的步骤

### 步骤 1: 修改 AIManager 构造函数

在 `rust-lib/flowy-ai/src/ai_manager.rs` 的 `AIManager::new()` 函数中：

```rust
pub fn new(
  chat_cloud_service: Arc<dyn ChatCloudService>,
  user_service: impl AIUserService,
  store_preferences: Arc<KVStorePreferences>,
  storage_service: Weak<dyn StorageService>,
  query_service: impl AIExternalService,
  local_ai: Arc<LocalAIController>,
) -> AIManager {
  // ... 现有代码 ...
  
  // 🆕 在 return 之前添加
  let external_service_arc = Arc::new(query_service);
  let folder_service = external_service_arc.clone(); // 转换为 Arc<dyn FolderService>
  let vector_index_manager = Arc::new(VectorIndexManager::new(folder_service));
  
  AIManager {
    // ... 现有字段 ...
    vector_index_manager, // 🆕 添加这个字段
  }
}
```

**注意：** 如果 `query_service` 没有实现 `FolderService` trait，您需要：
1. 在构造函数中添加 `folder_service: Arc<dyn FolderService>` 参数
2. 或者修改 `external_service` 使其实现 `FolderService` trait

### 步骤 2: 添加事件处理函数

在 `rust-lib/flowy-ai/src/event_handler.rs` 中添加以下三个函数：

```rust
// 在文件末尾添加

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn rebuild_vector_index_handler(
  data: AFPluginData<RebuildVectorIndexRequestPB>,
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<RebuildVectorIndexResponsePB, FlowyError> {
  let data = data.try_into_inner()?;
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  
  let workspace_id = ai_manager.user_service.workspace_id()?;
  let document_ids = if data.document_ids.is_empty() {
    None
  } else {
    Some(data.document_ids)
  };
  
  match ai_manager
    .vector_index_manager
    .rebuild_index(workspace_id, document_ids)
    .await
  {
    Ok(_) => data_result_ok(RebuildVectorIndexResponsePB {
      success: true,
      error: None,
    }),
    Err(err) => data_result_ok(RebuildVectorIndexResponsePB {
      success: false,
      error: Some(err.to_string()),
    }),
  }
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn get_vector_index_status_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> DataResult<VectorIndexStatusPB, FlowyError> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  let manager = &ai_manager.vector_index_manager;
  
  let state = match manager.get_state().await {
    crate::vector_index_manager::VectorIndexState::Idle => VectorIndexStatePB::IndexIdle,
    crate::vector_index_manager::VectorIndexState::Running => VectorIndexStatePB::IndexRunning,
    crate::vector_index_manager::VectorIndexState::Completed => VectorIndexStatePB::IndexCompleted,
    crate::vector_index_manager::VectorIndexState::Failed => VectorIndexStatePB::IndexFailed,
    crate::vector_index_manager::VectorIndexState::Stopping => VectorIndexStatePB::IndexStopping,
  };
  
  data_result_ok(VectorIndexStatusPB {
    state,
    total_documents: manager.get_total_documents(),
    indexed_documents: manager.get_indexed_documents(),
    recent_logs: manager.get_recent_logs().await,
    error: manager.get_error().await,
    start_time: manager.get_start_time(),
    last_update_time: manager.get_last_update_time(),
  })
}

#[tracing::instrument(level = "debug", skip_all, err)]
pub(crate) async fn stop_vector_indexing_handler(
  ai_manager: AFPluginState<Weak<AIManager>>,
) -> FlowyResult<()> {
  let ai_manager = upgrade_ai_manager(ai_manager)?;
  ai_manager.vector_index_manager.stop_indexing().await;
  Ok(())
}
```

### 步骤 3: 注册事件映射

在 `rust-lib/flowy-ai/src/event_map.rs` 中注册事件：

```rust
// 找到事件注册的位置，添加以下三行

AIEvent::RebuildVectorIndex => {
  make_async_handler!(rebuild_vector_index_handler)
}

AIEvent::GetVectorIndexStatus => {
  make_async_handler!(get_vector_index_status_handler)
}

AIEvent::StopVectorIndexing => {
  make_async_handler!(stop_vector_indexing_handler)
}
```

### 步骤 4: 重新编译生成 Dart 代码

```bash
cd appflowy_flutter
cargo make --profile development-mac-arm64 flowy-sdk-release
```

这将自动生成 Dart 的事件绑定和 Protobuf 类型。

## 📱 前端实现 (Flutter)

生成的 Dart 代码将位于：
- `lib/protobuf/flowy-ai/event_map.pb.dart` - 事件定义
- `lib/protobuf/flowy-ai/entities.pb.dart` - 消息类型
- `lib/protobuf/flowy-ai/notification.pb.dart` - 通知类型

### 前端需要创建的文件

1. **索引重建 UI 组件**
   - 文件：`appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/vector_index_setting.dart`
   - 包含：重建按钮、进度显示、日志显示

2. **状态管理 Bloc**
   - 文件：`appflowy_flutter/lib/workspace/application/settings/ai/vector_index_bloc.dart`
   - 管理索引状态、监听通知更新

3. **集成到 AI 设置页面**
   - 修改：`appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart`
   - 添加索引重建组件

## 📊 功能特性

✅ **已实现：**
- 一键重建所有文档索引
- 实时进度显示（X/Y 文档已索引）
- 详细的索引日志
- 支持停止索引
- 后台执行，不阻塞 UI
- 自动状态通知

✅ **用户体验：**
- 可以关闭设置页面，索引继续在后台执行
- 重新打开设置页面，可以看到最新进度
- 失败时显示详细错误信息
- 日志自动保留最近 50 条记录

## 🧪 测试建议

1. **基本功能测试**
   - 点击"重建索引"按钮
   - 观察进度更新
   - 查看日志输出

2. **边界情况测试**
   - 没有文档时重建索引
   - 索引过程中停止
   - 索引失败时的错误处理

3. **并发测试**
   - 索引运行时再次点击重建（应拒绝）
   - 关闭设置页面后重新打开

## 📝 注意事项

1. **Folder Service 依赖**
   - `VectorIndexManager` 需要 `Arc<dyn FolderService>` 来获取文档列表
   - 确保 `AIManager` 构造时能够提供这个依赖

2. **权限检查**
   - 可以考虑添加权限检查，只允许管理员重建索引

3. **性能优化**
   - 当前每个文档间隔 100ms，可根据实际情况调整
   - 考虑添加批量索引选项

4. **通知优化**
   - 当前每个文档更新都发送通知，可以改为每 N 个文档发送一次

## 🚀 后续优化建议

1. **选择性索引**
   - 允许用户选择特定文档进行索引
   - UI 中显示文档列表并支持多选

2. **增量索引**
   - 只索引有变更的文档
   - 减少不必要的重建

3. **定时自动索引**
   - 添加后台定时任务
   - 自动检测并索引新文档

4. **索引质量报告**
   - 显示索引覆盖率
   - 显示向量相似度分布统计

