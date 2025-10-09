# 任务文档

- [x] 1. 创建网络搜索数据模型和实体 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/entities.rs
  - 定义搜索供应商、结果和配置的 Rust 结构体
  - 扩展现有的 AI 实体模式
  - 目的: 为网络搜索实现建立类型安全
  - _Leverage: rust-lib/flowy-ai/src/entities.rs, rust-lib/flowy-ai/src/mcp/entities.rs
  - _Requirements: 1.1, 1.2, 1.3
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事数据建模和类型系统的 Rust 开发者 | 任务: 根据需求 1.1、1.2 和 1.3 创建网络搜索功能的综合 Rust 数据结构，扩展现有实体模式 | 限制: 不要修改现有实体结构，保持向后兼容性，遵循 AppFlowy 命名约定 | 成功: 所有结构体编译无错误，支持适当的序列化，完全覆盖网络搜索需求

- [x] 2. 创建网络搜索供应商管理器 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/provider_manager.rs
  - 实现供应商注册和管理逻辑
  - 添加供应商激活和停用功能
  - 目的: 管理不同的搜索引擎供应商
  - _Leverage: rust-lib/flowy-ai/src/mcp/config.rs, rust-lib/flowy-ai/src/agent/config_manager.rs
  - _Requirements: 1.1, 1.2
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 Rust 和服务管理的后端开发者 | 任务: 根据需求 1.1 和 1.2 创建网络搜索引擎的供应商管理器，利用现有的 MCP 和智能体配置管理器模式 | 限制: 必须遵循现有配置模式，不要绕过验证工具，保持一致的错误处理 | 成功: 供应商管理器正确处理注册，激活逻辑正常工作，遵循 AppFlowy 架构模式

- [x] 3. 实现 Tavily 搜索供应商 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/providers/tavily.rs
  - 使用 MCP 客户端创建 Tavily API 集成
  - 添加 API 密钥验证和连接测试
  - 目的: 集成 Tavily 搜索引擎
  - _Leverage: rust-lib/flowy-ai/src/mcp/client.rs, rust-lib/flowy-ai/src/mcp/manager.rs
  - _Requirements: 1.2, 2.1
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 API 集成和 MCP 协议的 Rust 开发者 | 任务: 根据需求 1.2 和 2.1 实现 Tavily 搜索供应商，使用现有的 MCP 客户端基础设施和管理器模式 | 限制: 必须使用现有的 MCP 客户端模式，不要重复连接逻辑，确保适当的错误处理 | 成功: Tavily 集成正常工作，API 密钥验证功能正常，连接测试可靠

- [x] 4. 实现 Brave Search 供应商 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/providers/brave.rs
  - 使用 MCP 客户端创建 Brave Search API 集成
  - 添加 API 密钥验证和连接测试
  - 目的: 集成 Brave Search 引擎
  - _Leverage: rust-lib/flowy-ai/src/mcp/client.rs, rust-lib/flowy-ai/src/mcp/manager.rs
  - _Requirements: 1.2, 2.2
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 API 集成和 MCP 协议的 Rust 开发者 | 任务: 根据需求 1.2 和 2.2 实现 Brave Search 供应商，使用现有的 MCP 客户端基础设施和管理器模式 | 限制: 必须使用现有的 MCP 客户端模式，不要重复连接逻辑，确保适当的错误处理 | 成功: Brave Search 集成正常工作，API 密钥验证功能正常，连接测试可靠

- [x] 5. 创建搜索结果处理器 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/result_processor.rs
  - 实现不同供应商的统一结果格式化
  - 添加引用提取和格式化
  - 目的: 标准化不同供应商的搜索结果
  - _Leverage: rust-lib/flowy-ai/src/web_search/entities.rs
  - _Requirements: 1.1, 1.3
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事数据处理和转换的 Rust 开发者 | 任务: 根据需求 1.1 和 1.3 创建搜索结果处理器，为不同搜索供应商实现统一格式化 | 限制: 必须处理所有供应商结果格式，不要在转换过程中丢失数据，保持结果质量 | 成功: 结果处理器正确处理所有供应商格式，引用被适当提取，统一格式一致

- [x] 6. 实现搜索缓存管理器 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/cache_manager.rs
  - 为搜索结果创建缓存系统
  - 添加缓存失效和清理
  - 目的: 提高搜索性能并减少 API 调用
  - _Leverage: rust-lib/flowy-ai/src/mcp/config.rs, flowy-sqlite crate
  - _Requirements: 1.1, 1.4
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事缓存系统和性能优化的 Rust 开发者 | 任务: 根据需求 1.1 和 1.4 实现搜索缓存管理器，使用现有的存储模式和 SQLite 集成 | 限制: 必须使用现有的存储模式，不要绕过缓存失效，确保线程安全 | 成功: 缓存管理器高效存储和检索结果，失效正常工作，性能得到改善

- [x] 7. 创建网络搜索中心服务 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/hub.rs
  - 实现统一的搜索接口
  - 添加供应商选择和结果聚合
  - 目的: 为网络搜索功能提供单一入口点
  - _Leverage: rust-lib/flowy-ai/src/web_search/provider_manager.rs, rust-lib/flowy-ai/src/web_search/result_processor.rs
  - _Requirements: 1.1, 1.2, 1.3
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事服务架构和 API 设计的高级 Rust 开发者 | 任务: 根据需求 1.1、1.2 和 1.3 创建网络搜索中心服务，集成供应商管理器和结果处理器 | 限制: 必须保持单一职责原则，不要重复供应商逻辑，确保适当的错误传播 | 成功: 中心服务正确提供统一接口，供应商选择正常工作，结果聚合高效

- [x] 8. 为智能体创建网络搜索工具 ✅
  - 文件: rust-lib/flowy-ai/src/web_search/tool.rs
  - 为智能体工具注册表实现网络搜索工具
  - 添加工具模式和执行逻辑
  - 目的: 使智能体能够使用网络搜索功能
  - _Leverage: rust-lib/flowy-ai/src/agent/tool_registry.rs, rust-lib/flowy-ai/src/web_search/hub.rs
  - _Requirements: 1.5, 2.1
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事工具系统和智能体集成的 Rust 开发者 | 任务: 根据需求 1.5 和 2.1 为智能体创建网络搜索工具，与现有的工具注册表和搜索中心集成 | 限制: 必须遵循现有的工具模式，不要绕过工具注册表，确保适当的模式定义 | 成功: 网络搜索工具与智能体系统正确集成，工具模式正确定义，执行逻辑可靠工作

- [x] 9. 扩展 AI 实体以支持网络搜索
  - 文件: rust-lib/flowy-ai/src/entities.rs (修改现有)
  - 添加网络搜索相关的 protobuf 定义
  - 扩展现有的 AI 事件系统
  - 目的: 启用网络搜索的前后端通信
  - _Leverage: rust-lib/flowy-ai/src/entities.rs 中的现有 AI 实体
  - _Requirements: 1.4, 1.6
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 protobuf 和事件系统的 Rust 开发者 | 任务: 根据需求 1.4 和 1.6 扩展 AI 实体以支持网络搜索功能，添加 protobuf 定义和事件系统集成 | 限制: 必须保持现有实体兼容性，不要破坏现有事件，遵循 protobuf 命名约定 | 成功: 新实体与现有系统无缝集成，protobuf 定义正确，事件系统支持网络搜索操作

- [x] 10. 创建网络搜索配置 UI 组件 ✅
  - 文件: appflowy_flutter/lib/plugins/ai_chat/widgets/web_search_settings.dart
  - 实现供应商配置界面
  - 添加 API 密钥输入和测试功能
  - 目的: 允许用户配置搜索供应商
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/widgets/, 现有设置模式
  - _Requirements: 1.4, 1.6
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 UI 组件和设置界面的 Flutter 开发者 | 任务: 根据需求 1.4 和 1.6 创建网络搜索配置 UI，使用现有的小部件模式和设置组件 | 限制: 必须遵循现有的 UI 模式，不要绕过验证，确保响应式设计 | 成功: 配置 UI 直观且功能正常，API 密钥测试正常工作，设置被正确保存

- [x] 11. 将网络搜索设置添加到全局设置
  - 文件: appflowy_flutter/lib/plugins/ai_chat/bloc/settings_ai_bloc.dart (修改现有)
  - 扩展设置 bloc 以处理网络搜索配置
  - 添加供应商管理和测试
  - 目的: 将网络搜索设置集成到现有设置系统中
  - _Leverage: 现有设置 bloc 模式, KVStorePreferences
  - _Requirements: 1.4, 1.6
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 BLoC 模式和状态管理的 Flutter 开发者 | 任务: 根据需求 1.4 和 1.6 扩展设置 bloc 以支持网络搜索配置，与现有设置模式集成 | 限制: 必须遵循现有的 BLoC 模式，不要绕过状态管理，确保适当的错误处理 | 成功: 设置 bloc 正确处理网络搜索配置，状态管理一致，错误处理健壮

- [ ] 12. 创建信息源选择器 UI 组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/widgets/source_selector.dart
  - 实现信息源选择下拉菜单
  - 添加网络搜索选项和状态管理
  - 目的: 允许用户在聊天中选择信息源
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/widgets/, 现有下拉菜单模式
  - _Requirements: 1.7, 2.3
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 UI 组件和用户交互的 Flutter 开发者 | 任务: 根据需求 1.7 和 2.3 创建信息源选择器组件，实现信息源选择的下拉菜单 | 限制: 必须遵循现有的 UI 模式，不要绕过状态管理，确保可访问性 | 成功: 信息源选择器正常工作，网络搜索选项被适当集成，用户交互流畅

- [ ] 13. 扩展聊天 UI 以支持网络搜索集成
  - 文件: appflowy_flutter/lib/plugins/ai_chat/widgets/chat_input.dart (修改现有)
  - 将信息源选择器添加到聊天输入
  - 将网络搜索选项与聊天流程集成
  - 目的: 在聊天界面中启用网络搜索
  - _Leverage: 现有聊天输入组件, 信息源选择器
  - _Requirements: 1.7, 2.3
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事聊天界面和用户体验的 Flutter 开发者 | 任务: 根据需求 1.7 和 2.3 扩展聊天输入以支持网络搜索集成，将信息源选择器与现有聊天流程集成 | 限制: 必须保持现有聊天功能，不要破坏用户体验，确保无缝集成 | 成功: 聊天输入支持网络搜索选项，集成无缝，用户体验得到增强

- [ ] 14. 添加引用显示组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/widgets/citation_display.dart
  - 实现搜索结果引用列表显示
  - 添加可点击链接和适当格式化
  - 目的: 在聊天响应中显示搜索结果引用
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/widgets/, 现有链接组件
  - _Requirements: 1.7, 2.3
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事内容显示和链接处理的 Flutter 开发者 | 任务: 根据需求 1.7 和 2.3 创建引用显示组件，实现适当的格式化和可点击链接 | 限制: 必须正确处理链接点击，不要绕过安全性，确保适当的格式化 | 成功: 引用被正确显示，链接可点击且安全，格式化一致

- [ ] 15. 添加多语言支持
  - 文件: frontend/resources/translations/zh.json (修改现有)
  - 文件: frontend/resources/translations/en.json (修改现有)
  - 添加网络搜索相关翻译
  - 目的: 支持中文和英文语言
  - _Leverage: 现有翻译文件和 EasyLocalization
  - _Requirements: 1.8
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事多语言支持的本地化专家 | 任务: 根据需求 1.8 添加网络搜索翻译，扩展现有翻译文件以支持中文和英文 | 限制: 必须遵循现有翻译模式，不要重复键，确保一致性 | 成功: 所有网络搜索 UI 元素都被适当翻译，语言切换正常工作，翻译准确

- [ ] 16. 为网络搜索组件创建单元测试
  - 文件: rust-lib/flowy-ai/src/web_search/tests/
  - 为所有网络搜索组件编写综合单元测试
  - 测试供应商集成、结果处理和错误处理
  - 目的: 确保可靠性并捕获回归
  - _Leverage: rust-lib/flowy-ai/src/ 中的现有测试模式
  - _Requirements: 全部
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 Rust 单元测试和模拟的 QA 工程师 | 任务: 为网络搜索组件创建综合单元测试，覆盖所有需求，使用现有测试模式和模拟策略 | 限制: 必须测试成功和失败场景，不要直接测试外部依赖，保持测试隔离 | 成功: 所有网络搜索组件都被测试，覆盖率高，边缘情况被覆盖，测试独立且一致运行

- [ ] 17. 创建集成测试
  - 文件: rust-lib/flowy-ai/src/web_search/tests/integration_tests.rs
  - 为网络搜索中心和供应商交互编写集成测试
  - 测试端到端搜索功能
  - 目的: 确保组件正确协同工作
  - _Leverage: 现有集成测试模式
  - _Requirements: 全部
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事 Rust 集成测试的集成工程师 | 任务: 为网络搜索功能创建集成测试，覆盖所有需求，测试组件交互和端到端流程 | 限制: 必须测试真实的组件交互，不要模拟内部组件，确保测试可靠性 | 成功: 集成测试覆盖所有关键工作流程，组件正确协同工作，端到端功能得到验证

- [ ] 18. 最终集成和文档
  - 文件: rust-lib/flowy-ai/src/web_search/README.md
  - 集成所有网络搜索组件
  - 创建综合文档
  - 目的: 完成实现并提供使用指导
  - _Leverage: 现有文档模式
  - _Requirements: 全部
  - _Prompt: 为规范 web-search-tools 实现任务，首先运行 spec-workflow-guide 获取工作流程指南，然后实现任务: 角色: 专门从事系统集成和文档的高级开发者 | 任务: 完成网络搜索组件的最终集成并创建综合文档，覆盖所有需求 | 限制: 不要破坏现有功能，确保代码质量标准得到满足，保持文档一致性 | 成功: 所有组件完全集成并协同工作，文档综合且准确，系统满足所有需求和质量标准