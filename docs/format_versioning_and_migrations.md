# Format versioning and migrations

Vault and archive formats are versioned independently. A release may change one
without changing the other, and users do not have to migrate both at the same
time.

The lockbox and vault APIs support only their current native formats. When an
older format is encountered they stop before interpreting version-specific
records and return an error containing the command needed to migrate it. Legacy
native readers belong in a versioned migration exporter, not in the current
lockbox or vault API.

## Migration flow

The normal command migrates directly to the latest format:

```console
lockbox migrate vault --output ~/.local/share/lockbox/vault-v2
lockbox migrate archive secrets.lbox --output secrets-v2.lbox
```

Archive migration requires a vault that is already in the current format. The
new archive is signed with the current vault owner signing key. Initialize or
migrate the vault first when necessary; archives are not migrated with a
one-off signing key.

To replace the source, use `--replace`:

```console
lockbox migrate vault --replace
lockbox migrate archive secrets.lbox --replace
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
version at a time and imports it into the latest native format.

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
  Archive exporters first try the profile key generations in the migrated
  current-format vault. `LOCKBOX_PASSWORD` is used only for password-only
  archives; an existing session-agent content key remains an optional fast
  path.
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

Vault native format v2 is the first migration target and the reference test case
for this policy.

For the user-facing procedure and command examples, see the
[vault and archive migration guide](migration_guide.md).

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
