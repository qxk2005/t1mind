import 'dart:async';
import 'dart:io';

import 'package:appflowy/workspace/presentation/home/menu/sidebar/import/conversion_progress_dialog.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

/// 转换日志查看对话框
/// 
/// 显示文档转换过程的详细日志，支持：
/// - 实时日志更新
/// - 按级别、任务过滤
/// - 关键词搜索
/// - 日志导出功能
/// - 高性能列表展示
class ConversionLogsDialog extends StatefulWidget {
  const ConversionLogsDialog({
    super.key,
    required this.tasks,
    this.onClose,
  });

  final List<ConversionTask> tasks;
  final VoidCallback? onClose;

  @override
  State<ConversionLogsDialog> createState() => _ConversionLogsDialogState();
}

class _ConversionLogsDialogState extends State<ConversionLogsDialog> {
  final TextEditingController _searchController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  Timer? _searchDebounce;
  
  // 过滤状态
  LogLevel? _selectedLevel;
  String? _selectedTaskId;
  bool _autoScroll = true;
  
  // 日志数据
  List<ConversionLogEntry> _allLogs = [];
  List<ConversionLogEntry> _filteredLogs = [];
  
  // 分页控制
  static const int _pageSize = 50;
  int _currentPage = 0;
  bool _hasMoreLogs = true;

  @override
  void initState() {
    super.initState();
    _searchController.addListener(_onSearchChanged);
    _scrollController.addListener(_onScrollChanged);
    _generateLogsFromTasks();
  }

  @override
  void dispose() {
    _searchDebounce?.cancel();
    _searchController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _generateLogsFromTasks() {
    final logs = <ConversionLogEntry>[];
    
    for (final task in widget.tasks) {
      // 添加任务开始日志
      logs.add(ConversionLogEntry(
        id: '${task.fileName}_start',
        taskId: task.fileName,
        level: LogLevel.info,
        message: '开始转换文件: ${task.fileName}',
        timestamp: task.startTime ?? DateTime.now(),
        details: '文件路径: ${task.filePath}\n导入类型: ${task.importType.toString()}',
      ));
      
      // 根据任务状态添加相应日志
      switch (task.status) {
        case ConversionTaskStatus.processing:
          logs.add(ConversionLogEntry(
            id: '${task.fileName}_processing',
            taskId: task.fileName,
            level: LogLevel.info,
            message: '正在转换文件: ${task.fileName}',
            timestamp: DateTime.now(),
            details: '进度: ${(task.progress * 100).toStringAsFixed(1)}%',
          ));
          break;
        case ConversionTaskStatus.completed:
          logs.add(ConversionLogEntry(
            id: '${task.fileName}_completed',
            taskId: task.fileName,
            level: LogLevel.success,
            message: '文件转换完成: ${task.fileName}',
            timestamp: task.endTime ?? DateTime.now(),
            details: '转换成功完成',
          ));
          break;
        case ConversionTaskStatus.error:
          logs.add(ConversionLogEntry(
            id: '${task.fileName}_error',
            taskId: task.fileName,
            level: LogLevel.error,
            message: '文件转换失败: ${task.fileName}',
            timestamp: task.endTime ?? DateTime.now(),
            details: task.error ?? '未知错误',
          ));
          break;
        case ConversionTaskStatus.cancelled:
          logs.add(ConversionLogEntry(
            id: '${task.fileName}_cancelled',
            taskId: task.fileName,
            level: LogLevel.warning,
            message: '文件转换已取消: ${task.fileName}',
            timestamp: task.endTime ?? DateTime.now(),
            details: '用户取消了转换操作',
          ));
          break;
        case ConversionTaskStatus.pending:
          logs.add(ConversionLogEntry(
            id: '${task.fileName}_pending',
            taskId: task.fileName,
            level: LogLevel.info,
            message: '文件等待转换: ${task.fileName}',
            timestamp: DateTime.now(),
            details: '文件已添加到转换队列',
          ));
          break;
      }
    }
    
    // 按时间排序
    logs.sort((a, b) => a.timestamp.compareTo(b.timestamp));
    
    setState(() {
      _allLogs = logs;
      _applyFilters();
    });
  }

  void _onSearchChanged() {
    _searchDebounce?.cancel();
    _searchDebounce = Timer(const Duration(milliseconds: 300), () {
      if (mounted) {
        _applyFilters();
      }
    });
  }

  void _onScrollChanged() {
    if (_autoScroll && _scrollController.hasClients) {
      // 自动滚动到底部
      _scrollController.animateTo(
        _scrollController.position.maxScrollExtent,
        duration: const Duration(milliseconds: 200),
        curve: Curves.easeOut,
      );
    }
  }

  void _applyFilters() {
    final searchQuery = _searchController.text.toLowerCase();
    
    // 重置分页
    _currentPage = 0;
    _hasMoreLogs = true;
    
    final allFilteredLogs = _allLogs.where((log) {
      // 级别过滤
      if (_selectedLevel != null && log.level != _selectedLevel) {
        return false;
      }
      
      // 任务过滤
      if (_selectedTaskId != null && log.taskId != _selectedTaskId) {
        return false;
      }
      
      // 搜索过滤
      if (searchQuery.isNotEmpty) {
        final messageMatch = log.message.toLowerCase().contains(searchQuery);
        final detailsMatch = log.details.toLowerCase().contains(searchQuery);
        if (!messageMatch && !detailsMatch) {
          return false;
        }
      }
      
      return true;
    }).toList();
    
    // 应用分页
    final endIndex = (_currentPage + 1) * _pageSize;
    _filteredLogs = allFilteredLogs.take(endIndex).toList();
    _hasMoreLogs = endIndex < allFilteredLogs.length;
    
    setState(() {});
  }
  
  void _loadMoreLogs() {
    if (!_hasMoreLogs) return;
    
    final searchQuery = _searchController.text.toLowerCase();
    final allFilteredLogs = _allLogs.where((log) {
      // 级别过滤
      if (_selectedLevel != null && log.level != _selectedLevel) {
        return false;
      }
      
      // 任务过滤
      if (_selectedTaskId != null && log.taskId != _selectedTaskId) {
        return false;
      }
      
      // 搜索过滤
      if (searchQuery.isNotEmpty) {
        final messageMatch = log.message.toLowerCase().contains(searchQuery);
        final detailsMatch = log.details.toLowerCase().contains(searchQuery);
        if (!messageMatch && !detailsMatch) {
          return false;
        }
      }
      
      return true;
    }).toList();
    
    _currentPage++;
    final endIndex = (_currentPage + 1) * _pageSize;
    _filteredLogs = allFilteredLogs.take(endIndex).toList();
    _hasMoreLogs = endIndex < allFilteredLogs.length;
    
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return FlowyDialog(
      backgroundColor: Theme.of(context).colorScheme.surface,
      title: FlowyText.semibold(
        '转换日志',
        fontSize: 20,
        color: Theme.of(context).colorScheme.tertiary,
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(
          vertical: 10.0,
          horizontal: 20.0,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildToolbar(),
            const VSpace(16),
            _buildFilters(),
            const VSpace(16),
            _buildLogList(),
            const VSpace(16),
            _buildActionButtons(),
          ],
        ),
      ),
    );
  }

  Widget _buildToolbar() {
    return Row(
      children: [
        Expanded(
          child: FlowyTextField(
            controller: _searchController,
            hintText: '搜索日志...',
            prefixIcon: const Icon(Icons.search, size: 16),
          ),
        ),
        const HSpace(12),
        OutlinedRoundedButton(
          text: '导出日志',
          onTap: _exportLogs,
        ),
        const HSpace(12),
        OutlinedRoundedButton(
          text: '清除日志',
          onTap: _clearLogs,
        ),
      ],
    );
  }

  Widget _buildFilters() {
    return Wrap(
      spacing: 12,
      runSpacing: 8,
      alignment: WrapAlignment.start,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        _buildLevelFilter(),
        _buildTaskFilter(),
        _buildAutoScrollToggle(),
      ],
    );
  }

  Widget _buildLevelFilter() {
    return PopupMenuButton<LogLevel?>(
      initialValue: _selectedLevel,
      onSelected: (level) {
        setState(() {
          _selectedLevel = level;
          _applyFilters();
        });
      },
      child: _buildFilterChip(
        _selectedLevel?.displayName ?? '所有级别',
        Icons.filter_list,
      ),
      itemBuilder: (context) => [
        const PopupMenuItem(
          value: null,
          child: Text('所有级别'),
        ),
        ...LogLevel.values.map((level) => PopupMenuItem(
          value: level,
          child: Row(
            children: [
              Icon(level.icon, size: 16, color: level.color),
              const SizedBox(width: 8),
              Text(level.displayName),
            ],
          ),
        )),
      ],
    );
  }

  Widget _buildTaskFilter() {
    final taskIds = widget.tasks.map((task) => task.fileName).toSet().toList();
    
    return PopupMenuButton<String?>(
      initialValue: _selectedTaskId,
      onSelected: (taskId) {
        setState(() {
          _selectedTaskId = taskId;
          _applyFilters();
        });
      },
      child: _buildFilterChip(
        _selectedTaskId ?? '所有任务',
        Icons.task,
      ),
      itemBuilder: (context) => [
        const PopupMenuItem(
          value: null,
          child: Text('所有任务'),
        ),
        ...taskIds.map((taskId) => PopupMenuItem<String?>(
          value: taskId,
          child: Text(taskId),
        )),
      ],
    );
  }

  Widget _buildAutoScrollToggle() {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        FlowyText.regular(
          '自动滚动',
          fontSize: 12,
        ),
        const HSpace(4),
        Switch(
          value: _autoScroll,
          onChanged: (value) {
            setState(() {
              _autoScroll = value;
            });
          },
        ),
      ],
    );
  }

  Widget _buildFilterChip(String text, IconData icon) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Theme.of(context).dividerColor, width: 0.8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 16),
          const HSpace(4),
          Text(text, style: const TextStyle(fontSize: 12)),
          const HSpace(4),
          const Icon(Icons.keyboard_arrow_down, size: 16),
        ],
      ),
    );
  }

  Widget _buildLogList() {
    return Container(
      constraints: const BoxConstraints(maxHeight: 400),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerLow,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
        ),
      ),
      child: _filteredLogs.isEmpty
          ? Center(
              child: FlowyText.regular(
                '没有找到匹配的日志',
                fontSize: 14,
                color: Theme.of(context).hintColor,
              ),
            )
          : ListView.builder(
              controller: _scrollController,
              padding: const EdgeInsets.all(8),
              itemCount: _filteredLogs.length + (_hasMoreLogs ? 1 : 0),
              // 性能优化：添加缓存范围
              cacheExtent: 200,
              // 性能优化：添加项目高度估算
              itemExtent: null, // 让Flutter自动计算
              itemBuilder: (context, index) {
                // 加载更多按钮
                if (index == _filteredLogs.length) {
                  return _buildLoadMoreButton();
                }
                
                final log = _filteredLogs[index];
                return ConversionLogItem(
                  key: ValueKey(log.id), // 添加key以提高性能
                  log: log,
                  searchQuery: _searchController.text,
                );
              },
            ),
    );
  }

  Widget _buildLoadMoreButton() {
    return Container(
      margin: const EdgeInsets.all(8),
      child: OutlinedRoundedButton(
        text: '加载更多日志',
        onTap: _loadMoreLogs,
      ),
    );
  }

  Widget _buildActionButtons() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        OutlinedRoundedButton(
          text: '关闭',
          onTap: widget.onClose,
        ),
      ],
    );
  }

  Future<void> _exportLogs() async {
    try {
      final logsText = _filteredLogs.map((log) => log.toExportString()).join('\n\n');
      final timestamp = DateTime.now().toIso8601String().replaceAll(':', '-');
      final fileName = 'conversion_logs_$timestamp.txt';
      
      if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
        // 桌面平台：保存到下载目录
        final directory = await getDownloadsDirectory() ?? await getApplicationDocumentsDirectory();
        final file = File('${directory.path}/$fileName');
        await file.writeAsString(logsText);
        
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('日志已导出到: ${file.path}'),
              duration: const Duration(seconds: 3),
            ),
          );
        }
      } else {
        // 移动平台：保存到应用目录
        final directory = await getApplicationDocumentsDirectory();
        final file = File('${directory.path}/$fileName');
        await file.writeAsString(logsText);
        
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('日志已保存到: ${file.path}'),
              duration: const Duration(seconds: 3),
            ),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('导出失败: $e'),
            backgroundColor: Theme.of(context).colorScheme.error,
            duration: const Duration(seconds: 3),
          ),
        );
      }
    }
  }

  void _clearLogs() {
    setState(() {
      _allLogs.clear();
      _filteredLogs.clear();
      _searchController.clear();
      _selectedLevel = null;
      _selectedTaskId = null;
      _currentPage = 0;
      _hasMoreLogs = true;
    });
  }
}

/// 转换日志项组件
class ConversionLogItem extends StatelessWidget {
  const ConversionLogItem({
    super.key,
    required this.log,
    this.searchQuery,
  });

  final ConversionLogEntry log;
  final String? searchQuery;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: log.level.color.withOpacity(0.3),
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildHeader(context),
          const VSpace(8),
          _buildMessage(context),
          if (log.details.isNotEmpty) ...[
            const VSpace(8),
            _buildDetails(context),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Row(
      children: [
        Icon(
          log.level.icon,
          size: 16,
          color: log.level.color,
        ),
        const HSpace(8),
        Expanded(
          child: FlowyText.medium(
            log.message,
            fontSize: 14,
            color: Theme.of(context).colorScheme.onSurface,
          ),
        ),
        FlowyText.regular(
          _formatTimestamp(log.timestamp),
          fontSize: 12,
          color: Theme.of(context).hintColor,
        ),
      ],
    );
  }

  Widget _buildMessage(BuildContext context) {
    return FlowyText.regular(
      log.message,
      fontSize: 13,
      color: Theme.of(context).colorScheme.onSurface,
    );
  }

  Widget _buildDetails(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(4),
      ),
      child: FlowyText.regular(
        log.details,
        fontSize: 12,
        color: Theme.of(context).colorScheme.onSurfaceVariant,
      ),
    );
  }

  String _formatTimestamp(DateTime timestamp) {
    final now = DateTime.now();
    final difference = now.difference(timestamp);
    
    if (difference.inMinutes < 1) {
      return '刚刚';
    } else if (difference.inHours < 1) {
      return '${difference.inMinutes}分钟前';
    } else if (difference.inDays < 1) {
      return '${difference.inHours}小时前';
    } else {
      return '${timestamp.month}/${timestamp.day} ${timestamp.hour.toString().padLeft(2, '0')}:${timestamp.minute.toString().padLeft(2, '0')}';
    }
  }
}

/// 日志级别枚举
enum LogLevel {
  debug,
  info,
  warning,
  error,
  success;

  String get displayName {
    switch (this) {
      case LogLevel.debug:
        return '调试';
      case LogLevel.info:
        return '信息';
      case LogLevel.warning:
        return '警告';
      case LogLevel.error:
        return '错误';
      case LogLevel.success:
        return '成功';
    }
  }

  IconData get icon {
    switch (this) {
      case LogLevel.debug:
        return Icons.bug_report;
      case LogLevel.info:
        return Icons.info;
      case LogLevel.warning:
        return Icons.warning;
      case LogLevel.error:
        return Icons.error;
      case LogLevel.success:
        return Icons.check_circle;
    }
  }

  Color get color {
    switch (this) {
      case LogLevel.debug:
        return Colors.grey;
      case LogLevel.info:
        return Colors.blue;
      case LogLevel.warning:
        return Colors.orange;
      case LogLevel.error:
        return Colors.red;
      case LogLevel.success:
        return Colors.green;
    }
  }
}

/// 转换日志条目
class ConversionLogEntry {
  const ConversionLogEntry({
    required this.id,
    required this.taskId,
    required this.level,
    required this.message,
    required this.timestamp,
    this.details = '',
  });

  final String id;
  final String taskId;
  final LogLevel level;
  final String message;
  final DateTime timestamp;
  final String details;

  String toExportString() {
    final buffer = StringBuffer();
    buffer.writeln('时间: ${timestamp.toIso8601String()}');
    buffer.writeln('任务: $taskId');
    buffer.writeln('级别: ${level.displayName}');
    buffer.writeln('消息: $message');
    if (details.isNotEmpty) {
      buffer.writeln('详情: $details');
    }
    buffer.writeln('---');
    return buffer.toString();
  }
}

/// 显示转换日志对话框
Future<void> showConversionLogsDialog(
  BuildContext context, {
  required List<ConversionTask> tasks,
  VoidCallback? onClose,
}) async {
  await FlowyOverlay.show(
    context: context,
    builder: (context) => ConversionLogsDialog(
      tasks: tasks,
      onClose: onClose,
    ),
  );
}
