# Matched-toolchain typed-tree read protocol

Declared before measurement on 2026-10-10. Status: harness validation and builds
complete; no qualifying measurements or performance claims in this checkpoint.

The October 2 comparisons used different Rust compiler versions for typed trees
and retained ZIP/packed-C controls. Preserve those observations with their stated
limits. This batch rebuilds every participant with Rust 1.88.0 and the same current
lockfile. Packed C and typed-tree protocol adapters use the same frozen core test
executable and the same sampling/verification loop. ZIP uses the existing common
runner, rebuilt with the same compiler. This compares current components, not the
effect of recent credential changes or a historical before/after optimization.

The direct tree exporter replaces the previous harness's dense intermediate, so
the previously blocked raw 64 MiB case is included without increasing admission
caps. Fixture creation remains untimed preparation; it is not a public write sample.

## Fixed batch

| Case | Corpus | Codec | Read | Unit |
| --- | --- | --- | --- | --- |
| 512 × 4 KiB | mixed | compressed | whole files | 256 KiB |
| 1 × 8 MiB | seeded random | raw | whole file | 64 KiB |
| 1 × 8 MiB | pattern | compressed | whole file | 256 KiB |
| 1 × 8 MiB | seeded random | raw | 4 KiB midpoint | 64 KiB |
| 1 × 64 MiB | seeded random | raw | whole file | 64 KiB |
| 1 × 64 MiB | seeded random | raw | 4 KiB midpoint | 64 KiB |

The existing generator's named `mixed` corpus alternates 256 KiB blocks within
each file. Its 4 KiB files therefore contain only the patterned block. Preserve
this historical workload for comparability; the small-file case is compressible
and does not represent a mixture of compressible and random small files.

ZIP uses the pinned `zip` 8.6.0 crate: Stored for raw cases, Deflate through
flate2 at its default level 6 for compressed cases. Candidate files use the
existing `Compression::default()` Zstd level 3 with their raw fallback. Full-file
ZIP reads retain CRC checking; candidate checksums, padding checks, wiping and
verification-before-exposure remain unchanged.

All measured cases are plaintext, unsigned, default padded. Each has 30 paired
observations after three warmup pairs, one pass/worker, fresh processes/handles,
warm OS cache, and alternating ZIP/C/tree order through `compare-existing`.
Use CPU 2 if available; otherwise declare the selected allowed CPU before timing.
The existing fixed-seed 10,000-resample paired log-ratio bootstrap supplies 95%
intervals. Report each case separately, including failures; do not pool or repeat
the batch to obtain a passing interval. Raw range ZIP reads omit full-entry CRC.

Correctness smokes cover all 16 protection/codec/padding combinations on small
fixtures plus the new 64 MiB fixture. Their timings are setup diagnostics and are
excluded from this batch. Any changed workload or failed batch is retained and
requires a separately recorded new experiment, not silent replacement.

Finish all owned builds/tests before measured runs. Freeze source/lockfile/compiler
and executable identities; retain corpus inventories and archive/public-key hashes.
The comparison checks exact inventories and all artifact hashes before and after
sampling. Byte-for-byte whole-file and requested-range verification happens outside
timing. The outer batch script must compare the frozen core, adapters and runner hashes
before and after the batch; the Rust comparison checks archive/source identities.

Report open, read and total elapsed time, CPU and whole-worker peak RSS separately.
The worker RSS is not total public CLI RSS or an incremental empty-open budget.
This does not qualify G3 scale, G11 write/PGP/CLI memory targets, signed/encrypted
performance, aging, recovery, migration, or any complete-format gate. Existing
native recovery failures and credential overflow/mutation remain outstanding.
## Harness validation

The [preparation report](preparation/prepare-report.txt) records pinned builds,
strict Clippy, all 16 small mode/codec/padding smokes, and independent 64 MiB
stream/range checks. Raw invocation logs, scripts and source/binary identities are
retained. An initial build command incorrectly passed `--release` to `cargo bench`;
Cargo refused it, and the documented optimized bench command then succeeded.
The forthcoming measured batch uses fresh binaries built after hook formatting,
with separate identities; preparation timings are excluded.
