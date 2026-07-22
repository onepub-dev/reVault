"""Generate owned C# domain records over the private FlatBuffers transport."""

from pathlib import Path
import re
from model_docs import description, field_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "csharp/DomainModels.cs"

SCALARS = {"string": "string", "bool": "bool", "uint": "uint", "ulong": "ulong", "ubyte": "byte"}

def pascal(name: str) -> str:
    return "".join(part.title() for part in name.split("_"))

def ctype(value: str) -> str:
    if value.startswith("["):
        inner = value[1:-1].split(".")[-1]
        return "byte[]" if inner == "ubyte" else f"IReadOnlyList<{SCALARS.get(inner, inner)}>"
    raw = value.split(".")[-1]
    return "LockboxEntryKind" if raw == "LockboxEntryKind" else SCALARS.get(raw, raw)

tables: dict[str, list[tuple[str, str]]] = {}
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
wrappers.add("StringValue")
base = [name for name in tables if name not in wrappers]

lines = [
    "using Google.FlatBuffers;", "", "namespace Revault;", "",
    "/// <summary>Identifies the filesystem object stored at a lockbox path.</summary>",
    "public enum LockboxEntryKind",
    "{",
    "    /// <summary>No recognized kind was reported.</summary>", "    Unspecified,",
    "    /// <summary>A regular file.</summary>", "    File,",
    "    /// <summary>A symbolic link.</summary>", "    Symlink,",
    "    /// <summary>A directory.</summary>", "    Directory",
    "}", "",
]
for name in base:
    fields = tables[name]
    params = ", ".join(f"{ctype(value)} {pascal(field)}" for field, value in fields)
    lines += [f"/// <summary>{description(name)}</summary>"]
    for field, _ in fields:
        lines.append(f"/// <param name=\"{pascal(field)}\">{field_description(name, field)}</param>")
    lines += [f"public sealed record {name}({params});", ""]

lines += ["/// <summary>Converts private native transport buffers into public domain values.</summary>", "internal static class DomainCodec", "{"]
for name in base:
    args = []
    for field, value in tables[name]:
        prop = pascal(field)
        raw = value.split(".")[-1]
        if value.startswith("["):
            inner = value[1:-1].split(".")[-1]
            if inner == "ubyte":
                expr = f"value.{prop}?.ToArray() ?? Array.Empty<byte>()"
            elif inner in SCALARS:
                expr = f"value.{prop}?.ToArray() ?? Array.Empty<{SCALARS[inner]}>()"
            else:
                expr = f"value.{prop} is null ? Array.Empty<{inner}>() : value.{prop}.Select(FromInternal).ToArray()"
        elif raw == "LockboxEntryKind":
            expr = f"(LockboxEntryKind)(int)value.{prop}"
        elif raw == "string":
            expr = f"value.{prop} ?? string.Empty"
        elif raw in SCALARS:
            expr = f"value.{prop}"
        else:
            expr = f"FromInternal(value.{prop})"
        args.append(expr)
    lines += [f"    private static {name} FromInternal(Revault.Internal.Transport.{name}T value) =>", f"        new({', '.join(args)});", f"    internal static {name} {name}(byte[] bytes) => FromInternal(Revault.Internal.Transport.{name}.GetRootAs{name}(new ByteBuffer(bytes)).UnPack());"]

list_types = {
    "StreamChunkList": "StreamChunk", "PageInspectionList": "PageInspection", "LockboxEntryList": "LockboxEntry",
    "VariableList": "Variable", "KeySlotList": "KeySlot", "FormDefinitionList": "FormDefinition",
    "FormRecordList": "FormRecord", "ContactList": "Contact", "KnownLockboxList": "KnownLockbox",
    "AccessSlotLabelList": "AccessSlotLabel", "AgentEntryList": "AgentEntry", "ProfileHistoryList": "ProfileHistory",
}
for wrapper, item in list_types.items():
    field = "Entries" if wrapper == "LockboxEntryList" else "Values"
    lines += [f"    internal static IReadOnlyList<{item}> {wrapper}(byte[] bytes)", "    {", f"        var values = Revault.Internal.Transport.{wrapper}.GetRootAs{wrapper}(new ByteBuffer(bytes)).UnPack().{field};", f"        return values is null ? Array.Empty<{item}>() : values.Select(FromInternal).ToArray();", "    }"]
lines += [
    "    internal static IReadOnlyList<string> StringList(byte[] bytes) => Revault.Internal.Transport.StringList.GetRootAsStringList(new ByteBuffer(bytes)).UnPack().Values?.ToArray() ?? Array.Empty<string>();",
    "    internal static LockboxEntry? OptionalLockboxEntry(byte[] bytes) { var value = Revault.Internal.Transport.OptionalLockboxEntry.GetRootAsOptionalLockboxEntry(new ByteBuffer(bytes)).UnPack().Value; return value is null ? null : FromInternal(value); }",
    "    internal static FormRecord? OptionalFormRecord(byte[] bytes) { var value = Revault.Internal.Transport.OptionalFormRecord.GetRootAsOptionalFormRecord(new ByteBuffer(bytes)).UnPack().Value; return value is null ? null : FromInternal(value); }",
    "    internal static FormValue? OptionalFormValue(byte[] bytes) { var value = Revault.Internal.Transport.OptionalFormValue.GetRootAsOptionalFormValue(new ByteBuffer(bytes)).UnPack().Value; return value is null ? null : FromInternal(value); }",
    "    internal static string? OptionalString(byte[] bytes) { var value = Revault.Internal.Transport.OptionalString.GetRootAsOptionalString(new ByteBuffer(bytes)).UnPack(); return value.Present ? value.Value : null; }",
    "    internal static byte[] EncodePathMoves(IReadOnlyList<PathMove> values) { var builder = new FlatBufferBuilder(256); var transport = new Revault.Internal.Transport.PathMoveListT { Values = values.Select(value => new Revault.Internal.Transport.PathMoveT { Source = value.Source, Destination = value.Destination }).ToList() }; var root = Revault.Internal.Transport.PathMoveList.Pack(builder, transport); builder.Finish(root.Value); return builder.SizedByteArray(); }",
    "    internal static byte[] EncodeFormFields(IReadOnlyList<FormField> values) { var builder = new FlatBufferBuilder(256); var transport = new Revault.Internal.Transport.FormFieldListT { Values = values.Select(value => new Revault.Internal.Transport.FormFieldT { Id = value.Id, Label = value.Label, Kind = value.Kind, Required = value.Required }).ToList() }; var root = Revault.Internal.Transport.FormFieldList.Pack(builder, transport); builder.Finish(root.Value); return builder.SizedByteArray(); }",
    "}", "",
]
OUTPUT.write_text("\n".join(lines))
