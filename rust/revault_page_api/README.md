# revault_page_api

**Deprecated:** use [`revault_secure_memory`](https://crates.io/crates/revault_secure_memory)
for new development. Historical releases remain available for existing pinned
migration dependencies. This package is no longer part of routine releases.

Secure, page-pooled byte and string storage primitives for reVault APIs.

The crate provides zeroizing secure buffers, guarded access, page allocation,
and platform-aware memory protection for applications that handle secrets.
It is the low-level memory-safety building block used by the higher-level
lockbox and vault APIs; it does not create or open lockboxes itself.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## License

See the repository license for licensing terms.
