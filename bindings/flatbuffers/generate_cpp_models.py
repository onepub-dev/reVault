"""Generate C++ domain structs and private FlatBuffers conversions."""

from pathlib import Path
import re
from model_docs import description, field_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "cpp/domain_models.hpp"
CODEC_OUTPUT = ROOT / "cpp/domain_codec.hpp"

SCALARS = {"string": "std::string", "bool": "bool", "uint": "std::uint32_t", "ulong": "std::uint64_t", "ubyte": "std::uint8_t"}

def ctype(value: str) -> str:
    if value.startswith("["):
        inner = value[1:-1].split(".")[-1]
        return f"std::vector<{SCALARS.get(inner, inner)}>"
    raw = value.split(".")[-1]
    return "LockboxEntryKind" if raw == "LockboxEntryKind" else SCALARS.get(raw, raw)

def public_field(name: str) -> str:
    return "is_signed" if name == "signed" else name

def internal_field(name: str) -> str:
    return "signed_" if name == "signed" else name

tables = {}
current = None
for line in SCHEMA.read_text().splitlines():
    if match := re.match(r"table (\w+) \{", line.strip()):
        current = match.group(1); tables[current] = []
    elif current and line.strip() == "}": current = None
    elif current and (match := re.match(r"\s*(\w+):([^ ]+) \(id:", line)):
        tables[current].append(match.groups())

wrappers = {name for name in tables if name.endswith("List") or name.startswith("Optional")}
wrappers.add("StringValue")
base = [name for name in tables if name not in wrappers]
list_types = {
    "StreamChunkList": "StreamChunk", "PageInspectionList": "PageInspection", "LockboxEntryList": "LockboxEntry",
    "VariableList": "Variable", "KeySlotList": "KeySlot", "FormDefinitionList": "FormDefinition",
    "FormRecordList": "FormRecord", "ContactList": "Contact", "KnownLockboxList": "KnownLockbox",
    "AccessSlotLabelList": "AccessSlotLabel", "AgentEntryList": "AgentEntry", "ProfileHistoryList": "ProfileHistory",
}

lines = ["#pragma once", "", "#include <cstdint>", "#include <memory>", "#include <optional>", "#include <span>", "#include <string>", "#include <vector>", '#include "generated/flatbuffers/revault_bindings_generated.h"', "", "namespace revault::bindings {", "", "/** Identifies the filesystem object stored at a lockbox path. */", "enum class LockboxEntryKind { unspecified, file, symlink, directory };", ""]
for name in base:
    lines += [f"/** {description(name)} */", f"struct {name} {{"]
    for field, value in tables[name]:
        lines.append(f"  /** {field_description(name, field)} */")
        lines.append(f"  {ctype(value)} {public_field(field)}{{}};")
    lines += ["};", ""]
for wrapper, item in list_types.items():
    lines.append(f"using {wrapper} = std::vector<{item}>;")
lines += ["using StringList = std::vector<std::string>;", "using OptionalLockboxEntry = std::optional<LockboxEntry>;", "using OptionalFormRecord = std::optional<FormRecord>;", "using OptionalFormValue = std::optional<FormValue>;", "using OptionalString = std::optional<std::string>;", "using PathMoveList = std::vector<PathMove>;", "using FormFieldList = std::vector<FormField>;", "", "}  // namespace revault::bindings", "", "namespace revault::detail {", ""]

for name in base:
    args = []
    for field, value in tables[name]:
        raw = value.split(".")[-1]
        if value.startswith("["):
            inner = value[1:-1].split(".")[-1]
            if inner in SCALARS:
                expr = f"value.{field}"
            else:
                expr = f"convert_vector(value.{field})"
        elif raw == "LockboxEntryKind": expr = f"static_cast<bindings::LockboxEntryKind>(value.{field})"
        elif raw in SCALARS: expr = f"value.{field}"
        else: expr = f"convert(*value.{field})"
        args.append(expr)
    lines += [f"inline bindings::{name} convert(const internal::{name}T& value);" ]
lines += ["", "template <typename T, typename U>", "std::vector<T> convert_owned_vector(const std::vector<std::unique_ptr<U>>& values) {", "  std::vector<T> result; result.reserve(values.size());", "  for (const auto& value : values) result.push_back(convert(*value));", "  return result;", "}", ""]
for name in base:
    args=[]
    for field,value in tables[name]:
        raw=value.split(".")[-1]
        if value.startswith("["):
            inner=value[1:-1].split(".")[-1]
            expr=f"value.{internal_field(field)}" if inner in SCALARS else f"convert_owned_vector<bindings::{inner}>(value.{internal_field(field)})"
        elif raw=="LockboxEntryKind": expr=f"static_cast<bindings::LockboxEntryKind>(value.{internal_field(field)})"
        elif raw in SCALARS: expr=f"value.{internal_field(field)}"
        else: expr=f"convert(*value.{internal_field(field)})"
        args.append(expr)
    lines += [f"inline bindings::{name} convert(const internal::{name}T& value) {{", f"  return {{{', '.join(args)}}};", "}"]

lines += ["", "template <typename T> T decode(std::span<const std::uint8_t> bytes);", ""]
for name in base:
    lines += [f"template <> inline bindings::{name} decode<bindings::{name}>(std::span<const std::uint8_t> bytes) {{", f"  return convert(*flatbuffers::GetRoot<internal::{name}>(bytes.data())->UnPack());", "}"]
for wrapper,item in list_types.items():
    field="entries" if wrapper=="LockboxEntryList" else "values"
    lines += [f"template <> inline bindings::{wrapper} decode<bindings::{wrapper}>(std::span<const std::uint8_t> bytes) {{", f"  auto value = flatbuffers::GetRoot<internal::{wrapper}>(bytes.data())->UnPack();", f"  return convert_owned_vector<bindings::{item}>(value->{field});", "}"]
lines += [
    "template <> inline bindings::StringList decode<bindings::StringList>(std::span<const std::uint8_t> bytes) { return flatbuffers::GetRoot<internal::StringList>(bytes.data())->UnPack()->values; }",
    "template <> inline bindings::OptionalLockboxEntry decode<bindings::OptionalLockboxEntry>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalLockboxEntry>(bytes.data())->UnPack(); return value->value ? std::optional(convert(*value->value)) : std::nullopt; }",
    "template <> inline bindings::OptionalFormRecord decode<bindings::OptionalFormRecord>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalFormRecord>(bytes.data())->UnPack(); return value->value ? std::optional(convert(*value->value)) : std::nullopt; }",
    "template <> inline bindings::OptionalFormValue decode<bindings::OptionalFormValue>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalFormValue>(bytes.data())->UnPack(); return value->value ? std::optional(convert(*value->value)) : std::nullopt; }",
    "template <> inline bindings::OptionalString decode<bindings::OptionalString>(std::span<const std::uint8_t> bytes) { auto value = flatbuffers::GetRoot<internal::OptionalString>(bytes.data())->UnPack(); return value->present ? std::optional(value->value) : std::nullopt; }",
    "", "inline std::string encode_moves(const bindings::PathMoveList& values) {", "  internal::PathMoveListT transport;", "  for (const auto& value : values) { auto item = std::make_unique<internal::PathMoveT>(); item->source = value.source; item->destination = value.destination; transport.values.push_back(std::move(item)); }", "  flatbuffers::FlatBufferBuilder builder; builder.Finish(internal::PathMoveList::Pack(builder, &transport));", "  return {reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize()};", "}",
    "inline std::string encode_fields(const bindings::FormFieldList& values) {", "  internal::FormFieldListT transport;", "  for (const auto& value : values) { auto item = std::make_unique<internal::FormFieldT>(); item->id = value.id; item->label = value.label; item->kind = value.kind; item->required = value.required; transport.values.push_back(std::move(item)); }", "  flatbuffers::FlatBufferBuilder builder; builder.Finish(internal::FormFieldList::Pack(builder, &transport));", "  return {reinterpret_cast<const char*>(builder.GetBufferPointer()), builder.GetSize()};", "}", "", "}  // namespace revault::detail", "",
]
content = "\n".join(lines)
public, private = content.split("namespace revault::detail {", 1)
public = public.replace('#include "generated/flatbuffers/revault_bindings_generated.h"\n', "")
result_types = [*base, *list_types, "StringList", "OptionalLockboxEntry", "OptionalFormRecord", "OptionalFormValue", "OptionalString"]
declarations = ["namespace revault::detail {", "", "template <typename T> T decode(std::span<const std::uint8_t> bytes);", ""]
for name in result_types:
    declarations.append(f"template <> bindings::{name} decode<bindings::{name}>(std::span<const std::uint8_t> bytes);")
declarations += ["", "std::string encode_moves(const bindings::PathMoveList& values);", "std::string encode_fields(const bindings::FormFieldList& values);", "", "}  // namespace revault::detail", ""]
OUTPUT.write_text(public + "\n".join(declarations))
private = private.replace("template <> inline ", "template <> ").replace("inline std::string encode_", "std::string encode_")
CODEC_OUTPUT.write_text("\n".join([
    "#pragma once", "", '#include "domain_models.hpp"',
    '#include "generated/flatbuffers/revault_bindings_generated.h"', "",
    "namespace revault::detail {", private,
]))
