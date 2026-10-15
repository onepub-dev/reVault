# reVault format 3 migration exporter

This package contains the isolated archive and vault exporters for archive/container format 3. Historical readers are private bundled implementation modules; applications should use `revault-api`.

Use `lockbox doctor migrate` to select the exporter automatically. The exporter reads historical data and writes an encrypted migration artifact; it does not import into the current format.

See `PROVENANCE.json` for the frozen implementation sources.

Supported inputs: Archive format 3 and Vault structure 3 inside archive/container format 3. Vault container rewrites preserve aliases and unknown records without changing the Vault structure.

Executables: `revault-migrate-archive-v3` and `revault-migrate-vault-v3`. The package version is an exporter software version, not the source archive format number.
