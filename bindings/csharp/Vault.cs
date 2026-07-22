using System.Text;

namespace Revault;

/// <summary>Consumes temporary secret bytes before their transfer copy is wiped.</summary>
/// <typeparam name="T">The callback result type.</typeparam>
/// <param name="secret">A temporary read-only view that must not be retained.</param>
public delegate T SecretCallback<T>(ReadOnlySpan<byte> secret);

/// <summary>
/// Entry point for encrypted lockboxes, cryptographic keys, the local metadata
/// vault, the session agent, and the platform secret store.
/// </summary>
/// <remarks>
/// Create one when the application starts, then use it to open lockboxes and
/// manage keys and local services.
/// Values that retain sensitive state implement <see cref="IDisposable"/> and should be disposed
/// promptly. Secret variables and form fields are available only through
/// callback-scoped APIs so callers can avoid retaining plaintext. See the
/// <see href="https://github.com/onepub-dev/reVault#readme">repository README</see>
/// for installation, security guidance, and examples.
/// </remarks>
public sealed class Vault
{
    private readonly BindingOperations operations = new();
    private static void Open(IntPtr handle) { if (handle == IntPtr.Zero) throw new ObjectDisposedException("native object"); }

    /// <summary>Memory and CPU settings applied when creating or opening a lockbox.</summary>
    /// <param name="CacheMode">Cache strategy, such as <c>bytes</c>.</param>
    /// <param name="CacheBytes">Maximum cache capacity in bytes.</param>
    /// <param name="Workload">Workload profile, such as <c>interactive</c>.</param>
    /// <param name="Worker">Worker-selection policy, such as <c>auto</c>.</param>
    /// <param name="Jobs">Worker count; zero lets the library select it.</param>
    public sealed record LockboxOptions(string CacheMode, ulong CacheBytes, string Workload, string Worker, nuint Jobs)
    {
        /// <summary>Returns the recommended interactive runtime defaults.</summary>
        public static LockboxOptions Defaults => new("bytes", 64UL << 20, "interactive", "auto", 0);
    }

    /// <summary>Returns the last error.</summary>
    public string LastError => operations.LastErrorMessage();
    /// <summary>Returns the last error details.</summary>
    public ErrorDetails LastErrorDetails() => operations.BufferLastErrorDetails();
    /// <summary>Returns the lockbox format version.</summary>
    public ushort LockboxFormatVersion => (ushort)operations.LockboxFormatVersion();
    /// <summary>Determines lockbox format version without fully opening it.</summary>
    public ushort ProbeLockboxFormatVersion(byte[] value) => (ushort)operations.LockboxProbeFormatVersion(value);
    /// <summary>Returns the current vault structure version.</summary>
    public uint CurrentVaultStructureVersion => (uint)operations.VaultStructureVersionCurrent();
    /// <summary>Determines vault structure version without fully opening it.</summary>
    public uint ProbeVaultStructureVersion(string root, byte[] password) => (uint)operations.VaultDirectoryProbeStructureVersion(root, password);

    /// <summary>A recipient's shareable encryption identity used when granting lockbox access.</summary>
    public sealed class ContactPublicKey : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal ContactPublicKey(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Exports this key in the requested format.</summary>
        public byte[] Export(string format) { Open(Handle); return owner.operations.VaultKeyExportPublic(Handle, format); }
        /// <summary>Returns the stable fingerprint of this key.</summary>
        public byte[] Fingerprint() { Open(Handle); return owner.operations.VaultKeyFingerprint(Handle); }
        /// <summary>Encrypts a content key for the selected contact.</summary>
        public WrappedContactKey Encrypt(byte[] contentKey) => new(owner, owner.operations.KeyContactEncrypt(Handle, contentKey));
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeyContactPublicFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native public-key handle during finalization.</summary>
        ~ContactPublicKey() => Dispose();
    }

    /// <summary>A content key encrypted for one contact and recoverable only by its matching key pair.</summary>
    public sealed class WrappedContactKey : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal WrappedContactKey(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the public bytes.</summary>
        public byte[] PublicBytes() => owner.operations.KeyContactWrappedPublic(Handle);
        /// <summary>Returns the ciphertext.</summary>
        public byte[] Ciphertext() => owner.operations.KeyContactWrappedCiphertext(Handle);
        /// <summary>Returns the encrypted bytes.</summary>
        public byte[] EncryptedBytes() => owner.operations.KeyContactWrappedEncrypted(Handle);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeyContactWrappedFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native wrapped-key handle during finalization.</summary>
        ~WrappedContactKey() => Dispose();
    }

    /// <summary>A profile's contact-encryption identity used to decrypt content keys addressed to it.</summary>
    public sealed class ContactKeyPair : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal ContactKeyPair(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the public bytes.</summary>
        public byte[] PublicBytes() => owner.operations.KeyContactPublic(Handle);
        /// <summary>Returns the private record.</summary>
        public byte[] PrivateRecord() => owner.operations.KeyContactPrivate(Handle);
        /// <summary>Returns the public key.</summary>
        public ContactPublicKey PublicKey() => owner.ContactPublicKeyFromBytes(PublicBytes());
        /// <summary>Exports this key in the requested format.</summary>
        public byte[] Export(string format) => owner.operations.VaultKeyExportPrivate(Handle, format);
        /// <summary>Decrypts a wrapped content key for this contact.</summary>
        public byte[] Decrypt(WrappedContactKey wrapped) => owner.operations.KeyContactDecrypt(Handle, wrapped.Handle);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeyContactFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native contact-key handle during finalization.</summary>
        ~ContactKeyPair() => Dispose();
    }

    /// <summary>The public identity readers use to verify owner-authorized lockbox revisions.</summary>
    public sealed class SigningPublicKey : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal SigningPublicKey(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeySigningPublicFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native signing-public-key handle during finalization.</summary>
        ~SigningPublicKey() => Dispose();
    }

    /// <summary>A lockbox owner's signing identity used to authorize mutable revisions.</summary>
    public sealed class SigningKeyPair : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal SigningKeyPair(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the public bytes.</summary>
        public byte[] PublicBytes() => owner.operations.KeySigningPublic(Handle);
        /// <summary>Returns the private record.</summary>
        public byte[] PrivateRecord() => owner.operations.KeySigningPrivate(Handle);
        /// <summary>Returns the public key.</summary>
        public SigningPublicKey PublicKey() => owner.SigningPublicKeyFromBytes(PublicBytes());
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeySigningFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native signing-key handle during finalization.</summary>
        ~SigningKeyPair() => Dispose();
    }

    /// <summary>Generates contact key pair.</summary>
    public ContactKeyPair GenerateContactKeyPair() => new(this, operations.KeyContactGenerate());
    /// <summary>Returns the contact key pair from private.</summary>
    public ContactKeyPair ContactKeyPairFromPrivate(byte[] value) => new(this, operations.KeyContactFromPrivate(value));
    /// <summary>Imports contact key pair.</summary>
    public ContactKeyPair ImportContactKeyPair(byte[] value) => new(this, operations.VaultKeyImportPrivate(value));
    /// <summary>Returns the contact public key from bytes.</summary>
    public ContactPublicKey ContactPublicKeyFromBytes(byte[] value) => new(this, operations.KeyContactPublicFromBytes(value));
    /// <summary>Imports contact public key.</summary>
    public ContactPublicKey ImportContactPublicKey(byte[] value) => new(this, operations.VaultKeyImportPublic(value));
    /// <summary>Generates signing key pair.</summary>
    public SigningKeyPair GenerateSigningKeyPair() => new(this, operations.KeySigningGenerate());
    /// <summary>Returns the signing key pair from private.</summary>
    public SigningKeyPair SigningKeyPairFromPrivate(byte[] value) => new(this, operations.KeySigningFromPrivate(value));
    /// <summary>Returns the signing public key from bytes.</summary>
    public SigningPublicKey SigningPublicKeyFromBytes(byte[] value) => new(this, operations.KeySigningPublicFromBytes(value));

    /// <summary>Formats key hex.</summary>
    public string FormatKeyHex(byte[] value) => operations.VaultKeyFormatHex(value);
    /// <summary>Decodes key hex.</summary>
    public byte[] DecodeKeyHex(string value) => operations.VaultKeyDecodeHex(value);
    /// <summary>Formats key crockford.</summary>
    public string FormatKeyCrockford(byte[] value) => operations.VaultKeyFormatCrockford(value);
    /// <summary>Formats key crockford reading.</summary>
    public string FormatKeyCrockfordReading(string value) => operations.VaultKeyFormatCrockfordReading(value);
    /// <summary>Decodes key crockford.</summary>
    public byte[] DecodeKeyCrockford(string value) => operations.VaultKeyDecodeCrockford(value);
    /// <summary>Returns the hex encode.</summary>
    public string HexEncode(byte[] value) => operations.VaultKeyHexEncode(value);
    /// <summary>Returns the hex decode.</summary>
    public byte[] HexDecode(string value) => operations.VaultKeyHexDecode(value);

    /// <summary>Creates lockbox.</summary>
    public Lockbox CreateLockbox(byte[] key) => new(this, operations.LockboxCreate(key));
    /// <summary>Creates lockbox.</summary>
    public Lockbox CreateLockbox(byte[] key, LockboxOptions options) => new(this,
        operations.LockboxCreateWithOptions(key, options.CacheMode, options.CacheBytes, options.Workload, options.Worker, options.Jobs));
    /// <summary>Creates lockbox with password.</summary>
    public Lockbox CreateLockboxWithPassword(byte[] password) => new(this, operations.LockboxCreatePassword(password));
    /// <summary>Creates lockbox for contact.</summary>
    public Lockbox CreateLockboxForContact(ContactPublicKey contact) => new(this, operations.LockboxCreateContact(contact.Handle));
    /// <summary>Creates signed lockbox.</summary>
    public Lockbox CreateSignedLockbox(byte[] key, SigningKeyPair signing) => new(this, operations.LockboxCreateWithSigningKey(key, signing.Handle));
    /// <summary>Opens lockbox.</summary>
    public Lockbox OpenLockbox(byte[] archive, byte[] key) => new(this, operations.LockboxOpen(archive, key));
    /// <summary>Opens lockbox.</summary>
    public Lockbox OpenLockbox(byte[] archive, byte[] key, LockboxOptions options) => new(this,
        operations.LockboxOpenWithOptions(archive, key, options.CacheMode, options.CacheBytes, options.Workload, options.Worker, options.Jobs));
    /// <summary>Opens lockbox with password.</summary>
    public Lockbox OpenLockboxWithPassword(byte[] archive, byte[] password) => new(this, operations.LockboxOpenPassword(archive, password));
    /// <summary>Opens lockbox for contact.</summary>
    public Lockbox OpenLockboxForContact(byte[] archive, ContactKeyPair contact) => new(this, operations.LockboxOpenContact(archive, contact.Handle));
    /// <summary>Inspects lockbox file.</summary>
    public FileInspection InspectLockboxFile(string path) => operations.LockboxInspectFile(path);
    /// <summary>Scans lockbox path.</summary>
    public RecoveryReport ScanLockboxPath(string path, byte[] key) => operations.LockboxRecoveryScanPath(path, key);
    /// <summary>Scans lockbox.</summary>
    public RecoveryReport ScanLockbox(byte[] archive, byte[] key) => operations.LockboxRecoveryScan(archive, key);
    /// <summary>Salvages lockbox.</summary>
    public Lockbox SalvageLockbox(byte[] archive, byte[] key, SigningKeyPair? signing = null) =>
        new(this, operations.LockboxRecoverySalvage(archive, key, signing?.Handle ?? IntPtr.Zero));

    /// <summary>An open encrypted archive containing files, variables, secrets, and forms.</summary>
    public sealed class Lockbox : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal Lockbox(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Adds file.</summary>
        public void AddFile(string path, byte[] value, bool replace = false) => owner.operations.LockboxAddFile(Handle, path, value, replace);
        /// <summary>Adds file.</summary>
        public void AddFile(string path, byte[] value, uint permissions, bool replace = false) => owner.operations.LockboxAddFileWithPermissions(Handle, path, value, permissions, replace);
        /// <summary>Returns file.</summary>
        public byte[] GetFile(string path) => owner.operations.LockboxGetFile(Handle, path);
        /// <summary>Extracts file.</summary>
        public void ExtractFile(string source, string destination, bool replace = false) => owner.operations.LockboxExtractFile(Handle, source, destination, replace);
        /// <summary>Extracts directory.</summary>
        public void ExtractDirectory(string destination, ulong maxFileBytes, ulong maxTotalBytes, nuint maxFiles,
            bool restoreSymlinks, bool restorePermissions, bool overwrite) => owner.operations.LockboxExtractDirectory(
                Handle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite);
        /// <summary>Returns the stream content.</summary>
        public IReadOnlyList<StreamChunk> StreamContent(bool physical = false) => owner.operations.LockboxStreamContent(Handle, physical);
        /// <summary>Returns cache statistics for this lockbox.</summary>
        public CacheStats CacheStats() => owner.operations.LockboxCacheStats(Handle);
        /// <summary>Returns import statistics for this lockbox.</summary>
        public ImportStats ImportStats() => owner.operations.LockboxImportStats(Handle);
        /// <summary>Updates import stats.</summary>
        public void ResetImportStats() => owner.operations.LockboxResetImportStats(Handle);
        /// <summary>Returns the page inspection.</summary>
        public IReadOnlyList<PageInspection> PageInspection() => owner.operations.LockboxPageInspection(Handle);
        /// <summary>Returns the recovery report.</summary>
        public RecoveryReport RecoveryReport() => owner.operations.LockboxRecoveryReport(Handle);
        /// <summary>Returns the render recovery report.</summary>
        public string RenderRecoveryReport(bool verbose, nuint maxEntries) => owner.operations.LockboxRecoveryReportRender(Handle, verbose, maxEntries);
        /// <summary>Returns the storage length.</summary>
        public ulong StorageLength => owner.operations.LockboxStorageLen(Handle);
        /// <summary>Sets workload profile.</summary>
        public void SetWorkloadProfile(string profile) => owner.operations.LockboxSetWorkloadProfile(Handle, profile);
        /// <summary>Sets worker policy.</summary>
        public void SetWorkerPolicy(string mode, nuint jobs) => owner.operations.LockboxSetWorkerPolicy(Handle, mode, jobs);
        /// <summary>Returns the runtime options.</summary>
        public RuntimeOptions RuntimeOptions() => owner.operations.LockboxRuntimeOptions(Handle);
        /// <summary>Authenticates and publishes the staged changes.</summary>
        public void Commit() => owner.operations.LockboxCommit(Handle);
        /// <summary>Creates directory.</summary>
        public void CreateDirectory(string path, bool parents = false) => owner.operations.LockboxCreateDir(Handle, path, parents);
        /// <summary>Removes delete.</summary>
        public void Delete(string path) => owner.operations.LockboxDelete(Handle, path);
        /// <summary>Removes directory.</summary>
        public void RemoveDirectory(string path, bool recursive = false) => owner.operations.LockboxRemoveDir(Handle, path, recursive);
        /// <summary>Creates parent directories.</summary>
        public void CreateParentDirectories(string path) => owner.operations.LockboxCreateParentDirs(Handle, path);
        /// <summary>Updates rename.</summary>
        public void Rename(string from, string to) => owner.operations.LockboxRename(Handle, from, to);
        /// <summary>Lists list.</summary>
        public IReadOnlyList<LockboxEntry> List(string path = "/", bool recursive = false) => owner.operations.LockboxList(Handle, path, recursive);
        /// <summary>Lists list.</summary>
        public IReadOnlyList<LockboxEntry> List(string path, string glob, bool recursive, bool includeFiles,
            bool includeSymlinks, bool includeDirectories, nuint limit) => owner.operations.LockboxListWithOptions(
                Handle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit);
        /// <summary>Returns metadata for the selected lockbox entry.</summary>
        public LockboxEntry? Stat(string path) => owner.operations.LockboxStat(Handle, path);
        /// <summary>Sets variable.</summary>
        public void SetVariable(string name, string value) => owner.operations.LockboxSetVariable(Handle, name, value);
        /// <summary>Stores a secret variable from mutable bytes.</summary>
        public void SetSecretVariable(string name, byte[] value) => owner.operations.LockboxSetSecretVariable(Handle, name, value);
        /// <summary>Returns variable.</summary>
        public string? GetVariable(string name) => owner.operations.LockboxGetVariable(Handle, name);
        /// <summary>Invokes <paramref name="callback"/> with temporary secret bytes, then wipes the transfer buffer.</summary>
        public T? WithSecretVariable<T>(string name, SecretCallback<T> callback) => owner.operations.LockboxWithSecretVariable(Handle, name, callback);
        /// <summary>Removes variable.</summary>
        public void DeleteVariable(string name) => owner.operations.LockboxDeleteVariable(Handle, name);
        /// <summary>Updates variables.</summary>
        public void MoveVariables(IReadOnlyList<PathMove> moves) => owner.operations.LockboxMoveVariables(Handle, DomainCodec.EncodePathMoves(moves));
        /// <summary>Lists variables.</summary>
        public IReadOnlyList<Variable> ListVariables() => owner.operations.LockboxListVariables(Handle);
        /// <summary>Returns the variable sensitivity.</summary>
        public string? VariableSensitivity(string name) => owner.operations.LockboxVariableSensitivity(Handle, name);
        /// <summary>Adds symlink.</summary>
        public void AddSymlink(string path, string target, bool replace = false) => owner.operations.LockboxAddSymlink(Handle, path, target, replace);
        /// <summary>Returns the symlink target.</summary>
        public string SymlinkTarget(string path) => owner.operations.LockboxGetSymlinkTarget(Handle, path);
        /// <summary>Returns the id.</summary>
        public byte[] Id => owner.operations.LockboxId(Handle);
        /// <summary>Reports whether exists.</summary>
        public bool Exists(string path) => owner.operations.LockboxExists(Handle, path);
        /// <summary>Reports whether directory.</summary>
        public bool IsDirectory(string path) => owner.operations.LockboxIsDir(Handle, path);
        /// <summary>Returns the permissions.</summary>
        public uint Permissions(string path) => owner.operations.LockboxPermissions(Handle, path);
        /// <summary>Sets permissions.</summary>
        public void SetPermissions(string path, uint value) => owner.operations.LockboxSetPermissions(Handle, path, value);
        /// <summary>Returns range.</summary>
        public byte[] ReadRange(string path, ulong offset, ulong length) => owner.operations.LockboxReadRange(Handle, path, offset, length);
        /// <summary>Adds password.</summary>
        public ulong AddPassword(byte[] password) { var id = owner.operations.LockboxAddPassword(Handle, password); if (id == ulong.MaxValue) throw new InvalidOperationException(owner.LastError); return id; }
        /// <summary>Adds contact.</summary>
        public ulong AddContact(ContactPublicKey contact, string name) { var id = owner.operations.LockboxAddContact(Handle, contact.Handle, name); if (id == ulong.MaxValue) throw new InvalidOperationException(owner.LastError); return id; }
        /// <summary>Removes key.</summary>
        public void DeleteKey(ulong id) => owner.operations.LockboxDeleteKey(Handle, id);
        /// <summary>Lists key slots.</summary>
        public IReadOnlyList<KeySlot> ListKeySlots() => owner.operations.LockboxListKeySlots(Handle);
        /// <summary>Sets owner signing key.</summary>
        public void SetOwnerSigningKey(SigningKeyPair key) => owner.operations.LockboxSetOwnerSigningKey(Handle, key.Handle);
        /// <summary>Returns the owner inspection.</summary>
        public OwnerInspection OwnerInspection() => owner.operations.LockboxOwnerInspection(Handle);
        /// <summary>Returns the define form.</summary>
        public FormDefinition DefineForm(string alias, string name, string description, IReadOnlyList<FormField> fields) =>
            owner.operations.LockboxDefineForm(Handle, alias, name, description, DomainCodec.EncodeFormFields(fields));
        /// <summary>Lists form definitions.</summary>
        public IReadOnlyList<FormDefinition> ListFormDefinitions() => owner.operations.LockboxListFormDefinitions(Handle);
        /// <summary>Returns the resolve form.</summary>
        public FormDefinition ResolveForm(string reference) => owner.operations.LockboxResolveForm(Handle, reference);
        /// <summary>Lists form revisions.</summary>
        public IReadOnlyList<FormDefinition> ListFormRevisions(string typeId) => owner.operations.LockboxListFormRevisions(Handle, typeId);
        /// <summary>Creates form record.</summary>
        public FormRecord CreateFormRecord(string path, string typeReference, string name) => owner.operations.LockboxCreateFormRecord(Handle, path, typeReference, name);
        /// <summary>Sets form field.</summary>
        public void SetFormField(string path, string field, string value) => owner.operations.LockboxSetFormField(Handle, path, field, value);
        /// <summary>Stores a secret form field from mutable bytes.</summary>
        public void SetSecretFormField(string path, string field, byte[] value) => owner.operations.LockboxSetSecretFormField(Handle, path, field, value);
        /// <summary>Lists form records.</summary>
        public IReadOnlyList<FormRecord> ListFormRecords() => owner.operations.LockboxListFormRecords(Handle);
        /// <summary>Returns form record.</summary>
        public FormRecord? GetFormRecord(string path) => owner.operations.LockboxGetFormRecord(Handle, path);
        /// <summary>Removes form record.</summary>
        public void DeleteFormRecord(string path) => owner.operations.LockboxDeleteFormRecord(Handle, path);
        /// <summary>Updates form records.</summary>
        public void MoveFormRecords(IReadOnlyList<PathMove> moves) => owner.operations.LockboxMoveFormRecords(Handle, DomainCodec.EncodePathMoves(moves));
        /// <summary>Returns form field.</summary>
        public FormValue? GetFormField(string path, string field) => owner.operations.LockboxGetFormField(Handle, path, field);
        /// <summary>Invokes <paramref name="callback"/> with temporary field bytes, then wipes the transfer buffer.</summary>
        public T? WithSecretFormField<T>(string path, string field, SecretCallback<T> callback) => owner.operations.LockboxWithSecretFormField(Handle, path, field, callback);
        /// <summary>Returns the bytes.</summary>
        public byte[] Bytes => owner.operations.LockboxToBytes(Handle);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.LockboxFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native lockbox handle during finalization.</summary>
        ~Lockbox() => Dispose();
    }

    /// <summary>Opens vault directory.</summary>
    public VaultDirectory OpenVaultDirectory(string root, byte[] password) => new(this, operations.VaultDirectoryOpen(root, password));
    /// <summary>Opens or create vault directory.</summary>
    public VaultDirectory OpenOrCreateVaultDirectory(string root, byte[] password) => new(this, operations.VaultDirectoryOpenOrCreate(root, password));
    /// <summary>Updates vault directory.</summary>
    public VaultDirectory ReplaceVaultDirectory(string root, byte[] password) => new(this, operations.VaultDirectoryReplace(root, password));
    /// <summary>Opens or create default vault directory.</summary>
    public VaultDirectory OpenOrCreateDefaultVaultDirectory(byte[] password) => new(this, operations.VaultDirectoryOpenOrCreateDefault(password));
    /// <summary>Updates default vault directory.</summary>
    public VaultDirectory ReplaceDefaultVaultDirectory(byte[] password) => new(this, operations.VaultDirectoryReplaceDefault(password));
    /// <summary>Updates vault directory password.</summary>
    public void ChangeVaultDirectoryPassword(string root, byte[] oldPassword, byte[] newPassword) => operations.VaultDirectoryChangePassword(root, oldPassword, newPassword);
    /// <summary>Updates default vault directory password.</summary>
    public void ChangeDefaultVaultDirectoryPassword(byte[] oldPassword, byte[] newPassword) => operations.VaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
    /// <summary>Returns the default vault directory.</summary>
    public string DefaultVaultDirectory => operations.VaultDefaultDirectory();
    /// <summary>Returns the default vault path.</summary>
    public string DefaultVaultPath => operations.VaultDefaultPath();
    /// <summary>Returns the backup default vault.</summary>
    public VaultBackupManifest BackupDefaultVault(string path, bool overwrite = false) => operations.VaultBackupDefault(path, overwrite);
    /// <summary>Returns the restore default vault.</summary>
    public VaultBackupManifest RestoreDefaultVault(string path, bool overwrite = false) => operations.VaultRestoreDefault(path, overwrite);

    /// <summary>Password-protected storage for profile keys, contacts, forms, backups, and known lockbox paths.</summary>
    public sealed class VaultDirectory : IDisposable
    {
        private readonly Vault owner; internal IntPtr Handle;
        internal VaultDirectory(Vault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the root.</summary>
        public string Root => owner.operations.VaultDirectoryRoot(Handle);
        /// <summary>Returns the structure version.</summary>
        public uint StructureVersion => owner.operations.VaultDirectoryStructureVersion(Handle);
        /// <summary>Lists private keys.</summary>
        public IReadOnlyList<string> ListPrivateKeys() => owner.operations.VaultDirectoryListPrivateKeys(Handle);
        /// <summary>Lists private key names.</summary>
        public IReadOnlyList<string> ListPrivateKeyNames() => owner.operations.VaultDirectoryListPrivateKeyNames(Handle);
        /// <summary>Lists contact names.</summary>
        public IReadOnlyList<string> ListContactNames() => owner.operations.VaultDirectoryListContactNames(Handle);
        /// <summary>Lists form aliases.</summary>
        public IReadOnlyList<string> ListFormAliases() => owner.operations.VaultDirectoryListFormAliases(Handle);
        /// <summary>Returns the private key exists.</summary>
        public bool PrivateKeyExists(string name) => owner.operations.VaultDirectoryPrivateKeyExists(Handle, name);
        /// <summary>Removes private key.</summary>
        public void DeletePrivateKey(string name) => owner.operations.VaultDirectoryDeletePrivateKey(Handle, name);
        /// <summary>Stores private key.</summary>
        public void StorePrivateKey(string name, ContactKeyPair key) => owner.operations.VaultDirectoryStorePrivateKey(Handle, name, key.Handle);
        /// <summary>Loads private key.</summary>
        public ContactKeyPair LoadPrivateKey(string name) => new(owner, owner.operations.VaultDirectoryLoadPrivateKey(Handle, name));
        /// <summary>Loads private key generation.</summary>
        public ContactKeyPair LoadPrivateKeyGeneration(string name, ushort index) => new(owner, owner.operations.VaultDirectoryLoadPrivateKeyGeneration(Handle, name, index));
        /// <summary>Stores contact.</summary>
        public void StoreContact(string name, ContactPublicKey key) => owner.operations.VaultDirectoryStoreContact(Handle, name, key.Handle);
        /// <summary>Loads contact.</summary>
        public ContactPublicKey LoadContact(string name) => new(owner, owner.operations.VaultDirectoryLoadContact(Handle, name));
        /// <summary>Returns the contact exists.</summary>
        public bool ContactExists(string name) => owner.operations.VaultDirectoryContactExists(Handle, name);
        /// <summary>Removes contact.</summary>
        public void DeleteContact(string name) => owner.operations.VaultDirectoryDeleteContact(Handle, name);
        /// <summary>Lists contacts.</summary>
        public IReadOnlyList<Contact> ListContacts() => owner.operations.VaultDirectoryListContacts(Handle);
        /// <summary>Stores profile email.</summary>
        public void StoreProfileEmail(string name, string email) => owner.operations.VaultDirectoryStoreProfileEmail(Handle, name, email);
        /// <summary>Returns the profile email.</summary>
        public string? ProfileEmail(string name) => owner.operations.VaultDirectoryProfileEmail(Handle, name);
        /// <summary>Stores backup.</summary>
        public void StoreBackup(byte[] id, byte[] value) => owner.operations.VaultDirectoryStoreBackup(Handle, id, value);
        /// <summary>Loads backup.</summary>
        public byte[] LoadBackup(byte[] id) => owner.operations.VaultDirectoryLoadBackup(Handle, id);
        /// <summary>Returns the backup count.</summary>
        public ulong BackupCount => owner.operations.VaultDirectoryBackupCount(Handle);
        /// <summary>Returns the restore private key.</summary>
        public void RestorePrivateKey(string name, ContactKeyPair key, SigningKeyPair signing, bool overwrite) =>
            owner.operations.VaultDirectoryRestorePrivateKey(Handle, name, key.Handle, signing.Handle, overwrite);
        /// <summary>Loads owner signing key.</summary>
        public SigningKeyPair LoadOwnerSigningKey(string name) => new(owner, owner.operations.VaultDirectoryLoadOwnerSigningKey(Handle, name));
        /// <summary>Loads owner signing key generation.</summary>
        public SigningKeyPair LoadOwnerSigningKeyGeneration(string name, ushort index) =>
            new(owner, owner.operations.VaultDirectoryLoadOwnerSigningKeyGeneration(Handle, name, index));
        /// <summary>Stores contact signing key.</summary>
        public void StoreContactSigningKey(string name, SigningPublicKey key) => owner.operations.VaultDirectoryStoreContactSigningKey(Handle, name, key.Handle);
        /// <summary>Loads contact signing key.</summary>
        public SigningPublicKey LoadContactSigningKey(string name) => new(owner, owner.operations.VaultDirectoryLoadContactSigningKey(Handle, name));
        /// <summary>Lists profile generations.</summary>
        public ProfileHistory ListProfileGenerations(string name) => owner.operations.VaultDirectoryListProfileGenerations(Handle, name);
        /// <summary>Updates private key.</summary>
        public ProfileHistory RotatePrivateKey(string name) => owner.operations.VaultDirectoryRotatePrivateKey(Handle, name);
        /// <summary>Stores lockbox.</summary>
        public void RememberLockbox(byte[] id, string path) => owner.operations.VaultDirectoryRememberLockbox(Handle, id, path);
        /// <summary>Lists known lockboxes.</summary>
        public IReadOnlyList<KnownLockbox> ListKnownLockboxes() => owner.operations.VaultDirectoryListKnownLockboxes(Handle);
        /// <summary>Removes lockbox.</summary>
        public void ForgetLockbox(string path) => owner.operations.VaultDirectoryForgetLockbox(Handle, path);
        /// <summary>Stores access slot label.</summary>
        public void RememberAccessSlotLabel(byte[] id, ulong slotId, string name) => owner.operations.VaultDirectoryRememberAccessSlotLabel(Handle, id, slotId, name);
        /// <summary>Lists access slot labels.</summary>
        public IReadOnlyList<AccessSlotLabel> ListAccessSlotLabels(byte[] id) => owner.operations.VaultDirectoryListAccessSlotLabels(Handle, id);
        /// <summary>Returns the find access slot labels.</summary>
        public IReadOnlyList<AccessSlotLabel> FindAccessSlotLabels(byte[] id, string name) => owner.operations.VaultDirectoryFindAccessSlotLabels(Handle, id, name);
        /// <summary>Removes access slot label.</summary>
        public void ForgetAccessSlotLabel(byte[] id, ulong slotId) => owner.operations.VaultDirectoryForgetAccessSlotLabel(Handle, id, slotId);
        /// <summary>Returns the define form.</summary>
        public FormDefinition DefineForm(string alias, string name, string description, IReadOnlyList<FormField> fields) =>
            owner.operations.VaultDirectoryDefineForm(Handle, alias, name, description, DomainCodec.EncodeFormFields(fields));
        /// <summary>Returns the resolve form.</summary>
        public FormDefinition ResolveForm(string reference) => owner.operations.VaultDirectoryResolveForm(Handle, reference);
        /// <summary>Lists forms.</summary>
        public IReadOnlyList<FormDefinition> ListForms() => owner.operations.VaultDirectoryListForms(Handle);
        /// <summary>Lists form revisions.</summary>
        public IReadOnlyList<FormDefinition> ListFormRevisions(string typeId) => owner.operations.VaultDirectoryListFormRevisions(Handle, typeId);
        /// <summary>Returns the seed forms.</summary>
        public nuint SeedForms() => owner.operations.VaultDirectorySeedForms(Handle);
        /// <summary>Stores password.</summary>
        public void RememberPassword(byte[] id, byte[] password) => owner.operations.VaultDirectoryRememberPassword(Handle, id, password);
        /// <summary>Returns the remembered password.</summary>
        public byte[] RememberedPassword(byte[] id) => owner.operations.VaultDirectoryRememberedPassword(Handle, id);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.VaultDirectoryFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the writable vault handle during finalization.</summary>
        ~VaultDirectory() => Dispose();
    }

    /// <summary>Opens read only vault directory.</summary>
    public ReadOnlyVaultDirectory OpenReadOnlyVaultDirectory(string root, byte[] password) =>
        new(this, operations.VaultReadOnlyOpen(root, password));
    /// <summary>Opens default read only vault directory.</summary>
    public ReadOnlyVaultDirectory OpenDefaultReadOnlyVaultDirectory(byte[] password) =>
        new(this, operations.VaultReadOnlyOpenDefault(password));
    /// <summary>A metadata view for discovery and diagnostics that never loads an owner signing key.</summary>
    public sealed class ReadOnlyVaultDirectory : IDisposable
    {
        private readonly Vault owner; private IntPtr handle;
        internal ReadOnlyVaultDirectory(Vault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        /// <summary>Lists profile names.</summary>
        public IReadOnlyList<string> ListProfileNames() => owner.operations.VaultReadOnlyListProfileNames(handle);
        /// <summary>Lists contact names.</summary>
        public IReadOnlyList<string> ListContactNames() => owner.operations.VaultReadOnlyListContactNames(handle);
        /// <summary>Lists form aliases.</summary>
        public IReadOnlyList<string> ListFormAliases() => owner.operations.VaultReadOnlyListFormAliases(handle);
        /// <summary>Lists known lockboxes.</summary>
        public IReadOnlyList<KnownLockbox> ListKnownLockboxes() => owner.operations.VaultReadOnlyListKnownLockboxes(handle);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (handle != IntPtr.Zero) { owner.operations.VaultReadOnlyFree(handle); handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the read-only vault handle during finalization.</summary>
        ~ReadOnlyVaultDirectory() => Dispose();
    }

    /// <summary>Returns the agent is running.</summary>
    public bool AgentIsRunning => operations.VaultIsRunning();
    /// <summary>Returns the serve agent.</summary>
    public void ServeAgent() => operations.VaultAgentServe();
    /// <summary>Verifies agent transport.</summary>
    public void VerifyAgentTransport() => operations.VaultAgentVerifyTransport();
    /// <summary>Removes all agent secrets.</summary>
    public void ForgetAllAgentSecrets() => operations.VaultForgetAll();
    /// <summary>Stops agent.</summary>
    public void StopAgent() => operations.VaultAgentStop();
    /// <summary>Starts agent.</summary>
    public void StartAgent() => operations.VaultAgentStart();
    /// <summary>Stores agent key.</summary>
    public void PutAgentKey(byte[] id, byte[] key) => operations.VaultAgentPut(id, key);
    /// <summary>Returns agent key.</summary>
    public byte[] GetAgentKey(byte[] id) => operations.VaultAgentGet(id);
    /// <summary>Removes agent key.</summary>
    public void ForgetAgentKey(byte[] id) => operations.VaultAgentForget(id);
    /// <summary>Lists agent keys.</summary>
    public IReadOnlyList<AgentEntry> ListAgentKeys() => operations.VaultAgentList();
    /// <summary>Returns the agent sleep support.</summary>
    public SleepSupport AgentSleepSupport() => operations.VaultAgentSleepSupport();
    /// <summary>Returns the agent log path.</summary>
    public string AgentLogPath => operations.VaultAgentLogPath();
    /// <summary>Returns the agent log destination.</summary>
    public string AgentLogDestination => operations.VaultAgentLogDestination();
    /// <summary>Stores agent vault unlock key.</summary>
    public void PutAgentVaultUnlockKey(string vaultId, byte[] key, ulong ttlSeconds) => operations.VaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds);
    /// <summary>Returns agent vault unlock key.</summary>
    public byte[] GetAgentVaultUnlockKey(string vaultId) => operations.VaultAgentGetVaultUnlockKey(vaultId);
    /// <summary>Removes agent vault unlock key.</summary>
    public void ForgetAgentVaultUnlockKey(string vaultId) => operations.VaultAgentForgetVaultUnlockKey(vaultId);
    /// <summary>Stores agent owner signing key.</summary>
    public void PutAgentOwnerSigningKey(string vaultId, string profile, SigningKeyPair key, ulong ttlSeconds) =>
        operations.VaultAgentPutOwnerSigningKey(vaultId, profile, key.Handle, ttlSeconds);
    /// <summary>Returns agent owner signing key.</summary>
    public SigningKeyPair GetAgentOwnerSigningKey(string vaultId, string profile) => new(this, operations.VaultAgentGetOwnerSigningKey(vaultId, profile));
    /// <summary>Removes agent owner signing key.</summary>
    public void ForgetAgentOwnerSigningKey(string vaultId, string profile) => operations.VaultAgentForgetOwnerSigningKey(vaultId, profile);
    /// <summary>Starts agent activity.</summary>
    public AgentActivity BeginAgentActivity(string kind) => new(this, operations.VaultAgentBeginActivity(kind));

    /// <summary>A token kept alive while an operation needs secrets cached by the session agent.</summary>
    public sealed class AgentActivity : IDisposable
    {
        private readonly Vault owner; private IntPtr handle;
        internal AgentActivity(Vault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (handle != IntPtr.Zero) { owner.operations.VaultAgentEndActivity(handle); handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Ends and releases the agent activity during finalization.</summary>
        ~AgentActivity() => Dispose();
    }

    /// <summary>Returns the platform status.</summary>
    public PlatformStatus PlatformStatus() => operations.VaultPlatformStatus();
    /// <summary>Sets platform scope.</summary>
    public void SetPlatformScope(string scope) => operations.VaultPlatformSetScope(scope);
    /// <summary>Returns the enable platform store.</summary>
    public void EnablePlatformStore() => operations.VaultPlatformEnable();
    /// <summary>Returns the disable platform store.</summary>
    public void DisablePlatformStore() => operations.VaultPlatformDisable();
    /// <summary>Returns the platform store disabled.</summary>
    public bool PlatformStoreDisabled => operations.VaultPlatformDisabled();
    /// <summary>Stores platform password.</summary>
    public void PutPlatformPassword(byte[] password) => operations.VaultPlatformPutPassword(password);
    /// <summary>Returns platform password.</summary>
    public byte[] GetPlatformPassword() => operations.VaultPlatformGetPassword();
    /// <summary>Removes platform password.</summary>
    public void ForgetPlatformPassword() => operations.VaultPlatformForgetPassword();

    /// <summary>Opens local vault.</summary>
    public LocalVault OpenLocalVault() => new(this, operations.VaultLocal());
    /// <summary>A session that opens lockboxes by host path, caches passwords, and closes locally used files.</summary>
    public sealed class LocalVault : IDisposable
    {
        private readonly Vault owner; private IntPtr handle;
        internal LocalVault(Vault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        /// <summary>Creates with password.</summary>
        public Lockbox CreateWithPassword(string path, byte[] password) => new(owner, owner.operations.VaultCreateLockboxPassword(handle, path, password));
        /// <summary>Opens with password.</summary>
        public Lockbox OpenWithPassword(string path, byte[] password) => new(owner, owner.operations.VaultOpenLockboxPassword(handle, path, password));
        /// <summary>Creates with content key.</summary>
        public Lockbox CreateWithContentKey(string path, byte[] key, SigningKeyPair signing) =>
            new(owner, owner.operations.VaultCreateLockboxContentKey(handle, path, key, signing.Handle));
        /// <summary>Opens with content key.</summary>
        public Lockbox OpenWithContentKey(string path, byte[] key, SigningKeyPair signing) =>
            new(owner, owner.operations.VaultOpenLockboxContentKey(handle, path, key, signing.Handle));
        /// <summary>Creates for contact.</summary>
        public Lockbox CreateForContact(string path, ContactPublicKey contact, string name, SigningKeyPair signing) =>
            new(owner, owner.operations.VaultCreateLockboxContact(handle, path, contact.Handle, name, signing.Handle));
        /// <summary>Stores password.</summary>
        public void CachePassword(string path, byte[] password, ulong ttlSeconds) => owner.operations.VaultCacheLockboxPassword(handle, path, password, ttlSeconds);
        /// <summary>Releases the native resources held by lockbox.</summary>
        public void CloseLockbox(string path) => owner.operations.VaultCloseLockbox(handle, path);
        /// <summary>Releases the native resources held by all.</summary>
        public void CloseAll() => owner.operations.VaultCloseAll(handle);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (handle != IntPtr.Zero) { owner.operations.VaultFree(handle); handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the local-vault handle during finalization.</summary>
        ~LocalVault() => Dispose();
    }
}
