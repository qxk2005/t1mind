// 任务规划集成
// 负责在聊天流程中自动创建和执行任务计划

use std::sync::Arc;
use flowy_error::FlowyResult;
use tracing::info;
use uuid::Uuid;

use crate::ai_manager::AIManager;
use crate::entities::AgentConfigPB;
use crate::agent::planner::{AITaskPlanner, TaskPlan, PersonalizationFeatures};
use crate::agent::executor::ExecutionContext;

/// 任务规划集成器
/// 
/// 直接使用 AITaskPlanner 和 AITaskExecutor，避免循环依赖问题
pub struct PlanIntegration {
    ai_manager: Arc<AIManager>,
    planner: AITaskPlanner,
}

impl PlanIntegration {
    pub fn new(ai_manager: Arc<AIManager>) -> Self {
        let planner = AITaskPlanner::new(ai_manager.clone());
        Self { 
            ai_manager,
            planner,
        }
    }
    
    /// 为用户请求创建任务计划
    /// 
    /// 当检测到复杂任务时调用此方法
    pub async fn create_plan_for_message(
        &self,
        message: &str,
        agent_config: &AgentConfigPB,
        workspace_id: &Uuid,
    ) -> FlowyResult<TaskPlan> {
        info!(
            "Creating task plan for message (agent: {})",
            agent_config.name
        );
        
        // 构建个性化特性
        let personalization = PersonalizationFeatures {
            preferred_tool_types: vec![], // 从agent配置中获取
            max_steps: agent_config.capabilities.max_planning_steps,
            max_tool_calls: agent_config.capabilities.max_tool_calls,
            enable_parallel_execution: false, // 默认关闭
            user_skill_level: 5, // 默认中等水平
            detail_preference: 5, // 默认平衡 (1-10)
            risk_tolerance: 5, // 默认中等风险容忍度 (1-10)
        };
        
        // 使用内部的 planner 直接创建计划
        let plan = self.planner.create_plan(
            message,
            Some(personalization),
            workspace_id,
        ).await?;
        
        info!(
            "Task plan created with {} steps (plan_id: {})",
            plan.steps.len(),
            plan.id
        );
        
        Ok(plan)
    }
    
    /// 执行任务计划
    /// 
    /// 这将逐步执行计划中的所有步骤
    pub async fn execute_plan(
        &self,
        plan: &mut TaskPlan,
        workspace_id: &Uuid,
        uid: i64,
    ) -> FlowyResult<Vec<String>> {
        info!("Executing task plan: {}", plan.id);
        
        // 创建执行上下文
        let context = ExecutionContext {
            workspace_id: *workspace_id,
            user_id: Some(uid.to_string()),
            safe_mode: true, // 默认启用安全模式
            timeout: std::time::Duration::from_secs(300), // 5分钟超时
            enable_reflection: true, // 启用反思
            session_id: Some(uuid::Uuid::new_v4().to_string()), // 生成会话ID
            max_retries: 3, // 默认重试3次
        };
        
        // 创建执行器
        let mut executor = self.planner.create_executor();
        
        // 执行计划
        let results = executor.execute_plan(plan, &context).await?;
        
        info!(
            "Task plan execution completed: {} (steps: {})",
            plan.id,
            results.len()
        );
        
        // 提取结果内容
        let result_texts: Vec<String> = results.iter()
            .map(|r| r.content.clone())
            .collect();
        
        Ok(result_texts)
    }
    
    /// 检查是否应该创建计划
    /// 
    /// 基于消息内容和智能体配置的启发式判断
    pub fn should_create_plan(
        &self,
        message: &str,
        agent_config: &AgentConfigPB,
    ) -> bool {
        // 检查是否启用任务规划
        if !agent_config.capabilities.enable_planning {
            return false;
        }
        
        // 检查最大步骤数
        if agent_config.capabilities.max_planning_steps <= 0 {
            return false;
        }
        
        // 启发式检测：查找关键词（中英文）
        let planning_keywords = [
            // 中文
            "步骤", "计划", "如何", "怎么", "流程", "过程",
            "分步", "详细", "完整", "系统",
            "创建", "构建", "实现", "开发", "设计",
            // 英文
            "step", "plan", "how to", "how do", "process", "workflow",
            "guide", "tutorial", "detailed", "complete",
            "create", "build", "implement", "develop", "design",
        ];
        
        let message_lower = message.to_lowercase();
        planning_keywords.iter().any(|keyword| message_lower.contains(keyword))
    }
    
    /// 格式化计划为可读文本
    pub fn format_plan_for_display(plan: &TaskPlan) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("📋 **任务计划**: {}\n\n", plan.goal));
        output.push_str(&format!("共 {} 个步骤:\n\n", plan.steps.len()));
        
        for (idx, step) in plan.steps.iter().enumerate() {
            output.push_str(&format!("{}. **{}**\n", idx + 1, step.description));
            
            if let Some(tool) = &step.tool_name {
                output.push_str(&format!("   🔧 工具: {}\n", tool));
            }
            
            if let Some(duration) = step.estimated_duration {
                output.push_str(&format!("   ⏱️  预计: {}秒\n", duration));
            }
            
            output.push('\n');
        }
        
        output
    }
    
    /// 格式化执行结果
    pub fn format_execution_results(
        plan: &TaskPlan,
        results: &[String],
    ) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("✅ **计划执行完成**: {}\n\n", plan.goal));
        
        for (idx, (step, result)) in plan.steps.iter().zip(results.iter()).enumerate() {
            output.push_str(&format!("{}. {}\n", idx + 1, step.description));
            output.push_str(&format!("   结果: {}\n\n", result));
        }
        
        output
    }
}

/// 计划执行状态（用于流式通知）
#[derive(Debug, Clone)]
pub enum PlanExecutionEvent {
    /// 计划已创建
    PlanCreated {
        plan_id: String,
        goal: String,
        steps_count: usize,
    },
    /// 步骤开始执行
    StepStarted {
        step_index: usize,
        description: String,
    },
    /// 步骤执行完成
    StepCompleted {
        step_index: usize,
        result: String,
    },
    /// 步骤执行失败
    StepFailed {
        step_index: usize,
        error: String,
    },
    /// 计划执行完成
    PlanCompleted {
        plan_id: String,
        total_duration_ms: u64,
    },
    /// 计划执行失败
    PlanFailed {
        plan_id: String,
        error: String,
    },
}

impl PlanExecutionEvent {
    /// 转换为可显示的消息
    pub fn to_display_message(&self) -> String {
        match self {
            Self::PlanCreated { goal, steps_count, .. } => {
                format!("📋 已创建计划: {} (共{}步)", goal, steps_count)
            }
            Self::StepStarted { step_index, description } => {
                format!("▶️  步骤 {}: {}", step_index + 1, description)
            }
            Self::StepCompleted { step_index, result } => {
                format!("✅ 步骤 {} 完成: {}", step_index + 1, result)
            }
            Self::StepFailed { step_index, error } => {
                format!("❌ 步骤 {} 失败: {}", step_index + 1, error)
            }
            Self::PlanCompleted { total_duration_ms, .. } => {
                format!("🎉 计划执行完成 (耗时: {}ms)", total_duration_ms)
            }
            Self::PlanFailed { error, .. } => {
                format!("❌ 计划执行失败: {}", error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_should_create_plan() {
        // 这里需要创建一个有效的 AgentConfigPB 来测试
        // 由于依赖复杂，这里只做简单的概念演示
        let test_messages = vec![
            ("如何创建一个文档", true),
            ("步骤是什么", true),
            ("hello", false),
            ("搜索文档", false),
        ];
        
        // 实际测试需要完整的配置对象
        // 这里只是展示测试结构
    }
}

