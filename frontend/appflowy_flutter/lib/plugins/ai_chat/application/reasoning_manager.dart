import 'dart:collection';

/// 全局推理文本管理器，用于在不同消息之间共享推理信息
class ReasoningManager {
  static final ReasoningManager _instance = ReasoningManager._internal();
  factory ReasoningManager() => _instance;
  ReasoningManager._internal();

  // 存储每个聊天会话的推理信息
  final HashMap<String, String> _reasoningTexts = HashMap();
  final HashMap<String, bool> _reasoningCompleteStatus = HashMap();

  /// 获取指定聊天的推理文本
  String? getReasoningText(String chatId) {
    return _reasoningTexts[chatId];
  }

  /// 设置指定聊天的推理文本
  void setReasoningText(String chatId, String reasoningText) {
    _reasoningTexts[chatId] = reasoningText;
  }

  /// 追加推理文本
  void appendReasoningText(String chatId, String deltaText) {
    final currentText = _reasoningTexts[chatId] ?? "";
    _reasoningTexts[chatId] = currentText + deltaText;
  }

  /// 获取推理完成状态
  bool isReasoningComplete(String chatId) {
    return _reasoningCompleteStatus[chatId] ?? false;
  }

  /// 设置推理完成状态
  void setReasoningComplete(String chatId, bool isComplete) {
    _reasoningCompleteStatus[chatId] = isComplete;
  }

  /// 清除指定聊天的推理信息
  void clearReasoning(String chatId) {
    _reasoningTexts.remove(chatId);
    _reasoningCompleteStatus.remove(chatId);
  }

  /// 清除所有推理信息
  void clearAll() {
    _reasoningTexts.clear();
    _reasoningCompleteStatus.clear();
  }
}
