/// Agent Reflection Module
/// 
/// 智能体反思模块用于评估AI回答的质量，判断问题是否被完全解决，
/// 并提供改进建议。该模块通过调用LLM来执行反思逻辑，避免循环推理，
/// 并为智能体提供有价值的问题解决反馈。

use crate::entities::AgentCapabilitiesPB;
use flowy_ai_pub::cloud::AIModel;
use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite_vec::entities::SqliteEmbeddedDocument;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// 反思结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    /// 问题是否被完全解决
    pub is_resolved: bool,
    
    /// 答案质量评分 (0-100)
    pub quality_score: u8,
    
    /// 反思总结
    pub summary: String,
    
    /// 改进建议
    pub suggestions: Vec<String>,
    
    /// 是否需要更多上下文
    pub needs_more_context: bool,
    
    /// 缺失的上下文描述
    pub missing_context: Option<String>,
    
    /// 反思置信度 (0-100)
    pub confidence: u8,
}

/// 反思配置
#[derive(Debug, Clone)]
pub struct ReflectionConfig {
    /// 是否启用反思
    pub enabled: bool,
    
    /// 最大反思次数
    pub max_iterations: usize,
    
    /// 反思超时时间（秒）
    pub timeout_seconds: u64,
    
    /// 质量评分阈值（低于此值需要改进）
    pub quality_threshold: u8,
    
    /// 置信度阈值（低于此值认为结果不确定）
    pub confidence_threshold: u8,
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_iterations: 3,
            timeout_seconds: 30,
            quality_threshold: 70,
            confidence_threshold: 60,
        }
    }
}

impl ReflectionConfig {
    /// 从智能体能力配置创建反思配置
    pub fn from_capabilities(capabilities: &AgentCapabilitiesPB) -> Self {
        Self {
            enabled: capabilities.enable_reflection,
            max_iterations: capabilities.max_reflection_iterations.max(0).min(10) as usize,
            timeout_seconds: 30,
            quality_threshold: 70,
            confidence_threshold: 60,
        }
    }
}

/// AgentReflection 结构体
pub struct AgentReflection {
    config: ReflectionConfig,
}

impl AgentReflection {
    /// 创建新的反思模块实例
    pub fn new(config: ReflectionConfig) -> Self {
        Self { config }
    }

    /// 从智能体能力配置创建反思模块
    pub fn from_capabilities(capabilities: &AgentCapabilitiesPB) -> Self {
        Self::new(ReflectionConfig::from_capabilities(capabilities))
    }

    /// 执行反思评估
    /// 
    /// # 参数
    /// - `user_question`: 用户的原始问题
    /// - `ai_answer`: AI 的回答
    /// - `context_docs`: 检索到的文档（用于评估上下文是否充分）
    /// - `ai_model`: AI模型配置
    /// - `workspace_id`: 工作区ID
    /// - `ai_manager_weak`: AI管理器的弱引用（用于调用LLM）
    /// 
    /// # 返回
    /// 返回反思结果，包含质量评分、改进建议等
    pub async fn reflect(
        &self,
        user_question: &str,
        ai_answer: &str,
        context_docs: &[SqliteEmbeddedDocument],
        _ai_model: &AIModel,
        _workspace_id: &Uuid,
    ) -> FlowyResult<ReflectionResult> {
        // 检查是否启用反思
        if !self.config.enabled {
            return Ok(ReflectionResult {
                is_resolved: true,
                quality_score: 100,
                summary: "反思功能未启用".to_string(),
                suggestions: vec![],
                needs_more_context: false,
                missing_context: None,
                confidence: 100,
            });
        }

        info!("🤔 [AgentReflection] 开始反思评估...");
        let start_time = Instant::now();

        // 构建反思提示词
        let reflection_prompt = self.build_reflection_prompt(
            user_question,
            ai_answer,
            context_docs,
        );

        info!("🤔 [AgentReflection] 反思提示词长度: {} 字符", reflection_prompt.len());

        // 调用 LLM 进行反思（TODO: 实际实现需要集成到 AIManager）
        let result = self.call_reflection_llm(&reflection_prompt).await;

        let duration = start_time.elapsed();
        info!("🤔 [AgentReflection] 反思完成，耗时: {:?}", duration);

        result
    }

    /// 构建反思提示词
    fn build_reflection_prompt(
        &self,
        user_question: &str,
        ai_answer: &str,
        context_docs: &[SqliteEmbeddedDocument],
    ) -> String {
        // 统计上下文信息
        let context_summary = if context_docs.is_empty() {
            "没有检索到相关文档".to_string()
        } else {
            let total_length: usize = context_docs.iter()
                .map(|doc| doc.fragments.iter().map(|f| f.content.len()).sum::<usize>())
                .sum();
            format!(
                "检索到 {} 个文档，{} 个片段，总长度约 {} 字符",
                context_docs.len(),
                context_docs.iter().map(|d| d.fragments.len()).sum::<usize>(),
                total_length
            )
        };

        format!(
            r#"请作为智能体反思专家，评估以下AI回答的质量。

## 用户问题
{}

## AI的回答
{}

## 检索到的上下文
{}

## 评估任务
请从以下维度评估这个回答：

1. **准确性** - 回答是否准确？是否有明显的错误？
2. **完整性** - 回答是否完整地解决了用户问题？是否有遗漏的关键点？
3. **相关性** - 回答是否与用户问题高度相关？
4. **上下文利用** - 是否有效利用了检索到的上下文信息？
5. **清晰度** - 回答是否清晰易懂？

## 输出格式要求
请按照以下JSON格式输出你的评估结果：

```json
{{
    "is_resolved": true/false,
    "quality_score": 0-100,
    "summary": "一句话总结回答质量",
    "suggestions": ["改进建议1", "改进建议2"],
    "needs_more_context": true/false,
    "missing_context": "如果缺少上下文，请描述需要什么信息",
    "confidence": 0-100
}}
```

注意：
- is_resolved: 如果问题被完全解决则为true，否则为false
- quality_score: 0-100的分数，100为完美
- summary: 简要总结回答质量
- suggestions: 具体的改进建议列表
- needs_more_context: 是否需要更多上下文信息
- missing_context: 如果needs_more_context为true，请说明需要什么上下文
- confidence: 你对这个评估结果的置信度，0-100

请只输出JSON，不要输出其他内容。"#,
            user_question,
            ai_answer,
            context_summary
        )
    }

    /// 调用 LLM 进行反思
    async fn call_reflection_llm(
        &self,
        _prompt: &str,
    ) -> FlowyResult<ReflectionResult> {
        // 注意：这里我们需要调用 LLM，但是需要集成到 AIManager 中
        // 实际实现中，我们需要一个更好的方式来调用 LLM
        // 这里先返回一个简化版本的实现

        warn!("⚠️ [AgentReflection] 直接调用 LLM 功能暂时未实现，返回启发式评估结果");

        // TODO: 实现实际的 LLM 调用
        // 这需要访问 AI Manager 的完整流式调用能力
        // 目前返回一个基于启发式规则的评估结果
        
        let result = self.evaluate_with_heuristics();
        
        Ok(result)
    }

    /// 使用启发式规则进行评估（临时方案）
    fn evaluate_with_heuristics(&self) -> ReflectionResult {
        // 这是一个简化的评估逻辑
        // 实际应用时应该调用 LLM 进行真正的反思
        
        ReflectionResult {
            is_resolved: true,
            quality_score: 75,
            summary: "启发式评估：回答基本完整".to_string(),
            suggestions: vec![
                "可以考虑添加更多具体的例子".to_string(),
                "可以更详细地解释关键概念".to_string(),
            ],
            needs_more_context: false,
            missing_context: None,
            confidence: 70,
        }
    }

    /// 判断是否需要继续迭代
    /// 
    /// 根据反思结果决定是否需要让智能体继续改进回答
    pub fn should_continue_iteration(&self, result: &ReflectionResult) -> bool {
        if !self.config.enabled {
            return false;
        }

        // 如果问题未解决，或者质量评分低于阈值，或者置信度低于阈值
        !result.is_resolved 
            || result.quality_score < self.config.quality_threshold
            || result.confidence < self.config.confidence_threshold
    }

    /// 生成改进提示词
    /// 
    /// 根据反思结果生成用于改进下一轮回答的提示词
    pub fn generate_improvement_prompt(&self, result: &ReflectionResult) -> String {
        if result.suggestions.is_empty() {
            return String::new();
        }

        let mut prompt = "请根据以下反馈改进你的回答：\n".to_string();
        
        prompt.push_str(&format!("反思总结：{}\n\n", result.summary));
        
        if result.needs_more_context {
            if let Some(ref missing) = result.missing_context {
                prompt.push_str(&format!("需要更多上下文：{}\n\n", missing));
            }
        }
        
        prompt.push_str("改进建议：\n");
        for (i, suggestion) in result.suggestions.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", i + 1, suggestion));
        }
        
        prompt
    }

    /// 评估上下文是否充分
    /// 
    /// 根据检索到的文档数量和质量判断是否需要更多上下文
    pub fn evaluate_context_adequacy(
        &self,
        context_docs: &[SqliteEmbeddedDocument],
    ) -> (bool, Option<String>) {
        if context_docs.is_empty() {
            return (false, Some("没有检索到任何文档".to_string()));
        }

        let total_fragments: usize = context_docs.iter().map(|d| d.fragments.len()).sum();
        let total_length: usize = context_docs.iter()
            .map(|doc| doc.fragments.iter().map(|f| f.content.len()).sum::<usize>())
            .sum();
        let avg_length = total_length / total_fragments.max(1);
        
        // 如果文档太少或太短，认为上下文不足
        if context_docs.len() < 2 || avg_length < 100 {
            return (
                false,
                Some(format!(
                    "只检索到 {} 个文档，{} 个片段，平均长度 {} 字符，可能不够充分",
                    context_docs.len(),
                    total_fragments,
                    avg_length
                )),
            );
        }

        (true, None)
    }

    /// 合并多个反思结果
    /// 
    /// 当进行多次反思时，合并多个结果的反馈
    pub fn merge_reflection_results(&self, results: &[ReflectionResult]) -> ReflectionResult {
        if results.is_empty() {
            return ReflectionResult {
                is_resolved: true,
                quality_score: 100,
                summary: "没有反思结果".to_string(),
                suggestions: vec![],
                needs_more_context: false,
                missing_context: None,
                confidence: 100,
            };
        }

        let is_resolved = results.iter().any(|r| r.is_resolved);
        let quality_score = (results.iter().map(|r| r.quality_score as u16).sum::<u16>() 
            / results.len() as u16) as u8;
        let confidence = (results.iter().map(|r| r.confidence as u16).sum::<u16>() 
            / results.len() as u16) as u8;

        // 收集所有建议
        let mut all_suggestions = Vec::new();
        for result in results {
            all_suggestions.extend(result.suggestions.iter().cloned());
        }

        // 去重
        all_suggestions.sort();
        all_suggestions.dedup();

        // 找到最常见的需要上下文的情况
        let needs_more_context = results.iter().any(|r| r.needs_more_context);
        let missing_context = results.iter()
            .find(|r| r.needs_more_context && r.missing_context.is_some())
            .and_then(|r| r.missing_context.clone());

        ReflectionResult {
            is_resolved,
            quality_score,
            summary: format!("综合 {} 次反思结果", results.len()),
            suggestions: all_suggestions,
            needs_more_context,
            missing_context,
            confidence,
        }
    }
}

// TODO: 实际集成时将需要以下功能：
// 1. 调用 LLM 生成反思
// 2. 解析 JSON 格式的反思结果
// 3. 处理流式响应
//
// 这些功能应该通过 AIManager 的现有能力来实现，
// 而不是通过一个新的 trait

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_reflection() -> AgentReflection {
        let config = ReflectionConfig {
            enabled: true,
            max_iterations: 3,
            timeout_seconds: 30,
            quality_threshold: 70,
            confidence_threshold: 60,
        };
        AgentReflection::new(config)
    }

    #[test]
    fn test_config_from_capabilities() {
        let mut capabilities = AgentCapabilitiesPB::default();
        capabilities.enable_reflection = true;
        capabilities.max_reflection_iterations = 5;

        let config = ReflectionConfig::from_capabilities(&capabilities);
        assert!(config.enabled);
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.quality_threshold, 70);
    }

    #[test]
    fn test_should_continue_iteration() {
        let reflection = create_test_reflection();

        // 高质量结果，应该停止迭代
        let good_result = ReflectionResult {
            is_resolved: true,
            quality_score: 90,
            summary: "很好".to_string(),
            suggestions: vec![],
            needs_more_context: false,
            missing_context: None,
            confidence: 80,
        };
        assert!(!reflection.should_continue_iteration(&good_result));

        // 低质量结果，应该继续迭代
        let poor_result = ReflectionResult {
            is_resolved: false,
            quality_score: 50,
            summary: "不够好".to_string(),
            suggestions: vec!["需要改进".to_string()],
            needs_more_context: true,
            missing_context: Some("需要更多信息".to_string()),
            confidence: 40,
        };
        assert!(reflection.should_continue_iteration(&poor_result));
    }

    #[test]
    fn test_evaluate_context_adequacy() {
        let reflection = create_test_reflection();

        // 空上下文
        let empty_docs = vec![];
        let (adequate, reason) = reflection.evaluate_context_adequacy(&empty_docs);
        assert!(!adequate);
        assert!(reason.is_some());

        // 足够的上下文
        use flowy_sqlite_vec::entities::SqliteEmbeddedFragment;
        let docs = vec![
            SqliteEmbeddedDocument {
                workspace_id: "workspace1".to_string(),
                object_id: "1".to_string(),
                fragments: vec![
                    SqliteEmbeddedFragment {
                        content: "这是一个足够长的文档片段，包含了有用的信息。".repeat(20),
                        embeddings: vec![],
                    }
                ],
            },
            SqliteEmbeddedDocument {
                workspace_id: "workspace1".to_string(),
                object_id: "2".to_string(),
                fragments: vec![
                    SqliteEmbeddedFragment {
                        content: "这是另一个足够长的文档片段。".repeat(20),
                        embeddings: vec![],
                    }
                ],
            },
        ];
        let (adequate, reason) = reflection.evaluate_context_adequacy(&docs);
        assert!(adequate);
        assert!(reason.is_none());
    }

    #[test]
    fn test_merge_reflection_results() {
        let reflection = create_test_reflection();

        let results = vec![
            ReflectionResult {
                is_resolved: true,
                quality_score: 80,
                summary: "第一次评估".to_string(),
                suggestions: vec!["建议1".to_string(), "建议2".to_string()],
                needs_more_context: false,
                missing_context: None,
                confidence: 70,
            },
            ReflectionResult {
                is_resolved: true,
                quality_score: 70,
                summary: "第二次评估".to_string(),
                suggestions: vec!["建议2".to_string(), "建议3".to_string()],
                needs_more_context: false,
                missing_context: None,
                confidence: 75,
            },
        ];

        let merged = reflection.merge_reflection_results(&results);
        assert_eq!(merged.is_resolved, true);
        assert_eq!(merged.quality_score, 75); // (80 + 70) / 2
        assert_eq!(merged.confidence, 72); // (70 + 75) / 2
        assert_eq!(merged.suggestions.len(), 3); // 去重后的建议
    }
}


