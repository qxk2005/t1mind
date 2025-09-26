# MCP工具AI自动选择功能改进

## 问题诊断

之前的实现存在以下问题：

1. **工具Schema信息不完整**：`inputSchema` 被设置为空对象 `{}`，导致AI无法了解工具的参数信息
2. **缺少调试信息**：无法知道AI实际看到了什么工具信息
3. **后备逻辑过于简单**：总是选择第一个工具

## 已实施的改进

### 1. 完整传递工具Schema信息

```dart
inputSchema: {
  'type': 'object',
  'properties': {
    for (final field in tool.fields)
      field.name: {
        'type': field.type,
        'description': field.name,
        if (field.defaultValue != null) 'default': field.defaultValue,
      }
  },
  'required': tool.fields.where((f) => f.required).map((f) => f.name).toList(),
},
```

现在AI可以看到：
- 每个工具的完整参数列表
- 参数类型（string, number等）
- 哪些参数是必需的
- 参数的默认值

### 2. 增强的调试日志

```dart
Log.info('开始为端点 $endpointId 选择工具，可用工具数量: ${availableTools.length}');
Log.info('可用工具: ${tool.name} - ${tool.description}');
Log.info('生成的AI提示词长度: ${prompt.length} 字符');
Log.debug('AI提示词内容:\n$prompt');
Log.info('AI返回结果长度: ${aiResult.length} 字符');
Log.info('AI选择了工具: ${selection.toolName}');
Log.info('选择理由: ${selection.reason}');
```

### 3. 改进的后备选择逻辑

使用评分系统替代简单的"选第一个"：

```dart
// 计算关键词匹配得分
if (queryLower.contains('读取') || queryLower.contains('read')) {
  if (nameLower.contains('read')) score += 5;
  if (descLower.contains('读取')) score += 3;
}

// 检查查询中的其他关键词
for (final word in queryWords) {
  if (nameLower.contains(word)) score += 2;
  if (descLower.contains(word)) score += 1;
}
```

## 示例场景

### 用户查询："读取Excel文件中的数据"

#### 可用工具（假设）：
1. `excel_reader` - "读取Excel文件内容"
2. `excel_writer` - "写入数据到Excel文件"
3. `excel_formatter` - "格式化Excel文件"

#### AI将看到的完整信息：
```json
{
  "name": "excel_reader",
  "description": "读取Excel文件内容",
  "inputSchema": {
    "type": "object",
    "properties": {
      "filepath": {
        "type": "string",
        "description": "filepath"
      },
      "sheet_name": {
        "type": "string",
        "description": "sheet_name",
        "default": "Sheet1"
      }
    },
    "required": ["filepath"]
  },
  "type": "function",
  "required": ["filepath"]
}
```

#### 预期结果：
- **选择的工具**：`excel_reader`
- **选择理由**：用户明确需要读取Excel文件中的数据，excel_reader工具专门用于读取Excel文件内容，具有filepath参数用于指定文件路径，是最适合完成此任务的工具
- **执行目标**：使用excel_reader工具读取指定的Excel文件，提取其中的数据内容

### 后备逻辑评分示例：

如果AI服务失败，后备逻辑会这样评分：
- `excel_reader`：得分8分（名称包含"read"+5, 描述包含"读取"+3）
- `excel_writer`：得分0分（不匹配读取操作）
- `excel_formatter`：得分0分（不匹配读取操作）

选择：`excel_reader`（最高分）

## 验证方法

1. 查看日志输出，确认：
   - 所有工具信息都被列出
   - AI提示词包含完整的工具schema
   - AI的选择理由合理

2. 测试不同查询：
   - 读取操作
   - 写入操作
   - 查询操作
   - 模糊查询

3. 故意让AI服务失败，验证后备逻辑是否正确选择工具

## 总结

通过这些改进，MCP工具的AI自动选择功能现在：
1. 向AI提供了完整的工具信息（名称、描述、参数schema）
2. 有详细的日志帮助调试
3. 即使AI服务失败，也有智能的后备选择逻辑
4. 不再默认选择第一个工具，而是基于相关性选择最合适的工具
