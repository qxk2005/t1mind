//
//  Generated code. Do not modify.
//  source: import.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class ImportTypePB extends $pb.ProtobufEnum {
  static const ImportTypePB HistoryDocument = ImportTypePB._(0, _omitEnumNames ? '' : 'HistoryDocument');
  static const ImportTypePB HistoryDatabase = ImportTypePB._(1, _omitEnumNames ? '' : 'HistoryDatabase');
  static const ImportTypePB Markdown = ImportTypePB._(2, _omitEnumNames ? '' : 'Markdown');
  static const ImportTypePB AFDatabase = ImportTypePB._(3, _omitEnumNames ? '' : 'AFDatabase');
  static const ImportTypePB CSV = ImportTypePB._(4, _omitEnumNames ? '' : 'CSV');
  static const ImportTypePB Word = ImportTypePB._(5, _omitEnumNames ? '' : 'Word');
  static const ImportTypePB Pdf = ImportTypePB._(6, _omitEnumNames ? '' : 'Pdf');

  static const $core.List<ImportTypePB> values = <ImportTypePB> [
    HistoryDocument,
    HistoryDatabase,
    Markdown,
    AFDatabase,
    CSV,
    Word,
    Pdf,
  ];

  static final $core.Map<$core.int, ImportTypePB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ImportTypePB? valueOf($core.int value) => _byValue[value];

  const ImportTypePB._($core.int v, $core.String n) : super(v, n);
}

class ImportProgressStep extends $pb.ProtobufEnum {
  static const ImportProgressStep Preparing = ImportProgressStep._(0, _omitEnumNames ? '' : 'Preparing');
  static const ImportProgressStep ReadingFile = ImportProgressStep._(1, _omitEnumNames ? '' : 'ReadingFile');
  static const ImportProgressStep ExtractingText = ImportProgressStep._(2, _omitEnumNames ? '' : 'ExtractingText');
  static const ImportProgressStep ExtractingImages = ImportProgressStep._(3, _omitEnumNames ? '' : 'ExtractingImages');
  static const ImportProgressStep ParsingFormat = ImportProgressStep._(4, _omitEnumNames ? '' : 'ParsingFormat');
  static const ImportProgressStep Converting = ImportProgressStep._(5, _omitEnumNames ? '' : 'Converting');
  static const ImportProgressStep CreatingDocument = ImportProgressStep._(6, _omitEnumNames ? '' : 'CreatingDocument');
  static const ImportProgressStep Completed = ImportProgressStep._(7, _omitEnumNames ? '' : 'Completed');
  static const ImportProgressStep Error = ImportProgressStep._(8, _omitEnumNames ? '' : 'Error');

  static const $core.List<ImportProgressStep> values = <ImportProgressStep> [
    Preparing,
    ReadingFile,
    ExtractingText,
    ExtractingImages,
    ParsingFormat,
    Converting,
    CreatingDocument,
    Completed,
    Error,
  ];

  static final $core.Map<$core.int, ImportProgressStep> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ImportProgressStep? valueOf($core.int value) => _byValue[value];

  const ImportProgressStep._($core.int v, $core.String n) : super(v, n);
}


const _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
