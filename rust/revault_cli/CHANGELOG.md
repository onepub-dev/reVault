# Changelog

## Unreleased

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
