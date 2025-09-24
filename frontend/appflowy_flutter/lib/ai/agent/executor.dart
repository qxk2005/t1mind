import 'dart:async';
import 'dart:convert';

import 'package:appflowy/ai/service/ai_entities.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_message_stream.dart';

import 'planner.dart';

class CancelToken {
  bool _cancelled = false;
  bool get isCancelled => _cancelled;
  void cancel() => _cancelled = true;
}

class AgentExecutor {
  Future<void> runMock({
    required AnswerStream answerStream,
    required AgentPlan plan,
    required CancelToken cancelToken,
  }) async {
    // Emit a minimal log metadata at the beginning
    answerStream.injectMetadataJson(jsonEncode({
      'id': 'tool',
      'name': plan.steps.first.endpointName,
      'source': 'mcp',
    }));

    // Simulate step-by-step streaming
    final text = '正在调用 ${plan.steps.first.endpointName} …\n\n输入: ${plan.steps.first.input}\n\n结果: 这是模拟工具返回的数据。';
    const chunk = 8;
    for (int i = 0; i < text.length; i += chunk) {
      if (cancelToken.isCancelled) return;
      final end = (i + chunk).clamp(0, text.length);
      answerStream.injectData(text.substring(i, end));
      await Future<void>.delayed(const Duration(milliseconds: 40));
    }

    // Finish
    answerStream.injectRaw(AIStreamEventPrefix.finish);
  }
}


