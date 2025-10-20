use std::sync::Arc;
use std::time::SystemTime;

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{
    AgentConfigPB, AgentCapabilitiesPB, AgentStatusPB, CreateAgentRequestPB, 
    UpdateAgentRequestPB, DeleteAgentRequestPB, GetAgentRequestPB, AgentListPB
};

/// 智能体配置管理器的键前缀
const AGENT_CONFIG_PREFIX: &str = "agent_config";
const AGENT_LIST_KEY: &str = "agent_list";
const AGENT_GLOBAL_SETTINGS_KEY: &str = "agent_global_settings";
const AGENT_VERSION_KEY: &str = "agent_config_version";

/// 当前配置版本
const CURRENT_CONFIG_VERSION: u32 = 1;

/// 智能体全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGlobalSettings {
    /// 是否启用智能体功能
    pub enabled: bool,
    /// 默认最大规划步骤数
    pub default_max_planning_steps: i32,
    /// 默认最大工具调用次数
    pub default_max_tool_calls: i32,
    /// 默认会话记忆长度限制
    pub default_memory_limit: i32,
    /// 是否启用调试日志
    pub debug_logging: bool,
    /// 智能体执行超时时间（秒）
    pub execution_timeout: u64,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

impl Default for AgentGlobalSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            default_max_planning_steps: 10,
            default_max_tool_calls: 20,
            default_memory_limit: 100,
            debug_logging: false,
            execution_timeout: 300, // 5分钟
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

/// 智能体配置管理器
pub struct AgentConfigManager {
    store_preferences: Arc<KVStorePreferences>,
}

impl AgentConfigManager {
    /// 创建新的智能体配置管理器
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        let manager = Self { store_preferences };
        
        // 检查并执行配置迁移
        if let Err(e) = manager.migrate_config_if_needed() {
            error!("Failed to migrate agent config: {}", e);
        }
        
        manager
    }

    /// 获取全局智能体设置
    pub fn get_global_settings(&self) -> AgentGlobalSettings {
        self.store_preferences
            .get_object::<AgentGlobalSettings>(AGENT_GLOBAL_SETTINGS_KEY)
            .unwrap_or_default()
    }

    /// 保存全局智能体设置
    pub fn save_global_settings(&self, mut settings: AgentGlobalSettings) -> FlowyResult<()> {
        settings.updated_at = SystemTime::now();
        
        self.store_preferences
            .set_object(AGENT_GLOBAL_SETTINGS_KEY, &settings)
            .map_err(|e| {
                error!("Failed to save agent global settings: {}", e);
                FlowyError::internal().with_context(format!("保存智能体全局设置失败: {}", e))
            })?;
        
        info!("Agent global settings saved successfully");
        Ok(())
    }

    /// 创建智能体配置
    pub fn create_agent(&self, request: CreateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
        // 验证请求
        self.validate_create_request(&request)?;
        
        // 生成唯一ID
        let agent_id = self.generate_agent_id();
        let now = Utc::now().timestamp();
        
        // 自动填充可用工具（如果为空且启用了工具调用）
        let mut available_tools = request.available_tools;
        if available_tools.is_empty() && request.capabilities.enable_tool_calling {
            available_tools = self.get_default_tools();
        }
        
        // 创建智能体配置
        let mut agent_config = AgentConfigPB {
            id: agent_id.clone(),
            name: request.name,
            description: request.description,
            avatar: request.avatar,
            personality: request.personality,
            capabilities: request.capabilities,
            available_tools,
            status: AgentStatusPB::AgentActive,
            selected_mcp_servers: request.selected_mcp_servers,  // 🆕 保存选中的服务器
            created_at: now,
            updated_at: now,
            metadata: request.metadata,
        };

        // 应用默认设置
        self.apply_default_capabilities(&mut agent_config.capabilities);
        
        // 保存配置
        self.save_agent_config(&agent_config)?;
        
        info!("Agent created successfully: {} ({})", agent_config.name, agent_config.id);
        Ok(agent_config)
    }

    /// 获取智能体配置
    pub fn get_agent(&self, request: GetAgentRequestPB) -> FlowyResult<AgentConfigPB> {
        let agent_config = self.get_agent_config(&request.id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("智能体配置不存在"))?;
        
        Ok(agent_config)
    }

    /// 更新智能体配置
    pub fn update_agent(&self, request: UpdateAgentRequestPB) -> FlowyResult<AgentConfigPB> {
        // 验证请求
        self.validate_update_request(&request)?;
        
        // 获取现有配置
        let mut agent_config = self.get_agent_config(&request.id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("智能体配置不存在"))?;
        
        // 更新字段
        if let Some(name) = request.name {
            if !name.is_empty() {
                agent_config.name = name;
            }
        }
        
        if let Some(description) = request.description {
            agent_config.description = description;
        }
        
        if let Some(avatar) = request.avatar {
            agent_config.avatar = avatar;
        }
        
        if let Some(personality) = request.personality {
            agent_config.personality = personality;
        }
        
        if let Some(capabilities) = request.capabilities {
            agent_config.capabilities = capabilities;
        }
        
        if !request.available_tools.is_empty() {
            agent_config.available_tools = request.available_tools;
        } else if agent_config.available_tools.is_empty() && agent_config.capabilities.enable_tool_calling {
            // 为现有智能体自动填充默认工具
            agent_config.available_tools = self.get_default_tools();
        }
        
        if let Some(status) = request.status {
            agent_config.status = status;
        }
        
        if !request.metadata.is_empty() {
            agent_config.metadata.extend(request.metadata);
        }
        
        // 🆕 更新已选择的 MCP 服务器列表
        // 注意：即使列表为空也要更新，因为用户可能取消了所有选择
        agent_config.selected_mcp_servers = request.selected_mcp_servers;
        
        // 更新时间戳
        agent_config.updated_at = Utc::now().timestamp();
        
        // 应用默认设置
        self.apply_default_capabilities(&mut agent_config.capabilities);
        
        // 保存配置
        self.save_agent_config(&agent_config)?;
        
        info!("Agent updated successfully: {} ({})", agent_config.name, agent_config.id);
        info!("🔧 [Agent Config] Updated capabilities:");
        info!("🔧 [Agent Config]   enable_reflection: {}", agent_config.capabilities.enable_reflection);
        info!("🔧 [Agent Config]   max_reflection_iterations: {}", agent_config.capabilities.max_reflection_iterations);
        info!("🔧 [Agent Config]   enable_tool_calling: {}", agent_config.capabilities.enable_tool_calling);
        info!("🔧 [Agent Config]   max_tool_calls: {}", agent_config.capabilities.max_tool_calls);
        Ok(agent_config)
    }

    /// 删除智能体配置
    pub fn delete_agent(&self, request: DeleteAgentRequestPB) -> FlowyResult<()> {
        // 检查智能体是否存在
        if !self.agent_exists(&request.id) {
            return Err(FlowyError::record_not_found().with_context("智能体配置不存在"));
        }
        
        // 删除智能体配置
        let key = self.agent_config_key(&request.id);
        self.store_preferences.remove(&key);
        
        // 从智能体列表中移除
        self.update_agent_list(&request.id, false)?;
        
        info!("Agent deleted successfully: {}", request.id);
        Ok(())
    }

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

    /// 获取活跃的智能体列表
    pub fn get_active_agents(&self) -> FlowyResult<AgentListPB> {
        let all_agents = self.get_all_agents()?;
        let active_agents = all_agents.agents
            .into_iter()
            .filter(|agent| agent.status == AgentStatusPB::AgentActive)
            .collect();
        
        Ok(AgentListPB { agents: active_agents })
    }

    /// 更新智能体状态
    pub fn update_agent_status(&self, agent_id: &str, status: AgentStatusPB) -> FlowyResult<()> {
        let mut agent_config = self.get_agent_config(agent_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("智能体配置不存在"))?;
        
        agent_config.status = status;
        agent_config.updated_at = Utc::now().timestamp();
        
        self.save_agent_config(&agent_config)?;
        info!("Agent {} status updated to: {:?}", agent_id, status);
        Ok(())
    }

    /// 检查智能体ID是否已存在
    pub fn agent_exists(&self, agent_id: &str) -> bool {
        self.get_agent_config(agent_id).is_some()
    }

    /// 生成唯一的智能体ID
    pub fn generate_agent_id(&self) -> String {
        loop {
            let id = Uuid::new_v4().to_string();
            if !self.agent_exists(&id) {
                return id;
            }
        }
    }

    /// 导出所有智能体配置
    pub fn export_config(&self) -> FlowyResult<AgentConfigExport> {
        let global_settings = self.get_global_settings();
        let agents = self.get_all_agents()?.agents;
        
        Ok(AgentConfigExport {
            version: CURRENT_CONFIG_VERSION,
            exported_at: SystemTime::now(),
            global_settings,
            agents,
        })
    }

    /// 导入智能体配置
    pub fn import_config(&self, config: AgentConfigExport) -> FlowyResult<AgentImportResult> {
        let mut result = AgentImportResult::default();
        
        // 检查版本兼容性
        if config.version > CURRENT_CONFIG_VERSION {
            return Err(FlowyError::invalid_data()
                .with_context("配置版本不兼容，请升级应用程序"));
        }
        
        // 导入全局设置
        if let Err(e) = self.save_global_settings(config.global_settings) {
            result.errors.push(format!("导入全局设置失败: {}", e));
        } else {
            result.global_settings_imported = true;
        }
        
        // 导入智能体配置
        for agent in config.agents {
            match self.save_agent_config(&agent) {
                Ok(_) => {
                    result.agents_imported += 1;
                    result.imported_agent_ids.push(agent.id.clone());
                }
                Err(e) => {
                    result.errors.push(format!("导入智能体 {} 失败: {}", agent.name, e));
                }
            }
        }
        
        info!("Agent config import completed: {} agents imported, {} errors", 
              result.agents_imported, result.errors.len());
        Ok(result)
    }

    /// 清理所有配置（危险操作）
    pub fn clear_all_config(&self) -> FlowyResult<()> {
        warn!("Clearing all agent configuration data");
        
        // 获取所有智能体ID并删除
        let agent_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(AGENT_LIST_KEY)
            .unwrap_or_default();
        
        for agent_id in agent_ids {
            let key = self.agent_config_key(&agent_id);
            self.store_preferences.remove(&key);
        }
        
        // 清理智能体列表、全局设置和版本信息
        self.store_preferences.remove(AGENT_LIST_KEY);
        self.store_preferences.remove(AGENT_GLOBAL_SETTINGS_KEY);
        self.store_preferences.remove(AGENT_VERSION_KEY);
        
        info!("All agent configuration data cleared");
        Ok(())
    }

    /// 获取智能体配置
    pub fn get_agent_config(&self, agent_id: &str) -> Option<AgentConfigPB> {
        let key = self.agent_config_key(agent_id);
        self.store_preferences.get_object::<AgentConfigPB>(&key)
    }

    /// 保存智能体配置（内部方法）
    fn save_agent_config(&self, config: &AgentConfigPB) -> FlowyResult<()> {
        // 验证配置
        self.validate_agent_config_internal(config)?;
        
        // 保存智能体配置
        let key = self.agent_config_key(&config.id);
        self.store_preferences
            .set_object(&key, config)
            .map_err(|e| {
                error!("Failed to save agent config {}: {}", config.id, e);
                FlowyError::internal().with_context(format!("保存智能体配置失败: {}", e))
            })?;

        // 更新智能体列表
        self.update_agent_list(&config.id, true)?;
        
        Ok(())
    }

    /// 验证创建请求
    fn validate_create_request(&self, request: &CreateAgentRequestPB) -> FlowyResult<()> {
        if request.name.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("智能体名称不能为空"));
        }
        
        // 验证能力配置
        self.validate_capabilities(&request.capabilities)?;
        
        Ok(())
    }

    /// 验证更新请求
    fn validate_update_request(&self, request: &UpdateAgentRequestPB) -> FlowyResult<()> {
        if request.id.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("智能体ID不能为空"));
        }
        
        if let Some(ref name) = request.name {
            if name.trim().is_empty() {
                return Err(FlowyError::invalid_data().with_context("智能体名称不能为空"));
            }
        }
        
        if let Some(ref capabilities) = request.capabilities {
            self.validate_capabilities(capabilities)?;
        }
        
        Ok(())
    }

    /// 验证智能体配置（内部方法）
    fn validate_agent_config_internal(&self, config: &AgentConfigPB) -> FlowyResult<()> {
        if config.id.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("智能体ID不能为空"));
        }
        
        if config.name.trim().is_empty() {
            return Err(FlowyError::invalid_data().with_context("智能体名称不能为空"));
        }
        
        // 验证能力配置
        self.validate_capabilities(&config.capabilities)?;
        
        Ok(())
    }

    /// 验证智能体配置并返回错误列表
    pub fn validate_agent_config(&self, config: &AgentConfigPB) -> FlowyResult<Vec<String>> {
        let mut errors = Vec::new();
        
        if config.id.trim().is_empty() {
            errors.push("智能体ID不能为空".to_string());
        }
        
        if config.name.trim().is_empty() {
            errors.push("智能体名称不能为空".to_string());
        }
        
        if config.name.len() > 50 {
            errors.push("智能体名称不能超过50个字符".to_string());
        }
        
        if config.description.len() > 500 {
            errors.push("智能体描述不能超过500个字符".to_string());
        }
        
        if config.personality.len() > 2000 {
            errors.push("个性描述不能超过2000个字符".to_string());
        }
        
        // 验证能力配置
        let capabilities = &config.capabilities;
        if capabilities.max_planning_steps < 1 || capabilities.max_planning_steps > 100 {
            errors.push("最大规划步骤数必须在1-100之间".to_string());
        }
        
        if capabilities.max_tool_calls < 1 || capabilities.max_tool_calls > 1000 {
            errors.push("最大工具调用次数必须在1-1000之间".to_string());
        }
        
        if capabilities.memory_limit < 10 || capabilities.memory_limit > 10000 {
            errors.push("会话记忆长度限制必须在10-10000之间".to_string());
        }
        
        // 注意：工具列表验证已移除
        // 工具现在是从 MCP 服务器动态发现的，在创建智能体时可以为空
        // 系统会在首次使用时自动从已配置的 MCP 服务器加载可用工具
        
        Ok(errors)
    }

    /// 验证能力配置
    fn validate_capabilities(&self, capabilities: &AgentCapabilitiesPB) -> FlowyResult<()> {
        if capabilities.max_planning_steps < 1 || capabilities.max_planning_steps > 100 {
            return Err(FlowyError::invalid_data()
                .with_context("最大规划步骤数必须在1-100之间"));
        }
        
        if capabilities.max_tool_calls < 1 || capabilities.max_tool_calls > 1000 {
            return Err(FlowyError::invalid_data()
                .with_context("最大工具调用次数必须在1-1000之间"));
        }
        
        if capabilities.memory_limit < 10 || capabilities.memory_limit > 10000 {
            return Err(FlowyError::invalid_data()
                .with_context("会话记忆长度限制必须在10-10000之间"));
        }
        
        Ok(())
    }

    /// 应用默认能力配置
    fn apply_default_capabilities(&self, capabilities: &mut AgentCapabilitiesPB) {
        let global_settings = self.get_global_settings();
        
        if capabilities.max_planning_steps <= 0 {
            capabilities.max_planning_steps = global_settings.default_max_planning_steps;
        }
        
        if capabilities.max_tool_calls <= 0 {
            capabilities.max_tool_calls = global_settings.default_max_tool_calls;
        }
        
        if capabilities.memory_limit <= 0 {
            capabilities.memory_limit = global_settings.default_memory_limit;
        }
    }
    
    /// 从 MCP 服务器动态获取默认工具列表
    /// 
    /// 此方法需要访问 MCP 管理器，因此需要异步调用
    /// 为了保持向后兼容，我们提供同步版本返回内置工具
    fn get_default_tools(&self) -> Vec<String> {
        // 🆕 返回内置工具列表，包括网络搜索工具
        let tools = vec![
            "search_documents".to_string(),
            "create_document".to_string(),
            "update_document".to_string(),
            "delete_document".to_string(),
        ];
        
        // 🆕 添加内置网络搜索工具
        #[cfg(feature = "web-search")]
        {
            let mut tools = tools;
            tools.extend(vec![
                "web_search".to_string(),
                // "quick_search".to_string(), // 已注释以避免重复调用
            ]);
            tools
        }
        #[cfg(not(feature = "web-search"))]
        {
            tools
        }
    }
    
    /// 为现有智能体自动填充工具（如果工具列表为空）
    pub fn auto_populate_agent_tools(&self, agent_id: &str) -> FlowyResult<bool> {
        if let Some(mut config) = self.get_agent_config(agent_id) {
            if config.available_tools.is_empty() && config.capabilities.enable_tool_calling {
                config.available_tools = self.get_default_tools();
                config.updated_at = Utc::now().timestamp();
                
                self.save_agent_config(&config)?;
                
                info!("为智能体 {} 自动填充了 {} 个默认工具", 
                      config.name, config.available_tools.len());
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// 更新智能体列表
    fn update_agent_list(&self, agent_id: &str, add: bool) -> FlowyResult<()> {
        let mut agent_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(AGENT_LIST_KEY)
            .unwrap_or_default();

        if add {
            if !agent_ids.contains(&agent_id.to_string()) {
                agent_ids.push(agent_id.to_string());
            }
        } else {
            agent_ids.retain(|id| id != agent_id);
        }

        self.store_preferences
            .set_object(AGENT_LIST_KEY, &agent_ids)
            .map_err(|e| {
                error!("Failed to update agent list: {}", e);
                FlowyError::internal().with_context(format!("更新智能体列表失败: {}", e))
            })?;

        Ok(())
    }

    /// 生成智能体配置的存储键
    fn agent_config_key(&self, agent_id: &str) -> String {
        format!("{}:agent:{}", AGENT_CONFIG_PREFIX, agent_id)
    }

    /// 检查并执行配置迁移
    fn migrate_config_if_needed(&self) -> FlowyResult<()> {
        let current_version = self.store_preferences
            .get_object::<u32>(AGENT_VERSION_KEY)
            .unwrap_or(0);
        
        if current_version < CURRENT_CONFIG_VERSION {
            info!("Migrating agent config from version {} to {}", 
                  current_version, CURRENT_CONFIG_VERSION);
            
            // 这里可以添加版本迁移逻辑
            // 目前只是更新版本号
            self.store_preferences
                .set_object(AGENT_VERSION_KEY, &CURRENT_CONFIG_VERSION)
                .map_err(|e| {
                    error!("Failed to update agent config version: {}", e);
                    FlowyError::internal().with_context("更新配置版本失败")
                })?;
            
            info!("Agent config migration completed");
        }
        
        Ok(())
    }
}

/// 配置导出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigExport {
    pub version: u32,
    pub exported_at: SystemTime,
    pub global_settings: AgentGlobalSettings,
    pub agents: Vec<AgentConfigPB>,
}

/// 配置导入结果
#[derive(Debug, Clone, Default)]
pub struct AgentImportResult {
    pub global_settings_imported: bool,
    pub agents_imported: usize,
    pub imported_agent_ids: Vec<String>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use flowy_sqlite::kv::KVStorePreferences;
    use tempfile::TempDir;

    fn create_test_config_manager() -> (AgentConfigManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        let manager = AgentConfigManager::new(store);
        (manager, tempdir)
    }

    fn create_test_agent_request() -> CreateAgentRequestPB {
        CreateAgentRequestPB {
            name: "测试智能体".to_string(),
            description: "这是一个测试智能体".to_string(),
            avatar: "🤖".to_string(),
            personality: "你是一个友好的助手".to_string(),
            capabilities: AgentCapabilitiesPB {
                enable_planning: true,
                enable_tool_calling: true,
                enable_reflection: true,
                enable_memory: true,
                max_planning_steps: 10,
                max_tool_calls: 20,
                memory_limit: 100,
                max_tool_result_length: 4000,
                max_reflection_iterations: 3,
            },
            available_tools: vec!["search".to_string(), "calculator".to_string()],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_global_settings() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试默认设置
        let default_settings = manager.get_global_settings();
        assert!(default_settings.enabled);
        assert_eq!(default_settings.default_max_planning_steps, 10);
        
        // 测试保存和读取设置
        let mut custom_settings = default_settings.clone();
        custom_settings.enabled = false;
        custom_settings.default_max_planning_steps = 20;
        
        manager.save_global_settings(custom_settings.clone()).unwrap();
        let loaded_settings = manager.get_global_settings();
        
        assert!(!loaded_settings.enabled);
        assert_eq!(loaded_settings.default_max_planning_steps, 20);
    }

    #[test]
    fn test_agent_crud_operations() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试创建智能体
        let request = create_test_agent_request();
        let agent = manager.create_agent(request).unwrap();
        assert_eq!(agent.name, "测试智能体");
        assert_eq!(agent.status, AgentStatusPB::AgentActive);
        
        // 测试获取智能体
        let get_request = GetAgentRequestPB { id: agent.id.clone() };
        let retrieved_agent = manager.get_agent(get_request).unwrap();
        assert_eq!(retrieved_agent.id, agent.id);
        assert_eq!(retrieved_agent.name, agent.name);
        
        // 测试更新智能体
        let update_request = UpdateAgentRequestPB {
            id: agent.id.clone(),
            name: Some("更新后的智能体".to_string()),
            description: Some("更新后的描述".to_string()),
            avatar: None,
            personality: None,
            capabilities: None,
            available_tools: vec![],
            status: Some(AgentStatusPB::AgentPaused),
            metadata: HashMap::new(),
            selected_mcp_servers: vec![],  // 🆕 添加字段
        };
        let updated_agent = manager.update_agent(update_request).unwrap();
        assert_eq!(updated_agent.name, "更新后的智能体");
        assert_eq!(updated_agent.status, AgentStatusPB::AgentPaused);
        
        // 测试删除智能体
        let delete_request = DeleteAgentRequestPB { id: agent.id.clone() };
        manager.delete_agent(delete_request).unwrap();
        
        let get_request = GetAgentRequestPB { id: agent.id };
        assert!(manager.get_agent(get_request).is_err());
    }

    #[test]
    fn test_agent_list_operations() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 创建多个智能体
        let request1 = create_test_agent_request();
        let agent1 = manager.create_agent(request1).unwrap();
        
        let mut request2 = create_test_agent_request();
        request2.name = "第二个智能体".to_string();
        let agent2 = manager.create_agent(request2).unwrap();
        
        // 测试获取所有智能体
        let all_agents = manager.get_all_agents().unwrap();
        assert_eq!(all_agents.agents.len(), 2);
        
        // 暂停一个智能体
        manager.update_agent_status(&agent1.id, AgentStatusPB::AgentPaused).unwrap();
        
        // 测试获取活跃智能体
        let active_agents = manager.get_active_agents().unwrap();
        assert_eq!(active_agents.agents.len(), 1);
        assert_eq!(active_agents.agents[0].id, agent2.id);
    }

    #[test]
    fn test_validation() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试空名称验证
        let mut invalid_request = create_test_agent_request();
        invalid_request.name = "".to_string();
        assert!(manager.create_agent(invalid_request).is_err());
        
        // 测试无效的能力配置
        let mut invalid_request = create_test_agent_request();
        invalid_request.capabilities.max_planning_steps = 0;
        assert!(manager.create_agent(invalid_request).is_err());
        
        invalid_request = create_test_agent_request();
        invalid_request.capabilities.max_tool_calls = 2000;
        assert!(manager.create_agent(invalid_request).is_err());
    }

    #[test]
    fn test_config_export_import() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 创建测试数据
        let request = create_test_agent_request();
        manager.create_agent(request).unwrap();
        
        // 测试导出
        let export = manager.export_config().unwrap();
        assert_eq!(export.agents.len(), 1);
        assert_eq!(export.version, CURRENT_CONFIG_VERSION);
        
        // 清理配置
        manager.clear_all_config().unwrap();
        let all_agents = manager.get_all_agents().unwrap();
        assert_eq!(all_agents.agents.len(), 0);
        
        // 测试导入
        let import_result = manager.import_config(export).unwrap();
        assert!(import_result.global_settings_imported);
        assert_eq!(import_result.agents_imported, 1);
        
        let all_agents = manager.get_all_agents().unwrap();
        assert_eq!(all_agents.agents.len(), 1);
    }
}
