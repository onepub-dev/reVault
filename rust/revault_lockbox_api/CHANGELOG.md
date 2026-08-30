# Changelog

## 0.0.7 - 2026-08-30

- Replace the pinned `zstd-rs` Git revision with the published pure-Rust
  `zstd-complete` 0.1.0 crate. Compression remains parallelized by reVault
  across independent frames; the crate's optional internal multithreading is
  intentionally not enabled.
- Enforce owner-signing-key continuity across the authenticated commit chain.
  Opens reject chains that switch signing identities, hybrid signatures must
  contain exactly one Ed25519 and one ML-DSA-65 signature, and commits reject a
  signing key that does not match the established owner.
- Make post-publication cleanup crash-safe and automatically resumable by
  write-capable opens. Explicit read-only opens remain non-mutating and return
  authenticated recovery status for diagnostic or controlled recovery tools.
- Harden extraction against pre-existing symlink components in destination
  paths for sequential, parallel, directory, file, and symlink extraction.
- Add exhaustive storage-failure, torn-header, recovery-checkpoint, signer
  substitution, corrupt-manifest, and symlink-parent regression tests.
- Keep historical archive formats out of the core API; current-format probes
  return unsupported-version errors for migration tooling to handle.

## 0.0.4

- Added persistent mirror-project metadata and core-enforced exclusive subtree
  ownership. Ordinary mutation APIs cannot change managed paths; trusted
  mirror orchestration receives a mutation scope that cannot escape its
  project destination. The metadata uses existing encrypted variables and does
  not change the archive format.
- Variable path components may use a single leading dot for encrypted
  hidden/internal metadata namespaces. The unsafe `.` and `..` components
  remain invalid.
- Normal variables and form fields can now be promoted to secret storage.
  Form-field promotion creates a new definition revision and upgrades existing
  values across records of that form type; secret-to-normal changes remain
  prohibited in place.
- Restored the minimum supported Rust version to Rust 1.88.
- Removed the `sysinfo` dependency. Automatic page-cache sizing now uses a
  conservative platform default, and Windows stale-lock detection uses native
  process APIs.
- Renamed vault identity access labels to profile access labels. Named access
  entries now use the `profile:` prefix; the former `identity:` prefix is not
  retained.
- Added stable archive-format probing and actionable unsupported-version errors.
- Added narrowly scoped migration APIs for streaming logical archive contents
  and access material into a new native archive. Imported archives create a new
  commit/signature chain; old public commit and signature records are not
  preserved.
