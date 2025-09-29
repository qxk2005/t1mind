# AppFlowy MCP支持与AI助手智能体 — 任务清单

说明：每个任务均需有可视化产出或可验证的接口行为；支持 Windows/macOS/Android；中英双语。完成一个任务后请在本文件中将其状态从 [ ] 改为 [x]。

## 阶段1：MCP基础设施建设

### M1.1：MCP客户端基础架构
- [x] 1. 扩展MCP管理器核心功能
  - 文件: rust-lib/flowy-ai/src/mcp/manager.rs
  - 实现多传输方式支持（STDIO、SSE、HTTP）
  - 添加连接池管理和工具发现机制
  - 目的: 建立MCP连接和工具调用的基础设施
  - _Leverage: 现有的MCPClient基础代码_
  - _Requirements: 需求5_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust系统工程师，专精网络通信和协议实现 | Task: 扩展MCP管理器以支持STDIO、SSE、HTTP三种传输方式，实现连接池管理和工具发现机制，遵循需求5的规范 | Restrictions: 不能破坏现有的MCP模块结构，必须保持向后兼容性，确保线程安全 | _Leverage: rust-lib/flowy-ai/src/mcp/manager.rs, 现有的MCPClient基础代码 | _Requirements: 需求5 | Success: 三种传输方式都能正常连接，工具发现机制工作正常，连接池管理有效 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [x] 2. 实现MCP客户端池
  - 文件: rust-lib/flowy-ai/src/mcp/client_pool.rs
  - 创建不同传输方式的客户端实现
  - 添加客户端生命周期管理
  - 目的: 管理多个MCP服务器连接
  - _Leverage: 现有的网络通信模块_
  - _Requirements: 需求5_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust并发编程专家，专精连接池和资源管理 | Task: 实现MCP客户端池，支持STDIO、SSE、HTTP客户端的创建和管理，包含生命周期管理，遵循需求5的规范 | Restrictions: 必须确保线程安全，避免连接泄漏，实现优雅的连接关闭 | _Leverage: rust-lib/flowy-ai/src/mcp/manager.rs, 现有网络通信模块 | _Requirements: 需求5 | Success: 客户端池能正确管理多个连接，支持连接复用，资源清理完整 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [x] 3. 添加MCP配置存储
  - 文件: rust-lib/flowy-ai/src/mcp/config.rs
  - 实现MCP服务器配置的数据库存储
  - 添加配置验证和序列化
  - 目的: 持久化MCP配置信息
  - _Leverage: 现有的KVStorePreferences和SQLite_
  - _Requirements: 需求5_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust数据库专家，专精SQLite和数据持久化 | Task: 实现MCP配置的数据库存储，包括配置验证、序列化和反序列化，遵循需求5的规范 | Restrictions: 必须使用现有的数据库架构，确保数据一致性，支持配置迁移 | _Leverage: 现有的KVStorePreferences, SQLite数据库模块 | _Requirements: 需求5 | Success: 配置能正确存储和读取，验证机制有效，支持配置更新 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M1.2：MCP事件系统集成
- [x] 4. 扩展AI事件映射
  - 文件: rust-lib/flowy-ai/src/event_map.rs
  - 添加MCP相关的事件定义和处理器
  - 集成到现有的AFPlugin系统
  - 目的: 将MCP功能集成到AppFlowy事件系统
  - _Leverage: 现有的事件映射模式_
  - _Requirements: 需求5_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust事件系统架构师，专精AFPlugin和事件驱动架构 | Task: 扩展AI事件映射以支持MCP功能，添加事件定义和处理器，集成到AFPlugin系统，遵循需求5的规范 | Restrictions: 必须遵循现有的事件命名约定，不能破坏现有事件处理，确保事件序列化正确 | _Leverage: rust-lib/flowy-ai/src/event_map.rs, 现有事件映射模式 | _Requirements: 需求5 | Success: MCP事件能正确注册和处理，与现有事件系统无冲突，事件序列化正常 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [x] 5. 实现MCP事件处理器
  - 文件: rust-lib/flowy-ai/src/mcp/event_handler.rs
  - 实现MCP配置、连接、工具调用的事件处理
  - 添加错误处理和状态管理
  - 目的: 处理来自Flutter的MCP相关请求
  - _Leverage: 现有的事件处理器模式_
  - _Requirements: 需求5, 需求6_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust异步编程专家，专精事件处理和错误管理 | Task: 实现MCP事件处理器，处理配置、连接、工具调用等操作，包含完整的错误处理，遵循需求5和需求6的规范 | Restrictions: 必须使用异步处理，确保错误信息清晰，维护事件处理的一致性 | _Leverage: 现有事件处理器模式, rust-lib/flowy-ai/src/event_handler.rs | _Requirements: 需求5, 需求6 | Success: 所有MCP事件都能正确处理，错误信息详细准确，异步处理性能良好 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

## 阶段2：智能体核心系统

### M2.1：智能体配置管理
- [x] 6. 创建智能体数据模型
  - 文件: rust-lib/flowy-ai/src/agent/entities.rs
  - 定义智能体配置、会话历史等数据结构
  - 实现序列化和验证
  - 目的: 建立智能体数据模型基础
  - _Leverage: 现有的protobuf定义模式_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust数据建模专家，专精结构体设计和序列化 | Task: 创建智能体相关的数据模型，包括配置、会话历史等结构，实现序列化和验证，遵循需求2的规范 | Restrictions: 必须遵循现有的数据模型约定，确保向前兼容性，支持数据迁移 | _Leverage: 现有protobuf定义模式, rust-lib/flowy-ai/src/entities.rs | _Requirements: 需求2 | Success: 数据模型完整准确，序列化正常，验证机制有效 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [x] 7. 实现智能体配置管理器
  - 文件: rust-lib/flowy-ai/src/agent/config_manager.rs
  - 实现智能体配置的CRUD操作
  - 添加配置验证和个性化设置
  - 目的: 管理智能体的配置信息
  - _Leverage: 现有的数据库访问模式_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust数据库操作专家，专精CRUD操作和数据验证 | Task: 实现智能体配置管理器，支持配置的创建、读取、更新、删除操作，包含验证和个性化设置，遵循需求2的规范 | Restrictions: 必须确保数据一致性，支持事务操作，实现配置版本控制 | _Leverage: 现有数据库访问模式, SQLite操作模块 | _Requirements: 需求2 | Success: 配置CRUD操作正常，验证机制完整，个性化设置生效 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M2.2：AI驱动的任务规划器
- [ ] 8. 实现AI驱动任务规划器
  - 文件: rust-lib/flowy-ai/src/agent/planner.rs
  - 基于全局AI模型实现任务分解和规划
  - 集成工具选择和个性化特性
  - 目的: 智能分析用户问题并制定解决方案
  - _Leverage: 现有的AI模型调用机制_
  - _Requirements: 需求7_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: AI系统架构师，专精大模型集成和任务规划 | Task: 实现AI驱动的任务规划器，基于全局AI模型进行任务分解，集成工具选择和个性化特性，遵循需求7的规范 | Restrictions: 必须复用现有AI调用接口，确保规划结果可执行，支持规划失败重试 | _Leverage: 现有AI模型调用机制, rust-lib/flowy-ai/src/ai_manager.rs | _Requirements: 需求7 | Success: 能根据用户问题生成合理的任务计划，工具选择准确，个性化特性体现 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 9. 实现AI驱动任务执行器
  - 文件: rust-lib/flowy-ai/src/agent/executor.rs
  - 基于AI模型的工具调用功能执行任务
  - 添加反思和调整机制
  - 目的: 智能执行任务并处理执行结果
  - _Leverage: 现有的AI工具调用基础设施_
  - _Requirements: 需求7_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: AI执行引擎专家，专精工具调用和智能反思 | Task: 实现AI驱动的任务执行器，基于AI模型进行工具调用，包含反思和调整机制，遵循需求7的规范 | Restrictions: 必须支持多种工具类型，确保执行安全性，实现智能重试机制 | _Leverage: 现有AI工具调用基础设施, MCP工具调用器 | _Requirements: 需求7 | Success: 能正确执行各种任务，工具调用成功，反思机制有效 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M2.3：工具注册和调用系统
- [ ] 10. 实现工具注册表
  - 文件: rust-lib/flowy-ai/src/agent/tool_registry.rs
  - 管理所有可用工具的元数据
  - 实现工具发现和权限管理
  - 目的: 统一管理智能体可用的工具
  - _Leverage: MCP工具发现机制_
  - _Requirements: 需求1, 需求7_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: 工具系统架构师，专精插件管理和权限控制 | Task: 实现工具注册表，管理MCP、原生、搜索等工具的元数据，包含发现和权限管理，遵循需求1和需求7的规范 | Restrictions: 必须支持动态工具注册，确保权限安全，实现工具版本管理 | _Leverage: MCP工具发现机制, 现有权限管理模块 | _Requirements: 需求1, 需求7 | Success: 工具注册和发现正常，权限控制有效，支持多种工具类型 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 11. 实现AppFlowy原生工具
  - 文件: rust-lib/flowy-ai/src/agent/native_tools.rs
  - 实现文档CRUD操作工具
  - 集成现有的文档管理API
  - 目的: 为智能体提供AppFlowy内置功能
  - _Leverage: 现有的文档管理API_
  - _Requirements: 需求7_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: AppFlowy核心开发者，专精文档系统和API集成 | Task: 实现AppFlowy原生工具，专注于文档CRUD操作，集成现有文档管理API，遵循需求7的规范 | Restrictions: 只实现文档相关功能，不涉及用户和工作区管理，确保API调用安全 | _Leverage: 现有文档管理API, rust-lib/flowy-document/ | _Requirements: 需求7 | Success: 文档CRUD工具正常工作，API集成无误，权限控制正确 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

## 阶段3：Flutter UI集成

### M3.1：MCP配置界面
- [ ] 12. 创建MCP配置管理页面
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/mcp_settings_page.dart
  - 实现MCP服务器列表和配置界面
  - 添加连接测试和状态显示
  - 目的: 为用户提供MCP配置管理界面
  - _Leverage: 现有的设置页面模式_
  - _Requirements: 需求5, 需求6_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI开发专家，专精设置界面和表单处理 | Task: 创建MCP配置管理页面，包含服务器列表、配置表单、连接测试功能，遵循需求5和需求6的规范 | Restrictions: 必须遵循AppFlowy UI设计规范，确保跨平台兼容性，支持中英双语 | _Leverage: 现有设置页面模式, appflowy_flutter/lib/workspace/presentation/settings/ | _Requirements: 需求5, 需求6 | Success: 配置界面功能完整，连接测试正常，用户体验良好 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 13. 实现MCP配置BLoC
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/mcp_settings_bloc.dart
  - 管理MCP配置的状态和业务逻辑
  - 集成Rust后端的MCP事件
  - 目的: 处理MCP配置的状态管理
  - _Leverage: 现有的BLoC模式_
  - _Requirements: 需求5, 需求6_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter状态管理专家，专精BLoC模式和事件处理 | Task: 实现MCP配置BLoC，管理配置状态和业务逻辑，集成Rust后端事件，遵循需求5和需求6的规范 | Restrictions: 必须遵循现有BLoC模式，确保状态管理一致性，处理异步操作 | _Leverage: 现有BLoC模式, appflowy_flutter/lib/workspace/application/settings/ | _Requirements: 需求5, 需求6 | Success: 状态管理正确，事件处理正常，异步操作稳定 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M3.2：智能体管理界面
- [ ] 14. 创建智能体配置页面
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/agent_settings_page.dart
  - 实现智能体创建、编辑、删除界面
  - 添加个性化配置和工具选择
  - 目的: 为用户提供智能体管理界面
  - _Leverage: 现有的设置页面模式_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI/UX专家，专精表单设计和用户交互 | Task: 创建智能体配置页面，包含创建、编辑、删除功能，支持个性化配置和工具选择，遵循需求2的规范 | Restrictions: 必须提供直观的用户界面，确保配置项清晰易懂，支持配置验证 | _Leverage: 现有设置页面模式, 表单组件库 | _Requirements: 需求2 | Success: 智能体管理界面完整，配置功能正常，用户体验优秀 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 15. 实现智能体管理BLoC
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/agent_settings_bloc.dart
  - 管理智能体配置的状态和操作
  - 集成后端的智能体管理事件
  - 目的: 处理智能体配置的状态管理
  - _Leverage: 现有的BLoC模式_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter状态管理架构师，专精复杂状态管理和数据流 | Task: 实现智能体管理BLoC，处理配置状态和操作，集成后端事件，遵循需求2的规范 | Restrictions: 必须确保状态一致性，支持配置验证，处理并发操作 | _Leverage: 现有BLoC模式, 事件处理机制 | _Requirements: 需求2 | Success: 状态管理稳定，配置操作正确，事件集成无误 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M3.3：智能体聊天界面增强
- [ ] 16. 扩展聊天界面支持智能体
  - 文件: appflowy_flutter/lib/plugins/ai_chat/chat_page.dart
  - 添加智能体选择和状态显示
  - 集成执行过程可视化
  - 目的: 在聊天界面中展示智能体功能
  - _Leverage: 现有的聊天界面_
  - _Requirements: 需求3, 需求7_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter聊天界面专家，专精实时UI更新和用户交互 | Task: 扩展现有聊天界面以支持智能体，添加选择器、状态显示、执行过程可视化，遵循需求3和需求7的规范 | Restrictions: 不能破坏现有聊天功能，确保界面响应性，支持实时更新 | _Leverage: 现有聊天界面, appflowy_flutter/lib/plugins/ai_chat/chat_page.dart | _Requirements: 需求3, 需求7 | Success: 智能体集成无缝，执行过程清晰可见，用户体验流畅 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 17. 实现执行日志查看器
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/execution_log_viewer.dart
  - 创建执行日志的展示界面
  - 添加日志过滤和搜索功能
  - 目的: 为用户提供执行过程的详细视图
  - _Leverage: 现有的日志展示组件_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter数据展示专家，专精列表组件和数据可视化 | Task: 实现执行日志查看器，展示智能体执行过程，包含过滤和搜索功能，遵循需求4的规范 | Restrictions: 必须处理大量日志数据，确保界面性能，支持实时更新 | _Leverage: 现有日志展示组件, 列表组件库 | _Requirements: 需求4 | Success: 日志展示清晰，过滤搜索有效，性能表现良好 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

## 阶段4：跨平台设置集成

### M4.1：设置界面集成
- [ ] 18. 集成到服务器端设置
  - 文件: appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart
  - 添加MCP和智能体配置选项
  - 确保与现有AI设置的协调
  - 目的: 在桌面端设置中提供完整功能
  - _Leverage: 现有的AI设置页面_
  - _Requirements: 需求8_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter设置界面专家，专精设置页面集成和布局设计 | Task: 将MCP和智能体配置集成到服务器端设置页面，确保与现有AI设置协调，遵循需求8的规范 | Restrictions: 不能破坏现有设置结构，确保设置项逻辑清晰，维持界面一致性 | _Leverage: 现有AI设置页面, appflowy_flutter/lib/workspace/presentation/settings/ | _Requirements: 需求8 | Success: 设置集成无缝，功能完整可用，界面布局合理 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 19. 适配移动端设置界面
  - 文件: appflowy_flutter/lib/mobile/presentation/setting/ai/ai_settings_group.dart
  - 适配MCP和智能体配置到移动端
  - 优化触摸操作和界面布局
  - 目的: 在移动端提供完整的配置功能
  - _Leverage: 现有的移动端设置模式_
  - _Requirements: 需求8_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter移动端UI专家，专精触摸界面和响应式设计 | Task: 将MCP和智能体配置适配到移动端设置，优化触摸操作和布局，遵循需求8的规范 | Restrictions: 必须适配小屏幕，确保触摸友好，保持功能完整性 | _Leverage: 现有移动端设置模式, appflowy_flutter/lib/mobile/presentation/setting/ | _Requirements: 需求8 | Success: 移动端设置功能完整，操作体验良好，界面适配正确 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 20. 实现工作空间级别设置
  - 文件: appflowy_flutter/lib/workspace/presentation/settings/workspace/workspace_settings_page.dart
  - 添加工作空间级别的MCP和智能体配置
  - 实现配置作用域管理
  - 目的: 支持工作空间级别的配置隔离
  - _Leverage: 现有的工作空间设置_
  - _Requirements: 需求8_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter企业级应用专家，专精多租户和权限管理 | Task: 实现工作空间级别的MCP和智能体配置，包含作用域管理，遵循需求8的规范 | Restrictions: 必须确保配置隔离，支持权限控制，维护数据安全 | _Leverage: 现有工作空间设置, 权限管理模块 | _Requirements: 需求8 | Success: 工作空间配置隔离正确，权限控制有效，功能完整可用 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

## 阶段5：多语言和测试

### M5.1：国际化支持
- [ ] 21. 添加中英文翻译
  - 文件: frontend/resources/translations/zh-CN.json, frontend/resources/translations/en-US.json
  - 为所有新增界面和消息添加翻译
  - 确保术语一致性和准确性
  - 目的: 支持中英双语界面
  - _Leverage: 现有的翻译文件结构_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: 国际化专家，专精多语言支持和本地化 | Task: 为MCP和智能体功能添加中英文翻译，确保术语一致性和准确性，遵循所有需求的规范 | Restrictions: 必须保持翻译质量，确保术语专业性，维护文件结构一致 | _Leverage: 现有翻译文件结构, frontend/resources/translations/ | _Requirements: 所有需求 | Success: 翻译完整准确，术语一致，界面显示正常 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 22. 更新本地化键值
  - 文件: appflowy_flutter/lib/generated/locale_keys.g.dart
  - 生成新的本地化键值定义
  - 确保代码中的引用正确
  - 目的: 完成本地化集成
  - _Leverage: 现有的本地化生成流程_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter本地化工程师，专精代码生成和键值管理 | Task: 更新本地化键值定义，确保代码引用正确，遵循所有需求的规范 | Restrictions: 必须使用现有生成工具，确保键值命名规范，避免重复定义 | _Leverage: 现有本地化生成流程, easy_localization工具 | _Requirements: 所有需求 | Success: 键值生成正确，代码引用无误，本地化功能正常 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M5.2：测试和质量保证
- [ ] 23. 编写单元测试
  - 文件: rust-lib/flowy-ai/tests/mcp_tests.rs, rust-lib/flowy-ai/tests/agent_tests.rs
  - 为MCP和智能体核心功能编写单元测试
  - 确保代码覆盖率和质量
  - 目的: 保证代码质量和功能稳定性
  - _Leverage: 现有的测试框架_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust测试工程师，专精单元测试和代码质量 | Task: 为MCP和智能体功能编写全面的单元测试，确保代码覆盖率，遵循所有需求的规范 | Restrictions: 必须测试核心功能，包含边界情况，确保测试稳定性 | _Leverage: 现有测试框架, rust测试工具 | _Requirements: 所有需求 | Success: 测试覆盖率达标，所有测试通过，代码质量良好 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 24. 编写集成测试
  - 文件: appflowy_flutter/integration_test/mcp_agent_test.dart
  - 编写端到端的集成测试
  - 测试完整的用户工作流
  - 目的: 验证整体功能的正确性
  - _Leverage: 现有的集成测试框架_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter集成测试专家，专精端到端测试和用户场景 | Task: 编写MCP和智能体的集成测试，覆盖完整用户工作流，遵循所有需求的规范 | Restrictions: 必须测试真实用户场景，确保测试可靠性，支持自动化运行 | _Leverage: 现有集成测试框架, Flutter测试工具 | _Requirements: 所有需求 | Success: 集成测试通过，用户工作流验证正确，测试自动化完整 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

## 阶段6：文档和部署

### M6.1：文档完善
- [ ] 25. 编写用户文档
  - 文件: docs/mcp-ai-agent-user-guide.md
  - 创建用户使用指南和配置说明
  - 包含常见问题和故障排除
  - 目的: 帮助用户理解和使用新功能
  - _Leverage: 现有的文档模板_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: 技术文档专家，专精用户指南和教程编写 | Task: 编写MCP和智能体功能的用户文档，包含配置指南和故障排除，遵循所有需求的规范 | Restrictions: 必须语言清晰易懂，包含实际操作步骤，提供问题解决方案 | _Leverage: 现有文档模板, 文档写作规范 | _Requirements: 所有需求 | Success: 文档完整清晰，用户能够按照指南成功使用功能 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 26. 编写开发者文档
  - 文件: docs/mcp-ai-agent-developer-guide.md
  - 创建开发者API文档和扩展指南
  - 包含架构说明和代码示例
  - 目的: 支持开发者扩展和维护功能
  - _Leverage: 现有的开发者文档模式_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: 技术架构文档专家，专精API文档和开发指南 | Task: 编写MCP和智能体的开发者文档，包含API说明、架构介绍和代码示例，遵循所有需求的规范 | Restrictions: 必须技术准确，包含完整API参考，提供可运行的示例 | _Leverage: 现有开发者文档模式, 代码注释 | _Requirements: 所有需求 | Success: 开发者文档技术准确，API参考完整，示例代码可用 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

### M6.2：最终集成和验证
- [ ] 27. 系统集成验证
  - 跨平台功能验证（Windows/macOS/Android）
  - 性能测试和优化
  - 安全性检查和漏洞修复
  - 目的: 确保系统整体质量
  - _Leverage: 现有的CI/CD流程_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: 系统集成工程师，专精跨平台测试和性能优化 | Task: 进行系统集成验证，包含跨平台测试、性能优化、安全检查，遵循所有需求的规范 | Restrictions: 必须在所有支持平台测试，确保性能达标，修复安全问题 | _Leverage: 现有CI/CD流程, 测试工具链 | _Requirements: 所有需求 | Success: 所有平台功能正常，性能满足要求，安全检查通过 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_

- [ ] 28. 发布准备和清理
  - 代码审查和质量检查
  - 版本标记和发布说明
  - 清理临时文件和调试代码
  - 目的: 准备功能发布
  - _Leverage: 现有的发布流程_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec appflowy-mcp-ai-agent, first run spec-workflow-guide to get the workflow guide then implement the task: Role: 发布工程师，专精代码质量和版本管理 | Task: 完成发布准备工作，包含代码审查、版本标记、清理工作，遵循所有需求的规范 | Restrictions: 必须通过代码审查，确保版本信息正确，清理所有临时内容 | _Leverage: 现有发布流程, 代码审查工具 | _Requirements: 所有需求 | Success: 代码质量达标，版本准备完成，功能可以正式发布 | Instructions: 首先将任务状态标记为进行中[-]，完成后标记为已完成[x]_
