//
//  Generated code. Do not modify.
//  source: trash.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use trashPBDescriptor instead')
const TrashPB$json = {
  '1': 'TrashPB',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'modified_time', '3': 3, '4': 1, '5': 3, '10': 'modifiedTime'},
    {'1': 'create_time', '3': 4, '4': 1, '5': 3, '10': 'createTime'},
  ],
};

/// Descriptor for `TrashPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List trashPBDescriptor = $convert.base64Decode(
    'CgdUcmFzaFBCEg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiMKDW1vZGlmaW'
    'VkX3RpbWUYAyABKANSDG1vZGlmaWVkVGltZRIfCgtjcmVhdGVfdGltZRgEIAEoA1IKY3JlYXRl'
    'VGltZQ==');

@$core.Deprecated('Use repeatedTrashPBDescriptor instead')
const RepeatedTrashPB$json = {
  '1': 'RepeatedTrashPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.TrashPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedTrashPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedTrashPBDescriptor = $convert.base64Decode(
    'Cg9SZXBlYXRlZFRyYXNoUEISHgoFaXRlbXMYASADKAsyCC5UcmFzaFBCUgVpdGVtcw==');

@$core.Deprecated('Use trashIdPBDescriptor instead')
const TrashIdPB$json = {
  '1': 'TrashIdPB',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `TrashIdPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List trashIdPBDescriptor = $convert.base64Decode(
    'CglUcmFzaElkUEISDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use repeatedTrashIdPBDescriptor instead')
const RepeatedTrashIdPB$json = {
  '1': 'RepeatedTrashIdPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.TrashIdPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedTrashIdPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedTrashIdPBDescriptor = $convert.base64Decode(
    'ChFSZXBlYXRlZFRyYXNoSWRQQhIgCgVpdGVtcxgBIAMoCzIKLlRyYXNoSWRQQlIFaXRlbXM=');

