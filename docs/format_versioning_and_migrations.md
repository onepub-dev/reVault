# Format versioning and migrations

The [release compatibility contract](../AGENTS.md#release-version-compatibility-contract)
defines compatibility lines. Before 1.0 a line is `0.minor`; from 1.0 onward
it is the major version. Archives and persisted Vault data must remain readable
in both directions within that line. A breaking persisted-format change needs
a new line and validation of supported migration paths.

Archive formats, Vault structure/container formats and package release numbers
are separate identifiers. A version bump alone does not demonstrate compatibility
or release readiness.

## Maintained references

- [Versions and compatibility](../manual/develop-with-revault/compatibility.md)
  describes the manual's release scope and user-facing contract.
- [Migration procedure](../manual/maintain-and-recover/migrating-between-versions.md)
  describes explicit migration and source-preservation behavior.
- [Retained fixture inventory](../rust/revault_migration/tests/retained/README.md)
  records historical writers and the logical contents of immutable test archives.
- [V4 work entry point](archive_v4_plan.md) locates development plans and
  outstanding qualification evidence.

## Engineering requirements

Keep historical readers/exporters versioned and preserve native fixtures with
their producing versions and expected logical records. Verify migrations through
independent reopen/read operations, including subsequent supported mutations.
Validate CLI/binding interoperability and persisted Vault compatibility across
the supported release combinations. Preserve the source until the replacement
has been validated and installed safely.

Exporter registrations must identify available, compatible versions. Check
the actual dependency graph and package availability before publishing; do not
reuse an old prose publication order as a release checklist.

The [previous engineering narrative](https://github.com/onepub-dev/reVault/blob/bc85ea5c3957cf3421da713f5dd4b16965771dce/docs/format_versioning_and_migrations.md)
is retained in Git history. Its assertions about the then-current format,
retained readers, fixtures and release order are not current qualification evidence.
