#pragma once

#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <vector>

namespace revault::bindings {

/** Identifies the filesystem object stored at a lockbox path. */
enum class LockboxEntryKind { unspecified, file, symlink, directory };

/** Metadata for one file, directory, or symbolic link stored at a lockbox path. */
struct LockboxEntry {
  /** Absolute lockbox path of the stored entry. */
  std::string path{};
  /** Filesystem kind: file, directory, or symbolic link. */
  LockboxEntryKind kind{};
  /** Logical file length in bytes; zero for directories. */
  std::uint64_t length{};
  /** Portable Unix permission bits stored with the entry. */
  std::uint32_t permissions{};
};

/** A source and destination pair used to rename a variable or form record atomically. */
struct PathMove {
  /** Existing variable name or form-record path to rename. */
  std::string source{};
  /** New variable name or form-record path. */
  std::string destination{};
};

/** One named input in a reusable form definition, including its display label and sensitivity kind. */
struct FormField {
  /** Stable field identifier used when reading and writing records. */
  std::string id{};
  /** Human-readable label presented to a person entering data. */
  std::string label{};
  /** Field kind that determines validation and secret handling. */
  std::string kind{};
  /** Whether a record must provide a value for this field. */
  bool required{};
};

/** A versioned form schema used to validate and label structured records in a lockbox. */
struct FormDefinition {
  /** Stable identifier shared by every revision of this form type. */
  std::string type_id{};
  /** Short name used to resolve the current form revision. */
  std::string alias{};
  /** Monotonically increasing revision number. */
  std::uint32_t revision{};
  /** Human-readable name shown for this form. */
  std::string name{};
  /** Explanation shown to people completing the form. */
  std::string description{};
  /** Ordered inputs accepted by this form revision. */
  std::vector<FormField> fields{};
};

/** The current value and sensitivity metadata for one field in a stored form record. */
struct FormValue {
  /** Identifier of the form field that owns this value. */
  std::string field_id{};
  /** Display label captured from the form revision. */
  std::string label{};
  /** Field kind captured from the form revision. */
  std::string kind{};
  /** Plain value, or an empty string when the field is secret. */
  std::string value{};
  /** Whether the value must be read through a scoped secret callback. */
  bool secret{};
};

/** A named structured record stored at a lockbox path and tied to a form-definition revision. */
struct FormRecord {
  /** Absolute lockbox path that identifies the record. */
  std::string path{};
  /** Human-readable name assigned to this record. */
  std::string name{};
  /** Stable identifier of the record's form type. */
  std::string type_id{};
  /** Alias of the form definition used by the record. */
  std::string definition_alias{};
  /** Exact form revision against which the record was created. */
  std::uint32_t definition_revision{};
  /** Ordered non-secret field metadata and values. */
  std::vector<FormValue> values{};
};

/** The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox. */
struct RecoveryReport {
  /** Files whose complete contents remain recoverable. */
  std::vector<LockboxEntry> intact_files{};
  /** Number of completely recoverable files. */
  std::uint64_t intact_file_count{};
  /** Number of files for which only some content is recoverable. */
  std::uint64_t partial_files{};
  /** Number of encrypted records that failed validation. */
  std::uint64_t corrupt_records{};
  /** Whether a usable table of contents was recovered. */
  bool toc_recovered{};
  /** Whether variable metadata was recovered. */
  bool variables_recovered{};
  /** Number of recovered variables. */
  std::uint64_t variable_count{};
  /** Whether form definitions and records were recovered. */
  bool forms_recovered{};
  /** Number of recovered form definitions. */
  std::uint64_t form_definition_count{};
  /** Number of recovered form records. */
  std::uint64_t form_record_count{};
};

/** One password or contact credential that can unlock a lockbox content key. */
struct KeySlot {
  /** Stable slot identifier used when removing this access method. */
  std::uint64_t id{};
  /** Access method, such as password or contact key. */
  std::string protection{};
  /** Cryptographic algorithm protecting the content key. */
  std::string algorithm{};
};

/** Current capacity, occupancy, hit, and miss counters for an open lockbox cache. */
struct CacheStats {
  /** Maximum decoded-page memory permitted for the cache. */
  std::uint64_t limit_bytes{};
  /** Decoded-page memory currently held by the cache. */
  std::uint64_t used_bytes{};
  /** Number of decoded pages currently cached. */
  std::uint64_t entries{};
  /** Reads served by an already decoded page. */
  std::uint64_t hits{};
  /** Reads that required decoding another page. */
  std::uint64_t misses{};
};

/** Time spent reading host files and preparing encrypted pages during the latest import work. */
struct ImportStats {
  /** Nanoseconds spent reading host filesystem metadata, as decimal text. */
  std::string host_stat_nanos{};
  /** Nanoseconds spent reading host file content, as decimal text. */
  std::string host_read_nanos{};
  /** Nanoseconds spent preparing encrypted records, as decimal text. */
  std::string frame_prepare_nanos{};
  /** Nanoseconds spent writing encrypted pages, as decimal text. */
  std::string page_write_nanos{};
};

/** One logical object recorded inside an inspected encrypted lockbox page. */
struct PageObject {
  /** Object identifier recorded in the encrypted page. */
  std::uint64_t id{};
  /** Kind of logical object stored in the page. */
  std::string kind{};
  /** Encrypted object payload length in bytes. */
  std::uint64_t payload_len{};
};

/** Layout and utilization details for one encrypted page in a lockbox archive. */
struct PageInspection {
  /** Byte offset at which the page begins in the archive. */
  std::uint64_t offset{};
  /** Identifier stored in the page header. */
  std::uint64_t page_id{};
  /** Commit sequence that wrote this page. */
  std::uint64_t sequence{};
  /** Total encoded page size in bytes. */
  std::uint64_t page_size{};
  /** Encrypted body length in bytes. */
  std::uint64_t encrypted_body_len{};
  /** Unused capacity remaining in the page. */
  std::uint64_t unused_bytes{};
  /** Number of logical objects stored in the page. */
  std::uint64_t object_count{};
  /** Logical objects discovered in the page. */
  std::vector<PageObject> objects{};
};

/** Header, owner-signature, and key-slot information read from a lockbox file without opening its contents. */
struct FileInspection {
  /** Stable binary identifier read from the lockbox header. */
  std::vector<std::uint8_t> lockbox_id{};
  /** Whether the archive header passed structural validation. */
  bool header_readable{};
  /** Latest readable access-key directory generation. */
  std::uint64_t key_directory_generation{};
  /** Number of readable redundant key-directory copies. */
  std::uint64_t key_directory_copy_count{};
  /** Whether commits require an owner signature. */
  bool owner_signed{};
  /** Password and contact access methods found in the header. */
  std::vector<KeySlot> key_slots{};
};

/** One active or retired generation of the contact keys belonging to a named vault profile. */
struct ProfileGeneration {
  /** Generation number used to address this key version. */
  std::uint32_t index{};
  /** Lifecycle state, such as active or retired. */
  std::string status{};
  /** Fingerprint of this generation's contact public key. */
  std::vector<std::uint8_t> contact_fingerprint{};
  /** Creation time in Unix milliseconds. */
  std::uint64_t created_at_unix_ms{};
  /** Retirement time in Unix milliseconds when retired. */
  std::uint64_t retired_at_unix_ms{};
  /** Whether a retirement time is present. */
  bool has_retired_at{};
};

/** The active generation and rotation history for a named vault profile. */
struct ProfileHistory {
  /** Vault profile name whose generations are listed. */
  std::string name{};
  /** Generation number currently used for new access grants. */
  std::uint32_t active_generation{};
  /** Active and retired contact-key generations. */
  std::vector<ProfileGeneration> generations{};
};

/** A lockbox identifier and host path remembered by the local vault for later discovery. */
struct KnownLockbox {
  /** Stable binary identifier of the remembered lockbox. */
  std::vector<std::uint8_t> lockbox_id{};
  /** Last known host filesystem path of the lockbox. */
  std::string path{};
  /** Most recent observation time in Unix milliseconds. */
  std::uint64_t last_seen_unix_ms{};
};

/** A local human-readable label attached to one lockbox access slot. */
struct AccessSlotLabel {
  /** Lockbox whose access slot is labelled. */
  std::vector<std::uint8_t> lockbox_id{};
  /** Stable identifier of the labelled access slot. */
  std::uint64_t slot_id{};
  /** Local human-readable label for the access slot. */
  std::string name{};
  /** Last label update time in Unix milliseconds. */
  std::uint64_t updated_at_unix_ms{};
};

/** A logical or physical byte range emitted while walking the contents of a lockbox. */
struct StreamChunk {
  /** Lockbox file path to which this byte range belongs. */
  std::string path{};
  /** Logical byte offset within the file. */
  std::uint64_t file_offset{};
  /** Logical range length in bytes. */
  std::uint64_t length{};
  /** Archive byte offset, when physical streaming is requested. */
  std::uint64_t physical_offset{};
  /** Whether the range represents a sparse zero-filled extent. */
  bool sparse{};
  /** File bytes for a populated logical range. */
  std::vector<std::uint8_t> data{};
};

/** The workload and worker policies currently applied to an open lockbox. */
struct RuntimeOptions {
  /** I/O workload policy used to tune page access. */
  std::string workload_profile{};
  /** Worker scheduling policy and effective parallelism. */
  std::string worker_policy{};
};

/** The name and sensitivity classification of a variable stored in a lockbox. */
struct Variable {
  /** Name used to address the variable in the lockbox. */
  std::string name{};
  /** Whether the value is ordinary text or a protected secret. */
  std::string sensitivity{};
};

/** Whether a lockbox is owner-signed and, when available, the signing-key fingerprint. */
struct OwnerInspection {
  /** Whether the lockbox requires owner-signed commits. */
  bool is_signed{};
  /** Owner signing-key fingerprint when one is configured. */
  std::string fingerprint{};
  /** Whether an owner fingerprint is available. */
  bool has_fingerprint{};
};

/** A named recipient public key stored in the local vault address book. */
struct Contact {
  /** Local address-book name of the contact. */
  std::string name{};
  /** Serialized contact public key used to grant lockbox access. */
  std::vector<std::uint8_t> key{};
};

/** A lockbox key currently held by the local session agent, identified by lockbox and path. */
struct AgentEntry {
  /** Stable lockbox identifier for the cached key. */
  std::string id{};
  /** Host path associated with the cached lockbox key. */
  std::string path{};
};

/** The host capabilities used to protect cached secrets across suspend and sleep. */
struct SleepSupport {
  /** Whether the host reports impending system suspend. */
  bool suspend_notifications{};
  /** Whether the agent can delay sleep while handling secrets. */
  bool sleep_inhibition{};
  /** Whether the host supplies enough integration for safe caching. */
  bool supported{};
};

/** Availability and configuration of the operating-system credential store used for the vault password. */
struct PlatformStatus {
  /** Whether a usable operating-system credential store exists. */
  bool supported{};
  /** Whether the user disabled credential-store integration. */
  bool disabled{};
  /** Application-specific scope used to isolate the stored password. */
  std::string scope{};
  /** Operating-system credential-store backend in use. */
  std::string backend{};
  /** Credential item name used by the backend. */
  std::string item{};
};

/** The version, size, checksum, and creation time of an exported local-vault backup. */
struct VaultBackupManifest {
  /** Backup container format version. */
  std::uint32_t format_version{};
  /** Backup creation time in Unix milliseconds. */
  std::uint64_t created_at_unix_ms{};
  /** Metadata-vault filename stored in the backup. */
  std::string vault_file_name{};
  /** Encrypted vault payload size in bytes. */
  std::uint64_t vault_size{};
  /** Lowercase SHA-256 digest of the encrypted vault payload. */
  std::string vault_sha256{};
};

/** Structured category, version, guidance, and artifact context for the most recent native failure. */
struct ErrorDetails {
  /** Stable error category suitable for programmatic handling. */
  std::string category{};
  /** Kind of archive or vault artifact involved in the failure. */
  std::string artifact_kind{};
  /** Format version read from the failing artifact. */
  std::uint32_t found_version{};
  /** Newest format version supported by this library. */
  std::uint32_t supported_version{};
  /** Human-readable explanation of the failure. */
  std::string message{};
  /** Suggested corrective action for the caller or user. */
  std::string guidance{};
};

using StreamChunkList = std::vector<StreamChunk>;
using PageInspectionList = std::vector<PageInspection>;
using LockboxEntryList = std::vector<LockboxEntry>;
using VariableList = std::vector<Variable>;
using KeySlotList = std::vector<KeySlot>;
using FormDefinitionList = std::vector<FormDefinition>;
using FormRecordList = std::vector<FormRecord>;
using ContactList = std::vector<Contact>;
using KnownLockboxList = std::vector<KnownLockbox>;
using AccessSlotLabelList = std::vector<AccessSlotLabel>;
using AgentEntryList = std::vector<AgentEntry>;
using ProfileHistoryList = std::vector<ProfileHistory>;
using StringList = std::vector<std::string>;
using OptionalLockboxEntry = std::optional<LockboxEntry>;
using OptionalFormRecord = std::optional<FormRecord>;
using OptionalFormValue = std::optional<FormValue>;
using OptionalString = std::optional<std::string>;
using PathMoveList = std::vector<PathMove>;
using FormFieldList = std::vector<FormField>;

}  // namespace revault::bindings

namespace revault::detail {

template <typename T> T decode(std::span<const std::uint8_t> bytes);

template <> bindings::LockboxEntry decode<bindings::LockboxEntry>(std::span<const std::uint8_t> bytes);
template <> bindings::PathMove decode<bindings::PathMove>(std::span<const std::uint8_t> bytes);
template <> bindings::FormField decode<bindings::FormField>(std::span<const std::uint8_t> bytes);
template <> bindings::FormDefinition decode<bindings::FormDefinition>(std::span<const std::uint8_t> bytes);
template <> bindings::FormValue decode<bindings::FormValue>(std::span<const std::uint8_t> bytes);
template <> bindings::FormRecord decode<bindings::FormRecord>(std::span<const std::uint8_t> bytes);
template <> bindings::RecoveryReport decode<bindings::RecoveryReport>(std::span<const std::uint8_t> bytes);
template <> bindings::KeySlot decode<bindings::KeySlot>(std::span<const std::uint8_t> bytes);
template <> bindings::CacheStats decode<bindings::CacheStats>(std::span<const std::uint8_t> bytes);
template <> bindings::ImportStats decode<bindings::ImportStats>(std::span<const std::uint8_t> bytes);
template <> bindings::PageObject decode<bindings::PageObject>(std::span<const std::uint8_t> bytes);
template <> bindings::PageInspection decode<bindings::PageInspection>(std::span<const std::uint8_t> bytes);
template <> bindings::FileInspection decode<bindings::FileInspection>(std::span<const std::uint8_t> bytes);
template <> bindings::ProfileGeneration decode<bindings::ProfileGeneration>(std::span<const std::uint8_t> bytes);
template <> bindings::ProfileHistory decode<bindings::ProfileHistory>(std::span<const std::uint8_t> bytes);
template <> bindings::KnownLockbox decode<bindings::KnownLockbox>(std::span<const std::uint8_t> bytes);
template <> bindings::AccessSlotLabel decode<bindings::AccessSlotLabel>(std::span<const std::uint8_t> bytes);
template <> bindings::StreamChunk decode<bindings::StreamChunk>(std::span<const std::uint8_t> bytes);
template <> bindings::RuntimeOptions decode<bindings::RuntimeOptions>(std::span<const std::uint8_t> bytes);
template <> bindings::Variable decode<bindings::Variable>(std::span<const std::uint8_t> bytes);
template <> bindings::OwnerInspection decode<bindings::OwnerInspection>(std::span<const std::uint8_t> bytes);
template <> bindings::Contact decode<bindings::Contact>(std::span<const std::uint8_t> bytes);
template <> bindings::AgentEntry decode<bindings::AgentEntry>(std::span<const std::uint8_t> bytes);
template <> bindings::SleepSupport decode<bindings::SleepSupport>(std::span<const std::uint8_t> bytes);
template <> bindings::PlatformStatus decode<bindings::PlatformStatus>(std::span<const std::uint8_t> bytes);
template <> bindings::VaultBackupManifest decode<bindings::VaultBackupManifest>(std::span<const std::uint8_t> bytes);
template <> bindings::ErrorDetails decode<bindings::ErrorDetails>(std::span<const std::uint8_t> bytes);
template <> bindings::StreamChunkList decode<bindings::StreamChunkList>(std::span<const std::uint8_t> bytes);
template <> bindings::PageInspectionList decode<bindings::PageInspectionList>(std::span<const std::uint8_t> bytes);
template <> bindings::LockboxEntryList decode<bindings::LockboxEntryList>(std::span<const std::uint8_t> bytes);
template <> bindings::VariableList decode<bindings::VariableList>(std::span<const std::uint8_t> bytes);
template <> bindings::KeySlotList decode<bindings::KeySlotList>(std::span<const std::uint8_t> bytes);
template <> bindings::FormDefinitionList decode<bindings::FormDefinitionList>(std::span<const std::uint8_t> bytes);
template <> bindings::FormRecordList decode<bindings::FormRecordList>(std::span<const std::uint8_t> bytes);
template <> bindings::ContactList decode<bindings::ContactList>(std::span<const std::uint8_t> bytes);
template <> bindings::KnownLockboxList decode<bindings::KnownLockboxList>(std::span<const std::uint8_t> bytes);
template <> bindings::AccessSlotLabelList decode<bindings::AccessSlotLabelList>(std::span<const std::uint8_t> bytes);
template <> bindings::AgentEntryList decode<bindings::AgentEntryList>(std::span<const std::uint8_t> bytes);
template <> bindings::ProfileHistoryList decode<bindings::ProfileHistoryList>(std::span<const std::uint8_t> bytes);
template <> bindings::StringList decode<bindings::StringList>(std::span<const std::uint8_t> bytes);
template <> bindings::OptionalLockboxEntry decode<bindings::OptionalLockboxEntry>(std::span<const std::uint8_t> bytes);
template <> bindings::OptionalFormRecord decode<bindings::OptionalFormRecord>(std::span<const std::uint8_t> bytes);
template <> bindings::OptionalFormValue decode<bindings::OptionalFormValue>(std::span<const std::uint8_t> bytes);
template <> bindings::OptionalString decode<bindings::OptionalString>(std::span<const std::uint8_t> bytes);

std::string encode_moves(const bindings::PathMoveList& values);
std::string encode_fields(const bindings::FormFieldList& values);

}  // namespace revault::detail
