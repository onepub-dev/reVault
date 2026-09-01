# Changelog

## 0.0.12 - 2026-09-01

- Moved format migration under the `doctor` maintenance namespace. Use
  `lockbox doctor migrate vault` and `lockbox doctor migrate lockbox`; the
  former top-level `migrate` command is no longer accepted.
- Fixed large mirror updates being reported as unpersisted after staging many
  additions or recursive removals. Mirror updates now verify the committed
  contents through a fresh open before reporting success.
- Exclude the selected lockbox and its active lock file when mirroring or
  recursively adding a directory that contains the lockbox itself.
- Redraw mirror progress on one terminal line when supported, retain
  line-oriented redirected logs, and add `--quiet` to mirror, add, and recovery
  workflows.
- Make `doctor` report user-facing health states and recovery commands, hide
  implementation counters unless `--verbose` is requested, show human-readable
  sizes, and tell users to open a closed lockbox before rerunning health checks.
- Store the default lockbox in Session Agent state, remember lockboxes by
  absolute canonical paths, and add an explicit command for remembering an
  existing lockbox.
- Install the historical lockbox migration helper alongside `lockbox` and
  `lbx` when installing from source with `cargo xtask install-cli`.

## 0.0.11 - 2026-08-31

- Allow explicit and prompted Vault passphrases to remain usable when the
  optional platform credential service is unavailable, as on headless Linux.
  `LOCKBOX_VAULT_PASSWORD` now takes precedence over the credential store, and
  credential lookup failure falls back to the remaining passphrase sources.
- Refresh CI's current Vault API selector and invalidate pre-format-2 endian
  fixtures so release validation exercises the current formats.

## 0.0.10 - 2026-08-31

- Fixed automatic migration of Vault structure version 2 when it is stored in
  historical Lockbox container format 1. Migration now installs the 0.0.4
  historical Vault exporter, which detects the embedded structure independently
  and reads the corresponding profile records before rebuilding a current
  format Vault.
- Preserve platform credential-store errors instead of treating them as a
  missing or invalid passphrase, and retain an explicitly entered Vault
  passphrase when the Vault requires migration so the migration command can
  continue without an avoidable second prompt.
- Refreshed the packaged Cargo lockfile to use `chacha20` 0.10.2 after 0.10.1
  was yanked. This removes the warning from locked CLI installations and
  includes the upstream correction for an SSE4.1 intrinsic used by the SSE2
  backend's RNG and legacy-counter variants. reVault's Lockbox format and
  encryption behavior are unchanged.

## 0.0.9 - 2026-08-30

- Preserve the advertised Rust 1.88 minimum for fresh Cargo installations by
  using the vault API release that pins its Linux Secret Service dependency to
  the last Rust-1.88-compatible version.

## 0.0.8 - 2026-08-30

- Install the current v1 vault and archive exporter releases during automatic
  migration so schema-2 archive metadata and migration-format 0.0.4 are used.
- Moved the infrequently used `recover` operation under `doctor`. `doctor
  recover` now detects authenticated interrupted cleanup automatically and
  rolls it forward in place; otherwise it salvages readable records into a
  new Lockbox. `--dry-run` remains non-mutating, and the separate transaction
  recovery mode flag has been removed.
- Write-capable Lockbox opens now complete authenticated pending cleanup
  automatically, while `doctor` diagnostics use an explicitly read-only open
  and report the recovery action without changing the Lockbox.
- Improved recovery, migration, session, file, variable, and form command
  coverage, including failure-injection tests for interrupted commits and
  recovery checkpoints.
- Added encrypted Lockbox descriptions. `create --description` records the
  initial purpose, `description get|set|clear` manages it, `doctor LOCKBOX`
  shows it after a successful open, and `vault lockbox list
  --with-description` includes descriptions that can be decrypted. Existing
  encrypted variable storage is reused, so the lockbox format is unchanged.
- Added persistent named `mirror` projects for one-way host-directory updates.
  Projects own non-overlapping lockbox subtrees, store portable include/exclude
  rules and missing-file policy inside the encrypted archive, use `status` as
  the sole preview, and provide scoped variants of every file command. Empty
  sources and large deletion plans retain explicit safeguards. This reuses
  existing encrypted variable records and does not change the lockbox format
  version.
- Top-level help now prints the installed CLI version.
- Corrected `variable move` usage to show one optional lockbox followed by one
  source and destination.
- Missing lockboxes now produce a direct path-specific error from `open`.
- Added regression coverage for completing nested host lockbox paths.
- `variable set --secret` and `form set --secret` now upgrade existing normal
  values to secure storage. Form-field promotion creates a definition revision
  and upgrades that field across records of the same form type. Downgrades
  remain explicit delete-and-recreate operations.
- Added target-first lockbox commands such as `lbx secrets.lbox add ...` and
  `lbx secrets.lbox create`. Existing unambiguous command-first forms remain
  available, but `add` now reserves every positional value for a source and
  selects its lockbox before the command or from the session default. It
  accepts multiple source files, uses `--to` for an optional logical
  destination, supports explicit replacement with `--overwrite`, and
  normalizes relative logical paths before storage.
- Mutating CLI commands now confirm successful outcomes. File imports report
  the number of files added, and create, extract, rename, variable, form,
  access, and developer key operations report what changed.
- `remove` now handles every shell-expanded path, supports quoted archive
  globs, validates a batch before mutation, and prompts once. `*.json` matches
  one directory while `**/*.json` matches recursively; directory removal
  requires `--recursive`.
- Standardized frequent command names and aliases across the CLI: `list`/`ls`,
  `remove`/`rm`, and `move`/`mv`. `rename` remains a full descriptive synonym
  for the top-level `move` operation.
- Added prebuilt Linux, macOS, and Windows release artifacts, Unix and
  PowerShell installers, a Windows MSI, checksums, and build provenance.
- Added `--version` so installers and package managers can verify the installed
  CLI without opening a vault.
- Restored the minimum supported Rust version to Rust 1.88.
- Renamed vault identities to profiles throughout the CLI. Profile commands,
  output labels, completion, publishing, recovery backups, and file extensions
  now consistently use `profile`; the former `identity` forms are not retained.
- Added commandline completion for Bash, Zsh, Fish, PowerShell, and Elvish.
  - Added `completion generate`, `completion install`, and
  `completion uninstall` to aid installing the completion tooling.
  - Added context-aware completion for vault profiles, contacts, reusable
  forms definitions, and unlocked archive variables, forms and paths.
  - Added graceful fallback to static command completion when the vault or
  archive metadata is unavailable.
- Added encrypted, resumable vault and archive migration commands. Normal
  migrations upgrade directly to the latest format, require either an explicit
  output or `--replace`, validate before replacement, and retain a versioned
  backup when replacing the source.
- Added hidden advanced migration export, upgrade, import, and verify commands.
  Historical native readers are obtained as exact-version migration exporters
  from crates.io instead of being retained in the current CLI.
  Exporters are spawned behind a versioned, length-prefixed secret protocol and
  must report matching capabilities before execution.
- Direct migrations now generate a random zeroizing artifact key instead of
  prompting for a migration password. The key is retained only in the
  source-protected resumability journal. Archive migration now requires an
  initialized, current-format vault so the new archive can be signed with the
  current owner signing key. The hidden/manual artifact commands still
  require an explicit migration passphrase, supplied by prompt, environment,
  or the export stdin option.
- Archive migration now supplies the already-validated vault credential to the
  historical exporter through secret IPC. The exporter tries all migrated
  profile key generations before using the password-only archive fallback.
- Running session agents now report their protocol and implementation version.
  A CLI upgrade automatically replaces an incompatible agent and clears its
  old cache on the next agent operation.
