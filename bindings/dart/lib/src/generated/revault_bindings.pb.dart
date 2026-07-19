// This is a generated file - do not edit.
//
// Generated from revault_bindings.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: public_member_api_docs

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'revault_bindings.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'revault_bindings.pbenum.dart';

/// Metadata for one entry in a lockbox filesystem.
class LockboxEntry extends $pb.GeneratedMessage {
  factory LockboxEntry({
    $core.String? path,
    LockboxEntry_Kind? kind,
    $fixnum.Int64? length,
    $core.int? permissions,
  }) {
    final result = create();
    if (path != null) result.path = path;
    if (kind != null) result.kind = kind;
    if (length != null) result.length = length;
    if (permissions != null) result.permissions = permissions;
    return result;
  }

  LockboxEntry._();

  factory LockboxEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory LockboxEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LockboxEntry',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'path')
    ..aE<LockboxEntry_Kind>(2, _omitFieldNames ? '' : 'kind',
        enumValues: LockboxEntry_Kind.values)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'length', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aI(4, _omitFieldNames ? '' : 'permissions',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LockboxEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LockboxEntry copyWith(void Function(LockboxEntry) updates) =>
      super.copyWith((message) => updates(message as LockboxEntry))
          as LockboxEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static LockboxEntry create() => LockboxEntry._();
  @$core.override
  LockboxEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static LockboxEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LockboxEntry>(create);
  static LockboxEntry? _defaultInstance;

  /// Normalized absolute path within the lockbox.
  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => $_clearField(1);

  /// Type of the entry.
  @$pb.TagNumber(2)
  LockboxEntry_Kind get kind => $_getN(1);
  @$pb.TagNumber(2)
  set kind(LockboxEntry_Kind value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  /// Logical file length in bytes; zero for directories.
  @$pb.TagNumber(3)
  $fixnum.Int64 get length => $_getI64(2);
  @$pb.TagNumber(3)
  set length($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLength() => $_has(2);
  @$pb.TagNumber(3)
  void clearLength() => $_clearField(3);

  /// Stored Unix permission bits.
  @$pb.TagNumber(4)
  $core.int get permissions => $_getIZ(3);
  @$pb.TagNumber(4)
  set permissions($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPermissions() => $_has(3);
  @$pb.TagNumber(4)
  void clearPermissions() => $_clearField(4);
}

/// A collection of lockbox entries.
class LockboxEntryList extends $pb.GeneratedMessage {
  factory LockboxEntryList({
    $core.Iterable<LockboxEntry>? entries,
  }) {
    final result = create();
    if (entries != null) result.entries.addAll(entries);
    return result;
  }

  LockboxEntryList._();

  factory LockboxEntryList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory LockboxEntryList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LockboxEntryList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<LockboxEntry>(1, _omitFieldNames ? '' : 'entries',
        subBuilder: LockboxEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LockboxEntryList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LockboxEntryList copyWith(void Function(LockboxEntryList) updates) =>
      super.copyWith((message) => updates(message as LockboxEntryList))
          as LockboxEntryList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static LockboxEntryList create() => LockboxEntryList._();
  @$core.override
  LockboxEntryList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static LockboxEntryList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LockboxEntryList>(create);
  static LockboxEntryList? _defaultInstance;

  /// Entries in the order returned by the requested listing operation.
  @$pb.TagNumber(1)
  $pb.PbList<LockboxEntry> get entries => $_getList(0);
}

/// Optional entry; `value` is absent when a path does not exist.
class OptionalLockboxEntry extends $pb.GeneratedMessage {
  factory OptionalLockboxEntry({
    LockboxEntry? value,
  }) {
    final result = create();
    if (value != null) result.value = value;
    return result;
  }

  OptionalLockboxEntry._();

  factory OptionalLockboxEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OptionalLockboxEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OptionalLockboxEntry',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOM<LockboxEntry>(1, _omitFieldNames ? '' : 'value',
        subBuilder: LockboxEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalLockboxEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalLockboxEntry copyWith(void Function(OptionalLockboxEntry) updates) =>
      super.copyWith((message) => updates(message as OptionalLockboxEntry))
          as OptionalLockboxEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OptionalLockboxEntry create() => OptionalLockboxEntry._();
  @$core.override
  OptionalLockboxEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OptionalLockboxEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OptionalLockboxEntry>(create);
  static OptionalLockboxEntry? _defaultInstance;

  /// Present entry value.
  @$pb.TagNumber(1)
  LockboxEntry get value => $_getN(0);
  @$pb.TagNumber(1)
  set value(LockboxEntry value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => $_clearField(1);
  @$pb.TagNumber(1)
  LockboxEntry ensureValue() => $_ensure(0);
}

/// A collection of UTF-8 strings.
class StringList extends $pb.GeneratedMessage {
  factory StringList({
    $core.Iterable<$core.String>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  StringList._();

  factory StringList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StringList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StringList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'values')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringList copyWith(void Function(StringList) updates) =>
      super.copyWith((message) => updates(message as StringList)) as StringList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StringList create() => StringList._();
  @$core.override
  StringList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StringList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StringList>(create);
  static StringList? _defaultInstance;

  /// String values in API-defined order.
  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get values => $_getList(0);
}

/// One atomic source-to-destination rename.
class PathMove extends $pb.GeneratedMessage {
  factory PathMove({
    $core.String? source,
    $core.String? destination,
  }) {
    final result = create();
    if (source != null) result.source = source;
    if (destination != null) result.destination = destination;
    return result;
  }

  PathMove._();

  factory PathMove.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PathMove.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PathMove',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'source')
    ..aOS(2, _omitFieldNames ? '' : 'destination')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PathMove clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PathMove copyWith(void Function(PathMove) updates) =>
      super.copyWith((message) => updates(message as PathMove)) as PathMove;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PathMove create() => PathMove._();
  @$core.override
  PathMove createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PathMove getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PathMove>(create);
  static PathMove? _defaultInstance;

  /// Existing variable name or form-record path.
  @$pb.TagNumber(1)
  $core.String get source => $_getSZ(0);
  @$pb.TagNumber(1)
  set source($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSource() => $_has(0);
  @$pb.TagNumber(1)
  void clearSource() => $_clearField(1);

  /// New variable name or form-record path.
  @$pb.TagNumber(2)
  $core.String get destination => $_getSZ(1);
  @$pb.TagNumber(2)
  set destination($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDestination() => $_has(1);
  @$pb.TagNumber(2)
  void clearDestination() => $_clearField(2);
}

/// Renames applied as one operation.
class PathMoveList extends $pb.GeneratedMessage {
  factory PathMoveList({
    $core.Iterable<PathMove>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  PathMoveList._();

  factory PathMoveList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PathMoveList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PathMoveList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<PathMove>(1, _omitFieldNames ? '' : 'values',
        subBuilder: PathMove.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PathMoveList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PathMoveList copyWith(void Function(PathMoveList) updates) =>
      super.copyWith((message) => updates(message as PathMoveList))
          as PathMoveList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PathMoveList create() => PathMoveList._();
  @$core.override
  PathMoveList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PathMoveList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PathMoveList>(create);
  static PathMoveList? _defaultInstance;

  /// Requested source-to-destination moves.
  @$pb.TagNumber(1)
  $pb.PbList<PathMove> get values => $_getList(0);
}

/// A collection of arbitrary byte strings.
class ByteList extends $pb.GeneratedMessage {
  factory ByteList({
    $core.Iterable<$core.List<$core.int>>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  ByteList._();

  factory ByteList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ByteList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ByteList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..p<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'values', $pb.PbFieldType.PY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ByteList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ByteList copyWith(void Function(ByteList) updates) =>
      super.copyWith((message) => updates(message as ByteList)) as ByteList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ByteList create() => ByteList._();
  @$core.override
  ByteList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ByteList getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ByteList>(create);
  static ByteList? _defaultInstance;

  /// Binary values in API-defined order.
  @$pb.TagNumber(1)
  $pb.PbList<$core.List<$core.int>> get values => $_getList(0);
}

/// One field in a typed form definition.
class FormField extends $pb.GeneratedMessage {
  factory FormField({
    $core.String? id,
    $core.String? label,
    $core.String? kind,
    $core.bool? required,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (label != null) result.label = label;
    if (kind != null) result.kind = kind;
    if (required != null) result.required = required;
    return result;
  }

  FormField._();

  factory FormField.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormField.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormField',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'label')
    ..aOS(3, _omitFieldNames ? '' : 'kind')
    ..aOB(4, _omitFieldNames ? '' : 'required')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormField clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormField copyWith(void Function(FormField) updates) =>
      super.copyWith((message) => updates(message as FormField)) as FormField;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormField create() => FormField._();
  @$core.override
  FormField createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormField getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FormField>(create);
  static FormField? _defaultInstance;

  /// Stable field identifier used by records.
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// User-facing label captured in records.
  @$pb.TagNumber(2)
  $core.String get label => $_getSZ(1);
  @$pb.TagNumber(2)
  set label($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLabel() => $_has(1);
  @$pb.TagNumber(2)
  void clearLabel() => $_clearField(2);

  /// Field kind, such as `text`, `password`, or `secret`.
  @$pb.TagNumber(3)
  $core.String get kind => $_getSZ(2);
  @$pb.TagNumber(3)
  set kind($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);

  /// Whether records must provide a value.
  @$pb.TagNumber(4)
  $core.bool get required => $_getBF(3);
  @$pb.TagNumber(4)
  set required($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRequired() => $_has(3);
  @$pb.TagNumber(4)
  void clearRequired() => $_clearField(4);
}

/// A collection of form fields.
class FormFieldList extends $pb.GeneratedMessage {
  factory FormFieldList({
    $core.Iterable<FormField>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  FormFieldList._();

  factory FormFieldList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormFieldList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormFieldList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<FormField>(1, _omitFieldNames ? '' : 'values',
        subBuilder: FormField.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormFieldList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormFieldList copyWith(void Function(FormFieldList) updates) =>
      super.copyWith((message) => updates(message as FormFieldList))
          as FormFieldList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormFieldList create() => FormFieldList._();
  @$core.override
  FormFieldList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormFieldList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FormFieldList>(create);
  static FormFieldList? _defaultInstance;

  /// Ordered form fields.
  @$pb.TagNumber(1)
  $pb.PbList<FormField> get values => $_getList(0);
}

/// One immutable revision of a typed form definition.
class FormDefinition extends $pb.GeneratedMessage {
  factory FormDefinition({
    $core.String? typeId,
    $core.String? alias,
    $core.int? revision,
    $core.String? name,
    $core.String? description,
    $core.Iterable<FormField>? fields,
  }) {
    final result = create();
    if (typeId != null) result.typeId = typeId;
    if (alias != null) result.alias = alias;
    if (revision != null) result.revision = revision;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (fields != null) result.fields.addAll(fields);
    return result;
  }

  FormDefinition._();

  factory FormDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormDefinition',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'typeId')
    ..aOS(2, _omitFieldNames ? '' : 'alias')
    ..aI(3, _omitFieldNames ? '' : 'revision', fieldType: $pb.PbFieldType.OU3)
    ..aOS(4, _omitFieldNames ? '' : 'name')
    ..aOS(5, _omitFieldNames ? '' : 'description')
    ..pPM<FormField>(6, _omitFieldNames ? '' : 'fields',
        subBuilder: FormField.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormDefinition copyWith(void Function(FormDefinition) updates) =>
      super.copyWith((message) => updates(message as FormDefinition))
          as FormDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormDefinition create() => FormDefinition._();
  @$core.override
  FormDefinition createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FormDefinition>(create);
  static FormDefinition? _defaultInstance;

  /// Stable type identifier shared by all revisions.
  @$pb.TagNumber(1)
  $core.String get typeId => $_getSZ(0);
  @$pb.TagNumber(1)
  set typeId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTypeId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTypeId() => $_clearField(1);

  /// Human-readable reference used to resolve the current revision.
  @$pb.TagNumber(2)
  $core.String get alias => $_getSZ(1);
  @$pb.TagNumber(2)
  set alias($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAlias() => $_has(1);
  @$pb.TagNumber(2)
  void clearAlias() => $_clearField(2);

  /// Monotonically increasing definition revision.
  @$pb.TagNumber(3)
  $core.int get revision => $_getIZ(2);
  @$pb.TagNumber(3)
  set revision($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRevision() => $_has(2);
  @$pb.TagNumber(3)
  void clearRevision() => $_clearField(3);

  /// Display name.
  @$pb.TagNumber(4)
  $core.String get name => $_getSZ(3);
  @$pb.TagNumber(4)
  set name($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasName() => $_has(3);
  @$pb.TagNumber(4)
  void clearName() => $_clearField(4);

  /// Optional user-facing description.
  @$pb.TagNumber(5)
  $core.String get description => $_getSZ(4);
  @$pb.TagNumber(5)
  set description($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDescription() => $_has(4);
  @$pb.TagNumber(5)
  void clearDescription() => $_clearField(5);

  /// Ordered fields belonging to the definition.
  @$pb.TagNumber(6)
  $pb.PbList<FormField> get fields => $_getList(5);
}

/// A collection of form definitions.
class FormDefinitionList extends $pb.GeneratedMessage {
  factory FormDefinitionList({
    $core.Iterable<FormDefinition>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  FormDefinitionList._();

  factory FormDefinitionList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormDefinitionList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormDefinitionList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<FormDefinition>(1, _omitFieldNames ? '' : 'values',
        subBuilder: FormDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormDefinitionList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormDefinitionList copyWith(void Function(FormDefinitionList) updates) =>
      super.copyWith((message) => updates(message as FormDefinitionList))
          as FormDefinitionList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormDefinitionList create() => FormDefinitionList._();
  @$core.override
  FormDefinitionList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormDefinitionList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FormDefinitionList>(create);
  static FormDefinitionList? _defaultInstance;

  /// Definitions in revision or alias order, as specified by the operation.
  @$pb.TagNumber(1)
  $pb.PbList<FormDefinition> get values => $_getList(0);
}

/// One value captured in a typed form record.
class FormValue extends $pb.GeneratedMessage {
  factory FormValue({
    $core.String? fieldId,
    $core.String? label,
    $core.String? kind,
    $core.String? value,
    $core.bool? secret,
  }) {
    final result = create();
    if (fieldId != null) result.fieldId = fieldId;
    if (label != null) result.label = label;
    if (kind != null) result.kind = kind;
    if (value != null) result.value = value;
    if (secret != null) result.secret = secret;
    return result;
  }

  FormValue._();

  factory FormValue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormValue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormValue',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'fieldId')
    ..aOS(2, _omitFieldNames ? '' : 'label')
    ..aOS(3, _omitFieldNames ? '' : 'kind')
    ..aOS(4, _omitFieldNames ? '' : 'value')
    ..aOB(5, _omitFieldNames ? '' : 'secret')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormValue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormValue copyWith(void Function(FormValue) updates) =>
      super.copyWith((message) => updates(message as FormValue)) as FormValue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormValue create() => FormValue._();
  @$core.override
  FormValue createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormValue getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FormValue>(create);
  static FormValue? _defaultInstance;

  /// Stable field identifier from the definition revision.
  @$pb.TagNumber(1)
  $core.String get fieldId => $_getSZ(0);
  @$pb.TagNumber(1)
  set fieldId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFieldId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFieldId() => $_clearField(1);

  /// Field label captured when the value was written.
  @$pb.TagNumber(2)
  $core.String get label => $_getSZ(1);
  @$pb.TagNumber(2)
  set label($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLabel() => $_has(1);
  @$pb.TagNumber(2)
  void clearLabel() => $_clearField(2);

  /// Captured field kind.
  @$pb.TagNumber(3)
  $core.String get kind => $_getSZ(2);
  @$pb.TagNumber(3)
  set kind($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);

  /// Non-secret text; intentionally empty for secret fields.
  @$pb.TagNumber(4)
  $core.String get value => $_getSZ(3);
  @$pb.TagNumber(4)
  set value($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearValue() => $_clearField(4);

  /// Whether the value must be read through a callback-scoped secret API.
  @$pb.TagNumber(5)
  $core.bool get secret => $_getBF(4);
  @$pb.TagNumber(5)
  set secret($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasSecret() => $_has(4);
  @$pb.TagNumber(5)
  void clearSecret() => $_clearField(5);
}

/// One typed record stored at a lockbox path.
class FormRecord extends $pb.GeneratedMessage {
  factory FormRecord({
    $core.String? path,
    $core.String? name,
    $core.String? typeId,
    $core.String? definitionAlias,
    $core.int? definitionRevision,
    $core.Iterable<FormValue>? values,
  }) {
    final result = create();
    if (path != null) result.path = path;
    if (name != null) result.name = name;
    if (typeId != null) result.typeId = typeId;
    if (definitionAlias != null) result.definitionAlias = definitionAlias;
    if (definitionRevision != null)
      result.definitionRevision = definitionRevision;
    if (values != null) result.values.addAll(values);
    return result;
  }

  FormRecord._();

  factory FormRecord.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormRecord.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormRecord',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'path')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'typeId')
    ..aOS(4, _omitFieldNames ? '' : 'definitionAlias')
    ..aI(5, _omitFieldNames ? '' : 'definitionRevision',
        fieldType: $pb.PbFieldType.OU3)
    ..pPM<FormValue>(6, _omitFieldNames ? '' : 'values',
        subBuilder: FormValue.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormRecord clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormRecord copyWith(void Function(FormRecord) updates) =>
      super.copyWith((message) => updates(message as FormRecord)) as FormRecord;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormRecord create() => FormRecord._();
  @$core.override
  FormRecord createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormRecord getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FormRecord>(create);
  static FormRecord? _defaultInstance;

  /// Record path within the lockbox.
  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => $_clearField(1);

  /// User-facing record name.
  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  /// Stable form type identifier.
  @$pb.TagNumber(3)
  $core.String get typeId => $_getSZ(2);
  @$pb.TagNumber(3)
  set typeId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTypeId() => $_has(2);
  @$pb.TagNumber(3)
  void clearTypeId() => $_clearField(3);

  /// Alias captured from the definition.
  @$pb.TagNumber(4)
  $core.String get definitionAlias => $_getSZ(3);
  @$pb.TagNumber(4)
  set definitionAlias($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDefinitionAlias() => $_has(3);
  @$pb.TagNumber(4)
  void clearDefinitionAlias() => $_clearField(4);

  /// Definition revision used to create the record.
  @$pb.TagNumber(5)
  $core.int get definitionRevision => $_getIZ(4);
  @$pb.TagNumber(5)
  set definitionRevision($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDefinitionRevision() => $_has(4);
  @$pb.TagNumber(5)
  void clearDefinitionRevision() => $_clearField(5);

  /// Field values in definition order.
  @$pb.TagNumber(6)
  $pb.PbList<FormValue> get values => $_getList(5);
}

/// A collection of form records.
class FormRecordList extends $pb.GeneratedMessage {
  factory FormRecordList({
    $core.Iterable<FormRecord>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  FormRecordList._();

  factory FormRecordList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FormRecordList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FormRecordList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<FormRecord>(1, _omitFieldNames ? '' : 'values',
        subBuilder: FormRecord.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormRecordList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FormRecordList copyWith(void Function(FormRecordList) updates) =>
      super.copyWith((message) => updates(message as FormRecordList))
          as FormRecordList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FormRecordList create() => FormRecordList._();
  @$core.override
  FormRecordList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FormRecordList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FormRecordList>(create);
  static FormRecordList? _defaultInstance;

  /// Records in normalized path order.
  @$pb.TagNumber(1)
  $pb.PbList<FormRecord> get values => $_getList(0);
}

/// Optional form record; `value` is absent when the record does not exist.
class OptionalFormRecord extends $pb.GeneratedMessage {
  factory OptionalFormRecord({
    FormRecord? value,
  }) {
    final result = create();
    if (value != null) result.value = value;
    return result;
  }

  OptionalFormRecord._();

  factory OptionalFormRecord.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OptionalFormRecord.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OptionalFormRecord',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOM<FormRecord>(1, _omitFieldNames ? '' : 'value',
        subBuilder: FormRecord.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalFormRecord clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalFormRecord copyWith(void Function(OptionalFormRecord) updates) =>
      super.copyWith((message) => updates(message as OptionalFormRecord))
          as OptionalFormRecord;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OptionalFormRecord create() => OptionalFormRecord._();
  @$core.override
  OptionalFormRecord createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OptionalFormRecord getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OptionalFormRecord>(create);
  static OptionalFormRecord? _defaultInstance;

  /// Present record value.
  @$pb.TagNumber(1)
  FormRecord get value => $_getN(0);
  @$pb.TagNumber(1)
  set value(FormRecord value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => $_clearField(1);
  @$pb.TagNumber(1)
  FormRecord ensureValue() => $_ensure(0);
}

/// Optional form value; `value` is absent when the field does not exist.
class OptionalFormValue extends $pb.GeneratedMessage {
  factory OptionalFormValue({
    FormValue? value,
  }) {
    final result = create();
    if (value != null) result.value = value;
    return result;
  }

  OptionalFormValue._();

  factory OptionalFormValue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OptionalFormValue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OptionalFormValue',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOM<FormValue>(1, _omitFieldNames ? '' : 'value',
        subBuilder: FormValue.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalFormValue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalFormValue copyWith(void Function(OptionalFormValue) updates) =>
      super.copyWith((message) => updates(message as OptionalFormValue))
          as OptionalFormValue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OptionalFormValue create() => OptionalFormValue._();
  @$core.override
  OptionalFormValue createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OptionalFormValue getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OptionalFormValue>(create);
  static OptionalFormValue? _defaultInstance;

  /// Present field value; secret payloads remain omitted.
  @$pb.TagNumber(1)
  FormValue get value => $_getN(0);
  @$pb.TagNumber(1)
  set value(FormValue value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => $_clearField(1);
  @$pb.TagNumber(1)
  FormValue ensureValue() => $_ensure(0);
}

/// Results from scanning or salvaging a damaged lockbox.
class RecoveryReport extends $pb.GeneratedMessage {
  factory RecoveryReport({
    $core.Iterable<LockboxEntry>? intactFiles,
    $fixnum.Int64? intactFileCount,
    $fixnum.Int64? partialFiles,
    $fixnum.Int64? corruptRecords,
    $core.bool? tocRecovered,
    $core.bool? variablesRecovered,
    $fixnum.Int64? variableCount,
    $core.bool? formsRecovered,
    $fixnum.Int64? formDefinitionCount,
    $fixnum.Int64? formRecordCount,
  }) {
    final result = create();
    if (intactFiles != null) result.intactFiles.addAll(intactFiles);
    if (intactFileCount != null) result.intactFileCount = intactFileCount;
    if (partialFiles != null) result.partialFiles = partialFiles;
    if (corruptRecords != null) result.corruptRecords = corruptRecords;
    if (tocRecovered != null) result.tocRecovered = tocRecovered;
    if (variablesRecovered != null)
      result.variablesRecovered = variablesRecovered;
    if (variableCount != null) result.variableCount = variableCount;
    if (formsRecovered != null) result.formsRecovered = formsRecovered;
    if (formDefinitionCount != null)
      result.formDefinitionCount = formDefinitionCount;
    if (formRecordCount != null) result.formRecordCount = formRecordCount;
    return result;
  }

  RecoveryReport._();

  factory RecoveryReport.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RecoveryReport.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RecoveryReport',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<LockboxEntry>(1, _omitFieldNames ? '' : 'intactFiles',
        subBuilder: LockboxEntry.create)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'intactFileCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'partialFiles', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'corruptRecords', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(5, _omitFieldNames ? '' : 'tocRecovered')
    ..aOB(6, _omitFieldNames ? '' : 'variablesRecovered')
    ..a<$fixnum.Int64>(
        7, _omitFieldNames ? '' : 'variableCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(8, _omitFieldNames ? '' : 'formsRecovered')
    ..a<$fixnum.Int64>(
        9, _omitFieldNames ? '' : 'formDefinitionCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        10, _omitFieldNames ? '' : 'formRecordCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RecoveryReport clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RecoveryReport copyWith(void Function(RecoveryReport) updates) =>
      super.copyWith((message) => updates(message as RecoveryReport))
          as RecoveryReport;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RecoveryReport create() => RecoveryReport._();
  @$core.override
  RecoveryReport createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RecoveryReport getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RecoveryReport>(create);
  static RecoveryReport? _defaultInstance;

  /// Metadata for files recovered in full.
  @$pb.TagNumber(1)
  $pb.PbList<LockboxEntry> get intactFiles => $_getList(0);

  /// Total number of fully recovered files.
  @$pb.TagNumber(2)
  $fixnum.Int64 get intactFileCount => $_getI64(1);
  @$pb.TagNumber(2)
  set intactFileCount($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIntactFileCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearIntactFileCount() => $_clearField(2);

  /// Number of files for which only partial content was recovered.
  @$pb.TagNumber(3)
  $fixnum.Int64 get partialFiles => $_getI64(2);
  @$pb.TagNumber(3)
  set partialFiles($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPartialFiles() => $_has(2);
  @$pb.TagNumber(3)
  void clearPartialFiles() => $_clearField(3);

  /// Number of corrupt records skipped by the scanner.
  @$pb.TagNumber(4)
  $fixnum.Int64 get corruptRecords => $_getI64(3);
  @$pb.TagNumber(4)
  set corruptRecords($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCorruptRecords() => $_has(3);
  @$pb.TagNumber(4)
  void clearCorruptRecords() => $_clearField(4);

  /// Whether the table of contents was recovered.
  @$pb.TagNumber(5)
  $core.bool get tocRecovered => $_getBF(4);
  @$pb.TagNumber(5)
  set tocRecovered($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTocRecovered() => $_has(4);
  @$pb.TagNumber(5)
  void clearTocRecovered() => $_clearField(5);

  /// Whether variable metadata was recovered.
  @$pb.TagNumber(6)
  $core.bool get variablesRecovered => $_getBF(5);
  @$pb.TagNumber(6)
  set variablesRecovered($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasVariablesRecovered() => $_has(5);
  @$pb.TagNumber(6)
  void clearVariablesRecovered() => $_clearField(6);

  /// Number of recovered variables.
  @$pb.TagNumber(7)
  $fixnum.Int64 get variableCount => $_getI64(6);
  @$pb.TagNumber(7)
  set variableCount($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasVariableCount() => $_has(6);
  @$pb.TagNumber(7)
  void clearVariableCount() => $_clearField(7);

  /// Whether typed form metadata was recovered.
  @$pb.TagNumber(8)
  $core.bool get formsRecovered => $_getBF(7);
  @$pb.TagNumber(8)
  set formsRecovered($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasFormsRecovered() => $_has(7);
  @$pb.TagNumber(8)
  void clearFormsRecovered() => $_clearField(8);

  /// Number of recovered form definitions.
  @$pb.TagNumber(9)
  $fixnum.Int64 get formDefinitionCount => $_getI64(8);
  @$pb.TagNumber(9)
  set formDefinitionCount($fixnum.Int64 value) => $_setInt64(8, value);
  @$pb.TagNumber(9)
  $core.bool hasFormDefinitionCount() => $_has(8);
  @$pb.TagNumber(9)
  void clearFormDefinitionCount() => $_clearField(9);

  /// Number of recovered form records.
  @$pb.TagNumber(10)
  $fixnum.Int64 get formRecordCount => $_getI64(9);
  @$pb.TagNumber(10)
  set formRecordCount($fixnum.Int64 value) => $_setInt64(9, value);
  @$pb.TagNumber(10)
  $core.bool hasFormRecordCount() => $_has(9);
  @$pb.TagNumber(10)
  void clearFormRecordCount() => $_clearField(10);
}

/// Metadata for one password or contact access slot.
class KeySlot extends $pb.GeneratedMessage {
  factory KeySlot({
    $fixnum.Int64? id,
    $core.String? protection,
    $core.String? algorithm,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (protection != null) result.protection = protection;
    if (algorithm != null) result.algorithm = algorithm;
    return result;
  }

  KeySlot._();

  factory KeySlot.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory KeySlot.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'KeySlot',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'protection')
    ..aOS(3, _omitFieldNames ? '' : 'algorithm')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KeySlot clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KeySlot copyWith(void Function(KeySlot) updates) =>
      super.copyWith((message) => updates(message as KeySlot)) as KeySlot;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static KeySlot create() => KeySlot._();
  @$core.override
  KeySlot createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static KeySlot getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<KeySlot>(create);
  static KeySlot? _defaultInstance;

  /// Stable slot identifier used for deletion and local labels.
  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// Protection type, such as `password` or `contact`.
  @$pb.TagNumber(2)
  $core.String get protection => $_getSZ(1);
  @$pb.TagNumber(2)
  set protection($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasProtection() => $_has(1);
  @$pb.TagNumber(2)
  void clearProtection() => $_clearField(2);

  /// Cryptographic algorithm identifier stored by the slot.
  @$pb.TagNumber(3)
  $core.String get algorithm => $_getSZ(2);
  @$pb.TagNumber(3)
  set algorithm($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAlgorithm() => $_has(2);
  @$pb.TagNumber(3)
  void clearAlgorithm() => $_clearField(3);
}

/// A collection of access slots.
class KeySlotList extends $pb.GeneratedMessage {
  factory KeySlotList({
    $core.Iterable<KeySlot>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  KeySlotList._();

  factory KeySlotList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory KeySlotList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'KeySlotList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<KeySlot>(1, _omitFieldNames ? '' : 'values',
        subBuilder: KeySlot.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KeySlotList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KeySlotList copyWith(void Function(KeySlotList) updates) =>
      super.copyWith((message) => updates(message as KeySlotList))
          as KeySlotList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static KeySlotList create() => KeySlotList._();
  @$core.override
  KeySlotList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static KeySlotList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<KeySlotList>(create);
  static KeySlotList? _defaultInstance;

  /// Access slots in stable id order.
  @$pb.TagNumber(1)
  $pb.PbList<KeySlot> get values => $_getList(0);
}

/// Current decrypted-page cache counters.
class CacheStats extends $pb.GeneratedMessage {
  factory CacheStats({
    $fixnum.Int64? limitBytes,
    $fixnum.Int64? usedBytes,
    $fixnum.Int64? entries,
    $fixnum.Int64? hits,
    $fixnum.Int64? misses,
  }) {
    final result = create();
    if (limitBytes != null) result.limitBytes = limitBytes;
    if (usedBytes != null) result.usedBytes = usedBytes;
    if (entries != null) result.entries = entries;
    if (hits != null) result.hits = hits;
    if (misses != null) result.misses = misses;
    return result;
  }

  CacheStats._();

  factory CacheStats.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CacheStats.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CacheStats',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'limitBytes', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'usedBytes', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'entries', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'hits', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'misses', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CacheStats clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CacheStats copyWith(void Function(CacheStats) updates) =>
      super.copyWith((message) => updates(message as CacheStats)) as CacheStats;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CacheStats create() => CacheStats._();
  @$core.override
  CacheStats createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CacheStats getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CacheStats>(create);
  static CacheStats? _defaultInstance;

  /// Configured cache limit in bytes.
  @$pb.TagNumber(1)
  $fixnum.Int64 get limitBytes => $_getI64(0);
  @$pb.TagNumber(1)
  set limitBytes($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLimitBytes() => $_has(0);
  @$pb.TagNumber(1)
  void clearLimitBytes() => $_clearField(1);

  /// Bytes currently held by the cache.
  @$pb.TagNumber(2)
  $fixnum.Int64 get usedBytes => $_getI64(1);
  @$pb.TagNumber(2)
  set usedBytes($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUsedBytes() => $_has(1);
  @$pb.TagNumber(2)
  void clearUsedBytes() => $_clearField(2);

  /// Number of cached entries.
  @$pb.TagNumber(3)
  $fixnum.Int64 get entries => $_getI64(2);
  @$pb.TagNumber(3)
  set entries($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEntries() => $_has(2);
  @$pb.TagNumber(3)
  void clearEntries() => $_clearField(3);

  /// Successful cache lookup count.
  @$pb.TagNumber(4)
  $fixnum.Int64 get hits => $_getI64(3);
  @$pb.TagNumber(4)
  set hits($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasHits() => $_has(3);
  @$pb.TagNumber(4)
  void clearHits() => $_clearField(4);

  /// Failed cache lookup count.
  @$pb.TagNumber(5)
  $fixnum.Int64 get misses => $_getI64(4);
  @$pb.TagNumber(5)
  set misses($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMisses() => $_has(4);
  @$pb.TagNumber(5)
  void clearMisses() => $_clearField(5);
}

/// Cumulative timing counters for bulk imports, represented as decimal nanoseconds.
class ImportStats extends $pb.GeneratedMessage {
  factory ImportStats({
    $core.String? hostStatNanos,
    $core.String? hostReadNanos,
    $core.String? framePrepareNanos,
    $core.String? pageWriteNanos,
  }) {
    final result = create();
    if (hostStatNanos != null) result.hostStatNanos = hostStatNanos;
    if (hostReadNanos != null) result.hostReadNanos = hostReadNanos;
    if (framePrepareNanos != null) result.framePrepareNanos = framePrepareNanos;
    if (pageWriteNanos != null) result.pageWriteNanos = pageWriteNanos;
    return result;
  }

  ImportStats._();

  factory ImportStats.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImportStats.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImportStats',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'hostStatNanos')
    ..aOS(2, _omitFieldNames ? '' : 'hostReadNanos')
    ..aOS(3, _omitFieldNames ? '' : 'framePrepareNanos')
    ..aOS(4, _omitFieldNames ? '' : 'pageWriteNanos')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportStats clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportStats copyWith(void Function(ImportStats) updates) =>
      super.copyWith((message) => updates(message as ImportStats))
          as ImportStats;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportStats create() => ImportStats._();
  @$core.override
  ImportStats createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImportStats getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImportStats>(create);
  static ImportStats? _defaultInstance;

  /// Time spent querying host file metadata.
  @$pb.TagNumber(1)
  $core.String get hostStatNanos => $_getSZ(0);
  @$pb.TagNumber(1)
  set hostStatNanos($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasHostStatNanos() => $_has(0);
  @$pb.TagNumber(1)
  void clearHostStatNanos() => $_clearField(1);

  /// Time spent reading host file content.
  @$pb.TagNumber(2)
  $core.String get hostReadNanos => $_getSZ(1);
  @$pb.TagNumber(2)
  set hostReadNanos($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasHostReadNanos() => $_has(1);
  @$pb.TagNumber(2)
  void clearHostReadNanos() => $_clearField(2);

  /// Time spent preparing compression/encryption frames.
  @$pb.TagNumber(3)
  $core.String get framePrepareNanos => $_getSZ(2);
  @$pb.TagNumber(3)
  set framePrepareNanos($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFramePrepareNanos() => $_has(2);
  @$pb.TagNumber(3)
  void clearFramePrepareNanos() => $_clearField(3);

  /// Time spent writing encrypted pages.
  @$pb.TagNumber(4)
  $core.String get pageWriteNanos => $_getSZ(3);
  @$pb.TagNumber(4)
  set pageWriteNanos($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPageWriteNanos() => $_has(3);
  @$pb.TagNumber(4)
  void clearPageWriteNanos() => $_clearField(4);
}

/// One logical object stored inside an encrypted page.
class PageObject extends $pb.GeneratedMessage {
  factory PageObject({
    $fixnum.Int64? id,
    $core.String? kind,
    $fixnum.Int64? payloadLen,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (kind != null) result.kind = kind;
    if (payloadLen != null) result.payloadLen = payloadLen;
    return result;
  }

  PageObject._();

  factory PageObject.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PageObject.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PageObject',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'kind')
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'payloadLen', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PageObject clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PageObject copyWith(void Function(PageObject) updates) =>
      super.copyWith((message) => updates(message as PageObject)) as PageObject;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PageObject create() => PageObject._();
  @$core.override
  PageObject createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PageObject getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PageObject>(create);
  static PageObject? _defaultInstance;

  /// Object identifier within the page.
  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// Object type name.
  @$pb.TagNumber(2)
  $core.String get kind => $_getSZ(1);
  @$pb.TagNumber(2)
  set kind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  /// Encoded payload length in bytes.
  @$pb.TagNumber(3)
  $fixnum.Int64 get payloadLen => $_getI64(2);
  @$pb.TagNumber(3)
  set payloadLen($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPayloadLen() => $_has(2);
  @$pb.TagNumber(3)
  void clearPayloadLen() => $_clearField(3);
}

/// Diagnostic metadata for one encrypted lockbox page.
class PageInspection extends $pb.GeneratedMessage {
  factory PageInspection({
    $fixnum.Int64? offset,
    $fixnum.Int64? pageId,
    $fixnum.Int64? sequence,
    $fixnum.Int64? pageSize,
    $fixnum.Int64? encryptedBodyLen,
    $fixnum.Int64? unusedBytes,
    $fixnum.Int64? objectCount,
    $core.Iterable<PageObject>? objects,
  }) {
    final result = create();
    if (offset != null) result.offset = offset;
    if (pageId != null) result.pageId = pageId;
    if (sequence != null) result.sequence = sequence;
    if (pageSize != null) result.pageSize = pageSize;
    if (encryptedBodyLen != null) result.encryptedBodyLen = encryptedBodyLen;
    if (unusedBytes != null) result.unusedBytes = unusedBytes;
    if (objectCount != null) result.objectCount = objectCount;
    if (objects != null) result.objects.addAll(objects);
    return result;
  }

  PageInspection._();

  factory PageInspection.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PageInspection.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PageInspection',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'offset', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'pageId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'sequence', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'pageSize', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        5, _omitFieldNames ? '' : 'encryptedBodyLen', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        6, _omitFieldNames ? '' : 'unusedBytes', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        7, _omitFieldNames ? '' : 'objectCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..pPM<PageObject>(8, _omitFieldNames ? '' : 'objects',
        subBuilder: PageObject.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PageInspection clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PageInspection copyWith(void Function(PageInspection) updates) =>
      super.copyWith((message) => updates(message as PageInspection))
          as PageInspection;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PageInspection create() => PageInspection._();
  @$core.override
  PageInspection createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PageInspection getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PageInspection>(create);
  static PageInspection? _defaultInstance;

  /// Physical page offset in the archive.
  @$pb.TagNumber(1)
  $fixnum.Int64 get offset => $_getI64(0);
  @$pb.TagNumber(1)
  set offset($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOffset() => $_has(0);
  @$pb.TagNumber(1)
  void clearOffset() => $_clearField(1);

  /// Stable page identifier.
  @$pb.TagNumber(2)
  $fixnum.Int64 get pageId => $_getI64(1);
  @$pb.TagNumber(2)
  set pageId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPageId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPageId() => $_clearField(2);

  /// Commit sequence that last wrote the page.
  @$pb.TagNumber(3)
  $fixnum.Int64 get sequence => $_getI64(2);
  @$pb.TagNumber(3)
  set sequence($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSequence() => $_has(2);
  @$pb.TagNumber(3)
  void clearSequence() => $_clearField(3);

  /// Total encoded page length.
  @$pb.TagNumber(4)
  $fixnum.Int64 get pageSize => $_getI64(3);
  @$pb.TagNumber(4)
  set pageSize($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPageSize() => $_has(3);
  @$pb.TagNumber(4)
  void clearPageSize() => $_clearField(4);

  /// Length of the authenticated encrypted body.
  @$pb.TagNumber(5)
  $fixnum.Int64 get encryptedBodyLen => $_getI64(4);
  @$pb.TagNumber(5)
  set encryptedBodyLen($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEncryptedBodyLen() => $_has(4);
  @$pb.TagNumber(5)
  void clearEncryptedBodyLen() => $_clearField(5);

  /// Unused capacity remaining in the page.
  @$pb.TagNumber(6)
  $fixnum.Int64 get unusedBytes => $_getI64(5);
  @$pb.TagNumber(6)
  set unusedBytes($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasUnusedBytes() => $_has(5);
  @$pb.TagNumber(6)
  void clearUnusedBytes() => $_clearField(6);

  /// Number of logical objects in the page.
  @$pb.TagNumber(7)
  $fixnum.Int64 get objectCount => $_getI64(6);
  @$pb.TagNumber(7)
  set objectCount($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasObjectCount() => $_has(6);
  @$pb.TagNumber(7)
  void clearObjectCount() => $_clearField(7);

  /// Objects discoverable without exposing plaintext payloads.
  @$pb.TagNumber(8)
  $pb.PbList<PageObject> get objects => $_getList(7);
}

/// A collection of page diagnostics.
class PageInspectionList extends $pb.GeneratedMessage {
  factory PageInspectionList({
    $core.Iterable<PageInspection>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  PageInspectionList._();

  factory PageInspectionList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PageInspectionList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PageInspectionList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<PageInspection>(1, _omitFieldNames ? '' : 'values',
        subBuilder: PageInspection.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PageInspectionList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PageInspectionList copyWith(void Function(PageInspectionList) updates) =>
      super.copyWith((message) => updates(message as PageInspectionList))
          as PageInspectionList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PageInspectionList create() => PageInspectionList._();
  @$core.override
  PageInspectionList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PageInspectionList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PageInspectionList>(create);
  static PageInspectionList? _defaultInstance;

  /// Pages in physical archive order.
  @$pb.TagNumber(1)
  $pb.PbList<PageInspection> get values => $_getList(0);
}

/// Metadata that can be inspected from a lockbox file without opening content.
class FileInspection extends $pb.GeneratedMessage {
  factory FileInspection({
    $core.List<$core.int>? lockboxId,
    $core.bool? headerReadable,
    $fixnum.Int64? keyDirectoryGeneration,
    $fixnum.Int64? keyDirectoryCopyCount,
    $core.bool? ownerSigned,
    $core.Iterable<KeySlot>? keySlots,
  }) {
    final result = create();
    if (lockboxId != null) result.lockboxId = lockboxId;
    if (headerReadable != null) result.headerReadable = headerReadable;
    if (keyDirectoryGeneration != null)
      result.keyDirectoryGeneration = keyDirectoryGeneration;
    if (keyDirectoryCopyCount != null)
      result.keyDirectoryCopyCount = keyDirectoryCopyCount;
    if (ownerSigned != null) result.ownerSigned = ownerSigned;
    if (keySlots != null) result.keySlots.addAll(keySlots);
    return result;
  }

  FileInspection._();

  factory FileInspection.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FileInspection.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FileInspection',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'lockboxId', $pb.PbFieldType.OY)
    ..aOB(2, _omitFieldNames ? '' : 'headerReadable')
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'keyDirectoryGeneration', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'keyDirectoryCopyCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(5, _omitFieldNames ? '' : 'ownerSigned')
    ..pPM<KeySlot>(6, _omitFieldNames ? '' : 'keySlots',
        subBuilder: KeySlot.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FileInspection clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FileInspection copyWith(void Function(FileInspection) updates) =>
      super.copyWith((message) => updates(message as FileInspection))
          as FileInspection;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FileInspection create() => FileInspection._();
  @$core.override
  FileInspection createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FileInspection getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FileInspection>(create);
  static FileInspection? _defaultInstance;

  /// Stable lockbox identifier.
  @$pb.TagNumber(1)
  $core.List<$core.int> get lockboxId => $_getN(0);
  @$pb.TagNumber(1)
  set lockboxId($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLockboxId() => $_has(0);
  @$pb.TagNumber(1)
  void clearLockboxId() => $_clearField(1);

  /// Whether the file header passed structural validation.
  @$pb.TagNumber(2)
  $core.bool get headerReadable => $_getBF(1);
  @$pb.TagNumber(2)
  set headerReadable($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasHeaderReadable() => $_has(1);
  @$pb.TagNumber(2)
  void clearHeaderReadable() => $_clearField(2);

  /// Latest readable key-directory generation.
  @$pb.TagNumber(3)
  $fixnum.Int64 get keyDirectoryGeneration => $_getI64(2);
  @$pb.TagNumber(3)
  set keyDirectoryGeneration($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKeyDirectoryGeneration() => $_has(2);
  @$pb.TagNumber(3)
  void clearKeyDirectoryGeneration() => $_clearField(3);

  /// Number of valid redundant key-directory copies.
  @$pb.TagNumber(4)
  $fixnum.Int64 get keyDirectoryCopyCount => $_getI64(3);
  @$pb.TagNumber(4)
  set keyDirectoryCopyCount($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasKeyDirectoryCopyCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearKeyDirectoryCopyCount() => $_clearField(4);

  /// Whether commits are authorized by an owner signing key.
  @$pb.TagNumber(5)
  $core.bool get ownerSigned => $_getBF(4);
  @$pb.TagNumber(5)
  set ownerSigned($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOwnerSigned() => $_has(4);
  @$pb.TagNumber(5)
  void clearOwnerSigned() => $_clearField(5);

  /// Access slots visible in the key directory.
  @$pb.TagNumber(6)
  $pb.PbList<KeySlot> get keySlots => $_getList(5);
}

/// One historical generation of a vault profile's contact keys.
class ProfileGeneration extends $pb.GeneratedMessage {
  factory ProfileGeneration({
    $core.int? index,
    $core.String? status,
    $core.List<$core.int>? contactFingerprint,
    $fixnum.Int64? createdAtUnixMs,
    $fixnum.Int64? retiredAtUnixMs,
    $core.bool? hasRetiredAt,
  }) {
    final result = create();
    if (index != null) result.index = index;
    if (status != null) result.status = status;
    if (contactFingerprint != null)
      result.contactFingerprint = contactFingerprint;
    if (createdAtUnixMs != null) result.createdAtUnixMs = createdAtUnixMs;
    if (retiredAtUnixMs != null) result.retiredAtUnixMs = retiredAtUnixMs;
    if (hasRetiredAt != null) result.hasRetiredAt = hasRetiredAt;
    return result;
  }

  ProfileGeneration._();

  factory ProfileGeneration.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ProfileGeneration.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ProfileGeneration',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'contactFingerprint', $pb.PbFieldType.OY)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'createdAtUnixMs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        5, _omitFieldNames ? '' : 'retiredAtUnixMs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(6, _omitFieldNames ? '' : 'hasRetiredAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProfileGeneration clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProfileGeneration copyWith(void Function(ProfileGeneration) updates) =>
      super.copyWith((message) => updates(message as ProfileGeneration))
          as ProfileGeneration;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ProfileGeneration create() => ProfileGeneration._();
  @$core.override
  ProfileGeneration createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ProfileGeneration getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProfileGeneration>(create);
  static ProfileGeneration? _defaultInstance;

  /// Monotonically increasing generation number.
  @$pb.TagNumber(1)
  $core.int get index => $_getIZ(0);
  @$pb.TagNumber(1)
  set index($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearIndex() => $_clearField(1);

  /// Lifecycle status: `active`, `retired`, or `compromised`.
  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  /// Fingerprint of the generation's contact public key.
  @$pb.TagNumber(3)
  $core.List<$core.int> get contactFingerprint => $_getN(2);
  @$pb.TagNumber(3)
  set contactFingerprint($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContactFingerprint() => $_has(2);
  @$pb.TagNumber(3)
  void clearContactFingerprint() => $_clearField(3);

  /// Creation time in Unix milliseconds.
  @$pb.TagNumber(4)
  $fixnum.Int64 get createdAtUnixMs => $_getI64(3);
  @$pb.TagNumber(4)
  set createdAtUnixMs($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedAtUnixMs() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedAtUnixMs() => $_clearField(4);

  /// Retirement time in Unix milliseconds when present.
  @$pb.TagNumber(5)
  $fixnum.Int64 get retiredAtUnixMs => $_getI64(4);
  @$pb.TagNumber(5)
  set retiredAtUnixMs($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRetiredAtUnixMs() => $_has(4);
  @$pb.TagNumber(5)
  void clearRetiredAtUnixMs() => $_clearField(5);

  /// Whether `retired_at_unix_ms` is present.
  @$pb.TagNumber(6)
  $core.bool get hasRetiredAt => $_getBF(5);
  @$pb.TagNumber(6)
  set hasRetiredAt($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasHasRetiredAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearHasRetiredAt() => $_clearField(6);
}

/// Versioned key-generation history for one named profile.
class ProfileHistory extends $pb.GeneratedMessage {
  factory ProfileHistory({
    $core.String? name,
    $core.int? activeGeneration,
    $core.Iterable<ProfileGeneration>? generations,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (activeGeneration != null) result.activeGeneration = activeGeneration;
    if (generations != null) result.generations.addAll(generations);
    return result;
  }

  ProfileHistory._();

  factory ProfileHistory.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ProfileHistory.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ProfileHistory',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'activeGeneration',
        fieldType: $pb.PbFieldType.OU3)
    ..pPM<ProfileGeneration>(3, _omitFieldNames ? '' : 'generations',
        subBuilder: ProfileGeneration.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProfileHistory clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProfileHistory copyWith(void Function(ProfileHistory) updates) =>
      super.copyWith((message) => updates(message as ProfileHistory))
          as ProfileHistory;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ProfileHistory create() => ProfileHistory._();
  @$core.override
  ProfileHistory createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ProfileHistory getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProfileHistory>(create);
  static ProfileHistory? _defaultInstance;

  /// Profile name.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// Index of the active generation.
  @$pb.TagNumber(2)
  $core.int get activeGeneration => $_getIZ(1);
  @$pb.TagNumber(2)
  set activeGeneration($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasActiveGeneration() => $_has(1);
  @$pb.TagNumber(2)
  void clearActiveGeneration() => $_clearField(2);

  /// All active, retired, and compromised generations.
  @$pb.TagNumber(3)
  $pb.PbList<ProfileGeneration> get generations => $_getList(2);
}

/// Local metadata for a remembered lockbox path.
class KnownLockbox extends $pb.GeneratedMessage {
  factory KnownLockbox({
    $core.List<$core.int>? lockboxId,
    $core.String? path,
    $fixnum.Int64? lastSeenUnixMs,
  }) {
    final result = create();
    if (lockboxId != null) result.lockboxId = lockboxId;
    if (path != null) result.path = path;
    if (lastSeenUnixMs != null) result.lastSeenUnixMs = lastSeenUnixMs;
    return result;
  }

  KnownLockbox._();

  factory KnownLockbox.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory KnownLockbox.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'KnownLockbox',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'lockboxId', $pb.PbFieldType.OY)
    ..aOS(2, _omitFieldNames ? '' : 'path')
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'lastSeenUnixMs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KnownLockbox clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KnownLockbox copyWith(void Function(KnownLockbox) updates) =>
      super.copyWith((message) => updates(message as KnownLockbox))
          as KnownLockbox;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static KnownLockbox create() => KnownLockbox._();
  @$core.override
  KnownLockbox createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static KnownLockbox getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<KnownLockbox>(create);
  static KnownLockbox? _defaultInstance;

  /// Stable identifier embedded in the lockbox.
  @$pb.TagNumber(1)
  $core.List<$core.int> get lockboxId => $_getN(0);
  @$pb.TagNumber(1)
  set lockboxId($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLockboxId() => $_has(0);
  @$pb.TagNumber(1)
  void clearLockboxId() => $_clearField(1);

  /// Filesystem path at which the lockbox was last seen.
  @$pb.TagNumber(2)
  $core.String get path => $_getSZ(1);
  @$pb.TagNumber(2)
  set path($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPath() => $_clearField(2);

  /// Last observation time in Unix milliseconds.
  @$pb.TagNumber(3)
  $fixnum.Int64 get lastSeenUnixMs => $_getI64(2);
  @$pb.TagNumber(3)
  set lastSeenUnixMs($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLastSeenUnixMs() => $_has(2);
  @$pb.TagNumber(3)
  void clearLastSeenUnixMs() => $_clearField(3);
}

/// A collection of remembered lockboxes.
class KnownLockboxList extends $pb.GeneratedMessage {
  factory KnownLockboxList({
    $core.Iterable<KnownLockbox>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  KnownLockboxList._();

  factory KnownLockboxList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory KnownLockboxList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'KnownLockboxList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<KnownLockbox>(1, _omitFieldNames ? '' : 'values',
        subBuilder: KnownLockbox.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KnownLockboxList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  KnownLockboxList copyWith(void Function(KnownLockboxList) updates) =>
      super.copyWith((message) => updates(message as KnownLockboxList))
          as KnownLockboxList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static KnownLockboxList create() => KnownLockboxList._();
  @$core.override
  KnownLockboxList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static KnownLockboxList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<KnownLockboxList>(create);
  static KnownLockboxList? _defaultInstance;

  /// Remembered lockbox records.
  @$pb.TagNumber(1)
  $pb.PbList<KnownLockbox> get values => $_getList(0);
}

/// A local-only label for one lockbox access slot.
class AccessSlotLabel extends $pb.GeneratedMessage {
  factory AccessSlotLabel({
    $core.List<$core.int>? lockboxId,
    $fixnum.Int64? slotId,
    $core.String? name,
    $fixnum.Int64? updatedAtUnixMs,
  }) {
    final result = create();
    if (lockboxId != null) result.lockboxId = lockboxId;
    if (slotId != null) result.slotId = slotId;
    if (name != null) result.name = name;
    if (updatedAtUnixMs != null) result.updatedAtUnixMs = updatedAtUnixMs;
    return result;
  }

  AccessSlotLabel._();

  factory AccessSlotLabel.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AccessSlotLabel.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AccessSlotLabel',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'lockboxId', $pb.PbFieldType.OY)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'slotId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'name')
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'updatedAtUnixMs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AccessSlotLabel clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AccessSlotLabel copyWith(void Function(AccessSlotLabel) updates) =>
      super.copyWith((message) => updates(message as AccessSlotLabel))
          as AccessSlotLabel;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AccessSlotLabel create() => AccessSlotLabel._();
  @$core.override
  AccessSlotLabel createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AccessSlotLabel getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AccessSlotLabel>(create);
  static AccessSlotLabel? _defaultInstance;

  /// Lockbox containing the access slot.
  @$pb.TagNumber(1)
  $core.List<$core.int> get lockboxId => $_getN(0);
  @$pb.TagNumber(1)
  set lockboxId($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLockboxId() => $_has(0);
  @$pb.TagNumber(1)
  void clearLockboxId() => $_clearField(1);

  /// Stable slot identifier within that lockbox.
  @$pb.TagNumber(2)
  $fixnum.Int64 get slotId => $_getI64(1);
  @$pb.TagNumber(2)
  set slotId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSlotId() => $_has(1);
  @$pb.TagNumber(2)
  void clearSlotId() => $_clearField(2);

  /// User-assigned local label.
  @$pb.TagNumber(3)
  $core.String get name => $_getSZ(2);
  @$pb.TagNumber(3)
  set name($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasName() => $_has(2);
  @$pb.TagNumber(3)
  void clearName() => $_clearField(3);

  /// Last update time in Unix milliseconds.
  @$pb.TagNumber(4)
  $fixnum.Int64 get updatedAtUnixMs => $_getI64(3);
  @$pb.TagNumber(4)
  set updatedAtUnixMs($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasUpdatedAtUnixMs() => $_has(3);
  @$pb.TagNumber(4)
  void clearUpdatedAtUnixMs() => $_clearField(4);
}

/// A collection of local access-slot labels.
class AccessSlotLabelList extends $pb.GeneratedMessage {
  factory AccessSlotLabelList({
    $core.Iterable<AccessSlotLabel>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  AccessSlotLabelList._();

  factory AccessSlotLabelList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AccessSlotLabelList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AccessSlotLabelList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<AccessSlotLabel>(1, _omitFieldNames ? '' : 'values',
        subBuilder: AccessSlotLabel.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AccessSlotLabelList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AccessSlotLabelList copyWith(void Function(AccessSlotLabelList) updates) =>
      super.copyWith((message) => updates(message as AccessSlotLabelList))
          as AccessSlotLabelList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AccessSlotLabelList create() => AccessSlotLabelList._();
  @$core.override
  AccessSlotLabelList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AccessSlotLabelList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AccessSlotLabelList>(create);
  static AccessSlotLabelList? _defaultInstance;

  /// Matching access-slot labels.
  @$pb.TagNumber(1)
  $pb.PbList<AccessSlotLabel> get values => $_getList(0);
}

/// One logical or physical chunk in a streamed lockbox file.
class StreamChunk extends $pb.GeneratedMessage {
  factory StreamChunk({
    $core.String? path,
    $fixnum.Int64? fileOffset,
    $fixnum.Int64? length,
    $fixnum.Int64? physicalOffset,
    $core.bool? sparse,
    $core.List<$core.int>? data,
  }) {
    final result = create();
    if (path != null) result.path = path;
    if (fileOffset != null) result.fileOffset = fileOffset;
    if (length != null) result.length = length;
    if (physicalOffset != null) result.physicalOffset = physicalOffset;
    if (sparse != null) result.sparse = sparse;
    if (data != null) result.data = data;
    return result;
  }

  StreamChunk._();

  factory StreamChunk.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StreamChunk.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StreamChunk',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'path')
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'fileOffset', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'length', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'physicalOffset', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(5, _omitFieldNames ? '' : 'sparse')
    ..a<$core.List<$core.int>>(
        6, _omitFieldNames ? '' : 'data', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StreamChunk clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StreamChunk copyWith(void Function(StreamChunk) updates) =>
      super.copyWith((message) => updates(message as StreamChunk))
          as StreamChunk;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StreamChunk create() => StreamChunk._();
  @$core.override
  StreamChunk createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StreamChunk getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StreamChunk>(create);
  static StreamChunk? _defaultInstance;

  /// Lockbox path of the streamed file.
  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => $_clearField(1);

  /// Logical offset within the reconstructed file.
  @$pb.TagNumber(2)
  $fixnum.Int64 get fileOffset => $_getI64(1);
  @$pb.TagNumber(2)
  set fileOffset($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFileOffset() => $_has(1);
  @$pb.TagNumber(2)
  void clearFileOffset() => $_clearField(2);

  /// Logical chunk length in bytes.
  @$pb.TagNumber(3)
  $fixnum.Int64 get length => $_getI64(2);
  @$pb.TagNumber(3)
  set length($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLength() => $_has(2);
  @$pb.TagNumber(3)
  void clearLength() => $_clearField(3);

  /// Physical archive offset, when physical ordering was requested.
  @$pb.TagNumber(4)
  $fixnum.Int64 get physicalOffset => $_getI64(3);
  @$pb.TagNumber(4)
  set physicalOffset($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPhysicalOffset() => $_has(3);
  @$pb.TagNumber(4)
  void clearPhysicalOffset() => $_clearField(4);

  /// Whether the chunk represents a sparse zero-filled range.
  @$pb.TagNumber(5)
  $core.bool get sparse => $_getBF(4);
  @$pb.TagNumber(5)
  set sparse($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasSparse() => $_has(4);
  @$pb.TagNumber(5)
  void clearSparse() => $_clearField(5);

  /// Decrypted bytes; empty for sparse ranges.
  @$pb.TagNumber(6)
  $core.List<$core.int> get data => $_getN(5);
  @$pb.TagNumber(6)
  set data($core.List<$core.int> value) => $_setBytes(5, value);
  @$pb.TagNumber(6)
  $core.bool hasData() => $_has(5);
  @$pb.TagNumber(6)
  void clearData() => $_clearField(6);
}

/// A collection of streamed content chunks.
class StreamChunkList extends $pb.GeneratedMessage {
  factory StreamChunkList({
    $core.Iterable<StreamChunk>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  StreamChunkList._();

  factory StreamChunkList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StreamChunkList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StreamChunkList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<StreamChunk>(1, _omitFieldNames ? '' : 'values',
        subBuilder: StreamChunk.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StreamChunkList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StreamChunkList copyWith(void Function(StreamChunkList) updates) =>
      super.copyWith((message) => updates(message as StreamChunkList))
          as StreamChunkList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StreamChunkList create() => StreamChunkList._();
  @$core.override
  StreamChunkList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StreamChunkList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StreamChunkList>(create);
  static StreamChunkList? _defaultInstance;

  /// Chunks in logical or physical order as requested.
  @$pb.TagNumber(1)
  $pb.PbList<StreamChunk> get values => $_getList(0);
}

/// Effective runtime tuning for one open lockbox.
class RuntimeOptions extends $pb.GeneratedMessage {
  factory RuntimeOptions({
    $core.String? workloadProfile,
    $core.String? workerPolicy,
  }) {
    final result = create();
    if (workloadProfile != null) result.workloadProfile = workloadProfile;
    if (workerPolicy != null) result.workerPolicy = workerPolicy;
    return result;
  }

  RuntimeOptions._();

  factory RuntimeOptions.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RuntimeOptions.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RuntimeOptions',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workloadProfile')
    ..aOS(2, _omitFieldNames ? '' : 'workerPolicy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RuntimeOptions clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RuntimeOptions copyWith(void Function(RuntimeOptions) updates) =>
      super.copyWith((message) => updates(message as RuntimeOptions))
          as RuntimeOptions;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RuntimeOptions create() => RuntimeOptions._();
  @$core.override
  RuntimeOptions createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RuntimeOptions getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RuntimeOptions>(create);
  static RuntimeOptions? _defaultInstance;

  /// Selected workload profile.
  @$pb.TagNumber(1)
  $core.String get workloadProfile => $_getSZ(0);
  @$pb.TagNumber(1)
  set workloadProfile($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkloadProfile() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkloadProfile() => $_clearField(1);

  /// Selected worker policy and concurrency.
  @$pb.TagNumber(2)
  $core.String get workerPolicy => $_getSZ(1);
  @$pb.TagNumber(2)
  set workerPolicy($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWorkerPolicy() => $_has(1);
  @$pb.TagNumber(2)
  void clearWorkerPolicy() => $_clearField(2);
}

/// Name and sensitivity metadata for one variable; no secret value is included.
class Variable extends $pb.GeneratedMessage {
  factory Variable({
    $core.String? name,
    $core.String? sensitivity,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (sensitivity != null) result.sensitivity = sensitivity;
    return result;
  }

  Variable._();

  factory Variable.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Variable.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Variable',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'sensitivity')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Variable clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Variable copyWith(void Function(Variable) updates) =>
      super.copyWith((message) => updates(message as Variable)) as Variable;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Variable create() => Variable._();
  @$core.override
  Variable createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Variable getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Variable>(create);
  static Variable? _defaultInstance;

  /// Variable name.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// `normal` or `secret`.
  @$pb.TagNumber(2)
  $core.String get sensitivity => $_getSZ(1);
  @$pb.TagNumber(2)
  set sensitivity($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSensitivity() => $_has(1);
  @$pb.TagNumber(2)
  void clearSensitivity() => $_clearField(2);
}

/// A collection of variable metadata.
class VariableList extends $pb.GeneratedMessage {
  factory VariableList({
    $core.Iterable<Variable>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  VariableList._();

  factory VariableList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VariableList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VariableList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<Variable>(1, _omitFieldNames ? '' : 'values',
        subBuilder: Variable.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableList copyWith(void Function(VariableList) updates) =>
      super.copyWith((message) => updates(message as VariableList))
          as VariableList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VariableList create() => VariableList._();
  @$core.override
  VariableList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VariableList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VariableList>(create);
  static VariableList? _defaultInstance;

  /// Variables in normalized name order.
  @$pb.TagNumber(1)
  $pb.PbList<Variable> get values => $_getList(0);
}

/// Optional string that distinguishes absence from an empty value.
class OptionalString extends $pb.GeneratedMessage {
  factory OptionalString({
    $core.bool? present,
    $core.String? value,
  }) {
    final result = create();
    if (present != null) result.present = present;
    if (value != null) result.value = value;
    return result;
  }

  OptionalString._();

  factory OptionalString.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OptionalString.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OptionalString',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'present')
    ..aOS(2, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalString clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OptionalString copyWith(void Function(OptionalString) updates) =>
      super.copyWith((message) => updates(message as OptionalString))
          as OptionalString;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OptionalString create() => OptionalString._();
  @$core.override
  OptionalString createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OptionalString getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OptionalString>(create);
  static OptionalString? _defaultInstance;

  /// Whether a value is present.
  @$pb.TagNumber(1)
  $core.bool get present => $_getBF(0);
  @$pb.TagNumber(1)
  set present($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPresent() => $_has(0);
  @$pb.TagNumber(1)
  void clearPresent() => $_clearField(1);

  /// Present UTF-8 value, which may itself be empty.
  @$pb.TagNumber(2)
  $core.String get value => $_getSZ(1);
  @$pb.TagNumber(2)
  set value($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);
}

/// Owner-signature status for an open lockbox.
class OwnerInspection extends $pb.GeneratedMessage {
  factory OwnerInspection({
    $core.bool? signed,
    $core.String? fingerprint,
    $core.bool? hasFingerprint_3,
  }) {
    final result = create();
    if (signed != null) result.signed = signed;
    if (fingerprint != null) result.fingerprint = fingerprint;
    if (hasFingerprint_3 != null) result.hasFingerprint_3 = hasFingerprint_3;
    return result;
  }

  OwnerInspection._();

  factory OwnerInspection.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OwnerInspection.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OwnerInspection',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'signed')
    ..aOS(2, _omitFieldNames ? '' : 'fingerprint')
    ..aOB(3, _omitFieldNames ? '' : 'hasFingerprint')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OwnerInspection clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OwnerInspection copyWith(void Function(OwnerInspection) updates) =>
      super.copyWith((message) => updates(message as OwnerInspection))
          as OwnerInspection;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OwnerInspection create() => OwnerInspection._();
  @$core.override
  OwnerInspection createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OwnerInspection getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OwnerInspection>(create);
  static OwnerInspection? _defaultInstance;

  /// Whether the lockbox has owner-signed commits.
  @$pb.TagNumber(1)
  $core.bool get signed => $_getBF(0);
  @$pb.TagNumber(1)
  set signed($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSigned() => $_has(0);
  @$pb.TagNumber(1)
  void clearSigned() => $_clearField(1);

  /// Human-readable owner signing-key fingerprint.
  @$pb.TagNumber(2)
  $core.String get fingerprint => $_getSZ(1);
  @$pb.TagNumber(2)
  set fingerprint($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFingerprint() => $_has(1);
  @$pb.TagNumber(2)
  void clearFingerprint() => $_clearField(2);

  /// Whether `fingerprint` is present.
  @$pb.TagNumber(3)
  $core.bool get hasFingerprint_3 => $_getBF(2);
  @$pb.TagNumber(3)
  set hasFingerprint_3($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasHasFingerprint_3() => $_has(2);
  @$pb.TagNumber(3)
  void clearHasFingerprint_3() => $_clearField(3);
}

/// Named contact public key stored in the local vault.
class Contact extends $pb.GeneratedMessage {
  factory Contact({
    $core.String? name,
    $core.List<$core.int>? key,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (key != null) result.key = key;
    return result;
  }

  Contact._();

  factory Contact.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Contact.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Contact',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'key', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Contact clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Contact copyWith(void Function(Contact) updates) =>
      super.copyWith((message) => updates(message as Contact)) as Contact;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Contact create() => Contact._();
  @$core.override
  Contact createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Contact getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Contact>(create);
  static Contact? _defaultInstance;

  /// User-assigned contact name.
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// Encoded contact public key.
  @$pb.TagNumber(2)
  $core.List<$core.int> get key => $_getN(1);
  @$pb.TagNumber(2)
  set key($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearKey() => $_clearField(2);
}

/// A collection of contacts.
class ContactList extends $pb.GeneratedMessage {
  factory ContactList({
    $core.Iterable<Contact>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  ContactList._();

  factory ContactList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ContactList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ContactList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<Contact>(1, _omitFieldNames ? '' : 'values',
        subBuilder: Contact.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ContactList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ContactList copyWith(void Function(ContactList) updates) =>
      super.copyWith((message) => updates(message as ContactList))
          as ContactList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ContactList create() => ContactList._();
  @$core.override
  ContactList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ContactList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ContactList>(create);
  static ContactList? _defaultInstance;

  /// Contacts in normalized name order.
  @$pb.TagNumber(1)
  $pb.PbList<Contact> get values => $_getList(0);
}

/// A collection of profile histories.
class ProfileHistoryList extends $pb.GeneratedMessage {
  factory ProfileHistoryList({
    $core.Iterable<ProfileHistory>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  ProfileHistoryList._();

  factory ProfileHistoryList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ProfileHistoryList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ProfileHistoryList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<ProfileHistory>(1, _omitFieldNames ? '' : 'values',
        subBuilder: ProfileHistory.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProfileHistoryList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProfileHistoryList copyWith(void Function(ProfileHistoryList) updates) =>
      super.copyWith((message) => updates(message as ProfileHistoryList))
          as ProfileHistoryList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ProfileHistoryList create() => ProfileHistoryList._();
  @$core.override
  ProfileHistoryList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ProfileHistoryList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProfileHistoryList>(create);
  static ProfileHistoryList? _defaultInstance;

  /// Profile histories in normalized name order.
  @$pb.TagNumber(1)
  $pb.PbList<ProfileHistory> get values => $_getList(0);
}

/// Metadata for one secret cached by the session agent.
class AgentEntry extends $pb.GeneratedMessage {
  factory AgentEntry({
    $core.String? id,
    $core.String? path,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (path != null) result.path = path;
    return result;
  }

  AgentEntry._();

  factory AgentEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AgentEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AgentEntry',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'path')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AgentEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AgentEntry copyWith(void Function(AgentEntry) updates) =>
      super.copyWith((message) => updates(message as AgentEntry)) as AgentEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AgentEntry create() => AgentEntry._();
  @$core.override
  AgentEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AgentEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AgentEntry>(create);
  static AgentEntry? _defaultInstance;

  /// Cache key or lockbox identifier.
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// Associated lockbox path when supplied by the client.
  @$pb.TagNumber(2)
  $core.String get path => $_getSZ(1);
  @$pb.TagNumber(2)
  set path($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPath() => $_clearField(2);
}

/// A collection of session-agent cache entries.
class AgentEntryList extends $pb.GeneratedMessage {
  factory AgentEntryList({
    $core.Iterable<AgentEntry>? values,
  }) {
    final result = create();
    if (values != null) result.values.addAll(values);
    return result;
  }

  AgentEntryList._();

  factory AgentEntryList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AgentEntryList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AgentEntryList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..pPM<AgentEntry>(1, _omitFieldNames ? '' : 'values',
        subBuilder: AgentEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AgentEntryList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AgentEntryList copyWith(void Function(AgentEntryList) updates) =>
      super.copyWith((message) => updates(message as AgentEntryList))
          as AgentEntryList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AgentEntryList create() => AgentEntryList._();
  @$core.override
  AgentEntryList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AgentEntryList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AgentEntryList>(create);
  static AgentEntryList? _defaultInstance;

  /// Current cache entries without their secret values.
  @$pb.TagNumber(1)
  $pb.PbList<AgentEntry> get values => $_getList(0);
}

/// Platform capabilities used to protect secrets across system sleep.
class SleepSupport extends $pb.GeneratedMessage {
  factory SleepSupport({
    $core.bool? suspendNotifications,
    $core.bool? sleepInhibition,
    $core.bool? supported,
  }) {
    final result = create();
    if (suspendNotifications != null)
      result.suspendNotifications = suspendNotifications;
    if (sleepInhibition != null) result.sleepInhibition = sleepInhibition;
    if (supported != null) result.supported = supported;
    return result;
  }

  SleepSupport._();

  factory SleepSupport.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SleepSupport.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SleepSupport',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'suspendNotifications')
    ..aOB(2, _omitFieldNames ? '' : 'sleepInhibition')
    ..aOB(3, _omitFieldNames ? '' : 'supported')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SleepSupport clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SleepSupport copyWith(void Function(SleepSupport) updates) =>
      super.copyWith((message) => updates(message as SleepSupport))
          as SleepSupport;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SleepSupport create() => SleepSupport._();
  @$core.override
  SleepSupport createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SleepSupport getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SleepSupport>(create);
  static SleepSupport? _defaultInstance;

  /// Whether the platform reports suspend and resume transitions.
  @$pb.TagNumber(1)
  $core.bool get suspendNotifications => $_getBF(0);
  @$pb.TagNumber(1)
  set suspendNotifications($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuspendNotifications() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuspendNotifications() => $_clearField(1);

  /// Whether the agent can temporarily inhibit system sleep.
  @$pb.TagNumber(2)
  $core.bool get sleepInhibition => $_getBF(1);
  @$pb.TagNumber(2)
  set sleepInhibition($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSleepInhibition() => $_has(1);
  @$pb.TagNumber(2)
  void clearSleepInhibition() => $_clearField(2);

  /// Whether the configured protection policy is supported.
  @$pb.TagNumber(3)
  $core.bool get supported => $_getBF(2);
  @$pb.TagNumber(3)
  set supported($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSupported() => $_has(2);
  @$pb.TagNumber(3)
  void clearSupported() => $_clearField(3);
}

/// Effective operating-system secret-store configuration.
class PlatformStatus extends $pb.GeneratedMessage {
  factory PlatformStatus({
    $core.bool? supported,
    $core.bool? disabled,
    $core.String? scope,
    $core.String? backend,
    $core.String? item,
  }) {
    final result = create();
    if (supported != null) result.supported = supported;
    if (disabled != null) result.disabled = disabled;
    if (scope != null) result.scope = scope;
    if (backend != null) result.backend = backend;
    if (item != null) result.item = item;
    return result;
  }

  PlatformStatus._();

  factory PlatformStatus.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PlatformStatus.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PlatformStatus',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'supported')
    ..aOB(2, _omitFieldNames ? '' : 'disabled')
    ..aOS(3, _omitFieldNames ? '' : 'scope')
    ..aOS(4, _omitFieldNames ? '' : 'backend')
    ..aOS(5, _omitFieldNames ? '' : 'item')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PlatformStatus clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PlatformStatus copyWith(void Function(PlatformStatus) updates) =>
      super.copyWith((message) => updates(message as PlatformStatus))
          as PlatformStatus;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PlatformStatus create() => PlatformStatus._();
  @$core.override
  PlatformStatus createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PlatformStatus getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PlatformStatus>(create);
  static PlatformStatus? _defaultInstance;

  /// Whether this build has a backend for the current operating system.
  @$pb.TagNumber(1)
  $core.bool get supported => $_getBF(0);
  @$pb.TagNumber(1)
  set supported($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSupported() => $_has(0);
  @$pb.TagNumber(1)
  void clearSupported() => $_clearField(1);

  /// Whether secret-store integration is currently disabled.
  @$pb.TagNumber(2)
  $core.bool get disabled => $_getBF(1);
  @$pb.TagNumber(2)
  set disabled($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisabled() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisabled() => $_clearField(2);

  /// Effective automatic-open scope: `off`, `vault`, or `lockboxes`.
  @$pb.TagNumber(3)
  $core.String get scope => $_getSZ(2);
  @$pb.TagNumber(3)
  set scope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearScope() => $_clearField(3);

  /// Human-readable platform backend name.
  @$pb.TagNumber(4)
  $core.String get backend => $_getSZ(3);
  @$pb.TagNumber(4)
  set backend($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasBackend() => $_has(3);
  @$pb.TagNumber(4)
  void clearBackend() => $_clearField(4);

  /// Key used for the default local-vault item.
  @$pb.TagNumber(5)
  $core.String get item => $_getSZ(4);
  @$pb.TagNumber(5)
  set item($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasItem() => $_has(4);
  @$pb.TagNumber(5)
  void clearItem() => $_clearField(5);
}

/// Required UTF-8 string result.
class StringValue extends $pb.GeneratedMessage {
  factory StringValue({
    $core.String? value,
  }) {
    final result = create();
    if (value != null) result.value = value;
    return result;
  }

  StringValue._();

  factory StringValue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StringValue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StringValue',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringValue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringValue copyWith(void Function(StringValue) updates) =>
      super.copyWith((message) => updates(message as StringValue))
          as StringValue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StringValue create() => StringValue._();
  @$core.override
  StringValue createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StringValue getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StringValue>(create);
  static StringValue? _defaultInstance;

  /// Returned UTF-8 value.
  @$pb.TagNumber(1)
  $core.String get value => $_getSZ(0);
  @$pb.TagNumber(1)
  set value($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => $_clearField(1);
}

/// Metadata describing an encrypted local-vault backup archive.
class VaultBackupManifest extends $pb.GeneratedMessage {
  factory VaultBackupManifest({
    $core.int? formatVersion,
    $fixnum.Int64? createdAtUnixMs,
    $core.String? vaultFileName,
    $fixnum.Int64? vaultSize,
    $core.String? vaultSha256,
  }) {
    final result = create();
    if (formatVersion != null) result.formatVersion = formatVersion;
    if (createdAtUnixMs != null) result.createdAtUnixMs = createdAtUnixMs;
    if (vaultFileName != null) result.vaultFileName = vaultFileName;
    if (vaultSize != null) result.vaultSize = vaultSize;
    if (vaultSha256 != null) result.vaultSha256 = vaultSha256;
    return result;
  }

  VaultBackupManifest._();

  factory VaultBackupManifest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VaultBackupManifest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VaultBackupManifest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'formatVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'createdAtUnixMs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'vaultFileName')
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'vaultSize', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(5, _omitFieldNames ? '' : 'vaultSha256')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VaultBackupManifest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VaultBackupManifest copyWith(void Function(VaultBackupManifest) updates) =>
      super.copyWith((message) => updates(message as VaultBackupManifest))
          as VaultBackupManifest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VaultBackupManifest create() => VaultBackupManifest._();
  @$core.override
  VaultBackupManifest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VaultBackupManifest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VaultBackupManifest>(create);
  static VaultBackupManifest? _defaultInstance;

  /// Backup container format version.
  @$pb.TagNumber(1)
  $core.int get formatVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set formatVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFormatVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearFormatVersion() => $_clearField(1);

  /// Backup creation time in Unix milliseconds.
  @$pb.TagNumber(2)
  $fixnum.Int64 get createdAtUnixMs => $_getI64(1);
  @$pb.TagNumber(2)
  set createdAtUnixMs($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCreatedAtUnixMs() => $_has(1);
  @$pb.TagNumber(2)
  void clearCreatedAtUnixMs() => $_clearField(2);

  /// Name of the encrypted vault file inside the archive.
  @$pb.TagNumber(3)
  $core.String get vaultFileName => $_getSZ(2);
  @$pb.TagNumber(3)
  set vaultFileName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasVaultFileName() => $_has(2);
  @$pb.TagNumber(3)
  void clearVaultFileName() => $_clearField(3);

  /// Encrypted vault file length in bytes.
  @$pb.TagNumber(4)
  $fixnum.Int64 get vaultSize => $_getI64(3);
  @$pb.TagNumber(4)
  set vaultSize($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVaultSize() => $_has(3);
  @$pb.TagNumber(4)
  void clearVaultSize() => $_clearField(4);

  /// Lowercase hexadecimal SHA-256 of the encrypted vault file.
  @$pb.TagNumber(5)
  $core.String get vaultSha256 => $_getSZ(4);
  @$pb.TagNumber(5)
  set vaultSha256($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasVaultSha256() => $_has(4);
  @$pb.TagNumber(5)
  void clearVaultSha256() => $_clearField(5);
}

/// Structured diagnostic for the most recent failed binding operation.
class ErrorDetails extends $pb.GeneratedMessage {
  factory ErrorDetails({
    $core.String? category,
    $core.String? artifactKind,
    $core.int? foundVersion,
    $core.int? supportedVersion,
    $core.String? message,
    $core.String? guidance,
  }) {
    final result = create();
    if (category != null) result.category = category;
    if (artifactKind != null) result.artifactKind = artifactKind;
    if (foundVersion != null) result.foundVersion = foundVersion;
    if (supportedVersion != null) result.supportedVersion = supportedVersion;
    if (message != null) result.message = message;
    if (guidance != null) result.guidance = guidance;
    return result;
  }

  ErrorDetails._();

  factory ErrorDetails.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ErrorDetails.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ErrorDetails',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'revault.bindings'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'category')
    ..aOS(2, _omitFieldNames ? '' : 'artifactKind')
    ..aI(3, _omitFieldNames ? '' : 'foundVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'supportedVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(5, _omitFieldNames ? '' : 'message')
    ..aOS(6, _omitFieldNames ? '' : 'guidance')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ErrorDetails clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ErrorDetails copyWith(void Function(ErrorDetails) updates) =>
      super.copyWith((message) => updates(message as ErrorDetails))
          as ErrorDetails;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ErrorDetails create() => ErrorDetails._();
  @$core.override
  ErrorDetails createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ErrorDetails getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ErrorDetails>(create);
  static ErrorDetails? _defaultInstance;

  /// Stable error category suitable for programmatic handling.
  @$pb.TagNumber(1)
  $core.String get category => $_getSZ(0);
  @$pb.TagNumber(1)
  set category($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategory() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategory() => $_clearField(1);

  /// Artifact type involved in a format-version error, when applicable.
  @$pb.TagNumber(2)
  $core.String get artifactKind => $_getSZ(1);
  @$pb.TagNumber(2)
  set artifactKind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasArtifactKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearArtifactKind() => $_clearField(2);

  /// Version found in the input artifact.
  @$pb.TagNumber(3)
  $core.int get foundVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set foundVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFoundVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearFoundVersion() => $_clearField(3);

  /// Highest version supported by this binding.
  @$pb.TagNumber(4)
  $core.int get supportedVersion => $_getIZ(3);
  @$pb.TagNumber(4)
  set supportedVersion($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSupportedVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearSupportedVersion() => $_clearField(4);

  /// Human-readable failure message.
  @$pb.TagNumber(5)
  $core.String get message => $_getSZ(4);
  @$pb.TagNumber(5)
  set message($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMessage() => $_has(4);
  @$pb.TagNumber(5)
  void clearMessage() => $_clearField(5);

  /// Suggested corrective action for the user.
  @$pb.TagNumber(6)
  $core.String get guidance => $_getSZ(5);
  @$pb.TagNumber(6)
  set guidance($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasGuidance() => $_has(5);
  @$pb.TagNumber(6)
  void clearGuidance() => $_clearField(6);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
