// 智能体工具调用处理器
// 负责解析AI响应中的工具调用请求，执行工具，并返回结果

use std::sync::Arc;
use flowy_error::{FlowyError, FlowyResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::ai_manager::AIManager;
use crate::entities::AgentConfigPB;

/// 工具调用请求（从AI响应中解析）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// 工具调用ID（用于追踪）
    pub id: String,
    /// 工具名称
    pub tool_name: String,
    /// 工具参数
    pub arguments: Value,
    /// 工具来源（可选：MCP server ID 或 "native"）
    pub source: Option<String>,
}

/// 工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// 工具调用ID
    pub id: String,
    /// 是否成功
    pub success: bool,
    /// 结果内容
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
}

/// 工具调用协议格式
/// 
/// AI 应该在响应中使用以下格式请求工具调用：
/// 
/// ```
/// <tool_call>
/// {
///   "id": "call_123",
///   "tool_name": "search_documents",
///   "arguments": {
///     "query": "用户搜索词",
///     "limit": 10
///   },
///   "source": "appflowy"
/// }
/// </tool_call>
/// ```
#[derive(Debug)]
pub struct ToolCallProtocol;

impl ToolCallProtocol {
    /// 开始标签
    pub const START_TAG: &'static str = "<tool_call>";
    /// 结束标签
    pub const END_TAG: &'static str = "</tool_call>";
    
    /// 解析工具调用请求
    pub fn parse(text: &str) -> FlowyResult<ToolCallRequest> {
        // 尝试修复常见的 JSON 格式错误
        let fixed_text = Self::fix_common_json_errors(text);
        
        let request: ToolCallRequest = serde_json::from_str(&fixed_text)
            .map_err(|e| {
                warn!("JSON parse error: {}", e);
                warn!("Original text: {}", text);
                if fixed_text != text {
                    warn!("Fixed text: {}", fixed_text);
                }
                FlowyError::invalid_data()
                    .with_context(format!("Failed to parse tool call request: {}", e))
            })?;
        
        // 验证必需字段
        if request.tool_name.is_empty() {
            return Err(FlowyError::invalid_data()
                .with_context("Tool name cannot be empty"));
        }
        
        Ok(request)
    }
    
    /// 修复 AI 生成的常见 JSON 格式错误
    fn fix_common_json_errors(text: &str) -> String {
        let mut fixed = text.to_string();
        
        // 修复 1: "arguments {" → "arguments": {
        fixed = fixed.replace("\"arguments {", "\"arguments\": {");
        fixed = fixed.replace("\"arguments{", "\"arguments\": {");
        
        // 修复 2: 检查并修复不完整的 JSON（缺少闭合括号）
        // 这是流式传输常见的问题：AI 返回了完整标签但 JSON 内容不完整
        let trimmed = fixed.trim();
        
        // 统计括号数量
        let open_braces = trimmed.matches('{').count();
        let close_braces = trimmed.matches('}').count();
        let open_brackets = trimmed.matches('[').count();
        let close_brackets = trimmed.matches(']').count();
        
        // 如果缺少闭合括号，尝试补全
        if open_braces > close_braces || open_brackets > close_brackets {
            warn!("🔧 [JSON FIX] Detected incomplete JSON - open_braces: {}, close_braces: {}, open_brackets: {}, close_brackets: {}", 
                  open_braces, close_braces, open_brackets, close_brackets);
            
            // 补全缺少的括号
            let mut fixed_with_braces = fixed.clone();
            
            // 先补全方括号
            for _ in 0..(open_brackets - close_brackets) {
                fixed_with_braces.push_str("\n]");
            }
            
            // 再补全大括号
            for _ in 0..(open_braces - close_braces) {
                fixed_with_braces.push_str("\n}");
            }
            
            info!("🔧 [JSON FIX] Added {} closing brackets and {} closing braces", 
                  open_brackets - close_brackets, open_braces - close_braces);
            
            // 使用修复后的文本继续后续处理
            fixed = fixed_with_braces;
        }
        
        // 修复 3: 缺少逗号和括号
        // 特别处理常见模式：arguments 结束后缺少 }, 然后是 "source"
        let lines: Vec<&str> = fixed.lines().collect();
        let mut result_lines = Vec::new();
        let mut in_arguments = false;
        let mut arguments_depth = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // 检测进入 arguments
            if trimmed.contains("\"arguments\"") && trimmed.contains("{") {
                in_arguments = true;
                arguments_depth = 1;
                result_lines.push(line.to_string());
                continue;
            }
            
            // 在 arguments 内部，跟踪括号深度
            if in_arguments {
                for ch in trimmed.chars() {
                    if ch == '{' {
                        arguments_depth += 1;
                    } else if ch == '}' {
                        arguments_depth -= 1;
                        if arguments_depth == 0 {
                            in_arguments = false;
                        }
                    }
                }
            }
            
            // 如果当前在 arguments 内部，但下一行是 "source" 或其他顶级键
            if in_arguments && i + 1 < lines.len() {
                let next_trimmed = lines[i + 1].trim();
                if next_trimmed.starts_with("\"source\"") || 
                   next_trimmed.starts_with("\"id\"") ||
                   next_trimmed.starts_with("\"tool_name\"") {
                    // 需要关闭 arguments 对象
                    let indent = line.len() - line.trim_start().len();
                    let spaces = " ".repeat(indent);
                    
                    if !trimmed.ends_with(',') && !trimmed.is_empty() {
                        result_lines.push(line.to_string());
                        result_lines.push(format!("{}  }},", spaces));
                    } else {
                        result_lines.push(line.to_string());
                        result_lines.push(format!("{}  }},", spaces));
                    }
                    in_arguments = false;
                    continue;
                }
            }
            
            // 一般情况：添加缺失的逗号
            if i + 1 < lines.len() {
                let next_trimmed = lines[i + 1].trim();
                if (trimmed.ends_with('"') || trimmed.ends_with('}') || 
                    trimmed.ends_with("null") ||
                    trimmed.chars().last().map_or(false, |c| c.is_numeric())) &&
                   !trimmed.ends_with(',') &&
                   !trimmed.ends_with('{') &&
                   next_trimmed.starts_with('"') {
                    result_lines.push(format!("{},", line));
                    continue;
                }
            }
            
            result_lines.push(line.to_string());
        }
        
        result_lines.join("\n")
    }
    
    /// 格式化工具调用响应
    pub fn format_response(response: &ToolCallResponse) -> String {
        let json_str = serde_json::to_string_pretty(response)
            .unwrap_or_else(|_| "{}".to_string());
        
        format!("<tool_result>\n{}\n</tool_result>", json_str)
    }
}

/// 工具调用处理器
#[derive(Clone)]
pub struct ToolCallHandler {
    mcp_manager: Arc<crate::mcp::MCPClientManager>,
}

impl ToolCallHandler {
    pub fn new(mcp_manager: Arc<crate::mcp::MCPClientManager>) -> Self {
        Self { mcp_manager }
    }
    
    /// 从 AIManager 创建（便捷方法）
    pub fn from_ai_manager(ai_manager: &AIManager) -> Self {
        Self {
            mcp_manager: ai_manager.mcp_manager.clone(),
        }
    }
    
    /// 检测文本中是否包含工具调用请求
    pub fn contains_tool_call(text: &str) -> bool {
        text.contains(ToolCallProtocol::START_TAG)
    }
    
    /// 从文本中提取所有工具调用请求
    pub fn extract_tool_calls(text: &str) -> Vec<(ToolCallRequest, usize, usize)> {
        let mut calls = Vec::new();
        let mut start = 0;
        
        debug!("🔍 [TOOL PARSE] Starting extraction, text length: {} chars", text.len());
        debug!("🔍 [TOOL PARSE] Text contains {} <tool_call> tags", text.matches(ToolCallProtocol::START_TAG).count());
        debug!("🔍 [TOOL PARSE] Text contains {} </tool_call> tags", text.matches(ToolCallProtocol::END_TAG).count());
        
        while let Some(start_pos) = text[start..].find(ToolCallProtocol::START_TAG) {
            let abs_start = start + start_pos;
            let json_start = abs_start + ToolCallProtocol::START_TAG.len();
            
            debug!("🔍 [TOOL PARSE] Found <tool_call> tag at position {}", abs_start);
            debug!("🔍 [TOOL PARSE] Searching for </tool_call> starting from position {}", json_start);
            
            if let Some(end_pos) = text[json_start..].find(ToolCallProtocol::END_TAG) {
                let json_end = json_start + end_pos;
                let abs_end = json_end + ToolCallProtocol::END_TAG.len();
                let json_text = &text[json_start..json_end].trim();
                
                debug!("🔍 [TOOL PARSE] Found </tool_call> at position {}", json_end);
                debug!("🔍 [TOOL PARSE] JSON content length: {}", json_text.len());
                debug!("🔍 [TOOL PARSE] JSON content: {}", json_text);
                
                match ToolCallProtocol::parse(json_text) {
                    Ok(request) => {
                        info!("✅ [TOOL PARSE] Successfully parsed tool call: {} (id: {})", 
                              request.tool_name, request.id);
                        calls.push((request, abs_start, abs_end));
                        start = abs_end;
                    }
                    Err(e) => {
                        warn!("❌ [TOOL PARSE] Failed to parse tool call JSON: {}", e);
                        // 安全地切割字符串，避免在 UTF-8 字符边界中间切割
                        let preview = if json_text.len() > 200 {
                            let mut preview_len = 200.min(json_text.len());
                            while preview_len > 0 && !json_text.is_char_boundary(preview_len) {
                                preview_len -= 1;
                            }
                            &json_text[..preview_len]
                        } else {
                            json_text
                        };
                        warn!("❌ [TOOL PARSE] Invalid JSON (first {} chars): {}", preview.len(), preview);
                        // 跳过这个失败的工具调用，继续查找下一个
                        start = abs_end;
                    }
                }
            } else {
                warn!("❌ [TOOL PARSE] Found <tool_call> at position {} but no matching </tool_call> tag", abs_start);
                warn!("❌ [TOOL PARSE] Remaining text length: {} chars", text[json_start..].len());
                if text[json_start..].len() < 100 {
                    warn!("❌ [TOOL PARSE] Remaining text: {}", &text[json_start..]);
                } else {
                    warn!("❌ [TOOL PARSE] Remaining text preview: {}...", &text[json_start..std::cmp::min(json_start + 100, text.len())]);
                }
                break;
            }
        }
        
        info!("🔍 [TOOL PARSE] Extraction complete: {} valid tool calls found", calls.len());
        calls
    }
    
    /// 执行单个工具调用
    pub async fn execute_tool_call(
        &self,
        request: &ToolCallRequest,
        agent_config: Option<&AgentConfigPB>,
    ) -> ToolCallResponse {
        let start_time = std::time::Instant::now();
        
        info!("═══════════════════════════════════════════════════════════");
        info!("🔧 [TOOL EXEC] Starting tool execution");
        info!("🔧 [TOOL EXEC]   ID: {}", request.id);
        info!("🔧 [TOOL EXEC]   Tool: {}", request.tool_name);
        info!("🔧 [TOOL EXEC]   Source: {:?}", request.source);
        info!("🔧 [TOOL EXEC]   Arguments: {}", 
              serde_json::to_string_pretty(&request.arguments).unwrap_or_else(|_| "{}".to_string()));
        
        // 验证工具权限
        if let Some(config) = agent_config {
            if !self.is_tool_allowed(config, &request.tool_name) {
                error!("🔧 [TOOL EXEC] ❌ Tool '{}' is not allowed for this agent", request.tool_name);
                return ToolCallResponse {
                    id: request.id.clone(),
                    success: false,
                    result: None,
                    error: Some(format!(
                        "Tool '{}' is not allowed for this agent",
                        request.tool_name
                    )),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                };
            } else {
                info!("🔧 [TOOL EXEC] ✅ Tool permission verified");
            }
        }
        
        // 执行工具
        info!("🔧 [TOOL EXEC] Executing tool...");
        let result = if let Some(source) = &request.source {
            // 检查 source 是否是 "native" 或 "appflowy"
            if source == "native" || source == "appflowy" {
                // 即使指定了 native/appflowy，也先尝试从 MCP 查找
                // 因为 AI 可能错误地标记了 source
                info!("🔧 [TOOL EXEC] Source specified as '{}', checking if tool exists in MCP first...", source);
                
                match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
                    Some((server_id, _)) => {
                        info!("✅ [TOOL EXEC] Tool '{}' found in MCP server '{}', using MCP instead", 
                              request.tool_name, server_id);
                        self.execute_mcp_tool(&server_id, request).await
                    }
                    None => {
                        info!("🔧 [TOOL EXEC] Tool not found in MCP, calling native tool");
                        self.execute_native_tool(request).await
                    }
                }
            } else {
                // source 是具体的 MCP server ID
                info!("🔧 [TOOL EXEC] Calling MCP tool on server: {}", source);
                self.execute_mcp_tool(source, request).await
            }
        } else {
            info!("🔧 [TOOL EXEC] No source specified, auto-detecting...");
            self.execute_auto_detected_tool(request).await
        };
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(content) => {
                info!("🔧 [TOOL EXEC] ✅ Tool call SUCCEEDED");
                info!("🔧 [TOOL EXEC]   Duration: {}ms", duration_ms);
                info!("🔧 [TOOL EXEC]   Original result size: {} chars", content.len());
                
                // 🔧 应用工具结果最大长度限制（从智能体配置中获取）
                let max_result_length = agent_config
                    .map(|config| {
                        // 确保值在合理范围内：最小 1000，默认 4000
                        let configured = config.capabilities.max_tool_result_length;
                        if configured <= 0 {
                            4000 // 默认值
                        } else if configured < 1000 {
                            1000 // 最小值
                        } else {
                            configured as usize
                        }
                    })
                    .unwrap_or(4000); // 如果没有配置，使用默认值 4000
                
                // 智能截断长结果
                let final_content = if content.len() > max_result_length {
                    // 安全截断，考虑 UTF-8 字符边界
                    let mut truncate_len = max_result_length.min(content.len());
                    while truncate_len > 0 && !content.is_char_boundary(truncate_len) {
                        truncate_len -= 1;
                    }
                    let truncated = &content[..truncate_len];
                    
                    warn!("🔧 [TOOL EXEC] ⚠️ Tool result truncated from {} to {} chars (max: {})", 
                          content.len(), truncate_len, max_result_length);
                    
                    format!(
                        "{}\n\n--- 结果已截断 ---\n原始长度: {} 字符\n显示长度: {} 字符\n配置限制: {} 字符\n\n💡 提示：如需查看完整结果，请在智能体配置中增加「工具结果最大长度」",
                        truncated,
                        content.len(),
                        truncate_len,
                        max_result_length
                    )
                } else {
                    info!("🔧 [TOOL EXEC]   Result within limit (max: {} chars)", max_result_length);
                    content
                };
                
                // 日志预览（使用截断后的内容）
                if final_content.len() <= 300 {
                    info!("🔧 [TOOL EXEC]   Final result: {}", final_content);
                } else {
                    let mut preview_len = 300.min(final_content.len());
                    while preview_len > 0 && !final_content.is_char_boundary(preview_len) {
                        preview_len -= 1;
                    }
                    info!("🔧 [TOOL EXEC]   Result preview: {}...", &final_content[..preview_len]);
                }
                info!("═══════════════════════════════════════════════════════════");
                
                ToolCallResponse {
                    id: request.id.clone(),
                    success: true,
                    result: Some(final_content),
                    error: None,
                    duration_ms,
                }
            }
            Err(e) => {
                error!("🔧 [TOOL EXEC] ❌ Tool call FAILED");
                error!("🔧 [TOOL EXEC]   Duration: {}ms", duration_ms);
                error!("🔧 [TOOL EXEC]   Error: {}", e);
                info!("═══════════════════════════════════════════════════════════");
                
                ToolCallResponse {
                    id: request.id.clone(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    duration_ms,
                }
            }
        }
    }
    
    /// 执行MCP工具
    async fn execute_mcp_tool(
        &self,
        server_id: &str,
        request: &ToolCallRequest,
    ) -> FlowyResult<String> {
        info!("🔧 [MCP TOOL] Calling MCP tool: {} on server: {}", request.tool_name, server_id);
        info!("🔧 [MCP TOOL] Arguments: {}", serde_json::to_string(&request.arguments).unwrap_or_default());
        
        // 🔌 自动连接检查:如果服务器未连接,先尝试连接
        if !self.mcp_manager.is_server_connected(server_id) {
            info!("🔌 [MCP AUTO-CONNECT] Server '{}' is not connected, attempting to connect...", server_id);
            
            match self.mcp_manager.connect_server_from_config(server_id).await {
                Ok(()) => {
                    info!("✅ [MCP AUTO-CONNECT] Successfully connected to server '{}'", server_id);
                }
                Err(e) => {
                    error!("❌ [MCP AUTO-CONNECT] Failed to connect to server '{}': {}", server_id, e);
                    return Err(FlowyError::internal()
                        .with_context(format!("Auto-connect failed for server '{}': {}", server_id, e)));
                }
            }
        } else {
            info!("✓ [MCP TOOL] Server '{}' already connected", server_id);
        }
        
        let call_start = std::time::Instant::now();
        
        let response = self.mcp_manager
            .call_tool(server_id, &request.tool_name, request.arguments.clone())
            .await
            .map_err(|e| {
                error!("🔧 [MCP TOOL] ❌ Tool call failed: {} - {}", request.tool_name, e);
                e
            })?;
        
        let call_duration = call_start.elapsed();
        
        // 提取文本内容
        let mut result_parts = Vec::new();
        for (idx, content) in response.content.iter().enumerate() {
            if let Some(text) = &content.text {
                info!("🔧 [MCP TOOL] Response content #{}: {} chars", idx + 1, text.len());
                result_parts.push(text.clone());
            }
        }
        
        let result = result_parts.join("\n");
        info!("🔧 [MCP TOOL] ✅ Tool call succeeded in {:?}", call_duration);
        info!("🔧 [MCP TOOL] Total result length: {} chars", result.len());
        
        if result.len() <= 200 {
            info!("🔧 [MCP TOOL] Full result: {}", result);
        } else {
            // 安全地切割字符串，避免在 UTF-8 字符边界中间切割
            let mut preview_len = 200.min(result.len());
            while preview_len > 0 && !result.is_char_boundary(preview_len) {
                preview_len -= 1;
            }
            info!("🔧 [MCP TOOL] Result preview (first {} chars): {}", preview_len, &result[..preview_len]);
        }
        
        Ok(result)
    }
    
    /// 执行原生工具
    async fn execute_native_tool(
        &self,
        request: &ToolCallRequest,
    ) -> FlowyResult<String> {
        debug!("Calling native tool: {}", request.tool_name);
        
        // TODO: 实现原生工具调用
        // 这里需要根据实际的原生工具实现来调用
        
        Err(FlowyError::not_support()
            .with_context(format!("Native tool '{}' not yet implemented", request.tool_name)))
    }
    
    /// 自动检测并执行工具
    async fn execute_auto_detected_tool(
        &self,
        request: &ToolCallRequest,
    ) -> FlowyResult<String> {
        info!("🔍 [TOOL AUTO] Auto-detecting tool: {}", request.tool_name);
        
        // 使用 find_tool_by_name 从所有配置的 MCP 服务器中查找工具
        match self.mcp_manager.find_tool_by_name(&request.tool_name).await {
            Some((server_id, tool)) => {
                info!("✅ [TOOL AUTO] Tool '{}' found in MCP server '{}' ({})", 
                      request.tool_name, server_id, tool.description.as_deref().unwrap_or("No description"));
                self.execute_mcp_tool(&server_id, request).await
            }
            None => {
                info!("🔍 [TOOL AUTO] Tool '{}' not found in any MCP server, trying native tools", request.tool_name);
                self.execute_native_tool(request).await
            }
        }
    }
    
    /// 检查工具是否被允许
    fn is_tool_allowed(&self, agent_config: &AgentConfigPB, tool_name: &str) -> bool {
        // 如果没有配置可用工具列表，则允许所有工具
        if agent_config.available_tools.is_empty() {
            return true;
        }
        
        // 检查工具是否在允许列表中
        agent_config.available_tools.iter().any(|t| t == tool_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_tool_call() {
        let text = r#"
        这是一些文本
        <tool_call>
        {
          "id": "call_123",
          "tool_name": "search",
          "arguments": {"query": "test"},
          "source": "mcp"
        }
        </tool_call>
        更多文本
        "#;
        
        let calls = ToolCallHandler::extract_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0.tool_name, "search");
        assert_eq!(calls[0].0.id, "call_123");
    }
    
    #[test]
    fn test_multiple_tool_calls() {
        let text = r#"
        <tool_call>{"id": "1", "tool_name": "tool1", "arguments": {}}</tool_call>
        <tool_call>{"id": "2", "tool_name": "tool2", "arguments": {}}</tool_call>
        "#;
        
        let calls = ToolCallHandler::extract_tool_calls(text);
        assert_eq!(calls.len(), 2);
    }
}


