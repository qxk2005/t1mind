# AI聊天MCP工具编排功能 — 任务文档（Tasks）

## 核心数据模型和类型定义

- [x] 1. 创建任务规划相关数据模型
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/task_planner_entities.dart
  - 定义TaskPlan、TaskStep、AgentConfig等核心数据结构
  - 实现JSON序列化和反序列化方法
  - 目的: 建立任务规划功能的数据基础
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/application/chat_entity.dart_
  - _Requirements: 需求1, 需求6_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter开发者，专精Dart数据模型和序列化 | Task: 创建任务规划相关的数据模型，包括TaskPlan、TaskStep、AgentConfig等，参考现有chat_entity.dart的模式，实现完整的JSON序列化支持 | Restrictions: 必须遵循现有项目的数据模型模式，不要修改现有的chat_entity.dart，保持与现有代码的兼容性 | Success: 所有数据模型编译无错误，JSON序列化工作正常，类型安全覆盖所有任务规划需求_

- [x] 2. 创建执行日志数据模型
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/execution_log_entities.dart
  - 定义ExecutionLog、ExecutionStep、McpToolInfo等数据结构
  - 添加状态枚举和错误处理类型
  - 目的: 支持执行追溯和日志管理功能
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/application/chat_entity.dart_
  - _Requirements: 需求4, 需求7_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter开发者，专精状态管理和数据持久化 | Task: 创建执行日志相关数据模型，包括ExecutionLog、ExecutionStep等，支持完整的执行追溯功能，参考chat_entity.dart的设计模式 | Restrictions: 必须支持状态枚举，包含完整的错误处理类型，不要与现有消息实体冲突 | Success: 执行日志模型完整，支持状态追踪，错误处理完善，与聊天系统集成良好_

## Rust核心层实现

- [x] 3. 扩展Rust AI管理器支持任务编排
  - 文件: rust-lib/flowy-ai/src/task_orchestrator.rs
  - 实现TaskOrchestrator结构体和核心方法
  - 集成现有的MCPClientManager和AI服务
  - 目的: 提供任务规划和执行的核心逻辑
  - _Leverage: rust-lib/flowy-ai/src/ai_manager.rs, rust-lib/flowy-ai/src/mcp/manager.rs_
  - _Requirements: 需求1, 需求2, 需求3_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Rust后端开发者，专精异步编程和系统架构 | Task: 实现TaskOrchestrator核心逻辑，集成现有AI管理器和MCP客户端管理器，提供任务规划和执行能力 | Restrictions: 必须保持与现有AI系统的兼容性，不要破坏现有MCP管理功能，确保线程安全和异步处理 | Success: TaskOrchestrator功能完整，与现有系统集成良好，支持并发任务执行，错误处理健壮_

- [x] 4. 实现执行日志记录器
  - 文件: rust-lib/flowy-ai/src/execution_logger.rs
  - 创建ExecutionLogger结构体，支持日志持久化
  - 实现日志查询、过滤和导出功能
  - 目的: 提供完整的执行追溯能力
  - _Leverage: rust-lib/flowy-sqlite/src/lib.rs_
  - _Requirements: 需求4, 需求7_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Rust数据库开发者，专精SQLite和数据持久化 | Task: 实现ExecutionLogger，支持执行日志的完整生命周期管理，包括记录、查询、过滤和导出功能 | Restrictions: 必须使用现有的SQLite基础设施，确保数据安全和性能，支持大量日志数据的高效查询 | Success: 执行日志功能完整，数据持久化可靠，查询性能良好，支持多种导出格式_

- [x] 5. 扩展FFI接口支持新功能
  - 文件: rust-lib/dart-ffi/src/lib.rs (修改现有文件)
  - 添加任务规划和执行相关的FFI函数
  - 实现事件通知机制支持实时状态更新
  - 目的: 为Flutter层提供完整的Rust功能访问
  - _Leverage: 现有FFI模式和事件系统_
  - _Requirements: 需求1, 需求2, 需求3, 需求4_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: 系统集成开发者，专精FFI和跨语言通信 | Task: 扩展现有FFI接口，添加任务编排相关功能，实现Flutter到Rust的完整通信支持 | Restrictions: 必须遵循现有FFI模式，不要破坏现有接口，确保类型安全和错误处理 | Success: FFI接口完整，Flutter可以访问所有Rust功能，事件通知工作正常，类型转换安全_

## Flutter应用层实现

- [x] 6. 实现任务规划BLoC
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/task_planner_bloc.dart
  - 创建TaskPlannerBloc，管理任务规划状态
  - 实现规划、确认、执行状态转换逻辑
  - 目的: 提供任务规划的状态管理
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/application/chat_bloc.dart_
  - _Requirements: 需求1_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter状态管理专家，专精BLoC模式 | Task: 实现TaskPlannerBloc，管理任务规划的完整状态流程，参考现有ChatBloc的设计模式 | Restrictions: 必须遵循现有BLoC模式，确保状态转换的一致性，处理所有边界情况 | Success: 状态管理完整，状态转换逻辑清晰，与UI层集成良好，错误状态处理完善_

- [x] 7. 实现执行监控BLoC
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/execution_bloc.dart
  - 创建ExecutionBloc，监控任务执行状态
  - 实现实时进度更新和错误处理
  - 目的: 提供任务执行的实时监控
  - _Leverage: appflowy_flutter/lib/plugins/ai_chat/application/chat_ai_message_bloc.dart_
  - _Requirements: 需求3_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter实时数据处理专家，专精流式状态管理 | Task: 实现ExecutionBloc，提供任务执行的实时监控和状态更新，参考AI消息BLoC的流式处理模式 | Restrictions: 必须支持实时更新，确保UI响应性，处理长时间运行的任务 | Success: 执行监控功能完整，实时更新流畅，用户体验良好，支持取消和重试操作_

- [x] 8. 实现智能体配置BLoC
  - 文件: appflowy_flutter/lib/plugins/ai_chat/application/agent_config_bloc.dart
  - 创建AgentConfigBloc，管理智能体配置
  - 实现配置的CRUD操作和验证逻辑
  - 目的: 支持智能体的个性化配置
  - _Leverage: appflowy_flutter/lib/workspace/application/settings/ai/settings_ai_bloc.dart_
  - _Requirements: 需求6_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter配置管理专家，专精设置和验证逻辑 | Task: 实现AgentConfigBloc，提供智能体配置的完整管理功能，参考现有AI设置BLoC的模式 | Restrictions: 必须包含配置验证，支持多个智能体管理，确保配置持久化 | Success: 智能体配置功能完整，验证逻辑健壮，支持导入导出，用户界面友好_

## UI组件实现

- [x] 9. 创建任务确认对话框组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/task_confirmation_dialog.dart
  - 实现TaskConfirmationDialog，展示任务规划详情
  - 添加确认、拒绝和修改选项
  - 目的: 提供用户确认任务规划的界面
  - _Leverage: appflowy_flutter/lib/shared/af_dialog.dart_
  - _Requirements: 需求1_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter UI开发者，专精对话框和用户交互设计 | Task: 创建任务确认对话框，提供清晰的任务规划展示和用户确认界面，使用现有对话框组件模式 | Restrictions: 必须遵循现有设计系统，确保可访问性，支持键盘导航 | Success: 对话框界面清晰易懂，用户交互流畅，支持所有确认选项，符合设计规范_

- [x] 10. 创建执行进度显示组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/execution_progress_widget.dart
  - 实现ExecutionProgressWidget，显示实时执行进度
  - 添加步骤详情和取消功能
  - 目的: 提供任务执行的可视化反馈
  - _Leverage: appflowy_flutter/lib/shared/progress_indicator.dart_
  - _Requirements: 需求3_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter UI开发者，专精进度显示和动画效果 | Task: 创建执行进度组件，提供直观的任务执行状态显示，包括步骤详情和控制功能 | Restrictions: 必须支持实时更新，确保性能优化，提供清晰的视觉反馈 | Success: 进度显示准确，动画流畅，用户可以清楚了解执行状态，支持交互控制_

- [x] 11. 创建MCP工具选择器组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/mcp_tool_selector.dart
  - 实现McpToolSelector，支持多选MCP工具
  - 添加工具状态显示和过滤功能
  - 目的: 让用户选择要使用的MCP工具
  - _Leverage: appflowy_flutter/lib/shared/multi_select.dart_
  - _Requirements: 需求2_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter组件开发者，专精选择器和过滤界面 | Task: 创建MCP工具选择器，支持多选、过滤和状态显示，使用现有多选组件模式 | Restrictions: 必须支持大量工具的高效显示，提供搜索和分类功能，确保选择状态清晰 | Success: 工具选择器功能完整，支持快速选择和搜索，状态显示准确，用户体验良好_

- [x] 12. 创建执行日志查看器组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/execution_log_viewer.dart
  - 实现ExecutionLogViewer，展示详细执行日志
  - 添加搜索、过滤和导出功能
  - 目的: 提供执行追溯和调试支持
  - _Leverage: appflowy_flutter/lib/shared/log_viewer.dart_
  - _Requirements: 需求4_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter数据展示专家，专精列表和搜索界面 | Task: 创建执行日志查看器，提供完整的日志浏览、搜索和导出功能，参考现有日志组件模式 | Restrictions: 必须支持大量日志数据的高效显示，提供多种过滤选项，确保导出功能完整 | Success: 日志查看器性能良好，搜索功能准确，导出格式多样，用户可以快速定位问题_

## 聊天界面集成

- [x] 13. 扩展聊天输入区域
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/chat_page/chat_footer.dart (修改现有文件)
  - 在现有输入区域添加MCP工具选择器
  - 集成任务规划触发逻辑
  - 目的: 在聊天界面中集成新功能
  - _Leverage: 现有chat_footer.dart的布局和逻辑_
  - _Requirements: 需求1, 需求2_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter UI集成专家，专精现有组件扩展 | Task: 扩展现有聊天输入区域，无缝集成MCP工具选择和任务规划功能，保持现有用户体验 | Restrictions: 不要破坏现有聊天功能，确保向后兼容，保持界面简洁 | Success: 新功能集成无缝，现有功能不受影响，用户界面保持一致性，交互逻辑清晰_

- [x] 14. 扩展AI消息显示组件
  - 文件: appflowy_flutter/lib/plugins/ai_chat/presentation/message/ai_text_message.dart (修改现有文件)
  - 添加执行日志和引用信息显示
  - 实现展开/折叠详细信息功能
  - 目的: 在AI回复中显示执行追溯信息
  - _Leverage: 现有ai_text_message.dart的渲染逻辑_
  - _Requirements: 需求4_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter消息渲染专家，专精富文本和交互组件 | Task: 扩展AI消息组件，添加执行日志和引用信息的显示功能，保持消息渲染的性能和美观 | Restrictions: 必须保持消息渲染性能，不要影响现有消息类型，确保信息层次清晰 | Success: 执行信息显示完整，消息渲染性能良好，用户可以方便查看详细信息，界面布局合理_

## 设置界面扩展

- [ ] 15. 扩展工作空间AI设置页面
  - 文件: appflowy_flutter/lib/workspace/presentation/settings/pages/setting_ai_view/settings_ai_view.dart (修改现有文件)
  - 添加智能体配置管理界面
  - 集成MCP工具管理功能
  - 目的: 在工作空间设置中提供完整配置
  - _Leverage: 现有settings_ai_view.dart的布局结构_
  - _Requirements: 需求5, 需求6_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter设置界面专家，专精配置管理和表单设计 | Task: 扩展工作空间AI设置页面，添加智能体和MCP工具的完整配置管理功能 | Restrictions: 必须保持现有设置页面的结构，确保配置项组织清晰，支持不同权限级别 | Success: 设置界面功能完整，配置项组织合理，用户可以方便管理所有AI相关设置_

- [ ] 16. 扩展移动端AI设置
  - 文件: appflowy_flutter/lib/mobile/presentation/setting/ai/ai_settings_group.dart (修改现有文件)
  - 适配移动端界面，添加智能体配置选项
  - 简化复杂配置，提供快捷设置
  - 目的: 在移动端提供适配的配置界面
  - _Leverage: 现有ai_settings_group.dart的移动端适配模式_
  - _Requirements: 需求5, 需求6_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: 移动端UI专家，专精触摸界面和简化设计 | Task: 扩展移动端AI设置，提供适合触摸操作的智能体配置界面，简化复杂配置项 | Restrictions: 必须适配小屏幕，确保触摸友好，简化配置流程，保持移动端体验 | Success: 移动端设置界面友好，配置流程简化，触摸操作流畅，功能覆盖完整_

## 国际化和本地化

- [ ] 17. 添加中文本地化资源
  - 文件: resources/translations/zh-CN.json (修改现有文件)
  - 添加所有新功能的中文翻译
  - 确保术语一致性和表达准确性
  - 目的: 提供完整的中文用户界面
  - _Leverage: 现有zh-CN.json的翻译模式_
  - _Requirements: 需求5_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: 本地化专家，专精中文技术术语翻译 | Task: 为所有新功能添加准确的中文翻译，确保术语一致性和用户理解 | Restrictions: 必须保持现有翻译风格，确保术语准确，避免歧义表达 | Success: 中文界面完整，术语准确一致，用户理解无障碍，翻译质量高_

- [ ] 18. 添加英文本地化资源
  - 文件: resources/translations/en-US.json (修改现有文件)
  - 添加所有新功能的英文翻译
  - 确保专业术语的准确使用
  - 目的: 提供完整的英文用户界面
  - _Leverage: 现有en-US.json的翻译模式_
  - _Requirements: 需求5_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: 技术文档专家，专精英文技术写作 | Task: 为所有新功能添加专业的英文翻译，确保技术术语准确和表达清晰 | Restrictions: 必须使用标准技术术语，确保表达清晰，保持专业性 | Success: 英文界面专业完整，术语使用准确，表达清晰易懂，符合国际标准_

## 测试和质量保证

- [ ] 19. 编写核心功能单元测试
  - 文件: appflowy_flutter/test/plugins/ai_chat/task_planner_test.dart
  - 为TaskPlannerBloc和相关逻辑编写单元测试
  - 覆盖所有状态转换和错误场景
  - 目的: 确保核心功能的可靠性
  - _Leverage: appflowy_flutter/test/plugins/ai_chat/chat_bloc_test.dart_
  - _Requirements: 需求1, 需求3_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: Flutter测试专家，专精BLoC测试和状态验证 | Task: 编写TaskPlannerBloc的完整单元测试，覆盖所有状态转换和边界情况，参考现有聊天测试模式 | Restrictions: 必须测试所有状态转换，包含错误场景，确保测试独立性和可重复性 | Success: 测试覆盖率高，所有状态转换被验证，边界情况处理正确，测试运行稳定_

- [ ] 20. 编写集成测试
  - 文件: appflowy_flutter/integration_test/ai_chat_mcp_test.dart
  - 编写端到端的功能集成测试
  - 测试完整的用户工作流程
  - 目的: 验证功能的端到端正确性
  - _Leverage: appflowy_flutter/integration_test/desktop/board/board_test.dart_
  - _Requirements: 所有需求_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: QA自动化工程师，专精集成测试和用户流程验证 | Task: 编写完整的集成测试，验证从任务规划到执行完成的整个用户工作流程 | Restrictions: 必须测试真实用户场景，确保测试稳定性，覆盖跨平台兼容性 | Success: 集成测试覆盖完整流程，测试运行稳定，用户体验得到验证，跨平台功能正常_

## 文档和部署

- [ ] 21. 更新用户文档
  - 文件: docs/user_guide/ai_chat_mcp_orchestration.md
  - 编写功能使用指南和最佳实践
  - 添加常见问题解答和故障排除
  - 目的: 帮助用户理解和使用新功能
  - _Leverage: docs/templates/user_guide_template.md_
  - _Requirements: 所有需求_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: 技术文档写作专家，专精用户指南和教程编写 | Task: 编写完整的用户指南，包括功能介绍、使用步骤、最佳实践和故障排除 | Restrictions: 必须面向最终用户，语言简洁易懂，提供充足的示例和截图 | Success: 用户文档完整易懂，用户可以独立学习使用功能，常见问题得到解答_

- [ ] 22. 最终集成和代码清理
  - 文件: 所有相关文件的最终检查和优化
  - 进行代码审查和性能优化
  - 确保所有功能正常集成
  - 目的: 确保代码质量和系统稳定性
  - _Leverage: 项目代码规范和质量检查工具_
  - _Requirements: 所有需求_
  - _Prompt: 为规格ai-chat-mcp-orchestration实现任务，首先运行spec-workflow-guide获取工作流程指南然后实现任务：Role: 高级开发者，专精代码质量和系统集成 | Task: 进行最终的代码审查、性能优化和集成验证，确保所有功能正常工作且代码质量达标 | Restrictions: 不能破坏现有功能，必须遵循项目代码规范，确保性能不退化 | Success: 所有功能完整集成，代码质量高，性能良好，系统稳定可靠，满足所有需求_
