//
//  Generated code. Do not modify.
//  source: publish.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use publishViewParamsPBDescriptor instead')
const PublishViewParamsPB$json = {
  '1': 'PublishViewParamsPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'publish_name', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'publishName'},
    {'1': 'selected_view_ids', '3': 3, '4': 1, '5': 11, '6': '.RepeatedViewIdPB', '9': 1, '10': 'selectedViewIds'},
  ],
  '8': [
    {'1': 'one_of_publish_name'},
    {'1': 'one_of_selected_view_ids'},
  ],
};

/// Descriptor for `PublishViewParamsPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishViewParamsPBDescriptor = $convert.base64Decode(
    'ChNQdWJsaXNoVmlld1BhcmFtc1BCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBIjCgxwdWJsaX'
    'NoX25hbWUYAiABKAlIAFILcHVibGlzaE5hbWUSPwoRc2VsZWN0ZWRfdmlld19pZHMYAyABKAsy'
    'ES5SZXBlYXRlZFZpZXdJZFBCSAFSD3NlbGVjdGVkVmlld0lkc0IVChNvbmVfb2ZfcHVibGlzaF'
    '9uYW1lQhoKGG9uZV9vZl9zZWxlY3RlZF92aWV3X2lkcw==');

@$core.Deprecated('Use unpublishViewsPayloadPBDescriptor instead')
const UnpublishViewsPayloadPB$json = {
  '1': 'UnpublishViewsPayloadPB',
  '2': [
    {'1': 'view_ids', '3': 1, '4': 3, '5': 9, '10': 'viewIds'},
  ],
};

/// Descriptor for `UnpublishViewsPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unpublishViewsPayloadPBDescriptor = $convert.base64Decode(
    'ChdVbnB1Ymxpc2hWaWV3c1BheWxvYWRQQhIZCgh2aWV3X2lkcxgBIAMoCVIHdmlld0lkcw==');

@$core.Deprecated('Use publishInfoViewPBDescriptor instead')
const PublishInfoViewPB$json = {
  '1': 'PublishInfoViewPB',
  '2': [
    {'1': 'view', '3': 1, '4': 1, '5': 11, '6': '.FolderViewMinimalPB', '10': 'view'},
    {'1': 'info', '3': 2, '4': 1, '5': 11, '6': '.PublishInfoResponsePB', '10': 'info'},
  ],
};

/// Descriptor for `PublishInfoViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishInfoViewPBDescriptor = $convert.base64Decode(
    'ChFQdWJsaXNoSW5mb1ZpZXdQQhIoCgR2aWV3GAEgASgLMhQuRm9sZGVyVmlld01pbmltYWxQQl'
    'IEdmlldxIqCgRpbmZvGAIgASgLMhYuUHVibGlzaEluZm9SZXNwb25zZVBCUgRpbmZv');

@$core.Deprecated('Use folderViewMinimalPBDescriptor instead')
const FolderViewMinimalPB$json = {
  '1': 'FolderViewMinimalPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'icon', '3': 3, '4': 1, '5': 11, '6': '.ViewIconPB', '9': 0, '10': 'icon'},
    {'1': 'layout', '3': 4, '4': 1, '5': 14, '6': '.ViewLayoutPB', '10': 'layout'},
  ],
  '8': [
    {'1': 'one_of_icon'},
  ],
};

/// Descriptor for `FolderViewMinimalPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List folderViewMinimalPBDescriptor = $convert.base64Decode(
    'ChNGb2xkZXJWaWV3TWluaW1hbFBCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBISCgRuYW1lGA'
    'IgASgJUgRuYW1lEiEKBGljb24YAyABKAsyCy5WaWV3SWNvblBCSABSBGljb24SJQoGbGF5b3V0'
    'GAQgASgOMg0uVmlld0xheW91dFBCUgZsYXlvdXRCDQoLb25lX29mX2ljb24=');

@$core.Deprecated('Use publishInfoResponsePBDescriptor instead')
const PublishInfoResponsePB$json = {
  '1': 'PublishInfoResponsePB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'publish_name', '3': 2, '4': 1, '5': 9, '10': 'publishName'},
    {'1': 'namespace', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'namespace'},
    {'1': 'publisher_email', '3': 4, '4': 1, '5': 9, '10': 'publisherEmail'},
    {'1': 'publish_timestamp_sec', '3': 5, '4': 1, '5': 3, '10': 'publishTimestampSec'},
    {'1': 'unpublished_at_timestamp_sec', '3': 6, '4': 1, '5': 3, '9': 1, '10': 'unpublishedAtTimestampSec'},
  ],
  '8': [
    {'1': 'one_of_namespace'},
    {'1': 'one_of_unpublished_at_timestamp_sec'},
  ],
};

/// Descriptor for `PublishInfoResponsePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishInfoResponsePBDescriptor = $convert.base64Decode(
    'ChVQdWJsaXNoSW5mb1Jlc3BvbnNlUEISFwoHdmlld19pZBgBIAEoCVIGdmlld0lkEiEKDHB1Ym'
    'xpc2hfbmFtZRgCIAEoCVILcHVibGlzaE5hbWUSHgoJbmFtZXNwYWNlGAMgASgJSABSCW5hbWVz'
    'cGFjZRInCg9wdWJsaXNoZXJfZW1haWwYBCABKAlSDnB1Ymxpc2hlckVtYWlsEjIKFXB1Ymxpc2'
    'hfdGltZXN0YW1wX3NlYxgFIAEoA1ITcHVibGlzaFRpbWVzdGFtcFNlYxJBChx1bnB1Ymxpc2hl'
    'ZF9hdF90aW1lc3RhbXBfc2VjGAYgASgDSAFSGXVucHVibGlzaGVkQXRUaW1lc3RhbXBTZWNCEg'
    'oQb25lX29mX25hbWVzcGFjZUIlCiNvbmVfb2ZfdW5wdWJsaXNoZWRfYXRfdGltZXN0YW1wX3Nl'
    'Yw==');

@$core.Deprecated('Use repeatedPublishInfoViewPBDescriptor instead')
const RepeatedPublishInfoViewPB$json = {
  '1': 'RepeatedPublishInfoViewPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.PublishInfoViewPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedPublishInfoViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedPublishInfoViewPBDescriptor = $convert.base64Decode(
    'ChlSZXBlYXRlZFB1Ymxpc2hJbmZvVmlld1BCEigKBWl0ZW1zGAEgAygLMhIuUHVibGlzaEluZm'
    '9WaWV3UEJSBWl0ZW1z');

@$core.Deprecated('Use setPublishNamespacePayloadPBDescriptor instead')
const SetPublishNamespacePayloadPB$json = {
  '1': 'SetPublishNamespacePayloadPB',
  '2': [
    {'1': 'new_namespace', '3': 1, '4': 1, '5': 9, '10': 'newNamespace'},
  ],
};

/// Descriptor for `SetPublishNamespacePayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setPublishNamespacePayloadPBDescriptor = $convert.base64Decode(
    'ChxTZXRQdWJsaXNoTmFtZXNwYWNlUGF5bG9hZFBCEiMKDW5ld19uYW1lc3BhY2UYASABKAlSDG'
    '5ld05hbWVzcGFjZQ==');

@$core.Deprecated('Use publishNamespacePBDescriptor instead')
const PublishNamespacePB$json = {
  '1': 'PublishNamespacePB',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
  ],
};

/// Descriptor for `PublishNamespacePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishNamespacePBDescriptor = $convert.base64Decode(
    'ChJQdWJsaXNoTmFtZXNwYWNlUEISHAoJbmFtZXNwYWNlGAEgASgJUgluYW1lc3BhY2U=');

