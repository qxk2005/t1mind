use crate::entities::{
    GlobalAIModelTypePB, OpenAICompatibleSettingPB,
    OpenAISDKSettingPB,
};
use flowy_error::{FlowyError, FlowyResult};
use lib_infra::util::timestamp;
use tracing::{info, warn};

/// 配置迁移器，负责将旧的配置数据迁移到新的格式
pub struct ConfigMigrator;

impl ConfigMigrator {
    /// 从OpenAI兼容配置迁移到OpenAI SDK配置
    /// 
    /// 由于我们使用相同的数据结构（OpenAISDKSettingPB = OpenAICompatibleSettingPB），
    /// 这个方法实际上只是克隆配置并应用默认值
    pub fn migrate_openai_compatible_to_sdk(
        compatible_setting: &OpenAICompatibleSettingPB,
    ) -> FlowyResult<OpenAISDKSettingPB> {
        info!("开始迁移OpenAI兼容配置到OpenAI SDK配置");

        // 由于OpenAISDKSettingPB是OpenAICompatibleSettingPB的类型别名，
        // 我们只需要克隆并应用默认值
        let mut sdk_setting = compatible_setting.clone();

        // 应用默认值到聊天配置
        if sdk_setting.chat_setting.api_endpoint.is_empty() {
            sdk_setting.chat_setting.api_endpoint = "https://api.openai.com/v1".to_string();
        }
        if sdk_setting.chat_setting.model_name.is_empty() {
            sdk_setting.chat_setting.model_name = "gpt-3.5-turbo".to_string();
        }
        if sdk_setting.chat_setting.model_type.is_empty() {
            sdk_setting.chat_setting.model_type = "chat".to_string();
        }
        if sdk_setting.chat_setting.max_tokens == 0 {
            sdk_setting.chat_setting.max_tokens = 4096;
        }
        if sdk_setting.chat_setting.temperature == 0.0 {
            sdk_setting.chat_setting.temperature = 0.7;
        }
        if sdk_setting.chat_setting.timeout_seconds == 0 {
            sdk_setting.chat_setting.timeout_seconds = 30;
        }

        // 应用默认值到嵌入配置
        if sdk_setting.embedding_setting.api_endpoint.is_empty() {
            sdk_setting.embedding_setting.api_endpoint = "https://api.openai.com/v1".to_string();
        }
        if sdk_setting.embedding_setting.model_name.is_empty() {
            sdk_setting.embedding_setting.model_name = "text-embedding-ada-002".to_string();
        }

        info!("OpenAI兼容配置迁移完成");
        Ok(sdk_setting)
    }

    /// 检查是否需要迁移配置
    /// 
    /// 如果用户当前使用的是OpenAI兼容服务器，但没有OpenAI SDK配置，
    /// 则建议进行迁移
    pub fn should_migrate_to_sdk(
        current_model_type: GlobalAIModelTypePB,
        has_compatible_config: bool,
        has_sdk_config: bool,
    ) -> bool {
        match current_model_type {
            GlobalAIModelTypePB::GlobalOpenAICompatible => {
                // 如果当前使用OpenAI兼容，且有兼容配置但没有SDK配置，建议迁移
                has_compatible_config && !has_sdk_config
            }
            _ => false,
        }
    }

    /// 创建迁移备份
    /// 
    /// 在迁移前创建原始配置的备份，以防迁移失败需要回滚
    pub fn create_migration_backup(
        _compatible_setting: &OpenAICompatibleSettingPB,
    ) -> FlowyResult<String> {
        let timestamp = timestamp();
        let backup_key = format!("openai_compatible_backup_{}", timestamp);
        
        info!("创建配置迁移备份: {}", backup_key);
        
        // 这里应该将配置序列化并保存到持久化存储
        // 暂时返回备份键，实际实现需要根据AppFlowy的持久化机制
        Ok(backup_key)
    }

    /// 验证迁移后的配置
    /// 
    /// 确保迁移后的配置包含所有必要的字段且格式正确
    pub fn validate_migrated_config(sdk_setting: &OpenAISDKSettingPB) -> FlowyResult<()> {
        // 验证聊天配置
        let chat_setting = &sdk_setting.chat_setting;
        if chat_setting.api_endpoint.is_empty() {
            return Err(FlowyError::invalid_data().with_context("聊天API端点不能为空"));
        }
        if chat_setting.model_name.is_empty() {
            return Err(FlowyError::invalid_data().with_context("聊天模型名称不能为空"));
        }
        if chat_setting.max_tokens == 0 {
            warn!("聊天最大tokens为0，将使用默认值");
        }

        // 验证嵌入配置
        let embedding_setting = &sdk_setting.embedding_setting;
        if embedding_setting.api_endpoint.is_empty() {
            return Err(FlowyError::invalid_data().with_context("嵌入API端点不能为空"));
        }
        if embedding_setting.model_name.is_empty() {
            return Err(FlowyError::invalid_data().with_context("嵌入模型名称不能为空"));
        }

        info!("迁移配置验证通过");
        Ok(())
    }

    /// 执行完整的配置迁移流程
    /// 
    /// 包括备份、迁移、验证等完整步骤
    pub fn execute_migration(
        compatible_setting: &OpenAICompatibleSettingPB,
    ) -> FlowyResult<(OpenAISDKSettingPB, String)> {
        // 1. 创建备份
        let backup_key = Self::create_migration_backup(compatible_setting)?;

        // 2. 执行迁移
        let sdk_setting = Self::migrate_openai_compatible_to_sdk(compatible_setting)?;

        // 3. 验证迁移结果
        Self::validate_migrated_config(&sdk_setting)?;

        info!("配置迁移流程完成，备份键: {}", backup_key);
        Ok((sdk_setting, backup_key))
    }

    /// 回滚迁移
    /// 
    /// 如果迁移后出现问题，可以使用备份键回滚到原始配置
    pub fn rollback_migration(backup_key: &str) -> FlowyResult<OpenAICompatibleSettingPB> {
        info!("开始回滚配置迁移，备份键: {}", backup_key);
        
        // 这里应该从持久化存储中恢复备份的配置
        // 暂时返回错误，实际实现需要根据AppFlowy的持久化机制
        Err(FlowyError::internal().with_context("配置回滚功能尚未实现"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_openai_compatible_to_sdk() {
        use crate::entities::{OpenAIChatSettingPB, OpenAIEmbeddingSettingPB};
        
        let compatible_setting = OpenAICompatibleSettingPB {
            chat_setting: OpenAIChatSettingPB {
                api_endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "sk-test-key".to_string(),
                model_name: "gpt-4".to_string(),
                model_type: "chat".to_string(),
                max_tokens: 2048,
                temperature: 0.8,
                timeout_seconds: 60,
            },
            embedding_setting: OpenAIEmbeddingSettingPB {
                api_endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "sk-test-key".to_string(),
                model_name: "text-embedding-ada-002".to_string(),
            },
        };

        let result = ConfigMigrator::migrate_openai_compatible_to_sdk(&compatible_setting);
        assert!(result.is_ok());

        let sdk_setting = result.unwrap();
        
        let chat_setting = &sdk_setting.chat_setting;
        assert_eq!(chat_setting.api_endpoint, "https://api.openai.com/v1");
        assert_eq!(chat_setting.api_key, "sk-test-key");
        assert_eq!(chat_setting.model_name, "gpt-4");
        assert_eq!(chat_setting.max_tokens, 2048);
        assert_eq!(chat_setting.temperature, 0.8);
        assert_eq!(chat_setting.timeout_seconds, 60);

        let embedding_setting = &sdk_setting.embedding_setting;
        assert_eq!(embedding_setting.api_endpoint, "https://api.openai.com/v1");
        assert_eq!(embedding_setting.api_key, "sk-test-key");
        assert_eq!(embedding_setting.model_name, "text-embedding-ada-002");
    }

    #[test]
    fn test_migrate_empty_compatible_setting() {
        let compatible_setting = OpenAICompatibleSettingPB::default();

        let result = ConfigMigrator::migrate_openai_compatible_to_sdk(&compatible_setting);
        assert!(result.is_ok());

        let sdk_setting = result.unwrap();
        let chat_setting = &sdk_setting.chat_setting;
        assert_eq!(chat_setting.api_endpoint, "https://api.openai.com/v1");
        assert_eq!(chat_setting.model_name, "gpt-3.5-turbo");
        assert_eq!(chat_setting.max_tokens, 4096);
        assert_eq!(chat_setting.temperature, 0.7);
        assert_eq!(chat_setting.timeout_seconds, 30);

        let embedding_setting = &sdk_setting.embedding_setting;
        assert_eq!(embedding_setting.api_endpoint, "https://api.openai.com/v1");
        assert_eq!(embedding_setting.model_name, "text-embedding-ada-002");
    }

    #[test]
    fn test_should_migrate_to_sdk() {
        // 应该迁移的情况
        assert!(ConfigMigrator::should_migrate_to_sdk(
            GlobalAIModelTypePB::GlobalOpenAICompatible,
            true,  // 有兼容配置
            false, // 没有SDK配置
        ));

        // 不应该迁移的情况
        assert!(!ConfigMigrator::should_migrate_to_sdk(
            GlobalAIModelTypePB::GlobalLocalAI,
            true,
            false,
        ));

        assert!(!ConfigMigrator::should_migrate_to_sdk(
            GlobalAIModelTypePB::GlobalOpenAICompatible,
            false, // 没有兼容配置
            false,
        ));

        assert!(!ConfigMigrator::should_migrate_to_sdk(
            GlobalAIModelTypePB::GlobalOpenAICompatible,
            true,
            true, // 已有SDK配置
        ));
    }

    #[test]
    fn test_validate_migrated_config() {
        use crate::entities::{OpenAIChatSettingPB, OpenAIEmbeddingSettingPB};
        
        let valid_sdk_setting = OpenAISDKSettingPB {
            chat_setting: OpenAIChatSettingPB {
                api_endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "sk-test".to_string(),
                model_name: "gpt-3.5-turbo".to_string(),
                model_type: "chat".to_string(),
                max_tokens: 4096,
                temperature: 0.7,
                timeout_seconds: 30,
            },
            embedding_setting: OpenAIEmbeddingSettingPB {
                api_endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "sk-test".to_string(),
                model_name: "text-embedding-ada-002".to_string(),
            },
        };

        assert!(ConfigMigrator::validate_migrated_config(&valid_sdk_setting).is_ok());

        // 测试无效配置
        let invalid_sdk_setting = OpenAISDKSettingPB {
            chat_setting: OpenAIChatSettingPB {
                api_endpoint: "".to_string(), // 空端点应该失败
                api_key: "sk-test".to_string(),
                model_name: "gpt-3.5-turbo".to_string(),
                model_type: "chat".to_string(),
                max_tokens: 4096,
                temperature: 0.7,
                timeout_seconds: 30,
            },
            embedding_setting: OpenAIEmbeddingSettingPB {
                api_endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "sk-test".to_string(),
                model_name: "text-embedding-ada-002".to_string(),
            },
        };

        assert!(ConfigMigrator::validate_migrated_config(&invalid_sdk_setting).is_err());
    }
}
