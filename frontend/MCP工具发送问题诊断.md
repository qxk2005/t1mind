# MCP 工具只发送一个服务器的问题诊断

## 问题描述
- 全局设置中有 2 个已连接的 MCP 服务器，都发现了工具
- 在 AI 聊天中提问时，只有一个 MCP 服务器的工具被发送给 AI 模型

## 已完成的修复

### 1. 修复 HashMap 覆盖问题
**位置**: `rust-lib/flowy-ai/src/ai_manager.rs`

**问题**: 之前使用 `HashMap<String, MCPTool>` 存储工具，如果两个服务器有同名工具会被覆盖

**修复**: 改为 `Vec<(String, MCPTool)>` 存储，保留服务器ID信息

```rust
// 修复前
async fn discover_available_tools(&self) -> (Vec<String>, HashMap<String, MCPTool>)

// 修复后
async fn discover_available_tools(&self) -> Vec<(String, MCPTool)>
```

### 2. 添加详细日志
在关键位置添加了详细日志，帮助诊断问题：

1. **工具发现阶段** (`discover_available_tools`):
   - 显示扫描了多少个 MCP 服务器
   - 显示每个服务器的激活状态
   - 显示从每个服务器发现的工具数量和名称
   - 显示每个工具来自哪个服务器

2. **工具定义获取阶段** (`get_tool_definitions_by_names`):
   - 显示智能体配置中的工具列表
   - 显示实际转换的工具定义数量
   - 显示每个工具的名称、来源服务器和描述

## 可能的原因分析

### 原因1: 智能体配置中只保存了部分工具
**现象**: `config.available_tools` 列表中只包含一个服务器的工具

**检查方法**: 
查看日志中这行：
```
[Chat] 🔧 Available tools list: [...]
```

**可能的情况**:
- 创建智能体时，UI 只显示了一个服务器的工具供选择
- 自动发现工具时只发现了一个服务器的工具
- 工具列表在保存时被截断

**解决方案**: 
- 删除现有智能体，重新创建
- 或者通过 UI 手动添加缺失的工具

### 原因2: 只有一个服务器被标记为激活状态
**现象**: `discover_available_tools` 跳过了未激活的服务器

**检查方法**:
查看日志中这些行：
```
[Tool Discovery] 检查配置: xxx (ID: xxx, 激活: true/false)
[Tool Discovery] 跳过未激活的服务器: xxx
```

**解决方案**: 在全局设置中确保两个 MCP 服务器都是激活状态

### 原因3: 一个服务器连接失败
**现象**: 一个服务器无法获取工具列表

**检查方法**:
查看日志中是否有：
```
[Tool Discovery] 从服务器 'xxx' 获取工具列表失败: ... - 可能未连接
```

**解决方案**: 
- 检查该服务器的连接状态
- 尝试重新连接
- 检查服务器配置是否正确

### 原因4: 工具缓存问题
**现象**: 一个服务器的缓存是空的或过期的

**检查方法**:
查看日志中：
```
[Tool Discovery] 从服务器 'xxx' 的缓存中发现 N 个工具
```
vs
```
[Tool Discovery] 服务器 'xxx' 没有缓存，尝试从客户端获取...
```

**解决方案**: 
- 重启应用以重新发现工具
- 或者在 UI 中触发工具发现操作

## 诊断步骤

### 第1步: 查看工具发现日志
运行应用并在 AI 聊天中提问，查找以下日志：

```
[Tool Discovery] 开始扫描 N 个已配置的 MCP 服务器...
[Tool Discovery] 检查配置: SERVER1 (ID: xxx, 激活: xxx)
[Tool Discovery] 检查配置: SERVER2 (ID: xxx, 激活: xxx)
[Tool Discovery]   - 工具: TOOL1 (服务器: xxx)
[Tool Discovery]   - 工具: TOOL2 (服务器: xxx)
```

**关键检查点**:
- [ ] 扫描了 2 个服务器？
- [ ] 两个服务器都是激活状态？
- [ ] 两个服务器都发现了工具？
- [ ] 工具总数是否正确？

### 第2步: 查看智能体配置日志
继续查找：

```
[Chat] 🔧 Available tools list: [...]
[Chat] 🔧 Got N tool definitions for OpenAI Function Call
[Chat] 🔧   - Tool 'xxx' from server 'xxx': ...
```

**关键检查点**:
- [ ] `available_tools` 列表包含所有工具名称？
- [ ] 转换的工具定义数量正确？
- [ ] 每个工具的来源服务器正确？

### 第3步: 检查发送给 AI 的工具
查找：

```
[OpenAI] Added N tools to request
```

**关键检查点**:
- [ ] 发送的工具数量是否等于 `available_tools` 的数量？

## 如何获取完整日志

1. 关闭 AppFlowy
2. 重新启动 AppFlowy（从终端启动可以看到完整日志）
3. 进入 AI 聊天
4. 发送一条需要工具的问题
5. 查看日志输出

## 临时解决方案

如果问题出在智能体配置上，可以：

1. **方案A**: 删除并重新创建智能体
   - 删除当前智能体
   - 重新创建，让系统自动发现所有工具

2. **方案B**: 通过配置文件手动修改
   - 找到智能体配置文件
   - 手动添加缺失的工具名称到 `available_tools` 列表

3. **方案C**: 使用更新 API
   - 通过 UI 的智能体编辑界面
   - 手动勾选所有需要的工具

## 下一步

请运行应用，在 AI 聊天中提问，然后将日志发给我。重点关注：
1. `[Tool Discovery]` 开头的日志
2. `[Chat] 🔧` 开头的日志
3. 特别是 `Available tools list:` 这一行

这样我们就能准确定位问题所在。

