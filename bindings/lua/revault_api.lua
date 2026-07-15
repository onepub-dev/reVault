-- Generated complete class-oriented LuaJIT API. Do not edit.
local ffi = require('ffi')
local pb = require('pb')

ffi.cdef[[
typedef struct { uint8_t *ptr; size_t len; } RevaultBuffer;
uint32_t api_abi_version(void);
const char * buffer_last_error(void);
RevaultBuffer buffer_last_error_details(void);
void buffer_free(RevaultBuffer value);
uint16_t lockbox_format_version(void);
uint16_t lockbox_probe_format_version(const uint8_t * bytes, size_t len);
void * lockbox_create(const uint8_t * key, size_t key_len);
void * lockbox_create_with_options(const uint8_t * key, size_t key_len, const char * cache_mode, size_t cache_len, uint64_t cache_bytes, const char * workload, size_t workload_len, const char * worker, size_t worker_len, size_t jobs);
void * lockbox_create_password(const uint8_t * password, size_t len);
void * lockbox_create_contact(const void * contact);
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
bool lockbox_set_variable(void * handle, const char * name, size_t name_len, const char * value, size_t value_len, bool secret);
RevaultBuffer lockbox_get_variable(const void * handle, const char * name, size_t name_len);
bool lockbox_delete_variable(void * handle, const char * name, size_t name_len);
bool lockbox_move_variables(void * handle, const uint8_t * moves_proto, size_t moves_len);
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
RevaultBuffer lockbox_define_form(void * handle, const char * alias, size_t alias_len, const char * name, size_t name_len, const char * description, size_t description_len, const uint8_t * fields_proto, size_t fields_len);
RevaultBuffer lockbox_list_form_definitions(const void * handle);
RevaultBuffer lockbox_resolve_form(const void * handle, const char * reference, size_t reference_len);
RevaultBuffer lockbox_list_form_revisions(const void * handle, const char * type_id, size_t type_id_len);
RevaultBuffer lockbox_create_form_record(void * handle, const char * path, size_t path_len, const char * type_reference, size_t type_len, const char * name, size_t name_len);
bool lockbox_set_form_field(void * handle, const char * path, size_t path_len, const char * field, size_t field_len, const char * value, size_t value_len, bool secret);
RevaultBuffer lockbox_list_form_records(const void * handle);
RevaultBuffer lockbox_get_form_record(const void * handle, const char * path, size_t path_len);
bool lockbox_delete_form_record(void * handle, const char * path, size_t path_len);
bool lockbox_move_form_records(void * handle, const uint8_t * moves_proto, size_t moves_len);
RevaultBuffer lockbox_get_form_field(const void * handle, const char * path, size_t path_len, const char * field, size_t field_len);
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
RevaultBuffer vault_directory_define_form(const void * handle, const char * alias, size_t alias_len, const char * name, size_t name_len, const char * description, size_t description_len, const uint8_t * fields_proto, size_t fields_len);
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
  local override = os.getenv('REVAULT_LIBRARY')
  if override and #override > 0 then return override end
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
  error('revault-api native carrier is missing for ' .. target .. '; set REVAULT_LIBRARY for development')
end
local native = ffi.load(native_library())
if tonumber(native.api_abi_version()) ~= 1 then error('revault-api native ABI mismatch; expected 1') end
local descriptor = os.getenv('REVAULT_PROTO_DESCRIPTOR')
if not descriptor then
  for pattern in package.path:gmatch('[^;]+') do
    local directory = pattern:match('^(.*[/\\])')
    if directory then
      local candidate = directory .. 'revault_bindings.pb'
      local file = io.open(candidate, 'rb')
      if file then file:close(); descriptor = candidate; break end
    end
  end
end
descriptor = descriptor or 'bindings/lua/revault_bindings.pb'
assert(pb.loadfile(descriptor))

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
  function class:encode() return pb.encode('.revault.bindings.' .. name, self) end
  function class.decode(bytes) return wrap(name, assert(pb.decode('.revault.bindings.' .. name, bytes))) end
  Models[name] = class
end
model("AccessSlotLabel")
model("AccessSlotLabelList")
model("AgentEntry")
model("AgentEntryList")
model("ByteList")
model("CacheStats")
model("Contact")
model("ContactList")
model("ErrorDetails")
model("FileInspection")
model("FormDefinition")
model("FormDefinitionList")
model("FormField")
model("FormFieldList")
model("FormRecord")
model("FormRecordList")
model("FormValue")
model("ImportStats")
model("KeySlot")
model("KeySlotList")
model("KnownLockbox")
model("KnownLockboxList")
model("LockboxEntry")
model("LockboxEntryList")
model("OptionalFormRecord")
model("OptionalFormValue")
model("OptionalLockboxEntry")
model("OptionalString")
model("OwnerInspection")
model("PageInspection")
model("PageInspectionList")
model("PageObject")
model("PathMove")
model("PathMoveList")
model("PlatformStatus")
model("ProfileGeneration")
model("ProfileHistory")
model("ProfileHistoryList")
model("RecoveryReport")
model("RuntimeOptions")
model("SleepSupport")
model("StreamChunk")
model("StreamChunkList")
model("StringList")
model("StringValue")
model("Variable")
model("VariableList")
model("VaultBackupManifest")

local function take(buffer)
  if buffer.ptr == nil then error(last_error(), 3) end
  local value = ffi.string(buffer.ptr, tonumber(buffer.len))
  native.buffer_free(buffer)
  return value
end

local function payload(buffer)
  local frame = take(buffer)
  if #frame < 12 or frame:sub(1, 4) ~= 'LBWF' then error('invalid reVault binding frame', 3) end
  local a, b, c, d = frame:byte(9, 12)
  local length = ((a * 256 + b) * 256 + c) * 256 + d
  if length ~= #frame - 12 then error('invalid reVault binding frame length', 3) end
  return frame:sub(13)
end

local Operations = {}
Operations.__index = Operations
function Operations.new() return setmetatable({}, Operations) end
function Operations:last_error_message() return last_error() end

function Operations:buffer_last_error_details()
  return Models.ErrorDetails.decode(payload(native.buffer_last_error_details()))
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

function Operations:lockbox_create_contact(contact)
  local value = native.lockbox_create_contact(contact)
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
  return Models.StreamChunkList.decode(payload(native.lockbox_stream_content(handle, physical)))
end

function Operations:lockbox_cache_stats(handle)
  return Models.CacheStats.decode(payload(native.lockbox_cache_stats(handle)))
end

function Operations:lockbox_import_stats(handle)
  return Models.ImportStats.decode(payload(native.lockbox_import_stats(handle)))
end

function Operations:lockbox_reset_import_stats(handle)
  if not native.lockbox_reset_import_stats(handle) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_inspect_file(path)
  return Models.FileInspection.decode(payload(native.lockbox_inspect_file(path, #path)))
end

function Operations:lockbox_page_inspection(handle)
  return Models.PageInspectionList.decode(payload(native.lockbox_page_inspection(handle)))
end

function Operations:lockbox_recovery_report(handle)
  return Models.RecoveryReport.decode(payload(native.lockbox_recovery_report(handle)))
end

function Operations:lockbox_recovery_report_render(handle, verbose, max_entries)
  return take(native.lockbox_recovery_report_render(handle, verbose, max_entries))
end

function Operations:lockbox_recovery_scan_path(path, key)
  return Models.RecoveryReport.decode(payload(native.lockbox_recovery_scan_path(path, #path, key, #key)))
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
  return Models.RuntimeOptions.decode(payload(native.lockbox_runtime_options(handle)))
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
  return Models.LockboxEntryList.decode(payload(native.lockbox_list(handle, path, #path, recursive)))
end

function Operations:lockbox_list_with_options(handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
  return Models.LockboxEntryList.decode(payload(native.lockbox_list_with_options(handle, path, #path, glob, #glob, recursive, include_files, include_symlinks, include_directories, limit)))
end

function Operations:lockbox_stat(handle, path)
  return Models.OptionalLockboxEntry.decode(payload(native.lockbox_stat(handle, path, #path)))
end

function Operations:lockbox_set_variable(handle, name, value, secret)
  if not native.lockbox_set_variable(handle, name, #name, value, #value, secret) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_get_variable(handle, name)
  return take(native.lockbox_get_variable(handle, name, #name))
end

function Operations:lockbox_delete_variable(handle, name)
  if not native.lockbox_delete_variable(handle, name, #name) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_move_variables(handle, moves_proto)
  if not native.lockbox_move_variables(handle, moves_proto, #moves_proto) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_list_variables(handle)
  return Models.VariableList.decode(payload(native.lockbox_list_variables(handle)))
end

function Operations:lockbox_variable_sensitivity(handle, name)
  return Models.OptionalString.decode(payload(native.lockbox_variable_sensitivity(handle, name, #name)))
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
  return Models.RecoveryReport.decode(payload(native.lockbox_recovery_scan(bytes, #bytes, key, #key)))
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
  return Models.KeySlotList.decode(payload(native.lockbox_list_key_slots(handle)))
end

function Operations:lockbox_set_owner_signing_key(handle, key)
  if not native.lockbox_set_owner_signing_key(handle, key) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_owner_inspection(handle)
  return Models.OwnerInspection.decode(payload(native.lockbox_owner_inspection(handle)))
end

function Operations:lockbox_define_form(handle, alias, name, description, fields_proto)
  return Models.FormDefinition.decode(payload(native.lockbox_define_form(handle, alias, #alias, name, #name, description, #description, fields_proto, #fields_proto)))
end

function Operations:lockbox_list_form_definitions(handle)
  return Models.FormDefinitionList.decode(payload(native.lockbox_list_form_definitions(handle)))
end

function Operations:lockbox_resolve_form(handle, reference)
  return Models.FormDefinition.decode(payload(native.lockbox_resolve_form(handle, reference, #reference)))
end

function Operations:lockbox_list_form_revisions(handle, type_id)
  return Models.FormDefinitionList.decode(payload(native.lockbox_list_form_revisions(handle, type_id, #type_id)))
end

function Operations:lockbox_create_form_record(handle, path, type_reference, name)
  return Models.FormRecord.decode(payload(native.lockbox_create_form_record(handle, path, #path, type_reference, #type_reference, name, #name)))
end

function Operations:lockbox_set_form_field(handle, path, field, value, secret)
  if not native.lockbox_set_form_field(handle, path, #path, field, #field, value, #value, secret) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_list_form_records(handle)
  return Models.FormRecordList.decode(payload(native.lockbox_list_form_records(handle)))
end

function Operations:lockbox_get_form_record(handle, path)
  return Models.FormRecord.decode(payload(native.lockbox_get_form_record(handle, path, #path)))
end

function Operations:lockbox_delete_form_record(handle, path)
  if not native.lockbox_delete_form_record(handle, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_move_form_records(handle, moves_proto)
  if not native.lockbox_move_form_records(handle, moves_proto, #moves_proto) then error(last_error(), 2) end
  return true
end

function Operations:lockbox_get_form_field(handle, path, field)
  return Models.FormValue.decode(payload(native.lockbox_get_form_field(handle, path, #path, field, #field)))
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
  return Models.StringList.decode(payload(native.vault_directory_list_private_keys(handle)))
end

function Operations:vault_directory_list_private_key_names(handle)
  return Models.StringList.decode(payload(native.vault_directory_list_private_key_names(handle)))
end

function Operations:vault_directory_list_contact_names(handle)
  return Models.StringList.decode(payload(native.vault_directory_list_contact_names(handle)))
end

function Operations:vault_directory_list_form_aliases(handle)
  return Models.StringList.decode(payload(native.vault_directory_list_form_aliases(handle)))
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
  return Models.ContactList.decode(payload(native.vault_directory_list_contacts(handle)))
end

function Operations:vault_directory_store_profile_email(handle, name, email)
  if not native.vault_directory_store_profile_email(handle, name, #name, email, #email) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_profile_email(handle, name)
  return Models.OptionalString.decode(payload(native.vault_directory_profile_email(handle, name, #name)))
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
  return Models.ProfileHistory.decode(payload(native.vault_directory_list_profile_generations(handle, name, #name)))
end

function Operations:vault_directory_rotate_private_key(handle, name)
  return Models.ProfileHistory.decode(payload(native.vault_directory_rotate_private_key(handle, name, #name)))
end

function Operations:vault_directory_remember_lockbox(handle, id, path)
  if not native.vault_directory_remember_lockbox(handle, id, #id, path, #path) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_list_known_lockboxes(handle)
  return Models.KnownLockboxList.decode(payload(native.vault_directory_list_known_lockboxes(handle)))
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
  return Models.AccessSlotLabelList.decode(payload(native.vault_directory_list_access_slot_labels(handle, id, #id)))
end

function Operations:vault_directory_find_access_slot_labels(handle, id, name)
  return Models.AccessSlotLabelList.decode(payload(native.vault_directory_find_access_slot_labels(handle, id, #id, name, #name)))
end

function Operations:vault_directory_forget_access_slot_label(handle, id, slot_id)
  if not native.vault_directory_forget_access_slot_label(handle, id, #id, slot_id) then error(last_error(), 2) end
  return true
end

function Operations:vault_directory_define_form(handle, alias, name, description, fields_proto)
  return Models.FormDefinition.decode(payload(native.vault_directory_define_form(handle, alias, #alias, name, #name, description, #description, fields_proto, #fields_proto)))
end

function Operations:vault_directory_resolve_form(handle, reference)
  return Models.FormDefinition.decode(payload(native.vault_directory_resolve_form(handle, reference, #reference)))
end

function Operations:vault_directory_list_forms(handle)
  return Models.FormDefinitionList.decode(payload(native.vault_directory_list_forms(handle)))
end

function Operations:vault_directory_list_form_revisions(handle, type_id)
  return Models.FormDefinitionList.decode(payload(native.vault_directory_list_form_revisions(handle, type_id, #type_id)))
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
  return Models.VaultBackupManifest.decode(payload(native.vault_backup_default(path, #path, overwrite)))
end

function Operations:vault_restore_default(path, overwrite)
  return Models.VaultBackupManifest.decode(payload(native.vault_restore_default(path, #path, overwrite)))
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
  return Models.StringList.decode(payload(native.vault_read_only_list_profile_names(handle)))
end

function Operations:vault_read_only_list_contact_names(handle)
  return Models.StringList.decode(payload(native.vault_read_only_list_contact_names(handle)))
end

function Operations:vault_read_only_list_form_aliases(handle)
  return Models.StringList.decode(payload(native.vault_read_only_list_form_aliases(handle)))
end

function Operations:vault_read_only_list_known_lockboxes(handle)
  return Models.KnownLockboxList.decode(payload(native.vault_read_only_list_known_lockboxes(handle)))
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
  return Models.AgentEntryList.decode(payload(native.vault_agent_list()))
end

function Operations:vault_agent_sleep_support()
  return Models.SleepSupport.decode(payload(native.vault_agent_sleep_support()))
end

function Operations:vault_platform_status()
  return Models.PlatformStatus.decode(payload(native.vault_platform_status()))
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

local Vault = owned("Vault")
local Lockbox = owned("Lockbox")
local ContactKeyPair = owned("ContactKeyPair")
local ContactPublicKey = owned("ContactPublicKey")
local WrappedContactKey = owned("WrappedContactKey")
local SigningKeyPair = owned("SigningKeyPair")
local SigningPublicKey = owned("SigningPublicKey")
local VaultDirectory = owned("VaultDirectory")
local ReadOnlyVaultDirectory = owned("ReadOnlyVaultDirectory")
local Agent = owned("Agent")
local AgentActivity = owned("AgentActivity")
local Platform = owned("Platform")
local LocalVault = owned("LocalVault")

Vault.new_handle = Vault.new
function Vault.new()
  local operations = Operations.new()
  local value = Vault.new_handle and Vault.new_handle(operations, nil) or setmetatable({ operations = operations }, Vault)
  value.agent = Agent.new(operations, nil); value.platform = Platform.new(operations, nil)
  return value
end
function Vault:last_error() return self.operations:last_error_message() end
function Vault:last_error_details() return self.operations:buffer_last_error_details() end

function Vault:lockbox_format_version()
  return self.operations:lockbox_format_version()
end

function Vault:lockbox_probe_format_version(bytes)
  return self.operations:lockbox_probe_format_version(bytes)
end

function Vault:lockbox_create(key)
  return Lockbox.new(self.operations, self.operations:lockbox_create(key))
end

function Vault:lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs)
  return Lockbox.new(self.operations, self.operations:lockbox_create_with_options(key, cache_mode, cache_bytes, workload, worker, jobs))
end

function Vault:lockbox_create_password(password)
  return Lockbox.new(self.operations, self.operations:lockbox_create_password(password))
end

function Vault:lockbox_create_contact(contact)
  return Lockbox.new(self.operations, self.operations:lockbox_create_contact(contact.handle))
end

function Vault:lockbox_create_with_signing_key(content_key, signing_key)
  return Lockbox.new(self.operations, self.operations:lockbox_create_with_signing_key(content_key, signing_key.handle))
end

function Vault:lockbox_open(archive, key)
  return Lockbox.new(self.operations, self.operations:lockbox_open(archive, key))
end

function Vault:lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs)
  return Lockbox.new(self.operations, self.operations:lockbox_open_with_options(archive, key, cache_mode, cache_bytes, workload, worker, jobs))
end

function Vault:lockbox_open_password(archive, password)
  return Lockbox.new(self.operations, self.operations:lockbox_open_password(archive, password))
end

function Vault:lockbox_open_contact(archive, contact)
  return Lockbox.new(self.operations, self.operations:lockbox_open_contact(archive, contact.handle))
end

function Vault:lockbox_inspect_file(path)
  return self.operations:lockbox_inspect_file(path)
end

function Vault:lockbox_recovery_scan_path(path, key)
  return self.operations:lockbox_recovery_scan_path(path, key)
end

function Vault:lockbox_recovery_scan(bytes, key)
  return self.operations:lockbox_recovery_scan(bytes, key)
end

function Vault:lockbox_recovery_salvage(bytes, key, signing_key)
  return Lockbox.new(self.operations, self.operations:lockbox_recovery_salvage(bytes, key, signing_key.handle))
end

function Vault:key_contact_generate()
  return ContactKeyPair.new(self.operations, self.operations:key_contact_generate())
end

function Vault:key_contact_from_private(bytes)
  return ContactKeyPair.new(self.operations, self.operations:key_contact_from_private(bytes))
end

function Vault:key_contact_public_from_bytes(bytes)
  return ContactPublicKey.new(self.operations, self.operations:key_contact_public_from_bytes(bytes))
end

function Vault:key_signing_generate()
  return SigningKeyPair.new(self.operations, self.operations:key_signing_generate())
end

function Vault:key_signing_from_private(bytes)
  return SigningKeyPair.new(self.operations, self.operations:key_signing_from_private(bytes))
end

function Vault:key_signing_public_from_bytes(bytes)
  return SigningPublicKey.new(self.operations, self.operations:key_signing_public_from_bytes(bytes))
end

function Vault:vault_key_export_private(key, format)
  return self.operations:vault_key_export_private(key.handle, format)
end

function Vault:vault_key_export_public(key, format)
  return self.operations:vault_key_export_public(key.handle, format)
end

function Vault:vault_key_import_private(bytes)
  return ContactKeyPair.new(self.operations, self.operations:vault_key_import_private(bytes))
end

function Vault:vault_key_import_public(bytes)
  return ContactPublicKey.new(self.operations, self.operations:vault_key_import_public(bytes))
end

function Vault:vault_key_fingerprint(key)
  return self.operations:vault_key_fingerprint(key.handle)
end

function Vault:vault_key_format_hex(bytes)
  return self.operations:vault_key_format_hex(bytes)
end

function Vault:vault_key_decode_hex(text)
  return self.operations:vault_key_decode_hex(text)
end

function Vault:vault_key_format_crockford(bytes)
  return self.operations:vault_key_format_crockford(bytes)
end

function Vault:vault_key_format_crockford_reading(code)
  return self.operations:vault_key_format_crockford_reading(code)
end

function Vault:vault_key_decode_crockford(code)
  return self.operations:vault_key_decode_crockford(code)
end

function Vault:vault_key_hex_encode(bytes)
  return self.operations:vault_key_hex_encode(bytes)
end

function Vault:vault_key_hex_decode(text)
  return self.operations:vault_key_hex_decode(text)
end

function Vault:vault_directory_open(root, password)
  return VaultDirectory.new(self.operations, self.operations:vault_directory_open(root, password))
end

function Vault:vault_structure_version_current()
  return self.operations:vault_structure_version_current()
end

function Vault:vault_directory_probe_structure_version(root, password)
  return self.operations:vault_directory_probe_structure_version(root, password)
end

function Vault:vault_directory_open_or_create_default(password)
  return VaultDirectory.new(self.operations, self.operations:vault_directory_open_or_create_default(password))
end

function Vault:vault_directory_replace_default(password)
  return VaultDirectory.new(self.operations, self.operations:vault_directory_replace_default(password))
end

function Vault:vault_directory_change_password(root, old_password, new_password)
  return self.operations:vault_directory_change_password(root, old_password, new_password)
end

function Vault:vault_directory_change_default_password(old_password, new_password)
  return self.operations:vault_directory_change_default_password(old_password, new_password)
end

function Vault:vault_directory_replace(root, password)
  return VaultDirectory.new(self.operations, self.operations:vault_directory_replace(root, password))
end

function Vault:vault_directory_open_or_create(root, password)
  return VaultDirectory.new(self.operations, self.operations:vault_directory_open_or_create(root, password))
end

function Vault:vault_backup_default(path, overwrite)
  return self.operations:vault_backup_default(path, overwrite)
end

function Vault:vault_restore_default(path, overwrite)
  return self.operations:vault_restore_default(path, overwrite)
end

function Vault:vault_read_only_open(root, password)
  return ReadOnlyVaultDirectory.new(self.operations, self.operations:vault_read_only_open(root, password))
end

function Vault:vault_read_only_open_default(password)
  return ReadOnlyVaultDirectory.new(self.operations, self.operations:vault_read_only_open_default(password))
end

function Vault:vault_default_directory()
  return self.operations:vault_default_directory()
end

function Vault:vault_default_path()
  return self.operations:vault_default_path()
end

function Vault:vault_agent_log_path()
  return self.operations:vault_agent_log_path()
end

function Vault:vault_agent_log_destination()
  return self.operations:vault_agent_log_destination()
end

function Vault:vault_local()
  return LocalVault.new(self.operations, self.operations:vault_local())
end

function Lockbox:add_file(path, data, replace)
  return self.operations:lockbox_add_file(self.handle, path, data, replace)
end

function Lockbox:add_file_with_permissions(path, data, permissions, replace)
  return self.operations:lockbox_add_file_with_permissions(self.handle, path, data, permissions, replace)
end

function Lockbox:get_file(path)
  return self.operations:lockbox_get_file(self.handle, path)
end

function Lockbox:extract_file(source, destination, replace)
  return self.operations:lockbox_extract_file(self.handle, source, destination, replace)
end

function Lockbox:extract_directory(destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
  return self.operations:lockbox_extract_directory(self.handle, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite)
end

function Lockbox:stream_content(physical)
  return self.operations:lockbox_stream_content(self.handle, physical)
end

function Lockbox:cache_stats()
  return self.operations:lockbox_cache_stats(self.handle)
end

function Lockbox:import_stats()
  return self.operations:lockbox_import_stats(self.handle)
end

function Lockbox:reset_import_stats()
  return self.operations:lockbox_reset_import_stats(self.handle)
end

function Lockbox:page_inspection()
  return self.operations:lockbox_page_inspection(self.handle)
end

function Lockbox:recovery_report()
  return self.operations:lockbox_recovery_report(self.handle)
end

function Lockbox:recovery_report_render(verbose, max_entries)
  return self.operations:lockbox_recovery_report_render(self.handle, verbose, max_entries)
end

function Lockbox:storage_len()
  return self.operations:lockbox_storage_len(self.handle)
end

function Lockbox:set_workload_profile(profile)
  return self.operations:lockbox_set_workload_profile(self.handle, profile)
end

function Lockbox:set_worker_policy(mode, jobs)
  return self.operations:lockbox_set_worker_policy(self.handle, mode, jobs)
end

function Lockbox:runtime_options()
  return self.operations:lockbox_runtime_options(self.handle)
end

function Lockbox:commit()
  return self.operations:lockbox_commit(self.handle)
end

function Lockbox:create_dir(path, create_parents)
  return self.operations:lockbox_create_dir(self.handle, path, create_parents)
end

function Lockbox:delete(path)
  return self.operations:lockbox_delete(self.handle, path)
end

function Lockbox:remove_dir(path, recursive)
  return self.operations:lockbox_remove_dir(self.handle, path, recursive)
end

function Lockbox:create_parent_dirs(path)
  return self.operations:lockbox_create_parent_dirs(self.handle, path)
end

function Lockbox:rename(from, to)
  return self.operations:lockbox_rename(self.handle, from, to)
end

function Lockbox:list(path, recursive)
  return self.operations:lockbox_list(self.handle, path, recursive)
end

function Lockbox:list_with_options(path, glob, recursive, include_files, include_symlinks, include_directories, limit)
  return self.operations:lockbox_list_with_options(self.handle, path, glob, recursive, include_files, include_symlinks, include_directories, limit)
end

function Lockbox:stat(path)
  return self.operations:lockbox_stat(self.handle, path)
end

function Lockbox:set_variable(name, value, secret)
  return self.operations:lockbox_set_variable(self.handle, name, value, secret)
end

function Lockbox:get_variable(name)
  return self.operations:lockbox_get_variable(self.handle, name)
end

function Lockbox:delete_variable(name)
  return self.operations:lockbox_delete_variable(self.handle, name)
end

function Lockbox:move_variables(moves_proto)
  return self.operations:lockbox_move_variables(self.handle, moves_proto)
end

function Lockbox:list_variables()
  return self.operations:lockbox_list_variables(self.handle)
end

function Lockbox:variable_sensitivity(name)
  return self.operations:lockbox_variable_sensitivity(self.handle, name)
end

function Lockbox:add_symlink(path, target, replace)
  return self.operations:lockbox_add_symlink(self.handle, path, target, replace)
end

function Lockbox:get_symlink_target(path)
  return self.operations:lockbox_get_symlink_target(self.handle, path)
end

function Lockbox:id()
  return self.operations:lockbox_id(self.handle)
end

function Lockbox:exists(path)
  return self.operations:lockbox_exists(self.handle, path)
end

function Lockbox:is_dir(path)
  return self.operations:lockbox_is_dir(self.handle, path)
end

function Lockbox:permissions(path)
  return self.operations:lockbox_permissions(self.handle, path)
end

function Lockbox:set_permissions(path, permissions)
  return self.operations:lockbox_set_permissions(self.handle, path, permissions)
end

function Lockbox:read_range(path, offset, len)
  return self.operations:lockbox_read_range(self.handle, path, offset, len)
end

function Lockbox:add_password(password)
  return self.operations:lockbox_add_password(self.handle, password)
end

function Lockbox:add_contact(contact, name)
  return self.operations:lockbox_add_contact(self.handle, contact.handle, name)
end

function Lockbox:delete_key(id)
  return self.operations:lockbox_delete_key(self.handle, id)
end

function Lockbox:list_key_slots()
  return self.operations:lockbox_list_key_slots(self.handle)
end

function Lockbox:set_owner_signing_key(key)
  return self.operations:lockbox_set_owner_signing_key(self.handle, key.handle)
end

function Lockbox:owner_inspection()
  return self.operations:lockbox_owner_inspection(self.handle)
end

function Lockbox:define_form(alias, name, description, fields_proto)
  return self.operations:lockbox_define_form(self.handle, alias, name, description, fields_proto)
end

function Lockbox:list_form_definitions()
  return self.operations:lockbox_list_form_definitions(self.handle)
end

function Lockbox:resolve_form(reference)
  return self.operations:lockbox_resolve_form(self.handle, reference)
end

function Lockbox:list_form_revisions(type_id)
  return self.operations:lockbox_list_form_revisions(self.handle, type_id)
end

function Lockbox:create_form_record(path, type_reference, name)
  return self.operations:lockbox_create_form_record(self.handle, path, type_reference, name)
end

function Lockbox:set_form_field(path, field, value, secret)
  return self.operations:lockbox_set_form_field(self.handle, path, field, value, secret)
end

function Lockbox:list_form_records()
  return self.operations:lockbox_list_form_records(self.handle)
end

function Lockbox:get_form_record(path)
  return self.operations:lockbox_get_form_record(self.handle, path)
end

function Lockbox:delete_form_record(path)
  return self.operations:lockbox_delete_form_record(self.handle, path)
end

function Lockbox:move_form_records(moves_proto)
  return self.operations:lockbox_move_form_records(self.handle, moves_proto)
end

function Lockbox:get_form_field(path, field)
  return self.operations:lockbox_get_form_field(self.handle, path, field)
end

function Lockbox:to_bytes()
  return self.operations:lockbox_to_bytes(self.handle)
end

function Lockbox:free()
  self.operations:lockbox_free(self.handle)
  self.handle = nil
end

function ContactKeyPair:public()
  return self.operations:key_contact_public(self.handle)
end

function ContactKeyPair:private()
  return self.operations:key_contact_private(self.handle)
end

function ContactKeyPair:free()
  self.operations:key_contact_free(self.handle)
  self.handle = nil
end

function ContactKeyPair:decrypt(wrapped)
  return self.operations:key_contact_decrypt(self.handle, wrapped.handle)
end

function ContactPublicKey:public_free()
  self.operations:key_contact_public_free(self.handle)
  self.handle = nil
end

function ContactPublicKey:encrypt(content_key)
  return WrappedContactKey.new(self.operations, self.operations:key_contact_encrypt(self.handle, content_key))
end

function WrappedContactKey:public()
  return self.operations:key_contact_wrapped_public(self.handle)
end

function WrappedContactKey:ciphertext()
  return self.operations:key_contact_wrapped_ciphertext(self.handle)
end

function WrappedContactKey:encrypted()
  return self.operations:key_contact_wrapped_encrypted(self.handle)
end

function WrappedContactKey:free()
  self.operations:key_contact_wrapped_free(self.handle)
  self.handle = nil
end

function SigningKeyPair:public()
  return self.operations:key_signing_public(self.handle)
end

function SigningKeyPair:private()
  return self.operations:key_signing_private(self.handle)
end

function SigningKeyPair:free()
  self.operations:key_signing_free(self.handle)
  self.handle = nil
end

function SigningPublicKey:public_free()
  self.operations:key_signing_public_free(self.handle)
  self.handle = nil
end

function VaultDirectory:root()
  return self.operations:vault_directory_root(self.handle)
end

function VaultDirectory:structure_version()
  return self.operations:vault_directory_structure_version(self.handle)
end

function VaultDirectory:list_private_keys()
  return self.operations:vault_directory_list_private_keys(self.handle)
end

function VaultDirectory:list_private_key_names()
  return self.operations:vault_directory_list_private_key_names(self.handle)
end

function VaultDirectory:list_contact_names()
  return self.operations:vault_directory_list_contact_names(self.handle)
end

function VaultDirectory:list_form_aliases()
  return self.operations:vault_directory_list_form_aliases(self.handle)
end

function VaultDirectory:private_key_exists(name)
  return self.operations:vault_directory_private_key_exists(self.handle, name)
end

function VaultDirectory:delete_private_key(name)
  return self.operations:vault_directory_delete_private_key(self.handle, name)
end

function VaultDirectory:store_private_key(name, key)
  return self.operations:vault_directory_store_private_key(self.handle, name, key.handle)
end

function VaultDirectory:load_private_key(name)
  return ContactKeyPair.new(self.operations, self.operations:vault_directory_load_private_key(self.handle, name))
end

function VaultDirectory:load_private_key_generation(name, index)
  return ContactKeyPair.new(self.operations, self.operations:vault_directory_load_private_key_generation(self.handle, name, index))
end

function VaultDirectory:store_contact(name, key)
  return self.operations:vault_directory_store_contact(self.handle, name, key.handle)
end

function VaultDirectory:load_contact(name)
  return ContactPublicKey.new(self.operations, self.operations:vault_directory_load_contact(self.handle, name))
end

function VaultDirectory:contact_exists(name)
  return self.operations:vault_directory_contact_exists(self.handle, name)
end

function VaultDirectory:delete_contact(name)
  return self.operations:vault_directory_delete_contact(self.handle, name)
end

function VaultDirectory:list_contacts()
  return self.operations:vault_directory_list_contacts(self.handle)
end

function VaultDirectory:store_profile_email(name, email)
  return self.operations:vault_directory_store_profile_email(self.handle, name, email)
end

function VaultDirectory:profile_email(name)
  return self.operations:vault_directory_profile_email(self.handle, name)
end

function VaultDirectory:store_backup(id, bytes)
  return self.operations:vault_directory_store_backup(self.handle, id, bytes)
end

function VaultDirectory:load_backup(id)
  return self.operations:vault_directory_load_backup(self.handle, id)
end

function VaultDirectory:backup_count()
  return self.operations:vault_directory_backup_count(self.handle)
end

function VaultDirectory:restore_private_key(name, key, signing_key, overwrite)
  return self.operations:vault_directory_restore_private_key(self.handle, name, key.handle, signing_key.handle, overwrite)
end

function VaultDirectory:load_owner_signing_key(name)
  return SigningKeyPair.new(self.operations, self.operations:vault_directory_load_owner_signing_key(self.handle, name))
end

function VaultDirectory:load_owner_signing_key_generation(name, index)
  return SigningKeyPair.new(self.operations, self.operations:vault_directory_load_owner_signing_key_generation(self.handle, name, index))
end

function VaultDirectory:store_contact_signing_key(name, key)
  return self.operations:vault_directory_store_contact_signing_key(self.handle, name, key.handle)
end

function VaultDirectory:load_contact_signing_key(name)
  return SigningPublicKey.new(self.operations, self.operations:vault_directory_load_contact_signing_key(self.handle, name))
end

function VaultDirectory:list_profile_generations(name)
  return self.operations:vault_directory_list_profile_generations(self.handle, name)
end

function VaultDirectory:rotate_private_key(name)
  return self.operations:vault_directory_rotate_private_key(self.handle, name)
end

function VaultDirectory:remember_lockbox(id, path)
  return self.operations:vault_directory_remember_lockbox(self.handle, id, path)
end

function VaultDirectory:list_known_lockboxes()
  return self.operations:vault_directory_list_known_lockboxes(self.handle)
end

function VaultDirectory:forget_lockbox(path)
  return self.operations:vault_directory_forget_lockbox(self.handle, path)
end

function VaultDirectory:remember_access_slot_label(id, slot_id, name)
  return self.operations:vault_directory_remember_access_slot_label(self.handle, id, slot_id, name)
end

function VaultDirectory:list_access_slot_labels(id)
  return self.operations:vault_directory_list_access_slot_labels(self.handle, id)
end

function VaultDirectory:find_access_slot_labels(id, name)
  return self.operations:vault_directory_find_access_slot_labels(self.handle, id, name)
end

function VaultDirectory:forget_access_slot_label(id, slot_id)
  return self.operations:vault_directory_forget_access_slot_label(self.handle, id, slot_id)
end

function VaultDirectory:define_form(alias, name, description, fields_proto)
  return self.operations:vault_directory_define_form(self.handle, alias, name, description, fields_proto)
end

function VaultDirectory:resolve_form(reference)
  return self.operations:vault_directory_resolve_form(self.handle, reference)
end

function VaultDirectory:list_forms()
  return self.operations:vault_directory_list_forms(self.handle)
end

function VaultDirectory:list_form_revisions(type_id)
  return self.operations:vault_directory_list_form_revisions(self.handle, type_id)
end

function VaultDirectory:seed_forms()
  return self.operations:vault_directory_seed_forms(self.handle)
end

function VaultDirectory:remember_password(id, password)
  return self.operations:vault_directory_remember_password(self.handle, id, password)
end

function VaultDirectory:remembered_password(id)
  return self.operations:vault_directory_remembered_password(self.handle, id)
end

function VaultDirectory:free()
  self.operations:vault_directory_free(self.handle)
  self.handle = nil
end

function ReadOnlyVaultDirectory:list_profile_names()
  return self.operations:vault_read_only_list_profile_names(self.handle)
end

function ReadOnlyVaultDirectory:list_contact_names()
  return self.operations:vault_read_only_list_contact_names(self.handle)
end

function ReadOnlyVaultDirectory:list_form_aliases()
  return self.operations:vault_read_only_list_form_aliases(self.handle)
end

function ReadOnlyVaultDirectory:list_known_lockboxes()
  return self.operations:vault_read_only_list_known_lockboxes(self.handle)
end

function ReadOnlyVaultDirectory:free()
  self.operations:vault_read_only_free(self.handle)
  self.handle = nil
end

function Agent:is_running()
  return self.operations:vault_is_running()
end

function Agent:forget_all()
  return self.operations:vault_forget_all()
end

function Agent:serve()
  return self.operations:vault_agent_serve()
end

function Agent:verify_transport()
  return self.operations:vault_agent_verify_transport()
end

function Agent:get(id)
  return self.operations:vault_agent_get(id)
end

function Agent:put(id, key)
  return self.operations:vault_agent_put(id, key)
end

function Agent:forget(id)
  return self.operations:vault_agent_forget(id)
end

function Agent:stop()
  return self.operations:vault_agent_stop()
end

function Agent:start()
  return self.operations:vault_agent_start()
end

function Agent:list()
  return self.operations:vault_agent_list()
end

function Agent:sleep_support()
  return self.operations:vault_agent_sleep_support()
end

function Agent:get_vault_unlock_key(vault_id)
  return self.operations:vault_agent_get_vault_unlock_key(vault_id)
end

function Agent:put_vault_unlock_key(vault_id, key, ttl_seconds)
  return self.operations:vault_agent_put_vault_unlock_key(vault_id, key, ttl_seconds)
end

function Agent:forget_vault_unlock_key(vault_id)
  return self.operations:vault_agent_forget_vault_unlock_key(vault_id)
end

function Agent:get_owner_signing_key(vault_id, profile)
  return SigningKeyPair.new(self.operations, self.operations:vault_agent_get_owner_signing_key(vault_id, profile))
end

function Agent:put_owner_signing_key(vault_id, profile, key, ttl_seconds)
  return self.operations:vault_agent_put_owner_signing_key(vault_id, profile, key.handle, ttl_seconds)
end

function Agent:forget_owner_signing_key(vault_id, profile)
  return self.operations:vault_agent_forget_owner_signing_key(vault_id, profile)
end

function Agent:begin_activity(kind)
  return AgentActivity.new(self.operations, self.operations:vault_agent_begin_activity(kind))
end

function Agent:end_activity(handle)
  return self.operations:vault_agent_end_activity(handle.handle)
end

function Platform:status()
  return self.operations:vault_platform_status()
end

function Platform:set_scope(scope)
  return self.operations:vault_platform_set_scope(scope)
end

function Platform:forget_password()
  return self.operations:vault_platform_forget_password()
end

function Platform:put_password(password)
  return self.operations:vault_platform_put_password(password)
end

function Platform:enable()
  return self.operations:vault_platform_enable()
end

function Platform:disable()
  return self.operations:vault_platform_disable()
end

function Platform:disabled()
  return self.operations:vault_platform_disabled()
end

function Platform:get_password()
  return self.operations:vault_platform_get_password()
end

function LocalVault:create_lockbox_password(path, password)
  return Lockbox.new(self.operations, self.operations:vault_create_lockbox_password(self.handle, path, password))
end

function LocalVault:open_lockbox_password(path, password)
  return Lockbox.new(self.operations, self.operations:vault_open_lockbox_password(self.handle, path, password))
end

function LocalVault:create_lockbox_content_key(path, content_key, signing_key)
  return Lockbox.new(self.operations, self.operations:vault_create_lockbox_content_key(self.handle, path, content_key, signing_key.handle))
end

function LocalVault:create_lockbox_contact(path, contact, name, signing_key)
  return Lockbox.new(self.operations, self.operations:vault_create_lockbox_contact(self.handle, path, contact.handle, name, signing_key.handle))
end

function LocalVault:open_lockbox_content_key(path, content_key, signing_key)
  return Lockbox.new(self.operations, self.operations:vault_open_lockbox_content_key(self.handle, path, content_key, signing_key.handle))
end

function LocalVault:cache_lockbox_password(path, password, ttl_seconds)
  return self.operations:vault_cache_lockbox_password(self.handle, path, password, ttl_seconds)
end

function LocalVault:close_lockbox(path)
  return self.operations:vault_close_lockbox(self.handle, path)
end

function LocalVault:close_all()
  return self.operations:vault_close_all(self.handle)
end

function LocalVault:free()
  self.operations:vault_free(self.handle)
  self.handle = nil
end

local M = {
  Vault = Vault, Models = Models, native = native,
  Lockbox = Lockbox, ContactKeyPair = ContactKeyPair, ContactPublicKey = ContactPublicKey,
  WrappedContactKey = WrappedContactKey, SigningKeyPair = SigningKeyPair,
  SigningPublicKey = SigningPublicKey, VaultDirectory = VaultDirectory, ReadOnlyVaultDirectory = ReadOnlyVaultDirectory,
  Agent = Agent, AgentActivity = AgentActivity, Platform = Platform, LocalVault = LocalVault,
}
return M
