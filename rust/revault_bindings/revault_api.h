#ifndef REVAULT_API_H
#define REVAULT_API_H
/**
 * @file revault_api.h
 * @brief Stable C ABI for encrypted reVault lockboxes and local vaults.
 *
 * The API stores files, variables, and typed form records in portable encrypted
 * lockboxes. It also manages contact keys, signing keys, the local metadata
 * vault, the session agent, and the platform secret store.
 *
 * Native objects are returned as opaque pointers and must be released with the
 * matching `_free` function. A failed pointer or Boolean result can be
 * diagnosed with `buffer_last_error()` or `buffer_last_error_details()`.
 * Returned `RevaultBuffer` values belong to the caller and must be passed to
 * `buffer_free()`. Secret getters return opaque secret handles so plaintext can
 * be copied only for the shortest practical scope and then wiped.
 *
 * @see https://github.com/onepub-dev/reVault#readme Repository overview,
 *      security model, installation, and examples.
 */
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Caller-owned bytes returned by the native library. Release with `buffer_free`. */
typedef struct { uint8_t *ptr; size_t len; } RevaultBuffer;
/* Structured buffers use the private FlatBuffers schema in
 * bindings/flatbuffers. Raw file/key buffers remain unframed bytes. Language
 * facades own these buffers and expose reVault domain objects, not transport
 * tables. */
/** Returns the major version of this stable native ABI. */
uint32_t api_abi_version(void);
/** Returns the diagnostic for the most recent failed call on this thread. */
const char *buffer_last_error(void);
/** Returns structured details for the most recent failed call on this thread. */
RevaultBuffer buffer_last_error_details(void);
/** Wipes and releases bytes returned in a `RevaultBuffer`. */
void buffer_free(RevaultBuffer value);
/** Returns the byte length owned by an opaque secret handle. */
bool secret_len(const void *handle, size_t *out_len);
/** Copies a secret into caller memory; wipe the destination immediately after use. */
bool secret_copy(const void *handle, uint8_t *destination, size_t destination_len);
/** Wipes and releases an opaque secret handle. */
void secret_free(void *handle);
/** Returns the supported lockbox format version. */
uint16_t lockbox_format_version(void);
/** Determines format version without fully opening it. */
uint16_t lockbox_probe_format_version(const uint8_t *bytes, size_t len);
/** Creates a new lockbox. */
void *lockbox_create(const uint8_t *key, size_t key_len);
/** Creates with options. */
void *lockbox_create_with_options(const uint8_t *key, size_t key_len, const char *cache_mode, size_t cache_len, uint64_t cache_bytes, const char *workload, size_t workload_len, const char *worker, size_t worker_len, size_t jobs);
/** Creates password. */
void *lockbox_create_password(const uint8_t *password, size_t len);
/** Creates contact. */
void *lockbox_create_contact(const void *contact);
/** Creates with signing key. */
void *lockbox_create_with_signing_key(const uint8_t *content_key, size_t key_len, const void *signing_key);
/** Opens an existing lockbox. */
void *lockbox_open(const uint8_t *archive, size_t archive_len, const uint8_t *key, size_t key_len);
/** Opens with options. */
void *lockbox_open_with_options(const uint8_t *archive, size_t archive_len, const uint8_t *key, size_t key_len, const char *cache_mode, size_t cache_len, uint64_t cache_bytes, const char *workload, size_t workload_len, const char *worker, size_t worker_len, size_t jobs);
/** Opens password. */
void *lockbox_open_password(const uint8_t *archive, size_t archive_len, const uint8_t *password, size_t password_len);
/** Opens contact. */
void *lockbox_open_contact(const uint8_t *archive, size_t archive_len, const void *contact);
/** Adds file. */
bool lockbox_add_file(void *handle, const char *path, size_t path_len, const uint8_t *data, size_t data_len, bool replace);
/** Adds file with permissions. */
bool lockbox_add_file_with_permissions(void *handle, const char *path, size_t path_len, const uint8_t *data, size_t data_len, uint32_t permissions, bool replace);
/** Returns file. */
RevaultBuffer lockbox_get_file(const void *handle, const char *path, size_t path_len);
/** Extracts file. */
bool lockbox_extract_file(const void *handle, const char *source, size_t source_len, const char *destination, size_t destination_len, bool replace);
/** Extracts directory. */
bool lockbox_extract_directory(const void *handle, const char *destination, size_t destination_len, uint64_t max_file_bytes, uint64_t max_total_bytes, size_t max_files, bool restore_symlinks, bool restore_permissions, bool overwrite);
/** Returns the stream content. */
RevaultBuffer lockbox_stream_content(const void *handle, bool physical);
/** Returns cache statistics for this lockbox. */
RevaultBuffer lockbox_cache_stats(const void *handle);
/** Returns import statistics for this lockbox. */
RevaultBuffer lockbox_import_stats(const void *handle);
/** Updates import stats. */
bool lockbox_reset_import_stats(const void *handle);
/** Inspects file. */
RevaultBuffer lockbox_inspect_file(const char *path, size_t path_len);
/** Returns the page inspection. */
RevaultBuffer lockbox_page_inspection(const void *handle);
/** Returns the recovery report. */
RevaultBuffer lockbox_recovery_report(const void *handle);
/** Returns the recovery report render. */
RevaultBuffer lockbox_recovery_report_render(const void *handle, bool verbose, size_t max_entries);
/** Returns the recovery scan path. */
RevaultBuffer lockbox_recovery_scan_path(const char *path, size_t path_len, const uint8_t *key, size_t key_len);
/** Returns the storage len. */
uint64_t lockbox_storage_len(const void *handle);
/** Sets workload profile. */
bool lockbox_set_workload_profile(void *handle, const char *profile, size_t profile_len);
/** Sets worker policy. */
bool lockbox_set_worker_policy(void *handle, const char *mode, size_t mode_len, size_t jobs);
/** Returns the runtime options. */
RevaultBuffer lockbox_runtime_options(const void *handle);
/** Authenticates and publishes the staged changes. */
bool lockbox_commit(void *handle);
/** Creates dir. */
bool lockbox_create_dir(void *handle, const char *path, size_t path_len, bool create_parents);
/** Removes delete. */
bool lockbox_delete(void *handle, const char *path, size_t path_len);
/** Removes dir. */
bool lockbox_remove_dir(void *handle, const char *path, size_t path_len, bool recursive);
/** Creates parent dirs. */
bool lockbox_create_parent_dirs(void *handle, const char *path, size_t path_len);
/** Updates rename. */
bool lockbox_rename(void *handle, const char *from, size_t from_len, const char *to, size_t to_len);
/** Lists list. */
RevaultBuffer lockbox_list(const void *handle, const char *path, size_t path_len, bool recursive);
/** Lists with options. */
RevaultBuffer lockbox_list_with_options(const void *handle, const char *path, size_t path_len, const char *glob, size_t glob_len, bool recursive, bool include_files, bool include_symlinks, bool include_directories, size_t limit);
/** Returns metadata for the selected lockbox entry. */
RevaultBuffer lockbox_stat(const void *handle, const char *path, size_t path_len);
/** Sets variable. */
bool lockbox_set_variable(void *handle, const char *name, size_t name_len, const char *value, size_t value_len);
/** Stores a secret variable from bytes without requiring an immutable string. */
bool lockbox_set_secret_variable(void *handle, const char *name, size_t name_len, const uint8_t *value, size_t value_len);
/** Returns variable. */
RevaultBuffer lockbox_get_variable(const void *handle, const char *name, size_t name_len);
/** Returns an opaque secret handle through `output`; inspect, copy, and free it promptly. */
bool lockbox_get_secret_variable(const void *handle, const char *name, size_t name_len, void **output);
/** Removes variable. */
bool lockbox_delete_variable(void *handle, const char *name, size_t name_len);
/** Updates variables. */
bool lockbox_move_variables(void *handle, const uint8_t *moves_flatbuffer, size_t moves_len);
/** Lists variables. */
RevaultBuffer lockbox_list_variables(const void *handle);
/** Returns the variable sensitivity. */
RevaultBuffer lockbox_variable_sensitivity(const void *handle, const char *name, size_t name_len);
/** Adds symlink. */
bool lockbox_add_symlink(void *handle, const char *path, size_t path_len, const char *target, size_t target_len, bool replace);
/** Returns symlink target. */
RevaultBuffer lockbox_get_symlink_target(const void *handle, const char *path, size_t path_len);
/** Returns the id. */
RevaultBuffer lockbox_id(const void *handle);
/** Reports whether exists. */
bool lockbox_exists(const void *handle, const char *path, size_t path_len);
/** Reports whether dir. */
bool lockbox_is_dir(const void *handle, const char *path, size_t path_len);
/** Returns the permissions. */
uint32_t lockbox_permissions(const void *handle, const char *path, size_t path_len);
/** Sets permissions. */
bool lockbox_set_permissions(void *handle, const char *path, size_t path_len, uint32_t permissions);
/** Returns range. */
RevaultBuffer lockbox_read_range(const void *handle, const char *path, size_t path_len, uint64_t offset, uint64_t len);
/** Returns the recovery scan. */
RevaultBuffer lockbox_recovery_scan(const uint8_t *bytes, size_t len, const uint8_t *key, size_t key_len);
/** Returns the recovery salvage. */
void *lockbox_recovery_salvage(const uint8_t *bytes, size_t len, const uint8_t *key, size_t key_len, const void *signing_key);
/** Adds password. */
uint64_t lockbox_add_password(void *handle, const uint8_t *password, size_t len);
/** Adds contact. */
uint64_t lockbox_add_contact(void *handle, const void *contact, const char *name, size_t name_len);
/** Removes key. */
bool lockbox_delete_key(void *handle, uint64_t id);
/** Lists key slots. */
RevaultBuffer lockbox_list_key_slots(const void *handle);
/** Sets owner signing key. */
bool lockbox_set_owner_signing_key(void *handle, const void *key);
/** Returns the owner inspection. */
RevaultBuffer lockbox_owner_inspection(const void *handle);
/** Returns the define form. */
RevaultBuffer lockbox_define_form(void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len);
/** Lists form definitions. */
RevaultBuffer lockbox_list_form_definitions(const void *handle);
/** Returns the resolve form. */
RevaultBuffer lockbox_resolve_form(const void *handle, const char *reference, size_t reference_len);
/** Lists form revisions. */
RevaultBuffer lockbox_list_form_revisions(const void *handle, const char *type_id, size_t type_id_len);
/** Creates form record. */
RevaultBuffer lockbox_create_form_record(void *handle, const char *path, size_t path_len, const char *type_reference, size_t type_len, const char *name, size_t name_len);
/** Sets form field. */
bool lockbox_set_form_field(void *handle, const char *path, size_t path_len, const char *field, size_t field_len, const char *value, size_t value_len);
/** Stores a secret form field from bytes without requiring an immutable string. */
bool lockbox_set_secret_form_field(void *handle, const char *path, size_t path_len, const char *field, size_t field_len, const uint8_t *value, size_t value_len);
/** Lists form records. */
RevaultBuffer lockbox_list_form_records(const void *handle);
/** Returns form record. */
RevaultBuffer lockbox_get_form_record(const void *handle, const char *path, size_t path_len);
/** Removes form record. */
bool lockbox_delete_form_record(void *handle, const char *path, size_t path_len);
/** Updates form records. */
bool lockbox_move_form_records(void *handle, const uint8_t *moves_flatbuffer, size_t moves_len);
/** Returns form field. */
RevaultBuffer lockbox_get_form_field(const void *handle, const char *path, size_t path_len, const char *field, size_t field_len);
/** Returns a secret field through an opaque handle; copy and free it promptly. */
bool lockbox_get_secret_form_field(const void *handle, const char *path, size_t path_len, const char *field, size_t field_len, void **output);
/** Returns the to bytes. */
RevaultBuffer lockbox_to_bytes(const void *handle);
/** Releases the native resources held by this object. */
void lockbox_free(void *handle);
/** Reports whether running. */
bool vault_is_running(void);
/** Removes all. */
bool vault_forget_all(void);
/** Generates generate. */
void *key_contact_generate(void);
/** Returns the from private. */
void *key_contact_from_private(const uint8_t *bytes, size_t len);
/** Returns the public. */
RevaultBuffer key_contact_public(const void *handle);
/** Returns the private. */
RevaultBuffer key_contact_private(const void *handle);
/** Returns the public from bytes. */
void *key_contact_public_from_bytes(const uint8_t *bytes, size_t len);
/** Returns the public free. */
void key_contact_public_free(void *handle);
/** Releases the native resources held by this object. */
void key_contact_free(void *handle);
/** Encrypts a content key for the selected contact. */
void *key_contact_encrypt(const void *contact, const uint8_t *content_key, size_t key_len);
/** Decrypts a wrapped content key for this contact. */
RevaultBuffer key_contact_decrypt(const void *contact, const void *wrapped);
/** Returns the wrapped public. */
RevaultBuffer key_contact_wrapped_public(const void *wrapped);
/** Returns the wrapped ciphertext. */
RevaultBuffer key_contact_wrapped_ciphertext(const void *wrapped);
/** Returns the wrapped encrypted. */
RevaultBuffer key_contact_wrapped_encrypted(const void *wrapped);
/** Returns the wrapped free. */
void key_contact_wrapped_free(void *handle);
/** Generates generate. */
void *key_signing_generate(void);
/** Returns the from private. */
void *key_signing_from_private(const uint8_t *bytes, size_t len);
/** Returns the public. */
RevaultBuffer key_signing_public(const void *handle);
/** Returns the private. */
RevaultBuffer key_signing_private(const void *handle);
/** Returns the public from bytes. */
void *key_signing_public_from_bytes(const uint8_t *bytes, size_t len);
/** Returns the public free. */
void key_signing_public_free(void *handle);
/** Releases the native resources held by this object. */
void key_signing_free(void *handle);
/** Returns the key export private. */
RevaultBuffer vault_key_export_private(const void *key, const char *format, size_t format_len);
/** Returns the key export public. */
RevaultBuffer vault_key_export_public(const void *key, const char *format, size_t format_len);
/** Returns the key import private. */
void *vault_key_import_private(const uint8_t *bytes, size_t len);
/** Returns the key import public. */
void *vault_key_import_public(const uint8_t *bytes, size_t len);
/** Returns the key fingerprint. */
RevaultBuffer vault_key_fingerprint(const void *key);
/** Returns the key format hex. */
RevaultBuffer vault_key_format_hex(const uint8_t *bytes, size_t len);
/** Returns the key decode hex. */
RevaultBuffer vault_key_decode_hex(const char *text, size_t len);
/** Returns the key format crockford. */
RevaultBuffer vault_key_format_crockford(const uint8_t *bytes, size_t len);
/** Returns the key format crockford reading. */
RevaultBuffer vault_key_format_crockford_reading(const char *code, size_t len);
/** Returns the key decode crockford. */
RevaultBuffer vault_key_decode_crockford(const char *code, size_t len);
/** Returns the key hex encode. */
RevaultBuffer vault_key_hex_encode(const uint8_t *bytes, size_t len);
/** Returns the key hex decode. */
RevaultBuffer vault_key_hex_decode(const char *text, size_t len);
/** Returns the directory open. */
void *vault_directory_open(const char *root, size_t root_len, const uint8_t *password, size_t password_len);
/** Returns the structure version current. */
uint32_t vault_structure_version_current(void);
/** Returns the directory probe structure version. */
uint32_t vault_directory_probe_structure_version(const char *root, size_t root_len, const uint8_t *password, size_t password_len);
/** Returns the directory open or create default. */
void *vault_directory_open_or_create_default(const uint8_t *password, size_t password_len);
/** Returns the directory replace default. */
void *vault_directory_replace_default(const uint8_t *password, size_t password_len);
/** Returns the directory change password. */
bool vault_directory_change_password(const char *root, size_t root_len, const uint8_t *old_password, size_t old_len, const uint8_t *new_password, size_t new_len);
/** Returns the directory change default password. */
bool vault_directory_change_default_password(const uint8_t *old_password, size_t old_len, const uint8_t *new_password, size_t new_len);
/** Returns the directory replace. */
void *vault_directory_replace(const char *root, size_t root_len, const uint8_t *password, size_t password_len);
/** Returns the directory open or create. */
void *vault_directory_open_or_create(const char *root, size_t root_len, const uint8_t *password, size_t password_len);
/** Returns the directory root. */
RevaultBuffer vault_directory_root(const void *handle);
/** Returns the directory structure version. */
uint32_t vault_directory_structure_version(const void *handle);
/** Returns the directory list private keys. */
RevaultBuffer vault_directory_list_private_keys(const void *handle);
/** Returns the directory list private key names. */
RevaultBuffer vault_directory_list_private_key_names(const void *handle);
/** Returns the directory list contact names. */
RevaultBuffer vault_directory_list_contact_names(const void *handle);
/** Returns the directory list form aliases. */
RevaultBuffer vault_directory_list_form_aliases(const void *handle);
/** Returns the directory private key exists. */
bool vault_directory_private_key_exists(const void *handle, const char *name, size_t name_len);
/** Returns the directory delete private key. */
bool vault_directory_delete_private_key(const void *handle, const char *name, size_t name_len);
/** Returns the directory store private key. */
bool vault_directory_store_private_key(const void *handle, const char *name, size_t name_len, const void *key);
/** Returns the directory load private key. */
void *vault_directory_load_private_key(const void *handle, const char *name, size_t name_len);
/** Returns the directory load private key generation. */
void *vault_directory_load_private_key_generation(const void *handle, const char *name, size_t name_len, uint16_t index);
/** Returns the directory store contact. */
bool vault_directory_store_contact(const void *handle, const char *name, size_t name_len, const void *key);
/** Returns the directory load contact. */
void *vault_directory_load_contact(const void *handle, const char *name, size_t name_len);
/** Returns the directory contact exists. */
bool vault_directory_contact_exists(const void *handle, const char *name, size_t name_len);
/** Returns the directory delete contact. */
bool vault_directory_delete_contact(const void *handle, const char *name, size_t name_len);
/** Returns the directory list contacts. */
RevaultBuffer vault_directory_list_contacts(const void *handle);
/** Returns the directory store profile email. */
bool vault_directory_store_profile_email(const void *handle, const char *name, size_t name_len, const char *email, size_t email_len);
/** Returns the directory profile email. */
RevaultBuffer vault_directory_profile_email(const void *handle, const char *name, size_t name_len);
/** Returns the directory store backup. */
bool vault_directory_store_backup(const void *handle, const uint8_t *id, size_t id_len, const uint8_t *bytes, size_t len);
/** Returns the directory load backup. */
RevaultBuffer vault_directory_load_backup(const void *handle, const uint8_t *id, size_t id_len);
/** Returns the directory backup count. */
uint64_t vault_directory_backup_count(const void *handle);
/** Returns the directory restore private key. */
bool vault_directory_restore_private_key(const void *handle, const char *name, size_t name_len, const void *key, const void *signing_key, bool overwrite);
/** Returns the directory load owner signing key. */
void *vault_directory_load_owner_signing_key(const void *handle, const char *name, size_t name_len);
/** Returns the directory load owner signing key generation. */
void *vault_directory_load_owner_signing_key_generation(const void *handle, const char *name, size_t name_len, uint16_t index);
/** Returns the directory store contact signing key. */
bool vault_directory_store_contact_signing_key(const void *handle, const char *name, size_t name_len, const void *key);
/** Returns the directory load contact signing key. */
void *vault_directory_load_contact_signing_key(const void *handle, const char *name, size_t name_len);
/** Returns the directory list profile generations. */
RevaultBuffer vault_directory_list_profile_generations(const void *handle, const char *name, size_t name_len);
/** Returns the directory rotate private key. */
RevaultBuffer vault_directory_rotate_private_key(const void *handle, const char *name, size_t name_len);
/** Returns the directory remember lockbox. */
bool vault_directory_remember_lockbox(const void *handle, const uint8_t *id, size_t id_len, const char *path, size_t path_len);
/** Returns the directory list known lockboxes. */
RevaultBuffer vault_directory_list_known_lockboxes(const void *handle);
/** Returns the directory forget lockbox. */
bool vault_directory_forget_lockbox(const void *handle, const char *path, size_t path_len);
/** Returns the directory remember access slot label. */
bool vault_directory_remember_access_slot_label(const void *handle, const uint8_t *id, size_t id_len, uint64_t slot_id, const char *name, size_t name_len);
/** Returns the directory list access slot labels. */
RevaultBuffer vault_directory_list_access_slot_labels(const void *handle, const uint8_t *id, size_t id_len);
/** Returns the directory find access slot labels. */
RevaultBuffer vault_directory_find_access_slot_labels(const void *handle, const uint8_t *id, size_t id_len, const char *name, size_t name_len);
/** Returns the directory forget access slot label. */
bool vault_directory_forget_access_slot_label(const void *handle, const uint8_t *id, size_t id_len, uint64_t slot_id);
/** Returns the directory define form. */
RevaultBuffer vault_directory_define_form(const void *handle, const char *alias, size_t alias_len, const char *name, size_t name_len, const char *description, size_t description_len, const uint8_t *fields_flatbuffer, size_t fields_len);
/** Returns the directory resolve form. */
RevaultBuffer vault_directory_resolve_form(const void *handle, const char *reference, size_t reference_len);
/** Returns the directory list forms. */
RevaultBuffer vault_directory_list_forms(const void *handle);
/** Returns the directory list form revisions. */
RevaultBuffer vault_directory_list_form_revisions(const void *handle, const char *type_id, size_t type_id_len);
/** Returns the directory seed forms. */
size_t vault_directory_seed_forms(const void *handle);
/** Returns the directory remember password. */
bool vault_directory_remember_password(const void *handle, const uint8_t *id, size_t id_len, const uint8_t *password, size_t password_len);
/** Returns the directory remembered password. */
RevaultBuffer vault_directory_remembered_password(const void *handle, const uint8_t *id, size_t id_len);
/** Returns the backup default. */
RevaultBuffer vault_backup_default(const char *path, size_t path_len, bool overwrite);
/** Returns the restore default. */
RevaultBuffer vault_restore_default(const char *path, size_t path_len, bool overwrite);
/** Returns the directory free. */
void vault_directory_free(void *handle);
/** Returns only open. */
void *vault_read_only_open(const char *root, size_t root_len, const uint8_t *password, size_t password_len);
/** Returns only open default. */
void *vault_read_only_open_default(const uint8_t *password, size_t password_len);
/** Returns only list profile names. */
RevaultBuffer vault_read_only_list_profile_names(const void *handle);
/** Returns only list contact names. */
RevaultBuffer vault_read_only_list_contact_names(const void *handle);
/** Returns only list form aliases. */
RevaultBuffer vault_read_only_list_form_aliases(const void *handle);
/** Returns only list known lockboxes. */
RevaultBuffer vault_read_only_list_known_lockboxes(const void *handle);
/** Returns only free. */
void vault_read_only_free(void *handle);
/** Returns the agent serve. */
bool vault_agent_serve(void);
/** Returns the agent verify transport. */
bool vault_agent_verify_transport(void);
/** Returns the agent get. */
RevaultBuffer vault_agent_get(const uint8_t *id, size_t id_len);
/** Returns the agent put. */
bool vault_agent_put(const uint8_t *id, size_t id_len, const uint8_t *key, size_t key_len);
/** Returns the agent forget. */
bool vault_agent_forget(const uint8_t *id, size_t id_len);
/** Returns the agent stop. */
bool vault_agent_stop(void);
/** Returns the agent start. */
bool vault_agent_start(void);
/** Returns the agent list. */
RevaultBuffer vault_agent_list(void);
/** Returns the agent sleep support. */
RevaultBuffer vault_agent_sleep_support(void);
/** Returns the platform status. */
RevaultBuffer vault_platform_status(void);
/** Returns the platform set scope. */
bool vault_platform_set_scope(const char *scope, size_t len);
/** Returns the platform forget password. */
bool vault_platform_forget_password(void);
/** Returns the platform put password. */
bool vault_platform_put_password(const uint8_t *password, size_t len);
/** Returns the platform enable. */
bool vault_platform_enable(void);
/** Returns the platform disable. */
bool vault_platform_disable(void);
/** Returns the platform disabled. */
bool vault_platform_disabled(void);
/** Returns the platform get password. */
RevaultBuffer vault_platform_get_password(void);
/** Returns the default directory. */
RevaultBuffer vault_default_directory(void);
/** Returns the default path. */
RevaultBuffer vault_default_path(void);
/** Returns the agent log path. */
RevaultBuffer vault_agent_log_path(void);
/** Returns the agent log destination. */
RevaultBuffer vault_agent_log_destination(void);
/** Returns the agent get vault unlock key. */
RevaultBuffer vault_agent_get_vault_unlock_key(const char *vault_id, size_t vault_id_len);
/** Returns the agent put vault unlock key. */
bool vault_agent_put_vault_unlock_key(const char *vault_id, size_t vault_id_len, const uint8_t *key, size_t key_len, uint64_t ttl_seconds);
/** Returns the agent forget vault unlock key. */
bool vault_agent_forget_vault_unlock_key(const char *vault_id, size_t vault_id_len);
/** Returns the agent get owner signing key. */
void *vault_agent_get_owner_signing_key(const char *vault_id, size_t vault_len, const char *profile, size_t profile_len);
/** Returns the agent put owner signing key. */
bool vault_agent_put_owner_signing_key(const char *vault_id, size_t vault_len, const char *profile, size_t profile_len, const void *key, uint64_t ttl_seconds);
/** Returns the agent forget owner signing key. */
bool vault_agent_forget_owner_signing_key(const char *vault_id, size_t vault_len, const char *profile, size_t profile_len);
/** Returns the agent begin activity. */
void *vault_agent_begin_activity(const char *kind, size_t len);
/** Returns the agent end activity. */
void vault_agent_end_activity(void *handle);
/** Returns the local. */
void *vault_local(void);
/** Creates lockbox password. */
void *vault_create_lockbox_password(const void *vault, const char *path, size_t path_len, const uint8_t *password, size_t password_len);
/** Opens lockbox password. */
void *vault_open_lockbox_password(const void *vault, const char *path, size_t path_len, const uint8_t *password, size_t password_len);
/** Creates lockbox content key. */
void *vault_create_lockbox_content_key(const void *vault, const char *path, size_t path_len, const uint8_t *content_key, size_t key_len, const void *signing_key);
/** Creates lockbox contact. */
void *vault_create_lockbox_contact(const void *vault, const char *path, size_t path_len, const void *contact, const char *name, size_t name_len, const void *signing_key);
/** Opens lockbox content key. */
void *vault_open_lockbox_content_key(const void *vault, const char *path, size_t path_len, const uint8_t *content_key, size_t key_len, const void *signing_key);
/** Stores lockbox password. */
bool vault_cache_lockbox_password(const void *vault, const char *path, size_t path_len, const uint8_t *password, size_t password_len, uint64_t ttl_seconds);
/** Releases the native resources held by lockbox. */
bool vault_close_lockbox(const void *vault, const char *path, size_t path_len);
/** Releases the native resources held by all. */
bool vault_close_all(const void *vault);
/** Releases the native resources held by this object. */
void vault_free(void *vault);
#ifdef __cplusplus
}
#endif
#endif
