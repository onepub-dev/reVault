"""Generate lazy Java domain views over private FlatBuffers tables."""

from pathlib import Path
import re
from model_docs import description, field_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "java/src/com/onepub/revault"

SCALARS = {"string": "String", "bool": "boolean", "uint": "long", "ulong": "long", "ubyte": "byte"}


def camel(name: str) -> str:
    parts = name.split("_")
    return parts[0] + "".join(part.title() for part in parts[1:])


def cap(name: str) -> str:
    value = camel(name)
    return value[:1].upper() + value[1:]


def java_type(value: str) -> str:
    if value.startswith("["):
        inner = value[1:-1].split(".")[-1]
        return "byte[]" if inner == "ubyte" else f"java.util.List<{SCALARS.get(inner, inner)}>"
    value = value.split(".")[-1]
    return "LockboxEntryKind" if value == "LockboxEntryKind" else SCALARS.get(value, value)


tables = {}
current = None
for line in SCHEMA.read_text().splitlines():
    if match := re.match(r"table (\w+) \{", line.strip()):
        current = match.group(1)
        tables[current] = []
    elif current and line.strip() == "}":
        current = None
    elif current and (match := re.match(r"\s*(\w+):([^ ]+) \(id:", line)):
        tables[current].append(match.groups())

wrappers = {name for name in tables if name.endswith("List") or name.startswith("Optional")}
wrappers |= {"StringValue"}
base = [name for name in tables if name not in wrappers]

(OUTPUT / "LockboxEntryKind.java").write_text("""package com.onepub.revault;

/** Identifies the filesystem object stored at a lockbox path. */
public enum LockboxEntryKind {
  /** No recognized kind was reported. */ UNSPECIFIED,
  /** A regular file. */ FILE,
  /** A symbolic link. */ SYMLINK,
  /** A directory. */ DIRECTORY
}
""")

for name in base:
    fields = tables[name]
    lines = ["package com.onepub.revault;", "", f"/** {description(name)} */", f"public final class {name} {{", f"  private final com.onepub.revault.internal.{name} view;"]
    for field, value in fields:
        lines.append(f"  private final {java_type(value)} {camel(field)};")
    params = ", ".join(f"{java_type(value)} {camel(field)}" for field, value in fields)
    lines += ["", f"  /** Creates an application-owned {name} value. */", f"  public {name}({params}) {{", "    this.view = null;"]
    for field, _ in fields:
        lines.append(f"    this.{camel(field)} = {camel(field)};")
    lines += ["  }", "", f"  {name}(com.onepub.revault.internal.{name} view) {{", "    this.view = java.util.Objects.requireNonNull(view);"]
    for field, value in fields:
        dtype = java_type(value)
        default = "null" if not dtype in ("boolean", "long") else "false" if dtype == "boolean" else "0"
        lines.append(f"    this.{camel(field)} = {default};")
    lines += ["  }", ""]
    for field, value in fields:
        method, dtype = camel(field), java_type(value)
        raw = value.split(".")[-1]
        lines += [f"  /** {field_description(name, field)} */", f"  public {dtype} {method}() {{"]
        if value.startswith("["):
            inner = value[1:-1].split(".")[-1]
            if inner == "ubyte":
                lines += [f"    if (view == null) return {method}.clone();", f"    var result = new byte[view.{method}Length()];", f"    for (int index = 0; index < result.length; index++) result[index] = (byte) view.{method}(index);", "    return result;"]
            elif inner in SCALARS:
                lines += [f"    if (view == null) return {method};", f"    var result = new java.util.ArrayList<{SCALARS[inner]}>(view.{method}Length());", f"    for (int index = 0; index < view.{method}Length(); index++) result.add(view.{method}(index));", "    return java.util.List.copyOf(result);"]
            else:
                lines += [f"    if (view == null) return {method};", f"    var result = new java.util.ArrayList<{inner}>(view.{method}Length());", f"    for (int index = 0; index < view.{method}Length(); index++) result.add(new {inner}(view.{method}(index)));", "    return java.util.List.copyOf(result);"]
        elif raw == "LockboxEntryKind":
            lines.append(f"    return view == null ? {method} : LockboxEntryKind.values()[view.{method}()];")
        elif raw in SCALARS:
            suffix = " & 0xffffffffL" if raw == "uint" else ""
            lines.append(f"    return view == null ? {method} : view.{method}(){suffix};")
        else:
            lines.append(f"    return view == null ? {method} : new {raw}(view.{method}());")
        lines += ["  }", ""]
    lines += ["}", ""]
    (OUTPUT / f"{name}.java").write_text("\n".join(lines))

list_types = {
    "StreamChunkList": "StreamChunk", "PageInspectionList": "PageInspection",
    "LockboxEntryList": "LockboxEntry", "VariableList": "Variable", "KeySlotList": "KeySlot",
    "FormDefinitionList": "FormDefinition", "FormRecordList": "FormRecord", "ContactList": "Contact",
    "KnownLockboxList": "KnownLockbox", "AccessSlotLabelList": "AccessSlotLabel",
    "AgentEntryList": "AgentEntry", "ProfileHistoryList": "ProfileHistory",
}
codec = [
    "package com.onepub.revault;", "", "import com.google.flatbuffers.FlatBufferBuilder;", "import java.nio.ByteBuffer;", "import java.nio.ByteOrder;", "",
    "/** Internal conversion between native result bytes and public domain values. */", "final class DomainCodec {", "  private DomainCodec() {}",
]
for name in base:
    method = name[:1].lower() + name[1:]
    codec += [f"  static {name} {method}(byte[] bytes) {{", f"    return new {name}(com.onepub.revault.internal.{name}.getRootAs{name}(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)));", "  }"]
for wrapper, item in list_types.items():
    method = wrapper[:1].lower() + wrapper[1:]
    field = "entries" if wrapper == "LockboxEntryList" else "values"
    codec += [f"  static java.util.List<{item}> {method}(byte[] bytes) {{", f"    var view = com.onepub.revault.internal.{wrapper}.getRootAs{wrapper}(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN));", f"    var result = new java.util.ArrayList<{item}>(view.{field}Length());", f"    for (int index = 0; index < view.{field}Length(); index++) result.add(new {item}(view.{field}(index)));", "    return java.util.List.copyOf(result);", "  }"]
codec += [
    "  static java.util.List<String> stringList(byte[] bytes) { var view = com.onepub.revault.internal.StringList.getRootAsStringList(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)); var result = new java.util.ArrayList<String>(view.valuesLength()); for (int i=0;i<view.valuesLength();i++) result.add(view.values(i)); return java.util.List.copyOf(result); }",
    "  static LockboxEntry optionalLockboxEntry(byte[] bytes) { var value = com.onepub.revault.internal.OptionalLockboxEntry.getRootAsOptionalLockboxEntry(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)).value(); return value == null ? null : new LockboxEntry(value); }",
    "  static FormRecord optionalFormRecord(byte[] bytes) { var value = com.onepub.revault.internal.OptionalFormRecord.getRootAsOptionalFormRecord(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)).value(); return value == null ? null : new FormRecord(value); }",
    "  static FormValue optionalFormValue(byte[] bytes) { var value = com.onepub.revault.internal.OptionalFormValue.getRootAsOptionalFormValue(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)).value(); return value == null ? null : new FormValue(value); }",
    "  static String optionalString(byte[] bytes) { var value = com.onepub.revault.internal.OptionalString.getRootAsOptionalString(ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)); return value.present() ? value.value() : null; }",
    "  static byte[] encodePathMoves(java.util.List<PathMove> values) {",
    "    var builder = new FlatBufferBuilder();",
    "    var offsets = new int[values.size()];",
    "    for (int index = 0; index < values.size(); index++) {",
    "      var value = values.get(index);",
    "      offsets[index] = com.onepub.revault.internal.PathMove.createPathMove(builder, builder.createString(value.source()), builder.createString(value.destination()));",
    "    }",
    "    var vector = com.onepub.revault.internal.PathMoveList.createValuesVector(builder, offsets);",
    "    var root = com.onepub.revault.internal.PathMoveList.createPathMoveList(builder, vector);",
    "    builder.finish(root);",
    "    return builder.sizedByteArray();",
    "  }",
    "  static byte[] encodeFormFields(java.util.List<FormField> values) {",
    "    var builder = new FlatBufferBuilder();",
    "    var offsets = new int[values.size()];",
    "    for (int index = 0; index < values.size(); index++) {",
    "      var value = values.get(index);",
    "      offsets[index] = com.onepub.revault.internal.FormField.createFormField(builder, builder.createString(value.id()), builder.createString(value.label()), builder.createString(value.kind()), value.required());",
    "    }",
    "    var vector = com.onepub.revault.internal.FormFieldList.createValuesVector(builder, offsets);",
    "    var root = com.onepub.revault.internal.FormFieldList.createFormFieldList(builder, vector);",
    "    builder.finish(root);",
    "    return builder.sizedByteArray();",
    "  }",
    "}", "",
]
(OUTPUT / "DomainCodec.java").write_text("\n".join(codec))
