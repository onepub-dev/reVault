# Candidate C source-preserving compaction

2026-09-27. This extends the test-only file adapter after `542218ab`.
It does not select candidate C, enable a production writer or change the
[acceptance budgets](../../archive_v4_evaluation.md).

## Behaviour

Compaction authenticates and fully decodes every source file, including normally
lazy unsigned modes. It copies each live physical pack once into a private
replacement, verifies its stored digest, rebuilds both metadata copies and the
allocation map, and discards free/history bytes. Relative fragment descriptors,
file identities, logical content, codec and protection choices survive unchanged.
Ciphertext is copied unchanged; no new plaintext is encrypted under a reused nonce.

The replacement publication advances the source generation by one and commits to
its predecessor. It never resets an existing archive to generation one or briefly
publishes an empty same-identity archive. Fresh validation components decode and
hash every replacement file before installation. An initialized preparation region
allows subsequent mutations.

The file adapter holds an exclusive source lock, preserves permission bits and
refuses symlink paths. It renames only the verified replacement, then syncs the
parent directory. Before rename, errors erase and remove the private output;
cleanup errors remain errors. After rename, a directory-sync error reports uncertain
durability and requires reopening; it must never erase the installed archive.
A terminated process may leave a private temporary file before installation.
No automatic promotion or discovery of such files is implemented.

Unsupported key/access trees are refused before destination writes. This is
file-only compaction; it must not be used to silently discard other record types.

## Correctness evidence

* All 16 encryption/signing/codec/padding combinations preserve identity, exact
  content, generation linkage and writable allocation state after compaction.
* All 90 injected replacement mutations fail safely: the original remains
  byte-identical and the replacement is cleared.
* Four representative protection modes install real files, retain permissions,
  then add/remove records and independently reopen and compare stored bytes.
* Sixteen process deaths cover copied payloads, completed dependencies, verified
  replacement and completed rename across four protection modes. Before rename,
  the original path contains the exact original bytes. After rename, it contains
  a complete readable replacement linked to the original generation.
* Nonempty outputs and damaged source payloads in normally lazy modes are refused.
  Missing signing capability at late publication cleans up the replacement.
  Unsupported access roots and symlink paths are refused.

The process tests exit without Rust destructors; they establish process-crash
behaviour, not power-loss durability. Existing publication/allocator power-loss
simulations exercise their own protocols and do not qualify platform rename or
parent-directory persistence. Candidate C has no public CLI writer, so these tests
explicitly use its internal adapter; they are not claimed as CLI E2E coverage.

The full default-layout suite with `external-source` passed: 459 unit tests,
all executed integration tests and 10 doctests (9 unit probes and one integration
probe intentionally ignored). Strict native/external-source all-target Clippy
passed. Logs and SHA-256 sums are retained beside this document. This does not
claim the known native-feature recovery failures were fixed.

## Reproduction and remaining work

From the Rust workspace:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib compaction -- --nocapture
cargo test -p revault_lockbox_api --release --features external-source
cargo clippy -p revault_lockbox_api --features native-block-layout,external-source --all-targets -- -D warnings
```

The existing Linux resource probe accepts `REVAULT_CANDIDATE_PHASE=compact` for
unsigned fixtures and reports elapsed/CPU time, peak RSS and source/replacement
sizes. It performs independent persisted-byte verification outside timing.
Replacement construction only appends before rename, so peak extra logical file
length equals final replacement length; this excludes filesystem allocation
rounding and existing backups. Signed cases use ephemeral owner keys in the
correctness tests; the performance protocol never persists a private signing key.

Resource measurements, preflight disk-headroom reporting, complete access/record
semantics, cross-platform replacement tests and storage power-loss qualification
remain outstanding. Passing these local tests does not close A1, A3, A4 or A5.
