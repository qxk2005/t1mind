use std::sync::Arc;
use std::collections::HashMap;

use flowy_error::FlowyResult;
use serde_json::Value;
use tracing::{info, warn};
use uuid::Uuid;

use crate::ai_manager::AIManager;
use crate::agent::planner::{AITaskPlanner, TaskPlan, PlanStatus, PersonalizationFeatures};
use crate::agent::executor::{ExecutionContext, ExecutionResult};
use crate::agent::tool_registry::{ToolRegistry, ToolRegistryStatistics, ToolSearchFilter, RegisteredTool};
use crate::agent::native_tools::NativeToolsManager;
#[cfg(feature = "mcp")]
use crate::mcp::tool_security::ToolSecurityManager;
#[cfg(feature = "web-search")]
use crate::web_search::WebSearchToolManager;
use crate::entities::{ToolDefinitionPB, ToolTypePB};

/// 智能体管理器 - 集成规划器、执行器和工具注册表
pub struct AgentManager {
    /// AI管理器引用
    ai_manager: Arc<AIManager>,
    /// 任务规划器
    planner: AITaskPlanner,
    /// 活跃的任务计划
    active_plans: HashMap<String, TaskPlan>,
    /// 工具注册表
    tool_registry: Arc<ToolRegistry>,
    /// 原生工具管理器
    native_tools: Option<Arc<NativeToolsManager>>,
    /// 网络搜索工具管理器
    #[cfg(feature = "web-search")]
    web_search_tools: Option<Arc<WebSearchToolManager>>,
}

impl AgentManager {
    /// 创建新的智能体管理器
    pub fn new(ai_manager: Arc<AIManager>) -> Self {
        let planner = AITaskPlanner::new(ai_manager.clone());
        
        // 创建工具安全管理器
        #[cfg(feature = "mcp")]
        let security_manager = Arc::new(ToolSecurityManager::new(ai_manager.store_preferences.clone()));
        
        // 创建原生工具管理器 - 暂时不创建，因为需要 DocumentManager
        // TODO: 需要从外部传入 DocumentManager 或修改 NativeToolsManager 的构造函数
        let native_tools: Option<Arc<NativeToolsManager>> = None;
        
        // 创建网络搜索工具管理器
        #[cfg(feature = "web-search")]
        let web_search_tools = Some(Arc::new(WebSearchToolManager::new(ai_manager.store_preferences.clone())));
        
        // 创建工具注册表
        let mut tool_registry = ToolRegistry::new(
            #[cfg(feature = "mcp")]
            security_manager,
            ai_manager.store_preferences.clone(),
        );
        
        // 设置原生工具管理器（如果有的话）
        if let Some(native_tools) = &native_tools {
            tool_registry = tool_registry.with_native_tools(native_tools.clone());
        }
        
        // 设置网络搜索工具管理器
        #[cfg(feature = "web-search")]
        {
            if let Some(web_search_tools) = &web_search_tools {
                tool_registry = tool_registry.with_web_search_tools(web_search_tools.clone());
            }
        }
        
        let tool_registry = Arc::new(tool_registry);
        
        Self {
            ai_manager,
            planner,
            active_plans: HashMap::new(),
            tool_registry,
            native_tools,
            #[cfg(feature = "web-search")]
            web_search_tools,
        }
    }

    /// 初始化智能体管理器
    pub async fn initialize(&self) -> FlowyResult<()> {
        info!("初始化智能体管理器");
        
        // 初始化工具注册表
        self.tool_registry.initialize().await?;
        
        // 初始化网络搜索供应商
        self.initialize_web_search_providers().await?;
        
        // 发现并注册MCP工具
        self.discover_and_register_mcp_tools().await?;
        
        info!("智能体管理器初始化完成");
        Ok(())
    }

    /// 创建并执行任务计划
    #[tracing::instrument(level = "info", skip(self, personalization, context))]
    pub async fn plan_and_execute(
        &mut self,
        user_question: &str,
        personalization: Option<PersonalizationFeatures>,
        context: ExecutionContext,
    ) -> FlowyResult<(TaskPlan, Vec<ExecutionResult>)> {
        info!("开始为用户问题创建并执行任务计划: {}", user_question);

        // 1. 创建任务计划
        let mut plan = self.planner.create_plan(
            user_question,
            personalization,
            &context.workspace_id,
        ).await?;

        info!("任务计划创建完成: {} - {}", plan.id, plan.goal);

        // 2. 存储活跃计划
        self.active_plans.insert(plan.id.clone(), plan.clone());

        // 3. 创建执行器并执行计划
        let mut executor = self.create_executor();
        let results = executor.execute_plan(&mut plan, &context).await?;

        // 4. 更新存储的计划
        self.active_plans.insert(plan.id.clone(), plan.clone());

        info!("任务计划执行完成: {} - 状态: {:?}", plan.id, plan.status);

        Ok((plan, results))
    }

    /// 仅创建任务计划（不执行）
    pub async fn create_plan_only(
        &mut self,
        user_question: &str,
        personalization: Option<PersonalizationFeatures>,
        workspace_id: &Uuid,
    ) -> FlowyResult<TaskPlan> {
        let plan = self.planner.create_plan(
            user_question,
            personalization,
            workspace_id,
        ).await?;

        self.active_plans.insert(plan.id.clone(), plan.clone());
        Ok(plan)
    }

    /// 执行已存在的任务计划
    pub async fn execute_existing_plan(
        &mut self,
        plan_id: &str,
        context: ExecutionContext,
    ) -> FlowyResult<Vec<ExecutionResult>> {
        let mut plan = self.active_plans.get(plan_id)
            .ok_or_else(|| flowy_error::FlowyError::record_not_found()
                .with_context(format!("找不到任务计划: {}", plan_id)))?
            .clone();

        let mut executor = self.create_executor();
        let results = executor.execute_plan(&mut plan, &context).await?;

        // 更新存储的计划
        self.active_plans.insert(plan.id.clone(), plan);

        Ok(results)
    }

    /// 获取任务计划
    pub fn get_plan(&self, plan_id: &str) -> Option<&TaskPlan> {
        self.active_plans.get(plan_id)
    }

    /// 获取所有活跃的任务计划
    pub fn get_all_plans(&self) -> Vec<&TaskPlan> {
        self.active_plans.values().collect()
    }

    /// 删除任务计划
    pub fn remove_plan(&mut self, plan_id: &str) -> Option<TaskPlan> {
        self.active_plans.remove(plan_id)
    }

    /// 更新任务计划状态
    pub async fn update_plan_status(&mut self, plan_id: &str, status: PlanStatus) -> FlowyResult<()> {
        if let Some(plan) = self.active_plans.get_mut(plan_id) {
            self.planner.update_plan_status(plan, status).await;
            Ok(())
        } else {
            Err(flowy_error::FlowyError::record_not_found()
                .with_context(format!("找不到任务计划: {}", plan_id)))
        }
    }

    /// 取消任务计划
    pub async fn cancel_plan(&mut self, plan_id: &str) -> FlowyResult<()> {
        self.update_plan_status(plan_id, PlanStatus::Cancelled).await?;
        info!("任务计划已取消: {}", plan_id);
        Ok(())
    }

    /// 获取计划统计信息
    pub fn get_plan_statistics(&self, plan_id: &str) -> Option<HashMap<String, Value>> {
        self.active_plans.get(plan_id)
            .map(|plan| self.planner.get_plan_statistics(plan))
    }

    /// 获取所有计划的汇总统计
    pub fn get_overall_statistics(&self) -> HashMap<String, Value> {
        let mut stats = HashMap::new();
        
        let total_plans = self.active_plans.len();
        let completed_plans = self.active_plans.values()
            .filter(|p| p.status == PlanStatus::Completed)
            .count();
        let failed_plans = self.active_plans.values()
            .filter(|p| p.status == PlanStatus::Failed)
            .count();
        let executing_plans = self.active_plans.values()
            .filter(|p| p.status == PlanStatus::Executing)
            .count();

        stats.insert("total_plans".to_string(), serde_json::json!(total_plans));
        stats.insert("completed_plans".to_string(), serde_json::json!(completed_plans));
        stats.insert("failed_plans".to_string(), serde_json::json!(failed_plans));
        stats.insert("executing_plans".to_string(), serde_json::json!(executing_plans));
        stats.insert("success_rate".to_string(), serde_json::json!(
            if total_plans > 0 { 
                completed_plans as f64 / total_plans as f64 
            } else { 
                0.0 
            }
        ));

        stats
    }

    /// 清理已完成或失败的计划
    pub fn cleanup_finished_plans(&mut self) -> usize {
        let initial_count = self.active_plans.len();
        
        self.active_plans.retain(|_, plan| {
            !matches!(plan.status, PlanStatus::Completed | PlanStatus::Failed | PlanStatus::Cancelled)
        });

        let removed_count = initial_count - self.active_plans.len();
        if removed_count > 0 {
            info!("清理了 {} 个已完成的任务计划", removed_count);
        }
        
        removed_count
    }

    /// 获取计划执行进度
    pub fn get_plan_progress(&self, plan_id: &str) -> Option<f64> {
        self.active_plans.get(plan_id).map(|plan| {
            if plan.steps.is_empty() {
                return 0.0;
            }

            let completed_steps = plan.steps.iter()
                .filter(|s| matches!(s.status, crate::agent::planner::PlanningStepStatus::Completed))
                .count();

            completed_steps as f64 / plan.steps.len() as f64
        })
    }

    /// 创建任务执行器
    pub fn create_executor(&self) -> crate::agent::executor::AITaskExecutor {
        let mut executor = crate::agent::executor::AITaskExecutor::new(self.ai_manager.clone());
        
        // 设置原生工具管理器
        if let Some(native_tools) = &self.native_tools {
            executor = executor.with_native_tools(native_tools.clone());
        }
        
        // 设置网络搜索工具管理器
        #[cfg(feature = "web-search")]
        {
            if let Some(web_search_tools) = &self.web_search_tools {
                executor = executor.with_web_search_tools(web_search_tools.clone());
            }
        }
        
        executor
    }

    /// 暂停任务计划执行
    pub async fn pause_plan(&mut self, plan_id: &str) -> FlowyResult<()> {
        // 注意：这里只是更新状态，实际的暂停逻辑需要在执行器中实现
        // 这是一个简化的实现
        if let Some(plan) = self.active_plans.get_mut(plan_id) {
            if plan.status == PlanStatus::Executing {
                plan.status = PlanStatus::Ready; // 暂停后回到就绪状态
                plan.updated_at = chrono::Utc::now();
                info!("任务计划已暂停: {}", plan_id);
            } else {
                warn!("任务计划 {} 当前状态不支持暂停: {:?}", plan_id, plan.status);
            }
            Ok(())
        } else {
            Err(flowy_error::FlowyError::record_not_found()
                .with_context(format!("找不到任务计划: {}", plan_id)))
        }
    }

    /// 恢复任务计划执行
    pub async fn resume_plan(&mut self, plan_id: &str, context: ExecutionContext) -> FlowyResult<Vec<ExecutionResult>> {
        // 检查计划是否存在且可以恢复
        if let Some(plan) = self.active_plans.get(plan_id) {
            if plan.status != PlanStatus::Ready {
                return Err(flowy_error::FlowyError::invalid_data()
                    .with_context(format!("任务计划 {} 当前状态不支持恢复: {:?}", plan_id, plan.status)));
            }
        } else {
            return Err(flowy_error::FlowyError::record_not_found()
                .with_context(format!("找不到任务计划: {}", plan_id)));
        }

        // 恢复执行
        self.execute_existing_plan(plan_id, context).await
    }

    // ==================== 工具注册表相关方法 ====================

    /// 初始化网络搜索供应商
    #[cfg(feature = "web-search")]
    async fn initialize_web_search_providers(&self) -> FlowyResult<()> {
        info!("初始化网络搜索供应商");
        
        if let Some(_web_search_tools) = &self.web_search_tools {
            // 获取网络搜索中心
            let web_search_hub = self.ai_manager.get_web_search_hub().await?;
            
            // 检查是否已有活跃的供应商
            let status = web_search_hub.get_status();
            if status.active_providers > 0 {
                info!("网络搜索供应商已存在，跳过初始化");
                return Ok(());
            }
            
            // 检查是否有已配置但未激活的供应商
            let all_providers = web_search_hub.provider_manager.get_all_providers()?;
            let inactive_providers: Vec<_> = all_providers.providers.iter()
                .filter(|p| p.is_enabled && !p.is_active)
                .collect();
            
            if !inactive_providers.is_empty() {
                info!("发现 {} 个已配置但未激活的供应商，尝试激活第一个", inactive_providers.len());
                
                // 激活第一个已配置的供应商
                let provider_to_activate = &inactive_providers[0];
                let update_request = crate::web_search::entities::UpdateWebSearchProviderRequestPB {
                    id: provider_to_activate.id.clone(),
                    name: Some(provider_to_activate.name.clone()),
                    description: Some(provider_to_activate.description.clone()),
                    icon: Some(provider_to_activate.icon.clone()),
                    api_key: Some(provider_to_activate.api_key.clone()),
                    base_url: Some(provider_to_activate.base_url.clone()),
                    is_active: Some(true), // 激活供应商
                    is_enabled: Some(true),
                    max_results: Some(provider_to_activate.max_results),
                    timeout_seconds: Some(provider_to_activate.timeout_seconds),
                    metadata: provider_to_activate.metadata.clone(),
                };
                
                match web_search_hub.provider_manager.update_provider(update_request) {
                    Ok(_) => {
                        info!("已激活供应商: {}", provider_to_activate.name);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("激活供应商 {} 失败: {}", provider_to_activate.name, e);
                    }
                }
            }
            
            // 如果没有已配置的供应商，创建默认的 Tavily 供应商
            info!("没有找到已配置的供应商，创建默认供应商");
            
            // 尝试从环境变量获取 Tavily API 密钥
            let tavily_api_key = std::env::var("TAVILY_API_KEY")
                .unwrap_or_else(|_| "demo_key".to_string());
            
            // 创建默认的 Tavily 供应商
            let tavily_request = crate::web_search::entities::CreateWebSearchProviderRequestPB {
                name: "Tavily Search".to_string(),
                provider_type: crate::entities::WebSearchProviderTypePB::Tavily,
                description: "Tavily 搜索引擎 - 默认供应商".to_string(),
                icon: "🔍".to_string(),
                api_key: tavily_api_key.clone(),
                base_url: "https://api.tavily.com".to_string(),
                max_results: 10,
                timeout_seconds: 30,
                metadata: std::collections::HashMap::new(),
            };
            
            match web_search_hub.provider_manager.create_provider(tavily_request) {
                Ok(provider) => {
                    info!("创建默认 Tavily 供应商: {}", provider.name);
                    
                    // 激活供应商
                    let update_request = crate::web_search::entities::UpdateWebSearchProviderRequestPB {
                        id: provider.id.clone(),
                        name: Some(provider.name.clone()),
                        description: Some(provider.description.clone()),
                        icon: Some(provider.icon.clone()),
                        api_key: Some(provider.api_key.clone()),
                        base_url: Some(provider.base_url.clone()),
                        is_active: Some(true), // 激活供应商
                        is_enabled: Some(true),
                        max_results: Some(provider.max_results),
                        timeout_seconds: Some(provider.timeout_seconds),
                        metadata: provider.metadata.clone(),
                    };
                    
                    match web_search_hub.provider_manager.update_provider(update_request) {
                        Ok(_) => {
                            info!("默认网络搜索供应商已激活");
                            
                            // 如果使用的是演示密钥，尝试测试供应商
                            if tavily_api_key == "demo_key" {
                                info!("使用演示密钥，跳过供应商测试");
                                // 对于演示模式，我们直接标记为测试通过
                                // 注意：这里我们需要通过 update_provider 来更新测试状态
                                // 但是由于 UpdateWebSearchProviderRequestPB 没有测试状态字段
                                // 我们需要通过其他方式来标记测试通过
                                info!("演示模式供应商已创建，建议手动测试或配置真实 API 密钥");
                            } else {
                                info!("使用真实 API 密钥，建议手动测试供应商");
                            }
                        }
                        Err(e) => {
                            warn!("激活默认网络搜索供应商失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("创建默认网络搜索供应商失败: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "web-search"))]
    async fn initialize_web_search_providers(&self) -> FlowyResult<()> {
        info!("网络搜索功能未启用，跳过供应商初始化");
        Ok(())
    }

    /// 发现并注册MCP工具
    async fn discover_and_register_mcp_tools(&self) -> FlowyResult<()> {
        info!("开始发现并注册MCP工具");
        
        #[cfg(feature = "mcp")]
        {
            let servers = self.ai_manager.mcp_manager.list_servers().await;
            for server in servers {
                if let Ok(tools_list) = self.ai_manager.mcp_manager.tool_list(&server.server_id).await {
                    if let Err(e) = self.tool_registry.discover_mcp_tools(&server.server_id, tools_list.tools).await {
                        warn!("注册MCP服务器 {} 的工具失败: {}", server.server_id, e);
                    }
                }
            }
        }
        #[cfg(not(feature = "mcp"))]
        {
            info!("MCP功能未启用，跳过工具发现");
        }
        
        Ok(())
    }

    /// 获取工具注册表引用
    pub fn tool_registry(&self) -> &Arc<ToolRegistry> {
        &self.tool_registry
    }

    /// 搜索工具
    pub async fn search_tools(&self, query: &str, filter: Option<ToolSearchFilter>) -> Vec<RegisteredTool> {
        self.tool_registry.search_tools(query, filter).await
    }

    /// 按类型获取工具
    pub async fn get_tools_by_type(&self, tool_type: ToolTypePB) -> Vec<RegisteredTool> {
        self.tool_registry.get_tools_by_type(tool_type).await
    }

    /// 获取所有可用工具
    pub async fn get_all_available_tools(&self) -> Vec<ToolDefinitionPB> {
        let all_tools = self.tool_registry.get_all_tools().await;
        let mut available_tools = Vec::new();
        
        for (_, type_tools) in all_tools {
            for (_, registered_tool) in type_tools {
                if registered_tool.definition.is_available {
                    available_tools.push(registered_tool.definition);
                }
            }
        }
        
        available_tools
    }

    /// 获取工具注册表统计信息
    pub async fn get_tool_statistics(&self) -> ToolRegistryStatistics {
        self.tool_registry.get_tool_statistics().await
    }

    /// 更新工具使用统计
    pub async fn update_tool_usage(
        &self,
        tool_name: &str,
        tool_type: ToolTypePB,
        execution_time_ms: u64,
        success: bool,
    ) -> FlowyResult<()> {
        self.tool_registry.update_tool_usage(tool_name, tool_type, execution_time_ms, success).await
    }

    /// 检查工具权限
    #[cfg(feature = "mcp")]
    pub async fn check_tool_permission(
        &self,
        tool_name: &str,
        tool_type: ToolTypePB,
        server_id: Option<&str>,
    ) -> FlowyResult<crate::mcp::tool_security::ToolExecutionPermission> {
        self.tool_registry.check_tool_permission(tool_name, tool_type, server_id).await
    }

    /// 当MCP服务器连接时注册其工具
    #[cfg(feature = "mcp")]
    pub async fn on_mcp_server_connected(&self, server_id: &str) -> FlowyResult<()> {
        info!("MCP服务器已连接，注册工具: {}", server_id);
        
        if let Ok(tools_list) = self.ai_manager.mcp_manager.tool_list(server_id).await {
            self.tool_registry.discover_mcp_tools(server_id, tools_list.tools).await?;
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "mcp"))]
    pub async fn on_mcp_server_connected(&self, _server_id: &str) -> FlowyResult<()> {
        info!("MCP功能未启用，跳过工具注册");
        Ok(())
    }

    /// 当MCP服务器断开时清理其工具
    pub async fn on_mcp_server_disconnected(&self, server_id: &str) -> FlowyResult<()> {
        info!("MCP服务器已断开，清理工具: {}", server_id);
        
        self.tool_registry.cleanup_server_tools(server_id).await
    }
    

    /// 导出工具注册表
    pub async fn export_tool_registry(&self) -> FlowyResult<String> {
        self.tool_registry.export_registry().await
    }

    /// 导入工具注册表
    pub async fn import_tool_registry(&self, data: &str, merge: bool) -> FlowyResult<()> {
        self.tool_registry.import_registry(data, merge).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_manager_creation() {
        // 这里需要模拟AIManager，实际测试中需要更完整的设置
        // let ai_manager = Arc::new(AIManager::new(...));
        // let agent_manager = AgentManager::new(ai_manager);
        // assert_eq!(agent_manager.active_plans.len(), 0);
    }

    #[test]
    fn test_plan_progress_calculation() {
        // 测试进度计算逻辑
        // 需要创建模拟的TaskPlan和PlanningStep
    }
}
