# Changelog

## Unreleased

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
- Raised the minimum supported Rust version from 1.88 to 1.95.
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
