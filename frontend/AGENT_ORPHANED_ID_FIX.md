# 智能体孤立 ID 自动清理修复

## 问题描述

用户在全局设置的智能体配置中无法添加新的智能体。日志显示：

```
WARN flowy_ai::agent::config_manager: Agent config not found for ID: fbe524fc-5fb4-470e-bb0b-c9c98d058860
INFO flowy_ai::agent::event_handler: ✅ Successfully retrieved 0 agents
```

## 问题原因

系统中存在一个**孤立的智能体 ID**（`fbe524fc-5fb4-470e-bb0b-c9c98d058860`）：
- 该 ID 存在于智能体列表（`agent_list`）中
- 但对应的智能体配置不存在

这种情况可能由以下原因导致：
1. 之前创建智能体时部分失败（配置未保存但 ID 已添加到列表）
2. 手动删除或清理了配置文件但未更新列表
3. 存储层操作失败导致数据不一致

## 修复方案

### 修改文件
- `rust-lib/flowy-ai/src/agent/config_manager.rs`

### 修改内容

在 `get_all_agents()` 方法中添加自动清理孤立 ID 的逻辑：

```rust
/// 获取所有智能体配置
pub fn get_all_agents(&self) -> FlowyResult<AgentListPB> {
    let agent_ids: Vec<String> = self.store_preferences
        .get_object::<Vec<String>>(AGENT_LIST_KEY)
        .unwrap_or_default();

    let mut agents = Vec::new();
    let mut orphaned_ids = Vec::new();
    
    for agent_id in agent_ids {
        if let Some(agent) = self.get_agent_config(&agent_id) {
            agents.push(agent);
        } else {
            warn!("Agent config not found for ID: {}, will clean up", agent_id);
            orphaned_ids.push(agent_id);
        }
    }
    
    // 自动清理孤立的 agent ID
    if !orphaned_ids.is_empty() {
        warn!("Cleaning up {} orphaned agent IDs", orphaned_ids.len());
        for orphaned_id in orphaned_ids {
            if let Err(e) = self.update_agent_list(&orphaned_id, false) {
                error!("Failed to clean up orphaned agent ID {}: {}", orphaned_id, e);
            }
        }
    }
    
    debug!("Retrieved {} agent configurations", agents.len());
    Ok(AgentListPB { agents })
}
```

## 修复效果

### 修复前
1. 系统启动时加载智能体列表
2. 发现孤立的 ID，记录警告日志
3. 跳过该 ID，但保留在列表中
4. 每次加载都重复警告
5. 可能影响添加新智能体的功能

### 修复后
1. 系统启动时加载智能体列表
2. 发现孤立的 ID，记录警告日志
3. **自动从列表中删除孤立的 ID**
4. 清理完成，系统恢复正常
5. 可以正常添加新的智能体

## 测试步骤

1. **重新编译项目**：
   ```bash
   cd rust-lib/flowy-ai && cargo check
   ```

2. **重启应用**：
   - 关闭当前应用
   - 重新启动应用

3. **观察日志**：
   - 应该看到 "Cleaning up 1 orphaned agent IDs" 的日志
   - 之后不再出现该警告

4. **验证功能**：
   - 进入全局设置的智能体配置页面
   - 尝试添加新的智能体
   - 应该能够成功添加

## 预期日志输出

### 首次修复运行
```
WARN flowy_ai::agent::config_manager: Agent config not found for ID: fbe524fc-5fb4-470e-bb0b-c9c98d058860, will clean up
WARN flowy_ai::agent::config_manager: Cleaning up 1 orphaned agent IDs
INFO flowy_ai::agent::event_handler: ✅ Successfully retrieved 0 agents
```

### 后续运行（清理完成后）
```
INFO flowy_ai::agent::event_handler: ✅ Successfully retrieved 0 agents
```
（不再有警告）

## 技术细节

### 孤立 ID 检测
- 在加载智能体列表时，遍历所有 ID
- 调用 `get_agent_config()` 检查配置是否存在
- 如果配置不存在，记录为孤立 ID

### 自动清理逻辑
- 收集所有孤立的 ID
- 调用 `update_agent_list(id, false)` 从列表中移除
- 记录清理日志，便于诊断

### 错误处理
- 清理失败不影响整体流程
- 记录错误日志以便排查
- 下次运行时会重新尝试清理

## 相关代码

### 配置存储结构
```rust
// 智能体列表键
const AGENT_LIST_KEY: &str = "agent_list";

// 单个智能体配置键（格式）
fn agent_config_key(&self, agent_id: &str) -> String {
    format!("{}:agent:{}", AGENT_CONFIG_PREFIX, agent_id)
}
```

### update_agent_list 方法
```rust
fn update_agent_list(&self, agent_id: &str, add: bool) -> FlowyResult<()> {
    let mut agent_ids: Vec<String> = self.store_preferences
        .get_object::<Vec<String>>(AGENT_LIST_KEY)
        .unwrap_or_default();

    if add {
        if !agent_ids.contains(&agent_id.to_string()) {
            agent_ids.push(agent_id.to_string());
        }
    } else {
        agent_ids.retain(|id| id != agent_id);  // 删除指定 ID
    }

    self.store_preferences
        .set_object(AGENT_LIST_KEY, &agent_ids)
        .map_err(|e| {
            error!("Failed to update agent list: {}", e);
            FlowyError::internal().with_context(format!("更新智能体列表失败: {}", e))
        })?;

    Ok(())
}
```

## 预防措施

为了避免未来再次出现孤立 ID：

1. **事务性操作**：确保配置保存和列表更新是原子操作
2. **错误恢复**：在 `save_agent_config` 失败时，应该回滚 ID 添加
3. **删除逻辑**：确保 `delete_agent` 同时删除配置和列表项

## 编译状态

✅ 编译成功，无错误
⚠️ 6 个警告（与本次修改无关，为已存在的未使用字段警告）

## 总结

此修复通过在 `get_all_agents()` 方法中添加自动清理逻辑，解决了孤立智能体 ID 导致的系统问题。修复后系统能够：
1. 自动检测孤立的 ID
2. 自动从列表中删除它们
3. 恢复正常的智能体管理功能
4. 防止此类问题影响用户体验

这是一个**自我修复（Self-healing）**的设计，确保系统在遇到数据不一致时能够自动恢复。

