import 'dart:async';
import 'dart:convert';

import 'package:appflowy/core/config/kv.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_message_stream.dart';
import 'package:easy_localization/easy_localization.dart';

import 'executor.dart';
import 'planner.dart';

class AgentRun {
  AgentRun({required this.cancel, required this.done});
  final void Function() cancel;
  final Future<void> done;
}

class AgentService {
  static const String _agentsPrefsKey = 'settings.agents.items';

  final DartKeyValue _kv = DartKeyValue();
  final AgentPlanner _planner = AgentPlanner();
  final AgentExecutor _executor = AgentExecutor();

  Future<List<String>> _loadAgentAllowedEndpointNames() async {
    try {
      final str = await _kv.get(_agentsPrefsKey);
      if (str?.isEmpty ?? true) return const [];
      final dynamic decoded = jsonDecode(str!);
      if (decoded is! List) return const [];
      if (decoded.isEmpty) return const [];
      final first = decoded.first;
      if (first is Map<String, dynamic>) {
        final v = first['allowedEndpointNames'];
        if (v is List) {
          return v.map((e) => e.toString()).toList();
        }
      }
      return const [];
    } catch (_) {
      return const [];
    }
  }

  AgentRun run({
    required AnswerStream answerStream,
    required String userMessage,
    required List<String> selectedMcpNames,
  }) {
    final cancelToken = CancelToken();

    // Inject a localized progress step first.
    final step = 'agent.toolCalling'.tr();
    answerStream.injectMetadataJson(jsonEncode({'step': step}));

    final completer = Completer<void>();
    () async {
      final allowed = await _loadAgentAllowedEndpointNames();
      final plan = _planner.planOnce(
        userMessage: userMessage,
        selectedEndpointNames: selectedMcpNames,
        allowedEndpointNames: allowed,
      );

      await _executor.runMock(
        answerStream: answerStream,
        plan: plan,
        cancelToken: cancelToken,
      );

      if (!cancelToken.isCancelled) {
        answerStream.signalEnd();
      }
      completer.complete();
    }();

    return AgentRun(
      cancel: cancelToken.cancel,
      done: completer.future,
    );
  }
}


