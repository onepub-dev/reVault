# Dependency duplicate review

Reviewed with:

```bash
cargo tree -d
```

Current findings:

- `block-buffer`, `crypto-common`, `digest`, `hkdf`, `hmac`, and `sha2` exist in two generations.
- The older generation is pulled by stable `argon2 0.5`, `ed25519-dalek 2`, `x25519-dalek 2`, and `keyring`/`secret-service` transitive dependencies.
- The newer generation is pulled by direct `sha2 0.11`, `hkdf 0.13`, `chacha20poly1305 0.11`, `ml-dsa`, and `ml-kem`.
- `signature` exists as `2.2` for `ed25519-dalek 2` and `3.0` for `ml-dsa`.
- `getrandom` exists as `0.2` through `ring`/`rustls`/`secret-service` and `0.4` through direct dependencies and modern crypto crates.
- `cipher` exists as `0.4` via `secret-service` and `0.5` via direct AEAD usage.
- `winnow`/`toml_parser`/`toml_datetime` duplicates come from proc-macro/config tooling and are not runtime-sensitive for lockbox operations.

Assessment:

- No direct dependency action is recommended right now.
- The duplicate crypto stacks are a consequence of choosing stable production releases for dalek/argon2 while using newer production releases for the lockbox symmetric/PQ stack.
- Returning dalek/argon2 to RC releases would reduce some duplication but would conflict with the production-release requirement.
- `keyring 4.1.3` removed the old `dbus-secret-service` dependency path and moved Linux keyring storage to the newer `zbus` path, which is a net improvement.
- Remaining `ring`/`rustls` dependencies come from HTTPS support in `ureq` and SMTP TLS support in `lettre`; they provide clear value.

Revisit when:

- Stable `ed25519-dalek`/`x25519-dalek` releases move to the newer crypto trait stack.
- `argon2` publishes a non-RC release on the newer stack.
- `secret-service` or `keyring` removes older crypto-generation dependencies.
