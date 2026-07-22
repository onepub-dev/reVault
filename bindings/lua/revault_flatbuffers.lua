-- Generated private FlatBuffer codec. Do not edit.
local schema = {
  LockboxEntry = {{'path','string'}, {'kind','revault.internal.LockboxEntryKind'}, {'length','ulong'}, {'permissions','uint'}},
  LockboxEntryList = {{'entries','[revault.internal.LockboxEntry]'}},
  OptionalLockboxEntry = {{'value','revault.internal.LockboxEntry'}},
  StringList = {{'values','[string]'}},
  PathMove = {{'source','string'}, {'destination','string'}},
  PathMoveList = {{'values','[revault.internal.PathMove]'}},
  FormField = {{'id','string'}, {'label','string'}, {'kind','string'}, {'required','bool'}},
  FormFieldList = {{'values','[revault.internal.FormField]'}},
  FormDefinition = {{'type_id','string'}, {'alias','string'}, {'revision','uint'}, {'name','string'}, {'description','string'}, {'fields','[revault.internal.FormField]'}},
  FormDefinitionList = {{'values','[revault.internal.FormDefinition]'}},
  FormValue = {{'field_id','string'}, {'label','string'}, {'kind','string'}, {'value','string'}, {'secret','bool'}},
  FormRecord = {{'path','string'}, {'name','string'}, {'type_id','string'}, {'definition_alias','string'}, {'definition_revision','uint'}, {'values','[revault.internal.FormValue]'}},
  FormRecordList = {{'values','[revault.internal.FormRecord]'}},
  OptionalFormRecord = {{'value','revault.internal.FormRecord'}},
  OptionalFormValue = {{'value','revault.internal.FormValue'}},
  RecoveryReport = {{'intact_files','[revault.internal.LockboxEntry]'}, {'intact_file_count','ulong'}, {'partial_files','ulong'}, {'corrupt_records','ulong'}, {'toc_recovered','bool'}, {'variables_recovered','bool'}, {'variable_count','ulong'}, {'forms_recovered','bool'}, {'form_definition_count','ulong'}, {'form_record_count','ulong'}},
  KeySlot = {{'id','ulong'}, {'protection','string'}, {'algorithm','string'}},
  KeySlotList = {{'values','[revault.internal.KeySlot]'}},
  CacheStats = {{'limit_bytes','ulong'}, {'used_bytes','ulong'}, {'entries','ulong'}, {'hits','ulong'}, {'misses','ulong'}},
  ImportStats = {{'host_stat_nanos','string'}, {'host_read_nanos','string'}, {'frame_prepare_nanos','string'}, {'page_write_nanos','string'}},
  PageObject = {{'id','ulong'}, {'kind','string'}, {'payload_len','ulong'}},
  PageInspection = {{'offset','ulong'}, {'page_id','ulong'}, {'sequence','ulong'}, {'page_size','ulong'}, {'encrypted_body_len','ulong'}, {'unused_bytes','ulong'}, {'object_count','ulong'}, {'objects','[revault.internal.PageObject]'}},
  PageInspectionList = {{'values','[revault.internal.PageInspection]'}},
  FileInspection = {{'lockbox_id','[ubyte]'}, {'header_readable','bool'}, {'key_directory_generation','ulong'}, {'key_directory_copy_count','ulong'}, {'owner_signed','bool'}, {'key_slots','[revault.internal.KeySlot]'}},
  ProfileGeneration = {{'index','uint'}, {'status','string'}, {'contact_fingerprint','[ubyte]'}, {'created_at_unix_ms','ulong'}, {'retired_at_unix_ms','ulong'}, {'has_retired_at','bool'}},
  ProfileHistory = {{'name','string'}, {'active_generation','uint'}, {'generations','[revault.internal.ProfileGeneration]'}},
  KnownLockbox = {{'lockbox_id','[ubyte]'}, {'path','string'}, {'last_seen_unix_ms','ulong'}},
  KnownLockboxList = {{'values','[revault.internal.KnownLockbox]'}},
  AccessSlotLabel = {{'lockbox_id','[ubyte]'}, {'slot_id','ulong'}, {'name','string'}, {'updated_at_unix_ms','ulong'}},
  AccessSlotLabelList = {{'values','[revault.internal.AccessSlotLabel]'}},
  StreamChunk = {{'path','string'}, {'file_offset','ulong'}, {'length','ulong'}, {'physical_offset','ulong'}, {'sparse','bool'}, {'data','[ubyte]'}},
  StreamChunkList = {{'values','[revault.internal.StreamChunk]'}},
  RuntimeOptions = {{'workload_profile','string'}, {'worker_policy','string'}},
  Variable = {{'name','string'}, {'sensitivity','string'}},
  VariableList = {{'values','[revault.internal.Variable]'}},
  OptionalString = {{'present','bool'}, {'value','string'}},
  OwnerInspection = {{'signed','bool'}, {'fingerprint','string'}, {'has_fingerprint','bool'}},
  Contact = {{'name','string'}, {'key','[ubyte]'}},
  ContactList = {{'values','[revault.internal.Contact]'}},
  ProfileHistoryList = {{'values','[revault.internal.ProfileHistory]'}},
  AgentEntry = {{'id','string'}, {'path','string'}},
  AgentEntryList = {{'values','[revault.internal.AgentEntry]'}},
  SleepSupport = {{'suspend_notifications','bool'}, {'sleep_inhibition','bool'}, {'supported','bool'}},
  PlatformStatus = {{'supported','bool'}, {'disabled','bool'}, {'scope','string'}, {'backend','string'}, {'item','string'}},
  StringValue = {{'value','string'}},
  VaultBackupManifest = {{'format_version','uint'}, {'created_at_unix_ms','ulong'}, {'vault_file_name','string'}, {'vault_size','ulong'}, {'vault_sha256','string'}},
  ErrorDetails = {{'category','string'}, {'artifact_kind','string'}, {'found_version','uint'}, {'supported_version','uint'}, {'message','string'}, {'guidance','string'}}
}

local function u16(bytes, position)
  local a, b = bytes:byte(position, position + 1)
  return a + b * 256
end

local function u32(bytes, position)
  local a, b, c, d = bytes:byte(position, position + 3)
  return a + b * 256 + c * 65536 + d * 16777216
end

local function u64(bytes, position)
  return u32(bytes, position) + u32(bytes, position + 4) * 4294967296
end

local decode_table
local function field_location(bytes, position, index)
  local vtable = position - u32(bytes, position)
  local entry = vtable + 4 + index * 2
  if entry + 1 >= vtable + u16(bytes, vtable) then return nil end
  local offset = u16(bytes, entry)
  if offset == 0 then return nil end
  return position + offset
end

local function string_value(bytes, location)
  local value = location + u32(bytes, location)
  local length = u32(bytes, value)
  return bytes:sub(value + 4, value + 3 + length)
end

local function default_value(kind)
  if kind:sub(1, 1) == '[' then return {} end
  if kind == 'string' then return '' end
  if kind == 'bool' then return false end
  if schema[kind:match('[^.]+$')] then return nil end
  return 0
end

local function scalar(bytes, location, kind)
  if kind == 'string' then return string_value(bytes, location) end
  if kind == 'bool' then return bytes:byte(location) ~= 0 end
  if kind == 'ulong' then return u64(bytes, location) end
  if kind == 'ubyte' then return bytes:byte(location) end
  return u32(bytes, location)
end

local function vector(bytes, location, kind)
  local start = location + u32(bytes, location)
  local length = u32(bytes, start)
  if kind == 'ubyte' then return bytes:sub(start + 4, start + 3 + length) end
  local values = {}
  for index = 0, length - 1 do
    local element = start + 4 + index * 4
    local name = kind:match('[^.]+$')
    if schema[name] then
      values[index + 1] = decode_table(name, bytes, element + u32(bytes, element))
    elseif kind == 'string' then
      values[index + 1] = string_value(bytes, element)
    else
      values[index + 1] = scalar(bytes, element, kind)
    end
  end
  return values
end

decode_table = function(name, bytes, position)
  local value = {}
  for index, field in ipairs(assert(schema[name], 'unknown reVault domain type: ' .. name)) do
    local location = field_location(bytes, position, index - 1)
    local kind = field[2]
    if location == nil then
      value[field[1]] = default_value(kind)
    elseif kind:sub(1, 1) == '[' then
      value[field[1]] = vector(bytes, location, kind:sub(2, -2):match('[^.]+$'))
    elseif schema[kind:match('[^.]+$')] then
      value[field[1]] = decode_table(kind:match('[^.]+$'), bytes, location + u32(bytes, location))
    else
      value[field[1]] = scalar(bytes, location, kind)
    end
  end
  return value
end

local function put_u16(output, value)
  output[#output + 1] = value % 256
  output[#output + 1] = math.floor(value / 256) % 256
end

local function put_u32(output, value)
  for shift = 0, 3 do output[#output + 1] = math.floor(value / 256 ^ shift) % 256 end
end

local function patch_u32(output, position, value)
  for shift = 0, 3 do output[position + shift] = math.floor(value / 256 ^ shift) % 256 end
end

local function align(output, alignment)
  alignment = alignment or 4
  while #output % alignment ~= 0 do output[#output + 1] = 0 end
  return #output + 1
end

local function append_vtable(output, offsets, object_size)
  align(output, 2)
  local position = #output + 1
  put_u16(output, 4 + #offsets * 2); put_u16(output, object_size)
  for _, offset in ipairs(offsets) do put_u16(output, offset) end
  return position
end

local function append_string(output, field, value)
  local target = align(output)
  patch_u32(output, field, target - field)
  put_u32(output, #value)
  for index = 1, #value do output[#output + 1] = value:byte(index) end
  output[#output + 1] = 0
end

local function append_path_move(output, value)
  local vtable = append_vtable(output, {4, 8}, 12)
  local position = align(output)
  put_u32(output, position - vtable); put_u32(output, 0); put_u32(output, 0)
  append_string(output, position + 4, value.source or '')
  append_string(output, position + 8, value.destination or '')
  return position
end

local function append_form_field(output, value)
  local vtable = append_vtable(output, {4, 8, 12, 16}, 20)
  local position = align(output)
  put_u32(output, position - vtable); put_u32(output, 0); put_u32(output, 0); put_u32(output, 0)
  output[#output + 1] = value.required and 1 or 0
  output[#output + 1] = 0; output[#output + 1] = 0; output[#output + 1] = 0
  append_string(output, position + 4, value.id or '')
  append_string(output, position + 8, value.label or '')
  append_string(output, position + 12, value.kind or '')
  return position
end

local function bytes(output)
  local chunks = {}
  for start = 1, #output, 4096 do
    chunks[#chunks + 1] = string.char(unpack(output, start, math.min(start + 4095, #output)))
  end
  return table.concat(chunks)
end

local function encode_vector(values, append)
  local output = {0, 0, 0, 0}
  local vtable = append_vtable(output, {4}, 8)
  local root = align(output)
  put_u32(output, root - vtable); put_u32(output, 0); patch_u32(output, 1, root - 1)
  local vector_position = align(output)
  patch_u32(output, root + 4, vector_position - (root + 4))
  put_u32(output, #values)
  for _ = 1, #values do put_u32(output, 0) end
  for index, value in ipairs(values) do
    local position = append(output, value)
    local element = vector_position + 4 + (index - 1) * 4
    patch_u32(output, element, position - element)
  end
  return bytes(output)
end

return {
  decode = function(name, value) return decode_table(name, value, u32(value, 1) + 1) end,
  encode_path_moves = function(values) return encode_vector(values, append_path_move) end,
  encode_form_fields = function(values) return encode_vector(values, append_form_field) end,
}
