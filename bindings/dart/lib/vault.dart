import 'dart:ffi' as ffi;
import 'dart:typed_data';

import 'revault_native.dart';
import 'src/binding_operations.dart';
import 'src/generated/revault_bindings.pb.dart' as pb;
import 'src/native_library.dart';

/// Entry point for encrypted lockboxes, keys, local vault metadata, the session
/// agent, and the platform secret store.
///
/// Call [load] once to locate and validate the native ABI. Structured results
/// are concrete protobuf classes. See the
/// [repository README](https://github.com/onepub-dev/reVault#readme) for a
/// complete example and guidance on handling secret values.
final class Vault {
  static Future<Vault> load() async => Vault(await loadNativeLibrary());

  Vault(ffi.DynamicLibrary library)
    : operations = BindingOperations(RevaultNative(library));

  final BindingOperations operations;
  String get lastError => operations.lastErrorMessage();
  pb.ErrorDetails get lastErrorDetails => operations.bufferLastErrorDetails();
  int get lockboxFormatVersion => operations.lockboxFormatVersion();
  int probeLockboxFormatVersion(Uint8List value) =>
      operations.lockboxProbeFormatVersion(value);
  int get currentVaultStructureVersion =>
      operations.vaultStructureVersionCurrent();
  int probeVaultStructureVersion(String root, Uint8List password) =>
      operations.vaultDirectoryProbeStructureVersion(root, password);

  ContactKeyPair generateContactKeyPair() =>
      ContactKeyPair._(this, operations.keyContactGenerate());
  ContactKeyPair contactKeyPairFromPrivate(Uint8List value) =>
      ContactKeyPair._(this, operations.keyContactFromPrivate(value));
  ContactKeyPair importContactKeyPair(Uint8List value) =>
      ContactKeyPair._(this, operations.vaultKeyImportPrivate(value));
  ContactPublicKey contactPublicKeyFromBytes(Uint8List value) =>
      ContactPublicKey._(this, operations.keyContactPublicFromBytes(value));
  ContactPublicKey importContactPublicKey(Uint8List value) =>
      ContactPublicKey._(this, operations.vaultKeyImportPublic(value));
  SigningKeyPair generateSigningKeyPair() =>
      SigningKeyPair._(this, operations.keySigningGenerate());
  SigningKeyPair signingKeyPairFromPrivate(Uint8List value) =>
      SigningKeyPair._(this, operations.keySigningFromPrivate(value));
  SigningPublicKey signingPublicKeyFromBytes(Uint8List value) =>
      SigningPublicKey._(this, operations.keySigningPublicFromBytes(value));

  String formatKeyHex(Uint8List value) => operations.vaultKeyFormatHex(value);
  Uint8List decodeKeyHex(String value) => operations.vaultKeyDecodeHex(value);
  String formatKeyCrockford(Uint8List value) =>
      operations.vaultKeyFormatCrockford(value);
  String formatKeyCrockfordReading(String value) =>
      operations.vaultKeyFormatCrockfordReading(value);
  Uint8List decodeKeyCrockford(String value) =>
      operations.vaultKeyDecodeCrockford(value);
  String hexEncode(Uint8List value) => operations.vaultKeyHexEncode(value);
  Uint8List hexDecode(String value) => operations.vaultKeyHexDecode(value);

  Lockbox createLockbox(Uint8List key, [LockboxOptions? options]) {
    final handle =
        options == null
            ? operations.lockboxCreate(key)
            : operations.lockboxCreateWithOptions(
              key,
              options.cacheMode,
              options.cacheBytes,
              options.workload,
              options.worker,
              options.jobs,
            );
    return Lockbox._(this, handle);
  }

  Lockbox createLockboxWithPassword(Uint8List password) =>
      Lockbox._(this, operations.lockboxCreatePassword(password));
  Lockbox createLockboxForContact(ContactPublicKey contact) =>
      Lockbox._(this, operations.lockboxCreateContact(contact._handle));
  Lockbox createSignedLockbox(Uint8List key, SigningKeyPair signing) =>
      Lockbox._(
        this,
        operations.lockboxCreateWithSigningKey(key, signing._handle),
      );
  Lockbox openLockbox(
    Uint8List archive,
    Uint8List key, [
    LockboxOptions? options,
  ]) {
    final handle =
        options == null
            ? operations.lockboxOpen(archive, key)
            : operations.lockboxOpenWithOptions(
              archive,
              key,
              options.cacheMode,
              options.cacheBytes,
              options.workload,
              options.worker,
              options.jobs,
            );
    return Lockbox._(this, handle);
  }

  Lockbox openLockboxWithPassword(Uint8List archive, Uint8List password) =>
      Lockbox._(this, operations.lockboxOpenPassword(archive, password));
  Lockbox openLockboxForContact(Uint8List archive, ContactKeyPair contact) =>
      Lockbox._(this, operations.lockboxOpenContact(archive, contact._handle));
  pb.FileInspection inspectLockboxFile(String path) =>
      operations.lockboxInspectFile(path);
  pb.RecoveryReport scanLockboxPath(String path, Uint8List key) =>
      operations.lockboxRecoveryScanPath(path, key);
  pb.RecoveryReport scanLockbox(Uint8List archive, Uint8List key) =>
      operations.lockboxRecoveryScan(archive, key);
  Lockbox salvageLockbox(
    Uint8List archive,
    Uint8List key, [
    SigningKeyPair? signing,
  ]) => Lockbox._(
    this,
    operations.lockboxRecoverySalvage(
      archive,
      key,
      signing?._handle ?? ffi.nullptr,
    ),
  );

  VaultDirectory openVaultDirectory(String root, Uint8List password) =>
      VaultDirectory._(this, operations.vaultDirectoryOpen(root, password));
  ReadOnlyVaultDirectory openReadOnlyVaultDirectory(
    String root,
    Uint8List password,
  ) => ReadOnlyVaultDirectory._(
    this,
    operations.vaultReadOnlyOpen(root, password),
  );
  ReadOnlyVaultDirectory openDefaultReadOnlyVaultDirectory(
    Uint8List password,
  ) => ReadOnlyVaultDirectory._(
    this,
    operations.vaultReadOnlyOpenDefault(password),
  );
  VaultDirectory openOrCreateVaultDirectory(String root, Uint8List password) =>
      VaultDirectory._(
        this,
        operations.vaultDirectoryOpenOrCreate(root, password),
      );
  VaultDirectory replaceVaultDirectory(String root, Uint8List password) =>
      VaultDirectory._(this, operations.vaultDirectoryReplace(root, password));
  VaultDirectory openOrCreateDefaultVaultDirectory(Uint8List password) =>
      VaultDirectory._(
        this,
        operations.vaultDirectoryOpenOrCreateDefault(password),
      );
  VaultDirectory replaceDefaultVaultDirectory(Uint8List password) =>
      VaultDirectory._(this, operations.vaultDirectoryReplaceDefault(password));
  void changeVaultDirectoryPassword(
    String root,
    Uint8List oldPassword,
    Uint8List newPassword,
  ) => operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  void changeDefaultVaultDirectoryPassword(
    Uint8List oldPassword,
    Uint8List newPassword,
  ) => operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  String get defaultVaultDirectory => operations.vaultDefaultDirectory();
  String get defaultVaultPath => operations.vaultDefaultPath();
  pb.VaultBackupManifest backupDefaultVault(
    String path, {
    bool overwrite = false,
  }) => operations.vaultBackupDefault(path, overwrite);
  pb.VaultBackupManifest restoreDefaultVault(
    String path, {
    bool overwrite = false,
  }) => operations.vaultRestoreDefault(path, overwrite);

  bool get agentIsRunning => operations.vaultIsRunning();
  void serveAgent() => operations.vaultAgentServe();
  void verifyAgentTransport() => operations.vaultAgentVerifyTransport();
  void forgetAllAgentSecrets() => operations.vaultForgetAll();
  void stopAgent() => operations.vaultAgentStop();
  void startAgent() => operations.vaultAgentStart();
  void putAgentKey(Uint8List id, Uint8List key) =>
      operations.vaultAgentPut(id, key);
  Uint8List getAgentKey(Uint8List id) => operations.vaultAgentGet(id);
  void forgetAgentKey(Uint8List id) => operations.vaultAgentForget(id);
  pb.AgentEntryList listAgentKeys() => operations.vaultAgentList();
  pb.SleepSupport agentSleepSupport() => operations.vaultAgentSleepSupport();
  String get agentLogPath => operations.vaultAgentLogPath();
  String get agentLogDestination => operations.vaultAgentLogDestination();
  void putAgentVaultUnlockKey(String vaultId, Uint8List key, int ttlSeconds) =>
      operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
  Uint8List getAgentVaultUnlockKey(String vaultId) =>
      operations.vaultAgentGetVaultUnlockKey(vaultId);
  void forgetAgentVaultUnlockKey(String vaultId) =>
      operations.vaultAgentForgetVaultUnlockKey(vaultId);
  void putAgentOwnerSigningKey(
    String vaultId,
    String profile,
    SigningKeyPair key,
    int ttlSeconds,
  ) => operations.vaultAgentPutOwnerSigningKey(
    vaultId,
    profile,
    key._handle,
    ttlSeconds,
  );
  SigningKeyPair getAgentOwnerSigningKey(String vaultId, String profile) =>
      SigningKeyPair._(
        this,
        operations.vaultAgentGetOwnerSigningKey(vaultId, profile),
      );
  void forgetAgentOwnerSigningKey(String vaultId, String profile) =>
      operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);
  AgentActivity beginAgentActivity(String kind) =>
      AgentActivity._(this, operations.vaultAgentBeginActivity(kind));

  pb.PlatformStatus platformStatus() => operations.vaultPlatformStatus();
  void setPlatformScope(String scope) =>
      operations.vaultPlatformSetScope(scope);
  void enablePlatformStore() => operations.vaultPlatformEnable();
  void disablePlatformStore() => operations.vaultPlatformDisable();
  bool get platformStoreDisabled => operations.vaultPlatformDisabled();
  void putPlatformPassword(Uint8List password) =>
      operations.vaultPlatformPutPassword(password);
  Uint8List getPlatformPassword() => operations.vaultPlatformGetPassword();
  void forgetPlatformPassword() => operations.vaultPlatformForgetPassword();
  LocalVault openLocalVault() => LocalVault._(this, operations.vaultLocal());
}

/// Runtime cache and worker tuning used when a lockbox is created or opened.
final class LockboxOptions {
  const LockboxOptions({
    this.cacheMode = 'bytes',
    this.cacheBytes = 64 << 20,
    this.workload = 'interactive',
    this.worker = 'auto',
    this.jobs = 0,
  });
  final String cacheMode, workload, worker;
  final int cacheBytes, jobs;
}

abstract base class _Owned {
  _Owned(this.vault, this._handle);
  final Vault vault;
  ffi.Pointer<ffi.Void> _handle;
  bool get disposed => _handle == ffi.nullptr;
}

/// Shareable contact public key used to encrypt a content key for a recipient.
final class ContactPublicKey extends _Owned {
  ContactPublicKey._(super.vault, super.handle);
  Uint8List export(String format) =>
      vault.operations.vaultKeyExportPublic(_handle, format);
  Uint8List fingerprint() => vault.operations.vaultKeyFingerprint(_handle);
  WrappedContactKey encrypt(Uint8List key) => WrappedContactKey._(
    vault,
    vault.operations.keyContactEncrypt(_handle, key),
  );
  void dispose() {
    if (!disposed) {
      vault.operations.keyContactPublicFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Owned encrypted content-key envelope for one contact recipient.
final class WrappedContactKey extends _Owned {
  WrappedContactKey._(super.vault, super.handle);
  Uint8List publicBytes() => vault.operations.keyContactWrappedPublic(_handle);
  Uint8List ciphertext() =>
      vault.operations.keyContactWrappedCiphertext(_handle);
  Uint8List encryptedBytes() =>
      vault.operations.keyContactWrappedEncrypted(_handle);
  void dispose() {
    if (!disposed) {
      vault.operations.keyContactWrappedFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Owned contact key pair used to decrypt content keys sent by contacts.
final class ContactKeyPair extends _Owned {
  ContactKeyPair._(super.vault, super.handle);
  Uint8List publicBytes() => vault.operations.keyContactPublic(_handle);
  Uint8List privateRecord() => vault.operations.keyContactPrivate(_handle);
  ContactPublicKey publicKey() =>
      vault.contactPublicKeyFromBytes(publicBytes());
  Uint8List export(String format) =>
      vault.operations.vaultKeyExportPrivate(_handle, format);
  Uint8List decrypt(WrappedContactKey wrapped) =>
      vault.operations.keyContactDecrypt(_handle, wrapped._handle);
  void dispose() {
    if (!disposed) {
      vault.operations.keyContactFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Shareable public key used to verify owner-authorized lockbox commits.
final class SigningPublicKey extends _Owned {
  SigningPublicKey._(super.vault, super.handle);
  void dispose() {
    if (!disposed) {
      vault.operations.keySigningPublicFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Owned signing key pair used to authorize mutable lockbox commits.
final class SigningKeyPair extends _Owned {
  SigningKeyPair._(super.vault, super.handle);
  Uint8List publicBytes() => vault.operations.keySigningPublic(_handle);
  Uint8List privateRecord() => vault.operations.keySigningPrivate(_handle);
  SigningPublicKey publicKey() =>
      vault.signingPublicKeyFromBytes(publicBytes());
  void dispose() {
    if (!disposed) {
      vault.operations.keySigningFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Owned, mutable view of an encrypted lockbox archive.
///
/// Call [commit] after mutations and [dispose] when the native handle is no
/// longer required.
final class Lockbox extends _Owned {
  Lockbox._(super.vault, super.handle);
  void addFile(
    String path,
    Uint8List value, {
    int? permissions,
    bool replace = false,
  }) {
    if (permissions == null) {
      vault.operations.lockboxAddFile(_handle, path, value, replace);
    } else {
      vault.operations.lockboxAddFileWithPermissions(
        _handle,
        path,
        value,
        permissions,
        replace,
      );
    }
  }

  Uint8List getFile(String path) =>
      vault.operations.lockboxGetFile(_handle, path);
  void extractFile(String source, String destination, {bool replace = false}) =>
      vault.operations.lockboxExtractFile(
        _handle,
        source,
        destination,
        replace,
      );
  void extractDirectory(
    String destination, {
    required int maxFileBytes,
    required int maxTotalBytes,
    required int maxFiles,
    bool restoreSymlinks = false,
    bool restorePermissions = false,
    bool overwrite = false,
  }) => vault.operations.lockboxExtractDirectory(
    _handle,
    destination,
    maxFileBytes,
    maxTotalBytes,
    maxFiles,
    restoreSymlinks,
    restorePermissions,
    overwrite,
  );
  pb.StreamChunkList streamContent({bool physical = false}) =>
      vault.operations.lockboxStreamContent(_handle, physical);
  pb.CacheStats cacheStats() => vault.operations.lockboxCacheStats(_handle);
  pb.ImportStats importStats() => vault.operations.lockboxImportStats(_handle);
  void resetImportStats() => vault.operations.lockboxResetImportStats(_handle);
  pb.PageInspectionList pageInspection() =>
      vault.operations.lockboxPageInspection(_handle);
  pb.RecoveryReport recoveryReport() =>
      vault.operations.lockboxRecoveryReport(_handle);
  String renderRecoveryReport({bool verbose = false, int maxEntries = 100}) =>
      vault.operations.lockboxRecoveryReportRender(
        _handle,
        verbose,
        maxEntries,
      );
  int get storageLength => vault.operations.lockboxStorageLen(_handle);
  void setWorkloadProfile(String profile) =>
      vault.operations.lockboxSetWorkloadProfile(_handle, profile);
  void setWorkerPolicy(String mode, int jobs) =>
      vault.operations.lockboxSetWorkerPolicy(_handle, mode, jobs);
  pb.RuntimeOptions runtimeOptions() =>
      vault.operations.lockboxRuntimeOptions(_handle);
  void commit() => vault.operations.lockboxCommit(_handle);
  void createDirectory(String path, {bool parents = false}) =>
      vault.operations.lockboxCreateDir(_handle, path, parents);
  void delete(String path) => vault.operations.lockboxDelete(_handle, path);
  void removeDirectory(String path, {bool recursive = false}) =>
      vault.operations.lockboxRemoveDir(_handle, path, recursive);
  void createParentDirectories(String path) =>
      vault.operations.lockboxCreateParentDirs(_handle, path);
  void rename(String from, String to) =>
      vault.operations.lockboxRename(_handle, from, to);
  pb.LockboxEntryList list(String path, {bool recursive = false}) =>
      vault.operations.lockboxList(_handle, path, recursive);
  pb.LockboxEntryList listWithOptions(
    String path,
    String glob, {
    bool recursive = false,
    bool includeFiles = true,
    bool includeSymlinks = true,
    bool includeDirectories = true,
    int limit = 0,
  }) => vault.operations.lockboxListWithOptions(
    _handle,
    path,
    glob,
    recursive,
    includeFiles,
    includeSymlinks,
    includeDirectories,
    limit,
  );
  pb.OptionalLockboxEntry stat(String path) =>
      vault.operations.lockboxStat(_handle, path);
  void setVariable(String name, String value) =>
      vault.operations.lockboxSetVariable(_handle, name, value);

  /// Stores a secret variable without converting it to an immutable String.
  void setSecretVariable(String name, Uint8List value) =>
      vault.operations.lockboxSetSecretVariable(_handle, name, value);
  String? getVariable(String name) {
    final value = vault.operations.lockboxGetVariable(_handle, name);
    return value.present ? value.value : null;
  }

  /// Invokes [callback] with temporary secret bytes, then wipes the transfer copy.
  T? withSecretVariable<T>(
    String name,
    T Function(Uint8List secret) callback,
  ) => vault.operations.lockboxWithSecretVariable(_handle, name, callback);
  void deleteVariable(String name) =>
      vault.operations.lockboxDeleteVariable(_handle, name);
  void moveVariables(pb.PathMoveList moves) => vault.operations
      .lockboxMoveVariables(_handle, Uint8List.fromList(moves.writeToBuffer()));
  pb.VariableList listVariables() =>
      vault.operations.lockboxListVariables(_handle);
  pb.OptionalString variableSensitivity(String name) =>
      vault.operations.lockboxVariableSensitivity(_handle, name);
  void addSymlink(String path, String target, {bool replace = false}) =>
      vault.operations.lockboxAddSymlink(_handle, path, target, replace);
  String symlinkTarget(String path) =>
      vault.operations.lockboxGetSymlinkTarget(_handle, path);
  Uint8List get id => vault.operations.lockboxId(_handle);
  bool exists(String path) => vault.operations.lockboxExists(_handle, path);
  bool isDirectory(String path) => vault.operations.lockboxIsDir(_handle, path);
  int permissions(String path) =>
      vault.operations.lockboxPermissions(_handle, path);
  void setPermissions(String path, int value) =>
      vault.operations.lockboxSetPermissions(_handle, path, value);
  Uint8List readRange(String path, int offset, int length) =>
      vault.operations.lockboxReadRange(_handle, path, offset, length);
  int addPassword(Uint8List password) =>
      vault.operations.lockboxAddPassword(_handle, password);
  int addContact(ContactPublicKey contact, String name) =>
      vault.operations.lockboxAddContact(_handle, contact._handle, name);
  void deleteKey(int id) => vault.operations.lockboxDeleteKey(_handle, id);
  pb.KeySlotList listKeySlots() =>
      vault.operations.lockboxListKeySlots(_handle);
  void setOwnerSigningKey(SigningKeyPair key) =>
      vault.operations.lockboxSetOwnerSigningKey(_handle, key._handle);
  pb.OwnerInspection ownerInspection() =>
      vault.operations.lockboxOwnerInspection(_handle);
  pb.FormDefinition defineForm(
    String alias,
    String name,
    String description,
    pb.FormFieldList fields,
  ) => vault.operations.lockboxDefineForm(
    _handle,
    alias,
    name,
    description,
    Uint8List.fromList(fields.writeToBuffer()),
  );
  pb.FormDefinitionList listFormDefinitions() =>
      vault.operations.lockboxListFormDefinitions(_handle);
  pb.FormDefinition resolveForm(String reference) =>
      vault.operations.lockboxResolveForm(_handle, reference);
  pb.FormDefinitionList listFormRevisions(String typeId) =>
      vault.operations.lockboxListFormRevisions(_handle, typeId);
  pb.FormRecord createFormRecord(
    String path,
    String typeReference,
    String name,
  ) => vault.operations.lockboxCreateFormRecord(
    _handle,
    path,
    typeReference,
    name,
  );
  void setFormField(String path, String field, String value) =>
      vault.operations.lockboxSetFormField(_handle, path, field, value);

  /// Stores a secret form field without creating an immutable String.
  void setSecretFormField(String path, String field, Uint8List value) =>
      vault.operations.lockboxSetSecretFormField(_handle, path, field, value);
  pb.FormRecordList listFormRecords() =>
      vault.operations.lockboxListFormRecords(_handle);
  pb.OptionalFormRecord getFormRecord(String path) =>
      vault.operations.lockboxGetFormRecord(_handle, path);
  void deleteFormRecord(String path) =>
      vault.operations.lockboxDeleteFormRecord(_handle, path);
  void moveFormRecords(pb.PathMoveList moves) =>
      vault.operations.lockboxMoveFormRecords(
        _handle,
        Uint8List.fromList(moves.writeToBuffer()),
      );
  pb.OptionalFormValue getFormField(String path, String field) =>
      vault.operations.lockboxGetFormField(_handle, path, field);

  /// Invokes [callback] with temporary field bytes, then wipes the transfer copy.
  T? withSecretFormField<T>(
    String path,
    String field,
    T Function(Uint8List secret) callback,
  ) => vault.operations.lockboxWithSecretFormField(
    _handle,
    path,
    field,
    callback,
  );
  Uint8List get bytes => vault.operations.lockboxToBytes(_handle);
  void dispose() {
    if (!disposed) {
      vault.operations.lockboxFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Writable, password-protected local metadata vault.
final class VaultDirectory extends _Owned {
  VaultDirectory._(super.vault, super.handle);
  String get root => vault.operations.vaultDirectoryRoot(_handle);
  int get structureVersion =>
      vault.operations.vaultDirectoryStructureVersion(_handle);
  pb.StringList listPrivateKeys() =>
      vault.operations.vaultDirectoryListPrivateKeys(_handle);
  pb.StringList listPrivateKeyNames() =>
      vault.operations.vaultDirectoryListPrivateKeyNames(_handle);
  pb.StringList listContactNames() =>
      vault.operations.vaultDirectoryListContactNames(_handle);
  pb.StringList listFormAliases() =>
      vault.operations.vaultDirectoryListFormAliases(_handle);
  bool privateKeyExists(String name) =>
      vault.operations.vaultDirectoryPrivateKeyExists(_handle, name);
  void deletePrivateKey(String name) =>
      vault.operations.vaultDirectoryDeletePrivateKey(_handle, name);
  void storePrivateKey(String name, ContactKeyPair key) => vault.operations
      .vaultDirectoryStorePrivateKey(_handle, name, key._handle);
  ContactKeyPair loadPrivateKey(String name) => ContactKeyPair._(
    vault,
    vault.operations.vaultDirectoryLoadPrivateKey(_handle, name),
  );
  ContactKeyPair loadPrivateKeyGeneration(String name, int index) =>
      ContactKeyPair._(
        vault,
        vault.operations.vaultDirectoryLoadPrivateKeyGeneration(
          _handle,
          name,
          index,
        ),
      );
  void storeContact(String name, ContactPublicKey key) =>
      vault.operations.vaultDirectoryStoreContact(_handle, name, key._handle);
  ContactPublicKey loadContact(String name) => ContactPublicKey._(
    vault,
    vault.operations.vaultDirectoryLoadContact(_handle, name),
  );
  bool contactExists(String name) =>
      vault.operations.vaultDirectoryContactExists(_handle, name);
  void deleteContact(String name) =>
      vault.operations.vaultDirectoryDeleteContact(_handle, name);
  pb.ContactList listContacts() =>
      vault.operations.vaultDirectoryListContacts(_handle);
  void storeProfileEmail(String name, String email) =>
      vault.operations.vaultDirectoryStoreProfileEmail(_handle, name, email);
  pb.OptionalString profileEmail(String name) =>
      vault.operations.vaultDirectoryProfileEmail(_handle, name);
  void storeBackup(Uint8List id, Uint8List value) =>
      vault.operations.vaultDirectoryStoreBackup(_handle, id, value);
  Uint8List loadBackup(Uint8List id) =>
      vault.operations.vaultDirectoryLoadBackup(_handle, id);
  int get backupCount => vault.operations.vaultDirectoryBackupCount(_handle);
  void restorePrivateKey(
    String name,
    ContactKeyPair key,
    SigningKeyPair signing, {
    bool overwrite = false,
  }) => vault.operations.vaultDirectoryRestorePrivateKey(
    _handle,
    name,
    key._handle,
    signing._handle,
    overwrite,
  );
  SigningKeyPair loadOwnerSigningKey(String name) => SigningKeyPair._(
    vault,
    vault.operations.vaultDirectoryLoadOwnerSigningKey(_handle, name),
  );
  SigningKeyPair loadOwnerSigningKeyGeneration(String name, int index) =>
      SigningKeyPair._(
        vault,
        vault.operations.vaultDirectoryLoadOwnerSigningKeyGeneration(
          _handle,
          name,
          index,
        ),
      );
  void storeContactSigningKey(String name, SigningPublicKey key) => vault
      .operations
      .vaultDirectoryStoreContactSigningKey(_handle, name, key._handle);
  SigningPublicKey loadContactSigningKey(String name) => SigningPublicKey._(
    vault,
    vault.operations.vaultDirectoryLoadContactSigningKey(_handle, name),
  );
  pb.ProfileHistory listProfileGenerations(String name) =>
      vault.operations.vaultDirectoryListProfileGenerations(_handle, name);
  pb.ProfileHistory rotatePrivateKey(String name) =>
      vault.operations.vaultDirectoryRotatePrivateKey(_handle, name);
  void rememberLockbox(Uint8List id, String path) =>
      vault.operations.vaultDirectoryRememberLockbox(_handle, id, path);
  pb.KnownLockboxList listKnownLockboxes() =>
      vault.operations.vaultDirectoryListKnownLockboxes(_handle);
  void forgetLockbox(String path) =>
      vault.operations.vaultDirectoryForgetLockbox(_handle, path);
  void rememberAccessSlotLabel(Uint8List id, int slotId, String name) => vault
      .operations
      .vaultDirectoryRememberAccessSlotLabel(_handle, id, slotId, name);
  pb.AccessSlotLabelList listAccessSlotLabels(Uint8List id) =>
      vault.operations.vaultDirectoryListAccessSlotLabels(_handle, id);
  pb.AccessSlotLabelList findAccessSlotLabels(Uint8List id, String name) =>
      vault.operations.vaultDirectoryFindAccessSlotLabels(_handle, id, name);
  void forgetAccessSlotLabel(Uint8List id, int slotId) =>
      vault.operations.vaultDirectoryForgetAccessSlotLabel(_handle, id, slotId);
  pb.FormDefinition defineForm(
    String alias,
    String name,
    String description,
    pb.FormFieldList fields,
  ) => vault.operations.vaultDirectoryDefineForm(
    _handle,
    alias,
    name,
    description,
    Uint8List.fromList(fields.writeToBuffer()),
  );
  pb.FormDefinition resolveForm(String reference) =>
      vault.operations.vaultDirectoryResolveForm(_handle, reference);
  pb.FormDefinitionList listForms() =>
      vault.operations.vaultDirectoryListForms(_handle);
  pb.FormDefinitionList listFormRevisions(String typeId) =>
      vault.operations.vaultDirectoryListFormRevisions(_handle, typeId);
  int seedForms() => vault.operations.vaultDirectorySeedForms(_handle);
  void rememberPassword(Uint8List id, Uint8List password) =>
      vault.operations.vaultDirectoryRememberPassword(_handle, id, password);
  Uint8List rememberedPassword(Uint8List id) =>
      vault.operations.vaultDirectoryRememberedPassword(_handle, id);
  void dispose() {
    if (!disposed) {
      vault.operations.vaultDirectoryFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Read-only metadata view that never loads the owner signing key.
final class ReadOnlyVaultDirectory extends _Owned {
  ReadOnlyVaultDirectory._(super.vault, super.handle);
  pb.StringList listProfileNames() =>
      vault.operations.vaultReadOnlyListProfileNames(_handle);
  pb.StringList listContactNames() =>
      vault.operations.vaultReadOnlyListContactNames(_handle);
  pb.StringList listFormAliases() =>
      vault.operations.vaultReadOnlyListFormAliases(_handle);
  pb.KnownLockboxList listKnownLockboxes() =>
      vault.operations.vaultReadOnlyListKnownLockboxes(_handle);
  void dispose() {
    if (!disposed) {
      vault.operations.vaultReadOnlyFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// Owned registration of an operation that currently requires secret access.
final class AgentActivity extends _Owned {
  AgentActivity._(super.vault, super.handle);
  void dispose() {
    if (!disposed) {
      vault.operations.vaultAgentEndActivity(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// High-level workflow for local metadata and remembered lockbox files.
final class LocalVault extends _Owned {
  LocalVault._(super.vault, super.handle);
  Lockbox createWithPassword(String path, Uint8List password) => Lockbox._(
    vault,
    vault.operations.vaultCreateLockboxPassword(_handle, path, password),
  );
  Lockbox openWithPassword(String path, Uint8List password) => Lockbox._(
    vault,
    vault.operations.vaultOpenLockboxPassword(_handle, path, password),
  );
  Lockbox createWithContentKey(
    String path,
    Uint8List key,
    SigningKeyPair signing,
  ) => Lockbox._(
    vault,
    vault.operations.vaultCreateLockboxContentKey(
      _handle,
      path,
      key,
      signing._handle,
    ),
  );
  Lockbox openWithContentKey(
    String path,
    Uint8List key,
    SigningKeyPair signing,
  ) => Lockbox._(
    vault,
    vault.operations.vaultOpenLockboxContentKey(
      _handle,
      path,
      key,
      signing._handle,
    ),
  );
  Lockbox createForContact(
    String path,
    ContactPublicKey contact,
    String name,
    SigningKeyPair signing,
  ) => Lockbox._(
    vault,
    vault.operations.vaultCreateLockboxContact(
      _handle,
      path,
      contact._handle,
      name,
      signing._handle,
    ),
  );
  void cachePassword(String path, Uint8List password, int ttlSeconds) => vault
      .operations
      .vaultCacheLockboxPassword(_handle, path, password, ttlSeconds);
  void closeLockbox(String path) =>
      vault.operations.vaultCloseLockbox(_handle, path);
  void closeAll() => vault.operations.vaultCloseAll(_handle);
  void dispose() {
    if (!disposed) {
      vault.operations.vaultFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}
