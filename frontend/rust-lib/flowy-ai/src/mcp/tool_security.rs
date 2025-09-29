use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::mcp::entities::{MCPTool, ToolSafetyLevel};

/// MCP工具安全管理器
/// 负责管理工具权限、安全策略和用户确认
pub struct ToolSecurityManager {
    store_preferences: Arc<KVStorePreferences>,
}

/// 工具安全策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSecurityPolicy {
    /// 是否允许自动执行只读工具
    pub auto_execute_read_only: bool,
    /// 是否允许自动执行安全工具
    pub auto_execute_safe: bool,
    /// 是否需要用户确认外部交互工具
    pub require_confirmation_external: bool,
    /// 是否需要用户确认破坏性工具
    pub require_confirmation_destructive: bool,
    /// 被禁用的工具列表
    pub disabled_tools: HashSet<String>,
    /// 被信任的工具列表（可以跳过某些安全检查）
    pub trusted_tools: HashSet<String>,
    /// 工具调用速率限制（每分钟最大调用次数）
    pub rate_limit_per_minute: HashMap<String, u32>,
}

impl Default for ToolSecurityPolicy {
    fn default() -> Self {
        Self {
            auto_execute_read_only: true,
            auto_execute_safe: false,
            require_confirmation_external: true,
            require_confirmation_destructive: true,
            disabled_tools: HashSet::new(),
            trusted_tools: HashSet::new(),
            rate_limit_per_minute: HashMap::new(),
        }
    }
}

/// 工具执行权限检查结果
#[derive(Debug, Clone, PartialEq)]
pub enum ToolExecutionPermission {
    /// 允许自动执行
    AutoExecute,
    /// 需要用户确认
    RequireConfirmation(String), // 确认消息
    /// 被禁止执行
    Denied(String), // 拒绝原因
}

/// 工具调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub server_id: String,
    pub timestamp: std::time::SystemTime,
    pub safety_level: String,
    pub user_confirmed: bool,
    pub execution_result: Option<String>,
}

const TOOL_SECURITY_POLICY_KEY: &str = "mcp_tool_security_policy";
const TOOL_CALL_RECORDS_KEY: &str = "mcp_tool_call_records";

impl ToolSecurityManager {
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        Self { store_preferences }
    }

    /// 获取工具安全策略
    pub fn get_security_policy(&self) -> ToolSecurityPolicy {
        self.store_preferences
            .get_object::<ToolSecurityPolicy>(TOOL_SECURITY_POLICY_KEY)
            .unwrap_or_default()
    }

    /// 保存工具安全策略
    pub fn save_security_policy(&self, policy: ToolSecurityPolicy) -> FlowyResult<()> {
        self.store_preferences
            .set_object(TOOL_SECURITY_POLICY_KEY, &policy)
            .map_err(|e| {
                FlowyError::internal().with_context(format!("保存工具安全策略失败: {}", e))
            })?;
        
        info!("Tool security policy saved successfully");
        Ok(())
    }

    /// 检查工具执行权限
    pub fn check_tool_permission(
        &self,
        tool: &MCPTool,
        _server_id: &str,
    ) -> ToolExecutionPermission {
        let policy = self.get_security_policy();
        
        // 检查工具是否被禁用
        if policy.disabled_tools.contains(&tool.name) {
            return ToolExecutionPermission::Denied(
                format!("工具 '{}' 已被管理员禁用", tool.name)
            );
        }

        // 检查工具是否在信任列表中
        if policy.trusted_tools.contains(&tool.name) {
            debug!("Tool '{}' is trusted, allowing auto execution", tool.name);
            return ToolExecutionPermission::AutoExecute;
        }

        // 根据安全级别检查权限
        match tool.safety_level() {
            ToolSafetyLevel::ReadOnly => {
                if policy.auto_execute_read_only {
                    ToolExecutionPermission::AutoExecute
                } else {
                    ToolExecutionPermission::RequireConfirmation(
                        format!("工具 '{}' 将执行只读操作，是否继续？", tool.display_title())
                    )
                }
            }
            ToolSafetyLevel::Safe => {
                if policy.auto_execute_safe {
                    ToolExecutionPermission::AutoExecute
                } else {
                    ToolExecutionPermission::RequireConfirmation(
                        format!("工具 '{}' 将执行安全操作，是否继续？", tool.display_title())
                    )
                }
            }
            ToolSafetyLevel::ExternalInteraction => {
                if policy.require_confirmation_external {
                    ToolExecutionPermission::RequireConfirmation(
                        format!(
                            "工具 '{}' 将与外部服务交互，这可能涉及网络请求或第三方API调用。是否继续？",
                            tool.display_title()
                        )
                    )
                } else {
                    ToolExecutionPermission::AutoExecute
                }
            }
            ToolSafetyLevel::Destructive => {
                if policy.require_confirmation_destructive {
                    ToolExecutionPermission::RequireConfirmation(
                        format!(
                            "⚠️ 警告：工具 '{}' 可能执行破坏性操作（如删除文件、修改系统设置等）。\n\
                            请仔细确认后再继续。是否执行？",
                            tool.display_title()
                        )
                    )
                } else {
                    ToolExecutionPermission::AutoExecute
                }
            }
        }
    }

    /// 检查工具调用速率限制
    pub fn check_rate_limit(&self, tool_name: &str) -> FlowyResult<bool> {
        let policy = self.get_security_policy();
        
        if let Some(&limit) = policy.rate_limit_per_minute.get(tool_name) {
            let records = self.get_recent_tool_calls(tool_name, 60)?; // 最近1分钟
            if records.len() >= limit as usize {
                warn!("Rate limit exceeded for tool: {} ({} calls in last minute)", 
                      tool_name, records.len());
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// 记录工具调用
    pub fn record_tool_call(&self, record: ToolCallRecord) -> FlowyResult<()> {
        let mut records = self.get_tool_call_records()?;
        records.push(record.clone());
        
        // 只保留最近1000条记录
        if records.len() > 1000 {
            let skip_count = records.len() - 1000;
            records = records.into_iter().skip(skip_count).collect();
        }
        
        self.store_preferences
            .set_object(TOOL_CALL_RECORDS_KEY, &records)
            .map_err(|e| {
                FlowyError::internal().with_context(format!("保存工具调用记录失败: {}", e))
            })?;
        
        debug!("Tool call recorded: {} on {}", record.tool_name, record.server_id);
        Ok(())
    }

    /// 获取工具调用记录
    pub fn get_tool_call_records(&self) -> FlowyResult<Vec<ToolCallRecord>> {
        Ok(self.store_preferences
            .get_object::<Vec<ToolCallRecord>>(TOOL_CALL_RECORDS_KEY)
            .unwrap_or_default())
    }

    /// 获取最近的工具调用记录
    pub fn get_recent_tool_calls(&self, tool_name: &str, seconds: u64) -> FlowyResult<Vec<ToolCallRecord>> {
        let records = self.get_tool_call_records()?;
        let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(seconds);
        
        let recent_records = records
            .into_iter()
            .filter(|record| {
                record.tool_name == tool_name && record.timestamp > cutoff_time
            })
            .collect();
        
        Ok(recent_records)
    }

    /// 添加工具到禁用列表
    pub fn disable_tool(&self, tool_name: String) -> FlowyResult<()> {
        let mut policy = self.get_security_policy();
        policy.disabled_tools.insert(tool_name.clone());
        self.save_security_policy(policy)?;
        
        info!("Tool '{}' has been disabled", tool_name);
        Ok(())
    }

    /// 从禁用列表移除工具
    pub fn enable_tool(&self, tool_name: &str) -> FlowyResult<()> {
        let mut policy = self.get_security_policy();
        policy.disabled_tools.remove(tool_name);
        self.save_security_policy(policy)?;
        
        info!("Tool '{}' has been enabled", tool_name);
        Ok(())
    }

    /// 添加工具到信任列表
    pub fn trust_tool(&self, tool_name: String) -> FlowyResult<()> {
        let mut policy = self.get_security_policy();
        policy.trusted_tools.insert(tool_name.clone());
        self.save_security_policy(policy)?;
        
        info!("Tool '{}' has been added to trusted list", tool_name);
        Ok(())
    }

    /// 从信任列表移除工具
    pub fn untrust_tool(&self, tool_name: &str) -> FlowyResult<()> {
        let mut policy = self.get_security_policy();
        policy.trusted_tools.remove(tool_name);
        self.save_security_policy(policy)?;
        
        info!("Tool '{}' has been removed from trusted list", tool_name);
        Ok(())
    }

    /// 设置工具速率限制
    pub fn set_tool_rate_limit(&self, tool_name: String, calls_per_minute: u32) -> FlowyResult<()> {
        let mut policy = self.get_security_policy();
        policy.rate_limit_per_minute.insert(tool_name.clone(), calls_per_minute);
        self.save_security_policy(policy)?;
        
        info!("Rate limit set for tool '{}': {} calls per minute", tool_name, calls_per_minute);
        Ok(())
    }

    /// 移除工具速率限制
    pub fn remove_tool_rate_limit(&self, tool_name: &str) -> FlowyResult<()> {
        let mut policy = self.get_security_policy();
        policy.rate_limit_per_minute.remove(tool_name);
        self.save_security_policy(policy)?;
        
        info!("Rate limit removed for tool '{}'", tool_name);
        Ok(())
    }

    /// 获取工具安全统计
    pub fn get_security_statistics(&self) -> FlowyResult<ToolSecurityStatistics> {
        let policy = self.get_security_policy();
        let records = self.get_tool_call_records()?;
        
        let total_calls = records.len();
        let confirmed_calls = records.iter().filter(|r| r.user_confirmed).count();
        let auto_executed_calls = total_calls - confirmed_calls;
        
        let mut safety_level_calls = HashMap::new();
        for record in &records {
            *safety_level_calls.entry(record.safety_level.clone()).or_insert(0) += 1;
        }
        
        Ok(ToolSecurityStatistics {
            disabled_tools_count: policy.disabled_tools.len(),
            trusted_tools_count: policy.trusted_tools.len(),
            rate_limited_tools_count: policy.rate_limit_per_minute.len(),
            total_tool_calls: total_calls,
            confirmed_calls,
            auto_executed_calls,
            safety_level_calls,
        })
    }

    /// 清理旧的工具调用记录
    pub fn cleanup_old_records(&self, days: u64) -> FlowyResult<usize> {
        let records = self.get_tool_call_records()?;
        let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 24 * 3600);
        
        let old_count = records.len();
        let filtered_records: Vec<ToolCallRecord> = records
            .into_iter()
            .filter(|record| record.timestamp > cutoff_time)
            .collect();
        
        let removed_count = old_count - filtered_records.len();
        
        if removed_count > 0 {
            self.store_preferences
                .set_object(TOOL_CALL_RECORDS_KEY, &filtered_records)
                .map_err(|e| {
                    FlowyError::internal().with_context(format!("清理工具调用记录失败: {}", e))
                })?;
            
            info!("Cleaned up {} old tool call records", removed_count);
        }
        
        Ok(removed_count)
    }
}

/// 工具安全统计信息
#[derive(Debug, Clone)]
pub struct ToolSecurityStatistics {
    pub disabled_tools_count: usize,
    pub trusted_tools_count: usize,
    pub rate_limited_tools_count: usize,
    pub total_tool_calls: usize,
    pub confirmed_calls: usize,
    pub auto_executed_calls: usize,
    pub safety_level_calls: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    use crate::mcp::entities::{MCPTool, MCPToolAnnotations};

    fn create_test_security_manager() -> (ToolSecurityManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        let manager = ToolSecurityManager::new(store);
        (manager, tempdir)
    }

    fn create_test_tool(name: &str, annotations: Option<MCPToolAnnotations>) -> MCPTool {
        MCPTool {
            name: name.to_string(),
            description: format!("Test tool: {}", name),
            input_schema: json!({"type": "object"}),
            annotations,
        }
    }

    #[test]
    fn test_security_policy_management() {
        let (manager, _tempdir) = create_test_security_manager();
        
        // 测试默认策略
        let default_policy = manager.get_security_policy();
        assert!(default_policy.auto_execute_read_only);
        assert!(!default_policy.auto_execute_safe);
        
        // 测试保存和读取策略
        let mut custom_policy = default_policy;
        custom_policy.auto_execute_safe = true;
        custom_policy.disabled_tools.insert("dangerous_tool".to_string());
        
        manager.save_security_policy(custom_policy.clone()).unwrap();
        let loaded_policy = manager.get_security_policy();
        
        assert!(loaded_policy.auto_execute_safe);
        assert!(loaded_policy.disabled_tools.contains("dangerous_tool"));
    }

    #[test]
    fn test_tool_permission_checking() {
        let (manager, _tempdir) = create_test_security_manager();
        
        // 测试只读工具
        let read_only_tool = create_test_tool("read_file", Some(MCPToolAnnotations::safe_tool()));
        let permission = manager.check_tool_permission(&read_only_tool, "server1");
        assert_eq!(permission, ToolExecutionPermission::AutoExecute);
        
        // 测试破坏性工具
        let destructive_tool = create_test_tool("delete_file", Some(MCPToolAnnotations::destructive_tool()));
        let permission = manager.check_tool_permission(&destructive_tool, "server1");
        assert!(matches!(permission, ToolExecutionPermission::RequireConfirmation(_)));
        
        // 测试禁用工具
        manager.disable_tool("delete_file".to_string()).unwrap();
        let permission = manager.check_tool_permission(&destructive_tool, "server1");
        assert!(matches!(permission, ToolExecutionPermission::Denied(_)));
    }

    #[test]
    fn test_tool_call_recording() {
        let (manager, _tempdir) = create_test_security_manager();
        
        let record = ToolCallRecord {
            tool_name: "test_tool".to_string(),
            server_id: "server1".to_string(),
            timestamp: std::time::SystemTime::now(),
            safety_level: "Safe".to_string(),
            user_confirmed: false,
            execution_result: Some("Success".to_string()),
        };
        
        manager.record_tool_call(record.clone()).unwrap();
        
        let records = manager.get_tool_call_records().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tool_name, "test_tool");
    }

    #[test]
    fn test_rate_limiting() {
        let (manager, _tempdir) = create_test_security_manager();
        
        // 设置速率限制
        manager.set_tool_rate_limit("limited_tool".to_string(), 2).unwrap();
        
        // 第一次调用应该通过
        assert!(manager.check_rate_limit("limited_tool").unwrap());
        
        // 记录调用
        let record = ToolCallRecord {
            tool_name: "limited_tool".to_string(),
            server_id: "server1".to_string(),
            timestamp: std::time::SystemTime::now(),
            safety_level: "Safe".to_string(),
            user_confirmed: false,
            execution_result: None,
        };
        
        manager.record_tool_call(record.clone()).unwrap();
        manager.record_tool_call(record.clone()).unwrap();
        
        // 第三次调用应该被限制
        assert!(!manager.check_rate_limit("limited_tool").unwrap());
    }
}
