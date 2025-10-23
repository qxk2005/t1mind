import 'dart:io';
import 'dart:convert';

import 'package:flutter/widgets.dart';

enum CustomImageType {
  local,
  internal, // the images saved in self-host cloud
  external; // the images linked from network, like unsplash, https://xxx/yyy/zzz.jpg

  static CustomImageType fromIntValue(int value) {
    switch (value) {
      case 0:
        return CustomImageType.local;
      case 1:
        return CustomImageType.internal;
      case 2:
        return CustomImageType.external;
      default:
        throw UnimplementedError();
    }
  }

  int toIntValue() {
    switch (this) {
      case CustomImageType.local:
        return 0;
      case CustomImageType.internal:
        return 1;
      case CustomImageType.external:
        return 2;
    }
  }
}

class ImageBlockData {
  factory ImageBlockData.fromJson(Map<String, dynamic> json) {
    return ImageBlockData(
      url: json['url'] as String? ?? '',
      type: CustomImageType.fromIntValue(json['type'] as int),
    );
  }

  ImageBlockData({required this.url, required this.type});

  final String url;
  final CustomImageType type;

  bool get isLocal => type == CustomImageType.local;
  bool get isNotInternal => type != CustomImageType.internal;

  Map<String, dynamic> toJson() {
    return {'url': url, 'type': type.toIntValue()};
  }

  ImageProvider toImageProvider() {
    switch (type) {
      case CustomImageType.internal:
      case CustomImageType.external:
        if (url.startsWith('data:')) {
          // Handle base64 data URLs
          try {
            final base64String = url.split(',')[1];
            final bytes = base64Decode(base64String);
            return MemoryImage(bytes);
          } catch (e) {
            // Fallback to NetworkImage if base64 decoding fails
            return NetworkImage(url);
          }
        }
        return NetworkImage(url);
      case CustomImageType.local:
        return FileImage(File(url));
    }
  }
}
