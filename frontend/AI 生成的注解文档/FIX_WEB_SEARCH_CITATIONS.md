# 修复：重启后工具调用引用丢失问题（web_search + MCP 工具）

## 问题描述

在 AI 聊天中，提问后调用了工具（网络搜索或 MCP 工具），实时显示正常，但关闭并重新打开应用后，UI 中的引用列表只能看到本地文档的信息，而工具调用的引用就消失了。

**影响的工具类型**：
- ✅ AppFlowy 内置 `web_search` 工具
- ✅ 所有 MCP 工具（如 `web_baidu_search` 等）

## 根本原因

1. **实时聊天时**：
   - Rust 后端发送包含 `tool_call` 的 metadata
   - 前端 `parseMetadata` 函数从 `tool_call.result` 字符串中提取网络搜索引用
   - 提取的引用被存储在内存中的 `SourcesManager`
   - Rust 后端保存消息时，只保存了 `tool_calls` 数组，**没有保存提取后的引用列表**

2. **重启后恢复时**：
   - 从数据库读取的 metadata 结构是：
     ```json
     {
       "tool_calls": [...],
       "execution_logs": [...]
     }
     ```
   - 前端 `chat_message_handler.dart` 期望 metadata 有 `sources` 字段：
     ```json
     {
       "sources": [
         {"id": "url", "name": "title", "source": "web"}
       ]
     }
     ```
   - 因为数据库中没有这个字段，所以网络搜索的引用丢失了

## 解决方案

在 Rust 后端保存消息到数据库之前，从 `tool_calls` 数组中提取工具调用信息：
- **web_search 工具**：从结果字符串中提取 URL 引用
- **MCP 工具**：创建工具引用（包含工具名称和 server 信息）

将提取的引用添加到 metadata 的 `sources` 字段中，这样重启后可以从数据库恢复。

## 修改的文件

### 1. `rust-lib/flowy-ai/src/chat.rs`

#### 修改 1：在保存前提取引用（第 1087-1160 行）

在保存消息之前，添加了以下逻辑：

```rust
// 🔧 关键修复：从 tool_calls 中提取引用信息并添加到 sources 字段
// 这样重启后可以从数据库恢复引用信息（支持 web_search 和 MCP 工具）
if let Some(metadata_obj) = metadata.as_mut() {
  if let Some(obj) = metadata_obj.as_object() {
    let mut sources = Vec::new();
    
    // 检查是否有 tool_calls 数组
    if let Some(tool_calls) = obj.get("tool_calls") {
      if let Some(calls_array) = tool_calls.as_array() {
        for tool_call in calls_array {
          if let Some(tool_obj) = tool_call.as_object() {
            // 获取工具调用的基本信息
            let tool_name = tool_obj.get("tool_name").and_then(|v| v.as_str());
            let status = tool_obj.get("status").and_then(|v| v.as_str());
            let tool_id = tool_obj.get("id").and_then(|v| v.as_str());
            
            // 只处理成功的工具调用
            if let (Some(name), Some("success")) = (tool_name, status) {
              // 处理 AppFlowy 内置的 web_search 工具
              if name == "web_search" {
                if let Some(result) = tool_obj.get("result").and_then(|v| v.as_str()) {
                  // 从搜索结果中提取 URL 引用
                  let citations = extract_citations_from_search_result(result);
                  if !citations.is_empty() {
                    info!("📎 [METADATA] 从 web_search 结果中提取了 {} 个引用", citations.len());
                    sources.extend(citations);
                  }
                }
              } 
              // 处理 MCP 工具（非 web_search 的所有其他工具）
              else {
                // 提取 server 信息（如果 tool_name 包含 server 前缀）
                // 格式: "server_name.tool_name" 或直接 "tool_name"
                let (display_name, server_id) = if name.contains('.') {
                  let parts: Vec<&str> = name.split('.').collect();
                  if parts.len() >= 2 {
                    let server = parts[0];
                    let tool = parts[1..].join(".");
                    (tool, Some(server))
                  } else {
                    (name.to_string(), None)
                  }
                } else {
                  (name.to_string(), None)
                };
                
                // 创建 MCP 引用
                let mcp_source = format!("mcp:{}", server_id.unwrap_or("unknown"));
                let citation = serde_json::json!({
                  "id": tool_id.unwrap_or(name),
                  "name": display_name,
                  "source": mcp_source
                });
                
                sources.push(citation);
                info!("📎 [METADATA] 添加 MCP 工具引用: {} (source: {})", display_name, mcp_source);
              }
            }
          }
        }
      }
    }
    
    // 如果提取到了引用，添加到 sources 字段
    if !sources.is_empty() {
      if let Some(metadata_obj_mut) = metadata.as_mut() {
        if let Some(obj_mut) = metadata_obj_mut.as_object_mut() {
          obj_mut.insert("sources".to_string(), serde_json::json!(sources));
          info!("✅ [METADATA] 已将 {} 个引用添加到 metadata.sources", sources.len());
        }
      }
    }
  }
}
```

**工作原理**：
- 遍历 `tool_calls` 数组中的每个工具调用
- 只处理状态为 `success` 的工具调用
- **web_search 工具**：使用正则表达式从 result 字符串中提取 URL 引用
- **MCP 工具**（所有其他工具）：
  - 从工具名称中提取 server 信息（格式：`server.tool` 或 `tool`）
  - 创建 MCP 引用，包含工具 ID、显示名称和 source（`mcp:server_id`）
- 将所有提取的引用添加到 metadata 的 `sources` 字段中

#### 修改 2：添加引用提取辅助函数（第 1442-1477 行）

```rust
/// 从网络搜索结果字符串中提取引用信息
/// 
/// 搜索结果格式示例：
/// ```
/// 搜索结果 (查询词):
/// 1. 标题1
///    链接: https://example.com/1
/// 2. 标题2
///    链接: https://example.com/2
/// ```
fn extract_citations_from_search_result(result: &str) -> Vec<serde_json::Value> {
  use regex::Regex;
  
  let mut citations = Vec::new();
  
  // 使用正则表达式匹配引用模式
  // 匹配格式: 数字. 标题\n   链接: URL
  let pattern = Regex::new(r"(\d+)\.\s+([^\n]+)\s+链接:\s+(https?://[^\s]+)").unwrap();
  
  for cap in pattern.captures_iter(result) {
    if let (Some(_index), Some(title), Some(url)) = (cap.get(1), cap.get(2), cap.get(3)) {
      let title_str = title.as_str().trim();
      let url_str = url.as_str().trim();
      
      if !title_str.is_empty() && !url_str.is_empty() {
        citations.push(serde_json::json!({
          "id": url_str,
          "name": title_str,
          "source": "web"
        }));
      }
    }
  }
  
  citations
}
```

**功能**：
- 使用与前端相同的正则表达式逻辑提取引用
- 匹配格式：`数字. 标题\n   链接: URL`
- 返回引用的 JSON 数组

### 2. `rust-lib/flowy-ai/Cargo.toml`

添加了 `regex` 依赖：

```toml
regex = "1.10"
```

## 测试步骤

1. **重新编译应用**：
   ```bash
   cd /Users/niuzhidao/Documents/Program/t1mind/frontend
   # 编译 Rust 代码
   cd rust-lib
   cargo build
   # 或者重新运行 Flutter 应用
   cd ../appflowy_flutter
   flutter run
   ```

2. **发起新的对话**：
   - 在 AI 聊天中提一个问题，触发工具调用（web_search 或 MCP 工具）
   - 确认实时显示时可以看到工具的引用

3. **重启应用**：
   - 完全关闭应用
   - 重新打开应用
   - 打开之前的聊天记录

4. **验证修复**：
   - 检查引用列表是否同时显示：
     - ✅ 本地文档引用
     - ✅ 网络搜索引用（之前会消失，现在应该能看到）
     - ✅ MCP 工具引用（之前会消失，现在应该能看到）

## 预期结果

重启后，数据库中保存的 metadata 结构应该是：

### 示例 1：web_search 工具

```json
{
  "tool_calls": [
    {
      "id": "...",
      "tool_name": "web_search",
      "status": "success",
      "result": "搜索结果..."
    }
  ],
  "sources": [
    {
      "id": "https://example.com/1",
      "name": "标题1",
      "source": "web"
    },
    {
      "id": "https://example.com/2",
      "name": "标题2",
      "source": "web"
    }
  ],
  "execution_logs": [...]
}
```

### 示例 2：MCP 工具

```json
{
  "tool_calls": [
    {
      "id": "tool_call_123",
      "tool_name": "mcp_server.web_baidu_search",
      "status": "success",
      "result": "..."
    }
  ],
  "sources": [
    {
      "id": "tool_call_123",
      "name": "web_baidu_search",
      "source": "mcp:mcp_server"
    }
  ],
  "execution_logs": [...]
}
```

### 示例 3：混合使用（文档 + web_search + MCP）

```json
{
  "tool_calls": [...],
  "sources": [
    {
      "id": "doc-uuid-123",
      "name": "Loading...",
      "source": "appflowy"
    },
    {
      "id": "https://example.com/article",
      "name": "文章标题",
      "source": "web"
    },
    {
      "id": "tool_call_456",
      "name": "calculator",
      "source": "mcp:math_server"
    }
  ],
  "execution_logs": [...]
}
```

前端恢复时，会从 `sources` 字段读取引用信息，因此所有工具的引用都不会丢失。

## 日志输出

成功修复后，在保存消息时应该能看到以下日志：

### web_search 工具
```
📎 [METADATA] 从 web_search 结果中提取了 X 个引用
✅ [METADATA] 已将 X 个引用添加到 metadata.sources
```

### MCP 工具
```
📎 [METADATA] 添加 MCP 工具引用: tool_name (source: mcp:server_id)
✅ [METADATA] 已将 X 个引用添加到 metadata.sources
```

## 注意事项

1. **只影响新消息**：这个修复只对新创建的聊天消息生效。已有的消息（修复前创建的）不会自动修复。

2. **兼容性**：修改不会破坏现有的消息格式，因为：
   - 旧消息没有 `sources` 字段：前端会尝试从内存恢复（如果还在内存中）
   - 新消息有 `sources` 字段：前端优先使用这个字段

3. **工具覆盖**：
   - ✅ **web_search**：提取 URL 引用
   - ✅ **所有 MCP 工具**：自动创建工具引用
   - 如果将来添加其他特殊的引用提取逻辑，可以在代码中扩展

## 相关文件

- 前端引用解析：`appflowy_flutter/lib/plugins/ai_chat/application/chat_message_service.dart` (line 230-261)
- 前端引用恢复：`appflowy_flutter/lib/plugins/ai_chat/application/chat_message_handler.dart` (line 61-129)
- 引用显示组件：`appflowy_flutter/lib/plugins/ai_chat/widgets/unified_reference_display.dart`

