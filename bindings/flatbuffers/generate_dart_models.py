"""Generate documented Dart domain views over the private FlatBuffers tables."""

from pathlib import Path
import re
from model_docs import DESCRIPTIONS, field_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "dart/lib/src/domain_models.dart"

LEGACY_DESCRIPTIONS = {
    "LockboxEntry": "Metadata for one file, directory, or symbolic link stored in a lockbox.",
    "PathMove": "A source and destination pair used in an atomic lockbox rename operation.",
    "FormField": "One field in a reusable form definition, including its validation requirements.",
    "FormDefinition": "A versioned template that defines the fields accepted by form records.",
    "FormValue": "A non-secret field value read from a form record.",
    "FormRecord": "A named record stored in a lockbox using a particular form definition revision.",
    "RecoveryReport": "The files and metadata recovered while scanning a damaged lockbox.",
    "KeySlot": "A lockbox access slot and the protection mechanism used by its wrapped key.",
    "CacheStats": "Current capacity and hit statistics for an open lockbox's page cache.",
    "ImportStats": "Time spent in each stage of the most recent lockbox import.",
    "PageObject": "A logical object found while inspecting an encrypted lockbox page.",
    "PageInspection": "Storage layout and object metadata for one encrypted lockbox page.",
    "FileInspection": "Header, owner, and key-slot metadata read without opening a lockbox.",
    "ProfileGeneration": "One active or retired encryption-key generation for a local profile.",
    "ProfileHistory": "The key-generation history and active generation for a local profile.",
    "KnownLockbox": "A lockbox path remembered by the local vault for later discovery.",
    "AccessSlotLabel": "A local, human-readable label assigned to a lockbox access slot.",
    "StreamChunk": "A logical or physical extent returned while streaming lockbox content.",
    "RuntimeOptions": "The workload and worker policies active on an open lockbox.",
    "Variable": "The name and sensitivity classification of a lockbox variable.",
    "OwnerInspection": "The owner-signature state and fingerprint of an open lockbox.",
    "Contact": "A named recipient and their public contact-encryption key.",
    "AgentEntry": "A secret currently cached by the local session agent.",
    "SleepSupport": "The host capabilities used to protect cached secrets across sleep.",
    "PlatformStatus": "Availability and configuration of the operating-system credential store.",
    "VaultBackupManifest": "Identity, size, and digest of a local-vault backup archive.",
    "ErrorDetails": "Actionable classification and recovery guidance for a native API failure.",
}

SCALAR_TYPES = {
    "string": "String",
    "bool": "bool",
    "ubyte": "int",
    "uint": "int",
    "ulong": "int",
}
DEFAULTS = {"String": "''", "bool": "false", "int": "0"}


def lower_camel(value: str) -> str:
    words = value.split("_")
    return words[0] + "".join(word.title() for word in words[1:])


def parse_tables() -> dict[str, list[tuple[str, str]]]:
    tables: dict[str, list[tuple[str, str]]] = {}
    current = None
    for line in SCHEMA.read_text().splitlines():
        match = re.match(r"table (\w+) \{", line.strip())
        if match:
            current = match.group(1)
            tables[current] = []
            continue
        if current and line.strip() == "}":
            current = None
            continue
        match = re.match(r"\s*(\w+):([^ ]+) \(id:", line)
        if current and match:
            tables[current].append((match.group(1), match.group(2)))
    return tables


def dart_type(schema_type: str) -> str:
    if schema_type.startswith("["):
        item = schema_type[1:-1].split(".")[-1]
        return "Uint8List" if item == "ubyte" else f"List<{SCALAR_TYPES.get(item, item)}>"
    name = schema_type.split(".")[-1]
    if name == "LockboxEntryKind":
        return "LockboxEntryKind"
    return SCALAR_TYPES.get(name, name)


def field_expression(field: str, schema_type: str) -> str:
    name = lower_camel(field)
    transport_name = f"${name}" if name == "required" else name
    dtype = dart_type(schema_type)
    if schema_type.startswith("["):
        item = schema_type[1:-1].split(".")[-1]
        if item == "ubyte":
            return f"Uint8List.fromList(_view.{transport_name} ?? const <int>[])"
        if item in SCALAR_TYPES:
            return f"List<{SCALAR_TYPES[item]}>.unmodifiable(_view.{transport_name} ?? const [])"
        return f"List<{item}>.unmodifiable((_view.{transport_name} ?? const []).map({item}._from))"
    raw = schema_type.split(".")[-1]
    if raw == "LockboxEntryKind":
        return f"LockboxEntryKind.values[_view.{transport_name}.value]"
    if raw in SCALAR_TYPES:
        return f"_view.{transport_name} ?? {DEFAULTS[dtype]}" if raw == "string" else f"_view.{transport_name}"
    return f"_view.{transport_name} == null ? null : {raw}._from(_view.{transport_name}!)"


def generate() -> str:
    tables = parse_tables()
    public = [name for name in tables if name in DESCRIPTIONS]
    lines = [
        "// Generated by bindings/flatbuffers/generate_dart_models.py.",
        "",
        "import 'dart:typed_data';",
        "import 'generated/flatbuffers/revault_bindings_revault.internal_generated.dart' as fb;",
        "",
        "/// The kind of filesystem object represented by a [LockboxEntry].",
        "enum LockboxEntryKind {",
        "  /// No recognized filesystem kind was reported.",
        "  unspecified,",
        "  /// A regular file containing bytes.",
        "  file,",
        "  /// A symbolic link containing a target path.",
        "  symlink,",
        "  /// A directory that may contain other entries.",
        "  directory,",
        "}",
        "",
    ]
    for table in public:
        fields = tables[table]
        lines += [f"/// {DESCRIPTIONS[table]}", f"final class {table} {{"]
        constructor_fields = []
        initializers = ["_view = null"]
        view_initializers = []
        lines.append(f"  final fb.{table}? _view;")
        for field, schema_type in fields:
            name = lower_camel(field)
            dtype = dart_type(schema_type)
            nullable = "?" if schema_type.split(".")[-1] not in SCALAR_TYPES and not schema_type.startswith("[") and schema_type.split(".")[-1] != "LockboxEntryKind" else ""
            default = ""
            if not nullable:
                if dtype.startswith("List<"):
                    default = " = const []"
                elif dtype == "Uint8List":
                    default = None
                else:
                    default = f" = {DEFAULTS.get(dtype, 'LockboxEntryKind.unspecified')}"
            parameter = f"required {dtype} {name}" if default is None else f"{dtype}{nullable} {name}{default}"
            constructor_fields.append(parameter)
            initializers.append(f"_{name} = {name}")
            view_initializers.append(f"_{name} = null")
            lines.append(f"  final {dtype}? _{name};")
            fallback = f"_{name}" if nullable else f"_{name}!"
            expression = field_expression(field, schema_type)
            lines += [
                f"  /// {field_description(table, field)}",
                f"  late final {dtype}{nullable} {name} = _view == null ? {fallback} : {expression};",
            ]
        lines.append("")
        lines.append(f"  /// Creates a {table} value for an API input or application-owned copy.")
        lines.append(f"  {table}({{{', '.join(constructor_fields)}}}) : {', '.join(initializers)};")
        lines += [f"  {table}._from(this._view) : {', '.join(view_initializers)};", "}", ""]

    lines += ["/// Private decoders used by the native operation layer.", "abstract final class DomainDecoders {"]
    for table in public:
        method = table[0].lower() + table[1:]
        lines.append(f"  /// Decodes a native {table} result without copying its fields.")
        lines.append(f"  static {table} {method}(Uint8List bytes) => {table}._from(fb.{table}(bytes));")
    list_types = {
        "StreamChunkList": "StreamChunk",
        "PageInspectionList": "PageInspection",
        "VariableList": "Variable",
        "KeySlotList": "KeySlot",
        "FormDefinitionList": "FormDefinition",
        "FormRecordList": "FormRecord",
        "ContactList": "Contact",
        "KnownLockboxList": "KnownLockbox",
        "AccessSlotLabelList": "AccessSlotLabel",
        "AgentEntryList": "AgentEntry",
        "ProfileHistoryList": "ProfileHistory",
    }
    for wrapper, item in list_types.items():
        method = wrapper[0].lower() + wrapper[1:]
        lines.append(f"  /// Decodes a native list of {item} values.")
        lines.append(
            f"  static List<{item}> {method}(Uint8List bytes) => "
            f"List<{item}>.unmodifiable((fb.{wrapper}(bytes).values ?? const []).map({item}._from));"
        )
    lines += [
        "  /// Decodes lockbox entry metadata.",
        "  static List<LockboxEntry> lockboxEntryList(Uint8List bytes) => List<LockboxEntry>.unmodifiable((fb.LockboxEntryList(bytes).entries ?? const []).map(LockboxEntry._from));",
        "  /// Decodes a list of UTF-8 strings.",
        "  static List<String> stringList(Uint8List bytes) => List<String>.unmodifiable(fb.StringList(bytes).values ?? const []);",
        "  /// Decodes optional lockbox entry metadata.",
        "  static LockboxEntry? optionalLockboxEntry(Uint8List bytes) { final value = fb.OptionalLockboxEntry(bytes).value; return value == null ? null : LockboxEntry._from(value); }",
        "  /// Decodes an optional form record.",
        "  static FormRecord? optionalFormRecord(Uint8List bytes) { final value = fb.OptionalFormRecord(bytes).value; return value == null ? null : FormRecord._from(value); }",
        "  /// Decodes an optional form value.",
        "  static FormValue? optionalFormValue(Uint8List bytes) { final value = fb.OptionalFormValue(bytes).value; return value == null ? null : FormValue._from(value); }",
        "  /// Decodes an optional UTF-8 string.",
        "  static String? optionalString(Uint8List bytes) { final value = fb.OptionalString(bytes); return value.present ? value.value : null; }",
        "  /// Encodes atomic path moves for the native ABI.",
        "  static Uint8List pathMoves(List<PathMove> values) => fb.PathMoveListObjectBuilder(values: values.map((value) => fb.PathMoveObjectBuilder(source: value.source, destination: value.destination)).toList()).toBytes();",
        "  /// Encodes form fields for the native ABI.",
        "  static Uint8List formFields(List<FormField> values) => fb.FormFieldListObjectBuilder(values: values.map((value) => fb.FormFieldObjectBuilder(id: value.id, label: value.label, kind: value.kind, $required: value.required)).toList()).toBytes();",
    ]
    lines += ["}", ""]
    return "\n".join(lines)


OUTPUT.write_text(generate())
