use std::sync::Arc;
use std::collections::HashMap;

use flowy_error::FlowyResult;
use serde_json::Value;
use tracing::{info, warn};
use uuid::Uuid;

use crate::ai_manager::AIManager;
use crate::agent::planner::{AITaskPlanner, TaskPlan, PlanStatus, PersonalizationFeatures};
use crate::agent::executor::{AITaskExecutor, ExecutionContext, ExecutionResult};

/// 智能体管理器 - 集成规划器和执行器
pub struct AgentManager {
    /// AI管理器引用
    ai_manager: Arc<AIManager>,
    /// 任务规划器
    planner: AITaskPlanner,
    /// 活跃的任务计划
    active_plans: HashMap<String, TaskPlan>,
}

impl AgentManager {
    /// 创建新的智能体管理器
    pub fn new(ai_manager: Arc<AIManager>) -> Self {
        let planner = AITaskPlanner::new(ai_manager.clone());
        
        Self {
            ai_manager,
            planner,
            active_plans: HashMap::new(),
        }
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
        let mut executor = self.planner.create_executor();
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

        let mut executor = self.planner.create_executor();
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
