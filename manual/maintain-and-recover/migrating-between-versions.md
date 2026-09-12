# Migrating between versions

reVault aims to have a stable format for archives and the vault.

From time to time, a format revision may be necessary. reVault includes migration commands for upgrading Vaults and Lockboxes between versions.

Vault structure version 3 adds password Profiles. Upgrade an existing version 2 Vault before using this feature:

```bash
lbx doctor migrate vault --replace
```

The migration retains the previous Vault as a versioned backup. Older clients reject version 3 Vaults for normal Vault operations, protecting password Profiles from tools that do not understand them. Vault structure version 3 is separate from its Lockbox container version. The v4 development implementation retains Vault structure version 3 but upgrades its container to v4. Standalone older Lockboxes must also be migrated before the v4 client can open them.

## Migrating a reVault vault or archive

reVault vaults and archives use independent on-disk formats. You can migrate each independently, however your vault must be migrated first.

Use the `lockbox doctor migrate` command when a newer reVault release reports that a vault or archive uses an older format. `doctor` groups health checks, recovery, and format maintenance in one place. The migration command upgrades Lockboxes and the Vault to the latest format supported by the installed CLI.

You don't need to migrate all of your archives at once, but you will need to migrate them before you can access an archive with the latest CLI version.

### Before you start

Make sure that:

* no other reVault process is writing to the vault or archive;
* you know the Vault passphrase, unless the platform credential store can provide it;
* the migrated Vault contains a Profile key or remembered Lockbox password that can open the Lockbox; if it does not, and the Lockbox has password access, you know that password; and
* you have enough free disk space for a complete new copy of the vault or archive you are migrating.

Migration does not delete the source when an output path is supplied. It creates and validates the new artifact first, so the source can be kept until you have confirmed the result.

### Migrate a vault

The vault command operates on the configured default vault.

To migrate your Vault and replace the existing file, use:

```
lockbox doctor migrate vault --replace
```

If you want a safer path you can do a migration to a separate vault via:

```console
lockbox doctor migrate vault --output ~/.local/share/lockbox/vault-migrated
```

The CLI obtains the Vault passphrase through the normal Vault access flow, or from `LOCKBOX_VAULT_PASSWORD` in agentless automation. If none is available, it prompts for the passphrase.

For example, in automation where the password is supplied by a protected secret store:

```console
LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox doctor migrate vault --output "$WORK_DIR/vault-migrated"
```

Temporary file artifacts created to facilitate the migration are encrypted with a temporary migration key.

After migration, the new vault is at the path passed to `--output`. Point the CLI at that vault according to your normal vault configuration, then verify that profiles, forms, variables, and keys are present before removing the old vault.

### Migrate an archive

Before you migrate an archive you must first migrate the local vault.

Pass the archive path and a separate output path:

```console
lockbox doctor migrate lockbox secrets.lbox --output secrets-migrated.lbox
```

The CLI first tries the current and historical Profile keys stored in the migrated Vault. It can also use a Lockbox password remembered by the Vault. You only need to supply a password when the Vault does not hold a credential that can open the Lockbox and the Lockbox has password access.

The vault must already exist and be in the current format. If the vault exists use the above vault migration guide to migrate the vault.

If the Vault does not exist, restore a Vault backup or initialise a new Vault and restore the required Profile backups with `lbx vault profile restore`. A newly initialised Vault does not contain the old Profile keys and cannot open Lockboxes that relied on them.

You can now migrate your archives:

`lockbox doctor migrate lockbox secrets.lbox --output secrets-migrated.lbox`

For agentless automation, provide the Vault passphrase through a protected environment variable:

```console
LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox doctor migrate lockbox secrets.lbox --output secrets-migrated.lbox
```

Add `LOCKBOX_PASSWORD` only when the Vault does not contain a Profile key or remembered Lockbox password that can open the Lockbox:

```console
LOCKBOX_PASSWORD="$ARCHIVE_PASSWORD" \
  LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox doctor migrate lockbox secrets.lbox --output secrets-migrated.lbox
```

Open the migrated archive and check important paths before replacing or removing the original:

```console
lockbox secrets-migrated.lbox open
lockbox secrets-migrated.lbox list /
lockbox secrets-migrated.lbox cat /path/to/important-file
lockbox secrets-migrated.lbox close
```

Archive migration creates a new signed commit chain. The files, forms, and other logical records are migrated, but the old archive's public commit and signature history is not copied into the new archive. The new archive uses the current format. For signed v2/v3 archives it retains the established signing owner; the matching signing key must be available in the migrated Vault, including historical Profile generations. Missing signing material is an error rather than permission to substitute a different owner. Only committed reachable contents are imported; abandoned allocations and free space are omitted.

### Replacing the source in place

If you have backups of your archives then you can use the `--replace` switch perform the replacement automatically:

```console
lockbox doctor migrate vault --replace
lockbox doctor migrate lockbox secrets.lbox --replace
```

`--replace` cannot be combined with `--output`. The CLI validates the replacement before renaming the original to a versioned backup and installing the new artifact at the original location. For a Lockbox, validation includes reopening it, checking physical ownership, and comparing logical records and file bytes against the migration artifact.

For example, replacing `secrets.lbox` from archive format version 1 retains a backup similar to:

```
secrets.lbox.v1.pre-migration
```

Do not delete the backup until the replacement has been opened and checked. The backup is also useful if another application still needs the old file.

Without `--replace`, `--output` is required. Existing output paths are not overwritten.

### Older formats and historical exporters

The current lockbox and vault APIs intentionally read only their current native formats. When the CLI encounters an older format, the direct migration command automatically:

1. detects the source format version;
2. installs the exact historical exporter registered for that version from crates.io, if it is not already cached;
3. exports the old native records to an encrypted streaming migration artifact;
4. upgrades the migration schema one step at a time; and
5. imports and validates a new current-format vault or archive.

The current destination is Lockbox container v4. The CLI selects a historical reader for the source and imports its committed logical records directly into the current container; it does not rewrite the source through every intermediate native format. Migration artifact schemas may still be upgraded in steps.

The v4 implementation adds `revault-migrate-archive-v3` for v2/v3 archives and `revault-migrate-vault-v3` for structure-v3 Vaults in v3 containers, alongside the older registered exporters. The `v3` names describe their historical input reader, not the destination format. These helpers use isolated working copies so historical recovery cannot alter the original. They are part of the development branch and must be published with the v4 release before automatic installation can work.

The first migration may therefore require network access and a working Cargo installation. The exporter is cached under the user cache directory and is checked for the expected artifact type, native version, and migration schema before it is used.

If the machine cannot access crates.io, install the matching exporter by some other means and pass its executable path with the advanced `--exporter` option. The exporter must advertise support for the source format and migration schema; the CLI validates that handshake.

### Resuming an interrupted migration

Migration is resumable. The CLI stores an encrypted migration journal and temporary artifacts beside the source. If the process is interrupted, repeat the same command:

```console
lockbox doctor migrate lockbox secrets.lbox --output secrets-migrated.lbox
```

Completed export, upgrade, and import stages are verified and reused. Reusing an imported archive requires a full content comparison, not just a readable header. An incomplete stage is discarded and rebuilt. The CLI refuses to resume when the source path, source format version, or source contents no longer match the saved journal.

If the process stopped during `--replace`, run the same replacement command again. The CLI detects the interrupted replacement and completes the safe rename when the retained backup and validated output are available.

Do not manually remove `.revault-migration-*` directories while a migration is being resumed. After a successful migration, the CLI removes its temporary working directory.

### Manually controlling migration stages

The normal direct command is recommended. The lower-level export, upgrade, import, and verify commands are hidden from ordinary help because they are primarily useful for diagnostics, transferring an encrypted migration artifact between machines, or testing a migration step.

To view them:

```console
lockbox --verbose --help
lockbox doctor migrate vault --help
lockbox doctor migrate lockbox --help
```

The stage commands use encrypted migration artefacts and require an explicit migration artefact passphrase. This is intentional: a manually exported artefact may outlive the source Vault and may be transferred to another machine. For example, to verify an artefact without importing it:

```console
lockbox doctor migrate vault verify vault.migration
lockbox doctor migrate lockbox verify archive.migration
```

For a manually staged migration, keep the artefact files private and transfer the migration passphrase through a separate secure channel. Do not put Vault, Lockbox or migration passwords in command-line arguments, because command arguments may be visible to other processes.

### Troubleshooting

#### The CLI says the format version is unsupported

Run the matching direct command and provide a destination:

```console
lockbox doctor migrate vault --output ./vault-migrated
lockbox doctor migrate lockbox old-secrets.lbox --output ./old-secrets-migrated.lbox
```

If you want the original replaced after validation, use `--replace` instead.

#### The migration cannot open the vault

Check the Vault passphrase and whether the platform credential store is available. You can provide the passphrase explicitly for one invocation:

```console
LOCKBOX_VAULT_PASSWORD="$VAULT_PASSWORD" \
  lockbox doctor migrate vault --output ./vault-migrated
```

#### The migration cannot open the archive

Confirm that you migrated or restored the correct Vault and that all saved Profile keys were imported.

If the Vault does not hold a credential that can open the Lockbox and the Lockbox has password access, provide that password through `LOCKBOX_PASSWORD`:

```console
LOCKBOX_PASSWORD="$ARCHIVE_PASSWORD" \
  lockbox doctor migrate lockbox old-secrets.lbox --output ./old-secrets-migrated.lbox
```

#### The exporter cannot be installed

The current CLI needs the historical exporter for an old native format. Check network access to crates.io and that Cargo is installed. If installation is not possible, obtain the exact exporter binary through your deployment system and pass it with the advanced `--exporter <path>` option.

#### The destination already exists

Choose a new output path. Migration never overwrites an existing destination:

```console
lockbox doctor migrate lockbox secrets.lbox --output secrets-migrated-new.lbox
```

#### The source changed while migration was in progress

The saved source fingerprint no longer matches. Remove or rename the partial destination if necessary, decide which source copy is authoritative, and start a new migration from that unchanged source.
