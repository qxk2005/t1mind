# MCP引用显示功能集成检查清单

## 概述

本文档提供了MCP引用显示功能的完整集成检查清单，帮助验证所有功能都已正确实现和集成。

## ✅ 完成状态

### 1. 核心组件 ✓

- [x] **MCPReferenceDisplay组件** (`mcp_reference_display.dart`)
  - [x] 基本显示功能
  - [x] 服务器.工具名称格式支持
  - [x] 展开/折叠功能
  - [x] 点击回调支持
  - [x] 主题适配（明暗模式）
  - [x] 响应式布局

### 2. 数据处理 ✓

- [x] **Metadata解析** (`chat_message_service.dart`)
  - [x] 识别tool_call类型的metadata
  - [x] 区分web_search和MCP工具
  - [x] 从tool_name解析服务器信息
  - [x] 创建正确格式的ChatMessageRefSource
  - [x] 使用"mcp:server_id"格式存储源信息

### 3. UI集成 ✓

- [x] **AI消息显示** (`ai_text_message.dart`)
  - [x] 导入MCPReferenceDisplay组件
  - [x] 添加_hasMCPReferences检测方法
  - [x] 添加_extractMCPReferences提取方法
  - [x] 在消息内容后正确位置显示组件
  - [x] 传递onSelectedMetadata回调

- [x] **消息点击处理** (`text_message_widget.dart`)
  - [x] 在_onSelectMetadata中添加MCP引用处理
  - [x] 正确识别source.startsWith("mcp")
  - [x] 添加基本的日志记录

### 4. 文档和示例 ✓

- [x] **使用文档**
  - [x] README_MCP_REFERENCE_DISPLAY.md - 组件使用指南
  - [x] AI聊天引用系统集成指南.md - 完整的集成文档
  - [x] MCP引用显示集成检查清单.md - 本文档

- [x] **代码示例**
  - [x] mcp_reference_display_example.dart - 详细示例
  - [x] 包含多种使用场景
  - [x] 提供快速测试页面

### 5. 代码质量 ✓

- [x] **Linter检查**
  - [x] 无linter错误
  - [x] 无未使用的导入
  - [x] 代码格式正确

- [x] **注释和文档**
  - [x] 所有公开API都有文档注释
  - [x] 关键逻辑有行内注释
  - [x] 参数说明完整

## 🧪 功能测试清单

### 基本显示测试

- [ ] **单个MCP引用显示**
  ```
  测试步骤:
  1. 调用MCP工具（如Excel读取）
  2. 查看AI回复消息
  3. 确认显示格式: [1] excel.read_data_from_excel ✓
  ```

- [ ] **多个MCP引用显示**
  ```
  测试步骤:
  1. 调用多个MCP工具（如Excel读取、创建图表）
  2. 查看AI回复消息
  3. 确认默认只显示前3个引用
  4. 点击"查看全部"按钮
  5. 确认显示所有引用
  ```

### 格式测试

- [ ] **服务器名称高亮**
  ```
  测试步骤:
  1. 查看任意MCP引用
  2. 确认服务器名称使用主色显示
  3. 确认与工具名称之间有点号分隔
  ```

- [ ] **未知服务器处理**
  ```
  测试步骤:
  1. 创建source为"mcp:unknown"的引用
  2. 确认只显示工具名称，不显示"unknown."前缀
  ```

### 交互测试

- [ ] **引用点击**
  ```
  测试步骤:
  1. 点击任意MCP引用
  2. 确认触发了回调（检查日志）
  3. 未来应显示工具详情弹窗
  ```

- [ ] **展开/折叠**
  ```
  测试步骤:
  1. 有超过3个MCP引用时
  2. 确认显示"查看全部 X 个工具"按钮
  3. 点击按钮展开
  4. 确认按钮变为"收起"
  5. 再次点击确认能收起
  ```

### 主题测试

- [ ] **明亮主题**
  ```
  测试步骤:
  1. 切换到明亮主题
  2. 查看MCP引用显示
  3. 确认背景、文字、边框颜色正确
  ```

- [ ] **暗色主题**
  ```
  测试步骤:
  1. 切换到暗色主题
  2. 查看MCP引用显示
  3. 确认背景、文字、边框颜色正确
  ```

### 混合引用测试

- [ ] **与网络搜索引用共存**
  ```
  测试步骤:
  1. 同时使用web_search和MCP工具
  2. 确认两种引用分别显示
  3. 网络搜索使用CitationDisplay
  4. MCP工具使用MCPReferenceDisplay
  ```

- [ ] **与文档检索引用共存**
  ```
  测试步骤:
  1. 启用文档检索并调用MCP工具
  2. 确认两种引用分别显示
  3. 文档检索使用AIMessageMetadata
  4. MCP工具使用MCPReferenceDisplay
  ```

## 🔧 集成验证

### 前端代码检查

```bash
# 1. 检查导入
grep -r "mcp_reference_display" appflowy_flutter/lib/plugins/ai_chat/

# 2. 检查source格式处理
grep -r "mcp:" appflowy_flutter/lib/plugins/ai_chat/

# 3. 运行linter
cd appflowy_flutter
flutter analyze
```

### 后端metadata格式检查

检查Rust后端发送的metadata是否包含正确的tool_call信息：

```json
{
  "tool_call": {
    "id": "call_xxx",
    "tool_name": "read_data_from_excel",  // 或 "excel.read_data_from_excel"
    "status": "success",
    "result": "..."
  }
}
```

## 📊 性能检查

- [ ] **渲染性能**
  ```
  测试场景:
  - 10个MCP引用: 应流畅渲染
  - 50个MCP引用: 展开/折叠应流畅
  - 100个MCP引用: 应考虑虚拟滚动
  ```

- [ ] **内存使用**
  ```
  测试场景:
  - 长时间使用不应有内存泄漏
  - 多次展开/折叠不应增加内存
  ```

## 🐛 已知限制

1. **服务器名称识别**
   - 当前依赖于tool_name包含"server.tool"格式
   - 如果后端只发送工具名称，会显示为"unknown"
   - **解决方案**: 后端在metadata中添加server_id字段

2. **点击交互**
   - 目前点击只记录日志
   - 需要实现工具详情弹窗
   - **计划**: 在下个版本实现

3. **国际化**
   - 组件内文本未完全国际化
   - **计划**: 添加到locale_keys

## 🚀 后续开发计划

### 短期（1-2周）

- [ ] 实现工具详情弹窗
  - 显示工具参数
  - 显示执行时间
  - 显示返回值预览

- [ ] 添加工具状态动画
  - pending状态：加载动画
  - success状态：成功动画
  - failed状态：错误提示

### 中期（1个月）

- [ ] 工具结果预览
  - 在引用卡片中显示摘要
  - 支持展开查看完整结果

- [ ] 工具分组显示
  - 按服务器分组
  - 显示统计信息

### 长期（3个月）

- [ ] 高级交互
  - 工具调用历史记录
  - 工具收藏功能
  - 工具执行重放

- [ ] 性能优化
  - 虚拟滚动支持
  - 大量引用时的优化

## 📝 集成步骤回顾

如果需要在新项目中集成此功能，按以下步骤操作：

1. **复制组件文件**
   - `mcp_reference_display.dart`
   - `mcp_reference_display_example.dart`（可选）

2. **修改metadata解析**
   - 在`chat_message_service.dart`中添加MCP工具识别逻辑

3. **集成到消息显示**
   - 在`ai_text_message.dart`中添加显示逻辑

4. **处理点击事件**
   - 在`text_message_widget.dart`中添加点击处理

5. **测试验证**
   - 使用本检查清单验证所有功能

## 📞 支持

如有问题，请参考：
- `README_MCP_REFERENCE_DISPLAY.md` - 详细使用指南
- `AI聊天引用系统集成指南.md` - 完整系统架构
- `mcp_reference_display_example.dart` - 代码示例

## 更新日志

- **2025-01-11**: 初始版本，完成基本功能集成

