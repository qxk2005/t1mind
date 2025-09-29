use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use flowy_error::{FlowyError, FlowyResult};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::ai_manager::AIManager;
use crate::entities::{ToolDefinitionPB, ToolTypePB};
use crate::mcp::entities::MCPTool;
use crate::agent::tool_registry::ToolRegistry;
use flowy_ai_pub::cloud::{AIModel, CompleteTextParams, CompletionType, ResponseFormat, ChatCloudService};

/// 任务规划步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningStep {
    /// 步骤ID
    pub id: String,
    /// 步骤描述
    pub description: String,
    /// 需要使用的工具
    pub tool_name: Option<String>,
    /// 工具参数
    pub tool_arguments: Option<Value>,
    /// 工具来源（MCP服务器ID或内置标识）
    pub tool_source: Option<String>,
    /// 步骤状态
    pub status: PlanningStepStatus,
    /// 执行结果
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 依赖的步骤ID列表
    pub dependencies: Vec<String>,
    /// 步骤优先级（1-10，10最高）
    pub priority: i32,
    /// 预估执行时间（秒）
    pub estimated_duration: Option<u64>,
    /// 实际执行时间（毫秒）
    pub actual_duration: Option<u64>,
}

/// 规划步骤状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanningStepStatus {
    /// 待执行
    Pending,
    /// 执行中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 跳过
    Skipped,
}

/// 任务规划结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    /// 规划ID
    pub id: String,
    /// 原始用户问题
    pub user_question: String,
    /// 规划目标
    pub goal: String,
    /// 规划步骤列表
    pub steps: Vec<PlanningStep>,
    /// 可用工具列表
    pub available_tools: Vec<ToolDefinitionPB>,
    /// 个性化特性
    pub personalization: PersonalizationFeatures,
    /// 规划状态
    pub status: PlanStatus,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 规划元数据
    pub metadata: HashMap<String, String>,
}

/// 规划状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    /// 规划中
    Planning,
    /// 就绪
    Ready,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 个性化特性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationFeatures {
    /// 用户偏好的工具类型
    pub preferred_tool_types: Vec<ToolTypePB>,
    /// 最大规划步骤数
    pub max_steps: i32,
    /// 最大工具调用次数
    pub max_tool_calls: i32,
    /// 是否启用并行执行
    pub enable_parallel_execution: bool,
    /// 用户技能水平（1-10）
    pub user_skill_level: i32,
    /// 详细程度偏好（1-5）
    pub detail_preference: i32,
    /// 风险承受度（1-5）
    pub risk_tolerance: i32,
}

impl Default for PersonalizationFeatures {
    fn default() -> Self {
        Self {
            preferred_tool_types: vec![ToolTypePB::Native, ToolTypePB::MCP],
            max_steps: 10,
            max_tool_calls: 20,
            enable_parallel_execution: true,
            user_skill_level: 5,
            detail_preference: 3,
            risk_tolerance: 3,
        }
    }
}

/// 规划重试配置
#[derive(Debug, Clone)]
pub struct PlanningRetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 指数退避因子
    pub backoff_factor: f64,
    /// 最大重试延迟（毫秒）
    pub max_retry_delay_ms: u64,
}

impl Default for PlanningRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            backoff_factor: 2.0,
            max_retry_delay_ms: 10000,
        }
    }
}

/// AI驱动的任务规划器
pub struct AITaskPlanner {
    /// AI管理器引用
    ai_manager: Arc<AIManager>,
    /// 工具注册表引用
    tool_registry: Option<Arc<ToolRegistry>>,
    /// 重试配置
    retry_config: PlanningRetryConfig,
    /// 规划超时时间
    planning_timeout: Duration,
}

impl AITaskPlanner {
    /// 创建新的任务规划器
    pub fn new(ai_manager: Arc<AIManager>) -> Self {
        Self {
            ai_manager,
            tool_registry: None,
            retry_config: PlanningRetryConfig::default(),
            planning_timeout: Duration::from_secs(60),
        }
    }

    /// 设置工具注册表
    pub fn with_tool_registry(mut self, tool_registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(tool_registry);
        self
    }

    /// 设置重试配置
    pub fn with_retry_config(mut self, config: PlanningRetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// 设置规划超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.planning_timeout = timeout;
        self
    }

    /// 基于用户问题生成任务规划
    #[tracing::instrument(level = "info", skip(self))]
    pub async fn create_plan(
        &self,
        user_question: &str,
        personalization: Option<PersonalizationFeatures>,
        workspace_id: &Uuid,
    ) -> FlowyResult<TaskPlan> {
        let start_time = Instant::now();
        info!("开始为用户问题创建任务规划: {}", user_question);

        let personalization = personalization.unwrap_or_default();
        
        // 获取可用工具
        let available_tools = self.get_available_tools().await?;
        info!("发现 {} 个可用工具", available_tools.len());

        // 使用重试机制生成规划
        let plan = self.generate_plan_with_retry(
            user_question,
            &available_tools,
            &personalization,
            workspace_id,
        ).await?;

        let duration = start_time.elapsed();
        info!("任务规划生成完成，耗时: {:?}ms", duration.as_millis());

        Ok(plan)
    }

    /// 带重试机制的规划生成
    async fn generate_plan_with_retry(
        &self,
        user_question: &str,
        available_tools: &[ToolDefinitionPB],
        personalization: &PersonalizationFeatures,
        workspace_id: &Uuid,
    ) -> FlowyResult<TaskPlan> {
        let mut last_error = None;
        let mut delay = self.retry_config.retry_delay_ms;

        for attempt in 0..=self.retry_config.max_retries {
            match self.generate_plan_internal(
                user_question,
                available_tools,
                personalization,
                workspace_id,
            ).await {
                Ok(plan) => {
                    if attempt > 0 {
                        info!("规划生成在第 {} 次重试后成功", attempt);
                    }
                    return Ok(plan);
                }
                Err(err) => {
                    last_error = Some(err);
                    if attempt < self.retry_config.max_retries {
                        warn!("规划生成失败，第 {} 次重试，延迟 {}ms", attempt + 1, delay);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        delay = (delay as f64 * self.retry_config.backoff_factor) as u64;
                        delay = delay.min(self.retry_config.max_retry_delay_ms);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            FlowyError::internal().with_context("规划生成失败，已达到最大重试次数")
        }))
    }

    /// 内部规划生成逻辑
    async fn generate_plan_internal(
        &self,
        user_question: &str,
        available_tools: &[ToolDefinitionPB],
        personalization: &PersonalizationFeatures,
        workspace_id: &Uuid,
    ) -> FlowyResult<TaskPlan> {
        // 构建规划提示词
        let planning_prompt = self.build_planning_prompt(
            user_question,
            available_tools,
            personalization,
        );

        // 获取AI模型
        let ai_model = self.ai_manager.get_active_model(&workspace_id.to_string()).await;
        debug!("使用AI模型进行规划: {:?}", ai_model);

        // 调用AI模型生成规划
        let response = self.call_ai_model(&planning_prompt, &ai_model, workspace_id).await?;
        
        // 解析AI响应为任务规划
        let mut plan = self.parse_ai_response(&response, user_question, available_tools)?;
        
        // 应用个性化特性
        self.apply_personalization(&mut plan, personalization);
        
        // 验证和优化规划
        self.validate_and_optimize_plan(&mut plan).await?;

        Ok(plan)
    }

    /// 构建规划提示词
    fn build_planning_prompt(
        &self,
        user_question: &str,
        available_tools: &[ToolDefinitionPB],
        personalization: &PersonalizationFeatures,
    ) -> String {
        let tools_description = available_tools
            .iter()
            .map(|tool| {
                format!(
                    "- {} ({}): {} [来源: {}]",
                    tool.name,
                    match tool.tool_type {
                        ToolTypePB::MCP => "MCP工具",
                        ToolTypePB::Native => "原生工具",
                        ToolTypePB::Search => "搜索工具",
                        ToolTypePB::ExternalAPI => "外部API",
                    },
                    tool.description,
                    tool.source
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"你是一个AI任务规划专家。请根据用户问题制定详细的执行计划。

用户问题: {}

可用工具:
{}

个性化设置:
- 最大步骤数: {}
- 最大工具调用次数: {}
- 用户技能水平: {}/10
- 详细程度偏好: {}/5
- 风险承受度: {}/5
- 并行执行: {}

请生成一个JSON格式的任务规划，包含以下结构:
{{
  "goal": "明确的目标描述",
  "steps": [
    {{
      "id": "step_1",
      "description": "步骤描述",
      "tool_name": "工具名称或null",
      "tool_arguments": {{"参数": "值"}} 或 null,
      "tool_source": "工具来源或null",
      "dependencies": ["依赖的步骤ID"],
      "priority": 1-10,
      "estimated_duration": 预估秒数或null
    }}
  ]
}}

规划原则:
1. 步骤要具体可执行
2. 合理选择和使用可用工具
3. 考虑步骤间的依赖关系
4. 根据用户技能水平调整复杂度
5. 优先使用用户偏好的工具类型
6. 控制步骤数量在限制范围内
7. 为每个步骤分配合理的优先级

请只返回JSON，不要包含其他文本。"#,
            user_question,
            tools_description,
            personalization.max_steps,
            personalization.max_tool_calls,
            personalization.user_skill_level,
            personalization.detail_preference,
            personalization.risk_tolerance,
            if personalization.enable_parallel_execution { "启用" } else { "禁用" }
        )
    }

    /// 调用AI模型
    async fn call_ai_model(
        &self,
        prompt: &str,
        ai_model: &AIModel,
        workspace_id: &Uuid,
    ) -> FlowyResult<String> {
        let params = CompleteTextParams {
            text: prompt.to_string(),
            completion_type: Some(CompletionType::AskAI),
            format: ResponseFormat::default(),
            metadata: None,
        };

        // 使用超时机制
        let completion_future = ChatCloudService::stream_complete(
            &*self.ai_manager.cloud_service_wm,
            workspace_id,
            params,
            ai_model.clone()
        );

        let mut stream = tokio::time::timeout(self.planning_timeout, completion_future)
            .await
            .map_err(|_| FlowyError::response_timeout().with_context("AI规划请求超时"))?
            .map_err(|e| FlowyError::internal().with_context(format!("AI模型调用失败: {}", e)))?;

        // 收集流式响应
        let mut response = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(value) => {
                    match value {
                        flowy_ai_pub::cloud::CompletionStreamValue::Answer { value } => {
                            response.push_str(&value);
                        }
                        _ => {} // 忽略其他类型的响应
                    }
                }
                Err(e) => {
                    return Err(FlowyError::internal().with_context(format!("流式响应错误: {}", e)));
                }
            }
        }

        Ok(response)
    }

    /// 解析AI响应
    fn parse_ai_response(
        &self,
        response: &str,
        user_question: &str,
        available_tools: &[ToolDefinitionPB],
    ) -> FlowyResult<TaskPlan> {
        // 尝试提取JSON部分
        let json_str = self.extract_json_from_response(response)?;
        
        // 解析JSON
        let parsed: Value = serde_json::from_str(&json_str)
            .map_err(|e| FlowyError::invalid_data().with_context(format!("JSON解析失败: {}", e)))?;

        // 提取目标
        let goal = parsed.get("goal")
            .and_then(|v| v.as_str())
            .unwrap_or("未指定目标")
            .to_string();

        // 解析步骤
        let steps_array = parsed.get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少steps数组"))?;

        let mut steps = Vec::new();
        for (index, step_value) in steps_array.iter().enumerate() {
            let step = self.parse_planning_step(step_value, index)?;
            steps.push(step);
        }

        // 验证工具引用
        self.validate_tool_references(&steps, available_tools)?;

        let now = chrono::Utc::now();
        let plan_id = Uuid::new_v4().to_string();

        Ok(TaskPlan {
            id: plan_id,
            user_question: user_question.to_string(),
            goal,
            steps,
            available_tools: available_tools.to_vec(),
            personalization: PersonalizationFeatures::default(), // 将在后续应用
            status: PlanStatus::Planning,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }

    /// 从响应中提取JSON
    fn extract_json_from_response(&self, response: &str) -> FlowyResult<String> {
        let response = response.trim();
        
        // 如果响应直接是JSON
        if response.starts_with('{') && response.ends_with('}') {
            return Ok(response.to_string());
        }

        // 尝试从markdown代码块中提取
        if let Some(start) = response.find("```json") {
            if let Some(end) = response[start..].find("```") {
                let json_start = start + 7; // "```json".len()
                let json_end = start + end;
                if json_start < json_end {
                    return Ok(response[json_start..json_end].trim().to_string());
                }
            }
        }

        // 尝试查找JSON对象
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if start <= end {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        Err(FlowyError::invalid_data().with_context("无法从AI响应中提取有效的JSON"))
    }

    /// 解析单个规划步骤
    fn parse_planning_step(&self, step_value: &Value, index: usize) -> FlowyResult<PlanningStep> {
        let id = step_value.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("step_{}", index + 1))
            .to_string();

        let description = step_value.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("步骤缺少description"))?
            .to_string();

        let tool_name = step_value.get("tool_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tool_arguments = step_value.get("tool_arguments").cloned();

        let tool_source = step_value.get("tool_source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let dependencies = step_value.get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let priority = step_value.get("priority")
            .and_then(|v| v.as_i64())
            .unwrap_or(5) as i32;

        let estimated_duration = step_value.get("estimated_duration")
            .and_then(|v| v.as_u64());

        Ok(PlanningStep {
            id,
            description,
            tool_name,
            tool_arguments,
            tool_source,
            status: PlanningStepStatus::Pending,
            result: None,
            error: None,
            dependencies,
            priority: priority.clamp(1, 10),
            estimated_duration,
            actual_duration: None,
        })
    }

    /// 验证工具引用
    fn validate_tool_references(
        &self,
        steps: &[PlanningStep],
        available_tools: &[ToolDefinitionPB],
    ) -> FlowyResult<()> {
        let tool_map: HashMap<String, &ToolDefinitionPB> = available_tools
            .iter()
            .map(|tool| (tool.name.clone(), tool))
            .collect();

        for step in steps {
            if let Some(tool_name) = &step.tool_name {
                if !tool_map.contains_key(tool_name) {
                    return Err(FlowyError::invalid_data()
                        .with_context(format!("步骤 '{}' 引用了不存在的工具: {}", step.id, tool_name)));
                }
            }
        }

        Ok(())
    }

    /// 应用个性化特性
    fn apply_personalization(&self, plan: &mut TaskPlan, personalization: &PersonalizationFeatures) {
        plan.personalization = personalization.clone();

        // 限制步骤数量
        if plan.steps.len() > personalization.max_steps as usize {
            plan.steps.truncate(personalization.max_steps as usize);
            warn!("规划步骤数量超过限制，已截断至 {} 步", personalization.max_steps);
        }

        // 根据用户技能水平调整步骤详细程度
        if personalization.user_skill_level < 5 {
            // 为初级用户添加更多说明
            for step in &mut plan.steps {
                if !step.description.contains("注意") && !step.description.contains("提示") {
                    step.description = format!("{} (提示: 这一步将帮助您完成任务的一部分)", step.description);
                }
            }
        }

        // 根据风险承受度调整工具选择
        if personalization.risk_tolerance < 3 {
            // 低风险承受度：移除可能有破坏性的工具调用
            for step in &mut plan.steps {
                if let Some(tool_name) = &step.tool_name {
                    if tool_name.contains("delete") || tool_name.contains("remove") || tool_name.contains("clear") {
                        step.tool_name = None;
                        step.tool_arguments = None;
                        step.description = format!("{} (已移除潜在风险操作，请手动执行)", step.description);
                    }
                }
            }
        }
    }

    /// 验证和优化规划
    async fn validate_and_optimize_plan(&self, plan: &mut TaskPlan) -> FlowyResult<()> {
        // 验证依赖关系
        self.validate_dependencies(plan)?;
        
        // 优化步骤顺序
        self.optimize_step_order(plan);
        
        // 设置规划状态为就绪
        plan.status = PlanStatus::Ready;
        plan.updated_at = chrono::Utc::now();

        info!("任务规划验证和优化完成，包含 {} 个步骤", plan.steps.len());
        Ok(())
    }

    /// 验证依赖关系
    fn validate_dependencies(&self, plan: &TaskPlan) -> FlowyResult<()> {
        let step_ids: std::collections::HashSet<String> = plan.steps
            .iter()
            .map(|step| step.id.clone())
            .collect();

        for step in &plan.steps {
            for dep in &step.dependencies {
                if !step_ids.contains(dep) {
                    return Err(FlowyError::invalid_data()
                        .with_context(format!("步骤 '{}' 依赖不存在的步骤: {}", step.id, dep)));
                }
            }
        }

        // 检查循环依赖
        if self.has_circular_dependencies(plan) {
            return Err(FlowyError::invalid_data().with_context("检测到循环依赖"));
        }

        Ok(())
    }

    /// 检查循环依赖
    fn has_circular_dependencies(&self, plan: &TaskPlan) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        let dep_map: HashMap<String, Vec<String>> = plan.steps
            .iter()
            .map(|step| (step.id.clone(), step.dependencies.clone()))
            .collect();

        for step in &plan.steps {
            if self.has_cycle_util(&step.id, &dep_map, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    /// 循环依赖检查辅助函数
    fn has_cycle_util(
        &self,
        step_id: &str,
        dep_map: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if rec_stack.contains(step_id) {
            return true;
        }

        if visited.contains(step_id) {
            return false;
        }

        visited.insert(step_id.to_string());
        rec_stack.insert(step_id.to_string());

        if let Some(deps) = dep_map.get(step_id) {
            for dep in deps {
                if self.has_cycle_util(dep, dep_map, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(step_id);
        false
    }

    /// 优化步骤顺序
    fn optimize_step_order(&self, plan: &mut TaskPlan) {
        // 根据依赖关系和优先级进行拓扑排序
        let mut sorted_steps = Vec::new();
        let mut remaining_steps: HashMap<String, PlanningStep> = plan.steps
            .drain(..)
            .map(|step| (step.id.clone(), step))
            .collect();

        while !remaining_steps.is_empty() {
            // 找到没有未满足依赖的步骤
            let mut ready_steps: Vec<_> = remaining_steps
                .values()
                .filter(|step| {
                    step.dependencies.iter().all(|dep| {
                        sorted_steps.iter().any(|s: &PlanningStep| s.id == *dep)
                    })
                })
                .cloned()
                .collect();

            if ready_steps.is_empty() {
                // 如果没有就绪的步骤，说明有循环依赖或其他问题
                // 将剩余步骤按优先级排序后添加
                let mut remaining: Vec<_> = remaining_steps.into_values().collect();
                remaining.sort_by(|a, b| b.priority.cmp(&a.priority));
                sorted_steps.extend(remaining);
                break;
            }

            // 按优先级排序就绪的步骤
            ready_steps.sort_by(|a, b| b.priority.cmp(&a.priority));

            // 添加优先级最高的步骤
            let next_step = ready_steps.into_iter().next().unwrap();
            remaining_steps.remove(&next_step.id);
            sorted_steps.push(next_step);
        }

        plan.steps = sorted_steps;
    }

    /// 获取可用工具列表
    async fn get_available_tools(&self) -> FlowyResult<Vec<ToolDefinitionPB>> {
        let mut tools = Vec::new();

        // 如果有工具注册表，从中获取工具
        if let Some(tool_registry) = &self.tool_registry {
            let all_tools = tool_registry.get_all_tools().await;
            for (_, type_tools) in all_tools {
                for (_, registered_tool) in type_tools {
                    if registered_tool.status == crate::agent::tool_registry::ToolStatus::Available {
                        tools.push(registered_tool.definition);
                    }
                }
            }
        } else {
            // 回退到旧的实现
            // 获取MCP工具
            let mcp_servers = self.ai_manager.mcp_manager.list_servers().await;
            for server in mcp_servers {
                if let Ok(tool_list) = self.ai_manager.mcp_manager.tool_list(&server.server_id).await {
                    for mcp_tool in tool_list.tools {
                        tools.push(self.convert_mcp_tool_to_definition(mcp_tool, &server.server_id));
                    }
                }
            }

            // 添加原生工具（这里可以扩展）
            tools.extend(self.get_native_tools());

            // 添加搜索工具
            tools.extend(self.get_search_tools());
        }

        info!("总共发现 {} 个可用工具", tools.len());
        Ok(tools)
    }

    /// 转换MCP工具为工具定义
    fn convert_mcp_tool_to_definition(&self, mcp_tool: MCPTool, server_id: &str) -> ToolDefinitionPB {
        ToolDefinitionPB {
            name: mcp_tool.name,
            description: mcp_tool.description,
            tool_type: ToolTypePB::MCP,
            source: server_id.to_string(),
            parameters_schema: serde_json::to_string(&mcp_tool.input_schema).unwrap_or_default(),
            permissions: Vec::new(), // MCP工具权限管理可以后续扩展
            is_available: true,
            metadata: HashMap::new(),
        }
    }

    /// 获取原生工具
    fn get_native_tools(&self) -> Vec<ToolDefinitionPB> {
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
    }

    /// 获取搜索工具
    fn get_search_tools(&self) -> Vec<ToolDefinitionPB> {
        vec![
            ToolDefinitionPB {
                name: "web_search".to_string(),
                description: "网络搜索".to_string(),
                tool_type: ToolTypePB::Search,
                source: "search_engine".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "搜索查询"},
                        "max_results": {"type": "integer", "description": "最大结果数", "default": 5}
                    },
                    "required": ["query"]
                }).to_string(),
                permissions: vec!["search.web".to_string()],
                is_available: true,
                metadata: HashMap::new(),
            },
        ]
    }

    /// 更新规划状态
    pub async fn update_plan_status(&self, plan: &mut TaskPlan, status: PlanStatus) {
        plan.status = status;
        plan.updated_at = chrono::Utc::now();
        debug!("任务规划 {} 状态更新为: {:?}", plan.id, plan.status);
    }

    /// 创建任务执行器
    pub fn create_executor(&self) -> crate::agent::executor::AITaskExecutor {
        crate::agent::executor::AITaskExecutor::new(self.ai_manager.clone())
    }

    /// 获取规划统计信息
    pub fn get_plan_statistics(&self, plan: &TaskPlan) -> HashMap<String, Value> {
        let mut stats = HashMap::new();
        
        let total_steps = plan.steps.len();
        let completed_steps = plan.steps.iter().filter(|s| s.status == PlanningStepStatus::Completed).count();
        let failed_steps = plan.steps.iter().filter(|s| s.status == PlanningStepStatus::Failed).count();
        let pending_steps = plan.steps.iter().filter(|s| s.status == PlanningStepStatus::Pending).count();
        
        let total_estimated_duration: u64 = plan.steps
            .iter()
            .filter_map(|s| s.estimated_duration)
            .sum();
            
        let total_actual_duration: u64 = plan.steps
            .iter()
            .filter_map(|s| s.actual_duration)
            .sum();

        stats.insert("total_steps".to_string(), json!(total_steps));
        stats.insert("completed_steps".to_string(), json!(completed_steps));
        stats.insert("failed_steps".to_string(), json!(failed_steps));
        stats.insert("pending_steps".to_string(), json!(pending_steps));
        stats.insert("completion_rate".to_string(), json!(if total_steps > 0 { completed_steps as f64 / total_steps as f64 } else { 0.0 }));
        stats.insert("total_estimated_duration_seconds".to_string(), json!(total_estimated_duration));
        stats.insert("total_actual_duration_ms".to_string(), json!(total_actual_duration));
        stats.insert("available_tools_count".to_string(), json!(plan.available_tools.len()));
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personalization_features_default() {
        let features = PersonalizationFeatures::default();
        assert_eq!(features.max_steps, 10);
        assert_eq!(features.max_tool_calls, 20);
        assert!(features.enable_parallel_execution);
        assert_eq!(features.user_skill_level, 5);
    }

    #[test]
    fn test_planning_step_status() {
        let step = PlanningStep {
            id: "test_step".to_string(),
            description: "Test step".to_string(),
            tool_name: None,
            tool_arguments: None,
            tool_source: None,
            status: PlanningStepStatus::Pending,
            result: None,
            error: None,
            dependencies: vec![],
            priority: 5,
            estimated_duration: Some(30),
            actual_duration: None,
        };

        assert_eq!(step.status, PlanningStepStatus::Pending);
        assert_eq!(step.priority, 5);
        assert_eq!(step.estimated_duration, Some(30));
    }

    #[test]
    fn test_plan_status() {
        let mut plan = TaskPlan {
            id: "test_plan".to_string(),
            user_question: "Test question".to_string(),
            goal: "Test goal".to_string(),
            steps: vec![],
            available_tools: vec![],
            personalization: PersonalizationFeatures::default(),
            status: PlanStatus::Planning,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(plan.status, PlanStatus::Planning);
        
        plan.status = PlanStatus::Ready;
        assert_eq!(plan.status, PlanStatus::Ready);
    }
}
