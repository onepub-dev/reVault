// Generated complete binary operation layer. Do not edit.
import koffi from 'koffi';
import { Builder, ByteBuffer } from 'flatbuffers';
import * as transport from './generated/flatbuffers.js';
import { nativeLibraryPath } from './native-loader.js';

const library = koffi.load(nativeLibraryPath());
const api_abi_version = library.func('uint32_t api_abi_version(void)');
if (api_abi_version() !== 3) throw new Error('revault-api native ABI mismatch; expected 3');
const RevaultBuffer = koffi.struct('RevaultBuffer', { ptr: 'uint8_t *', len: 'size_t' });

export function createMessage(name, fields = {}) {
  if (transport[name] == null) throw new TypeError(`unknown reVault domain type: ${name}`);
  return Object.freeze({ type: name, ...fields });
}
export function encodeMessage(message) {
  const builder = new Builder(256);
  const table = transport[message?.type];
  if (message?.type === 'PathMoveList') {
    const offsets = (message.values ?? []).map((value) => {
      const source = builder.createString(value.source ?? '');
      const destination = builder.createString(value.destination ?? '');
      return transport.PathMove.createPathMove(builder, source, destination);
    });
    const values = transport.PathMoveList.createValuesVector(builder, offsets);
    builder.finish(transport.PathMoveList.createPathMoveList(builder, values));
  } else if (message?.type === 'FormFieldList') {
    const offsets = (message.values ?? []).map((value) => {
      const id = builder.createString(value.id ?? '');
      const label = builder.createString(value.label ?? '');
      const kind = builder.createString(value.kind ?? '');
      return transport.FormField.createFormField(builder, id, label, kind, value.required === true);
    });
    const values = transport.FormFieldList.createValuesVector(builder, offsets);
    builder.finish(transport.FormFieldList.createFormFieldList(builder, values));
  } else {
    throw new TypeError(`encoding ${message?.type ?? typeof message} is not a supported API input`);
  }
  return Buffer.from(builder.asUint8Array());
}

const buffer_last_error = library.func('const char * buffer_last_error(void)');
const buffer_last_error_details = library.func('RevaultBuffer buffer_last_error_details(void)');
const buffer_free = library.func('void buffer_free(RevaultBuffer)');
const secret_len = library.func('bool secret_len(void *, _Out_ size_t *)');
const secret_copy = library.func('bool secret_copy(void *, void *, size_t)');
const secret_free = library.func('void secret_free(void *)');
const lockbox_format_version = library.func('uint16_t lockbox_format_version(void)');
const lockbox_probe_format_version = library.func('uint16_t lockbox_probe_format_version(void *, size_t)');
const lockbox_create = library.func('void * lockbox_create(void *, size_t)');
const lockbox_create_with_options = library.func('void * lockbox_create_with_options(void *, size_t, const char *, size_t, uint64_t, const char *, size_t, const char *, size_t, size_t)');
const lockbox_create_password = library.func('void * lockbox_create_password(void *, size_t)');
const lockbox_create_contact = library.func('void * lockbox_create_contact(void *)');
const lockbox_create_with_signing_key = library.func('void * lockbox_create_with_signing_key(void *, size_t, void *)');
const lockbox_open = library.func('void * lockbox_open(void *, size_t, void *, size_t)');
const lockbox_open_with_options = library.func('void * lockbox_open_with_options(void *, size_t, void *, size_t, const char *, size_t, uint64_t, const char *, size_t, const char *, size_t, size_t)');
const lockbox_open_password = library.func('void * lockbox_open_password(void *, size_t, void *, size_t)');
const lockbox_open_contact = library.func('void * lockbox_open_contact(void *, size_t, void *)');
const lockbox_add_file = library.func('bool lockbox_add_file(void *, const char *, size_t, void *, size_t, bool)');
const lockbox_add_file_with_permissions = library.func('bool lockbox_add_file_with_permissions(void *, const char *, size_t, void *, size_t, uint32_t, bool)');
const lockbox_get_file = library.func('RevaultBuffer lockbox_get_file(void *, const char *, size_t)');
const lockbox_extract_file = library.func('bool lockbox_extract_file(void *, const char *, size_t, const char *, size_t, bool)');
const lockbox_extract_directory = library.func('bool lockbox_extract_directory(void *, const char *, size_t, uint64_t, uint64_t, size_t, bool, bool, bool)');
const lockbox_stream_content = library.func('RevaultBuffer lockbox_stream_content(void *, bool)');
const lockbox_cache_stats = library.func('RevaultBuffer lockbox_cache_stats(void *)');
const lockbox_import_stats = library.func('RevaultBuffer lockbox_import_stats(void *)');
const lockbox_reset_import_stats = library.func('bool lockbox_reset_import_stats(void *)');
const lockbox_inspect_file = library.func('RevaultBuffer lockbox_inspect_file(const char *, size_t)');
const lockbox_page_inspection = library.func('RevaultBuffer lockbox_page_inspection(void *)');
const lockbox_recovery_report = library.func('RevaultBuffer lockbox_recovery_report(void *)');
const lockbox_recovery_report_render = library.func('RevaultBuffer lockbox_recovery_report_render(void *, bool, size_t)');
const lockbox_recovery_scan_path = library.func('RevaultBuffer lockbox_recovery_scan_path(const char *, size_t, void *, size_t)');
const lockbox_storage_len = library.func('uint64_t lockbox_storage_len(void *)');
const lockbox_set_workload_profile = library.func('bool lockbox_set_workload_profile(void *, const char *, size_t)');
const lockbox_set_worker_policy = library.func('bool lockbox_set_worker_policy(void *, const char *, size_t, size_t)');
const lockbox_runtime_options = library.func('RevaultBuffer lockbox_runtime_options(void *)');
const lockbox_commit = library.func('bool lockbox_commit(void *)');
const lockbox_create_dir = library.func('bool lockbox_create_dir(void *, const char *, size_t, bool)');
const lockbox_delete = library.func('bool lockbox_delete(void *, const char *, size_t)');
const lockbox_remove_dir = library.func('bool lockbox_remove_dir(void *, const char *, size_t, bool)');
const lockbox_create_parent_dirs = library.func('bool lockbox_create_parent_dirs(void *, const char *, size_t)');
const lockbox_rename = library.func('bool lockbox_rename(void *, const char *, size_t, const char *, size_t)');
const lockbox_list = library.func('RevaultBuffer lockbox_list(void *, const char *, size_t, bool)');
const lockbox_list_with_options = library.func('RevaultBuffer lockbox_list_with_options(void *, const char *, size_t, const char *, size_t, bool, bool, bool, bool, size_t)');
const lockbox_stat = library.func('RevaultBuffer lockbox_stat(void *, const char *, size_t)');
const lockbox_set_variable = library.func('bool lockbox_set_variable(void *, const char *, size_t, const char *, size_t)');
const lockbox_set_secret_variable = library.func('bool lockbox_set_secret_variable(void *, const char *, size_t, void *, size_t)');
const lockbox_get_variable = library.func('RevaultBuffer lockbox_get_variable(void *, const char *, size_t)');
const lockbox_get_secret_variable = library.func('bool lockbox_get_secret_variable(void *, const char *, size_t, _Out_ void **)');
const lockbox_delete_variable = library.func('bool lockbox_delete_variable(void *, const char *, size_t)');
const lockbox_move_variables = library.func('bool lockbox_move_variables(void *, void *, size_t)');
const lockbox_list_variables = library.func('RevaultBuffer lockbox_list_variables(void *)');
const lockbox_variable_sensitivity = library.func('RevaultBuffer lockbox_variable_sensitivity(void *, const char *, size_t)');
const lockbox_add_symlink = library.func('bool lockbox_add_symlink(void *, const char *, size_t, const char *, size_t, bool)');
const lockbox_get_symlink_target = library.func('RevaultBuffer lockbox_get_symlink_target(void *, const char *, size_t)');
const lockbox_id = library.func('RevaultBuffer lockbox_id(void *)');
const lockbox_exists = library.func('bool lockbox_exists(void *, const char *, size_t)');
const lockbox_is_dir = library.func('bool lockbox_is_dir(void *, const char *, size_t)');
const lockbox_permissions = library.func('uint32_t lockbox_permissions(void *, const char *, size_t)');
const lockbox_set_permissions = library.func('bool lockbox_set_permissions(void *, const char *, size_t, uint32_t)');
const lockbox_read_range = library.func('RevaultBuffer lockbox_read_range(void *, const char *, size_t, uint64_t, uint64_t)');
const lockbox_recovery_scan = library.func('RevaultBuffer lockbox_recovery_scan(void *, size_t, void *, size_t)');
const lockbox_recovery_salvage = library.func('void * lockbox_recovery_salvage(void *, size_t, void *, size_t, void *)');
const lockbox_add_password = library.func('uint64_t lockbox_add_password(void *, void *, size_t)');
const lockbox_add_contact = library.func('uint64_t lockbox_add_contact(void *, void *, const char *, size_t)');
const lockbox_delete_key = library.func('bool lockbox_delete_key(void *, uint64_t)');
const lockbox_list_key_slots = library.func('RevaultBuffer lockbox_list_key_slots(void *)');
const lockbox_set_owner_signing_key = library.func('bool lockbox_set_owner_signing_key(void *, void *)');
const lockbox_owner_inspection = library.func('RevaultBuffer lockbox_owner_inspection(void *)');
const lockbox_define_form = library.func('RevaultBuffer lockbox_define_form(void *, const char *, size_t, const char *, size_t, const char *, size_t, void *, size_t)');
const lockbox_list_form_definitions = library.func('RevaultBuffer lockbox_list_form_definitions(void *)');
const lockbox_resolve_form = library.func('RevaultBuffer lockbox_resolve_form(void *, const char *, size_t)');
const lockbox_list_form_revisions = library.func('RevaultBuffer lockbox_list_form_revisions(void *, const char *, size_t)');
const lockbox_create_form_record = library.func('RevaultBuffer lockbox_create_form_record(void *, const char *, size_t, const char *, size_t, const char *, size_t)');
const lockbox_set_form_field = library.func('bool lockbox_set_form_field(void *, const char *, size_t, const char *, size_t, const char *, size_t)');
const lockbox_set_secret_form_field = library.func('bool lockbox_set_secret_form_field(void *, const char *, size_t, const char *, size_t, void *, size_t)');
const lockbox_list_form_records = library.func('RevaultBuffer lockbox_list_form_records(void *)');
const lockbox_get_form_record = library.func('RevaultBuffer lockbox_get_form_record(void *, const char *, size_t)');
const lockbox_delete_form_record = library.func('bool lockbox_delete_form_record(void *, const char *, size_t)');
const lockbox_move_form_records = library.func('bool lockbox_move_form_records(void *, void *, size_t)');
const lockbox_get_form_field = library.func('RevaultBuffer lockbox_get_form_field(void *, const char *, size_t, const char *, size_t)');
const lockbox_get_secret_form_field = library.func('bool lockbox_get_secret_form_field(void *, const char *, size_t, const char *, size_t, _Out_ void **)');
const lockbox_to_bytes = library.func('RevaultBuffer lockbox_to_bytes(void *)');
const lockbox_free = library.func('void lockbox_free(void *)');
const vault_is_running = library.func('bool vault_is_running(void)');
const vault_forget_all = library.func('bool vault_forget_all(void)');
const key_contact_generate = library.func('void * key_contact_generate(void)');
const key_contact_from_private = library.func('void * key_contact_from_private(void *, size_t)');
const key_contact_public = library.func('RevaultBuffer key_contact_public(void *)');
const key_contact_private = library.func('RevaultBuffer key_contact_private(void *)');
const key_contact_public_from_bytes = library.func('void * key_contact_public_from_bytes(void *, size_t)');
const key_contact_public_free = library.func('void key_contact_public_free(void *)');
const key_contact_free = library.func('void key_contact_free(void *)');
const key_contact_encrypt = library.func('void * key_contact_encrypt(void *, void *, size_t)');
const key_contact_decrypt = library.func('RevaultBuffer key_contact_decrypt(void *, void *)');
const key_contact_wrapped_public = library.func('RevaultBuffer key_contact_wrapped_public(void *)');
const key_contact_wrapped_ciphertext = library.func('RevaultBuffer key_contact_wrapped_ciphertext(void *)');
const key_contact_wrapped_encrypted = library.func('RevaultBuffer key_contact_wrapped_encrypted(void *)');
const key_contact_wrapped_free = library.func('void key_contact_wrapped_free(void *)');
const key_signing_generate = library.func('void * key_signing_generate(void)');
const key_signing_from_private = library.func('void * key_signing_from_private(void *, size_t)');
const key_signing_public = library.func('RevaultBuffer key_signing_public(void *)');
const key_signing_private = library.func('RevaultBuffer key_signing_private(void *)');
const key_signing_public_from_bytes = library.func('void * key_signing_public_from_bytes(void *, size_t)');
const key_signing_public_free = library.func('void key_signing_public_free(void *)');
const key_signing_free = library.func('void key_signing_free(void *)');
const vault_key_export_private = library.func('RevaultBuffer vault_key_export_private(void *, const char *, size_t)');
const vault_key_export_public = library.func('RevaultBuffer vault_key_export_public(void *, const char *, size_t)');
const vault_key_import_private = library.func('void * vault_key_import_private(void *, size_t)');
const vault_key_import_public = library.func('void * vault_key_import_public(void *, size_t)');
const vault_key_fingerprint = library.func('RevaultBuffer vault_key_fingerprint(void *)');
const vault_key_format_hex = library.func('RevaultBuffer vault_key_format_hex(void *, size_t)');
const vault_key_decode_hex = library.func('RevaultBuffer vault_key_decode_hex(const char *, size_t)');
const vault_key_format_crockford = library.func('RevaultBuffer vault_key_format_crockford(void *, size_t)');
const vault_key_format_crockford_reading = library.func('RevaultBuffer vault_key_format_crockford_reading(const char *, size_t)');
const vault_key_decode_crockford = library.func('RevaultBuffer vault_key_decode_crockford(const char *, size_t)');
const vault_key_hex_encode = library.func('RevaultBuffer vault_key_hex_encode(void *, size_t)');
const vault_key_hex_decode = library.func('RevaultBuffer vault_key_hex_decode(const char *, size_t)');
const vault_directory_open = library.func('void * vault_directory_open(const char *, size_t, void *, size_t)');
const vault_structure_version_current = library.func('uint32_t vault_structure_version_current(void)');
const vault_directory_probe_structure_version = library.func('uint32_t vault_directory_probe_structure_version(const char *, size_t, void *, size_t)');
const vault_directory_open_or_create_default = library.func('void * vault_directory_open_or_create_default(void *, size_t)');
const vault_directory_replace_default = library.func('void * vault_directory_replace_default(void *, size_t)');
const vault_directory_change_password = library.func('bool vault_directory_change_password(const char *, size_t, void *, size_t, void *, size_t)');
const vault_directory_change_default_password = library.func('bool vault_directory_change_default_password(void *, size_t, void *, size_t)');
const vault_directory_replace = library.func('void * vault_directory_replace(const char *, size_t, void *, size_t)');
const vault_directory_open_or_create = library.func('void * vault_directory_open_or_create(const char *, size_t, void *, size_t)');
const vault_directory_root = library.func('RevaultBuffer vault_directory_root(void *)');
const vault_directory_structure_version = library.func('uint32_t vault_directory_structure_version(void *)');
const vault_directory_list_private_keys = library.func('RevaultBuffer vault_directory_list_private_keys(void *)');
const vault_directory_list_private_key_names = library.func('RevaultBuffer vault_directory_list_private_key_names(void *)');
const vault_directory_list_contact_names = library.func('RevaultBuffer vault_directory_list_contact_names(void *)');
const vault_directory_list_form_aliases = library.func('RevaultBuffer vault_directory_list_form_aliases(void *)');
const vault_directory_private_key_exists = library.func('bool vault_directory_private_key_exists(void *, const char *, size_t)');
const vault_directory_delete_private_key = library.func('bool vault_directory_delete_private_key(void *, const char *, size_t)');
const vault_directory_store_private_key = library.func('bool vault_directory_store_private_key(void *, const char *, size_t, void *)');
const vault_directory_load_private_key = library.func('void * vault_directory_load_private_key(void *, const char *, size_t)');
const vault_directory_load_private_key_generation = library.func('void * vault_directory_load_private_key_generation(void *, const char *, size_t, uint16_t)');
const vault_directory_store_contact = library.func('bool vault_directory_store_contact(void *, const char *, size_t, void *)');
const vault_directory_load_contact = library.func('void * vault_directory_load_contact(void *, const char *, size_t)');
const vault_directory_contact_exists = library.func('bool vault_directory_contact_exists(void *, const char *, size_t)');
const vault_directory_delete_contact = library.func('bool vault_directory_delete_contact(void *, const char *, size_t)');
const vault_directory_list_contacts = library.func('RevaultBuffer vault_directory_list_contacts(void *)');
const vault_directory_store_profile_email = library.func('bool vault_directory_store_profile_email(void *, const char *, size_t, const char *, size_t)');
const vault_directory_profile_email = library.func('RevaultBuffer vault_directory_profile_email(void *, const char *, size_t)');
const vault_directory_store_backup = library.func('bool vault_directory_store_backup(void *, void *, size_t, void *, size_t)');
const vault_directory_load_backup = library.func('RevaultBuffer vault_directory_load_backup(void *, void *, size_t)');
const vault_directory_backup_count = library.func('uint64_t vault_directory_backup_count(void *)');
const vault_directory_restore_private_key = library.func('bool vault_directory_restore_private_key(void *, const char *, size_t, void *, void *, bool)');
const vault_directory_load_owner_signing_key = library.func('void * vault_directory_load_owner_signing_key(void *, const char *, size_t)');
const vault_directory_load_owner_signing_key_generation = library.func('void * vault_directory_load_owner_signing_key_generation(void *, const char *, size_t, uint16_t)');
const vault_directory_store_contact_signing_key = library.func('bool vault_directory_store_contact_signing_key(void *, const char *, size_t, void *)');
const vault_directory_load_contact_signing_key = library.func('void * vault_directory_load_contact_signing_key(void *, const char *, size_t)');
const vault_directory_list_profile_generations = library.func('RevaultBuffer vault_directory_list_profile_generations(void *, const char *, size_t)');
const vault_directory_rotate_private_key = library.func('RevaultBuffer vault_directory_rotate_private_key(void *, const char *, size_t)');
const vault_directory_remember_lockbox = library.func('bool vault_directory_remember_lockbox(void *, void *, size_t, const char *, size_t)');
const vault_directory_list_known_lockboxes = library.func('RevaultBuffer vault_directory_list_known_lockboxes(void *)');
const vault_directory_forget_lockbox = library.func('bool vault_directory_forget_lockbox(void *, const char *, size_t)');
const vault_directory_remember_access_slot_label = library.func('bool vault_directory_remember_access_slot_label(void *, void *, size_t, uint64_t, const char *, size_t)');
const vault_directory_list_access_slot_labels = library.func('RevaultBuffer vault_directory_list_access_slot_labels(void *, void *, size_t)');
const vault_directory_find_access_slot_labels = library.func('RevaultBuffer vault_directory_find_access_slot_labels(void *, void *, size_t, const char *, size_t)');
const vault_directory_forget_access_slot_label = library.func('bool vault_directory_forget_access_slot_label(void *, void *, size_t, uint64_t)');
const vault_directory_define_form = library.func('RevaultBuffer vault_directory_define_form(void *, const char *, size_t, const char *, size_t, const char *, size_t, void *, size_t)');
const vault_directory_resolve_form = library.func('RevaultBuffer vault_directory_resolve_form(void *, const char *, size_t)');
const vault_directory_list_forms = library.func('RevaultBuffer vault_directory_list_forms(void *)');
const vault_directory_list_form_revisions = library.func('RevaultBuffer vault_directory_list_form_revisions(void *, const char *, size_t)');
const vault_directory_seed_forms = library.func('size_t vault_directory_seed_forms(void *)');
const vault_directory_remember_password = library.func('bool vault_directory_remember_password(void *, void *, size_t, void *, size_t)');
const vault_directory_remembered_password = library.func('RevaultBuffer vault_directory_remembered_password(void *, void *, size_t)');
const vault_backup_default = library.func('RevaultBuffer vault_backup_default(const char *, size_t, bool)');
const vault_restore_default = library.func('RevaultBuffer vault_restore_default(const char *, size_t, bool)');
const vault_directory_free = library.func('void vault_directory_free(void *)');
const vault_read_only_open = library.func('void * vault_read_only_open(const char *, size_t, void *, size_t)');
const vault_read_only_open_default = library.func('void * vault_read_only_open_default(void *, size_t)');
const vault_read_only_list_profile_names = library.func('RevaultBuffer vault_read_only_list_profile_names(void *)');
const vault_read_only_list_contact_names = library.func('RevaultBuffer vault_read_only_list_contact_names(void *)');
const vault_read_only_list_form_aliases = library.func('RevaultBuffer vault_read_only_list_form_aliases(void *)');
const vault_read_only_list_known_lockboxes = library.func('RevaultBuffer vault_read_only_list_known_lockboxes(void *)');
const vault_read_only_free = library.func('void vault_read_only_free(void *)');
const vault_agent_serve = library.func('bool vault_agent_serve(void)');
const vault_agent_verify_transport = library.func('bool vault_agent_verify_transport(void)');
const vault_agent_get = library.func('RevaultBuffer vault_agent_get(void *, size_t)');
const vault_agent_put = library.func('bool vault_agent_put(void *, size_t, void *, size_t)');
const vault_agent_forget = library.func('bool vault_agent_forget(void *, size_t)');
const vault_agent_stop = library.func('bool vault_agent_stop(void)');
const vault_agent_start = library.func('bool vault_agent_start(void)');
const vault_agent_list = library.func('RevaultBuffer vault_agent_list(void)');
const vault_agent_sleep_support = library.func('RevaultBuffer vault_agent_sleep_support(void)');
const vault_platform_status = library.func('RevaultBuffer vault_platform_status(void)');
const vault_platform_set_scope = library.func('bool vault_platform_set_scope(const char *, size_t)');
const vault_platform_forget_password = library.func('bool vault_platform_forget_password(void)');
const vault_platform_put_password = library.func('bool vault_platform_put_password(void *, size_t)');
const vault_platform_enable = library.func('bool vault_platform_enable(void)');
const vault_platform_disable = library.func('bool vault_platform_disable(void)');
const vault_platform_disabled = library.func('bool vault_platform_disabled(void)');
const vault_platform_get_password = library.func('RevaultBuffer vault_platform_get_password(void)');
const vault_default_directory = library.func('RevaultBuffer vault_default_directory(void)');
const vault_default_path = library.func('RevaultBuffer vault_default_path(void)');
const vault_agent_log_path = library.func('RevaultBuffer vault_agent_log_path(void)');
const vault_agent_log_destination = library.func('RevaultBuffer vault_agent_log_destination(void)');
const vault_agent_get_vault_unlock_key = library.func('RevaultBuffer vault_agent_get_vault_unlock_key(const char *, size_t)');
const vault_agent_put_vault_unlock_key = library.func('bool vault_agent_put_vault_unlock_key(const char *, size_t, void *, size_t, uint64_t)');
const vault_agent_forget_vault_unlock_key = library.func('bool vault_agent_forget_vault_unlock_key(const char *, size_t)');
const vault_agent_get_owner_signing_key = library.func('void * vault_agent_get_owner_signing_key(const char *, size_t, const char *, size_t)');
const vault_agent_put_owner_signing_key = library.func('bool vault_agent_put_owner_signing_key(const char *, size_t, const char *, size_t, void *, uint64_t)');
const vault_agent_forget_owner_signing_key = library.func('bool vault_agent_forget_owner_signing_key(const char *, size_t, const char *, size_t)');
const vault_agent_begin_activity = library.func('void * vault_agent_begin_activity(const char *, size_t)');
const vault_agent_end_activity = library.func('void vault_agent_end_activity(void *)');
const vault_local = library.func('void * vault_local(void)');
const vault_create_lockbox_password = library.func('void * vault_create_lockbox_password(void *, const char *, size_t, void *, size_t)');
const vault_open_lockbox_password = library.func('void * vault_open_lockbox_password(void *, const char *, size_t, void *, size_t)');
const vault_create_lockbox_content_key = library.func('void * vault_create_lockbox_content_key(void *, const char *, size_t, void *, size_t, void *)');
const vault_create_lockbox_contact = library.func('void * vault_create_lockbox_contact(void *, const char *, size_t, void *, const char *, size_t, void *)');
const vault_open_lockbox_content_key = library.func('void * vault_open_lockbox_content_key(void *, const char *, size_t, void *, size_t, void *)');
const vault_cache_lockbox_password = library.func('bool vault_cache_lockbox_password(void *, const char *, size_t, void *, size_t, uint64_t)');
const vault_close_lockbox = library.func('bool vault_close_lockbox(void *, const char *, size_t)');
const vault_close_all = library.func('bool vault_close_all(void *)');
const vault_free = library.func('void vault_free(void *)');

function take(value) {
  if (value.ptr == null) throw new Error(lastError());
  try { return Buffer.from(koffi.decode(value.ptr, 'uint8_t', Number(value.len))); } finally { buffer_free(value); }
}
function domainView(view) {
  if (view == null || typeof view !== 'object') return view;
  return new Proxy(Object.create(null), {
    get(_target, property) {
      if (property === Symbol.toStringTag) return view.constructor.name;
      if (typeof property !== 'string' || property.startsWith('__') || property === 'bb' || property === 'bb_pos') return undefined;
      const array = view[`${property}Array`];
      if (typeof array === 'function') return array.call(view) ?? new Uint8Array();
      const length = view[`${property}Length`];
      const getter = view[property];
      if (typeof length === 'function' && typeof getter === 'function') {
        return Array.from({ length: length.call(view) }, (_, index) => domainView(getter.call(view, index)));
      }
      if (typeof getter === 'function') return domainView(getter.call(view));
      return undefined;
    },
  });
}
function decode(name, value) {
  const bytes = take(value);
  const Table = transport[name];
  const root = Table?.[`getRootAs${name}`];
  if (typeof root !== 'function') throw new TypeError(`unknown native result type: ${name}`);
  const result = domainView(root.call(Table, new ByteBuffer(bytes)));
  if (name === 'LockboxEntryList') return result.entries;
  if (name.endsWith('List')) return result.values;
  if (name === 'OptionalString') return result.present ? result.value : undefined;
  if (name === 'OptionalLockboxEntry' || name === 'OptionalFormRecord' || name === 'OptionalFormValue') return result.value ?? undefined;
  return result;
}
function lastError() { return buffer_last_error(); }
function requireValue(value) { if (!value) throw new Error(lastError()); return value; }
function requireHandle(value) { if (value == null) throw new Error(lastError()); return value; }
function withSecret(getter, callback) {
  const output = [null];
  requireValue(getter(output));
  const handle = output[0];
  if (handle == null) return undefined;
  try {
    const length = [0];
    requireValue(secret_len(handle, length));
    const bytes = Buffer.alloc(Number(length[0]));
    try {
      requireValue(secret_copy(handle, bytes, bytes.length));
      return callback(bytes);
    } finally { bytes.fill(0); }
  } finally { secret_free(handle); }
}

export class BindingOperations {
  lastErrorMessage() { return lastError(); }

  bufferLastErrorDetails() { return decode('ErrorDetails', buffer_last_error_details()); }

  lockboxFormatVersion() { return lockbox_format_version(); }

  lockboxProbeFormatVersion(bytes) { return lockbox_probe_format_version(Buffer.from(bytes), Buffer.byteLength(bytes)); }

  lockboxCreate(key) { return requireHandle(lockbox_create(Buffer.from(key), Buffer.byteLength(key))); }

  lockboxCreateWithOptions(key, cacheMode, cacheBytes, workload, worker, jobs) { return requireHandle(lockbox_create_with_options(Buffer.from(key), Buffer.byteLength(key), Buffer.from(cacheMode), Buffer.byteLength(cacheMode), cacheBytes, Buffer.from(workload), Buffer.byteLength(workload), Buffer.from(worker), Buffer.byteLength(worker), jobs)); }

  lockboxCreatePassword(password) { return requireHandle(lockbox_create_password(Buffer.from(password), Buffer.byteLength(password))); }

  lockboxCreateContact(contact) { return requireHandle(lockbox_create_contact(contact)); }

  lockboxCreateWithSigningKey(contentKey, signingKey) { return requireHandle(lockbox_create_with_signing_key(Buffer.from(contentKey), Buffer.byteLength(contentKey), signingKey)); }

  lockboxOpen(archive, key) { return requireHandle(lockbox_open(Buffer.from(archive), Buffer.byteLength(archive), Buffer.from(key), Buffer.byteLength(key))); }

  lockboxOpenWithOptions(archive, key, cacheMode, cacheBytes, workload, worker, jobs) { return requireHandle(lockbox_open_with_options(Buffer.from(archive), Buffer.byteLength(archive), Buffer.from(key), Buffer.byteLength(key), Buffer.from(cacheMode), Buffer.byteLength(cacheMode), cacheBytes, Buffer.from(workload), Buffer.byteLength(workload), Buffer.from(worker), Buffer.byteLength(worker), jobs)); }

  lockboxOpenPassword(archive, password) { return requireHandle(lockbox_open_password(Buffer.from(archive), Buffer.byteLength(archive), Buffer.from(password), Buffer.byteLength(password))); }

  lockboxOpenContact(archive, contact) { return requireHandle(lockbox_open_contact(Buffer.from(archive), Buffer.byteLength(archive), contact)); }

  lockboxAddFile(handle, path, data, replace) { return requireValue(lockbox_add_file(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(data), Buffer.byteLength(data), replace)); }

  lockboxAddFileWithPermissions(handle, path, data, permissions, replace) { return requireValue(lockbox_add_file_with_permissions(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(data), Buffer.byteLength(data), permissions, replace)); }

  lockboxGetFile(handle, path) { return take(lockbox_get_file(handle, Buffer.from(path), Buffer.byteLength(path))); }

  lockboxExtractFile(handle, source, destination, replace) { return requireValue(lockbox_extract_file(handle, Buffer.from(source), Buffer.byteLength(source), Buffer.from(destination), Buffer.byteLength(destination), replace)); }

  lockboxExtractDirectory(handle, destination, maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite) { return requireValue(lockbox_extract_directory(handle, Buffer.from(destination), Buffer.byteLength(destination), maxFileBytes, maxTotalBytes, maxFiles, restoreSymlinks, restorePermissions, overwrite)); }

  lockboxStreamContent(handle, physical) { return decode('StreamChunkList', lockbox_stream_content(handle, physical)); }

  lockboxCacheStats(handle) { return decode('CacheStats', lockbox_cache_stats(handle)); }

  lockboxImportStats(handle) { return decode('ImportStats', lockbox_import_stats(handle)); }

  lockboxResetImportStats(handle) { return requireValue(lockbox_reset_import_stats(handle)); }

  lockboxInspectFile(path) { return decode('FileInspection', lockbox_inspect_file(Buffer.from(path), Buffer.byteLength(path))); }

  lockboxPageInspection(handle) { return decode('PageInspectionList', lockbox_page_inspection(handle)); }

  lockboxRecoveryReport(handle) { return decode('RecoveryReport', lockbox_recovery_report(handle)); }

  lockboxRecoveryReportRender(handle, verbose, maxEntries) { return take(lockbox_recovery_report_render(handle, verbose, maxEntries)).toString(); }

  lockboxRecoveryScanPath(path, key) { return decode('RecoveryReport', lockbox_recovery_scan_path(Buffer.from(path), Buffer.byteLength(path), Buffer.from(key), Buffer.byteLength(key))); }

  lockboxStorageLen(handle) { return lockbox_storage_len(handle); }

  lockboxSetWorkloadProfile(handle, profile) { return requireValue(lockbox_set_workload_profile(handle, Buffer.from(profile), Buffer.byteLength(profile))); }

  lockboxSetWorkerPolicy(handle, mode, jobs) { return requireValue(lockbox_set_worker_policy(handle, Buffer.from(mode), Buffer.byteLength(mode), jobs)); }

  lockboxRuntimeOptions(handle) { return decode('RuntimeOptions', lockbox_runtime_options(handle)); }

  lockboxCommit(handle) { return requireValue(lockbox_commit(handle)); }

  lockboxCreateDir(handle, path, createParents) { return requireValue(lockbox_create_dir(handle, Buffer.from(path), Buffer.byteLength(path), createParents)); }

  lockboxDelete(handle, path) { return requireValue(lockbox_delete(handle, Buffer.from(path), Buffer.byteLength(path))); }

  lockboxRemoveDir(handle, path, recursive) { return requireValue(lockbox_remove_dir(handle, Buffer.from(path), Buffer.byteLength(path), recursive)); }

  lockboxCreateParentDirs(handle, path) { return requireValue(lockbox_create_parent_dirs(handle, Buffer.from(path), Buffer.byteLength(path))); }

  lockboxRename(handle, from, to) { return requireValue(lockbox_rename(handle, Buffer.from(from), Buffer.byteLength(from), Buffer.from(to), Buffer.byteLength(to))); }

  lockboxList(handle, path, recursive) { return decode('LockboxEntryList', lockbox_list(handle, Buffer.from(path), Buffer.byteLength(path), recursive)); }

  lockboxListWithOptions(handle, path, glob, recursive, includeFiles, includeSymlinks, includeDirectories, limit) { return decode('LockboxEntryList', lockbox_list_with_options(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(glob), Buffer.byteLength(glob), recursive, includeFiles, includeSymlinks, includeDirectories, limit)); }

  lockboxStat(handle, path) { return decode('OptionalLockboxEntry', lockbox_stat(handle, Buffer.from(path), Buffer.byteLength(path))); }

  lockboxSetVariable(handle, name, value) { return requireValue(lockbox_set_variable(handle, Buffer.from(name), Buffer.byteLength(name), Buffer.from(value), Buffer.byteLength(value))); }

  lockboxSetSecretVariable(handle, name, value) {
    const secret = Buffer.from(value);
    try { return requireValue(lockbox_set_secret_variable(handle, Buffer.from(name), Buffer.byteLength(name), secret, secret.length)); }
    finally { secret.fill(0); }
  }

  lockboxGetVariable(handle, name) {
    return decode('OptionalString', lockbox_get_variable(handle, Buffer.from(name), Buffer.byteLength(name)));
  }

  lockboxWithSecretVariable(handle, name, callback) {
    return withSecret(output => lockbox_get_secret_variable(handle, Buffer.from(name), Buffer.byteLength(name), output), callback);
  }

  lockboxDeleteVariable(handle, name) { return requireValue(lockbox_delete_variable(handle, Buffer.from(name), Buffer.byteLength(name))); }

  lockboxMoveVariables(handle, movesFlatbuffer) { return requireValue(lockbox_move_variables(handle, Buffer.from(movesFlatbuffer), Buffer.byteLength(movesFlatbuffer))); }

  lockboxListVariables(handle) { return decode('VariableList', lockbox_list_variables(handle)); }

  lockboxVariableSensitivity(handle, name) { return decode('OptionalString', lockbox_variable_sensitivity(handle, Buffer.from(name), Buffer.byteLength(name))); }

  lockboxAddSymlink(handle, path, target, replace) { return requireValue(lockbox_add_symlink(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(target), Buffer.byteLength(target), replace)); }

  lockboxGetSymlinkTarget(handle, path) { return take(lockbox_get_symlink_target(handle, Buffer.from(path), Buffer.byteLength(path))).toString(); }

  lockboxId(handle) { return take(lockbox_id(handle)); }

  lockboxExists(handle, path) { return lockbox_exists(handle, Buffer.from(path), Buffer.byteLength(path)); }

  lockboxIsDir(handle, path) { return lockbox_is_dir(handle, Buffer.from(path), Buffer.byteLength(path)); }

  lockboxPermissions(handle, path) { return lockbox_permissions(handle, Buffer.from(path), Buffer.byteLength(path)); }

  lockboxSetPermissions(handle, path, permissions) { return requireValue(lockbox_set_permissions(handle, Buffer.from(path), Buffer.byteLength(path), permissions)); }

  lockboxReadRange(handle, path, offset, len) { return take(lockbox_read_range(handle, Buffer.from(path), Buffer.byteLength(path), offset, len)); }

  lockboxRecoveryScan(bytes, key) { return decode('RecoveryReport', lockbox_recovery_scan(Buffer.from(bytes), Buffer.byteLength(bytes), Buffer.from(key), Buffer.byteLength(key))); }

  lockboxRecoverySalvage(bytes, key, signingKey) { return requireHandle(lockbox_recovery_salvage(Buffer.from(bytes), Buffer.byteLength(bytes), Buffer.from(key), Buffer.byteLength(key), signingKey)); }

  lockboxAddPassword(handle, password) { return lockbox_add_password(handle, Buffer.from(password), Buffer.byteLength(password)); }

  lockboxAddContact(handle, contact, name) { return lockbox_add_contact(handle, contact, Buffer.from(name), Buffer.byteLength(name)); }

  lockboxDeleteKey(handle, id) { return requireValue(lockbox_delete_key(handle, id)); }

  lockboxListKeySlots(handle) { return decode('KeySlotList', lockbox_list_key_slots(handle)); }

  lockboxSetOwnerSigningKey(handle, key) { return requireValue(lockbox_set_owner_signing_key(handle, key)); }

  lockboxOwnerInspection(handle) { return decode('OwnerInspection', lockbox_owner_inspection(handle)); }

  lockboxDefineForm(handle, alias, name, description, fieldsFlatbuffer) { return decode('FormDefinition', lockbox_define_form(handle, Buffer.from(alias), Buffer.byteLength(alias), Buffer.from(name), Buffer.byteLength(name), Buffer.from(description), Buffer.byteLength(description), Buffer.from(fieldsFlatbuffer), Buffer.byteLength(fieldsFlatbuffer))); }

  lockboxListFormDefinitions(handle) { return decode('FormDefinitionList', lockbox_list_form_definitions(handle)); }

  lockboxResolveForm(handle, reference) { return decode('FormDefinition', lockbox_resolve_form(handle, Buffer.from(reference), Buffer.byteLength(reference))); }

  lockboxListFormRevisions(handle, typeId) { return decode('FormDefinitionList', lockbox_list_form_revisions(handle, Buffer.from(typeId), Buffer.byteLength(typeId))); }

  lockboxCreateFormRecord(handle, path, typeReference, name) { return decode('FormRecord', lockbox_create_form_record(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(typeReference), Buffer.byteLength(typeReference), Buffer.from(name), Buffer.byteLength(name))); }

  lockboxSetFormField(handle, path, field, value) { return requireValue(lockbox_set_form_field(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(field), Buffer.byteLength(field), Buffer.from(value), Buffer.byteLength(value))); }

  lockboxSetSecretFormField(handle, path, field, value) {
    const secret = Buffer.from(value);
    try { return requireValue(lockbox_set_secret_form_field(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(field), Buffer.byteLength(field), secret, secret.length)); }
    finally { secret.fill(0); }
  }

  lockboxListFormRecords(handle) { return decode('FormRecordList', lockbox_list_form_records(handle)); }

  lockboxGetFormRecord(handle, path) { return decode('OptionalFormRecord', lockbox_get_form_record(handle, Buffer.from(path), Buffer.byteLength(path))); }

  lockboxDeleteFormRecord(handle, path) { return requireValue(lockbox_delete_form_record(handle, Buffer.from(path), Buffer.byteLength(path))); }

  lockboxMoveFormRecords(handle, movesFlatbuffer) { return requireValue(lockbox_move_form_records(handle, Buffer.from(movesFlatbuffer), Buffer.byteLength(movesFlatbuffer))); }

  lockboxGetFormField(handle, path, field) { return decode('OptionalFormValue', lockbox_get_form_field(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(field), Buffer.byteLength(field))); }

  lockboxWithSecretFormField(handle, path, field, callback) {
    return withSecret(output => lockbox_get_secret_form_field(handle, Buffer.from(path), Buffer.byteLength(path), Buffer.from(field), Buffer.byteLength(field), output), callback);
  }

  lockboxToBytes(handle) { return take(lockbox_to_bytes(handle)); }

  lockboxFree(handle) { return lockbox_free(handle); }

  vaultIsRunning() { return vault_is_running(); }

  vaultForgetAll() { return requireValue(vault_forget_all()); }

  keyContactGenerate() { return requireHandle(key_contact_generate()); }

  keyContactFromPrivate(bytes) { return requireHandle(key_contact_from_private(Buffer.from(bytes), Buffer.byteLength(bytes))); }

  keyContactPublic(handle) { return take(key_contact_public(handle)); }

  keyContactPrivate(handle) { return take(key_contact_private(handle)); }

  keyContactPublicFromBytes(bytes) { return requireHandle(key_contact_public_from_bytes(Buffer.from(bytes), Buffer.byteLength(bytes))); }

  keyContactPublicFree(handle) { return key_contact_public_free(handle); }

  keyContactFree(handle) { return key_contact_free(handle); }

  keyContactEncrypt(contact, contentKey) { return requireHandle(key_contact_encrypt(contact, Buffer.from(contentKey), Buffer.byteLength(contentKey))); }

  keyContactDecrypt(contact, wrapped) { return take(key_contact_decrypt(contact, wrapped)); }

  keyContactWrappedPublic(wrapped) { return take(key_contact_wrapped_public(wrapped)); }

  keyContactWrappedCiphertext(wrapped) { return take(key_contact_wrapped_ciphertext(wrapped)); }

  keyContactWrappedEncrypted(wrapped) { return take(key_contact_wrapped_encrypted(wrapped)); }

  keyContactWrappedFree(handle) { return key_contact_wrapped_free(handle); }

  keySigningGenerate() { return requireHandle(key_signing_generate()); }

  keySigningFromPrivate(bytes) { return requireHandle(key_signing_from_private(Buffer.from(bytes), Buffer.byteLength(bytes))); }

  keySigningPublic(handle) { return take(key_signing_public(handle)); }

  keySigningPrivate(handle) { return take(key_signing_private(handle)); }

  keySigningPublicFromBytes(bytes) { return requireHandle(key_signing_public_from_bytes(Buffer.from(bytes), Buffer.byteLength(bytes))); }

  keySigningPublicFree(handle) { return key_signing_public_free(handle); }

  keySigningFree(handle) { return key_signing_free(handle); }

  vaultKeyExportPrivate(key, format) { return take(vault_key_export_private(key, Buffer.from(format), Buffer.byteLength(format))); }

  vaultKeyExportPublic(key, format) { return take(vault_key_export_public(key, Buffer.from(format), Buffer.byteLength(format))); }

  vaultKeyImportPrivate(bytes) { return requireHandle(vault_key_import_private(Buffer.from(bytes), Buffer.byteLength(bytes))); }

  vaultKeyImportPublic(bytes) { return requireHandle(vault_key_import_public(Buffer.from(bytes), Buffer.byteLength(bytes))); }

  vaultKeyFingerprint(key) { return take(vault_key_fingerprint(key)); }

  vaultKeyFormatHex(bytes) { return take(vault_key_format_hex(Buffer.from(bytes), Buffer.byteLength(bytes))).toString(); }

  vaultKeyDecodeHex(text) { return take(vault_key_decode_hex(Buffer.from(text), Buffer.byteLength(text))); }

  vaultKeyFormatCrockford(bytes) { return take(vault_key_format_crockford(Buffer.from(bytes), Buffer.byteLength(bytes))).toString(); }

  vaultKeyFormatCrockfordReading(code) { return take(vault_key_format_crockford_reading(Buffer.from(code), Buffer.byteLength(code))).toString(); }

  vaultKeyDecodeCrockford(code) { return take(vault_key_decode_crockford(Buffer.from(code), Buffer.byteLength(code))); }

  vaultKeyHexEncode(bytes) { return take(vault_key_hex_encode(Buffer.from(bytes), Buffer.byteLength(bytes))).toString(); }

  vaultKeyHexDecode(text) { return take(vault_key_hex_decode(Buffer.from(text), Buffer.byteLength(text))); }

  vaultDirectoryOpen(root, password) { return requireHandle(vault_directory_open(Buffer.from(root), Buffer.byteLength(root), Buffer.from(password), Buffer.byteLength(password))); }

  vaultStructureVersionCurrent() { return vault_structure_version_current(); }

  vaultDirectoryProbeStructureVersion(root, password) { return vault_directory_probe_structure_version(Buffer.from(root), Buffer.byteLength(root), Buffer.from(password), Buffer.byteLength(password)); }

  vaultDirectoryOpenOrCreateDefault(password) { return requireHandle(vault_directory_open_or_create_default(Buffer.from(password), Buffer.byteLength(password))); }

  vaultDirectoryReplaceDefault(password) { return requireHandle(vault_directory_replace_default(Buffer.from(password), Buffer.byteLength(password))); }

  vaultDirectoryChangePassword(root, oldPassword, newPassword) { return requireValue(vault_directory_change_password(Buffer.from(root), Buffer.byteLength(root), Buffer.from(oldPassword), Buffer.byteLength(oldPassword), Buffer.from(newPassword), Buffer.byteLength(newPassword))); }

  vaultDirectoryChangeDefaultPassword(oldPassword, newPassword) { return requireValue(vault_directory_change_default_password(Buffer.from(oldPassword), Buffer.byteLength(oldPassword), Buffer.from(newPassword), Buffer.byteLength(newPassword))); }

  vaultDirectoryReplace(root, password) { return requireHandle(vault_directory_replace(Buffer.from(root), Buffer.byteLength(root), Buffer.from(password), Buffer.byteLength(password))); }

  vaultDirectoryOpenOrCreate(root, password) { return requireHandle(vault_directory_open_or_create(Buffer.from(root), Buffer.byteLength(root), Buffer.from(password), Buffer.byteLength(password))); }

  vaultDirectoryRoot(handle) { return take(vault_directory_root(handle)).toString(); }

  vaultDirectoryStructureVersion(handle) { return vault_directory_structure_version(handle); }

  vaultDirectoryListPrivateKeys(handle) { return decode('StringList', vault_directory_list_private_keys(handle)); }

  vaultDirectoryListPrivateKeyNames(handle) { return decode('StringList', vault_directory_list_private_key_names(handle)); }

  vaultDirectoryListContactNames(handle) { return decode('StringList', vault_directory_list_contact_names(handle)); }

  vaultDirectoryListFormAliases(handle) { return decode('StringList', vault_directory_list_form_aliases(handle)); }

  vaultDirectoryPrivateKeyExists(handle, name) { return vault_directory_private_key_exists(handle, Buffer.from(name), Buffer.byteLength(name)); }

  vaultDirectoryDeletePrivateKey(handle, name) { return requireValue(vault_directory_delete_private_key(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryStorePrivateKey(handle, name, key) { return requireValue(vault_directory_store_private_key(handle, Buffer.from(name), Buffer.byteLength(name), key)); }

  vaultDirectoryLoadPrivateKey(handle, name) { return requireHandle(vault_directory_load_private_key(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryLoadPrivateKeyGeneration(handle, name, index) { return requireHandle(vault_directory_load_private_key_generation(handle, Buffer.from(name), Buffer.byteLength(name), index)); }

  vaultDirectoryStoreContact(handle, name, key) { return requireValue(vault_directory_store_contact(handle, Buffer.from(name), Buffer.byteLength(name), key)); }

  vaultDirectoryLoadContact(handle, name) { return requireHandle(vault_directory_load_contact(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryContactExists(handle, name) { return vault_directory_contact_exists(handle, Buffer.from(name), Buffer.byteLength(name)); }

  vaultDirectoryDeleteContact(handle, name) { return requireValue(vault_directory_delete_contact(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryListContacts(handle) { return decode('ContactList', vault_directory_list_contacts(handle)); }

  vaultDirectoryStoreProfileEmail(handle, name, email) { return requireValue(vault_directory_store_profile_email(handle, Buffer.from(name), Buffer.byteLength(name), Buffer.from(email), Buffer.byteLength(email))); }

  vaultDirectoryProfileEmail(handle, name) { return decode('OptionalString', vault_directory_profile_email(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryStoreBackup(handle, id, bytes) { return requireValue(vault_directory_store_backup(handle, Buffer.from(id), Buffer.byteLength(id), Buffer.from(bytes), Buffer.byteLength(bytes))); }

  vaultDirectoryLoadBackup(handle, id) { return take(vault_directory_load_backup(handle, Buffer.from(id), Buffer.byteLength(id))); }

  vaultDirectoryBackupCount(handle) { return vault_directory_backup_count(handle); }

  vaultDirectoryRestorePrivateKey(handle, name, key, signingKey, overwrite) { return requireValue(vault_directory_restore_private_key(handle, Buffer.from(name), Buffer.byteLength(name), key, signingKey, overwrite)); }

  vaultDirectoryLoadOwnerSigningKey(handle, name) { return requireHandle(vault_directory_load_owner_signing_key(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryLoadOwnerSigningKeyGeneration(handle, name, index) { return requireHandle(vault_directory_load_owner_signing_key_generation(handle, Buffer.from(name), Buffer.byteLength(name), index)); }

  vaultDirectoryStoreContactSigningKey(handle, name, key) { return requireValue(vault_directory_store_contact_signing_key(handle, Buffer.from(name), Buffer.byteLength(name), key)); }

  vaultDirectoryLoadContactSigningKey(handle, name) { return requireHandle(vault_directory_load_contact_signing_key(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryListProfileGenerations(handle, name) { return decode('ProfileHistory', vault_directory_list_profile_generations(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryRotatePrivateKey(handle, name) { return decode('ProfileHistory', vault_directory_rotate_private_key(handle, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryRememberLockbox(handle, id, path) { return requireValue(vault_directory_remember_lockbox(handle, Buffer.from(id), Buffer.byteLength(id), Buffer.from(path), Buffer.byteLength(path))); }

  vaultDirectoryListKnownLockboxes(handle) { return decode('KnownLockboxList', vault_directory_list_known_lockboxes(handle)); }

  vaultDirectoryForgetLockbox(handle, path) { return requireValue(vault_directory_forget_lockbox(handle, Buffer.from(path), Buffer.byteLength(path))); }

  vaultDirectoryRememberAccessSlotLabel(handle, id, slotId, name) { return requireValue(vault_directory_remember_access_slot_label(handle, Buffer.from(id), Buffer.byteLength(id), slotId, Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryListAccessSlotLabels(handle, id) { return decode('AccessSlotLabelList', vault_directory_list_access_slot_labels(handle, Buffer.from(id), Buffer.byteLength(id))); }

  vaultDirectoryFindAccessSlotLabels(handle, id, name) { return decode('AccessSlotLabelList', vault_directory_find_access_slot_labels(handle, Buffer.from(id), Buffer.byteLength(id), Buffer.from(name), Buffer.byteLength(name))); }

  vaultDirectoryForgetAccessSlotLabel(handle, id, slotId) { return requireValue(vault_directory_forget_access_slot_label(handle, Buffer.from(id), Buffer.byteLength(id), slotId)); }

  vaultDirectoryDefineForm(handle, alias, name, description, fieldsFlatbuffer) { return decode('FormDefinition', vault_directory_define_form(handle, Buffer.from(alias), Buffer.byteLength(alias), Buffer.from(name), Buffer.byteLength(name), Buffer.from(description), Buffer.byteLength(description), Buffer.from(fieldsFlatbuffer), Buffer.byteLength(fieldsFlatbuffer))); }

  vaultDirectoryResolveForm(handle, reference) { return decode('FormDefinition', vault_directory_resolve_form(handle, Buffer.from(reference), Buffer.byteLength(reference))); }

  vaultDirectoryListForms(handle) { return decode('FormDefinitionList', vault_directory_list_forms(handle)); }

  vaultDirectoryListFormRevisions(handle, typeId) { return decode('FormDefinitionList', vault_directory_list_form_revisions(handle, Buffer.from(typeId), Buffer.byteLength(typeId))); }

  vaultDirectorySeedForms(handle) { return vault_directory_seed_forms(handle); }

  vaultDirectoryRememberPassword(handle, id, password) { return requireValue(vault_directory_remember_password(handle, Buffer.from(id), Buffer.byteLength(id), Buffer.from(password), Buffer.byteLength(password))); }

  vaultDirectoryRememberedPassword(handle, id) { return take(vault_directory_remembered_password(handle, Buffer.from(id), Buffer.byteLength(id))); }

  vaultBackupDefault(path, overwrite) { return decode('VaultBackupManifest', vault_backup_default(Buffer.from(path), Buffer.byteLength(path), overwrite)); }

  vaultRestoreDefault(path, overwrite) { return decode('VaultBackupManifest', vault_restore_default(Buffer.from(path), Buffer.byteLength(path), overwrite)); }

  vaultDirectoryFree(handle) { return vault_directory_free(handle); }

  vaultReadOnlyOpen(root, password) { return requireHandle(vault_read_only_open(Buffer.from(root), Buffer.byteLength(root), Buffer.from(password), Buffer.byteLength(password))); }

  vaultReadOnlyOpenDefault(password) { return requireHandle(vault_read_only_open_default(Buffer.from(password), Buffer.byteLength(password))); }

  vaultReadOnlyListProfileNames(handle) { return decode('StringList', vault_read_only_list_profile_names(handle)); }

  vaultReadOnlyListContactNames(handle) { return decode('StringList', vault_read_only_list_contact_names(handle)); }

  vaultReadOnlyListFormAliases(handle) { return decode('StringList', vault_read_only_list_form_aliases(handle)); }

  vaultReadOnlyListKnownLockboxes(handle) { return decode('KnownLockboxList', vault_read_only_list_known_lockboxes(handle)); }

  vaultReadOnlyFree(handle) { return vault_read_only_free(handle); }

  vaultAgentServe() { return requireValue(vault_agent_serve()); }

  vaultAgentVerifyTransport() { return requireValue(vault_agent_verify_transport()); }

  vaultAgentGet(id) { return take(vault_agent_get(Buffer.from(id), Buffer.byteLength(id))); }

  vaultAgentPut(id, key) { return requireValue(vault_agent_put(Buffer.from(id), Buffer.byteLength(id), Buffer.from(key), Buffer.byteLength(key))); }

  vaultAgentForget(id) { return requireValue(vault_agent_forget(Buffer.from(id), Buffer.byteLength(id))); }

  vaultAgentStop() { return requireValue(vault_agent_stop()); }

  vaultAgentStart() { return requireValue(vault_agent_start()); }

  vaultAgentList() { return decode('AgentEntryList', vault_agent_list()); }

  vaultAgentSleepSupport() { return decode('SleepSupport', vault_agent_sleep_support()); }

  vaultPlatformStatus() { return decode('PlatformStatus', vault_platform_status()); }

  vaultPlatformSetScope(scope) { return requireValue(vault_platform_set_scope(Buffer.from(scope), Buffer.byteLength(scope))); }

  vaultPlatformForgetPassword() { return requireValue(vault_platform_forget_password()); }

  vaultPlatformPutPassword(password) { return requireValue(vault_platform_put_password(Buffer.from(password), Buffer.byteLength(password))); }

  vaultPlatformEnable() { return requireValue(vault_platform_enable()); }

  vaultPlatformDisable() { return requireValue(vault_platform_disable()); }

  vaultPlatformDisabled() { return vault_platform_disabled(); }

  vaultPlatformGetPassword() { return take(vault_platform_get_password()); }

  vaultDefaultDirectory() { return take(vault_default_directory()).toString(); }

  vaultDefaultPath() { return take(vault_default_path()).toString(); }

  vaultAgentLogPath() { return take(vault_agent_log_path()).toString(); }

  vaultAgentLogDestination() { return take(vault_agent_log_destination()).toString(); }

  vaultAgentGetVaultUnlockKey(vaultId) { return take(vault_agent_get_vault_unlock_key(Buffer.from(vaultId), Buffer.byteLength(vaultId))); }

  vaultAgentPutVaultUnlockKey(vaultId, key, ttlSeconds) { return requireValue(vault_agent_put_vault_unlock_key(Buffer.from(vaultId), Buffer.byteLength(vaultId), Buffer.from(key), Buffer.byteLength(key), ttlSeconds)); }

  vaultAgentForgetVaultUnlockKey(vaultId) { return requireValue(vault_agent_forget_vault_unlock_key(Buffer.from(vaultId), Buffer.byteLength(vaultId))); }

  vaultAgentGetOwnerSigningKey(vaultId, profile) { return requireHandle(vault_agent_get_owner_signing_key(Buffer.from(vaultId), Buffer.byteLength(vaultId), Buffer.from(profile), Buffer.byteLength(profile))); }

  vaultAgentPutOwnerSigningKey(vaultId, profile, key, ttlSeconds) { return requireValue(vault_agent_put_owner_signing_key(Buffer.from(vaultId), Buffer.byteLength(vaultId), Buffer.from(profile), Buffer.byteLength(profile), key, ttlSeconds)); }

  vaultAgentForgetOwnerSigningKey(vaultId, profile) { return requireValue(vault_agent_forget_owner_signing_key(Buffer.from(vaultId), Buffer.byteLength(vaultId), Buffer.from(profile), Buffer.byteLength(profile))); }

  vaultAgentBeginActivity(kind) { return requireHandle(vault_agent_begin_activity(Buffer.from(kind), Buffer.byteLength(kind))); }

  vaultAgentEndActivity(handle) { return vault_agent_end_activity(handle); }

  vaultLocal() { return requireHandle(vault_local()); }

  vaultCreateLockboxPassword(vault, path, password) { return requireHandle(vault_create_lockbox_password(vault, Buffer.from(path), Buffer.byteLength(path), Buffer.from(password), Buffer.byteLength(password))); }

  vaultOpenLockboxPassword(vault, path, password) { return requireHandle(vault_open_lockbox_password(vault, Buffer.from(path), Buffer.byteLength(path), Buffer.from(password), Buffer.byteLength(password))); }

  vaultCreateLockboxContentKey(vault, path, contentKey, signingKey) { return requireHandle(vault_create_lockbox_content_key(vault, Buffer.from(path), Buffer.byteLength(path), Buffer.from(contentKey), Buffer.byteLength(contentKey), signingKey)); }

  vaultCreateLockboxContact(vault, path, contact, name, signingKey) { return requireHandle(vault_create_lockbox_contact(vault, Buffer.from(path), Buffer.byteLength(path), contact, Buffer.from(name), Buffer.byteLength(name), signingKey)); }

  vaultOpenLockboxContentKey(vault, path, contentKey, signingKey) { return requireHandle(vault_open_lockbox_content_key(vault, Buffer.from(path), Buffer.byteLength(path), Buffer.from(contentKey), Buffer.byteLength(contentKey), signingKey)); }

  vaultCacheLockboxPassword(vault, path, password, ttlSeconds) { return requireValue(vault_cache_lockbox_password(vault, Buffer.from(path), Buffer.byteLength(path), Buffer.from(password), Buffer.byteLength(password), ttlSeconds)); }

  vaultCloseLockbox(vault, path) { return requireValue(vault_close_lockbox(vault, Buffer.from(path), Buffer.byteLength(path))); }

  vaultCloseAll(vault) { return requireValue(vault_close_all(vault)); }

  vaultFree(vault) { return vault_free(vault); }

  freeBuffer(value) { buffer_free(value); }
}
