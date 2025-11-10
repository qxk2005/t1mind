//
//  Generated code. Do not modify.
//  source: trash.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

class TrashPB extends $pb.GeneratedMessage {
  factory TrashPB({
    $core.String? id,
    $core.String? name,
    $fixnum.Int64? modifiedTime,
    $fixnum.Int64? createTime,
  }) {
    final $result = create();
    if (id != null) {
      $result.id = id;
    }
    if (name != null) {
      $result.name = name;
    }
    if (modifiedTime != null) {
      $result.modifiedTime = modifiedTime;
    }
    if (createTime != null) {
      $result.createTime = createTime;
    }
    return $result;
  }
  TrashPB._() : super();
  factory TrashPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TrashPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'TrashPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aInt64(3, _omitFieldNames ? '' : 'modifiedTime')
    ..aInt64(4, _omitFieldNames ? '' : 'createTime')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  TrashPB clone() => TrashPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  TrashPB copyWith(void Function(TrashPB) updates) => super.copyWith((message) => updates(message as TrashPB)) as TrashPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TrashPB create() => TrashPB._();
  TrashPB createEmptyInstance() => create();
  static $pb.PbList<TrashPB> createRepeated() => $pb.PbList<TrashPB>();
  @$core.pragma('dart2js:noInline')
  static TrashPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TrashPB>(create);
  static TrashPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get modifiedTime => $_getI64(2);
  @$pb.TagNumber(3)
  set modifiedTime($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasModifiedTime() => $_has(2);
  @$pb.TagNumber(3)
  void clearModifiedTime() => clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get createTime => $_getI64(3);
  @$pb.TagNumber(4)
  set createTime($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasCreateTime() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreateTime() => clearField(4);
}

class RepeatedTrashPB extends $pb.GeneratedMessage {
  factory RepeatedTrashPB({
    $core.Iterable<TrashPB>? items,
  }) {
    final $result = create();
    if (items != null) {
      $result.items.addAll(items);
    }
    return $result;
  }
  RepeatedTrashPB._() : super();
  factory RepeatedTrashPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RepeatedTrashPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RepeatedTrashPB', createEmptyInstance: create)
    ..pc<TrashPB>(1, _omitFieldNames ? '' : 'items', $pb.PbFieldType.PM, subBuilder: TrashPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RepeatedTrashPB clone() => RepeatedTrashPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RepeatedTrashPB copyWith(void Function(RepeatedTrashPB) updates) => super.copyWith((message) => updates(message as RepeatedTrashPB)) as RepeatedTrashPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RepeatedTrashPB create() => RepeatedTrashPB._();
  RepeatedTrashPB createEmptyInstance() => create();
  static $pb.PbList<RepeatedTrashPB> createRepeated() => $pb.PbList<RepeatedTrashPB>();
  @$core.pragma('dart2js:noInline')
  static RepeatedTrashPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RepeatedTrashPB>(create);
  static RepeatedTrashPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<TrashPB> get items => $_getList(0);
}

class TrashIdPB extends $pb.GeneratedMessage {
  factory TrashIdPB({
    $core.String? id,
  }) {
    final $result = create();
    if (id != null) {
      $result.id = id;
    }
    return $result;
  }
  TrashIdPB._() : super();
  factory TrashIdPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TrashIdPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'TrashIdPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  TrashIdPB clone() => TrashIdPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  TrashIdPB copyWith(void Function(TrashIdPB) updates) => super.copyWith((message) => updates(message as TrashIdPB)) as TrashIdPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TrashIdPB create() => TrashIdPB._();
  TrashIdPB createEmptyInstance() => create();
  static $pb.PbList<TrashIdPB> createRepeated() => $pb.PbList<TrashIdPB>();
  @$core.pragma('dart2js:noInline')
  static TrashIdPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TrashIdPB>(create);
  static TrashIdPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);
}

class RepeatedTrashIdPB extends $pb.GeneratedMessage {
  factory RepeatedTrashIdPB({
    $core.Iterable<TrashIdPB>? items,
  }) {
    final $result = create();
    if (items != null) {
      $result.items.addAll(items);
    }
    return $result;
  }
  RepeatedTrashIdPB._() : super();
  factory RepeatedTrashIdPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RepeatedTrashIdPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RepeatedTrashIdPB', createEmptyInstance: create)
    ..pc<TrashIdPB>(1, _omitFieldNames ? '' : 'items', $pb.PbFieldType.PM, subBuilder: TrashIdPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RepeatedTrashIdPB clone() => RepeatedTrashIdPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RepeatedTrashIdPB copyWith(void Function(RepeatedTrashIdPB) updates) => super.copyWith((message) => updates(message as RepeatedTrashIdPB)) as RepeatedTrashIdPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RepeatedTrashIdPB create() => RepeatedTrashIdPB._();
  RepeatedTrashIdPB createEmptyInstance() => create();
  static $pb.PbList<RepeatedTrashIdPB> createRepeated() => $pb.PbList<RepeatedTrashIdPB>();
  @$core.pragma('dart2js:noInline')
  static RepeatedTrashIdPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RepeatedTrashIdPB>(create);
  static RepeatedTrashIdPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<TrashIdPB> get items => $_getList(0);
}


const _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
