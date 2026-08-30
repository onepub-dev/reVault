import 'dart:ffi' as ffi;
import 'dart:typed_data';

import 'package:revault_api/src/agent_activity.dart';
import 'package:revault_api/src/agent_activity_kind.dart';
import 'package:revault_api/src/domain_models.dart';
import 'package:revault_api/src/exceptions.dart';
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/lockbox_options.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/profile_signing_key_pair.dart';
import 'package:revault_api/src/revault.dart';
import 'package:revault_api/src/secret_bytes.dart';
import 'package:revault_api/src/secret_string.dart';
import 'package:revault_api/src/vault.dart';

/// Controls the single platform Session Agent process and its temporary keys.
///
/// Ordinary [Lockbox.open] and [Lockbox.create] calls do not use this service.
/// The agent is useful when access must survive a short-lived CLI process or be
/// shared with another process. It caches decrypted content keys, never open
/// file handles, and its entries are always temporary.
///
/// Example:
/// ```dart
/// final agent = AgentSession.instance;
/// agent.start();
/// agent.keepOpenWithPassword(path, password, duration: Duration(minutes: 15));
/// final box = agent.acquireOpenLockbox(path);
/// ```
final class AgentSession extends Owned {
  AgentSession._(Revault runtime)
    : super(runtime, runtime.operations.vaultLocal());

  static AgentSession? _instance;

  /// The process-wide client for the single platform Session Agent.
  ///
  /// Retrieving the client does not start the agent.
  ///
  /// Example:
  /// ```dart
  /// final agent = AgentSession.instance;
  /// if (!agent.isRunning) agent.start();
  /// ```
  static AgentSession get instance {
    final runtime = Revault.runtime;
    final existing = _instance;
    if (existing != null && identical(existing.runtime, runtime)) {
      return existing;
    }
    return _instance = AgentSession._(runtime);
  }

  /// Whether the Session Agent process is currently reachable.
  ///
  /// Example:
  /// ```dart
  /// print(agent.isRunning ? 'Agent available' : 'Agent stopped');
  /// ```
  bool get isRunning => runtime.operations.vaultIsRunning();

  /// Starts the platform Session Agent service if it is not already running.
  ///
  /// Example:
  /// ```dart
  /// final agent = AgentSession.instance;
  /// agent.start();
  /// ```
  void start() => runtime.operations.vaultAgentStart();

  /// Runs the agent server loop until another client requests [stop].
  ///
  /// This is for applications building their own agent executable. It blocks
  /// the current isolate and is not part of normal client startup.
  ///
  /// Example:
  /// ```dart
  /// if (arguments.contains('--serve-revault-agent')) {
  ///   AgentSession.instance.serve();
  /// }
  /// ```
  void serve() => runtime.operations.vaultAgentServe();

  /// Wipes agent-held secrets and stops the agent process.
  ///
  /// Existing process-local Lockbox handles remain usable until closed.
  ///
  /// Example:
  /// ```dart
  /// if (agent.isRunning) agent.stop();
  /// ```
  void stop() => runtime.operations.vaultAgentStop();

  /// Verifies that agent IPC ownership and permissions meet security policy.
  ///
  /// Call this in diagnostic tools when explaining why the agent is unusable.
  ///
  /// Example:
  /// ```dart
  /// try {
  ///   agent.verifyTransport();
  /// } on RevaultException catch (error) {
  ///   showAgentRepairGuidance(error.guidance);
  /// }
  /// ```
  void verifyTransport() => runtime.operations.vaultAgentVerifyTransport();

  /// Keeps the password-protected lockbox at [path] open for [duration].
  ///
  /// The agent stores the derived content key, not [password]. This operation
  /// is explicit; closing a process-local [Lockbox] does not undo it.
  ///
  /// Example:
  /// ```dart
  /// agent.keepOpenWithPassword(
  ///   '/secrets/team.lbox',
  ///   password,
  ///   duration: Duration(minutes: 30),
  /// );
  /// ```
  void keepOpenWithPassword(
    String path,
    SecretString password, {
    required Duration duration,
  }) => password.withBytes(
    (bytes) => runtime.operations.vaultCacheLockboxPassword(
      handle,
      path,
      bytes,
      duration.inSeconds,
    ),
  );

  /// Caches an explicitly supplied [contentKey] under [lockboxId].
  ///
  /// Use this when the application already holds a Lockbox content key. The
  /// native default TTL applies; [keepOpenWithPassword] accepts a password and
  /// an explicit duration instead.
  ///
  /// Example:
  /// ```dart
  /// agent.keepOpenWithContentKey(lockbox.id, contentKey);
  /// ```
  void keepOpenWithContentKey(Uint8List lockboxId, SecretBytes contentKey) =>
      contentKey.withBytes(
        (bytes) => runtime.operations.vaultAgentPut(lockboxId, bytes),
      );

  /// Closes the lockbox at [path] by forgetting its agent-cached content key.
  ///
  /// Existing in-process [Lockbox] objects retain their independent keys.
  ///
  /// Example:
  /// ```dart
  /// agent.closeLockbox('/secrets/team.lbox');
  /// ```
  void closeLockbox(String path) =>
      runtime.operations.vaultCloseLockbox(handle, path);

  /// Acquires an independent process-local handle to an already-open Lockbox.
  ///
  /// The agent's expiry remains fixed at the time the key was cached. The
  /// returned [Lockbox] owns a copied content key and remains usable after that
  /// expiry until it is closed. Throws [AgentLockboxNotOpenException] only for
  /// a cache miss; transport and security failures propagate unchanged.
  ///
  /// Example:
  /// ```dart
  /// final box = AgentSession.instance.acquireOpenLockbox(path);
  /// try {
  ///   print(box.list('/'));
  /// } finally {
  ///   box.close();
  /// }
  /// ```
  Lockbox acquireOpenLockbox(String path, {LockboxOptions? options}) {
    final lockbox = tryAcquireOpenLockbox(path, options: options);
    if (lockbox == null) throw AgentLockboxNotOpenException(path);
    return lockbox;
  }

  /// Tries to acquire an independent handle to an already-open Lockbox.
  ///
  /// Returns `null` only when the agent has no live key for [path]. Genuine
  /// agent errors are not converted to cache misses.
  ///
  /// Example:
  /// ```dart
  /// final box = AgentSession.instance.tryAcquireOpenLockbox(path);
  /// if (box == null) return Lockbox.open(path);
  /// return box;
  /// ```
  Lockbox? tryAcquireOpenLockbox(String path, {LockboxOptions? options}) {
    final lockboxId = runtime.inspectLockboxFile(path).lockboxId;
    final bytes = runtime.operations.vaultAgentTryGet(lockboxId);
    if (bytes == null) return null;
    final key = SecretBytes.take(bytes);
    try {
      return Lockbox.open(path, contentKey: key, options: options);
    } finally {
      key.close();
    }
  }

  /// Closes every agent-open Lockbox without changing Vault credentials.
  ///
  /// Example:
  /// ```dart
  /// agent.closeAll();
  /// assert(agent.listOpenLockboxes().isEmpty);
  /// ```
  void closeAll() => runtime.operations.vaultCloseAll(handle);

  /// Lists non-secret metadata for Lockboxes currently open in the agent.
  ///
  /// Example:
  /// ```dart
  /// for (final entry in agent.listOpenLockboxes()) print(entry.path);
  /// ```
  List<AgentEntry> listOpenLockboxes() => runtime.operations.vaultAgentList();

  /// Removes every secret currently held by the agent.
  ///
  /// This is broader than [closeAll], which targets Lockbox content keys.
  ///
  /// Example:
  /// ```dart
  /// agent.clearAllSecrets();
  /// ```
  void clearAllSecrets() => runtime.operations.vaultForgetAll();

  /// Caches [key] under opaque [id].
  ///
  /// The agent retains its own copy until expiry or an explicit forget
  /// operation. The caller retains ownership of [key].
  ///
  /// Example:
  /// ```dart
  /// agent.cacheKey(integrationId, secretKey);
  /// ```
  void cacheKey(Uint8List id, SecretBytes key) =>
      key.withBytes((bytes) => runtime.operations.vaultAgentPut(id, bytes));

  /// Returns the key cached under [id].
  ///
  /// The caller owns and must close the returned [SecretBytes].
  ///
  /// Example:
  /// ```dart
  /// final key = agent.key(integrationId);
  /// try {
  ///   useIntegrationKey(key);
  /// } finally {
  ///   key.close();
  /// }
  /// ```
  SecretBytes key(Uint8List id) =>
      SecretBytes.take(runtime.operations.vaultAgentGet(id));

  /// Forgets the key cached under [id].
  ///
  /// Example:
  /// ```dart
  /// agent.forgetKey(integrationId);
  /// ```
  void forgetKey(Uint8List id) => runtime.operations.vaultAgentForget(id);

  /// Caches a Vault unlock [key] for [duration].
  ///
  /// Use this for a cross-process workflow that has already derived an unlock
  /// key. Platform credential policy and [Vault.open] operate on passphrases.
  ///
  /// Example:
  /// ```dart
  /// agent.cacheVaultUnlockKey(
  ///   vaultId,
  ///   unlockKey,
  ///   duration: Duration(minutes: 5),
  /// );
  /// ```
  void cacheVaultUnlockKey(
    String vaultId,
    SecretBytes key, {
    required Duration duration,
  }) => key.withBytes(
    (bytes) => runtime.operations.vaultAgentPutVaultUnlockKey(
      vaultId,
      bytes,
      duration.inSeconds,
    ),
  );

  /// Returns the cached unlock key for [vaultId].
  ///
  /// The caller must close the returned [SecretBytes].
  ///
  /// Example:
  /// ```dart
  /// final key = agent.vaultUnlockKey(vaultId);
  /// try {
  ///   useUnlockKey(key);
  /// } finally {
  ///   key.close();
  /// }
  /// ```
  SecretBytes vaultUnlockKey(String vaultId) =>
      SecretBytes.take(runtime.operations.vaultAgentGetVaultUnlockKey(vaultId));

  /// Forgets the cached unlock key for [vaultId].
  ///
  /// Example:
  /// ```dart
  /// agent.forgetVaultUnlockKey(vaultId);
  /// ```
  void forgetVaultUnlockKey(String vaultId) =>
      runtime.operations.vaultAgentForgetVaultUnlockKey(vaultId);

  /// Caches a Vault Profile [signingKey] for [duration].
  ///
  /// This cross-process cache avoids repeatedly opening the Vault in
  /// short-lived tools. The key is a Profile credential until a Lockbox assigns
  /// it the owner role.
  ///
  /// Example:
  /// ```dart
  /// agent.cacheProfileSigningKey(
  ///   vault.id,
  ///   'personal',
  ///   signingKey,
  ///   duration: Duration(minutes: 10),
  /// );
  /// ```
  void cacheProfileSigningKey(
    String vaultId,
    String profile,
    ProfileSigningKeyPair signingKey, {
    required Duration duration,
  }) => runtime.operations.vaultAgentPutOwnerSigningKey(
    vaultId,
    profile,
    signingKey.handle,
    duration.inSeconds,
  );

  /// Returns the cached signing key for a Vault [profile].
  ///
  /// The caller owns the returned handle and must dispose it.
  ///
  /// Example:
  /// ```dart
  /// final key = agent.profileSigningKey(vault.id, 'personal');
  /// try {
  ///   lockbox.setOwnerSigningKey(key);
  /// } finally {
  ///   key.dispose();
  /// }
  /// ```
  ProfileSigningKeyPair profileSigningKey(String vaultId, String profile) =>
      ProfileSigningKeyPair.internal(
        runtime,
        runtime.operations.vaultAgentGetOwnerSigningKey(vaultId, profile),
      );

  /// Forgets the cached signing key for a Vault [profile].
  ///
  /// Existing process-local key handles remain usable until disposed.
  ///
  /// Example:
  /// ```dart
  /// agent.forgetProfileSigningKey(vault.id, 'personal');
  /// ```
  void forgetProfileSigningKey(String vaultId, String profile) =>
      runtime.operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);

  /// Registers secret-using [kind] until the returned token is disposed.
  ///
  /// Agent executables use activity tokens to coordinate suspend protection
  /// while an operation is actively handling decrypted material.
  ///
  /// Example:
  /// ```dart
  /// final activity = agent.beginActivity(AgentActivityKind.open);
  /// try {
  ///   openSelectedLockbox();
  /// } finally {
  ///   activity.dispose();
  /// }
  /// ```
  AgentActivity beginActivity(AgentActivityKind kind) => AgentActivity.internal(
    runtime,
    runtime.operations.vaultAgentBeginActivity(kind.nativeName),
  );

  /// Reports support for sleep inhibition and suspend notification.
  ///
  /// Use this in agent diagnostics; ordinary Lockbox users need not inspect it.
  ///
  /// Example:
  /// ```dart
  /// final support = agent.sleepSupport();
  /// if (!support.supported) showReducedProtectionWarning();
  /// ```
  SleepSupport sleepSupport() => runtime.operations.vaultAgentSleepSupport();

  /// The agent log file path, when file logging is enabled.
  ///
  /// Example:
  /// ```dart
  /// print('Agent log: ${agent.logPath}');
  /// ```
  String get logPath => runtime.operations.vaultAgentLogPath();

  /// A human-readable description of the active log destination.
  ///
  /// Example:
  /// ```dart
  /// print('Agent logging to ${agent.logDestination}');
  /// ```
  String get logDestination => runtime.operations.vaultAgentLogDestination();

  /// @nodoc
  SecretBytes contentKeyInternal(Uint8List lockboxId) =>
      SecretBytes.take(runtime.operations.vaultAgentGet(lockboxId));

  /// Releases this process's agent client handle without stopping the agent.
  ///
  /// A later [instance] access creates a new client handle.
  ///
  /// Example:
  /// ```dart
  /// final agent = AgentSession.instance;
  /// try {
  ///   print(agent.listOpenLockboxes());
  /// } finally {
  ///   agent.dispose();
  /// }
  /// ```
  void dispose() {
    if (!disposed) {
      runtime.operations.vaultFree(handle);
      handle = ffi.nullptr;
      if (identical(_instance, this)) _instance = null;
    }
  }
}
