package com.onepub.revault;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.nio.charset.StandardCharsets;

/**
 * Entry point for encrypted lockboxes, keys, local vault metadata, the session
 * agent, and the platform secret store.
 *
 * <p>Create one when the application starts, then use it to open lockboxes and
 * manage keys and local services. Owned objects implement {@link AutoCloseable}. Secret variables and form
 * fields are exposed through callback-scoped methods. See the
 * <a href="https://github.com/onepub-dev/reVault#readme">repository README</a>
 * for installation and examples.
 */
public final class Revault {
  /** Consumes temporary secret bytes before their transfer copy is wiped. */
  @FunctionalInterface
  public interface SecretCallback<T> {
    /** Uses temporary secret bytes that must not be retained after this call. */
    T use(byte[] secret);
  }

  private final BindingOperations operations;

  /** Returns the revault. */
  public Revault() { this(NativeLibrary.resolve()); }

  /** Returns the revault. */
  public Revault(String libraryPath) {
    this(new RevaultNativeApi(SymbolLookup.libraryLookup(libraryPath, Arena.global())));
  }

  Revault(RevaultNativeApi nativeApi) { operations = new BindingOperations(nativeApi); }

  /**
   * Runtime cache and worker tuning for opening or creating lockboxes.
   *
   * @param cacheMode cache strategy, such as {@code bytes}
   * @param cacheBytes maximum cache capacity in bytes
   * @param workload workload profile, such as {@code interactive}
   * @param worker worker-selection policy, such as {@code auto}
   * @param jobs worker count; zero lets the library select it
   */
  public record LockboxOptions(String cacheMode, long cacheBytes, String workload, String worker, long jobs) {
    /** Returns the recommended interactive runtime defaults. */
    public static LockboxOptions defaults() {
      return new LockboxOptions("bytes", 64L << 20, "interactive", "auto", 0);
    }
  }

  private static void ensureOpen(MemorySegment handle) {
    if (handle == null || handle.address() == 0) throw new IllegalStateException("object is closed");
  }

  /** Returns the last error. */
  public String lastError() { return operations.lastErrorMessage(); }
  /** Returns the last error details. */
  public ErrorDetails lastErrorDetails() { return operations.bufferLastErrorDetails(); }
  /** Returns the lockbox format version. */
  public int lockboxFormatVersion() { return operations.lockboxFormatVersion(); }
  /** Determines lockbox format version without fully opening it. */
  public int probeLockboxFormatVersion(byte[] value) { return operations.lockboxProbeFormatVersion(value); }
  /** Returns the current vault structure version. */
  public int currentVaultStructureVersion() { return operations.vaultStructureVersionCurrent(); }
  /** Determines vault structure version without fully opening it. */
  public int probeVaultStructureVersion(String root, byte[] password) { return operations.vaultDirectoryProbeStructureVersion(root, password); }

  /** A recipient's shareable encryption identity used when granting lockbox access. */
  public final class ContactPublicKey implements AutoCloseable {
    private MemorySegment handle;
    private ContactPublicKey(MemorySegment handle) { this.handle = handle; }
    /** Exports this key in the requested format. */
    public byte[] export(String format) { ensureOpen(handle); return operations.vaultKeyExportPublic(handle, format); }
    /** Returns the stable fingerprint of this key. */
    public byte[] fingerprint() { ensureOpen(handle); return operations.vaultKeyFingerprint(handle); }
    /** Encrypts a content key for the selected contact. */
    public WrappedContactKey encrypt(byte[] contentKey) {
      ensureOpen(handle); return new WrappedContactKey(operations.keyContactEncrypt(handle, contentKey));
    }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.keyContactPublicFree(handle); handle = null; }
    }
  }

  /** A content key encrypted for one contact and recoverable only by its matching key pair. */
  public final class WrappedContactKey implements AutoCloseable {
    private MemorySegment handle;
    private WrappedContactKey(MemorySegment handle) { this.handle = handle; }
    /** Returns the public bytes. */
    public byte[] publicBytes() { ensureOpen(handle); return operations.keyContactWrappedPublic(handle); }
    /** Returns the ciphertext. */
    public byte[] ciphertext() { ensureOpen(handle); return operations.keyContactWrappedCiphertext(handle); }
    /** Returns the encrypted bytes. */
    public byte[] encryptedBytes() { ensureOpen(handle); return operations.keyContactWrappedEncrypted(handle); }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.keyContactWrappedFree(handle); handle = null; }
    }
  }

  /** A profile's contact-encryption identity used to decrypt content keys addressed to it. */
  public final class ContactKeyPair implements AutoCloseable {
    private MemorySegment handle;
    private ContactKeyPair(MemorySegment handle) { this.handle = handle; }
    /** Returns the public bytes. */
    public byte[] publicBytes() { ensureOpen(handle); return operations.keyContactPublic(handle); }
    /** Returns the private record. */
    public byte[] privateRecord() { ensureOpen(handle); return operations.keyContactPrivate(handle); }
    /** Returns the public key. */
    public ContactPublicKey publicKey() { return contactPublicKey(publicBytes()); }
    /** Exports this key in the requested format. */
    public byte[] export(String format) { ensureOpen(handle); return operations.vaultKeyExportPrivate(handle, format); }
    /** Decrypts a wrapped content key for this contact. */
    public byte[] decrypt(WrappedContactKey wrapped) {
      ensureOpen(handle); ensureOpen(wrapped.handle); return operations.keyContactDecrypt(handle, wrapped.handle);
    }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.keyContactFree(handle); handle = null; }
    }
  }

  /** The public identity readers use to verify owner-authorized lockbox revisions. */
  public final class SigningPublicKey implements AutoCloseable {
    private MemorySegment handle;
    private SigningPublicKey(MemorySegment handle) { this.handle = handle; }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.keySigningPublicFree(handle); handle = null; }
    }
  }

  /** A lockbox owner's signing identity used to authorize mutable revisions. */
  public final class SigningKeyPair implements AutoCloseable {
    private MemorySegment handle;
    private SigningKeyPair(MemorySegment handle) { this.handle = handle; }
    /** Returns the public bytes. */
    public byte[] publicBytes() { ensureOpen(handle); return operations.keySigningPublic(handle); }
    /** Returns the private record. */
    public byte[] privateRecord() { ensureOpen(handle); return operations.keySigningPrivate(handle); }
    /** Returns the public key. */
    public SigningPublicKey publicKey() { return signingPublicKey(publicBytes()); }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.keySigningFree(handle); handle = null; }
    }
  }

  /** Generates contact key pair. */
  public ContactKeyPair generateContactKeyPair() { return new ContactKeyPair(operations.keyContactGenerate()); }
  /** Returns the contact key pair from private. */
  public ContactKeyPair contactKeyPairFromPrivate(byte[] value) { return new ContactKeyPair(operations.keyContactFromPrivate(value)); }
  /** Imports contact key pair. */
  public ContactKeyPair importContactKeyPair(byte[] value) { return new ContactKeyPair(operations.vaultKeyImportPrivate(value)); }
  /** Returns the contact public key. */
  public ContactPublicKey contactPublicKey(byte[] value) { return new ContactPublicKey(operations.keyContactPublicFromBytes(value)); }
  /** Imports contact public key. */
  public ContactPublicKey importContactPublicKey(byte[] value) { return new ContactPublicKey(operations.vaultKeyImportPublic(value)); }
  /** Generates signing key pair. */
  public SigningKeyPair generateSigningKeyPair() { return new SigningKeyPair(operations.keySigningGenerate()); }
  /** Returns the signing key pair from private. */
  public SigningKeyPair signingKeyPairFromPrivate(byte[] value) { return new SigningKeyPair(operations.keySigningFromPrivate(value)); }
  /** Returns the signing public key. */
  public SigningPublicKey signingPublicKey(byte[] value) { return new SigningPublicKey(operations.keySigningPublicFromBytes(value)); }

  /** Formats key hex. */
  public String formatKeyHex(byte[] value) { return operations.vaultKeyFormatHex(value); }
  /** Decodes key hex. */
  public byte[] decodeKeyHex(String value) { return operations.vaultKeyDecodeHex(value); }
  /** Formats key crockford. */
  public String formatKeyCrockford(byte[] value) { return operations.vaultKeyFormatCrockford(value); }
  /** Formats key crockford reading. */
  public String formatKeyCrockfordReading(String value) { return operations.vaultKeyFormatCrockfordReading(value); }
  /** Decodes key crockford. */
  public byte[] decodeKeyCrockford(String value) { return operations.vaultKeyDecodeCrockford(value); }
  /** Returns the hex encode. */
  public String hexEncode(byte[] value) { return operations.vaultKeyHexEncode(value); }
  /** Returns the hex decode. */
  public byte[] hexDecode(String value) { return operations.vaultKeyHexDecode(value); }

  /** Creates lockbox. */
  public Lockbox createLockbox(byte[] key) { return new Lockbox(operations.lockboxCreate(key)); }
  /** Creates lockbox. */
  public Lockbox createLockbox(byte[] key, LockboxOptions options) {
    return new Lockbox(operations.lockboxCreateWithOptions(key, options.cacheMode(), options.cacheBytes(),
        options.workload(), options.worker(), options.jobs()));
  }
  /** Creates lockbox with password. */
  public Lockbox createLockboxWithPassword(byte[] password) { return new Lockbox(operations.lockboxCreatePassword(password)); }
  /** Creates lockbox for contact. */
  public Lockbox createLockboxForContact(ContactPublicKey contact) {
    ensureOpen(contact.handle); return new Lockbox(operations.lockboxCreateContact(contact.handle));
  }
  /** Creates signed lockbox. */
  public Lockbox createSignedLockbox(byte[] contentKey, SigningKeyPair signingKey) {
    ensureOpen(signingKey.handle); return new Lockbox(operations.lockboxCreateWithSigningKey(contentKey, signingKey.handle));
  }
  /** Opens lockbox. */
  public Lockbox openLockbox(byte[] archive, byte[] key) { return new Lockbox(operations.lockboxOpen(archive, key)); }
  /** Opens lockbox. */
  public Lockbox openLockbox(byte[] archive, byte[] key, LockboxOptions options) {
    return new Lockbox(operations.lockboxOpenWithOptions(archive, key, options.cacheMode(),
        options.cacheBytes(), options.workload(), options.worker(), options.jobs()));
  }
  /** Opens lockbox with password. */
  public Lockbox openLockboxWithPassword(byte[] archive, byte[] password) {
    return new Lockbox(operations.lockboxOpenPassword(archive, password));
  }
  /** Opens lockbox for contact. */
  public Lockbox openLockboxForContact(byte[] archive, ContactKeyPair contact) {
    ensureOpen(contact.handle); return new Lockbox(operations.lockboxOpenContact(archive, contact.handle));
  }

  /** Inspects lockbox file. */
  public FileInspection inspectLockboxFile(String path) { return operations.lockboxInspectFile(path); }
  /** Scans lockbox path. */
  public RecoveryReport scanLockboxPath(String path, byte[] key) { return operations.lockboxRecoveryScanPath(path, key); }
  /** Scans lockbox. */
  public RecoveryReport scanLockbox(byte[] archive, byte[] key) { return operations.lockboxRecoveryScan(archive, key); }
  /** Salvages lockbox. */
  public Lockbox salvageLockbox(byte[] archive, byte[] key, SigningKeyPair signingKey) {
    return new Lockbox(operations.lockboxRecoverySalvage(archive, key,
        signingKey == null ? MemorySegment.NULL : signingKey.handle));
  }

  /** An open encrypted archive containing files, variables, secrets, and forms. */
  public final class Lockbox implements AutoCloseable {
    private MemorySegment handle;
    private Lockbox(MemorySegment handle) { this.handle = handle; }
    /** Adds file. */
    public void addFile(String path, byte[] value, boolean replace) { operations.lockboxAddFile(handle, path, value, replace); }
    /** Adds file. */
    public void addFile(String path, byte[] value, int permissions, boolean replace) { operations.lockboxAddFileWithPermissions(handle, path, value, permissions, replace); }
    /** Returns file. */
    public byte[] getFile(String path) { return operations.lockboxGetFile(handle, path); }
    /** Extracts file. */
    public void extractFile(String source, String destination, boolean replace) { operations.lockboxExtractFile(handle, source, destination, replace); }
    /** Extracts directory. */
    public void extractDirectory(String destination, long maxFileBytes, long maxTotalBytes, long maxFiles,
        boolean restoreSymlinks, boolean restorePermissions, boolean overwrite) {
      operations.lockboxExtractDirectory(handle, destination, maxFileBytes, maxTotalBytes, maxFiles,
          restoreSymlinks, restorePermissions, overwrite);
    }
    /** Returns the stream content. */
    public java.util.List<StreamChunk> streamContent(boolean physical) { return operations.lockboxStreamContent(handle, physical); }
    /** Returns cache statistics for this lockbox. */
    public CacheStats cacheStats() { return operations.lockboxCacheStats(handle); }
    /** Returns import statistics for this lockbox. */
    public ImportStats importStats() { return operations.lockboxImportStats(handle); }
    /** Updates import stats. */
    public void resetImportStats() { operations.lockboxResetImportStats(handle); }
    /** Returns the page inspection. */
    public java.util.List<PageInspection> pageInspection() { return operations.lockboxPageInspection(handle); }
    /** Returns the recovery report. */
    public RecoveryReport recoveryReport() { return operations.lockboxRecoveryReport(handle); }
    /** Returns the render recovery report. */
    public String renderRecoveryReport(boolean verbose, long maxEntries) { return operations.lockboxRecoveryReportRender(handle, verbose, maxEntries); }
    /** Returns the storage length. */
    public long storageLength() { return operations.lockboxStorageLen(handle); }
    /** Sets workload profile. */
    public void setWorkloadProfile(String profile) { operations.lockboxSetWorkloadProfile(handle, profile); }
    /** Sets worker policy. */
    public void setWorkerPolicy(String mode, long jobs) { operations.lockboxSetWorkerPolicy(handle, mode, jobs); }
    /** Returns the runtime options. */
    public RuntimeOptions runtimeOptions() { return operations.lockboxRuntimeOptions(handle); }
    /** Authenticates and publishes the staged changes. */
    public void commit() { operations.lockboxCommit(handle); }
    /** Creates directory. */
    public void createDirectory(String path, boolean parents) { operations.lockboxCreateDir(handle, path, parents); }
    /** Removes delete. */
    public void delete(String path) { operations.lockboxDelete(handle, path); }
    /** Removes directory. */
    public void removeDirectory(String path, boolean recursive) { operations.lockboxRemoveDir(handle, path, recursive); }
    /** Creates parent directories. */
    public void createParentDirectories(String path) { operations.lockboxCreateParentDirs(handle, path); }
    /** Updates rename. */
    public void rename(String from, String to) { operations.lockboxRename(handle, from, to); }
    /** Lists list. */
    public java.util.List<LockboxEntry> list(String path, boolean recursive) { return operations.lockboxList(handle, path, recursive); }
    /** Lists list. */
    public java.util.List<LockboxEntry> list(String path, String glob, boolean recursive, boolean includeFiles,
        boolean includeSymlinks, boolean includeDirectories, long limit) {
      return operations.lockboxListWithOptions(handle, path, glob, recursive, includeFiles,
          includeSymlinks, includeDirectories, limit);
    }
    /** Returns metadata for the selected lockbox entry. */
    public LockboxEntry stat(String path) { return operations.lockboxStat(handle, path); }
    /** Sets variable. */
    public void setVariable(String name, String value) { operations.lockboxSetVariable(handle, name, value); }
    /** Stores a secret variable from mutable bytes. */
    public void setSecretVariable(String name, byte[] value) { operations.lockboxSetSecretVariable(handle, name, value); }
    /** Returns variable. */
    public String getVariable(String name) { return operations.lockboxGetVariable(handle, name); }
    /** Invokes {@code callback} with temporary secret bytes, then wipes the transfer buffer. */
    public <T> T withSecretVariable(String name, SecretCallback<T> callback) { return operations.lockboxWithSecretVariable(handle, name, callback); }
    /** Removes variable. */
    public void deleteVariable(String name) { operations.lockboxDeleteVariable(handle, name); }
    /** Updates variables. */
    public void moveVariables(java.util.List<PathMove> moves) { operations.lockboxMoveVariables(handle, DomainCodec.encodePathMoves(moves)); }
    /** Lists variables. */
    public java.util.List<Variable> listVariables() { return operations.lockboxListVariables(handle); }
    /** Returns the variable sensitivity. */
    public String variableSensitivity(String name) { return operations.lockboxVariableSensitivity(handle, name); }
    /** Adds symlink. */
    public void addSymlink(String path, String target, boolean replace) { operations.lockboxAddSymlink(handle, path, target, replace); }
    /** Returns the symlink target. */
    public String symlinkTarget(String path) { return operations.lockboxGetSymlinkTarget(handle, path); }
    /** Returns the id. */
    public byte[] id() { return operations.lockboxId(handle); }
    /** Reports whether exists. */
    public boolean exists(String path) { return operations.lockboxExists(handle, path); }
    /** Reports whether directory. */
    public boolean isDirectory(String path) { return operations.lockboxIsDir(handle, path); }
    /** Returns the permissions. */
    public int permissions(String path) { return operations.lockboxPermissions(handle, path); }
    /** Sets permissions. */
    public void setPermissions(String path, int value) { operations.lockboxSetPermissions(handle, path, value); }
    /** Returns range. */
    public byte[] readRange(String path, long offset, long length) { return operations.lockboxReadRange(handle, path, offset, length); }
    /** Adds password. */
    public long addPassword(byte[] password) {
      long result = operations.lockboxAddPassword(handle, password);
      if (result == -1L) throw new IllegalStateException(operations.lastErrorMessage());
      return result;
    }
    /** Adds contact. */
    public long addContact(ContactPublicKey contact, String name) {
      long result = operations.lockboxAddContact(handle, contact.handle, name);
      if (result == -1L) throw new IllegalStateException(operations.lastErrorMessage());
      return result;
    }
    /** Removes key. */
    public void deleteKey(long id) { operations.lockboxDeleteKey(handle, id); }
    /** Lists key slots. */
    public java.util.List<KeySlot> listKeySlots() { return operations.lockboxListKeySlots(handle); }
    /** Sets owner signing key. */
    public void setOwnerSigningKey(SigningKeyPair key) { operations.lockboxSetOwnerSigningKey(handle, key.handle); }
    /** Returns the owner inspection. */
    public OwnerInspection ownerInspection() { return operations.lockboxOwnerInspection(handle); }
    /** Returns the define form. */
    public FormDefinition defineForm(String alias, String name, String description, java.util.List<FormField> fields) {
      return operations.lockboxDefineForm(handle, alias, name, description, DomainCodec.encodeFormFields(fields));
    }
    /** Lists form definitions. */
    public java.util.List<FormDefinition> listFormDefinitions() { return operations.lockboxListFormDefinitions(handle); }
    /** Returns the resolve form. */
    public FormDefinition resolveForm(String reference) { return operations.lockboxResolveForm(handle, reference); }
    /** Lists form revisions. */
    public java.util.List<FormDefinition> listFormRevisions(String typeId) { return operations.lockboxListFormRevisions(handle, typeId); }
    /** Creates form record. */
    public FormRecord createFormRecord(String path, String typeReference, String name) {
      return operations.lockboxCreateFormRecord(handle, path, typeReference, name);
    }
    /** Sets form field. */
    public void setFormField(String path, String field, String value) {
      operations.lockboxSetFormField(handle, path, field, value);
    }
    /** Stores a secret form field from mutable bytes. */
    public void setSecretFormField(String path, String field, byte[] value) { operations.lockboxSetSecretFormField(handle, path, field, value); }
    /** Lists form records. */
    public java.util.List<FormRecord> listFormRecords() { return operations.lockboxListFormRecords(handle); }
    /** Returns form record. */
    public FormRecord getFormRecord(String path) { return operations.lockboxGetFormRecord(handle, path); }
    /** Removes form record. */
    public void deleteFormRecord(String path) { operations.lockboxDeleteFormRecord(handle, path); }
    /** Updates form records. */
    public void moveFormRecords(java.util.List<PathMove> moves) { operations.lockboxMoveFormRecords(handle, DomainCodec.encodePathMoves(moves)); }
    /** Returns form field. */
    public FormValue getFormField(String path, String field) { return operations.lockboxGetFormField(handle, path, field); }
    /** Invokes {@code callback} with temporary field bytes, then wipes the transfer buffer. */
    public <T> T withSecretFormField(String path, String field, SecretCallback<T> callback) { return operations.lockboxWithSecretFormField(handle, path, field, callback); }
    /** Returns the bytes. */
    public byte[] bytes() { return operations.lockboxToBytes(handle); }
    /** Releases the native resources held by this object. */
    @Override public void close() { if (handle != null) { operations.lockboxFree(handle); handle = null; } }
  }

  /** Opens vault directory. */
  public VaultDirectory openVaultDirectory(String root, byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryOpen(root, password));
  }
  /** Opens read only vault directory. */
  public ReadOnlyVaultDirectory openReadOnlyVaultDirectory(String root, byte[] password) {
    return new ReadOnlyVaultDirectory(operations.vaultReadOnlyOpen(root, password));
  }
  /** Opens default read only vault directory. */
  public ReadOnlyVaultDirectory openDefaultReadOnlyVaultDirectory(byte[] password) {
    return new ReadOnlyVaultDirectory(operations.vaultReadOnlyOpenDefault(password));
  }
  /** Opens or create vault directory. */
  public VaultDirectory openOrCreateVaultDirectory(String root, byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryOpenOrCreate(root, password));
  }
  /** Updates vault directory. */
  public VaultDirectory replaceVaultDirectory(String root, byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryReplace(root, password));
  }
  /** Opens or create default vault directory. */
  public VaultDirectory openOrCreateDefaultVaultDirectory(byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryOpenOrCreateDefault(password));
  }
  /** Updates default vault directory. */
  public VaultDirectory replaceDefaultVaultDirectory(byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryReplaceDefault(password));
  }
  /** Updates vault directory password. */
  public void changeVaultDirectoryPassword(String root, byte[] oldPassword, byte[] newPassword) {
    operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  }
  /** Updates default vault directory password. */
  public void changeDefaultVaultDirectoryPassword(byte[] oldPassword, byte[] newPassword) {
    operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  }
  /** Returns the default vault directory. */
  public String defaultVaultDirectory() { return operations.vaultDefaultDirectory(); }
  /** Returns the default vault path. */
  public String defaultVaultPath() { return operations.vaultDefaultPath(); }
  /** Returns the backup default vault. */
  public VaultBackupManifest backupDefaultVault(String path, boolean overwrite) {
    return operations.vaultBackupDefault(path, overwrite);
  }
  /** Returns the restore default vault. */
  public VaultBackupManifest restoreDefaultVault(String path, boolean overwrite) {
    return operations.vaultRestoreDefault(path, overwrite);
  }

  /** Password-protected storage for profile keys, contacts, forms, backups, and known lockbox paths. */
  public final class VaultDirectory implements AutoCloseable {
    private MemorySegment handle;
    private VaultDirectory(MemorySegment handle) { this.handle = handle; }
    /** Returns the root. */
    public String root() { return operations.vaultDirectoryRoot(handle); }
    /** Returns the structure version. */
    public int structureVersion() { return operations.vaultDirectoryStructureVersion(handle); }
    /** Lists private keys. */
    public java.util.List<String> listPrivateKeys() { return operations.vaultDirectoryListPrivateKeys(handle); }
    /** Lists private key names. */
    public java.util.List<String> listPrivateKeyNames() { return operations.vaultDirectoryListPrivateKeyNames(handle); }
    /** Lists contact names. */
    public java.util.List<String> listContactNames() { return operations.vaultDirectoryListContactNames(handle); }
    /** Lists form aliases. */
    public java.util.List<String> listFormAliases() { return operations.vaultDirectoryListFormAliases(handle); }
    /** Returns the private key exists. */
    public boolean privateKeyExists(String name) { return operations.vaultDirectoryPrivateKeyExists(handle, name); }
    /** Removes private key. */
    public void deletePrivateKey(String name) { operations.vaultDirectoryDeletePrivateKey(handle, name); }
    /** Stores private key. */
    public void storePrivateKey(String name, ContactKeyPair key) { operations.vaultDirectoryStorePrivateKey(handle, name, key.handle); }
    /** Loads private key. */
    public ContactKeyPair loadPrivateKey(String name) { return new ContactKeyPair(operations.vaultDirectoryLoadPrivateKey(handle, name)); }
    /** Loads private key generation. */
    public ContactKeyPair loadPrivateKeyGeneration(String name, int index) {
      return new ContactKeyPair(operations.vaultDirectoryLoadPrivateKeyGeneration(handle, name, (short) index));
    }
    /** Stores contact. */
    public void storeContact(String name, ContactPublicKey key) { operations.vaultDirectoryStoreContact(handle, name, key.handle); }
    /** Loads contact. */
    public ContactPublicKey loadContact(String name) { return new ContactPublicKey(operations.vaultDirectoryLoadContact(handle, name)); }
    /** Returns the contact exists. */
    public boolean contactExists(String name) { return operations.vaultDirectoryContactExists(handle, name); }
    /** Removes contact. */
    public void deleteContact(String name) { operations.vaultDirectoryDeleteContact(handle, name); }
    /** Lists contacts. */
    public java.util.List<Contact> listContacts() { return operations.vaultDirectoryListContacts(handle); }
    /** Stores profile email. */
    public void storeProfileEmail(String name, String email) { operations.vaultDirectoryStoreProfileEmail(handle, name, email); }
    /** Returns the profile email. */
    public String profileEmail(String name) { return operations.vaultDirectoryProfileEmail(handle, name); }
    /** Stores backup. */
    public void storeBackup(byte[] id, byte[] value) { operations.vaultDirectoryStoreBackup(handle, id, value); }
    /** Loads backup. */
    public byte[] loadBackup(byte[] id) { return operations.vaultDirectoryLoadBackup(handle, id); }
    /** Returns the backup count. */
    public long backupCount() { return operations.vaultDirectoryBackupCount(handle); }
    /** Returns the restore private key. */
    public void restorePrivateKey(String name, ContactKeyPair key, SigningKeyPair signingKey, boolean overwrite) {
      operations.vaultDirectoryRestorePrivateKey(handle, name, key.handle, signingKey.handle, overwrite);
    }
    /** Loads owner signing key. */
    public SigningKeyPair loadOwnerSigningKey(String name) {
      return new SigningKeyPair(operations.vaultDirectoryLoadOwnerSigningKey(handle, name));
    }
    /** Loads owner signing key generation. */
    public SigningKeyPair loadOwnerSigningKeyGeneration(String name, int index) {
      return new SigningKeyPair(operations.vaultDirectoryLoadOwnerSigningKeyGeneration(handle, name, (short) index));
    }
    /** Stores contact signing key. */
    public void storeContactSigningKey(String name, SigningPublicKey key) {
      operations.vaultDirectoryStoreContactSigningKey(handle, name, key.handle);
    }
    /** Loads contact signing key. */
    public SigningPublicKey loadContactSigningKey(String name) {
      return new SigningPublicKey(operations.vaultDirectoryLoadContactSigningKey(handle, name));
    }
    /** Lists profile generations. */
    public ProfileHistory listProfileGenerations(String name) {
      return operations.vaultDirectoryListProfileGenerations(handle, name);
    }
    /** Updates private key. */
    public ProfileHistory rotatePrivateKey(String name) { return operations.vaultDirectoryRotatePrivateKey(handle, name); }
    /** Stores lockbox. */
    public void rememberLockbox(byte[] id, String path) { operations.vaultDirectoryRememberLockbox(handle, id, path); }
    /** Lists known lockboxes. */
    public java.util.List<KnownLockbox> listKnownLockboxes() { return operations.vaultDirectoryListKnownLockboxes(handle); }
    /** Removes lockbox. */
    public void forgetLockbox(String path) { operations.vaultDirectoryForgetLockbox(handle, path); }
    /** Stores access slot label. */
    public void rememberAccessSlotLabel(byte[] id, long slotId, String name) {
      operations.vaultDirectoryRememberAccessSlotLabel(handle, id, slotId, name);
    }
    /** Lists access slot labels. */
    public java.util.List<AccessSlotLabel> listAccessSlotLabels(byte[] id) { return operations.vaultDirectoryListAccessSlotLabels(handle, id); }
    /** Returns the find access slot labels. */
    public java.util.List<AccessSlotLabel> findAccessSlotLabels(byte[] id, String name) {
      return operations.vaultDirectoryFindAccessSlotLabels(handle, id, name);
    }
    /** Removes access slot label. */
    public void forgetAccessSlotLabel(byte[] id, long slotId) { operations.vaultDirectoryForgetAccessSlotLabel(handle, id, slotId); }
    /** Returns the define form. */
    public FormDefinition defineForm(String alias, String name, String description, java.util.List<FormField> fields) {
      return operations.vaultDirectoryDefineForm(handle, alias, name, description, DomainCodec.encodeFormFields(fields));
    }
    /** Returns the resolve form. */
    public FormDefinition resolveForm(String reference) { return operations.vaultDirectoryResolveForm(handle, reference); }
    /** Lists forms. */
    public java.util.List<FormDefinition> listForms() { return operations.vaultDirectoryListForms(handle); }
    /** Lists form revisions. */
    public java.util.List<FormDefinition> listFormRevisions(String typeId) { return operations.vaultDirectoryListFormRevisions(handle, typeId); }
    /** Returns the seed forms. */
    public long seedForms() { return operations.vaultDirectorySeedForms(handle); }
    /** Stores password. */
    public void rememberPassword(byte[] id, byte[] password) { operations.vaultDirectoryRememberPassword(handle, id, password); }
    /** Returns the remembered password. */
    public byte[] rememberedPassword(byte[] id) { return operations.vaultDirectoryRememberedPassword(handle, id); }
    /** Releases the native resources held by this object. */
    @Override public void close() { if (handle != null) { operations.vaultDirectoryFree(handle); handle = null; } }
  }

  /** A metadata view for discovery and diagnostics that never loads an owner signing key. */
  public final class ReadOnlyVaultDirectory implements AutoCloseable {
    private MemorySegment handle;
    private ReadOnlyVaultDirectory(MemorySegment handle) { this.handle = handle; }
    /** Lists profile names. */
    public java.util.List<String> listProfileNames() { return operations.vaultReadOnlyListProfileNames(handle); }
    /** Lists contact names. */
    public java.util.List<String> listContactNames() { return operations.vaultReadOnlyListContactNames(handle); }
    /** Lists form aliases. */
    public java.util.List<String> listFormAliases() { return operations.vaultReadOnlyListFormAliases(handle); }
    /** Lists known lockboxes. */
    public java.util.List<KnownLockbox> listKnownLockboxes() { return operations.vaultReadOnlyListKnownLockboxes(handle); }
    /** Releases the native resources held by this object. */
    @Override public void close() { if (handle != null) { operations.vaultReadOnlyFree(handle); handle = null; } }
  }

  /** Returns the agent is running. */
  public boolean agentIsRunning() { return operations.vaultIsRunning(); }
  /** Returns the serve agent. */
  public void serveAgent() { operations.vaultAgentServe(); }
  /** Verifies agent transport. */
  public void verifyAgentTransport() { operations.vaultAgentVerifyTransport(); }
  /** Removes all agent secrets. */
  public void forgetAllAgentSecrets() { operations.vaultForgetAll(); }
  /** Stops agent. */
  public void stopAgent() { operations.vaultAgentStop(); }
  /** Starts agent. */
  public void startAgent() { operations.vaultAgentStart(); }
  /** Stores agent key. */
  public void putAgentKey(byte[] id, byte[] key) { operations.vaultAgentPut(id, key); }
  /** Returns agent key. */
  public byte[] getAgentKey(byte[] id) { return operations.vaultAgentGet(id); }
  /** Removes agent key. */
  public void forgetAgentKey(byte[] id) { operations.vaultAgentForget(id); }
  /** Lists agent keys. */
  public java.util.List<AgentEntry> listAgentKeys() { return operations.vaultAgentList(); }
  /** Returns the agent sleep support. */
  public SleepSupport agentSleepSupport() { return operations.vaultAgentSleepSupport(); }
  /** Returns the agent log path. */
  public String agentLogPath() { return operations.vaultAgentLogPath(); }
  /** Returns the agent log destination. */
  public String agentLogDestination() { return operations.vaultAgentLogDestination(); }
  /** Stores agent vault unlock key. */
  public void putAgentVaultUnlockKey(String vaultId, byte[] key, long ttlSeconds) {
    operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
  }
  /** Returns agent vault unlock key. */
  public byte[] getAgentVaultUnlockKey(String vaultId) { return operations.vaultAgentGetVaultUnlockKey(vaultId); }
  /** Removes agent vault unlock key. */
  public void forgetAgentVaultUnlockKey(String vaultId) { operations.vaultAgentForgetVaultUnlockKey(vaultId); }
  /** Stores agent owner signing key. */
  public void putAgentOwnerSigningKey(String vaultId, String profile, SigningKeyPair key, long ttlSeconds) {
    operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key.handle, ttlSeconds);
  }
  /** Returns agent owner signing key. */
  public SigningKeyPair getAgentOwnerSigningKey(String vaultId, String profile) {
    return new SigningKeyPair(operations.vaultAgentGetOwnerSigningKey(vaultId, profile));
  }
  /** Removes agent owner signing key. */
  public void forgetAgentOwnerSigningKey(String vaultId, String profile) {
    operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);
  }

  /** Starts agent activity. */
  public AgentActivity beginAgentActivity(String kind) {
    return new AgentActivity(operations.vaultAgentBeginActivity(kind));
  }
  /** A token kept alive while an operation needs secrets cached by the session agent. */
  public final class AgentActivity implements AutoCloseable {
    private MemorySegment handle;
    private AgentActivity(MemorySegment handle) { this.handle = handle; }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.vaultAgentEndActivity(handle); handle = null; }
    }
  }

  /** Returns the platform status. */
  public PlatformStatus platformStatus() { return operations.vaultPlatformStatus(); }
  /** Sets platform scope. */
  public void setPlatformScope(String scope) { operations.vaultPlatformSetScope(scope); }
  /** Returns the enable platform store. */
  public void enablePlatformStore() { operations.vaultPlatformEnable(); }
  /** Returns the disable platform store. */
  public void disablePlatformStore() { operations.vaultPlatformDisable(); }
  /** Returns the platform store disabled. */
  public boolean platformStoreDisabled() { return operations.vaultPlatformDisabled(); }
  /** Stores platform password. */
  public void putPlatformPassword(byte[] password) { operations.vaultPlatformPutPassword(password); }
  /** Returns platform password. */
  public byte[] getPlatformPassword() { return operations.vaultPlatformGetPassword(); }
  /** Removes platform password. */
  public void forgetPlatformPassword() { operations.vaultPlatformForgetPassword(); }

  /** Opens local vault. */
  public LocalVault openLocalVault() { return new LocalVault(operations.vaultLocal()); }
  /** A session that opens lockboxes by host path, caches passwords, and closes locally used files. */
  public final class LocalVault implements AutoCloseable {
    private MemorySegment handle;
    private LocalVault(MemorySegment handle) { this.handle = handle; }
    /** Creates with password. */
    public Lockbox createWithPassword(String path, byte[] password) {
      return new Lockbox(operations.vaultCreateLockboxPassword(handle, path, password));
    }
    /** Opens with password. */
    public Lockbox openWithPassword(String path, byte[] password) {
      return new Lockbox(operations.vaultOpenLockboxPassword(handle, path, password));
    }
    /** Creates with content key. */
    public Lockbox createWithContentKey(String path, byte[] key, SigningKeyPair signingKey) {
      return new Lockbox(operations.vaultCreateLockboxContentKey(handle, path, key, signingKey.handle));
    }
    /** Opens with content key. */
    public Lockbox openWithContentKey(String path, byte[] key, SigningKeyPair signingKey) {
      return new Lockbox(operations.vaultOpenLockboxContentKey(handle, path, key, signingKey.handle));
    }
    /** Creates for contact. */
    public Lockbox createForContact(String path, ContactPublicKey contact, String name, SigningKeyPair signingKey) {
      return new Lockbox(operations.vaultCreateLockboxContact(handle, path, contact.handle, name, signingKey.handle));
    }
    /** Stores password. */
    public void cachePassword(String path, byte[] password, long ttlSeconds) {
      operations.vaultCacheLockboxPassword(handle, path, password, ttlSeconds);
    }
    /** Releases the native resources held by lockbox. */
    public void closeLockbox(String path) { operations.vaultCloseLockbox(handle, path); }
    /** Releases the native resources held by all. */
    public void closeAll() { operations.vaultCloseAll(handle); }
    /** Releases the native resources held by this object. */
    @Override public void close() { if (handle != null) { operations.vaultFree(handle); handle = null; } }
  }
}
