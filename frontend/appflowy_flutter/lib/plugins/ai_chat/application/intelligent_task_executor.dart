import 'dart:async';

import 'package:appflowy_backend/log.dart';
import 'package:nanoid/nanoid.dart';

import 'task_planner_entities.dart';
import 'mcp_endpoint_service.dart';

/// 智能任务执行器
/// 
/// 负责：
/// 1. 子任务执行前的目标告知
/// 2. 智能工具选择和调用
/// 3. 执行过程的实时反馈
/// 4. 失败重试和替代工具选择
class IntelligentTaskExecutor {
  IntelligentTaskExecutor({
    required this.sessionId,
    required this.userId,
    this.onNotification,
  });

  final String sessionId;
  final String userId;
  final Function(TaskExecutionNotification)? onNotification;

  final McpEndpointService _endpointService = McpEndpointService();

  /// 执行任务规划
  Future<ExecutionResult> executeTaskPlan(TaskPlan taskPlan) async {
    Log.info('开始执行任务规划: ${taskPlan.id}');
    
    try {
      // 发送任务开始通知
      await _sendNotification(TaskExecutionNotification(
        id: nanoid(),
        taskPlanId: taskPlan.id,
        type: TaskNotificationType.info,
        title: '任务执行开始',
        message: '开始执行任务："${taskPlan.userQuery}"',
        timestamp: DateTime.now(),
        data: {
          'totalSteps': taskPlan.steps.length,
          'estimatedDuration': taskPlan.estimatedDurationSeconds,
        },
      ));

      final executionLog = ExecutionLog(
        id: nanoid(),
        sessionId: sessionId,
        taskPlanId: taskPlan.id,
        startTime: DateTime.now(),
        totalSteps: taskPlan.steps.length,
        status: ExecutionStatus.running,
      );

      final results = <String, dynamic>{};
      int completedSteps = 0;

      // 按顺序执行每个步骤
      for (final step in taskPlan.steps) {
        try {
          Log.info('开始执行步骤: ${step.id} - ${step.description}');
          
          // 发送步骤开始通知
          await _sendNotification(TaskExecutionNotification(
            id: nanoid(),
            taskPlanId: taskPlan.id,
            stepId: step.id,
            type: TaskNotificationType.stepStarted,
            title: '步骤开始执行',
            message: '目标：${step.objective.isNotEmpty ? step.objective : step.description}',
            timestamp: DateTime.now(),
            data: {
              'stepOrder': step.order,
              'stepId': step.id,
              'description': step.description,
              'objective': step.objective,
              'estimatedDuration': step.estimatedDurationSeconds,
            },
          ));

          // 执行步骤
          final stepResult = await _executeStep(step, taskPlan, results);
          results[step.id] = stepResult;
          completedSteps++;

          // 发送步骤完成通知
          await _sendNotification(TaskExecutionNotification(
            id: nanoid(),
            taskPlanId: taskPlan.id,
            stepId: step.id,
            type: TaskNotificationType.stepCompleted,
            title: '步骤执行完成',
            message: '已完成：${step.description}',
            timestamp: DateTime.now(),
            data: {
              'stepOrder': step.order,
              'stepId': step.id,
              'result': stepResult,
              'completedSteps': completedSteps,
              'totalSteps': taskPlan.steps.length,
            },
          ));

        } catch (e) {
          Log.error('步骤执行失败: ${step.id} - $e');
          
          // 发送步骤失败通知
          await _sendNotification(TaskExecutionNotification(
            id: nanoid(),
            taskPlanId: taskPlan.id,
            stepId: step.id,
            type: TaskNotificationType.stepFailed,
            title: '步骤执行失败',
            message: '步骤"${step.description}"执行失败：$e',
            timestamp: DateTime.now(),
            data: {
              'stepOrder': step.order,
              'stepId': step.id,
              'error': e.toString(),
            },
          ));

          // 根据步骤配置决定是否继续执行
          if (!_shouldContinueAfterStepFailure(step, e)) {
            throw Exception('关键步骤失败，终止任务执行: $e');
          }
        }
      }

      // 发送任务完成通知
      await _sendNotification(TaskExecutionNotification(
        id: nanoid(),
        taskPlanId: taskPlan.id,
        type: TaskNotificationType.taskCompleted,
        title: '任务执行完成',
        message: '任务"${taskPlan.userQuery}"已成功完成',
        timestamp: DateTime.now(),
        data: {
          'completedSteps': completedSteps,
          'totalSteps': taskPlan.steps.length,
          'results': results,
        },
      ));

      return ExecutionResult(
        executionId: executionLog.id,
        status: ExecutionStatus.completed,
        result: results,
        executionLog: executionLog.copyWith(
          endTime: DateTime.now(),
          status: ExecutionStatus.completed,
          completedSteps: completedSteps,
        ),
      );

    } catch (e) {
      Log.error('任务执行失败: ${taskPlan.id} - $e');
      
      // 发送任务失败通知
      await _sendNotification(TaskExecutionNotification(
        id: nanoid(),
        taskPlanId: taskPlan.id,
        type: TaskNotificationType.taskFailed,
        title: '任务执行失败',
        message: '任务"${taskPlan.userQuery}"执行失败：$e',
        timestamp: DateTime.now(),
        data: {
          'error': e.toString(),
        },
      ));

      return ExecutionResult(
        executionId: nanoid(),
        status: ExecutionStatus.failed,
        errorMessage: e.toString(),
      );
    }
  }

  /// 执行单个步骤
  Future<Map<String, dynamic>> _executeStep(
    TaskStep step,
    TaskPlan taskPlan,
    Map<String, dynamic> previousResults,
  ) async {
    
    try {
      // 如果步骤指定了具体的工具ID，直接使用
      if (step.mcpToolId != null && step.mcpToolId != 'ai-assistant') {
        return await _executeWithSpecificTool(step, previousResults);
      }
      
      // 如果步骤指定了端点但没有具体工具，进行智能选择
      if (step.mcpEndpointId != null && step.availableTools.isNotEmpty) {
        return await _executeWithIntelligentToolSelection(step, taskPlan, previousResults);
      }
      
      // 默认使用AI助手处理
      return await _executeWithAIAssistant(step, previousResults);
      
    } catch (e) {
      // 如果允许工具替代，尝试使用其他工具
      if (step.allowToolSubstitution && step.retryCount < step.maxRetries) {
        Log.info('尝试使用替代工具执行步骤: ${step.id}');
        return await _retryStepWithAlternativeTool(step, taskPlan, previousResults, e);
      }
      
      rethrow;
    }
  }

  /// 使用指定工具执行步骤
  Future<Map<String, dynamic>> _executeWithSpecificTool(
    TaskStep step,
    Map<String, dynamic> previousResults,
  ) async {
    Log.info('使用指定工具执行步骤: ${step.mcpToolId}');
    
    // 发送工具调用开始通知
    await _sendNotification(TaskExecutionNotification(
      id: nanoid(),
      taskPlanId: '', // 需要从上下文获取
      stepId: step.id,
      type: TaskNotificationType.toolCallStarted,
      title: '工具调用开始',
      message: '正在调用工具：${step.mcpToolId}',
      timestamp: DateTime.now(),
      data: {
        'toolId': step.mcpToolId,
        'parameters': step.parameters,
      },
    ));

    // 这里应该调用实际的MCP工具
    // 目前返回模拟结果
    final result = {
      'toolId': step.mcpToolId,
      'status': 'success',
      'output': '工具 ${step.mcpToolId} 执行成功',
      'timestamp': DateTime.now().toIso8601String(),
    };

    // 发送工具调用完成通知
    await _sendNotification(TaskExecutionNotification(
      id: nanoid(),
      taskPlanId: '', // 需要从上下文获取
      stepId: step.id,
      type: TaskNotificationType.toolCallCompleted,
      title: '工具调用完成',
      message: '工具 ${step.mcpToolId} 调用成功',
      timestamp: DateTime.now(),
      data: {
        'toolId': step.mcpToolId,
        'result': result,
      },
    ));

    return result;
  }

  /// 使用智能工具选择执行步骤
  Future<Map<String, dynamic>> _executeWithIntelligentToolSelection(
    TaskStep step,
    TaskPlan taskPlan,
    Map<String, dynamic> previousResults,
  ) async {
    Log.info('智能选择工具执行步骤: ${step.id}');
    
    // 分析步骤目标和可用工具，选择最合适的工具
    final selectedTool = await _selectBestTool(step, taskPlan, previousResults);
    
    if (selectedTool == null) {
      throw Exception('无法为步骤 ${step.id} 选择合适的工具');
    }

    Log.info('AI选择了工具: ${selectedTool.name} 用于步骤: ${step.id}');
    
    // 发送工具选择通知
    await _sendNotification(TaskExecutionNotification(
      id: nanoid(),
      taskPlanId: taskPlan.id,
      stepId: step.id,
      type: TaskNotificationType.info,
      title: 'AI工具选择',
      message: 'AI选择了工具"${selectedTool.name}"来完成此步骤',
      timestamp: DateTime.now(),
      data: {
        'selectedTool': selectedTool.name,
        'toolDescription': selectedTool.description,
        'selectionReason': '基于步骤目标和工具能力的智能匹配',
      },
    ));

    // 生成工具调用参数
    final toolParameters = await _generateToolParameters(step, selectedTool, previousResults);
    
    // 调用选定的工具
    return await _callMcpTool(step.mcpEndpointId!, selectedTool.name, toolParameters);
  }

  /// 使用AI助手执行步骤
  Future<Map<String, dynamic>> _executeWithAIAssistant(
    TaskStep step,
    Map<String, dynamic> previousResults,
  ) async {
    Log.info('使用AI助手执行步骤: ${step.id}');
    
    // 这里应该调用AI助手的逻辑
    // 目前返回模拟结果
    return {
      'toolId': 'ai-assistant',
      'status': 'success',
      'output': 'AI助手完成了步骤：${step.description}',
      'reasoning': step.objective,
      'timestamp': DateTime.now().toIso8601String(),
    };
  }

  /// 使用替代工具重试步骤
  Future<Map<String, dynamic>> _retryStepWithAlternativeTool(
    TaskStep step,
    TaskPlan taskPlan,
    Map<String, dynamic> previousResults,
    dynamic originalError,
  ) async {
    Log.info('使用替代工具重试步骤: ${step.id}, 重试次数: ${step.retryCount + 1}');
    
    // 发送重试通知
    await _sendNotification(TaskExecutionNotification(
      id: nanoid(),
      taskPlanId: taskPlan.id,
      stepId: step.id,
      type: TaskNotificationType.warning,
      title: '步骤重试',
      message: '原工具执行失败，正在尝试使用替代工具重试',
      timestamp: DateTime.now(),
      data: {
        'retryCount': step.retryCount + 1,
        'maxRetries': step.maxRetries,
        'originalError': originalError.toString(),
      },
    ));

    // 选择替代工具（排除之前失败的工具）
    final alternativeTool = await _selectAlternativeTool(step, taskPlan, previousResults);
    
    if (alternativeTool == null) {
      throw Exception('无法找到替代工具来重试步骤 ${step.id}');
    }

    // 使用替代工具执行
    final toolParameters = await _generateToolParameters(step, alternativeTool, previousResults);
    return await _callMcpTool(step.mcpEndpointId!, alternativeTool.name, toolParameters);
  }

  /// 选择最佳工具
  Future<McpToolSchema?> _selectBestTool(
    TaskStep step,
    TaskPlan taskPlan,
    Map<String, dynamic> previousResults,
  ) async {
    if (step.availableTools.isEmpty) {
      return null;
    }

    // 这里应该使用AI来分析步骤目标和工具能力，选择最合适的工具
    // 目前使用简化的逻辑：选择第一个可用的工具
    return step.availableTools.first;
  }

  /// 选择替代工具
  Future<McpToolSchema?> _selectAlternativeTool(
    TaskStep step,
    TaskPlan taskPlan,
    Map<String, dynamic> previousResults,
  ) async {
    // 从可用工具中选择一个不同的工具
    if (step.availableTools.length > 1) {
      return step.availableTools[1]; // 选择第二个工具作为替代
    }
    return null;
  }

  /// 生成工具调用参数
  Future<Map<String, dynamic>> _generateToolParameters(
    TaskStep step,
    McpToolSchema tool,
    Map<String, dynamic> previousResults,
  ) async {
    // 这里应该使用AI来根据步骤目标、工具schema和之前的结果生成合适的参数
    // 目前返回基本参数
    return {
      'objective': step.objective,
      'description': step.description,
      'previousResults': previousResults,
      ...step.parameters,
    };
  }

  /// 调用MCP工具
  Future<Map<String, dynamic>> _callMcpTool(
    String endpointId,
    String toolName,
    Map<String, dynamic> parameters,
  ) async {
    try {
      // 这里应该调用实际的MCP工具服务
      // 目前返回模拟结果
      await Future.delayed(const Duration(milliseconds: 500)); // 模拟网络延迟
      
      return {
        'endpointId': endpointId,
        'toolName': toolName,
        'status': 'success',
        'output': '工具 $toolName 在端点 $endpointId 上执行成功',
        'parameters': parameters,
        'timestamp': DateTime.now().toIso8601String(),
      };
    } catch (e) {
      throw Exception('MCP工具调用失败: $endpointId/$toolName - $e');
    }
  }

  /// 判断步骤失败后是否应该继续执行
  bool _shouldContinueAfterStepFailure(TaskStep step, dynamic error) {
    // 这里可以根据步骤的重要性、错误类型等来决定
    // 目前的简化逻辑：如果不是关键步骤，可以继续
    return !step.parameters.containsKey('critical') || 
           step.parameters['critical'] != true;
  }

  /// 发送通知
  Future<void> _sendNotification(TaskExecutionNotification notification) async {
    Log.debug('发送任务执行通知: ${notification.type} - ${notification.message}');
    onNotification?.call(notification);
  }
}
