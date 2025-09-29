use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use flowy_error::{FlowyError, FlowyResult};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::ai_manager::AIManager;
use crate::agent::planner::{PlanningStep, PlanningStepStatus, TaskPlan, PlanStatus};
// use crate::entities::{ToolDefinitionPB, ToolTypePB};
use flowy_ai_pub::cloud::{CompleteTextParams, CompletionType, ResponseFormat, ChatCloudService};

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 执行是否成功
    pub success: bool,
    /// 执行结果内容
    pub content: String,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 使用的工具
    pub tool_used: Option<String>,
    /// 工具参数
    pub tool_arguments: Option<Value>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 反思结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    /// 是否需要重试
    pub should_retry: bool,
    /// 调整后的参数
    pub adjusted_arguments: Option<Value>,
    /// 调整后的工具
    pub adjusted_tool: Option<String>,
    /// 反思原因
    pub reason: String,
    /// 建议的下一步行动
    pub next_action: String,
}

/// 执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 工作空间ID
    pub workspace_id: Uuid,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 执行超时时间
    pub timeout: Duration,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用反思机制
    pub enable_reflection: bool,
    /// 安全模式（限制危险操作）
    pub safe_mode: bool,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            workspace_id: Uuid::new_v4(),
            session_id: None,
            user_id: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_reflection: true,
            safe_mode: true,
        }
    }
}

/// AI驱动的任务执行器
pub struct AITaskExecutor {
    /// AI管理器引用
    ai_manager: Arc<AIManager>,
    /// 执行历史记录
    execution_history: Vec<ExecutionResult>,
}

impl AITaskExecutor {
    /// 创建新的任务执行器
    pub fn new(ai_manager: Arc<AIManager>) -> Self {
        Self {
            ai_manager,
            execution_history: Vec::new(),
        }
    }

    /// 执行单个步骤
    #[tracing::instrument(level = "info", skip(self, context))]
    pub async fn execute_step(
        &mut self,
        step: &mut PlanningStep,
        context: &ExecutionContext,
    ) -> FlowyResult<ExecutionResult> {
        let start_time = Instant::now();
        info!("开始执行步骤: {} - {}", step.id, step.description);

        // 更新步骤状态为执行中
        step.status = PlanningStepStatus::InProgress;

        // 检查是否需要工具调用
        if step.tool_name.is_none() {
            // 无工具步骤，直接标记完成
            let result = ExecutionResult {
                success: true,
                content: format!("步骤 '{}' 已完成: {}", step.id, step.description),
                error: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
                tool_used: None,
                tool_arguments: None,
                metadata: HashMap::new(),
            };

            step.status = PlanningStepStatus::Completed;
            step.result = Some(result.content.clone());
            step.actual_duration = Some(result.duration_ms);

            self.execution_history.push(result.clone());
            return Ok(result);
        }

        // 执行工具调用
        let mut last_error = None;
        for attempt in 0..=context.max_retries {
            match self.execute_tool_call(step, context).await {
                Ok(result) => {
                    if attempt > 0 {
                        info!("步骤 '{}' 在第 {} 次重试后成功", step.id, attempt);
                    }

                    // 如果启用反思机制且执行成功，进行反思验证
                    if context.enable_reflection && result.success {
                        match self.reflect_on_execution(step, &result, context).await {
                            Ok(reflection) => {
                                if reflection.should_retry {
                                    warn!("反思建议重试步骤 '{}': {}", step.id, reflection.reason);
                                    
                                    // 应用反思建议的调整
                                    if let Some(adjusted_args) = reflection.adjusted_arguments {
                                        step.tool_arguments = Some(adjusted_args);
                                    }
                                    if let Some(adjusted_tool) = reflection.adjusted_tool {
                                        step.tool_name = Some(adjusted_tool);
                                    }
                                    
                                    // 继续重试
                                    continue;
                                }
                            }
                            Err(e) => {
                                warn!("反思过程失败，继续使用原始结果: {}", e);
                            }
                        }
                    }

                    // 执行成功
                    step.status = PlanningStepStatus::Completed;
                    step.result = Some(result.content.clone());
                    step.actual_duration = Some(result.duration_ms);

                    self.execution_history.push(result.clone());
                    return Ok(result);
                }
                Err(err) => {
                    last_error = Some(err);
                    if attempt < context.max_retries {
                        warn!("步骤 '{}' 执行失败，第 {} 次重试", step.id, attempt + 1);
                        
                        // 如果启用反思机制，尝试分析失败原因并调整
                        if context.enable_reflection {
                            if let Ok(reflection) = self.reflect_on_failure(step, &last_error.as_ref().unwrap(), context).await {
                                info!("反思建议: {}", reflection.reason);
                                
                                // 应用反思建议的调整
                                if let Some(adjusted_args) = reflection.adjusted_arguments {
                                    step.tool_arguments = Some(adjusted_args);
                                }
                                if let Some(adjusted_tool) = reflection.adjusted_tool {
                                    step.tool_name = Some(adjusted_tool);
                                }
                            }
                        }
                        
                        // 等待一段时间后重试
                        tokio::time::sleep(Duration::from_millis(1000 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }

        // 所有重试都失败了
        let error = last_error.unwrap_or_else(|| {
            FlowyError::internal().with_context("步骤执行失败，已达到最大重试次数")
        });

        let result = ExecutionResult {
            success: false,
            content: String::new(),
            error: Some(error.to_string()),
            duration_ms: start_time.elapsed().as_millis() as u64,
            tool_used: step.tool_name.clone(),
            tool_arguments: step.tool_arguments.clone(),
            metadata: HashMap::new(),
        };

        step.status = PlanningStepStatus::Failed;
        step.error = Some(error.to_string());
        step.actual_duration = Some(result.duration_ms);

        self.execution_history.push(result.clone());
        Err(error)
    }

    /// 执行整个任务计划
    #[tracing::instrument(level = "info", skip(self, context))]
    pub async fn execute_plan(
        &mut self,
        plan: &mut TaskPlan,
        context: &ExecutionContext,
    ) -> FlowyResult<Vec<ExecutionResult>> {
        info!("开始执行任务计划: {} - {}", plan.id, plan.goal);
        
        plan.status = PlanStatus::Executing;
        plan.updated_at = chrono::Utc::now();

        let mut results = Vec::new();
        let mut failed_steps = Vec::new();

        // 按依赖关系执行步骤
        let steps_len = plan.steps.len();
        for i in 0..steps_len {
            // 检查依赖是否满足
            let step_id = plan.steps[i].id.clone();
            let dependencies_satisfied = {
                let step = &plan.steps[i];
                self.are_dependencies_satisfied(step, &plan.steps)
            };
            
            if !dependencies_satisfied {
                warn!("步骤 '{}' 的依赖未满足，跳过执行", step_id);
                plan.steps[i].status = PlanningStepStatus::Skipped;
                continue;
            }
            
            let step = &mut plan.steps[i];

            match self.execute_step(step, context).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    error!("步骤 '{}' 执行失败: {}", step.id, e);
                    failed_steps.push(step.id.clone());
                    
                    // 根据错误严重程度决定是否继续
                    if self.is_critical_failure(&e) {
                        plan.status = PlanStatus::Failed;
                        return Err(e);
                    }
                }
            }
        }

        // 更新计划状态
        if failed_steps.is_empty() {
            plan.status = PlanStatus::Completed;
            info!("任务计划执行完成: {}", plan.id);
        } else {
            plan.status = PlanStatus::Failed;
            warn!("任务计划部分失败: {}，失败步骤: {:?}", plan.id, failed_steps);
        }

        plan.updated_at = chrono::Utc::now();
        Ok(results)
    }

    /// 执行工具调用
    async fn execute_tool_call(
        &self,
        step: &PlanningStep,
        context: &ExecutionContext,
    ) -> FlowyResult<ExecutionResult> {
        let start_time = Instant::now();
        
        let tool_name = step.tool_name.as_ref().unwrap();
        let tool_source = step.tool_source.as_ref();
        let default_args = json!({});
        let arguments = step.tool_arguments.as_ref().unwrap_or(&default_args);

        debug!("执行工具调用: {} (来源: {:?})", tool_name, tool_source);

        // 安全检查
        if context.safe_mode && self.is_dangerous_operation(tool_name, arguments) {
            return Err(FlowyError::invalid_data()
                .with_context(format!("安全模式下禁止执行危险操作: {}", tool_name)));
        }

        // 根据工具来源选择执行方式
        let result = if let Some(source) = tool_source {
            if source == "appflowy" {
                // 执行原生工具
                self.execute_native_tool(tool_name, arguments, context).await?
            } else {
                // 执行MCP工具
                self.execute_mcp_tool(source, tool_name, arguments, context).await?
            }
        } else {
            // 尝试自动检测工具类型
            self.execute_auto_detected_tool(tool_name, arguments, context).await?
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            success: true,
            content: result,
            error: None,
            duration_ms,
            tool_used: Some(tool_name.clone()),
            tool_arguments: Some(arguments.clone()),
            metadata: HashMap::new(),
        })
    }

    /// 执行MCP工具
    async fn execute_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: &Value,
        _context: &ExecutionContext,
    ) -> FlowyResult<String> {
        debug!("调用MCP工具: {} on server: {}", tool_name, server_id);

        let response = self.ai_manager.mcp_manager
            .call_tool(server_id, tool_name, arguments.clone())
            .await?;

        // 处理响应内容
        let mut result_parts = Vec::new();
        for content in response.content {
            if let Some(text) = content.text {
                result_parts.push(text);
            }
        }

        Ok(result_parts.join("\n"))
    }

    /// 执行原生工具
    async fn execute_native_tool(
        &self,
        tool_name: &str,
        arguments: &Value,
        context: &ExecutionContext,
    ) -> FlowyResult<String> {
        debug!("调用原生工具: {}", tool_name);

        match tool_name {
            "create_document" => {
                self.create_document_tool(arguments, context).await
            }
            "search_documents" => {
                self.search_documents_tool(arguments, context).await
            }
            _ => {
                Err(FlowyError::invalid_data()
                    .with_context(format!("未知的原生工具: {}", tool_name)))
            }
        }
    }

    /// 自动检测并执行工具
    async fn execute_auto_detected_tool(
        &self,
        tool_name: &str,
        arguments: &Value,
        context: &ExecutionContext,
    ) -> FlowyResult<String> {
        // 首先尝试原生工具
        if let Ok(result) = self.execute_native_tool(tool_name, arguments, context).await {
            return Ok(result);
        }

        // 然后尝试MCP工具
        let servers = self.ai_manager.mcp_manager.list_servers().await;
        for server in servers {
            if let Ok(tools) = self.ai_manager.mcp_manager.tool_list(&server.server_id).await {
                if tools.tools.iter().any(|t| t.name == tool_name) {
                    return self.execute_mcp_tool(&server.server_id, tool_name, arguments, context).await;
                }
            }
        }

        Err(FlowyError::record_not_found()
            .with_context(format!("找不到工具: {}", tool_name)))
    }

    /// 反思执行结果
    async fn reflect_on_execution(
        &self,
        step: &PlanningStep,
        result: &ExecutionResult,
        context: &ExecutionContext,
    ) -> FlowyResult<ReflectionResult> {
        let reflection_prompt = self.build_reflection_prompt(step, Some(result), None);
        let ai_response = self.call_ai_for_reflection(&reflection_prompt, context).await?;
        self.parse_reflection_response(&ai_response)
    }

    /// 反思失败原因
    async fn reflect_on_failure(
        &self,
        step: &PlanningStep,
        error: &FlowyError,
        context: &ExecutionContext,
    ) -> FlowyResult<ReflectionResult> {
        let reflection_prompt = self.build_reflection_prompt(step, None, Some(error));
        let ai_response = self.call_ai_for_reflection(&reflection_prompt, context).await?;
        self.parse_reflection_response(&ai_response)
    }

    /// 构建反思提示词
    fn build_reflection_prompt(
        &self,
        step: &PlanningStep,
        result: Option<&ExecutionResult>,
        error: Option<&FlowyError>,
    ) -> String {
        let mut prompt = format!(
            r#"你是一个AI执行反思专家。请分析以下步骤的执行情况并提供改进建议。

步骤信息:
- ID: {}
- 描述: {}
- 工具: {:?}
- 参数: {}

"#,
            step.id,
            step.description,
            step.tool_name,
            step.tool_arguments.as_ref()
                .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
                .unwrap_or_default()
        );

        if let Some(result) = result {
            prompt.push_str(&format!(
                r#"执行结果:
- 成功: {}
- 内容: {}
- 执行时间: {}ms

"#,
                result.success,
                result.content.chars().take(200).collect::<String>(),
                result.duration_ms
            ));
        }

        if let Some(error) = error {
            prompt.push_str(&format!(
                r#"错误信息:
- 错误: {}

"#,
                error.to_string()
            ));
        }

        prompt.push_str(r#"请分析执行情况并返回JSON格式的反思结果:
{
  "should_retry": false,
  "adjusted_arguments": null,
  "adjusted_tool": null,
  "reason": "分析原因",
  "next_action": "建议的下一步行动"
}

分析要点:
1. 执行是否达到预期目标
2. 参数是否合适
3. 工具选择是否正确
4. 是否需要调整或重试
5. 如何改进执行效果

请只返回JSON，不要包含其他文本。"#);

        prompt
    }

    /// 调用AI进行反思
    async fn call_ai_for_reflection(
        &self,
        prompt: &str,
        context: &ExecutionContext,
    ) -> FlowyResult<String> {
        let ai_model = self.ai_manager.get_active_model(&context.workspace_id.to_string()).await;
        
        let params = CompleteTextParams {
            text: prompt.to_string(),
            completion_type: Some(CompletionType::AskAI),
            format: ResponseFormat::default(),
            metadata: None,
        };

        let mut stream = ChatCloudService::stream_complete(
            &*self.ai_manager.cloud_service_wm,
            &context.workspace_id,
            params,
            ai_model
        ).await?;

        let mut response = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(value) => {
                    match value {
                        flowy_ai_pub::cloud::CompletionStreamValue::Answer { value } => {
                            response.push_str(&value);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(FlowyError::internal().with_context(format!("反思AI调用失败: {}", e)));
                }
            }
        }

        Ok(response)
    }

    /// 解析反思响应
    fn parse_reflection_response(&self, response: &str) -> FlowyResult<ReflectionResult> {
        // 提取JSON部分
        let json_str = self.extract_json_from_response(response)?;
        
        let parsed: Value = serde_json::from_str(&json_str)
            .map_err(|e| FlowyError::invalid_data().with_context(format!("反思响应JSON解析失败: {}", e)))?;

        let should_retry = parsed.get("should_retry")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let adjusted_arguments = parsed.get("adjusted_arguments").cloned();
        
        let adjusted_tool = parsed.get("adjusted_tool")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let reason = parsed.get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("无具体原因")
            .to_string();

        let next_action = parsed.get("next_action")
            .and_then(|v| v.as_str())
            .unwrap_or("继续执行")
            .to_string();

        Ok(ReflectionResult {
            should_retry,
            adjusted_arguments,
            adjusted_tool,
            reason,
            next_action,
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

        Err(FlowyError::invalid_data().with_context("无法从反思响应中提取有效的JSON"))
    }

    /// 检查依赖是否满足
    fn are_dependencies_satisfied(&self, step: &PlanningStep, all_steps: &[PlanningStep]) -> bool {
        for dep_id in &step.dependencies {
            if let Some(dep_step) = all_steps.iter().find(|s| s.id == *dep_id) {
                if dep_step.status != PlanningStepStatus::Completed {
                    return false;
                }
            } else {
                // 依赖步骤不存在
                return false;
            }
        }
        true
    }

    /// 检查是否为关键失败
    fn is_critical_failure(&self, error: &FlowyError) -> bool {
        // 可以根据错误类型判断是否为关键失败
        // 这里简单实现，可以根据需要扩展
        error.code == flowy_error::ErrorCode::Internal
    }

    /// 检查是否为危险操作
    fn is_dangerous_operation(&self, tool_name: &str, _arguments: &Value) -> bool {
        // 定义危险操作列表
        let dangerous_tools = [
            "delete", "remove", "clear", "drop", "truncate",
            "format", "wipe", "destroy", "purge"
        ];

        dangerous_tools.iter().any(|&dangerous| 
            tool_name.to_lowercase().contains(dangerous)
        )
    }

    /// 创建文档工具实现
    async fn create_document_tool(
        &self,
        arguments: &Value,
        _context: &ExecutionContext,
    ) -> FlowyResult<String> {
        let title = arguments.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少文档标题"))?;

        let content = arguments.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 这里应该调用实际的文档创建API
        // 暂时返回模拟结果
        Ok(format!("已创建文档: '{}', 内容长度: {} 字符", title, content.len()))
    }

    /// 搜索文档工具实现
    async fn search_documents_tool(
        &self,
        arguments: &Value,
        _context: &ExecutionContext,
    ) -> FlowyResult<String> {
        let query = arguments.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowyError::invalid_data().with_context("缺少搜索查询"))?;

        let limit = arguments.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        // 这里应该调用实际的文档搜索API
        // 暂时返回模拟结果
        Ok(format!("搜索查询 '{}' 找到 {} 个结果（限制: {}）", query, 0, limit))
    }

    /// 获取执行历史
    pub fn get_execution_history(&self) -> &[ExecutionResult] {
        &self.execution_history
    }

    /// 清除执行历史
    pub fn clear_execution_history(&mut self) {
        self.execution_history.clear();
    }

    /// 获取执行统计信息
    pub fn get_execution_statistics(&self) -> HashMap<String, Value> {
        let mut stats = HashMap::new();
        
        let total_executions = self.execution_history.len();
        let successful_executions = self.execution_history.iter().filter(|r| r.success).count();
        let failed_executions = total_executions - successful_executions;
        
        let total_duration: u64 = self.execution_history.iter().map(|r| r.duration_ms).sum();
        let avg_duration = if total_executions > 0 { 
            total_duration / total_executions as u64 
        } else { 
            0 
        };

        stats.insert("total_executions".to_string(), json!(total_executions));
        stats.insert("successful_executions".to_string(), json!(successful_executions));
        stats.insert("failed_executions".to_string(), json!(failed_executions));
        stats.insert("success_rate".to_string(), json!(if total_executions > 0 { 
            successful_executions as f64 / total_executions as f64 
        } else { 
            0.0 
        }));
        stats.insert("total_duration_ms".to_string(), json!(total_duration));
        stats.insert("average_duration_ms".to_string(), json!(avg_duration));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::planner::PersonalizationFeatures;

    #[test]
    fn test_execution_context_default() {
        let context = ExecutionContext::default();
        assert_eq!(context.max_retries, 3);
        assert_eq!(context.timeout, Duration::from_secs(30));
        assert!(context.enable_reflection);
        assert!(context.safe_mode);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            success: true,
            content: "Test result".to_string(),
            error: None,
            duration_ms: 100,
            tool_used: Some("test_tool".to_string()),
            tool_arguments: Some(json!({"param": "value"})),
            metadata: HashMap::new(),
        };

        assert!(result.success);
        assert_eq!(result.content, "Test result");
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_reflection_result() {
        let reflection = ReflectionResult {
            should_retry: true,
            adjusted_arguments: Some(json!({"new_param": "new_value"})),
            adjusted_tool: Some("new_tool".to_string()),
            reason: "Need adjustment".to_string(),
            next_action: "Retry with new parameters".to_string(),
        };

        assert!(reflection.should_retry);
        assert!(reflection.adjusted_arguments.is_some());
        assert_eq!(reflection.reason, "Need adjustment");
    }
}
