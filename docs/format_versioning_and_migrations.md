# Format versioning and migrations

Vault and archive formats are versioned independently. A release may change one
without changing the other, and users do not have to migrate both at the same
time.

The lockbox API creates v3 archives and retains a v2 reader. The vault API
requires the current vault structure (v3). Unsupported older formats return an
error containing the command needed to migrate them. Legacy
native readers belong in a versioned migration exporter, not in the current
lockbox or vault API.

## Migration flow

The normal command migrates directly to the latest format:

```console
lockbox doctor migrate vault --output ~/.local/share/lockbox/vault-current
lockbox doctor migrate lockbox secrets.lbox --output secrets-current.lbox
```

Archive migration requires a vault that is already in the current format. The
new archive is signed with the current vault owner signing key. Initialize or
migrate the vault first when necessary; archives are not migrated with a
one-off signing key.

To replace the source, use `--replace`:

```console
lockbox doctor migrate vault --replace
lockbox doctor migrate lockbox secrets.lbox --replace
```

Replacement is deliberately explicit. The completed output is validated first,
the source is renamed to a versioned backup, and the new artifact is then renamed
into place. The backup is retained. Without `--replace`, `--output` is required
and an existing destination is never overwritten.

For an old native format, the current executable installs the registered
historical exporter from crates.io using an exact version. Vault v1 uses
`revault_migrate_vault_v1`; archive v1 uses
`revault_migrate_archive_v1`.
That executable reads the old native format and writes an encrypted, streaming
migration artifact. The current executable upgrades that migration schema one
version at a time and imports it into the latest native format (currently v3).

The encrypted export/import boundary means a current release never needs to
carry every historical native reader. It also allows vaults and archives to be
exported with the release that understands them and imported later with a newer
release.

Advanced `export`, `upgrade`, `import`, and `verify` commands are intentionally
hidden from ordinary help. Use `lockbox --verbose --help` and the corresponding
verbose subcommand help when diagnosing or manually controlling a migration.

## Security and data rules

- Migration artifacts and journals are encrypted and authenticated.
- Decrypted keys, passwords, key directories, and secret record values use
  secure or zeroizing buffers while they are serialized, upgraded, or
  imported. The resumability journal keeps its generated artifact key in a
  locked `SecretVec` at runtime and writes it as a separate encrypted frame,
  rather than as an ordinary serialized byte field.
- Direct migrations generate a random migration key. It is sent to a historical
  exporter over the private stdin protocol, never as a command argument, and is
  retained only in the encrypted source-bound journal for resumability. Manual
  export/import commands continue to use an explicit artifact passphrase.
  Lockbox exporters first try the Profile key generations in the migrated
  current-format Vault. `LOCKBOX_PASSWORD` is used only when the Vault cannot
  provide access and the Lockbox has password access. A key already cached by
  the Session Agent remains an optional fast path.
- Archive file bodies are processed as bounded chunks and must never be loaded
  wholly into memory.
- Import creates a new archive commit chain. Old public commits and signatures
  are not copied into the new native archive.
- Temporary output is validated before replacement. Interrupted work retains an
  encrypted journal and is resumed only when its source path, format version, and
  source fingerprint still match.

Resumption occurs at authenticated stage boundaries. A completed export,
upgrade, or import is verified and reused even if the process stopped before its
journal update. An incomplete stage is discarded and streamed again; completed
earlier stages are not repeated. Replacement also recovers the crash window
between renaming the source to its retained backup and moving the validated
output into place.

## Adding a format version

Every native format change must include all of the following:

1. Increment only the affected vault or archive native version.
2. Preserve a stable, version-independent probe that can identify the version
   without parsing version-specific records.
3. Add the logical migration schema step from `n` to `n + 1`; do not skip steps.
4. Publish or retain a crates.io exporter version that can read the old native
   format and register the exact exporter version in the current CLI.
5. Add fixtures produced by the old release and tests for export, every upgrade
   step, import, verification, interruption/resume, corruption, wrong passwords,
   and replacement rollback.
6. Test logical equivalence rather than native byte equality. Archive migration
   intentionally produces a new commit/signature history.

The current native format is v3 for both the vault structure and lockbox archive.
The v1 migration tests cover the complete v1 export, migration-schema upgrade,
and import path into that current format. The archive fixture exercises the full
record families: descriptions, directories, files, symlinks, normal and secret
variables, form definitions and revisions, all eight form field kinds,
permissions, and access keys. Hidden mirror variables are decoded as projects:
both removal policies, source/destination, selection rules, host identity, and
managed file bytes are checked. Legacy projects default the newer `strict`
setting to false. A subsequent signed update verifies persisted mirror ownership.

A CLI test creates a strict mirror, copies files, migrates it, compares the
project configuration, and performs unchanged and changed updates. Separate CLI
reads verify additions, replacements, removals, and original archive content.
The v2 archive test uses a pinned historical writer and asserts both source and
destination versions. The vault round trip includes contacts and signing keys,
known lockboxes, access labels, remembered passwords, key-directory backups,
key profiles, password profiles, and form definitions.

For the user-facing procedure and command examples, see the
[vault and archive migration guide](migration_guide.md).

## Permanent fixtures and release gate

The repository retains immutable native fixtures in
[`rust/revault_migration/tests/retained`](../rust/revault_migration/tests/retained/README.md).
There are archive fixtures for v1, v2 and v3, and vault fixtures for structures
v1, v2 and v3. Structure v2 has both its container-v1 and container-v2 fixtures.
Each contains the record families available to its producing writer, with
synthetic credentials, writer provenance, native-byte SHA-256 and expected
logical records. The files are retained in Git and release tags; normal tests
read them without rebuilding or replacing them.

`retained_fixtures` runs in the release candidate's existing Rust migration test
group. Every retained source is exported with its appropriate reader, upgraded,
imported into current format, reopened, and compared against the saved records.
It also tests subsequent mirror and vault mutations. The test fails if any
historical combination is missing, a native version lacks a fixture, or the
current vault/container combination has not been retained.

Keep all previous fixtures when adding a version. Add a new generator with the
exact historical/current writer and register its reader before retiring old
native support. The explicit ignored generator refuses to overwrite existing
fixtures; generation is never part of release validation. Review new fixtures
and extend the inventory assertions for newly supported content types. The corpus
and its dedicated test are excluded from the published library package to avoid
shipping test archives to consumers; repository release tests always include them.

## crates.io release order

crates.io is the authoritative source for historical exporters. Because releases
are immutable, migration-related crates must be published in dependency order:

1. `revault_lockbox_api`
2. `revault_migration_format`
3. `revault_migrate_vault_v1`
4. `revault_vault_api`
5. `revault_migrate_archive_v1`
6. `revault_migration`
7. `revault_publish_protocol`
8. `revault_cli`

The exporter registry in the CLI must refer to an exact published
exporter version. Do not register a version until that package and all
of its exact API dependencies are visible on crates.io. A crate package version
change is independent of the vault and archive native format versions; publishing
API version `0.0.3`, for example, does not itself change either persisted format.
