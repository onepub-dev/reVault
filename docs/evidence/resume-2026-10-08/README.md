# Resumed archive checkpoint — 2026-10-08

This retains the accumulated October 2 implementation on top of `4744b635`.
It does not select format 4, activate the experimental writer, or qualify a release.
The [plan](../../archive_v4_plan.md) and [scorecard](../../archive_v4_evaluation.md)
retain the unresolved large/range ZIP, public-semantics, native-recovery and
compatibility gates. Earlier measurements keep their original scope.

## Revalidation before formatting

From `rust/`, using pinned Rust 1.88.0:

- `cargo test -p revault_lockbox_api --release --features external-source --lib compression::`: **21 passed**.
- `cargo test -p revault_lockbox_api --release --features external-source --lib file_format::`: **241 passed, 7 ignored**.
- `cargo clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings`: **passed**.
- `cargo test --manifest-path vendor/zstd-complete/Cargo.toml --release --test workspaces`: **13 passed, 1 ignored**.

The exhaustive FSE equivalence unit test also passes in the historical default
unoptimized test profile (**1 passed**, 0.43 seconds test runtime; 1 minute
36 seconds compile). An unnecessary release unit-test harness compile was
interrupted after 16 minutes 22 seconds; its Cargo and rustc children exited.
The canceled compile is not a test failure or pass. Optimized decoder execution
is covered by the release workspace and archive suites above.

Logs and a pre-format source manifest accompany this page. No performance batch
ran during these checks. Initial attempts stopped before execution because the
environment lacked `cc`; build-essential was subsequently installed. Compression
and format tests used GCC 15.2.0 from the temporary official-package extraction at
`/tmp/revault-build-tools/sysroot/usr/bin`; Clippy and vendor checks used the
subsequently installed `/usr/bin/cc` (GCC 15.2.0-16ubuntu1).

## Reproducible vendor checks

The published vendor package omitted files referenced by existing `include_bytes!`
tests. Eleven original fixtures were restored from the clean corresponding paths
in `/home/bsutton/git/zstd-rs/ruzstd`, revision
`298793ed8fdb2a098312ce7dc38674947c63551f`. Their [hashes](vendor-fixtures.sha256)
record the copied bytes. They are public upstream codec corpus/service fixtures,
not Lockboxes or user credentials. No upstream implementation changes were imported.

## Frozen evidence and available artifacts

Fourteen historical Rust source snapshots are stored with `.rs.txt` suffixes so
the required staged-Rust formatting hook cannot rewrite measured source bytes.
The [filename mapping](frozen-source-names.tsv) records original evidence paths,
retained paths and unchanged hashes. Restore the original suffix when rebuilding
an isolated historical harness. Original source-path provenance manifests and
source tarballs remain unchanged. Only references to renamed evidence artifacts
were updated.

The historical `/tmp/revault-*` frozen binaries, corpora and full source snapshots
were unavailable in this resumed environment. Repository evidence remains intact;
this does not re-create or revalidate the historical measurements. A new cost
comparison must rebuild matched baseline/current executables and identical
fixtures, record the current host/toolchain, and retain its own artifacts.
