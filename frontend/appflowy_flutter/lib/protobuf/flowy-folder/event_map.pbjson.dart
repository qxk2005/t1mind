//
//  Generated code. Do not modify.
//  source: event_map.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use folderEventDescriptor instead')
const FolderEvent$json = {
  '1': 'FolderEvent',
  '2': [
    {'1': 'CreateFolderWorkspace', '2': 0},
    {'1': 'GetCurrentWorkspaceSetting', '2': 1},
    {'1': 'ReadCurrentWorkspace', '2': 2},
    {'1': 'DeleteWorkspace', '2': 3},
    {'1': 'ReadWorkspaceViews', '2': 5},
    {'1': 'CreateView', '2': 10},
    {'1': 'GetView', '2': 11},
    {'1': 'UpdateView', '2': 12},
    {'1': 'DeleteView', '2': 13},
    {'1': 'DuplicateView', '2': 14},
    {'1': 'CloseView', '2': 15},
    {'1': 'CreateOrphanView', '2': 16},
    {'1': 'GetAllViews', '2': 17},
    {'1': 'CopyLink', '2': 20},
    {'1': 'SetLatestView', '2': 21},
    {'1': 'MoveView', '2': 22},
    {'1': 'ListTrashItems', '2': 23},
    {'1': 'RestoreTrashItem', '2': 24},
    {'1': 'PermanentlyDeleteTrashItem', '2': 25},
    {'1': 'RecoverAllTrashItems', '2': 26},
    {'1': 'PermanentlyDeleteAllTrashItem', '2': 27},
    {'1': 'ImportData', '2': 30},
    {'1': 'RegisterImportProgressStream', '2': 70},
    {'1': 'GetImportProgress', '2': 71},
    {'1': 'GetFolderSnapshots', '2': 31},
    {'1': 'MoveNestedView', '2': 32},
    {'1': 'ReadFavorites', '2': 33},
    {'1': 'ToggleFavorite', '2': 34},
    {'1': 'UpdateViewIcon', '2': 35},
    {'1': 'ReadRecentViews', '2': 36},
    {'1': 'UpdateRecentViews', '2': 37},
    {'1': 'ReadPrivateViews', '2': 39},
    {'1': 'ReadCurrentWorkspaceViews', '2': 40},
    {'1': 'UpdateViewVisibilityStatus', '2': 41},
    {'1': 'GetViewAncestors', '2': 42},
    {'1': 'PublishView', '2': 43},
    {'1': 'GetPublishInfo', '2': 44},
    {'1': 'GetPublishNamespace', '2': 45},
    {'1': 'SetPublishNamespace', '2': 46},
    {'1': 'UnpublishViews', '2': 47},
    {'1': 'ImportZipFile', '2': 48},
    {'1': 'ListPublishedViews', '2': 49},
    {'1': 'GetDefaultPublishInfo', '2': 50},
    {'1': 'SetDefaultPublishView', '2': 51},
    {'1': 'SetPublishName', '2': 52},
    {'1': 'RemoveDefaultPublishView', '2': 53},
    {'1': 'LockView', '2': 54},
    {'1': 'UnlockView', '2': 55},
    {'1': 'SharePageWithUser', '2': 56},
    {'1': 'RemoveUserFromSharedPage', '2': 57},
    {'1': 'GetSharedUsers', '2': 58},
    {'1': 'GetSharedViews', '2': 59},
    {'1': 'GetSharedViewSection', '2': 60},
  ],
};

/// Descriptor for `FolderEvent`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List folderEventDescriptor = $convert.base64Decode(
    'CgtGb2xkZXJFdmVudBIZChVDcmVhdGVGb2xkZXJXb3Jrc3BhY2UQABIeChpHZXRDdXJyZW50V2'
    '9ya3NwYWNlU2V0dGluZxABEhgKFFJlYWRDdXJyZW50V29ya3NwYWNlEAISEwoPRGVsZXRlV29y'
    'a3NwYWNlEAMSFgoSUmVhZFdvcmtzcGFjZVZpZXdzEAUSDgoKQ3JlYXRlVmlldxAKEgsKB0dldF'
    'ZpZXcQCxIOCgpVcGRhdGVWaWV3EAwSDgoKRGVsZXRlVmlldxANEhEKDUR1cGxpY2F0ZVZpZXcQ'
    'DhINCglDbG9zZVZpZXcQDxIUChBDcmVhdGVPcnBoYW5WaWV3EBASDwoLR2V0QWxsVmlld3MQER'
    'IMCghDb3B5TGluaxAUEhEKDVNldExhdGVzdFZpZXcQFRIMCghNb3ZlVmlldxAWEhIKDkxpc3RU'
    'cmFzaEl0ZW1zEBcSFAoQUmVzdG9yZVRyYXNoSXRlbRAYEh4KGlBlcm1hbmVudGx5RGVsZXRlVH'
    'Jhc2hJdGVtEBkSGAoUUmVjb3ZlckFsbFRyYXNoSXRlbXMQGhIhCh1QZXJtYW5lbnRseURlbGV0'
    'ZUFsbFRyYXNoSXRlbRAbEg4KCkltcG9ydERhdGEQHhIgChxSZWdpc3RlckltcG9ydFByb2dyZX'
    'NzU3RyZWFtEEYSFQoRR2V0SW1wb3J0UHJvZ3Jlc3MQRxIWChJHZXRGb2xkZXJTbmFwc2hvdHMQ'
    'HxISCg5Nb3ZlTmVzdGVkVmlldxAgEhEKDVJlYWRGYXZvcml0ZXMQIRISCg5Ub2dnbGVGYXZvcm'
    'l0ZRAiEhIKDlVwZGF0ZVZpZXdJY29uECMSEwoPUmVhZFJlY2VudFZpZXdzECQSFQoRVXBkYXRl'
    'UmVjZW50Vmlld3MQJRIUChBSZWFkUHJpdmF0ZVZpZXdzECcSHQoZUmVhZEN1cnJlbnRXb3Jrc3'
    'BhY2VWaWV3cxAoEh4KGlVwZGF0ZVZpZXdWaXNpYmlsaXR5U3RhdHVzECkSFAoQR2V0Vmlld0Fu'
    'Y2VzdG9ycxAqEg8KC1B1Ymxpc2hWaWV3ECsSEgoOR2V0UHVibGlzaEluZm8QLBIXChNHZXRQdW'
    'JsaXNoTmFtZXNwYWNlEC0SFwoTU2V0UHVibGlzaE5hbWVzcGFjZRAuEhIKDlVucHVibGlzaFZp'
    'ZXdzEC8SEQoNSW1wb3J0WmlwRmlsZRAwEhYKEkxpc3RQdWJsaXNoZWRWaWV3cxAxEhkKFUdldE'
    'RlZmF1bHRQdWJsaXNoSW5mbxAyEhkKFVNldERlZmF1bHRQdWJsaXNoVmlldxAzEhIKDlNldFB1'
    'Ymxpc2hOYW1lEDQSHAoYUmVtb3ZlRGVmYXVsdFB1Ymxpc2hWaWV3EDUSDAoITG9ja1ZpZXcQNh'
    'IOCgpVbmxvY2tWaWV3EDcSFQoRU2hhcmVQYWdlV2l0aFVzZXIQOBIcChhSZW1vdmVVc2VyRnJv'
    'bVNoYXJlZFBhZ2UQORISCg5HZXRTaGFyZWRVc2VycxA6EhIKDkdldFNoYXJlZFZpZXdzEDsSGA'
    'oUR2V0U2hhcmVkVmlld1NlY3Rpb24QPA==');

