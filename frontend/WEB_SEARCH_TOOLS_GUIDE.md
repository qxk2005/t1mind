# 网络搜索工具说明

## 问题：为什么出现两个搜索工具？

在AI聊天中，您看到了 `web_search` 和 `quick_search` 两个工具同时被调用。

### 原因

系统原本设计了两个网络搜索工具：

1. **`web_search`** - 通用网络搜索工具
   - 支持完整的搜索参数：查询、最大结果数、语言、区域、内容详情
   - 适合需要详细信息的场景

2. **`quick_search`** - 快速搜索工具
   - 只需要查询参数
   - 优化用于快速获取简短答案
   - 元数据标记为 "fast"

AI模型看到这两个工具后，**自主决定**同时调用它们，可能是为了：
- 并行搜索以获得更全面的结果
- 多源验证信息准确性
- 结合速度和质量优势

## 解决方案

### 已实施的修改

为了避免重复调用，我们已经**禁用了 `quick_search` 工具**：

#### 修改的文件

1. **rust-lib/flowy-ai/src/web_search/tool.rs**
   - 注释掉了 `quick_search` 工具的定义

2. **rust-lib/flowy-ai/src/agent/config_manager.rs**
   - 从默认工具列表中移除 `quick_search`

3. **rust-lib/flowy-ai/src/ai_manager.rs**
   - 从内置工具配置中移除 `quick_search`

4. **rust-lib/flowy-ai/src/agent/tool_call_handler.rs**
   - 从工具执行器中移除 `quick_search` 支持

### 生效步骤

修改Rust代码后需要重新编译：

```bash
# 停止应用
# 重新编译Rust代码
cd rust-lib
cargo build --release

# 重新运行Flutter应用
cd ..
flutter run
```

### 效果

修改后，AI只会调用 `web_search` 工具，避免了重复搜索。

## 如果您想恢复双工具模式

如果您确实希望保留两个搜索工具（让AI自主选择），可以：

1. 取消注释 `web_search/tool.rs` 中的 `quick_search` 定义
2. 恢复其他文件中的 `quick_search` 配置
3. 重新编译

或者，您可以修改 `quick_search` 的描述，让AI更清楚地理解两者的区别：

```rust
description: "仅当需要快速、简短的答案时使用。如果需要详细信息，使用web_search。".to_string(),
```

## 其他网络搜索工具

系统还定义了其他搜索工具（默认也已注释）：

- **`news_search`** - 新闻搜索
- **`academic_search`** - 学术搜索  
- **`image_search`** - 图片搜索
- **`video_search`** - 视频搜索

如果需要启用这些工具，取消相应的注释即可。

## 工具调用优化建议

### 1. 清晰的工具描述
确保每个工具的描述清楚说明其用途和适用场景，帮助AI做出正确选择。

### 2. 工具参数设计
- 避免功能重叠的工具
- 参数设计应该明确体现工具的特点

### 3. 监控和日志
保留工具调用日志，以便了解AI的决策过程：
```
[TOOL CALL] web_search: 查询="珠江天河都荟"
[TOOL CALL] quick_search: 查询="珠江天河都荟"
```

## 相关配置

### 查看当前可用工具

在AI聊天设置中，可以查看和管理可用的工具列表。

### 智能体工具配置

每个AI智能体可以配置自己的可用工具列表：
- 如果列表为空，允许所有工具
- 如果指定了工具列表，只能使用列表中的工具

## 技术细节

### 工具注册流程

```
WebSearchToolManager::get_tool_definitions()
  ↓
AgentManager::initialize()
  ↓
ToolCallHandler::execute_tool()
  ↓
WebSearchHub::search()
```

### 多工具并行执行

系统支持并行执行多个工具调用，这是AI决策的结果，不是bug。如果不希望并行，需要在工具定义层面避免功能重叠。

