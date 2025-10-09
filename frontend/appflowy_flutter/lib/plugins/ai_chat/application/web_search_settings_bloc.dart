import 'dart:async';

import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:bloc/bloc.dart';
import 'package:fixnum/fixnum.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'web_search_settings_bloc.freezed.dart';

/// 网络搜索设置BLoC - 管理网络搜索配置的状态和业务逻辑
class WebSearchSettingsBloc extends Bloc<WebSearchSettingsEvent, WebSearchSettingsState> {
  WebSearchSettingsBloc() : super(const WebSearchSettingsState()) {
    _dispatch();
  }

  void _dispatch() {
    // 异步事件处理
    on<_Started>((event, emit) async => await _handleStarted(emit));
    on<_LoadProviderList>((event, emit) async => await _handleLoadProviderList(emit));
    on<_AddProvider>((event, emit) async => await _handleAddProvider(event.config, emit));
    on<_UpdateProvider>((event, emit) async => await _handleUpdateProvider(event.config, emit));
    on<_RemoveProvider>((event, emit) async => await _handleRemoveProvider(event.providerId, emit));
    on<_TestProvider>((event, emit) async => await _handleTestProvider(event.providerId, emit));
    on<_ActivateProvider>((event, emit) async => await _handleActivateProvider(event.providerId, emit));
    on<_DeactivateProvider>((event, emit) async => await _handleDeactivateProvider(event.providerId, emit));
    on<_LoadGlobalConfig>((event, emit) async => await _handleLoadGlobalConfig(emit));
    on<_UpdateGlobalConfig>((event, emit) async => await _handleUpdateGlobalConfig(event.config, emit));
    on<_GetCacheStats>((event, emit) async => await _handleGetCacheStats(emit));
    on<_ClearCache>((event, emit) async => await _handleClearCache(emit));
    
    // 同步事件处理
    on<_DidReceiveProviderList>((event, emit) => _handleDidReceiveProviderList(event.providers, emit));
    on<_DidReceiveGlobalConfig>((event, emit) => _handleDidReceiveGlobalConfig(event.config, emit));
    on<_DidReceiveCacheStats>((event, emit) => _handleDidReceiveCacheStats(event.stats, emit));
    on<_DidReceiveError>((event, emit) => _handleDidReceiveError(event.error, emit));
  }

  /// 处理初始化事件
  Future<void> _handleStarted(Emitter<WebSearchSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadProviderListAndEmit(emit);
    await _loadGlobalConfigAndEmit(emit);
    await _loadCacheStatsAndEmit(emit);
  }

  /// 处理加载供应商列表事件
  Future<void> _handleLoadProviderList(Emitter<WebSearchSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadProviderListAndEmit(emit);
  }

  /// 处理添加供应商事件
  Future<void> _handleAddProvider(
    WebSearchProviderConfigPB config,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final request = CreateWebSearchProviderRequestPB()
        ..name = config.name
        ..providerType = config.providerType
        ..description = config.description
        ..icon = config.icon
        ..apiKey = config.apiKey
        ..baseUrl = config.baseUrl
        ..maxResults = config.maxResults
        ..timeoutSeconds = config.timeoutSeconds
        ..metadata.addAll(config.metadata);
      
      final result = await AIEventCreateWebSearchProvider(request).send();
      
      final isSuccess = result.fold(
        (success) {
          Log.info('网络搜索供应商添加成功: ${config.name}');
          return true;
        },
        (error) {
          Log.error('添加网络搜索供应商失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        await _loadProviderListAndEmit(emit);
      } else {
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '添加供应商失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('添加网络搜索供应商异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '添加供应商异常: $e',
        ));
      }
    }
  }

  /// 处理更新供应商事件
  Future<void> _handleUpdateProvider(
    WebSearchProviderConfigPB config,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final request = UpdateWebSearchProviderRequestPB()
        ..id = config.id
        ..name = config.name
        ..description = config.description
        ..icon = config.icon
        ..apiKey = config.apiKey
        ..baseUrl = config.baseUrl
        ..maxResults = config.maxResults
        ..timeoutSeconds = config.timeoutSeconds
        ..metadata.addAll(config.metadata);
      
      final result = await AIEventUpdateWebSearchProvider(request).send();
      
      final isSuccess = result.fold(
        (success) {
          Log.info('网络搜索供应商更新成功: ${config.name}');
          return true;
        },
        (error) {
          Log.error('更新网络搜索供应商失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        await _loadProviderListAndEmit(emit);
      } else {
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '更新供应商失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('更新网络搜索供应商异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '更新供应商异常: $e',
        ));
      }
    }
  }

  /// 处理删除供应商事件
  Future<void> _handleRemoveProvider(
    String providerId,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final request = DeleteWebSearchProviderRequestPB()..id = providerId;
      final result = await AIEventDeleteWebSearchProvider(request).send();
      
      final isSuccess = result.fold(
        (success) {
          Log.info('网络搜索供应商删除成功: $providerId');
          return true;
        },
        (error) {
          Log.error('删除网络搜索供应商失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        await _loadProviderListAndEmit(emit);
      } else {
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '删除供应商失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('删除网络搜索供应商异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '删除供应商异常: $e',
        ));
      }
    }
  }

  /// 处理测试供应商事件
  Future<void> _handleTestProvider(
    String providerId,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(
      testingProviders: {...state.testingProviders, providerId},
      error: null,
    ));
    
    try {
      final request = TestWebSearchProviderRequestPB()..id = providerId;
      final result = await AIEventTestWebSearchProvider(request).send();
      
      if (emit.isDone) return;
      
      result.fold(
        (response) {
          Log.info('网络搜索供应商测试完成: $providerId, 成功: ${response.success}');
          // 更新供应商列表
          final updatedProviders = state.providers.map((p) {
            if (p.id == providerId) {
              final updatedProvider = p.clone();
              updatedProvider.testStatus = response.success ? ProviderTestStatusPB.TestPassed : ProviderTestStatusPB.TestFailed;
              return updatedProvider;
            }
            return p;
          }).toList();
          
          final testingProviders = Set<String>.from(state.testingProviders);
          testingProviders.remove(providerId);
          emit(state.copyWith(
            providers: updatedProviders,
            testingProviders: testingProviders,
          ));
        },
        (error) {
          Log.error('测试网络搜索供应商失败: $error');
          final testingProviders = Set<String>.from(state.testingProviders);
          testingProviders.remove(providerId);
          emit(state.copyWith(
            testingProviders: testingProviders,
            error: '测试供应商失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('测试网络搜索供应商异常: $e');
      if (!emit.isDone) {
        final testingProviders = Set<String>.from(state.testingProviders);
        testingProviders.remove(providerId);
        emit(state.copyWith(
          testingProviders: testingProviders,
          error: '测试供应商异常: $e',
        ));
      }
    }
  }

  /// 处理激活供应商事件
  Future<void> _handleActivateProvider(
    String providerId,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(
      activatingProviders: {...state.activatingProviders, providerId},
      error: null,
    ));
    
    try {
      // 先获取供应商配置
      final getRequest = GetWebSearchProviderRequestPB()..id = providerId;
      final getResult = await AIEventGetWebSearchProvider(getRequest).send();
      
      if (emit.isDone) return;
      
      getResult.fold(
        (provider) async {
          // 更新供应商为激活状态
          final updateRequest = UpdateWebSearchProviderRequestPB()
            ..id = provider.id
            ..name = provider.name
            ..description = provider.description
            ..icon = provider.icon
            ..apiKey = provider.apiKey
            ..baseUrl = provider.baseUrl
            ..maxResults = provider.maxResults
            ..timeoutSeconds = provider.timeoutSeconds
            ..isActive = true
            ..metadata.addAll(provider.metadata);
          
          final updateResult = await AIEventUpdateWebSearchProvider(updateRequest).send();
          
          if (emit.isDone) return;
          
          updateResult.fold(
            (updatedProvider) {
              Log.info('网络搜索供应商激活成功: $providerId');
              // 重新加载供应商列表
              add(const WebSearchSettingsEvent.loadProviderList());
            },
            (error) {
              Log.error('激活网络搜索供应商失败: $error');
              final activatingProviders = Set<String>.from(state.activatingProviders);
              activatingProviders.remove(providerId);
              emit(state.copyWith(
                activatingProviders: activatingProviders,
                error: '激活供应商失败: ${error.msg}',
              ));
            },
          );
        },
        (error) {
          Log.error('获取网络搜索供应商失败: $error');
          final activatingProviders = Set<String>.from(state.activatingProviders);
          activatingProviders.remove(providerId);
          emit(state.copyWith(
            activatingProviders: activatingProviders,
            error: '获取供应商失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('激活网络搜索供应商异常: $e');
      if (!emit.isDone) {
        final activatingProviders = Set<String>.from(state.activatingProviders);
        activatingProviders.remove(providerId);
        emit(state.copyWith(
          activatingProviders: activatingProviders,
          error: '激活供应商异常: $e',
        ));
      }
    }
  }

  /// 处理停用供应商事件
  Future<void> _handleDeactivateProvider(
    String providerId,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(
      activatingProviders: {...state.activatingProviders, providerId},
      error: null,
    ));
    
    try {
      // 先获取供应商配置
      final getRequest = GetWebSearchProviderRequestPB()..id = providerId;
      final getResult = await AIEventGetWebSearchProvider(getRequest).send();
      
      if (emit.isDone) return;
      
      getResult.fold(
        (provider) async {
          // 更新供应商为停用状态
          final updateRequest = UpdateWebSearchProviderRequestPB()
            ..id = provider.id
            ..name = provider.name
            ..description = provider.description
            ..icon = provider.icon
            ..apiKey = provider.apiKey
            ..baseUrl = provider.baseUrl
            ..maxResults = provider.maxResults
            ..timeoutSeconds = provider.timeoutSeconds
            ..isActive = false
            ..metadata.addAll(provider.metadata);
          
          final updateResult = await AIEventUpdateWebSearchProvider(updateRequest).send();
          
          if (emit.isDone) return;
          
          updateResult.fold(
            (updatedProvider) {
              Log.info('网络搜索供应商停用成功: $providerId');
              // 重新加载供应商列表
              add(const WebSearchSettingsEvent.loadProviderList());
            },
            (error) {
              Log.error('停用网络搜索供应商失败: $error');
              final activatingProviders = Set<String>.from(state.activatingProviders);
              activatingProviders.remove(providerId);
              emit(state.copyWith(
                activatingProviders: activatingProviders,
                error: '停用供应商失败: ${error.msg}',
              ));
            },
          );
        },
        (error) {
          Log.error('获取网络搜索供应商失败: $error');
          final activatingProviders = Set<String>.from(state.activatingProviders);
          activatingProviders.remove(providerId);
          emit(state.copyWith(
            activatingProviders: activatingProviders,
            error: '获取供应商失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('停用网络搜索供应商异常: $e');
      if (!emit.isDone) {
        final activatingProviders = Set<String>.from(state.activatingProviders);
        activatingProviders.remove(providerId);
        emit(state.copyWith(
          activatingProviders: activatingProviders,
          error: '停用供应商异常: $e',
        ));
      }
    }
  }

  /// 处理加载全局配置事件
  Future<void> _handleLoadGlobalConfig(Emitter<WebSearchSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    await _loadGlobalConfigAndEmit(emit);
  }

  /// 处理更新全局配置事件
  Future<void> _handleUpdateGlobalConfig(
    WebSearchGlobalConfigPB config,
    Emitter<WebSearchSettingsState> emit,
  ) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final request = UpdateWebSearchGlobalConfigRequestPB()
        ..enabled = config.enabled
        ..defaultProviderId = config.defaultProviderId
        ..defaultMaxResults = config.defaultMaxResults
        ..defaultTimeoutSeconds = config.defaultTimeoutSeconds
        ..enableCache = config.enableCache
        ..cacheExpirySeconds = config.cacheExpirySeconds
        ..enableContentFilter = config.enableContentFilter
        ..contentFilterRules.addAll(config.contentFilterRules)
        ..metadata.addAll(config.metadata);
      
      final result = await AIEventUpdateWebSearchGlobalConfig(request).send();
      
      final isSuccess = result.fold(
        (success) {
          Log.info('网络搜索全局配置更新成功');
          return true;
        },
        (error) {
          Log.error('更新网络搜索全局配置失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        await _loadGlobalConfigAndEmit(emit);
      } else {
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '更新全局配置失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('更新网络搜索全局配置异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '更新全局配置异常: $e',
        ));
      }
    }
  }

  /// 处理获取缓存统计事件
  Future<void> _handleGetCacheStats(Emitter<WebSearchSettingsState> emit) async {
    emit(state.copyWith(isLoading: true, error: null));
    
    try {
      final result = await AIEventGetWebSearchCacheStats().send();
      
      if (emit.isDone) return;
      
      result.fold(
        (stats) {
          Log.info('获取网络搜索缓存统计成功');
          add(WebSearchSettingsEvent.didReceiveCacheStats(stats));
        },
        (error) {
          Log.error('获取网络搜索缓存统计失败: $error');
          emit(state.copyWith(
            isLoading: false,
            error: '获取缓存统计失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('获取网络搜索缓存统计异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isLoading: false,
          error: '获取缓存统计异常: $e',
        ));
      }
    }
  }

  /// 处理清空缓存事件
  Future<void> _handleClearCache(Emitter<WebSearchSettingsState> emit) async {
    emit(state.copyWith(isOperating: true, error: null));
    
    try {
      final result = await AIEventClearWebSearchCache().send();
      
      final isSuccess = result.fold(
        (success) {
          Log.info('网络搜索缓存清空成功');
          return true;
        },
        (error) {
          Log.error('清空网络搜索缓存失败: $error');
          return false;
        },
      );
      
      if (isSuccess) {
        // 重新获取缓存统计
        await _handleGetCacheStats(emit);
      } else {
        final errorMsg = result.fold(
          (success) => '',
          (error) => error.msg,
        );
        if (!emit.isDone) {
          emit(state.copyWith(
            isOperating: false,
            error: '清空缓存失败: $errorMsg',
          ));
        }
      }
    } catch (e) {
      Log.error('清空网络搜索缓存异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isOperating: false,
          error: '清空缓存异常: $e',
        ));
      }
    }
  }

  /// 处理接收到供应商列表事件
  void _handleDidReceiveProviderList(
    WebSearchProviderListPB providers,
    Emitter<WebSearchSettingsState> emit,
  ) {
    Log.info('接收到网络搜索供应商列表，数量: ${providers.providers.length}');
    emit(state.copyWith(
      providers: providers.providers,
      isLoading: false,
      error: null,
    ));
  }


  /// 处理接收到全局配置事件
  void _handleDidReceiveGlobalConfig(
    WebSearchGlobalConfigPB config,
    Emitter<WebSearchSettingsState> emit,
  ) {
    Log.info('接收到网络搜索全局配置');
    emit(state.copyWith(
      globalConfig: config,
      isLoading: false,
      error: null,
    ));
  }

  /// 处理接收到缓存统计事件
  void _handleDidReceiveCacheStats(
    WebSearchCacheStatsPB stats,
    Emitter<WebSearchSettingsState> emit,
  ) {
    Log.info('接收到网络搜索缓存统计');
    emit(state.copyWith(
      cacheStats: stats,
      isLoading: false,
      error: null,
    ));
  }

  /// 处理接收到错误事件
  void _handleDidReceiveError(
    String error,
    Emitter<WebSearchSettingsState> emit,
  ) {
    Log.error('网络搜索设置错误: $error');
    emit(state.copyWith(
      isLoading: false,
      isOperating: false,
      testingProviders: {},
      activatingProviders: {},
      error: error,
    ));
  }

  /// 加载供应商列表并直接emit（避免嵌套事件）
  Future<void> _loadProviderListAndEmit(Emitter<WebSearchSettingsState> emit) async {
    try {
      Log.info('开始加载网络搜索供应商列表...');
      final result = await AIEventGetWebSearchProviderList().send();
      
      Log.info('网络搜索供应商列表请求完成，检查emit状态: isDone=${emit.isDone}');
      
      if (emit.isDone) {
        Log.warn('emit已完成，无法更新状态');
        return;
      }
      
      result.fold(
        (providers) {
          Log.info('接收到网络搜索供应商列表，数量: ${providers.providers.length}');
          emit(state.copyWith(
            providers: providers.providers,
            isLoading: false,
            isOperating: false,
            error: null,
          ));
        },
        (error) {
          Log.error('加载网络搜索供应商列表失败: $error');
          emit(state.copyWith(
            isLoading: false,
            isOperating: false,
            error: '加载供应商列表失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('加载网络搜索供应商列表异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isLoading: false,
          isOperating: false,
          error: '加载供应商列表异常: $e',
        ));
      }
    }
  }

  /// 加载全局配置并直接emit（避免嵌套事件）
  Future<void> _loadGlobalConfigAndEmit(Emitter<WebSearchSettingsState> emit) async {
    try {
      Log.info('开始加载网络搜索全局配置...');
      final result = await AIEventGetWebSearchGlobalConfig().send();
      
      Log.info('网络搜索全局配置请求完成，检查emit状态: isDone=${emit.isDone}');
      
      if (emit.isDone) {
        Log.warn('emit已完成，无法更新状态');
        return;
      }
      
      result.fold(
        (config) {
          Log.info('接收到网络搜索全局配置');
          emit(state.copyWith(
            globalConfig: config,
            isLoading: false,
            isOperating: false,
            error: null,
          ));
        },
        (error) {
          Log.error('加载网络搜索全局配置失败: $error');
          emit(state.copyWith(
            isLoading: false,
            isOperating: false,
            error: '加载全局配置失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('加载网络搜索全局配置异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isLoading: false,
          isOperating: false,
          error: '加载全局配置异常: $e',
        ));
      }
    }
  }

  /// 加载缓存统计并直接emit（避免嵌套事件）
  Future<void> _loadCacheStatsAndEmit(Emitter<WebSearchSettingsState> emit) async {
    try {
      Log.info('开始加载网络搜索缓存统计...');
      final result = await AIEventGetWebSearchCacheStats().send();
      
      Log.info('网络搜索缓存统计请求完成，检查emit状态: isDone=${emit.isDone}');
      
      if (emit.isDone) {
        Log.warn('emit已完成，无法更新状态');
        return;
      }
      
      result.fold(
        (stats) {
          Log.info('接收到网络搜索缓存统计');
          emit(state.copyWith(
            cacheStats: stats,
            isLoading: false,
            isOperating: false,
            error: null,
          ));
        },
        (error) {
          Log.error('加载网络搜索缓存统计失败: $error');
          emit(state.copyWith(
            isLoading: false,
            isOperating: false,
            error: '加载缓存统计失败: ${error.msg}',
          ));
        },
      );
    } catch (e) {
      Log.error('加载网络搜索缓存统计异常: $e');
      if (!emit.isDone) {
        emit(state.copyWith(
          isLoading: false,
          isOperating: false,
          error: '加载缓存统计异常: $e',
        ));
      }
    }
  }

  /// 获取供应商状态（从供应商列表中获取）
  bool isProviderActive(String providerId) {
    return state.providers.firstWhere(
      (p) => p.id == providerId,
      orElse: () => WebSearchProviderConfigPB(),
    ).isActive;
  }

  /// 获取供应商是否正在测试
  bool isProviderTesting(String providerId) {
    return state.testingProviders.contains(providerId);
  }

  /// 获取供应商是否正在激活/停用
  bool isProviderActivating(String providerId) {
    return state.activatingProviders.contains(providerId);
  }

  /// 获取供应商测试状态（从供应商列表中获取）
  ProviderTestStatusPB? getProviderTestStatus(String providerId) {
    final provider = state.providers.firstWhere(
      (p) => p.id == providerId,
      orElse: () => WebSearchProviderConfigPB(),
    );
    return provider.hasTestStatus() ? provider.testStatus : null;
  }

  /// 获取供应商错误信息
  String? getProviderError(String providerId) {
    // WebSearchProviderConfigPB 没有 errorMessage 字段
    // 如果需要错误信息，应该从测试结果中获取
    return null;
  }

  /// 获取激活的供应商列表
  List<WebSearchProviderConfigPB> getActiveProviders() {
    return state.providers.where((provider) => provider.isActive).toList();
  }

  /// 获取默认供应商
  WebSearchProviderConfigPB? getDefaultProvider() {
    if (state.globalConfig?.defaultProviderId == null) return null;
    
    return state.providers.firstWhere(
      (provider) => provider.id == state.globalConfig!.defaultProviderId,
      orElse: () => state.providers.first,
    );
  }

  /// 检查网络搜索是否启用
  bool isWebSearchEnabled() {
    return state.globalConfig?.enabled ?? false;
  }

  /// 获取缓存命中率
  double getCacheHitRate() {
    return state.cacheStats?.hitRate ?? 0.0;
  }

  /// 获取缓存大小（MB）
  double getCacheSizeMB() {
    final sizeBytes = state.cacheStats?.cacheSizeBytes ?? Int64.ZERO;
    return sizeBytes.toInt() / (1024 * 1024);
  }
}

/// 网络搜索设置事件定义
@freezed
class WebSearchSettingsEvent with _$WebSearchSettingsEvent {
  /// 开始初始化
  const factory WebSearchSettingsEvent.started() = _Started;

  /// 加载供应商列表
  const factory WebSearchSettingsEvent.loadProviderList() = _LoadProviderList;

  /// 添加供应商
  const factory WebSearchSettingsEvent.addProvider(WebSearchProviderConfigPB config) = _AddProvider;

  /// 更新供应商
  const factory WebSearchSettingsEvent.updateProvider(WebSearchProviderConfigPB config) = _UpdateProvider;

  /// 删除供应商
  const factory WebSearchSettingsEvent.removeProvider(String providerId) = _RemoveProvider;

  /// 测试供应商
  const factory WebSearchSettingsEvent.testProvider(String providerId) = _TestProvider;

  /// 激活供应商
  const factory WebSearchSettingsEvent.activateProvider(String providerId) = _ActivateProvider;

  /// 停用供应商
  const factory WebSearchSettingsEvent.deactivateProvider(String providerId) = _DeactivateProvider;

  /// 加载全局配置
  const factory WebSearchSettingsEvent.loadGlobalConfig() = _LoadGlobalConfig;

  /// 更新全局配置
  const factory WebSearchSettingsEvent.updateGlobalConfig(WebSearchGlobalConfigPB config) = _UpdateGlobalConfig;

  /// 获取缓存统计
  const factory WebSearchSettingsEvent.getCacheStats() = _GetCacheStats;

  /// 清空缓存
  const factory WebSearchSettingsEvent.clearCache() = _ClearCache;

  /// 接收到供应商列表
  const factory WebSearchSettingsEvent.didReceiveProviderList(WebSearchProviderListPB providers) = _DidReceiveProviderList;

  /// 接收到供应商状态

  /// 接收到全局配置
  const factory WebSearchSettingsEvent.didReceiveGlobalConfig(WebSearchGlobalConfigPB config) = _DidReceiveGlobalConfig;

  /// 接收到缓存统计
  const factory WebSearchSettingsEvent.didReceiveCacheStats(WebSearchCacheStatsPB stats) = _DidReceiveCacheStats;

  /// 接收到错误
  const factory WebSearchSettingsEvent.didReceiveError(String error) = _DidReceiveError;
}

/// 网络搜索设置状态定义
@freezed
class WebSearchSettingsState with _$WebSearchSettingsState {
  const factory WebSearchSettingsState({
    /// 供应商列表
    @Default([]) List<WebSearchProviderConfigPB> providers,
    
    /// 供应商状态映射
    @Default({}) Map<String, ProviderTestStatusPB> providerStatuses,
    
    /// 全局配置
    WebSearchGlobalConfigPB? globalConfig,
    
    /// 缓存统计
    WebSearchCacheStatsPB? cacheStats,
    
    /// 正在测试的供应商ID集合
    @Default({}) Set<String> testingProviders,
    
    /// 正在激活/停用的供应商ID集合
    @Default({}) Set<String> activatingProviders,
    
    /// 是否正在加载
    @Default(false) bool isLoading,
    
    /// 是否正在执行操作（添加、更新、删除）
    @Default(false) bool isOperating,
    
    /// 错误信息
    String? error,
  }) = _WebSearchSettingsState;
}
