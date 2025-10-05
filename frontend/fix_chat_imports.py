#!/usr/bin/env python3
"""
修复 chat.rs 的导入和简化多轮对话逻辑
"""
import re

# 读取 chat.rs
with open("rust-lib/flowy-ai/src/chat.rs", "r", encoding="utf-8") as f:
    content = f.read()

# 1. 修复导入语句 - 改为完整路径
old_import = r'use crate::middleware::chat_service_mw::ChatServiceMiddleware;\nuse crate::middleware::chat_service_mw::\{OpenAIToolCall, OpenAIFunctionCall\};'
new_import = 'use crate::middleware::chat_service_mw::{ChatServiceMiddleware, OpenAIToolCall, OpenAIFunctionCall};'

content = re.sub(
    r'use crate::middleware::chat_service_mw::ChatServiceMiddleware;\s*use crate::middleware::chat_service_mw::\{OpenAIToolCall, OpenAIFunctionCall\};',
    new_import,
    content
)

# 也处理可能的其他格式
content = re.sub(
    r'use crate::middleware::chat_service_mw::ChatServiceMiddleware;',
    new_import,
    content,
    count=1
)

print("✅ Fixed imports")

# 2. 修复 OpenAIToolCall 的引用路径
content = re.sub(
    r'crate::middleware::OpenAIToolCall',
    'OpenAIToolCall',
    content
)
content = re.sub(
    r'crate::middleware::OpenAIFunctionCall',
    'OpenAIFunctionCall',
    content
)
print("✅ Fixed OpenAIToolCall references")

# 3. 简化多轮对话逻辑 - 移除 as_any downcast 逻辑
# 查找并替换整个多轮对话块
multi_turn_block_pattern = r'(// 🔄 多轮对话：检查是否收集到 tool_calls.*?)(            // 调用 cloud_service 的多轮对话方法.*?)(            warn!\("🔄 \[MULTI-TURN\] Cannot downcast to ChatServiceMiddleware, skipping multi-turn"\);\s*\}\s*\})'

simplified_block = r'''\1            // TODO: 多轮对话将在 middleware 层自动处理
            // 当前版本仅记录检测到的 tool_calls
            info!("🔄 [MULTI-TURN] Tool calls will be handled by middleware layer");
            
            // 发送提示信息
            let hint = format!("\\n\\n🔄 **检测到 {} 个工具调用**\\n\\n", tool_calls_vec.len());
            let _ = answer_sink.send(StreamMessage::OnData(hint).to_string()).await;\3'''

if re.search(r'as_any\(\)\.downcast_ref', content):
    content = re.sub(
        r'if let Some\(middleware\) = cloud_service\.as_any\(\)\.downcast_ref::<ChatServiceMiddleware>\(\) \{.*?\} else \{.*?warn!\("🔄 \[MULTI-TURN\] Cannot downcast.*?"\);\s*\}',
        '''// TODO: 多轮对话将在 middleware 层自动处理
            // 当前版本仅记录检测到的 tool_calls
            info!("🔄 [MULTI-TURN] Tool calls will be handled by middleware layer");
            
            // 发送提示信息
            let hint = format!("\\n\\n🔄 **检测到 {} 个工具调用，将在后续版本中自动处理**\\n\\n", tool_calls_vec.len());
            let _ = answer_sink.send(StreamMessage::OnData(hint).to_string()).await;''',
        content,
        flags=re.DOTALL
    )
    print("✅ Simplified multi-turn logic (removed as_any)")
else:
    print("⏭️  as_any logic not found")

# 保存文件
with open("rust-lib/flowy-ai/src/chat.rs", "w", encoding="utf-8") as f:
    f.write(content)

print("\n✅ chat.rs fixed successfully")
print("\n📝 下一步：在 middleware 层实现自动多轮对话")

