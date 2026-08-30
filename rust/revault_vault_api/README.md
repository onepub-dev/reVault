# revault_vault_api

Public Rust API for creating and managing reVault local vaults.

The Vault API stores Profiles, Contacts, signing keys, Lockbox access-directory
backups, and content-key integrations. It also provides platform credential
store integration and the local Session Agent.

Use it above `revault_lockbox_api` when an application needs local profile and
key lifecycle management, rather than only direct access to a `.lbox` archive.

See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
for the complete project overview.

## License

See the repository license for licensing terms.
