# reVault Dart bindings

Class-based Dart bindings for the complete reVault lockbox and vault API.
Structured values use generated Protobuf classes and all calls use the binary
LBWF ABI.

Release packages include the native library for Linux, macOS, and Windows on
x86-64 and ARM64. Use `await Vault.load()` for the packaged runtime, or pass an
explicit `DynamicLibrary` to `Vault(...)`. `REVAULT_LIBRARY` overrides native
discovery for development and controlled deployments.
