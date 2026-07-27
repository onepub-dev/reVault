"""Private FlatBuffers adapters for public Python domain values."""

from __future__ import annotations

from types import SimpleNamespace
from typing import Any

import flatbuffers

from ._flatbuffers import revault_bindings_generated as transport


def _pascal(name: str) -> str:
    return "".join(part.title() for part in name.split("_"))


class DomainView:
    """Read-only, lazy view of a structured result owned by its byte buffer."""

    __slots__ = ("_table",)

    def __init__(self, table: Any):
        self._table = table

    def __getattr__(self, name: str) -> Any:
        field = _pascal(name)
        numpy = getattr(self._table, field + "AsNumpy", None)
        if numpy is not None:
            value = numpy()
            return bytes(value) if value is not None else b""
        length = getattr(self._table, field + "Length", None)
        getter = getattr(self._table, field, None)
        if length is not None and getter is not None:
            return tuple(_adapt(getter(index)) for index in range(length()))
        if getter is None:
            raise AttributeError(name)
        return _adapt(getter())

    def __repr__(self) -> str:
        return f"{self._table.__class__.__name__}(...)"


def _adapt(value: Any) -> Any:
    if isinstance(value, bytes):
        return value.decode("utf-8")
    if hasattr(value, "_tab"):
        return DomainView(value)
    return value


def decode(name: str, value: bytes) -> Any:
    """Create a lazy domain view for a trusted native result buffer."""
    result = DomainView(getattr(transport, name).GetRootAs(value))
    if name == "LockboxEntryList":
        return result.entries
    if name.endswith("List"):
        return result.values
    if name == "OptionalString":
        return result.value if result.present else None
    if name in ("OptionalLockboxEntry", "OptionalFormRecord", "OptionalFormValue"):
        return result.value
    return result


def encode_path_moves(values: list[Any]) -> bytes:
    """Encode application path moves for the private native ABI."""
    builder = flatbuffers.Builder(256)
    offsets = []
    for value in values:
        source = builder.CreateString(value.source)
        destination = builder.CreateString(value.destination)
        offsets.append(transport.PathMoveStart(builder) or None)
        transport.PathMoveAddSource(builder, source)
        transport.PathMoveAddDestination(builder, destination)
        offsets[-1] = transport.PathMoveEnd(builder)
    transport.PathMoveListStartValuesVector(builder, len(offsets))
    for offset in reversed(offsets):
        builder.PrependUOffsetTRelative(offset)
    vector = builder.EndVector()
    transport.PathMoveListStart(builder)
    transport.PathMoveListAddValues(builder, vector)
    root = transport.PathMoveListEnd(builder)
    builder.Finish(root)
    return bytes(builder.Output())


def encode_form_fields(values: list[Any]) -> bytes:
    """Encode application form fields for the private native ABI."""
    builder = flatbuffers.Builder(256)
    offsets = []
    for value in values:
        field_id = builder.CreateString(value.id)
        label = builder.CreateString(value.label)
        kind = builder.CreateString(value.kind)
        transport.FormFieldStart(builder)
        transport.FormFieldAddId(builder, field_id)
        transport.FormFieldAddLabel(builder, label)
        transport.FormFieldAddKind(builder, kind)
        transport.FormFieldAddRequired(builder, value.required)
        offsets.append(transport.FormFieldEnd(builder))
    transport.FormFieldListStartValuesVector(builder, len(offsets))
    for offset in reversed(offsets):
        builder.PrependUOffsetTRelative(offset)
    vector = builder.EndVector()
    transport.FormFieldListStart(builder)
    transport.FormFieldListAddValues(builder, vector)
    root = transport.FormFieldListEnd(builder)
    builder.Finish(root)
    return bytes(builder.Output())


class PathMove(SimpleNamespace):
    """A source and destination pair applied atomically inside a lockbox."""


class FormField(SimpleNamespace):
    """One field in a reusable, versioned form definition."""
