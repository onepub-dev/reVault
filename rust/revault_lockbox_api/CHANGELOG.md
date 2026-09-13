# Changelog

## Unreleased

- Add the non-default `native-block-layout` experiment for public-API/CLI and
  performance testing of one indexed layout across all protection modes. Normal
  builds retain the existing writer pending migration and performance gates;
  ZIP parity is not yet achieved. Experimental readers preserve the public
  content-key length contract and authenticate repaired-header recovery copies.

- Keep page-flush length accounting current when retrying a partially appended
  file page, so the following page is not preceded by an unintended gap.

- Extend storage source-current checks to external-source cancellation,
  revision/length changes and latched transport failures.

- Avoid copying full file frames merely to size their page allocations. Raw
  page sizes are calculated from framing and payload lengths; compression,
  encryption and stored bytes are unchanged by this optimization.

- Add creation-time `SizePadding::None` and CLI `--no-size-padding` for compact
  small lockboxes. The choice is persisted in native v4 and applies to every
  encryption/signing mode; default size-hiding padding is unchanged. Opting out
  reveals more about stored content lengths and does not remove headers,
  signatures, encryption tags, recovery metadata or reusable free space.
- Refresh the in-memory free index after interrupted-commit cleanup so the
  recovered write handle can immediately account for and reuse reclaimed space.

- Reduce large-file read copying and secure-buffer wiping overhead, and share
  verified raw-page data with seekable readers. These reader improvements also
  apply to existing archives; checksum, encryption and signature checks remain
  in place.
- Allow concurrent Unix positional reads on a shared archive handle while
  retaining exclusive access for writes and truncation.
- Preserve compression of repetitive binary data that previously looked
  incompressible to the entropy heuristic. Keep the requested Zstd encoder
  single-pass: the experimental second encoder was withdrawn because it slowed
  writes. Existing-archive reader improvements do not require recompression.

## 0.0.11 - 2026-09-08

- Give newer lockbox formats direct upgrade guidance instead of suggesting
  migration to an older reader.

## 0.0.10 - 2026-09-06

- Preserve password recipients during content-key replacement and support conversion to read-only handles. Fix atomic replacement for relative paths.

## 0.0.9 - 2026-09-02

- Fix secure page relocation retaining the old physical page length in live
  file references. Repeated packed-file deletion transactions no longer use a
  stale oversized extent that can overwrite adjacent live pages and damage the
  lockbox.
- Add verified host-path imports that hash the exact bytes consumed, allowing
  mirror updates to reject a source file changed after inventory collection.
- Add exact-destination extraction for a selected stored directory, including
  subtree limits, symlink protections, and optional permission restoration.
- Handle an existing non-directory extraction destination consistently with
  the overwrite policy.

## 0.0.8 - 2026-09-01

- Keep the last published transaction sequence separate from sequence numbers
  allocated while staging new pages. Large mirror deletions after an earlier
  cleanup transaction now commit and remain visible after reopening instead of
  being misclassified as unfinished recovery.
- Use “lockbox” consistently in unsupported-format and recovery guidance.

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
