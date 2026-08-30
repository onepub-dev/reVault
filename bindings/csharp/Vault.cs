using System.Text;

namespace Revault;

/// <summary>Consumes temporary secret bytes before their transfer copy is wiped.</summary>
/// <typeparam name="T">The callback result type.</typeparam>
/// <param name="secret">A temporary read-only view that must not be retained.</param>
public delegate T SecretCallback<T>(ReadOnlySpan<byte> secret);

/// <summary>
/// Entry point for encrypted lockboxes, cryptographic keys, the local metadata
/// Vault, Session Agent, and the platform credential store.
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
public sealed class Revault
{
    private readonly BindingOperations operations;
    /// <summary>Loads the native carrier without opening a Vault or Lockbox.</summary>
    /// <param name="nativeLibraryPath">Explicit path/name, or null for inherited/package discovery.</param>
    public static Revault Load(string? nativeLibraryPath = null) => new(nativeLibraryPath);
    /// <summary>Creates a runtime using explicit, inherited, then packaged discovery.</summary>
    /// <param name="nativeLibraryPath">Explicit path/name, or null for inherited/package discovery.</param>
    public Revault(string? nativeLibraryPath = null)
    {
        NativeLibraryResolver.Configure(nativeLibraryPath);
        operations = new BindingOperations();
    }
    /// Explicit controls for the optional Session Agent.
    public AgentSession AgentSession => new(this);
    private static void Open(IntPtr handle) { if (handle == IntPtr.Zero) throw new ObjectDisposedException("native object"); }

    /// <summary>Memory and CPU settings applied when creating or opening a lockbox.</summary>
    /// <param name="CacheMode">Cache strategy, such as <c>bytes</c>.</param>
    /// <param name="CacheBytes">Maximum cache capacity in bytes.</param>
    /// <param name="Workload">Workload profile, such as <c>interactive</c>.</param>
    /// <param name="Worker">Worker-selection policy, such as <c>auto</c>.</param>
    /// <param name="Jobs">Worker count; zero lets the library select it.</param>
    public sealed record LockboxOptions(string CacheMode, ulong CacheBytes, string Workload, string Worker, nuint Jobs)
    {
        /// <summary>Creates options from the closed policy enums.</summary>
        public LockboxOptions(CacheMode cacheMode, ulong cacheBytes, WorkloadProfile workload,
            WorkerPolicy worker, nuint jobs = 0)
            : this(cacheMode switch { global::Revault.CacheMode.Bytes => "bytes", global::Revault.CacheMode.Pages => "pages", _ => throw new ArgumentOutOfRangeException(nameof(cacheMode)) },
                cacheBytes,
                workload switch { WorkloadProfile.Interactive => "interactive", WorkloadProfile.BulkImport => "bulk-import", _ => throw new ArgumentOutOfRangeException(nameof(workload)) },
                worker switch { WorkerPolicy.Auto => "auto", WorkerPolicy.Single => "single", _ => throw new ArgumentOutOfRangeException(nameof(worker)) }, jobs)
        { }

        /// <summary>Returns the recommended interactive runtime defaults.</summary>
        public static LockboxOptions Defaults => new("bytes", 64UL << 20, "interactive", "auto", 0);
    }

    /// <summary>Returns the last error.</summary>
    public string LastError => operations.LastErrorMessage();
    /// <summary>Returns the last error details.</summary>
    public ErrorDetails LastErrorDetails() => operations.BufferLastErrorDetails();
    /// <summary>Returns the newest Lockbox archive format version supported by this engine.</summary>
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
        private readonly Revault owner; internal IntPtr Handle;
        internal ContactPublicKey(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
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
        private readonly Revault owner; internal IntPtr Handle;
        internal WrappedContactKey(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the public bytes.</summary>
        public byte[] PublicBytes() => owner.operations.KeyContactWrappedPublic(Handle);
        /// <summary>Returns the encrypted content key bytes.</summary>
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
        private readonly Revault owner; internal IntPtr Handle;
        internal ContactKeyPair(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
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

    /// <summary>The public profile identity readers use to verify authorized lockbox revisions.</summary>
    public sealed class ProfileSigningPublicKey : IDisposable
    {
        private readonly Revault owner; internal IntPtr Handle;
        internal ProfileSigningPublicKey(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeySigningPublicFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native signing-public-key handle during finalization.</summary>
        ~ProfileSigningPublicKey() => Dispose();
    }

    /// <summary>A profile signing identity used to authorize mutable lockbox revisions.</summary>
    public sealed class ProfileSigningKeyPair : IDisposable
    {
        private readonly Revault owner; internal IntPtr Handle;
        internal ProfileSigningKeyPair(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the public bytes.</summary>
        public byte[] PublicBytes() => owner.operations.KeySigningPublic(Handle);
        /// <summary>Returns the private record.</summary>
        public byte[] PrivateRecord() => owner.operations.KeySigningPrivate(Handle);
        /// <summary>Returns the public key.</summary>
        public ProfileSigningPublicKey PublicKey() => owner.ProfileSigningPublicKeyFromBytes(PublicBytes());
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.KeySigningFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the native signing-key handle during finalization.</summary>
        ~ProfileSigningKeyPair() => Dispose();
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
    /// <summary>Generates a profile signing key pair.</summary>
    public ProfileSigningKeyPair GenerateProfileSigningKeyPair() => new(this, operations.KeySigningGenerate());
    /// <summary>Imports a profile signing key pair from its private record.</summary>
    public ProfileSigningKeyPair ProfileSigningKeyPairFromPrivate(byte[] value) => new(this, operations.KeySigningFromPrivate(value));
    /// <summary>Imports a profile signing public key from encoded bytes.</summary>
    public ProfileSigningPublicKey ProfileSigningPublicKeyFromBytes(byte[] value) => new(this, operations.KeySigningPublicFromBytes(value));

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
    public Lockbox CreateLockboxWithProfileSigningKey(byte[] key, ProfileSigningKeyPair signing) => new(this, operations.LockboxCreateWithSigningKey(key, signing.Handle));
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
    public Lockbox SalvageLockbox(byte[] archive, byte[] key, ProfileSigningKeyPair? signing = null) =>
        new(this, operations.LockboxRecoverySalvage(archive, key, signing?.Handle ?? IntPtr.Zero));

    /// <summary>An open encrypted archive containing files, variables, secrets, and forms.</summary>
    public sealed class Lockbox : IDisposable
    {
        private readonly Revault owner; internal IntPtr Handle;
        internal Lockbox(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Stages a file at the Lockbox path; replace controls an existing entry.</summary>
        public void AddFile(string path, byte[] value, bool replace = false) => owner.operations.LockboxAddFile(Handle, path, value, replace);
        /// <summary>Stages a file at the Lockbox path; replace controls an existing entry.</summary>
        public void AddFile(string path, byte[] value, uint permissions, bool replace = false) => owner.operations.LockboxAddFileWithPermissions(Handle, path, value, permissions, replace);
        /// <summary>Reads the complete file stored at the Lockbox path.</summary>
        public byte[] GetFile(string path) => owner.operations.LockboxGetFile(Handle, path);
        /// <summary>Writes one Lockbox file to the host filesystem.</summary>
        public void ExtractFile(string source, string destination, bool replace = false) => owner.operations.LockboxExtractFile(Handle, source, destination, replace);
        /// <summary>Extracts the Lockbox with explicit size, count, link, and permission limits.</summary>
        public void ExtractDirectory(string destination, ulong maxFileBytes, ulong maxTotalBytes, nuint maxFiles,
            bool restoreSymlinks, bool restorePermissions, bool overwrite) => owner.operations.LockboxExtractDirectory(
                Handle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite);
        /// <summary>Lists logical or physical content chunks for streaming diagnostics.</summary>
        public IReadOnlyList<StreamChunk> StreamContent(bool physical = false) => owner.operations.LockboxStreamContent(Handle, physical);
        /// <summary>Returns cache statistics for this lockbox.</summary>
        public CacheStats CacheStats() => owner.operations.LockboxCacheStats(Handle);
        /// <summary>Returns import statistics for this lockbox.</summary>
        public ImportStats ImportStats() => owner.operations.LockboxImportStats(Handle);
        /// <summary>Updates import stats.</summary>
        public void ResetImportStats() => owner.operations.LockboxResetImportStats(Handle);
        /// <summary>Returns page metadata for diagnostics without exposing plaintext secrets.</summary>
        public IReadOnlyList<PageInspection> PageInspection() => owner.operations.LockboxPageInspection(Handle);
        /// <summary>Scans the open archive and returns its structured recovery report.</summary>
        public RecoveryReport RecoveryReport() => owner.operations.LockboxRecoveryReport(Handle);
        /// <summary>Returns the render recovery report.</summary>
        public string RenderRecoveryReport(bool verbose, nuint maxEntries) => owner.operations.LockboxRecoveryReportRender(Handle, verbose, maxEntries);
        /// <summary>Returns the storage length.</summary>
        public ulong StorageLength => owner.operations.LockboxStorageLen(Handle);
        /// <summary>Sets workload profile.</summary>
        public void SetWorkloadProfile(string profile) => owner.operations.LockboxSetWorkloadProfile(Handle, profile);
        /// <summary>Sets worker policy.</summary>
        public void SetWorkerPolicy(string mode, nuint jobs) => owner.operations.LockboxSetWorkerPolicy(Handle, mode, jobs);
        /// <summary>Returns the cache, workload, and worker settings used by this Lockbox.</summary>
        public RuntimeOptions RuntimeOptions() => owner.operations.LockboxRuntimeOptions(Handle);
        /// <summary>Authenticates and publishes the staged changes.</summary>
        public void Commit() => owner.operations.LockboxCommit(Handle);
        /// <summary>Creates directory.</summary>
        public void CreateDirectory(string path, bool parents = false) => owner.operations.LockboxCreateDir(Handle, path, parents);
        /// <summary>Stages removal of a file, link, or empty directory at path.</summary>
        public void Delete(string path) => owner.operations.LockboxDelete(Handle, path);
        /// <summary>Removes directory.</summary>
        public void RemoveDirectory(string path, bool recursive = false) => owner.operations.LockboxRemoveDir(Handle, path, recursive);
        /// <summary>Creates parent directories.</summary>
        public void CreateParentDirectories(string path) => owner.operations.LockboxCreateParentDirs(Handle, path);
        /// <summary>Stages an atomic move from one Lockbox path to another.</summary>
        public void Rename(string from, string to) => owner.operations.LockboxRename(Handle, from, to);
        /// <summary>Lists entries below path, optionally including descendants.</summary>
        public IReadOnlyList<LockboxEntry> List(string path = "/", bool recursive = false) => owner.operations.LockboxList(Handle, path, recursive);
        /// <summary>Lists entries below path, optionally including descendants.</summary>
        public IReadOnlyList<LockboxEntry> List(string path, string glob, bool recursive, bool includeFiles,
            bool includeSymlinks, bool includeDirectories, nuint limit) => owner.operations.LockboxListWithOptions(
                Handle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit);
        /// <summary>Returns metadata for the selected lockbox entry.</summary>
        public LockboxEntry? Stat(string path) => owner.operations.LockboxStat(Handle, path);
        /// <summary>Stages a plain text variable; commit to publish the change.</summary>
        public void SetVariable(string name, string value) => owner.operations.LockboxSetVariable(Handle, name, value);
        /// <summary>Stores a secret variable from mutable bytes.</summary>
        public void SetSecretVariable(string name, byte[] value) => owner.operations.LockboxSetSecretVariable(Handle, name, value);
        /// <summary>Returns a plain variable when it is present.</summary>
        public string? GetVariable(string name) => owner.operations.LockboxGetVariable(Handle, name);
        /// <summary>Returns the encrypted Lockbox description, or null when unset. Example: <c>box.SetDescription("Production credentials"); box.Commit(); Console.WriteLine(box.Description);</c></summary>
        public string? Description => GetVariable("/.revault/description");
        /// <summary>Stages an encrypted Lockbox description; call Commit to publish it. Example: <c>box.SetDescription("Production credentials"); box.Commit();</c></summary>
        public void SetDescription(string description) => SetVariable("/.revault/description", description);
        /// <summary>Stages removal of the encrypted Lockbox description; call Commit. Example: <c>box.ClearDescription(); box.Commit();</c></summary>
        public void ClearDescription() => DeleteVariable("/.revault/description");
        /// <summary>Invokes <paramref name="callback"/> with temporary secret bytes, then wipes the transfer buffer.</summary>
        public T? WithSecretVariable<T>(string name, SecretCallback<T> callback) => owner.operations.LockboxWithSecretVariable(Handle, name, callback);
        /// <summary>Stages removal of a variable.</summary>
        public void DeleteVariable(string name) => owner.operations.LockboxDeleteVariable(Handle, name);
        /// <summary>Updates variables.</summary>
        public void MoveVariables(IReadOnlyList<PathMove> moves) => owner.operations.LockboxMoveVariables(Handle, DomainCodec.EncodePathMoves(moves));
        /// <summary>Lists variable names and metadata without exposing secret values.</summary>
        public IReadOnlyList<Variable> ListVariables() => owner.operations.LockboxListVariables(Handle);
        /// <summary>Returns whether a variable is plain or secret.</summary>
        public string? VariableSensitivity(string name) => owner.operations.LockboxVariableSensitivity(Handle, name);
        /// <summary>Stages a symbolic link with its stored target text.</summary>
        public void AddSymlink(string path, string target, bool replace = false) => owner.operations.LockboxAddSymlink(Handle, path, target, replace);
        /// <summary>Returns the symlink target.</summary>
        public string SymlinkTarget(string path) => owner.operations.LockboxGetSymlinkTarget(Handle, path);
        /// <summary>Returns the stable public identifier stored in the Lockbox header.</summary>
        public byte[] Id => owner.operations.LockboxId(Handle);
        /// <summary>Reports whether an entry exists at path.</summary>
        public bool Exists(string path) => owner.operations.LockboxExists(Handle, path);
        /// <summary>Reports whether path names a directory entry.</summary>
        public bool IsDirectory(string path) => owner.operations.LockboxIsDir(Handle, path);
        /// <summary>Returns the portable Unix permission bits stored for path.</summary>
        public uint Permissions(string path) => owner.operations.LockboxPermissions(Handle, path);
        /// <summary>Stages portable Unix permission bits for path.</summary>
        public void SetPermissions(string path, uint value) => owner.operations.LockboxSetPermissions(Handle, path, value);
        /// <summary>Reads the requested byte range from a stored file.</summary>
        public byte[] ReadRange(string path, ulong offset, ulong length) => owner.operations.LockboxReadRange(Handle, path, offset, length);
        /// <summary>Adds a password access slot and returns its slot identifier.</summary>
        public ulong AddPassword(byte[] password) { var id = owner.operations.LockboxAddPassword(Handle, password); if (id == ulong.MaxValue) throw new RevaultException(owner.LastError); return id; }
        /// <summary>Grants a named contact access and returns the new slot identifier.</summary>
        public ulong AddContact(ContactPublicKey contact, string name) { var id = owner.operations.LockboxAddContact(Handle, contact.Handle, name); if (id == ulong.MaxValue) throw new RevaultException(owner.LastError); return id; }
        /// <summary>Removes an access slot; at least one usable slot must remain.</summary>
        public void DeleteKey(ulong id) => owner.operations.LockboxDeleteKey(Handle, id);
        /// <summary>Lists public access slot metadata without returning credentials.</summary>
        public IReadOnlyList<KeySlot> ListKeySlots() => owner.operations.LockboxListKeySlots(Handle);
        /// <summary>Assigns a profile signing key to the Lockbox owner role.</summary>
        public void SetOwnerSigningKey(ProfileSigningKeyPair key) => owner.operations.LockboxSetOwnerSigningKey(Handle, key.Handle);
        /// <summary>Returns public signing and ownership metadata for the current revision.</summary>
        public OwnerInspection OwnerInspection() => owner.operations.LockboxOwnerInspection(Handle);
        /// <summary>Defines and stores a reusable versioned form.</summary>
        public FormDefinition DefineForm(string alias, string name, string description, IReadOnlyList<FormField> fields) =>
            owner.operations.LockboxDefineForm(Handle, alias, name, description, DomainCodec.EncodeFormFields(fields));
        /// <summary>Lists the form definitions stored in this Lockbox.</summary>
        public IReadOnlyList<FormDefinition> ListFormDefinitions() => owner.operations.LockboxListFormDefinitions(Handle);
        /// <summary>Resolves a form alias, type identifier, or revision.</summary>
        public FormDefinition ResolveForm(string reference) => owner.operations.LockboxResolveForm(Handle, reference);
        /// <summary>Lists every stored revision for a form type identifier.</summary>
        public IReadOnlyList<FormDefinition> ListFormRevisions(string typeId) => owner.operations.LockboxListFormRevisions(Handle, typeId);
        /// <summary>Stages a form record at path using the referenced definition.</summary>
        public FormRecord CreateFormRecord(string path, string typeReference, string name) => owner.operations.LockboxCreateFormRecord(Handle, path, typeReference, name);
        /// <summary>Stages a plain field value in a form record.</summary>
        public void SetFormField(string path, string field, string value) => owner.operations.LockboxSetFormField(Handle, path, field, value);
        /// <summary>Stores a secret form field from mutable bytes.</summary>
        public void SetSecretFormField(string path, string field, byte[] value) => owner.operations.LockboxSetSecretFormField(Handle, path, field, value);
        /// <summary>Lists form records without exposing secret field values.</summary>
        public IReadOnlyList<FormRecord> ListFormRecords() => owner.operations.LockboxListFormRecords(Handle);
        /// <summary>Returns the form record at path when present.</summary>
        public FormRecord? GetFormRecord(string path) => owner.operations.LockboxGetFormRecord(Handle, path);
        /// <summary>Stages removal of a form record.</summary>
        public void DeleteFormRecord(string path) => owner.operations.LockboxDeleteFormRecord(Handle, path);
        /// <summary>Updates form records.</summary>
        public void MoveFormRecords(IReadOnlyList<PathMove> moves) => owner.operations.LockboxMoveFormRecords(Handle, DomainCodec.EncodePathMoves(moves));
        /// <summary>Returns a plain form field when it exists.</summary>
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
    public Vault OpenVault(string root, byte[] password) => new(this, operations.VaultDirectoryOpen(root, password));
    /// <summary>Opens or create vault directory.</summary>
    public Vault OpenOrCreateVault(string root, byte[] password) => new(this, operations.VaultDirectoryOpenOrCreate(root, password));
    /// <summary>Updates vault directory.</summary>
    public Vault ReplaceVault(string root, byte[] password) => new(this, operations.VaultDirectoryReplace(root, password));
    /// <summary>Opens or create default vault directory.</summary>
    public Vault OpenOrCreateDefaultVault(byte[] password) => new(this, operations.VaultDirectoryOpenOrCreateDefault(password));
    /// <summary>Updates default vault directory.</summary>
    public Vault ReplaceDefaultVault(byte[] password) => new(this, operations.VaultDirectoryReplaceDefault(password));
    /// <summary>Updates vault directory password.</summary>
    public void ChangeVaultPassword(string root, byte[] oldPassword, byte[] newPassword) => operations.VaultDirectoryChangePassword(root, oldPassword, newPassword);
    /// <summary>Updates default vault directory password.</summary>
    public void ChangeDefaultVaultPassword(byte[] oldPassword, byte[] newPassword) => operations.VaultDirectoryChangeDefaultPassword(oldPassword, newPassword);
    /// <summary>Returns the default vault directory.</summary>
    public string DefaultVaultRoot => operations.VaultDefaultDirectory();
    /// <summary>Returns the default vault path.</summary>
    public string DefaultVaultPath => operations.VaultDefaultPath();
    /// <summary>Returns the backup default vault.</summary>
    public VaultBackupManifest BackupDefaultVault(string path, bool overwrite = false) => operations.VaultBackupDefault(path, overwrite);
    /// <summary>Returns the restore default vault.</summary>
    public VaultBackupManifest RestoreDefaultVault(string path, bool overwrite = false) => operations.VaultRestoreDefault(path, overwrite);

    /// <summary>Password-protected storage for Profile keys, contacts, forms, backups, and known lockbox paths.</summary>
    public class VaultStore : IDisposable
    {
        private readonly Revault owner; internal IntPtr Handle;
        internal VaultStore(Revault owner, IntPtr handle) { this.owner = owner; Handle = handle; }
        /// <summary>Returns the canonical root directory of this Vault.</summary>
        public string Root => owner.operations.VaultDirectoryRoot(Handle);
        /// <summary>Returns the persistent structure version of this Vault.</summary>
        public uint StructureVersion => owner.operations.VaultDirectoryStructureVersion(Handle);
        /// <summary>Lists private keys.</summary>
        public IReadOnlyList<string> ListPrivateKeys() => owner.operations.VaultDirectoryListPrivateKeys(Handle);
        /// <summary>Lists private key names.</summary>
        public IReadOnlyList<string> ListPrivateKeyNames() => owner.operations.VaultDirectoryListPrivateKeyNames(Handle);
        /// <summary>Lists contact names.</summary>
        public IReadOnlyList<string> ListContactNames() => owner.operations.VaultDirectoryListContactNames(Handle);
        /// <summary>Lists form aliases.</summary>
        public IReadOnlyList<string> ListFormAliases() => owner.operations.VaultDirectoryListFormAliases(Handle);
        /// <summary>Reports whether the named profile private key exists.</summary>
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
        /// <summary>Reports whether the named contact exists.</summary>
        public bool ContactExists(string name) => owner.operations.VaultDirectoryContactExists(Handle, name);
        /// <summary>Removes contact.</summary>
        public void DeleteContact(string name) => owner.operations.VaultDirectoryDeleteContact(Handle, name);
        /// <summary>Lists contacts.</summary>
        public IReadOnlyList<Contact> ListContacts() => owner.operations.VaultDirectoryListContacts(Handle);
        /// <summary>Stores profile email.</summary>
        public void StoreProfileEmail(string name, string email) => owner.operations.VaultDirectoryStoreProfileEmail(Handle, name, email);
        /// <summary>Returns the email recorded for a profile, when present.</summary>
        public string? ProfileEmail(string name) => owner.operations.VaultDirectoryProfileEmail(Handle, name);
        /// <summary>Stores backup.</summary>
        public void StoreBackup(byte[] id, byte[] value) => owner.operations.VaultDirectoryStoreBackup(Handle, id, value);
        /// <summary>Loads backup.</summary>
        public byte[] LoadBackup(byte[] id) => owner.operations.VaultDirectoryLoadBackup(Handle, id);
        /// <summary>Returns the number of stored key recovery backups.</summary>
        public ulong BackupCount => owner.operations.VaultDirectoryBackupCount(Handle);
        /// <summary>Restores a profile private key and signing key from recovery material.</summary>
        public void RestorePrivateKey(string name, ContactKeyPair key, ProfileSigningKeyPair signing, bool overwrite) =>
            owner.operations.VaultDirectoryRestorePrivateKey(Handle, name, key.Handle, signing.Handle, overwrite);
        /// <summary>Loads the current profile signing key.</summary>
        public ProfileSigningKeyPair LoadProfileSigningKey(string name) => new(owner, owner.operations.VaultDirectoryLoadOwnerSigningKey(Handle, name));
        /// <summary>Loads a historical profile signing key generation.</summary>
        public ProfileSigningKeyPair LoadProfileSigningKeyGeneration(string name, ushort index) =>
            new(owner, owner.operations.VaultDirectoryLoadOwnerSigningKeyGeneration(Handle, name, index));
        /// <summary>Stores contact signing key.</summary>
        public void StoreContactSigningKey(string name, ProfileSigningPublicKey key) => owner.operations.VaultDirectoryStoreContactSigningKey(Handle, name, key.Handle);
        /// <summary>Loads contact signing key.</summary>
        public ProfileSigningPublicKey LoadContactSigningKey(string name) => new(owner, owner.operations.VaultDirectoryLoadContactSigningKey(Handle, name));
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
        /// <summary>Finds access slot labels with the supplied name for one Lockbox.</summary>
        public IReadOnlyList<AccessSlotLabel> FindAccessSlotLabels(byte[] id, string name) => owner.operations.VaultDirectoryFindAccessSlotLabels(Handle, id, name);
        /// <summary>Removes access slot label.</summary>
        public void ForgetAccessSlotLabel(byte[] id, ulong slotId) => owner.operations.VaultDirectoryForgetAccessSlotLabel(Handle, id, slotId);
        /// <summary>Defines and stores a reusable versioned form.</summary>
        public FormDefinition DefineForm(string alias, string name, string description, IReadOnlyList<FormField> fields) =>
            owner.operations.VaultDirectoryDefineForm(Handle, alias, name, description, DomainCodec.EncodeFormFields(fields));
        /// <summary>Resolves a form alias, type identifier, or revision.</summary>
        public FormDefinition ResolveForm(string reference) => owner.operations.VaultDirectoryResolveForm(Handle, reference);
        /// <summary>Lists forms.</summary>
        public IReadOnlyList<FormDefinition> ListForms() => owner.operations.VaultDirectoryListForms(Handle);
        /// <summary>Lists every stored revision for a form type identifier.</summary>
        public IReadOnlyList<FormDefinition> ListFormRevisions(string typeId) => owner.operations.VaultDirectoryListFormRevisions(Handle, typeId);
        /// <summary>Adds missing standard form definitions and returns the number added.</summary>
        public nuint SeedForms() => owner.operations.VaultDirectorySeedForms(Handle);
        /// <summary>Stores password.</summary>
        public void RememberPassword(byte[] id, byte[] password) => owner.operations.VaultDirectoryRememberPassword(Handle, id, password);
        /// <summary>Returns the Lockbox password encrypted inside this Vault.</summary>
        public byte[] RememberedPassword(byte[] id) => owner.operations.VaultDirectoryRememberedPassword(Handle, id);
        /// <summary>Releases the native resources held by this object.</summary>
        public void Dispose() { if (Handle != IntPtr.Zero) { owner.operations.VaultDirectoryFree(Handle); Handle = IntPtr.Zero; } GC.SuppressFinalize(this); }
        /// <summary>Releases the writable vault handle during finalization.</summary>
        ~VaultStore() => Dispose();
    }

    /// <summary>Opens read only vault directory.</summary>
    public ReadOnlyVault OpenReadOnlyVault(string root, byte[] password) =>
        new(this, operations.VaultReadOnlyOpen(root, password));
    /// <summary>Opens default read only vault directory.</summary>
    public ReadOnlyVault OpenDefaultReadOnlyVault(byte[] password) =>
        new(this, operations.VaultReadOnlyOpenDefault(password));
    /// <summary>A metadata view for discovery and diagnostics that never loads private profile signing material.</summary>
    public class ReadOnlyVaultStore : IDisposable
    {
        private readonly Revault owner; private IntPtr handle;
        internal ReadOnlyVaultStore(Revault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
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
        ~ReadOnlyVaultStore() => Dispose();
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
    /// <summary>Caches a profile signing key in the Session Agent.</summary>
    public void CacheProfileSigningKey(string vaultId, string profile, ProfileSigningKeyPair key, ulong ttlSeconds) =>
        operations.VaultAgentPutOwnerSigningKey(vaultId, profile, key.Handle, ttlSeconds);
    /// <summary>Returns a profile signing key cached by the Session Agent.</summary>
    public ProfileSigningKeyPair ProfileSigningKey(string vaultId, string profile) => new(this, operations.VaultAgentGetOwnerSigningKey(vaultId, profile));
    /// <summary>Removes a cached profile signing key.</summary>
    public void ForgetProfileSigningKey(string vaultId, string profile) => operations.VaultAgentForgetOwnerSigningKey(vaultId, profile);
    /// <summary>Starts agent activity.</summary>
    public AgentActivity BeginAgentActivity(string kind) => new(this, operations.VaultAgentBeginActivity(kind));

    /// <summary>A token kept alive while an operation needs secrets cached by the Session Agent.</summary>
    public sealed class AgentActivity : IDisposable
    {
        private readonly Revault owner; private IntPtr handle;
        internal AgentActivity(Revault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
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
    public LockboxSession OpenLockboxSession() => new(this, operations.VaultLocal());
    /// <summary>A session that opens lockboxes by host path, caches passwords, and closes locally used files.</summary>
    public sealed class LockboxSession : IDisposable
    {
        private readonly Revault owner; private IntPtr handle;
        internal LockboxSession(Revault owner, IntPtr handle) { this.owner = owner; this.handle = handle; }
        /// <summary>Creates with password.</summary>
        public Lockbox CreateWithPassword(string path, byte[] password) => new(owner, owner.operations.VaultCreateLockboxPassword(handle, path, password));
        /// <summary>Opens with password.</summary>
        public Lockbox OpenWithPassword(string path, byte[] password) => new(owner, owner.operations.VaultOpenLockboxPassword(handle, path, password));
        /// <summary>Creates with content key.</summary>
        public Lockbox CreateWithContentKey(string path, byte[] key, ProfileSigningKeyPair signing) =>
            new(owner, owner.operations.VaultCreateLockboxContentKey(handle, path, key, signing.Handle));
        /// <summary>Opens with content key.</summary>
        public Lockbox OpenWithContentKey(string path, byte[] key, ProfileSigningKeyPair signing) =>
            new(owner, owner.operations.VaultOpenLockboxContentKey(handle, path, key, signing.Handle));
        /// <summary>Creates for contact.</summary>
        public Lockbox CreateForContact(string path, ContactPublicKey contact, string name, ProfileSigningKeyPair signing) =>
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
        ~LockboxSession() => Dispose();
    }
}
