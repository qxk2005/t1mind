import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';
import 'package:appflowy_backend/log.dart';

/// 全局管理器，用于累积和持久化聊天消息的引用来源（sources）
/// 
/// 该管理器解决了多路召回场景下，引用来源在消息重新加载后丢失的问题：
/// - 在流式传输过程中，累积所有工具的引用
/// - 当消息被重新加载时，恢复累积的引用
/// 
/// 使用 chatId + questionId 作为唯一标识来存储和检索 sources
class SourcesManager {
  // 单例模式
  static final SourcesManager _instance = SourcesManager._internal();
  factory SourcesManager() => _instance;
  SourcesManager._internal();

  // 使用 Map 存储每个消息的累积 sources
  // Key: "chatId:questionId"
  final Map<String, List<ChatMessageRefSource>> _sourcesMap = {};

  /// 生成唯一键
  String _generateKey(String chatId, String questionId) {
    return '$chatId:$questionId';
  }

  /// 获取指定消息的累积 sources
  List<ChatMessageRefSource>? getSources(String chatId, String questionId) {
    final key = _generateKey(chatId, questionId);
    final sources = _sourcesMap[key];
    return sources;
  }

  /// 设置指定消息的 sources（完全替换）
  void setSources(String chatId, String questionId, List<ChatMessageRefSource> sources) {
    final key = _generateKey(chatId, questionId);
    _sourcesMap[key] = List.from(sources);
    
    // 输出详细信息
    final webCount = sources.where((s) => s.source == 'web').length;
    final mcpCount = sources.where((s) => s.source.startsWith('mcp')).length;
    final docCount = sources.where((s) => s.source == 'appflowy').length;
    
  }

  /// 追加 sources 到指定消息（累积）
  void appendSources(String chatId, String questionId, List<ChatMessageRefSource> newSources) {
    final key = _generateKey(chatId, questionId);
    final existingSources = _sourcesMap[key] ?? [];
    
    // 使用 source + id 作为唯一标识进行去重
    final Set<String> existingKeys = existingSources
        .map((source) => '${source.source}:${source.id}')
        .toSet();
    
    int addedCount = 0;
    for (final newSource in newSources) {
      final sourceKey = '${newSource.source}:${newSource.id}';
      if (!existingKeys.contains(sourceKey)) {
        existingSources.add(newSource);
        existingKeys.add(sourceKey);
        addedCount++;
      }
    }
    
    _sourcesMap[key] = existingSources;
  }

  /// 清除指定消息的 sources
  void clearSources(String chatId, String questionId) {
    final key = _generateKey(chatId, questionId);
    _sourcesMap.remove(key);
  }

  /// 清除指定 chat 的所有 sources
  void clearChatSources(String chatId) {
    final keysToRemove = _sourcesMap.keys
        .where((key) => key.startsWith('$chatId:'))
        .toList();
    
    for (final key in keysToRemove) {
      _sourcesMap.remove(key);
    }
    
  }

  /// 获取指定 chat 的最近问题ID（用于没有 reply_message_id 的AI消息）
  String? getMostRecentQuestionId(String chatId) {
    final chatKeys = _sourcesMap.keys
        .where((key) => key.startsWith('$chatId:'))
        .toList();
    
    if (chatKeys.isEmpty) {
      return null;
    }
    
    // 获取最近的问题ID（假设ID是递增的）
    final questionIds = chatKeys
        .map((key) => key.split(':')[1])
        .where((id) => id.isNotEmpty)
        .toList();
    
    if (questionIds.isEmpty) {
      return null;
    }
    
    // 按数字大小排序，取最大值
    questionIds.sort((a, b) {
      final aNum = int.tryParse(a) ?? 0;
      final bNum = int.tryParse(b) ?? 0;
      return bNum.compareTo(aNum); // 降序排列
    });
    
    final recentQuestionId = questionIds.first;
    return recentQuestionId;
  }

  /// 获取当前管理的消息数量
  int get managedMessagesCount => _sourcesMap.length;
}

