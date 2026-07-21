"""Generate Ruby domain values and a private zero-dependency FlatBuffer codec."""

from pathlib import Path
import re
from model_docs import DESCRIPTIONS, field_description, wrapper_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "ruby/lib/revault/domain_models.rb"

tables: dict[str, list[tuple[str, str]]] = {}
current = None
for line in SCHEMA.read_text().splitlines():
    if match := re.match(r"table (\w+) \{", line.strip()):
        current = match.group(1); tables[current] = []
    elif current and line.strip() == "}": current = None
    elif current and (match := re.match(r"\s*(\w+):([^ ]+) \(id:", line)):
        tables[current].append(match.groups())

def public_field(field: str) -> str:
    return "form_alias" if field == "alias" else field

lines = ["# frozen_string_literal: true", "", "module Revault", "  module Bindings"]
for name, fields in tables.items():
    doc = DESCRIPTIONS.get(name, wrapper_description(name))
    lines += [f"    # {doc}", f"    class {name}"]
    for field, _ in fields:
        field_doc = field_description(name, field) if name in DESCRIPTIONS else wrapper_description(name)
        lines += [f"      # {field_doc}", f"      attr_reader :{public_field(field)}"]
    args = ", ".join(f"{public_field(field)}: {('false' if value == 'bool' else '0' if value in {'uint', 'ulong', 'ubyte'} or value.endswith('LockboxEntryKind') else '[]' if value.startswith('[') else "''" if value == 'string' else 'nil')}" for field, value in fields)
    assigns = "; ".join(f"@{public_field(field)} = {public_field(field)}" for field, _ in fields)
    lines += ["      # Creates a domain value from named fields.", f"      def initialize({args}) = ({assigns})", "    end", ""]
lines += ["  end", "", "  module Internal", "    # FlatBuffer reader and input encoder used only by the FFI layer.", "    module DomainCodec", "      module_function", ""]
schema_items = []
for name, fields in tables.items():
    entries = []
    for field, value in fields:
        entries.append(f"[:{field}, {value!r}]")
    schema_items.append(f"      {name!r} => [{', '.join(entries)}]")
lines += ["    SCHEMA = {", ",\n".join(schema_items), "    }.freeze", ""]
lines += [
    "    def decode(name, bytes)", "      table(name, bytes.b, bytes.unpack1('V'))", "    end", "",
    "    def table(name, bytes, position)", "      values = {}", "      SCHEMA.fetch(name).each_with_index do |(field, type), index|",
    "        location = field_location(bytes, position, index)", "        public_field = field == :alias ? :form_alias : field", "        values[public_field] = read_value(bytes, location, type)", "      end",
    "      Bindings.const_get(name).new(**values)", "    end", "",
    "    def field_location(bytes, table_position, index)", "      vtable = table_position - bytes.unpack1('l<', offset: table_position)",
    "      vtable_length = bytes.unpack1('v', offset: vtable)", "      entry = vtable + 4 + index * 2", "      return nil if entry + 2 > vtable + vtable_length",
    "      offset = bytes.unpack1('v', offset: entry)", "      offset.zero? ? nil : table_position + offset", "    end", "",
    "    def read_value(bytes, location, type)", "      return default_value(type) unless location",
    "      if type.start_with?('[')", "        read_vector(bytes, location, type[1...-1])", "      elsif SCHEMA.key?(type.split('.').last)",
    "        table(type.split('.').last, bytes, location + bytes.unpack1('V', offset: location))", "      else", "        read_scalar(bytes, location, type)", "      end", "    end", "",
    "    def read_vector(bytes, location, type)", "      vector = location + bytes.unpack1('V', offset: location)", "      length = bytes.unpack1('V', offset: vector)", "      start = vector + 4",
    "      return bytes.byteslice(start, length) if type == 'ubyte'", "      Array.new(length) do |index|", "        element = start + index * 4",
    "        if type == 'string'", "          read_string(bytes, element)", "        elsif SCHEMA.key?(type.split('.').last)", "          table(type.split('.').last, bytes, element + bytes.unpack1('V', offset: element))",
    "        else", "          read_scalar(bytes, element, type)", "        end", "      end", "    end", "",
    "    def read_scalar(bytes, location, type)", "      case type", "      when 'string' then read_string(bytes, location)", "      when 'bool', 'ubyte' then bytes.getbyte(location) != 0",
    "      when 'uint' then bytes.unpack1('V', offset: location)", "      when 'ulong' then bytes.unpack1('Q<', offset: location)", "      else bytes.unpack1('V', offset: location)", "      end", "    end", "",
    "    def read_string(bytes, location)", "      value = location + bytes.unpack1('V', offset: location)", "      bytes.byteslice(value + 4, bytes.unpack1('V', offset: value)).force_encoding(Encoding::UTF_8)", "    end", "",
    "    def default_value(type)", "      return [] if type.start_with?('[')", "      return '' if type == 'string'", "      return false if type == 'bool'", "      return 0 if %w[uint ulong ubyte].include?(type) || type.end_with?('LockboxEntryKind')", "      nil", "    end", "",
    "    def encode_path_moves(values) = encode_table_vector(values, :path_move)", "    def encode_form_fields(values) = encode_table_vector(values, :form_field)", "",
    "    def encode_table_vector(values, kind)", "      bytes = [0].pack('V')", "      append_vtable(bytes, [4], 8)", "      root = align(bytes); bytes << [root - 4, 0].pack('V2')", "      patch_u32(bytes, 0, root)", "      vector = align(bytes); patch_u32(bytes, root + 4, vector - (root + 4)); bytes << [values.length].pack('V') << (\"\\0\" * (values.length * 4))",
    "      values.each_with_index do |value, index|", "        position = kind == :path_move ? append_path_move(bytes, value) : append_form_field(bytes, value)", "        element = vector + 4 + index * 4; patch_u32(bytes, element, position - element)", "      end", "      bytes", "    end", "",
    "    def append_path_move(bytes, value)", "      vtable = append_vtable(bytes, [4, 8], 12); table = align(bytes); bytes << [table - vtable, 0, 0].pack('V3')",
    "      append_string_field(bytes, table + 4, value.source); append_string_field(bytes, table + 8, value.destination); table", "    end", "",
    "    def append_form_field(bytes, value)", "      vtable = append_vtable(bytes, [4, 8, 12, 16], 20); table = align(bytes); bytes << [table - vtable, 0, 0, 0].pack('V4') << [value.required ? 1 : 0].pack('C') << \"\\0\\0\\0\"",
    "      append_string_field(bytes, table + 4, value.id); append_string_field(bytes, table + 8, value.label); append_string_field(bytes, table + 12, value.kind); table", "    end", "",
    "    def append_vtable(bytes, offsets, object_size)", "      align(bytes, 2); position = bytes.bytesize; bytes << [4 + offsets.length * 2, object_size, *offsets].pack('v*'); position", "    end", "",
    "    def append_string_field(bytes, field, value)", "      target = align(bytes); patch_u32(bytes, field, target - field); bytes << [value.bytesize].pack('V') << value.b << \"\\0\"", "    end", "",
    "    def align(bytes, alignment = 4)", "      bytes << \"\\0\" until (bytes.bytesize % alignment).zero?; bytes.bytesize", "    end", "",
    "    def patch_u32(bytes, offset, value) = bytes[offset, 4] = [value].pack('V')", "    end", "  end", "end", "",
]
OUTPUT.write_text("\n".join(lines))
