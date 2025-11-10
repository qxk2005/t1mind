//
//  Generated code. Do not modify.
//  source: workspace.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'view.pb.dart' as $1;

class WorkspacePB extends $pb.GeneratedMessage {
  factory WorkspacePB({
    $core.String? id,
    $core.String? name,
    $core.Iterable<$1.ViewPB>? views,
    $fixnum.Int64? createTime,
  }) {
    final $result = create();
    if (id != null) {
      $result.id = id;
    }
    if (name != null) {
      $result.name = name;
    }
    if (views != null) {
      $result.views.addAll(views);
    }
    if (createTime != null) {
      $result.createTime = createTime;
    }
    return $result;
  }
  WorkspacePB._() : super();
  factory WorkspacePB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory WorkspacePB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'WorkspacePB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..pc<$1.ViewPB>(3, _omitFieldNames ? '' : 'views', $pb.PbFieldType.PM, subBuilder: $1.ViewPB.create)
    ..aInt64(4, _omitFieldNames ? '' : 'createTime')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  WorkspacePB clone() => WorkspacePB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  WorkspacePB copyWith(void Function(WorkspacePB) updates) => super.copyWith((message) => updates(message as WorkspacePB)) as WorkspacePB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkspacePB create() => WorkspacePB._();
  WorkspacePB createEmptyInstance() => create();
  static $pb.PbList<WorkspacePB> createRepeated() => $pb.PbList<WorkspacePB>();
  @$core.pragma('dart2js:noInline')
  static WorkspacePB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<WorkspacePB>(create);
  static WorkspacePB? _defaultInstance;

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
  $core.List<$1.ViewPB> get views => $_getList(2);

  @$pb.TagNumber(4)
  $fixnum.Int64 get createTime => $_getI64(3);
  @$pb.TagNumber(4)
  set createTime($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasCreateTime() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreateTime() => clearField(4);
}

class RepeatedWorkspacePB extends $pb.GeneratedMessage {
  factory RepeatedWorkspacePB({
    $core.Iterable<WorkspacePB>? items,
  }) {
    final $result = create();
    if (items != null) {
      $result.items.addAll(items);
    }
    return $result;
  }
  RepeatedWorkspacePB._() : super();
  factory RepeatedWorkspacePB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RepeatedWorkspacePB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RepeatedWorkspacePB', createEmptyInstance: create)
    ..pc<WorkspacePB>(1, _omitFieldNames ? '' : 'items', $pb.PbFieldType.PM, subBuilder: WorkspacePB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RepeatedWorkspacePB clone() => RepeatedWorkspacePB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RepeatedWorkspacePB copyWith(void Function(RepeatedWorkspacePB) updates) => super.copyWith((message) => updates(message as RepeatedWorkspacePB)) as RepeatedWorkspacePB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RepeatedWorkspacePB create() => RepeatedWorkspacePB._();
  RepeatedWorkspacePB createEmptyInstance() => create();
  static $pb.PbList<RepeatedWorkspacePB> createRepeated() => $pb.PbList<RepeatedWorkspacePB>();
  @$core.pragma('dart2js:noInline')
  static RepeatedWorkspacePB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RepeatedWorkspacePB>(create);
  static RepeatedWorkspacePB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<WorkspacePB> get items => $_getList(0);
}

class CreateWorkspacePayloadPB extends $pb.GeneratedMessage {
  factory CreateWorkspacePayloadPB({
    $core.String? name,
    $core.String? desc,
  }) {
    final $result = create();
    if (name != null) {
      $result.name = name;
    }
    if (desc != null) {
      $result.desc = desc;
    }
    return $result;
  }
  CreateWorkspacePayloadPB._() : super();
  factory CreateWorkspacePayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CreateWorkspacePayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CreateWorkspacePayloadPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'desc')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CreateWorkspacePayloadPB clone() => CreateWorkspacePayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CreateWorkspacePayloadPB copyWith(void Function(CreateWorkspacePayloadPB) updates) => super.copyWith((message) => updates(message as CreateWorkspacePayloadPB)) as CreateWorkspacePayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateWorkspacePayloadPB create() => CreateWorkspacePayloadPB._();
  CreateWorkspacePayloadPB createEmptyInstance() => create();
  static $pb.PbList<CreateWorkspacePayloadPB> createRepeated() => $pb.PbList<CreateWorkspacePayloadPB>();
  @$core.pragma('dart2js:noInline')
  static CreateWorkspacePayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CreateWorkspacePayloadPB>(create);
  static CreateWorkspacePayloadPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get desc => $_getSZ(1);
  @$pb.TagNumber(2)
  set desc($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasDesc() => $_has(1);
  @$pb.TagNumber(2)
  void clearDesc() => clearField(2);
}

class WorkspaceIdPB extends $pb.GeneratedMessage {
  factory WorkspaceIdPB({
    $core.String? value,
  }) {
    final $result = create();
    if (value != null) {
      $result.value = value;
    }
    return $result;
  }
  WorkspaceIdPB._() : super();
  factory WorkspaceIdPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory WorkspaceIdPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'WorkspaceIdPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  WorkspaceIdPB clone() => WorkspaceIdPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  WorkspaceIdPB copyWith(void Function(WorkspaceIdPB) updates) => super.copyWith((message) => updates(message as WorkspaceIdPB)) as WorkspaceIdPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkspaceIdPB create() => WorkspaceIdPB._();
  WorkspaceIdPB createEmptyInstance() => create();
  static $pb.PbList<WorkspaceIdPB> createRepeated() => $pb.PbList<WorkspaceIdPB>();
  @$core.pragma('dart2js:noInline')
  static WorkspaceIdPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<WorkspaceIdPB>(create);
  static WorkspaceIdPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get value => $_getSZ(0);
  @$pb.TagNumber(1)
  set value($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => clearField(1);
}

class GetWorkspaceViewPB extends $pb.GeneratedMessage {
  factory GetWorkspaceViewPB({
    $core.String? value,
  }) {
    final $result = create();
    if (value != null) {
      $result.value = value;
    }
    return $result;
  }
  GetWorkspaceViewPB._() : super();
  factory GetWorkspaceViewPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetWorkspaceViewPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'GetWorkspaceViewPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetWorkspaceViewPB clone() => GetWorkspaceViewPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetWorkspaceViewPB copyWith(void Function(GetWorkspaceViewPB) updates) => super.copyWith((message) => updates(message as GetWorkspaceViewPB)) as GetWorkspaceViewPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetWorkspaceViewPB create() => GetWorkspaceViewPB._();
  GetWorkspaceViewPB createEmptyInstance() => create();
  static $pb.PbList<GetWorkspaceViewPB> createRepeated() => $pb.PbList<GetWorkspaceViewPB>();
  @$core.pragma('dart2js:noInline')
  static GetWorkspaceViewPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetWorkspaceViewPB>(create);
  static GetWorkspaceViewPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get value => $_getSZ(0);
  @$pb.TagNumber(1)
  set value($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => clearField(1);
}

enum WorkspaceLatestPB_OneOfLatestView {
  latestView, 
  notSet
}

class WorkspaceLatestPB extends $pb.GeneratedMessage {
  factory WorkspaceLatestPB({
    $core.String? workspaceId,
    $1.ViewPB? latestView,
  }) {
    final $result = create();
    if (workspaceId != null) {
      $result.workspaceId = workspaceId;
    }
    if (latestView != null) {
      $result.latestView = latestView;
    }
    return $result;
  }
  WorkspaceLatestPB._() : super();
  factory WorkspaceLatestPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory WorkspaceLatestPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, WorkspaceLatestPB_OneOfLatestView> _WorkspaceLatestPB_OneOfLatestViewByTag = {
    2 : WorkspaceLatestPB_OneOfLatestView.latestView,
    0 : WorkspaceLatestPB_OneOfLatestView.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'WorkspaceLatestPB', createEmptyInstance: create)
    ..oo(0, [2])
    ..aOS(1, _omitFieldNames ? '' : 'workspaceId')
    ..aOM<$1.ViewPB>(2, _omitFieldNames ? '' : 'latestView', subBuilder: $1.ViewPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  WorkspaceLatestPB clone() => WorkspaceLatestPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  WorkspaceLatestPB copyWith(void Function(WorkspaceLatestPB) updates) => super.copyWith((message) => updates(message as WorkspaceLatestPB)) as WorkspaceLatestPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkspaceLatestPB create() => WorkspaceLatestPB._();
  WorkspaceLatestPB createEmptyInstance() => create();
  static $pb.PbList<WorkspaceLatestPB> createRepeated() => $pb.PbList<WorkspaceLatestPB>();
  @$core.pragma('dart2js:noInline')
  static WorkspaceLatestPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<WorkspaceLatestPB>(create);
  static WorkspaceLatestPB? _defaultInstance;

  WorkspaceLatestPB_OneOfLatestView whichOneOfLatestView() => _WorkspaceLatestPB_OneOfLatestViewByTag[$_whichOneof(0)]!;
  void clearOneOfLatestView() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get workspaceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set workspaceId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasWorkspaceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkspaceId() => clearField(1);

  @$pb.TagNumber(2)
  $1.ViewPB get latestView => $_getN(1);
  @$pb.TagNumber(2)
  set latestView($1.ViewPB v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasLatestView() => $_has(1);
  @$pb.TagNumber(2)
  void clearLatestView() => clearField(2);
  @$pb.TagNumber(2)
  $1.ViewPB ensureLatestView() => $_ensure(1);
}

enum UpdateWorkspacePayloadPB_OneOfName {
  name, 
  notSet
}

enum UpdateWorkspacePayloadPB_OneOfDesc {
  desc, 
  notSet
}

class UpdateWorkspacePayloadPB extends $pb.GeneratedMessage {
  factory UpdateWorkspacePayloadPB({
    $core.String? id,
    $core.String? name,
    $core.String? desc,
  }) {
    final $result = create();
    if (id != null) {
      $result.id = id;
    }
    if (name != null) {
      $result.name = name;
    }
    if (desc != null) {
      $result.desc = desc;
    }
    return $result;
  }
  UpdateWorkspacePayloadPB._() : super();
  factory UpdateWorkspacePayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory UpdateWorkspacePayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, UpdateWorkspacePayloadPB_OneOfName> _UpdateWorkspacePayloadPB_OneOfNameByTag = {
    2 : UpdateWorkspacePayloadPB_OneOfName.name,
    0 : UpdateWorkspacePayloadPB_OneOfName.notSet
  };
  static const $core.Map<$core.int, UpdateWorkspacePayloadPB_OneOfDesc> _UpdateWorkspacePayloadPB_OneOfDescByTag = {
    3 : UpdateWorkspacePayloadPB_OneOfDesc.desc,
    0 : UpdateWorkspacePayloadPB_OneOfDesc.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'UpdateWorkspacePayloadPB', createEmptyInstance: create)
    ..oo(0, [2])
    ..oo(1, [3])
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'desc')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  UpdateWorkspacePayloadPB clone() => UpdateWorkspacePayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  UpdateWorkspacePayloadPB copyWith(void Function(UpdateWorkspacePayloadPB) updates) => super.copyWith((message) => updates(message as UpdateWorkspacePayloadPB)) as UpdateWorkspacePayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateWorkspacePayloadPB create() => UpdateWorkspacePayloadPB._();
  UpdateWorkspacePayloadPB createEmptyInstance() => create();
  static $pb.PbList<UpdateWorkspacePayloadPB> createRepeated() => $pb.PbList<UpdateWorkspacePayloadPB>();
  @$core.pragma('dart2js:noInline')
  static UpdateWorkspacePayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<UpdateWorkspacePayloadPB>(create);
  static UpdateWorkspacePayloadPB? _defaultInstance;

  UpdateWorkspacePayloadPB_OneOfName whichOneOfName() => _UpdateWorkspacePayloadPB_OneOfNameByTag[$_whichOneof(0)]!;
  void clearOneOfName() => clearField($_whichOneof(0));

  UpdateWorkspacePayloadPB_OneOfDesc whichOneOfDesc() => _UpdateWorkspacePayloadPB_OneOfDescByTag[$_whichOneof(1)]!;
  void clearOneOfDesc() => clearField($_whichOneof(1));

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
  $core.String get desc => $_getSZ(2);
  @$pb.TagNumber(3)
  set desc($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasDesc() => $_has(2);
  @$pb.TagNumber(3)
  void clearDesc() => clearField(3);
}

class RepeatedFolderSnapshotPB extends $pb.GeneratedMessage {
  factory RepeatedFolderSnapshotPB({
    $core.Iterable<FolderSnapshotPB>? items,
  }) {
    final $result = create();
    if (items != null) {
      $result.items.addAll(items);
    }
    return $result;
  }
  RepeatedFolderSnapshotPB._() : super();
  factory RepeatedFolderSnapshotPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RepeatedFolderSnapshotPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RepeatedFolderSnapshotPB', createEmptyInstance: create)
    ..pc<FolderSnapshotPB>(1, _omitFieldNames ? '' : 'items', $pb.PbFieldType.PM, subBuilder: FolderSnapshotPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RepeatedFolderSnapshotPB clone() => RepeatedFolderSnapshotPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RepeatedFolderSnapshotPB copyWith(void Function(RepeatedFolderSnapshotPB) updates) => super.copyWith((message) => updates(message as RepeatedFolderSnapshotPB)) as RepeatedFolderSnapshotPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RepeatedFolderSnapshotPB create() => RepeatedFolderSnapshotPB._();
  RepeatedFolderSnapshotPB createEmptyInstance() => create();
  static $pb.PbList<RepeatedFolderSnapshotPB> createRepeated() => $pb.PbList<RepeatedFolderSnapshotPB>();
  @$core.pragma('dart2js:noInline')
  static RepeatedFolderSnapshotPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RepeatedFolderSnapshotPB>(create);
  static RepeatedFolderSnapshotPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<FolderSnapshotPB> get items => $_getList(0);
}

class FolderSnapshotPB extends $pb.GeneratedMessage {
  factory FolderSnapshotPB({
    $fixnum.Int64? snapshotId,
    $core.String? snapshotDesc,
    $fixnum.Int64? createdAt,
    $core.List<$core.int>? data,
  }) {
    final $result = create();
    if (snapshotId != null) {
      $result.snapshotId = snapshotId;
    }
    if (snapshotDesc != null) {
      $result.snapshotDesc = snapshotDesc;
    }
    if (createdAt != null) {
      $result.createdAt = createdAt;
    }
    if (data != null) {
      $result.data = data;
    }
    return $result;
  }
  FolderSnapshotPB._() : super();
  factory FolderSnapshotPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FolderSnapshotPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'FolderSnapshotPB', createEmptyInstance: create)
    ..aInt64(1, _omitFieldNames ? '' : 'snapshotId')
    ..aOS(2, _omitFieldNames ? '' : 'snapshotDesc')
    ..aInt64(3, _omitFieldNames ? '' : 'createdAt')
    ..a<$core.List<$core.int>>(4, _omitFieldNames ? '' : 'data', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FolderSnapshotPB clone() => FolderSnapshotPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FolderSnapshotPB copyWith(void Function(FolderSnapshotPB) updates) => super.copyWith((message) => updates(message as FolderSnapshotPB)) as FolderSnapshotPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FolderSnapshotPB create() => FolderSnapshotPB._();
  FolderSnapshotPB createEmptyInstance() => create();
  static $pb.PbList<FolderSnapshotPB> createRepeated() => $pb.PbList<FolderSnapshotPB>();
  @$core.pragma('dart2js:noInline')
  static FolderSnapshotPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FolderSnapshotPB>(create);
  static FolderSnapshotPB? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get snapshotId => $_getI64(0);
  @$pb.TagNumber(1)
  set snapshotId($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasSnapshotId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSnapshotId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get snapshotDesc => $_getSZ(1);
  @$pb.TagNumber(2)
  set snapshotDesc($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasSnapshotDesc() => $_has(1);
  @$pb.TagNumber(2)
  void clearSnapshotDesc() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get createdAt => $_getI64(2);
  @$pb.TagNumber(3)
  set createdAt($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasCreatedAt() => $_has(2);
  @$pb.TagNumber(3)
  void clearCreatedAt() => clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get data => $_getN(3);
  @$pb.TagNumber(4)
  set data($core.List<$core.int> v) { $_setBytes(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasData() => $_has(3);
  @$pb.TagNumber(4)
  void clearData() => clearField(4);
}

class FolderSnapshotStatePB extends $pb.GeneratedMessage {
  factory FolderSnapshotStatePB({
    $fixnum.Int64? newSnapshotId,
  }) {
    final $result = create();
    if (newSnapshotId != null) {
      $result.newSnapshotId = newSnapshotId;
    }
    return $result;
  }
  FolderSnapshotStatePB._() : super();
  factory FolderSnapshotStatePB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FolderSnapshotStatePB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'FolderSnapshotStatePB', createEmptyInstance: create)
    ..aInt64(1, _omitFieldNames ? '' : 'newSnapshotId')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FolderSnapshotStatePB clone() => FolderSnapshotStatePB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FolderSnapshotStatePB copyWith(void Function(FolderSnapshotStatePB) updates) => super.copyWith((message) => updates(message as FolderSnapshotStatePB)) as FolderSnapshotStatePB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FolderSnapshotStatePB create() => FolderSnapshotStatePB._();
  FolderSnapshotStatePB createEmptyInstance() => create();
  static $pb.PbList<FolderSnapshotStatePB> createRepeated() => $pb.PbList<FolderSnapshotStatePB>();
  @$core.pragma('dart2js:noInline')
  static FolderSnapshotStatePB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FolderSnapshotStatePB>(create);
  static FolderSnapshotStatePB? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get newSnapshotId => $_getI64(0);
  @$pb.TagNumber(1)
  set newSnapshotId($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasNewSnapshotId() => $_has(0);
  @$pb.TagNumber(1)
  void clearNewSnapshotId() => clearField(1);
}

class FolderSyncStatePB extends $pb.GeneratedMessage {
  factory FolderSyncStatePB({
    $core.bool? isSyncing,
    $core.bool? isFinish,
  }) {
    final $result = create();
    if (isSyncing != null) {
      $result.isSyncing = isSyncing;
    }
    if (isFinish != null) {
      $result.isFinish = isFinish;
    }
    return $result;
  }
  FolderSyncStatePB._() : super();
  factory FolderSyncStatePB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FolderSyncStatePB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'FolderSyncStatePB', createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'isSyncing')
    ..aOB(2, _omitFieldNames ? '' : 'isFinish')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FolderSyncStatePB clone() => FolderSyncStatePB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FolderSyncStatePB copyWith(void Function(FolderSyncStatePB) updates) => super.copyWith((message) => updates(message as FolderSyncStatePB)) as FolderSyncStatePB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FolderSyncStatePB create() => FolderSyncStatePB._();
  FolderSyncStatePB createEmptyInstance() => create();
  static $pb.PbList<FolderSyncStatePB> createRepeated() => $pb.PbList<FolderSyncStatePB>();
  @$core.pragma('dart2js:noInline')
  static FolderSyncStatePB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FolderSyncStatePB>(create);
  static FolderSyncStatePB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get isSyncing => $_getBF(0);
  @$pb.TagNumber(1)
  set isSyncing($core.bool v) { $_setBool(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasIsSyncing() => $_has(0);
  @$pb.TagNumber(1)
  void clearIsSyncing() => clearField(1);

  @$pb.TagNumber(2)
  $core.bool get isFinish => $_getBF(1);
  @$pb.TagNumber(2)
  set isFinish($core.bool v) { $_setBool(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasIsFinish() => $_has(1);
  @$pb.TagNumber(2)
  void clearIsFinish() => clearField(2);
}

class UserFolderPB extends $pb.GeneratedMessage {
  factory UserFolderPB({
    $fixnum.Int64? uid,
    $core.String? workspaceId,
  }) {
    final $result = create();
    if (uid != null) {
      $result.uid = uid;
    }
    if (workspaceId != null) {
      $result.workspaceId = workspaceId;
    }
    return $result;
  }
  UserFolderPB._() : super();
  factory UserFolderPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory UserFolderPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'UserFolderPB', createEmptyInstance: create)
    ..aInt64(1, _omitFieldNames ? '' : 'uid')
    ..aOS(2, _omitFieldNames ? '' : 'workspaceId')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  UserFolderPB clone() => UserFolderPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  UserFolderPB copyWith(void Function(UserFolderPB) updates) => super.copyWith((message) => updates(message as UserFolderPB)) as UserFolderPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UserFolderPB create() => UserFolderPB._();
  UserFolderPB createEmptyInstance() => create();
  static $pb.PbList<UserFolderPB> createRepeated() => $pb.PbList<UserFolderPB>();
  @$core.pragma('dart2js:noInline')
  static UserFolderPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<UserFolderPB>(create);
  static UserFolderPB? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get uid => $_getI64(0);
  @$pb.TagNumber(1)
  set uid($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasUid() => $_has(0);
  @$pb.TagNumber(1)
  void clearUid() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get workspaceId => $_getSZ(1);
  @$pb.TagNumber(2)
  set workspaceId($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasWorkspaceId() => $_has(1);
  @$pb.TagNumber(2)
  void clearWorkspaceId() => clearField(2);
}


const _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
