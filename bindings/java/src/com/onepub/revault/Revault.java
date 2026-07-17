package com.onepub.revault;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.nio.charset.StandardCharsets;
import revault.bindings.RevaultBindings.AccessSlotLabelList;
import revault.bindings.RevaultBindings.AgentEntryList;
import revault.bindings.RevaultBindings.CacheStats;
import revault.bindings.RevaultBindings.ContactList;
import revault.bindings.RevaultBindings.ErrorDetails;
import revault.bindings.RevaultBindings.FileInspection;
import revault.bindings.RevaultBindings.FormDefinition;
import revault.bindings.RevaultBindings.FormDefinitionList;
import revault.bindings.RevaultBindings.FormFieldList;
import revault.bindings.RevaultBindings.FormRecord;
import revault.bindings.RevaultBindings.FormRecordList;
import revault.bindings.RevaultBindings.ImportStats;
import revault.bindings.RevaultBindings.KeySlotList;
import revault.bindings.RevaultBindings.KnownLockboxList;
import revault.bindings.RevaultBindings.LockboxEntryList;
import revault.bindings.RevaultBindings.OptionalLockboxEntry;
import revault.bindings.RevaultBindings.OptionalFormRecord;
import revault.bindings.RevaultBindings.OptionalFormValue;
import revault.bindings.RevaultBindings.OptionalString;
import revault.bindings.RevaultBindings.OwnerInspection;
import revault.bindings.RevaultBindings.PageInspectionList;
import revault.bindings.RevaultBindings.PathMoveList;
import revault.bindings.RevaultBindings.PlatformStatus;
import revault.bindings.RevaultBindings.ProfileHistory;
import revault.bindings.RevaultBindings.RecoveryReport;
import revault.bindings.RevaultBindings.RuntimeOptions;
import revault.bindings.RevaultBindings.SleepSupport;
import revault.bindings.RevaultBindings.StreamChunkList;
import revault.bindings.RevaultBindings.StringList;
import revault.bindings.RevaultBindings.VariableList;
import revault.bindings.RevaultBindings.VaultBackupManifest;

/** Idiomatic, owned Java API for the complete lockbox and vault binding surface. */
public final class Revault {
  private final BindingOperations operations;

  public Revault() { this(NativeLibrary.resolve()); }

  public Revault(String libraryPath) {
    this(new RevaultNativeApi(SymbolLookup.libraryLookup(libraryPath, Arena.global())));
  }

  public Revault(RevaultNativeApi nativeApi) { operations = new BindingOperations(nativeApi); }

  public record LockboxOptions(String cacheMode, long cacheBytes, String workload, String worker, long jobs) {
    public static LockboxOptions defaults() {
      return new LockboxOptions("bytes", 64L << 20, "interactive", "auto", 0);
    }
  }

  private static void ensureOpen(MemorySegment handle) {
    if (handle == null || handle.address() == 0) throw new IllegalStateException("object is closed");
  }

  public String lastError() { return operations.lastErrorMessage(); }
  public ErrorDetails lastErrorDetails() { return operations.bufferLastErrorDetails(); }
  public int lockboxFormatVersion() { return operations.lockboxFormatVersion(); }
  public int probeLockboxFormatVersion(byte[] value) { return operations.lockboxProbeFormatVersion(value); }
  public int currentVaultStructureVersion() { return operations.vaultStructureVersionCurrent(); }
  public int probeVaultStructureVersion(String root, byte[] password) { return operations.vaultDirectoryProbeStructureVersion(root, password); }

  public final class ContactPublicKey implements AutoCloseable {
    private MemorySegment handle;
    private ContactPublicKey(MemorySegment handle) { this.handle = handle; }
    public byte[] export(String format) { ensureOpen(handle); return operations.vaultKeyExportPublic(handle, format); }
    public byte[] fingerprint() { ensureOpen(handle); return operations.vaultKeyFingerprint(handle); }
    public WrappedContactKey encrypt(byte[] contentKey) {
      ensureOpen(handle); return new WrappedContactKey(operations.keyContactEncrypt(handle, contentKey));
    }
    @Override public void close() {
      if (handle != null) { operations.keyContactPublicFree(handle); handle = null; }
    }
  }

  public final class WrappedContactKey implements AutoCloseable {
    private MemorySegment handle;
    private WrappedContactKey(MemorySegment handle) { this.handle = handle; }
    public byte[] publicBytes() { ensureOpen(handle); return operations.keyContactWrappedPublic(handle); }
    public byte[] ciphertext() { ensureOpen(handle); return operations.keyContactWrappedCiphertext(handle); }
    public byte[] encryptedBytes() { ensureOpen(handle); return operations.keyContactWrappedEncrypted(handle); }
    @Override public void close() {
      if (handle != null) { operations.keyContactWrappedFree(handle); handle = null; }
    }
  }

  public final class ContactKeyPair implements AutoCloseable {
    private MemorySegment handle;
    private ContactKeyPair(MemorySegment handle) { this.handle = handle; }
    public byte[] publicBytes() { ensureOpen(handle); return operations.keyContactPublic(handle); }
    public byte[] privateRecord() { ensureOpen(handle); return operations.keyContactPrivate(handle); }
    public ContactPublicKey publicKey() { return contactPublicKey(publicBytes()); }
    public byte[] export(String format) { ensureOpen(handle); return operations.vaultKeyExportPrivate(handle, format); }
    public byte[] decrypt(WrappedContactKey wrapped) {
      ensureOpen(handle); ensureOpen(wrapped.handle); return operations.keyContactDecrypt(handle, wrapped.handle);
    }
    @Override public void close() {
      if (handle != null) { operations.keyContactFree(handle); handle = null; }
    }
  }

  public final class SigningPublicKey implements AutoCloseable {
    private MemorySegment handle;
    private SigningPublicKey(MemorySegment handle) { this.handle = handle; }
    @Override public void close() {
      if (handle != null) { operations.keySigningPublicFree(handle); handle = null; }
    }
  }

  public final class SigningKeyPair implements AutoCloseable {
    private MemorySegment handle;
    private SigningKeyPair(MemorySegment handle) { this.handle = handle; }
    public byte[] publicBytes() { ensureOpen(handle); return operations.keySigningPublic(handle); }
    public byte[] privateRecord() { ensureOpen(handle); return operations.keySigningPrivate(handle); }
    public SigningPublicKey publicKey() { return signingPublicKey(publicBytes()); }
    @Override public void close() {
      if (handle != null) { operations.keySigningFree(handle); handle = null; }
    }
  }

  public ContactKeyPair generateContactKeyPair() { return new ContactKeyPair(operations.keyContactGenerate()); }
  public ContactKeyPair contactKeyPairFromPrivate(byte[] value) { return new ContactKeyPair(operations.keyContactFromPrivate(value)); }
  public ContactKeyPair importContactKeyPair(byte[] value) { return new ContactKeyPair(operations.vaultKeyImportPrivate(value)); }
  public ContactPublicKey contactPublicKey(byte[] value) { return new ContactPublicKey(operations.keyContactPublicFromBytes(value)); }
  public ContactPublicKey importContactPublicKey(byte[] value) { return new ContactPublicKey(operations.vaultKeyImportPublic(value)); }
  public SigningKeyPair generateSigningKeyPair() { return new SigningKeyPair(operations.keySigningGenerate()); }
  public SigningKeyPair signingKeyPairFromPrivate(byte[] value) { return new SigningKeyPair(operations.keySigningFromPrivate(value)); }
  public SigningPublicKey signingPublicKey(byte[] value) { return new SigningPublicKey(operations.keySigningPublicFromBytes(value)); }

  public String formatKeyHex(byte[] value) { return operations.vaultKeyFormatHex(value); }
  public byte[] decodeKeyHex(String value) { return operations.vaultKeyDecodeHex(value); }
  public String formatKeyCrockford(byte[] value) { return operations.vaultKeyFormatCrockford(value); }
  public String formatKeyCrockfordReading(String value) { return operations.vaultKeyFormatCrockfordReading(value); }
  public byte[] decodeKeyCrockford(String value) { return operations.vaultKeyDecodeCrockford(value); }
  public String hexEncode(byte[] value) { return operations.vaultKeyHexEncode(value); }
  public byte[] hexDecode(String value) { return operations.vaultKeyHexDecode(value); }

  public Lockbox createLockbox(byte[] key) { return new Lockbox(operations.lockboxCreate(key)); }
  public Lockbox createLockbox(byte[] key, LockboxOptions options) {
    return new Lockbox(operations.lockboxCreateWithOptions(key, options.cacheMode(), options.cacheBytes(),
        options.workload(), options.worker(), options.jobs()));
  }
  public Lockbox createLockboxWithPassword(byte[] password) { return new Lockbox(operations.lockboxCreatePassword(password)); }
  public Lockbox createLockboxForContact(ContactPublicKey contact) {
    ensureOpen(contact.handle); return new Lockbox(operations.lockboxCreateContact(contact.handle));
  }
  public Lockbox createSignedLockbox(byte[] contentKey, SigningKeyPair signingKey) {
    ensureOpen(signingKey.handle); return new Lockbox(operations.lockboxCreateWithSigningKey(contentKey, signingKey.handle));
  }
  public Lockbox openLockbox(byte[] archive, byte[] key) { return new Lockbox(operations.lockboxOpen(archive, key)); }
  public Lockbox openLockbox(byte[] archive, byte[] key, LockboxOptions options) {
    return new Lockbox(operations.lockboxOpenWithOptions(archive, key, options.cacheMode(),
        options.cacheBytes(), options.workload(), options.worker(), options.jobs()));
  }
  public Lockbox openLockboxWithPassword(byte[] archive, byte[] password) {
    return new Lockbox(operations.lockboxOpenPassword(archive, password));
  }
  public Lockbox openLockboxForContact(byte[] archive, ContactKeyPair contact) {
    ensureOpen(contact.handle); return new Lockbox(operations.lockboxOpenContact(archive, contact.handle));
  }

  public FileInspection inspectLockboxFile(String path) { return operations.lockboxInspectFile(path); }
  public RecoveryReport scanLockboxPath(String path, byte[] key) { return operations.lockboxRecoveryScanPath(path, key); }
  public RecoveryReport scanLockbox(byte[] archive, byte[] key) { return operations.lockboxRecoveryScan(archive, key); }
  public Lockbox salvageLockbox(byte[] archive, byte[] key, SigningKeyPair signingKey) {
    return new Lockbox(operations.lockboxRecoverySalvage(archive, key,
        signingKey == null ? MemorySegment.NULL : signingKey.handle));
  }

  public final class Lockbox implements AutoCloseable {
    private MemorySegment handle;
    private Lockbox(MemorySegment handle) { this.handle = handle; }
    public void addFile(String path, byte[] value, boolean replace) { operations.lockboxAddFile(handle, path, value, replace); }
    public void addFile(String path, byte[] value, int permissions, boolean replace) { operations.lockboxAddFileWithPermissions(handle, path, value, permissions, replace); }
    public byte[] getFile(String path) { return operations.lockboxGetFile(handle, path); }
    public void extractFile(String source, String destination, boolean replace) { operations.lockboxExtractFile(handle, source, destination, replace); }
    public void extractDirectory(String destination, long maxFileBytes, long maxTotalBytes, long maxFiles,
        boolean restoreSymlinks, boolean restorePermissions, boolean overwrite) {
      operations.lockboxExtractDirectory(handle, destination, maxFileBytes, maxTotalBytes, maxFiles,
          restoreSymlinks, restorePermissions, overwrite);
    }
    public StreamChunkList streamContent(boolean physical) { return operations.lockboxStreamContent(handle, physical); }
    public CacheStats cacheStats() { return operations.lockboxCacheStats(handle); }
    public ImportStats importStats() { return operations.lockboxImportStats(handle); }
    public void resetImportStats() { operations.lockboxResetImportStats(handle); }
    public PageInspectionList pageInspection() { return operations.lockboxPageInspection(handle); }
    public RecoveryReport recoveryReport() { return operations.lockboxRecoveryReport(handle); }
    public String renderRecoveryReport(boolean verbose, long maxEntries) { return operations.lockboxRecoveryReportRender(handle, verbose, maxEntries); }
    public long storageLength() { return operations.lockboxStorageLen(handle); }
    public void setWorkloadProfile(String profile) { operations.lockboxSetWorkloadProfile(handle, profile); }
    public void setWorkerPolicy(String mode, long jobs) { operations.lockboxSetWorkerPolicy(handle, mode, jobs); }
    public RuntimeOptions runtimeOptions() { return operations.lockboxRuntimeOptions(handle); }
    public void commit() { operations.lockboxCommit(handle); }
    public void createDirectory(String path, boolean parents) { operations.lockboxCreateDir(handle, path, parents); }
    public void delete(String path) { operations.lockboxDelete(handle, path); }
    public void removeDirectory(String path, boolean recursive) { operations.lockboxRemoveDir(handle, path, recursive); }
    public void createParentDirectories(String path) { operations.lockboxCreateParentDirs(handle, path); }
    public void rename(String from, String to) { operations.lockboxRename(handle, from, to); }
    public LockboxEntryList list(String path, boolean recursive) { return operations.lockboxList(handle, path, recursive); }
    public LockboxEntryList list(String path, String glob, boolean recursive, boolean includeFiles,
        boolean includeSymlinks, boolean includeDirectories, long limit) {
      return operations.lockboxListWithOptions(handle, path, glob, recursive, includeFiles,
          includeSymlinks, includeDirectories, limit);
    }
    public OptionalLockboxEntry stat(String path) { return operations.lockboxStat(handle, path); }
    public void setVariable(String name, String value) { operations.lockboxSetVariable(handle, name, value); }
    public void setSecretVariable(String name, byte[] value) { operations.lockboxSetSecretVariable(handle, name, value); }
    public String getVariable(String name) { var value = operations.lockboxGetVariable(handle, name); return value.getPresent() ? value.getValue() : null; }
    public <T> T withSecretVariable(String name, BindingOperations.SecretCallback<T> callback) { return operations.lockboxWithSecretVariable(handle, name, callback); }
    public void deleteVariable(String name) { operations.lockboxDeleteVariable(handle, name); }
    public void moveVariables(PathMoveList moves) { operations.lockboxMoveVariables(handle, moves.toByteArray()); }
    public VariableList listVariables() { return operations.lockboxListVariables(handle); }
    public OptionalString variableSensitivity(String name) { return operations.lockboxVariableSensitivity(handle, name); }
    public void addSymlink(String path, String target, boolean replace) { operations.lockboxAddSymlink(handle, path, target, replace); }
    public String symlinkTarget(String path) { return operations.lockboxGetSymlinkTarget(handle, path); }
    public byte[] id() { return operations.lockboxId(handle); }
    public boolean exists(String path) { return operations.lockboxExists(handle, path); }
    public boolean isDirectory(String path) { return operations.lockboxIsDir(handle, path); }
    public int permissions(String path) { return operations.lockboxPermissions(handle, path); }
    public void setPermissions(String path, int value) { operations.lockboxSetPermissions(handle, path, value); }
    public byte[] readRange(String path, long offset, long length) { return operations.lockboxReadRange(handle, path, offset, length); }
    public long addPassword(byte[] password) {
      long result = operations.lockboxAddPassword(handle, password);
      if (result == -1L) throw new IllegalStateException(operations.lastErrorMessage());
      return result;
    }
    public long addContact(ContactPublicKey contact, String name) {
      long result = operations.lockboxAddContact(handle, contact.handle, name);
      if (result == -1L) throw new IllegalStateException(operations.lastErrorMessage());
      return result;
    }
    public void deleteKey(long id) { operations.lockboxDeleteKey(handle, id); }
    public KeySlotList listKeySlots() { return operations.lockboxListKeySlots(handle); }
    public void setOwnerSigningKey(SigningKeyPair key) { operations.lockboxSetOwnerSigningKey(handle, key.handle); }
    public OwnerInspection ownerInspection() { return operations.lockboxOwnerInspection(handle); }
    public FormDefinition defineForm(String alias, String name, String description, FormFieldList fields) {
      return operations.lockboxDefineForm(handle, alias, name, description, fields.toByteArray());
    }
    public FormDefinitionList listFormDefinitions() { return operations.lockboxListFormDefinitions(handle); }
    public FormDefinition resolveForm(String reference) { return operations.lockboxResolveForm(handle, reference); }
    public FormDefinitionList listFormRevisions(String typeId) { return operations.lockboxListFormRevisions(handle, typeId); }
    public FormRecord createFormRecord(String path, String typeReference, String name) {
      return operations.lockboxCreateFormRecord(handle, path, typeReference, name);
    }
    public void setFormField(String path, String field, String value) {
      operations.lockboxSetFormField(handle, path, field, value);
    }
    public void setSecretFormField(String path, String field, byte[] value) { operations.lockboxSetSecretFormField(handle, path, field, value); }
    public FormRecordList listFormRecords() { return operations.lockboxListFormRecords(handle); }
    public OptionalFormRecord getFormRecord(String path) { return operations.lockboxGetFormRecord(handle, path); }
    public void deleteFormRecord(String path) { operations.lockboxDeleteFormRecord(handle, path); }
    public void moveFormRecords(PathMoveList moves) { operations.lockboxMoveFormRecords(handle, moves.toByteArray()); }
    public OptionalFormValue getFormField(String path, String field) { return operations.lockboxGetFormField(handle, path, field); }
    public <T> T withSecretFormField(String path, String field, BindingOperations.SecretCallback<T> callback) { return operations.lockboxWithSecretFormField(handle, path, field, callback); }
    public byte[] bytes() { return operations.lockboxToBytes(handle); }
    @Override public void close() { if (handle != null) { operations.lockboxFree(handle); handle = null; } }
  }

  public VaultDirectory openVaultDirectory(String root, byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryOpen(root, password));
  }
  public ReadOnlyVaultDirectory openReadOnlyVaultDirectory(String root, byte[] password) {
    return new ReadOnlyVaultDirectory(operations.vaultReadOnlyOpen(root, password));
  }
  public ReadOnlyVaultDirectory openDefaultReadOnlyVaultDirectory(byte[] password) {
    return new ReadOnlyVaultDirectory(operations.vaultReadOnlyOpenDefault(password));
  }
  public VaultDirectory openOrCreateVaultDirectory(String root, byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryOpenOrCreate(root, password));
  }
  public VaultDirectory replaceVaultDirectory(String root, byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryReplace(root, password));
  }
  public VaultDirectory openOrCreateDefaultVaultDirectory(byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryOpenOrCreateDefault(password));
  }
  public VaultDirectory replaceDefaultVaultDirectory(byte[] password) {
    return new VaultDirectory(operations.vaultDirectoryReplaceDefault(password));
  }
  public void changeVaultDirectoryPassword(String root, byte[] oldPassword, byte[] newPassword) {
    operations.vaultDirectoryChangePassword(root, oldPassword, newPassword);
  }
  public void changeDefaultVaultDirectoryPassword(byte[] oldPassword, byte[] newPassword) {
    operations.vaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
  }
  public String defaultVaultDirectory() { return operations.vaultDefaultDirectory(); }
  public String defaultVaultPath() { return operations.vaultDefaultPath(); }
  public VaultBackupManifest backupDefaultVault(String path, boolean overwrite) {
    return operations.vaultBackupDefault(path, overwrite);
  }
  public VaultBackupManifest restoreDefaultVault(String path, boolean overwrite) {
    return operations.vaultRestoreDefault(path, overwrite);
  }

  public final class VaultDirectory implements AutoCloseable {
    private MemorySegment handle;
    private VaultDirectory(MemorySegment handle) { this.handle = handle; }
    public String root() { return operations.vaultDirectoryRoot(handle); }
    public int structureVersion() { return operations.vaultDirectoryStructureVersion(handle); }
    public StringList listPrivateKeys() { return operations.vaultDirectoryListPrivateKeys(handle); }
    public StringList listPrivateKeyNames() { return operations.vaultDirectoryListPrivateKeyNames(handle); }
    public StringList listContactNames() { return operations.vaultDirectoryListContactNames(handle); }
    public StringList listFormAliases() { return operations.vaultDirectoryListFormAliases(handle); }
    public boolean privateKeyExists(String name) { return operations.vaultDirectoryPrivateKeyExists(handle, name); }
    public void deletePrivateKey(String name) { operations.vaultDirectoryDeletePrivateKey(handle, name); }
    public void storePrivateKey(String name, ContactKeyPair key) { operations.vaultDirectoryStorePrivateKey(handle, name, key.handle); }
    public ContactKeyPair loadPrivateKey(String name) { return new ContactKeyPair(operations.vaultDirectoryLoadPrivateKey(handle, name)); }
    public ContactKeyPair loadPrivateKeyGeneration(String name, int index) {
      return new ContactKeyPair(operations.vaultDirectoryLoadPrivateKeyGeneration(handle, name, (short) index));
    }
    public void storeContact(String name, ContactPublicKey key) { operations.vaultDirectoryStoreContact(handle, name, key.handle); }
    public ContactPublicKey loadContact(String name) { return new ContactPublicKey(operations.vaultDirectoryLoadContact(handle, name)); }
    public boolean contactExists(String name) { return operations.vaultDirectoryContactExists(handle, name); }
    public void deleteContact(String name) { operations.vaultDirectoryDeleteContact(handle, name); }
    public ContactList listContacts() { return operations.vaultDirectoryListContacts(handle); }
    public void storeProfileEmail(String name, String email) { operations.vaultDirectoryStoreProfileEmail(handle, name, email); }
    public OptionalString profileEmail(String name) { return operations.vaultDirectoryProfileEmail(handle, name); }
    public void storeBackup(byte[] id, byte[] value) { operations.vaultDirectoryStoreBackup(handle, id, value); }
    public byte[] loadBackup(byte[] id) { return operations.vaultDirectoryLoadBackup(handle, id); }
    public long backupCount() { return operations.vaultDirectoryBackupCount(handle); }
    public void restorePrivateKey(String name, ContactKeyPair key, SigningKeyPair signingKey, boolean overwrite) {
      operations.vaultDirectoryRestorePrivateKey(handle, name, key.handle, signingKey.handle, overwrite);
    }
    public SigningKeyPair loadOwnerSigningKey(String name) {
      return new SigningKeyPair(operations.vaultDirectoryLoadOwnerSigningKey(handle, name));
    }
    public SigningKeyPair loadOwnerSigningKeyGeneration(String name, int index) {
      return new SigningKeyPair(operations.vaultDirectoryLoadOwnerSigningKeyGeneration(handle, name, (short) index));
    }
    public void storeContactSigningKey(String name, SigningPublicKey key) {
      operations.vaultDirectoryStoreContactSigningKey(handle, name, key.handle);
    }
    public SigningPublicKey loadContactSigningKey(String name) {
      return new SigningPublicKey(operations.vaultDirectoryLoadContactSigningKey(handle, name));
    }
    public ProfileHistory listProfileGenerations(String name) {
      return operations.vaultDirectoryListProfileGenerations(handle, name);
    }
    public ProfileHistory rotatePrivateKey(String name) { return operations.vaultDirectoryRotatePrivateKey(handle, name); }
    public void rememberLockbox(byte[] id, String path) { operations.vaultDirectoryRememberLockbox(handle, id, path); }
    public KnownLockboxList listKnownLockboxes() { return operations.vaultDirectoryListKnownLockboxes(handle); }
    public void forgetLockbox(String path) { operations.vaultDirectoryForgetLockbox(handle, path); }
    public void rememberAccessSlotLabel(byte[] id, long slotId, String name) {
      operations.vaultDirectoryRememberAccessSlotLabel(handle, id, slotId, name);
    }
    public AccessSlotLabelList listAccessSlotLabels(byte[] id) { return operations.vaultDirectoryListAccessSlotLabels(handle, id); }
    public AccessSlotLabelList findAccessSlotLabels(byte[] id, String name) {
      return operations.vaultDirectoryFindAccessSlotLabels(handle, id, name);
    }
    public void forgetAccessSlotLabel(byte[] id, long slotId) { operations.vaultDirectoryForgetAccessSlotLabel(handle, id, slotId); }
    public FormDefinition defineForm(String alias, String name, String description, FormFieldList fields) {
      return operations.vaultDirectoryDefineForm(handle, alias, name, description, fields.toByteArray());
    }
    public FormDefinition resolveForm(String reference) { return operations.vaultDirectoryResolveForm(handle, reference); }
    public FormDefinitionList listForms() { return operations.vaultDirectoryListForms(handle); }
    public FormDefinitionList listFormRevisions(String typeId) { return operations.vaultDirectoryListFormRevisions(handle, typeId); }
    public long seedForms() { return operations.vaultDirectorySeedForms(handle); }
    public void rememberPassword(byte[] id, byte[] password) { operations.vaultDirectoryRememberPassword(handle, id, password); }
    public byte[] rememberedPassword(byte[] id) { return operations.vaultDirectoryRememberedPassword(handle, id); }
    @Override public void close() { if (handle != null) { operations.vaultDirectoryFree(handle); handle = null; } }
  }

  public final class ReadOnlyVaultDirectory implements AutoCloseable {
    private MemorySegment handle;
    private ReadOnlyVaultDirectory(MemorySegment handle) { this.handle = handle; }
    public StringList listProfileNames() { return operations.vaultReadOnlyListProfileNames(handle); }
    public StringList listContactNames() { return operations.vaultReadOnlyListContactNames(handle); }
    public StringList listFormAliases() { return operations.vaultReadOnlyListFormAliases(handle); }
    public KnownLockboxList listKnownLockboxes() { return operations.vaultReadOnlyListKnownLockboxes(handle); }
    @Override public void close() { if (handle != null) { operations.vaultReadOnlyFree(handle); handle = null; } }
  }

  public boolean agentIsRunning() { return operations.vaultIsRunning(); }
  public void serveAgent() { operations.vaultAgentServe(); }
  public void verifyAgentTransport() { operations.vaultAgentVerifyTransport(); }
  public void forgetAllAgentSecrets() { operations.vaultForgetAll(); }
  public void stopAgent() { operations.vaultAgentStop(); }
  public void startAgent() { operations.vaultAgentStart(); }
  public void putAgentKey(byte[] id, byte[] key) { operations.vaultAgentPut(id, key); }
  public byte[] getAgentKey(byte[] id) { return operations.vaultAgentGet(id); }
  public void forgetAgentKey(byte[] id) { operations.vaultAgentForget(id); }
  public AgentEntryList listAgentKeys() { return operations.vaultAgentList(); }
  public SleepSupport agentSleepSupport() { return operations.vaultAgentSleepSupport(); }
  public String agentLogPath() { return operations.vaultAgentLogPath(); }
  public String agentLogDestination() { return operations.vaultAgentLogDestination(); }
  public void putAgentVaultUnlockKey(String vaultId, byte[] key, long ttlSeconds) {
    operations.vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
  }
  public byte[] getAgentVaultUnlockKey(String vaultId) { return operations.vaultAgentGetVaultUnlockKey(vaultId); }
  public void forgetAgentVaultUnlockKey(String vaultId) { operations.vaultAgentForgetVaultUnlockKey(vaultId); }
  public void putAgentOwnerSigningKey(String vaultId, String profile, SigningKeyPair key, long ttlSeconds) {
    operations.vaultAgentPutOwnerSigningKey(vaultId, profile, key.handle, ttlSeconds);
  }
  public SigningKeyPair getAgentOwnerSigningKey(String vaultId, String profile) {
    return new SigningKeyPair(operations.vaultAgentGetOwnerSigningKey(vaultId, profile));
  }
  public void forgetAgentOwnerSigningKey(String vaultId, String profile) {
    operations.vaultAgentForgetOwnerSigningKey(vaultId, profile);
  }

  public AgentActivity beginAgentActivity(String kind) {
    return new AgentActivity(operations.vaultAgentBeginActivity(kind));
  }
  public final class AgentActivity implements AutoCloseable {
    private MemorySegment handle;
    private AgentActivity(MemorySegment handle) { this.handle = handle; }
    @Override public void close() {
      if (handle != null) { operations.vaultAgentEndActivity(handle); handle = null; }
    }
  }

  public PlatformStatus platformStatus() { return operations.vaultPlatformStatus(); }
  public void setPlatformScope(String scope) { operations.vaultPlatformSetScope(scope); }
  public void enablePlatformStore() { operations.vaultPlatformEnable(); }
  public void disablePlatformStore() { operations.vaultPlatformDisable(); }
  public boolean platformStoreDisabled() { return operations.vaultPlatformDisabled(); }
  public void putPlatformPassword(byte[] password) { operations.vaultPlatformPutPassword(password); }
  public byte[] getPlatformPassword() { return operations.vaultPlatformGetPassword(); }
  public void forgetPlatformPassword() { operations.vaultPlatformForgetPassword(); }

  public LocalVault openLocalVault() { return new LocalVault(operations.vaultLocal()); }
  public final class LocalVault implements AutoCloseable {
    private MemorySegment handle;
    private LocalVault(MemorySegment handle) { this.handle = handle; }
    public Lockbox createWithPassword(String path, byte[] password) {
      return new Lockbox(operations.vaultCreateLockboxPassword(handle, path, password));
    }
    public Lockbox openWithPassword(String path, byte[] password) {
      return new Lockbox(operations.vaultOpenLockboxPassword(handle, path, password));
    }
    public Lockbox createWithContentKey(String path, byte[] key, SigningKeyPair signingKey) {
      return new Lockbox(operations.vaultCreateLockboxContentKey(handle, path, key, signingKey.handle));
    }
    public Lockbox openWithContentKey(String path, byte[] key, SigningKeyPair signingKey) {
      return new Lockbox(operations.vaultOpenLockboxContentKey(handle, path, key, signingKey.handle));
    }
    public Lockbox createForContact(String path, ContactPublicKey contact, String name, SigningKeyPair signingKey) {
      return new Lockbox(operations.vaultCreateLockboxContact(handle, path, contact.handle, name, signingKey.handle));
    }
    public void cachePassword(String path, byte[] password, long ttlSeconds) {
      operations.vaultCacheLockboxPassword(handle, path, password, ttlSeconds);
    }
    public void closeLockbox(String path) { operations.vaultCloseLockbox(handle, path); }
    public void closeAll() { operations.vaultCloseAll(handle); }
    @Override public void close() { if (handle != null) { operations.vaultFree(handle); handle = null; } }
  }
}
