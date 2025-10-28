// RAG设置相关的实体类
// 这些类对应后端的protobuf定义，用于前端状态管理和数据传输

import 'package:appflowy_backend/protobuf/flowy-ai/protobuf.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:fixnum/fixnum.dart';

part 'entities.freezed.dart';

/// RAG设置数据类
/// 对应后端的RAGSettingsPB protobuf结构
@freezed
class RAGSettingsData with _$RAGSettingsData {
  const factory RAGSettingsData({
    /// 文本块大小（字符数）
    @Default(1000) int chunkSize,

    /// 块之间的重叠字符数
    @Default(200) int chunkOverlap,

    /// 是否启用语义分割
    @Default(false) bool enableSemanticSplitting,

    /// 是否启用混合检索
    @Default(false) bool enableHybridSearch,

    /// 向量检索权重 (0.0 - 1.0)
    @Default(0.5) double vectorWeight,

    /// 关键词检索权重 (0.0 - 1.0)
    @Default(0.5) double keywordWeight,

    /// 初始检索返回的文档数量
    @Default(20) int initialTopK,

    /// 最终返回的文档数量
    @Default(5) int finalTopK,

    /// 是否启用重排序
    @Default(false) bool enableReranking,

    /// 重排序模型名称（可选）
    String? rerankerModel,

    /// 重排序 API URL（用于独立的重排序服务）
    String? rerankerApiUrl,

    /// 重排序 API Key（用于独立的重排序服务）
    String? rerankerApiKey,

    /// 是否启用智能体反思
    @Default(false) bool enableAgentReflection,

    /// 反思阈值 (0.0 - 1.0)
    @Default(0.7) double reflectionThreshold,

    /// 创建时间戳
    Int64? createdAt,

    /// 更新时间戳
    Int64? updatedAt,

    /// 额外元数据
    @Default(<String, String>{}) Map<String, String> metadata,
  }) = _RAGSettingsData;

  const RAGSettingsData._();

  /// 从protobuf创建实例
  factory RAGSettingsData.fromProtobuf(RAGSettingsPB pb) {
    return RAGSettingsData(
      chunkSize: pb.chunkSize,
      chunkOverlap: pb.chunkOverlap,
      enableSemanticSplitting: pb.enableSemanticSplitting,
      enableHybridSearch: pb.enableHybridSearch,
      vectorWeight: pb.vectorWeight,
      keywordWeight: pb.keywordWeight,
      initialTopK: pb.initialTopK,
      finalTopK: pb.finalTopK,
      enableReranking: pb.enableReranking,
      rerankerModel: pb.hasRerankerModel() ? pb.rerankerModel : null,
      rerankerApiUrl: pb.hasRerankerApiUrl() ? pb.rerankerApiUrl : null,
      rerankerApiKey: pb.hasRerankerApiKey() ? pb.rerankerApiKey : null,
      enableAgentReflection: pb.enableAgentReflection,
      reflectionThreshold: pb.reflectionThreshold,
      createdAt: pb.hasCreatedAt() ? pb.createdAt : null,
      updatedAt: pb.hasUpdatedAt() ? pb.updatedAt : null,
      metadata: pb.metadata,
    );
  }

  /// 获取默认设置
  factory RAGSettingsData.defaultSettings() {
    return const RAGSettingsData();
  }

  /// 转换为protobuf
  RAGSettingsPB toProtobuf() {
    final pb = RAGSettingsPB.create()
      ..chunkSize = chunkSize
      ..chunkOverlap = chunkOverlap
      ..enableSemanticSplitting = enableSemanticSplitting
      ..enableHybridSearch = enableHybridSearch
      ..vectorWeight = vectorWeight
      ..keywordWeight = keywordWeight
      ..initialTopK = initialTopK
      ..finalTopK = finalTopK
      ..enableReranking = enableReranking
      ..enableAgentReflection = enableAgentReflection
      ..reflectionThreshold = reflectionThreshold
      ..metadata.addAll(metadata);

    if (rerankerModel != null) {
      pb.rerankerModel = rerankerModel!;
    }
    if (rerankerApiUrl != null) {
      pb.rerankerApiUrl = rerankerApiUrl!;
    }
    if (rerankerApiKey != null) {
      pb.rerankerApiKey = rerankerApiKey!;
    }
    if (createdAt != null) {
      pb.createdAt = createdAt!;
    }
    if (updatedAt != null) {
      pb.updatedAt = updatedAt!;
    }

    return pb;
  }

  /// 验证设置是否有效
  bool isValid() {
    // 验证chunk_size > 0
    if (chunkSize <= 0) return false;

    // 验证chunk_overlap >= 0 且 < chunk_size
    if (chunkOverlap < 0 || chunkOverlap >= chunkSize) return false;

    // 验证权重在0.0-1.0之间
    if (vectorWeight < 0.0 || vectorWeight > 1.0) return false;
    if (keywordWeight < 0.0 || keywordWeight > 1.0) return false;

    // 如果启用混合检索，权重和应该接近1.0
    if (enableHybridSearch) {
      final sum = vectorWeight + keywordWeight;
      if (sum < 0.9 || sum > 1.1) return false;
    }

    // 验证top_k参数
    if (initialTopK <= 0 || finalTopK <= 0) return false;
    if (finalTopK > initialTopK) return false;

    // 验证反思阈值
    if (reflectionThreshold < 0.0 || reflectionThreshold > 1.0) return false;

    return true;
  }
}

/// RAG设置状态
/// 用于BLoC状态管理
@freezed
class RAGSettingsState with _$RAGSettingsState {
  const factory RAGSettingsState({
    /// 当前RAG设置
    required RAGSettingsData settings,

    /// 是否正在加载
    @Default(false) bool isLoading,

    /// 是否正在保存
    @Default(false) bool isSaving,

    /// 是否有错误
    String? error,

    /// 保存成功的消息
    String? successMessage,
  }) = _RAGSettingsState;

  /// 初始状态
  factory RAGSettingsState.initial() {
    return RAGSettingsState(
      settings: RAGSettingsData.defaultSettings(),
      isLoading: true,
    );
  }
}

/// RAG设置事件
/// 用于BLoC事件处理
@freezed
class RAGSettingsEvent with _$RAGSettingsEvent {
  /// 初始化加载设置
  const factory RAGSettingsEvent.load() = _LoadRAGSettings;

  /// 更新块大小
  const factory RAGSettingsEvent.updateChunkSize(int value) = _UpdateChunkSize;

  /// 更新块重叠
  const factory RAGSettingsEvent.updateChunkOverlap(int value) = _UpdateChunkOverlap;

  /// 更新语义分割启用状态
  const factory RAGSettingsEvent.updateEnableSemanticSplitting(bool value) =
      _UpdateEnableSemanticSplitting;

  /// 更新混合检索启用状态
  const factory RAGSettingsEvent.updateEnableHybridSearch(bool value) = _UpdateEnableHybridSearch;

  /// 更新向量权重
  const factory RAGSettingsEvent.updateVectorWeight(double value) = _UpdateVectorWeight;

  /// 更新关键词权重
  const factory RAGSettingsEvent.updateKeywordWeight(double value) = _UpdateKeywordWeight;

  /// 更新初始TopK
  const factory RAGSettingsEvent.updateInitialTopK(int value) = _UpdateInitialTopK;

  /// 更新最终TopK
  const factory RAGSettingsEvent.updateFinalTopK(int value) = _UpdateFinalTopK;

  /// 更新重排序启用状态
  const factory RAGSettingsEvent.updateEnableReranking(bool value) = _UpdateEnableReranking;

  /// 更新重排序模型
  const factory RAGSettingsEvent.updateRerankerModel(String? value) = _UpdateRerankerModel;

  /// 更新智能体反思启用状态
  const factory RAGSettingsEvent.updateEnableAgentReflection(bool value) =
      _UpdateEnableAgentReflection;

  /// 更新反思阈值
  const factory RAGSettingsEvent.updateReflectionThreshold(double value) =
      _UpdateReflectionThreshold;

  /// 保存设置
  const factory RAGSettingsEvent.save() = _SaveRAGSettings;

  /// 重置为默认设置
  const factory RAGSettingsEvent.reset() = _ResetRAGSettings;

  /// 清除错误消息
  const factory RAGSettingsEvent.clearError() = _ClearError;

  /// 清除成功消息
  const factory RAGSettingsEvent.clearSuccess() = _ClearSuccess;
}

/// 检索到的文档
/// 对应后端的RetrievedDocumentPB
@freezed
class RetrievedDocument with _$RetrievedDocument {
  const factory RetrievedDocument({
    required String documentId,
    required String content,
    required Map<String, String> metadata,
    double? vectorScore,
    double? keywordScore,
    required double combinedScore,
    required RetrievalTypePB retrievalType,
  }) = _RetrievedDocument;

  const RetrievedDocument._();

  /// 从protobuf创建
  factory RetrievedDocument.fromProtobuf(RetrievedDocumentPB pb) {
    return RetrievedDocument(
      documentId: pb.documentId,
      content: pb.content,
      metadata: pb.metadata,
      vectorScore: pb.hasVectorScore() ? pb.vectorScore : null,
      keywordScore: pb.hasKeywordScore() ? pb.keywordScore : null,
      combinedScore: pb.combinedScore,
      retrievalType: pb.retrievalType,
    );
  }

  /// 转换为protobuf
  RetrievedDocumentPB toProtobuf() {
    final pb = RetrievedDocumentPB.create()
      ..documentId = documentId
      ..content = content
      ..combinedScore = combinedScore
      ..retrievalType = retrievalType
      ..metadata.addAll(metadata);

    if (vectorScore != null) {
      pb.vectorScore = vectorScore!;
    }
    if (keywordScore != null) {
      pb.keywordScore = keywordScore!;
    }

    return pb;
  }
}

/// 智能体反思结果
/// 对应后端的AgentReflectionResultPB
@freezed
class AgentReflectionResult with _$AgentReflectionResult {
  const factory AgentReflectionResult({
    required double confidence,
    required bool isQuestionSolved,
    String? suggestion,
    required List<String> missingContext,
    required String reflectionPrompt,
  }) = _AgentReflectionResult;

  const AgentReflectionResult._();

  /// 从protobuf创建
  factory AgentReflectionResult.fromProtobuf(AgentReflectionResultPB pb) {
    return AgentReflectionResult(
      confidence: pb.confidence,
      isQuestionSolved: pb.isQuestionSolved,
      suggestion: pb.hasSuggestion() ? pb.suggestion : null,
      missingContext: pb.missingContext,
      reflectionPrompt: pb.reflectionPrompt,
    );
  }

  /// 转换为protobuf
  AgentReflectionResultPB toProtobuf() {
    final pb = AgentReflectionResultPB.create()
      ..confidence = confidence
      ..isQuestionSolved = isQuestionSolved
      ..missingContext.addAll(missingContext)
      ..reflectionPrompt = reflectionPrompt;

    if (suggestion != null) {
      pb.suggestion = suggestion!;
    }

    return pb;
  }
}