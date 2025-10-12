use flowy_error::{FlowyError, FlowyResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn, debug};

/// 子任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    /// 子任务ID
    pub id: String,
    /// 子任务描述
    pub description: String,
    /// 推荐使用的工具类型
    pub recommended_tool: ToolType,
    /// 优先级（1-5，5最高）
    pub priority: u8,
    /// 是否可以并行执行
    pub can_parallel: bool,
}

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// 文档RAG检索
    DocumentRag,
    /// Web搜索
    WebSearch,
    /// MCP工具
    Mcp { server_id: String, tool_name: String },
    /// 原生工具
    Native { tool_name: String },
    /// 不需要工具（直接回答）
    None,
}

/// 任务分解结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDecomposition {
    /// 原始用户问题
    pub original_question: String,
    /// 是否需要分解（简单问题可能不需要）
    pub needs_decomposition: bool,
    /// 子任务列表
    pub sub_tasks: Vec<SubTask>,
    /// 分解推理过程
    pub reasoning: String,
}

/// 子任务分解器
/// 使用全局 AI 模型能力将用户问题分解为多个子任务
pub struct TaskDecomposer {
    /// OpenAI 兼容配置
    base_url: String,
    api_key: String,
    model: String,
}

impl TaskDecomposer {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
        }
    }

    /// 分解用户问题为子任务
    /// 
    /// # Arguments
    /// * `user_question` - 用户的原始问题
    /// * `available_tools` - 当前可用的工具列表
    /// * `context` - 可选的上下文信息（如对话历史）
    pub async fn decompose(
        &self,
        user_question: &str,
        available_tools: &AvailableTools,
        context: Option<&str>,
    ) -> FlowyResult<TaskDecomposition> {
        info!("🔀 [TASK-DECOMPOSE] Starting task decomposition for question: {}", user_question);
        
        // 构建系统提示词
        let system_prompt = self.build_decomposition_prompt(available_tools);
        
        // 构建用户消息
        let user_message = if let Some(ctx) = context {
            format!(
                r#"## Context
{}

## User Question
{}

Please analyze this question and decompose it into sub-tasks if needed."#,
                ctx, user_question
            )
        } else {
            format!(
                r#"## User Question
{}

Please analyze this question and decompose it into sub-tasks if needed."#,
                user_question
            )
        };
        
        // 调用 AI 进行分解
        let decomposition_json = self.call_ai_for_decomposition(
            &system_prompt,
            &user_message,
        ).await?;
        
        // 解析结果
        let mut decomposition: TaskDecomposition = serde_json::from_str(&decomposition_json)
            .map_err(|e| {
                warn!("❌ [TASK-DECOMPOSE] Failed to parse decomposition JSON: {}", e);
                FlowyError::internal().with_context(format!("Failed to parse task decomposition: {}", e))
            })?;
        
        decomposition.original_question = user_question.to_string();
        
        info!(
            "✅ [TASK-DECOMPOSE] Decomposition complete: {} sub-tasks, needs_decomposition={}",
            decomposition.sub_tasks.len(),
            decomposition.needs_decomposition
        );
        
        Ok(decomposition)
    }

    /// 构建任务分解的系统提示词
    fn build_decomposition_prompt(&self, available_tools: &AvailableTools) -> String {
        let mut tools_desc = String::new();
        
        // 文档RAG工具
        if available_tools.has_document_rag {
            tools_desc.push_str("\n- **DocumentRag**: Search internal documents and knowledge base. Best for questions about stored documents, notes, or internal information.");
        }
        
        // Web搜索工具
        if available_tools.has_web_search {
            tools_desc.push_str("\n- **WebSearch**: Search the internet for current information. Best for questions about recent events, external knowledge, or web content.");
        }
        
        // MCP工具
        if !available_tools.mcp_tools.is_empty() {
            tools_desc.push_str("\n- **MCP Tools**:");
            for (server_id, tools) in &available_tools.mcp_tools {
                for tool in tools {
                    tools_desc.push_str(&format!(
                        "\n  - `{}` (server: {}): {}",
                        tool.name,
                        server_id,
                        tool.description.as_deref().unwrap_or("No description")
                    ));
                }
            }
        }
        
        // 原生工具
        if !available_tools.native_tools.is_empty() {
            tools_desc.push_str("\n- **Native Tools**:");
            for tool_name in &available_tools.native_tools {
                tools_desc.push_str(&format!("\n  - `{}`", tool_name));
            }
        }
        
        format!(
            r#"You are an expert task decomposition assistant. Your job is to analyze user questions and determine:
1. Whether the question is simple enough to answer directly (needs_decomposition=false)
2. If complex, break it down into sub-tasks that can be executed in parallel

## Available Tools
{}

## Task Decomposition Guidelines

### Simple Questions (needs_decomposition=false)
- Single-source questions (only need one tool)
- Direct factual questions
- Questions that can be answered immediately
- Example: "What is in document X?" → Use DocumentRag only

### Complex Questions (needs_decomposition=true)
- Multi-source questions (need multiple tools)
- Questions requiring comparison or synthesis
- Questions with multiple parts
- Example: "Compare our internal pricing strategy with market trends" → DocumentRag + WebSearch

## Output Format (JSON ONLY, NO MARKDOWN)
{{
  "needs_decomposition": true/false,
  "sub_tasks": [
    {{
      "id": "task_1",
      "description": "Clear description of what to search/retrieve",
      "recommended_tool": {{"document_rag"}} or {{"web_search"}} or {{"mcp": {{"server_id": "...", "tool_name": "..."}}}},
      "priority": 5,
      "can_parallel": true
    }}
  ],
  "reasoning": "Explanation of your decomposition strategy"
}}

## Important Rules
1. ONLY output valid JSON, no markdown code blocks
2. If needs_decomposition=false, still include ONE sub-task
3. Each sub-task should be independent and executable in parallel
4. Assign priorities based on importance (1-5, 5 highest)
5. Be concise but clear in task descriptions
"#,
            tools_desc
        )
    }

    /// 调用 AI 进行任务分解
    async fn call_ai_for_decomposition(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> FlowyResult<String> {
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
        
        let payload = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_message
                }
            ],
            "temperature": 0.3,
            "response_format": { "type": "json_object" }
        });
        
        debug!("🔀 [TASK-DECOMPOSE] Calling AI with payload: {}", serde_json::to_string_pretty(&payload).unwrap());
        
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            warn!("❌ [TASK-DECOMPOSE] AI call failed: {} - {}", status, error_text);
            return Err(FlowyError::server_error()
                .with_context(format!("Task decomposition AI call failed: {}", status)));
        }
        
        let response_json: serde_json::Value = resp.json().await
            .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
        
        debug!("🔀 [TASK-DECOMPOSE] AI response: {}", serde_json::to_string_pretty(&response_json).unwrap());
        
        // 提取内容
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                FlowyError::internal().with_context("Invalid AI response format")
            })?;
        
        Ok(content.to_string())
    }
}

/// 可用工具列表
#[derive(Debug, Clone, Default)]
pub struct AvailableTools {
    /// 是否有文档RAG
    pub has_document_rag: bool,
    /// 是否有Web搜索
    pub has_web_search: bool,
    /// MCP工具列表 (server_id -> tools)
    pub mcp_tools: std::collections::HashMap<String, Vec<MCPToolInfo>>,
    /// 原生工具列表
    pub native_tools: Vec<String>,
}

/// MCP 工具信息
#[derive(Debug, Clone)]
pub struct MCPToolInfo {
    pub name: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_type_serialization() {
        let tool = ToolType::DocumentRag;
        let json = serde_json::to_string(&tool).unwrap();
        assert_eq!(json, r#""document_rag"#);

        let tool = ToolType::Mcp {
            server_id: "server1".to_string(),
            tool_name: "tool1".to_string(),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("mcp"));
    }
}

