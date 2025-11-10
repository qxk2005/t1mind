//
//  Generated code. Do not modify.
//  source: workspace.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use workspacePBDescriptor instead')
const WorkspacePB$json = {
  '1': 'WorkspacePB',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'views', '3': 3, '4': 3, '5': 11, '6': '.ViewPB', '10': 'views'},
    {'1': 'create_time', '3': 4, '4': 1, '5': 3, '10': 'createTime'},
  ],
};

/// Descriptor for `WorkspacePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workspacePBDescriptor = $convert.base64Decode(
    'CgtXb3Jrc3BhY2VQQhIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIdCgV2aW'
    'V3cxgDIAMoCzIHLlZpZXdQQlIFdmlld3MSHwoLY3JlYXRlX3RpbWUYBCABKANSCmNyZWF0ZVRp'
    'bWU=');

@$core.Deprecated('Use repeatedWorkspacePBDescriptor instead')
const RepeatedWorkspacePB$json = {
  '1': 'RepeatedWorkspacePB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.WorkspacePB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedWorkspacePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedWorkspacePBDescriptor = $convert.base64Decode(
    'ChNSZXBlYXRlZFdvcmtzcGFjZVBCEiIKBWl0ZW1zGAEgAygLMgwuV29ya3NwYWNlUEJSBWl0ZW'
    '1z');

@$core.Deprecated('Use createWorkspacePayloadPBDescriptor instead')
const CreateWorkspacePayloadPB$json = {
  '1': 'CreateWorkspacePayloadPB',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'desc', '3': 2, '4': 1, '5': 9, '10': 'desc'},
  ],
};

/// Descriptor for `CreateWorkspacePayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createWorkspacePayloadPBDescriptor = $convert.base64Decode(
    'ChhDcmVhdGVXb3Jrc3BhY2VQYXlsb2FkUEISEgoEbmFtZRgBIAEoCVIEbmFtZRISCgRkZXNjGA'
    'IgASgJUgRkZXNj');

@$core.Deprecated('Use workspaceIdPBDescriptor instead')
const WorkspaceIdPB$json = {
  '1': 'WorkspaceIdPB',
  '2': [
    {'1': 'value', '3': 1, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `WorkspaceIdPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workspaceIdPBDescriptor = $convert.base64Decode(
    'Cg1Xb3Jrc3BhY2VJZFBCEhQKBXZhbHVlGAEgASgJUgV2YWx1ZQ==');

@$core.Deprecated('Use getWorkspaceViewPBDescriptor instead')
const GetWorkspaceViewPB$json = {
  '1': 'GetWorkspaceViewPB',
  '2': [
    {'1': 'value', '3': 1, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `GetWorkspaceViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getWorkspaceViewPBDescriptor = $convert.base64Decode(
    'ChJHZXRXb3Jrc3BhY2VWaWV3UEISFAoFdmFsdWUYASABKAlSBXZhbHVl');

@$core.Deprecated('Use workspaceLatestPBDescriptor instead')
const WorkspaceLatestPB$json = {
  '1': 'WorkspaceLatestPB',
  '2': [
    {'1': 'workspace_id', '3': 1, '4': 1, '5': 9, '10': 'workspaceId'},
    {'1': 'latest_view', '3': 2, '4': 1, '5': 11, '6': '.ViewPB', '9': 0, '10': 'latestView'},
  ],
  '8': [
    {'1': 'one_of_latest_view'},
  ],
};

/// Descriptor for `WorkspaceLatestPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workspaceLatestPBDescriptor = $convert.base64Decode(
    'ChFXb3Jrc3BhY2VMYXRlc3RQQhIhCgx3b3Jrc3BhY2VfaWQYASABKAlSC3dvcmtzcGFjZUlkEi'
    'oKC2xhdGVzdF92aWV3GAIgASgLMgcuVmlld1BCSABSCmxhdGVzdFZpZXdCFAoSb25lX29mX2xh'
    'dGVzdF92aWV3');

@$core.Deprecated('Use updateWorkspacePayloadPBDescriptor instead')
const UpdateWorkspacePayloadPB$json = {
  '1': 'UpdateWorkspacePayloadPB',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'name'},
    {'1': 'desc', '3': 3, '4': 1, '5': 9, '9': 1, '10': 'desc'},
  ],
  '8': [
    {'1': 'one_of_name'},
    {'1': 'one_of_desc'},
  ],
};

/// Descriptor for `UpdateWorkspacePayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateWorkspacePayloadPBDescriptor = $convert.base64Decode(
    'ChhVcGRhdGVXb3Jrc3BhY2VQYXlsb2FkUEISDgoCaWQYASABKAlSAmlkEhQKBG5hbWUYAiABKA'
    'lIAFIEbmFtZRIUCgRkZXNjGAMgASgJSAFSBGRlc2NCDQoLb25lX29mX25hbWVCDQoLb25lX29m'
    'X2Rlc2M=');

@$core.Deprecated('Use repeatedFolderSnapshotPBDescriptor instead')
const RepeatedFolderSnapshotPB$json = {
  '1': 'RepeatedFolderSnapshotPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.FolderSnapshotPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedFolderSnapshotPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedFolderSnapshotPBDescriptor = $convert.base64Decode(
    'ChhSZXBlYXRlZEZvbGRlclNuYXBzaG90UEISJwoFaXRlbXMYASADKAsyES5Gb2xkZXJTbmFwc2'
    'hvdFBCUgVpdGVtcw==');

@$core.Deprecated('Use folderSnapshotPBDescriptor instead')
const FolderSnapshotPB$json = {
  '1': 'FolderSnapshotPB',
  '2': [
    {'1': 'snapshot_id', '3': 1, '4': 1, '5': 3, '10': 'snapshotId'},
    {'1': 'snapshot_desc', '3': 2, '4': 1, '5': 9, '10': 'snapshotDesc'},
    {'1': 'created_at', '3': 3, '4': 1, '5': 3, '10': 'createdAt'},
    {'1': 'data', '3': 4, '4': 1, '5': 12, '10': 'data'},
  ],
};

/// Descriptor for `FolderSnapshotPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List folderSnapshotPBDescriptor = $convert.base64Decode(
    'ChBGb2xkZXJTbmFwc2hvdFBCEh8KC3NuYXBzaG90X2lkGAEgASgDUgpzbmFwc2hvdElkEiMKDX'
    'NuYXBzaG90X2Rlc2MYAiABKAlSDHNuYXBzaG90RGVzYxIdCgpjcmVhdGVkX2F0GAMgASgDUglj'
    'cmVhdGVkQXQSEgoEZGF0YRgEIAEoDFIEZGF0YQ==');

@$core.Deprecated('Use folderSnapshotStatePBDescriptor instead')
const FolderSnapshotStatePB$json = {
  '1': 'FolderSnapshotStatePB',
  '2': [
    {'1': 'new_snapshot_id', '3': 1, '4': 1, '5': 3, '10': 'newSnapshotId'},
  ],
};

/// Descriptor for `FolderSnapshotStatePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List folderSnapshotStatePBDescriptor = $convert.base64Decode(
    'ChVGb2xkZXJTbmFwc2hvdFN0YXRlUEISJgoPbmV3X3NuYXBzaG90X2lkGAEgASgDUg1uZXdTbm'
    'Fwc2hvdElk');

@$core.Deprecated('Use folderSyncStatePBDescriptor instead')
const FolderSyncStatePB$json = {
  '1': 'FolderSyncStatePB',
  '2': [
    {'1': 'is_syncing', '3': 1, '4': 1, '5': 8, '10': 'isSyncing'},
    {'1': 'is_finish', '3': 2, '4': 1, '5': 8, '10': 'isFinish'},
  ],
};

/// Descriptor for `FolderSyncStatePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List folderSyncStatePBDescriptor = $convert.base64Decode(
    'ChFGb2xkZXJTeW5jU3RhdGVQQhIdCgppc19zeW5jaW5nGAEgASgIUglpc1N5bmNpbmcSGwoJaX'
    'NfZmluaXNoGAIgASgIUghpc0ZpbmlzaA==');

@$core.Deprecated('Use userFolderPBDescriptor instead')
const UserFolderPB$json = {
  '1': 'UserFolderPB',
  '2': [
    {'1': 'uid', '3': 1, '4': 1, '5': 3, '10': 'uid'},
    {'1': 'workspace_id', '3': 2, '4': 1, '5': 9, '10': 'workspaceId'},
  ],
};

/// Descriptor for `UserFolderPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userFolderPBDescriptor = $convert.base64Decode(
    'CgxVc2VyRm9sZGVyUEISEAoDdWlkGAEgASgDUgN1aWQSIQoMd29ya3NwYWNlX2lkGAIgASgJUg'
    't3b3Jrc3BhY2VJZA==');

