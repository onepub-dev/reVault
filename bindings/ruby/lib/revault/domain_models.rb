# frozen_string_literal: true

module Revault
  module Bindings
    # Metadata for one file, directory, or symbolic link stored at a lockbox path.
    class LockboxEntry
      # Absolute lockbox path of the stored entry.
      attr_reader :path
      # Filesystem kind: file, directory, or symbolic link.
      attr_reader :kind
      # Logical file length in bytes; zero for directories.
      attr_reader :length
      # Portable Unix permission bits stored with the entry.
      attr_reader :permissions
      # Creates a domain value from named fields.
      def initialize(path: '', kind: 0, length: 0, permissions: 0) = (@path = path; @kind = kind; @length = length; @permissions = permissions)
    end

    # Ordered lockbox entries selected by a list operation.
    class LockboxEntryList
      # Ordered lockbox entries selected by a list operation.
      attr_reader :entries
      # Creates a domain value from named fields.
      def initialize(entries: []) = (@entries = entries)
    end

    # The metadata found for a lockbox path, or no value when the path is absent.
    class OptionalLockboxEntry
      # The metadata found for a lockbox path, or no value when the path is absent.
      attr_reader :value
      # Creates a domain value from named fields.
      def initialize(value: nil) = (@value = value)
    end

    # Ordered names or identifiers returned by a vault list operation.
    class StringList
      # Ordered names or identifiers returned by a vault list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # A source and destination pair used to rename a variable or form record atomically.
    class PathMove
      # Existing variable name or form-record path to rename.
      attr_reader :source
      # New variable name or form-record path.
      attr_reader :destination
      # Creates a domain value from named fields.
      def initialize(source: '', destination: '') = (@source = source; @destination = destination)
    end

    # Atomic variable or form-record renames supplied to a move operation.
    class PathMoveList
      # Atomic variable or form-record renames supplied to a move operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # One named input in a reusable form definition, including its display label and sensitivity kind.
    class FormField
      # Stable field identifier used when reading and writing records.
      attr_reader :id
      # Human-readable label presented to a person entering data.
      attr_reader :label
      # Field kind that determines validation and secret handling.
      attr_reader :kind
      # Whether a record must provide a value for this field.
      attr_reader :required
      # Creates a domain value from named fields.
      def initialize(id: '', label: '', kind: '', required: false) = (@id = id; @label = label; @kind = kind; @required = required)
    end

    # Ordered field definitions supplied when defining a form.
    class FormFieldList
      # Ordered field definitions supplied when defining a form.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # A versioned form schema used to validate and label structured records in a lockbox.
    class FormDefinition
      # Stable identifier shared by every revision of this form type.
      attr_reader :type_id
      # Short name used to resolve the current form revision.
      attr_reader :form_alias
      # Monotonically increasing revision number.
      attr_reader :revision
      # Human-readable name shown for this form.
      attr_reader :name
      # Explanation shown to people completing the form.
      attr_reader :description
      # Ordered inputs accepted by this form revision.
      attr_reader :fields
      # Creates a domain value from named fields.
      def initialize(type_id: '', form_alias: '', revision: 0, name: '', description: '', fields: []) = (@type_id = type_id; @form_alias = form_alias; @revision = revision; @name = name; @description = description; @fields = fields)
    end

    # Ordered FormDefinition values returned by the corresponding list operation.
    class FormDefinitionList
      # Ordered FormDefinition values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # The current value and sensitivity metadata for one field in a stored form record.
    class FormValue
      # Identifier of the form field that owns this value.
      attr_reader :field_id
      # Display label captured from the form revision.
      attr_reader :label
      # Field kind captured from the form revision.
      attr_reader :kind
      # Plain value, or an empty string when the field is secret.
      attr_reader :value
      # Whether the value must be read through a scoped secret callback.
      attr_reader :secret
      # Creates a domain value from named fields.
      def initialize(field_id: '', label: '', kind: '', value: '', secret: false) = (@field_id = field_id; @label = label; @kind = kind; @value = value; @secret = secret)
    end

    # A named structured record stored at a lockbox path and tied to a form-definition revision.
    class FormRecord
      # Absolute lockbox path that identifies the record.
      attr_reader :path
      # Human-readable name assigned to this record.
      attr_reader :name
      # Stable identifier of the record's form type.
      attr_reader :type_id
      # Alias of the form definition used by the record.
      attr_reader :definition_alias
      # Exact form revision against which the record was created.
      attr_reader :definition_revision
      # Ordered non-secret field metadata and values.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(path: '', name: '', type_id: '', definition_alias: '', definition_revision: 0, values: []) = (@path = path; @name = name; @type_id = type_id; @definition_alias = definition_alias; @definition_revision = definition_revision; @values = values)
    end

    # Ordered FormRecord values returned by the corresponding list operation.
    class FormRecordList
      # Ordered FormRecord values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # The form record found at a lockbox path, or no value when absent.
    class OptionalFormRecord
      # The form record found at a lockbox path, or no value when absent.
      attr_reader :value
      # Creates a domain value from named fields.
      def initialize(value: nil) = (@value = value)
    end

    # The requested non-secret form value, or no value when the field is absent.
    class OptionalFormValue
      # The requested non-secret form value, or no value when the field is absent.
      attr_reader :value
      # Creates a domain value from named fields.
      def initialize(value: nil) = (@value = value)
    end

    # The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox.
    class RecoveryReport
      # Files whose complete contents remain recoverable.
      attr_reader :intact_files
      # Number of completely recoverable files.
      attr_reader :intact_file_count
      # Number of files for which only some content is recoverable.
      attr_reader :partial_files
      # Number of encrypted records that failed validation.
      attr_reader :corrupt_records
      # Whether a usable table of contents was recovered.
      attr_reader :toc_recovered
      # Whether variable metadata was recovered.
      attr_reader :variables_recovered
      # Number of recovered variables.
      attr_reader :variable_count
      # Whether form definitions and records were recovered.
      attr_reader :forms_recovered
      # Number of recovered form definitions.
      attr_reader :form_definition_count
      # Number of recovered form records.
      attr_reader :form_record_count
      # Creates a domain value from named fields.
      def initialize(intact_files: [], intact_file_count: 0, partial_files: 0, corrupt_records: 0, toc_recovered: false, variables_recovered: false, variable_count: 0, forms_recovered: false, form_definition_count: 0, form_record_count: 0) = (@intact_files = intact_files; @intact_file_count = intact_file_count; @partial_files = partial_files; @corrupt_records = corrupt_records; @toc_recovered = toc_recovered; @variables_recovered = variables_recovered; @variable_count = variable_count; @forms_recovered = forms_recovered; @form_definition_count = form_definition_count; @form_record_count = form_record_count)
    end

    # One password or contact credential that can unlock a lockbox content key.
    class KeySlot
      # Stable slot identifier used when removing this access method.
      attr_reader :id
      # Access method, such as password or contact key.
      attr_reader :protection
      # Cryptographic algorithm protecting the content key.
      attr_reader :algorithm
      # Creates a domain value from named fields.
      def initialize(id: 0, protection: '', algorithm: '') = (@id = id; @protection = protection; @algorithm = algorithm)
    end

    # Ordered KeySlot values returned by the corresponding list operation.
    class KeySlotList
      # Ordered KeySlot values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # Current capacity, occupancy, hit, and miss counters for an open lockbox cache.
    class CacheStats
      # Maximum decoded-page memory permitted for the cache.
      attr_reader :limit_bytes
      # Decoded-page memory currently held by the cache.
      attr_reader :used_bytes
      # Number of decoded pages currently cached.
      attr_reader :entries
      # Reads served by an already decoded page.
      attr_reader :hits
      # Reads that required decoding another page.
      attr_reader :misses
      # Creates a domain value from named fields.
      def initialize(limit_bytes: 0, used_bytes: 0, entries: 0, hits: 0, misses: 0) = (@limit_bytes = limit_bytes; @used_bytes = used_bytes; @entries = entries; @hits = hits; @misses = misses)
    end

    # Time spent reading host files and preparing encrypted pages during the latest import work.
    class ImportStats
      # Nanoseconds spent reading host filesystem metadata, as decimal text.
      attr_reader :host_stat_nanos
      # Nanoseconds spent reading host file content, as decimal text.
      attr_reader :host_read_nanos
      # Nanoseconds spent preparing encrypted records, as decimal text.
      attr_reader :frame_prepare_nanos
      # Nanoseconds spent writing encrypted pages, as decimal text.
      attr_reader :page_write_nanos
      # Creates a domain value from named fields.
      def initialize(host_stat_nanos: '', host_read_nanos: '', frame_prepare_nanos: '', page_write_nanos: '') = (@host_stat_nanos = host_stat_nanos; @host_read_nanos = host_read_nanos; @frame_prepare_nanos = frame_prepare_nanos; @page_write_nanos = page_write_nanos)
    end

    # One logical object recorded inside an inspected encrypted lockbox page.
    class PageObject
      # Object identifier recorded in the encrypted page.
      attr_reader :id
      # Kind of logical object stored in the page.
      attr_reader :kind
      # Encrypted object payload length in bytes.
      attr_reader :payload_len
      # Creates a domain value from named fields.
      def initialize(id: 0, kind: '', payload_len: 0) = (@id = id; @kind = kind; @payload_len = payload_len)
    end

    # Layout and utilization details for one encrypted page in a lockbox archive.
    class PageInspection
      # Byte offset at which the page begins in the archive.
      attr_reader :offset
      # Identifier stored in the page header.
      attr_reader :page_id
      # Commit sequence that wrote this page.
      attr_reader :sequence
      # Total encoded page size in bytes.
      attr_reader :page_size
      # Encrypted body length in bytes.
      attr_reader :encrypted_body_len
      # Unused capacity remaining in the page.
      attr_reader :unused_bytes
      # Number of logical objects stored in the page.
      attr_reader :object_count
      # Logical objects discovered in the page.
      attr_reader :objects
      # Creates a domain value from named fields.
      def initialize(offset: 0, page_id: 0, sequence: 0, page_size: 0, encrypted_body_len: 0, unused_bytes: 0, object_count: 0, objects: []) = (@offset = offset; @page_id = page_id; @sequence = sequence; @page_size = page_size; @encrypted_body_len = encrypted_body_len; @unused_bytes = unused_bytes; @object_count = object_count; @objects = objects)
    end

    # Ordered PageInspection values returned by the corresponding list operation.
    class PageInspectionList
      # Ordered PageInspection values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # Header, owner-signature, and key-slot information read from a lockbox file without opening its contents.
    class FileInspection
      # Stable binary identifier read from the lockbox header.
      attr_reader :lockbox_id
      # Whether the archive header passed structural validation.
      attr_reader :header_readable
      # Latest readable access-key directory generation.
      attr_reader :key_directory_generation
      # Number of readable redundant key-directory copies.
      attr_reader :key_directory_copy_count
      # Whether commits require an owner signature.
      attr_reader :owner_signed
      # Password and contact access methods found in the header.
      attr_reader :key_slots
      # Creates a domain value from named fields.
      def initialize(lockbox_id: [], header_readable: false, key_directory_generation: 0, key_directory_copy_count: 0, owner_signed: false, key_slots: []) = (@lockbox_id = lockbox_id; @header_readable = header_readable; @key_directory_generation = key_directory_generation; @key_directory_copy_count = key_directory_copy_count; @owner_signed = owner_signed; @key_slots = key_slots)
    end

    # One active or retired generation of the contact keys belonging to a named vault profile.
    class ProfileGeneration
      # Generation number used to address this key version.
      attr_reader :index
      # Lifecycle state, such as active or retired.
      attr_reader :status
      # Fingerprint of this generation's contact public key.
      attr_reader :contact_fingerprint
      # Creation time in Unix milliseconds.
      attr_reader :created_at_unix_ms
      # Retirement time in Unix milliseconds when retired.
      attr_reader :retired_at_unix_ms
      # Whether a retirement time is present.
      attr_reader :has_retired_at
      # Creates a domain value from named fields.
      def initialize(index: 0, status: '', contact_fingerprint: [], created_at_unix_ms: 0, retired_at_unix_ms: 0, has_retired_at: false) = (@index = index; @status = status; @contact_fingerprint = contact_fingerprint; @created_at_unix_ms = created_at_unix_ms; @retired_at_unix_ms = retired_at_unix_ms; @has_retired_at = has_retired_at)
    end

    # The active generation and rotation history for a named vault profile.
    class ProfileHistory
      # Vault profile name whose generations are listed.
      attr_reader :name
      # Generation number currently used for new access grants.
      attr_reader :active_generation
      # Active and retired contact-key generations.
      attr_reader :generations
      # Creates a domain value from named fields.
      def initialize(name: '', active_generation: 0, generations: []) = (@name = name; @active_generation = active_generation; @generations = generations)
    end

    # A lockbox identifier and host path remembered by the local vault for later discovery.
    class KnownLockbox
      # Stable binary identifier of the remembered lockbox.
      attr_reader :lockbox_id
      # Last known host filesystem path of the lockbox.
      attr_reader :path
      # Most recent observation time in Unix milliseconds.
      attr_reader :last_seen_unix_ms
      # Creates a domain value from named fields.
      def initialize(lockbox_id: [], path: '', last_seen_unix_ms: 0) = (@lockbox_id = lockbox_id; @path = path; @last_seen_unix_ms = last_seen_unix_ms)
    end

    # Ordered KnownLockbox values returned by the corresponding list operation.
    class KnownLockboxList
      # Ordered KnownLockbox values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # A local human-readable label attached to one lockbox access slot.
    class AccessSlotLabel
      # Lockbox whose access slot is labelled.
      attr_reader :lockbox_id
      # Stable identifier of the labelled access slot.
      attr_reader :slot_id
      # Local human-readable label for the access slot.
      attr_reader :name
      # Last label update time in Unix milliseconds.
      attr_reader :updated_at_unix_ms
      # Creates a domain value from named fields.
      def initialize(lockbox_id: [], slot_id: 0, name: '', updated_at_unix_ms: 0) = (@lockbox_id = lockbox_id; @slot_id = slot_id; @name = name; @updated_at_unix_ms = updated_at_unix_ms)
    end

    # Ordered AccessSlotLabel values returned by the corresponding list operation.
    class AccessSlotLabelList
      # Ordered AccessSlotLabel values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # A logical or physical byte range emitted while walking the contents of a lockbox.
    class StreamChunk
      # Lockbox file path to which this byte range belongs.
      attr_reader :path
      # Logical byte offset within the file.
      attr_reader :file_offset
      # Logical range length in bytes.
      attr_reader :length
      # Archive byte offset, when physical streaming is requested.
      attr_reader :physical_offset
      # Whether the range represents a sparse zero-filled extent.
      attr_reader :sparse
      # File bytes for a populated logical range.
      attr_reader :data
      # Creates a domain value from named fields.
      def initialize(path: '', file_offset: 0, length: 0, physical_offset: 0, sparse: false, data: []) = (@path = path; @file_offset = file_offset; @length = length; @physical_offset = physical_offset; @sparse = sparse; @data = data)
    end

    # Ordered StreamChunk values returned by the corresponding list operation.
    class StreamChunkList
      # Ordered StreamChunk values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # The workload and worker policies currently applied to an open lockbox.
    class RuntimeOptions
      # I/O workload policy used to tune page access.
      attr_reader :workload_profile
      # Worker scheduling policy and effective parallelism.
      attr_reader :worker_policy
      # Creates a domain value from named fields.
      def initialize(workload_profile: '', worker_policy: '') = (@workload_profile = workload_profile; @worker_policy = worker_policy)
    end

    # The name and sensitivity classification of a variable stored in a lockbox.
    class Variable
      # Name used to address the variable in the lockbox.
      attr_reader :name
      # Whether the value is ordinary text or a protected secret.
      attr_reader :sensitivity
      # Creates a domain value from named fields.
      def initialize(name: '', sensitivity: '') = (@name = name; @sensitivity = sensitivity)
    end

    # Ordered Variable values returned by the corresponding list operation.
    class VariableList
      # Ordered Variable values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # A text lookup that distinguishes an absent value from an empty string.
    class OptionalString
      # A text lookup that distinguishes an absent value from an empty string.
      attr_reader :present
      # A text lookup that distinguishes an absent value from an empty string.
      attr_reader :value
      # Creates a domain value from named fields.
      def initialize(present: false, value: '') = (@present = present; @value = value)
    end

    # Whether a lockbox is owner-signed and, when available, the signing-key fingerprint.
    class OwnerInspection
      # Whether the lockbox requires owner-signed commits.
      attr_reader :signed
      # Owner signing-key fingerprint when one is configured.
      attr_reader :fingerprint
      # Whether an owner fingerprint is available.
      attr_reader :has_fingerprint
      # Creates a domain value from named fields.
      def initialize(signed: false, fingerprint: '', has_fingerprint: false) = (@signed = signed; @fingerprint = fingerprint; @has_fingerprint = has_fingerprint)
    end

    # A named recipient public key stored in the local vault address book.
    class Contact
      # Local address-book name of the contact.
      attr_reader :name
      # Serialized contact public key used to grant lockbox access.
      attr_reader :key
      # Creates a domain value from named fields.
      def initialize(name: '', key: []) = (@name = name; @key = key)
    end

    # Ordered Contact values returned by the corresponding list operation.
    class ContactList
      # Ordered Contact values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # Ordered ProfileHistory values returned by the corresponding list operation.
    class ProfileHistoryList
      # Ordered ProfileHistory values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # A lockbox key currently held by the local session agent, identified by lockbox and path.
    class AgentEntry
      # Stable lockbox identifier for the cached key.
      attr_reader :id
      # Host path associated with the cached lockbox key.
      attr_reader :path
      # Creates a domain value from named fields.
      def initialize(id: '', path: '') = (@id = id; @path = path)
    end

    # Ordered AgentEntry values returned by the corresponding list operation.
    class AgentEntryList
      # Ordered AgentEntry values returned by the corresponding list operation.
      attr_reader :values
      # Creates a domain value from named fields.
      def initialize(values: []) = (@values = values)
    end

    # The host capabilities used to protect cached secrets across suspend and sleep.
    class SleepSupport
      # Whether the host reports impending system suspend.
      attr_reader :suspend_notifications
      # Whether the agent can delay sleep while handling secrets.
      attr_reader :sleep_inhibition
      # Whether the host supplies enough integration for safe caching.
      attr_reader :supported
      # Creates a domain value from named fields.
      def initialize(suspend_notifications: false, sleep_inhibition: false, supported: false) = (@suspend_notifications = suspend_notifications; @sleep_inhibition = sleep_inhibition; @supported = supported)
    end

    # Availability and configuration of the operating-system credential store used for the vault password.
    class PlatformStatus
      # Whether a usable operating-system credential store exists.
      attr_reader :supported
      # Whether the user disabled credential-store integration.
      attr_reader :disabled
      # Application-specific scope used to isolate the stored password.
      attr_reader :scope
      # Operating-system credential-store backend in use.
      attr_reader :backend
      # Credential item name used by the backend.
      attr_reader :item
      # Creates a domain value from named fields.
      def initialize(supported: false, disabled: false, scope: '', backend: '', item: '') = (@supported = supported; @disabled = disabled; @scope = scope; @backend = backend; @item = item)
    end

    # One text result returned by the native API.
    class StringValue
      # One text result returned by the native API.
      attr_reader :value
      # Creates a domain value from named fields.
      def initialize(value: '') = (@value = value)
    end

    # The version, size, checksum, and creation time of an exported local-vault backup.
    class VaultBackupManifest
      # Backup container format version.
      attr_reader :format_version
      # Backup creation time in Unix milliseconds.
      attr_reader :created_at_unix_ms
      # Metadata-vault filename stored in the backup.
      attr_reader :vault_file_name
      # Encrypted vault payload size in bytes.
      attr_reader :vault_size
      # Lowercase SHA-256 digest of the encrypted vault payload.
      attr_reader :vault_sha256
      # Creates a domain value from named fields.
      def initialize(format_version: 0, created_at_unix_ms: 0, vault_file_name: '', vault_size: 0, vault_sha256: '') = (@format_version = format_version; @created_at_unix_ms = created_at_unix_ms; @vault_file_name = vault_file_name; @vault_size = vault_size; @vault_sha256 = vault_sha256)
    end

    # Structured category, version, guidance, and artifact context for the most recent native failure.
    class ErrorDetails
      # Stable error category suitable for programmatic handling.
      attr_reader :category
      # Kind of archive or vault artifact involved in the failure.
      attr_reader :artifact_kind
      # Format version read from the failing artifact.
      attr_reader :found_version
      # Newest format version supported by this library.
      attr_reader :supported_version
      # Human-readable explanation of the failure.
      attr_reader :message
      # Suggested corrective action for the caller or user.
      attr_reader :guidance
      # Creates a domain value from named fields.
      def initialize(category: '', artifact_kind: '', found_version: 0, supported_version: 0, message: '', guidance: '') = (@category = category; @artifact_kind = artifact_kind; @found_version = found_version; @supported_version = supported_version; @message = message; @guidance = guidance)
    end

  end

  module Internal
    # FlatBuffer reader and input encoder used only by the FFI layer.
    module DomainCodec
      module_function

    SCHEMA = {
      'LockboxEntry' => [[:path, 'string'], [:kind, 'revault.internal.LockboxEntryKind'], [:length, 'ulong'], [:permissions, 'uint']],
      'LockboxEntryList' => [[:entries, '[revault.internal.LockboxEntry]']],
      'OptionalLockboxEntry' => [[:value, 'revault.internal.LockboxEntry']],
      'StringList' => [[:values, '[string]']],
      'PathMove' => [[:source, 'string'], [:destination, 'string']],
      'PathMoveList' => [[:values, '[revault.internal.PathMove]']],
      'FormField' => [[:id, 'string'], [:label, 'string'], [:kind, 'string'], [:required, 'bool']],
      'FormFieldList' => [[:values, '[revault.internal.FormField]']],
      'FormDefinition' => [[:type_id, 'string'], [:alias, 'string'], [:revision, 'uint'], [:name, 'string'], [:description, 'string'], [:fields, '[revault.internal.FormField]']],
      'FormDefinitionList' => [[:values, '[revault.internal.FormDefinition]']],
      'FormValue' => [[:field_id, 'string'], [:label, 'string'], [:kind, 'string'], [:value, 'string'], [:secret, 'bool']],
      'FormRecord' => [[:path, 'string'], [:name, 'string'], [:type_id, 'string'], [:definition_alias, 'string'], [:definition_revision, 'uint'], [:values, '[revault.internal.FormValue]']],
      'FormRecordList' => [[:values, '[revault.internal.FormRecord]']],
      'OptionalFormRecord' => [[:value, 'revault.internal.FormRecord']],
      'OptionalFormValue' => [[:value, 'revault.internal.FormValue']],
      'RecoveryReport' => [[:intact_files, '[revault.internal.LockboxEntry]'], [:intact_file_count, 'ulong'], [:partial_files, 'ulong'], [:corrupt_records, 'ulong'], [:toc_recovered, 'bool'], [:variables_recovered, 'bool'], [:variable_count, 'ulong'], [:forms_recovered, 'bool'], [:form_definition_count, 'ulong'], [:form_record_count, 'ulong']],
      'KeySlot' => [[:id, 'ulong'], [:protection, 'string'], [:algorithm, 'string']],
      'KeySlotList' => [[:values, '[revault.internal.KeySlot]']],
      'CacheStats' => [[:limit_bytes, 'ulong'], [:used_bytes, 'ulong'], [:entries, 'ulong'], [:hits, 'ulong'], [:misses, 'ulong']],
      'ImportStats' => [[:host_stat_nanos, 'string'], [:host_read_nanos, 'string'], [:frame_prepare_nanos, 'string'], [:page_write_nanos, 'string']],
      'PageObject' => [[:id, 'ulong'], [:kind, 'string'], [:payload_len, 'ulong']],
      'PageInspection' => [[:offset, 'ulong'], [:page_id, 'ulong'], [:sequence, 'ulong'], [:page_size, 'ulong'], [:encrypted_body_len, 'ulong'], [:unused_bytes, 'ulong'], [:object_count, 'ulong'], [:objects, '[revault.internal.PageObject]']],
      'PageInspectionList' => [[:values, '[revault.internal.PageInspection]']],
      'FileInspection' => [[:lockbox_id, '[ubyte]'], [:header_readable, 'bool'], [:key_directory_generation, 'ulong'], [:key_directory_copy_count, 'ulong'], [:owner_signed, 'bool'], [:key_slots, '[revault.internal.KeySlot]']],
      'ProfileGeneration' => [[:index, 'uint'], [:status, 'string'], [:contact_fingerprint, '[ubyte]'], [:created_at_unix_ms, 'ulong'], [:retired_at_unix_ms, 'ulong'], [:has_retired_at, 'bool']],
      'ProfileHistory' => [[:name, 'string'], [:active_generation, 'uint'], [:generations, '[revault.internal.ProfileGeneration]']],
      'KnownLockbox' => [[:lockbox_id, '[ubyte]'], [:path, 'string'], [:last_seen_unix_ms, 'ulong']],
      'KnownLockboxList' => [[:values, '[revault.internal.KnownLockbox]']],
      'AccessSlotLabel' => [[:lockbox_id, '[ubyte]'], [:slot_id, 'ulong'], [:name, 'string'], [:updated_at_unix_ms, 'ulong']],
      'AccessSlotLabelList' => [[:values, '[revault.internal.AccessSlotLabel]']],
      'StreamChunk' => [[:path, 'string'], [:file_offset, 'ulong'], [:length, 'ulong'], [:physical_offset, 'ulong'], [:sparse, 'bool'], [:data, '[ubyte]']],
      'StreamChunkList' => [[:values, '[revault.internal.StreamChunk]']],
      'RuntimeOptions' => [[:workload_profile, 'string'], [:worker_policy, 'string']],
      'Variable' => [[:name, 'string'], [:sensitivity, 'string']],
      'VariableList' => [[:values, '[revault.internal.Variable]']],
      'OptionalString' => [[:present, 'bool'], [:value, 'string']],
      'OwnerInspection' => [[:signed, 'bool'], [:fingerprint, 'string'], [:has_fingerprint, 'bool']],
      'Contact' => [[:name, 'string'], [:key, '[ubyte]']],
      'ContactList' => [[:values, '[revault.internal.Contact]']],
      'ProfileHistoryList' => [[:values, '[revault.internal.ProfileHistory]']],
      'AgentEntry' => [[:id, 'string'], [:path, 'string']],
      'AgentEntryList' => [[:values, '[revault.internal.AgentEntry]']],
      'SleepSupport' => [[:suspend_notifications, 'bool'], [:sleep_inhibition, 'bool'], [:supported, 'bool']],
      'PlatformStatus' => [[:supported, 'bool'], [:disabled, 'bool'], [:scope, 'string'], [:backend, 'string'], [:item, 'string']],
      'StringValue' => [[:value, 'string']],
      'VaultBackupManifest' => [[:format_version, 'uint'], [:created_at_unix_ms, 'ulong'], [:vault_file_name, 'string'], [:vault_size, 'ulong'], [:vault_sha256, 'string']],
      'ErrorDetails' => [[:category, 'string'], [:artifact_kind, 'string'], [:found_version, 'uint'], [:supported_version, 'uint'], [:message, 'string'], [:guidance, 'string']]
    }.freeze

    def decode(name, bytes)
      table(name, bytes.b, bytes.unpack1('V'))
    end

    def table(name, bytes, position)
      values = {}
      SCHEMA.fetch(name).each_with_index do |(field, type), index|
        location = field_location(bytes, position, index)
        public_field = field == :alias ? :form_alias : field
        values[public_field] = read_value(bytes, location, type)
      end
      Bindings.const_get(name).new(**values)
    end

    def field_location(bytes, table_position, index)
      vtable = table_position - bytes.unpack1('l<', offset: table_position)
      vtable_length = bytes.unpack1('v', offset: vtable)
      entry = vtable + 4 + (index * 2)
      return nil if entry + 2 > vtable + vtable_length
      offset = bytes.unpack1('v', offset: entry)
      offset.zero? ? nil : table_position + offset
    end

    def read_value(bytes, location, type)
      return default_value(type) unless location
      if type.start_with?('[')
        read_vector(bytes, location, type[1...-1])
      elsif SCHEMA.key?(type.split('.').last)
        table(type.split('.').last, bytes, location + bytes.unpack1('V', offset: location))
      else
        read_scalar(bytes, location, type)
      end
    end

    def read_vector(bytes, location, type)
      vector = location + bytes.unpack1('V', offset: location)
      length = bytes.unpack1('V', offset: vector)
      start = vector + 4
      return bytes.byteslice(start, length) if type == 'ubyte'
      Array.new(length) do |index|
        element = start + (index * 4)
        if type == 'string'
          read_string(bytes, element)
        elsif SCHEMA.key?(type.split('.').last)
          table(type.split('.').last, bytes, element + bytes.unpack1('V', offset: element))
        else
          read_scalar(bytes, element, type)
        end
      end
    end

    def read_scalar(bytes, location, type)
      case type
      when 'string' then read_string(bytes, location)
      when 'bool', 'ubyte' then bytes.getbyte(location) != 0
      when 'ulong' then bytes.unpack1('Q<', offset: location)
      else bytes.unpack1('V', offset: location)
      end
    end

    def read_string(bytes, location)
      value = location + bytes.unpack1('V', offset: location)
      bytes.byteslice(value + 4, bytes.unpack1('V', offset: value)).force_encoding(Encoding::UTF_8)
    end

    def default_value(type)
      return [] if type.start_with?('[')
      return '' if type == 'string'
      return false if type == 'bool'
      return 0 if %w[uint ulong ubyte].include?(type) || type.end_with?('LockboxEntryKind')
      nil
    end

    def encode_path_moves(values) = encode_table_vector(values, :path_move)
    def encode_form_fields(values) = encode_table_vector(values, :form_field)

    def encode_table_vector(values, kind)
      bytes = [0].pack('V')
      append_vtable(bytes, [4], 8)
      root = align(bytes); bytes << [root - 4, 0].pack('V2')
      patch_u32(bytes, 0, root)
      vector = align(bytes); patch_u32(bytes, root + 4, vector - (root + 4)); bytes << [values.length].pack('V') << ("\0" * (values.length * 4))
      values.each_with_index do |value, index|
        position = kind == :path_move ? append_path_move(bytes, value) : append_form_field(bytes, value)
        element = vector + 4 + (index * 4); patch_u32(bytes, element, position - element)
      end
      bytes
    end

    def append_path_move(bytes, value)
      vtable = append_vtable(bytes, [4, 8], 12); table = align(bytes); bytes << [table - vtable, 0, 0].pack('V3')
      append_string_field(bytes, table + 4, value.source); append_string_field(bytes, table + 8, value.destination); table
    end

    def append_form_field(bytes, value)
      vtable = append_vtable(bytes, [4, 8, 12, 16], 20); table = align(bytes); bytes << [table - vtable, 0, 0, 0].pack('V4') << [value.required ? 1 : 0].pack('C') << "\0\0\0"
      append_string_field(bytes, table + 4, value.id); append_string_field(bytes, table + 8, value.label); append_string_field(bytes, table + 12, value.kind); table
    end

    def append_vtable(bytes, offsets, object_size)
      align(bytes, 2); position = bytes.bytesize; bytes << [4 + (offsets.length * 2), object_size, *offsets].pack('v*'); position
    end

    def append_string_field(bytes, field, value)
      target = align(bytes); patch_u32(bytes, field, target - field); bytes << [value.bytesize].pack('V') << value.b << "\0"
    end

    def align(bytes, alignment = 4)
      bytes << "\0" until (bytes.bytesize % alignment).zero?; bytes.bytesize
    end

    def patch_u32(bytes, offset, value) = bytes[offset, 4] = [value].pack('V')
    end
  end
end
