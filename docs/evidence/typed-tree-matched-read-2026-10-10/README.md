# Matched-toolchain typed-tree read results

Protocol declared before measurement on 2026-10-10; fixed batch completed on
source `a47f686f` with Rust 1.88.0. All six cases fail ZIP parity.

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
The measured batch used fresh binaries built after hook formatting,
with separate identities; preparation timings are excluded.

## Completed results

All cases completed 30 measured groups and three warmup groups on CPU 2, each
containing ZIP, packed C and typed tree: 540 measured and 54 warmup observations.
Every observation passed independent byte verification. Frozen executable/source
hashes and source/archive identities remained unchanged. No owned build or test
overlapped timing. Host load and CPU affinity are retained, not assumed idle.

Ratios below are total elapsed tree/control time with paired-bootstrap 95%
intervals. Lower is better; every tree/ZIP interval is entirely above one.

| Case | Tree/ZIP [95% CI] | Tree/packed C [95% CI] | Tree open/read/total median (ms) | Worker peak RSS median (KiB) |
| --- | --- | --- | --- | --- |
| 512 × 4 KiB compressible stream | 1.066 [1.035, 1.091] | 0.0165 [0.0161, 0.0168] | 1.343 / 1.931 / 3.283 | 10,548 |
| 8 MiB random raw stream | 3.883 [3.774, 3.981] | 0.989 [0.983, 0.997] | 0.747 / 5.317 / 6.076 | 9,924 |
| 8 MiB patterned compressed stream | 2.593 [2.572, 2.614] | 0.655 [0.651, 0.660] | 0.633 / 3.366 / 3.983 | 10,772 |
| 8 MiB random raw midpoint 4 KiB | 10.887 [10.523, 11.247] | 0.914 [0.880, 0.941] | 0.756 / 0.047 / 0.805 | 9,924 |
| 64 MiB random raw stream | 3.707 [3.669, 3.741] | 0.976 [0.971, 0.981] | 2.050 / 42.467 / 44.498 | 10,436 |
| 64 MiB random raw midpoint 4 KiB | 28.005 [27.266, 28.825] | 0.812 [0.799, 0.828] | 2.038 / 0.047 / 2.091 | 10,440 |

Separate medians need not sum exactly. Median tree CPU times are respectively
3.284, 6.077, 3.985, 0.806, 44.498 and 2.092 ms. Exact results are in
[metrics-summary.json](results/metrics-summary.json); per-observation open/read,
CPU and RSS values are in each `results/measure/<case>/samples.jsonl`.

Opening dominates the two range cases, making open-path profiling the next useful
diagnostic. This is an inference from timing, not a sampled CPU attribution.
The 64 MiB case now runs without raising admission caps. The small-file result
does not reproduce the earlier ZIP pass; retain both batches with their distinct
source/host identities. This batch compares current components and does not
isolate the effect of a particular change or compiler version. The October 2
pinned-1.88 batch remains historical evidence alongside its earlier cross-compiler
exploration.

## Reproduction and retained evidence

- [Hash and source identities before](results/hash-before.json) and
  [after](results/hash-after.json) show the frozen Rust 1.88.0 participants.
- [Final audit](results/final-audit.json) records every verification, inventory,
  archive/public-key size and digest; [affinity](results/affinity-audit.json)
  records CPU 2 for every case.
- [Host before](results/host-before.json) and [after](results/host-after.json)
  record the Ryzen 7 3700X, kernel and load context.
- Executed setup, batch and audit Dart scripts are retained as `.dart.txt`,
  alongside build/Clippy/setup/timing logs, summaries and raw observations.
  These scripts record original absolute paths and are evidence, not installed tools.
- Fixture payloads and binaries are excluded; inventories and hashes identify
  them. Setup smoke observations under `results/cases/` are not measured evidence.

No passing ZIP read claim, public write claim, public CLI memory qualification,
PGP comparison, scale/aging/migration qualification or complete-format acceptance
follows from this batch. Checksums, padding validation, owner checks, wiping and
verification-before-exposure remain unchanged.
