# The owned, class-oriented reVault API. See the
# {repository README}[https://github.com/onepub-dev/reVault#readme] for the
# security model and complete examples.
require_relative 'native_library'

# Encrypts lockbox content and manages local vault metadata.
module Revault
  # Load an explicit, inherited, or installed native carrier.
  def self.load(native_library_path = nil) = Runtime.new(native_library_path)
  # Return a new runtime facade synchronously for factory operations.
  def self.runtime = Runtime.new

  # Select the process-wide native carrier and load binding operations.
  def self.ensure_native(native_library_path = nil)
    NativeLibrary.path(native_library_path)
    unless const_defined?(:BindingOperations, false)
      require_relative 'binding_operations'
      private_constant(:BindingOperations)
    end
  end

  # Native operation failure with structured diagnostic details.
  class RevaultError < StandardError
    attr_reader :details
    # Preserve the stable native message and structured details.
    def initialize(message, details = nil)
      super(message); @details = details
    end
  end

  # Returns the owned handle.
  class OwnedHandle
    attr_reader :native_handle
    # Returns the initialize.
    def initialize(operations, native_handle)
      @operations = operations
      @native_handle = native_handle
    end
  end

  # Primary API used to open lockboxes, manage keys and metadata, use the
  # Session Agent, and access to the platform credential store.
  class Runtime
    attr_reader :agent, :platform
    # Returns the initialize.
    def initialize(native_library_path = nil)
      Revault.ensure_native(native_library_path)
      @operations = BindingOperations.new
      @agent = Agent.new(@operations)
      @platform = Platform.new(@operations)
    end
    # Returns the last error.
    def last_error = @operations.last_error_message
    # Returns the last error details.
    def last_error_details = @operations.buffer_last_error_details()

    # Returns the newest Lockbox archive format version supported by this engine.
    def lockbox_format_version()
      @operations.lockbox_format_version()
    end

    # Reads the format version from serialized Lockbox bytes without opening them.
    def lockbox_probe_format_version(bytes)
      @operations.lockbox_probe_format_version(bytes)
    end

    # Creates an in memory Lockbox protected by a 32 byte content key.
    def lockbox_create(key)
      Lockbox.new(@operations, @operations.lockbox_create(key))
    end

    # Creates a lockbox with explicit cache capacity, workload, worker policy, and job count.
    def lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
      Lockbox.new(@operations, @operations.lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs))
    end

    # Creates an in memory Lockbox protected by the supplied password.
    def lockbox_create_password(password)
      Lockbox.new(@operations, @operations.lockbox_create_password(password))
    end

    # Creates a password protected Lockbox with its profile signing key.
    def lockbox_create_password_with_signing_key(password, signing_key)
      Lockbox.new(@operations, @operations.lockbox_create_password_with_signing_key(password, signing_key.native_handle))
    end

    # Creates an in memory Lockbox that the supplied contact can open.
    def lockbox_create_contact(contact)
      Lockbox.new(@operations, @operations.lockbox_create_contact(contact.native_handle))
    end

    # Creates a contact protected Lockbox with its profile signing key.
    def lockbox_create_contact_with_signing_key(contact, signing_key)
      Lockbox.new(@operations, @operations.lockbox_create_contact_with_signing_key(contact.native_handle, signing_key.native_handle))
    end

    # Creates an in memory Lockbox and assigns its profile signing key.
    def lockbox_create_with_signing_key(content_key, signing_key)
      Lockbox.new(@operations, @operations.lockbox_create_with_signing_key(content_key, signing_key.native_handle))
    end

    # Opens serialized Lockbox bytes with a 32 byte content key.
    def lockbox_open(archive, key)
      Lockbox.new(@operations, @operations.lockbox_open(archive, key))
    end

    # Opens a lockbox with explicit cache capacity, workload, worker policy, and job count.
    def lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
      Lockbox.new(@operations, @operations.lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs))
    end

    # Opens serialized Lockbox bytes with the supplied password.
    def lockbox_open_password(archive, password)
      Lockbox.new(@operations, @operations.lockbox_open_password(archive, password))
    end

    # Opens serialized Lockbox bytes with the supplied contact private key.
    def lockbox_open_contact(archive, contact)
      Lockbox.new(@operations, @operations.lockbox_open_contact(archive, contact.native_handle))
    end

    # Reads public header, signature, and access slot metadata from a Lockbox file.
    def lockbox_inspect_file(path)
      @operations.lockbox_inspect_file(path)
    end

    # Scans a damaged Lockbox file with its 32 byte content key.
    def lockbox_recovery_scan_path(path, key)
      @operations.lockbox_recovery_scan_path(path, key)
    end

    # Scans damaged serialized Lockbox bytes with their 32 byte content key.
    def lockbox_recovery_scan(bytes, key)
      @operations.lockbox_recovery_scan(bytes, key)
    end

    # Builds a new Lockbox from recoverable records without changing the source.
    def lockbox_recovery_salvage(bytes, key, signing_key)
      Lockbox.new(@operations, @operations.lockbox_recovery_salvage(bytes, key, signing_key.native_handle))
    end

    # Generates a contact encryption key pair using secure random data.
    def key_contact_generate()
      ContactKeyPair.new(@operations, @operations.key_contact_generate())
    end

    # Imports a contact key pair from its private binary record.
    def key_contact_from_private(bytes)
      ContactKeyPair.new(@operations, @operations.key_contact_from_private(bytes))
    end

    # Imports a contact public key from its binary representation.
    def key_contact_public_from_bytes(bytes)
      ContactPublicKey.new(@operations, @operations.key_contact_public_from_bytes(bytes))
    end

    # Generates a signing identity owned by a Vault Profile.
    def generate_profile_signing_key_pair()
      ProfileSigningKeyPair.new(@operations, @operations.key_signing_generate())
    end

    # Imports a Vault Profile signing identity from its private record.
    def profile_signing_key_pair_from_private(bytes)
      ProfileSigningKeyPair.new(@operations, @operations.key_signing_from_private(bytes))
    end

    # Imports the public half of a Vault Profile signing identity.
    def profile_signing_public_key_from_bytes(bytes)
      ProfileSigningPublicKey.new(@operations, @operations.key_signing_public_from_bytes(bytes))
    end

    # Exports a private key in the requested key format.
    def vault_key_export_private(key, format)
      @operations.vault_key_export_private(key.native_handle, format)
    end

    # Exports a public key in the requested key format.
    def vault_key_export_public(key, format)
      @operations.vault_key_export_public(key.native_handle, format)
    end

    # Imports a private contact key from a detected supported encoding.
    def vault_key_import_private(bytes)
      ContactKeyPair.new(@operations, @operations.vault_key_import_private(bytes))
    end

    # Imports a public contact key from a detected supported encoding.
    def vault_key_import_public(bytes)
      ContactPublicKey.new(@operations, @operations.vault_key_import_public(bytes))
    end

    # Returns the stable fingerprint used to verify a public key.
    def vault_key_fingerprint(key)
      @operations.vault_key_fingerprint(key.native_handle)
    end

    # Encodes key bytes as hexadecimal text.
    def vault_key_format_hex(bytes)
      @operations.vault_key_format_hex(bytes)
    end

    # Decodes hexadecimal key text and rejects malformed input.
    def vault_key_decode_hex(text)
      @operations.vault_key_decode_hex(text)
    end

    # Encodes key bytes using Crockford Base32.
    def vault_key_format_crockford(bytes)
      @operations.vault_key_format_crockford(bytes)
    end

    # Groups a Crockford code for easier reading and transcription.
    def vault_key_format_crockford_reading(code)
      @operations.vault_key_format_crockford_reading(code)
    end

    # Decodes Crockford Base32 key text and rejects malformed input.
    def vault_key_decode_crockford(code)
      @operations.vault_key_decode_crockford(code)
    end

    # Encodes arbitrary bytes as hexadecimal text.
    def vault_key_hex_encode(bytes)
      @operations.vault_key_hex_encode(bytes)
    end

    # Decodes arbitrary hexadecimal text and rejects malformed input.
    def vault_key_hex_decode(text)
      @operations.vault_key_hex_decode(text)
    end

    # Opens an existing Vault directory with its passphrase.
    def vault_directory_open(root, password)
      Vault.new(@operations, @operations.vault_directory_open(root, password))
    end

    # Returns the newest Vault structure version supported by this engine.
    def vault_structure_version_current()
      @operations.vault_structure_version_current()
    end

    # Reads an existing Vault structure version without changing it.
    def vault_directory_probe_structure_version(root, password)
      @operations.vault_directory_probe_structure_version(root, password)
    end

    # Opens or creates the default Vault without replacing existing state.
    def vault_directory_open_or_create_default(password)
      Vault.new(@operations, @operations.vault_directory_open_or_create_default(password))
    end

    # Replaces the default Vault and all persistent data it contains.
    def vault_directory_replace_default(password)
      Vault.new(@operations, @operations.vault_directory_replace_default(password))
    end

    # Changes the passphrase for an existing Vault.
    def vault_directory_change_password(root, old_password, new_password)
      @operations.vault_directory_change_password(root, old_password, new_password)
    end

    # Changes the passphrase for the default Vault.
    def vault_directory_change_default_password(old_password, new_password)
      @operations.vault_directory_change_default_password(old_password, new_password)
    end

    # Replaces the selected Vault and all persistent data it contains.
    def vault_directory_replace(root, password)
      Vault.new(@operations, @operations.vault_directory_replace(root, password))
    end

    # Opens the selected Vault, creating it only when absent.
    def vault_directory_open_or_create(root, password)
      Vault.new(@operations, @operations.vault_directory_open_or_create(root, password))
    end

    # Writes a backup of the default Vault to the selected path.
    def vault_backup_default(path, overwrite)
      @operations.vault_backup_default(path, overwrite)
    end

    # Restores the default Vault from the selected backup.
    def vault_restore_default(path, overwrite)
      @operations.vault_restore_default(path, overwrite)
    end

    # Opens an existing Vault metadata view that cannot load private keys.
    def vault_read_only_open(root, password)
      ReadOnlyVault.new(@operations, @operations.vault_read_only_open(root, password))
    end

    # Opens the default Vault metadata view without loading private keys.
    def vault_read_only_open_default(password)
      ReadOnlyVault.new(@operations, @operations.vault_read_only_open_default(password))
    end

    # Returns the platform default Vault directory.
    def vault_default_directory()
      @operations.vault_default_directory()
    end

    # Returns the path of the default Vault file.
    def vault_default_path()
      @operations.vault_default_path()
    end

    # Returns the session agent log path.
    def vault_agent_log_path()
      @operations.vault_agent_log_path()
    end

    # Returns the configured session agent log destination.
    def vault_agent_log_destination()
      @operations.vault_agent_log_destination()
    end

  end

  # An open encrypted archive containing files, variables, secrets, and forms.
  class Lockbox < OwnedHandle
    # Creates an in-memory archive protected by exactly one credential.
    def self.create_in_memory(password: nil, content_key: nil, contact: nil, signing_key: nil, options: nil)
      credentials = [password, content_key, contact].compact
      raise ArgumentError, 'supply exactly one of password, content_key, or contact' unless credentials.length == 1
      runtime = Revault.runtime
      box = if password
              signing_key ? runtime.lockbox_create_password_with_signing_key(password.to_str, signing_key) : runtime.lockbox_create_password(password.to_str)
            elsif contact
              signing_key ? runtime.lockbox_create_contact_with_signing_key(contact, signing_key) : runtime.lockbox_create_contact(contact)
            elsif options
              runtime.lockbox_create_with_options(content_key, options[:cache_mode], options[:cache_bytes] || 0, options[:workload], options[:worker], options[:jobs] || 0)
            else
              runtime.lockbox_create(content_key)
            end
      box.set_owner_signing_key(signing_key) if signing_key && !password && !contact
      box
    end

    # Opens serialized archive bytes without consulting the Session Agent.
    def self.open_bytes(archive, password: nil, content_key: nil, contact: nil, options: nil)
      credentials = [password, content_key, contact].compact
      raise ArgumentError, 'supply exactly one of password, content_key, or contact' unless credentials.length == 1
      runtime = Revault.runtime
      return runtime.lockbox_open_password(archive, password.to_str) if password
      return runtime.lockbox_open_contact(archive, contact) if contact
      options ? runtime.lockbox_open_with_options(archive, content_key, options[:cache_mode], options[:cache_bytes] || 0, options[:workload], options[:worker], options[:jobs] || 0) : runtime.lockbox_open(archive, content_key)
    end

    # Creates an archive file and returns its process-local handle.
    def self.create(path, **options)
      raise Errno::EEXIST, path if File.exist?(path) && !options.delete(:overwrite)
      box = create_in_memory(**options)
      File.binwrite(path, box.to_bytes)
      box.instance_variable_set(:@backing_path, path)
      box
    end

    # Opens an archive file without consulting the Session Agent.
    def self.open(path, **options)
      box = open_bytes(File.binread(path), **options)
      box.instance_variable_set(:@backing_path, path)
      box
    end

    # Stages a file at the Lockbox path; replace controls an existing entry.
    def add_file(path, data, replace)
      @operations.lockbox_add_file(@native_handle, path, data, replace)
    end

    # Stages a file and its portable Unix permission bits.
    def add_file_with_permissions(path, data, permissions, replace)
      @operations.lockbox_add_file_with_permissions(@native_handle, path, data, permissions, replace)
    end

    # Reads the complete file stored at the Lockbox path.
    def get_file(path)
      @operations.lockbox_get_file(@native_handle, path)
    end

    # Writes one Lockbox file to the host filesystem.
    def extract_file(source, destination, replace)
      @operations.lockbox_extract_file(@native_handle, source, destination, replace)
    end

    # Extracts the Lockbox with explicit size, count, link, and permission limits.
    def extract_directory(destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
      @operations.lockbox_extract_directory(@native_handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
    end

    # Lists logical or physical content chunks for streaming diagnostics.
    def stream_content(physical)
      @operations.lockbox_stream_content(@native_handle, physical)
    end

    # Returns cache statistics for this lockbox.
    def cache_stats()
      @operations.lockbox_cache_stats(@native_handle)
    end

    # Returns import statistics for this lockbox.
    def import_stats()
      @operations.lockbox_import_stats(@native_handle)
    end

    # Updates import stats.
    def reset_import_stats()
      @operations.lockbox_reset_import_stats(@native_handle)
    end

    # Returns page metadata for diagnostics without exposing plaintext secrets.
    def page_inspection()
      @operations.lockbox_page_inspection(@native_handle)
    end

    # Scans the open archive and returns its structured recovery report.
    def recovery_report()
      @operations.lockbox_recovery_report(@native_handle)
    end

    # Renders the recovery report for a person, capped at maxEntries.
    def recovery_report_render(verbose, max_entries)
      @operations.lockbox_recovery_report_render(@native_handle, verbose, max_entries)
    end

    # Returns the current serialized archive size in bytes.
    def storage_len()
      @operations.lockbox_storage_len(@native_handle)
    end

    # Sets workload profile.
    def set_workload_profile(profile)
      @operations.lockbox_set_workload_profile(@native_handle, profile)
    end

    # Sets worker policy.
    def set_worker_policy(mode, jobs)
      @operations.lockbox_set_worker_policy(@native_handle, mode, jobs)
    end

    # Returns the cache, workload, and worker settings used by this Lockbox.
    def runtime_options()
      @operations.lockbox_runtime_options(@native_handle)
    end

    # Authenticates and publishes the staged changes.
    def commit()
      result = @operations.lockbox_commit(@native_handle)
      File.binwrite(@backing_path, to_bytes) if @backing_path
      result
    end

    # Stages a directory entry and optionally creates missing parents.
    def create_dir(path, create_parents)
      @operations.lockbox_create_dir(@native_handle, path, create_parents)
    end

    # Stages removal of a file, link, or empty directory at path.
    def delete(path)
      @operations.lockbox_delete(@native_handle, path)
    end

    # Stages removal of a directory, optionally including its descendants.
    def remove_dir(path, recursive)
      @operations.lockbox_remove_dir(@native_handle, path, recursive)
    end

    # Stages every missing parent directory for path.
    def create_parent_dirs(path)
      @operations.lockbox_create_parent_dirs(@native_handle, path)
    end

    # Stages an atomic move from one Lockbox path to another.
    def rename(from, to)
      @operations.lockbox_rename(@native_handle, from, to)
    end

    # Lists entries below path, optionally including descendants.
    def list(path, recursive)
      @operations.lockbox_list(@native_handle, path, recursive)
    end

    # Lists entries using glob, type, recursion, and result limit filters.
    def list_with_options(path, glob, recursive, include_files, include_symlinks, include_directories, limit)
      @operations.lockbox_list_with_options(@native_handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
    end

    # Returns metadata for the selected lockbox entry.
    def stat(path)
      @operations.lockbox_stat(@native_handle, path)
    end

    # Stages a plain text variable; commit to publish the change.
    def set_variable(name, value)
      @operations.lockbox_set_variable(@native_handle, name, value)
    end

    # Stores a secret variable from mutable bytes.
    def set_secret_variable(name, value)
      @operations.lockbox_set_secret_variable(@native_handle, name, value)
    end

    # Returns a plain variable when it is present.
    def get_variable(name)
      @operations.lockbox_get_variable(@native_handle, name)
    end

    # Returns the encrypted lockbox description, or nil when unset.
    # Example: set it, commit, then `puts box.description`.
    def description
      get_variable('/.revault/description')
    end

    # Stages encrypted description text; call commit to publish it.
    # Example: `box.set_description('Production credentials'); box.commit`.
    def set_description(description)
      set_variable('/.revault/description', description)
    end

    # Stages removal of the encrypted description; call commit.
    # Example: `box.clear_description; box.commit`.
    def clear_description
      delete_variable('/.revault/description')
    end

    # Yields temporary secret bytes and wipes the native transfer afterwards.
    def with_secret_variable(name, &callback)
      @operations.lockbox_get_secret_variable(@native_handle, name, &callback)
    end

    # Stages removal of a variable.
    def delete_variable(name)
      @operations.lockbox_delete_variable(@native_handle, name)
    end

    # Atomically renames variables using source and destination path pairs.
    def move_variables(moves)
      @operations.lockbox_move_variables(@native_handle, Internal::DomainCodec.encode_path_moves(moves))
    end

    # Lists variable names and metadata without exposing secret values.
    def list_variables()
      @operations.lockbox_list_variables(@native_handle)
    end

    # Returns whether a variable is plain or secret.
    def variable_sensitivity(name)
      @operations.lockbox_variable_sensitivity(@native_handle, name)
    end

    # Stages a symbolic link with its stored target text.
    def add_symlink(path, target, replace)
      @operations.lockbox_add_symlink(@native_handle, path, target, replace)
    end

    # Returns the target text stored for a symbolic link.
    def get_symlink_target(path)
      @operations.lockbox_get_symlink_target(@native_handle, path)
    end

    # Returns the stable public identifier stored in the Lockbox header.
    def id()
      @operations.lockbox_id(@native_handle)
    end

    # Reports whether an entry exists at path.
    def exists(path)
      @operations.lockbox_exists(@native_handle, path)
    end

    # Reports whether path names a directory entry.
    def is_dir(path)
      @operations.lockbox_is_dir(@native_handle, path)
    end

    # Returns the portable Unix permission bits stored for path.
    def permissions(path)
      @operations.lockbox_permissions(@native_handle, path)
    end

    # Stages portable Unix permission bits for path.
    def set_permissions(path, permissions)
      @operations.lockbox_set_permissions(@native_handle, path, permissions)
    end

    # Reads the requested byte range from a stored file.
    def read_range(path, offset, len)
      @operations.lockbox_read_range(@native_handle, path, offset, len)
    end

    # Adds a password access slot and returns its slot identifier.
    def add_password(password)
      @operations.lockbox_add_password(@native_handle, password)
    end

    # Grants a named contact access and returns the new slot identifier.
    def add_contact(contact, name)
      @operations.lockbox_add_contact(@native_handle, contact.native_handle, name)
    end

    # Removes an access slot; at least one usable slot must remain.
    def delete_key(id)
      @operations.lockbox_delete_key(@native_handle, id)
    end

    # Lists public access slot metadata without returning credentials.
    def list_key_slots()
      @operations.lockbox_list_key_slots(@native_handle)
    end

    # Assigns a profile signing key to the Lockbox owner role.
    def set_owner_signing_key(key)
      @operations.lockbox_set_owner_signing_key(@native_handle, key.native_handle)
    end

    # Returns public signing and ownership metadata for the current revision.
    def owner_inspection()
      @operations.lockbox_owner_inspection(@native_handle)
    end

    # Defines a reusable, versioned form from the supplied field definitions.
    def define_form(alias_name, name, description, fields)
      @operations.lockbox_define_form(@native_handle, alias_name, name, description, Internal::DomainCodec.encode_form_fields(fields))
    end

    # Lists the form definitions stored in this Lockbox.
    def list_form_definitions()
      @operations.lockbox_list_form_definitions(@native_handle)
    end

    # Resolves a form alias, type identifier, or revision.
    def resolve_form(reference)
      @operations.lockbox_resolve_form(@native_handle, reference)
    end

    # Lists every stored revision for a form type identifier.
    def list_form_revisions(type_id)
      @operations.lockbox_list_form_revisions(@native_handle, type_id)
    end

    # Stages a form record at path using the referenced definition.
    def create_form_record(path, type_reference, name)
      @operations.lockbox_create_form_record(@native_handle, path, type_reference, name)
    end

    # Stages a plain field value in a form record.
    def set_form_field(path, field, value)
      @operations.lockbox_set_form_field(@native_handle, path, field, value)
    end

    # Stores a secret form field from mutable bytes.
    def set_secret_form_field(path, field, value)
      @operations.lockbox_set_secret_form_field(@native_handle, path, field, value)
    end

    # Lists form records without exposing secret field values.
    def list_form_records()
      @operations.lockbox_list_form_records(@native_handle)
    end

    # Returns the form record at path when present.
    def get_form_record(path)
      @operations.lockbox_get_form_record(@native_handle, path)
    end

    # Stages removal of a form record.
    def delete_form_record(path)
      @operations.lockbox_delete_form_record(@native_handle, path)
    end

    # Atomically renames form records using source and destination path pairs.
    def move_form_records(moves)
      @operations.lockbox_move_form_records(@native_handle, Internal::DomainCodec.encode_path_moves(moves))
    end

    # Returns a plain form field when it exists.
    def get_form_field(path, field)
      @operations.lockbox_get_form_field(@native_handle, path, field)
    end

    # Yields temporary field bytes and wipes the native transfer afterwards.
    def with_secret_form_field(path, field, &callback)
      @operations.lockbox_get_secret_form_field(@native_handle, path, field, &callback)
    end

    # Serializes the current Lockbox, including committed changes.
    def to_bytes()
      @operations.lockbox_to_bytes(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.lockbox_free(@native_handle)
      @native_handle = nil
    end

  end

  # A profile's contact-encryption identity used to decrypt keys addressed to it.
  class ContactKeyPair < OwnedHandle
    # Returns the canonical public bytes paired with this identity.
    def public_bytes()
      @operations.key_contact_public(@native_handle)
    end

    # Returns the private signing-key record for secure binary backup.
    def private_record()
      @operations.key_contact_private(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.key_contact_free(@native_handle)
      @native_handle = nil
    end

    # Decrypts a wrapped content key for this contact.
    def decrypt(wrapped)
      @operations.key_contact_decrypt(@native_handle, wrapped.native_handle)
    end

  end

  # A recipient's shareable encryption identity used when granting access.
  class ContactPublicKey < OwnedHandle
    # Releases this public contact key.
    def public_free()
      @operations.key_contact_public_free(@native_handle)
      @native_handle = nil
    end

    # Encrypts a content key for the selected contact.
    def encrypt(content_key)
      WrappedContactKey.new(@operations, @operations.key_contact_encrypt(@native_handle, content_key))
    end

  end

  # A content key encrypted for one contact and recoverable by its matching key pair.
  class WrappedContactKey < OwnedHandle
    # Returns the public.
    def public()
      @operations.key_contact_wrapped_public(@native_handle)
    end

    # Returns the encrypted content key bytes.
    def ciphertext()
      @operations.key_contact_wrapped_ciphertext(@native_handle)
    end

    # Returns the complete wrapped key record for storage or transport.
    def encrypted()
      @operations.key_contact_wrapped_encrypted(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.key_contact_wrapped_free(@native_handle)
      @native_handle = nil
    end

  end

  # A Vault Profile signing identity used to authorize mutable Lockbox revisions.
  class ProfileSigningKeyPair < OwnedHandle
    # Returns the public.
    def public()
      @operations.key_signing_public(@native_handle)
    end

    # Returns the private.
    def private()
      @operations.key_signing_private(@native_handle)
    end

    # Creates an independently owned public verification-key handle.
    def public_key()
      ProfileSigningPublicKey.new(
        @operations,
        @operations.key_signing_public_from_bytes(public)
      )
    end

    # Releases the native resources held by this object.
    def free()
      @operations.key_signing_free(@native_handle)
      @native_handle = nil
    end

  end

  # The public half of a Vault Profile signing identity.
  class ProfileSigningPublicKey < OwnedHandle
    # Releases the native resources held by this object.
    def free()
      @operations.key_signing_public_free(@native_handle)
      @native_handle = nil
    end

  end

  # Password-protected storage for Profile keys, contacts, forms, backups, and lockbox paths.
  class VaultStore < OwnedHandle
    # Returns the canonical root directory of this Vault.
    def root()
      @operations.vault_directory_root(@native_handle)
    end

    # Returns the persistent structure version of this Vault.
    def structure_version()
      @operations.vault_directory_structure_version(@native_handle)
    end

    # Lists private keys.
    def list_private_keys()
      @operations.vault_directory_list_private_keys(@native_handle)
    end

    # Lists private key names.
    def list_private_key_names()
      @operations.vault_directory_list_private_key_names(@native_handle)
    end

    # Lists contact names.
    def list_contact_names()
      @operations.vault_directory_list_contact_names(@native_handle)
    end

    # Lists form aliases.
    def list_form_aliases()
      @operations.vault_directory_list_form_aliases(@native_handle)
    end

    # Reports whether the named profile private key exists.
    def private_key_exists(name)
      @operations.vault_directory_private_key_exists(@native_handle, name)
    end

    # Removes private key.
    def delete_private_key(name)
      @operations.vault_directory_delete_private_key(@native_handle, name)
    end

    # Stores private key.
    def store_private_key(name, key)
      @operations.vault_directory_store_private_key(@native_handle, name, key.native_handle)
    end

    # Loads private key.
    def load_private_key(name)
      ContactKeyPair.new(@operations, @operations.vault_directory_load_private_key(@native_handle, name))
    end

    # Loads private key generation.
    def load_private_key_generation(name, index)
      ContactKeyPair.new(@operations, @operations.vault_directory_load_private_key_generation(@native_handle, name, index))
    end

    # Stores contact.
    def store_contact(name, key)
      @operations.vault_directory_store_contact(@native_handle, name, key.native_handle)
    end

    # Loads contact.
    def load_contact(name)
      ContactPublicKey.new(@operations, @operations.vault_directory_load_contact(@native_handle, name))
    end

    # Reports whether the named contact exists.
    def contact_exists(name)
      @operations.vault_directory_contact_exists(@native_handle, name)
    end

    # Removes contact.
    def delete_contact(name)
      @operations.vault_directory_delete_contact(@native_handle, name)
    end

    # Lists contacts.
    def list_contacts()
      @operations.vault_directory_list_contacts(@native_handle)
    end

    # Stores profile email.
    def store_profile_email(name, email)
      @operations.vault_directory_store_profile_email(@native_handle, name, email)
    end

    # Returns the email recorded for a profile, when present.
    def profile_email(name)
      @operations.vault_directory_profile_email(@native_handle, name)
    end

    # Stores backup.
    def store_backup(id, bytes)
      @operations.vault_directory_store_backup(@native_handle, id, bytes)
    end

    # Loads backup.
    def load_backup(id)
      @operations.vault_directory_load_backup(@native_handle, id)
    end

    # Returns the number of stored key recovery backups.
    def backup_count()
      @operations.vault_directory_backup_count(@native_handle)
    end

    # Restores a profile private key and signing key from recovery material.
    def restore_private_key(name, key, signing_key, overwrite)
      @operations.vault_directory_restore_private_key(@native_handle, name, key.native_handle, signing_key.native_handle, overwrite)
    end

    # Loads the current signing identity for a Vault Profile.
    def load_profile_signing_key(name)
      ProfileSigningKeyPair.new(@operations, @operations.vault_directory_load_owner_signing_key(@native_handle, name))
    end

    # Loads one historical signing identity for a Vault Profile.
    def load_profile_signing_key_generation(name, index)
      ProfileSigningKeyPair.new(@operations, @operations.vault_directory_load_owner_signing_key_generation(@native_handle, name, index))
    end

    # Stores contact signing key.
    def store_contact_signing_key(name, key)
      @operations.vault_directory_store_contact_signing_key(@native_handle, name, key.native_handle)
    end

    # Loads contact signing key.
    def load_contact_signing_key(name)
      ProfileSigningPublicKey.new(@operations, @operations.vault_directory_load_contact_signing_key(@native_handle, name))
    end

    # Lists profile generations.
    def list_profile_generations(name)
      @operations.vault_directory_list_profile_generations(@native_handle, name)
    end

    # Updates private key.
    def rotate_private_key(name)
      @operations.vault_directory_rotate_private_key(@native_handle, name)
    end

    # Stores lockbox.
    def remember_lockbox(id, path)
      @operations.vault_directory_remember_lockbox(@native_handle, id, path)
    end

    # Lists known lockboxes.
    def list_known_lockboxes()
      @operations.vault_directory_list_known_lockboxes(@native_handle)
    end

    # Removes lockbox.
    def forget_lockbox(path)
      @operations.vault_directory_forget_lockbox(@native_handle, path)
    end

    # Stores access slot label.
    def remember_access_slot_label(id, slot_id, name)
      @operations.vault_directory_remember_access_slot_label(@native_handle, id, slot_id, name)
    end

    # Lists access slot labels.
    def list_access_slot_labels(id)
      @operations.vault_directory_list_access_slot_labels(@native_handle, id)
    end

    # Finds access slot labels with the supplied name for one Lockbox.
    def find_access_slot_labels(id, name)
      @operations.vault_directory_find_access_slot_labels(@native_handle, id, name)
    end

    # Removes access slot label.
    def forget_access_slot_label(id, slot_id)
      @operations.vault_directory_forget_access_slot_label(@native_handle, id, slot_id)
    end

    # Defines a reusable, versioned form in the local vault.
    def define_form(alias_name, name, description, fields)
      @operations.vault_directory_define_form(@native_handle, alias_name, name, description, Internal::DomainCodec.encode_form_fields(fields))
    end

    # Resolves a form alias, type identifier, or revision.
    def resolve_form(reference)
      @operations.vault_directory_resolve_form(@native_handle, reference)
    end

    # Lists forms.
    def list_forms()
      @operations.vault_directory_list_forms(@native_handle)
    end

    # Lists every stored revision for a form type identifier.
    def list_form_revisions(type_id)
      @operations.vault_directory_list_form_revisions(@native_handle, type_id)
    end

    # Adds missing standard form definitions and returns the number added.
    def seed_forms()
      @operations.vault_directory_seed_forms(@native_handle)
    end

    # Stores password.
    def remember_password(id, password)
      @operations.vault_directory_remember_password(@native_handle, id, password)
    end

    # Returns the Lockbox password encrypted inside this Vault.
    def remembered_password(id)
      @operations.vault_directory_remembered_password(@native_handle, id)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.vault_directory_free(@native_handle)
      @native_handle = nil
    end

  end

  # A metadata view for discovery that never loads an owner signing key.
  class ReadOnlyVault < OwnedHandle
    # Lists profile names.
    def list_profile_names()
      @operations.vault_read_only_list_profile_names(@native_handle)
    end

    # Lists contact names.
    def list_contact_names()
      @operations.vault_read_only_list_contact_names(@native_handle)
    end

    # Lists form aliases.
    def list_form_aliases()
      @operations.vault_read_only_list_form_aliases(@native_handle)
    end

    # Lists known lockboxes.
    def list_known_lockboxes()
      @operations.vault_read_only_list_known_lockboxes(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.vault_read_only_free(@native_handle)
    end

  end

  # Client for the session service that temporarily caches unlock and signing keys.
  class Agent
    # Returns the initialize.
    def initialize(operations)
      @operations = operations
    end

    # Reports whether running.
    def is_running()
      @operations.vault_is_running()
    end

    # Removes all.
    def forget_all()
      @operations.vault_forget_all()
    end

    # Runs the session agent server until it is stopped.
    def serve()
      @operations.vault_agent_serve()
    end

    # Verifies transport.
    def verify_transport()
      @operations.vault_agent_verify_transport()
    end

    # Returns get.
    def get(id)
      @operations.vault_agent_get(id)
    end

    # Stores put.
    def put(id, key)
      @operations.vault_agent_put(id, key)
    end

    # Removes forget.
    def forget(id)
      @operations.vault_agent_forget(id)
    end

    # Stops stop.
    def stop()
      @operations.vault_agent_stop()
    end

    # Starts start.
    def start()
      @operations.vault_agent_start()
    end

    # Lists entries below path, optionally including descendants.
    def list()
      @operations.vault_agent_list()
    end

    # Reports how the platform handles agent expiry during system sleep.
    def sleep_support()
      @operations.vault_agent_sleep_support()
    end

    # Returns vault unlock key.
    def get_vault_unlock_key(vault_id)
      @operations.vault_agent_get_vault_unlock_key(vault_id)
    end

    # Stores vault unlock key.
    def put_vault_unlock_key(vault_id, key, ttl_seconds)
      @operations.vault_agent_put_vault_unlock_key(vault_id, key, ttl_seconds)
    end

    # Removes vault unlock key.
    def forget_vault_unlock_key(vault_id)
      @operations.vault_agent_forget_vault_unlock_key(vault_id)
    end

    # Returns the cached signing identity for a Vault Profile.
    def profile_signing_key(vault_id, profile)
      ProfileSigningKeyPair.new(@operations, @operations.vault_agent_get_owner_signing_key(vault_id, profile))
    end

    # Caches a signing identity for a Vault Profile.
    def cache_profile_signing_key(vault_id, profile, key, ttl_seconds)
      @operations.vault_agent_put_owner_signing_key(vault_id, profile, key.native_handle, ttl_seconds)
    end

    # Removes a cached signing identity for a Vault Profile.
    def forget_profile_signing_key(vault_id, profile)
      @operations.vault_agent_forget_owner_signing_key(vault_id, profile)
    end

    # Starts activity.
    def begin_activity(kind)
      AgentActivity.new(@operations, @operations.vault_agent_begin_activity(kind))
    end

    # Stops activity.
    def end_activity(handle)
      @operations.vault_agent_end_activity(handle.native_handle)
    end

  end

  # A token kept alive while an operation needs secrets cached by the agent.
  class AgentActivity < OwnedHandle
  end

  # Access to the platform credential store for a scoped Vault passphrase.
  class Platform
    # Returns the initialize.
    def initialize(operations)
      @operations = operations
    end

    # Returns availability and user presence guarantees for platform storage.
    def status()
      @operations.vault_platform_status()
    end

    # Sets scope.
    def set_scope(scope)
      @operations.vault_platform_set_scope(scope)
    end

    # Removes password.
    def forget_password()
      @operations.vault_platform_forget_password()
    end

    # Stores password.
    def put_password(password)
      @operations.vault_platform_put_password(password)
    end

    # Enables storage of the Vault passphrase in platform credentials.
    def enable()
      @operations.vault_platform_enable()
    end

    # Disables platform credential use without deleting the stored value.
    def disable()
      @operations.vault_platform_disable()
    end

    # Reports whether platform credential use is disabled.
    def disabled()
      @operations.vault_platform_disabled()
    end

    # Returns password.
    def get_password()
      @operations.vault_platform_get_password()
    end

  end

  # A session that opens lockboxes by host path, caches passwords, and closes local files.
  class LocalSession < OwnedHandle
    # Creates Lockbox password.
    def create_lockbox_password(path, password)
      Lockbox.new(@operations, @operations.vault_create_lockbox_password(@native_handle, path, password))
    end

    # Opens Lockbox password.
    def open_lockbox_password(path, password)
      Lockbox.new(@operations, @operations.vault_open_lockbox_password(@native_handle, path, password))
    end

    # Creates lockbox content key.
    def create_lockbox_content_key(path, content_key, signing_key)
      Lockbox.new(@operations, @operations.vault_create_lockbox_content_key(@native_handle, path, content_key, signing_key.native_handle))
    end

    # Creates lockbox contact.
    def create_lockbox_contact(path, contact, name, signing_key)
      Lockbox.new(@operations, @operations.vault_create_lockbox_contact(@native_handle, path, contact.native_handle, name, signing_key.native_handle))
    end

    # Opens lockbox content key.
    def open_lockbox_content_key(path, content_key, signing_key)
      Lockbox.new(@operations, @operations.vault_open_lockbox_content_key(@native_handle, path, content_key, signing_key.native_handle))
    end

    # Stores Lockbox password.
    def cache_lockbox_password(path, password, ttl_seconds)
      @operations.vault_cache_lockbox_password(@native_handle, path, password, ttl_seconds)
    end

    # Releases the native resources held by lockbox.
    def close_lockbox(path)
      @operations.vault_close_lockbox(@native_handle, path)
    end

    # Releases the native resources held by all.
    def close_all()
      @operations.vault_close_all(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.vault_free(@native_handle)
      @native_handle = nil
    end

  end

end

# Reviewed 0.3 facade names. The Fiddle transport classes above remain
# implementation details of these domain objects.
module Revault
  module LockboxCacheMode
    BYTES = 'bytes'; DISABLED = 'disabled'; AUTOMATIC = 'automatic'
  end
  module LockboxWorkload
    INTERACTIVE = 'interactive'; BULK_IMPORT = 'bulk-import'; READ_MOSTLY = 'read-mostly'
  end
  module LockboxWorker
    AUTO = 'auto'; SINGLE = 'single'; THREADS = 'threads'
  end
  module AgentActivityKind
    OPEN = 'open'; CLOSE = 'close'; VARIABLES = 'variables'; FORM = 'form'; RECOVERY = 'recovery'; VAULT = 'vault'
  end
  module KeyExportFormat
    LOCKBOX_PEM = 'lockbox-pem'; JWK = 'jwk'; JWKS = 'jwks'; RAW_HEX = 'raw-hex'
  end

  # Mutable byte secret that can be wiped after a native operation.
  class SecretBytes
    # Copy bytes into owned mutable storage.
    def initialize(value = ''.b) @bytes = value.to_s.b.dup end
    # Return a copy of the secret bytes.
    def to_str = @bytes
    # Return a defensive copy of the secret bytes.
    def bytes = @bytes.dup
    # Wipe the secret in place.
    def close = @bytes.replace("\0" * @bytes.bytesize)
    alias dispose close
  end
  # UTF-8 secret passphrase with the same wipe semantics as SecretBytes.
  class SecretString < SecretBytes
    # Copy a passphrase into mutable UTF-8 storage.
    def initialize(value = '') super(value.to_s.encode(Encoding::UTF_8)) end
  end

  # Persistent encrypted local store for profiles, keys, contacts and metadata.
  class Vault < VaultStore
    # Constructors for the persistent store lifecycle.
    class << self
      # Open an existing store without creating or replacing it.
      def open(root, vault_passphrase) = Runtime.new.vault_directory_open(root, vault_passphrase.to_str)
      # Open or create a store at root.
      def open_or_create(root, vault_passphrase) = Runtime.new.vault_directory_open_or_create(root, vault_passphrase.to_str)
      # Create a new store at root.
      def create(root, vault_passphrase) = Runtime.new.vault_directory_replace(root, vault_passphrase.to_str)
      # Replace an existing store explicitly.
      def replace(root, vault_passphrase) = Runtime.new.vault_directory_replace(root, vault_passphrase.to_str)
    end
  end

  # Shared close convenience for all owned facade handles.
  class OwnedHandle
    # Release the native resource.
    def close = free
  end
  # Lockbox-specific close convenience.
  class Lockbox
    # Release the lockbox handle.
    alias close free
  end
  # Persistent store close convenience.
  class VaultStore
    # Release the store handle.
    alias close free
  end
  # Read-only store close convenience.
  class ReadOnlyVault
    # Release the read-only store handle.
    alias close free
  end
  # Local session close convenience.
  class LocalSession
    # Release the local session handle.
    alias close free
  end
  # Explicit Session Agent controller with process-local lockbox operations.
  class AgentSession < Agent
    # Return the process-wide explicit session controller.
    def self.instance
      Revault.ensure_native
      @instance ||= new(BindingOperations.new)
    end
    # Attach this controller to the local native session handle.
    def initialize(operations)
      super
      @local = operations.vault_local
    end
    # Remove one cached lockbox key.
    def close_lockbox(path) = @operations.vault_close_lockbox(@local, path)
    # Remove all cached lockbox keys.
    def close_all = @operations.vault_close_all(@local)
    # Create a password-protected lockbox file.
    def create_lockbox_password(path, password) = Lockbox.new(@operations, @operations.vault_create_lockbox_password(@local, path, password.to_str))
    # Open a password-protected lockbox file.
    def open_lockbox_password(path, password) = Lockbox.new(@operations, @operations.vault_open_lockbox_password(@local, path, password.to_str))
    # Create a content-key lockbox file.
    def create_lockbox_content_key(path, content_key, signing_key) = Lockbox.new(@operations, @operations.vault_create_lockbox_content_key(@local, path, content_key, signing_key.native_handle))
    # Create a contact-addressed lockbox file.
    def create_lockbox_contact(path, contact, name, signing_key) = Lockbox.new(@operations, @operations.vault_create_lockbox_contact(@local, path, contact.native_handle, name, signing_key.native_handle))
    # Open a content-key lockbox file.
    def open_lockbox_content_key(path, content_key, signing_key) = Lockbox.new(@operations, @operations.vault_open_lockbox_content_key(@local, path, content_key, signing_key.native_handle))
    # Cache a password-derived key for the requested TTL.
    def cache_lockbox_password(path, password, ttl_seconds) = @operations.vault_cache_lockbox_password(@local, path, password.to_str, ttl_seconds)
    # Release the local session handle.
    def close
      @operations.vault_free(@local) if @local
      @local = nil
    end
    alias free close
  end
end

# Native handles and operation routing are implementation details of the
# application-facing facade above.
Revault.private_constant(:OwnedHandle, :Runtime, :VaultStore, :LocalSession)
