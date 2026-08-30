package com.onepub.revault;

import java.lang.foreign.MemorySegment;
import java.lang.foreign.Arena;
import java.lang.foreign.SymbolLookup;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;

/**
 * Entry point for encrypted lockboxes, keys, local vault metadata, the session
 * Session Agent, and the platform credential store.
 *
 * <p>Create one when the application starts, then use it to open lockboxes and
 * manage keys and local services. Owned objects implement {@link AutoCloseable}. Secret variables and form
 * fields are exposed through callback-scoped methods. See the
 * <a href="https://github.com/onepub-dev/reVault#readme">repository README</a>
 * for installation and examples.
 */
public final class Revault {
  /**
   * Consumes temporary secret bytes before their transfer copy is wiped.
   *
   * @param <T> value returned by the callback
   */
  @FunctionalInterface
  public interface SecretCallback<T> {
    /** Uses temporary secret bytes that must not be retained after this call. */
    T use(byte[] secret);
  }

  private final BindingOperations operations;

  /** Loads the process-wide native reVault runtime using platform discovery. */
  public static Revault load() { return new Revault(); }

  /**
   * Loads an application-owned carrier before inherited or packaged discovery.
   * @param nativeLibraryPath filesystem path or operating-system library name
   */
  public static Revault load(String nativeLibraryPath) { return new Revault(nativeLibraryPath); }

  /**
   * Loads an application-owned carrier before inherited or packaged discovery.
   * @param nativeLibraryPath filesystem path to the carrier
   */
  public static Revault load(Path nativeLibraryPath) {
    if (nativeLibraryPath == null) throw new NullPointerException("nativeLibraryPath");
    return load(nativeLibraryPath.toString());
  }

  /** Creates a runtime loader. Prefer {@link #load()} in application code. */
  public Revault() {
    this((String) null);
  }

  /**
   * Creates a runtime loader using explicit, inherited, then packaged discovery.
   * @param nativeLibraryPath explicit path/name, or {@code null} for inherited/package discovery
   */
  public Revault(String nativeLibraryPath) {
    this(new RevaultNativeApi(SymbolLookup.libraryLookup(
        NativeLibrary.resolve(nativeLibraryPath), Arena.global())));
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
    /** Creates options from the closed policy enums used by the public facade. */
    public LockboxOptions(CacheMode cacheMode, long cacheBytes, WorkloadProfile workload,
        WorkerPolicy worker, long jobs) {
      this(cacheMode.wire(), cacheBytes, workload.wire(), worker.wire(), jobs);
    }

    /** Returns the cache policy as a closed enum. */
    public CacheMode cacheModeValue() { return CacheMode.valueOf(cacheMode.toUpperCase(java.util.Locale.ROOT)); }
    /** Returns the workload policy as a closed enum. */
    public WorkloadProfile workloadValue() {
      return "bulk-import".equals(workload) ? WorkloadProfile.BULK_IMPORT : WorkloadProfile.INTERACTIVE;
    }
    /** Returns the worker policy as a closed enum. */
    public WorkerPolicy workerValue() { return WorkerPolicy.valueOf(worker.toUpperCase(java.util.Locale.ROOT)); }

    /** Returns the recommended interactive runtime defaults. */
    public static LockboxOptions defaults() {
      return new LockboxOptions("bytes", 64L << 20, "interactive", "auto", 0);
    }
  }

  private static void ensureOpen(MemorySegment handle) {
    if (handle == null || handle.address() == 0) throw new IllegalStateException("object is closed");
  }

  /** Returns the last native error message. */
  public String lastError() { return operations.lastErrorMessage(); }
  /** Returns structured details for the last native error. */
  public ErrorDetails lastErrorDetails() { return operations.bufferLastErrorDetails(); }

  /** Returns the newest Lockbox archive format version supported by this engine. */
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
    /** Returns the encrypted content key bytes. */
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

  /** The public profile identity readers use to verify authorized lockbox revisions. */
  public final class ProfileSigningPublicKey implements AutoCloseable {
    private MemorySegment handle;
    private ProfileSigningPublicKey(MemorySegment handle) { this.handle = handle; }
    /** Releases the native resources held by this object. */
    @Override public void close() {
      if (handle != null) { operations.keySigningPublicFree(handle); handle = null; }
    }
  }

  /** A profile signing identity used to authorize mutable lockbox revisions. */
  public final class ProfileSigningKeyPair implements AutoCloseable {
    private MemorySegment handle;
    private ProfileSigningKeyPair(MemorySegment handle) { this.handle = handle; }
    /** Returns the public bytes. */
    public byte[] publicBytes() { ensureOpen(handle); return operations.keySigningPublic(handle); }
    /** Returns the private record. */
    public byte[] privateRecord() { ensureOpen(handle); return operations.keySigningPrivate(handle); }
    /** Returns the public key. */
    public ProfileSigningPublicKey publicKey() { return profileSigningPublicKeyFromBytes(publicBytes()); }
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
  /** Generates a profile signing key pair. */
  public ProfileSigningKeyPair generateProfileSigningKeyPair() { return new ProfileSigningKeyPair(operations.keySigningGenerate()); }
  /** Imports a profile signing key pair from its private record. */
  public ProfileSigningKeyPair profileSigningKeyPairFromPrivate(byte[] value) { return new ProfileSigningKeyPair(operations.keySigningFromPrivate(value)); }
  /** Imports a profile signing public key from encoded bytes. */
  public ProfileSigningPublicKey profileSigningPublicKeyFromBytes(byte[] value) { return new ProfileSigningPublicKey(operations.keySigningPublicFromBytes(value)); }

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
  public Lockbox createLockboxWithProfileSigningKey(byte[] contentKey, ProfileSigningKeyPair signingKey) {
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
  public Lockbox salvageLockbox(byte[] archive, byte[] key, ProfileSigningKeyPair signingKey) {
    return new Lockbox(operations.lockboxRecoverySalvage(archive, key,
        signingKey == null ? MemorySegment.NULL : signingKey.handle));
  }

  /** An open encrypted archive containing files, variables, secrets, and forms. */
  public final class Lockbox implements AutoCloseable {
    private MemorySegment handle;
    private Lockbox(MemorySegment handle) { this.handle = handle; }
    /** Stages a file at the Lockbox path; replace controls an existing entry. */
    public void addFile(String path, byte[] value, boolean replace) { operations.lockboxAddFile(handle, path, value, replace); }
    /** Stages a file at the Lockbox path; replace controls an existing entry. */
    public void addFile(String path, byte[] value, int permissions, boolean replace) { operations.lockboxAddFileWithPermissions(handle, path, value, permissions, replace); }
    /** Reads the complete file stored at the Lockbox path. */
    public byte[] getFile(String path) { return operations.lockboxGetFile(handle, path); }
    /** Writes one Lockbox file to the host filesystem. */
    public void extractFile(String source, String destination, boolean replace) { operations.lockboxExtractFile(handle, source, destination, replace); }
    /** Extracts the Lockbox with explicit size, count, link, and permission limits. */
    public void extractDirectory(String destination, long maxFileBytes, long maxTotalBytes, long maxFiles,
        boolean restoreSymlinks, boolean restorePermissions, boolean overwrite) {
      operations.lockboxExtractDirectory(handle, destination, maxFileBytes, maxTotalBytes, maxFiles,
          restoreSymlinks, restorePermissions, overwrite);
    }
    /** Lists logical or physical content chunks for streaming diagnostics. */
    public java.util.List<StreamChunk> streamContent(boolean physical) { return operations.lockboxStreamContent(handle, physical); }
    /** Returns cache statistics for this lockbox. */
    public CacheStats cacheStats() { return operations.lockboxCacheStats(handle); }
    /** Returns import statistics for this lockbox. */
    public ImportStats importStats() { return operations.lockboxImportStats(handle); }
    /** Updates import stats. */
    public void resetImportStats() { operations.lockboxResetImportStats(handle); }
    /** Returns page metadata for diagnostics without exposing plaintext secrets. */
    public java.util.List<PageInspection> pageInspection() { return operations.lockboxPageInspection(handle); }
    /** Scans the open archive and returns its structured recovery report. */
    public RecoveryReport recoveryReport() { return operations.lockboxRecoveryReport(handle); }
    /** Returns the render recovery report. */
    public String renderRecoveryReport(boolean verbose, long maxEntries) { return operations.lockboxRecoveryReportRender(handle, verbose, maxEntries); }
    /** Returns the storage length. */
    public long storageLength() { return operations.lockboxStorageLen(handle); }
    /** Sets workload profile. */
    public void setWorkloadProfile(String profile) { operations.lockboxSetWorkloadProfile(handle, profile); }
    /** Sets worker policy. */
    public void setWorkerPolicy(String mode, long jobs) { operations.lockboxSetWorkerPolicy(handle, mode, jobs); }
    /** Returns the cache, workload, and worker settings used by this Lockbox. */
    public RuntimeOptions runtimeOptions() { return operations.lockboxRuntimeOptions(handle); }
    /** Authenticates and publishes the staged changes. */
    public void commit() { operations.lockboxCommit(handle); }
    /** Creates directory. */
    public void createDirectory(String path, boolean parents) { operations.lockboxCreateDir(handle, path, parents); }
    /** Stages removal of a file, link, or empty directory at path. */
    public void delete(String path) { operations.lockboxDelete(handle, path); }
    /** Removes directory. */
    public void removeDirectory(String path, boolean recursive) { operations.lockboxRemoveDir(handle, path, recursive); }
    /** Creates parent directories. */
    public void createParentDirectories(String path) { operations.lockboxCreateParentDirs(handle, path); }
    /** Stages an atomic move from one Lockbox path to another. */
    public void rename(String from, String to) { operations.lockboxRename(handle, from, to); }
    /** Lists entries below path, optionally including descendants. */
    public java.util.List<LockboxEntry> list(String path, boolean recursive) { return operations.lockboxList(handle, path, recursive); }
    /** Lists entries below path, optionally including descendants. */
    public java.util.List<LockboxEntry> list(String path, String glob, boolean recursive, boolean includeFiles,
        boolean includeSymlinks, boolean includeDirectories, long limit) {
      return operations.lockboxListWithOptions(handle, path, glob, recursive, includeFiles,
          includeSymlinks, includeDirectories, limit);
    }
    /** Returns metadata for the selected lockbox entry. */
    public LockboxEntry stat(String path) { return operations.lockboxStat(handle, path); }
    /** Stages a plain text variable; commit to publish the change. */
    public void setVariable(String name, String value) { operations.lockboxSetVariable(handle, name, value); }
    /** Stores a secret variable from mutable bytes. */
    public void setSecretVariable(String name, byte[] value) { operations.lockboxSetSecretVariable(handle, name, value); }
    /** Returns a plain variable when it is present. */
    public String getVariable(String name) { return operations.lockboxGetVariable(handle, name); }
    /** Returns the encrypted Lockbox description, or {@code null} when unset. Example: {@code box.setDescription("Production credentials"); box.commit(); System.out.println(box.description());} */
    public String description() { return getVariable("/.revault/description"); }
    /** Stages an encrypted Lockbox description; call {@link #commit()} to publish it. Example: {@code box.setDescription("Production credentials"); box.commit();} */
    public void setDescription(String description) { setVariable("/.revault/description", description); }
    /** Stages removal of the encrypted Lockbox description; call {@link #commit()}. Example: {@code box.clearDescription(); box.commit();} */
    public void clearDescription() { deleteVariable("/.revault/description"); }
    /** Invokes {@code callback} with temporary secret bytes, then wipes the transfer buffer. */
    public <T> T withSecretVariable(String name, SecretCallback<T> callback) { return operations.lockboxWithSecretVariable(handle, name, callback); }
    /** Stages removal of a variable. */
    public void deleteVariable(String name) { operations.lockboxDeleteVariable(handle, name); }
    /** Updates variables. */
    public void moveVariables(java.util.List<PathMove> moves) { operations.lockboxMoveVariables(handle, DomainCodec.encodePathMoves(moves)); }
    /** Lists variable names and metadata without exposing secret values. */
    public java.util.List<Variable> listVariables() { return operations.lockboxListVariables(handle); }
    /** Returns whether a variable is plain or secret. */
    public String variableSensitivity(String name) { return operations.lockboxVariableSensitivity(handle, name); }
    /** Stages a symbolic link with its stored target text. */
    public void addSymlink(String path, String target, boolean replace) { operations.lockboxAddSymlink(handle, path, target, replace); }
    /** Returns the symlink target. */
    public String symlinkTarget(String path) { return operations.lockboxGetSymlinkTarget(handle, path); }
    /** Returns the stable public identifier stored in the Lockbox header. */
    public byte[] id() { return operations.lockboxId(handle); }
    /** Reports whether an entry exists at path. */
    public boolean exists(String path) { return operations.lockboxExists(handle, path); }
    /** Reports whether path names a directory entry. */
    public boolean isDirectory(String path) { return operations.lockboxIsDir(handle, path); }
    /** Returns the portable Unix permission bits stored for path. */
    public int permissions(String path) { return operations.lockboxPermissions(handle, path); }
    /** Stages portable Unix permission bits for path. */
    public void setPermissions(String path, int value) { operations.lockboxSetPermissions(handle, path, value); }
    /** Reads the requested byte range from a stored file. */
    public byte[] readRange(String path, long offset, long length) { return operations.lockboxReadRange(handle, path, offset, length); }
    /** Adds a password access slot and returns its slot identifier. */
    public long addPassword(byte[] password) {
      long result = operations.lockboxAddPassword(handle, password);
      if (result == -1L) throw new IllegalStateException(operations.lastErrorMessage());
      return result;
    }
    /** Grants a named contact access and returns the new slot identifier. */
    public long addContact(ContactPublicKey contact, String name) {
      long result = operations.lockboxAddContact(handle, contact.handle, name);
      if (result == -1L) throw new IllegalStateException(operations.lastErrorMessage());
      return result;
    }
    /** Removes an access slot; at least one usable slot must remain. */
    public void deleteKey(long id) { operations.lockboxDeleteKey(handle, id); }
    /** Lists public access slot metadata without returning credentials. */
    public java.util.List<KeySlot> listKeySlots() { return operations.lockboxListKeySlots(handle); }
    /** Assigns a profile signing key to the Lockbox owner role. */
    public void setOwnerSigningKey(ProfileSigningKeyPair key) { operations.lockboxSetOwnerSigningKey(handle, key.handle); }
    /** Returns public signing and ownership metadata for the current revision. */
    public OwnerInspection ownerInspection() { return operations.lockboxOwnerInspection(handle); }
    /** Defines and stores a reusable versioned form. */
    public FormDefinition defineForm(String alias, String name, String description, java.util.List<FormField> fields) {
      return operations.lockboxDefineForm(handle, alias, name, description, DomainCodec.encodeFormFields(fields));
    }
    /** Lists the form definitions stored in this Lockbox. */
    public java.util.List<FormDefinition> listFormDefinitions() { return operations.lockboxListFormDefinitions(handle); }
    /** Resolves a form alias, type identifier, or revision. */
    public FormDefinition resolveForm(String reference) { return operations.lockboxResolveForm(handle, reference); }
    /** Lists every stored revision for a form type identifier. */
    public java.util.List<FormDefinition> listFormRevisions(String typeId) { return operations.lockboxListFormRevisions(handle, typeId); }
    /** Stages a form record at path using the referenced definition. */
    public FormRecord createFormRecord(String path, String typeReference, String name) {
      return operations.lockboxCreateFormRecord(handle, path, typeReference, name);
    }
    /** Stages a plain field value in a form record. */
    public void setFormField(String path, String field, String value) {
      operations.lockboxSetFormField(handle, path, field, value);
    }
    /** Stores a secret form field from mutable bytes. */
    public void setSecretFormField(String path, String field, byte[] value) { operations.lockboxSetSecretFormField(handle, path, field, value); }
    /** Lists form records without exposing secret field values. */
    public java.util.List<FormRecord> listFormRecords() { return operations.lockboxListFormRecords(handle); }
    /** Returns the form record at path when present. */
    public FormRecord getFormRecord(String path) { return operations.lockboxGetFormRecord(handle, path); }
    /** Stages removal of a form record. */
    public void deleteFormRecord(String path) { operations.lockboxDeleteFormRecord(handle, path); }
    /** Updates form records. */
    public void moveFormRecords(java.util.List<PathMove> moves) { operations.lockboxMoveFormRecords(handle, DomainCodec.encodePathMoves(moves)); }
    /** Returns a plain form field when it exists. */
    public FormValue getFormField(String path, String field) { return operations.lockboxGetFormField(handle, path, field); }
    /** Invokes {@code callback} with temporary field bytes, then wipes the transfer buffer. */
    public <T> T withSecretFormField(String path, String field, SecretCallback<T> callback) { return operations.lockboxWithSecretFormField(handle, path, field, callback); }
    /** Returns the bytes. */
    public byte[] bytes() { return operations.lockboxToBytes(handle); }
    /** Releases the native resources held by this object. */
    @Override public void close() { if (handle != null) { operations.lockboxFree(handle); handle = null; } }
  }

  /** Opens vault directory. */
  public VaultHandle openVault(String root, byte[] password) {
    return new VaultHandle(operations.vaultDirectoryOpen(root, password));
  }
  /** Opens read only vault directory. */
  public ReadOnlyVaultHandle openReadOnlyVault(String root, byte[] password) {
    return new ReadOnlyVaultHandle(operations.vaultReadOnlyOpen(root, password));
  }
  /** Opens default read only vault directory. */
  public ReadOnlyVaultHandle openDefaultReadOnlyVault(byte[] password) {
    return new ReadOnlyVaultHandle(operations.vaultReadOnlyOpenDefault(password));
  }
  /** Opens or create vault directory. */
  public VaultHandle openOrCreateVault(String root, byte[] password) {
    return new VaultHandle(operations.vaultDirectoryOpenOrCreate(root, password));
  }
  /** Updates vault directory. */
  public VaultHandle replaceVault(String root, byte[] password) {
    return new VaultHandle(operations.vaultDirectoryReplace(root, password));
  }
  /** Opens or create default vault directory. */
  public VaultHandle openOrCreateDefaultVault(byte[] password) {
    return new VaultHandle(operations.vaultDirectoryOpenOrCreateDefault(password));
  }
  /** Updates default vault directory. */
  public VaultHandle replaceDefaultVault(byte[] password) {
    return new VaultHandle(operations.vaultDirectoryReplaceDefault(password));
  }
  /** Updates vault directory password. */
  public void changeVaultPassword(String root, byte[] oldPassword, byte[] newPassword) {
    operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  }
  /** Updates default vault directory password. */
  public void changeDefaultVaultPassword(byte[] oldPassword, byte[] newPassword) {
    operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  }
  /** Returns the default vault directory. */
  public String defaultVaultRoot() { return operations.vaultDefaultDirectory(); }
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

  /**
   * Password-protected storage for Profile keys, contacts, forms, backups,
   * and known lockbox paths. This internal implementation base is exposed
   * through the public {@link Vault} facade.
   */
  public class VaultHandle implements AutoCloseable {
    private MemorySegment handle;
    protected VaultHandle(MemorySegment handle) { this.handle = handle; }
    MemorySegment detach() { var value = handle; handle = null; return value; }
    /** Returns the canonical root directory of this Vault. */
    public String root() { return operations.vaultDirectoryRoot(handle); }
    /** Returns the persistent structure version of this Vault. */
    public int structureVersion() { return operations.vaultDirectoryStructureVersion(handle); }
    /** Lists private keys. */
    public java.util.List<String> listPrivateKeys() { return operations.vaultDirectoryListPrivateKeys(handle); }
    /** Lists private key names. */
    public java.util.List<String> listPrivateKeyNames() { return operations.vaultDirectoryListPrivateKeyNames(handle); }
    /** Lists contact names. */
    public java.util.List<String> listContactNames() { return operations.vaultDirectoryListContactNames(handle); }
    /** Lists form aliases. */
    public java.util.List<String> listFormAliases() { return operations.vaultDirectoryListFormAliases(handle); }
    /** Reports whether the named profile private key exists. */
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
    /** Reports whether the named contact exists. */
    public boolean contactExists(String name) { return operations.vaultDirectoryContactExists(handle, name); }
    /** Removes contact. */
    public void deleteContact(String name) { operations.vaultDirectoryDeleteContact(handle, name); }
    /** Lists contacts. */
    public java.util.List<Contact> listContacts() { return operations.vaultDirectoryListContacts(handle); }
    /** Stores profile email. */
    public void storeProfileEmail(String name, String email) { operations.vaultDirectoryStoreProfileEmail(handle, name, email); }
    /** Returns the email recorded for a profile, when present. */
    public String profileEmail(String name) { return operations.vaultDirectoryProfileEmail(handle, name); }
    /** Stores backup. */
    public void storeBackup(byte[] id, byte[] value) { operations.vaultDirectoryStoreBackup(handle, id, value); }
    /** Loads backup. */
    public byte[] loadBackup(byte[] id) { return operations.vaultDirectoryLoadBackup(handle, id); }
    /** Returns the number of stored key recovery backups. */
    public long backupCount() { return operations.vaultDirectoryBackupCount(handle); }
    /** Restores a profile private key and signing key from recovery material. */
    public void restorePrivateKey(String name, ContactKeyPair key, ProfileSigningKeyPair signingKey, boolean overwrite) {
      operations.vaultDirectoryRestorePrivateKey(handle, name, key.handle, signingKey.handle, overwrite);
    }
    /** Loads the current profile signing key. */
    public ProfileSigningKeyPair loadProfileSigningKey(String name) {
      return new ProfileSigningKeyPair(operations.vaultDirectoryLoadOwnerSigningKey(handle, name));
    }
    /** Loads a historical profile signing key generation. */
    public ProfileSigningKeyPair loadProfileSigningKeyGeneration(String name, int index) {
      return new ProfileSigningKeyPair(operations.vaultDirectoryLoadOwnerSigningKeyGeneration(handle, name, (short) index));
    }
    /** Stores contact signing key. */
    public void storeContactSigningKey(String name, ProfileSigningPublicKey key) {
      operations.vaultDirectoryStoreContactSigningKey(handle, name, key.handle);
    }
    /** Loads contact signing key. */
    public ProfileSigningPublicKey loadContactSigningKey(String name) {
      return new ProfileSigningPublicKey(operations.vaultDirectoryLoadContactSigningKey(handle, name));
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
    /** Finds access slot labels with the supplied name for one Lockbox. */
    public java.util.List<AccessSlotLabel> findAccessSlotLabels(byte[] id, String name) {
      return operations.vaultDirectoryFindAccessSlotLabels(handle, id, name);
    }
    /** Removes access slot label. */
    public void forgetAccessSlotLabel(byte[] id, long slotId) { operations.vaultDirectoryForgetAccessSlotLabel(handle, id, slotId); }
    /** Defines and stores a reusable versioned form. */
    public FormDefinition defineForm(String alias, String name, String description, java.util.List<FormField> fields) {
      return operations.vaultDirectoryDefineForm(handle, alias, name, description, DomainCodec.encodeFormFields(fields));
    }
    /** Resolves a form alias, type identifier, or revision. */
    public FormDefinition resolveForm(String reference) { return operations.vaultDirectoryResolveForm(handle, reference); }
    /** Lists forms. */
    public java.util.List<FormDefinition> listForms() { return operations.vaultDirectoryListForms(handle); }
    /** Lists every stored revision for a form type identifier. */
    public java.util.List<FormDefinition> listFormRevisions(String typeId) { return operations.vaultDirectoryListFormRevisions(handle, typeId); }
    /** Adds missing standard form definitions and returns the number added. */
    public long seedForms() { return operations.vaultDirectorySeedForms(handle); }
    /** Stores password. */
    public void rememberPassword(byte[] id, byte[] password) { operations.vaultDirectoryRememberPassword(handle, id, password); }
    /** Returns the Lockbox password encrypted inside this Vault. */
    public byte[] rememberedPassword(byte[] id) { return operations.vaultDirectoryRememberedPassword(handle, id); }
    /** Releases the native resources held by this object. */
    @Override public void close() { if (handle != null) { operations.vaultDirectoryFree(handle); handle = null; } }
  }

  /**
   * A metadata view for discovery and diagnostics that never loads private
   * profile signing material. This internal implementation base is exposed
   * through the public {@link ReadOnlyVault} facade.
   */
  public class ReadOnlyVaultHandle implements AutoCloseable {
    private MemorySegment handle;
    protected ReadOnlyVaultHandle(MemorySegment handle) { this.handle = handle; }
    MemorySegment detach() { var value = handle; handle = null; return value; }
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
  /** Caches a profile signing key in the Session Agent. */
  public void cacheProfileSigningKey(String vaultId, String profile, ProfileSigningKeyPair key, long ttlSeconds) {
    operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key.handle, ttlSeconds);
  }
  /** Returns a profile signing key cached by the Session Agent. */
  public ProfileSigningKeyPair profileSigningKey(String vaultId, String profile) {
    return new ProfileSigningKeyPair(operations.vaultAgentGetOwnerSigningKey(vaultId, profile));
  }
  /** Removes a cached profile signing key. */
  public void forgetProfileSigningKey(String vaultId, String profile) {
    operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);
  }

  /** Starts agent activity. */
  public AgentActivity beginAgentActivity(String kind) {
    return new AgentActivity(operations.vaultAgentBeginActivity(kind));
  }
  /** A token kept alive while an operation needs secrets cached by the Session Agent. */
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
  public LockboxSessionHandle openLockboxSession() { return new LockboxSessionHandle(operations.vaultLocal()); }
  /** A session that opens lockboxes by host path, caches passwords, and closes locally used files. */
  public final class LockboxSessionHandle implements AutoCloseable {
    private MemorySegment handle;
    private LockboxSessionHandle(MemorySegment handle) { this.handle = handle; }
    /** Creates with password. */
    public Lockbox createWithPassword(String path, byte[] password) {
      return new Lockbox(operations.vaultCreateLockboxPassword(handle, path, password));
    }
    /** Opens with password. */
    public Lockbox openWithPassword(String path, byte[] password) {
      return new Lockbox(operations.vaultOpenLockboxPassword(handle, path, password));
    }
    /** Creates with content key. */
    public Lockbox createWithContentKey(String path, byte[] key, ProfileSigningKeyPair signingKey) {
      return new Lockbox(operations.vaultCreateLockboxContentKey(handle, path, key, signingKey.handle));
    }
    /** Opens with content key. */
    public Lockbox openWithContentKey(String path, byte[] key, ProfileSigningKeyPair signingKey) {
      return new Lockbox(operations.vaultOpenLockboxContentKey(handle, path, key, signingKey.handle));
    }
    /** Creates for contact. */
    public Lockbox createForContact(String path, ContactPublicKey contact, String name, ProfileSigningKeyPair signingKey) {
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
