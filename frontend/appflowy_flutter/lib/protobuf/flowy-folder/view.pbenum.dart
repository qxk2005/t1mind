//
//  Generated code. Do not modify.
//  source: view.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class ViewLayoutPB extends $pb.ProtobufEnum {
  static const ViewLayoutPB Document = ViewLayoutPB._(0, _omitEnumNames ? '' : 'Document');
  static const ViewLayoutPB Grid = ViewLayoutPB._(1, _omitEnumNames ? '' : 'Grid');
  static const ViewLayoutPB Board = ViewLayoutPB._(2, _omitEnumNames ? '' : 'Board');
  static const ViewLayoutPB Calendar = ViewLayoutPB._(3, _omitEnumNames ? '' : 'Calendar');
  static const ViewLayoutPB Chat = ViewLayoutPB._(4, _omitEnumNames ? '' : 'Chat');

  static const $core.List<ViewLayoutPB> values = <ViewLayoutPB> [
    Document,
    Grid,
    Board,
    Calendar,
    Chat,
  ];

  static final $core.Map<$core.int, ViewLayoutPB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ViewLayoutPB? valueOf($core.int value) => _byValue[value];

  const ViewLayoutPB._($core.int v, $core.String n) : super(v, n);
}

class ViewSectionPB extends $pb.ProtobufEnum {
  static const ViewSectionPB Private = ViewSectionPB._(0, _omitEnumNames ? '' : 'Private');
  static const ViewSectionPB Public = ViewSectionPB._(1, _omitEnumNames ? '' : 'Public');

  static const $core.List<ViewSectionPB> values = <ViewSectionPB> [
    Private,
    Public,
  ];

  static final $core.Map<$core.int, ViewSectionPB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ViewSectionPB? valueOf($core.int value) => _byValue[value];

  const ViewSectionPB._($core.int v, $core.String n) : super(v, n);
}

class AFAccessLevelPB extends $pb.ProtobufEnum {
  static const AFAccessLevelPB ReadOnly = AFAccessLevelPB._(0, _omitEnumNames ? '' : 'ReadOnly');
  static const AFAccessLevelPB ReadAndComment = AFAccessLevelPB._(1, _omitEnumNames ? '' : 'ReadAndComment');
  static const AFAccessLevelPB ReadAndWrite = AFAccessLevelPB._(2, _omitEnumNames ? '' : 'ReadAndWrite');
  static const AFAccessLevelPB FullAccess = AFAccessLevelPB._(3, _omitEnumNames ? '' : 'FullAccess');

  static const $core.List<AFAccessLevelPB> values = <AFAccessLevelPB> [
    ReadOnly,
    ReadAndComment,
    ReadAndWrite,
    FullAccess,
  ];

  static final $core.Map<$core.int, AFAccessLevelPB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static AFAccessLevelPB? valueOf($core.int value) => _byValue[value];

  const AFAccessLevelPB._($core.int v, $core.String n) : super(v, n);
}

class AFRolePB extends $pb.ProtobufEnum {
  static const AFRolePB Owner = AFRolePB._(0, _omitEnumNames ? '' : 'Owner');
  static const AFRolePB Member = AFRolePB._(1, _omitEnumNames ? '' : 'Member');
  static const AFRolePB Guest = AFRolePB._(2, _omitEnumNames ? '' : 'Guest');

  static const $core.List<AFRolePB> values = <AFRolePB> [
    Owner,
    Member,
    Guest,
  ];

  static final $core.Map<$core.int, AFRolePB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static AFRolePB? valueOf($core.int value) => _byValue[value];

  const AFRolePB._($core.int v, $core.String n) : super(v, n);
}

class SharedViewSectionPB extends $pb.ProtobufEnum {
  static const SharedViewSectionPB PrivateSection = SharedViewSectionPB._(0, _omitEnumNames ? '' : 'PrivateSection');
  static const SharedViewSectionPB PublicSection = SharedViewSectionPB._(1, _omitEnumNames ? '' : 'PublicSection');
  static const SharedViewSectionPB SharedSection = SharedViewSectionPB._(2, _omitEnumNames ? '' : 'SharedSection');

  static const $core.List<SharedViewSectionPB> values = <SharedViewSectionPB> [
    PrivateSection,
    PublicSection,
    SharedSection,
  ];

  static final $core.Map<$core.int, SharedViewSectionPB> _byValue = $pb.ProtobufEnum.initByValue(values);
  static SharedViewSectionPB? valueOf($core.int value) => _byValue[value];

  const SharedViewSectionPB._($core.int v, $core.String n) : super(v, n);
}


const _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
