import 'package:equatable/equatable.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'task_planner_entities.g.dart';
part 'task_planner_entities.freezed.dart';

/// 任务规划状态枚举
enum TaskPlanStatus {
  /// 草稿状态，刚创建
  draft,
  /// 等待用户确认
  pendingConfirmation,
  /// 用户已确认，准备执行
  confirmed,
  /// 正在执行中
  executing,
  /// 执行完成
  completed,
  /// 执行失败
  failed,
  /// 用户拒绝
  rejected,
  /// 已取消
  cancelled;

  /// 是否可以执行
  bool get canExecute => this == confirmed;
  
  /// 是否正在执行
  bool get isExecuting => this == executing;
  
  /// 是否已完成（成功或失败）
  bool get isFinished => [completed, failed, rejected, cancelled].contains(this);
}

/// 任务步骤状态枚举
enum TaskStepStatus {
  /// 等待执行
  pending,
  /// 正在执行
  executing,
  /// 执行成功
  completed,
  /// 执行失败
  failed,
  /// 已跳过
  skipped;

  /// 是否已完成
  bool get isFinished => [completed, failed, skipped].contains(this);
  
  /// 是否成功
  bool get isSuccessful => this == completed;
}

/// 执行状态枚举
enum ExecutionStatus {
  /// 空闲状态
  idle,
  /// 准备中
  preparing,
  /// 执行中
  running,
  /// 暂停
  paused,
  /// 完成
  completed,
  /// 失败
  failed,
  /// 已取消
  cancelled;

  /// 是否正在运行
  bool get isRunning => [preparing, running].contains(this);
  
  /// 是否已完成
  bool get isFinished => [completed, failed, cancelled].contains(this);
}

/// 步骤状态枚举
enum StepStatus {
  /// 等待执行
  pending,
  /// 正在执行
  executing,
  /// 执行成功
  success,
  /// 执行失败
  error,
  /// 已跳过
  skipped;

  /// 是否已完成
  bool get isFinished => [success, error, skipped].contains(this);
}

/// MCP传输类型枚举
enum McpTransportType {
  /// HTTP传输
  http,
  /// WebSocket传输
  websocket,
  /// 进程间通信
  ipc,
  /// 标准输入输出
  stdio;

  /// 转换为字符串
  String get displayName {
    switch (this) {
      case McpTransportType.http:
        return 'HTTP';
      case McpTransportType.websocket:
        return 'WebSocket';
      case McpTransportType.ipc:
        return 'IPC';
      case McpTransportType.stdio:
        return 'STDIO';
    }
  }
}

/// 任务规划数据模型
@freezed
class TaskPlan with _$TaskPlan {
  const factory TaskPlan({
    /// 任务规划唯一标识
    required String id,
    /// 用户原始查询
    required String userQuery,
    /// 整体策略描述
    required String overallStrategy,
    /// 任务步骤列表
    @Default([]) List<TaskStep> steps,
    /// 所需的MCP端点ID列表
    @Default([]) List<String> requiredMcpEndpoints,
    /// 创建时间
    required DateTime createdAt,
    /// 任务状态
    @Default(TaskPlanStatus.draft) TaskPlanStatus status,
    /// 预估执行时间（秒）
    @Default(0) int estimatedDurationSeconds,
    /// 智能体ID
    String? agentId,
    /// 会话ID
    String? sessionId,
    /// 错误信息
    String? errorMessage,
    /// 更新时间
    DateTime? updatedAt,
  }) = _TaskPlan;

  factory TaskPlan.fromJson(Map<String, dynamic> json) =>
      _$TaskPlanFromJson(json);
}

/// 任务步骤数据模型
@freezed
class TaskStep with _$TaskStep {
  const factory TaskStep({
    /// 步骤唯一标识
    required String id,
    /// 步骤描述
    required String description,
    /// 使用的MCP工具ID（可选，如果为null则由AI自动选择）
    String? mcpToolId,
    /// 使用的MCP端点ID（可选）
    String? mcpEndpointId,
    /// 工具调用参数
    @Default({}) Map<String, dynamic> parameters,
    /// 依赖的步骤ID列表
    @Default([]) List<String> dependencies,
    /// 步骤状态
    @Default(TaskStepStatus.pending) TaskStepStatus status,
    /// 预估执行时间（秒）
    @Default(0) int estimatedDurationSeconds,
    /// 步骤顺序
    @Default(0) int order,
    /// 执行结果
    Map<String, dynamic>? result,
    /// 错误信息
    String? errorMessage,
    /// 开始时间
    DateTime? startTime,
    /// 结束时间
    DateTime? endTime,
  }) = _TaskStep;

  factory TaskStep.fromJson(Map<String, dynamic> json) =>
      _$TaskStepFromJson(json);
}

/// 智能体配置数据模型
@freezed
class AgentConfig with _$AgentConfig {
  const factory AgentConfig({
    /// 智能体唯一标识
    required String id,
    /// 智能体名称
    required String name,
    /// 个性描述
    @Default('') String personality,
    /// 系统提示词
    @Default('') String systemPrompt,
    /// 允许使用的工具ID列表（白名单）
    @Default([]) List<String> allowedTools,
    /// 禁止使用的工具ID列表（黑名单）
    @Default([]) List<String> deniedTools,
    /// 语言偏好
    @Default('zh-CN') String languagePreference,
    /// 创建时间
    required DateTime createdAt,
    /// 更新时间
    DateTime? updatedAt,
    /// 是否启用
    @Default(true) bool isEnabled,
    /// 最大并发工具调用数
    @Default(3) int maxConcurrentTools,
    /// 工具调用超时时间（秒）
    @Default(30) int toolTimeoutSeconds,
    /// 智能体描述
    @Default('') String description,
    /// 智能体头像URL
    String? avatarUrl,
  }) = _AgentConfig;

  factory AgentConfig.fromJson(Map<String, dynamic> json) =>
      _$AgentConfigFromJson(json);
}

/// 执行日志数据模型
@freezed
class ExecutionLog with _$ExecutionLog {
  const factory ExecutionLog({
    /// 执行日志唯一标识
    required String id,
    /// 会话ID
    required String sessionId,
    /// 任务规划ID
    required String taskPlanId,
    /// 执行步骤列表
    @Default([]) List<ExecutionStep> steps,
    /// 开始时间
    required DateTime startTime,
    /// 结束时间
    DateTime? endTime,
    /// 执行状态
    @Default(ExecutionStatus.idle) ExecutionStatus status,
    /// 错误信息
    String? errorMessage,
    /// 智能体ID
    String? agentId,
    /// 用户ID
    String? userId,
    /// 总步骤数
    @Default(0) int totalSteps,
    /// 已完成步骤数
    @Default(0) int completedSteps,
    /// 执行上下文信息
    @Default({}) Map<String, dynamic> context,
  }) = _ExecutionLog;

  factory ExecutionLog.fromJson(Map<String, dynamic> json) =>
      _$ExecutionLogFromJson(json);
}

/// 执行步骤数据模型
@freezed
class ExecutionStep with _$ExecutionStep {
  const factory ExecutionStep({
    /// 步骤唯一标识
    required String id,
    /// 步骤描述
    required String stepDescription,
    /// MCP工具名称
    required String mcpToolName,
    /// 输入参数
    @Default({}) Map<String, dynamic> inputParameters,
    /// 输出结果
    Map<String, dynamic>? outputResult,
    /// 执行时间（毫秒）
    @Default(0) int executionTimeMs,
    /// 引用信息列表
    @Default([]) List<String> references,
    /// 步骤状态
    @Default(StepStatus.pending) StepStatus status,
    /// 开始时间
    DateTime? startTime,
    /// 结束时间
    DateTime? endTime,
    /// 错误信息
    String? errorMessage,
    /// 步骤顺序
    @Default(0) int order,
    /// 重试次数
    @Default(0) int retryCount,
    /// 最大重试次数
    @Default(3) int maxRetries,
  }) = _ExecutionStep;

  factory ExecutionStep.fromJson(Map<String, dynamic> json) =>
      _$ExecutionStepFromJson(json);
}

/// MCP工具信息数据模型
@freezed
class McpToolInfo with _$McpToolInfo {
  const factory McpToolInfo({
    /// 工具唯一标识
    required String id,
    /// 工具名称
    required String name,
    /// 工具描述
    @Default('') String description,
    /// 传输类型
    @Default(McpTransportType.http) McpTransportType transport,
    /// 工具模式定义
    @Default({}) Map<String, dynamic> schema,
    /// 是否可用
    @Default(false) bool isAvailable,
    /// 最后检查时间
    required DateTime lastChecked,
    /// 工具版本
    @Default('') String version,
    /// 工具提供者
    @Default('') String provider,
    /// 工具分类
    @Default('') String category,
    /// 工具图标URL
    String? iconUrl,
    /// 工具配置
    @Default({}) Map<String, dynamic> config,
    /// 是否需要认证
    @Default(false) bool requiresAuth,
    /// 认证配置
    Map<String, dynamic>? authConfig,
  }) = _McpToolInfo;

  factory McpToolInfo.fromJson(Map<String, dynamic> json) =>
      _$McpToolInfoFromJson(json);
}

/// 执行进度数据模型
@freezed
class ExecutionProgress with _$ExecutionProgress {
  const ExecutionProgress._();
  
  const factory ExecutionProgress({
    /// 当前步骤索引
    @Default(0) int currentStep,
    /// 总步骤数
    @Default(0) int totalSteps,
    /// 当前步骤描述
    @Default('') String currentStepDescription,
    /// 执行状态
    @Default(ExecutionStatus.idle) ExecutionStatus status,
    /// 开始时间
    DateTime? startTime,
    /// 预估剩余时间（秒）
    int? estimatedRemainingSeconds,
    /// 错误信息
    String? errorMessage,
  }) = _ExecutionProgress;

  factory ExecutionProgress.fromJson(Map<String, dynamic> json) =>
      _$ExecutionProgressFromJson(json);

  /// 计算进度百分比
  double get percentage {
    if (totalSteps == 0) return 0.0;
    return currentStep / totalSteps;
  }

  /// 是否已完成
  bool get isCompleted => currentStep >= totalSteps;
}

/// 任务规划器状态枚举
enum TaskPlannerStatus {
  /// 空闲状态
  idle,
  /// 正在规划
  planning,
  /// 等待确认
  waitingConfirmation,
  /// 规划完成
  planReady,
  /// 规划失败
  planFailed;

  /// 是否正在处理
  bool get isProcessing => [planning].contains(this);
  
  /// 是否需要用户操作
  bool get needsUserAction => this == waitingConfirmation;
}

/// 日志搜索条件
class LogSearchCriteria extends Equatable {
  const LogSearchCriteria({
    this.sessionId,
    this.agentId,
    this.status,
    this.startTime,
    this.endTime,
    this.keyword,
    this.toolName,
  });

  final String? sessionId;
  final String? agentId;
  final ExecutionStatus? status;
  final DateTime? startTime;
  final DateTime? endTime;
  final String? keyword;
  final String? toolName;

  @override
  List<Object?> get props => [
        sessionId,
        agentId,
        status,
        startTime,
        endTime,
        keyword,
        toolName,
      ];

  LogSearchCriteria copyWith({
    String? sessionId,
    String? agentId,
    ExecutionStatus? status,
    DateTime? startTime,
    DateTime? endTime,
    String? keyword,
    String? toolName,
  }) {
    return LogSearchCriteria(
      sessionId: sessionId ?? this.sessionId,
      agentId: agentId ?? this.agentId,
      status: status ?? this.status,
      startTime: startTime ?? this.startTime,
      endTime: endTime ?? this.endTime,
      keyword: keyword ?? this.keyword,
      toolName: toolName ?? this.toolName,
    );
  }
}

/// 导出格式枚举
enum ExportFormat {
  json,
  csv,
  txt,
  html;

  String get extension {
    switch (this) {
      case ExportFormat.json:
        return 'json';
      case ExportFormat.csv:
        return 'csv';
      case ExportFormat.txt:
        return 'txt';
      case ExportFormat.html:
        return 'html';
    }
  }

  String get mimeType {
    switch (this) {
      case ExportFormat.json:
        return 'application/json';
      case ExportFormat.csv:
        return 'text/csv';
      case ExportFormat.txt:
        return 'text/plain';
      case ExportFormat.html:
        return 'text/html';
    }
  }
}

/// 执行上下文
class ExecutionContext extends Equatable {
  const ExecutionContext({
    required this.sessionId,
    required this.userId,
    this.workspaceId,
    this.agentId,
    this.metadata = const {},
  });

  final String sessionId;
  final String userId;
  final String? workspaceId;
  final String? agentId;
  final Map<String, dynamic> metadata;

  @override
  List<Object?> get props => [
        sessionId,
        userId,
        workspaceId,
        agentId,
        metadata,
      ];

  ExecutionContext copyWith({
    String? sessionId,
    String? userId,
    String? workspaceId,
    String? agentId,
    Map<String, dynamic>? metadata,
  }) {
    return ExecutionContext(
      sessionId: sessionId ?? this.sessionId,
      userId: userId ?? this.userId,
      workspaceId: workspaceId ?? this.workspaceId,
      agentId: agentId ?? this.agentId,
      metadata: metadata ?? this.metadata,
    );
  }
}

/// 执行结果
class ExecutionResult extends Equatable {
  const ExecutionResult({
    required this.executionId,
    required this.status,
    this.result,
    this.errorMessage,
    this.executionLog,
  });

  final String executionId;
  final ExecutionStatus status;
  final Map<String, dynamic>? result;
  final String? errorMessage;
  final ExecutionLog? executionLog;

  @override
  List<Object?> get props => [
        executionId,
        status,
        result,
        errorMessage,
        executionLog,
      ];

  /// 是否成功
  bool get isSuccess => status == ExecutionStatus.completed;

  /// 是否失败
  bool get isFailure => status == ExecutionStatus.failed;

  ExecutionResult copyWith({
    String? executionId,
    ExecutionStatus? status,
    Map<String, dynamic>? result,
    String? errorMessage,
    ExecutionLog? executionLog,
  }) {
    return ExecutionResult(
      executionId: executionId ?? this.executionId,
      status: status ?? this.status,
      result: result ?? this.result,
      errorMessage: errorMessage ?? this.errorMessage,
      executionLog: executionLog ?? this.executionLog,
    );
  }
}
