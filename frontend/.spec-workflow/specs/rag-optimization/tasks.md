# RAG优化功能 — 任务文档

## Phase 1: 后端RAG配置管理（高优先级）

- [x] 1. 在`rust-lib/flowy-ai/src/entities.rs`中添加RAG配置相关的protobuf结构
  - 文件: `rust-lib/flowy-ai/src/entities.rs`
  - 添加`RAGSettingsPB`、`RAGConfigPB`等protobuf结构体
  - 定义RAG配置的读写请求和响应
  - 目的: 为前后端通信定义数据协议
  - _Leverage: 参考`AgentConfigPB`、`WebSearchGlobalConfigPB`的结构_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in protobuf and data modeling | Task: Add RAG configuration protobuf structures to flowy-ai/src/entities.rs, following the pattern of existing AgentConfigPB and WebSearchGlobalConfigPB, defining RAGSettingsPB with fields for chunk_size, chunk_overlap, hybrid search weights, top_k parameters, and reflection settings | Restrictions: Must follow existing ProtoBuf macro conventions, use appropriate field indices, include validation where needed, maintain backward compatibility | Success: All protobuf structures compile without errors, match the design document specifications, integrate properly with existing protobuf code_

- [x] 2. 创建`rust-lib/flowy-ai/src/rag/config_manager.rs`
  - 文件: `rust-lib/flowy-ai/src/rag/config_manager.rs`
  - 实现RAG配置管理器，支持读写配置到KVStorePreferences
  - 添加配置验证逻辑
  - 目的: 提供统一的RAG配置管理接口
  - _Leverage: `rust-lib/flowy-ai/src/agent/config_manager.rs`, `rust-lib/flowy-sqlite/src/kv.rs`_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Backend Developer with expertise in configuration management | Task: Create RAGConfigManager in flowy-ai/src/rag/config_manager.rs following the pattern of AgentConfigManager, implementing get_rag_settings, save_rag_settings, and validate_settings methods, storing config in KVStorePreferences with key "rag_global_settings" | Restrictions: Must follow existing KVStorePreferences patterns, validate parameter ranges (chunk_size>0, weights sum=1, etc.), handle errors gracefully, ensure thread safety | Success: Config manager reads/writes to local storage correctly, validation prevents invalid configurations, all methods work as specified in design document_

- [x] 3. 修改`DocumentIndexer`支持配置化切块
  - 文件: `rust-lib/flowy-ai/src/embeddings/document_indexer.rs`
  - 修改`create_embedded_chunks_from_text`接受配置参数
  - 修改`split_text_into_chunks`使用配置的chunk_size和chunk_overlap
  - 目的: 统一切块大小，消除1000和2000的差异
  - _Leverage: 当前`document_indexer.rs`的实现，新建的`RAGConfigManager`_
  - _Requirements: 需求1_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in text processing and embeddings | Task: Modify DocumentIndexer to read chunk_size and chunk_overlap from RAGConfigManager instead of hardcoding values, update both the indexing path and direct add_documents path to use consistent chunking | Restrictions: Must maintain existing API compatibility, preserve text splitting logic, ensure both paths use the same configuration, handle configuration errors gracefully | Success: Both indexing and direct document addition use the same chunk_size from config, no hardcoded values remain, text splitting works correctly with configurable parameters_

- [x] 4. 修改`store.rs`修复切块不一致
  - 文件: `rust-lib/flowy-ai/src/embeddings/store.rs`
  - 修改第235行附近的`split_text_into_chunks`调用使用配置
  - 修复2000 vs 1000的不一致问题
  - 目的: 确保所有文档使用统一的切块参数
  - _Leverage: `RAGConfigManager`, 当前的`split_text_into_chunks`实现_
  - _Requirements: 需求1_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with expertise in vector stores and embeddings | Task: Modify the embed_paragraphs method in flowy-ai/src/embeddings/store.rs to use RAGConfigManager for chunk_size and chunk_overlap instead of hardcoded 2000/200, ensuring consistency with DocumentIndexer | Restrictions: Must pass config to split_text_into_chunks correctly, maintain async semantics, ensure error handling is preserved, keep metadata handling intact | Success: Direct document addition uses configurable chunk_size from RAG settings, matches indexing path behavior, no hardcoded 2000 values remain_

## Phase 2: 混合检索功能（中优先级）

- [x] 5. 创建关键词检索接口trait ✅ (已完成：实现了完整的 BM25 算法，不使用 FTS5)
  - 文件: `rust-lib/flowy-ai/src/rag/keyword_retriever.rs`
  - 定义`KeywordStore` trait和`KeywordDocument`结构
  - 实现SQLite全文搜索能力（FTS5或tantivy集成）
  - 目的: 提供关键词检索能力基础
  - _Leverage: `flowy-sqlite-vec`的表结构，现有向量存储_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in full-text search and database integration | Task: Create KeywordStore trait and SQLiteKeywordStore implementation in flowy-ai/src/rag/keyword_retriever.rs, using SQLite FTS5 extension or tantivy library for full-text indexing and searching on document content | Restrictions: Must design efficient indexing strategy, handle query parsing for FTS5 syntax, support ranking algorithms like BM25, ensure concurrent access safety, integrate with existing VectorSqliteDB | Success: Keyword retrieval returns documents ranked by relevance, efficiently searches full document content, properly integrates with existing vector database schema_

- [x] 6. 创建`HybridRetriever`实现
  - 文件: `rust-lib/flowy-ai/src/rag/hybrid_retriever.rs`
  - 实现向量检索和关键词检索的合并逻辑
  - 支持可配置的权重和去重
  - 目的: 提供混合检索能力
  - _Leverage: `SqliteVectorStore`, 新建的`KeywordStore`, 现有的检索接口_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in search algorithms and data fusion | Task: Create HybridRetriever in flowy-ai/src/rag/hybrid_retriever.rs that parallelly executes vector and keyword retrieval, merges results by weighted scoring, removes duplicates by document_id, and returns sorted candidate set | Restrictions: Must handle both retrieval types gracefully if one fails, implement proper score normalization and weighting, ensure efficient deduplication, maintain async performance | Success: Hybrid retrieval returns merged results with combined scores, handles failures gracefully with fallback to single retrieval type, deduplication works correctly, performance is acceptable_

## Phase 3: 检索流程集成（中优先级）

- [x] 7. 扩展`EmbedScheduler`支持配置化检索
  - 文件: `rust-lib/flowy-ai/src/embeddings/scheduler.rs`
  - 修改`search_with_filter`方法支持读取RAG配置的top_k参数
  - 添加对混合检索的支持
  - 目的: 让检索流程使用配置参数和混合检索
  - _Leverage: 当前`scheduler.rs`的实现，新建的`RAGConfigManager`, `HybridRetriever`_
  - _Requirements: 需求3_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in search orchestration | Task: Modify EmbedScheduler::search_with_filter to read top_k from RAGConfigManager, integrate HybridRetriever for hybrid search when enabled, support two-stage retrieval with initial_top_k and final_top_k | Restrictions: Must maintain backward compatibility, handle configuration errors, preserve existing single-vector-search fallback, ensure performance doesn't degrade | Success: Search uses configurable top_k parameters, hybrid search works when enabled, two-stage retrieval selects best documents, maintains or improves search quality_

- [x] 8. 创建重排序器（可选）
  - 文件: `rust-lib/flowy-ai/src/rag/reranker.rs`
  - 实现重排序逻辑（初始版本使用分数排序，预留模型接口）
  - 添加降级处理
  - 目的: 为未来重排序模型集成做准备
  - _Leverage: 当前搜索结果的分数，LLM集成点_
  - _Requirements: 需求3_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in ranking algorithms | Task: Create Reranker in flowy-ai/src/rag/reranker.rs that sorts candidates by combined_score initially (model integration can be added later), implements graceful fallback when reranking disabled or model unavailable | Restrictions: Must handle empty result sets, preserve document metadata, implement efficient sorting, support future model integration without breaking changes | Success: Reranking sorts documents by relevance score, handles all edge cases, integrates seamlessly into retrieval pipeline, ready for future model integration_

## Phase 4: 前端设置界面（高优先级）

- [x] 9. 在entities.rs中添加RAG设置前端结构
  - 文件: `appflowy_flutter/lib/workspace/presentation/settings/entities.dart`
  - 添加RAG配置的Dart类，对应后端的protobuf结构
  - 定义事件和状态类
  - 目的: 定义前端数据模型
  - _Leverage: 现有的`openai_compat_setting_bloc.dart`的数据结构_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer specializing in state management and data models | Task: Create RAGSettingsPB Dart classes in workspace/presentation/settings/entities.dart matching the protobuf definitions, including RAGSettingsState, RAGSettingsEvent classes for bloc pattern | Restrictions: Must follow existing protobuf conversion patterns, use proper type mapping, maintain null safety, ensure immutability where appropriate | Success: All RAG setting classes compile without errors, properly map to Rust protobuf, support all required configuration fields_

- [x] 10. 创建RAG设置Bloc
  - 文件: `appflowy_flutter/lib/workspace/application/settings/ai/rag_setting_bloc.dart`
  - 实现Bloc/Cubit管理RAG设置状态
  - 实现加载、保存、测试等功能
  - 目的: 提供RAG设置的状态管理
  - _Leverage: `openai_compat_setting_bloc.dart`的实现模式_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer specializing in BLoC pattern and state management | Task: Create RAGSettingBloc in workspace/application/settings/ai/rag_setting_bloc.dart following the pattern of OpenAICompatSettingBloc, implementing load settings, update settings, and save settings with AI manager integration | Restrictions: Must handle loading/error states properly, validate inputs before saving, provide user feedback on save success/failure, maintain reactive UI updates | Success: Bloc manages RAG settings state correctly, loads from backend on init, saves changes to backend, provides clear loading and error feedback to users_

- [x] 11. 创建RAG设置UI组件
  - 文件: `appflowy_flutter/lib/workspace/presentation/settings/ai/rag_setting_view.dart`
  - 实现RAG设置的UI界面，包含所有配置项
  - 支持实时预览和验证
  - 目的: 提供友好的RAG配置界面
  - _Leverage: 现有的设置页面UI组件库_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI Developer specializing in settings interfaces and form design | Task: Create RAGSettingView UI in workspace/presentation/settings/ai/rag_setting_view.dart with all configuration sections (chunking, retrieval weights, top_k parameters, reflection settings) using existing AppFlowy UI components | Restrictions: Must follow AppFlowy design system, provide input validation with clear error messages, show recommended values for each parameter, ensure responsive layout, maintain accessibility standards | Success: UI displays all RAG settings in organized sections, inputs validate correctly with helpful hints, saves successfully to backend, provides clear feedback on configuration changes_

## Phase 5: 多语言支持（中优先级）

- [ ] 12. 添加RAG设置的中文翻译
  - 文件: `resources/translations/zh-CN.json`
  - 添加RAG设置相关的所有翻译键
  - 包括设置标题、配置项说明、错误消息等
  - 目的: 支持简体中文界面
  - _Leverage: 现有的翻译结构和已有的设置翻译_
  - _Requirements: 需求6_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Chinese Localization Specialist | Task: Add RAG settings translations to resources/translations/zh-CN.json following existing structure, translating all UI labels, tooltips, validation messages, and help text for chunking, retrieval, and reflection settings | Restrictions: Must maintain translation key structure consistency, use appropriate technical terminology, ensure clarity for end users, match existing translation style | Success: All RAG setting strings are properly translated to Chinese, translations are clear and user-friendly, key structure matches frontend code_

- [ ] 13. 添加RAG设置的英文翻译
  - 文件: `resources/translations/en-US.json`
  - 添加RAG设置相关的英文翻译
  - 目的: 支持英文界面
  - _Leverage: 现有的英文翻译结构_
  - _Requirements: 需求6_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: English Localization Specialist | Task: Add RAG settings translations to resources/translations/en-US.json, providing clear and professional English translations for all RAG configuration UI elements | Restrictions: Must use consistent terminology, ensure technical accuracy, maintain professional tone, follow AppFlowy writing guidelines | Success: All RAG setting strings are translated to clear English, terminology is consistent, professional tone maintained throughout_

- [ ] 14. 运行多语言代码生成脚本
  - 命令: `/Users/niuzhidao/Documents/Program/t1mind/frontend/scripts/code_generation/language_files/generate_language_files.sh`
  - 生成Flutter端的locale代码
  - 目的: 使翻译生效
  - _Leverage: 现有的语言文件生成脚本_
  - _Requirements: 需求6_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer specializing in code generation | Task: Run the language files generation script to generate Flutter Dart code from translation JSON files, ensuring RAG setting strings are available in locale_keys.g.dart | Restrictions: Must run from correct directory, ensure script has proper permissions, verify generated code compiles, check for any missing translations | Success: Language code generation completes without errors, locale_keys.g.dart includes all RAG setting keys, generated code compiles successfully_

## Phase 6: 智能体反思集成（中优先级）

- [x] 15. 创建智能体反思模块
  - 文件: `rust-lib/flowy-ai/src/rag/agent_reflection.rs`
  - 实现反思逻辑：评估答案质量，判断是否解决问题
  - 提供改进建议
  - 目的: 让智能体能够自我评估回答质量
  - _Leverage: 现有的LLM调用接口，智能体的执行框架_
  - _Requirements: 需求5_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: AI Engineer specializing in agentic AI and self-reflection | Task: Create AgentReflection module in flowy-ai/src/rag/agent_reflection.rs that evaluates AI answers against user questions, determines if question is fully resolved, provides suggestions for missing context, integrates with existing LLM calls | Restrictions: Must avoid circular reasoning, ensure reflection provides value, handle cases where reflection is inconclusive, maintain reasonable performance | Success: Reflection module evaluates answers and provides useful feedback, detects unsolved questions, suggests improvements, integrates seamlessly with agent execution flow_

- [x] 16. 集成反思机制到智能体对话流程
  - 文件: `rust-lib/flowy-ai/src/local_ai/chat/chains/conversation_chain.rs`
  - 在对话流程中添加反思步骤
  - 根据反思结果调整回答或提示
  - 目的: 让智能体在实际对话中使用反思能力
  - _Leverage: 当前conversation_chain的实现，新建的`AgentReflection`_
  - _Requirements: 需求5_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: AI Engineer specializing in conversational AI and agent workflows | Task: Integrate AgentReflection into the conversation chain, adding reflection step after answer generation, using reflection results to enhance or modify the response | Restrictions: Must not significantly slow down response generation, handle reflection failures gracefully, maintain conversation flow, avoid redundant reflections | Success: Reflection is called after answers when enabled, reflection results influence subsequent responses or prompts, conversation quality improves with reflection enabled_

## Phase 7: 测试和验证（高优先级）

- [ ] 17. 添加RAG配置管理器的单元测试
  - 文件: `rust-lib/flowy-ai/src/rag/config_manager_test.rs`
  - 测试配置读写、验证逻辑
  - 目的: 确保配置管理可靠性
  - _Leverage: AppFlowy现有的测试框架_
  - _Requirements: 非功能需求-可靠性_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in Rust unit testing | Task: Write comprehensive unit tests for RAGConfigManager covering get/save operations, validation logic, error handling, testing both valid and invalid configurations | Restrictions: Must test edge cases, ensure tests are isolated, mock dependencies appropriately, maintain test maintainability | Success: All configuration operations are tested, validation catches invalid inputs, tests pass consistently, coverage meets thresholds_

- [ ] 18. 添加混合检索的集成测试
  - 文件: `rust-lib/flowy-ai/src/rag/hybrid_retriever_test.rs`
  - 测试混合检索的合并逻辑和权重计算
  - 目的: 确保检索质量
  - _Leverage: AppFlowy测试数据和框架_
  - _Requirements: 需求2_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer specializing in integration testing | Task: Create integration tests for HybridRetriever testing document retrieval, result merging, score weighting, and deduplication with sample documents | Restrictions: Must use realistic test data, verify score calculations are correct, test both retrieval types separately and combined, ensure proper handling of edge cases | Success: Tests verify hybrid retrieval improves recall, score weighting works correctly, deduplication handles duplicates properly, performance is acceptable_

- [ ] 19. 端到端测试：完整的RAG优化流程
  - 文件: `appflowy_flutter/integration_test/rag_optimization_test.dart`
  - 测试从配置修改到实际检索和回答的完整流程
  - 目的: 验证整个功能链
  - _Leverage: AppFlowy的集成测试框架_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer specializing in E2E testing | Task: Create end-to-end integration test for RAG optimization covering configuration changes, document indexing with new settings, retrieval with hybrid search, and answer quality verification | Restrictions: Must test real user workflows, ensure tests are maintainable and reliable, handle flakiness appropriately, verify actual RAG improvements | Success: E2E test validates complete RAG optimization workflow, demonstrates improved retrieval accuracy, tests configuration persistence, verifies cross-platform behavior_

## Phase 8: 文档和发布（中优先级）

- [ ] 20. 更新changelog.md记录RAG优化功能
  - 文件: `appflowy_flutter/assets/changelog.md`
  - 添加新版本的功能说明
  - 目的: 记录新功能
  - _Leverage: 现有的changelog格式_
  - _Requirements: 所有需求_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Update changelog.md with RAG optimization features, listing key improvements in chunking consistency, hybrid search, and configurable retrieval parameters | Restrictions: Must follow existing changelog format, be concise but informative, highlight user-facing improvements, use appropriate emoji and formatting | Success: Changelog clearly documents RAG optimization features, users can understand improvements, format matches existing entries_

- [ ] 21. 运行protobuf代码生成
  - 命令: `cd rust-lib/flowy-ai && cargo build --features dart`
  - 生成Dart的protobuf绑定
  - 目的: 使前后端protobuf同步
  - _Leverage: AppFlowy的protobuf生成系统_
  - _Requirements: 需求4, 需求6_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build Engineer | Task: Run AppFlowy protobuf code generation to create Dart bindings for new RAG configuration protobuf structures | Restrictions: Must ensure all Rust changes are committed first, verify no breaking changes to existing protobuf, run from correct directory, check for errors | Success: Dart protobuf files are generated successfully, no compilation errors, new RAG structures are available in Dart_

- [ ] 22. 运行freezed代码生成（如需要）
  - 命令: `cd appflowy_flutter && dart run build_runner build --delete-conflicting-outputs`
  - 生成freezed类
  - 目的: 为Dart数据类生成不可变实现
  - _Leverage: Dart的freezed包_
  - _Requirements: 需求4_
  - _Prompt: Implement the task for spec rag-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Build Engineer | Task: Run build_runner to generate freezed classes for any RAG-related data structures that use freezed annotations | Restrictions: Must handle code generation errors gracefully, ensure clean build, check for conflicts | Success: All freezed classes are generated, no build conflicts, generated code compiles successfully_

