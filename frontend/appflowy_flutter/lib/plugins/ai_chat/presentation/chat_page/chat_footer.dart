import 'package:appflowy/ai/ai.dart';
import 'package:appflowy/plugins/ai_chat/application/ai_chat_prelude.dart';
import 'package:appflowy/plugins/ai_chat/presentation/chat_input/mobile_chat_input.dart';
import 'package:appflowy/plugins/ai_chat/presentation/layout_define.dart';
import 'package:appflowy/plugins/mcp/chat/mcp_selector.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/task_planner_entities.dart';
import 'package:appflowy/plugins/ai_chat/application/mcp_endpoint_service.dart';
import 'package:appflowy/plugins/ai_chat/presentation/mcp_endpoint_selector.dart';
import 'package:appflowy/plugins/ai_chat/presentation/task_confirmation_dialog.dart';
import 'package:appflowy/workspace/presentation/home/home_stack.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:appflowy_backend/log.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:universal_platform/universal_platform.dart';

class ChatFooter extends StatefulWidget {
  const ChatFooter({
    super.key,
    required this.view,
  });

  final ViewPB view;

  @override
  State<ChatFooter> createState() => _ChatFooterState();
}

class _ChatFooterState extends State<ChatFooter> {
  final textController = AiPromptInputTextEditingController();
  final ValueNotifier<List<String>> _selectedMcpNames = ValueNotifier([]);
  
  // 任务规划相关状态
  final ValueNotifier<List<String>> _selectedEndpointIds = ValueNotifier([]);
  final ValueNotifier<bool> _enableTaskPlanning = ValueNotifier(false);
  final ValueNotifier<List<McpEndpointInfo>> _availableEndpoints = ValueNotifier([]);
  final McpEndpointService _mcpEndpointService = McpEndpointService();
  TaskPlan? _pendingTaskPlan;

  @override
  void initState() {
    super.initState();
    _loadAvailableEndpoints();
  }

  @override
  void dispose() {
    textController.dispose();
    _selectedMcpNames.dispose();
    _selectedEndpointIds.dispose();
    _enableTaskPlanning.dispose();
    _availableEndpoints.dispose();
    super.dispose();
  }

  /// 加载可用的MCP端点
  Future<void> _loadAvailableEndpoints() async {
    try {
      final endpoints = await _mcpEndpointService.getAvailableEndpoints();
      _availableEndpoints.value = endpoints;
    } catch (e) {
      Log.error('Failed to load MCP endpoints: $e');
      _availableEndpoints.value = [];
    }
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<TaskPlannerBloc, TaskPlannerState>(
      listener: (context, state) {
        // 记录状态变化
        Log.debug('TaskPlannerBloc状态变化: ${state.status}');
        
        // 处理规划中状态
        if (state.status == TaskPlannerStatus.planning) {
          Log.debug('正在生成任务规划...');
        }
        
        // 处理等待确认状态
        if (state.status == TaskPlannerStatus.waitingConfirmation && 
            state.currentTaskPlan != null &&
            _pendingTaskPlan?.id != state.currentTaskPlan!.id) {
          Log.debug('显示任务确认对话框: ${state.currentTaskPlan!.id}');
          _pendingTaskPlan = state.currentTaskPlan;
          
          // 隐藏加载提示
          ScaffoldMessenger.of(context).hideCurrentSnackBar();
          
          // 显示任务确认对话框
          _showTaskConfirmationDialog(context, state.currentTaskPlan!);
        }
        
        // 处理规划完成状态
        if (state.status == TaskPlannerStatus.planReady) {
          _pendingTaskPlan = null;
          Log.debug('任务规划已确认，准备执行');
          
          try {
            // 任务已确认，发送消息开始执行
            if (state.currentTaskPlan != null) {
              final taskPlan = state.currentTaskPlan!;
              final chatBloc = context.read<ChatBloc>();
              
              final metadata = <String, dynamic>{};
              
              // 对于任务规划消息，我们使用特殊的处理方式
              // 不直接使用MCP选择器，而是通过任务规划系统来协调
              metadata['taskPlanId'] = taskPlan.id;
              metadata['isTaskPlanExecution'] = true;
              
              // 保存MCP工具信息供任务执行时使用
              if (_selectedMcpNames.value.isNotEmpty) {
                metadata['mcpNames'] = _selectedMcpNames.value;
              }
              if (_selectedEndpointIds.value.isNotEmpty) {
                metadata['selectedEndpointIds'] = _selectedEndpointIds.value;
              }
              
              // 隐藏加载提示
              ScaffoldMessenger.of(context).hideCurrentSnackBar();
              
              // 清空输入框
              textController.clear();
              
              Log.debug('准备发送任务规划消息: ${taskPlan.userQuery}');
              Log.debug('消息元数据: $metadata');
              
              chatBloc.add(
                ChatEvent.sendMessage(
                  message: taskPlan.userQuery,
                  metadata: metadata,
                ),
              );
              
              Log.debug('任务规划消息已发送: ${taskPlan.userQuery}');
            }
          } catch (e, stackTrace) {
            Log.error('发送任务规划消息时出错: $e');
            Log.error('堆栈跟踪: $stackTrace');
            
            // 显示错误信息
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text('发送消息失败: $e'),
                backgroundColor: Theme.of(context).colorScheme.error,
              ),
            );
          }
        }
        
        // 处理错误状态
        if (state.status == TaskPlannerStatus.planFailed) {
          _pendingTaskPlan = null;
          Log.error('任务规划失败: ${state.errorMessage}');
          
          // 隐藏加载提示
          ScaffoldMessenger.of(context).hideCurrentSnackBar();
          
          // 显示错误信息
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('任务规划失败: ${state.errorMessage ?? "未知错误"}'),
              backgroundColor: Theme.of(context).colorScheme.error,
            ),
          );
        }
      },
      child: BlocSelector<ChatSelectMessageBloc, ChatSelectMessageState, bool>(
        selector: (state) => state.isSelectingMessages,
        builder: (context, isSelectingMessages) {
          return AnimatedSwitcher(
            duration: const Duration(milliseconds: 150),
            transitionBuilder: (child, animation) {
              return NonClippingSizeTransition(
                sizeFactor: animation,
                axisAlignment: -1,
                child: child,
              );
            },
            child: isSelectingMessages
                ? const SizedBox.shrink()
                : Padding(
                    padding: AIChatUILayout.safeAreaInsets(context),
                    child: BlocSelector<ChatBloc, ChatState, bool>(
                      selector: (state) {
                        return state.promptResponseState.isReady;
                      },
                      builder: (context, canSendMessage) {
                        final chatBloc = context.read<ChatBloc>();

                        return UniversalPlatform.isDesktop
                            ? _buildDesktopInput(
                                context,
                                chatBloc,
                                canSendMessage,
                              )
                            : _buildMobileInput(
                                context,
                                chatBloc,
                                canSendMessage,
                              );
                      },
                    ),
                  ),
          );
        },
      ),
    );
  }

  Widget _buildDesktopInput(
    BuildContext context,
    ChatBloc chatBloc,
    bool canSendMessage,
  ) {
    return DesktopPromptInput(
      isStreaming: !canSendMessage,
      textController: textController,
      onStopStreaming: () {
        chatBloc.add(const ChatEvent.stopStream());
      },
      onSubmitted: (text, format, metadata, promptId) {
        _handleMessageSubmission(context, chatBloc, text, format, metadata, promptId);
      },
      selectedSourcesNotifier: chatBloc.selectedSourcesNotifier,
      onUpdateSelectedSources: (ids) {
        chatBloc.add(
          ChatEvent.updateSelectedSources(
            selectedSourcesIds: ids,
          ),
        );
      },
      leadingExtra: _buildInputExtras(context),
    );
  }

  Widget _buildMobileInput(
    BuildContext context,
    ChatBloc chatBloc,
    bool canSendMessage,
  ) {
    return MobileChatInput(
      isStreaming: !canSendMessage,
      onStopStreaming: () {
        chatBloc.add(const ChatEvent.stopStream());
      },
      onSubmitted: (text, format, metadata) {
        _handleMessageSubmission(context, chatBloc, text, format, metadata, null);
      },
      selectedSourcesNotifier: chatBloc.selectedSourcesNotifier,
      onUpdateSelectedSources: (ids) {
        chatBloc.add(
          ChatEvent.updateSelectedSources(
            selectedSourcesIds: ids,
          ),
        );
      },
      leadingExtra: _buildInputExtras(context, isMobile: true),
    );
  }

  /// 构建输入区域的额外组件（简化版本）
  Widget _buildInputExtras(BuildContext context, {bool isMobile = false}) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 保留原有的MCP选择器（向后兼容）
        McpSelector(
          onChanged: (names) => _selectedMcpNames.value = names,
          iconSize: isMobile ? 18.0 : 20.0,
        ),
        
        // 如果有选中的工具或启用了任务规划，显示高级功能按钮
        ValueListenableBuilder<bool>(
          valueListenable: _enableTaskPlanning,
          builder: (context, taskPlanningEnabled, child) {
            return ValueListenableBuilder<List<String>>(
              valueListenable: _selectedEndpointIds,
              builder: (context, selectedEndpointIds, child) {
                final hasAdvancedFeatures = taskPlanningEnabled || selectedEndpointIds.isNotEmpty;
                
                if (!hasAdvancedFeatures) {
                  return _buildAdvancedFeaturesButton(context, isMobile);
                }
                
                return Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const SizedBox(width: 8),
                    _buildAdvancedFeaturesIndicator(context, isMobile, taskPlanningEnabled, selectedEndpointIds),
                  ],
                );
              },
            );
          },
        ),
      ],
    );
  }

  /// 构建高级功能按钮（当没有启用任何高级功能时显示）
  Widget _buildAdvancedFeaturesButton(BuildContext context, bool isMobile) {
    return Padding(
      padding: const EdgeInsets.only(left: 8.0),
      child: Tooltip(
        message: '高级功能',
        child: IconButton(
          icon: Icon(
            Icons.tune,
            size: isMobile ? 18.0 : 20.0,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
          onPressed: () => _showAdvancedFeaturesDialog(context),
        ),
      ),
    );
  }

  /// 构建高级功能指示器（当启用了高级功能时显示）
  Widget _buildAdvancedFeaturesIndicator(
    BuildContext context, 
    bool isMobile, 
    bool taskPlanningEnabled, 
    List<String> selectedEndpointIds,
  ) {
    final features = <String>[];
    if (taskPlanningEnabled) features.add('任务规划');
    if (selectedEndpointIds.isNotEmpty) features.add('${selectedEndpointIds.length}个端点');
    
    return Tooltip(
      message: '已启用: ${features.join(', ')}',
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.3),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              taskPlanningEnabled ? Icons.auto_awesome : Icons.extension,
              size: isMobile ? 14.0 : 16.0,
              color: Theme.of(context).colorScheme.primary,
            ),
            const SizedBox(width: 4),
            Text(
              features.join(' + '),
              style: TextStyle(
                fontSize: isMobile ? 11.0 : 12.0,
                color: Theme.of(context).colorScheme.primary,
                fontWeight: FontWeight.w500,
              ),
            ),
            const SizedBox(width: 4),
            GestureDetector(
              onTap: () => _showAdvancedFeaturesDialog(context),
              child: Icon(
                Icons.settings,
                size: isMobile ? 14.0 : 16.0,
                color: Theme.of(context).colorScheme.primary,
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 处理消息提交逻辑
  void _handleMessageSubmission(
    BuildContext context,
    ChatBloc chatBloc,
    String text,
    PredefinedFormat? format,
    Map<String, dynamic> metadata,
    String? promptId,
  ) {
    final m = {...metadata};
    
    // 添加MCP名称（保持向后兼容）
    if (_selectedMcpNames.value.isNotEmpty) {
      m[messageSelectedMcpNamesKey] = _selectedMcpNames.value;
    }
    
    // 添加选中的端点ID
    if (_selectedEndpointIds.value.isNotEmpty) {
      m['selectedEndpointIds'] = _selectedEndpointIds.value;
    }
    
    // 如果启用了任务规划，先创建任务规划
    if (_enableTaskPlanning.value) {
      _createTaskPlan(context, text, _selectedEndpointIds.value);
    } else {
      // 直接发送消息
      chatBloc.add(
        ChatEvent.sendMessage(
          message: text,
          format: format,
          metadata: m,
          promptId: promptId,
        ),
      );
    }
  }

  /// 创建任务规划
  void _createTaskPlan(BuildContext context, String userQuery, List<String> endpointIds) {
    try {
      final taskPlannerBloc = context.read<TaskPlannerBloc>();
      
      // 显示加载指示器
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('正在生成任务规划...'),
          duration: Duration(seconds: 2),
        ),
      );
      
      taskPlannerBloc.add(
        TaskPlannerEvent.createTaskPlan(
          userQuery: userQuery,
          mcpEndpoints: endpointIds,
        ),
      );
    } catch (error) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('创建任务规划失败: $error')),
      );
    }
  }

  /// 显示任务确认对话框
  void _showTaskConfirmationDialog(BuildContext context, TaskPlan taskPlan) {
    Log.debug('准备显示任务确认对话框');
    
    // 确保在下一帧显示对话框，避免构建过程中的冲突
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        Log.debug('显示任务确认对话框: ${taskPlan.userQuery}');
        showTaskConfirmationDialog(
          context: context,
          taskPlan: taskPlan,
        ).then((action) {
          if (action != null && mounted) {
            _handleTaskConfirmationAction(context, taskPlan, action);
          }
        });
      }
    });
  }

  /// 处理任务确认对话框的操作
  void _handleTaskConfirmationAction(
    BuildContext context,
    TaskPlan taskPlan,
    TaskConfirmationAction action,
  ) {
    try {
      final taskPlannerBloc = context.read<TaskPlannerBloc>();
      
      Log.debug('处理任务确认动作: $action, 任务ID: ${taskPlan.id}');
      
      switch (action) {
        case TaskConfirmationAction.confirm:
          Log.debug('用户确认任务规划，任务ID: ${taskPlan.id}');
          Log.debug('当前TaskPlannerBloc状态: ${taskPlannerBloc.state.status}');
          taskPlannerBloc.add(TaskPlannerEvent.confirmTaskPlan(taskPlanId: taskPlan.id));
          Log.debug('已发送confirmTaskPlan事件');
          break;
        case TaskConfirmationAction.reject:
          Log.debug('用户拒绝任务规划');
          taskPlannerBloc.add(TaskPlannerEvent.rejectTaskPlan(
            taskPlanId: taskPlan.id, 
            reason: '用户拒绝',
          ),);
          break;
        case TaskConfirmationAction.modify:
          Log.debug('用户请求修改任务规划');
          // TODO: 实现任务修改功能
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('任务修改功能即将推出')),
          );
          break;
      }
    } catch (e, stackTrace) {
      Log.error('处理任务确认动作时出错: $e');
      Log.error('堆栈跟踪: $stackTrace');
      
      // 显示错误信息
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('处理任务确认失败: $e'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }

  /// 显示高级功能配置对话框
  void _showAdvancedFeaturesDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => ValueListenableBuilder<List<McpEndpointInfo>>(
        valueListenable: _availableEndpoints,
        builder: (context, availableEndpoints, child) {
          return _AdvancedFeaturesDialog(
            enableTaskPlanning: _enableTaskPlanning.value,
            selectedEndpointIds: _selectedEndpointIds.value,
            availableEndpoints: availableEndpoints,
            onTaskPlanningChanged: (enabled) {
              _enableTaskPlanning.value = enabled;
            },
            onEndpointSelectionChanged: (endpointIds) {
              _selectedEndpointIds.value = endpointIds;
            },
            onRefreshEndpoints: _loadAvailableEndpoints,
          );
        },
      ),
    );
  }
}

/// 高级功能配置对话框
class _AdvancedFeaturesDialog extends StatefulWidget {
  const _AdvancedFeaturesDialog({
    required this.enableTaskPlanning,
    required this.selectedEndpointIds,
    required this.availableEndpoints,
    required this.onTaskPlanningChanged,
    required this.onEndpointSelectionChanged,
    required this.onRefreshEndpoints,
  });

  final bool enableTaskPlanning;
  final List<String> selectedEndpointIds;
  final List<McpEndpointInfo> availableEndpoints;
  final ValueChanged<bool> onTaskPlanningChanged;
  final ValueChanged<List<String>> onEndpointSelectionChanged;
  final VoidCallback onRefreshEndpoints;

  @override
  State<_AdvancedFeaturesDialog> createState() => _AdvancedFeaturesDialogState();
}

class _AdvancedFeaturesDialogState extends State<_AdvancedFeaturesDialog> {
  late bool _enableTaskPlanning;
  late List<String> _selectedEndpointIds;

  @override
  void initState() {
    super.initState();
    _enableTaskPlanning = widget.enableTaskPlanning;
    _selectedEndpointIds = List.from(widget.selectedEndpointIds);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('高级功能设置'),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 任务规划开关
            SwitchListTile(
              title: const Text('启用任务规划'),
              subtitle: const Text('AI将为复杂任务创建详细的执行计划'),
              value: _enableTaskPlanning,
              onChanged: (value) {
                setState(() {
                  _enableTaskPlanning = value;
                });
              },
            ),
            const SizedBox(height: 16),
            
            // MCP工具选择
            const Text(
              'MCP端点选择',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w500,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'AI将从选中的端点中自动选择合适的工具来完成任务',
              style: TextStyle(
                fontSize: 12,
                color: Colors.grey,
              ),
            ),
            const SizedBox(height: 12),
            
            // 端点选择器
            if (widget.availableEndpoints.isNotEmpty)
              McpEndpointSelector(
                availableEndpoints: widget.availableEndpoints,
                selectedEndpointIds: _selectedEndpointIds,
                onSelectionChanged: (endpointIds) {
                  setState(() {
                    _selectedEndpointIds = endpointIds;
                  });
                },
                maxHeight: 300,
              )
            else
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.grey.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  children: [
                    const Text(
                      '暂无可用的MCP端点',
                      style: TextStyle(color: Colors.grey),
                    ),
                    const SizedBox(height: 8),
                    TextButton.icon(
                      onPressed: widget.onRefreshEndpoints,
                      icon: const Icon(Icons.refresh, size: 16),
                      label: const Text('刷新端点列表'),
                    ),
                  ],
                ),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('取消'),
        ),
        ElevatedButton(
          onPressed: () {
            widget.onTaskPlanningChanged(_enableTaskPlanning);
            widget.onEndpointSelectionChanged(_selectedEndpointIds);
            Navigator.of(context).pop();
          },
          child: const Text('确定'),
        ),
      ],
    );
  }
}
