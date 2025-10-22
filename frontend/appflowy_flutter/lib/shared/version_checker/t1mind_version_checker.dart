import 'dart:convert';
import 'package:appflowy/startup/tasks/device_info_task.dart';
import 'package:appflowy_backend/log.dart';
import 'package:auto_updater/auto_updater.dart';
import 'package:http/http.dart' as http;
import 'package:universal_platform/universal_platform.dart';
import 'package:version/version.dart';

/// T1Mind专用的版本检查器，扩展现有的VersionChecker以支持GitHub API
class T1MindVersionChecker {
  factory T1MindVersionChecker() => _instance;

  T1MindVersionChecker._internal();

  static final T1MindVersionChecker _instance = T1MindVersionChecker._internal();

  /// GitHub仓库信息
  static const String _owner = 'qxk2005';
  static const String _repo = 't1mind';
  static const String _githubApiUrl = 'https://api.github.com/repos/$_owner/$_repo/releases/latest';

  /// 检查T1Mind的最新版本信息
  /// 返回GitHub API格式的版本信息，转换为AppcastItem格式
  Future<AppcastItem?> checkForT1MindUpdate() async {
    try {
      Log.info('[T1MindVersionChecker] Checking for updates from GitHub API');
      
      final response = await http.get(
        Uri.parse(_githubApiUrl),
        headers: {
          'Accept': 'application/vnd.github.v3+json',
          'User-Agent': 'T1Mind-Flutter-App',
        },
      ).timeout(const Duration(seconds: 10));

      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        final appcastItem = _parseGitHubRelease(data);
        
        if (appcastItem != null) {
          Log.info('[T1MindVersionChecker] Latest version found: ${appcastItem.displayVersionString}');
          return appcastItem;
        }
      } else if (response.statusCode == 404) {
        // 没有找到releases，尝试检查仓库是否存在
        Log.info('[T1MindVersionChecker] No releases found, checking repository existence');
        final repoResponse = await http.get(
          Uri.parse('https://api.github.com/repos/$_owner/$_repo'),
          headers: {
            'Accept': 'application/vnd.github.v3+json',
            'User-Agent': 'T1Mind-Flutter-App',
          },
        ).timeout(const Duration(seconds: 5));
        
        if (repoResponse.statusCode == 200) {
          Log.info('[T1MindVersionChecker] Repository exists but no releases available');
          return _createNoReleaseAvailableItem();
        } else {
          Log.info('[T1MindVersionChecker] Repository not found or inaccessible');
        }
      } else if (response.statusCode == 429) {
        // GitHub API rate limiting
        final retryAfter = response.headers['retry-after'];
        if (retryAfter != null) {
          final waitTime = int.tryParse(retryAfter) ?? 60;
          Log.info('[T1MindVersionChecker] Rate limited, retry after $waitTime seconds');
          await Future.delayed(Duration(seconds: waitTime));
          return checkForT1MindUpdate(); // 递归重试
        } else {
          Log.info('[T1MindVersionChecker] Rate limited without retry-after header');
        }
      } else {
        Log.info('[T1MindVersionChecker] GitHub API request failed: ${response.statusCode}');
      }
    } catch (e) {
      Log.error('[T1MindVersionChecker] Network error: $e');
    }

    return null;
  }

  /// 解析GitHub Release API响应，转换为AppcastItem格式
  AppcastItem? _parseGitHubRelease(Map<String, dynamic> data) {
    try {
      final tagName = data['tag_name'] as String?;
      final publishedAt = data['published_at'] as String?;
      final htmlUrl = data['html_url'] as String?;
      final assets = data['assets'] as List<dynamic>?;

      if (tagName == null) {
        Log.info('[T1MindVersionChecker] No tag_name found in GitHub response');
        return null;
      }

      // 查找适合当前平台的下载资源
      String? downloadUrl;
      String? os;
      
      if (assets != null && assets.isNotEmpty) {
        final platformAsset = _findPlatformAsset(assets);
        if (platformAsset != null) {
          downloadUrl = platformAsset['browser_download_url'] as String?;
          os = _getPlatformString();
        }
      }

      // 如果没有找到特定平台的资源，使用HTML URL作为备用
      if (downloadUrl == null) {
        downloadUrl = htmlUrl;
      }

      return AppcastItem.fromJson({
        'title': 'T1Mind $tagName',
        'versionString': tagName,
        'displayVersionString': tagName,
        'releaseNotesUrl': htmlUrl,
        'pubDate': publishedAt,
        'fileURL': downloadUrl ?? '',
        'os': os ?? '',
        'criticalUpdate': false,
      });
    } catch (e) {
      Log.error('[T1MindVersionChecker] Failed to parse GitHub release: $e');
      return null;
    }
  }

  /// 查找适合当前平台的下载资源
  Map<String, dynamic>? _findPlatformAsset(List<dynamic> assets) {
    for (final asset in assets) {
      final name = asset['name'] as String?;
      if (name != null) {
        // 根据文件名判断平台
        if (UniversalPlatform.isMacOS && (name.contains('.dmg') || name.contains('macos'))) {
          return asset;
        } else if (UniversalPlatform.isWindows && (name.contains('.exe') || name.contains('windows'))) {
          return asset;
        } else if (UniversalPlatform.isLinux && (name.contains('.AppImage') || name.contains('linux'))) {
          return asset;
        } else if (UniversalPlatform.isAndroid && (name.contains('.apk') || name.contains('android'))) {
          return asset;
        }
      }
    }

    // 如果没有找到特定平台的资源，返回第一个资源
    return assets.isNotEmpty ? assets.first : null;
  }

  /// 获取当前平台的字符串标识
  String _getPlatformString() {
    if (UniversalPlatform.isMacOS) return 'macos';
    if (UniversalPlatform.isWindows) return 'windows';
    if (UniversalPlatform.isLinux) return 'linux';
    if (UniversalPlatform.isAndroid) return 'android';
    return 'unknown';
  }

  /// 创建没有可用发布版本的AppcastItem
  /// 用于当仓库存在但没有releases时的情况
  AppcastItem _createNoReleaseAvailableItem() {
    return AppcastItem.fromJson({
      'title': 'T1Mind - No Releases Available',
      'versionString': '0.0.0',
      'displayVersionString': 'No releases available',
      'releaseNotesUrl': 'https://github.com/$_owner/$_repo',
      'pubDate': DateTime.now().toIso8601String(),
      'fileURL': '',
      'os': '',
      'criticalUpdate': false,
    });
  }

  /// 检查是否有更新可用
  /// 比较当前版本和GitHub上的最新版本
  Future<bool> isUpdateAvailable() async {
    try {
      final latestAppcast = await checkForT1MindUpdate();
      if (latestAppcast == null) return false;

      final currentVersion = ApplicationInfo.applicationVersion;
      final latestVersion = latestAppcast.displayVersionString ?? '';

      if (currentVersion.isEmpty || latestVersion.isEmpty) return false;
      
      // 如果没有可用版本，返回false
      if (latestVersion == 'No releases available') return false;

      // 使用semantic versioning进行比较
      final current = Version.parse(currentVersion);
      final latest = Version.parse(latestVersion);
      
      return latest > current;
    } catch (e) {
      Log.error('[T1MindVersionChecker] Failed to check update availability: $e');
      return false;
    }
  }

  /// 获取更新信息摘要
  Future<T1MindUpdateInfo?> getUpdateInfo() async {
    try {
      final latestAppcast = await checkForT1MindUpdate();
      if (latestAppcast == null) return null;

      final currentVersion = ApplicationInfo.applicationVersion;
      final latestVersion = latestAppcast.displayVersionString ?? '';
      final downloadUrl = latestAppcast.fileURL ?? '';
      final releaseNotesUrl = latestAppcast.fileURL ?? '';

      return T1MindUpdateInfo(
        currentVersion: currentVersion,
        latestVersion: latestVersion,
        downloadUrl: downloadUrl,
        releaseNotesUrl: releaseNotesUrl,
        isUpdateAvailable: await isUpdateAvailable(),
      );
    } catch (e) {
      Log.error('[T1MindVersionChecker] Failed to get update info: $e');
      return null;
    }
  }
}

/// T1Mind更新信息数据类
class T1MindUpdateInfo {
  final String currentVersion;
  final String latestVersion;
  final String downloadUrl;
  final String releaseNotesUrl;
  final bool isUpdateAvailable;

  const T1MindUpdateInfo({
    required this.currentVersion,
    required this.latestVersion,
    required this.downloadUrl,
    required this.releaseNotesUrl,
    required this.isUpdateAvailable,
  });

  @override
  String toString() {
    return 'T1MindUpdateInfo(currentVersion: $currentVersion, latestVersion: $latestVersion, isUpdateAvailable: $isUpdateAvailable)';
  }
}
