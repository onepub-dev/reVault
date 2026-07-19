// This is a generated file - do not edit.
//
// Generated from revault_bindings.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import
// ignore_for_file: public_member_api_docs

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use lockboxEntryDescriptor instead')
const LockboxEntry$json = {
  '1': 'LockboxEntry',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'kind',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.revault.bindings.LockboxEntry.Kind',
      '10': 'kind'
    },
    {'1': 'length', '3': 3, '4': 1, '5': 4, '10': 'length'},
    {'1': 'permissions', '3': 4, '4': 1, '5': 13, '10': 'permissions'},
  ],
  '4': [LockboxEntry_Kind$json],
};

@$core.Deprecated('Use lockboxEntryDescriptor instead')
const LockboxEntry_Kind$json = {
  '1': 'Kind',
  '2': [
    {'1': 'KIND_UNSPECIFIED', '2': 0},
    {'1': 'FILE', '2': 1},
    {'1': 'SYMLINK', '2': 2},
    {'1': 'DIRECTORY', '2': 3},
  ],
};

/// Descriptor for `LockboxEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List lockboxEntryDescriptor = $convert.base64Decode(
    'CgxMb2NrYm94RW50cnkSEgoEcGF0aBgBIAEoCVIEcGF0aBI3CgRraW5kGAIgASgOMiMucmV2YX'
    'VsdC5iaW5kaW5ncy5Mb2NrYm94RW50cnkuS2luZFIEa2luZBIWCgZsZW5ndGgYAyABKARSBmxl'
    'bmd0aBIgCgtwZXJtaXNzaW9ucxgEIAEoDVILcGVybWlzc2lvbnMiQgoES2luZBIUChBLSU5EX1'
    'VOU1BFQ0lGSUVEEAASCAoERklMRRABEgsKB1NZTUxJTksQAhINCglESVJFQ1RPUlkQAw==');

@$core.Deprecated('Use lockboxEntryListDescriptor instead')
const LockboxEntryList$json = {
  '1': 'LockboxEntryList',
  '2': [
    {
      '1': 'entries',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.LockboxEntry',
      '10': 'entries'
    },
  ],
};

/// Descriptor for `LockboxEntryList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List lockboxEntryListDescriptor = $convert.base64Decode(
    'ChBMb2NrYm94RW50cnlMaXN0EjgKB2VudHJpZXMYASADKAsyHi5yZXZhdWx0LmJpbmRpbmdzLk'
    'xvY2tib3hFbnRyeVIHZW50cmllcw==');

@$core.Deprecated('Use optionalLockboxEntryDescriptor instead')
const OptionalLockboxEntry$json = {
  '1': 'OptionalLockboxEntry',
  '2': [
    {
      '1': 'value',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.revault.bindings.LockboxEntry',
      '10': 'value'
    },
  ],
};

/// Descriptor for `OptionalLockboxEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List optionalLockboxEntryDescriptor = $convert.base64Decode(
    'ChRPcHRpb25hbExvY2tib3hFbnRyeRI0CgV2YWx1ZRgBIAEoCzIeLnJldmF1bHQuYmluZGluZ3'
    'MuTG9ja2JveEVudHJ5UgV2YWx1ZQ==');

@$core.Deprecated('Use stringListDescriptor instead')
const StringList$json = {
  '1': 'StringList',
  '2': [
    {'1': 'values', '3': 1, '4': 3, '5': 9, '10': 'values'},
  ],
};

/// Descriptor for `StringList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List stringListDescriptor =
    $convert.base64Decode('CgpTdHJpbmdMaXN0EhYKBnZhbHVlcxgBIAMoCVIGdmFsdWVz');

@$core.Deprecated('Use pathMoveDescriptor instead')
const PathMove$json = {
  '1': 'PathMove',
  '2': [
    {'1': 'source', '3': 1, '4': 1, '5': 9, '10': 'source'},
    {'1': 'destination', '3': 2, '4': 1, '5': 9, '10': 'destination'},
  ],
};

/// Descriptor for `PathMove`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pathMoveDescriptor = $convert.base64Decode(
    'CghQYXRoTW92ZRIWCgZzb3VyY2UYASABKAlSBnNvdXJjZRIgCgtkZXN0aW5hdGlvbhgCIAEoCV'
    'ILZGVzdGluYXRpb24=');

@$core.Deprecated('Use pathMoveListDescriptor instead')
const PathMoveList$json = {
  '1': 'PathMoveList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.PathMove',
      '10': 'values'
    },
  ],
};

/// Descriptor for `PathMoveList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pathMoveListDescriptor = $convert.base64Decode(
    'CgxQYXRoTW92ZUxpc3QSMgoGdmFsdWVzGAEgAygLMhoucmV2YXVsdC5iaW5kaW5ncy5QYXRoTW'
    '92ZVIGdmFsdWVz');

@$core.Deprecated('Use byteListDescriptor instead')
const ByteList$json = {
  '1': 'ByteList',
  '2': [
    {'1': 'values', '3': 1, '4': 3, '5': 12, '10': 'values'},
  ],
};

/// Descriptor for `ByteList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List byteListDescriptor =
    $convert.base64Decode('CghCeXRlTGlzdBIWCgZ2YWx1ZXMYASADKAxSBnZhbHVlcw==');

@$core.Deprecated('Use formFieldDescriptor instead')
const FormField$json = {
  '1': 'FormField',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'label', '3': 2, '4': 1, '5': 9, '10': 'label'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'required', '3': 4, '4': 1, '5': 8, '10': 'required'},
  ],
};

/// Descriptor for `FormField`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formFieldDescriptor = $convert.base64Decode(
    'CglGb3JtRmllbGQSDgoCaWQYASABKAlSAmlkEhQKBWxhYmVsGAIgASgJUgVsYWJlbBISCgRraW'
    '5kGAMgASgJUgRraW5kEhoKCHJlcXVpcmVkGAQgASgIUghyZXF1aXJlZA==');

@$core.Deprecated('Use formFieldListDescriptor instead')
const FormFieldList$json = {
  '1': 'FormFieldList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.FormField',
      '10': 'values'
    },
  ],
};

/// Descriptor for `FormFieldList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formFieldListDescriptor = $convert.base64Decode(
    'Cg1Gb3JtRmllbGRMaXN0EjMKBnZhbHVlcxgBIAMoCzIbLnJldmF1bHQuYmluZGluZ3MuRm9ybU'
    'ZpZWxkUgZ2YWx1ZXM=');

@$core.Deprecated('Use formDefinitionDescriptor instead')
const FormDefinition$json = {
  '1': 'FormDefinition',
  '2': [
    {'1': 'type_id', '3': 1, '4': 1, '5': 9, '10': 'typeId'},
    {'1': 'alias', '3': 2, '4': 1, '5': 9, '10': 'alias'},
    {'1': 'revision', '3': 3, '4': 1, '5': 13, '10': 'revision'},
    {'1': 'name', '3': 4, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 5, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'fields',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.FormField',
      '10': 'fields'
    },
  ],
};

/// Descriptor for `FormDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formDefinitionDescriptor = $convert.base64Decode(
    'Cg5Gb3JtRGVmaW5pdGlvbhIXCgd0eXBlX2lkGAEgASgJUgZ0eXBlSWQSFAoFYWxpYXMYAiABKA'
    'lSBWFsaWFzEhoKCHJldmlzaW9uGAMgASgNUghyZXZpc2lvbhISCgRuYW1lGAQgASgJUgRuYW1l'
    'EiAKC2Rlc2NyaXB0aW9uGAUgASgJUgtkZXNjcmlwdGlvbhIzCgZmaWVsZHMYBiADKAsyGy5yZX'
    'ZhdWx0LmJpbmRpbmdzLkZvcm1GaWVsZFIGZmllbGRz');

@$core.Deprecated('Use formDefinitionListDescriptor instead')
const FormDefinitionList$json = {
  '1': 'FormDefinitionList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.FormDefinition',
      '10': 'values'
    },
  ],
};

/// Descriptor for `FormDefinitionList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formDefinitionListDescriptor = $convert.base64Decode(
    'ChJGb3JtRGVmaW5pdGlvbkxpc3QSOAoGdmFsdWVzGAEgAygLMiAucmV2YXVsdC5iaW5kaW5ncy'
    '5Gb3JtRGVmaW5pdGlvblIGdmFsdWVz');

@$core.Deprecated('Use formValueDescriptor instead')
const FormValue$json = {
  '1': 'FormValue',
  '2': [
    {'1': 'field_id', '3': 1, '4': 1, '5': 9, '10': 'fieldId'},
    {'1': 'label', '3': 2, '4': 1, '5': 9, '10': 'label'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'value', '3': 4, '4': 1, '5': 9, '10': 'value'},
    {'1': 'secret', '3': 5, '4': 1, '5': 8, '10': 'secret'},
  ],
};

/// Descriptor for `FormValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formValueDescriptor = $convert.base64Decode(
    'CglGb3JtVmFsdWUSGQoIZmllbGRfaWQYASABKAlSB2ZpZWxkSWQSFAoFbGFiZWwYAiABKAlSBW'
    'xhYmVsEhIKBGtpbmQYAyABKAlSBGtpbmQSFAoFdmFsdWUYBCABKAlSBXZhbHVlEhYKBnNlY3Jl'
    'dBgFIAEoCFIGc2VjcmV0');

@$core.Deprecated('Use formRecordDescriptor instead')
const FormRecord$json = {
  '1': 'FormRecord',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'type_id', '3': 3, '4': 1, '5': 9, '10': 'typeId'},
    {'1': 'definition_alias', '3': 4, '4': 1, '5': 9, '10': 'definitionAlias'},
    {
      '1': 'definition_revision',
      '3': 5,
      '4': 1,
      '5': 13,
      '10': 'definitionRevision'
    },
    {
      '1': 'values',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.FormValue',
      '10': 'values'
    },
  ],
};

/// Descriptor for `FormRecord`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formRecordDescriptor = $convert.base64Decode(
    'CgpGb3JtUmVjb3JkEhIKBHBhdGgYASABKAlSBHBhdGgSEgoEbmFtZRgCIAEoCVIEbmFtZRIXCg'
    'd0eXBlX2lkGAMgASgJUgZ0eXBlSWQSKQoQZGVmaW5pdGlvbl9hbGlhcxgEIAEoCVIPZGVmaW5p'
    'dGlvbkFsaWFzEi8KE2RlZmluaXRpb25fcmV2aXNpb24YBSABKA1SEmRlZmluaXRpb25SZXZpc2'
    'lvbhIzCgZ2YWx1ZXMYBiADKAsyGy5yZXZhdWx0LmJpbmRpbmdzLkZvcm1WYWx1ZVIGdmFsdWVz');

@$core.Deprecated('Use formRecordListDescriptor instead')
const FormRecordList$json = {
  '1': 'FormRecordList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.FormRecord',
      '10': 'values'
    },
  ],
};

/// Descriptor for `FormRecordList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formRecordListDescriptor = $convert.base64Decode(
    'Cg5Gb3JtUmVjb3JkTGlzdBI0CgZ2YWx1ZXMYASADKAsyHC5yZXZhdWx0LmJpbmRpbmdzLkZvcm'
    '1SZWNvcmRSBnZhbHVlcw==');

@$core.Deprecated('Use optionalFormRecordDescriptor instead')
const OptionalFormRecord$json = {
  '1': 'OptionalFormRecord',
  '2': [
    {
      '1': 'value',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.revault.bindings.FormRecord',
      '10': 'value'
    },
  ],
};

/// Descriptor for `OptionalFormRecord`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List optionalFormRecordDescriptor = $convert.base64Decode(
    'ChJPcHRpb25hbEZvcm1SZWNvcmQSMgoFdmFsdWUYASABKAsyHC5yZXZhdWx0LmJpbmRpbmdzLk'
    'Zvcm1SZWNvcmRSBXZhbHVl');

@$core.Deprecated('Use optionalFormValueDescriptor instead')
const OptionalFormValue$json = {
  '1': 'OptionalFormValue',
  '2': [
    {
      '1': 'value',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.revault.bindings.FormValue',
      '10': 'value'
    },
  ],
};

/// Descriptor for `OptionalFormValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List optionalFormValueDescriptor = $convert.base64Decode(
    'ChFPcHRpb25hbEZvcm1WYWx1ZRIxCgV2YWx1ZRgBIAEoCzIbLnJldmF1bHQuYmluZGluZ3MuRm'
    '9ybVZhbHVlUgV2YWx1ZQ==');

@$core.Deprecated('Use recoveryReportDescriptor instead')
const RecoveryReport$json = {
  '1': 'RecoveryReport',
  '2': [
    {
      '1': 'intact_files',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.LockboxEntry',
      '10': 'intactFiles'
    },
    {'1': 'intact_file_count', '3': 2, '4': 1, '5': 4, '10': 'intactFileCount'},
    {'1': 'partial_files', '3': 3, '4': 1, '5': 4, '10': 'partialFiles'},
    {'1': 'corrupt_records', '3': 4, '4': 1, '5': 4, '10': 'corruptRecords'},
    {'1': 'toc_recovered', '3': 5, '4': 1, '5': 8, '10': 'tocRecovered'},
    {
      '1': 'variables_recovered',
      '3': 6,
      '4': 1,
      '5': 8,
      '10': 'variablesRecovered'
    },
    {'1': 'variable_count', '3': 7, '4': 1, '5': 4, '10': 'variableCount'},
    {'1': 'forms_recovered', '3': 8, '4': 1, '5': 8, '10': 'formsRecovered'},
    {
      '1': 'form_definition_count',
      '3': 9,
      '4': 1,
      '5': 4,
      '10': 'formDefinitionCount'
    },
    {
      '1': 'form_record_count',
      '3': 10,
      '4': 1,
      '5': 4,
      '10': 'formRecordCount'
    },
  ],
};

/// Descriptor for `RecoveryReport`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List recoveryReportDescriptor = $convert.base64Decode(
    'Cg5SZWNvdmVyeVJlcG9ydBJBCgxpbnRhY3RfZmlsZXMYASADKAsyHi5yZXZhdWx0LmJpbmRpbm'
    'dzLkxvY2tib3hFbnRyeVILaW50YWN0RmlsZXMSKgoRaW50YWN0X2ZpbGVfY291bnQYAiABKARS'
    'D2ludGFjdEZpbGVDb3VudBIjCg1wYXJ0aWFsX2ZpbGVzGAMgASgEUgxwYXJ0aWFsRmlsZXMSJw'
    'oPY29ycnVwdF9yZWNvcmRzGAQgASgEUg5jb3JydXB0UmVjb3JkcxIjCg10b2NfcmVjb3ZlcmVk'
    'GAUgASgIUgx0b2NSZWNvdmVyZWQSLwoTdmFyaWFibGVzX3JlY292ZXJlZBgGIAEoCFISdmFyaW'
    'FibGVzUmVjb3ZlcmVkEiUKDnZhcmlhYmxlX2NvdW50GAcgASgEUg12YXJpYWJsZUNvdW50EicK'
    'D2Zvcm1zX3JlY292ZXJlZBgIIAEoCFIOZm9ybXNSZWNvdmVyZWQSMgoVZm9ybV9kZWZpbml0aW'
    '9uX2NvdW50GAkgASgEUhNmb3JtRGVmaW5pdGlvbkNvdW50EioKEWZvcm1fcmVjb3JkX2NvdW50'
    'GAogASgEUg9mb3JtUmVjb3JkQ291bnQ=');

@$core.Deprecated('Use keySlotDescriptor instead')
const KeySlot$json = {
  '1': 'KeySlot',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'protection', '3': 2, '4': 1, '5': 9, '10': 'protection'},
    {'1': 'algorithm', '3': 3, '4': 1, '5': 9, '10': 'algorithm'},
  ],
};

/// Descriptor for `KeySlot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List keySlotDescriptor = $convert.base64Decode(
    'CgdLZXlTbG90Eg4KAmlkGAEgASgEUgJpZBIeCgpwcm90ZWN0aW9uGAIgASgJUgpwcm90ZWN0aW'
    '9uEhwKCWFsZ29yaXRobRgDIAEoCVIJYWxnb3JpdGht');

@$core.Deprecated('Use keySlotListDescriptor instead')
const KeySlotList$json = {
  '1': 'KeySlotList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.KeySlot',
      '10': 'values'
    },
  ],
};

/// Descriptor for `KeySlotList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List keySlotListDescriptor = $convert.base64Decode(
    'CgtLZXlTbG90TGlzdBIxCgZ2YWx1ZXMYASADKAsyGS5yZXZhdWx0LmJpbmRpbmdzLktleVNsb3'
    'RSBnZhbHVlcw==');

@$core.Deprecated('Use cacheStatsDescriptor instead')
const CacheStats$json = {
  '1': 'CacheStats',
  '2': [
    {'1': 'limit_bytes', '3': 1, '4': 1, '5': 4, '10': 'limitBytes'},
    {'1': 'used_bytes', '3': 2, '4': 1, '5': 4, '10': 'usedBytes'},
    {'1': 'entries', '3': 3, '4': 1, '5': 4, '10': 'entries'},
    {'1': 'hits', '3': 4, '4': 1, '5': 4, '10': 'hits'},
    {'1': 'misses', '3': 5, '4': 1, '5': 4, '10': 'misses'},
  ],
};

/// Descriptor for `CacheStats`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cacheStatsDescriptor = $convert.base64Decode(
    'CgpDYWNoZVN0YXRzEh8KC2xpbWl0X2J5dGVzGAEgASgEUgpsaW1pdEJ5dGVzEh0KCnVzZWRfYn'
    'l0ZXMYAiABKARSCXVzZWRCeXRlcxIYCgdlbnRyaWVzGAMgASgEUgdlbnRyaWVzEhIKBGhpdHMY'
    'BCABKARSBGhpdHMSFgoGbWlzc2VzGAUgASgEUgZtaXNzZXM=');

@$core.Deprecated('Use importStatsDescriptor instead')
const ImportStats$json = {
  '1': 'ImportStats',
  '2': [
    {'1': 'host_stat_nanos', '3': 1, '4': 1, '5': 9, '10': 'hostStatNanos'},
    {'1': 'host_read_nanos', '3': 2, '4': 1, '5': 9, '10': 'hostReadNanos'},
    {
      '1': 'frame_prepare_nanos',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'framePrepareNanos'
    },
    {'1': 'page_write_nanos', '3': 4, '4': 1, '5': 9, '10': 'pageWriteNanos'},
  ],
};

/// Descriptor for `ImportStats`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importStatsDescriptor = $convert.base64Decode(
    'CgtJbXBvcnRTdGF0cxImCg9ob3N0X3N0YXRfbmFub3MYASABKAlSDWhvc3RTdGF0TmFub3MSJg'
    'oPaG9zdF9yZWFkX25hbm9zGAIgASgJUg1ob3N0UmVhZE5hbm9zEi4KE2ZyYW1lX3ByZXBhcmVf'
    'bmFub3MYAyABKAlSEWZyYW1lUHJlcGFyZU5hbm9zEigKEHBhZ2Vfd3JpdGVfbmFub3MYBCABKA'
    'lSDnBhZ2VXcml0ZU5hbm9z');

@$core.Deprecated('Use pageObjectDescriptor instead')
const PageObject$json = {
  '1': 'PageObject',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'payload_len', '3': 3, '4': 1, '5': 4, '10': 'payloadLen'},
  ],
};

/// Descriptor for `PageObject`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pageObjectDescriptor = $convert.base64Decode(
    'CgpQYWdlT2JqZWN0Eg4KAmlkGAEgASgEUgJpZBISCgRraW5kGAIgASgJUgRraW5kEh8KC3BheW'
    'xvYWRfbGVuGAMgASgEUgpwYXlsb2FkTGVu');

@$core.Deprecated('Use pageInspectionDescriptor instead')
const PageInspection$json = {
  '1': 'PageInspection',
  '2': [
    {'1': 'offset', '3': 1, '4': 1, '5': 4, '10': 'offset'},
    {'1': 'page_id', '3': 2, '4': 1, '5': 4, '10': 'pageId'},
    {'1': 'sequence', '3': 3, '4': 1, '5': 4, '10': 'sequence'},
    {'1': 'page_size', '3': 4, '4': 1, '5': 4, '10': 'pageSize'},
    {
      '1': 'encrypted_body_len',
      '3': 5,
      '4': 1,
      '5': 4,
      '10': 'encryptedBodyLen'
    },
    {'1': 'unused_bytes', '3': 6, '4': 1, '5': 4, '10': 'unusedBytes'},
    {'1': 'object_count', '3': 7, '4': 1, '5': 4, '10': 'objectCount'},
    {
      '1': 'objects',
      '3': 8,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.PageObject',
      '10': 'objects'
    },
  ],
};

/// Descriptor for `PageInspection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pageInspectionDescriptor = $convert.base64Decode(
    'Cg5QYWdlSW5zcGVjdGlvbhIWCgZvZmZzZXQYASABKARSBm9mZnNldBIXCgdwYWdlX2lkGAIgAS'
    'gEUgZwYWdlSWQSGgoIc2VxdWVuY2UYAyABKARSCHNlcXVlbmNlEhsKCXBhZ2Vfc2l6ZRgEIAEo'
    'BFIIcGFnZVNpemUSLAoSZW5jcnlwdGVkX2JvZHlfbGVuGAUgASgEUhBlbmNyeXB0ZWRCb2R5TG'
    'VuEiEKDHVudXNlZF9ieXRlcxgGIAEoBFILdW51c2VkQnl0ZXMSIQoMb2JqZWN0X2NvdW50GAcg'
    'ASgEUgtvYmplY3RDb3VudBI2CgdvYmplY3RzGAggAygLMhwucmV2YXVsdC5iaW5kaW5ncy5QYW'
    'dlT2JqZWN0UgdvYmplY3Rz');

@$core.Deprecated('Use pageInspectionListDescriptor instead')
const PageInspectionList$json = {
  '1': 'PageInspectionList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.PageInspection',
      '10': 'values'
    },
  ],
};

/// Descriptor for `PageInspectionList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pageInspectionListDescriptor = $convert.base64Decode(
    'ChJQYWdlSW5zcGVjdGlvbkxpc3QSOAoGdmFsdWVzGAEgAygLMiAucmV2YXVsdC5iaW5kaW5ncy'
    '5QYWdlSW5zcGVjdGlvblIGdmFsdWVz');

@$core.Deprecated('Use fileInspectionDescriptor instead')
const FileInspection$json = {
  '1': 'FileInspection',
  '2': [
    {'1': 'lockbox_id', '3': 1, '4': 1, '5': 12, '10': 'lockboxId'},
    {'1': 'header_readable', '3': 2, '4': 1, '5': 8, '10': 'headerReadable'},
    {
      '1': 'key_directory_generation',
      '3': 3,
      '4': 1,
      '5': 4,
      '10': 'keyDirectoryGeneration'
    },
    {
      '1': 'key_directory_copy_count',
      '3': 4,
      '4': 1,
      '5': 4,
      '10': 'keyDirectoryCopyCount'
    },
    {'1': 'owner_signed', '3': 5, '4': 1, '5': 8, '10': 'ownerSigned'},
    {
      '1': 'key_slots',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.KeySlot',
      '10': 'keySlots'
    },
  ],
};

/// Descriptor for `FileInspection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List fileInspectionDescriptor = $convert.base64Decode(
    'Cg5GaWxlSW5zcGVjdGlvbhIdCgpsb2NrYm94X2lkGAEgASgMUglsb2NrYm94SWQSJwoPaGVhZG'
    'VyX3JlYWRhYmxlGAIgASgIUg5oZWFkZXJSZWFkYWJsZRI4ChhrZXlfZGlyZWN0b3J5X2dlbmVy'
    'YXRpb24YAyABKARSFmtleURpcmVjdG9yeUdlbmVyYXRpb24SNwoYa2V5X2RpcmVjdG9yeV9jb3'
    'B5X2NvdW50GAQgASgEUhVrZXlEaXJlY3RvcnlDb3B5Q291bnQSIQoMb3duZXJfc2lnbmVkGAUg'
    'ASgIUgtvd25lclNpZ25lZBI2CglrZXlfc2xvdHMYBiADKAsyGS5yZXZhdWx0LmJpbmRpbmdzLk'
    'tleVNsb3RSCGtleVNsb3Rz');

@$core.Deprecated('Use profileGenerationDescriptor instead')
const ProfileGeneration$json = {
  '1': 'ProfileGeneration',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 13, '10': 'index'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'contact_fingerprint',
      '3': 3,
      '4': 1,
      '5': 12,
      '10': 'contactFingerprint'
    },
    {
      '1': 'created_at_unix_ms',
      '3': 4,
      '4': 1,
      '5': 4,
      '10': 'createdAtUnixMs'
    },
    {
      '1': 'retired_at_unix_ms',
      '3': 5,
      '4': 1,
      '5': 4,
      '10': 'retiredAtUnixMs'
    },
    {'1': 'has_retired_at', '3': 6, '4': 1, '5': 8, '10': 'hasRetiredAt'},
  ],
};

/// Descriptor for `ProfileGeneration`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List profileGenerationDescriptor = $convert.base64Decode(
    'ChFQcm9maWxlR2VuZXJhdGlvbhIUCgVpbmRleBgBIAEoDVIFaW5kZXgSFgoGc3RhdHVzGAIgAS'
    'gJUgZzdGF0dXMSLwoTY29udGFjdF9maW5nZXJwcmludBgDIAEoDFISY29udGFjdEZpbmdlcnBy'
    'aW50EisKEmNyZWF0ZWRfYXRfdW5peF9tcxgEIAEoBFIPY3JlYXRlZEF0VW5peE1zEisKEnJldG'
    'lyZWRfYXRfdW5peF9tcxgFIAEoBFIPcmV0aXJlZEF0VW5peE1zEiQKDmhhc19yZXRpcmVkX2F0'
    'GAYgASgIUgxoYXNSZXRpcmVkQXQ=');

@$core.Deprecated('Use profileHistoryDescriptor instead')
const ProfileHistory$json = {
  '1': 'ProfileHistory',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'active_generation',
      '3': 2,
      '4': 1,
      '5': 13,
      '10': 'activeGeneration'
    },
    {
      '1': 'generations',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.ProfileGeneration',
      '10': 'generations'
    },
  ],
};

/// Descriptor for `ProfileHistory`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List profileHistoryDescriptor = $convert.base64Decode(
    'Cg5Qcm9maWxlSGlzdG9yeRISCgRuYW1lGAEgASgJUgRuYW1lEisKEWFjdGl2ZV9nZW5lcmF0aW'
    '9uGAIgASgNUhBhY3RpdmVHZW5lcmF0aW9uEkUKC2dlbmVyYXRpb25zGAMgAygLMiMucmV2YXVs'
    'dC5iaW5kaW5ncy5Qcm9maWxlR2VuZXJhdGlvblILZ2VuZXJhdGlvbnM=');

@$core.Deprecated('Use knownLockboxDescriptor instead')
const KnownLockbox$json = {
  '1': 'KnownLockbox',
  '2': [
    {'1': 'lockbox_id', '3': 1, '4': 1, '5': 12, '10': 'lockboxId'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
    {'1': 'last_seen_unix_ms', '3': 3, '4': 1, '5': 4, '10': 'lastSeenUnixMs'},
  ],
};

/// Descriptor for `KnownLockbox`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List knownLockboxDescriptor = $convert.base64Decode(
    'CgxLbm93bkxvY2tib3gSHQoKbG9ja2JveF9pZBgBIAEoDFIJbG9ja2JveElkEhIKBHBhdGgYAi'
    'ABKAlSBHBhdGgSKQoRbGFzdF9zZWVuX3VuaXhfbXMYAyABKARSDmxhc3RTZWVuVW5peE1z');

@$core.Deprecated('Use knownLockboxListDescriptor instead')
const KnownLockboxList$json = {
  '1': 'KnownLockboxList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.KnownLockbox',
      '10': 'values'
    },
  ],
};

/// Descriptor for `KnownLockboxList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List knownLockboxListDescriptor = $convert.base64Decode(
    'ChBLbm93bkxvY2tib3hMaXN0EjYKBnZhbHVlcxgBIAMoCzIeLnJldmF1bHQuYmluZGluZ3MuS2'
    '5vd25Mb2NrYm94UgZ2YWx1ZXM=');

@$core.Deprecated('Use accessSlotLabelDescriptor instead')
const AccessSlotLabel$json = {
  '1': 'AccessSlotLabel',
  '2': [
    {'1': 'lockbox_id', '3': 1, '4': 1, '5': 12, '10': 'lockboxId'},
    {'1': 'slot_id', '3': 2, '4': 1, '5': 4, '10': 'slotId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'updated_at_unix_ms',
      '3': 4,
      '4': 1,
      '5': 4,
      '10': 'updatedAtUnixMs'
    },
  ],
};

/// Descriptor for `AccessSlotLabel`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List accessSlotLabelDescriptor = $convert.base64Decode(
    'Cg9BY2Nlc3NTbG90TGFiZWwSHQoKbG9ja2JveF9pZBgBIAEoDFIJbG9ja2JveElkEhcKB3Nsb3'
    'RfaWQYAiABKARSBnNsb3RJZBISCgRuYW1lGAMgASgJUgRuYW1lEisKEnVwZGF0ZWRfYXRfdW5p'
    'eF9tcxgEIAEoBFIPdXBkYXRlZEF0VW5peE1z');

@$core.Deprecated('Use accessSlotLabelListDescriptor instead')
const AccessSlotLabelList$json = {
  '1': 'AccessSlotLabelList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.AccessSlotLabel',
      '10': 'values'
    },
  ],
};

/// Descriptor for `AccessSlotLabelList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List accessSlotLabelListDescriptor = $convert.base64Decode(
    'ChNBY2Nlc3NTbG90TGFiZWxMaXN0EjkKBnZhbHVlcxgBIAMoCzIhLnJldmF1bHQuYmluZGluZ3'
    'MuQWNjZXNzU2xvdExhYmVsUgZ2YWx1ZXM=');

@$core.Deprecated('Use streamChunkDescriptor instead')
const StreamChunk$json = {
  '1': 'StreamChunk',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'file_offset', '3': 2, '4': 1, '5': 4, '10': 'fileOffset'},
    {'1': 'length', '3': 3, '4': 1, '5': 4, '10': 'length'},
    {'1': 'physical_offset', '3': 4, '4': 1, '5': 4, '10': 'physicalOffset'},
    {'1': 'sparse', '3': 5, '4': 1, '5': 8, '10': 'sparse'},
    {'1': 'data', '3': 6, '4': 1, '5': 12, '10': 'data'},
  ],
};

/// Descriptor for `StreamChunk`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List streamChunkDescriptor = $convert.base64Decode(
    'CgtTdHJlYW1DaHVuaxISCgRwYXRoGAEgASgJUgRwYXRoEh8KC2ZpbGVfb2Zmc2V0GAIgASgEUg'
    'pmaWxlT2Zmc2V0EhYKBmxlbmd0aBgDIAEoBFIGbGVuZ3RoEicKD3BoeXNpY2FsX29mZnNldBgE'
    'IAEoBFIOcGh5c2ljYWxPZmZzZXQSFgoGc3BhcnNlGAUgASgIUgZzcGFyc2USEgoEZGF0YRgGIA'
    'EoDFIEZGF0YQ==');

@$core.Deprecated('Use streamChunkListDescriptor instead')
const StreamChunkList$json = {
  '1': 'StreamChunkList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.StreamChunk',
      '10': 'values'
    },
  ],
};

/// Descriptor for `StreamChunkList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List streamChunkListDescriptor = $convert.base64Decode(
    'Cg9TdHJlYW1DaHVua0xpc3QSNQoGdmFsdWVzGAEgAygLMh0ucmV2YXVsdC5iaW5kaW5ncy5TdH'
    'JlYW1DaHVua1IGdmFsdWVz');

@$core.Deprecated('Use runtimeOptionsDescriptor instead')
const RuntimeOptions$json = {
  '1': 'RuntimeOptions',
  '2': [
    {'1': 'workload_profile', '3': 1, '4': 1, '5': 9, '10': 'workloadProfile'},
    {'1': 'worker_policy', '3': 2, '4': 1, '5': 9, '10': 'workerPolicy'},
  ],
};

/// Descriptor for `RuntimeOptions`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List runtimeOptionsDescriptor = $convert.base64Decode(
    'Cg5SdW50aW1lT3B0aW9ucxIpChB3b3JrbG9hZF9wcm9maWxlGAEgASgJUg93b3JrbG9hZFByb2'
    'ZpbGUSIwoNd29ya2VyX3BvbGljeRgCIAEoCVIMd29ya2VyUG9saWN5');

@$core.Deprecated('Use variableDescriptor instead')
const Variable$json = {
  '1': 'Variable',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'sensitivity', '3': 2, '4': 1, '5': 9, '10': 'sensitivity'},
  ],
};

/// Descriptor for `Variable`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List variableDescriptor = $convert.base64Decode(
    'CghWYXJpYWJsZRISCgRuYW1lGAEgASgJUgRuYW1lEiAKC3NlbnNpdGl2aXR5GAIgASgJUgtzZW'
    '5zaXRpdml0eQ==');

@$core.Deprecated('Use variableListDescriptor instead')
const VariableList$json = {
  '1': 'VariableList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.Variable',
      '10': 'values'
    },
  ],
};

/// Descriptor for `VariableList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List variableListDescriptor = $convert.base64Decode(
    'CgxWYXJpYWJsZUxpc3QSMgoGdmFsdWVzGAEgAygLMhoucmV2YXVsdC5iaW5kaW5ncy5WYXJpYW'
    'JsZVIGdmFsdWVz');

@$core.Deprecated('Use optionalStringDescriptor instead')
const OptionalString$json = {
  '1': 'OptionalString',
  '2': [
    {'1': 'present', '3': 1, '4': 1, '5': 8, '10': 'present'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `OptionalString`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List optionalStringDescriptor = $convert.base64Decode(
    'Cg5PcHRpb25hbFN0cmluZxIYCgdwcmVzZW50GAEgASgIUgdwcmVzZW50EhQKBXZhbHVlGAIgAS'
    'gJUgV2YWx1ZQ==');

@$core.Deprecated('Use ownerInspectionDescriptor instead')
const OwnerInspection$json = {
  '1': 'OwnerInspection',
  '2': [
    {'1': 'signed', '3': 1, '4': 1, '5': 8, '10': 'signed'},
    {'1': 'fingerprint', '3': 2, '4': 1, '5': 9, '10': 'fingerprint'},
    {'1': 'has_fingerprint', '3': 3, '4': 1, '5': 8, '10': 'hasFingerprint'},
  ],
};

/// Descriptor for `OwnerInspection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List ownerInspectionDescriptor = $convert.base64Decode(
    'Cg9Pd25lckluc3BlY3Rpb24SFgoGc2lnbmVkGAEgASgIUgZzaWduZWQSIAoLZmluZ2VycHJpbn'
    'QYAiABKAlSC2ZpbmdlcnByaW50EicKD2hhc19maW5nZXJwcmludBgDIAEoCFIOaGFzRmluZ2Vy'
    'cHJpbnQ=');

@$core.Deprecated('Use contactDescriptor instead')
const Contact$json = {
  '1': 'Contact',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'key', '3': 2, '4': 1, '5': 12, '10': 'key'},
  ],
};

/// Descriptor for `Contact`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List contactDescriptor = $convert.base64Decode(
    'CgdDb250YWN0EhIKBG5hbWUYASABKAlSBG5hbWUSEAoDa2V5GAIgASgMUgNrZXk=');

@$core.Deprecated('Use contactListDescriptor instead')
const ContactList$json = {
  '1': 'ContactList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.Contact',
      '10': 'values'
    },
  ],
};

/// Descriptor for `ContactList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List contactListDescriptor = $convert.base64Decode(
    'CgtDb250YWN0TGlzdBIxCgZ2YWx1ZXMYASADKAsyGS5yZXZhdWx0LmJpbmRpbmdzLkNvbnRhY3'
    'RSBnZhbHVlcw==');

@$core.Deprecated('Use profileHistoryListDescriptor instead')
const ProfileHistoryList$json = {
  '1': 'ProfileHistoryList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.ProfileHistory',
      '10': 'values'
    },
  ],
};

/// Descriptor for `ProfileHistoryList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List profileHistoryListDescriptor = $convert.base64Decode(
    'ChJQcm9maWxlSGlzdG9yeUxpc3QSOAoGdmFsdWVzGAEgAygLMiAucmV2YXVsdC5iaW5kaW5ncy'
    '5Qcm9maWxlSGlzdG9yeVIGdmFsdWVz');

@$core.Deprecated('Use agentEntryDescriptor instead')
const AgentEntry$json = {
  '1': 'AgentEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
  ],
};

/// Descriptor for `AgentEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List agentEntryDescriptor = $convert.base64Decode(
    'CgpBZ2VudEVudHJ5Eg4KAmlkGAEgASgJUgJpZBISCgRwYXRoGAIgASgJUgRwYXRo');

@$core.Deprecated('Use agentEntryListDescriptor instead')
const AgentEntryList$json = {
  '1': 'AgentEntryList',
  '2': [
    {
      '1': 'values',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.revault.bindings.AgentEntry',
      '10': 'values'
    },
  ],
};

/// Descriptor for `AgentEntryList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List agentEntryListDescriptor = $convert.base64Decode(
    'Cg5BZ2VudEVudHJ5TGlzdBI0CgZ2YWx1ZXMYASADKAsyHC5yZXZhdWx0LmJpbmRpbmdzLkFnZW'
    '50RW50cnlSBnZhbHVlcw==');

@$core.Deprecated('Use sleepSupportDescriptor instead')
const SleepSupport$json = {
  '1': 'SleepSupport',
  '2': [
    {
      '1': 'suspend_notifications',
      '3': 1,
      '4': 1,
      '5': 8,
      '10': 'suspendNotifications'
    },
    {'1': 'sleep_inhibition', '3': 2, '4': 1, '5': 8, '10': 'sleepInhibition'},
    {'1': 'supported', '3': 3, '4': 1, '5': 8, '10': 'supported'},
  ],
};

/// Descriptor for `SleepSupport`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sleepSupportDescriptor = $convert.base64Decode(
    'CgxTbGVlcFN1cHBvcnQSMwoVc3VzcGVuZF9ub3RpZmljYXRpb25zGAEgASgIUhRzdXNwZW5kTm'
    '90aWZpY2F0aW9ucxIpChBzbGVlcF9pbmhpYml0aW9uGAIgASgIUg9zbGVlcEluaGliaXRpb24S'
    'HAoJc3VwcG9ydGVkGAMgASgIUglzdXBwb3J0ZWQ=');

@$core.Deprecated('Use platformStatusDescriptor instead')
const PlatformStatus$json = {
  '1': 'PlatformStatus',
  '2': [
    {'1': 'supported', '3': 1, '4': 1, '5': 8, '10': 'supported'},
    {'1': 'disabled', '3': 2, '4': 1, '5': 8, '10': 'disabled'},
    {'1': 'scope', '3': 3, '4': 1, '5': 9, '10': 'scope'},
    {'1': 'backend', '3': 4, '4': 1, '5': 9, '10': 'backend'},
    {'1': 'item', '3': 5, '4': 1, '5': 9, '10': 'item'},
  ],
};

/// Descriptor for `PlatformStatus`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List platformStatusDescriptor = $convert.base64Decode(
    'Cg5QbGF0Zm9ybVN0YXR1cxIcCglzdXBwb3J0ZWQYASABKAhSCXN1cHBvcnRlZBIaCghkaXNhYm'
    'xlZBgCIAEoCFIIZGlzYWJsZWQSFAoFc2NvcGUYAyABKAlSBXNjb3BlEhgKB2JhY2tlbmQYBCAB'
    'KAlSB2JhY2tlbmQSEgoEaXRlbRgFIAEoCVIEaXRlbQ==');

@$core.Deprecated('Use stringValueDescriptor instead')
const StringValue$json = {
  '1': 'StringValue',
  '2': [
    {'1': 'value', '3': 1, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `StringValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List stringValueDescriptor =
    $convert.base64Decode('CgtTdHJpbmdWYWx1ZRIUCgV2YWx1ZRgBIAEoCVIFdmFsdWU=');

@$core.Deprecated('Use vaultBackupManifestDescriptor instead')
const VaultBackupManifest$json = {
  '1': 'VaultBackupManifest',
  '2': [
    {'1': 'format_version', '3': 1, '4': 1, '5': 13, '10': 'formatVersion'},
    {
      '1': 'created_at_unix_ms',
      '3': 2,
      '4': 1,
      '5': 4,
      '10': 'createdAtUnixMs'
    },
    {'1': 'vault_file_name', '3': 3, '4': 1, '5': 9, '10': 'vaultFileName'},
    {'1': 'vault_size', '3': 4, '4': 1, '5': 4, '10': 'vaultSize'},
    {'1': 'vault_sha256', '3': 5, '4': 1, '5': 9, '10': 'vaultSha256'},
  ],
};

/// Descriptor for `VaultBackupManifest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vaultBackupManifestDescriptor = $convert.base64Decode(
    'ChNWYXVsdEJhY2t1cE1hbmlmZXN0EiUKDmZvcm1hdF92ZXJzaW9uGAEgASgNUg1mb3JtYXRWZX'
    'JzaW9uEisKEmNyZWF0ZWRfYXRfdW5peF9tcxgCIAEoBFIPY3JlYXRlZEF0VW5peE1zEiYKD3Zh'
    'dWx0X2ZpbGVfbmFtZRgDIAEoCVINdmF1bHRGaWxlTmFtZRIdCgp2YXVsdF9zaXplGAQgASgEUg'
    'l2YXVsdFNpemUSIQoMdmF1bHRfc2hhMjU2GAUgASgJUgt2YXVsdFNoYTI1Ng==');

@$core.Deprecated('Use errorDetailsDescriptor instead')
const ErrorDetails$json = {
  '1': 'ErrorDetails',
  '2': [
    {'1': 'category', '3': 1, '4': 1, '5': 9, '10': 'category'},
    {'1': 'artifact_kind', '3': 2, '4': 1, '5': 9, '10': 'artifactKind'},
    {'1': 'found_version', '3': 3, '4': 1, '5': 13, '10': 'foundVersion'},
    {
      '1': 'supported_version',
      '3': 4,
      '4': 1,
      '5': 13,
      '10': 'supportedVersion'
    },
    {'1': 'message', '3': 5, '4': 1, '5': 9, '10': 'message'},
    {'1': 'guidance', '3': 6, '4': 1, '5': 9, '10': 'guidance'},
  ],
};

/// Descriptor for `ErrorDetails`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List errorDetailsDescriptor = $convert.base64Decode(
    'CgxFcnJvckRldGFpbHMSGgoIY2F0ZWdvcnkYASABKAlSCGNhdGVnb3J5EiMKDWFydGlmYWN0X2'
    'tpbmQYAiABKAlSDGFydGlmYWN0S2luZBIjCg1mb3VuZF92ZXJzaW9uGAMgASgNUgxmb3VuZFZl'
    'cnNpb24SKwoRc3VwcG9ydGVkX3ZlcnNpb24YBCABKA1SEHN1cHBvcnRlZFZlcnNpb24SGAoHbW'
    'Vzc2FnZRgFIAEoCVIHbWVzc2FnZRIaCghndWlkYW5jZRgGIAEoCVIIZ3VpZGFuY2U=');
