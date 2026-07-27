#include "revault_api.h"

void ruby_buffer_free(RevaultBuffer *value) {
  if (value != NULL) { buffer_free(*value); value->ptr = NULL; value->len = 0; }
}

void ruby_buffer_last_error_details(RevaultBuffer *out) {
  if (out != NULL) *out = buffer_last_error_details();
}

void ruby_lockbox_get_file(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_get_file(handle, path, path_len);
}

void ruby_lockbox_stream_content(const void *handle, bool physical, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_stream_content(handle, physical);
}

void ruby_lockbox_cache_stats(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_cache_stats(handle);
}

void ruby_lockbox_import_stats(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_import_stats(handle);
}

void ruby_lockbox_inspect_file(const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_inspect_file(path, path_len);
}

void ruby_lockbox_page_inspection(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_page_inspection(handle);
}

void ruby_lockbox_recovery_report(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_recovery_report(handle);
}

void ruby_lockbox_recovery_report_render(const void *handle, bool verbose, size_t max_entries, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_recovery_report_render(handle, verbose, max_entries);
}

void ruby_lockbox_recovery_scan_path(const char *path, size_t path_len, const uint8_t *key, size_t key_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_recovery_scan_path(path, path_len, key, key_len);
}

void ruby_lockbox_runtime_options(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_runtime_options(handle);
}

void ruby_lockbox_list(const void *handle, const char *path, size_t path_len, bool recursive, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list(handle, path, path_len, recursive);
}

void ruby_lockbox_list_with_options(const void *handle, const char *path, size_t path_len, const char *glob, size_t glob_len, bool recursive, bool include_files, bool include_symlinks, bool include_directories, size_t limit, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list_with_options(handle, path, path_len, glob, glob_len, recursive, include_files, include_symlinks, include_directories, limit);
}

void ruby_lockbox_stat(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_stat(handle, path, path_len);
}

void ruby_lockbox_get_variable(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_get_variable(handle, name, name_len);
}

void ruby_lockbox_list_variables(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list_variables(handle);
}

void ruby_lockbox_variable_sensitivity(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_variable_sensitivity(handle, name, name_len);
}

void ruby_lockbox_get_symlink_target(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_get_symlink_target(handle, path, path_len);
}

void ruby_lockbox_id(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_id(handle);
}

void ruby_lockbox_read_range(const void *handle, const char *path, size_t path_len, uint64_t offset, uint64_t len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_read_range(handle, path, path_len, offset, len);
}

void ruby_lockbox_recovery_scan(const uint8_t *bytes, size_t len, const uint8_t *key, size_t key_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_recovery_scan(bytes, len, key, key_len);
}

void ruby_lockbox_list_key_slots(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list_key_slots(handle);
}

void ruby_lockbox_owner_inspection(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_owner_inspection(handle);
}

void ruby_lockbox_define_form(void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_define_form(handle, alias, alias_len, name, name_len, description, description_len, fields_flatbuffer, fields_len);
}

void ruby_lockbox_list_form_definitions(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list_form_definitions(handle);
}

void ruby_lockbox_resolve_form(const void *handle, const char *reference, size_t reference_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_resolve_form(handle, reference, reference_len);
}

void ruby_lockbox_list_form_revisions(const void *handle, const char *type_id, size_t type_id_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list_form_revisions(handle, type_id, type_id_len);
}

void ruby_lockbox_create_form_record(void *handle, const char *path, size_t path_len, const char *type_reference, size_t type_len, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_create_form_record(handle, path, path_len, type_reference, type_len, name, name_len);
}

void ruby_lockbox_list_form_records(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_list_form_records(handle);
}

void ruby_lockbox_get_form_record(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_get_form_record(handle, path, path_len);
}

void ruby_lockbox_get_form_field(const void *handle, const char *path, size_t path_len, const char *field, size_t field_len, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_get_form_field(handle, path, path_len, field, field_len);
}

void ruby_lockbox_to_bytes(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = lockbox_to_bytes(handle);
}

void ruby_key_contact_public(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = key_contact_public(handle);
}

void ruby_key_contact_private(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = key_contact_private(handle);
}

void ruby_key_contact_decrypt(const void *contact, const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = key_contact_decrypt(contact, wrapped);
}

void ruby_key_contact_wrapped_public(const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = key_contact_wrapped_public(wrapped);
}

void ruby_key_contact_wrapped_ciphertext(const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = key_contact_wrapped_ciphertext(wrapped);
}

void ruby_key_contact_wrapped_encrypted(const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = key_contact_wrapped_encrypted(wrapped);
}

void ruby_key_signing_public(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = key_signing_public(handle);
}

void ruby_key_signing_private(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = key_signing_private(handle);
}

void ruby_vault_key_export_private(const void *key, const char *format, size_t format_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_export_private(key, format, format_len);
}

void ruby_vault_key_export_public(const void *key, const char *format, size_t format_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_export_public(key, format, format_len);
}

void ruby_vault_key_fingerprint(const void *key, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_fingerprint(key);
}

void ruby_vault_key_format_hex(const uint8_t *bytes, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_format_hex(bytes, len);
}

void ruby_vault_key_decode_hex(const char *text, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_decode_hex(text, len);
}

void ruby_vault_key_format_crockford(const uint8_t *bytes, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_format_crockford(bytes, len);
}

void ruby_vault_key_format_crockford_reading(const char *code, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_format_crockford_reading(code, len);
}

void ruby_vault_key_decode_crockford(const char *code, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_decode_crockford(code, len);
}

void ruby_vault_key_hex_encode(const uint8_t *bytes, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_hex_encode(bytes, len);
}

void ruby_vault_key_hex_decode(const char *text, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_key_hex_decode(text, len);
}

void ruby_vault_directory_root(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_root(handle);
}

void ruby_vault_directory_list_private_keys(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_private_keys(handle);
}

void ruby_vault_directory_list_private_key_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_private_key_names(handle);
}

void ruby_vault_directory_list_contact_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_contact_names(handle);
}

void ruby_vault_directory_list_form_aliases(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_form_aliases(handle);
}

void ruby_vault_directory_list_contacts(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_contacts(handle);
}

void ruby_vault_directory_profile_email(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_profile_email(handle, name, name_len);
}

void ruby_vault_directory_load_backup(const void *handle, const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_load_backup(handle, id, id_len);
}

void ruby_vault_directory_list_profile_generations(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_profile_generations(handle, name, name_len);
}

void ruby_vault_directory_rotate_private_key(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_rotate_private_key(handle, name, name_len);
}

void ruby_vault_directory_list_known_lockboxes(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_known_lockboxes(handle);
}

void ruby_vault_directory_list_access_slot_labels(const void *handle, const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_access_slot_labels(handle, id, id_len);
}

void ruby_vault_directory_find_access_slot_labels(const void *handle, const uint8_t *id, size_t id_len, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_find_access_slot_labels(handle, id, id_len, name, name_len);
}

void ruby_vault_directory_define_form(const void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_define_form(handle, alias, alias_len, name, name_len, description, description_len, fields_flatbuffer, fields_len);
}

void ruby_vault_directory_resolve_form(const void *handle, const char *reference, size_t reference_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_resolve_form(handle, reference, reference_len);
}

void ruby_vault_directory_list_forms(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_forms(handle);
}

void ruby_vault_directory_list_form_revisions(const void *handle, const char *type_id, size_t type_id_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_list_form_revisions(handle, type_id, type_id_len);
}

void ruby_vault_directory_remembered_password(const void *handle, const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_directory_remembered_password(handle, id, id_len);
}

void ruby_vault_backup_default(const char *path, size_t path_len, bool overwrite, RevaultBuffer *out) {
  if (out != NULL) *out = vault_backup_default(path, path_len, overwrite);
}

void ruby_vault_restore_default(const char *path, size_t path_len, bool overwrite, RevaultBuffer *out) {
  if (out != NULL) *out = vault_restore_default(path, path_len, overwrite);
}

void ruby_vault_read_only_list_profile_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_read_only_list_profile_names(handle);
}

void ruby_vault_read_only_list_contact_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_read_only_list_contact_names(handle);
}

void ruby_vault_read_only_list_form_aliases(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_read_only_list_form_aliases(handle);
}

void ruby_vault_read_only_list_known_lockboxes(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = vault_read_only_list_known_lockboxes(handle);
}

void ruby_vault_agent_get(const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_agent_get(id, id_len);
}

void ruby_vault_agent_list(RevaultBuffer *out) {
  if (out != NULL) *out = vault_agent_list();
}

void ruby_vault_agent_sleep_support(RevaultBuffer *out) {
  if (out != NULL) *out = vault_agent_sleep_support();
}

void ruby_vault_platform_status(RevaultBuffer *out) {
  if (out != NULL) *out = vault_platform_status();
}

void ruby_vault_platform_get_password(RevaultBuffer *out) {
  if (out != NULL) *out = vault_platform_get_password();
}

void ruby_vault_default_directory(RevaultBuffer *out) {
  if (out != NULL) *out = vault_default_directory();
}

void ruby_vault_default_path(RevaultBuffer *out) {
  if (out != NULL) *out = vault_default_path();
}

void ruby_vault_agent_log_path(RevaultBuffer *out) {
  if (out != NULL) *out = vault_agent_log_path();
}

void ruby_vault_agent_log_destination(RevaultBuffer *out) {
  if (out != NULL) *out = vault_agent_log_destination();
}

void ruby_vault_agent_get_vault_unlock_key(const char *vault_id, size_t vault_id_len, RevaultBuffer *out) {
  if (out != NULL) *out = vault_agent_get_vault_unlock_key(vault_id, vault_id_len);
}
