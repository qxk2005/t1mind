# 修复后快速测试指南

## 🎯 测试目标

验证 UTF-8 字符边界修复后，应用能正常处理包含中文的工具结果。

## ⚡ 快速测试步骤

### 1. 重新编译应用

```bash
cd rust-lib/flowy-ai
cargo build
```

### 2. 启动应用

重启应用并打开聊天界面。

### 3. 测试工具调用（触发原问题）

输入以下问题（这会触发 Readwise MCP 工具调用，返回大量中文内容）：

```
推荐几个 readwise 中的跟禅宗相关的书籍
```

### 4. 验证成功标志

#### ✅ 成功的表现

1. **应用不崩溃** ⭐ 最重要
2. **工具成功执行**
   - 看到工具调用 JSON
   - 看到工具结果显示
3. **日志正常输出**
   ```
   🔧 [TOOL EXEC] ✅ Tool call SUCCEEDED
   🔧 [TOOL EXEC]   Result preview: [正常显示的中文文本]...
   ```
4. **看到分隔符** `---`
5. **AI 生成最终回答**（多轮对话功能）
   - AI 基于工具结果用中文总结
   - 推荐具体的书籍

#### ❌ 失败的表现

1. **应用崩溃**
   ```
   thread panicked at ... byte index ... is not a char boundary
   ```
2. **日志输出异常**
3. **没有看到工具结果**

## 🔍 详细验证

### 检查日志输出

在应用日志中应该看到类似：

```
INFO  flowy_ai::agent::tool_call_handler  🔧 [TOOL EXEC] ✅ Tool call SUCCEEDED
INFO  flowy_ai::agent::tool_call_handler  🔧 [TOOL EXEC]   Duration: 620ms
INFO  flowy_ai::agent::tool_call_handler  🔧 [TOOL EXEC]   Result size: 35870 chars
INFO  flowy_ai::agent::tool_call_handler  🔧 [TOOL EXEC]   Result preview: [{"id":713666031...
```

**关键点**：
- 不会看到 panic 信息
- 预览文本能正常显示（即使包含中文字符）
- 结果大小正确显示

### 验证多轮对话

继续观察，应该看到：

```
INFO  flowy_ai::chat  🔧 [MULTI-TURN] Detected 1 tool call(s), initiating follow-up AI response
INFO  flowy_ai::chat  🔧 [MULTI-TURN] Calling AI with follow-up context (XXX chars)
INFO  flowy_ai::chat  🔧 [MULTI-TURN] Follow-up stream started
INFO  flowy_ai::chat  🔧 [MULTI-TURN] Follow-up response completed
```

### UI 验证

在聊天界面应该看到：

1. **第一部分**：AI 说明要使用工具
   ```
   Okay, I will use the search_readwise_highlights tool...
   ```

2. **工具调用**：显示 Tool_call JSON

3. **工具结果**：
   ```
   <tool_result>
   工具执行成功：search_readwise_highlights
   结果：[JSON数据]
   </tool_result>
   ```

4. **分隔符**：
   ```
   ---
   ```

5. **AI 最终回答**：
   ```
   根据搜索结果，我为您推荐以下几本与禅宗相关的书籍：
   
   1. **《The Way of Zen》** by Alan Watts
      ...
   
   2. **《洞见：从科学到哲学，打开人类的认知真相》**
      ...
   ```

## 🧪 额外测试用例

### 测试用例 1: Excel 操作（如果配置了 Excel MCP）

```
读取 test.xlsx 的内容
```

预期：能正常处理包含中文的 Excel 数据。

### 测试用例 2: 长结果

```
搜索 readwise 中所有关于"心理学"的书籍
```

预期：即使结果很长（超过 35KB），也能正常处理。

### 测试用例 3: 特殊字符

```
搜索包含表情符号 😊 和特殊符号 ✨ 的内容
```

预期：正确处理 4 字节的 UTF-8 字符（表情符号）。

## 📊 性能检查

正常情况下：
- 工具执行时间：< 2 秒
- 多轮对话启动：< 1 秒
- 总响应时间：< 5 秒

如果超时或卡住，查看日志找出瓶颈。

## 🐛 如果还有问题

### 问题 1: 仍然崩溃

**检查**：
```bash
# 查看是否还有其他字符串切割的地方
cd rust-lib
grep -r "\[\.\..*\]" . --include="*.rs" | grep -v "target" | grep -v ".."
```

**解决**：
- 找到所有硬编码索引的位置
- 应用相同的修复模式

### 问题 2: 日志乱码

**原因**：可能是终端编码问题，不是应用问题

**检查**：
```bash
# 确认终端使用 UTF-8 编码
echo $LANG
# 应该输出类似: zh_CN.UTF-8 或 en_US.UTF-8
```

### 问题 3: 工具结果不完整

**原因**：可能是其他问题，不是字符边界问题

**检查**：
- MCP 服务器是否正常
- 网络连接是否稳定
- API 配额是否充足

## ✅ 测试完成清单

- [ ] 应用成功启动
- [ ] 能发起工具调用
- [ ] 工具执行成功
- [ ] 应用没有崩溃
- [ ] 日志正常输出（包含中文预览）
- [ ] 看到分隔符 `---`
- [ ] AI 生成了最终回答
- [ ] 回答质量满意

## 📝 报告

如果测试成功：
```
✅ UTF-8 字符边界修复验证通过
- 应用稳定运行
- 工具调用正常
- 多轮对话正常
- 中文内容处理正常
```

如果测试失败：
```
❌ 测试失败
- 失败现象：[描述]
- 错误日志：[粘贴日志]
- 复现步骤：[详细步骤]
```

---

**测试时间**: 预计 5 分钟  
**测试人员**: [您的名字]  
**测试环境**: [操作系统 + 应用版本]  
**测试结果**: [ ] 通过 [ ] 失败

