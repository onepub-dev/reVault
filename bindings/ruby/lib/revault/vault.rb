# Generated complete class-oriented Ruby API. Do not edit.
require_relative 'binding_operations'

module Revault

  class OwnedHandle
    attr_reader :native_handle
    def initialize(operations, native_handle)
      @operations = operations
      @native_handle = native_handle
    end
  end

  class Vault
    attr_reader :agent, :platform
    def initialize
      @operations = BindingOperations.new
      @agent = Agent.new(@operations)
      @platform = Platform.new(@operations)
    end
    def last_error = @operations.last_error_message
    def last_error_details = @operations.buffer_last_error_details()

    def lockbox_format_version()
      @operations.lockbox_format_version()
    end

    def lockbox_probe_format_version(bytes)
      @operations.lockbox_probe_format_version(bytes)
    end

    def lockbox_create(key)
      Lockbox.new(@operations, @operations.lockbox_create(key))
    end

    def lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
      Lockbox.new(@operations, @operations.lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs))
    end

    def lockbox_create_password(password)
      Lockbox.new(@operations, @operations.lockbox_create_password(password))
    end

    def lockbox_create_contact(contact)
      Lockbox.new(@operations, @operations.lockbox_create_contact(contact.native_handle))
    end

    def lockbox_create_with_signing_key(content_key, signing_key)
      Lockbox.new(@operations, @operations.lockbox_create_with_signing_key(content_key, signing_key.native_handle))
    end

    def lockbox_open(archive, key)
      Lockbox.new(@operations, @operations.lockbox_open(archive, key))
    end

    def lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
      Lockbox.new(@operations, @operations.lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs))
    end

    def lockbox_open_password(archive, password)
      Lockbox.new(@operations, @operations.lockbox_open_password(archive, password))
    end

    def lockbox_open_contact(archive, contact)
      Lockbox.new(@operations, @operations.lockbox_open_contact(archive, contact.native_handle))
    end

    def lockbox_inspect_file(path)
      @operations.lockbox_inspect_file(path)
    end

    def lockbox_recovery_scan_path(path, key)
      @operations.lockbox_recovery_scan_path(path, key)
    end

    def lockbox_recovery_scan(bytes, key)
      @operations.lockbox_recovery_scan(bytes, key)
    end

    def lockbox_recovery_salvage(bytes, key, signing_key)
      Lockbox.new(@operations, @operations.lockbox_recovery_salvage(bytes, key, signing_key.native_handle))
    end

    def key_contact_generate()
      ContactKeyPair.new(@operations, @operations.key_contact_generate())
    end

    def key_contact_from_private(bytes)
      ContactKeyPair.new(@operations, @operations.key_contact_from_private(bytes))
    end

    def key_contact_public_from_bytes(bytes)
      ContactPublicKey.new(@operations, @operations.key_contact_public_from_bytes(bytes))
    end

    def key_signing_generate()
      SigningKeyPair.new(@operations, @operations.key_signing_generate())
    end

    def key_signing_from_private(bytes)
      SigningKeyPair.new(@operations, @operations.key_signing_from_private(bytes))
    end

    def key_signing_public_from_bytes(bytes)
      SigningPublicKey.new(@operations, @operations.key_signing_public_from_bytes(bytes))
    end

    def vault_key_export_private(key, format)
      @operations.vault_key_export_private(key.native_handle, format)
    end

    def vault_key_export_public(key, format)
      @operations.vault_key_export_public(key.native_handle, format)
    end

    def vault_key_import_private(bytes)
      ContactKeyPair.new(@operations, @operations.vault_key_import_private(bytes))
    end

    def vault_key_import_public(bytes)
      ContactPublicKey.new(@operations, @operations.vault_key_import_public(bytes))
    end

    def vault_key_fingerprint(key)
      @operations.vault_key_fingerprint(key.native_handle)
    end

    def vault_key_format_hex(bytes)
      @operations.vault_key_format_hex(bytes)
    end

    def vault_key_decode_hex(text)
      @operations.vault_key_decode_hex(text)
    end

    def vault_key_format_crockford(bytes)
      @operations.vault_key_format_crockford(bytes)
    end

    def vault_key_format_crockford_reading(code)
      @operations.vault_key_format_crockford_reading(code)
    end

    def vault_key_decode_crockford(code)
      @operations.vault_key_decode_crockford(code)
    end

    def vault_key_hex_encode(bytes)
      @operations.vault_key_hex_encode(bytes)
    end

    def vault_key_hex_decode(text)
      @operations.vault_key_hex_decode(text)
    end

    def vault_directory_open(root, password)
      VaultDirectory.new(@operations, @operations.vault_directory_open(root, password))
    end

    def vault_structure_version_current()
      @operations.vault_structure_version_current()
    end

    def vault_directory_probe_structure_version(root, password)
      @operations.vault_directory_probe_structure_version(root, password)
    end

    def vault_directory_open_or_create_default(password)
      VaultDirectory.new(@operations, @operations.vault_directory_open_or_create_default(password))
    end

    def vault_directory_replace_default(password)
      VaultDirectory.new(@operations, @operations.vault_directory_replace_default(password))
    end

    def vault_directory_change_password(root, old_password, new_password)
      @operations.vault_directory_change_password(root, old_password, new_password)
    end

    def vault_directory_change_default_password(old_password, new_password)
      @operations.vault_directory_change_default_password(old_password, new_password)
    end

    def vault_directory_replace(root, password)
      VaultDirectory.new(@operations, @operations.vault_directory_replace(root, password))
    end

    def vault_directory_open_or_create(root, password)
      VaultDirectory.new(@operations, @operations.vault_directory_open_or_create(root, password))
    end

    def vault_backup_default(path, overwrite)
      @operations.vault_backup_default(path, overwrite)
    end

    def vault_restore_default(path, overwrite)
      @operations.vault_restore_default(path, overwrite)
    end

    def vault_read_only_open(root, password)
      ReadOnlyVaultDirectory.new(@operations, @operations.vault_read_only_open(root, password))
    end

    def vault_read_only_open_default(password)
      ReadOnlyVaultDirectory.new(@operations, @operations.vault_read_only_open_default(password))
    end

    def vault_default_directory()
      @operations.vault_default_directory()
    end

    def vault_default_path()
      @operations.vault_default_path()
    end

    def vault_agent_log_path()
      @operations.vault_agent_log_path()
    end

    def vault_agent_log_destination()
      @operations.vault_agent_log_destination()
    end

    def vault_local()
      LocalVault.new(@operations, @operations.vault_local())
    end

  end

  class Lockbox < OwnedHandle
    def add_file(path, data, replace)
      @operations.lockbox_add_file(@native_handle, path, data, replace)
    end

    def add_file_with_permissions(path, data, permissions, replace)
      @operations.lockbox_add_file_with_permissions(@native_handle, path, data, permissions, replace)
    end

    def get_file(path)
      @operations.lockbox_get_file(@native_handle, path)
    end

    def extract_file(source, destination, replace)
      @operations.lockbox_extract_file(@native_handle, source, destination, replace)
    end

    def extract_directory(destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
      @operations.lockbox_extract_directory(@native_handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
    end

    def stream_content(physical)
      @operations.lockbox_stream_content(@native_handle, physical)
    end

    def cache_stats()
      @operations.lockbox_cache_stats(@native_handle)
    end

    def import_stats()
      @operations.lockbox_import_stats(@native_handle)
    end

    def reset_import_stats()
      @operations.lockbox_reset_import_stats(@native_handle)
    end

    def page_inspection()
      @operations.lockbox_page_inspection(@native_handle)
    end

    def recovery_report()
      @operations.lockbox_recovery_report(@native_handle)
    end

    def recovery_report_render(verbose, max_entries)
      @operations.lockbox_recovery_report_render(@native_handle, verbose, max_entries)
    end

    def storage_len()
      @operations.lockbox_storage_len(@native_handle)
    end

    def set_workload_profile(profile)
      @operations.lockbox_set_workload_profile(@native_handle, profile)
    end

    def set_worker_policy(mode, jobs)
      @operations.lockbox_set_worker_policy(@native_handle, mode, jobs)
    end

    def runtime_options()
      @operations.lockbox_runtime_options(@native_handle)
    end

    def commit()
      @operations.lockbox_commit(@native_handle)
    end

    def create_dir(path, create_parents)
      @operations.lockbox_create_dir(@native_handle, path, create_parents)
    end

    def delete(path)
      @operations.lockbox_delete(@native_handle, path)
    end

    def remove_dir(path, recursive)
      @operations.lockbox_remove_dir(@native_handle, path, recursive)
    end

    def create_parent_dirs(path)
      @operations.lockbox_create_parent_dirs(@native_handle, path)
    end

    def rename(from, to)
      @operations.lockbox_rename(@native_handle, from, to)
    end

    def list(path, recursive)
      @operations.lockbox_list(@native_handle, path, recursive)
    end

    def list_with_options(path, glob, recursive, include_files, include_symlinks, include_directories, limit)
      @operations.lockbox_list_with_options(@native_handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
    end

    def stat(path)
      @operations.lockbox_stat(@native_handle, path)
    end

    def set_variable(name, value, secret)
      @operations.lockbox_set_variable(@native_handle, name, value, secret)
    end

    def get_variable(name)
      @operations.lockbox_get_variable(@native_handle, name)
    end

    def delete_variable(name)
      @operations.lockbox_delete_variable(@native_handle, name)
    end

    def move_variables(moves_proto)
      @operations.lockbox_move_variables(@native_handle, moves_proto)
    end

    def list_variables()
      @operations.lockbox_list_variables(@native_handle)
    end

    def variable_sensitivity(name)
      @operations.lockbox_variable_sensitivity(@native_handle, name)
    end

    def add_symlink(path, target, replace)
      @operations.lockbox_add_symlink(@native_handle, path, target, replace)
    end

    def get_symlink_target(path)
      @operations.lockbox_get_symlink_target(@native_handle, path)
    end

    def id()
      @operations.lockbox_id(@native_handle)
    end

    def exists(path)
      @operations.lockbox_exists(@native_handle, path)
    end

    def is_dir(path)
      @operations.lockbox_is_dir(@native_handle, path)
    end

    def permissions(path)
      @operations.lockbox_permissions(@native_handle, path)
    end

    def set_permissions(path, permissions)
      @operations.lockbox_set_permissions(@native_handle, path, permissions)
    end

    def read_range(path, offset, len)
      @operations.lockbox_read_range(@native_handle, path, offset, len)
    end

    def add_password(password)
      @operations.lockbox_add_password(@native_handle, password)
    end

    def add_contact(contact, name)
      @operations.lockbox_add_contact(@native_handle, contact.native_handle, name)
    end

    def delete_key(id)
      @operations.lockbox_delete_key(@native_handle, id)
    end

    def list_key_slots()
      @operations.lockbox_list_key_slots(@native_handle)
    end

    def set_owner_signing_key(key)
      @operations.lockbox_set_owner_signing_key(@native_handle, key.native_handle)
    end

    def owner_inspection()
      @operations.lockbox_owner_inspection(@native_handle)
    end

    def define_form(alias_name, name, description, fields_proto)
      @operations.lockbox_define_form(@native_handle, alias_name, name, description, fields_proto)
    end

    def list_form_definitions()
      @operations.lockbox_list_form_definitions(@native_handle)
    end

    def resolve_form(reference)
      @operations.lockbox_resolve_form(@native_handle, reference)
    end

    def list_form_revisions(type_id)
      @operations.lockbox_list_form_revisions(@native_handle, type_id)
    end

    def create_form_record(path, type_reference, name)
      @operations.lockbox_create_form_record(@native_handle, path, type_reference, name)
    end

    def set_form_field(path, field, value, secret)
      @operations.lockbox_set_form_field(@native_handle, path, field, value, secret)
    end

    def list_form_records()
      @operations.lockbox_list_form_records(@native_handle)
    end

    def get_form_record(path)
      @operations.lockbox_get_form_record(@native_handle, path)
    end

    def delete_form_record(path)
      @operations.lockbox_delete_form_record(@native_handle, path)
    end

    def move_form_records(moves_proto)
      @operations.lockbox_move_form_records(@native_handle, moves_proto)
    end

    def get_form_field(path, field)
      @operations.lockbox_get_form_field(@native_handle, path, field)
    end

    def to_bytes()
      @operations.lockbox_to_bytes(@native_handle)
    end

    def free()
      @operations.lockbox_free(@native_handle)
      @native_handle = nil
    end

  end

  class ContactKeyPair < OwnedHandle
    def public()
      @operations.key_contact_public(@native_handle)
    end

    def private()
      @operations.key_contact_private(@native_handle)
    end

    def free()
      @operations.key_contact_free(@native_handle)
      @native_handle = nil
    end

    def decrypt(wrapped)
      @operations.key_contact_decrypt(@native_handle, wrapped.native_handle)
    end

  end

  class ContactPublicKey < OwnedHandle
    def public_free()
      @operations.key_contact_public_free(@native_handle)
      @native_handle = nil
    end

    def encrypt(content_key)
      WrappedContactKey.new(@operations, @operations.key_contact_encrypt(@native_handle, content_key))
    end

  end

  class WrappedContactKey < OwnedHandle
    def public()
      @operations.key_contact_wrapped_public(@native_handle)
    end

    def ciphertext()
      @operations.key_contact_wrapped_ciphertext(@native_handle)
    end

    def encrypted()
      @operations.key_contact_wrapped_encrypted(@native_handle)
    end

    def free()
      @operations.key_contact_wrapped_free(@native_handle)
      @native_handle = nil
    end

  end

  class SigningKeyPair < OwnedHandle
    def public()
      @operations.key_signing_public(@native_handle)
    end

    def private()
      @operations.key_signing_private(@native_handle)
    end

    def free()
      @operations.key_signing_free(@native_handle)
      @native_handle = nil
    end

  end

  class SigningPublicKey < OwnedHandle
    def public_free()
      @operations.key_signing_public_free(@native_handle)
      @native_handle = nil
    end

  end

  class VaultDirectory < OwnedHandle
    def root()
      @operations.vault_directory_root(@native_handle)
    end

    def structure_version()
      @operations.vault_directory_structure_version(@native_handle)
    end

    def list_private_keys()
      @operations.vault_directory_list_private_keys(@native_handle)
    end

    def list_private_key_names()
      @operations.vault_directory_list_private_key_names(@native_handle)
    end

    def list_contact_names()
      @operations.vault_directory_list_contact_names(@native_handle)
    end

    def list_form_aliases()
      @operations.vault_directory_list_form_aliases(@native_handle)
    end

    def private_key_exists(name)
      @operations.vault_directory_private_key_exists(@native_handle, name)
    end

    def delete_private_key(name)
      @operations.vault_directory_delete_private_key(@native_handle, name)
    end

    def store_private_key(name, key)
      @operations.vault_directory_store_private_key(@native_handle, name, key.native_handle)
    end

    def load_private_key(name)
      ContactKeyPair.new(@operations, @operations.vault_directory_load_private_key(@native_handle, name))
    end

    def load_private_key_generation(name, index)
      ContactKeyPair.new(@operations, @operations.vault_directory_load_private_key_generation(@native_handle, name, index))
    end

    def store_contact(name, key)
      @operations.vault_directory_store_contact(@native_handle, name, key.native_handle)
    end

    def load_contact(name)
      ContactPublicKey.new(@operations, @operations.vault_directory_load_contact(@native_handle, name))
    end

    def contact_exists(name)
      @operations.vault_directory_contact_exists(@native_handle, name)
    end

    def delete_contact(name)
      @operations.vault_directory_delete_contact(@native_handle, name)
    end

    def list_contacts()
      @operations.vault_directory_list_contacts(@native_handle)
    end

    def store_profile_email(name, email)
      @operations.vault_directory_store_profile_email(@native_handle, name, email)
    end

    def profile_email(name)
      @operations.vault_directory_profile_email(@native_handle, name)
    end

    def store_backup(id, bytes)
      @operations.vault_directory_store_backup(@native_handle, id, bytes)
    end

    def load_backup(id)
      @operations.vault_directory_load_backup(@native_handle, id)
    end

    def backup_count()
      @operations.vault_directory_backup_count(@native_handle)
    end

    def restore_private_key(name, key, signing_key, overwrite)
      @operations.vault_directory_restore_private_key(@native_handle, name, key.native_handle, signing_key.native_handle, overwrite)
    end

    def load_owner_signing_key(name)
      SigningKeyPair.new(@operations, @operations.vault_directory_load_owner_signing_key(@native_handle, name))
    end

    def load_owner_signing_key_generation(name, index)
      SigningKeyPair.new(@operations, @operations.vault_directory_load_owner_signing_key_generation(@native_handle, name, index))
    end

    def store_contact_signing_key(name, key)
      @operations.vault_directory_store_contact_signing_key(@native_handle, name, key.native_handle)
    end

    def load_contact_signing_key(name)
      SigningPublicKey.new(@operations, @operations.vault_directory_load_contact_signing_key(@native_handle, name))
    end

    def list_profile_generations(name)
      @operations.vault_directory_list_profile_generations(@native_handle, name)
    end

    def rotate_private_key(name)
      @operations.vault_directory_rotate_private_key(@native_handle, name)
    end

    def remember_lockbox(id, path)
      @operations.vault_directory_remember_lockbox(@native_handle, id, path)
    end

    def list_known_lockboxes()
      @operations.vault_directory_list_known_lockboxes(@native_handle)
    end

    def forget_lockbox(path)
      @operations.vault_directory_forget_lockbox(@native_handle, path)
    end

    def remember_access_slot_label(id, slot_id, name)
      @operations.vault_directory_remember_access_slot_label(@native_handle, id, slot_id, name)
    end

    def list_access_slot_labels(id)
      @operations.vault_directory_list_access_slot_labels(@native_handle, id)
    end

    def find_access_slot_labels(id, name)
      @operations.vault_directory_find_access_slot_labels(@native_handle, id, name)
    end

    def forget_access_slot_label(id, slot_id)
      @operations.vault_directory_forget_access_slot_label(@native_handle, id, slot_id)
    end

    def define_form(alias_name, name, description, fields_proto)
      @operations.vault_directory_define_form(@native_handle, alias_name, name, description, fields_proto)
    end

    def resolve_form(reference)
      @operations.vault_directory_resolve_form(@native_handle, reference)
    end

    def list_forms()
      @operations.vault_directory_list_forms(@native_handle)
    end

    def list_form_revisions(type_id)
      @operations.vault_directory_list_form_revisions(@native_handle, type_id)
    end

    def seed_forms()
      @operations.vault_directory_seed_forms(@native_handle)
    end

    def remember_password(id, password)
      @operations.vault_directory_remember_password(@native_handle, id, password)
    end

    def remembered_password(id)
      @operations.vault_directory_remembered_password(@native_handle, id)
    end

    def free()
      @operations.vault_directory_free(@native_handle)
      @native_handle = nil
    end

  end

  class ReadOnlyVaultDirectory < OwnedHandle
    def list_profile_names()
      @operations.vault_read_only_list_profile_names(@native_handle)
    end

    def list_contact_names()
      @operations.vault_read_only_list_contact_names(@native_handle)
    end

    def list_form_aliases()
      @operations.vault_read_only_list_form_aliases(@native_handle)
    end

    def list_known_lockboxes()
      @operations.vault_read_only_list_known_lockboxes(@native_handle)
    end

    def free()
      @operations.vault_read_only_free(@native_handle)
    end

  end

  class Agent
    def initialize(operations)
      @operations = operations
    end

    def is_running()
      @operations.vault_is_running()
    end

    def forget_all()
      @operations.vault_forget_all()
    end

    def serve()
      @operations.vault_agent_serve()
    end

    def verify_transport()
      @operations.vault_agent_verify_transport()
    end

    def get(id)
      @operations.vault_agent_get(id)
    end

    def put(id, key)
      @operations.vault_agent_put(id, key)
    end

    def forget(id)
      @operations.vault_agent_forget(id)
    end

    def stop()
      @operations.vault_agent_stop()
    end

    def start()
      @operations.vault_agent_start()
    end

    def list()
      @operations.vault_agent_list()
    end

    def sleep_support()
      @operations.vault_agent_sleep_support()
    end

    def get_vault_unlock_key(vault_id)
      @operations.vault_agent_get_vault_unlock_key(vault_id)
    end

    def put_vault_unlock_key(vault_id, key, ttl_seconds)
      @operations.vault_agent_put_vault_unlock_key(vault_id, key, ttl_seconds)
    end

    def forget_vault_unlock_key(vault_id)
      @operations.vault_agent_forget_vault_unlock_key(vault_id)
    end

    def get_owner_signing_key(vault_id, profile)
      SigningKeyPair.new(@operations, @operations.vault_agent_get_owner_signing_key(vault_id, profile))
    end

    def put_owner_signing_key(vault_id, profile, key, ttl_seconds)
      @operations.vault_agent_put_owner_signing_key(vault_id, profile, key.native_handle, ttl_seconds)
    end

    def forget_owner_signing_key(vault_id, profile)
      @operations.vault_agent_forget_owner_signing_key(vault_id, profile)
    end

    def begin_activity(kind)
      AgentActivity.new(@operations, @operations.vault_agent_begin_activity(kind))
    end

    def end_activity(handle)
      @operations.vault_agent_end_activity(handle.native_handle)
    end

  end

  class AgentActivity < OwnedHandle
  end

  class Platform
    def initialize(operations)
      @operations = operations
    end

    def status()
      @operations.vault_platform_status()
    end

    def set_scope(scope)
      @operations.vault_platform_set_scope(scope)
    end

    def forget_password()
      @operations.vault_platform_forget_password()
    end

    def put_password(password)
      @operations.vault_platform_put_password(password)
    end

    def enable()
      @operations.vault_platform_enable()
    end

    def disable()
      @operations.vault_platform_disable()
    end

    def disabled()
      @operations.vault_platform_disabled()
    end

    def get_password()
      @operations.vault_platform_get_password()
    end

  end

  class LocalVault < OwnedHandle
    def create_lockbox_password(path, password)
      Lockbox.new(@operations, @operations.vault_create_lockbox_password(@native_handle, path, password))
    end

    def open_lockbox_password(path, password)
      Lockbox.new(@operations, @operations.vault_open_lockbox_password(@native_handle, path, password))
    end

    def create_lockbox_content_key(path, content_key, signing_key)
      Lockbox.new(@operations, @operations.vault_create_lockbox_content_key(@native_handle, path, content_key, signing_key.native_handle))
    end

    def create_lockbox_contact(path, contact, name, signing_key)
      Lockbox.new(@operations, @operations.vault_create_lockbox_contact(@native_handle, path, contact.native_handle, name, signing_key.native_handle))
    end

    def open_lockbox_content_key(path, content_key, signing_key)
      Lockbox.new(@operations, @operations.vault_open_lockbox_content_key(@native_handle, path, content_key, signing_key.native_handle))
    end

    def cache_lockbox_password(path, password, ttl_seconds)
      @operations.vault_cache_lockbox_password(@native_handle, path, password, ttl_seconds)
    end

    def close_lockbox(path)
      @operations.vault_close_lockbox(@native_handle, path)
    end

    def close_all()
      @operations.vault_close_all(@native_handle)
    end

    def free()
      @operations.vault_free(@native_handle)
      @native_handle = nil
    end

  end

end
