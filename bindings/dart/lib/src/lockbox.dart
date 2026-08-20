import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:typed_data';

import 'package:revault_api/src/agent_session.dart';
import 'package:revault_api/src/contact_key_pair.dart';
import 'package:revault_api/src/contact_public_key.dart';
import 'package:revault_api/src/domain_models.dart';
import 'package:revault_api/src/exceptions.dart';
import 'package:revault_api/src/lockbox_options.dart';
import 'package:revault_api/src/lockbox_worker.dart';
import 'package:revault_api/src/lockbox_workload.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/profile_signing_key_pair.dart';
import 'package:revault_api/src/revault.dart';
import 'package:revault_api/src/secret_bytes.dart';
import 'package:revault_api/src/secret_string.dart';
import 'package:revault_api/src/vault.dart';

const _lockboxDescriptionVariable = '/.revault/description';

/// An open encrypted archive containing files, variables, secrets, and forms.
///
/// Create or open it with the static factories on [Lockbox]. Reads observe its
/// current contents; mutations remain pending until [commit]. A file-backed
/// Lockbox writes its committed bytes back to its path. Call [close] when
/// finished to release decrypted state.
///
/// Example:
/// ```dart
/// final password = SecretString.fromString(promptedPassword);
/// final lockbox = Lockbox.open('/secrets/team.lbox', password: password);
/// try {
///   print(lockbox.list('/', recursive: true));
/// } finally {
///   lockbox.close();
///   password.close();
/// }
/// ```
final class Lockbox extends Owned implements ffi.Finalizable {
  /// @nodoc
  Lockbox.internal(super.runtime, super.handle, {String? backingPath})
    : _backingPath = backingPath {
    final finalizer = _nativeFinalizer ??= ffi.NativeFinalizer(
      runtime.operations.lockboxFreeAddress,
    );
    finalizer.attach(this, handle, detach: this);
  }

  static ffi.NativeFinalizer? _nativeFinalizer;

  String? _backingPath;

  /// Creates an in-memory lockbox.
  ///
  /// Supply exactly one of [password], [contentKey], or [contact]. Use
  /// [create] when the Lockbox should be backed by a host file. This variant is
  /// useful for database, network, or test workflows that own serialized bytes.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString('temporary test password');
  /// final box = Lockbox.createInMemory(password: password);
  /// try {
  ///   box.addFile('/hello.txt', Uint8List.fromList(utf8.encode('hello')));
  ///   box.commit();
  ///   await upload(box.bytes);
  /// } finally {
  ///   box.close();
  ///   password.close();
  /// }
  /// ```
  static Lockbox createInMemory({
    SecretString? password,
    SecretBytes? contentKey,
    ContactPublicKey? contact,
    ProfileSigningKeyPair? signingKey,
    LockboxOptions? options,
  }) {
    _requireOneCredential(password, contentKey, contact);
    final runtime = Revault.runtime;
    late final Lockbox lockbox;
    if (password != null) {
      if (options != null) {
        throw UnsupportedError(
          'LockboxOptions are currently supported only with a content key.',
        );
      }
      lockbox = runtime.createLockboxWithPasswordInternal(password);
    } else if (contact != null) {
      if (options != null) {
        throw UnsupportedError(
          'LockboxOptions are currently supported only with a content key.',
        );
      }
      lockbox = runtime.createLockboxForContactInternal(contact);
    } else {
      lockbox = runtime.createLockboxInternal(contentKey!, options);
    }
    if (signingKey != null) lockbox.setOwnerSigningKey(signingKey);
    return lockbox;
  }

  /// Opens serialized [archive] bytes without using the session agent.
  ///
  /// Supply exactly one of [password], [contentKey], or [contact]. Use [open]
  /// instead when a host file should be updated by [commit].
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString(promptedPassword);
  /// final box = Lockbox.openBytes(downloadedBytes, password: password);
  /// try {
  ///   print(utf8.decode(box.getFile('/message.txt')));
  /// } finally {
  ///   box.close();
  ///   password.close();
  /// }
  /// ```
  static Lockbox openBytes(
    Uint8List archive, {
    SecretString? password,
    SecretBytes? contentKey,
    ContactKeyPair? contact,
    LockboxOptions? options,
  }) {
    final count = [password, contentKey, contact].whereType<Object>().length;
    if (count != 1) {
      throw ArgumentError(
        'Supply exactly one of password, contentKey, or contact.',
      );
    }
    final runtime = Revault.runtime;
    if (contentKey != null) {
      return runtime.openLockboxInternal(archive, contentKey, options);
    }
    if (options != null) {
      throw UnsupportedError(
        'LockboxOptions are currently supported only with a content key.',
      );
    }
    return password != null
        ? runtime.openLockboxWithPasswordInternal(archive, password)
        : runtime.openLockboxForContactInternal(archive, contact!);
  }

  /// Creates a lockbox file at [path].
  ///
  /// Supply exactly one of [password], [contentKey], or [contact]. A password is
  /// the lockbox-specific secret used by a password access slot; it is not the
  /// vault passphrase. Existing files require [overwrite]. Call [commit] after
  /// mutations and [close] when finished.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString(promptedPassword);
  /// final box = Lockbox.create('/secrets/team.lbox', password: password);
  /// try {
  ///   box.addFile('/README.txt', readmeBytes);
  ///   box.commit();
  /// } finally {
  ///   box.close();
  ///   password.close();
  /// }
  /// ```
  static Lockbox create(
    String path, {
    SecretString? password,
    SecretBytes? contentKey,
    ContactPublicKey? contact,
    ProfileSigningKeyPair? signingKey,
    LockboxOptions? options,
    bool overwrite = false,
  }) {
    _requireOneCredential(password, contentKey, contact);
    final file = File(path);
    if (file.existsSync() && !overwrite) {
      throw FileSystemException('Lockbox already exists', path);
    }
    final lockbox = createInMemory(
      password: password,
      contentKey: contentKey,
      contact: contact,
      signingKey: signingKey,
      options: options,
    );
    lockbox._backingPath = path;
    file.writeAsBytesSync(lockbox.bytes, flush: true);
    return lockbox;
  }

  /// Opens the lockbox file at [path] without using the session agent.
  ///
  /// Supply one explicit credential or an open [vault]. With no credential,
  /// reVault opens the default Vault using its platform-stored passphrase and
  /// asks that Vault for a remembered lockbox password or matching profile
  /// key. It never consults or starts the session agent.
  ///
  /// The decrypted content key remains only in this process and is wiped by
  /// [close].
  ///
  /// Example:
  /// ```dart
  /// final vault = Vault.open();
  /// final box = Lockbox.open('/secrets/team.lbox', vault: vault);
  /// try {
  ///   print(box.list('/', recursive: true));
  /// } finally {
  ///   box.close();
  ///   vault.close();
  /// }
  /// ```
  static Lockbox open(
    String path, {
    Vault? vault,
    SecretString? password,
    SecretBytes? contentKey,
    ContactKeyPair? contact,
    LockboxOptions? options,
  }) {
    final explicit = [password, contentKey, contact].whereType<Object>().length;
    if (explicit > 1 || (explicit == 1 && vault != null)) {
      throw ArgumentError(
        'Supply exactly one explicit credential or a vault, not both.',
      );
    }
    final archive = Uint8List.fromList(File(path).readAsBytesSync());
    late final Lockbox opened;
    if (contentKey != null) {
      opened = openBytes(archive, contentKey: contentKey, options: options);
    } else if (password != null) {
      if (options != null) {
        throw UnsupportedError(
          'LockboxOptions are currently supported only with a content key.',
        );
      }
      opened = openBytes(archive, password: password);
    } else if (contact != null) {
      if (options != null) {
        throw UnsupportedError(
          'LockboxOptions are currently supported only with a content key.',
        );
      }
      opened = openBytes(archive, contact: contact);
    } else if (vault != null) {
      opened = _openUsingVault(path, archive, vault);
    } else {
      Vault? defaultVault;
      try {
        defaultVault = Vault.open();
        opened = _openUsingVault(path, archive, defaultVault);
      } on VaultPassphraseUnavailableException {
        throw LockboxCredentialUnavailableException(path);
      } finally {
        defaultVault?.close();
      }
    }
    opened._backingPath = path;
    return opened;
  }

  /// Opens [path] using a content key already cached by [agentSession].
  ///
  /// This does not extend the agent entry's expiry. The returned lockbox owns
  /// an independent in-process copy until [close].
  ///
  /// Example:
  /// ```dart
  /// final box = AgentSession.instance.acquireOpenLockbox(
  ///   '/secrets/team.lbox',
  /// );
  /// try {
  ///   print(box.list('/', recursive: true));
  /// } finally {
  ///   box.close();
  /// }
  /// ```
  @Deprecated('Use AgentSession.acquireOpenLockbox().')
  static Lockbox openFromAgent(
    String path, {
    AgentSession? agentSession,
    LockboxOptions? options,
  }) {
    return (agentSession ?? AgentSession.instance).acquireOpenLockbox(
      path,
      options: options,
    );
  }

  static Lockbox _openUsingVault(
    String path,
    Uint8List archive,
    Vault persistentVault,
  ) {
    final runtime = Revault.runtime;
    final lockboxId = runtime.inspectLockboxFile(path).lockboxId;
    try {
      final remembered = persistentVault.rememberedPassword(lockboxId);
      try {
        final opened = runtime.openLockboxWithPasswordInternal(
          archive,
          remembered,
        );
        _attachDefaultSigningKey(opened, persistentVault);
        return opened;
      } finally {
        remembered.close();
      }
    } on RevaultException {
      // This vault may instead hold a matching private profile key.
    }
    for (final profile in persistentVault.listPrivateKeyNames()) {
      final key = persistentVault.loadPrivateKey(profile);
      try {
        final opened = runtime.openLockboxForContactInternal(archive, key);
        try {
          final signing = persistentVault.loadProfileSigningKey(profile);
          try {
            opened.setOwnerSigningKey(signing);
          } finally {
            signing.dispose();
          }
        } on RevaultException {
          // Read-only use remains possible when this profile has no signing key.
        }
        return opened;
      } on RevaultException {
        // Try the next profile stored in the vault.
      } finally {
        key.dispose();
      }
    }
    throw LockboxCredentialUnavailableException(path);
  }

  static void _attachDefaultSigningKey(Lockbox opened, Vault persistentVault) {
    try {
      final signing = persistentVault.loadProfileSigningKey('default');
      try {
        opened.setOwnerSigningKey(signing);
      } finally {
        signing.dispose();
      }
    } on RevaultException {
      // Password-only and read-only lockboxes do not require a signing key.
    }
  }

  static void _requireOneCredential(
    SecretString? password,
    SecretBytes? contentKey,
    ContactPublicKey? contact,
  ) {
    if ([password, contentKey, contact].whereType<Object>().length != 1) {
      throw ArgumentError(
        'Supply exactly one of password, contentKey, or contact.',
      );
    }
  }

  /// Adds [fileBytes] at normalized lockbox [path].
  ///
  /// Existing entries require [replace]. [permissions] contains Unix mode bits
  /// when provided. The change becomes durable only after [commit].
  ///
  /// Example:
  /// ```dart
  /// lockbox.addFile('/docs/readme.txt', readmeBytes, permissions: 0x180);
  /// lockbox.commit();
  /// ```
  void addFile(
    String path,
    Uint8List fileBytes, {
    int? permissions,
    bool replace = false,
  }) {
    if (permissions == null) {
      runtime.operations.lockboxAddFile(handle, path, fileBytes, replace);
    } else {
      runtime.operations.lockboxAddFileWithPermissions(
        handle,
        path,
        fileBytes,
        permissions,
        replace,
      );
    }
  }

  /// Returns all bytes stored in the regular file at [path].
  ///
  /// Use [readRange] for a large file when only part of it is needed.
  ///
  /// Example:
  /// ```dart
  /// final text = utf8.decode(lockbox.getFile('/docs/readme.txt'));
  /// ```
  Uint8List getFile(String path) =>
      runtime.operations.lockboxGetFile(handle, path);

  /// Extracts one Lockbox file from [source] to host [destination].
  ///
  /// Existing host files are protected unless [replace] is true.
  ///
  /// Example:
  /// ```dart
  /// lockbox.extractFile('/docs/readme.txt', '/tmp/readme.txt');
  /// ```
  void extractFile(String source, String destination, {bool replace = false}) =>
      runtime.operations.lockboxExtractFile(
        handle,
        source,
        destination,
        replace,
      );

  /// Safely extracts the lockbox tree to host [destination].
  ///
  /// The three required limits are checked before extraction. Symlinks,
  /// permissions, and overwriting remain disabled unless explicitly enabled.
  ///
  /// Example:
  /// ```dart
  /// lockbox.extractDirectory(
  ///   '/tmp/restored',
  ///   maxFileBytes: 10 << 20,
  ///   maxTotalBytes: 100 << 20,
  ///   maxFiles: 1000,
  /// );
  /// ```
  void extractDirectory(
    String destination, {
    required int maxFileBytes,
    required int maxTotalBytes,
    required int maxFiles,
    bool restoreSymlinks = false,
    bool restorePermissions = false,
    bool overwrite = false,
  }) => runtime.operations.lockboxExtractDirectory(
    handle,
    destination,
    maxFileBytes,
    maxTotalBytes,
    maxFiles,
    restoreSymlinks,
    restorePermissions,
    overwrite,
  );

  /// Returns logical content extents, or storage extents when [physical] is true.
  ///
  /// Use this for diagnostics, progress planning, or range-based streaming;
  /// use [getFile] when the application needs the complete file content.
  ///
  /// Example:
  /// ```dart
  /// for (final chunk in lockbox.streamContent()) {
  ///   print('${chunk.path}: ${chunk.length} bytes');
  /// }
  /// ```
  List<StreamChunk> streamContent({bool physical = false}) =>
      runtime.operations.lockboxStreamContent(handle, physical);

  /// Returns current decoded-page cache occupancy and hit counters.
  ///
  /// Use these measurements when tuning [LockboxOptions.cacheBytes].
  ///
  /// Example:
  /// ```dart
  /// final stats = lockbox.cacheStats();
  /// print('${stats.hits} hits, ${stats.misses} misses');
  /// ```
  CacheStats cacheStats() => runtime.operations.lockboxCacheStats(handle);

  /// Returns accumulated timings for host-file import work.
  ///
  /// Example:
  /// ```dart
  /// final stats = lockbox.importStats();
  /// print('Host reads: ${stats.hostReadNanos} ns');
  /// ```
  ImportStats importStats() => runtime.operations.lockboxImportStats(handle);

  /// Resets accumulated import timings before measuring another operation.
  ///
  /// Example:
  /// ```dart
  /// lockbox.resetImportStats();
  /// lockbox.addFile('/large.bin', bytes);
  /// print(lockbox.importStats().pageWriteNanos);
  /// ```
  void resetImportStats() => runtime.operations.lockboxResetImportStats(handle);

  /// Returns structural metadata for each currently readable encrypted page.
  ///
  /// This is intended for diagnostics and storage analysis, not normal content
  /// traversal.
  ///
  /// Example:
  /// ```dart
  /// for (final page in lockbox.pageInspection()) {
  ///   print('page ${page.pageId}: ${page.unusedBytes} unused bytes');
  /// }
  /// ```
  List<PageInspection> pageInspection() =>
      runtime.operations.lockboxPageInspection(handle);

  /// Scans the open Lockbox and summarizes recoverable content.
  ///
  /// Use it after an integrity concern before deciding whether to salvage.
  ///
  /// Example:
  /// ```dart
  /// final report = lockbox.recoveryReport();
  /// if (report.corruptRecords > 0) showRecoveryWarning(report);
  /// ```
  RecoveryReport recoveryReport() =>
      runtime.operations.lockboxRecoveryReport(handle);

  /// Renders [recoveryReport] as text suitable for logs or support output.
  ///
  /// Example:
  /// ```dart
  /// print(lockbox.renderRecoveryReport(verbose: true, maxEntries: 50));
  /// ```
  String renderRecoveryReport({bool verbose = false, int maxEntries = 100}) =>
      runtime.operations.lockboxRecoveryReportRender(
        handle,
        verbose,
        maxEntries,
      );

  /// Returns the current serialized archive length in bytes.
  ///
  /// Example:
  /// ```dart
  /// print('Encrypted archive size: ${lockbox.storageLength} bytes');
  /// ```
  int get storageLength => runtime.operations.lockboxStorageLen(handle);

  /// Changes tuning for the expected workload [profile].
  ///
  /// This affects runtime behavior, not encrypted data or future openings.
  ///
  /// Example:
  /// ```dart
  /// lockbox.setWorkloadProfile(LockboxWorkload.bulkImport);
  /// ```
  void setWorkloadProfile(LockboxWorkload profile) =>
      runtime.operations.lockboxSetWorkloadProfile(handle, profile.nativeName);

  /// Selects worker [mode] and its requested [jobs] count for this handle.
  ///
  /// Example:
  /// ```dart
  /// lockbox.setWorkerPolicy(LockboxWorker.threads, jobs: 4);
  /// ```
  void setWorkerPolicy(LockboxWorker mode, {int jobs = 0}) =>
      runtime.operations.lockboxSetWorkerPolicy(handle, mode.nativeName, jobs);

  /// Returns the workload and worker policies active on this handle.
  ///
  /// Example:
  /// ```dart
  /// final options = lockbox.runtimeOptions();
  /// print('${options.workloadProfile}/${options.workerPolicy}');
  /// ```
  RuntimeOptions runtimeOptions() =>
      runtime.operations.lockboxRuntimeOptions(handle);

  /// Authenticates and publishes all staged changes as a new revision.
  ///
  /// For a file-backed Lockbox, this also writes the serialized revision to its
  /// host path. It does not close the Lockbox.
  ///
  /// Example:
  /// ```dart
  /// lockbox.setVariable('environment', 'production');
  /// lockbox.commit();
  /// ```
  void commit() {
    runtime.operations.lockboxCommit(handle);
    final path = _backingPath;
    if (path != null) {
      File(path).writeAsBytesSync(bytes, flush: true);
    }
  }

  /// Creates a directory at [path], optionally creating missing parents.
  ///
  /// Example:
  /// ```dart
  /// lockbox.createDirectory('/projects/acme', parents: true);
  /// ```
  void createDirectory(String path, {bool parents = false}) =>
      runtime.operations.lockboxCreateDir(handle, path, parents);

  /// Deletes the file, symlink, variable, or form record at [path].
  ///
  /// Use [removeDirectory] for directories. Call [commit] to publish deletion.
  ///
  /// Example:
  /// ```dart
  /// lockbox.delete('/obsolete.txt');
  /// lockbox.commit();
  /// ```
  void delete(String path) => runtime.operations.lockboxDelete(handle, path);

  /// Removes a directory, requiring [recursive] when it is not empty.
  ///
  /// Example:
  /// ```dart
  /// lockbox.removeDirectory('/old-project', recursive: true);
  /// ```
  void removeDirectory(String path, {bool recursive = false}) =>
      runtime.operations.lockboxRemoveDir(handle, path, recursive);

  /// Creates every missing directory above a future entry [path].
  ///
  /// The final path component is not created.
  ///
  /// Example:
  /// ```dart
  /// lockbox.createParentDirectories('/projects/acme/config.json');
  /// ```
  void createParentDirectories(String path) =>
      runtime.operations.lockboxCreateParentDirs(handle, path);

  /// Stages an atomic move of an entry from [from] to [to].
  ///
  /// Example:
  /// ```dart
  /// lockbox.rename('/draft.txt', '/published.txt');
  /// lockbox.commit();
  /// ```
  void rename(String from, String to) =>
      runtime.operations.lockboxRename(handle, from, to);

  /// Lists children below [path], optionally recursively.
  ///
  /// This returns metadata only; use [getFile] to read file content.
  ///
  /// Example:
  /// ```dart
  /// for (final entry in lockbox.list('/', recursive: true)) {
  ///   print('${entry.kind}: ${entry.path}');
  /// }
  /// ```
  List<LockboxEntry> list(String path, {bool recursive = false}) =>
      runtime.operations.lockboxList(handle, path, recursive);

  /// Lists entries matching [glob] with explicit kind and result filters.
  ///
  /// A zero [limit] means no explicit result limit.
  ///
  /// Example:
  /// ```dart
  /// final markdown = lockbox.listWithOptions(
  ///   '/docs',
  ///   '**/*.md',
  ///   recursive: true,
  ///   includeDirectories: false,
  ///   limit: 100,
  /// );
  /// ```
  List<LockboxEntry> listWithOptions(
    String path,
    String glob, {
    bool recursive = false,
    bool includeFiles = true,
    bool includeSymlinks = true,
    bool includeDirectories = true,
    int limit = 0,
  }) => runtime.operations.lockboxListWithOptions(
    handle,
    path,
    glob,
    recursive,
    includeFiles,
    includeSymlinks,
    includeDirectories,
    limit,
  );

  /// Returns metadata for [path], or `null` when it does not exist.
  ///
  /// Example:
  /// ```dart
  /// final entry = lockbox.stat('/docs/readme.txt');
  /// if (entry != null) print('${entry.length} bytes');
  /// ```
  LockboxEntry? stat(String path) =>
      runtime.operations.lockboxStat(handle, path);

  /// Returns this Lockbox's encrypted human-readable description.
  ///
  /// The description is UTF-8 text stored inside the encrypted Lockbox rather
  /// than its public header. It is therefore available only after [open] and
  /// is `null` when no description has been assigned.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString(promptedPassword);
  /// final lockbox = Lockbox.open('/backups/production.lbox', password: password);
  /// try {
  ///   print(lockbox.description ?? 'No purpose recorded');
  /// } finally {
  ///   lockbox.close();
  ///   password.close();
  /// }
  /// ```
  String? get description => runtime.operations.lockboxGetVariable(
    handle,
    _lockboxDescriptionVariable,
  );

  /// Stores or replaces this Lockbox's encrypted human-readable description.
  ///
  /// [description] accepts the same UTF-8 content and one-mebibyte limit as a
  /// normal variable. The immutable Dart String remains outside reVault's
  /// control. Call [commit] to authenticate and publish the change; use
  /// [clearDescription] to remove it.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString(promptedPassword);
  /// final lockbox = Lockbox.create(
  ///   '/backups/production.lbox',
  ///   password: password,
  /// );
  /// try {
  ///   lockbox.setDescription(
  ///     'Production deployment credentials and recovery material',
  ///   );
  ///   lockbox.commit();
  /// } finally {
  ///   lockbox.close();
  ///   password.close();
  /// }
  /// ```
  void setDescription(String description) => runtime.operations
      .lockboxSetVariable(handle, _lockboxDescriptionVariable, description);

  /// Removes this Lockbox's encrypted description, if one is present.
  ///
  /// Call [commit] to authenticate and publish the change. Repeated calls are
  /// safe.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString(promptedPassword);
  /// final lockbox = Lockbox.open('/backups/retired.lbox', password: password);
  /// try {
  ///   lockbox.clearDescription();
  ///   lockbox.commit();
  /// } finally {
  ///   lockbox.close();
  ///   password.close();
  /// }
  /// ```
  void clearDescription() => runtime.operations.lockboxDeleteVariable(
    handle,
    _lockboxDescriptionVariable,
  );

  /// Stores a non-secret UTF-8 variable in Lockbox metadata.
  ///
  /// Use [setSecretVariable] for passwords, tokens, and other sensitive values.
  ///
  /// Example:
  /// ```dart
  /// lockbox.setVariable('environment', 'production');
  /// ```
  void setVariable(String name, String text) =>
      runtime.operations.lockboxSetVariable(handle, name, text);

  /// Stores a secret variable without converting it to an immutable String.
  ///
  /// The secret is copied into encrypted Lockbox state; the caller still owns
  /// and must close [secret].
  ///
  /// Example:
  /// ```dart
  /// final token = SecretBytes.take(tokenBytes);
  /// try {
  ///   lockbox.setSecretVariable('api-token', token);
  /// } finally {
  ///   token.close();
  /// }
  /// ```
  void setSecretVariable(String name, SecretBytes secret) => secret.withBytes(
    (bytes) => runtime.operations.lockboxSetSecretVariable(handle, name, bytes),
  );

  /// Returns a non-secret variable, or `null` when [name] is absent.
  ///
  /// Secret values must be read with [withSecretVariable].
  ///
  /// Example:
  /// ```dart
  /// final environment = lockbox.getVariable('environment') ?? 'development';
  /// ```
  String? getVariable(String name) =>
      runtime.operations.lockboxGetVariable(handle, name);

  /// Invokes [callback] with temporary secret bytes, then wipes that copy.
  ///
  /// Do not retain the supplied list outside the callback. Return only a
  /// non-secret result derived from it.
  ///
  /// Example:
  /// ```dart
  /// final response = lockbox.withSecretVariable(
  ///   'api-token',
  ///   (token) => client.authenticate(token),
  /// );
  /// ```
  T? withSecretVariable<T>(
    String name,
    T Function(Uint8List secret) callback,
  ) => runtime.operations.lockboxWithSecretVariable(handle, name, callback);

  /// Deletes the variable named [name].
  ///
  /// Example:
  /// ```dart
  /// lockbox.deleteVariable('legacy-token');
  /// lockbox.commit();
  /// ```
  void deleteVariable(String name) =>
      runtime.operations.lockboxDeleteVariable(handle, name);

  /// Applies all variable [moves] atomically in the next commit.
  ///
  /// Example:
  /// ```dart
  /// lockbox.moveVariables([
  ///   PathMove(source: 'old-name', destination: 'new-name'),
  /// ]);
  /// ```
  void moveVariables(List<PathMove> moves) => runtime.operations
      .lockboxMoveVariables(handle, DomainDecoders.pathMoves(moves));

  /// Lists variable names and sensitivity without returning secret values.
  ///
  /// Example:
  /// ```dart
  /// for (final variable in lockbox.listVariables()) {
  ///   print('${variable.name}: ${variable.sensitivity}');
  /// }
  /// ```
  List<Variable> listVariables() =>
      runtime.operations.lockboxListVariables(handle);

  /// Returns the sensitivity classification of variable [name].
  ///
  /// The result is `null` if the variable does not exist.
  ///
  /// Example:
  /// ```dart
  /// if (lockbox.variableSensitivity('api-token') == 'secret') {
  ///   // Read it only through withSecretVariable.
  /// }
  /// ```
  String? variableSensitivity(String name) =>
      runtime.operations.lockboxVariableSensitivity(handle, name);

  /// Adds a stored symbolic link from [path] to Lockbox [target].
  ///
  /// The link is archive metadata; it is created on the host only when
  /// extraction explicitly enables symlink restoration.
  ///
  /// Example:
  /// ```dart
  /// lockbox.addSymlink('/latest.txt', '/versions/v2.txt');
  /// ```
  void addSymlink(String path, String target, {bool replace = false}) =>
      runtime.operations.lockboxAddSymlink(handle, path, target, replace);

  /// Returns the stored target for the symbolic link at [path].
  ///
  /// Example:
  /// ```dart
  /// print(lockbox.symlinkTarget('/latest.txt'));
  /// ```
  String symlinkTarget(String path) =>
      runtime.operations.lockboxGetSymlinkTarget(handle, path);

  /// Returns the stable public identifier of this Lockbox.
  ///
  /// Use it to index Vault metadata such as remembered paths, access labels,
  /// and encrypted Lockbox passwords. It is not a content key.
  ///
  /// Example:
  /// ```dart
  /// vault.rememberLockbox(lockbox.id, '/secrets/team.lbox');
  /// ```
  Uint8List get id => runtime.operations.lockboxId(handle);

  /// Whether any file, directory, symlink, or record exists at [path].
  ///
  /// Example:
  /// ```dart
  /// if (!lockbox.exists('/config.json')) addDefaultConfig(lockbox);
  /// ```
  bool exists(String path) => runtime.operations.lockboxExists(handle, path);

  /// Whether [path] identifies a stored directory.
  ///
  /// Example:
  /// ```dart
  /// if (lockbox.isDirectory('/docs')) print('docs exists');
  /// ```
  bool isDirectory(String path) =>
      runtime.operations.lockboxIsDir(handle, path);

  /// Returns the portable Unix permission bits stored for [path].
  ///
  /// These bits are metadata until extraction enables permission restoration.
  ///
  /// Example:
  /// ```dart
  /// final mode = lockbox.permissions('/scripts/deploy.sh');
  /// ```
  int permissions(String path) =>
      runtime.operations.lockboxPermissions(handle, path);

  /// Replaces the portable Unix permission bits stored for [path].
  ///
  /// Example:
  /// ```dart
  /// lockbox.setPermissions('/scripts/deploy.sh', 0x1c0); // 0700
  /// ```
  void setPermissions(String path, int permissions) =>
      runtime.operations.lockboxSetPermissions(handle, path, permissions);

  /// Reads at most [length] bytes starting at [offset] from a stored file.
  ///
  /// Use this to avoid allocating an entire large file with [getFile].
  ///
  /// Example:
  /// ```dart
  /// final header = lockbox.readRange('/large.bin', 0, 4096);
  /// ```
  Uint8List readRange(String path, int offset, int length) =>
      runtime.operations.lockboxReadRange(handle, path, offset, length);

  /// Adds another password access slot and returns its stable slot identifier.
  ///
  /// This grants access; it does not store the password in the Vault. Remember
  /// it separately with [Vault.rememberPassword] if the product requires that.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretString.fromString(newPassword);
  /// try {
  ///   final slotId = lockbox.addPassword(password);
  ///   lockbox.commit();
  /// } finally {
  ///   password.close();
  /// }
  /// ```
  int addPassword(SecretString password) => password.withBytes(
    (bytes) => runtime.operations.lockboxAddPassword(handle, bytes),
  );

  /// Adds a [contact] access slot with local display [name].
  ///
  /// The recipient can then open subsequent committed revisions using the
  /// matching [ContactKeyPair].
  ///
  /// Example:
  /// ```dart
  /// final alice = vault.loadContact('alice');
  /// try {
  ///   lockbox.addContact(alice, 'alice');
  ///   lockbox.commit();
  /// } finally {
  ///   alice.dispose();
  /// }
  /// ```
  int addContact(ContactPublicKey contact, String name) =>
      runtime.operations.lockboxAddContact(handle, contact.handle, name);

  /// Deletes the access slot identified by [id].
  ///
  /// Keep at least one usable access method and commit the change. Deleting a
  /// slot does not erase copies of credentials held elsewhere.
  ///
  /// Example:
  /// ```dart
  /// lockbox.deleteKey(obsoleteSlotId);
  /// lockbox.commit();
  /// ```
  void deleteKey(int id) => runtime.operations.lockboxDeleteKey(handle, id);

  /// Lists password and contact access slots without exposing their secrets.
  ///
  /// Example:
  /// ```dart
  /// for (final slot in lockbox.listKeySlots()) {
  ///   print('${slot.id}: ${slot.protection} (${slot.algorithm})');
  /// }
  /// ```
  List<KeySlot> listKeySlots() =>
      runtime.operations.lockboxListKeySlots(handle);

  /// Sets the owner signing key used by subsequent [commit] calls.
  ///
  /// Use this after opening signed archives through a raw content key or
  /// password when the signing key was not loaded automatically from a Vault.
  ///
  /// Example:
  /// ```dart
  /// final owner = vault.loadProfileSigningKey('default');
  /// try {
  ///   lockbox.setOwnerSigningKey(owner);
  ///   lockbox.commit();
  /// } finally {
  ///   owner.dispose();
  /// }
  /// ```
  void setOwnerSigningKey(ProfileSigningKeyPair key) =>
      runtime.operations.lockboxSetOwnerSigningKey(handle, key.handle);

  /// Reports whether revisions are owner-signed and the owner fingerprint.
  ///
  /// Example:
  /// ```dart
  /// final owner = lockbox.ownerInspection();
  /// if (owner.signed) print(owner.fingerprint);
  /// ```
  OwnerInspection ownerInspection() =>
      runtime.operations.lockboxOwnerInspection(handle);

  /// Defines a new immutable revision of a typed form inside this Lockbox.
  ///
  /// Reusing an alias creates a later revision while existing records remain
  /// tied to their original revision.
  ///
  /// Example:
  /// ```dart
  /// final login = lockbox.defineForm('login', 'Login', 'Service login', [
  ///   FormField(id: 'username', label: 'Username', kind: 'text', required: true),
  ///   FormField(id: 'password', label: 'Password', kind: 'secret', required: true),
  /// ]);
  /// ```
  FormDefinition defineForm(
    String alias,
    String name,
    String description,
    List<FormField> fields,
  ) => runtime.operations.lockboxDefineForm(
    handle,
    alias,
    name,
    description,
    DomainDecoders.formFields(fields),
  );

  /// Lists the current form definitions embedded in this Lockbox.
  ///
  /// Example:
  /// ```dart
  /// for (final form in lockbox.listFormDefinitions()) {
  ///   print('${form.alias} revision ${form.revision}');
  /// }
  /// ```
  List<FormDefinition> listFormDefinitions() =>
      runtime.operations.lockboxListFormDefinitions(handle);

  /// Resolves a current form by alias or stable type identifier.
  ///
  /// Example:
  /// ```dart
  /// final login = lockbox.resolveForm('login');
  /// ```
  FormDefinition resolveForm(String reference) =>
      runtime.operations.lockboxResolveForm(handle, reference);

  /// Lists every stored revision for stable form [typeId].
  ///
  /// Example:
  /// ```dart
  /// final history = lockbox.listFormRevisions(login.typeId);
  /// ```
  List<FormDefinition> listFormRevisions(String typeId) =>
      runtime.operations.lockboxListFormRevisions(handle, typeId);

  /// Creates a named form record at [path] using [typeReference].
  ///
  /// [typeReference] may be the current alias or stable form type ID.
  ///
  /// Example:
  /// ```dart
  /// final record = lockbox.createFormRecord(
  ///   '/logins/example',
  ///   'login',
  ///   'Example account',
  /// );
  /// ```
  FormRecord createFormRecord(String path, String typeReference, String name) =>
      runtime.operations.lockboxCreateFormRecord(
        handle,
        path,
        typeReference,
        name,
      );

  /// Stores non-secret [text] in a form [field].
  ///
  /// Use the field's stable ID, not its display label.
  ///
  /// Example:
  /// ```dart
  /// lockbox.setFormField('/logins/example', 'username', 'alice@example.com');
  /// ```
  void setFormField(String path, String field, String text) =>
      runtime.operations.lockboxSetFormField(handle, path, field, text);

  /// Stores a secret form field without creating an immutable String.
  ///
  /// Example:
  /// ```dart
  /// final password = SecretBytes.take(passwordUtf8);
  /// try {
  ///   lockbox.setSecretFormField('/logins/example', 'password', password);
  /// } finally {
  ///   password.close();
  /// }
  /// ```
  void setSecretFormField(String path, String field, SecretBytes secret) =>
      secret.withBytes(
        (bytes) => runtime.operations.lockboxSetSecretFormField(
          handle,
          path,
          field,
          bytes,
        ),
      );

  /// Lists all typed form records without returning secret field values.
  ///
  /// Example:
  /// ```dart
  /// for (final record in lockbox.listFormRecords()) {
  ///   print('${record.path}: ${record.name}');
  /// }
  /// ```
  List<FormRecord> listFormRecords() =>
      runtime.operations.lockboxListFormRecords(handle);

  /// Returns the form record at [path], or `null` when it is absent.
  ///
  /// Secret values are omitted; read them through [withSecretFormField].
  ///
  /// Example:
  /// ```dart
  /// final record = lockbox.getFormRecord('/logins/example');
  /// if (record != null) showRecord(record);
  /// ```
  FormRecord? getFormRecord(String path) =>
      runtime.operations.lockboxGetFormRecord(handle, path);

  /// Deletes the form record at [path] in the next commit.
  ///
  /// Example:
  /// ```dart
  /// lockbox.deleteFormRecord('/logins/retired');
  /// lockbox.commit();
  /// ```
  void deleteFormRecord(String path) =>
      runtime.operations.lockboxDeleteFormRecord(handle, path);

  /// Applies all form-record [moves] atomically.
  ///
  /// Example:
  /// ```dart
  /// lockbox.moveFormRecords([
  ///   PathMove(source: '/logins/old', destination: '/logins/new'),
  /// ]);
  /// ```
  void moveFormRecords(List<PathMove> moves) => runtime.operations
      .lockboxMoveFormRecords(handle, DomainDecoders.pathMoves(moves));

  /// Returns a non-secret form [field], or `null` when it is absent.
  ///
  /// Example:
  /// ```dart
  /// final username = lockbox.getFormField('/logins/example', 'username');
  /// print(username?.value);
  /// ```
  FormValue? getFormField(String path, String field) =>
      runtime.operations.lockboxGetFormField(handle, path, field);

  /// Invokes [callback] with temporary secret field bytes, then wipes the copy.
  ///
  /// Do not retain the supplied list outside the callback.
  ///
  /// Example:
  /// ```dart
  /// final result = lockbox.withSecretFormField(
  ///   '/logins/example',
  ///   'password',
  ///   (password) => client.login(password),
  /// );
  /// ```
  T? withSecretFormField<T>(
    String path,
    String field,
    T Function(Uint8List secret) callback,
  ) => runtime.operations.lockboxWithSecretFormField(
    handle,
    path,
    field,
    callback,
  );

  /// Serializes the current committed Lockbox state.
  ///
  /// Use this for an in-memory Lockbox. File-backed handles are written by
  /// [commit] automatically.
  ///
  /// Example:
  /// ```dart
  /// lockbox.commit();
  /// await upload(lockbox.bytes);
  /// ```
  Uint8List get bytes => runtime.operations.lockboxToBytes(handle);

  /// Wipes this process's content key and releases the native lockbox handle.
  ///
  /// This does not remove a copy explicitly retained by [AgentSession].
  /// Pending uncommitted mutations are discarded.
  ///
  /// Example:
  /// ```dart
  /// try {
  ///   print(lockbox.list('/'));
  /// } finally {
  ///   lockbox.close();
  /// }
  /// ```
  void close() {
    if (!disposed) {
      _nativeFinalizer?.detach(this);
      runtime.operations.lockboxFree(handle);
      handle = ffi.nullptr;
    }
  }

  /// Deprecated alias for [close].
  ///
  /// Example:
  /// ```dart
  /// lockbox.dispose(); // New code uses lockbox.close().
  /// ```
  @Deprecated('Use close().')
  void dispose() => close();
}
