"""Generate owned Swift domain values over private FlatBuffers tables."""
from pathlib import Path
import re
from model_docs import description, field_description

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = Path(__file__).with_name("revault_bindings.fbs")
OUTPUT = ROOT / "swift/Sources/RevaultAPI/DomainModels.swift"
SCALARS = {"string":"String", "bool":"Bool", "uint":"UInt32", "ulong":"UInt64", "ubyte":"UInt8"}

def camel(name):
    parts=name.split("_"); return parts[0]+"".join(p.title() for p in parts[1:])
def stype(value):
    if value.startswith("["):
        inner=value[1:-1].split(".")[-1]; return f"[{SCALARS.get(inner, inner)}]"
    raw=value.split(".")[-1]; return "LockboxEntryKind" if raw=="LockboxEntryKind" else SCALARS.get(raw,raw)

tables={}; current=None
for line in SCHEMA.read_text().splitlines():
    if m:=re.match(r"table (\w+) \{",line.strip()): current=m.group(1); tables[current]=[]
    elif current and line.strip()=="}": current=None
    elif current and (m:=re.match(r"\s*(\w+):([^ ]+) \(id:",line)): tables[current].append(m.groups())
wrappers={n for n in tables if n.endswith("List") or n.startswith("Optional")}|{"StringValue"}
base=[n for n in tables if n not in wrappers]
lists={"StreamChunkList":"StreamChunk","PageInspectionList":"PageInspection","LockboxEntryList":"LockboxEntry","VariableList":"Variable","KeySlotList":"KeySlot","FormDefinitionList":"FormDefinition","FormRecordList":"FormRecord","ContactList":"Contact","KnownLockboxList":"KnownLockbox","AccessSlotLabelList":"AccessSlotLabel","AgentEntryList":"AgentEntry","ProfileHistoryList":"ProfileHistory"}

out=["import FlatBuffers","import Foundation","","/// Identifies the filesystem object stored at a lockbox path.","public enum LockboxEntryKind: Int { case unspecified, file, symlink, directory }",""]
for name in base:
    out += [f"/// {description(name)}",f"public struct {name} {{"]
    for field,value in tables[name]: out.append(f"    /// {field_description(name, field)}\n    public let {camel(field)}: {stype(value)}")
    params=", ".join(f"{camel(f)}: {stype(v)}" for f,v in tables[name])
    out += [f"    /// Creates an application-owned {name} value.",f"    public init({params}) {{"]
    for field,_ in tables[name]: out.append(f"        self.{camel(field)} = {camel(field)}")
    out += ["    }","}",""]
out += ["enum DomainCodec {"]
for name in base:
    args=[]
    for field,value in tables[name]:
        prop=camel(field); raw=value.split(".")[-1]
        if value.startswith("["):
            inner=value[1:-1].split(".")[-1]
            expr=f"value.{prop} ?? []" if inner in SCALARS else f"(value.{prop} ?? []).map(convert)"
        elif raw=="LockboxEntryKind": expr=f"LockboxEntryKind(rawValue: Int(value.{prop}.rawValue)) ?? .unspecified"
        elif raw=="string": expr=f"value.{prop} ?? \"\""
        elif raw in SCALARS: expr=f"value.{prop}"
        else: expr=f"convert(value.{prop}!)"
        args.append(f"{prop}: {expr}")
    out += [f"    private static func convert(_ value: revault_internal__{name}T) -> {name} {{",f"        {name}({', '.join(args)})","    }",f"    static func {name[0].lower()+name[1:]}(_ data: Data) -> {name} {{",f"        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__{name} = getRoot(byteBuffer: &buffer); return convert(root.unpack())","    }"]
for wrapper,item in lists.items():
    field="entries" if wrapper=="LockboxEntryList" else "values"; method=wrapper[0].lower()+wrapper[1:]
    out += [f"    static func {method}(_ data: Data) -> [{item}] {{",f"        var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__{wrapper} = getRoot(byteBuffer: &buffer); return (root.unpack().{field} ?? []).map(convert)","    }"]
out += [
"    static func stringList(_ data: Data) -> [String] { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__StringList = getRoot(byteBuffer: &buffer); return root.unpack().values ?? [] }",
"    static func optionalString(_ data: Data) -> String? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalString = getRoot(byteBuffer: &buffer); let value = root.unpack(); return value.present ? value.value : nil }",
"    static func optionalLockboxEntry(_ data: Data) -> LockboxEntry? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalLockboxEntry = getRoot(byteBuffer: &buffer); return root.unpack().value.map(convert) }",
"    static func optionalFormRecord(_ data: Data) -> FormRecord? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalFormRecord = getRoot(byteBuffer: &buffer); return root.unpack().value.map(convert) }",
"    static func optionalFormValue(_ data: Data) -> FormValue? { var buffer = ByteBuffer(bytes: [UInt8](data)); var root: revault_internal__OptionalFormValue = getRoot(byteBuffer: &buffer); return root.unpack().value.map(convert) }",
"    static func encodePathMoves(_ values: [PathMove]) -> Data { var builder = FlatBufferBuilder(initialSize: 256); var offsets: [Offset] = []; for value in values { let source = builder.create(string: value.source); let destination = builder.create(string: value.destination); offsets.append(revault_internal__PathMove.createPathMove(&builder, sourceOffset: source, destinationOffset: destination)) }; let vector = builder.createVector(ofOffsets: offsets); let root = revault_internal__PathMoveList.createPathMoveList(&builder, valuesVectorOffset: vector); builder.finish(offset: root); return Data(builder.sizedByteArray) }",
"    static func encodeFormFields(_ values: [FormField]) -> Data { var builder = FlatBufferBuilder(initialSize: 256); var offsets: [Offset] = []; for value in values { let id = builder.create(string: value.id); let label = builder.create(string: value.label); let kind = builder.create(string: value.kind); offsets.append(revault_internal__FormField.createFormField(&builder, idOffset: id, labelOffset: label, kindOffset: kind, required: value.required)) }; let vector = builder.createVector(ofOffsets: offsets); let root = revault_internal__FormFieldList.createFormFieldList(&builder, valuesVectorOffset: vector); builder.finish(offset: root); return Data(builder.sizedByteArray) }",
"}",""]
OUTPUT.write_text("\n".join(out))
