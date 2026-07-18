using System.Text;
using Google.Protobuf;
using Revault.Bindings;

namespace Revault;

/// <summary>Owned, class-based API for all lockbox and vault operations.</summary>
public sealed class Vault
{
    private readonly BindingOperations operations = new();
    private static void Open(IntPtr handle) { if (handle == IntPtr.Zero) throw new ObjectDisposedException("native object"); }

    public sealed record LockboxOptions(string CacheMode, ulong CacheBytes, string Workload, string Worker, nuint Jobs)
    {
        public static LockboxOptions Defaults => new("bytes", 64UL << 20, "interactive", "auto", 0);
    }

    public string LastError => operations.LastErrorMessage();
    public ErrorDetails LastErrorDetails() => operations.BufferLastErrorDetails();
    public ushort LockboxFormatVersion => (ushort)operations.LockboxFormatVersion();
    public ushort ProbeLockboxFormatVersion(byte[] value) => (ushort)operations.LockboxProbeFormatVersion(value);
    public uint CurrentVaultStructureVersion => (uint)operations.VaultStructureVersionCurrent();
    public uint ProbeVaultStructureVersion(string root, byte[] password) => (uint)operations.VaultDirectoryProbeStructureVersion(root, password);

    public sealed class ContactPublicKey : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal ContactPublicKey(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public byte[] Export(string format) { Open(Handle); return owner.operations.VaultKeyExportPublic(Handle, format); }
        public byte[] Fingerprint() { Open(Handle); return owner.operations.VaultKeyFingerprint(Handle); }
        public WrappedContactKey Encrypt(byte[] contentKey) => new(owner, owner.operations.KeyContactEncrypt(Handle, contentKey));
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeyContactPublicFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~ContactPublicKey() => Dispose();
    }

    public sealed class WrappedContactKey : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal WrappedContactKey(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public byte[] PublicBytes() => owner.operations.KeyContactWrappedPublic(Handle);
        public byte[] Ciphertext() => owner.operations.KeyContactWrappedCiphertext(Handle);
        public byte[] EncryptedBytes() => owner.operations.KeyContactWrappedEncrypted(Handle);
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeyContactWrappedFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~WrappedContactKey() => Dispose();
    }

    public sealed class ContactKeyPair : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal ContactKeyPair(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public byte[] PublicBytes() => owner.operations.KeyContactPublic(Handle);
        public byte[] PrivateRecord() => owner.operations.KeyContactPrivate(Handle);
        public ContactPublicKey PublicKey() => owner.ContactPublicKeyFromBytes(PublicBytes());
        public byte[] Export(string format) => owner.operations.VaultKeyExportPrivate(Handle, format);
        public byte[] Decrypt(WrappedContactKey wrapped) => owner.operations.KeyContactDecrypt(Handle, wrapped.Handle);
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeyContactFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~ContactKeyPair() => Dispose();
    }

    public sealed class SigningPublicKey : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal SigningPublicKey(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeySigningPublicFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~SigningPublicKey() => Dispose();
    }

    public sealed class SigningKeyPair : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal SigningKeyPair(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public byte[] PublicBytes() => owner.operations.KeySigningPublic(Handle);
        public byte[] PrivateRecord() => owner.operations.KeySigningPrivate(Handle);
        public SigningPublicKey PublicKey() => owner.SigningPublicKeyFromBytes(PublicBytes());
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeySigningFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~SigningKeyPair() => Dispose();
    }

    public ContactKeyPair GenerateContactKeyPair() => new(this, operations.KeyContactGenerate());
    public ContactKeyPair ContactKeyPairFromPrivate(byte[] value) => new(this, operations.KeyContactFromPrivate(value));
    public ContactKeyPair ImportContactKeyPair(byte[] value) => new(this, operations.VaultKeyImportPrivate(value));
    public ContactPublicKey ContactPublicKeyFromBytes(byte[] value) => new(this, operations.KeyContactPublicFromBytes(value));
    public ContactPublicKey ImportContactPublicKey(byte[] value) => new(this, operations.VaultKeyImportPublic(value));
    public SigningKeyPair GenerateSigningKeyPair() => new(this, operations.KeySigningGenerate());
    public SigningKeyPair SigningKeyPairFromPrivate(byte[] value) => new(this, operations.KeySigningFromPrivate(value));
    public SigningPublicKey SigningPublicKeyFromBytes(byte[] value) => new(this, operations.KeySigningPublicFromBytes(value));

    public string FormatKeyHex(byte[] value) => operations.VaultKeyFormatHex(value);
    public byte[] DecodeKeyHex(string value) => operations.VaultKeyDecodeHex(value);
    public string FormatKeyCrockford(byte[] value) => operations.VaultKeyFormatCrockford(value);
    public string FormatKeyCrockfordReading(string value) => operations.VaultKeyFormatCrockfordReading(value);
    public byte[] DecodeKeyCrockford(string value) => operations.VaultKeyDecodeCrockford(value);
    public string HexEncode(byte[] value) => operations.VaultKeyHexEncode(value);
    public byte[] HexDecode(string value) => operations.VaultKeyHexDecode(value);

    public Lockbox CreateLockbox(byte[] key) => new(this, operations.LockboxCreate(key));
    public Lockbox CreateLockbox(byte[] key, LockboxOptions options) => new(this,
        operations.LockboxCreateWithOptions(key, options.CacheMode, options.CacheBytes, options.Workload, options.Worker, options.Jobs));
    public Lockbox CreateLockboxWithPassword(byte[] password) => new(this, operations.LockboxCreatePassword(password));
    public Lockbox CreateLockboxForContact(ContactPublicKey contact) => new(this, operations.LockboxCreateContact(contact.Handle));
    public Lockbox CreateSignedLockbox(byte[] key, SigningKeyPair signing) => new(this, operations.LockboxCreateWithSigningKey(key, signing.Handle));
    public Lockbox OpenLockbox(byte[] archive, byte[] key) => new(this, operations.LockboxOpen(archive, key));
    public Lockbox OpenLockbox(byte[] archive, byte[] key, LockboxOptions options) => new(this,
        operations.LockboxOpenWithOptions(archive, key, options.CacheMode, options.CacheBytes, options.Workload, options.Worker, options.Jobs));
    public Lockbox OpenLockboxWithPassword(byte[] archive, byte[] password) => new(this, operations.LockboxOpenPassword(archive, password));
    public Lockbox OpenLockboxForContact(byte[] archive, ContactKeyPair contact) => new(this, operations.LockboxOpenContact(archive, contact.Handle));
    public FileInspection InspectLockboxFile(string path) => operations.LockboxInspectFile(path);
    public RecoveryReport ScanLockboxPath(string path, byte[] key) => operations.LockboxRecoveryScanPath(path, key);
    public RecoveryReport ScanLockbox(byte[] archive, byte[] key) => operations.LockboxRecoveryScan(archive, key);
    public Lockbox SalvageLockbox(byte[] archive, byte[] key, SigningKeyPair? signing = null) =>
        new(this, operations.LockboxRecoverySalvage(archive, key, signing?.Handle ?? IntPtr.Zero));

    public sealed class Lockbox : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal Lockbox(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public void AddFile(string path, byte[] value, bool replace = false) => owner.operations.LockboxAddFile(Handle, path, value, replace);
        public void AddFile(string path, byte[] value, uint permissions, bool replace = false) => owner.operations.LockboxAddFileWithPermissions(Handle, path, value, permissions, replace);
        public byte[] GetFile(string path) => owner.operations.LockboxGetFile(Handle, path);
        public void ExtractFile(string source, string destination, bool replace = false) => owner.operations.LockboxExtractFile(Handle, source, destination, replace);
        public void ExtractDirectory(string destination, ulong maxFileBytes, ulong maxTotalBytes, nuint maxFiles,
            bool restoreSymlinks, bool restorePermissions, bool overwrite) => owner.operations.LockboxExtractDirectory(
                Handle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite);
        public StreamChunkList StreamContent(bool physical = false) => owner.operations.LockboxStreamContent(Handle, physical);
        public CacheStats CacheStats() => owner.operations.LockboxCacheStats(Handle);
        public ImportStats ImportStats() => owner.operations.LockboxImportStats(Handle);
        public void ResetImportStats() => owner.operations.LockboxResetImportStats(Handle);
        public PageInspectionList PageInspection() => owner.operations.LockboxPageInspection(Handle);
        public RecoveryReport RecoveryReport() => owner.operations.LockboxRecoveryReport(Handle);
        public string RenderRecoveryReport(bool verbose, nuint maxEntries) => owner.operations.LockboxRecoveryReportRender(Handle, verbose, maxEntries);
        public ulong StorageLength => owner.operations.LockboxStorageLen(Handle);
        public void SetWorkloadProfile(string profile) => owner.operations.LockboxSetWorkloadProfile(Handle, profile);
        public void SetWorkerPolicy(string mode, nuint jobs) => owner.operations.LockboxSetWorkerPolicy(Handle, mode, jobs);
        public RuntimeOptions RuntimeOptions() => owner.operations.LockboxRuntimeOptions(Handle);
        public void Commit() => owner.operations.LockboxCommit(Handle);
        public void CreateDirectory(string path, bool parents = false) => owner.operations.LockboxCreateDir(Handle, path, parents);
        public void Delete(string path) => owner.operations.LockboxDelete(Handle, path);
        public void RemoveDirectory(string path, bool recursive = false) => owner.operations.LockboxRemoveDir(Handle, path, recursive);
        public void CreateParentDirectories(string path) => owner.operations.LockboxCreateParentDirs(Handle, path);
        public void Rename(string from, string to) => owner.operations.LockboxRename(Handle, from, to);
        public LockboxEntryList List(string path = "/", bool recursive = false) => owner.operations.LockboxList(Handle, path, recursive);
        public LockboxEntryList List(string path, string glob, bool recursive, bool includeFiles,
            bool includeSymlinks, bool includeDirectories, nuint limit) => owner.operations.LockboxListWithOptions(
                Handle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit);
        public OptionalLockboxEntry Stat(string path) => owner.operations.LockboxStat(Handle, path);
        public void SetVariable(string name, string value, bool secret = false) => owner.operations.LockboxSetVariable(Handle, name, value, secret);
        public string GetVariable(string name) => owner.operations.LockboxGetVariable(Handle, name);
        public void DeleteVariable(string name) => owner.operations.LockboxDeleteVariable(Handle, name);
        public void MoveVariables(PathMoveList moves) => owner.operations.LockboxMoveVariables(Handle, moves.ToByteArray());
        public VariableList ListVariables() => owner.operations.LockboxListVariables(Handle);
        public OptionalString VariableSensitivity(string name) => owner.operations.LockboxVariableSensitivity(Handle, name);
        public void AddSymlink(string path, string target, bool replace = false) => owner.operations.LockboxAddSymlink(Handle, path, target, replace);
        public string SymlinkTarget(string path) => owner.operations.LockboxGetSymlinkTarget(Handle, path);
        public byte[] Id => owner.operations.LockboxId(Handle);
        public bool Exists(string path) => owner.operations.LockboxExists(Handle, path);
        public bool IsDirectory(string path) => owner.operations.LockboxIsDir(Handle, path);
        public uint Permissions(string path) => owner.operations.LockboxPermissions(Handle, path);
        public void SetPermissions(string path, uint value) => owner.operations.LockboxSetPermissions(Handle, path, value);
        public byte[] ReadRange(string path, ulong offset, ulong length) => owner.operations.LockboxReadRange(Handle, path, offset, length);
        public ulong AddPassword(byte[] password) { var id = owner.operations.LockboxAddPassword(Handle, password); if (id == ulong.MaxValue) throw new InvalidOperationException(owner.LastError); return id; }
        public ulong AddContact(ContactPublicKey contact, string name) { var id = owner.operations.LockboxAddContact(Handle, contact.Handle, name); if (id == ulong.MaxValue) throw new InvalidOperationException(owner.LastError); return id; }
        public void DeleteKey(ulong id) => owner.operations.LockboxDeleteKey(Handle, id);
        public KeySlotList ListKeySlots() => owner.operations.LockboxListKeySlots(Handle);
        public void SetOwnerSigningKey(SigningKeyPair key) => owner.operations.LockboxSetOwnerSigningKey(Handle, key.Handle);
        public OwnerInspection OwnerInspection() => owner.operations.LockboxOwnerInspection(Handle);
        public FormDefinition DefineForm(string alias, string name, string description, FormFieldList fields) =>
            owner.operations.LockboxDefineForm(Handle, alias, name, description, fields.ToByteArray());
        public FormDefinitionList ListFormDefinitions() => owner.operations.LockboxListFormDefinitions(Handle);
        public FormDefinition ResolveForm(string reference) => owner.operations.LockboxResolveForm(Handle, reference);
        public FormDefinitionList ListFormRevisions(string typeId) => owner.operations.LockboxListFormRevisions(Handle, typeId);
        public FormRecord CreateFormRecord(string path, string typeReference, string name) => owner.operations.LockboxCreateFormRecord(Handle, path, typeReference, name);
        public void SetFormField(string path, string field, string value, bool secret = false) => owner.operations.LockboxSetFormField(Handle, path, field, value, secret);
        public FormRecordList ListFormRecords() => owner.operations.LockboxListFormRecords(Handle);
        public FormRecord GetFormRecord(string path) => owner.operations.LockboxGetFormRecord(Handle, path);
        public void DeleteFormRecord(string path) => owner.operations.LockboxDeleteFormRecord(Handle, path);
        public void MoveFormRecords(PathMoveList moves) => owner.operations.LockboxMoveFormRecords(Handle, moves.ToByteArray());
        public FormValue GetFormField(string path, string field) => owner.operations.LockboxGetFormField(Handle, path, field);
        public byte[] Bytes => owner.operations.LockboxToBytes(Handle);
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.LockboxFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~Lockbox() => Dispose();
    }

    public VaultDirectory OpenVaultDirectory(string root, byte[] password) => new(this, operations.VaultDirectoryOpen(root, password));
    public VaultDirectory OpenOrCreateVaultDirectory(string root, byte[] password) => new(this, operations.VaultDirectoryOpenOrCreate(root, password));
    public VaultDirectory ReplaceVaultDirectory(string root, byte[] password) => new(this, operations.VaultDirectoryReplace(root, password));
    public VaultDirectory OpenOrCreateDefaultVaultDirectory(byte[] password) => new(this, operations.VaultDirectoryOpenOrCreateDefault(password));
    public VaultDirectory ReplaceDefaultVaultDirectory(byte[] password) => new(this, operations.VaultDirectoryReplaceDefault(password));
    public void ChangeVaultDirectoryPassword(string root, byte[] oldPassword, byte[] newPassword) => operations.VaultDirectoryChangePassword(root, oldPassword, newPassword);
    public void ChangeDefaultVaultDirectoryPassword(byte[] oldPassword, byte[] newPassword) => operations.VaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
    public string DefaultVaultDirectory => operations.VaultDefaultDirectory();
    public string DefaultVaultPath => operations.VaultDefaultPath();
    public VaultBackupManifest BackupDefaultVault(string path, bool overwrite = false) => operations.VaultBackupDefault(path, overwrite);
    public VaultBackupManifest RestoreDefaultVault(string path, bool overwrite = false) => operations.VaultRestoreDefault(path, overwrite);

    public sealed class VaultDirectory : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal VaultDirectory(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        public string Root => owner.operations.VaultDirectoryRoot(Handle);
        public uint StructureVersion => owner.operations.VaultDirectoryStructureVersion(Handle);
        public StringList ListPrivateKeys() => owner.operations.VaultDirectoryListPrivateKeys(Handle);
        public StringList ListPrivateKeyNames() => owner.operations.VaultDirectoryListPrivateKeyNames(Handle);
        public StringList ListContactNames() => owner.operations.VaultDirectoryListContactNames(Handle);
        public StringList ListFormAliases() => owner.operations.VaultDirectoryListFormAliases(Handle);
        public bool PrivateKeyExists(string name) => owner.operations.VaultDirectoryPrivateKeyExists(Handle, name);
        public void DeletePrivateKey(string name) => owner.operations.VaultDirectoryDeletePrivateKey(Handle, name);
        public void StorePrivateKey(string name, ContactKeyPair key) => owner.operations.VaultDirectoryStorePrivateKey(Handle, name, key.Handle);
        public ContactKeyPair LoadPrivateKey(string name) => new(owner, owner.operations.VaultDirectoryLoadPrivateKey(Handle, name));
        public ContactKeyPair LoadPrivateKeyGeneration(string name, ushort index) => new(owner, owner.operations.VaultDirectoryLoadPrivateKeyGeneration(Handle, name, index));
        public void StoreContact(string name, ContactPublicKey key) => owner.operations.VaultDirectoryStoreContact(Handle, name, key.Handle);
        public ContactPublicKey LoadContact(string name) => new(owner, owner.operations.VaultDirectoryLoadContact(Handle, name));
        public bool ContactExists(string name) => owner.operations.VaultDirectoryContactExists(Handle, name);
        public void DeleteContact(string name) => owner.operations.VaultDirectoryDeleteContact(Handle, name);
        public ContactList ListContacts() => owner.operations.VaultDirectoryListContacts(Handle);
        public void StoreProfileEmail(string name, string email) => owner.operations.VaultDirectoryStoreProfileEmail(Handle, name, email);
        public OptionalString ProfileEmail(string name) => owner.operations.VaultDirectoryProfileEmail(Handle, name);
        public void StoreBackup(byte[] id, byte[] value) => owner.operations.VaultDirectoryStoreBackup(Handle, id, value);
        public byte[] LoadBackup(byte[] id) => owner.operations.VaultDirectoryLoadBackup(Handle, id);
        public ulong BackupCount => owner.operations.VaultDirectoryBackupCount(Handle);
        public void RestorePrivateKey(string name, ContactKeyPair key, SigningKeyPair signing, bool overwrite) =>
            owner.operations.VaultDirectoryRestorePrivateKey(Handle, name, key.Handle, signing.Handle, overwrite);
        public SigningKeyPair LoadOwnerSigningKey(string name) => new(owner, owner.operations.VaultDirectoryLoadOwnerSigningKey(Handle, name));
        public SigningKeyPair LoadOwnerSigningKeyGeneration(string name, ushort index) =>
            new(owner, owner.operations.VaultDirectoryLoadOwnerSigningKeyGeneration(Handle, name, index));
        public void StoreContactSigningKey(string name, SigningPublicKey key) => owner.operations.VaultDirectoryStoreContactSigningKey(Handle, name, key.Handle);
        public SigningPublicKey LoadContactSigningKey(string name) => new(owner, owner.operations.VaultDirectoryLoadContactSigningKey(Handle, name));
        public ProfileHistory ListProfileGenerations(string name) => owner.operations.VaultDirectoryListProfileGenerations(Handle, name);
        public ProfileHistory RotatePrivateKey(string name) => owner.operations.VaultDirectoryRotatePrivateKey(Handle, name);
        public void RememberLockbox(byte[] id, string path) => owner.operations.VaultDirectoryRememberLockbox(Handle, id, path);
        public KnownLockboxList ListKnownLockboxes() => owner.operations.VaultDirectoryListKnownLockboxes(Handle);
        public void ForgetLockbox(string path) => owner.operations.VaultDirectoryForgetLockbox(Handle, path);
        public void RememberAccessSlotLabel(byte[] id, ulong slotId, string name) => owner.operations.VaultDirectoryRememberAccessSlotLabel(Handle, id, slotId, name);
        public AccessSlotLabelList ListAccessSlotLabels(byte[] id) => owner.operations.VaultDirectoryListAccessSlotLabels(Handle, id);
        public AccessSlotLabelList FindAccessSlotLabels(byte[] id, string name) => owner.operations.VaultDirectoryFindAccessSlotLabels(Handle, id, name);
        public void ForgetAccessSlotLabel(byte[] id, ulong slotId) => owner.operations.VaultDirectoryForgetAccessSlotLabel(Handle, id, slotId);
        public FormDefinition DefineForm(string alias, string name, string description, FormFieldList fields) =>
            owner.operations.VaultDirectoryDefineForm(Handle, alias, name, description, fields.ToByteArray());
        public FormDefinition ResolveForm(string reference) => owner.operations.VaultDirectoryResolveForm(Handle, reference);
        public FormDefinitionList ListForms() => owner.operations.VaultDirectoryListForms(Handle);
        public FormDefinitionList ListFormRevisions(string typeId) => owner.operations.VaultDirectoryListFormRevisions(Handle, typeId);
        public nuint SeedForms() => owner.operations.VaultDirectorySeedForms(Handle);
        public void RememberPassword(byte[] id, byte[] password) => owner.operations.VaultDirectoryRememberPassword(Handle, id, password);
        public byte[] RememberedPassword(byte[] id) => owner.operations.VaultDirectoryRememberedPassword(Handle, id);
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.VaultDirectoryFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~VaultDirectory() => Dispose();
    }

    public ReadOnlyVaultDirectory OpenReadOnlyVaultDirectory(string root, byte[] password) =>
        new(this, operations.VaultReadOnlyOpen(root, password));
    public ReadOnlyVaultDirectory OpenDefaultReadOnlyVaultDirectory(byte[] password) =>
        new(this, operations.VaultReadOnlyOpenDefault(password));
    public sealed class ReadOnlyVaultDirectory : IDisposable
    {
        private readonly Vault owner; private IntPtr handle;
        internal ReadOnlyVaultDirectory(Vault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        public StringList ListProfileNames() => owner.operations.VaultReadOnlyListProfileNames(handle);
        public StringList ListContactNames() => owner.operations.VaultReadOnlyListContactNames(handle);
        public StringList ListFormAliases() => owner.operations.VaultReadOnlyListFormAliases(handle);
        public KnownLockboxList ListKnownLockboxes() => owner.operations.VaultReadOnlyListKnownLockboxes(handle);
        public void Dispose() { if (handle != IntPtr.Zero) { owner.operations.VaultReadOnlyFree(handle); handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~ReadOnlyVaultDirectory() => Dispose();
    }

    public bool AgentIsRunning => operations.VaultIsRunning();
    public void ServeAgent() => operations.VaultAgentServe();
    public void VerifyAgentTransport() => operations.VaultAgentVerifyTransport();
    public void ForgetAllAgentSecrets() => operations.VaultForgetAll();
    public void StopAgent() => operations.VaultAgentStop();
    public void StartAgent() => operations.VaultAgentStart();
    public void PutAgentKey(byte[] id, byte[] key) => operations.VaultAgentPut(id, key);
    public byte[] GetAgentKey(byte[] id) => operations.VaultAgentGet(id);
    public void ForgetAgentKey(byte[] id) => operations.VaultAgentForget(id);
    public AgentEntryList ListAgentKeys() => operations.VaultAgentList();
    public SleepSupport AgentSleepSupport() => operations.VaultAgentSleepSupport();
    public string AgentLogPath => operations.VaultAgentLogPath();
    public string AgentLogDestination => operations.VaultAgentLogDestination();
    public void PutAgentVaultUnlockKey(string vaultId, byte[] key, ulong ttlSeconds) => operations.VaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
    public byte[] GetAgentVaultUnlockKey(string vaultId) => operations.VaultAgentGetVaultUnlockKey(vaultId);
    public void ForgetAgentVaultUnlockKey(string vaultId) => operations.VaultAgentForgetVaultUnlockKey(vaultId);
    public void PutAgentOwnerSigningKey(string vaultId, string profile, SigningKeyPair key, ulong ttlSeconds) =>
        operations.VaultAgentPutOwnerSigningKey(vaultId, profile, key.Handle, ttlSeconds);
    public SigningKeyPair GetAgentOwnerSigningKey(string vaultId, string profile) => new(this, operations.VaultAgentGetOwnerSigningKey(vaultId, profile));
    public void ForgetAgentOwnerSigningKey(string vaultId, string profile) => operations.VaultAgentForgetOwnerSigningKey(vaultId, profile);
    public AgentActivity BeginAgentActivity(string kind) => new(this, operations.VaultAgentBeginActivity(kind));

    public sealed class AgentActivity : IDisposable
    {
        private readonly Vault owner; private IntPtr handle;
        internal AgentActivity(Vault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        public void Dispose() { if (handle != IntPtr.Zero) { owner.operations.VaultAgentEndActivity(handle); handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~AgentActivity() => Dispose();
    }

    public PlatformStatus PlatformStatus() => operations.VaultPlatformStatus();
    public void SetPlatformScope(string scope) => operations.VaultPlatformSetScope(scope);
    public void EnablePlatformStore() => operations.VaultPlatformEnable();
    public void DisablePlatformStore() => operations.VaultPlatformDisable();
    public bool PlatformStoreDisabled => operations.VaultPlatformDisabled();
    public void PutPlatformPassword(byte[] password) => operations.VaultPlatformPutPassword(password);
    public byte[] GetPlatformPassword() => operations.VaultPlatformGetPassword();
    public void ForgetPlatformPassword() => operations.VaultPlatformForgetPassword();

    public LocalVault OpenLocalVault() => new(this, operations.VaultLocal());
    public sealed class LocalVault : IDisposable
    {
        private readonly Vault owner; private IntPtr handle;
        internal LocalVault(Vault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        public Lockbox CreateWithPassword(string path, byte[] password) => new(owner, owner.operations.VaultCreateLockboxPassword(handle, path, password));
        public Lockbox OpenWithPassword(string path, byte[] password) => new(owner, owner.operations.VaultOpenLockboxPassword(handle, path, password));
        public Lockbox CreateWithContentKey(string path, byte[] key, SigningKeyPair signing) =>
            new(owner, owner.operations.VaultCreateLockboxContentKey(handle, path, key, signing.Handle));
        public Lockbox OpenWithContentKey(string path, byte[] key, SigningKeyPair signing) =>
            new(owner, owner.operations.VaultOpenLockboxContentKey(handle, path, key, signing.Handle));
        public Lockbox CreateForContact(string path, ContactPublicKey contact, string name, SigningKeyPair signing) =>
            new(owner, owner.operations.VaultCreateLockboxContact(handle, path, contact.Handle, name, signing.Handle));
        public void CachePassword(string path, byte[] password, ulong ttlSeconds) => owner.operations.VaultCacheLockboxPassword(handle, path, password, ttlSeconds);
        public void CloseLockbox(string path) => owner.operations.VaultCloseLockbox(handle, path);
        public void CloseAll() => owner.operations.VaultCloseAll(handle);
        public void Dispose() { if (handle != IntPtr.Zero) { owner.operations.VaultFree(handle); handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        ~LocalVault() => Dispose();
    }
}
