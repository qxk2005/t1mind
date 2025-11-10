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

import 'icon.pbenum.dart';

export 'icon.pbenum.dart';

class ViewIconPB extends $pb.GeneratedMessage {
  factory ViewIconPB({
    ViewIconTypePB? ty,
    $core.String? value,
  }) {
    final $result = create();
    if (ty != null) {
      $result.ty = ty;
    }
    if (value != null) {
      $result.value = value;
    }
    return $result;
  }
  ViewIconPB._() : super();
  factory ViewIconPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ViewIconPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ViewIconPB', createEmptyInstance: create)
    ..e<ViewIconTypePB>(1, _omitFieldNames ? '' : 'ty', $pb.PbFieldType.OE, defaultOrMaker: ViewIconTypePB.Emoji, valueOf: ViewIconTypePB.valueOf, enumValues: ViewIconTypePB.values)
    ..aOS(2, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ViewIconPB clone() => ViewIconPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ViewIconPB copyWith(void Function(ViewIconPB) updates) => super.copyWith((message) => updates(message as ViewIconPB)) as ViewIconPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ViewIconPB create() => ViewIconPB._();
  ViewIconPB createEmptyInstance() => create();
  static $pb.PbList<ViewIconPB> createRepeated() => $pb.PbList<ViewIconPB>();
  @$core.pragma('dart2js:noInline')
  static ViewIconPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ViewIconPB>(create);
  static ViewIconPB? _defaultInstance;

  @$pb.TagNumber(1)
  ViewIconTypePB get ty => $_getN(0);
  @$pb.TagNumber(1)
  set ty(ViewIconTypePB v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasTy() => $_has(0);
  @$pb.TagNumber(1)
  void clearTy() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get value => $_getSZ(1);
  @$pb.TagNumber(2)
  set value($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => clearField(2);
}

enum UpdateViewIconPayloadPB_OneOfIcon {
  icon, 
  notSet
}

class UpdateViewIconPayloadPB extends $pb.GeneratedMessage {
  factory UpdateViewIconPayloadPB({
    $core.String? viewId,
    ViewIconPB? icon,
  }) {
    final $result = create();
    if (viewId != null) {
      $result.viewId = viewId;
    }
    if (icon != null) {
      $result.icon = icon;
    }
    return $result;
  }
  UpdateViewIconPayloadPB._() : super();
  factory UpdateViewIconPayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory UpdateViewIconPayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, UpdateViewIconPayloadPB_OneOfIcon> _UpdateViewIconPayloadPB_OneOfIconByTag = {
    2 : UpdateViewIconPayloadPB_OneOfIcon.icon,
    0 : UpdateViewIconPayloadPB_OneOfIcon.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'UpdateViewIconPayloadPB', createEmptyInstance: create)
    ..oo(0, [2])
    ..aOS(1, _omitFieldNames ? '' : 'viewId')
    ..aOM<ViewIconPB>(2, _omitFieldNames ? '' : 'icon', subBuilder: ViewIconPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  UpdateViewIconPayloadPB clone() => UpdateViewIconPayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  UpdateViewIconPayloadPB copyWith(void Function(UpdateViewIconPayloadPB) updates) => super.copyWith((message) => updates(message as UpdateViewIconPayloadPB)) as UpdateViewIconPayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateViewIconPayloadPB create() => UpdateViewIconPayloadPB._();
  UpdateViewIconPayloadPB createEmptyInstance() => create();
  static $pb.PbList<UpdateViewIconPayloadPB> createRepeated() => $pb.PbList<UpdateViewIconPayloadPB>();
  @$core.pragma('dart2js:noInline')
  static UpdateViewIconPayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<UpdateViewIconPayloadPB>(create);
  static UpdateViewIconPayloadPB? _defaultInstance;

  UpdateViewIconPayloadPB_OneOfIcon whichOneOfIcon() => _UpdateViewIconPayloadPB_OneOfIconByTag[$_whichOneof(0)]!;
  void clearOneOfIcon() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get viewId => $_getSZ(0);
  @$pb.TagNumber(1)
  set viewId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasViewId() => $_has(0);
  @$pb.TagNumber(1)
  void clearViewId() => clearField(1);

  @$pb.TagNumber(2)
  ViewIconPB get icon => $_getN(1);
  @$pb.TagNumber(2)
  set icon(ViewIconPB v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasIcon() => $_has(1);
  @$pb.TagNumber(2)
  void clearIcon() => clearField(2);
  @$pb.TagNumber(2)
  ViewIconPB ensureIcon() => $_ensure(1);
}


const _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
