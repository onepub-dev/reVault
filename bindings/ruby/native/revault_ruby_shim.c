#include "revault_api.h"

/* The shim must never link to a second carrier. All calls, including buffer
 * release, use addresses resolved from Ruby's selected native library. */
static void (*selected_buffer_free)(RevaultBuffer value);
static RevaultBuffer (*selected_buffer_last_error_details)(void);
static RevaultBuffer (*selected_lockbox_get_file)(const void *handle, const char *path, size_t path_len);
static RevaultBuffer (*selected_lockbox_stream_content)(const void *handle, bool physical);
static RevaultBuffer (*selected_lockbox_cache_stats)(const void *handle);
static RevaultBuffer (*selected_lockbox_import_stats)(const void *handle);
static RevaultBuffer (*selected_lockbox_inspect_file)(const char *path, size_t path_len);
static RevaultBuffer (*selected_lockbox_page_inspection)(const void *handle);
static RevaultBuffer (*selected_lockbox_recovery_report)(const void *handle);
static RevaultBuffer (*selected_lockbox_recovery_report_render)(const void *handle, bool verbose, size_t max_entries);
static RevaultBuffer (*selected_lockbox_recovery_scan_path)(const char *path, size_t path_len, const uint8_t *key, size_t key_len);
static RevaultBuffer (*selected_lockbox_runtime_options)(const void *handle);
static RevaultBuffer (*selected_lockbox_list)(const void *handle, const char *path, size_t path_len, bool recursive);
static RevaultBuffer (*selected_lockbox_list_with_options)(const void *handle, const char *path, size_t path_len, const char *glob, size_t glob_len, bool recursive, bool include_files, bool include_symlinks, bool include_directories, size_t limit);
static RevaultBuffer (*selected_lockbox_stat)(const void *handle, const char *path, size_t path_len);
static RevaultBuffer (*selected_lockbox_get_variable)(const void *handle, const char *name, size_t name_len);
static RevaultBuffer (*selected_lockbox_list_variables)(const void *handle);
static RevaultBuffer (*selected_lockbox_variable_sensitivity)(const void *handle, const char *name, size_t name_len);
static RevaultBuffer (*selected_lockbox_get_symlink_target)(const void *handle, const char *path, size_t path_len);
static RevaultBuffer (*selected_lockbox_id)(const void *handle);
static RevaultBuffer (*selected_lockbox_read_range)(const void *handle, const char *path, size_t path_len, uint64_t offset, uint64_t len);
static RevaultBuffer (*selected_lockbox_recovery_scan)(const uint8_t *bytes, size_t len, const uint8_t *key, size_t key_len);
static RevaultBuffer (*selected_lockbox_list_key_slots)(const void *handle);
static RevaultBuffer (*selected_lockbox_owner_inspection)(const void *handle);
static RevaultBuffer (*selected_lockbox_define_form)(void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len);
static RevaultBuffer (*selected_lockbox_list_form_definitions)(const void *handle);
static RevaultBuffer (*selected_lockbox_resolve_form)(const void *handle, const char *reference, size_t reference_len);
static RevaultBuffer (*selected_lockbox_list_form_revisions)(const void *handle, const char *type_id, size_t type_id_len);
static RevaultBuffer (*selected_lockbox_create_form_record)(void *handle, const char *path, size_t path_len, const char *type_reference, size_t type_len, const char *name, size_t name_len);
static RevaultBuffer (*selected_lockbox_list_form_records)(const void *handle);
static RevaultBuffer (*selected_lockbox_get_form_record)(const void *handle, const char *path, size_t path_len);
static RevaultBuffer (*selected_lockbox_get_form_field)(const void *handle, const char *path, size_t path_len, const char *field, size_t field_len);
static RevaultBuffer (*selected_lockbox_to_bytes)(const void *handle);
static RevaultBuffer (*selected_key_contact_public)(const void *handle);
static RevaultBuffer (*selected_key_contact_private)(const void *handle);
static RevaultBuffer (*selected_key_contact_decrypt)(const void *contact, const void *wrapped);
static RevaultBuffer (*selected_key_contact_wrapped_public)(const void *wrapped);
static RevaultBuffer (*selected_key_contact_wrapped_ciphertext)(const void *wrapped);
static RevaultBuffer (*selected_key_contact_wrapped_encrypted)(const void *wrapped);
static RevaultBuffer (*selected_key_signing_public)(const void *handle);
static RevaultBuffer (*selected_key_signing_private)(const void *handle);
static RevaultBuffer (*selected_vault_key_export_private)(const void *key, const char *format, size_t format_len);
static RevaultBuffer (*selected_vault_key_export_public)(const void *key, const char *format, size_t format_len);
static RevaultBuffer (*selected_vault_key_fingerprint)(const void *key);
static RevaultBuffer (*selected_vault_key_format_hex)(const uint8_t *bytes, size_t len);
static RevaultBuffer (*selected_vault_key_decode_hex)(const char *text, size_t len);
static RevaultBuffer (*selected_vault_key_format_crockford)(const uint8_t *bytes, size_t len);
static RevaultBuffer (*selected_vault_key_format_crockford_reading)(const char *code, size_t len);
static RevaultBuffer (*selected_vault_key_decode_crockford)(const char *code, size_t len);
static RevaultBuffer (*selected_vault_key_hex_encode)(const uint8_t *bytes, size_t len);
static RevaultBuffer (*selected_vault_key_hex_decode)(const char *text, size_t len);
static RevaultBuffer (*selected_vault_directory_root)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_private_keys)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_private_key_names)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_contact_names)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_form_aliases)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_contacts)(const void *handle);
static RevaultBuffer (*selected_vault_directory_profile_email)(const void *handle, const char *name, size_t name_len);
static RevaultBuffer (*selected_vault_directory_load_backup)(const void *handle, const uint8_t *id, size_t id_len);
static RevaultBuffer (*selected_vault_directory_list_profile_generations)(const void *handle, const char *name, size_t name_len);
static RevaultBuffer (*selected_vault_directory_rotate_private_key)(const void *handle, const char *name, size_t name_len);
static RevaultBuffer (*selected_vault_directory_list_known_lockboxes)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_access_slot_labels)(const void *handle, const uint8_t *id, size_t id_len);
static RevaultBuffer (*selected_vault_directory_find_access_slot_labels)(const void *handle, const uint8_t *id, size_t id_len, const char *name, size_t name_len);
static RevaultBuffer (*selected_vault_directory_define_form)(const void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len);
static RevaultBuffer (*selected_vault_directory_resolve_form)(const void *handle, const char *reference, size_t reference_len);
static RevaultBuffer (*selected_vault_directory_list_forms)(const void *handle);
static RevaultBuffer (*selected_vault_directory_list_form_revisions)(const void *handle, const char *type_id, size_t type_id_len);
static RevaultBuffer (*selected_vault_directory_remembered_password)(const void *handle, const uint8_t *id, size_t id_len);
static RevaultBuffer (*selected_vault_backup_default)(const char *path, size_t path_len, bool overwrite);
static RevaultBuffer (*selected_vault_restore_default)(const char *path, size_t path_len, bool overwrite);
static RevaultBuffer (*selected_vault_read_only_list_profile_names)(const void *handle);
static RevaultBuffer (*selected_vault_read_only_list_contact_names)(const void *handle);
static RevaultBuffer (*selected_vault_read_only_list_form_aliases)(const void *handle);
static RevaultBuffer (*selected_vault_read_only_list_known_lockboxes)(const void *handle);
static RevaultBuffer (*selected_vault_agent_get)(const uint8_t *id, size_t id_len);
static RevaultBuffer (*selected_vault_agent_list)(void);
static RevaultBuffer (*selected_vault_agent_sleep_support)(void);
static RevaultBuffer (*selected_vault_platform_status)(void);
static RevaultBuffer (*selected_vault_platform_get_password)(void);
static RevaultBuffer (*selected_vault_default_directory)(void);
static RevaultBuffer (*selected_vault_default_path)(void);
static RevaultBuffer (*selected_vault_agent_log_path)(void);
static RevaultBuffer (*selected_vault_agent_log_destination)(void);
static RevaultBuffer (*selected_vault_agent_get_vault_unlock_key)(const char *vault_id, size_t vault_id_len);

void ruby_bind_library(void *(*resolve)(const char *)) {
  selected_buffer_free = (void (*)(RevaultBuffer value))resolve("buffer_free");
  selected_buffer_last_error_details = (RevaultBuffer (*)(void))resolve("buffer_last_error_details");
  selected_lockbox_get_file = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len))resolve("lockbox_get_file");
  selected_lockbox_stream_content = (RevaultBuffer (*)(const void *handle, bool physical))resolve("lockbox_stream_content");
  selected_lockbox_cache_stats = (RevaultBuffer (*)(const void *handle))resolve("lockbox_cache_stats");
  selected_lockbox_import_stats = (RevaultBuffer (*)(const void *handle))resolve("lockbox_import_stats");
  selected_lockbox_inspect_file = (RevaultBuffer (*)(const char *path, size_t path_len))resolve("lockbox_inspect_file");
  selected_lockbox_page_inspection = (RevaultBuffer (*)(const void *handle))resolve("lockbox_page_inspection");
  selected_lockbox_recovery_report = (RevaultBuffer (*)(const void *handle))resolve("lockbox_recovery_report");
  selected_lockbox_recovery_report_render = (RevaultBuffer (*)(const void *handle, bool verbose, size_t max_entries))resolve("lockbox_recovery_report_render");
  selected_lockbox_recovery_scan_path = (RevaultBuffer (*)(const char *path, size_t path_len, const uint8_t *key, size_t key_len))resolve("lockbox_recovery_scan_path");
  selected_lockbox_runtime_options = (RevaultBuffer (*)(const void *handle))resolve("lockbox_runtime_options");
  selected_lockbox_list = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len, bool recursive))resolve("lockbox_list");
  selected_lockbox_list_with_options = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len, const char *glob, size_t glob_len, bool recursive, bool include_files, bool include_symlinks, bool include_directories, size_t limit))resolve("lockbox_list_with_options");
  selected_lockbox_stat = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len))resolve("lockbox_stat");
  selected_lockbox_get_variable = (RevaultBuffer (*)(const void *handle, const char *name, size_t name_len))resolve("lockbox_get_variable");
  selected_lockbox_list_variables = (RevaultBuffer (*)(const void *handle))resolve("lockbox_list_variables");
  selected_lockbox_variable_sensitivity = (RevaultBuffer (*)(const void *handle, const char *name, size_t name_len))resolve("lockbox_variable_sensitivity");
  selected_lockbox_get_symlink_target = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len))resolve("lockbox_get_symlink_target");
  selected_lockbox_id = (RevaultBuffer (*)(const void *handle))resolve("lockbox_id");
  selected_lockbox_read_range = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len, uint64_t offset, uint64_t len))resolve("lockbox_read_range");
  selected_lockbox_recovery_scan = (RevaultBuffer (*)(const uint8_t *bytes, size_t len, const uint8_t *key, size_t key_len))resolve("lockbox_recovery_scan");
  selected_lockbox_list_key_slots = (RevaultBuffer (*)(const void *handle))resolve("lockbox_list_key_slots");
  selected_lockbox_owner_inspection = (RevaultBuffer (*)(const void *handle))resolve("lockbox_owner_inspection");
  selected_lockbox_define_form = (RevaultBuffer (*)(void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len))resolve("lockbox_define_form");
  selected_lockbox_list_form_definitions = (RevaultBuffer (*)(const void *handle))resolve("lockbox_list_form_definitions");
  selected_lockbox_resolve_form = (RevaultBuffer (*)(const void *handle, const char *reference, size_t reference_len))resolve("lockbox_resolve_form");
  selected_lockbox_list_form_revisions = (RevaultBuffer (*)(const void *handle, const char *type_id, size_t type_id_len))resolve("lockbox_list_form_revisions");
  selected_lockbox_create_form_record = (RevaultBuffer (*)(void *handle, const char *path, size_t path_len, const char *type_reference, size_t type_len, const char *name, size_t name_len))resolve("lockbox_create_form_record");
  selected_lockbox_list_form_records = (RevaultBuffer (*)(const void *handle))resolve("lockbox_list_form_records");
  selected_lockbox_get_form_record = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len))resolve("lockbox_get_form_record");
  selected_lockbox_get_form_field = (RevaultBuffer (*)(const void *handle, const char *path, size_t path_len, const char *field, size_t field_len))resolve("lockbox_get_form_field");
  selected_lockbox_to_bytes = (RevaultBuffer (*)(const void *handle))resolve("lockbox_to_bytes");
  selected_key_contact_public = (RevaultBuffer (*)(const void *handle))resolve("key_contact_public");
  selected_key_contact_private = (RevaultBuffer (*)(const void *handle))resolve("key_contact_private");
  selected_key_contact_decrypt = (RevaultBuffer (*)(const void *contact, const void *wrapped))resolve("key_contact_decrypt");
  selected_key_contact_wrapped_public = (RevaultBuffer (*)(const void *wrapped))resolve("key_contact_wrapped_public");
  selected_key_contact_wrapped_ciphertext = (RevaultBuffer (*)(const void *wrapped))resolve("key_contact_wrapped_ciphertext");
  selected_key_contact_wrapped_encrypted = (RevaultBuffer (*)(const void *wrapped))resolve("key_contact_wrapped_encrypted");
  selected_key_signing_public = (RevaultBuffer (*)(const void *handle))resolve("key_signing_public");
  selected_key_signing_private = (RevaultBuffer (*)(const void *handle))resolve("key_signing_private");
  selected_vault_key_export_private = (RevaultBuffer (*)(const void *key, const char *format, size_t format_len))resolve("vault_key_export_private");
  selected_vault_key_export_public = (RevaultBuffer (*)(const void *key, const char *format, size_t format_len))resolve("vault_key_export_public");
  selected_vault_key_fingerprint = (RevaultBuffer (*)(const void *key))resolve("vault_key_fingerprint");
  selected_vault_key_format_hex = (RevaultBuffer (*)(const uint8_t *bytes, size_t len))resolve("vault_key_format_hex");
  selected_vault_key_decode_hex = (RevaultBuffer (*)(const char *text, size_t len))resolve("vault_key_decode_hex");
  selected_vault_key_format_crockford = (RevaultBuffer (*)(const uint8_t *bytes, size_t len))resolve("vault_key_format_crockford");
  selected_vault_key_format_crockford_reading = (RevaultBuffer (*)(const char *code, size_t len))resolve("vault_key_format_crockford_reading");
  selected_vault_key_decode_crockford = (RevaultBuffer (*)(const char *code, size_t len))resolve("vault_key_decode_crockford");
  selected_vault_key_hex_encode = (RevaultBuffer (*)(const uint8_t *bytes, size_t len))resolve("vault_key_hex_encode");
  selected_vault_key_hex_decode = (RevaultBuffer (*)(const char *text, size_t len))resolve("vault_key_hex_decode");
  selected_vault_directory_root = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_root");
  selected_vault_directory_list_private_keys = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_private_keys");
  selected_vault_directory_list_private_key_names = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_private_key_names");
  selected_vault_directory_list_contact_names = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_contact_names");
  selected_vault_directory_list_form_aliases = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_form_aliases");
  selected_vault_directory_list_contacts = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_contacts");
  selected_vault_directory_profile_email = (RevaultBuffer (*)(const void *handle, const char *name, size_t name_len))resolve("vault_directory_profile_email");
  selected_vault_directory_load_backup = (RevaultBuffer (*)(const void *handle, const uint8_t *id, size_t id_len))resolve("vault_directory_load_backup");
  selected_vault_directory_list_profile_generations = (RevaultBuffer (*)(const void *handle, const char *name, size_t name_len))resolve("vault_directory_list_profile_generations");
  selected_vault_directory_rotate_private_key = (RevaultBuffer (*)(const void *handle, const char *name, size_t name_len))resolve("vault_directory_rotate_private_key");
  selected_vault_directory_list_known_lockboxes = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_known_lockboxes");
  selected_vault_directory_list_access_slot_labels = (RevaultBuffer (*)(const void *handle, const uint8_t *id, size_t id_len))resolve("vault_directory_list_access_slot_labels");
  selected_vault_directory_find_access_slot_labels = (RevaultBuffer (*)(const void *handle, const uint8_t *id, size_t id_len, const char *name, size_t name_len))resolve("vault_directory_find_access_slot_labels");
  selected_vault_directory_define_form = (RevaultBuffer (*)(const void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len))resolve("vault_directory_define_form");
  selected_vault_directory_resolve_form = (RevaultBuffer (*)(const void *handle, const char *reference, size_t reference_len))resolve("vault_directory_resolve_form");
  selected_vault_directory_list_forms = (RevaultBuffer (*)(const void *handle))resolve("vault_directory_list_forms");
  selected_vault_directory_list_form_revisions = (RevaultBuffer (*)(const void *handle, const char *type_id, size_t type_id_len))resolve("vault_directory_list_form_revisions");
  selected_vault_directory_remembered_password = (RevaultBuffer (*)(const void *handle, const uint8_t *id, size_t id_len))resolve("vault_directory_remembered_password");
  selected_vault_backup_default = (RevaultBuffer (*)(const char *path, size_t path_len, bool overwrite))resolve("vault_backup_default");
  selected_vault_restore_default = (RevaultBuffer (*)(const char *path, size_t path_len, bool overwrite))resolve("vault_restore_default");
  selected_vault_read_only_list_profile_names = (RevaultBuffer (*)(const void *handle))resolve("vault_read_only_list_profile_names");
  selected_vault_read_only_list_contact_names = (RevaultBuffer (*)(const void *handle))resolve("vault_read_only_list_contact_names");
  selected_vault_read_only_list_form_aliases = (RevaultBuffer (*)(const void *handle))resolve("vault_read_only_list_form_aliases");
  selected_vault_read_only_list_known_lockboxes = (RevaultBuffer (*)(const void *handle))resolve("vault_read_only_list_known_lockboxes");
  selected_vault_agent_get = (RevaultBuffer (*)(const uint8_t *id, size_t id_len))resolve("vault_agent_get");
  selected_vault_agent_list = (RevaultBuffer (*)(void))resolve("vault_agent_list");
  selected_vault_agent_sleep_support = (RevaultBuffer (*)(void))resolve("vault_agent_sleep_support");
  selected_vault_platform_status = (RevaultBuffer (*)(void))resolve("vault_platform_status");
  selected_vault_platform_get_password = (RevaultBuffer (*)(void))resolve("vault_platform_get_password");
  selected_vault_default_directory = (RevaultBuffer (*)(void))resolve("vault_default_directory");
  selected_vault_default_path = (RevaultBuffer (*)(void))resolve("vault_default_path");
  selected_vault_agent_log_path = (RevaultBuffer (*)(void))resolve("vault_agent_log_path");
  selected_vault_agent_log_destination = (RevaultBuffer (*)(void))resolve("vault_agent_log_destination");
  selected_vault_agent_get_vault_unlock_key = (RevaultBuffer (*)(const char *vault_id, size_t vault_id_len))resolve("vault_agent_get_vault_unlock_key");
}


void ruby_buffer_free(RevaultBuffer *value) {
  if (value != NULL) { selected_buffer_free(*value); value->ptr = NULL; value->len = 0; }
}

void ruby_buffer_last_error_details(RevaultBuffer *out) {
  if (out != NULL) *out = selected_buffer_last_error_details();
}

void ruby_lockbox_get_file(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_get_file(handle, path, path_len);
}

void ruby_lockbox_stream_content(const void *handle, bool physical, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_stream_content(handle, physical);
}

void ruby_lockbox_cache_stats(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_cache_stats(handle);
}

void ruby_lockbox_import_stats(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_import_stats(handle);
}

void ruby_lockbox_inspect_file(const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_inspect_file(path, path_len);
}

void ruby_lockbox_page_inspection(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_page_inspection(handle);
}

void ruby_lockbox_recovery_report(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_recovery_report(handle);
}

void ruby_lockbox_recovery_report_render(const void *handle, bool verbose, size_t max_entries, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_recovery_report_render(handle, verbose, max_entries);
}

void ruby_lockbox_recovery_scan_path(const char *path, size_t path_len, const uint8_t *key, size_t key_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_recovery_scan_path(path, path_len, key, key_len);
}

void ruby_lockbox_runtime_options(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_runtime_options(handle);
}

void ruby_lockbox_list(const void *handle, const char *path, size_t path_len, bool recursive, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list(handle, path, path_len, recursive);
}

void ruby_lockbox_list_with_options(const void *handle, const char *path, size_t path_len, const char *glob, size_t glob_len, bool recursive, bool include_files, bool include_symlinks, bool include_directories, size_t limit, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list_with_options(handle, path, path_len, glob, glob_len, recursive, include_files, include_symlinks, include_directories, limit);
}

void ruby_lockbox_stat(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_stat(handle, path, path_len);
}

void ruby_lockbox_get_variable(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_get_variable(handle, name, name_len);
}

void ruby_lockbox_list_variables(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list_variables(handle);
}

void ruby_lockbox_variable_sensitivity(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_variable_sensitivity(handle, name, name_len);
}

void ruby_lockbox_get_symlink_target(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_get_symlink_target(handle, path, path_len);
}

void ruby_lockbox_id(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_id(handle);
}

void ruby_lockbox_read_range(const void *handle, const char *path, size_t path_len, uint64_t offset, uint64_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_read_range(handle, path, path_len, offset, len);
}

void ruby_lockbox_recovery_scan(const uint8_t *bytes, size_t len, const uint8_t *key, size_t key_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_recovery_scan(bytes, len, key, key_len);
}

void ruby_lockbox_list_key_slots(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list_key_slots(handle);
}

void ruby_lockbox_owner_inspection(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_owner_inspection(handle);
}

void ruby_lockbox_define_form(void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_define_form(handle, alias, alias_len, name, name_len, description, description_len, fields_flatbuffer, fields_len);
}

void ruby_lockbox_list_form_definitions(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list_form_definitions(handle);
}

void ruby_lockbox_resolve_form(const void *handle, const char *reference, size_t reference_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_resolve_form(handle, reference, reference_len);
}

void ruby_lockbox_list_form_revisions(const void *handle, const char *type_id, size_t type_id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list_form_revisions(handle, type_id, type_id_len);
}

void ruby_lockbox_create_form_record(void *handle, const char *path, size_t path_len, const char *type_reference, size_t type_len, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_create_form_record(handle, path, path_len, type_reference, type_len, name, name_len);
}

void ruby_lockbox_list_form_records(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_list_form_records(handle);
}

void ruby_lockbox_get_form_record(const void *handle, const char *path, size_t path_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_get_form_record(handle, path, path_len);
}

void ruby_lockbox_get_form_field(const void *handle, const char *path, size_t path_len, const char *field, size_t field_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_get_form_field(handle, path, path_len, field, field_len);
}

void ruby_lockbox_to_bytes(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_lockbox_to_bytes(handle);
}

void ruby_key_contact_public(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_contact_public(handle);
}

void ruby_key_contact_private(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_contact_private(handle);
}

void ruby_key_contact_decrypt(const void *contact, const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_contact_decrypt(contact, wrapped);
}

void ruby_key_contact_wrapped_public(const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_contact_wrapped_public(wrapped);
}

void ruby_key_contact_wrapped_ciphertext(const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_contact_wrapped_ciphertext(wrapped);
}

void ruby_key_contact_wrapped_encrypted(const void *wrapped, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_contact_wrapped_encrypted(wrapped);
}

void ruby_key_signing_public(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_signing_public(handle);
}

void ruby_key_signing_private(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_key_signing_private(handle);
}

void ruby_vault_key_export_private(const void *key, const char *format, size_t format_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_export_private(key, format, format_len);
}

void ruby_vault_key_export_public(const void *key, const char *format, size_t format_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_export_public(key, format, format_len);
}

void ruby_vault_key_fingerprint(const void *key, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_fingerprint(key);
}

void ruby_vault_key_format_hex(const uint8_t *bytes, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_format_hex(bytes, len);
}

void ruby_vault_key_decode_hex(const char *text, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_decode_hex(text, len);
}

void ruby_vault_key_format_crockford(const uint8_t *bytes, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_format_crockford(bytes, len);
}

void ruby_vault_key_format_crockford_reading(const char *code, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_format_crockford_reading(code, len);
}

void ruby_vault_key_decode_crockford(const char *code, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_decode_crockford(code, len);
}

void ruby_vault_key_hex_encode(const uint8_t *bytes, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_hex_encode(bytes, len);
}

void ruby_vault_key_hex_decode(const char *text, size_t len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_key_hex_decode(text, len);
}

void ruby_vault_directory_root(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_root(handle);
}

void ruby_vault_directory_list_private_keys(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_private_keys(handle);
}

void ruby_vault_directory_list_private_key_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_private_key_names(handle);
}

void ruby_vault_directory_list_contact_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_contact_names(handle);
}

void ruby_vault_directory_list_form_aliases(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_form_aliases(handle);
}

void ruby_vault_directory_list_contacts(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_contacts(handle);
}

void ruby_vault_directory_profile_email(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_profile_email(handle, name, name_len);
}

void ruby_vault_directory_load_backup(const void *handle, const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_load_backup(handle, id, id_len);
}

void ruby_vault_directory_list_profile_generations(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_profile_generations(handle, name, name_len);
}

void ruby_vault_directory_rotate_private_key(const void *handle, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_rotate_private_key(handle, name, name_len);
}

void ruby_vault_directory_list_known_lockboxes(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_known_lockboxes(handle);
}

void ruby_vault_directory_list_access_slot_labels(const void *handle, const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_access_slot_labels(handle, id, id_len);
}

void ruby_vault_directory_find_access_slot_labels(const void *handle, const uint8_t *id, size_t id_len, const char *name, size_t name_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_find_access_slot_labels(handle, id, id_len, name, name_len);
}

void ruby_vault_directory_define_form(const void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_define_form(handle, alias, alias_len, name, name_len, description, description_len, fields_flatbuffer, fields_len);
}

void ruby_vault_directory_resolve_form(const void *handle, const char *reference, size_t reference_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_resolve_form(handle, reference, reference_len);
}

void ruby_vault_directory_list_forms(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_forms(handle);
}

void ruby_vault_directory_list_form_revisions(const void *handle, const char *type_id, size_t type_id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_list_form_revisions(handle, type_id, type_id_len);
}

void ruby_vault_directory_remembered_password(const void *handle, const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_directory_remembered_password(handle, id, id_len);
}

void ruby_vault_backup_default(const char *path, size_t path_len, bool overwrite, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_backup_default(path, path_len, overwrite);
}

void ruby_vault_restore_default(const char *path, size_t path_len, bool overwrite, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_restore_default(path, path_len, overwrite);
}

void ruby_vault_read_only_list_profile_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_read_only_list_profile_names(handle);
}

void ruby_vault_read_only_list_contact_names(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_read_only_list_contact_names(handle);
}

void ruby_vault_read_only_list_form_aliases(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_read_only_list_form_aliases(handle);
}

void ruby_vault_read_only_list_known_lockboxes(const void *handle, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_read_only_list_known_lockboxes(handle);
}

void ruby_vault_agent_get(const uint8_t *id, size_t id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_agent_get(id, id_len);
}

void ruby_vault_agent_list(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_agent_list();
}

void ruby_vault_agent_sleep_support(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_agent_sleep_support();
}

void ruby_vault_platform_status(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_platform_status();
}

void ruby_vault_platform_get_password(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_platform_get_password();
}

void ruby_vault_default_directory(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_default_directory();
}

void ruby_vault_default_path(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_default_path();
}

void ruby_vault_agent_log_path(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_agent_log_path();
}

void ruby_vault_agent_log_destination(RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_agent_log_destination();
}

void ruby_vault_agent_get_vault_unlock_key(const char *vault_id, size_t vault_id_len, RevaultBuffer *out) {
  if (out != NULL) *out = selected_vault_agent_get_vault_unlock_key(vault_id, vault_id_len);
}
