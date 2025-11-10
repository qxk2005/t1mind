//
//  Generated code. Do not modify.
//  source: notification.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class FolderNotification extends $pb.ProtobufEnum {
  static const FolderNotification Unknown = FolderNotification._(0, _omitEnumNames ? '' : 'Unknown');
  static const FolderNotification DidCreateWorkspace = FolderNotification._(1, _omitEnumNames ? '' : 'DidCreateWorkspace');
  static const FolderNotification DidUpdateWorkspace = FolderNotification._(2, _omitEnumNames ? '' : 'DidUpdateWorkspace');
  static const FolderNotification DidUpdateWorkspaceViews = FolderNotification._(3, _omitEnumNames ? '' : 'DidUpdateWorkspaceViews');
  static const FolderNotification DidUpdateWorkspaceSetting = FolderNotification._(4, _omitEnumNames ? '' : 'DidUpdateWorkspaceSetting');
  static const FolderNotification DidUpdateView = FolderNotification._(10, _omitEnumNames ? '' : 'DidUpdateView');
  static const FolderNotification DidUpdateChildViews = FolderNotification._(11, _omitEnumNames ? '' : 'DidUpdateChildViews');
  static const FolderNotification DidDeleteView = FolderNotification._(12, _omitEnumNames ? '' : 'DidDeleteView');
  static const FolderNotification DidRestoreView = FolderNotification._(13, _omitEnumNames ? '' : 'DidRestoreView');
  static const FolderNotification DidMoveViewToTrash = FolderNotification._(14, _omitEnumNames ? '' : 'DidMoveViewToTrash');
  static const FolderNotification DidUpdateTrash = FolderNotification._(15, _omitEnumNames ? '' : 'DidUpdateTrash');
  static const FolderNotification DidUpdateFolderSnapshotState = FolderNotification._(16, _omitEnumNames ? '' : 'DidUpdateFolderSnapshotState');
  static const FolderNotification DidUpdateFolderSyncUpdate = FolderNotification._(17, _omitEnumNames ? '' : 'DidUpdateFolderSyncUpdate');
  static const FolderNotification DidFavoriteView = FolderNotification._(36, _omitEnumNames ? '' : 'DidFavoriteView');
  static const FolderNotification DidUnfavoriteView = FolderNotification._(37, _omitEnumNames ? '' : 'DidUnfavoriteView');
  static const FolderNotification DidUpdateRecentViews = FolderNotification._(38, _omitEnumNames ? '' : 'DidUpdateRecentViews');
  static const FolderNotification DidUpdateSectionViews = FolderNotification._(39, _omitEnumNames ? '' : 'DidUpdateSectionViews');
  static const FolderNotification DidUpdateSharedViews = FolderNotification._(40, _omitEnumNames ? '' : 'DidUpdateSharedViews');
  static const FolderNotification DidUpdateSharedUsers = FolderNotification._(41, _omitEnumNames ? '' : 'DidUpdateSharedUsers');

  static const $core.List<FolderNotification> values = <FolderNotification> [
    Unknown,
    DidCreateWorkspace,
    DidUpdateWorkspace,
    DidUpdateWorkspaceViews,
    DidUpdateWorkspaceSetting,
    DidUpdateView,
    DidUpdateChildViews,
    DidDeleteView,
    DidRestoreView,
    DidMoveViewToTrash,
    DidUpdateTrash,
    DidUpdateFolderSnapshotState,
    DidUpdateFolderSyncUpdate,
    DidFavoriteView,
    DidUnfavoriteView,
    DidUpdateRecentViews,
    DidUpdateSectionViews,
    DidUpdateSharedViews,
    DidUpdateSharedUsers,
  ];

  static final $core.Map<$core.int, FolderNotification> _byValue = $pb.ProtobufEnum.initByValue(values);
  static FolderNotification? valueOf($core.int value) => _byValue[value];

  const FolderNotification._($core.int v, $core.String n) : super(v, n);
}


const _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
