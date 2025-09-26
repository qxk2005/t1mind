import 'dart:io';
import 'dart:convert';

import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart';
import 'package:appflowy/shared/loading.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/hover.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flowy_infra_ui/widget/spacing.dart';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

/// 执行日志查看器组件
/// 提供完整的日志浏览、搜索、过滤和导出功能
class ExecutionLogViewer extends StatefulWidget {
  const ExecutionLogViewer({
    super.key,
    this.sessionId,
    this.agentId,
    this.onLogSelected,
    this.maxHeight = 600,
    this.showExportButton = true,
    this.showFilterButton = true,
    this.initialFilter,
  });

  /// 会话ID过滤
  final String? sessionId;
  
  /// 智能体ID过滤
  final String? agentId;
  
  /// 日志选择回调
  final ValueChanged<ExecutionLog>? onLogSelected;
  
  /// 最大高度
  final double maxHeight;
  
  /// 是否显示导出按钮
  final bool showExportButton;
  
  /// 是否显示过滤按钮
  final bool showFilterButton;
  
  /// 初始过滤器
  final ExecutionLogFilter? initialFilter;

  @override
  State<ExecutionLogViewer> createState() => _ExecutionLogViewerState();
}

class _ExecutionLogViewerState extends State<ExecutionLogViewer> {
  // 控制器和状态
  final TextEditingController _searchController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  
  // 数据状态
  List<ExecutionLog> _allLogs = [];
  List<ExecutionLog> _filteredLogs = [];
  bool _isLoading = false;
  String? _errorMessage;
  
  // 过滤和排序状态
  ExecutionLogFilter _filter = const ExecutionLogFilter();
  ExecutionLogSortOptions _sortOptions = const ExecutionLogSortOptions();
  String _searchKeyword = '';
  
  // 分页状态
  int _currentPage = 0;
  static const int _pageSize = 20;
  bool _hasMoreData = true;
  
  // 选中状态
  ExecutionLog? _selectedLog;
  final Set<String> _selectedLogIds = {};
  bool _isMultiSelectMode = false;

  @override
  void initState() {
    super.initState();
    _filter = widget.initialFilter ?? const ExecutionLogFilter();
    _searchController.addListener(_onSearchChanged);
    _scrollController.addListener(_onScroll);
    _loadLogs();
  }

  @override
  void dispose() {
    _searchController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _onSearchChanged() {
    final keyword = _searchController.text.trim();
    if (keyword != _searchKeyword) {
      setState(() {
        _searchKeyword = keyword;
      });
      _applyFilters();
    }
  }

  void _onScroll() {
    if (_scrollController.position.pixels >= 
        _scrollController.position.maxScrollExtent - 200) {
      _loadMoreLogs();
    }
  }

  /// 加载日志数据
  Future<void> _loadLogs() async {
    if (_isLoading) return;
    
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      // TODO: 调用实际的日志服务
      // final result = await ExecutionLogService.getLogs(
      //   criteria: ExecutionLogSearchCriteria(
      //     sessionId: widget.sessionId,
      //     agentId: widget.agentId,
      //     keyword: _searchKeyword,
      //     limit: _pageSize,
      //     offset: _currentPage * _pageSize,
      //   ),
      // );
      
      // 模拟数据加载
      await Future.delayed(const Duration(milliseconds: 500));
      final mockLogs = _generateMockLogs();
      
      setState(() {
        if (_currentPage == 0) {
          _allLogs = mockLogs;
        } else {
          _allLogs.addAll(mockLogs);
        }
        _hasMoreData = mockLogs.length == _pageSize;
        _isLoading = false;
      });
      
      _applyFilters();
    } catch (e) {
      setState(() {
        _errorMessage = e.toString();
        _isLoading = false;
      });
    }
  }

  /// 加载更多日志
  Future<void> _loadMoreLogs() async {
    if (!_hasMoreData || _isLoading) return;
    
    _currentPage++;
    await _loadLogs();
  }

  /// 应用过滤器
  void _applyFilters() {
    setState(() {
      _filteredLogs = _allLogs.where((log) {
        // 关键词搜索
        if (_searchKeyword.isNotEmpty) {
          final keyword = _searchKeyword.toLowerCase();
          if (!log.userQuery.toLowerCase().contains(keyword) &&
              !(log.errorMessage?.toLowerCase().contains(keyword) ?? false) &&
              !log.usedMcpTools.any((tool) => tool.toLowerCase().contains(keyword))) {
            return false;
          }
        }
        
        // 状态过滤
        if (_filter.statuses.isNotEmpty && !_filter.statuses.contains(log.status)) {
          return false;
        }
        
        // 错误类型过滤
        if (_filter.errorTypes.isNotEmpty && 
            (log.errorType == null || !_filter.errorTypes.contains(log.errorType))) {
          return false;
        }
        
        // MCP工具过滤
        if (_filter.mcpTools.isNotEmpty && 
            !_filter.mcpTools.any((tool) => log.usedMcpTools.contains(tool))) {
          return false;
        }
        
        // 智能体过滤
        if (_filter.agents.isNotEmpty && 
            (log.agentId == null || !_filter.agents.contains(log.agentId!))) {
          return false;
        }
        
        // 标签过滤
        if (_filter.tags.isNotEmpty && 
            !_filter.tags.any((tag) => log.tags.contains(tag))) {
          return false;
        }
        
        // 执行时间过滤
        if (_filter.minDuration != null || _filter.maxDuration != null) {
          final duration = log.endTime?.difference(log.startTime);
          if (duration != null) {
            if (_filter.minDuration != null && duration < _filter.minDuration!) {
              return false;
            }
            if (_filter.maxDuration != null && duration > _filter.maxDuration!) {
              return false;
            }
          }
        }
        
        // 错误过滤
        if (_filter.hasErrors && log.errorMessage == null) {
          return false;
        }
        
        // 引用过滤
        if (_filter.hasReferences && log.steps.every((step) => step.references.isEmpty)) {
          return false;
        }
        
        return true;
      }).toList();
      
      // 应用排序
      _applySorting();
    });
  }

  /// 应用排序
  void _applySorting() {
    _filteredLogs.sort((a, b) {
      int comparison = 0;
      
      switch (_sortOptions.sortBy) {
        case ExecutionLogSortBy.createdTime:
          comparison = a.startTime.compareTo(b.startTime);
          break;
        case ExecutionLogSortBy.endTime:
          final aEndTime = a.endTime ?? DateTime.now();
          final bEndTime = b.endTime ?? DateTime.now();
          comparison = aEndTime.compareTo(bEndTime);
          break;
        case ExecutionLogSortBy.duration:
          final aDuration = a.endTime?.difference(a.startTime) ?? Duration.zero;
          final bDuration = b.endTime?.difference(b.startTime) ?? Duration.zero;
          comparison = aDuration.compareTo(bDuration);
          break;
        case ExecutionLogSortBy.status:
          comparison = a.status.index.compareTo(b.status.index);
          break;
        case ExecutionLogSortBy.stepCount:
          comparison = a.totalSteps.compareTo(b.totalSteps);
          break;
        case ExecutionLogSortBy.errorType:
          final aError = a.errorType?.index ?? -1;
          final bError = b.errorType?.index ?? -1;
          comparison = aError.compareTo(bError);
          break;
      }
      
      return _sortOptions.direction == ExecutionLogSortDirection.ascending 
          ? comparison 
          : -comparison;
    });
  }

  /// 刷新数据
  Future<void> _refreshLogs() async {
    _currentPage = 0;
    _hasMoreData = true;
    await _loadLogs();
  }

  /// 显示过滤器对话框
  void _showFilterDialog() {
    showDialog(
      context: context,
      builder: (context) => _ExecutionLogFilterDialog(
        filter: _filter,
        onFilterChanged: (newFilter) {
          setState(() {
            _filter = newFilter;
          });
          _applyFilters();
        },
      ),
    );
  }

  /// 显示排序选项对话框
  void _showSortDialog() {
    showDialog(
      context: context,
      builder: (context) => _ExecutionLogSortDialog(
        sortOptions: _sortOptions,
        onSortChanged: (newSortOptions) {
          setState(() {
            _sortOptions = newSortOptions;
          });
          _applySorting();
        },
      ),
    );
  }

  /// 显示导出选项对话框
  void _showExportDialog() {
    showDialog(
      context: context,
      builder: (context) => _ExecutionLogExportDialog(
        logs: _isMultiSelectMode 
            ? _filteredLogs.where((log) => _selectedLogIds.contains(log.id)).toList()
            : _filteredLogs,
        onExport: _exportLogs,
      ),
    );
  }

  /// 导出日志
  Future<void> _exportLogs(
    List<ExecutionLog> logs,
    ExecutionLogExportOptions options,
  ) async {
    final loading = Loading(context);
    loading.start();
    
    try {
      String content;
      String fileName;
      
      switch (options.format) {
        case ExecutionLogExportFormat.json:
          content = _exportToJson(logs, options);
          fileName = 'execution_logs_${DateTime.now().millisecondsSinceEpoch}.json';
          break;
        case ExecutionLogExportFormat.csv:
          content = _exportToCsv(logs, options);
          fileName = 'execution_logs_${DateTime.now().millisecondsSinceEpoch}.csv';
          break;
        case ExecutionLogExportFormat.text:
          content = _exportToText(logs, options);
          fileName = 'execution_logs_${DateTime.now().millisecondsSinceEpoch}.txt';
          break;
        case ExecutionLogExportFormat.html:
          content = _exportToHtml(logs, options);
          fileName = 'execution_logs_${DateTime.now().millisecondsSinceEpoch}.html';
          break;
        default:
          throw UnsupportedError('不支持的导出格式: ${options.format}');
      }
      
      // 保存文件
      final directory = await getApplicationDocumentsDirectory();
      final file = File('${directory.path}/$fileName');
      await file.writeAsString(content);
      
      // 分享文件
      await Share.shareXFiles([XFile(file.path)]);
      
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: FlowyText('导出成功: $fileName'),
            backgroundColor: Theme.of(context).colorScheme.primary,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: FlowyText('导出失败: $e'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    } finally {
      loading.stop();
    }
  }

  /// 切换多选模式
  void _toggleMultiSelectMode() {
    setState(() {
      _isMultiSelectMode = !_isMultiSelectMode;
      if (!_isMultiSelectMode) {
        _selectedLogIds.clear();
      }
    });
  }

  /// 选择/取消选择日志
  void _toggleLogSelection(String logId) {
    setState(() {
      if (_selectedLogIds.contains(logId)) {
        _selectedLogIds.remove(logId);
      } else {
        _selectedLogIds.add(logId);
      }
    });
  }

  /// 全选/取消全选
  void _toggleSelectAll() {
    setState(() {
      if (_selectedLogIds.length == _filteredLogs.length) {
        _selectedLogIds.clear();
      } else {
        _selectedLogIds.clear();
        _selectedLogIds.addAll(_filteredLogs.map((log) => log.id));
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      height: widget.maxHeight,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).dividerColor,
          width: 1,
        ),
      ),
      child: Column(
        children: [
          _buildHeader(),
          const Divider(height: 1),
          _buildSearchBar(),
          const Divider(height: 1),
          Expanded(
            child: _buildLogList(),
          ),
          if (_isMultiSelectMode) ...[
            const Divider(height: 1),
            _buildMultiSelectActions(),
          ],
        ],
      ),
    );
  }

  /// 构建头部
  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Row(
        children: [
          const FlowySvg(FlowySvgs.document_s, size: Size.square(20)),
          const HSpace(8),
          Expanded(
            child: FlowyText.medium(
              '执行日志',
              fontSize: 16,
            ),
          ),
          if (_filteredLogs.isNotEmpty) ...[
            FlowyText.regular(
              '${_filteredLogs.length} 条记录',
              fontSize: 12,
              color: Theme.of(context).hintColor,
            ),
            const HSpace(12),
          ],
          FlowyIconButton(
            icon: FlowySvg(
              _isMultiSelectMode ? FlowySvgs.uncheck_s : FlowySvgs.multiselect_s,
            ),
            tooltipText: _isMultiSelectMode ? '退出多选' : '多选模式',
            onPressed: _toggleMultiSelectMode,
          ),
          const HSpace(4),
          if (widget.showFilterButton)
            FlowyIconButton(
              icon: FlowySvg(
                FlowySvgs.filter_s,
                color: _filter.isEmpty 
                    ? null 
                    : Theme.of(context).colorScheme.primary,
              ),
              tooltipText: '过滤',
              onPressed: _showFilterDialog,
            ),
          const HSpace(4),
          FlowyIconButton(
            icon: const FlowySvg(FlowySvgs.sort_ascending_s),
            tooltipText: '排序',
            onPressed: _showSortDialog,
          ),
          const HSpace(4),
          if (widget.showExportButton)
            FlowyIconButton(
              icon: const FlowySvg(FlowySvgs.share_s),
              tooltipText: '导出',
              onPressed: _showExportDialog,
            ),
          const HSpace(4),
          FlowyIconButton(
            icon: const FlowySvg(FlowySvgs.reload_s),
            tooltipText: '刷新',
            onPressed: _refreshLogs,
          ),
        ],
      ),
    );
  }

  /// 构建搜索栏
  Widget _buildSearchBar() {
    return Container(
      padding: const EdgeInsets.all(12),
      child: FlowyTextField(
        controller: _searchController,
        hintText: '搜索日志...',
        prefixIcon: const Padding(
          padding: EdgeInsets.all(4),
          child: FlowySvg(FlowySvgs.search_s),
        ),
        suffixIcon: _searchController.text.isNotEmpty
            ? FlowyIconButton(
                icon: const FlowySvg(FlowySvgs.close_s),
                onPressed: () {
                  _searchController.clear();
                },
              )
            : null,
      ),
    );
  }

  /// 构建日志列表
  Widget _buildLogList() {
    if (_isLoading && _filteredLogs.isEmpty) {
      return const Center(
        child: CircularProgressIndicator(),
      );
    }

    if (_errorMessage != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const FlowySvg(FlowySvgs.warning_s, size: Size.square(48)),
            const VSpace(16),
            FlowyText.medium('加载失败'),
            const VSpace(8),
            FlowyText.regular(
              _errorMessage!,
              color: Theme.of(context).hintColor,
              maxLines: 3,
            ),
            const VSpace(16),
            FlowyButton(
              text: const FlowyText('重试'),
              onTap: _refreshLogs,
            ),
          ],
        ),
      );
    }

    if (_filteredLogs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const FlowySvg(FlowySvgs.m_empty_page_xl, size: Size.square(48)),
            const VSpace(16),
            FlowyText.medium('暂无日志数据'),
            const VSpace(8),
            FlowyText.regular(
              _searchKeyword.isNotEmpty || !_filter.isEmpty
                  ? '尝试调整搜索条件或过滤器'
                  : '还没有执行日志记录',
              color: Theme.of(context).hintColor,
            ),
          ],
        ),
      );
    }

    return ListView.separated(
      controller: _scrollController,
      itemCount: _filteredLogs.length + (_hasMoreData ? 1 : 0),
      separatorBuilder: (context, index) => const Divider(height: 1),
      itemBuilder: (context, index) {
        if (index >= _filteredLogs.length) {
          return const Padding(
            padding: EdgeInsets.all(16),
            child: Center(child: CircularProgressIndicator()),
          );
        }

        final log = _filteredLogs[index];
        final isSelected = _selectedLog?.id == log.id;
        final isChecked = _selectedLogIds.contains(log.id);

        return _ExecutionLogListItem(
          log: log,
          isSelected: isSelected,
          isMultiSelectMode: _isMultiSelectMode,
          isChecked: isChecked,
          onTap: () {
            if (_isMultiSelectMode) {
              _toggleLogSelection(log.id);
            } else {
              setState(() {
                _selectedLog = isSelected ? null : log;
              });
              widget.onLogSelected?.call(log);
            }
          },
          onLongPress: () {
            if (!_isMultiSelectMode) {
              _toggleMultiSelectMode();
              _toggleLogSelection(log.id);
            }
          },
        );
      },
    );
  }

  /// 构建多选操作栏
  Widget _buildMultiSelectActions() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        children: [
          FlowyButton(
            text: FlowyText(
              _selectedLogIds.length == _filteredLogs.length ? '取消全选' : '全选',
              fontSize: 12,
            ),
            onTap: _toggleSelectAll,
          ),
          const HSpace(12),
          FlowyText.regular(
            '已选择 ${_selectedLogIds.length} 项',
            fontSize: 12,
            color: Theme.of(context).hintColor,
          ),
          const Spacer(),
          if (_selectedLogIds.isNotEmpty) ...[
            FlowyButton(
              text: const FlowyText('导出选中', fontSize: 12),
              onTap: _showExportDialog,
            ),
            const HSpace(8),
            FlowyButton(
              text: const FlowyText('删除选中', fontSize: 12),
              onTap: () {
                // TODO: 实现删除功能
              },
            ),
          ],
        ],
      ),
    );
  }

  /// 生成模拟数据
  List<ExecutionLog> _generateMockLogs() {
    // TODO: 替换为实际的数据加载逻辑
    return List.generate(20, (index) {
      final now = DateTime.now();
      final startTime = now.subtract(Duration(hours: index, minutes: index * 5));
      final endTime = startTime.add(Duration(minutes: 2 + index % 5));
      
      return ExecutionLog(
        id: 'log_${DateTime.now().millisecondsSinceEpoch}_$index',
        sessionId: 'session_${index % 3}',
        userQuery: '用户查询示例 $index',
        startTime: startTime,
        endTime: endTime,
        status: ExecutionLogStatus.values[index % ExecutionLogStatus.values.length],
        errorMessage: index % 4 == 0 ? '示例错误信息 $index' : null,
        errorType: index % 4 == 0 ? ExecutionErrorType.values[index % ExecutionErrorType.values.length] : null,
        agentId: 'agent_${index % 2}',
        totalSteps: 3 + index % 5,
        completedSteps: index % 6,
        usedMcpTools: ['tool_${index % 3}', 'tool_${(index + 1) % 3}'],
        tags: ['tag_${index % 2}'],
        steps: [],
      );
    });
  }

  /// 导出为JSON格式
  String _exportToJson(List<ExecutionLog> logs, ExecutionLogExportOptions options) {
    final data = logs.map((log) => log.toJson()).toList();
    return const JsonEncoder.withIndent('  ').convert(data);
  }

  /// 导出为CSV格式
  String _exportToCsv(List<ExecutionLog> logs, ExecutionLogExportOptions options) {
    final buffer = StringBuffer();
    
    // CSV头部
    buffer.writeln('ID,会话ID,用户查询,开始时间,结束时间,状态,错误信息,智能体ID,总步骤,完成步骤,使用工具');
    
    // CSV数据
    for (final log in logs) {
      final row = [
        log.id,
        log.sessionId,
        '"${log.userQuery.replaceAll('"', '""')}"',
        DateFormat(options.dateFormat).format(log.startTime),
        log.endTime != null ? DateFormat(options.dateFormat).format(log.endTime!) : '',
        log.status.name,
        log.errorMessage != null ? '"${log.errorMessage!.replaceAll('"', '""')}"' : '',
        log.agentId ?? '',
        log.totalSteps.toString(),
        log.completedSteps.toString(),
        '"${log.usedMcpTools.join(', ')}"',
      ];
      buffer.writeln(row.join(','));
    }
    
    return buffer.toString();
  }

  /// 导出为纯文本格式
  String _exportToText(List<ExecutionLog> logs, ExecutionLogExportOptions options) {
    final buffer = StringBuffer();
    
    buffer.writeln('执行日志导出');
    buffer.writeln('导出时间: ${DateFormat(options.dateFormat).format(DateTime.now())}');
    buffer.writeln('记录数量: ${logs.length}');
    buffer.writeln('${'=' * 50}');
    buffer.writeln();
    
    for (int i = 0; i < logs.length; i++) {
      final log = logs[i];
      buffer.writeln('日志 ${i + 1}:');
      buffer.writeln('  ID: ${log.id}');
      buffer.writeln('  会话ID: ${log.sessionId}');
      buffer.writeln('  用户查询: ${log.userQuery}');
      buffer.writeln('  开始时间: ${DateFormat(options.dateFormat).format(log.startTime)}');
      if (log.endTime != null) {
        buffer.writeln('  结束时间: ${DateFormat(options.dateFormat).format(log.endTime!)}');
      }
      buffer.writeln('  状态: ${log.status.name}');
      if (log.errorMessage != null) {
        buffer.writeln('  错误信息: ${log.errorMessage}');
      }
      if (log.agentId != null) {
        buffer.writeln('  智能体ID: ${log.agentId}');
      }
      buffer.writeln('  总步骤: ${log.totalSteps}');
      buffer.writeln('  完成步骤: ${log.completedSteps}');
      if (log.usedMcpTools.isNotEmpty) {
        buffer.writeln('  使用工具: ${log.usedMcpTools.join(', ')}');
      }
      buffer.writeln();
    }
    
    return buffer.toString();
  }

  /// 导出为HTML格式
  String _exportToHtml(List<ExecutionLog> logs, ExecutionLogExportOptions options) {
    final buffer = StringBuffer();
    
    buffer.writeln('<!DOCTYPE html>');
    buffer.writeln('<html>');
    buffer.writeln('<head>');
    buffer.writeln('<meta charset="UTF-8">');
    buffer.writeln('<title>执行日志导出</title>');
    buffer.writeln('<style>');
    buffer.writeln('body { font-family: Arial, sans-serif; margin: 20px; }');
    buffer.writeln('table { border-collapse: collapse; width: 100%; }');
    buffer.writeln('th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }');
    buffer.writeln('th { background-color: #f2f2f2; }');
    buffer.writeln('tr:nth-child(even) { background-color: #f9f9f9; }');
    buffer.writeln('.status-completed { color: green; }');
    buffer.writeln('.status-failed { color: red; }');
    buffer.writeln('.status-running { color: blue; }');
    buffer.writeln('</style>');
    buffer.writeln('</head>');
    buffer.writeln('<body>');
    buffer.writeln('<h1>执行日志导出</h1>');
    buffer.writeln('<p>导出时间: ${DateFormat(options.dateFormat).format(DateTime.now())}</p>');
    buffer.writeln('<p>记录数量: ${logs.length}</p>');
    buffer.writeln('<table>');
    buffer.writeln('<tr>');
    buffer.writeln('<th>ID</th>');
    buffer.writeln('<th>会话ID</th>');
    buffer.writeln('<th>用户查询</th>');
    buffer.writeln('<th>开始时间</th>');
    buffer.writeln('<th>结束时间</th>');
    buffer.writeln('<th>状态</th>');
    buffer.writeln('<th>错误信息</th>');
    buffer.writeln('<th>智能体ID</th>');
    buffer.writeln('<th>总步骤</th>');
    buffer.writeln('<th>完成步骤</th>');
    buffer.writeln('<th>使用工具</th>');
    buffer.writeln('</tr>');
    
    for (final log in logs) {
      buffer.writeln('<tr>');
      buffer.writeln('<td>${log.id}</td>');
      buffer.writeln('<td>${log.sessionId}</td>');
      buffer.writeln('<td>${log.userQuery}</td>');
      buffer.writeln('<td>${DateFormat(options.dateFormat).format(log.startTime)}</td>');
      buffer.writeln('<td>${log.endTime != null ? DateFormat(options.dateFormat).format(log.endTime!) : ''}</td>');
      buffer.writeln('<td class="status-${log.status.name}">${log.status.name}</td>');
      buffer.writeln('<td>${log.errorMessage ?? ''}</td>');
      buffer.writeln('<td>${log.agentId ?? ''}</td>');
      buffer.writeln('<td>${log.totalSteps}</td>');
      buffer.writeln('<td>${log.completedSteps}</td>');
      buffer.writeln('<td>${log.usedMcpTools.join(', ')}</td>');
      buffer.writeln('</tr>');
    }
    
    buffer.writeln('</table>');
    buffer.writeln('</body>');
    buffer.writeln('</html>');
    
    return buffer.toString();
  }
}

/// 执行日志列表项
class _ExecutionLogListItem extends StatelessWidget {
  const _ExecutionLogListItem({
    required this.log,
    required this.isSelected,
    required this.isMultiSelectMode,
    required this.isChecked,
    required this.onTap,
    this.onLongPress,
  });

  final ExecutionLog log;
  final bool isSelected;
  final bool isMultiSelectMode;
  final bool isChecked;
  final VoidCallback onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    return FlowyHover(
      style: HoverStyle(
        hoverColor: Theme.of(context).colorScheme.primary.withOpacity(0.05),
      ),
      child: GestureDetector(
        onTap: onTap,
        onLongPress: onLongPress,
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: isSelected 
                ? Theme.of(context).colorScheme.primary.withOpacity(0.1)
                : null,
          ),
          child: Row(
            children: [
              if (isMultiSelectMode) ...[
                Checkbox(
                  value: isChecked,
                  onChanged: (_) => onTap(),
                ),
                const HSpace(12),
              ],
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        _buildStatusIcon(),
                        const HSpace(8),
                        Expanded(
                          child: FlowyText.medium(
                            log.userQuery,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        FlowyText.regular(
                          _formatDuration(),
                          fontSize: 12,
                          color: Theme.of(context).hintColor,
                        ),
                      ],
                    ),
                    const VSpace(4),
                    Row(
                      children: [
                        FlowyText.regular(
                          DateFormat('MM-dd HH:mm').format(log.startTime),
                          fontSize: 12,
                          color: Theme.of(context).hintColor,
                        ),
                        const HSpace(8),
                        if (log.agentId != null) ...[
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: Theme.of(context).colorScheme.primaryContainer,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: FlowyText.regular(
                              log.agentId!,
                              fontSize: 10,
                            ),
                          ),
                          const HSpace(8),
                        ],
                        FlowyText.regular(
                          '${log.completedSteps}/${log.totalSteps} 步骤',
                          fontSize: 12,
                          color: Theme.of(context).hintColor,
                        ),
                        const Spacer(),
                        if (log.usedMcpTools.isNotEmpty)
                          FlowyText.regular(
                            '${log.usedMcpTools.length} 工具',
                            fontSize: 12,
                            color: Theme.of(context).hintColor,
                          ),
                      ],
                    ),
                    if (log.errorMessage != null) ...[
                      const VSpace(4),
                      FlowyText.regular(
                        log.errorMessage!,
                        fontSize: 12,
                        color: Theme.of(context).colorScheme.error,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ],
                ),
              ),
              const HSpace(8),
              const FlowySvg(FlowySvgs.arrow_right_s, size: Size.square(16)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatusIcon() {
    Color color;
    IconData icon;
    
    switch (log.status) {
      case ExecutionLogStatus.completed:
        color = Colors.green;
        icon = Icons.check_circle;
        break;
      case ExecutionLogStatus.failed:
        color = Colors.red;
        icon = Icons.error;
        break;
      case ExecutionLogStatus.running:
      case ExecutionLogStatus.preparing:
        color = Colors.blue;
        icon = Icons.hourglass_empty;
        break;
      case ExecutionLogStatus.cancelled:
        color = Colors.orange;
        icon = Icons.cancel;
        break;
      case ExecutionLogStatus.timeout:
        color = Colors.red;
        icon = Icons.timer_off;
        break;
      default:
        color = Colors.grey;
        icon = Icons.radio_button_unchecked;
    }
    
    return Icon(
      icon,
      size: 16,
      color: color,
    );
  }

  String _formatDuration() {
    if (log.endTime == null) {
      return log.status.isRunning ? '运行中' : '--';
    }
    
    final duration = log.endTime!.difference(log.startTime);
    if (duration.inHours > 0) {
      return '${duration.inHours}h ${duration.inMinutes % 60}m';
    } else if (duration.inMinutes > 0) {
      return '${duration.inMinutes}m ${duration.inSeconds % 60}s';
    } else {
      return '${duration.inSeconds}s';
    }
  }
}

/// 过滤器对话框
class _ExecutionLogFilterDialog extends StatefulWidget {
  const _ExecutionLogFilterDialog({
    required this.filter,
    required this.onFilterChanged,
  });

  final ExecutionLogFilter filter;
  final ValueChanged<ExecutionLogFilter> onFilterChanged;

  @override
  State<_ExecutionLogFilterDialog> createState() => _ExecutionLogFilterDialogState();
}

class _ExecutionLogFilterDialogState extends State<_ExecutionLogFilterDialog> {
  late ExecutionLogFilter _filter;

  @override
  void initState() {
    super.initState();
    _filter = widget.filter;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const FlowyText.medium('过滤条件'),
      content: SizedBox(
        width: 400,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // 状态过滤
              const FlowyText.medium('状态'),
              const VSpace(8),
              Wrap(
                spacing: 8,
                runSpacing: 4,
                children: ExecutionLogStatus.values.map((status) {
                  final isSelected = _filter.statuses.contains(status);
                  return FilterChip(
                    label: FlowyText(status.name),
                    selected: isSelected,
                    onSelected: (selected) {
                      setState(() {
                        if (selected) {
                          _filter = _filter.copyWith(
                            statuses: [..._filter.statuses, status],
                          );
                        } else {
                          _filter = _filter.copyWith(
                            statuses: _filter.statuses.where((s) => s != status).toList(),
                          );
                        }
                      });
                    },
                  );
                }).toList(),
              ),
              const VSpace(16),
              
              // 错误类型过滤
              const FlowyText.medium('错误类型'),
              const VSpace(8),
              Wrap(
                spacing: 8,
                runSpacing: 4,
                children: ExecutionErrorType.values.map((errorType) {
                  final isSelected = _filter.errorTypes.contains(errorType);
                  return FilterChip(
                    label: FlowyText(errorType.description),
                    selected: isSelected,
                    onSelected: (selected) {
                      setState(() {
                        if (selected) {
                          _filter = _filter.copyWith(
                            errorTypes: [..._filter.errorTypes, errorType],
                          );
                        } else {
                          _filter = _filter.copyWith(
                            errorTypes: _filter.errorTypes.where((e) => e != errorType).toList(),
                          );
                        }
                      });
                    },
                  );
                }).toList(),
              ),
              const VSpace(16),
              
              // 其他选项
              CheckboxListTile(
                title: const FlowyText('仅显示有错误的日志'),
                value: _filter.hasErrors,
                onChanged: (value) {
                  setState(() {
                    _filter = _filter.copyWith(hasErrors: value ?? false);
                  });
                },
              ),
              CheckboxListTile(
                title: const FlowyText('仅显示有引用的日志'),
                value: _filter.hasReferences,
                onChanged: (value) {
                  setState(() {
                    _filter = _filter.copyWith(hasReferences: value ?? false);
                  });
                },
              ),
            ],
          ),
        ),
      ),
      actions: [
        FlowyButton(
          text: const FlowyText('重置'),
          onTap: () {
            setState(() {
              _filter = const ExecutionLogFilter();
            });
          },
        ),
        FlowyButton(
          text: const FlowyText('取消'),
          onTap: () => Navigator.of(context).pop(),
        ),
        FlowyButton(
          text: const FlowyText('确定'),
          onTap: () {
            widget.onFilterChanged(_filter);
            Navigator.of(context).pop();
          },
        ),
      ],
    );
  }
}

/// 排序选项对话框
class _ExecutionLogSortDialog extends StatefulWidget {
  const _ExecutionLogSortDialog({
    required this.sortOptions,
    required this.onSortChanged,
  });

  final ExecutionLogSortOptions sortOptions;
  final ValueChanged<ExecutionLogSortOptions> onSortChanged;

  @override
  State<_ExecutionLogSortDialog> createState() => _ExecutionLogSortDialogState();
}

class _ExecutionLogSortDialogState extends State<_ExecutionLogSortDialog> {
  late ExecutionLogSortOptions _sortOptions;

  @override
  void initState() {
    super.initState();
    _sortOptions = widget.sortOptions;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const FlowyText.medium('排序选项'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const FlowyText.medium('排序字段'),
          const VSpace(8),
          ...ExecutionLogSortBy.values.map((sortBy) {
            return RadioListTile<ExecutionLogSortBy>(
              title: FlowyText(sortBy.displayName),
              value: sortBy,
              groupValue: _sortOptions.sortBy,
              onChanged: (value) {
                if (value != null) {
                  setState(() {
                    _sortOptions = _sortOptions.copyWith(sortBy: value);
                  });
                }
              },
            );
          }),
          const VSpace(16),
          const FlowyText.medium('排序方向'),
          const VSpace(8),
          ...ExecutionLogSortDirection.values.map((direction) {
            return RadioListTile<ExecutionLogSortDirection>(
              title: FlowyText(direction.displayName),
              value: direction,
              groupValue: _sortOptions.direction,
              onChanged: (value) {
                if (value != null) {
                  setState(() {
                    _sortOptions = _sortOptions.copyWith(direction: value);
                  });
                }
              },
            );
          }),
        ],
      ),
      actions: [
        FlowyButton(
          text: const FlowyText('取消'),
          onTap: () => Navigator.of(context).pop(),
        ),
        FlowyButton(
          text: const FlowyText('确定'),
          onTap: () {
            widget.onSortChanged(_sortOptions);
            Navigator.of(context).pop();
          },
        ),
      ],
    );
  }
}

/// 导出选项对话框
class _ExecutionLogExportDialog extends StatefulWidget {
  const _ExecutionLogExportDialog({
    required this.logs,
    required this.onExport,
  });

  final List<ExecutionLog> logs;
  final Function(List<ExecutionLog>, ExecutionLogExportOptions) onExport;

  @override
  State<_ExecutionLogExportDialog> createState() => _ExecutionLogExportDialogState();
}

class _ExecutionLogExportDialogState extends State<_ExecutionLogExportDialog> {
  ExecutionLogExportOptions _options = const ExecutionLogExportOptions(
    format: ExecutionLogExportFormat.json,
  );

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const FlowyText.medium('导出选项'),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            FlowyText.regular('将导出 ${widget.logs.length} 条日志记录'),
            const VSpace(16),
            const FlowyText.medium('导出格式'),
            const VSpace(8),
            ...ExecutionLogExportFormat.values.map((format) {
              return RadioListTile<ExecutionLogExportFormat>(
                title: FlowyText(format.displayName),
                value: format,
                groupValue: _options.format,
                onChanged: (value) {
                  if (value != null) {
                    setState(() {
                      _options = _options.copyWith(format: value);
                    });
                  }
                },
              );
            }),
            const VSpace(16),
            const FlowyText.medium('包含内容'),
            CheckboxListTile(
              title: const FlowyText('包含执行步骤'),
              value: _options.includeSteps,
              onChanged: (value) {
                setState(() {
                  _options = _options.copyWith(includeSteps: value ?? false);
                });
              },
            ),
            CheckboxListTile(
              title: const FlowyText('包含引用信息'),
              value: _options.includeReferences,
              onChanged: (value) {
                setState(() {
                  _options = _options.copyWith(includeReferences: value ?? false);
                });
              },
            ),
            CheckboxListTile(
              title: const FlowyText('包含元数据'),
              value: _options.includeMetadata,
              onChanged: (value) {
                setState(() {
                  _options = _options.copyWith(includeMetadata: value ?? false);
                });
              },
            ),
            CheckboxListTile(
              title: const FlowyText('包含错误详情'),
              value: _options.includeErrorDetails,
              onChanged: (value) {
                setState(() {
                  _options = _options.copyWith(includeErrorDetails: value ?? false);
                });
              },
            ),
          ],
        ),
      ),
      actions: [
        FlowyButton(
          text: const FlowyText('取消'),
          onTap: () => Navigator.of(context).pop(),
        ),
        FlowyButton(
          text: const FlowyText('导出'),
          onTap: () {
            widget.onExport(widget.logs, _options);
            Navigator.of(context).pop();
          },
        ),
      ],
    );
  }
}
