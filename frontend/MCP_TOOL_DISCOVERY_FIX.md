# MCP 工具发现修复 - 从配置管理器查询

## 问题根因

### 发现的问题
在更新智能体时，工具发现日志显示：
```
[Tool Discovery] 开始扫描 0 个 MCP 服务器...
```

即使后台已经配置了 MCP 服务器，工具发现也找不到任何服务器。

### 根本原因

原始代码使用 `self.mcp_manager.list_servers()` 来获取服务器列表，但这个方法有重要限制：

```rust
// ❌ 原始代码 - 只查询客户端池
let servers = self.mcp_manager.list_servers().await;

// list_servers() 的实现：
pub async fn list_servers(&self) -> Vec<MCPClientInfo> {
    self.client_pool.get_all_clients_info().await  // 只返回已在 pool 中的客户端
}
```

**关键问题**：
1. `list_servers()` 只返回 **client_pool** 中已存在的客户端
2. 客户端只有在 **主动调用 `connect_server()`** 时才会被添加到 pool
3. 如果服务器只是 **配置了** 但 **没有连接**，就不会出现在结果中

**触发场景**：
- 用户在设置中添加了 MCP 服务器配置
- 配置已保存到数据库（通过 `MCPConfigManager`）
- 但没有立即建立连接（没有调用 `connect_server()`）
- 此时 `list_servers()` 返回空列表

## 解决方案

### 修复策略

从 **配置管理器** 查询所有已配置的服务器，而不是从客户端池：

```rust
// ✅ 修复后的代码
let server_configs = self.mcp_manager.config_manager().get_all_servers();
```

### 完整的工具发现逻辑

```rust
async fn discover_available_tools(&self) -> Vec<String> {
    let mut tool_names = Vec::new();
    
    // 🔍 关键修复：从配置管理器获取所有已配置的服务器
    let server_configs = self.mcp_manager.config_manager().get_all_servers();
    let config_count = server_configs.len();
    
    info!("[Tool Discovery] 开始扫描 {} 个已配置的 MCP 服务器...", config_count);
    
    if server_configs.is_empty() {
      info!("[Tool Discovery] 未找到任何已配置的 MCP 服务器");
      return tool_names;
    }
    
    // 遍历所有已配置且活跃的服务器
    for config in server_configs {
      info!("[Tool Discovery] 检查配置: {} (ID: {}, 激活: {})", 
            config.name, config.id, config.is_active);
      
      // 跳过未激活的服务器
      if !config.is_active {
        info!("[Tool Discovery] 跳过未激活的服务器: {}", config.name);
        continue;
      }
      
      // 优先使用缓存的工具列表（避免重复连接）
      if let Some(cached_tools) = &config.cached_tools {
        let tool_count = cached_tools.len();
        info!("[Tool Discovery] 从服务器 '{}' 的缓存中发现 {} 个工具", 
              config.name, tool_count);
        
        for tool in cached_tools {
          tool_names.push(tool.name.clone());
        }
        continue;
      }
      
      // 如果没有缓存，尝试从已连接的客户端获取
      info!("[Tool Discovery] 服务器 '{}' 没有缓存，尝试从客户端获取...", 
            config.name);
      match self.mcp_manager.tool_list(&config.id).await {
        Ok(tools_list) => {
          let tool_count = tools_list.tools.len();
          if tool_count > 0 {
            info!("[Tool Discovery] 从服务器 '{}' 的客户端获取到 {} 个工具", 
                  config.name, tool_count);
            for tool in tools_list.tools {
              tool_names.push(tool.name);
            }
          } else {
            warn!("[Tool Discovery] 服务器 '{}' 已激活但未返回任何工具", 
                  config.name);
          }
        }
        Err(e) => {
          warn!("[Tool Discovery] 从服务器 '{}' 获取工具列表失败: {} - 可能未连接", 
                config.name, e);
        }
      }
    }
    
    info!("✅ [Tool Discovery] 共从 {} 个已配置服务器发现 {} 个可用工具", 
          config_count, tool_names.len());
    tool_names
}
```

### 工具获取策略（三级回退）

1. **优先级 1：使用缓存的工具列表** ✅ 最快
   - 从 `config.cached_tools` 读取
   - 优点：无需网络请求，即时可用
   - 适用：服务器之前成功连接过

2. **优先级 2：从已连接的客户端获取** ⚡
   - 调用 `self.mcp_manager.tool_list(&config.id)`
   - 优点：获取最新的工具列表
   - 适用：服务器当前已连接

3. **优先级 3：记录警告但不中断** ⚠️
   - 如果服务器未连接或获取失败
   - 记录警告日志，继续处理其他服务器
   - 保证工具发现的鲁棒性

## 关键修复点对比

### 修复前 ❌

```rust
// 只查询已连接的客户端
let servers = self.mcp_manager.list_servers().await;
// servers.len() = 0  ← 即使配置了服务器
```

**问题**：
- 依赖客户端池的实时连接状态
- 如果服务器未连接，无法发现其工具
- 即使有缓存也无法利用

### 修复后 ✅

```rust
// 从配置管理器查询所有已配置的服务器
let server_configs = self.mcp_manager.config_manager().get_all_servers();
// server_configs.len() = 1  ← 正确返回配置的服务器

// 优先使用缓存
if let Some(cached_tools) = &config.cached_tools {
    // 直接使用缓存，无需连接
}
```

**优点**：
- 查询持久化的配置，不依赖临时连接状态
- 充分利用工具缓存机制
- 即使服务器暂时未连接，也能使用历史工具列表

## 预期的日志输出

### 修复后的成功日志

```
🔄 [Agent Update] 开始更新智能体: fbe524fc-5fb4-470e-bb0b-c9c98d058860
🔄 [Agent Update] 请求工具列表长度: 0
🔄 [Agent Update] 请求是否包含 capabilities: true
🔄 [Agent Update] 现有智能体: 段子高手
🔄 [Agent Update] 现有工具列表长度: 1
🔄 [Agent Update] 现有 enable_tool_calling: false
🔄 [Agent Update] 新能力配置 - enable_tool_calling: true
🔄 [Agent Update] 条件满足：工具调用已启用且工具列表为空
🔄 [Agent Update] 是否需要发现工具: true
✨ [Agent Update] 检测到工具调用能力变更或工具列表为空，开始自动发现工具...

[Tool Discovery] 开始扫描 1 个已配置的 MCP 服务器...  ← ✅ 找到配置
[Tool Discovery] 检查配置: excel-mcp (ID: xxx, 激活: true)
[Tool Discovery] 从服务器 'excel-mcp' 的缓存中发现 20 个工具  ← ✅ 使用缓存
✅ [Tool Discovery] 共从 1 个已配置服务器发现 20 个可用工具

✅ [Agent Update] 为智能体 '段子高手' 自动发现了 20 个工具
Agent updated successfully: 段子高手 (fbe524fc-5fb4-470e-bb0b-c9c98d058860)
🔄 [Agent Update] 更新完成
✅ Successfully updated agent: 段子高手 (fbe524fc-5fb4-470e-bb0b-c9c98d058860)
```

### 不同场景的日志

**场景 1：服务器有缓存**
```
[Tool Discovery] 从服务器 'excel-mcp' 的缓存中发现 20 个工具
```

**场景 2：服务器无缓存但已连接**
```
[Tool Discovery] 服务器 'excel-mcp' 没有缓存，尝试从客户端获取...
[Tool Discovery] 从服务器 'excel-mcp' 的客户端获取到 20 个工具
```

**场景 3：服务器未连接**
```
[Tool Discovery] 服务器 'excel-mcp' 没有缓存，尝试从客户端获取...
⚠️  [Tool Discovery] 从服务器 'excel-mcp' 获取工具列表失败: Client not found - 可能未连接
```

**场景 4：服务器未激活**
```
[Tool Discovery] 检查配置: old-server (ID: xxx, 激活: false)
[Tool Discovery] 跳过未激活的服务器: old-server
```

## 测试步骤

### 1. 重新编译
```bash
cd /Users/niuzhidao/Documents/Program/t1mind/frontend
cargo build --manifest-path rust-lib/Cargo.toml
```

### 2. 重启应用
重新启动 Flutter 应用以加载新的 Rust 库。

### 3. 更新智能体
1. 打开智能体设置（"段子高手"）
2. 将"工具调用"开关打开
3. 点击"保存"

### 4. 验证日志
应该看到：
```
[Tool Discovery] 开始扫描 1 个已配置的 MCP 服务器...  ← ✅ 不再是 0
[Tool Discovery] 从服务器 'xxx' 的缓存中发现 N 个工具  ← ✅ 找到工具
✅ [Tool Discovery] 共从 1 个已配置服务器发现 N 个可用工具  ← ✅ 成功
```

### 5. 使用智能体测试
创建聊天，选择"段子高手"，发送消息：
```
查看 excel 文件 myfile.xlsx 的内容有什么
```

AI 应该能够：
1. 识别需要使用工具 ✅
2. 调用正确的 MCP 工具 ✅
3. 返回文件内容 ✅

## 架构说明

### MCP 服务器的两种状态

1. **配置状态**（持久化）
   - 存储位置：SQLite 数据库（通过 `KVStorePreferences`）
   - 管理器：`MCPConfigManager`
   - 包含：服务器配置、工具缓存、最后检查时间等
   - 生命周期：永久，直到用户删除

2. **连接状态**（临时）
   - 存储位置：内存中的 `MCPClientPool`
   - 管理器：`MCPClientManager`
   - 包含：活跃的客户端连接、实时状态
   - 生命周期：应用运行期间，重启后需要重新连接

### 正确的查询策略

| 需求 | 查询源 | 方法 |
|------|--------|------|
| 获取所有配置的服务器 | 配置管理器 | `config_manager().get_all_servers()` ✅ |
| 获取已连接的服务器 | 客户端池 | `list_servers()` |
| 获取工具列表（优先缓存） | 配置 + 客户端 | 先查 `cached_tools`，再查 `tool_list()` ✅ |
| 实时调用工具 | 客户端池 | `call_tool()` |

## 相关代码位置

- **修复文件**：`rust-lib/flowy-ai/src/ai_manager.rs`
  - 方法：`discover_available_tools()`（第 1155-1214 行）

- **配置管理器**：`rust-lib/flowy-ai/src/mcp/config.rs`
  - 方法：`MCPConfigManager::get_all_servers()`

- **客户端池**：`rust-lib/flowy-ai/src/mcp/client_pool.rs`
  - 方法：`MCPClientPool::get_all_clients_info()`

- **MCP 管理器**：`rust-lib/flowy-ai/src/mcp/manager.rs`
  - 方法：`MCPClientManager::list_servers()`

## 总结

这个修复解决了工具发现无法找到已配置服务器的根本问题：

1. **问题**：只查询客户端池，漏掉了未连接的服务器
2. **修复**：改为查询配置管理器，获取所有已配置的服务器
3. **优化**：优先使用缓存，避免不必要的连接尝试
4. **鲁棒**：即使部分服务器不可用，也能发现其他服务器的工具

现在工具发现机制能够正确识别所有已配置的 MCP 服务器，并充分利用缓存机制，提供更快、更可靠的工具发现体验。

