# AppFlowy OpenAI SDK集成设计规范

## 1. 设计概述

### 1.1 架构目标
将AppFlowy的OpenAI兼容服务器实现从自定义HTTP客户端迁移到OpenAI官方Dart SDK，同时保持现有功能和界面的兼容性。

### 1.2 设计原则
- **最小化变更**: 保留现有的配置界面和数据结构
- **向后兼容**: 确保现有配置可以无缝迁移
- **模块化设计**: SDK集成作为独立模块，不影响其他功能
- **可测试性**: 每个组件都可以独立测试

## 2. 系统架构

### 2.1 整体架构图
```
┌─────────────────────────────────────────────────────────────┐
│                    AppFlowy Frontend                        │
├─────────────────────────────────────────────────────────────┤
│  Settings UI Layer                                          │
│  ├─ AI Settings Page (Desktop/Mobile/Server)                │
│  ├─ Global Model Type Selector                              │
│  ├─ OpenAI Compatible Settings Panel                        │
│  └─ Test Buttons & Results Display                          │
├─────────────────────────────────────────────────────────────┤
│  BLoC Layer                                                 │
│  ├─ SettingsAIBloc (Global Model Type)                      │
│  ├─ OpenAICompatibleSettingBloc (Configuration)             │
│  └─ OpenAISDKBloc (SDK Operations)                          │
├─────────────────────────────────────────────────────────────┤
│  Service Layer                                              │
│  ├─ OpenAISDKService (New)                                  │
│  ├─ OpenAICompatibleService (Modified)                      │
│  └─ AIConfigurationService                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Rust Backend                             │
├─────────────────────────────────────────────────────────────┤
│  Event Handlers                                             │
│  ├─ AI Event Handlers (Modified)                            │
│  └─ OpenAI SDK Event Handlers (New)                         │
├─────────────────────────────────────────────────────────────┤
│  Controllers                                                │
│  ├─ AIController (Modified)                                 │
│  ├─ OpenAISDKController (New)                               │
│  └─ GlobalModelTypeController (New)                         │
├─────────────────────────────────────────────────────────────┤
│  Services                                                   │
│  ├─ OpenAISDKChatService (New)                              │
│  ├─ OpenAISDKEmbeddingService (New)                         │
│  └─ ConfigurationPersistenceService                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 OpenAI Dart SDK                             │
│  ├─ Chat Completions API                                    │
│  ├─ Embeddings API                                          │
│  └─ Configuration & Authentication                          │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 数据流设计
```
User Configuration → BLoC → Rust Backend → OpenAI SDK → API
                                    ↓
                            Persistent Storage
```

## 3. 组件设计

### 3.1 Frontend Components

#### 3.1.1 全局模型类型选择器
**文件**: `lib/workspace/presentation/settings/pages/setting_ai_view/global_model_type_selector.dart`
```dart
class GlobalModelTypeSelector extends StatelessWidget {
  // 下拉框选择全局模型类型
  // 选项：GlobalLocalAI, GlobalOpenAICompatible
  // 选择后触发导航到详细配置页面
}
```

#### 3.1.2 OpenAI SDK配置面板
**文件**: `lib/workspace/presentation/settings/pages/setting_ai_view/openai_sdk_setting.dart`
```dart
class OpenAISDKSetting extends StatelessWidget {
  // 聊天模型配置区域
  // 嵌入模型配置区域  
  // 测试按钮和结果显示
  // 保存/重置操作
}
```

#### 3.1.3 BLoC状态管理
**文件**: `lib/workspace/application/settings/ai/openai_sdk_bloc.dart`
```dart
class OpenAISDKBloc extends Bloc<OpenAISDKEvent, OpenAISDKState> {
  // 处理配置加载、保存、测试等事件
  // 管理UI状态和错误处理
}
```

### 3.2 Backend Components

#### 3.2.1 全局模型类型控制器
**文件**: `rust-lib/flowy-ai/src/global_model/controller.rs`
```rust
pub struct GlobalModelTypeController {
    // 管理全局模型类型选择
    // 协调不同AI服务的切换
}
```

#### 3.2.2 OpenAI SDK控制器
**文件**: `rust-lib/flowy-ai/src/openai_sdk/controller.rs`
```rust
pub struct OpenAISDKController {
    // 管理OpenAI SDK的生命周期
    // 处理配置变更和服务重启
}
```

#### 3.2.3 OpenAI SDK聊天服务
**文件**: `rust-lib/flowy-ai/src/openai_sdk/chat_service.rs`
```rust
pub struct OpenAISDKChatService {
    // 使用OpenAI Dart SDK实现聊天功能
    // 支持流式响应和错误处理
}
```

#### 3.2.4 OpenAI SDK嵌入服务
**文件**: `rust-lib/flowy-ai/src/openai_sdk/embedding_service.rs`
```rust
pub struct OpenAISDKEmbeddingService {
    // 使用OpenAI Dart SDK实现嵌入功能
    // 批量处理和缓存优化
}
```

## 4. 数据模型设计

### 4.1 配置数据结构

#### 4.1.1 全局模型类型配置
```rust
#[derive(Clone, Debug, ProtoBuf_Enum, Default)]
pub enum GlobalAIModelTypePB {
    #[default]
    GlobalLocalAI = 0,
    GlobalOpenAICompatible = 1,
}
```

#### 4.1.2 OpenAI SDK配置
```rust
#[derive(Default, ProtoBuf, Clone, Debug, Validate)]
pub struct OpenAISDKSettingPB {
    #[pb(index = 1)]
    pub chat_setting: OpenAISDKChatSettingPB,
    
    #[pb(index = 2)]
    pub embedding_setting: OpenAISDKEmbeddingSettingPB,
}

#[derive(Default, ProtoBuf, Clone, Debug, Validate)]
pub struct OpenAISDKChatSettingPB {
    #[pb(index = 1)]
    pub api_endpoint: String,
    
    #[pb(index = 2)]
    pub api_key: String,
    
    #[pb(index = 3)]
    pub model_name: String,
    
    #[pb(index = 4)]
    pub model_type: String,
    
    #[pb(index = 5)]
    pub max_tokens: i32,
    
    #[pb(index = 6)]
    pub temperature: f32,
    
    #[pb(index = 7)]
    pub timeout_seconds: i32,
}

#[derive(Default, ProtoBuf, Clone, Debug, Validate)]
pub struct OpenAISDKEmbeddingSettingPB {
    #[pb(index = 1)]
    pub api_endpoint: String,
    
    #[pb(index = 2)]
    pub api_key: String,
    
    #[pb(index = 3)]
    pub model_name: String,
}
```

### 4.2 测试结果数据结构
```rust
#[derive(Default, ProtoBuf, Clone, Debug)]
pub struct OpenAISDKTestResultPB {
    #[pb(index = 1)]
    pub success: bool,
    
    #[pb(index = 2)]
    pub error_message: String,
    
    #[pb(index = 3)]
    pub response_time_ms: String,
    
    #[pb(index = 4)]
    pub status_code: i32,
    
    #[pb(index = 5)]
    pub request_details: String,
    
    #[pb(index = 6)]
    pub server_response: String,
}
```

## 5. API设计

### 5.1 Rust Backend Events

#### 5.1.1 全局模型类型事件
```rust
// 获取全局模型类型
AIEventGetGlobalAIModelType() -> GlobalAIModelTypeSettingPB

// 保存全局模型类型
AIEventSaveGlobalAIModelType(GlobalAIModelTypeSettingPB) -> ()
```

#### 5.1.2 OpenAI SDK配置事件
```rust
// 获取OpenAI SDK配置
AIEventGetOpenAISDKSetting() -> OpenAISDKSettingPB

// 保存OpenAI SDK配置
AIEventSaveOpenAISDKSetting(OpenAISDKSettingPB) -> ()

// 测试聊天模型
AIEventTestOpenAISDKChat(OpenAISDKChatSettingPB) -> OpenAISDKTestResultPB

// 测试嵌入模型
AIEventTestOpenAISDKEmbedding(OpenAISDKEmbeddingSettingPB) -> OpenAISDKTestResultPB
```

### 5.2 Flutter Service APIs

#### 5.2.1 OpenAI SDK服务接口
```dart
abstract class OpenAISDKService {
  Future<FlowyResult<OpenAISDKSettingPB, FlowyError>> getSettings();
  Future<FlowyResult<void, FlowyError>> saveSettings(OpenAISDKSettingPB settings);
  Future<FlowyResult<OpenAISDKTestResultPB, FlowyError>> testChat(OpenAISDKChatSettingPB chatSettings);
  Future<FlowyResult<OpenAISDKTestResultPB, FlowyError>> testEmbedding(OpenAISDKEmbeddingSettingPB embeddingSettings);
}
```

## 6. 依赖管理

### 6.1 新增依赖

#### 6.1.1 Dart依赖 (pubspec.yaml)
```yaml
dependencies:
  openai_dart: ^3.0.0  # OpenAI官方Dart SDK
```

#### 6.1.2 Rust依赖 (Cargo.toml)
```toml
[dependencies]
# 如果需要Rust端的OpenAI客户端
openai-api-rs = "4.0.0"  # 可选，主要使用Dart SDK
```

### 6.2 依赖集成策略
- 主要使用OpenAI Dart SDK在Flutter层实现
- Rust后端主要负责配置管理和协调
- 保持现有依赖的兼容性

## 7. 配置迁移设计

### 7.1 数据迁移策略
```rust
pub struct ConfigurationMigrator {
    // 从现有OpenAI兼容配置迁移到新的SDK配置
    // 保持字段映射和默认值处理
}

impl ConfigurationMigrator {
    pub fn migrate_openai_compatible_to_sdk(
        old_config: OpenAICompatibleSettingPB
    ) -> OpenAISDKSettingPB {
        // 实现配置数据的平滑迁移
    }
}
```

### 7.2 向后兼容性
- 保留现有的protobuf消息定义
- 新增字段使用optional标记
- 提供默认值和迁移逻辑

## 8. 错误处理设计

### 8.1 错误类型定义
```rust
#[derive(Debug, Clone)]
pub enum OpenAISDKError {
    ConfigurationError(String),
    NetworkError(String),
    AuthenticationError(String),
    ModelNotFoundError(String),
    RateLimitError(String),
    TimeoutError(String),
    UnknownError(String),
}
```

### 8.2 错误处理策略
- 网络错误：自动重试机制
- 配置错误：用户友好的错误提示
- 认证错误：引导用户检查API密钥
- 超时错误：建议调整超时设置

## 9. 测试策略

### 9.1 单元测试
- 每个服务组件的独立测试
- 配置验证逻辑测试
- 错误处理路径测试

### 9.2 集成测试
- OpenAI SDK集成测试
- 端到端配置流程测试
- 多平台兼容性测试

### 9.3 UI测试
- 配置界面交互测试
- 测试功能验证
- 错误状态显示测试

## 10. 性能考虑

### 10.1 优化策略
- 配置缓存：避免重复读取配置
- 连接池：复用HTTP连接
- 异步处理：非阻塞的API调用
- 错误缓存：避免重复的失败请求

### 10.2 监控指标
- API响应时间
- 错误率统计
- 配置变更频率
- 用户操作延迟

## 11. 安全设计

### 11.1 API密钥管理
- 安全存储：使用系统密钥链
- 传输加密：HTTPS通信
- 日志脱敏：不记录敏感信息

### 11.2 网络安全
- 证书验证：验证SSL证书
- 超时控制：防止长时间阻塞
- 请求限制：防止滥用

## 12. 国际化设计

### 12.1 多语言支持
- 中文：简体中文界面和错误信息
- 英文：英文界面和错误信息
- 扩展性：支持添加更多语言

### 12.2 本地化策略
- 使用AppFlowy现有的国际化框架
- 错误信息本地化
- 帮助文档本地化
