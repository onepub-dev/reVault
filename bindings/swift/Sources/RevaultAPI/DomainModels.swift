import FlatBuffers
import Foundation

/// Identifies the filesystem object stored at a lockbox path.
public enum LockboxEntryKind: Int { case unspecified, file, symlink, directory }

/// Metadata for one file, directory, or symbolic link stored at a lockbox path.
public struct LockboxEntry {
    /// Absolute lockbox path of the stored entry.
    public let path: String
    /// Filesystem kind: file, directory, or symbolic link.
    public let kind: LockboxEntryKind
    /// Logical file length in bytes; zero for directories.
    public let length: UInt64
    /// Portable Unix permission bits stored with the entry.
    public let permissions: UInt32
    /// Creates an application-owned LockboxEntry value.
    public init(path: String, kind: LockboxEntryKind, length: UInt64, permissions: UInt32) {
        self.path = path
        self.kind = kind
        self.length = length
        self.permissions = permissions
    }
}

/// A source and destination pair used to rename a variable or form record atomically.
public struct PathMove {
    /// Existing variable name or form-record path to rename.
    public let source: String
    /// New variable name or form-record path.
    public let destination: String
    /// Creates an application-owned PathMove value.
    public init(source: String, destination: String) {
        self.source = source
        self.destination = destination
    }
}

/// One named input in a reusable form definition, including its display label and sensitivity kind.
public struct FormField {
    /// Stable field identifier used when reading and writing records.
    public let id: String
    /// Human-readable label presented to a person entering data.
    public let label: String
    /// Field kind that determines validation and secret handling.
    public let kind: String
    /// Whether a record must provide a value for this field.
    public let required: Bool
    /// Creates an application-owned FormField value.
    public init(id: String, label: String, kind: String, required: Bool) {
        self.id = id
        self.label = label
        self.kind = kind
        self.required = required
    }
}

/// A versioned form schema used to validate and label structured records in a lockbox.
public struct FormDefinition {
    /// Stable identifier shared by every revision of this form type.
    public let typeId: String
    /// Short name used to resolve the current form revision.
    public let alias: String
    /// Monotonically increasing revision number.
    public let revision: UInt32
    /// Human-readable name shown for this form.
    public let name: String
    /// Explanation shown to people completing the form.
    public let description: String
    /// Ordered inputs accepted by this form revision.
    public let fields: [FormField]
    /// Creates an application-owned FormDefinition value.
    public init(typeId: String, alias: String, revision: UInt32, name: String, description: String, fields: [FormField]) {
        self.typeId = typeId
        self.alias = alias
        self.revision = revision
        self.name = name
        self.description = description
        self.fields = fields
    }
}

/// The current value and sensitivity metadata for one field in a stored form record.
public struct FormValue {
    /// Identifier of the form field that owns this value.
    public let fieldId: String
    /// Display label captured from the form revision.
    public let label: String
    /// Field kind captured from the form revision.
    public let kind: String
    /// Plain value, or an empty string when the field is secret.
    public let value: String
    /// Whether the value must be read through a scoped secret callback.
    public let secret: Bool
    /// Creates an application-owned FormValue value.
    public init(fieldId: String, label: String, kind: String, value: String, secret: Bool) {
        self.fieldId = fieldId
        self.label = label
        self.kind = kind
        self.value = value
        self.secret = secret
    }
}

/// A named structured record stored at a lockbox path and tied to a form-definition revision.
public struct FormRecord {
    /// Absolute lockbox path that identifies the record.
    public let path: String
    /// Human-readable name assigned to this record.
    public let name: String
    /// Stable identifier of the record's form type.
    public let typeId: String
    /// Alias of the form definition used by the record.
    public let definitionAlias: String
    /// Exact form revision against which the record was created.
    public let definitionRevision: UInt32
    /// Ordered non-secret field metadata and values.
    public let values: [FormValue]
    /// Creates an application-owned FormRecord value.
    public init(path: String, name: String, typeId: String, definitionAlias: String, definitionRevision: UInt32, values: [FormValue]) {
        self.path = path
        self.name = name
        self.typeId = typeId
        self.definitionAlias = definitionAlias
        self.definitionRevision = definitionRevision
        self.values = values
    }
}

/// The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox.
public struct RecoveryReport {
    /// Files whose complete contents remain recoverable.
    public let intactFiles: [LockboxEntry]
    /// Number of completely recoverable files.
    public let intactFileCount: UInt64
    /// Number of files for which only some content is recoverable.
    public let partialFiles: UInt64
    /// Number of encrypted records that failed validation.
    public let corruptRecords: UInt64
    /// Whether a usable table of contents was recovered.
    public let tocRecovered: Bool
    /// Whether variable metadata was recovered.
    public let variablesRecovered: Bool
    /// Number of recovered variables.
    public let variableCount: UInt64
    /// Whether form definitions and records were recovered.
    public let formsRecovered: Bool
    /// Number of recovered form definitions.
    public let formDefinitionCount: UInt64
    /// Number of recovered form records.
    public let formRecordCount: UInt64
    /// Creates an application-owned RecoveryReport value.
    public init(intactFiles: [LockboxEntry], intactFileCount: UInt64, partialFiles: UInt64, corruptRecords: UInt64, tocRecovered: Bool, variablesRecovered: Bool, variableCount: UInt64, formsRecovered: Bool, formDefinitionCount: UInt64, formRecordCount: UInt64) {
        self.intactFiles = intactFiles
        self.intactFileCount = intactFileCount
        self.partialFiles = partialFiles
        self.corruptRecords = corruptRecords
        self.tocRecovered = tocRecovered
        self.variablesRecovered = variablesRecovered
        self.variableCount = variableCount
        self.formsRecovered = formsRecovered
        self.formDefinitionCount = formDefinitionCount
        self.formRecordCount = formRecordCount
    }
}

/// One password or contact credential that can unlock a lockbox content key.
public struct KeySlot {
    /// Stable slot identifier used when removing this access method.
    public let id: UInt64
    /// Access method, such as password or contact key.
    public let protection: String
    /// Cryptographic algorithm protecting the content key.
    public let algorithm: String
    /// Creates an application-owned KeySlot value.
    public init(id: UInt64, protection: String, algorithm: String) {
        self.id = id
        self.protection = protection
        self.algorithm = algorithm
    }
}

/// Current capacity, occupancy, hit, and miss counters for an open lockbox cache.
public struct CacheStats {
    /// Maximum decoded-page memory permitted for the cache.
    public let limitBytes: UInt64
    /// Decoded-page memory currently held by the cache.
    public let usedBytes: UInt64
    /// Number of decoded pages currently cached.
    public let entries: UInt64
    /// Reads served by an already decoded page.
    public let hits: UInt64
    /// Reads that required decoding another page.
    public let misses: UInt64
    /// Creates an application-owned CacheStats value.
    public init(limitBytes: UInt64, usedBytes: UInt64, entries: UInt64, hits: UInt64, misses: UInt64) {
        self.limitBytes = limitBytes
        self.usedBytes = usedBytes
        self.entries = entries
        self.hits = hits
        self.misses = misses
    }
}

/// Time spent reading host files and preparing encrypted pages during the latest import work.
public struct ImportStats {
    /// Nanoseconds spent reading host filesystem metadata, as decimal text.
    public let hostStatNanos: String
    /// Nanoseconds spent reading host file content, as decimal text.
    public let hostReadNanos: String
    /// Nanoseconds spent preparing encrypted records, as decimal text.
    public let framePrepareNanos: String
    /// Nanoseconds spent writing encrypted pages, as decimal text.
    public let pageWriteNanos: String
    /// Creates an application-owned ImportStats value.
    public init(hostStatNanos: String, hostReadNanos: String, framePrepareNanos: String, pageWriteNanos: String) {
        self.hostStatNanos = hostStatNanos
        self.hostReadNanos = hostReadNanos
        self.framePrepareNanos = framePrepareNanos
        self.pageWriteNanos = pageWriteNanos
    }
}

/// One logical object recorded inside an inspected encrypted lockbox page.
public struct PageObject {
    /// Object identifier recorded in the encrypted page.
    public let id: UInt64
    /// Kind of logical object stored in the page.
    public let kind: String
    /// Encrypted object payload length in bytes.
    public let payloadLen: UInt64
    /// Creates an application-owned PageObject value.
    public init(id: UInt64, kind: String, payloadLen: UInt64) {
        self.id = id
        self.kind = kind
        self.payloadLen = payloadLen
    }
}

/// Layout and utilization details for one encrypted page in a lockbox archive.
public struct PageInspection {
    /// Byte offset at which the page begins in the archive.
    public let offset: UInt64
    /// Identifier stored in the page header.
    public let pageId: UInt64
    /// Commit sequence that wrote this page.
    public let sequence: UInt64
    /// Total encoded page size in bytes.
    public let pageSize: UInt64
    /// Encrypted body length in bytes.
    public let encryptedBodyLen: UInt64
    /// Unused capacity remaining in the page.
    public let unusedBytes: UInt64
    /// Number of logical objects stored in the page.
    public let objectCount: UInt64
    /// Logical objects discovered in the page.
    public let objects: [PageObject]
    /// Creates an application-owned PageInspection value.
    public init(offset: UInt64, pageId: UInt64, sequence: UInt64, pageSize: UInt64, encryptedBodyLen: UInt64, unusedBytes: UInt64, objectCount: UInt64, objects: [PageObject]) {
        self.offset = offset
        self.pageId = pageId
        self.sequence = sequence
        self.pageSize = pageSize
        self.encryptedBodyLen = encryptedBodyLen
        self.unusedBytes = unusedBytes
        self.objectCount = objectCount
        self.objects = objects
    }
}

/// Header, owner-signature, and key-slot information read from a lockbox file without opening its contents.
public struct FileInspection {
    /// Stable binary identifier read from the lockbox header.
    public let lockboxId: [UInt8]
    /// Whether the archive header passed structural validation.
    public let headerReadable: Bool
    /// Latest readable access-key directory generation.
    public let keyDirectoryGeneration: UInt64
    /// Number of readable redundant key-directory copies.
    public let keyDirectoryCopyCount: UInt64
    /// Whether commits require an owner signature.
    public let ownerSigned: Bool
    /// Password and contact access methods found in the header.
    public let keySlots: [KeySlot]
    /// Creates an application-owned FileInspection value.
    public init(lockboxId: [UInt8], headerReadable: Bool, keyDirectoryGeneration: UInt64, keyDirectoryCopyCount: UInt64, ownerSigned: Bool, keySlots: [KeySlot]) {
        self.lockboxId = lockboxId
        self.headerReadable = headerReadable
        self.keyDirectoryGeneration = keyDirectoryGeneration
        self.keyDirectoryCopyCount = keyDirectoryCopyCount
        self.ownerSigned = ownerSigned
        self.keySlots = keySlots
    }
}

/// One active or retired generation of the contact keys belonging to a named vault profile.
public struct ProfileGeneration {
    /// Generation number used to address this key version.
    public let index: UInt32
    /// Lifecycle state, such as active or retired.
    public let status: String
    /// Fingerprint of this generation's contact public key.
    public let contactFingerprint: [UInt8]
    /// Creation time in Unix milliseconds.
    public let createdAtUnixMs: UInt64
    /// Retirement time in Unix milliseconds when retired.
    public let retiredAtUnixMs: UInt64
    /// Whether a retirement time is present.
    public let hasRetiredAt: Bool
    /// Creates an application-owned ProfileGeneration value.
    public init(index: UInt32, status: String, contactFingerprint: [UInt8], createdAtUnixMs: UInt64, retiredAtUnixMs: UInt64, hasRetiredAt: Bool) {
        self.index = index
        self.status = status
        self.contactFingerprint = contactFingerprint
        self.createdAtUnixMs = createdAtUnixMs
        self.retiredAtUnixMs = retiredAtUnixMs
        self.hasRetiredAt = hasRetiredAt
    }
}

/// The active generation and rotation history for a named vault profile.
public struct ProfileHistory {
    /// Vault profile name whose generations are listed.
    public let name: String
    /// Generation number currently used for new access grants.
    public let activeGeneration: UInt32
    /// Active and retired contact-key generations.
    public let generations: [ProfileGeneration]
    /// Creates an application-owned ProfileHistory value.
    public init(name: String, activeGeneration: UInt32, generations: [ProfileGeneration]) {
        self.name = name
        self.activeGeneration = activeGeneration
        self.generations = generations
    }
}

/// A lockbox identifier and host path remembered by the local vault for later discovery.
public struct KnownLockbox {
    /// Stable binary identifier of the remembered lockbox.
    public let lockboxId: [UInt8]
    /// Last known host filesystem path of the lockbox.
    public let path: String
    /// Most recent observation time in Unix milliseconds.
    public let lastSeenUnixMs: UInt64
    /// Creates an application-owned KnownLockbox value.
    public init(lockboxId: [UInt8], path: String, lastSeenUnixMs: UInt64) {
        self.lockboxId = lockboxId
        self.path = path
        self.lastSeenUnixMs = lastSeenUnixMs
    }
}

/// A local human-readable label attached to one lockbox access slot.
public struct AccessSlotLabel {
    /// Lockbox whose access slot is labelled.
    public let lockboxId: [UInt8]
    /// Stable identifier of the labelled access slot.
    public let slotId: UInt64
    /// Local human-readable label for the access slot.
    public let name: String
    /// Last label update time in Unix milliseconds.
    public let updatedAtUnixMs: UInt64
    /// Creates an application-owned AccessSlotLabel value.
    public init(lockboxId: [UInt8], slotId: UInt64, name: String, updatedAtUnixMs: UInt64) {
        self.lockboxId = lockboxId
        self.slotId = slotId
        self.name = name
        self.updatedAtUnixMs = updatedAtUnixMs
    }
}

/// A logical or physical byte range emitted while walking the contents of a lockbox.
public struct StreamChunk {
    /// Lockbox file path to which this byte range belongs.
    public let path: String
    /// Logical byte offset within the file.
    public let fileOffset: UInt64
    /// Logical range length in bytes.
    public let length: UInt64
    /// Archive byte offset, when physical streaming is requested.
    public let physicalOffset: UInt64
    /// Whether the range represents a sparse zero-filled extent.
    public let sparse: Bool
    /// File bytes for a populated logical range.
    public let data: [UInt8]
    /// Creates an application-owned StreamChunk value.
    public init(path: String, fileOffset: UInt64, length: UInt64, physicalOffset: UInt64, sparse: Bool, data: [UInt8]) {
        self.path = path
        self.fileOffset = fileOffset
        self.length = length
        self.physicalOffset = physicalOffset
        self.sparse = sparse
        self.data = data
    }
}

/// The workload and worker policies currently applied to an open lockbox.
public struct RuntimeOptions {
    /// I/O workload policy used to tune page access.
    public let workloadProfile: String
    /// Worker scheduling policy and effective parallelism.
    public let workerPolicy: String
    /// Creates an application-owned RuntimeOptions value.
    public init(workloadProfile: String, workerPolicy: String) {
        self.workloadProfile = workloadProfile
        self.workerPolicy = workerPolicy
    }
}

/// The name and sensitivity classification of a variable stored in a lockbox.
public struct Variable {
    /// Name used to address the variable in the lockbox.
    public let name: String
    /// Whether the value is ordinary text or a protected secret.
    public let sensitivity: String
    /// Creates an application-owned Variable value.
    public init(name: String, sensitivity: String) {
        self.name = name
        self.sensitivity = sensitivity
    }
}

/// Whether a lockbox is owner-signed and, when available, the signing-key fingerprint.
public struct OwnerInspection {
    /// Whether the lockbox requires owner-signed commits.
    public let signed: Bool
    /// Owner signing-key fingerprint when one is configured.
    public let fingerprint: String
    /// Whether an owner fingerprint is available.
    public let hasFingerprint: Bool
    /// Creates an application-owned OwnerInspection value.
    public init(signed: Bool, fingerprint: String, hasFingerprint: Bool) {
        self.signed = signed
        self.fingerprint = fingerprint
        self.hasFingerprint = hasFingerprint
    }
}

/// A named recipient public key stored in the local vault address book.
public struct Contact {
    /// Local address-book name of the contact.
    public let name: String
    /// Serialized contact public key used to grant lockbox access.
    public let key: [UInt8]
    /// Creates an application-owned Contact value.
    public init(name: String, key: [UInt8]) {
        self.name = name
        self.key = key
    }
}

/// A lockbox key currently held by the local session agent, identified by lockbox and path.
public struct AgentEntry {
    /// Stable lockbox identifier for the cached key.
    public let id: String
    /// Host path associated with the cached lockbox key.
    public let path: String
    /// Creates an application-owned AgentEntry value.
    public init(id: String, path: String) {
        self.id = id
        self.path = path
    }
}

/// The host capabilities used to protect cached secrets across suspend and sleep.
public struct SleepSupport {
    /// Whether the host reports impending system suspend.
    public let suspendNotifications: Bool
    /// Whether the agent can delay sleep while handling secrets.
    public let sleepInhibition: Bool
    /// Whether the host supplies enough integration for safe caching.
    public let supported: Bool
    /// Creates an application-owned SleepSupport value.
    public init(suspendNotifications: Bool, sleepInhibition: Bool, supported: Bool) {
        self.suspendNotifications = suspendNotifications
        self.sleepInhibition = sleepInhibition
        self.supported = supported
    }
}

/// Availability and configuration of the operating-system credential store used for the vault password.
public struct PlatformStatus {
    /// Whether a usable operating-system credential store exists.
    public let supported: Bool
    /// Whether the user disabled credential-store integration.
    public let disabled: Bool
    /// Application-specific scope used to isolate the stored password.
    public let scope: String
    /// Operating-system credential-store backend in use.
    public let backend: String
    /// Credential item name used by the backend.
    public let item: String
    /// Creates an application-owned PlatformStatus value.
    public init(supported: Bool, disabled: Bool, scope: String, backend: String, item: String) {
        self.supported = supported
        self.disabled = disabled
        self.scope = scope
        self.backend = backend
        self.item = item
    }
}

/// The version, size, checksum, and creation time of an exported local-vault backup.
public struct VaultBackupManifest {
    /// Backup container format version.
    public let formatVersion: UInt32
    /// Backup creation time in Unix milliseconds.
    public let createdAtUnixMs: UInt64
    /// Metadata-vault filename stored in the backup.
    public let vaultFileName: String
    /// Encrypted vault payload size in bytes.
    public let vaultSize: UInt64
    /// Lowercase SHA-256 digest of the encrypted vault payload.
    public let vaultSha256: String
    /// Creates an application-owned VaultBackupManifest value.
    public init(formatVersion: UInt32, createdAtUnixMs: UInt64, vaultFileName: String, vaultSize: UInt64, vaultSha256: String) {
        self.formatVersion = formatVersion
        self.createdAtUnixMs = createdAtUnixMs
        self.vaultFileName = vaultFileName
        self.vaultSize = vaultSize
        self.vaultSha256 = vaultSha256
    }
}

/// Structured category, version, guidance, and artifact context for the most recent native failure.
public struct ErrorDetails {
    /// Stable error category suitable for programmatic handling.
    public let category: String
    /// Kind of archive or vault artifact involved in the failure.
    public let artifactKind: String
    /// Format version read from the failing artifact.
    public let foundVersion: UInt32
    /// Newest format version supported by this library.
    public let supportedVersion: UInt32
    /// Human-readable explanation of the failure.
    public let message: String
    /// Suggested corrective action for the caller or user.
    public let guidance: String
    /// Creates an application-owned ErrorDetails value.
    public init(category: String, artifactKind: String, foundVersion: UInt32, supportedVersion: UInt32, message: String, guidance: String) {
        self.category = category
        self.artifactKind = artifactKind
        self.foundVersion = foundVersion
        self.supportedVersion = supportedVersion
        self.message = message
        self.guidance = guidance
    }
}

enum DomainCodec {
    private static func convert(_ value: revault_internal__LockboxEntryT) -> LockboxEntry {
        LockboxEntry(path: value.path ?? "", kind: LockboxEntryKind(rawValue: Int(value.kind.rawValue)) ?? .unspecified, length: value.length, permissions: value.permissions)
    }
    static func lockboxEntry(_ data: Data) -> LockboxEntry {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__LockboxEntry = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__PathMoveT) -> PathMove {
        PathMove(source: value.source ?? "", destination: value.destination ?? "")
    }
    static func pathMove(_ data: Data) -> PathMove {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__PathMove = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__FormFieldT) -> FormField {
        FormField(id: value.id ?? "", label: value.label ?? "", kind: value.kind ?? "", required: value.required)
    }
    static func formField(_ data: Data) -> FormField {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FormField = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__FormDefinitionT) -> FormDefinition {
        FormDefinition(typeId: value.typeId ?? "", alias: value.alias ?? "", revision: value.revision, name: value.name ?? "", description: value.description ?? "", fields: (value.fields ?? []).map(convert))
    }
    static func formDefinition(_ data: Data) -> FormDefinition {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FormDefinition = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__FormValueT) -> FormValue {
        FormValue(fieldId: value.fieldId ?? "", label: value.label ?? "", kind: value.kind ?? "", value: value.value ?? "", secret: value.secret)
    }
    static func formValue(_ data: Data) -> FormValue {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FormValue = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__FormRecordT) -> FormRecord {
        FormRecord(path: value.path ?? "", name: value.name ?? "", typeId: value.typeId ?? "", definitionAlias: value.definitionAlias ?? "", definitionRevision: value.definitionRevision, values: (value.values ?? []).map(convert))
    }
    static func formRecord(_ data: Data) -> FormRecord {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FormRecord = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__RecoveryReportT) -> RecoveryReport {
        RecoveryReport(intactFiles: (value.intactFiles ?? []).map(convert), intactFileCount: value.intactFileCount, partialFiles: value.partialFiles, corruptRecords: value.corruptRecords, tocRecovered: value.tocRecovered, variablesRecovered: value.variablesRecovered, variableCount: value.variableCount, formsRecovered: value.formsRecovered, formDefinitionCount: value.formDefinitionCount, formRecordCount: value.formRecordCount)
    }
    static func recoveryReport(_ data: Data) -> RecoveryReport {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__RecoveryReport = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__KeySlotT) -> KeySlot {
        KeySlot(id: value.id, protection: value.protection ?? "", algorithm: value.algorithm ?? "")
    }
    static func keySlot(_ data: Data) -> KeySlot {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__KeySlot = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__CacheStatsT) -> CacheStats {
        CacheStats(limitBytes: value.limitBytes, usedBytes: value.usedBytes, entries: value.entries, hits: value.hits, misses: value.misses)
    }
    static func cacheStats(_ data: Data) -> CacheStats {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__CacheStats = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__ImportStatsT) -> ImportStats {
        ImportStats(hostStatNanos: value.hostStatNanos ?? "", hostReadNanos: value.hostReadNanos ?? "", framePrepareNanos: value.framePrepareNanos ?? "", pageWriteNanos: value.pageWriteNanos ?? "")
    }
    static func importStats(_ data: Data) -> ImportStats {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__ImportStats = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__PageObjectT) -> PageObject {
        PageObject(id: value.id, kind: value.kind ?? "", payloadLen: value.payloadLen)
    }
    static func pageObject(_ data: Data) -> PageObject {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__PageObject = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__PageInspectionT) -> PageInspection {
        PageInspection(offset: value.offset, pageId: value.pageId, sequence: value.sequence, pageSize: value.pageSize, encryptedBodyLen: value.encryptedBodyLen, unusedBytes: value.unusedBytes, objectCount: value.objectCount, objects: (value.objects ?? []).map(convert))
    }
    static func pageInspection(_ data: Data) -> PageInspection {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__PageInspection = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__FileInspectionT) -> FileInspection {
        FileInspection(lockboxId: value.lockboxId ?? [], headerReadable: value.headerReadable, keyDirectoryGeneration: value.keyDirectoryGeneration, keyDirectoryCopyCount: value.keyDirectoryCopyCount, ownerSigned: value.ownerSigned, keySlots: (value.keySlots ?? []).map(convert))
    }
    static func fileInspection(_ data: Data) -> FileInspection {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FileInspection = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__ProfileGenerationT) -> ProfileGeneration {
        ProfileGeneration(index: value.index, status: value.status ?? "", contactFingerprint: value.contactFingerprint ?? [], createdAtUnixMs: value.createdAtUnixMs, retiredAtUnixMs: value.retiredAtUnixMs, hasRetiredAt: value.hasRetiredAt)
    }
    static func profileGeneration(_ data: Data) -> ProfileGeneration {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__ProfileGeneration = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__ProfileHistoryT) -> ProfileHistory {
        ProfileHistory(name: value.name ?? "", activeGeneration: value.activeGeneration, generations: (value.generations ?? []).map(convert))
    }
    static func profileHistory(_ data: Data) -> ProfileHistory {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__ProfileHistory = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__KnownLockboxT) -> KnownLockbox {
        KnownLockbox(lockboxId: value.lockboxId ?? [], path: value.path ?? "", lastSeenUnixMs: value.lastSeenUnixMs)
    }
    static func knownLockbox(_ data: Data) -> KnownLockbox {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__KnownLockbox = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__AccessSlotLabelT) -> AccessSlotLabel {
        AccessSlotLabel(lockboxId: value.lockboxId ?? [], slotId: value.slotId, name: value.name ?? "", updatedAtUnixMs: value.updatedAtUnixMs)
    }
    static func accessSlotLabel(_ data: Data) -> AccessSlotLabel {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__AccessSlotLabel = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__StreamChunkT) -> StreamChunk {
        StreamChunk(path: value.path ?? "", fileOffset: value.fileOffset, length: value.length, physicalOffset: value.physicalOffset, sparse: value.sparse, data: value.data ?? [])
    }
    static func streamChunk(_ data: Data) -> StreamChunk {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__StreamChunk = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__RuntimeOptionsT) -> RuntimeOptions {
        RuntimeOptions(workloadProfile: value.workloadProfile ?? "", workerPolicy: value.workerPolicy ?? "")
    }
    static func runtimeOptions(_ data: Data) -> RuntimeOptions {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__RuntimeOptions = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__VariableT) -> Variable {
        Variable(name: value.name ?? "", sensitivity: value.sensitivity ?? "")
    }
    static func variable(_ data: Data) -> Variable {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__Variable = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__OwnerInspectionT) -> OwnerInspection {
        OwnerInspection(signed: value.signed, fingerprint: value.fingerprint ?? "", hasFingerprint: value.hasFingerprint)
    }
    static func ownerInspection(_ data: Data) -> OwnerInspection {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OwnerInspection = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__ContactT) -> Contact {
        Contact(name: value.name ?? "", key: value.key ?? [])
    }
    static func contact(_ data: Data) -> Contact {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__Contact = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__AgentEntryT) -> AgentEntry {
        AgentEntry(id: value.id ?? "", path: value.path ?? "")
    }
    static func agentEntry(_ data: Data) -> AgentEntry {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__AgentEntry = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__SleepSupportT) -> SleepSupport {
        SleepSupport(suspendNotifications: value.suspendNotifications, sleepInhibition: value.sleepInhibition, supported: value.supported)
    }
    static func sleepSupport(_ data: Data) -> SleepSupport {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__SleepSupport = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__PlatformStatusT) -> PlatformStatus {
        PlatformStatus(supported: value.supported, disabled: value.disabled, scope: value.scope ?? "", backend: value.backend ?? "", item: value.item ?? "")
    }
    static func platformStatus(_ data: Data) -> PlatformStatus {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__PlatformStatus = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__VaultBackupManifestT) -> VaultBackupManifest {
        VaultBackupManifest(formatVersion: value.formatVersion, createdAtUnixMs: value.createdAtUnixMs, vaultFileName: value.vaultFileName ?? "", vaultSize: value.vaultSize, vaultSha256: value.vaultSha256 ?? "")
    }
    static func vaultBackupManifest(_ data: Data) -> VaultBackupManifest {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__VaultBackupManifest = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    private static func convert(_ value: revault_internal__ErrorDetailsT) -> ErrorDetails {
        ErrorDetails(category: value.category ?? "", artifactKind: value.artifactKind ?? "", foundVersion: value.foundVersion, supportedVersion: value.supportedVersion, message: value.message ?? "", guidance: value.guidance ?? "")
    }
    static func errorDetails(_ data: Data) -> ErrorDetails {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__ErrorDetails = getRoot(byteBuffer: &buffer); return convert(root.unpack())
    }
    static func streamChunkList(_ data: Data) -> [StreamChunk] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__StreamChunkList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func pageInspectionList(_ data: Data) -> [PageInspection] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__PageInspectionList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func lockboxEntryList(_ data: Data) -> [LockboxEntry] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__LockboxEntryList = getRoot(byteBuffer: &buffer); return (root.unpack().entries ?? []).map(convert)
    }
    static func variableList(_ data: Data) -> [Variable] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__VariableList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func keySlotList(_ data: Data) -> [KeySlot] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__KeySlotList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func formDefinitionList(_ data: Data) -> [FormDefinition] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FormDefinitionList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func formRecordList(_ data: Data) -> [FormRecord] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__FormRecordList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func contactList(_ data: Data) -> [Contact] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__ContactList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func knownLockboxList(_ data: Data) -> [KnownLockbox] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__KnownLockboxList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func accessSlotLabelList(_ data: Data) -> [AccessSlotLabel] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__AccessSlotLabelList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func agentEntryList(_ data: Data) -> [AgentEntry] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__AgentEntryList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func profileHistoryList(_ data: Data) -> [ProfileHistory] {
        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__ProfileHistoryList = getRoot(byteBuffer: &buffer); return (root.unpack().values ?? []).map(convert)
    }
    static func stringList(_ data: Data) -> [String] { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__StringList = getRoot(byteBuffer: &buffer); return root.unpack().values ?? [] }
    static func optionalString(_ data: Data) -> String? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalString = getRoot(byteBuffer: &buffer); let value = root.unpack(); return value.present ? value.value : nil }
    static func optionalLockboxEntry(_ data: Data) -> LockboxEntry? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalLockboxEntry = getRoot(byteBuffer: &buffer); return root.unpack().value.map(convert) }
    static func optionalFormRecord(_ data: Data) -> FormRecord? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalFormRecord = getRoot(byteBuffer: &buffer); return root.unpack().value.map(convert) }
    static func optionalFormValue(_ data: Data) -> FormValue? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalFormValue = getRoot(byteBuffer: &buffer); return root.unpack().value.map(convert) }
    static func encodePathMoves(_ values: [PathMove]) -> Data { var builder = FlatBufferBuilder(initialSize: 256); var offsets: [Offset] = []; for value in values { let source = builder.create(string: value.source); let destination = builder.create(string: value.destination); offsets.append(revault_internal__PathMove.createPathMove(&builder, sourceOffset: source, destinationOffset: destination)) }; let vector = builder.createVector(ofOffsets: offsets); let root = revault_internal__PathMoveList.createPathMoveList(&builder, valuesVectorOffset: vector); builder.finish(offset: root); return Data(builder.sizedByteArray) }
    static func encodeFormFields(_ values: [FormField]) -> Data { var builder = FlatBufferBuilder(initialSize: 256); var offsets: [Offset] = []; for value in values { let id = builder.create(string: value.id); let label = builder.create(string: value.label); let kind = builder.create(string: value.kind); offsets.append(revault_internal__FormField.createFormField(&builder, idOffset: id, labelOffset: label, kindOffset: kind, required: value.required)) }; let vector = builder.createVector(ofOffsets: offsets); let root = revault_internal__FormFieldList.createFormFieldList(&builder, valuesVectorOffset: vector); builder.finish(offset: root); return Data(builder.sizedByteArray) }
}
