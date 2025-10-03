// 流式工具调用处理器
// 作为流处理的包装器，检测和执行工具调用

use std::sync::Arc;
use futures::Stream;
use futures::StreamExt;
use async_stream::try_stream;
use tracing::{debug, info};

use flowy_ai_pub::cloud::QuestionStreamValue;
use flowy_error::FlowyResult;

use crate::ai_manager::AIManager;
use crate::entities::AgentConfigPB;
use crate::agent::{ToolCallHandler, ToolCallProtocol};

/// 流式工具处理包装器
/// 
/// 包装原始流，检测工具调用并执行
pub struct StreamToolWrapper {
    ai_manager: Arc<AIManager>,
    tool_handler: ToolCallHandler,
}

impl StreamToolWrapper {
    pub fn new(ai_manager: Arc<AIManager>) -> Self {
        let tool_handler = ToolCallHandler::from_ai_manager(&ai_manager);
        Self {
            ai_manager,
            tool_handler,
        }
    }
    
    /// 包装流以支持工具调用
    /// 
    /// 这个方法接收原始的 AI 响应流，检测其中的工具调用，
    /// 执行工具，并将结果插入到流中
    pub fn wrap_stream<S>(
        &self,
        original_stream: S,
        agent_config: Option<AgentConfigPB>,
    ) -> impl Stream<Item = FlowyResult<QuestionStreamValue>>
    where
        S: Stream<Item = FlowyResult<QuestionStreamValue>> + Send + 'static,
    {
        let tool_handler = self.tool_handler.clone();
        let mut accumulated_text = String::new();
        let mut original_stream = Box::pin(original_stream);
        
        try_stream! {
            while let Some(result) = original_stream.next().await {
                match result? {
                    QuestionStreamValue::Answer { value } => {
                        accumulated_text.push_str(&value);
                        
                        // 检测工具调用
                        if ToolCallHandler::contains_tool_call(&accumulated_text) {
                            debug!("[StreamTool] Tool call detected in accumulated text");
                            
                            let calls = ToolCallHandler::extract_tool_calls(&accumulated_text);
                            
                            for (request, start, end) in calls {
                                // 发送工具调用前的文本
                                let before_text = &accumulated_text[..start];
                                if !before_text.is_empty() {
                                    yield QuestionStreamValue::Answer {
                                        value: before_text.to_string()
                                    };
                                }
                                
                                // 执行工具
                                info!("[StreamTool] Executing tool: {}", request.tool_name);
                                
                                let response = tool_handler.execute_tool_call(
                                    &request,
                                    agent_config.as_ref(),
                                ).await;
                                
                                // 发送工具执行状态
                                yield QuestionStreamValue::Metadata {
                                    value: serde_json::json!({
                                        "tool_call": {
                                            "id": request.id,
                                            "tool_name": request.tool_name,
                                            "status": if response.success { "success" } else { "failed" }
                                        }
                                    })
                                };
                                
                                // 发送工具执行结果
                                let result_text = ToolCallProtocol::format_response(&response);
                                yield QuestionStreamValue::Answer {
                                    value: result_text
                                };
                                
                                // 清除已处理的文本
                                accumulated_text = accumulated_text[end..].to_string();
                            }
                        } else {
                            // 没有工具调用，直接输出
                            yield QuestionStreamValue::Answer { value };
                            // 清空accumulated_text以避免无限增长
                            accumulated_text.clear();
                        }
                    },
                    other => {
                        // 其他类型的消息（Metadata等）直接传递
                        yield other;
                    }
                }
            }
            
            // 流结束后，如果还有剩余文本，输出它
            if !accumulated_text.is_empty() {
                yield QuestionStreamValue::Answer {
                    value: accumulated_text
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stream_wrapper_creation() {
        // 这里需要模拟 AIManager
        // 实际测试需要完整的集成测试环境
    }
}

