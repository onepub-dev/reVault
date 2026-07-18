// This is a generated file - do not edit.
//
// Generated from revault_bindings.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class LockboxEntry_Kind extends $pb.ProtobufEnum {
  static const LockboxEntry_Kind KIND_UNSPECIFIED =
      LockboxEntry_Kind._(0, _omitEnumNames ? '' : 'KIND_UNSPECIFIED');
  static const LockboxEntry_Kind FILE =
      LockboxEntry_Kind._(1, _omitEnumNames ? '' : 'FILE');
  static const LockboxEntry_Kind SYMLINK =
      LockboxEntry_Kind._(2, _omitEnumNames ? '' : 'SYMLINK');
  static const LockboxEntry_Kind DIRECTORY =
      LockboxEntry_Kind._(3, _omitEnumNames ? '' : 'DIRECTORY');

  static const $core.List<LockboxEntry_Kind> values = <LockboxEntry_Kind>[
    KIND_UNSPECIFIED,
    FILE,
    SYMLINK,
    DIRECTORY,
  ];

  static final $core.List<LockboxEntry_Kind?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static LockboxEntry_Kind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const LockboxEntry_Kind._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
