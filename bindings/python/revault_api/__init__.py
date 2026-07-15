"""Typed ctypes wrapper for the reVault native binding library."""
from __future__ import annotations

import ctypes
import os
import platform
import sys
from pathlib import Path
from dataclasses import dataclass
from enum import Enum
from typing import Optional

from . import revault_bindings_pb2 as messages


class _Buffer(ctypes.Structure):
    _fields_ = [("ptr", ctypes.POINTER(ctypes.c_uint8)), ("len", ctypes.c_size_t)]


class LockboxEntryKind(str, Enum):
    FILE = "file"
    SYMLINK = "symlink"
    DIRECTORY = "directory"


class ProfileGenerationStatus(str, Enum):
    ACTIVE = "active"
    RETIRED = "retired"
    COMPROMISED = "compromised"


@dataclass(frozen=True)
class ProfileGeneration:
    index: int
    status: ProfileGenerationStatus
    contact_fingerprint: bytes
    created_at_unix_ms: int
    retired_at_unix_ms: int | None


@dataclass(frozen=True)
class ProfileHistory:
    name: str
    active_generation: int
    generations: tuple[ProfileGeneration, ...]


@dataclass(frozen=True)
class LockboxEntry:
    path: str
    kind: LockboxEntryKind
    length: int
    permissions: int

    @classmethod
    def from_protobuf(cls, value: bytes) -> "LockboxEntry":
        message = messages.LockboxEntry.FromString(value)
        return cls(
            path=message.path,
            kind=LockboxEntryKind({1: "file", 2: "symlink", 3: "directory"}[message.kind]),
            length=message.length,
            permissions=message.permissions,
        )


class Lockbox:
    def __init__(self, library: ctypes.CDLL, handle: int):
        self._lib, self._handle = library, ctypes.c_void_p(handle)

    def __del__(self) -> None:
        if getattr(self, "_handle", None):
            self._lib.lockbox_free(self._handle)
            self._handle = None

    def add_file(self, path: str, data: bytes, replace: bool = False) -> None:
        encoded = path.encode()
        if not self._lib.lockbox_add_file(self._handle, encoded, len(encoded), data, len(data), replace):
            raise RuntimeError(_error(self._lib))

    def commit(self) -> None:
        if not self._lib.lockbox_commit(self._handle):
            raise RuntimeError(_error(self._lib))

    def _bytes(self, result: _Buffer) -> bytes:
        if not result.ptr:
            raise RuntimeError(_error(self._lib))
        try:
            return ctypes.string_at(result.ptr, result.len)
        finally:
            self._lib.buffer_free(result)

    def add_file_with_permissions(self, path: str, data: bytes, permissions: int, replace: bool = False) -> None:
        encoded = path.encode()
        if not self._lib.lockbox_add_file_with_permissions(self._handle, encoded, len(encoded), data, len(data), permissions, replace):
            raise RuntimeError(_error(self._lib))

    def create_dir(self, path: str, create_parents: bool = True) -> None:
        encoded = path.encode()
        if not self._lib.lockbox_create_dir(self._handle, encoded, len(encoded), create_parents):
            raise RuntimeError(_error(self._lib))

    def remove_dir(self, path: str, recursive: bool = False) -> None:
        encoded = path.encode()
        if not self._lib.lockbox_remove_dir(self._handle, encoded, len(encoded), recursive):
            raise RuntimeError(_error(self._lib))

    def delete(self, path: str) -> None:
        encoded = path.encode()
        if not self._lib.lockbox_delete(self._handle, encoded, len(encoded)):
            raise RuntimeError(_error(self._lib))

    def rename(self, source: str, destination: str) -> None:
        source_bytes, destination_bytes = source.encode(), destination.encode()
        if not self._lib.lockbox_rename(self._handle, source_bytes, len(source_bytes), destination_bytes, len(destination_bytes)):
            raise RuntimeError(_error(self._lib))

    def read_range(self, path: str, offset: int, length: int) -> bytes:
        encoded = path.encode()
        return self._bytes(self._lib.lockbox_read_range(self._handle, encoded, len(encoded), offset, length))

    def list(self, path: str = "/", recursive: bool = False) -> list[LockboxEntry]:
        encoded = path.encode()
        payload = _wire_payload(self._bytes(self._lib.lockbox_list(self._handle, encoded, len(encoded), recursive)))
        result = messages.LockboxEntryList.FromString(payload)
        return [LockboxEntry.from_protobuf(value.SerializeToString()) for value in result.entries]

    def stat(self, path: str) -> LockboxEntry | None:
        encoded = path.encode()
        result = self._lib.lockbox_stat(self._handle, encoded, len(encoded))
        if not result.ptr:
            return None
        payload = _wire_payload(self._bytes(result))
        entry = messages.OptionalLockboxEntry.FromString(payload)
        return LockboxEntry.from_protobuf(entry.value.SerializeToString()) if entry.HasField("value") else None

    def set_variable(self, name: str, value: str, secret: bool = False) -> None:
        name_bytes, value_bytes = name.encode(), value.encode()
        if not self._lib.lockbox_set_variable(self._handle, name_bytes, len(name_bytes), value_bytes, len(value_bytes), secret):
            raise RuntimeError(_error(self._lib))

    def get_variable(self, name: str) -> str:
        encoded = name.encode()
        return self._bytes(self._lib.lockbox_get_variable(self._handle, encoded, len(encoded))).decode()

    def get_file(self, path: str) -> bytes:
        encoded = path.encode()
        return self._bytes(self._lib.lockbox_get_file(self._handle, encoded, len(encoded)))

    def move_variables(self, moves: messages.PathMoveList) -> None:
        value = moves.SerializeToString()
        if not self._lib.lockbox_move_variables(self._handle, value, len(value)):
            raise RuntimeError(_error(self._lib))

    def move_form_records(self, moves: messages.PathMoveList) -> None:
        value = moves.SerializeToString()
        if not self._lib.lockbox_move_form_records(self._handle, value, len(value)):
            raise RuntimeError(_error(self._lib))


class ContactKey:
    def __init__(self, library: ctypes.CDLL, handle: int):
        self._lib, self._handle = library, ctypes.c_void_p(handle)

    def __del__(self) -> None:
        if getattr(self, "_handle", None):
            self._lib.key_contact_free(self._handle)
            self._handle = None

    def public_bytes(self) -> bytes:
        return Lockbox._bytes(self, self._lib.key_contact_public(self._handle))

    def private_record(self) -> bytes:
        return Lockbox._bytes(self, self._lib.key_contact_private(self._handle))


class VaultDirectory:
    def __init__(self, library: ctypes.CDLL, handle: int):
        self._lib, self._handle = library, ctypes.c_void_p(handle)

    def __del__(self) -> None:
        if getattr(self, "_handle", None):
            self._lib.vault_directory_free(self._handle)
            self._handle = None

    def list_profile_generations(self, name: str) -> ProfileHistory:
        encoded = name.encode()
        frame = self._lib.vault_directory_list_profile_generations(self._handle, encoded, len(encoded))
        payload = _wire_payload(Lockbox._bytes(self, frame))
        value = messages.ProfileHistory.FromString(payload)
        generations = tuple(
            ProfileGeneration(
                index=generation.index,
                status=ProfileGenerationStatus(generation.status),
                contact_fingerprint=generation.contact_fingerprint,
                created_at_unix_ms=generation.created_at_unix_ms,
                retired_at_unix_ms=generation.retired_at_unix_ms if generation.has_retired_at else None,
            )
            for generation in value.generations
        )
        return ProfileHistory(
            name=value.name,
            active_generation=value.active_generation,
            generations=generations,
        )

    def list_form_revisions(self, type_id: str) -> messages.FormDefinitionList:
        encoded = type_id.encode()
        return _message(self, self._lib.vault_directory_list_form_revisions(
            self._handle, encoded, len(encoded)), messages.FormDefinitionList)


class ReadOnlyVaultDirectory:
    def __init__(self, library: ctypes.CDLL, handle: int):
        self._lib, self._handle = library, ctypes.c_void_p(handle)

    def close(self) -> None:
        if self._handle:
            self._lib.vault_read_only_free(self._handle)
            self._handle = None

    def __del__(self) -> None:
        self.close()

    def list_profile_names(self) -> messages.StringList:
        return _message(self, self._lib.vault_read_only_list_profile_names(self._handle), messages.StringList)

    def list_contact_names(self) -> messages.StringList:
        return _message(self, self._lib.vault_read_only_list_contact_names(self._handle), messages.StringList)

    def list_form_aliases(self) -> messages.StringList:
        return _message(self, self._lib.vault_read_only_list_form_aliases(self._handle), messages.StringList)

    def list_known_lockboxes(self) -> messages.KnownLockboxList:
        return _message(self, self._lib.vault_read_only_list_known_lockboxes(self._handle), messages.KnownLockboxList)


class Revault:
    """Owned entry point for the native lockbox and vault APIs."""
    def __init__(self, path: Optional[str | Path] = None):
        self.library = load(path)
        self._lib = self.library

    @property
    def lockbox_format_version(self) -> int:
        return self.library.lockbox_format_version()

    def probe_lockbox_format_version(self, value: bytes) -> int:
        return self.library.lockbox_probe_format_version(value, len(value))

    @property
    def current_vault_structure_version(self) -> int:
        return self.library.vault_structure_version_current()

    def probe_vault_structure_version(self, root: str, password: bytes) -> int:
        encoded = root.encode()
        return self.library.vault_directory_probe_structure_version(encoded, len(encoded), password, len(password))

    def last_error_details(self) -> messages.ErrorDetails:
        return _message(self, self.library.buffer_last_error_details(), messages.ErrorDetails)

    def open_read_only_vault(self, root: str, password: bytes) -> ReadOnlyVaultDirectory:
        encoded = root.encode()
        handle = self.library.vault_read_only_open(encoded, len(encoded), password, len(password))
        if not handle: raise RuntimeError(_error(self.library))
        return ReadOnlyVaultDirectory(self.library, handle)

    def open_default_read_only_vault(self, password: bytes) -> ReadOnlyVaultDirectory:
        handle = self.library.vault_read_only_open_default(password, len(password))
        if not handle: raise RuntimeError(_error(self.library))
        return ReadOnlyVaultDirectory(self.library, handle)

    def start_agent(self) -> None:
        if not self.library.vault_agent_start(): raise RuntimeError(_error(self.library))

    def put_vault_unlock_key(self, vault_id: str, key: bytes, ttl_seconds: int) -> None:
        value = vault_id.encode()
        if not self.library.vault_agent_put_vault_unlock_key(value, len(value), key, len(key), ttl_seconds):
            raise RuntimeError(_error(self.library))

    def get_vault_unlock_key(self, vault_id: str) -> bytes:
        value = vault_id.encode()
        return Lockbox._bytes(self, self.library.vault_agent_get_vault_unlock_key(value, len(value)))


def _error(lib: ctypes.CDLL) -> str:
    return lib.buffer_last_error().decode()


def _wire_payload(frame: bytes) -> bytes:
    if frame[:4] != b"LBWF" or len(frame) < 12:
        raise RuntimeError("invalid binding frame")
    length = int.from_bytes(frame[8:12], "big")
    if len(frame) != 12 + length:
        raise RuntimeError("invalid binding frame length")
    return frame[12:]


def _message(owner: object, frame: _Buffer, message_type: type):
    return message_type.FromString(_wire_payload(Lockbox._bytes(owner, frame)))


def _native_library_path() -> str:
    override = os.environ.get("REVAULT_LIBRARY")
    if override:
        return override
    machine = platform.machine().lower()
    arch = {"amd64": "x86_64", "x86_64": "x86_64", "arm64": "aarch64", "aarch64": "aarch64"}.get(machine)
    os_name = "macos" if sys.platform == "darwin" else "windows" if sys.platform == "win32" else "linux" if sys.platform.startswith("linux") else None
    if arch is None or os_name is None:
        raise RuntimeError(f"reVault does not publish a native library for {sys.platform}/{machine}")
    target = f"{os_name}-{arch}" + ("-gnu" if os_name == "linux" else "-msvc" if os_name == "windows" else "")
    filename = "revault_api.dll" if os_name == "windows" else "librevault_api.dylib" if os_name == "macos" else "librevault_api.so"
    bundled = Path(__file__).resolve().parent / "_native" / target / filename
    if not bundled.is_file():
        raise RuntimeError(
            f"revault-api native carrier is missing for {target}; "
            "set REVAULT_LIBRARY for development"
        )
    return str(bundled)


def load(path: Optional[str | Path] = None) -> ctypes.CDLL:
    """Load the native library and configure its ABI signatures."""
    lib = ctypes.CDLL(str(path or _native_library_path()))
    lib.api_abi_version.argtypes = []
    lib.api_abi_version.restype = ctypes.c_uint32
    if lib.api_abi_version() != 1:
        raise RuntimeError("revault-api native ABI mismatch; expected 1")
    try:
        from .revault_native import configure_native
    except ImportError:
        from revault_native import configure_native
    configure_native(lib, _Buffer)
    lib.lockbox_create.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    lib.lockbox_create.restype = ctypes.c_void_p
    _configure(lib, "lockbox_add_file", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_bool], ctypes.c_bool)
    _configure(lib, "lockbox_add_file_with_permissions", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_uint32, ctypes.c_bool], ctypes.c_bool)
    _configure(lib, "lockbox_commit", [ctypes.c_void_p], ctypes.c_bool)
    _configure(lib, "lockbox_create_dir", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_bool], ctypes.c_bool)
    _configure(lib, "lockbox_remove_dir", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_bool], ctypes.c_bool)
    _configure(lib, "lockbox_delete", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t], ctypes.c_bool)
    _configure(lib, "lockbox_rename", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_char_p, ctypes.c_size_t], ctypes.c_bool)
    _configure(lib, "lockbox_get_file", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t], _Buffer)
    _configure(lib, "lockbox_read_range", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_uint64, ctypes.c_uint64], _Buffer)
    _configure(lib, "lockbox_list", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_bool], _Buffer)
    _configure(lib, "lockbox_stat", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t], _Buffer)
    _configure(lib, "lockbox_set_variable", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_char_p, ctypes.c_size_t, ctypes.c_bool], ctypes.c_bool)
    _configure(lib, "lockbox_get_variable", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t], _Buffer)
    _configure(lib, "key_contact_generate", [], ctypes.c_void_p)
    _configure(lib, "key_contact_public", [ctypes.c_void_p], _Buffer)
    _configure(lib, "key_contact_private", [ctypes.c_void_p], _Buffer)
    _configure(lib, "key_contact_free", [ctypes.c_void_p], None)
    lib.lockbox_free.argtypes = [ctypes.c_void_p]
    _configure(lib, "vault_directory_list_profile_generations", [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_size_t], _Buffer)
    _configure(lib, "vault_directory_open_or_create", [ctypes.c_char_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p)
    _configure(lib, "vault_directory_free", [ctypes.c_void_p], None)
    lib.buffer_free.argtypes = [_Buffer]
    lib.buffer_last_error.restype = ctypes.c_char_p
    return lib


def _configure(lib: ctypes.CDLL, name: str, args: list[object], result: object) -> None:
    function = getattr(lib, name)
    function.argtypes, function.restype = args, result


# The generated facade is imported after the low-level loader and frame helpers
# are defined so every public native operation has an owned, class-oriented route.
from .facade import (  # noqa: E402,F401
    Agent,
    AgentActivity,
    ContactKeyPair,
    ContactPublicKey,
    LocalVault,
    Lockbox,
    Platform,
    ReadOnlyVaultDirectory,
    Revault,
    SigningKeyPair,
    SigningPublicKey,
    Vault,
    VaultDirectory,
    WrappedContactKey,
)


def generate_contact_key(library: Optional[ctypes.CDLL] = None) -> ContactKey:
    lib = library or load()
    handle = lib.key_contact_generate()
    if not handle:
        raise RuntimeError(_error(lib))
    return ContactKey(lib, handle)


def create(key: bytes, library: Optional[ctypes.CDLL] = None) -> Lockbox:
    lib = library or load()
    handle = lib.lockbox_create(key, len(key))
    if not handle:
        raise RuntimeError(_error(lib))
    return Lockbox(lib, handle)


def open_vault_directory(root: str, password: bytes, library: Optional[ctypes.CDLL] = None) -> VaultDirectory:
    lib = library or load()
    encoded = root.encode()
    handle = lib.vault_directory_open_or_create(encoded, len(encoded), password, len(password))
    if not handle:
        raise RuntimeError(_error(lib))
    return VaultDirectory(lib, handle)
