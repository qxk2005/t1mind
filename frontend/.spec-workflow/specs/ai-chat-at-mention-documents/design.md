# Design Document

## Overview

本设计文档描述了在 AppFlowy AI 聊天功能中实现 @ 符号选择文档作为 RAG 上下文的技术实现方案。该功能基于现有的文档提及机制，通过增强 `ChatInputControlCubit`、重启用已有的 `PromptInputMentionPageMenu` 组件，以及修改文本处理逻辑，实现类似 Notion 的 @mention 体验。

该功能将无缝集成到现有的 AI 聊天架构中，支持所有 AI 类型（本地 AI、OpenAI 兼容服务器、AppFlowy Cloud），并自动利用现有的 RAG 文档检索机制。

## Steering Document Alignment

### Technical Standards (tech.md)

遵循 AppFlowy 的技术标准：
- **Bloc/Cubit 模式**: 使用 Cubit 管理状态，保持与现有 AI 聊天模块一致
- **跨平台兼容**: 支持 macOS 和 Windows，不涉及移动端特定功能
- **前后端分离**: Flutter 前端负责 UI 和用户交互，Rust 后端负责文档检索和 RAG 上下文处理
- **模块化设计**: 最小化对现有代码的影响，创建可复用的组件

### Project Structure (structure.md)

遵循 AppFlowy 的项目结构：
- **UI 组件**: 位于 `appflowy_flutter/lib/ai/widgets/prompt_input/` 和 `appflowy_flutter/lib/plugins/ai_chat/presentation/`
- **状态管理**: 位于 `appflowy_flutter/lib/plugins/ai_chat/application/`
- **后端逻辑**: 位于 `rust-lib/flowy-ai/src/`
- **多语言**: 使用 `resources/translations/` 中的翻译文件

## Code Reuse Analysis

### Existing Components to Leverage

- **ChatInputControlCubit**: 已实现文档搜索和过滤逻辑，位于 `appflowy_flutter/lib/plugins/ai_chat/application/chat_input_control_cubit.dart`
  - 需要启用并增强文本监听逻辑以支持 @ 符号触发
  - 需要改进文档选择后的文本插入逻辑

- **PromptInputMentionPageMenu**: 已存在文档选择菜单 UI 组件，位于 `appflowy_flutter/lib/ai/widgets/prompt_input/mention_page_menu.dart`
  - 需要重启用（当前部分代码被注释）
  - 需要确保模糊匹配逻辑正常工作

- **DesktopPromptInput**: AI 聊天输入框组件，位于 `appflowy_flutter/lib/ai/widgets/prompt_input/desktop_prompt_input.dart`
  - 需要启用 overlay controller 和 mention 触发逻辑

- **RAG 文档检索机制**: 后端已完全支持，位于 `rust-lib/flowy-ai/src/local_ai/chat/`
  - 无需修改，自动利用现有的 `set_rag_ids` 和文档检索功能

- **Sources Manager**: 文档源管理组件，位于 `appflowy_flutter/lib/plugins/ai_chat/application/sources_manager.dart`
  - 需要扩展以支持从 @mention 提取的文档 ID

### Integration Points

- **AIPromptInputBloc**: 与现有的 AI 提示输入 Bloc 集成，同步 @ 文档选择
- **ChatBloc**: 与聊天 Bloc 集成，确保文档选择信息正确传递给后端
- **DocumentIndexer**: 利用现有的向量数据库和文档索引系统
- **Database Layer**: 使用现有的 RAG IDs 存储和检索机制

## Architecture

### Modular Design Principles

- **Single File Responsibility**: 每个文件专注于特定功能
  - `ChatInputControlCubit`: 仅处理 @mention 触发和文档搜索
  - `DocumentMentionHandler`: 专门处理 @文档名称 的解析和提取
  - `PromptInputMentionPageMenu`: 仅负责 UI 显示和用户交互

- **Component Isolation**: 
  - @mention 功能作为独立的特性模块，不影响现有的文档选择器
  - 与现有的搜索源功能（网络搜索、文档）并行工作

- **Service Layer Separation**:
  - 前端：UI 和用户交互逻辑
  - Bloc/Cubit：状态管理和事件处理
  - 后端：文档检索和 RAG 上下文注入

- **Utility Modularity**:
  - 创建 `@mention_parser.dart` 用于解析 @文档名称 格式
  - 创建 `document_mention_extractor.dart` 用于提取文档 ID

### Architecture Diagram

```mermaid
graph TD
    A[用户输入@符号] --> B[ChatInputControlCubit]
    B --> C[启用Overlay Menu]
    C --> D[PromptInputMentionPageMenu]
    D --> E[文档列表模糊搜索]
    E --> F[用户选择文档]
    F --> G[插入@文档名称到输入框]
    G --> H[同步到DocumentMentionExtractor]
    H --> I[用户提交消息]
    I --> J[提取所有@文档ID]
    J --> K[ChatBloc]
    K --> L[设置RAG IDs到后端]
    L --> M[Rust后端检索文档]
    M --> N[AI使用文档上下文回答]
    
    H --> O[右下角文档选择框更新]
```

## Components and Interfaces

### Component 1: ChatInputControlCubit Enhancement

**Purpose**: 增强现有的 ChatInputControlCubit 以支持 @ 符号触发和实时文本监听

**Interfaces**:
- `void enableAtMentionMode()`: 启用 @mention 模式
- `void disableAtMentionMode()`: 禁用 @mention 模式
- `bool isAtMentionActive`: 检查当前是否处于 @mention 模式
- `void handleTextChange(String text, int caretOffset)`: 处理文本变化，检测 @ 符号
- `void startDocumentSearch(String filter)`: 开始文档搜索

**Dependencies**: 
- `ViewBackendService`: 获取文档列表
- `TextEditingController`: 监听文本变化

**Reuses**: 
- 现有的 `refreshViews()`, `updateFilter()` 方法
- 现有的模糊匹配逻辑

**Modifications Needed**:
```dart
// appflowy_flutter/lib/plugins/ai_chat/application/chat_input_control_cubit.dart

// 添加状态跟踪
bool _isAtMentionMode = false;
int _atSymbolPosition = -1;

// 启用 @mention 模式检测
void enableMentionTracking(TextEditingController controller) {
  controller.addListener(_onTextChanged);
}

void _onTextChanged() {
  final text = controller.text;
  final cursorPosition = controller.selection.baseOffset;
  
  // 检测 @ 符号
  if (text.length > 0 && cursorPosition > 0) {
    final charBeforeCursor = text[cursorPosition - 1];
    if (charBeforeCursor == '@') {
      _startAtMention(text, cursorPosition);
    } else if (_isAtMentionActive) {
      _continueAtMention(text, cursorPosition);
    }
  }
}
```

### Component 2: DocumentMentionExtractor

**Purpose**: 新建工具类，用于从输入文本中提取 @文档名称 并转换为文档 ID

**Interfaces**:
- `List<String> extractDocumentIds(String text)`: 提取所有 @文档名称 对应的文档 ID
- `List<DocumentMention> parseMentions(String text)`: 解析文本中的所有 @mention
- `DocumentMention? findMentionAtCursor(String text, int cursorPosition)`: 查找光标位置的 mention

**Dependencies**: 
- `ChatInputControlCubit`: 获取文档 ID 映射
- `ViewBackendService`: 验证文档是否存在

**Structure**:
```dart
// appflowy_flutter/lib/plugins/ai_chat/application/document_mention_extractor.dart

class DocumentMention {
  final String documentId;
  final String documentName;
  final int startPosition;
  final int endPosition;
}

class DocumentMentionExtractor {
  final ChatInputControlCubit _controlCubit;
  
  /// 从输入文本中提取所有文档 ID
  List<String> extractDocumentIds(String text) {
    // 使用正则表达式匹配 @文档名称 格式
    final pattern = RegExp(r'@(\S+)');
    final matches = pattern.allMatches(text);
    
    return matches.map((match) {
      final documentName = match.group(1)!;
      // 从 ChatInputControlCubit 获取文档 ID
      return _controlCubit.getDocumentIdByName(documentName);
    }).where((id) => id != null).cast<String>().toList();
  }
  
  /// 解析输入文本中的所有 mentions
  List<DocumentMention> parseMentions(String text) {
    // 实现 mention 解析逻辑
  }
}
```

### Component 3: TextController Enhancement

**Purpose**: 增强 AiPromptInputTextEditingController 以支持 @mention 检测和特殊文本渲染

**Interfaces**:
- 监听器自动触发 @mention 菜单
- 使用 `TextSpan` 渲染 @文档名称 为特殊样式（蓝色、加粗）

**Dependencies**:
- `ChatInputControlCubit`: 协调 mention 状态
- `extended_text_field` 包: 支持自定义文本渲染

**Modifications Needed**:
```dart
// appflowy_flutter/lib/plugins/ai_chat/application/ai_prompt_input_text_editing_controller.dart

class AtMentionTextSpanBuilder extends SpecialTextSpanBuilder {
  @override
  SpecialText? createSpecialText(String flag, TextStyle? textStyle, {
    TextEditingController? controller,
  }) {
    if (flag == '@') {
      // 匹配 @文档名称 模式
      return AtMentionText(
        text: flag,
        textStyle: textStyle,
        controller: controller,
      );
    }
    return null;
  }
}

class AtMentionText extends SpecialText {
  @override
  InlineSpan finishText() {
    return TextSpan(
      text: fullText,
      style: TextStyle(
        color: Colors.blue,
        fontWeight: FontWeight.w600,
      ),
    );
  }
}
```

### Component 4: DesktopPromptInput Reactivation

**Purpose**: 重启用 desktop_prompt_input.dart 中被注释掉的 @mention 功能

**Modifications**:
```dart
// appflowy_flutter/lib/ai/widgets/prompt_input/desktop_prompt_input.dart

void handleTextControllerChanged() {
  setState(() {
    updateSendButtonState();
    isComposing = !widget.textController.value.composing.isCollapsed;
  });

  if (isComposing) {
    return;
  }

  // ✅ 启用 @mention 检测
  final textController = widget.textController;
  final textSelection = textController.value.selection;
  final text = textController.text;
  
  // 检测是否需要打开 @mention 菜单
  if (overlayController.isShowing) {
    // 已打开，继续处理过滤
    if (inputControlCubit.filterStartPosition != -1) {
      _handleMentionFiltering(text, textSelection);
    }
  } else {
    // 未打开，检查是否需要打开
    _checkForAtMention(text, textSelection);
  }
}

void _checkForAtMention(String text, TextSelection selection) {
  final cursorPos = selection.baseOffset;
  if (cursorPos > 0) {
    final charBeforeCursor = text[cursorPos - 1];
    if (charBeforeCursor == '@') {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        inputControlCubit.startSearching(widget.textController.value);
        overlayController.show();
      });
    }
  }
}
```

### Component 5: Integration with ChatBloc

**Purpose**: 确保 @mention 选中的文档 ID 正确传递给后端进行 RAG 检索

**Modifications**:
```dart
// appflowy_flutter/lib/plugins/ai_chat/application/chat_bloc.dart

// 在发送消息时提取 @mention 文档 ID
void sendMessage(String message, ...) {
  // 提取 @mention 文档 ID
  final documentIds = DocumentMentionExtractor()
      .extractDocumentIds(message);
  
  // 合并到 selectedSourcesNotifier
  if (documentIds.isNotEmpty) {
    final currentSources = selectedSourcesNotifier.value;
    final newSources = [...currentSources, ...documentIds].toSet().toList();
    selectedSourcesNotifier.value = newSources;
  }
  
  // 发送事件到 Rust 后端
  add(ChatEvent.sendMessage(
    message: message,
    ragIds: documentIds,
    // ...
  ));
}
```

### Component 6: Backend Integration (Rust)

**Purpose**: 后端无需修改，自动利用现有的 RAG 机制

**Current Flow**:
- Rust 后端在 `flowy-ai/src/ai_manager.rs` 中已有 `set_rag_ids()` 调用
- 文档检索在 `flowy-ai/src/local_ai/chat/mod.rs` 中实现
- OpenAI 兼容模式在 `flowy-ai/src/middleware/chat_service_mw.rs` 中处理 RAG 文档

**What We Leverage**:
- 现有的 `set_rag_ids()` 机制
- 现有的文档向量检索
- 现有的 RAG 上下文注入到 system prompt

## Data Models

### DocumentMention

```dart
class DocumentMention {
  final String documentId;       // 文档唯一 ID
  final String documentName;      // 文档显示名称
  final int startPosition;        // 在文本中的起始位置
  final int endPosition;          // 在文本中的结束位置
}

extension DocumentMentionExtension on DocumentMention {
  String get mentionText => '@$documentName';
}
```

### ChatInputControlState Enhancement

```dart
// 在 chat_input_control_cubit.freezed.dart 中添加新状态

@freezed
class ChatInputControlState with _$ChatInputControlState {
  const factory ChatInputControlState.loading() = _Loading;
  
  const factory ChatInputControlState.ready({
    required List<ViewPB> visibleViews,
    required int focusedViewIndex,
    // ✅ 新增字段
    @Default(false) bool isAtMentionActive,
    @Default(-1) int atSymbolPosition,
    required List<String> selectedDocumentIds,
  }) = _Ready;
}

// 添加新方法到 ChatInputControlCubit
String? getDocumentIdByName(String name);
Map<String, ViewPB> getDocumentIdMap();
void setDocumentIdMapping(List<ViewPB> views);
```

### RAG Document Metadata

```rust
// rust-lib/flowy-ai/src/ 后端已支持，无需修改

struct RAGDocument {
    object_id: String,      // 文档 ID
    page_content: String,    // 文档内容
    metadata: HashMap<String, String>, // 元数据
    score: f32,              // 相似度分数
}
```

## Error Handling

### Error Scenarios

1. **文档不存在或被删除**: 当用户选择的文档在提交时已不存在
   - **Handling**: 自动从 mention 列表中移除，通知用户
   - **User Impact**: 显示提示信息 "文档 [名称] 不再可用，已从引用中移除"

2. **@mention 格式解析失败**: 文本中的 @文档名称 无法解析
   - **Handling**: 记录警告日志，使用原始文本
   - **User Impact**: 消息正常发送，但不包含该文档上下文

3. **文档检索超时**: 向量数据库查询超时
   - **Handling**: 跳过该文档，继续处理其他文档
   - **User Impact**: 显示提示 "部分文档检索超时，AI 可能使用较少上下文"

4. **输入框焦点丢失**: 用户点击其他地方导致 @mention 菜单消失
   - **Handling**: 优雅关闭菜单，保留已输入内容
   - **User Impact**: 可以继续输入或重新触发 @mention

## Testing Strategy

### Unit Testing

- **DocumentMentionExtractor**: 
  - 测试从各种格式的文本中提取 @mention
  - 测试边界情况（多个 @mention、嵌套 @、特殊字符）
  
- **ChatInputControlCubit**:
  - 测试 @ 符号检测
  - 测试模糊匹配逻辑
  - 测试文档选择状态更新

### Integration Testing

- **@mention 触发流程**:
  - 输入 @ 符号 → 菜单弹出 → 输入过滤文字 → 选择文档 → 文本插入
  
- **文档同步流程**:
  - @mention 选择 → 文档选择框更新 → 提交消息 → RAG IDs 传递 → 后端检索

### End-to-End Testing

- **完整场景 1**: 用户输入 @，搜索并选择文档，提交问题，AI 使用文档内容回答
- **完整场景 2**: 用户输入多个 @mention，所有文档都被正确索引和使用
- **完整场景 3**: 用户在输入框选择文档，在文档选择框移除文档，两者同步更新

