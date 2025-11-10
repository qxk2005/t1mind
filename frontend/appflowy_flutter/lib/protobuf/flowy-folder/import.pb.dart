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

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'import.pbenum.dart';
import 'view.pbenum.dart' as $0;

export 'import.pbenum.dart';

enum ImportItemPayloadPB_OneOfData {
  data, 
  notSet
}

enum ImportItemPayloadPB_OneOfFilePath {
  filePath, 
  notSet
}

enum ImportItemPayloadPB_OneOfViewId {
  viewId, 
  notSet
}

class ImportItemPayloadPB extends $pb.GeneratedMessage {
  factory ImportItemPayloadPB({
    $core.String? name,
    $core.List<$core.int>? data,
    $core.String? filePath,
    $0.ViewLayoutPB? viewLayout,
    ImportTypePB? importType,
    $core.String? viewId,
  }) {
    final $result = create();
    if (name != null) {
      $result.name = name;
    }
    if (data != null) {
      $result.data = data;
    }
    if (filePath != null) {
      $result.filePath = filePath;
    }
    if (viewLayout != null) {
      $result.viewLayout = viewLayout;
    }
    if (importType != null) {
      $result.importType = importType;
    }
    if (viewId != null) {
      $result.viewId = viewId;
    }
    return $result;
  }
  ImportItemPayloadPB._() : super();
  factory ImportItemPayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ImportItemPayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, ImportItemPayloadPB_OneOfData> _ImportItemPayloadPB_OneOfDataByTag = {
    2 : ImportItemPayloadPB_OneOfData.data,
    0 : ImportItemPayloadPB_OneOfData.notSet
  };
  static const $core.Map<$core.int, ImportItemPayloadPB_OneOfFilePath> _ImportItemPayloadPB_OneOfFilePathByTag = {
    3 : ImportItemPayloadPB_OneOfFilePath.filePath,
    0 : ImportItemPayloadPB_OneOfFilePath.notSet
  };
  static const $core.Map<$core.int, ImportItemPayloadPB_OneOfViewId> _ImportItemPayloadPB_OneOfViewIdByTag = {
    6 : ImportItemPayloadPB_OneOfViewId.viewId,
    0 : ImportItemPayloadPB_OneOfViewId.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ImportItemPayloadPB', createEmptyInstance: create)
    ..oo(0, [2])
    ..oo(1, [3])
    ..oo(2, [6])
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..a<$core.List<$core.int>>(2, _omitFieldNames ? '' : 'data', $pb.PbFieldType.OY)
    ..aOS(3, _omitFieldNames ? '' : 'filePath')
    ..e<$0.ViewLayoutPB>(4, _omitFieldNames ? '' : 'viewLayout', $pb.PbFieldType.OE, defaultOrMaker: $0.ViewLayoutPB.Document, valueOf: $0.ViewLayoutPB.valueOf, enumValues: $0.ViewLayoutPB.values)
    ..e<ImportTypePB>(5, _omitFieldNames ? '' : 'importType', $pb.PbFieldType.OE, defaultOrMaker: ImportTypePB.HistoryDocument, valueOf: ImportTypePB.valueOf, enumValues: ImportTypePB.values)
    ..aOS(6, _omitFieldNames ? '' : 'viewId')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ImportItemPayloadPB clone() => ImportItemPayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ImportItemPayloadPB copyWith(void Function(ImportItemPayloadPB) updates) => super.copyWith((message) => updates(message as ImportItemPayloadPB)) as ImportItemPayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportItemPayloadPB create() => ImportItemPayloadPB._();
  ImportItemPayloadPB createEmptyInstance() => create();
  static $pb.PbList<ImportItemPayloadPB> createRepeated() => $pb.PbList<ImportItemPayloadPB>();
  @$core.pragma('dart2js:noInline')
  static ImportItemPayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ImportItemPayloadPB>(create);
  static ImportItemPayloadPB? _defaultInstance;

  ImportItemPayloadPB_OneOfData whichOneOfData() => _ImportItemPayloadPB_OneOfDataByTag[$_whichOneof(0)]!;
  void clearOneOfData() => clearField($_whichOneof(0));

  ImportItemPayloadPB_OneOfFilePath whichOneOfFilePath() => _ImportItemPayloadPB_OneOfFilePathByTag[$_whichOneof(1)]!;
  void clearOneOfFilePath() => clearField($_whichOneof(1));

  ImportItemPayloadPB_OneOfViewId whichOneOfViewId() => _ImportItemPayloadPB_OneOfViewIdByTag[$_whichOneof(2)]!;
  void clearOneOfViewId() => clearField($_whichOneof(2));

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($core.List<$core.int> v) { $_setBytes(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => clearField(2);

  @$pb.TagNumber(3)
  $core.String get filePath => $_getSZ(2);
  @$pb.TagNumber(3)
  set filePath($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasFilePath() => $_has(2);
  @$pb.TagNumber(3)
  void clearFilePath() => clearField(3);

  @$pb.TagNumber(4)
  $0.ViewLayoutPB get viewLayout => $_getN(3);
  @$pb.TagNumber(4)
  set viewLayout($0.ViewLayoutPB v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasViewLayout() => $_has(3);
  @$pb.TagNumber(4)
  void clearViewLayout() => clearField(4);

  @$pb.TagNumber(5)
  ImportTypePB get importType => $_getN(4);
  @$pb.TagNumber(5)
  set importType(ImportTypePB v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasImportType() => $_has(4);
  @$pb.TagNumber(5)
  void clearImportType() => clearField(5);

  @$pb.TagNumber(6)
  $core.String get viewId => $_getSZ(5);
  @$pb.TagNumber(6)
  set viewId($core.String v) { $_setString(5, v); }
  @$pb.TagNumber(6)
  $core.bool hasViewId() => $_has(5);
  @$pb.TagNumber(6)
  void clearViewId() => clearField(6);
}

class ImportPayloadPB extends $pb.GeneratedMessage {
  factory ImportPayloadPB({
    $core.String? parentViewId,
    $core.Iterable<ImportItemPayloadPB>? items,
  }) {
    final $result = create();
    if (parentViewId != null) {
      $result.parentViewId = parentViewId;
    }
    if (items != null) {
      $result.items.addAll(items);
    }
    return $result;
  }
  ImportPayloadPB._() : super();
  factory ImportPayloadPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ImportPayloadPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ImportPayloadPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'parentViewId')
    ..pc<ImportItemPayloadPB>(2, _omitFieldNames ? '' : 'items', $pb.PbFieldType.PM, subBuilder: ImportItemPayloadPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ImportPayloadPB clone() => ImportPayloadPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ImportPayloadPB copyWith(void Function(ImportPayloadPB) updates) => super.copyWith((message) => updates(message as ImportPayloadPB)) as ImportPayloadPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportPayloadPB create() => ImportPayloadPB._();
  ImportPayloadPB createEmptyInstance() => create();
  static $pb.PbList<ImportPayloadPB> createRepeated() => $pb.PbList<ImportPayloadPB>();
  @$core.pragma('dart2js:noInline')
  static ImportPayloadPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ImportPayloadPB>(create);
  static ImportPayloadPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get parentViewId => $_getSZ(0);
  @$pb.TagNumber(1)
  set parentViewId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasParentViewId() => $_has(0);
  @$pb.TagNumber(1)
  void clearParentViewId() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<ImportItemPayloadPB> get items => $_getList(1);
}

class ImportZipPB extends $pb.GeneratedMessage {
  factory ImportZipPB({
    $core.String? filePath,
  }) {
    final $result = create();
    if (filePath != null) {
      $result.filePath = filePath;
    }
    return $result;
  }
  ImportZipPB._() : super();
  factory ImportZipPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ImportZipPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ImportZipPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'filePath')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ImportZipPB clone() => ImportZipPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ImportZipPB copyWith(void Function(ImportZipPB) updates) => super.copyWith((message) => updates(message as ImportZipPB)) as ImportZipPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportZipPB create() => ImportZipPB._();
  ImportZipPB createEmptyInstance() => create();
  static $pb.PbList<ImportZipPB> createRepeated() => $pb.PbList<ImportZipPB>();
  @$core.pragma('dart2js:noInline')
  static ImportZipPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ImportZipPB>(create);
  static ImportZipPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get filePath => $_getSZ(0);
  @$pb.TagNumber(1)
  set filePath($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasFilePath() => $_has(0);
  @$pb.TagNumber(1)
  void clearFilePath() => clearField(1);
}

class RegisterImportProgressStreamPB extends $pb.GeneratedMessage {
  factory RegisterImportProgressStreamPB({
    $fixnum.Int64? port,
  }) {
    final $result = create();
    if (port != null) {
      $result.port = port;
    }
    return $result;
  }
  RegisterImportProgressStreamPB._() : super();
  factory RegisterImportProgressStreamPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RegisterImportProgressStreamPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RegisterImportProgressStreamPB', createEmptyInstance: create)
    ..aInt64(1, _omitFieldNames ? '' : 'port')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RegisterImportProgressStreamPB clone() => RegisterImportProgressStreamPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RegisterImportProgressStreamPB copyWith(void Function(RegisterImportProgressStreamPB) updates) => super.copyWith((message) => updates(message as RegisterImportProgressStreamPB)) as RegisterImportProgressStreamPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterImportProgressStreamPB create() => RegisterImportProgressStreamPB._();
  RegisterImportProgressStreamPB createEmptyInstance() => create();
  static $pb.PbList<RegisterImportProgressStreamPB> createRepeated() => $pb.PbList<RegisterImportProgressStreamPB>();
  @$core.pragma('dart2js:noInline')
  static RegisterImportProgressStreamPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RegisterImportProgressStreamPB>(create);
  static RegisterImportProgressStreamPB? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get port => $_getI64(0);
  @$pb.TagNumber(1)
  set port($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasPort() => $_has(0);
  @$pb.TagNumber(1)
  void clearPort() => clearField(1);
}

class GetImportProgressPB extends $pb.GeneratedMessage {
  factory GetImportProgressPB({
    $core.String? importId,
  }) {
    final $result = create();
    if (importId != null) {
      $result.importId = importId;
    }
    return $result;
  }
  GetImportProgressPB._() : super();
  factory GetImportProgressPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetImportProgressPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'GetImportProgressPB', createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'importId')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetImportProgressPB clone() => GetImportProgressPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetImportProgressPB copyWith(void Function(GetImportProgressPB) updates) => super.copyWith((message) => updates(message as GetImportProgressPB)) as GetImportProgressPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetImportProgressPB create() => GetImportProgressPB._();
  GetImportProgressPB createEmptyInstance() => create();
  static $pb.PbList<GetImportProgressPB> createRepeated() => $pb.PbList<GetImportProgressPB>();
  @$core.pragma('dart2js:noInline')
  static GetImportProgressPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetImportProgressPB>(create);
  static GetImportProgressPB? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get importId => $_getSZ(0);
  @$pb.TagNumber(1)
  set importId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasImportId() => $_has(0);
  @$pb.TagNumber(1)
  void clearImportId() => clearField(1);
}

class ImportLogEntryPB extends $pb.GeneratedMessage {
  factory ImportLogEntryPB({
    $fixnum.Int64? timestamp,
    $core.String? level,
    $core.String? message,
  }) {
    final $result = create();
    if (timestamp != null) {
      $result.timestamp = timestamp;
    }
    if (level != null) {
      $result.level = level;
    }
    if (message != null) {
      $result.message = message;
    }
    return $result;
  }
  ImportLogEntryPB._() : super();
  factory ImportLogEntryPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ImportLogEntryPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ImportLogEntryPB', createEmptyInstance: create)
    ..aInt64(1, _omitFieldNames ? '' : 'timestamp')
    ..aOS(2, _omitFieldNames ? '' : 'level')
    ..aOS(3, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ImportLogEntryPB clone() => ImportLogEntryPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ImportLogEntryPB copyWith(void Function(ImportLogEntryPB) updates) => super.copyWith((message) => updates(message as ImportLogEntryPB)) as ImportLogEntryPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportLogEntryPB create() => ImportLogEntryPB._();
  ImportLogEntryPB createEmptyInstance() => create();
  static $pb.PbList<ImportLogEntryPB> createRepeated() => $pb.PbList<ImportLogEntryPB>();
  @$core.pragma('dart2js:noInline')
  static ImportLogEntryPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ImportLogEntryPB>(create);
  static ImportLogEntryPB? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get timestamp => $_getI64(0);
  @$pb.TagNumber(1)
  set timestamp($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasTimestamp() => $_has(0);
  @$pb.TagNumber(1)
  void clearTimestamp() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get level => $_getSZ(1);
  @$pb.TagNumber(2)
  set level($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasLevel() => $_has(1);
  @$pb.TagNumber(2)
  void clearLevel() => clearField(2);

  @$pb.TagNumber(3)
  $core.String get message => $_getSZ(2);
  @$pb.TagNumber(3)
  set message($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasMessage() => $_has(2);
  @$pb.TagNumber(3)
  void clearMessage() => clearField(3);
}

enum ImportProgressPB_OneOfError {
  error, 
  notSet
}

class ImportProgressPB extends $pb.GeneratedMessage {
  factory ImportProgressPB({
    $core.String? importId,
    $core.String? fileName,
    $core.double? progress,
    $core.String? currentStep,
    $core.String? error,
    $core.Iterable<ImportLogEntryPB>? logs,
  }) {
    final $result = create();
    if (importId != null) {
      $result.importId = importId;
    }
    if (fileName != null) {
      $result.fileName = fileName;
    }
    if (progress != null) {
      $result.progress = progress;
    }
    if (currentStep != null) {
      $result.currentStep = currentStep;
    }
    if (error != null) {
      $result.error = error;
    }
    if (logs != null) {
      $result.logs.addAll(logs);
    }
    return $result;
  }
  ImportProgressPB._() : super();
  factory ImportProgressPB.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ImportProgressPB.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static const $core.Map<$core.int, ImportProgressPB_OneOfError> _ImportProgressPB_OneOfErrorByTag = {
    5 : ImportProgressPB_OneOfError.error,
    0 : ImportProgressPB_OneOfError.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ImportProgressPB', createEmptyInstance: create)
    ..oo(0, [5])
    ..aOS(1, _omitFieldNames ? '' : 'importId')
    ..aOS(2, _omitFieldNames ? '' : 'fileName')
    ..a<$core.double>(3, _omitFieldNames ? '' : 'progress', $pb.PbFieldType.OD)
    ..aOS(4, _omitFieldNames ? '' : 'currentStep')
    ..aOS(5, _omitFieldNames ? '' : 'error')
    ..pc<ImportLogEntryPB>(6, _omitFieldNames ? '' : 'logs', $pb.PbFieldType.PM, subBuilder: ImportLogEntryPB.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ImportProgressPB clone() => ImportProgressPB()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ImportProgressPB copyWith(void Function(ImportProgressPB) updates) => super.copyWith((message) => updates(message as ImportProgressPB)) as ImportProgressPB;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportProgressPB create() => ImportProgressPB._();
  ImportProgressPB createEmptyInstance() => create();
  static $pb.PbList<ImportProgressPB> createRepeated() => $pb.PbList<ImportProgressPB>();
  @$core.pragma('dart2js:noInline')
  static ImportProgressPB getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ImportProgressPB>(create);
  static ImportProgressPB? _defaultInstance;

  ImportProgressPB_OneOfError whichOneOfError() => _ImportProgressPB_OneOfErrorByTag[$_whichOneof(0)]!;
  void clearOneOfError() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get importId => $_getSZ(0);
  @$pb.TagNumber(1)
  set importId($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasImportId() => $_has(0);
  @$pb.TagNumber(1)
  void clearImportId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get fileName => $_getSZ(1);
  @$pb.TagNumber(2)
  set fileName($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasFileName() => $_has(1);
  @$pb.TagNumber(2)
  void clearFileName() => clearField(2);

  @$pb.TagNumber(3)
  $core.double get progress => $_getN(2);
  @$pb.TagNumber(3)
  set progress($core.double v) { $_setDouble(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasProgress() => $_has(2);
  @$pb.TagNumber(3)
  void clearProgress() => clearField(3);

  @$pb.TagNumber(4)
  $core.String get currentStep => $_getSZ(3);
  @$pb.TagNumber(4)
  set currentStep($core.String v) { $_setString(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasCurrentStep() => $_has(3);
  @$pb.TagNumber(4)
  void clearCurrentStep() => clearField(4);

  @$pb.TagNumber(5)
  $core.String get error => $_getSZ(4);
  @$pb.TagNumber(5)
  set error($core.String v) { $_setString(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasError() => $_has(4);
  @$pb.TagNumber(5)
  void clearError() => clearField(5);

  @$pb.TagNumber(6)
  $core.List<ImportLogEntryPB> get logs => $_getList(5);
}


const _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
