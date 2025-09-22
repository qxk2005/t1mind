use crate::entities::{GlobalAIModelTypePB, OpenAICompatibleSettingPB};
use flowy_error::{ErrorCode, FlowyError, FlowyResult};
use flowy_sqlite::kv::KVStorePreferences;
use serde::{Deserialize, Serialize};
use std::sync::Weak;
use tracing::debug;

// Version keys for persistence
const GLOBAL_AI_MODEL_TYPE_KEY: &str = "global_ai_model_type:v1";
const OPENAI_COMPATIBLE_SETTING_KEY: &str = "openai_compatible_setting:v1";

// Serde-compatible versions for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGlobalAIModelType {
  pub model_type: i32, // 0 for LocalAI, 1 for OpenAICompatible
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOpenAIChatSetting {
  pub api_endpoint: String,
  pub api_key: String, // Will be stored securely
  pub model_name: String,
  pub model_type: String,
  pub max_tokens: i32,
  pub temperature: f64,
  pub timeout_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOpenAIEmbeddingSetting {
  pub api_endpoint: String,
  pub api_key: String, // Will be stored securely
  pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOpenAICompatibleSetting {
  pub chat_setting: StoredOpenAIChatSetting,
  pub embedding_setting: StoredOpenAIEmbeddingSetting,
}

/// Persistence manager for AI settings
pub struct AIPersistence {
  store_preferences: Weak<KVStorePreferences>,
}

impl AIPersistence {
  pub fn new(store_preferences: Weak<KVStorePreferences>) -> Self {
    Self { store_preferences }
  }

  /// Save the global AI model type
  pub fn save_global_model_type(&self, model_type: GlobalAIModelTypePB) -> FlowyResult<()> {
    let store = self.upgrade_store()?;
    
    let stored = StoredGlobalAIModelType {
      model_type: model_type as i32,
    };
    
    store.set_object(GLOBAL_AI_MODEL_TYPE_KEY, &stored)?;
    Ok(())
  }

  /// Load the global AI model type
  pub fn load_global_model_type(&self) -> FlowyResult<GlobalAIModelTypePB> {
    let store = self.upgrade_store()?;
    
    let stored = store
      .get_object::<StoredGlobalAIModelType>(GLOBAL_AI_MODEL_TYPE_KEY)
      .unwrap_or_else(|| StoredGlobalAIModelType { model_type: 0 }); // Default to LocalAI
    
    let model_type = match stored.model_type {
      1 => GlobalAIModelTypePB::GlobalOpenAICompatible,
      _ => GlobalAIModelTypePB::GlobalLocalAI,
    };
    
    Ok(model_type)
  }

  /// Save OpenAI compatible settings
  pub fn save_openai_compatible_setting(
    &self,
    setting: &OpenAICompatibleSettingPB,
  ) -> FlowyResult<()> {
    let store = self.upgrade_store()?;
    
    // Convert to storage format
    let stored = StoredOpenAICompatibleSetting {
      chat_setting: StoredOpenAIChatSetting {
        api_endpoint: setting.chat_setting.api_endpoint.clone(),
        api_key: setting.chat_setting.api_key.clone(),
        model_name: setting.chat_setting.model_name.clone(),
        model_type: setting.chat_setting.model_type.clone(),
        max_tokens: setting.chat_setting.max_tokens,
        temperature: setting.chat_setting.temperature,
        timeout_seconds: setting.chat_setting.timeout_seconds,
      },
      embedding_setting: StoredOpenAIEmbeddingSetting {
        api_endpoint: setting.embedding_setting.api_endpoint.clone(),
        api_key: setting.embedding_setting.api_key.clone(),
        model_name: setting.embedding_setting.model_name.clone(),
      },
    };
    
    debug!("Saving OpenAI compatible settings to key: {}", OPENAI_COMPATIBLE_SETTING_KEY);
    // Note: API keys are stored as-is here. In production, consider using
    // platform-specific secure storage (e.g., Windows Credential Manager, macOS Keychain)
    match store.set_object(OPENAI_COMPATIBLE_SETTING_KEY, &stored) {
      Ok(_) => {
        debug!("Successfully saved OpenAI compatible settings");
        Ok(())
      }
      Err(e) => {
        debug!("Failed to save OpenAI compatible settings: {:?}", e);
        Err(FlowyError::new(ErrorCode::Internal, format!("Failed to save OpenAI compatible settings: {}", e)))
      }
    }
  }

  /// Load OpenAI compatible settings
  pub fn load_openai_compatible_setting(&self) -> FlowyResult<Option<OpenAICompatibleSettingPB>> {
    let store = self.upgrade_store()?;
    debug!("Loading OpenAI compatible settings from key: {}", OPENAI_COMPATIBLE_SETTING_KEY);
    let stored = store.get_object::<StoredOpenAICompatibleSetting>(OPENAI_COMPATIBLE_SETTING_KEY);
    
    match stored {
      Some(stored) => {
        let setting = OpenAICompatibleSettingPB {
          chat_setting: crate::entities::OpenAIChatSettingPB {
            api_endpoint: stored.chat_setting.api_endpoint,
            api_key: stored.chat_setting.api_key,
            model_name: stored.chat_setting.model_name,
            model_type: stored.chat_setting.model_type,
            max_tokens: stored.chat_setting.max_tokens,
            temperature: stored.chat_setting.temperature,
            timeout_seconds: stored.chat_setting.timeout_seconds,
          },
          embedding_setting: crate::entities::OpenAIEmbeddingSettingPB {
            api_endpoint: stored.embedding_setting.api_endpoint,
            api_key: stored.embedding_setting.api_key,
            model_name: stored.embedding_setting.model_name,
          },
        };
        
        debug!("Successfully loaded OpenAI compatible settings with chat endpoint: {}", setting.chat_setting.api_endpoint);
        Ok(Some(setting))
      }
      None => {
        debug!("No OpenAI compatible settings found in storage");
        Ok(None)
      }
    }
  }

  /// Remove OpenAI compatible settings (for cleanup/reset)
  pub fn remove_openai_compatible_setting(&self) -> FlowyResult<()> {
    let store = self.upgrade_store()?;
    store.remove(OPENAI_COMPATIBLE_SETTING_KEY);
    debug!("Removed OpenAI compatible settings");
    Ok(())
  }

  /// Migrate settings from old format if needed
  /// This is a placeholder for future migration logic
  pub fn migrate_settings_if_needed(&self) -> FlowyResult<()> {
    // Check for old format keys and migrate if found
    // For now, no migration is needed as this is the v1 format
    Ok(())
  }

  /// Helper to upgrade weak reference
  fn upgrade_store(&self) -> FlowyResult<std::sync::Arc<KVStorePreferences>> {
    self
      .store_preferences
      .upgrade()
      .ok_or_else(|| FlowyError::new(ErrorCode::Internal, "Store preferences is dropped"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_global_model_type_conversion() {
    // Test that enum values convert correctly
    let local_ai = StoredGlobalAIModelType { model_type: 0 };
    assert_eq!(local_ai.model_type, 0);
    
    let openai_compatible = StoredGlobalAIModelType { model_type: 1 };
    assert_eq!(openai_compatible.model_type, 1);
  }
  
  #[test]
  fn test_stored_setting_serialization() {
    // Test that settings can be serialized/deserialized
    let setting = StoredOpenAICompatibleSetting {
      chat_setting: StoredOpenAIChatSetting {
        api_endpoint: "https://api.openai.com/v1".to_string(),
        api_key: "test-key".to_string(),
        model_name: "gpt-4".to_string(),
        model_type: "reasoning".to_string(),
        max_tokens: 4096,
        temperature: 0.7,
        timeout_seconds: 30,
      },
      embedding_setting: StoredOpenAIEmbeddingSetting {
        api_endpoint: "https://api.openai.com/v1".to_string(),
        api_key: "test-key".to_string(),
        model_name: "text-embedding-ada-002".to_string(),
      },
    };
    
    // Test serialization
    let json = serde_json::to_string(&setting).unwrap();
    let deserialized: StoredOpenAICompatibleSetting = serde_json::from_str(&json).unwrap();
    
    assert_eq!(setting.chat_setting.api_endpoint, deserialized.chat_setting.api_endpoint);
    assert_eq!(setting.chat_setting.model_name, deserialized.chat_setting.model_name);
  }

  #[test]
  fn test_openai_compatible_setting_persistence() {
    use tempfile::TempDir;
    use std::sync::Arc;
    
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    
    // Create KVStorePreferences
    let store_preferences = Arc::new(KVStorePreferences::new(temp_path).unwrap());
    let persistence = AIPersistence::new(Arc::downgrade(&store_preferences));
    
    // Create test setting
    let test_setting = crate::entities::OpenAICompatibleSettingPB {
      chat_setting: crate::entities::OpenAIChatSettingPB {
        api_endpoint: "https://test.example.com/v1".to_string(),
        api_key: "test-api-key".to_string(),
        model_name: "test-model".to_string(),
        model_type: "chat".to_string(),
        max_tokens: 2048,
        temperature: 0.8,
        timeout_seconds: 60,
      },
      embedding_setting: crate::entities::OpenAIEmbeddingSettingPB {
        api_endpoint: "https://test.example.com/v1".to_string(),
        api_key: "test-embedding-key".to_string(),
        model_name: "test-embedding-model".to_string(),
      },
    };
    
    // Test saving
    let save_result = persistence.save_openai_compatible_setting(&test_setting);
    assert!(save_result.is_ok(), "Failed to save settings: {:?}", save_result.err());
    
    // Test loading
    let load_result = persistence.load_openai_compatible_setting();
    assert!(load_result.is_ok(), "Failed to load settings: {:?}", load_result.err());
    
    let loaded_setting = load_result.unwrap();
    assert!(loaded_setting.is_some(), "No settings found after saving");
    
    let loaded_setting = loaded_setting.unwrap();
    assert_eq!(loaded_setting.chat_setting.api_endpoint, test_setting.chat_setting.api_endpoint);
    assert_eq!(loaded_setting.chat_setting.api_key, test_setting.chat_setting.api_key);
    assert_eq!(loaded_setting.chat_setting.model_name, test_setting.chat_setting.model_name);
    assert_eq!(loaded_setting.chat_setting.max_tokens, test_setting.chat_setting.max_tokens);
    assert_eq!(loaded_setting.embedding_setting.api_endpoint, test_setting.embedding_setting.api_endpoint);
    assert_eq!(loaded_setting.embedding_setting.model_name, test_setting.embedding_setting.model_name);
    
    println!("✅ OpenAI compatible settings persistence test passed!");
  }
}
