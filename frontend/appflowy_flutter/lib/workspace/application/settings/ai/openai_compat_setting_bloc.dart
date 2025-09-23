import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-user/protobuf.dart' as userpb;
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:http/http.dart' as http;

import 'package:appflowy/user/application/user_settings_service.dart'
    as user_settings;

class OpenAICompatSettingState extends Equatable {
  const OpenAICompatSettingState({
    this.isLoading = true,
    this.isSaving = false,
    this.isTesting = false,
    this.isTestingEmbed = false,
    this.baseUrl = '',
    this.chatBaseUrl = '',
    this.embedBaseUrl = '',
    this.apiKey = '',
    this.model = 'gpt-4o-mini',
    this.embeddingModel = 'text-embedding-3-small',
    this.temperature = 0.7,
    this.maxTokens = 1024,
    this.timeoutMs = 20000,
    this.error,
    this.testResult,
    this.embedTestResult,
  });

  final bool isLoading;
  final bool isSaving;
  final bool isTesting;
  final bool isTestingEmbed;
  final String baseUrl;
  final String chatBaseUrl; // overrides baseUrl if provided
  final String embedBaseUrl; // overrides baseUrl if provided
  final String apiKey; // stored in-memory; never logged
  final String model;
  final String embeddingModel;
  final double temperature; // 0..2
  final int maxTokens;
  final int timeoutMs;
  final String? error;
  final OpenAITestResult? testResult;
  final OpenAIEmbedTestResult? embedTestResult;

  OpenAICompatSettingState copyWith({
    bool? isLoading,
    bool? isSaving,
    bool? isTesting,
    bool? isTestingEmbed,
    String? baseUrl,
    String? chatBaseUrl,
    String? embedBaseUrl,
    String? apiKey,
    String? model,
    String? embeddingModel,
    double? temperature,
    int? maxTokens,
    int? timeoutMs,
    String? error,
    OpenAITestResult? testResult,
    OpenAIEmbedTestResult? embedTestResult,
  }) {
    return OpenAICompatSettingState(
      isLoading: isLoading ?? this.isLoading,
      isSaving: isSaving ?? this.isSaving,
      isTesting: isTesting ?? this.isTesting,
      isTestingEmbed: isTestingEmbed ?? this.isTestingEmbed,
      baseUrl: baseUrl ?? this.baseUrl,
      chatBaseUrl: chatBaseUrl ?? this.chatBaseUrl,
      embedBaseUrl: embedBaseUrl ?? this.embedBaseUrl,
      apiKey: apiKey ?? this.apiKey,
      model: model ?? this.model,
      embeddingModel: embeddingModel ?? this.embeddingModel,
      temperature: temperature ?? this.temperature,
      maxTokens: maxTokens ?? this.maxTokens,
      timeoutMs: timeoutMs ?? this.timeoutMs,
      error: error,
      testResult: testResult,
      embedTestResult: embedTestResult,
    );
  }

  @override
  List<Object?> get props => [
        isLoading,
        isSaving,
        isTesting,
        isTestingEmbed,
        baseUrl,
        chatBaseUrl,
        embedBaseUrl,
        apiKey,
        model,
        embeddingModel,
        temperature,
        maxTokens,
        timeoutMs,
        error,
        testResult,
        embedTestResult,
      ];
}

class OpenAITestResult extends Equatable {
  const OpenAITestResult({
    required this.success,
    required this.latencyMs,
    this.text = '',
    this.error,
    this.timeline = const [],
    this.debug = '',
  });

  final bool success;
  final int latencyMs;
  final String text;
  final String? error;
  final List<String> timeline; // SSE markers, no secrets
  final String debug; // sanitized request/response info

  @override
  List<Object?> get props => [success, latencyMs, text, error, timeline, debug];
}

class OpenAIEmbedTestResult extends Equatable {
  const OpenAIEmbedTestResult({
    required this.success,
    required this.latencyMs,
    required this.vectorDim,
    this.error,
    this.debug = '',
  });

  final bool success;
  final int latencyMs;
  final int vectorDim;
  final String? error;
  final String debug;

  @override
  List<Object?> get props => [success, latencyMs, vectorDim, error, debug];
}

class OpenAICompatSettingBloc extends Cubit<OpenAICompatSettingState> {
  OpenAICompatSettingBloc({user_settings.UserSettingsBackendService? service, String? workspaceId})
      : _service = service ?? const user_settings.UserSettingsBackendService(),
        _workspaceId = workspaceId,
        super(const OpenAICompatSettingState()) {
    _load();
  }

  // Keys saved into AppearanceSettingsPB.settingKeyValue
  static const String _kBaseUrl = 'ai.openai.baseUrl';
  static const String _kChatBaseUrl = 'ai.openai.chatBaseUrl';
  static const String _kEmbedBaseUrl = 'ai.openai.embedBaseUrl';
  static const String _kApiKey = 'ai.openai.apiKey';
  static const String _kModel = 'ai.openai.model';
  static const String _kEmbeddingModel = 'ai.openai.embeddingModel';
  static const String _kTemperature = 'ai.openai.temperature';
  static const String _kMaxTokens = 'ai.openai.maxTokens';
  static const String _kTimeoutMs = 'ai.openai.timeoutMs';

  final user_settings.UserSettingsBackendService _service;
  userpb.AppearanceSettingsPB? _appearance;
  final String? _workspaceId;

  Future<void> _load() async {
    try {
      final appearance = await _service.getAppearanceSetting();
      _appearance = appearance;
      String scoped(String key) => _workspaceId == null ? key : '$key.${_workspaceId}';
      String getKV(String key, {String fallback = ''}) {
        return appearance.settingKeyValue[scoped(key)] ?? appearance.settingKeyValue[key] ?? fallback;
      }

      emit(state.copyWith(
        isLoading: false,
        baseUrl: getKV(_kBaseUrl),
        chatBaseUrl: getKV(_kChatBaseUrl),
        embedBaseUrl: getKV(_kEmbedBaseUrl),
        apiKey: getKV(_kApiKey),
        model: getKV(_kModel, fallback: 'gpt-4o-mini'),
        embeddingModel: getKV(_kEmbeddingModel, fallback: 'text-embedding-3-small'),
        temperature: double.tryParse(getKV(_kTemperature)) ?? 0.7,
        maxTokens: int.tryParse(getKV(_kMaxTokens)) ?? 1024,
        timeoutMs: int.tryParse(getKV(_kTimeoutMs)) ?? 20000,
      ));
    } catch (e) {
      Log.warn('Failed to load OpenAI compat settings');
      emit(state.copyWith(isLoading: false));
    }
  }

  void updateBaseUrl(String v) => emit(state.copyWith(baseUrl: v));
  void updateChatBaseUrl(String v) => emit(state.copyWith(chatBaseUrl: v));
  void updateEmbedBaseUrl(String v) => emit(state.copyWith(embedBaseUrl: v));
  void updateApiKey(String v) => emit(state.copyWith(apiKey: v));
  void updateModel(String v) => emit(state.copyWith(model: v));
  void updateEmbeddingModel(String v) => emit(state.copyWith(embeddingModel: v));
  void updateTemperature(double v) => emit(state.copyWith(temperature: v));
  void updateMaxTokens(int v) => emit(state.copyWith(maxTokens: v));
  void updateTimeoutMs(int v) => emit(state.copyWith(timeoutMs: v));

  Future<void> save() async {
    emit(state.copyWith(isSaving: true));
    try {
      _appearance ??= await _service.getAppearanceSetting();
      final a = _appearance!;
      String scoped(String key) => _workspaceId == null ? key : '$key.${_workspaceId}';
      a.settingKeyValue[scoped(_kBaseUrl)] = state.baseUrl.trim();
      if (state.chatBaseUrl.trim().isNotEmpty) {
        a.settingKeyValue[scoped(_kChatBaseUrl)] = state.chatBaseUrl.trim();
      } else {
        a.settingKeyValue.remove(scoped(_kChatBaseUrl));
      }
      if (state.embedBaseUrl.trim().isNotEmpty) {
        a.settingKeyValue[scoped(_kEmbedBaseUrl)] = state.embedBaseUrl.trim();
      } else {
        a.settingKeyValue.remove(scoped(_kEmbedBaseUrl));
      }
      a.settingKeyValue[scoped(_kApiKey)] = state.apiKey.trim();
      a.settingKeyValue[scoped(_kModel)] = state.model.trim();
      a.settingKeyValue[scoped(_kEmbeddingModel)] = state.embeddingModel.trim();
      a.settingKeyValue[scoped(_kTemperature)] = state.temperature.toString();
      a.settingKeyValue[scoped(_kMaxTokens)] = state.maxTokens.toString();
      a.settingKeyValue[scoped(_kTimeoutMs)] = state.timeoutMs.toString();
      await _service.setAppearanceSetting(a);
      emit(state.copyWith(isSaving: false));
    } catch (e) {
      Log.error('Failed to save OpenAI compat settings');
      emit(state.copyWith(isSaving: false, error: e.toString()));
    }
  }

  Future<void> testChat({String prompt = 'ping'}) async {
    if ((state.chatBaseUrl.isEmpty && state.baseUrl.isEmpty) || state.apiKey.isEmpty || state.model.isEmpty) {
      emit(state.copyWith(
        testResult: const OpenAITestResult(
          success: false,
          latencyMs: 0,
          error: 'Missing required fields',
        ),
      ));
      return;
    }

    emit(state.copyWith(isTesting: true, testResult: null, error: null));
    final started = DateTime.now();
    try {
      final base = (state.chatBaseUrl.isNotEmpty ? state.chatBaseUrl : state.baseUrl).trim();
      final result = await _runSseChatTest(
        baseUrl: base,
        apiKey: state.apiKey,
        model: state.model,
        temperature: state.temperature,
        maxTokens: state.maxTokens,
        timeoutMs: state.timeoutMs,
        prompt: prompt,
      );
      final latency = DateTime.now().difference(started).inMilliseconds;
      emit(state.copyWith(
        isTesting: false,
        testResult: OpenAITestResult(
          success: true,
          latencyMs: latency,
          text: result.$1,
          timeline: result.$2,
          debug: _buildChatDebug(_joinUrl(base, '/v1/chat/completions'), state.model, state.temperature, state.maxTokens, 200, ''),
        ),
      ));
    } catch (e) {
      final latency = DateTime.now().difference(started).inMilliseconds;
      final base = (state.chatBaseUrl.isNotEmpty ? state.chatBaseUrl : state.baseUrl).trim();
      emit(state.copyWith(
        isTesting: false,
        testResult: OpenAITestResult(
          success: false,
          latencyMs: latency,
          error: e.toString(),
          debug: _buildChatDebug(_joinUrl(base, '/v1/chat/completions'), state.model, state.temperature, state.maxTokens, -1, e.toString()),
        ),
      ));
    }
  }

  // Returns (text, timeline)
  Future<(String, List<String>)> _runSseChatTest({
    required String baseUrl,
    required String apiKey,
    required String model,
    required double temperature,
    required int maxTokens,
    required int timeoutMs,
    required String prompt,
  }) async {
    final uri = Uri.parse(_joinUrl(baseUrl.trim(), '/v1/chat/completions'));

    // Build request
    final headers = <String, String>{
      'Authorization': 'Bearer $apiKey',
      'Content-Type': 'application/json',
      'Accept': 'text/event-stream',
    };

    final body = jsonEncode({
      'model': model.trim(),
      'messages': [
        {
          'role': 'user',
          'content': prompt,
        }
      ],
      'temperature': temperature,
      'max_tokens': maxTokens,
      'stream': true,
    });

    final req = http.Request('POST', uri);
    req.headers.addAll(headers);
    req.body = body;

    final client = http.Client();
    try {
      final streamed = await client.send(req).timeout(
        Duration(milliseconds: timeoutMs.clamp(1000, 120000)),
      );

      if (streamed.statusCode < 200 || streamed.statusCode >= 300) {
        final respBody = await streamed.stream.bytesToString();
        throw Exception('HTTP ${streamed.statusCode}: $respBody');
      }

      final sse = streamed.stream
          .transform(utf8.decoder)
          .transform(const LineSplitter());

      final buffer = StringBuffer();
      final timeline = <String>[];
      await for (final line in sse.timeout(
        Duration(milliseconds: (timeoutMs * 1.5).toInt().clamp(2000, 180000)),
        onTimeout: (sink) {
          sink.addError(TimeoutException('SSE timed out'));
        },
      )) {
        if (line.isEmpty) continue;
        if (line.startsWith('data:')) {
          final data = line.substring(5).trimLeft();
          if (data == '[DONE]') {
            timeline.add('done');
            break;
          }
          try {
            final jsonObj = jsonDecode(data) as Map<String, dynamic>;
            final choices = jsonObj['choices'] as List<dynamic>?;
            if (choices != null && choices.isNotEmpty) {
              final first = choices.first as Map<String, dynamic>;
              final delta = first['delta'] as Map<String, dynamic>?;
              final content = (delta != null ? delta['content'] : null) as String?;
              if (content != null && content.isNotEmpty) {
                buffer.write(content);
              }
            }
            timeline.add('chunk');
          } catch (_) {
            // Skip malformed chunks
            timeline.add('malformed');
          }
        }
      }

      return (buffer.toString(), timeline);
    } finally {
      client.close();
    }
  }

  String _joinUrl(String base, String path) {
    if (base.endsWith('/')) {
      base = base.substring(0, base.length - 1);
    }
    return '$base$path';
  }

  Future<void> testEmbeddings({String input = 'hello world'}) async {
    if ((state.embedBaseUrl.isEmpty && state.baseUrl.isEmpty) || state.apiKey.isEmpty || state.embeddingModel.isEmpty) {
      emit(state.copyWith(
        embedTestResult: const OpenAIEmbedTestResult(
          success: false,
          latencyMs: 0,
          vectorDim: 0,
          error: 'Missing required fields',
        ),
      ));
      return;
    }

    emit(state.copyWith(isTestingEmbed: true, embedTestResult: null, error: null));
    final started = DateTime.now();
    final base = (state.embedBaseUrl.isNotEmpty ? state.embedBaseUrl : state.baseUrl).trim();
    final uri = Uri.parse(_joinUrl(base, '/v1/embeddings'));
    final headers = <String, String>{
      'Authorization': 'Bearer ${state.apiKey}',
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };
    final body = jsonEncode({
      'model': state.embeddingModel.trim(),
      'input': input,
    });

    try {
      final resp = await http
          .post(uri, headers: headers, body: body)
          .timeout(Duration(milliseconds: state.timeoutMs.clamp(1000, 120000)));

      final latency = DateTime.now().difference(started).inMilliseconds;
      final respBody = resp.body;

      if (resp.statusCode < 200 || resp.statusCode >= 300) {
        final debug = _buildEmbedDebug(uri.toString(), state.embeddingModel, input.length, resp.statusCode, respBody);
        emit(state.copyWith(
          isTestingEmbed: false,
          embedTestResult: OpenAIEmbedTestResult(
            success: false,
            latencyMs: latency,
            vectorDim: 0,
            error: 'http_error: ${resp.statusCode}',
            debug: debug,
          ),
        ));
        return;
      }

      int vectorDim = 0;
      try {
        final jsonObj = jsonDecode(respBody) as Map<String, dynamic>;
        final data = jsonObj['data'] as List<dynamic>?;
        if (data != null && data.isNotEmpty) {
          final first = data.first as Map<String, dynamic>;
          final embedding = first['embedding'] as List<dynamic>?;
          vectorDim = embedding?.length ?? 0;
        }
      } catch (_) {
        // ignore and fall through with vectorDim 0
      }

      final debug = _buildEmbedDebug(uri.toString(), state.embeddingModel, input.length, resp.statusCode, respBody);
      emit(state.copyWith(
        isTestingEmbed: false,
        embedTestResult: OpenAIEmbedTestResult(
          success: vectorDim > 0,
          latencyMs: latency,
          vectorDim: vectorDim,
          error: vectorDim > 0 ? null : 'json_parse: missing embedding',
          debug: debug,
        ),
      ));
    } on TimeoutException catch (e) {
      final latency = DateTime.now().difference(started).inMilliseconds;
      emit(state.copyWith(
        isTestingEmbed: false,
        embedTestResult: OpenAIEmbedTestResult(
          success: false,
          latencyMs: latency,
          vectorDim: 0,
          error: 'timeout: ${e.message ?? ''}'.trim(),
          debug: _buildEmbedDebug(_joinUrl(state.baseUrl.trim(), '/v1/embeddings'), state.embeddingModel, input.length, -1, ''),
        ),
      ));
    } on SocketException catch (e) {
      final latency = DateTime.now().difference(started).inMilliseconds;
      emit(state.copyWith(
        isTestingEmbed: false,
        embedTestResult: OpenAIEmbedTestResult(
          success: false,
          latencyMs: latency,
          vectorDim: 0,
          error: 'network: ${e.message}',
          debug: _buildEmbedDebug(_joinUrl(state.baseUrl.trim(), '/v1/embeddings'), state.embeddingModel, input.length, -1, ''),
        ),
      ));
    } catch (e) {
      final latency = DateTime.now().difference(started).inMilliseconds;
      emit(state.copyWith(
        isTestingEmbed: false,
        embedTestResult: OpenAIEmbedTestResult(
          success: false,
          latencyMs: latency,
          vectorDim: 0,
          error: e.toString(),
          debug: _buildEmbedDebug(_joinUrl(state.baseUrl.trim(), '/v1/embeddings'), state.embeddingModel, input.length, -1, ''),
        ),
      ));
    }
  }

  String _buildEmbedDebug(String url, String model, int inputLength, int status, String respBody) {
    final preview = respBody.length > 512 ? '${respBody.substring(0, 512)}...' : respBody;
    final debug = {
      'url': url,
      'model': model,
      'authorization': 'Bearer ***',
      'input_length': inputLength,
      'status': status,
      'response_preview': preview,
    };
    return const JsonEncoder.withIndent('  ').convert(debug);
  }

  String _buildChatDebug(String url, String model, double temperature, int maxTokens, int status, String error) {
    final body = {
      'model': model.trim(),
      'messages': [
        {'role': 'user', 'content': '...'}
      ],
      'temperature': temperature,
      'max_tokens': maxTokens,
      'stream': true,
    };
    final debug = {
      'url': url,
      'authorization': 'Bearer ***',
      'request_body': body,
      'status': status,
      'error': error,
    };
    return const JsonEncoder.withIndent('  ').convert(debug);
  }
}


