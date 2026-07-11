# revault_page_api

Secure, page-pooled byte and string storage primitives for reVault APIs.

The crate provides zeroizing secure buffers, guarded access, page allocation,
and platform-aware memory protection for applications that handle secrets.
It is the low-level memory-safety building block used by the higher-level
lockbox and vault APIs; it does not create or open lockboxes itself.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## License

See the repository license for licensing terms.
