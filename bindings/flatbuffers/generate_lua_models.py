"""Generate the private, zero-dependency Lua FlatBuffer codec."""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "lua/revault_flatbuffers.lua"

tables = {}
current = None
for line in SCHEMA.read_text().splitlines():
    if match := re.match(r"table (\w+) \{", line.strip()):
        current = match.group(1); tables[current] = []
    elif current and line.strip() == "}": current = None
    elif current and (match := re.match(r"\s*(\w+):([^ ]+) \(id:", line)):
        tables[current].append(match.groups())

schema = []
for name, fields in tables.items():
    values = ", ".join("{'%s','%s'}" % field for field in fields)
    schema.append("  %s = {%s}" % (name, values))

template = r'''-- Generated private FlatBuffer codec. Do not edit.
local schema = {
@SCHEMA@
}

local function u16(bytes, position)
  assert(position >= 1 and position + 1 <= #bytes,
    string.format('invalid FlatBuffer u16 at byte %d of %d', position, #bytes))
  local a, b = bytes:byte(position, position + 1)
  return a + b * 256
end

local function u32(bytes, position)
  assert(position >= 1 and position + 3 <= #bytes,
    string.format('invalid FlatBuffer u32 at byte %d of %d', position, #bytes))
  local a, b, c, d = bytes:byte(position, position + 3)
  return a + b * 256 + c * 65536 + d * 16777216
end

local function u64(bytes, position)
  return u32(bytes, position) + u32(bytes, position + 4) * 4294967296
end

local function i32(bytes, position)
  local value = u32(bytes, position)
  return value >= 2147483648 and value - 4294967296 or value
end

local decode_table
local function field_location(bytes, position, index)
  local vtable = position - i32(bytes, position)
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
'''
OUTPUT.write_text(template.replace("@SCHEMA@", ",\n".join(schema)))
