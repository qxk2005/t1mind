use super::task_decomposer::{SubTask, ToolType};
use super::tool_call_handler::{ToolCallHandler, ToolCallRequest, ToolCallResponse};
use crate::entities::AgentConfigPB;
use flowy_error::{FlowyError, FlowyResult};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, debug};
use uuid::Uuid;

/// 子任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskResult {
    /// 子任务ID
    pub task_id: String,
    /// 子任务描述
    pub task_description: String,
    /// 使用的工具
    pub tool_used: String,
    /// 执行是否成功
    pub success: bool,
    /// 结果内容
    pub content: String,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 数据来源标识
    pub source: String,
}

/// 并行调度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    /// 所有子任务结果
    pub results: Vec<SubTaskResult>,
    /// 成功的任务数
    pub success_count: usize,
    /// 失败的任务数
    pub failure_count: usize,
    /// 总耗时（毫秒）
    pub total_duration_ms: u64,
}

/// 并行调度器
/// 负责并行执行多个子任务
pub struct ParallelDispatcher {
    tool_handler: Arc<ToolCallHandler>,
    /// RAG文档检索（chat_id -> rag_ids）
    rag_context: Option<RagContext>,
}

/// RAG上下文
#[derive(Clone)]
pub struct RagContext {
    pub chat_id: Uuid,
    pub workspace_id: Uuid,
}

impl ParallelDispatcher {
    pub fn new(tool_handler: Arc<ToolCallHandler>) -> Self {
        Self {
            tool_handler,
            rag_context: None,
        }
    }

    /// 设置 RAG 上下文
    pub fn with_rag_context(mut self, chat_id: Uuid, workspace_id: Uuid) -> Self {
        self.rag_context = Some(RagContext {
            chat_id,
            workspace_id,
        });
        self
    }

    /// 并行执行所有子任务
    pub async fn dispatch_parallel(
        &self,
        sub_tasks: Vec<SubTask>,
        agent_config: Option<&AgentConfigPB>,
    ) -> FlowyResult<DispatchResult> {
        info!(
            "🚀 [PARALLEL-DISPATCH] Starting parallel execution of {} tasks",
            sub_tasks.len()
        );

        let start_time = std::time::Instant::now();

        // 按优先级排序（可选，如果想要优先处理高优先级任务）
        let mut sorted_tasks = sub_tasks.clone();
        sorted_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 将任务分组：可并行 vs 需串行
        let (parallel_tasks, serial_tasks): (Vec<_>, Vec<_>) = sorted_tasks
            .into_iter()
            .partition(|t| t.can_parallel);

        let mut all_results = Vec::new();

        // 1. 并行执行所有可并行任务
        if !parallel_tasks.is_empty() {
            info!(
                "⚡ [PARALLEL-DISPATCH] Executing {} tasks in parallel",
                parallel_tasks.len()
            );

            let futures: Vec<_> = parallel_tasks
                .into_iter()
                .map(|task| {
                    let handler = self.tool_handler.clone();
                    let agent_cfg = agent_config.cloned();
                    let rag_ctx = self.rag_context.clone();
                    
                    async move {
                        Self::execute_single_task(task, handler, agent_cfg.as_ref(), rag_ctx).await
                    }
                })
                .collect();

            let parallel_results = join_all(futures).await;
            all_results.extend(parallel_results);
        }

        // 2. 串行执行需要串行的任务（如果有）
        if !serial_tasks.is_empty() {
            info!(
                "🔗 [PARALLEL-DISPATCH] Executing {} tasks serially",
                serial_tasks.len()
            );

            for task in serial_tasks {
                let result = Self::execute_single_task(
                    task,
                    self.tool_handler.clone(),
                    agent_config,
                    self.rag_context.clone(),
                )
                .await;
                all_results.push(result);
            }
        }

        let total_duration = start_time.elapsed().as_millis() as u64;

        // 统计结果
        let success_count = all_results.iter().filter(|r| r.success).count();
        let failure_count = all_results.len() - success_count;

        info!(
            "✅ [PARALLEL-DISPATCH] Dispatch complete: {} success, {} failed, {}ms total",
            success_count, failure_count, total_duration
        );

        Ok(DispatchResult {
            results: all_results,
            success_count,
            failure_count,
            total_duration_ms: total_duration,
        })
    }

    /// 执行单个子任务
    async fn execute_single_task(
        task: SubTask,
        tool_handler: Arc<ToolCallHandler>,
        agent_config: Option<&AgentConfigPB>,
        rag_context: Option<RagContext>,
    ) -> SubTaskResult {
        let task_id = task.id.clone();
        let task_desc = task.description.clone();
        
        info!("🔧 [SUB-TASK] Executing: {} - {}", task_id, task_desc);
        
        let start_time = std::time::Instant::now();

        // 根据工具类型构建请求
        let result = match &task.recommended_tool {
            ToolType::DocumentRag => {
                Self::execute_document_rag(&task, rag_context).await
            }
            ToolType::WebSearch => {
                Self::execute_web_search(&task, &tool_handler, agent_config).await
            }
            ToolType::Mcp { server_id, tool_name } => {
                Self::execute_mcp_tool(&task, server_id, tool_name, &tool_handler, agent_config).await
            }
            ToolType::Native { tool_name } => {
                Self::execute_native_tool(&task, tool_name, &tool_handler, agent_config).await
            }
            ToolType::None => {
                // 不需要工具，直接返回任务描述
                Ok(("No tool needed, question can be answered directly".to_string(), "none".to_string()))
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok((content, source)) => {
                info!("✅ [SUB-TASK] {} completed in {}ms", task_id, duration_ms);
                SubTaskResult {
                    task_id,
                    task_description: task_desc,
                    tool_used: format!("{:?}", task.recommended_tool),
                    success: true,
                    content,
                    error: None,
                    duration_ms,
                    source,
                }
            }
            Err(e) => {
                warn!("❌ [SUB-TASK] {} failed: {}", task_id, e);
                SubTaskResult {
                    task_id,
                    task_description: task_desc,
                    tool_used: format!("{:?}", task.recommended_tool),
                    success: false,
                    content: String::new(),
                    error: Some(e.to_string()),
                    duration_ms,
                    source: "error".to_string(),
                }
            }
        }
    }

    /// 执行文档RAG检索
    async fn execute_document_rag(
        task: &SubTask,
        rag_context: Option<RagContext>,
    ) -> FlowyResult<(String, String)> {
        if let Some(_ctx) = rag_context {
            // RAG检索通过在主流程中已经完成，这里标记即可
            // 实际的RAG检索在 chat_service_mw.rs 中的 get_message_content_with_rag 完成
            Ok((
                format!("Document RAG search for: {}", task.description),
                "document_rag".to_string()
            ))
        } else {
            Err(FlowyError::internal().with_context("RAG context not available"))
        }
    }

    /// 执行Web搜索
    async fn execute_web_search(
        task: &SubTask,
        tool_handler: &Arc<ToolCallHandler>,
        agent_config: Option<&AgentConfigPB>,
    ) -> FlowyResult<(String, String)> {
        let request = ToolCallRequest {
            id: format!("web_search_{}", task.id),
            tool_name: "web_search".to_string(),
            arguments: serde_json::json!({
                "query": task.description
            }),
            source: None, // 自动检测
        };

        let response = tool_handler.execute_tool_call(&request, agent_config).await;
        
        if response.success {
            Ok((
                response.result.unwrap_or_else(|| "Search completed".to_string()),
                "web_search".to_string()
            ))
        } else {
            Err(FlowyError::internal().with_context(
                response.error.unwrap_or_else(|| "Web search failed".to_string())
            ))
        }
    }

    /// 执行MCP工具
    async fn execute_mcp_tool(
        task: &SubTask,
        server_id: &str,
        tool_name: &str,
        tool_handler: &Arc<ToolCallHandler>,
        agent_config: Option<&AgentConfigPB>,
    ) -> FlowyResult<(String, String)> {
        // 从任务描述中提取参数（简单实现）
        let arguments = serde_json::json!({
            "query": task.description
        });

        let request = ToolCallRequest {
            id: format!("mcp_{}_{}", server_id, task.id),
            tool_name: tool_name.to_string(),
            arguments,
            source: Some(server_id.to_string()),
        };

        let response = tool_handler.execute_tool_call(&request, agent_config).await;
        
        if response.success {
            Ok((
                response.result.unwrap_or_else(|| "MCP tool executed".to_string()),
                format!("mcp:{}", server_id)
            ))
        } else {
            Err(FlowyError::internal().with_context(
                response.error.unwrap_or_else(|| "MCP tool execution failed".to_string())
            ))
        }
    }

    /// 执行原生工具
    async fn execute_native_tool(
        task: &SubTask,
        tool_name: &str,
        tool_handler: &Arc<ToolCallHandler>,
        agent_config: Option<&AgentConfigPB>,
    ) -> FlowyResult<(String, String)> {
        let arguments = serde_json::json!({
            "query": task.description
        });

        let request = ToolCallRequest {
            id: format!("native_{}_{}", tool_name, task.id),
            tool_name: tool_name.to_string(),
            arguments,
            source: Some("native".to_string()),
        };

        let response = tool_handler.execute_tool_call(&request, agent_config).await;
        
        if response.success {
            Ok((
                response.result.unwrap_or_else(|| "Native tool executed".to_string()),
                "native".to_string()
            ))
        } else {
            Err(FlowyError::internal().with_context(
                response.error.unwrap_or_else(|| "Native tool execution failed".to_string())
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_task_result_serialization() {
        let result = SubTaskResult {
            task_id: "task_1".to_string(),
            task_description: "Search for X".to_string(),
            tool_used: "DocumentRag".to_string(),
            success: true,
            content: "Result content".to_string(),
            error: None,
            duration_ms: 100,
            source: "document_rag".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("task_1"));
        assert!(json.contains("DocumentRag"));
    }
}

