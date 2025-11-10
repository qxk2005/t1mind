//
//  Generated code. Do not modify.
//  source: icon.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use viewIconTypePBDescriptor instead')
const ViewIconTypePB$json = {
  '1': 'ViewIconTypePB',
  '2': [
    {'1': 'Emoji', '2': 0},
    {'1': 'Url', '2': 1},
    {'1': 'Icon', '2': 2},
  ],
};

/// Descriptor for `ViewIconTypePB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List viewIconTypePBDescriptor = $convert.base64Decode(
    'Cg5WaWV3SWNvblR5cGVQQhIJCgVFbW9qaRAAEgcKA1VybBABEggKBEljb24QAg==');

@$core.Deprecated('Use viewIconPBDescriptor instead')
const ViewIconPB$json = {
  '1': 'ViewIconPB',
  '2': [
    {'1': 'ty', '3': 1, '4': 1, '5': 14, '6': '.ViewIconTypePB', '10': 'ty'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `ViewIconPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List viewIconPBDescriptor = $convert.base64Decode(
    'CgpWaWV3SWNvblBCEh8KAnR5GAEgASgOMg8uVmlld0ljb25UeXBlUEJSAnR5EhQKBXZhbHVlGA'
    'IgASgJUgV2YWx1ZQ==');

@$core.Deprecated('Use updateViewIconPayloadPBDescriptor instead')
const UpdateViewIconPayloadPB$json = {
  '1': 'UpdateViewIconPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'icon', '3': 2, '4': 1, '5': 11, '6': '.ViewIconPB', '9': 0, '10': 'icon'},
  ],
  '8': [
    {'1': 'one_of_icon'},
  ],
};

/// Descriptor for `UpdateViewIconPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateViewIconPayloadPBDescriptor = $convert.base64Decode(
    'ChdVcGRhdGVWaWV3SWNvblBheWxvYWRQQhIXCgd2aWV3X2lkGAEgASgJUgZ2aWV3SWQSIQoEaW'
    'NvbhgCIAEoCzILLlZpZXdJY29uUEJIAFIEaWNvbkINCgtvbmVfb2ZfaWNvbg==');

