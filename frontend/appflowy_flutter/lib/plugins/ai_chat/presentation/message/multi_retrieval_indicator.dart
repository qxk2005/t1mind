import 'package:flutter/material.dart';
import 'dart:convert';

/// 多路召回进度指示器
/// 
/// 用于显示多路召回的执行过程：
/// - 任务分解阶段
/// - 并行执行阶段
/// - 结果综合阶段
class MultiRetrievalIndicator extends StatefulWidget {
  const MultiRetrievalIndicator({super.key});

  @override
  State<MultiRetrievalIndicator> createState() => _MultiRetrievalIndicatorState();
}

class _MultiRetrievalIndicatorState extends State<MultiRetrievalIndicator> {
  String _currentPhase = '';
  int _totalTasks = 0;
  int _successCount = 0;
  int _failureCount = 0;
  List<SubTaskResultData> _subTaskResults = [];
  bool _isCompleted = false;

  void updateFromMetadata(Map<String, dynamic> metadata) {
    setState(() {
      if (metadata.containsKey('multi_retrieval')) {
        final multiRetrieval = metadata['multi_retrieval'];
        _currentPhase = multiRetrieval['phase'] ?? '';

        switch (_currentPhase) {
          case 'decomposition':
            _totalTasks = multiRetrieval['sub_tasks_count'] ?? 0;
            break;

          case 'execution':
            _successCount = multiRetrieval['success_count'] ?? 0;
            _failureCount = multiRetrieval['failure_count'] ?? 0;
            _totalTasks = multiRetrieval['total_tasks'] ?? 0;
            break;

          case 'completed':
            _isCompleted = true;
            break;
        }
      }

      if (metadata.containsKey('sub_task_result')) {
        final result = metadata['sub_task_result'];
        _subTaskResults.add(SubTaskResultData.fromJson(result));
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_isCompleted && _subTaskResults.isEmpty) {
      return const SizedBox.shrink();
    }

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // 标题和状态
          _buildHeader(context),
          const SizedBox(height: 12),

          // 进度条
          if (!_isCompleted) _buildProgressBar(context),
          
          // 子任务列表
          if (_subTaskResults.isNotEmpty) ...[
            const SizedBox(height: 12),
            _buildSubTaskList(context),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    String title;
    IconData icon;
    Color color;

    switch (_currentPhase) {
      case 'decomposition':
        title = '任务分解中...';
        icon = Icons.splitscreen;
        color = Colors.blue;
        break;
      case 'execution':
        title = '并行执行中...';
        icon = Icons.play_circle_outline;
        color = Colors.orange;
        break;
      case 'synthesis':
        title = '综合结果中...';
        icon = Icons.merge_type;
        color = Colors.purple;
        break;
      case 'completed':
        title = '多路召回完成';
        icon = Icons.check_circle;
        color = Colors.green;
        break;
      default:
        title = '多路召回';
        icon = Icons.auto_awesome;
        color = Colors.grey;
    }

    return Row(
      children: [
        Icon(icon, size: 20, color: color),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            title,
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
              fontWeight: FontWeight.w600,
              color: color,
            ),
          ),
        ),
        if (_totalTasks > 0)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: color.withOpacity(0.2),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(
              '$_successCount/$_totalTasks',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: color,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildProgressBar(BuildContext context) {
    final progress = _totalTasks > 0 ? _successCount / _totalTasks : 0.0;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LinearProgressIndicator(
          value: progress,
          backgroundColor: Theme.of(context).colorScheme.surfaceVariant,
          valueColor: AlwaysStoppedAnimation<Color>(
            Theme.of(context).colorScheme.primary,
          ),
        ),
        if (_totalTasks > 0) ...[
          const SizedBox(height: 4),
          Text(
            '${(_successCount)} 个任务完成${_failureCount > 0 ? '，${_failureCount} 个失败' : ''}',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      ],
    );
  }

  Widget _buildSubTaskList(BuildContext context) {
    return ExpansionTile(
      title: Text(
        '任务详情 (${_subTaskResults.length})',
        style: Theme.of(context).textTheme.bodyMedium,
      ),
      initiallyExpanded: false,
      children: _subTaskResults.map((result) {
        return _buildSubTaskItem(context, result);
      }).toList(),
    );
  }

  Widget _buildSubTaskItem(BuildContext context, SubTaskResultData result) {
    final icon = result.success
        ? Icons.check_circle
        : Icons.error;
    final iconColor = result.success ? Colors.green : Colors.red;

    return ListTile(
      dense: true,
      leading: Icon(icon, size: 20, color: iconColor),
      title: Text(
        result.description,
        style: Theme.of(context).textTheme.bodySmall,
      ),
      subtitle: Text(
        '${result.toolUsed} • ${result.durationMs}ms',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(
          color: Theme.of(context).colorScheme.onSurfaceVariant,
          fontSize: 11,
        ),
      ),
      trailing: result.success
          ? Icon(
              Icons.visibility,
              size: 18,
              color: Theme.of(context).colorScheme.primary,
            )
          : result.error != null
              ? Tooltip(
                  message: result.error!,
                  child: Icon(
                    Icons.info_outline,
                    size: 18,
                    color: iconColor,
                  ),
                )
              : null,
    );
  }
}

/// 子任务结果数据模型
class SubTaskResultData {
  final String taskId;
  final String description;
  final String toolUsed;
  final bool success;
  final String source;
  final int durationMs;
  final String? error;

  SubTaskResultData({
    required this.taskId,
    required this.description,
    required this.toolUsed,
    required this.success,
    required this.source,
    required this.durationMs,
    this.error,
  });

  factory SubTaskResultData.fromJson(Map<String, dynamic> json) {
    return SubTaskResultData(
      taskId: json['task_id'] ?? '',
      description: json['task_description'] ?? '',
      toolUsed: json['tool_used'] ?? '',
      success: json['success'] ?? false,
      source: json['source'] ?? '',
      durationMs: json['duration_ms'] ?? 0,
      error: json['error'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'task_id': taskId,
      'task_description': description,
      'tool_used': toolUsed,
      'success': success,
      'source': source,
      'duration_ms': durationMs,
      if (error != null) 'error': error,
    };
  }
}

/// 多路召回状态管理器
/// 
/// 用于在消息流中管理多路召回的状态
class MultiRetrievalStateManager {
  final Map<String, MultiRetrievalState> _states = {};

  MultiRetrievalState? getState(String messageId) {
    return _states[messageId];
  }

  void updateState(String messageId, Map<String, dynamic> metadata) {
    if (!_states.containsKey(messageId)) {
      _states[messageId] = MultiRetrievalState();
    }
    _states[messageId]!.updateFromMetadata(metadata);
  }

  void clearState(String messageId) {
    _states.remove(messageId);
  }

  void clearAll() {
    _states.clear();
  }
}

/// 多路召回状态
class MultiRetrievalState {
  String currentPhase = '';
  int totalTasks = 0;
  int successCount = 0;
  int failureCount = 0;
  List<SubTaskResultData> subTaskResults = [];
  bool isCompleted = false;
  String? reasoning;

  void updateFromMetadata(Map<String, dynamic> metadata) {
    if (metadata.containsKey('multi_retrieval')) {
      final multiRetrieval = metadata['multi_retrieval'];
      currentPhase = multiRetrieval['phase'] ?? '';

      switch (currentPhase) {
        case 'decomposition':
          totalTasks = multiRetrieval['sub_tasks_count'] ?? 0;
          reasoning = multiRetrieval['reasoning'];
          break;

        case 'execution':
          successCount = multiRetrieval['success_count'] ?? 0;
          failureCount = multiRetrieval['failure_count'] ?? 0;
          totalTasks = multiRetrieval['total_tasks'] ?? 0;
          break;

        case 'completed':
          isCompleted = true;
          break;
      }
    }

    if (metadata.containsKey('sub_task_result')) {
      final result = metadata['sub_task_result'];
      subTaskResults.add(SubTaskResultData.fromJson(result));
    }
  }

  double get progress {
    if (totalTasks == 0) return 0.0;
    return successCount / totalTasks;
  }

  String get statusText {
    if (isCompleted) return '完成';
    if (currentPhase == 'synthesis') return '综合中';
    if (currentPhase == 'execution') {
      return '$successCount/$totalTasks 完成';
    }
    return '处理中';
  }
}

