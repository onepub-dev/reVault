"""Generate PHP-owned domain values over the private FlatBuffers transport."""

from pathlib import Path
import re
from model_docs import description, field_description, wrapper_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "php/src/DomainModels.php"


def pascal(value: str) -> str:
    return "".join(part.title() for part in value.split("_"))


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
models = [name for name in tables if name not in wrappers]

lines = [
    "<?php", "declare(strict_types=1);", "", "namespace Revault;", "",
    "use Google\\FlatBuffers\\ByteBuffer;",
    "use Google\\FlatBuffers\\FlatBufferBuilder;", "",
]
for name in models:
    lines += [f"/** {description(name)} */", f"final class {name}", "{"]
    params = []
    for field, value in tables[name]:
        raw = value.strip("[]").split(".")[-1]
        kind = "array" if value.startswith("[") else ("string" if raw == "string" else "bool" if raw == "bool" else "int")
        params.append(f"public readonly {kind} ${field}")
    lines += ["    /** Creates a value from fields returned by the reVault API. */", f"    public function __construct({', '.join(params)}) {{}}"]
    for field, _ in tables[name]:
        lines += [f"    /** {field_description(name, field)} */", f"    public function get{pascal(field)}(): mixed {{ return $this->{field}; }}"]
    lines += ["}", ""]

list_types = {
    "LockboxEntryList": "LockboxEntry", "FormFieldList": "FormField",
    "FormDefinitionList": "FormDefinition", "FormRecordList": "FormRecord",
    "KeySlotList": "KeySlot", "PageInspectionList": "PageInspection",
    "KnownLockboxList": "KnownLockbox", "AccessSlotLabelList": "AccessSlotLabel",
    "StreamChunkList": "StreamChunk", "VariableList": "Variable",
    "ContactList": "Contact", "ProfileHistoryList": "ProfileHistory",
    "AgentEntryList": "AgentEntry",
}
for name, item in list_types.items():
    noun = "entries" if name == "LockboxEntryList" else "values"
    lines += [
        f"/** {wrapper_description(name)} */", f"final class {name}", "{",
        f"    /** @param list<{item}> ${noun} */",
        f"    public function __construct(public readonly array ${noun}) {{}}",
        f"    /** Returns the ordered {item} values. */",
        f"    public function get{pascal(noun)}(): array {{ return $this->{noun}; }}", "}", "",
    ]
lines += [
    "/** Ordered text values returned by a list operation. */", "final class StringList", "{",
    "    /** @param list<string> $values */", "    public function __construct(public readonly array $values) {}",
    "    /** Returns the ordered text values. */", "    public function getValues(): array { return $this->values; }", "}", "",
]
for name, item in [("OptionalLockboxEntry", "LockboxEntry"), ("OptionalFormRecord", "FormRecord"), ("OptionalFormValue", "FormValue")]:
    lines += [
        f"/** A lookup result that may contain one {item}. */", f"final class {name}", "{",
        f"    /** Creates a lookup result; null means the requested value was absent. */",
        f"    public function __construct(public readonly ?{item} $value) {{}}",
        f"    /** Returns the value, or null when it was absent. */", f"    public function getValue(): ?{item} {{ return $this->value; }}", "}", "",
    ]
lines += [
    "/** A text lookup result that distinguishes absence from an empty string. */", "final class OptionalString", "{",
    "    /** Creates an optional text result. */", "    public function __construct(public readonly bool $present, public readonly string $value) {}",
    "    /** Reports whether the requested text exists. */", "    public function getPresent(): bool { return $this->present; }",
    "    /** Returns the text, which may be empty even when present. */", "    public function getValue(): string { return $this->value; }", "}", "",
    "/** @internal Converts private native buffers to public values and encodes supported inputs. */", "final class DomainCodec", "{",
    "    private function __construct() {}",
]

def decode_expr(value: str, getter: str) -> str:
    raw = value.strip("[]").split(".")[-1]
    if value.startswith("["):
        if raw == "ubyte":
            return f"self::bytes($value, 'get{getter}')"
        if raw in {"string", "bool", "uint", "ulong", "ubyte"}:
            return f"self::scalars($value, 'get{getter}')"
        return f"self::tables($value, 'get{getter}', '{raw}')"
    if raw in tables:
        return f"self::from{raw}($value->get{getter}())"
    if raw == "string":
        return f"$value->get{getter}() ?? ''"
    return f"$value->get{getter}()"

for name in models:
    args = [decode_expr(value, pascal(field)) for field, value in tables[name]]
    lines += [
        f"    private static function from{name}(\\Revault\\Internal\\Transport\\{name} $value): {name}", "    {",
        f"        return new {name}({', '.join(args)});", "    }",
    ]
for name in models:
    lines += [
        f"    /** @internal Decodes a native {name} result. */",
        f"    public static function decode{name}(string $bytes): {name}", "    {",
        f"        return self::from{name}(\\Revault\\Internal\\Transport\\{name}::getRootAs{name}(ByteBuffer::wrap($bytes)));", "    }",
    ]
for name, item in list_types.items():
    field = "Entries" if name == "LockboxEntryList" else "Values"
    noun = "entries" if name == "LockboxEntryList" else "values"
    lines += [
        f"    /** @internal Decodes a native {name} result. */", f"    public static function decode{name}(string $bytes): {name}", "    {",
        f"        $root = \\Revault\\Internal\\Transport\\{name}::getRootAs{name}(ByteBuffer::wrap($bytes));",
        f"        return new {name}(self::tables($root, 'get{field}', '{item}'));", "    }",
    ]
lines += [
    "    /** @internal Decodes a native StringList result. */", "    public static function decodeStringList(string $bytes): StringList", "    {",
    "        $root = \\Revault\\Internal\\Transport\\StringList::getRootAsStringList(ByteBuffer::wrap($bytes));",
    "        return new StringList(self::scalars($root, 'getValues'));", "    }",
]
for name, item in [("OptionalLockboxEntry", "LockboxEntry"), ("OptionalFormRecord", "FormRecord"), ("OptionalFormValue", "FormValue")]:
    lines += [
        f"    /** @internal Decodes a native {name} result. */", f"    public static function decode{name}(string $bytes): {name}", "    {",
        f"        $root = \\Revault\\Internal\\Transport\\{name}::getRootAs{name}(ByteBuffer::wrap($bytes)); $value = $root->getValue();",
        f"        return new {name}($value === null ? null : self::from{item}($value));", "    }",
    ]
lines += [
    "    /** @internal Decodes a native OptionalString result. */", "    public static function decodeOptionalString(string $bytes): OptionalString", "    {",
    "        $root = \\Revault\\Internal\\Transport\\OptionalString::getRootAsOptionalString(ByteBuffer::wrap($bytes));",
    "        return new OptionalString($root->getPresent(), $root->getValue() ?? '');", "    }",
    "    /** @return list<mixed> */", "    private static function tables(object $value, string $getter, string $type): array", "    {",
    "        $length = $getter . 'Length'; $result = [];",
    "        for ($index = 0; $index < $value->$length(); $index++) { $convert = 'from' . $type; $result[] = self::$convert($value->$getter($index)); }",
    "        return $result;", "    }",
    "    /** @return list<mixed> */", "    private static function scalars(object $value, string $getter): array", "    {",
    "        $length = $getter . 'Length'; $result = []; for ($index = 0; $index < $value->$length(); $index++) $result[] = $value->$getter($index); return $result;", "    }",
    "    /** Returns a byte vector as a PHP binary string. */", "    private static function bytes(object $value, string $getter): string", "    {",
    "        return implode('', array_map(chr(...), self::scalars($value, $getter)));", "    }",
    "    /** Encodes variable or form-record path moves for the native API. */", "    public static function encodePathMoves(array $moves): string", "    {",
    "        $builder = new FlatBufferBuilder(256); $offsets = [];",
    "        foreach ($moves as $move) { $source = $builder->createString($move->source); $destination = $builder->createString($move->destination); $offsets[] = \\Revault\\Internal\\Transport\\PathMove::createPathMove($builder, $source, $destination); }",
    "        $values = \\Revault\\Internal\\Transport\\PathMoveList::createValuesVector($builder, $offsets); $root = \\Revault\\Internal\\Transport\\PathMoveList::createPathMoveList($builder, $values); $builder->finish($root); return $builder->sizedByteArray();", "    }",
    "    /** Encodes form fields for the native API. */", "    public static function encodeFormFields(array $fields): string", "    {",
    "        $builder = new FlatBufferBuilder(256); $offsets = [];",
    "        foreach ($fields as $field) { $id = $builder->createString($field->id); $label = $builder->createString($field->label); $kind = $builder->createString($field->kind); $offsets[] = \\Revault\\Internal\\Transport\\FormField::createFormField($builder, $id, $label, $kind, $field->required); }",
    "        $values = \\Revault\\Internal\\Transport\\FormFieldList::createValuesVector($builder, $offsets); $root = \\Revault\\Internal\\Transport\\FormFieldList::createFormFieldList($builder, $values); $builder->finish($root); return $builder->sizedByteArray();", "    }", "}", "",
]
OUTPUT.write_text("\n".join(lines))
