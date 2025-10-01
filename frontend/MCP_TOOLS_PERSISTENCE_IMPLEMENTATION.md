# MCP 工具持久化功能实现总结

## 功能概述

为 MCP 服务器添加工具缓存和最后检查时间的持久化功能，使得关闭窗口后再次打开时可以直接显示缓存的工具信息，无需重新检查。

## 实现的功能

### 1. 后端数据结构更新 ✅

#### Rust 内部实体（`rust-lib/flowy-ai/src/mcp/entities.rs`）
```rust
pub struct MCPServerConfig {
    // ... 原有字段 ...
    /// 缓存的工具列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tools: Option<Vec<MCPTool>>,
    /// 最后检查工具的时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tools_check_at: Option<SystemTime>,
}
```

#### Protobuf 定义（`rust-lib/flowy-ai/resources/proto/entities.proto`）
```protobuf
message MCPServerConfigPB {
    // ... 原有字段 ...
    oneof one_of_cached_tools { MCPToolListPB cached_tools = 9; };
    oneof one_of_last_tools_check_at { int64 last_tools_check_at = 10; };
}
```

### 2. 配置管理器功能扩展 ✅

#### 新增方法（`rust-lib/flowy-ai/src/mcp/config.rs`）

**保存工具缓存**：
```rust
pub fn save_tools_cache(&self, server_id: &str, tools: Vec<MCPTool>) -> FlowyResult<()> {
    let mut config = self.get_server(server_id)?;
    config.cached_tools = Some(tools.clone());
    config.last_tools_check_at = Some(SystemTime::now());
    config.updated_at = SystemTime::now();
    self.save_server(config)?;
    Ok(())
}
```

**获取缓存工具**：
```rust
pub fn get_cached_tools(&self, server_id: &str) -> Option<(Vec<MCPTool>, SystemTime)> {
    let config = self.get_server(server_id)?;
    config.cached_tools.zip(config.last_tools_check_at)
}
```

### 3. 自动缓存机制 ✅

#### 连接时自动缓存（`rust-lib/flowy-ai/src/mcp/manager.rs`）
```rust
pub async fn connect_server(&self, config: MCPServerConfig) -> Result<(), FlowyError> {
    // ... 创建客户端 ...
    
    // 发现工具并保存到缓存
    match self.tool_discovery.discover_tools(&config.id).await {
        Ok(tools) => {
            // 保存工具缓存到配置
            if let Err(e) = self.config_manager.save_tools_cache(&config.id, tools) {
                tracing::warn!("Failed to save tools cache: {}", e);
            }
        }
        // ...
    }
    Ok(())
}
```

### 4. 前端 UI 更新 ✅

#### 显示缓存工具（`workspace_mcp_settings_v2.dart`）

**优先显示实时工具，否则使用缓存**：
```dart
...state.servers.map((server) {
  // 优先使用实时获取的工具，否则使用缓存
  final realTimeTools = state.serverTools[server.id];
  final cachedTools = server.cachedTools?.tools ?? [];
  final tools = realTimeTools ?? cachedTools;
  // ...
})
```

**显示最后检查时间**：
```dart
// 最后检查时间
if (server.hasLastToolsCheckAt()) ...[
  const SizedBox(height: 8),
  _buildLastCheckTime(context, server.lastToolsCheckAt),
],
```

**时间格式化显示**：
```dart
Widget _buildLastCheckTime(BuildContext context, int timestamp) {
  final checkTime = DateTime.fromMillisecondsSinceEpoch(timestamp * 1000);
  final now = DateTime.now();
  final difference = now.difference(checkTime);
  
  String timeText;
  if (difference.inMinutes < 1) {
    timeText = '刚刚';
  } else if (difference.inMinutes < 60) {
    timeText = '${difference.inMinutes}分钟前';
  } else if (difference.inHours < 24) {
    timeText = '${difference.inHours}小时前';
  } else if (difference.inDays < 7) {
    timeText = '${difference.inDays}天前';
  } else {
    timeText = '${checkTime.year}-${checkTime.month.toString().padLeft(2, '0')}-${checkTime.day.toString().padLeft(2, '0')} ${checkTime.hour.toString().padLeft(2, '0')}:${checkTime.minute.toString().padLeft(2, '0')}';
  }
  
  return Row(
    children: [
      Icon(Icons.schedule, size: 12, color: Theme.of(context).hintColor),
      const SizedBox(width: 4),
      FlowyText.regular(
        '最后检查: $timeText',
        fontSize: 11,
        color: Theme.of(context).hintColor,
      ),
    ],
  );
}
```

### 5. 一键检查功能增强 ✅

**检查后自动刷新列表**：
```dart
void _checkAllServers(BuildContext context, MCPSettingsState state) {
  final bloc = context.read<MCPSettingsBloc>();
  
  int checkCount = 0;
  for (final server in state.servers) {
    final isConnected = state.serverStatuses[server.id]?.isConnected ?? false;
    final hasTools = state.serverTools[server.id]?.isNotEmpty ?? false;
    
    if (!isConnected) {
      bloc.add(MCPSettingsEvent.connectServer(server.id));
      checkCount++;
    } else if (!hasTools && !state.loadingTools.contains(server.id)) {
      bloc.add(MCPSettingsEvent.refreshTools(server.id));
      checkCount++;
    }
  }
  
  // 延迟后重新加载服务器列表以获取更新的缓存数据
  if (checkCount > 0) {
    Future.delayed(const Duration(seconds: 3), () {
      if (context.mounted) {
        bloc.add(const MCPSettingsEvent.loadServerList());
      }
    });
  }
}
```

### 6. 工具标签颜色优化 ✅

**之前（低对比度）**：
- 背景：`secondaryContainer`（浅蓝色）
- 文字：`onSecondaryContainer`（浅色/白色）

**现在（高对比度）**：
- 背景：`surfaceVariant`（浅灰色）
- 文字：`onSurfaceVariant`（深色）
- 悬停时：主题色背景 + 主题色文字 + 加粗边框

## Protobuf 代码生成问题 ⚠️

### 问题描述
Dart protobuf 代码未自动更新，需要手动重新生成。

### 生成要求
Dart protobuf 生成需要两个环境变量：
- `CARGO_MAKE_WORKING_DIRECTORY`
- `FLUTTER_FLOWY_SDK_PATH`

### 手动生成命令

**方式 1：使用环境变量**
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend/rust-lib/dart-ffi
CARGO_MAKE_WORKING_DIRECTORY=/Users/niuzhidao/Documents/Program/t1mind/frontend \
FLUTTER_FLOWY_SDK_PATH=appflowy_flutter/packages/appflowy_backend \
cargo build --features dart
```

**方式 2：使用 cargo-make**
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
cargo make appflowy-core-dev
```

**方式 3：删除旧文件并重新生成**
```bash
# 1. 删除旧的 Dart protobuf 文件
rm -rf appflowy_flutter/packages/appflowy_backend/lib/protobuf/flowy-ai/entities.pb.dart

# 2. 触碰 proto 文件
touch rust-lib/flowy-ai/resources/proto/entities.proto

# 3. 使用正确的环境变量重新编译
cd rust-lib/dart-ffi
CARGO_MAKE_WORKING_DIRECTORY=/Users/niuzhidao/Documents/Program/t1mind/frontend \
FLUTTER_FLOWY_SDK_PATH=appflowy_flutter/packages/appflowy_backend \
cargo clean && cargo build --features dart
```

## 数据流程

### 1. 工具缓存保存流程
```
连接服务器 → 发现工具 → save_tools_cache() 
→ 更新 cached_tools & last_tools_check_at 
→ 保存到 SQLite
```

### 2. 工具显示流程
```
加载服务器列表 → 读取配置（包含缓存）
→ 前端优先显示实时工具 → 无实时工具则显示缓存
```

### 3. 时间更新流程
```
每次工具检查成功 → 更新 last_tools_check_at 
→ UI 显示相对时间（刚刚/N分钟前/N小时前/日期）
```

## 用户体验提升

### 1. 启动速度优化
- ✅ 无需等待工具检查即可看到缓存的工具信息
- ✅ 后台异步更新工具列表

### 2. 信息完整性
- ✅ 显示工具标签，一目了然
- ✅ 显示最后检查时间，提醒用户更新
- ✅ 支持手动"一键检查"更新所有服务器

### 3. 视觉优化
- ✅ 工具标签颜色对比度提升
- ✅ 时间显示人性化（相对时间）
- ✅ 悬停效果明显

## 测试步骤

### 1. 基本功能测试
1. 添加 MCP 服务器并连接
2. 等待工具加载完成
3. 关闭应用
4. 重新打开应用
5. ✅ 验证：工具标签立即显示（从缓存）
6. ✅ 验证：显示"最后检查"时间

### 2. 一键检查测试
1. 点击"一键检查"按钮
2. ✅ 验证：未连接的服务器自动连接
3. ✅ 验证：已连接但无工具的服务器重新加载
4. ✅ 验证：3秒后列表自动刷新，显示更新的时间

### 3. 时间显示测试
- ✅ 刚检查：显示"刚刚"
- ✅ 5分钟前：显示"5分钟前"
- ✅ 2小时前：显示"2小时前"
- ✅ 昨天：显示"1天前"
- ✅ 一周前：显示完整日期时间

## 相关文件

### 后端（Rust）
- `rust-lib/flowy-ai/src/mcp/entities.rs` - 数据结构定义
- `rust-lib/flowy-ai/src/mcp/config.rs` - 配置管理器
- `rust-lib/flowy-ai/src/mcp/manager.rs` - MCP 管理器
- `rust-lib/flowy-ai/src/mcp/event_handler.rs` - 事件处理
- `rust-lib/flowy-ai/resources/proto/entities.proto` - Protobuf 定义
- `rust-lib/flowy-ai/src/entities.rs` - Protobuf Rust 绑定

### 前端（Dart/Flutter）
- `appflowy_flutter/lib/workspace/presentation/settings/workspace/workspace_mcp_settings_v2.dart` - MCP 设置 UI
- `appflowy_flutter/lib/plugins/ai_chat/application/mcp_settings_bloc.dart` - 状态管理
- `appflowy_flutter/packages/appflowy_backend/lib/protobuf/flowy-ai/entities.pb.dart` - Protobuf Dart 绑定（需重新生成）

## 实现日期

2025-10-01

## 相关文档

- [MCP_TOOL_TAG_COLOR_FIX.md](./MCP_TOOL_TAG_COLOR_FIX.md) - 工具标签颜色优化
- [MCP_MVP_COMPLETED.md](./MCP_MVP_COMPLETED.md) - MCP MVP 完成总结
- [MCP_PERSISTENCE_IMPLEMENTATION_PLAN.md](./MCP_PERSISTENCE_IMPLEMENTATION_PLAN.md) - 持久化实现计划

