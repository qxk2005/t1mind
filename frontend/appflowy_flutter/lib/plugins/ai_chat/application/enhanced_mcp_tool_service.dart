import 'dart:async';

import 'package:appflowy_backend/log.dart';
import 'package:nanoid/nanoid.dart';

import 'task_planner_entities.dart';
import 'mcp_endpoint_service.dart';
import 'mcp_tools_service.dart';

/// 增强的MCP工具调用服务
/// 
/// 提供智能重试和替代工具选择机制，包括：
/// 1. 智能工具选择算法
/// 2. 失败重试机制
/// 3. 替代工具自动选择
/// 4. 执行结果评估
/// 5. 性能监控和优化
class EnhancedMcpToolService {
  EnhancedMcpToolService({
    this.onToolCallStarted,
    this.onToolCallCompleted,
    this.onToolCallFailed,
    this.onToolRetry,
  });

  final Function(String toolId, Map<String, dynamic> parameters)? onToolCallStarted;
  final Function(String toolId, Map<String, dynamic> result)? onToolCallCompleted;
  final Function(String toolId, String error)? onToolCallFailed;
  final Function(String toolId, int retryCount, String reason)? onToolRetry;

  final McpEndpointService _endpointService = McpEndpointService();
  final McpToolsService _toolsService = McpToolsService();

  // 工具调用统计
  final Map<String, ToolCallStats> _toolStats = {};
  
  // 工具性能缓存
  final Map<String, ToolPerformanceMetrics> _performanceCache = {};

  /// 智能执行工具调用
  /// 
  /// 根据步骤目标和可用工具，智能选择最合适的工具并执行
  /// 支持失败重试和替代工具选择
  Future<ToolCallResult> executeToolCall({
    required TaskStep step,
    required Map<String, dynamic> context,
    int maxRetries = 3,
    Duration timeout = const Duration(seconds: 30),
  }) async {
    Log.info('开始智能工具调用: 步骤 ${step.id}');
    
    final attemptId = nanoid();
    final startTime = DateTime.now();
    
    try {
      // 1. 智能选择工具
      final selectedTool = await _selectOptimalTool(step, context);
      if (selectedTool == null) {
        throw ToolSelectionException('无法为步骤找到合适的工具: ${step.objective}');
      }

      Log.info('AI选择了工具: ${selectedTool.name} 用于步骤: ${step.id}');
      
      // 2. 生成工具调用参数
      final parameters = await _generateToolParameters(step, selectedTool, context);
      
      // 3. 执行工具调用（带重试机制）
      final result = await _executeWithRetry(
        step: step,
        tool: selectedTool,
        parameters: parameters,
        maxRetries: maxRetries,
        timeout: timeout,
        context: context,
      );

      // 4. 更新统计信息
      _updateToolStats(selectedTool.name, true, DateTime.now().difference(startTime));
      
      return result;

    } catch (e) {
      Log.error('工具调用失败: 步骤 ${step.id} - $e');
      
      // 更新失败统计
      if (step.mcpToolId != null) {
        _updateToolStats(step.mcpToolId!, false, DateTime.now().difference(startTime));
      }
      
      return ToolCallResult(
        attemptId: attemptId,
        stepId: step.id,
        toolId: step.mcpToolId ?? 'unknown',
        toolName: step.mcpToolId ?? 'unknown',
        status: ToolCallStatus.failed,
        startTime: startTime,
        endTime: DateTime.now(),
        errorMessage: e.toString(),
      );
    }
  }

  /// 智能选择最优工具
  Future<McpToolSchema?> _selectOptimalTool(
    TaskStep step,
    Map<String, dynamic> context,
  ) async {
    // 如果步骤已指定工具，直接使用
    if (step.mcpToolId != null && step.mcpToolId != 'ai-assistant') {
      return _findToolByName(step.availableTools, step.mcpToolId!);
    }

    // 如果没有可用工具，返回null
    if (step.availableTools.isEmpty) {
      return null;
    }

    // 使用智能算法选择最佳工具
    return await _analyzeAndSelectBestTool(step, context);
  }

  /// 分析并选择最佳工具
  Future<McpToolSchema?> _analyzeAndSelectBestTool(
    TaskStep step,
    Map<String, dynamic> context,
  ) async {
    final candidates = <ToolCandidate>[];
    
    // 为每个可用工具计算适合度分数
    for (final tool in step.availableTools) {
      final score = await _calculateToolFitnessScore(tool, step, context);
      candidates.add(ToolCandidate(tool: tool, score: score));
    }
    
    // 按分数排序，选择最高分的工具
    candidates.sort((a, b) => b.score.compareTo(a.score));
    
    if (candidates.isNotEmpty && candidates.first.score > 0.3) {
      Log.info('选择工具: ${candidates.first.tool.name}, 适合度分数: ${candidates.first.score}');
      return candidates.first.tool;
    }
    
    return null;
  }

  /// 计算工具适合度分数
  Future<double> _calculateToolFitnessScore(
    McpToolSchema tool,
    TaskStep step,
    Map<String, dynamic> context,
  ) async {
    double score = 0.0;
    
    // 1. 基于工具描述和步骤目标的语义匹配 (40%)
    score += _calculateSemanticMatch(tool.description, step.objective) * 0.4;
    
    // 2. 基于工具标签和步骤参数的匹配 (20%)
    score += _calculateTagMatch(tool.tags, step.parameters) * 0.2;
    
    // 3. 基于历史性能数据 (20%)
    score += _calculatePerformanceScore(tool.name) * 0.2;
    
    // 4. 基于工具可用性和稳定性 (10%)
    score += _calculateReliabilityScore(tool.name) * 0.1;
    
    // 5. 基于上下文相关性 (10%)
    score += _calculateContextRelevance(tool, context) * 0.1;
    
    return score.clamp(0.0, 1.0);
  }

  /// 计算语义匹配度
  double _calculateSemanticMatch(String toolDescription, String stepObjective) {
    if (toolDescription.isEmpty || stepObjective.isEmpty) {
      return 0.0;
    }
    
    // 简化的语义匹配算法
    // 实际应用中可以使用更复杂的NLP算法
    final toolWords = toolDescription.toLowerCase().split(RegExp(r'\W+'));
    final objectiveWords = stepObjective.toLowerCase().split(RegExp(r'\W+'));
    
    int matchCount = 0;
    for (final word in objectiveWords) {
      if (word.length > 2 && toolWords.contains(word)) {
        matchCount++;
      }
    }
    
    return objectiveWords.isNotEmpty ? matchCount / objectiveWords.length : 0.0;
  }

  /// 计算标签匹配度
  double _calculateTagMatch(List<String> toolTags, Map<String, dynamic> stepParameters) {
    if (toolTags.isEmpty) {
      return 0.5; // 中性分数
    }
    
    // 检查步骤参数中是否包含相关的标签关键词
    final parameterText = stepParameters.values.join(' ').toLowerCase();
    int matchCount = 0;
    
    for (final tag in toolTags) {
      if (parameterText.contains(tag.toLowerCase())) {
        matchCount++;
      }
    }
    
    return toolTags.isNotEmpty ? matchCount / toolTags.length : 0.0;
  }

  /// 计算性能分数
  double _calculatePerformanceScore(String toolName) {
    final metrics = _performanceCache[toolName];
    if (metrics == null) {
      return 0.5; // 中性分数，对于未知工具
    }
    
    // 基于成功率和平均响应时间计算分数
    final successRateScore = metrics.successRate;
    final responseTimeScore = 1.0 - (metrics.averageResponseTimeMs / 10000.0).clamp(0.0, 1.0);
    
    return (successRateScore * 0.7 + responseTimeScore * 0.3);
  }

  /// 计算可靠性分数
  double _calculateReliabilityScore(String toolName) {
    final stats = _toolStats[toolName];
    if (stats == null) {
      return 0.5; // 中性分数
    }
    
    // 基于调用次数和成功率计算可靠性
    final totalCalls = stats.successCount + stats.failureCount;
    if (totalCalls == 0) {
      return 0.5;
    }
    
    final successRate = stats.successCount / totalCalls;
    final experienceBonus = (totalCalls / 100.0).clamp(0.0, 0.2); // 经验加成
    
    return (successRate + experienceBonus).clamp(0.0, 1.0);
  }

  /// 计算上下文相关性
  double _calculateContextRelevance(McpToolSchema tool, Map<String, dynamic> context) {
    // 简化的上下文相关性计算
    // 可以根据具体需求扩展
    return 0.5;
  }

  /// 带重试机制的工具执行
  Future<ToolCallResult> _executeWithRetry({
    required TaskStep step,
    required McpToolSchema tool,
    required Map<String, dynamic> parameters,
    required int maxRetries,
    required Duration timeout,
    required Map<String, dynamic> context,
  }) async {
    int retryCount = 0;
    Exception? lastException;
    
    while (retryCount <= maxRetries) {
      final attemptId = nanoid();
      final startTime = DateTime.now();
      
      try {
        Log.info('执行工具调用: ${tool.name}, 尝试次数: ${retryCount + 1}');
        
        // 通知工具调用开始
        onToolCallStarted?.call(tool.name, parameters);
        
        // 执行实际的工具调用
        final result = await _performToolCall(
          endpointId: step.mcpEndpointId!,
          toolName: tool.name,
          parameters: parameters,
          timeout: timeout,
        );
        
        // 验证结果
        final isValid = await _validateToolResult(tool, result, step.objective);
        if (!isValid && retryCount < maxRetries) {
          retryCount++;
          onToolRetry?.call(tool.name, retryCount, '结果验证失败，尝试重试');
          continue;
        }
        
        // 成功完成
        final endTime = DateTime.now();
        onToolCallCompleted?.call(tool.name, result);
        
        return ToolCallResult(
          attemptId: attemptId,
          stepId: step.id,
          toolId: tool.name,
          toolName: tool.name,
          status: ToolCallStatus.success,
          inputParameters: parameters,
          outputResult: result,
          startTime: startTime,
          endTime: endTime,
          durationMs: endTime.difference(startTime).inMilliseconds,
        );
        
      } catch (e) {
        lastException = e is Exception ? e : Exception(e.toString());
        Log.warn('工具调用失败: ${tool.name}, 尝试 ${retryCount + 1}, 错误: $e');
        
        // 如果还有重试机会，尝试使用替代工具或参数
        if (retryCount < maxRetries) {
          retryCount++;
          
          // 决定重试策略
          final retryStrategy = _determineRetryStrategy(e, retryCount);
          
          switch (retryStrategy) {
            case RetryStrategy.useAlternativeTool:
              // 尝试使用替代工具
              final alternativeTool = await _selectAlternativeTool(step, tool, context);
              if (alternativeTool != null) {
                tool = alternativeTool;
                parameters = await _generateToolParameters(step, tool, context);
                onToolRetry?.call(tool.name, retryCount, '使用替代工具重试');
              }
              break;
              
            case RetryStrategy.adjustParameters:
              // 调整参数重试
              parameters = await _adjustParametersForRetry(parameters, e, retryCount);
              onToolRetry?.call(tool.name, retryCount, '调整参数重试');
              break;
              
            case RetryStrategy.waitAndRetry:
              // 等待后重试
              await Future.delayed(Duration(seconds: retryCount * 2));
              onToolRetry?.call(tool.name, retryCount, '等待后重试');
              break;
          }
        }
      }
    }
    
    // 所有重试都失败了
    final endTime = DateTime.now();
    onToolCallFailed?.call(tool.name, lastException?.toString() ?? 'Unknown error');
    
    return ToolCallResult(
      attemptId: nanoid(),
      stepId: step.id,
      toolId: tool.name,
      toolName: tool.name,
      status: ToolCallStatus.failed,
      inputParameters: parameters,
      startTime: DateTime.now().subtract(Duration(seconds: retryCount * 5)),
      endTime: endTime,
      errorMessage: lastException?.toString() ?? 'Unknown error',
      retryCount: retryCount,
    );
  }

  /// 执行实际的工具调用
  Future<Map<String, dynamic>> _performToolCall({
    required String endpointId,
    required String toolName,
    required Map<String, dynamic> parameters,
    required Duration timeout,
  }) async {
    // 这里应该调用实际的MCP工具服务
    // 目前返回模拟结果
    await Future.delayed(Duration(milliseconds: 200 + (parameters.length * 50)));
    
    return {
      'endpointId': endpointId,
      'toolName': toolName,
      'status': 'success',
      'output': '工具 $toolName 执行成功',
      'parameters': parameters,
      'timestamp': DateTime.now().toIso8601String(),
    };
  }

  /// 验证工具执行结果
  Future<bool> _validateToolResult(
    McpToolSchema tool,
    Map<String, dynamic> result,
    String objective,
  ) async {
    // 基本验证：检查结果是否包含必要的字段
    if (!result.containsKey('status') || result['status'] != 'success') {
      return false;
    }
    
    // 可以添加更复杂的验证逻辑
    // 例如：检查输出格式、内容质量等
    
    return true;
  }

  /// 确定重试策略
  RetryStrategy _determineRetryStrategy(dynamic error, int retryCount) {
    final errorMessage = error.toString().toLowerCase();
    
    if (errorMessage.contains('timeout') || errorMessage.contains('network')) {
      return RetryStrategy.waitAndRetry;
    } else if (errorMessage.contains('parameter') || errorMessage.contains('invalid')) {
      return RetryStrategy.adjustParameters;
    } else if (retryCount <= 2) {
      return RetryStrategy.useAlternativeTool;
    } else {
      return RetryStrategy.waitAndRetry;
    }
  }

  /// 选择替代工具
  Future<McpToolSchema?> _selectAlternativeTool(
    TaskStep step,
    McpToolSchema currentTool,
    Map<String, dynamic> context,
  ) async {
    // 从可用工具中选择一个不同的工具
    final alternatives = step.availableTools
        .where((tool) => tool.name != currentTool.name)
        .toList();
    
    if (alternatives.isEmpty) {
      return null;
    }
    
    // 重新计算适合度分数，选择最佳替代工具
    return await _analyzeAndSelectBestTool(
      step.copyWith(availableTools: alternatives),
      context,
    );
  }

  /// 调整参数用于重试
  Future<Map<String, dynamic>> _adjustParametersForRetry(
    Map<String, dynamic> originalParameters,
    dynamic error,
    int retryCount,
  ) async {
    final adjustedParameters = Map<String, dynamic>.from(originalParameters);
    
    // 根据错误类型调整参数
    final errorMessage = error.toString().toLowerCase();
    
    if (errorMessage.contains('timeout')) {
      // 增加超时时间
      adjustedParameters['timeout'] = (adjustedParameters['timeout'] as int? ?? 30) * 2;
    } else if (errorMessage.contains('rate limit')) {
      // 添加延迟参数
      adjustedParameters['delay'] = retryCount * 1000;
    }
    
    // 添加重试标识
    adjustedParameters['retry_attempt'] = retryCount;
    
    return adjustedParameters;
  }

  /// 生成工具调用参数
  Future<Map<String, dynamic>> _generateToolParameters(
    TaskStep step,
    McpToolSchema tool,
    Map<String, dynamic> context,
  ) async {
    final parameters = Map<String, dynamic>.from(step.parameters);
    
    // 添加步骤相关信息
    parameters['step_objective'] = step.objective;
    parameters['step_description'] = step.description;
    
    // 添加上下文信息
    parameters['context'] = context;
    
    // 根据工具schema调整参数
    if (tool.inputSchema.isNotEmpty) {
      parameters.removeWhere((key, value) => 
          !tool.inputSchema.containsKey(key) && 
          !['step_objective', 'step_description', 'context'].contains(key));
    }
    
    return parameters;
  }

  /// 查找工具
  McpToolSchema? _findToolByName(List<McpToolSchema> tools, String toolName) {
    try {
      return tools.firstWhere((tool) => tool.name == toolName);
    } catch (e) {
      return null;
    }
  }

  /// 更新工具统计信息
  void _updateToolStats(String toolName, bool success, Duration duration) {
    final stats = _toolStats[toolName] ?? ToolCallStats(toolName: toolName);
    
    if (success) {
      stats.successCount++;
      stats.totalResponseTime += duration.inMilliseconds;
    } else {
      stats.failureCount++;
    }
    
    stats.lastCallTime = DateTime.now();
    _toolStats[toolName] = stats;
    
    // 更新性能缓存
    _updatePerformanceCache(toolName, stats);
  }

  /// 更新性能缓存
  void _updatePerformanceCache(String toolName, ToolCallStats stats) {
    final totalCalls = stats.successCount + stats.failureCount;
    if (totalCalls == 0) return;
    
    final successRate = stats.successCount / totalCalls;
    final averageResponseTime = stats.successCount > 0 
        ? stats.totalResponseTime / stats.successCount 
        : 0.0;
    
    _performanceCache[toolName] = ToolPerformanceMetrics(
      toolName: toolName,
      successRate: successRate,
      averageResponseTimeMs: averageResponseTime,
      totalCalls: totalCalls,
      lastUpdated: DateTime.now(),
    );
  }

  /// 获取工具统计信息
  ToolCallStats? getToolStats(String toolName) {
    return _toolStats[toolName];
  }

  /// 获取所有工具统计信息
  Map<String, ToolCallStats> getAllToolStats() {
    return Map.from(_toolStats);
  }

  /// 清除统计信息
  void clearStats() {
    _toolStats.clear();
    _performanceCache.clear();
  }
}

/// 工具候选者
class ToolCandidate {
  const ToolCandidate({
    required this.tool,
    required this.score,
  });

  final McpToolSchema tool;
  final double score;
}

/// 重试策略
enum RetryStrategy {
  useAlternativeTool,
  adjustParameters,
  waitAndRetry,
}

/// 工具调用结果
class ToolCallResult {
  const ToolCallResult({
    required this.attemptId,
    required this.stepId,
    required this.toolId,
    required this.toolName,
    required this.status,
    required this.startTime,
    required this.endTime,
    this.inputParameters,
    this.outputResult,
    this.errorMessage,
    this.retryCount = 0,
    this.durationMs,
  });

  final String attemptId;
  final String stepId;
  final String toolId;
  final String toolName;
  final ToolCallStatus status;
  final Map<String, dynamic>? inputParameters;
  final Map<String, dynamic>? outputResult;
  final DateTime startTime;
  final DateTime endTime;
  final int? durationMs;
  final String? errorMessage;
  final int retryCount;

  bool get isSuccess => status == ToolCallStatus.success;
  bool get isFailed => status == ToolCallStatus.failed;
}

/// 工具调用状态
enum ToolCallStatus {
  success,
  failed,
  timeout,
  cancelled,
}

/// 工具调用统计
class ToolCallStats {
  ToolCallStats({
    required this.toolName,
    this.successCount = 0,
    this.failureCount = 0,
    this.totalResponseTime = 0,
    this.lastCallTime,
  });

  final String toolName;
  int successCount;
  int failureCount;
  int totalResponseTime;
  DateTime? lastCallTime;
}

/// 工具性能指标
class ToolPerformanceMetrics {
  const ToolPerformanceMetrics({
    required this.toolName,
    required this.successRate,
    required this.averageResponseTimeMs,
    required this.totalCalls,
    required this.lastUpdated,
  });

  final String toolName;
  final double successRate;
  final double averageResponseTimeMs;
  final int totalCalls;
  final DateTime lastUpdated;
}

/// 工具选择异常
class ToolSelectionException implements Exception {
  const ToolSelectionException(this.message);
  
  final String message;
  
  @override
  String toString() => 'ToolSelectionException: $message';
}
