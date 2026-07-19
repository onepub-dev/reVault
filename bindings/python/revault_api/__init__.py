"""Encrypt files, variables, and typed records in portable reVault lockboxes.

``Vault`` is the entry point for lockboxes, keys, local vault metadata, the
session agent, and the platform secret store. Owned handles are context
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
from typing import Optional

from . import revault_bindings_pb2 as messages


class _Buffer(ctypes.Structure):
    _fields_ = [("ptr", ctypes.POINTER(ctypes.c_uint8)), ("len", ctypes.c_size_t)]


def _error(library: ctypes.CDLL) -> str:
    return library.buffer_last_error().decode()


def _wire_payload(frame: bytes) -> bytes:
    if frame[:4] != b"LBWF" or len(frame) < 12:
        raise RuntimeError("invalid binding frame")
    length = int.from_bytes(frame[8:12], "big")
    if len(frame) != 12 + length:
        raise RuntimeError("invalid binding frame length")
    return frame[12:]


def _native_library_path() -> str:
    override = os.environ.get("REVAULT_LIBRARY")
    if override:
        return override
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
            "set REVAULT_LIBRARY for development"
        )
    return str(bundled)


def load(path: Optional[str | Path] = None) -> ctypes.CDLL:
    """Load and validate the version-matched ABI-v2 native library."""
    library = ctypes.CDLL(str(path or _native_library_path()))
    library.api_abi_version.argtypes = []
    library.api_abi_version.restype = ctypes.c_uint32
    if library.api_abi_version() != 2:
        raise RuntimeError("revault-api native ABI mismatch; expected 2")

    from ._revault_native import configure_native

    configure_native(library, _Buffer)
    return library


# Imported after the loader helpers because the generated facade uses them.
from .facade import (  # noqa: E402
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

__all__ = [
    "Agent",
    "AgentActivity",
    "ContactKeyPair",
    "ContactPublicKey",
    "LocalVault",
    "Lockbox",
    "Platform",
    "ReadOnlyVaultDirectory",
    "Revault",
    "SigningKeyPair",
    "SigningPublicKey",
    "Vault",
    "VaultDirectory",
    "WrappedContactKey",
    "load",
    "messages",
]
