// Generated complete typed operation layer. Do not edit by hand.
// ignore_for_file: public_member_api_docs

import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:typed_data';
import 'package:ffi/ffi.dart';
import 'domain_models.dart';
import 'revault_native.dart';

final class BindingOperations {
  BindingOperations(this.native) {
    if (native.api_abi_version() != 3) {
      throw StateError('revault-api native ABI mismatch; expected 3');
    }
  }
  final RevaultNative native;
  String get lastError =>
      native.buffer_last_error().cast<Utf8>().toDartString();
  bool _requireBool(bool value) {
    if (!value) {
      throw StateError(lastError);
    }
    return true;
  }

  ffi.Pointer<ffi.Void> _requireHandle(ffi.Pointer<ffi.Void> value) {
    if (value == ffi.nullptr) {
      throw StateError(lastError);
    }
    return value;
  }

  Uint8List _take(RevaultBuffer value) {
    if (value.ptr == ffi.nullptr) {
      throw StateError(lastError);
    }
    final result = Uint8List.fromList(value.ptr.asTypedList(value.len));
    native.buffer_free(value);
    return result;
  }

  String _takeString(RevaultBuffer value) => utf8.decode(_take(value));
  T _withBytes<T>(
    Uint8List value,
    T Function(ffi.Pointer<ffi.Uint8>, int) callback,
  ) {
    final pointer = calloc<ffi.Uint8>(value.length);
    if (value.isNotEmpty) pointer.asTypedList(value.length).setAll(0, value);
    try {
      return callback(pointer, value.length);
    } finally {
      pointer.asTypedList(value.length).fillRange(0, value.length, 0);
      calloc.free(pointer);
    }
  }

  T _withText<T>(
    String value,
    T Function(ffi.Pointer<ffi.Uint8>, int) callback,
  ) => _withBytes(Uint8List.fromList(utf8.encode(value)), callback);

  T? _withSecret<T>(
    bool Function(ffi.Pointer<ffi.Pointer<ffi.Void>>) getSecret,
    T Function(Uint8List) callback,
  ) {
    final output = calloc<ffi.Pointer<ffi.Void>>();
    try {
      _requireBool(getSecret(output));
      final handle = output.value;
      if (handle == ffi.nullptr) return null;
      final length = calloc<ffi.Size>();
      try {
        _requireBool(native.secret_len(handle, length));
        final bytes = calloc<ffi.Uint8>(length.value);
        try {
          _requireBool(native.secret_copy(handle, bytes, length.value));
          final value = Uint8List.fromList(bytes.asTypedList(length.value));
          try {
            return callback(value);
          } finally {
            value.fillRange(0, value.length, 0);
            bytes.asTypedList(length.value).fillRange(0, length.value, 0);
          }
        } finally {
          calloc.free(bytes);
        }
      } finally {
        calloc.free(length);
        native.secret_free(handle);
      }
    } finally {
      calloc.free(output);
    }
  }

  String lastErrorMessage() => lastError;

  ErrorDetails bufferLastErrorDetails() =>
      DomainDecoders.errorDetails(_take(native.buffer_last_error_details()));

  int lockboxFormatVersion() => native.lockbox_format_version();

  int lockboxProbeFormatVersion(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) =>
        native.lockbox_probe_format_version(bytesPointer, bytesLength),
  );

  ffi.Pointer<ffi.Void> lockboxCreate(Uint8List key) => _withBytes(
    key,
    (keyPointer, keyLength) =>
        _requireHandle(native.lockbox_create(keyPointer, keyLength)),
  );

  ffi.Pointer<ffi.Void> lockboxCreateWithOptions(
    Uint8List key,
    String cacheMode,
    int cacheBytes,
    String workload,
    String worker,
    int jobs,
  ) => _withBytes(
    key,
    (keyPointer, keyLength) => _withText(
      cacheMode,
      (cacheModePointer, cacheModeLength) => _withText(
        workload,
        (workloadPointer, workloadLength) => _withText(
          worker,
          (workerPointer, workerLength) => _requireHandle(
            native.lockbox_create_with_options(
              keyPointer,
              keyLength,
              cacheModePointer,
              cacheModeLength,
              cacheBytes,
              workloadPointer,
              workloadLength,
              workerPointer,
              workerLength,
              jobs,
            ),
          ),
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> lockboxCreatePassword(Uint8List password) => _withBytes(
    password,
    (passwordPointer, passwordLength) => _requireHandle(
      native.lockbox_create_password(passwordPointer, passwordLength),
    ),
  );

  ffi.Pointer<ffi.Void> lockboxCreateContact(ffi.Pointer<ffi.Void> contact) =>
      _requireHandle(native.lockbox_create_contact(contact));

  ffi.Pointer<ffi.Void> lockboxCreateWithSigningKey(
    Uint8List contentKey,
    ffi.Pointer<ffi.Void> signingKey,
  ) => _withBytes(
    contentKey,
    (contentKeyPointer, contentKeyLength) => _requireHandle(
      native.lockbox_create_with_signing_key(
        contentKeyPointer,
        contentKeyLength,
        signingKey,
      ),
    ),
  );

  ffi.Pointer<ffi.Void> lockboxOpen(Uint8List archive, Uint8List key) =>
      _withBytes(
        archive,
        (archivePointer, archiveLength) => _withBytes(
          key,
          (keyPointer, keyLength) => _requireHandle(
            native.lockbox_open(
              archivePointer,
              archiveLength,
              keyPointer,
              keyLength,
            ),
          ),
        ),
      );

  ffi.Pointer<ffi.Void> lockboxOpenWithOptions(
    Uint8List archive,
    Uint8List key,
    String cacheMode,
    int cacheBytes,
    String workload,
    String worker,
    int jobs,
  ) => _withBytes(
    archive,
    (archivePointer, archiveLength) => _withBytes(
      key,
      (keyPointer, keyLength) => _withText(
        cacheMode,
        (cacheModePointer, cacheModeLength) => _withText(
          workload,
          (workloadPointer, workloadLength) => _withText(
            worker,
            (workerPointer, workerLength) => _requireHandle(
              native.lockbox_open_with_options(
                archivePointer,
                archiveLength,
                keyPointer,
                keyLength,
                cacheModePointer,
                cacheModeLength,
                cacheBytes,
                workloadPointer,
                workloadLength,
                workerPointer,
                workerLength,
                jobs,
              ),
            ),
          ),
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> lockboxOpenPassword(
    Uint8List archive,
    Uint8List password,
  ) => _withBytes(
    archive,
    (archivePointer, archiveLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireHandle(
        native.lockbox_open_password(
          archivePointer,
          archiveLength,
          passwordPointer,
          passwordLength,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> lockboxOpenContact(
    Uint8List archive,
    ffi.Pointer<ffi.Void> contact,
  ) => _withBytes(
    archive,
    (archivePointer, archiveLength) => _requireHandle(
      native.lockbox_open_contact(archivePointer, archiveLength, contact),
    ),
  );

  bool lockboxAddFile(
    ffi.Pointer<ffi.Void> handle,
    String path,
    Uint8List data,
    bool replace,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      data,
      (dataPointer, dataLength) => _requireBool(
        native.lockbox_add_file(
          handle,
          pathPointer,
          pathLength,
          dataPointer,
          dataLength,
          replace,
        ),
      ),
    ),
  );

  bool lockboxAddFileWithPermissions(
    ffi.Pointer<ffi.Void> handle,
    String path,
    Uint8List data,
    int permissions,
    bool replace,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      data,
      (dataPointer, dataLength) => _requireBool(
        native.lockbox_add_file_with_permissions(
          handle,
          pathPointer,
          pathLength,
          dataPointer,
          dataLength,
          permissions,
          replace,
        ),
      ),
    ),
  );

  Uint8List lockboxGetFile(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) =>
            _take(native.lockbox_get_file(handle, pathPointer, pathLength)),
      );

  bool lockboxExtractFile(
    ffi.Pointer<ffi.Void> handle,
    String source,
    String destination,
    bool replace,
  ) => _withText(
    source,
    (sourcePointer, sourceLength) => _withText(
      destination,
      (destinationPointer, destinationLength) => _requireBool(
        native.lockbox_extract_file(
          handle,
          sourcePointer,
          sourceLength,
          destinationPointer,
          destinationLength,
          replace,
        ),
      ),
    ),
  );

  bool lockboxExtractDirectory(
    ffi.Pointer<ffi.Void> handle,
    String destination,
    int maxFileBytes,
    int maxTotalBytes,
    int maxFiles,
    bool restoreSymlinks,
    bool restorePermissions,
    bool overwrite,
  ) => _withText(
    destination,
    (destinationPointer, destinationLength) => _requireBool(
      native.lockbox_extract_directory(
        handle,
        destinationPointer,
        destinationLength,
        maxFileBytes,
        maxTotalBytes,
        maxFiles,
        restoreSymlinks,
        restorePermissions,
        overwrite,
      ),
    ),
  );

  List<StreamChunk> lockboxStreamContent(
    ffi.Pointer<ffi.Void> handle,
    bool physical,
  ) => DomainDecoders.streamChunkList(
    _take(native.lockbox_stream_content(handle, physical)),
  );

  CacheStats lockboxCacheStats(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.cacheStats(_take(native.lockbox_cache_stats(handle)));

  ImportStats lockboxImportStats(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.importStats(_take(native.lockbox_import_stats(handle)));

  bool lockboxResetImportStats(ffi.Pointer<ffi.Void> handle) =>
      _requireBool(native.lockbox_reset_import_stats(handle));

  FileInspection lockboxInspectFile(String path) => _withText(
    path,
    (pathPointer, pathLength) => DomainDecoders.fileInspection(
      _take(native.lockbox_inspect_file(pathPointer, pathLength)),
    ),
  );

  List<PageInspection> lockboxPageInspection(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.pageInspectionList(
        _take(native.lockbox_page_inspection(handle)),
      );

  RecoveryReport lockboxRecoveryReport(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.recoveryReport(
        _take(native.lockbox_recovery_report(handle)),
      );

  String lockboxRecoveryReportRender(
    ffi.Pointer<ffi.Void> handle,
    bool verbose,
    int maxEntries,
  ) => _takeString(
    native.lockbox_recovery_report_render(handle, verbose, maxEntries),
  );

  RecoveryReport lockboxRecoveryScanPath(String path, Uint8List key) =>
      _withText(
        path,
        (pathPointer, pathLength) => _withBytes(
          key,
          (keyPointer, keyLength) => DomainDecoders.recoveryReport(
            _take(
              native.lockbox_recovery_scan_path(
                pathPointer,
                pathLength,
                keyPointer,
                keyLength,
              ),
            ),
          ),
        ),
      );

  int lockboxStorageLen(ffi.Pointer<ffi.Void> handle) =>
      native.lockbox_storage_len(handle);

  bool lockboxSetWorkloadProfile(
    ffi.Pointer<ffi.Void> handle,
    String profile,
  ) => _withText(
    profile,
    (profilePointer, profileLength) => _requireBool(
      native.lockbox_set_workload_profile(
        handle,
        profilePointer,
        profileLength,
      ),
    ),
  );

  bool lockboxSetWorkerPolicy(
    ffi.Pointer<ffi.Void> handle,
    String mode,
    int jobs,
  ) => _withText(
    mode,
    (modePointer, modeLength) => _requireBool(
      native.lockbox_set_worker_policy(handle, modePointer, modeLength, jobs),
    ),
  );

  RuntimeOptions lockboxRuntimeOptions(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.runtimeOptions(
        _take(native.lockbox_runtime_options(handle)),
      );

  bool lockboxCommit(ffi.Pointer<ffi.Void> handle) =>
      _requireBool(native.lockbox_commit(handle));

  bool lockboxCreateDir(
    ffi.Pointer<ffi.Void> handle,
    String path,
    bool createParents,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _requireBool(
      native.lockbox_create_dir(handle, pathPointer, pathLength, createParents),
    ),
  );

  bool lockboxDelete(ffi.Pointer<ffi.Void> handle, String path) => _withText(
    path,
    (pathPointer, pathLength) =>
        _requireBool(native.lockbox_delete(handle, pathPointer, pathLength)),
  );

  bool lockboxRemoveDir(
    ffi.Pointer<ffi.Void> handle,
    String path,
    bool recursive,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _requireBool(
      native.lockbox_remove_dir(handle, pathPointer, pathLength, recursive),
    ),
  );

  bool lockboxCreateParentDirs(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) => _requireBool(
          native.lockbox_create_parent_dirs(handle, pathPointer, pathLength),
        ),
      );

  bool lockboxRename(ffi.Pointer<ffi.Void> handle, String from, String to) =>
      _withText(
        from,
        (fromPointer, fromLength) => _withText(
          to,
          (toPointer, toLength) => _requireBool(
            native.lockbox_rename(
              handle,
              fromPointer,
              fromLength,
              toPointer,
              toLength,
            ),
          ),
        ),
      );

  List<LockboxEntry> lockboxList(
    ffi.Pointer<ffi.Void> handle,
    String path,
    bool recursive,
  ) => _withText(
    path,
    (pathPointer, pathLength) => DomainDecoders.lockboxEntryList(
      _take(native.lockbox_list(handle, pathPointer, pathLength, recursive)),
    ),
  );

  List<LockboxEntry> lockboxListWithOptions(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String glob,
    bool recursive,
    bool includeFiles,
    bool includeSymlinks,
    bool includeDirectories,
    int limit,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      glob,
      (globPointer, globLength) => DomainDecoders.lockboxEntryList(
        _take(
          native.lockbox_list_with_options(
            handle,
            pathPointer,
            pathLength,
            globPointer,
            globLength,
            recursive,
            includeFiles,
            includeSymlinks,
            includeDirectories,
            limit,
          ),
        ),
      ),
    ),
  );

  LockboxEntry? lockboxStat(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) => DomainDecoders.optionalLockboxEntry(
          _take(native.lockbox_stat(handle, pathPointer, pathLength)),
        ),
      );

  bool lockboxSetVariable(
    ffi.Pointer<ffi.Void> handle,
    String name,
    String value,
  ) => _withText(
    name,
    (namePointer, nameLength) => _withText(
      value,
      (valuePointer, valueLength) => _requireBool(
        native.lockbox_set_variable(
          handle,
          namePointer,
          nameLength,
          valuePointer,
          valueLength,
        ),
      ),
    ),
  );

  bool lockboxSetSecretVariable(
    ffi.Pointer<ffi.Void> handle,
    String name,
    Uint8List value,
  ) => _withText(
    name,
    (namePointer, nameLength) => _withBytes(
      value,
      (valuePointer, valueLength) => _requireBool(
        native.lockbox_set_secret_variable(
          handle,
          namePointer,
          nameLength,
          valuePointer,
          valueLength,
        ),
      ),
    ),
  );

  String? lockboxGetVariable(ffi.Pointer<ffi.Void> handle, String name) =>
      _withText(
        name,
        (namePointer, nameLength) => DomainDecoders.optionalString(
          _take(native.lockbox_get_variable(handle, namePointer, nameLength)),
        ),
      );

  T? lockboxWithSecretVariable<T>(
    ffi.Pointer<ffi.Void> handle,
    String name,
    T Function(Uint8List) callback,
  ) => _withText(
    name,
    (namePointer, nameLength) => _withSecret(
      (output) => native.lockbox_get_secret_variable(
        handle,
        namePointer,
        nameLength,
        output,
      ),
      callback,
    ),
  );

  bool lockboxDeleteVariable(ffi.Pointer<ffi.Void> handle, String name) =>
      _withText(
        name,
        (namePointer, nameLength) => _requireBool(
          native.lockbox_delete_variable(handle, namePointer, nameLength),
        ),
      );

  bool lockboxMoveVariables(
    ffi.Pointer<ffi.Void> handle,
    Uint8List movesFlatbuffer,
  ) => _withBytes(
    movesFlatbuffer,
    (movesFlatbufferPointer, movesFlatbufferLength) => _requireBool(
      native.lockbox_move_variables(
        handle,
        movesFlatbufferPointer,
        movesFlatbufferLength,
      ),
    ),
  );

  List<Variable> lockboxListVariables(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.variableList(_take(native.lockbox_list_variables(handle)));

  String? lockboxVariableSensitivity(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => DomainDecoders.optionalString(
      _take(
        native.lockbox_variable_sensitivity(handle, namePointer, nameLength),
      ),
    ),
  );

  bool lockboxAddSymlink(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String target,
    bool replace,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      target,
      (targetPointer, targetLength) => _requireBool(
        native.lockbox_add_symlink(
          handle,
          pathPointer,
          pathLength,
          targetPointer,
          targetLength,
          replace,
        ),
      ),
    ),
  );

  String lockboxGetSymlinkTarget(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) => _takeString(
          native.lockbox_get_symlink_target(handle, pathPointer, pathLength),
        ),
      );

  Uint8List lockboxId(ffi.Pointer<ffi.Void> handle) =>
      _take(native.lockbox_id(handle));

  bool lockboxExists(ffi.Pointer<ffi.Void> handle, String path) => _withText(
    path,
    (pathPointer, pathLength) =>
        native.lockbox_exists(handle, pathPointer, pathLength),
  );

  bool lockboxIsDir(ffi.Pointer<ffi.Void> handle, String path) => _withText(
    path,
    (pathPointer, pathLength) =>
        native.lockbox_is_dir(handle, pathPointer, pathLength),
  );

  int lockboxPermissions(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) =>
            native.lockbox_permissions(handle, pathPointer, pathLength),
      );

  bool lockboxSetPermissions(
    ffi.Pointer<ffi.Void> handle,
    String path,
    int permissions,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _requireBool(
      native.lockbox_set_permissions(
        handle,
        pathPointer,
        pathLength,
        permissions,
      ),
    ),
  );

  Uint8List lockboxReadRange(
    ffi.Pointer<ffi.Void> handle,
    String path,
    int offset,
    int len,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _take(
      native.lockbox_read_range(handle, pathPointer, pathLength, offset, len),
    ),
  );

  RecoveryReport lockboxRecoveryScan(Uint8List bytes, Uint8List key) =>
      _withBytes(
        bytes,
        (bytesPointer, bytesLength) => _withBytes(
          key,
          (keyPointer, keyLength) => DomainDecoders.recoveryReport(
            _take(
              native.lockbox_recovery_scan(
                bytesPointer,
                bytesLength,
                keyPointer,
                keyLength,
              ),
            ),
          ),
        ),
      );

  ffi.Pointer<ffi.Void> lockboxRecoverySalvage(
    Uint8List bytes,
    Uint8List key,
    ffi.Pointer<ffi.Void> signingKey,
  ) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) => _withBytes(
      key,
      (keyPointer, keyLength) => _requireHandle(
        native.lockbox_recovery_salvage(
          bytesPointer,
          bytesLength,
          keyPointer,
          keyLength,
          signingKey,
        ),
      ),
    ),
  );

  int lockboxAddPassword(ffi.Pointer<ffi.Void> handle, Uint8List password) =>
      _withBytes(
        password,
        (passwordPointer, passwordLength) => native.lockbox_add_password(
          handle,
          passwordPointer,
          passwordLength,
        ),
      );

  int lockboxAddContact(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Void> contact,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) =>
        native.lockbox_add_contact(handle, contact, namePointer, nameLength),
  );

  bool lockboxDeleteKey(ffi.Pointer<ffi.Void> handle, int id) =>
      _requireBool(native.lockbox_delete_key(handle, id));

  List<KeySlot> lockboxListKeySlots(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.keySlotList(_take(native.lockbox_list_key_slots(handle)));

  bool lockboxSetOwnerSigningKey(
    ffi.Pointer<ffi.Void> handle,
    ffi.Pointer<ffi.Void> key,
  ) => _requireBool(native.lockbox_set_owner_signing_key(handle, key));

  OwnerInspection lockboxOwnerInspection(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.ownerInspection(
        _take(native.lockbox_owner_inspection(handle)),
      );

  FormDefinition lockboxDefineForm(
    ffi.Pointer<ffi.Void> handle,
    String alias,
    String name,
    String description,
    Uint8List fieldsFlatbuffer,
  ) => _withText(
    alias,
    (aliasPointer, aliasLength) => _withText(
      name,
      (namePointer, nameLength) => _withText(
        description,
        (descriptionPointer, descriptionLength) => _withBytes(
          fieldsFlatbuffer,
          (fieldsFlatbufferPointer, fieldsFlatbufferLength) =>
              DomainDecoders.formDefinition(
                _take(
                  native.lockbox_define_form(
                    handle,
                    aliasPointer,
                    aliasLength,
                    namePointer,
                    nameLength,
                    descriptionPointer,
                    descriptionLength,
                    fieldsFlatbufferPointer,
                    fieldsFlatbufferLength,
                  ),
                ),
              ),
        ),
      ),
    ),
  );

  List<FormDefinition> lockboxListFormDefinitions(
    ffi.Pointer<ffi.Void> handle,
  ) => DomainDecoders.formDefinitionList(
    _take(native.lockbox_list_form_definitions(handle)),
  );

  FormDefinition lockboxResolveForm(
    ffi.Pointer<ffi.Void> handle,
    String reference,
  ) => _withText(
    reference,
    (referencePointer, referenceLength) => DomainDecoders.formDefinition(
      _take(
        native.lockbox_resolve_form(handle, referencePointer, referenceLength),
      ),
    ),
  );

  List<FormDefinition> lockboxListFormRevisions(
    ffi.Pointer<ffi.Void> handle,
    String typeId,
  ) => _withText(
    typeId,
    (typeIdPointer, typeIdLength) => DomainDecoders.formDefinitionList(
      _take(
        native.lockbox_list_form_revisions(handle, typeIdPointer, typeIdLength),
      ),
    ),
  );

  FormRecord lockboxCreateFormRecord(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String typeReference,
    String name,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      typeReference,
      (typeReferencePointer, typeReferenceLength) => _withText(
        name,
        (namePointer, nameLength) => DomainDecoders.formRecord(
          _take(
            native.lockbox_create_form_record(
              handle,
              pathPointer,
              pathLength,
              typeReferencePointer,
              typeReferenceLength,
              namePointer,
              nameLength,
            ),
          ),
        ),
      ),
    ),
  );

  bool lockboxSetFormField(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String field,
    String value,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      field,
      (fieldPointer, fieldLength) => _withText(
        value,
        (valuePointer, valueLength) => _requireBool(
          native.lockbox_set_form_field(
            handle,
            pathPointer,
            pathLength,
            fieldPointer,
            fieldLength,
            valuePointer,
            valueLength,
          ),
        ),
      ),
    ),
  );

  bool lockboxSetSecretFormField(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String field,
    Uint8List value,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      field,
      (fieldPointer, fieldLength) => _withBytes(
        value,
        (valuePointer, valueLength) => _requireBool(
          native.lockbox_set_secret_form_field(
            handle,
            pathPointer,
            pathLength,
            fieldPointer,
            fieldLength,
            valuePointer,
            valueLength,
          ),
        ),
      ),
    ),
  );

  List<FormRecord> lockboxListFormRecords(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.formRecordList(
        _take(native.lockbox_list_form_records(handle)),
      );

  FormRecord? lockboxGetFormRecord(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) => DomainDecoders.optionalFormRecord(
          _take(
            native.lockbox_get_form_record(handle, pathPointer, pathLength),
          ),
        ),
      );

  bool lockboxDeleteFormRecord(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) => _requireBool(
          native.lockbox_delete_form_record(handle, pathPointer, pathLength),
        ),
      );

  bool lockboxMoveFormRecords(
    ffi.Pointer<ffi.Void> handle,
    Uint8List movesFlatbuffer,
  ) => _withBytes(
    movesFlatbuffer,
    (movesFlatbufferPointer, movesFlatbufferLength) => _requireBool(
      native.lockbox_move_form_records(
        handle,
        movesFlatbufferPointer,
        movesFlatbufferLength,
      ),
    ),
  );

  FormValue? lockboxGetFormField(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String field,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      field,
      (fieldPointer, fieldLength) => DomainDecoders.optionalFormValue(
        _take(
          native.lockbox_get_form_field(
            handle,
            pathPointer,
            pathLength,
            fieldPointer,
            fieldLength,
          ),
        ),
      ),
    ),
  );

  T? lockboxWithSecretFormField<T>(
    ffi.Pointer<ffi.Void> handle,
    String path,
    String field,
    T Function(Uint8List) callback,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      field,
      (fieldPointer, fieldLength) => _withSecret(
        (output) => native.lockbox_get_secret_form_field(
          handle,
          pathPointer,
          pathLength,
          fieldPointer,
          fieldLength,
          output,
        ),
        callback,
      ),
    ),
  );

  Uint8List lockboxToBytes(ffi.Pointer<ffi.Void> handle) =>
      _take(native.lockbox_to_bytes(handle));

  void lockboxFree(ffi.Pointer<ffi.Void> handle) => native.lockbox_free(handle);

  bool vaultIsRunning() => native.vault_is_running();

  bool vaultForgetAll() => _requireBool(native.vault_forget_all());

  ffi.Pointer<ffi.Void> keyContactGenerate() =>
      _requireHandle(native.key_contact_generate());

  ffi.Pointer<ffi.Void> keyContactFromPrivate(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) => _requireHandle(
      native.key_contact_from_private(bytesPointer, bytesLength),
    ),
  );

  Uint8List keyContactPublic(ffi.Pointer<ffi.Void> handle) =>
      _take(native.key_contact_public(handle));

  Uint8List keyContactPrivate(ffi.Pointer<ffi.Void> handle) =>
      _take(native.key_contact_private(handle));

  ffi.Pointer<ffi.Void> keyContactPublicFromBytes(Uint8List bytes) =>
      _withBytes(
        bytes,
        (bytesPointer, bytesLength) => _requireHandle(
          native.key_contact_public_from_bytes(bytesPointer, bytesLength),
        ),
      );

  void keyContactPublicFree(ffi.Pointer<ffi.Void> handle) =>
      native.key_contact_public_free(handle);

  void keyContactFree(ffi.Pointer<ffi.Void> handle) =>
      native.key_contact_free(handle);

  ffi.Pointer<ffi.Void> keyContactEncrypt(
    ffi.Pointer<ffi.Void> contact,
    Uint8List contentKey,
  ) => _withBytes(
    contentKey,
    (contentKeyPointer, contentKeyLength) => _requireHandle(
      native.key_contact_encrypt(contact, contentKeyPointer, contentKeyLength),
    ),
  );

  Uint8List keyContactDecrypt(
    ffi.Pointer<ffi.Void> contact,
    ffi.Pointer<ffi.Void> wrapped,
  ) => _take(native.key_contact_decrypt(contact, wrapped));

  Uint8List keyContactWrappedPublic(ffi.Pointer<ffi.Void> wrapped) =>
      _take(native.key_contact_wrapped_public(wrapped));

  Uint8List keyContactWrappedCiphertext(ffi.Pointer<ffi.Void> wrapped) =>
      _take(native.key_contact_wrapped_ciphertext(wrapped));

  Uint8List keyContactWrappedEncrypted(ffi.Pointer<ffi.Void> wrapped) =>
      _take(native.key_contact_wrapped_encrypted(wrapped));

  void keyContactWrappedFree(ffi.Pointer<ffi.Void> handle) =>
      native.key_contact_wrapped_free(handle);

  ffi.Pointer<ffi.Void> keySigningGenerate() =>
      _requireHandle(native.key_signing_generate());

  ffi.Pointer<ffi.Void> keySigningFromPrivate(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) => _requireHandle(
      native.key_signing_from_private(bytesPointer, bytesLength),
    ),
  );

  Uint8List keySigningPublic(ffi.Pointer<ffi.Void> handle) =>
      _take(native.key_signing_public(handle));

  Uint8List keySigningPrivate(ffi.Pointer<ffi.Void> handle) =>
      _take(native.key_signing_private(handle));

  ffi.Pointer<ffi.Void> keySigningPublicFromBytes(Uint8List bytes) =>
      _withBytes(
        bytes,
        (bytesPointer, bytesLength) => _requireHandle(
          native.key_signing_public_from_bytes(bytesPointer, bytesLength),
        ),
      );

  void keySigningPublicFree(ffi.Pointer<ffi.Void> handle) =>
      native.key_signing_public_free(handle);

  void keySigningFree(ffi.Pointer<ffi.Void> handle) =>
      native.key_signing_free(handle);

  Uint8List vaultKeyExportPrivate(ffi.Pointer<ffi.Void> key, String format) =>
      _withText(
        format,
        (formatPointer, formatLength) => _take(
          native.vault_key_export_private(key, formatPointer, formatLength),
        ),
      );

  Uint8List vaultKeyExportPublic(ffi.Pointer<ffi.Void> key, String format) =>
      _withText(
        format,
        (formatPointer, formatLength) => _take(
          native.vault_key_export_public(key, formatPointer, formatLength),
        ),
      );

  ffi.Pointer<ffi.Void> vaultKeyImportPrivate(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) => _requireHandle(
      native.vault_key_import_private(bytesPointer, bytesLength),
    ),
  );

  ffi.Pointer<ffi.Void> vaultKeyImportPublic(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) => _requireHandle(
      native.vault_key_import_public(bytesPointer, bytesLength),
    ),
  );

  Uint8List vaultKeyFingerprint(ffi.Pointer<ffi.Void> key) =>
      _take(native.vault_key_fingerprint(key));

  String vaultKeyFormatHex(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) =>
        _takeString(native.vault_key_format_hex(bytesPointer, bytesLength)),
  );

  Uint8List vaultKeyDecodeHex(String text) => _withText(
    text,
    (textPointer, textLength) =>
        _take(native.vault_key_decode_hex(textPointer, textLength)),
  );

  String vaultKeyFormatCrockford(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) => _takeString(
      native.vault_key_format_crockford(bytesPointer, bytesLength),
    ),
  );

  String vaultKeyFormatCrockfordReading(String code) => _withText(
    code,
    (codePointer, codeLength) => _takeString(
      native.vault_key_format_crockford_reading(codePointer, codeLength),
    ),
  );

  Uint8List vaultKeyDecodeCrockford(String code) => _withText(
    code,
    (codePointer, codeLength) =>
        _take(native.vault_key_decode_crockford(codePointer, codeLength)),
  );

  String vaultKeyHexEncode(Uint8List bytes) => _withBytes(
    bytes,
    (bytesPointer, bytesLength) =>
        _takeString(native.vault_key_hex_encode(bytesPointer, bytesLength)),
  );

  Uint8List vaultKeyHexDecode(String text) => _withText(
    text,
    (textPointer, textLength) =>
        _take(native.vault_key_hex_decode(textPointer, textLength)),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryOpen(String root, Uint8List password) =>
      _withText(
        root,
        (rootPointer, rootLength) => _withBytes(
          password,
          (passwordPointer, passwordLength) => _requireHandle(
            native.vault_directory_open(
              rootPointer,
              rootLength,
              passwordPointer,
              passwordLength,
            ),
          ),
        ),
      );

  int vaultStructureVersionCurrent() =>
      native.vault_structure_version_current();

  int vaultDirectoryProbeStructureVersion(String root, Uint8List password) =>
      _withText(
        root,
        (rootPointer, rootLength) => _withBytes(
          password,
          (passwordPointer, passwordLength) =>
              native.vault_directory_probe_structure_version(
                rootPointer,
                rootLength,
                passwordPointer,
                passwordLength,
              ),
        ),
      );

  ffi.Pointer<ffi.Void> vaultDirectoryOpenOrCreateDefault(Uint8List password) =>
      _withBytes(
        password,
        (passwordPointer, passwordLength) => _requireHandle(
          native.vault_directory_open_or_create_default(
            passwordPointer,
            passwordLength,
          ),
        ),
      );

  ffi.Pointer<ffi.Void> vaultDirectoryReplaceDefault(Uint8List password) =>
      _withBytes(
        password,
        (passwordPointer, passwordLength) => _requireHandle(
          native.vault_directory_replace_default(
            passwordPointer,
            passwordLength,
          ),
        ),
      );

  bool vaultDirectoryChangePassword(
    String root,
    Uint8List oldPassword,
    Uint8List newPassword,
  ) => _withText(
    root,
    (rootPointer, rootLength) => _withBytes(
      oldPassword,
      (oldPasswordPointer, oldPasswordLength) => _withBytes(
        newPassword,
        (newPasswordPointer, newPasswordLength) => _requireBool(
          native.vault_directory_change_password(
            rootPointer,
            rootLength,
            oldPasswordPointer,
            oldPasswordLength,
            newPasswordPointer,
            newPasswordLength,
          ),
        ),
      ),
    ),
  );

  bool vaultDirectoryChangeDefaultPassword(
    Uint8List oldPassword,
    Uint8List newPassword,
  ) => _withBytes(
    oldPassword,
    (oldPasswordPointer, oldPasswordLength) => _withBytes(
      newPassword,
      (newPasswordPointer, newPasswordLength) => _requireBool(
        native.vault_directory_change_default_password(
          oldPasswordPointer,
          oldPasswordLength,
          newPasswordPointer,
          newPasswordLength,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryReplace(
    String root,
    Uint8List password,
  ) => _withText(
    root,
    (rootPointer, rootLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireHandle(
        native.vault_directory_replace(
          rootPointer,
          rootLength,
          passwordPointer,
          passwordLength,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryOpenOrCreate(
    String root,
    Uint8List password,
  ) => _withText(
    root,
    (rootPointer, rootLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireHandle(
        native.vault_directory_open_or_create(
          rootPointer,
          rootLength,
          passwordPointer,
          passwordLength,
        ),
      ),
    ),
  );

  String vaultDirectoryRoot(ffi.Pointer<ffi.Void> handle) =>
      _takeString(native.vault_directory_root(handle));

  int vaultDirectoryStructureVersion(ffi.Pointer<ffi.Void> handle) =>
      native.vault_directory_structure_version(handle);

  List<String> vaultDirectoryListPrivateKeys(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.stringList(
        _take(native.vault_directory_list_private_keys(handle)),
      );

  List<String> vaultDirectoryListPrivateKeyNames(
    ffi.Pointer<ffi.Void> handle,
  ) => DomainDecoders.stringList(
    _take(native.vault_directory_list_private_key_names(handle)),
  );

  List<String> vaultDirectoryListContactNames(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.stringList(
        _take(native.vault_directory_list_contact_names(handle)),
      );

  List<String> vaultDirectoryListFormAliases(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.stringList(
        _take(native.vault_directory_list_form_aliases(handle)),
      );

  bool vaultDirectoryPrivateKeyExists(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => native.vault_directory_private_key_exists(
      handle,
      namePointer,
      nameLength,
    ),
  );

  bool vaultDirectoryDeletePrivateKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireBool(
      native.vault_directory_delete_private_key(
        handle,
        namePointer,
        nameLength,
      ),
    ),
  );

  bool vaultDirectoryStorePrivateKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
    ffi.Pointer<ffi.Void> key,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireBool(
      native.vault_directory_store_private_key(
        handle,
        namePointer,
        nameLength,
        key,
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryLoadPrivateKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireHandle(
      native.vault_directory_load_private_key(handle, namePointer, nameLength),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryLoadPrivateKeyGeneration(
    ffi.Pointer<ffi.Void> handle,
    String name,
    int index,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireHandle(
      native.vault_directory_load_private_key_generation(
        handle,
        namePointer,
        nameLength,
        index,
      ),
    ),
  );

  bool vaultDirectoryStoreContact(
    ffi.Pointer<ffi.Void> handle,
    String name,
    ffi.Pointer<ffi.Void> key,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireBool(
      native.vault_directory_store_contact(
        handle,
        namePointer,
        nameLength,
        key,
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryLoadContact(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireHandle(
      native.vault_directory_load_contact(handle, namePointer, nameLength),
    ),
  );

  bool vaultDirectoryContactExists(ffi.Pointer<ffi.Void> handle, String name) =>
      _withText(
        name,
        (namePointer, nameLength) => native.vault_directory_contact_exists(
          handle,
          namePointer,
          nameLength,
        ),
      );

  bool vaultDirectoryDeleteContact(ffi.Pointer<ffi.Void> handle, String name) =>
      _withText(
        name,
        (namePointer, nameLength) => _requireBool(
          native.vault_directory_delete_contact(
            handle,
            namePointer,
            nameLength,
          ),
        ),
      );

  List<Contact> vaultDirectoryListContacts(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.contactList(
        _take(native.vault_directory_list_contacts(handle)),
      );

  bool vaultDirectoryStoreProfileEmail(
    ffi.Pointer<ffi.Void> handle,
    String name,
    String email,
  ) => _withText(
    name,
    (namePointer, nameLength) => _withText(
      email,
      (emailPointer, emailLength) => _requireBool(
        native.vault_directory_store_profile_email(
          handle,
          namePointer,
          nameLength,
          emailPointer,
          emailLength,
        ),
      ),
    ),
  );

  String? vaultDirectoryProfileEmail(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => DomainDecoders.optionalString(
      _take(
        native.vault_directory_profile_email(handle, namePointer, nameLength),
      ),
    ),
  );

  bool vaultDirectoryStoreBackup(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
    Uint8List bytes,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _withBytes(
      bytes,
      (bytesPointer, bytesLength) => _requireBool(
        native.vault_directory_store_backup(
          handle,
          idPointer,
          idLength,
          bytesPointer,
          bytesLength,
        ),
      ),
    ),
  );

  Uint8List vaultDirectoryLoadBackup(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
  ) => _withBytes(
    id,
    (idPointer, idLength) =>
        _take(native.vault_directory_load_backup(handle, idPointer, idLength)),
  );

  int vaultDirectoryBackupCount(ffi.Pointer<ffi.Void> handle) =>
      native.vault_directory_backup_count(handle);

  bool vaultDirectoryRestorePrivateKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
    ffi.Pointer<ffi.Void> key,
    ffi.Pointer<ffi.Void> signingKey,
    bool overwrite,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireBool(
      native.vault_directory_restore_private_key(
        handle,
        namePointer,
        nameLength,
        key,
        signingKey,
        overwrite,
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryLoadOwnerSigningKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireHandle(
      native.vault_directory_load_owner_signing_key(
        handle,
        namePointer,
        nameLength,
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryLoadOwnerSigningKeyGeneration(
    ffi.Pointer<ffi.Void> handle,
    String name,
    int index,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireHandle(
      native.vault_directory_load_owner_signing_key_generation(
        handle,
        namePointer,
        nameLength,
        index,
      ),
    ),
  );

  bool vaultDirectoryStoreContactSigningKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
    ffi.Pointer<ffi.Void> key,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireBool(
      native.vault_directory_store_contact_signing_key(
        handle,
        namePointer,
        nameLength,
        key,
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultDirectoryLoadContactSigningKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => _requireHandle(
      native.vault_directory_load_contact_signing_key(
        handle,
        namePointer,
        nameLength,
      ),
    ),
  );

  ProfileHistory vaultDirectoryListProfileGenerations(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => DomainDecoders.profileHistory(
      _take(
        native.vault_directory_list_profile_generations(
          handle,
          namePointer,
          nameLength,
        ),
      ),
    ),
  );

  ProfileHistory vaultDirectoryRotatePrivateKey(
    ffi.Pointer<ffi.Void> handle,
    String name,
  ) => _withText(
    name,
    (namePointer, nameLength) => DomainDecoders.profileHistory(
      _take(
        native.vault_directory_rotate_private_key(
          handle,
          namePointer,
          nameLength,
        ),
      ),
    ),
  );

  bool vaultDirectoryRememberLockbox(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
    String path,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _withText(
      path,
      (pathPointer, pathLength) => _requireBool(
        native.vault_directory_remember_lockbox(
          handle,
          idPointer,
          idLength,
          pathPointer,
          pathLength,
        ),
      ),
    ),
  );

  List<KnownLockbox> vaultDirectoryListKnownLockboxes(
    ffi.Pointer<ffi.Void> handle,
  ) => DomainDecoders.knownLockboxList(
    _take(native.vault_directory_list_known_lockboxes(handle)),
  );

  bool vaultDirectoryForgetLockbox(ffi.Pointer<ffi.Void> handle, String path) =>
      _withText(
        path,
        (pathPointer, pathLength) => _requireBool(
          native.vault_directory_forget_lockbox(
            handle,
            pathPointer,
            pathLength,
          ),
        ),
      );

  bool vaultDirectoryRememberAccessSlotLabel(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
    int slotId,
    String name,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _withText(
      name,
      (namePointer, nameLength) => _requireBool(
        native.vault_directory_remember_access_slot_label(
          handle,
          idPointer,
          idLength,
          slotId,
          namePointer,
          nameLength,
        ),
      ),
    ),
  );

  List<AccessSlotLabel> vaultDirectoryListAccessSlotLabels(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
  ) => _withBytes(
    id,
    (idPointer, idLength) => DomainDecoders.accessSlotLabelList(
      _take(
        native.vault_directory_list_access_slot_labels(
          handle,
          idPointer,
          idLength,
        ),
      ),
    ),
  );

  List<AccessSlotLabel> vaultDirectoryFindAccessSlotLabels(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
    String name,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _withText(
      name,
      (namePointer, nameLength) => DomainDecoders.accessSlotLabelList(
        _take(
          native.vault_directory_find_access_slot_labels(
            handle,
            idPointer,
            idLength,
            namePointer,
            nameLength,
          ),
        ),
      ),
    ),
  );

  bool vaultDirectoryForgetAccessSlotLabel(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
    int slotId,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _requireBool(
      native.vault_directory_forget_access_slot_label(
        handle,
        idPointer,
        idLength,
        slotId,
      ),
    ),
  );

  FormDefinition vaultDirectoryDefineForm(
    ffi.Pointer<ffi.Void> handle,
    String alias,
    String name,
    String description,
    Uint8List fieldsFlatbuffer,
  ) => _withText(
    alias,
    (aliasPointer, aliasLength) => _withText(
      name,
      (namePointer, nameLength) => _withText(
        description,
        (descriptionPointer, descriptionLength) => _withBytes(
          fieldsFlatbuffer,
          (fieldsFlatbufferPointer, fieldsFlatbufferLength) =>
              DomainDecoders.formDefinition(
                _take(
                  native.vault_directory_define_form(
                    handle,
                    aliasPointer,
                    aliasLength,
                    namePointer,
                    nameLength,
                    descriptionPointer,
                    descriptionLength,
                    fieldsFlatbufferPointer,
                    fieldsFlatbufferLength,
                  ),
                ),
              ),
        ),
      ),
    ),
  );

  FormDefinition vaultDirectoryResolveForm(
    ffi.Pointer<ffi.Void> handle,
    String reference,
  ) => _withText(
    reference,
    (referencePointer, referenceLength) => DomainDecoders.formDefinition(
      _take(
        native.vault_directory_resolve_form(
          handle,
          referencePointer,
          referenceLength,
        ),
      ),
    ),
  );

  List<FormDefinition> vaultDirectoryListForms(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.formDefinitionList(
        _take(native.vault_directory_list_forms(handle)),
      );

  List<FormDefinition> vaultDirectoryListFormRevisions(
    ffi.Pointer<ffi.Void> handle,
    String typeId,
  ) => _withText(
    typeId,
    (typeIdPointer, typeIdLength) => DomainDecoders.formDefinitionList(
      _take(
        native.vault_directory_list_form_revisions(
          handle,
          typeIdPointer,
          typeIdLength,
        ),
      ),
    ),
  );

  int vaultDirectorySeedForms(ffi.Pointer<ffi.Void> handle) =>
      native.vault_directory_seed_forms(handle);

  bool vaultDirectoryRememberPassword(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
    Uint8List password,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireBool(
        native.vault_directory_remember_password(
          handle,
          idPointer,
          idLength,
          passwordPointer,
          passwordLength,
        ),
      ),
    ),
  );

  Uint8List vaultDirectoryRememberedPassword(
    ffi.Pointer<ffi.Void> handle,
    Uint8List id,
  ) => _withBytes(
    id,
    (idPointer, idLength) => _take(
      native.vault_directory_remembered_password(handle, idPointer, idLength),
    ),
  );

  VaultBackupManifest vaultBackupDefault(String path, bool overwrite) =>
      _withText(
        path,
        (pathPointer, pathLength) => DomainDecoders.vaultBackupManifest(
          _take(
            native.vault_backup_default(pathPointer, pathLength, overwrite),
          ),
        ),
      );

  VaultBackupManifest vaultRestoreDefault(String path, bool overwrite) =>
      _withText(
        path,
        (pathPointer, pathLength) => DomainDecoders.vaultBackupManifest(
          _take(
            native.vault_restore_default(pathPointer, pathLength, overwrite),
          ),
        ),
      );

  void vaultDirectoryFree(ffi.Pointer<ffi.Void> handle) =>
      native.vault_directory_free(handle);

  ffi.Pointer<ffi.Void> vaultReadOnlyOpen(String root, Uint8List password) =>
      _withText(
        root,
        (rootPointer, rootLength) => _withBytes(
          password,
          (passwordPointer, passwordLength) => _requireHandle(
            native.vault_read_only_open(
              rootPointer,
              rootLength,
              passwordPointer,
              passwordLength,
            ),
          ),
        ),
      );

  ffi.Pointer<ffi.Void> vaultReadOnlyOpenDefault(Uint8List password) =>
      _withBytes(
        password,
        (passwordPointer, passwordLength) => _requireHandle(
          native.vault_read_only_open_default(passwordPointer, passwordLength),
        ),
      );

  List<String> vaultReadOnlyListProfileNames(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.stringList(
        _take(native.vault_read_only_list_profile_names(handle)),
      );

  List<String> vaultReadOnlyListContactNames(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.stringList(
        _take(native.vault_read_only_list_contact_names(handle)),
      );

  List<String> vaultReadOnlyListFormAliases(ffi.Pointer<ffi.Void> handle) =>
      DomainDecoders.stringList(
        _take(native.vault_read_only_list_form_aliases(handle)),
      );

  List<KnownLockbox> vaultReadOnlyListKnownLockboxes(
    ffi.Pointer<ffi.Void> handle,
  ) => DomainDecoders.knownLockboxList(
    _take(native.vault_read_only_list_known_lockboxes(handle)),
  );

  void vaultReadOnlyFree(ffi.Pointer<ffi.Void> handle) =>
      native.vault_read_only_free(handle);

  bool vaultAgentServe() => _requireBool(native.vault_agent_serve());

  bool vaultAgentVerifyTransport() =>
      _requireBool(native.vault_agent_verify_transport());

  Uint8List vaultAgentGet(Uint8List id) => _withBytes(
    id,
    (idPointer, idLength) => _take(native.vault_agent_get(idPointer, idLength)),
  );

  bool vaultAgentPut(Uint8List id, Uint8List key) => _withBytes(
    id,
    (idPointer, idLength) => _withBytes(
      key,
      (keyPointer, keyLength) => _requireBool(
        native.vault_agent_put(idPointer, idLength, keyPointer, keyLength),
      ),
    ),
  );

  bool vaultAgentForget(Uint8List id) => _withBytes(
    id,
    (idPointer, idLength) =>
        _requireBool(native.vault_agent_forget(idPointer, idLength)),
  );

  bool vaultAgentStop() => _requireBool(native.vault_agent_stop());

  bool vaultAgentStart() => _requireBool(native.vault_agent_start());

  List<AgentEntry> vaultAgentList() =>
      DomainDecoders.agentEntryList(_take(native.vault_agent_list()));

  SleepSupport vaultAgentSleepSupport() =>
      DomainDecoders.sleepSupport(_take(native.vault_agent_sleep_support()));

  PlatformStatus vaultPlatformStatus() =>
      DomainDecoders.platformStatus(_take(native.vault_platform_status()));

  bool vaultPlatformSetScope(String scope) => _withText(
    scope,
    (scopePointer, scopeLength) => _requireBool(
      native.vault_platform_set_scope(scopePointer, scopeLength),
    ),
  );

  bool vaultPlatformForgetPassword() =>
      _requireBool(native.vault_platform_forget_password());

  bool vaultPlatformPutPassword(Uint8List password) => _withBytes(
    password,
    (passwordPointer, passwordLength) => _requireBool(
      native.vault_platform_put_password(passwordPointer, passwordLength),
    ),
  );

  bool vaultPlatformEnable() => _requireBool(native.vault_platform_enable());

  bool vaultPlatformDisable() => _requireBool(native.vault_platform_disable());

  bool vaultPlatformDisabled() => native.vault_platform_disabled();

  Uint8List vaultPlatformGetPassword() =>
      _take(native.vault_platform_get_password());

  String vaultDefaultDirectory() =>
      _takeString(native.vault_default_directory());

  String vaultDefaultPath() => _takeString(native.vault_default_path());

  String vaultAgentLogPath() => _takeString(native.vault_agent_log_path());

  String vaultAgentLogDestination() =>
      _takeString(native.vault_agent_log_destination());

  Uint8List vaultAgentGetVaultUnlockKey(String vaultId) => _withText(
    vaultId,
    (vaultIdPointer, vaultIdLength) => _take(
      native.vault_agent_get_vault_unlock_key(vaultIdPointer, vaultIdLength),
    ),
  );

  bool vaultAgentPutVaultUnlockKey(
    String vaultId,
    Uint8List key,
    int ttlSeconds,
  ) => _withText(
    vaultId,
    (vaultIdPointer, vaultIdLength) => _withBytes(
      key,
      (keyPointer, keyLength) => _requireBool(
        native.vault_agent_put_vault_unlock_key(
          vaultIdPointer,
          vaultIdLength,
          keyPointer,
          keyLength,
          ttlSeconds,
        ),
      ),
    ),
  );

  bool vaultAgentForgetVaultUnlockKey(String vaultId) => _withText(
    vaultId,
    (vaultIdPointer, vaultIdLength) => _requireBool(
      native.vault_agent_forget_vault_unlock_key(vaultIdPointer, vaultIdLength),
    ),
  );

  ffi.Pointer<ffi.Void> vaultAgentGetOwnerSigningKey(
    String vaultId,
    String profile,
  ) => _withText(
    vaultId,
    (vaultIdPointer, vaultIdLength) => _withText(
      profile,
      (profilePointer, profileLength) => _requireHandle(
        native.vault_agent_get_owner_signing_key(
          vaultIdPointer,
          vaultIdLength,
          profilePointer,
          profileLength,
        ),
      ),
    ),
  );

  bool vaultAgentPutOwnerSigningKey(
    String vaultId,
    String profile,
    ffi.Pointer<ffi.Void> key,
    int ttlSeconds,
  ) => _withText(
    vaultId,
    (vaultIdPointer, vaultIdLength) => _withText(
      profile,
      (profilePointer, profileLength) => _requireBool(
        native.vault_agent_put_owner_signing_key(
          vaultIdPointer,
          vaultIdLength,
          profilePointer,
          profileLength,
          key,
          ttlSeconds,
        ),
      ),
    ),
  );

  bool vaultAgentForgetOwnerSigningKey(String vaultId, String profile) =>
      _withText(
        vaultId,
        (vaultIdPointer, vaultIdLength) => _withText(
          profile,
          (profilePointer, profileLength) => _requireBool(
            native.vault_agent_forget_owner_signing_key(
              vaultIdPointer,
              vaultIdLength,
              profilePointer,
              profileLength,
            ),
          ),
        ),
      );

  ffi.Pointer<ffi.Void> vaultAgentBeginActivity(String kind) => _withText(
    kind,
    (kindPointer, kindLength) => _requireHandle(
      native.vault_agent_begin_activity(kindPointer, kindLength),
    ),
  );

  void vaultAgentEndActivity(ffi.Pointer<ffi.Void> handle) =>
      native.vault_agent_end_activity(handle);

  ffi.Pointer<ffi.Void> vaultLocal() => _requireHandle(native.vault_local());

  ffi.Pointer<ffi.Void> vaultCreateLockboxPassword(
    ffi.Pointer<ffi.Void> vault,
    String path,
    Uint8List password,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireHandle(
        native.vault_create_lockbox_password(
          vault,
          pathPointer,
          pathLength,
          passwordPointer,
          passwordLength,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultOpenLockboxPassword(
    ffi.Pointer<ffi.Void> vault,
    String path,
    Uint8List password,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireHandle(
        native.vault_open_lockbox_password(
          vault,
          pathPointer,
          pathLength,
          passwordPointer,
          passwordLength,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultCreateLockboxContentKey(
    ffi.Pointer<ffi.Void> vault,
    String path,
    Uint8List contentKey,
    ffi.Pointer<ffi.Void> signingKey,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      contentKey,
      (contentKeyPointer, contentKeyLength) => _requireHandle(
        native.vault_create_lockbox_content_key(
          vault,
          pathPointer,
          pathLength,
          contentKeyPointer,
          contentKeyLength,
          signingKey,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultCreateLockboxContact(
    ffi.Pointer<ffi.Void> vault,
    String path,
    ffi.Pointer<ffi.Void> contact,
    String name,
    ffi.Pointer<ffi.Void> signingKey,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withText(
      name,
      (namePointer, nameLength) => _requireHandle(
        native.vault_create_lockbox_contact(
          vault,
          pathPointer,
          pathLength,
          contact,
          namePointer,
          nameLength,
          signingKey,
        ),
      ),
    ),
  );

  ffi.Pointer<ffi.Void> vaultOpenLockboxContentKey(
    ffi.Pointer<ffi.Void> vault,
    String path,
    Uint8List contentKey,
    ffi.Pointer<ffi.Void> signingKey,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      contentKey,
      (contentKeyPointer, contentKeyLength) => _requireHandle(
        native.vault_open_lockbox_content_key(
          vault,
          pathPointer,
          pathLength,
          contentKeyPointer,
          contentKeyLength,
          signingKey,
        ),
      ),
    ),
  );

  bool vaultCacheLockboxPassword(
    ffi.Pointer<ffi.Void> vault,
    String path,
    Uint8List password,
    int ttlSeconds,
  ) => _withText(
    path,
    (pathPointer, pathLength) => _withBytes(
      password,
      (passwordPointer, passwordLength) => _requireBool(
        native.vault_cache_lockbox_password(
          vault,
          pathPointer,
          pathLength,
          passwordPointer,
          passwordLength,
          ttlSeconds,
        ),
      ),
    ),
  );

  bool vaultCloseLockbox(ffi.Pointer<ffi.Void> vault, String path) => _withText(
    path,
    (pathPointer, pathLength) => _requireBool(
      native.vault_close_lockbox(vault, pathPointer, pathLength),
    ),
  );

  bool vaultCloseAll(ffi.Pointer<ffi.Void> vault) =>
      _requireBool(native.vault_close_all(vault));

  void vaultFree(ffi.Pointer<ffi.Void> vault) => native.vault_free(vault);
}
