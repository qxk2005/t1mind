//
//  Generated code. Do not modify.
//  source: notification.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use folderNotificationDescriptor instead')
const FolderNotification$json = {
  '1': 'FolderNotification',
  '2': [
    {'1': 'Unknown', '2': 0},
    {'1': 'DidCreateWorkspace', '2': 1},
    {'1': 'DidUpdateWorkspace', '2': 2},
    {'1': 'DidUpdateWorkspaceViews', '2': 3},
    {'1': 'DidUpdateWorkspaceSetting', '2': 4},
    {'1': 'DidUpdateView', '2': 10},
    {'1': 'DidUpdateChildViews', '2': 11},
    {'1': 'DidDeleteView', '2': 12},
    {'1': 'DidRestoreView', '2': 13},
    {'1': 'DidMoveViewToTrash', '2': 14},
    {'1': 'DidUpdateTrash', '2': 15},
    {'1': 'DidUpdateFolderSnapshotState', '2': 16},
    {'1': 'DidUpdateFolderSyncUpdate', '2': 17},
    {'1': 'DidFavoriteView', '2': 36},
    {'1': 'DidUnfavoriteView', '2': 37},
    {'1': 'DidUpdateRecentViews', '2': 38},
    {'1': 'DidUpdateSectionViews', '2': 39},
    {'1': 'DidUpdateSharedViews', '2': 40},
    {'1': 'DidUpdateSharedUsers', '2': 41},
  ],
};

/// Descriptor for `FolderNotification`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List folderNotificationDescriptor = $convert.base64Decode(
    'ChJGb2xkZXJOb3RpZmljYXRpb24SCwoHVW5rbm93bhAAEhYKEkRpZENyZWF0ZVdvcmtzcGFjZR'
    'ABEhYKEkRpZFVwZGF0ZVdvcmtzcGFjZRACEhsKF0RpZFVwZGF0ZVdvcmtzcGFjZVZpZXdzEAMS'
    'HQoZRGlkVXBkYXRlV29ya3NwYWNlU2V0dGluZxAEEhEKDURpZFVwZGF0ZVZpZXcQChIXChNEaW'
    'RVcGRhdGVDaGlsZFZpZXdzEAsSEQoNRGlkRGVsZXRlVmlldxAMEhIKDkRpZFJlc3RvcmVWaWV3'
    'EA0SFgoSRGlkTW92ZVZpZXdUb1RyYXNoEA4SEgoORGlkVXBkYXRlVHJhc2gQDxIgChxEaWRVcG'
    'RhdGVGb2xkZXJTbmFwc2hvdFN0YXRlEBASHQoZRGlkVXBkYXRlRm9sZGVyU3luY1VwZGF0ZRAR'
    'EhMKD0RpZEZhdm9yaXRlVmlldxAkEhUKEURpZFVuZmF2b3JpdGVWaWV3ECUSGAoURGlkVXBkYX'
    'RlUmVjZW50Vmlld3MQJhIZChVEaWRVcGRhdGVTZWN0aW9uVmlld3MQJxIYChREaWRVcGRhdGVT'
    'aGFyZWRWaWV3cxAoEhgKFERpZFVwZGF0ZVNoYXJlZFVzZXJzECk=');

