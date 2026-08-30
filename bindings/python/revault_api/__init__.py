"""Encrypt files, variables, and typed records in portable reVault lockboxes.

``Revault`` loads the native runtime for lockboxes, keys, the Session Agent,
and platform services; ``Vault`` is the persistent encrypted metadata store.
Owned handles are context
managers; secret values use callback-scoped accessors to limit plaintext
lifetime.

Installation, security guidance, and examples are in the repository README:
https://github.com/onepub-dev/reVault#readme
"""
from __future__ import annotations

import ctypes
import os
import platform
import sys
from pathlib import Path

from ._domain import FormField, PathMove


class _Buffer(ctypes.Structure):
    _fields_ = [("ptr", ctypes.POINTER(ctypes.c_uint8)), ("len", ctypes.c_size_t)]


def _error(library: ctypes.CDLL) -> str:
    return library.buffer_last_error().decode()


def _native_library_path() -> str:
    machine = platform.machine().lower()
    arch = {
        "amd64": "x86_64",
        "x86_64": "x86_64",
        "arm64": "aarch64",
        "aarch64": "aarch64",
    }.get(machine)
    os_name = (
        "macos"
        if sys.platform == "darwin"
        else "windows"
        if sys.platform == "win32"
        else "linux"
        if sys.platform.startswith("linux")
        else None
    )
    if arch is None or os_name is None:
        raise RuntimeError(
            f"reVault does not publish a native library for {sys.platform}/{machine}"
        )
    suffix = "-gnu" if os_name == "linux" else "-msvc" if os_name == "windows" else ""
    target = f"{os_name}-{arch}{suffix}"
    filename = (
        "revault_api.dll"
        if os_name == "windows"
        else "librevault_api.dylib"
        if os_name == "macos"
        else "librevault_api.so"
    )
    bundled = Path(__file__).resolve().parent / "_native" / target / filename
    if not bundled.is_file():
        raise RuntimeError(
            f"revault-api native carrier is missing for {target}; "
            "install the matching revault-api native carrier for this platform"
        )
    return str(bundled)


def _load(native_library_path: str | os.PathLike[str] | None = None) -> ctypes.CDLL:
    """Load and validate the selected version-matched native library."""
    if native_library_path is not None and not os.fspath(native_library_path):
        raise ValueError("native_library_path must not be empty")
    inherited = os.environ.get("REVAULT_LIBRARY")
    selected = (
        os.fspath(native_library_path)
        if native_library_path is not None
        else inherited
        if inherited
        else _native_library_path()
    )
    if sys.platform == "win32" and Path(selected).name == selected:
        for directory in os.get_exec_path():
            candidate = Path(directory) / selected
            if candidate.is_file():
                selected = str(candidate.resolve())
                break
    library = ctypes.CDLL(selected)
    library.api_abi_version.argtypes = []
    library.api_abi_version.restype = ctypes.c_uint32
    if library.api_abi_version() != 3:
        raise RuntimeError("revault-api native ABI mismatch; expected 3")

    from ._revault_native import configure_native

    configure_native(library, _Buffer)
    return library


# Imported after the loader helpers because the generated facade uses them.
from .facade import (  # noqa: E402
    AgentActivityKind,
    AgentSession,
    AgentActivity,
    ContactKeyPair,
    ContactPublicKey,
    Lockbox,
    ProfileSigningKeyPair,
    ProfileSigningPublicKey,
    ReadOnlyVault,
    RevaultError,
    Revault,
    Vault,
    WrappedContactKey,
    SecretBytes,
    SecretString,
    LockboxCacheMode,
    LockboxWorkload,
    LockboxWorker,
    KeyExportFormat,
)

__all__ = [
    "AgentSession",
    "AgentActivityKind",
    "AgentActivity",
    "ContactKeyPair",
    "ContactPublicKey",
    "Lockbox",
    "Vault",
    "ProfileSigningKeyPair",
    "ProfileSigningPublicKey",
    "ReadOnlyVault",
    "Revault",
    "WrappedContactKey",
    "SecretString",
    "SecretBytes",
    "RevaultError",
    "LockboxCacheMode",
    "LockboxWorkload",
    "LockboxWorker",
    "KeyExportFormat",
    "FormField",
    "PathMove",
]
