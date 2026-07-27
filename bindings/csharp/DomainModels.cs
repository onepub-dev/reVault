using Google.FlatBuffers;

namespace Revault;

/// <summary>Identifies the filesystem object stored at a lockbox path.</summary>
public enum LockboxEntryKind
{
    /// <summary>No recognized kind was reported.</summary>
    Unspecified,
    /// <summary>A regular file.</summary>
    File,
    /// <summary>A symbolic link.</summary>
    Symlink,
    /// <summary>A directory.</summary>
    Directory
}

/// <summary>Metadata for one file, directory, or symbolic link stored at a lockbox path.</summary>
/// <param name="Path">Absolute lockbox path of the stored entry.</param>
/// <param name="Kind">Filesystem kind: file, directory, or symbolic link.</param>
/// <param name="Length">Logical file length in bytes; zero for directories.</param>
/// <param name="Permissions">Portable Unix permission bits stored with the entry.</param>
public sealed record LockboxEntry(string Path, LockboxEntryKind Kind, ulong Length, uint Permissions);

/// <summary>A source and destination pair used to rename a variable or form record atomically.</summary>
/// <param name="Source">Existing variable name or form-record path to rename.</param>
/// <param name="Destination">New variable name or form-record path.</param>
public sealed record PathMove(string Source, string Destination);

/// <summary>One named input in a reusable form definition, including its display label and sensitivity kind.</summary>
/// <param name="Id">Stable field identifier used when reading and writing records.</param>
/// <param name="Label">Human-readable label presented to a person entering data.</param>
/// <param name="Kind">Field kind that determines validation and secret handling.</param>
/// <param name="Required">Whether a record must provide a value for this field.</param>
public sealed record FormField(string Id, string Label, string Kind, bool Required);

/// <summary>A versioned form schema used to validate and label structured records in a lockbox.</summary>
/// <param name="TypeId">Stable identifier shared by every revision of this form type.</param>
/// <param name="Alias">Short name used to resolve the current form revision.</param>
/// <param name="Revision">Monotonically increasing revision number.</param>
/// <param name="Name">Human-readable name shown for this form.</param>
/// <param name="Description">Explanation shown to people completing the form.</param>
/// <param name="Fields">Ordered inputs accepted by this form revision.</param>
public sealed record FormDefinition(string TypeId, string Alias, uint Revision, string Name, string Description, IReadOnlyList<FormField> Fields);

/// <summary>The current value and sensitivity metadata for one field in a stored form record.</summary>
/// <param name="FieldId">Identifier of the form field that owns this value.</param>
/// <param name="Label">Display label captured from the form revision.</param>
/// <param name="Kind">Field kind captured from the form revision.</param>
/// <param name="Value">Plain value, or an empty string when the field is secret.</param>
/// <param name="Secret">Whether the value must be read through a scoped secret callback.</param>
public sealed record FormValue(string FieldId, string Label, string Kind, string Value, bool Secret);

/// <summary>A named structured record stored at a lockbox path and tied to a form-definition revision.</summary>
/// <param name="Path">Absolute lockbox path that identifies the record.</param>
/// <param name="Name">Human-readable name assigned to this record.</param>
/// <param name="TypeId">Stable identifier of the record's form type.</param>
/// <param name="DefinitionAlias">Alias of the form definition used by the record.</param>
/// <param name="DefinitionRevision">Exact form revision against which the record was created.</param>
/// <param name="Values">Ordered non-secret field metadata and values.</param>
public sealed record FormRecord(string Path, string Name, string TypeId, string DefinitionAlias, uint DefinitionRevision, IReadOnlyList<FormValue> Values);

/// <summary>The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox.</summary>
/// <param name="IntactFiles">Files whose complete contents remain recoverable.</param>
/// <param name="IntactFileCount">Number of completely recoverable files.</param>
/// <param name="PartialFiles">Number of files for which only some content is recoverable.</param>
/// <param name="CorruptRecords">Number of encrypted records that failed validation.</param>
/// <param name="TocRecovered">Whether a usable table of contents was recovered.</param>
/// <param name="VariablesRecovered">Whether variable metadata was recovered.</param>
/// <param name="VariableCount">Number of recovered variables.</param>
/// <param name="FormsRecovered">Whether form definitions and records were recovered.</param>
/// <param name="FormDefinitionCount">Number of recovered form definitions.</param>
/// <param name="FormRecordCount">Number of recovered form records.</param>
public sealed record RecoveryReport(IReadOnlyList<LockboxEntry> IntactFiles, ulong IntactFileCount, ulong PartialFiles, ulong CorruptRecords, bool TocRecovered, bool VariablesRecovered, ulong VariableCount, bool FormsRecovered, ulong FormDefinitionCount, ulong FormRecordCount);

/// <summary>One password or contact credential that can unlock a lockbox content key.</summary>
/// <param name="Id">Stable slot identifier used when removing this access method.</param>
/// <param name="Protection">Access method, such as password or contact key.</param>
/// <param name="Algorithm">Cryptographic algorithm protecting the content key.</param>
public sealed record KeySlot(ulong Id, string Protection, string Algorithm);

/// <summary>Current capacity, occupancy, hit, and miss counters for an open lockbox cache.</summary>
/// <param name="LimitBytes">Maximum decoded-page memory permitted for the cache.</param>
/// <param name="UsedBytes">Decoded-page memory currently held by the cache.</param>
/// <param name="Entries">Number of decoded pages currently cached.</param>
/// <param name="Hits">Reads served by an already decoded page.</param>
/// <param name="Misses">Reads that required decoding another page.</param>
public sealed record CacheStats(ulong LimitBytes, ulong UsedBytes, ulong Entries, ulong Hits, ulong Misses);

/// <summary>Time spent reading host files and preparing encrypted pages during the latest import work.</summary>
/// <param name="HostStatNanos">Nanoseconds spent reading host filesystem metadata, as decimal text.</param>
/// <param name="HostReadNanos">Nanoseconds spent reading host file content, as decimal text.</param>
/// <param name="FramePrepareNanos">Nanoseconds spent preparing encrypted records, as decimal text.</param>
/// <param name="PageWriteNanos">Nanoseconds spent writing encrypted pages, as decimal text.</param>
public sealed record ImportStats(string HostStatNanos, string HostReadNanos, string FramePrepareNanos, string PageWriteNanos);

/// <summary>One logical object recorded inside an inspected encrypted lockbox page.</summary>
/// <param name="Id">Object identifier recorded in the encrypted page.</param>
/// <param name="Kind">Kind of logical object stored in the page.</param>
/// <param name="PayloadLen">Encrypted object payload length in bytes.</param>
public sealed record PageObject(ulong Id, string Kind, ulong PayloadLen);

/// <summary>Layout and utilization details for one encrypted page in a lockbox archive.</summary>
/// <param name="Offset">Byte offset at which the page begins in the archive.</param>
/// <param name="PageId">Identifier stored in the page header.</param>
/// <param name="Sequence">Commit sequence that wrote this page.</param>
/// <param name="PageSize">Total encoded page size in bytes.</param>
/// <param name="EncryptedBodyLen">Encrypted body length in bytes.</param>
/// <param name="UnusedBytes">Unused capacity remaining in the page.</param>
/// <param name="ObjectCount">Number of logical objects stored in the page.</param>
/// <param name="Objects">Logical objects discovered in the page.</param>
public sealed record PageInspection(ulong Offset, ulong PageId, ulong Sequence, ulong PageSize, ulong EncryptedBodyLen, ulong UnusedBytes, ulong ObjectCount, IReadOnlyList<PageObject> Objects);

/// <summary>Header, owner-signature, and key-slot information read from a lockbox file without opening its contents.</summary>
/// <param name="LockboxId">Stable binary identifier read from the lockbox header.</param>
/// <param name="HeaderReadable">Whether the archive header passed structural validation.</param>
/// <param name="KeyDirectoryGeneration">Latest readable access-key directory generation.</param>
/// <param name="KeyDirectoryCopyCount">Number of readable redundant key-directory copies.</param>
/// <param name="OwnerSigned">Whether commits require an owner signature.</param>
/// <param name="KeySlots">Password and contact access methods found in the header.</param>
public sealed record FileInspection(byte[] LockboxId, bool HeaderReadable, ulong KeyDirectoryGeneration, ulong KeyDirectoryCopyCount, bool OwnerSigned, IReadOnlyList<KeySlot> KeySlots);

/// <summary>One active or retired generation of the contact keys belonging to a named vault profile.</summary>
/// <param name="Index">Generation number used to address this key version.</param>
/// <param name="Status">Lifecycle state, such as active or retired.</param>
/// <param name="ContactFingerprint">Fingerprint of this generation's contact public key.</param>
/// <param name="CreatedAtUnixMs">Creation time in Unix milliseconds.</param>
/// <param name="RetiredAtUnixMs">Retirement time in Unix milliseconds when retired.</param>
/// <param name="HasRetiredAt">Whether a retirement time is present.</param>
public sealed record ProfileGeneration(uint Index, string Status, byte[] ContactFingerprint, ulong CreatedAtUnixMs, ulong RetiredAtUnixMs, bool HasRetiredAt);

/// <summary>The active generation and rotation history for a named vault profile.</summary>
/// <param name="Name">Vault profile name whose generations are listed.</param>
/// <param name="ActiveGeneration">Generation number currently used for new access grants.</param>
/// <param name="Generations">Active and retired contact-key generations.</param>
public sealed record ProfileHistory(string Name, uint ActiveGeneration, IReadOnlyList<ProfileGeneration> Generations);

/// <summary>A lockbox identifier and host path remembered by the local vault for later discovery.</summary>
/// <param name="LockboxId">Stable binary identifier of the remembered lockbox.</param>
/// <param name="Path">Last known host filesystem path of the lockbox.</param>
/// <param name="LastSeenUnixMs">Most recent observation time in Unix milliseconds.</param>
public sealed record KnownLockbox(byte[] LockboxId, string Path, ulong LastSeenUnixMs);

/// <summary>A local human-readable label attached to one lockbox access slot.</summary>
/// <param name="LockboxId">Lockbox whose access slot is labelled.</param>
/// <param name="SlotId">Stable identifier of the labelled access slot.</param>
/// <param name="Name">Local human-readable label for the access slot.</param>
/// <param name="UpdatedAtUnixMs">Last label update time in Unix milliseconds.</param>
public sealed record AccessSlotLabel(byte[] LockboxId, ulong SlotId, string Name, ulong UpdatedAtUnixMs);

/// <summary>A logical or physical byte range emitted while walking the contents of a lockbox.</summary>
/// <param name="Path">Lockbox file path to which this byte range belongs.</param>
/// <param name="FileOffset">Logical byte offset within the file.</param>
/// <param name="Length">Logical range length in bytes.</param>
/// <param name="PhysicalOffset">Archive byte offset, when physical streaming is requested.</param>
/// <param name="Sparse">Whether the range represents a sparse zero-filled extent.</param>
/// <param name="Data">File bytes for a populated logical range.</param>
public sealed record StreamChunk(string Path, ulong FileOffset, ulong Length, ulong PhysicalOffset, bool Sparse, byte[] Data);

/// <summary>The workload and worker policies currently applied to an open lockbox.</summary>
/// <param name="WorkloadProfile">I/O workload policy used to tune page access.</param>
/// <param name="WorkerPolicy">Worker scheduling policy and effective parallelism.</param>
public sealed record RuntimeOptions(string WorkloadProfile, string WorkerPolicy);

/// <summary>The name and sensitivity classification of a variable stored in a lockbox.</summary>
/// <param name="Name">Name used to address the variable in the lockbox.</param>
/// <param name="Sensitivity">Whether the value is ordinary text or a protected secret.</param>
public sealed record Variable(string Name, string Sensitivity);

/// <summary>Whether a lockbox is owner-signed and, when available, the signing-key fingerprint.</summary>
/// <param name="Signed">Whether the lockbox requires owner-signed commits.</param>
/// <param name="Fingerprint">Owner signing-key fingerprint when one is configured.</param>
/// <param name="HasFingerprint">Whether an owner fingerprint is available.</param>
public sealed record OwnerInspection(bool Signed, string Fingerprint, bool HasFingerprint);

/// <summary>A named recipient public key stored in the local vault address book.</summary>
/// <param name="Name">Local address-book name of the contact.</param>
/// <param name="Key">Serialized contact public key used to grant lockbox access.</param>
public sealed record Contact(string Name, byte[] Key);

/// <summary>A lockbox key currently held by the local session agent, identified by lockbox and path.</summary>
/// <param name="Id">Stable lockbox identifier for the cached key.</param>
/// <param name="Path">Host path associated with the cached lockbox key.</param>
public sealed record AgentEntry(string Id, string Path);

/// <summary>The host capabilities used to protect cached secrets across suspend and sleep.</summary>
/// <param name="SuspendNotifications">Whether the host reports impending system suspend.</param>
/// <param name="SleepInhibition">Whether the agent can delay sleep while handling secrets.</param>
/// <param name="Supported">Whether the host supplies enough integration for safe caching.</param>
public sealed record SleepSupport(bool SuspendNotifications, bool SleepInhibition, bool Supported);

/// <summary>Availability and configuration of the operating-system credential store used for the vault password.</summary>
/// <param name="Supported">Whether a usable operating-system credential store exists.</param>
/// <param name="Disabled">Whether the user disabled credential-store integration.</param>
/// <param name="Scope">Application-specific scope used to isolate the stored password.</param>
/// <param name="Backend">Operating-system credential-store backend in use.</param>
/// <param name="Item">Credential item name used by the backend.</param>
public sealed record PlatformStatus(bool Supported, bool Disabled, string Scope, string Backend, string Item);

/// <summary>The version, size, checksum, and creation time of an exported local-vault backup.</summary>
/// <param name="FormatVersion">Backup container format version.</param>
/// <param name="CreatedAtUnixMs">Backup creation time in Unix milliseconds.</param>
/// <param name="VaultFileName">Metadata-vault filename stored in the backup.</param>
/// <param name="VaultSize">Encrypted vault payload size in bytes.</param>
/// <param name="VaultSha256">Lowercase SHA-256 digest of the encrypted vault payload.</param>
public sealed record VaultBackupManifest(uint FormatVersion, ulong CreatedAtUnixMs, string VaultFileName, ulong VaultSize, string VaultSha256);

/// <summary>Structured category, version, guidance, and artifact context for the most recent native failure.</summary>
/// <param name="Category">Stable error category suitable for programmatic handling.</param>
/// <param name="ArtifactKind">Kind of archive or vault artifact involved in the failure.</param>
/// <param name="FoundVersion">Format version read from the failing artifact.</param>
/// <param name="SupportedVersion">Newest format version supported by this library.</param>
/// <param name="Message">Human-readable explanation of the failure.</param>
/// <param name="Guidance">Suggested corrective action for the caller or user.</param>
public sealed record ErrorDetails(string Category, string ArtifactKind, uint FoundVersion, uint SupportedVersion, string Message, string Guidance);

/// <summary>Converts private native transport buffers into public domain values.</summary>
internal static class DomainCodec
{
    private static LockboxEntry FromInternal(Revault.Internal.Transport.LockboxEntryT value) =>
        new(value.Path ?? string.Empty, (LockboxEntryKind)(int)value.Kind, value.Length, value.Permissions);
    internal static LockboxEntry LockboxEntry(byte[] bytes) => FromInternal(Revault.Internal.Transport.LockboxEntry.GetRootAsLockboxEntry(new ByteBuffer(bytes)).UnPack());
    private static PathMove FromInternal(Revault.Internal.Transport.PathMoveT value) =>
        new(value.Source ?? string.Empty, value.Destination ?? string.Empty);
    internal static PathMove PathMove(byte[] bytes) => FromInternal(Revault.Internal.Transport.PathMove.GetRootAsPathMove(new ByteBuffer(bytes)).UnPack());
    private static FormField FromInternal(Revault.Internal.Transport.FormFieldT value) =>
        new(value.Id ?? string.Empty, value.Label ?? string.Empty, value.Kind ?? string.Empty, value.Required);
    internal static FormField FormField(byte[] bytes) => FromInternal(Revault.Internal.Transport.FormField.GetRootAsFormField(new ByteBuffer(bytes)).UnPack());
    private static FormDefinition FromInternal(Revault.Internal.Transport.FormDefinitionT value) =>
        new(value.TypeId ?? string.Empty, value.Alias ?? string.Empty, value.Revision, value.Name ?? string.Empty, value.Description ?? string.Empty, value.Fields is null ? Array.Empty<FormField>() : value.Fields.Select(FromInternal).ToArray());
    internal static FormDefinition FormDefinition(byte[] bytes) => FromInternal(Revault.Internal.Transport.FormDefinition.GetRootAsFormDefinition(new ByteBuffer(bytes)).UnPack());
    private static FormValue FromInternal(Revault.Internal.Transport.FormValueT value) =>
        new(value.FieldId ?? string.Empty, value.Label ?? string.Empty, value.Kind ?? string.Empty, value.Value ?? string.Empty, value.Secret);
    internal static FormValue FormValue(byte[] bytes) => FromInternal(Revault.Internal.Transport.FormValue.GetRootAsFormValue(new ByteBuffer(bytes)).UnPack());
    private static FormRecord FromInternal(Revault.Internal.Transport.FormRecordT value) =>
        new(value.Path ?? string.Empty, value.Name ?? string.Empty, value.TypeId ?? string.Empty, value.DefinitionAlias ?? string.Empty, value.DefinitionRevision, value.Values is null ? Array.Empty<FormValue>() : value.Values.Select(FromInternal).ToArray());
    internal static FormRecord FormRecord(byte[] bytes) => FromInternal(Revault.Internal.Transport.FormRecord.GetRootAsFormRecord(new ByteBuffer(bytes)).UnPack());
    private static RecoveryReport FromInternal(Revault.Internal.Transport.RecoveryReportT value) =>
        new(value.IntactFiles is null ? Array.Empty<LockboxEntry>() : value.IntactFiles.Select(FromInternal).ToArray(), value.IntactFileCount, value.PartialFiles, value.CorruptRecords, value.TocRecovered, value.VariablesRecovered, value.VariableCount, value.FormsRecovered, value.FormDefinitionCount, value.FormRecordCount);
    internal static RecoveryReport RecoveryReport(byte[] bytes) => FromInternal(Revault.Internal.Transport.RecoveryReport.GetRootAsRecoveryReport(new ByteBuffer(bytes)).UnPack());
    private static KeySlot FromInternal(Revault.Internal.Transport.KeySlotT value) =>
        new(value.Id, value.Protection ?? string.Empty, value.Algorithm ?? string.Empty);
    internal static KeySlot KeySlot(byte[] bytes) => FromInternal(Revault.Internal.Transport.KeySlot.GetRootAsKeySlot(new ByteBuffer(bytes)).UnPack());
    private static CacheStats FromInternal(Revault.Internal.Transport.CacheStatsT value) =>
        new(value.LimitBytes, value.UsedBytes, value.Entries, value.Hits, value.Misses);
    internal static CacheStats CacheStats(byte[] bytes) => FromInternal(Revault.Internal.Transport.CacheStats.GetRootAsCacheStats(new ByteBuffer(bytes)).UnPack());
    private static ImportStats FromInternal(Revault.Internal.Transport.ImportStatsT value) =>
        new(value.HostStatNanos ?? string.Empty, value.HostReadNanos ?? string.Empty, value.FramePrepareNanos ?? string.Empty, value.PageWriteNanos ?? string.Empty);
    internal static ImportStats ImportStats(byte[] bytes) => FromInternal(Revault.Internal.Transport.ImportStats.GetRootAsImportStats(new ByteBuffer(bytes)).UnPack());
    private static PageObject FromInternal(Revault.Internal.Transport.PageObjectT value) =>
        new(value.Id, value.Kind ?? string.Empty, value.PayloadLen);
    internal static PageObject PageObject(byte[] bytes) => FromInternal(Revault.Internal.Transport.PageObject.GetRootAsPageObject(new ByteBuffer(bytes)).UnPack());
    private static PageInspection FromInternal(Revault.Internal.Transport.PageInspectionT value) =>
        new(value.Offset, value.PageId, value.Sequence, value.PageSize, value.EncryptedBodyLen, value.UnusedBytes, value.ObjectCount, value.Objects is null ? Array.Empty<PageObject>() : value.Objects.Select(FromInternal).ToArray());
    internal static PageInspection PageInspection(byte[] bytes) => FromInternal(Revault.Internal.Transport.PageInspection.GetRootAsPageInspection(new ByteBuffer(bytes)).UnPack());
    private static FileInspection FromInternal(Revault.Internal.Transport.FileInspectionT value) =>
        new(value.LockboxId?.ToArray() ?? Array.Empty<byte>(), value.HeaderReadable, value.KeyDirectoryGeneration, value.KeyDirectoryCopyCount, value.OwnerSigned, value.KeySlots is null ? Array.Empty<KeySlot>() : value.KeySlots.Select(FromInternal).ToArray());
    internal static FileInspection FileInspection(byte[] bytes) => FromInternal(Revault.Internal.Transport.FileInspection.GetRootAsFileInspection(new ByteBuffer(bytes)).UnPack());
    private static ProfileGeneration FromInternal(Revault.Internal.Transport.ProfileGenerationT value) =>
        new(value.Index, value.Status ?? string.Empty, value.ContactFingerprint?.ToArray() ?? Array.Empty<byte>(), value.CreatedAtUnixMs, value.RetiredAtUnixMs, value.HasRetiredAt);
    internal static ProfileGeneration ProfileGeneration(byte[] bytes) => FromInternal(Revault.Internal.Transport.ProfileGeneration.GetRootAsProfileGeneration(new ByteBuffer(bytes)).UnPack());
    private static ProfileHistory FromInternal(Revault.Internal.Transport.ProfileHistoryT value) =>
        new(value.Name ?? string.Empty, value.ActiveGeneration, value.Generations is null ? Array.Empty<ProfileGeneration>() : value.Generations.Select(FromInternal).ToArray());
    internal static ProfileHistory ProfileHistory(byte[] bytes) => FromInternal(Revault.Internal.Transport.ProfileHistory.GetRootAsProfileHistory(new ByteBuffer(bytes)).UnPack());
    private static KnownLockbox FromInternal(Revault.Internal.Transport.KnownLockboxT value) =>
        new(value.LockboxId?.ToArray() ?? Array.Empty<byte>(), value.Path ?? string.Empty, value.LastSeenUnixMs);
    internal static KnownLockbox KnownLockbox(byte[] bytes) => FromInternal(Revault.Internal.Transport.KnownLockbox.GetRootAsKnownLockbox(new ByteBuffer(bytes)).UnPack());
    private static AccessSlotLabel FromInternal(Revault.Internal.Transport.AccessSlotLabelT value) =>
        new(value.LockboxId?.ToArray() ?? Array.Empty<byte>(), value.SlotId, value.Name ?? string.Empty, value.UpdatedAtUnixMs);
    internal static AccessSlotLabel AccessSlotLabel(byte[] bytes) => FromInternal(Revault.Internal.Transport.AccessSlotLabel.GetRootAsAccessSlotLabel(new ByteBuffer(bytes)).UnPack());
    private static StreamChunk FromInternal(Revault.Internal.Transport.StreamChunkT value) =>
        new(value.Path ?? string.Empty, value.FileOffset, value.Length, value.PhysicalOffset, value.Sparse, value.Data?.ToArray() ?? Array.Empty<byte>());
    internal static StreamChunk StreamChunk(byte[] bytes) => FromInternal(Revault.Internal.Transport.StreamChunk.GetRootAsStreamChunk(new ByteBuffer(bytes)).UnPack());
    private static RuntimeOptions FromInternal(Revault.Internal.Transport.RuntimeOptionsT value) =>
        new(value.WorkloadProfile ?? string.Empty, value.WorkerPolicy ?? string.Empty);
    internal static RuntimeOptions RuntimeOptions(byte[] bytes) => FromInternal(Revault.Internal.Transport.RuntimeOptions.GetRootAsRuntimeOptions(new ByteBuffer(bytes)).UnPack());
    private static Variable FromInternal(Revault.Internal.Transport.VariableT value) =>
        new(value.Name ?? string.Empty, value.Sensitivity ?? string.Empty);
    internal static Variable Variable(byte[] bytes) => FromInternal(Revault.Internal.Transport.Variable.GetRootAsVariable(new ByteBuffer(bytes)).UnPack());
    private static OwnerInspection FromInternal(Revault.Internal.Transport.OwnerInspectionT value) =>
        new(value.Signed, value.Fingerprint ?? string.Empty, value.HasFingerprint);
    internal static OwnerInspection OwnerInspection(byte[] bytes) => FromInternal(Revault.Internal.Transport.OwnerInspection.GetRootAsOwnerInspection(new ByteBuffer(bytes)).UnPack());
    private static Contact FromInternal(Revault.Internal.Transport.ContactT value) =>
        new(value.Name ?? string.Empty, value.Key?.ToArray() ?? Array.Empty<byte>());
    internal static Contact Contact(byte[] bytes) => FromInternal(Revault.Internal.Transport.Contact.GetRootAsContact(new ByteBuffer(bytes)).UnPack());
    private static AgentEntry FromInternal(Revault.Internal.Transport.AgentEntryT value) =>
        new(value.Id ?? string.Empty, value.Path ?? string.Empty);
    internal static AgentEntry AgentEntry(byte[] bytes) => FromInternal(Revault.Internal.Transport.AgentEntry.GetRootAsAgentEntry(new ByteBuffer(bytes)).UnPack());
    private static SleepSupport FromInternal(Revault.Internal.Transport.SleepSupportT value) =>
        new(value.SuspendNotifications, value.SleepInhibition, value.Supported);
    internal static SleepSupport SleepSupport(byte[] bytes) => FromInternal(Revault.Internal.Transport.SleepSupport.GetRootAsSleepSupport(new ByteBuffer(bytes)).UnPack());
    private static PlatformStatus FromInternal(Revault.Internal.Transport.PlatformStatusT value) =>
        new(value.Supported, value.Disabled, value.Scope ?? string.Empty, value.Backend ?? string.Empty, value.Item ?? string.Empty);
    internal static PlatformStatus PlatformStatus(byte[] bytes) => FromInternal(Revault.Internal.Transport.PlatformStatus.GetRootAsPlatformStatus(new ByteBuffer(bytes)).UnPack());
    private static VaultBackupManifest FromInternal(Revault.Internal.Transport.VaultBackupManifestT value) =>
        new(value.FormatVersion, value.CreatedAtUnixMs, value.VaultFileName ?? string.Empty, value.VaultSize, value.VaultSha256 ?? string.Empty);
    internal static VaultBackupManifest VaultBackupManifest(byte[] bytes) => FromInternal(Revault.Internal.Transport.VaultBackupManifest.GetRootAsVaultBackupManifest(new ByteBuffer(bytes)).UnPack());
    private static ErrorDetails FromInternal(Revault.Internal.Transport.ErrorDetailsT value) =>
        new(value.Category ?? string.Empty, value.ArtifactKind ?? string.Empty, value.FoundVersion, value.SupportedVersion, value.Message ?? string.Empty, value.Guidance ?? string.Empty);
    internal static ErrorDetails ErrorDetails(byte[] bytes) => FromInternal(Revault.Internal.Transport.ErrorDetails.GetRootAsErrorDetails(new ByteBuffer(bytes)).UnPack());
    internal static IReadOnlyList<StreamChunk> StreamChunkList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.StreamChunkList.GetRootAsStreamChunkList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<StreamChunk>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<PageInspection> PageInspectionList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.PageInspectionList.GetRootAsPageInspectionList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<PageInspection>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<LockboxEntry> LockboxEntryList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.LockboxEntryList.GetRootAsLockboxEntryList(new ByteBuffer(bytes)).UnPack().Entries;
        return values is null ? Array.Empty<LockboxEntry>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<Variable> VariableList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.VariableList.GetRootAsVariableList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<Variable>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<KeySlot> KeySlotList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.KeySlotList.GetRootAsKeySlotList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<KeySlot>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<FormDefinition> FormDefinitionList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.FormDefinitionList.GetRootAsFormDefinitionList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<FormDefinition>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<FormRecord> FormRecordList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.FormRecordList.GetRootAsFormRecordList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<FormRecord>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<Contact> ContactList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.ContactList.GetRootAsContactList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<Contact>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<KnownLockbox> KnownLockboxList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.KnownLockboxList.GetRootAsKnownLockboxList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<KnownLockbox>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<AccessSlotLabel> AccessSlotLabelList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.AccessSlotLabelList.GetRootAsAccessSlotLabelList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<AccessSlotLabel>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<AgentEntry> AgentEntryList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.AgentEntryList.GetRootAsAgentEntryList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<AgentEntry>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<ProfileHistory> ProfileHistoryList(byte[] bytes)
    {
        var values = Revault.Internal.Transport.ProfileHistoryList.GetRootAsProfileHistoryList(new ByteBuffer(bytes)).UnPack().Values;
        return values is null ? Array.Empty<ProfileHistory>() : values.Select(FromInternal).ToArray();
    }
    internal static IReadOnlyList<string> StringList(byte[] bytes) => Revault.Internal.Transport.StringList.GetRootAsStringList(new ByteBuffer(bytes)).UnPack().Values?.ToArray() ?? Array.Empty<string>();
    internal static LockboxEntry? OptionalLockboxEntry(byte[] bytes) { var value = Revault.Internal.Transport.OptionalLockboxEntry.GetRootAsOptionalLockboxEntry(new ByteBuffer(bytes)).UnPack().Value; return value is null ? null : FromInternal(value); }
    internal static FormRecord? OptionalFormRecord(byte[] bytes) { var value = Revault.Internal.Transport.OptionalFormRecord.GetRootAsOptionalFormRecord(new ByteBuffer(bytes)).UnPack().Value; return value is null ? null : FromInternal(value); }
    internal static FormValue? OptionalFormValue(byte[] bytes) { var value = Revault.Internal.Transport.OptionalFormValue.GetRootAsOptionalFormValue(new ByteBuffer(bytes)).UnPack().Value; return value is null ? null : FromInternal(value); }
    internal static string? OptionalString(byte[] bytes) { var value = Revault.Internal.Transport.OptionalString.GetRootAsOptionalString(new ByteBuffer(bytes)).UnPack(); return value.Present ? value.Value : null; }
    internal static byte[] EncodePathMoves(IReadOnlyList<PathMove> values) { var builder = new FlatBufferBuilder(256); var transport = new Revault.Internal.Transport.PathMoveListT { Values = values.Select(value => new Revault.Internal.Transport.PathMoveT { Source = value.Source, Destination = value.Destination }).ToList() }; var root = Revault.Internal.Transport.PathMoveList.Pack(builder, transport); builder.Finish(root.Value); return builder.SizedByteArray(); }
    internal static byte[] EncodeFormFields(IReadOnlyList<FormField> values) { var builder = new FlatBufferBuilder(256); var transport = new Revault.Internal.Transport.FormFieldListT { Values = values.Select(value => new Revault.Internal.Transport.FormFieldT { Id = value.Id, Label = value.Label, Kind = value.Kind, Required = value.Required }).ToList() }; var root = Revault.Internal.Transport.FormFieldList.Pack(builder, transport); builder.Finish(root.Value); return builder.SizedByteArray(); }
}
