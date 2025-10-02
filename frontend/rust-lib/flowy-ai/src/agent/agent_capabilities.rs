// 智能体能力集成模块
// 负责将智能体配置转化为实际的AI增强功能

use std::sync::Arc;
use flowy_error::FlowyResult;
use tracing::{info, debug};
use uuid::Uuid;

use crate::entities::{AgentConfigPB, AgentCapabilitiesPB};
use flowy_ai_pub::cloud::ChatMessage;
use flowy_ai_pub::cloud::chat_dto::{ChatAuthor, ChatAuthorType};
use flowy_ai_pub::persistence::{select_chat_messages};
use flowy_ai_pub::user_service::AIUserService;
use flowy_ai_pub::cloud::MessageCursor;

/// 智能体能力执行器
/// 根据智能体配置应用各种AI能力（记忆、规划、工具调用、反思）
pub struct AgentCapabilityExecutor {
    user_service: Arc<dyn AIUserService>,
}

impl AgentCapabilityExecutor {
    pub fn new(user_service: Arc<dyn AIUserService>) -> Self {
        Self { user_service }
    }

    /// 根据智能体的记忆配置加载对话历史
    /// 
    /// # 参数
    /// - `chat_id`: 聊天会话ID
    /// - `capabilities`: 智能体能力配置
    /// - `uid`: 用户ID
    /// 
    /// # 返回
    /// 限制数量的历史消息列表
    pub fn load_conversation_history(
        &self,
        chat_id: &Uuid,
        capabilities: &AgentCapabilitiesPB,
        uid: i64,
    ) -> FlowyResult<Vec<ChatMessage>> {
        if !capabilities.enable_memory || capabilities.memory_limit <= 0 {
            debug!("[Agent] Memory disabled or limit is 0, skipping history load");
            return Ok(Vec::new());
        }

        let limit = capabilities.memory_limit as u64;
        debug!("[Agent] Loading last {} messages from conversation history", limit);

        let conn = self.user_service.sqlite_connection(uid)?;
        let result = select_chat_messages(
            conn,
            &chat_id.to_string(),
            limit,
            MessageCursor::NextBack,
        )?;

        let history: Vec<ChatMessage> = result.messages.into_iter()
            .map(|record| ChatMessage {
                message_id: record.message_id,
                content: record.content,
                created_at: chrono::DateTime::from_timestamp(record.created_at, 0)
                    .unwrap_or_else(chrono::Utc::now),
                author: ChatAuthor {
                    author_type: if record.author_type == 1 { 
                        ChatAuthorType::Human 
                    } else if record.author_type == 3 {
                        ChatAuthorType::AI
                    } else { 
                        ChatAuthorType::System 
                    },
                    author_id: record.author_id.parse().unwrap_or(0),
                    meta: None,
                },
                reply_message_id: record.reply_message_id,
                metadata: record.metadata
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
            })
            .collect();

        info!("[Agent] Loaded {} messages from history", history.len());
        Ok(history)
    }

    /// 格式化对话历史为文本，用于附加到系统提示词
    pub fn format_conversation_history(&self, history: &[ChatMessage]) -> String {
        if history.is_empty() {
            return String::new();
        }

        let mut formatted = String::from("\n\n# Conversation History\n\n");
        formatted.push_str("Here are the recent messages in this conversation:\n\n");

        for (idx, msg) in history.iter().enumerate() {
            let role = match msg.author.author_type {
                ChatAuthorType::Human => "User",
                ChatAuthorType::System | ChatAuthorType::AI => "Assistant",
                ChatAuthorType::Unknown => "Unknown",
            };
            formatted.push_str(&format!(
                "{}. **{}**: {}\n",
                idx + 1,
                role,
                msg.content.trim()
            ));
        }

        formatted.push_str("\nUse this context to provide more relevant and coherent responses.\n");
        formatted
    }

    /// 构建增强的系统提示词
    /// 整合基础提示词、对话历史、工具使用说明等
    pub fn build_enhanced_system_prompt(
        &self,
        base_prompt: String,
        agent_config: &AgentConfigPB,
        conversation_history: &[ChatMessage],
    ) -> String {
        let mut prompt = base_prompt;

        // 添加对话历史
        if agent_config.capabilities.enable_memory && !conversation_history.is_empty() {
            let history_text = self.format_conversation_history(conversation_history);
            prompt.push_str(&history_text);
        }

        // 添加工具使用指南
        if agent_config.capabilities.enable_tool_calling && !agent_config.available_tools.is_empty() {
            prompt.push_str(&self.build_tool_usage_guide(agent_config));
        }

        // 添加任务规划指南
        if agent_config.capabilities.enable_planning {
            prompt.push_str(&self.build_planning_guide(agent_config));
        }

        // 添加反思指南
        if agent_config.capabilities.enable_reflection {
            prompt.push_str(&self.build_reflection_guide());
        }

        prompt
    }

    /// 构建工具使用指南
    fn build_tool_usage_guide(&self, config: &AgentConfigPB) -> String {
        let mut guide = String::from("\n\n# Tool Usage Guidelines\n\n");
        guide.push_str("You have access to the following tools:\n\n");

        for (idx, tool) in config.available_tools.iter().enumerate() {
            guide.push_str(&format!("{}. **{}**\n", idx + 1, tool));
        }

        guide.push_str(&format!(
            "\nYou can use up to {} tool calls in this conversation.\n",
            config.capabilities.max_tool_calls
        ));

        guide.push_str(
            "\nWhen you need to use a tool:\n\
            1. Clearly state which tool you're using\n\
            2. Explain why you need to use it\n\
            3. Provide the necessary parameters\n\
            4. Interpret the results for the user\n"
        );

        guide
    }

    /// 构建任务规划指南
    fn build_planning_guide(&self, config: &AgentConfigPB) -> String {
        format!(
            "\n\n# Task Planning Guidelines\n\n\
            For complex tasks, break them down into clear steps:\n\
            - Maximum {} steps allowed\n\
            - Each step should be specific and actionable\n\
            - Explain your planning process\n\
            - Execute steps systematically\n\
            - Validate results at each step\n",
            config.capabilities.max_planning_steps
        )
    }

    /// 构建反思指南
    fn build_reflection_guide(&self) -> String {
        String::from(
            "\n\n# Self-Reflection Guidelines\n\n\
            After generating responses:\n\
            - Review your answer for accuracy and completeness\n\
            - Consider alternative approaches\n\
            - Identify potential improvements\n\
            - If needed, refine your response before presenting it\n\
            - Be transparent about uncertainties\n"
        )
    }

    /// 检查是否应该执行任务规划
    pub fn should_create_plan(&self, capabilities: &AgentCapabilitiesPB, user_message: &str) -> bool {
        if !capabilities.enable_planning {
            return false;
        }

        // 简单的启发式规则：检测复杂任务的关键词
        let planning_keywords = [
            "步骤", "计划", "如何", "怎么", "流程", "过程",
            "step", "plan", "how to", "process", "workflow",
            "创建", "构建", "实现", "开发", "设计",
            "create", "build", "implement", "develop", "design"
        ];

        let message_lower = user_message.to_lowercase();
        planning_keywords.iter().any(|kw| message_lower.contains(kw))
    }

    /// 检查是否应该使用工具
    pub fn should_use_tools(&self, capabilities: &AgentCapabilitiesPB, user_message: &str) -> bool {
        if !capabilities.enable_tool_calling {
            return false;
        }

        // 简单的启发式规则：检测需要工具的关键词
        let tool_keywords = [
            "搜索", "查找", "计算", "分析", "数据",
            "search", "find", "calculate", "analyze", "data",
            "读取", "写入", "文件", "excel", "文档",
            "read", "write", "file", "excel", "document"
        ];

        let message_lower = user_message.to_lowercase();
        tool_keywords.iter().any(|kw| message_lower.contains(kw))
    }

    /// 检查是否应该应用反思
    pub fn should_apply_reflection(&self, capabilities: &AgentCapabilitiesPB) -> bool {
        capabilities.enable_reflection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_conversation_history() {
        // Mock test - would need actual implementation
        // Testing the format output structure
    }

    #[test]
    fn test_should_create_plan() {
        // Test planning detection heuristics
        let caps = AgentCapabilitiesPB {
            enable_planning: true,
            max_planning_steps: 10,
            ..Default::default()
        };

        // This would need a mock executor
        // let executor = AgentCapabilityExecutor::new(...);
        // assert!(executor.should_create_plan(&caps, "如何创建一个应用?"));
    }
}

