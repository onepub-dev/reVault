# reVault format 1 migration exporter

This package contains the isolated archive and vault exporters for archive/container format 1. Historical readers are private bundled implementation modules; applications should use `revault-api`.

Use `lockbox doctor migrate` to select the exporter automatically. The exporter reads historical data and writes an encrypted migration artifact; it does not import into the current format.

See `PROVENANCE.json` for the frozen implementation sources.

Supported inputs: Archive format 1 and Vault structures 1 or 2 inside archive/container format 1.

Executables: `revault-migrate-archive-v1` and `revault-migrate-vault-v1`. The package version is an exporter software version, not the source archive format number.
