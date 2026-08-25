import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:typed_data';

import 'package:revault_api/src/agent_session.dart';
import 'package:revault_api/src/binding_operations.dart';
import 'package:revault_api/src/contact_key_pair.dart';
import 'package:revault_api/src/contact_public_key.dart';
import 'package:revault_api/src/domain_models.dart';
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/lockbox_options.dart';
import 'package:revault_api/src/profile_signing_key_pair.dart';
import 'package:revault_api/src/profile_signing_public_key.dart';
import 'package:revault_api/src/read_only_vault.dart';
import 'package:revault_api/src/revault_native.dart';
import 'package:revault_api/src/secret_bytes.dart';
import 'package:revault_api/src/secret_string.dart';
import 'package:revault_api/src/vault.dart';

/// Loads and owns the process-wide reVault native runtime.
///
/// Await [load] once when the application starts. Then use [Vault], [Lockbox],
/// and [AgentSession] directly; callers do not pass this runtime to them. Key
/// generation, import, export, and format utilities remain on this object.
///
/// Example:
/// ```dart
/// final revault = await Revault.load();
/// final vault = Vault.open(passphrase: vaultPassphrase);
/// final lockbox = Lockbox.open('/secrets/team.lbox', vault: vault);
/// ```
final class Revault {
  static Revault? _current;

  /// Loads the native library and installs the process-wide runtime.
  ///
  /// Call this once during application startup, before using any other reVault
  /// type.
  ///
  /// With no argument, loads the target-specific carrier published by the
  /// package build hook. The SDK bundles and resolves that carrier, so normal
  /// Dart and Flutter applications do not provide a filesystem path.
  ///
  /// Set [nativeLibraryPath] only when an application installer deliberately
  /// owns a shared carrier outside the Dart bundle, for example
  /// `/opt/my_app/lib/librevault_api.so`. The path is opened directly and takes
  /// precedence over the packaged native asset. A relative name uses the
  /// operating system loader's normal search rules.
  ///
  /// When [nativeLibraryPath] is omitted, a non-empty inherited
  /// `REVAULT_LIBRARY` value is opened next. This environment hook is intended
  /// for launchers and diagnostics; installed-package acceptance deliberately
  /// removes it so the default branch proves the packaged carrier works.
  ///
  /// Example:
  /// ```dart
  /// final revault = await Revault.load();
  /// print(revault.lockboxFormatVersion);
  /// ```
  static Future<Revault> load({String? nativeLibraryPath}) async {
    if (nativeLibraryPath != null && nativeLibraryPath.isEmpty) {
      throw ArgumentError.value(
        nativeLibraryPath,
        'nativeLibraryPath',
        'must not be empty',
      );
    }
    final inheritedPath = Platform.environment['REVAULT_LIBRARY'];
    final selectedPath = nativeLibraryPath ??
        (inheritedPath == null || inheritedPath.isEmpty ? null : inheritedPath);
    final loaded = Revault._(selectedPath);
    _current = loaded;
    return loaded;
  }

  /// @nodoc
  static Revault get runtime =>
      _current ??
      (throw StateError(
        'Call and await Revault.load() before using Vault, Lockbox, or AgentSession.',
      ));

  Revault._(String? nativeLibraryPath)
    : operations = BindingOperations(
        nativeLibraryPath == null
            ? RevaultNative()
            : RevaultNative.open(ffi.DynamicLibrary.open(nativeLibraryPath)),
      ) {
    _current = this;
  }

  /// @nodoc
  final BindingOperations operations;

  /// Returns the lockbox format version written by this runtime.
  ///
  /// Use this for diagnostics or compatibility displays; applications normally
  /// let [Lockbox.open] validate a file automatically.
  ///
  /// Example:
  /// ```dart
  /// print('Writes lockbox format ${revault.lockboxFormatVersion}');
  /// ```
  int get lockboxFormatVersion => operations.lockboxFormatVersion();

  /// Reads the format version from [lockboxArchive] without opening it.
  ///
  /// This is useful before requesting a credential or attempting migration.
  /// The archive contents remain encrypted and are not authenticated here.
  ///
  /// Example:
  /// ```dart
  /// final bytes = await File(path).readAsBytes();
  /// final version = revault.probeLockboxFormatVersion(bytes);
  /// ```
  int probeLockboxFormatVersion(Uint8List lockboxArchive) =>
      operations.lockboxProbeFormatVersion(lockboxArchive);

  /// Returns the local-Vault structure version written by this runtime.
  ///
  /// Use it in diagnostics and migration tooling, rather than to decide
  /// whether an existing Vault can be opened.
  ///
  /// Example:
  /// ```dart
  /// print('Vault structure ${revault.currentVaultStructureVersion}');
  /// ```
  int get currentVaultStructureVersion =>
      operations.vaultStructureVersionCurrent();

  /// Authenticates the Vault at [root] and returns its structure version.
  ///
  /// Use this in migration or support tooling when the full writable [Vault]
  /// is not needed. [password] is the Vault passphrase, not a Lockbox password.
  ///
  /// Example:
  /// ```dart
  /// final version = revault.probeVaultStructureVersion(
  ///   '/home/alice/.revault',
  ///   vaultPassphrase,
  /// );
  /// ```
  int probeVaultStructureVersion(String root, SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) => operations.vaultDirectoryProbeStructureVersion(root, bytes),
      );

  /// Generates a contact encryption identity in secure native memory.
  ///
  /// Create one for a Vault profile that needs to receive Lockbox access. Store
  /// the private pair in [Vault.storePrivateKey] and share only its public key.
  ///
  /// Example:
  /// ```dart
  /// final identity = revault.generateContactKeyPair();
  /// vault.storePrivateKey('alice', identity);
  /// identity.dispose();
  /// ```
  ContactKeyPair generateContactKeyPair() =>
      ContactKeyPair.internal(this, operations.keyContactGenerate());

  /// Reconstructs a contact key pair from [privateKeyRecord].
  ///
  /// Obtain this opaque binary record from [ContactKeyPair.privateRecord]. For
  /// PEM, JWK, JWKS, or raw hexadecimal exports, use [importContactKeyPair].
  /// Use this paired operation for an application-controlled binary backup.
  ///
  /// Example:
  /// ```dart
  /// final restored = revault.contactKeyPairFromPrivate(privateRecord);
  /// try {
  ///   vault.storePrivateKey('restored', restored);
  /// } finally {
  ///   restored.dispose();
  /// }
  /// ```
  ContactKeyPair contactKeyPairFromPrivate(Uint8List privateKeyRecord) =>
      ContactKeyPair.internal(
        this,
        operations.keyContactFromPrivate(privateKeyRecord),
      );

  /// Imports an [exportedPrivateKey] created by [ContactKeyPair.export].
  ///
  /// Native Lockbox PEM, JWK, JWKS, and raw hexadecimal records are detected
  /// automatically. The returned private key remains in native secure memory
  /// until [ContactKeyPair.dispose] is called.
  ///
  /// Example:
  /// ```dart
  /// final imported = revault.importContactKeyPair(pemBytes);
  /// try {
  ///   vault.storePrivateKey('alice', imported);
  /// } finally {
  ///   imported.dispose();
  /// }
  /// ```
  ContactKeyPair importContactKeyPair(Uint8List exportedPrivateKey) =>
      ContactKeyPair.internal(
        this,
        operations.vaultKeyImportPrivate(exportedPrivateKey),
      );

  /// Reconstructs a public key from [publicKeyBytes].
  ///
  /// Obtain this exact algorithm-specific representation from
  /// [ContactKeyPair.publicBytes]. For an exported PEM, JWK, JWKS, or raw
  /// hexadecimal record, use [importContactPublicKey].
  ///
  /// Example:
  /// ```dart
  /// final publicKey = revault.contactPublicKeyFromBytes(canonicalPublicBytes);
  /// try {
  ///   lockbox.addContact(publicKey, 'alice');
  /// } finally {
  ///   publicKey.dispose();
  /// }
  /// ```
  ContactPublicKey contactPublicKeyFromBytes(Uint8List publicKeyBytes) =>
      ContactPublicKey.internal(
        this,
        operations.keyContactPublicFromBytes(publicKeyBytes),
      );

  /// Imports an [exportedPublicKey] created by [ContactPublicKey.export].
  ///
  /// Native Lockbox PEM, JWK, JWKS, and raw hexadecimal records are detected
  /// automatically.
  ///
  /// Example:
  /// ```dart
  /// final contact = revault.importContactPublicKey(exportedPublicKey);
  /// try {
  ///   vault.storeContact('alice', contact);
  /// } finally {
  ///   contact.dispose();
  /// }
  /// ```
  ContactPublicKey importContactPublicKey(Uint8List exportedPublicKey) =>
      ContactPublicKey.internal(
        this,
        operations.vaultKeyImportPublic(exportedPublicKey),
      );

  /// Generates a signing identity for a Vault Profile.
  ///
  /// A Profile signing key is distinct from its contact-encryption key. Assign
  /// it to a Lockbox when that Profile is to occupy the owner role.
  ///
  /// Example:
  /// ```dart
  /// final signingKey = revault.generateProfileSigningKeyPair();
  /// final box = Lockbox.create(path, contentKey: key, signingKey: signingKey);
  /// ```
  ProfileSigningKeyPair generateProfileSigningKeyPair() =>
      ProfileSigningKeyPair.internal(this, operations.keySigningGenerate());

  /// Reconstructs a Profile signing key from [privateKeyRecord].
  ///
  /// Obtain the opaque record from [ProfileSigningKeyPair.privateRecord].
  ///
  /// Example:
  /// ```dart
  /// final signingKey = revault.profileSigningKeyPairFromPrivate(privateRecord);
  /// lockbox.setOwnerSigningKey(signingKey);
  /// ```
  ProfileSigningKeyPair profileSigningKeyPairFromPrivate(
    Uint8List privateKeyRecord,
  ) => ProfileSigningKeyPair.internal(
    this,
    operations.keySigningFromPrivate(privateKeyRecord),
  );

  /// Reconstructs a signing public key from [publicKeyBytes].
  ///
  /// Obtain this exact algorithm-specific representation from
  /// [ProfileSigningKeyPair.publicBytes].
  ///
  /// Example:
  /// ```dart
  /// final profileKey = revault.profileSigningPublicKeyFromBytes(publicBytes);
  /// vault.storeContactSigningKey('alice', profileKey);
  /// profileKey.dispose();
  /// ```
  ProfileSigningPublicKey profileSigningPublicKeyFromBytes(
    Uint8List publicKeyBytes,
  ) => ProfileSigningPublicKey.internal(
    this,
    operations.keySigningPublicFromBytes(publicKeyBytes),
  );

  /// Formats [fingerprint] as grouped hexadecimal text for human comparison.
  ///
  /// Use this for displaying or transcribing key fingerprints, not for
  /// exporting a complete key record.
  ///
  /// Example:
  /// ```dart
  /// final display = revault.formatKeyHex(contact.fingerprint());
  /// print('Verify with Alice: $display');
  /// ```
  String formatKeyHex(Uint8List fingerprint) =>
      operations.vaultKeyFormatHex(fingerprint);

  /// Decodes grouped hexadecimal text produced by [formatKeyHex].
  ///
  /// Example:
  /// ```dart
  /// final fingerprintBytes = revault.decodeKeyHex(userEnteredFingerprint);
  /// ```
  Uint8List decodeKeyHex(String groupedFingerprint) =>
      operations.vaultKeyDecodeHex(groupedFingerprint);

  /// Formats [fingerprint] as compact Crockford Base32 text.
  ///
  /// Use this when a fingerprint must be read aloud or entered manually.
  ///
  /// Example:
  /// ```dart
  /// final code = revault.formatKeyCrockford(contact.fingerprint());
  /// ```
  String formatKeyCrockford(Uint8List fingerprint) =>
      operations.vaultKeyFormatCrockford(fingerprint);

  /// Normalizes [encodedKey] into a grouped human-readable form.
  ///
  /// This accepts common transcription variations before presenting a value
  /// for side-by-side verification.
  ///
  /// Example:
  /// ```dart
  /// final normalized = revault.formatKeyCrockfordReading(userInput);
  /// ```
  String formatKeyCrockfordReading(String encodedKey) =>
      operations.vaultKeyFormatCrockfordReading(encodedKey);

  /// Decodes a Crockford fingerprint into its original bytes.
  ///
  /// Example:
  /// ```dart
  /// final fingerprintBytes = revault.decodeKeyCrockford(userInput);
  /// ```
  Uint8List decodeKeyCrockford(String encodedKey) =>
      operations.vaultKeyDecodeCrockford(encodedKey);

  /// Encodes arbitrary [bytes] as ungrouped lowercase hexadecimal text.
  ///
  /// This general byte utility is useful for machine-facing serialization;
  /// use [formatKeyHex] for fingerprints shown to people.
  ///
  /// Example:
  /// ```dart
  /// final encoded = revault.hexEncode(Uint8List.fromList([0, 15, 255]));
  /// // encoded == '000fff'
  /// ```
  String hexEncode(Uint8List bytes) => operations.vaultKeyHexEncode(bytes);

  /// Decodes ungrouped [hex] into bytes.
  ///
  /// Example:
  /// ```dart
  /// final bytes = revault.hexDecode('000fff');
  /// ```
  Uint8List hexDecode(String hex) => operations.vaultKeyHexDecode(hex);

  /// Creates an unsigned in-memory lockbox protected by [key].
  ///
  /// [key] must contain a valid content key. When supplied, [options] controls
  /// cache and worker behavior. Call [Lockbox.commit] after mutations and
  /// [Lockbox.dispose] when finished.
  /// @nodoc
  Lockbox createLockboxInternal(SecretBytes key, [LockboxOptions? options]) =>
      key.withBytes((keyBytes) {
        final handle = options == null
            ? operations.lockboxCreate(keyBytes)
            : operations.lockboxCreateWithOptions(
                keyBytes,
                options.cacheMode.nativeName,
                options.cacheBytes,
                options.workload.nativeName,
                options.worker.nativeName,
                options.jobs,
              );
        return Lockbox.internal(this, handle);
      });

  /// Creates an in-memory lockbox protected by [password].
  /// @nodoc
  Lockbox createLockboxWithPasswordInternal(SecretString password) =>
      password.withBytes(
        (bytes) =>
            Lockbox.internal(this, operations.lockboxCreatePassword(bytes)),
      );

  /// Creates a lockbox whose content key is wrapped for [contact].
  /// @nodoc
  Lockbox createLockboxForContactInternal(ContactPublicKey contact) =>
      Lockbox.internal(this, operations.lockboxCreateContact(contact.handle));

  /// Opens [archive] using its content [key].
  ///
  /// When supplied, [options] controls cache and worker behavior for the
  /// returned mutable lockbox.
  /// @nodoc
  Lockbox openLockboxInternal(
    Uint8List archive,
    SecretBytes key, [
    LockboxOptions? options,
  ]) => key.withBytes((keyBytes) {
    final handle = options == null
        ? operations.lockboxOpen(archive, keyBytes)
        : operations.lockboxOpenWithOptions(
            archive,
            keyBytes,
            options.cacheMode.nativeName,
            options.cacheBytes,
            options.workload.nativeName,
            options.worker.nativeName,
            options.jobs,
          );
    return Lockbox.internal(this, handle);
  });

  /// Opens [archive] using a password access slot.
  /// @nodoc
  Lockbox openLockboxWithPasswordInternal(
    Uint8List archive,
    SecretString password,
  ) => password.withBytes(
    (bytes) =>
        Lockbox.internal(this, operations.lockboxOpenPassword(archive, bytes)),
  );

  /// Opens [archive] using a contact private key.
  /// @nodoc
  Lockbox openLockboxForContactInternal(
    Uint8List archive,
    ContactKeyPair contact,
  ) => Lockbox.internal(
    this,
    operations.lockboxOpenContact(archive, contact.handle),
  );

  /// Inspects public structural metadata in the Lockbox file at [path].
  ///
  /// Use this before opening when you need its ID, key-slot summary, or signing
  /// status. It does not decrypt file names, variables, or content.
  ///
  /// Example:
  /// ```dart
  /// final inspection = revault.inspectLockboxFile('/secrets/team.lbox');
  /// print('${inspection.keySlots.length} access methods');
  /// ```
  FileInspection inspectLockboxFile(String path) =>
      operations.lockboxInspectFile(path);

  /// Scans a possibly damaged Lockbox file at [path] using its content [key].
  ///
  /// Use this diagnostic before salvage; it reports recoverable and corrupt
  /// records without replacing the original file.
  ///
  /// Example:
  /// ```dart
  /// final report = revault.scanLockboxPath(path, contentKey);
  /// print('${report.intactFileCount} intact files');
  /// ```
  RecoveryReport scanLockboxPath(String path, SecretBytes key) =>
      key.withBytes((bytes) => operations.lockboxRecoveryScanPath(path, bytes));

  /// Scans possibly damaged serialized [archive] bytes using content [key].
  ///
  /// Choose this byte-oriented variant when the archive came from a network or
  /// database rather than a host file.
  ///
  /// Example:
  /// ```dart
  /// final report = revault.scanLockbox(downloadedArchive, contentKey);
  /// ```
  RecoveryReport scanLockbox(Uint8List archive, SecretBytes key) =>
      key.withBytes((bytes) => operations.lockboxRecoveryScan(archive, bytes));

  /// Salvages intact state from [archive] into a new clean lockbox.
  ///
  /// [profileSigningKey] is required when the recovered Lockbox must remain
  /// owner-signed.
  /// The returned Lockbox is in memory; inspect it, call [Lockbox.commit], and
  /// persist [Lockbox.bytes] explicitly.
  ///
  /// Example:
  /// ```dart
  /// final recovered = revault.salvageLockbox(damagedBytes, contentKey);
  /// try {
  ///   recovered.commit();
  ///   await File(outputPath).writeAsBytes(recovered.bytes);
  /// } finally {
  ///   recovered.close();
  /// }
  /// ```
  Lockbox salvageLockbox(
    Uint8List archive,
    SecretBytes key, [
    ProfileSigningKeyPair? profileSigningKey,
  ]) => key.withBytes(
    (bytes) => Lockbox.internal(
      this,
      operations.lockboxRecoverySalvage(
        archive,
        bytes,
        profileSigningKey?.handle ?? ffi.nullptr,
      ),
    ),
  );

  /// Opens the writable local metadata vault at [root].
  /// @nodoc
  Vault openVaultInternal(String root, SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) =>
            Vault.internal(this, operations.vaultDirectoryOpen(root, bytes)),
      );

  /// Opens a metadata-only view at [root] without loading signing keys.
  /// @nodoc
  ReadOnlyVault openReadOnlyVaultInternal(
    String root,
    SecretString passphrase,
  ) => passphrase.withBytes(
    (bytes) =>
        ReadOnlyVault.internal(this, operations.vaultReadOnlyOpen(root, bytes)),
  );

  /// Opens the default metadata vault without loading signing keys.
  /// @nodoc
  ReadOnlyVault openDefaultReadOnlyVaultInternal(SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) => ReadOnlyVault.internal(
          this,
          operations.vaultReadOnlyOpenDefault(bytes),
        ),
      );

  /// Opens or creates a writable metadata vault at [root].
  /// @nodoc
  Vault openOrCreateVaultInternal(String root, SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) => Vault.internal(
          this,
          operations.vaultDirectoryOpenOrCreate(root, bytes),
        ),
      );

  /// Replaces the metadata vault at [root] with a new empty vault.
  /// @nodoc
  Vault replaceVaultInternal(String root, SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) =>
            Vault.internal(this, operations.vaultDirectoryReplace(root, bytes)),
      );

  /// Opens or creates the platform-default metadata vault.
  /// @nodoc
  Vault openOrCreateDefaultVaultInternal(SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) => Vault.internal(
          this,
          operations.vaultDirectoryOpenOrCreateDefault(bytes),
        ),
      );

  /// Replaces the platform-default metadata vault with a new empty vault.
  /// @nodoc
  Vault replaceDefaultVaultInternal(SecretString passphrase) =>
      passphrase.withBytes(
        (bytes) => Vault.internal(
          this,
          operations.vaultDirectoryReplaceDefault(bytes),
        ),
      );

  /// Returns the platform-default metadata-vault directory.
  /// @nodoc
  String get defaultVaultDirectoryInternal =>
      operations.vaultDefaultDirectory();

  /// Returns the platform-default metadata-vault data path.
  /// @nodoc
  String get defaultVaultPathInternal => operations.vaultDefaultPath();

  /// Writes an encrypted backup of the default metadata vault to [path].
  /// @nodoc
  VaultBackupManifest backupDefaultVaultInternal(
    String path, {
    bool overwrite = false,
  }) => operations.vaultBackupDefault(path, overwrite);

  /// Restores the default metadata vault from encrypted backup [path].
  /// @nodoc
  VaultBackupManifest restoreDefaultVaultInternal(
    String path, {
    bool overwrite = false,
  }) => operations.vaultRestoreDefault(path, overwrite);
}
