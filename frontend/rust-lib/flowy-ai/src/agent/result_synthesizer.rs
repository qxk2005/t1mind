use super::parallel_dispatcher::{DispatchResult, SubTaskResult};
use flowy_error::{FlowyError, FlowyResult};
use serde_json::json;
use tracing::{info, warn, debug};

/// 结果综合器
/// 使用AI将多个工具的结果综合成最终答案
pub struct ResultSynthesizer {
    base_url: String,
    api_key: String,
    model: String,
}

impl ResultSynthesizer {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
        }
    }

    /// 综合多个工具的结果，生成最终答案
    /// 
    /// # Arguments
    /// * `original_question` - 用户的原始问题
    /// * `dispatch_result` - 并行调度的结果
    /// * `system_prompt` - 可选的系统提示词
    pub async fn synthesize(
        &self,
        original_question: &str,
        dispatch_result: &DispatchResult,
        system_prompt: Option<&str>,
    ) -> FlowyResult<String> {
        info!(
            "🔄 [SYNTHESIZE] Starting result synthesis for {} sub-task results",
            dispatch_result.results.len()
        );

        // 如果只有一个成功的结果，直接返回（无需综合）
        if dispatch_result.success_count == 1 {
            if let Some(result) = dispatch_result.results.iter().find(|r| r.success) {
                info!("🔄 [SYNTHESIZE] Only one successful result, returning directly");
                return Ok(result.content.clone());
            }
        }

        // 如果所有任务都失败了
        if dispatch_result.success_count == 0 {
            warn!("⚠️ [SYNTHESIZE] All sub-tasks failed");
            return Ok("抱歉，所有信息检索都失败了，无法回答您的问题。".to_string());
        }

        // 构建综合提示词
        let synthesis_prompt = self.build_synthesis_prompt(
            original_question,
            &dispatch_result.results,
            system_prompt,
        );

        // 调用AI进行综合
        let synthesized_answer = self.call_ai_for_synthesis(&synthesis_prompt).await?;

        info!(
            "✅ [SYNTHESIZE] Synthesis complete, answer length: {} chars",
            synthesized_answer.len()
        );

        Ok(synthesized_answer)
    }

    /// 构建综合提示词
    fn build_synthesis_prompt(
        &self,
        original_question: &str,
        results: &[SubTaskResult],
        system_prompt: Option<&str>,
    ) -> String {
        let mut prompt = String::new();

        // 添加自定义系统提示词
        if let Some(sp) = system_prompt {
            prompt.push_str("## System Instructions\n");
            prompt.push_str(sp);
            prompt.push_str("\n\n");
        }

        // 添加综合指导
        prompt.push_str(
            r#"## Task
You are synthesizing information from multiple sources to answer a user's question.

## Instructions
1. Analyze all the information provided from different sources
2. Cross-reference and verify information when possible
3. Synthesize a comprehensive, coherent answer
4. Cite sources when mentioning specific information
5. If sources conflict, acknowledge the discrepancy
6. If information is insufficient, acknowledge what is missing
7. Maintain a natural, conversational tone

"#,
        );

        // 添加原始问题
        prompt.push_str(&format!("## Original Question\n{}\n\n", original_question));

        // 添加各个来源的信息
        prompt.push_str("## Information from Multiple Sources\n\n");

        for (idx, result) in results.iter().enumerate() {
            if result.success {
                prompt.push_str(&format!(
                    "### Source {} - {} ({})\n",
                    idx + 1,
                    self.format_tool_name(&result.tool_used),
                    result.source
                ));
                prompt.push_str(&format!("**Task**: {}\n", result.task_description));
                prompt.push_str(&format!("**Content**:\n{}\n\n", result.content));
            } else {
                prompt.push_str(&format!(
                    "### Source {} - {} (Failed)\n",
                    idx + 1,
                    self.format_tool_name(&result.tool_used)
                ));
                prompt.push_str(&format!("**Task**: {}\n", result.task_description));
                prompt.push_str(&format!(
                    "**Error**: {}\n\n",
                    result.error.as_deref().unwrap_or("Unknown error")
                ));
            }
        }

        // 添加综合要求
        prompt.push_str(
            r#"
## Your Response
Synthesize the above information into a comprehensive answer. Structure your response clearly and cite sources where appropriate. If you use information from a specific source, mention it (e.g., "According to the internal documents..." or "Based on web search results...").
"#,
        );

        prompt
    }

    /// 格式化工具名称
    fn format_tool_name(&self, tool_used: &str) -> String {
        if tool_used.contains("DocumentRag") {
            "Internal Documents".to_string()
        } else if tool_used.contains("WebSearch") {
            "Web Search".to_string()
        } else if tool_used.contains("Mcp") {
            "External Tool".to_string()
        } else if tool_used.contains("Native") {
            "System Tool".to_string()
        } else {
            tool_used.to_string()
        }
    }

    /// 调用AI进行结果综合
    async fn call_ai_for_synthesis(&self, prompt: &str) -> FlowyResult<String> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let payload = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 2000
        });

        debug!(
            "🔄 [SYNTHESIZE] Calling AI for synthesis, prompt length: {} chars",
            prompt.len()
        );

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
            warn!(
                "❌ [SYNTHESIZE] AI call failed: {} - {}",
                status, error_text
            );
            return Err(FlowyError::server_error()
                .with_context(format!("Synthesis AI call failed: {}", status)));
        }

        let response_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;

        debug!(
            "🔄 [SYNTHESIZE] AI response received, size: {} bytes",
            serde_json::to_string(&response_json)
                .unwrap_or_default()
                .len()
        );

        // 提取内容
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| FlowyError::internal().with_context("Invalid AI response format"))?;

        Ok(content.to_string())
    }

    /// 流式综合结果（用于实时显示）
    /// 返回一个异步流，逐步输出综合后的答案
    pub async fn synthesize_stream(
        &self,
        original_question: String,
        dispatch_result: DispatchResult,
        system_prompt: Option<String>,
    ) -> FlowyResult<impl futures::Stream<Item = FlowyResult<String>>> {
        info!(
            "🔄 [SYNTHESIZE-STREAM] Starting streaming synthesis for {} sub-task results",
            dispatch_result.results.len()
        );

        // 构建综合提示词
        let synthesis_prompt = self.build_synthesis_prompt(
            &original_question,
            &dispatch_result.results,
            system_prompt.as_deref(),
        );

        // 调用流式API
        let stream = self.call_ai_for_synthesis_stream(synthesis_prompt).await?;

        Ok(stream)
    }

    /// 调用流式AI进行结果综合
    async fn call_ai_for_synthesis_stream(
        &self,
        prompt: String,
    ) -> FlowyResult<impl futures::Stream<Item = FlowyResult<String>>> {
        use async_stream::try_stream;
        use futures_util::StreamExt;

        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let payload = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 2000,
            "stream": true
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "text/event-stream")
            .json(&payload)
            .send()
            .await
            .map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(FlowyError::server_error()
                .with_context(format!("Synthesis stream call failed: {}", status)));
        }

        let s = try_stream! {
            let mut stream = resp.bytes_stream();
            
            while let Some(chunk) = stream.next().await {
                let bytes = chunk.map_err(|e| FlowyError::server_error().with_context(e.to_string()))?;
                let s = String::from_utf8_lossy(&bytes);
                
                for line in s.lines() {
                    let l = line.trim_start();
                    if !l.starts_with("data:") { continue; }
                    let data = l.trim_start_matches("data:").trim();
                    if data == "[DONE]" { break; }
                    
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(delta) = v.get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("delta"))
                        {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    yield content.to_string();
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tool_name() {
        let synthesizer = ResultSynthesizer::new(
            "http://localhost".to_string(),
            "key".to_string(),
            "model".to_string(),
        );

        assert_eq!(
            synthesizer.format_tool_name("DocumentRag"),
            "Internal Documents"
        );
        assert_eq!(synthesizer.format_tool_name("WebSearch"), "Web Search");
    }
}

