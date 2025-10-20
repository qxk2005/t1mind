# 向量数据库重置功能说明

## 🎯 功能概述

向量数据库重置功能允许用户在嵌入模型维度发生变化时，安全地重置向量数据库以匹配新的维度。这个功能解决了嵌入模型维度不匹配的问题，如：

- OpenAI `text-embedding-3-small` (1536维) → `text-embedding-3-large` (3072维)
- Ollama `nomic-embed-text` (768维) → 其他模型
- 自定义嵌入服务维度变更

## 🔧 技术实现

### 后端功能

#### 1. 向量数据库层 (`VectorSqliteDB`)
- **`reset_vector_database()`**: 清空现有数据，保持表结构
- **`rebuild_vector_database(dimension)`**: 完全重建表结构，支持动态维度

#### 2. 嵌入上下文层 (`EmbedContext`)
- **`reset_vector_database()`**: 简单重置接口
- **`rebuild_vector_database(dimension)`**: 智能重建接口
- **`get_current_embedding_dimension()`**: 获取当前模型维度

#### 3. AI管理器层 (`AIManager`)
- **`reset_vector_database()`**: 基础重置
- **`smart_reset_vector_database()`**: 智能重置（自动检测维度）

### 前端功能

#### 1. 状态管理 (`SettingsAIBloc`)
- 添加了 `resetVectorDatabase` 事件
- 添加了 `isResettingVectorDB` 状态

#### 2. UI组件 (`_VectorDatabaseResetSection`)
- 独立的红色警告区块
- 双重确认机制（按钮 + 文字输入）
- 重置过程中的加载状态

## 📏 支持的嵌入模型维度

### OpenAI 模型
- `text-embedding-3-small`: 1536维
- `text-embedding-3-large`: 3072维
- `text-embedding-ada-002`: 1536维
- `text-embedding-002`: 1536维
- `text-similarity-davinci-001`: 12288维
- `text-similarity-curie-001`: 12288维
- `text-similarity-babbage-001`: 2048维
- `text-similarity-ada-001`: 1024维

### Ollama 模型
- `nomic-embed-text`: 2560维（当前默认）

## 🚀 使用流程

### 1. 触发重置
- 进入 **设置** → **AI设置** 页面
- 滚动到页面底部
- 看到红色警告区块："危险操作：重置向量数据库"

### 2. 第一次确认
- 点击 **"重置向量数据库"** 按钮
- 系统显示确认对话框

### 3. 第二次确认
- 在输入框中输入 **"我确认"**
- 点击 **"确认重置"** 按钮

### 4. 执行重置
- 系统自动检测当前嵌入模型维度
- 重建向量数据库表结构
- 清空所有现有数据
- 显示重置完成状态

## ⚠️ 安全特性

### 1. 视觉警告
- 红色主题和警告图标
- 醒目的危险操作标识
- 详细的操作后果说明

### 2. 双重确认
- 按钮点击确认
- 文字输入确认（"我确认"）
- 防止误操作

### 3. 位置隔离
- 放在页面最底部
- 独立区块设计
- 与常规设置分离

### 4. 状态反馈
- 重置过程中的加载动画
- 按钮禁用状态
- 清晰的进度提示

## 🔄 智能重置特性

### 自动维度检测
系统会自动检测当前配置的嵌入模型维度：

1. **OpenAI 兼容服务**: 根据模型名称确定维度
2. **Ollama 服务**: 使用默认维度
3. **未知模型**: 使用安全默认值

### 动态表重建
- 删除旧的向量表
- 使用新维度创建新表
- 保持其他表结构不变

## 📝 日志记录

重置过程会记录详细的日志：

```
[AI Manager] 🔄 开始智能重置向量数据库...
[AI Manager] 📏 当前嵌入模型维度: 1536
[Vector DB] 🔄 开始重建向量数据库，新维度: 1536
[Vector DB] ✅ 向量数据库重建完成，新维度: 1536
[AI Manager] ✅ 向量数据库智能重置完成，新维度: 1536
```

## 🎯 适用场景

### 1. 模型升级
- 从低维度模型升级到高维度模型
- 从高维度模型降级到低维度模型

### 2. 服务切换
- 从 Ollama 切换到 OpenAI
- 从 OpenAI 切换到自定义服务
- 不同嵌入服务间的切换

### 3. 配置错误修复
- 维度配置错误导致的搜索失败
- 嵌入服务配置变更

## 🔧 技术细节

### 数据库操作
```sql
-- 删除旧表
DROP TABLE IF EXISTS af_collab_embeddings;

-- 创建新表（动态维度）
CREATE VIRTUAL TABLE af_collab_embeddings 
USING vec0(
  workspace_id    TEXT    NOT NULL,
  object_id       TEXT    NOT NULL,
  fragment_id     TEXT    NOT NULL,
  content_type    INTEGER NOT NULL,
  content         TEXT    NOT NULL,
  metadata        TEXT,
  fragment_index  INTEGER NOT NULL DEFAULT 0,
  embedder_type   INTEGER NOT NULL DEFAULT 0,
  embedding       float[1536]  -- 动态维度
);
```

### 错误处理
- 数据库连接失败
- 表创建失败
- 权限不足
- 磁盘空间不足

## 🚨 注意事项

1. **数据丢失**: 重置会永久删除所有嵌入数据
2. **重新索引**: 重置后需要重新索引所有文档
3. **性能影响**: 重建过程可能需要一些时间
4. **备份建议**: 重要数据建议提前备份

## 🔮 未来改进

1. **增量迁移**: 支持部分数据迁移
2. **维度验证**: 自动验证嵌入维度
3. **批量操作**: 支持批量重置多个工作区
4. **进度显示**: 显示重建进度百分比
5. **回滚功能**: 支持重置操作回滚
