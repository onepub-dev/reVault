# Changelog

## 0.0.7 - 2026-09-02

- Align migration with `revault_lockbox_api` 0.0.9 and
  `revault_vault_api` 0.0.10 for the CLI 0.0.13 dependency chain.

## 0.0.6 - 2026-09-01

- Use `revault_lockbox_api 0.0.8` and `revault_vault_api 0.0.9` for the CLI
  0.0.12 migration path.

## 0.0.5 - 2026-08-30

- Use `revault_vault_api 0.0.8` so fresh migration-tool installations preserve
  the advertised Rust 1.88 minimum.

## 0.0.4 - 2026-08-30

- Restored the minimum supported Rust version to Rust 1.88.
- Preserve explicit directories, portable directory and symlink permissions,
  and archive descriptions through archive migration schema 2.
- Create imported vault containers with the restored active default signing
  key, keeping the container's established signer consistent with its restored
  profile records when the vault is reopened.
- Added an encrypted journal frame for the generated per-migration artifact
  key, allowing direct migrations to resume without requiring a user-supplied
  migration password. The runtime key remains in `SecretVec`; it is not
  serialized through an ordinary byte field.
