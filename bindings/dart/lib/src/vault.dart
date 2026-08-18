import 'dart:ffi' as ffi;
import 'dart:convert';
import 'dart:typed_data';

import 'package:revault_api/src/agent_session.dart';
import 'package:revault_api/src/approval_models.dart';
import 'package:revault_api/src/contact_key_pair.dart';
import 'package:revault_api/src/contact_public_key.dart';
import 'package:revault_api/src/lockbox.dart';
import 'package:revault_api/src/owned.dart';
import 'package:revault_api/src/profile_signing_key_pair.dart';
import 'package:revault_api/src/profile_signing_public_key.dart';
import 'package:revault_api/src/read_only_vault.dart';
import 'package:revault_api/src/revault.dart';
import 'package:revault_api/src/secret_string.dart';

import 'domain_models.dart';
import 'exceptions.dart';

/// A writable, password-protected metadata store for one reVault installation.
///
/// Open or create it through [Vault] to manage profile keys, contacts, form
/// definitions, encrypted key backups, remembered lockbox paths, and local
/// access-slot labels. Lockbox file contents are stored separately.
///
/// Example:
/// ```dart
/// final passphrase = SecretString.takeUtf8(await prompt.readUtf8());
/// final vault = Vault.open(passphrase: passphrase);
/// try {
///   final lockbox = Lockbox.open('/secrets/team.lbox', vault: vault);
/// } finally {
///   vault.close();
///   passphrase.close();
/// }
/// ```
final class Vault extends Owned {
  /// @nodoc
  Vault.internal(super.runtime, super.handle);

  /// Opens an existing persistent vault.
  ///
  /// When [passphrase] is omitted, reVault retrieves the vault passphrase from
  /// the operating system's credential store. Persisting that passphrase
  /// grants unattended same-user access to every lockbox for which this vault
  /// contains a usable credential. A future platform implementation may gate
  /// retrieval on user presence, such as biometric confirmation, but callers
  /// must not assume that protection is available.
  ///
  /// [root] defaults to [defaultRoot]. This method never creates a vault.
  ///
  /// Example:
  /// ```dart
  /// final vault = Vault.open();
  /// try {
  ///   print(vault.listPrivateKeyNames());
  /// } finally {
  ///   vault.close();
  /// }
  /// ```
  static Vault open({String? root, SecretString? passphrase}) {
    final runtime = Revault.runtime;
    final resolvedRoot = root ?? runtime.defaultVaultDirectoryInternal;
    if (passphrase != null) {
      return runtime.openVaultInternal(resolvedRoot, passphrase);
    }
    late final SecretString stored;
    try {
      stored = SecretString.takeUtf8(
        runtime.operations.vaultPlatformGetPassword(),
      );
    } on RevaultException catch (error) {
      if (error.message.isEmpty) {
        throw const VaultPassphraseUnavailableException();
      }
      rethrow;
    }
    try {
      return runtime.openVaultInternal(resolvedRoot, stored);
    } finally {
      stored.close();
    }
  }

  /// Opens an existing vault or creates it when [root] does not contain one.
  ///
  /// Creation is explicit because it can hide an incorrect path. Omit [root]
  /// to use [defaultRoot]. Unlike [open], a passphrase is always required.
  ///
  /// Example:
  /// ```dart
  /// final passphrase = SecretString.fromString(initialPassphrase);
  /// final vault = Vault.openOrCreate(passphrase: passphrase);
  /// ```
  static Vault openOrCreate({String? root, required SecretString passphrase}) {
    final runtime = Revault.runtime;
    return root == null
        ? runtime.openOrCreateDefaultVaultInternal(passphrase)
        : runtime.openOrCreateVaultInternal(root, passphrase);
  }

  /// Replaces [root] with a new empty vault protected by [passphrase].
  ///
  /// Existing vault data at that location is destroyed. Omit [root] to replace
  /// the platform-default vault.
  ///
  /// Example:
  /// ```dart
  /// final replacement = Vault.replace(
  ///   root: testDirectory,
  ///   passphrase: testPassphrase,
  /// );
  /// replacement.close();
  /// ```
  static Vault replace({String? root, required SecretString passphrase}) {
    final runtime = Revault.runtime;
    return root == null
        ? runtime.replaceDefaultVaultInternal(passphrase)
        : runtime.replaceVaultInternal(root, passphrase);
  }

  /// Opens a metadata-only view that cannot load private keys or mutate data.
  ///
  /// Use this for discovery screens that need names and paths but should not
  /// receive key-management authority.
  ///
  /// Example:
  /// ```dart
  /// final view = Vault.openReadOnly(passphrase: passphrase);
  /// try {
  ///   print(view.listKnownLockboxes());
  /// } finally {
  ///   view.close();
  /// }
  /// ```
  static ReadOnlyVault openReadOnly({
    String? root,
    required SecretString passphrase,
  }) {
    final runtime = Revault.runtime;
    return root == null
        ? runtime.openDefaultReadOnlyVaultInternal(passphrase)
        : runtime.openReadOnlyVaultInternal(root, passphrase);
  }

  /// The platform-default directory containing the persistent Vault file.
  ///
  /// Example:
  /// ```dart
  /// print('Vault directory: ${Vault.defaultRoot}');
  /// ```
  static String get defaultRoot =>
      Revault.runtime.defaultVaultDirectoryInternal;

  /// The platform-default encrypted Vault file.
  ///
  /// Example:
  /// ```dart
  /// if (File(Vault.defaultPath).existsSync()) print('Vault exists');
  /// ```
  static String get defaultPath => Revault.runtime.defaultVaultPathInternal;

  /// Stores [passphrase] in the operating system credential store.
  ///
  /// On platforms without per-use user-presence enforcement, any process
  /// running as this user may be able to retrieve it and open this vault.
  /// Remembering this credential is therefore broad unattended authority, not
  /// merely a convenience flag for one Lockbox.
  ///
  /// Example:
  /// ```dart
  /// Vault.rememberPassphrase(passphrase);
  /// final reopened = Vault.open();
  /// reopened.close();
  /// ```
  static void rememberPassphrase(SecretString passphrase) =>
      _rememberPassphrase(passphrase);

  static void _rememberPassphrase(SecretString passphrase) {
    final operations = Revault.runtime.operations;
    operations.vaultPlatformEnable();
    passphrase.withBytes(operations.vaultPlatformPutPassword);
  }

  /// Removes the vault passphrase from the operating system credential store.
  ///
  /// This neither deletes the vault nor closes lockboxes already held by this
  /// process or by [AgentSession].
  ///
  /// Example:
  /// ```dart
  /// Vault.forgetPassphrase();
  /// assert(Vault.platformCredentialsDisabled);
  /// ```
  static void forgetPassphrase() {
    final operations = Revault.runtime.operations;
    operations.vaultPlatformForgetPassword();
    operations.vaultPlatformDisable();
  }

  /// Reports the operating-system credential-store implementation and policy.
  ///
  /// Use this for settings and diagnostics, not as proof that each retrieval
  /// requires biometric user presence.
  ///
  /// Example:
  /// ```dart
  /// final status = Vault.platformCredentialStatus();
  /// print('${status.backend}: ${status.scope}');
  /// ```
  static PlatformStatus platformCredentialStatus() =>
      Revault.runtime.operations.vaultPlatformStatus();

  /// Whether automatic platform Vault-credential lookup is disabled.
  ///
  /// Example:
  /// ```dart
  /// if (Vault.platformCredentialsDisabled) showUnlockPrompt();
  /// ```
  static bool get platformCredentialsDisabled =>
      Revault.runtime.operations.vaultPlatformDisabled();

  /// Writes an encrypted backup of the default Vault to [destinationPath].
  ///
  /// The backup remains protected by Vault encryption. Existing destinations
  /// require [overwrite].
  ///
  /// Example:
  /// ```dart
  /// final manifest = Vault.backupDefault('/backups/vault.backup');
  /// print(manifest.vaultSha256);
  /// ```
  static VaultBackupManifest backupDefault(
    String destinationPath, {
    bool overwrite = false,
  }) => Revault.runtime.backupDefaultVaultInternal(
    destinationPath,
    overwrite: overwrite,
  );

  /// Restores the default Vault from encrypted [backupPath].
  ///
  /// This changes persistent local security data; [overwrite] must be explicit
  /// when a destination Vault already exists.
  ///
  /// Example:
  /// ```dart
  /// Vault.restoreDefault('/backups/vault.backup', overwrite: true);
  /// ```
  static VaultBackupManifest restoreDefault(
    String backupPath, {
    bool overwrite = false,
  }) => Revault.runtime.restoreDefaultVaultInternal(
    backupPath,
    overwrite: overwrite,
  );

  /// Returns the canonical root directory of this Vault.
  ///
  /// Example:
  /// ```dart
  /// print('Opened Vault at ${vault.root}');
  /// ```
  String get root => runtime.operations.vaultDirectoryRoot(handle);

  /// Returns the on-disk Vault structure version.
  ///
  /// Example:
  /// ```dart
  /// print('Structure version ${vault.structureVersion}');
  /// ```
  int get structureVersion =>
      runtime.operations.vaultDirectoryStructureVersion(handle);

  /// Re-encrypts this Vault using [newPassphrase].
  ///
  /// This does not automatically replace a passphrase previously stored by
  /// [rememberPassphrase]; update or forget that credential separately.
  ///
  /// Example:
  /// ```dart
  /// vault.changePassphrase(oldPassphrase, newPassphrase);
  /// Vault.rememberPassphrase(newPassphrase);
  /// ```
  void changePassphrase(
    SecretString currentPassphrase,
    SecretString newPassphrase,
  ) => currentPassphrase.withBytes(
    (currentBytes) => newPassphrase.withBytes(
      (newBytes) => runtime.operations.vaultDirectoryChangePassword(
        root,
        currentBytes,
        newBytes,
      ),
    ),
  );

  /// Lists serialized private-key records for backup and audit workflows.
  ///
  /// Use [listPrivateKeyNames] when the application only needs profile names.
  ///
  /// Example:
  /// ```dart
  /// for (final record in vault.listPrivateKeys()) secureArchive.add(record);
  /// ```
  List<String> listPrivateKeys() =>
      runtime.operations.vaultDirectoryListPrivateKeys(handle);

  /// Lists profile names that have private contact keys.
  ///
  /// Example:
  /// ```dart
  /// final profiles = vault.listPrivateKeyNames();
  /// ```
  List<String> listPrivateKeyNames() =>
      runtime.operations.vaultDirectoryListPrivateKeyNames(handle);

  /// Lists names in the Vault's public-contact address book.
  ///
  /// Example:
  /// ```dart
  /// for (final name in vault.listContactNames()) print(name);
  /// ```
  List<String> listContactNames() =>
      runtime.operations.vaultDirectoryListContactNames(handle);

  /// Lists aliases of the current Vault-wide form definitions.
  ///
  /// Example:
  /// ```dart
  /// final availableForms = vault.listFormAliases();
  /// ```
  List<String> listFormAliases() =>
      runtime.operations.vaultDirectoryListFormAliases(handle);

  /// Whether a profile private key exists under [name].
  ///
  /// Example:
  /// ```dart
  /// if (!vault.privateKeyExists('default')) createDefaultProfile(vault);
  /// ```
  bool privateKeyExists(String name) =>
      runtime.operations.vaultDirectoryPrivateKeyExists(handle, name);

  /// Deletes the contact-decryption and signing keys for Profile [name].
  ///
  /// This can permanently remove the ability to open recipient-authorized
  /// Lockboxes unless a backup or older generation remains available.
  ///
  /// Example:
  /// ```dart
  /// if (confirmed) vault.deletePrivateKey('retired-profile');
  /// ```
  void deletePrivateKey(String name) =>
      runtime.operations.vaultDirectoryDeletePrivateKey(handle, name);

  /// Stores [key] as the active private contact identity for profile [name].
  ///
  /// The Vault copies the key; the caller should still dispose its handle.
  ///
  /// Example:
  /// ```dart
  /// final key = revault.generateContactKeyPair();
  /// try {
  ///   vault.storePrivateKey('alice', key);
  /// } finally {
  ///   key.dispose();
  /// }
  /// ```
  void storePrivateKey(String name, ContactKeyPair key) => runtime.operations
      .vaultDirectoryStorePrivateKey(handle, name, key.handle);

  /// Loads the active private contact identity for profile [name].
  ///
  /// Dispose the returned private-key handle.
  ///
  /// Example:
  /// ```dart
  /// final key = vault.loadPrivateKey('alice');
  /// try {
  ///   final box = Lockbox.open(path, contact: key);
  /// } finally {
  ///   key.dispose();
  /// }
  /// ```
  ContactKeyPair loadPrivateKey(String name) => ContactKeyPair.internal(
    runtime,
    runtime.operations.vaultDirectoryLoadPrivateKey(handle, name),
  );

  /// Loads historical private-key generation [index] for profile [name].
  ///
  /// Use this when opening an archive granted to a retired key generation.
  ///
  /// Example:
  /// ```dart
  /// final oldKey = vault.loadPrivateKeyGeneration('alice', 1);
  /// try {
  ///   final oldBox = Lockbox.open(path, contact: oldKey);
  /// } finally {
  ///   oldKey.dispose();
  /// }
  /// ```
  ContactKeyPair loadPrivateKeyGeneration(String name, int index) =>
      ContactKeyPair.internal(
        runtime,
        runtime.operations.vaultDirectoryLoadPrivateKeyGeneration(
          handle,
          name,
          index,
        ),
      );

  /// Stores [key] as the public encryption key for contact [name].
  ///
  /// Example:
  /// ```dart
  /// final alice = revault.importContactPublicKey(alicePem);
  /// try {
  ///   vault.storeContact('alice', alice);
  /// } finally {
  ///   alice.dispose();
  /// }
  /// ```
  void storeContact(String name, ContactPublicKey key) =>
      runtime.operations.vaultDirectoryStoreContact(handle, name, key.handle);

  /// Loads the public encryption key for contact [name].
  ///
  /// Example:
  /// ```dart
  /// final alice = vault.loadContact('alice');
  /// try {
  ///   lockbox.addContact(alice, 'alice');
  /// } finally {
  ///   alice.dispose();
  /// }
  /// ```
  ContactPublicKey loadContact(String name) => ContactPublicKey.internal(
    runtime,
    runtime.operations.vaultDirectoryLoadContact(handle, name),
  );

  /// Whether the public-contact address book contains [name].
  ///
  /// Example:
  /// ```dart
  /// if (vault.contactExists('alice')) showContact('alice');
  /// ```
  bool contactExists(String name) =>
      runtime.operations.vaultDirectoryContactExists(handle, name);

  /// Deletes contact [name] from the local address book.
  ///
  /// Existing Lockbox access slots are unchanged.
  ///
  /// Example:
  /// ```dart
  /// vault.deleteContact('former-contractor');
  /// ```
  void deleteContact(String name) =>
      runtime.operations.vaultDirectoryDeleteContact(handle, name);

  /// Lists stored contacts with their serialized public-key records.
  ///
  /// Example:
  /// ```dart
  /// for (final contact in vault.listContacts()) print(contact.name);
  /// ```
  List<Contact> listContacts() =>
      runtime.operations.vaultDirectoryListContacts(handle);

  /// Enrolls [device] using public data from an authenticated pairing transcript.
  ///
  /// The phone recipient private key remains on the phone. Adding this record
  /// does not grant it access to any Lockbox.
  ///
  /// Example:
  /// ```dart
  /// vault.storeApprovalDevice(scannedDevice);
  /// ```
  void storeApprovalDevice(ApprovalDevice device) =>
      runtime.operations.vaultDirectoryStoreDeviceJson(handle, device.toJson());

  /// Lists active and revoked phone enrollment records.
  ///
  /// Example:
  /// ```dart
  /// for (final device in vault.listApprovalDevices()) print(device.name);
  /// ```
  List<ApprovalDevice> listApprovalDevices() =>
      (jsonDecode(runtime.operations.vaultDirectoryListDevicesJson(handle))
              as List<Object?>)
          .map(
            (record) => ApprovalDevice.fromJson(
              (record! as Map<Object?, Object?>).cast<String, Object?>(),
            ),
          )
          .toList(growable: false);

  /// Marks an enrolled phone revoked in the Vault administration record.
  ///
  /// This does not itself rotate Lockbox content keys. The caller must use the
  /// Lockbox revocation workflow for every granted archive.
  ///
  /// Example:
  /// ```dart
  /// vault.revokeApprovalDevice(lostPhone.id);
  /// ```
  void revokeApprovalDevice(Uint8List deviceId) =>
      runtime.operations.vaultDirectoryRevokeDevice(handle, deviceId);

  /// Stores a new local or CI source policy.
  ///
  /// Example:
  /// ```dart
  /// vault.storeApprovalSource(githubProductionSource);
  /// ```
  void storeApprovalSource(ApprovalSource source) => runtime.operations
      .vaultDirectoryStoreApprovalSourceJson(handle, source.toJson());

  /// Replaces the policy of an existing source with the same stable ID.
  ///
  /// Example:
  /// ```dart
  /// vault.updateApprovalSource(restrictedSource);
  /// ```
  void updateApprovalSource(ApprovalSource source) => runtime.operations
      .vaultDirectoryUpdateApprovalSourceJson(handle, source.toJson());

  /// Lists active and revoked local and CI approval sources.
  ///
  /// Example:
  /// ```dart
  /// for (final source in vault.listApprovalSources()) print(source.name);
  /// ```
  List<ApprovalSource> listApprovalSources() =>
      (jsonDecode(
                runtime.operations.vaultDirectoryListApprovalSourcesJson(
                  handle,
                ),
              )
              as List<Object?>)
          .map(
            (record) => ApprovalSource.fromJson(
              (record! as Map<Object?, Object?>).cast<String, Object?>(),
            ),
          )
          .toList(growable: false);

  /// Revokes a source policy by its stable 16-byte ID.
  ///
  /// Example:
  /// ```dart
  /// vault.revokeApprovalSource(retiredPipeline.id);
  /// ```
  void revokeApprovalSource(Uint8List sourceId) =>
      runtime.operations.vaultDirectoryRevokeApprovalSource(handle, sourceId);

  /// Stores the contact [email] associated with local profile [name].
  ///
  /// This is descriptive metadata, not part of the cryptographic identity.
  ///
  /// Example:
  /// ```dart
  /// vault.storeProfileEmail('alice', 'alice@example.com');
  /// ```
  void storeProfileEmail(String name, String email) =>
      runtime.operations.vaultDirectoryStoreProfileEmail(handle, name, email);

  /// Returns the email associated with profile [name], when present.
  ///
  /// Example:
  /// ```dart
  /// final email = vault.profileEmail('alice');
  /// ```
  String? profileEmail(String name) =>
      runtime.operations.vaultDirectoryProfileEmail(handle, name);

  /// Stores [encryptedBackup] under Lockbox [id].
  ///
  /// This recovery record contains encrypted access metadata, not a raw
  /// content key.
  ///
  /// Example:
  /// ```dart
  /// vault.storeBackup(lockbox.id, encryptedKeyDirectoryBackup);
  /// ```
  void storeBackup(Uint8List id, Uint8List encryptedBackup) =>
      runtime.operations.vaultDirectoryStoreBackup(handle, id, encryptedBackup);

  /// Loads the encrypted key-directory backup for Lockbox [id].
  ///
  /// Example:
  /// ```dart
  /// final backup = vault.loadBackup(lockboxId);
  /// ```
  Uint8List loadBackup(Uint8List id) =>
      runtime.operations.vaultDirectoryLoadBackup(handle, id);

  /// Returns the number of stored key-directory recovery backups.
  ///
  /// Example:
  /// ```dart
  /// print('${vault.backupCount} Lockbox key backups');
  /// ```
  int get backupCount => runtime.operations.vaultDirectoryBackupCount(handle);

  /// Restores contact-decryption [profileKey] and [profileSigningKey] records
  /// for a Profile.
  ///
  /// Existing profile material requires [overwrite].
  ///
  /// Example:
  /// ```dart
  /// vault.restorePrivateKey(
  ///   'alice',
  ///   restoredContactKey,
  ///   restoredSigningKey,
  ///   overwrite: true,
  /// );
  /// ```
  void restorePrivateKey(
    String name,
    ContactKeyPair profileKey,
    ProfileSigningKeyPair profileSigningKey, {
    bool overwrite = false,
  }) => runtime.operations.vaultDirectoryRestorePrivateKey(
    handle,
    name,
    profileKey.handle,
    profileSigningKey.handle,
    overwrite,
  );

  /// Loads the active signing key for Profile [name].
  ///
  /// Example:
  /// ```dart
  /// final profileSigningKey = vault.loadProfileSigningKey('alice');
  /// try {
  ///   lockbox.setOwnerSigningKey(profileSigningKey);
  /// } finally {
  ///   profileSigningKey.dispose();
  /// }
  /// ```
  ProfileSigningKeyPair loadProfileSigningKey(String name) =>
      ProfileSigningKeyPair.internal(
        runtime,
        runtime.operations.vaultDirectoryLoadOwnerSigningKey(handle, name),
      );

  /// Loads historical Profile-signing generation [index] for [name].
  ///
  /// Example:
  /// ```dart
  /// final priorKey = vault.loadProfileSigningKeyGeneration('alice', 1);
  /// ```
  ProfileSigningKeyPair loadProfileSigningKeyGeneration(
    String name,
    int index,
  ) => ProfileSigningKeyPair.internal(
    runtime,
    runtime.operations.vaultDirectoryLoadOwnerSigningKeyGeneration(
      handle,
      name,
      index,
    ),
  );

  /// Stores a contact's public verification [key] under [name].
  ///
  /// Use it to associate owner-signature verification material with a contact.
  ///
  /// Example:
  /// ```dart
  /// vault.storeContactSigningKey('alice', aliceProfileSigningPublicKey);
  /// ```
  void storeContactSigningKey(String name, ProfileSigningPublicKey key) =>
      runtime.operations.vaultDirectoryStoreContactSigningKey(
        handle,
        name,
        key.handle,
      );

  /// Loads the public verification key stored for contact [name].
  ///
  /// Example:
  /// ```dart
  /// final owner = vault.loadContactSigningKey('alice');
  /// try {
  ///   inspectOwner(owner);
  /// } finally {
  ///   owner.dispose();
  /// }
  /// ```
  ProfileSigningPublicKey loadContactSigningKey(String name) =>
      ProfileSigningPublicKey.internal(
        runtime,
        runtime.operations.vaultDirectoryLoadContactSigningKey(handle, name),
      );

  /// Lists active and retired key generations for profile [name].
  ///
  /// Example:
  /// ```dart
  /// final history = vault.listProfileGenerations('alice');
  /// print('Active generation: ${history.activeGeneration}');
  /// ```
  ProfileHistory listProfileGenerations(String name) =>
      runtime.operations.vaultDirectoryListProfileGenerations(handle, name);

  /// Rotates profile [name] to newly generated contact and signing keys.
  ///
  /// Old generations remain available for opening existing Lockboxes; new
  /// grants should use the returned active generation.
  ///
  /// Example:
  /// ```dart
  /// final history = vault.rotatePrivateKey('alice');
  /// backupRotatedProfile(history);
  /// ```
  ProfileHistory rotatePrivateKey(String name) =>
      runtime.operations.vaultDirectoryRotatePrivateKey(handle, name);

  /// Remembers that Lockbox [id] is stored at host [path].
  ///
  /// This is discovery metadata only; it does not cache a content key or open
  /// the Lockbox.
  ///
  /// Example:
  /// ```dart
  /// vault.rememberLockbox(lockbox.id, '/secrets/team.lbox');
  /// ```
  void rememberLockbox(Uint8List id, String path) =>
      runtime.operations.vaultDirectoryRememberLockbox(handle, id, path);

  /// Lists remembered Lockbox identifiers and host paths.
  ///
  /// Example:
  /// ```dart
  /// for (final known in vault.listKnownLockboxes()) print(known.path);
  /// ```
  List<KnownLockbox> listKnownLockboxes() =>
      runtime.operations.vaultDirectoryListKnownLockboxes(handle);

  /// Forgets the Lockbox discovery record at host [path].
  ///
  /// This does not delete or close the Lockbox and does not remove an agent key.
  ///
  /// Example:
  /// ```dart
  /// vault.forgetLockbox('/secrets/moved.lbox');
  /// ```
  void forgetLockbox(String path) =>
      runtime.operations.vaultDirectoryForgetLockbox(handle, path);

  /// Stores local display [name] for one Lockbox access slot.
  ///
  /// Labels are Vault-local because contact names are not embedded in portable
  /// Lockbox metadata.
  ///
  /// Example:
  /// ```dart
  /// vault.rememberAccessSlotLabel(lockbox.id, slotId, 'alice');
  /// ```
  void rememberAccessSlotLabel(Uint8List id, int slotId, String name) => runtime
      .operations
      .vaultDirectoryRememberAccessSlotLabel(handle, id, slotId, name);

  /// Lists local access-slot labels for Lockbox [id].
  ///
  /// Example:
  /// ```dart
  /// final labels = vault.listAccessSlotLabels(lockbox.id);
  /// ```
  List<AccessSlotLabel> listAccessSlotLabels(Uint8List id) =>
      runtime.operations.vaultDirectoryListAccessSlotLabels(handle, id);

  /// Finds access-slot labels named [name] for Lockbox [id].
  ///
  /// Example:
  /// ```dart
  /// final aliceSlots = vault.findAccessSlotLabels(lockbox.id, 'alice');
  /// ```
  List<AccessSlotLabel> findAccessSlotLabels(Uint8List id, String name) =>
      runtime.operations.vaultDirectoryFindAccessSlotLabels(handle, id, name);

  /// Deletes the local label for one Lockbox access slot.
  ///
  /// This does not revoke the corresponding access slot; use
  /// [Lockbox.deleteKey] for revocation.
  ///
  /// Example:
  /// ```dart
  /// vault.forgetAccessSlotLabel(lockbox.id, slotId);
  /// ```
  void forgetAccessSlotLabel(Uint8List id, int slotId) => runtime.operations
      .vaultDirectoryForgetAccessSlotLabel(handle, id, slotId);

  /// Defines a new immutable revision of a Vault-wide form.
  ///
  /// Vault-wide forms can be reused when creating records in multiple
  /// Lockboxes.
  ///
  /// Example:
  /// ```dart
  /// final login = vault.defineForm('login', 'Login', 'Service login', fields);
  /// ```
  FormDefinition defineForm(
    String alias,
    String name,
    String description,
    List<FormField> fields,
  ) => runtime.operations.vaultDirectoryDefineForm(
    handle,
    alias,
    name,
    description,
    DomainDecoders.formFields(fields),
  );

  /// Resolves a Vault-wide form alias or stable type identifier.
  ///
  /// Example:
  /// ```dart
  /// final login = vault.resolveForm('login');
  /// ```
  FormDefinition resolveForm(String reference) =>
      runtime.operations.vaultDirectoryResolveForm(handle, reference);

  /// Lists current Vault-wide form definitions.
  ///
  /// Example:
  /// ```dart
  /// for (final form in vault.listForms()) print(form.alias);
  /// ```
  List<FormDefinition> listForms() =>
      runtime.operations.vaultDirectoryListForms(handle);

  /// Lists every Vault-wide revision for [typeId].
  ///
  /// Example:
  /// ```dart
  /// final revisions = vault.listFormRevisions(login.typeId);
  /// ```
  List<FormDefinition> listFormRevisions(String typeId) =>
      runtime.operations.vaultDirectoryListFormRevisions(handle, typeId);

  /// Installs built-in form definitions that are not already present.
  ///
  /// The returned count is the number newly installed; calling it repeatedly is
  /// safe.
  ///
  /// Example:
  /// ```dart
  /// print('Installed ${vault.seedForms()} built-in forms');
  /// ```
  int seedForms() => runtime.operations.vaultDirectorySeedForms(handle);

  /// Encrypts and remembers a Lockbox [password] inside this Vault under [id].
  ///
  /// This is not an independent platform credential. Opening the password
  /// later requires this Vault to be open, either explicitly or through its
  /// platform-stored Vault passphrase.
  ///
  /// Example:
  /// ```dart
  /// vault.rememberPassword(lockbox.id, lockboxPassword);
  /// ```
  void rememberPassword(Uint8List id, SecretString password) =>
      password.withBytes(
        (bytes) => runtime.operations.vaultDirectoryRememberPassword(
          handle,
          id,
          bytes,
        ),
      );

  /// Returns the password remembered inside this Vault for Lockbox [id].
  ///
  /// The caller owns the returned [SecretString] and must close it.
  ///
  /// Example:
  /// ```dart
  /// final password = vault.rememberedPassword(lockboxId);
  /// try {
  ///   final box = Lockbox.open(path, password: password);
  /// } finally {
  ///   password.close();
  /// }
  /// ```
  SecretString rememberedPassword(Uint8List id) => SecretString.takeUtf8(
    runtime.operations.vaultDirectoryRememberedPassword(handle, id),
  );

  /// Wipes decrypted state and releases the writable Vault handle.
  ///
  /// This does not close Lockboxes or stop the agent.
  ///
  /// Example:
  /// ```dart
  /// try {
  ///   useVault(vault);
  /// } finally {
  ///   vault.close();
  /// }
  /// ```
  void close() {
    if (!disposed) {
      runtime.operations.vaultDirectoryFree(handle);
      handle = ffi.nullptr;
    }
  }

  /// Deprecated alias for [close].
  ///
  /// Example:
  /// ```dart
  /// vault.dispose(); // New code uses vault.close().
  /// ```
  @Deprecated('Use close().')
  void dispose() => close();
}
