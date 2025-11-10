//
//  Generated code. Do not modify.
//  source: view.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use viewLayoutPBDescriptor instead')
const ViewLayoutPB$json = {
  '1': 'ViewLayoutPB',
  '2': [
    {'1': 'Document', '2': 0},
    {'1': 'Grid', '2': 1},
    {'1': 'Board', '2': 2},
    {'1': 'Calendar', '2': 3},
    {'1': 'Chat', '2': 4},
  ],
};

/// Descriptor for `ViewLayoutPB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List viewLayoutPBDescriptor = $convert.base64Decode(
    'CgxWaWV3TGF5b3V0UEISDAoIRG9jdW1lbnQQABIICgRHcmlkEAESCQoFQm9hcmQQAhIMCghDYW'
    'xlbmRhchADEggKBENoYXQQBA==');

@$core.Deprecated('Use viewSectionPBDescriptor instead')
const ViewSectionPB$json = {
  '1': 'ViewSectionPB',
  '2': [
    {'1': 'Private', '2': 0},
    {'1': 'Public', '2': 1},
  ],
};

/// Descriptor for `ViewSectionPB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List viewSectionPBDescriptor = $convert.base64Decode(
    'Cg1WaWV3U2VjdGlvblBCEgsKB1ByaXZhdGUQABIKCgZQdWJsaWMQAQ==');

@$core.Deprecated('Use aFAccessLevelPBDescriptor instead')
const AFAccessLevelPB$json = {
  '1': 'AFAccessLevelPB',
  '2': [
    {'1': 'ReadOnly', '2': 0},
    {'1': 'ReadAndComment', '2': 1},
    {'1': 'ReadAndWrite', '2': 2},
    {'1': 'FullAccess', '2': 3},
  ],
};

/// Descriptor for `AFAccessLevelPB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List aFAccessLevelPBDescriptor = $convert.base64Decode(
    'Cg9BRkFjY2Vzc0xldmVsUEISDAoIUmVhZE9ubHkQABISCg5SZWFkQW5kQ29tbWVudBABEhAKDF'
    'JlYWRBbmRXcml0ZRACEg4KCkZ1bGxBY2Nlc3MQAw==');

@$core.Deprecated('Use aFRolePBDescriptor instead')
const AFRolePB$json = {
  '1': 'AFRolePB',
  '2': [
    {'1': 'Owner', '2': 0},
    {'1': 'Member', '2': 1},
    {'1': 'Guest', '2': 2},
  ],
};

/// Descriptor for `AFRolePB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List aFRolePBDescriptor = $convert.base64Decode(
    'CghBRlJvbGVQQhIJCgVPd25lchAAEgoKBk1lbWJlchABEgkKBUd1ZXN0EAI=');

@$core.Deprecated('Use sharedViewSectionPBDescriptor instead')
const SharedViewSectionPB$json = {
  '1': 'SharedViewSectionPB',
  '2': [
    {'1': 'PrivateSection', '2': 0},
    {'1': 'PublicSection', '2': 1},
    {'1': 'SharedSection', '2': 2},
  ],
};

/// Descriptor for `SharedViewSectionPB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List sharedViewSectionPBDescriptor = $convert.base64Decode(
    'ChNTaGFyZWRWaWV3U2VjdGlvblBCEhIKDlByaXZhdGVTZWN0aW9uEAASEQoNUHVibGljU2VjdG'
    'lvbhABEhEKDVNoYXJlZFNlY3Rpb24QAg==');

@$core.Deprecated('Use childViewUpdatePBDescriptor instead')
const ChildViewUpdatePB$json = {
  '1': 'ChildViewUpdatePB',
  '2': [
    {'1': 'parent_view_id', '3': 1, '4': 1, '5': 9, '10': 'parentViewId'},
    {'1': 'create_child_views', '3': 2, '4': 3, '5': 11, '6': '.ViewPB', '10': 'createChildViews'},
    {'1': 'delete_child_views', '3': 3, '4': 3, '5': 9, '10': 'deleteChildViews'},
    {'1': 'update_child_views', '3': 4, '4': 3, '5': 11, '6': '.ViewPB', '10': 'updateChildViews'},
  ],
};

/// Descriptor for `ChildViewUpdatePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List childViewUpdatePBDescriptor = $convert.base64Decode(
    'ChFDaGlsZFZpZXdVcGRhdGVQQhIkCg5wYXJlbnRfdmlld19pZBgBIAEoCVIMcGFyZW50Vmlld0'
    'lkEjUKEmNyZWF0ZV9jaGlsZF92aWV3cxgCIAMoCzIHLlZpZXdQQlIQY3JlYXRlQ2hpbGRWaWV3'
    'cxIsChJkZWxldGVfY2hpbGRfdmlld3MYAyADKAlSEGRlbGV0ZUNoaWxkVmlld3MSNQoSdXBkYX'
    'RlX2NoaWxkX3ZpZXdzGAQgAygLMgcuVmlld1BCUhB1cGRhdGVDaGlsZFZpZXdz');

@$core.Deprecated('Use viewPBDescriptor instead')
const ViewPB$json = {
  '1': 'ViewPB',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'parent_view_id', '3': 2, '4': 1, '5': 9, '10': 'parentViewId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {'1': 'create_time', '3': 4, '4': 1, '5': 3, '10': 'createTime'},
    {'1': 'child_views', '3': 5, '4': 3, '5': 11, '6': '.ViewPB', '10': 'childViews'},
    {'1': 'layout', '3': 6, '4': 1, '5': 14, '6': '.ViewLayoutPB', '10': 'layout'},
    {'1': 'icon', '3': 7, '4': 1, '5': 11, '6': '.ViewIconPB', '9': 0, '10': 'icon'},
    {'1': 'is_favorite', '3': 8, '4': 1, '5': 8, '10': 'isFavorite'},
    {'1': 'extra', '3': 9, '4': 1, '5': 9, '9': 1, '10': 'extra'},
    {'1': 'created_by', '3': 10, '4': 1, '5': 3, '9': 2, '10': 'createdBy'},
    {'1': 'last_edited', '3': 11, '4': 1, '5': 3, '10': 'lastEdited'},
    {'1': 'last_edited_by', '3': 12, '4': 1, '5': 3, '9': 3, '10': 'lastEditedBy'},
    {'1': 'is_locked', '3': 13, '4': 1, '5': 8, '9': 4, '10': 'isLocked'},
  ],
  '8': [
    {'1': 'one_of_icon'},
    {'1': 'one_of_extra'},
    {'1': 'one_of_created_by'},
    {'1': 'one_of_last_edited_by'},
    {'1': 'one_of_is_locked'},
  ],
};

/// Descriptor for `ViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List viewPBDescriptor = $convert.base64Decode(
    'CgZWaWV3UEISDgoCaWQYASABKAlSAmlkEiQKDnBhcmVudF92aWV3X2lkGAIgASgJUgxwYXJlbn'
    'RWaWV3SWQSEgoEbmFtZRgDIAEoCVIEbmFtZRIfCgtjcmVhdGVfdGltZRgEIAEoA1IKY3JlYXRl'
    'VGltZRIoCgtjaGlsZF92aWV3cxgFIAMoCzIHLlZpZXdQQlIKY2hpbGRWaWV3cxIlCgZsYXlvdX'
    'QYBiABKA4yDS5WaWV3TGF5b3V0UEJSBmxheW91dBIhCgRpY29uGAcgASgLMgsuVmlld0ljb25Q'
    'QkgAUgRpY29uEh8KC2lzX2Zhdm9yaXRlGAggASgIUgppc0Zhdm9yaXRlEhYKBWV4dHJhGAkgAS'
    'gJSAFSBWV4dHJhEh8KCmNyZWF0ZWRfYnkYCiABKANIAlIJY3JlYXRlZEJ5Eh8KC2xhc3RfZWRp'
    'dGVkGAsgASgDUgpsYXN0RWRpdGVkEiYKDmxhc3RfZWRpdGVkX2J5GAwgASgDSANSDGxhc3RFZG'
    'l0ZWRCeRIdCglpc19sb2NrZWQYDSABKAhIBFIIaXNMb2NrZWRCDQoLb25lX29mX2ljb25CDgoM'
    'b25lX29mX2V4dHJhQhMKEW9uZV9vZl9jcmVhdGVkX2J5QhcKFW9uZV9vZl9sYXN0X2VkaXRlZF'
    '9ieUISChBvbmVfb2ZfaXNfbG9ja2Vk');

@$core.Deprecated('Use sectionViewsPBDescriptor instead')
const SectionViewsPB$json = {
  '1': 'SectionViewsPB',
  '2': [
    {'1': 'section', '3': 1, '4': 1, '5': 14, '6': '.ViewSectionPB', '10': 'section'},
    {'1': 'views', '3': 2, '4': 3, '5': 11, '6': '.ViewPB', '10': 'views'},
  ],
};

/// Descriptor for `SectionViewsPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sectionViewsPBDescriptor = $convert.base64Decode(
    'Cg5TZWN0aW9uVmlld3NQQhIoCgdzZWN0aW9uGAEgASgOMg4uVmlld1NlY3Rpb25QQlIHc2VjdG'
    'lvbhIdCgV2aWV3cxgCIAMoCzIHLlZpZXdQQlIFdmlld3M=');

@$core.Deprecated('Use repeatedViewPBDescriptor instead')
const RepeatedViewPB$json = {
  '1': 'RepeatedViewPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.ViewPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedViewPBDescriptor = $convert.base64Decode(
    'Cg5SZXBlYXRlZFZpZXdQQhIdCgVpdGVtcxgBIAMoCzIHLlZpZXdQQlIFaXRlbXM=');

@$core.Deprecated('Use repeatedFavoriteViewPBDescriptor instead')
const RepeatedFavoriteViewPB$json = {
  '1': 'RepeatedFavoriteViewPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.SectionViewPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedFavoriteViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedFavoriteViewPBDescriptor = $convert.base64Decode(
    'ChZSZXBlYXRlZEZhdm9yaXRlVmlld1BCEiQKBWl0ZW1zGAEgAygLMg4uU2VjdGlvblZpZXdQQl'
    'IFaXRlbXM=');

@$core.Deprecated('Use readRecentViewsPBDescriptor instead')
const ReadRecentViewsPB$json = {
  '1': 'ReadRecentViewsPB',
  '2': [
    {'1': 'start', '3': 1, '4': 1, '5': 4, '10': 'start'},
    {'1': 'limit', '3': 2, '4': 1, '5': 4, '10': 'limit'},
  ],
};

/// Descriptor for `ReadRecentViewsPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List readRecentViewsPBDescriptor = $convert.base64Decode(
    'ChFSZWFkUmVjZW50Vmlld3NQQhIUCgVzdGFydBgBIAEoBFIFc3RhcnQSFAoFbGltaXQYAiABKA'
    'RSBWxpbWl0');

@$core.Deprecated('Use repeatedRecentViewPBDescriptor instead')
const RepeatedRecentViewPB$json = {
  '1': 'RepeatedRecentViewPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.SectionViewPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedRecentViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedRecentViewPBDescriptor = $convert.base64Decode(
    'ChRSZXBlYXRlZFJlY2VudFZpZXdQQhIkCgVpdGVtcxgBIAMoCzIOLlNlY3Rpb25WaWV3UEJSBW'
    'l0ZW1z');

@$core.Deprecated('Use sectionViewPBDescriptor instead')
const SectionViewPB$json = {
  '1': 'SectionViewPB',
  '2': [
    {'1': 'item', '3': 1, '4': 1, '5': 11, '6': '.ViewPB', '10': 'item'},
    {'1': 'timestamp', '3': 2, '4': 1, '5': 3, '10': 'timestamp'},
  ],
};

/// Descriptor for `SectionViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sectionViewPBDescriptor = $convert.base64Decode(
    'Cg1TZWN0aW9uVmlld1BCEhsKBGl0ZW0YASABKAsyBy5WaWV3UEJSBGl0ZW0SHAoJdGltZXN0YW'
    '1wGAIgASgDUgl0aW1lc3RhbXA=');

@$core.Deprecated('Use repeatedViewIdPBDescriptor instead')
const RepeatedViewIdPB$json = {
  '1': 'RepeatedViewIdPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 9, '10': 'items'},
  ],
};

/// Descriptor for `RepeatedViewIdPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedViewIdPBDescriptor = $convert.base64Decode(
    'ChBSZXBlYXRlZFZpZXdJZFBCEhQKBWl0ZW1zGAEgAygJUgVpdGVtcw==');

@$core.Deprecated('Use createViewPayloadPBDescriptor instead')
const CreateViewPayloadPB$json = {
  '1': 'CreateViewPayloadPB',
  '2': [
    {'1': 'parent_view_id', '3': 1, '4': 1, '5': 9, '10': 'parentViewId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'thumbnail', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'thumbnail'},
    {'1': 'layout', '3': 4, '4': 1, '5': 14, '6': '.ViewLayoutPB', '10': 'layout'},
    {'1': 'initial_data', '3': 5, '4': 1, '5': 12, '10': 'initialData'},
    {'1': 'meta', '3': 6, '4': 3, '5': 11, '6': '.CreateViewPayloadPB.MetaEntry', '10': 'meta'},
    {'1': 'set_as_current', '3': 7, '4': 1, '5': 8, '10': 'setAsCurrent'},
    {'1': 'index', '3': 8, '4': 1, '5': 13, '9': 1, '10': 'index'},
    {'1': 'section', '3': 9, '4': 1, '5': 14, '6': '.ViewSectionPB', '9': 2, '10': 'section'},
    {'1': 'view_id', '3': 10, '4': 1, '5': 9, '9': 3, '10': 'viewId'},
    {'1': 'extra', '3': 11, '4': 1, '5': 9, '9': 4, '10': 'extra'},
  ],
  '3': [CreateViewPayloadPB_MetaEntry$json],
  '8': [
    {'1': 'one_of_thumbnail'},
    {'1': 'one_of_index'},
    {'1': 'one_of_section'},
    {'1': 'one_of_view_id'},
    {'1': 'one_of_extra'},
  ],
};

@$core.Deprecated('Use createViewPayloadPBDescriptor instead')
const CreateViewPayloadPB_MetaEntry$json = {
  '1': 'MetaEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `CreateViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createViewPayloadPBDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVWaWV3UGF5bG9hZFBCEiQKDnBhcmVudF92aWV3X2lkGAEgASgJUgxwYXJlbnRWaW'
    'V3SWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIeCgl0aHVtYm5haWwYAyABKAlIAFIJdGh1bWJuYWls'
    'EiUKBmxheW91dBgEIAEoDjINLlZpZXdMYXlvdXRQQlIGbGF5b3V0EiEKDGluaXRpYWxfZGF0YR'
    'gFIAEoDFILaW5pdGlhbERhdGESMgoEbWV0YRgGIAMoCzIeLkNyZWF0ZVZpZXdQYXlsb2FkUEIu'
    'TWV0YUVudHJ5UgRtZXRhEiQKDnNldF9hc19jdXJyZW50GAcgASgIUgxzZXRBc0N1cnJlbnQSFg'
    'oFaW5kZXgYCCABKA1IAVIFaW5kZXgSKgoHc2VjdGlvbhgJIAEoDjIOLlZpZXdTZWN0aW9uUEJI'
    'AlIHc2VjdGlvbhIZCgd2aWV3X2lkGAogASgJSANSBnZpZXdJZBIWCgVleHRyYRgLIAEoCUgEUg'
    'VleHRyYRo3CglNZXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZh'
    'bHVlOgI4AUISChBvbmVfb2ZfdGh1bWJuYWlsQg4KDG9uZV9vZl9pbmRleEIQCg5vbmVfb2Zfc2'
    'VjdGlvbkIQCg5vbmVfb2Zfdmlld19pZEIOCgxvbmVfb2ZfZXh0cmE=');

@$core.Deprecated('Use createOrphanViewPayloadPBDescriptor instead')
const CreateOrphanViewPayloadPB$json = {
  '1': 'CreateOrphanViewPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'layout', '3': 3, '4': 1, '5': 14, '6': '.ViewLayoutPB', '10': 'layout'},
    {'1': 'initial_data', '3': 4, '4': 1, '5': 12, '10': 'initialData'},
  ],
};

/// Descriptor for `CreateOrphanViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createOrphanViewPayloadPBDescriptor = $convert.base64Decode(
    'ChlDcmVhdGVPcnBoYW5WaWV3UGF5bG9hZFBCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBISCg'
    'RuYW1lGAIgASgJUgRuYW1lEiUKBmxheW91dBgDIAEoDjINLlZpZXdMYXlvdXRQQlIGbGF5b3V0'
    'EiEKDGluaXRpYWxfZGF0YRgEIAEoDFILaW5pdGlhbERhdGE=');

@$core.Deprecated('Use viewIdPBDescriptor instead')
const ViewIdPB$json = {
  '1': 'ViewIdPB',
  '2': [
    {'1': 'value', '3': 1, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `ViewIdPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List viewIdPBDescriptor = $convert.base64Decode(
    'CghWaWV3SWRQQhIUCgV2YWx1ZRgBIAEoCVIFdmFsdWU=');

@$core.Deprecated('Use setPublishNamePBDescriptor instead')
const SetPublishNamePB$json = {
  '1': 'SetPublishNamePB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'new_name', '3': 2, '4': 1, '5': 9, '10': 'newName'},
  ],
};

/// Descriptor for `SetPublishNamePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setPublishNamePBDescriptor = $convert.base64Decode(
    'ChBTZXRQdWJsaXNoTmFtZVBCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBIZCghuZXdfbmFtZR'
    'gCIAEoCVIHbmV3TmFtZQ==');

@$core.Deprecated('Use deletedViewPBDescriptor instead')
const DeletedViewPB$json = {
  '1': 'DeletedViewPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'index', '3': 2, '4': 1, '5': 5, '9': 0, '10': 'index'},
  ],
  '8': [
    {'1': 'one_of_index'},
  ],
};

/// Descriptor for `DeletedViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deletedViewPBDescriptor = $convert.base64Decode(
    'Cg1EZWxldGVkVmlld1BCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBIWCgVpbmRleBgCIAEoBU'
    'gAUgVpbmRleEIOCgxvbmVfb2ZfaW5kZXg=');

@$core.Deprecated('Use updateViewPayloadPBDescriptor instead')
const UpdateViewPayloadPB$json = {
  '1': 'UpdateViewPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'name'},
    {'1': 'desc', '3': 3, '4': 1, '5': 9, '9': 1, '10': 'desc'},
    {'1': 'thumbnail', '3': 4, '4': 1, '5': 9, '9': 2, '10': 'thumbnail'},
    {'1': 'layout', '3': 5, '4': 1, '5': 14, '6': '.ViewLayoutPB', '9': 3, '10': 'layout'},
    {'1': 'is_favorite', '3': 6, '4': 1, '5': 8, '9': 4, '10': 'isFavorite'},
    {'1': 'extra', '3': 7, '4': 1, '5': 9, '9': 5, '10': 'extra'},
  ],
  '8': [
    {'1': 'one_of_name'},
    {'1': 'one_of_desc'},
    {'1': 'one_of_thumbnail'},
    {'1': 'one_of_layout'},
    {'1': 'one_of_is_favorite'},
    {'1': 'one_of_extra'},
  ],
};

/// Descriptor for `UpdateViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateViewPayloadPBDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVWaWV3UGF5bG9hZFBCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBIUCgRuYW1lGA'
    'IgASgJSABSBG5hbWUSFAoEZGVzYxgDIAEoCUgBUgRkZXNjEh4KCXRodW1ibmFpbBgEIAEoCUgC'
    'Ugl0aHVtYm5haWwSJwoGbGF5b3V0GAUgASgOMg0uVmlld0xheW91dFBCSANSBmxheW91dBIhCg'
    'tpc19mYXZvcml0ZRgGIAEoCEgEUgppc0Zhdm9yaXRlEhYKBWV4dHJhGAcgASgJSAVSBWV4dHJh'
    'Qg0KC29uZV9vZl9uYW1lQg0KC29uZV9vZl9kZXNjQhIKEG9uZV9vZl90aHVtYm5haWxCDwoNb2'
    '5lX29mX2xheW91dEIUChJvbmVfb2ZfaXNfZmF2b3JpdGVCDgoMb25lX29mX2V4dHJh');

@$core.Deprecated('Use moveViewPayloadPBDescriptor instead')
const MoveViewPayloadPB$json = {
  '1': 'MoveViewPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'from', '3': 2, '4': 1, '5': 5, '10': 'from'},
    {'1': 'to', '3': 3, '4': 1, '5': 5, '10': 'to'},
  ],
};

/// Descriptor for `MoveViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List moveViewPayloadPBDescriptor = $convert.base64Decode(
    'ChFNb3ZlVmlld1BheWxvYWRQQhIXCgd2aWV3X2lkGAEgASgJUgZ2aWV3SWQSEgoEZnJvbRgCIA'
    'EoBVIEZnJvbRIOCgJ0bxgDIAEoBVICdG8=');

@$core.Deprecated('Use moveNestedViewPayloadPBDescriptor instead')
const MoveNestedViewPayloadPB$json = {
  '1': 'MoveNestedViewPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'new_parent_id', '3': 2, '4': 1, '5': 9, '10': 'newParentId'},
    {'1': 'prev_view_id', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'prevViewId'},
    {'1': 'from_section', '3': 4, '4': 1, '5': 14, '6': '.ViewSectionPB', '9': 1, '10': 'fromSection'},
    {'1': 'to_section', '3': 5, '4': 1, '5': 14, '6': '.ViewSectionPB', '9': 2, '10': 'toSection'},
  ],
  '8': [
    {'1': 'one_of_prev_view_id'},
    {'1': 'one_of_from_section'},
    {'1': 'one_of_to_section'},
  ],
};

/// Descriptor for `MoveNestedViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List moveNestedViewPayloadPBDescriptor = $convert.base64Decode(
    'ChdNb3ZlTmVzdGVkVmlld1BheWxvYWRQQhIXCgd2aWV3X2lkGAEgASgJUgZ2aWV3SWQSIgoNbm'
    'V3X3BhcmVudF9pZBgCIAEoCVILbmV3UGFyZW50SWQSIgoMcHJldl92aWV3X2lkGAMgASgJSABS'
    'CnByZXZWaWV3SWQSMwoMZnJvbV9zZWN0aW9uGAQgASgOMg4uVmlld1NlY3Rpb25QQkgBUgtmcm'
    '9tU2VjdGlvbhIvCgp0b19zZWN0aW9uGAUgASgOMg4uVmlld1NlY3Rpb25QQkgCUgl0b1NlY3Rp'
    'b25CFQoTb25lX29mX3ByZXZfdmlld19pZEIVChNvbmVfb2ZfZnJvbV9zZWN0aW9uQhMKEW9uZV'
    '9vZl90b19zZWN0aW9u');

@$core.Deprecated('Use updateRecentViewPayloadPBDescriptor instead')
const UpdateRecentViewPayloadPB$json = {
  '1': 'UpdateRecentViewPayloadPB',
  '2': [
    {'1': 'view_ids', '3': 1, '4': 3, '5': 9, '10': 'viewIds'},
    {'1': 'add_in_recent', '3': 2, '4': 1, '5': 8, '10': 'addInRecent'},
  ],
};

/// Descriptor for `UpdateRecentViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRecentViewPayloadPBDescriptor = $convert.base64Decode(
    'ChlVcGRhdGVSZWNlbnRWaWV3UGF5bG9hZFBCEhkKCHZpZXdfaWRzGAEgAygJUgd2aWV3SWRzEi'
    'IKDWFkZF9pbl9yZWNlbnQYAiABKAhSC2FkZEluUmVjZW50');

@$core.Deprecated('Use updateViewVisibilityStatusPayloadPBDescriptor instead')
const UpdateViewVisibilityStatusPayloadPB$json = {
  '1': 'UpdateViewVisibilityStatusPayloadPB',
  '2': [
    {'1': 'view_ids', '3': 1, '4': 3, '5': 9, '10': 'viewIds'},
    {'1': 'is_public', '3': 2, '4': 1, '5': 8, '10': 'isPublic'},
  ],
};

/// Descriptor for `UpdateViewVisibilityStatusPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateViewVisibilityStatusPayloadPBDescriptor = $convert.base64Decode(
    'CiNVcGRhdGVWaWV3VmlzaWJpbGl0eVN0YXR1c1BheWxvYWRQQhIZCgh2aWV3X2lkcxgBIAMoCV'
    'IHdmlld0lkcxIbCglpc19wdWJsaWMYAiABKAhSCGlzUHVibGlj');

@$core.Deprecated('Use duplicateViewPayloadPBDescriptor instead')
const DuplicateViewPayloadPB$json = {
  '1': 'DuplicateViewPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'open_after_duplicate', '3': 2, '4': 1, '5': 8, '10': 'openAfterDuplicate'},
    {'1': 'include_children', '3': 3, '4': 1, '5': 8, '10': 'includeChildren'},
    {'1': 'parent_view_id', '3': 4, '4': 1, '5': 9, '9': 0, '10': 'parentViewId'},
    {'1': 'suffix', '3': 5, '4': 1, '5': 9, '9': 1, '10': 'suffix'},
    {'1': 'sync_after_create', '3': 6, '4': 1, '5': 8, '10': 'syncAfterCreate'},
  ],
  '8': [
    {'1': 'one_of_parent_view_id'},
    {'1': 'one_of_suffix'},
  ],
};

/// Descriptor for `DuplicateViewPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List duplicateViewPayloadPBDescriptor = $convert.base64Decode(
    'ChZEdXBsaWNhdGVWaWV3UGF5bG9hZFBCEhcKB3ZpZXdfaWQYASABKAlSBnZpZXdJZBIwChRvcG'
    'VuX2FmdGVyX2R1cGxpY2F0ZRgCIAEoCFISb3BlbkFmdGVyRHVwbGljYXRlEikKEGluY2x1ZGVf'
    'Y2hpbGRyZW4YAyABKAhSD2luY2x1ZGVDaGlsZHJlbhImCg5wYXJlbnRfdmlld19pZBgEIAEoCU'
    'gAUgxwYXJlbnRWaWV3SWQSGAoGc3VmZml4GAUgASgJSAFSBnN1ZmZpeBIqChFzeW5jX2FmdGVy'
    'X2NyZWF0ZRgGIAEoCFIPc3luY0FmdGVyQ3JlYXRlQhcKFW9uZV9vZl9wYXJlbnRfdmlld19pZE'
    'IPCg1vbmVfb2Zfc3VmZml4');

@$core.Deprecated('Use sharePageWithUserPayloadPBDescriptor instead')
const SharePageWithUserPayloadPB$json = {
  '1': 'SharePageWithUserPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'emails', '3': 2, '4': 3, '5': 9, '10': 'emails'},
    {'1': 'access_level', '3': 3, '4': 1, '5': 14, '6': '.AFAccessLevelPB', '10': 'accessLevel'},
    {'1': 'auto_confirm', '3': 4, '4': 1, '5': 8, '10': 'autoConfirm'},
  ],
};

/// Descriptor for `SharePageWithUserPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sharePageWithUserPayloadPBDescriptor = $convert.base64Decode(
    'ChpTaGFyZVBhZ2VXaXRoVXNlclBheWxvYWRQQhIXCgd2aWV3X2lkGAEgASgJUgZ2aWV3SWQSFg'
    'oGZW1haWxzGAIgAygJUgZlbWFpbHMSMwoMYWNjZXNzX2xldmVsGAMgASgOMhAuQUZBY2Nlc3NM'
    'ZXZlbFBCUgthY2Nlc3NMZXZlbBIhCgxhdXRvX2NvbmZpcm0YBCABKAhSC2F1dG9Db25maXJt');

@$core.Deprecated('Use removeUserFromSharedPagePayloadPBDescriptor instead')
const RemoveUserFromSharedPagePayloadPB$json = {
  '1': 'RemoveUserFromSharedPagePayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
    {'1': 'emails', '3': 2, '4': 3, '5': 9, '10': 'emails'},
  ],
};

/// Descriptor for `RemoveUserFromSharedPagePayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List removeUserFromSharedPagePayloadPBDescriptor = $convert.base64Decode(
    'CiFSZW1vdmVVc2VyRnJvbVNoYXJlZFBhZ2VQYXlsb2FkUEISFwoHdmlld19pZBgBIAEoCVIGdm'
    'lld0lkEhYKBmVtYWlscxgCIAMoCVIGZW1haWxz');

@$core.Deprecated('Use sharedUserPBDescriptor instead')
const SharedUserPB$json = {
  '1': 'SharedUserPB',
  '2': [
    {'1': 'email', '3': 1, '4': 1, '5': 9, '10': 'email'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'role', '3': 3, '4': 1, '5': 14, '6': '.AFRolePB', '10': 'role'},
    {'1': 'access_level', '3': 4, '4': 1, '5': 14, '6': '.AFAccessLevelPB', '10': 'accessLevel'},
    {'1': 'avatar_url', '3': 5, '4': 1, '5': 9, '9': 0, '10': 'avatarUrl'},
  ],
  '8': [
    {'1': 'one_of_avatar_url'},
  ],
};

/// Descriptor for `SharedUserPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sharedUserPBDescriptor = $convert.base64Decode(
    'CgxTaGFyZWRVc2VyUEISFAoFZW1haWwYASABKAlSBWVtYWlsEhIKBG5hbWUYAiABKAlSBG5hbW'
    'USHQoEcm9sZRgDIAEoDjIJLkFGUm9sZVBCUgRyb2xlEjMKDGFjY2Vzc19sZXZlbBgEIAEoDjIQ'
    'LkFGQWNjZXNzTGV2ZWxQQlILYWNjZXNzTGV2ZWwSHwoKYXZhdGFyX3VybBgFIAEoCUgAUglhdm'
    'F0YXJVcmxCEwoRb25lX29mX2F2YXRhcl91cmw=');

@$core.Deprecated('Use repeatedSharedUserPBDescriptor instead')
const RepeatedSharedUserPB$json = {
  '1': 'RepeatedSharedUserPB',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.SharedUserPB', '10': 'items'},
  ],
};

/// Descriptor for `RepeatedSharedUserPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedSharedUserPBDescriptor = $convert.base64Decode(
    'ChRSZXBlYXRlZFNoYXJlZFVzZXJQQhIjCgVpdGVtcxgBIAMoCzINLlNoYXJlZFVzZXJQQlIFaX'
    'RlbXM=');

@$core.Deprecated('Use getSharedUsersPayloadPBDescriptor instead')
const GetSharedUsersPayloadPB$json = {
  '1': 'GetSharedUsersPayloadPB',
  '2': [
    {'1': 'view_id', '3': 1, '4': 1, '5': 9, '10': 'viewId'},
  ],
};

/// Descriptor for `GetSharedUsersPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSharedUsersPayloadPBDescriptor = $convert.base64Decode(
    'ChdHZXRTaGFyZWRVc2Vyc1BheWxvYWRQQhIXCgd2aWV3X2lkGAEgASgJUgZ2aWV3SWQ=');

@$core.Deprecated('Use sharedViewPBDescriptor instead')
const SharedViewPB$json = {
  '1': 'SharedViewPB',
  '2': [
    {'1': 'view', '3': 1, '4': 1, '5': 11, '6': '.ViewPB', '10': 'view'},
    {'1': 'access_level', '3': 2, '4': 1, '5': 14, '6': '.AFAccessLevelPB', '10': 'accessLevel'},
  ],
};

/// Descriptor for `SharedViewPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sharedViewPBDescriptor = $convert.base64Decode(
    'CgxTaGFyZWRWaWV3UEISGwoEdmlldxgBIAEoCzIHLlZpZXdQQlIEdmlldxIzCgxhY2Nlc3NfbG'
    'V2ZWwYAiABKA4yEC5BRkFjY2Vzc0xldmVsUEJSC2FjY2Vzc0xldmVs');

@$core.Deprecated('Use repeatedSharedViewResponsePBDescriptor instead')
const RepeatedSharedViewResponsePB$json = {
  '1': 'RepeatedSharedViewResponsePB',
  '2': [
    {'1': 'shared_views', '3': 1, '4': 3, '5': 11, '6': '.SharedViewPB', '10': 'sharedViews'},
  ],
};

/// Descriptor for `RepeatedSharedViewResponsePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatedSharedViewResponsePBDescriptor = $convert.base64Decode(
    'ChxSZXBlYXRlZFNoYXJlZFZpZXdSZXNwb25zZVBCEjAKDHNoYXJlZF92aWV3cxgBIAMoCzINLl'
    'NoYXJlZFZpZXdQQlILc2hhcmVkVmlld3M=');

@$core.Deprecated('Use getSharedViewSectionResponsePBDescriptor instead')
const GetSharedViewSectionResponsePB$json = {
  '1': 'GetSharedViewSectionResponsePB',
  '2': [
    {'1': 'section', '3': 1, '4': 1, '5': 14, '6': '.SharedViewSectionPB', '10': 'section'},
  ],
};

/// Descriptor for `GetSharedViewSectionResponsePB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSharedViewSectionResponsePBDescriptor = $convert.base64Decode(
    'Ch5HZXRTaGFyZWRWaWV3U2VjdGlvblJlc3BvbnNlUEISLgoHc2VjdGlvbhgBIAEoDjIULlNoYX'
    'JlZFZpZXdTZWN0aW9uUEJSB3NlY3Rpb24=');

