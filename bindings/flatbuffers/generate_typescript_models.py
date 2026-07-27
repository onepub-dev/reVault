"""Generate public TypeScript domain interfaces from the private schema."""

from pathlib import Path
import re
from model_docs import DESCRIPTIONS, field_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "javascript/domain.d.ts"

LEGACY_CONTEXT = {
    "LockboxEntry": "Metadata for one filesystem object stored in a lockbox.",
    "PathMove": "A source and destination used by an atomic rename.",
    "FormField": "One field in a reusable form definition.",
    "FormDefinition": "A versioned template for form records.",
    "FormValue": "A non-secret value read from a form record.",
    "FormRecord": "A named record stored using a form definition.",
    "RecoveryReport": "Results from scanning a damaged lockbox.",
    "KeySlot": "A mechanism that grants access to a lockbox content key.",
    "CacheStats": "Current page-cache usage and hit statistics.",
    "ImportStats": "Timing for stages of the most recent import.",
    "PageObject": "A logical object found on a lockbox page.",
    "PageInspection": "Storage details for an encrypted lockbox page.",
    "FileInspection": "Lockbox header and access metadata read without opening it.",
    "ProfileGeneration": "One active or retired profile key generation.",
    "ProfileHistory": "The key-generation history for a local profile.",
    "KnownLockbox": "A lockbox remembered by the local vault.",
    "AccessSlotLabel": "A local label for a lockbox access slot.",
    "StreamChunk": "A logical or physical extent of lockbox content.",
    "RuntimeOptions": "Worker and workload policies for an open lockbox.",
    "Variable": "A lockbox variable name and sensitivity.",
    "OwnerInspection": "Owner-signature state for a lockbox.",
    "Contact": "A named recipient and public encryption key.",
    "AgentEntry": "A secret cached by the local session agent.",
    "SleepSupport": "Host protections available while the system sleeps.",
    "PlatformStatus": "Operating-system credential-store status.",
    "VaultBackupManifest": "Identity and digest of a local-vault backup.",
    "ErrorDetails": "Classification and recovery guidance for a failed call.",
}


def camel(name: str) -> str:
    parts = name.split("_")
    return parts[0] + "".join(part.title() for part in parts[1:])


def ts_type(value: str) -> str:
    if value.startswith("["):
        inner = value[1:-1].split(".")[-1]
        return "Uint8Array" if inner == "ubyte" else f"readonly {ts_type(inner)}[]"
    value = value.split(".")[-1]
    return {
        "string": "string",
        "bool": "boolean",
        "ubyte": "number",
        "uint": "number",
        "ulong": "bigint",
        "LockboxEntryKind": "LockboxEntryKind",
    }.get(value, value)


tables = {}
current = None
for line in SCHEMA.read_text().splitlines():
    match = re.match(r"table (\w+) \{", line.strip())
    if match:
        current = match.group(1)
        tables[current] = []
    elif current and line.strip() == "}":
        current = None
    elif current and (match := re.match(r"\s*(\w+):([^ ]+) \(id:", line)):
        tables[current].append(match.groups())

lines = [
    "/** Public reVault domain values. Serialization is deliberately private. */",
    "export type LockboxEntryKind = 0 | 1 | 2 | 3;",
    "",
]
for name, fields in tables.items():
    if name not in DESCRIPTIONS:
        continue
    lines += [f"/** {DESCRIPTIONS[name]} */", f"export interface {name} {{"]
    for field, value in fields:
        lines += [f"  /** {field_description(name, field)} */", f"  readonly {camel(field)}: {ts_type(value)};"]
    lines += ["}", ""]

lines += [
    "/** A source and destination pair accepted by move operations. */",
    "export interface PathMoveInput { readonly source: string; readonly destination: string; }",
    "/** A field accepted when defining a form. */",
    "export interface FormFieldInput { readonly id: string; readonly label: string; readonly kind: string; readonly required: boolean; }",
]
OUTPUT.write_text("\n".join(lines) + "\n")
