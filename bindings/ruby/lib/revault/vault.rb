# The owned, class-oriented reVault API. See the
# {repository README}[https://github.com/onepub-dev/reVault#readme] for the
# security model and complete examples.
require_relative 'binding_operations'

# Encrypts lockbox content and manages local vault metadata.
module Revault

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
  # session agent, and access operating-system credential storage.
  class Vault
    attr_reader :agent, :platform
    # Returns the initialize.
    def initialize
      @operations = BindingOperations.new
      @agent = Agent.new(@operations)
      @platform = Platform.new(@operations)
    end
    # Returns the last error.
    def last_error = @operations.last_error_message
    # Returns the last error details.
    def last_error_details = @operations.buffer_last_error_details()

    # Returns the lockbox format version.
    def lockbox_format_version()
      @operations.lockbox_format_version()
    end

    # Returns the lockbox probe format version.
    def lockbox_probe_format_version(bytes)
      @operations.lockbox_probe_format_version(bytes)
    end

    # Returns the lockbox create.
    def lockbox_create(key)
      Lockbox.new(@operations, @operations.lockbox_create(key))
    end

    # Creates a lockbox with explicit cache capacity, workload, worker policy, and job count.
    def lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
      Lockbox.new(@operations, @operations.lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs))
    end

    # Returns the lockbox create password.
    def lockbox_create_password(password)
      Lockbox.new(@operations, @operations.lockbox_create_password(password))
    end

    # Returns the lockbox create contact.
    def lockbox_create_contact(contact)
      Lockbox.new(@operations, @operations.lockbox_create_contact(contact.native_handle))
    end

    # Returns the lockbox create with signing key.
    def lockbox_create_with_signing_key(content_key, signing_key)
      Lockbox.new(@operations, @operations.lockbox_create_with_signing_key(content_key, signing_key.native_handle))
    end

    # Returns the lockbox open.
    def lockbox_open(archive, key)
      Lockbox.new(@operations, @operations.lockbox_open(archive, key))
    end

    # Opens a lockbox with explicit cache capacity, workload, worker policy, and job count.
    def lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
      Lockbox.new(@operations, @operations.lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs))
    end

    # Returns the lockbox open password.
    def lockbox_open_password(archive, password)
      Lockbox.new(@operations, @operations.lockbox_open_password(archive, password))
    end

    # Returns the lockbox open contact.
    def lockbox_open_contact(archive, contact)
      Lockbox.new(@operations, @operations.lockbox_open_contact(archive, contact.native_handle))
    end

    # Returns the lockbox inspect file.
    def lockbox_inspect_file(path)
      @operations.lockbox_inspect_file(path)
    end

    # Returns the lockbox recovery scan path.
    def lockbox_recovery_scan_path(path, key)
      @operations.lockbox_recovery_scan_path(path, key)
    end

    # Returns the lockbox recovery scan.
    def lockbox_recovery_scan(bytes, key)
      @operations.lockbox_recovery_scan(bytes, key)
    end

    # Returns the lockbox recovery salvage.
    def lockbox_recovery_salvage(bytes, key, signing_key)
      Lockbox.new(@operations, @operations.lockbox_recovery_salvage(bytes, key, signing_key.native_handle))
    end

    # Returns the key contact generate.
    def key_contact_generate()
      ContactKeyPair.new(@operations, @operations.key_contact_generate())
    end

    # Returns the key contact from private.
    def key_contact_from_private(bytes)
      ContactKeyPair.new(@operations, @operations.key_contact_from_private(bytes))
    end

    # Returns the key contact public from bytes.
    def key_contact_public_from_bytes(bytes)
      ContactPublicKey.new(@operations, @operations.key_contact_public_from_bytes(bytes))
    end

    # Returns the key signing generate.
    def key_signing_generate()
      SigningKeyPair.new(@operations, @operations.key_signing_generate())
    end

    # Returns the key signing from private.
    def key_signing_from_private(bytes)
      SigningKeyPair.new(@operations, @operations.key_signing_from_private(bytes))
    end

    # Returns the key signing public from bytes.
    def key_signing_public_from_bytes(bytes)
      SigningPublicKey.new(@operations, @operations.key_signing_public_from_bytes(bytes))
    end

    # Returns the vault key export private.
    def vault_key_export_private(key, format)
      @operations.vault_key_export_private(key.native_handle, format)
    end

    # Returns the vault key export public.
    def vault_key_export_public(key, format)
      @operations.vault_key_export_public(key.native_handle, format)
    end

    # Returns the vault key import private.
    def vault_key_import_private(bytes)
      ContactKeyPair.new(@operations, @operations.vault_key_import_private(bytes))
    end

    # Returns the vault key import public.
    def vault_key_import_public(bytes)
      ContactPublicKey.new(@operations, @operations.vault_key_import_public(bytes))
    end

    # Returns the vault key fingerprint.
    def vault_key_fingerprint(key)
      @operations.vault_key_fingerprint(key.native_handle)
    end

    # Returns the vault key format hex.
    def vault_key_format_hex(bytes)
      @operations.vault_key_format_hex(bytes)
    end

    # Returns the vault key decode hex.
    def vault_key_decode_hex(text)
      @operations.vault_key_decode_hex(text)
    end

    # Returns the vault key format crockford.
    def vault_key_format_crockford(bytes)
      @operations.vault_key_format_crockford(bytes)
    end

    # Returns the vault key format crockford reading.
    def vault_key_format_crockford_reading(code)
      @operations.vault_key_format_crockford_reading(code)
    end

    # Returns the vault key decode crockford.
    def vault_key_decode_crockford(code)
      @operations.vault_key_decode_crockford(code)
    end

    # Returns the vault key hex encode.
    def vault_key_hex_encode(bytes)
      @operations.vault_key_hex_encode(bytes)
    end

    # Returns the vault key hex decode.
    def vault_key_hex_decode(text)
      @operations.vault_key_hex_decode(text)
    end

    # Returns the vault directory open.
    def vault_directory_open(root, password)
      VaultDirectory.new(@operations, @operations.vault_directory_open(root, password))
    end

    # Returns the vault structure version current.
    def vault_structure_version_current()
      @operations.vault_structure_version_current()
    end

    # Returns the vault directory probe structure version.
    def vault_directory_probe_structure_version(root, password)
      @operations.vault_directory_probe_structure_version(root, password)
    end

    # Returns the vault directory open or create default.
    def vault_directory_open_or_create_default(password)
      VaultDirectory.new(@operations, @operations.vault_directory_open_or_create_default(password))
    end

    # Returns the vault directory replace default.
    def vault_directory_replace_default(password)
      VaultDirectory.new(@operations, @operations.vault_directory_replace_default(password))
    end

    # Returns the vault directory change password.
    def vault_directory_change_password(root, old_password, new_password)
      @operations.vault_directory_change_password(root, old_password, new_password)
    end

    # Returns the vault directory change default password.
    def vault_directory_change_default_password(old_password, new_password)
      @operations.vault_directory_change_default_password(old_password, new_password)
    end

    # Returns the vault directory replace.
    def vault_directory_replace(root, password)
      VaultDirectory.new(@operations, @operations.vault_directory_replace(root, password))
    end

    # Returns the vault directory open or create.
    def vault_directory_open_or_create(root, password)
      VaultDirectory.new(@operations, @operations.vault_directory_open_or_create(root, password))
    end

    # Returns the vault backup default.
    def vault_backup_default(path, overwrite)
      @operations.vault_backup_default(path, overwrite)
    end

    # Returns the vault restore default.
    def vault_restore_default(path, overwrite)
      @operations.vault_restore_default(path, overwrite)
    end

    # Returns the vault read only open.
    def vault_read_only_open(root, password)
      ReadOnlyVaultDirectory.new(@operations, @operations.vault_read_only_open(root, password))
    end

    # Returns the vault read only open default.
    def vault_read_only_open_default(password)
      ReadOnlyVaultDirectory.new(@operations, @operations.vault_read_only_open_default(password))
    end

    # Returns the vault default directory.
    def vault_default_directory()
      @operations.vault_default_directory()
    end

    # Returns the vault default path.
    def vault_default_path()
      @operations.vault_default_path()
    end

    # Returns the vault agent log path.
    def vault_agent_log_path()
      @operations.vault_agent_log_path()
    end

    # Returns the vault agent log destination.
    def vault_agent_log_destination()
      @operations.vault_agent_log_destination()
    end

    # Returns the vault local.
    def vault_local()
      LocalVault.new(@operations, @operations.vault_local())
    end

  end

  # An open encrypted archive containing files, variables, secrets, and forms.
  class Lockbox < OwnedHandle
    # Adds file.
    def add_file(path, data, replace)
      @operations.lockbox_add_file(@native_handle, path, data, replace)
    end

    # Adds file with permissions.
    def add_file_with_permissions(path, data, permissions, replace)
      @operations.lockbox_add_file_with_permissions(@native_handle, path, data, permissions, replace)
    end

    # Returns file.
    def get_file(path)
      @operations.lockbox_get_file(@native_handle, path)
    end

    # Extracts file.
    def extract_file(source, destination, replace)
      @operations.lockbox_extract_file(@native_handle, source, destination, replace)
    end

    # Extracts directory.
    def extract_directory(destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
      @operations.lockbox_extract_directory(@native_handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
    end

    # Returns the stream content.
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

    # Returns the page inspection.
    def page_inspection()
      @operations.lockbox_page_inspection(@native_handle)
    end

    # Returns the recovery report.
    def recovery_report()
      @operations.lockbox_recovery_report(@native_handle)
    end

    # Returns the recovery report render.
    def recovery_report_render(verbose, max_entries)
      @operations.lockbox_recovery_report_render(@native_handle, verbose, max_entries)
    end

    # Returns the storage len.
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

    # Returns the runtime options.
    def runtime_options()
      @operations.lockbox_runtime_options(@native_handle)
    end

    # Authenticates and publishes the staged changes.
    def commit()
      @operations.lockbox_commit(@native_handle)
    end

    # Creates dir.
    def create_dir(path, create_parents)
      @operations.lockbox_create_dir(@native_handle, path, create_parents)
    end

    # Removes delete.
    def delete(path)
      @operations.lockbox_delete(@native_handle, path)
    end

    # Removes dir.
    def remove_dir(path, recursive)
      @operations.lockbox_remove_dir(@native_handle, path, recursive)
    end

    # Creates parent dirs.
    def create_parent_dirs(path)
      @operations.lockbox_create_parent_dirs(@native_handle, path)
    end

    # Updates rename.
    def rename(from, to)
      @operations.lockbox_rename(@native_handle, from, to)
    end

    # Lists list.
    def list(path, recursive)
      @operations.lockbox_list(@native_handle, path, recursive)
    end

    # Lists with options.
    def list_with_options(path, glob, recursive, include_files, include_symlinks, include_directories, limit)
      @operations.lockbox_list_with_options(@native_handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
    end

    # Returns metadata for the selected lockbox entry.
    def stat(path)
      @operations.lockbox_stat(@native_handle, path)
    end

    # Sets variable.
    def set_variable(name, value)
      @operations.lockbox_set_variable(@native_handle, name, value)
    end

    # Stores a secret variable from mutable bytes.
    def set_secret_variable(name, value)
      @operations.lockbox_set_secret_variable(@native_handle, name, value)
    end

    # Returns variable.
    def get_variable(name)
      @operations.lockbox_get_variable(@native_handle, name)
    end

    # Yields temporary secret bytes and wipes the native transfer afterwards.
    def with_secret_variable(name, &callback)
      @operations.lockbox_get_secret_variable(@native_handle, name, &callback)
    end

    # Removes variable.
    def delete_variable(name)
      @operations.lockbox_delete_variable(@native_handle, name)
    end

    # Atomically renames variables using source and destination path pairs.
    def move_variables(moves)
      @operations.lockbox_move_variables(@native_handle, Internal::DomainCodec.encode_path_moves(moves))
    end

    # Lists variables.
    def list_variables()
      @operations.lockbox_list_variables(@native_handle)
    end

    # Returns the variable sensitivity.
    def variable_sensitivity(name)
      @operations.lockbox_variable_sensitivity(@native_handle, name)
    end

    # Adds symlink.
    def add_symlink(path, target, replace)
      @operations.lockbox_add_symlink(@native_handle, path, target, replace)
    end

    # Returns symlink target.
    def get_symlink_target(path)
      @operations.lockbox_get_symlink_target(@native_handle, path)
    end

    # Returns the id.
    def id()
      @operations.lockbox_id(@native_handle)
    end

    # Reports whether exists.
    def exists(path)
      @operations.lockbox_exists(@native_handle, path)
    end

    # Reports whether dir.
    def is_dir(path)
      @operations.lockbox_is_dir(@native_handle, path)
    end

    # Returns the permissions.
    def permissions(path)
      @operations.lockbox_permissions(@native_handle, path)
    end

    # Sets permissions.
    def set_permissions(path, permissions)
      @operations.lockbox_set_permissions(@native_handle, path, permissions)
    end

    # Returns range.
    def read_range(path, offset, len)
      @operations.lockbox_read_range(@native_handle, path, offset, len)
    end

    # Adds password.
    def add_password(password)
      @operations.lockbox_add_password(@native_handle, password)
    end

    # Adds contact.
    def add_contact(contact, name)
      @operations.lockbox_add_contact(@native_handle, contact.native_handle, name)
    end

    # Removes key.
    def delete_key(id)
      @operations.lockbox_delete_key(@native_handle, id)
    end

    # Lists key slots.
    def list_key_slots()
      @operations.lockbox_list_key_slots(@native_handle)
    end

    # Sets owner signing key.
    def set_owner_signing_key(key)
      @operations.lockbox_set_owner_signing_key(@native_handle, key.native_handle)
    end

    # Returns the owner inspection.
    def owner_inspection()
      @operations.lockbox_owner_inspection(@native_handle)
    end

    # Defines a reusable, versioned form from the supplied field definitions.
    def define_form(alias_name, name, description, fields)
      @operations.lockbox_define_form(@native_handle, alias_name, name, description, Internal::DomainCodec.encode_form_fields(fields))
    end

    # Lists form definitions.
    def list_form_definitions()
      @operations.lockbox_list_form_definitions(@native_handle)
    end

    # Returns the resolve form.
    def resolve_form(reference)
      @operations.lockbox_resolve_form(@native_handle, reference)
    end

    # Lists form revisions.
    def list_form_revisions(type_id)
      @operations.lockbox_list_form_revisions(@native_handle, type_id)
    end

    # Creates form record.
    def create_form_record(path, type_reference, name)
      @operations.lockbox_create_form_record(@native_handle, path, type_reference, name)
    end

    # Sets form field.
    def set_form_field(path, field, value)
      @operations.lockbox_set_form_field(@native_handle, path, field, value)
    end

    # Stores a secret form field from mutable bytes.
    def set_secret_form_field(path, field, value)
      @operations.lockbox_set_secret_form_field(@native_handle, path, field, value)
    end

    # Lists form records.
    def list_form_records()
      @operations.lockbox_list_form_records(@native_handle)
    end

    # Returns form record.
    def get_form_record(path)
      @operations.lockbox_get_form_record(@native_handle, path)
    end

    # Removes form record.
    def delete_form_record(path)
      @operations.lockbox_delete_form_record(@native_handle, path)
    end

    # Atomically renames form records using source and destination path pairs.
    def move_form_records(moves)
      @operations.lockbox_move_form_records(@native_handle, Internal::DomainCodec.encode_path_moves(moves))
    end

    # Returns form field.
    def get_form_field(path, field)
      @operations.lockbox_get_form_field(@native_handle, path, field)
    end

    # Yields temporary field bytes and wipes the native transfer afterwards.
    def with_secret_form_field(path, field, &callback)
      @operations.lockbox_get_secret_form_field(@native_handle, path, field, &callback)
    end

    # Returns the to bytes.
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
    # Returns the public.
    def public()
      @operations.key_contact_public(@native_handle)
    end

    # Returns the private.
    def private()
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
    # Returns the public free.
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

    # Returns the ciphertext.
    def ciphertext()
      @operations.key_contact_wrapped_ciphertext(@native_handle)
    end

    # Returns the encrypted.
    def encrypted()
      @operations.key_contact_wrapped_encrypted(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.key_contact_wrapped_free(@native_handle)
      @native_handle = nil
    end

  end

  # A lockbox owner's signing identity used to authorize mutable revisions.
  class SigningKeyPair < OwnedHandle
    # Returns the public.
    def public()
      @operations.key_signing_public(@native_handle)
    end

    # Returns the private.
    def private()
      @operations.key_signing_private(@native_handle)
    end

    # Releases the native resources held by this object.
    def free()
      @operations.key_signing_free(@native_handle)
      @native_handle = nil
    end

  end

  # The public identity readers use to verify owner-authorized revisions.
  class SigningPublicKey < OwnedHandle
    # Returns the public free.
    def public_free()
      @operations.key_signing_public_free(@native_handle)
      @native_handle = nil
    end

  end

  # Password-protected storage for profile keys, contacts, forms, backups, and lockbox paths.
  class VaultDirectory < OwnedHandle
    # Returns the root.
    def root()
      @operations.vault_directory_root(@native_handle)
    end

    # Returns the structure version.
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

    # Returns the private key exists.
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

    # Returns the contact exists.
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

    # Returns the profile email.
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

    # Returns the backup count.
    def backup_count()
      @operations.vault_directory_backup_count(@native_handle)
    end

    # Returns the restore private key.
    def restore_private_key(name, key, signing_key, overwrite)
      @operations.vault_directory_restore_private_key(@native_handle, name, key.native_handle, signing_key.native_handle, overwrite)
    end

    # Loads owner signing key.
    def load_owner_signing_key(name)
      SigningKeyPair.new(@operations, @operations.vault_directory_load_owner_signing_key(@native_handle, name))
    end

    # Loads owner signing key generation.
    def load_owner_signing_key_generation(name, index)
      SigningKeyPair.new(@operations, @operations.vault_directory_load_owner_signing_key_generation(@native_handle, name, index))
    end

    # Stores contact signing key.
    def store_contact_signing_key(name, key)
      @operations.vault_directory_store_contact_signing_key(@native_handle, name, key.native_handle)
    end

    # Loads contact signing key.
    def load_contact_signing_key(name)
      SigningPublicKey.new(@operations, @operations.vault_directory_load_contact_signing_key(@native_handle, name))
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

    # Returns the find access slot labels.
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

    # Returns the resolve form.
    def resolve_form(reference)
      @operations.vault_directory_resolve_form(@native_handle, reference)
    end

    # Lists forms.
    def list_forms()
      @operations.vault_directory_list_forms(@native_handle)
    end

    # Lists form revisions.
    def list_form_revisions(type_id)
      @operations.vault_directory_list_form_revisions(@native_handle, type_id)
    end

    # Returns the seed forms.
    def seed_forms()
      @operations.vault_directory_seed_forms(@native_handle)
    end

    # Stores password.
    def remember_password(id, password)
      @operations.vault_directory_remember_password(@native_handle, id, password)
    end

    # Returns the remembered password.
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
  class ReadOnlyVaultDirectory < OwnedHandle
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

    # Returns the serve.
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

    # Lists list.
    def list()
      @operations.vault_agent_list()
    end

    # Returns the sleep support.
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

    # Returns owner signing key.
    def get_owner_signing_key(vault_id, profile)
      SigningKeyPair.new(@operations, @operations.vault_agent_get_owner_signing_key(vault_id, profile))
    end

    # Stores owner signing key.
    def put_owner_signing_key(vault_id, profile, key, ttl_seconds)
      @operations.vault_agent_put_owner_signing_key(vault_id, profile, key.native_handle, ttl_seconds)
    end

    # Removes owner signing key.
    def forget_owner_signing_key(vault_id, profile)
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

  # Access to operating-system credential storage for a scoped vault password.
  class Platform
    # Returns the initialize.
    def initialize(operations)
      @operations = operations
    end

    # Returns the status.
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

    # Returns the enable.
    def enable()
      @operations.vault_platform_enable()
    end

    # Returns the disable.
    def disable()
      @operations.vault_platform_disable()
    end

    # Returns the disabled.
    def disabled()
      @operations.vault_platform_disabled()
    end

    # Returns password.
    def get_password()
      @operations.vault_platform_get_password()
    end

  end

  # A session that opens lockboxes by host path, caches passwords, and closes local files.
  class LocalVault < OwnedHandle
    # Creates lockbox password.
    def create_lockbox_password(path, password)
      Lockbox.new(@operations, @operations.vault_create_lockbox_password(@native_handle, path, password))
    end

    # Opens lockbox password.
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

    # Stores lockbox password.
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

# Native handles and operation routing are implementation details of the
# application-facing facade above.
Revault.private_constant(:BindingOperations, :OwnedHandle)
