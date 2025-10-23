import 'dart:io';
import 'dart:convert';

import 'package:flutter/material.dart';

import 'package:appflowy/plugins/document/presentation/editor_plugins/image/common.dart';
import 'package:appflowy/shared/appflowy_network_image.dart';
import 'package:appflowy_backend/protobuf/flowy-user/user_profile.pb.dart';
import 'package:flowy_infra/size.dart';

@visibleForTesting
class ImageRender extends StatelessWidget {
  const ImageRender({
    super.key,
    required this.image,
    this.userProfile,
    this.fit = BoxFit.cover,
    this.borderRadius = Corners.s6Border,
  });

  final ImageBlockData image;
  final UserProfilePB? userProfile;
  final BoxFit fit;
  final BorderRadius? borderRadius;

  @override
  Widget build(BuildContext context) {
    final child = switch (image.type) {
      CustomImageType.internal || CustomImageType.external => 
        image.url.startsWith('data:') 
          ? _buildBase64Image()
          : FlowyNetworkImage(
              url: image.url,
              userProfilePB: userProfile,
              fit: fit,
            ),
      CustomImageType.local => Image.file(File(image.url), fit: fit),
    };

    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(borderRadius: borderRadius),
      child: child,
    );
  }

  Widget _buildBase64Image() {
    try {
      final base64String = image.url.split(',')[1];
      final bytes = base64Decode(base64String);
      return Image.memory(
        bytes,
        fit: fit,
        errorBuilder: (context, error, stackTrace) {
          return Container(
            width: 100,
            height: 100,
            color: Colors.grey[300],
            child: const Icon(Icons.broken_image),
          );
        },
      );
    } catch (e) {
      return Container(
        width: 100,
        height: 100,
        color: Colors.grey[300],
        child: const Icon(Icons.broken_image),
      );
    }
  }
}
