import 'package:equatable/equatable.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'execution_log_entities.g.dart';
part 'execution_log_entities.freezed.dart';

/// 执行日志状态枚举
enum ExecutionLogStatus {
  /// 初始化
  initialized,
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
  cancelled,
  /// 超时
  timeout;

  /// 是否正在运行
  bool get isRunning => [preparing, running].contains(this);
  
  /// 是否已完成
  bool get isFinished => [completed, failed, cancelled, timeout].contains(this);
  
  /// 是否成功
  bool get isSuccessful => this == completed;
  
  /// 是否可以重试
  bool get canRetry => [failed, timeout, cancelled].contains(this);
}

/// 执行步骤状态枚举
enum ExecutionStepStatus {
  /// 等待执行
  pending,
  /// 正在执行
  executing,
  /// 执行成功
  success,
  /// 执行失败
  error,
  /// 已跳过
  skipped,
  /// 超时
  timeout,
  /// 已取消
  cancelled;

  /// 是否已完成
  bool get isFinished => [success, error, skipped, timeout, cancelled].contains(this);
  
  /// 是否成功
  bool get isSuccessful => this == success;
  
  /// 是否可以重试
  bool get canRetry => [error, timeout].contains(this);
}

/// 错误类型枚举
enum ExecutionErrorType {
  /// 网络错误
  network,
  /// 认证错误
  authentication,
  /// 授权错误
  authorization,
  /// 参数错误
  invalidParameters,
  /// 工具不可用
  toolUnavailable,
  /// 超时错误
  timeout,
  /// 系统错误
  system,
  /// 用户取消
  userCancelled,
  /// 依赖错误
  dependency,
  /// 配置错误
  configuration,
  /// 未知错误
  unknown;

  /// 错误描述
  String get description {
    switch (this) {
      case ExecutionErrorType.network:
        return '网络连接错误';
      case ExecutionErrorType.authentication:
        return '身份认证失败';
      case ExecutionErrorType.authorization:
        return '权限不足';
      case ExecutionErrorType.invalidParameters:
        return '参数无效';
      case ExecutionErrorType.toolUnavailable:
        return '工具不可用';
      case ExecutionErrorType.timeout:
        return '执行超时';
      case ExecutionErrorType.system:
        return '系统错误';
      case ExecutionErrorType.userCancelled:
        return '用户取消';
      case ExecutionErrorType.dependency:
        return '依赖错误';
      case ExecutionErrorType.configuration:
        return '配置错误';
      case ExecutionErrorType.unknown:
        return '未知错误';
    }
  }
  
  /// 是否可以重试
  bool get canRetry {
    switch (this) {
      case ExecutionErrorType.network:
      case ExecutionErrorType.timeout:
      case ExecutionErrorType.system:
      case ExecutionErrorType.toolUnavailable:
        return true;
      case ExecutionErrorType.authentication:
      case ExecutionErrorType.authorization:
      case ExecutionErrorType.invalidParameters:
      case ExecutionErrorType.userCancelled:
      case ExecutionErrorType.dependency:
      case ExecutionErrorType.configuration:
      case ExecutionErrorType.unknown:
        return false;
    }
  }
}

/// MCP工具状态枚举
enum McpToolStatus {
  /// 未知状态
  unknown,
  /// 可用
  available,
  /// 不可用
  unavailable,
  /// 连接中
  connecting,
  /// 已连接
  connected,
  /// 断开连接
  disconnected,
  /// 错误状态
  error;

  /// 是否可用
  bool get isAvailable => this == available || this == connected;
  
  /// 是否需要重连
  bool get needsReconnection => [unavailable, disconnected, error].contains(this);
}

/// 执行日志主数据模型
@freezed
class ExecutionLog with _$ExecutionLog {
  const factory ExecutionLog({
    /// 执行日志唯一标识
    required String id,
    /// 会话ID
    required String sessionId,
    /// 任务规划ID
    String? taskPlanId,
    /// 用户查询
    required String userQuery,
    /// 执行步骤列表
    @Default([]) List<ExecutionStep> steps,
    /// 开始时间
    required DateTime startTime,
    /// 结束时间
    DateTime? endTime,
    /// 执行状态
    @Default(ExecutionLogStatus.initialized) ExecutionLogStatus status,
    /// 错误信息
    String? errorMessage,
    /// 错误类型
    ExecutionErrorType? errorType,
    /// 智能体ID
    String? agentId,
    /// 用户ID
    String? userId,
    /// 工作空间ID
    String? workspaceId,
    /// 总步骤数
    @Default(0) int totalSteps,
    /// 已完成步骤数
    @Default(0) int completedSteps,
    /// 失败步骤数
    @Default(0) int failedSteps,
    /// 跳过步骤数
    @Default(0) int skippedSteps,
    /// 执行上下文信息
    @Default({}) Map<String, dynamic> context,
    /// 执行结果摘要
    Map<String, dynamic>? resultSummary,
    /// 使用的MCP工具列表
    @Default([]) List<String> usedMcpTools,
    /// 执行标签
    @Default([]) List<String> tags,
    /// 重试次数
    @Default(0) int retryCount,
    /// 最大重试次数
    @Default(3) int maxRetries,
    /// 父执行日志ID（用于重试）
    String? parentExecutionId,
    /// 子执行日志ID列表
    @Default([]) List<String> childExecutionIds,
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
    /// 执行日志ID
    required String executionLogId,
    /// 步骤名称
    required String name,
    /// 步骤描述
    required String description,
    /// MCP工具信息
    required McpToolInfo mcpTool,
    /// 输入参数
    @Default({}) Map<String, dynamic> inputParameters,
    /// 输出结果
    Map<String, dynamic>? outputResult,
    /// 执行时间（毫秒）
    @Default(0) int executionTimeMs,
    /// 引用信息列表
    @Default([]) List<ExecutionReference> references,
    /// 步骤状态
    @Default(ExecutionStepStatus.pending) ExecutionStepStatus status,
    /// 开始时间
    DateTime? startTime,
    /// 结束时间
    DateTime? endTime,
    /// 错误信息
    String? errorMessage,
    /// 错误类型
    ExecutionErrorType? errorType,
    /// 错误堆栈
    String? errorStack,
    /// 步骤顺序
    @Default(0) int order,
    /// 重试次数
    @Default(0) int retryCount,
    /// 最大重试次数
    @Default(3) int maxRetries,
    /// 依赖的步骤ID列表
    @Default([]) List<String> dependencies,
    /// 步骤标签
    @Default([]) List<String> tags,
    /// 步骤元数据
    @Default({}) Map<String, dynamic> metadata,
    /// 是否可以跳过
    @Default(false) bool canSkip,
    /// 是否关键步骤
    @Default(false) bool isCritical,
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
    /// 工具显示名称
    String? displayName,
    /// 工具描述
    @Default('') String description,
    /// 工具版本
    @Default('') String version,
    /// 工具提供者
    @Default('') String provider,
    /// 工具分类
    @Default('') String category,
    /// 工具状态
    @Default(McpToolStatus.unknown) McpToolStatus status,
    /// 工具配置
    @Default({}) Map<String, dynamic> config,
    /// 工具模式定义
    @Default({}) Map<String, dynamic> schema,
    /// 是否需要认证
    @Default(false) bool requiresAuth,
    /// 认证配置
    Map<String, dynamic>? authConfig,
    /// 工具图标URL
    String? iconUrl,
    /// 工具文档URL
    String? documentationUrl,
    /// 最后检查时间
    DateTime? lastChecked,
    /// 最后使用时间
    DateTime? lastUsed,
    /// 使用次数
    @Default(0) int usageCount,
    /// 成功次数
    @Default(0) int successCount,
    /// 失败次数
    @Default(0) int failureCount,
    /// 平均执行时间（毫秒）
    @Default(0) int averageExecutionTimeMs,
  }) = _McpToolInfo;

  factory McpToolInfo.fromJson(Map<String, dynamic> json) =>
      _$McpToolInfoFromJson(json);
}

/// 执行引用信息数据模型
@freezed
class ExecutionReference with _$ExecutionReference {
  const factory ExecutionReference({
    /// 引用唯一标识
    required String id,
    /// 引用类型
    required ExecutionReferenceType type,
    /// 引用标题
    required String title,
    /// 引用内容
    String? content,
    /// 引用URL
    String? url,
    /// 引用来源
    String? source,
    /// 引用时间
    required DateTime timestamp,
    /// 引用元数据
    @Default({}) Map<String, dynamic> metadata,
    /// 引用相关性评分（0-1）
    @Default(0.0) double relevanceScore,
  }) = _ExecutionReference;

  factory ExecutionReference.fromJson(Map<String, dynamic> json) =>
      _$ExecutionReferenceFromJson(json);
}

/// 执行引用类型枚举
enum ExecutionReferenceType {
  /// 文档引用
  document,
  /// 网页引用
  webpage,
  /// API引用
  api,
  /// 数据库引用
  database,
  /// 文件引用
  file,
  /// 图片引用
  image,
  /// 视频引用
  video,
  /// 其他引用
  other;

  /// 引用类型显示名称
  String get displayName {
    switch (this) {
      case ExecutionReferenceType.document:
        return '文档';
      case ExecutionReferenceType.webpage:
        return '网页';
      case ExecutionReferenceType.api:
        return 'API';
      case ExecutionReferenceType.database:
        return '数据库';
      case ExecutionReferenceType.file:
        return '文件';
      case ExecutionReferenceType.image:
        return '图片';
      case ExecutionReferenceType.video:
        return '视频';
      case ExecutionReferenceType.other:
        return '其他';
    }
  }
}

/// 执行统计信息
@freezed
class ExecutionStatistics with _$ExecutionStatistics {
  const factory ExecutionStatistics({
    /// 总执行次数
    @Default(0) int totalExecutions,
    /// 成功执行次数
    @Default(0) int successfulExecutions,
    /// 失败执行次数
    @Default(0) int failedExecutions,
    /// 取消执行次数
    @Default(0) int cancelledExecutions,
    /// 平均执行时间（毫秒）
    @Default(0) int averageExecutionTimeMs,
    /// 最短执行时间（毫秒）
    @Default(0) int minExecutionTimeMs,
    /// 最长执行时间（毫秒）
    @Default(0) int maxExecutionTimeMs,
    /// 最常用的工具
    @Default([]) List<String> mostUsedTools,
    /// 最常见的错误类型
    @Default([]) List<ExecutionErrorType> commonErrorTypes,
    /// 统计时间范围开始
    DateTime? periodStart,
    /// 统计时间范围结束
    DateTime? periodEnd,
  }) = _ExecutionStatistics;

  factory ExecutionStatistics.fromJson(Map<String, dynamic> json) =>
      _$ExecutionStatisticsFromJson(json);
}

/// 执行日志搜索条件
class ExecutionLogSearchCriteria extends Equatable {
  const ExecutionLogSearchCriteria({
    this.sessionId,
    this.agentId,
    this.userId,
    this.workspaceId,
    this.status,
    this.errorType,
    this.startTime,
    this.endTime,
    this.keyword,
    this.mcpToolName,
    this.tags,
    this.limit = 100,
    this.offset = 0,
  });

  final String? sessionId;
  final String? agentId;
  final String? userId;
  final String? workspaceId;
  final ExecutionLogStatus? status;
  final ExecutionErrorType? errorType;
  final DateTime? startTime;
  final DateTime? endTime;
  final String? keyword;
  final String? mcpToolName;
  final List<String>? tags;
  final int limit;
  final int offset;

  @override
  List<Object?> get props => [
        sessionId,
        agentId,
        userId,
        workspaceId,
        status,
        errorType,
        startTime,
        endTime,
        keyword,
        mcpToolName,
        tags,
        limit,
        offset,
      ];

  ExecutionLogSearchCriteria copyWith({
    String? sessionId,
    String? agentId,
    String? userId,
    String? workspaceId,
    ExecutionLogStatus? status,
    ExecutionErrorType? errorType,
    DateTime? startTime,
    DateTime? endTime,
    String? keyword,
    String? mcpToolName,
    List<String>? tags,
    int? limit,
    int? offset,
  }) {
    return ExecutionLogSearchCriteria(
      sessionId: sessionId ?? this.sessionId,
      agentId: agentId ?? this.agentId,
      userId: userId ?? this.userId,
      workspaceId: workspaceId ?? this.workspaceId,
      status: status ?? this.status,
      errorType: errorType ?? this.errorType,
      startTime: startTime ?? this.startTime,
      endTime: endTime ?? this.endTime,
      keyword: keyword ?? this.keyword,
      mcpToolName: mcpToolName ?? this.mcpToolName,
      tags: tags ?? this.tags,
      limit: limit ?? this.limit,
      offset: offset ?? this.offset,
    );
  }
}

/// 执行日志导出格式枚举
enum ExecutionLogExportFormat {
  /// JSON格式
  json,
  /// CSV格式
  csv,
  /// Excel格式
  excel,
  /// PDF格式
  pdf,
  /// HTML格式
  html,
  /// 纯文本格式
  text;

  /// 文件扩展名
  String get extension {
    switch (this) {
      case ExecutionLogExportFormat.json:
        return 'json';
      case ExecutionLogExportFormat.csv:
        return 'csv';
      case ExecutionLogExportFormat.excel:
        return 'xlsx';
      case ExecutionLogExportFormat.pdf:
        return 'pdf';
      case ExecutionLogExportFormat.html:
        return 'html';
      case ExecutionLogExportFormat.text:
        return 'txt';
    }
  }

  /// MIME类型
  String get mimeType {
    switch (this) {
      case ExecutionLogExportFormat.json:
        return 'application/json';
      case ExecutionLogExportFormat.csv:
        return 'text/csv';
      case ExecutionLogExportFormat.excel:
        return 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';
      case ExecutionLogExportFormat.pdf:
        return 'application/pdf';
      case ExecutionLogExportFormat.html:
        return 'text/html';
      case ExecutionLogExportFormat.text:
        return 'text/plain';
    }
  }

  /// 格式显示名称
  String get displayName {
    switch (this) {
      case ExecutionLogExportFormat.json:
        return 'JSON';
      case ExecutionLogExportFormat.csv:
        return 'CSV';
      case ExecutionLogExportFormat.excel:
        return 'Excel';
      case ExecutionLogExportFormat.pdf:
        return 'PDF';
      case ExecutionLogExportFormat.html:
        return 'HTML';
      case ExecutionLogExportFormat.text:
        return '纯文本';
    }
  }
}

/// 执行日志导出选项
class ExecutionLogExportOptions extends Equatable {
  const ExecutionLogExportOptions({
    required this.format,
    this.includeSteps = true,
    this.includeReferences = true,
    this.includeMetadata = false,
    this.includeErrorDetails = true,
    this.dateFormat = 'yyyy-MM-dd HH:mm:ss',
    this.maxRecords,
  });

  final ExecutionLogExportFormat format;
  final bool includeSteps;
  final bool includeReferences;
  final bool includeMetadata;
  final bool includeErrorDetails;
  final String dateFormat;
  final int? maxRecords;

  @override
  List<Object?> get props => [
        format,
        includeSteps,
        includeReferences,
        includeMetadata,
        includeErrorDetails,
        dateFormat,
        maxRecords,
      ];

  ExecutionLogExportOptions copyWith({
    ExecutionLogExportFormat? format,
    bool? includeSteps,
    bool? includeReferences,
    bool? includeMetadata,
    bool? includeErrorDetails,
    String? dateFormat,
    int? maxRecords,
  }) {
    return ExecutionLogExportOptions(
      format: format ?? this.format,
      includeSteps: includeSteps ?? this.includeSteps,
      includeReferences: includeReferences ?? this.includeReferences,
      includeMetadata: includeMetadata ?? this.includeMetadata,
      includeErrorDetails: includeErrorDetails ?? this.includeErrorDetails,
      dateFormat: dateFormat ?? this.dateFormat,
      maxRecords: maxRecords ?? this.maxRecords,
    );
  }
}

/// 执行日志过滤器
class ExecutionLogFilter extends Equatable {
  const ExecutionLogFilter({
    this.statuses = const [],
    this.errorTypes = const [],
    this.mcpTools = const [],
    this.agents = const [],
    this.tags = const [],
    this.minDuration,
    this.maxDuration,
    this.hasErrors = false,
    this.hasReferences = false,
  });

  final List<ExecutionLogStatus> statuses;
  final List<ExecutionErrorType> errorTypes;
  final List<String> mcpTools;
  final List<String> agents;
  final List<String> tags;
  final Duration? minDuration;
  final Duration? maxDuration;
  final bool hasErrors;
  final bool hasReferences;

  @override
  List<Object?> get props => [
        statuses,
        errorTypes,
        mcpTools,
        agents,
        tags,
        minDuration,
        maxDuration,
        hasErrors,
        hasReferences,
      ];

  ExecutionLogFilter copyWith({
    List<ExecutionLogStatus>? statuses,
    List<ExecutionErrorType>? errorTypes,
    List<String>? mcpTools,
    List<String>? agents,
    List<String>? tags,
    Duration? minDuration,
    Duration? maxDuration,
    bool? hasErrors,
    bool? hasReferences,
  }) {
    return ExecutionLogFilter(
      statuses: statuses ?? this.statuses,
      errorTypes: errorTypes ?? this.errorTypes,
      mcpTools: mcpTools ?? this.mcpTools,
      agents: agents ?? this.agents,
      tags: tags ?? this.tags,
      minDuration: minDuration ?? this.minDuration,
      maxDuration: maxDuration ?? this.maxDuration,
      hasErrors: hasErrors ?? this.hasErrors,
      hasReferences: hasReferences ?? this.hasReferences,
    );
  }

  /// 是否为空过滤器
  bool get isEmpty =>
      statuses.isEmpty &&
      errorTypes.isEmpty &&
      mcpTools.isEmpty &&
      agents.isEmpty &&
      tags.isEmpty &&
      minDuration == null &&
      maxDuration == null &&
      !hasErrors &&
      !hasReferences;
}

/// 执行日志排序选项
enum ExecutionLogSortBy {
  /// 按创建时间排序
  createdTime,
  /// 按结束时间排序
  endTime,
  /// 按执行时间排序
  duration,
  /// 按状态排序
  status,
  /// 按步骤数排序
  stepCount,
  /// 按错误类型排序
  errorType;

  /// 排序字段显示名称
  String get displayName {
    switch (this) {
      case ExecutionLogSortBy.createdTime:
        return '创建时间';
      case ExecutionLogSortBy.endTime:
        return '结束时间';
      case ExecutionLogSortBy.duration:
        return '执行时间';
      case ExecutionLogSortBy.status:
        return '状态';
      case ExecutionLogSortBy.stepCount:
        return '步骤数';
      case ExecutionLogSortBy.errorType:
        return '错误类型';
    }
  }
}

/// 执行日志排序方向
enum ExecutionLogSortDirection {
  /// 升序
  ascending,
  /// 降序
  descending;

  /// 排序方向显示名称
  String get displayName {
    switch (this) {
      case ExecutionLogSortDirection.ascending:
        return '升序';
      case ExecutionLogSortDirection.descending:
        return '降序';
    }
  }
}

/// 执行日志排序选项
class ExecutionLogSortOptions extends Equatable {
  const ExecutionLogSortOptions({
    this.sortBy = ExecutionLogSortBy.createdTime,
    this.direction = ExecutionLogSortDirection.descending,
  });

  final ExecutionLogSortBy sortBy;
  final ExecutionLogSortDirection direction;

  @override
  List<Object?> get props => [sortBy, direction];

  ExecutionLogSortOptions copyWith({
    ExecutionLogSortBy? sortBy,
    ExecutionLogSortDirection? direction,
  }) {
    return ExecutionLogSortOptions(
      sortBy: sortBy ?? this.sortBy,
      direction: direction ?? this.direction,
    );
  }
}
