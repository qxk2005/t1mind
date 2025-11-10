//
//  Generated code. Do not modify.
//  source: icon.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class ViewIconTypePB extends $pb.ProtobufEnum {
  static const ViewIconTypePB Emoji = ViewIconTypePB._(0, _omitEnumNames ? '' : 'Emoji');
  static const ViewIconTypePB Url = ViewIconTypePB._(1, _omitEnumNames ? '' : 'Url');
  static const ViewIconTypePB Icon = ViewIconTypePB._(2, _omitEnumNames ? '' : 'Icon');

  static const $core.List<ViewIconTypePB> values = <ViewIconTypePB> [
    Emoji,
    Url,
    Icon,
  ];

  static final $core.Map<$core.int, ViewIconTypePB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ViewIconTypePB? valueOf($core.int value) => _byValue[value];

  const ViewIconTypePB._($core.int v, $core.String n) : super(v, n);
}


const _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
