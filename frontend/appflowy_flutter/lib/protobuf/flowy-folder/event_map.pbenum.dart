//
//  Generated code. Do not modify.
//  source: event_map.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class FolderEvent extends $pb.ProtobufEnum {
  static const FolderEvent CreateFolderWorkspace = FolderEvent._(0, _omitEnumNames ? '' : 'CreateFolderWorkspace');
  static const FolderEvent GetCurrentWorkspaceSetting = FolderEvent._(1, _omitEnumNames ? '' : 'GetCurrentWorkspaceSetting');
  static const FolderEvent ReadCurrentWorkspace = FolderEvent._(2, _omitEnumNames ? '' : 'ReadCurrentWorkspace');
  static const FolderEvent DeleteWorkspace = FolderEvent._(3, _omitEnumNames ? '' : 'DeleteWorkspace');
  static const FolderEvent ReadWorkspaceViews = FolderEvent._(5, _omitEnumNames ? '' : 'ReadWorkspaceViews');
  static const FolderEvent CreateView = FolderEvent._(10, _omitEnumNames ? '' : 'CreateView');
  static const FolderEvent GetView = FolderEvent._(11, _omitEnumNames ? '' : 'GetView');
  static const FolderEvent UpdateView = FolderEvent._(12, _omitEnumNames ? '' : 'UpdateView');
  static const FolderEvent DeleteView = FolderEvent._(13, _omitEnumNames ? '' : 'DeleteView');
  static const FolderEvent DuplicateView = FolderEvent._(14, _omitEnumNames ? '' : 'DuplicateView');
  static const FolderEvent CloseView = FolderEvent._(15, _omitEnumNames ? '' : 'CloseView');
  static const FolderEvent CreateOrphanView = FolderEvent._(16, _omitEnumNames ? '' : 'CreateOrphanView');
  static const FolderEvent GetAllViews = FolderEvent._(17, _omitEnumNames ? '' : 'GetAllViews');
  static const FolderEvent CopyLink = FolderEvent._(20, _omitEnumNames ? '' : 'CopyLink');
  static const FolderEvent SetLatestView = FolderEvent._(21, _omitEnumNames ? '' : 'SetLatestView');
  static const FolderEvent MoveView = FolderEvent._(22, _omitEnumNames ? '' : 'MoveView');
  static const FolderEvent ListTrashItems = FolderEvent._(23, _omitEnumNames ? '' : 'ListTrashItems');
  static const FolderEvent RestoreTrashItem = FolderEvent._(24, _omitEnumNames ? '' : 'RestoreTrashItem');
  static const FolderEvent PermanentlyDeleteTrashItem = FolderEvent._(25, _omitEnumNames ? '' : 'PermanentlyDeleteTrashItem');
  static const FolderEvent RecoverAllTrashItems = FolderEvent._(26, _omitEnumNames ? '' : 'RecoverAllTrashItems');
  static const FolderEvent PermanentlyDeleteAllTrashItem = FolderEvent._(27, _omitEnumNames ? '' : 'PermanentlyDeleteAllTrashItem');
  static const FolderEvent ImportData = FolderEvent._(30, _omitEnumNames ? '' : 'ImportData');
  static const FolderEvent RegisterImportProgressStream = FolderEvent._(70, _omitEnumNames ? '' : 'RegisterImportProgressStream');
  static const FolderEvent GetImportProgress = FolderEvent._(71, _omitEnumNames ? '' : 'GetImportProgress');
  static const FolderEvent GetFolderSnapshots = FolderEvent._(31, _omitEnumNames ? '' : 'GetFolderSnapshots');
  static const FolderEvent MoveNestedView = FolderEvent._(32, _omitEnumNames ? '' : 'MoveNestedView');
  static const FolderEvent ReadFavorites = FolderEvent._(33, _omitEnumNames ? '' : 'ReadFavorites');
  static const FolderEvent ToggleFavorite = FolderEvent._(34, _omitEnumNames ? '' : 'ToggleFavorite');
  static const FolderEvent UpdateViewIcon = FolderEvent._(35, _omitEnumNames ? '' : 'UpdateViewIcon');
  static const FolderEvent ReadRecentViews = FolderEvent._(36, _omitEnumNames ? '' : 'ReadRecentViews');
  static const FolderEvent UpdateRecentViews = FolderEvent._(37, _omitEnumNames ? '' : 'UpdateRecentViews');
  static const FolderEvent ReadPrivateViews = FolderEvent._(39, _omitEnumNames ? '' : 'ReadPrivateViews');
  static const FolderEvent ReadCurrentWorkspaceViews = FolderEvent._(40, _omitEnumNames ? '' : 'ReadCurrentWorkspaceViews');
  static const FolderEvent UpdateViewVisibilityStatus = FolderEvent._(41, _omitEnumNames ? '' : 'UpdateViewVisibilityStatus');
  static const FolderEvent GetViewAncestors = FolderEvent._(42, _omitEnumNames ? '' : 'GetViewAncestors');
  static const FolderEvent PublishView = FolderEvent._(43, _omitEnumNames ? '' : 'PublishView');
  static const FolderEvent GetPublishInfo = FolderEvent._(44, _omitEnumNames ? '' : 'GetPublishInfo');
  static const FolderEvent GetPublishNamespace = FolderEvent._(45, _omitEnumNames ? '' : 'GetPublishNamespace');
  static const FolderEvent SetPublishNamespace = FolderEvent._(46, _omitEnumNames ? '' : 'SetPublishNamespace');
  static const FolderEvent UnpublishViews = FolderEvent._(47, _omitEnumNames ? '' : 'UnpublishViews');
  static const FolderEvent ImportZipFile = FolderEvent._(48, _omitEnumNames ? '' : 'ImportZipFile');
  static const FolderEvent ListPublishedViews = FolderEvent._(49, _omitEnumNames ? '' : 'ListPublishedViews');
  static const FolderEvent GetDefaultPublishInfo = FolderEvent._(50, _omitEnumNames ? '' : 'GetDefaultPublishInfo');
  static const FolderEvent SetDefaultPublishView = FolderEvent._(51, _omitEnumNames ? '' : 'SetDefaultPublishView');
  static const FolderEvent SetPublishName = FolderEvent._(52, _omitEnumNames ? '' : 'SetPublishName');
  static const FolderEvent RemoveDefaultPublishView = FolderEvent._(53, _omitEnumNames ? '' : 'RemoveDefaultPublishView');
  static const FolderEvent LockView = FolderEvent._(54, _omitEnumNames ? '' : 'LockView');
  static const FolderEvent UnlockView = FolderEvent._(55, _omitEnumNames ? '' : 'UnlockView');
  static const FolderEvent SharePageWithUser = FolderEvent._(56, _omitEnumNames ? '' : 'SharePageWithUser');
  static const FolderEvent RemoveUserFromSharedPage = FolderEvent._(57, _omitEnumNames ? '' : 'RemoveUserFromSharedPage');
  static const FolderEvent GetSharedUsers = FolderEvent._(58, _omitEnumNames ? '' : 'GetSharedUsers');
  static const FolderEvent GetSharedViews = FolderEvent._(59, _omitEnumNames ? '' : 'GetSharedViews');
  static const FolderEvent GetSharedViewSection = FolderEvent._(60, _omitEnumNames ? '' : 'GetSharedViewSection');

  static const $core.List<FolderEvent> values = <FolderEvent> [
    CreateFolderWorkspace,
    GetCurrentWorkspaceSetting,
    ReadCurrentWorkspace,
    DeleteWorkspace,
    ReadWorkspaceViews,
    CreateView,
    GetView,
    UpdateView,
    DeleteView,
    DuplicateView,
    CloseView,
    CreateOrphanView,
    GetAllViews,
    CopyLink,
    SetLatestView,
    MoveView,
    ListTrashItems,
    RestoreTrashItem,
    PermanentlyDeleteTrashItem,
    RecoverAllTrashItems,
    PermanentlyDeleteAllTrashItem,
    ImportData,
    RegisterImportProgressStream,
    GetImportProgress,
    GetFolderSnapshots,
    MoveNestedView,
    ReadFavorites,
    ToggleFavorite,
    UpdateViewIcon,
    ReadRecentViews,
    UpdateRecentViews,
    ReadPrivateViews,
    ReadCurrentWorkspaceViews,
    UpdateViewVisibilityStatus,
    GetViewAncestors,
    PublishView,
    GetPublishInfo,
    GetPublishNamespace,
    SetPublishNamespace,
    UnpublishViews,
    ImportZipFile,
    ListPublishedViews,
    GetDefaultPublishInfo,
    SetDefaultPublishView,
    SetPublishName,
    RemoveDefaultPublishView,
    LockView,
    UnlockView,
    SharePageWithUser,
    RemoveUserFromSharedPage,
    GetSharedUsers,
    GetSharedViews,
    GetSharedViewSection,
  ];

  static final $core.Map<$core.int, FolderEvent> _byValue = $pb.ProtobufEnum.initByValue(values);
  static FolderEvent? valueOf($core.int value) => _byValue[value];

  const FolderEvent._($core.int v, $core.String n) : super(v, n);
}


const _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
