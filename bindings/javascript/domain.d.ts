/** Public reVault domain values. Serialization is deliberately private. */
export type LockboxEntryKind = 0 | 1 | 2 | 3;

/** Metadata for one file, directory, or symbolic link stored at a lockbox path. */
export interface LockboxEntry {
  /** Absolute lockbox path of the stored entry. */
  readonly path: string;
  /** Filesystem kind: file, directory, or symbolic link. */
  readonly kind: LockboxEntryKind;
  /** Logical file length in bytes; zero for directories. */
  readonly length: bigint;
  /** Portable Unix permission bits stored with the entry. */
  readonly permissions: number;
}

/** A source and destination pair used to rename a variable or form record atomically. */
export interface PathMove {
  /** Existing variable name or form-record path to rename. */
  readonly source: string;
  /** New variable name or form-record path. */
  readonly destination: string;
}

/** One named input in a reusable form definition, including its display label and sensitivity kind. */
export interface FormField {
  /** Stable field identifier used when reading and writing records. */
  readonly id: string;
  /** Human-readable label presented to a person entering data. */
  readonly label: string;
  /** Field kind that determines validation and secret handling. */
  readonly kind: string;
  /** Whether a record must provide a value for this field. */
  readonly required: boolean;
}

/** A versioned form schema used to validate and label structured records in a lockbox. */
export interface FormDefinition {
  /** Stable identifier shared by every revision of this form type. */
  readonly typeId: string;
  /** Short name used to resolve the current form revision. */
  readonly alias: string;
  /** Monotonically increasing revision number. */
  readonly revision: number;
  /** Human-readable name shown for this form. */
  readonly name: string;
  /** Explanation shown to people completing the form. */
  readonly description: string;
  /** Ordered inputs accepted by this form revision. */
  readonly fields: readonly FormField[];
}

/** The current value and sensitivity metadata for one field in a stored form record. */
export interface FormValue {
  /** Identifier of the form field that owns this value. */
  readonly fieldId: string;
  /** Display label captured from the form revision. */
  readonly label: string;
  /** Field kind captured from the form revision. */
  readonly kind: string;
  /** Plain value, or an empty string when the field is secret. */
  readonly value: string;
  /** Whether the value must be read through a scoped secret callback. */
  readonly secret: boolean;
}

/** A named structured record stored at a lockbox path and tied to a form-definition revision. */
export interface FormRecord {
  /** Absolute lockbox path that identifies the record. */
  readonly path: string;
  /** Human-readable name assigned to this record. */
  readonly name: string;
  /** Stable identifier of the record's form type. */
  readonly typeId: string;
  /** Alias of the form definition used by the record. */
  readonly definitionAlias: string;
  /** Exact form revision against which the record was created. */
  readonly definitionRevision: number;
  /** Ordered non-secret field metadata and values. */
  readonly values: readonly FormValue[];
}

/** The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox. */
export interface RecoveryReport {
  /** Files whose complete contents remain recoverable. */
  readonly intactFiles: readonly LockboxEntry[];
  /** Number of completely recoverable files. */
  readonly intactFileCount: bigint;
  /** Number of files for which only some content is recoverable. */
  readonly partialFiles: bigint;
  /** Number of encrypted records that failed validation. */
  readonly corruptRecords: bigint;
  /** Whether a usable table of contents was recovered. */
  readonly tocRecovered: boolean;
  /** Whether variable metadata was recovered. */
  readonly variablesRecovered: boolean;
  /** Number of recovered variables. */
  readonly variableCount: bigint;
  /** Whether form definitions and records were recovered. */
  readonly formsRecovered: boolean;
  /** Number of recovered form definitions. */
  readonly formDefinitionCount: bigint;
  /** Number of recovered form records. */
  readonly formRecordCount: bigint;
}

/** One password or contact credential that can unlock a lockbox content key. */
export interface KeySlot {
  /** Stable slot identifier used when removing this access method. */
  readonly id: bigint;
  /** Access method, such as password or contact key. */
  readonly protection: string;
  /** Cryptographic algorithm protecting the content key. */
  readonly algorithm: string;
}

/** Current capacity, occupancy, hit, and miss counters for an open lockbox cache. */
export interface CacheStats {
  /** Maximum decoded-page memory permitted for the cache. */
  readonly limitBytes: bigint;
  /** Decoded-page memory currently held by the cache. */
  readonly usedBytes: bigint;
  /** Number of decoded pages currently cached. */
  readonly entries: bigint;
  /** Reads served by an already decoded page. */
  readonly hits: bigint;
  /** Reads that required decoding another page. */
  readonly misses: bigint;
}

/** Time spent reading host files and preparing encrypted pages during the latest import work. */
export interface ImportStats {
  /** Nanoseconds spent reading host filesystem metadata, as decimal text. */
  readonly hostStatNanos: string;
  /** Nanoseconds spent reading host file content, as decimal text. */
  readonly hostReadNanos: string;
  /** Nanoseconds spent preparing encrypted records, as decimal text. */
  readonly framePrepareNanos: string;
  /** Nanoseconds spent writing encrypted pages, as decimal text. */
  readonly pageWriteNanos: string;
}

/** One logical object recorded inside an inspected encrypted lockbox page. */
export interface PageObject {
  /** Object identifier recorded in the encrypted page. */
  readonly id: bigint;
  /** Kind of logical object stored in the page. */
  readonly kind: string;
  /** Encrypted object payload length in bytes. */
  readonly payloadLen: bigint;
}

/** Layout and utilization details for one encrypted page in a lockbox archive. */
export interface PageInspection {
  /** Byte offset at which the page begins in the archive. */
  readonly offset: bigint;
  /** Identifier stored in the page header. */
  readonly pageId: bigint;
  /** Commit sequence that wrote this page. */
  readonly sequence: bigint;
  /** Total encoded page size in bytes. */
  readonly pageSize: bigint;
  /** Encrypted body length in bytes. */
  readonly encryptedBodyLen: bigint;
  /** Unused capacity remaining in the page. */
  readonly unusedBytes: bigint;
  /** Number of logical objects stored in the page. */
  readonly objectCount: bigint;
  /** Logical objects discovered in the page. */
  readonly objects: readonly PageObject[];
}

/** Header, owner-signature, and key-slot information read from a lockbox file without opening its contents. */
export interface FileInspection {
  /** Stable binary identifier read from the lockbox header. */
  readonly lockboxId: Uint8Array;
  /** Whether the archive header passed structural validation. */
  readonly headerReadable: boolean;
  /** Latest readable access-key directory generation. */
  readonly keyDirectoryGeneration: bigint;
  /** Number of readable redundant key-directory copies. */
  readonly keyDirectoryCopyCount: bigint;
  /** Whether commits require an owner signature. */
  readonly ownerSigned: boolean;
  /** Password and contact access methods found in the header. */
  readonly keySlots: readonly KeySlot[];
}

/** One active or retired generation of the contact keys belonging to a named vault profile. */
export interface ProfileGeneration {
  /** Generation number used to address this key version. */
  readonly index: number;
  /** Lifecycle state, such as active or retired. */
  readonly status: string;
  /** Fingerprint of this generation's contact public key. */
  readonly contactFingerprint: Uint8Array;
  /** Creation time in Unix milliseconds. */
  readonly createdAtUnixMs: bigint;
  /** Retirement time in Unix milliseconds when retired. */
  readonly retiredAtUnixMs: bigint;
  /** Whether a retirement time is present. */
  readonly hasRetiredAt: boolean;
}

/** The active generation and rotation history for a named vault profile. */
export interface ProfileHistory {
  /** Vault profile name whose generations are listed. */
  readonly name: string;
  /** Generation number currently used for new access grants. */
  readonly activeGeneration: number;
  /** Active and retired contact-key generations. */
  readonly generations: readonly ProfileGeneration[];
}

/** A lockbox identifier and host path remembered by the local vault for later discovery. */
export interface KnownLockbox {
  /** Stable binary identifier of the remembered lockbox. */
  readonly lockboxId: Uint8Array;
  /** Last known host filesystem path of the lockbox. */
  readonly path: string;
  /** Most recent observation time in Unix milliseconds. */
  readonly lastSeenUnixMs: bigint;
}

/** A local human-readable label attached to one lockbox access slot. */
export interface AccessSlotLabel {
  /** Lockbox whose access slot is labelled. */
  readonly lockboxId: Uint8Array;
  /** Stable identifier of the labelled access slot. */
  readonly slotId: bigint;
  /** Local human-readable label for the access slot. */
  readonly name: string;
  /** Last label update time in Unix milliseconds. */
  readonly updatedAtUnixMs: bigint;
}

/** A logical or physical byte range emitted while walking the contents of a lockbox. */
export interface StreamChunk {
  /** Lockbox file path to which this byte range belongs. */
  readonly path: string;
  /** Logical byte offset within the file. */
  readonly fileOffset: bigint;
  /** Logical range length in bytes. */
  readonly length: bigint;
  /** Archive byte offset, when physical streaming is requested. */
  readonly physicalOffset: bigint;
  /** Whether the range represents a sparse zero-filled extent. */
  readonly sparse: boolean;
  /** File bytes for a populated logical range. */
  readonly data: Uint8Array;
}

/** The workload and worker policies currently applied to an open lockbox. */
export interface RuntimeOptions {
  /** I/O workload policy used to tune page access. */
  readonly workloadProfile: string;
  /** Worker scheduling policy and effective parallelism. */
  readonly workerPolicy: string;
}

/** The name and sensitivity classification of a variable stored in a lockbox. */
export interface Variable {
  /** Name used to address the variable in the lockbox. */
  readonly name: string;
  /** Whether the value is ordinary text or a protected secret. */
  readonly sensitivity: string;
}

/** Whether a lockbox is owner-signed and, when available, the signing-key fingerprint. */
export interface OwnerInspection {
  /** Whether the lockbox requires owner-signed commits. */
  readonly signed: boolean;
  /** Owner signing-key fingerprint when one is configured. */
  readonly fingerprint: string;
  /** Whether an owner fingerprint is available. */
  readonly hasFingerprint: boolean;
}

/** A named recipient public key stored in the local vault address book. */
export interface Contact {
  /** Local address-book name of the contact. */
  readonly name: string;
  /** Serialized contact public key used to grant lockbox access. */
  readonly key: Uint8Array;
}

/** A lockbox key currently held by the local session agent, identified by lockbox and path. */
export interface AgentEntry {
  /** Stable lockbox identifier for the cached key. */
  readonly id: string;
  /** Host path associated with the cached lockbox key. */
  readonly path: string;
}

/** The host capabilities used to protect cached secrets across suspend and sleep. */
export interface SleepSupport {
  /** Whether the host reports impending system suspend. */
  readonly suspendNotifications: boolean;
  /** Whether the agent can delay sleep while handling secrets. */
  readonly sleepInhibition: boolean;
  /** Whether the host supplies enough integration for safe caching. */
  readonly supported: boolean;
}

/** Availability and configuration of the operating-system credential store used for the vault password. */
export interface PlatformStatus {
  /** Whether a usable operating-system credential store exists. */
  readonly supported: boolean;
  /** Whether the user disabled credential-store integration. */
  readonly disabled: boolean;
  /** Application-specific scope used to isolate the stored password. */
  readonly scope: string;
  /** Operating-system credential-store backend in use. */
  readonly backend: string;
  /** Credential item name used by the backend. */
  readonly item: string;
}

/** The version, size, checksum, and creation time of an exported local-vault backup. */
export interface VaultBackupManifest {
  /** Backup container format version. */
  readonly formatVersion: number;
  /** Backup creation time in Unix milliseconds. */
  readonly createdAtUnixMs: bigint;
  /** Metadata-vault filename stored in the backup. */
  readonly vaultFileName: string;
  /** Encrypted vault payload size in bytes. */
  readonly vaultSize: bigint;
  /** Lowercase SHA-256 digest of the encrypted vault payload. */
  readonly vaultSha256: string;
}

/** Structured category, version, guidance, and artifact context for the most recent native failure. */
export interface ErrorDetails {
  /** Stable error category suitable for programmatic handling. */
  readonly category: string;
  /** Kind of archive or vault artifact involved in the failure. */
  readonly artifactKind: string;
  /** Format version read from the failing artifact. */
  readonly foundVersion: number;
  /** Newest format version supported by this library. */
  readonly supportedVersion: number;
  /** Human-readable explanation of the failure. */
  readonly message: string;
  /** Suggested corrective action for the caller or user. */
  readonly guidance: string;
}

/** A source and destination pair accepted by move operations. */
export interface PathMoveInput { readonly source: string; readonly destination: string; }
/** A field accepted when defining a form. */
export interface FormFieldInput { readonly id: string; readonly label: string; readonly kind: string; readonly required: boolean; }
