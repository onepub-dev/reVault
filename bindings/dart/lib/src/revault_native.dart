// Generated low-level FFI declarations. Not part of the package public API.
// ignore_for_file: non_constant_identifier_names, public_member_api_docs
// ignore_for_file: unused_element

import 'dart:ffi' as ffi;

final class RevaultBuffer extends ffi.Struct {
  external ffi.Pointer<ffi.Uint8> ptr;
  @ffi.UintPtr()
  external int len;
}

typedef _ApiAbiVersionNative = ffi.Uint32 Function();
typedef _ApiAbiVersionDart = int Function();
typedef _BufferLastErrorNative = ffi.Pointer<ffi.Uint8> Function();
typedef _BufferLastErrorDart = ffi.Pointer<ffi.Uint8> Function();
typedef _BufferLastErrorDetailsNative = RevaultBuffer Function();
typedef _BufferLastErrorDetailsDart = RevaultBuffer Function();
typedef _BufferFreeNative = ffi.Void Function(RevaultBuffer);
typedef _BufferFreeDart = void Function(RevaultBuffer);
typedef _SecretLenNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Size>);
typedef _SecretLenDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Size>);
typedef _SecretCopyNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _SecretCopyDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _SecretFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _SecretFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _LockboxFormatVersionNative = ffi.Uint16 Function();
typedef _LockboxFormatVersionDart = int Function();
typedef _LockboxProbeFormatVersionNative =
    ffi.Uint16 Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxProbeFormatVersionDart =
    int Function(ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxCreateNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxCreateDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxCreateWithOptionsNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Size,
    );
typedef _LockboxCreateWithOptionsDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
    );
typedef _LockboxCreatePasswordNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxCreatePasswordDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxCreateContactNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Void>);
typedef _LockboxCreateContactDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Void>);
typedef _LockboxCreateWithSigningKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _LockboxCreateWithSigningKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _LockboxOpenNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxOpenDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxOpenWithOptionsNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Size,
    );
typedef _LockboxOpenWithOptionsDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
    );
typedef _LockboxOpenPasswordNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxOpenPasswordDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxOpenContactNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _LockboxOpenContactDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _LockboxAddFileNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
    );
typedef _LockboxAddFileDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      bool,
    );
typedef _LockboxAddFileWithPermissionsNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint32,
      ffi.Bool,
    );
typedef _LockboxAddFileWithPermissionsDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
      bool,
    );
typedef _LockboxGetFileNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxGetFileDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxExtractFileNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
    );
typedef _LockboxExtractFileDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      bool,
    );
typedef _LockboxExtractDirectoryNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
      ffi.Uint64,
      ffi.Size,
      ffi.Bool,
      ffi.Bool,
      ffi.Bool,
    );
typedef _LockboxExtractDirectoryDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
      int,
      int,
      bool,
      bool,
      bool,
    );
typedef _LockboxStreamContentNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Bool);
typedef _LockboxStreamContentDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, bool);
typedef _LockboxCacheStatsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxCacheStatsDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxImportStatsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxImportStatsDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxResetImportStatsNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>);
typedef _LockboxResetImportStatsDart = bool Function(ffi.Pointer<ffi.Void>);
typedef _LockboxInspectFileNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxInspectFileDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxPageInspectionNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxPageInspectionDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxRecoveryReportNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxRecoveryReportDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxRecoveryReportRenderNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Bool, ffi.Size);
typedef _LockboxRecoveryReportRenderDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, bool, int);
typedef _LockboxRecoveryScanPathNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxRecoveryScanPathDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxStorageLenNative = ffi.Uint64 Function(ffi.Pointer<ffi.Void>);
typedef _LockboxStorageLenDart = int Function(ffi.Pointer<ffi.Void>);
typedef _LockboxSetWorkloadProfileNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxSetWorkloadProfileDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxSetWorkerPolicyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Size,
    );
typedef _LockboxSetWorkerPolicyDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int, int);
typedef _LockboxRuntimeOptionsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxRuntimeOptionsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxCommitNative = ffi.Bool Function(ffi.Pointer<ffi.Void>);
typedef _LockboxCommitDart = bool Function(ffi.Pointer<ffi.Void>);
typedef _LockboxCreateDirNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
    );
typedef _LockboxCreateDirDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int, bool);
typedef _LockboxDeleteNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxDeleteDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxRemoveDirNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
    );
typedef _LockboxRemoveDirDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int, bool);
typedef _LockboxCreateParentDirsNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxCreateParentDirsDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxRenameNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxRenameDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxListNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
    );
typedef _LockboxListDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      bool,
    );
typedef _LockboxListWithOptionsNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
      ffi.Bool,
      ffi.Bool,
      ffi.Bool,
      ffi.Size,
    );
typedef _LockboxListWithOptionsDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      bool,
      bool,
      bool,
      bool,
      int,
    );
typedef _LockboxStatNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxStatDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxSetVariableNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxSetVariableDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxSetSecretVariableNative = _LockboxSetVariableNative;
typedef _LockboxSetSecretVariableDart = _LockboxSetVariableDart;
typedef _LockboxGetVariableNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxGetVariableDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxGetSecretVariableNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Pointer<ffi.Void>>,
    );
typedef _LockboxGetSecretVariableDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Pointer<ffi.Void>>,
    );
typedef _LockboxDeleteVariableNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxDeleteVariableDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxMoveVariablesNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxMoveVariablesDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxListVariablesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxListVariablesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxVariableSensitivityNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxVariableSensitivityDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxAddSymlinkNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Bool,
    );
typedef _LockboxAddSymlinkDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      bool,
    );
typedef _LockboxGetSymlinkTargetNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxGetSymlinkTargetDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxIdNative = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxIdDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxExistsNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxExistsDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxIsDirNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxIsDirDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxPermissionsNative =
    ffi.Uint32 Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxPermissionsDart =
    int Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxSetPermissionsNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint32,
    );
typedef _LockboxSetPermissionsDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int, int);
typedef _LockboxReadRangeNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
      ffi.Uint64,
    );
typedef _LockboxReadRangeDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
      int,
    );
typedef _LockboxRecoveryScanNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxRecoveryScanDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxRecoverySalvageNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _LockboxRecoverySalvageDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _LockboxAddPasswordNative =
    ffi.Uint64 Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxAddPasswordDart =
    int Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxAddContactNative =
    ffi.Uint64 Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxAddContactDart =
    int Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxDeleteKeyNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Uint64);
typedef _LockboxDeleteKeyDart = bool Function(ffi.Pointer<ffi.Void>, int);
typedef _LockboxListKeySlotsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxListKeySlotsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxSetOwnerSigningKeyNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>);
typedef _LockboxSetOwnerSigningKeyDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>);
typedef _LockboxOwnerInspectionNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxOwnerInspectionDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxDefineFormNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxDefineFormDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxListFormDefinitionsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxListFormDefinitionsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxResolveFormNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxResolveFormDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxListFormRevisionsNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxListFormRevisionsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxCreateFormRecordNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxCreateFormRecordDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxSetFormFieldNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxSetFormFieldDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxSetSecretFormFieldNative = _LockboxSetFormFieldNative;
typedef _LockboxSetSecretFormFieldDart = _LockboxSetFormFieldDart;
typedef _LockboxListFormRecordsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxListFormRecordsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxGetFormRecordNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxGetFormRecordDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxDeleteFormRecordNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxDeleteFormRecordDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxMoveFormRecordsNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _LockboxMoveFormRecordsDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _LockboxGetFormFieldNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _LockboxGetFormFieldDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _LockboxGetSecretFormFieldNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Pointer<ffi.Void>>,
    );
typedef _LockboxGetSecretFormFieldDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Pointer<ffi.Void>>,
    );
typedef _LockboxToBytesNative = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxToBytesDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _LockboxFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _LockboxFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _VaultIsRunningNative = ffi.Bool Function();
typedef _VaultIsRunningDart = bool Function();
typedef _VaultForgetAllNative = ffi.Bool Function();
typedef _VaultForgetAllDart = bool Function();
typedef _KeyContactGenerateNative = ffi.Pointer<ffi.Void> Function();
typedef _KeyContactGenerateDart = ffi.Pointer<ffi.Void> Function();
typedef _KeyContactFromPrivateNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _KeyContactFromPrivateDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _KeyContactPublicNative = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactPublicDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactPrivateNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactPrivateDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactPublicFromBytesNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _KeyContactPublicFromBytesDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _KeyContactPublicFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactPublicFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactEncryptNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _KeyContactEncryptDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _KeyContactDecryptNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>);
typedef _KeyContactDecryptDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedPublicNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedPublicDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedCiphertextNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedCiphertextDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedEncryptedNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedEncryptedDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _KeyContactWrappedFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningGenerateNative = ffi.Pointer<ffi.Void> Function();
typedef _KeySigningGenerateDart = ffi.Pointer<ffi.Void> Function();
typedef _KeySigningFromPrivateNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _KeySigningFromPrivateDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _KeySigningPublicNative = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningPublicDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningPrivateNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningPrivateDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningPublicFromBytesNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _KeySigningPublicFromBytesDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _KeySigningPublicFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningPublicFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _KeySigningFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _VaultKeyExportPrivateNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultKeyExportPrivateDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyExportPublicNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultKeyExportPublicDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyImportPrivateNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyImportPrivateDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyImportPublicNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyImportPublicDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyFingerprintNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultKeyFingerprintDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultKeyFormatHexNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyFormatHexDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyDecodeHexNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyDecodeHexDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyFormatCrockfordNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyFormatCrockfordDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyFormatCrockfordReadingNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyFormatCrockfordReadingDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyDecodeCrockfordNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyDecodeCrockfordDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyHexEncodeNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyHexEncodeDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultKeyHexDecodeNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultKeyHexDecodeDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryOpenNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryOpenDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultStructureVersionCurrentNative = ffi.Uint32 Function();
typedef _VaultStructureVersionCurrentDart = int Function();
typedef _VaultDirectoryProbeStructureVersionNative =
    ffi.Uint32 Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryProbeStructureVersionDart =
    int Function(ffi.Pointer<ffi.Uint8>, int, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryOpenOrCreateDefaultNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryOpenOrCreateDefaultDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryReplaceDefaultNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryReplaceDefaultDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryChangePasswordNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryChangePasswordDart =
    bool Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryChangeDefaultPasswordNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryChangeDefaultPasswordDart =
    bool Function(ffi.Pointer<ffi.Uint8>, int, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryReplaceNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryReplaceDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryOpenOrCreateNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryOpenOrCreateDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryRootNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryRootDart = RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryStructureVersionNative =
    ffi.Uint32 Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryStructureVersionDart =
    int Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListPrivateKeysNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListPrivateKeysDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListPrivateKeyNamesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListPrivateKeyNamesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListContactNamesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListContactNamesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListFormAliasesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListFormAliasesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryPrivateKeyExistsNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryPrivateKeyExistsDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryDeletePrivateKeyNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryDeletePrivateKeyDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryStorePrivateKeyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultDirectoryStorePrivateKeyDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultDirectoryLoadPrivateKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryLoadPrivateKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryLoadPrivateKeyGenerationNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint16,
    );
typedef _VaultDirectoryLoadPrivateKeyGenerationDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
    );
typedef _VaultDirectoryStoreContactNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultDirectoryStoreContactDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultDirectoryLoadContactNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryLoadContactDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryContactExistsNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryContactExistsDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryDeleteContactNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryDeleteContactDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryListContactsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListContactsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryStoreProfileEmailNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryStoreProfileEmailDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryProfileEmailNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryProfileEmailDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryStoreBackupNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryStoreBackupDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryLoadBackupNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryLoadBackupDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryBackupCountNative =
    ffi.Uint64 Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryBackupCountDart = int Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryRestorePrivateKeyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Void>,
      ffi.Bool,
    );
typedef _VaultDirectoryRestorePrivateKeyDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Void>,
      bool,
    );
typedef _VaultDirectoryLoadOwnerSigningKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryLoadOwnerSigningKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryLoadOwnerSigningKeyGenerationNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint16,
    );
typedef _VaultDirectoryLoadOwnerSigningKeyGenerationDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
    );
typedef _VaultDirectoryStoreContactSigningKeyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultDirectoryStoreContactSigningKeyDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultDirectoryLoadContactSigningKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryLoadContactSigningKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryListProfileGenerationsNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryListProfileGenerationsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryRotatePrivateKeyNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryRotatePrivateKeyDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryRememberLockboxNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryRememberLockboxDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryListKnownLockboxesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListKnownLockboxesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryForgetLockboxNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultDirectoryForgetLockboxDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryRememberAccessSlotLabelNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryRememberAccessSlotLabelDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryListAccessSlotLabelsNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryListAccessSlotLabelsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryFindAccessSlotLabelsNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryFindAccessSlotLabelsDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryForgetAccessSlotLabelNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
    );
typedef _VaultDirectoryForgetAccessSlotLabelDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int, int);
typedef _VaultDirectoryDefineFormNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryDefineFormDart =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryResolveFormNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryResolveFormDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectoryListFormsNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListFormsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryListFormRevisionsNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryListFormRevisionsDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultDirectorySeedFormsNative =
    ffi.Size Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectorySeedFormsDart = int Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryRememberPasswordNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryRememberPasswordDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultDirectoryRememberedPasswordNative =
    RevaultBuffer Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultDirectoryRememberedPasswordDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultBackupDefaultNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Bool);
typedef _VaultBackupDefaultDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int, bool);
typedef _VaultRestoreDefaultNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size, ffi.Bool);
typedef _VaultRestoreDefaultDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int, bool);
typedef _VaultDirectoryFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _VaultDirectoryFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyOpenNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultReadOnlyOpenDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultReadOnlyOpenDefaultNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultReadOnlyOpenDefaultDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultReadOnlyListProfileNamesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListProfileNamesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListContactNamesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListContactNamesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListFormAliasesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListFormAliasesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListKnownLockboxesNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyListKnownLockboxesDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _VaultReadOnlyFreeDart = void Function(ffi.Pointer<ffi.Void>);
typedef _VaultAgentServeNative = ffi.Bool Function();
typedef _VaultAgentServeDart = bool Function();
typedef _VaultAgentVerifyTransportNative = ffi.Bool Function();
typedef _VaultAgentVerifyTransportDart = bool Function();
typedef _VaultAgentGetNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultAgentGetDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentPutNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultAgentPutDart =
    bool Function(ffi.Pointer<ffi.Uint8>, int, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentForgetNative =
    ffi.Bool Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultAgentForgetDart = bool Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentStopNative = ffi.Bool Function();
typedef _VaultAgentStopDart = bool Function();
typedef _VaultAgentStartNative = ffi.Bool Function();
typedef _VaultAgentStartDart = bool Function();
typedef _VaultAgentListNative = RevaultBuffer Function();
typedef _VaultAgentListDart = RevaultBuffer Function();
typedef _VaultAgentSleepSupportNative = RevaultBuffer Function();
typedef _VaultAgentSleepSupportDart = RevaultBuffer Function();
typedef _VaultPlatformStatusNative = RevaultBuffer Function();
typedef _VaultPlatformStatusDart = RevaultBuffer Function();
typedef _VaultPlatformSetScopeNative =
    ffi.Bool Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultPlatformSetScopeDart = bool Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultPlatformForgetPasswordNative = ffi.Bool Function();
typedef _VaultPlatformForgetPasswordDart = bool Function();
typedef _VaultPlatformPutPasswordNative =
    ffi.Bool Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultPlatformPutPasswordDart =
    bool Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultPlatformEnableNative = ffi.Bool Function();
typedef _VaultPlatformEnableDart = bool Function();
typedef _VaultPlatformDisableNative = ffi.Bool Function();
typedef _VaultPlatformDisableDart = bool Function();
typedef _VaultPlatformDisabledNative = ffi.Bool Function();
typedef _VaultPlatformDisabledDart = bool Function();
typedef _VaultPlatformGetPasswordNative = RevaultBuffer Function();
typedef _VaultPlatformGetPasswordDart = RevaultBuffer Function();
typedef _VaultDefaultDirectoryNative = RevaultBuffer Function();
typedef _VaultDefaultDirectoryDart = RevaultBuffer Function();
typedef _VaultDefaultPathNative = RevaultBuffer Function();
typedef _VaultDefaultPathDart = RevaultBuffer Function();
typedef _VaultAgentLogPathNative = RevaultBuffer Function();
typedef _VaultAgentLogPathDart = RevaultBuffer Function();
typedef _VaultAgentLogDestinationNative = RevaultBuffer Function();
typedef _VaultAgentLogDestinationDart = RevaultBuffer Function();
typedef _VaultAgentGetVaultUnlockKeyNative =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultAgentGetVaultUnlockKeyDart =
    RevaultBuffer Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentPutVaultUnlockKeyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
    );
typedef _VaultAgentPutVaultUnlockKeyDart =
    bool Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
    );
typedef _VaultAgentForgetVaultUnlockKeyNative =
    ffi.Bool Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultAgentForgetVaultUnlockKeyDart =
    bool Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentGetOwnerSigningKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultAgentGetOwnerSigningKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultAgentPutOwnerSigningKeyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
      ffi.Uint64,
    );
typedef _VaultAgentPutOwnerSigningKeyDart =
    bool Function(
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
      int,
    );
typedef _VaultAgentForgetOwnerSigningKeyNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultAgentForgetOwnerSigningKeyDart =
    bool Function(ffi.Pointer<ffi.Uint8>, int, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentBeginActivityNative =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultAgentBeginActivityDart =
    ffi.Pointer<ffi.Void> Function(ffi.Pointer<ffi.Uint8>, int);
typedef _VaultAgentEndActivityNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _VaultAgentEndActivityDart = void Function(ffi.Pointer<ffi.Void>);
typedef _VaultLocalNative = ffi.Pointer<ffi.Void> Function();
typedef _VaultLocalDart = ffi.Pointer<ffi.Void> Function();
typedef _VaultCreateLockboxPasswordNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultCreateLockboxPasswordDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultOpenLockboxPasswordNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
    );
typedef _VaultOpenLockboxPasswordDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
    );
typedef _VaultCreateLockboxContentKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultCreateLockboxContentKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultCreateLockboxContactNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultCreateLockboxContactDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultOpenLockboxContentKeyNative =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultOpenLockboxContentKeyDart =
    ffi.Pointer<ffi.Void> Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Void>,
    );
typedef _VaultCacheLockboxPasswordNative =
    ffi.Bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Pointer<ffi.Uint8>,
      ffi.Size,
      ffi.Uint64,
    );
typedef _VaultCacheLockboxPasswordDart =
    bool Function(
      ffi.Pointer<ffi.Void>,
      ffi.Pointer<ffi.Uint8>,
      int,
      ffi.Pointer<ffi.Uint8>,
      int,
      int,
    );
typedef _VaultCloseLockboxNative =
    ffi.Bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, ffi.Size);
typedef _VaultCloseLockboxDart =
    bool Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Uint8>, int);
typedef _VaultCloseAllNative = ffi.Bool Function(ffi.Pointer<ffi.Void>);
typedef _VaultCloseAllDart = bool Function(ffi.Pointer<ffi.Void>);
typedef _VaultFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _VaultFreeDart = void Function(ffi.Pointer<ffi.Void>);

final class RevaultNative {
  RevaultNative(ffi.DynamicLibrary library)
    : _api_abi_version = library
          .lookupFunction<_ApiAbiVersionNative, _ApiAbiVersionDart>(
            'api_abi_version',
          ),
      _buffer_last_error = library
          .lookupFunction<_BufferLastErrorNative, _BufferLastErrorDart>(
            'buffer_last_error',
          ),
      _buffer_last_error_details = library
          .lookupFunction<
            _BufferLastErrorDetailsNative,
            _BufferLastErrorDetailsDart
          >('buffer_last_error_details'),
      _buffer_free = library.lookupFunction<_BufferFreeNative, _BufferFreeDart>(
        'buffer_free',
      ),
      _secret_len = library.lookupFunction<_SecretLenNative, _SecretLenDart>(
        'secret_len',
      ),
      _secret_copy = library.lookupFunction<_SecretCopyNative, _SecretCopyDart>(
        'secret_copy',
      ),
      _secret_free = library.lookupFunction<_SecretFreeNative, _SecretFreeDart>(
        'secret_free',
      ),
      _lockbox_format_version = library
          .lookupFunction<
            _LockboxFormatVersionNative,
            _LockboxFormatVersionDart
          >('lockbox_format_version'),
      _lockbox_probe_format_version = library
          .lookupFunction<
            _LockboxProbeFormatVersionNative,
            _LockboxProbeFormatVersionDart
          >('lockbox_probe_format_version'),
      _lockbox_create = library
          .lookupFunction<_LockboxCreateNative, _LockboxCreateDart>(
            'lockbox_create',
          ),
      _lockbox_create_with_options = library
          .lookupFunction<
            _LockboxCreateWithOptionsNative,
            _LockboxCreateWithOptionsDart
          >('lockbox_create_with_options'),
      _lockbox_create_password = library
          .lookupFunction<
            _LockboxCreatePasswordNative,
            _LockboxCreatePasswordDart
          >('lockbox_create_password'),
      _lockbox_create_contact = library
          .lookupFunction<
            _LockboxCreateContactNative,
            _LockboxCreateContactDart
          >('lockbox_create_contact'),
      _lockbox_create_with_signing_key = library
          .lookupFunction<
            _LockboxCreateWithSigningKeyNative,
            _LockboxCreateWithSigningKeyDart
          >('lockbox_create_with_signing_key'),
      _lockbox_open = library
          .lookupFunction<_LockboxOpenNative, _LockboxOpenDart>('lockbox_open'),
      _lockbox_open_with_options = library
          .lookupFunction<
            _LockboxOpenWithOptionsNative,
            _LockboxOpenWithOptionsDart
          >('lockbox_open_with_options'),
      _lockbox_open_password = library
          .lookupFunction<_LockboxOpenPasswordNative, _LockboxOpenPasswordDart>(
            'lockbox_open_password',
          ),
      _lockbox_open_contact = library
          .lookupFunction<_LockboxOpenContactNative, _LockboxOpenContactDart>(
            'lockbox_open_contact',
          ),
      _lockbox_add_file = library
          .lookupFunction<_LockboxAddFileNative, _LockboxAddFileDart>(
            'lockbox_add_file',
          ),
      _lockbox_add_file_with_permissions = library
          .lookupFunction<
            _LockboxAddFileWithPermissionsNative,
            _LockboxAddFileWithPermissionsDart
          >('lockbox_add_file_with_permissions'),
      _lockbox_get_file = library
          .lookupFunction<_LockboxGetFileNative, _LockboxGetFileDart>(
            'lockbox_get_file',
          ),
      _lockbox_extract_file = library
          .lookupFunction<_LockboxExtractFileNative, _LockboxExtractFileDart>(
            'lockbox_extract_file',
          ),
      _lockbox_extract_directory = library
          .lookupFunction<
            _LockboxExtractDirectoryNative,
            _LockboxExtractDirectoryDart
          >('lockbox_extract_directory'),
      _lockbox_stream_content = library
          .lookupFunction<
            _LockboxStreamContentNative,
            _LockboxStreamContentDart
          >('lockbox_stream_content'),
      _lockbox_cache_stats = library
          .lookupFunction<_LockboxCacheStatsNative, _LockboxCacheStatsDart>(
            'lockbox_cache_stats',
          ),
      _lockbox_import_stats = library
          .lookupFunction<_LockboxImportStatsNative, _LockboxImportStatsDart>(
            'lockbox_import_stats',
          ),
      _lockbox_reset_import_stats = library
          .lookupFunction<
            _LockboxResetImportStatsNative,
            _LockboxResetImportStatsDart
          >('lockbox_reset_import_stats'),
      _lockbox_inspect_file = library
          .lookupFunction<_LockboxInspectFileNative, _LockboxInspectFileDart>(
            'lockbox_inspect_file',
          ),
      _lockbox_page_inspection = library
          .lookupFunction<
            _LockboxPageInspectionNative,
            _LockboxPageInspectionDart
          >('lockbox_page_inspection'),
      _lockbox_recovery_report = library
          .lookupFunction<
            _LockboxRecoveryReportNative,
            _LockboxRecoveryReportDart
          >('lockbox_recovery_report'),
      _lockbox_recovery_report_render = library
          .lookupFunction<
            _LockboxRecoveryReportRenderNative,
            _LockboxRecoveryReportRenderDart
          >('lockbox_recovery_report_render'),
      _lockbox_recovery_scan_path = library
          .lookupFunction<
            _LockboxRecoveryScanPathNative,
            _LockboxRecoveryScanPathDart
          >('lockbox_recovery_scan_path'),
      _lockbox_storage_len = library
          .lookupFunction<_LockboxStorageLenNative, _LockboxStorageLenDart>(
            'lockbox_storage_len',
          ),
      _lockbox_set_workload_profile = library
          .lookupFunction<
            _LockboxSetWorkloadProfileNative,
            _LockboxSetWorkloadProfileDart
          >('lockbox_set_workload_profile'),
      _lockbox_set_worker_policy = library
          .lookupFunction<
            _LockboxSetWorkerPolicyNative,
            _LockboxSetWorkerPolicyDart
          >('lockbox_set_worker_policy'),
      _lockbox_runtime_options = library
          .lookupFunction<
            _LockboxRuntimeOptionsNative,
            _LockboxRuntimeOptionsDart
          >('lockbox_runtime_options'),
      _lockbox_commit = library
          .lookupFunction<_LockboxCommitNative, _LockboxCommitDart>(
            'lockbox_commit',
          ),
      _lockbox_create_dir = library
          .lookupFunction<_LockboxCreateDirNative, _LockboxCreateDirDart>(
            'lockbox_create_dir',
          ),
      _lockbox_delete = library
          .lookupFunction<_LockboxDeleteNative, _LockboxDeleteDart>(
            'lockbox_delete',
          ),
      _lockbox_remove_dir = library
          .lookupFunction<_LockboxRemoveDirNative, _LockboxRemoveDirDart>(
            'lockbox_remove_dir',
          ),
      _lockbox_create_parent_dirs = library
          .lookupFunction<
            _LockboxCreateParentDirsNative,
            _LockboxCreateParentDirsDart
          >('lockbox_create_parent_dirs'),
      _lockbox_rename = library
          .lookupFunction<_LockboxRenameNative, _LockboxRenameDart>(
            'lockbox_rename',
          ),
      _lockbox_list = library
          .lookupFunction<_LockboxListNative, _LockboxListDart>('lockbox_list'),
      _lockbox_list_with_options = library
          .lookupFunction<
            _LockboxListWithOptionsNative,
            _LockboxListWithOptionsDart
          >('lockbox_list_with_options'),
      _lockbox_stat = library
          .lookupFunction<_LockboxStatNative, _LockboxStatDart>('lockbox_stat'),
      _lockbox_set_variable = library
          .lookupFunction<_LockboxSetVariableNative, _LockboxSetVariableDart>(
            'lockbox_set_variable',
          ),
      _lockbox_set_secret_variable = library
          .lookupFunction<
            _LockboxSetSecretVariableNative,
            _LockboxSetSecretVariableDart
          >('lockbox_set_secret_variable'),
      _lockbox_get_variable = library
          .lookupFunction<_LockboxGetVariableNative, _LockboxGetVariableDart>(
            'lockbox_get_variable',
          ),
      _lockbox_get_secret_variable = library
          .lookupFunction<
            _LockboxGetSecretVariableNative,
            _LockboxGetSecretVariableDart
          >('lockbox_get_secret_variable'),
      _lockbox_delete_variable = library
          .lookupFunction<
            _LockboxDeleteVariableNative,
            _LockboxDeleteVariableDart
          >('lockbox_delete_variable'),
      _lockbox_move_variables = library
          .lookupFunction<
            _LockboxMoveVariablesNative,
            _LockboxMoveVariablesDart
          >('lockbox_move_variables'),
      _lockbox_list_variables = library
          .lookupFunction<
            _LockboxListVariablesNative,
            _LockboxListVariablesDart
          >('lockbox_list_variables'),
      _lockbox_variable_sensitivity = library
          .lookupFunction<
            _LockboxVariableSensitivityNative,
            _LockboxVariableSensitivityDart
          >('lockbox_variable_sensitivity'),
      _lockbox_add_symlink = library
          .lookupFunction<_LockboxAddSymlinkNative, _LockboxAddSymlinkDart>(
            'lockbox_add_symlink',
          ),
      _lockbox_get_symlink_target = library
          .lookupFunction<
            _LockboxGetSymlinkTargetNative,
            _LockboxGetSymlinkTargetDart
          >('lockbox_get_symlink_target'),
      _lockbox_id = library.lookupFunction<_LockboxIdNative, _LockboxIdDart>(
        'lockbox_id',
      ),
      _lockbox_exists = library
          .lookupFunction<_LockboxExistsNative, _LockboxExistsDart>(
            'lockbox_exists',
          ),
      _lockbox_is_dir = library
          .lookupFunction<_LockboxIsDirNative, _LockboxIsDirDart>(
            'lockbox_is_dir',
          ),
      _lockbox_permissions = library
          .lookupFunction<_LockboxPermissionsNative, _LockboxPermissionsDart>(
            'lockbox_permissions',
          ),
      _lockbox_set_permissions = library
          .lookupFunction<
            _LockboxSetPermissionsNative,
            _LockboxSetPermissionsDart
          >('lockbox_set_permissions'),
      _lockbox_read_range = library
          .lookupFunction<_LockboxReadRangeNative, _LockboxReadRangeDart>(
            'lockbox_read_range',
          ),
      _lockbox_recovery_scan = library
          .lookupFunction<_LockboxRecoveryScanNative, _LockboxRecoveryScanDart>(
            'lockbox_recovery_scan',
          ),
      _lockbox_recovery_salvage = library
          .lookupFunction<
            _LockboxRecoverySalvageNative,
            _LockboxRecoverySalvageDart
          >('lockbox_recovery_salvage'),
      _lockbox_add_password = library
          .lookupFunction<_LockboxAddPasswordNative, _LockboxAddPasswordDart>(
            'lockbox_add_password',
          ),
      _lockbox_add_contact = library
          .lookupFunction<_LockboxAddContactNative, _LockboxAddContactDart>(
            'lockbox_add_contact',
          ),
      _lockbox_delete_key = library
          .lookupFunction<_LockboxDeleteKeyNative, _LockboxDeleteKeyDart>(
            'lockbox_delete_key',
          ),
      _lockbox_list_key_slots = library
          .lookupFunction<_LockboxListKeySlotsNative, _LockboxListKeySlotsDart>(
            'lockbox_list_key_slots',
          ),
      _lockbox_set_owner_signing_key = library
          .lookupFunction<
            _LockboxSetOwnerSigningKeyNative,
            _LockboxSetOwnerSigningKeyDart
          >('lockbox_set_owner_signing_key'),
      _lockbox_owner_inspection = library
          .lookupFunction<
            _LockboxOwnerInspectionNative,
            _LockboxOwnerInspectionDart
          >('lockbox_owner_inspection'),
      _lockbox_define_form = library
          .lookupFunction<_LockboxDefineFormNative, _LockboxDefineFormDart>(
            'lockbox_define_form',
          ),
      _lockbox_list_form_definitions = library
          .lookupFunction<
            _LockboxListFormDefinitionsNative,
            _LockboxListFormDefinitionsDart
          >('lockbox_list_form_definitions'),
      _lockbox_resolve_form = library
          .lookupFunction<_LockboxResolveFormNative, _LockboxResolveFormDart>(
            'lockbox_resolve_form',
          ),
      _lockbox_list_form_revisions = library
          .lookupFunction<
            _LockboxListFormRevisionsNative,
            _LockboxListFormRevisionsDart
          >('lockbox_list_form_revisions'),
      _lockbox_create_form_record = library
          .lookupFunction<
            _LockboxCreateFormRecordNative,
            _LockboxCreateFormRecordDart
          >('lockbox_create_form_record'),
      _lockbox_set_form_field = library
          .lookupFunction<_LockboxSetFormFieldNative, _LockboxSetFormFieldDart>(
            'lockbox_set_form_field',
          ),
      _lockbox_set_secret_form_field = library
          .lookupFunction<
            _LockboxSetSecretFormFieldNative,
            _LockboxSetSecretFormFieldDart
          >('lockbox_set_secret_form_field'),
      _lockbox_list_form_records = library
          .lookupFunction<
            _LockboxListFormRecordsNative,
            _LockboxListFormRecordsDart
          >('lockbox_list_form_records'),
      _lockbox_get_form_record = library
          .lookupFunction<
            _LockboxGetFormRecordNative,
            _LockboxGetFormRecordDart
          >('lockbox_get_form_record'),
      _lockbox_delete_form_record = library
          .lookupFunction<
            _LockboxDeleteFormRecordNative,
            _LockboxDeleteFormRecordDart
          >('lockbox_delete_form_record'),
      _lockbox_move_form_records = library
          .lookupFunction<
            _LockboxMoveFormRecordsNative,
            _LockboxMoveFormRecordsDart
          >('lockbox_move_form_records'),
      _lockbox_get_form_field = library
          .lookupFunction<_LockboxGetFormFieldNative, _LockboxGetFormFieldDart>(
            'lockbox_get_form_field',
          ),
      _lockbox_get_secret_form_field = library
          .lookupFunction<
            _LockboxGetSecretFormFieldNative,
            _LockboxGetSecretFormFieldDart
          >('lockbox_get_secret_form_field'),
      _lockbox_to_bytes = library
          .lookupFunction<_LockboxToBytesNative, _LockboxToBytesDart>(
            'lockbox_to_bytes',
          ),
      _lockbox_free = library
          .lookupFunction<_LockboxFreeNative, _LockboxFreeDart>('lockbox_free'),
      _vault_is_running = library
          .lookupFunction<_VaultIsRunningNative, _VaultIsRunningDart>(
            'vault_is_running',
          ),
      _vault_forget_all = library
          .lookupFunction<_VaultForgetAllNative, _VaultForgetAllDart>(
            'vault_forget_all',
          ),
      _key_contact_generate = library
          .lookupFunction<_KeyContactGenerateNative, _KeyContactGenerateDart>(
            'key_contact_generate',
          ),
      _key_contact_from_private = library
          .lookupFunction<
            _KeyContactFromPrivateNative,
            _KeyContactFromPrivateDart
          >('key_contact_from_private'),
      _key_contact_public = library
          .lookupFunction<_KeyContactPublicNative, _KeyContactPublicDart>(
            'key_contact_public',
          ),
      _key_contact_private = library
          .lookupFunction<_KeyContactPrivateNative, _KeyContactPrivateDart>(
            'key_contact_private',
          ),
      _key_contact_public_from_bytes = library
          .lookupFunction<
            _KeyContactPublicFromBytesNative,
            _KeyContactPublicFromBytesDart
          >('key_contact_public_from_bytes'),
      _key_contact_public_free = library
          .lookupFunction<
            _KeyContactPublicFreeNative,
            _KeyContactPublicFreeDart
          >('key_contact_public_free'),
      _key_contact_free = library
          .lookupFunction<_KeyContactFreeNative, _KeyContactFreeDart>(
            'key_contact_free',
          ),
      _key_contact_encrypt = library
          .lookupFunction<_KeyContactEncryptNative, _KeyContactEncryptDart>(
            'key_contact_encrypt',
          ),
      _key_contact_decrypt = library
          .lookupFunction<_KeyContactDecryptNative, _KeyContactDecryptDart>(
            'key_contact_decrypt',
          ),
      _key_contact_wrapped_public = library
          .lookupFunction<
            _KeyContactWrappedPublicNative,
            _KeyContactWrappedPublicDart
          >('key_contact_wrapped_public'),
      _key_contact_wrapped_ciphertext = library
          .lookupFunction<
            _KeyContactWrappedCiphertextNative,
            _KeyContactWrappedCiphertextDart
          >('key_contact_wrapped_ciphertext'),
      _key_contact_wrapped_encrypted = library
          .lookupFunction<
            _KeyContactWrappedEncryptedNative,
            _KeyContactWrappedEncryptedDart
          >('key_contact_wrapped_encrypted'),
      _key_contact_wrapped_free = library
          .lookupFunction<
            _KeyContactWrappedFreeNative,
            _KeyContactWrappedFreeDart
          >('key_contact_wrapped_free'),
      _key_signing_generate = library
          .lookupFunction<_KeySigningGenerateNative, _KeySigningGenerateDart>(
            'key_signing_generate',
          ),
      _key_signing_from_private = library
          .lookupFunction<
            _KeySigningFromPrivateNative,
            _KeySigningFromPrivateDart
          >('key_signing_from_private'),
      _key_signing_public = library
          .lookupFunction<_KeySigningPublicNative, _KeySigningPublicDart>(
            'key_signing_public',
          ),
      _key_signing_private = library
          .lookupFunction<_KeySigningPrivateNative, _KeySigningPrivateDart>(
            'key_signing_private',
          ),
      _key_signing_public_from_bytes = library
          .lookupFunction<
            _KeySigningPublicFromBytesNative,
            _KeySigningPublicFromBytesDart
          >('key_signing_public_from_bytes'),
      _key_signing_public_free = library
          .lookupFunction<
            _KeySigningPublicFreeNative,
            _KeySigningPublicFreeDart
          >('key_signing_public_free'),
      _key_signing_free = library
          .lookupFunction<_KeySigningFreeNative, _KeySigningFreeDart>(
            'key_signing_free',
          ),
      _vault_key_export_private = library
          .lookupFunction<
            _VaultKeyExportPrivateNative,
            _VaultKeyExportPrivateDart
          >('vault_key_export_private'),
      _vault_key_export_public = library
          .lookupFunction<
            _VaultKeyExportPublicNative,
            _VaultKeyExportPublicDart
          >('vault_key_export_public'),
      _vault_key_import_private = library
          .lookupFunction<
            _VaultKeyImportPrivateNative,
            _VaultKeyImportPrivateDart
          >('vault_key_import_private'),
      _vault_key_import_public = library
          .lookupFunction<
            _VaultKeyImportPublicNative,
            _VaultKeyImportPublicDart
          >('vault_key_import_public'),
      _vault_key_fingerprint = library
          .lookupFunction<_VaultKeyFingerprintNative, _VaultKeyFingerprintDart>(
            'vault_key_fingerprint',
          ),
      _vault_key_format_hex = library
          .lookupFunction<_VaultKeyFormatHexNative, _VaultKeyFormatHexDart>(
            'vault_key_format_hex',
          ),
      _vault_key_decode_hex = library
          .lookupFunction<_VaultKeyDecodeHexNative, _VaultKeyDecodeHexDart>(
            'vault_key_decode_hex',
          ),
      _vault_key_format_crockford = library
          .lookupFunction<
            _VaultKeyFormatCrockfordNative,
            _VaultKeyFormatCrockfordDart
          >('vault_key_format_crockford'),
      _vault_key_format_crockford_reading = library
          .lookupFunction<
            _VaultKeyFormatCrockfordReadingNative,
            _VaultKeyFormatCrockfordReadingDart
          >('vault_key_format_crockford_reading'),
      _vault_key_decode_crockford = library
          .lookupFunction<
            _VaultKeyDecodeCrockfordNative,
            _VaultKeyDecodeCrockfordDart
          >('vault_key_decode_crockford'),
      _vault_key_hex_encode = library
          .lookupFunction<_VaultKeyHexEncodeNative, _VaultKeyHexEncodeDart>(
            'vault_key_hex_encode',
          ),
      _vault_key_hex_decode = library
          .lookupFunction<_VaultKeyHexDecodeNative, _VaultKeyHexDecodeDart>(
            'vault_key_hex_decode',
          ),
      _vault_directory_open = library
          .lookupFunction<_VaultDirectoryOpenNative, _VaultDirectoryOpenDart>(
            'vault_directory_open',
          ),
      _vault_structure_version_current = library
          .lookupFunction<
            _VaultStructureVersionCurrentNative,
            _VaultStructureVersionCurrentDart
          >('vault_structure_version_current'),
      _vault_directory_probe_structure_version = library
          .lookupFunction<
            _VaultDirectoryProbeStructureVersionNative,
            _VaultDirectoryProbeStructureVersionDart
          >('vault_directory_probe_structure_version'),
      _vault_directory_open_or_create_default = library
          .lookupFunction<
            _VaultDirectoryOpenOrCreateDefaultNative,
            _VaultDirectoryOpenOrCreateDefaultDart
          >('vault_directory_open_or_create_default'),
      _vault_directory_replace_default = library
          .lookupFunction<
            _VaultDirectoryReplaceDefaultNative,
            _VaultDirectoryReplaceDefaultDart
          >('vault_directory_replace_default'),
      _vault_directory_change_password = library
          .lookupFunction<
            _VaultDirectoryChangePasswordNative,
            _VaultDirectoryChangePasswordDart
          >('vault_directory_change_password'),
      _vault_directory_change_default_password = library
          .lookupFunction<
            _VaultDirectoryChangeDefaultPasswordNative,
            _VaultDirectoryChangeDefaultPasswordDart
          >('vault_directory_change_default_password'),
      _vault_directory_replace = library
          .lookupFunction<
            _VaultDirectoryReplaceNative,
            _VaultDirectoryReplaceDart
          >('vault_directory_replace'),
      _vault_directory_open_or_create = library
          .lookupFunction<
            _VaultDirectoryOpenOrCreateNative,
            _VaultDirectoryOpenOrCreateDart
          >('vault_directory_open_or_create'),
      _vault_directory_root = library
          .lookupFunction<_VaultDirectoryRootNative, _VaultDirectoryRootDart>(
            'vault_directory_root',
          ),
      _vault_directory_structure_version = library
          .lookupFunction<
            _VaultDirectoryStructureVersionNative,
            _VaultDirectoryStructureVersionDart
          >('vault_directory_structure_version'),
      _vault_directory_list_private_keys = library
          .lookupFunction<
            _VaultDirectoryListPrivateKeysNative,
            _VaultDirectoryListPrivateKeysDart
          >('vault_directory_list_private_keys'),
      _vault_directory_list_private_key_names = library
          .lookupFunction<
            _VaultDirectoryListPrivateKeyNamesNative,
            _VaultDirectoryListPrivateKeyNamesDart
          >('vault_directory_list_private_key_names'),
      _vault_directory_list_contact_names = library
          .lookupFunction<
            _VaultDirectoryListContactNamesNative,
            _VaultDirectoryListContactNamesDart
          >('vault_directory_list_contact_names'),
      _vault_directory_list_form_aliases = library
          .lookupFunction<
            _VaultDirectoryListFormAliasesNative,
            _VaultDirectoryListFormAliasesDart
          >('vault_directory_list_form_aliases'),
      _vault_directory_private_key_exists = library
          .lookupFunction<
            _VaultDirectoryPrivateKeyExistsNative,
            _VaultDirectoryPrivateKeyExistsDart
          >('vault_directory_private_key_exists'),
      _vault_directory_delete_private_key = library
          .lookupFunction<
            _VaultDirectoryDeletePrivateKeyNative,
            _VaultDirectoryDeletePrivateKeyDart
          >('vault_directory_delete_private_key'),
      _vault_directory_store_private_key = library
          .lookupFunction<
            _VaultDirectoryStorePrivateKeyNative,
            _VaultDirectoryStorePrivateKeyDart
          >('vault_directory_store_private_key'),
      _vault_directory_load_private_key = library
          .lookupFunction<
            _VaultDirectoryLoadPrivateKeyNative,
            _VaultDirectoryLoadPrivateKeyDart
          >('vault_directory_load_private_key'),
      _vault_directory_load_private_key_generation = library
          .lookupFunction<
            _VaultDirectoryLoadPrivateKeyGenerationNative,
            _VaultDirectoryLoadPrivateKeyGenerationDart
          >('vault_directory_load_private_key_generation'),
      _vault_directory_store_contact = library
          .lookupFunction<
            _VaultDirectoryStoreContactNative,
            _VaultDirectoryStoreContactDart
          >('vault_directory_store_contact'),
      _vault_directory_load_contact = library
          .lookupFunction<
            _VaultDirectoryLoadContactNative,
            _VaultDirectoryLoadContactDart
          >('vault_directory_load_contact'),
      _vault_directory_contact_exists = library
          .lookupFunction<
            _VaultDirectoryContactExistsNative,
            _VaultDirectoryContactExistsDart
          >('vault_directory_contact_exists'),
      _vault_directory_delete_contact = library
          .lookupFunction<
            _VaultDirectoryDeleteContactNative,
            _VaultDirectoryDeleteContactDart
          >('vault_directory_delete_contact'),
      _vault_directory_list_contacts = library
          .lookupFunction<
            _VaultDirectoryListContactsNative,
            _VaultDirectoryListContactsDart
          >('vault_directory_list_contacts'),
      _vault_directory_store_profile_email = library
          .lookupFunction<
            _VaultDirectoryStoreProfileEmailNative,
            _VaultDirectoryStoreProfileEmailDart
          >('vault_directory_store_profile_email'),
      _vault_directory_profile_email = library
          .lookupFunction<
            _VaultDirectoryProfileEmailNative,
            _VaultDirectoryProfileEmailDart
          >('vault_directory_profile_email'),
      _vault_directory_store_backup = library
          .lookupFunction<
            _VaultDirectoryStoreBackupNative,
            _VaultDirectoryStoreBackupDart
          >('vault_directory_store_backup'),
      _vault_directory_load_backup = library
          .lookupFunction<
            _VaultDirectoryLoadBackupNative,
            _VaultDirectoryLoadBackupDart
          >('vault_directory_load_backup'),
      _vault_directory_backup_count = library
          .lookupFunction<
            _VaultDirectoryBackupCountNative,
            _VaultDirectoryBackupCountDart
          >('vault_directory_backup_count'),
      _vault_directory_restore_private_key = library
          .lookupFunction<
            _VaultDirectoryRestorePrivateKeyNative,
            _VaultDirectoryRestorePrivateKeyDart
          >('vault_directory_restore_private_key'),
      _vault_directory_load_owner_signing_key = library
          .lookupFunction<
            _VaultDirectoryLoadOwnerSigningKeyNative,
            _VaultDirectoryLoadOwnerSigningKeyDart
          >('vault_directory_load_owner_signing_key'),
      _vault_directory_load_owner_signing_key_generation = library
          .lookupFunction<
            _VaultDirectoryLoadOwnerSigningKeyGenerationNative,
            _VaultDirectoryLoadOwnerSigningKeyGenerationDart
          >('vault_directory_load_owner_signing_key_generation'),
      _vault_directory_store_contact_signing_key = library
          .lookupFunction<
            _VaultDirectoryStoreContactSigningKeyNative,
            _VaultDirectoryStoreContactSigningKeyDart
          >('vault_directory_store_contact_signing_key'),
      _vault_directory_load_contact_signing_key = library
          .lookupFunction<
            _VaultDirectoryLoadContactSigningKeyNative,
            _VaultDirectoryLoadContactSigningKeyDart
          >('vault_directory_load_contact_signing_key'),
      _vault_directory_list_profile_generations = library
          .lookupFunction<
            _VaultDirectoryListProfileGenerationsNative,
            _VaultDirectoryListProfileGenerationsDart
          >('vault_directory_list_profile_generations'),
      _vault_directory_rotate_private_key = library
          .lookupFunction<
            _VaultDirectoryRotatePrivateKeyNative,
            _VaultDirectoryRotatePrivateKeyDart
          >('vault_directory_rotate_private_key'),
      _vault_directory_remember_lockbox = library
          .lookupFunction<
            _VaultDirectoryRememberLockboxNative,
            _VaultDirectoryRememberLockboxDart
          >('vault_directory_remember_lockbox'),
      _vault_directory_list_known_lockboxes = library
          .lookupFunction<
            _VaultDirectoryListKnownLockboxesNative,
            _VaultDirectoryListKnownLockboxesDart
          >('vault_directory_list_known_lockboxes'),
      _vault_directory_forget_lockbox = library
          .lookupFunction<
            _VaultDirectoryForgetLockboxNative,
            _VaultDirectoryForgetLockboxDart
          >('vault_directory_forget_lockbox'),
      _vault_directory_remember_access_slot_label = library
          .lookupFunction<
            _VaultDirectoryRememberAccessSlotLabelNative,
            _VaultDirectoryRememberAccessSlotLabelDart
          >('vault_directory_remember_access_slot_label'),
      _vault_directory_list_access_slot_labels = library
          .lookupFunction<
            _VaultDirectoryListAccessSlotLabelsNative,
            _VaultDirectoryListAccessSlotLabelsDart
          >('vault_directory_list_access_slot_labels'),
      _vault_directory_find_access_slot_labels = library
          .lookupFunction<
            _VaultDirectoryFindAccessSlotLabelsNative,
            _VaultDirectoryFindAccessSlotLabelsDart
          >('vault_directory_find_access_slot_labels'),
      _vault_directory_forget_access_slot_label = library
          .lookupFunction<
            _VaultDirectoryForgetAccessSlotLabelNative,
            _VaultDirectoryForgetAccessSlotLabelDart
          >('vault_directory_forget_access_slot_label'),
      _vault_directory_define_form = library
          .lookupFunction<
            _VaultDirectoryDefineFormNative,
            _VaultDirectoryDefineFormDart
          >('vault_directory_define_form'),
      _vault_directory_resolve_form = library
          .lookupFunction<
            _VaultDirectoryResolveFormNative,
            _VaultDirectoryResolveFormDart
          >('vault_directory_resolve_form'),
      _vault_directory_list_forms = library
          .lookupFunction<
            _VaultDirectoryListFormsNative,
            _VaultDirectoryListFormsDart
          >('vault_directory_list_forms'),
      _vault_directory_list_form_revisions = library
          .lookupFunction<
            _VaultDirectoryListFormRevisionsNative,
            _VaultDirectoryListFormRevisionsDart
          >('vault_directory_list_form_revisions'),
      _vault_directory_seed_forms = library
          .lookupFunction<
            _VaultDirectorySeedFormsNative,
            _VaultDirectorySeedFormsDart
          >('vault_directory_seed_forms'),
      _vault_directory_remember_password = library
          .lookupFunction<
            _VaultDirectoryRememberPasswordNative,
            _VaultDirectoryRememberPasswordDart
          >('vault_directory_remember_password'),
      _vault_directory_remembered_password = library
          .lookupFunction<
            _VaultDirectoryRememberedPasswordNative,
            _VaultDirectoryRememberedPasswordDart
          >('vault_directory_remembered_password'),
      _vault_backup_default = library
          .lookupFunction<_VaultBackupDefaultNative, _VaultBackupDefaultDart>(
            'vault_backup_default',
          ),
      _vault_restore_default = library
          .lookupFunction<_VaultRestoreDefaultNative, _VaultRestoreDefaultDart>(
            'vault_restore_default',
          ),
      _vault_directory_free = library
          .lookupFunction<_VaultDirectoryFreeNative, _VaultDirectoryFreeDart>(
            'vault_directory_free',
          ),
      _vault_read_only_open = library
          .lookupFunction<_VaultReadOnlyOpenNative, _VaultReadOnlyOpenDart>(
            'vault_read_only_open',
          ),
      _vault_read_only_open_default = library
          .lookupFunction<
            _VaultReadOnlyOpenDefaultNative,
            _VaultReadOnlyOpenDefaultDart
          >('vault_read_only_open_default'),
      _vault_read_only_list_profile_names = library
          .lookupFunction<
            _VaultReadOnlyListProfileNamesNative,
            _VaultReadOnlyListProfileNamesDart
          >('vault_read_only_list_profile_names'),
      _vault_read_only_list_contact_names = library
          .lookupFunction<
            _VaultReadOnlyListContactNamesNative,
            _VaultReadOnlyListContactNamesDart
          >('vault_read_only_list_contact_names'),
      _vault_read_only_list_form_aliases = library
          .lookupFunction<
            _VaultReadOnlyListFormAliasesNative,
            _VaultReadOnlyListFormAliasesDart
          >('vault_read_only_list_form_aliases'),
      _vault_read_only_list_known_lockboxes = library
          .lookupFunction<
            _VaultReadOnlyListKnownLockboxesNative,
            _VaultReadOnlyListKnownLockboxesDart
          >('vault_read_only_list_known_lockboxes'),
      _vault_read_only_free = library
          .lookupFunction<_VaultReadOnlyFreeNative, _VaultReadOnlyFreeDart>(
            'vault_read_only_free',
          ),
      _vault_agent_serve = library
          .lookupFunction<_VaultAgentServeNative, _VaultAgentServeDart>(
            'vault_agent_serve',
          ),
      _vault_agent_verify_transport = library
          .lookupFunction<
            _VaultAgentVerifyTransportNative,
            _VaultAgentVerifyTransportDart
          >('vault_agent_verify_transport'),
      _vault_agent_get = library
          .lookupFunction<_VaultAgentGetNative, _VaultAgentGetDart>(
            'vault_agent_get',
          ),
      _vault_agent_put = library
          .lookupFunction<_VaultAgentPutNative, _VaultAgentPutDart>(
            'vault_agent_put',
          ),
      _vault_agent_forget = library
          .lookupFunction<_VaultAgentForgetNative, _VaultAgentForgetDart>(
            'vault_agent_forget',
          ),
      _vault_agent_stop = library
          .lookupFunction<_VaultAgentStopNative, _VaultAgentStopDart>(
            'vault_agent_stop',
          ),
      _vault_agent_start = library
          .lookupFunction<_VaultAgentStartNative, _VaultAgentStartDart>(
            'vault_agent_start',
          ),
      _vault_agent_list = library
          .lookupFunction<_VaultAgentListNative, _VaultAgentListDart>(
            'vault_agent_list',
          ),
      _vault_agent_sleep_support = library
          .lookupFunction<
            _VaultAgentSleepSupportNative,
            _VaultAgentSleepSupportDart
          >('vault_agent_sleep_support'),
      _vault_platform_status = library
          .lookupFunction<_VaultPlatformStatusNative, _VaultPlatformStatusDart>(
            'vault_platform_status',
          ),
      _vault_platform_set_scope = library
          .lookupFunction<
            _VaultPlatformSetScopeNative,
            _VaultPlatformSetScopeDart
          >('vault_platform_set_scope'),
      _vault_platform_forget_password = library
          .lookupFunction<
            _VaultPlatformForgetPasswordNative,
            _VaultPlatformForgetPasswordDart
          >('vault_platform_forget_password'),
      _vault_platform_put_password = library
          .lookupFunction<
            _VaultPlatformPutPasswordNative,
            _VaultPlatformPutPasswordDart
          >('vault_platform_put_password'),
      _vault_platform_enable = library
          .lookupFunction<_VaultPlatformEnableNative, _VaultPlatformEnableDart>(
            'vault_platform_enable',
          ),
      _vault_platform_disable = library
          .lookupFunction<
            _VaultPlatformDisableNative,
            _VaultPlatformDisableDart
          >('vault_platform_disable'),
      _vault_platform_disabled = library
          .lookupFunction<
            _VaultPlatformDisabledNative,
            _VaultPlatformDisabledDart
          >('vault_platform_disabled'),
      _vault_platform_get_password = library
          .lookupFunction<
            _VaultPlatformGetPasswordNative,
            _VaultPlatformGetPasswordDart
          >('vault_platform_get_password'),
      _vault_default_directory = library
          .lookupFunction<
            _VaultDefaultDirectoryNative,
            _VaultDefaultDirectoryDart
          >('vault_default_directory'),
      _vault_default_path = library
          .lookupFunction<_VaultDefaultPathNative, _VaultDefaultPathDart>(
            'vault_default_path',
          ),
      _vault_agent_log_path = library
          .lookupFunction<_VaultAgentLogPathNative, _VaultAgentLogPathDart>(
            'vault_agent_log_path',
          ),
      _vault_agent_log_destination = library
          .lookupFunction<
            _VaultAgentLogDestinationNative,
            _VaultAgentLogDestinationDart
          >('vault_agent_log_destination'),
      _vault_agent_get_vault_unlock_key = library
          .lookupFunction<
            _VaultAgentGetVaultUnlockKeyNative,
            _VaultAgentGetVaultUnlockKeyDart
          >('vault_agent_get_vault_unlock_key'),
      _vault_agent_put_vault_unlock_key = library
          .lookupFunction<
            _VaultAgentPutVaultUnlockKeyNative,
            _VaultAgentPutVaultUnlockKeyDart
          >('vault_agent_put_vault_unlock_key'),
      _vault_agent_forget_vault_unlock_key = library
          .lookupFunction<
            _VaultAgentForgetVaultUnlockKeyNative,
            _VaultAgentForgetVaultUnlockKeyDart
          >('vault_agent_forget_vault_unlock_key'),
      _vault_agent_get_owner_signing_key = library
          .lookupFunction<
            _VaultAgentGetOwnerSigningKeyNative,
            _VaultAgentGetOwnerSigningKeyDart
          >('vault_agent_get_owner_signing_key'),
      _vault_agent_put_owner_signing_key = library
          .lookupFunction<
            _VaultAgentPutOwnerSigningKeyNative,
            _VaultAgentPutOwnerSigningKeyDart
          >('vault_agent_put_owner_signing_key'),
      _vault_agent_forget_owner_signing_key = library
          .lookupFunction<
            _VaultAgentForgetOwnerSigningKeyNative,
            _VaultAgentForgetOwnerSigningKeyDart
          >('vault_agent_forget_owner_signing_key'),
      _vault_agent_begin_activity = library
          .lookupFunction<
            _VaultAgentBeginActivityNative,
            _VaultAgentBeginActivityDart
          >('vault_agent_begin_activity'),
      _vault_agent_end_activity = library
          .lookupFunction<
            _VaultAgentEndActivityNative,
            _VaultAgentEndActivityDart
          >('vault_agent_end_activity'),
      _vault_local = library.lookupFunction<_VaultLocalNative, _VaultLocalDart>(
        'vault_local',
      ),
      _vault_create_lockbox_password = library
          .lookupFunction<
            _VaultCreateLockboxPasswordNative,
            _VaultCreateLockboxPasswordDart
          >('vault_create_lockbox_password'),
      _vault_open_lockbox_password = library
          .lookupFunction<
            _VaultOpenLockboxPasswordNative,
            _VaultOpenLockboxPasswordDart
          >('vault_open_lockbox_password'),
      _vault_create_lockbox_content_key = library
          .lookupFunction<
            _VaultCreateLockboxContentKeyNative,
            _VaultCreateLockboxContentKeyDart
          >('vault_create_lockbox_content_key'),
      _vault_create_lockbox_contact = library
          .lookupFunction<
            _VaultCreateLockboxContactNative,
            _VaultCreateLockboxContactDart
          >('vault_create_lockbox_contact'),
      _vault_open_lockbox_content_key = library
          .lookupFunction<
            _VaultOpenLockboxContentKeyNative,
            _VaultOpenLockboxContentKeyDart
          >('vault_open_lockbox_content_key'),
      _vault_cache_lockbox_password = library
          .lookupFunction<
            _VaultCacheLockboxPasswordNative,
            _VaultCacheLockboxPasswordDart
          >('vault_cache_lockbox_password'),
      _vault_close_lockbox = library
          .lookupFunction<_VaultCloseLockboxNative, _VaultCloseLockboxDart>(
            'vault_close_lockbox',
          ),
      _vault_close_all = library
          .lookupFunction<_VaultCloseAllNative, _VaultCloseAllDart>(
            'vault_close_all',
          ),
      _vault_free = library.lookupFunction<_VaultFreeNative, _VaultFreeDart>(
        'vault_free',
      );
  final _ApiAbiVersionDart _api_abi_version;
  int api_abi_version() => _api_abi_version();
  final _BufferLastErrorDart _buffer_last_error;
  ffi.Pointer<ffi.Uint8> buffer_last_error() => _buffer_last_error();
  final _BufferLastErrorDetailsDart _buffer_last_error_details;
  RevaultBuffer buffer_last_error_details() => _buffer_last_error_details();
  final _BufferFreeDart _buffer_free;
  void buffer_free(RevaultBuffer value) => _buffer_free(value);
  final _SecretLenDart _secret_len;
  bool secret_len(ffi.Pointer<ffi.Void> handle, ffi.Pointer<ffi.Size> outLen) =>
      _secret_len(handle, outLen);
  final _SecretCopyDart _secret_copy;
  bool secret_copy(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> destination,
    int destinationLen,
  ) => _secret_copy(handle, destination, destinationLen);
  final _SecretFreeDart _secret_free;
  void secret_free(ffi.Pointer<ffi.Void> handle) => _secret_free(handle);
  final _LockboxFormatVersionDart _lockbox_format_version;
  int lockbox_format_version() => _lockbox_format_version();
  final _LockboxProbeFormatVersionDart _lockbox_probe_format_version;
  int lockbox_probe_format_version(ffi.Pointer<ffi.Uint8> bytes, int len) =>
      _lockbox_probe_format_version(bytes, len);
  final _LockboxCreateDart _lockbox_create;
  ffi.Pointer<ffi.Void> lockbox_create(
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
  ) => _lockbox_create(key, key_len);
  final _LockboxCreateWithOptionsDart _lockbox_create_with_options;
  ffi.Pointer<ffi.Void> lockbox_create_with_options(
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
    ffi.Pointer<ffi.Uint8> cache_mode,
    int cache_len,
    int cache_bytes,
    ffi.Pointer<ffi.Uint8> workload,
    int workload_len,
    ffi.Pointer<ffi.Uint8> worker,
    int worker_len,
    int jobs,
  ) => _lockbox_create_with_options(
    key,
    key_len,
    cache_mode,
    cache_len,
    cache_bytes,
    workload,
    workload_len,
    worker,
    worker_len,
    jobs,
  );
  final _LockboxCreatePasswordDart _lockbox_create_password;
  ffi.Pointer<ffi.Void> lockbox_create_password(
    ffi.Pointer<ffi.Uint8> password,
    int len,
  ) => _lockbox_create_password(password, len);
  final _LockboxCreateContactDart _lockbox_create_contact;
  ffi.Pointer<ffi.Void> lockbox_create_contact(ffi.Pointer<ffi.Void> contact) =>
      _lockbox_create_contact(contact);
  final _LockboxCreateWithSigningKeyDart _lockbox_create_with_signing_key;
  ffi.Pointer<ffi.Void> lockbox_create_with_signing_key(
    ffi.Pointer<ffi.Uint8> content_key,
    int key_len,
    ffi.Pointer<ffi.Void> signing_key,
  ) => _lockbox_create_with_signing_key(content_key, key_len, signing_key);
  final _LockboxOpenDart _lockbox_open;
  ffi.Pointer<ffi.Void> lockbox_open(
    ffi.Pointer<ffi.Uint8> archive,
    int archive_len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
  ) => _lockbox_open(archive, archive_len, key, key_len);
  final _LockboxOpenWithOptionsDart _lockbox_open_with_options;
  ffi.Pointer<ffi.Void> lockbox_open_with_options(
    ffi.Pointer<ffi.Uint8> archive,
    int archive_len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
    ffi.Pointer<ffi.Uint8> cache_mode,
    int cache_len,
    int cache_bytes,
    ffi.Pointer<ffi.Uint8> workload,
    int workload_len,
    ffi.Pointer<ffi.Uint8> worker,
    int worker_len,
    int jobs,
  ) => _lockbox_open_with_options(
    archive,
    archive_len,
    key,
    key_len,
    cache_mode,
    cache_len,
    cache_bytes,
    workload,
    workload_len,
    worker,
    worker_len,
    jobs,
  );
  final _LockboxOpenPasswordDart _lockbox_open_password;
  ffi.Pointer<ffi.Void> lockbox_open_password(
    ffi.Pointer<ffi.Uint8> archive,
    int archive_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _lockbox_open_password(archive, archive_len, password, password_len);
  final _LockboxOpenContactDart _lockbox_open_contact;
  ffi.Pointer<ffi.Void> lockbox_open_contact(
    ffi.Pointer<ffi.Uint8> archive,
    int archive_len,
    ffi.Pointer<ffi.Void> contact,
  ) => _lockbox_open_contact(archive, archive_len, contact);
  final _LockboxAddFileDart _lockbox_add_file;
  bool lockbox_add_file(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> data,
    int data_len,
    bool replace,
  ) => _lockbox_add_file(handle, path, path_len, data, data_len, replace);
  final _LockboxAddFileWithPermissionsDart _lockbox_add_file_with_permissions;
  bool lockbox_add_file_with_permissions(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> data,
    int data_len,
    int permissions,
    bool replace,
  ) => _lockbox_add_file_with_permissions(
    handle,
    path,
    path_len,
    data,
    data_len,
    permissions,
    replace,
  );
  final _LockboxGetFileDart _lockbox_get_file;
  RevaultBuffer lockbox_get_file(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_get_file(handle, path, path_len);
  final _LockboxExtractFileDart _lockbox_extract_file;
  bool lockbox_extract_file(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> source,
    int source_len,
    ffi.Pointer<ffi.Uint8> destination,
    int destination_len,
    bool replace,
  ) => _lockbox_extract_file(
    handle,
    source,
    source_len,
    destination,
    destination_len,
    replace,
  );
  final _LockboxExtractDirectoryDart _lockbox_extract_directory;
  bool lockbox_extract_directory(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> destination,
    int destination_len,
    int max_file_bytes,
    int max_total_bytes,
    int max_files,
    bool restore_symlinks,
    bool restore_permissions,
    bool overwrite,
  ) => _lockbox_extract_directory(
    handle,
    destination,
    destination_len,
    max_file_bytes,
    max_total_bytes,
    max_files,
    restore_symlinks,
    restore_permissions,
    overwrite,
  );
  final _LockboxStreamContentDart _lockbox_stream_content;
  RevaultBuffer lockbox_stream_content(
    ffi.Pointer<ffi.Void> handle,
    bool physical,
  ) => _lockbox_stream_content(handle, physical);
  final _LockboxCacheStatsDart _lockbox_cache_stats;
  RevaultBuffer lockbox_cache_stats(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_cache_stats(handle);
  final _LockboxImportStatsDart _lockbox_import_stats;
  RevaultBuffer lockbox_import_stats(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_import_stats(handle);
  final _LockboxResetImportStatsDart _lockbox_reset_import_stats;
  bool lockbox_reset_import_stats(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_reset_import_stats(handle);
  final _LockboxInspectFileDart _lockbox_inspect_file;
  RevaultBuffer lockbox_inspect_file(
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_inspect_file(path, path_len);
  final _LockboxPageInspectionDart _lockbox_page_inspection;
  RevaultBuffer lockbox_page_inspection(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_page_inspection(handle);
  final _LockboxRecoveryReportDart _lockbox_recovery_report;
  RevaultBuffer lockbox_recovery_report(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_recovery_report(handle);
  final _LockboxRecoveryReportRenderDart _lockbox_recovery_report_render;
  RevaultBuffer lockbox_recovery_report_render(
    ffi.Pointer<ffi.Void> handle,
    bool verbose,
    int max_entries,
  ) => _lockbox_recovery_report_render(handle, verbose, max_entries);
  final _LockboxRecoveryScanPathDart _lockbox_recovery_scan_path;
  RevaultBuffer lockbox_recovery_scan_path(
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
  ) => _lockbox_recovery_scan_path(path, path_len, key, key_len);
  final _LockboxStorageLenDart _lockbox_storage_len;
  int lockbox_storage_len(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_storage_len(handle);
  final _LockboxSetWorkloadProfileDart _lockbox_set_workload_profile;
  bool lockbox_set_workload_profile(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> profile,
    int profile_len,
  ) => _lockbox_set_workload_profile(handle, profile, profile_len);
  final _LockboxSetWorkerPolicyDart _lockbox_set_worker_policy;
  bool lockbox_set_worker_policy(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> mode,
    int mode_len,
    int jobs,
  ) => _lockbox_set_worker_policy(handle, mode, mode_len, jobs);
  final _LockboxRuntimeOptionsDart _lockbox_runtime_options;
  RevaultBuffer lockbox_runtime_options(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_runtime_options(handle);
  final _LockboxCommitDart _lockbox_commit;
  bool lockbox_commit(ffi.Pointer<ffi.Void> handle) => _lockbox_commit(handle);
  final _LockboxCreateDirDart _lockbox_create_dir;
  bool lockbox_create_dir(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    bool create_parents,
  ) => _lockbox_create_dir(handle, path, path_len, create_parents);
  final _LockboxDeleteDart _lockbox_delete;
  bool lockbox_delete(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_delete(handle, path, path_len);
  final _LockboxRemoveDirDart _lockbox_remove_dir;
  bool lockbox_remove_dir(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    bool recursive,
  ) => _lockbox_remove_dir(handle, path, path_len, recursive);
  final _LockboxCreateParentDirsDart _lockbox_create_parent_dirs;
  bool lockbox_create_parent_dirs(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_create_parent_dirs(handle, path, path_len);
  final _LockboxRenameDart _lockbox_rename;
  bool lockbox_rename(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> from,
    int from_len,
    ffi.Pointer<ffi.Uint8> to,
    int to_len,
  ) => _lockbox_rename(handle, from, from_len, to, to_len);
  final _LockboxListDart _lockbox_list;
  RevaultBuffer lockbox_list(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    bool recursive,
  ) => _lockbox_list(handle, path, path_len, recursive);
  final _LockboxListWithOptionsDart _lockbox_list_with_options;
  RevaultBuffer lockbox_list_with_options(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> glob,
    int glob_len,
    bool recursive,
    bool include_files,
    bool include_symlinks,
    bool include_directories,
    int limit,
  ) => _lockbox_list_with_options(
    handle,
    path,
    path_len,
    glob,
    glob_len,
    recursive,
    include_files,
    include_symlinks,
    include_directories,
    limit,
  );
  final _LockboxStatDart _lockbox_stat;
  RevaultBuffer lockbox_stat(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_stat(handle, path, path_len);
  final _LockboxSetVariableDart _lockbox_set_variable;
  bool lockbox_set_variable(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Uint8> value,
    int value_len,
  ) => _lockbox_set_variable(handle, name, name_len, value, value_len);
  final _LockboxSetSecretVariableDart _lockbox_set_secret_variable;
  bool lockbox_set_secret_variable(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int nameLen,
    ffi.Pointer<ffi.Uint8> value,
    int valueLen,
  ) => _lockbox_set_secret_variable(handle, name, nameLen, value, valueLen);
  final _LockboxGetVariableDart _lockbox_get_variable;
  RevaultBuffer lockbox_get_variable(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _lockbox_get_variable(handle, name, name_len);
  final _LockboxGetSecretVariableDart _lockbox_get_secret_variable;
  bool lockbox_get_secret_variable(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int nameLen,
    ffi.Pointer<ffi.Pointer<ffi.Void>> output,
  ) => _lockbox_get_secret_variable(handle, name, nameLen, output);
  final _LockboxDeleteVariableDart _lockbox_delete_variable;
  bool lockbox_delete_variable(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _lockbox_delete_variable(handle, name, name_len);
  final _LockboxMoveVariablesDart _lockbox_move_variables;
  bool lockbox_move_variables(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> moves_proto,
    int moves_len,
  ) => _lockbox_move_variables(handle, moves_proto, moves_len);
  final _LockboxListVariablesDart _lockbox_list_variables;
  RevaultBuffer lockbox_list_variables(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_list_variables(handle);
  final _LockboxVariableSensitivityDart _lockbox_variable_sensitivity;
  RevaultBuffer lockbox_variable_sensitivity(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _lockbox_variable_sensitivity(handle, name, name_len);
  final _LockboxAddSymlinkDart _lockbox_add_symlink;
  bool lockbox_add_symlink(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> target,
    int target_len,
    bool replace,
  ) =>
      _lockbox_add_symlink(handle, path, path_len, target, target_len, replace);
  final _LockboxGetSymlinkTargetDart _lockbox_get_symlink_target;
  RevaultBuffer lockbox_get_symlink_target(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_get_symlink_target(handle, path, path_len);
  final _LockboxIdDart _lockbox_id;
  RevaultBuffer lockbox_id(ffi.Pointer<ffi.Void> handle) => _lockbox_id(handle);
  final _LockboxExistsDart _lockbox_exists;
  bool lockbox_exists(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_exists(handle, path, path_len);
  final _LockboxIsDirDart _lockbox_is_dir;
  bool lockbox_is_dir(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_is_dir(handle, path, path_len);
  final _LockboxPermissionsDart _lockbox_permissions;
  int lockbox_permissions(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_permissions(handle, path, path_len);
  final _LockboxSetPermissionsDart _lockbox_set_permissions;
  bool lockbox_set_permissions(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    int permissions,
  ) => _lockbox_set_permissions(handle, path, path_len, permissions);
  final _LockboxReadRangeDart _lockbox_read_range;
  RevaultBuffer lockbox_read_range(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    int offset,
    int len,
  ) => _lockbox_read_range(handle, path, path_len, offset, len);
  final _LockboxRecoveryScanDart _lockbox_recovery_scan;
  RevaultBuffer lockbox_recovery_scan(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
  ) => _lockbox_recovery_scan(bytes, len, key, key_len);
  final _LockboxRecoverySalvageDart _lockbox_recovery_salvage;
  ffi.Pointer<ffi.Void> lockbox_recovery_salvage(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
    ffi.Pointer<ffi.Void> signing_key,
  ) => _lockbox_recovery_salvage(bytes, len, key, key_len, signing_key);
  final _LockboxAddPasswordDart _lockbox_add_password;
  int lockbox_add_password(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> password,
    int len,
  ) => _lockbox_add_password(handle, password, len);
  final _LockboxAddContactDart _lockbox_add_contact;
  int lockbox_add_contact(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Void> contact,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _lockbox_add_contact(handle, contact, name, name_len);
  final _LockboxDeleteKeyDart _lockbox_delete_key;
  bool lockbox_delete_key(ffi.Pointer<ffi.Void> handle, int id) =>
      _lockbox_delete_key(handle, id);
  final _LockboxListKeySlotsDart _lockbox_list_key_slots;
  RevaultBuffer lockbox_list_key_slots(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_list_key_slots(handle);
  final _LockboxSetOwnerSigningKeyDart _lockbox_set_owner_signing_key;
  bool lockbox_set_owner_signing_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Void> key,
  ) => _lockbox_set_owner_signing_key(handle, key);
  final _LockboxOwnerInspectionDart _lockbox_owner_inspection;
  RevaultBuffer lockbox_owner_inspection(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_owner_inspection(handle);
  final _LockboxDefineFormDart _lockbox_define_form;
  RevaultBuffer lockbox_define_form(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> alias,
    int alias_len,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Uint8> description,
    int description_len,
    ffi.Pointer<ffi.Uint8> fields_proto,
    int fields_len,
  ) => _lockbox_define_form(
    handle,
    alias,
    alias_len,
    name,
    name_len,
    description,
    description_len,
    fields_proto,
    fields_len,
  );
  final _LockboxListFormDefinitionsDart _lockbox_list_form_definitions;
  RevaultBuffer lockbox_list_form_definitions(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_list_form_definitions(handle);
  final _LockboxResolveFormDart _lockbox_resolve_form;
  RevaultBuffer lockbox_resolve_form(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> reference,
    int reference_len,
  ) => _lockbox_resolve_form(handle, reference, reference_len);
  final _LockboxListFormRevisionsDart _lockbox_list_form_revisions;
  RevaultBuffer lockbox_list_form_revisions(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> type_id,
    int type_id_len,
  ) => _lockbox_list_form_revisions(handle, type_id, type_id_len);
  final _LockboxCreateFormRecordDart _lockbox_create_form_record;
  RevaultBuffer lockbox_create_form_record(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> type_reference,
    int type_len,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _lockbox_create_form_record(
    handle,
    path,
    path_len,
    type_reference,
    type_len,
    name,
    name_len,
  );
  final _LockboxSetFormFieldDart _lockbox_set_form_field;
  bool lockbox_set_form_field(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> field,
    int field_len,
    ffi.Pointer<ffi.Uint8> value,
    int value_len,
  ) => _lockbox_set_form_field(
    handle,
    path,
    path_len,
    field,
    field_len,
    value,
    value_len,
  );
  final _LockboxSetSecretFormFieldDart _lockbox_set_secret_form_field;
  bool lockbox_set_secret_form_field(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int pathLen,
    ffi.Pointer<ffi.Uint8> field,
    int fieldLen,
    ffi.Pointer<ffi.Uint8> value,
    int valueLen,
  ) => _lockbox_set_secret_form_field(
    handle,
    path,
    pathLen,
    field,
    fieldLen,
    value,
    valueLen,
  );
  final _LockboxListFormRecordsDart _lockbox_list_form_records;
  RevaultBuffer lockbox_list_form_records(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_list_form_records(handle);
  final _LockboxGetFormRecordDart _lockbox_get_form_record;
  RevaultBuffer lockbox_get_form_record(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_get_form_record(handle, path, path_len);
  final _LockboxDeleteFormRecordDart _lockbox_delete_form_record;
  bool lockbox_delete_form_record(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _lockbox_delete_form_record(handle, path, path_len);
  final _LockboxMoveFormRecordsDart _lockbox_move_form_records;
  bool lockbox_move_form_records(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> moves_proto,
    int moves_len,
  ) => _lockbox_move_form_records(handle, moves_proto, moves_len);
  final _LockboxGetFormFieldDart _lockbox_get_form_field;
  RevaultBuffer lockbox_get_form_field(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> field,
    int field_len,
  ) => _lockbox_get_form_field(handle, path, path_len, field, field_len);
  final _LockboxGetSecretFormFieldDart _lockbox_get_secret_form_field;
  bool lockbox_get_secret_form_field(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int pathLen,
    ffi.Pointer<ffi.Uint8> field,
    int fieldLen,
    ffi.Pointer<ffi.Pointer<ffi.Void>> output,
  ) => _lockbox_get_secret_form_field(
    handle,
    path,
    pathLen,
    field,
    fieldLen,
    output,
  );
  final _LockboxToBytesDart _lockbox_to_bytes;
  RevaultBuffer lockbox_to_bytes(ffi.Pointer<ffi.Void> handle) =>
      _lockbox_to_bytes(handle);
  final _LockboxFreeDart _lockbox_free;
  void lockbox_free(ffi.Pointer<ffi.Void> handle) => _lockbox_free(handle);
  final _VaultIsRunningDart _vault_is_running;
  bool vault_is_running() => _vault_is_running();
  final _VaultForgetAllDart _vault_forget_all;
  bool vault_forget_all() => _vault_forget_all();
  final _KeyContactGenerateDart _key_contact_generate;
  ffi.Pointer<ffi.Void> key_contact_generate() => _key_contact_generate();
  final _KeyContactFromPrivateDart _key_contact_from_private;
  ffi.Pointer<ffi.Void> key_contact_from_private(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _key_contact_from_private(bytes, len);
  final _KeyContactPublicDart _key_contact_public;
  RevaultBuffer key_contact_public(ffi.Pointer<ffi.Void> handle) =>
      _key_contact_public(handle);
  final _KeyContactPrivateDart _key_contact_private;
  RevaultBuffer key_contact_private(ffi.Pointer<ffi.Void> handle) =>
      _key_contact_private(handle);
  final _KeyContactPublicFromBytesDart _key_contact_public_from_bytes;
  ffi.Pointer<ffi.Void> key_contact_public_from_bytes(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _key_contact_public_from_bytes(bytes, len);
  final _KeyContactPublicFreeDart _key_contact_public_free;
  void key_contact_public_free(ffi.Pointer<ffi.Void> handle) =>
      _key_contact_public_free(handle);
  final _KeyContactFreeDart _key_contact_free;
  void key_contact_free(ffi.Pointer<ffi.Void> handle) =>
      _key_contact_free(handle);
  final _KeyContactEncryptDart _key_contact_encrypt;
  ffi.Pointer<ffi.Void> key_contact_encrypt(
    ffi.Pointer<ffi.Void> contact,
    ffi.Pointer<ffi.Uint8> content_key,
    int key_len,
  ) => _key_contact_encrypt(contact, content_key, key_len);
  final _KeyContactDecryptDart _key_contact_decrypt;
  RevaultBuffer key_contact_decrypt(
    ffi.Pointer<ffi.Void> contact,
    ffi.Pointer<ffi.Void> wrapped,
  ) => _key_contact_decrypt(contact, wrapped);
  final _KeyContactWrappedPublicDart _key_contact_wrapped_public;
  RevaultBuffer key_contact_wrapped_public(ffi.Pointer<ffi.Void> wrapped) =>
      _key_contact_wrapped_public(wrapped);
  final _KeyContactWrappedCiphertextDart _key_contact_wrapped_ciphertext;
  RevaultBuffer key_contact_wrapped_ciphertext(ffi.Pointer<ffi.Void> wrapped) =>
      _key_contact_wrapped_ciphertext(wrapped);
  final _KeyContactWrappedEncryptedDart _key_contact_wrapped_encrypted;
  RevaultBuffer key_contact_wrapped_encrypted(ffi.Pointer<ffi.Void> wrapped) =>
      _key_contact_wrapped_encrypted(wrapped);
  final _KeyContactWrappedFreeDart _key_contact_wrapped_free;
  void key_contact_wrapped_free(ffi.Pointer<ffi.Void> handle) =>
      _key_contact_wrapped_free(handle);
  final _KeySigningGenerateDart _key_signing_generate;
  ffi.Pointer<ffi.Void> key_signing_generate() => _key_signing_generate();
  final _KeySigningFromPrivateDart _key_signing_from_private;
  ffi.Pointer<ffi.Void> key_signing_from_private(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _key_signing_from_private(bytes, len);
  final _KeySigningPublicDart _key_signing_public;
  RevaultBuffer key_signing_public(ffi.Pointer<ffi.Void> handle) =>
      _key_signing_public(handle);
  final _KeySigningPrivateDart _key_signing_private;
  RevaultBuffer key_signing_private(ffi.Pointer<ffi.Void> handle) =>
      _key_signing_private(handle);
  final _KeySigningPublicFromBytesDart _key_signing_public_from_bytes;
  ffi.Pointer<ffi.Void> key_signing_public_from_bytes(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _key_signing_public_from_bytes(bytes, len);
  final _KeySigningPublicFreeDart _key_signing_public_free;
  void key_signing_public_free(ffi.Pointer<ffi.Void> handle) =>
      _key_signing_public_free(handle);
  final _KeySigningFreeDart _key_signing_free;
  void key_signing_free(ffi.Pointer<ffi.Void> handle) =>
      _key_signing_free(handle);
  final _VaultKeyExportPrivateDart _vault_key_export_private;
  RevaultBuffer vault_key_export_private(
    ffi.Pointer<ffi.Void> key,
    ffi.Pointer<ffi.Uint8> format,
    int format_len,
  ) => _vault_key_export_private(key, format, format_len);
  final _VaultKeyExportPublicDart _vault_key_export_public;
  RevaultBuffer vault_key_export_public(
    ffi.Pointer<ffi.Void> key,
    ffi.Pointer<ffi.Uint8> format,
    int format_len,
  ) => _vault_key_export_public(key, format, format_len);
  final _VaultKeyImportPrivateDart _vault_key_import_private;
  ffi.Pointer<ffi.Void> vault_key_import_private(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _vault_key_import_private(bytes, len);
  final _VaultKeyImportPublicDart _vault_key_import_public;
  ffi.Pointer<ffi.Void> vault_key_import_public(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _vault_key_import_public(bytes, len);
  final _VaultKeyFingerprintDart _vault_key_fingerprint;
  RevaultBuffer vault_key_fingerprint(ffi.Pointer<ffi.Void> key) =>
      _vault_key_fingerprint(key);
  final _VaultKeyFormatHexDart _vault_key_format_hex;
  RevaultBuffer vault_key_format_hex(ffi.Pointer<ffi.Uint8> bytes, int len) =>
      _vault_key_format_hex(bytes, len);
  final _VaultKeyDecodeHexDart _vault_key_decode_hex;
  RevaultBuffer vault_key_decode_hex(ffi.Pointer<ffi.Uint8> text, int len) =>
      _vault_key_decode_hex(text, len);
  final _VaultKeyFormatCrockfordDart _vault_key_format_crockford;
  RevaultBuffer vault_key_format_crockford(
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _vault_key_format_crockford(bytes, len);
  final _VaultKeyFormatCrockfordReadingDart _vault_key_format_crockford_reading;
  RevaultBuffer vault_key_format_crockford_reading(
    ffi.Pointer<ffi.Uint8> code,
    int len,
  ) => _vault_key_format_crockford_reading(code, len);
  final _VaultKeyDecodeCrockfordDart _vault_key_decode_crockford;
  RevaultBuffer vault_key_decode_crockford(
    ffi.Pointer<ffi.Uint8> code,
    int len,
  ) => _vault_key_decode_crockford(code, len);
  final _VaultKeyHexEncodeDart _vault_key_hex_encode;
  RevaultBuffer vault_key_hex_encode(ffi.Pointer<ffi.Uint8> bytes, int len) =>
      _vault_key_hex_encode(bytes, len);
  final _VaultKeyHexDecodeDart _vault_key_hex_decode;
  RevaultBuffer vault_key_hex_decode(ffi.Pointer<ffi.Uint8> text, int len) =>
      _vault_key_hex_decode(text, len);
  final _VaultDirectoryOpenDart _vault_directory_open;
  ffi.Pointer<ffi.Void> vault_directory_open(
    ffi.Pointer<ffi.Uint8> root,
    int root_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_open(root, root_len, password, password_len);
  final _VaultStructureVersionCurrentDart _vault_structure_version_current;
  int vault_structure_version_current() => _vault_structure_version_current();
  final _VaultDirectoryProbeStructureVersionDart
  _vault_directory_probe_structure_version;
  int vault_directory_probe_structure_version(
    ffi.Pointer<ffi.Uint8> root,
    int root_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_probe_structure_version(
    root,
    root_len,
    password,
    password_len,
  );
  final _VaultDirectoryOpenOrCreateDefaultDart
  _vault_directory_open_or_create_default;
  ffi.Pointer<ffi.Void> vault_directory_open_or_create_default(
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_open_or_create_default(password, password_len);
  final _VaultDirectoryReplaceDefaultDart _vault_directory_replace_default;
  ffi.Pointer<ffi.Void> vault_directory_replace_default(
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_replace_default(password, password_len);
  final _VaultDirectoryChangePasswordDart _vault_directory_change_password;
  bool vault_directory_change_password(
    ffi.Pointer<ffi.Uint8> root,
    int root_len,
    ffi.Pointer<ffi.Uint8> old_password,
    int old_len,
    ffi.Pointer<ffi.Uint8> new_password,
    int new_len,
  ) => _vault_directory_change_password(
    root,
    root_len,
    old_password,
    old_len,
    new_password,
    new_len,
  );
  final _VaultDirectoryChangeDefaultPasswordDart
  _vault_directory_change_default_password;
  bool vault_directory_change_default_password(
    ffi.Pointer<ffi.Uint8> old_password,
    int old_len,
    ffi.Pointer<ffi.Uint8> new_password,
    int new_len,
  ) => _vault_directory_change_default_password(
    old_password,
    old_len,
    new_password,
    new_len,
  );
  final _VaultDirectoryReplaceDart _vault_directory_replace;
  ffi.Pointer<ffi.Void> vault_directory_replace(
    ffi.Pointer<ffi.Uint8> root,
    int root_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_replace(root, root_len, password, password_len);
  final _VaultDirectoryOpenOrCreateDart _vault_directory_open_or_create;
  ffi.Pointer<ffi.Void> vault_directory_open_or_create(
    ffi.Pointer<ffi.Uint8> root,
    int root_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_open_or_create(root, root_len, password, password_len);
  final _VaultDirectoryRootDart _vault_directory_root;
  RevaultBuffer vault_directory_root(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_root(handle);
  final _VaultDirectoryStructureVersionDart _vault_directory_structure_version;
  int vault_directory_structure_version(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_structure_version(handle);
  final _VaultDirectoryListPrivateKeysDart _vault_directory_list_private_keys;
  RevaultBuffer vault_directory_list_private_keys(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_directory_list_private_keys(handle);
  final _VaultDirectoryListPrivateKeyNamesDart
  _vault_directory_list_private_key_names;
  RevaultBuffer vault_directory_list_private_key_names(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_directory_list_private_key_names(handle);
  final _VaultDirectoryListContactNamesDart _vault_directory_list_contact_names;
  RevaultBuffer vault_directory_list_contact_names(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_directory_list_contact_names(handle);
  final _VaultDirectoryListFormAliasesDart _vault_directory_list_form_aliases;
  RevaultBuffer vault_directory_list_form_aliases(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_directory_list_form_aliases(handle);
  final _VaultDirectoryPrivateKeyExistsDart _vault_directory_private_key_exists;
  bool vault_directory_private_key_exists(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_private_key_exists(handle, name, name_len);
  final _VaultDirectoryDeletePrivateKeyDart _vault_directory_delete_private_key;
  bool vault_directory_delete_private_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_delete_private_key(handle, name, name_len);
  final _VaultDirectoryStorePrivateKeyDart _vault_directory_store_private_key;
  bool vault_directory_store_private_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Void> key,
  ) => _vault_directory_store_private_key(handle, name, name_len, key);
  final _VaultDirectoryLoadPrivateKeyDart _vault_directory_load_private_key;
  ffi.Pointer<ffi.Void> vault_directory_load_private_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_load_private_key(handle, name, name_len);
  final _VaultDirectoryLoadPrivateKeyGenerationDart
  _vault_directory_load_private_key_generation;
  ffi.Pointer<ffi.Void> vault_directory_load_private_key_generation(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    int index,
  ) => _vault_directory_load_private_key_generation(
    handle,
    name,
    name_len,
    index,
  );
  final _VaultDirectoryStoreContactDart _vault_directory_store_contact;
  bool vault_directory_store_contact(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Void> key,
  ) => _vault_directory_store_contact(handle, name, name_len, key);
  final _VaultDirectoryLoadContactDart _vault_directory_load_contact;
  ffi.Pointer<ffi.Void> vault_directory_load_contact(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_load_contact(handle, name, name_len);
  final _VaultDirectoryContactExistsDart _vault_directory_contact_exists;
  bool vault_directory_contact_exists(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_contact_exists(handle, name, name_len);
  final _VaultDirectoryDeleteContactDart _vault_directory_delete_contact;
  bool vault_directory_delete_contact(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_delete_contact(handle, name, name_len);
  final _VaultDirectoryListContactsDart _vault_directory_list_contacts;
  RevaultBuffer vault_directory_list_contacts(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_list_contacts(handle);
  final _VaultDirectoryStoreProfileEmailDart
  _vault_directory_store_profile_email;
  bool vault_directory_store_profile_email(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Uint8> email,
    int email_len,
  ) => _vault_directory_store_profile_email(
    handle,
    name,
    name_len,
    email,
    email_len,
  );
  final _VaultDirectoryProfileEmailDart _vault_directory_profile_email;
  RevaultBuffer vault_directory_profile_email(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_profile_email(handle, name, name_len);
  final _VaultDirectoryStoreBackupDart _vault_directory_store_backup;
  bool vault_directory_store_backup(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    ffi.Pointer<ffi.Uint8> bytes,
    int len,
  ) => _vault_directory_store_backup(handle, id, id_len, bytes, len);
  final _VaultDirectoryLoadBackupDart _vault_directory_load_backup;
  RevaultBuffer vault_directory_load_backup(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
  ) => _vault_directory_load_backup(handle, id, id_len);
  final _VaultDirectoryBackupCountDart _vault_directory_backup_count;
  int vault_directory_backup_count(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_backup_count(handle);
  final _VaultDirectoryRestorePrivateKeyDart
  _vault_directory_restore_private_key;
  bool vault_directory_restore_private_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Void> key,
    ffi.Pointer<ffi.Void> signing_key,
    bool overwrite,
  ) => _vault_directory_restore_private_key(
    handle,
    name,
    name_len,
    key,
    signing_key,
    overwrite,
  );
  final _VaultDirectoryLoadOwnerSigningKeyDart
  _vault_directory_load_owner_signing_key;
  ffi.Pointer<ffi.Void> vault_directory_load_owner_signing_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_load_owner_signing_key(handle, name, name_len);
  final _VaultDirectoryLoadOwnerSigningKeyGenerationDart
  _vault_directory_load_owner_signing_key_generation;
  ffi.Pointer<ffi.Void> vault_directory_load_owner_signing_key_generation(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    int index,
  ) => _vault_directory_load_owner_signing_key_generation(
    handle,
    name,
    name_len,
    index,
  );
  final _VaultDirectoryStoreContactSigningKeyDart
  _vault_directory_store_contact_signing_key;
  bool vault_directory_store_contact_signing_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Void> key,
  ) => _vault_directory_store_contact_signing_key(handle, name, name_len, key);
  final _VaultDirectoryLoadContactSigningKeyDart
  _vault_directory_load_contact_signing_key;
  ffi.Pointer<ffi.Void> vault_directory_load_contact_signing_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_load_contact_signing_key(handle, name, name_len);
  final _VaultDirectoryListProfileGenerationsDart
  _vault_directory_list_profile_generations;
  RevaultBuffer vault_directory_list_profile_generations(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_list_profile_generations(handle, name, name_len);
  final _VaultDirectoryRotatePrivateKeyDart _vault_directory_rotate_private_key;
  RevaultBuffer vault_directory_rotate_private_key(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_rotate_private_key(handle, name, name_len);
  final _VaultDirectoryRememberLockboxDart _vault_directory_remember_lockbox;
  bool vault_directory_remember_lockbox(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _vault_directory_remember_lockbox(handle, id, id_len, path, path_len);
  final _VaultDirectoryListKnownLockboxesDart
  _vault_directory_list_known_lockboxes;
  RevaultBuffer vault_directory_list_known_lockboxes(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_directory_list_known_lockboxes(handle);
  final _VaultDirectoryForgetLockboxDart _vault_directory_forget_lockbox;
  bool vault_directory_forget_lockbox(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _vault_directory_forget_lockbox(handle, path, path_len);
  final _VaultDirectoryRememberAccessSlotLabelDart
  _vault_directory_remember_access_slot_label;
  bool vault_directory_remember_access_slot_label(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    int slot_id,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_remember_access_slot_label(
    handle,
    id,
    id_len,
    slot_id,
    name,
    name_len,
  );
  final _VaultDirectoryListAccessSlotLabelsDart
  _vault_directory_list_access_slot_labels;
  RevaultBuffer vault_directory_list_access_slot_labels(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
  ) => _vault_directory_list_access_slot_labels(handle, id, id_len);
  final _VaultDirectoryFindAccessSlotLabelsDart
  _vault_directory_find_access_slot_labels;
  RevaultBuffer vault_directory_find_access_slot_labels(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
  ) => _vault_directory_find_access_slot_labels(
    handle,
    id,
    id_len,
    name,
    name_len,
  );
  final _VaultDirectoryForgetAccessSlotLabelDart
  _vault_directory_forget_access_slot_label;
  bool vault_directory_forget_access_slot_label(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    int slot_id,
  ) => _vault_directory_forget_access_slot_label(handle, id, id_len, slot_id);
  final _VaultDirectoryDefineFormDart _vault_directory_define_form;
  RevaultBuffer vault_directory_define_form(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> alias,
    int alias_len,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Uint8> description,
    int description_len,
    ffi.Pointer<ffi.Uint8> fields_proto,
    int fields_len,
  ) => _vault_directory_define_form(
    handle,
    alias,
    alias_len,
    name,
    name_len,
    description,
    description_len,
    fields_proto,
    fields_len,
  );
  final _VaultDirectoryResolveFormDart _vault_directory_resolve_form;
  RevaultBuffer vault_directory_resolve_form(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> reference,
    int reference_len,
  ) => _vault_directory_resolve_form(handle, reference, reference_len);
  final _VaultDirectoryListFormsDart _vault_directory_list_forms;
  RevaultBuffer vault_directory_list_forms(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_list_forms(handle);
  final _VaultDirectoryListFormRevisionsDart
  _vault_directory_list_form_revisions;
  RevaultBuffer vault_directory_list_form_revisions(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> type_id,
    int type_id_len,
  ) => _vault_directory_list_form_revisions(handle, type_id, type_id_len);
  final _VaultDirectorySeedFormsDart _vault_directory_seed_forms;
  int vault_directory_seed_forms(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_seed_forms(handle);
  final _VaultDirectoryRememberPasswordDart _vault_directory_remember_password;
  bool vault_directory_remember_password(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_directory_remember_password(
    handle,
    id,
    id_len,
    password,
    password_len,
  );
  final _VaultDirectoryRememberedPasswordDart
  _vault_directory_remembered_password;
  RevaultBuffer vault_directory_remembered_password(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
  ) => _vault_directory_remembered_password(handle, id, id_len);
  final _VaultBackupDefaultDart _vault_backup_default;
  RevaultBuffer vault_backup_default(
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    bool overwrite,
  ) => _vault_backup_default(path, path_len, overwrite);
  final _VaultRestoreDefaultDart _vault_restore_default;
  RevaultBuffer vault_restore_default(
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    bool overwrite,
  ) => _vault_restore_default(path, path_len, overwrite);
  final _VaultDirectoryFreeDart _vault_directory_free;
  void vault_directory_free(ffi.Pointer<ffi.Void> handle) =>
      _vault_directory_free(handle);
  final _VaultReadOnlyOpenDart _vault_read_only_open;
  ffi.Pointer<ffi.Void> vault_read_only_open(
    ffi.Pointer<ffi.Uint8> root,
    int root_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_read_only_open(root, root_len, password, password_len);
  final _VaultReadOnlyOpenDefaultDart _vault_read_only_open_default;
  ffi.Pointer<ffi.Void> vault_read_only_open_default(
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_read_only_open_default(password, password_len);
  final _VaultReadOnlyListProfileNamesDart _vault_read_only_list_profile_names;
  RevaultBuffer vault_read_only_list_profile_names(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_read_only_list_profile_names(handle);
  final _VaultReadOnlyListContactNamesDart _vault_read_only_list_contact_names;
  RevaultBuffer vault_read_only_list_contact_names(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_read_only_list_contact_names(handle);
  final _VaultReadOnlyListFormAliasesDart _vault_read_only_list_form_aliases;
  RevaultBuffer vault_read_only_list_form_aliases(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_read_only_list_form_aliases(handle);
  final _VaultReadOnlyListKnownLockboxesDart
  _vault_read_only_list_known_lockboxes;
  RevaultBuffer vault_read_only_list_known_lockboxes(
    ffi.Pointer<ffi.Void> handle,
  ) => _vault_read_only_list_known_lockboxes(handle);
  final _VaultReadOnlyFreeDart _vault_read_only_free;
  void vault_read_only_free(ffi.Pointer<ffi.Void> handle) =>
      _vault_read_only_free(handle);
  final _VaultAgentServeDart _vault_agent_serve;
  bool vault_agent_serve() => _vault_agent_serve();
  final _VaultAgentVerifyTransportDart _vault_agent_verify_transport;
  bool vault_agent_verify_transport() => _vault_agent_verify_transport();
  final _VaultAgentGetDart _vault_agent_get;
  RevaultBuffer vault_agent_get(ffi.Pointer<ffi.Uint8> id, int id_len) =>
      _vault_agent_get(id, id_len);
  final _VaultAgentPutDart _vault_agent_put;
  bool vault_agent_put(
    ffi.Pointer<ffi.Uint8> id,
    int id_len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
  ) => _vault_agent_put(id, id_len, key, key_len);
  final _VaultAgentForgetDart _vault_agent_forget;
  bool vault_agent_forget(ffi.Pointer<ffi.Uint8> id, int id_len) =>
      _vault_agent_forget(id, id_len);
  final _VaultAgentStopDart _vault_agent_stop;
  bool vault_agent_stop() => _vault_agent_stop();
  final _VaultAgentStartDart _vault_agent_start;
  bool vault_agent_start() => _vault_agent_start();
  final _VaultAgentListDart _vault_agent_list;
  RevaultBuffer vault_agent_list() => _vault_agent_list();
  final _VaultAgentSleepSupportDart _vault_agent_sleep_support;
  RevaultBuffer vault_agent_sleep_support() => _vault_agent_sleep_support();
  final _VaultPlatformStatusDart _vault_platform_status;
  RevaultBuffer vault_platform_status() => _vault_platform_status();
  final _VaultPlatformSetScopeDart _vault_platform_set_scope;
  bool vault_platform_set_scope(ffi.Pointer<ffi.Uint8> scope, int len) =>
      _vault_platform_set_scope(scope, len);
  final _VaultPlatformForgetPasswordDart _vault_platform_forget_password;
  bool vault_platform_forget_password() => _vault_platform_forget_password();
  final _VaultPlatformPutPasswordDart _vault_platform_put_password;
  bool vault_platform_put_password(ffi.Pointer<ffi.Uint8> password, int len) =>
      _vault_platform_put_password(password, len);
  final _VaultPlatformEnableDart _vault_platform_enable;
  bool vault_platform_enable() => _vault_platform_enable();
  final _VaultPlatformDisableDart _vault_platform_disable;
  bool vault_platform_disable() => _vault_platform_disable();
  final _VaultPlatformDisabledDart _vault_platform_disabled;
  bool vault_platform_disabled() => _vault_platform_disabled();
  final _VaultPlatformGetPasswordDart _vault_platform_get_password;
  RevaultBuffer vault_platform_get_password() => _vault_platform_get_password();
  final _VaultDefaultDirectoryDart _vault_default_directory;
  RevaultBuffer vault_default_directory() => _vault_default_directory();
  final _VaultDefaultPathDart _vault_default_path;
  RevaultBuffer vault_default_path() => _vault_default_path();
  final _VaultAgentLogPathDart _vault_agent_log_path;
  RevaultBuffer vault_agent_log_path() => _vault_agent_log_path();
  final _VaultAgentLogDestinationDart _vault_agent_log_destination;
  RevaultBuffer vault_agent_log_destination() => _vault_agent_log_destination();
  final _VaultAgentGetVaultUnlockKeyDart _vault_agent_get_vault_unlock_key;
  RevaultBuffer vault_agent_get_vault_unlock_key(
    ffi.Pointer<ffi.Uint8> vault_id,
    int vault_id_len,
  ) => _vault_agent_get_vault_unlock_key(vault_id, vault_id_len);
  final _VaultAgentPutVaultUnlockKeyDart _vault_agent_put_vault_unlock_key;
  bool vault_agent_put_vault_unlock_key(
    ffi.Pointer<ffi.Uint8> vault_id,
    int vault_id_len,
    ffi.Pointer<ffi.Uint8> key,
    int key_len,
    int ttl_seconds,
  ) => _vault_agent_put_vault_unlock_key(
    vault_id,
    vault_id_len,
    key,
    key_len,
    ttl_seconds,
  );
  final _VaultAgentForgetVaultUnlockKeyDart
  _vault_agent_forget_vault_unlock_key;
  bool vault_agent_forget_vault_unlock_key(
    ffi.Pointer<ffi.Uint8> vault_id,
    int vault_id_len,
  ) => _vault_agent_forget_vault_unlock_key(vault_id, vault_id_len);
  final _VaultAgentGetOwnerSigningKeyDart _vault_agent_get_owner_signing_key;
  ffi.Pointer<ffi.Void> vault_agent_get_owner_signing_key(
    ffi.Pointer<ffi.Uint8> vault_id,
    int vault_len,
    ffi.Pointer<ffi.Uint8> profile,
    int profile_len,
  ) => _vault_agent_get_owner_signing_key(
    vault_id,
    vault_len,
    profile,
    profile_len,
  );
  final _VaultAgentPutOwnerSigningKeyDart _vault_agent_put_owner_signing_key;
  bool vault_agent_put_owner_signing_key(
    ffi.Pointer<ffi.Uint8> vault_id,
    int vault_len,
    ffi.Pointer<ffi.Uint8> profile,
    int profile_len,
    ffi.Pointer<ffi.Void> key,
    int ttl_seconds,
  ) => _vault_agent_put_owner_signing_key(
    vault_id,
    vault_len,
    profile,
    profile_len,
    key,
    ttl_seconds,
  );
  final _VaultAgentForgetOwnerSigningKeyDart
  _vault_agent_forget_owner_signing_key;
  bool vault_agent_forget_owner_signing_key(
    ffi.Pointer<ffi.Uint8> vault_id,
    int vault_len,
    ffi.Pointer<ffi.Uint8> profile,
    int profile_len,
  ) => _vault_agent_forget_owner_signing_key(
    vault_id,
    vault_len,
    profile,
    profile_len,
  );
  final _VaultAgentBeginActivityDart _vault_agent_begin_activity;
  ffi.Pointer<ffi.Void> vault_agent_begin_activity(
    ffi.Pointer<ffi.Uint8> kind,
    int len,
  ) => _vault_agent_begin_activity(kind, len);
  final _VaultAgentEndActivityDart _vault_agent_end_activity;
  void vault_agent_end_activity(ffi.Pointer<ffi.Void> handle) =>
      _vault_agent_end_activity(handle);
  final _VaultLocalDart _vault_local;
  ffi.Pointer<ffi.Void> vault_local() => _vault_local();
  final _VaultCreateLockboxPasswordDart _vault_create_lockbox_password;
  ffi.Pointer<ffi.Void> vault_create_lockbox_password(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_create_lockbox_password(
    vault,
    path,
    path_len,
    password,
    password_len,
  );
  final _VaultOpenLockboxPasswordDart _vault_open_lockbox_password;
  ffi.Pointer<ffi.Void> vault_open_lockbox_password(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
  ) => _vault_open_lockbox_password(
    vault,
    path,
    path_len,
    password,
    password_len,
  );
  final _VaultCreateLockboxContentKeyDart _vault_create_lockbox_content_key;
  ffi.Pointer<ffi.Void> vault_create_lockbox_content_key(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> content_key,
    int key_len,
    ffi.Pointer<ffi.Void> signing_key,
  ) => _vault_create_lockbox_content_key(
    vault,
    path,
    path_len,
    content_key,
    key_len,
    signing_key,
  );
  final _VaultCreateLockboxContactDart _vault_create_lockbox_contact;
  ffi.Pointer<ffi.Void> vault_create_lockbox_contact(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Void> contact,
    ffi.Pointer<ffi.Uint8> name,
    int name_len,
    ffi.Pointer<ffi.Void> signing_key,
  ) => _vault_create_lockbox_contact(
    vault,
    path,
    path_len,
    contact,
    name,
    name_len,
    signing_key,
  );
  final _VaultOpenLockboxContentKeyDart _vault_open_lockbox_content_key;
  ffi.Pointer<ffi.Void> vault_open_lockbox_content_key(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> content_key,
    int key_len,
    ffi.Pointer<ffi.Void> signing_key,
  ) => _vault_open_lockbox_content_key(
    vault,
    path,
    path_len,
    content_key,
    key_len,
    signing_key,
  );
  final _VaultCacheLockboxPasswordDart _vault_cache_lockbox_password;
  bool vault_cache_lockbox_password(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
    ffi.Pointer<ffi.Uint8> password,
    int password_len,
    int ttl_seconds,
  ) => _vault_cache_lockbox_password(
    vault,
    path,
    path_len,
    password,
    password_len,
    ttl_seconds,
  );
  final _VaultCloseLockboxDart _vault_close_lockbox;
  bool vault_close_lockbox(
    ffi.Pointer<ffi.Void> vault,
    ffi.Pointer<ffi.Uint8> path,
    int path_len,
  ) => _vault_close_lockbox(vault, path, path_len);
  final _VaultCloseAllDart _vault_close_all;
  bool vault_close_all(ffi.Pointer<ffi.Void> vault) => _vault_close_all(vault);
  final _VaultFreeDart _vault_free;
  void vault_free(ffi.Pointer<ffi.Void> vault) => _vault_free(vault);
}
