use std::sync::Arc;

use flowy_error::{FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use tracing::{debug, error, info};

use crate::entities::{RAGSettingsPB, UpdateRAGSettingsRequestPB};

/// RAG配置管理器的存储键
const RAG_GLOBAL_SETTINGS_KEY: &str = "rag_global_settings";
const RAG_VERSION_KEY: &str = "rag_config_version";

/// 当前配置版本
const CURRENT_CONFIG_VERSION: u32 = 1;

/// RAG配置管理器
/// 
/// 负责管理RAG全局配置，包括文档切块、检索策略、重排序等设置。
/// 配置存储在KVStorePreferences中，使用键"rag_global_settings"。
pub struct RAGConfigManager {
    store_preferences: Arc<KVStorePreferences>,
}

impl RAGConfigManager {
    /// 创建新的RAG配置管理器
    /// 
    /// # 参数
    /// * `store_preferences` - KVStorePreferences实例，用于持久化配置
    /// 
    /// # 返回
    /// 返回初始化的RAGConfigManager实例
    pub fn new(store_preferences: Arc<KVStorePreferences>) -> Self {
        let manager = Self { store_preferences };
        
        // 检查并执行配置迁移
        if let Err(e) = manager.migrate_config_if_needed() {
            error!("Failed to migrate RAG config: {}", e);
        }
        
        manager
    }

    /// 获取RAG全局设置
    /// 
    /// # 返回
    /// 返回当前RAG设置。如果不存在，返回默认配置。
    pub fn get_rag_settings(&self) -> RAGSettingsPB {
        self.store_preferences
            .get_object::<RAGSettingsPB>(RAG_GLOBAL_SETTINGS_KEY)
            .unwrap_or_else(|| {
                debug!("No RAG settings found, returning default configuration");
                RAGSettingsPB::default_config()
            })
    }

    /// 保存RAG全局设置
    /// 
    /// # 参数
    /// * `settings` - 要保存的RAG设置
    /// 
    /// # 返回
    /// * `Ok(())` - 保存成功
    /// * `Err(FlowyError)` - 保存失败，包含错误信息
    /// 
    /// # 错误处理
    /// 在保存前会验证配置的有效性。如果配置无效，返回错误。
    pub fn save_rag_settings(&self, mut settings: RAGSettingsPB) -> FlowyResult<()> {
        // 验证配置
        self.validate_settings(&settings)?;
        
        // 更新时间戳
        use chrono::Utc;
        settings.updated_at = Utc::now().timestamp();
        
        // 保存到KVStorePreferences
        self.store_preferences
            .set_object(RAG_GLOBAL_SETTINGS_KEY, &settings)
            .map_err(|e| {
                error!("Failed to save RAG global settings: {}", e);
                FlowyError::internal().with_context(format!("保存RAG全局设置失败: {}", e))
            })?;
        
        info!("RAG global settings saved successfully");
        debug!("Saved RAG settings: chunk_size={}, chunk_overlap={}, enable_hybrid_search={}", 
               settings.chunk_size, settings.chunk_overlap, settings.enable_hybrid_search);
        Ok(())
    }

    /// 验证RAG设置的有效性
    /// 
    /// # 参数
    /// * `settings` - 要验证的RAG设置
    /// 
    /// # 返回
    /// * `Ok(())` - 配置有效
    /// * `Err(FlowyError)` - 配置无效，包含错误信息
    /// 
    /// # 验证规则
    /// 1. `chunk_size` > 0
    /// 2. `chunk_overlap` >= 0 且 < `chunk_size`
    /// 3. `vector_weight` >= 0
    /// 4. `keyword_weight` >= 0
    /// 5. `vector_weight + keyword_weight` 约等于 1.0（允许小的浮点误差）
    /// 6. `initial_top_k` > 0
    /// 7. `final_top_k` > 0 且 <= `initial_top_k`
    /// 8. `reflection_threshold` 在 [0, 1] 范围内
    pub fn validate_settings(&self, settings: &RAGSettingsPB) -> FlowyResult<()> {
        // 验证 chunk_size
        if settings.chunk_size <= 0 {
            return Err(FlowyError::invalid_data()
                .with_context(format!("chunk_size 必须大于 0，当前值: {}", settings.chunk_size)));
        }

        // 验证 chunk_overlap
        if settings.chunk_overlap < 0 {
            return Err(FlowyError::invalid_data()
                .with_context(format!("chunk_overlap 不能小于 0，当前值: {}", settings.chunk_overlap)));
        }
        
        if settings.chunk_overlap >= settings.chunk_size {
            return Err(FlowyError::invalid_data()
                .with_context(format!("chunk_overlap ({}) 必须小于 chunk_size ({})", 
                    settings.chunk_overlap, settings.chunk_size)));
        }

        // 验证权重
        if settings.vector_weight < 0.0 || settings.keyword_weight < 0.0 {
            return Err(FlowyError::invalid_data()
                .with_context("权重值不能为负数"));
        }

        // 验证权重总和约为 1.0（允许小的浮点误差）
        let weight_sum = settings.vector_weight + settings.keyword_weight;
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(FlowyError::invalid_data()
                .with_context(format!("vector_weight ({}) + keyword_weight ({}) 必须约等于 1.0，当前总和: {:.3}", 
                    settings.vector_weight, settings.keyword_weight, weight_sum)));
        }

        // 验证 top_k 参数
        if settings.initial_top_k <= 0 {
            return Err(FlowyError::invalid_data()
                .with_context(format!("initial_top_k 必须大于 0，当前值: {}", settings.initial_top_k)));
        }

        if settings.final_top_k <= 0 {
            return Err(FlowyError::invalid_data()
                .with_context(format!("final_top_k 必须大于 0，当前值: {}", settings.final_top_k)));
        }

        if settings.final_top_k > settings.initial_top_k {
            return Err(FlowyError::invalid_data()
                .with_context(format!("final_top_k ({}) 必须小于或等于 initial_top_k ({})", 
                    settings.final_top_k, settings.initial_top_k)));
        }

        // 验证 reflection_threshold
        if settings.reflection_threshold < 0.0 || settings.reflection_threshold > 1.0 {
            return Err(FlowyError::invalid_data()
                .with_context(format!("reflection_threshold 必须在 [0, 1] 范围内，当前值: {}", 
                    settings.reflection_threshold)));
        }

        debug!("RAG settings validation passed");
        Ok(())
    }

    /// 更新RAG设置（部分更新）
    /// 
    /// 从现有设置中读取，应用更新请求，然后保存。
    /// 
    /// # 参数
    /// * `update` - 包含要更新的字段的请求
    /// 
    /// # 返回
    /// * `Ok(RAGSettingsPB)` - 更新后的设置
    /// * `Err(FlowyError)` - 更新失败
    pub fn update_rag_settings(&self, update: UpdateRAGSettingsRequestPB) -> FlowyResult<RAGSettingsPB> {
        // 获取当前设置
        let mut settings = self.get_rag_settings();
        
        // 应用更新
        settings.update_from(update);
        
        // 验证并保存
        self.save_rag_settings(settings.clone())?;
        
        info!("RAG settings updated successfully");
        Ok(settings)
    }

    /// 重置为默认配置
    /// 
    /// # 返回
    /// * `Ok(RAGSettingsPB)` - 默认设置
    /// * `Err(FlowyError)` - 重置失败
    pub fn reset_to_default(&self) -> FlowyResult<RAGSettingsPB> {
        let default_settings = RAGSettingsPB::default_config();
        self.save_rag_settings(default_settings.clone())?;
        
        info!("RAG settings reset to default");
        Ok(default_settings)
    }

    /// 检查并执行配置迁移
    /// 
    /// 当配置版本更新时，可以在这里添加迁移逻辑。
    fn migrate_config_if_needed(&self) -> FlowyResult<()> {
        let current_version = self.store_preferences
            .get_object::<u32>(RAG_VERSION_KEY)
            .unwrap_or(0);
        
        if current_version < CURRENT_CONFIG_VERSION {
            info!("Migrating RAG config from version {} to {}", 
                  current_version, CURRENT_CONFIG_VERSION);
            
            // 这里可以添加版本迁移逻辑
            // 目前只是更新版本号
            self.store_preferences
                .set_object(RAG_VERSION_KEY, &CURRENT_CONFIG_VERSION)
                .map_err(|e| {
                    error!("Failed to update RAG config version: {}", e);
                    FlowyError::internal().with_context("更新配置版本失败")
                })?;
            
            info!("RAG config migration completed");
        }
        
        Ok(())
    }

    /// 获取配置版本
    /// 
    /// # 返回
    /// 返回当前配置版本号
    pub fn get_config_version(&self) -> u32 {
        self.store_preferences
            .get_object::<u32>(RAG_VERSION_KEY)
            .unwrap_or(0)
    }

    /// 检查配置是否存在
    /// 
    /// # 返回
    /// 如果配置已存在返回 true，否则返回 false
    pub fn has_config(&self) -> bool {
        self.store_preferences
            .get_object::<RAGSettingsPB>(RAG_GLOBAL_SETTINGS_KEY)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowy_sqlite::kv::KVStorePreferences;
    use tempfile::TempDir;
    use std::collections::HashMap;

    fn create_test_config_manager() -> (RAGConfigManager, TempDir) {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let store = Arc::new(KVStorePreferences::new(path).unwrap());
        let manager = RAGConfigManager::new(store);
        (manager, tempdir)
    }

    #[test]
    fn test_default_settings() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试默认设置
        let default_settings = manager.get_rag_settings();
        assert_eq!(default_settings.chunk_size, 1000);
        assert_eq!(default_settings.chunk_overlap, 200);
        assert_eq!(default_settings.enable_hybrid_search, true);
        assert_eq!(default_settings.vector_weight, 0.7);
        assert_eq!(default_settings.keyword_weight, 0.3);
    }

    #[test]
    fn test_save_and_load_settings() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 创建测试配置
        let mut settings = RAGSettingsPB::default_config();
        settings.chunk_size = 2000;
        settings.chunk_overlap = 400;
        
        // 保存设置
        manager.save_rag_settings(settings.clone()).unwrap();
        
        // 读取设置
        let loaded_settings = manager.get_rag_settings();
        assert_eq!(loaded_settings.chunk_size, 2000);
        assert_eq!(loaded_settings.chunk_overlap, 400);
    }

    #[test]
    fn test_validation_chunk_size() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试 chunk_size <= 0
        let mut invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.chunk_size = 0;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
        
        invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.chunk_size = -1;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
    }

    #[test]
    fn test_validation_chunk_overlap() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试 chunk_overlap >= chunk_size
        let mut invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.chunk_overlap = 1000;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
        
        // 测试 chunk_overlap < 0
        invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.chunk_overlap = -1;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
    }

    #[test]
    fn test_validation_weights() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试权重和不等于 1
        let mut invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.vector_weight = 0.5;
        invalid_settings.keyword_weight = 0.3;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
        
        // 测试负数权重
        invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.vector_weight = -0.1;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
    }

    #[test]
    fn test_validation_top_k() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试 final_top_k > initial_top_k
        let mut invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.final_top_k = 20;
        invalid_settings.initial_top_k = 10;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
        
        // 测试 top_k <= 0
        invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.initial_top_k = 0;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
    }

    #[test]
    fn test_validation_reflection_threshold() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 测试 reflection_threshold 超出范围
        let mut invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.reflection_threshold = 1.5;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
        
        invalid_settings = RAGSettingsPB::default_config();
        invalid_settings.reflection_threshold = -0.1;
        assert!(manager.save_rag_settings(invalid_settings).is_err());
    }

    #[test]
    fn test_update_settings() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 更新部分设置
        let update = UpdateRAGSettingsRequestPB {
            chunk_size: Some(1500),
            chunk_overlap: Some(300),
            enable_semantic_splitting: None,
            enable_hybrid_search: None,
            vector_weight: None,
            keyword_weight: None,
            initial_top_k: None,
            final_top_k: None,
            enable_reranking: None,
            reranker_model: None,
            enable_agent_reflection: None,
            reflection_threshold: None,
            metadata: HashMap::new(),
        };
        
        let updated_settings = manager.update_rag_settings(update).unwrap();
        assert_eq!(updated_settings.chunk_size, 1500);
        assert_eq!(updated_settings.chunk_overlap, 300);
        // 其他字段应保持不变
        assert_eq!(updated_settings.enable_hybrid_search, true);
    }

    #[test]
    fn test_reset_to_default() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 先设置自定义配置
        let mut custom_settings = RAGSettingsPB::default_config();
        custom_settings.chunk_size = 2000;
        manager.save_rag_settings(custom_settings).unwrap();
        
        // 重置为默认值
        let default_settings = manager.reset_to_default().unwrap();
        assert_eq!(default_settings.chunk_size, 1000);
        
        // 验证已保存
        let loaded_settings = manager.get_rag_settings();
        assert_eq!(loaded_settings.chunk_size, 1000);
    }

    #[test]
    fn test_valid_config() {
        let (manager, _tempdir) = create_test_config_manager();
        
        let valid_settings = RAGSettingsPB::default_config();
        
        // 验证应该通过
        assert!(manager.validate_settings(&valid_settings).is_ok());
    }

    #[test]
    fn test_has_config() {
        let (manager, _tempdir) = create_test_config_manager();
        
        // 初始状态应该没有配置（会返回默认值）
        // 注意：由于 get_rag_settings 返回默认值，我们需要检查 has_config
        // 第一次调用 has_config 应该返回 false
        assert!(!manager.has_config());
        
        // 保存配置后应该返回 true
        let settings = RAGSettingsPB::default_config();
        manager.save_rag_settings(settings).unwrap();
        assert!(manager.has_config());
    }
}

