# revault_vault_api

Public Rust API for creating and managing reVault local vaults.

The Vault API stores Profiles, Contacts, signing keys, Lockbox access-directory
backups, and content-key integrations. It also provides platform credential
store integration and the local Session Agent.

Use it above `revault_lockbox_api` when an application needs local profile and
key lifecycle management, rather than only direct access to a `.lbox` archive.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## Lockbox aliases

`VaultDirectory` exposes `set_lockbox_alias`, `list_lockbox_aliases`,
`resolve_lockbox_alias` and `remove_lockbox_alias`. Read-only Vaults can list
and resolve aliases. Each alias maps a case-sensitive ASCII name to a stable
`LockboxId`; resolution obtains the current path from the known-Lockbox records.
The caller must verify that identity against the opened target.

Aliases are optional files at `/lockbox_aliases/<name>.lbla`. Each contains
`LBLA`, a little-endian `u16` record version of 1, and the 16-byte Lockbox ID.
The existing known-Lockbox codec and Vault structure version remain unchanged.
Whole-Vault backup and restore preserve the collection. Alias records do not
contain credentials or change the Lockbox archive format.

## License

See the repository license for licensing terms.
