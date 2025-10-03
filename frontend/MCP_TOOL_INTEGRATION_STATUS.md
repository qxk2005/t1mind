# MCP 工具集成实现状态

## ✅ 已完成的后端功能

### 1. Protobuf 实体定义 ✅
所有必要的实体都已定义在 `rust-lib/flowy-ai/src/entities.rs`:

- `MCPToolPB` - 工具定义（名称、描述、输入模式、注解）
- `MCPToolAnnotationsPB` - 工具注解（只读、破坏性、幂等性等）
- `MCPToolListPB` - 工具列表响应
- `MCPToolCallRequestPB` - 工具调用请求
- `MCPToolCallResponsePB` - 工具调用响应
- `MCPContentPB` - 内容项（文本、图片、资源）
- `MCPServerStatusPB` - 服务器状态（包含工具数量）

### 2. 后端事件处理器 ✅
已在 `rust-lib/flowy-ai/src/mcp/event_handler.rs` 实现:

- `get_mcp_tool_list_handler` - 获取指定服务器的工具列表
- `call_mcp_tool_handler` - 调用指定服务器的工具
- `connect_mcp_server_handler` - 连接服务器
- `disconnect_mcp_server_handler` - 断开服务器连接
- `get_mcp_server_status_handler` - 获取服务器状态

### 3. 后端事件映射 ✅
在 `rust-lib/flowy-ai/src/event_map.rs` 中定义:

```rust
GetMCPToolList = 44,           // 获取工具列表
CallMCPTool = 45,              // 调用工具
ConnectMCPServer = 41,         // 连接服务器
DisconnectMCPServer = 42,      // 断开服务器
GetMCPServerStatus = 43,       // 获取服务器状态
```

## 🚧 需要实现的功能

### 1. 工具信息持久化 🔨
**位置**: `rust-lib/flowy-ai/src/mcp/config.rs`

需要添加:
- `save_tool_cache(server_id: &str, tools: Vec<MCPTool>)` - 缓存工具信息
- `get_tool_cache(server_id: &str) -> Option<Vec<MCPTool>>` - 读取缓存的工具
- `clear_tool_cache(server_id: &str)` - 清除工具缓存

**数据结构**:
```rust
pub struct MCPToolCache {
    pub server_id: String,
    pub tools: Vec<MCPTool>,
    pub cached_at: i64, // 时间戳
}
```

### 2. 会话管理 🔨
**位置**: `rust-lib/flowy-ai/src/mcp/session.rs` (新文件)

需要实现:
```rust
pub struct MCPSessionManager {
    sessions: HashMap<String, MCPSession>,
}

pub struct MCPSession {
    pub server_id: String,
    pub session_id: Option<String>,
    pub initialized: bool,
    pub tools: Vec<MCPTool>,
    pub connection_time: i64,
}

impl MCPSessionManager {
    pub fn create_session(server_id: String) -> MCPSession;
    pub fn get_session(server_id: &str) -> Option<&MCPSession>;
    pub fn update_session_tools(server_id: &str, tools: Vec<MCPTool>);
    pub fn close_session(server_id: &str);
}
```

### 3. SSE 客户端 🔨
**位置**: `rust-lib/flowy-ai/src/mcp/sse_client.rs` (新文件)

需要实现:
```rust
pub struct SSEClient {
    url: String,
    headers: HashMap<String, String>,
}

impl SSEClient {
    pub async fn connect() -> Result<SSEStream>;
    pub async fn send_message(message: MCPMessage) -> Result<MCPMessage>;
    pub fn parse_sse_response(data: &str) -> Result<Vec<MCPMessage>>;
}
```

SSE 响应格式:
```
event: message
data: {"jsonrpc":"2.0","id":1,"result":{...}}

event: message
data: {"jsonrpc":"2.0","id":2,"result":{...}}
```

### 4. Flutter 前端集成 🔨

#### 4.1 更新 MCPSettingsBloc
**位置**: `appflowy_flutter/lib/plugins/ai_chat/application/mcp_settings_bloc.dart`

需要添加:
- `loadToolList(String serverId)` - 加载工具列表
- `callTool(String serverId, String toolName, Map<String, dynamic> args)` - 调用工具
- `refreshTools(String serverId)` - 刷新工具列表

**State 更新**:
```dart
class MCPSettingsState {
  final Map<String, List<MCPToolPB>> serverTools; // 新增
  final Map<String, bool> loadingTools; // 新增
  final String? selectedServerId; // 新增
  final MCPToolCallResponsePB? lastToolResponse; // 新增
}
```

#### 4.2 UI 组件
**位置**: `appflowy_flutter/lib/workspace/presentation/settings/workspace/`

需要创建:
- `mcp_tool_list_widget.dart` - 工具列表展示
- `mcp_tool_call_dialog.dart` - 工具调用对话框
- `mcp_tool_result_viewer.dart` - 工具调用结果展示

### 5. AI 大模型集成 🔨

#### 5.1 工具调用协议
AI 模型需要能够:
1. 获取可用工具列表
2. 根据上下文选择合适的工具
3. 构造工具调用参数
4. 处理工具调用结果

#### 5.2 Function Calling 支持
对于支持 Function Calling 的模型（如 GPT-4）:
```json
{
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "mcp_read_data_from_excel",
        "description": "从 Excel 文件读取数据",
        "parameters": {
          "type": "object",
          "properties": {
            "filepath": {"type": "string"},
            "sheet_name": {"type": "string"}
          },
          "required": ["filepath", "sheet_name"]
        }
      }
    }
  ]
}
```

## 📋 实现计划

### Phase 1: 工具发现 (1-2天)
1. ✅ 检查后端事件处理器 - 已完成
2. ✅ 检查 Protobuf 定义 - 已完成
3. 🔨 实现工具缓存持久化
4. 🔨 更新 MCPSettingsBloc 支持工具列表
5. 🔨 创建工具列表 UI

### Phase 2: 工具调用 (1-2天)
1. 🔨 实现工具调用对话框
2. 🔨 实现参数输入表单（基于 inputSchema）
3. 🔨 实现结果展示组件
4. 🔨 添加调用历史记录

### Phase 3: 会话管理 (1天)
1. 🔨 实现 MCPSessionManager
2. 🔨 添加会话状态持久化
3. 🔨 实现自动重连机制

### Phase 4: SSE 支持 (1-2天)
1. 🔨 实现 SSE 客户端
2. 🔨 实现流式响应解析
3. 🔨 添加进度展示 UI

### Phase 5: AI 集成 (2-3天)
1. 🔨 实现工具转换为 Function Calling 格式
2. 🔨 实现 AI 工具选择逻辑
3. 🔨 实现自动工具调用
4. 🔨 实现工具调用链（多步骤）

## 🎯 近期目标

**今天（第1步）**: 实现工具发现和展示
1. 更新 MCPSettingsBloc 添加工具列表功能
2. 在服务器卡片中显示工具数量
3. 添加"查看工具"按钮
4. 创建工具列表对话框

**明天（第2步）**: 实现工具调用
1. 创建工具调用对话框
2. 基于 inputSchema 动态生成表单
3. 实现工具调用逻辑
4. 展示调用结果

## 🔍 技术细节

### 工具调用流程
```
1. 用户点击"连接服务器" 
   → ConnectMCPServer
   → 自动调用 GetMCPToolList
   → 保存工具到缓存

2. 用户查看工具列表
   → 从缓存读取
   → 显示工具卡片

3. 用户调用工具
   → 打开工具调用对话框
   → 输入参数
   → CallMCPTool
   → 显示结果

4. AI 自动调用
   → AI 选择工具
   → 构造参数
   → CallMCPTool
   → 处理结果
   → 继续对话
```

### 数据流
```
Flutter UI 
  ↓ (事件)
MCPSettingsBloc 
  ↓ (AIEvent)
Rust Backend 
  ↓ (MCP Protocol)
MCP Server 
  ↓ (结果)
Rust Backend 
  ↓ (ProtoBuf)
MCPSettingsBloc 
  ↓ (状态更新)
Flutter UI
```




