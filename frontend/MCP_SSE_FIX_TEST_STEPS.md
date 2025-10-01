# MCP SSE 修复测试步骤

## 🔧 修复内容

已修复SSE MCP客户端的两个关键问题：
1. ✅ **initialize方法** - 使用POST请求并发送MCP initialize消息
2. ✅ **list_tools方法** - 实现真正的工具列表获取

## 🚀 快速测试步骤

### 准备工作

#### 1. 启动Excel MCP服务器
```bash
# 方法1：使用uvx（推荐）
FASTMCP_PORT=8007 uvx excel-mcp-server streamable-http

# 方法2：使用npx
FASTMCP_PORT=8007 npx @modelcontextprotocol/server-excel streamable-http
```

**预期输出**：
```
🚀 Excel MCP Server starting...
📡 Listening on http://localhost:8007
✅ SSE endpoint: http://localhost:8007/mcp
```

#### 2. 验证服务器运行
```bash
# 使用curl测试
curl -X POST http://localhost:8007/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{
    "jsonrpc": "2.0",
    "id": 0,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "Test", "version": "1.0"}
    }
  }'
```

**预期返回**: 200 OK 或初始化响应

### AppFlowy 测试

#### 步骤1：编译Rust后端
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend/rust-lib/flowy-ai
cargo build --release
```

#### 步骤2：配置MCP服务器

1. 打开AppFlowy
2. 进入 **设置 → 工作空间 → MCP配置**
3. 点击 **"添加服务器"**
4. 填写配置：
   - **名称**: `Excel MCP`
   - **传输类型**: `SSE`
   - **URL**: `http://localhost:8007/mcp`
   - **描述**: `Excel文件操作服务器`

#### 步骤3：一键检查
1. 点击 **"一键检查"** 按钮
2. 观察服务器卡片

**预期结果**：
- ✅ 绿色圆点（已连接）
- ✅ 显示工具数量徽章（如 "🔧 18"）
- ✅ 底部显示工具标签

#### 步骤4：验证工具标签
1. 查看服务器卡片底部
2. 应该显示类似：
   ```
   🔧 read_data_from_excel  🔧 write_data_to_excel
   🔧 apply_formula  🔧 format_range  +14
   ```

3. 鼠标悬停在任意工具标签上
4. 应该显示工具描述Tooltip

## 📊 预期日志输出

### Rust后端日志

**成功的连接日志**：
```
INFO  flowy_ai::mcp::client_pool: Creating MCP client for server: Excel MCP (SSE)
INFO  flowy_ai::mcp::client: SSE MCP client initialized for: Excel MCP (status: 200)
INFO  flowy_ai::mcp::client: SSE MCP client found 18 tools for: Excel MCP
```

**失败的日志（如果服务器未运行）**：
```
WARN  flowy_ai::mcp::client_pool: Failed to initialize client for Excel MCP: ...
ERROR flowy_ai::mcp::event_handler: Failed to connect to MCP server Excel MCP: ...
```

### Flutter前端日志

**成功连接**：
```
INFO | MCP服务器连接结果: mcp_xxx, 连接状态: true
INFO | 接收到MCP服务器状态: mcp_xxx, 连接: true
INFO | 获取到MCP工具列表: 18 个工具
```

## 🐛 故障排查

### 问题1：仍然显示406错误

**可能原因**：代码没有重新编译

**解决方案**：
```bash
# 清理并重新编译
cd rust-lib/flowy-ai
cargo clean
cargo build --release

# 重启AppFlowy
```

### 问题2：无法连接到服务器

**检查清单**：
- [ ] Excel MCP服务器是否在运行？
- [ ] 端口8007是否被占用？
- [ ] URL是否正确：`http://localhost:8007/mcp`
- [ ] 传输类型是否选择了SSE？

### 问题3：工具列表为空

**可能原因**：
- list_tools请求失败
- 响应格式解析错误

**调试步骤**：
```bash
# 查看详细日志
RUST_LOG=debug cargo run

# 手动测试tools/list
curl -X POST http://localhost:8007/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### 问题4：工具标签不显示

**检查清单**：
- [ ] 服务器是否已连接？（绿色圆点）
- [ ] 工具数量徽章是否显示？
- [ ] 前端代码是否已重新运行？

## 📝 测试检查清单

### 基本功能测试
- [ ] SSE服务器可以成功连接
- [ ] 工具列表可以正确获取
- [ ] 工具数量徽章正确显示
- [ ] 工具标签正确显示（最多5个+more）
- [ ] 鼠标悬停显示工具描述

### 边界情况测试
- [ ] 服务器未启动时显示错误
- [ ] 网络断开时正确处理
- [ ] 重连机制正常工作
- [ ] 多个SSE服务器可以同时工作

### 性能测试
- [ ] 连接时间 < 3秒
- [ ] 工具列表加载 < 2秒
- [ ] UI响应流畅，无卡顿

## 🎯 成功标准

以下所有条件都满足即为成功：

1. ✅ SSE MCP服务器连接成功（状态码200）
2. ✅ 工具列表获取成功（18个工具）
3. ✅ 前端显示绿色连接状态
4. ✅ 工具标签正确显示
5. ✅ Tooltip悬停显示正常
6. ✅ 无错误日志

## 📚 相关文档

- [MCP_SSE_CLIENT_FIX.md](./MCP_SSE_CLIENT_FIX.md) - 详细修复说明
- [MCP_AUTO_DETECT_QUICK_TEST.md](./MCP_AUTO_DETECT_QUICK_TEST.md) - UI功能测试
- [test_excel_mcp.rs](./test_excel_mcp.rs) - MCP测试脚本

## 📅 测试记录

**日期**: 2025-10-01  
**测试人**: _____________  
**测试环境**: _____________  

### 测试结果
- [ ] ✅ 通过
- [ ] ❌ 失败
- [ ] ⏳ 待测试

**备注**: 
_________________________________________________
_________________________________________________
_________________________________________________

