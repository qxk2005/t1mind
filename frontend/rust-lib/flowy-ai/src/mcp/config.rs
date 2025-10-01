use std::sync::Arc;
use std::time::SystemTime;

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::mcp::entities::{MCPServerConfig, MCPTransportType};

/// MCP配置管理器的键前缀
const MCP_CONFIG_PREFIX: &str = "mcp_config";
const MCP_SERVER_LIST_KEY: &str = "mcp_server_list";
const MCP_GLOBAL_SETTINGS_KEY: &str = "mcp_global_settings";

/// MCP全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPGlobalSettings {
    /// 是否启用MCP功能
    pub enabled: bool,
    /// 默认连接超时时间（秒）
    pub connection_timeout: u64,
    /// 工具调用超时时间（秒）
    pub tool_call_timeout: u64,
    /// 最大并发连接数
    pub max_concurrent_connections: u32,
    /// 是否启用调试日志
    pub debug_logging: bool,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

impl Default for MCPGlobalSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            connection_timeout: 30,
            tool_call_timeout: 60,
            max_concurrent_connections: 10,
            debug_logging: false,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

/// MCP配置管理器
pub struct MCPConfigManager {
    store_preferences: Arc<KVStorePreferences>,
}

impl MCPConfigManager {
    /// 创建新的MCP配置管理器
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        Self { store_preferences }
    }

    /// 获取全局MCP设置
    pub fn get_global_settings(&self) -> MCPGlobalSettings {
        self.store_preferences
            .get_object::<MCPGlobalSettings>(MCP_GLOBAL_SETTINGS_KEY)
            .unwrap_or_default()
    }

    /// 保存全局MCP设置
    pub fn save_global_settings(&self, mut settings: MCPGlobalSettings) -> FlowyResult<()> {
        settings.updated_at = SystemTime::now();
        
        self.store_preferences
            .set_object(MCP_GLOBAL_SETTINGS_KEY, &settings)
            .map_err(|e| {
                error!("Failed to save MCP global settings: {}", e);
                FlowyError::internal().with_context(format!("保存MCP全局设置失败: {}", e))
            })?;
        
        info!("MCP global settings saved successfully");
        Ok(())
    }

    /// 获取所有MCP服务器配置
    pub fn get_all_servers(&self) -> Vec<MCPServerConfig> {
        let server_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(MCP_SERVER_LIST_KEY)
            .unwrap_or_default();

        let mut servers = Vec::new();
        for server_id in server_ids {
            if let Some(server) = self.get_server(&server_id) {
                servers.push(server);
            } else {
                warn!("Server config not found for ID: {}", server_id);
            }
        }
        
        debug!("Retrieved {} MCP server configurations", servers.len());
        servers
    }

    /// 获取单个MCP服务器配置
    pub fn get_server(&self, server_id: &str) -> Option<MCPServerConfig> {
        let key = self.server_config_key(server_id);
        let config = self.store_preferences.get_object::<MCPServerConfig>(&key);
        
        if let Some(ref cfg) = config {
            debug!("Loaded server config: {} (id: {})", cfg.name, cfg.id);
            
            // 调试：打印加载的配置 JSON
            if let Ok(json_str) = serde_json::to_string_pretty(cfg) {
                debug!("Loaded server config JSON:\n{}", json_str);
            }
            
            if let Some(ref tools) = cfg.cached_tools {
                info!("✅ Server {} has {} cached tools", cfg.name, tools.len());
                for (i, tool) in tools.iter().take(3).enumerate() {
                    debug!("  Tool {}: {}", i + 1, tool.name);
                }
                if tools.len() > 3 {
                    debug!("  ... and {} more tools", tools.len() - 3);
                }
            } else {
                warn!("⚠️  Server {} has no cached tools", cfg.name);
            }
            if let Some(check_time) = cfg.last_tools_check_at {
                if let Ok(duration) = check_time.duration_since(std::time::UNIX_EPOCH) {
                    info!("✅ Server {} last check time: {} seconds since epoch", cfg.name, duration.as_secs());
                }
            } else {
                warn!("⚠️  Server {} has no last check time", cfg.name);
            }
        } else {
            warn!("❌ Failed to load server config for ID: {}", server_id);
        }
        
        config
    }

    /// 保存MCP服务器配置
    pub fn save_server(&self, mut config: MCPServerConfig) -> FlowyResult<()> {
        // 验证配置
        self.validate_server_config(&config)?;
        
        // 更新时间戳
        config.updated_at = SystemTime::now();
        
        // 调试：序列化配置并打印
        if let Ok(json_str) = serde_json::to_string_pretty(&config) {
            debug!("Saving server config JSON:\n{}", json_str);
        }
        
        // 保存服务器配置
        let key = self.server_config_key(&config.id);
        self.store_preferences
            .set_object(&key, &config)
            .map_err(|e| {
                error!("Failed to save MCP server config {}: {}", config.id, e);
                FlowyError::internal().with_context(format!("保存MCP服务器配置失败: {}", e))
            })?;

        // 更新服务器列表
        self.update_server_list(&config.id, true)?;
        
        info!("MCP server config saved: {} ({})", config.name, config.id);
        Ok(())
    }

    /// 删除MCP服务器配置
    pub fn delete_server(&self, server_id: &str) -> FlowyResult<()> {
        // 删除服务器配置
        let key = self.server_config_key(server_id);
        self.store_preferences.remove(&key);
        
        // 从服务器列表中移除
        self.update_server_list(server_id, false)?;
        
        info!("MCP server config deleted: {}", server_id);
        Ok(())
    }

    /// 更新服务器激活状态
    pub fn update_server_active_status(&self, server_id: &str, is_active: bool) -> FlowyResult<()> {
        let mut config = self.get_server(server_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("MCP服务器配置不存在"))?;
        
        config.is_active = is_active;
        config.updated_at = SystemTime::now();
        
        self.save_server(config)?;
        info!("MCP server {} active status updated to: {}", server_id, is_active);
        Ok(())
    }
    
    /// 保存MCP服务器的工具缓存
    pub fn save_tools_cache(&self, server_id: &str, tools: Vec<crate::mcp::entities::MCPTool>) -> FlowyResult<()> {
        info!("Saving {} tools to cache for server: {}", tools.len(), server_id);
        
        let mut config = self.get_server(server_id)
            .ok_or_else(|| FlowyError::record_not_found().with_context("MCP服务器配置不存在"))?;
        
        config.cached_tools = Some(tools.clone());
        config.last_tools_check_at = Some(SystemTime::now());
        config.updated_at = SystemTime::now();
        
        info!("Saving server config with {} cached tools", tools.len());
        self.save_server(config)?;
        info!("✅ MCP server {} tools cache successfully saved with {} tools", server_id, tools.len());
        Ok(())
    }
    
    /// 获取MCP服务器的缓存工具
    pub fn get_cached_tools(&self, server_id: &str) -> Option<(Vec<crate::mcp::entities::MCPTool>, SystemTime)> {
        let config = self.get_server(server_id)?;
        config.cached_tools.zip(config.last_tools_check_at)
    }

    /// 获取激活的服务器列表
    pub fn get_active_servers(&self) -> Vec<MCPServerConfig> {
        self.get_all_servers()
            .into_iter()
            .filter(|server| server.is_active)
            .collect()
    }

    /// 根据传输类型获取服务器列表
    pub fn get_servers_by_transport(&self, transport_type: MCPTransportType) -> Vec<MCPServerConfig> {
        self.get_all_servers()
            .into_iter()
            .filter(|server| server.transport_type == transport_type)
            .collect()
    }

    /// 检查服务器ID是否已存在
    pub fn server_exists(&self, server_id: &str) -> bool {
        self.get_server(server_id).is_some()
    }

    /// 生成唯一的服务器ID
    pub fn generate_server_id(&self) -> String {
        loop {
            let id = Uuid::new_v4().to_string();
            if !self.server_exists(&id) {
                return id;
            }
        }
    }

    /// 导出所有配置
    pub fn export_config(&self) -> FlowyResult<MCPConfigExport> {
        let global_settings = self.get_global_settings();
        let servers = self.get_all_servers();
        
        Ok(MCPConfigExport {
            version: "1.0".to_string(),
            exported_at: SystemTime::now(),
            global_settings,
            servers,
        })
    }

    /// 导入配置
    pub fn import_config(&self, config: MCPConfigExport) -> FlowyResult<MCPImportResult> {
        let mut result = MCPImportResult::default();
        
        // 导入全局设置
        if let Err(e) = self.save_global_settings(config.global_settings) {
            result.errors.push(format!("导入全局设置失败: {}", e));
        } else {
            result.global_settings_imported = true;
        }
        
        // 导入服务器配置
        for server in config.servers {
            match self.save_server(server.clone()) {
                Ok(_) => {
                    result.servers_imported += 1;
                    result.imported_server_ids.push(server.id);
                }
                Err(e) => {
                    result.errors.push(format!("导入服务器 {} 失败: {}", server.name, e));
                }
            }
        }
        
        info!("MCP config import completed: {} servers imported, {} errors", 
              result.servers_imported, result.errors.len());
        Ok(result)
    }

    /// 清理所有配置（危险操作）
    pub fn clear_all_config(&self) -> FlowyResult<()> {
        warn!("Clearing all MCP configuration data");
        
        // 获取所有服务器ID并删除
        let server_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(MCP_SERVER_LIST_KEY)
            .unwrap_or_default();
        
        for server_id in server_ids {
            let key = self.server_config_key(&server_id);
            self.store_preferences.remove(&key);
        }
        
        // 清理服务器列表和全局设置
        self.store_preferences.remove(MCP_SERVER_LIST_KEY);
        self.store_preferences.remove(MCP_GLOBAL_SETTINGS_KEY);
        
        info!("All MCP configuration data cleared");
        Ok(())
    }

    /// 验证服务器配置
    fn validate_server_config(&self, config: &MCPServerConfig) -> FlowyResult<()> {
        // 基本字段验证
        if config.id.is_empty() {
            return Err(FlowyError::invalid_data().with_context("服务器ID不能为空"));
        }
        
        if config.name.is_empty() {
            return Err(FlowyError::invalid_data().with_context("服务器名称不能为空"));
        }

        // 根据传输类型验证配置
        match config.transport_type {
            MCPTransportType::Stdio => {
                if let Some(stdio_config) = &config.stdio_config {
                    if stdio_config.command.is_empty() {
                        return Err(FlowyError::invalid_data().with_context("STDIO命令不能为空"));
                    }
                } else {
                    return Err(FlowyError::invalid_data().with_context("STDIO传输方式需要配置命令"));
                }
            }
            MCPTransportType::HTTP | MCPTransportType::SSE => {
                if let Some(http_config) = &config.http_config {
                    if http_config.url.is_empty() {
                        return Err(FlowyError::invalid_data().with_context("HTTP/SSE URL不能为空"));
                    }
                    
                    // 简单的URL格式验证
                    if !http_config.url.starts_with("http://") && !http_config.url.starts_with("https://") {
                        return Err(FlowyError::invalid_data().with_context("URL必须以http://或https://开头"));
                    }
                } else {
                    return Err(FlowyError::invalid_data().with_context("HTTP/SSE传输方式需要配置URL"));
                }
            }
        }

        Ok(())
    }

    /// 更新服务器列表
    fn update_server_list(&self, server_id: &str, add: bool) -> FlowyResult<()> {
        let mut server_ids: Vec<String> = self.store_preferences
            .get_object::<Vec<String>>(MCP_SERVER_LIST_KEY)
            .unwrap_or_default();

        if add {
            if !server_ids.contains(&server_id.to_string()) {
                server_ids.push(server_id.to_string());
            }
        } else {
            server_ids.retain(|id| id != server_id);
        }

        self.store_preferences
            .set_object(MCP_SERVER_LIST_KEY, &server_ids)
            .map_err(|e| {
                error!("Failed to update server list: {}", e);
                FlowyError::internal().with_context(format!("更新服务器列表失败: {}", e))
            })?;

        Ok(())
    }

    /// 生成服务器配置的存储键
    fn server_config_key(&self, server_id: &str) -> String {
        format!("{}:server:{}", MCP_CONFIG_PREFIX, server_id)
    }
}

/// 配置导出结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfigExport {
    pub version: String,
    pub exported_at: SystemTime,
    pub global_settings: MCPGlobalSettings,
    pub servers: Vec<MCPServerConfig>,
}

/// 配置导入结果
#[derive(Debug, Clone, Default)]
pub struct MCPImportResult {
    pub global_settings_imported: bool,
    pub servers_imported: usize,
    pub imported_server_ids: Vec<String>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowy_sqlite::kv::KVStorePreferences;
    use tempfile::TempDir;

    fn create_test_config_manager() -> (MCPConfigManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        let manager = MCPConfigManager::new(store);
        (manager, tempdir)
    }

    fn create_test_stdio_config() -> MCPServerConfig {
        MCPServerConfig::new_stdio(
            "test-stdio".to_string(),
            "Test STDIO Server".to_string(),
            "node".to_string(),
            vec!["server.js".to_string()],
        )
    }

    fn create_test_http_config() -> MCPServerConfig {
        MCPServerConfig::new_http(
            "test-http".to_string(),
            "Test HTTP Server".to_string(),
            "http://localhost:3000".to_string(),
            MCPTransportType::HTTP,
        )
    }

    #[test]
    fn test_global_settings() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试默认设置
        let default_settings = manager.get_global_settings();
        assert!(default_settings.enabled);
        assert_eq!(default_settings.connection_timeout, 30);
        
        // 测试保存和读取设置
        let mut custom_settings = default_settings.clone();
        custom_settings.enabled = false;
        custom_settings.connection_timeout = 60;
        
        manager.save_global_settings(custom_settings.clone()).unwrap();
        let loaded_settings = manager.get_global_settings();
        
        assert!(!loaded_settings.enabled);
        assert_eq!(loaded_settings.connection_timeout, 60);
    }

    #[test]
    fn test_server_config_crud() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试保存服务器配置
        let config = create_test_stdio_config();
        manager.save_server(config.clone()).unwrap();
        
        // 测试读取服务器配置
        let loaded_config = manager.get_server(&config.id).unwrap();
        assert_eq!(loaded_config.id, config.id);
        assert_eq!(loaded_config.name, config.name);
        
        // 测试获取所有服务器
        let all_servers = manager.get_all_servers();
        assert_eq!(all_servers.len(), 1);
        
        // 测试删除服务器配置
        manager.delete_server(&config.id).unwrap();
        assert!(manager.get_server(&config.id).is_none());
        assert_eq!(manager.get_all_servers().len(), 0);
    }

    #[test]
    fn test_server_validation() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试空ID验证
        let mut invalid_config = create_test_stdio_config();
        invalid_config.id = "".to_string();
        assert!(manager.save_server(invalid_config).is_err());
        
        // 测试空名称验证
        let mut invalid_config = create_test_stdio_config();
        invalid_config.name = "".to_string();
        assert!(manager.save_server(invalid_config).is_err());
        
        // 测试STDIO配置验证
        let mut invalid_config = create_test_stdio_config();
        invalid_config.stdio_config = None;
        assert!(manager.save_server(invalid_config).is_err());
        
        // 测试HTTP配置验证
        let mut invalid_config = create_test_http_config();
        invalid_config.http_config = None;
        assert!(manager.save_server(invalid_config).is_err());
    }

    #[test]
    fn test_active_status_management() {
        let (manager, _tempdir) = create_test_config_manager();
        
        let config = create_test_stdio_config();
        manager.save_server(config.clone()).unwrap();
        
        // 测试更新激活状态
        manager.update_server_active_status(&config.id, false).unwrap();
        let updated_config = manager.get_server(&config.id).unwrap();
        assert!(!updated_config.is_active);
        
        // 测试获取激活的服务器
        let active_servers = manager.get_active_servers();
        assert_eq!(active_servers.len(), 0);
        
        manager.update_server_active_status(&config.id, true).unwrap();
        let active_servers = manager.get_active_servers();
        assert_eq!(active_servers.len(), 1);
    }

    #[test]
    fn test_transport_type_filtering() {
        let (manager, _tempdir) = create_test_config_manager();
        
        let stdio_config = create_test_stdio_config();
        let http_config = create_test_http_config();
        
        manager.save_server(stdio_config).unwrap();
        manager.save_server(http_config).unwrap();
        
        let stdio_servers = manager.get_servers_by_transport(MCPTransportType::Stdio);
        let http_servers = manager.get_servers_by_transport(MCPTransportType::HTTP);
        
        assert_eq!(stdio_servers.len(), 1);
        assert_eq!(http_servers.len(), 1);
    }

    #[test]
    fn test_config_export_import() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 准备测试数据
        let stdio_config = create_test_stdio_config();
        let http_config = create_test_http_config();
        
        manager.save_server(stdio_config).unwrap();
        manager.save_server(http_config).unwrap();
        
        // 测试导出
        let export = manager.export_config().unwrap();
        assert_eq!(export.servers.len(), 2);
        
        // 清理配置
        manager.clear_all_config().unwrap();
        assert_eq!(manager.get_all_servers().len(), 0);
        
        // 测试导入
        let import_result = manager.import_config(export).unwrap();
        assert!(import_result.global_settings_imported);
        assert_eq!(import_result.servers_imported, 2);
        assert_eq!(manager.get_all_servers().len(), 2);
    }
}
