# Local 0.6.0 integration checkpoint — 2026-10-24

Local `main` contains the October security fixes, format-5 item descriptions,
the #324 mutation regressions, plaintext read-only file opening across the
bindings, and the five October Cargo dependency updates. The code merge is
`7667ab4f`; the security checklist update is `3165b3f4`. None of these commits
has been pushed, published, or deployed. The old Java-only #4142 branch was
superseded by the shared binding implementation and removed locally. The five
Dependabot pull requests remain open on GitHub.

## Local checks

- Locked Rust workspace `cargo check --workspace --all-targets` and Clippy with
  warnings denied passed on the integrated code.
- Both #324 mutation-sequence tests, the format-5 native layout test, and the
  retained migration fixtures passed (three fixtures; one fixture-generation
  test intentionally ignored).
- The binding inventory check and native plaintext read-only regression passed.
  A format-5 plaintext archive made by the current Rust API was read through
  Java, Python, and JavaScript against the integrated native library. Reads and
  byte-range reads matched the source; mutations were refused and the archive
  hash stayed unchanged.
- The dependency update passed locked workspace check and Clippy, 34 compression
  tests, 15 external-source tests, and three retained migration fixtures before
  integration. The `oxiarc-zstd` update is a current-core development dependency;
  frozen historical exporter manifests were not changed.
- The integrated CLI and native binding library built. The CLI reports `0.6.0`.

The CLI could not create an archive in this sandbox: its Session Agent needs a
runtime directory under `/run/user/1000`, which is read-only here, and this
sandbox rejects Unix socket binding under `/tmp`. The current Rust API supplied
the format-5 plaintext fixture for the binding checks instead. This does not
replace a public CLI end-to-end check in the release candidate.

## Still required before release

- Review the eight private advisories against their fixes and assign accurate
  affected and patched versions. Prepare coordinated security notes and the
  key-server deployment and incident-response plan in the
  [security release checklist](../security_release_todo.md).
- Publish and verify the new `revault_migrate_v4` package before any CLI that
  depends on it. Validate migration from the supported older formats using
  packaged exporters and the public CLI, reopening content, credentials,
  metadata, and backups.
- Run the exact candidate's installed-package, platform, binding, format-line,
  and applicable resource checks. The initial `0.6.0` line uses published
  `0.5.0` as a migration source; later `0.6.x` releases must preserve
  bidirectional compatibility within the line.
- Push only as part of the coordinated security release, then seal and validate
  the exact candidate from remote `main`. Publication and production deployment
  each require explicit authorization.
