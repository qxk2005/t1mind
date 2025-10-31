# Requirements Document

## Introduction

本规范定义了在 AppFlowy AI 聊天功能中，通过 `@` 符号选择文档作为 RAG（检索增强生成）上下文参考的特性。该功能允许用户在 AI 聊天输入框中输入 `@` 符号后，实时搜索和选择本地文档，自动将这些选中的文档作为 RAG 上下文提供给 AI 模型。

此功能旨在提升用户与 AI 交互的效率，使用户能够更精确地指定 AI 应该参考哪些文档来回答问题。

## Alignment with Product Vision

该功能支持 AppFlowy 的核心价值主张：提供智能、高效的工作空间体验。通过增强 AI 聊天功能，用户可以更精确地利用本地文档内容，提高 AI 回答的准确性和相关性。

## Requirements

### Requirement 1: @ 符号触发文档选择

**User Story:** 作为一名 AppFlowy 用户，我期望在 AI 聊天输入框中输入 `@` 符号时能够弹出本地文档列表，以便我可以快速选择要作为上下文的文档。

#### Acceptance Criteria

1. WHEN 用户在 AI 聊天输入框中输入 `@` 符号 THEN 系统 SHALL 自动弹出本地文档名称列表
2. WHEN 输入框获得焦点 AND 用户输入 `@` 符号 THEN 系统 SHALL 显示模糊搜索的文档列表
3. IF 用户点击输入框外部 OR 按下 Escape 键 THEN 系统 SHALL 关闭文档选择菜单

### Requirement 2: 实时模糊搜索文档

**User Story:** 作为一名 AppFlowy 用户，我期望在输入 `@` 符号后继续输入文字时，能够通过模糊匹配过滤文档列表，以便快速找到目标文档。

#### Acceptance Criteria

1. WHEN 用户在 `@` 符号后继续输入文字 THEN 系统 SHALL 实时根据输入内容进行模糊匹配过滤文档列表
2. WHEN 模糊匹配执行时 THEN 系统 SHALL 不区分大小写进行搜索
3. WHEN 匹配到文档名称时 THEN 系统 SHALL 高亮显示匹配部分
4. IF 没有匹配到任何文档 THEN 系统 SHALL 显示 "未找到匹配文档" 的消息

### Requirement 3: 文档选择与显示

**User Story:** 作为一名 AppFlowy 用户，我期望在选择文档后，输入框中能够显示完整的 `@文档名称`，以便我可以确认已选择的文档。

#### Acceptance Criteria

1. WHEN 用户从列表中选择一个文档 THEN 系统 SHALL 在输入框中显示完整的 `@文档名称` 格式
2. WHEN 文档被选择 THEN 系统 SHALL 关闭文档选择菜单
3. WHEN 文档被选择 THEN 系统 SHALL 自动将光标定位到文档名称末尾，允许用户继续输入
4. IF 用户选择了多个文档 THEN 系统 SHALL 在输入框中显示多个 `@文档名称`，以空格或逗号分隔

### Requirement 4: RAG 上下文自动提取

**User Story:** 作为一名 AppFlowy 用户，我期望在提交聊天问题时，系统能够自动使用所有 `@文档名称` 标记的文档作为 RAG 上下文，以便 AI 能够基于这些文档内容回答问题。

#### Acceptance Criteria

1. WHEN 用户提交聊天消息 AND 输入框中有 `@文档名称` 标记 THEN 系统 SHALL 自动提取所有被标记的文档 ID
2. WHEN 提取文档 ID 后 THEN 系统 SHALL 将这些文档作为 RAG 上下文提供给 AI 模型
3. WHEN 使用 RAG 上下文时 THEN 系统 SHALL 在系统提示词中包含文档内容
4. IF 标记的文档不存在或无法访问 THEN 系统 SHALL 显示错误消息并询问用户是否继续发送消息

### Requirement 5: 文档选择框同步更新

**User Story:** 作为一名 AppFlowy 用户，我期望在输入框中选择 `@文档名称` 后，输入框右下角的文档选择框能够自动同步显示这些选中的文档选项。

#### Acceptance Criteria

1. WHEN 用户在输入框中选择 `@文档名称` THEN 系统 SHALL 在输入框右下角的文档选择框中自动添加该文档选项
2. WHEN 输入框右下角的文档选择框显示文档时 THEN 系统 SHALL 允许用户点击移除该文档
3. WHEN 用户在文档选择框中移除文档时 THEN 系统 SHALL 同时在输入框中移除对应的 `@文档名称` 标记（如果存在）
4. IF 在文档选择框中手动添加文档时 THEN 系统 SHALL 同时在输入框中添加 `@文档名称` 标记（如果不存在）

### Requirement 6: 多语言支持

**User Story:** 作为一名使用不同语言的 AppFlowy 用户，我期望能够在我的语言环境下使用 @ 文档选择功能。

#### Acceptance Criteria

1. WHEN 用户界面设置为简体中文 THEN 系统 SHALL 显示中文界面文本（如 "选择文档"、"未找到匹配文档" 等）
2. WHEN 用户界面设置为英文 THEN 系统 SHALL 显示英文界面文本（如 "Select Document"、"No matching documents found" 等）
3. WHEN 使用文档搜索功能 THEN 系统 SHALL 支持搜索中文和英文文档名称
4. IF 用户界面为其他语言 THEN 系统 SHALL 回退到英文界面

### Requirement 7: 跨平台支持

**User Story:** 作为一名在不同平台上使用 AppFlowy 的用户，我期望能够在 macOS 和 Windows 上使用 @ 文档选择功能。

#### Acceptance Criteria

1. WHEN 用户在 macOS 上使用 AI 聊天功能 THEN 系统 SHALL 完全支持 @ 文档选择功能
2. WHEN 用户在 Windows 上使用 AI 聊天功能 THEN 系统 SHALL 完全支持 @ 文档选择功能
3. IF 用户在移动平台上 THEN 系统 SHALL 不支持此功能（按照需求说明）

### Requirement 8: 设置界面支持

**User Story:** 作为一名使用不同 AI 配置的用户，我期望在本地端设置和服务器端设置界面中都能够使用 @ 文档选择功能。

#### Acceptance Criteria

1. WHEN 用户在本地端设置界面中使用 AI 聊天 THEN 系统 SHALL 支持 @ 文档选择功能
2. WHEN 用户在服务器端设置界面中使用 AI 聊天 THEN 系统 SHALL 支持 @ 文档选择功能
3. WHEN 使用本地 AI（如 Ollama）时 THEN 系统 SHALL 在本地向量数据库中进行文档检索
4. WHEN 使用 OpenAI 兼容服务器时 THEN 系统 SHALL 使用向量数据库进行文档检索并添加上下文

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: 文档选择功能应独立模块化，不影响现有的 AI 聊天架构
- **Modular Design**: 创建一个独立的 `DocumentMentionHandler` 类来处理 @ 符号触发和文档选择逻辑
- **Dependency Management**: 最小化与现有 AI 聊天模块的依赖关系
- **Clear Interfaces**: 定义清晰的接口用于文档搜索、选择和数据同步

### Performance
- 文档搜索响应时间应小于 100ms（即使在有大量文档的情况下）
- 模糊匹配算法应使用高效的字符串匹配算法（如 KMP 或 Boyer-Moore）
- 文档列表渲染应支持虚拟滚动，以处理大量文档
- 在选择文档时，UI 应无延迟地更新

### Security
- 用户的文档选择信息不应暴露给未授权的第三方
- 文档 ID 验证应确保只有用户有权访问的文档才能被选择
- RAG 上下文的文档内容传输应使用安全的通道（本地或加密的连接）

### Reliability
- 当文档数据库不可用时，系统应优雅地处理错误并提示用户
- 如果文档被删除或不可访问，系统应自动从选择列表中移除该文档
- 在提取文档上下文时，如果某个文档提取失败，系统应继续处理其他文档并通知用户

### Usability
- @ 符号触发应该容易学习和记忆，符合直觉
- 文档列表应清晰显示文档名称和所属工作区
- 选择的文档应在输入框中以特殊样式显示，使与其他文本区分开来
- 输入框应提供可视化的指示器，显示当前有多少文档被选择
- 文档搜索应支持拼音匹配（针对中文用户）

