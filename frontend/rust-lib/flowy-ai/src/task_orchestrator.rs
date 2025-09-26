use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use dashmap::DashMap;
use flowy_error::{FlowyError, FlowyResult, ErrorCode};
use tokio::sync::{RwLock, Semaphore, mpsc};
use tracing::{debug, error, info, warn, instrument};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai_manager::AIManager;
use crate::mcp::manager::MCPClientManager;

/// 任务规划状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPlanStatus {
    /// 草稿状态，刚创建
    Draft,
    /// 等待用户确认
    PendingConfirmation,
    /// 用户已确认，准备执行
    Confirmed,
    /// 正在执行中
    Executing,
    /// 执行完成
    Completed,
    /// 执行失败
    Failed,
    /// 用户拒绝
    Rejected,
    /// 已取消
    Cancelled,
}

impl TaskPlanStatus {
    /// 是否可以执行
    pub fn can_execute(&self) -> bool {
        matches!(self, TaskPlanStatus::Confirmed)
    }
    
    /// 是否正在执行
    pub fn is_executing(&self) -> bool {
        matches!(self, TaskPlanStatus::Executing)
    }
    
    /// 是否已完成（成功或失败）
    pub fn is_finished(&self) -> bool {
        matches!(self, TaskPlanStatus::Completed | TaskPlanStatus::Failed | TaskPlanStatus::Rejected | TaskPlanStatus::Cancelled)
    }
}

/// 任务步骤状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStepStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Executing,
    /// 执行成功
    Completed,
    /// 执行失败
    Failed,
    /// 已跳过
    Skipped,
}

impl TaskStepStatus {
    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        matches!(self, TaskStepStatus::Completed | TaskStepStatus::Failed | TaskStepStatus::Skipped)
    }
    
    /// 是否成功
    pub fn is_successful(&self) -> bool {
        matches!(self, TaskStepStatus::Completed)
    }
}

/// 执行状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 空闲状态
    Idle,
    /// 准备中
    Preparing,
    /// 执行中
    Running,
    /// 暂停
    Paused,
    /// 完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

impl ExecutionStatus {
    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self, ExecutionStatus::Preparing | ExecutionStatus::Running)
    }
    
    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        matches!(self, ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled)
    }
}

/// 任务规划数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    /// 任务规划唯一标识
    pub id: String,
    /// 用户原始查询
    pub user_query: String,
    /// 整体策略描述
    pub overall_strategy: String,
    /// 任务步骤列表
    pub steps: Vec<TaskStep>,
    /// 所需的MCP工具ID列表
    pub required_mcp_tools: Vec<String>,
    /// 创建时间
    pub created_at: SystemTime,
    /// 任务状态
    pub status: TaskPlanStatus,
    /// 预估执行时间（秒）
    pub estimated_duration_seconds: u64,
    /// 智能体ID
    pub agent_id: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 更新时间
    pub updated_at: Option<SystemTime>,
}

/// 任务步骤数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    /// 步骤唯一标识
    pub id: String,
    /// 步骤描述
    pub description: String,
    /// 使用的MCP工具ID
    pub mcp_tool_id: String,
    /// 工具调用参数
    pub parameters: HashMap<String, Value>,
    /// 依赖的步骤ID列表
    pub dependencies: Vec<String>,
    /// 步骤状态
    pub status: TaskStepStatus,
    /// 预估执行时间（秒）
    pub estimated_duration_seconds: u64,
    /// 步骤顺序
    pub order: u32,
    /// 执行结果
    pub result: Option<HashMap<String, Value>>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 开始时间
    pub start_time: Option<SystemTime>,
    /// 结束时间
    pub end_time: Option<SystemTime>,
}

/// 智能体配置数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 智能体唯一标识
    pub id: String,
    /// 智能体名称
    pub name: String,
    /// 个性描述
    pub personality: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 允许使用的工具ID列表（白名单）
    pub allowed_tools: Vec<String>,
    /// 禁止使用的工具ID列表（黑名单）
    pub denied_tools: Vec<String>,
    /// 语言偏好
    pub language_preference: String,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: Option<SystemTime>,
    /// 是否启用
    pub is_enabled: bool,
    /// 最大并发工具调用数
    pub max_concurrent_tools: u32,
    /// 工具调用超时时间（秒）
    pub tool_timeout_seconds: u64,
    /// 智能体描述
    pub description: String,
    /// 智能体头像URL
    pub avatar_url: Option<String>,
}

/// 执行进度数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    /// 当前步骤索引
    pub current_step: u32,
    /// 总步骤数
    pub total_steps: u32,
    /// 当前步骤描述
    pub current_step_description: String,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 开始时间
    pub start_time: Option<SystemTime>,
    /// 预估剩余时间（秒）
    pub estimated_remaining_seconds: Option<u64>,
    /// 错误信息
    pub error_message: Option<String>,
}

impl ExecutionProgress {
    /// 计算进度百分比
    pub fn percentage(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.current_step as f64 / self.total_steps as f64
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.current_step >= self.total_steps
    }
}

/// 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub session_id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub result: Option<HashMap<String, Value>>,
    pub error_message: Option<String>,
    pub execution_log: Option<Value>, // 简化的执行日志
}

impl ExecutionResult {
    /// 是否成功
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Completed)
    }

    /// 是否失败
    pub fn is_failure(&self) -> bool {
        matches!(self.status, ExecutionStatus::Failed)
    }
}

/// 任务编排器
pub struct TaskOrchestrator {
    /// AI管理器引用
    ai_manager: Arc<AIManager>,
    /// MCP客户端管理器引用
    mcp_manager: Arc<MCPClientManager>,
    /// 活跃的任务规划
    active_plans: Arc<DashMap<String, TaskPlan>>,
    /// 活跃的执行上下文
    active_executions: Arc<DashMap<String, ExecutionContext>>,
    /// 智能体配置
    agent_configs: Arc<RwLock<HashMap<String, AgentConfig>>>,
    /// 并发控制信号量
    concurrency_semaphore: Arc<Semaphore>,
    /// 进度通知发送器
    progress_sender: Arc<RwLock<Option<mpsc::UnboundedSender<ExecutionProgress>>>>,
}

impl TaskOrchestrator {
    /// 创建新的任务编排器实例
    pub fn new(
        ai_manager: Arc<AIManager>,
        mcp_manager: Arc<MCPClientManager>,
        max_concurrent_executions: usize,
    ) -> Self {
        Self {
            ai_manager,
            mcp_manager,
            active_plans: Arc::new(DashMap::new()),
            active_executions: Arc::new(DashMap::new()),
            agent_configs: Arc::new(RwLock::new(HashMap::new())),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent_executions)),
            progress_sender: Arc::new(RwLock::new(None)),
        }
    }

    /// 设置进度通知接收器
    pub async fn set_progress_receiver(&self, sender: mpsc::UnboundedSender<ExecutionProgress>) {
        let mut progress_sender = self.progress_sender.write().await;
        *progress_sender = Some(sender);
    }

    /// 发送进度通知
    async fn notify_progress(&self, progress: ExecutionProgress) {
        let progress_sender = self.progress_sender.read().await;
        if let Some(sender) = progress_sender.as_ref() {
            if let Err(e) = sender.send(progress) {
                warn!("Failed to send progress notification: {}", e);
            }
        }
    }

    /// 创建任务规划
    #[instrument(skip(self), err)]
    pub async fn create_task_plan(
        &self,
        user_query: String,
        session_id: Option<String>,
        agent_id: Option<String>,
    ) -> FlowyResult<TaskPlan> {
        let plan_id = Uuid::new_v4().to_string();
        
        info!("Creating task plan {} for query: {}", plan_id, user_query);

        // 使用AI管理器生成任务规划
        let strategy = self.generate_strategy(&user_query, agent_id.as_deref()).await?;
        let steps = self.generate_steps(&user_query, &strategy, agent_id.as_deref()).await?;
        let required_tools = self.extract_required_tools(&steps);

        let plan = TaskPlan {
            id: plan_id.clone(),
            user_query,
            overall_strategy: strategy,
            steps,
            required_mcp_tools: required_tools,
            created_at: SystemTime::now(),
            status: TaskPlanStatus::Draft,
            estimated_duration_seconds: 0, // 将在后续计算
            agent_id,
            session_id,
            error_message: None,
            updated_at: None,
        };

        // 计算预估执行时间
        let estimated_duration = self.calculate_estimated_duration(&plan.steps);
        let mut plan = plan;
        plan.estimated_duration_seconds = estimated_duration;

        // 存储任务规划
        self.active_plans.insert(plan_id.clone(), plan.clone());

        info!("Task plan {} created successfully with {} steps", plan_id, plan.steps.len());
        Ok(plan)
    }

    /// 确认任务规划
    #[instrument(skip(self), err)]
    pub async fn confirm_task_plan(&self, plan_id: &str) -> FlowyResult<()> {
        let mut plan = self.active_plans.get_mut(plan_id)
            .ok_or_else(|| FlowyError::new(ErrorCode::RecordNotFound, "Task plan not found"))?;
        
        if plan.status != TaskPlanStatus::Draft && plan.status != TaskPlanStatus::PendingConfirmation {
            return Err(FlowyError::new(ErrorCode::InvalidParams, "Task plan cannot be confirmed in current status"));
        }

        plan.status = TaskPlanStatus::Confirmed;
        plan.updated_at = Some(SystemTime::now());

        info!("Task plan {} confirmed", plan_id);
        Ok(())
    }

    /// 执行任务规划
    #[instrument(skip(self), err)]
    pub async fn execute_task_plan(
        &self,
        plan_id: &str,
        context: ExecutionContext,
    ) -> FlowyResult<ExecutionResult> {
        // 获取并发许可
        let _permit = self.concurrency_semaphore.acquire().await
            .map_err(|_| FlowyError::new(ErrorCode::Internal, "Failed to acquire execution permit"))?;

        let mut plan = self.active_plans.get_mut(plan_id)
            .ok_or_else(|| FlowyError::new(ErrorCode::RecordNotFound, "Task plan not found"))?;

        if !plan.status.can_execute() {
            return Err(FlowyError::new(ErrorCode::InvalidParams, "Task plan is not ready for execution"));
        }

        plan.status = TaskPlanStatus::Executing;
        plan.updated_at = Some(SystemTime::now());

        let execution_id = Uuid::new_v4().to_string();
        self.active_executions.insert(execution_id.clone(), context.clone());

        info!("Starting execution of task plan {} with execution ID {}", plan_id, execution_id);

        // 发送初始进度通知
        let initial_progress = ExecutionProgress {
            current_step: 0,
            total_steps: plan.steps.len() as u32,
            current_step_description: "准备执行".to_string(),
            status: ExecutionStatus::Preparing,
            start_time: Some(SystemTime::now()),
            estimated_remaining_seconds: Some(plan.estimated_duration_seconds),
            error_message: None,
        };
        self.notify_progress(initial_progress).await;

        // 执行步骤
        let execution_result = self.execute_steps(&mut plan, &execution_id, &context).await;

        // 更新最终状态
        match &execution_result {
            Ok(result) => {
                plan.status = if result.is_success() {
                    TaskPlanStatus::Completed
                } else {
                    TaskPlanStatus::Failed
                };
            }
            Err(_) => {
                plan.status = TaskPlanStatus::Failed;
            }
        }
        plan.updated_at = Some(SystemTime::now());

        // 清理执行上下文
        self.active_executions.remove(&execution_id);

        // 发送最终进度通知
        let final_progress = ExecutionProgress {
            current_step: plan.steps.len() as u32,
            total_steps: plan.steps.len() as u32,
            current_step_description: "执行完成".to_string(),
            status: if execution_result.is_ok() { ExecutionStatus::Completed } else { ExecutionStatus::Failed },
            start_time: Some(SystemTime::now()),
            estimated_remaining_seconds: Some(0),
            error_message: execution_result.as_ref().err().map(|e| e.to_string()),
        };
        self.notify_progress(final_progress).await;

        match execution_result {
            Ok(result) => {
                info!("Task plan {} execution completed successfully", plan_id);
                Ok(result)
            }
            Err(e) => {
                error!("Task plan {} execution failed: {}", plan_id, e);
                Err(e)
            }
        }
    }

    /// 取消任务执行
    #[instrument(skip(self), err)]
    pub async fn cancel_task_execution(&self, plan_id: &str) -> FlowyResult<()> {
        let mut plan = self.active_plans.get_mut(plan_id)
            .ok_or_else(|| FlowyError::new(ErrorCode::RecordNotFound, "Task plan not found"))?;

        if !plan.status.is_executing() {
            return Err(FlowyError::new(ErrorCode::InvalidParams, "Task plan is not executing"));
        }

        plan.status = TaskPlanStatus::Cancelled;
        plan.updated_at = Some(SystemTime::now());

        info!("Task plan {} execution cancelled", plan_id);
        Ok(())
    }

    /// 获取任务规划
    pub async fn get_task_plan(&self, plan_id: &str) -> FlowyResult<TaskPlan> {
        self.active_plans.get(plan_id)
            .map(|plan| plan.clone())
            .ok_or_else(|| FlowyError::new(ErrorCode::RecordNotFound, "Task plan not found"))
    }

    /// 获取所有活跃的任务规划
    pub async fn get_active_task_plans(&self) -> Vec<TaskPlan> {
        self.active_plans.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 添加智能体配置
    pub async fn add_agent_config(&self, config: AgentConfig) -> FlowyResult<()> {
        let mut configs = self.agent_configs.write().await;
        configs.insert(config.id.clone(), config);
        Ok(())
    }

    /// 获取智能体配置
    pub async fn get_agent_config(&self, agent_id: &str) -> FlowyResult<AgentConfig> {
        let configs = self.agent_configs.read().await;
        configs.get(agent_id)
            .cloned()
            .ok_or_else(|| FlowyError::new(ErrorCode::RecordNotFound, "Agent config not found"))
    }

    /// 生成策略描述
    async fn generate_strategy(&self, user_query: &str, agent_id: Option<&str>) -> FlowyResult<String> {
        // 这里应该调用AI服务生成策略
        // 暂时返回一个简单的策略描述
        let strategy = format!("为查询 '{}' 制定执行策略", user_query);
        
        if let Some(agent_id) = agent_id {
            if let Ok(agent_config) = self.get_agent_config(agent_id).await {
                return Ok(format!("{} (使用智能体: {})", strategy, agent_config.name));
            }
        }
        
        Ok(strategy)
    }

    /// 生成执行步骤
    async fn generate_steps(&self, user_query: &str, strategy: &str, _agent_id: Option<&str>) -> FlowyResult<Vec<TaskStep>> {
        // 这里应该调用AI服务生成具体的执行步骤
        // 暂时返回一些示例步骤
        let mut steps = Vec::new();
        
        // 示例步骤1：分析用户查询
        steps.push(TaskStep {
            id: Uuid::new_v4().to_string(),
            description: format!("分析用户查询: {}", user_query),
            mcp_tool_id: "analysis_tool".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("query".to_string(), json!(user_query));
                params
            },
            dependencies: vec![],
            status: TaskStepStatus::Pending,
            estimated_duration_seconds: 5,
            order: 1,
            result: None,
            error_message: None,
            start_time: None,
            end_time: None,
        });

        // 示例步骤2：执行策略
        steps.push(TaskStep {
            id: Uuid::new_v4().to_string(),
            description: format!("执行策略: {}", strategy),
            mcp_tool_id: "execution_tool".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("strategy".to_string(), json!(strategy));
                params
            },
            dependencies: vec![steps[0].id.clone()],
            status: TaskStepStatus::Pending,
            estimated_duration_seconds: 10,
            order: 2,
            result: None,
            error_message: None,
            start_time: None,
            end_time: None,
        });

        Ok(steps)
    }

    /// 提取所需的MCP工具
    fn extract_required_tools(&self, steps: &[TaskStep]) -> Vec<String> {
        steps.iter()
            .map(|step| step.mcp_tool_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// 计算预估执行时间
    fn calculate_estimated_duration(&self, steps: &[TaskStep]) -> u64 {
        steps.iter()
            .map(|step| step.estimated_duration_seconds)
            .sum()
    }

    /// 执行所有步骤
    async fn execute_steps(
        &self,
        plan: &mut TaskPlan,
        execution_id: &str,
        context: &ExecutionContext,
    ) -> FlowyResult<ExecutionResult> {
        let mut executed_steps = 0;
        let total_steps = plan.steps.len();
        
        // 预先计算每个步骤的剩余时间，避免借用冲突
        let remaining_times: Vec<u64> = (0..total_steps)
            .map(|index| {
                plan.steps[index..].iter()
                    .map(|s| s.estimated_duration_seconds)
                    .sum()
            })
            .collect();
        
        for (index, step) in plan.steps.iter_mut().enumerate() {
            // 检查是否被取消
            if plan.status == TaskPlanStatus::Cancelled {
                return Ok(ExecutionResult {
                    execution_id: execution_id.to_string(),
                    status: ExecutionStatus::Cancelled,
                    result: None,
                    error_message: Some("Execution was cancelled".to_string()),
                    execution_log: None,
                });
            }

            // 发送步骤开始通知
            let step_progress = ExecutionProgress {
                current_step: index as u32,
                total_steps: total_steps as u32,
                current_step_description: step.description.clone(),
                status: ExecutionStatus::Running,
                start_time: Some(SystemTime::now()),
                estimated_remaining_seconds: Some(remaining_times[index]),
                error_message: None,
            };
            self.notify_progress(step_progress).await;

            // 执行步骤
            match self.execute_single_step(step, context).await {
                Ok(_) => {
                    executed_steps += 1;
                    step.status = TaskStepStatus::Completed;
                    step.end_time = Some(SystemTime::now());
                }
                Err(e) => {
                    step.status = TaskStepStatus::Failed;
                    step.error_message = Some(e.to_string());
                    step.end_time = Some(SystemTime::now());
                    
                    return Ok(ExecutionResult {
                        execution_id: execution_id.to_string(),
                        status: ExecutionStatus::Failed,
                        result: None,
                        error_message: Some(format!("Step {} failed: {}", step.description, e)),
                        execution_log: None,
                    });
                }
            }
        }

        Ok(ExecutionResult {
            execution_id: execution_id.to_string(),
            status: ExecutionStatus::Completed,
            result: Some({
                let mut result = HashMap::new();
                result.insert("executed_steps".to_string(), json!(executed_steps));
                result.insert("total_steps".to_string(), json!(total_steps));
                result
            }),
            error_message: None,
            execution_log: None,
        })
    }

    /// 执行单个步骤
    async fn execute_single_step(
        &self,
        step: &mut TaskStep,
        _context: &ExecutionContext,
    ) -> FlowyResult<()> {
        step.status = TaskStepStatus::Executing;
        step.start_time = Some(SystemTime::now());

        debug!("Executing step: {} with tool: {}", step.description, step.mcp_tool_id);

        // 这里应该调用实际的MCP工具
        // 暂时模拟执行
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 模拟执行结果
        let mut result = HashMap::new();
        result.insert("status".to_string(), json!("success"));
        result.insert("message".to_string(), json!(format!("Step '{}' completed successfully", step.description)));
        step.result = Some(result);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_task_orchestrator_creation() {
        // 这里需要模拟AI管理器和MCP管理器
        // 由于依赖复杂，暂时跳过具体测试实现
    }

    #[test]
    fn test_task_plan_status() {
        assert!(TaskPlanStatus::Confirmed.can_execute());
        assert!(!TaskPlanStatus::Draft.can_execute());
        assert!(TaskPlanStatus::Executing.is_executing());
        assert!(TaskPlanStatus::Completed.is_finished());
    }

    #[test]
    fn test_execution_progress_percentage() {
        let progress = ExecutionProgress {
            current_step: 3,
            total_steps: 10,
            current_step_description: "Test".to_string(),
            status: ExecutionStatus::Running,
            start_time: None,
            estimated_remaining_seconds: None,
            error_message: None,
        };
        
        assert_eq!(progress.percentage(), 0.3);
        assert!(!progress.is_completed());
    }
}
