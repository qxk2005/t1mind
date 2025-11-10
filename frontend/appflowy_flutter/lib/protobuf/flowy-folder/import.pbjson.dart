//
//  Generated code. Do not modify.
//  source: import.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use importTypePBDescriptor instead')
const ImportTypePB$json = {
  '1': 'ImportTypePB',
  '2': [
    {'1': 'HistoryDocument', '2': 0},
    {'1': 'HistoryDatabase', '2': 1},
    {'1': 'Markdown', '2': 2},
    {'1': 'AFDatabase', '2': 3},
    {'1': 'CSV', '2': 4},
    {'1': 'Word', '2': 5},
    {'1': 'Pdf', '2': 6},
  ],
};

/// Descriptor for `ImportTypePB`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List importTypePBDescriptor = $convert.base64Decode(
    'CgxJbXBvcnRUeXBlUEISEwoPSGlzdG9yeURvY3VtZW50EAASEwoPSGlzdG9yeURhdGFiYXNlEA'
    'ESDAoITWFya2Rvd24QAhIOCgpBRkRhdGFiYXNlEAMSBwoDQ1NWEAQSCAoEV29yZBAFEgcKA1Bk'
    'ZhAG');

@$core.Deprecated('Use importProgressStepDescriptor instead')
const ImportProgressStep$json = {
  '1': 'ImportProgressStep',
  '2': [
    {'1': 'Preparing', '2': 0},
    {'1': 'ReadingFile', '2': 1},
    {'1': 'ExtractingText', '2': 2},
    {'1': 'ExtractingImages', '2': 3},
    {'1': 'ParsingFormat', '2': 4},
    {'1': 'Converting', '2': 5},
    {'1': 'CreatingDocument', '2': 6},
    {'1': 'Completed', '2': 7},
    {'1': 'Error', '2': 8},
  ],
};

/// Descriptor for `ImportProgressStep`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List importProgressStepDescriptor = $convert.base64Decode(
    'ChJJbXBvcnRQcm9ncmVzc1N0ZXASDQoJUHJlcGFyaW5nEAASDwoLUmVhZGluZ0ZpbGUQARISCg'
    '5FeHRyYWN0aW5nVGV4dBACEhQKEEV4dHJhY3RpbmdJbWFnZXMQAxIRCg1QYXJzaW5nRm9ybWF0'
    'EAQSDgoKQ29udmVydGluZxAFEhQKEENyZWF0aW5nRG9jdW1lbnQQBhINCglDb21wbGV0ZWQQBx'
    'IJCgVFcnJvchAI');

@$core.Deprecated('Use importItemPayloadPBDescriptor instead')
const ImportItemPayloadPB$json = {
  '1': 'ImportItemPayloadPB',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'data', '3': 2, '4': 1, '5': 12, '9': 0, '10': 'data'},
    {'1': 'file_path', '3': 3, '4': 1, '5': 9, '9': 1, '10': 'filePath'},
    {'1': 'view_layout', '3': 4, '4': 1, '5': 14, '6': '.ViewLayoutPB', '10': 'viewLayout'},
    {'1': 'import_type', '3': 5, '4': 1, '5': 14, '6': '.ImportTypePB', '10': 'importType'},
    {'1': 'view_id', '3': 6, '4': 1, '5': 9, '9': 2, '10': 'viewId'},
  ],
  '8': [
    {'1': 'one_of_data'},
    {'1': 'one_of_file_path'},
    {'1': 'one_of_view_id'},
  ],
};

/// Descriptor for `ImportItemPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importItemPayloadPBDescriptor = $convert.base64Decode(
    'ChNJbXBvcnRJdGVtUGF5bG9hZFBCEhIKBG5hbWUYASABKAlSBG5hbWUSFAoEZGF0YRgCIAEoDE'
    'gAUgRkYXRhEh0KCWZpbGVfcGF0aBgDIAEoCUgBUghmaWxlUGF0aBIuCgt2aWV3X2xheW91dBgE'
    'IAEoDjINLlZpZXdMYXlvdXRQQlIKdmlld0xheW91dBIuCgtpbXBvcnRfdHlwZRgFIAEoDjINLk'
    'ltcG9ydFR5cGVQQlIKaW1wb3J0VHlwZRIZCgd2aWV3X2lkGAYgASgJSAJSBnZpZXdJZEINCgtv'
    'bmVfb2ZfZGF0YUISChBvbmVfb2ZfZmlsZV9wYXRoQhAKDm9uZV9vZl92aWV3X2lk');

@$core.Deprecated('Use importPayloadPBDescriptor instead')
const ImportPayloadPB$json = {
  '1': 'ImportPayloadPB',
  '2': [
    {'1': 'parent_view_id', '3': 1, '4': 1, '5': 9, '10': 'parentViewId'},
    {'1': 'items', '3': 2, '4': 3, '5': 11, '6': '.ImportItemPayloadPB', '10': 'items'},
  ],
};

/// Descriptor for `ImportPayloadPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importPayloadPBDescriptor = $convert.base64Decode(
    'Cg9JbXBvcnRQYXlsb2FkUEISJAoOcGFyZW50X3ZpZXdfaWQYASABKAlSDHBhcmVudFZpZXdJZB'
    'IqCgVpdGVtcxgCIAMoCzIULkltcG9ydEl0ZW1QYXlsb2FkUEJSBWl0ZW1z');

@$core.Deprecated('Use importZipPBDescriptor instead')
const ImportZipPB$json = {
  '1': 'ImportZipPB',
  '2': [
    {'1': 'file_path', '3': 1, '4': 1, '5': 9, '10': 'filePath'},
  ],
};

/// Descriptor for `ImportZipPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importZipPBDescriptor = $convert.base64Decode(
    'CgtJbXBvcnRaaXBQQhIbCglmaWxlX3BhdGgYASABKAlSCGZpbGVQYXRo');

@$core.Deprecated('Use registerImportProgressStreamPBDescriptor instead')
const RegisterImportProgressStreamPB$json = {
  '1': 'RegisterImportProgressStreamPB',
  '2': [
    {'1': 'port', '3': 1, '4': 1, '5': 3, '10': 'port'},
  ],
};

/// Descriptor for `RegisterImportProgressStreamPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerImportProgressStreamPBDescriptor = $convert.base64Decode(
    'Ch5SZWdpc3RlckltcG9ydFByb2dyZXNzU3RyZWFtUEISEgoEcG9ydBgBIAEoA1IEcG9ydA==');

@$core.Deprecated('Use getImportProgressPBDescriptor instead')
const GetImportProgressPB$json = {
  '1': 'GetImportProgressPB',
  '2': [
    {'1': 'import_id', '3': 1, '4': 1, '5': 9, '10': 'importId'},
  ],
};

/// Descriptor for `GetImportProgressPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getImportProgressPBDescriptor = $convert.base64Decode(
    'ChNHZXRJbXBvcnRQcm9ncmVzc1BCEhsKCWltcG9ydF9pZBgBIAEoCVIIaW1wb3J0SWQ=');

@$core.Deprecated('Use importLogEntryPBDescriptor instead')
const ImportLogEntryPB$json = {
  '1': 'ImportLogEntryPB',
  '2': [
    {'1': 'timestamp', '3': 1, '4': 1, '5': 3, '10': 'timestamp'},
    {'1': 'level', '3': 2, '4': 1, '5': 9, '10': 'level'},
    {'1': 'message', '3': 3, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `ImportLogEntryPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importLogEntryPBDescriptor = $convert.base64Decode(
    'ChBJbXBvcnRMb2dFbnRyeVBCEhwKCXRpbWVzdGFtcBgBIAEoA1IJdGltZXN0YW1wEhQKBWxldm'
    'VsGAIgASgJUgVsZXZlbBIYCgdtZXNzYWdlGAMgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use importProgressPBDescriptor instead')
const ImportProgressPB$json = {
  '1': 'ImportProgressPB',
  '2': [
    {'1': 'import_id', '3': 1, '4': 1, '5': 9, '10': 'importId'},
    {'1': 'file_name', '3': 2, '4': 1, '5': 9, '10': 'fileName'},
    {'1': 'progress', '3': 3, '4': 1, '5': 1, '10': 'progress'},
    {'1': 'current_step', '3': 4, '4': 1, '5': 9, '10': 'currentStep'},
    {'1': 'error', '3': 5, '4': 1, '5': 9, '9': 0, '10': 'error'},
    {'1': 'logs', '3': 6, '4': 3, '5': 11, '6': '.ImportLogEntryPB', '10': 'logs'},
  ],
  '8': [
    {'1': 'one_of_error'},
  ],
};

/// Descriptor for `ImportProgressPB`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importProgressPBDescriptor = $convert.base64Decode(
    'ChBJbXBvcnRQcm9ncmVzc1BCEhsKCWltcG9ydF9pZBgBIAEoCVIIaW1wb3J0SWQSGwoJZmlsZV'
    '9uYW1lGAIgASgJUghmaWxlTmFtZRIaCghwcm9ncmVzcxgDIAEoAVIIcHJvZ3Jlc3MSIQoMY3Vy'
    'cmVudF9zdGVwGAQgASgJUgtjdXJyZW50U3RlcBIWCgVlcnJvchgFIAEoCUgAUgVlcnJvchIlCg'
    'Rsb2dzGAYgAygLMhEuSW1wb3J0TG9nRW50cnlQQlIEbG9nc0IOCgxvbmVfb2ZfZXJyb3I=');

