# Excel MCP服务器测试指南

本指南说明如何测试AppFlowy的MCP数据结构与Excel MCP服务器的兼容性。

## 前提条件

1. **安装Excel MCP服务器**
   ```bash
   # 安装uvx（如果还没有安装）
   pip install uvx
   
   # 安装excel-mcp-server
   uvx install excel-mcp-server
   ```

2. **安装rust-script**（用于运行独立测试脚本）
   ```bash
   cargo install rust-script
   ```

## 测试方法

### 方法1：使用自动化测试脚本

1. **启动Excel MCP服务器**
   ```bash
   excelfile FASTMCP_PORT=8007 uvx excel-mcp-server streamable-http
   ```

2. **运行测试脚本**
   ```bash
   cd /Users/niuzhidao/Documents/Program/t1mind/frontend
   ./run_excel_mcp_test.sh
   ```

   或指定不同端口：
   ```bash
   ./run_excel_mcp_test.sh 8008
   ```

### 方法2：使用独立Rust脚本

1. **启动Excel MCP服务器**
   ```bash
   excelfile FASTMCP_PORT=8007 uvx excel-mcp-server streamable-http
   ```

2. **运行独立测试脚本**
   ```bash
   cd /Users/niuzhidao/Documents/Program/t1mind/frontend
   FASTMCP_PORT=8007 rust-script test_excel_mcp.rs
   ```

### 方法3：运行Rust单元测试

```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend/rust-lib/flowy-ai

# 运行MCP集成测试
cargo test mcp::integration_test --lib -- --nocapture

# 运行所有MCP测试
cargo test mcp --lib -- --nocapture
```

## 测试内容

### 1. 连接测试
- 验证能否连接到Excel MCP服务器
- 检查服务器响应状态

### 2. MCP协议测试
- **初始化会话**：发送`initialize`请求，验证协议握手
- **工具发现**：发送`tools/list`请求，获取可用工具列表
- **工具调用**：发送`tools/call`请求，测试工具执行

### 3. 数据结构兼容性测试
- **MCPTool结构**：验证工具定义的序列化/反序列化
- **MCPToolAnnotations**：验证工具注解的处理
- **安全级别分类**：验证工具安全级别的自动分类
- **权限检查**：验证工具执行权限的检查机制

### 4. 协议消息测试
- **请求消息**：验证JSON-RPC 2.0请求格式
- **响应消息**：验证响应和错误处理
- **通知消息**：验证通知消息格式

## 预期结果

成功的测试应该显示：

```
🧪 Excel MCP服务器测试
============================================
📡 目标端口: 8007
🔗 测试Excel MCP服务器连接 (http://localhost:8007)
✅ 服务器响应状态: 200 OK
🚀 初始化MCP会话...
✅ 会话初始化成功
📋 获取工具列表...
✅ 发现工具: read_excel - Read data from an Excel file
✅ 发现工具: write_excel - Write data to an Excel file
✅ 总共发现 X 个工具
🔍 验证AppFlowy MCP数据结构兼容性...
✅ 所有数据结构验证通过!
🔧 测试工具调用...
✅ 工具调用测试成功

============================================
🎉 Excel MCP测试完成!
✅ 连接测试: 通过
✅ 会话初始化: 通过
✅ 工具发现: X 个工具
✅ 数据结构兼容性: 完全兼容
✅ AppFlowy MCP实现: 准备就绪
```

## 故障排除

### 1. 连接失败
```
❌ 无法连接到Excel MCP服务器
```
**解决方案**：
- 检查Excel MCP服务器是否正在运行
- 确认端口号是否正确（默认8007）
- 检查防火墙设置

### 2. 会话初始化失败
```
❌ MCP会话初始化失败: Protocol version mismatch
```
**解决方案**：
- 更新excel-mcp-server到最新版本
- 检查协议版本兼容性

### 3. 工具调用失败
```
⚠️ 工具调用测试失败: Missing required parameter
```
**解决方案**：
- 这是正常的，因为测试使用的是模拟参数
- 实际使用时需要提供正确的参数

## 支持的Excel MCP工具

根据测试，AppFlowy MCP实现支持以下类型的Excel工具：

### 只读工具（自动执行）
- `read_excel` - 读取Excel文件
- `get_sheet_names` - 获取工作表名称
- `get_cell_value` - 获取单元格值

### 写入工具（需要确认）
- `write_excel` - 写入Excel文件
- `create_sheet` - 创建新工作表
- `delete_sheet` - 删除工作表

### 图表工具（安全操作）
- `create_chart` - 创建图表
- `update_chart` - 更新图表

## 安全特性

AppFlowy MCP实现包含以下安全特性：

1. **工具分类**：根据MCP注解自动分类工具安全级别
2. **权限检查**：破坏性操作需要用户确认
3. **速率限制**：防止工具被过度调用
4. **审计日志**：记录所有工具调用
5. **配置管理**：支持工具的启用/禁用

## 下一步

测试通过后，您可以：

1. 在AppFlowy中配置Excel MCP服务器
2. 使用AI助手调用Excel工具
3. 根据需要调整安全策略
4. 添加更多MCP服务器

## 技术细节

### MCP协议版本
- 支持MCP协议版本：`2024-11-05`
- JSON-RPC版本：`2.0`

### 传输方式
- HTTP/HTTPS
- Server-Sent Events (SSE)
- STDIO（计划支持）

### 数据格式
- 请求/响应：JSON格式
- 工具参数：JSON Schema验证
- 错误处理：标准JSON-RPC错误格式
