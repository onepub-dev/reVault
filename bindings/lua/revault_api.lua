--- Encrypt files, variables, and typed form records in portable reVault
-- lockboxes, and manage keys and local vault metadata.
--
-- Create a `Vault` to access the API. Call `free` on values that retain sensitive state, and
-- use callback-scoped secret accessors to avoid retaining plaintext.
--
-- See the [repository README](https://github.com/onepub-dev/reVault#readme)
-- for installation, security guidance, and complete examples.
-- @module revault_api
-- @usage
-- local revault = require('revault_api')
-- local vault = revault.Revault.new()
-- local lockbox = vault:lockbox_create(string.rep('\0', 32))
-- lockbox:set_variable('environment', 'production')
-- local bytes = lockbox:to_bytes()
-- lockbox:free()
local ffi = require('ffi')
local flatbuffers = require('revault_flatbuffers')

ffi.cdef[[
typedef struct { uint8_t *ptr; size_t len; } RevaultBuffer;
uint32_t api_abi_version(void);
const char * buffer_last_error(void);
RevaultBuffer buffer_last_error_details(void);
void buffer_free(RevaultBuffer value);
bool secret_len(const void * handle, size_t * out_len);
bool secret_copy(const void * handle, uint8_t * destination, size_t destination_len);
void secret_free(void * handle);
uint16_t lockbox_format_version(void);
uint16_t lockbox_probe_format_version(const uint8_t * bytes, size_t len);
void * lockbox_create(const uint8_t * key, size_t key_len);
void * lockbox_create_with_options(const uint8_t * key, size_t key_len, const char * cache_mode, size_t cache_len, uint64_t cache_bytes, const char * workload, size_t workload_len, const char * worker, size_t worker_len, size_t jobs);
void * lockbox_create_password(const uint8_t * password, size_t len);
void * lockbox_create_password_with_signing_key(const uint8_t * password, size_t len, const void * signing_key);
void * lockbox_create_contact(const void * contact);
void * lockbox_create_contact_with_signing_key(const void * contact, const void * signing_key);
void * lockbox_create_with_signing_key(const uint8_t * content_key, size_t key_len, const void * signing_key);
void * lockbox_open(const uint8_t * archive, size_t archive_len, const uint8_t * key, size_t key_len);
void * lockbox_open_with_options(const uint8_t * archive, size_t archive_len, const uint8_t * key, size_t key_len, const char * cache_mode, size_t cache_len, uint64_t cache_bytes, const char * workload, size_t workload_len, const char * worker, size_t worker_len, size_t jobs);
void * lockbox_open_password(const uint8_t * archive, size_t archive_len, const uint8_t * password, size_t password_len);
void * lockbox_open_contact(const uint8_t * archive, size_t archive_len, const void * contact);
bool lockbox_add_file(void * handle, const char * path, size_t path_len, const uint8_t * data, size_t data_len, bool replace);
bool lockbox_add_file_with_permissions(void * handle, const char * path, size_t path_len, const uint8_t * data, size_t data_len, uint32_t permissions, bool replace);
RevaultBuffer lockbox_get_file(const void * handle, const char * path, size_t path_len);
bool lockbox_extract_file(const void * handle, const char * source, size_t source_len, const char * destination, size_t destination_len, bool replace);
bool lockbox_extract_directory(const void * handle, const char * destination, size_t destination_len, uint64_t max_file_bytes, uint64_t max_total_bytes, size_t max_files, bool restore_symlinks, bool restore_permissions, bool overwrite);
RevaultBuffer lockbox_stream_content(const void * handle, bool physical);
RevaultBuffer lockbox_cache_stats(const void * handle);
RevaultBuffer lockbox_import_stats(const void * handle);
bool lockbox_reset_import_stats(const void * handle);
RevaultBuffer lockbox_inspect_file(const char * path, size_t path_len);
RevaultBuffer lockbox_page_inspection(const void * handle);
RevaultBuffer lockbox_recovery_report(const void * handle);
RevaultBuffer lockbox_recovery_report_render(const void * handle, bool verbose, size_t max_entries);
RevaultBuffer lockbox_recovery_scan_path(const char * path, size_t path_len, const uint8_t * key, size_t key_len);
uint64_t lockbox_storage_len(const void * handle);
bool lockbox_set_workload_profile(void * handle, const char * profile, size_t profile_len);
bool lockbox_set_worker_policy(void * handle, const char * mode, size_t mode_len, size_t jobs);
RevaultBuffer lockbox_runtime_options(const void * handle);
bool lockbox_commit(void * handle);
bool lockbox_create_dir(void * handle, const char * path, size_t path_len, bool create_parents);
bool lockbox_delete(void * handle, const char * path, size_t path_len);
bool lockbox_remove_dir(void * handle, const char * path, size_t path_len, bool recursive);
bool lockbox_create_parent_dirs(void * handle, const char * path, size_t path_len);
bool lockbox_rename(void * handle, const char * from, size_t from_len, const char * to, size_t to_len);
RevaultBuffer lockbox_list(const void * handle, const char * path, size_t path_len, bool recursive);
RevaultBuffer lockbox_list_with_options(const void * handle, const char * path, size_t path_len, const char * glob, size_t glob_len, bool recursive, bool include_files, bool include_symlinks, bool include_directories, size_t limit);
RevaultBuffer lockbox_stat(const void * handle, const char * path, size_t path_len);
bool lockbox_set_variable(void * handle, const char * name, size_t name_len, const char * value, size_t value_len);
bool lockbox_set_secret_variable(void * handle, const char * name, size_t name_len, const uint8_t * value, size_t value_len);
RevaultBuffer lockbox_get_variable(const void * handle, const char * name, size_t name_len);
bool lockbox_get_secret_variable(const void * handle, const char * name, size_t name_len, void ** output);
bool lockbox_delete_variable(void * handle, const char * name, size_t name_len);
bool lockbox_move_variables(void * handle, const uint8_t * moves_flatbuffer, size_t moves_len);
RevaultBuffer lockbox_list_variables(const void * handle);
RevaultBuffer lockbox_variable_sensitivity(const void * handle, const char * name, size_t name_len);
bool lockbox_add_symlink(void * handle, const char * path, size_t path_len, const char * target, size_t target_len, bool replace);
RevaultBuffer lockbox_get_symlink_target(const void * handle, const char * path, size_t path_len);
RevaultBuffer lockbox_id(const void * handle);
bool lockbox_exists(const void * handle, const char * path, size_t path_len);
bool lockbox_is_dir(const void * handle, const char * path, size_t path_len);
uint32_t lockbox_permissions(const void * handle, const char * path, size_t path_len);
bool lockbox_set_permissions(void * handle, const char * path, size_t path_len, uint32_t permissions);
RevaultBuffer lockbox_read_range(const void * handle, const char * path, size_t path_len, uint64_t offset, uint64_t len);
RevaultBuffer lockbox_recovery_scan(const uint8_t * bytes, size_t len, const uint8_t * key, size_t key_len);
void * lockbox_recovery_salvage(const uint8_t * bytes, size_t len, const uint8_t * key, size_t key_len, const void * signing_key);
uint64_t lockbox_add_password(void * handle, const uint8_t * password, size_t len);
uint64_t lockbox_add_contact(void * handle, const void * contact, const char * name, size_t name_len);
bool lockbox_delete_key(void * handle, uint64_t id);
RevaultBuffer lockbox_list_key_slots(const void * handle);
bool lockbox_set_owner_signing_key(void * handle, const void * key);
RevaultBuffer lockbox_owner_inspection(const void * handle);
RevaultBuffer lockbox_define_form(void * handle, const char * alias, size_t alias_len, const char * name, size_t name_len, const char * description, size_t description_len, const uint8_t * fields_flatbuffer, size_t fields_len);
RevaultBuffer lockbox_list_form_definitions(const void * handle);
RevaultBuffer lockbox_resolve_form(const void * handle, const char * reference, size_t reference_len);
RevaultBuffer lockbox_list_form_revisions(const void * handle, const char * type_id, size_t type_id_len);
RevaultBuffer lockbox_create_form_record(void * handle, const char * path, size_t path_len, const char * type_reference, size_t type_len, const char * name, size_t name_len);
bool lockbox_set_form_field(void * handle, const char * path, size_t path_len, const char * field, size_t field_len, const char * value, size_t value_len);
bool lockbox_set_secret_form_field(void * handle, const char * path, size_t path_len, const char * field, size_t field_len, const uint8_t * value, size_t value_len);
RevaultBuffer lockbox_list_form_records(const void * handle);
RevaultBuffer lockbox_get_form_record(const void * handle, const char * path, size_t path_len);
bool lockbox_delete_form_record(void * handle, const char * path, size_t path_len);
bool lockbox_move_form_records(void * handle, const uint8_t * moves_flatbuffer, size_t moves_len);
RevaultBuffer lockbox_get_form_field(const void * handle, const char * path, size_t path_len, const char * field, size_t field_len);
bool lockbox_get_secret_form_field(const void * handle, const char * path, size_t path_len, const char * field, size_t field_len, void ** output);
RevaultBuffer lockbox_to_bytes(const void * handle);
void lockbox_free(void * handle);
bool vault_is_running(void);
bool vault_forget_all(void);
void * key_contact_generate(void);
void * key_contact_from_private(const uint8_t * bytes, size_t len);
RevaultBuffer key_contact_public(const void * handle);
RevaultBuffer key_contact_private(const void * handle);
void * key_contact_public_from_bytes(const uint8_t * bytes, size_t len);
void key_contact_public_free(void * handle);
void key_contact_free(void * handle);
void * key_contact_encrypt(const void * contact, const uint8_t * content_key, size_t key_len);
RevaultBuffer key_contact_decrypt(const void * contact, const void * wrapped);
RevaultBuffer key_contact_wrapped_public(const void * wrapped);
RevaultBuffer key_contact_wrapped_ciphertext(const void * wrapped);
RevaultBuffer key_contact_wrapped_encrypted(const void * wrapped);
void key_contact_wrapped_free(void * handle);
void * key_signing_generate(void);
void * key_signing_from_private(const uint8_t * bytes, size_t len);
RevaultBuffer key_signing_public(const void * handle);
RevaultBuffer key_signing_private(const void * handle);
void * key_signing_public_from_bytes(const uint8_t * bytes, size_t len);
void key_signing_public_free(void * handle);
void key_signing_free(void * handle);
RevaultBuffer vault_key_export_private(const void * key, const char * format, size_t format_len);
RevaultBuffer vault_key_export_public(const void * key, const char * format, size_t format_len);
void * vault_key_import_private(const uint8_t * bytes, size_t len);
void * vault_key_import_public(const uint8_t * bytes, size_t len);
RevaultBuffer vault_key_fingerprint(const void * key);
RevaultBuffer vault_key_format_hex(const uint8_t * bytes, size_t len);
RevaultBuffer vault_key_decode_hex(const char * text, size_t len);
RevaultBuffer vault_key_format_crockford(const uint8_t * bytes, size_t len);
RevaultBuffer vault_key_format_crockford_reading(const char * code, size_t len);
RevaultBuffer vault_key_decode_crockford(const char * code, size_t len);
RevaultBuffer vault_key_hex_encode(const uint8_t * bytes, size_t len);
RevaultBuffer vault_key_hex_decode(const char * text, size_t len);
void * vault_directory_open(const char * root, size_t root_len, const uint8_t * password, size_t password_len);
uint32_t vault_structure_version_current(void);
uint32_t vault_directory_probe_structure_version(const char * root, size_t root_len, const uint8_t * password, size_t password_len);
void * vault_directory_open_or_create_default(const uint8_t * password, size_t password_len);
void * vault_directory_replace_default(const uint8_t * password, size_t password_len);
bool vault_directory_change_password(const char * root, size_t root_len, const uint8_t * old_password, size_t old_len, const uint8_t * new_password, size_t new_len);
bool vault_directory_change_default_password(const uint8_t * old_password, size_t old_len, const uint8_t * new_password, size_t new_len);
void * vault_directory_replace(const char * root, size_t root_len, const uint8_t * password, size_t password_len);
void * vault_directory_open_or_create(const char * root, size_t root_len, const uint8_t * password, size_t password_len);
RevaultBuffer vault_directory_root(const void * handle);
uint32_t vault_directory_structure_version(const void * handle);
RevaultBuffer vault_directory_list_private_keys(const void * handle);
RevaultBuffer vault_directory_list_private_key_names(const void * handle);
RevaultBuffer vault_directory_list_contact_names(const void * handle);
RevaultBuffer vault_directory_list_form_aliases(const void * handle);
bool vault_directory_private_key_exists(const void * handle, const char * name, size_t name_len);
bool vault_directory_delete_private_key(const void * handle, const char * name, size_t name_len);
bool vault_directory_store_private_key(const void * handle, const char * name, size_t name_len, const void * key);
void * vault_directory_load_private_key(const void * handle, const char * name, size_t name_len);
void * vault_directory_load_private_key_generation(const void * handle, const char * name, size_t name_len, uint16_t index);
bool vault_directory_store_contact(const void * handle, const char * name, size_t name_len, const void * key);
void * vault_directory_load_contact(const void * handle, const char * name, size_t name_len);
bool vault_directory_contact_exists(const void * handle, const char * name, size_t name_len);
bool vault_directory_delete_contact(const void * handle, const char * name, size_t name_len);
RevaultBuffer vault_directory_list_contacts(const void * handle);
bool vault_directory_store_profile_email(const void * handle, const char * name, size_t name_len, const char * email, size_t email_len);
RevaultBuffer vault_directory_profile_email(const void * handle, const char * name, size_t name_len);
bool vault_directory_store_backup(const void * handle, const uint8_t * id, size_t id_len, const uint8_t * bytes, size_t len);
RevaultBuffer vault_directory_load_backup(const void * handle, const uint8_t * id, size_t id_len);
uint64_t vault_directory_backup_count(const void * handle);
bool vault_directory_restore_private_key(const void * handle, const char * name, size_t name_len, const void * key, const void * signing_key, bool overwrite);
void * vault_directory_load_owner_signing_key(const void * handle, const char * name, size_t name_len);
void * vault_directory_load_owner_signing_key_generation(const void * handle, const char * name, size_t name_len, uint16_t index);
bool vault_directory_store_contact_signing_key(const void * handle, const char * name, size_t name_len, const void * key);
void * vault_directory_load_contact_signing_key(const void * handle, const char * name, size_t name_len);
RevaultBuffer vault_directory_list_profile_generations(const void * handle, const char * name, size_t name_len);
RevaultBuffer vault_directory_rotate_private_key(const void * handle, const char * name, size_t name_len);
bool vault_directory_remember_lockbox(const void * handle, const uint8_t * id, size_t id_len, const char * path, size_t path_len);
RevaultBuffer vault_directory_list_known_lockboxes(const void * handle);
bool vault_directory_forget_lockbox(const void * handle, const char * path, size_t path_len);
bool vault_directory_remember_access_slot_label(const void * handle, const uint8_t * id, size_t id_len, uint64_t slot_id, const char * name, size_t name_len);
RevaultBuffer vault_directory_list_access_slot_labels(const void * handle, const uint8_t * id, size_t id_len);
RevaultBuffer vault_directory_find_access_slot_labels(const void * handle, const uint8_t * id, size_t id_len, const char * name, size_t name_len);
bool vault_directory_forget_access_slot_label(const void * handle, const uint8_t * id, size_t id_len, uint64_t slot_id);
RevaultBuffer vault_directory_define_form(const void * handle, const char * alias, size_t alias_len, const char * name, size_t name_len, const char * description, size_t description_len, const uint8_t * fields_flatbuffer, size_t fields_len);
RevaultBuffer vault_directory_resolve_form(const void * handle, const char * reference, size_t reference_len);
RevaultBuffer vault_directory_list_forms(const void * handle);
RevaultBuffer vault_directory_list_form_revisions(const void * handle, const char * type_id, size_t type_id_len);
size_t vault_directory_seed_forms(const void * handle);
bool vault_directory_remember_password(const void * handle, const uint8_t * id, size_t id_len, const uint8_t * password, size_t password_len);
RevaultBuffer vault_directory_remembered_password(const void * handle, const uint8_t * id, size_t id_len);
RevaultBuffer vault_backup_default(const char * path, size_t path_len, bool overwrite);
RevaultBuffer vault_restore_default(const char * path, size_t path_len, bool overwrite);
void vault_directory_free(void * handle);
void * vault_read_only_open(const char * root, size_t root_len, const uint8_t * password, size_t password_len);
void * vault_read_only_open_default(const uint8_t * password, size_t password_len);
RevaultBuffer vault_read_only_list_profile_names(const void * handle);
RevaultBuffer vault_read_only_list_contact_names(const void * handle);
RevaultBuffer vault_read_only_list_form_aliases(const void * handle);
RevaultBuffer vault_read_only_list_known_lockboxes(const void * handle);
void vault_read_only_free(void * handle);
bool vault_agent_serve(void);
bool vault_agent_verify_transport(void);
RevaultBuffer vault_agent_get(const uint8_t * id, size_t id_len);
bool vault_agent_put(const uint8_t * id, size_t id_len, const uint8_t * key, size_t key_len);
bool vault_agent_forget(const uint8_t * id, size_t id_len);
bool vault_agent_stop(void);
bool vault_agent_start(void);
RevaultBuffer vault_agent_list(void);
RevaultBuffer vault_agent_sleep_support(void);
RevaultBuffer vault_platform_status(void);
bool vault_platform_set_scope(const char * scope, size_t len);
bool vault_platform_forget_password(void);
bool vault_platform_put_password(const uint8_t * password, size_t len);
bool vault_platform_enable(void);
bool vault_platform_disable(void);
bool vault_platform_disabled(void);
RevaultBuffer vault_platform_get_password(void);
RevaultBuffer vault_default_directory(void);
RevaultBuffer vault_default_path(void);
RevaultBuffer vault_agent_log_path(void);
RevaultBuffer vault_agent_log_destination(void);
RevaultBuffer vault_agent_get_vault_unlock_key(const char * vault_id, size_t vault_id_len);
bool vault_agent_put_vault_unlock_key(const char * vault_id, size_t vault_id_len, const uint8_t * key, size_t key_len, uint64_t ttl_seconds);
bool vault_agent_forget_vault_unlock_key(const char * vault_id, size_t vault_id_len);
void * vault_agent_get_owner_signing_key(const char * vault_id, size_t vault_len, const char * profile, size_t profile_len);
bool vault_agent_put_owner_signing_key(const char * vault_id, size_t vault_len, const char * profile, size_t profile_len, const void * key, uint64_t ttl_seconds);
bool vault_agent_forget_owner_signing_key(const char * vault_id, size_t vault_len, const char * profile, size_t profile_len);
void * vault_agent_begin_activity(const char * kind, size_t len);
void vault_agent_end_activity(void * handle);
void * vault_local(void);
void * vault_create_lockbox_password(const void * vault, const char * path, size_t path_len, const uint8_t * password, size_t password_len);
void * vault_open_lockbox_password(const void * vault, const char * path, size_t path_len, const uint8_t * password, size_t password_len);
void * vault_create_lockbox_content_key(const void * vault, const char * path, size_t path_len, const uint8_t * content_key, size_t key_len, const void * signing_key);
void * vault_create_lockbox_contact(const void * vault, const char * path, size_t path_len, const void * contact, const char * name, size_t name_len, const void * signing_key);
void * vault_open_lockbox_content_key(const void * vault, const char * path, size_t path_len, const uint8_t * content_key, size_t key_len, const void * signing_key);
bool vault_cache_lockbox_password(const void * vault, const char * path, size_t path_len, const uint8_t * password, size_t password_len, uint64_t ttl_seconds);
bool vault_close_lockbox(const void * vault, const char * path, size_t path_len);
bool vault_close_all(const void * vault);
void vault_free(void * vault);
]]

local function native_library()
  local cpu = ({ x64 = 'x86_64', arm64 = 'aarch64' })[jit.arch]
  if not cpu then error('unsupported reVault architecture: ' .. jit.arch) end
  local target, library
  if jit.os == 'Linux' then target, library = 'linux-' .. cpu .. '-gnu', 'librevault_api.so'
  elseif jit.os == 'OSX' then target, library = 'macos-' .. cpu, 'librevault_api.dylib'
  elseif jit.os == 'Windows' then target, library = 'windows-' .. cpu .. '-msvc', 'revault_api.dll'
  else error('unsupported reVault operating system: ' .. jit.os) end
  for pattern in package.cpath:gmatch('[^;]+') do
    local directory = pattern:match('^(.*[/\\])')
    if directory then
      local installed = directory .. library
      local file = io.open(installed, 'rb')
      if file then file:close(); return installed end
    end
  end
  local source = debug.getinfo(1, 'S').source
  if source:sub(1, 1) == '@' then
    local directory = source:sub(2):match('^(.*[/\\])') or './'
    local bundled = directory .. 'native/' .. target .. '/' .. library
    local file = io.open(bundled, 'rb')
    if file then file:close(); return bundled end
  end
  error('revault-api native carrier is missing for ' .. target .. '; install the matching platform rock')
end
local loaded_native
local function load_native(explicit_path)
  if explicit_path == '' then error('native library path must not be empty') end
  local inherited = os.getenv('REVAULT_LIBRARY')
  local selected = explicit_path or
    (inherited ~= nil and inherited ~= '' and inherited or nil) or
    native_library()
  local library = ffi.load(selected)
  if tonumber(library.api_abi_version()) ~= 3 then
    error('revault-api native ABI mismatch; expected 3')
  end
  loaded_native = library
end
local native = setmetatable({}, {
  __index = function(_, symbol)
    if loaded_native == nil then load_native(nil) end
    return loaded_native[symbol]
  end,
})
local function last_error()
  local value = native.buffer_last_error()
  return value == nil and 'native reVault operation failed' or ffi.string(value)
end

local Models = {}
local child_fields = {
  AccessSlotLabelList = { { "values", "AccessSlotLabel", true } },
  AgentEntryList = { { "values", "AgentEntry", true } },
  ContactList = { { "values", "Contact", true } },
  FileInspection = { { "key_slots", "KeySlot", true } },
  FormDefinition = { { "fields", "FormField", true } },
  FormDefinitionList = { { "values", "FormDefinition", true } },
  FormFieldList = { { "values", "FormField", true } },
  FormRecord = { { "values", "FormValue", true } },
  FormRecordList = { { "values", "FormRecord", true } },
  KeySlotList = { { "values", "KeySlot", true } },
  KnownLockboxList = { { "values", "KnownLockbox", true } },
  LockboxEntryList = { { "entries", "LockboxEntry", true } },
  OptionalFormRecord = { { "value", "FormRecord", false } },
  OptionalFormValue = { { "value", "FormValue", false } },
  OptionalLockboxEntry = { { "value", "LockboxEntry", false } },
  PageInspection = { { "objects", "PageObject", true } },
  PageInspectionList = { { "values", "PageInspection", true } },
  PathMoveList = { { "values", "PathMove", true } },
  ProfileHistory = { { "generations", "ProfileGeneration", true } },
  ProfileHistoryList = { { "values", "ProfileHistory", true } },
  RecoveryReport = { { "intact_files", "LockboxEntry", true } },
  StreamChunkList = { { "values", "StreamChunk", true } },
  VariableList = { { "values", "Variable", true } },
}
local function wrap(name, value)
  if value == nil then return nil end
  for _, spec in ipairs(child_fields[name] or {}) do
    local field, child, repeated = spec[1], spec[2], spec[3]
    if repeated then
      for index, item in ipairs(value[field] or {}) do value[field][index] = wrap(child, item) end
    elseif value[field] ~= nil then value[field] = wrap(child, value[field]) end
  end
  return setmetatable(value, Models[name])
end

local function model(name)
  local class = { __name = name }
  class.__index = class
  function class.new(fields) return wrap(name, fields or {}) end
  function class:encode()
    if name == 'PathMoveList' then return flatbuffers.encode_path_moves(self.values or {}) end
    if name == 'FormFieldList' then return flatbuffers.encode_form_fields(self.values or {}) end
    error('encoding ' .. name .. ' is not a supported reVault API input')
  end
  function class.decode(bytes) return wrap(name, flatbuffers.decode(name, bytes)) end
  Models[name] = class
end
--- A local human-readable label attached to one lockbox access slot.
-- @type AccessSlotLabel
model("AccessSlotLabel")
--- Ordered AccessSlotLabel values returned by the corresponding list operation.
-- @type AccessSlotLabelList
model("AccessSlotLabelList")
--- A lockbox key currently held by the local Session Agent, identified by lockbox and path.
-- @type AgentEntry
model("AgentEntry")
--- Ordered AgentEntry values returned by the corresponding list operation.
-- @type AgentEntryList
model("AgentEntryList")
--- Ordered Byte values returned by the corresponding list operation.
-- @type ByteList
model("ByteList")
--- Current capacity, occupancy, hit, and miss counters for an open lockbox cache.
-- @type CacheStats
model("CacheStats")
--- A named recipient public key stored in the local vault address book.
-- @type Contact
model("Contact")
--- Ordered Contact values returned by the corresponding list operation.
-- @type ContactList
model("ContactList")
--- Structured category, version, guidance, and artifact context for the most recent native failure.
-- @type ErrorDetails
model("ErrorDetails")
--- Header, owner-signature, and key-slot information read from a lockbox file without opening its contents.
-- @type FileInspection
model("FileInspection")
--- A versioned form schema used to validate and label structured records in a lockbox.
-- @type FormDefinition
model("FormDefinition")
--- Ordered FormDefinition values returned by the corresponding list operation.
-- @type FormDefinitionList
model("FormDefinitionList")
--- One named input in a reusable form definition, including its display label and sensitivity kind.
-- @type FormField
model("FormField")
--- Ordered field definitions supplied when defining a form.
-- @type FormFieldList
model("FormFieldList")
--- A named structured record stored at a lockbox path and tied to a form-definition revision.
-- @type FormRecord
model("FormRecord")
--- Ordered FormRecord values returned by the corresponding list operation.
-- @type FormRecordList
model("FormRecordList")
--- The current value and sensitivity metadata for one field in a stored form record.
-- @type FormValue
model("FormValue")
--- Time spent reading host files and preparing encrypted pages during the latest import work.
-- @type ImportStats
model("ImportStats")
--- One password or contact credential that can unlock a lockbox content key.
-- @type KeySlot
model("KeySlot")
--- Ordered KeySlot values returned by the corresponding list operation.
-- @type KeySlotList
model("KeySlotList")
--- A lockbox identifier and host path remembered by the local vault for later discovery.
-- @type KnownLockbox
model("KnownLockbox")
--- Ordered KnownLockbox values returned by the corresponding list operation.
-- @type KnownLockboxList
model("KnownLockboxList")
--- Metadata for one file, directory, or symbolic link stored at a lockbox path.
-- @type LockboxEntry
model("LockboxEntry")
--- Ordered lockbox entries selected by a list operation.
-- @type LockboxEntryList
model("LockboxEntryList")
--- The form record found at a lockbox path, or no value when absent.
-- @type OptionalFormRecord
model("OptionalFormRecord")
--- The requested non-secret form value, or no value when the field is absent.
-- @type OptionalFormValue
model("OptionalFormValue")
--- The metadata found for a lockbox path, or no value when the path is absent.
-- @type OptionalLockboxEntry
model("OptionalLockboxEntry")
--- A text lookup that distinguishes an absent value from an empty string.
-- @type OptionalString
model("OptionalString")
--- Whether a lockbox is owner-signed and, when available, the signing-key fingerprint.
-- @type OwnerInspection
model("OwnerInspection")
--- Layout and utilization details for one encrypted page in a lockbox archive.
-- @type PageInspection
model("PageInspection")
--- Ordered PageInspection values returned by the corresponding list operation.
-- @type PageInspectionList
model("PageInspectionList")
--- One logical object recorded inside an inspected encrypted lockbox page.
-- @type PageObject
model("PageObject")
--- A source and destination pair used to rename a variable or form record atomically.
-- @type PathMove
model("PathMove")
--- Atomic variable or form-record renames supplied to a move operation.
-- @type PathMoveList
model("PathMoveList")
--- Availability and configuration of the platform credential store used for the Vault passphrase.
-- @type PlatformStatus
model("PlatformStatus")
--- One active or retired generation of the contact keys belonging to a named vault profile.
-- @type ProfileGeneration
model("ProfileGeneration")
--- The active generation and rotation history for a named vault profile.
-- @type ProfileHistory
model("ProfileHistory")
--- Ordered ProfileHistory values returned by the corresponding list operation.
-- @type ProfileHistoryList
model("ProfileHistoryList")
--- The files and metadata recovered, or found damaged, while inspecting or salvaging a lockbox.
-- @type RecoveryReport
model("RecoveryReport")
--- The workload and worker policies currently applied to an open lockbox.
-- @type RuntimeOptions
model("RuntimeOptions")
--- The host capabilities used to protect cached secrets across suspend and sleep.
-- @type SleepSupport
model("SleepSupport")
--- A logical or physical byte range emitted while walking the contents of a lockbox.
-- @type StreamChunk
model("StreamChunk")
--- Ordered StreamChunk values returned by the corresponding list operation.
-- @type StreamChunkList
model("StreamChunkList")
--- Ordered names or identifiers returned by a vault list operation.
-- @type StringList
model("StringList")
--- One text result returned by the native API.
-- @type StringValue
model("StringValue")
--- The name and sensitivity classification of a variable stored in a lockbox.
-- @type Variable
model("Variable")
--- Ordered Variable values returned by the corresponding list operation.
-- @type VariableList
model("VariableList")
--- The version, size, checksum, and creation time of an exported local-vault backup.
-- @type VaultBackupManifest
model("VaultBackupManifest")

local function take(buffer)
  if buffer.ptr == nil then error(last_error(), 3) end
  local value = ffi.string(buffer.ptr, tonumber(buffer.len))
  native.buffer_free(buffer)
  return value
end

local function with_secret(getter, callback)
  local output = ffi.new('void *[1]')
  if not getter(output) then error(last_error(), 3) end
  if output[0] == nil then return nil end
  local length = ffi.new('size_t[1]')
  local ok, result
  if not native.secret_len(output[0], length) then
    native.secret_free(output[0]); error(last_error(), 3)
  end
  local bytes = ffi.new('uint8_t[?]', math.max(1, tonumber(length[0])))
  if not native.secret_copy(output[0], bytes, length[0]) then
    native.secret_free(output[0]); error(last_error(), 3)
  end
  ok, result = pcall(callback, bytes, tonumber(length[0]))
  ffi.fill(bytes, math.max(1, tonumber(length[0])), 0)
  native.secret_free(output[0])
  if not ok then error(result, 3) end
  return result
end

local Operations = {}
Operations.__index = Operations
function Operations.new() return setmetatable({}, Operations) end
function Operations:last_error_message() return last_error() end

function Operations:buffer_last_error_details()
  return Models.ErrorDetails.decode(take(native.buffer_last_error_details()))
end

function Operations:lockbox_format_version()
  return tonumber(native.lockbox_format_version())
end

function Operations:lockbox_probe_format_version(bytes)
  return tonumber(native.lockbox_probe_format_version(bytes, #bytes))
end

function Operations:lockbox_create(key)
  local value = native.lockbox_create(key, #key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
  local value = native.lockbox_create_with_options(key, #key, cache_mode, #cache_mode, cache_bytes, workload, #workload, worker, #worker, jobs)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_create_password(password)
  local value = native.lockbox_create_password(password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_create_password_with_signing_key(password, signing_key)
  local value = native.lockbox_create_password_with_signing_key(password, #password, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_create_contact(contact)
  local value = native.lockbox_create_contact(contact)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_create_contact_with_signing_key(contact, signing_key)
  local value = native.lockbox_create_contact_with_signing_key(contact, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_create_with_signing_key(content_key, signing_key)
  local value = native.lockbox_create_with_signing_key(content_key, #content_key, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_open(archive, key)
  local value = native.lockbox_open(archive, #archive, key, #key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
  local value = native.lockbox_open_with_options(archive, #archive, key, #key, cache_mode, #cache_mode, cache_bytes, workload, #workload, worker, #worker, jobs)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_open_password(archive, password)
  local value = native.lockbox_open_password(archive, #archive, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_open_contact(archive, contact)
  local value = native.lockbox_open_contact(archive, #archive, contact)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_add_file(handle, path, data, replace)
  if not native.lockbox_add_file(handle, path, #path, data, #data, replace) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_add_file_with_permissions(handle, path, data, permissions, replace)
  if not native.lockbox_add_file_with_permissions(handle, path, #path, data, #data, permissions, replace) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_get_file(handle, path)
  return take(native.lockbox_get_file(handle, path, #path))
end

function Operations:lockbox_extract_file(handle, source, destination, replace)
  if not native.lockbox_extract_file(handle, source, #source, destination, #destination, replace) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_extract_directory(handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
  if not native.lockbox_extract_directory(handle, destination, #destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_stream_content(handle, physical)
  return Models.StreamChunkList.decode(take(native.lockbox_stream_content(handle, physical)))
end

function Operations:lockbox_cache_stats(handle)
  return Models.CacheStats.decode(take(native.lockbox_cache_stats(handle)))
end

function Operations:lockbox_import_stats(handle)
  return Models.ImportStats.decode(take(native.lockbox_import_stats(handle)))
end

function Operations:lockbox_reset_import_stats(handle)
  if not native.lockbox_reset_import_stats(handle) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_inspect_file(path)
  return Models.FileInspection.decode(take(native.lockbox_inspect_file(path, #path)))
end

function Operations:lockbox_page_inspection(handle)
  return Models.PageInspectionList.decode(take(native.lockbox_page_inspection(handle)))
end

function Operations:lockbox_recovery_report(handle)
  return Models.RecoveryReport.decode(take(native.lockbox_recovery_report(handle)))
end

function Operations:lockbox_recovery_report_render(handle, verbose, max_entries)
  return take(native.lockbox_recovery_report_render(handle, verbose, max_entries))
end

function Operations:lockbox_recovery_scan_path(path, key)
  return Models.RecoveryReport.decode(take(native.lockbox_recovery_scan_path(path, #path, key, #key)))
end

function Operations:lockbox_storage_len(handle)
  return tonumber(native.lockbox_storage_len(handle))
end

function Operations:lockbox_set_workload_profile(handle, profile)
  if not native.lockbox_set_workload_profile(handle, profile, #profile) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_set_worker_policy(handle, mode, jobs)
  if not native.lockbox_set_worker_policy(handle, mode, #mode, jobs) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_runtime_options(handle)
  return Models.RuntimeOptions.decode(take(native.lockbox_runtime_options(handle)))
end

function Operations:lockbox_commit(handle)
  if not native.lockbox_commit(handle) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_create_dir(handle, path, create_parents)
  if not native.lockbox_create_dir(handle, path, #path, create_parents) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_delete(handle, path)
  if not native.lockbox_delete(handle, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_remove_dir(handle, path, recursive)
  if not native.lockbox_remove_dir(handle, path, #path, recursive) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_create_parent_dirs(handle, path)
  if not native.lockbox_create_parent_dirs(handle, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_rename(handle, from, to)
  if not native.lockbox_rename(handle, from, #from, to, #to) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_list(handle, path, recursive)
  return Models.LockboxEntryList.decode(take(native.lockbox_list(handle, path, #path, recursive)))
end

function Operations:lockbox_list_with_options(handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
  return Models.LockboxEntryList.decode(take(native.lockbox_list_with_options(handle, path, #path, glob, #glob, recursive, include_files, include_symlinks, include_directories, limit)))
end

function Operations:lockbox_stat(handle, path)
  return Models.OptionalLockboxEntry.decode(take(native.lockbox_stat(handle, path, #path)))
end

function Operations:lockbox_set_variable(handle, name, value)
  if not native.lockbox_set_variable(handle, name, #name, value, #value) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_set_secret_variable(handle, name, value)
  local bytes = ffi.new('uint8_t[?]', math.max(1, #value)); ffi.copy(bytes, value, #value)
  local ok = native.lockbox_set_secret_variable(handle, name, #name, bytes, #value)
  ffi.fill(bytes, math.max(1, #value), 0)
  if not ok then error(last_error(), 2) end
  return true
end

function Operations:lockbox_get_variable(handle, name)
  local value = Models.OptionalString.decode(take(native.lockbox_get_variable(handle, name, #name)))
  return value.present and value.value or nil
end

function Operations:lockbox_get_secret_variable(handle, name, callback)
  return with_secret(function(output) return native.lockbox_get_secret_variable(handle, name, #name, output) end, callback)
end

function Operations:lockbox_delete_variable(handle, name)
  if not native.lockbox_delete_variable(handle, name, #name) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_move_variables(handle, moves_flatbuffer)
  if not native.lockbox_move_variables(handle, moves_flatbuffer, #moves_flatbuffer) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_list_variables(handle)
  return Models.VariableList.decode(take(native.lockbox_list_variables(handle)))
end

function Operations:lockbox_variable_sensitivity(handle, name)
  return Models.OptionalString.decode(take(native.lockbox_variable_sensitivity(handle, name, #name)))
end

function Operations:lockbox_add_symlink(handle, path, target, replace)
  if not native.lockbox_add_symlink(handle, path, #path, target, #target, replace) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_get_symlink_target(handle, path)
  return take(native.lockbox_get_symlink_target(handle, path, #path))
end

function Operations:lockbox_id(handle)
  return take(native.lockbox_id(handle))
end

function Operations:lockbox_exists(handle, path)
  return native.lockbox_exists(handle, path, #path)
end

function Operations:lockbox_is_dir(handle, path)
  return native.lockbox_is_dir(handle, path, #path)
end

function Operations:lockbox_permissions(handle, path)
  return tonumber(native.lockbox_permissions(handle, path, #path))
end

function Operations:lockbox_set_permissions(handle, path, permissions)
  if not native.lockbox_set_permissions(handle, path, #path, permissions) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_read_range(handle, path, offset, len)
  return take(native.lockbox_read_range(handle, path, #path, offset, len))
end

function Operations:lockbox_recovery_scan(bytes, key)
  return Models.RecoveryReport.decode(take(native.lockbox_recovery_scan(bytes, #bytes, key, #key)))
end

function Operations:lockbox_recovery_salvage(bytes, key, signing_key)
  local value = native.lockbox_recovery_salvage(bytes, #bytes, key, #key, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:lockbox_add_password(handle, password)
  return tonumber(native.lockbox_add_password(handle, password, #password))
end

function Operations:lockbox_add_contact(handle, contact, name)
  return tonumber(native.lockbox_add_contact(handle, contact, name, #name))
end

function Operations:lockbox_delete_key(handle, id)
  if not native.lockbox_delete_key(handle, id) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_list_key_slots(handle)
  return Models.KeySlotList.decode(take(native.lockbox_list_key_slots(handle)))
end

function Operations:lockbox_set_owner_signing_key(handle, key)
  if not native.lockbox_set_owner_signing_key(handle, key) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_owner_inspection(handle)
  return Models.OwnerInspection.decode(take(native.lockbox_owner_inspection(handle)))
end

function Operations:lockbox_define_form(handle, alias, name, description, fields_flatbuffer)
  return Models.FormDefinition.decode(take(native.lockbox_define_form(handle, alias, #alias, name, #name, description, #description, fields_flatbuffer, #fields_flatbuffer)))
end

function Operations:lockbox_list_form_definitions(handle)
  return Models.FormDefinitionList.decode(take(native.lockbox_list_form_definitions(handle)))
end

function Operations:lockbox_resolve_form(handle, reference)
  return Models.FormDefinition.decode(take(native.lockbox_resolve_form(handle, reference, #reference)))
end

function Operations:lockbox_list_form_revisions(handle, type_id)
  return Models.FormDefinitionList.decode(take(native.lockbox_list_form_revisions(handle, type_id, #type_id)))
end

function Operations:lockbox_create_form_record(handle, path, type_reference, name)
  return Models.FormRecord.decode(take(native.lockbox_create_form_record(handle, path, #path, type_reference, #type_reference, name, #name)))
end

function Operations:lockbox_set_form_field(handle, path, field, value)
  if not native.lockbox_set_form_field(handle, path, #path, field, #field, value, #value) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_set_secret_form_field(handle, path, field, value)
  local bytes = ffi.new('uint8_t[?]', math.max(1, #value)); ffi.copy(bytes, value, #value)
  local ok = native.lockbox_set_secret_form_field(handle, path, #path, field, #field, bytes, #value)
  ffi.fill(bytes, math.max(1, #value), 0)
  if not ok then error(last_error(), 2) end
  return true
end

function Operations:lockbox_list_form_records(handle)
  return Models.FormRecordList.decode(take(native.lockbox_list_form_records(handle)))
end

function Operations:lockbox_get_form_record(handle, path)
  return Models.OptionalFormRecord.decode(take(native.lockbox_get_form_record(handle, path, #path)))
end

function Operations:lockbox_delete_form_record(handle, path)
  if not native.lockbox_delete_form_record(handle, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_move_form_records(handle, moves_flatbuffer)
  if not native.lockbox_move_form_records(handle, moves_flatbuffer, #moves_flatbuffer) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_get_form_field(handle, path, field)
  return Models.OptionalFormValue.decode(take(native.lockbox_get_form_field(handle, path, #path, field, #field)))
end

function Operations:lockbox_get_secret_form_field(handle, path, field, callback)
  return with_secret(function(output) return native.lockbox_get_secret_form_field(handle, path, #path, field, #field, output) end, callback)
end

function Operations:lockbox_to_bytes(handle)
  return take(native.lockbox_to_bytes(handle))
end

function Operations:lockbox_free(handle)
  native.lockbox_free(handle)
end

function Operations:vault_is_running()
  return native.vault_is_running()
end

function Operations:vault_forget_all()
  if not native.vault_forget_all() then error(last_error(), 2) end
  return true
end

function Operations:key_contact_generate()
  local value = native.key_contact_generate()
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_contact_from_private(bytes)
  local value = native.key_contact_from_private(bytes, #bytes)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_contact_public(handle)
  return take(native.key_contact_public(handle))
end

function Operations:key_contact_private(handle)
  return take(native.key_contact_private(handle))
end

function Operations:key_contact_public_from_bytes(bytes)
  local value = native.key_contact_public_from_bytes(bytes, #bytes)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_contact_public_free(handle)
  native.key_contact_public_free(handle)
end

function Operations:key_contact_free(handle)
  native.key_contact_free(handle)
end

function Operations:key_contact_encrypt(contact, content_key)
  local value = native.key_contact_encrypt(contact, content_key, #content_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_contact_decrypt(contact, wrapped)
  return take(native.key_contact_decrypt(contact, wrapped))
end

function Operations:key_contact_wrapped_public(wrapped)
  return take(native.key_contact_wrapped_public(wrapped))
end

function Operations:key_contact_wrapped_ciphertext(wrapped)
  return take(native.key_contact_wrapped_ciphertext(wrapped))
end

function Operations:key_contact_wrapped_encrypted(wrapped)
  return take(native.key_contact_wrapped_encrypted(wrapped))
end

function Operations:key_contact_wrapped_free(handle)
  native.key_contact_wrapped_free(handle)
end

function Operations:key_signing_generate()
  local value = native.key_signing_generate()
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_signing_from_private(bytes)
  local value = native.key_signing_from_private(bytes, #bytes)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_signing_public(handle)
  return take(native.key_signing_public(handle))
end

function Operations:key_signing_private(handle)
  return take(native.key_signing_private(handle))
end

function Operations:key_signing_public_from_bytes(bytes)
  local value = native.key_signing_public_from_bytes(bytes, #bytes)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:key_signing_public_free(handle)
  native.key_signing_public_free(handle)
end

function Operations:key_signing_free(handle)
  native.key_signing_free(handle)
end

function Operations:vault_key_export_private(key, format)
  return take(native.vault_key_export_private(key, format, #format))
end

function Operations:vault_key_export_public(key, format)
  return take(native.vault_key_export_public(key, format, #format))
end

function Operations:vault_key_import_private(bytes)
  local value = native.vault_key_import_private(bytes, #bytes)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_key_import_public(bytes)
  local value = native.vault_key_import_public(bytes, #bytes)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_key_fingerprint(key)
  return take(native.vault_key_fingerprint(key))
end

function Operations:vault_key_format_hex(bytes)
  return take(native.vault_key_format_hex(bytes, #bytes))
end

function Operations:vault_key_decode_hex(text)
  return take(native.vault_key_decode_hex(text, #text))
end

function Operations:vault_key_format_crockford(bytes)
  return take(native.vault_key_format_crockford(bytes, #bytes))
end

function Operations:vault_key_format_crockford_reading(code)
  return take(native.vault_key_format_crockford_reading(code, #code))
end

function Operations:vault_key_decode_crockford(code)
  return take(native.vault_key_decode_crockford(code, #code))
end

function Operations:vault_key_hex_encode(bytes)
  return take(native.vault_key_hex_encode(bytes, #bytes))
end

function Operations:vault_key_hex_decode(text)
  return take(native.vault_key_hex_decode(text, #text))
end

function Operations:vault_directory_open(root, password)
  local value = native.vault_directory_open(root, #root, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_structure_version_current()
  return tonumber(native.vault_structure_version_current())
end

function Operations:vault_directory_probe_structure_version(root, password)
  return tonumber(native.vault_directory_probe_structure_version(root, #root, password, #password))
end

function Operations:vault_directory_open_or_create_default(password)
  local value = native.vault_directory_open_or_create_default(password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_replace_default(password)
  local value = native.vault_directory_replace_default(password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_change_password(root, old_password, new_password)
  if not native.vault_directory_change_password(root, #root, old_password, #old_password, new_password, #new_password) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_change_default_password(old_password, new_password)
  if not native.vault_directory_change_default_password(old_password, #old_password, new_password, #new_password) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_replace(root, password)
  local value = native.vault_directory_replace(root, #root, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_open_or_create(root, password)
  local value = native.vault_directory_open_or_create(root, #root, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_root(handle)
  return take(native.vault_directory_root(handle))
end

function Operations:vault_directory_structure_version(handle)
  return tonumber(native.vault_directory_structure_version(handle))
end

function Operations:vault_directory_list_private_keys(handle)
  return Models.StringList.decode(take(native.vault_directory_list_private_keys(handle)))
end

function Operations:vault_directory_list_private_key_names(handle)
  return Models.StringList.decode(take(native.vault_directory_list_private_key_names(handle)))
end

function Operations:vault_directory_list_contact_names(handle)
  return Models.StringList.decode(take(native.vault_directory_list_contact_names(handle)))
end

function Operations:vault_directory_list_form_aliases(handle)
  return Models.StringList.decode(take(native.vault_directory_list_form_aliases(handle)))
end

function Operations:vault_directory_private_key_exists(handle, name)
  return native.vault_directory_private_key_exists(handle, name, #name)
end

function Operations:vault_directory_delete_private_key(handle, name)
  if not native.vault_directory_delete_private_key(handle, name, #name) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_store_private_key(handle, name, key)
  if not native.vault_directory_store_private_key(handle, name, #name, key) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_load_private_key(handle, name)
  local value = native.vault_directory_load_private_key(handle, name, #name)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_load_private_key_generation(handle, name, index)
  local value = native.vault_directory_load_private_key_generation(handle, name, #name, index)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_store_contact(handle, name, key)
  if not native.vault_directory_store_contact(handle, name, #name, key) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_load_contact(handle, name)
  local value = native.vault_directory_load_contact(handle, name, #name)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_contact_exists(handle, name)
  return native.vault_directory_contact_exists(handle, name, #name)
end

function Operations:vault_directory_delete_contact(handle, name)
  if not native.vault_directory_delete_contact(handle, name, #name) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_list_contacts(handle)
  return Models.ContactList.decode(take(native.vault_directory_list_contacts(handle)))
end

function Operations:vault_directory_store_profile_email(handle, name, email)
  if not native.vault_directory_store_profile_email(handle, name, #name, email, #email) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_profile_email(handle, name)
  return Models.OptionalString.decode(take(native.vault_directory_profile_email(handle, name, #name)))
end

function Operations:vault_directory_store_backup(handle, id, bytes)
  if not native.vault_directory_store_backup(handle, id, #id, bytes, #bytes) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_load_backup(handle, id)
  return take(native.vault_directory_load_backup(handle, id, #id))
end

function Operations:vault_directory_backup_count(handle)
  return tonumber(native.vault_directory_backup_count(handle))
end

function Operations:vault_directory_restore_private_key(handle, name, key, signing_key, overwrite)
  if not native.vault_directory_restore_private_key(handle, name, #name, key, signing_key, overwrite) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_load_owner_signing_key(handle, name)
  local value = native.vault_directory_load_owner_signing_key(handle, name, #name)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_load_owner_signing_key_generation(handle, name, index)
  local value = native.vault_directory_load_owner_signing_key_generation(handle, name, #name, index)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_store_contact_signing_key(handle, name, key)
  if not native.vault_directory_store_contact_signing_key(handle, name, #name, key) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_load_contact_signing_key(handle, name)
  local value = native.vault_directory_load_contact_signing_key(handle, name, #name)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_directory_list_profile_generations(handle, name)
  return Models.ProfileHistory.decode(take(native.vault_directory_list_profile_generations(handle, name, #name)))
end

function Operations:vault_directory_rotate_private_key(handle, name)
  return Models.ProfileHistory.decode(take(native.vault_directory_rotate_private_key(handle, name, #name)))
end

function Operations:vault_directory_remember_lockbox(handle, id, path)
  if not native.vault_directory_remember_lockbox(handle, id, #id, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_list_known_lockboxes(handle)
  return Models.KnownLockboxList.decode(take(native.vault_directory_list_known_lockboxes(handle)))
end

function Operations:vault_directory_forget_lockbox(handle, path)
  if not native.vault_directory_forget_lockbox(handle, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_remember_access_slot_label(handle, id, slot_id, name)
  if not native.vault_directory_remember_access_slot_label(handle, id, #id, slot_id, name, #name) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_list_access_slot_labels(handle, id)
  return Models.AccessSlotLabelList.decode(take(native.vault_directory_list_access_slot_labels(handle, id, #id)))
end

function Operations:vault_directory_find_access_slot_labels(handle, id, name)
  return Models.AccessSlotLabelList.decode(take(native.vault_directory_find_access_slot_labels(handle, id, #id, name, #name)))
end

function Operations:vault_directory_forget_access_slot_label(handle, id, slot_id)
  if not native.vault_directory_forget_access_slot_label(handle, id, #id, slot_id) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_define_form(handle, alias, name, description, fields_flatbuffer)
  return Models.FormDefinition.decode(take(native.vault_directory_define_form(handle, alias, #alias, name, #name, description, #description, fields_flatbuffer, #fields_flatbuffer)))
end

function Operations:vault_directory_resolve_form(handle, reference)
  return Models.FormDefinition.decode(take(native.vault_directory_resolve_form(handle, reference, #reference)))
end

function Operations:vault_directory_list_forms(handle)
  return Models.FormDefinitionList.decode(take(native.vault_directory_list_forms(handle)))
end

function Operations:vault_directory_list_form_revisions(handle, type_id)
  return Models.FormDefinitionList.decode(take(native.vault_directory_list_form_revisions(handle, type_id, #type_id)))
end

function Operations:vault_directory_seed_forms(handle)
  return tonumber(native.vault_directory_seed_forms(handle))
end

function Operations:vault_directory_remember_password(handle, id, password)
  if not native.vault_directory_remember_password(handle, id, #id, password, #password) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_remembered_password(handle, id)
  return take(native.vault_directory_remembered_password(handle, id, #id))
end

function Operations:vault_backup_default(path, overwrite)
  return Models.VaultBackupManifest.decode(take(native.vault_backup_default(path, #path, overwrite)))
end

function Operations:vault_restore_default(path, overwrite)
  return Models.VaultBackupManifest.decode(take(native.vault_restore_default(path, #path, overwrite)))
end

function Operations:vault_directory_free(handle)
  native.vault_directory_free(handle)
end

function Operations:vault_read_only_open(root, password)
  local value = native.vault_read_only_open(root, #root, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_read_only_open_default(password)
  local value = native.vault_read_only_open_default(password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_read_only_list_profile_names(handle)
  return Models.StringList.decode(take(native.vault_read_only_list_profile_names(handle)))
end

function Operations:vault_read_only_list_contact_names(handle)
  return Models.StringList.decode(take(native.vault_read_only_list_contact_names(handle)))
end

function Operations:vault_read_only_list_form_aliases(handle)
  return Models.StringList.decode(take(native.vault_read_only_list_form_aliases(handle)))
end

function Operations:vault_read_only_list_known_lockboxes(handle)
  return Models.KnownLockboxList.decode(take(native.vault_read_only_list_known_lockboxes(handle)))
end

function Operations:vault_read_only_free(handle)
  native.vault_read_only_free(handle)
end

function Operations:vault_agent_serve()
  if not native.vault_agent_serve() then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_verify_transport()
  if not native.vault_agent_verify_transport() then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_get(id)
  return take(native.vault_agent_get(id, #id))
end

function Operations:vault_agent_put(id, key)
  if not native.vault_agent_put(id, #id, key, #key) then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_forget(id)
  if not native.vault_agent_forget(id, #id) then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_stop()
  if not native.vault_agent_stop() then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_start()
  if not native.vault_agent_start() then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_list()
  return Models.AgentEntryList.decode(take(native.vault_agent_list()))
end

function Operations:vault_agent_sleep_support()
  return Models.SleepSupport.decode(take(native.vault_agent_sleep_support()))
end

function Operations:vault_platform_status()
  return Models.PlatformStatus.decode(take(native.vault_platform_status()))
end

function Operations:vault_platform_set_scope(scope)
  if not native.vault_platform_set_scope(scope, #scope) then error(last_error(), 2) end
  return true
end

function Operations:vault_platform_forget_password()
  if not native.vault_platform_forget_password() then error(last_error(), 2) end
  return true
end

function Operations:vault_platform_put_password(password)
  if not native.vault_platform_put_password(password, #password) then error(last_error(), 2) end
  return true
end

function Operations:vault_platform_enable()
  if not native.vault_platform_enable() then error(last_error(), 2) end
  return true
end

function Operations:vault_platform_disable()
  if not native.vault_platform_disable() then error(last_error(), 2) end
  return true
end

function Operations:vault_platform_disabled()
  return native.vault_platform_disabled()
end

function Operations:vault_platform_get_password()
  return take(native.vault_platform_get_password())
end

function Operations:vault_default_directory()
  return take(native.vault_default_directory())
end

function Operations:vault_default_path()
  return take(native.vault_default_path())
end

function Operations:vault_agent_log_path()
  return take(native.vault_agent_log_path())
end

function Operations:vault_agent_log_destination()
  return take(native.vault_agent_log_destination())
end

function Operations:vault_agent_get_vault_unlock_key(vault_id)
  return take(native.vault_agent_get_vault_unlock_key(vault_id, #vault_id))
end

function Operations:vault_agent_put_vault_unlock_key(vault_id, key, ttl_seconds)
  if not native.vault_agent_put_vault_unlock_key(vault_id, #vault_id, key, #key, ttl_seconds) then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_forget_vault_unlock_key(vault_id)
  if not native.vault_agent_forget_vault_unlock_key(vault_id, #vault_id) then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_get_owner_signing_key(vault_id, profile)
  local value = native.vault_agent_get_owner_signing_key(vault_id, #vault_id, profile, #profile)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_agent_put_owner_signing_key(vault_id, profile, key, ttl_seconds)
  if not native.vault_agent_put_owner_signing_key(vault_id, #vault_id, profile, #profile, key, ttl_seconds) then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_forget_owner_signing_key(vault_id, profile)
  if not native.vault_agent_forget_owner_signing_key(vault_id, #vault_id, profile, #profile) then error(last_error(), 2) end
  return true
end

function Operations:vault_agent_begin_activity(kind)
  local value = native.vault_agent_begin_activity(kind, #kind)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_agent_end_activity(handle)
  native.vault_agent_end_activity(handle)
end

function Operations:vault_local()
  local value = native.vault_local()
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_create_lockbox_password(vault, path, password)
  local value = native.vault_create_lockbox_password(vault, path, #path, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_open_lockbox_password(vault, path, password)
  local value = native.vault_open_lockbox_password(vault, path, #path, password, #password)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_create_lockbox_content_key(vault, path, content_key, signing_key)
  local value = native.vault_create_lockbox_content_key(vault, path, #path, content_key, #content_key, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_create_lockbox_contact(vault, path, contact, name, signing_key)
  local value = native.vault_create_lockbox_contact(vault, path, #path, contact, name, #name, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_open_lockbox_content_key(vault, path, content_key, signing_key)
  local value = native.vault_open_lockbox_content_key(vault, path, #path, content_key, #content_key, signing_key)
  if value == nil then error(last_error(), 2) end
  return value
end

function Operations:vault_cache_lockbox_password(vault, path, password, ttl_seconds)
  if not native.vault_cache_lockbox_password(vault, path, #path, password, #password, ttl_seconds) then error(last_error(), 2) end
  return true
end

function Operations:vault_close_lockbox(vault, path)
  if not native.vault_close_lockbox(vault, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:vault_close_all(vault)
  if not native.vault_close_all(vault) then error(last_error(), 2) end
  return true
end

function Operations:vault_free(vault)
  native.vault_free(vault)
end

local classes = {}
local function owned(name)
  local class = { __name = name }; class.__index = class
  function class.new(operations, handle) return setmetatable({ operations = operations, handle = handle }, class) end
  classes[name] = class; return class
end

--- Primary API for opening lockboxes, managing keys and metadata, using the
--- Session Agent, and accessing the platform credential store.
-- @type Vault
local Revault = owned("Revault")
--- An open encrypted archive containing files, variables, secrets, and forms.
-- @type Lockbox
local Lockbox = owned("Lockbox")
--- A profile's contact-encryption identity used to decrypt keys addressed to it.
-- @type ContactKeyPair
local ContactKeyPair = owned("ContactKeyPair")
--- A recipient's shareable encryption identity used when granting access.
-- @type ContactPublicKey
local ContactPublicKey = owned("ContactPublicKey")
--- A content key encrypted for one contact and recoverable by its matching key pair.
-- @type WrappedContactKey
local WrappedContactKey = owned("WrappedContactKey")
--- A Vault Profile signing identity used to authorize mutable Lockbox revisions.
-- @type ProfileSigningKeyPair
local ProfileSigningKeyPair = owned("ProfileSigningKeyPair")
--- The public half of a Vault Profile signing identity.
-- @type ProfileSigningPublicKey
local ProfileSigningPublicKey = owned("ProfileSigningPublicKey")
--- Password-protected storage for Profile keys, contacts, forms, backups, and lockbox paths.
-- @type VaultStore
local VaultStore = owned("VaultStore")
--- A metadata view for discovery that never loads an owner signing key.
-- @type ReadOnlyVault
local ReadOnlyVault = owned("ReadOnlyVault")
--- Client for the session service that temporarily caches unlock and signing keys.
-- @type Agent
local Agent = owned("Agent")
--- A token kept alive while an operation needs secrets cached by the agent.
-- @type AgentActivity
local AgentActivity = owned("AgentActivity")
--- Access to the platform credential store for a scoped Vault passphrase.
-- @type Platform
local Platform = owned("Platform")
--- A session that opens lockboxes by host path, caches passwords, and closes local files.
-- @type LocalSession
local LocalSession = owned("LocalSession")
local Vault = VaultStore

local new_vault_handle = Revault.new
--- Loads the native library and creates the main reVault facade.
function Revault.new()
  local operations = Operations.new()
  local value = new_vault_handle(operations, nil)
  value.agent = Agent.new(operations, nil); value.platform = Platform.new(operations, nil)
  return value
end
--- Returns the last error.
function Revault:last_error() return self.operations:last_error_message() end
--- Returns the last error details.
function Revault:last_error_details() return self.operations:buffer_last_error_details() end

--- Returns the newest Lockbox archive format version supported by this engine.
function Revault:lockbox_format_version()
  return self.operations:lockbox_format_version()
end

--- Reads the format version from serialized Lockbox bytes without opening them.
function Revault:lockbox_probe_format_version(bytes)
  return self.operations:lockbox_probe_format_version(bytes)
end

--- Creates an in memory Lockbox protected by a 32 byte content key.
function Revault:lockbox_create(key)
  return Lockbox.new(self.operations, self.operations:lockbox_create(key))
end

--- Creates a lockbox with explicit cache capacity, workload, worker policy, and job count.
function Revault:lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
  return Lockbox.new(self.operations, self.operations:lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs))
end

--- Creates an in memory Lockbox protected by the supplied password.
function Revault:lockbox_create_password(password)
  return Lockbox.new(self.operations, self.operations:lockbox_create_password(password))
end

--- Creates a password protected Lockbox with the supplied profile signing key.
function Revault:lockbox_create_password_with_signing_key(password, signing_key)
  return Lockbox.new(self.operations, self.operations:lockbox_create_password_with_signing_key(password, signing_key.handle))
end

--- Creates an in memory Lockbox that the supplied contact can open.
function Revault:lockbox_create_contact(contact)
  return Lockbox.new(self.operations, self.operations:lockbox_create_contact(contact.handle))
end

--- Creates a contact protected Lockbox with the supplied profile signing key.
function Revault:lockbox_create_contact_with_signing_key(contact, signing_key)
  return Lockbox.new(self.operations, self.operations:lockbox_create_contact_with_signing_key(contact.handle, signing_key.handle))
end

--- Creates an in memory Lockbox and assigns its profile signing key.
function Revault:lockbox_create_with_signing_key(content_key, signing_key)
  return Lockbox.new(self.operations, self.operations:lockbox_create_with_signing_key(content_key, signing_key.handle))
end

--- Opens serialized Lockbox bytes with a 32 byte content key.
function Revault:lockbox_open(archive, key)
  return Lockbox.new(self.operations, self.operations:lockbox_open(archive, key))
end

--- Opens a lockbox with explicit cache capacity, workload, worker policy, and job count.
function Revault:lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
  return Lockbox.new(self.operations, self.operations:lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs))
end

--- Opens serialized Lockbox bytes with the supplied password.
function Revault:lockbox_open_password(archive, password)
  return Lockbox.new(self.operations, self.operations:lockbox_open_password(archive, password))
end

--- Opens serialized Lockbox bytes with the supplied contact private key.
function Revault:lockbox_open_contact(archive, contact)
  return Lockbox.new(self.operations, self.operations:lockbox_open_contact(archive, contact.handle))
end

--- Reads public header, signature, and access slot metadata from a Lockbox file.
function Revault:lockbox_inspect_file(path)
  return self.operations:lockbox_inspect_file(path)
end

--- Scans a damaged Lockbox file with its 32 byte content key.
function Revault:lockbox_recovery_scan_path(path, key)
  return self.operations:lockbox_recovery_scan_path(path, key)
end

--- Scans damaged serialized Lockbox bytes with their 32 byte content key.
function Revault:lockbox_recovery_scan(bytes, key)
  return self.operations:lockbox_recovery_scan(bytes, key)
end

--- Builds a new Lockbox from recoverable records without changing the source.
function Revault:lockbox_recovery_salvage(bytes, key, signing_key)
  return Lockbox.new(self.operations, self.operations:lockbox_recovery_salvage(bytes, key, signing_key.handle))
end

--- Generates a contact encryption key pair using secure random data.
function Revault:key_contact_generate()
  return ContactKeyPair.new(self.operations, self.operations:key_contact_generate())
end

--- Imports a contact key pair from its private binary record.
function Revault:key_contact_from_private(bytes)
  return ContactKeyPair.new(self.operations, self.operations:key_contact_from_private(bytes))
end

--- Imports a contact public key from its binary representation.
function Revault:key_contact_public_from_bytes(bytes)
  return ContactPublicKey.new(self.operations, self.operations:key_contact_public_from_bytes(bytes))
end

--- Generates a signing identity owned by a Vault Profile.
function Revault:generate_profile_signing_key_pair()
  return ProfileSigningKeyPair.new(self.operations, self.operations:key_signing_generate())
end

--- Imports a Vault Profile signing identity from its private record.
function Revault:profile_signing_key_pair_from_private(bytes)
  return ProfileSigningKeyPair.new(self.operations, self.operations:key_signing_from_private(bytes))
end

--- Imports the public half of a Vault Profile signing identity.
function Revault:profile_signing_public_key_from_bytes(bytes)
  return ProfileSigningPublicKey.new(self.operations, self.operations:key_signing_public_from_bytes(bytes))
end

--- Exports a private key in the requested key format.
function Revault:vault_key_export_private(key, format)
  return self.operations:vault_key_export_private(key.handle, format)
end

--- Exports a public key in the requested key format.
function Revault:vault_key_export_public(key, format)
  return self.operations:vault_key_export_public(key.handle, format)
end

--- Imports a private contact key from a detected supported encoding.
function Revault:vault_key_import_private(bytes)
  return ContactKeyPair.new(self.operations, self.operations:vault_key_import_private(bytes))
end

--- Imports a public contact key from a detected supported encoding.
function Revault:vault_key_import_public(bytes)
  return ContactPublicKey.new(self.operations, self.operations:vault_key_import_public(bytes))
end

--- Returns the stable fingerprint used to verify a public key.
function Revault:vault_key_fingerprint(key)
  return self.operations:vault_key_fingerprint(key.handle)
end

--- Encodes key bytes as hexadecimal text.
function Revault:vault_key_format_hex(bytes)
  return self.operations:vault_key_format_hex(bytes)
end

--- Decodes hexadecimal key text and rejects malformed input.
function Revault:vault_key_decode_hex(text)
  return self.operations:vault_key_decode_hex(text)
end

--- Encodes key bytes using Crockford Base32.
function Revault:vault_key_format_crockford(bytes)
  return self.operations:vault_key_format_crockford(bytes)
end

--- Groups a Crockford code for easier reading and transcription.
function Revault:vault_key_format_crockford_reading(code)
  return self.operations:vault_key_format_crockford_reading(code)
end

--- Decodes Crockford Base32 key text and rejects malformed input.
function Revault:vault_key_decode_crockford(code)
  return self.operations:vault_key_decode_crockford(code)
end

--- Encodes arbitrary bytes as hexadecimal text.
function Revault:vault_key_hex_encode(bytes)
  return self.operations:vault_key_hex_encode(bytes)
end

--- Decodes arbitrary hexadecimal text and rejects malformed input.
function Revault:vault_key_hex_decode(text)
  return self.operations:vault_key_hex_decode(text)
end

--- Opens an existing Vault directory with its passphrase.
function Revault:vault_directory_open(root, password)
  return VaultStore.new(self.operations, self.operations:vault_directory_open(root, password))
end

--- Returns the newest Vault structure version supported by this engine.
function Revault:vault_structure_version_current()
  return self.operations:vault_structure_version_current()
end

--- Reads an existing Vault structure version without changing it.
function Revault:vault_directory_probe_structure_version(root, password)
  return self.operations:vault_directory_probe_structure_version(root, password)
end

--- Opens or creates the default Vault without replacing existing state.
function Revault:vault_directory_open_or_create_default(password)
  return VaultStore.new(self.operations, self.operations:vault_directory_open_or_create_default(password))
end

--- Replaces the default Vault and all persistent data it contains.
function Revault:vault_directory_replace_default(password)
  return VaultStore.new(self.operations, self.operations:vault_directory_replace_default(password))
end

--- Changes the passphrase for an existing Vault.
function Revault:vault_directory_change_password(root, old_password, new_password)
  return self.operations:vault_directory_change_password(root, old_password, new_password)
end

--- Changes the passphrase for the default Vault.
function Revault:vault_directory_change_default_password(old_password, new_password)
  return self.operations:vault_directory_change_default_password(old_password, new_password)
end

--- Replaces the selected Vault and all persistent data it contains.
function Revault:vault_directory_replace(root, password)
  return VaultStore.new(self.operations, self.operations:vault_directory_replace(root, password))
end

--- Opens the selected Vault, creating it only when absent.
function Revault:vault_directory_open_or_create(root, password)
  return VaultStore.new(self.operations, self.operations:vault_directory_open_or_create(root, password))
end

--- Writes a backup of the default Vault to the selected path.
function Revault:vault_backup_default(path, overwrite)
  return self.operations:vault_backup_default(path, overwrite)
end

--- Restores the default Vault from the selected backup.
function Revault:vault_restore_default(path, overwrite)
  return self.operations:vault_restore_default(path, overwrite)
end

--- Opens an existing Vault metadata view that cannot load private keys.
function Revault:vault_read_only_open(root, password)
  return ReadOnlyVault.new(self.operations, self.operations:vault_read_only_open(root, password))
end

--- Opens the default Vault metadata view without loading private keys.
function Revault:vault_read_only_open_default(password)
  return ReadOnlyVault.new(self.operations, self.operations:vault_read_only_open_default(password))
end

--- Returns the platform default Vault directory.
function Revault:vault_default_directory()
  return self.operations:vault_default_directory()
end

--- Returns the path of the default Vault file.
function Revault:vault_default_path()
  return self.operations:vault_default_path()
end

--- Returns the session agent log path.
function Revault:vault_agent_log_path()
  return self.operations:vault_agent_log_path()
end

--- Returns the configured session agent log destination.
function Revault:vault_agent_log_destination()
  return self.operations:vault_agent_log_destination()
end

--- Stages a file at the Lockbox path; replace controls an existing entry.
function Lockbox:add_file(path, data, replace)
  return self.operations:lockbox_add_file(self.handle, path, data, replace)
end

--- Stages a file and its portable Unix permission bits.
function Lockbox:add_file_with_permissions(path, data, permissions, replace)
  return self.operations:lockbox_add_file_with_permissions(self.handle, path, data, permissions, replace)
end

--- Reads the complete file stored at the Lockbox path.
function Lockbox:get_file(path)
  return self.operations:lockbox_get_file(self.handle, path)
end

--- Writes one Lockbox file to the host filesystem.
function Lockbox:extract_file(source, destination, replace)
  return self.operations:lockbox_extract_file(self.handle, source, destination, replace)
end

--- Extracts the Lockbox with explicit size, count, link, and permission limits.
function Lockbox:extract_directory(destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
  return self.operations:lockbox_extract_directory(self.handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
end

--- Lists logical or physical content chunks for streaming diagnostics.
function Lockbox:stream_content(physical)
  return self.operations:lockbox_stream_content(self.handle, physical)
end

--- Returns cache statistics for this lockbox.
function Lockbox:cache_stats()
  return self.operations:lockbox_cache_stats(self.handle)
end

--- Returns import statistics for this lockbox.
function Lockbox:import_stats()
  return self.operations:lockbox_import_stats(self.handle)
end

--- Updates import stats.
function Lockbox:reset_import_stats()
  return self.operations:lockbox_reset_import_stats(self.handle)
end

--- Returns page metadata for diagnostics without exposing plaintext secrets.
function Lockbox:page_inspection()
  return self.operations:lockbox_page_inspection(self.handle)
end

--- Scans the open archive and returns its structured recovery report.
function Lockbox:recovery_report()
  return self.operations:lockbox_recovery_report(self.handle)
end

--- Renders the recovery report for a person, capped at maxEntries.
function Lockbox:recovery_report_render(verbose, max_entries)
  return self.operations:lockbox_recovery_report_render(self.handle, verbose, max_entries)
end

--- Returns the current serialized archive size in bytes.
function Lockbox:storage_len()
  return self.operations:lockbox_storage_len(self.handle)
end

--- Sets workload profile.
function Lockbox:set_workload_profile(profile)
  return self.operations:lockbox_set_workload_profile(self.handle, profile)
end

--- Sets worker policy.
function Lockbox:set_worker_policy(mode, jobs)
  return self.operations:lockbox_set_worker_policy(self.handle, mode, jobs)
end

--- Returns the cache, workload, and worker settings used by this Lockbox.
function Lockbox:runtime_options()
  return self.operations:lockbox_runtime_options(self.handle)
end

--- Authenticates and publishes the staged changes.
function Lockbox:commit()
  return self.operations:lockbox_commit(self.handle)
end

--- Stages a directory entry and optionally creates missing parents.
function Lockbox:create_dir(path, create_parents)
  return self.operations:lockbox_create_dir(self.handle, path, create_parents)
end

--- Stages removal of a file, link, or empty directory at path.
function Lockbox:delete(path)
  return self.operations:lockbox_delete(self.handle, path)
end

--- Stages removal of a directory, optionally including its descendants.
function Lockbox:remove_dir(path, recursive)
  return self.operations:lockbox_remove_dir(self.handle, path, recursive)
end

--- Stages every missing parent directory for path.
function Lockbox:create_parent_dirs(path)
  return self.operations:lockbox_create_parent_dirs(self.handle, path)
end

--- Stages an atomic move from one Lockbox path to another.
function Lockbox:rename(from, to)
  return self.operations:lockbox_rename(self.handle, from, to)
end

--- Lists entries below path, optionally including descendants.
function Lockbox:list(path, recursive)
  return self.operations:lockbox_list(self.handle, path, recursive)
end

--- Lists entries using glob, type, recursion, and result limit filters.
function Lockbox:list_with_options(path, glob, recursive, include_files, include_symlinks, include_directories, limit)
  return self.operations:lockbox_list_with_options(self.handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
end

--- Returns metadata for the selected lockbox entry.
function Lockbox:stat(path)
  return self.operations:lockbox_stat(self.handle, path)
end

--- Stages a plain text variable; commit to publish the change.
function Lockbox:set_variable(name, value)
  return self.operations:lockbox_set_variable(self.handle, name, value)
end

--- Stores a secret variable from binary string bytes.
function Lockbox:set_secret_variable(name, value)
  return self.operations:lockbox_set_secret_variable(self.handle, name, value)
end

--- Returns a plain variable when it is present.
function Lockbox:get_variable(name)
  return self.operations:lockbox_get_variable(self.handle, name)
end

--- Returns the encrypted lockbox description, or nil when unset.
--- Example: set it, commit, then `print(box:description())`.
function Lockbox:description()
  return self:get_variable('/.revault/description')
end

--- Stages encrypted description text; call commit() to publish it.
--- Example: `box:set_description('Production credentials'); box:commit()`.
function Lockbox:set_description(description)
  return self:set_variable('/.revault/description', description)
end

--- Stages removal of the encrypted description; call commit().
--- Example: `box:clear_description(); box:commit()`.
function Lockbox:clear_description()
  return self:delete_variable('/.revault/description')
end

--- Calls `callback` with temporary secret bytes, then wipes the transfer copy.
function Lockbox:with_secret_variable(name, callback)
  return self.operations:lockbox_get_secret_variable(self.handle, name, callback)
end

--- Stages removal of a variable.
function Lockbox:delete_variable(name)
  return self.operations:lockbox_delete_variable(self.handle, name)
end

--- Atomically renames variables using source and destination path pairs.
function Lockbox:move_variables(moves)
  return self.operations:lockbox_move_variables(self.handle, flatbuffers.encode_path_moves(moves))
end

--- Lists variable names and metadata without exposing secret values.
function Lockbox:list_variables()
  return self.operations:lockbox_list_variables(self.handle)
end

--- Returns whether a variable is plain or secret.
function Lockbox:variable_sensitivity(name)
  return self.operations:lockbox_variable_sensitivity(self.handle, name)
end

--- Stages a symbolic link with its stored target text.
function Lockbox:add_symlink(path, target, replace)
  return self.operations:lockbox_add_symlink(self.handle, path, target, replace)
end

--- Returns the target text stored for a symbolic link.
function Lockbox:get_symlink_target(path)
  return self.operations:lockbox_get_symlink_target(self.handle, path)
end

--- Returns the stable public identifier stored in the Lockbox header.
function Lockbox:id()
  return self.operations:lockbox_id(self.handle)
end

--- Reports whether an entry exists at path.
function Lockbox:exists(path)
  return self.operations:lockbox_exists(self.handle, path)
end

--- Reports whether path names a directory entry.
function Lockbox:is_dir(path)
  return self.operations:lockbox_is_dir(self.handle, path)
end

--- Returns the portable Unix permission bits stored for path.
function Lockbox:permissions(path)
  return self.operations:lockbox_permissions(self.handle, path)
end

--- Stages portable Unix permission bits for path.
function Lockbox:set_permissions(path, permissions)
  return self.operations:lockbox_set_permissions(self.handle, path, permissions)
end

--- Reads the requested byte range from a stored file.
function Lockbox:read_range(path, offset, len)
  return self.operations:lockbox_read_range(self.handle, path, offset, len)
end

--- Adds a password access slot and returns its slot identifier.
function Lockbox:add_password(password)
  return self.operations:lockbox_add_password(self.handle, password)
end

--- Grants a named contact access and returns the new slot identifier.
function Lockbox:add_contact(contact, name)
  return self.operations:lockbox_add_contact(self.handle, contact.handle, name)
end

--- Removes an access slot; at least one usable slot must remain.
function Lockbox:delete_key(id)
  return self.operations:lockbox_delete_key(self.handle, id)
end

--- Lists public access slot metadata without returning credentials.
function Lockbox:list_key_slots()
  return self.operations:lockbox_list_key_slots(self.handle)
end

--- Assigns a profile signing key to the Lockbox owner role.
function Lockbox:set_owner_signing_key(key)
  return self.operations:lockbox_set_owner_signing_key(self.handle, key.handle)
end

--- Returns public signing and ownership metadata for the current revision.
function Lockbox:owner_inspection()
  return self.operations:lockbox_owner_inspection(self.handle)
end

--- Defines a reusable, versioned form from the supplied field definitions.
function Lockbox:define_form(alias, name, description, fields)
  return self.operations:lockbox_define_form(self.handle, alias, name, description, flatbuffers.encode_form_fields(fields))
end

--- Lists the form definitions stored in this Lockbox.
function Lockbox:list_form_definitions()
  return self.operations:lockbox_list_form_definitions(self.handle)
end

--- Resolves a form alias, type identifier, or revision.
function Lockbox:resolve_form(reference)
  return self.operations:lockbox_resolve_form(self.handle, reference)
end

--- Lists every stored revision for a form type identifier.
function Lockbox:list_form_revisions(type_id)
  return self.operations:lockbox_list_form_revisions(self.handle, type_id)
end

--- Stages a form record at path using the referenced definition.
function Lockbox:create_form_record(path, type_reference, name)
  return self.operations:lockbox_create_form_record(self.handle, path, type_reference, name)
end

--- Stages a plain field value in a form record.
function Lockbox:set_form_field(path, field, value)
  return self.operations:lockbox_set_form_field(self.handle, path, field, value)
end

--- Stores a secret form field from binary string bytes.
function Lockbox:set_secret_form_field(path, field, value)
  return self.operations:lockbox_set_secret_form_field(self.handle, path, field, value)
end

--- Lists form records without exposing secret field values.
function Lockbox:list_form_records()
  return self.operations:lockbox_list_form_records(self.handle)
end

--- Returns the form record at path when present.
function Lockbox:get_form_record(path)
  return self.operations:lockbox_get_form_record(self.handle, path)
end

--- Stages removal of a form record.
function Lockbox:delete_form_record(path)
  return self.operations:lockbox_delete_form_record(self.handle, path)
end

--- Atomically renames form records using source and destination path pairs.
function Lockbox:move_form_records(moves)
  return self.operations:lockbox_move_form_records(self.handle, flatbuffers.encode_path_moves(moves))
end

--- Returns a plain form field when it exists.
function Lockbox:get_form_field(path, field)
  return self.operations:lockbox_get_form_field(self.handle, path, field)
end

--- Calls `callback` with temporary field bytes, then wipes the transfer copy.
function Lockbox:with_secret_form_field(path, field, callback)
  return self.operations:lockbox_get_secret_form_field(self.handle, path, field, callback)
end

--- Serializes the current Lockbox, including committed changes.
function Lockbox:to_bytes()
  return self.operations:lockbox_to_bytes(self.handle)
end

--- Releases the native resources held by this object.
function Lockbox:free()
  self.operations:lockbox_free(self.handle)
  self.handle = nil
end

--- Returns the public.
function ContactKeyPair:public()
  return self.operations:key_contact_public(self.handle)
end

--- Returns the private.
function ContactKeyPair:private()
  return self.operations:key_contact_private(self.handle)
end

--- Releases the native resources held by this object.
function ContactKeyPair:free()
  self.operations:key_contact_free(self.handle)
  self.handle = nil
end

--- Decrypts a wrapped content key for this contact.
function ContactKeyPair:decrypt(wrapped)
  return self.operations:key_contact_decrypt(self.handle, wrapped.handle)
end

--- Releases this public contact key.
function ContactPublicKey:public_free()
  self.operations:key_contact_public_free(self.handle)
  self.handle = nil
end

--- Encrypts a content key for the selected contact.
function ContactPublicKey:encrypt(content_key)
  return WrappedContactKey.new(self.operations, self.operations:key_contact_encrypt(self.handle, content_key))
end

--- Returns the public.
function WrappedContactKey:public()
  return self.operations:key_contact_wrapped_public(self.handle)
end

--- Returns the encrypted content key bytes.
function WrappedContactKey:ciphertext()
  return self.operations:key_contact_wrapped_ciphertext(self.handle)
end

--- Returns the complete wrapped key record for storage or transport.
function WrappedContactKey:encrypted()
  return self.operations:key_contact_wrapped_encrypted(self.handle)
end

--- Releases the native resources held by this object.
function WrappedContactKey:free()
  self.operations:key_contact_wrapped_free(self.handle)
  self.handle = nil
end

--- Returns the canonical public bytes paired with this identity.
function ProfileSigningKeyPair:public_bytes()
  return self.operations:key_signing_public(self.handle)
end

--- Returns the private signing-key record for secure binary backup.
function ProfileSigningKeyPair:private_record()
  return self.operations:key_signing_private(self.handle)
end

--- Creates an independently owned public verification-key handle.
function ProfileSigningKeyPair:public_key()
  return ProfileSigningPublicKey.new(
    self.operations,
    self.operations:key_signing_public_from_bytes(self:public_bytes())
  )
end

--- Releases the native resources held by this object.
function ProfileSigningKeyPair:free()
  self.operations:key_signing_free(self.handle)
  self.handle = nil
end

--- Releases the native resources held by this object.
function ProfileSigningPublicKey:free()
  self.operations:key_signing_public_free(self.handle)
  self.handle = nil
end

--- Returns the canonical root directory of this Vault.
function VaultStore:root()
  return self.operations:vault_directory_root(self.handle)
end

--- Returns the persistent structure version of this Vault.
function VaultStore:structure_version()
  return self.operations:vault_directory_structure_version(self.handle)
end

--- Lists private keys.
function VaultStore:list_private_keys()
  return self.operations:vault_directory_list_private_keys(self.handle)
end

--- Lists private key names.
function VaultStore:list_private_key_names()
  return self.operations:vault_directory_list_private_key_names(self.handle)
end

--- Lists contact names.
function VaultStore:list_contact_names()
  return self.operations:vault_directory_list_contact_names(self.handle)
end

--- Lists form aliases.
function VaultStore:list_form_aliases()
  return self.operations:vault_directory_list_form_aliases(self.handle)
end

--- Reports whether the named profile private key exists.
function VaultStore:private_key_exists(name)
  return self.operations:vault_directory_private_key_exists(self.handle, name)
end

--- Removes private key.
function VaultStore:delete_private_key(name)
  return self.operations:vault_directory_delete_private_key(self.handle, name)
end

--- Stores private key.
function VaultStore:store_private_key(name, key)
  return self.operations:vault_directory_store_private_key(self.handle, name, key.handle)
end

--- Loads private key.
function VaultStore:load_private_key(name)
  return ContactKeyPair.new(self.operations, self.operations:vault_directory_load_private_key(self.handle, name))
end

--- Loads private key generation.
function VaultStore:load_private_key_generation(name, index)
  return ContactKeyPair.new(self.operations, self.operations:vault_directory_load_private_key_generation(self.handle, name, index))
end

--- Stores contact.
function VaultStore:store_contact(name, key)
  return self.operations:vault_directory_store_contact(self.handle, name, key.handle)
end

--- Loads contact.
function VaultStore:load_contact(name)
  return ContactPublicKey.new(self.operations, self.operations:vault_directory_load_contact(self.handle, name))
end

--- Reports whether the named contact exists.
function VaultStore:contact_exists(name)
  return self.operations:vault_directory_contact_exists(self.handle, name)
end

--- Removes contact.
function VaultStore:delete_contact(name)
  return self.operations:vault_directory_delete_contact(self.handle, name)
end

--- Lists contacts.
function VaultStore:list_contacts()
  return self.operations:vault_directory_list_contacts(self.handle)
end

--- Stores profile email.
function VaultStore:store_profile_email(name, email)
  return self.operations:vault_directory_store_profile_email(self.handle, name, email)
end

--- Returns the email recorded for a profile, when present.
function VaultStore:profile_email(name)
  return self.operations:vault_directory_profile_email(self.handle, name)
end

--- Stores backup.
function VaultStore:store_backup(id, bytes)
  return self.operations:vault_directory_store_backup(self.handle, id, bytes)
end

--- Loads backup.
function VaultStore:load_backup(id)
  return self.operations:vault_directory_load_backup(self.handle, id)
end

--- Returns the number of stored key recovery backups.
function VaultStore:backup_count()
  return self.operations:vault_directory_backup_count(self.handle)
end

--- Restores a profile private key and signing key from recovery material.
function VaultStore:restore_private_key(name, key, signing_key, overwrite)
  return self.operations:vault_directory_restore_private_key(self.handle, name, key.handle, signing_key.handle, overwrite)
end

--- Loads the current signing identity for a Vault Profile.
function VaultStore:load_profile_signing_key(name)
  return ProfileSigningKeyPair.new(self.operations, self.operations:vault_directory_load_owner_signing_key(self.handle, name))
end

--- Loads one historical signing identity for a Vault Profile.
function VaultStore:load_profile_signing_key_generation(name, index)
  return ProfileSigningKeyPair.new(self.operations, self.operations:vault_directory_load_owner_signing_key_generation(self.handle, name, index))
end

--- Stores contact signing key.
function VaultStore:store_contact_signing_key(name, key)
  return self.operations:vault_directory_store_contact_signing_key(self.handle, name, key.handle)
end

--- Loads contact signing key.
function VaultStore:load_contact_signing_key(name)
  return ProfileSigningPublicKey.new(self.operations, self.operations:vault_directory_load_contact_signing_key(self.handle, name))
end

--- Lists profile generations.
function VaultStore:list_profile_generations(name)
  return self.operations:vault_directory_list_profile_generations(self.handle, name)
end

--- Updates private key.
function VaultStore:rotate_private_key(name)
  return self.operations:vault_directory_rotate_private_key(self.handle, name)
end

--- Stores lockbox.
function VaultStore:remember_lockbox(id, path)
  return self.operations:vault_directory_remember_lockbox(self.handle, id, path)
end

--- Lists known lockboxes.
function VaultStore:list_known_lockboxes()
  return self.operations:vault_directory_list_known_lockboxes(self.handle)
end

--- Removes lockbox.
function VaultStore:forget_lockbox(path)
  return self.operations:vault_directory_forget_lockbox(self.handle, path)
end

--- Stores access slot label.
function VaultStore:remember_access_slot_label(id, slot_id, name)
  return self.operations:vault_directory_remember_access_slot_label(self.handle, id, slot_id, name)
end

--- Lists access slot labels.
function VaultStore:list_access_slot_labels(id)
  return self.operations:vault_directory_list_access_slot_labels(self.handle, id)
end

--- Finds access slot labels with the supplied name for one Lockbox.
function VaultStore:find_access_slot_labels(id, name)
  return self.operations:vault_directory_find_access_slot_labels(self.handle, id, name)
end

--- Removes access slot label.
function VaultStore:forget_access_slot_label(id, slot_id)
  return self.operations:vault_directory_forget_access_slot_label(self.handle, id, slot_id)
end

--- Defines a reusable, versioned form in the local vault.
function VaultStore:define_form(alias, name, description, fields)
  return self.operations:vault_directory_define_form(self.handle, alias, name, description, flatbuffers.encode_form_fields(fields))
end

--- Resolves a form alias, type identifier, or revision.
function VaultStore:resolve_form(reference)
  return self.operations:vault_directory_resolve_form(self.handle, reference)
end

--- Lists forms.
function VaultStore:list_forms()
  return self.operations:vault_directory_list_forms(self.handle)
end

--- Lists every stored revision for a form type identifier.
function VaultStore:list_form_revisions(type_id)
  return self.operations:vault_directory_list_form_revisions(self.handle, type_id)
end

--- Adds missing standard form definitions and returns the number added.
function VaultStore:seed_forms()
  return self.operations:vault_directory_seed_forms(self.handle)
end

--- Stores password.
function VaultStore:remember_password(id, password)
  return self.operations:vault_directory_remember_password(self.handle, id, password)
end

--- Returns the Lockbox password encrypted inside this Vault.
function VaultStore:remembered_password(id)
  return self.operations:vault_directory_remembered_password(self.handle, id)
end

--- Releases the native resources held by this object.
function VaultStore:free()
  self.operations:vault_directory_free(self.handle)
  self.handle = nil
end

--- Lists profile names.
function ReadOnlyVault:list_profile_names()
  return self.operations:vault_read_only_list_profile_names(self.handle)
end

--- Lists contact names.
function ReadOnlyVault:list_contact_names()
  return self.operations:vault_read_only_list_contact_names(self.handle)
end

--- Lists form aliases.
function ReadOnlyVault:list_form_aliases()
  return self.operations:vault_read_only_list_form_aliases(self.handle)
end

--- Lists known lockboxes.
function ReadOnlyVault:list_known_lockboxes()
  return self.operations:vault_read_only_list_known_lockboxes(self.handle)
end

--- Releases the native resources held by this object.
function ReadOnlyVault:free()
  self.operations:vault_read_only_free(self.handle)
  self.handle = nil
end

--- Reports whether running.
function Agent:is_running()
  return self.operations:vault_is_running()
end

--- Removes all.
function Agent:forget_all()
  return self.operations:vault_forget_all()
end

--- Runs the session agent server until it is stopped.
function Agent:serve()
  return self.operations:vault_agent_serve()
end

--- Verifies transport.
function Agent:verify_transport()
  return self.operations:vault_agent_verify_transport()
end

--- Returns get.
function Agent:get(id)
  return self.operations:vault_agent_get(id)
end

--- Stores put.
function Agent:put(id, key)
  return self.operations:vault_agent_put(id, key)
end

--- Removes forget.
function Agent:forget(id)
  return self.operations:vault_agent_forget(id)
end

--- Stops stop.
function Agent:stop()
  return self.operations:vault_agent_stop()
end

--- Starts start.
function Agent:start()
  return self.operations:vault_agent_start()
end

--- Lists entries below path, optionally including descendants.
function Agent:list()
  return self.operations:vault_agent_list()
end

--- Reports how the platform handles agent expiry during system sleep.
function Agent:sleep_support()
  return self.operations:vault_agent_sleep_support()
end

--- Returns vault unlock key.
function Agent:get_vault_unlock_key(vault_id)
  return self.operations:vault_agent_get_vault_unlock_key(vault_id)
end

--- Stores vault unlock key.
function Agent:put_vault_unlock_key(vault_id, key, ttl_seconds)
  return self.operations:vault_agent_put_vault_unlock_key(vault_id, key, ttl_seconds)
end

--- Removes vault unlock key.
function Agent:forget_vault_unlock_key(vault_id)
  return self.operations:vault_agent_forget_vault_unlock_key(vault_id)
end

--- Returns the cached signing identity for a Vault Profile.
function Agent:profile_signing_key(vault_id, profile)
  return ProfileSigningKeyPair.new(self.operations, self.operations:vault_agent_get_owner_signing_key(vault_id, profile))
end

--- Caches a signing identity for a Vault Profile.
function Agent:cache_profile_signing_key(vault_id, profile, key, ttl_seconds)
  return self.operations:vault_agent_put_owner_signing_key(vault_id, profile, key.handle, ttl_seconds)
end

--- Removes a cached signing identity for a Vault Profile.
function Agent:forget_profile_signing_key(vault_id, profile)
  return self.operations:vault_agent_forget_owner_signing_key(vault_id, profile)
end

--- Starts activity.
function Agent:begin_activity(kind)
  return AgentActivity.new(self.operations, self.operations:vault_agent_begin_activity(kind))
end

--- Stops activity.
function Agent:end_activity(handle)
  return self.operations:vault_agent_end_activity(handle.handle)
end

--- Returns availability and user presence guarantees for platform storage.
function Platform:status()
  return self.operations:vault_platform_status()
end

--- Sets scope.
function Platform:set_scope(scope)
  return self.operations:vault_platform_set_scope(scope)
end

--- Removes password.
function Platform:forget_password()
  return self.operations:vault_platform_forget_password()
end

--- Stores password.
function Platform:put_password(password)
  return self.operations:vault_platform_put_password(password)
end

--- Enables storage of the Vault passphrase in platform credentials.
function Platform:enable()
  return self.operations:vault_platform_enable()
end

--- Disables platform credential use without deleting the stored value.
function Platform:disable()
  return self.operations:vault_platform_disable()
end

--- Reports whether platform credential use is disabled.
function Platform:disabled()
  return self.operations:vault_platform_disabled()
end

--- Returns password.
function Platform:get_password()
  return self.operations:vault_platform_get_password()
end

--- Creates Lockbox password.
function LocalSession:create_lockbox_password(path, password)
  return Lockbox.new(self.operations, self.operations:vault_create_lockbox_password(self.handle, path, password))
end

--- Opens Lockbox password.
function LocalSession:open_lockbox_password(path, password)
  return Lockbox.new(self.operations, self.operations:vault_open_lockbox_password(self.handle, path, password))
end

--- Creates lockbox content key.
function LocalSession:create_lockbox_content_key(path, content_key, signing_key)
  return Lockbox.new(self.operations, self.operations:vault_create_lockbox_content_key(self.handle, path, content_key, signing_key.handle))
end

--- Creates lockbox contact.
function LocalSession:create_lockbox_contact(path, contact, name, signing_key)
  return Lockbox.new(self.operations, self.operations:vault_create_lockbox_contact(self.handle, path, contact.handle, name, signing_key.handle))
end

--- Opens lockbox content key.
function LocalSession:open_lockbox_content_key(path, content_key, signing_key)
  return Lockbox.new(self.operations, self.operations:vault_open_lockbox_content_key(self.handle, path, content_key, signing_key.handle))
end

--- Stores Lockbox password.
function LocalSession:cache_lockbox_password(path, password, ttl_seconds)
  return self.operations:vault_cache_lockbox_password(self.handle, path, password, ttl_seconds)
end

--- Releases the native resources held by lockbox.
function LocalSession:close_lockbox(path)
  return self.operations:vault_close_lockbox(self.handle, path)
end

--- Releases the native resources held by all.
function LocalSession:close_all()
  return self.operations:vault_close_all(self.handle)
end

--- Releases the native resources held by this object.
function LocalSession:free()
  self.operations:vault_free(self.handle)
  self.handle = nil
end

-- Reviewed 0.3 terminology. Native transport classes retain their private
-- implementation names while these values are the package-facing concepts.
local LockboxCacheMode = { bytes = 'bytes', disabled = 'disabled', automatic = 'automatic' }
local LockboxWorkload = { interactive = 'interactive', bulk_import = 'bulk-import', read_mostly = 'read-mostly' }
local LockboxWorker = { auto = 'auto', single = 'single', threads = 'threads' }
local AgentActivityKind = { open = 'open', close = 'close', variables = 'variables', form = 'form', recovery = 'recovery', vault = 'vault' }
local KeyExportFormat = { lockbox_pem = 'lockbox-pem', jwk = 'jwk', jwks = 'jwks', raw_hex = 'raw-hex' }
local SecretBytes = {}
function SecretBytes.new(value)
  local secret = { bytes = value or '' }
  function secret:close() self.bytes = string.rep('\0', #self.bytes) end
  return secret
end
local SecretString = SecretBytes

-- Opening an existing Vault is distinct from the destructive replacement API.
function Vault.open(root, vault_passphrase)
  return Revault.new():vault_directory_open(root, vault_passphrase)
end
function Vault.open_or_create(root, vault_passphrase)
  return Revault.new():vault_directory_open_or_create(root, vault_passphrase)
end
function Vault.create(root, vault_passphrase)
  return Revault.new():vault_directory_replace(root, vault_passphrase)
end
function Vault.replace(root, vault_passphrase)
  return Revault.new():vault_directory_replace(root, vault_passphrase)
end

--- Creates an in-memory archive protected by exactly one credential.
function Lockbox.create_in_memory(options)
  options = options or {}
  local credentials = 0
  for _, value in ipairs({ options.password, options.content_key, options.contact }) do if value ~= nil then credentials = credentials + 1 end end
  if credentials ~= 1 then error('supply exactly one of password, content_key, or contact') end
  local runtime = Revault.runtime()
  local box
  if options.password then box = options.signing_key and runtime:lockbox_create_password_with_signing_key(options.password, options.signing_key) or runtime:lockbox_create_password(options.password)
  elseif options.contact then box = options.signing_key and runtime:lockbox_create_contact_with_signing_key(options.contact, options.signing_key) or runtime:lockbox_create_contact(options.contact)
  elseif options.options then
    local tuning = options.options
    box = runtime:lockbox_create_with_options(options.content_key, tuning.cache_mode, tuning.cache_bytes or 0, tuning.workload, tuning.worker, tuning.jobs or 0)
  else box = runtime:lockbox_create(options.content_key) end
  if options.signing_key and not options.password and not options.contact then box:set_owner_signing_key(options.signing_key) end
  return box
end
--- Opens serialized archive bytes without consulting the Session Agent.
function Lockbox.open_bytes(archive, options)
  options = options or {}
  local credentials = 0
  for _, value in ipairs({ options.password, options.content_key, options.contact }) do if value ~= nil then credentials = credentials + 1 end end
  if credentials ~= 1 then error('supply exactly one of password, content_key, or contact') end
  local runtime = Revault.runtime()
  if options.password then return runtime:lockbox_open_password(archive, options.password) end
  if options.contact then return runtime:lockbox_open_contact(archive, options.contact) end
  if options.options then
    local tuning = options.options
    return runtime:lockbox_open_with_options(archive, options.content_key, tuning.cache_mode, tuning.cache_bytes or 0, tuning.workload, tuning.worker, tuning.jobs or 0)
  end
  return runtime:lockbox_open(archive, options.content_key)
end
--- Creates an archive file and returns its process-local handle.
function Lockbox.create(path, options)
  options = options or {}
  local existing = io.open(path, 'rb')
  if existing then existing:close(); if not options.overwrite then error('Lockbox already exists: ' .. path) end end
  local box = Lockbox.create_in_memory(options)
  local file = assert(io.open(path, 'wb')); file:write(box:to_bytes()); file:close()
  return box
end
--- Opens an archive file without consulting the Session Agent.
function Lockbox.open(path, options)
  local file = assert(io.open(path, 'rb')); local archive = file:read('*a'); file:close()
  return Lockbox.open_bytes(archive, options)
end

Lockbox.close = Lockbox.free
VaultStore.close = VaultStore.free
ReadOnlyVault.close = ReadOnlyVault.free
LocalSession.close = LocalSession.free
--- Explicit Session Agent controller with process-local lockbox operations.
local AgentSession = setmetatable({}, { __index = Agent })
AgentSession.__index = AgentSession
function AgentSession.new(operations, handle) return setmetatable({ operations = operations, handle = handle }, AgentSession) end
function AgentSession.instance()
  local runtime = Revault.new()
  return AgentSession.new(runtime.operations, runtime.operations:vault_local())
end
function AgentSession:create_lockbox_password(path, password) return Lockbox.new(self.operations, self.operations:vault_create_lockbox_password(self.handle, path, password)) end
function AgentSession:open_lockbox_password(path, password) return Lockbox.new(self.operations, self.operations:vault_open_lockbox_password(self.handle, path, password)) end
function AgentSession:create_lockbox_content_key(path, content_key, signing_key) return Lockbox.new(self.operations, self.operations:vault_create_lockbox_content_key(self.handle, path, content_key, signing_key.handle)) end
function AgentSession:create_lockbox_contact(path, contact, name, signing_key) return Lockbox.new(self.operations, self.operations:vault_create_lockbox_contact(self.handle, path, contact.handle, name, signing_key.handle)) end
function AgentSession:open_lockbox_content_key(path, content_key, signing_key) return Lockbox.new(self.operations, self.operations:vault_open_lockbox_content_key(self.handle, path, content_key, signing_key.handle)) end
function AgentSession:cache_lockbox_password(path, password, ttl_seconds) return self.operations:vault_cache_lockbox_password(self.handle, path, password, ttl_seconds) end
function AgentSession:close_lockbox(path) return self.operations:vault_close_lockbox(self.handle, path) end
function AgentSession:close_all() return self.operations:vault_close_all(self.handle) end
function AgentSession:free() self.operations:vault_free(self.handle); self.handle = nil end
Revault.load = function(native_library_path)
  if native_library_path ~= nil or loaded_native == nil then
    load_native(native_library_path)
  end
  return Revault.new()
end
Revault.runtime = Revault.load
local M = {
  Revault = Revault, Vault = Vault, ReadOnlyVault = ReadOnlyVault, Models = Models,
  Lockbox = Lockbox, ContactKeyPair = ContactKeyPair, ContactPublicKey = ContactPublicKey,
  WrappedContactKey = WrappedContactKey, ProfileSigningKeyPair = ProfileSigningKeyPair,
  ProfileSigningPublicKey = ProfileSigningPublicKey, AgentSession = AgentSession,
  AgentActivity = AgentActivity,
  SecretBytes = SecretBytes, SecretString = SecretString, LockboxCacheMode = LockboxCacheMode,
  LockboxWorkload = LockboxWorkload, LockboxWorker = LockboxWorker,
  AgentActivityKind = AgentActivityKind, KeyExportFormat = KeyExportFormat,
}
return M
