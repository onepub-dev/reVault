<?php
declare(strict_types=1);

namespace Revault;

use Google\FlatBuffers\ByteBuffer;
use Google\FlatBuffers\FlatbufferBuilder;

/** Metadata for one file, directory, or symbolic link stored at a lockbox path. */
final class LockboxEntry
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $path, public readonly int $kind, public readonly int $length, public readonly int $permissions) {}
    /** Absolute lockbox path of the stored entry. */
    public function getPath(): mixed { return $this->path; }
    /** Filesystem kind: file, directory, or symbolic link. */
    public function getKind(): mixed { return $this->kind; }
    /** Logical file length in bytes; zero for directories. */
    public function getLength(): mixed { return $this->length; }
    /** Portable Unix permission bits stored with the entry. */
    public function getPermissions(): mixed { return $this->permissions; }
}

/** A source and destination pair used to rename a variable or form record atomically. */
final class PathMove
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $source, public readonly string $destination) {}
    /** Existing variable name or form-record path to rename. */
    public function getSource(): mixed { return $this->source; }
    /** New variable name or form-record path. */
    public function getDestination(): mixed { return $this->destination; }
}

/** One named input in a reusable form definition, including its display label and sensitivity kind. */
final class FormField
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $id, public readonly string $label, public readonly string $kind, public readonly bool $required) {}
    /** Stable field identifier used when reading and writing records. */
    public function getId(): mixed { return $this->id; }
    /** Human-readable label presented to a person entering data. */
    public function getLabel(): mixed { return $this->label; }
    /** Field kind that determines validation and secret handling. */
    public function getKind(): mixed { return $this->kind; }
    /** Whether a record must provide a value for this field. */
    public function getRequired(): mixed { return $this->required; }
}

/** A versioned form schema used to validate and label structured records in a lockbox. */
final class FormDefinition
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $type_id, public readonly string $alias, public readonly int $revision, public readonly string $name, public readonly string $description, public readonly array $fields) {}
    /** Stable identifier shared by every revision of this form type. */
    public function getTypeId(): mixed { return $this->type_id; }
    /** Short name used to resolve the current form revision. */
    public function getAlias(): mixed { return $this->alias; }
    /** Monotonically increasing revision number. */
    public function getRevision(): mixed { return $this->revision; }
    /** Human-readable name shown for this form. */
    public function getName(): mixed { return $this->name; }
    /** Explanation shown to people completing the form. */
    public function getDescription(): mixed { return $this->description; }
    /** Ordered inputs accepted by this form revision. */
    public function getFields(): mixed { return $this->fields; }
}

/** The current value and sensitivity metadata for one field in a stored form record. */
final class FormValue
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $field_id, public readonly string $label, public readonly string $kind, public readonly string $value, public readonly bool $secret) {}
    /** Identifier of the form field that owns this value. */
    public function getFieldId(): mixed { return $this->field_id; }
    /** Display label captured from the form revision. */
    public function getLabel(): mixed { return $this->label; }
    /** Field kind captured from the form revision. */
    public function getKind(): mixed { return $this->kind; }
    /** Plain value, or an empty string when the field is secret. */
    public function getValue(): mixed { return $this->value; }
    /** Whether the value must be read through a scoped secret callback. */
    public function getSecret(): mixed { return $this->secret; }
}

/** A named structured record stored at a lockbox path and tied to a form-definition revision. */
final class FormRecord
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $path, public readonly string $name, public readonly string $type_id, public readonly string $definition_alias, public readonly int $definition_revision, public readonly array $values) {}
    /** Absolute lockbox path that identifies the record. */
    public function getPath(): mixed { return $this->path; }
    /** Human-readable name assigned to this record. */
    public function getName(): mixed { return $this->name; }
    /** Stable identifier of the record's form type. */
    public function getTypeId(): mixed { return $this->type_id; }
    /** Alias of the form definition used by the record. */
    public function getDefinitionAlias(): mixed { return $this->definition_alias; }
    /** Exact form revision against which the record was created. */
    public function getDefinitionRevision(): mixed { return $this->definition_revision; }
    /** Ordered non-secret field metadata and values. */
    public function getValues(): mixed { return $this->values; }
}

/** The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox. */
final class RecoveryReport
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly array $intact_files, public readonly int $intact_file_count, public readonly int $partial_files, public readonly int $corrupt_records, public readonly bool $toc_recovered, public readonly bool $variables_recovered, public readonly int $variable_count, public readonly bool $forms_recovered, public readonly int $form_definition_count, public readonly int $form_record_count) {}
    /** Files whose complete contents remain recoverable. */
    public function getIntactFiles(): mixed { return $this->intact_files; }
    /** Number of completely recoverable files. */
    public function getIntactFileCount(): mixed { return $this->intact_file_count; }
    /** Number of files for which only some content is recoverable. */
    public function getPartialFiles(): mixed { return $this->partial_files; }
    /** Number of encrypted records that failed validation. */
    public function getCorruptRecords(): mixed { return $this->corrupt_records; }
    /** Whether a usable table of contents was recovered. */
    public function getTocRecovered(): mixed { return $this->toc_recovered; }
    /** Whether variable metadata was recovered. */
    public function getVariablesRecovered(): mixed { return $this->variables_recovered; }
    /** Number of recovered variables. */
    public function getVariableCount(): mixed { return $this->variable_count; }
    /** Whether form definitions and records were recovered. */
    public function getFormsRecovered(): mixed { return $this->forms_recovered; }
    /** Number of recovered form definitions. */
    public function getFormDefinitionCount(): mixed { return $this->form_definition_count; }
    /** Number of recovered form records. */
    public function getFormRecordCount(): mixed { return $this->form_record_count; }
}

/** One password or contact credential that can unlock a lockbox content key. */
final class KeySlot
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly int $id, public readonly string $protection, public readonly string $algorithm) {}
    /** Stable slot identifier used when removing this access method. */
    public function getId(): mixed { return $this->id; }
    /** Access method, such as password or contact key. */
    public function getProtection(): mixed { return $this->protection; }
    /** Cryptographic algorithm protecting the content key. */
    public function getAlgorithm(): mixed { return $this->algorithm; }
}

/** Current capacity, occupancy, hit, and miss counters for an open lockbox cache. */
final class CacheStats
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly int $limit_bytes, public readonly int $used_bytes, public readonly int $entries, public readonly int $hits, public readonly int $misses) {}
    /** Maximum decoded-page memory permitted for the cache. */
    public function getLimitBytes(): mixed { return $this->limit_bytes; }
    /** Decoded-page memory currently held by the cache. */
    public function getUsedBytes(): mixed { return $this->used_bytes; }
    /** Number of decoded pages currently cached. */
    public function getEntries(): mixed { return $this->entries; }
    /** Reads served by an already decoded page. */
    public function getHits(): mixed { return $this->hits; }
    /** Reads that required decoding another page. */
    public function getMisses(): mixed { return $this->misses; }
}

/** Time spent reading host files and preparing encrypted pages during the latest import work. */
final class ImportStats
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $host_stat_nanos, public readonly string $host_read_nanos, public readonly string $frame_prepare_nanos, public readonly string $page_write_nanos) {}
    /** Nanoseconds spent reading host filesystem metadata, as decimal text. */
    public function getHostStatNanos(): mixed { return $this->host_stat_nanos; }
    /** Nanoseconds spent reading host file content, as decimal text. */
    public function getHostReadNanos(): mixed { return $this->host_read_nanos; }
    /** Nanoseconds spent preparing encrypted records, as decimal text. */
    public function getFramePrepareNanos(): mixed { return $this->frame_prepare_nanos; }
    /** Nanoseconds spent writing encrypted pages, as decimal text. */
    public function getPageWriteNanos(): mixed { return $this->page_write_nanos; }
}

/** One logical object recorded inside an inspected encrypted lockbox page. */
final class PageObject
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly int $id, public readonly string $kind, public readonly int $payload_len) {}
    /** Object identifier recorded in the encrypted page. */
    public function getId(): mixed { return $this->id; }
    /** Kind of logical object stored in the page. */
    public function getKind(): mixed { return $this->kind; }
    /** Encrypted object payload length in bytes. */
    public function getPayloadLen(): mixed { return $this->payload_len; }
}

/** Layout and utilization details for one encrypted page in a lockbox archive. */
final class PageInspection
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly int $offset, public readonly int $page_id, public readonly int $sequence, public readonly int $page_size, public readonly int $encrypted_body_len, public readonly int $unused_bytes, public readonly int $object_count, public readonly array $objects) {}
    /** Byte offset at which the page begins in the archive. */
    public function getOffset(): mixed { return $this->offset; }
    /** Identifier stored in the page header. */
    public function getPageId(): mixed { return $this->page_id; }
    /** Commit sequence that wrote this page. */
    public function getSequence(): mixed { return $this->sequence; }
    /** Total encoded page size in bytes. */
    public function getPageSize(): mixed { return $this->page_size; }
    /** Encrypted body length in bytes. */
    public function getEncryptedBodyLen(): mixed { return $this->encrypted_body_len; }
    /** Unused capacity remaining in the page. */
    public function getUnusedBytes(): mixed { return $this->unused_bytes; }
    /** Number of logical objects stored in the page. */
    public function getObjectCount(): mixed { return $this->object_count; }
    /** Logical objects discovered in the page. */
    public function getObjects(): mixed { return $this->objects; }
}

/** Header, owner-signature, and key-slot information read from a lockbox file without opening its contents. */
final class FileInspection
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $lockbox_id, public readonly bool $header_readable, public readonly int $key_directory_generation, public readonly int $key_directory_copy_count, public readonly bool $owner_signed, public readonly array $key_slots) {}
    /** Stable binary identifier read from the lockbox header. */
    public function getLockboxId(): mixed { return $this->lockbox_id; }
    /** Whether the archive header passed structural validation. */
    public function getHeaderReadable(): mixed { return $this->header_readable; }
    /** Latest readable access-key directory generation. */
    public function getKeyDirectoryGeneration(): mixed { return $this->key_directory_generation; }
    /** Number of readable redundant key-directory copies. */
    public function getKeyDirectoryCopyCount(): mixed { return $this->key_directory_copy_count; }
    /** Whether commits require an owner signature. */
    public function getOwnerSigned(): mixed { return $this->owner_signed; }
    /** Password and contact access methods found in the header. */
    public function getKeySlots(): mixed { return $this->key_slots; }
}

/** One active or retired generation of the contact keys belonging to a named vault profile. */
final class ProfileGeneration
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly int $index, public readonly string $status, public readonly string $contact_fingerprint, public readonly int $created_at_unix_ms, public readonly int $retired_at_unix_ms, public readonly bool $has_retired_at) {}
    /** Generation number used to address this key version. */
    public function getIndex(): mixed { return $this->index; }
    /** Lifecycle state, such as active or retired. */
    public function getStatus(): mixed { return $this->status; }
    /** Fingerprint of this generation's contact public key. */
    public function getContactFingerprint(): mixed { return $this->contact_fingerprint; }
    /** Creation time in Unix milliseconds. */
    public function getCreatedAtUnixMs(): mixed { return $this->created_at_unix_ms; }
    /** Retirement time in Unix milliseconds when retired. */
    public function getRetiredAtUnixMs(): mixed { return $this->retired_at_unix_ms; }
    /** Whether a retirement time is present. */
    public function getHasRetiredAt(): mixed { return $this->has_retired_at; }
}

/** The active generation and rotation history for a named vault profile. */
final class ProfileHistory
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $name, public readonly int $active_generation, public readonly array $generations) {}
    /** Vault profile name whose generations are listed. */
    public function getName(): mixed { return $this->name; }
    /** Generation number currently used for new access grants. */
    public function getActiveGeneration(): mixed { return $this->active_generation; }
    /** Active and retired contact-key generations. */
    public function getGenerations(): mixed { return $this->generations; }
}

/** A lockbox identifier and host path remembered by the local vault for later discovery. */
final class KnownLockbox
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $lockbox_id, public readonly string $path, public readonly int $last_seen_unix_ms) {}
    /** Stable binary identifier of the remembered lockbox. */
    public function getLockboxId(): mixed { return $this->lockbox_id; }
    /** Last known host filesystem path of the lockbox. */
    public function getPath(): mixed { return $this->path; }
    /** Most recent observation time in Unix milliseconds. */
    public function getLastSeenUnixMs(): mixed { return $this->last_seen_unix_ms; }
}

/** A local human-readable label attached to one lockbox access slot. */
final class AccessSlotLabel
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $lockbox_id, public readonly int $slot_id, public readonly string $name, public readonly int $updated_at_unix_ms) {}
    /** Lockbox whose access slot is labelled. */
    public function getLockboxId(): mixed { return $this->lockbox_id; }
    /** Stable identifier of the labelled access slot. */
    public function getSlotId(): mixed { return $this->slot_id; }
    /** Local human-readable label for the access slot. */
    public function getName(): mixed { return $this->name; }
    /** Last label update time in Unix milliseconds. */
    public function getUpdatedAtUnixMs(): mixed { return $this->updated_at_unix_ms; }
}

/** A logical or physical byte range emitted while walking the contents of a lockbox. */
final class StreamChunk
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $path, public readonly int $file_offset, public readonly int $length, public readonly int $physical_offset, public readonly bool $sparse, public readonly string $data) {}
    /** Lockbox file path to which this byte range belongs. */
    public function getPath(): mixed { return $this->path; }
    /** Logical byte offset within the file. */
    public function getFileOffset(): mixed { return $this->file_offset; }
    /** Logical range length in bytes. */
    public function getLength(): mixed { return $this->length; }
    /** Archive byte offset, when physical streaming is requested. */
    public function getPhysicalOffset(): mixed { return $this->physical_offset; }
    /** Whether the range represents a sparse zero-filled extent. */
    public function getSparse(): mixed { return $this->sparse; }
    /** File bytes for a populated logical range. */
    public function getData(): mixed { return $this->data; }
}

/** The workload and worker policies currently applied to an open lockbox. */
final class RuntimeOptions
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $workload_profile, public readonly string $worker_policy) {}
    /** I/O workload policy used to tune page access. */
    public function getWorkloadProfile(): mixed { return $this->workload_profile; }
    /** Worker scheduling policy and effective parallelism. */
    public function getWorkerPolicy(): mixed { return $this->worker_policy; }
}

/** The name and sensitivity classification of a variable stored in a lockbox. */
final class Variable
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $name, public readonly string $sensitivity) {}
    /** Name used to address the variable in the lockbox. */
    public function getName(): mixed { return $this->name; }
    /** Whether the value is ordinary text or a protected secret. */
    public function getSensitivity(): mixed { return $this->sensitivity; }
}

/** Whether a lockbox is owner-signed and, when available, the signing-key fingerprint. */
final class OwnerInspection
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly bool $signed, public readonly string $fingerprint, public readonly bool $has_fingerprint) {}
    /** Whether the lockbox requires owner-signed commits. */
    public function getSigned(): mixed { return $this->signed; }
    /** Owner signing-key fingerprint when one is configured. */
    public function getFingerprint(): mixed { return $this->fingerprint; }
    /** Whether an owner fingerprint is available. */
    public function getHasFingerprint(): mixed { return $this->has_fingerprint; }
}

/** A named recipient public key stored in the local vault address book. */
final class Contact
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $name, public readonly string $key) {}
    /** Local address-book name of the contact. */
    public function getName(): mixed { return $this->name; }
    /** Serialized contact public key used to grant lockbox access. */
    public function getKey(): mixed { return $this->key; }
}

/** A lockbox key currently held by the local session agent, identified by lockbox and path. */
final class AgentEntry
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $id, public readonly string $path) {}
    /** Stable lockbox identifier for the cached key. */
    public function getId(): mixed { return $this->id; }
    /** Host path associated with the cached lockbox key. */
    public function getPath(): mixed { return $this->path; }
}

/** The host capabilities used to protect cached secrets across suspend and sleep. */
final class SleepSupport
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly bool $suspend_notifications, public readonly bool $sleep_inhibition, public readonly bool $supported) {}
    /** Whether the host reports impending system suspend. */
    public function getSuspendNotifications(): mixed { return $this->suspend_notifications; }
    /** Whether the agent can delay sleep while handling secrets. */
    public function getSleepInhibition(): mixed { return $this->sleep_inhibition; }
    /** Whether the host supplies enough integration for safe caching. */
    public function getSupported(): mixed { return $this->supported; }
}

/** Availability and configuration of the operating-system credential store used for the vault password. */
final class PlatformStatus
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly bool $supported, public readonly bool $disabled, public readonly string $scope, public readonly string $backend, public readonly string $item) {}
    /** Whether a usable operating-system credential store exists. */
    public function getSupported(): mixed { return $this->supported; }
    /** Whether the user disabled credential-store integration. */
    public function getDisabled(): mixed { return $this->disabled; }
    /** Application-specific scope used to isolate the stored password. */
    public function getScope(): mixed { return $this->scope; }
    /** Operating-system credential-store backend in use. */
    public function getBackend(): mixed { return $this->backend; }
    /** Credential item name used by the backend. */
    public function getItem(): mixed { return $this->item; }
}

/** The version, size, checksum, and creation time of an exported local-vault backup. */
final class VaultBackupManifest
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly int $format_version, public readonly int $created_at_unix_ms, public readonly string $vault_file_name, public readonly int $vault_size, public readonly string $vault_sha256) {}
    /** Backup container format version. */
    public function getFormatVersion(): mixed { return $this->format_version; }
    /** Backup creation time in Unix milliseconds. */
    public function getCreatedAtUnixMs(): mixed { return $this->created_at_unix_ms; }
    /** Metadata-vault filename stored in the backup. */
    public function getVaultFileName(): mixed { return $this->vault_file_name; }
    /** Encrypted vault payload size in bytes. */
    public function getVaultSize(): mixed { return $this->vault_size; }
    /** Lowercase SHA-256 digest of the encrypted vault payload. */
    public function getVaultSha256(): mixed { return $this->vault_sha256; }
}

/** Structured category, version, guidance, and artifact context for the most recent native failure. */
final class ErrorDetails
{
    /** Creates a value from fields returned by the reVault API. */
    public function __construct(public readonly string $category, public readonly string $artifact_kind, public readonly int $found_version, public readonly int $supported_version, public readonly string $message, public readonly string $guidance) {}
    /** Stable error category suitable for programmatic handling. */
    public function getCategory(): mixed { return $this->category; }
    /** Kind of archive or vault artifact involved in the failure. */
    public function getArtifactKind(): mixed { return $this->artifact_kind; }
    /** Format version read from the failing artifact. */
    public function getFoundVersion(): mixed { return $this->found_version; }
    /** Newest format version supported by this library. */
    public function getSupportedVersion(): mixed { return $this->supported_version; }
    /** Human-readable explanation of the failure. */
    public function getMessage(): mixed { return $this->message; }
    /** Suggested corrective action for the caller or user. */
    public function getGuidance(): mixed { return $this->guidance; }
}

/** Ordered lockbox entries selected by a list operation. */
final class LockboxEntryList
{
    /** @param list<LockboxEntry> $entries */
    public function __construct(public readonly array $entries) {}
    /** Returns the ordered LockboxEntry values. */
    public function getEntries(): array { return $this->entries; }
}

/** Ordered field definitions supplied when defining a form. */
final class FormFieldList
{
    /** @param list<FormField> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered FormField values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered FormDefinition values returned by the corresponding list operation. */
final class FormDefinitionList
{
    /** @param list<FormDefinition> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered FormDefinition values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered FormRecord values returned by the corresponding list operation. */
final class FormRecordList
{
    /** @param list<FormRecord> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered FormRecord values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered KeySlot values returned by the corresponding list operation. */
final class KeySlotList
{
    /** @param list<KeySlot> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered KeySlot values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered PageInspection values returned by the corresponding list operation. */
final class PageInspectionList
{
    /** @param list<PageInspection> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered PageInspection values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered KnownLockbox values returned by the corresponding list operation. */
final class KnownLockboxList
{
    /** @param list<KnownLockbox> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered KnownLockbox values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered AccessSlotLabel values returned by the corresponding list operation. */
final class AccessSlotLabelList
{
    /** @param list<AccessSlotLabel> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered AccessSlotLabel values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered StreamChunk values returned by the corresponding list operation. */
final class StreamChunkList
{
    /** @param list<StreamChunk> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered StreamChunk values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered Variable values returned by the corresponding list operation. */
final class VariableList
{
    /** @param list<Variable> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered Variable values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered Contact values returned by the corresponding list operation. */
final class ContactList
{
    /** @param list<Contact> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered Contact values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered ProfileHistory values returned by the corresponding list operation. */
final class ProfileHistoryList
{
    /** @param list<ProfileHistory> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered ProfileHistory values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered AgentEntry values returned by the corresponding list operation. */
final class AgentEntryList
{
    /** @param list<AgentEntry> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered AgentEntry values. */
    public function getValues(): array { return $this->values; }
}

/** Ordered text values returned by a list operation. */
final class StringList
{
    /** @param list<string> $values */
    public function __construct(public readonly array $values) {}
    /** Returns the ordered text values. */
    public function getValues(): array { return $this->values; }
}

/** A lookup result that may contain one LockboxEntry. */
final class OptionalLockboxEntry
{
    /** Creates a lookup result; null means the requested value was absent. */
    public function __construct(public readonly ?LockboxEntry $value) {}
    /** Returns the value, or null when it was absent. */
    public function getValue(): ?LockboxEntry { return $this->value; }
}

/** A lookup result that may contain one FormRecord. */
final class OptionalFormRecord
{
    /** Creates a lookup result; null means the requested value was absent. */
    public function __construct(public readonly ?FormRecord $value) {}
    /** Returns the value, or null when it was absent. */
    public function getValue(): ?FormRecord { return $this->value; }
}

/** A lookup result that may contain one FormValue. */
final class OptionalFormValue
{
    /** Creates a lookup result; null means the requested value was absent. */
    public function __construct(public readonly ?FormValue $value) {}
    /** Returns the value, or null when it was absent. */
    public function getValue(): ?FormValue { return $this->value; }
}

/** A text lookup result that distinguishes absence from an empty string. */
final class OptionalString
{
    /** Creates an optional text result. */
    public function __construct(public readonly bool $present, public readonly string $value) {}
    /** Reports whether the requested text exists. */
    public function getPresent(): bool { return $this->present; }
    /** Returns the text, which may be empty even when present. */
    public function getValue(): string { return $this->value; }
}

/** @internal Converts private native buffers to public values and encodes supported inputs. */
final class DomainCodec
{
    private function __construct() {}
    private static function fromLockboxEntry(\Revault\Internal\Transport\LockboxEntry $value): LockboxEntry
    {
        return new LockboxEntry($value->getPath() ?? '', $value->getKind(), $value->getLength(), $value->getPermissions());
    }
    private static function fromPathMove(\Revault\Internal\Transport\PathMove $value): PathMove
    {
        return new PathMove($value->getSource() ?? '', $value->getDestination() ?? '');
    }
    private static function fromFormField(\Revault\Internal\Transport\FormField $value): FormField
    {
        return new FormField($value->getId() ?? '', $value->getLabel() ?? '', $value->getKind() ?? '', $value->getRequired());
    }
    private static function fromFormDefinition(\Revault\Internal\Transport\FormDefinition $value): FormDefinition
    {
        return new FormDefinition($value->getTypeId() ?? '', $value->getAlias() ?? '', $value->getRevision(), $value->getName() ?? '', $value->getDescription() ?? '', self::tables($value, 'getFields', 'FormField'));
    }
    private static function fromFormValue(\Revault\Internal\Transport\FormValue $value): FormValue
    {
        return new FormValue($value->getFieldId() ?? '', $value->getLabel() ?? '', $value->getKind() ?? '', $value->getValue() ?? '', $value->getSecret());
    }
    private static function fromFormRecord(\Revault\Internal\Transport\FormRecord $value): FormRecord
    {
        return new FormRecord($value->getPath() ?? '', $value->getName() ?? '', $value->getTypeId() ?? '', $value->getDefinitionAlias() ?? '', $value->getDefinitionRevision(), self::tables($value, 'getValues', 'FormValue'));
    }
    private static function fromRecoveryReport(\Revault\Internal\Transport\RecoveryReport $value): RecoveryReport
    {
        return new RecoveryReport(self::tables($value, 'getIntactFiles', 'LockboxEntry'), $value->getIntactFileCount(), $value->getPartialFiles(), $value->getCorruptRecords(), $value->getTocRecovered(), $value->getVariablesRecovered(), $value->getVariableCount(), $value->getFormsRecovered(), $value->getFormDefinitionCount(), $value->getFormRecordCount());
    }
    private static function fromKeySlot(\Revault\Internal\Transport\KeySlot $value): KeySlot
    {
        return new KeySlot($value->getId(), $value->getProtection() ?? '', $value->getAlgorithm() ?? '');
    }
    private static function fromCacheStats(\Revault\Internal\Transport\CacheStats $value): CacheStats
    {
        return new CacheStats($value->getLimitBytes(), $value->getUsedBytes(), $value->getEntries(), $value->getHits(), $value->getMisses());
    }
    private static function fromImportStats(\Revault\Internal\Transport\ImportStats $value): ImportStats
    {
        return new ImportStats($value->getHostStatNanos() ?? '', $value->getHostReadNanos() ?? '', $value->getFramePrepareNanos() ?? '', $value->getPageWriteNanos() ?? '');
    }
    private static function fromPageObject(\Revault\Internal\Transport\PageObject $value): PageObject
    {
        return new PageObject($value->getId(), $value->getKind() ?? '', $value->getPayloadLen());
    }
    private static function fromPageInspection(\Revault\Internal\Transport\PageInspection $value): PageInspection
    {
        return new PageInspection($value->getOffset(), $value->getPageId(), $value->getSequence(), $value->getPageSize(), $value->getEncryptedBodyLen(), $value->getUnusedBytes(), $value->getObjectCount(), self::tables($value, 'getObjects', 'PageObject'));
    }
    private static function fromFileInspection(\Revault\Internal\Transport\FileInspection $value): FileInspection
    {
        return new FileInspection(self::bytes($value, 'getLockboxId'), $value->getHeaderReadable(), $value->getKeyDirectoryGeneration(), $value->getKeyDirectoryCopyCount(), $value->getOwnerSigned(), self::tables($value, 'getKeySlots', 'KeySlot'));
    }
    private static function fromProfileGeneration(\Revault\Internal\Transport\ProfileGeneration $value): ProfileGeneration
    {
        return new ProfileGeneration($value->getIndex(), $value->getStatus() ?? '', self::bytes($value, 'getContactFingerprint'), $value->getCreatedAtUnixMs(), $value->getRetiredAtUnixMs(), $value->getHasRetiredAt());
    }
    private static function fromProfileHistory(\Revault\Internal\Transport\ProfileHistory $value): ProfileHistory
    {
        return new ProfileHistory($value->getName() ?? '', $value->getActiveGeneration(), self::tables($value, 'getGenerations', 'ProfileGeneration'));
    }
    private static function fromKnownLockbox(\Revault\Internal\Transport\KnownLockbox $value): KnownLockbox
    {
        return new KnownLockbox(self::bytes($value, 'getLockboxId'), $value->getPath() ?? '', $value->getLastSeenUnixMs());
    }
    private static function fromAccessSlotLabel(\Revault\Internal\Transport\AccessSlotLabel $value): AccessSlotLabel
    {
        return new AccessSlotLabel(self::bytes($value, 'getLockboxId'), $value->getSlotId(), $value->getName() ?? '', $value->getUpdatedAtUnixMs());
    }
    private static function fromStreamChunk(\Revault\Internal\Transport\StreamChunk $value): StreamChunk
    {
        return new StreamChunk($value->getPath() ?? '', $value->getFileOffset(), $value->getLength(), $value->getPhysicalOffset(), $value->getSparse(), self::bytes($value, 'getData'));
    }
    private static function fromRuntimeOptions(\Revault\Internal\Transport\RuntimeOptions $value): RuntimeOptions
    {
        return new RuntimeOptions($value->getWorkloadProfile() ?? '', $value->getWorkerPolicy() ?? '');
    }
    private static function fromVariable(\Revault\Internal\Transport\Variable $value): Variable
    {
        return new Variable($value->getName() ?? '', $value->getSensitivity() ?? '');
    }
    private static function fromOwnerInspection(\Revault\Internal\Transport\OwnerInspection $value): OwnerInspection
    {
        return new OwnerInspection($value->getSigned(), $value->getFingerprint() ?? '', $value->getHasFingerprint());
    }
    private static function fromContact(\Revault\Internal\Transport\Contact $value): Contact
    {
        return new Contact($value->getName() ?? '', self::bytes($value, 'getKey'));
    }
    private static function fromAgentEntry(\Revault\Internal\Transport\AgentEntry $value): AgentEntry
    {
        return new AgentEntry($value->getId() ?? '', $value->getPath() ?? '');
    }
    private static function fromSleepSupport(\Revault\Internal\Transport\SleepSupport $value): SleepSupport
    {
        return new SleepSupport($value->getSuspendNotifications(), $value->getSleepInhibition(), $value->getSupported());
    }
    private static function fromPlatformStatus(\Revault\Internal\Transport\PlatformStatus $value): PlatformStatus
    {
        return new PlatformStatus($value->getSupported(), $value->getDisabled(), $value->getScope() ?? '', $value->getBackend() ?? '', $value->getItem() ?? '');
    }
    private static function fromVaultBackupManifest(\Revault\Internal\Transport\VaultBackupManifest $value): VaultBackupManifest
    {
        return new VaultBackupManifest($value->getFormatVersion(), $value->getCreatedAtUnixMs(), $value->getVaultFileName() ?? '', $value->getVaultSize(), $value->getVaultSha256() ?? '');
    }
    private static function fromErrorDetails(\Revault\Internal\Transport\ErrorDetails $value): ErrorDetails
    {
        return new ErrorDetails($value->getCategory() ?? '', $value->getArtifactKind() ?? '', $value->getFoundVersion(), $value->getSupportedVersion(), $value->getMessage() ?? '', $value->getGuidance() ?? '');
    }
    /** @internal Decodes a native LockboxEntry result. */
    public static function decodeLockboxEntry(string $bytes): LockboxEntry
    {
        return self::fromLockboxEntry(\Revault\Internal\Transport\LockboxEntry::getRootAsLockboxEntry(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native PathMove result. */
    public static function decodePathMove(string $bytes): PathMove
    {
        return self::fromPathMove(\Revault\Internal\Transport\PathMove::getRootAsPathMove(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native FormField result. */
    public static function decodeFormField(string $bytes): FormField
    {
        return self::fromFormField(\Revault\Internal\Transport\FormField::getRootAsFormField(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native FormDefinition result. */
    public static function decodeFormDefinition(string $bytes): FormDefinition
    {
        return self::fromFormDefinition(\Revault\Internal\Transport\FormDefinition::getRootAsFormDefinition(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native FormValue result. */
    public static function decodeFormValue(string $bytes): FormValue
    {
        return self::fromFormValue(\Revault\Internal\Transport\FormValue::getRootAsFormValue(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native FormRecord result. */
    public static function decodeFormRecord(string $bytes): FormRecord
    {
        return self::fromFormRecord(\Revault\Internal\Transport\FormRecord::getRootAsFormRecord(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native RecoveryReport result. */
    public static function decodeRecoveryReport(string $bytes): RecoveryReport
    {
        return self::fromRecoveryReport(\Revault\Internal\Transport\RecoveryReport::getRootAsRecoveryReport(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native KeySlot result. */
    public static function decodeKeySlot(string $bytes): KeySlot
    {
        return self::fromKeySlot(\Revault\Internal\Transport\KeySlot::getRootAsKeySlot(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native CacheStats result. */
    public static function decodeCacheStats(string $bytes): CacheStats
    {
        return self::fromCacheStats(\Revault\Internal\Transport\CacheStats::getRootAsCacheStats(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native ImportStats result. */
    public static function decodeImportStats(string $bytes): ImportStats
    {
        return self::fromImportStats(\Revault\Internal\Transport\ImportStats::getRootAsImportStats(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native PageObject result. */
    public static function decodePageObject(string $bytes): PageObject
    {
        return self::fromPageObject(\Revault\Internal\Transport\PageObject::getRootAsPageObject(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native PageInspection result. */
    public static function decodePageInspection(string $bytes): PageInspection
    {
        return self::fromPageInspection(\Revault\Internal\Transport\PageInspection::getRootAsPageInspection(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native FileInspection result. */
    public static function decodeFileInspection(string $bytes): FileInspection
    {
        return self::fromFileInspection(\Revault\Internal\Transport\FileInspection::getRootAsFileInspection(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native ProfileGeneration result. */
    public static function decodeProfileGeneration(string $bytes): ProfileGeneration
    {
        return self::fromProfileGeneration(\Revault\Internal\Transport\ProfileGeneration::getRootAsProfileGeneration(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native ProfileHistory result. */
    public static function decodeProfileHistory(string $bytes): ProfileHistory
    {
        return self::fromProfileHistory(\Revault\Internal\Transport\ProfileHistory::getRootAsProfileHistory(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native KnownLockbox result. */
    public static function decodeKnownLockbox(string $bytes): KnownLockbox
    {
        return self::fromKnownLockbox(\Revault\Internal\Transport\KnownLockbox::getRootAsKnownLockbox(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native AccessSlotLabel result. */
    public static function decodeAccessSlotLabel(string $bytes): AccessSlotLabel
    {
        return self::fromAccessSlotLabel(\Revault\Internal\Transport\AccessSlotLabel::getRootAsAccessSlotLabel(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native StreamChunk result. */
    public static function decodeStreamChunk(string $bytes): StreamChunk
    {
        return self::fromStreamChunk(\Revault\Internal\Transport\StreamChunk::getRootAsStreamChunk(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native RuntimeOptions result. */
    public static function decodeRuntimeOptions(string $bytes): RuntimeOptions
    {
        return self::fromRuntimeOptions(\Revault\Internal\Transport\RuntimeOptions::getRootAsRuntimeOptions(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native Variable result. */
    public static function decodeVariable(string $bytes): Variable
    {
        return self::fromVariable(\Revault\Internal\Transport\Variable::getRootAsVariable(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native OwnerInspection result. */
    public static function decodeOwnerInspection(string $bytes): OwnerInspection
    {
        return self::fromOwnerInspection(\Revault\Internal\Transport\OwnerInspection::getRootAsOwnerInspection(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native Contact result. */
    public static function decodeContact(string $bytes): Contact
    {
        return self::fromContact(\Revault\Internal\Transport\Contact::getRootAsContact(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native AgentEntry result. */
    public static function decodeAgentEntry(string $bytes): AgentEntry
    {
        return self::fromAgentEntry(\Revault\Internal\Transport\AgentEntry::getRootAsAgentEntry(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native SleepSupport result. */
    public static function decodeSleepSupport(string $bytes): SleepSupport
    {
        return self::fromSleepSupport(\Revault\Internal\Transport\SleepSupport::getRootAsSleepSupport(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native PlatformStatus result. */
    public static function decodePlatformStatus(string $bytes): PlatformStatus
    {
        return self::fromPlatformStatus(\Revault\Internal\Transport\PlatformStatus::getRootAsPlatformStatus(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native VaultBackupManifest result. */
    public static function decodeVaultBackupManifest(string $bytes): VaultBackupManifest
    {
        return self::fromVaultBackupManifest(\Revault\Internal\Transport\VaultBackupManifest::getRootAsVaultBackupManifest(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native ErrorDetails result. */
    public static function decodeErrorDetails(string $bytes): ErrorDetails
    {
        return self::fromErrorDetails(\Revault\Internal\Transport\ErrorDetails::getRootAsErrorDetails(ByteBuffer::wrap($bytes)));
    }
    /** @internal Decodes a native LockboxEntryList result. */
    public static function decodeLockboxEntryList(string $bytes): LockboxEntryList
    {
        $root = \Revault\Internal\Transport\LockboxEntryList::getRootAsLockboxEntryList(ByteBuffer::wrap($bytes));
        return new LockboxEntryList(self::tables($root, 'getEntries', 'LockboxEntry'));
    }
    /** @internal Decodes a native FormFieldList result. */
    public static function decodeFormFieldList(string $bytes): FormFieldList
    {
        $root = \Revault\Internal\Transport\FormFieldList::getRootAsFormFieldList(ByteBuffer::wrap($bytes));
        return new FormFieldList(self::tables($root, 'getValues', 'FormField'));
    }
    /** @internal Decodes a native FormDefinitionList result. */
    public static function decodeFormDefinitionList(string $bytes): FormDefinitionList
    {
        $root = \Revault\Internal\Transport\FormDefinitionList::getRootAsFormDefinitionList(ByteBuffer::wrap($bytes));
        return new FormDefinitionList(self::tables($root, 'getValues', 'FormDefinition'));
    }
    /** @internal Decodes a native FormRecordList result. */
    public static function decodeFormRecordList(string $bytes): FormRecordList
    {
        $root = \Revault\Internal\Transport\FormRecordList::getRootAsFormRecordList(ByteBuffer::wrap($bytes));
        return new FormRecordList(self::tables($root, 'getValues', 'FormRecord'));
    }
    /** @internal Decodes a native KeySlotList result. */
    public static function decodeKeySlotList(string $bytes): KeySlotList
    {
        $root = \Revault\Internal\Transport\KeySlotList::getRootAsKeySlotList(ByteBuffer::wrap($bytes));
        return new KeySlotList(self::tables($root, 'getValues', 'KeySlot'));
    }
    /** @internal Decodes a native PageInspectionList result. */
    public static function decodePageInspectionList(string $bytes): PageInspectionList
    {
        $root = \Revault\Internal\Transport\PageInspectionList::getRootAsPageInspectionList(ByteBuffer::wrap($bytes));
        return new PageInspectionList(self::tables($root, 'getValues', 'PageInspection'));
    }
    /** @internal Decodes a native KnownLockboxList result. */
    public static function decodeKnownLockboxList(string $bytes): KnownLockboxList
    {
        $root = \Revault\Internal\Transport\KnownLockboxList::getRootAsKnownLockboxList(ByteBuffer::wrap($bytes));
        return new KnownLockboxList(self::tables($root, 'getValues', 'KnownLockbox'));
    }
    /** @internal Decodes a native AccessSlotLabelList result. */
    public static function decodeAccessSlotLabelList(string $bytes): AccessSlotLabelList
    {
        $root = \Revault\Internal\Transport\AccessSlotLabelList::getRootAsAccessSlotLabelList(ByteBuffer::wrap($bytes));
        return new AccessSlotLabelList(self::tables($root, 'getValues', 'AccessSlotLabel'));
    }
    /** @internal Decodes a native StreamChunkList result. */
    public static function decodeStreamChunkList(string $bytes): StreamChunkList
    {
        $root = \Revault\Internal\Transport\StreamChunkList::getRootAsStreamChunkList(ByteBuffer::wrap($bytes));
        return new StreamChunkList(self::tables($root, 'getValues', 'StreamChunk'));
    }
    /** @internal Decodes a native VariableList result. */
    public static function decodeVariableList(string $bytes): VariableList
    {
        $root = \Revault\Internal\Transport\VariableList::getRootAsVariableList(ByteBuffer::wrap($bytes));
        return new VariableList(self::tables($root, 'getValues', 'Variable'));
    }
    /** @internal Decodes a native ContactList result. */
    public static function decodeContactList(string $bytes): ContactList
    {
        $root = \Revault\Internal\Transport\ContactList::getRootAsContactList(ByteBuffer::wrap($bytes));
        return new ContactList(self::tables($root, 'getValues', 'Contact'));
    }
    /** @internal Decodes a native ProfileHistoryList result. */
    public static function decodeProfileHistoryList(string $bytes): ProfileHistoryList
    {
        $root = \Revault\Internal\Transport\ProfileHistoryList::getRootAsProfileHistoryList(ByteBuffer::wrap($bytes));
        return new ProfileHistoryList(self::tables($root, 'getValues', 'ProfileHistory'));
    }
    /** @internal Decodes a native AgentEntryList result. */
    public static function decodeAgentEntryList(string $bytes): AgentEntryList
    {
        $root = \Revault\Internal\Transport\AgentEntryList::getRootAsAgentEntryList(ByteBuffer::wrap($bytes));
        return new AgentEntryList(self::tables($root, 'getValues', 'AgentEntry'));
    }
    /** @internal Decodes a native StringList result. */
    public static function decodeStringList(string $bytes): StringList
    {
        $root = \Revault\Internal\Transport\StringList::getRootAsStringList(ByteBuffer::wrap($bytes));
        return new StringList(self::scalars($root, 'getValues'));
    }
    /** @internal Decodes a native OptionalLockboxEntry result. */
    public static function decodeOptionalLockboxEntry(string $bytes): OptionalLockboxEntry
    {
        $root = \Revault\Internal\Transport\OptionalLockboxEntry::getRootAsOptionalLockboxEntry(ByteBuffer::wrap($bytes)); $value = $root->getValue();
        return new OptionalLockboxEntry($value === null ? null : self::fromLockboxEntry($value));
    }
    /** @internal Decodes a native OptionalFormRecord result. */
    public static function decodeOptionalFormRecord(string $bytes): OptionalFormRecord
    {
        $root = \Revault\Internal\Transport\OptionalFormRecord::getRootAsOptionalFormRecord(ByteBuffer::wrap($bytes)); $value = $root->getValue();
        return new OptionalFormRecord($value === null ? null : self::fromFormRecord($value));
    }
    /** @internal Decodes a native OptionalFormValue result. */
    public static function decodeOptionalFormValue(string $bytes): OptionalFormValue
    {
        $root = \Revault\Internal\Transport\OptionalFormValue::getRootAsOptionalFormValue(ByteBuffer::wrap($bytes)); $value = $root->getValue();
        return new OptionalFormValue($value === null ? null : self::fromFormValue($value));
    }
    /** @internal Decodes a native OptionalString result. */
    public static function decodeOptionalString(string $bytes): OptionalString
    {
        $root = \Revault\Internal\Transport\OptionalString::getRootAsOptionalString(ByteBuffer::wrap($bytes));
        return new OptionalString($root->getPresent(), $root->getValue() ?? '');
    }
    /** @return list<mixed> */
    private static function tables(object $value, string $getter, string $type): array
    {
        $length = $getter . 'Length'; $result = [];
        for ($index = 0; $index < $value->$length(); $index++) { $convert = 'from' . $type; $result[] = self::$convert($value->$getter($index)); }
        return $result;
    }
    /** @return list<mixed> */
    private static function scalars(object $value, string $getter): array
    {
        $length = $getter . 'Length'; $result = []; for ($index = 0; $index < $value->$length(); $index++) $result[] = $value->$getter($index); return $result;
    }
    /** Returns a byte vector as a PHP binary string. */
    private static function bytes(object $value, string $getter): string
    {
        return implode('', array_map(chr(...), self::scalars($value, $getter)));
    }
    /** Encodes variable or form-record path moves for the native API. */
    public static function encodePathMoves(array $moves): string
    {
        $builder = new FlatBufferBuilder(256); $offsets = [];
        foreach ($moves as $move) { $source = $builder->createString($move->source); $destination = $builder->createString($move->destination); $offsets[] = \Revault\Internal\Transport\PathMove::createPathMove($builder, $source, $destination); }
        $values = \Revault\Internal\Transport\PathMoveList::createValuesVector($builder, $offsets); $root = \Revault\Internal\Transport\PathMoveList::createPathMoveList($builder, $values); $builder->finish($root); return $builder->sizedByteArray();
    }
    /** Encodes form fields for the native API. */
    public static function encodeFormFields(array $fields): string
    {
        $builder = new FlatBufferBuilder(256); $offsets = [];
        foreach ($fields as $field) { $id = $builder->createString($field->id); $label = $builder->createString($field->label); $kind = $builder->createString($field->kind); $offsets[] = \Revault\Internal\Transport\FormField::createFormField($builder, $id, $label, $kind, $field->required); }
        $values = \Revault\Internal\Transport\FormFieldList::createValuesVector($builder, $offsets); $root = \Revault\Internal\Transport\FormFieldList::createFormFieldList($builder, $values); $builder->finish($root); return $builder->sizedByteArray();
    }
}
