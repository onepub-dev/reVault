import 'dart:ffi' as ffi;
import 'dart:typed_data';

import 'src/revault_native.dart';
import 'src/binding_operations.dart';
import 'src/domain_models.dart';
import 'src/native_library.dart';

/// Entry point for encrypted lockboxes, keys, local vault metadata, the session
/// agent, and the platform secret store.
///
/// Call [load] once when the application starts, then use the returned object
/// to create or open lockboxes, manage recipient and signing keys, and access
/// the local services used to remember vaults and credentials. See the
/// [repository README](https://github.com/onepub-dev/reVault#readme) for a
/// complete example and guidance on handling secret values.
final class Vault {
  /// Loads the bundled native library, verifies its ABI, and creates a facade.
  static Future<Vault> load() async => Vault(await loadNativeLibrary());

  /// Creates a facade over an already loaded reVault [library].
  Vault(ffi.DynamicLibrary library)
    : _operations = BindingOperations(RevaultNative(library));

  final BindingOperations _operations;

  /// Returns the diagnostic for the most recent failed call on this isolate.
  String get lastError => _operations.lastErrorMessage();

  /// Returns structured details for the most recent failed native call.
  ErrorDetails get lastErrorDetails => _operations.bufferLastErrorDetails();

  /// Returns the lockbox format version written by this library.
  int get lockboxFormatVersion => _operations.lockboxFormatVersion();

  /// Reads the format version from lockbox [value] without opening it.
  int probeLockboxFormatVersion(Uint8List value) =>
      _operations.lockboxProbeFormatVersion(value);

  /// Returns the current on-disk local-vault structure version.
  int get currentVaultStructureVersion =>
      _operations.vaultStructureVersionCurrent();

  /// Opens enough metadata at [root] to determine its structure version.
  int probeVaultStructureVersion(String root, Uint8List password) =>
      _operations.vaultDirectoryProbeStructureVersion(root, password);

  /// Generates a contact encryption key pair in secure native memory.
  ContactKeyPair generateContactKeyPair() =>
      ContactKeyPair._(this, _operations.keyContactGenerate());

  /// Reconstructs a contact key pair from an exported private [value].
  ContactKeyPair contactKeyPairFromPrivate(Uint8List value) =>
      ContactKeyPair._(this, _operations.keyContactFromPrivate(value));

  /// Imports a supported private-key record from [value].
  ContactKeyPair importContactKeyPair(Uint8List value) =>
      ContactKeyPair._(this, _operations.vaultKeyImportPrivate(value));

  /// Reconstructs a contact public key from its canonical [value].
  ContactPublicKey contactPublicKeyFromBytes(Uint8List value) =>
      ContactPublicKey._(this, _operations.keyContactPublicFromBytes(value));

  /// Imports a supported public-key record from [value].
  ContactPublicKey importContactPublicKey(Uint8List value) =>
      ContactPublicKey._(this, _operations.vaultKeyImportPublic(value));

  /// Generates an owner signing key pair in secure native memory.
  SigningKeyPair generateSigningKeyPair() =>
      SigningKeyPair._(this, _operations.keySigningGenerate());

  /// Reconstructs an owner signing key pair from a private record.
  SigningKeyPair signingKeyPairFromPrivate(Uint8List value) =>
      SigningKeyPair._(this, _operations.keySigningFromPrivate(value));

  /// Reconstructs a signing public key from its canonical bytes.
  SigningPublicKey signingPublicKeyFromBytes(Uint8List value) =>
      SigningPublicKey._(this, _operations.keySigningPublicFromBytes(value));

  /// Formats key [value] as the canonical grouped hexadecimal fingerprint.
  String formatKeyHex(Uint8List value) => _operations.vaultKeyFormatHex(value);

  /// Decodes a canonical grouped hexadecimal key representation.
  Uint8List decodeKeyHex(String value) => _operations.vaultKeyDecodeHex(value);

  /// Formats key [value] using the canonical Crockford representation.
  String formatKeyCrockford(Uint8List value) =>
      _operations.vaultKeyFormatCrockford(value);

  /// Normalizes [value] for human-readable Crockford comparison.
  String formatKeyCrockfordReading(String value) =>
      _operations.vaultKeyFormatCrockfordReading(value);

  /// Decodes a Crockford key representation into canonical bytes.
  Uint8List decodeKeyCrockford(String value) =>
      _operations.vaultKeyDecodeCrockford(value);

  /// Encodes arbitrary bytes as lowercase hexadecimal text.
  String hexEncode(Uint8List value) => _operations.vaultKeyHexEncode(value);

  /// Decodes hexadecimal [value] into bytes.
  Uint8List hexDecode(String value) => _operations.vaultKeyHexDecode(value);

  /// Creates an unsigned in-memory lockbox protected by [key].
  ///
  /// [key] must contain a valid content key. When supplied, [options] controls
  /// cache and worker behavior. Call [Lockbox.commit] after mutations and
  /// [Lockbox.dispose] when finished.
  Lockbox createLockbox(Uint8List key, [LockboxOptions? options]) {
    final handle = options == null
        ? _operations.lockboxCreate(key)
        : _operations.lockboxCreateWithOptions(
            key,
            options.cacheMode,
            options.cacheBytes,
            options.workload,
            options.worker,
            options.jobs,
          );
    return Lockbox._(this, handle);
  }

  /// Creates an in-memory lockbox protected by [password].
  Lockbox createLockboxWithPassword(Uint8List password) =>
      Lockbox._(this, _operations.lockboxCreatePassword(password));

  /// Creates a lockbox whose content key is wrapped for [contact].
  Lockbox createLockboxForContact(ContactPublicKey contact) =>
      Lockbox._(this, _operations.lockboxCreateContact(contact._handle));

  /// Creates a lockbox protected by [key] and authorized by [signing].
  Lockbox createSignedLockbox(Uint8List key, SigningKeyPair signing) =>
      Lockbox._(
        this,
        _operations.lockboxCreateWithSigningKey(key, signing._handle),
      );

  /// Opens [archive] using its content [key].
  ///
  /// When supplied, [options] controls cache and worker behavior for the
  /// returned mutable lockbox.
  Lockbox openLockbox(
    Uint8List archive,
    Uint8List key, [
    LockboxOptions? options,
  ]) {
    final handle = options == null
        ? _operations.lockboxOpen(archive, key)
        : _operations.lockboxOpenWithOptions(
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

  /// Opens [archive] using a password access slot.
  Lockbox openLockboxWithPassword(Uint8List archive, Uint8List password) =>
      Lockbox._(this, _operations.lockboxOpenPassword(archive, password));

  /// Opens [archive] using a contact private key.
  Lockbox openLockboxForContact(Uint8List archive, ContactKeyPair contact) =>
      Lockbox._(this, _operations.lockboxOpenContact(archive, contact._handle));

  /// Inspects clear-text structural metadata in the lockbox file at [path].
  FileInspection inspectLockboxFile(String path) =>
      _operations.lockboxInspectFile(path);

  /// Scans a possibly damaged lockbox at [path] using [key].
  RecoveryReport scanLockboxPath(String path, Uint8List key) =>
      _operations.lockboxRecoveryScanPath(path, key);

  /// Scans possibly damaged lockbox [archive] bytes using [key].
  RecoveryReport scanLockbox(Uint8List archive, Uint8List key) =>
      _operations.lockboxRecoveryScan(archive, key);

  /// Salvages intact state from [archive] into a new clean lockbox.
  ///
  /// [signing] is required when the recovered lockbox must remain owner-signed.
  Lockbox salvageLockbox(
    Uint8List archive,
    Uint8List key, [
    SigningKeyPair? signing,
  ]) => Lockbox._(
    this,
    _operations.lockboxRecoverySalvage(
      archive,
      key,
      signing?._handle ?? ffi.nullptr,
    ),
  );

  /// Opens the writable local metadata vault at [root].
  VaultDirectory openVaultDirectory(String root, Uint8List password) =>
      VaultDirectory._(this, _operations.vaultDirectoryOpen(root, password));

  /// Opens a metadata-only view at [root] without loading signing keys.
  ReadOnlyVaultDirectory openReadOnlyVaultDirectory(
    String root,
    Uint8List password,
  ) => ReadOnlyVaultDirectory._(
    this,
    _operations.vaultReadOnlyOpen(root, password),
  );

  /// Opens the default metadata vault without loading signing keys.
  ReadOnlyVaultDirectory openDefaultReadOnlyVaultDirectory(
    Uint8List password,
  ) => ReadOnlyVaultDirectory._(
    this,
    _operations.vaultReadOnlyOpenDefault(password),
  );

  /// Opens or creates a writable metadata vault at [root].
  VaultDirectory openOrCreateVaultDirectory(String root, Uint8List password) =>
      VaultDirectory._(
        this,
        _operations.vaultDirectoryOpenOrCreate(root, password),
      );

  /// Replaces the metadata vault at [root] with a new empty vault.
  VaultDirectory replaceVaultDirectory(String root, Uint8List password) =>
      VaultDirectory._(this, _operations.vaultDirectoryReplace(root, password));

  /// Opens or creates the platform-default metadata vault.
  VaultDirectory openOrCreateDefaultVaultDirectory(Uint8List password) =>
      VaultDirectory._(
        this,
        _operations.vaultDirectoryOpenOrCreateDefault(password),
      );

  /// Replaces the platform-default metadata vault with a new empty vault.
  VaultDirectory replaceDefaultVaultDirectory(Uint8List password) =>
      VaultDirectory._(
        this,
        _operations.vaultDirectoryReplaceDefault(password),
      );

  /// Changes the password protecting the metadata vault at [root].
  void changeVaultDirectoryPassword(
    String root,
    Uint8List oldPassword,
    Uint8List newPassword,
  ) => _operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);

  /// Changes the password protecting the platform-default metadata vault.
  void changeDefaultVaultDirectoryPassword(
    Uint8List oldPassword,
    Uint8List newPassword,
  ) =>
      _operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);

  /// Returns the platform-default metadata-vault directory.
  String get defaultVaultDirectory => _operations.vaultDefaultDirectory();

  /// Returns the platform-default metadata-vault data path.
  String get defaultVaultPath => _operations.vaultDefaultPath();

  /// Writes an encrypted backup of the default metadata vault to [path].
  VaultBackupManifest backupDefaultVault(
    String path, {
    bool overwrite = false,
  }) => _operations.vaultBackupDefault(path, overwrite);

  /// Restores the default metadata vault from encrypted backup [path].
  VaultBackupManifest restoreDefaultVault(
    String path, {
    bool overwrite = false,
  }) => _operations.vaultRestoreDefault(path, overwrite);

  /// Whether the local secret-caching agent is currently reachable.
  bool get agentIsRunning => _operations.vaultIsRunning();

  /// Runs the agent server loop until it is asked to stop.
  void serveAgent() => _operations.vaultAgentServe();

  /// Verifies that the agent transport has the required security properties.
  void verifyAgentTransport() => _operations.vaultAgentVerifyTransport();

  /// Removes every cached secret from the agent.
  void forgetAllAgentSecrets() => _operations.vaultForgetAll();

  /// Requests that the running agent stop.
  void stopAgent() => _operations.vaultAgentStop();

  /// Starts the platform agent service.
  void startAgent() => _operations.vaultAgentStart();

  /// Caches [key] under opaque identifier [id].
  void putAgentKey(Uint8List id, Uint8List key) =>
      _operations.vaultAgentPut(id, key);

  /// Returns the key cached under [id].
  Uint8List getAgentKey(Uint8List id) => _operations.vaultAgentGet(id);

  /// Removes the key cached under [id].
  void forgetAgentKey(Uint8List id) => _operations.vaultAgentForget(id);

  /// Lists the non-secret identifiers currently known to the agent.
  List<AgentEntry> listAgentKeys() => _operations.vaultAgentList();

  /// Reports whether sleep inhibition and suspend notification are supported.
  SleepSupport agentSleepSupport() => _operations.vaultAgentSleepSupport();

  /// Returns the agent log file path, when file logging is enabled.
  String get agentLogPath => _operations.vaultAgentLogPath();

  /// Describes the active agent log destination.
  String get agentLogDestination => _operations.vaultAgentLogDestination();

  /// Caches a vault unlock [key] for [ttlSeconds] seconds.
  void putAgentVaultUnlockKey(String vaultId, Uint8List key, int ttlSeconds) =>
      _operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);

  /// Returns the cached unlock key for [vaultId].
  Uint8List getAgentVaultUnlockKey(String vaultId) =>
      _operations.vaultAgentGetVaultUnlockKey(vaultId);

  /// Removes the cached unlock key for [vaultId].
  void forgetAgentVaultUnlockKey(String vaultId) =>
      _operations.vaultAgentForgetVaultUnlockKey(vaultId);

  /// Caches [key] for a vault [profile] for [ttlSeconds] seconds.
  void putAgentOwnerSigningKey(
    String vaultId,
    String profile,
    SigningKeyPair key,
    int ttlSeconds,
  ) => _operations.vaultAgentPutOwnerSigningKey(
    vaultId,
    profile,
    key._handle,
    ttlSeconds,
  );

  /// Returns the cached owner signing key for a vault [profile].
  SigningKeyPair getAgentOwnerSigningKey(String vaultId, String profile) =>
      SigningKeyPair._(
        this,
        _operations.vaultAgentGetOwnerSigningKey(vaultId, profile),
      );

  /// Removes the cached owner signing key for a vault [profile].
  void forgetAgentOwnerSigningKey(String vaultId, String profile) =>
      _operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);

  /// Registers a secret-using operation of [kind] until the result is disposed.
  AgentActivity beginAgentActivity(String kind) =>
      AgentActivity._(this, _operations.vaultAgentBeginActivity(kind));

  /// Reports availability and configuration of the platform secret store.
  PlatformStatus platformStatus() => _operations.vaultPlatformStatus();

  /// Selects the platform-store namespace identified by [scope].
  void setPlatformScope(String scope) =>
      _operations.vaultPlatformSetScope(scope);

  /// Enables use of the platform secret store.
  void enablePlatformStore() => _operations.vaultPlatformEnable();

  /// Disables use of the platform secret store without deleting its item.
  void disablePlatformStore() => _operations.vaultPlatformDisable();

  /// Whether use of the platform secret store is disabled.
  bool get platformStoreDisabled => _operations.vaultPlatformDisabled();

  /// Stores [password] in the selected platform scope.
  void putPlatformPassword(Uint8List password) =>
      _operations.vaultPlatformPutPassword(password);

  /// Loads the password stored in the selected platform scope.
  Uint8List getPlatformPassword() => _operations.vaultPlatformGetPassword();

  /// Deletes the password stored in the selected platform scope.
  void forgetPlatformPassword() => _operations.vaultPlatformForgetPassword();

  /// Starts a local-vault session for lockboxes addressed by host file paths.
  LocalVault openLocalVault() => LocalVault._(this, _operations.vaultLocal());
}

/// Memory and CPU settings applied when [Vault] creates or opens a [Lockbox].
///
/// The defaults suit interactive applications. Increase the cache or select a
/// parallel worker policy for bulk operations after measuring the host's
/// available memory and CPU capacity.
final class LockboxOptions {
  /// Creates runtime options with conservative interactive defaults.
  const LockboxOptions({
    this.cacheMode = 'bytes',
    this.cacheBytes = 64 << 20,
    this.workload = 'interactive',
    this.worker = 'auto',
    this.jobs = 0,
  });

  /// Decoded-page cache policy: `bytes`, `disabled`, or `automatic`.
  final String cacheMode;

  /// Maximum decoded-page cache capacity in bytes when [cacheMode] is `bytes`.
  final int cacheBytes;

  /// Workload profile, such as `interactive` or `bulk-import`.
  final String workload;

  /// Worker policy: `auto`, `single`, or `threads`.
  final String worker;

  /// Worker count for policies that accept it; zero requests an automatic count.
  final int jobs;
}

abstract base class _Owned {
  _Owned(this.vault, this._handle);
  final Vault vault;
  ffi.Pointer<ffi.Void> _handle;
  bool get disposed => _handle == ffi.nullptr;
}

/// A recipient's shareable encryption identity.
///
/// Import or load this key before granting that recipient lockbox access. It
/// contains no private key material and encrypts content keys that only the
/// matching [ContactKeyPair] can recover.
final class ContactPublicKey extends _Owned {
  ContactPublicKey._(super.vault, super.handle);

  /// Exports this public key using the requested stable [format].
  Uint8List export(String format) =>
      vault._operations.vaultKeyExportPublic(_handle, format);

  /// Returns the canonical fingerprint bytes for this key.
  Uint8List fingerprint() => vault._operations.vaultKeyFingerprint(_handle);

  /// Wraps content [key] so only the corresponding contact can decrypt it.
  WrappedContactKey encrypt(Uint8List key) => WrappedContactKey._(
    vault,
    vault._operations.keyContactEncrypt(_handle, key),
  );

  /// Releases the native key handle; repeated calls have no effect.
  void dispose() {
    if (!disposed) {
      vault._operations.keyContactPublicFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// An encrypted content key addressed to one contact.
///
/// [ContactPublicKey.encrypt] creates this value for storage or transfer with a
/// lockbox access record. Only the matching [ContactKeyPair] can decrypt it.
final class WrappedContactKey extends _Owned {
  WrappedContactKey._(super.vault, super.handle);

  /// Returns the ephemeral public material stored with this envelope.
  Uint8List publicBytes() => vault._operations.keyContactWrappedPublic(_handle);

  /// Returns the encapsulated key ciphertext.
  Uint8List ciphertext() =>
      vault._operations.keyContactWrappedCiphertext(_handle);

  /// Returns the complete serialized encrypted-key envelope.
  Uint8List encryptedBytes() =>
      vault._operations.keyContactWrappedEncrypted(_handle);

  /// Wipes and releases the native envelope handle.
  void dispose() {
    if (!disposed) {
      vault._operations.keyContactWrappedFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// A profile's private and public contact-encryption identity.
///
/// Generate, import, or load this key pair for a profile; distribute its public
/// half to contacts and retain the private half to decrypt content keys they
/// address to that profile.
final class ContactKeyPair extends _Owned {
  ContactKeyPair._(super.vault, super.handle);

  /// Returns the canonical public-key bytes.
  Uint8List publicBytes() => vault._operations.keyContactPublic(_handle);

  /// Returns the private key record for encrypted backup or transfer.
  Uint8List privateRecord() => vault._operations.keyContactPrivate(_handle);

  /// Creates an independently owned public-key handle.
  ContactPublicKey publicKey() =>
      vault.contactPublicKeyFromBytes(publicBytes());

  /// Exports this key pair using the requested stable [format].
  Uint8List export(String format) =>
      vault._operations.vaultKeyExportPrivate(_handle, format);

  /// Decrypts a content key from [wrapped].
  Uint8List decrypt(WrappedContactKey wrapped) =>
      vault._operations.keyContactDecrypt(_handle, wrapped._handle);

  /// Wipes and releases the native private-key handle.
  void dispose() {
    if (!disposed) {
      vault._operations.keyContactFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// The shareable half of a lockbox owner's signing identity.
///
/// Readers use it to verify that a mutable lockbox revision was authorized by
/// the owner. It contains no private signing material.
final class SigningPublicKey extends _Owned {
  SigningPublicKey._(super.vault, super.handle);

  /// Releases the native verification-key handle.
  void dispose() {
    if (!disposed) {
      vault._operations.keySigningPublicFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// A lockbox owner's private and public signing identity.
///
/// Supply it when creating or committing a mutable lockbox so later readers can
/// authenticate revisions with the corresponding [SigningPublicKey].
final class SigningKeyPair extends _Owned {
  SigningKeyPair._(super.vault, super.handle);

  /// Returns the canonical signing public-key bytes.
  Uint8List publicBytes() => vault._operations.keySigningPublic(_handle);

  /// Returns the private signing-key record for secure backup.
  Uint8List privateRecord() => vault._operations.keySigningPrivate(_handle);

  /// Creates an independently owned verification-key handle.
  SigningPublicKey publicKey() =>
      vault.signingPublicKeyFromBytes(publicBytes());

  /// Wipes and releases the native signing-key handle.
  void dispose() {
    if (!disposed) {
      vault._operations.keySigningFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// An open encrypted archive containing files, variables, secrets, and forms.
///
/// Obtain it from [Vault] or [LocalVault]. Reads observe its current contents;
/// mutations remain pending until [commit]. Call [dispose] when finished to
/// release decrypted state.
final class Lockbox extends _Owned {
  Lockbox._(super.vault, super.handle);

  /// Adds [value] at normalized lockbox [path].
  ///
  /// Existing entries require [replace]. [permissions] contains Unix mode bits
  /// when provided. The change becomes durable only after [commit].
  void addFile(
    String path,
    Uint8List value, {
    int? permissions,
    bool replace = false,
  }) {
    if (permissions == null) {
      vault._operations.lockboxAddFile(_handle, path, value, replace);
    } else {
      vault._operations.lockboxAddFileWithPermissions(
        _handle,
        path,
        value,
        permissions,
        replace,
      );
    }
  }

  /// Returns all bytes stored in the regular file at [path].
  Uint8List getFile(String path) =>
      vault._operations.lockboxGetFile(_handle, path);

  /// Extracts one file from [source] to host [destination].
  void extractFile(String source, String destination, {bool replace = false}) =>
      vault._operations.lockboxExtractFile(
        _handle,
        source,
        destination,
        replace,
      );

  /// Safely extracts the lockbox tree to host [destination].
  ///
  /// The three required limits are checked before extraction. Symlinks,
  /// permissions, and overwriting remain disabled unless explicitly enabled.
  void extractDirectory(
    String destination, {
    required int maxFileBytes,
    required int maxTotalBytes,
    required int maxFiles,
    bool restoreSymlinks = false,
    bool restorePermissions = false,
    bool overwrite = false,
  }) => vault._operations.lockboxExtractDirectory(
    _handle,
    destination,
    maxFileBytes,
    maxTotalBytes,
    maxFiles,
    restoreSymlinks,
    restorePermissions,
    overwrite,
  );

  /// Returns logical content chunks, or physical chunks when [physical] is set.
  List<StreamChunk> streamContent({bool physical = false}) =>
      vault._operations.lockboxStreamContent(_handle, physical);

  /// Returns current decoded-page cache counters.
  CacheStats cacheStats() => vault._operations.lockboxCacheStats(_handle);

  /// Returns accumulated file-import timing counters.
  ImportStats importStats() => vault._operations.lockboxImportStats(_handle);

  /// Resets accumulated file-import timing counters.
  void resetImportStats() => vault._operations.lockboxResetImportStats(_handle);

  /// Returns structural metadata for each currently readable page.
  List<PageInspection> pageInspection() =>
      vault._operations.lockboxPageInspection(_handle);

  /// Scans the open lockbox and summarizes recoverable content.
  RecoveryReport recoveryReport() =>
      vault._operations.lockboxRecoveryReport(_handle);

  /// Renders a human-readable recovery report.
  String renderRecoveryReport({bool verbose = false, int maxEntries = 100}) =>
      vault._operations.lockboxRecoveryReportRender(
        _handle,
        verbose,
        maxEntries,
      );

  /// Returns the current serialized storage length in bytes.
  int get storageLength => vault._operations.lockboxStorageLen(_handle);

  /// Selects a stable runtime workload [profile].
  void setWorkloadProfile(String profile) =>
      vault._operations.lockboxSetWorkloadProfile(_handle, profile);

  /// Selects worker [mode] and its requested [jobs] count.
  void setWorkerPolicy(String mode, int jobs) =>
      vault._operations.lockboxSetWorkerPolicy(_handle, mode, jobs);

  /// Returns the effective workload and worker policies.
  RuntimeOptions runtimeOptions() =>
      vault._operations.lockboxRuntimeOptions(_handle);

  /// Authenticates and publishes all staged changes as a new commit.
  void commit() => vault._operations.lockboxCommit(_handle);

  /// Creates a directory at [path], optionally creating missing parents.
  void createDirectory(String path, {bool parents = false}) =>
      vault._operations.lockboxCreateDir(_handle, path, parents);

  /// Deletes the file, symlink, variable, or form record at [path].
  void delete(String path) => vault._operations.lockboxDelete(_handle, path);

  /// Removes a directory, requiring [recursive] when it is not empty.
  void removeDirectory(String path, {bool recursive = false}) =>
      vault._operations.lockboxRemoveDir(_handle, path, recursive);

  /// Creates every missing directory above [path].
  void createParentDirectories(String path) =>
      vault._operations.lockboxCreateParentDirs(_handle, path);

  /// Atomically moves an entry from [from] to [to].
  void rename(String from, String to) =>
      vault._operations.lockboxRename(_handle, from, to);

  /// Lists children below [path], optionally recursively.
  List<LockboxEntry> list(String path, {bool recursive = false}) =>
      vault._operations.lockboxList(_handle, path, recursive);

  /// Lists entries matching [glob] with explicit kind and result filters.
  List<LockboxEntry> listWithOptions(
    String path,
    String glob, {
    bool recursive = false,
    bool includeFiles = true,
    bool includeSymlinks = true,
    bool includeDirectories = true,
    int limit = 0,
  }) => vault._operations.lockboxListWithOptions(
    _handle,
    path,
    glob,
    recursive,
    includeFiles,
    includeSymlinks,
    includeDirectories,
    limit,
  );

  /// Returns metadata for [path], or an absent optional value when missing.
  LockboxEntry? stat(String path) =>
      vault._operations.lockboxStat(_handle, path);

  /// Stores a non-secret UTF-8 variable.
  void setVariable(String name, String value) =>
      vault._operations.lockboxSetVariable(_handle, name, value);

  /// Stores a secret variable without converting it to an immutable string.
  void setSecretVariable(String name, Uint8List value) =>
      vault._operations.lockboxSetSecretVariable(_handle, name, value);

  /// Returns a non-secret variable, or `null` when [name] is absent.
  String? getVariable(String name) =>
      vault._operations.lockboxGetVariable(_handle, name);

  /// Invokes [callback] with temporary secret bytes, then wipes the transfer copy.
  T? withSecretVariable<T>(
    String name,
    T Function(Uint8List secret) callback,
  ) => vault._operations.lockboxWithSecretVariable(_handle, name, callback);

  /// Deletes the variable named [name].
  void deleteVariable(String name) =>
      vault._operations.lockboxDeleteVariable(_handle, name);

  /// Applies all variable [moves] atomically.
  void moveVariables(List<PathMove> moves) => vault._operations
      .lockboxMoveVariables(_handle, DomainDecoders.pathMoves(moves));

  /// Lists variable names and sensitivity without returning secret values.
  List<Variable> listVariables() =>
      vault._operations.lockboxListVariables(_handle);

  /// Returns the sensitivity classification of variable [name].
  String? variableSensitivity(String name) =>
      vault._operations.lockboxVariableSensitivity(_handle, name);

  /// Adds a stored symlink from [path] to lockbox [target].
  void addSymlink(String path, String target, {bool replace = false}) =>
      vault._operations.lockboxAddSymlink(_handle, path, target, replace);

  /// Returns the stored target for the symlink at [path].
  String symlinkTarget(String path) =>
      vault._operations.lockboxGetSymlinkTarget(_handle, path);

  /// Returns the stable binary identifier of this lockbox.
  Uint8List get id => vault._operations.lockboxId(_handle);

  /// Whether any entry exists at [path].
  bool exists(String path) => vault._operations.lockboxExists(_handle, path);

  /// Whether [path] identifies a directory.
  bool isDirectory(String path) =>
      vault._operations.lockboxIsDir(_handle, path);

  /// Returns the stored Unix permission bits for [path].
  int permissions(String path) =>
      vault._operations.lockboxPermissions(_handle, path);

  /// Replaces the stored Unix permission bits for [path].
  void setPermissions(String path, int value) =>
      vault._operations.lockboxSetPermissions(_handle, path, value);

  /// Reads at most [length] bytes at [offset] from the file at [path].
  Uint8List readRange(String path, int offset, int length) =>
      vault._operations.lockboxReadRange(_handle, path, offset, length);

  /// Adds a password access slot and returns its stable slot identifier.
  int addPassword(Uint8List password) =>
      vault._operations.lockboxAddPassword(_handle, password);

  /// Adds a [contact] access slot with local display [name].
  int addContact(ContactPublicKey contact, String name) =>
      vault._operations.lockboxAddContact(_handle, contact._handle, name);

  /// Deletes the access slot identified by [id].
  void deleteKey(int id) => vault._operations.lockboxDeleteKey(_handle, id);

  /// Lists password and contact access slots without exposing key material.
  List<KeySlot> listKeySlots() =>
      vault._operations.lockboxListKeySlots(_handle);

  /// Sets the signing key required to authorize subsequent commits.
  void setOwnerSigningKey(SigningKeyPair key) =>
      vault._operations.lockboxSetOwnerSigningKey(_handle, key._handle);

  /// Reports whether the lockbox is signed and its owner fingerprint.
  OwnerInspection ownerInspection() =>
      vault._operations.lockboxOwnerInspection(_handle);

  /// Defines a new immutable revision of a typed form.
  FormDefinition defineForm(
    String alias,
    String name,
    String description,
    List<FormField> fields,
  ) => vault._operations.lockboxDefineForm(
    _handle,
    alias,
    name,
    description,
    DomainDecoders.formFields(fields),
  );

  /// Lists the current form definitions embedded in this lockbox.
  List<FormDefinition> listFormDefinitions() =>
      vault._operations.lockboxListFormDefinitions(_handle);

  /// Resolves a form alias or stable type identifier.
  FormDefinition resolveForm(String reference) =>
      vault._operations.lockboxResolveForm(_handle, reference);

  /// Lists every stored revision for [typeId].
  List<FormDefinition> listFormRevisions(String typeId) =>
      vault._operations.lockboxListFormRevisions(_handle, typeId);

  /// Creates a form record at [path] using [typeReference].
  FormRecord createFormRecord(String path, String typeReference, String name) =>
      vault._operations.lockboxCreateFormRecord(
        _handle,
        path,
        typeReference,
        name,
      );

  /// Stores a non-secret [value] in a form [field].
  void setFormField(String path, String field, String value) =>
      vault._operations.lockboxSetFormField(_handle, path, field, value);

  /// Stores a secret form field without creating an immutable string.
  void setSecretFormField(String path, String field, Uint8List value) =>
      vault._operations.lockboxSetSecretFormField(_handle, path, field, value);

  /// Lists all typed form records without returning secret field values.
  List<FormRecord> listFormRecords() =>
      vault._operations.lockboxListFormRecords(_handle);

  /// Returns the form record at [path], when present.
  FormRecord? getFormRecord(String path) =>
      vault._operations.lockboxGetFormRecord(_handle, path);

  /// Deletes the form record at [path].
  void deleteFormRecord(String path) =>
      vault._operations.lockboxDeleteFormRecord(_handle, path);

  /// Applies all form-record [moves] atomically.
  void moveFormRecords(List<PathMove> moves) => vault._operations
      .lockboxMoveFormRecords(_handle, DomainDecoders.pathMoves(moves));

  /// Returns a non-secret form [field], or an absent optional value.
  FormValue? getFormField(String path, String field) =>
      vault._operations.lockboxGetFormField(_handle, path, field);

  /// Invokes [callback] with temporary field bytes, then wipes the transfer copy.
  T? withSecretFormField<T>(
    String path,
    String field,
    T Function(Uint8List secret) callback,
  ) => vault._operations.lockboxWithSecretFormField(
    _handle,
    path,
    field,
    callback,
  );

  /// Serializes the current committed lockbox state.
  Uint8List get bytes => vault._operations.lockboxToBytes(_handle);

  /// Wipes cached secret state and releases the native lockbox handle.
  void dispose() {
    if (!disposed) {
      vault._operations.lockboxFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// A writable, password-protected metadata store for one reVault installation.
///
/// Open or create it through [Vault] to manage profile keys, contacts, form
/// definitions, encrypted key backups, remembered lockbox paths, and local
/// access-slot labels. Lockbox file contents are stored separately.
final class VaultDirectory extends _Owned {
  VaultDirectory._(super.vault, super.handle);

  /// Returns the canonical root directory of this metadata vault.
  String get root => vault._operations.vaultDirectoryRoot(_handle);

  /// Returns the on-disk metadata-vault structure version.
  int get structureVersion =>
      vault._operations.vaultDirectoryStructureVersion(_handle);

  /// Lists serialized private-key records for backup workflows.
  List<String> listPrivateKeys() =>
      vault._operations.vaultDirectoryListPrivateKeys(_handle);

  /// Lists profile names that have private contact keys.
  List<String> listPrivateKeyNames() =>
      vault._operations.vaultDirectoryListPrivateKeyNames(_handle);

  /// Lists stored contact names.
  List<String> listContactNames() =>
      vault._operations.vaultDirectoryListContactNames(_handle);

  /// Lists stored form aliases.
  List<String> listFormAliases() =>
      vault._operations.vaultDirectoryListFormAliases(_handle);

  /// Whether a profile private key exists under [name].
  bool privateKeyExists(String name) =>
      vault._operations.vaultDirectoryPrivateKeyExists(_handle, name);

  /// Deletes the active private key and its signing key for [name].
  void deletePrivateKey(String name) =>
      vault._operations.vaultDirectoryDeletePrivateKey(_handle, name);

  /// Stores [key] as the active private contact key for [name].
  void storePrivateKey(String name, ContactKeyPair key) => vault._operations
      .vaultDirectoryStorePrivateKey(_handle, name, key._handle);

  /// Loads the active private contact key for [name].
  ContactKeyPair loadPrivateKey(String name) => ContactKeyPair._(
    vault,
    vault._operations.vaultDirectoryLoadPrivateKey(_handle, name),
  );

  /// Loads historical private-key generation [index] for [name].
  ContactKeyPair loadPrivateKeyGeneration(String name, int index) =>
      ContactKeyPair._(
        vault,
        vault._operations.vaultDirectoryLoadPrivateKeyGeneration(
          _handle,
          name,
          index,
        ),
      );

  /// Stores [key] as the public key for contact [name].
  void storeContact(String name, ContactPublicKey key) =>
      vault._operations.vaultDirectoryStoreContact(_handle, name, key._handle);

  /// Loads the public key for contact [name].
  ContactPublicKey loadContact(String name) => ContactPublicKey._(
    vault,
    vault._operations.vaultDirectoryLoadContact(_handle, name),
  );

  /// Whether contact [name] exists.
  bool contactExists(String name) =>
      vault._operations.vaultDirectoryContactExists(_handle, name);

  /// Deletes contact [name].
  void deleteContact(String name) =>
      vault._operations.vaultDirectoryDeleteContact(_handle, name);

  /// Lists stored contacts and their public-key records.
  List<Contact> listContacts() =>
      vault._operations.vaultDirectoryListContacts(_handle);

  /// Stores the contact [email] associated with profile [name].
  void storeProfileEmail(String name, String email) =>
      vault._operations.vaultDirectoryStoreProfileEmail(_handle, name, email);

  /// Returns the email associated with profile [name], when present.
  String? profileEmail(String name) =>
      vault._operations.vaultDirectoryProfileEmail(_handle, name);

  /// Stores encrypted key-directory backup [value] under lockbox [id].
  void storeBackup(Uint8List id, Uint8List value) =>
      vault._operations.vaultDirectoryStoreBackup(_handle, id, value);

  /// Loads the encrypted key-directory backup for lockbox [id].
  Uint8List loadBackup(Uint8List id) =>
      vault._operations.vaultDirectoryLoadBackup(_handle, id);

  /// Returns the number of stored key-directory backups.
  int get backupCount => vault._operations.vaultDirectoryBackupCount(_handle);

  /// Restores a profile key pair and its owner [signing] key.
  void restorePrivateKey(
    String name,
    ContactKeyPair key,
    SigningKeyPair signing, {
    bool overwrite = false,
  }) => vault._operations.vaultDirectoryRestorePrivateKey(
    _handle,
    name,
    key._handle,
    signing._handle,
    overwrite,
  );

  /// Loads the active owner signing key for profile [name].
  SigningKeyPair loadOwnerSigningKey(String name) => SigningKeyPair._(
    vault,
    vault._operations.vaultDirectoryLoadOwnerSigningKey(_handle, name),
  );

  /// Loads historical owner-signing generation [index] for [name].
  SigningKeyPair loadOwnerSigningKeyGeneration(String name, int index) =>
      SigningKeyPair._(
        vault,
        vault._operations.vaultDirectoryLoadOwnerSigningKeyGeneration(
          _handle,
          name,
          index,
        ),
      );

  /// Stores a contact's signing public [key] under [name].
  void storeContactSigningKey(String name, SigningPublicKey key) => vault
      ._operations
      .vaultDirectoryStoreContactSigningKey(_handle, name, key._handle);

  /// Loads the signing public key stored for contact [name].
  SigningPublicKey loadContactSigningKey(String name) => SigningPublicKey._(
    vault,
    vault._operations.vaultDirectoryLoadContactSigningKey(_handle, name),
  );

  /// Lists active and retired key generations for profile [name].
  ProfileHistory listProfileGenerations(String name) =>
      vault._operations.vaultDirectoryListProfileGenerations(_handle, name);

  /// Rotates profile [name] to newly generated contact and signing keys.
  ProfileHistory rotatePrivateKey(String name) =>
      vault._operations.vaultDirectoryRotatePrivateKey(_handle, name);

  /// Remembers that lockbox [id] is stored at host [path].
  void rememberLockbox(Uint8List id, String path) =>
      vault._operations.vaultDirectoryRememberLockbox(_handle, id, path);

  /// Lists remembered lockbox identifiers and host paths.
  List<KnownLockbox> listKnownLockboxes() =>
      vault._operations.vaultDirectoryListKnownLockboxes(_handle);

  /// Forgets the lockbox remembered at host [path].
  void forgetLockbox(String path) =>
      vault._operations.vaultDirectoryForgetLockbox(_handle, path);

  /// Stores local display [name] for one lockbox access slot.
  void rememberAccessSlotLabel(Uint8List id, int slotId, String name) => vault
      ._operations
      .vaultDirectoryRememberAccessSlotLabel(_handle, id, slotId, name);

  /// Lists local access-slot labels for lockbox [id].
  List<AccessSlotLabel> listAccessSlotLabels(Uint8List id) =>
      vault._operations.vaultDirectoryListAccessSlotLabels(_handle, id);

  /// Finds labels named [name] for lockbox [id].
  List<AccessSlotLabel> findAccessSlotLabels(Uint8List id, String name) =>
      vault._operations.vaultDirectoryFindAccessSlotLabels(_handle, id, name);

  /// Deletes the local label for one lockbox access slot.
  void forgetAccessSlotLabel(Uint8List id, int slotId) => vault._operations
      .vaultDirectoryForgetAccessSlotLabel(_handle, id, slotId);

  /// Defines a new immutable revision of a vault-wide form.
  FormDefinition defineForm(
    String alias,
    String name,
    String description,
    List<FormField> fields,
  ) => vault._operations.vaultDirectoryDefineForm(
    _handle,
    alias,
    name,
    description,
    DomainDecoders.formFields(fields),
  );

  /// Resolves a vault-wide form alias or stable type identifier.
  FormDefinition resolveForm(String reference) =>
      vault._operations.vaultDirectoryResolveForm(_handle, reference);

  /// Lists current vault-wide form definitions.
  List<FormDefinition> listForms() =>
      vault._operations.vaultDirectoryListForms(_handle);

  /// Lists every vault-wide revision for [typeId].
  List<FormDefinition> listFormRevisions(String typeId) =>
      vault._operations.vaultDirectoryListFormRevisions(_handle, typeId);

  /// Installs any built-in form definitions that are not already present.
  int seedForms() => vault._operations.vaultDirectorySeedForms(_handle);

  /// Remembers a lockbox [password] under lockbox [id].
  void rememberPassword(Uint8List id, Uint8List password) =>
      vault._operations.vaultDirectoryRememberPassword(_handle, id, password);

  /// Returns the password remembered for lockbox [id].
  Uint8List rememberedPassword(Uint8List id) =>
      vault._operations.vaultDirectoryRememberedPassword(_handle, id);

  /// Wipes decrypted state and releases the writable vault handle.
  void dispose() {
    if (!disposed) {
      vault._operations.vaultDirectoryFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// A restricted view of a local metadata store for discovery and listing.
///
/// Use it when an application needs profile names, contacts, forms, or known
/// lockbox paths without gaining access to owner signing keys or mutation APIs.
final class ReadOnlyVaultDirectory extends _Owned {
  ReadOnlyVaultDirectory._(super.vault, super.handle);

  /// Lists profile names without loading private or signing keys.
  List<String> listProfileNames() =>
      vault._operations.vaultReadOnlyListProfileNames(_handle);

  /// Lists contact names without loading contact key material.
  List<String> listContactNames() =>
      vault._operations.vaultReadOnlyListContactNames(_handle);

  /// Lists vault-wide form aliases.
  List<String> listFormAliases() =>
      vault._operations.vaultReadOnlyListFormAliases(_handle);

  /// Lists remembered lockbox identifiers and host paths.
  List<KnownLockbox> listKnownLockboxes() =>
      vault._operations.vaultReadOnlyListKnownLockboxes(_handle);

  /// Wipes decrypted metadata and releases the read-only handle.
  void dispose() {
    if (!disposed) {
      vault._operations.vaultReadOnlyFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// A lifetime token for an operation that currently needs cached secrets.
///
/// Keep the token returned by the agent activity API alive for the duration of
/// the operation, then call [dispose] so the session agent can expire secrets
/// when no other activity needs them.
final class AgentActivity extends _Owned {
  AgentActivity._(super.vault, super.handle);

  /// Ends the registered activity; repeated calls have no effect.
  void dispose() {
    if (!disposed) {
      vault._operations.vaultAgentEndActivity(_handle);
      _handle = ffi.nullptr;
    }
  }
}

/// A session that manages lockbox files used by a local application.
///
/// Obtain it from [Vault.openLocalVault] to create or open lockboxes by host
/// path, cache short-lived passwords, and close individual or all open files.
/// Call [dispose] when the application session ends.
final class LocalVault extends _Owned {
  LocalVault._(super.vault, super.handle);

  /// Creates and remembers a lockbox at [path] protected by [password].
  Lockbox createWithPassword(String path, Uint8List password) => Lockbox._(
    vault,
    vault._operations.vaultCreateLockboxPassword(_handle, path, password),
  );

  /// Opens and remembers the password-protected lockbox at [path].
  Lockbox openWithPassword(String path, Uint8List password) => Lockbox._(
    vault,
    vault._operations.vaultOpenLockboxPassword(_handle, path, password),
  );

  /// Creates a signed lockbox at [path] using a content [key].
  Lockbox createWithContentKey(
    String path,
    Uint8List key,
    SigningKeyPair signing,
  ) => Lockbox._(
    vault,
    vault._operations.vaultCreateLockboxContentKey(
      _handle,
      path,
      key,
      signing._handle,
    ),
  );

  /// Opens the signed lockbox at [path] using a content [key].
  Lockbox openWithContentKey(
    String path,
    Uint8List key,
    SigningKeyPair signing,
  ) => Lockbox._(
    vault,
    vault._operations.vaultOpenLockboxContentKey(
      _handle,
      path,
      key,
      signing._handle,
    ),
  );

  /// Creates a signed lockbox at [path] and grants [contact] access.
  Lockbox createForContact(
    String path,
    ContactPublicKey contact,
    String name,
    SigningKeyPair signing,
  ) => Lockbox._(
    vault,
    vault._operations.vaultCreateLockboxContact(
      _handle,
      path,
      contact._handle,
      name,
      signing._handle,
    ),
  );

  /// Caches [password] for lockbox [path] for [ttlSeconds] seconds.
  void cachePassword(String path, Uint8List password, int ttlSeconds) => vault
      ._operations
      .vaultCacheLockboxPassword(_handle, path, password, ttlSeconds);

  /// Commits, closes, and forgets the open lockbox at [path].
  void closeLockbox(String path) =>
      vault._operations.vaultCloseLockbox(_handle, path);

  /// Commits and closes every lockbox opened by this local-vault session.
  void closeAll() => vault._operations.vaultCloseAll(_handle);

  /// Closes open lockboxes and ends this local-vault session.
  void dispose() {
    if (!disposed) {
      vault._operations.vaultFree(_handle);
      _handle = ffi.nullptr;
    }
  }
}
