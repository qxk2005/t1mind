use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::entities::{ToolDefinitionPB, ToolTypePB};
use crate::mcp::entities::MCPTool;
use crate::mcp::tool_security::{ToolSecurityManager, ToolExecutionPermission};
use crate::agent::native_tools::NativeToolsManager;

/// 工具注册表 - 统一管理所有类型的工具
/// 支持MCP、原生、搜索等工具的元数据管理，包含发现和权限管理
pub struct ToolRegistry {
    /// 工具存储：按类型分组的工具映射
    tools: Arc<RwLock<HashMap<ToolTypePB, HashMap<String, RegisteredTool>>>>,
    /// 工具版本管理
    tool_versions: Arc<RwLock<HashMap<String, ToolVersion>>>,
    /// 权限管理器
    security_manager: Arc<ToolSecurityManager>,
    /// 持久化存储
    store_preferences: Arc<KVStorePreferences>,
    /// 工具发现监听器
    discovery_listeners: Arc<RwLock<Vec<Box<dyn ToolDiscoveryListener + Send + Sync>>>>,
    /// 原生工具管理器
    native_tools: Option<Arc<NativeToolsManager>>,
}

/// 注册的工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredTool {
    /// 工具定义
    pub definition: ToolDefinitionPB,
    /// 注册时间
    pub registered_at: SystemTime,
    /// 最后更新时间
    pub updated_at: SystemTime,
    /// 工具状态
    pub status: ToolStatus,
    /// 使用统计
    pub usage_stats: ToolUsageStats,
    /// 工具配置
    pub config: ToolConfig,
    /// 依赖关系
    pub dependencies: Vec<String>,
}

/// 工具状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolStatus {
    /// 可用
    Available,
    /// 不可用
    Unavailable,
    /// 已禁用
    Disabled,
    /// 维护中
    Maintenance,
    /// 已弃用
    Deprecated,
}

/// 工具使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolUsageStats {
    /// 总调用次数
    pub total_calls: u64,
    /// 成功调用次数
    pub successful_calls: u64,
    /// 失败调用次数
    pub failed_calls: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 最后调用时间
    pub last_called_at: Option<SystemTime>,
    /// 用户评分（1-5）
    pub user_rating: Option<f32>,
}

/// 工具配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    /// 超时设置（秒）
    pub timeout_seconds: Option<u64>,
    /// 重试次数
    pub retry_count: Option<u32>,
    /// 缓存策略
    pub cache_policy: CachePolicy,
    /// 并发限制
    pub concurrency_limit: Option<u32>,
    /// 自定义配置
    pub custom_config: HashMap<String, Value>,
}

/// 缓存策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CachePolicy {
    /// 不缓存
    None,
    /// 短期缓存（5分钟）
    Short,
    /// 中期缓存（1小时）
    Medium,
    /// 长期缓存（24小时）
    Long,
    /// 自定义缓存时间（秒）
    Custom(u64),
}

impl Default for CachePolicy {
    fn default() -> Self {
        CachePolicy::None
    }
}

/// 工具版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersion {
    /// 工具名称
    pub tool_name: String,
    /// 当前版本
    pub current_version: String,
    /// 版本历史
    pub version_history: Vec<VersionEntry>,
    /// 兼容性信息
    pub compatibility: CompatibilityInfo,
}

/// 版本条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// 版本号
    pub version: String,
    /// 发布时间
    pub released_at: SystemTime,
    /// 变更说明
    pub changelog: String,
    /// 是否向后兼容
    pub backward_compatible: bool,
}

/// 兼容性信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompatibilityInfo {
    /// 最小支持版本
    pub min_supported_version: Option<String>,
    /// 推荐版本
    pub recommended_version: Option<String>,
    /// 已知不兼容版本
    pub incompatible_versions: Vec<String>,
}

/// 工具发现监听器接口
pub trait ToolDiscoveryListener {
    /// 工具被发现时调用
    fn on_tool_discovered(&self, tool: &RegisteredTool);
    /// 工具被移除时调用
    fn on_tool_removed(&self, tool_name: &str, tool_type: ToolTypePB);
    /// 工具状态变更时调用
    fn on_tool_status_changed(&self, tool_name: &str, old_status: ToolStatus, new_status: ToolStatus);
}

/// 工具搜索过滤器
#[derive(Debug, Clone, Default)]
pub struct ToolSearchFilter {
    /// 工具类型过滤
    pub tool_types: Option<Vec<ToolTypePB>>,
    /// 状态过滤
    pub statuses: Option<Vec<ToolStatus>>,
    /// 权限过滤
    pub required_permissions: Option<Vec<String>>,
    /// 来源过滤
    pub sources: Option<Vec<String>>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 最小评分
    pub min_rating: Option<f32>,
}

/// 工具注册请求
#[derive(Debug, Clone)]
pub struct ToolRegistrationRequest {
    /// 工具定义
    pub definition: ToolDefinitionPB,
    /// 工具配置
    pub config: Option<ToolConfig>,
    /// 依赖关系
    pub dependencies: Vec<String>,
    /// 是否覆盖已存在的工具
    pub overwrite: bool,
}

const TOOL_REGISTRY_KEY: &str = "agent_tool_registry";
const TOOL_VERSIONS_KEY: &str = "agent_tool_versions";

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new(
        security_manager: Arc<ToolSecurityManager>,
        store_preferences: Arc<KVStorePreferences>,
    ) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tool_versions: Arc::new(RwLock::new(HashMap::new())),
            security_manager,
            store_preferences,
            discovery_listeners: Arc::new(RwLock::new(Vec::new())),
            native_tools: None,
        }
    }

    /// 设置原生工具管理器
    pub fn with_native_tools(mut self, native_tools: Arc<NativeToolsManager>) -> Self {
        self.native_tools = Some(native_tools);
        self
    }

    /// 初始化工具注册表
    pub async fn initialize(&self) -> FlowyResult<()> {
        info!("初始化工具注册表");
        
        // 从持久化存储加载工具
        self.load_from_storage().await?;
        
        // 注册内置工具
        self.register_builtin_tools().await?;
        
        info!("工具注册表初始化完成");
        Ok(())
    }

    /// 注册工具
    pub async fn register_tool(&self, request: ToolRegistrationRequest) -> FlowyResult<()> {
        let tool_name = &request.definition.name;
        let tool_type = request.definition.tool_type;
        
        info!("注册工具: {} (类型: {:?})", tool_name, tool_type);
        
        // 验证工具定义
        self.validate_tool_definition(&request.definition)?;
        
        // 检查权限
        self.check_registration_permission(&request.definition)?;
        
        let mut tools = self.tools.write().await;
        let type_tools = tools.entry(tool_type).or_insert_with(HashMap::new);
        
        // 检查是否已存在
        if type_tools.contains_key(tool_name) && !request.overwrite {
            return Err(FlowyError::invalid_data()
                .with_context(format!("工具 '{}' 已存在，使用 overwrite=true 来覆盖", tool_name)));
        }
        
        // 创建注册工具
        let registered_tool = RegisteredTool {
            definition: request.definition.clone(),
            registered_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            status: ToolStatus::Available,
            usage_stats: ToolUsageStats::default(),
            config: request.config.unwrap_or_default(),
            dependencies: request.dependencies,
        };
        
        // 插入工具
        type_tools.insert(tool_name.clone(), registered_tool.clone());
        
        // 更新版本信息
        self.update_tool_version(tool_name, "1.0.0", "初始注册").await?;
        
        // 持久化
        self.save_to_storage().await?;
        
        // 通知监听器
        self.notify_tool_discovered(&registered_tool).await;
        
        info!("工具注册成功: {}", tool_name);
        Ok(())
    }

    /// 注销工具
    pub async fn unregister_tool(&self, tool_name: &str, tool_type: ToolTypePB) -> FlowyResult<()> {
        info!("注销工具: {} (类型: {:?})", tool_name, tool_type);
        
        let mut tools = self.tools.write().await;
        
        if let Some(type_tools) = tools.get_mut(&tool_type) {
            if type_tools.remove(tool_name).is_some() {
                // 持久化
                drop(tools);
                self.save_to_storage().await?;
                
                // 通知监听器
                self.notify_tool_removed(tool_name, tool_type).await;
                
                info!("工具注销成功: {}", tool_name);
                Ok(())
            } else {
                Err(FlowyError::record_not_found()
                    .with_context(format!("工具 '{}' 不存在", tool_name)))
            }
        } else {
            Err(FlowyError::record_not_found()
                .with_context(format!("工具类型 '{:?}' 不存在", tool_type)))
        }
    }

    /// 获取工具
    pub async fn get_tool(&self, tool_name: &str, tool_type: ToolTypePB) -> Option<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.get(&tool_type)?.get(tool_name).cloned()
    }

    /// 获取所有工具
    pub async fn get_all_tools(&self) -> HashMap<ToolTypePB, HashMap<String, RegisteredTool>> {
        let tools = self.tools.read().await;
        tools.clone()
    }

    /// 按类型获取工具
    pub async fn get_tools_by_type(&self, tool_type: ToolTypePB) -> Vec<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.get(&tool_type)
            .map(|type_tools| type_tools.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 搜索工具
    pub async fn search_tools(&self, query: &str, filter: Option<ToolSearchFilter>) -> Vec<RegisteredTool> {
        let tools = self.tools.read().await;
        let mut results = Vec::new();
        
        let query_lower = query.to_lowercase();
        let filter = filter.unwrap_or_default();
        
        for (tool_type, type_tools) in tools.iter() {
            // 类型过滤
            if let Some(ref allowed_types) = filter.tool_types {
                if !allowed_types.contains(tool_type) {
                    continue;
                }
            }
            
            for tool in type_tools.values() {
                // 状态过滤
                if let Some(ref allowed_statuses) = filter.statuses {
                    if !allowed_statuses.contains(&tool.status) {
                        continue;
                    }
                }
                
                // 权限过滤
                if let Some(ref required_perms) = filter.required_permissions {
                    if !required_perms.iter().all(|perm| tool.definition.permissions.contains(perm)) {
                        continue;
                    }
                }
                
                // 来源过滤
                if let Some(ref allowed_sources) = filter.sources {
                    if !allowed_sources.contains(&tool.definition.source) {
                        continue;
                    }
                }
                
                // 评分过滤
                if let Some(min_rating) = filter.min_rating {
                    if let Some(rating) = tool.usage_stats.user_rating {
                        if rating < min_rating {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                
                // 文本搜索
                if query.is_empty() || 
                   tool.definition.name.to_lowercase().contains(&query_lower) ||
                   tool.definition.description.to_lowercase().contains(&query_lower) {
                    results.push(tool.clone());
                }
            }
        }
        
        // 按相关性排序
        results.sort_by(|a, b| {
            let a_score = self.calculate_relevance_score(&a.definition, query);
            let b_score = self.calculate_relevance_score(&b.definition, query);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }

    /// 更新工具状态
    pub async fn update_tool_status(
        &self,
        tool_name: &str,
        tool_type: ToolTypePB,
        new_status: ToolStatus,
    ) -> FlowyResult<()> {
        let mut tools = self.tools.write().await;
        
        if let Some(type_tools) = tools.get_mut(&tool_type) {
            if let Some(tool) = type_tools.get_mut(tool_name) {
                let old_status = tool.status.clone();
                tool.status = new_status.clone();
                tool.updated_at = SystemTime::now();
                
                // 持久化
                drop(tools);
                self.save_to_storage().await?;
                
                // 通知监听器
                self.notify_tool_status_changed(tool_name, old_status, new_status).await;
                
                Ok(())
            } else {
                Err(FlowyError::record_not_found()
                    .with_context(format!("工具 '{}' 不存在", tool_name)))
            }
        } else {
            Err(FlowyError::record_not_found()
                .with_context(format!("工具类型 '{:?}' 不存在", tool_type)))
        }
    }

    /// 更新工具使用统计
    pub async fn update_tool_usage(
        &self,
        tool_name: &str,
        tool_type: ToolTypePB,
        execution_time_ms: u64,
        success: bool,
    ) -> FlowyResult<()> {
        let mut tools = self.tools.write().await;
        
        if let Some(type_tools) = tools.get_mut(&tool_type) {
            if let Some(tool) = type_tools.get_mut(tool_name) {
                let stats = &mut tool.usage_stats;
                
                stats.total_calls += 1;
                if success {
                    stats.successful_calls += 1;
                } else {
                    stats.failed_calls += 1;
                }
                
                // 更新平均执行时间
                let total_time = stats.avg_execution_time_ms * (stats.total_calls - 1) as f64 + execution_time_ms as f64;
                stats.avg_execution_time_ms = total_time / stats.total_calls as f64;
                
                stats.last_called_at = Some(SystemTime::now());
                tool.updated_at = SystemTime::now();
                
                // 异步持久化（不阻塞）
                let registry = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = registry.save_to_storage().await {
                        warn!("保存工具使用统计失败: {}", e);
                    }
                });
                
                Ok(())
            } else {
                Err(FlowyError::record_not_found()
                    .with_context(format!("工具 '{}' 不存在", tool_name)))
            }
        } else {
            Err(FlowyError::record_not_found()
                .with_context(format!("工具类型 '{:?}' 不存在", tool_type)))
        }
    }

    /// 检查工具权限
    pub async fn check_tool_permission(
        &self,
        tool_name: &str,
        tool_type: ToolTypePB,
        server_id: Option<&str>,
    ) -> FlowyResult<ToolExecutionPermission> {
        let tool = self.get_tool(tool_name, tool_type).await
            .ok_or_else(|| FlowyError::record_not_found()
                .with_context(format!("工具 '{}' 不存在", tool_name)))?;
        
        // 检查工具状态
        match tool.status {
            ToolStatus::Available => {},
            ToolStatus::Unavailable => {
                return Ok(ToolExecutionPermission::Denied("工具当前不可用".to_string()));
            },
            ToolStatus::Disabled => {
                return Ok(ToolExecutionPermission::Denied("工具已被禁用".to_string()));
            },
            ToolStatus::Maintenance => {
                return Ok(ToolExecutionPermission::Denied("工具正在维护中".to_string()));
            },
            ToolStatus::Deprecated => {
                return Ok(ToolExecutionPermission::RequireConfirmation(
                    "工具已弃用，建议使用替代方案。是否继续？".to_string()
                ));
            },
        }
        
        // 对于MCP工具，使用现有的安全管理器
        if tool_type == ToolTypePB::MCP {
            if let Some(mcp_tool) = self.convert_to_mcp_tool(&tool.definition) {
                return Ok(self.security_manager.check_tool_permission(&mcp_tool, server_id.unwrap_or("")));
            }
        }
        
        // 对于其他类型的工具，进行基本权限检查
        if tool.definition.permissions.is_empty() {
            Ok(ToolExecutionPermission::AutoExecute)
        } else {
            // 这里可以集成更复杂的权限检查逻辑
            Ok(ToolExecutionPermission::RequireConfirmation(
                format!("工具 '{}' 需要以下权限: {}。是否继续？", 
                    tool.definition.name,
                    tool.definition.permissions.join(", ")
                )
            ))
        }
    }

    /// 发现MCP工具
    pub async fn discover_mcp_tools(&self, server_id: &str, tools: Vec<MCPTool>) -> FlowyResult<()> {
        info!("发现 {} 个MCP工具来自服务器: {}", tools.len(), server_id);
        
        for mcp_tool in tools {
            let tool_definition = ToolDefinitionPB {
                name: mcp_tool.name.clone(),
                description: mcp_tool.description.clone().unwrap_or_default(),
                tool_type: ToolTypePB::MCP,
                source: server_id.to_string(),
                parameters_schema: serde_json::to_string(&mcp_tool.input_schema).unwrap_or_default(),
                permissions: self.extract_permissions_from_mcp_tool(&mcp_tool),
                is_available: true,
                metadata: self.extract_metadata_from_mcp_tool(&mcp_tool),
            };
            
            let request = ToolRegistrationRequest {
                definition: tool_definition,
                config: Some(self.create_default_mcp_tool_config(&mcp_tool)),
                dependencies: Vec::new(),
                overwrite: true, // MCP工具发现时允许覆盖
            };
            
            if let Err(e) = self.register_tool(request).await {
                warn!("注册MCP工具失败 '{}': {}", mcp_tool.name, e);
            }
        }
        
        Ok(())
    }

    /// 清理服务器工具
    pub async fn cleanup_server_tools(&self, server_id: &str) -> FlowyResult<()> {
        info!("清理服务器工具: {}", server_id);
        
        let mut tools = self.tools.write().await;
        let mut removed_tools = Vec::new();
        
        if let Some(mcp_tools) = tools.get_mut(&ToolTypePB::MCP) {
            mcp_tools.retain(|name, tool| {
                if tool.definition.source == server_id {
                    removed_tools.push(name.clone());
                    false
                } else {
                    true
                }
            });
        }
        
        if !removed_tools.is_empty() {
            // 持久化
            drop(tools);
            self.save_to_storage().await?;
            
            // 通知监听器
            for tool_name in &removed_tools {
                self.notify_tool_removed(tool_name, ToolTypePB::MCP).await;
            }
        }
        
        Ok(())
    }

    /// 添加发现监听器
    pub async fn add_discovery_listener(&self, listener: Box<dyn ToolDiscoveryListener + Send + Sync>) {
        let mut listeners = self.discovery_listeners.write().await;
        listeners.push(listener);
    }

    /// 获取工具统计信息
    pub async fn get_tool_statistics(&self) -> ToolRegistryStatistics {
        let tools = self.tools.read().await;
        let mut stats = ToolRegistryStatistics::default();
        
        for (tool_type, type_tools) in tools.iter() {
            let type_count = type_tools.len();
            stats.total_tools += type_count;
            
            match tool_type {
                ToolTypePB::MCP => stats.mcp_tools = type_count,
                ToolTypePB::Native => stats.native_tools = type_count,
                ToolTypePB::Search => stats.search_tools = type_count,
                ToolTypePB::ExternalAPI => stats.external_api_tools = type_count,
            }
            
            for tool in type_tools.values() {
                match tool.status {
                    ToolStatus::Available => stats.available_tools += 1,
                    ToolStatus::Unavailable => stats.unavailable_tools += 1,
                    ToolStatus::Disabled => stats.disabled_tools += 1,
                    ToolStatus::Maintenance => stats.maintenance_tools += 1,
                    ToolStatus::Deprecated => stats.deprecated_tools += 1,
                }
                
                stats.total_calls += tool.usage_stats.total_calls;
                stats.successful_calls += tool.usage_stats.successful_calls;
                stats.failed_calls += tool.usage_stats.failed_calls;
            }
        }
        
        stats
    }

    /// 导出工具注册表
    pub async fn export_registry(&self) -> FlowyResult<String> {
        let tools = self.tools.read().await;
        let versions = self.tool_versions.read().await;
        
        let export_data = json!({
            "tools": *tools,
            "versions": *versions,
            "exported_at": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });
        
        serde_json::to_string_pretty(&export_data)
            .map_err(|e| FlowyError::internal().with_context(format!("导出失败: {}", e)))
    }

    /// 导入工具注册表
    pub async fn import_registry(&self, data: &str, merge: bool) -> FlowyResult<()> {
        let import_data: Value = serde_json::from_str(data)
            .map_err(|e| FlowyError::invalid_data().with_context(format!("解析导入数据失败: {}", e)))?;
        
        if !merge {
            // 清空现有数据
            let mut tools = self.tools.write().await;
            let mut versions = self.tool_versions.write().await;
            tools.clear();
            versions.clear();
        }
        
        // 导入工具
        if let Some(tools_data) = import_data.get("tools") {
            let imported_tools: HashMap<ToolTypePB, HashMap<String, RegisteredTool>> = 
                serde_json::from_value(tools_data.clone())
                    .map_err(|e| FlowyError::invalid_data().with_context(format!("导入工具数据失败: {}", e)))?;
            
            let mut tools = self.tools.write().await;
            for (tool_type, type_tools) in imported_tools {
                let existing_type_tools = tools.entry(tool_type).or_insert_with(HashMap::new);
                for (name, tool) in type_tools {
                    existing_type_tools.insert(name, tool);
                }
            }
        }
        
        // 导入版本信息
        if let Some(versions_data) = import_data.get("versions") {
            let imported_versions: HashMap<String, ToolVersion> = 
                serde_json::from_value(versions_data.clone())
                    .map_err(|e| FlowyError::invalid_data().with_context(format!("导入版本数据失败: {}", e)))?;
            
            let mut versions = self.tool_versions.write().await;
            for (name, version) in imported_versions {
                versions.insert(name, version);
            }
        }
        
        // 持久化
        self.save_to_storage().await?;
        
        info!("工具注册表导入完成");
        Ok(())
    }

    // 私有辅助方法

    /// 验证工具定义
    fn validate_tool_definition(&self, definition: &ToolDefinitionPB) -> FlowyResult<()> {
        if definition.name.is_empty() {
            return Err(FlowyError::invalid_data().with_context("工具名称不能为空"));
        }
        
        if definition.description.is_empty() {
            return Err(FlowyError::invalid_data().with_context("工具描述不能为空"));
        }
        
        if definition.source.is_empty() {
            return Err(FlowyError::invalid_data().with_context("工具来源不能为空"));
        }
        
        // 验证参数schema
        if !definition.parameters_schema.is_empty() {
            serde_json::from_str::<Value>(&definition.parameters_schema)
                .map_err(|e| FlowyError::invalid_data()
                    .with_context(format!("无效的参数schema: {}", e)))?;
        }
        
        Ok(())
    }

    /// 检查注册权限
    fn check_registration_permission(&self, _definition: &ToolDefinitionPB) -> FlowyResult<()> {
        // 这里可以实现更复杂的权限检查逻辑
        // 例如：检查用户是否有权限注册特定类型的工具
        Ok(())
    }

    /// 从持久化存储加载
    async fn load_from_storage(&self) -> FlowyResult<()> {
        // 加载工具
        if let Some(tools_data) = self.store_preferences.get_str(TOOL_REGISTRY_KEY) {
            if let Ok(loaded_tools) = serde_json::from_str::<HashMap<ToolTypePB, HashMap<String, RegisteredTool>>>(&tools_data) {
                let mut tools = self.tools.write().await;
                *tools = loaded_tools;
                debug!("从存储加载了工具注册表");
            }
        }
        
        // 加载版本信息
        if let Some(versions_data) = self.store_preferences.get_str(TOOL_VERSIONS_KEY) {
            if let Ok(loaded_versions) = serde_json::from_str::<HashMap<String, ToolVersion>>(&versions_data) {
                let mut versions = self.tool_versions.write().await;
                *versions = loaded_versions;
                debug!("从存储加载了工具版本信息");
            }
        }
        
        Ok(())
    }

    /// 保存到持久化存储
    async fn save_to_storage(&self) -> FlowyResult<()> {
        // 保存工具
        let tools = self.tools.read().await;
        let tools_json = serde_json::to_string(&*tools)
            .map_err(|e| FlowyError::internal().with_context(format!("序列化工具数据失败: {}", e)))?;
        
        self.store_preferences.set_str(TOOL_REGISTRY_KEY, tools_json);
        
        // 保存版本信息
        let versions = self.tool_versions.read().await;
        let versions_json = serde_json::to_string(&*versions)
            .map_err(|e| FlowyError::internal().with_context(format!("序列化版本数据失败: {}", e)))?;
        
        self.store_preferences.set_str(TOOL_VERSIONS_KEY, versions_json);
        
        debug!("工具注册表已保存到存储");
        Ok(())
    }

    /// 注册内置工具
    async fn register_builtin_tools(&self) -> FlowyResult<()> {
        // 从原生工具管理器获取工具定义
        let native_tools = if let Some(native_tools_manager) = &self.native_tools {
            native_tools_manager.get_tool_definitions()
        } else {
            // 回退到旧的硬编码定义
            vec![
                ToolDefinitionPB {
                    name: "create_document".to_string(),
                    description: "创建新文档".to_string(),
                    tool_type: ToolTypePB::Native,
                    source: "appflowy".to_string(),
                    parameters_schema: json!({
                        "type": "object",
                        "properties": {
                            "title": {"type": "string", "description": "文档标题"},
                            "content": {"type": "string", "description": "文档内容"}
                        },
                        "required": ["title"]
                    }).to_string(),
                    permissions: vec!["document.create".to_string()],
                    is_available: true,
                    metadata: HashMap::new(),
                },
                ToolDefinitionPB {
                    name: "search_documents".to_string(),
                    description: "搜索文档".to_string(),
                    tool_type: ToolTypePB::Native,
                    source: "appflowy".to_string(),
                    parameters_schema: json!({
                        "type": "object",
                        "properties": {
                            "query": {"type": "string", "description": "搜索关键词"},
                            "limit": {"type": "integer", "description": "结果数量限制", "default": 10}
                        },
                        "required": ["query"]
                    }).to_string(),
                    permissions: vec!["document.read".to_string()],
                    is_available: true,
                    metadata: HashMap::new(),
                },
            ]
        };
        
        for tool_def in native_tools {
            let request = ToolRegistrationRequest {
                definition: tool_def,
                config: Some(ToolConfig {
                    timeout_seconds: Some(30),
                    retry_count: Some(3),
                    cache_policy: CachePolicy::Short,
                    concurrency_limit: Some(10),
                    custom_config: HashMap::new(),
                }),
                dependencies: Vec::new(),
                overwrite: true,
            };
            
            if let Err(e) = self.register_tool(request).await {
                warn!("注册内置工具失败: {}", e);
            }
        }
        
        // 注册搜索工具
        let search_tools = vec![
            ToolDefinitionPB {
                name: "web_search".to_string(),
                description: "网络搜索".to_string(),
                tool_type: ToolTypePB::Search,
                source: "builtin".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "搜索查询"},
                        "max_results": {"type": "integer", "description": "最大结果数", "default": 10}
                    },
                    "required": ["query"]
                }).to_string(),
                permissions: vec!["search.web".to_string()],
                is_available: true,
                metadata: HashMap::new(),
            },
        ];
        
        for tool_def in search_tools {
            let request = ToolRegistrationRequest {
                definition: tool_def,
                config: Some(ToolConfig {
                    timeout_seconds: Some(60),
                    retry_count: Some(2),
                    cache_policy: CachePolicy::Medium,
                    concurrency_limit: Some(5),
                    custom_config: HashMap::new(),
                }),
                dependencies: Vec::new(),
                overwrite: true,
            };
            
            if let Err(e) = self.register_tool(request).await {
                warn!("注册搜索工具失败: {}", e);
            }
        }
        
        Ok(())
    }

    /// 更新工具版本
    async fn update_tool_version(&self, tool_name: &str, version: &str, changelog: &str) -> FlowyResult<()> {
        let mut versions = self.tool_versions.write().await;
        
        let tool_version = versions.entry(tool_name.to_string()).or_insert_with(|| ToolVersion {
            tool_name: tool_name.to_string(),
            current_version: version.to_string(),
            version_history: Vec::new(),
            compatibility: CompatibilityInfo::default(),
        });
        
        // 添加版本历史
        tool_version.version_history.push(VersionEntry {
            version: version.to_string(),
            released_at: SystemTime::now(),
            changelog: changelog.to_string(),
            backward_compatible: true, // 默认向后兼容
        });
        
        tool_version.current_version = version.to_string();
        
        Ok(())
    }

    /// 计算相关性分数
    fn calculate_relevance_score(&self, definition: &ToolDefinitionPB, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;
        
        // 名称匹配权重最高
        if definition.name.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }
        
        // 描述匹配权重中等
        if definition.description.to_lowercase().contains(&query_lower) {
            score += 5.0;
        }
        
        // 来源匹配权重较低
        if definition.source.to_lowercase().contains(&query_lower) {
            score += 2.0;
        }
        
        score
    }

    /// 转换为MCP工具
    fn convert_to_mcp_tool(&self, definition: &ToolDefinitionPB) -> Option<MCPTool> {
        if definition.tool_type != ToolTypePB::MCP {
            return None;
        }
        
        let input_schema = serde_json::from_str(&definition.parameters_schema).ok()?;
        
        Some(MCPTool {
            name: definition.name.clone(),
            description: if definition.description.is_empty() { 
                None 
            } else { 
                Some(definition.description.clone()) 
            },
            input_schema,
            annotations: None, // 可以从metadata中提取
        })
    }

    /// 从MCP工具提取权限
    fn extract_permissions_from_mcp_tool(&self, mcp_tool: &MCPTool) -> Vec<String> {
        let mut permissions = Vec::new();
        
        if let Some(ref annotations) = mcp_tool.annotations {
            if annotations.destructive_hint == Some(true) {
                permissions.push("destructive".to_string());
            }
            if annotations.read_only_hint == Some(true) {
                permissions.push("read_only".to_string());
            }
        }
        
        permissions
    }

    /// 从MCP工具提取元数据
    fn extract_metadata_from_mcp_tool(&self, mcp_tool: &MCPTool) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        if let Some(ref annotations) = mcp_tool.annotations {
            if let Some(ref title) = annotations.title {
                metadata.insert("title".to_string(), title.clone());
            }
            if let Some(destructive) = annotations.destructive_hint {
                metadata.insert("destructive".to_string(), destructive.to_string());
            }
            if let Some(read_only) = annotations.read_only_hint {
                metadata.insert("read_only".to_string(), read_only.to_string());
            }
            if let Some(idempotent) = annotations.idempotent_hint {
                metadata.insert("idempotent".to_string(), idempotent.to_string());
            }
        }
        
        metadata
    }

    /// 创建默认MCP工具配置
    fn create_default_mcp_tool_config(&self, mcp_tool: &MCPTool) -> ToolConfig {
        let mut config = ToolConfig::default();
        
        // 根据工具注解设置配置
        if let Some(ref annotations) = mcp_tool.annotations {
            if annotations.destructive_hint == Some(true) {
                config.timeout_seconds = Some(120); // 破坏性工具更长超时
                config.retry_count = Some(1); // 破坏性工具不重试
            } else if annotations.read_only_hint == Some(true) {
                config.cache_policy = CachePolicy::Medium; // 只读工具可以缓存
                config.retry_count = Some(3);
            }
        }
        
        config
    }

    /// 通知工具发现
    async fn notify_tool_discovered(&self, tool: &RegisteredTool) {
        let listeners = self.discovery_listeners.read().await;
        for listener in listeners.iter() {
            listener.on_tool_discovered(tool);
        }
    }

    /// 通知工具移除
    async fn notify_tool_removed(&self, tool_name: &str, tool_type: ToolTypePB) {
        let listeners = self.discovery_listeners.read().await;
        for listener in listeners.iter() {
            listener.on_tool_removed(tool_name, tool_type);
        }
    }

    /// 通知工具状态变更
    async fn notify_tool_status_changed(&self, tool_name: &str, old_status: ToolStatus, new_status: ToolStatus) {
        let listeners = self.discovery_listeners.read().await;
        for listener in listeners.iter() {
            listener.on_tool_status_changed(tool_name, old_status.clone(), new_status.clone());
        }
    }
}

// 实现Clone以支持异步任务
impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
            tool_versions: self.tool_versions.clone(),
            security_manager: self.security_manager.clone(),
            store_preferences: self.store_preferences.clone(),
            discovery_listeners: self.discovery_listeners.clone(),
            native_tools: self.native_tools.clone(),
        }
    }
}

/// 工具注册表统计信息
#[derive(Debug, Clone, Default)]
pub struct ToolRegistryStatistics {
    pub total_tools: usize,
    pub mcp_tools: usize,
    pub native_tools: usize,
    pub search_tools: usize,
    pub external_api_tools: usize,
    pub available_tools: usize,
    pub unavailable_tools: usize,
    pub disabled_tools: usize,
    pub maintenance_tools: usize,
    pub deprecated_tools: usize,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
}

#[cfg(test)]
#[path = "tool_registry_simple_test.rs"]
mod tool_registry_simple_test;
