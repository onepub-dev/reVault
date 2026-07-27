# Generated complete binary operation layer. Do not edit.
require 'fiddle'
require 'fiddle/import'
require_relative 'domain_models'
require_relative 'native_library'

module Revault
  module Native
    extend Fiddle::Importer
    dlload Revault::NativeLibrary.path
    BufferStruct = struct ['void *ptr', 'size_t len']
    extern 'uint32_t api_abi_version(void)'
    extern 'const char * buffer_last_error(void)'
    extern 'bool secret_len(void *, size_t *)'
    extern 'bool secret_copy(void *, void *, size_t)'
    extern 'void secret_free(void *)'
    extern 'uint16_t lockbox_format_version(void)'
    extern 'uint16_t lockbox_probe_format_version(void *, size_t)'
    extern 'void * lockbox_create(void *, size_t)'
    extern 'void * lockbox_create_with_options(void *, size_t, void *, size_t, uint64_t, void *, size_t, void *, size_t, size_t)'
    extern 'void * lockbox_create_password(void *, size_t)'
    extern 'void * lockbox_create_contact(void *)'
    extern 'void * lockbox_create_with_signing_key(void *, size_t, void *)'
    extern 'void * lockbox_open(void *, size_t, void *, size_t)'
    extern 'void * lockbox_open_with_options(void *, size_t, void *, size_t, void *, size_t, uint64_t, void *, size_t, void *, size_t, size_t)'
    extern 'void * lockbox_open_password(void *, size_t, void *, size_t)'
    extern 'void * lockbox_open_contact(void *, size_t, void *)'
    extern 'bool lockbox_add_file(void *, void *, size_t, void *, size_t, bool)'
    extern 'bool lockbox_add_file_with_permissions(void *, void *, size_t, void *, size_t, uint32_t, bool)'
    extern 'bool lockbox_extract_file(void *, void *, size_t, void *, size_t, bool)'
    extern 'bool lockbox_extract_directory(void *, void *, size_t, uint64_t, uint64_t, size_t, bool, bool, bool)'
    extern 'bool lockbox_reset_import_stats(void *)'
    extern 'uint64_t lockbox_storage_len(void *)'
    extern 'bool lockbox_set_workload_profile(void *, void *, size_t)'
    extern 'bool lockbox_set_worker_policy(void *, void *, size_t, size_t)'
    extern 'bool lockbox_commit(void *)'
    extern 'bool lockbox_create_dir(void *, void *, size_t, bool)'
    extern 'bool lockbox_delete(void *, void *, size_t)'
    extern 'bool lockbox_remove_dir(void *, void *, size_t, bool)'
    extern 'bool lockbox_create_parent_dirs(void *, void *, size_t)'
    extern 'bool lockbox_rename(void *, void *, size_t, void *, size_t)'
    extern 'bool lockbox_set_variable(void *, void *, size_t, void *, size_t)'
    extern 'bool lockbox_set_secret_variable(void *, void *, size_t, void *, size_t)'
    extern 'bool lockbox_get_secret_variable(void *, void *, size_t, void *)'
    extern 'bool lockbox_delete_variable(void *, void *, size_t)'
    extern 'bool lockbox_move_variables(void *, void *, size_t)'
    extern 'bool lockbox_add_symlink(void *, void *, size_t, void *, size_t, bool)'
    extern 'bool lockbox_exists(void *, void *, size_t)'
    extern 'bool lockbox_is_dir(void *, void *, size_t)'
    extern 'uint32_t lockbox_permissions(void *, void *, size_t)'
    extern 'bool lockbox_set_permissions(void *, void *, size_t, uint32_t)'
    extern 'void * lockbox_recovery_salvage(void *, size_t, void *, size_t, void *)'
    extern 'uint64_t lockbox_add_password(void *, void *, size_t)'
    extern 'uint64_t lockbox_add_contact(void *, void *, void *, size_t)'
    extern 'bool lockbox_delete_key(void *, uint64_t)'
    extern 'bool lockbox_set_owner_signing_key(void *, void *)'
    extern 'bool lockbox_set_form_field(void *, void *, size_t, void *, size_t, void *, size_t)'
    extern 'bool lockbox_set_secret_form_field(void *, void *, size_t, void *, size_t, void *, size_t)'
    extern 'bool lockbox_get_secret_form_field(void *, void *, size_t, void *, size_t, void *)'
    extern 'bool lockbox_delete_form_record(void *, void *, size_t)'
    extern 'bool lockbox_move_form_records(void *, void *, size_t)'
    extern 'void lockbox_free(void *)'
    extern 'bool vault_is_running(void)'
    extern 'bool vault_forget_all(void)'
    extern 'void * key_contact_generate(void)'
    extern 'void * key_contact_from_private(void *, size_t)'
    extern 'void * key_contact_public_from_bytes(void *, size_t)'
    extern 'void key_contact_public_free(void *)'
    extern 'void key_contact_free(void *)'
    extern 'void * key_contact_encrypt(void *, void *, size_t)'
    extern 'void key_contact_wrapped_free(void *)'
    extern 'void * key_signing_generate(void)'
    extern 'void * key_signing_from_private(void *, size_t)'
    extern 'void * key_signing_public_from_bytes(void *, size_t)'
    extern 'void key_signing_public_free(void *)'
    extern 'void key_signing_free(void *)'
    extern 'void * vault_key_import_private(void *, size_t)'
    extern 'void * vault_key_import_public(void *, size_t)'
    extern 'void * vault_directory_open(void *, size_t, void *, size_t)'
    extern 'uint32_t vault_structure_version_current(void)'
    extern 'uint32_t vault_directory_probe_structure_version(void *, size_t, void *, size_t)'
    extern 'void * vault_directory_open_or_create_default(void *, size_t)'
    extern 'void * vault_directory_replace_default(void *, size_t)'
    extern 'bool vault_directory_change_password(void *, size_t, void *, size_t, void *, size_t)'
    extern 'bool vault_directory_change_default_password(void *, size_t, void *, size_t)'
    extern 'void * vault_directory_replace(void *, size_t, void *, size_t)'
    extern 'void * vault_directory_open_or_create(void *, size_t, void *, size_t)'
    extern 'uint32_t vault_directory_structure_version(void *)'
    extern 'bool vault_directory_private_key_exists(void *, void *, size_t)'
    extern 'bool vault_directory_delete_private_key(void *, void *, size_t)'
    extern 'bool vault_directory_store_private_key(void *, void *, size_t, void *)'
    extern 'void * vault_directory_load_private_key(void *, void *, size_t)'
    extern 'void * vault_directory_load_private_key_generation(void *, void *, size_t, uint16_t)'
    extern 'bool vault_directory_store_contact(void *, void *, size_t, void *)'
    extern 'void * vault_directory_load_contact(void *, void *, size_t)'
    extern 'bool vault_directory_contact_exists(void *, void *, size_t)'
    extern 'bool vault_directory_delete_contact(void *, void *, size_t)'
    extern 'bool vault_directory_store_profile_email(void *, void *, size_t, void *, size_t)'
    extern 'bool vault_directory_store_backup(void *, void *, size_t, void *, size_t)'
    extern 'uint64_t vault_directory_backup_count(void *)'
    extern 'bool vault_directory_restore_private_key(void *, void *, size_t, void *, void *, bool)'
    extern 'void * vault_directory_load_owner_signing_key(void *, void *, size_t)'
    extern 'void * vault_directory_load_owner_signing_key_generation(void *, void *, size_t, uint16_t)'
    extern 'bool vault_directory_store_contact_signing_key(void *, void *, size_t, void *)'
    extern 'void * vault_directory_load_contact_signing_key(void *, void *, size_t)'
    extern 'bool vault_directory_remember_lockbox(void *, void *, size_t, void *, size_t)'
    extern 'bool vault_directory_forget_lockbox(void *, void *, size_t)'
    extern 'bool vault_directory_remember_access_slot_label(void *, void *, size_t, uint64_t, void *, size_t)'
    extern 'bool vault_directory_forget_access_slot_label(void *, void *, size_t, uint64_t)'
    extern 'size_t vault_directory_seed_forms(void *)'
    extern 'bool vault_directory_remember_password(void *, void *, size_t, void *, size_t)'
    extern 'void vault_directory_free(void *)'
    extern 'void * vault_read_only_open(void *, size_t, void *, size_t)'
    extern 'void * vault_read_only_open_default(void *, size_t)'
    extern 'void vault_read_only_free(void *)'
    extern 'bool vault_agent_serve(void)'
    extern 'bool vault_agent_verify_transport(void)'
    extern 'bool vault_agent_put(void *, size_t, void *, size_t)'
    extern 'bool vault_agent_forget(void *, size_t)'
    extern 'bool vault_agent_stop(void)'
    extern 'bool vault_agent_start(void)'
    extern 'bool vault_platform_set_scope(void *, size_t)'
    extern 'bool vault_platform_forget_password(void)'
    extern 'bool vault_platform_put_password(void *, size_t)'
    extern 'bool vault_platform_enable(void)'
    extern 'bool vault_platform_disable(void)'
    extern 'bool vault_platform_disabled(void)'
    extern 'bool vault_agent_put_vault_unlock_key(void *, size_t, void *, size_t, uint64_t)'
    extern 'bool vault_agent_forget_vault_unlock_key(void *, size_t)'
    extern 'void * vault_agent_get_owner_signing_key(void *, size_t, void *, size_t)'
    extern 'bool vault_agent_put_owner_signing_key(void *, size_t, void *, size_t, void *, uint64_t)'
    extern 'bool vault_agent_forget_owner_signing_key(void *, size_t, void *, size_t)'
    extern 'void * vault_agent_begin_activity(void *, size_t)'
    extern 'void vault_agent_end_activity(void *)'
    extern 'void * vault_local(void)'
    extern 'void * vault_create_lockbox_password(void *, void *, size_t, void *, size_t)'
    extern 'void * vault_open_lockbox_password(void *, void *, size_t, void *, size_t)'
    extern 'void * vault_create_lockbox_content_key(void *, void *, size_t, void *, size_t, void *)'
    extern 'void * vault_create_lockbox_contact(void *, void *, size_t, void *, void *, size_t, void *)'
    extern 'void * vault_open_lockbox_content_key(void *, void *, size_t, void *, size_t, void *)'
    extern 'bool vault_cache_lockbox_password(void *, void *, size_t, void *, size_t, uint64_t)'
    extern 'bool vault_close_lockbox(void *, void *, size_t)'
    extern 'bool vault_close_all(void *)'
    extern 'void vault_free(void *)'
  end

  module Shim
    extend Fiddle::Importer
    dlload Revault::NativeLibrary.shim_path
    extern 'void ruby_buffer_free(void *)'
    extern 'void ruby_buffer_last_error_details(void *)'
    extern 'void ruby_lockbox_get_file(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_stream_content(void *, bool, void *)'
    extern 'void ruby_lockbox_cache_stats(void *, void *)'
    extern 'void ruby_lockbox_import_stats(void *, void *)'
    extern 'void ruby_lockbox_inspect_file(void *, size_t, void *)'
    extern 'void ruby_lockbox_page_inspection(void *, void *)'
    extern 'void ruby_lockbox_recovery_report(void *, void *)'
    extern 'void ruby_lockbox_recovery_report_render(void *, bool, size_t, void *)'
    extern 'void ruby_lockbox_recovery_scan_path(void *, size_t, void *, size_t, void *)'
    extern 'void ruby_lockbox_runtime_options(void *, void *)'
    extern 'void ruby_lockbox_list(void *, void *, size_t, bool, void *)'
    extern 'void ruby_lockbox_list_with_options(void *, void *, size_t, void *, size_t, bool, bool, bool, bool, size_t, void *)'
    extern 'void ruby_lockbox_stat(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_get_variable(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_list_variables(void *, void *)'
    extern 'void ruby_lockbox_variable_sensitivity(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_get_symlink_target(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_id(void *, void *)'
    extern 'void ruby_lockbox_read_range(void *, void *, size_t, uint64_t, uint64_t, void *)'
    extern 'void ruby_lockbox_recovery_scan(void *, size_t, void *, size_t, void *)'
    extern 'void ruby_lockbox_list_key_slots(void *, void *)'
    extern 'void ruby_lockbox_owner_inspection(void *, void *)'
    extern 'void ruby_lockbox_define_form(void *, void *, size_t, void *, size_t, void *, size_t, void *, size_t, void *)'
    extern 'void ruby_lockbox_list_form_definitions(void *, void *)'
    extern 'void ruby_lockbox_resolve_form(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_list_form_revisions(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_create_form_record(void *, void *, size_t, void *, size_t, void *, size_t, void *)'
    extern 'void ruby_lockbox_list_form_records(void *, void *)'
    extern 'void ruby_lockbox_get_form_record(void *, void *, size_t, void *)'
    extern 'void ruby_lockbox_get_form_field(void *, void *, size_t, void *, size_t, void *)'
    extern 'void ruby_lockbox_to_bytes(void *, void *)'
    extern 'void ruby_key_contact_public(void *, void *)'
    extern 'void ruby_key_contact_private(void *, void *)'
    extern 'void ruby_key_contact_decrypt(void *, void *, void *)'
    extern 'void ruby_key_contact_wrapped_public(void *, void *)'
    extern 'void ruby_key_contact_wrapped_ciphertext(void *, void *)'
    extern 'void ruby_key_contact_wrapped_encrypted(void *, void *)'
    extern 'void ruby_key_signing_public(void *, void *)'
    extern 'void ruby_key_signing_private(void *, void *)'
    extern 'void ruby_vault_key_export_private(void *, void *, size_t, void *)'
    extern 'void ruby_vault_key_export_public(void *, void *, size_t, void *)'
    extern 'void ruby_vault_key_fingerprint(void *, void *)'
    extern 'void ruby_vault_key_format_hex(void *, size_t, void *)'
    extern 'void ruby_vault_key_decode_hex(void *, size_t, void *)'
    extern 'void ruby_vault_key_format_crockford(void *, size_t, void *)'
    extern 'void ruby_vault_key_format_crockford_reading(void *, size_t, void *)'
    extern 'void ruby_vault_key_decode_crockford(void *, size_t, void *)'
    extern 'void ruby_vault_key_hex_encode(void *, size_t, void *)'
    extern 'void ruby_vault_key_hex_decode(void *, size_t, void *)'
    extern 'void ruby_vault_directory_root(void *, void *)'
    extern 'void ruby_vault_directory_list_private_keys(void *, void *)'
    extern 'void ruby_vault_directory_list_private_key_names(void *, void *)'
    extern 'void ruby_vault_directory_list_contact_names(void *, void *)'
    extern 'void ruby_vault_directory_list_form_aliases(void *, void *)'
    extern 'void ruby_vault_directory_list_contacts(void *, void *)'
    extern 'void ruby_vault_directory_profile_email(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_load_backup(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_list_profile_generations(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_rotate_private_key(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_list_known_lockboxes(void *, void *)'
    extern 'void ruby_vault_directory_list_access_slot_labels(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_find_access_slot_labels(void *, void *, size_t, void *, size_t, void *)'
    extern 'void ruby_vault_directory_define_form(void *, void *, size_t, void *, size_t, void *, size_t, void *, size_t, void *)'
    extern 'void ruby_vault_directory_resolve_form(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_list_forms(void *, void *)'
    extern 'void ruby_vault_directory_list_form_revisions(void *, void *, size_t, void *)'
    extern 'void ruby_vault_directory_remembered_password(void *, void *, size_t, void *)'
    extern 'void ruby_vault_backup_default(void *, size_t, bool, void *)'
    extern 'void ruby_vault_restore_default(void *, size_t, bool, void *)'
    extern 'void ruby_vault_read_only_list_profile_names(void *, void *)'
    extern 'void ruby_vault_read_only_list_contact_names(void *, void *)'
    extern 'void ruby_vault_read_only_list_form_aliases(void *, void *)'
    extern 'void ruby_vault_read_only_list_known_lockboxes(void *, void *)'
    extern 'void ruby_vault_agent_get(void *, size_t, void *)'
    extern 'void ruby_vault_agent_list(void *)'
    extern 'void ruby_vault_agent_sleep_support(void *)'
    extern 'void ruby_vault_platform_status(void *)'
    extern 'void ruby_vault_platform_get_password(void *)'
    extern 'void ruby_vault_default_directory(void *)'
    extern 'void ruby_vault_default_path(void *)'
    extern 'void ruby_vault_agent_log_path(void *)'
    extern 'void ruby_vault_agent_log_destination(void *)'
    extern 'void ruby_vault_agent_get_vault_unlock_key(void *, size_t, void *)'
  end

  class BindingOperations
    def initialize = raise('revault-api native ABI mismatch; expected 3') unless Native.api_abi_version == 3
    def last_error_message = Native.buffer_last_error.to_s
    def require_value(value)
      raise last_error_message unless value
      value
    end
    def require_handle(value)
      raise last_error_message if value.nil? || value.null?
      value
    end
    def buffer_call(symbol, *arguments)
      value = Native::BufferStruct.malloc
      Shim.public_send("ruby_#{symbol}", *arguments, value)
      value
    end
    def take(value)
      raise last_error_message if value.ptr.null?
      value.ptr.to_s(value.len)
    ensure
      Shim.ruby_buffer_free(value) if value && !value.ptr.null?
    end
    def with_secret(getter)
      output = Fiddle::Pointer.malloc(Fiddle::SIZEOF_VOIDP)
      raise last_error_message unless getter.call(output)
      address = output[0, Fiddle::SIZEOF_VOIDP].unpack1('J')
      return nil if address.zero?
      handle = Fiddle::Pointer.new(address)
      begin
        length_out = Fiddle::Pointer.malloc(Fiddle::SIZEOF_SIZE_T)
        raise last_error_message unless Native.secret_len(handle, length_out)
        length = length_out[0, Fiddle::SIZEOF_SIZE_T].unpack1('J')
        native = Fiddle::Pointer.malloc([length, 1].max)
        raise last_error_message unless Native.secret_copy(handle, native, length)
        secret = native.to_s(length)
        begin
          yield secret, length
        ensure
          secret.replace("\0" * secret.bytesize)
          native[0, [length, 1].max] = "\0" * [length, 1].max
        end
      ensure
        Native.secret_free(handle)
      end
    end

    def buffer_last_error_details()
      Revault::Internal::DomainCodec.decode('ErrorDetails', take(buffer_call(:buffer_last_error_details)))
    end

    def lockbox_format_version()
      Native.lockbox_format_version()
    end

    def lockbox_probe_format_version(bytes)
      Native.lockbox_probe_format_version(Fiddle::Pointer[bytes], bytes.bytesize)
    end

    def lockbox_create(key)
      require_handle(Native.lockbox_create(Fiddle::Pointer[key], key.bytesize))
    end

    def lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
      require_handle(Native.lockbox_create_with_options(Fiddle::Pointer[key], key.bytesize, Fiddle::Pointer[cache_mode], cache_mode.bytesize, cache_bytes, Fiddle::Pointer[workload], workload.bytesize, Fiddle::Pointer[worker], worker.bytesize, jobs))
    end

    def lockbox_create_password(password)
      require_handle(Native.lockbox_create_password(Fiddle::Pointer[password], password.bytesize))
    end

    def lockbox_create_contact(contact)
      require_handle(Native.lockbox_create_contact(contact))
    end

    def lockbox_create_with_signing_key(content_key, signing_key)
      require_handle(Native.lockbox_create_with_signing_key(Fiddle::Pointer[content_key], content_key.bytesize, signing_key))
    end

    def lockbox_open(archive, key)
      require_handle(Native.lockbox_open(Fiddle::Pointer[archive], archive.bytesize, Fiddle::Pointer[key], key.bytesize))
    end

    def lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
      require_handle(Native.lockbox_open_with_options(Fiddle::Pointer[archive], archive.bytesize, Fiddle::Pointer[key], key.bytesize, Fiddle::Pointer[cache_mode], cache_mode.bytesize, cache_bytes, Fiddle::Pointer[workload], workload.bytesize, Fiddle::Pointer[worker], worker.bytesize, jobs))
    end

    def lockbox_open_password(archive, password)
      require_handle(Native.lockbox_open_password(Fiddle::Pointer[archive], archive.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def lockbox_open_contact(archive, contact)
      require_handle(Native.lockbox_open_contact(Fiddle::Pointer[archive], archive.bytesize, contact))
    end

    def lockbox_add_file(handle, path, data, replace)
      require_value(Native.lockbox_add_file(handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[data], data.bytesize, replace))
    end

    def lockbox_add_file_with_permissions(handle, path, data, permissions, replace)
      require_value(Native.lockbox_add_file_with_permissions(handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[data], data.bytesize, permissions, replace))
    end

    def lockbox_get_file(handle, path)
      take(buffer_call(:lockbox_get_file, handle, Fiddle::Pointer[path], path.bytesize))
    end

    def lockbox_extract_file(handle, source, destination, replace)
      require_value(Native.lockbox_extract_file(handle, Fiddle::Pointer[source], source.bytesize, Fiddle::Pointer[destination], destination.bytesize, replace))
    end

    def lockbox_extract_directory(handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
      require_value(Native.lockbox_extract_directory(handle, Fiddle::Pointer[destination], destination.bytesize, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite))
    end

    def lockbox_stream_content(handle, physical)
      Revault::Internal::DomainCodec.decode('StreamChunkList', take(buffer_call(:lockbox_stream_content, handle, physical)))
    end

    def lockbox_cache_stats(handle)
      Revault::Internal::DomainCodec.decode('CacheStats', take(buffer_call(:lockbox_cache_stats, handle)))
    end

    def lockbox_import_stats(handle)
      Revault::Internal::DomainCodec.decode('ImportStats', take(buffer_call(:lockbox_import_stats, handle)))
    end

    def lockbox_reset_import_stats(handle)
      require_value(Native.lockbox_reset_import_stats(handle))
    end

    def lockbox_inspect_file(path)
      Revault::Internal::DomainCodec.decode('FileInspection', take(buffer_call(:lockbox_inspect_file, Fiddle::Pointer[path], path.bytesize)))
    end

    def lockbox_page_inspection(handle)
      Revault::Internal::DomainCodec.decode('PageInspectionList', take(buffer_call(:lockbox_page_inspection, handle)))
    end

    def lockbox_recovery_report(handle)
      Revault::Internal::DomainCodec.decode('RecoveryReport', take(buffer_call(:lockbox_recovery_report, handle)))
    end

    def lockbox_recovery_report_render(handle, verbose, max_entries)
      take(buffer_call(:lockbox_recovery_report_render, handle, verbose, max_entries))
    end

    def lockbox_recovery_scan_path(path, key)
      Revault::Internal::DomainCodec.decode('RecoveryReport', take(buffer_call(:lockbox_recovery_scan_path, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[key], key.bytesize)))
    end

    def lockbox_storage_len(handle)
      Native.lockbox_storage_len(handle)
    end

    def lockbox_set_workload_profile(handle, profile)
      require_value(Native.lockbox_set_workload_profile(handle, Fiddle::Pointer[profile], profile.bytesize))
    end

    def lockbox_set_worker_policy(handle, mode, jobs)
      require_value(Native.lockbox_set_worker_policy(handle, Fiddle::Pointer[mode], mode.bytesize, jobs))
    end

    def lockbox_runtime_options(handle)
      Revault::Internal::DomainCodec.decode('RuntimeOptions', take(buffer_call(:lockbox_runtime_options, handle)))
    end

    def lockbox_commit(handle)
      require_value(Native.lockbox_commit(handle))
    end

    def lockbox_create_dir(handle, path, create_parents)
      require_value(Native.lockbox_create_dir(handle, Fiddle::Pointer[path], path.bytesize, create_parents))
    end

    def lockbox_delete(handle, path)
      require_value(Native.lockbox_delete(handle, Fiddle::Pointer[path], path.bytesize))
    end

    def lockbox_remove_dir(handle, path, recursive)
      require_value(Native.lockbox_remove_dir(handle, Fiddle::Pointer[path], path.bytesize, recursive))
    end

    def lockbox_create_parent_dirs(handle, path)
      require_value(Native.lockbox_create_parent_dirs(handle, Fiddle::Pointer[path], path.bytesize))
    end

    def lockbox_rename(handle, from, to)
      require_value(Native.lockbox_rename(handle, Fiddle::Pointer[from], from.bytesize, Fiddle::Pointer[to], to.bytesize))
    end

    def lockbox_list(handle, path, recursive)
      Revault::Internal::DomainCodec.decode('LockboxEntryList', take(buffer_call(:lockbox_list, handle, Fiddle::Pointer[path], path.bytesize, recursive)))
    end

    def lockbox_list_with_options(handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
      Revault::Internal::DomainCodec.decode('LockboxEntryList', take(buffer_call(:lockbox_list_with_options, handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[glob], glob.bytesize, recursive, include_files, include_symlinks, include_directories, limit)))
    end

    def lockbox_stat(handle, path)
      Revault::Internal::DomainCodec.decode('OptionalLockboxEntry', take(buffer_call(:lockbox_stat, handle, Fiddle::Pointer[path], path.bytesize)))
    end

    def lockbox_set_variable(handle, name, value)
      require_value(Native.lockbox_set_variable(handle, Fiddle::Pointer[name], name.bytesize, Fiddle::Pointer[value], value.bytesize))
    end

    def lockbox_set_secret_variable(handle, name, value)
      secret = value.dup
      require_value(Native.lockbox_set_secret_variable(handle, Fiddle::Pointer[name], name.bytesize, Fiddle::Pointer[secret], secret.bytesize))
    ensure
      secret&.replace("\0" * secret.bytesize)
    end

    def lockbox_get_variable(handle, name)
      value = Revault::Internal::DomainCodec.decode('OptionalString', take(buffer_call(:lockbox_get_variable, handle, Fiddle::Pointer[name], name.bytesize)))
      value.present ? value.value : nil
    end

    def lockbox_get_secret_variable(handle, name, &callback)
      with_secret(->(output) { Native.lockbox_get_secret_variable(handle, Fiddle::Pointer[name], name.bytesize, output) }, &callback)
    end

    def lockbox_delete_variable(handle, name)
      require_value(Native.lockbox_delete_variable(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def lockbox_move_variables(handle, moves_flatbuffer)
      require_value(Native.lockbox_move_variables(handle, Fiddle::Pointer[moves_flatbuffer], moves_flatbuffer.bytesize))
    end

    def lockbox_list_variables(handle)
      Revault::Internal::DomainCodec.decode('VariableList', take(buffer_call(:lockbox_list_variables, handle)))
    end

    def lockbox_variable_sensitivity(handle, name)
      Revault::Internal::DomainCodec.decode('OptionalString', take(buffer_call(:lockbox_variable_sensitivity, handle, Fiddle::Pointer[name], name.bytesize)))
    end

    def lockbox_add_symlink(handle, path, target, replace)
      require_value(Native.lockbox_add_symlink(handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[target], target.bytesize, replace))
    end

    def lockbox_get_symlink_target(handle, path)
      take(buffer_call(:lockbox_get_symlink_target, handle, Fiddle::Pointer[path], path.bytesize))
    end

    def lockbox_id(handle)
      take(buffer_call(:lockbox_id, handle))
    end

    def lockbox_exists(handle, path)
      Native.lockbox_exists(handle, Fiddle::Pointer[path], path.bytesize)
    end

    def lockbox_is_dir(handle, path)
      Native.lockbox_is_dir(handle, Fiddle::Pointer[path], path.bytesize)
    end

    def lockbox_permissions(handle, path)
      Native.lockbox_permissions(handle, Fiddle::Pointer[path], path.bytesize)
    end

    def lockbox_set_permissions(handle, path, permissions)
      require_value(Native.lockbox_set_permissions(handle, Fiddle::Pointer[path], path.bytesize, permissions))
    end

    def lockbox_read_range(handle, path, offset, len)
      take(buffer_call(:lockbox_read_range, handle, Fiddle::Pointer[path], path.bytesize, offset, len))
    end

    def lockbox_recovery_scan(bytes, key)
      Revault::Internal::DomainCodec.decode('RecoveryReport', take(buffer_call(:lockbox_recovery_scan, Fiddle::Pointer[bytes], bytes.bytesize, Fiddle::Pointer[key], key.bytesize)))
    end

    def lockbox_recovery_salvage(bytes, key, signing_key)
      require_handle(Native.lockbox_recovery_salvage(Fiddle::Pointer[bytes], bytes.bytesize, Fiddle::Pointer[key], key.bytesize, signing_key))
    end

    def lockbox_add_password(handle, password)
      Native.lockbox_add_password(handle, Fiddle::Pointer[password], password.bytesize)
    end

    def lockbox_add_contact(handle, contact, name)
      Native.lockbox_add_contact(handle, contact, Fiddle::Pointer[name], name.bytesize)
    end

    def lockbox_delete_key(handle, id)
      require_value(Native.lockbox_delete_key(handle, id))
    end

    def lockbox_list_key_slots(handle)
      Revault::Internal::DomainCodec.decode('KeySlotList', take(buffer_call(:lockbox_list_key_slots, handle)))
    end

    def lockbox_set_owner_signing_key(handle, key)
      require_value(Native.lockbox_set_owner_signing_key(handle, key))
    end

    def lockbox_owner_inspection(handle)
      Revault::Internal::DomainCodec.decode('OwnerInspection', take(buffer_call(:lockbox_owner_inspection, handle)))
    end

    def lockbox_define_form(handle, alias_name, name, description, fields_flatbuffer)
      Revault::Internal::DomainCodec.decode('FormDefinition', take(buffer_call(:lockbox_define_form, handle, Fiddle::Pointer[alias_name], alias_name.bytesize, Fiddle::Pointer[name], name.bytesize, Fiddle::Pointer[description], description.bytesize, Fiddle::Pointer[fields_flatbuffer], fields_flatbuffer.bytesize)))
    end

    def lockbox_list_form_definitions(handle)
      Revault::Internal::DomainCodec.decode('FormDefinitionList', take(buffer_call(:lockbox_list_form_definitions, handle)))
    end

    def lockbox_resolve_form(handle, reference)
      Revault::Internal::DomainCodec.decode('FormDefinition', take(buffer_call(:lockbox_resolve_form, handle, Fiddle::Pointer[reference], reference.bytesize)))
    end

    def lockbox_list_form_revisions(handle, type_id)
      Revault::Internal::DomainCodec.decode('FormDefinitionList', take(buffer_call(:lockbox_list_form_revisions, handle, Fiddle::Pointer[type_id], type_id.bytesize)))
    end

    def lockbox_create_form_record(handle, path, type_reference, name)
      Revault::Internal::DomainCodec.decode('FormRecord', take(buffer_call(:lockbox_create_form_record, handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[type_reference], type_reference.bytesize, Fiddle::Pointer[name], name.bytesize)))
    end

    def lockbox_set_form_field(handle, path, field, value)
      require_value(Native.lockbox_set_form_field(handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[field], field.bytesize, Fiddle::Pointer[value], value.bytesize))
    end

    def lockbox_set_secret_form_field(handle, path, field, value)
      secret = value.dup
      require_value(Native.lockbox_set_secret_form_field(handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[field], field.bytesize, Fiddle::Pointer[secret], secret.bytesize))
    ensure
      secret&.replace("\0" * secret.bytesize)
    end

    def lockbox_list_form_records(handle)
      Revault::Internal::DomainCodec.decode('FormRecordList', take(buffer_call(:lockbox_list_form_records, handle)))
    end

    def lockbox_get_form_record(handle, path)
      Revault::Internal::DomainCodec.decode('OptionalFormRecord', take(buffer_call(:lockbox_get_form_record, handle, Fiddle::Pointer[path], path.bytesize)))
    end

    def lockbox_delete_form_record(handle, path)
      require_value(Native.lockbox_delete_form_record(handle, Fiddle::Pointer[path], path.bytesize))
    end

    def lockbox_move_form_records(handle, moves_flatbuffer)
      require_value(Native.lockbox_move_form_records(handle, Fiddle::Pointer[moves_flatbuffer], moves_flatbuffer.bytesize))
    end

    def lockbox_get_form_field(handle, path, field)
      Revault::Internal::DomainCodec.decode('OptionalFormValue', take(buffer_call(:lockbox_get_form_field, handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[field], field.bytesize)))
    end

    def lockbox_get_secret_form_field(handle, path, field, &callback)
      with_secret(->(output) { Native.lockbox_get_secret_form_field(handle, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[field], field.bytesize, output) }, &callback)
    end

    def lockbox_to_bytes(handle)
      take(buffer_call(:lockbox_to_bytes, handle))
    end

    def lockbox_free(handle)
      Native.lockbox_free(handle)
    end

    def vault_is_running()
      Native.vault_is_running()
    end

    def vault_forget_all()
      require_value(Native.vault_forget_all())
    end

    def key_contact_generate()
      require_handle(Native.key_contact_generate())
    end

    def key_contact_from_private(bytes)
      require_handle(Native.key_contact_from_private(Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def key_contact_public(handle)
      take(buffer_call(:key_contact_public, handle))
    end

    def key_contact_private(handle)
      take(buffer_call(:key_contact_private, handle))
    end

    def key_contact_public_from_bytes(bytes)
      require_handle(Native.key_contact_public_from_bytes(Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def key_contact_public_free(handle)
      Native.key_contact_public_free(handle)
    end

    def key_contact_free(handle)
      Native.key_contact_free(handle)
    end

    def key_contact_encrypt(contact, content_key)
      require_handle(Native.key_contact_encrypt(contact, Fiddle::Pointer[content_key], content_key.bytesize))
    end

    def key_contact_decrypt(contact, wrapped)
      take(buffer_call(:key_contact_decrypt, contact, wrapped))
    end

    def key_contact_wrapped_public(wrapped)
      take(buffer_call(:key_contact_wrapped_public, wrapped))
    end

    def key_contact_wrapped_ciphertext(wrapped)
      take(buffer_call(:key_contact_wrapped_ciphertext, wrapped))
    end

    def key_contact_wrapped_encrypted(wrapped)
      take(buffer_call(:key_contact_wrapped_encrypted, wrapped))
    end

    def key_contact_wrapped_free(handle)
      Native.key_contact_wrapped_free(handle)
    end

    def key_signing_generate()
      require_handle(Native.key_signing_generate())
    end

    def key_signing_from_private(bytes)
      require_handle(Native.key_signing_from_private(Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def key_signing_public(handle)
      take(buffer_call(:key_signing_public, handle))
    end

    def key_signing_private(handle)
      take(buffer_call(:key_signing_private, handle))
    end

    def key_signing_public_from_bytes(bytes)
      require_handle(Native.key_signing_public_from_bytes(Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def key_signing_public_free(handle)
      Native.key_signing_public_free(handle)
    end

    def key_signing_free(handle)
      Native.key_signing_free(handle)
    end

    def vault_key_export_private(key, format)
      take(buffer_call(:vault_key_export_private, key, Fiddle::Pointer[format], format.bytesize))
    end

    def vault_key_export_public(key, format)
      take(buffer_call(:vault_key_export_public, key, Fiddle::Pointer[format], format.bytesize))
    end

    def vault_key_import_private(bytes)
      require_handle(Native.vault_key_import_private(Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def vault_key_import_public(bytes)
      require_handle(Native.vault_key_import_public(Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def vault_key_fingerprint(key)
      take(buffer_call(:vault_key_fingerprint, key))
    end

    def vault_key_format_hex(bytes)
      take(buffer_call(:vault_key_format_hex, Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def vault_key_decode_hex(text)
      take(buffer_call(:vault_key_decode_hex, Fiddle::Pointer[text], text.bytesize))
    end

    def vault_key_format_crockford(bytes)
      take(buffer_call(:vault_key_format_crockford, Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def vault_key_format_crockford_reading(code)
      take(buffer_call(:vault_key_format_crockford_reading, Fiddle::Pointer[code], code.bytesize))
    end

    def vault_key_decode_crockford(code)
      take(buffer_call(:vault_key_decode_crockford, Fiddle::Pointer[code], code.bytesize))
    end

    def vault_key_hex_encode(bytes)
      take(buffer_call(:vault_key_hex_encode, Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def vault_key_hex_decode(text)
      take(buffer_call(:vault_key_hex_decode, Fiddle::Pointer[text], text.bytesize))
    end

    def vault_directory_open(root, password)
      require_handle(Native.vault_directory_open(Fiddle::Pointer[root], root.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_structure_version_current()
      Native.vault_structure_version_current()
    end

    def vault_directory_probe_structure_version(root, password)
      Native.vault_directory_probe_structure_version(Fiddle::Pointer[root], root.bytesize, Fiddle::Pointer[password], password.bytesize)
    end

    def vault_directory_open_or_create_default(password)
      require_handle(Native.vault_directory_open_or_create_default(Fiddle::Pointer[password], password.bytesize))
    end

    def vault_directory_replace_default(password)
      require_handle(Native.vault_directory_replace_default(Fiddle::Pointer[password], password.bytesize))
    end

    def vault_directory_change_password(root, old_password, new_password)
      require_value(Native.vault_directory_change_password(Fiddle::Pointer[root], root.bytesize, Fiddle::Pointer[old_password], old_password.bytesize, Fiddle::Pointer[new_password], new_password.bytesize))
    end

    def vault_directory_change_default_password(old_password, new_password)
      require_value(Native.vault_directory_change_default_password(Fiddle::Pointer[old_password], old_password.bytesize, Fiddle::Pointer[new_password], new_password.bytesize))
    end

    def vault_directory_replace(root, password)
      require_handle(Native.vault_directory_replace(Fiddle::Pointer[root], root.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_directory_open_or_create(root, password)
      require_handle(Native.vault_directory_open_or_create(Fiddle::Pointer[root], root.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_directory_root(handle)
      take(buffer_call(:vault_directory_root, handle))
    end

    def vault_directory_structure_version(handle)
      Native.vault_directory_structure_version(handle)
    end

    def vault_directory_list_private_keys(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_directory_list_private_keys, handle)))
    end

    def vault_directory_list_private_key_names(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_directory_list_private_key_names, handle)))
    end

    def vault_directory_list_contact_names(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_directory_list_contact_names, handle)))
    end

    def vault_directory_list_form_aliases(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_directory_list_form_aliases, handle)))
    end

    def vault_directory_private_key_exists(handle, name)
      Native.vault_directory_private_key_exists(handle, Fiddle::Pointer[name], name.bytesize)
    end

    def vault_directory_delete_private_key(handle, name)
      require_value(Native.vault_directory_delete_private_key(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_store_private_key(handle, name, key)
      require_value(Native.vault_directory_store_private_key(handle, Fiddle::Pointer[name], name.bytesize, key))
    end

    def vault_directory_load_private_key(handle, name)
      require_handle(Native.vault_directory_load_private_key(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_load_private_key_generation(handle, name, index)
      require_handle(Native.vault_directory_load_private_key_generation(handle, Fiddle::Pointer[name], name.bytesize, index))
    end

    def vault_directory_store_contact(handle, name, key)
      require_value(Native.vault_directory_store_contact(handle, Fiddle::Pointer[name], name.bytesize, key))
    end

    def vault_directory_load_contact(handle, name)
      require_handle(Native.vault_directory_load_contact(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_contact_exists(handle, name)
      Native.vault_directory_contact_exists(handle, Fiddle::Pointer[name], name.bytesize)
    end

    def vault_directory_delete_contact(handle, name)
      require_value(Native.vault_directory_delete_contact(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_list_contacts(handle)
      Revault::Internal::DomainCodec.decode('ContactList', take(buffer_call(:vault_directory_list_contacts, handle)))
    end

    def vault_directory_store_profile_email(handle, name, email)
      require_value(Native.vault_directory_store_profile_email(handle, Fiddle::Pointer[name], name.bytesize, Fiddle::Pointer[email], email.bytesize))
    end

    def vault_directory_profile_email(handle, name)
      Revault::Internal::DomainCodec.decode('OptionalString', take(buffer_call(:vault_directory_profile_email, handle, Fiddle::Pointer[name], name.bytesize)))
    end

    def vault_directory_store_backup(handle, id, bytes)
      require_value(Native.vault_directory_store_backup(handle, Fiddle::Pointer[id], id.bytesize, Fiddle::Pointer[bytes], bytes.bytesize))
    end

    def vault_directory_load_backup(handle, id)
      take(buffer_call(:vault_directory_load_backup, handle, Fiddle::Pointer[id], id.bytesize))
    end

    def vault_directory_backup_count(handle)
      Native.vault_directory_backup_count(handle)
    end

    def vault_directory_restore_private_key(handle, name, key, signing_key, overwrite)
      require_value(Native.vault_directory_restore_private_key(handle, Fiddle::Pointer[name], name.bytesize, key, signing_key, overwrite))
    end

    def vault_directory_load_owner_signing_key(handle, name)
      require_handle(Native.vault_directory_load_owner_signing_key(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_load_owner_signing_key_generation(handle, name, index)
      require_handle(Native.vault_directory_load_owner_signing_key_generation(handle, Fiddle::Pointer[name], name.bytesize, index))
    end

    def vault_directory_store_contact_signing_key(handle, name, key)
      require_value(Native.vault_directory_store_contact_signing_key(handle, Fiddle::Pointer[name], name.bytesize, key))
    end

    def vault_directory_load_contact_signing_key(handle, name)
      require_handle(Native.vault_directory_load_contact_signing_key(handle, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_list_profile_generations(handle, name)
      Revault::Internal::DomainCodec.decode('ProfileHistory', take(buffer_call(:vault_directory_list_profile_generations, handle, Fiddle::Pointer[name], name.bytesize)))
    end

    def vault_directory_rotate_private_key(handle, name)
      Revault::Internal::DomainCodec.decode('ProfileHistory', take(buffer_call(:vault_directory_rotate_private_key, handle, Fiddle::Pointer[name], name.bytesize)))
    end

    def vault_directory_remember_lockbox(handle, id, path)
      require_value(Native.vault_directory_remember_lockbox(handle, Fiddle::Pointer[id], id.bytesize, Fiddle::Pointer[path], path.bytesize))
    end

    def vault_directory_list_known_lockboxes(handle)
      Revault::Internal::DomainCodec.decode('KnownLockboxList', take(buffer_call(:vault_directory_list_known_lockboxes, handle)))
    end

    def vault_directory_forget_lockbox(handle, path)
      require_value(Native.vault_directory_forget_lockbox(handle, Fiddle::Pointer[path], path.bytesize))
    end

    def vault_directory_remember_access_slot_label(handle, id, slot_id, name)
      require_value(Native.vault_directory_remember_access_slot_label(handle, Fiddle::Pointer[id], id.bytesize, slot_id, Fiddle::Pointer[name], name.bytesize))
    end

    def vault_directory_list_access_slot_labels(handle, id)
      Revault::Internal::DomainCodec.decode('AccessSlotLabelList', take(buffer_call(:vault_directory_list_access_slot_labels, handle, Fiddle::Pointer[id], id.bytesize)))
    end

    def vault_directory_find_access_slot_labels(handle, id, name)
      Revault::Internal::DomainCodec.decode('AccessSlotLabelList', take(buffer_call(:vault_directory_find_access_slot_labels, handle, Fiddle::Pointer[id], id.bytesize, Fiddle::Pointer[name], name.bytesize)))
    end

    def vault_directory_forget_access_slot_label(handle, id, slot_id)
      require_value(Native.vault_directory_forget_access_slot_label(handle, Fiddle::Pointer[id], id.bytesize, slot_id))
    end

    def vault_directory_define_form(handle, alias_name, name, description, fields_flatbuffer)
      Revault::Internal::DomainCodec.decode('FormDefinition', take(buffer_call(:vault_directory_define_form, handle, Fiddle::Pointer[alias_name], alias_name.bytesize, Fiddle::Pointer[name], name.bytesize, Fiddle::Pointer[description], description.bytesize, Fiddle::Pointer[fields_flatbuffer], fields_flatbuffer.bytesize)))
    end

    def vault_directory_resolve_form(handle, reference)
      Revault::Internal::DomainCodec.decode('FormDefinition', take(buffer_call(:vault_directory_resolve_form, handle, Fiddle::Pointer[reference], reference.bytesize)))
    end

    def vault_directory_list_forms(handle)
      Revault::Internal::DomainCodec.decode('FormDefinitionList', take(buffer_call(:vault_directory_list_forms, handle)))
    end

    def vault_directory_list_form_revisions(handle, type_id)
      Revault::Internal::DomainCodec.decode('FormDefinitionList', take(buffer_call(:vault_directory_list_form_revisions, handle, Fiddle::Pointer[type_id], type_id.bytesize)))
    end

    def vault_directory_seed_forms(handle)
      Native.vault_directory_seed_forms(handle)
    end

    def vault_directory_remember_password(handle, id, password)
      require_value(Native.vault_directory_remember_password(handle, Fiddle::Pointer[id], id.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_directory_remembered_password(handle, id)
      take(buffer_call(:vault_directory_remembered_password, handle, Fiddle::Pointer[id], id.bytesize))
    end

    def vault_backup_default(path, overwrite)
      Revault::Internal::DomainCodec.decode('VaultBackupManifest', take(buffer_call(:vault_backup_default, Fiddle::Pointer[path], path.bytesize, overwrite)))
    end

    def vault_restore_default(path, overwrite)
      Revault::Internal::DomainCodec.decode('VaultBackupManifest', take(buffer_call(:vault_restore_default, Fiddle::Pointer[path], path.bytesize, overwrite)))
    end

    def vault_directory_free(handle)
      Native.vault_directory_free(handle)
    end

    def vault_read_only_open(root, password)
      require_handle(Native.vault_read_only_open(Fiddle::Pointer[root], root.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_read_only_open_default(password)
      require_handle(Native.vault_read_only_open_default(Fiddle::Pointer[password], password.bytesize))
    end

    def vault_read_only_list_profile_names(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_read_only_list_profile_names, handle)))
    end

    def vault_read_only_list_contact_names(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_read_only_list_contact_names, handle)))
    end

    def vault_read_only_list_form_aliases(handle)
      Revault::Internal::DomainCodec.decode('StringList', take(buffer_call(:vault_read_only_list_form_aliases, handle)))
    end

    def vault_read_only_list_known_lockboxes(handle)
      Revault::Internal::DomainCodec.decode('KnownLockboxList', take(buffer_call(:vault_read_only_list_known_lockboxes, handle)))
    end

    def vault_read_only_free(handle)
      Native.vault_read_only_free(handle)
    end

    def vault_agent_serve()
      require_value(Native.vault_agent_serve())
    end

    def vault_agent_verify_transport()
      require_value(Native.vault_agent_verify_transport())
    end

    def vault_agent_get(id)
      take(buffer_call(:vault_agent_get, Fiddle::Pointer[id], id.bytesize))
    end

    def vault_agent_put(id, key)
      require_value(Native.vault_agent_put(Fiddle::Pointer[id], id.bytesize, Fiddle::Pointer[key], key.bytesize))
    end

    def vault_agent_forget(id)
      require_value(Native.vault_agent_forget(Fiddle::Pointer[id], id.bytesize))
    end

    def vault_agent_stop()
      require_value(Native.vault_agent_stop())
    end

    def vault_agent_start()
      require_value(Native.vault_agent_start())
    end

    def vault_agent_list()
      Revault::Internal::DomainCodec.decode('AgentEntryList', take(buffer_call(:vault_agent_list)))
    end

    def vault_agent_sleep_support()
      Revault::Internal::DomainCodec.decode('SleepSupport', take(buffer_call(:vault_agent_sleep_support)))
    end

    def vault_platform_status()
      Revault::Internal::DomainCodec.decode('PlatformStatus', take(buffer_call(:vault_platform_status)))
    end

    def vault_platform_set_scope(scope)
      require_value(Native.vault_platform_set_scope(Fiddle::Pointer[scope], scope.bytesize))
    end

    def vault_platform_forget_password()
      require_value(Native.vault_platform_forget_password())
    end

    def vault_platform_put_password(password)
      require_value(Native.vault_platform_put_password(Fiddle::Pointer[password], password.bytesize))
    end

    def vault_platform_enable()
      require_value(Native.vault_platform_enable())
    end

    def vault_platform_disable()
      require_value(Native.vault_platform_disable())
    end

    def vault_platform_disabled()
      Native.vault_platform_disabled()
    end

    def vault_platform_get_password()
      take(buffer_call(:vault_platform_get_password))
    end

    def vault_default_directory()
      take(buffer_call(:vault_default_directory))
    end

    def vault_default_path()
      take(buffer_call(:vault_default_path))
    end

    def vault_agent_log_path()
      take(buffer_call(:vault_agent_log_path))
    end

    def vault_agent_log_destination()
      take(buffer_call(:vault_agent_log_destination))
    end

    def vault_agent_get_vault_unlock_key(vault_id)
      take(buffer_call(:vault_agent_get_vault_unlock_key, Fiddle::Pointer[vault_id], vault_id.bytesize))
    end

    def vault_agent_put_vault_unlock_key(vault_id, key, ttl_seconds)
      require_value(Native.vault_agent_put_vault_unlock_key(Fiddle::Pointer[vault_id], vault_id.bytesize, Fiddle::Pointer[key], key.bytesize, ttl_seconds))
    end

    def vault_agent_forget_vault_unlock_key(vault_id)
      require_value(Native.vault_agent_forget_vault_unlock_key(Fiddle::Pointer[vault_id], vault_id.bytesize))
    end

    def vault_agent_get_owner_signing_key(vault_id, profile)
      require_handle(Native.vault_agent_get_owner_signing_key(Fiddle::Pointer[vault_id], vault_id.bytesize, Fiddle::Pointer[profile], profile.bytesize))
    end

    def vault_agent_put_owner_signing_key(vault_id, profile, key, ttl_seconds)
      require_value(Native.vault_agent_put_owner_signing_key(Fiddle::Pointer[vault_id], vault_id.bytesize, Fiddle::Pointer[profile], profile.bytesize, key, ttl_seconds))
    end

    def vault_agent_forget_owner_signing_key(vault_id, profile)
      require_value(Native.vault_agent_forget_owner_signing_key(Fiddle::Pointer[vault_id], vault_id.bytesize, Fiddle::Pointer[profile], profile.bytesize))
    end

    def vault_agent_begin_activity(kind)
      require_handle(Native.vault_agent_begin_activity(Fiddle::Pointer[kind], kind.bytesize))
    end

    def vault_agent_end_activity(handle)
      Native.vault_agent_end_activity(handle)
    end

    def vault_local()
      require_handle(Native.vault_local())
    end

    def vault_create_lockbox_password(vault, path, password)
      require_handle(Native.vault_create_lockbox_password(vault, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_open_lockbox_password(vault, path, password)
      require_handle(Native.vault_open_lockbox_password(vault, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[password], password.bytesize))
    end

    def vault_create_lockbox_content_key(vault, path, content_key, signing_key)
      require_handle(Native.vault_create_lockbox_content_key(vault, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[content_key], content_key.bytesize, signing_key))
    end

    def vault_create_lockbox_contact(vault, path, contact, name, signing_key)
      require_handle(Native.vault_create_lockbox_contact(vault, Fiddle::Pointer[path], path.bytesize, contact, Fiddle::Pointer[name], name.bytesize, signing_key))
    end

    def vault_open_lockbox_content_key(vault, path, content_key, signing_key)
      require_handle(Native.vault_open_lockbox_content_key(vault, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[content_key], content_key.bytesize, signing_key))
    end

    def vault_cache_lockbox_password(vault, path, password, ttl_seconds)
      require_value(Native.vault_cache_lockbox_password(vault, Fiddle::Pointer[path], path.bytesize, Fiddle::Pointer[password], password.bytesize, ttl_seconds))
    end

    def vault_close_lockbox(vault, path)
      require_value(Native.vault_close_lockbox(vault, Fiddle::Pointer[path], path.bytesize))
    end

    def vault_close_all(vault)
      require_value(Native.vault_close_all(vault))
    end

    def vault_free(vault)
      Native.vault_free(vault)
    end

    def free_buffer(value) = Shim.ruby_buffer_free(value)
  end
end
