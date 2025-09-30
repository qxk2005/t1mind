import 'dart:async';

import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_bloc.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 执行日志查看器
/// 
/// 用于展示智能体执行过程的详细日志，支持：
/// - 实时日志更新
/// - 按阶段、状态过滤
/// - 关键词搜索
/// - 高性能列表展示
class ExecutionLogViewer extends StatefulWidget {
  const ExecutionLogViewer({
    super.key,
    required this.sessionId,
    this.messageId,
    this.height = 400,
    this.showHeader = true,
  });

  /// 会话ID
  final String sessionId;
  
  /// 消息ID（可选，用于过滤特定消息的日志）
  final String? messageId;
  
  /// 查看器高度
  final double height;
  
  /// 是否显示头部
  final bool showHeader;

  @override
  State<ExecutionLogViewer> createState() => _ExecutionLogViewerState();
}

class _ExecutionLogViewerState extends State<ExecutionLogViewer> {
  ExecutionLogBloc? _bloc;
  final TextEditingController _searchController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  Timer? _searchDebounce;

  @override
  void initState() {
    super.initState();
    
    // 监听搜索输入变化
    _searchController.addListener(_onSearchChanged);
    
    // 监听滚动到底部加载更多
    _scrollController.addListener(_onScrollChanged);
  }

  @override
  void dispose() {
    _searchDebounce?.cancel();
    _searchController.dispose();
    _scrollController.dispose();
    // 不要在这里关闭BLoC，因为它可能由外部管理
    super.dispose();
  }

  ExecutionLogBloc get bloc {
    return _bloc ?? context.read<ExecutionLogBloc>();
  }

  void _onSearchChanged() {
    _searchDebounce?.cancel();
    _searchDebounce = Timer(const Duration(milliseconds: 300), () {
      bloc.add(ExecutionLogEvent.searchLogs(_searchController.text));
    });
  }

  void _onScrollChanged() {
    if (_scrollController.position.pixels >= 
        _scrollController.position.maxScrollExtent - 100) {
      bloc.add(const ExecutionLogEvent.loadMoreLogs());
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      height: widget.height,
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
          if (widget.showHeader) _buildHeader(),
          _buildFilterBar(),
          Expanded(child: _buildLogList()),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(8),
          topRight: Radius.circular(8),
        ),
      ),
      child: Row(
        children: [
          const FlowySvg(FlowySvgs.ai_summary_generate_s),
          const HSpace(8),
          FlowyText.medium(
            '执行日志',
            fontSize: 14,
          ),
          const Spacer(),
          BlocBuilder<ExecutionLogBloc, ExecutionLogState>(
            builder: (context, state) {
              return FlowyText.regular(
                '共 ${state.totalCount} 条日志',
                fontSize: 12,
                color: Theme.of(context).hintColor,
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildFilterBar() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).dividerColor,
            width: 0.5,
          ),
        ),
      ),
      child: Column(
        children: [
          // 搜索框
          Row(
            children: [
              Expanded(
                child:                 FlowyTextField(
                  controller: _searchController,
                  hintText: '搜索日志内容...',
                  prefixIcon: const FlowySvg(FlowySvgs.search_s),
                  suffixIcon: _searchController.text.isNotEmpty
                      ? FlowyIconButton(
                          icon: const FlowySvg(FlowySvgs.close_s),
                          onPressed: () {
                            _searchController.clear();
                          },
                        )
                      : null,
                ),
              ),
              const HSpace(8),
              // 刷新按钮
              FlowyIconButton(
                icon: const Icon(Icons.refresh),
                onPressed: () {
                  bloc.add(const ExecutionLogEvent.refreshLogs());
                },
              ),
            ],
          ),
          const VSpace(8),
          // 过滤器
          _buildFilters(),
        ],
      ),
    );
  }

  Widget _buildFilters() {
    return BlocBuilder<ExecutionLogBloc, ExecutionLogState>(
      builder: (context, state) {
        return Row(
          children: [
            // 阶段过滤
            _buildPhaseFilter(state),
            const HSpace(12),
            // 状态过滤
            _buildStatusFilter(state),
            const Spacer(),
            // 自动滚动开关
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                FlowyText.regular(
                  '自动滚动',
                  fontSize: 12,
                ),
                const HSpace(4),
                Switch(
                  value: state.autoScroll,
                  onChanged: (value) {
                    bloc.add(ExecutionLogEvent.toggleAutoScroll(value));
                  },
                ),
              ],
            ),
          ],
        );
      },
    );
  }

  Widget _buildPhaseFilter(ExecutionLogState state) {
    return DropdownButton<ExecutionPhasePB?>(
      value: state.phaseFilter,
      hint: const Text('所有阶段', style: TextStyle(fontSize: 12)),
      items: [
        const DropdownMenuItem<ExecutionPhasePB?>(
          value: null,
          child: Text('所有阶段', style: TextStyle(fontSize: 12)),
        ),
        ...ExecutionPhasePB.values.map(
          (phase) => DropdownMenuItem<ExecutionPhasePB?>(
            value: phase,
            child: Text(_getPhaseDisplayName(phase), style: const TextStyle(fontSize: 12)),
          ),
        ),
      ],
      onChanged: (value) {
        bloc.add(ExecutionLogEvent.filterByPhase(value));
      },
    );
  }

  Widget _buildStatusFilter(ExecutionLogState state) {
    return DropdownButton<ExecutionStatusPB?>(
      value: state.statusFilter,
      hint: const Text('所有状态', style: TextStyle(fontSize: 12)),
      items: [
        const DropdownMenuItem<ExecutionStatusPB?>(
          value: null,
          child: Text('所有状态', style: TextStyle(fontSize: 12)),
        ),
        ...ExecutionStatusPB.values.map(
          (status) => DropdownMenuItem<ExecutionStatusPB?>(
            value: status,
            child: Text(_getStatusDisplayName(status), style: const TextStyle(fontSize: 12)),
          ),
        ),
      ],
      onChanged: (value) {
        bloc.add(ExecutionLogEvent.filterByStatus(value));
      },
    );
  }

  Widget _buildLogList() {
    return BlocBuilder<ExecutionLogBloc, ExecutionLogState>(
      builder: (context, state) {
        // 添加调试信息
        print('🔍 [ExecutionLogViewer] Building log list - logs count: ${state.logs.length}, isLoading: ${state.isLoading}, error: ${state.error}');
        
        if (state.isLoading && state.logs.isEmpty) {
          print('🔍 [ExecutionLogViewer] Showing loading indicator');
          return const Center(
            child: CircularProgressIndicator(),
          );
        }

        if (state.error != null) {
          print('🔍 [ExecutionLogViewer] Showing error: ${state.error}');
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.error_outline, size: 48, color: Colors.red),
                const VSpace(16),
                FlowyText.medium('加载失败', fontSize: 16),
                const VSpace(8),
                FlowyText.regular(
                  state.error!,
                  fontSize: 12,
                  color: Theme.of(context).hintColor,
                ),
                const VSpace(16),
                FlowyButton(
                  text: FlowyText.regular('重试'),
                  onTap: () => bloc.add(const ExecutionLogEvent.refreshLogs()),
                ),
              ],
            ),
          );
        }

        if (state.logs.isEmpty) {
          print('🔍 [ExecutionLogViewer] Showing empty state');
          return _buildEmptyState();
        }

        print('🔍 [ExecutionLogViewer] Building ListView with ${state.logs.length} logs');
        return ListView.builder(
          controller: _scrollController,
          padding: const EdgeInsets.all(8),
          itemCount: state.logs.length + (state.hasMore ? 1 : 0),
          itemBuilder: (context, index) {
            if (index >= state.logs.length) {
              // 加载更多指示器
              return const Padding(
                padding: EdgeInsets.all(16),
                child: Center(
                  child: CircularProgressIndicator(),
                ),
              );
            }

            final log = state.logs[index];
            print('🔍 [ExecutionLogViewer] Building log item ${index}: ${log.id} - ${log.step}');
            return ExecutionLogItem(
              key: ValueKey(log.id),
              log: log,
              searchQuery: state.searchQuery,
            );
          },
        );
      },
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const FlowySvg(
            FlowySvgs.ai_summary_generate_s,
            size: Size.square(48),
          ),
          const VSpace(16),
          FlowyText.medium(
            '暂无日志',
            fontSize: 16,
          ),
          const VSpace(8),
          FlowyText.regular(
            '智能体执行时将在此显示详细日志',
            fontSize: 12,
            color: Theme.of(context).hintColor,
          ),
        ],
      ),
    );
  }

  String _getPhaseDisplayName(ExecutionPhasePB phase) {
    switch (phase) {
      case ExecutionPhasePB.ExecPlanning:
        return '规划阶段';
      case ExecutionPhasePB.ExecExecution:
        return '执行阶段';
      case ExecutionPhasePB.ExecToolCall:
        return '工具调用';
      case ExecutionPhasePB.ExecReflection:
        return '反思阶段';
      case ExecutionPhasePB.ExecCompletion:
        return '完成阶段';
      default:
        return '未知阶段';
    }
  }

  String _getStatusDisplayName(ExecutionStatusPB status) {
    switch (status) {
      case ExecutionStatusPB.ExecRunning:
        return '进行中';
      case ExecutionStatusPB.ExecSuccess:
        return '成功';
      case ExecutionStatusPB.ExecFailed:
        return '失败';
      case ExecutionStatusPB.ExecCancelled:
        return '已取消';
      default:
        return '未知状态';
    }
  }
}

/// 执行日志项组件
class ExecutionLogItem extends StatelessWidget {
  const ExecutionLogItem({
    super.key,
    required this.log,
    this.searchQuery,
  });

  final AgentExecutionLogPB log;
  final String? searchQuery;

  @override
  Widget build(BuildContext context) {
    print('🔍 [ExecutionLogItem] Building item for log: ${log.id} - ${log.step}');
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerLow,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: _getStatusColor(log.status).withOpacity(0.3),
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildHeader(context),
          if (log.input.isNotEmpty) ...[
            const VSpace(8),
            _buildSection(
              context,
              '输入',
              log.input,
            ),
          ],
          if (log.output.isNotEmpty) ...[
            const VSpace(8),
            _buildSection(
              context,
              '输出',
              log.output,
            ),
          ],
          if (log.hasErrorMessage() && log.errorMessage.isNotEmpty) ...[
            const VSpace(8),
            _buildErrorSection(context, log.errorMessage),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Row(
      children: [
        // 状态指示器
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: _getStatusColor(log.status),
            shape: BoxShape.circle,
          ),
        ),
        const HSpace(8),
        // 阶段和步骤
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  FlowyText.medium(
                    _getPhaseDisplayName(log.phase),
                    fontSize: 12,
                    color: _getStatusColor(log.status),
                  ),
                  const HSpace(8),
                  FlowyText.regular(
                    log.step,
                    fontSize: 12,
                  ),
                ],
              ),
              const VSpace(2),
              Row(
                children: [
                  FlowyText.regular(
                    _formatTimestamp(log.startedAt.toInt()),
                    fontSize: 10,
                    color: Theme.of(context).hintColor,
                  ),
                  if (log.durationMs > 0) ...[
                    const HSpace(8),
                    FlowyText.regular(
                      '${log.durationMs}ms',
                      fontSize: 10,
                      color: Theme.of(context).hintColor,
                    ),
                  ],
                ],
              ),
            ],
          ),
        ),
        // 状态标签
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(
            color: _getStatusColor(log.status).withOpacity(0.1),
            borderRadius: BorderRadius.circular(4),
          ),
          child: FlowyText.regular(
            _getStatusDisplayName(log.status),
            fontSize: 10,
            color: _getStatusColor(log.status),
          ),
        ),
      ],
    );
  }

  Widget _buildSection(BuildContext context, String title, String content) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          title,
          fontSize: 11,
          color: Theme.of(context).hintColor,
        ),
        const VSpace(4),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(6),
          ),
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: _highlightSearchQuery(content),
          ),
        ),
      ],
    );
  }

  Widget _buildErrorSection(BuildContext context, String error) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.medium(
          '错误',
          fontSize: 11,
          color: Colors.red,
        ),
        const VSpace(4),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: Colors.red.withOpacity(0.1),
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: Colors.red.withOpacity(0.3),
              width: 1,
            ),
          ),
          child: FlowyText.regular(
            error,
            fontSize: 11,
            color: Colors.red,
            maxLines: null,
          ),
        ),
      ],
    );
  }

  Color _getStatusColor(ExecutionStatusPB status) {
    switch (status) {
      case ExecutionStatusPB.ExecRunning:
        return Colors.blue;
      case ExecutionStatusPB.ExecSuccess:
        return Colors.green;
      case ExecutionStatusPB.ExecFailed:
        return Colors.red;
      case ExecutionStatusPB.ExecCancelled:
        return Colors.orange;
      default:
        return Colors.grey;
    }
  }

  String _getPhaseDisplayName(ExecutionPhasePB phase) {
    switch (phase) {
      case ExecutionPhasePB.ExecPlanning:
        return '规划';
      case ExecutionPhasePB.ExecExecution:
        return '执行';
      case ExecutionPhasePB.ExecToolCall:
        return '工具调用';
      case ExecutionPhasePB.ExecReflection:
        return '反思';
      case ExecutionPhasePB.ExecCompletion:
        return '完成';
      default:
        return '未知';
    }
  }

  String _getStatusDisplayName(ExecutionStatusPB status) {
    switch (status) {
      case ExecutionStatusPB.ExecRunning:
        return '进行中';
      case ExecutionStatusPB.ExecSuccess:
        return '成功';
      case ExecutionStatusPB.ExecFailed:
        return '失败';
      case ExecutionStatusPB.ExecCancelled:
        return '已取消';
      default:
        return '未知';
    }
  }

  String _formatTimestamp(int timestamp) {
    final dateTime = DateTime.fromMillisecondsSinceEpoch(timestamp);
    return DateFormat('HH:mm:ss').format(dateTime);
  }

  Widget _highlightSearchQuery(String text) {
    if (searchQuery == null || searchQuery!.isEmpty) {
      return FlowyText.regular(
        text,
        fontSize: 11,
        maxLines: null,
      );
    }

    final query = searchQuery!.toLowerCase();
    final lowerText = text.toLowerCase();
    
    if (!lowerText.contains(query)) {
      return FlowyText.regular(
        text,
        fontSize: 11,
        maxLines: null,
      );
    }

    final spans = <TextSpan>[];
    int start = 0;
    
    while (start < text.length) {
      final index = lowerText.indexOf(query, start);
      if (index == -1) {
        // 添加剩余文本
        if (start < text.length) {
          spans.add(TextSpan(
            text: text.substring(start),
            style: const TextStyle(fontSize: 11),
          ));
        }
        break;
      }
      
      // 添加匹配前的文本
      if (index > start) {
        spans.add(TextSpan(
          text: text.substring(start, index),
          style: const TextStyle(fontSize: 11),
        ));
      }
      
      // 添加高亮的匹配文本
      spans.add(TextSpan(
        text: text.substring(index, index + query.length),
        style: const TextStyle(
          fontSize: 11,
          backgroundColor: Colors.yellow,
          color: Colors.black,
          fontWeight: FontWeight.bold,
        ),
      ));
      
      start = index + query.length;
    }

    return RichText(
      text: TextSpan(children: spans),
      maxLines: null,
    );
  }
}
