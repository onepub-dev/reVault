# reVault format 2 migration exporter

This package contains the isolated archive and vault exporters for archive/container format 2. Historical readers are private bundled implementation modules; applications should use `revault-api`.

Use `lockbox doctor migrate` to select the exporter automatically. The exporter reads historical data and writes an encrypted migration artifact; it does not import into the current format.

See `PROVENANCE.json` for the frozen implementation sources.

The archive executable also supports explicit forensic recovery through `lockbox doctor recover`. A dry-run scan requires original opening credentials. Writing a recovered copy also requires the established owner's signing key. Recovery preserves readable access records and writes a separate current-format copy; damaged entries may be omitted. The original archive is retained.

Supported inputs: Archive format 2 and Vault structure 2 inside archive/container format 2.

Executables: `revault-migrate-archive-v2` and `revault-migrate-vault-v2`. The package version is an exporter software version, not the source archive format number.
