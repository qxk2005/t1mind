# Tasks Document

## Implementation Tasks

- [x] 1. 增强 ChatInputControlCubit 以支持 @mention 模式
  - 文件: `appflowy_flutter/lib/plugins/ai_chat/application/chat_input_control_cubit.dart`
  - 添加 `isAtMentionMode` 状态跟踪
  - 添加 `atSymbolPosition` 跟踪 @ 符号位置
  - 实现 `enableMentionTracking()` 方法监听文本变化
  - 实现 `_onTextChanged()` 方法检测 @ 符号
  - 实现 `_startAtMention()` 和 `_continueAtMention()` 方法
  - 目的: 使 ChatInputControlCubit 能够检测和处理 @mention 输入
  - _Leverage: 现有的 `refreshViews()`, `updateFilter()` 方法_
  - _Requirements: 1, 2_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in Cubit state management | Task: Enhance ChatInputControlCubit to support @mention mode by adding state tracking (isAtMentionMode, atSymbolPosition), implementing enableMentionTracking() to listen to text changes, and implementing _onTextChanged(), _startAtMention(), _continueAtMention() methods following requirements 1 and 2. Leverage existing refreshViews() and updateFilter() methods | Restrictions: Do not break existing mention functionality, maintain backward compatibility, handle text changes efficiently without performance issues | Success: @ symbol is detected correctly, mention mode is properly tracked, text listener works without memory leaks, existing functionality remains intact_

- [x] 2. 创建 DocumentMentionExtractor 工具类
  - 文件: `appflowy_flutter/lib/plugins/ai_chat/application/document_mention_extractor.dart` (新建)
  - 定义 `DocumentMention` 数据类（包含 documentId, documentName, startPosition, endPosition）
  - 实现 `extractDocumentIds()` 方法从文本中提取文档 ID
  - 实现 `parseMentions()` 方法解析所有 @mention
  - 实现 `findMentionAtCursor()` 方法查找光标位置的 mention
  - 使用正则表达式匹配 `@文档名称` 格式
  - 目的: 提供工具类用于解析和提取 @mention 文档信息
  - _Leverage: ChatInputControlCubit 获取文档 ID 映射_
  - _Requirements: 3, 4_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in text parsing and regex | Task: Create DocumentMentionExtractor utility class with DocumentMention data class and methods extractDocumentIds(), parseMentions(), and findMentionAtCursor() using regex to match @文档名称 format, following requirements 3 and 4. Integrate with ChatInputControlCubit to get document ID mappings | Restrictions: Handle edge cases like multiple mentions, empty text, malformed mentions gracefully. Ensure efficient regex matching | Success: All @mention patterns are correctly extracted from text, document IDs are resolved correctly, cursor position detection works accurately, no memory leaks in text processing_

- [x] 3. 启用 DesktopPromptInput 中的 @mention 功能
  - 文件: `appflowy_flutter/lib/ai/widgets/prompt_input/desktop_prompt_input.dart`
  - 取消注释并修复 `handleTextControllerChanged()` 方法中的 @mention 检测逻辑
  - 启用 `startMentionPageFromButton()` 中的 @ 符号插入逻辑
  - 实现 `_checkForAtMention()` 方法检测 @ 符号输入
  - 实现 `_handleMentionFiltering()` 方法处理实时过滤
  - 修复 `handlePageSelected()` 方法确保文档选择后正确插入文本
  - 目的: 重新启用并修复 @mention 菜单显示和文档选择功能
  - _Leverage: 现有的 `overlayController`, `inputControlCubit`_
  - _Requirements: 1, 3_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in text editing and overlay UI | Task: Re-enable and fix @mention functionality in DesktopPromptInput by uncommenting and fixing handleTextControllerChanged(), enabling startMentionPageFromButton(), implementing _checkForAtMention() and _handleMentionFiltering(), and fixing handlePageSelected() following requirements 1 and 3. Leverage existing overlayController and inputControlCubit | Restrictions: Do not break existing text input functionality, handle overlapping menus gracefully, ensure smooth keyboard navigation | Success: @mention menu appears when @ is typed, real-time filtering works, document selection inserts text correctly, menu closes properly_

- [x] 4. 更新 ChatBloc 以支持从 @mention 提取的文档 ID
  - 文件: `appflowy_flutter/lib/plugins/ai_chat/application/chat_bloc.dart`
  - 在 `sendMessage()` 方法中集成 DocumentMentionExtractor
  - 提取所有 @mention 文档 ID
  - 合并到 `selectedSourcesNotifier`
  - 确保 RAG IDs 正确传递到 Rust 后端
  - 目的: 确保 @mention 文档 ID 被正确提取和传递给后端进行 RAG 检索
  - _Leverage: DocumentMentionExtractor, selectedSourcesNotifier_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in Bloc pattern and state management | Task: Update ChatBloc to support extracting document IDs from @mentions by integrating DocumentMentionExtractor in sendMessage(), extracting all @mention document IDs, merging into selectedSourcesNotifier, and ensuring RAG IDs are correctly passed to Rust backend following requirement 4 | Restrictions: Do not break existing source selection logic, handle empty mentions gracefully, maintain proper state updates | Success: Document IDs from @mentions are correctly extracted and sent to backend, RAG IDs are properly set, existing source selection continues to work_

- [x] 5. 添加多语言翻译
  - 文件: `resources/translations/zh-CN.json` 和 `resources/translations/en-US.json`
  - 添加 "atMentionPlaceholder": "输入 @ 选择文档" / "Type @ to select document"
  - 添加 "atMentionNoDocuments": "未找到匹配文档" / "No matching documents found"
  - 添加 "atMentionSelectDocument": "选择文档" / "Select Document"
  - 添加 "atMentionDocumentSelected": "已选择 {} 个文档" / "{} documents selected"
  - 使用脚本生成 Dart 代码: `./scripts/code_generation/language_files/generate_language_files.sh`
  - 目的: 为 @mention 功能提供中英文界面文本支持
  - _Leverage: 现有的翻译文件结构_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Localization Engineer with expertise in i18n and Flutter localization | Task: Add multilingual translation keys for @mention feature including atMentionPlaceholder, atMentionNoDocuments, atMentionSelectDocument, atMentionDocumentSelected in both zh-CN.json and en-US.json following requirement 6. Run generate_language_files.sh to generate Dart code | Restrictions: Follow existing translation key naming conventions, ensure consistent translations across all supported languages, do not hardcode any text | Success: All UI text for @mention feature is translated and accessible via translation keys, Dart code is generated successfully_

- [x] 6. 更新文档选择框以同步 @mention 选择的文档
  - 文件: `appflowy_flutter/lib/plugins/ai_chat/presentation/chat_page/chat_footer.dart`
  - 监听输入框中的 @mention 变化
  - 同步到右下角的文档选择框 (`PromptInputDesktopSelectSourcesButton`)
  - 实现双向同步：从输入框删除 @mention 时从选择框移除，从选择框移除时从输入框删除
  - 目的: 确保输入框和文档选择框保持同步
  - _Leverage: PromptInputDesktopSelectSourcesButton, selectedSourcesNotifier_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in widget coordination and state synchronization | Task: Update document selection UI to sync @mention selections by listening to @mention changes in input field, syncing to bottom-right document selector (PromptInputDesktopSelectSourcesButton), implementing bidirectional sync (remove from selector when removed from input, remove from input when removed from selector) following requirement 5 | Restrictions: Prevent infinite sync loops, handle edge cases where documents are renamed or deleted, maintain performance with large document lists | Success: Input field and document selector are fully synchronized in both directions, no sync loops occur, edge cases are handled gracefully_

- [x] 7. 添加 @mention 文本高亮显示
  - 文件: `appflowy_flutter/lib/ai/widgets/prompt_input/desktop_prompt_input.dart`
  - 实现 `AtMentionTextSpanBuilder` 类继承 `SpecialTextSpanBuilder`
  - 识别并高亮显示 `@文档名称` 文本（蓝色、加粗）
  - 集成到 `PromptInputTextField` 中使用
  - 目的: 使 @mention 文本在输入框中以特殊样式显示，区别于普通文本
  - _Leverage: extended_text_field 包的 SpecialTextSpanBuilder_
  - _Requirements: 3, Usability_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in custom text rendering and styling | Task: Add @mention text highlighting by implementing AtMentionTextSpanBuilder extending SpecialTextSpanBuilder, identifying and highlighting @文档名称 text (blue, bold), integrating into PromptInputTextField following requirements 3 and Usability | Restrictions: Do not interfere with existing text formatting, ensure efficient text rendering, handle overlapping styles gracefully | Success: @mention text is visually distinct (blue and bold) in the input field, text rendering performance is maintained, existing formatting continues to work_

- [ ] 8. 添加移动端 @mention 支持（可选，仅在需求扩展时）
  - 文件: `appflowy_flutter/lib/plugins/ai_chat/presentation/chat_input/mobile_chat_input.dart`
  - 注意: 根据需求 #7，移动端不在此次范围内，此任务预留
  - 如需实现: 启用 `mentionPage()` 方法，添加 @ 符号检测，显示移动端底部菜单
  - 目的: 为未来移动端支持预留接口
  - _Leverage: 现有的 showPageSelectorSheet_
  - _Requirements: 7（明确不实现）_
  - _Prompt: This task is marked as optional and not to be implemented in current scope as per requirement 7 which states mobile platforms are NOT supported. Keep as placeholder for future expansion._

- [ ] 9. 添加单元测试
  - 文件: `appflowy_flutter/test/plugins/ai_chat/document_mention_extractor_test.dart` (新建)
  - 测试 `extractDocumentIds()` 各种文本格式
  - 测试 `parseMentions()` 多个 @mention 场景
  - 测试 `findMentionAtCursor()` 光标位置检测
  - 测试边界情况（空文本、无效格式、特殊字符）
  - 目的: 确保 DocumentMentionExtractor 的功能正确性和可靠性
  - _Leverage: Flutter test framework, existing test utilities_
  - _Requirements: All_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Flutter testing and test-driven development | Task: Add comprehensive unit tests for DocumentMentionExtractor testing extractDocumentIds() with various text formats, parseMentions() with multiple @mention scenarios, findMentionAtCursor() cursor position detection, and edge cases (empty text, invalid format, special characters) to ensure correctness and reliability following all requirements | Restrictions: Do not test implementation details, ensure tests are isolated and fast, cover both success and failure scenarios | Success: All extraction methods are thoroughly tested with good coverage, edge cases are covered, tests run quickly and reliably_

- [ ] 10. 集成测试：完整 @mention 流程
  - 文件: `appflowy_flutter/integration_test/desktop/ai_chat_at_mention_test.dart` (新建)
  - 测试场景：输入 @ → 菜单弹出 → 搜索文档 → 选择文档 → 文本插入
  - 测试场景：多个 @mention 文档选择和提交
  - 测试场景：文档选择框同步
  - 测试场景：从输入框删除 @mention 时选择框更新
  - 目的: 验证完整的 @mention 用户流程
  - _Leverage: Flutter integration test framework_
  - _Requirements: All_
  - _Prompt: Implement the task for spec ai-chat-at-mention-documents, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer with expertise in Flutter integration testing | Task: Create comprehensive integration tests for complete @mention flow including scenarios for: typing @, menu popup, document search, document selection, text insertion; multiple @mention document selection and submission; document selector synchronization; input field deletion syncing with selector, following all requirements | Restrictions: Tests must simulate real user interactions, handle async operations properly, ensure test isolation | Success: All user workflows are tested and pass, tests can run repeatedly without flakiness, comprehensive coverage of all interaction paths_
