# Migrating a reVault vault or archive

reVault vaults and archives have independent on-disk formats and may be
upgraded at different times. Migrate the vault first: archive migration uses
the profile and signing keys from the current-format vault.

Use the `lockbox migrate` command when a newer reVault release reports that a
vault or archive uses an older format. The command migrates to the latest
format supported by the installed CLI.

## Before you start

Make sure that:

- no other reVault process is writing to the vault or archive;
- you know the vault pass phrase, unless the platform key store or session
  agent can provide it;
- for a password-only archive, you know its archive password; and
- you have enough free disk space for a complete new copy.

Migration does not delete the source when an output path is supplied. It
creates and validates the new artifact first, so the source can be kept until
you have confirmed the result.

## Migrate a vault

The vault command operates on the configured default vault. Always provide an
output directory for the first migration so that the original vault remains
untouched:

```console
lockbox migrate vault --output ~/.local/share/lockbox/vault-migrated
```

The CLI obtains the vault pass phrase from the platform key store, the session
cache, or `LOCKBOX_VAULT_PASSWORD`. If none is available, it prompts for the
pass phrase.

For example, in automation where the password is supplied by a protected
secret store:

```console
LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox migrate vault --output "$WORK_DIR/vault-migrated"
```

The CLI generates and manages a temporary key for the encrypted migration
artifacts. It is retained only in the encrypted resumability journal, which is
protected by the vault credential. No migration password is required for a
direct migration.

After migration, the new vault is at the path passed to `--output`. Point the
CLI at that vault according to your normal vault configuration, then verify
that profiles, forms, variables, and keys are present before removing the old
vault.

## Migrate an archive

Pass the archive path and a separate output path:

```console
lockbox migrate archive secrets.lbox --output secrets-migrated.lbox
```

The migration obtains access to the source archive automatically using the
keys in the migrated vault. A password is needed only when the archive is
password-only.

The CLI generates and manages the migration key automatically. The archive
migration also needs a vault containing the keys used to manage the archive.
A newly initialized vault has newly generated keys and is not a replacement
for the original vault.

If the original vault is missing, restore it from a backup or recreate its
profiles by importing the saved private keys before migrating the archive. For
example:

```console
lockbox vault init
lockbox vault import-key legacy alice.key alice.pub
```

Repeat the import for each saved key required by the vault. If neither a vault
backup nor the saved private keys are available, recover those first; do not
continue with a newly initialized vault.

Once the original vault has been restored, migrate it if it is older than the
current format:

```console
lockbox migrate vault --replace
lockbox migrate archive secrets.lbox --output secrets-migrated.lbox
```

For automation, provide the vault password through the protected environment.
Add `LOCKBOX_PASSWORD` only when the archive is password-only:

```console
LOCKBOX_PASSWORD="$ARCHIVE_PASSWORD" \
  LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox migrate archive secrets.lbox --output secrets-migrated.lbox
```

Open the migrated archive and check important paths before replacing or
removing the original:

```console
lockbox open secrets-migrated.lbox
lockbox list secrets-migrated.lbox /
lockbox cat secrets-migrated.lbox /path/to/important-file
lockbox close secrets-migrated.lbox
```

Archive migration creates a new signed commit chain. The files, forms, and
other logical records are migrated, but the old archive's public commit and
signature history is not copied into the new archive. This is intentional:
the new archive is freshly written and signed using the current format and
current signing material.

## Replacing the source in place

Once you have tested a migration, `--replace` can perform the replacement
automatically:

```console
lockbox migrate vault --replace
lockbox migrate archive secrets.lbox --replace
```

`--replace` cannot be combined with `--output`. The CLI validates the migrated
artifact, renames the original to a versioned backup, and then renames the
new artifact into the original location.

For example, replacing `secrets.lbox` from archive format version 1 retains a
backup similar to:

```text
secrets.lbox.v1.pre-migration
```

Do not delete the backup until the replacement has been opened and checked.
The backup is also useful if another application still needs the old file.

Without `--replace`, `--output` is required. Existing output paths are not
overwritten.

## Older formats and historical exporters

The current lockbox and vault APIs intentionally read only their current
native formats. When the CLI encounters an older format, the direct migration
command automatically:

1. detects the source format version;
2. installs the exact historical exporter registered for that version from
   crates.io, if it is not already cached;
3. exports the old native records to an encrypted streaming migration
   artifact;
4. upgrades the migration schema one step at a time; and
5. imports and validates a new current-format vault or archive.

The first supported migration is from native format v1 to v2. The historical
exporters are installed automatically as needed:

- vault v1: `revault_migrate_vault_v1`;
- archive v1: `revault_migrate_archive_v1`.

The first migration may therefore require network access and a working Cargo
installation. The exporter is cached under the user cache directory and is
checked for the expected artifact type, native version, and migration schema
before it is used.

If the machine cannot access crates.io, install the matching exporter by some
other means and pass its executable path with the advanced `--exporter` option.
The exporter must be the exact reader for the source format; do not use an
exporter from a different native version.

## Resuming an interrupted migration

Migration is resumable. The CLI stores an encrypted migration journal and
temporary artifacts beside the source. If the process is interrupted, repeat
the same command:

```console
lockbox migrate archive secrets.lbox --output secrets-migrated.lbox
```

Completed export, upgrade, and import stages are verified and reused. An
incomplete stage is discarded and rebuilt. The CLI refuses to resume when the
source path, source format version, or source contents no longer match the
saved journal.

If the process stopped during `--replace`, run the same replacement command
again. The CLI detects the interrupted replacement and completes the safe
rename when the retained backup and validated output are available.

Do not manually remove `.revault-migration-*` directories while a migration is
being resumed. After a successful migration, the CLI removes its temporary
working directory.

## Manually controlling migration stages

The normal direct command is recommended. The lower-level export, upgrade,
import, and verify commands are hidden from ordinary help because they are
primarily useful for diagnostics, transferring an encrypted migration artifact
between machines, or testing a migration step.

To view them:

```console
lockbox --verbose --help
lockbox --verbose migrate vault --help
lockbox --verbose migrate archive --help
```

The stage commands use encrypted migration artifacts and require an explicit
migration artifact pass phrase. This is intentional: a manually exported
artifact may outlive the source vault and may be transferred to another
machine. For example, to verify an artifact without importing it:

```console
lockbox migrate vault verify vault.migration
lockbox migrate archive verify archive.migration
```

For a manually staged migration, keep the artifact files private and transfer
the migration pass phrase through a separate secure channel. Do not put vault,
archive, or migration passwords in command-line arguments, because command
arguments may be visible to other processes.

## Troubleshooting

### The CLI says the format version is unsupported

Run the matching direct command and provide a destination:

```console
lockbox migrate vault --output ./vault-v2
lockbox migrate archive old-secrets.lbox --output ./old-secrets-v2.lbox
```

If you want the original replaced after validation, use `--replace` instead.

### The migration cannot open the vault

Check the vault pass phrase and whether the platform key store or session agent
is available. You can provide the pass phrase explicitly for one invocation:

```console
LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox migrate vault --output ./vault-v2
```

### The migration cannot open the archive

Confirm that you migrated or restored the correct vault and that all saved
profile keys were imported.

If the archive is password-only, provide its password through
`LOCKBOX_PASSWORD`:

```console
LOCKBOX_PASSWORD="$ARCHIVE_PASSWORD" \
  lockbox migrate archive old-secrets.lbox --output ./old-secrets-v2.lbox
```

### The exporter cannot be installed

The current CLI needs the historical exporter for an old native format. Check
network access to crates.io and that Cargo is installed. If installation is
not possible, obtain the exact exporter binary through your deployment system
and pass it with the advanced `--exporter <path>` option.

### The destination already exists

Choose a new output path. Migration never overwrites an existing destination:

```console
lockbox migrate archive secrets.lbox --output secrets-v2-new.lbox
```

### The source changed while migration was in progress

The saved source fingerprint no longer matches. Remove or rename the partial
destination if necessary, decide which source copy is authoritative, and
start a new migration from that unchanged source.
/
