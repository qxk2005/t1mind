import 'package:appflowy/plugins/ai_chat/application/chat_entity.dart';

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

  /// 设置指定消息的 sources（智能合并，保留现有sources）
  /// 🔧 修复：优先保留有名称的文档（来自@mention），对于相同ID的文档，优先使用有名称的版本
  void setSources(String chatId, String questionId, List<ChatMessageRefSource> sources) {
    final key = _generateKey(chatId, questionId);
    final existingSources = _sourcesMap[key] ?? [];
    
    // 🔧 修复：智能合并sources而不是完全替换
    // 使用 source + id 作为唯一标识进行去重和合并
    // 对于相同ID的文档，优先保留有名称的版本（来自@mention）
    final Map<String, ChatMessageRefSource> sourceMap = {};
    
    // 首先保留现有的sources
    for (final existingSource in existingSources) {
      final sourceKey = '${existingSource.source}:${existingSource.id}';
      sourceMap[sourceKey] = existingSource;
    }
    
    // 然后添加新的sources，对于相同ID的文档，优先使用有名称的版本
    for (final newSource in sources) {
      final sourceKey = '${newSource.source}:${newSource.id}';
      final existing = sourceMap[sourceKey];
      
      // 如果新source有名称且现有source没有名称，或者两者都没有名称，则替换
      // 如果现有source有名称，则保留现有的（不覆盖）
      if (existing == null) {
        sourceMap[sourceKey] = newSource;
      } else {
        // 优先保留有名称的版本
        final newHasName = newSource.name.isNotEmpty && 
                          newSource.name != "Loading..." && 
                          newSource.name != "加载中...";
        final existingHasName = existing.name.isNotEmpty && 
                               existing.name != "Loading..." && 
                               existing.name != "加载中...";
        
        if (newHasName && !existingHasName) {
          // 新source有名称，现有source没有名称，使用新的
          sourceMap[sourceKey] = newSource;
        }
        // 其他情况保持现有的（existingHasName时保留现有的，都不有名称时也保留现有的）
      }
    }
    
    // 更新sources列表
    final mergedSources = sourceMap.values.toList();
    _sourcesMap[key] = mergedSources;
  }

  /// 追加 sources 到指定消息（累积）
  /// 🔧 修复：对于相同ID的文档，优先保留有名称的版本（来自@mention）
  void appendSources(String chatId, String questionId, List<ChatMessageRefSource> newSources) {
    final key = _generateKey(chatId, questionId);
    final existingSources = _sourcesMap[key] ?? [];
    
    // 使用 source + id 作为唯一标识进行去重，同时优先保留有名称的版本
    final Map<String, ChatMessageRefSource> sourceMap = {};
    
    // 首先添加现有的sources
    for (final existingSource in existingSources) {
      final sourceKey = '${existingSource.source}:${existingSource.id}';
      sourceMap[sourceKey] = existingSource;
    }
    
    // 然后添加新的sources，对于相同ID的文档，优先使用有名称的版本
    for (final newSource in newSources) {
      final sourceKey = '${newSource.source}:${newSource.id}';
      final existing = sourceMap[sourceKey];
      
      if (existing == null) {
        // 新文档，直接添加
        sourceMap[sourceKey] = newSource;
      } else {
        // 已存在，优先保留有名称的版本
        final newHasName = newSource.name.isNotEmpty && 
                          newSource.name != "Loading..." && 
                          newSource.name != "加载中...";
        final existingHasName = existing.name.isNotEmpty && 
                               existing.name != "Loading..." && 
                               existing.name != "加载中...";
        
        if (newHasName && !existingHasName) {
          // 新source有名称，现有source没有名称，使用新的
          sourceMap[sourceKey] = newSource;
        }
        // 其他情况保持现有的
      }
    }
    
    _sourcesMap[key] = sourceMap.values.toList();
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

