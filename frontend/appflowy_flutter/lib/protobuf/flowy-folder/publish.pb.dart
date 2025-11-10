//
//  Generated code. Do not modify.
//  source: publish.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'icon.pb.dart' as $0;
import 'view.pb.dart' as $1;
import 'view.pbenum.dart' as $1;

enum PublishViewParamsPB_OneOfPublishName {
  publishName, 
  notSet
}

enum PublishViewParamsPB_OneOfSelectedViewIds {
  selectedViewIds, 
  notSet
}

class PublishViewParamsPB extends $pb.GeneratedMessage {
  factory PublishViewParamsPB({
    $core.String? viewId,
    $core.String? publishName,
    $1.RepeatedViewIdPB? selectedViewIds,
  }) {
    final $result = create();
    if (viewId != null) {
      $result.viewId = viewId;
    }
    if (publishName != null) {
      $result.publishName = publishName;
    }
    if (selectedViewIds != null) {
      $result.selectedViewIds = selectedViewIds;
    }
    return $result;
  }
  PublishViewParamsPB._() : super();
  factory PublishViewParamsPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory PublishViewParamsPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, PublishViewParamsPB_OneOfPublishName> _PublishViewParamsPB_OneOfPublishNameByTag = {
    2 : PublishViewParamsPB_OneOfPublishName.publishName,
    0 : PublishViewParamsPB_OneOfPublishName.notSet
  };
  static const $core.Map<$core.int, PublishViewParamsPB_OneOfSelectedViewIds> _PublishViewParamsPB_OneOfSelectedViewIdsByTag = {
    3 : PublishViewParamsPB_OneOfSelectedViewIds.selectedViewIds,
    0 : PublishViewParamsPB_OneOfSelectedViewIds.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'PublishViewParamsPB', createEmptyInstance: create)
    ..oo(0, [2])
    ..oo(1, [3])
    ..aOS(1, _omitFieldNames ? '' : 'viewId')
    ..aOS(2, _omitFieldNames ? '' : 'publishName')
    ..aOM<$1.RepeatedViewIdPB>(3, _omitFieldNames ? '' : 'selectedViewIds', subBuilder: $1.RepeatedViewIdPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  PublishViewParamsPB clone() => PublishViewParamsPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  PublishViewParamsPB copyWith(void Function(PublishViewParamsPB) updates) => super.copyWith((message) => updates(message as PublishViewParamsPB)) as PublishViewParamsPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishViewParamsPB create() => PublishViewParamsPB._();
  PublishViewParamsPB createEmptyInstance() => create();
  static $pb.PbList<PublishViewParamsPB> createRepeated() => $pb.PbList<PublishViewParamsPB>();
  @$core.pragma('dart2js:noInline')
  static PublishViewParamsPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PublishViewParamsPB>(create);
  static PublishViewParamsPB? _defaultInstance;

  PublishViewParamsPB_OneOfPublishName whichOneOfPublishName() => _PublishViewParamsPB_OneOfPublishNameByTag[$_whichOneof(0)]!;
  void clearOneOfPublishName() => clearField($_whichOneof(0));

  PublishViewParamsPB_OneOfSelectedViewIds whichOneOfSelectedViewIds() => _PublishViewParamsPB_OneOfSelectedViewIdsByTag[$_whichOneof(1)]!;
  void clearOneOfSelectedViewIds() => clearField($_whichOneof(1));

  @$pb.TagNumber(1)
  $core.String get viewId => $_getSZ(0);
  @$pb.TagNumber(1)
  set viewId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasViewId() => $_has(0);
  @$pb.TagNumber(1)
  void clearViewId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get publishName => $_getSZ(1);
  @$pb.TagNumber(2)
  set publishName($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasPublishName() => $_has(1);
  @$pb.TagNumber(2)
  void clearPublishName() => clearField(2);

  @$pb.TagNumber(3)
  $1.RepeatedViewIdPB get selectedViewIds => $_getN(2);
  @$pb.TagNumber(3)
  set selectedViewIds($1.RepeatedViewIdPB v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasSelectedViewIds() => $_has(2);
  @$pb.TagNumber(3)
  void clearSelectedViewIds() => clearField(3);
  @$pb.TagNumber(3)
  $1.RepeatedViewIdPB ensureSelectedViewIds() => $_ensure(2);
}

class UnpublishViewsPayloadPB extends $pb.GeneratedMessage {
  factory UnpublishViewsPayloadPB({
    $core.Iterable<$core.String>? viewIds,
  }) {
    final $result = create();
    if (viewIds != null) {
      $result.viewIds.addAll(viewIds);
    }
    return $result;
  }
  UnpublishViewsPayloadPB._() : super();
  factory UnpublishViewsPayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory UnpublishViewsPayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'UnpublishViewsPayloadPB', createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'viewIds')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  UnpublishViewsPayloadPB clone() => UnpublishViewsPayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  UnpublishViewsPayloadPB copyWith(void Function(UnpublishViewsPayloadPB) updates) => super.copyWith((message) => updates(message as UnpublishViewsPayloadPB)) as UnpublishViewsPayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UnpublishViewsPayloadPB create() => UnpublishViewsPayloadPB._();
  UnpublishViewsPayloadPB createEmptyInstance() => create();
  static $pb.PbList<UnpublishViewsPayloadPB> createRepeated() => $pb.PbList<UnpublishViewsPayloadPB>();
  @$core.pragma('dart2js:noInline')
  static UnpublishViewsPayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<UnpublishViewsPayloadPB>(create);
  static UnpublishViewsPayloadPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.String> get viewIds => $_getList(0);
}

class PublishInfoViewPB extends $pb.GeneratedMessage {
  factory PublishInfoViewPB({
    FolderViewMinimalPB? view,
    PublishInfoResponsePB? info,
  }) {
    final $result = create();
    if (view != null) {
      $result.view = view;
    }
    if (info != null) {
      $result.info = info;
    }
    return $result;
  }
  PublishInfoViewPB._() : super();
  factory PublishInfoViewPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory PublishInfoViewPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'PublishInfoViewPB', createEmptyInstance: create)
    ..aOM<FolderViewMinimalPB>(1, _omitFieldNames ? '' : 'view', subBuilder: FolderViewMinimalPB.create)
    ..aOM<PublishInfoResponsePB>(2, _omitFieldNames ? '' : 'info', subBuilder: PublishInfoResponsePB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  PublishInfoViewPB clone() => PublishInfoViewPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  PublishInfoViewPB copyWith(void Function(PublishInfoViewPB) updates) => super.copyWith((message) => updates(message as PublishInfoViewPB)) as PublishInfoViewPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishInfoViewPB create() => PublishInfoViewPB._();
  PublishInfoViewPB createEmptyInstance() => create();
  static $pb.PbList<PublishInfoViewPB> createRepeated() => $pb.PbList<PublishInfoViewPB>();
  @$core.pragma('dart2js:noInline')
  static PublishInfoViewPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PublishInfoViewPB>(create);
  static PublishInfoViewPB? _defaultInstance;

  @$pb.TagNumber(1)
  FolderViewMinimalPB get view => $_getN(0);
  @$pb.TagNumber(1)
  set view(FolderViewMinimalPB v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasView() => $_has(0);
  @$pb.TagNumber(1)
  void clearView() => clearField(1);
  @$pb.TagNumber(1)
  FolderViewMinimalPB ensureView() => $_ensure(0);

  @$pb.TagNumber(2)
  PublishInfoResponsePB get info => $_getN(1);
  @$pb.TagNumber(2)
  set info(PublishInfoResponsePB v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasInfo() => $_has(1);
  @$pb.TagNumber(2)
  void clearInfo() => clearField(2);
  @$pb.TagNumber(2)
  PublishInfoResponsePB ensureInfo() => $_ensure(1);
}

enum FolderViewMinimalPB_OneOfIcon {
  icon, 
  notSet
}

class FolderViewMinimalPB extends $pb.GeneratedMessage {
  factory FolderViewMinimalPB({
    $core.String? viewId,
    $core.String? name,
    $0.ViewIconPB? icon,
    $1.ViewLayoutPB? layout,
  }) {
    final $result = create();
    if (viewId != null) {
      $result.viewId = viewId;
    }
    if (name != null) {
      $result.name = name;
    }
    if (icon != null) {
      $result.icon = icon;
    }
    if (layout != null) {
      $result.layout = layout;
    }
    return $result;
  }
  FolderViewMinimalPB._() : super();
  factory FolderViewMinimalPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FolderViewMinimalPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, FolderViewMinimalPB_OneOfIcon> _FolderViewMinimalPB_OneOfIconByTag = {
    3 : FolderViewMinimalPB_OneOfIcon.icon,
    0 : FolderViewMinimalPB_OneOfIcon.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'FolderViewMinimalPB', createEmptyInstance: create)
    ..oo(0, [3])
    ..aOS(1, _omitFieldNames ? '' : 'viewId')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOM<$0.ViewIconPB>(3, _omitFieldNames ? '' : 'icon', subBuilder: $0.ViewIconPB.create)
    ..e<$1.ViewLayoutPB>(4, _omitFieldNames ? '' : 'layout', $pb.PbFieldType.OE, defaultOrMaker: $1.ViewLayoutPB.Document, valueOf: $1.ViewLayoutPB.valueOf, enumValues: $1.ViewLayoutPB.values)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FolderViewMinimalPB clone() => FolderViewMinimalPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FolderViewMinimalPB copyWith(void Function(FolderViewMinimalPB) updates) => super.copyWith((message) => updates(message as FolderViewMinimalPB)) as FolderViewMinimalPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FolderViewMinimalPB create() => FolderViewMinimalPB._();
  FolderViewMinimalPB createEmptyInstance() => create();
  static $pb.PbList<FolderViewMinimalPB> createRepeated() => $pb.PbList<FolderViewMinimalPB>();
  @$core.pragma('dart2js:noInline')
  static FolderViewMinimalPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FolderViewMinimalPB>(create);
  static FolderViewMinimalPB? _defaultInstance;

  FolderViewMinimalPB_OneOfIcon whichOneOfIcon() => _FolderViewMinimalPB_OneOfIconByTag[$_whichOneof(0)]!;
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
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => clearField(2);

  @$pb.TagNumber(3)
  $0.ViewIconPB get icon => $_getN(2);
  @$pb.TagNumber(3)
  set icon($0.ViewIconPB v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasIcon() => $_has(2);
  @$pb.TagNumber(3)
  void clearIcon() => clearField(3);
  @$pb.TagNumber(3)
  $0.ViewIconPB ensureIcon() => $_ensure(2);

  @$pb.TagNumber(4)
  $1.ViewLayoutPB get layout => $_getN(3);
  @$pb.TagNumber(4)
  set layout($1.ViewLayoutPB v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasLayout() => $_has(3);
  @$pb.TagNumber(4)
  void clearLayout() => clearField(4);
}

enum PublishInfoResponsePB_OneOfNamespace {
  namespace, 
  notSet
}

enum PublishInfoResponsePB_OneOfUnpublishedAtTimestampSec {
  unpublishedAtTimestampSec, 
  notSet
}

class PublishInfoResponsePB extends $pb.GeneratedMessage {
  factory PublishInfoResponsePB({
    $core.String? viewId,
    $core.String? publishName,
    $core.String? namespace,
    $core.String? publisherEmail,
    $fixnum.Int64? publishTimestampSec,
    $fixnum.Int64? unpublishedAtTimestampSec,
  }) {
    final $result = create();
    if (viewId != null) {
      $result.viewId = viewId;
    }
    if (publishName != null) {
      $result.publishName = publishName;
    }
    if (namespace != null) {
      $result.namespace = namespace;
    }
    if (publisherEmail != null) {
      $result.publisherEmail = publisherEmail;
    }
    if (publishTimestampSec != null) {
      $result.publishTimestampSec = publishTimestampSec;
    }
    if (unpublishedAtTimestampSec != null) {
      $result.unpublishedAtTimestampSec = unpublishedAtTimestampSec;
    }
    return $result;
  }
  PublishInfoResponsePB._() : super();
  factory PublishInfoResponsePB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory PublishInfoResponsePB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, PublishInfoResponsePB_OneOfNamespace> _PublishInfoResponsePB_OneOfNamespaceByTag = {
    3 : PublishInfoResponsePB_OneOfNamespace.namespace,
    0 : PublishInfoResponsePB_OneOfNamespace.notSet
  };
  static const $core.Map<$core.int, PublishInfoResponsePB_OneOfUnpublishedAtTimestampSec> _PublishInfoResponsePB_OneOfUnpublishedAtTimestampSecByTag = {
    6 : PublishInfoResponsePB_OneOfUnpublishedAtTimestampSec.unpublishedAtTimestampSec,
    0 : PublishInfoResponsePB_OneOfUnpublishedAtTimestampSec.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'PublishInfoResponsePB', createEmptyInstance: create)
    ..oo(0, [3])
    ..oo(1, [6])
    ..aOS(1, _omitFieldNames ? '' : 'viewId')
    ..aOS(2, _omitFieldNames ? '' : 'publishName')
    ..aOS(3, _omitFieldNames ? '' : 'namespace')
    ..aOS(4, _omitFieldNames ? '' : 'publisherEmail')
    ..aInt64(5, _omitFieldNames ? '' : 'publishTimestampSec')
    ..aInt64(6, _omitFieldNames ? '' : 'unpublishedAtTimestampSec')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  PublishInfoResponsePB clone() => PublishInfoResponsePB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  PublishInfoResponsePB copyWith(void Function(PublishInfoResponsePB) updates) => super.copyWith((message) => updates(message as PublishInfoResponsePB)) as PublishInfoResponsePB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishInfoResponsePB create() => PublishInfoResponsePB._();
  PublishInfoResponsePB createEmptyInstance() => create();
  static $pb.PbList<PublishInfoResponsePB> createRepeated() => $pb.PbList<PublishInfoResponsePB>();
  @$core.pragma('dart2js:noInline')
  static PublishInfoResponsePB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PublishInfoResponsePB>(create);
  static PublishInfoResponsePB? _defaultInstance;

  PublishInfoResponsePB_OneOfNamespace whichOneOfNamespace() => _PublishInfoResponsePB_OneOfNamespaceByTag[$_whichOneof(0)]!;
  void clearOneOfNamespace() => clearField($_whichOneof(0));

  PublishInfoResponsePB_OneOfUnpublishedAtTimestampSec whichOneOfUnpublishedAtTimestampSec() => _PublishInfoResponsePB_OneOfUnpublishedAtTimestampSecByTag[$_whichOneof(1)]!;
  void clearOneOfUnpublishedAtTimestampSec() => clearField($_whichOneof(1));

  @$pb.TagNumber(1)
  $core.String get viewId => $_getSZ(0);
  @$pb.TagNumber(1)
  set viewId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasViewId() => $_has(0);
  @$pb.TagNumber(1)
  void clearViewId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get publishName => $_getSZ(1);
  @$pb.TagNumber(2)
  set publishName($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasPublishName() => $_has(1);
  @$pb.TagNumber(2)
  void clearPublishName() => clearField(2);

  @$pb.TagNumber(3)
  $core.String get namespace => $_getSZ(2);
  @$pb.TagNumber(3)
  set namespace($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasNamespace() => $_has(2);
  @$pb.TagNumber(3)
  void clearNamespace() => clearField(3);

  @$pb.TagNumber(4)
  $core.String get publisherEmail => $_getSZ(3);
  @$pb.TagNumber(4)
  set publisherEmail($core.String v) { $_setString(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasPublisherEmail() => $_has(3);
  @$pb.TagNumber(4)
  void clearPublisherEmail() => clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get publishTimestampSec => $_getI64(4);
  @$pb.TagNumber(5)
  set publishTimestampSec($fixnum.Int64 v) { $_setInt64(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasPublishTimestampSec() => $_has(4);
  @$pb.TagNumber(5)
  void clearPublishTimestampSec() => clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get unpublishedAtTimestampSec => $_getI64(5);
  @$pb.TagNumber(6)
  set unpublishedAtTimestampSec($fixnum.Int64 v) { $_setInt64(5, v); }
  @$pb.TagNumber(6)
  $core.bool hasUnpublishedAtTimestampSec() => $_has(5);
  @$pb.TagNumber(6)
  void clearUnpublishedAtTimestampSec() => clearField(6);
}

class RepeatedPublishInfoViewPB extends $pb.GeneratedMessage {
  factory RepeatedPublishInfoViewPB({
    $core.Iterable<PublishInfoViewPB>? items,
  }) {
    final $result = create();
    if (items != null) {
      $result.items.addAll(items);
    }
    return $result;
  }
  RepeatedPublishInfoViewPB._() : super();
  factory RepeatedPublishInfoViewPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RepeatedPublishInfoViewPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RepeatedPublishInfoViewPB', createEmptyInstance: create)
    ..pc<PublishInfoViewPB>(1, _omitFieldNames ? '' : 'items', $pb.PbFieldType.PM, subBuilder: PublishInfoViewPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RepeatedPublishInfoViewPB clone() => RepeatedPublishInfoViewPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RepeatedPublishInfoViewPB copyWith(void Function(RepeatedPublishInfoViewPB) updates) => super.copyWith((message) => updates(message as RepeatedPublishInfoViewPB)) as RepeatedPublishInfoViewPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RepeatedPublishInfoViewPB create() => RepeatedPublishInfoViewPB._();
  RepeatedPublishInfoViewPB createEmptyInstance() => create();
  static $pb.PbList<RepeatedPublishInfoViewPB> createRepeated() => $pb.PbList<RepeatedPublishInfoViewPB>();
  @$core.pragma('dart2js:noInline')
  static RepeatedPublishInfoViewPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RepeatedPublishInfoViewPB>(create);
  static RepeatedPublishInfoViewPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<PublishInfoViewPB> get items => $_getList(0);
}

class SetPublishNamespacePayloadPB extends $pb.GeneratedMessage {
  factory SetPublishNamespacePayloadPB({
    $core.String? newNamespace,
  }) {
    final $result = create();
    if (newNamespace != null) {
      $result.newNamespace = newNamespace;
    }
    return $result;
  }
  SetPublishNamespacePayloadPB._() : super();
  factory SetPublishNamespacePayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SetPublishNamespacePayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetPublishNamespacePayloadPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'newNamespace')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SetPublishNamespacePayloadPB clone() => SetPublishNamespacePayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SetPublishNamespacePayloadPB copyWith(void Function(SetPublishNamespacePayloadPB) updates) => super.copyWith((message) => updates(message as SetPublishNamespacePayloadPB)) as SetPublishNamespacePayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SetPublishNamespacePayloadPB create() => SetPublishNamespacePayloadPB._();
  SetPublishNamespacePayloadPB createEmptyInstance() => create();
  static $pb.PbList<SetPublishNamespacePayloadPB> createRepeated() => $pb.PbList<SetPublishNamespacePayloadPB>();
  @$core.pragma('dart2js:noInline')
  static SetPublishNamespacePayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SetPublishNamespacePayloadPB>(create);
  static SetPublishNamespacePayloadPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get newNamespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set newNamespace($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasNewNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNewNamespace() => clearField(1);
}

class PublishNamespacePB extends $pb.GeneratedMessage {
  factory PublishNamespacePB({
    $core.String? namespace,
  }) {
    final $result = create();
    if (namespace != null) {
      $result.namespace = namespace;
    }
    return $result;
  }
  PublishNamespacePB._() : super();
  factory PublishNamespacePB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory PublishNamespacePB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'PublishNamespacePB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  PublishNamespacePB clone() => PublishNamespacePB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  PublishNamespacePB copyWith(void Function(PublishNamespacePB) updates) => super.copyWith((message) => updates(message as PublishNamespacePB)) as PublishNamespacePB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishNamespacePB create() => PublishNamespacePB._();
  PublishNamespacePB createEmptyInstance() => create();
  static $pb.PbList<PublishNamespacePB> createRepeated() => $pb.PbList<PublishNamespacePB>();
  @$core.pragma('dart2js:noInline')
  static PublishNamespacePB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PublishNamespacePB>(create);
  static PublishNamespacePB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => clearField(1);
}


const _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
